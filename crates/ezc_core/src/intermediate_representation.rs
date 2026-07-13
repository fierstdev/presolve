use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use crate::component_graph::{
    ArithmeticOperator, ComparisonOperator, LogicalOperator, UnaryOperator,
};
use crate::{
    ApplicationSemanticModel, ComponentNode, ComputedPurity, ComputedValue, ExpressionNode,
    ExpressionNodeKind, SemanticId, SemanticReference, SemanticReferenceKind, SemanticType,
    SerializableValue, SourceProvenance,
};

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
    pub computed_evaluations: Vec<IrComputedEvaluation>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IrTemplateEntrypoint {
    pub template: SemanticId,
    pub render_method: SemanticId,
    pub provenance: SourceProvenance,
}

/// Canonical IR evaluation result for one computed semantic entity.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IrComputedEvaluation {
    pub computed: SemanticId,
    pub function: SemanticId,
    pub result: IrValueId,
    pub provenance: SourceProvenance,
}

/// A stable compiler-owned DOM node identity within a template entrypoint.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct IrDomNodeId(String);

impl IrDomNodeId {
    #[must_use]
    pub fn for_template(template: &SemanticId, path: &str) -> Self {
        Self(format!("{template}/dom:{path}"))
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Backend-neutral DOM node semantics.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IrDomNode {
    pub id: IrDomNodeId,
    pub kind: IrDomNodeKind,
    pub provenance: SourceProvenance,
}

/// Structural DOM node forms before text, bindings, attributes, and events are lowered.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IrDomNodeKind {
    Element {
        tag: String,
        children: Vec<IrDomNodeId>,
    },
    Fragment {
        children: Vec<IrDomNodeId>,
    },
}

/// Text semantics owned by canonical DOM IR rather than a backend renderer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IrDomText {
    pub node: IrDomNodeId,
    pub value: String,
    pub provenance: SourceProvenance,
}

/// A value-driven DOM update target, independent of backend rendering syntax.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IrDomBinding {
    pub node: IrDomNodeId,
    pub value: IrValueId,
    pub provenance: SourceProvenance,
}

/// A static or value-driven DOM attribute independent of backend serialization.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IrDomAttribute {
    pub node: IrDomNodeId,
    pub name: String,
    pub value: IrDomAttributeValue,
    pub provenance: SourceProvenance,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IrDomAttributeValue {
    Static(String),
    Binding(IrValueId),
}

/// A DOM event bound to an authored handler semantic identity.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IrDomEvent {
    pub node: IrDomNodeId,
    pub event: String,
    pub handler: SemanticId,
    pub provenance: SourceProvenance,
}

/// Conditional DOM output driven by a canonical IR value.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IrDomConditional {
    pub condition: IrValueId,
    pub when_true: IrDomNodeId,
    pub when_false: Option<IrDomNodeId>,
    pub provenance: SourceProvenance,
}

/// Repeated DOM output driven by one canonical iterable value.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IrDomList {
    pub iterable: IrValueId,
    pub item: IrValueId,
    pub index: Option<IrValueId>,
    pub body: IrDomNodeId,
    pub provenance: SourceProvenance,
}

/// Deterministic read-only lookup surface for canonical DOM nodes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IrDomInspection {
    pub nodes: BTreeMap<IrDomNodeId, IrDomNode>,
}

/// Compiler-owned reactive dependency topology.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct IrReactiveGraph {
    pub nodes: BTreeMap<String, IrReactiveNode>,
    pub edges: Vec<IrReactiveEdge>,
}

/// Immutable transitive dependency and dependent topology derived from a
/// canonical reactive graph.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct IrReactiveTransitiveAnalysis {
    pub dependencies: BTreeMap<String, Vec<String>>,
    pub dependents: BTreeMap<String, Vec<String>>,
}

/// One strongly connected group of computed reactive nodes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IrReactiveCycle {
    pub nodes: Vec<String>,
}

/// Immutable computed dependency-cycle analysis derived from a canonical
/// reactive graph.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct IrReactiveCycleAnalysis {
    pub cycles: Vec<IrReactiveCycle>,
}

/// Compiler-generated evaluation order and update batches for computed values.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct IrComputedEvaluationPlan {
    pub evaluation_order: Vec<String>,
    pub update_batches: Vec<Vec<String>>,
    pub unplanned: Vec<String>,
}

/// Derive deterministic transitive reactive dependency and dependent maps.
///
/// This analysis preserves direct graph topology and deliberately makes no
/// cycle diagnosis or scheduling decision.
#[must_use]
pub fn analyze_reactive_transitive_graph(graph: &IrReactiveGraph) -> IrReactiveTransitiveAnalysis {
    let dependencies = graph
        .nodes
        .keys()
        .map(|id| {
            (
                id.clone(),
                graph.transitive_targets(id, IrReactiveEdgeKind::Reads),
            )
        })
        .collect();
    let dependents = graph
        .nodes
        .keys()
        .map(|id| {
            (
                id.clone(),
                graph.transitive_targets(id, IrReactiveEdgeKind::Invalidates),
            )
        })
        .collect();

    IrReactiveTransitiveAnalysis {
        dependencies,
        dependents,
    }
}

/// Detect deterministic computed dependency cycles from direct reactive reads.
#[must_use]
pub fn analyze_reactive_cycles(graph: &IrReactiveGraph) -> IrReactiveCycleAnalysis {
    let computed = graph
        .nodes
        .iter()
        .filter(|(_, node)| node.kind == IrReactiveNodeKind::Computed)
        .map(|(id, _)| id.clone())
        .collect::<BTreeSet<_>>();
    let adjacency = computed
        .iter()
        .map(|id| {
            let targets = graph
                .edges
                .iter()
                .filter(|edge| {
                    edge.source == *id
                        && edge.kind == IrReactiveEdgeKind::Reads
                        && computed.contains(&edge.target)
                })
                .map(|edge| edge.target.clone())
                .collect();
            (id.clone(), targets)
        })
        .collect::<BTreeMap<String, BTreeSet<String>>>();
    let reverse_adjacency = reverse_reactive_adjacency(&adjacency);
    let mut visited = BTreeSet::new();
    let mut finish_order = Vec::new();
    for node in &computed {
        visit_reactive_node(node, &adjacency, &mut visited, &mut finish_order);
    }

    let mut cycles = Vec::new();
    visited.clear();
    for node in finish_order.into_iter().rev() {
        if !visited.insert(node.clone()) {
            continue;
        }
        let mut members = BTreeSet::new();
        collect_reactive_component(&node, &reverse_adjacency, &mut visited, &mut members);
        let is_self_cycle = members.len() == 1
            && adjacency
                .get(&node)
                .is_some_and(|targets| targets.contains(&node));
        if members.len() > 1 || is_self_cycle {
            cycles.push(IrReactiveCycle {
                nodes: members.into_iter().collect(),
            });
        }
    }
    cycles.sort_by(|left, right| left.nodes.cmp(&right.nodes));

    IrReactiveCycleAnalysis { cycles }
}

