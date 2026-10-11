use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result, bail};
use presolve_bundle::{
    ComponentArtifactInput, OCI_IMAGE_INDEX_MEDIA_TYPE, OCI_IMAGE_MANIFEST_MEDIA_TYPE,
    OciImageArtifactInput, PresolveBundle, WorkloadArtifactInput,
};
use presolve_contract::{ApplicationContract, WorkloadKind};
use presolve_core::ArtifactDigest;

use crate::{
    application::{ApplicationDocument, load_application},
    dev::build_rust_component_release,
    typescript::{automatic_typescript_entry, build_typescript_component_release},
};

const DEFAULT_OUTPUT_DIRECTORY: &str = "dist";
const PRESOLVED_EXTENSION: &str = "presolved";

#[derive(Debug)]
pub(crate) enum BuildArtifactOutput {
    Component {
        workload: String,
        path: PathBuf,
    },
    OciImage {
        workload: String,
        media_type: String,
        digest: String,
        size: u64,
    },
}

#[derive(Debug)]
pub(crate) struct BuildOutput {
    pub(crate) application: String,
    pub(crate) version: String,
    pub(crate) artifacts: Vec<BuildArtifactOutput>,
    pub(crate) release: PathBuf,
    pub(crate) release_digest: ArtifactDigest,
    pub(crate) encoded_size: u64,
}

#[derive(Debug, Clone)]
struct DeclaredWorkload {
    name: String,
    kind: WorkloadKind,
}

#[derive(Debug)]
enum ResolvedWorkloadArtifact {
    Component {
        workload: String,
        path: PathBuf,
        bytes: Vec<u8>,
    },
    OciImage {
        workload: String,
        media_type: &'static str,
        digest: String,
        size: u64,
    },
}

#[derive(Debug, Clone)]
struct OciImageMapping {
    workload: String,
    media_type: &'static str,
    digest: String,
    size: u64,
}

#[derive(Debug, Clone, Copy)]
struct ArtifactSelection<'a> {
    component_override: Option<&'a Path>,
    typescript_entry: Option<&'a Path>,
    component_mappings: &'a [String],
    oci_image_mappings: &'a [String],
}

pub(crate) fn build_release(
    start: &Path,
    component_override: Option<&Path>,
    typescript_entry: Option<&Path>,
    component_mappings: &[String],
    oci_image_mappings: &[String],
    output_override: Option<&Path>,
) -> Result<BuildOutput> {
    let explicit_single_source = component_override.is_some() || typescript_entry.is_some();

    if explicit_single_source && (!component_mappings.is_empty() || !oci_image_mappings.is_empty())
    {
        bail!(
            "single-workload source/artifact overrides cannot be combined with \
             `--component-for` or `--oci-image-for`"
        );
    }

    if component_override.is_some() && typescript_entry.is_some() {
        bail!("`--component` and `--typescript-entry` cannot be used together");
    }

    let document = load_application(start)?;
    let application_directory = application_directory(&document)?;
    let workloads = declared_workloads(&document.contract);

    let selection = ArtifactSelection {
        component_override,
        typescript_entry,
        component_mappings,
        oci_image_mappings,
    };

    let artifacts = resolve_workload_artifacts(
        start,
        application_directory,
        &document.contract,
        &workloads,
        selection,
    )?;

    let bundle = bundle_from_artifacts(&document.contract, &artifacts)?;
    let (release_digest, encoded) = verify_encoded_bundle(&bundle)?;

    let release = release_output_path(
        start,
        application_directory,
        document.contract.application().name(),
        output_override,
    )?;

    reject_component_output_collision(&artifacts, &release)?;
    let encoded_size = write_release(&release, &encoded)?;

    Ok(BuildOutput {
        application: document.contract.application().name().to_owned(),
        version: document.contract.application().version().to_string(),
        artifacts: build_artifact_outputs(artifacts),
        release,
        release_digest,
        encoded_size,
    })
}

fn bundle_from_artifacts(
    contract: &ApplicationContract,
    artifacts: &[ResolvedWorkloadArtifact],
) -> Result<PresolveBundle> {
    let inputs = artifacts
        .iter()
        .map(|artifact| match artifact {
            ResolvedWorkloadArtifact::Component {
                workload, bytes, ..
            } => WorkloadArtifactInput::Component(ComponentArtifactInput::new(workload, bytes)),
            ResolvedWorkloadArtifact::OciImage {
                workload,
                media_type,
                digest,
                size,
            } => WorkloadArtifactInput::OciImage(OciImageArtifactInput::new(
                workload, media_type, digest, *size,
            )),
        })
        .collect::<Vec<_>>();

    PresolveBundle::from_workload_artifacts(contract, &inputs)
        .context("could not construct `.presolved` application release")
}

