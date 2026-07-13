use std::collections::BTreeMap;

use crate::{
    ComponentNode, EffectStatementSyntaxKind, ExecutionBoundary, ExpressionGraph,
    IrReactiveTransitiveAnalysis, SemanticId, SemanticOwner, SemanticTypeModel, SourceProvenance,
    UnsupportedEffectStatementKind,
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

/// Compiler classification of an effect's language-semantic validity.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EffectValidation {
    Unvalidated,
    Valid,
    Invalid,
}

/// Unsupported behavior that makes an effect ineligible for later lowering.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum EffectSemanticViolationKind {
    Async,
    ReactiveStateMutation,
    ActionInvocation,
    EffectInvocation,
    ComponentMethodInvocation,
    UnresolvedComponentCall,
    UnresolvedComponentAssignment,
    UnknownExternalCapability,
    CapabilitySignature,
    CapabilityBoundary,
    CapabilitySerialization,
    ValueReturn,
    UnsupportedStatement,
}

/// One compiler-owned effect semantic violation with canonical provenance.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EffectSemanticViolation {
    pub kind: EffectSemanticViolationKind,
    pub statement: Option<SemanticId>,
    pub provenance: SourceProvenance,
}

/// Transitive reactive topology for one valid terminal effect.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EffectReactiveAnalysis {
    pub effect: SemanticId,
    pub dependencies: Vec<SemanticId>,
    pub dependents: Vec<SemanticId>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActionBatch {
    pub id: SemanticId,
    pub authored_action_method: SemanticId,
    pub ordered_write_records: Vec<SemanticId>,
    pub written_states: Vec<SemanticId>,
    pub provenance: SourceProvenance,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActionBatchEffectTrigger {
    pub action_batch: SemanticId,
    pub effects: Vec<SemanticId>,
    pub matched_states: BTreeMap<SemanticId, Vec<SemanticId>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct EffectTriggerPlan {
    pub initial_effects: Vec<SemanticId>,
    pub action_batches: BTreeMap<SemanticId, ActionBatch>,
    pub action_batch_triggers: Vec<ActionBatchEffectTrigger>,
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
    pub validation: EffectValidation,
    pub semantic_violations: Vec<EffectSemanticViolation>,
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
                            validation: EffectValidation::Unvalidated,
                            semantic_violations: Vec::new(),
                            provenance,
                        },
                    )
                })
        })
        .collect()
}

/// Classifies effect legality from F4 statement facts without adding diagnostics.
#[allow(clippy::too_many_lines)]
#[must_use]
pub fn validate_effects(
    components: &[ComponentNode],
    mut effects: BTreeMap<SemanticId, Effect>,
    statements: &BTreeMap<SemanticId, EffectStatement>,
    semantic_types: &SemanticTypeModel,
) -> BTreeMap<SemanticId, Effect> {
    for effect in effects.values_mut() {
        let mut violations = Vec::new();
        let method = effect
            .owner
            .entity_id()
            .and_then(|component_id| {
                components
                    .iter()
                    .find(|component| component.id == *component_id)
            })
            .and_then(|component| {
                component
                    .methods
                    .iter()
                    .find(|method| method.id == effect.method)
            });
        if method.is_some_and(|method| method.is_async) {
            violations.push(EffectSemanticViolation {
                kind: EffectSemanticViolationKind::Async,
                statement: None,
                provenance: effect.provenance.clone(),
            });
        }
        for statement in statements
            .values()
            .filter(|statement| statement.owner == effect.id)
        {
            let Some(record) = semantic_types.effect_statements.get(&statement.id) else {
                continue;
            };
            let statement_violation = match record.operation_classification {
                crate::EffectOperationClassification::ReactiveStateAssignment => {
                    Some(EffectSemanticViolationKind::ReactiveStateMutation)
                }
                crate::EffectOperationClassification::ComponentActionCall => {
                    Some(EffectSemanticViolationKind::ActionInvocation)
                }
                crate::EffectOperationClassification::ComponentEffectCall => {
                    Some(EffectSemanticViolationKind::EffectInvocation)
                }
                crate::EffectOperationClassification::ComponentMethodCall => {
                    Some(EffectSemanticViolationKind::ComponentMethodInvocation)
                }
                crate::EffectOperationClassification::UnresolvedComponentCall => {
                    Some(EffectSemanticViolationKind::UnresolvedComponentCall)
                }
                crate::EffectOperationClassification::UnresolvedComponentAssignment => {
                    Some(EffectSemanticViolationKind::UnresolvedComponentAssignment)
                }
                crate::EffectOperationClassification::UnknownExternalCapability => {
                    Some(EffectSemanticViolationKind::UnknownExternalCapability)
                }
                crate::EffectOperationClassification::ValueReturn => {
                    Some(EffectSemanticViolationKind::ValueReturn)
                }
                crate::EffectOperationClassification::UnsupportedStatement => {
                    Some(EffectSemanticViolationKind::UnsupportedStatement)
                }
                crate::EffectOperationClassification::RecognizedCapability
                | crate::EffectOperationClassification::BareReturn
                | crate::EffectOperationClassification::Empty => None,
            };
            if let Some(kind) = statement_violation {
                violations.push(EffectSemanticViolation {
                    kind,
                    statement: Some(statement.id.clone()),
                    provenance: statement.provenance.clone(),
                });
            }
            for (compatibility, kind) in [
                (
                    record.signature_compatibility,
                    EffectSemanticViolationKind::CapabilitySignature,
                ),
                (
                    record.boundary_compatibility,
                    EffectSemanticViolationKind::CapabilityBoundary,
                ),
                (
                    record.serialization_compatibility,
                    EffectSemanticViolationKind::CapabilitySerialization,
                ),
            ] {
                if compatibility != crate::EffectCompatibility::Compatible
                    && record.operation_classification
                        == crate::EffectOperationClassification::RecognizedCapability
                {
                    violations.push(EffectSemanticViolation {
                        kind,
                        statement: Some(statement.id.clone()),
                        provenance: statement.provenance.clone(),
                    });
                }
            }
        }
        violations.sort_by(|left, right| {
            (
                left.kind,
                left.provenance.path.as_path(),
                left.provenance.span.start,
                left.provenance.span.end,
            )
                .cmp(&(
                    right.kind,
                    right.provenance.path.as_path(),
                    right.provenance.span.start,
                    right.provenance.span.end,
                ))
        });
        violations.dedup_by(|left, right| {
            left.kind == right.kind
                && left.statement == right.statement
                && left.provenance == right.provenance
        });
        effect.validation = if violations.is_empty() {
            EffectValidation::Valid
        } else {
            EffectValidation::Invalid
        };
        effect.semantic_violations = violations;
    }
    effects
}

