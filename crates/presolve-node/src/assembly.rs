use presolve_core::{ProviderId, WorkloadKind};
use presolve_provider_sdk::{KEY_VALUE_INTERFACE, OBJECT_STORE_INTERFACE, SQL_DATABASE_INTERFACE};
use presolve_resolver::DeploymentPlan;
use presolve_runtime::{Runtime, RuntimeError};
use thiserror::Error;

use crate::ProviderRegistry;

/// Failure while materializing a symbolic deployment plan.
#[derive(Debug, Error)]
pub enum AssemblyError {
    /// This runtime assembler only materializes WebAssembly Component workloads.
    #[error("component runtime cannot materialize workload `{workload}` of kind `{kind}`")]
    UnsupportedWorkload {
        /// Workload that cannot be materialized by this runtime.
        workload: String,
        /// Workload execution class selected by the resolver.
        kind: WorkloadKind,
    },

    /// The deployment plan references a capability this node cannot yet bind.
    #[error("node cannot materialize capability `{interface}`")]
    UnsupportedCapability { interface: String },

    /// The resolver selected a provider descriptor for which the node has no concrete implementation.
    #[error("provider `{provider_id}` selected for capability `{interface}` has no registered implementation")]
    MissingProvider {
        interface: String,
        provider_id: ProviderId,
    },

    /// Runtime construction failed.
    #[error(transparent)]
    Runtime(#[from] RuntimeError),
}

/// Materializes a symbolic deployment plan into an executable runtime.
///
/// # Errors
///
/// Returns a categorized materialization error when the selected workload or
/// capability cannot be assembled, a selected implementation is missing, or
/// runtime construction fails.
pub fn assemble_runtime(
    plan: &DeploymentPlan,
    registry: &ProviderRegistry,
) -> Result<Runtime, AssemblyError> {
    for workload in plan.workloads() {
        if workload.kind() != WorkloadKind::Component {
            return Err(AssemblyError::UnsupportedWorkload {
                workload: workload.name().to_owned(),
                kind: workload.kind(),
            });
        }
    }

    let mut builder = Runtime::builder();

    for binding in plan.bindings() {
        match binding.interface() {
            KEY_VALUE_INTERFACE => {
                let provider_id = *binding.provider_id();
                let provider = registry.key_value(provider_id).ok_or_else(|| {
                    AssemblyError::MissingProvider {
                        interface: binding.interface().to_owned(),
                        provider_id,
                    }
                })?;
                builder = builder.key_value_provider(provider);
            }
            OBJECT_STORE_INTERFACE => {
                let provider_id = *binding.provider_id();
                let provider = registry.object_store(provider_id).ok_or_else(|| {
                    AssemblyError::MissingProvider {
                        interface: binding.interface().to_owned(),
                        provider_id,
                    }
                })?;
                builder = builder.object_store_provider(provider);
            }
            SQL_DATABASE_INTERFACE => {
                let provider_id = *binding.provider_id();
                let provider = registry.sql_database(provider_id).ok_or_else(|| {
                    AssemblyError::MissingProvider {
                        interface: binding.interface().to_owned(),
                        provider_id,
                    }
                })?;
                builder = builder.sql_database_provider(provider);
            }
            interface => {
                return Err(AssemblyError::UnsupportedCapability {
                    interface: interface.to_owned(),
                });
            }
        }
    }

    builder.build().map_err(AssemblyError::Runtime)
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use presolve_contract::parse_contract;
    use presolve_core::{EnvironmentId, ProviderId};
    use presolve_provider_objects_memory::InMemoryObjectStoreProvider;
    use presolve_provider_sdk::UnavailableSqlDatabaseProvider;
    use presolve_resolver::{
        EnvironmentInventory, EnvironmentResources, ProvidedCapability, ProviderDescriptor, resolve,
    };
    use semver::Version;

    use super::*;

    fn plan_for(interface: &str, provider_id: ProviderId) -> presolve_resolver::DeploymentPlan {
        let contract = parse_contract(&format!(
            r#"
contract_version = "0.1"

[application]
name = "node-capability-test"
version = "0.0.1"

[[capabilities]]
interface = "{interface}"
version = "^0.1"

[resources]

[network]
outbound = "deny"
"#
        ))
        .expect("test contract should parse");

        let environment = EnvironmentInventory::new(
            EnvironmentId::new(),
            EnvironmentResources::new(512, 1_000),
            vec![ProviderDescriptor::new(
                provider_id,
                "test-provider",
                vec![ProvidedCapability::new(interface, Version::new(0, 1, 0))],
            )],
        );

        resolve(&contract, &environment)
            .plan()
            .expect("capability requirement should resolve")
            .clone()
    }

    #[test]
    fn component_runtime_rejects_container_workload() {
        let contract = parse_contract(
            r#"
contract_version = "0.1"

[application]
name = "container-node-test"
version = "0.0.1"

[[workloads]]
name = "api"
kind = "container"
"#,
        )
        .expect("container contract should parse");
        let environment = EnvironmentInventory::new(
            EnvironmentId::new(),
            EnvironmentResources::new(512, 1_000),
            Vec::new(),
        )
        .with_supported_workloads(vec![WorkloadKind::Container]);
        let plan = resolve(&contract, &environment)
            .plan()
            .expect("container workload should resolve semantically")
            .clone();
        let registry = ProviderRegistry::new();

        let Err(error) = assemble_runtime(&plan, &registry) else {
            panic!("component runtime should reject container workload");
        };
        assert!(matches!(
            error,
            AssemblyError::UnsupportedWorkload {
                workload,
                kind: WorkloadKind::Container,
            } if workload == "api"
        ));
    }

    #[test]
    fn assembles_object_storage_provider_from_resolved_plan() {
        let provider_id = ProviderId::new();
        let plan = plan_for(OBJECT_STORE_INTERFACE, provider_id);
        let mut registry = ProviderRegistry::new();
        registry
            .register_object_store(provider_id, Arc::new(InMemoryObjectStoreProvider::new()))
            .expect("object-store registration should succeed");
        assemble_runtime(&plan, &registry).expect("object-store plan should materialize");
    }

    #[test]
    fn assembles_sql_provider_from_resolved_plan() {
        let provider_id = ProviderId::new();
        let plan = plan_for(SQL_DATABASE_INTERFACE, provider_id);
        let mut registry = ProviderRegistry::new();
        registry
            .register_sql_database(provider_id, Arc::new(UnavailableSqlDatabaseProvider))
            .expect("SQL registration should succeed");
        assemble_runtime(&plan, &registry).expect("SQL plan should materialize");
    }

    #[test]
    fn missing_object_storage_provider_is_reported() {
        let provider_id = ProviderId::new();
        let plan = plan_for(OBJECT_STORE_INTERFACE, provider_id);
        let registry = ProviderRegistry::new();
        let Err(error) = assemble_runtime(&plan, &registry) else {
            panic!("missing provider should be rejected");
        };
        assert!(matches!(
            error,
            AssemblyError::MissingProvider {
                interface,
                provider_id: missing_provider_id,
            } if interface == OBJECT_STORE_INTERFACE && missing_provider_id == provider_id
        ));
    }
}
