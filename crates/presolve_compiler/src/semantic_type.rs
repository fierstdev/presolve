use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::path::PathBuf;

use crate::{
    BindingTable, CapabilityOperationId, CapabilityOperationKind, CapabilityParameters,
    CapabilityValueContract, ComponentNode, ComputedValue, ConsumerEntity, ContextEntity,
    DeclaredStateType, Effect, EffectStatement, EffectStatementKind, ExpressionGraph,
    ExpressionNodeKind, ImportBindingTarget, ProviderEntity, SemanticId, SerializableValue,
    SlotEntity, SourceProvenance, SymbolKind, EFFECT_CAPABILITY_REGISTRY,
};
use crate::{
    SemanticReference, SemanticReferenceKind, TemplateSemanticEntity, TemplateSemanticKind,
};
use presolve_parser::{ParsedFile, ParsedTypeAlias};

/// A compiler-owned operator category used by semantic expression typing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SemanticOperator {
    Arithmetic(crate::ArithmeticOperator),
    Comparison(crate::ComparisonOperator),
    Logical(crate::LogicalOperator),
    NullishCoalescing,
    Unary(crate::component_graph::UnaryOperator),
}

/// Whether a supported DOM binding writes an HTML attribute or a DOM property.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DomBindingKind {
    Attribute,
    Property,
}

/// Compiler-owned type contract for one supported DOM binding name.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DomBindingContract {
    pub name: &'static str,
    pub kind: DomBindingKind,
    pub semantic_type: SemanticType,
}

/// Returns the contract for the currently supported dynamic DOM bindings.
#[must_use]
pub fn dom_binding_contract(name: &str) -> Option<DomBindingContract> {
    match name {
        "disabled" => Some(DomBindingContract {
            name: "disabled",
            kind: DomBindingKind::Property,
            semantic_type: SemanticType::Boolean,
        }),
        "href" => Some(DomBindingContract {
            name: "href",
            kind: DomBindingKind::Attribute,
            semantic_type: SemanticType::String,
        }),
        "class" | "for" => Some(DomBindingContract {
            name: if name == "class" { "class" } else { "for" },
            kind: DomBindingKind::Attribute,
            semantic_type: SemanticType::String,
        }),
        "value" => Some(DomBindingContract {
            name: "value",
            kind: DomBindingKind::Property,
            semantic_type: SemanticType::Union(vec![SemanticType::String, SemanticType::Number]),
        }),
        _ => None,
    }
}

/// Compiler-owned semantic type algebra independent of TypeScript spelling.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SemanticType {
    Unknown,
    Never,
    Null,
    Boolean,
    Number,
    String,
    /// Nominal compile-time marker accepted only by canonical Form declarations.
    Form,
    /// Nominal built-in type accepted only by canonical Slot declarations.
    SlotContent,
    BooleanLiteral(bool),
    NumberLiteral(String),
    StringLiteral(String),
    Array(Box<SemanticType>),
    Tuple(Vec<SemanticType>),
    Object(ObjectType),
    Union(Vec<SemanticType>),
    Resource(ResourceType),
}

/// Renders a canonical semantic type using the stable inspection spelling.
#[must_use]
pub fn semantic_type_text(semantic_type: &SemanticType) -> String {
    match semantic_type {
        SemanticType::Unknown => "unknown".to_string(),
        SemanticType::Never => "never".to_string(),
        SemanticType::Null => "null".to_string(),
        SemanticType::Boolean => "boolean".to_string(),
        SemanticType::Number => "number".to_string(),
        SemanticType::String => "string".to_string(),
        SemanticType::Form => "Form".to_string(),
        SemanticType::SlotContent => "SlotContent".to_string(),
        SemanticType::BooleanLiteral(value) => value.to_string(),
        SemanticType::NumberLiteral(value) => value.clone(),
        SemanticType::StringLiteral(value) => format!("{value:?}"),
        SemanticType::Array(element) => format!("{}[]", semantic_type_text(element)),
        SemanticType::Tuple(items) => format!(
            "[{}]",
            items
                .iter()
                .map(semantic_type_text)
                .collect::<Vec<_>>()
                .join(", ")
        ),
        SemanticType::Object(object) => format!(
            "{{ {} }}",
            object
                .properties
                .iter()
                .map(|(name, semantic_type)| format!(
                    "{name}: {}",
                    semantic_type_text(semantic_type)
                ))
                .collect::<Vec<_>>()
                .join("; ")
        ),
        SemanticType::Union(members) => members
            .iter()
            .map(semantic_type_text)
            .collect::<Vec<_>>()
            .join(" | "),
        SemanticType::Resource(resource) => format!(
            "Resource<{}, {}>",
            semantic_type_text(&resource.data),
            semantic_type_text(&resource.error)
        ),
    }
}

/// Module-scoped authority for compiler-owned built-in marker types.
///
/// The parser supplies binding facts; this authority decides whether an exact
/// authored marker spelling resolves to the compiler built-in rather than a
/// module-local declaration or import. Downstream stages consume the resulting
/// canonical entity and never repeat source-text recognition.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BuiltinTypeAuthority {
    form_marker_is_unshadowed: bool,
}

impl BuiltinTypeAuthority {
    #[must_use]
    pub fn for_file(parsed: &ParsedFile) -> Self {
        let has_local_binding = parsed
            .local_type_bindings
            .iter()
            .any(|binding| binding == "Form");
        let has_import_binding = parsed.imports.iter().any(|import| {
            import
                .specifiers
                .iter()
                .any(|specifier| specifier.local == "Form")
        });
        Self {
            form_marker_is_unshadowed: !has_local_binding && !has_import_binding,
        }
    }

    #[must_use]
    pub fn recognizes_form_marker(self, type_text: &str) -> bool {
        self.form_marker_is_unshadowed && type_text == "Form"
    }
}

/// Compiler-owned resource contract with explicit data, error, and boundary semantics.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResourceType {
    pub data: Box<SemanticType>,
    pub error: Box<SemanticType>,
    pub pending: bool,
    pub serializable: bool,
    pub execution_boundary: ResourceExecutionBoundary,
}

/// Execution boundary declared by a canonical resource contract.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResourceExecutionBoundary {
    Client,
    Server,
    Shared,
}

/// Whether a canonical semantic type can safely cross a compiler/runtime boundary.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SerializationCompatibility {
    Serializable,
    NotSerializable,
}

/// Execution side participating in a compiler/runtime boundary crossing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExecutionBoundary {
    Client,
    Server,
}

/// Whether a semantic type may cross one execution boundary to another.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BoundaryCompatibility {
    Compatible,
    Incompatible,
}

/// Determines whether a type can cross from one execution boundary to another.
#[must_use]
pub fn boundary_compatibility(
    semantic_type: &SemanticType,
    source: ExecutionBoundary,
    target: ExecutionBoundary,
) -> BoundaryCompatibility {
    if source == target {
        return BoundaryCompatibility::Compatible;
    }
    let compatible = match semantic_type {
        SemanticType::Resource(resource) => {
            resource.execution_boundary == ResourceExecutionBoundary::Shared
                && serialization_compatibility(semantic_type)
                    == SerializationCompatibility::Serializable
        }
        _ => serialization_compatibility(semantic_type) == SerializationCompatibility::Serializable,
    };
    if compatible {
        BoundaryCompatibility::Compatible
    } else {
        BoundaryCompatibility::Incompatible
    }
}

/// Canonicalizes equivalent semantic type forms for stable equality and output.
#[must_use]
pub fn normalize_semantic_type(semantic_type: SemanticType) -> SemanticType {
    match semantic_type {
        SemanticType::Array(element) => {
            SemanticType::Array(Box::new(normalize_semantic_type(*element)))
        }
        SemanticType::Tuple(items) => {
            SemanticType::Tuple(items.into_iter().map(normalize_semantic_type).collect())
        }
        SemanticType::Object(object) => SemanticType::Object(ObjectType {
            properties: object
                .properties
                .into_iter()
                .map(|(name, semantic_type)| (name, normalize_semantic_type(semantic_type)))
                .collect(),
        }),
        SemanticType::Union(members) => normalize_union(members),
        SemanticType::Resource(resource) => SemanticType::Resource(ResourceType {
            data: Box::new(normalize_semantic_type(*resource.data)),
            error: Box::new(normalize_semantic_type(*resource.error)),
            pending: resource.pending,
            serializable: resource.serializable,
            execution_boundary: resource.execution_boundary,
        }),
        semantic_type => semantic_type,
    }
}

fn normalize_union(members: Vec<SemanticType>) -> SemanticType {
    let mut flattened = Vec::new();
    for member in members {
        match normalize_semantic_type(member) {
            SemanticType::Union(items) => flattened.extend(items),
            SemanticType::Never => {}
            member => flattened.push(member),
        }
    }
    flattened.sort_by_key(semantic_type_sort_key);
    flattened.dedup();
    match flattened.as_slice() {
        [] => SemanticType::Never,
        [member] => member.clone(),
        _ => SemanticType::Union(flattened),
    }
}

fn semantic_type_sort_key(semantic_type: &SemanticType) -> String {
    format!("{semantic_type:?}")
}

/// Determines structural serialization compatibility for a canonical type.
#[must_use]
pub fn serialization_compatibility(semantic_type: &SemanticType) -> SerializationCompatibility {
    let serializable = match semantic_type {
        SemanticType::Unknown
        | SemanticType::Never
        | SemanticType::Form
        | SemanticType::SlotContent => false,
        SemanticType::Null
        | SemanticType::Boolean
        | SemanticType::Number
        | SemanticType::String
        | SemanticType::BooleanLiteral(_)
        | SemanticType::NumberLiteral(_)
        | SemanticType::StringLiteral(_) => true,
        SemanticType::Array(element) => {
            serialization_compatibility(element) == SerializationCompatibility::Serializable
        }
        SemanticType::Tuple(items) | SemanticType::Union(items) => items.iter().all(|item| {
            serialization_compatibility(item) == SerializationCompatibility::Serializable
        }),
        SemanticType::Object(object) => object.properties.values().all(|property| {
            serialization_compatibility(property) == SerializationCompatibility::Serializable
        }),
        SemanticType::Resource(resource) => {
            resource.serializable
                && serialization_compatibility(&resource.data)
                    == SerializationCompatibility::Serializable
                && serialization_compatibility(&resource.error)
                    == SerializationCompatibility::Serializable
        }
    };
    if serializable {
        SerializationCompatibility::Serializable
    } else {
        SerializationCompatibility::NotSerializable
    }
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
    pub aliases: BTreeMap<SemanticId, SemanticTypeAlias>,
    pub list_scopes: BTreeMap<SemanticId, ListTemplateScopeType>,
    pub member_accesses: BTreeMap<SemanticId, MemberAccessType>,
    pub computed_values: BTreeMap<SemanticId, ComputedValueType>,
    pub action_signatures: BTreeMap<SemanticId, ActionSignatureType>,
    pub effect_statements: BTreeMap<SemanticId, EffectStatementTypeRecord>,
}

/// Compatibility facts recorded by effect typing without issuing F5 diagnostics.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EffectCompatibility {
    Compatible,
    Incompatible,
    Unknown,
}

/// The compiler-owned classification of one authored effect statement.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EffectOperationClassification {
    RecognizedCapability,
    UnknownExternalCapability,
    ReactiveStateAssignment,
    ComponentActionCall,
    ComponentEffectCall,
    ComponentMethodCall,
    UnresolvedComponentCall,
    UnresolvedComponentAssignment,
    BareReturn,
    ValueReturn,
    Empty,
    UnsupportedStatement,
}

/// Immutable F4 typing and compatibility facts keyed by effect statement identity.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EffectStatementTypeRecord {
    pub statement: SemanticId,
    pub operation_classification: EffectOperationClassification,
    pub operand_types: Vec<SemanticType>,
    pub target_type: Option<SemanticType>,
    pub signature_compatibility: EffectCompatibility,
    pub boundary_compatibility: EffectCompatibility,
    pub serialization_compatibility: EffectCompatibility,
    pub capability_operation: Option<CapabilityOperationId>,
    pub provenance: SourceProvenance,
}

