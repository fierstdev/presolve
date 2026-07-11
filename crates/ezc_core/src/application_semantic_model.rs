use std::collections::BTreeMap;

use ezc_parser::ParsedFile;

use crate::component_graph::{
    build_component_graph, render_event_handlers, ComponentDiagnostic, ComponentNode,
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

#[must_use]
pub fn build_application_semantic_model(parsed: &ParsedFile) -> ApplicationSemanticModel {
    let component_graph = build_component_graph(parsed);
    let template_graph = build_template_graph(&component_graph);
    let ownership = collect_ownership(&component_graph.components, &template_graph.templates);

    ApplicationSemanticModel {
        components: component_graph.components,
        templates: template_graph.templates,
        diagnostics: component_graph.diagnostics,
        ownership,
        references: component_graph.references,
        provenance: component_graph.provenance,
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
