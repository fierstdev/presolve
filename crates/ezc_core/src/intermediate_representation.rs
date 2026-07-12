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
        lower_components_to_ir, IntermediateRepresentation, IrBlock, IrBlockId, IrBranchArm,
        IrBranchEdge, IrFunction, IrInstruction, IrInstructionKind, IrLoop, IrLoopId, IrModule,
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
