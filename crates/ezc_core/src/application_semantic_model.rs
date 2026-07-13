use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use ezc_parser::ParsedFile;

use crate::compilation_unit::CompilationUnit;
use crate::component_graph::{
    build_component_graph_for_module, render_event_handlers, ComponentAction, ComponentDiagnostic,
    ComponentMethod, ComponentNode, MethodLocalVariable, RenderEventHandler, StateField,
};
use crate::computed_value::{collect_computed_values, ComputedValue};
use crate::expression_graph::{ExpressionGraph, ExpressionNode};
use crate::semantic_id::{SemanticId, SemanticOwner};
use crate::semantic_provenance::SourceProvenance;
use crate::semantic_reference::{SemanticReference, SemanticReferenceKind};
use crate::semantic_type::SemanticTypeModel;
use crate::template_graph::{build_template_graph, TemplateNode};
use crate::template_semantics::{
    build_template_semantic_entities, TemplateSemanticEntity, TemplateSemanticKind,
    TemplateSemanticScope,
};

/// Application-level semantic data assembled from the compiler's existing graphs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApplicationSemanticModel {
    pub expression_graph: ExpressionGraph,
    pub semantic_types: SemanticTypeModel,
    pub components: Vec<ComponentNode>,
    pub computed_values: BTreeMap<SemanticId, ComputedValue>,
    pub templates: Vec<TemplateNode>,
    pub template_entities: Vec<TemplateSemanticEntity>,
    pub diagnostics: Vec<ComponentDiagnostic>,
    pub ownership: BTreeMap<SemanticId, SemanticOwner>,
    pub references: Vec<SemanticReference>,
    pub provenance: BTreeMap<SemanticId, SourceProvenance>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SemanticEntity<'a> {
    Component(&'a ComponentNode),
    StateField(&'a StateField),
    Method(&'a ComponentMethod),
    Computed(&'a ComputedValue),
    Parameter(&'a crate::MethodParameter),
    LocalVariable(&'a MethodLocalVariable),
    Action(&'a ComponentAction),
    EventHandler(&'a RenderEventHandler),
    Template(&'a TemplateNode),
    TemplateEntity(&'a TemplateSemanticEntity),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SemanticEntityKind {
    Component,
    StateField,
    Method,
    Computed,
    Parameter,
    LocalVariable,
    Action,
    EventHandler,
    Template,
    TemplateEntity,
}

impl SemanticEntity<'_> {
    #[must_use]
    pub fn kind(self) -> SemanticEntityKind {
        match self {
            Self::Component(_) => SemanticEntityKind::Component,
            Self::StateField(_) => SemanticEntityKind::StateField,
            Self::Method(_) => SemanticEntityKind::Method,
            Self::Computed(_) => SemanticEntityKind::Computed,
            Self::Parameter(_) => SemanticEntityKind::Parameter,
            Self::LocalVariable(_) => SemanticEntityKind::LocalVariable,
            Self::Action(_) => SemanticEntityKind::Action,
            Self::EventHandler(_) => SemanticEntityKind::EventHandler,
            Self::Template(_) => SemanticEntityKind::Template,
            Self::TemplateEntity(_) => SemanticEntityKind::TemplateEntity,
        }
    }
}

impl ApplicationSemanticModel {
    #[must_use]
    pub fn entity(&self, id: &SemanticId) -> Option<SemanticEntity<'_>> {
        for component in &self.components {
            if component.id == *id {
                return Some(SemanticEntity::Component(component));
            }
            if let Some(field) = component.state_fields.iter().find(|field| field.id == *id) {
                return Some(SemanticEntity::StateField(field));
            }
            if let Some(method) = component.methods.iter().find(|method| method.id == *id) {
                return Some(SemanticEntity::Method(method));
            }
            if let Some(parameter) = component
                .methods
                .iter()
                .flat_map(|method| method.parameters.iter())
                .find(|parameter| parameter.id == *id)
            {
                return Some(SemanticEntity::Parameter(parameter));
            }
            if let Some(local) = component
                .methods
                .iter()
                .flat_map(|method| method.local_variables.iter())
                .find(|local| local.id == *id)
            {
                return Some(SemanticEntity::LocalVariable(local));
            }
            if let Some(action) = component.actions.iter().find(|action| action.id == *id) {
                return Some(SemanticEntity::Action(action));
            }
            if let Some(render) = &component.render {
                if let Some(handler) = render_event_handlers(render)
                    .into_iter()
                    .find(|handler| handler.id == *id)
                {
                    return Some(SemanticEntity::EventHandler(handler));
                }
            }
        }

        if let Some(computed) = self.computed_values.get(id) {
            return Some(SemanticEntity::Computed(computed));
        }

        if let Some(template) = self.templates.iter().find(|template| template.id == *id) {
            return Some(SemanticEntity::Template(template));
        }

        self.template_entities
            .iter()
            .find(|entity| entity.id == *id)
            .map(SemanticEntity::TemplateEntity)
    }

    #[must_use]
    pub fn component(&self, id: &SemanticId) -> Option<&ComponentNode> {
        self.components.iter().find(|component| component.id == *id)
    }

    #[must_use]
    pub fn template(&self, id: &SemanticId) -> Option<&TemplateNode> {
        self.templates.iter().find(|template| template.id == *id)
    }

    #[must_use]
    pub fn computed_value(&self, id: &SemanticId) -> Option<&ComputedValue> {
        self.computed_values.get(id)
    }

    #[must_use]
    pub fn template_entity(&self, id: &SemanticId) -> Option<&TemplateSemanticEntity> {
        self.template_entities
            .iter()
            .find(|entity| entity.id == *id)
    }

    #[must_use]
    pub fn template_entities_for(&self, template: &SemanticId) -> Vec<&TemplateSemanticEntity> {
        self.children_of(template)
            .into_iter()
            .filter_map(|id| self.template_entity(id))
            .collect()
    }

    #[must_use]
    pub fn owner(&self, id: &SemanticId) -> Option<&SemanticOwner> {
        self.ownership.get(id)
    }

    #[must_use]
    pub fn parent_of(&self, id: &SemanticId) -> Option<&SemanticId> {
        self.owner(id).and_then(SemanticOwner::entity_id)
    }

    #[must_use]
    pub fn ancestors_of(&self, id: &SemanticId) -> Vec<&SemanticId> {
        let mut ancestors = Vec::new();
        let mut seen = BTreeSet::from([id.clone()]);
        let mut current = id;

        while let Some(parent) = self.parent_of(current) {
            if !seen.insert(parent.clone()) {
                break;
            }
            ancestors.push(parent);
            current = parent;
        }

        ancestors
    }

    #[must_use]
    pub fn provenance(&self, id: &SemanticId) -> Option<&SourceProvenance> {
        self.provenance.get(id)
    }

    #[must_use]
    pub fn expression(&self, id: &SemanticId) -> Option<&ExpressionNode> {
        self.expression_graph.node(id)
    }

    #[must_use]
    pub fn expression_root(&self, owner: &SemanticId) -> Option<&SemanticId> {
        self.expression_graph.root_for(owner)
    }

    #[must_use]
    pub fn expressions_for(&self, owner: &SemanticId) -> Vec<&ExpressionNode> {
        self.expression_graph.nodes_for(owner)
    }

    #[must_use]
    pub fn expression_dependencies(&self, id: &SemanticId) -> Vec<&SemanticId> {
        self.expression_graph.dependencies_of(id)
    }

    #[must_use]
    pub fn expression_dependents(&self, id: &SemanticId) -> Vec<&ExpressionNode> {
        self.expression_graph.dependents_of(id)
    }

    #[must_use]
    pub fn expression_owner(&self, id: &SemanticId) -> Option<&SemanticId> {
        self.expression_graph.owner_of(id)
    }

    #[must_use]
    pub fn expression_provenance(&self, id: &SemanticId) -> Option<&SourceProvenance> {
        self.expression_graph.provenance_of(id)
    }

    #[must_use]
    pub fn semantic_type_of(&self, id: &SemanticId) -> Option<&crate::SemanticType> {
        self.semantic_types
            .assignments
            .get(id)
            .map(|assignment| &assignment.semantic_type)
    }

    #[must_use]
    pub fn expression_type(&self, id: &SemanticId) -> Option<&crate::SemanticType> {
        self.expression_graph
            .node(id)
            .and_then(|_| self.semantic_type_of(id))
    }

    #[must_use]
    pub fn type_declarations(
        &self,
        semantic_type: &crate::SemanticType,
    ) -> Vec<&crate::SemanticTypeAssignment> {
        self.semantic_types
            .assignments
            .values()
            .filter(|assignment| {
                assignment.status == crate::SemanticTypeStatus::Declared
                    && assignment.semantic_type == *semantic_type
            })
            .collect()
    }

    #[must_use]
    pub fn type_usages(
        &self,
        semantic_type: &crate::SemanticType,
    ) -> Vec<&crate::SemanticTypeAssignment> {
        self.semantic_types
            .assignments
            .values()
            .filter(|assignment| assignment.semantic_type == *semantic_type)
            .collect()
    }

    #[must_use]
    pub fn serialization_compatibility_of(
        &self,
        id: &SemanticId,
    ) -> Option<crate::SerializationCompatibility> {
        self.semantic_type_of(id)
            .map(crate::serialization_compatibility)
    }

    #[must_use]
    pub fn is_type_assignable(&self, source: &SemanticId, target: &SemanticId) -> Option<bool> {
        Some(crate::is_assignable(
            self.semantic_type_of(source)?,
            self.semantic_type_of(target)?,
        ))
    }

    #[must_use]
    pub fn expressions_in_file(&self, path: &Path) -> Vec<&ExpressionNode> {
        self.expression_graph.nodes_in_file(path)
    }

    #[must_use]
    pub fn expressions_at(&self, path: &Path, offset: usize) -> Vec<&ExpressionNode> {
        self.expression_graph.nodes_at(path, offset)
    }

    #[must_use]
    pub fn application_roots(&self) -> Vec<&SemanticId> {
        self.ownership
            .iter()
            .filter_map(|(id, owner)| matches!(owner, SemanticOwner::Application).then_some(id))
            .collect()
    }

    #[must_use]
    pub fn children_of(&self, owner: &SemanticId) -> Vec<&SemanticId> {
        self.ownership
            .iter()
            .filter_map(|(id, entity_owner)| {
                (entity_owner.entity_id() == Some(owner)).then_some(id)
            })
            .collect()
    }

    #[must_use]
    pub fn descendants_of(&self, owner: &SemanticId) -> Vec<&SemanticId> {
        let mut descendants = Vec::new();
        self.collect_descendants(owner, &mut descendants);
        descendants
    }

    #[must_use]
    pub fn entities_of_kind(&self, kind: SemanticEntityKind) -> Vec<&SemanticId> {
        self.ownership
            .keys()
            .filter(|id| self.entity(id).is_some_and(|entity| entity.kind() == kind))
            .collect()
    }

    #[must_use]
    pub fn entities_in_file(&self, path: &Path) -> Vec<&SemanticId> {
        self.provenance
            .iter()
            .filter_map(|(id, provenance)| (provenance.path == path).then_some(id))
            .collect()
    }

    #[must_use]
    pub fn entities_at(&self, path: &Path, offset: usize) -> Vec<&SemanticId> {
        self.provenance
            .iter()
            .filter_map(|(id, provenance)| {
                (provenance.path == path
                    && provenance.span.start <= offset
                    && offset < provenance.span.end)
                    .then_some(id)
            })
            .collect()
    }

    #[must_use]
    pub fn references_of_kind(&self, kind: SemanticReferenceKind) -> Vec<&SemanticReference> {
        let mut references = self
            .references
            .iter()
            .filter(|reference| reference.kind == kind)
            .collect::<Vec<_>>();
        references.sort_by(|left, right| {
            (left.source.as_str(), left.target.as_str())
                .cmp(&(right.source.as_str(), right.target.as_str()))
        });
        references
    }

    #[must_use]
    pub fn references_in_file(&self, path: &Path) -> Vec<&SemanticReference> {
        let mut references = self
            .references
            .iter()
            .filter(|reference| reference.provenance.path == path)
            .collect::<Vec<_>>();
        references.sort_by(|left, right| {
            (left.source.as_str(), left.target.as_str())
                .cmp(&(right.source.as_str(), right.target.as_str()))
        });
        references
    }

    #[must_use]
    pub fn references_at(&self, path: &Path, offset: usize) -> Vec<&SemanticReference> {
        let mut references = self
            .references
            .iter()
            .filter(|reference| {
                reference.provenance.path == path
                    && reference.provenance.span.start <= offset
                    && offset < reference.provenance.span.end
            })
            .collect::<Vec<_>>();
        references.sort_by(|left, right| {
            (left.source.as_str(), left.target.as_str())
                .cmp(&(right.source.as_str(), right.target.as_str()))
        });
        references
    }

    fn collect_descendants<'a>(
        &'a self,
        owner: &SemanticId,
        descendants: &mut Vec<&'a SemanticId>,
    ) {
        for child in self.children_of(owner) {
            descendants.push(child);
            self.collect_descendants(child, descendants);
        }
    }

    #[must_use]
    pub fn references_from(&self, id: &SemanticId) -> Vec<&SemanticReference> {
        self.references
            .iter()
            .filter(move |reference| reference.source == *id)
            .collect()
    }

    #[must_use]
    pub fn references_to(&self, id: &SemanticId) -> Vec<&SemanticReference> {
        self.references
            .iter()
            .filter(move |reference| reference.target == *id)
            .collect()
    }
}

#[must_use]
pub fn build_application_semantic_model(parsed: &ParsedFile) -> ApplicationSemanticModel {
    build_application_semantic_model_from_files(std::slice::from_ref(parsed))
}

/// Assemble canonical ASM from an existing component graph while preserving its identity mode.
#[must_use]
pub fn build_application_semantic_model_from_component_graph(
    component_graph: &crate::component_graph::ComponentGraph,
) -> ApplicationSemanticModel {
    let templates = build_template_graph(component_graph).templates;
    let template_entities = build_template_semantic_entities(&templates);
    let computed_values =
        collect_computed_values(&component_graph.components, &component_graph.provenance);
    let mut provenance = component_graph.provenance.clone();
    provenance.extend(
        computed_values
            .iter()
            .map(|(id, computed)| (id.clone(), computed.provenance.clone())),
    );
    let ownership = collect_ownership(
        &component_graph.components,
        &computed_values,
        &templates,
        &template_entities,
    );
    let expression_graph =
        ExpressionGraph::from_components(&component_graph.components, &component_graph.provenance);
    let mut references = component_graph.references.clone();
    references.extend(build_computed_references(
        &component_graph.components,
        &computed_values,
        &expression_graph,
    ));
    references.extend(build_template_state_references(
        &component_graph.components,
        &template_entities,
        &ownership,
    ));
    references.extend(build_template_event_references(
        &component_graph.components,
        &template_entities,
        &ownership,
    ));
    references.extend(build_template_local_references(
        &component_graph.components,
        &template_entities,
        &ownership,
    ));

    let semantic_types =
        SemanticTypeModel::from_components(&component_graph.components, &provenance)
            .with_expression_types(&expression_graph, &component_graph.components)
            .with_computed_value_types(
                &component_graph.components,
                &computed_values,
                &expression_graph,
                &references,
            )
            .with_template_binding_types(&template_entities, &references)
            .normalized();

    ApplicationSemanticModel {
        expression_graph,
        semantic_types,
        components: component_graph.components.clone(),
        computed_values,
        templates,
        template_entities,
        diagnostics: component_graph.diagnostics.clone(),
        ownership,
        references,
        provenance,
    }
}

#[must_use]
pub fn build_application_semantic_model_for_unit(
    unit: &CompilationUnit,
) -> ApplicationSemanticModel {
    let symbols = crate::build_symbol_table(unit);
    let modules = crate::build_module_graph(unit);
    let bindings = crate::build_binding_table(unit, &symbols, &modules);
    build_application_semantic_model_from_files_with_bindings(unit.files(), Some(&bindings))
}

fn build_application_semantic_model_from_files(files: &[ParsedFile]) -> ApplicationSemanticModel {
    build_application_semantic_model_from_files_with_bindings(files, None)
}

fn build_application_semantic_model_from_files_with_bindings(
    files: &[ParsedFile],
    bindings: Option<&crate::BindingTable>,
) -> ApplicationSemanticModel {
    let mut components = Vec::new();
    let mut templates = Vec::new();
    let mut diagnostics = Vec::new();
    let mut references = Vec::new();
    let mut provenance = BTreeMap::new();
    let mut template_entities = Vec::new();
    let mut type_aliases = Vec::new();

    for parsed in files {
        let component_graph = build_component_graph_for_module(parsed);
        let template_graph = build_template_graph(&component_graph);
        let file_template_entities = build_template_semantic_entities(&template_graph.templates);

        components.extend(component_graph.components);
        templates.extend(template_graph.templates);
        diagnostics.extend(component_graph.diagnostics);
        references.extend(component_graph.references);
        provenance.extend(component_graph.provenance);
        provenance.extend(
            file_template_entities
                .iter()
                .map(|entity| (entity.id.clone(), entity.provenance.clone())),
        );
        template_entities.extend(file_template_entities);
        type_aliases.extend(
            parsed
                .type_aliases
                .iter()
                .cloned()
                .map(|alias| (parsed.path.clone(), alias)),
        );
    }

    let computed_values = collect_computed_values(&components, &provenance);
    provenance.extend(
        computed_values
            .iter()
            .map(|(id, computed)| (id.clone(), computed.provenance.clone())),
    );
    let ownership = collect_ownership(
        &components,
        &computed_values,
        &templates,
        &template_entities,
    );
    let expression_graph = ExpressionGraph::from_components(&components, &provenance);

    references.extend(build_computed_references(
        &components,
        &computed_values,
        &expression_graph,
    ));
    references.extend(build_template_state_references(
        &components,
        &template_entities,
        &ownership,
    ));
    references.extend(build_template_event_references(
        &components,
        &template_entities,
        &ownership,
    ));
    references.extend(build_template_local_references(
        &components,
        &template_entities,
        &ownership,
    ));

    let semantic_types = SemanticTypeModel::from_components_with_aliases_and_bindings(
        &components,
        &provenance,
        &type_aliases,
        bindings,
    )
    .with_expression_types(&expression_graph, &components)
    .with_computed_value_types(
        &components,
        &computed_values,
        &expression_graph,
        &references,
    )
    .with_template_binding_types(&template_entities, &references)
    .normalized();

    ApplicationSemanticModel {
        expression_graph,
        semantic_types,
        components,
        computed_values,
        templates,
        template_entities,
        diagnostics,
        ownership,
        references,
        provenance,
    }
}

fn build_computed_references(
    components: &[ComponentNode],
    computed_values: &BTreeMap<SemanticId, ComputedValue>,
    expression_graph: &ExpressionGraph,
) -> Vec<SemanticReference> {
    let mut references = BTreeMap::new();

    for computed in computed_values.values() {
        let Some(component_id) = computed.owner.entity_id() else {
            continue;
        };
        let Some(component) = components
            .iter()
            .find(|component| component.id == *component_id)
        else {
            continue;
        };

        for node in expression_graph.nodes_for(&computed.id) {
            let crate::ExpressionNodeKind::ThisMember { name } = &node.kind else {
                continue;
            };
            let reference = if let Some(field) = component
                .state_fields
                .iter()
                .find(|field| field.name == *name)
            {
                SemanticReference {
                    kind: SemanticReferenceKind::ComputedState,
                    source: computed.id.clone(),
                    target: field.id.clone(),
                    provenance: computed.provenance.clone(),
                }
            } else if let Some(target) = computed_values.get(&component.id.computed(name)) {
                SemanticReference {
                    kind: SemanticReferenceKind::ComputedComputed,
                    source: computed.id.clone(),
                    target: target.id.clone(),
                    provenance: computed.provenance.clone(),
                }
            } else {
                continue;
            };
            references.insert(
                (reference.source.clone(), reference.target.clone()),
                reference,
            );
        }
    }

    references.into_values().collect()
}

fn build_template_state_references(
    components: &[ComponentNode],
    template_entities: &[TemplateSemanticEntity],
    ownership: &BTreeMap<SemanticId, SemanticOwner>,
) -> Vec<SemanticReference> {
    template_entities
        .iter()
        .filter(|entity| {
            matches!(
                entity.kind,
                TemplateSemanticKind::Binding
                    | TemplateSemanticKind::AttributeBinding
                    | TemplateSemanticKind::Conditional
                    | TemplateSemanticKind::List
            )
        })
        .filter_map(|entity| {
            let field_name = entity.expression.as_deref().and_then(this_member_name)?;
            let component = template_entity_component(components, ownership, entity)?;
            let field = component
                .state_fields
                .iter()
                .find(|field| field.name == field_name)?;

            Some(SemanticReference {
                kind: SemanticReferenceKind::TemplateState,
                source: entity.id.clone(),
                target: field.id.clone(),
                provenance: entity.provenance.clone(),
            })
        })
        .collect()
}

fn build_template_event_references(
    components: &[ComponentNode],
    template_entities: &[TemplateSemanticEntity],
    ownership: &BTreeMap<SemanticId, SemanticOwner>,
) -> Vec<SemanticReference> {
    template_entities
        .iter()
        .filter(|entity| entity.kind == TemplateSemanticKind::EventAttribute)
        .filter_map(|entity| {
            let method_name = entity.expression.as_deref().and_then(this_member_name)?;
            let component = template_entity_component(components, ownership, entity)?;
            let method = component
                .methods
                .iter()
                .find(|method| method.name == method_name)?;

            Some(SemanticReference {
                kind: SemanticReferenceKind::EventMethod,
                source: entity.id.clone(),
                target: method.id.clone(),
                provenance: entity.provenance.clone(),
            })
        })
        .collect()
}

fn build_template_local_references(
    components: &[ComponentNode],
    template_entities: &[TemplateSemanticEntity],
    ownership: &BTreeMap<SemanticId, SemanticOwner>,
) -> Vec<SemanticReference> {
    let mut references = template_entities
        .iter()
        .filter(|entity| {
            matches!(
                entity.kind,
                TemplateSemanticKind::Binding | TemplateSemanticKind::AttributeBinding
            ) && entity.scope == TemplateSemanticScope::Render
        })
        .filter_map(|entity| {
            let name = entity.expression.as_deref()?;
            let component = template_entity_component(components, ownership, entity)?;
            let render = component
                .methods
                .iter()
                .find(|method| method.name == "render")?;
            let local = unique_local_variable(render, name)?;

            Some(SemanticReference {
                kind: SemanticReferenceKind::TemplateLocal,
                source: entity.id.clone(),
                target: local.id.clone(),
                provenance: entity.provenance.clone(),
            })
        })
        .collect::<Vec<_>>();
    references.sort_by(|left, right| {
        (left.source.as_str(), left.target.as_str())
            .cmp(&(right.source.as_str(), right.target.as_str()))
    });
    references
}

fn unique_local_variable<'a>(
    method: &'a ComponentMethod,
    name: &str,
) -> Option<&'a MethodLocalVariable> {
    let mut locals = method
        .local_variables
        .iter()
        .filter(|local| local.name == name);
    let local = locals.next()?;
    locals.next().is_none().then_some(local)
}

fn template_entity_component<'a>(
    components: &'a [ComponentNode],
    ownership: &BTreeMap<SemanticId, SemanticOwner>,
    entity: &TemplateSemanticEntity,
) -> Option<&'a ComponentNode> {
    let template_id = ownership.get(&entity.id)?.entity_id()?;
    let component_id = ownership.get(template_id)?.entity_id()?;

    components
        .iter()
        .find(|component| component.id == *component_id)
}

fn this_member_name(expression: &str) -> Option<&str> {
    expression.strip_prefix("this.").filter(|name| {
        !name.is_empty()
            && name
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
    })
}

fn collect_ownership(
    components: &[ComponentNode],
    computed_values: &BTreeMap<SemanticId, ComputedValue>,
    templates: &[TemplateNode],
    template_entities: &[TemplateSemanticEntity],
) -> BTreeMap<SemanticId, SemanticOwner> {
    let mut ownership = BTreeMap::new();

    for component in components {
        ownership.insert(component.id.clone(), SemanticOwner::Application);

        for field in &component.state_fields {
            ownership.insert(
                field.id.clone(),
                SemanticOwner::entity(component.id.clone()),
            );
        }
        for method in &component.methods {
            ownership.insert(
                method.id.clone(),
                SemanticOwner::entity(component.id.clone()),
            );
            for parameter in &method.parameters {
                ownership.insert(
                    parameter.id.clone(),
                    SemanticOwner::entity(method.id.clone()),
                );
            }
            for local in &method.local_variables {
                ownership.insert(local.id.clone(), SemanticOwner::entity(method.id.clone()));
            }
        }
        for computed in computed_values
            .values()
            .filter(|computed| computed.owner.entity_id() == Some(&component.id))
        {
            ownership.insert(computed.id.clone(), computed.owner.clone());
        }
        for action in &component.actions {
            ownership.insert(
                action.id.clone(),
                SemanticOwner::entity(component.id.method(&action.method)),
            );
        }
        if let Some(render) = &component.render {
            for handler in render_event_handlers(render) {
                ownership.insert(
                    handler.id.clone(),
                    SemanticOwner::entity(component.id.template()),
                );
            }
        }
    }

    for template in templates {
        let component = components
            .iter()
            .find(|component| component.id.template() == template.id)
            .expect("template graph should only contain component templates");
        ownership.insert(
            template.id.clone(),
            SemanticOwner::entity(component.id.clone()),
        );
    }

    for entity in template_entities {
        ownership.insert(entity.id.clone(), entity.owner.clone());
    }

    ownership
}

#[cfg(test)]
mod tests {
    use super::{
        build_application_semantic_model, build_application_semantic_model_for_unit,
        collect_ownership,
    };
    use crate::{
        build_component_graph_for_module, build_template_graph, build_template_semantic_entities,
        CompilationUnit, SemanticEntity, SemanticEntityKind, SemanticOwner, SemanticReferenceKind,
        TemplateSemanticKind,
    };

    #[test]
    fn lowers_supported_primitive_state_annotations_into_canonical_types() {
        let parsed = ezc_parser::parse_file(
            "src/TypedState.tsx",
            r#"
@component("x-typed-state")
class TypedState extends Component {
  count: number = state(0);
}
"#,
        );

        let asm = build_application_semantic_model(&parsed);

        let field = &asm.components[0].state_fields[0];
        let assignment = asm
            .semantic_types
            .assignments
            .get(&field.id)
            .expect("canonical declared type");

        assert_eq!(assignment.semantic_type, crate::SemanticType::Number);
        assert_eq!(assignment.status, crate::SemanticTypeStatus::Declared);
        assert_eq!(
            assignment.provenance,
            field.declared_type.as_ref().unwrap().provenance
        );
    }

    #[test]
    fn queries_canonical_type_information_from_the_asm() {
        let parsed = ezc_parser::parse_file(
            "src/TypeQueries.tsx",
            r#"
@component("x-type-queries")
class TypeQueries extends Component {
  count: number = state(0);
  label = state("EdgeZero");
}
"#,
        );
        let asm = build_application_semantic_model(&parsed);
        let count = &asm.components[0].state_fields[0];
        let label = &asm.components[0].state_fields[1];

        assert_eq!(
            asm.semantic_type_of(&count.id),
            Some(&crate::SemanticType::Number)
        );
        assert_eq!(asm.type_declarations(&crate::SemanticType::Number).len(), 1);
        assert_eq!(asm.type_usages(&crate::SemanticType::String).len(), 1);
        assert_eq!(
            asm.serialization_compatibility_of(&count.id),
            Some(crate::SerializationCompatibility::Serializable)
        );
        assert_eq!(asm.is_type_assignable(&label.id, &count.id), Some(false));
    }

    #[test]
    fn lowers_typed_method_parameters_into_canonical_entities() {
        let parsed = ezc_parser::parse_file(
            "src/Parameters.tsx",
            r#"
@component("x-parameters")
class Parameters extends Component {
  save(title: string, retries?: number) {}
}
"#,
        );

        let asm = build_application_semantic_model(&parsed);
        let method = &asm.components[0].methods[0];
        let parameters = &method.parameters;

        assert_eq!(parameters.len(), 2);
        assert_eq!(
            asm.semantic_types.assignments[&parameters[0].id].semantic_type,
            crate::SemanticType::String
        );
        assert_eq!(
            asm.semantic_types.assignments[&parameters[1].id].semantic_type,
            crate::SemanticType::Number
        );
        assert!(parameters.iter().all(|parameter| {
            asm.owner(&parameter.id) == Some(&SemanticOwner::entity(method.id.clone()))
                && asm
                    .entity(&parameter.id)
                    .is_some_and(|entity| entity.kind() == SemanticEntityKind::Parameter)
        }));
    }

    #[test]
    fn lowers_declared_and_inferred_method_return_types() {
        let parsed = ezc_parser::parse_file(
            "src/Returns.tsx",
            r#"
@component("x-returns")
class Returns extends Component {
  declared(): string { return "EdgeZero"; }
  inferred() { return 1; }
}
"#,
        );

        let asm = build_application_semantic_model(&parsed);
        let methods = &asm.components[0].methods;
        let declared = &methods[0];
        let inferred = &methods[1];

        assert_eq!(
            asm.semantic_types.assignments[&declared.id].semantic_type,
            crate::SemanticType::String
        );
        assert_eq!(
            asm.semantic_types.assignments[&declared.id].status,
            crate::SemanticTypeStatus::Declared
        );
        assert_eq!(
            asm.semantic_types.assignments[&inferred.id].semantic_type,
            crate::SemanticType::Number
        );
        assert_eq!(
            asm.semantic_types.assignments[&inferred.id].status,
            crate::SemanticTypeStatus::Inferred
        );
    }

    #[test]
    fn establishes_typed_computed_getter_contracts() {
        let parsed = ezc_parser::parse_file(
            "src/Computed.tsx",
            r#"
@component("x-computed")
class Computed extends Component {
  @computed()
  get remainingCount(): number { return 1; }

  render() { return <p />; }
}
"#,
        );

        let asm = build_application_semantic_model(&parsed);
        let method = &asm.components[0].methods[0];
        let computed_id = asm.components[0].id.computed("remainingCount");
        let computed = asm
            .semantic_types
            .computed_values
            .get(&computed_id)
            .expect("computed type contract");
        let computed_entity = asm
            .computed_value(&computed_id)
            .expect("first-class computed entity");

        assert!(method.is_getter);
        assert!(method.is_computed());
        assert_eq!(
            computed.semantic_type,
            crate::SemanticType::NumberLiteral("1".to_string())
        );
        assert_eq!(
            computed.declared_return_type,
            Some(crate::SemanticType::Number)
        );
        assert_eq!(computed.declared_return_compatible, Some(true));
        assert_eq!(computed_entity.method, method.id);
        assert_eq!(
            computed_entity.owner,
            crate::SemanticOwner::entity(asm.components[0].id.clone())
        );
        assert_eq!(
            computed_entity.cache_policy,
            crate::ComputedCachePolicy::Memoized
        );
        assert_eq!(computed_entity.purity, crate::ComputedPurity::Unclassified);
        assert_eq!(
            computed_entity.execution_boundary,
            crate::ExecutionBoundary::Client
        );
        assert_eq!(
            asm.entity(&computed_id),
            Some(SemanticEntity::Computed(computed_entity))
        );
        assert_eq!(
            asm.owner(&computed_id),
            Some(&crate::SemanticOwner::entity(asm.components[0].id.clone()))
        );
        assert_eq!(asm.provenance(&computed_id), asm.provenance(&method.id));
        assert_eq!(
            asm.entities_of_kind(SemanticEntityKind::Computed),
            vec![&computed_id]
        );
    }

    #[test]
    fn resolves_computed_reads_to_canonical_state_and_computed_references() {
        let parsed = ezc_parser::parse_file(
            "src/ComputedReads.tsx",
            r#"
@component("x-computed-reads")
class ComputedReads extends Component {
  count = state(1);
  profile = state({ hidden: false });

  @computed()
  get doubled() { return this.count * 2; }

  @computed()
  get visible() { return this.doubled + this.count + this.count; }

  @computed()
  get profileHidden() { return this.profile.hidden; }

  @computed()
  get unresolved() { return this.missing; }
}
"#,
        );
        let asm = build_application_semantic_model(&parsed);
        let component = &asm.components[0];
        let count = component.id.state_field("count");
        let profile = component.id.state_field("profile");
        let doubled = component.id.computed("doubled");
        let visible = component.id.computed("visible");
        let profile_hidden = component.id.computed("profileHidden");
        let unresolved = component.id.computed("unresolved");

        let state_references = asm.references_of_kind(SemanticReferenceKind::ComputedState);
        assert_eq!(state_references.len(), 3);
        assert!(state_references
            .iter()
            .any(|reference| reference.source == doubled && reference.target == count));
        assert!(state_references
            .iter()
            .any(|reference| reference.source == visible && reference.target == count));
        assert!(state_references.iter().any(|reference| {
            reference.source == profile_hidden && reference.target == profile
        }));

        let computed_references = asm.references_of_kind(SemanticReferenceKind::ComputedComputed);
        assert_eq!(computed_references.len(), 1);
        assert_eq!(computed_references[0].source, visible);
        assert_eq!(computed_references[0].target, doubled);
        assert!(!asm
            .references
            .iter()
            .any(|reference| reference.source == unresolved));
        assert!(asm
            .references
            .iter()
            .all(|reference| { asm.provenance(&reference.source) == Some(&reference.provenance) }));
    }

    #[test]
    fn assigns_inferred_computed_types_and_validates_declared_returns() {
        let parsed = ezc_parser::parse_file(
            "src/ComputedTypes.tsx",
            r#"
@component("x-computed-types")
class ComputedTypes extends Component {
  count: number = state(1);
  profile = state({ label: "EdgeZero" });

  @computed()
  get doubled(): number { return this.count * 2; }

  @computed()
  get chained() { return this.doubled + 1; }

  @computed()
  get label(): string { return this.profile.label; }

  @computed()
  get invalid(): number { return "wrong"; }

  @computed()
  get unresolved() { return this.missing; }
}
"#,
        );
        let asm = build_application_semantic_model(&parsed);
        let component = &asm.components[0];
        let doubled = component.id.computed("doubled");
        let chained = component.id.computed("chained");
        let label = component.id.computed("label");
        let invalid = component.id.computed("invalid");
        let unresolved = component.id.computed("unresolved");

        let types = &asm.semantic_types.computed_values;
        assert_eq!(types[&doubled].semantic_type, crate::SemanticType::Number);
        assert_eq!(types[&chained].semantic_type, crate::SemanticType::Number);
        assert_eq!(types[&label].semantic_type, crate::SemanticType::String);
        assert_eq!(
            types[&invalid].semantic_type,
            crate::SemanticType::StringLiteral("wrong".to_string())
        );
        assert_eq!(types[&invalid].declared_return_compatible, Some(false));
        assert_eq!(types[&doubled].declared_return_compatible, Some(true));
        assert_eq!(
            types[&unresolved].semantic_type,
            crate::SemanticType::Unknown
        );
        assert_eq!(
            types[&unresolved].serialization,
            crate::SerializationCompatibility::NotSerializable
        );
        assert_eq!(
            types[&doubled].boundary_compatibility,
            crate::BoundaryCompatibility::Compatible
        );
        assert_eq!(
            types[&doubled].execution_boundary,
            crate::ExecutionBoundary::Client
        );
        assert_eq!(
            asm.semantic_type_of(&doubled),
            Some(&crate::SemanticType::Number)
        );
        assert_eq!(
            asm.serialization_compatibility_of(&chained),
            Some(crate::SerializationCompatibility::Serializable)
        );
    }

    #[test]
    fn establishes_typed_action_input_and_output_contracts() {
        let parsed = ezc_parser::parse_file(
            "src/Action.tsx",
            r#"
type ActionResult = number;

@component("x-action")
class Action extends Component {
  @action()
  async addTodo(input: string): ActionResult { return 1; }
}
"#,
        );

        let asm = build_application_semantic_model(&parsed);
        let method = &asm.components[0].methods[0];
        let signature = asm
            .semantic_types
            .action_signatures
            .get(&method.id)
            .expect("action signature");

        assert!(method.is_action());
        assert!(signature.is_async);
        assert_eq!(signature.input.len(), 1);
        assert_eq!(signature.input[0].1, crate::SemanticType::String);
        assert_eq!(signature.output, Some(crate::SemanticType::Number));
    }

    #[test]
    fn lowers_supported_literal_state_annotations_into_canonical_types() {
        let parsed = ezc_parser::parse_file(
            "src/LiteralTypes.tsx",
            r#"
@component("x-literal-types")
class LiteralTypes extends Component {
  filter: "all" = state("all");
  step: 42 = state(42);
  enabled: true = state(true);
}
"#,
        );

        let asm = build_application_semantic_model(&parsed);
        let types = asm.components[0]
            .state_fields
            .iter()
            .map(|field| {
                (
                    field.name.as_str(),
                    &asm.semantic_types.assignments[&field.id].semantic_type,
                )
            })
            .collect::<Vec<_>>();

        assert_eq!(
            types,
            vec![
                (
                    "filter",
                    &crate::SemanticType::StringLiteral("all".to_string())
                ),
                (
                    "step",
                    &crate::SemanticType::NumberLiteral("42".to_string())
                ),
                ("enabled", &crate::SemanticType::BooleanLiteral(true)),
            ]
        );
    }

    #[test]
    fn lowers_array_and_tuple_state_annotations_into_canonical_types() {
        let parsed = ezc_parser::parse_file(
            "src/CollectionTypes.tsx",
            r#"
@component("x-collection-types")
class CollectionTypes extends Component {
  names: string[] = state([]);
  todos: Todo[] = state([]);
  pair: [string, number] = state(["EdgeZero", 1]);
}
"#,
        );

        let asm = build_application_semantic_model(&parsed);
        let types = asm.components[0]
            .state_fields
            .iter()
            .map(|field| {
                (
                    field.name.as_str(),
                    &asm.semantic_types.assignments[&field.id].semantic_type,
                )
            })
            .collect::<Vec<_>>();

        assert_eq!(
            types,
            vec![
                (
                    "names",
                    &crate::SemanticType::Array(Box::new(crate::SemanticType::String)),
                ),
                (
                    "todos",
                    &crate::SemanticType::Array(Box::new(crate::SemanticType::Unknown)),
                ),
                (
                    "pair",
                    &crate::SemanticType::Tuple(vec![
                        crate::SemanticType::String,
                        crate::SemanticType::Number,
                    ]),
                ),
            ]
        );
    }

    #[test]
    fn lowers_structural_object_state_annotations_into_canonical_types() {
        let parsed = ezc_parser::parse_file(
            "src/ObjectTypes.tsx",
            r#"
@component("x-object-types")
class ObjectTypes extends Component {
  todo: { id: string; title: string; completed: boolean } = state({ id: "1", title: "EdgeZero", completed: false });
}
"#,
        );

        let asm = build_application_semantic_model(&parsed);
        let field = &asm.components[0].state_fields[0];
        let assignment = &asm.semantic_types.assignments[&field.id];
        let crate::SemanticType::Object(object) = &assignment.semantic_type else {
            panic!("expected structural object type");
        };

        assert_eq!(
            object.properties,
            std::collections::BTreeMap::from([
                ("completed".to_string(), crate::SemanticType::Boolean),
                ("id".to_string(), crate::SemanticType::String),
                ("title".to_string(), crate::SemanticType::String),
            ])
        );
    }

    #[test]
    fn lowers_union_and_nullable_state_annotations_into_canonical_types() {
        let parsed = ezc_parser::parse_file(
            "src/UnionTypes.tsx",
            r#"
@component("x-union-types")
class UnionTypes extends Component {
  filter: "all" | "active" | "completed" = state("all");
  user: { id: string } | null = state(null);
}
"#,
        );

        let asm = build_application_semantic_model(&parsed);
        let fields = &asm.components[0].state_fields;
        assert_eq!(
            asm.semantic_types.assignments[&fields[0].id].semantic_type,
            crate::SemanticType::Union(vec![
                crate::SemanticType::StringLiteral("active".to_string()),
                crate::SemanticType::StringLiteral("all".to_string()),
                crate::SemanticType::StringLiteral("completed".to_string()),
            ])
        );
        assert_eq!(
            asm.semantic_types.assignments[&fields[1].id].semantic_type,
            crate::SemanticType::Union(vec![
                crate::SemanticType::Null,
                crate::SemanticType::Object(crate::ObjectType {
                    properties: std::collections::BTreeMap::from([(
                        "id".to_string(),
                        crate::SemanticType::String,
                    )]),
                }),
            ])
        );
    }

    #[test]
    fn resolves_local_type_aliases_with_canonical_alias_identity() {
        let parsed = ezc_parser::parse_file(
            "src/Aliases.tsx",
            r#"
type TodoId = string;
type Filter = "all" | "active" | "completed";

@component("x-aliases")
class Aliases extends Component {
  id: TodoId = state("todo-1");
  filter: Filter = state("all");
}
"#,
        );

        let asm = build_application_semantic_model(&parsed);
        let fields = &asm.components[0].state_fields;
        let todo_id = asm
            .semantic_types
            .aliases
            .values()
            .find(|alias| alias.name == "TodoId")
            .expect("TodoId alias");
        let filter = asm
            .semantic_types
            .aliases
            .values()
            .find(|alias| alias.name == "Filter")
            .expect("Filter alias");

        assert_eq!(todo_id.semantic_type, crate::SemanticType::String);
        assert_eq!(
            filter.semantic_type,
            crate::SemanticType::Union(vec![
                crate::SemanticType::StringLiteral("active".to_string()),
                crate::SemanticType::StringLiteral("all".to_string()),
                crate::SemanticType::StringLiteral("completed".to_string()),
            ])
        );
        assert_eq!(
            asm.semantic_types.assignments[&fields[0].id].origin,
            todo_id.id
        );
        assert_eq!(
            asm.semantic_types.assignments[&fields[1].id].origin,
            filter.id
        );
    }

    #[test]
    fn resolves_imported_type_aliases_through_named_reexports() {
        let unit = CompilationUnit::parse_sources([
            (
                "src/types.ts",
                r#"export type Filter = "all" | "active" | "completed";"#,
            ),
            ("src/index.ts", r#"export { Filter } from "./types";"#),
            (
                "src/App.tsx",
                r#"
import { Filter } from "./index";

@component("x-app")
class App extends Component {
  filter: Filter = state("all");
}
"#,
            ),
        ]);

        let asm = build_application_semantic_model_for_unit(&unit);
        let field = &asm
            .components
            .iter()
            .find(|component| component.class_name == "App")
            .expect("App component")
            .state_fields[0];
        let assignment = &asm.semantic_types.assignments[&field.id];

        assert_eq!(
            assignment.semantic_type,
            crate::SemanticType::Union(vec![
                crate::SemanticType::StringLiteral("active".to_string()),
                crate::SemanticType::StringLiteral("all".to_string()),
                crate::SemanticType::StringLiteral("completed".to_string()),
            ])
        );
        assert_eq!(
            assignment.origin,
            crate::SemanticId::type_alias_in_module("src/types.ts", "Filter")
        );
    }

    #[test]
    fn infers_state_types_from_direct_serializable_initializers() {
        let parsed = ezc_parser::parse_file(
            "src/InferredState.tsx",
            r#"
@component("x-inferred-state")
class InferredState extends Component {
  count = state(0);
  todos = state([]);
  tags = state(["EdgeZero"]);
  todo = state({ id: "1", completed: false });
}
"#,
        );

        let asm = build_application_semantic_model(&parsed);
        let fields = &asm.components[0].state_fields;
        let types = fields
            .iter()
            .map(|field| {
                let assignment = &asm.semantic_types.assignments[&field.id];
                assert_eq!(assignment.status, crate::SemanticTypeStatus::Inferred);
                (field.name.as_str(), assignment.semantic_type.clone())
            })
            .collect::<Vec<_>>();

        assert_eq!(
            types,
            vec![
                ("count", crate::SemanticType::Number),
                (
                    "todos",
                    crate::SemanticType::Array(Box::new(crate::SemanticType::Unknown)),
                ),
                (
                    "tags",
                    crate::SemanticType::Array(Box::new(crate::SemanticType::String)),
                ),
                (
                    "todo",
                    crate::SemanticType::Object(crate::ObjectType {
                        properties: std::collections::BTreeMap::from([
                            ("completed".to_string(), crate::SemanticType::Boolean),
                            ("id".to_string(), crate::SemanticType::String),
                        ]),
                    }),
                ),
            ]
        );
    }

    #[test]
    fn propagates_canonical_types_to_expression_graph_nodes() {
        let parsed = ezc_parser::parse_file(
            "src/ExpressionTypes.tsx",
            r#"
@component("x-expression-types")
class ExpressionTypes extends Component {
  total = state((1 + 2) * 3);
  ready = state(1 < 2);
}
"#,
        );

        let asm = build_application_semantic_model(&parsed);
        let total = &asm.components[0].state_fields[0];
        let ready = &asm.components[0].state_fields[1];
        let total_root = asm
            .expression_root(&total.id)
            .expect("total expression root");
        let ready_root = asm
            .expression_root(&ready.id)
            .expect("ready expression root");

        assert_eq!(
            asm.semantic_types.assignments[total_root].semantic_type,
            crate::SemanticType::Number
        );
        assert_eq!(
            asm.semantic_types.assignments[ready_root].semantic_type,
            crate::SemanticType::Boolean
        );
        assert!(asm
            .expression_dependencies(total_root)
            .iter()
            .all(|id| asm.semantic_types.assignments.contains_key(*id)));
    }

    #[test]
    fn derives_component_ownership_without_legacy_owner_fields() {
        let parsed = ezc_parser::parse_file(
            "src/Counter.tsx",
            r#"
@component("x-counter")
class Counter extends Component {
  count = state(0);

  increment() {
    this.count++;
  }

  render() {
    return <button onClick={() => this.increment()}>{this.count}</button>;
  }
}
"#,
        );
        let mut component_graph = build_component_graph_for_module(&parsed);
        let mut templates = build_template_graph(&component_graph).templates;
        let template_entities = build_template_semantic_entities(&templates);
        let component = component_graph
            .components
            .first_mut()
            .expect("component graph should contain Counter");
        let component_id = component.id.clone();
        let method_id = component.methods[0].id.clone();
        let action_id = component.actions[0].id.clone();
        let state_id = component.state_fields[0].id.clone();
        let event_id = component
            .render
            .as_ref()
            .expect("Counter should render")
            .event_handlers[0]
            .id
            .clone();

        component.owner = SemanticOwner::entity(component_id.clone());
        component.state_fields[0].owner = SemanticOwner::Application;
        component.methods[0].owner = SemanticOwner::Application;
        component.actions[0].owner = SemanticOwner::Application;
        component
            .render
            .as_mut()
            .expect("Counter should render")
            .event_handlers[0]
            .owner = SemanticOwner::Application;
        templates[0].owner = SemanticOwner::Application;

        let ownership = collect_ownership(
            &component_graph.components,
            &std::collections::BTreeMap::new(),
            &templates,
            &template_entities,
        );
        assert_eq!(ownership[&component_id], SemanticOwner::Application);
        assert_eq!(
            ownership[&state_id],
            SemanticOwner::entity(component_id.clone())
        );
        assert_eq!(
            ownership[&method_id],
            SemanticOwner::entity(component_id.clone())
        );
        assert_eq!(ownership[&action_id], SemanticOwner::entity(method_id));
        assert_eq!(
            ownership[&event_id],
            SemanticOwner::entity(component_id.clone().template())
        );
        assert_eq!(
            ownership[&templates[0].id],
            SemanticOwner::entity(component_id)
        );
    }

    #[test]
    fn traverses_application_ownership_in_semantic_id_order() {
        let parsed = ezc_parser::parse_file(
            "src/Counter.tsx",
            r#"
@component("x-counter")
class Counter extends Component {
  count = state(0);

  increment() {
    this.count++;
  }

  render() {
    return <button onClick={this.increment}>{this.count}</button>;
  }
}
"#,
        );

        let asm = build_application_semantic_model(&parsed);
        let component = &asm.components[0];
        let roots = asm.application_roots();

        assert_eq!(roots, vec![&component.id]);
        assert_eq!(
            asm.children_of(&component.id),
            vec![
                &component.methods[0].id,
                &component.methods[1].id,
                &component.state_fields[0].id,
                &asm.templates[0].id,
            ]
        );
        assert_eq!(
            asm.children_of(&component.methods[0].id),
            vec![&component.actions[0].id]
        );
        assert_eq!(asm.parent_of(&component.id), None);
        assert_eq!(
            asm.parent_of(&component.actions[0].id),
            Some(&component.methods[0].id)
        );
        assert_eq!(
            asm.ancestors_of(&component.actions[0].id),
            vec![&component.methods[0].id, &component.id]
        );
        let descendants = asm.descendants_of(&component.id);
        assert_eq!(descendants[0], &component.methods[0].id);
        assert_eq!(descendants[1], &component.actions[0].id);
        assert_eq!(descendants[2], &component.methods[1].id);
        assert_eq!(descendants.len(), asm.ownership.len() - 1);
        assert_eq!(
            asm.entities_of_kind(SemanticEntityKind::Method),
            vec![&component.methods[0].id, &component.methods[1].id]
        );
        assert_eq!(
            asm.entities_of_kind(SemanticEntityKind::Action),
            vec![&component.actions[0].id]
        );
        assert_eq!(
            asm.entities_of_kind(SemanticEntityKind::StateField),
            vec![&component.state_fields[0].id]
        );
        let state_id = &component.state_fields[0].id;
        let state_provenance = asm.provenance(state_id).expect("state provenance");
        assert_eq!(
            asm.entities_in_file(state_provenance.path.as_path()).len(),
            asm.ownership.len()
        );
        let at_state =
            asm.entities_at(state_provenance.path.as_path(), state_provenance.span.start);
        assert!(at_state.contains(&state_id));
        assert!(at_state.iter().all(|id| {
            let provenance = asm.provenance(id).expect("entity provenance");
            provenance.span.start <= state_provenance.span.start
                && state_provenance.span.start < provenance.span.end
        }));
        let action_references = asm.references_of_kind(SemanticReferenceKind::ActionState);
        assert_eq!(action_references.len(), 1);
        assert_eq!(action_references[0].source, component.actions[0].id);
        assert_eq!(action_references[0].target, component.state_fields[0].id);
        let action_provenance = &action_references[0].provenance;
        assert_eq!(
            asm.references_in_file(action_provenance.path.as_path())
                .len(),
            asm.references.len()
        );
        let at_action = asm.references_at(
            action_provenance.path.as_path(),
            action_provenance.span.start,
        );
        assert!(at_action.iter().any(|reference| {
            reference.source == component.actions[0].id
                && reference.target == component.state_fields[0].id
        }));
    }

    #[test]
    fn resolves_template_state_dependencies() {
        let parsed = ezc_parser::parse_file(
            "src/Panel.tsx",
            r#"
@component("x-panel")
class Panel extends Component {
  enabled = state(true);

  render() {
    return <section hidden={this.enabled}>{this.enabled}{this.enabled ? <span>On</span> : <span>Off</span>}</section>;
  }
}
"#,
        );

        let asm = build_application_semantic_model(&parsed);
        let component = &asm.components[0];
        let state = &component.state_fields[0];
        let state_references = asm.references_to(&state.id);

        assert_eq!(state_references.len(), 3);
        assert!(state_references.iter().all(|reference| {
            reference.kind == SemanticReferenceKind::TemplateState
                && reference.target == state.id
                && reference.provenance.path.as_path() == std::path::Path::new("src/Panel.tsx")
        }));
        assert!(state_references.iter().any(|reference| {
            asm.template_entity(&reference.source)
                .is_some_and(|entity| entity.kind == TemplateSemanticKind::Binding)
        }));
        assert!(state_references.iter().any(|reference| {
            asm.template_entity(&reference.source)
                .is_some_and(|entity| entity.kind == TemplateSemanticKind::AttributeBinding)
        }));
        assert!(state_references.iter().any(|reference| {
            asm.template_entity(&reference.source)
                .is_some_and(|entity| entity.kind == TemplateSemanticKind::Conditional)
        }));
    }

    #[test]
    fn navigates_template_entities_from_canonical_ownership() {
        let parsed = ezc_parser::parse_file(
            "src/Panel.tsx",
            r#"
@component("x-panel")
class Panel extends Component {
  count = state(0);

  render() {
    return <section>{this.count}</section>;
  }
}
"#,
        );
        let mut asm = build_application_semantic_model(&parsed);
        let template_id = asm.templates[0].id.clone();
        let expected = asm.template_entities_for(&template_id).len();

        for entity in &mut asm.template_entities {
            entity.owner = SemanticOwner::Application;
        }

        assert_eq!(asm.template_entities_for(&template_id).len(), expected);
    }

    #[test]
    fn leaves_member_expressions_unresolved_without_expression_evaluation() {
        let parsed = ezc_parser::parse_file(
            "src/Panel.tsx",
            r#"
@component("x-panel")
class Panel extends Component {
  enabled = state(true);

  render() {
    return <section data-value={this.enabled.value}>Panel</section>;
  }
}
"#,
        );

        let asm = build_application_semantic_model(&parsed);
        assert_eq!(
            asm.references
                .iter()
                .filter(|reference| reference.kind == SemanticReferenceKind::TemplateState)
                .count(),
            0
        );
    }

    #[test]
    fn resolves_keyed_list_iterable_to_component_state() {
        let parsed = ezc_parser::parse_file(
            "src/KeyedList.tsx",
            r#"
@component("x-keyed-list")
class KeyedList extends Component {
  items = state([]);

  render() {
    return <ul>{this.items.map((item) => <li key={item}>{item}</li>)}</ul>;
  }
}
"#,
        );

        let asm = build_application_semantic_model(&parsed);
        let component = &asm.components[0];
        let state = &component.state_fields[0];
        let reference = asm
            .references_to(&state.id)
            .into_iter()
            .find(|reference| reference.kind == SemanticReferenceKind::TemplateState)
            .expect("list iterable state reference");
        let list = asm
            .template_entity(&reference.source)
            .expect("list semantic entity");

        assert_eq!(list.kind, TemplateSemanticKind::List);
        assert_eq!(list.expression.as_deref(), Some("this.items"));
        assert_eq!(reference.provenance, list.provenance);
    }

    #[test]
    fn resolves_template_event_attribute_to_component_method() {
        let parsed = ezc_parser::parse_file(
            "src/Counter.tsx",
            r#"
@component("x-counter")
class Counter extends Component {
  count = state(0);

  increment() {
    this.count += 1;
  }

  render() {
    return <button onClick={() => this.increment()}>Count</button>;
  }
}
"#,
        );

        let asm = build_application_semantic_model(&parsed);
        let component = &asm.components[0];
        let method = component
            .methods
            .iter()
            .find(|method| method.name == "increment")
            .expect("increment method");
        let reference = asm
            .references_to(&method.id)
            .into_iter()
            .find(|reference| {
                reference.kind == SemanticReferenceKind::EventMethod
                    && asm
                        .template_entity(&reference.source)
                        .is_some_and(|entity| entity.kind == TemplateSemanticKind::EventAttribute)
            })
            .expect("template event method reference");

        assert_eq!(
            reference.provenance.path,
            std::path::Path::new("src/Counter.tsx")
        );
        assert_eq!(reference.provenance.span.line, 11);
    }

    #[test]
    fn resolves_render_template_bindings_to_unique_method_locals() {
        let parsed = ezc_parser::parse_file(
            "src/LocalResolution.tsx",
            r#"
@component("x-local-resolution")
class LocalResolution extends Component {
  render() {
    const title = "EdgeZero";
    return <output title={title}>{title}</output>;
  }
}
"#,
        );

        let asm = build_application_semantic_model(&parsed);
        let component = &asm.components[0];
        let render = component
            .methods
            .iter()
            .find(|method| method.name == "render")
            .expect("render method");
        let local = &render.local_variables[0];
        let references = asm.references_of_kind(SemanticReferenceKind::TemplateLocal);
        let assignment = asm
            .semantic_types
            .assignments
            .get(&local.id)
            .expect("canonical inferred local type");

        assert_eq!(
            asm.owner(&local.id),
            Some(&SemanticOwner::entity(render.id.clone()))
        );
        assert_eq!(
            asm.entities_of_kind(SemanticEntityKind::LocalVariable),
            vec![&local.id]
        );
        assert_eq!(references.len(), 2);
        assert!(references.iter().all(|reference| {
            reference.target == local.id
                && reference.provenance.path == std::path::Path::new("src/LocalResolution.tsx")
        }));
        assert_eq!(assignment.semantic_type, crate::SemanticType::String);
        assert_eq!(assignment.status, crate::SemanticTypeStatus::Inferred);
        assert_eq!(assignment.provenance.span, local.span);

        let semantic_graph = crate::build_semantic_graph(&asm);
        assert!(semantic_graph.nodes.iter().any(|node| {
            node.kind == crate::SemanticGraphNodeKind::LocalVariable && node.id == local.id
        }));
        assert_eq!(
            semantic_graph
                .edges
                .iter()
                .filter(|edge| {
                    edge.kind == crate::SemanticGraphEdgeKind::TemplateLocal
                        && edge.target == local.id
                })
                .count(),
            2
        );

        let template_graph = build_template_graph(&build_component_graph_for_module(&parsed));
        assert_eq!(
            crate::generate_static_html(&template_graph),
            "<output data-ez-node=\"n0\" title=\"EdgeZero\" data-ez-bindings=\"title\"><!-- ez-binding:n2:title -->EdgeZero</output>\n"
        );
    }

    #[test]
    fn queries_canonical_expression_graphs_by_dependency_provenance_and_owner() {
        let parsed = ezc_parser::parse_file(
            "src/ExpressionQueries.tsx",
            r#"
@component("x-expression-queries")
class ExpressionQueries extends Component {
  total = state((1 + 2) * 3);
}
"#,
        );

        let asm = build_application_semantic_model(&parsed);
        let field = &asm.components[0].state_fields[0];
        let root = asm.expression_root(&field.id).expect("expression root");
        let left = field.id.expression("root.0");
        let right = field.id.expression("root.1");

        assert_eq!(asm.expression(root).map(|node| &node.id), Some(root));
        assert_eq!(asm.expression_owner(root), Some(&field.id));
        assert_eq!(asm.expression_dependencies(root), vec![&left, &right]);
        assert_eq!(
            asm.expression_dependents(&left)
                .into_iter()
                .map(|node| &node.id)
                .collect::<Vec<_>>(),
            vec![root]
        );
        assert_eq!(asm.expressions_for(&field.id).len(), 5);

        let provenance = asm
            .expression_provenance(root)
            .expect("expression provenance");
        assert_eq!(
            provenance.path,
            std::path::Path::new("src/ExpressionQueries.tsx")
        );
        assert_eq!(asm.expressions_in_file(&provenance.path).len(), 5);
        assert_eq!(
            asm.expressions_at(&provenance.path, provenance.span.start)
                .into_iter()
                .map(|node| &node.id)
                .collect::<Vec<_>>(),
            vec![root]
        );
    }
}
