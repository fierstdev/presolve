//! Assembles decorator-free V2 authoring forms into one canonical model.

use presolve_parser::ParsedFile;

use crate::{
    compose_authored_semantics_v1, lower_action_fields_v1, lower_component_inheritance_v1,
    lower_computed_getters_v1, lower_state_initializers_v1, ActionFieldLoweringErrorV1,
    AuthoredSemanticCompositionErrorV1, CanonicalAuthoredSemanticModelV1,
    ComponentInheritanceLoweringErrorV1, ComputedGetterLoweringErrorV1, EffectFieldLoweringErrorV1,
    ResolvedActionFieldV1, ResolvedComponentInheritanceV1, ResolvedEffectFieldV1,
    ResolvedStateInitializerV1, StateInitializerLoweringErrorV1,
};

/// The authority-resolved inputs for the currently implemented V2 source forms.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct V2AuthoringResolutionsV1 {
    pub components: Vec<ResolvedComponentInheritanceV1>,
    pub states: Vec<ResolvedStateInitializerV1>,
    pub actions: Vec<ResolvedActionFieldV1>,
    pub effects: Vec<ResolvedEffectFieldV1>,
}

/// The unified decorator-free canonical model and its constituent proof products.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct V2AuthoringLoweringV1 {
    pub model: CanonicalAuthoredSemanticModelV1,
    pub component_count: usize,
    pub state_count: usize,
    pub action_count: usize,
    pub computed_count: usize,
    pub effect_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum V2AuthoringLoweringErrorV1 {
    Component(ComponentInheritanceLoweringErrorV1),
    State(StateInitializerLoweringErrorV1),
    Action(ActionFieldLoweringErrorV1),
    Effect(EffectFieldLoweringErrorV1),
    Computed(ComputedGetterLoweringErrorV1),
    Composition(AuthoredSemanticCompositionErrorV1),
}

impl std::fmt::Display for V2AuthoringLoweringErrorV1 {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Component(error) => error.fmt(formatter),
            Self::State(error) => error.fmt(formatter),
            Self::Action(error) => error.fmt(formatter),
            Self::Effect(error) => error.fmt(formatter),
            Self::Computed(error) => error.fmt(formatter),
            Self::Composition(error) => error.fmt(formatter),
        }
    }
}

impl std::error::Error for V2AuthoringLoweringErrorV1 {}

/// Lower the implemented decorator-free V2 source forms through one canonical model.
pub fn lower_v2_authoring_v1(
    parsed: &ParsedFile,
    resolutions: V2AuthoringResolutionsV1,
) -> Result<V2AuthoringLoweringV1, V2AuthoringLoweringErrorV1> {
    let components = lower_component_inheritance_v1(parsed, resolutions.components)
        .map_err(V2AuthoringLoweringErrorV1::Component)?;
    let states = lower_state_initializers_v1(parsed, &components.model, resolutions.states)
        .map_err(V2AuthoringLoweringErrorV1::State)?;
    let actions = lower_action_fields_v1(parsed, &components.model, resolutions.actions)
        .map_err(V2AuthoringLoweringErrorV1::Action)?;
    let effects = crate::lower_effect_fields_v1(parsed, &components.model, resolutions.effects)
        .map_err(V2AuthoringLoweringErrorV1::Effect)?;
    let input = compose_authored_semantics_v1([
        components.model.clone(),
        states.model.clone(),
        actions.model.clone(),
        effects.model.clone(),
    ])
    .map_err(V2AuthoringLoweringErrorV1::Composition)?;
    let computed =
        lower_computed_getters_v1(parsed, &input).map_err(V2AuthoringLoweringErrorV1::Computed)?;
    let component_count = components.sites.len();
    let state_count = states.model.declarations.len();
    let action_count = actions.model.declarations.len();
    let computed_count = computed.model.declarations.len();
    let effect_count = effects.model.declarations.len();
    let model = compose_authored_semantics_v1([input, computed.model])
        .map_err(V2AuthoringLoweringErrorV1::Composition)?;
    Ok(V2AuthoringLoweringV1 {
        component_count,
        state_count,
        action_count,
        computed_count,
        effect_count,
        model,
    })
}

#[cfg(test)]
mod tests {
    use presolve_parser::parse_file;

    use crate::{
        build_v2_component_graph_for_module, component_inheritance_sites_v1,
        CanonicalAuthoredDeclarationKindV1, ResolvedIntrinsicIdentityV1,
    };

    use super::{lower_v2_authoring_v1, V2AuthoringResolutionsV1};

    fn identity(name: &str) -> ResolvedIntrinsicIdentityV1 {
        ResolvedIntrinsicIdentityV1 {
            name: name.to_owned(),
            flags: 32,
            declaration_modules: vec!["node_modules/presolve/src/index.d.ts".to_owned()],
        }
    }