fn verify_encoded_bundle(bundle: &PresolveBundle) -> Result<(ArtifactDigest, Vec<u8>)> {
    let release_digest = bundle
        .release_digest()
        .context("could not compute application release digest")?;

    let encoded = bundle
        .encode()
        .context("could not encode `.presolved` application release")?;

    let decoded =
        PresolveBundle::decode(&encoded).context("encoded `.presolved` release did not verify")?;

    let decoded_digest = decoded
        .release_digest()
        .context("could not compute verified release digest")?;

    if decoded_digest != release_digest {
        bail!(
            "application release identity changed after encode/decode verification: \
             {release_digest} != {decoded_digest}"
        );
    }

    Ok((release_digest, encoded))
}

fn reject_component_output_collision(
    artifacts: &[ResolvedWorkloadArtifact],
    release: &Path,
) -> Result<()> {
    if let Some(workload) = component_input_at_path(artifacts, release) {
        bail!(
            "release output `{}` would overwrite component input for workload `{workload}`",
            release.display()
        );
    }

    Ok(())
}

fn write_release(release: &Path, encoded: &[u8]) -> Result<u64> {
    if let Some(parent) = release.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("could not create output directory `{}`", parent.display()))?;
    }

    fs::write(release, encoded)
        .with_context(|| format!("could not write release `{}`", release.display()))?;

    u64::try_from(encoded.len()).context("encoded `.presolved` release is too large to represent")
}

fn build_artifact_outputs(artifacts: Vec<ResolvedWorkloadArtifact>) -> Vec<BuildArtifactOutput> {
    artifacts
        .into_iter()
        .map(|artifact| match artifact {
            ResolvedWorkloadArtifact::Component { workload, path, .. } => {
                BuildArtifactOutput::Component { workload, path }
            }
            ResolvedWorkloadArtifact::OciImage {
                workload,
                media_type,
                digest,
                size,
            } => BuildArtifactOutput::OciImage {
                workload,
                media_type: media_type.to_owned(),
                digest,
                size,
            },
        })
        .collect()
}

fn application_directory(document: &ApplicationDocument) -> Result<&Path> {
    document
        .path
        .parent()
        .context("Application Contract path has no parent directory")
}

fn declared_workloads(contract: &ApplicationContract) -> Vec<DeclaredWorkload> {
    if contract.workloads().is_empty() {
        return vec![DeclaredWorkload {
            name: "main".to_owned(),
            kind: WorkloadKind::Component,
        }];
    }

    contract
        .workloads()
        .iter()
        .map(|workload| DeclaredWorkload {
            name: workload.name().to_owned(),
            kind: workload.kind(),
        })
        .collect()
}

fn resolve_workload_artifacts(
    start: &Path,
    application_directory: &Path,
    contract: &ApplicationContract,
    workloads: &[DeclaredWorkload],
    selection: ArtifactSelection<'_>,
) -> Result<Vec<ResolvedWorkloadArtifact>> {
    if let Some(path) = selection.component_override {
        return resolve_single_component_override(start, workloads, path);
    }

    if let Some(entry) = selection.typescript_entry {
        return resolve_single_typescript_source(
            start,
            application_directory,
            contract,
            workloads,
            entry,
        );
    }

    if selection.component_mappings.is_empty() && selection.oci_image_mappings.is_empty() {
        return resolve_automatic_source_build(start, application_directory, contract, workloads);
    }

    resolve_explicit_workload_mappings(
        start,
        workloads,
        selection.component_mappings,
        selection.oci_image_mappings,
    )
}

fn resolve_single_component_override(
    start: &Path,
    workloads: &[DeclaredWorkload],
    path: &Path,
) -> Result<Vec<ResolvedWorkloadArtifact>> {
    if workloads.len() != 1 {
        bail!(
            "`--component PATH` is the single-workload shorthand; application declares {} \
             workloads, so use explicit workload mappings",
            workloads.len()
        );
    }

    let workload = &workloads[0];

    if workload.kind != WorkloadKind::Component {
        bail!(
            "`--component PATH` cannot realize container workload `{}`; \
             use `--oci-image-for {}=TYPE,DIGEST,SIZE`",
            workload.name,
            workload.name
        );
    }

    read_component_artifact(start, &workload.name, path).map(|artifact| vec![artifact])
}

