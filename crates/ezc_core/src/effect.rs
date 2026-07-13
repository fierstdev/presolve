use std::collections::BTreeMap;

use crate::{
    ComponentNode, EffectStatementSyntaxKind, ExecutionBoundary, ExpressionGraph, SemanticId,
    SemanticOwner, SourceProvenance, UnsupportedEffectStatementKind,
};

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

/// An ordered, compiler-owned effect body.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EffectBody {
    pub effect: SemanticId,
    pub statements: Vec<SemanticId>,
    pub provenance: SourceProvenance,
}

/// One compiler-owned statement belonging to an effect body.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EffectStatement {
    pub id: SemanticId,
    pub owner: SemanticId,
    pub kind: EffectStatementKind,
    pub provenance: SourceProvenance,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EffectStatementKind {
    ExternalMemberAssignment {
        target: SemanticId,
        value: SemanticId,
    },
    CapabilityCall {
        callee: SemanticId,
        arguments: Vec<SemanticId>,
    },
    EffectReturn {
        value: Option<SemanticId>,
    },
    Empty,
    Unsupported(UnsupportedEffectStatementKind),
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

/// Lower all authored effect bodies to ordered statement records.
#[must_use]
pub fn lower_effect_bodies(
    components: &[ComponentNode],
    effects: &BTreeMap<SemanticId, Effect>,
    expression_graph: &ExpressionGraph,
) -> (
    BTreeMap<SemanticId, EffectBody>,
    BTreeMap<SemanticId, EffectStatement>,
) {
    let mut bodies = BTreeMap::new();
    let mut statements = BTreeMap::new();
    for effect in effects.values() {
        let Some(component_id) = effect.owner.entity_id() else {
            continue;
        };
        let Some(method) = components
            .iter()
            .find(|component| component.id == *component_id)
            .and_then(|component| {
                component
                    .methods
                    .iter()
                    .find(|method| method.id == effect.method)
            })
        else {
            continue;
        };
        let Some(syntax) = &method.effect_body else {
            continue;
        };
        let mut body_statement_ids = Vec::new();
        for (index, statement) in syntax.statements.iter().enumerate() {
            let id = effect.id.effect_statement(index);
            let path = format!("statement:{index}");
            let expression = |suffix: &str| effect.id.expression(&format!("{path}/{suffix}"));
            let kind = match &statement.kind {
                EffectStatementSyntaxKind::StaticMemberAssignment { .. } => {
                    EffectStatementKind::ExternalMemberAssignment {
                        target: expression("target"),
                        value: expression("value"),
                    }
                }
                EffectStatementSyntaxKind::CapabilityCall { arguments, .. } => {
                    EffectStatementKind::CapabilityCall {
                        callee: expression("callee"),
                        arguments: (0..arguments.len())
                            .map(|argument| expression(&format!("argument:{argument}")))
                            .collect(),
                    }
                }
                EffectStatementSyntaxKind::EffectReturn { value } => {
                    EffectStatementKind::EffectReturn {
                        value: value.as_ref().map(|_| expression("return")),
                    }
                }
                EffectStatementSyntaxKind::Empty => EffectStatementKind::Empty,
                EffectStatementSyntaxKind::Unsupported(kind) => {
                    EffectStatementKind::Unsupported(*kind)
                }
            };
            assert_effect_statement_expressions_exist(&kind, expression_graph);
            body_statement_ids.push(id.clone());
            statements.insert(
                id.clone(),
                EffectStatement {
                    id,
                    owner: effect.id.clone(),
                    kind,
                    provenance: SourceProvenance::new(&effect.provenance.path, statement.span),
                },
            );
        }
        bodies.insert(
            effect.id.clone(),
            EffectBody {
                effect: effect.id.clone(),
                statements: body_statement_ids,
                provenance: effect.provenance.clone(),
            },
        );
    }
    (bodies, statements)
}

fn assert_effect_statement_expressions_exist(kind: &EffectStatementKind, graph: &ExpressionGraph) {
    let expressions = match kind {
        EffectStatementKind::ExternalMemberAssignment { target, value } => vec![target, value],
        EffectStatementKind::CapabilityCall { callee, arguments } => {
            let mut expressions = vec![callee];
            expressions.extend(arguments);
            expressions
        }
        EffectStatementKind::EffectReturn { value } => value.iter().collect(),
        EffectStatementKind::Empty | EffectStatementKind::Unsupported(_) => Vec::new(),
    };
    assert!(expressions.iter().all(|id| graph.node(id).is_some()));
}

#[cfg(test)]
mod tests {
    use crate::{
        build_application_semantic_model, build_component_graph, build_semantic_graph,
        collect_effects, validate_application_semantic_model, EffectExecutionPolicy,
        EffectStatementKind, ExecutionBoundary, ExpressionNodeKind, SemanticEntity,
        SemanticEntityKind, SemanticOwner, SemanticReferenceKind, UnsupportedEffectStatementKind,
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
        assert!(asm.references_from(&effect_id).iter().any(|reference| {
            reference.kind == SemanticReferenceKind::EffectState
                && reference.target == component.id.state_field("title")
        }));
        assert!(asm.semantic_type_of(&effect_id).is_none());
        assert_eq!(validate_application_semantic_model(&asm), Vec::new());
        assert!(build_semantic_graph(&asm)
            .nodes
            .iter()
            .any(|node| node.id == effect_id));
    }

    #[test]
    fn lowers_ordered_effect_statements_and_expression_operands_without_resolution() {
        let parsed = ezc_parser::parse_file(
            "src/Effects.tsx",
            include_str!("../../../fixtures/0052-effect-body-lowering/input/Effects.tsx"),
        );
        let asm = build_application_semantic_model(&parsed);
        let component = &asm.components[0];
        let sync = component.id.effect("sync");
        let body = asm.effect_body(&sync).expect("effect body");

        assert_eq!(body.statements.len(), 3);
        let assignment = asm
            .effect_statement(&body.statements[0])
            .expect("assignment");
        let call = asm.effect_statement(&body.statements[1]).expect("call");
        let completion = asm.effect_statement(&body.statements[2]).expect("return");
        let EffectStatementKind::ExternalMemberAssignment { target, value } = &assignment.kind
        else {
            panic!("expected static member assignment");
        };
        assert!(matches!(
            asm.expression(target).map(|node| &node.kind),
            Some(ExpressionNodeKind::MemberAccess { property, .. }) if property == "title"
        ));
        assert!(matches!(
            asm.expression(value).map(|node| &node.kind),
            Some(ExpressionNodeKind::ThisMember { name }) if name == "title"
        ));
        let EffectStatementKind::CapabilityCall { callee, arguments } = &call.kind else {
            panic!("expected capability call");
        };
        assert_eq!(arguments.len(), 2);
        assert!(matches!(
            asm.expression(callee).map(|node| &node.kind),
            Some(ExpressionNodeKind::MemberAccess { property, .. }) if property == "track"
        ));
        assert!(matches!(
            asm.expression(&arguments[1]).map(|node| &node.kind),
            Some(ExpressionNodeKind::Arithmetic { .. })
        ));
        assert!(matches!(
            completion.kind,
            EffectStatementKind::EffectReturn { value: None }
        ));
        assert!(asm.references_from(&sync).iter().any(|reference| {
            reference.kind == SemanticReferenceKind::EffectState
                && reference.target == component.id.state_field("title")
        }));
        assert!(asm
            .semantic_types
            .assignments
            .keys()
            .all(|id| id != target && id != value));

        let invalid = asm
            .effect_body(&component.id.effect("invalid"))
            .expect("invalid body");
        assert!(matches!(
            asm.effect_statement(&invalid.statements[0])
                .map(|statement| &statement.kind),
            Some(EffectStatementKind::ExternalMemberAssignment { .. })
        ));
        assert!(matches!(
            asm.effect_statement(&invalid.statements[1])
                .map(|statement| &statement.kind),
            Some(EffectStatementKind::CapabilityCall { .. })
        ));
        assert!(matches!(
            asm.effect_statement(&invalid.statements[2])
                .map(|statement| &statement.kind),
            Some(EffectStatementKind::Unsupported(
                UnsupportedEffectStatementKind::CleanupReturnCandidate
            ))
        ));
        assert!(matches!(
            asm.effect_statement(&invalid.statements[3])
                .map(|statement| &statement.kind),
            Some(EffectStatementKind::Unsupported(
                UnsupportedEffectStatementKind::LocalDeclaration
            ))
        ));
    }
}
