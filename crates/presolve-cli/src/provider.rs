use std::{
    collections::{BTreeMap, BTreeSet},
    path::Path,
};

use anyhow::{Context, Result, bail};
use presolve_capability::{key_value_contract, object_store_contract};
use presolve_core::ProviderId;
use presolve_environment::{
    EnvironmentSpecification, ProvidedCapabilitySpecification, ProviderSpecification,
    inventory_from_specification,
};
use semver::Version;

use crate::{
    cli::BuiltinCapabilityAlias,
    environment::{EnvironmentDocument, load_environment, replace_environment},
};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct CapabilityKey {
    interface: String,
    version: Version,
}

#[derive(Debug)]
pub(crate) struct ProviderMutation {
    pub(crate) path: std::path::PathBuf,
    pub(crate) provider: ProviderSpecification,
}

pub(crate) fn expand_provider_supply(
    capabilities: &[String],
    features: &[String],
    provides: &[BuiltinCapabilityAlias],
    builtin_features: &[String],
) -> Result<(Vec<String>, Vec<String>)> {
    let mut expanded_capabilities = capabilities.to_vec();
    let mut expanded_features = features.to_vec();

    for alias in provides {
        let key = builtin_capability(*alias)?;
        expanded_capabilities.push(format!("{}@{}", key.interface, key.version));
    }

    for value in builtin_features {
        let (alias_name, feature_list) = value.split_once('=').ok_or_else(|| {
            anyhow::anyhow!(
                "invalid built-in feature declaration `{value}`; expected `CAPABILITY=FEATURE[,FEATURE...]`"
            )
        })?;

        let alias = BuiltinCapabilityAlias::parse(alias_name).ok_or_else(|| {
            anyhow::anyhow!(
                "unknown built-in capability alias `{alias_name}`; expected `kv` or `objects`"
            )
        })?;

        if feature_list.is_empty() {
            bail!(
                "invalid built-in feature declaration `{value}`; at least one feature is required"
            );
        }

        let key = builtin_capability(alias)?;

        for feature in feature_list.split(',') {
            if feature.is_empty() {
                bail!(
                    "invalid built-in feature declaration `{value}`; feature names may not be empty"
                );
            }

            expanded_features.push(format!("{}@{}={feature}", key.interface, key.version));
        }
    }

    Ok((expanded_capabilities, expanded_features))
}

fn builtin_capability(alias: BuiltinCapabilityAlias) -> Result<CapabilityKey> {
    let contract = match alias {
        BuiltinCapabilityAlias::Kv => {
            key_value_contract().context("built-in `kv` capability contract is invalid")?
        }
        BuiltinCapabilityAlias::Objects => {
            object_store_contract().context("built-in `objects` capability contract is invalid")?
        }
    };

    Ok(CapabilityKey {
        interface: contract.interface().to_owned(),
        version: contract.version().clone(),
    })
}

pub(crate) fn add_provider(
    start: &Path,
    name: &str,
    capabilities: &[String],
    features: &[String],
) -> Result<ProviderMutation> {
    let document = load_environment(start)?;

    if document
        .specification
        .providers()
        .iter()
        .any(|provider| provider.name() == name)
    {
        bail!(
            "provider `{name}` already exists in `{}`",
            document.path.display()
        );
    }

    let parsed_capabilities = parse_provider_supply(capabilities, features)?;

    let provider = ProviderSpecification::new(
        ProviderId::new(),
        name,
        parsed_capabilities
            .into_iter()
            .map(|(key, features)| {
                ProvidedCapabilitySpecification::new(key.interface, key.version)
                    .with_features(features)
            })
            .collect(),
    );

    let mut providers = document.specification.providers().to_vec();
    providers.push(provider.clone());

    let specification = rebuild_with_providers(&document.specification, providers)?;

    inventory_from_specification(&specification)
        .context("provider supply is not semantically valid for this Presolve release")?;

    replace_environment(&document.path, &specification)?;

    Ok(ProviderMutation {
        path: document.path,
        provider,
    })
}

pub(crate) fn remove_provider(start: &Path, name: &str) -> Result<ProviderMutation> {
    let document = load_environment(start)?;
    let mut providers = document.specification.providers().to_vec();

    let Some(index) = providers
        .iter()
        .position(|provider| provider.name() == name)
    else {
        bail!(
            "provider `{name}` does not exist in `{}`",
            document.path.display()
        );
    };

    let provider = providers.remove(index);
    let specification = rebuild_with_providers(&document.specification, providers)?;

    inventory_from_specification(&specification)
        .context("updated provider supply is not semantically valid")?;

    replace_environment(&document.path, &specification)?;

    Ok(ProviderMutation {
        path: document.path,
        provider,
    })
}

pub(crate) fn list_providers(start: &Path) -> Result<EnvironmentDocument> {
    load_environment(start)
}

fn rebuild_with_providers(
    current: &EnvironmentSpecification,
    providers: Vec<ProviderSpecification>,
) -> Result<EnvironmentSpecification> {
    EnvironmentSpecification::new(
        current.specification_version(),
        current.environment().clone(),
        current.resources(),
        current.execution().clone(),
        providers,
    )
    .context("could not update Environment Specification")
}