/// Canonical item and index type information for one template list scope.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ListTemplateScopeType {
    pub list: SemanticId,
    pub item_name: String,
    pub item_type: SemanticType,
    pub index_name: Option<String>,
    pub index_type: Option<SemanticType>,
}

/// Canonical result of one supported template list-item member access.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MemberAccessType {
    pub entity: SemanticId,
    pub expression: String,
    pub semantic_type: Option<SemanticType>,
}

/// Canonical semantic type contract for one computed getter.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComputedValueType {
    pub computed: SemanticId,
    pub method: SemanticId,
    pub semantic_type: SemanticType,
    pub status: SemanticTypeStatus,
    pub declared_return_type: Option<SemanticType>,
    pub declared_return_compatible: Option<bool>,
    pub serialization: SerializationCompatibility,
    pub execution_boundary: ExecutionBoundary,
    pub boundary_compatibility: BoundaryCompatibility,
    pub provenance: SourceProvenance,
}

/// Canonical input/output type contract for one decorator-marked action method.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActionSignatureType {
    pub method: SemanticId,
    pub input: Vec<(SemanticId, SemanticType)>,
    pub output: Option<SemanticType>,
    pub is_async: bool,
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

/// Stable families for diagnostics produced from canonical semantic types.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TypeDiagnosticFamily {
    UnknownType,
    InvalidOperator,
    IncompatibleAssignment,
    MissingMember,
    InvalidCondition,
    NonIterableList,
    NonRenderableValue,
    InvalidBinding,
    NonSerializableState,
}

/// Stable codes for diagnostics produced from canonical semantic types.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TypeDiagnosticCode {
    IncompatibleStateInitializer,
    IncompatibleAssignment,
    InvalidToggleTarget,
    InvalidNumericMutationTarget,
    InvalidCompoundMutationTarget,
    InvalidCompoundMutationOperand,
    InvalidArithmeticOperator,
    InvalidComparisonOperator,
    InvalidLogicalOperator,
    InvalidNullishOperator,
    InvalidUnaryOperator,
    NonRenderableValue,
    InvalidBinding,
    InvalidCondition,
    NonIterableList,
    MissingMember,
    UnknownType,
    NonSerializableState,
}

impl TypeDiagnosticCode {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::IncompatibleStateInitializer => "PSC1016",
            Self::IncompatibleAssignment => "PSC1017",
            Self::InvalidToggleTarget => "PSC1018",
            Self::InvalidNumericMutationTarget => "PSC1019",
            Self::InvalidCompoundMutationTarget => "PSC1020",
            Self::InvalidCompoundMutationOperand => "PSC1021",
            Self::InvalidArithmeticOperator => "PSC1022",
            Self::InvalidComparisonOperator => "PSC1023",
            Self::InvalidLogicalOperator => "PSC1024",
            Self::InvalidNullishOperator => "PSC1025",
            Self::InvalidUnaryOperator => "PSC1026",
            Self::NonRenderableValue => "PSC1027",
            Self::InvalidBinding => "PSC1028",
            Self::InvalidCondition => "PSC1029",
            Self::NonIterableList => "PSC1030",
            Self::MissingMember => "PSC1031",
            Self::UnknownType => "PSC1032",
            Self::NonSerializableState => "PSC1033",
        }
    }

    #[must_use]
    pub const fn family(self) -> TypeDiagnosticFamily {
        match self {
            Self::UnknownType => TypeDiagnosticFamily::UnknownType,
            Self::InvalidArithmeticOperator
            | Self::InvalidComparisonOperator
            | Self::InvalidLogicalOperator
            | Self::InvalidNullishOperator
            | Self::InvalidUnaryOperator => TypeDiagnosticFamily::InvalidOperator,
            Self::IncompatibleStateInitializer
            | Self::IncompatibleAssignment
            | Self::InvalidToggleTarget
            | Self::InvalidNumericMutationTarget
            | Self::InvalidCompoundMutationTarget
            | Self::InvalidCompoundMutationOperand => TypeDiagnosticFamily::IncompatibleAssignment,
            Self::MissingMember => TypeDiagnosticFamily::MissingMember,
            Self::InvalidCondition => TypeDiagnosticFamily::InvalidCondition,
            Self::NonIterableList => TypeDiagnosticFamily::NonIterableList,
            Self::NonRenderableValue => TypeDiagnosticFamily::NonRenderableValue,
            Self::InvalidBinding => TypeDiagnosticFamily::InvalidBinding,
            Self::NonSerializableState => TypeDiagnosticFamily::NonSerializableState,
        }
    }
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

/// A named authored type alias resolved to canonical semantic type meaning.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SemanticTypeAlias {
    pub id: SemanticId,
    pub name: String,
    pub semantic_type: SemanticType,
    pub provenance: SourceProvenance,
}

/// Canonical result of resolving one authored type annotation through the
/// existing type and binding authorities.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedDeclaredSemanticType {
    pub semantic_type: SemanticType,
    pub alias_origin: Option<SemanticId>,
}

impl SemanticTypeModel {
    #[must_use]
    pub fn resolve_declared_type(
        &self,
        declared_type: &DeclaredStateType,
        bindings: Option<&BindingTable>,
    ) -> Option<ResolvedDeclaredSemanticType> {
        let local_alias = self.aliases.values().find(|alias| {
            alias.provenance.path == declared_type.provenance.path
                && alias.name == declared_type.text
        });
        let imported_alias = bindings
            .and_then(|bindings| {
                bindings.resolve_import(&declared_type.provenance.path, &declared_type.text)
            })
            .and_then(|binding| match &binding.target {
                ImportBindingTarget::Symbol(symbol) if symbol.kind == SymbolKind::TypeAlias => {
                    self.aliases.get(&symbol.id)
                }
                _ => None,
            });
        let alias = local_alias.or(imported_alias);
        let semantic_type = alias
            .map(|alias| alias.semantic_type.clone())
            .or_else(|| semantic_type_from_annotation(&declared_type.text))?;
        Some(ResolvedDeclaredSemanticType {
            semantic_type: normalize_semantic_type(semantic_type),
            alias_origin: alias.map(|alias| alias.id.clone()),
        })
    }

    #[must_use]
    pub fn from_components(
        components: &[ComponentNode],
        provenance: &BTreeMap<SemanticId, SourceProvenance>,
    ) -> Self {
        Self::from_components_with_aliases_and_bindings(components, provenance, &[], None)
    }

    #[must_use]
    pub fn from_components_with_aliases(
        components: &[ComponentNode],
        provenance: &BTreeMap<SemanticId, SourceProvenance>,
        parsed_aliases: &[(PathBuf, ParsedTypeAlias)],
    ) -> Self {
        Self::from_components_with_aliases_and_bindings(
            components,
            provenance,
            parsed_aliases,
            None,
        )
    }

    #[must_use]
    pub fn from_components_with_aliases_and_bindings(
        components: &[ComponentNode],
        provenance: &BTreeMap<SemanticId, SourceProvenance>,
        parsed_aliases: &[(PathBuf, ParsedTypeAlias)],
        bindings: Option<&BindingTable>,
    ) -> Self {
        let aliases = parsed_aliases
            .iter()
            .filter_map(|(path, alias)| {
                semantic_type_from_annotation(&alias.type_text).map(|semantic_type| {
                    let id = SemanticId::type_alias_in_module(path, &alias.name);
                    (
                        id.clone(),
                        SemanticTypeAlias {
                            id,
                            name: alias.name.clone(),
                            semantic_type,
                            provenance: SourceProvenance::new(path, alias.type_span),
                        },
                    )
                })
            })
            .collect::<BTreeMap<_, _>>();
        let aliases_by_path_and_name = aliases
            .values()
            .map(|alias| ((alias.provenance.path.clone(), alias.name.clone()), alias))
            .collect::<BTreeMap<_, _>>();
        let mut assignments = BTreeMap::new();

        for field in components
            .iter()
            .flat_map(|component| &component.state_fields)
        {
            let (semantic_type, origin, status, assignment_provenance) =
                if let Some(declared_type) = &field.declared_type {
                    let alias = aliases_by_path_and_name
                        .get(&(
                            declared_type.provenance.path.clone(),
                            declared_type.text.clone(),
                        ))
                        .copied();
                    let imported_alias = bindings
                        .and_then(|bindings| {
                            bindings
                                .resolve_import(&declared_type.provenance.path, &declared_type.text)
                        })
                        .and_then(|binding| match &binding.target {
                            ImportBindingTarget::Symbol(symbol)
                                if symbol.kind == SymbolKind::TypeAlias =>
                            {
                                aliases.get(&symbol.id)
                            }
                            _ => None,
                        });
                    let alias = alias.or(imported_alias);
                    let semantic_type = alias
                        .map(|alias| alias.semantic_type.clone())
                        .or_else(|| semantic_type_from_annotation(&declared_type.text));
                    let Some(semantic_type) = semantic_type else {
                        continue;
                    };
                    (
                        semantic_type,
                        alias.map_or_else(|| field.id.clone(), |alias| alias.id.clone()),
                        SemanticTypeStatus::Declared,
                        declared_type.provenance.clone(),
                    )
                } else {
                    let Some(value) = &field.initial_value else {
                        continue;
                    };
                    let Some(field_provenance) = provenance.get(&field.id) else {
                        continue;
                    };
                    (
                        infer_serializable_value_type(value),
                        field.id.clone(),
                        SemanticTypeStatus::Inferred,
                        field_provenance.clone(),
                    )
                };
            assignments.insert(
                field.id.clone(),
                SemanticTypeAssignment {
                    id: SemanticTypeId::for_subject(&field.id),
                    subject: field.id.clone(),
                    semantic_type,
                    origin,
                    status,
                    provenance: assignment_provenance,
                },
            );
        }

        extend_with_local_type_assignments(&mut assignments, components, provenance);
        extend_with_parameter_type_assignments(&mut assignments, components, &aliases, bindings);
        extend_with_method_return_type_assignments(
            &mut assignments,
            components,
            provenance,
            &aliases,
            bindings,
        );

        Self::from_assignments_and_aliases(assignments, aliases)
            .with_action_signature_types(components)
    }

    /// Attaches explicit G1 Context declaration contracts without performing
    /// provider/consumer compatibility analysis.
    #[must_use]
    pub fn with_context_types(
        mut self,
        contexts: &BTreeMap<crate::ContextId, ContextEntity>,
    ) -> Self {
        for context in contexts.values() {
            let semantic_type = self
                .aliases
                .values()
                .find(|alias| {
                    alias.provenance.path == context.declared_type.provenance.path
                        && alias.name == context.declared_type.text
                })
                .map_or_else(
                    || {
                        semantic_type_from_annotation(&context.declared_type.text)
                            .unwrap_or(SemanticType::Unknown)
                    },
                    |alias| alias.semantic_type.clone(),
                );
            let subject = context.id.as_semantic_id().clone();
            self.assignments.insert(
                subject.clone(),
                SemanticTypeAssignment {
                    id: context.declared_type_id.clone(),
                    subject: subject.clone(),
                    semantic_type,
                    origin: subject,
                    status: SemanticTypeStatus::Declared,
                    provenance: context.provenance.clone(),
                },
            );
        }
        self
    }

    /// Attaches the exact nominal H1 `SlotContent` type to canonical Slot
    /// entities. Alias and structural compatibility are intentionally absent.
    #[must_use]
    pub fn with_slot_types(mut self, slots: &BTreeMap<crate::SlotId, SlotEntity>) -> Self {
        for slot in slots.values() {
            let subject = slot.id.as_semantic_id().clone();
            self.assignments.insert(
                subject.clone(),
                SemanticTypeAssignment {
                    id: SemanticTypeId::for_subject(&subject),
                    subject: subject.clone(),
                    semantic_type: SemanticType::SlotContent,
                    origin: subject,
                    status: SemanticTypeStatus::Declared,
                    provenance: slot.provenance.clone(),
                },
            );
        }
        self
    }

