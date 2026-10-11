use std::{
    ffi::{OsStr, OsString},
    path::{Path, PathBuf},
    process::{Command, Output},
};

use presolve_executor_sdk::{
    ContainerExecutor, ContainerExecutorError, ContainerHandle, ContainerImageDescriptor,
    ContainerImageResolveError, ContainerImageResolver, ContainerImageSource,
    ContainerLaunchRequest, ContainerNetworkMode,
};

const IMAGE_LIST_FORMAT: &str = "{{.Repository}}\t{{.Digest}}";

/// Docker CLI implementation of Presolve's local container backend.
///
/// Image resolution is deliberately local-cache-only. Presolve does not perform
/// implicit registry pulls: an immutable descriptor must already be present in
/// the active Docker daemon before this backend can resolve it.
#[derive(Debug, Clone)]
pub struct DockerCliBackend {
    program: PathBuf,
}

impl Default for DockerCliBackend {
    fn default() -> Self {
        Self::new("docker")
    }
}

impl DockerCliBackend {
    /// Creates a Docker CLI backend using `program`.
    ///
    /// The default backend uses `docker` from `PATH`. Supplying an explicit
    /// program path is useful for custom Docker-compatible CLI installations.
    #[must_use]
    pub fn new(program: impl Into<PathBuf>) -> Self {
        Self {
            program: program.into(),
        }
    }

    /// Returns the configured Docker CLI executable.
    #[must_use]
    pub fn program(&self) -> &Path {
        &self.program
    }

    fn output<I, S>(&self, args: I) -> Result<Output, std::io::Error>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        Command::new(&self.program).args(args).output()
    }
}

impl ContainerImageResolver for DockerCliBackend {
    fn resolve(
        &self,
        image: &ContainerImageDescriptor,
    ) -> Result<ContainerImageSource, ContainerImageResolveError> {
        let output = self
            .output([
                OsStr::new("image"),
                OsStr::new("ls"),
                OsStr::new("--digests"),
                OsStr::new("--no-trunc"),
                OsStr::new("--format"),
                OsStr::new(IMAGE_LIST_FORMAT),
            ])
            .map_err(|error| {
                ContainerImageResolveError::unavailable(format!(
                    "could not execute Docker CLI `{}`: {error}",
                    self.program.display()
                ))
            })?;

        if !output.status.success() {
            return Err(ContainerImageResolveError::unavailable(format!(
                "Docker image inventory failed: {}",
                stderr_message(&output)
            )));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let source = resolve_local_source(&stdout, image.digest()).ok_or_else(|| {
            ContainerImageResolveError::not_found(format!(
                "Docker has no local repository digest matching `{}`; pull or load the image first",
                image.digest()
            ))
        })?;

        Ok(ContainerImageSource::new(source))
    }
}

impl ContainerExecutor for DockerCliBackend {
    fn launch(
        &self,
        request: &ContainerLaunchRequest,
    ) -> Result<ContainerHandle, ContainerExecutorError> {
        validate_source_digest(request)?;

        let source = request.source().reference();
        let inspect = self
            .output([
                OsStr::new("image"),
                OsStr::new("inspect"),
                OsStr::new(source),
            ])
            .map_err(|error| {
                ContainerExecutorError::unavailable(format!(
                    "could not execute Docker CLI `{}`: {error}",
                    self.program.display()
                ))
            })?;

        if !inspect.status.success() {
            return Err(ContainerExecutorError::unavailable(format!(
                "Docker image `{source}` is no longer available locally: {}",
                stderr_message(&inspect)
            )));
        }

        let arguments = build_run_arguments(request)?;
        let output = self.output(&arguments).map_err(|error| {
            ContainerExecutorError::unavailable(format!(
                "could not execute Docker CLI `{}`: {error}",
                self.program.display()
            ))
        })?;

        if !output.status.success() {
            return Err(ContainerExecutorError::internal(format!(
                "Docker container launch failed: {}",
                stderr_message(&output)
            )));
        }

        let id = String::from_utf8_lossy(&output.stdout)
            .lines()
            .next()
            .unwrap_or_default()
            .trim()
            .to_owned();

        if id.is_empty() {
            return Err(ContainerExecutorError::internal(
                "Docker reported a successful detached launch without a container ID",
            ));
        }

        Ok(ContainerHandle::new(id))
    }