fn parse_provider_supply(
    capabilities: &[String],
    features: &[String],
) -> Result<BTreeMap<CapabilityKey, Vec<String>>> {
    let mut supply = BTreeMap::<CapabilityKey, BTreeSet<String>>::new();

    for value in capabilities {
        let key = parse_capability(value)?;

        if supply.insert(key.clone(), BTreeSet::new()).is_some() {
            bail!(
                "capability `{}@{}` was supplied more than once",
                key.interface,
                key.version
            );
        }
    }

    for value in features {
        let (capability, feature) = value.rsplit_once('=').ok_or_else(|| {
            anyhow::anyhow!("invalid feature `{value}`; expected `INTERFACE@VERSION=FEATURE`")
        })?;

        if feature.is_empty() {
            bail!("invalid feature `{value}`; feature name may not be empty");
        }

        let key = parse_capability(capability)?;
        let supplied_features = supply.get_mut(&key).ok_or_else(|| {
            anyhow::anyhow!(
                "feature `{feature}` targets undeclared capability `{}@{}`; \
                 add that exact capability with `--capability` first",
                key.interface,
                key.version
            )
        })?;

        if !supplied_features.insert(feature.to_owned()) {
            bail!(
                "feature `{feature}` was supplied more than once for `{}@{}`",
                key.interface,
                key.version
            );
        }
    }

    Ok(supply
        .into_iter()
        .map(|(key, features)| (key, features.into_iter().collect()))
        .collect())
}

fn parse_capability(value: &str) -> Result<CapabilityKey> {
    let (interface, version) = value.rsplit_once('@').ok_or_else(|| {
        anyhow::anyhow!("invalid capability `{value}`; expected `INTERFACE@VERSION`")
    })?;

    if interface.is_empty() {
        bail!("invalid capability `{value}`; interface may not be empty");
    }

    let version = Version::parse(version)
        .with_context(|| format!("invalid exact capability version in `{value}`"))?;

    Ok(CapabilityKey {
        interface: interface.to_owned(),
        version,
    })
}

#[cfg(test)]
mod tests {
    use std::{
        env, fs,
        path::{Path, PathBuf},
        sync::atomic::{AtomicU64, Ordering},
        time::{SystemTime, UNIX_EPOCH},
    };

    use presolve_environment::parse_environment;

    use crate::environment::create_environment;

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
                "presolve-cli-provider-{}-{timestamp}-{sequence}",
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
    fn add_provider_generates_identity_and_persists_supply() {
        let directory = TestDirectory::new();
        create_environment(directory.path(), "local", 1_024, 1_000)
            .expect("environment should be created");

        let mutation = add_provider(
            directory.path(),
            "memory-services",
            &[
                "presolve:kv/store@0.1.0".to_owned(),
                "presolve:objects/store@0.1.0".to_owned(),
            ],
            &[
                "presolve:objects/store@0.1.0=range-read".to_owned(),
                "presolve:objects/store@0.1.0=user-metadata".to_owned(),
            ],
        )
        .expect("provider should be added");

        assert!(mutation.provider.id().to_string().starts_with("prv_"));
        assert_eq!(mutation.provider.name(), "memory-services");
        assert_eq!(mutation.provider.capabilities().len(), 2);

        let persisted =
            fs::read_to_string(&mutation.path).expect("updated environment should be readable");
        let specification =
            parse_environment(&persisted).expect("updated environment should parse");

        assert_eq!(specification.providers().len(), 1);
        assert_eq!(specification.providers()[0], mutation.provider);
    }

    #[test]
    fn add_provider_rejects_duplicate_name() {
        let directory = TestDirectory::new();
        create_environment(directory.path(), "local", 1_024, 1_000)
            .expect("environment should be created");

        add_provider(
            directory.path(),
            "kv",
            &["presolve:kv/store@0.1.0".to_owned()],
            &[],
        )
        .expect("first provider should be added");

        let error = add_provider(
            directory.path(),
            "kv",
            &["presolve:kv/store@0.1.0".to_owned()],
            &[],
        )
        .expect_err("duplicate provider name should fail");

        assert!(error.to_string().contains("already exists"));
    }

    #[test]
    fn add_provider_rejects_duplicate_capability_argument() {
        let error = parse_provider_supply(
            &[
                "presolve:kv/store@0.1.0".to_owned(),
                "presolve:kv/store@0.1.0".to_owned(),
            ],
            &[],
        )
        .expect_err("duplicate capability should fail");

        assert!(error.to_string().contains("supplied more than once"));
    }

    #[test]
    fn add_provider_rejects_feature_for_undeclared_capability() {
        let error = parse_provider_supply(
            &["presolve:kv/store@0.1.0".to_owned()],
            &["presolve:objects/store@0.1.0=range-read".to_owned()],
        )
        .expect_err("feature for undeclared capability should fail");

        assert!(error.to_string().contains("targets undeclared capability"));
    }