    /// Attaches the exact nominal I2 `Form` marker to canonical Form entities.
    /// Aliases, unions, generics, and structural compatibility never enter
    /// this authority because invalid declaration candidates do not lower.
    #[must_use]
    pub fn with_form_types(mut self, forms: &BTreeMap<crate::FormId, crate::FormEntity>) -> Self {
        for form in forms.values() {
            let subject = form.id.as_semantic_id().clone();
            self.assignments.insert(
                subject.clone(),
                SemanticTypeAssignment {
                    id: SemanticTypeId::for_subject(&subject),
                    subject: subject.clone(),
                    semantic_type: SemanticType::Form,
                    origin: subject,
                    status: SemanticTypeStatus::Declared,
                    provenance: form.provenance.clone(),
                },
            );
        }
        self
    }

    /// Attaches I3 Form Field assignments produced by the canonical field
    /// resolver. No annotation parsing, inference, or assignability is repeated.
    #[must_use]
    pub fn with_form_field_types(
        mut self,
        fields: &BTreeMap<crate::FieldId, crate::FormFieldEntity>,
    ) -> Self {
        self.assignments.extend(fields.values().map(|field| {
            (
                field.id.as_semantic_id().clone(),
                field.type_assignment.clone(),
            )
        }));
        self
    }

    /// Attaches explicit G2 Provider declaration contracts without deciding
    /// provider-value or Context compatibility.
    #[must_use]
    pub fn with_provider_types(
        mut self,
        providers: &BTreeMap<crate::ProviderId, ProviderEntity>,
    ) -> Self {
        for provider in providers.values() {
            let semantic_type = self
                .aliases
                .values()
                .find(|alias| {
                    alias.provenance.path == provider.declared_type.provenance.path
                        && alias.name == provider.declared_type.text
                })
                .map_or_else(
                    || {
                        semantic_type_from_annotation(&provider.declared_type.text)
                            .unwrap_or(SemanticType::Unknown)
                    },
                    |alias| alias.semantic_type.clone(),
                );
            let subject = provider.id.as_semantic_id().clone();
            self.assignments.insert(
                subject.clone(),
                SemanticTypeAssignment {
                    id: provider.declared_type_id.clone(),
                    subject: subject.clone(),
                    semantic_type,
                    origin: subject,
                    status: SemanticTypeStatus::Declared,
                    provenance: provider.provenance.clone(),
                },
            );
        }
        self
    }

    /// Attaches explicit G3 Consumer requested-type contracts without deciding
    /// Context, Provider, or Consumer compatibility.
    #[must_use]
    pub fn with_consumer_types(
        mut self,
        consumers: &BTreeMap<crate::ConsumerId, ConsumerEntity>,
    ) -> Self {
        for consumer in consumers.values() {
            let semantic_type = self
                .aliases
                .values()
                .find(|alias| {
                    alias.provenance.path == consumer.requested_type.provenance.path
                        && alias.name == consumer.requested_type.text
                })
                .map_or_else(
                    || {
                        semantic_type_from_annotation(&consumer.requested_type.text)
                            .unwrap_or(SemanticType::Unknown)
                    },
                    |alias| alias.semantic_type.clone(),
                );
            let subject = consumer.id.as_semantic_id().clone();
            self.assignments.insert(
                subject.clone(),
                SemanticTypeAssignment {
                    id: consumer.requested_type_id.clone(),
                    subject: subject.clone(),
                    semantic_type,
                    origin: subject,
                    status: SemanticTypeStatus::Declared,
                    provenance: consumer.provenance.clone(),
                },
            );
        }
        self
    }

    fn from_assignments_and_aliases(
        assignments: BTreeMap<SemanticId, SemanticTypeAssignment>,
        aliases: BTreeMap<SemanticId, SemanticTypeAlias>,
    ) -> Self {
        Self {
            assignments,
            aliases,
            list_scopes: BTreeMap::new(),
            member_accesses: BTreeMap::new(),
            computed_values: BTreeMap::new(),
            action_signatures: BTreeMap::new(),
            effect_statements: BTreeMap::new(),
        }
    }

    /// Attaches canonical inferred and declared contracts to computed entities.
    ///
    /// # Panics
    ///
    /// Panics when a computed expression ID does not resolve in `graph`.
    #[allow(clippy::too_many_lines)]
    #[must_use]
    pub fn with_computed_value_types(
        mut self,
        components: &[ComponentNode],
        computed_values: &BTreeMap<SemanticId, ComputedValue>,
        graph: &ExpressionGraph,
        references: &[SemanticReference],
    ) -> Self {
        let computed_components = components
            .iter()
            .flat_map(|component| {
                component
                    .methods
                    .iter()
                    .filter(|method| method.is_computed())
                    .map(move |method| (component.id.computed(&method.name), component))
            })
            .collect::<BTreeMap<_, _>>();
        let mut computed_types = BTreeMap::new();
        let mut expression_types = BTreeMap::new();

        for computed in computed_values.values() {
            let mut visiting = BTreeSet::new();
            infer_computed_type(
                &computed.id,
                graph,
                &self.assignments,
                &computed_components,
                computed_values,
                references,
                &mut computed_types,
                &mut expression_types,
                &mut visiting,
            );
        }

        for (id, semantic_type) in expression_types {
            let node = graph.node(&id).expect("computed expression graph node");
            self.assignments.insert(
                id.clone(),
                SemanticTypeAssignment {
                    id: SemanticTypeId::for_subject(&id),
                    subject: id,
                    semantic_type,
                    origin: node.owner.clone(),
                    status: SemanticTypeStatus::Inferred,
                    provenance: node.provenance.clone(),
                },
            );
        }

        for computed in computed_values.values() {
            let has_expression = graph.root_for(&computed.id).is_some();
            let inferred = computed_types
                .get(&computed.id)
                .cloned()
                .unwrap_or(SemanticType::Unknown);
            let method_assignment = self.assignments.get(&computed.method);
            let declared_return_type = method_assignment
                .filter(|assignment| assignment.status == SemanticTypeStatus::Declared)
                .map(|assignment| assignment.semantic_type.clone());
            let declared_return_compatible = declared_return_type
                .as_ref()
                .filter(|_| has_expression)
                .map(|declared| is_assignable(&inferred, declared));
            let (semantic_type, status, origin) = if has_expression {
                (
                    inferred,
                    SemanticTypeStatus::Inferred,
                    computed.method.clone(),
                )
            } else if let Some(assignment) = method_assignment {
                (
                    assignment.semantic_type.clone(),
                    assignment.status,
                    assignment.origin.clone(),
                )
            } else {
                (
                    SemanticType::Unknown,
                    SemanticTypeStatus::Inferred,
                    computed.method.clone(),
                )
            };
            let serialization = serialization_compatibility(&semantic_type);
            let boundary_compatibility = boundary_compatibility(
                &semantic_type,
                computed.execution_boundary,
                ExecutionBoundary::Client,
            );

            self.assignments.insert(
                computed.id.clone(),
                SemanticTypeAssignment {
                    id: SemanticTypeId::for_subject(&computed.id),
                    subject: computed.id.clone(),
                    semantic_type: semantic_type.clone(),
                    origin,
                    status,
                    provenance: computed.provenance.clone(),
                },
            );
            self.computed_values.insert(
                computed.id.clone(),
                ComputedValueType {
                    computed: computed.id.clone(),
                    method: computed.method.clone(),
                    semantic_type,
                    status,
                    declared_return_type,
                    declared_return_compatible,
                    serialization,
                    execution_boundary: computed.execution_boundary,
                    boundary_compatibility,
                    provenance: computed.provenance.clone(),
                },
            );
        }

        self
    }

    /// Attaches inferred types to canonical Provider and Context-default
    /// expressions using their already-resolved component ownership and the
    /// existing State/Computed type products. This creates no Context
    /// compatibility result; G5 consumes these facts separately.
    #[must_use]
    pub fn with_context_source_expression_types(
        mut self,
        components: &[ComponentNode],
        contexts: &BTreeMap<crate::ContextId, ContextEntity>,
        providers: &BTreeMap<crate::ProviderId, ProviderEntity>,
        graph: &ExpressionGraph,
    ) -> Self {
        let components_by_id = components
            .iter()
            .map(|component| (component.id.clone(), component))
            .collect::<BTreeMap<_, _>>();
        let owners = contexts
            .values()
            .filter_map(|context| {
                context
                    .owner
                    .entity_id()
                    .map(|owner| (context.id.as_semantic_id().clone(), owner.clone()))
            })
            .chain(providers.values().filter_map(|provider| {
                provider
                    .owner
                    .entity_id()
                    .map(|owner| (provider.id.as_semantic_id().clone(), owner.clone()))
            }))
            .collect::<BTreeMap<_, _>>();
        let expression_ids = owners
            .keys()
            .flat_map(|owner| graph.nodes_for(owner))
            .map(|node| node.id.clone())
            .collect::<Vec<_>>();
        let mut inferred = BTreeMap::new();
        for id in expression_ids {
            infer_context_source_expression_type(
                &id,
                graph,
                &self.assignments,
                &owners,
                &components_by_id,
                &mut inferred,
            );
        }
        for (id, semantic_type) in inferred {
            let Some(node) = graph.node(&id) else {
                continue;
            };
            self.assignments.insert(
                id.clone(),
                SemanticTypeAssignment {
                    id: SemanticTypeId::for_subject(&id),
                    subject: id,
                    semantic_type,
                    origin: node.owner.clone(),
                    status: SemanticTypeStatus::Inferred,
                    provenance: node.provenance.clone(),
                },
            );
        }
        self
    }

    fn with_action_signature_types(mut self, components: &[ComponentNode]) -> Self {
        for method in components
            .iter()
            .flat_map(|component| &component.methods)
            .filter(|method| method.is_action())
        {
            let input = method
                .parameters
                .iter()
                .filter_map(|parameter| {
                    self.assignments
                        .get(&parameter.id)
                        .map(|assignment| (parameter.id.clone(), assignment.semantic_type.clone()))
                })
                .collect();
            let output = self
                .assignments
                .get(&method.id)
                .map(|assignment| assignment.semantic_type.clone());
            self.action_signatures.insert(
                method.id.clone(),
                ActionSignatureType {
                    method: method.id.clone(),
                    input,
                    output,
                    is_async: method.is_async,
                },
            );
        }
        self
    }

    #[must_use]
    pub fn normalized(mut self) -> Self {
        for assignment in self.assignments.values_mut() {
            assignment.semantic_type = normalize_semantic_type(assignment.semantic_type.clone());
        }
        for alias in self.aliases.values_mut() {
            alias.semantic_type = normalize_semantic_type(alias.semantic_type.clone());
        }
        for scope in self.list_scopes.values_mut() {
            scope.item_type = normalize_semantic_type(scope.item_type.clone());
            if let Some(index_type) = &mut scope.index_type {
                *index_type = normalize_semantic_type(index_type.clone());
            }
        }
        for access in self.member_accesses.values_mut() {
            if let Some(semantic_type) = &mut access.semantic_type {
                *semantic_type = normalize_semantic_type(semantic_type.clone());
            }
        }
        for computed in self.computed_values.values_mut() {
            computed.semantic_type = normalize_semantic_type(computed.semantic_type.clone());
        }
        for action in self.action_signatures.values_mut() {
            for (_, input) in &mut action.input {
                *input = normalize_semantic_type(input.clone());
            }
            if let Some(output) = &mut action.output {
                *output = normalize_semantic_type(output.clone());
            }
        }
        for statement in self.effect_statements.values_mut() {
            statement.operand_types = statement
                .operand_types
                .iter()
                .cloned()
                .map(normalize_semantic_type)
                .collect();
            if let Some(target_type) = &mut statement.target_type {
                *target_type = normalize_semantic_type(target_type.clone());
            }
        }
        self
    }

