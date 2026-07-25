//! Immutable control-flow projections over canonical compiler IR.
//!
//! This product exposes only facts represented by `IntermediateRepresentation`.
//! Its coverage markers prevent later analyses from mistaking absent exception,
//! suspension, or unknown-call IR for a proven empty set.

use std::collections::BTreeSet;

use serde::Serialize;

use crate::intermediate_representation::ir_instruction_operands;
use crate::{
    IntermediateRepresentation, IrBranchArm, IrInstructionKind, IrOperand, SourceProvenance,
};

pub const CONTROL_FLOW_SCHEMA_VERSION: u32 = 1;

/// A deterministic, Presolve-owned per-function control-flow product.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ControlFlowGraphV1 {
    pub schema_version: u32,
    pub functions: Vec<ControlFlowFunctionV1>,
}

/// Control-flow and data-access facts for one canonical IR function.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ControlFlowFunctionV1 {
    pub module_path: String,
    pub id: String,
    pub name: String,
    pub provenance: ControlFlowProvenanceV1,
    pub entry_block: String,
    pub blocks: Vec<ControlFlowBlockV1>,
    pub branch_edges: Vec<ControlFlowBranchEdgeV1>,
    pub loops: Vec<ControlFlowLoopV1>,
    pub coverage: ControlFlowCoverageV1,
}

/// A basic block with exact IR-visible reads and writes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ControlFlowBlockV1 {
    pub id: String,
    pub provenance: ControlFlowProvenanceV1,
    pub reads: Vec<ControlFlowAccessV1>,
    pub writes: Vec<ControlFlowAccessV1>,
    pub observable_instructions: Vec<String>,
}

/// One directed conditional edge in canonical IR.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ControlFlowBranchEdgeV1 {
    pub from: String,
    pub to: String,
    pub arm: ControlFlowBranchArmV1,
    pub provenance: ControlFlowProvenanceV1,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ControlFlowBranchArmV1 {
    True,
    False,
}

/// Natural-loop facts already retained by canonical IR.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ControlFlowLoopV1 {
    pub id: String,
    pub header: String,
    pub body: Vec<String>,
    pub latches: Vec<String>,
    pub exits: Vec<String>,
    pub provenance: ControlFlowProvenanceV1,
}

/// One IR-owned data access; `kind` prevents values and semantic slots sharing
/// a textual ID from being conflated by later analyses.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub struct ControlFlowAccessV1 {
    pub kind: ControlFlowAccessKindV1,
    pub id: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ControlFlowAccessKindV1 {
    Value,
    Storage,
    ContextSlot,
    Computed,
    Resource,
}

/// Whether a required analysis dimension is encoded by the current IR.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ControlFlowCoverageStatusV1 {
    Available,
    Unavailable,
}

/// Explicit fail-closed coverage for a function control-flow projection.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ControlFlowCoverageV1 {
    pub branch_topology: ControlFlowCoverageStatusV1,
    pub definite_dataflow: ControlFlowCoverageStatusV1,
    pub natural_loops: ControlFlowCoverageStatusV1,
    pub exception_paths: ControlFlowCoverageStatusV1,
    pub async_suspension: ControlFlowCoverageStatusV1,
    pub unknown_calls: ControlFlowCoverageStatusV1,
    pub capture_escape: ControlFlowCoverageStatusV1,
    pub resource_cancellation: ControlFlowCoverageStatusV1,
}

/// Serializable source provenance retained through the CFG projection.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ControlFlowProvenanceV1 {
    pub path: String,
    pub start: usize,
    pub end: usize,
    pub line: usize,
    pub column: usize,
}

