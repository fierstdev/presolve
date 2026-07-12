use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use crate::{ApplicationSemanticModel, SemanticId, SemanticType, SourceProvenance};

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
    pub storages: Vec<IrStorage>,
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
                storages: Vec::new(),
                storage_initializers: Vec::new(),
                template_entrypoints: Vec::new(),
                functions: Vec::new(),
            });
        module.components.push(component.id.clone());
        module
            .storages
            .extend(component.state_fields.iter().filter_map(|field| {
                model.provenance(&field.id).map(|provenance| IrStorage {
                    id: IrStorageId::for_semantic_origin(&field.id),
                    semantic_origin: field.id.clone(),
                    value_type: model
                        .semantic_types
                        .assignments
                        .get(&field.id)
                        .map_or(SemanticType::Unknown, |assignment| {
                            assignment.semantic_type.clone()
                        }),
                    initial_value: field.initial_value.clone(),
                    provenance: provenance.clone(),
                })
            }));
        let storage_offset = module.storage_initializers.len();
        module
            .storage_initializers
            .extend(
                component
                    .state_fields
                    .iter()
                    .enumerate()
                    .filter_map(|(index, field)| {
                        model.provenance(&field.id).map(|provenance| IrInstruction {
                            id: IrInstructionId::for_module(&module.path, storage_offset + index),
                            provenance: provenance.clone(),
                            result: None,
                            semantic_origin: Some(field.id.clone()),
                            kind: IrInstructionKind::InitializeStorage {
                                storage: IrStorageId::for_semantic_origin(&field.id),
                            },
                        })
                    }),
            );
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
                    entry_block: IrBlockId::entry_for(&method.id),
                    blocks: vec![IrBlock {
                        id: IrBlockId::entry_for(&method.id),
                        provenance: provenance.clone(),
                        instructions: Vec::new(),
                    }],
                    branch_edges: Vec::new(),
                    values: BTreeMap::new(),
                    loops: Vec::new(),
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
    pub entry_block: IrBlockId,
    pub blocks: Vec<IrBlock>,
    pub branch_edges: Vec<IrBranchEdge>,
    pub values: BTreeMap<IrValueId, IrValue>,
    pub loops: Vec<IrLoop>,
}

impl IrFunction {
    #[must_use]
    pub fn block(&self, id: &IrBlockId) -> Option<&IrBlock> {
        self.blocks.iter().find(|block| block.id == *id)
    }

    #[must_use]
    pub fn successor_blocks(&self, id: &IrBlockId) -> Vec<IrBlockId> {
        self.branch_edges
            .iter()
            .filter(|edge| edge.from == *id)
            .map(|edge| edge.to.clone())
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect()
    }

    #[must_use]
    pub fn predecessor_blocks(&self, id: &IrBlockId) -> Vec<IrBlockId> {
        self.branch_edges
            .iter()
            .filter(|edge| edge.to == *id)
            .map(|edge| edge.from.clone())
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect()
    }

    #[must_use]
    pub fn is_exit_block(&self, id: &IrBlockId) -> bool {
        self.block(id).is_some() && self.successor_blocks(id).is_empty()
    }

    #[must_use]
    pub fn value(&self, id: &IrValueId) -> Option<&IrValue> {
        self.values.get(id)
    }
}

/// A stable compiler-owned basic-block identity within an IR function.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct IrBlockId(String);

impl IrBlockId {
    #[must_use]
    pub fn entry_for(function: &SemanticId) -> Self {
        Self::for_function(function, "entry")
    }

    #[must_use]
    pub fn for_function(function: &SemanticId, name: &str) -> Self {
        Self(format!("{function}/block:{name}"))
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for IrBlockId {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.0)
    }
}

/// A stable compiler-owned operation identity within an IR block.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct IrInstructionId(String);

impl IrInstructionId {
    #[must_use]
    pub fn for_block(block: &IrBlockId, index: usize) -> Self {
        Self(format!("{block}/instruction:{index}"))
    }

    #[must_use]
    pub fn for_module(path: &Path, index: usize) -> Self {
        Self(format!("module:{}/instruction:{index}", path.display()))
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for IrInstructionId {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.0)
    }
}

/// A stable compiler-owned transient value identity within an IR function.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct IrValueId(String);

impl IrValueId {
    #[must_use]
    pub fn for_function(function: &SemanticId, index: usize) -> Self {
        Self(format!("{function}/value:{index}"))
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for IrValueId {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.0)
    }
}

/// A stable compiler-owned storage-slot identity.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct IrStorageId(String);

impl IrStorageId {
    #[must_use]
    pub fn for_semantic_origin(origin: &SemanticId) -> Self {
        Self(format!("storage:{origin}"))
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// One mutable runtime storage slot lowered from an authored semantic entity.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IrStorage {
    pub id: IrStorageId,
    pub semantic_origin: SemanticId,
    pub value_type: SemanticType,
    pub initial_value: Option<crate::SerializableValue>,
    pub provenance: SourceProvenance,
}

impl std::fmt::Display for IrStorageId {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.0)
    }
}

/// An immutable primitive constant embedded directly in an IR operand.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IrConstant {
    Null,
    Boolean(bool),
    Number(String),
    String(String),
}