    /// Attaches inferred types to canonical declaration expression nodes.
    ///
    /// # Panics
    ///
    /// Panics when the graph's node index contains an ID that does not resolve
    /// to an expression node.
    #[must_use]
    pub fn with_expression_types(
        mut self,
        graph: &ExpressionGraph,
        components: &[ComponentNode],
        contexts: &BTreeMap<crate::ContextId, ContextEntity>,
        providers: &BTreeMap<crate::ProviderId, ProviderEntity>,
    ) -> Self {
        let declaration_owners = components
            .iter()
            .flat_map(|component| component.state_fields.iter().map(|field| field.id.clone()))
            .chain(
                contexts
                    .keys()
                    .map(|context| context.as_semantic_id().clone()),
            )
            .chain(
                providers
                    .keys()
                    .map(|provider| provider.as_semantic_id().clone()),
            )
            .collect::<BTreeSet<_>>();
        let nodes = graph
            .nodes
            .values()
            .filter(|node| declaration_owners.contains(&node.owner))
            .map(|node| node.id.clone())
            .collect::<Vec<_>>();
        for id in nodes {
            let semantic_type = expression_semantic_type(&id, graph, &self.assignments);
            let node = graph.node(&id).expect("expression graph node");
            self.assignments.insert(
                id.clone(),
                SemanticTypeAssignment {
                    id: SemanticTypeId::for_subject(&id),
                    subject: id,
                    semantic_type,
                    origin: node.owner.clone(),
                    status: SemanticTypeStatus::Inferred,
                    provenance: node.provenance.clone(),
                },
            );
        }
        self
    }

    /// Attaches effect operand types and statement-level capability facts.
    ///
    /// This consumes only the immutable registry and existing component/ASM
    /// products. It intentionally records incompatibilities without emitting
    /// semantic diagnostics; F5 owns language-rule rejection.
    ///
    /// # Panics
    ///
    /// Panics when the graph's effect expression index contains an ID that
    /// does not resolve to an expression node.
    #[must_use]
    pub fn with_effect_statement_types(
        mut self,
        components: &[ComponentNode],
        effects: &BTreeMap<SemanticId, Effect>,
        statements: &BTreeMap<SemanticId, EffectStatement>,
        graph: &ExpressionGraph,
    ) -> Self {
        let components_by_id = components
            .iter()
            .map(|component| (component.id.clone(), component))
            .collect::<BTreeMap<_, _>>();
        let effect_nodes = effects
            .values()
            .flat_map(|effect| graph.nodes_for(&effect.id))
            .map(|node| node.id.clone())
            .collect::<Vec<_>>();
        let mut expression_types = BTreeMap::new();
        for id in effect_nodes {
            infer_effect_expression_type(
                &id,
                graph,
                &self.assignments,
                &components_by_id,
                effects,
                &mut expression_types,
            );
        }
        for (id, semantic_type) in expression_types {
            let node = graph.node(&id).expect("effect expression graph node");
            self.assignments.insert(
                id.clone(),
                SemanticTypeAssignment {
                    id: SemanticTypeId::for_subject(&id),
                    subject: id,
                    semantic_type,
                    origin: node.owner.clone(),
                    status: SemanticTypeStatus::Inferred,
                    provenance: node.provenance.clone(),
                },
            );
        }
        for statement in statements.values() {
            let Some(effect) = effects.get(&statement.owner) else {
                continue;
            };
            let Some(component_id) = effect.owner.entity_id() else {
                continue;
            };
            let Some(component) = components_by_id.get(component_id) else {
                continue;
            };
            let record = effect_statement_type_record(
                statement,
                effect,
                component,
                graph,
                &self.assignments,
            );
            self.effect_statements.insert(statement.id.clone(), record);
        }
        self
    }

    /// Attaches inferred types to direct text-binding entities with resolved
    /// state or local targets.
    #[must_use]
    pub fn with_template_binding_types(
        mut self,
        entities: &[TemplateSemanticEntity],
        references: &[SemanticReference],
    ) -> Self {
        for entity in entities.iter().filter(|entity| {
            matches!(
                entity.kind,
                TemplateSemanticKind::Binding
                    | TemplateSemanticKind::AttributeBinding
                    | TemplateSemanticKind::Conditional
            )
        }) {
            let Some(reference) = references.iter().find(|reference| {
                reference.source == entity.id
                    && matches!(
                        reference.kind,
                        SemanticReferenceKind::TemplateState
                            | SemanticReferenceKind::TemplateComputed
                            | SemanticReferenceKind::TemplateLocal
                    )
            }) else {
                continue;
            };
            let Some(target) = self.assignments.get(&reference.target) else {
                continue;
            };
            self.assignments.insert(
                entity.id.clone(),
                SemanticTypeAssignment {
                    id: SemanticTypeId::for_subject(&entity.id),
                    subject: entity.id.clone(),
                    semantic_type: target.semantic_type.clone(),
                    origin: target.origin.clone(),
                    status: SemanticTypeStatus::Inferred,
                    provenance: entity.provenance.clone(),
                },
            );
        }
        self.with_list_scope_types(entities, references)
            .with_list_member_types(entities)
    }

    fn with_list_scope_types(
        mut self,
        entities: &[TemplateSemanticEntity],
        references: &[SemanticReference],
    ) -> Self {
        for entity in entities
            .iter()
            .filter(|entity| entity.kind == TemplateSemanticKind::List)
        {
            let Some(reference) = references.iter().find(|reference| {
                reference.source == entity.id
                    && matches!(
                        reference.kind,
                        SemanticReferenceKind::TemplateState
                            | SemanticReferenceKind::TemplateComputed
                    )
            }) else {
                continue;
            };
            let Some(iterable) = self.assignments.get(&reference.target) else {
                continue;
            };
            let iterable_type = iterable.semantic_type.clone();
            let iterable_origin = iterable.origin.clone();
            self.assignments.insert(
                entity.id.clone(),
                SemanticTypeAssignment {
                    id: SemanticTypeId::for_subject(&entity.id),
                    subject: entity.id.clone(),
                    semantic_type: iterable_type.clone(),
                    origin: iterable_origin,
                    status: SemanticTypeStatus::Inferred,
                    provenance: entity.provenance.clone(),
                },
            );
            let Some((item_name, item_type)) = entity
                .list_item_variable
                .as_ref()
                .zip(iterable_element_type(&iterable_type))
            else {
                continue;
            };
            self.list_scopes.insert(
                entity.id.clone(),
                ListTemplateScopeType {
                    list: entity.id.clone(),
                    item_name: item_name.clone(),
                    item_type,
                    index_name: entity.list_index_variable.clone(),
                    index_type: entity
                        .list_index_variable
                        .as_ref()
                        .map(|_| SemanticType::Number),
                },
            );
        }
        self
    }

    fn with_list_member_types(mut self, entities: &[TemplateSemanticEntity]) -> Self {
        for entity in entities.iter().filter(|entity| {
            matches!(
                entity.kind,
                TemplateSemanticKind::Binding | TemplateSemanticKind::AttributeBinding
            ) && entity.scope == crate::TemplateSemanticScope::ListItem
        }) {
            let Some(expression) = entity.expression.as_deref() else {
                continue;
            };
            let Some((root, path)) = expression.split_once('.') else {
                continue;
            };
            let scopes = self
                .list_scopes
                .values()
                .filter(|scope| scope.item_name == root)
                .collect::<Vec<_>>();
            if scopes.len() != 1 {
                continue;
            }
            let semantic_type = member_access_type(&scopes[0].item_type, path);
            self.member_accesses.insert(
                entity.id.clone(),
                MemberAccessType {
                    entity: entity.id.clone(),
                    expression: expression.to_string(),
                    semantic_type: semantic_type.clone(),
                },
            );
            if let Some(semantic_type) = semantic_type {
                self.assignments.insert(
                    entity.id.clone(),
                    SemanticTypeAssignment {
                        id: SemanticTypeId::for_subject(&entity.id),
                        subject: entity.id.clone(),
                        semantic_type,
                        origin: scopes[0].list.clone(),
                        status: SemanticTypeStatus::Inferred,
                        provenance: entity.provenance.clone(),
                    },
                );
            }
        }
        self
    }
}

fn member_access_type(semantic_type: &SemanticType, path: &str) -> Option<SemanticType> {
    let mut current = semantic_type;
    for member in path.split('.') {
        if matches!(current, SemanticType::Unknown) {
            return Some(SemanticType::Unknown);
        }
        let SemanticType::Object(object) = current else {
            return None;
        };
        current = object.properties.get(member)?;
    }
    Some(current.clone())
}

fn iterable_element_type(semantic_type: &SemanticType) -> Option<SemanticType> {
    match semantic_type {
        SemanticType::Array(element) => Some((**element).clone()),
        SemanticType::Tuple(items) if items.is_empty() => Some(SemanticType::Unknown),
        SemanticType::Tuple(items) if items.windows(2).all(|pair| pair[0] == pair[1]) => {
            items.first().cloned()
        }
        SemanticType::Tuple(items) => Some(SemanticType::Union(items.clone())),
        SemanticType::Unknown => Some(SemanticType::Unknown),
        _ => None,
    }
}

fn extend_with_local_type_assignments(
    assignments: &mut BTreeMap<SemanticId, SemanticTypeAssignment>,
    components: &[ComponentNode],
    provenance: &BTreeMap<SemanticId, SourceProvenance>,
) {
    for local in components
        .iter()
        .flat_map(|component| &component.methods)
        .flat_map(|method| &method.local_variables)
    {
        let Some(local_provenance) = provenance.get(&local.id) else {
            continue;
        };
        assignments.insert(
            local.id.clone(),
            SemanticTypeAssignment {
                id: SemanticTypeId::for_subject(&local.id),
                subject: local.id.clone(),
                semantic_type: infer_serializable_value_type(&local.value),
                origin: local.id.clone(),
                status: SemanticTypeStatus::Inferred,
                provenance: local_provenance.clone(),
            },
        );
    }
}

fn extend_with_parameter_type_assignments(
    assignments: &mut BTreeMap<SemanticId, SemanticTypeAssignment>,
    components: &[ComponentNode],
    aliases: &BTreeMap<SemanticId, SemanticTypeAlias>,
    bindings: Option<&BindingTable>,
) {
    let aliases_by_path_and_name = aliases
        .values()
        .map(|alias| ((alias.provenance.path.clone(), alias.name.clone()), alias))
        .collect::<BTreeMap<_, _>>();

    for parameter in components
        .iter()
        .flat_map(|component| &component.methods)
        .flat_map(|method| &method.parameters)
    {
        let Some(declared_type) = &parameter.declared_type else {
            continue;
        };
        let alias = aliases_by_path_and_name
            .get(&(
                declared_type.provenance.path.clone(),
                declared_type.text.clone(),
            ))
            .copied();
        let imported_alias = bindings
            .and_then(|bindings| {
                bindings.resolve_import(&declared_type.provenance.path, &declared_type.text)
            })
            .and_then(|binding| match &binding.target {
                ImportBindingTarget::Symbol(symbol) if symbol.kind == SymbolKind::TypeAlias => {
                    aliases.get(&symbol.id)
                }
                _ => None,
            });
        let alias = alias.or(imported_alias);
        let semantic_type = alias
            .map(|alias| alias.semantic_type.clone())
            .or_else(|| semantic_type_from_annotation(&declared_type.text));
        let Some(semantic_type) = semantic_type else {
            continue;
        };
        assignments.insert(
            parameter.id.clone(),
            SemanticTypeAssignment {
                id: SemanticTypeId::for_subject(&parameter.id),
                subject: parameter.id.clone(),
                semantic_type,
                origin: alias.map_or_else(|| parameter.id.clone(), |alias| alias.id.clone()),
                status: SemanticTypeStatus::Declared,
                provenance: declared_type.provenance.clone(),
            },
        );
    }
}

