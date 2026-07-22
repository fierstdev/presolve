use crate::semantic_id::{SemanticId, SemanticOwner};
use crate::semantic_provenance::SourceProvenance;
use crate::template_graph::{
    AttributeValue, ConditionalNode, ElementNode, FragmentNode, ListNode, TemplateChild,
    TemplateNode,
};

/// Typed semantic entities lowered from authored template descendants.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TemplateSemanticEntity {
    pub id: SemanticId,
    pub owner: SemanticOwner,
    pub kind: TemplateSemanticKind,
    pub scope: TemplateSemanticScope,
    pub tag_name: Option<String>,
    pub tag_name_provenance: Option<SourceProvenance>,
    pub attribute_name: Option<String>,
    pub list_item_variable: Option<String>,
    pub list_index_variable: Option<String>,
    pub expression: Option<String>,
    pub provenance: SourceProvenance,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TemplateSemanticKind {
    Element,
    Fragment,
    Text,
    Binding,
    Attribute,
    AttributeBinding,
    EventAttribute,
    Conditional,
    List,
}

/// The lexical template scope in which an authored template entity appears.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TemplateSemanticScope {
    Render,
    ListItem,
}

#[must_use]
pub fn build_template_semantic_entities(templates: &[TemplateNode]) -> Vec<TemplateSemanticEntity> {
    let mut entities = Vec::new();

    for template in templates {
        if let Some(root) = &template.root {
            collect_element(
                root,
                template,
                "root",
                TemplateSemanticScope::Render,
                &mut entities,
            );
        }
        if let Some(root) = &template.root_fragment {
            collect_fragment(
                root,
                template,
                "root",
                TemplateSemanticScope::Render,
                &mut entities,
            );
        }
    }

    entities
}

fn collect_element(
    element: &ElementNode,
    template: &TemplateNode,
    path: &str,
    scope: TemplateSemanticScope,
    entities: &mut Vec<TemplateSemanticEntity>,
) {
    push_entity(
        entities,
        template,
        TemplateSemanticKind::Element,
        scope,
        "element",
        path,
        None,
        element.span,
    );
    entities
        .last_mut()
        .expect("element semantic entity was just inserted")
        .tag_name = Some(element.tag_name.clone());
    entities
        .last_mut()
        .expect("element semantic entity was just inserted")
        .tag_name_provenance = Some(SourceProvenance::new(
        &template.provenance.path,
        element.tag_name_span,
    ));

    for attribute in &element.attributes {
        let Some(span) = attribute.span else {
            continue;
        };
        let attribute_path = format!("{path}.{}", attribute.name);
        let (kind, expression) = match &attribute.value {
            AttributeValue::Binding { expression, .. } => (
                TemplateSemanticKind::AttributeBinding,
                Some(expression.clone()),
            ),
            AttributeValue::EventHandler { handler, .. } => {
                (TemplateSemanticKind::EventAttribute, Some(handler.clone()))
            }
            AttributeValue::BindingList(_) => continue,
            AttributeValue::Boolean | AttributeValue::Static(_) => {
                (TemplateSemanticKind::Attribute, None)
            }
        };
        push_entity(
            entities,
            template,
            kind,
            scope,
            match kind {
                TemplateSemanticKind::AttributeBinding => "attribute-binding",
                TemplateSemanticKind::EventAttribute => "event-attribute",
                _ => "attribute",
            },
            &attribute_path,
            expression,
            span,
        );
        entities
            .last_mut()
            .expect("attribute semantic entity was just inserted")
            .attribute_name = Some(attribute.name.clone());
    }

    collect_children(&element.children, template, path, scope, entities);
}

fn collect_fragment(
    fragment: &FragmentNode,
    template: &TemplateNode,
    path: &str,
    scope: TemplateSemanticScope,
    entities: &mut Vec<TemplateSemanticEntity>,
) {
    push_entity(
        entities,
        template,
        TemplateSemanticKind::Fragment,
        scope,
        "fragment",
        path,
        None,
        fragment.span,
    );
    collect_children(&fragment.children, template, path, scope, entities);
}

