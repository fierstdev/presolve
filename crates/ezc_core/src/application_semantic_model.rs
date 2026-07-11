use std::collections::BTreeMap;

use ezc_parser::ParsedFile;

use crate::compilation_unit::CompilationUnit;
use crate::component_graph::{
    build_component_graph_for_module, render_event_handlers, ComponentAction, ComponentDiagnostic,
    ComponentMethod, ComponentNode, RenderEventHandler, StateField,
};
use crate::semantic_id::{SemanticId, SemanticOwner};
use crate::semantic_provenance::SourceProvenance;
use crate::semantic_reference::{SemanticReference, SemanticReferenceKind};
use crate::template_graph::{build_template_graph, TemplateNode};
use crate::template_semantics::{
    build_template_semantic_entities, TemplateSemanticEntity, TemplateSemanticKind,
};

/// Application-level semantic data assembled from the compiler's existing graphs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApplicationSemanticModel {
    pub components: Vec<ComponentNode>,
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
    Action(&'a ComponentAction),
    EventHandler(&'a RenderEventHandler),
    Template(&'a TemplateNode),
    TemplateEntity(&'a TemplateSemanticEntity),
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
    pub fn template_entity(&self, id: &SemanticId) -> Option<&TemplateSemanticEntity> {
        self.template_entities
            .iter()
            .find(|entity| entity.id == *id)
    }

    #[must_use]
    pub fn template_entities_for(&self, template: &SemanticId) -> Vec<&TemplateSemanticEntity> {
        self.template_entities
            .iter()
            .filter(|entity| entity.owner.entity_id() == Some(template))
            .collect()
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
    let mut template_entities = Vec::new();

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
    }

    references.extend(build_template_state_references(
        &components,
        &templates,
        &template_entities,
    ));
    references.extend(build_template_event_references(
        &components,
        &templates,
        &template_entities,
    ));

    let ownership = collect_ownership(&components, &templates, &template_entities);

    ApplicationSemanticModel {
        components,
        templates,
        template_entities,
        diagnostics,
        ownership,
        references,
        provenance,
    }
}

fn build_template_state_references(
    components: &[ComponentNode],
    templates: &[TemplateNode],
    template_entities: &[TemplateSemanticEntity],
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
            let component = template_entity_component(components, templates, entity)?;
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
    templates: &[TemplateNode],
    template_entities: &[TemplateSemanticEntity],
) -> Vec<SemanticReference> {
    template_entities
        .iter()
        .filter(|entity| entity.kind == TemplateSemanticKind::EventAttribute)
        .filter_map(|entity| {
            let method_name = entity.expression.as_deref().and_then(this_member_name)?;
            let component = template_entity_component(components, templates, entity)?;
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

fn template_entity_component<'a>(
    components: &'a [ComponentNode],
    templates: &[TemplateNode],
    entity: &TemplateSemanticEntity,
) -> Option<&'a ComponentNode> {
    let template_id = entity.owner.entity_id()?;
    let component_id = templates
        .iter()
        .find(|template| template.id == *template_id)
        .and_then(|template| template.owner.entity_id())?;

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
    templates: &[TemplateNode],
    template_entities: &[TemplateSemanticEntity],
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

    for entity in template_entities {
        ownership.insert(entity.id.clone(), entity.owner.clone());
    }

    ownership
}

#[cfg(test)]
mod tests {
    use super::build_application_semantic_model;
    use crate::{SemanticReferenceKind, TemplateSemanticKind};

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
}