/// Build a deterministic computed evaluation plan through the canonical update
/// scheduler.
#[must_use]
pub fn plan_computed_evaluation(graph: &IrReactiveGraph) -> IrComputedEvaluationPlan {
    let nodes = graph
        .nodes
        .iter()
        .filter(|(_, node)| node.kind == IrReactiveNodeKind::Computed)
        .map(|(id, node)| (id.clone(), node.clone()))
        .collect::<BTreeMap<_, _>>();
    let edges = graph
        .edges
        .iter()
        .filter(|edge| {
            edge.kind == IrReactiveEdgeKind::Invalidates
                && nodes.contains_key(&edge.source)
                && nodes.contains_key(&edge.target)
        })
        .cloned()
        .collect();
    let inspection = IrUpdateScheduler::new(IrReactiveGraph { nodes, edges }).inspect();

    IrComputedEvaluationPlan {
        evaluation_order: inspection.order,
        update_batches: inspection.batches,
        unplanned: inspection.cycles,
    }
}

fn reverse_reactive_adjacency(
    adjacency: &BTreeMap<String, BTreeSet<String>>,
) -> BTreeMap<String, BTreeSet<String>> {
    let mut reversed = adjacency
        .keys()
        .cloned()
        .map(|node| (node, BTreeSet::new()))
        .collect::<BTreeMap<_, _>>();
    for (source, targets) in adjacency {
        for target in targets {
            reversed
                .get_mut(target)
                .expect("computed dependency target should be a reactive node")
                .insert(source.clone());
        }
    }
    reversed
}

fn visit_reactive_node(
    node: &str,
    adjacency: &BTreeMap<String, BTreeSet<String>>,
    visited: &mut BTreeSet<String>,
    finish_order: &mut Vec<String>,
) {
    if !visited.insert(node.to_string()) {
        return;
    }
    for target in adjacency
        .get(node)
        .expect("computed reactive node should have adjacency")
    {
        visit_reactive_node(target, adjacency, visited, finish_order);
    }
    finish_order.push(node.to_string());
}

fn collect_reactive_component(
    node: &str,
    adjacency: &BTreeMap<String, BTreeSet<String>>,
    visited: &mut BTreeSet<String>,
    members: &mut BTreeSet<String>,
) {
    members.insert(node.to_string());
    for target in adjacency
        .get(node)
        .expect("computed reactive node should have reverse adjacency")
    {
        if visited.insert(target.clone()) {
            collect_reactive_component(target, adjacency, visited, members);
        }
    }
}