/// A closed set of executable inputs supported by the canonical IR.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IrOperand {
    Value(IrValueId),
    Constant(IrConstant),
    Storage(IrStorageId),
}

/// The canonical origin of an IR value.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IrValueDefinition {
    Instruction(IrInstructionId),
    Parameter { function: SemanticId, index: usize },
    BlockParameter { block: IrBlockId, index: usize },
}

/// One function-scoped transient value and its canonical definition metadata.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IrValue {
    pub id: IrValueId,
    pub definition: IrValueDefinition,
    pub semantic_type: SemanticType,
    pub provenance: SourceProvenance,
    pub semantic_origin: Option<SemanticId>,
}

/// A structural integrity failure in canonical IR value and operand metadata.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IrValidationDiagnostic {
    pub code: &'static str,
    pub message: String,
}

/// One exact value-consuming operand position in a canonical instruction.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IrUse {
    pub instruction: IrInstructionId,
    pub operand_index: usize,
}

/// Canonical definition and use relations for one IR function.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IrDefinitionUseAnalysis {
    pub definitions: BTreeMap<IrValueId, IrValueDefinition>,
    pub uses: BTreeMap<IrValueId, Vec<IrUse>>,
}

/// A resolved use-to-definition relation for one value-consuming operand.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IrUseDefinition {
    pub value: IrValueId,
    pub instruction: IrInstructionId,
    pub operand_index: usize,
    pub definition: IrValueDefinition,
}

/// Block-level live-in and live-out sets for one canonical IR function.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IrLivenessAnalysis {
    pub live_in: BTreeMap<IrBlockId, Vec<IrValueId>>,
    pub live_out: BTreeMap<IrBlockId, Vec<IrValueId>>,
}

/// The entry-reachable and unreachable block partition for one IR function.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IrReachabilityAnalysis {
    pub reachable: Vec<IrBlockId>,
    pub unreachable: Vec<IrBlockId>,
}

/// Computes canonical reachability from a function's entry block.
#[must_use]
pub fn analyze_reachability(function: &IrFunction) -> IrReachabilityAnalysis {
    let mut reachable = BTreeSet::from([function.entry_block.clone()]);
    let mut pending = vec![function.entry_block.clone()];
    while let Some(block) = pending.pop() {
        for successor in function.successor_blocks(&block) {
            if reachable.insert(successor.clone()) {
                pending.push(successor);
            }
        }
    }
    let all_blocks = function
        .blocks
        .iter()
        .map(|block| block.id.clone())
        .collect::<BTreeSet<_>>();
    IrReachabilityAnalysis {
        reachable: reachable
            .iter()
            .filter(|block| all_blocks.contains(*block))
            .cloned()
            .collect(),
        unreachable: all_blocks.difference(&reachable).cloned().collect(),
    }
}

/// Computes immutable block liveness from value uses, definitions, and CFG successors.
#[must_use]
pub fn analyze_liveness(function: &IrFunction) -> IrLivenessAnalysis {
    let block_ids = function
        .blocks
        .iter()
        .map(|block| block.id.clone())
        .collect::<BTreeSet<_>>();
    let mut block_uses = BTreeMap::new();
    let mut block_definitions = BTreeMap::new();
    for block in &function.blocks {
        let mut uses = BTreeSet::new();
        let mut definitions = BTreeSet::new();
        for instruction in &block.instructions {
            for operand in instruction_operands(&instruction.kind) {
                if let IrOperand::Value(value) = operand {
                    if !definitions.contains(value) {
                        uses.insert(value.clone());
                    }
                }
            }
            if let Some(result) = &instruction.result {
                definitions.insert(result.clone());
            }
        }
        block_uses.insert(block.id.clone(), uses);
        block_definitions.insert(block.id.clone(), definitions);
    }
    let mut live_in = block_ids
        .iter()
        .cloned()
        .map(|id| (id, BTreeSet::new()))
        .collect::<BTreeMap<_, _>>();
    let mut live_out = live_in.clone();
    let mut changed = true;
    while changed {
        changed = false;
        for block in block_ids.iter().rev() {
            let next_out = function
                .successor_blocks(block)
                .into_iter()
                .flat_map(|successor| live_in[&successor].clone())
                .collect::<BTreeSet<_>>();
            let mut next_in = block_uses[block].clone();
            next_in.extend(next_out.difference(&block_definitions[block]).cloned());
            if live_out.get(block) != Some(&next_out) || live_in.get(block) != Some(&next_in) {
                live_out.insert(block.clone(), next_out);
                live_in.insert(block.clone(), next_in);
                changed = true;
            }
        }
    }
    IrLivenessAnalysis {
        live_in: live_in
            .into_iter()
            .map(|(block, values)| (block, values.into_iter().collect()))
            .collect(),
        live_out: live_out
            .into_iter()
            .map(|(block, values)| (block, values.into_iter().collect()))
            .collect(),
    }
}

