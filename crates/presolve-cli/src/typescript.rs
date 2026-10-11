use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};

use anyhow::{Context, Result, bail};
use presolve_capability::{KEY_VALUE_INTERFACE, OBJECT_STORE_INTERFACE};
use presolve_contract::ApplicationContract;
use semver::Version;

const TYPESCRIPT_DEFAULT_ENTRY: &str = "src/main.ts";
const TYPESCRIPT_BUILD_DIRECTORY: &str = ".presolve/build/typescript";
const COMPONENT_WORLD_VERSION: (u64, u64, u64) = (0, 1, 0);

const KEY_VALUE_WIT: &str = r"package presolve:kv@0.1.0;

interface store {
    variant error {
        unavailable(string),
        invalid-key(string),
        internal(string),
    }

    get: func(key: string) -> result<option<string>, error>;
    set: func(key: string, value: string) -> result<_, error>;
    delete: func(key: string) -> result<bool, error>;
}
";

const OBJECT_STORE_WIT: &str = r"package presolve:objects@0.1.0;

interface store {
    record object-info {
        key: string,
        size: u64,
    }

    record read-session {
        handle: u64,
        info: object-info,
    }

    variant error {
        unavailable(string),
        invalid-key(string),
        invalid-body(string),
        invalid-handle(string),
        invalid-request(string),
        internal(string),
    }

    get: func(key: string) -> result<option<read-session>, error>;
    read: func(handle: u64, max-bytes: u64) -> result<list<u8>, error>;
    close-read: func(handle: u64) -> result<_, error>;
    put: func(key: string, size: u64) -> result<u64, error>;
    write: func(handle: u64, chunk: list<u8>) -> result<_, error>;
    finish-put: func(handle: u64) -> result<object-info, error>;
    abort-put: func(handle: u64) -> result<_, error>;
    head: func(key: string) -> result<option<object-info>, error>;
    delete: func(key: string) -> result<bool, error>;
    list-objects: func(prefix: string) -> result<list<object-info>, error>;
}
";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ComponentProjection {
    key_value: bool,
    object_store: bool,
}

pub(crate) fn automatic_typescript_entry(application_directory: &Path) -> Option<PathBuf> {
    let package = application_directory.join("package.json");
    let entry = application_directory.join(TYPESCRIPT_DEFAULT_ENTRY);

    (package.is_file() && entry.is_file()).then_some(entry)
}

pub(crate) fn build_typescript_component_release(
    contract: &ApplicationContract,
    application_directory: &Path,
    invocation_directory: &Path,
    workload: &str,
    entry: &Path,
) -> Result<PathBuf> {
    require_typescript_project(application_directory)?;
    let projection = component_projection(contract)?;
    let entry = resolve_typescript_entry(application_directory, invocation_directory, entry)?;

    run_typescript_typecheck(application_directory)?;

    let build_directory = application_directory.join(TYPESCRIPT_BUILD_DIRECTORY);
    let wit_directory = build_directory.join("wit");
    let output = build_directory.join(format!("{workload}.wasm"));

    reset_build_directory(&build_directory)?;
    write_wit_package(&wit_directory, projection)?;
    write_bootstrap(application_directory, &build_directory, &entry, projection)?;

    run_componentize(
        application_directory,
        &build_directory,
        &wit_directory,
        &output,
    )?;

    if !output.is_file() {
        bail!(
            "Jco reported success but expected Component `{}` was not produced",
            output.display()
        );
    }

    Ok(output)
}

fn require_typescript_project(application_directory: &Path) -> Result<()> {
    for file in ["package.json", "tsconfig.json"] {
        let path = application_directory.join(file);

        if !path.is_file() {
            bail!(
                "automatic TypeScript Component builds require `{}` beside `presolve.toml`",
                path.display()
            );
        }
    }

    Ok(())
}

fn component_projection(contract: &ApplicationContract) -> Result<ComponentProjection> {
    let version = Version::new(
        COMPONENT_WORLD_VERSION.0,
        COMPONENT_WORLD_VERSION.1,
        COMPONENT_WORLD_VERSION.2,
    );

    let mut projection = ComponentProjection {
        key_value: false,
        object_store: false,
    };

    for capability in contract.capabilities() {
        if capability.optional() {
            bail!(
                "automatic TypeScript Component builds cannot yet project optional capability `{}`; \
                 use a prebuilt Component for this application",
                capability.interface()
            );
        }

        if !capability.required_features().is_empty() {
            bail!(
                "automatic TypeScript Component builds cannot yet project required capability \
                 features for `{}`; use a prebuilt Component",
                capability.interface()
            );
        }

        if !capability.version().matches(&version) {
            bail!(
                "automatic TypeScript Component builds currently project `{}` at 0.1.0, \
                 which does not satisfy `{}`",
                capability.interface(),
                capability.version()
            );
        }

        match capability.interface() {
            KEY_VALUE_INTERFACE => projection.key_value = true,
            OBJECT_STORE_INTERFACE => projection.object_store = true,
            other => bail!(
                "automatic TypeScript Component builds do not yet have a WIT projection for \
                 capability `{other}`; use a prebuilt Component"
            ),
        }
    }

    Ok(projection)
}