fn extend_with_method_return_type_assignments(
    assignments: &mut BTreeMap<SemanticId, SemanticTypeAssignment>,
    components: &[ComponentNode],
    provenance: &BTreeMap<SemanticId, SourceProvenance>,
    aliases: &BTreeMap<SemanticId, SemanticTypeAlias>,
    bindings: Option<&BindingTable>,
) {
    let aliases_by_path_and_name = aliases
        .values()
        .map(|alias| ((alias.provenance.path.clone(), alias.name.clone()), alias))
        .collect::<BTreeMap<_, _>>();

    for method in components.iter().flat_map(|component| &component.methods) {
        let declared = method.declared_return_type.as_ref();
        let (semantic_type, origin, status, assignment_provenance) =
            if let Some(declared) = declared {
                let alias = aliases_by_path_and_name
                    .get(&(declared.provenance.path.clone(), declared.text.clone()))
                    .copied()
                    .or_else(|| {
                        bindings
                            .and_then(|bindings| {
                                bindings.resolve_import(&declared.provenance.path, &declared.text)
                            })
                            .and_then(|binding| match &binding.target {
                                ImportBindingTarget::Symbol(symbol)
                                    if symbol.kind == SymbolKind::TypeAlias =>
                                {
                                    aliases.get(&symbol.id)
                                }
                                _ => None,
                            })
                    });
                let Some(semantic_type) = alias
                    .map(|alias| alias.semantic_type.clone())
                    .or_else(|| semantic_type_from_annotation(&declared.text))
                else {
                    continue;
                };
                (
                    semantic_type,
                    alias.map_or_else(|| method.id.clone(), |alias| alias.id.clone()),
                    SemanticTypeStatus::Declared,
                    declared.provenance.clone(),
                )
            } else {
                let Some(return_type) = inferred_return_type(&method.return_values) else {
                    continue;
                };
                let Some(method_provenance) = provenance.get(&method.id) else {
                    continue;
                };
                (
                    return_type,
                    method.id.clone(),
                    SemanticTypeStatus::Inferred,
                    method_provenance.clone(),
                )
            };
        assignments.insert(
            method.id.clone(),
            SemanticTypeAssignment {
                id: SemanticTypeId::for_subject(&method.id),
                subject: method.id.clone(),
                semantic_type,
                origin,
                status,
                provenance: assignment_provenance,
            },
        );
    }
}

fn inferred_return_type(values: &[SerializableValue]) -> Option<SemanticType> {
    let mut types = values
        .iter()
        .map(infer_serializable_value_type)
        .collect::<Vec<_>>();
    match types.as_slice() {
        [] => None,
        [semantic_type] => Some(semantic_type.clone()),
        _ if types.windows(2).all(|window| window[0] == window[1]) => types.pop(),
        _ => Some(SemanticType::Union(types)),
    }
}

#[allow(clippy::too_many_arguments)]
fn infer_effect_expression_type(
    id: &SemanticId,
    graph: &ExpressionGraph,
    assignments: &BTreeMap<SemanticId, SemanticTypeAssignment>,
    components: &BTreeMap<SemanticId, &ComponentNode>,
    effects: &BTreeMap<SemanticId, Effect>,
    expression_types: &mut BTreeMap<SemanticId, SemanticType>,
) -> SemanticType {
    if let Some(semantic_type) = expression_types.get(id) {
        return semantic_type.clone();
    }
    let node = graph.node(id).expect("effect expression graph node");
    let child_type =
        |child: &SemanticId, expression_types: &mut BTreeMap<SemanticId, SemanticType>| {
            infer_effect_expression_type(
                child,
                graph,
                assignments,
                components,
                effects,
                expression_types,
            )
        };
    let semantic_type = match &node.kind {
        ExpressionNodeKind::Literal(value) => state_initializer_value_type(value),
        ExpressionNodeKind::Boolean(value) => SemanticType::BooleanLiteral(*value),
        ExpressionNodeKind::Template { .. } => SemanticType::String,
        ExpressionNodeKind::Identifier(_) | ExpressionNodeKind::Call { .. } => {
            SemanticType::Unknown
        }
        ExpressionNodeKind::SemanticPackagePureCall {
            operation: crate::semantic_package::SemanticPackagePureOperation::Identity,
            arguments,
            ..
        } => arguments.first().map_or(SemanticType::Unknown, |argument| {
            child_type(argument, expression_types)
        }),
        ExpressionNodeKind::BuiltinPureCall {
            operation:
                crate::component_graph::BuiltinPureOperation::MathAbs
                | crate::component_graph::BuiltinPureOperation::MathMin
                | crate::component_graph::BuiltinPureOperation::MathMax,
            ..
        } => SemanticType::Number,
        ExpressionNodeKind::ThisMember { name } => effects
            .get(&node.owner)
            .and_then(|effect| effect.owner.entity_id())
            .and_then(|component_id| components.get(component_id))
            .and_then(|component| {
                component
                    .state_fields
                    .iter()
                    .find(|field| field.name == *name)
                    .map(|field| field.id.clone())
                    .or_else(|| {
                        let computed = component.id.computed(name);
                        assignments.contains_key(&computed).then_some(computed)
                    })
            })
            .and_then(|target| assignments.get(&target))
            .map_or(SemanticType::Unknown, |assignment| {
                assignment.semantic_type.clone()
            }),
        ExpressionNodeKind::MemberAccess {
            object, property, ..
        } => computed_member_access_type(&child_type(object, expression_types), property),
        ExpressionNodeKind::IndexAccess { object, index } => computed_index_access_type(
            &child_type(object, expression_types),
            &child_type(index, expression_types),
        ),
        ExpressionNodeKind::Conditional {
            when_true,
            when_false,
            ..
        } => conditional_result_type(
            child_type(when_true, expression_types),
            child_type(when_false, expression_types),
        ),
        ExpressionNodeKind::Arithmetic {
            left,
            right,
            operator,
        } => operator_result_type(
            SemanticOperator::Arithmetic(*operator),
            &[
                child_type(left, expression_types),
                child_type(right, expression_types),
            ],
        )
        .unwrap_or(SemanticType::Unknown),
        ExpressionNodeKind::Comparison {
            left,
            right,
            operator,
        } => operator_result_type(
            SemanticOperator::Comparison(*operator),
            &[
                child_type(left, expression_types),
                child_type(right, expression_types),
            ],
        )
        .unwrap_or(SemanticType::Unknown),
        ExpressionNodeKind::Logical {
            left,
            right,
            operator,
        } => operator_result_type(
            SemanticOperator::Logical(*operator),
            &[
                child_type(left, expression_types),
                child_type(right, expression_types),
            ],
        )
        .unwrap_or(SemanticType::Unknown),
        ExpressionNodeKind::NullishCoalescing { left, right } => operator_result_type(
            SemanticOperator::NullishCoalescing,
            &[
                child_type(left, expression_types),
                child_type(right, expression_types),
            ],
        )
        .unwrap_or(SemanticType::Unknown),
        ExpressionNodeKind::Unary { operand, operator } => operator_result_type(
            SemanticOperator::Unary(*operator),
            &[child_type(operand, expression_types)],
        )
        .unwrap_or(SemanticType::Unknown),
    };
    expression_types.insert(id.clone(), semantic_type.clone());
    semantic_type
}

#[allow(clippy::too_many_lines)]
fn effect_statement_type_record(
    statement: &EffectStatement,
    effect: &Effect,
    component: &ComponentNode,
    graph: &ExpressionGraph,
    assignments: &BTreeMap<SemanticId, SemanticTypeAssignment>,
) -> EffectStatementTypeRecord {
    let expression_type = |id: &SemanticId| {
        assignments
            .get(id)
            .map_or(SemanticType::Unknown, |assignment| {
                assignment.semantic_type.clone()
            })
    };
    let record = |operation_classification,
                  operand_types,
                  target_type,
                  signature_compatibility,
                  boundary_compatibility,
                  serialization_compatibility,
                  capability_operation| EffectStatementTypeRecord {
        statement: statement.id.clone(),
        operation_classification,
        operand_types,
        target_type,
        signature_compatibility,
        boundary_compatibility,
        serialization_compatibility,
        capability_operation,
        provenance: statement.provenance.clone(),
    };
    match &statement.kind {
        EffectStatementKind::ExternalMemberAssignment { target, value } => {
            let value_type = expression_type(value);
            if let Some(name) = this_member_name(target, graph) {
                if let Some(field) = component
                    .state_fields
                    .iter()
                    .find(|field| field.name == name)
                {
                    let target_type = assignments
                        .get(&field.id)
                        .map(|assignment| assignment.semantic_type.clone());
                    let signature = target_type
                        .as_ref()
                        .map_or(EffectCompatibility::Unknown, |target| {
                            effect_assignability(&value_type, target)
                        });
                    return record(
                        EffectOperationClassification::ReactiveStateAssignment,
                        vec![value_type],
                        target_type,
                        signature,
                        EffectCompatibility::Compatible,
                        EffectCompatibility::Compatible,
                        None,
                    );
                }
                return record(
                    EffectOperationClassification::UnresolvedComponentAssignment,
                    vec![value_type],
                    None,
                    EffectCompatibility::Unknown,
                    EffectCompatibility::Unknown,
                    EffectCompatibility::Unknown,
                    None,
                );
            }
            let path = static_capability_path(target, graph);
            let operation = path.as_deref().and_then(|path| {
                EFFECT_CAPABILITY_REGISTRY
                    .operation_at(path, CapabilityOperationKind::MemberAssignment)
            });
            let Some(operation) = operation else {
                return record(
                    EffectOperationClassification::UnknownExternalCapability,
                    vec![value_type],
                    None,
                    EffectCompatibility::Unknown,
                    EffectCompatibility::Unknown,
                    EffectCompatibility::Unknown,
                    None,
                );
            };
            let target_type = operation_value_type(operation.signature.parameters, 0);
            let signature = target_type
                .as_ref()
                .map_or(EffectCompatibility::Unknown, |target| {
                    effect_assignability(&value_type, target)
                });
            record(
                EffectOperationClassification::RecognizedCapability,
                vec![value_type],
                target_type,
                signature,
                effect_boundary_compatibility(effect, operation.boundary),
                EffectCompatibility::Compatible,
                Some(operation.id),
            )
        }
        EffectStatementKind::CapabilityCall { callee, arguments } => {
            let argument_types = arguments.iter().map(expression_type).collect::<Vec<_>>();
            if let Some(name) = this_member_name(callee, graph) {
                let classification = component
                    .methods
                    .iter()
                    .find(|method| method.name == name)
                    .map_or(
                        EffectOperationClassification::UnresolvedComponentCall,
                        |method| {
                            if method.is_action() {
                                EffectOperationClassification::ComponentActionCall
                            } else if method.is_effect() {
                                EffectOperationClassification::ComponentEffectCall
                            } else {
                                EffectOperationClassification::ComponentMethodCall
                            }
                        },
                    );
                return record(
                    classification,
                    argument_types,
                    None,
                    EffectCompatibility::Unknown,
                    EffectCompatibility::Compatible,
                    EffectCompatibility::Compatible,
                    None,
                );
            }
            let path = static_capability_path(callee, graph);
            let operation = path.as_deref().and_then(|path| {
                EFFECT_CAPABILITY_REGISTRY.operation_at(path, CapabilityOperationKind::MethodCall)
            });
            let Some(operation) = operation else {
                return record(
                    EffectOperationClassification::UnknownExternalCapability,
                    argument_types,
                    None,
                    EffectCompatibility::Incompatible,
                    EffectCompatibility::Unknown,
                    EffectCompatibility::Unknown,
                    None,
                );
            };
            let signature =
                operation_signature_compatibility(operation.signature.parameters, &argument_types);
            let serialization = match operation.argument_serialization {
                crate::ArgumentSerializationPolicy::None => EffectCompatibility::Compatible,
                crate::ArgumentSerializationPolicy::Structural => argument_types.iter().fold(
                    EffectCompatibility::Compatible,
                    |compatibility, semantic_type| {
                        combine_effect_compatibility(
                            compatibility,
                            if serialization_compatibility(semantic_type)
                                == SerializationCompatibility::Serializable
                            {
                                EffectCompatibility::Compatible
                            } else {
                                EffectCompatibility::Incompatible
                            },
                        )
                    },
                ),
            };
            record(
                EffectOperationClassification::RecognizedCapability,
                argument_types,
                None,
                signature,
                effect_boundary_compatibility(effect, operation.boundary),
                serialization,
                Some(operation.id),
            )
        }
        EffectStatementKind::EffectReturn { value: None } => record(
            EffectOperationClassification::BareReturn,
            Vec::new(),
            None,
            EffectCompatibility::Compatible,
            EffectCompatibility::Compatible,
            EffectCompatibility::Compatible,
            None,
        ),
        EffectStatementKind::EffectReturn { value: Some(value) } => record(
            EffectOperationClassification::ValueReturn,
            vec![expression_type(value)],
            None,
            EffectCompatibility::Incompatible,
            EffectCompatibility::Compatible,
            EffectCompatibility::Compatible,
            None,
        ),
        EffectStatementKind::Empty => record(
            EffectOperationClassification::Empty,
            Vec::new(),
            None,
            EffectCompatibility::Compatible,
            EffectCompatibility::Compatible,
            EffectCompatibility::Compatible,
            None,
        ),
        EffectStatementKind::Unsupported(_) => record(
            EffectOperationClassification::UnsupportedStatement,
            Vec::new(),
            None,
            EffectCompatibility::Incompatible,
            EffectCompatibility::Unknown,
            EffectCompatibility::Unknown,
            None,
        ),
    }
}