/// Projects existing reactive transitive analysis into effect-keyed records.
#[must_use]
pub fn analyze_effect_reactivity(
    components: &[ComponentNode],
    computed_values: &BTreeMap<SemanticId, crate::ComputedValue>,
    effects: &BTreeMap<SemanticId, Effect>,
    transitive: &IrReactiveTransitiveAnalysis,
) -> BTreeMap<SemanticId, EffectReactiveAnalysis> {
    let identities = components
        .iter()
        .flat_map(|component| component.state_fields.iter().map(|field| field.id.clone()))
        .chain(computed_values.keys().cloned())
        .chain(effects.keys().cloned())
        .map(|id| (id.as_str().to_string(), id))
        .collect::<BTreeMap<_, _>>();
    effects
        .values()
        .filter(|effect| effect.validation == EffectValidation::Valid)
        .map(|effect| {
            let map_ids = |ids: Option<&Vec<String>>| {
                ids.into_iter()
                    .flatten()
                    .filter_map(|id| identities.get(id).cloned())
                    .collect()
            };
            (
                effect.id.clone(),
                EffectReactiveAnalysis {
                    effect: effect.id.clone(),
                    dependencies: map_ids(transitive.dependencies.get(effect.id.as_str())),
                    dependents: map_ids(transitive.dependents.get(effect.id.as_str())),
                },
            )
        })
        .collect()
}

