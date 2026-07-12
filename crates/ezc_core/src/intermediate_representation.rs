use std::path::PathBuf;

use crate::{SemanticId, SourceProvenance};

/// Compiler-owned intermediate representation, independent of backend output.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct IntermediateRepresentation {
    pub modules: Vec<IrModule>,
}

/// One source module represented in the canonical IR.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IrModule {
    pub path: PathBuf,
    pub functions: Vec<IrFunction>,
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
        IntermediateRepresentation, IrBlock, IrFunction, IrInstruction, IrInstructionKind, IrModule,
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
                functions: vec![function],
            }],
        };

        assert_eq!(ir.modules[0].functions[0].blocks[0].instructions.len(), 1);
    }
}
