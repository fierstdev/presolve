use std::collections::BTreeMap;

use crate::{ComponentNode, ExecutionBoundary, SemanticId, SemanticOwner, SourceProvenance};

/// Compiler-owned cache behavior for a computed value.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ComputedCachePolicy {
    Memoized,
}

/// Compiler classification of a computed getter's purity.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ComputedPurity {
    Unclassified,
    Pure,
    Impure,
}

/// Stable compiler diagnostics emitted for computed declarations and values.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ComputedDiagnosticCode {
    PurityViolation,
    DependencyCycle,
    InvalidDeclaration,
    UnsupportedBody,
    UnresolvedRead,
    TypeMismatch,
    SerializationViolation,
}

impl ComputedDiagnosticCode {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::PurityViolation => "PSC1034",
            Self::DependencyCycle => "PSC1035",
            Self::InvalidDeclaration => "PSC1036",
            Self::UnsupportedBody => "PSC1037",
            Self::UnresolvedRead => "PSC1038",
            Self::TypeMismatch => "PSC1039",
            Self::SerializationViolation => "PSC1040",
        }
    }
}

/// Unsupported behavior that makes a computed getter impure.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ComputedPurityViolationKind {
    StateMutation,
    Action,
    Effect,
    Async,
    Resource,
    ArbitraryMethodCall,
    NondeterministicOperation,
}

impl ComputedPurityViolationKind {
    #[must_use]
    pub const fn description(self) -> &'static str {
        match self {
            Self::StateMutation => "state mutation",
            Self::Action => "action invocation",
            Self::Effect => "effectful operation",
            Self::Async => "async execution",
            Self::Resource => "resource access",
            Self::ArbitraryMethodCall => "arbitrary method call",
            Self::NondeterministicOperation => "nondeterministic operation",
        }
    }
}

/// One compiler-owned purity violation with its authored location.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComputedPurityViolation {
    pub kind: ComputedPurityViolationKind,
    pub provenance: SourceProvenance,
}

/// A first-class compiler semantic entity for one `@computed()` getter.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComputedValue {
    pub id: SemanticId,
    pub owner: SemanticOwner,
    pub method: SemanticId,
    pub name: String,
    pub cache_policy: ComputedCachePolicy,
    pub purity: ComputedPurity,
    pub purity_violations: Vec<ComputedPurityViolation>,
    pub execution_boundary: ExecutionBoundary,
    pub provenance: SourceProvenance,
}

/// Collect canonical computed entities in stable semantic-ID order.
///
/// # Panics
///
/// Panics when a computed method has no canonical source provenance.
#[must_use]
pub fn collect_computed_values(
    components: &[ComponentNode],
    provenance: &BTreeMap<SemanticId, SourceProvenance>,
) -> BTreeMap<SemanticId, ComputedValue> {
    components
        .iter()
        .flat_map(|component| {
            component
                .methods
                .iter()
                .filter(|method| method.is_computed())
                .map(move |method| {
                    let id = component.id.computed(&method.name);
                    let provenance = provenance
                        .get(&method.id)
                        .expect("computed methods should have canonical provenance")
                        .clone();
                    (
                        id.clone(),
                        ComputedValue {
                            id,
                            owner: SemanticOwner::entity(component.id.clone()),
                            method: method.id.clone(),
                            name: method.name.clone(),
                            cache_policy: ComputedCachePolicy::Memoized,
                            purity: ComputedPurity::Unclassified,
                            purity_violations: Vec::new(),
                            execution_boundary: ExecutionBoundary::Client,
                            provenance,
                        },
                    )
                })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use crate::{
        build_component_graph, collect_computed_values, ComputedCachePolicy, ComputedPurity,
        ExecutionBoundary,
    };

    #[test]
    fn collects_stable_computed_entities_from_decorated_getters() {
        let parsed = presolve_parser::parse_file(
            "src/Computed.tsx",
            r#"
@component("x-computed")
class Computed extends Component {
  @computed()
  get remainingCount(): number { return 1; }
}
"#,
        );
        let graph = build_component_graph(&parsed);
        let component = &graph.components[0];
        let computed = collect_computed_values(&graph.components, &graph.provenance)
            .into_values()
            .next()
            .expect("computed entity");

        assert_eq!(
            computed.id.as_str(),
            "component:x-computed/computed:remainingCount"
        );
        assert_eq!(computed.method, component.methods[0].id);
        assert_eq!(computed.owner, component.methods[0].owner);
        assert_eq!(computed.cache_policy, ComputedCachePolicy::Memoized);
        assert_eq!(computed.purity, ComputedPurity::Unclassified);
        assert!(computed.purity_violations.is_empty());
        assert_eq!(computed.execution_boundary, ExecutionBoundary::Client);
    }
}
