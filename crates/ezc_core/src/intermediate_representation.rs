use std::collections::{BTreeMap, BTreeSet};
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
                    entry_block: IrBlockId::entry_for(&method.id),
                    blocks: vec![IrBlock {
                        id: IrBlockId::entry_for(&method.id),
                        provenance: provenance.clone(),
                        instructions: Vec::new(),
                    }],
                    branch_edges: Vec::new(),
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

impl std::fmt::Display for IrStorageId {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.0)
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
        compute_dominators, compute_post_dominators, lower_components_to_ir,
        IntermediateRepresentation, IrBlock, IrBlockId, IrBranchArm, IrBranchEdge, IrFunction,
        IrInstruction, IrInstructionId, IrInstructionKind, IrLoop, IrLoopId, IrModule, IrStorageId,
        IrValueId,
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
            entry_block: IrBlockId::entry_for(
                &SemanticId::component(Some("x-counter"), "Counter").method("increment"),
            ),
            blocks: vec![IrBlock {
                id: IrBlockId::entry_for(
                    &SemanticId::component(Some("x-counter"), "Counter").method("increment"),
                ),
                provenance: provenance.clone(),
                instructions: vec![IrInstruction {
                    id: "entry.0".to_string(),
                    provenance,
                    kind: IrInstructionKind::Nop,
                }],
            }],
            branch_edges: Vec::new(),
            loops: Vec::new(),
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