/// Build compiler-owned reactive topology from canonical state, computed, and
/// computed-reference products.
///
/// # Panics
///
/// Panics when a state field has no canonical source provenance.
#[must_use]
pub fn build_reactive_graph(
    components: &[ComponentNode],
    computed_values: &BTreeMap<SemanticId, ComputedValue>,
    references: &[SemanticReference],
    provenance: &BTreeMap<SemanticId, SourceProvenance>,
) -> IrReactiveGraph {
    let mut nodes = BTreeMap::new();
    for component in components {
        for field in &component.state_fields {
            let id = field.id.as_str().to_string();
            nodes.insert(
                id.clone(),
                IrReactiveNode {
                    id,
                    kind: IrReactiveNodeKind::State,
                    provenance: provenance
                        .get(&field.id)
                        .expect("state field should have canonical provenance")
                        .clone(),
                },
            );
        }
    }
    for computed in computed_values.values() {
        let id = computed.id.as_str().to_string();
        nodes.insert(
            id.clone(),
            IrReactiveNode {
                id,
                kind: IrReactiveNodeKind::Computed,
                provenance: computed.provenance.clone(),
            },
        );
    }

    let mut edges = Vec::new();
    for reference in references.iter().filter(|reference| {
        matches!(
            reference.kind,
            SemanticReferenceKind::ComputedState | SemanticReferenceKind::ComputedComputed
        )
    }) {
        let source = reference.source.as_str().to_string();
        let target = reference.target.as_str().to_string();
        if !nodes.contains_key(&source) || !nodes.contains_key(&target) {
            continue;
        }
        edges.push(IrReactiveEdge {
            source: source.clone(),
            target: target.clone(),
            kind: IrReactiveEdgeKind::Reads,
            provenance: reference.provenance.clone(),
        });
        edges.push(IrReactiveEdge {
            source: target,
            target: source,
            kind: IrReactiveEdgeKind::Invalidates,
            provenance: reference.provenance.clone(),
        });
    }
    edges.sort_by(|left, right| {
        (left.kind, left.source.as_str(), left.target.as_str()).cmp(&(
            right.kind,
            right.source.as_str(),
            right.target.as_str(),
        ))
    });
    edges.dedup_by(|left, right| {
        left.kind == right.kind && left.source == right.source && left.target == right.target
    });

    IrReactiveGraph { nodes, edges }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IrReactiveNode {
    pub id: String,
    pub kind: IrReactiveNodeKind,
    pub provenance: SourceProvenance,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IrReactiveNodeKind {
    State,
    Computed,
    Action,
    Template,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IrReactiveEdge {
    pub source: String,
    pub target: String,
    pub kind: IrReactiveEdgeKind,
    pub provenance: SourceProvenance,
}

/// Compiler-owned foundation for planning reactive updates from dependency topology.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IrUpdateScheduler {
    pub graph: IrReactiveGraph,
}

/// Read-only summary of one compiler-generated update plan.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IrSchedulerInspection {
    pub order: Vec<String>,
    pub batches: Vec<Vec<String>>,
    pub cycles: Vec<String>,
}

impl IrUpdateScheduler {
    #[must_use]
    pub fn new(graph: IrReactiveGraph) -> Self {
        Self { graph }
    }

    #[must_use]
    pub fn dependency_order(&self) -> Vec<String> {
        let mut incoming = self
            .graph
            .nodes
            .keys()
            .cloned()
            .map(|id| (id, 0_usize))
            .collect::<BTreeMap<_, _>>();
        for edge in &self.graph.edges {
            if let Some(count) = incoming.get_mut(&edge.target) {
                *count += 1;
            }
        }
        let mut ready = incoming
            .iter()
            .filter(|(_, count)| **count == 0)
            .map(|(id, _)| id.clone())
            .collect::<BTreeSet<_>>();
        let mut order = Vec::new();
        while let Some(id) = ready.pop_first() {
            order.push(id.clone());
            for edge in self.graph.dependents_of(&id) {
                if let Some(count) = incoming.get_mut(&edge.target) {
                    *count -= 1;
                    if *count == 0 {
                        ready.insert(edge.target.clone());
                    }
                }
            }
        }
        order
    }

    #[must_use]
    pub fn update_batches(&self) -> Vec<Vec<String>> {
        let mut incoming = self
            .graph
            .nodes
            .keys()
            .cloned()
            .map(|id| (id, 0_usize))
            .collect::<BTreeMap<_, _>>();
        for edge in &self.graph.edges {
            if let Some(count) = incoming.get_mut(&edge.target) {
                *count += 1;
            }
        }
        let mut ready = incoming
            .iter()
            .filter(|(_, count)| **count == 0)
            .map(|(id, _)| id.clone())
            .collect::<BTreeSet<_>>();
        let mut batches = Vec::new();
        while !ready.is_empty() {
            let batch = std::mem::take(&mut ready).into_iter().collect::<Vec<_>>();
            for id in &batch {
                for edge in self.graph.dependents_of(id) {
                    if let Some(count) = incoming.get_mut(&edge.target) {
                        *count -= 1;
                        if *count == 0 {
                            ready.insert(edge.target.clone());
                        }
                    }
                }
            }
            batches.push(batch);
        }
        batches
    }

    #[must_use]
    pub fn cyclic_nodes(&self) -> Vec<String> {
        let ordered = self.dependency_order().into_iter().collect::<BTreeSet<_>>();
        self.graph
            .nodes
            .keys()
            .filter(|id| !ordered.contains(*id))
            .cloned()
            .collect()
    }

    #[must_use]
    pub fn inspect(&self) -> IrSchedulerInspection {
        IrSchedulerInspection {
            order: self.dependency_order(),
            batches: self.update_batches(),
            cycles: self.cyclic_nodes(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum IrReactiveEdgeKind {
    Reads,
    Invalidates,
}

impl IrReactiveGraph {
    fn transitive_targets(&self, source: &str, kind: IrReactiveEdgeKind) -> Vec<String> {
        let mut discovered = BTreeSet::new();
        let mut pending = BTreeSet::from([source.to_string()]);
        while let Some(current) = pending.pop_first() {
            for edge in self
                .edges
                .iter()
                .filter(|edge| edge.source == current && edge.kind == kind)
            {
                if discovered.insert(edge.target.clone()) {
                    pending.insert(edge.target.clone());
                }
            }
        }
        discovered.into_iter().collect()
    }

    #[must_use]
    pub fn computed_dependencies(&self, computed: &str) -> Vec<&IrReactiveEdge> {
        matches!(
            self.nodes.get(computed).map(|node| node.kind),
            Some(IrReactiveNodeKind::Computed)
        )
        .then(|| {
            self.edges
                .iter()
                .filter(|edge| edge.source == computed && edge.kind == IrReactiveEdgeKind::Reads)
                .collect()
        })
        .unwrap_or_default()
    }

    #[must_use]
    pub fn action_dependencies(&self, action: &str) -> Vec<&IrReactiveEdge> {
        matches!(
            self.nodes.get(action).map(|node| node.kind),
            Some(IrReactiveNodeKind::Action)
        )
        .then(|| {
            self.edges
                .iter()
                .filter(|edge| edge.source == action && edge.kind == IrReactiveEdgeKind::Reads)
                .collect()
        })
        .unwrap_or_default()
    }

    #[must_use]
    pub fn invalidations_from(&self, source: &str) -> Vec<&IrReactiveEdge> {
        self.edges
            .iter()
            .filter(|edge| edge.source == source && edge.kind == IrReactiveEdgeKind::Invalidates)
            .collect()
    }

    #[must_use]
    pub fn dependencies_of(&self, target: &str) -> Vec<&IrReactiveEdge> {
        self.edges
            .iter()
            .filter(|edge| edge.target == target)
            .collect()
    }

    #[must_use]
    pub fn dependents_of(&self, source: &str) -> Vec<&IrReactiveEdge> {
        self.edges
            .iter()
            .filter(|edge| edge.source == source)
            .collect()
    }
}

impl IrReactiveTransitiveAnalysis {
    #[must_use]
    pub fn dependencies_of(&self, node: &str) -> &[String] {
        self.dependencies
            .get(node)
            .map(Vec::as_slice)
            .unwrap_or_default()
    }

    #[must_use]
    pub fn dependents_of(&self, node: &str) -> &[String] {
        self.dependents
            .get(node)
            .map(Vec::as_slice)
            .unwrap_or_default()
    }
}

#[must_use]
pub fn inspect_dom_nodes(nodes: Vec<IrDomNode>) -> IrDomInspection {
    IrDomInspection {
        nodes: nodes
            .into_iter()
            .map(|node| (node.id.clone(), node))
            .collect(),
    }
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
                computed_evaluations: Vec::new(),
            });
        lower_component_to_ir(model, component, module);
    }
    IntermediateRepresentation {
        modules: modules.into_values().collect(),
    }
}

fn lower_component_to_ir(
    model: &ApplicationSemanticModel,
    component: &ComponentNode,
    module: &mut IrModule,
) {
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
    for computed in model
        .computed_evaluation_plan
        .evaluation_order
        .iter()
        .filter_map(|id| {
            model
                .computed_values
                .values()
                .find(|computed| computed.id.as_str() == id)
        })
    {
        if computed.owner.entity_id() != Some(&component.id)
            || computed.purity != ComputedPurity::Pure
        {
            continue;
        }
        if let Some((function, evaluation)) = lower_computed_evaluation(model, component, computed)
        {
            module.functions.push(function);
            module.computed_evaluations.push(evaluation);
        }
    }
}

fn lower_computed_evaluation(
    model: &ApplicationSemanticModel,
    component: &ComponentNode,
    computed: &ComputedValue,
) -> Option<(IrFunction, IrComputedEvaluation)> {
    let root = model.expression_graph.root_for(&computed.id)?.clone();
    let entry_block = IrBlockId::entry_for(&computed.id);
    let mut lowering = ComputedIrLowering {
        model,
        component,
        computed,
        entry_block: entry_block.clone(),
        instructions: Vec::new(),
        values: BTreeMap::new(),
    };
    let result = lowering.lower_node(&root)?;
    let function = IrFunction {
        id: computed.id.clone(),
        name: computed.name.clone(),
        provenance: computed.provenance.clone(),
        entry_block: entry_block.clone(),
        blocks: vec![IrBlock {
            id: entry_block,
            provenance: computed.provenance.clone(),
            instructions: lowering.instructions,
        }],
        branch_edges: Vec::new(),
        values: lowering.values,
        loops: Vec::new(),
    };
    let evaluation = IrComputedEvaluation {
        computed: computed.id.clone(),
        function: function.id.clone(),
        result,
        provenance: computed.provenance.clone(),
    };
    Some((function, evaluation))
}

struct ComputedIrLowering<'a> {
    model: &'a ApplicationSemanticModel,
    component: &'a ComponentNode,
    computed: &'a ComputedValue,
    entry_block: IrBlockId,
    instructions: Vec<IrInstruction>,
    values: BTreeMap<IrValueId, IrValue>,
}

impl ComputedIrLowering<'_> {
    fn lower_node(&mut self, id: &SemanticId) -> Option<IrValueId> {
        let node = self.model.expression_graph.node(id)?.clone();
        let kind = match node.kind.clone() {
            ExpressionNodeKind::Literal(value) => IrInstructionKind::Constant {
                value: ir_constant(value),
            },
            ExpressionNodeKind::Boolean(value) => IrInstructionKind::Constant {
                value: IrConstant::Boolean(value),
            },
            ExpressionNodeKind::Identifier(_) => {
                return None;
            }
            ExpressionNodeKind::ThisMember { name } => self.lower_this_member(&name)?,
            ExpressionNodeKind::MemberAccess { object, property } => IrInstructionKind::GetMember {
                object: IrOperand::Value(self.lower_node(&object)?),
                property,
            },
            ExpressionNodeKind::Arithmetic {
                left,
                right,
                operator,
            } => IrInstructionKind::Binary {
                operation: ir_arithmetic_operation(operator),
                left: IrOperand::Value(self.lower_node(&left)?),
                right: IrOperand::Value(self.lower_node(&right)?),
            },
            ExpressionNodeKind::Comparison {
                left,
                right,
                operator,
            } => IrInstructionKind::Binary {
                operation: ir_comparison_operation(operator),
                left: IrOperand::Value(self.lower_node(&left)?),
                right: IrOperand::Value(self.lower_node(&right)?),
            },
            ExpressionNodeKind::Logical {
                left,
                right,
                operator,
            } => IrInstructionKind::Binary {
                operation: ir_logical_operation(operator),
                left: IrOperand::Value(self.lower_node(&left)?),
                right: IrOperand::Value(self.lower_node(&right)?),
            },
            ExpressionNodeKind::NullishCoalescing { left, right } => IrInstructionKind::Binary {
                operation: IrBinaryOperation::NullishCoalesce,
                left: IrOperand::Value(self.lower_node(&left)?),
                right: IrOperand::Value(self.lower_node(&right)?),
            },
            ExpressionNodeKind::Unary { operand, operator } => IrInstructionKind::Unary {
                operation: ir_unary_operation(operator),
                operand: IrOperand::Value(self.lower_node(&operand)?),
            },
        };
        Some(self.emit(node, kind))
    }

    fn lower_this_member(&self, name: &str) -> Option<IrInstructionKind> {
        let target = self
            .component
            .state_fields
            .iter()
            .find(|field| field.name == name)
            .map(|field| field.id.clone())
            .or_else(|| {
                self.model
                    .computed_values
                    .get(&self.component.id.computed(name))
                    .map(|computed| computed.id.clone())
            })?;
        self.model
            .references
            .iter()
            .find(|reference| reference.source == self.computed.id && reference.target == target)?;
        if self
            .component
            .state_fields
            .iter()
            .any(|field| field.id == target)
        {
            Some(IrInstructionKind::LoadStorage {
                storage: IrStorageId::for_semantic_origin(&target),
            })
        } else {
            Some(IrInstructionKind::LoadComputed { computed: target })
        }
    }

    fn emit(&mut self, node: ExpressionNode, kind: IrInstructionKind) -> IrValueId {
        let value = IrValueId::for_function(&self.computed.id, self.values.len());
        let instruction = IrInstructionId::for_block(&self.entry_block, self.instructions.len());
        self.values.insert(
            value.clone(),
            IrValue {
                id: value.clone(),
                definition: IrValueDefinition::Instruction(instruction.clone()),
                semantic_type: self
                    .model
                    .semantic_type_of(&node.id)
                    .cloned()
                    .unwrap_or(SemanticType::Unknown),
                provenance: node.provenance.clone(),
                semantic_origin: Some(node.id.clone()),
            },
        );
        self.instructions.push(IrInstruction {
            id: instruction,
            provenance: node.provenance,
            result: Some(value.clone()),
            semantic_origin: Some(node.id),
            kind,
        });
        value
    }
}