/// Projects all canonical IR functions into stable control-flow records.
#[must_use]
pub fn build_control_flow_graph_v1(ir: &IntermediateRepresentation) -> ControlFlowGraphV1 {
    let mut functions = ir
        .modules
        .iter()
        .flat_map(|module| {
            module.functions.iter().map(move |function| {
                project_function(module.path.to_string_lossy().as_ref(), function)
            })
        })
        .collect::<Vec<_>>();
    functions.sort_by(|left, right| {
        left.module_path
            .cmp(&right.module_path)
            .then_with(|| left.id.cmp(&right.id))
    });
    ControlFlowGraphV1 {
        schema_version: CONTROL_FLOW_SCHEMA_VERSION,
        functions,
    }
}

fn project_function(module_path: &str, function: &crate::IrFunction) -> ControlFlowFunctionV1 {
    let mut blocks = function
        .blocks
        .iter()
        .map(|block| {
            let mut reads = BTreeSet::new();
            let mut writes = BTreeSet::new();
            let mut observable_instructions = Vec::new();
            for instruction in &block.instructions {
                collect_instruction_accesses(&instruction.kind, &mut reads, &mut writes);
                for operand in ir_instruction_operands(&instruction.kind) {
                    match operand {
                        IrOperand::Value(value) => {
                            reads.insert(access(ControlFlowAccessKindV1::Value, value.to_string()));
                        }
                        IrOperand::Storage(storage) => {
                            reads.insert(access(
                                ControlFlowAccessKindV1::Storage,
                                storage.to_string(),
                            ));
                        }
                        IrOperand::Constant(_) => {}
                    }
                }
                if let Some(result) = &instruction.result {
                    writes.insert(access(ControlFlowAccessKindV1::Value, result.to_string()));
                }
                if instruction.kind.is_observable_side_effect() {
                    observable_instructions.push(instruction.id.to_string());
                }
            }
            ControlFlowBlockV1 {
                id: block.id.to_string(),
                provenance: provenance(&block.provenance),
                reads: reads.into_iter().collect(),
                writes: writes.into_iter().collect(),
                observable_instructions,
            }
        })
        .collect::<Vec<_>>();
    blocks.sort_by(|left, right| left.id.cmp(&right.id));

    let mut branch_edges = function
        .branch_edges
        .iter()
        .map(|edge| ControlFlowBranchEdgeV1 {
            from: edge.from.to_string(),
            to: edge.to.to_string(),
            arm: match edge.arm {
                IrBranchArm::True => ControlFlowBranchArmV1::True,
                IrBranchArm::False => ControlFlowBranchArmV1::False,
            },
            provenance: provenance(&edge.provenance),
        })
        .collect::<Vec<_>>();
    branch_edges.sort_by(|left, right| {
        left.from
            .cmp(&right.from)
            .then_with(|| left.to.cmp(&right.to))
            .then_with(|| left.arm.cmp(&right.arm))
    });

    let mut loops = function
        .loops
        .iter()
        .map(|loop_| ControlFlowLoopV1 {
            id: loop_.id.to_string(),
            header: loop_.header.to_string(),
            body: sorted_ids(&loop_.body),
            latches: sorted_ids(&loop_.latches),
            exits: sorted_ids(&loop_.exits),
            provenance: provenance(&loop_.provenance),
        })
        .collect::<Vec<_>>();
    loops.sort_by(|left, right| left.id.cmp(&right.id));

    ControlFlowFunctionV1 {
        module_path: module_path.to_owned(),
        id: function.id.to_string(),
        name: function.name.clone(),
        provenance: provenance(&function.provenance),
        entry_block: function.entry_block.to_string(),
        blocks,
        branch_edges,
        loops,
        coverage: ControlFlowCoverageV1 {
            branch_topology: ControlFlowCoverageStatusV1::Available,
            definite_dataflow: ControlFlowCoverageStatusV1::Available,
            natural_loops: ControlFlowCoverageStatusV1::Available,
            exception_paths: ControlFlowCoverageStatusV1::Unavailable,
            async_suspension: ControlFlowCoverageStatusV1::Unavailable,
            unknown_calls: ControlFlowCoverageStatusV1::Unavailable,
            capture_escape: ControlFlowCoverageStatusV1::Unavailable,
            resource_cancellation: ControlFlowCoverageStatusV1::Unavailable,
        },
    }
}