/// Resolves each canonical value use to its registered definition.
#[must_use]
pub fn analyze_use_definitions(function: &IrFunction) -> Vec<IrUseDefinition> {
    let mut relations = Vec::new();
    for block in &function.blocks {
        for instruction in &block.instructions {
            for (operand_index, operand) in instruction_operands(&instruction.kind)
                .into_iter()
                .enumerate()
            {
                let IrOperand::Value(value) = operand else {
                    continue;
                };
                let Some(definition) = function
                    .values
                    .get(value)
                    .map(|value| value.definition.clone())
                else {
                    continue;
                };
                relations.push(IrUseDefinition {
                    value: value.clone(),
                    instruction: instruction.id.clone(),
                    operand_index,
                    definition,
                });
            }
        }
    }
    relations
}

/// Computes definition and use chains from one function's value registry and instruction operands.
#[must_use]
pub fn analyze_definition_uses(function: &IrFunction) -> IrDefinitionUseAnalysis {
    let definitions = function
        .values
        .iter()
        .map(|(id, value)| (id.clone(), value.definition.clone()))
        .collect::<BTreeMap<_, _>>();
    let mut uses = function
        .values
        .keys()
        .cloned()
        .map(|id| (id, Vec::new()))
        .collect::<BTreeMap<_, _>>();
    for block in &function.blocks {
        for instruction in &block.instructions {
            for (operand_index, operand) in instruction_operands(&instruction.kind)
                .into_iter()
                .enumerate()
            {
                if let IrOperand::Value(value) = operand {
                    uses.entry(value.clone()).or_default().push(IrUse {
                        instruction: instruction.id.clone(),
                        operand_index,
                    });
                }
            }
        }
    }
    IrDefinitionUseAnalysis { definitions, uses }
}

/// Validates identity, definition, operand, and storage-reference integrity for canonical IR.
#[must_use]
pub fn validate_intermediate_representation(
    representation: &IntermediateRepresentation,
) -> Vec<IrValidationDiagnostic> {
    let storage_ids = representation
        .modules
        .iter()
        .flat_map(|module| module.storages.iter().map(|storage| storage.id.clone()))
        .collect::<BTreeSet<_>>();
    let mut diagnostics = Vec::new();
    let mut instruction_ids = BTreeSet::new();

    for module in &representation.modules {
        for instruction in &module.storage_initializers {
            validate_instruction(
                instruction,
                None,
                &BTreeSet::new(),
                &storage_ids,
                &mut instruction_ids,
                &mut diagnostics,
            );
        }
        for function in &module.functions {
            let function_instruction_ids = function
                .blocks
                .iter()
                .flat_map(|block| {
                    block
                        .instructions
                        .iter()
                        .map(|instruction| instruction.id.clone())
                })
                .collect::<BTreeSet<_>>();
            for block in &function.blocks {
                for instruction in &block.instructions {
                    validate_instruction(
                        instruction,
                        Some(function),
                        &function_instruction_ids,
                        &storage_ids,
                        &mut instruction_ids,
                        &mut diagnostics,
                    );
                }
            }
            for (id, value) in &function.values {
                if id != &value.id {
                    diagnostics.push(IrValidationDiagnostic {
                        code: "EZIR1001",
                        message: format!(
                            "value registry key {id} does not match value ID {}",
                            value.id
                        ),
                    });
                }
                match &value.definition {
                    IrValueDefinition::Instruction(instruction) => {
                        if !function_instruction_ids.contains(instruction) {
                            diagnostics.push(IrValidationDiagnostic {
                                code: "EZIR1002",
                                message: format!("value {id} references missing defining instruction {instruction}"),
                            });
                        }
                    }
                    IrValueDefinition::Parameter {
                        function: owner, ..
                    } if owner != &function.id => {
                        diagnostics.push(IrValidationDiagnostic {
                            code: "EZIR1003",
                            message: format!(
                                "value {id} belongs to parameter function {owner}, not {}",
                                function.id
                            ),
                        });
                    }
                    IrValueDefinition::BlockParameter { block, .. }
                        if function.block(block).is_none() =>
                    {
                        diagnostics.push(IrValidationDiagnostic {
                            code: "EZIR1004",
                            message: format!(
                                "value {id} references missing block parameter owner {block}"
                            ),
                        });
                    }
                    _ => {}
                }
            }
        }
    }
    diagnostics
}

fn validate_instruction(
    instruction: &IrInstruction,
    function: Option<&IrFunction>,
    function_instruction_ids: &BTreeSet<IrInstructionId>,
    storage_ids: &BTreeSet<IrStorageId>,
    instruction_ids: &mut BTreeSet<IrInstructionId>,
    diagnostics: &mut Vec<IrValidationDiagnostic>,
) {
    if !instruction_ids.insert(instruction.id.clone()) {
        diagnostics.push(IrValidationDiagnostic {
            code: "EZIR1005",
            message: format!("duplicate instruction ID {}", instruction.id),
        });
    }
    if let (Some(function), Some(result)) = (function, &instruction.result) {
        match function.values.get(result) {
            Some(value)
                if value.definition == IrValueDefinition::Instruction(instruction.id.clone()) => {}
            _ => diagnostics.push(IrValidationDiagnostic {
                code: "EZIR1006",
                message: format!(
                    "instruction {} result {result} lacks a matching value definition",
                    instruction.id
                ),
            }),
        }
    }
    for operand in instruction_operands(&instruction.kind) {
        if let IrOperand::Value(value) = operand {
            if function.is_none_or(|function| !function.values.contains_key(value)) {
                diagnostics.push(IrValidationDiagnostic {
                    code: "EZIR1007",
                    message: format!(
                        "instruction {} references unknown value {value}",
                        instruction.id
                    ),
                });
            }
        }
    }
    for storage in instruction_storages(&instruction.kind) {
        if !storage_ids.contains(storage) {
            diagnostics.push(IrValidationDiagnostic {
                code: "EZIR1008",
                message: format!(
                    "instruction {} references unknown storage {storage}",
                    instruction.id
                ),
            });
        }
    }
    if let Some(result) = &instruction.result {
        if !function_instruction_ids.contains(&instruction.id) {
            diagnostics.push(IrValidationDiagnostic {
                code: "EZIR1009",
                message: format!(
                    "module instruction {} must not produce value {result}",
                    instruction.id
                ),
            });
        }
    }
}