fn resolve_automatic_source_build(
    start: &Path,
    application_directory: &Path,
    contract: &ApplicationContract,
    workloads: &[DeclaredWorkload],
) -> Result<Vec<ResolvedWorkloadArtifact>> {
    let workload = single_source_workload(workloads)?;

    if workload.kind != WorkloadKind::Component {
        bail!(
            "container workload `{}` requires an immutable OCI descriptor through \
             `--oci-image-for {}=TYPE,DIGEST,SIZE`",
            workload.name,
            workload.name
        );
    }

    let rust_source = application_directory.join("Cargo.toml").is_file();
    let typescript_entry = automatic_typescript_entry(application_directory);

    match (rust_source, typescript_entry) {
        (true, Some(_)) => bail!(
            "both Rust and TypeScript automatic source conventions are present; \
             use `--typescript-entry PATH`, `--component PATH`, or an explicit artifact mapping"
        ),
        (true, None) => {
            let path = build_rust_component_release(application_directory)?;
            read_component_artifact(application_directory, &workload.name, &path)
                .map(|artifact| vec![artifact])
        }
        (false, Some(entry)) => {
            resolve_typescript_source(start, application_directory, contract, workload, &entry)
        }
        (false, None) => bail!(
            "no automatic Component source convention was found beside `presolve.toml`; \
             expected `Cargo.toml` or `package.json` with `src/main.ts`, or supply an explicit artifact"
        ),
    }
}

fn resolve_single_typescript_source(
    start: &Path,
    application_directory: &Path,
    contract: &ApplicationContract,
    workloads: &[DeclaredWorkload],
    entry: &Path,
) -> Result<Vec<ResolvedWorkloadArtifact>> {
    let workload = single_source_workload(workloads)?;

    if workload.kind != WorkloadKind::Component {
        bail!(
            "`--typescript-entry` cannot realize container workload `{}`",
            workload.name
        );
    }

    resolve_typescript_source(start, application_directory, contract, workload, entry)
}

fn resolve_typescript_source(
    start: &Path,
    application_directory: &Path,
    contract: &ApplicationContract,
    workload: &DeclaredWorkload,
    entry: &Path,
) -> Result<Vec<ResolvedWorkloadArtifact>> {
    let path = build_typescript_component_release(
        contract,
        application_directory,
        start,
        &workload.name,
        entry,
    )?;

    read_component_artifact(application_directory, &workload.name, &path)
        .map(|artifact| vec![artifact])
}

fn single_source_workload(workloads: &[DeclaredWorkload]) -> Result<&DeclaredWorkload> {
    match workloads {
        [workload] => Ok(workload),
        workloads => bail!(
            "automatic source building currently supports one workload; application declares {}. \
             Use explicit `--component-for` and/or `--oci-image-for` mappings.",
            workloads.len()
        ),
    }
}