fn collect_instruction_accesses(
    kind: &IrInstructionKind,
    reads: &mut BTreeSet<ControlFlowAccessV1>,
    writes: &mut BTreeSet<ControlFlowAccessV1>,
) {
    match kind {
        IrInstructionKind::InitializeStorage { storage } => {
            writes.insert(access(
                ControlFlowAccessKindV1::Storage,
                storage.to_string(),
            ));
        }
        IrInstructionKind::InitializeContextSlot { slot, .. } => {
            writes.insert(access(
                ControlFlowAccessKindV1::ContextSlot,
                slot.to_string(),
            ));
        }
        IrInstructionKind::LoadStorage { storage } => {
            reads.insert(access(
                ControlFlowAccessKindV1::Storage,
                storage.to_string(),
            ));
        }
        IrInstructionKind::LoadContextSlot { slot } => {
            reads.insert(access(
                ControlFlowAccessKindV1::ContextSlot,
                slot.to_string(),
            ));
        }
        IrInstructionKind::LoadComputed { computed } => {
            reads.insert(access(
                ControlFlowAccessKindV1::Computed,
                computed.to_string(),
            ));
        }
        IrInstructionKind::LoadResource { declaration } => {
            reads.insert(access(
                ControlFlowAccessKindV1::Resource,
                declaration.to_string(),
            ));
        }
        IrInstructionKind::StoreStorage { storage, .. } => {
            writes.insert(access(
                ControlFlowAccessKindV1::Storage,
                storage.to_string(),
            ));
        }
        IrInstructionKind::Nop
        | IrInstructionKind::Constant { .. }
        | IrInstructionKind::Copy { .. }
        | IrInstructionKind::GetMember { .. }
        | IrInstructionKind::GetIndex { .. }
        | IrInstructionKind::Select { .. }
        | IrInstructionKind::Template { .. }
        | IrInstructionKind::PurePackageCall { .. }
        | IrInstructionKind::CapabilityCall { .. }
        | IrInstructionKind::CapabilityAssign { .. }
        | IrInstructionKind::Binary { .. }
        | IrInstructionKind::Unary { .. } => {}
    }
}

fn access(kind: ControlFlowAccessKindV1, id: String) -> ControlFlowAccessV1 {
    ControlFlowAccessV1 { kind, id }
}

fn sorted_ids(values: &[crate::IrBlockId]) -> Vec<String> {
    let mut values = values.iter().map(ToString::to_string).collect::<Vec<_>>();
    values.sort();
    values.dedup();
    values
}

