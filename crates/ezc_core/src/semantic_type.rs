use std::collections::BTreeMap;

use crate::SemanticId;

/// Compiler-owned semantic type algebra independent of TypeScript spelling.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SemanticType {
    Unknown,
    Never,
    Null,
    Boolean,
    Number,
    String,
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
    pub assignments: BTreeMap<SemanticId, SemanticType>,
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::{ObjectType, SemanticType};

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
            SemanticType::Array(Box::new(SemanticType::Number)),
            todo,
            SemanticType::Union(vec![SemanticType::String, SemanticType::Null]),
        ];

        assert_eq!(types.len(), 9);
    }
}