fn ir_constant(value: SerializableValue) -> IrConstant {
    match value {
        SerializableValue::Null => IrConstant::Null,
        SerializableValue::Boolean(value) => IrConstant::Boolean(value),
        SerializableValue::Number(value) => IrConstant::Number(value),
        SerializableValue::String(value) => IrConstant::String(value),
        SerializableValue::Array(value) => IrConstant::Array(value),
        SerializableValue::Object(value) => IrConstant::Object(value),
    }
}

const fn ir_arithmetic_operation(operator: ArithmeticOperator) -> IrBinaryOperation {
    match operator {
        ArithmeticOperator::Add => IrBinaryOperation::Add,
        ArithmeticOperator::Subtract => IrBinaryOperation::Subtract,
        ArithmeticOperator::Multiply => IrBinaryOperation::Multiply,
        ArithmeticOperator::Divide => IrBinaryOperation::Divide,
        ArithmeticOperator::Remainder => IrBinaryOperation::Remainder,
    }
}

const fn ir_comparison_operation(operator: ComparisonOperator) -> IrBinaryOperation {
    match operator {
        ComparisonOperator::Equal => IrBinaryOperation::Equal,
        ComparisonOperator::NotEqual => IrBinaryOperation::NotEqual,
        ComparisonOperator::LessThan => IrBinaryOperation::LessThan,
        ComparisonOperator::LessThanOrEqual => IrBinaryOperation::LessThanOrEqual,
        ComparisonOperator::GreaterThan => IrBinaryOperation::GreaterThan,
        ComparisonOperator::GreaterThanOrEqual => IrBinaryOperation::GreaterThanOrEqual,
    }
}

const fn ir_logical_operation(operator: LogicalOperator) -> IrBinaryOperation {
    match operator {
        LogicalOperator::And => IrBinaryOperation::And,
        LogicalOperator::Or => IrBinaryOperation::Or,
    }
}

const fn ir_unary_operation(operator: UnaryOperator) -> IrUnaryOperation {
    match operator {
        UnaryOperator::Not => IrUnaryOperation::Not,
        UnaryOperator::Plus => IrUnaryOperation::Identity,
        UnaryOperator::Minus => IrUnaryOperation::Negate,
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
    Array(Vec<SerializableValue>),
    Object(BTreeMap<String, SerializableValue>),
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

/// Statically known transient values derived by canonical IR constant propagation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IrConstantPropagationAnalysis {
    pub constants: BTreeMap<IrValueId, IrConstant>,
}

/// Side-effect-free instructions whose produced values have no canonical uses.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IrDeadAssignmentAnalysis {
    pub instructions: Vec<IrInstructionId>,
}