fn resolve_explicit_workload_mappings(
    start: &Path,
    workloads: &[DeclaredWorkload],
    component_mappings: &[String],
    oci_image_mappings: &[String],
) -> Result<Vec<ResolvedWorkloadArtifact>> {
    let expected = workloads
        .iter()
        .map(|workload| (workload.name.as_str(), workload.kind))
        .collect::<BTreeMap<_, _>>();

    let mut components = BTreeMap::<String, PathBuf>::new();

    for mapping in component_mappings {
        let (workload, path) = parse_component_mapping(mapping)?;
        validate_mapping_target(
            &expected,
            workload,
            WorkloadKind::Component,
            "--component-for",
        )?;

        if components
            .insert(workload.to_owned(), PathBuf::from(path))
            .is_some()
        {
            bail!("duplicate `--component-for` mapping for workload `{workload}`");
        }
    }

    let mut images = BTreeMap::<String, OciImageMapping>::new();

    for mapping in oci_image_mappings {
        let image = parse_oci_image_mapping(mapping)?;
        validate_mapping_target(
            &expected,
            &image.workload,
            WorkloadKind::Container,
            "--oci-image-for",
        )?;

        let workload = image.workload.clone();

        if images.insert(workload.clone(), image).is_some() {
            bail!("duplicate `--oci-image-for` mapping for workload `{workload}`");
        }
    }

    let component_names = components.keys().map(String::as_str);
    let image_names = images.keys().map(String::as_str);
    let supplied = component_names.chain(image_names).collect::<BTreeSet<_>>();

    let missing = workloads
        .iter()
        .filter(|workload| !supplied.contains(workload.name.as_str()))
        .map(|workload| workload.name.clone())
        .collect::<Vec<_>>();

    if !missing.is_empty() {
        bail!(
            "missing artifact mapping for workload(s): {}",
            missing.join(", ")
        );
    }

    workloads
        .iter()
        .map(|workload| match workload.kind {
            WorkloadKind::Component => {
                let Some(path) = components.get(&workload.name) else {
                    bail!(
                        "Component workload `{}` requires `--component-for {}=PATH`",
                        workload.name,
                        workload.name
                    );
                };

                read_component_artifact(start, &workload.name, path)
            }
            WorkloadKind::Container => {
                let Some(image) = images.get(&workload.name) else {
                    bail!(
                        "container workload `{}` requires \
                         `--oci-image-for {}=TYPE,DIGEST,SIZE`",
                        workload.name,
                        workload.name
                    );
                };

                Ok(ResolvedWorkloadArtifact::OciImage {
                    workload: workload.name.clone(),
                    media_type: image.media_type,
                    digest: image.digest.clone(),
                    size: image.size,
                })
            }
        })
        .collect()
}

fn validate_mapping_target(
    expected: &BTreeMap<&str, WorkloadKind>,
    workload: &str,
    supplied_kind: WorkloadKind,
    flag: &str,
) -> Result<()> {
    let Some(expected_kind) = expected.get(workload) else {
        let names = expected.keys().copied().collect::<Vec<_>>().join(", ");
        bail!("`{flag}` names unknown workload `{workload}`; expected one of: {names}");
    };

    if *expected_kind != supplied_kind {
        bail!(
            "`{flag}` supplies a `{supplied_kind}` artifact for workload `{workload}`, \
             but the Application Contract declares `{expected_kind}`"
        );
    }

    Ok(())
}

fn parse_component_mapping(mapping: &str) -> Result<(&str, &str)> {
    let (workload, path) = mapping.split_once('=').ok_or_else(|| {
        anyhow::anyhow!("invalid `--component-for` value `{mapping}`; expected WORKLOAD=PATH")
    })?;

    if workload.is_empty() {
        bail!("invalid `--component-for` value `{mapping}`; workload name cannot be empty");
    }

    if path.is_empty() {
        bail!("invalid `--component-for` value `{mapping}`; path cannot be empty");
    }

    Ok((workload, path))
}

fn parse_oci_image_mapping(mapping: &str) -> Result<OciImageMapping> {
    let (workload, descriptor) = mapping.split_once('=').ok_or_else(|| {
        anyhow::anyhow!(
            "invalid `--oci-image-for` value `{mapping}`; expected \
             WORKLOAD=TYPE,DIGEST,SIZE"
        )
    })?;

    if workload.is_empty() {
        bail!("invalid `--oci-image-for` value `{mapping}`; workload name cannot be empty");
    }

    let mut fields = descriptor.split(',');
    let image_type = fields.next().unwrap_or_default();
    let digest = fields.next().unwrap_or_default();
    let size = fields.next().unwrap_or_default();

    if fields.next().is_some() || image_type.is_empty() || digest.is_empty() || size.is_empty() {
        bail!(
            "invalid `--oci-image-for` value `{mapping}`; expected \
             WORKLOAD=TYPE,DIGEST,SIZE"
        );
    }

    let media_type = match image_type {
        "manifest" => OCI_IMAGE_MANIFEST_MEDIA_TYPE,
        "index" => OCI_IMAGE_INDEX_MEDIA_TYPE,
        other => bail!(
            "unsupported OCI descriptor type `{other}` in `{mapping}`; \
             expected `manifest` or `index`"
        ),
    };

    let size = size.parse::<u64>().with_context(|| {
        format!("invalid OCI descriptor size `{size}` in `--oci-image-for {mapping}`")
    })?;

    Ok(OciImageMapping {
        workload: workload.to_owned(),
        media_type,
        digest: digest.to_owned(),
        size,
    })
}

