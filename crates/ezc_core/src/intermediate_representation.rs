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
    pub functions: Vec<IrFunction>,
}

/// Lowers application component ownership into deterministic IR module structure.
#[must_use]
pub fn lower_components_to_ir(model: &ApplicationSemanticModel) -> IntermediateRepresentation {
    let mut modules = std::collections::BTreeMap::<PathBuf, IrModule>::new();
    for component in &model.components {
        let Some(provenance) = model.provenance(&component.id) else {
            continue;
        };
        modules
            .entry(provenance.path.clone())
            .or_insert_with(|| IrModule {
                path: provenance.path.clone(),
                components: Vec::new(),
                functions: Vec::new(),
            })
            .components
            .push(component.id.clone());
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
                functions: vec![function],
            }],
        };

        assert_eq!(ir.modules[0].functions[0].blocks[0].instructions.len(), 1);
    }

    #[test]
    fn lowers_components_into_modules_without_functions() {
        let parsed = ezc_parser::parse_file(
            "src/Counter.tsx",
            "@component(\"x-counter\") class Counter extends Component {}",
        );
        let model = crate::build_application_semantic_model(&parsed);
        let ir = lower_components_to_ir(&model);

        assert_eq!(ir.modules.len(), 1);
        assert_eq!(
            ir.modules[0].components,
            vec![model.components[0].id.clone()]
        );
        assert!(ir.modules[0].functions.is_empty());
    }
}
