use crate::{
    ApplicationSemanticModel, ComponentInstanceId, ComponentInstanceStatus,
    ComponentStructuralRegionId, EffectDeclaration, EffectInstanceId, EffectValidation, SemanticId,
};

pub const RUNTIME_EFFECT_INSTANCE_REGISTRY_VERSION: u32 = 1;

/// Compiler-owned join of a V2 effect declaration with one planned component
/// instance. This is metadata only; capability execution remains unavailable
/// until instance-context programs are emitted.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeEffectInstanceRecord {
    pub id: EffectInstanceId,
    pub effect: SemanticId,
    pub component_instance: ComponentInstanceId,
    pub parent_instance: Option<ComponentInstanceId>,
    pub depth: usize,
    pub declaration_order: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeEffectInstanceRegistry {
    pub version: u32,
    pub records: Vec<RuntimeEffectInstanceRecord>,
}

/// Inactive compiler template for a V2 effect below a structural component
/// region. Runtime activation must issue a distinct occurrence identity.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeEffectStructuralTemplateRecord {
    pub template_instance: ComponentInstanceId,
    pub effect: SemanticId,
    pub parent_instance: Option<ComponentInstanceId>,
    pub structural_region: ComponentStructuralRegionId,
    pub depth: usize,
    pub declaration_order: u32,
}

#[must_use]
pub fn build_runtime_effect_structural_template_registry(
    model: &ApplicationSemanticModel,
) -> Vec<RuntimeEffectStructuralTemplateRecord> {
    let mut records = model
        .component_instance_plan
        .instances
        .values()
        .filter(|instance| instance.status == ComponentInstanceStatus::StructuralTemplate)
        .filter_map(|instance| Some((instance, instance.structural_region.as_ref()?)))
        .flat_map(|(instance, region)| {
            model.effects.values().filter_map(move |effect| {
                (matches!(effect.declaration, EffectDeclaration::V2Field)
                    && effect.owner.entity_id() == Some(&instance.component)
                    && effect.validation == EffectValidation::Valid)
                    .then(|| RuntimeEffectStructuralTemplateRecord {
                        template_instance: instance.id.clone(),
                        effect: effect.id.clone(),
                        parent_instance: instance.parent_instance.clone(),
                        structural_region: region.clone(),
                        depth: instance.depth,
                        declaration_order: effect
                            .declaration_order
                            .expect("V2 effect fields retain declaration order"),
                    })
            })
        })
        .collect::<Vec<_>>();
    records.sort_by(|left, right| {
        (
            &left.structural_region,
            &left.template_instance,
            left.declaration_order,
            &left.effect,
        )
            .cmp(&(
                &right.structural_region,
                &right.template_instance,
                right.declaration_order,
                &right.effect,
            ))
    });
    records
}

/// Project valid V2 field effects onto the canonical planned instance topology.
///
/// This function joins only canonical effect and component-instance facts. It
/// deliberately excludes structural templates and legacy decorator effects.
#[must_use]
pub fn build_runtime_effect_instance_registry(
    model: &ApplicationSemanticModel,
) -> RuntimeEffectInstanceRegistry {
    let mut records = model
        .component_instance_plan
        .instances
        .values()
        .filter(|instance| instance.status == ComponentInstanceStatus::Planned)
        .flat_map(|instance| {
            model.effects.values().filter_map(move |effect| {
                (matches!(effect.declaration, EffectDeclaration::V2Field)
                    && effect.owner.entity_id() == Some(&instance.component)
                    && effect.validation == EffectValidation::Valid)
                    .then(|| RuntimeEffectInstanceRecord {
                        id: EffectInstanceId::for_component_instance(&instance.id, &effect.id),
                        effect: effect.id.clone(),
                        component_instance: instance.id.clone(),
                        parent_instance: instance.parent_instance.clone(),
                        depth: instance.depth,
                        declaration_order: effect
                            .declaration_order
                            .expect("V2 effect fields should retain declaration order"),
                    })
            })
        })
        .collect::<Vec<_>>();
    records.sort_by(|left, right| {
        (
            &left.component_instance,
            left.declaration_order,
            &left.effect,
        )
            .cmp(&(
                &right.component_instance,
                right.declaration_order,
                &right.effect,
            ))
    });
    RuntimeEffectInstanceRegistry {
        version: RUNTIME_EFFECT_INSTANCE_REGISTRY_VERSION,
        records,
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        build_application_semantic_model, build_runtime_effect_instance_registry,
        build_runtime_effect_structural_template_registry, EffectDeclaration,
    };

    #[test]
    fn projects_repeated_nested_component_instances_without_name_inference() {
        let mut model = build_application_semantic_model(&presolve_parser::parse_file(
            "src/EffectInstances.tsx",
            r#"
@component("x-card") class Card extends Component {
  @effect() report() { document.title = "card"; }
  render() { return <article />; }
}
@component("x-page") class Page extends Component {
  render() { return <main><Card /><Card /></main>; }
}
"#,
        ));
        let effect = model.components[0].id.effect("report");
        let record = model.effects.get_mut(&effect).expect("Card effect");
        record.declaration = EffectDeclaration::V2Field;
        record.declaration_order = Some(0);
        let registry = build_runtime_effect_instance_registry(&model);

        assert_eq!(registry.version, 1);
        assert_eq!(registry.records.len(), 2);
        assert!(registry
            .records
            .iter()
            .all(|record| record.effect == effect));
        assert_ne!(registry.records[0].id, registry.records[1].id);
        assert!(registry
            .records
            .iter()
            .all(|record| record.parent_instance.is_some()));
        assert!(registry.records.iter().all(|record| record.depth == 1));
    }

    #[test]
    fn projects_inactive_effect_templates_for_structural_component_occurrences() {
        let mut model = build_application_semantic_model(&presolve_parser::parse_file(
            "src/StructuralEffects.tsx",
            r#"
@component("x-card") class Card extends Component { @effect() report() { document.title = "card"; } render() { return <article />; } }
@component("x-page") class Page extends Component { visible = state(true); render() { return <main>{this.visible ? <Card /> : <span />}</main>; } }
"#,
        ));
        let effect = model.components[0].id.effect("report");
        let record = model.effects.get_mut(&effect).unwrap();
        record.declaration = EffectDeclaration::V2Field;
        record.declaration_order = Some(0);
        let templates = build_runtime_effect_structural_template_registry(&model);
        assert_eq!(templates.len(), 1);
        assert_eq!(templates[0].effect, effect);
        assert!(templates[0].parent_instance.is_some());
    }
}