fn resolve_typescript_entry(
    application_directory: &Path,
    invocation_directory: &Path,
    entry: &Path,
) -> Result<PathBuf> {
    let candidate = if entry.is_absolute() {
        entry.to_path_buf()
    } else {
        invocation_directory.join(entry)
    };

    let entry = candidate.canonicalize().with_context(|| {
        format!(
            "could not resolve TypeScript entry `{}`",
            candidate.display()
        )
    })?;
    let application_directory = application_directory.canonicalize().with_context(|| {
        format!(
            "could not resolve application directory `{}`",
            application_directory.display()
        )
    })?;

    if !entry.is_file() {
        bail!(
            "TypeScript entry `{}` is not a regular file",
            entry.display()
        );
    }

    let extension = entry.extension().and_then(|value| value.to_str());

    if !matches!(extension, Some("ts" | "mts")) {
        bail!(
            "TypeScript Component entry `{}` must use `.ts` or `.mts`",
            entry.display()
        );
    }

    if !entry.starts_with(&application_directory) {
        bail!(
            "TypeScript Component entry `{}` must be inside application directory `{}`",
            entry.display(),
            application_directory.display()
        );
    }

    Ok(entry)
}

fn reset_build_directory(build_directory: &Path) -> Result<()> {
    if build_directory.exists() {
        fs::remove_dir_all(build_directory).with_context(|| {
            format!(
                "could not reset generated TypeScript build directory `{}`",
                build_directory.display()
            )
        })?;
    }

    fs::create_dir_all(build_directory).with_context(|| {
        format!(
            "could not create TypeScript build directory `{}`",
            build_directory.display()
        )
    })
}

fn write_wit_package(wit_directory: &Path, projection: ComponentProjection) -> Result<()> {
    fs::create_dir_all(wit_directory).with_context(|| {
        format!(
            "could not create WIT directory `{}`",
            wit_directory.display()
        )
    })?;

    let mut imports = Vec::new();

    if projection.key_value {
        imports.push("    import presolve:kv/store@0.1.0;");
        write_dependency(wit_directory, "presolve-kv", "store.wit", KEY_VALUE_WIT)?;
    }

    if projection.object_store {
        imports.push("    import presolve:objects/store@0.1.0;");
        write_dependency(
            wit_directory,
            "presolve-objects",
            "store.wit",
            OBJECT_STORE_WIT,
        )?;
    }

    let import_block = if imports.is_empty() {
        String::new()
    } else {
        format!("{}\n\n", imports.join("\n"))
    };

    let world = format!(
        "package presolve:app@0.1.0;\n\nworld application {{\n{import_block}    export run: func(input: string) -> string;\n}}\n"
    );

    fs::write(wit_directory.join("application.wit"), world)
        .context("could not write generated Presolve application WIT")
}

fn write_dependency(
    wit_directory: &Path,
    package_directory: &str,
    file_name: &str,
    source: &str,
) -> Result<()> {
    let directory = wit_directory.join("deps").join(package_directory);
    fs::create_dir_all(&directory).with_context(|| {
        format!(
            "could not create generated WIT dependency directory `{}`",
            directory.display()
        )
    })?;

    fs::write(directory.join(file_name), source)
        .with_context(|| format!("could not write generated WIT dependency `{file_name}`"))
}

