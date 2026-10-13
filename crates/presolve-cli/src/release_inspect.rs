use std::{
    ffi::OsStr,
    fmt::{self, Write as _},
    fs,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result, bail};
use presolve_bundle::{PresolveBundle, WorkloadManifest};
use presolve_contract::OutboundNetworkMode;

#[derive(Debug)]
pub(crate) struct ReleaseDocument {
    pub(crate) path: PathBuf,
    pub(crate) encoded_size: usize,
    pub(crate) bundle: PresolveBundle,
}

pub(crate) fn load_release(start: &Path, requested: &Path) -> Result<ReleaseDocument> {
    let unresolved = if requested.is_absolute() {
        requested.to_owned()
    } else {
        start.join(requested)
    };

    if unresolved.extension().and_then(OsStr::to_str) != Some("presolved") {
        bail!(
            "release inspection requires a `.presolved` file, got `{}`",
            unresolved.display()
        );
    }

    let metadata = fs::metadata(&unresolved)
        .with_context(|| format!("could not inspect release `{}`", unresolved.display()))?;

    if !metadata.is_file() {
        bail!("release `{}` is not a regular file", unresolved.display());
    }

    let path = unresolved
        .canonicalize()
        .with_context(|| format!("could not resolve release `{}`", unresolved.display()))?;

    let bytes =
        fs::read(&path).with_context(|| format!("could not read release `{}`", path.display()))?;
    let encoded_size = bytes.len();

    let bundle = PresolveBundle::decode(&bytes)
        .with_context(|| format!("invalid `.presolved` release `{}`", path.display()))?;

    Ok(ReleaseDocument {
        path,
        encoded_size,
        bundle,
    })
}

pub(crate) fn render_release(document: &ReleaseDocument) -> Result<String> {
    let manifest = document.bundle.manifest();
    let application = manifest.application();
    let requirements = manifest.requirements();
    let release_digest = document
        .bundle
        .release_digest()
        .context("could not compute canonical application release digest")?;

    let mut output = String::new();

    line(&mut output, format_args!("Presolve release"));
    line(
        &mut output,
        format_args!("  file: {}", document.path.display()),
    );
    line(
        &mut output,
        format_args!("  encoded size: {} bytes", document.encoded_size),
    );
    line(
        &mut output,
        format_args!("  bundle version: {}", manifest.bundle_version()),
    );
    line(
        &mut output,
        format_args!("  release digest: {release_digest}"),
    );
    line(
        &mut output,
        format_args!(
            "  application: {} {}",
            application.name(),
            application.version()
        ),
    );
    line(
        &mut output,
        format_args!("  contract version: {}", requirements.contract_version()),
    );

    if let Some(description) = application.description() {
        line(&mut output, format_args!("  description: {description}"));
    }

    render_workloads(&mut output, manifest.workloads());
    render_capabilities(&mut output, requirements.capabilities());
    render_resources(&mut output, requirements);
    render_network(&mut output, requirements);
    render_topology(&mut output, manifest);

    Ok(output)
}

fn render_workloads(output: &mut String, workloads: &[WorkloadManifest]) {
    line(output, format_args!("  workloads:"));

    for workload in workloads {
        line(
            output,
            format_args!("    {} -> {}", workload.name(), workload.kind()),
        );

        for artifact in workload.artifacts() {
            line(output, format_args!("      artifact: {}", artifact.path()));
            line(
                output,
                format_args!("        media type: {}", artifact.media_type()),
            );
            line(
                output,
                format_args!("        digest: {}", artifact.digest()),
            );
            line(
                output,
                format_args!("        size: {} bytes", artifact.size()),
            );
        }

        if let Some(image) = workload.oci_image() {
            line(output, format_args!("      OCI image:"));
            line(
                output,
                format_args!("        media type: {}", image.media_type()),
            );
            line(output, format_args!("        digest: {}", image.digest()));
            line(output, format_args!("        size: {} bytes", image.size()));
        }
    }
}

fn render_capabilities(output: &mut String, capabilities: &[presolve_bundle::CapabilityManifest]) {
    line(output, format_args!("  capabilities:"));

    if capabilities.is_empty() {
        line(output, format_args!("    none"));
        return;
    }

    for capability in capabilities {
        let requirement = if capability.optional() {
            "optional"
        } else {
            "required"
        };

        line(
            output,
            format_args!(
                "    {} {} ({requirement})",
                capability.interface(),
                capability.version()
            ),
        );

        if !capability.required_features().is_empty() {
            line(
                output,
                format_args!(
                    "      required features: {}",
                    capability.required_features().join(", ")
                ),
            );
        }

        if !capability.preferred_features().is_empty() {
            line(
                output,
                format_args!(
                    "      preferred features: {}",
                    capability.preferred_features().join(", ")
                ),
            );
        }
    }
}

fn render_resources(output: &mut String, requirements: &presolve_bundle::RequirementsManifest) {
    let resources = requirements.resources();

    line(output, format_args!("  resources:"));

    match resources.memory_mib() {
        Some(memory_mib) => line(output, format_args!("    memory: {memory_mib} MiB")),
        None => line(output, format_args!("    memory: unspecified")),
    }

    match resources.cpu_millis() {
        Some(cpu_millis) => line(output, format_args!("    cpu: {cpu_millis}m")),
        None => line(output, format_args!("    cpu: unspecified")),
    }
}

fn render_network(output: &mut String, requirements: &presolve_bundle::RequirementsManifest) {
    let network = requirements.network();

    line(output, format_args!("  network:"));
    line(
        output,
        format_args!("    outbound: {}", outbound_mode(network.outbound())),
    );

    if !network.allow().is_empty() {
        line(
            output,
            format_args!("    allow: {}", network.allow().join(", ")),
        );
    }
}