/// A compiler-owned transformation over canonical IR.
pub trait IrOptimizationPass {
    fn name(&self) -> &'static str;
    fn run(&self, input: &IntermediateRepresentation) -> IntermediateRepresentation;
}

/// Ordered owner and executor for canonical IR optimization passes.
#[derive(Default)]
pub struct IrPassManager {
    passes: Vec<Box<dyn IrOptimizationPass>>,
}

impl IrPassManager {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register(&mut self, pass: Box<dyn IrOptimizationPass>) {
        self.passes.push(pass);
    }

    #[must_use]
    pub fn pass_names(&self) -> Vec<&'static str> {
        self.passes.iter().map(|pass| pass.name()).collect()
    }

    #[must_use]
    pub fn run(&self, input: &IntermediateRepresentation) -> IntermediateRepresentation {
        self.passes
            .iter()
            .fold(input.clone(), |current, pass| pass.run(&current))
    }
}

/// An immutable ordered optimization pipeline.
pub struct IrOptimizationPipeline {
    passes: Vec<Box<dyn IrOptimizationPass>>,
}

impl IrOptimizationPipeline {
    #[must_use]
    pub fn new(passes: Vec<Box<dyn IrOptimizationPass>>) -> Self {
        Self { passes }
    }

    #[must_use]
    pub fn pass_names(&self) -> Vec<&'static str> {
        self.passes.iter().map(|pass| pass.name()).collect()
    }

    #[must_use]
    pub fn run(&self, input: &IntermediateRepresentation) -> IntermediateRepresentation {
        self.passes
            .iter()
            .fold(input.clone(), |current, pass| pass.run(&current))
    }

    #[must_use]
    pub fn run_with_report(&self, input: &IntermediateRepresentation) -> IrOptimizationReport {
        let mut current = input.clone();
        let mut passes = Vec::new();
        for pass in &self.passes {
            let before = optimization_metrics(&current);
            current = pass.run(&current);
            passes.push(IrOptimizationPassReport {
                name: pass.name(),
                before,
                after: optimization_metrics(&current),
            });
        }
        IrOptimizationReport {
            output: current,
            passes,
        }
    }
}

/// Build the immutable optimization pipeline applied to canonical computed IR.
#[must_use]
pub fn computed_optimization_pipeline() -> IrOptimizationPipeline {
    IrOptimizationPipeline::new(vec![
        Box::new(IrCommonSubexpressionEliminationPass),
        Box::new(IrCopyPropagationPass),
        Box::new(IrConstantFoldingPass),
        Box::new(IrInstructionSimplificationPass),
        Box::new(IrDeadCodeEliminationPass),
        Box::new(IrCfgCleanupPass),
    ])
}

/// Run the immutable canonical optimization pipeline over computed IR.
#[must_use]
pub fn optimize_computed_ir(input: &IntermediateRepresentation) -> IrOptimizationReport {
    computed_optimization_pipeline().run_with_report(input)
}

/// Compact structural metrics for one canonical IR snapshot.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IrOptimizationMetrics {
    pub blocks: usize,
    pub instructions: usize,
    pub values: usize,
}

/// One pass's observable before/after optimization result.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IrOptimizationPassReport {
    pub name: &'static str,
    pub before: IrOptimizationMetrics,
    pub after: IrOptimizationMetrics,
}

/// The output IR and ordered reports from one immutable pipeline run.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IrOptimizationReport {
    pub output: IntermediateRepresentation,
    pub passes: Vec<IrOptimizationPassReport>,
}

fn optimization_metrics(representation: &IntermediateRepresentation) -> IrOptimizationMetrics {
    IrOptimizationMetrics {
        blocks: representation
            .modules
            .iter()
            .flat_map(|module| &module.functions)
            .map(|function| function.blocks.len())
            .sum(),
        instructions: representation
            .modules
            .iter()
            .flat_map(|module| &module.functions)
            .flat_map(|function| &function.blocks)
            .map(|block| block.instructions.len())
            .sum(),
        values: representation
            .modules
            .iter()
            .flat_map(|module| &module.functions)
            .map(|function| function.values.len())
            .sum(),
    }
}

/// Detects dead result assignments without treating storage effects as removable.
#[must_use]
pub fn analyze_dead_assignments(function: &IrFunction) -> IrDeadAssignmentAnalysis {
    analyze_dead_assignments_preserving(function, &BTreeSet::new())
}

fn analyze_dead_assignments_preserving(
    function: &IrFunction,
    preserved_values: &BTreeSet<IrValueId>,
) -> IrDeadAssignmentAnalysis {
    let uses = analyze_definition_uses(function).uses;
    let instructions = function
        .blocks
        .iter()
        .flat_map(|block| block.instructions.iter())
        .filter(|instruction| {
            instruction.result.as_ref().is_some_and(|result| {
                !preserved_values.contains(result)
                    && uses.get(result).is_some_and(Vec::is_empty)
                    && matches!(
                        instruction.kind,
                        IrInstructionKind::Constant { .. }
                            | IrInstructionKind::LoadStorage { .. }
                            | IrInstructionKind::LoadComputed { .. }
                            | IrInstructionKind::GetMember { .. }
                            | IrInstructionKind::Copy { .. }
                            | IrInstructionKind::Binary { .. }
                            | IrInstructionKind::Unary { .. }
                    )
            })
        })
        .map(|instruction| instruction.id.clone())
        .collect();
    IrDeadAssignmentAnalysis { instructions }
}

/// Propagates inline primitive constants through the current unary and binary IR operations.
#[must_use]
pub fn analyze_constant_propagation(function: &IrFunction) -> IrConstantPropagationAnalysis {
    let mut constants = BTreeMap::new();
    for block in &function.blocks {
        for instruction in &block.instructions {
            let Some(result) = &instruction.result else {
                continue;
            };
            let constant = match &instruction.kind {
                IrInstructionKind::Constant { value } => Some(value.clone()),
                IrInstructionKind::Unary { operation, operand } => {
                    resolve_constant(operand, &constants).and_then(|operand| {
                        match (operation, operand) {
                            (IrUnaryOperation::Not, IrConstant::Boolean(value)) => {
                                Some(IrConstant::Boolean(!value))
                            }
                            (IrUnaryOperation::Identity, value) => Some(value),
                            (IrUnaryOperation::Negate, IrConstant::Number(value)) => {
                                negate_number(&value).map(IrConstant::Number)
                            }
                            _ => None,
                        }
                    })
                }
                IrInstructionKind::Binary {
                    operation,
                    left,
                    right,
                } => {
                    let (Some(IrConstant::Number(left)), Some(IrConstant::Number(right))) = (
                        resolve_constant(left, &constants),
                        resolve_constant(right, &constants),
                    ) else {
                        continue;
                    };
                    evaluate_numeric_binary(*operation, &left, &right).map(IrConstant::Number)
                }
                _ => None,
            };
            if let Some(constant) = constant {
                constants.insert(result.clone(), constant);
            }
        }
    }
    IrConstantPropagationAnalysis { constants }
}

