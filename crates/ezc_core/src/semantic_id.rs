use std::fmt;
use std::path::{Component, Path};

use serde::Serialize;

/// Globally stable identity for a compiler semantic entity.
///
/// IDs use the component element name when available because it is the
/// application-facing component identity. Invalid components without an
/// element declaration fall back to their class name so diagnostics and later
/// ASM validation can still refer to them deterministically.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(transparent)]
pub struct SemanticId(String);

/// Direct owner of a semantic entity within one compiled application.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SemanticOwner {
    Application,
    Entity(SemanticId),
}

impl SemanticOwner {
    #[must_use]
    pub fn entity(id: SemanticId) -> Self {
        Self::Entity(id)
    }

    #[must_use]
    pub fn entity_id(&self) -> Option<&SemanticId> {
        match self {
            Self::Application => None,
            Self::Entity(id) => Some(id),
        }
    }
}

impl SemanticId {
    #[must_use]
    pub fn component(element_name: Option<&str>, class_name: &str) -> Self {
        Self(format!("component:{}", element_name.unwrap_or(class_name)))
    }

    #[must_use]
    pub fn component_in_module(
        module_path: impl AsRef<Path>,
        element_name: Option<&str>,
        class_name: &str,
    ) -> Self {
        Self(format!(
            "module:{}/component:{}",
            normalized_module_path(module_path.as_ref()),
            element_name.unwrap_or(class_name)
        ))
    }

    #[must_use]
    pub fn state_field(&self, name: &str) -> Self {
        self.child("state", name)
    }

    #[must_use]
    pub fn method(&self, name: &str) -> Self {
        self.child("method", name)
    }

    #[must_use]
    pub fn action(&self, method: &str, index: usize) -> Self {
        self.child("action", &format!("{method}:{index}"))
    }

    #[must_use]
    pub fn local_variable(&self, name: &str, index: usize) -> Self {
        self.child("local", &format!("{name}:{index}"))
    }

    #[must_use]
    pub fn expression(&self, path: &str) -> Self {
        self.child("expression", path)
    }

    #[must_use]
    pub fn event_handler(&self, event: &str, index: usize) -> Self {
        self.child("event", &format!("{event}:{index}"))
    }

    #[must_use]
    pub fn template(&self) -> Self {
        self.child("template", "render")
    }

    #[must_use]
    pub fn template_entity(&self, kind: &str, path: &str) -> Self {
        self.child(kind, path)
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    fn child(&self, kind: &str, name: &str) -> Self {
        Self(format!("{}/{kind}:{name}", self.0))
    }
}

fn normalized_module_path(path: &Path) -> String {
    let mut segments = Vec::new();
    let absolute = path.is_absolute();

    for component in path.components() {
        match component {
            Component::CurDir | Component::RootDir | Component::Prefix(_) => {}
            Component::ParentDir => {
                if segments.last().is_some_and(|segment| *segment != "..") {
                    segments.pop();
                } else {
                    segments.push("..".to_string());
                }
            }
            Component::Normal(segment) => segments.push(segment.to_string_lossy().into_owned()),
        }
    }

    let path = segments.join("/");
    if absolute {
        format!("/{path}")
    } else {
        path
    }
}

impl fmt::Display for SemanticId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(formatter)
    }
}

#[cfg(test)]
mod tests {
    use super::{SemanticId, SemanticOwner};

    #[test]
    fn derives_component_scoped_ids() {
        let component = SemanticId::component(Some("x-counter"), "Counter");

        assert_eq!(component.as_str(), "component:x-counter");
        assert_eq!(
            component.state_field("count").as_str(),
            "component:x-counter/state:count"
        );
        assert_eq!(
            component.method("increment").as_str(),
            "component:x-counter/method:increment"
        );
        assert_eq!(
            component.action("increment", 0).as_str(),
            "component:x-counter/action:increment:0"
        );
        assert_eq!(
            component
                .method("render")
                .local_variable("title", 0)
                .as_str(),
            "component:x-counter/method:render/local:title:0"
        );
        assert_eq!(
            component.event_handler("click", 0).as_str(),
            "component:x-counter/event:click:0"
        );
        assert_eq!(
            component.template().as_str(),
            "component:x-counter/template:render"
        );
    }

    #[test]
    fn derives_module_qualified_component_ids() {
        let component =
            SemanticId::component_in_module("src/../src/Counter.tsx", Some("x-counter"), "Counter");

        assert_eq!(
            component.as_str(),
            "module:src/Counter.tsx/component:x-counter"
        );
        assert_eq!(
            component.state_field("count").as_str(),
            "module:src/Counter.tsx/component:x-counter/state:count"
        );
    }

    #[test]
    fn falls_back_to_class_name_for_invalid_components() {
        assert_eq!(
            SemanticId::component(None, "MissingDecorator").as_str(),
            "component:MissingDecorator"
        );
    }

    #[test]
    fn distinguishes_application_roots_from_entity_owners() {
        let component = SemanticId::component(Some("x-counter"), "Counter");

        assert_eq!(SemanticOwner::Application.entity_id(), None);
        assert_eq!(
            SemanticOwner::entity(component.clone()).entity_id(),
            Some(&component)
        );
    }
}