fn provenance(value: &SourceProvenance) -> ControlFlowProvenanceV1 {
    ControlFlowProvenanceV1 {
        path: value.path.to_string_lossy().into_owned(),
        start: value.span.start,
        end: value.span.end,
        line: value.span.line,
        column: value.span.column,
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use crate::{
        ContextIrReport, IntermediateRepresentation, IrBlock, IrBlockId, IrBranchArm, IrBranchEdge,
        IrConstant, IrFunction, IrInstruction, IrInstructionId, IrInstructionKind, IrModule,
        IrOperand, IrStorageId, IrValueId, SemanticId, SourceProvenance,
    };

    use super::{
        build_control_flow_graph_v1, ControlFlowAccessKindV1, ControlFlowCoverageStatusV1,
        CONTROL_FLOW_SCHEMA_VERSION,
    };

    #[test]
    fn projects_branch_and_definite_dataflow_from_canonical_ir() {
        let function = SemanticId::component(Some("x-counter"), "Counter").method("increment");
        let entry = IrBlockId::entry_for(&function);
        let when_true = IrBlockId::for_function(&function, "when-true");
        let when_false = IrBlockId::for_function(&function, "when-false");
        let storage = IrStorageId::for_semantic_origin(
            &SemanticId::component(Some("x-counter"), "Counter").state_field("count"),
        );
        let source = SourceProvenance::new(
            "src/Counter.tsx",
            presolve_parser::SourceSpan {
                start: 4,
                end: 9,
                line: 1,
                column: 5,
            },
        );
        let representation = IntermediateRepresentation {
            modules: vec![IrModule {
                path: "src/Counter.tsx".into(),
                components: Vec::new(),
                storages: Vec::new(),
                storage_initializers: Vec::new(),
                template_entrypoints: Vec::new(),
                functions: vec![IrFunction {
                    id: function.clone(),
                    name: "increment".to_owned(),
                    provenance: source.clone(),
                    entry_block: entry.clone(),
                    blocks: vec![
                        IrBlock {
                            id: entry.clone(),
                            provenance: source.clone(),
                            instructions: vec![IrInstruction {
                                id: IrInstructionId::for_block(&entry, 0),
                                provenance: source.clone(),
                                result: Some(IrValueId::for_function(&function, 0)),
                                semantic_origin: None,
                                kind: IrInstructionKind::LoadStorage {
                                    storage: storage.clone(),
                                },
                            }],
                        },
                        IrBlock {
                            id: when_true.clone(),
                            provenance: source.clone(),
                            instructions: vec![IrInstruction {
                                id: IrInstructionId::for_block(&when_true, 0),
                                provenance: source.clone(),
                                result: None,
                                semantic_origin: None,
                                kind: IrInstructionKind::StoreStorage {
                                    storage: storage.clone(),
                                    value: IrOperand::Value(IrValueId::for_function(&function, 0)),
                                },
                            }],
                        },
                        IrBlock {
                            id: when_false.clone(),
                            provenance: source.clone(),
                            instructions: vec![IrInstruction {
                                id: IrInstructionId::for_block(&when_false, 0),
                                provenance: source.clone(),
                                result: Some(IrValueId::for_function(&function, 1)),
                                semantic_origin: None,
                                kind: IrInstructionKind::Constant {
                                    value: IrConstant::Number("0".into()),
                                },
                            }],
                        },
                    ],
                    branch_edges: vec![
                        IrBranchEdge {
                            from: entry.clone(),
                            to: when_true,
                            arm: IrBranchArm::True,
                            provenance: source.clone(),
                        },
                        IrBranchEdge {
                            from: entry,
                            to: when_false,
                            arm: IrBranchArm::False,
                            provenance: source.clone(),
                        },
                    ],
                    values: BTreeMap::new(),
                    loops: Vec::new(),
                }],
                computed_evaluations: Vec::new(),
                effect_executions: Vec::new(),
            }],
            context_ir: ContextIrReport::default(),
        };

        let graph = build_control_flow_graph_v1(&representation);
        assert_eq!(graph.schema_version, CONTROL_FLOW_SCHEMA_VERSION);
        let function = &graph.functions[0];
        assert_eq!(function.branch_edges.len(), 2);
        assert_eq!(
            function.coverage.definite_dataflow,
            ControlFlowCoverageStatusV1::Available
        );
        assert_eq!(
            function.coverage.exception_paths,
            ControlFlowCoverageStatusV1::Unavailable
        );
        let entry = function
            .blocks
            .iter()
            .find(|block| block.id.ends_with("block:entry"))
            .unwrap();
        assert!(entry
            .reads
            .iter()
            .any(|access| access.kind == ControlFlowAccessKindV1::Storage));
        assert!(entry
            .writes
            .iter()
            .any(|access| access.kind == ControlFlowAccessKindV1::Value));
        let when_true = function
            .blocks
            .iter()
            .find(|block| block.id.ends_with("block:when-true"))
            .unwrap();
        assert!(when_true
            .reads
            .iter()
            .any(|access| access.kind == ControlFlowAccessKindV1::Value));
        assert!(when_true
            .writes
            .iter()
            .any(|access| access.kind == ControlFlowAccessKindV1::Storage));
    }
}