fn resolve_constant(
    operand: &IrOperand,
    constants: &BTreeMap<IrValueId, IrConstant>,
) -> Option<IrConstant> {
    match operand {
        IrOperand::Constant(constant) => Some(constant.clone()),
        IrOperand::Value(value) => constants.get(value).cloned(),
        IrOperand::Storage(_) => None,
    }
}

fn negate_number(value: &str) -> Option<String> {
    value.parse::<f64>().ok().map(|value| format_number(-value))
}
fn evaluate_numeric_binary(
    operation: IrBinaryOperation,
    left: &str,
    right: &str,
) -> Option<String> {
    let left = left.parse::<f64>().ok()?;
    let right = right.parse::<f64>().ok()?;
    let value = match operation {
        IrBinaryOperation::Add => left + right,
        IrBinaryOperation::Subtract => left - right,
        IrBinaryOperation::Multiply => left * right,
        IrBinaryOperation::Divide if right != 0.0 => left / right,
        IrBinaryOperation::Remainder if right != 0.0 => left % right,
        _ => return None,
    };
    Some(format_number(value))
}
fn format_number(value: f64) -> String {
    if value.fract() == 0.0 {
        format!("{value:.0}")
    } else {
        value.to_string()
    }
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
    let computed_evaluation_ids = representation
        .modules
        .iter()
        .flat_map(|module| {
            module
                .computed_evaluations
                .iter()
                .map(|evaluation| evaluation.computed.clone())
        })
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
                &computed_evaluation_ids,
                &mut instruction_ids,
                &mut diagnostics,
            );
        }
        for function in &module.functions {
            validate_function(
                function,
                &storage_ids,
                &computed_evaluation_ids,
                &mut instruction_ids,
                &mut diagnostics,
            );
        }
        validate_computed_evaluations(module, &mut diagnostics);
    }
    diagnostics
}

fn validate_function(
    function: &IrFunction,
    storage_ids: &BTreeSet<IrStorageId>,
    computed_evaluation_ids: &BTreeSet<SemanticId>,
    instruction_ids: &mut BTreeSet<IrInstructionId>,
    diagnostics: &mut Vec<IrValidationDiagnostic>,
) {
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
                storage_ids,
                computed_evaluation_ids,
                instruction_ids,
                diagnostics,
            );
        }
    }
    for (id, value) in &function.values {
        validate_value(function, id, value, &function_instruction_ids, diagnostics);
    }
}

fn validate_value(
    function: &IrFunction,
    id: &IrValueId,
    value: &IrValue,
    instruction_ids: &BTreeSet<IrInstructionId>,
    diagnostics: &mut Vec<IrValidationDiagnostic>,
) {
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
        IrValueDefinition::Instruction(instruction) if !instruction_ids.contains(instruction) => {
            diagnostics.push(IrValidationDiagnostic {
                code: "EZIR1002",
                message: format!(
                    "value {id} references missing defining instruction {instruction}"
                ),
            });
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
        IrValueDefinition::BlockParameter { block, .. } if function.block(block).is_none() => {
            diagnostics.push(IrValidationDiagnostic {
                code: "EZIR1004",
                message: format!("value {id} references missing block parameter owner {block}"),
            });
        }
        _ => {}
    }
}

fn validate_computed_evaluations(module: &IrModule, diagnostics: &mut Vec<IrValidationDiagnostic>) {
    for evaluation in &module.computed_evaluations {
        if evaluation.computed != evaluation.function {
            diagnostics.push(IrValidationDiagnostic {
                code: "EZIR1010",
                message: format!(
                    "computed evaluation {} must use its computed ID as function {}",
                    evaluation.computed, evaluation.function
                ),
            });
            continue;
        }
        let Some(function) = module
            .functions
            .iter()
            .find(|function| function.id == evaluation.function)
        else {
            diagnostics.push(IrValidationDiagnostic {
                code: "EZIR1010",
                message: format!(
                    "computed evaluation {} references missing function {}",
                    evaluation.computed, evaluation.function
                ),
            });
            continue;
        };
        if !function.values.contains_key(&evaluation.result) {
            diagnostics.push(IrValidationDiagnostic {
                code: "EZIR1011",
                message: format!(
                    "computed evaluation {} references missing result {}",
                    evaluation.computed, evaluation.result
                ),
            });
        }
    }
}

