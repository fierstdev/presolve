use std::collections::BTreeMap;

use ezc_parser::ParsedFile;

use crate::compilation_unit::CompilationUnit;
use crate::component_graph::{
    build_component_graph, render_event_handlers, ComponentAction, ComponentDiagnostic,
    ComponentMethod, ComponentNode, RenderEventHandler, StateField,
};
use crate::semantic_id::{SemanticId, SemanticOwner};
use crate::semantic_provenance::SourceProvenance;
use crate::semantic_reference::SemanticReference;
use crate::template_graph::{build_template_graph, TemplateNode};

/// Application-level semantic data assembled from the compiler's existing graphs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApplicationSemanticModel {
    pub components: Vec<ComponentNode>,
    pub templates: Vec<TemplateNode>,
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
    Action(&'a ComponentAction),
    EventHandler(&'a RenderEventHandler),
    Template(&'a TemplateNode),
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

        self.templates
            .iter()
            .find(|template| template.id == *id)
            .map(SemanticEntity::Template)
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
    pub fn owner(&self, id: &SemanticId) -> Option<&SemanticOwner> {
        self.ownership.get(id)
    }

    #[must_use]
    pub fn provenance(&self, id: &SemanticId) -> Option<&SourceProvenance> {
        self.provenance.get(id)
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

#[must_use]
pub fn build_application_semantic_model_for_unit(
    unit: &CompilationUnit,
) -> ApplicationSemanticModel {
    build_application_semantic_model_from_files(unit.files())
}

fn build_application_semantic_model_from_files(files: &[ParsedFile]) -> ApplicationSemanticModel {
    let mut components = Vec::new();
    let mut templates = Vec::new();
    let mut diagnostics = Vec::new();
    let mut references = Vec::new();
    let mut provenance = BTreeMap::new();

    for parsed in files {
        let component_graph = build_component_graph(parsed);
        let template_graph = build_template_graph(&component_graph);

        components.extend(component_graph.components);
        templates.extend(template_graph.templates);
        diagnostics.extend(component_graph.diagnostics);
        references.extend(component_graph.references);
        provenance.extend(component_graph.provenance);
    }

    let ownership = collect_ownership(&components, &templates);

    ApplicationSemanticModel {
        components,
        templates,
        diagnostics,
        ownership,
        references,
        provenance,
    }
}

fn collect_ownership(
    components: &[ComponentNode],
    templates: &[TemplateNode],
) -> BTreeMap<SemanticId, SemanticOwner> {
    let mut ownership = BTreeMap::new();

    for component in components {
        ownership.insert(component.id.clone(), component.owner.clone());

        for field in &component.state_fields {
            ownership.insert(field.id.clone(), field.owner.clone());
        }
        for method in &component.methods {
            ownership.insert(method.id.clone(), method.owner.clone());
        }
        for action in &component.actions {
            ownership.insert(action.id.clone(), action.owner.clone());
        }
        if let Some(render) = &component.render {
            for handler in render_event_handlers(render) {
                ownership.insert(handler.id.clone(), handler.owner.clone());
            }
        }
    }

    for template in templates {
        ownership.insert(template.id.clone(), template.owner.clone());
    }

    ownership
}
