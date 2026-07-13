use std::collections::BTreeMap;

use crate::{ComponentNode, ExecutionBoundary, SemanticId, SemanticOwner, SourceProvenance};

/// Compiler-owned execution contract for an effect.
///
/// The scheduler will consume this policy in later Phase F slices. It is
/// declared here so effect timing remains compiler semantics rather than a
/// runtime callback convention.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EffectExecutionPolicy {
    AfterInitialRenderAndCompletedActionBatch,
}

/// A first-class compiler semantic entity for one `@effect()` method.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Effect {
    pub id: SemanticId,
    pub owner: SemanticOwner,
    pub method: SemanticId,
    pub name: String,
    pub execution_boundary: ExecutionBoundary,
    pub execution_policy: EffectExecutionPolicy,
    pub provenance: SourceProvenance,
}

/// Collect canonical effect entities in stable semantic-ID order.
///
/// # Panics
///
/// Panics when an effect method has no canonical source provenance.
#[must_use]
pub fn collect_effects(
    components: &[ComponentNode],
    provenance: &BTreeMap<SemanticId, SourceProvenance>,
) -> BTreeMap<SemanticId, Effect> {
    components
        .iter()
        .flat_map(|component| {
            component
                .methods
                .iter()
                .filter(|method| method.is_effect())
                .map(move |method| {
                    let id = component.id.effect(&method.name);
                    let provenance = provenance
                        .get(&method.id)
                        .expect("effect methods should have canonical provenance")
                        .clone();
                    (
                        id.clone(),
                        Effect {
                            id,
                            owner: SemanticOwner::entity(component.id.clone()),
                            method: method.id.clone(),
                            name: method.name.clone(),
                            execution_boundary: ExecutionBoundary::Client,
                            execution_policy:
                                EffectExecutionPolicy::AfterInitialRenderAndCompletedActionBatch,
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
        build_application_semantic_model, build_component_graph, build_semantic_graph,
        collect_effects, validate_application_semantic_model, EffectExecutionPolicy,
        ExecutionBoundary, SemanticEntity, SemanticEntityKind, SemanticOwner,
    };

    #[test]
    fn collects_stable_effect_entities_from_decorated_methods() {
        let parsed = ezc_parser::parse_file(
            "src/Effects.tsx",
            r#"
@component("x-effects")
class Effects extends Component {
  @effect()
  syncTitle() {
    document.title = "EdgeZero";
  }
}
"#,
        );
        let graph = build_component_graph(&parsed);
        let component = &graph.components[0];
        let effect = collect_effects(&graph.components, &graph.provenance)
            .into_values()
            .next()
            .expect("effect entity");

        assert_eq!(effect.id.as_str(), "component:x-effects/effect:syncTitle");
        assert_eq!(effect.method, component.methods[0].id);
        assert_eq!(effect.owner, component.methods[0].owner);
        assert_eq!(effect.execution_boundary, ExecutionBoundary::Client);
        assert_eq!(
            effect.execution_policy,
            EffectExecutionPolicy::AfterInitialRenderAndCompletedActionBatch
        );
    }

    #[test]
    fn assembles_effects_into_canonical_asm_without_reactive_products() {
        let parsed = ezc_parser::parse_file(
            "src/Effects.tsx",
            r#"
@component("x-effects")
class Effects extends Component {
  title = state("EdgeZero");

  @effect()
  syncTitle() {
    document.title = this.title;
  }

  render() {
    return <p>{this.title}</p>;
  }
}
"#,
        );
        let asm = build_application_semantic_model(&parsed);
        let component = &asm.components[0];
        let effect_id = component.id.effect("syncTitle");
        let effect = asm.effect(&effect_id).expect("effect entity");

        assert_eq!(effect.method, component.id.method("syncTitle"));
        assert_eq!(effect.owner, SemanticOwner::entity(component.id.clone()));
        assert_eq!(asm.owner(&effect_id), Some(&effect.owner));
        assert_eq!(asm.provenance(&effect_id), asm.provenance(&effect.method));
        assert_eq!(
            asm.entity(&effect_id).map(SemanticEntity::kind),
            Some(SemanticEntityKind::Effect)
        );
        assert!(asm.references_from(&effect_id).is_empty());
        assert!(asm.semantic_type_of(&effect_id).is_none());
        assert_eq!(validate_application_semantic_model(&asm), Vec::new());
        assert!(build_semantic_graph(&asm)
            .nodes
            .iter()
            .any(|node| node.id == effect_id));
    }
}
