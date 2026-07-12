use std::collections::BTreeMap;
use std::fmt;
use std::path::PathBuf;

use crate::{
    BindingTable, ComponentNode, ImportBindingTarget, SemanticId, SerializableValue,
    SourceProvenance, SymbolKind,
};
use crate::{
    ExpressionGraph, ExpressionNodeKind, SemanticReference, SemanticReferenceKind,
    TemplateSemanticEntity, TemplateSemanticKind,
};
use ezc_parser::ParsedTypeAlias;

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
    BooleanLiteral(bool),
    NumberLiteral(String),
    StringLiteral(String),
    Array(Box<SemanticType>),
    Tuple(Vec<SemanticType>),
    Object(ObjectType),
    Union(Vec<SemanticType>),
    Resource(ResourceType),
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

/// Determines structural serialization compatibility for a canonical type.
#[must_use]
pub fn serialization_compatibility(semantic_type: &SemanticType) -> SerializationCompatibility {
    let serializable = match semantic_type {
        SemanticType::Unknown | SemanticType::Never => false,
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
    pub method: SemanticId,
    pub semantic_type: SemanticType,
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

impl SemanticTypeModel {
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
                        semantic_type_from_serializable_value(value),
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
            .with_computed_value_types(components)
            .with_action_signature_types(components)
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
        }
    }

    fn with_computed_value_types(mut self, components: &[ComponentNode]) -> Self {
        for method in components
            .iter()
            .flat_map(|component| &component.methods)
            .filter(|method| method.is_computed())
        {
            let Some(assignment) = self.assignments.get(&method.id) else {
                continue;
            };
            self.computed_values.insert(
                method.id.clone(),
                ComputedValueType {
                    method: method.id.clone(),
                    semantic_type: assignment.semantic_type.clone(),
                    provenance: assignment.provenance.clone(),
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

    /// Attaches inferred types to every canonical expression node in `graph`.
    ///
    /// # Panics
    ///
    /// Panics when the graph's node index contains an ID that does not resolve
    /// to an expression node.
    #[must_use]
    pub fn with_expression_types(mut self, graph: &ExpressionGraph) -> Self {
        let nodes = graph.nodes.keys().cloned().collect::<Vec<_>>();
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
                        SemanticReferenceKind::TemplateState | SemanticReferenceKind::TemplateLocal
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
                    && reference.kind == SemanticReferenceKind::TemplateState
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
                semantic_type: semantic_type_from_serializable_value(&local.value),
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
        .map(semantic_type_from_serializable_value)
        .collect::<Vec<_>>();
    match types.as_slice() {
        [] => None,
        [semantic_type] => Some(semantic_type.clone()),
        _ if types.windows(2).all(|window| window[0] == window[1]) => types.pop(),
        _ => Some(SemanticType::Union(types)),
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

fn semantic_type_from_serializable_value(value: &SerializableValue) -> SemanticType {
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
                .map(semantic_type_from_serializable_value)
                .collect::<Vec<_>>();
            let element = if types.windows(2).all(|window| window[0] == window[1]) {
                types.pop().expect("non-empty array should have a type")
            } else {
                SemanticType::Union(types)
            };
            SemanticType::Array(Box::new(element))
        }
        SerializableValue::Object(values) => SemanticType::Object(ObjectType {
            properties: values
                .iter()
                .map(|(name, value)| (name.clone(), semantic_type_from_serializable_value(value)))
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

/// C10 compatibility relation for one concrete state initializer value.
#[must_use]
pub fn is_state_initializer_assignable(source: &SemanticType, target: &SemanticType) -> bool {
    match (source, target) {
        (_, SemanticType::Unknown)
        | (SemanticType::Unknown | SemanticType::Never, _)
        | (SemanticType::Null, SemanticType::Null)
        | (SemanticType::Boolean | SemanticType::BooleanLiteral(_), SemanticType::Boolean)
        | (SemanticType::Number | SemanticType::NumberLiteral(_), SemanticType::Number)
        | (SemanticType::String | SemanticType::StringLiteral(_), SemanticType::String) => true,
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
                    .all(|(source, target)| is_state_initializer_assignable(source, target))
        }
        (SemanticType::Tuple(source), SemanticType::Array(target)) => source
            .iter()
            .all(|source| is_state_initializer_assignable(source, target)),
        (SemanticType::Array(source), SemanticType::Array(target)) => {
            is_state_initializer_assignable(source, target)
        }
        (SemanticType::Object(source), SemanticType::Object(target)) => {
            target.properties.iter().all(|(name, target)| {
                source
                    .properties
                    .get(name)
                    .is_some_and(|source| is_state_initializer_assignable(source, target))
            })
        }
        (SemanticType::Resource(source), SemanticType::Resource(target)) => {
            source.pending == target.pending
                && source.serializable == target.serializable
                && source.execution_boundary == target.execution_boundary
                && is_state_initializer_assignable(&source.data, &target.data)
                && is_state_initializer_assignable(&source.error, &target.error)
        }
        (SemanticType::Union(source), target) => source
            .iter()
            .all(|source| is_state_initializer_assignable(source, target)),
        (source, SemanticType::Union(target)) => target
            .iter()
            .any(|target| is_state_initializer_assignable(source, target)),
        _ => false,
    }
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
        operator_result_type, serialization_compatibility, ObjectType, ResourceExecutionBoundary,
        ResourceType, SemanticOperator, SemanticType, SemanticTypeAssignment, SemanticTypeId,
        SemanticTypeStatus, SerializationCompatibility,
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
