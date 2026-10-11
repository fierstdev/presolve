use std::{
    fs::{self, File, OpenOptions},
    io::{self, Write},
    path::{Path, PathBuf},
};

use anyhow::{Context, Result, bail};
use presolve_core::EnvironmentId;
use presolve_environment::{
    ENVIRONMENT_FILE_NAME, EnvironmentMetadata, EnvironmentSpecification, ExecutionSupport,
    ResourceCapacity, encode_environment, parse_environment,
};

#[derive(Debug)]
pub(crate) struct EnvironmentDocument {
    pub(crate) path: PathBuf,
    pub(crate) specification: EnvironmentSpecification,
}

pub(crate) fn create_environment(
    directory: &Path,
    name: &str,
    memory_mib: u64,
    cpu_millis: u32,
) -> Result<EnvironmentDocument> {
    let specification = EnvironmentSpecification::current(
        EnvironmentMetadata::new(EnvironmentId::new(), name),
        ResourceCapacity::new(memory_mib, cpu_millis),
        ExecutionSupport::components(),
        Vec::new(),
    )
    .context("could not create Environment Specification")?;

    let encoded =
        encode_environment(&specification).context("could not encode Environment Specification")?;

    let path = directory.join(ENVIRONMENT_FILE_NAME);
    let mut file = create_new_file(&path)?;

    file.write_all(encoded.as_bytes())
        .with_context(|| format!("could not write `{}`", path.display()))?;

    Ok(EnvironmentDocument {
        path,
        specification,
    })
}

pub(crate) fn discover_environment_file(start: &Path) -> Result<Option<PathBuf>> {
    for directory in start.ancestors() {
        let candidate = directory.join(ENVIRONMENT_FILE_NAME);

        match fs::metadata(&candidate) {
            Ok(metadata) if metadata.is_file() => return Ok(Some(candidate)),
            Ok(_) => {
                bail!("`{}` exists but is not a regular file", candidate.display());
            }
            Err(error) if error.kind() == io::ErrorKind::NotFound => {}
            Err(error) => {
                return Err(error)
                    .with_context(|| format!("could not inspect `{}`", candidate.display()));
            }
        }
    }

    Ok(None)
}

pub(crate) fn load_environment(start: &Path) -> Result<EnvironmentDocument> {
    let path = discover_environment_file(start)?.ok_or_else(|| {
        anyhow::anyhow!(
            "could not find `{ENVIRONMENT_FILE_NAME}` from `{}` or any parent directory; \
             run `presolve env init` to create one",
            start.display()
        )
    })?;

    let source = fs::read_to_string(&path)
        .with_context(|| format!("could not read `{}`", path.display()))?;

    let specification = parse_environment(&source)
        .with_context(|| format!("could not load `{}`", path.display()))?;

    Ok(EnvironmentDocument {
        path,
        specification,
    })
}

fn create_new_file(path: &Path) -> Result<File> {
    match OpenOptions::new().write(true).create_new(true).open(path) {
        Ok(file) => Ok(file),
        Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {
            bail!(
                "`{}` already exists; Presolve will not overwrite an Environment Specification",
                path.display()
            );
        }
        Err(error) => Err(error).with_context(|| format!("could not create `{}`", path.display())),
    }
}

#[cfg(test)]
mod tests {
    use std::{
        env, fs,
        sync::atomic::{AtomicU64, Ordering},
        time::{SystemTime, UNIX_EPOCH},
    };

    use presolve_core::WorkloadKind;

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
                "presolve-cli-{}-{timestamp}-{sequence}",
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

    #[test]
    fn create_environment_generates_valid_persisted_identity() {
        let directory = TestDirectory::new();

        let document = create_environment(directory.path(), "local", 1_024, 1_000)
            .expect("environment should be created");

        assert_eq!(document.path, directory.path().join(ENVIRONMENT_FILE_NAME));
        assert!(
            document
                .specification
                .environment()
                .id()
                .to_string()
                .starts_with("env_")
        );
        assert_eq!(document.specification.environment().name(), "local");
        assert_eq!(document.specification.resources().memory_mib(), 1_024);
        assert_eq!(document.specification.resources().cpu_millis(), 1_000);
        assert_eq!(
            document.specification.execution().workloads(),
            [WorkloadKind::Component]
        );
        assert_eq!(document.specification.providers(), []);

        let persisted =
            fs::read_to_string(&document.path).expect("persisted environment should be readable");
        let decoded =
            parse_environment(&persisted).expect("persisted environment should remain valid");

        assert_eq!(decoded, document.specification);
    }

    #[test]
    fn create_environment_refuses_to_overwrite_existing_specification() {
        let directory = TestDirectory::new();

        create_environment(directory.path(), "first", 1_024, 1_000)
            .expect("first environment should be created");

        let error = create_environment(directory.path(), "second", 2_048, 2_000)
            .expect_err("second environment should be rejected");

        assert!(
            error
                .to_string()
                .contains("will not overwrite an Environment Specification")
        );
    }

    #[test]
    fn discovery_walks_up_to_nearest_environment_file() {
        let directory = TestDirectory::new();
        let nested = directory.path().join("a/b/c");
        fs::create_dir_all(&nested).expect("nested directory should be created");

        let root_document = create_environment(directory.path(), "root", 1_024, 1_000)
            .expect("root environment should be created");

        assert_eq!(
            discover_environment_file(&nested).expect("discovery should succeed"),
            Some(root_document.path.clone())
        );

        let nearer_directory = directory.path().join("a");
        let nearer_document = create_environment(&nearer_directory, "nearer", 2_048, 2_000)
            .expect("nearer environment should be created");

        assert_eq!(
            discover_environment_file(&nested).expect("nearest discovery should succeed"),
            Some(nearer_document.path)
        );
    }

    #[test]
    fn load_environment_validates_discovered_document() {
        let directory = TestDirectory::new();
        let nested = directory.path().join("nested");
        fs::create_dir_all(&nested).expect("nested directory should be created");

        let created = create_environment(directory.path(), "local", 1_024, 1_000)
            .expect("environment should be created");

        let loaded = load_environment(&nested).expect("environment should load");

        assert_eq!(loaded.path, created.path);
        assert_eq!(loaded.specification, created.specification);
    }

    #[test]
    fn load_environment_reports_when_no_specification_exists() {
        let directory = TestDirectory::new();

        let error =
            load_environment(directory.path()).expect_err("missing environment should fail");

        assert!(error.to_string().contains("run `presolve env init`"));
    }
}
