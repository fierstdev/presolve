use ezc_parser::{ParsedClass, ParsedFile};

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
pub struct RenderModel {
    pub root_element: Option<String>,
    pub attributes: Vec<String>,
    pub bindings: Vec<String>,
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
        .map(|method| RenderModel {
            root_element: method.jsx_roots.first().map(|jsx| jsx.name.clone()),
            attributes: method
                .jsx_roots
                .first()
                .map(|jsx| jsx.attributes.clone())
                .unwrap_or_default(),
            bindings: method.bindings.clone(),
        });

    if render.is_none() {
        diagnostics.push(ComponentDiagnostic {
            code: "EZC1002".to_string(),
            message: format!("class `{}` is missing render()", class.name),
        });
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
