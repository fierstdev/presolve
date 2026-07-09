use ezc_parser::{ParsedClass, ParsedFile, ParsedJsxChild};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComponentGraph {
    pub components: Vec<ComponentNode>,
    pub diagnostics: Vec<ComponentDiagnostic>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComponentNode {
    pub class_name: String,
    pub element_name: Option<String>,
    pub route_path: Option<String>,
    pub state_fields: Vec<StateField>,
    pub methods: Vec<ComponentMethod>,
    pub render: Option<RenderModel>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StateField {
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComponentMethod {
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RenderChild {
    Text(String),
    Binding(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderModel {
    pub root_element: Option<String>,
    pub attributes: Vec<String>,
    pub bindings: Vec<String>,
    pub event_handler_refs: Vec<String>,
    pub children: Vec<RenderChild>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComponentDiagnostic {
    pub code: String,
    pub message: String,
}

pub fn build_component_graph(parsed: &ParsedFile) -> ComponentGraph {
    let mut components = Vec::new();
    let mut diagnostics = Vec::new();

    for class in &parsed.classes {
        components.push(build_component_node(class, &mut diagnostics));
    }

    if parsed.classes.is_empty() && parsed.diagnostics.is_empty() {
        diagnostics.push(ComponentDiagnostic {
            code: "EZC1000".to_string(),
            message: "no component classes found".to_string(),
        });
    }

    ComponentGraph {
        components,
        diagnostics,
    }
}

fn build_component_node(
    class: &ParsedClass,
    diagnostics: &mut Vec<ComponentDiagnostic>,
) -> ComponentNode {
    let element_name = decorator_argument(class, "component");
    let route_path = decorator_argument(class, "route");

    if element_name.is_none() {
        diagnostics.push(ComponentDiagnostic {
            code: "EZC1001".to_string(),
            message: format!("class `{}` is missing @component(...)", class.name),
        });
    }

    let state_fields = class
        .properties
        .iter()
        .filter(|property| property.initializer.as_deref() == Some("state(...)"))
        .map(|property| StateField {
            name: property.name.clone(),
        })
        .collect::<Vec<_>>();

    let methods = class
        .methods
        .iter()
        .map(|method| ComponentMethod {
            name: method.name.clone(),
        })
        .collect::<Vec<_>>();

    let render = class
        .methods
        .iter()
        .find(|method| method.name == "render")
        .map(|method| {
            let root = method.jsx_roots.first();

            RenderModel {
                root_element: root.map(|jsx| jsx.name.clone()),
                attributes: root.map(|jsx| jsx.attributes.clone()).unwrap_or_default(),
                event_handler_refs: root
                    .map(|jsx| jsx.event_handler_refs.clone())
                    .unwrap_or_default(),
                children: root
                    .map(|jsx| {
                        jsx.children
                            .iter()
                            .map(render_child_from_parsed)
                            .collect::<Vec<_>>()
                    })
                    .unwrap_or_default(),
                bindings: method.bindings.clone(),
            }
        });

    if render.is_none() {
        diagnostics.push(ComponentDiagnostic {
            code: "EZC1002".to_string(),
            message: format!("class `{}` is missing render()", class.name),
        });
    }

    if let Some(render) = &render {
        let property_names = class
            .properties
            .iter()
            .map(|property| property.name.as_str())
            .collect::<Vec<_>>();

        let method_names = class
            .methods
            .iter()
            .map(|method| method.name.as_str())
            .collect::<Vec<_>>();

        for binding in &render.bindings {
            if let Some(name) = this_member_name(binding) {
                if !property_names.contains(&name) {
                    diagnostics.push(ComponentDiagnostic {
                        code: "EZC1003".to_string(),
                        message: format!(
                            "render binding `{binding}` references unknown field `{name}` in class `{}`",
                            class.name
                        ),
                    });
                }
            }
        }

        for handler in &render.event_handler_refs {
            if let Some(name) = this_member_name(handler) {
                if !method_names.contains(&name) {
                    diagnostics.push(ComponentDiagnostic {
                        code: "EZC1004".to_string(),
                        message: format!(
                            "event handler `{handler}` references unknown method `{name}` in class `{}`",
                            class.name
                        ),
                    });
                }
            }
        }
    }

    ComponentNode {
        class_name: class.name.clone(),
        element_name,
        route_path,
        state_fields,
        methods,
        render,
    }
}

fn decorator_argument(class: &ParsedClass, name: &str) -> Option<String> {
    class
        .decorators
        .iter()
        .find(|decorator| decorator.name == name)
        .and_then(|decorator| decorator.argument.clone())
}

fn render_child_from_parsed(child: &ParsedJsxChild) -> RenderChild {
    match child {
        ParsedJsxChild::Text(text) => RenderChild::Text(text.clone()),
        ParsedJsxChild::Binding(binding) => RenderChild::Binding(binding.clone()),
    }
}

fn this_member_name(reference: &str) -> Option<&str> {
    reference.strip_prefix("this.")
}
