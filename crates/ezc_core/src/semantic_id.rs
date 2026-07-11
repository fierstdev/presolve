use std::fmt;

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

impl SemanticId {
    #[must_use]
    pub fn component(element_name: Option<&str>, class_name: &str) -> Self {
        Self(format!("component:{}", element_name.unwrap_or(class_name)))
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
    pub fn template(&self) -> Self {
        self.child("template", "render")
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    fn child(&self, kind: &str, name: &str) -> Self {
        Self(format!("{}/{kind}:{name}", self.0))
    }
}

impl fmt::Display for SemanticId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(formatter)
    }
}

#[cfg(test)]
mod tests {
    use super::SemanticId;

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
            component.template().as_str(),
            "component:x-counter/template:render"
        );
    }

    #[test]
    fn falls_back_to_class_name_for_invalid_components() {
        assert_eq!(
            SemanticId::component(None, "MissingDecorator").as_str(),
            "component:MissingDecorator"
        );
    }
}