fn read_component_artifact(
    start: &Path,
    workload: &str,
    path: &Path,
) -> Result<ResolvedWorkloadArtifact> {
    let path = resolve_input_path(start, path)?;
    let bytes = fs::read(&path)
        .with_context(|| format!("could not read component `{}`", path.display()))?;

    Ok(ResolvedWorkloadArtifact::Component {
        workload: workload.to_owned(),
        path,
        bytes,
    })
}

fn resolve_input_path(start: &Path, path: &Path) -> Result<PathBuf> {
    let resolved = if path.is_absolute() {
        path.to_path_buf()
    } else {
        start.join(path)
    };

    let metadata = fs::metadata(&resolved)
        .with_context(|| format!("could not inspect component `{}`", resolved.display()))?;

    if !metadata.is_file() {
        bail!("component `{}` is not a regular file", resolved.display());
    }

    Ok(resolved)
}

fn component_input_at_path<'a>(
    artifacts: &'a [ResolvedWorkloadArtifact],
    release: &Path,
) -> Option<&'a str> {
    artifacts.iter().find_map(|artifact| match artifact {
        ResolvedWorkloadArtifact::Component { workload, path, .. } if path == release => {
            Some(workload.as_str())
        }
        _ => None,
    })
}

fn release_output_path(
    start: &Path,
    application_directory: &Path,
    application_name: &str,
    output_override: Option<&Path>,
) -> Result<PathBuf> {
    let output = match output_override {
        Some(path) if path.is_absolute() => path.to_path_buf(),
        Some(path) => start.join(path),
        None => application_directory
            .join(DEFAULT_OUTPUT_DIRECTORY)
            .join(format!("{application_name}.{PRESOLVED_EXTENSION}")),
    };

    if output.extension().and_then(|extension| extension.to_str()) != Some(PRESOLVED_EXTENSION) {
        bail!(
            "release output `{}` must use the `.{PRESOLVED_EXTENSION}` extension",
            output.display()
        );
    }

    Ok(output)
}

#[cfg(test)]
mod tests {
    use std::{
        env,
        sync::atomic::{AtomicU64, Ordering},
        time::{SystemTime, UNIX_EPOCH},
    };

    use presolve_bundle::WorkloadManifest;

    use super::*;

    static NEXT_TEST_DIRECTORY: AtomicU64 = AtomicU64::new(0);
    const OCI_DIGEST: &str =
        "sha256:0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";

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
                "presolve-cli-build-{}-{timestamp}-{sequence}",
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

    fn write_contract(directory: &Path, workloads: &str) {
        fs::write(
            directory.join("presolve.toml"),
            format!(
                r#"
contract_version = "0.1"

[application]
name = "orders"
version = "1.2.3"

{workloads}
"#
            ),
        )
        .expect("contract should be written");
    }

    fn multi_component_contract() -> &'static str {
        r#"
[[workloads]]
name = "api"
kind = "component"

[[workloads]]
name = "worker"
kind = "component"
"#
    }

    fn mixed_contract() -> &'static str {
        r#"
[[workloads]]
name = "api"
kind = "component"

[[workloads]]
name = "worker"
kind = "container"
"#
    }

    fn container_contract() -> &'static str {
        r#"