fn validate_instruction(
    instruction: &IrInstruction,
    function: Option<&IrFunction>,
    function_instruction_ids: &BTreeSet<IrInstructionId>,
    storage_ids: &BTreeSet<IrStorageId>,
    computed_evaluation_ids: &BTreeSet<SemanticId>,
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
    if let IrInstructionKind::LoadComputed { computed } = &instruction.kind {
        if !computed_evaluation_ids.contains(computed) {
            diagnostics.push(IrValidationDiagnostic {
                code: "EZIR1012",
                message: format!(
                    "instruction {} references unknown computed evaluation {computed}",
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
        | IrInstructionKind::Unary { operand: value, .. }
        | IrInstructionKind::Copy { source: value }
        | IrInstructionKind::GetMember { object: value, .. } => vec![value],
        IrInstructionKind::Binary { left, right, .. } => vec![left, right],
        IrInstructionKind::Nop
        | IrInstructionKind::Constant { .. }
        | IrInstructionKind::InitializeStorage { .. }
        | IrInstructionKind::LoadStorage { .. }
        | IrInstructionKind::LoadComputed { .. } => Vec::new(),
    }
}

fn instruction_storages(kind: &IrInstructionKind) -> Vec<&IrStorageId> {
    match kind {
        IrInstructionKind::InitializeStorage { storage }
        | IrInstructionKind::LoadStorage { storage }
        | IrInstructionKind::StoreStorage { storage, .. } => vec![storage],
        IrInstructionKind::Nop
        | IrInstructionKind::Constant { .. }
        | IrInstructionKind::Copy { .. }
        | IrInstructionKind::LoadComputed { .. }
        | IrInstructionKind::GetMember { .. }
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
    Constant {
        value: IrConstant,
    },
    Copy {
        source: IrOperand,
    },
    InitializeStorage {
        storage: IrStorageId,
    },
    LoadStorage {
        storage: IrStorageId,
    },
    LoadComputed {
        computed: SemanticId,
    },
    GetMember {
        object: IrOperand,
        property: String,
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

/// Immutable copy-propagation pass for canonical IR operands.
pub struct IrCopyPropagationPass;

impl IrOptimizationPass for IrCopyPropagationPass {
    fn name(&self) -> &'static str {
        "copy-propagation"
    }

    fn run(&self, input: &IntermediateRepresentation) -> IntermediateRepresentation {
        let mut output = input.clone();
        for module in &mut output.modules {
            for function in &mut module.functions {
                let mut copies = BTreeMap::new();
                for block in &mut function.blocks {
                    for instruction in &mut block.instructions {
                        replace_copy_operands(&mut instruction.kind, &copies);
                        if let (Some(result), IrInstructionKind::Copy { source }) =
                            (&instruction.result, &instruction.kind)
                        {
                            copies.insert(result.clone(), source.clone());
                        }
                    }
                }
            }
        }
        output
    }
}

/// Immutable common-subexpression-elimination pass for current pure expressions.
pub struct IrCommonSubexpressionEliminationPass;

impl IrOptimizationPass for IrCommonSubexpressionEliminationPass {
    fn name(&self) -> &'static str {
        "common-subexpression-elimination"
    }

    fn run(&self, input: &IntermediateRepresentation) -> IntermediateRepresentation {
        let mut output = input.clone();
        for module in &mut output.modules {
            for function in &mut module.functions {
                let mut expressions = BTreeMap::<String, IrValueId>::new();
                for block in &mut function.blocks {
                    for instruction in &mut block.instructions {
                        let Some(result) = instruction.result.clone() else {
                            continue;
                        };
                        let key = match &instruction.kind {
                            IrInstructionKind::Unary { .. } | IrInstructionKind::Binary { .. } => {
                                format!("{:?}", instruction.kind)
                            }
                            _ => continue,
                        };
                        if let Some(existing) = expressions.get(&key) {
                            instruction.kind = IrInstructionKind::Copy {
                                source: IrOperand::Value(existing.clone()),
                            };
                        } else {
                            expressions.insert(key, result);
                        }
                    }
                }
            }
        }
        output
    }
}

/// Immutable instruction-simplification pass for current canonical IR forms.
pub struct IrInstructionSimplificationPass;

impl IrOptimizationPass for IrInstructionSimplificationPass {
    fn name(&self) -> &'static str {
        "instruction-simplification"
    }

    fn run(&self, input: &IntermediateRepresentation) -> IntermediateRepresentation {
        let mut output = input.clone();
        for module in &mut output.modules {
            for function in &mut module.functions {
                let constants = analyze_constant_propagation(function).constants;
                for block in &mut function.blocks {
                    for instruction in &mut block.instructions {
                        if let IrInstructionKind::Copy { source } = &instruction.kind {
                            if let Some(value) = resolve_constant(source, &constants) {
                                instruction.kind = IrInstructionKind::Constant { value };
                            }
                        }
                    }
                }
            }
        }
        output
    }
}

/// Immutable cleanup pass that removes unreachable canonical CFG artifacts.
pub struct IrCfgCleanupPass;

impl IrOptimizationPass for IrCfgCleanupPass {
    fn name(&self) -> &'static str {
        "cfg-cleanup"
    }

    fn run(&self, input: &IntermediateRepresentation) -> IntermediateRepresentation {
        let mut output = input.clone();
        for module in &mut output.modules {
            for function in &mut module.functions {
                let reachable = analyze_reachability(function)
                    .reachable
                    .into_iter()
                    .collect::<BTreeSet<_>>();
                function
                    .blocks
                    .retain(|block| reachable.contains(&block.id));
                function
                    .branch_edges
                    .retain(|edge| reachable.contains(&edge.from) && reachable.contains(&edge.to));
                function
                    .loops
                    .retain(|loop_region| reachable.contains(&loop_region.header));
                let instructions = function
                    .blocks
                    .iter()
                    .flat_map(|block| {
                        block
                            .instructions
                            .iter()
                            .map(|instruction| instruction.id.clone())
                    })
                    .collect::<BTreeSet<_>>();
                function.values.retain(|_, value| !matches!(&value.definition, IrValueDefinition::Instruction(instruction) if !instructions.contains(instruction)));
            }
        }
        output
    }
}

fn replace_copy_operands(kind: &mut IrInstructionKind, copies: &BTreeMap<IrValueId, IrOperand>) {
    let resolve = |operand: &mut IrOperand| {
        while let IrOperand::Value(value) = operand {
            let Some(replacement) = copies.get(value) else {
                break;
            };
            *operand = replacement.clone();
        }
    };
    match kind {
        IrInstructionKind::Copy { source }
        | IrInstructionKind::StoreStorage { value: source, .. }
        | IrInstructionKind::Unary {
            operand: source, ..
        }
        | IrInstructionKind::GetMember { object: source, .. } => resolve(source),
        IrInstructionKind::Binary { left, right, .. } => {
            resolve(left);
            resolve(right);
        }
        _ => {}
    }
}

/// Immutable primitive constant-folding pass for canonical IR.
pub struct IrConstantFoldingPass;

impl IrOptimizationPass for IrConstantFoldingPass {
    fn name(&self) -> &'static str {
        "constant-folding"
    }

    fn run(&self, input: &IntermediateRepresentation) -> IntermediateRepresentation {
        let mut output = input.clone();
        for module in &mut output.modules {
            for function in &mut module.functions {
                let constants = analyze_constant_propagation(function).constants;
                for block in &mut function.blocks {
                    for instruction in &mut block.instructions {
                        if let Some(result) = &instruction.result {
                            if let Some(value) = constants.get(result) {
                                instruction.kind = IrInstructionKind::Constant {
                                    value: value.clone(),
                                };
                            }
                        }
                    }
                }
            }
        }
        output
    }
}

/// Immutable dead-code-elimination pass for unused pure instruction results.
pub struct IrDeadCodeEliminationPass;

impl IrOptimizationPass for IrDeadCodeEliminationPass {
    fn name(&self) -> &'static str {
        "dead-code-elimination"
    }

    fn run(&self, input: &IntermediateRepresentation) -> IntermediateRepresentation {
        let mut output = input.clone();
        for module in &mut output.modules {
            for function in &mut module.functions {
                let preserved_values = module
                    .computed_evaluations
                    .iter()
                    .filter(|evaluation| evaluation.function == function.id)
                    .map(|evaluation| evaluation.result.clone())
                    .collect::<BTreeSet<_>>();
                loop {
                    let dead = analyze_dead_assignments_preserving(function, &preserved_values)
                        .instructions
                        .into_iter()
                        .collect::<BTreeSet<_>>();
                    if dead.is_empty() {
                        break;
                    }
                    for block in &mut function.blocks {
                        block
                            .instructions
                            .retain(|instruction| !dead.contains(&instruction.id));
                    }
                    function.values.retain(|_, value| {
                        !matches!(
                            &value.definition,
                            IrValueDefinition::Instruction(instruction) if dead.contains(instruction)
                        )
                    });
                }
            }
        }
        output
    }
}

/// A binary operation with value-producing IR semantics.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IrBinaryOperation {
    Add,
    Subtract,
    Multiply,
    Divide,
    Remainder,
    Equal,
    NotEqual,
    LessThan,
    LessThanOrEqual,
    GreaterThan,
    GreaterThanOrEqual,
    And,
    Or,
    NullishCoalesce,
}