fn collect_children(
    children: &[TemplateChild],
    template: &TemplateNode,
    parent_path: &str,
    scope: TemplateSemanticScope,
    entities: &mut Vec<TemplateSemanticEntity>,
) {
    for (index, child) in children.iter().enumerate() {
        let path = format!("{parent_path}.{index}");
        match child {
            TemplateChild::Text { span, .. } => push_entity(
                entities,
                template,
                TemplateSemanticKind::Text,
                scope,
                "text",
                &path,
                None,
                *span,
            ),
            TemplateChild::Binding {
                expression, span, ..
            } => push_entity(
                entities,
                template,
                TemplateSemanticKind::Binding,
                scope,
                "binding",
                &path,
                Some(expression.clone()),
                *span,
            ),
            TemplateChild::Element(element) => {
                collect_element(element, template, &path, scope, entities);
            }
            TemplateChild::Fragment(fragment) => {
                collect_fragment(fragment, template, &path, scope, entities);
            }
            TemplateChild::Conditional(conditional) => {
                collect_conditional(conditional, template, &path, scope, entities);
            }
            TemplateChild::List(list) => collect_list(list, template, &path, scope, entities),
        }
    }
}

fn collect_conditional(
    conditional: &ConditionalNode,
    template: &TemplateNode,
    path: &str,
    scope: TemplateSemanticScope,
    entities: &mut Vec<TemplateSemanticEntity>,
) {
    push_entity(
        entities,
        template,
        TemplateSemanticKind::Conditional,
        scope,
        "conditional",
        path,
        Some(conditional.condition.clone()),
        conditional.span,
    );
    collect_children(
        &conditional.when_true,
        template,
        &format!("{path}.true"),
        scope,
        entities,
    );
    collect_children(
        &conditional.when_false,
        template,
        &format!("{path}.false"),
        scope,
        entities,
    );
}

fn collect_list(
    list: &ListNode,
    template: &TemplateNode,
    path: &str,
    scope: TemplateSemanticScope,
    entities: &mut Vec<TemplateSemanticEntity>,
) {
    push_entity(
        entities,
        template,
        TemplateSemanticKind::List,
        scope,
        "list",
        path,
        Some(list.iterable.clone()),
        list.span,
    );
    let entity = entities
        .last_mut()
        .expect("list semantic entity was just inserted");
    entity.list_item_variable = Some(list.item_variable.clone());
    entity.list_index_variable.clone_from(&list.index_variable);
    collect_children(
        &list.item_template,
        template,
        &format!("{path}.item"),
        TemplateSemanticScope::ListItem,
        entities,
    );
}

#[allow(clippy::too_many_arguments)]
fn push_entity(
    entities: &mut Vec<TemplateSemanticEntity>,
    template: &TemplateNode,
    kind: TemplateSemanticKind,
    scope: TemplateSemanticScope,
    id_kind: &str,
    path: &str,
    expression: Option<String>,
    span: presolve_parser::SourceSpan,
) {
    entities.push(TemplateSemanticEntity {
        id: template.id.template_entity(id_kind, path),
        owner: SemanticOwner::entity(template.id.clone()),
        kind,
        scope,
        tag_name: None,
        tag_name_provenance: None,
        attribute_name: None,
        list_item_variable: None,
        list_index_variable: None,
        expression,
        provenance: SourceProvenance::new(&template.provenance.path, span),
    });
}

#[cfg(test)]
mod tests {
    use super::{build_template_semantic_entities, TemplateSemanticKind};
    use crate::{
        build_component_graph_for_module, build_template_graph, SemanticId, SemanticOwner,
    };

    #[test]
    fn lowers_authored_template_descendants_and_bindings() {
        let parsed = presolve_parser::parse_file(
            "src/Panel.tsx",
            r#"
@component("x-panel")
class Panel extends Component {
  enabled = state(true);

  render() {
    return <section title="Panel" hidden={this.enabled}>{this.enabled}</section>;
  }
}
"#,
        );
        let graph = build_component_graph_for_module(&parsed);
        let templates = build_template_graph(&graph);
        let entities = build_template_semantic_entities(&templates.templates);
        let template =
            SemanticId::component_in_module("src/Panel.tsx", Some("x-panel"), "Panel").template();

        let attribute_binding = entities
            .iter()
            .find(|entity| {
                entity
                    .id
                    .as_str()
                    .ends_with("attribute-binding:root.hidden")
            })
            .expect("attribute binding entity");
        let text_binding = entities
            .iter()
            .find(|entity| entity.id.as_str().ends_with("binding:root.0"))
            .expect("text binding entity");

        assert!(entities.iter().any(|entity| {
            entity.kind == TemplateSemanticKind::Element
                && entity.id.as_str().ends_with("element:root")
        }));
        assert_eq!(
            attribute_binding.kind,
            TemplateSemanticKind::AttributeBinding
        );
        assert_eq!(
            attribute_binding.expression.as_deref(),
            Some("this.enabled")
        );
        assert_eq!(attribute_binding.provenance.span.line, 7);
        assert_eq!(text_binding.kind, TemplateSemanticKind::Binding);
        assert_eq!(text_binding.owner, SemanticOwner::entity(template));
    }
}
