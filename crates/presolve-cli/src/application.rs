use std::{
    fs, io,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result, bail};
use presolve_contract::{
    ApplicationContract, CONTRACT_FILE_NAME, ContractDiagnostics, parse_contract,
};

#[derive(Debug)]
pub(crate) struct ApplicationDocument {
    pub(crate) path: PathBuf,
    pub(crate) contract: ApplicationContract,
}

pub(crate) fn discover_application_file(start: &Path) -> Result<Option<PathBuf>> {
    for directory in start.ancestors() {
        let candidate = directory.join(CONTRACT_FILE_NAME);

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

pub(crate) fn load_application(start: &Path) -> Result<ApplicationDocument> {
    let path = discover_application_file(start)?.ok_or_else(|| {
        anyhow::anyhow!(
            "could not find `{CONTRACT_FILE_NAME}` from `{}` or any parent directory",
            start.display()
        )
    })?;

    let source = fs::read_to_string(&path)
        .with_context(|| format!("could not read `{}`", path.display()))?;

    let contract = parse_contract(&source).map_err(|diagnostics| {
        anyhow::anyhow!(
            "could not load `{}`: {}",
            path.display(),
            format_contract_diagnostics(&diagnostics)
        )
    })?;

    Ok(ApplicationDocument { path, contract })
}

fn format_contract_diagnostics(diagnostics: &ContractDiagnostics) -> String {
    diagnostics
        .diagnostics()
        .iter()
        .map(|diagnostic| {
            if let Some(help) = diagnostic.help() {
                format!(
                    "{}: {}; help: {help}",
                    diagnostic.code(),
                    diagnostic.message()
                )
            } else {
                format!("{}: {}", diagnostic.code(), diagnostic.message())
            }
        })
        .collect::<Vec<_>>()
        .join(" | ")
}

#[cfg(test)]
mod tests {
    use std::{
        env, fs,
        sync::atomic::{AtomicU64, Ordering},
        time::{SystemTime, UNIX_EPOCH},
    };

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
                "presolve-cli-application-{}-{timestamp}-{sequence}",
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

    fn write_contract(directory: &Path, name: &str) -> PathBuf {
        let path = directory.join(CONTRACT_FILE_NAME);
        fs::write(
            &path,
            format!(
                r#"
contract_version = "0.1"

[application]
name = "{name}"
version = "1.0.0"
"#
            ),
        )
        .expect("contract should be written");

        path
    }

    #[test]
    fn discovery_walks_up_to_nearest_application_contract() {
        let directory = TestDirectory::new();
        let nested = directory.path().join("a/b/c");
        fs::create_dir_all(&nested).expect("nested directory should be created");

        let root_contract = write_contract(directory.path(), "root-app");

        assert_eq!(
            discover_application_file(&nested).expect("discovery should succeed"),
            Some(root_contract)
        );

        let nearer_directory = directory.path().join("a");
        let nearer_contract = write_contract(&nearer_directory, "nearer-app");

        assert_eq!(
            discover_application_file(&nested).expect("nearest discovery should succeed"),
            Some(nearer_contract)
        );
    }

    #[test]
    fn load_application_parses_discovered_contract() {
        let directory = TestDirectory::new();
        let nested = directory.path().join("nested");
        fs::create_dir_all(&nested).expect("nested directory should be created");

        let path = write_contract(directory.path(), "orders");

        let document = load_application(&nested).expect("application should load");

        assert_eq!(document.path, path);
        assert_eq!(document.contract.application().name(), "orders");
    }

    #[test]
    fn load_application_reports_contract_diagnostic_codes() {
        let directory = TestDirectory::new();
        fs::write(
            directory.path().join(CONTRACT_FILE_NAME),
            r#"
contract_version = "0.1"

[application]
name = "Invalid Name"
version = "1.0.0"
"#,
        )
        .expect("invalid contract should be written");

        let error = load_application(directory.path()).expect_err("invalid contract should fail");

        assert!(error.to_string().contains("PS1003"));
    }

    #[test]
    fn load_application_reports_when_contract_is_missing() {
        let directory = TestDirectory::new();

        let error = load_application(directory.path()).expect_err("missing contract should fail");

        assert!(error.to_string().contains(CONTRACT_FILE_NAME));
    }
}