fn instruction_operands(kind: &IrInstructionKind) -> Vec<&IrOperand> {
    match kind {
        IrInstructionKind::StoreStorage { value, .. }
        | IrInstructionKind::Unary { operand: value, .. } => vec![value],
        IrInstructionKind::Binary { left, right, .. } => vec![left, right],
        IrInstructionKind::Nop
        | IrInstructionKind::InitializeStorage { .. }
        | IrInstructionKind::LoadStorage { .. } => Vec::new(),
    }
}

fn instruction_storages(kind: &IrInstructionKind) -> Vec<&IrStorageId> {
    match kind {
        IrInstructionKind::InitializeStorage { storage }
        | IrInstructionKind::LoadStorage { storage }
        | IrInstructionKind::StoreStorage { storage, .. } => vec![storage],
        IrInstructionKind::Nop
        | IrInstructionKind::Binary { .. }
        | IrInstructionKind::Unary { .. } => Vec::new(),
    }
}

/// A stable compiler-owned loop identity within an IR function.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct IrLoopId(String);

impl IrLoopId {
    #[must_use]
    pub fn for_function(function: &SemanticId, name: &str) -> Self {
        Self(format!("{function}/loop:{name}"))
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for IrLoopId {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.0)
    }
}

/// One ordered instruction region in an IR function.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IrBlock {
    pub id: IrBlockId,
    pub provenance: SourceProvenance,
    pub instructions: Vec<IrInstruction>,
}

/// A directed conditional branch between compiler-owned basic blocks.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IrBranchEdge {
    pub from: IrBlockId,
    pub to: IrBlockId,
    pub arm: IrBranchArm,
    pub provenance: SourceProvenance,
}

/// The outcome of a conditional branch represented by an IR edge.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IrBranchArm {
    True,
    False,
}

/// A compiler-owned natural loop whose body includes its header and latches.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IrLoop {
    pub id: IrLoopId,
    pub header: IrBlockId,
    pub body: Vec<IrBlockId>,
    pub latches: Vec<IrBlockId>,
    pub exits: Vec<IrBlockId>,
    pub provenance: SourceProvenance,
}

/// The immutable dominator relation derived from one canonical IR function.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IrDominatorTree {
    pub function: SemanticId,
    pub dominators: BTreeMap<IrBlockId, Vec<IrBlockId>>,
}

impl IrDominatorTree {
    #[must_use]
    pub fn dominators_of(&self, block: &IrBlockId) -> Option<&[IrBlockId]> {
        self.dominators.get(block).map(Vec::as_slice)
    }

    #[must_use]
    pub fn dominates(&self, dominator: &IrBlockId, block: &IrBlockId) -> bool {
        self.dominators_of(block)
            .is_some_and(|dominators| dominators.contains(dominator))
    }
}

/// The immutable post-dominator relation derived from one canonical IR function.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IrPostDominatorTree {
    pub function: SemanticId,
    pub post_dominators: BTreeMap<IrBlockId, Vec<IrBlockId>>,
}

impl IrPostDominatorTree {
    #[must_use]
    pub fn post_dominators_of(&self, block: &IrBlockId) -> Option<&[IrBlockId]> {
        self.post_dominators.get(block).map(Vec::as_slice)
    }

    #[must_use]
    pub fn post_dominates(&self, post_dominator: &IrBlockId, block: &IrBlockId) -> bool {
        self.post_dominators_of(block)
            .is_some_and(|post_dominators| post_dominators.contains(post_dominator))
    }
}