/// A unary operation with value-producing IR semantics.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IrUnaryOperation {
    Not,
    Identity,
    Negate,
}

#[cfg(test)]
mod tests {
    use super::{
        compute_dominators, compute_post_dominators, lower_components_to_ir, optimize_computed_ir,
        validate_intermediate_representation, IntermediateRepresentation, IrBinaryOperation,
        IrBlock, IrBlockId, IrBranchArm, IrBranchEdge, IrConstant, IrFunction, IrInstruction,
        IrInstructionId, IrInstructionKind, IrLoop, IrLoopId, IrModule, IrOperand, IrStorageId,
        IrUnaryOperation, IrValue, IrValueDefinition, IrValueId,
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
                computed_evaluations: Vec::new(),
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

    fn computed_ir_fixture() -> (
        IntermediateRepresentation,
        SemanticId,
        SemanticId,
        SemanticId,
    ) {
        let parsed = ezc_parser::parse_file(
            "src/ComputedIr.tsx",
            r#"
@component("x-computed-ir")
class ComputedIr extends Component {
  count = state(1);
  profile = state({ hidden: false });

  @computed()
  get doubled() { return this.count * 2; }

  @computed()
  get visible() { return ((this.doubled >= 2 && !this.profile.hidden) ?? false) || true; }
}
"#,
        );
        let model = crate::build_application_semantic_model(&parsed);
        let component = &model.components[0];
        let count = component.id.state_field("count");
        let doubled = component.id.computed("doubled");
        let visible = component.id.computed("visible");
        (lower_components_to_ir(&model), count, doubled, visible)
    }

    #[test]
    fn lowers_planned_computed_evaluations_into_canonical_ir_functions() {
        let (ir, count, doubled, visible) = computed_ir_fixture();
        let module = &ir.modules[0];
        let doubled_function = module
            .functions
            .iter()
            .find(|function| function.id == doubled)
            .expect("computed doubled function");
        let visible_function = module
            .functions
            .iter()
            .find(|function| function.id == visible)
            .expect("computed visible function");

        assert_eq!(module.computed_evaluations.len(), 2);
        assert_eq!(doubled_function.name, "doubled");
        assert!(doubled_function.blocks[0]
            .instructions
            .iter()
            .any(|instruction| {
                matches!(
                    instruction.kind,
                    IrInstructionKind::LoadStorage { ref storage }
                        if storage == &IrStorageId::for_semantic_origin(&count)
                )
            }));
        assert!(doubled_function.blocks[0]
            .instructions
            .iter()
            .any(|instruction| {
                matches!(
                    instruction.kind,
                    IrInstructionKind::Binary {
                        operation: IrBinaryOperation::Multiply,
                        ..
                    }
                )
            }));
        assert!(visible_function.blocks[0]
            .instructions
            .iter()
            .any(|instruction| {
                matches!(
                    instruction.kind,
                    IrInstructionKind::LoadComputed { ref computed } if computed == &doubled
                )
            }));
        assert!(visible_function.blocks[0]
            .instructions
            .iter()
            .any(|instruction| {
                matches!(instruction.kind, IrInstructionKind::GetMember { .. })
            }));
        for operation in [
            IrBinaryOperation::GreaterThanOrEqual,
            IrBinaryOperation::And,
            IrBinaryOperation::NullishCoalesce,
            IrBinaryOperation::Or,
        ] {
            assert!(visible_function.blocks[0]
                .instructions
                .iter()
                .any(|instruction| {
                    matches!(
                        instruction.kind,
                        IrInstructionKind::Binary {
                            operation: candidate,
                            ..
                        } if candidate == operation
                    )
                }));
        }
        assert!(visible_function.blocks[0]
            .instructions
            .iter()
            .any(|instruction| {
                matches!(
                    instruction.kind,
                    IrInstructionKind::Unary {
                        operation: IrUnaryOperation::Not,
                        ..
                    }
                )
            }));
        assert!(module.computed_evaluations.iter().all(|evaluation| {
            module
                .functions
                .iter()
                .find(|function| function.id == evaluation.function)
                .is_some_and(|function| function.values.contains_key(&evaluation.result))
        }));
        assert!(validate_intermediate_representation(&ir).is_empty());
    }

    #[test]
    fn optimizes_computed_ir_immutably_and_preserves_evaluation_results() {
        let parsed = ezc_parser::parse_file(
            "src/OptimizedComputedIr.tsx",
            r#"
@component("x-optimized-computed-ir")
class OptimizedComputedIr extends Component {
  @computed()
  get total() { return 1 + 2; }
}
"#,
        );
        let model = crate::build_application_semantic_model(&parsed);
        let total = model.components[0].id.computed("total");
        let ir = lower_components_to_ir(&model);
        let original = ir.modules[0]
            .functions
            .iter()
            .find(|function| function.id == total)
            .expect("original computed function");
        assert_eq!(original.blocks[0].instructions.len(), 3);
        assert!(original.blocks[0].instructions.iter().any(|instruction| {
            matches!(
                instruction.kind,
                IrInstructionKind::Binary {
                    operation: IrBinaryOperation::Add,
                    ..
                }
            )
        }));

        let report = optimize_computed_ir(&ir);
        let optimized_module = &report.output.modules[0];
        let evaluation = optimized_module
            .computed_evaluations
            .iter()
            .find(|evaluation| evaluation.computed == total)
            .expect("computed evaluation");
        let optimized = optimized_module
            .functions
            .iter()
            .find(|function| function.id == total)
            .expect("optimized computed function");

        assert_eq!(
            report
                .passes
                .iter()
                .map(|pass| pass.name)
                .collect::<Vec<_>>(),
            vec![
                "common-subexpression-elimination",
                "copy-propagation",
                "constant-folding",
                "instruction-simplification",
                "dead-code-elimination",
                "cfg-cleanup",
            ]
        );
        assert_eq!(optimized.blocks[0].instructions.len(), 1);
        assert!(matches!(
            optimized.blocks[0].instructions[0].kind,
            IrInstructionKind::Constant {
                value: IrConstant::Number(ref value)
            } if value == "3"
        ));
        assert_eq!(optimized.values.len(), 1);
        assert!(optimized.values.contains_key(&evaluation.result));
        assert!(validate_intermediate_representation(&ir).is_empty());
        assert!(validate_intermediate_representation(&report.output).is_empty());
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