    #[test]
    fn add_provider_rejects_unknown_builtin_version_semantically() {
        let directory = TestDirectory::new();
        create_environment(directory.path(), "local", 1_024, 1_000)
            .expect("environment should be created");

        let error = add_provider(
            directory.path(),
            "future-kv",
            &["presolve:kv/store@9.0.0".to_owned()],
            &[],
        )
        .expect_err("unknown built-in version should fail");

        assert!(
            error
                .to_string()
                .contains("provider supply is not semantically valid")
        );
    }

    #[test]
    fn remove_provider_persists_new_document_and_readd_gets_new_identity() {
        let directory = TestDirectory::new();
        create_environment(directory.path(), "local", 1_024, 1_000)
            .expect("environment should be created");

        let first = add_provider(
            directory.path(),
            "kv",
            &["presolve:kv/store@0.1.0".to_owned()],
            &[],
        )
        .expect("provider should be added");

        let removed = remove_provider(directory.path(), "kv").expect("provider should be removed");

        assert_eq!(removed.provider.id(), first.provider.id());

        let after_remove = list_providers(directory.path()).expect("environment should load");
        assert_eq!(after_remove.specification.providers(), []);

        let second = add_provider(
            directory.path(),
            "kv",
            &["presolve:kv/store@0.1.0".to_owned()],
            &[],
        )
        .expect("provider should be re-added");

        assert_ne!(second.provider.id(), first.provider.id());
    }

    #[test]
    fn remove_provider_rejects_unknown_name() {
        let directory = TestDirectory::new();
        create_environment(directory.path(), "local", 1_024, 1_000)
            .expect("environment should be created");

        let error =
            remove_provider(directory.path(), "missing").expect_err("unknown provider should fail");

        assert!(error.to_string().contains("does not exist"));
    }

    #[test]
    fn custom_capability_without_features_is_preserved() {
        let directory = TestDirectory::new();
        create_environment(directory.path(), "local", 1_024, 1_000)
            .expect("environment should be created");

        let mutation = add_provider(
            directory.path(),
            "custom",
            &["example:search/index@1.2.3".to_owned()],
            &[],
        )
        .expect("custom generic capability should be accepted");

        assert_eq!(
            mutation.provider.capabilities()[0].interface(),
            "example:search/index"
        );
    }

    #[test]
    fn custom_capability_feature_claim_is_rejected_semantically() {
        let directory = TestDirectory::new();
        create_environment(directory.path(), "local", 1_024, 1_000)
            .expect("environment should be created");

        let error = add_provider(
            directory.path(),
            "custom",
            &["example:search/index@1.2.3".to_owned()],
            &["example:search/index@1.2.3=ranking".to_owned()],
        )
        .expect_err("custom semantic feature claim should fail");

        assert!(
            error
                .to_string()
                .contains("provider supply is not semantically valid")
        );
    }

    #[test]
    fn shorthand_expands_from_canonical_builtin_contracts() {
        let (capabilities, features) = expand_provider_supply(
            &[],
            &[],
            &[BuiltinCapabilityAlias::Kv, BuiltinCapabilityAlias::Objects],
            &["objects=range-read,user-metadata".to_owned()],
        )
        .expect("built-in shorthand should expand");

        assert_eq!(
            capabilities,
            ["presolve:kv/store@0.1.0", "presolve:objects/store@0.1.0"]
        );
        assert_eq!(
            features,
            [
                "presolve:objects/store@0.1.0=range-read",
                "presolve:objects/store@0.1.0=user-metadata"
            ]
        );
    }

    #[test]
    fn shorthand_features_can_target_explicit_builtin_capability() {
        let (capabilities, features) = expand_provider_supply(
            &["presolve:objects/store@0.1.0".to_owned()],
            &[],
            &[],
            &["objects=range-read".to_owned()],
        )
        .expect("shorthand features should compose with explicit capability");

        let supply =
            parse_provider_supply(&capabilities, &features).expect("expanded supply should parse");

        assert_eq!(supply.len(), 1);
        assert_eq!(
            supply
                .values()
                .next()
                .expect("object capability should be present"),
            &["range-read".to_owned()]
        );
    }

    #[test]
    fn shorthand_rejects_unknown_alias_in_features() {
        let error = expand_provider_supply(
            &["presolve:objects/store@0.1.0".to_owned()],
            &[],
            &[],
            &["blob=range-read".to_owned()],
        )
        .expect_err("unknown alias should fail");

        assert!(
            error
                .to_string()
                .contains("unknown built-in capability alias")
        );
    }

    #[test]
    fn shorthand_feature_requires_supplied_target_after_expansion() {
        let (capabilities, features) = expand_provider_supply(
            &["presolve:kv/store@0.1.0".to_owned()],
            &[],
            &[],
            &["objects=range-read".to_owned()],
        )
        .expect("shorthand should expand before semantic validation");

        let error = parse_provider_supply(&capabilities, &features)
            .expect_err("feature without object capability should fail");

        assert!(error.to_string().contains("targets undeclared capability"));
    }
}