/// Computes dominators from the function's entry block and canonical branch edges.
#[must_use]
pub fn compute_dominators(function: &IrFunction) -> IrDominatorTree {
    let block_ids = function
        .blocks
        .iter()
        .map(|block| block.id.clone())
        .collect::<BTreeSet<_>>();
    let mut predecessors = block_ids
        .iter()
        .cloned()
        .map(|block| (block, BTreeSet::new()))
        .collect::<BTreeMap<_, _>>();
    for edge in &function.branch_edges {
        if block_ids.contains(&edge.from) && block_ids.contains(&edge.to) {
            predecessors
                .entry(edge.to.clone())
                .or_default()
                .insert(edge.from.clone());
        }
    }

    let mut dominators = block_ids
        .iter()
        .cloned()
        .map(|block| {
            let initial = if block == function.entry_block {
                BTreeSet::from([block.clone()])
            } else {
                block_ids.clone()
            };
            (block, initial)
        })
        .collect::<BTreeMap<_, _>>();
    let mut changed = true;
    while changed {
        changed = false;
        for block in &block_ids {
            if *block == function.entry_block {
                continue;
            }
            let mut next = predecessors[block]
                .iter()
                .filter_map(|predecessor| dominators.get(predecessor).cloned())
                .reduce(|mut shared, predecessor_dominators| {
                    shared.retain(|candidate| predecessor_dominators.contains(candidate));
                    shared
                })
                .unwrap_or_default();
            next.insert(block.clone());
            if dominators.get(block) != Some(&next) {
                dominators.insert(block.clone(), next);
                changed = true;
            }
        }
    }

    IrDominatorTree {
        function: function.id.clone(),
        dominators: dominators
            .into_iter()
            .map(|(block, dominators)| (block, dominators.into_iter().collect()))
            .collect(),
    }
}

/// Computes post-dominators from canonical branch edges and their CFG exit blocks.
#[must_use]
pub fn compute_post_dominators(function: &IrFunction) -> IrPostDominatorTree {
    let block_ids = function
        .blocks
        .iter()
        .map(|block| block.id.clone())
        .collect::<BTreeSet<_>>();
    let mut successors = block_ids
        .iter()
        .cloned()
        .map(|block| (block, BTreeSet::new()))
        .collect::<BTreeMap<_, _>>();
    for edge in &function.branch_edges {
        if block_ids.contains(&edge.from) && block_ids.contains(&edge.to) {
            successors
                .entry(edge.from.clone())
                .or_default()
                .insert(edge.to.clone());
        }
    }
    let exits = successors
        .iter()
        .filter_map(|(block, successors)| successors.is_empty().then_some(block.clone()))
        .collect::<BTreeSet<_>>();

    let mut post_dominators = block_ids
        .iter()
        .cloned()
        .map(|block| {
            let initial = if exits.contains(&block) {
                BTreeSet::from([block.clone()])
            } else {
                block_ids.clone()
            };
            (block, initial)
        })
        .collect::<BTreeMap<_, _>>();
    let mut changed = true;
    while changed {
        changed = false;
        for block in &block_ids {
            if exits.contains(block) {
                continue;
            }
            let mut next = successors[block]
                .iter()
                .filter_map(|successor| post_dominators.get(successor).cloned())
                .reduce(|mut shared, successor_post_dominators| {
                    shared.retain(|candidate| successor_post_dominators.contains(candidate));
                    shared
                })
                .unwrap_or_default();
            next.insert(block.clone());
            if post_dominators.get(block) != Some(&next) {
                post_dominators.insert(block.clone(), next);
                changed = true;
            }
        }
    }

    IrPostDominatorTree {
        function: function.id.clone(),
        post_dominators: post_dominators
            .into_iter()
            .map(|(block, post_dominators)| (block, post_dominators.into_iter().collect()))
            .collect(),
    }
}

/// One backend-neutral instruction with stable source provenance.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IrInstruction {
    pub id: IrInstructionId,
    pub provenance: SourceProvenance,
    pub result: Option<IrValueId>,
    pub semantic_origin: Option<SemanticId>,
    pub kind: IrInstructionKind,
}

/// Instruction forms available to canonical IR lowering.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IrInstructionKind {
    Nop,
    InitializeStorage {
        storage: IrStorageId,
    },
    LoadStorage {
        storage: IrStorageId,
    },
    StoreStorage {
        storage: IrStorageId,
        value: IrOperand,
    },
    Binary {
        operation: IrBinaryOperation,
        left: IrOperand,
        right: IrOperand,
    },
    Unary {
        operation: IrUnaryOperation,
        operand: IrOperand,
    },
}

/// A binary operation with value-producing IR semantics.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IrBinaryOperation {
    Add,
    Subtract,
    Multiply,
    Divide,
}

/// A unary operation with value-producing IR semantics.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IrUnaryOperation {
    Not,
    Negate,
}