fn render_topology(output: &mut String, manifest: &presolve_bundle::BundleManifest) {
    line(output, format_args!("  interfaces:"));

    if manifest.interfaces().is_empty() {
        line(output, format_args!("    none"));
    } else {
        for interface in manifest.interfaces() {
            line(
                output,
                format_args!("    {} -> {}", interface.name(), interface.kind()),
            );
        }
    }

    line(output, format_args!("  relationships:"));

    if manifest.relationships().is_empty() {
        line(output, format_args!("    none"));
    } else {
        for relationship in manifest.relationships() {
            line(
                output,
                format_args!(
                    "    {} -> {} via {}",
                    relationship.from(),
                    relationship.to(),
                    relationship.interface()
                ),
            );
        }
    }
}

const fn outbound_mode(mode: OutboundNetworkMode) -> &'static str {
    match mode {
        OutboundNetworkMode::Deny => "deny",
        OutboundNetworkMode::AllowList => "allow_list",
        OutboundNetworkMode::AllowAll => "allow_all",
    }
}

fn line(output: &mut String, arguments: fmt::Arguments<'_>) {
    output
        .write_fmt(arguments)
        .expect("writing release inspection to String should not fail");
    output.push('\n');
}

#[cfg(test)]
mod tests {
    use std::{
        env,
        sync::atomic::{AtomicU64, Ordering},
        time::{SystemTime, UNIX_EPOCH},
    };

    use presolve_bundle::{
        ComponentArtifactInput, OCI_IMAGE_MANIFEST_MEDIA_TYPE, OciImageArtifactInput,
        WorkloadArtifactInput,
    };
    use presolve_contract::parse_contract;

    use super::*;

    static NEXT_TEST_DIRECTORY: AtomicU64 = AtomicU64::new(0);

    struct TestDirectory {
        path: PathBuf,
    }

    impl TestDirectory {
        fn new() -> Self {
            let sequence = NEXT_TEST_DIRECTORY.fetch_add(1, Ordering::Relaxed);
            let timestamp = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("system clock should follow Unix epoch")
                .as_nanos();

            let path = env::temp_dir().join(format!(
                "presolve-cli-release-inspect-{}-{timestamp}-{sequence}",
                std::process::id()
            ));

            fs::create_dir_all(&path).expect("test directory should be created");
            Self { path }
        }

        fn path(&self) -> &Path {
            &self.path
        }
    }

    impl Drop for TestDirectory {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.path);
        }
    }

    fn mixed_release() -> PresolveBundle {
        let contract = parse_contract(
            r#"
contract_version = "0.1"

[application]
name = "mixed-release"
version = "1.2.3"
description = "Release inspection fixture"

[[workloads]]
name = "api"
kind = "component"

[[workloads]]
name = "worker"
kind = "container"

[[interfaces]]
name = "jobs"
kind = "event"

[[relationships]]
from = "api"
to = "worker"
interface = "jobs"

[[capabilities]]
interface = "presolve:kv/store"
version = "^0.1"

[resources]
memory_mib = 256
cpu_millis = 500

[network]
outbound = "deny"
"#,
        )
        .expect("contract should parse");

        let inputs = [
            WorkloadArtifactInput::Component(ComponentArtifactInput::new(
                "api",
                b"component bytes",
            )),
            WorkloadArtifactInput::OciImage(OciImageArtifactInput::new(
                "worker",
                OCI_IMAGE_MANIFEST_MEDIA_TYPE,
                "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
                2_048,
            )),
        ];

        PresolveBundle::from_workload_artifacts(&contract, &inputs)
            .expect("mixed release should build")
    }

    #[test]
    fn release_inspection_decodes_and_renders_immutable_identity() {
        let directory = TestDirectory::new();
        let release = directory.path().join("mixed.presolved");
        fs::write(
            &release,
            mixed_release().encode().expect("release should encode"),
        )
        .expect("release should be written");

        let document = load_release(directory.path(), Path::new("mixed.presolved"))
            .expect("release should load");
        let rendered = render_release(&document).expect("release should render");

        assert_eq!(
            document.path,
            release.canonicalize().expect("path should resolve")
        );
        assert!(rendered.contains("bundle version: 0.2"));
        assert!(rendered.contains("release digest: sha256:"));
        assert!(rendered.contains("application: mixed-release 1.2.3"));
        assert!(rendered.contains("api -> component"));
        assert!(rendered.contains("artifact: artifacts/api.wasm"));
        assert!(rendered.contains("media type: application/wasm"));
        assert!(rendered.contains("worker -> container"));
        assert!(
            rendered.contains(
                "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
            )
        );
        assert!(rendered.contains("presolve:kv/store ^0.1 (required)"));
        assert!(rendered.contains("api -> worker via jobs"));
    }

    #[test]
    fn release_inspection_rejects_non_presolved_path() {
        let directory = TestDirectory::new();
        let path = directory.path().join("release.bin");
        fs::write(&path, b"not a release").expect("file should be written");

        let error = load_release(directory.path(), Path::new("release.bin"))
            .expect_err("non-presolved path should fail");

        assert!(error.to_string().contains("`.presolved`"));
    }

    #[test]
    fn release_inspection_rejects_corrupt_bundle() {
        let directory = TestDirectory::new();
        let path = directory.path().join("corrupt.presolved");
        fs::write(&path, b"not a bundle").expect("file should be written");

        let error = load_release(directory.path(), Path::new("corrupt.presolved"))
            .expect_err("corrupt release should fail");

        assert!(error.to_string().contains("invalid `.presolved` release"));
    }
}