fn write_bootstrap(
    application_directory: &Path,
    build_directory: &Path,
    entry: &Path,
    projection: ComponentProjection,
) -> Result<()> {
    let application_directory = application_directory.canonicalize()?;
    let relative_entry = entry
        .strip_prefix(&application_directory)
        .context("validated TypeScript entry should be application-local")?;
    let entry_specifier = format!(
        "../../../{}",
        relative_entry
            .to_str()
            .context("TypeScript entry path must be valid UTF-8")?
            .replace('\\', "/")
    );
    let entry_literal = serde_json::to_string(&entry_specifier)?;

    let mut imports = vec![format!(
        "import {{ run as applicationRun }} from {entry_literal};"
    )];
    let mut bindings = Vec::new();

    if projection.key_value {
        imports.push("import * as kv from \"presolve:kv/store@0.1.0\";".to_owned());
        bindings.push("kv");
    }

    if projection.object_store {
        imports.push("import * as objects from \"presolve:objects/store@0.1.0\";".to_owned());
        bindings.push("objects");
    }

    if !bindings.is_empty() {
        imports.insert(
            0,
            "import { installJcoComponentBindings } from \"@presolve/sdk/component-jco\";"
                .to_owned(),
        );
    }

    let installation = if bindings.is_empty() {
        String::new()
    } else {
        format!(
            "\ninstallJcoComponentBindings({{ {} }});\n",
            bindings.join(", ")
        )
    };

    let bootstrap = format!(
        "{}\n{}\nexport async function run(input: string): Promise<string> {{\n    const output = await applicationRun(input);\n\n    if (typeof output !== \"string\") {{\n        throw new TypeError(\"Presolve TypeScript run(input) must resolve to a string\");\n    }}\n\n    return output;\n}}\n",
        imports.join("\n"),
        installation
    );

    fs::write(build_directory.join("bootstrap.ts"), bootstrap)
        .context("could not write generated TypeScript Component bootstrap")
}

fn run_typescript_typecheck(application_directory: &Path) -> Result<()> {
    eprintln!("Type-checking TypeScript application...");

    let status = Command::new("pnpm")
        .args(["exec", "tsc", "-p", "tsconfig.json", "--noEmit"])
        .current_dir(application_directory)
        .status()
        .context(
            "could not run `pnpm`; install pnpm and the application's TypeScript dependencies",
        )?;

    if !status.success() {
        bail!("TypeScript type-check failed with status {status}");
    }

    Ok(())
}

fn run_componentize(
    application_directory: &Path,
    build_directory: &Path,
    wit_directory: &Path,
    output: &Path,
) -> Result<()> {
    eprintln!("Componentizing TypeScript with Jco...");

    let status = Command::new("pnpm")
        .args(["exec", "jco", "componentize"])
        .arg(build_directory.join("bootstrap.ts"))
        .arg("--wit")
        .arg(wit_directory)
        .args(["--world-name", "application", "--disable", "all", "-o"])
        .arg(output)
        .current_dir(application_directory)
        .status()
        .context(
            "could not run Jco; install `@bytecodealliance/jco` and \
             `@bytecodealliance/componentize-js` in the TypeScript project",
        )?;

    if !status.success() {
        bail!("Jco TypeScript Component build failed with status {status}");
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use presolve_contract::parse_contract;

    use super::*;

    #[test]
    fn default_entry_requires_package_and_source() {
        let temporary =
            std::env::temp_dir().join(format!("presolve-ts-detect-{}", std::process::id()));
        let _ = fs::remove_dir_all(&temporary);
        fs::create_dir_all(temporary.join("src")).expect("test directory should exist");

        assert_eq!(automatic_typescript_entry(&temporary), None);

        fs::write(temporary.join("package.json"), "{}").expect("package should write");
        fs::write(temporary.join("src/main.ts"), "export function run() {}")
            .expect("entry should write");

        assert_eq!(
            automatic_typescript_entry(&temporary),
            Some(temporary.join("src/main.ts"))
        );

        let _ = fs::remove_dir_all(temporary);
    }

    #[test]
    fn projection_supports_baseline_kv_and_objects() {
        let contract = parse_contract(
            r#"
contract_version = "0.1"

[application]
name = "ts"
version = "1.0.0"

[[capabilities]]
interface = "presolve:kv/store"
version = "^0.1"

[[capabilities]]
interface = "presolve:objects/store"
version = "^0.1"
"#,
        )
        .expect("contract should parse");

        assert_eq!(
            component_projection(&contract).expect("projection should succeed"),
            ComponentProjection {
                key_value: true,
                object_store: true,
            }
        );
    }

    #[test]
    fn projection_rejects_optional_capabilities() {
        let contract = parse_contract(
            r#"
contract_version = "0.1"

[application]
name = "ts"
version = "1.0.0"

[[capabilities]]
interface = "presolve:kv/store"
version = "^0.1"
optional = true
"#,
        )
        .expect("contract should parse");

        let error = component_projection(&contract).expect_err("optional import should fail");
        assert!(error.to_string().contains("optional capability"));
    }

    #[test]
    fn projection_rejects_unknown_capabilities() {
        let contract = parse_contract(
            r#"
contract_version = "0.1"

[application]
name = "ts"
version = "1.0.0"

[[capabilities]]
interface = "example:search/index"
version = "^1"
"#,
        )
        .expect("contract should parse");

        let error = component_projection(&contract).expect_err("unknown projection should fail");
        assert!(error.to_string().contains("example:search/index"));
    }
}