fn this_member_name<'a>(id: &SemanticId, graph: &'a ExpressionGraph) -> Option<&'a str> {
    match &graph.node(id)?.kind {
        ExpressionNodeKind::ThisMember { name } => Some(name),
        _ => None,
    }
}

fn static_capability_path(id: &SemanticId, graph: &ExpressionGraph) -> Option<String> {
    match &graph.node(id)?.kind {
        ExpressionNodeKind::Identifier(name) if name != "this" => Some(name.clone()),
        ExpressionNodeKind::MemberAccess {
            object, property, ..
        } => Some(format!(
            "{}.{}",
            static_capability_path(object, graph)?,
            property
        )),
        _ => None,
    }
}

fn operation_value_type(parameters: CapabilityParameters, index: usize) -> Option<SemanticType> {
    match parameters {
        CapabilityParameters::Fixed(parameters) => parameters
            .get(index)
            .and_then(|parameter| parameter.semantic_type()),
        CapabilityParameters::Variadic(parameter) => parameter.semantic_type(),
    }
}

fn operation_signature_compatibility(
    parameters: CapabilityParameters,
    arguments: &[SemanticType],
) -> EffectCompatibility {
    match parameters {
        CapabilityParameters::Fixed(parameters) if parameters.len() != arguments.len() => {
            EffectCompatibility::Incompatible
        }
        CapabilityParameters::Fixed(parameters) => parameters.iter().zip(arguments).fold(
            EffectCompatibility::Compatible,
            |compatibility, (parameter, argument)| {
                combine_effect_compatibility(
                    compatibility,
                    capability_value_compatibility(*parameter, argument),
                )
            },
        ),
        CapabilityParameters::Variadic(parameter) => arguments.iter().fold(
            EffectCompatibility::Compatible,
            |compatibility, argument| {
                combine_effect_compatibility(
                    compatibility,
                    capability_value_compatibility(parameter, argument),
                )
            },
        ),
    }
}

fn capability_value_compatibility(
    contract: CapabilityValueContract,
    value: &SemanticType,
) -> EffectCompatibility {
    match contract {
        CapabilityValueContract::String => effect_assignability(
            value,
            &contract
                .semantic_type()
                .expect("string capability contract type"),
        ),
        CapabilityValueContract::SerializableDiagnosticValue => {
            if serialization_compatibility(value) == SerializationCompatibility::Serializable {
                EffectCompatibility::Compatible
            } else {
                EffectCompatibility::Incompatible
            }
        }
    }
}

fn effect_assignability(source: &SemanticType, target: &SemanticType) -> EffectCompatibility {
    if source == &SemanticType::Unknown || source == &SemanticType::Never {
        EffectCompatibility::Unknown
    } else if is_assignable(source, target) {
        EffectCompatibility::Compatible
    } else {
        EffectCompatibility::Incompatible
    }
}

fn effect_boundary_compatibility(
    effect: &Effect,
    operation_boundary: ExecutionBoundary,
) -> EffectCompatibility {
    if effect.execution_boundary == operation_boundary {
        EffectCompatibility::Compatible
    } else {
        EffectCompatibility::Incompatible
    }
}

fn combine_effect_compatibility(
    left: EffectCompatibility,
    right: EffectCompatibility,
) -> EffectCompatibility {
    match (left, right) {
        (EffectCompatibility::Incompatible, _) | (_, EffectCompatibility::Incompatible) => {
            EffectCompatibility::Incompatible
        }
        (EffectCompatibility::Unknown, _) | (_, EffectCompatibility::Unknown) => {
            EffectCompatibility::Unknown
        }
        _ => EffectCompatibility::Compatible,
    }
}