/// # Panics
///
/// Panics when an authored action method has no canonical source provenance.
#[must_use]
pub fn derive_effect_trigger_plan(
    components: &[ComponentNode],
    effects: &BTreeMap<SemanticId, Effect>,
    analysis: &BTreeMap<SemanticId, EffectReactiveAnalysis>,
    provenance: &BTreeMap<SemanticId, SourceProvenance>,
) -> EffectTriggerPlan {
    let initial_effects = effects
        .values()
        .filter(|effect| effect.validation == EffectValidation::Valid)
        .map(|effect| effect.id.clone())
        .collect::<Vec<_>>();
    let mut action_batches = BTreeMap::new();
    for component in components {
        for method in component.methods.iter().filter(|method| method.is_action()) {
            let writes = component
                .actions
                .iter()
                .filter(|action| action.method == method.name)
                .collect::<Vec<_>>();
            let id = component.id.action_batch(&method.name);
            action_batches.insert(
                id.clone(),
                ActionBatch {
                    id,
                    authored_action_method: method.id.clone(),
                    ordered_write_records: writes.iter().map(|action| action.id.clone()).collect(),
                    written_states: writes
                        .iter()
                        .map(|action| component.id.state_field(&action.field))
                        .collect::<std::collections::BTreeSet<_>>()
                        .into_iter()
                        .collect(),
                    provenance: provenance
                        .get(&method.id)
                        .expect("action methods should have canonical provenance")
                        .clone(),
                },
            );
        }
    }
    let action_batch_triggers = action_batches
        .values()
        .filter_map(|batch| {
            let matched_states = initial_effects
                .iter()
                .filter_map(|effect| {
                    let matches = analysis
                        .get(effect)?
                        .dependencies
                        .iter()
                        .filter(|dependency| batch.written_states.contains(dependency))
                        .cloned()
                        .collect::<Vec<_>>();
                    (!matches.is_empty()).then(|| (effect.clone(), matches))
                })
                .collect::<BTreeMap<_, _>>();
            (!matched_states.is_empty()).then(|| ActionBatchEffectTrigger {
                action_batch: batch.id.clone(),
                effects: matched_states.keys().cloned().collect(),
                matched_states,
            })
        })
        .collect();
    EffectTriggerPlan {
        initial_effects,
        action_batches,
        action_batch_triggers,
    }
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
        EffectSemanticViolationKind, EffectStatementKind, EffectValidation, ExecutionBoundary,
        ExpressionNodeKind, SemanticEntity, SemanticEntityKind, SemanticOwner,
        SemanticReferenceKind, UnsupportedEffectStatementKind,
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
        assert!(asm.semantic_types.assignments.keys().any(|id| id == target));
        assert_eq!(
            asm.semantic_type_of(value),
            Some(&crate::SemanticType::String)
        );

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

    #[test]
    #[allow(clippy::too_many_lines)]
    fn types_effect_statements_against_the_versioned_capability_registry() {
        let parsed = ezc_parser::parse_file(
            "src/Effects.tsx",
            r#"
@component("x-effects")
class Effects extends Component {
  title = state("EdgeZero");
  theme = state("dark");
  total = state(3);
  profile = state({ name: "Ada" });

  @action()
  increment() { this.total += 1; this.total += 1; }

  helper() {}

  @effect()
  sync() {
    document.title = this.title;
    console.log("total", this.total);
    console.info(this.title);
    console.warn(this.title);
    console.error(this.title);
    localStorage.setItem("theme", this.theme);
    localStorage.removeItem("theme");
    sessionStorage.setItem("theme", this.theme);
    sessionStorage.removeItem("theme");
  }

  @effect()
  facts() {
    document.title = this.profile;
    analytics.track(this.total);
    this.total = 1;
    this.increment();
    this.sync();
    this.helper();
    this.unknown();
    return;
  }

  render() { return <p />; }
}
"#,
        );
        let asm = build_application_semantic_model(&parsed);
        let component = &asm.components[0];
        let sync = asm
            .effect_body(&component.id.effect("sync"))
            .expect("sync body");
        let facts = asm
            .effect_body(&component.id.effect("facts"))
            .expect("facts body");
        let record = |statement: &crate::SemanticId| {
            asm.effect_statement_type(statement)
                .expect("statement type")
        };

        assert_eq!(crate::EFFECT_CAPABILITY_REGISTRY.version(), 1);
        assert_eq!(
            record(&sync.statements[0]).capability_operation,
            Some(crate::CapabilityOperationId(
                "builtin.browser.document.title.assign"
            ))
        );
        for statement in &sync.statements[1..5] {
            assert_eq!(
                record(statement).operation_classification,
                crate::EffectOperationClassification::RecognizedCapability
            );
            assert_eq!(
                record(statement).serialization_compatibility,
                crate::EffectCompatibility::Compatible
            );
        }
        for statement in &sync.statements[5..] {
            assert_eq!(
                record(statement).signature_compatibility,
                crate::EffectCompatibility::Compatible
            );
        }
        assert_eq!(
            record(&facts.statements[0]).signature_compatibility,
            crate::EffectCompatibility::Incompatible
        );
        assert_eq!(
            record(&facts.statements[1]).operation_classification,
            crate::EffectOperationClassification::UnknownExternalCapability
        );
        assert_eq!(
            record(&facts.statements[1]).signature_compatibility,
            crate::EffectCompatibility::Incompatible
        );
        assert_eq!(
            record(&facts.statements[1]).operand_types,
            vec![crate::SemanticType::Number]
        );
        assert_eq!(
            record(&facts.statements[2]).operation_classification,
            crate::EffectOperationClassification::ReactiveStateAssignment
        );
        assert_eq!(
            record(&facts.statements[3]).operation_classification,
            crate::EffectOperationClassification::ComponentActionCall
        );
        assert_eq!(
            record(&facts.statements[4]).operation_classification,
            crate::EffectOperationClassification::ComponentEffectCall
        );
        assert_eq!(
            record(&facts.statements[5]).operation_classification,
            crate::EffectOperationClassification::ComponentMethodCall
        );
        assert_eq!(
            record(&facts.statements[6]).operation_classification,
            crate::EffectOperationClassification::UnresolvedComponentCall
        );
        assert_eq!(
            record(&facts.statements[7]).operation_classification,
            crate::EffectOperationClassification::BareReturn
        );
        assert_eq!(
            asm.effect(&component.id.effect("sync"))
                .expect("valid effect")
                .validation,
            EffectValidation::Valid
        );
        let violations = &asm
            .effect(&component.id.effect("facts"))
            .expect("invalid effect")
            .semantic_violations;
        assert_eq!(
            asm.effect(&component.id.effect("facts"))
                .expect("invalid effect")
                .validation,
            EffectValidation::Invalid
        );
        for kind in [
            EffectSemanticViolationKind::CapabilitySignature,
            EffectSemanticViolationKind::UnknownExternalCapability,
            EffectSemanticViolationKind::ReactiveStateMutation,
            EffectSemanticViolationKind::ActionInvocation,
            EffectSemanticViolationKind::EffectInvocation,
            EffectSemanticViolationKind::ComponentMethodInvocation,
            EffectSemanticViolationKind::UnresolvedComponentCall,
        ] {
            assert!(violations.iter().any(|violation| violation.kind == kind));
        }
        let sync_id = component.id.effect("sync");
        let facts_id = component.id.effect("facts");
        assert_eq!(
            asm.reactive_graph.nodes[sync_id.as_str()].kind,
            crate::IrReactiveNodeKind::Effect
        );
        assert!(!asm.reactive_graph.nodes.contains_key(facts_id.as_str()));
        assert!(asm.reactive_graph.edges.iter().any(|edge| {
            edge.source == sync_id.as_str()
                && edge.target == component.id.state_field("title").as_str()
                && edge.kind == crate::IrReactiveEdgeKind::Reads
        }));
        assert!(asm.reactive_graph.edges.iter().any(|edge| {
            edge.source == component.id.state_field("title").as_str()
                && edge.target == sync_id.as_str()
                && edge.kind == crate::IrReactiveEdgeKind::Invalidates
        }));
        let analysis = asm
            .effect_reactive_analysis(&sync_id)
            .expect("effect reactive analysis");
        assert!(analysis
            .dependencies
            .contains(&component.id.state_field("title")));
        assert!(analysis
            .dependencies
            .contains(&component.id.state_field("total")));
        assert!(analysis.dependents.is_empty());
        assert!(asm.effect_reactive_analysis(&facts_id).is_none());
        let trigger_plan = asm.effect_trigger_plan();
        assert!(trigger_plan.initial_effects.contains(&sync_id));
        assert!(!trigger_plan.initial_effects.contains(&facts_id));
        let batch = trigger_plan
            .action_batches
            .get(&component.id.action_batch("increment"))
            .expect("increment action batch");
        assert_eq!(batch.ordered_write_records.len(), 2);
        assert_eq!(
            batch.written_states,
            vec![component.id.state_field("total")]
        );
        let trigger = trigger_plan
            .action_batch_triggers
            .iter()
            .find(|trigger| trigger.action_batch == batch.id)
            .expect("batch trigger");
        assert_eq!(trigger.effects, vec![sync_id.clone()]);
        assert_eq!(
            trigger.matched_states[&sync_id],
            vec![component.id.state_field("total")]
        );
        assert_eq!(validate_application_semantic_model(&asm), Vec::new());
    }

    #[test]
    fn rejects_async_effects_without_runtime_execution() {
        let parsed = ezc_parser::parse_file(
            "src/AsyncEffect.tsx",
            r#"
@component("x-async-effect")
class AsyncEffect extends Component {
  title = state("EdgeZero");

  @effect()
  async syncTitle() {
    document.title = this.title;
  }

  @effect()
  invalidValueReturn() {
    return this.title;
  }

  @effect()
  invalidCleanupReturn() {
    return () => unsubscribe();
  }

  render() { return <p />; }
}
"#,
        );
        let asm = build_application_semantic_model(&parsed);
        let effect = asm
            .effect(&asm.components[0].id.effect("syncTitle"))
            .expect("effect");

        assert_eq!(effect.validation, EffectValidation::Invalid);
        assert!(effect
            .semantic_violations
            .iter()
            .any(|violation| violation.kind == EffectSemanticViolationKind::Async));
        for (name, kind) in [
            (
                "invalidValueReturn",
                EffectSemanticViolationKind::ValueReturn,
            ),
            (
                "invalidCleanupReturn",
                EffectSemanticViolationKind::UnsupportedStatement,
            ),
        ] {
            let effect = asm
                .effect(&asm.components[0].id.effect(name))
                .expect("invalid return effect");
            assert_eq!(effect.validation, EffectValidation::Invalid);
            assert!(effect
                .semantic_violations
                .iter()
                .any(|violation| violation.kind == kind));
        }
        assert!(asm.diagnostics.is_empty());
    }
}