    fn stop(&self, handle: &ContainerHandle) -> Result<(), ContainerExecutorError> {
        let output = self
            .output([
                OsStr::new("container"),
                OsStr::new("rm"),
                OsStr::new("--force"),
                OsStr::new(handle.id()),
            ])
            .map_err(|error| {
                ContainerExecutorError::unavailable(format!(
                    "could not execute Docker CLI `{}`: {error}",
                    self.program.display()
                ))
            })?;

        if !output.status.success() {
            return Err(ContainerExecutorError::internal(format!(
                "Docker could not remove container `{}`: {}",
                handle.id(),
                stderr_message(&output)
            )));
        }

        Ok(())
    }
}

fn resolve_local_source(listing: &str, digest: &str) -> Option<String> {
    let mut matches = listing
        .lines()
        .filter_map(|line| {
            let (repository, listed_digest) = line.split_once('\t')?;

            if repository == "<none>" || listed_digest != digest {
                return None;
            }

            Some(format!("{repository}@{digest}"))
        })
        .collect::<Vec<_>>();

    matches.sort();
    matches.into_iter().next()
}

fn validate_source_digest(request: &ContainerLaunchRequest) -> Result<(), ContainerExecutorError> {
    let source = request.source().reference();
    let Some((_, source_digest)) = source.rsplit_once('@') else {
        return Err(ContainerExecutorError::invalid_request(format!(
            "Docker image source `{source}` is not digest-pinned"
        )));
    };

    if source_digest != request.image().digest() {
        return Err(ContainerExecutorError::image_mismatch(format!(
            "Docker image source `{source}` does not match required digest `{}`",
            request.image().digest()
        )));
    }

    Ok(())
}

fn build_run_arguments(
    request: &ContainerLaunchRequest,
) -> Result<Vec<OsString>, ContainerExecutorError> {
    let mut arguments = vec![
        OsString::from("run"),
        OsString::from("--detach"),
        OsString::from("--pull=never"),
        OsString::from("--label"),
        OsString::from(format!("dev.presolve.workload={}", request.workload())),
        OsString::from("--label"),
        OsString::from(format!(
            "dev.presolve.image-digest={}",
            request.image().digest()
        )),
    ];

    if let Some(memory_mib) = request.resources().memory_mib() {
        arguments.push(OsString::from("--memory"));
        arguments.push(OsString::from(format!("{memory_mib}m")));
    }

    if let Some(cpu_millis) = request.resources().cpu_millis() {
        arguments.push(OsString::from("--cpus"));
        arguments.push(OsString::from(format_cpu_millis(cpu_millis)));
    }

    match request.network().outbound() {
        ContainerNetworkMode::Deny => {
            arguments.push(OsString::from("--network"));
            arguments.push(OsString::from("none"));
        }

        ContainerNetworkMode::AllowAll => {}

        ContainerNetworkMode::AllowList => {
            return Err(ContainerExecutorError::invalid_request(
                "Docker CLI backend cannot faithfully enforce Presolve outbound allow-list semantics",
            ));
        }
    }

    arguments.push(OsString::from(request.source().reference()));

    Ok(arguments)
}

fn format_cpu_millis(cpu_millis: u32) -> String {
    let whole = cpu_millis / 1_000;
    let remainder = cpu_millis % 1_000;

    if remainder == 0 {
        return whole.to_string();
    }

    let fractional = format!("{remainder:03}").trim_end_matches('0').to_owned();

    format!("{whole}.{fractional}")
}

fn stderr_message(output: &Output) -> String {
    let message = String::from_utf8_lossy(&output.stderr).trim().to_owned();

    if message.is_empty() {
        format!("Docker exited with status {}", output.status)
    } else {
        message
    }
}

#[cfg(test)]
mod tests {
    use presolve_executor_sdk::{
        ContainerImageSource, ContainerNetworkPolicy, ContainerResourceLimits,
    };

    use super::*;

    const DIGEST: &str = "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";

    fn request(network: ContainerNetworkMode) -> ContainerLaunchRequest {
        ContainerLaunchRequest::new(
            "api",
            ContainerImageDescriptor::new(
                "application/vnd.oci.image.manifest.v1+json",
                DIGEST,
                1_024,
            ),
            ContainerImageSource::new(format!("registry.example/acme/api@{DIGEST}")),
            ContainerResourceLimits::new(Some(256), Some(500)),
            ContainerNetworkPolicy::new(network, Vec::new()),
        )
    }

    #[test]
    fn local_source_resolution_is_deterministic() {
        let listing = format!(
            "z.example/acme/api\t{DIGEST}\n\
             <none>\t{DIGEST}\n\
             a.example/acme/api\t{DIGEST}\n\
             unrelated.example/api\tsha256:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb\n"
        );

        assert_eq!(
            resolve_local_source(&listing, DIGEST),
            Some(format!("a.example/acme/api@{DIGEST}"))
        );
    }

    #[test]
    fn deny_network_launch_is_digest_pinned_and_resource_limited() {
        let arguments = build_run_arguments(&request(ContainerNetworkMode::Deny))
            .expect("request should build");

        assert_eq!(
            arguments,
            [
                "run",
                "--detach",
                "--pull=never",
                "--label",
                "dev.presolve.workload=api",
                "--label",
                &format!("dev.presolve.image-digest={DIGEST}"),
                "--memory",
                "256m",
                "--cpus",
                "0.5",
                "--network",
                "none",
                &format!("registry.example/acme/api@{DIGEST}"),
            ]
            .map(OsString::from)
        );
    }

    #[test]
    fn allow_all_uses_docker_default_networking() {
        let arguments = build_run_arguments(&request(ContainerNetworkMode::AllowAll))
            .expect("allow-all request should build");

        assert!(!arguments.iter().any(|argument| argument == "--network"));
    }

    #[test]
    fn allow_list_is_rejected_instead_of_weakened() {
        let Err(error) = build_run_arguments(&request(ContainerNetworkMode::AllowList)) else {
            panic!("Docker adapter must reject unsupported allow-list semantics");
        };

        assert_eq!(
            error.code(),
            presolve_executor_sdk::ContainerExecutorErrorCode::InvalidRequest
        );
    }

    #[test]
    fn mismatched_source_digest_is_rejected() {
        let mut request = request(ContainerNetworkMode::Deny);
        request = ContainerLaunchRequest::new(
            request.workload(),
            request.image().clone(),
            ContainerImageSource::new(
                "registry.example/acme/api@sha256:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
            ),
            request.resources(),
            request.network().clone(),
        );

        let Err(error) = validate_source_digest(&request) else {
            panic!("mismatched source digest should fail");
        };

        assert_eq!(
            error.code(),
            presolve_executor_sdk::ContainerExecutorErrorCode::ImageMismatch
        );
    }

    #[test]
    fn cpu_millis_are_rendered_without_float_rounding() {
        assert_eq!(format_cpu_millis(125), "0.125");
        assert_eq!(format_cpu_millis(500), "0.5");
        assert_eq!(format_cpu_millis(1_000), "1");
        assert_eq!(format_cpu_millis(1_250), "1.25");
    }
}