fn expression_semantic_type(
    id: &SemanticId,
    graph: &ExpressionGraph,
    assignments: &BTreeMap<SemanticId, SemanticTypeAssignment>,
) -> SemanticType {
    let node = graph.node(id).expect("expression graph node");
    let child_type = |id: &SemanticId| {
        assignments.get(id).map_or_else(
            || expression_semantic_type(id, graph, assignments),
            |assignment| assignment.semantic_type.clone(),
        )
    };

    match &node.kind {
        ExpressionNodeKind::Literal(value) => state_initializer_value_type(value),
        ExpressionNodeKind::Boolean(value) => SemanticType::BooleanLiteral(*value),
        ExpressionNodeKind::Template { .. } => SemanticType::String,
        ExpressionNodeKind::Identifier(_)
        | ExpressionNodeKind::Call { .. }
        | ExpressionNodeKind::SemanticPackagePureCall { .. }
        | ExpressionNodeKind::BuiltinPureCall { .. }
        | ExpressionNodeKind::ThisMember { .. }
        | ExpressionNodeKind::MemberAccess { .. }
        | ExpressionNodeKind::IndexAccess { .. }
        | ExpressionNodeKind::Conditional { .. } => SemanticType::Unknown,
        ExpressionNodeKind::Arithmetic {
            left,
            right,
            operator,
        } => operator_result_type(
            SemanticOperator::Arithmetic(*operator),
            &[child_type(left), child_type(right)],
        )
        .unwrap_or(SemanticType::Unknown),
        ExpressionNodeKind::Comparison {
            left,
            right,
            operator,
        } => operator_result_type(
            SemanticOperator::Comparison(*operator),
            &[child_type(left), child_type(right)],
        )
        .unwrap_or(SemanticType::Unknown),
        ExpressionNodeKind::Logical {
            left,
            right,
            operator,
        } => operator_result_type(
            SemanticOperator::Logical(*operator),
            &[child_type(left), child_type(right)],
        )
        .unwrap_or(SemanticType::Unknown),
        ExpressionNodeKind::NullishCoalescing { left, right } => operator_result_type(
            SemanticOperator::NullishCoalescing,
            &[child_type(left), child_type(right)],
        )
        .unwrap_or(SemanticType::Unknown),
        ExpressionNodeKind::Unary { operand, operator } => {
            operator_result_type(SemanticOperator::Unary(*operator), &[child_type(operand)])
                .unwrap_or(SemanticType::Unknown)
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn infer_computed_type(
    computed: &SemanticId,
    graph: &ExpressionGraph,
    assignments: &BTreeMap<SemanticId, SemanticTypeAssignment>,
    computed_components: &BTreeMap<SemanticId, &ComponentNode>,
    computed_values: &BTreeMap<SemanticId, ComputedValue>,
    references: &[SemanticReference],
    computed_types: &mut BTreeMap<SemanticId, SemanticType>,
    expression_types: &mut BTreeMap<SemanticId, SemanticType>,
    visiting: &mut BTreeSet<SemanticId>,
) -> SemanticType {
    if let Some(semantic_type) = computed_types.get(computed) {
        return semantic_type.clone();
    }
    if !visiting.insert(computed.clone()) {
        return SemanticType::Unknown;
    }

    let semantic_type = graph
        .root_for(computed)
        .map_or(SemanticType::Unknown, |root| {
            infer_computed_expression_type(
                root,
                graph,
                assignments,
                computed_components,
                computed_values,
                references,
                computed_types,
                expression_types,
                visiting,
            )
        });
    visiting.remove(computed);
    computed_types.insert(computed.clone(), semantic_type.clone());
    semantic_type
}

fn infer_context_source_expression_type(
    id: &SemanticId,
    graph: &ExpressionGraph,
    assignments: &BTreeMap<SemanticId, SemanticTypeAssignment>,
    owners: &BTreeMap<SemanticId, SemanticId>,
    components: &BTreeMap<SemanticId, &ComponentNode>,
    inferred: &mut BTreeMap<SemanticId, SemanticType>,
) -> SemanticType {
    if let Some(semantic_type) = inferred.get(id) {
        return semantic_type.clone();
    }
    let Some(node) = graph.node(id) else {
        return SemanticType::Unknown;
    };
    let child = |child: &SemanticId, inferred: &mut BTreeMap<SemanticId, SemanticType>| {
        infer_context_source_expression_type(
            child,
            graph,
            assignments,
            owners,
            components,
            inferred,
        )
    };
    let semantic_type = match &node.kind {
        ExpressionNodeKind::Literal(value) => state_initializer_value_type(value),
        ExpressionNodeKind::Boolean(value) => SemanticType::BooleanLiteral(*value),
        ExpressionNodeKind::Template { .. } => SemanticType::String,
        ExpressionNodeKind::Identifier(_) | ExpressionNodeKind::Call { .. } => {
            SemanticType::Unknown
        }
        ExpressionNodeKind::SemanticPackagePureCall {
            operation: crate::semantic_package::SemanticPackagePureOperation::Identity,
            arguments,
            ..
        } => arguments
            .first()
            .map_or(SemanticType::Unknown, |argument| child(argument, inferred)),
        ExpressionNodeKind::BuiltinPureCall {
            operation:
                crate::component_graph::BuiltinPureOperation::MathAbs
                | crate::component_graph::BuiltinPureOperation::MathMin
                | crate::component_graph::BuiltinPureOperation::MathMax,
            ..
        } => SemanticType::Number,
        ExpressionNodeKind::ThisMember { name } => owners
            .get(&node.owner)
            .and_then(|owner| components.get(owner))
            .and_then(|component| {
                component
                    .state_fields
                    .iter()
                    .find(|field| field.name == *name)
                    .map(|field| field.id.clone())
                    .or_else(|| {
                        let computed = component.id.computed(name);
                        assignments.contains_key(&computed).then_some(computed)
                    })
            })
            .and_then(|target| assignments.get(&target))
            .map_or(SemanticType::Unknown, |assignment| {
                assignment.semantic_type.clone()
            }),
        ExpressionNodeKind::MemberAccess {
            object, property, ..
        } => computed_member_access_type(&child(object, inferred), property),
        ExpressionNodeKind::IndexAccess { object, index } => {
            computed_index_access_type(&child(object, inferred), &child(index, inferred))
        }
        ExpressionNodeKind::Conditional {
            when_true,
            when_false,
            ..
        } => conditional_result_type(child(when_true, inferred), child(when_false, inferred)),
        ExpressionNodeKind::Arithmetic {
            left,
            right,
            operator,
        } => operator_result_type(
            SemanticOperator::Arithmetic(*operator),
            &[child(left, inferred), child(right, inferred)],
        )
        .unwrap_or(SemanticType::Unknown),
        ExpressionNodeKind::Comparison {
            left,
            right,
            operator,
        } => operator_result_type(
            SemanticOperator::Comparison(*operator),
            &[child(left, inferred), child(right, inferred)],
        )
        .unwrap_or(SemanticType::Unknown),
        ExpressionNodeKind::Logical {
            left,
            right,
            operator,
        } => operator_result_type(
            SemanticOperator::Logical(*operator),
            &[child(left, inferred), child(right, inferred)],
        )
        .unwrap_or(SemanticType::Unknown),
        ExpressionNodeKind::NullishCoalescing { left, right } => operator_result_type(
            SemanticOperator::NullishCoalescing,
            &[child(left, inferred), child(right, inferred)],
        )
        .unwrap_or(SemanticType::Unknown),
        ExpressionNodeKind::Unary { operand, operator } => operator_result_type(
            SemanticOperator::Unary(*operator),
            &[child(operand, inferred)],
        )
        .unwrap_or(SemanticType::Unknown),
    };
    inferred.insert(id.clone(), semantic_type.clone());
    semantic_type
}

#[allow(clippy::too_many_arguments)]
fn infer_computed_expression_type(
    id: &SemanticId,
    graph: &ExpressionGraph,
    assignments: &BTreeMap<SemanticId, SemanticTypeAssignment>,
    computed_components: &BTreeMap<SemanticId, &ComponentNode>,
    computed_values: &BTreeMap<SemanticId, ComputedValue>,
    references: &[SemanticReference],
    computed_types: &mut BTreeMap<SemanticId, SemanticType>,
    expression_types: &mut BTreeMap<SemanticId, SemanticType>,
    visiting: &mut BTreeSet<SemanticId>,
) -> SemanticType {
    if let Some(semantic_type) = expression_types.get(id) {
        return semantic_type.clone();
    }

    let node = graph.node(id).expect("computed expression graph node");
    let child_type = |child: &SemanticId,
                      expression_types: &mut BTreeMap<SemanticId, SemanticType>,
                      computed_types: &mut BTreeMap<SemanticId, SemanticType>,
                      visiting: &mut BTreeSet<SemanticId>| {
        infer_computed_expression_type(
            child,
            graph,
            assignments,
            computed_components,
            computed_values,
            references,
            computed_types,
            expression_types,
            visiting,
        )
    };
    let semantic_type = match &node.kind {
        ExpressionNodeKind::Literal(value) => state_initializer_value_type(value),
        ExpressionNodeKind::Boolean(value) => SemanticType::BooleanLiteral(*value),
        ExpressionNodeKind::Template { .. } => SemanticType::String,
        ExpressionNodeKind::Identifier(_) | ExpressionNodeKind::Call { .. } => {
            SemanticType::Unknown
        }
        ExpressionNodeKind::SemanticPackagePureCall {
            operation: crate::semantic_package::SemanticPackagePureOperation::Identity,
            arguments,
            ..
        } => arguments.first().map_or(SemanticType::Unknown, |argument| {
            child_type(argument, expression_types, computed_types, visiting)
        }),
        ExpressionNodeKind::BuiltinPureCall {
            operation:
                crate::component_graph::BuiltinPureOperation::MathAbs
                | crate::component_graph::BuiltinPureOperation::MathMin
                | crate::component_graph::BuiltinPureOperation::MathMax,
            ..
        } => SemanticType::Number,
        ExpressionNodeKind::ThisMember { name } => infer_computed_read_type(
            &node.owner,
            name,
            assignments,
            computed_components,
            computed_values,
            references,
            graph,
            computed_types,
            expression_types,
            visiting,
        ),
        ExpressionNodeKind::MemberAccess {
            object, property, ..
        } => {
            let object = child_type(object, expression_types, computed_types, visiting);
            computed_member_access_type(&object, property)
        }
        ExpressionNodeKind::IndexAccess { object, index } => {
            let object = child_type(object, expression_types, computed_types, visiting);
            let index = child_type(index, expression_types, computed_types, visiting);
            computed_index_access_type(&object, &index)
        }
        ExpressionNodeKind::Conditional {
            when_true,
            when_false,
            ..
        } => conditional_result_type(
            child_type(when_true, expression_types, computed_types, visiting),
            child_type(when_false, expression_types, computed_types, visiting),
        ),
        ExpressionNodeKind::Arithmetic {
            left,
            right,
            operator,
        } => operator_result_type(
            SemanticOperator::Arithmetic(*operator),
            &[
                child_type(left, expression_types, computed_types, visiting),
                child_type(right, expression_types, computed_types, visiting),
            ],
        )
        .unwrap_or(SemanticType::Unknown),
        ExpressionNodeKind::Comparison {
            left,
            right,
            operator,
        } => operator_result_type(
            SemanticOperator::Comparison(*operator),
            &[
                child_type(left, expression_types, computed_types, visiting),
                child_type(right, expression_types, computed_types, visiting),
            ],
        )
        .unwrap_or(SemanticType::Unknown),
        ExpressionNodeKind::Logical {
            left,
            right,
            operator,
        } => operator_result_type(
            SemanticOperator::Logical(*operator),
            &[
                child_type(left, expression_types, computed_types, visiting),
                child_type(right, expression_types, computed_types, visiting),
            ],
        )
        .unwrap_or(SemanticType::Unknown),
        ExpressionNodeKind::NullishCoalescing { left, right } => operator_result_type(
            SemanticOperator::NullishCoalescing,
            &[
                child_type(left, expression_types, computed_types, visiting),
                child_type(right, expression_types, computed_types, visiting),
            ],
        )
        .unwrap_or(SemanticType::Unknown),
        ExpressionNodeKind::Unary { operand, operator } => operator_result_type(
            SemanticOperator::Unary(*operator),
            &[child_type(
                operand,
                expression_types,
                computed_types,
                visiting,
            )],
        )
        .unwrap_or(SemanticType::Unknown),
    };
    expression_types.insert(id.clone(), semantic_type.clone());
    semantic_type
}

#[allow(clippy::too_many_arguments)]
fn infer_computed_read_type(
    computed: &SemanticId,
    name: &str,
    assignments: &BTreeMap<SemanticId, SemanticTypeAssignment>,
    computed_components: &BTreeMap<SemanticId, &ComponentNode>,
    computed_values: &BTreeMap<SemanticId, ComputedValue>,
    references: &[SemanticReference],
    graph: &ExpressionGraph,
    computed_types: &mut BTreeMap<SemanticId, SemanticType>,
    expression_types: &mut BTreeMap<SemanticId, SemanticType>,
    visiting: &mut BTreeSet<SemanticId>,
) -> SemanticType {
    let Some(component) = computed_components.get(computed) else {
        return SemanticType::Unknown;
    };
    let target = component
        .state_fields
        .iter()
        .find(|field| field.name == name)
        .map(|field| field.id.clone())
        .or_else(|| {
            let target = component.id.computed(name);
            computed_values.contains_key(&target).then_some(target)
        });
    let Some(target) = target else {
        return SemanticType::Unknown;
    };
    if !references
        .iter()
        .any(|reference| reference.source == *computed && reference.target == target)
    {
        return SemanticType::Unknown;
    }
    if computed_values.contains_key(&target) {
        return infer_computed_type(
            &target,
            graph,
            assignments,
            computed_components,
            computed_values,
            references,
            computed_types,
            expression_types,
            visiting,
        );
    }
    assignments
        .get(&target)
        .map_or(SemanticType::Unknown, |assignment| {
            assignment.semantic_type.clone()
        })
}

fn computed_member_access_type(semantic_type: &SemanticType, property: &str) -> SemanticType {
    match semantic_type {
        SemanticType::Object(object) => object
            .properties
            .get(property)
            .cloned()
            .unwrap_or(SemanticType::Unknown),
        _ => SemanticType::Unknown,
    }
}

fn computed_index_access_type(object: &SemanticType, index: &SemanticType) -> SemanticType {
    match (object, index) {
        (SemanticType::Tuple(items), SemanticType::NumberLiteral(index)) => index
            .parse::<usize>()
            .ok()
            .and_then(|index| items.get(index))
            .cloned()
            .unwrap_or(SemanticType::Unknown),
        (SemanticType::Array(element), SemanticType::Number | SemanticType::NumberLiteral(_)) => {
            element.as_ref().clone()
        }
        (SemanticType::Object(object), SemanticType::StringLiteral(property)) => object
            .properties
            .get(property)
            .cloned()
            .unwrap_or(SemanticType::Unknown),
        _ => SemanticType::Unknown,
    }
}

fn conditional_result_type(when_true: SemanticType, when_false: SemanticType) -> SemanticType {
    if when_true == when_false {
        when_true
    } else {
        normalize_semantic_type(SemanticType::Union(vec![when_true, when_false]))
    }
}

/// Returns the result type for one valid semantic operator application.
///
/// An absent result means the supplied operand types are not valid for that
/// operator. Unknown operands deliberately propagate `unknown` rather than
/// being treated as a valid concrete operation.
#[must_use]
pub fn operator_result_type(
    operator: SemanticOperator,
    operands: &[SemanticType],
) -> Option<SemanticType> {
    match operator {
        SemanticOperator::Arithmetic(_) | SemanticOperator::Comparison(_) => {
            let [left, right] = operands else {
                return None;
            };
            if left == &SemanticType::Unknown || right == &SemanticType::Unknown {
                Some(SemanticType::Unknown)
            } else if is_number_like(left) && is_number_like(right) {
                Some(match operator {
                    SemanticOperator::Arithmetic(_) => SemanticType::Number,
                    SemanticOperator::Comparison(_) => SemanticType::Boolean,
                    _ => unreachable!("operator category was matched above"),
                })
            } else {
                None
            }
        }
        SemanticOperator::Logical(_) => {
            let [left, right] = operands else {
                return None;
            };
            if left == &SemanticType::Unknown || right == &SemanticType::Unknown {
                Some(SemanticType::Unknown)
            } else if is_boolean_like(left) && is_boolean_like(right) {
                Some(SemanticType::Boolean)
            } else {
                None
            }
        }
        SemanticOperator::NullishCoalescing => {
            let [left, right] = operands else {
                return None;
            };
            if left == &SemanticType::Unknown || right == &SemanticType::Unknown {
                return Some(SemanticType::Unknown);
            }
            let left = remove_null(left);
            if left == SemanticType::Never {
                Some(right.clone())
            } else if left == *right {
                Some(left)
            } else {
                Some(SemanticType::Union(vec![left, right.clone()]))
            }
        }
        SemanticOperator::Unary(crate::component_graph::UnaryOperator::Not) => {
            let [operand] = operands else {
                return None;
            };
            if operand == &SemanticType::Unknown {
                Some(SemanticType::Unknown)
            } else if is_boolean_like(operand) {
                Some(SemanticType::Boolean)
            } else {
                None
            }
        }
        SemanticOperator::Unary(
            crate::component_graph::UnaryOperator::Plus
            | crate::component_graph::UnaryOperator::Minus,
        ) => {
            let [operand] = operands else {
                return None;
            };
            if operand == &SemanticType::Unknown {
                Some(SemanticType::Unknown)
            } else if is_number_like(operand) {
                Some(SemanticType::Number)
            } else {
                None
            }
        }
    }
}

fn is_number_like(semantic_type: &SemanticType) -> bool {
    matches!(
        semantic_type,
        SemanticType::Number | SemanticType::NumberLiteral(_)
    )
}

fn is_boolean_like(semantic_type: &SemanticType) -> bool {
    matches!(
        semantic_type,
        SemanticType::Boolean | SemanticType::BooleanLiteral(_)
    )
}

fn remove_null(semantic_type: &SemanticType) -> SemanticType {
    match semantic_type {
        SemanticType::Null => SemanticType::Never,
        SemanticType::Union(members) => {
            let members = members
                .iter()
                .map(remove_null)
                .filter(|member| member != &SemanticType::Never)
                .collect::<Vec<_>>();
            match members.as_slice() {
                [] => SemanticType::Never,
                [member] => member.clone(),
                _ => SemanticType::Union(members),
            }
        }
        _ => semantic_type.clone(),
    }
}

#[must_use]
pub fn infer_serializable_value_type(value: &SerializableValue) -> SemanticType {
    match value {
        SerializableValue::Null => SemanticType::Null,
        SerializableValue::Number(_) => SemanticType::Number,
        SerializableValue::String(_) => SemanticType::String,
        SerializableValue::Boolean(_) => SemanticType::Boolean,
        SerializableValue::Array(values) if values.is_empty() => {
            SemanticType::Array(Box::new(SemanticType::Unknown))
        }
        SerializableValue::Array(values) => {
            let mut types = values
                .iter()
                .map(infer_serializable_value_type)
                .collect::<Vec<_>>();
            let element = if types.windows(2).all(|window| window[0] == window[1]) {
                types.pop().unwrap_or(SemanticType::Unknown)
            } else {
                SemanticType::Union(types)
            };
            SemanticType::Array(Box::new(element))
        }
        SerializableValue::Object(values) => SemanticType::Object(ObjectType {
            properties: values
                .iter()
                .map(|(name, value)| (name.clone(), infer_serializable_value_type(value)))
                .collect(),
        }),
    }
}

fn semantic_type_from_annotation(text: &str) -> Option<SemanticType> {
    let text = text.trim();
    let union_members = split_top_level(text, '|');
    if union_members.len() > 1 {
        return Some(SemanticType::Union(
            union_members
                .into_iter()
                .map(|member| {
                    semantic_type_from_annotation(member).unwrap_or(SemanticType::Unknown)
                })
                .collect(),
        ));
    }
    if let Some(element) = text.strip_suffix("[]") {
        return Some(SemanticType::Array(Box::new(
            semantic_type_from_annotation(element).unwrap_or(SemanticType::Unknown),
        )));
    }
    if text.starts_with('[') && text.ends_with(']') {
        let items = &text[1..text.len() - 1];
        return Some(SemanticType::Tuple(
            split_top_level(items, ',')
                .into_iter()
                .filter(|item| !item.trim().is_empty())
                .map(|item| semantic_type_from_annotation(item).unwrap_or(SemanticType::Unknown))
                .collect(),
        ));
    }
    if text.starts_with('{') && text.ends_with('}') {
        return object_type(text);
    }

    match text {
        "string" => Some(SemanticType::String),
        "number" => Some(SemanticType::Number),
        "boolean" => Some(SemanticType::Boolean),
        "null" => Some(SemanticType::Null),
        "SlotContent" => Some(SemanticType::SlotContent),
        "true" => Some(SemanticType::BooleanLiteral(true)),
        "false" => Some(SemanticType::BooleanLiteral(false)),
        _ => string_literal_type(text).or_else(|| numeric_literal_type(text)),
    }
}

fn object_type(text: &str) -> Option<SemanticType> {
    let mut properties = BTreeMap::new();
    let fields = &text[1..text.len() - 1];

    for field in split_top_level(fields, ';')
        .into_iter()
        .filter(|field| !field.trim().is_empty())
    {
        let (name, type_text) = field.split_once(':')?;
        let name = name.trim();
        if name.is_empty() {
            return None;
        }
        properties.insert(
            name.to_string(),
            semantic_type_from_annotation(type_text).unwrap_or(SemanticType::Unknown),
        );
    }

    Some(SemanticType::Object(ObjectType { properties }))
}

fn split_top_level(text: &str, delimiter: char) -> Vec<&str> {
    let mut depth = 0usize;
    let mut start = 0usize;
    let mut parts = Vec::new();

    for (index, character) in text.char_indices() {
        match character {
            '{' | '[' | '(' => depth += 1,
            '}' | ']' | ')' => depth = depth.saturating_sub(1),
            _ => {}
        }
        if character == delimiter && depth == 0 {
            parts.push(&text[start..index]);
            start = index + character.len_utf8();
        }
    }
    parts.push(&text[start..]);
    parts
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

/// Canonical semantic compatibility relation shared by compiler consumers.
#[must_use]
pub fn is_assignable(source: &SemanticType, target: &SemanticType) -> bool {
    match (source, target) {
        (_, SemanticType::Unknown)
        | (SemanticType::Unknown | SemanticType::Never, _)
        | (SemanticType::Null, SemanticType::Null)
        | (SemanticType::Boolean | SemanticType::BooleanLiteral(_), SemanticType::Boolean)
        | (SemanticType::Number | SemanticType::NumberLiteral(_), SemanticType::Number)
        | (SemanticType::String | SemanticType::StringLiteral(_), SemanticType::String)
        | (SemanticType::Form, SemanticType::Form)
        | (SemanticType::SlotContent, SemanticType::SlotContent) => true,
        (SemanticType::BooleanLiteral(source), SemanticType::BooleanLiteral(target)) => {
            source == target
        }
        (SemanticType::NumberLiteral(source), SemanticType::NumberLiteral(target))
        | (SemanticType::StringLiteral(source), SemanticType::StringLiteral(target)) => {
            source == target
        }
        (SemanticType::Tuple(source), SemanticType::Tuple(target)) => {
            source.len() == target.len()
                && source
                    .iter()
                    .zip(target)
                    .all(|(source, target)| is_assignable(source, target))
        }
        (SemanticType::Tuple(source), SemanticType::Array(target)) => {
            source.iter().all(|source| is_assignable(source, target))
        }
        (SemanticType::Array(source), SemanticType::Array(target)) => is_assignable(source, target),
        (SemanticType::Object(source), SemanticType::Object(target)) => {
            target.properties.iter().all(|(name, target)| {
                source
                    .properties
                    .get(name)
                    .is_some_and(|source| is_assignable(source, target))
            })
        }
        (SemanticType::Resource(source), SemanticType::Resource(target)) => {
            source.pending == target.pending
                && source.serializable == target.serializable
                && source.execution_boundary == target.execution_boundary
                && is_assignable(&source.data, &target.data)
                && is_assignable(&source.error, &target.error)
        }
        (SemanticType::Union(source), target) => {
            source.iter().all(|source| is_assignable(source, target))
        }
        (source, SemanticType::Union(target)) => {
            target.iter().any(|target| is_assignable(source, target))
        }
        _ => false,
    }
}

/// Compatibility alias retained for C10 callers.
#[must_use]
pub fn is_state_initializer_assignable(source: &SemanticType, target: &SemanticType) -> bool {
    is_assignable(source, target)
}

#[must_use]
pub fn state_initializer_value_type(value: &SerializableValue) -> SemanticType {
    match value {
        SerializableValue::Null => SemanticType::Null,
        SerializableValue::Number(value) => SemanticType::NumberLiteral(value.clone()),
        SerializableValue::String(value) => SemanticType::StringLiteral(value.clone()),
        SerializableValue::Boolean(value) => SemanticType::BooleanLiteral(*value),
        SerializableValue::Array(values) if values.is_empty() => {
            SemanticType::Array(Box::new(SemanticType::Unknown))
        }
        SerializableValue::Array(values) => {
            SemanticType::Tuple(values.iter().map(state_initializer_value_type).collect())
        }
        SerializableValue::Object(values) => SemanticType::Object(ObjectType {
            properties: values
                .iter()
                .map(|(name, value)| (name.clone(), state_initializer_value_type(value)))
                .collect(),
        }),
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::{
        boundary_compatibility, is_assignable, normalize_semantic_type, operator_result_type,
        serialization_compatibility, BoundaryCompatibility, ExecutionBoundary, ObjectType,
        ResourceExecutionBoundary, ResourceType, SemanticOperator, SemanticType,
        SemanticTypeAssignment, SemanticTypeId, SemanticTypeStatus, SerializationCompatibility,
        TypeDiagnosticCode, TypeDiagnosticFamily,
    };
    use crate::{
        component_graph::UnaryOperator, ArithmeticOperator, ComparisonOperator, LogicalOperator,
        SemanticId, SourceProvenance,
    };

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
            SemanticType::Tuple(vec![SemanticType::String, SemanticType::Number]),
            todo,
            SemanticType::Union(vec![SemanticType::String, SemanticType::Null]),
            SemanticType::Resource(ResourceType {
                data: Box::new(SemanticType::String),
                error: Box::new(SemanticType::Unknown),
                pending: true,
                serializable: true,
                execution_boundary: ResourceExecutionBoundary::Shared,
            }),
        ];

        assert_eq!(types.len(), 14);
    }

    #[test]
    fn assigns_stable_codes_to_canonical_type_diagnostic_families() {
        assert_eq!(TypeDiagnosticCode::UnknownType.as_str(), "PSC1032");
        assert_eq!(
            TypeDiagnosticCode::InvalidArithmeticOperator.family(),
            TypeDiagnosticFamily::InvalidOperator
        );
        assert_eq!(
            TypeDiagnosticCode::IncompatibleAssignment.family(),
            TypeDiagnosticFamily::IncompatibleAssignment
        );
        assert_eq!(
            TypeDiagnosticCode::NonSerializableState.family(),
            TypeDiagnosticFamily::NonSerializableState
        );
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
                presolve_parser::SourceSpan {
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

    #[test]
    fn determines_structural_serialization_compatibility() {
        assert_eq!(
            serialization_compatibility(&SemanticType::Number),
            SerializationCompatibility::Serializable
        );
        assert_eq!(
            serialization_compatibility(&SemanticType::Object(ObjectType {
                properties: BTreeMap::from([(
                    "todos".to_string(),
                    SemanticType::Array(Box::new(SemanticType::String)),
                )]),
            })),
            SerializationCompatibility::Serializable
        );
        assert_eq!(
            serialization_compatibility(&SemanticType::Unknown),
            SerializationCompatibility::NotSerializable
        );
        assert_eq!(
            serialization_compatibility(&SemanticType::Resource(ResourceType {
                data: Box::new(SemanticType::String),
                error: Box::new(SemanticType::Unknown),
                pending: true,
                serializable: true,
                execution_boundary: ResourceExecutionBoundary::Shared,
            })),
            SerializationCompatibility::NotSerializable
        );
    }

    #[test]
    fn determines_server_client_boundary_compatibility() {
        assert_eq!(
            boundary_compatibility(
                &SemanticType::String,
                ExecutionBoundary::Server,
                ExecutionBoundary::Client,
            ),
            BoundaryCompatibility::Compatible
        );
        assert_eq!(
            boundary_compatibility(
                &SemanticType::Resource(ResourceType {
                    data: Box::new(SemanticType::String),
                    error: Box::new(SemanticType::String),
                    pending: true,
                    serializable: true,
                    execution_boundary: ResourceExecutionBoundary::Client,
                }),
                ExecutionBoundary::Client,
                ExecutionBoundary::Server,
            ),
            BoundaryCompatibility::Incompatible
        );
    }

    #[test]
    fn normalizes_equivalent_union_forms_deterministically() {
        assert_eq!(
            normalize_semantic_type(SemanticType::Union(vec![
                SemanticType::String,
                SemanticType::Never,
                SemanticType::Union(vec![SemanticType::String, SemanticType::Null]),
                SemanticType::Null,
            ])),
            SemanticType::Union(vec![SemanticType::Null, SemanticType::String])
        );
        assert_eq!(
            normalize_semantic_type(SemanticType::Union(vec![SemanticType::Never])),
            SemanticType::Never
        );
    }

    #[test]
    fn centralizes_assignability_across_normalized_type_forms() {
        assert!(is_assignable(
            &SemanticType::StringLiteral("all".to_string()),
            &SemanticType::Union(vec![
                SemanticType::StringLiteral("active".to_string()),
                SemanticType::StringLiteral("all".to_string()),
            ])
        ));
        assert!(is_assignable(
            &SemanticType::Tuple(vec![SemanticType::NumberLiteral("1".to_string())]),
            &SemanticType::Array(Box::new(SemanticType::Number))
        ));
        assert!(!is_assignable(&SemanticType::String, &SemanticType::Number));
    }

    #[test]
    fn defines_explicit_operand_and_result_contracts_for_operators() {
        assert_eq!(
            operator_result_type(
                SemanticOperator::Arithmetic(ArithmeticOperator::Add),
                &[
                    SemanticType::NumberLiteral("1".to_string()),
                    SemanticType::Number,
                ],
            ),
            Some(SemanticType::Number)
        );
        assert_eq!(
            operator_result_type(
                SemanticOperator::Comparison(ComparisonOperator::LessThan),
                &[
                    SemanticType::Number,
                    SemanticType::NumberLiteral("2".to_string())
                ],
            ),
            Some(SemanticType::Boolean)
        );
        assert_eq!(
            operator_result_type(
                SemanticOperator::Logical(LogicalOperator::And),
                &[SemanticType::BooleanLiteral(true), SemanticType::Boolean],
            ),
            Some(SemanticType::Boolean)
        );
        assert_eq!(
            operator_result_type(
                SemanticOperator::Unary(UnaryOperator::Not),
                &[SemanticType::BooleanLiteral(false)],
            ),
            Some(SemanticType::Boolean)
        );
        assert_eq!(
            operator_result_type(
                SemanticOperator::NullishCoalescing,
                &[
                    SemanticType::Union(vec![SemanticType::String, SemanticType::Null]),
                    SemanticType::StringLiteral("fallback".to_string()),
                ],
            ),
            Some(SemanticType::Union(vec![
                SemanticType::String,
                SemanticType::StringLiteral("fallback".to_string()),
            ]))
        );
        assert_eq!(
            operator_result_type(
                SemanticOperator::Arithmetic(ArithmeticOperator::Add),
                &[SemanticType::String, SemanticType::Number],
            ),
            None
        );
        assert_eq!(
            operator_result_type(
                SemanticOperator::Logical(LogicalOperator::Or),
                &[SemanticType::Number, SemanticType::Boolean],
            ),
            None
        );
    }
}
