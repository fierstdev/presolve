use std::collections::BTreeMap;
use std::fmt;

use crate::{ComponentNode, SemanticId, SourceProvenance};

/// Compiler-owned semantic type algebra independent of TypeScript spelling.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SemanticType {
    Unknown,
    Never,
    Null,
    Boolean,
    Number,
    String,
    BooleanLiteral(bool),
    NumberLiteral(String),
    StringLiteral(String),
    Array(Box<SemanticType>),
    Object(ObjectType),
    Union(Vec<SemanticType>),
}

/// Structural object shape in the canonical semantic type algebra.
///
/// C1 establishes only the representation. Object declaration lowering,
/// property provenance, and member resolution are later Phase C slices.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ObjectType {
    pub properties: BTreeMap<String, SemanticType>,
}

/// Canonical type assignments owned by the application semantic model.
///
/// C1 provides the stable container but deliberately populates no assignments.
/// State inference, expression propagation, and typed declarations are added by
/// later Phase C slices.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SemanticTypeModel {
    pub assignments: BTreeMap<SemanticId, SemanticTypeAssignment>,
}

/// Stable identity for one compiler-owned type assignment.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SemanticTypeId(SemanticId);

impl SemanticTypeId {
    #[must_use]
    pub fn for_subject(subject: &SemanticId) -> Self {
        Self(subject.semantic_type())
    }

    #[must_use]
    pub fn as_semantic_id(&self) -> &SemanticId {
        &self.0
    }
}

impl fmt::Display for SemanticTypeId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(formatter)
    }
}

/// Whether a semantic type came from an authored declaration or compiler inference.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SemanticTypeStatus {
    Declared,
    Inferred,
}

/// A type assignment with canonical identity, semantic origin, and authored location.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SemanticTypeAssignment {
    pub id: SemanticTypeId,
    pub subject: SemanticId,
    pub semantic_type: SemanticType,
    pub origin: SemanticId,
    pub status: SemanticTypeStatus,
    pub provenance: SourceProvenance,
}

impl SemanticTypeModel {
    #[must_use]
    pub fn from_components(components: &[ComponentNode]) -> Self {
        let mut assignments = BTreeMap::new();

        for field in components
            .iter()
            .flat_map(|component| &component.state_fields)
        {
            let Some(declared_type) = &field.declared_type else {
                continue;
            };
            let Some(semantic_type) = semantic_type_from_annotation(&declared_type.text) else {
                continue;
            };
            assignments.insert(
                field.id.clone(),
                SemanticTypeAssignment {
                    id: SemanticTypeId::for_subject(&field.id),
                    subject: field.id.clone(),
                    semantic_type,
                    origin: field.id.clone(),
                    status: SemanticTypeStatus::Declared,
                    provenance: declared_type.provenance.clone(),
                },
            );
        }

        Self { assignments }
    }
}

fn semantic_type_from_annotation(text: &str) -> Option<SemanticType> {
    match text {
        "string" => Some(SemanticType::String),
        "number" => Some(SemanticType::Number),
        "boolean" => Some(SemanticType::Boolean),
        "null" => Some(SemanticType::Null),
        "true" => Some(SemanticType::BooleanLiteral(true)),
        "false" => Some(SemanticType::BooleanLiteral(false)),
        _ => string_literal_type(text).or_else(|| numeric_literal_type(text)),
    }
}

fn string_literal_type(text: &str) -> Option<SemanticType> {
    let quote = text.chars().next()?;
    (matches!(quote, '\'' | '"') && text.ends_with(quote) && text.len() >= 2)
        .then(|| SemanticType::StringLiteral(text[1..text.len() - 1].to_string()))
}

fn numeric_literal_type(text: &str) -> Option<SemanticType> {
    text.parse::<f64>()
        .ok()
        .filter(|value| value.is_finite())
        .map(|_| SemanticType::NumberLiteral(text.to_string()))
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::{
        ObjectType, SemanticType, SemanticTypeAssignment, SemanticTypeId, SemanticTypeStatus,
    };
    use crate::{SemanticId, SourceProvenance};

    #[test]
    fn represents_core_compiler_owned_type_forms() {
        let todo = SemanticType::Object(ObjectType {
            properties: BTreeMap::from([
                ("completed".to_string(), SemanticType::Boolean),
                ("title".to_string(), SemanticType::String),
            ]),
        });
        let types = vec![
            SemanticType::Unknown,
            SemanticType::Never,
            SemanticType::Null,
            SemanticType::Boolean,
            SemanticType::Number,
            SemanticType::String,
            SemanticType::BooleanLiteral(true),
            SemanticType::NumberLiteral("42".to_string()),
            SemanticType::StringLiteral("all".to_string()),
            SemanticType::Array(Box::new(SemanticType::Number)),
            todo,
            SemanticType::Union(vec![SemanticType::String, SemanticType::Null]),
        ];

        assert_eq!(types.len(), 12);
    }

    #[test]
    fn preserves_type_assignment_identity_status_and_provenance() {
        let subject = SemanticId::component(Some("x-counter"), "Counter").state_field("count");
        let assignment = SemanticTypeAssignment {
            id: SemanticTypeId::for_subject(&subject),
            subject: subject.clone(),
            semantic_type: SemanticType::Number,
            origin: subject.clone(),
            status: SemanticTypeStatus::Declared,
            provenance: SourceProvenance::new(
                "src/Counter.tsx",
                ezc_parser::SourceSpan {
                    start: 42,
                    end: 48,
                    line: 4,
                    column: 10,
                },
            ),
        };

        assert_eq!(
            assignment.id.to_string(),
            "component:x-counter/state:count/type:semantic"
        );
        assert_eq!(assignment.origin, subject);
        assert_eq!(assignment.status, SemanticTypeStatus::Declared);
        assert_eq!(
            assignment.provenance.path,
            std::path::Path::new("src/Counter.tsx")
        );
    }
}