    #[test]
    fn assembles_component_state_and_action_without_decorator_recognition() {
        let source = r#"
class Counter extends AliasedBase {
  count = reactiveCell(0);
  increment = activate(() => { this.count += 1; });
  sync = observe(() => { document.title = this.count; return () => { document.title = ""; }; });
  get doubled() { return this.count * 2; }
  get quadrupled() { return this.doubled * 2; }
}
@component() class LegacyOnly { count = reactiveCell(0); }
"#;
        let parsed = parse_file("src/Counter.tsx", source);
        let component = component_inheritance_sites_v1(&parsed).pop().unwrap();
        let state_callee = parsed.classes[0].properties[0]
            .initializer_call
            .as_ref()
            .unwrap()
            .callee_span;
        let action_callee = parsed.classes[0].properties[1]
            .initializer_call
            .as_ref()
            .unwrap()
            .callee_span;
        let effect_callee = parsed.classes[0].properties[2]
            .initializer_call
            .as_ref()
            .unwrap()
            .callee_span;
        let lowering = lower_v2_authoring_v1(
            &parsed,
            V2AuthoringResolutionsV1 {
                components: vec![crate::ResolvedComponentInheritanceV1 {
                    heritage_source: crate::AuthoredSourceRangeV1 {
                        start: component.heritage_source.start,
                        end: component.heritage_source.end,
                        line: component.heritage_source.line,
                        column: component.heritage_source.column,
                    },
                    component_identity: identity("Component"),
                }],
                states: vec![crate::ResolvedStateInitializerV1 {
                    callee_source: crate::AuthoredSourceRangeV1 {
                        start: state_callee.start,
                        end: state_callee.end,
                        line: state_callee.line,
                        column: state_callee.column,
                    },
                    state_identity: identity("state"),
                }],
                actions: vec![crate::ResolvedActionFieldV1 {
                    callee_source: crate::AuthoredSourceRangeV1 {
                        start: action_callee.start,
                        end: action_callee.end,
                        line: action_callee.line,
                        column: action_callee.column,
                    },
                    action_identity: identity("action"),
                }],
                effects: vec![crate::ResolvedEffectFieldV1 {
                    callee_source: crate::AuthoredSourceRangeV1 {
                        start: effect_callee.start,
                        end: effect_callee.end,
                        line: effect_callee.line,
                        column: effect_callee.column,
                    },
                    effect_identity: identity("effect"),
                }],
            },
        )
        .expect("one authority-backed V2 source model");
        assert_eq!(lowering.component_count, 1);
        assert_eq!(lowering.state_count, 1);
        assert_eq!(lowering.action_count, 1);
        assert_eq!(lowering.effect_count, 1);
        assert_eq!(lowering.computed_count, 2);
        assert_eq!(
            lowering
                .model
                .declarations
                .iter()
                .map(|declaration| declaration.kind.clone())
                .collect::<Vec<_>>(),
            vec![
                CanonicalAuthoredDeclarationKindV1::Component,
                CanonicalAuthoredDeclarationKindV1::State,
                CanonicalAuthoredDeclarationKindV1::Action,
                CanonicalAuthoredDeclarationKindV1::Computed,
                CanonicalAuthoredDeclarationKindV1::Computed,
                CanonicalAuthoredDeclarationKindV1::Effect,
            ]
        );
        assert!(lowering
            .model
            .declarations
            .iter()
            .all(|declaration| declaration.subject != "LegacyOnly"));
        let graph = build_v2_component_graph_for_module(&parsed, &lowering.model);
        assert_eq!(graph.components[0].effect_fields.len(), 1);
        assert_eq!(graph.components[0].effect_fields[0].name, "sync");
        assert_eq!(graph.components[0].effect_fields[0].declaration_order, 2);
        assert_eq!(
            graph.components[0].effect_fields[0].body.statements.len(),
            1
        );
        assert!(graph.components[0].effect_fields[0].body.cleanup.is_some());
        let effects = crate::collect_effects(&graph.components, &graph.provenance);
        let effect = effects
            .get(&graph.components[0].id.effect("sync"))
            .expect("canonical V2 effect field is collected");
        assert_eq!(effect.declaration, crate::EffectDeclaration::V2Field);
        let expressions =
            crate::ExpressionGraph::from_components(&graph.components, &graph.provenance);
        let (bodies, _) = crate::lower_effect_bodies(&graph.components, &effects, &expressions);
        assert_eq!(
            bodies
                .get(&graph.components[0].id.effect("sync"))
                .and_then(|body| body.cleanup.as_ref())
                .expect("V2 cleanup body")
                .statements
                .len(),
            1
        );
        assert_eq!(graph.components[0].methods.len(), 2);
        assert!(graph.components[0].methods[0].is_computed());
        assert_eq!(graph.components[0].methods[0].name, "doubled");
        assert!(graph.components[0].methods[1].is_computed());
        assert_eq!(graph.components[0].methods[1].name, "quadrupled");
    }
}