#[cfg(test)]
mod tests {
    use super::{
        compute_dominators, compute_post_dominators, lower_components_to_ir,
        validate_intermediate_representation, IntermediateRepresentation, IrBlock, IrBlockId,
        IrBranchArm, IrBranchEdge, IrConstant, IrFunction, IrInstruction, IrInstructionId,
        IrInstructionKind, IrLoop, IrLoopId, IrModule, IrOperand, IrStorageId, IrValue,
        IrValueDefinition, IrValueId,
    };
    use crate::{SemanticId, SemanticType, SourceProvenance};
    use std::collections::BTreeMap;

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
            entry_block: IrBlockId::entry_for(
                &SemanticId::component(Some("x-counter"), "Counter").method("increment"),
            ),
            blocks: vec![IrBlock {
                id: IrBlockId::entry_for(
                    &SemanticId::component(Some("x-counter"), "Counter").method("increment"),
                ),
                provenance: provenance.clone(),
                instructions: vec![IrInstruction {
                    id: IrInstructionId::for_block(
                        &IrBlockId::entry_for(
                            &SemanticId::component(Some("x-counter"), "Counter")
                                .method("increment"),
                        ),
                        0,
                    ),
                    provenance,
                    result: None,
                    semantic_origin: None,
                    kind: IrInstructionKind::Nop,
                }],
            }],
            branch_edges: Vec::new(),
            values: BTreeMap::new(),
            loops: Vec::new(),
        };
        let ir = IntermediateRepresentation {
            modules: vec![IrModule {
                path: "src/Counter.tsx".into(),
                components: vec![SemanticId::component(Some("x-counter"), "Counter")],
                storages: Vec::new(),
                storage_initializers: Vec::new(),
                template_entrypoints: Vec::new(),
                functions: vec![function],
            }],
        };

        assert_eq!(ir.modules[0].functions[0].blocks[0].instructions.len(), 1);
    }

    #[test]
    fn represents_provenanced_conditional_branch_edges() {
        let provenance = SourceProvenance::new(
            "src/Counter.tsx",
            ezc_parser::SourceSpan {
                start: 0,
                end: 1,
                line: 1,
                column: 1,
            },
        );
        let function = SemanticId::component(Some("x-counter"), "Counter").method("render");
        let entry = IrBlockId::entry_for(&function);
        let when_true = IrBlockId::for_function(&function, "when-true");
        let branch = IrBranchEdge {
            from: entry,
            to: when_true,
            arm: IrBranchArm::True,
            provenance,
        };

        assert_eq!(
            branch.from.as_str(),
            "component:x-counter/method:render/block:entry"
        );
        assert_eq!(
            branch.to.as_str(),
            "component:x-counter/method:render/block:when-true"
        );
        assert_eq!(branch.arm, IrBranchArm::True);
    }

    #[test]
    fn keeps_ir_identity_domains_distinct_and_deterministic() {
        let function = SemanticId::component(Some("x-counter"), "Counter").method("increment");
        let block = IrBlockId::entry_for(&function);
        let instruction = IrInstructionId::for_block(&block, 0);
        let value = IrValueId::for_function(&function, 0);
        let storage = IrStorageId::for_semantic_origin(
            &SemanticId::component(Some("x-counter"), "Counter").state_field("count"),
        );

        assert_eq!(
            instruction.as_str(),
            "component:x-counter/method:increment/block:entry/instruction:0"
        );
        assert_eq!(
            value.as_str(),
            "component:x-counter/method:increment/value:0"
        );
        assert_eq!(storage.as_str(), "storage:component:x-counter/state:count");
        assert_ne!(instruction.as_str(), value.as_str());
        assert_ne!(value.as_str(), storage.as_str());
    }

    #[test]
    fn represents_closed_ir_operands_without_semantic_identity_operands() {
        let function = SemanticId::component(Some("x-counter"), "Counter").method("increment");
        let value = IrOperand::Value(IrValueId::for_function(&function, 0));
        let constant = IrOperand::Constant(IrConstant::Number("1".to_string()));
        let storage = IrOperand::Storage(IrStorageId::for_semantic_origin(
            &SemanticId::component(Some("x-counter"), "Counter").state_field("count"),
        ));

        assert!(matches!(value, IrOperand::Value(_)));
        assert!(
            matches!(constant, IrOperand::Constant(IrConstant::Number(number)) if number == "1")
        );
        assert!(matches!(storage, IrOperand::Storage(_)));
    }

    #[test]
    fn records_instruction_results_separately_from_operation_identity() {
        let provenance = SourceProvenance::new(
            "src/Counter.tsx",
            ezc_parser::SourceSpan {
                start: 0,
                end: 1,
                line: 1,
                column: 1,
            },
        );
        let function = SemanticId::component(Some("x-counter"), "Counter").method("increment");
        let block = IrBlockId::entry_for(&function);
        let storage = IrStorageId::for_semantic_origin(
            &SemanticId::component(Some("x-counter"), "Counter").state_field("count"),
        );
        let instruction = IrInstruction {
            id: IrInstructionId::for_block(&block, 0),
            provenance,
            result: Some(IrValueId::for_function(&function, 0)),
            semantic_origin: Some(
                SemanticId::component(Some("x-counter"), "Counter").state_field("count"),
            ),
            kind: IrInstructionKind::LoadStorage { storage },
        };

        assert_eq!(
            instruction.id.as_str(),
            "component:x-counter/method:increment/block:entry/instruction:0"
        );
        assert_eq!(
            instruction.result.expect("load result").as_str(),
            "component:x-counter/method:increment/value:0"
        );
        assert!(matches!(
            instruction.kind,
            IrInstructionKind::LoadStorage { .. }
        ));
    }

    #[test]
    fn indexes_values_by_function_scoped_value_identity() {
        let provenance = SourceProvenance::new(
            "src/Counter.tsx",
            ezc_parser::SourceSpan {
                start: 0,
                end: 1,
                line: 1,
                column: 1,
            },
        );
        let id = SemanticId::component(Some("x-counter"), "Counter").method("increment");
        let entry = IrBlockId::entry_for(&id);
        let value_id = IrValueId::for_function(&id, 0);
        let value = IrValue {
            id: value_id.clone(),
            definition: IrValueDefinition::Instruction(IrInstructionId::for_block(&entry, 0)),
            semantic_type: SemanticType::Number,
            provenance: provenance.clone(),
            semantic_origin: Some(
                SemanticId::component(Some("x-counter"), "Counter").state_field("count"),
            ),
        };
        let function = IrFunction {
            id,
            name: "increment".to_string(),
            provenance: provenance.clone(),
            entry_block: entry.clone(),
            blocks: vec![IrBlock {
                id: entry,
                provenance,
                instructions: Vec::new(),
            }],
            branch_edges: Vec::new(),
            values: BTreeMap::from([(value_id.clone(), value)]),
            loops: Vec::new(),
        };

        assert_eq!(
            function.value(&value_id).expect("value").semantic_type,
            SemanticType::Number
        );
        assert!(matches!(
            function.value(&value_id).expect("value").definition,
            IrValueDefinition::Instruction(_)
        ));
    }

    #[test]
    fn represents_natural_loops_with_header_latches_and_exits() {
        let provenance = SourceProvenance::new(
            "src/Counter.tsx",
            ezc_parser::SourceSpan {
                start: 0,
                end: 1,
                line: 1,
                column: 1,
            },
        );
        let function = SemanticId::component(Some("x-counter"), "Counter").method("render");
        let header = IrBlockId::for_function(&function, "loop-header");
        let latch = IrBlockId::for_function(&function, "loop-latch");
        let exit = IrBlockId::for_function(&function, "loop-exit");
        let loop_region = IrLoop {
            id: IrLoopId::for_function(&function, "items"),
            header: header.clone(),
            body: vec![header, latch.clone()],
            latches: vec![latch],
            exits: vec![exit],
            provenance,
        };

        assert_eq!(
            loop_region.id.as_str(),
            "component:x-counter/method:render/loop:items"
        );
        assert_eq!(loop_region.body[0], loop_region.header);
        assert_eq!(loop_region.body[1], loop_region.latches[0]);
        assert_eq!(
            loop_region.exits[0].as_str(),
            "component:x-counter/method:render/block:loop-exit"
        );
    }

    #[test]
    fn computes_dominators_from_canonical_branch_edges() {
        let provenance = SourceProvenance::new(
            "src/Counter.tsx",
            ezc_parser::SourceSpan {
                start: 0,
                end: 1,
                line: 1,
                column: 1,
            },
        );
        let id = SemanticId::component(Some("x-counter"), "Counter").method("render");
        let entry = IrBlockId::entry_for(&id);
        let when_true = IrBlockId::for_function(&id, "when-true");
        let when_false = IrBlockId::for_function(&id, "when-false");
        let merge = IrBlockId::for_function(&id, "merge");
        let function = IrFunction {
            id: id.clone(),
            name: "render".to_string(),
            provenance: provenance.clone(),
            entry_block: entry.clone(),
            blocks: vec![
                IrBlock {
                    id: entry.clone(),
                    provenance: provenance.clone(),
                    instructions: Vec::new(),
                },
                IrBlock {
                    id: when_true.clone(),
                    provenance: provenance.clone(),
                    instructions: Vec::new(),
                },
                IrBlock {
                    id: when_false.clone(),
                    provenance: provenance.clone(),
                    instructions: Vec::new(),
                },
                IrBlock {
                    id: merge.clone(),
                    provenance: provenance.clone(),
                    instructions: Vec::new(),
                },
            ],
            branch_edges: vec![
                IrBranchEdge {
                    from: entry.clone(),
                    to: when_true.clone(),
                    arm: IrBranchArm::True,
                    provenance: provenance.clone(),
                },
                IrBranchEdge {
                    from: entry.clone(),
                    to: when_false.clone(),
                    arm: IrBranchArm::False,
                    provenance: provenance.clone(),
                },
                IrBranchEdge {
                    from: when_true,
                    to: merge.clone(),
                    arm: IrBranchArm::True,
                    provenance: provenance.clone(),
                },
                IrBranchEdge {
                    from: when_false,
                    to: merge.clone(),
                    arm: IrBranchArm::False,
                    provenance,
                },
            ],
            values: BTreeMap::new(),
            loops: Vec::new(),
        };

        let tree = compute_dominators(&function);

        assert_eq!(tree.function, id);
        assert_eq!(tree.dominators[&entry], vec![entry.clone()]);
        assert_eq!(tree.dominators[&merge], vec![entry, merge]);
    }

    #[test]
    fn computes_post_dominators_from_canonical_branch_edges() {
        let provenance = SourceProvenance::new(
            "src/Counter.tsx",
            ezc_parser::SourceSpan {
                start: 0,
                end: 1,
                line: 1,
                column: 1,
            },
        );
        let id = SemanticId::component(Some("x-counter"), "Counter").method("render");
        let entry = IrBlockId::entry_for(&id);
        let when_true = IrBlockId::for_function(&id, "when-true");
        let when_false = IrBlockId::for_function(&id, "when-false");
        let merge = IrBlockId::for_function(&id, "merge");
        let function = IrFunction {
            id: id.clone(),
            name: "render".to_string(),
            provenance: provenance.clone(),
            entry_block: entry.clone(),
            blocks: [
                entry.clone(),
                when_true.clone(),
                when_false.clone(),
                merge.clone(),
            ]
            .into_iter()
            .map(|id| IrBlock {
                id,
                provenance: provenance.clone(),
                instructions: Vec::new(),
            })
            .collect(),
            branch_edges: vec![
                IrBranchEdge {
                    from: entry.clone(),
                    to: when_true.clone(),
                    arm: IrBranchArm::True,
                    provenance: provenance.clone(),
                },
                IrBranchEdge {
                    from: entry.clone(),
                    to: when_false.clone(),
                    arm: IrBranchArm::False,
                    provenance: provenance.clone(),
                },
                IrBranchEdge {
                    from: when_true,
                    to: merge.clone(),
                    arm: IrBranchArm::True,
                    provenance: provenance.clone(),
                },
                IrBranchEdge {
                    from: when_false,
                    to: merge.clone(),
                    arm: IrBranchArm::False,
                    provenance,
                },
            ],
            values: BTreeMap::new(),
            loops: Vec::new(),
        };

        let tree = compute_post_dominators(&function);

        assert_eq!(tree.function, id);
        assert_eq!(tree.post_dominators[&merge], vec![merge.clone()]);
        assert_eq!(tree.post_dominators[&entry], vec![entry, merge]);
    }

    #[test]
    fn queries_canonical_cfg_connectivity_and_dominance() {
        let provenance = SourceProvenance::new(
            "src/Counter.tsx",
            ezc_parser::SourceSpan {
                start: 0,
                end: 1,
                line: 1,
                column: 1,
            },
        );
        let id = SemanticId::component(Some("x-counter"), "Counter").method("render");
        let entry = IrBlockId::entry_for(&id);
        let exit = IrBlockId::for_function(&id, "exit");
        let function = IrFunction {
            id,
            name: "render".to_string(),
            provenance: provenance.clone(),
            entry_block: entry.clone(),
            blocks: [entry.clone(), exit.clone()]
                .into_iter()
                .map(|id| IrBlock {
                    id,
                    provenance: provenance.clone(),
                    instructions: Vec::new(),
                })
                .collect(),
            branch_edges: vec![IrBranchEdge {
                from: entry.clone(),
                to: exit.clone(),
                arm: IrBranchArm::True,
                provenance,
            }],
            values: BTreeMap::new(),
            loops: Vec::new(),
        };
        let dominators = compute_dominators(&function);
        let post_dominators = compute_post_dominators(&function);

        assert_eq!(function.block(&entry).expect("entry").id, entry);
        assert_eq!(
            function.successor_blocks(&function.entry_block),
            vec![exit.clone()]
        );
        assert_eq!(
            function.predecessor_blocks(&exit),
            vec![function.entry_block.clone()]
        );
        assert!(function.is_exit_block(&exit));
        assert!(dominators.dominates(&function.entry_block, &exit));
        assert!(post_dominators.post_dominates(&exit, &function.entry_block));
    }

    #[test]
    fn lowers_components_into_modules_with_entry_blocks() {
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
        assert_eq!(ir.modules[0].functions[0].blocks.len(), 1);
        assert_eq!(
            ir.modules[0].functions[0].entry_block,
            ir.modules[0].functions[0].blocks[0].id
        );
        assert_eq!(
            ir.modules[0].functions[0].entry_block.as_str(),
            format!("{}/block:entry", ir.modules[0].functions[0].id).as_str()
        );
        assert!(ir.modules[0].functions[0].blocks[0].instructions.is_empty());
        assert!(ir.modules[0].functions[0].branch_edges.is_empty());
        assert!(ir.modules[0].functions[0].loops.is_empty());
        assert!(matches!(
            ir.modules[0].storage_initializers[0].kind,
            IrInstructionKind::InitializeStorage { .. }
        ));
        assert_eq!(ir.modules[0].storages.len(), 1);
        assert_eq!(
            ir.modules[0].storages[0].id.as_str(),
            format!("storage:{}", model.components[0].state_fields[0].id).as_str()
        );
        assert_eq!(
            ir.modules[0].storage_initializers[0].semantic_origin,
            Some(model.components[0].state_fields[0].id.clone())
        );
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

    #[test]
    fn validates_result_value_definitions_and_storage_references() {
        let parsed = ezc_parser::parse_file(
            "src/Counter.tsx",
            "@component(\"x-counter\") class Counter extends Component { count = state(0); increment() {} }",
        );
        let model = crate::build_application_semantic_model(&parsed);
        let mut representation = lower_components_to_ir(&model);
        assert!(validate_intermediate_representation(&representation).is_empty());

        let storage = representation.modules[0].storages[0].id.clone();
        let function = &mut representation.modules[0].functions[0];
        let result = IrValueId::for_function(&function.id, 0);
        function.blocks[0].instructions.push(IrInstruction {
            id: IrInstructionId::for_block(&function.entry_block, 0),
            provenance: function.provenance.clone(),
            result: Some(result),
            semantic_origin: None,
            kind: IrInstructionKind::LoadStorage { storage },
        });

        assert!(validate_intermediate_representation(&representation)
            .iter()
            .any(|diagnostic| diagnostic.code == "EZIR1006"));
    }
}
