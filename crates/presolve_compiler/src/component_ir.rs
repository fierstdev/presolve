use crate::{
    ApplicationSemanticModel, ComponentInstanceId, InstanceContextValueSlotId, SemanticId,
    SlotBindingId, SourceProvenance,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ComponentIrOperation {
    CreateComponentInstance {
        instance: ComponentInstanceId,
        component: SemanticId,
        parent: Option<ComponentInstanceId>,
    },
    InitializeComponentInstance {
        instance: ComponentInstanceId,
        context_slots: Vec<InstanceContextValueSlotId>,
    },
    MaterializeComponentTemplate {
        instance: ComponentInstanceId,
        template: SemanticId,
    },
    BindSlotContent {
        binding: SlotBindingId,
        callee_instance: ComponentInstanceId,
        slot: crate::SlotId,
        content_fragment: crate::SlotContentFragmentId,
        content_owner_instance: ComponentInstanceId,
    },
    DestroyComponentInstance {
        instance: ComponentInstanceId,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComponentIrInstruction {
    pub index: usize,
    pub operation: ComponentIrOperation,
    pub provenance: SourceProvenance,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ComponentIrReport {
    pub instructions: Vec<ComponentIrInstruction>,
}

/// Lower only executable H10 initialization/binding plans into observable IR.
#[must_use]
pub fn lower_component_ir(model: &ApplicationSemanticModel) -> ComponentIrReport {
    let mut instructions = Vec::new();
    let mut push = |operation, provenance: SourceProvenance| {
        let index = instructions.len();
        instructions.push(ComponentIrInstruction {
            index,
            operation,
            provenance,
        });
    };
    for batch in &model.component_initialization.instance_batches {
        for instance_id in &batch.instances {
            let Some(instance) = model.component_instance_plan.instances.get(instance_id) else {
                continue;
            };
            push(
                ComponentIrOperation::CreateComponentInstance {
                    instance: instance.id.clone(),
                    component: instance.component.clone(),
                    parent: instance.parent_instance.clone(),
                },
                instance.provenance.clone(),
            );
            let context_slots = model
                .instance_context
                .resolutions
                .values()
                .filter(|resolution| resolution.consumer_instance.component_instance == instance.id)
                .filter(|resolution| {
                    model
                        .composition_types
                        .instance_context_bindings
                        .get(&resolution.consumer_instance)
                        .is_some_and(|record| {
                            record.overall == crate::CompositionCompatibility::Compatible
                        })
                })
                .filter_map(|resolution| resolution.value_slot.clone())
                .collect();
            push(
                ComponentIrOperation::InitializeComponentInstance {
                    instance: instance.id.clone(),
                    context_slots,
                },
                instance.provenance.clone(),
            );
            push(
                ComponentIrOperation::MaterializeComponentTemplate {
                    instance: instance.id.clone(),
                    template: instance.component.template(),
                },
                instance.provenance.clone(),
            );
        }
    }
    for batch in &model.component_initialization.slot_binding_batches {
        for binding_id in &batch.bindings {
            let Some(binding) = model.slot_bindings.binding(binding_id) else {
                continue;
            };
            let (Some(slot), Some(content_fragment)) =
                (binding.slot.clone(), binding.content_fragment.clone())
            else {
                continue;
            };
            push(
                ComponentIrOperation::BindSlotContent {
                    binding: binding.id.clone(),
                    callee_instance: binding.callee_instance.clone(),
                    slot,
                    content_fragment,
                    content_owner_instance: binding.content_owner_instance.clone(),
                },
                binding.provenance.clone(),
            );
        }
    }
    ComponentIrReport { instructions }
}

#[must_use]
pub fn validate_component_ir(
    model: &ApplicationSemanticModel,
    report: &ComponentIrReport,
) -> Vec<String> {
    let expected = lower_component_ir(model);
    (report != &expected)
        .then(|| "component IR does not match the canonical H10 plan".to_string())
        .into_iter()
        .collect()
}

#[cfg(test)]
mod tests {
    use crate::{
        build_application_semantic_model, lower_component_ir, validate_component_ir,
        ComponentIrOperation,
    };

    #[test]
    fn lowers_ordered_instance_context_and_slot_operations() {
        let asm = build_application_semantic_model(&presolve_parser::parse_file(
            "src/Ir.tsx",
            r#"
@component("x-card") class Card extends Component { @slot() children!: SlotContent; render() { return <article><slot /></article>; } }
@component("x-page") class Page extends Component { render() { return <Card><p /></Card>; } }
"#,
        ));
        let report = lower_component_ir(&asm);
        assert_eq!(report.instructions.len(), 7);
        assert!(matches!(
            report.instructions[0].operation,
            ComponentIrOperation::CreateComponentInstance { .. }
        ));
        assert!(matches!(
            report.instructions[6].operation,
            ComponentIrOperation::BindSlotContent { .. }
        ));
        assert!(validate_component_ir(&asm, &report).is_empty());
    }

    #[test]
    fn asm_validation_rejects_mutated_component_ir() {
        let mut asm = build_application_semantic_model(&presolve_parser::parse_file(
            "src/IrValidate.tsx",
            r#"
@component("x-page") class Page extends Component { render() { return <main />; } }
"#,
        ));
        asm.component_ir.instructions.clear();
        assert!(crate::validate_application_semantic_model(&asm)
            .iter()
            .any(|diagnostic| diagnostic.code == "PSASM1199"));
    }
}