[[workloads]]
name = "api"
kind = "container"
"#
    }

    fn manifest_mapping(workload: &str, size: u64) -> String {
        format!("{workload}=manifest,{OCI_DIGEST},{size}")
    }

    fn output_workload(artifact: &BuildArtifactOutput) -> &str {
        match artifact {
            BuildArtifactOutput::Component { workload, .. }
            | BuildArtifactOutput::OciImage { workload, .. } => workload,
        }
    }

    #[test]
    fn implicit_component_build_writes_default_dist_release() {
        let directory = TestDirectory::new();
        write_contract(directory.path(), "");

        let component = directory.path().join("app.wasm");
        fs::write(&component, b"component-bytes").expect("component should be written");

        let output = build_release(directory.path(), Some(&component), None, &[], &[], None)
            .expect("prebuilt component should package");

        assert_eq!(output.application, "orders");
        assert_eq!(output.version, "1.2.3");
        assert_eq!(output.artifacts.len(), 1);
        assert_eq!(output_workload(&output.artifacts[0]), "main");
        assert_eq!(
            output.release,
            directory.path().join("dist").join("orders.presolved")
        );
        assert!(output.release.is_file());

        let encoded = fs::read(&output.release).expect("release should be readable");
        let decoded = PresolveBundle::decode(&encoded).expect("release should verify");

        assert_eq!(decoded.manifest().application().name(), "orders");
        assert_eq!(
            decoded
                .release_digest()
                .expect("release digest should compute"),
            output.release_digest
        );
    }

    #[test]
    fn build_does_not_require_environment_specification() {
        let directory = TestDirectory::new();
        write_contract(directory.path(), container_contract());

        assert!(!directory.path().join("presolve.env.toml").exists());

        build_release(
            directory.path(),
            None,
            None,
            &[],
            &[manifest_mapping("api", 512)],
            None,
        )
        .expect("container release construction must not require an environment");
    }

    #[test]
    fn explicit_output_is_resolved_from_invocation_directory() {
        let directory = TestDirectory::new();
        write_contract(directory.path(), "");

        let component = directory.path().join("app.wasm");
        fs::write(&component, b"component").expect("component should be written");

        let output = build_release(
            directory.path(),
            Some(&component),
            None,
            &[],
            &[],
            Some(Path::new("release/orders-custom.presolved")),
        )
        .expect("explicit output should work");

        assert_eq!(
            output.release,
            directory
                .path()
                .join("release")
                .join("orders-custom.presolved")
        );
    }

    #[test]
    fn multiple_components_are_mapped_by_workload_name() {
        let directory = TestDirectory::new();
        write_contract(directory.path(), multi_component_contract());

        fs::write(directory.path().join("api.wasm"), b"api-component")
            .expect("api component should be written");
        fs::write(directory.path().join("worker.wasm"), b"worker-component")
            .expect("worker component should be written");

        let mappings = vec!["api=api.wasm".to_owned(), "worker=worker.wasm".to_owned()];

        let output = build_release(directory.path(), None, None, &mappings, &[], None)
            .expect("multi-workload release should package");

        assert_eq!(output.artifacts.len(), 2);
        assert_eq!(output_workload(&output.artifacts[0]), "api");
        assert_eq!(output_workload(&output.artifacts[1]), "worker");

        let encoded = fs::read(&output.release).expect("release should be readable");
        let decoded = PresolveBundle::decode(&encoded).expect("release should verify");

        let workloads = decoded
            .manifest()
            .workloads()
            .iter()
            .map(WorkloadManifest::name)
            .collect::<Vec<_>>();

        assert_eq!(workloads, ["api", "worker"]);
    }

    #[test]
    fn container_workload_packages_immutable_oci_descriptor() {
        let directory = TestDirectory::new();
        write_contract(directory.path(), container_contract());

        let output = build_release(
            directory.path(),
            None,
            None,
            &[],
            &[manifest_mapping("api", 1_024)],
            None,
        )
        .expect("container release should package");

        let encoded = fs::read(&output.release).expect("release should be readable");
        let decoded = PresolveBundle::decode(&encoded).expect("release should verify");
        let workload = &decoded.manifest().workloads()[0];
        let image = workload
            .oci_image()
            .expect("container should have OCI image");

        assert_eq!(workload.kind(), WorkloadKind::Container);
        assert_eq!(image.media_type(), OCI_IMAGE_MANIFEST_MEDIA_TYPE);
        assert_eq!(image.digest(), OCI_DIGEST);
        assert_eq!(image.size(), 1_024);
        assert_eq!(workload.artifacts(), []);
    }

    #[test]
    fn mixed_component_and_container_release_packages() {
        let directory = TestDirectory::new();
        write_contract(directory.path(), mixed_contract());

        fs::write(directory.path().join("api.wasm"), b"api-component")
            .expect("api component should be written");

        let output = build_release(
            directory.path(),
            None,
            None,
            &["api=api.wasm".to_owned()],
            &[manifest_mapping("worker", 2_048)],
            None,
        )
        .expect("mixed release should package");

        assert_eq!(output.artifacts.len(), 2);
        assert_eq!(output_workload(&output.artifacts[0]), "api");
        assert_eq!(output_workload(&output.artifacts[1]), "worker");

        let encoded = fs::read(output.release).expect("release should be readable");
        let decoded = PresolveBundle::decode(&encoded).expect("release should verify");

        let api = decoded
            .manifest()
            .workloads()
            .iter()
            .find(|workload| workload.name() == "api")
            .expect("api workload should exist");
        let worker = decoded
            .manifest()
            .workloads()
            .iter()
            .find(|workload| workload.name() == "worker")
            .expect("worker workload should exist");

        assert_eq!(api.kind(), WorkloadKind::Component);
        assert_eq!(worker.kind(), WorkloadKind::Container);
        assert!(worker.oci_image().is_some());
    }

    #[test]
    fn index_alias_maps_to_canonical_oci_media_type() {
        let mapping = format!("api=index,{OCI_DIGEST},4096");
        let parsed = parse_oci_image_mapping(&mapping).expect("index mapping should parse");

        assert_eq!(parsed.workload, "api");
        assert_eq!(parsed.media_type, OCI_IMAGE_INDEX_MEDIA_TYPE);
        assert_eq!(parsed.digest, OCI_DIGEST);
        assert_eq!(parsed.size, 4_096);
    }

    #[test]
    fn component_mapping_order_does_not_change_release_identity() {
        let directory = TestDirectory::new();
        write_contract(directory.path(), multi_component_contract());

        fs::write(directory.path().join("api.wasm"), b"api-component")
            .expect("api component should be written");
        fs::write(directory.path().join("worker.wasm"), b"worker-component")
            .expect("worker component should be written");

        let first = vec!["api=api.wasm".to_owned(), "worker=worker.wasm".to_owned()];
        let second = vec!["worker=worker.wasm".to_owned(), "api=api.wasm".to_owned()];

        let first_output = build_release(
            directory.path(),
            None,
            None,
            &first,
            &[],
            Some(Path::new("dist/first.presolved")),
        )
        .expect("first release should package");

        let second_output = build_release(
            directory.path(),
            None,
            None,
            &second,
            &[],
            Some(Path::new("dist/second.presolved")),
        )
        .expect("second release should package");

        assert_eq!(first_output.release_digest, second_output.release_digest);
        assert_eq!(
            fs::read(first_output.release).expect("first release should read"),
            fs::read(second_output.release).expect("second release should read")
        );
    }

    #[test]
    fn missing_mixed_artifact_mapping_is_rejected() {
        let directory = TestDirectory::new();
        write_contract(directory.path(), mixed_contract());

        fs::write(directory.path().join("api.wasm"), b"api-component")
            .expect("api component should be written");

        let error = build_release(
            directory.path(),
            None,
            None,
            &["api=api.wasm".to_owned()],
            &[],
            None,
        )
        .expect_err("missing container mapping should fail");

        assert!(error.to_string().contains("worker"));
    }

    #[test]
    fn wrong_kind_mapping_is_rejected() {
        let directory = TestDirectory::new();
        write_contract(directory.path(), mixed_contract());

        let error = build_release(
            directory.path(),
            None,
            None,
            &["worker=worker.wasm".to_owned()],
            &[],
            None,
        )
        .expect_err("Component mapping for container workload should fail");

        assert!(error.to_string().contains("container"));
    }

    #[test]
    fn duplicate_oci_mapping_is_rejected() {
        let directory = TestDirectory::new();
        write_contract(directory.path(), container_contract());

        let mapping = manifest_mapping("api", 512);
        let error = build_release(
            directory.path(),
            None,
            None,
            &[],
            &[mapping.clone(), mapping],
            None,
        )
        .expect_err("duplicate OCI mapping should fail");

        assert!(error.to_string().contains("duplicate"));
    }

    #[test]
    fn invalid_oci_mapping_shape_is_rejected() {
        let error = parse_oci_image_mapping("api=manifest,sha256:abc")
            .expect_err("incomplete OCI mapping should fail");

        assert!(error.to_string().contains("WORKLOAD=TYPE,DIGEST,SIZE"));
    }

    #[test]
    fn invalid_oci_descriptor_is_rejected_by_bundle_validation() {
        let directory = TestDirectory::new();
        write_contract(directory.path(), container_contract());

        let error = build_release(
            directory.path(),
            None,
            None,
            &[],
            &["api=manifest,sha256:bad,512".to_owned()],
            None,
        )
        .expect_err("invalid digest should fail");

        assert!(error.to_string().contains("could not construct"));
    }

    #[test]
    fn single_component_shorthand_is_rejected_for_multiple_workloads() {
        let directory = TestDirectory::new();
        write_contract(directory.path(), mixed_contract());

        let component = directory.path().join("app.wasm");
        fs::write(&component, b"component").expect("component should be written");

        let error = build_release(directory.path(), Some(&component), None, &[], &[], None)
            .expect_err("single shorthand should not guess a workload");

        assert!(error.to_string().contains("single-workload shorthand"));
    }
}
