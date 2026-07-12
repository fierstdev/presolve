use std::path::PathBuf;

use crate::{ApplicationSemanticModel, SemanticId, SourceProvenance};

/// Compiler-owned intermediate representation, independent of backend output.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct IntermediateRepresentation {
    pub modules: Vec<IrModule>,
}

/// One source module represented in the canonical IR.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IrModule {
    pub path: PathBuf,
    pub components: Vec<SemanticId>,
    pub storage_initializers: Vec<IrInstruction>,
    pub template_entrypoints: Vec<IrTemplateEntrypoint>,
    pub functions: Vec<IrFunction>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IrTemplateEntrypoint {
    pub template: SemanticId,
    pub render_method: SemanticId,
    pub provenance: SourceProvenance,
}

/// Lowers application component ownership into deterministic IR module structure.
#[must_use]
pub fn lower_components_to_ir(model: &ApplicationSemanticModel) -> IntermediateRepresentation {
    let mut modules = std::collections::BTreeMap::<PathBuf, IrModule>::new();
    for component in &model.components {
        let Some(provenance) = model.provenance(&component.id) else {
            continue;
        };
        let module = modules
            .entry(provenance.path.clone())
            .or_insert_with(|| IrModule {
                path: provenance.path.clone(),
                components: Vec::new(),
                storage_initializers: Vec::new(),
                template_entrypoints: Vec::new(),
                functions: Vec::new(),
            });
        module.components.push(component.id.clone());
        module
            .storage_initializers
            .extend(component.state_fields.iter().filter_map(|field| {
                model.provenance(&field.id).map(|provenance| IrInstruction {
                    id: format!("storage:{}", field.id),
                    provenance: provenance.clone(),
                    kind: IrInstructionKind::InitializeStorage {
                        field: field.id.clone(),
                    },
                })
            }));
        if let (Some(template), Some(render)) = (
            model
                .templates
                .iter()
                .find(|template| template.component_name == component.class_name),
            component
                .methods
                .iter()
                .find(|method| method.name == "render"),
        ) {
            module.template_entrypoints.push(IrTemplateEntrypoint {
                template: template.id.clone(),
                render_method: render.id.clone(),
                provenance: template.provenance.clone(),
            });
        }
        module
            .functions
            .extend(component.methods.iter().filter_map(|method| {
                model.provenance(&method.id).map(|provenance| IrFunction {
                    id: method.id.clone(),
                    name: method.name.clone(),
                    provenance: provenance.clone(),
                    blocks: Vec::new(),
                })
            }));
    }
    IntermediateRepresentation {
        modules: modules.into_values().collect(),
    }
}

/// One compiler-owned executable function.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IrFunction {
    pub id: SemanticId,
    pub name: String,
    pub provenance: SourceProvenance,
    pub blocks: Vec<IrBlock>,
}

/// One ordered instruction region in an IR function.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IrBlock {
    pub id: String,
    pub provenance: SourceProvenance,
    pub instructions: Vec<IrInstruction>,
}

/// One backend-neutral instruction with stable source provenance.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IrInstruction {
    pub id: String,
    pub provenance: SourceProvenance,
    pub kind: IrInstructionKind,
}

/// Instruction forms available before lowering and control-flow slices.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IrInstructionKind {
    Nop,
    InitializeStorage { field: SemanticId },
}

#[cfg(test)]
mod tests {
    use super::{
        lower_components_to_ir, IntermediateRepresentation, IrBlock, IrFunction, IrInstruction,
        IrInstructionKind, IrModule,
    };
    use crate::{SemanticId, SourceProvenance};

    #[test]
    fn represents_backend_neutral_ir_structure_with_provenance() {
        let provenance = SourceProvenance::new(
            "src/Counter.tsx",
            ezc_parser::SourceSpan {
                start: 0,
                end: 1,
                line: 1,
                column: 1,
            },
        );
        let function = IrFunction {
            id: SemanticId::component(Some("x-counter"), "Counter").method("increment"),
            name: "increment".to_string(),
            provenance: provenance.clone(),
            blocks: vec![IrBlock {
                id: "entry".to_string(),
                provenance: provenance.clone(),
                instructions: vec![IrInstruction {
                    id: "entry.0".to_string(),
                    provenance,
                    kind: IrInstructionKind::Nop,
                }],
            }],
        };
        let ir = IntermediateRepresentation {
            modules: vec![IrModule {
                path: "src/Counter.tsx".into(),
                components: vec![SemanticId::component(Some("x-counter"), "Counter")],
                storage_initializers: Vec::new(),
                template_entrypoints: Vec::new(),
                functions: vec![function],
            }],
        };

        assert_eq!(ir.modules[0].functions[0].blocks[0].instructions.len(), 1);
    }

    #[test]
    fn lowers_components_into_modules_without_functions() {
        let parsed = ezc_parser::parse_file(
            "src/Counter.tsx",
            "@component(\"x-counter\") class Counter extends Component { count = state(0); increment() {} render() { return <p>{this.count}</p>; } }",
        );
        let model = crate::build_application_semantic_model(&parsed);
        let ir = lower_components_to_ir(&model);

        assert_eq!(ir.modules.len(), 1);
        assert_eq!(
            ir.modules[0].components,
            vec![model.components[0].id.clone()]
        );
        assert_eq!(ir.modules[0].functions[0].name, "increment");
        assert!(ir.modules[0].functions[0].blocks.is_empty());
        assert!(matches!(
            ir.modules[0].storage_initializers[0].kind,
            IrInstructionKind::InitializeStorage { .. }
        ));
        assert_eq!(ir.modules[0].template_entrypoints.len(), 1);
        assert_eq!(
            ir.modules[0].template_entrypoints[0].render_method,
            model.components[0]
                .methods
                .iter()
                .find(|method| method.name == "render")
                .expect("render")
                .id
        );
    }
}
