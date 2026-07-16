use std::collections::{BTreeMap, BTreeSet};

use crate::{
    is_assignable, serialization_compatibility, AuthoredDeclarationKind,
    AuthoredValidationRuleArgumentKind, AuthoredValidationRuleDeclarationFact,
    AuthoredValidationRuleExpressionKind, ComponentBuildRoot, ComponentNode, ComponentRootId,
    ExecutionBoundary, FieldId, FormEntity, FormFieldEntity, FormId, FormOwnershipGraph,
    FormOwnershipNodeKey, SemanticId, SemanticOwner, SemanticReference, SemanticReferenceKind,
    SemanticType, SerializableValue, SerializationCompatibility, SourceProvenance,
    ValidationDependencyCycleId, ValidationGraphId, ValidationRuleCandidateId, ValidationRuleId,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ValidationRuleKind {
    Required,
    Min,
    Max,
    MinLength,
    MaxLength,
    Pattern,
    Email,
    Equals,
    NotEquals,
}

impl ValidationRuleKind {
    fn from_name(name: &str) -> Option<Self> {
        match name {
            "required" => Some(Self::Required),
            "min" => Some(Self::Min),
            "max" => Some(Self::Max),
            "minLength" => Some(Self::MinLength),
            "maxLength" => Some(Self::MaxLength),
            "pattern" => Some(Self::Pattern),
            "email" => Some(Self::Email),
            "equals" => Some(Self::Equals),
            "notEquals" => Some(Self::NotEquals),
            _ => None,
        }
    }

    const fn expected_arity(self) -> usize {
        match self {
            Self::Required | Self::Email => 0,
            Self::Min
            | Self::Max
            | Self::MinLength
            | Self::MaxLength
            | Self::Pattern
            | Self::Equals
            | Self::NotEquals => 1,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum ValidationRuleArgument {
    None,
    Number(String),
    Length(u64),
    Pattern(String),
    Field(FieldId),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValidationCompatibility {
    Compatible,
    Incompatible {
        kind: ValidationRuleKind,
        field_type: SemanticType,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum ValidationRuleViolation {
    InvalidOwner,
    InvalidTarget { actual: AuthoredDeclarationKind },
    StaticField,
    InvalidFieldDeclaration,
    InvalidDecoratorInvocation,
    InvalidDecoratorArity { actual: usize, expected: usize },
    InvalidRuleExpression,
    UnknownRule,
    ShadowedCompilerRule,
    InvalidRuleArity { actual: usize, expected: usize },
    UnsupportedArgument,
    InvalidConstantArgument,
    InvalidPattern,
    UnresolvedDependency,
    CrossComponentDependency,
    CrossFormDependency,
    SelfDependency,
    IncompatibleType,
    DuplicateRule,
    ContradictoryRule,
    DependencyCycle,
    ConflictingSemanticDecorator,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationDependencyDesignator {
    pub authored_name: String,
    pub provenance: SourceProvenance,
    pub name_provenance: SourceProvenance,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationRuleCandidate {
    pub id: ValidationRuleCandidateId,
    pub rule_id: Option<ValidationRuleId>,
    pub owner_component: Option<SemanticId>,
    pub target_declaration_field: Option<SemanticId>,
    pub target_field: Option<FieldId>,
    pub target_form: Option<FormId>,
    pub authored_target_name: Option<String>,
    pub declaration_kind: AuthoredDeclarationKind,
    pub is_static: bool,
    pub authored_ordinal: usize,
    pub kind: Option<ValidationRuleKind>,
    pub argument: Option<ValidationRuleArgument>,
    pub dependency_designator: Option<ValidationDependencyDesignator>,
    pub resolved_dependency: Option<FieldId>,
    pub compatibility: Option<ValidationCompatibility>,
    pub conflicting_decorators: Vec<String>,
    pub decorator_provenance: SourceProvenance,
    pub rule_expression_provenance: Option<SourceProvenance>,
    pub argument_provenance: Option<SourceProvenance>,
    pub target_provenance: SourceProvenance,
    pub target_name_provenance: Option<SourceProvenance>,
    pub violations: Vec<ValidationRuleViolation>,
}

impl ValidationRuleCandidate {
    #[must_use]
    pub fn is_valid(&self) -> bool {
        self.violations.is_empty()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationRule {
    pub id: ValidationRuleId,
    pub candidate_id: ValidationRuleCandidateId,
    pub owner_form: FormId,
    pub target_field: FieldId,
    pub owner_component: SemanticId,
    pub kind: ValidationRuleKind,
    pub argument: ValidationRuleArgument,
    pub dependency: Option<FieldId>,
    pub compatibility: ValidationCompatibility,
    pub field_authored_order: usize,
    pub rule_authored_order: usize,
    pub provenance: SourceProvenance,
    pub decorator_provenance: SourceProvenance,
    pub argument_provenance: Option<SourceProvenance>,
    pub boundary: ExecutionBoundary,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationDependencyCycle {
    pub id: ValidationDependencyCycleId,
    pub form: FormId,
    pub fields: Vec<FieldId>,
    pub candidates: Vec<ValidationRuleCandidateId>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationProducts {
    pub candidates: Vec<ValidationRuleCandidate>,
    pub rules: BTreeMap<ValidationRuleId, ValidationRule>,
    pub cycles: Vec<ValidationDependencyCycle>,
}

/// Lowers normalized parser facts through canonical I3 field products. No
/// parser syntax or template/DOM state is consulted.
///
/// # Panics
///
/// Panics only when a candidate classified as valid is missing one of the
/// canonical target, form, type, ordering, or provenance facts required by
/// the I6 staged-lowering invariant.
#[allow(clippy::too_many_lines)]
#[must_use]
pub fn collect_validation_products(
    components: &[ComponentNode],
    forms: &BTreeMap<FormId, FormEntity>,
    fields: &BTreeMap<FieldId, FormFieldEntity>,
) -> ValidationProducts {
    let mut facts = components
        .iter()
        .flat_map(|component| component.validation_rule_declaration_facts.iter())
        .cloned()
        .collect::<Vec<_>>();
    facts.sort_by(fact_source_order);

    let fields_by_declaration = fields
        .values()
        .map(|field| (field.authored_field.clone(), field))
        .collect::<BTreeMap<_, _>>();
    let fields_by_component_name = fields
        .values()
        .map(|field| ((field.owner_component.clone(), field.name.clone()), field))
        .collect::<BTreeMap<_, _>>();
    let fields_by_name = fields.values().fold(
        BTreeMap::<String, Vec<&FormFieldEntity>>::new(),
        |mut grouped, field| {
            grouped.entry(field.name.clone()).or_default().push(field);
            grouped
        },
    );
    let shadowed_intrinsics = components
        .iter()
        .map(|component| {
            let mut shadows = component.shadowed_validation_intrinsics.clone();
            shadows.extend(component.methods.iter().map(|method| method.name.clone()));
            (component.id.clone(), shadows)
        })
        .collect::<BTreeMap<_, _>>();

    let mut candidates = facts
        .iter()
        .map(|fact| {
            lower_candidate(
                fact,
                forms,
                &fields_by_declaration,
                &fields_by_component_name,
                &fields_by_name,
                &shadowed_intrinsics,
            )
        })
        .collect::<Vec<_>>();

    mark_duplicate_rules(&mut candidates);
    mark_contradictions(&mut candidates);
    let cycles = mark_dependency_cycles(&mut candidates);

    let field_orders = fields
        .values()
        .map(|field| (field.id.clone(), field.declaration_order))
        .collect::<BTreeMap<_, _>>();
    let mut rules = BTreeMap::new();
    for candidate in &mut candidates {
        if !candidate.is_valid() {
            candidate.rule_id = None;
            continue;
        }
        let target_field = candidate
            .target_field
            .clone()
            .expect("valid validation candidate has target field");
        let id = ValidationRuleId::for_field(&target_field, candidate.authored_ordinal);
        let rule = ValidationRule {
            id: id.clone(),
            candidate_id: candidate.id.clone(),
            owner_form: candidate
                .target_form
                .clone()
                .expect("valid validation candidate has target form"),
            target_field: target_field.clone(),
            owner_component: candidate
                .owner_component
                .clone()
                .expect("valid validation candidate has component"),
            kind: candidate
                .kind
                .expect("valid validation candidate has rule kind"),
            argument: candidate
                .argument
                .clone()
                .expect("valid validation candidate has normalized argument"),
            dependency: candidate.resolved_dependency.clone(),
            compatibility: candidate
                .compatibility
                .clone()
                .expect("valid validation candidate has compatibility"),
            field_authored_order: *field_orders
                .get(&target_field)
                .expect("valid validation target has authored order"),
            rule_authored_order: candidate.authored_ordinal,
            provenance: candidate.target_provenance.clone(),
            decorator_provenance: candidate.decorator_provenance.clone(),
            argument_provenance: candidate.argument_provenance.clone(),
            boundary: ExecutionBoundary::Client,
        };
        candidate.rule_id = Some(id.clone());
        rules.insert(id, rule);
    }

    candidates.sort_by(candidate_source_order);
    ValidationProducts {
        candidates,
        rules,
        cycles,
    }
}

#[allow(clippy::too_many_lines)]
fn lower_candidate(
    fact: &AuthoredValidationRuleDeclarationFact,
    forms: &BTreeMap<FormId, FormEntity>,
    fields_by_declaration: &BTreeMap<SemanticId, &FormFieldEntity>,
    fields_by_component_name: &BTreeMap<(SemanticId, String), &FormFieldEntity>,
    fields_by_name: &BTreeMap<String, Vec<&FormFieldEntity>>,
    shadowed_intrinsics: &BTreeMap<SemanticId, BTreeSet<String>>,
) -> ValidationRuleCandidate {
    let target = fact
        .declaration_field
        .as_ref()
        .and_then(|id| fields_by_declaration.get(id).copied())
        .filter(|field| forms.contains_key(&field.owner_form));
    let mut violations = Vec::new();
    if fact.owner_component.is_none() {
        violations.push(ValidationRuleViolation::InvalidOwner);
    }
    if fact.declaration_kind != AuthoredDeclarationKind::InstanceField {
        violations.push(ValidationRuleViolation::InvalidTarget {
            actual: fact.declaration_kind,
        });
    }
    if fact.is_static {
        violations.push(ValidationRuleViolation::StaticField);
    }
    if target.is_none() {
        violations.push(ValidationRuleViolation::InvalidFieldDeclaration);
    }
    if !fact.decorator_invoked {
        violations.push(ValidationRuleViolation::InvalidDecoratorInvocation);
    }
    if fact.decorator_argument_count != 1 {
        violations.push(ValidationRuleViolation::InvalidDecoratorArity {
            actual: fact.decorator_argument_count,
            expected: 1,
        });
    }
    if !fact.conflicting_decorators.is_empty() {
        violations.push(ValidationRuleViolation::ConflictingSemanticDecorator);
    }

    let mut kind = None;
    let mut argument = None;
    let mut dependency_designator = None;
    let mut argument_provenance = None;
    let expression_provenance = fact
        .expression
        .as_ref()
        .map(|expression| expression.provenance.clone());

    match fact.expression.as_ref().map(|expression| &expression.kind) {
        Some(AuthoredValidationRuleExpressionKind::Call { callee, arguments }) => {
            let Some(callee) = callee.as_deref() else {
                violations.push(ValidationRuleViolation::InvalidRuleExpression);
                canonicalize_violations(&mut violations);
                return candidate_from_parts(
                    fact,
                    target,
                    kind,
                    argument,
                    dependency_designator,
                    None,
                    None,
                    expression_provenance,
                    argument_provenance,
                    violations,
                );
            };
            let Some(classified) = ValidationRuleKind::from_name(callee) else {
                violations.push(ValidationRuleViolation::UnknownRule);
                canonicalize_violations(&mut violations);
                return candidate_from_parts(
                    fact,
                    target,
                    kind,
                    argument,
                    dependency_designator,
                    None,
                    None,
                    expression_provenance,
                    argument_provenance,
                    violations,
                );
            };
            kind = Some(classified);
            if fact.owner_component.as_ref().is_some_and(|component| {
                shadowed_intrinsics
                    .get(component)
                    .is_some_and(|methods| methods.contains(callee))
            }) {
                violations.push(ValidationRuleViolation::ShadowedCompilerRule);
            }
            if arguments.len() == classified.expected_arity() {
                match normalize_argument(classified, arguments) {
                    Ok((normalized, designator, provenance)) => {
                        argument = Some(normalized);
                        dependency_designator = designator;
                        argument_provenance = provenance;
                    }
                    Err(violation) => violations.push(violation),
                }
            } else {
                violations.push(ValidationRuleViolation::InvalidRuleArity {
                    actual: arguments.len(),
                    expected: classified.expected_arity(),
                });
            }
        }
        Some(
            AuthoredValidationRuleExpressionKind::Identifier(_)
            | AuthoredValidationRuleExpressionKind::Unsupported,
        )
        | None => violations.push(ValidationRuleViolation::InvalidRuleExpression),
    }

    let mut resolved_dependency = None;
    if let (Some(owner), Some(target), Some(designator)) = (
        fact.owner_component.as_ref(),
        target,
        dependency_designator.as_ref(),
    ) {
        if let Some(dependency) = fields_by_component_name
            .get(&(owner.clone(), designator.authored_name.clone()))
            .copied()
        {
            if dependency.id == target.id {
                violations.push(ValidationRuleViolation::SelfDependency);
            } else if dependency.owner_form != target.owner_form {
                violations.push(ValidationRuleViolation::CrossFormDependency);
            } else {
                resolved_dependency = Some(dependency.id.clone());
                argument = Some(ValidationRuleArgument::Field(dependency.id.clone()));
            }
        } else if fields_by_name
            .get(&designator.authored_name)
            .is_some_and(|matches| matches.iter().any(|field| &field.owner_component != owner))
        {
            violations.push(ValidationRuleViolation::CrossComponentDependency);
        } else {
            violations.push(ValidationRuleViolation::UnresolvedDependency);
        }
    }

    let compatibility = kind.zip(target).map(|(kind, target)| {
        let dependency = resolved_dependency.as_ref().and_then(|id| {
            fields_by_declaration
                .values()
                .copied()
                .find(|field| &field.id == id)
        });
        if rule_is_compatible(
            kind,
            &target.semantic_type,
            dependency.map(|field| &field.semantic_type),
        ) {
            ValidationCompatibility::Compatible
        } else {
            violations.push(ValidationRuleViolation::IncompatibleType);
            ValidationCompatibility::Incompatible {
                kind,
                field_type: target.semantic_type.clone(),
            }
        }
    });

    canonicalize_violations(&mut violations);
    candidate_from_parts(
        fact,
        target,
        kind,
        argument,
        dependency_designator,
        resolved_dependency,
        compatibility,
        expression_provenance,
        argument_provenance,
        violations,
    )
}

#[allow(clippy::too_many_arguments)]
fn candidate_from_parts(
    fact: &AuthoredValidationRuleDeclarationFact,
    target: Option<&FormFieldEntity>,
    kind: Option<ValidationRuleKind>,
    argument: Option<ValidationRuleArgument>,
    dependency_designator: Option<ValidationDependencyDesignator>,
    resolved_dependency: Option<FieldId>,
    compatibility: Option<ValidationCompatibility>,
    rule_expression_provenance: Option<SourceProvenance>,
    argument_provenance: Option<SourceProvenance>,
    violations: Vec<ValidationRuleViolation>,
) -> ValidationRuleCandidate {
    ValidationRuleCandidate {
        id: fact.id.clone(),
        rule_id: None,
        owner_component: fact.owner_component.clone(),
        target_declaration_field: fact.declaration_field.clone(),
        target_field: target.map(|field| field.id.clone()),
        target_form: target.map(|field| field.owner_form.clone()),
        authored_target_name: fact.authored_name.clone(),
        declaration_kind: fact.declaration_kind,
        is_static: fact.is_static,
        authored_ordinal: fact.authored_ordinal,
        kind,
        argument,
        dependency_designator,
        resolved_dependency,
        compatibility,
        conflicting_decorators: fact.conflicting_decorators.clone(),
        decorator_provenance: fact.decorator_provenance.clone(),
        rule_expression_provenance,
        argument_provenance,
        target_provenance: fact.provenance.clone(),
        target_name_provenance: fact.name_provenance.clone(),
        violations,
    }
}

fn normalize_argument(
    kind: ValidationRuleKind,
    arguments: &[crate::AuthoredValidationRuleArgument],
) -> Result<
    (
        ValidationRuleArgument,
        Option<ValidationDependencyDesignator>,
        Option<SourceProvenance>,
    ),
    ValidationRuleViolation,
> {
    if arguments.is_empty() {
        return Ok((ValidationRuleArgument::None, None, None));
    }
    let argument = &arguments[0];
    let provenance = Some(argument.provenance.clone());
    match (kind, &argument.kind) {
        (
            ValidationRuleKind::Min | ValidationRuleKind::Max,
            AuthoredValidationRuleArgumentKind::Constant(expression),
        ) => {
            let number = constant_number(expression)?;
            Ok((ValidationRuleArgument::Number(number), None, provenance))
        }
        (
            ValidationRuleKind::MinLength | ValidationRuleKind::MaxLength,
            AuthoredValidationRuleArgumentKind::Constant(expression),
        ) => {
            let number = constant_number(expression)?;
            let number = number
                .parse::<u64>()
                .map_err(|_| ValidationRuleViolation::InvalidConstantArgument)?;
            Ok((ValidationRuleArgument::Length(number), None, provenance))
        }
        (
            ValidationRuleKind::Pattern,
            AuthoredValidationRuleArgumentKind::StringLiteral(pattern),
        ) => {
            if !ezc_parser::is_valid_ecmascript_pattern(pattern) {
                return Err(ValidationRuleViolation::InvalidPattern);
            }
            Ok((
                ValidationRuleArgument::Pattern(pattern.clone()),
                None,
                provenance,
            ))
        }
        (
            ValidationRuleKind::Equals | ValidationRuleKind::NotEquals,
            AuthoredValidationRuleArgumentKind::ThisMember {
                name,
                name_provenance,
            },
        ) => Ok((
            ValidationRuleArgument::None,
            Some(ValidationDependencyDesignator {
                authored_name: name.clone(),
                provenance: argument.provenance.clone(),
                name_provenance: name_provenance.clone(),
            }),
            provenance,
        )),
        _ => Err(ValidationRuleViolation::UnsupportedArgument),
    }
}

fn constant_number(
    expression: &crate::ConstantExpression,
) -> Result<String, ValidationRuleViolation> {
    let SerializableValue::Number(number) = expression
        .evaluate()
        .map_err(|_| ValidationRuleViolation::InvalidConstantArgument)?
    else {
        return Err(ValidationRuleViolation::InvalidConstantArgument);
    };
    let value = number
        .parse::<f64>()
        .map_err(|_| ValidationRuleViolation::InvalidConstantArgument)?;
    value
        .is_finite()
        .then(|| value.to_string())
        .ok_or(ValidationRuleViolation::InvalidConstantArgument)
}

fn rule_is_compatible(
    kind: ValidationRuleKind,
    target: &SemanticType,
    dependency: Option<&SemanticType>,
) -> bool {
    match kind {
        ValidationRuleKind::Required => {
            serialization_compatibility(target) == SerializationCompatibility::Serializable
                && !matches!(
                    target,
                    SemanticType::Null | SemanticType::Unknown | SemanticType::Never
                )
        }
        ValidationRuleKind::Min | ValidationRuleKind::Max => {
            type_has_domain(target, TypeDomain::Number)
        }
        ValidationRuleKind::MinLength | ValidationRuleKind::MaxLength => {
            length_domain(target).is_some()
        }
        ValidationRuleKind::Pattern | ValidationRuleKind::Email => {
            type_has_domain(target, TypeDomain::String)
        }
        ValidationRuleKind::Equals | ValidationRuleKind::NotEquals => {
            dependency.is_some_and(|dependency| {
                !contains_unknown_or_never(target)
                    && !contains_unknown_or_never(dependency)
                    && (is_assignable(target, dependency) || is_assignable(dependency, target))
            })
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TypeDomain {
    Number,
    String,
}

fn type_has_domain(semantic_type: &SemanticType, domain: TypeDomain) -> bool {
    let members = non_null_members(semantic_type);
    !members.is_empty()
        && members.iter().all(|member| {
            matches!(
                (domain, *member),
                (
                    TypeDomain::Number,
                    SemanticType::Number | SemanticType::NumberLiteral(_)
                ) | (
                    TypeDomain::String,
                    SemanticType::String | SemanticType::StringLiteral(_)
                )
            )
        })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LengthDomain {
    String,
    Sequence,
}

fn length_domain(semantic_type: &SemanticType) -> Option<LengthDomain> {
    let members = non_null_members(semantic_type);
    let mut domain = None;
    for member in members {
        let current = match member {
            SemanticType::String | SemanticType::StringLiteral(_) => LengthDomain::String,
            SemanticType::Array(_) | SemanticType::Tuple(_) => LengthDomain::Sequence,
            _ => return None,
        };
        if domain.is_some_and(|domain| domain != current) {
            return None;
        }
        domain = Some(current);
    }
    domain
}

fn non_null_members(semantic_type: &SemanticType) -> Vec<&SemanticType> {
    match semantic_type {
        SemanticType::Union(members) => members
            .iter()
            .filter(|member| !matches!(member, SemanticType::Null))
            .collect(),
        SemanticType::Null => Vec::new(),
        semantic_type => vec![semantic_type],
    }
}

fn contains_unknown_or_never(semantic_type: &SemanticType) -> bool {
    match semantic_type {
        SemanticType::Unknown | SemanticType::Never => true,
        SemanticType::Array(element) => contains_unknown_or_never(element),
        SemanticType::Tuple(items) | SemanticType::Union(items) => {
            items.iter().any(contains_unknown_or_never)
        }
        SemanticType::Object(object) => object.properties.values().any(contains_unknown_or_never),
        SemanticType::Resource(resource) => {
            contains_unknown_or_never(&resource.data) || contains_unknown_or_never(&resource.error)
        }
        _ => false,
    }
}

fn mark_duplicate_rules(candidates: &mut [ValidationRuleCandidate]) {
    let mut groups = BTreeMap::<
        (
            FieldId,
            ValidationRuleKind,
            ValidationRuleArgument,
            Option<FieldId>,
        ),
        Vec<usize>,
    >::new();
    for (index, candidate) in candidates.iter().enumerate() {
        if let (Some(field), Some(kind), Some(argument)) = (
            candidate.target_field.clone(),
            candidate.kind,
            candidate.argument.clone(),
        ) {
            groups
                .entry((field, kind, argument, candidate.resolved_dependency.clone()))
                .or_default()
                .push(index);
        }
    }
    for group in groups.values().filter(|group| group.len() > 1) {
        for &index in group {
            add_violation(
                &mut candidates[index],
                ValidationRuleViolation::DuplicateRule,
            );
        }
    }
}

fn mark_contradictions(candidates: &mut [ValidationRuleCandidate]) {
    let mut contradictory = BTreeSet::new();
    for (left_index, left) in candidates.iter().enumerate() {
        for (right_index, right) in candidates.iter().enumerate().skip(left_index + 1) {
            if left.target_field != right.target_field {
                continue;
            }
            let contradiction = match (left.kind, &left.argument, right.kind, &right.argument) {
                (
                    Some(ValidationRuleKind::Min),
                    Some(ValidationRuleArgument::Number(minimum)),
                    Some(ValidationRuleKind::Max),
                    Some(ValidationRuleArgument::Number(maximum)),
                )
                | (
                    Some(ValidationRuleKind::Max),
                    Some(ValidationRuleArgument::Number(maximum)),
                    Some(ValidationRuleKind::Min),
                    Some(ValidationRuleArgument::Number(minimum)),
                ) => numeric_value(minimum) > numeric_value(maximum),
                (
                    Some(ValidationRuleKind::MinLength),
                    Some(ValidationRuleArgument::Length(minimum)),
                    Some(ValidationRuleKind::MaxLength),
                    Some(ValidationRuleArgument::Length(maximum)),
                )
                | (
                    Some(ValidationRuleKind::MaxLength),
                    Some(ValidationRuleArgument::Length(maximum)),
                    Some(ValidationRuleKind::MinLength),
                    Some(ValidationRuleArgument::Length(minimum)),
                ) => minimum > maximum,
                (Some(ValidationRuleKind::Equals), _, Some(ValidationRuleKind::NotEquals), _)
                | (Some(ValidationRuleKind::NotEquals), _, Some(ValidationRuleKind::Equals), _) => {
                    left.resolved_dependency.is_some()
                        && left.resolved_dependency == right.resolved_dependency
                }
                _ => false,
            };
            if contradiction {
                contradictory.insert(left_index);
                contradictory.insert(right_index);
            }
        }
    }
    for index in contradictory {
        add_violation(
            &mut candidates[index],
            ValidationRuleViolation::ContradictoryRule,
        );
    }
}

fn numeric_value(value: &str) -> f64 {
    value
        .parse::<f64>()
        .expect("normalized validation number is finite")
}

fn mark_dependency_cycles(
    candidates: &mut [ValidationRuleCandidate],
) -> Vec<ValidationDependencyCycle> {
    let mut adjacency = BTreeMap::<FieldId, BTreeSet<FieldId>>::new();
    for candidate in candidates
        .iter()
        .filter(|candidate| candidate.violations.is_empty())
    {
        if let (Some(target), Some(dependency)) =
            (&candidate.target_field, &candidate.resolved_dependency)
        {
            adjacency
                .entry(target.clone())
                .or_default()
                .insert(dependency.clone());
            adjacency.entry(dependency.clone()).or_default();
        }
    }

    let all_fields = adjacency.keys().cloned().collect::<Vec<_>>();
    let mut assigned = BTreeSet::new();
    let mut cycles = Vec::new();
    for field in all_fields {
        if assigned.contains(&field) {
            continue;
        }
        let forward = reachable_fields(&field, &adjacency);
        let mut strongly_connected = forward
            .into_iter()
            .filter(|other| reachable_fields(other, &adjacency).contains(&field))
            .collect::<Vec<_>>();
        strongly_connected.sort();
        if strongly_connected.len() < 2 {
            assigned.insert(field);
            continue;
        }
        assigned.extend(strongly_connected.iter().cloned());
        let field_set = strongly_connected.iter().cloned().collect::<BTreeSet<_>>();
        let candidate_indexes = candidates
            .iter()
            .enumerate()
            .filter_map(|(index, candidate)| {
                let target = candidate.target_field.as_ref()?;
                let dependency = candidate.resolved_dependency.as_ref()?;
                (field_set.contains(target) && field_set.contains(dependency)).then_some(index)
            })
            .collect::<Vec<_>>();
        let mut candidate_ids = candidate_indexes
            .iter()
            .map(|index| candidates[*index].id.clone())
            .collect::<Vec<_>>();
        candidate_ids.sort();
        let form = candidates[*candidate_indexes.first().expect("cycle has rule")]
            .target_form
            .clone()
            .expect("cycle candidate has target form");
        for index in candidate_indexes {
            add_violation(
                &mut candidates[index],
                ValidationRuleViolation::DependencyCycle,
            );
        }
        cycles.push(ValidationDependencyCycle {
            id: ValidationDependencyCycleId::for_fields(&form, &strongly_connected),
            form,
            fields: strongly_connected,
            candidates: candidate_ids,
        });
    }
    cycles.sort_by(|left, right| left.id.cmp(&right.id));
    cycles
}

fn reachable_fields(
    start: &FieldId,
    adjacency: &BTreeMap<FieldId, BTreeSet<FieldId>>,
) -> BTreeSet<FieldId> {
    let mut visited = BTreeSet::new();
    let mut pending = vec![start.clone()];
    while let Some(field) = pending.pop() {
        if !visited.insert(field.clone()) {
            continue;
        }
        if let Some(next) = adjacency.get(&field) {
            pending.extend(next.iter().rev().cloned());
        }
    }
    visited
}

fn add_violation(candidate: &mut ValidationRuleCandidate, violation: ValidationRuleViolation) {
    candidate.violations.push(violation);
    canonicalize_violations(&mut candidate.violations);
}

fn canonicalize_violations(violations: &mut Vec<ValidationRuleViolation>) {
    violations.sort();
    violations.dedup();
}

fn fact_source_order(
    left: &AuthoredValidationRuleDeclarationFact,
    right: &AuthoredValidationRuleDeclarationFact,
) -> std::cmp::Ordering {
    (
        left.provenance.path.as_path(),
        left.decorator_provenance.span.start,
        left.id.as_str(),
    )
        .cmp(&(
            right.provenance.path.as_path(),
            right.decorator_provenance.span.start,
            right.id.as_str(),
        ))
}

fn candidate_source_order(
    left: &ValidationRuleCandidate,
    right: &ValidationRuleCandidate,
) -> std::cmp::Ordering {
    (
        left.target_provenance.path.as_path(),
        left.decorator_provenance.span.start,
        left.id.as_str(),
    )
        .cmp(&(
            right.target_provenance.path.as_path(),
            right.decorator_provenance.span.start,
            right.id.as_str(),
        ))
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum ValidationGraphNodeKey {
    Form(FormId),
    FormField(FieldId),
    ValidationRule(ValidationRuleId),
}

impl ValidationGraphNodeKey {
    #[must_use]
    pub fn semantic_id(&self) -> &SemanticId {
        match self {
            Self::Form(id) => id.as_semantic_id(),
            Self::FormField(id) => id.as_semantic_id(),
            Self::ValidationRule(id) => id.as_semantic_id(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValidationGraphNode {
    Form {
        id: FormId,
        provenance: SourceProvenance,
    },
    FormField {
        id: FieldId,
        provenance: SourceProvenance,
    },
    ValidationRule {
        id: ValidationRuleId,
        provenance: SourceProvenance,
    },
}

impl ValidationGraphNode {
    #[must_use]
    pub fn key(&self) -> ValidationGraphNodeKey {
        match self {
            Self::Form { id, .. } => ValidationGraphNodeKey::Form(id.clone()),
            Self::FormField { id, .. } => ValidationGraphNodeKey::FormField(id.clone()),
            Self::ValidationRule { id, .. } => ValidationGraphNodeKey::ValidationRule(id.clone()),
        }
    }

    #[must_use]
    pub const fn provenance(&self) -> &SourceProvenance {
        match self {
            Self::Form { provenance, .. }
            | Self::FormField { provenance, .. }
            | Self::ValidationRule { provenance, .. } => provenance,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ValidationGraphEdgeKind {
    FormOwnsField,
    FieldOwnsRule,
    RuleDependsOnField,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationGraphEdge {
    pub kind: ValidationGraphEdgeKind,
    pub source: ValidationGraphNodeKey,
    pub target: ValidationGraphNodeKey,
    pub provenance: SourceProvenance,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ValidationGraphIntegrityKind {
    DuplicateNode,
    MissingFormNode,
    MissingFieldNode,
    MissingRuleNode,
    UnknownEdgeEndpoint,
    MultipleRuleOwners,
    FieldFormMismatch,
    RuleTargetMismatch,
    DependencyMismatch,
    CrossFormDependency,
    CrossComponentDependency,
    SelfDependency,
    OwnershipCycle,
    DependencyCycleLeakage,
    InvalidCandidatePromoted,
    InstanceIdentityInValidationGraph,
    MissingProvenance,
    NonCanonicalOrdering,
    GraphIdentityMismatch,
}

impl ValidationGraphIntegrityKind {
    #[must_use]
    pub const fn code(self) -> &'static str {
        match self {
            Self::DuplicateNode => "EZASM1221",
            Self::MissingFormNode => "EZASM1222",
            Self::MissingFieldNode => "EZASM1223",
            Self::MissingRuleNode => "EZASM1224",
            Self::UnknownEdgeEndpoint => "EZASM1225",
            Self::MultipleRuleOwners => "EZASM1226",
            Self::FieldFormMismatch => "EZASM1227",
            Self::RuleTargetMismatch => "EZASM1228",
            Self::DependencyMismatch => "EZASM1229",
            Self::CrossFormDependency => "EZASM1230",
            Self::CrossComponentDependency => "EZASM1231",
            Self::SelfDependency => "EZASM1232",
            Self::OwnershipCycle => "EZASM1233",
            Self::DependencyCycleLeakage => "EZASM1234",
            Self::InvalidCandidatePromoted => "EZASM1235",
            Self::InstanceIdentityInValidationGraph => "EZASM1236",
            Self::MissingProvenance => "EZASM1237",
            Self::NonCanonicalOrdering => "EZASM1238",
            Self::GraphIdentityMismatch => "EZASM1239",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationGraphIntegrityDiagnostic {
    pub code: String,
    pub kind: ValidationGraphIntegrityKind,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationGraphValidation {
    pub diagnostics: Vec<ValidationGraphIntegrityDiagnostic>,
    pub is_valid: bool,
}

impl Default for ValidationGraphValidation {
    fn default() -> Self {
        Self {
            diagnostics: Vec::new(),
            is_valid: true,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationGraph {
    pub id: ValidationGraphId,
    pub nodes: BTreeMap<ValidationGraphNodeKey, ValidationGraphNode>,
    pub edges: Vec<ValidationGraphEdge>,
    pub cycles: Vec<ValidationDependencyCycle>,
    pub validation: ValidationGraphValidation,
}

impl ValidationGraph {
    #[must_use]
    pub fn node(&self, key: &ValidationGraphNodeKey) -> Option<&ValidationGraphNode> {
        self.nodes.get(key)
    }

    #[must_use]
    pub fn rules_of_field(&self, field: &FieldId) -> Vec<&ValidationRuleId> {
        let mut rules = self
            .edges
            .iter()
            .filter_map(|edge| {
                (edge.kind == ValidationGraphEdgeKind::FieldOwnsRule
                    && edge.source == ValidationGraphNodeKey::FormField(field.clone()))
                .then_some(match &edge.target {
                    ValidationGraphNodeKey::ValidationRule(id) => Some(id),
                    _ => None,
                })
                .flatten()
            })
            .collect::<Vec<_>>();
        rules.sort_by_key(|rule| validation_rule_ordinal(rule));
        rules
    }

    #[must_use]
    pub fn rules_of_form(&self, form: &FormId) -> Vec<&ValidationRuleId> {
        let fields = self
            .edges
            .iter()
            .filter_map(|edge| {
                (edge.kind == ValidationGraphEdgeKind::FormOwnsField
                    && edge.source == ValidationGraphNodeKey::Form(form.clone()))
                .then_some(match &edge.target {
                    ValidationGraphNodeKey::FormField(field) => Some(field),
                    _ => None,
                })
                .flatten()
            })
            .collect::<BTreeSet<_>>();
        let mut rules = self
            .edges
            .iter()
            .filter_map(|edge| {
                (edge.kind == ValidationGraphEdgeKind::FieldOwnsRule
                    && matches!(&edge.source, ValidationGraphNodeKey::FormField(field) if fields.contains(field)))
                .then_some(match &edge.target {
                    ValidationGraphNodeKey::ValidationRule(rule) => Some(rule),
                    _ => None,
                })
                .flatten()
            })
            .collect::<Vec<_>>();
        rules.sort();
        rules
    }

    #[must_use]
    pub fn target_of_rule(&self, rule: &ValidationRuleId) -> Option<&FieldId> {
        self.edges.iter().find_map(|edge| {
            (edge.kind == ValidationGraphEdgeKind::FieldOwnsRule
                && edge.target == ValidationGraphNodeKey::ValidationRule(rule.clone()))
            .then_some(match &edge.source {
                ValidationGraphNodeKey::FormField(field) => Some(field),
                _ => None,
            })
            .flatten()
        })
    }

    #[must_use]
    pub fn dependencies_of_rule(&self, rule: &ValidationRuleId) -> Vec<&FieldId> {
        self.edges
            .iter()
            .filter_map(|edge| {
                (edge.kind == ValidationGraphEdgeKind::RuleDependsOnField
                    && edge.source == ValidationGraphNodeKey::ValidationRule(rule.clone()))
                .then_some(match &edge.target {
                    ValidationGraphNodeKey::FormField(id) => Some(id),
                    _ => None,
                })
                .flatten()
            })
            .collect()
    }

    #[must_use]
    pub fn dependents_of_field(&self, field: &FieldId) -> Vec<&ValidationRuleId> {
        self.edges
            .iter()
            .filter_map(|edge| {
                (edge.kind == ValidationGraphEdgeKind::RuleDependsOnField
                    && edge.target == ValidationGraphNodeKey::FormField(field.clone()))
                .then_some(match &edge.source {
                    ValidationGraphNodeKey::ValidationRule(id) => Some(id),
                    _ => None,
                })
                .flatten()
            })
            .collect()
    }

    #[must_use]
    pub fn cycles_of_form(&self, form: &FormId) -> Vec<&ValidationDependencyCycle> {
        self.cycles
            .iter()
            .filter(|cycle| &cycle.form == form)
            .collect()
    }
}

fn validation_rule_ordinal(rule: &ValidationRuleId) -> usize {
    rule.as_str()
        .rsplit(':')
        .next()
        .and_then(|ordinal| ordinal.parse().ok())
        .unwrap_or(usize::MAX)
}

#[allow(clippy::too_many_arguments)]
#[must_use]
pub fn collect_validation_graph(
    build_roots: &BTreeMap<ComponentRootId, ComponentBuildRoot>,
    form_ownership: &FormOwnershipGraph,
    forms: &BTreeMap<FormId, FormEntity>,
    fields: &BTreeMap<FieldId, FormFieldEntity>,
    rules: &BTreeMap<ValidationRuleId, ValidationRule>,
    candidates: &[ValidationRuleCandidate],
    cycles: &[ValidationDependencyCycle],
    ownership: &BTreeMap<SemanticId, SemanticOwner>,
    references: &[SemanticReference],
) -> ValidationGraph {
    let mut nodes = BTreeMap::new();
    for form in forms.values() {
        nodes.insert(
            ValidationGraphNodeKey::Form(form.id.clone()),
            ValidationGraphNode::Form {
                id: form.id.clone(),
                provenance: form.provenance.clone(),
            },
        );
    }
    for field in fields.values() {
        nodes.insert(
            ValidationGraphNodeKey::FormField(field.id.clone()),
            ValidationGraphNode::FormField {
                id: field.id.clone(),
                provenance: field.provenance.clone(),
            },
        );
    }
    for rule in rules.values() {
        nodes.insert(
            ValidationGraphNodeKey::ValidationRule(rule.id.clone()),
            ValidationGraphNode::ValidationRule {
                id: rule.id.clone(),
                provenance: rule.provenance.clone(),
            },
        );
    }

    let mut edges = Vec::new();
    for edge in &form_ownership.ownership_edges {
        if let (FormOwnershipNodeKey::Form(form), FormOwnershipNodeKey::FormField(field)) =
            (&edge.owner, &edge.child)
        {
            edges.push(ValidationGraphEdge {
                kind: ValidationGraphEdgeKind::FormOwnsField,
                source: ValidationGraphNodeKey::Form(form.clone()),
                target: ValidationGraphNodeKey::FormField(field.clone()),
                provenance: edge.provenance.clone(),
            });
        }
    }
    for rule in rules.values() {
        if let Some(SemanticOwner::Entity(owner)) = ownership.get(rule.id.as_semantic_id()) {
            if let Some(field) = fields.keys().find(|field| field.as_semantic_id() == owner) {
                edges.push(ValidationGraphEdge {
                    kind: ValidationGraphEdgeKind::FieldOwnsRule,
                    source: ValidationGraphNodeKey::FormField(field.clone()),
                    target: ValidationGraphNodeKey::ValidationRule(rule.id.clone()),
                    provenance: rule.decorator_provenance.clone(),
                });
            }
        }
    }
    for reference in references
        .iter()
        .filter(|reference| reference.kind == SemanticReferenceKind::ValidationRuleField)
    {
        let Some(rule) = rules
            .keys()
            .find(|rule| rule.as_semantic_id() == &reference.source)
        else {
            continue;
        };
        let Some(field) = fields
            .keys()
            .find(|field| field.as_semantic_id() == &reference.target)
        else {
            continue;
        };
        edges.push(ValidationGraphEdge {
            kind: ValidationGraphEdgeKind::RuleDependsOnField,
            source: ValidationGraphNodeKey::ValidationRule(rule.clone()),
            target: ValidationGraphNodeKey::FormField(field.clone()),
            provenance: reference.provenance.clone(),
        });
    }
    edges.sort_by(|left, right| {
        (&left.source, left.kind, &left.target).cmp(&(&right.source, right.kind, &right.target))
    });
    let mut graph = ValidationGraph {
        id: ValidationGraphId::for_build_roots(build_roots.keys()),
        nodes,
        edges,
        cycles: cycles.to_vec(),
        validation: ValidationGraphValidation::default(),
    };
    graph.validation = validate_validation_graph(
        &graph,
        build_roots,
        form_ownership,
        forms,
        fields,
        rules,
        candidates,
    );
    graph
}

#[allow(clippy::too_many_lines)]
#[must_use]
pub fn validate_validation_graph(
    graph: &ValidationGraph,
    build_roots: &BTreeMap<ComponentRootId, ComponentBuildRoot>,
    form_ownership: &FormOwnershipGraph,
    forms: &BTreeMap<FormId, FormEntity>,
    fields: &BTreeMap<FieldId, FormFieldEntity>,
    rules: &BTreeMap<ValidationRuleId, ValidationRule>,
    candidates: &[ValidationRuleCandidate],
) -> ValidationGraphValidation {
    let mut diagnostics = Vec::new();
    if graph.id != ValidationGraphId::for_build_roots(build_roots.keys()) {
        push_integrity(
            &mut diagnostics,
            ValidationGraphIntegrityKind::GraphIdentityMismatch,
            "validation graph identity does not match canonical build roots",
        );
    }
    if graph.nodes.iter().any(|(key, node)| key != &node.key()) {
        push_integrity(
            &mut diagnostics,
            ValidationGraphIntegrityKind::DuplicateNode,
            "validation graph node key does not match its canonical node identity",
        );
    }
    for node in graph.nodes.values() {
        if provenance_is_missing(node.provenance()) {
            push_integrity(
                &mut diagnostics,
                ValidationGraphIntegrityKind::MissingProvenance,
                "validation graph node has no canonical provenance",
            );
        }
        if node.key().semantic_id().as_str().contains("form-instance:")
            || node
                .key()
                .semantic_id()
                .as_str()
                .contains("component-instance:")
        {
            push_integrity(
                &mut diagnostics,
                ValidationGraphIntegrityKind::InstanceIdentityInValidationGraph,
                "instance identity leaked into declaration validation graph",
            );
        }
    }
    if !graph.edges.windows(2).all(|pair| {
        (&pair[0].source, pair[0].kind, &pair[0].target)
            <= (&pair[1].source, pair[1].kind, &pair[1].target)
    }) {
        push_integrity(
            &mut diagnostics,
            ValidationGraphIntegrityKind::NonCanonicalOrdering,
            "validation graph edges are not canonically ordered",
        );
    }
    for edge in &graph.edges {
        if !graph.nodes.contains_key(&edge.source) || !graph.nodes.contains_key(&edge.target) {
            push_integrity(
                &mut diagnostics,
                ValidationGraphIntegrityKind::UnknownEdgeEndpoint,
                "validation graph edge has an unknown endpoint",
            );
        }
        if provenance_is_missing(&edge.provenance) {
            push_integrity(
                &mut diagnostics,
                ValidationGraphIntegrityKind::MissingProvenance,
                "validation graph edge has no canonical provenance",
            );
        }
        let shape_is_valid = matches!(
            (&edge.kind, &edge.source, &edge.target),
            (
                ValidationGraphEdgeKind::FormOwnsField,
                ValidationGraphNodeKey::Form(_),
                ValidationGraphNodeKey::FormField(_)
            ) | (
                ValidationGraphEdgeKind::FieldOwnsRule,
                ValidationGraphNodeKey::FormField(_),
                ValidationGraphNodeKey::ValidationRule(_)
            ) | (
                ValidationGraphEdgeKind::RuleDependsOnField,
                ValidationGraphNodeKey::ValidationRule(_),
                ValidationGraphNodeKey::FormField(_)
            )
        );
        if !shape_is_valid {
            push_integrity(
                &mut diagnostics,
                match edge.kind {
                    ValidationGraphEdgeKind::FormOwnsField => {
                        ValidationGraphIntegrityKind::FieldFormMismatch
                    }
                    ValidationGraphEdgeKind::FieldOwnsRule => {
                        ValidationGraphIntegrityKind::RuleTargetMismatch
                    }
                    ValidationGraphEdgeKind::RuleDependsOnField => {
                        ValidationGraphIntegrityKind::DependencyMismatch
                    }
                },
                "validation graph edge kind has invalid endpoint domains",
            );
        }
    }
    if validation_ownership_has_cycle(graph) {
        push_integrity(
            &mut diagnostics,
            ValidationGraphIntegrityKind::OwnershipCycle,
            "validation graph ownership edges contain a cycle",
        );
    }
    for form in forms.keys() {
        if !graph
            .nodes
            .contains_key(&ValidationGraphNodeKey::Form(form.clone()))
        {
            push_integrity(
                &mut diagnostics,
                ValidationGraphIntegrityKind::MissingFormNode,
                "canonical form is missing from validation graph",
            );
        }
    }
    for field in fields.values() {
        if !graph
            .nodes
            .contains_key(&ValidationGraphNodeKey::FormField(field.id.clone()))
        {
            push_integrity(
                &mut diagnostics,
                ValidationGraphIntegrityKind::MissingFieldNode,
                "canonical form field is missing from validation graph",
            );
        }
        if form_ownership.owner_of(&FormOwnershipNodeKey::FormField(field.id.clone()))
            != Some(&FormOwnershipNodeKey::Form(field.owner_form.clone()))
        {
            push_integrity(
                &mut diagnostics,
                ValidationGraphIntegrityKind::FieldFormMismatch,
                "validation graph field ownership disagrees with I5",
            );
        }
    }
    for rule in rules.values() {
        if !graph
            .nodes
            .contains_key(&ValidationGraphNodeKey::ValidationRule(rule.id.clone()))
        {
            push_integrity(
                &mut diagnostics,
                ValidationGraphIntegrityKind::MissingRuleNode,
                "canonical validation rule is missing from validation graph",
            );
        }
        let owners = graph
            .edges
            .iter()
            .filter(|edge| {
                edge.kind == ValidationGraphEdgeKind::FieldOwnsRule
                    && edge.target == ValidationGraphNodeKey::ValidationRule(rule.id.clone())
            })
            .collect::<Vec<_>>();
        if owners.len() != 1 {
            push_integrity(
                &mut diagnostics,
                ValidationGraphIntegrityKind::MultipleRuleOwners,
                "validation rule does not have exactly one field owner",
            );
        } else if owners[0].source != ValidationGraphNodeKey::FormField(rule.target_field.clone()) {
            push_integrity(
                &mut diagnostics,
                ValidationGraphIntegrityKind::RuleTargetMismatch,
                "validation rule owner does not match its target field",
            );
        }
        if let Some(dependency) = &rule.dependency {
            if dependency == &rule.target_field {
                push_integrity(
                    &mut diagnostics,
                    ValidationGraphIntegrityKind::SelfDependency,
                    "validation rule depends on its own target",
                );
            }
            let Some(target) = fields.get(&rule.target_field) else {
                continue;
            };
            let Some(dependency_field) = fields.get(dependency) else {
                push_integrity(
                    &mut diagnostics,
                    ValidationGraphIntegrityKind::DependencyMismatch,
                    "validation rule dependency is not a canonical field",
                );
                continue;
            };
            if target.owner_component != dependency_field.owner_component {
                push_integrity(
                    &mut diagnostics,
                    ValidationGraphIntegrityKind::CrossComponentDependency,
                    "validation dependency crosses component ownership",
                );
            }
            if target.owner_form != dependency_field.owner_form {
                push_integrity(
                    &mut diagnostics,
                    ValidationGraphIntegrityKind::CrossFormDependency,
                    "validation dependency crosses form ownership",
                );
            }
        }
        let dependency_edges = graph
            .edges
            .iter()
            .filter(|edge| {
                edge.kind == ValidationGraphEdgeKind::RuleDependsOnField
                    && edge.source == ValidationGraphNodeKey::ValidationRule(rule.id.clone())
            })
            .collect::<Vec<_>>();
        match &rule.dependency {
            None if !dependency_edges.is_empty() => push_integrity(
                &mut diagnostics,
                ValidationGraphIntegrityKind::DependencyMismatch,
                "unary validation rule has dependency edges",
            ),
            Some(dependency)
                if dependency_edges.len() != 1
                    || dependency_edges[0].target
                        != ValidationGraphNodeKey::FormField(dependency.clone()) =>
            {
                push_integrity(
                    &mut diagnostics,
                    ValidationGraphIntegrityKind::DependencyMismatch,
                    "validation rule dependency edge disagrees with canonical rule",
                );
            }
            _ => {}
        }
    }
    let invalid_candidate_ids = candidates
        .iter()
        .filter(|candidate| !candidate.is_valid())
        .map(|candidate| candidate.id.as_str())
        .collect::<BTreeSet<_>>();
    if graph
        .nodes
        .keys()
        .any(|key| invalid_candidate_ids.contains(key.semantic_id().as_str()))
    {
        push_integrity(
            &mut diagnostics,
            ValidationGraphIntegrityKind::InvalidCandidatePromoted,
            "invalid validation candidate was promoted into the valid graph",
        );
    }
    let cycle_candidates = graph
        .cycles
        .iter()
        .flat_map(|cycle| cycle.candidates.iter())
        .collect::<BTreeSet<_>>();
    if rules
        .values()
        .any(|rule| cycle_candidates.contains(&rule.candidate_id))
    {
        push_integrity(
            &mut diagnostics,
            ValidationGraphIntegrityKind::DependencyCycleLeakage,
            "cycle-participating rule leaked into executable graph membership",
        );
    }
    let executable_adjacency = rules.values().fold(
        BTreeMap::<FieldId, BTreeSet<FieldId>>::new(),
        |mut adjacency, rule| {
            if let Some(dependency) = &rule.dependency {
                adjacency
                    .entry(rule.target_field.clone())
                    .or_default()
                    .insert(dependency.clone());
                adjacency.entry(dependency.clone()).or_default();
            }
            adjacency
        },
    );
    if executable_adjacency.keys().any(|field| {
        reachable_fields(field, &executable_adjacency)
            .into_iter()
            .any(|other| {
                other != *field && reachable_fields(&other, &executable_adjacency).contains(field)
            })
    }) {
        push_integrity(
            &mut diagnostics,
            ValidationGraphIntegrityKind::DependencyCycleLeakage,
            "executable validation rules contain a dependency cycle",
        );
    }
    if !graph.cycles.windows(2).all(|pair| pair[0].id <= pair[1].id)
        || graph.cycles.iter().any(|cycle| {
            !cycle.fields.windows(2).all(|pair| pair[0] < pair[1])
                || !cycle.candidates.windows(2).all(|pair| pair[0] < pair[1])
        })
    {
        push_integrity(
            &mut diagnostics,
            ValidationGraphIntegrityKind::NonCanonicalOrdering,
            "validation dependency cycles are not canonically ordered",
        );
    }
    diagnostics.sort_by(|left, right| {
        (left.code.as_str(), left.message.as_str())
            .cmp(&(right.code.as_str(), right.message.as_str()))
    });
    diagnostics.dedup();
    ValidationGraphValidation {
        is_valid: diagnostics.is_empty(),
        diagnostics,
    }
}

fn provenance_is_missing(provenance: &SourceProvenance) -> bool {
    provenance.path.as_os_str().is_empty() || provenance.span.end <= provenance.span.start
}

fn validation_ownership_has_cycle(graph: &ValidationGraph) -> bool {
    let adjacency = graph
        .edges
        .iter()
        .filter(|edge| edge.kind != ValidationGraphEdgeKind::RuleDependsOnField)
        .fold(
            BTreeMap::<ValidationGraphNodeKey, BTreeSet<ValidationGraphNodeKey>>::new(),
            |mut adjacency, edge| {
                adjacency
                    .entry(edge.source.clone())
                    .or_default()
                    .insert(edge.target.clone());
                adjacency.entry(edge.target.clone()).or_default();
                adjacency
            },
        );
    adjacency.keys().any(|start| {
        adjacency
            .get(start)
            .into_iter()
            .flatten()
            .any(|next| validation_graph_reaches(next, start, &adjacency))
    })
}

fn validation_graph_reaches(
    start: &ValidationGraphNodeKey,
    target: &ValidationGraphNodeKey,
    adjacency: &BTreeMap<ValidationGraphNodeKey, BTreeSet<ValidationGraphNodeKey>>,
) -> bool {
    let mut visited = BTreeSet::new();
    let mut pending = vec![start.clone()];
    while let Some(node) = pending.pop() {
        if &node == target {
            return true;
        }
        if !visited.insert(node.clone()) {
            continue;
        }
        if let Some(next) = adjacency.get(&node) {
            pending.extend(next.iter().rev().cloned());
        }
    }
    false
}

fn push_integrity(
    diagnostics: &mut Vec<ValidationGraphIntegrityDiagnostic>,
    kind: ValidationGraphIntegrityKind,
    message: &str,
) {
    diagnostics.push(ValidationGraphIntegrityDiagnostic {
        code: kind.code().to_string(),
        kind,
        message: message.to_string(),
    });
}

#[cfg(test)]
mod tests {
    use super::{
        validate_validation_graph, ValidationGraphEdgeKind, ValidationGraphIntegrityKind,
        ValidationGraphNodeKey, ValidationRuleArgument, ValidationRuleKind,
        ValidationRuleViolation,
    };
    use crate::{
        build_application_semantic_model, build_application_semantic_model_for_unit,
        build_semantic_graph, semantic_graph_json, validate_application_semantic_model,
        CompilationUnit, ExecutionBoundary, SemanticEntityKind, SemanticOwner,
        SemanticReferenceKind, SEMANTIC_GRAPH_SCHEMA_VERSION,
    };

    fn build(source: &str) -> crate::ApplicationSemanticModel {
        build_application_semantic_model(&ezc_parser::parse_file("src/Profile.tsx", source))
    }

    #[test]
    fn lowers_unary_and_cross_field_rules_with_canonical_identity_and_ownership() {
        let source = r#"
@component("profile")
class Profile {
  @form()
  profile!: Form;

  @validate(required())
  @validate(minLength(2))
  @validate(pattern("^[a-z]+$"))
  @field(this.profile)
  email: string = "";

  @validate(equals(this.email))
  @field(this.profile)
  confirmation: string = "";

  render() { return <input field={this.email} />; }
}
"#;
        let asm = build(source);
        assert_eq!(asm.validation_rule_candidates.len(), 4);
        assert!(
            asm.validation_rule_candidates
                .iter()
                .all(|candidate| candidate.is_valid() && candidate.rule_id.is_some()),
            "{:#?}",
            asm.validation_rule_candidates
        );
        let rules = asm.validation_rules();
        assert_eq!(rules.len(), 4);
        assert_eq!(rules[0].kind, ValidationRuleKind::Required);
        assert_eq!(rules[0].rule_authored_order, 0);
        assert_eq!(rules[1].kind, ValidationRuleKind::MinLength);
        assert_eq!(rules[1].argument, ValidationRuleArgument::Length(2));
        assert_eq!(rules[2].kind, ValidationRuleKind::Pattern);
        assert_eq!(rules[3].kind, ValidationRuleKind::Equals);
        assert_eq!(rules[3].dependency.as_ref(), Some(&rules[0].target_field));
        assert!(rules
            .iter()
            .all(|rule| rule.boundary == ExecutionBoundary::Client));
        assert!(rules.iter().all(|rule| {
            asm.owner(rule.id.as_semantic_id())
                == Some(&SemanticOwner::entity(
                    rule.target_field.as_semantic_id().clone(),
                ))
                && asm
                    .entity(rule.id.as_semantic_id())
                    .is_some_and(|entity| entity.kind() == SemanticEntityKind::ValidationRule)
        }));
        assert_eq!(
            asm.references_of_kind(SemanticReferenceKind::ValidationRuleField)
                .len(),
            1
        );
        assert!(asm.validation_graph.validation.is_valid);
        assert_eq!(
            asm.validation_graph
                .edges
                .iter()
                .filter(|edge| edge.kind == ValidationGraphEdgeKind::FieldOwnsRule)
                .count(),
            4
        );
        assert!(validate_application_semantic_model(&asm).is_empty());
    }

    #[test]
    fn invalidates_duplicate_contradictory_and_cycle_groups_without_winners() {
        let source = r#"
@component("profile")
class Profile {
  @form() profile!: Form;

  @validate(required())
  @validate(required())
  @field(this.profile)
  duplicate = "";

  @validate(min(10))
  @validate(max(5))
  @field(this.profile)
  age = 20;

  @validate(equals(this.right))
  @field(this.profile)
  left = "";

  @validate(equals(this.left))
  @field(this.profile)
  right = "";

  render() { return <div />; }
}
"#;
        let asm = build(source);
        assert_eq!(asm.validation_rule_candidates.len(), 6);
        let duplicate = asm
            .validation_rule_candidates
            .iter()
            .filter(|candidate| candidate.authored_target_name.as_deref() == Some("duplicate"))
            .collect::<Vec<_>>();
        assert_eq!(duplicate.len(), 2);
        assert!(duplicate.iter().all(|candidate| {
            candidate.rule_id.is_none()
                && candidate
                    .violations
                    .contains(&ValidationRuleViolation::DuplicateRule)
        }));
        assert!(asm
            .validation_rule_candidates
            .iter()
            .filter(|candidate| candidate.authored_target_name.as_deref() == Some("age"))
            .all(|candidate| candidate
                .violations
                .contains(&ValidationRuleViolation::ContradictoryRule)));
        assert_eq!(asm.validation_graph.cycles.len(), 1);
        assert_eq!(asm.validation_graph.cycles[0].fields.len(), 2);
        assert!(asm
            .validation_rule_candidates
            .iter()
            .filter(|candidate| matches!(
                candidate.authored_target_name.as_deref(),
                Some("left" | "right")
            ))
            .all(|candidate| candidate
                .violations
                .contains(&ValidationRuleViolation::DependencyCycle)));
        assert!(asm.validation_rules.is_empty());
        assert!(asm.validation_graph.validation.is_valid);
    }

    #[test]
    fn retains_invalid_targets_rules_arguments_dependencies_and_type_evidence() {
        let source = r#"
@validate(required())
@component("profile")
class Profile {
  @form() profile!: Form;

  @validate
  @field(this.profile)
  uninvoked = "";

  @validate(schema.required())
  @field(this.profile)
  memberCall = "";

  @validate(min("one"))
  @field(this.profile)
  wrongArgument = 1;

  @validate(email())
  @field(this.profile)
  wrongType = 1;

  @validate(equals(this.missing))
  @field(this.profile)
  unresolved = "";

  @validate(required())
  method() {}

  render() { return <div />; }
}
"#;
        let asm = build(source);
        assert_eq!(asm.validation_rule_candidates.len(), 7);
        assert!(asm
            .validation_rule_candidates
            .iter()
            .all(|candidate| { candidate.rule_id.is_none() && !candidate.violations.is_empty() }));
        assert!(asm
            .validation_rule_candidates
            .iter()
            .any(|candidate| candidate
                .violations
                .contains(&ValidationRuleViolation::InvalidDecoratorInvocation)));
        assert!(asm
            .validation_rule_candidates
            .iter()
            .any(|candidate| candidate
                .violations
                .contains(&ValidationRuleViolation::InvalidRuleExpression)));
        assert!(asm
            .validation_rule_candidates
            .iter()
            .any(|candidate| candidate
                .violations
                .contains(&ValidationRuleViolation::UnsupportedArgument)));
        assert!(asm
            .validation_rule_candidates
            .iter()
            .any(|candidate| candidate
                .violations
                .contains(&ValidationRuleViolation::IncompatibleType)));
        assert!(asm
            .validation_rule_candidates
            .iter()
            .any(|candidate| candidate
                .violations
                .contains(&ValidationRuleViolation::UnresolvedDependency)));
        assert!(asm
            .validation_rule_candidates
            .iter()
            .any(|candidate| candidate.violations.contains(
                &ValidationRuleViolation::InvalidTarget {
                    actual: crate::AuthoredDeclarationKind::Method,
                }
            )));
    }

    #[test]
    fn graph_validation_is_deterministic_and_detects_stale_integrity() {
        let source = r#"
@component("profile")
class Profile {
  @form() profile!: Form;
  @validate(required())
  @field(this.profile)
  name = "";
  render() { return <div />; }
}
"#;
        let asm = build(source);
        let mut malformed = asm.validation_graph.clone();
        let rule = asm.validation_rules.values().next().unwrap();
        malformed
            .nodes
            .remove(&ValidationGraphNodeKey::ValidationRule(rule.id.clone()));
        let validation = validate_validation_graph(
            &malformed,
            &asm.component_instance_plan.roots,
            &asm.form_ownership,
            &asm.forms,
            &asm.form_fields,
            &asm.validation_rules,
            &asm.validation_rule_candidates,
        );
        assert!(!validation.is_valid);
        assert!(validation.diagnostics.iter().any(|diagnostic| {
            diagnostic.kind == ValidationGraphIntegrityKind::MissingRuleNode
        }));
        assert_eq!(
            validation,
            validate_validation_graph(
                &malformed,
                &asm.component_instance_plan.roots,
                &asm.form_ownership,
                &asm.forms,
                &asm.form_fields,
                &asm.validation_rules,
                &asm.validation_rule_candidates,
            )
        );

        let mut cyclic = asm.validation_graph.clone();
        let ownership = cyclic
            .edges
            .iter()
            .find(|edge| edge.kind == ValidationGraphEdgeKind::FieldOwnsRule)
            .unwrap()
            .clone();
        cyclic.edges.push(super::ValidationGraphEdge {
            kind: ValidationGraphEdgeKind::FieldOwnsRule,
            source: ownership.target,
            target: ownership.source,
            provenance: ownership.provenance,
        });
        cyclic.edges.sort_by(|left, right| {
            (&left.source, left.kind, &left.target).cmp(&(&right.source, right.kind, &right.target))
        });
        let cyclic_validation = validate_validation_graph(
            &cyclic,
            &asm.component_instance_plan.roots,
            &asm.form_ownership,
            &asm.forms,
            &asm.form_fields,
            &asm.validation_rules,
            &asm.validation_rule_candidates,
        );
        assert!(cyclic_validation
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.kind == ValidationGraphIntegrityKind::OwnershipCycle));
    }

    #[test]
    fn rejects_authored_functions_and_imports_shadowing_compiler_rule_names() {
        let local = build(
            r#"
function required() { return true; }
@component("profile")
class Profile {
  @form() profile!: Form;
  @validate(required())
  @field(this.profile)
  name = "";
  render() { return <div />; }
}
"#,
        );
        assert!(local.validation_rule_candidates[0]
            .violations
            .contains(&ValidationRuleViolation::ShadowedCompilerRule));
        assert!(local.validation_rules.is_empty());

        let imported = build(
            r#"
import { authored as email } from "./rules";
@component("profile")
class Profile {
  @form() profile!: Form;
  @validate(email())
  @field(this.profile)
  address = "";
  render() { return <div />; }
}
"#,
        );
        assert!(imported.validation_rule_candidates[0]
            .violations
            .contains(&ValidationRuleViolation::ShadowedCompilerRule));
        assert!(imported.validation_rules.is_empty());
    }

    #[test]
    fn applies_canonical_type_domains_and_exact_same_form_dependency_scope() {
        let source = r#"
@component("profile")
class Profile {
  @form() primary!: Form;
  @form() secondary!: Form;

  @validate(min(0))
  @field(this.primary)
  amount: number | null = null;

  @validate(minLength(1))
  @field(this.primary)
  tags: string[] = [];

  @validate(maxLength(2))
  @field(this.primary)
  pair: [string, string] = ["", ""];

  @validate(email())
  @field(this.primary)
  address: string | null = null;

  @validate(equals(this.foreign))
  @field(this.primary)
  local = "";

  @field(this.secondary)
  foreign = "";

  @validate(equals(this.selfReference))
  @field(this.primary)
  selfReference = "";

  @validate(min(1))
  @field(this.primary)
  wrongDomain = "";

  render() { return <div />; }
}
"#;
        let asm = build(source);
        assert_eq!(asm.validation_rules.len(), 4);
        assert!(asm
            .validation_rule_candidates
            .iter()
            .any(|candidate| candidate
                .violations
                .contains(&ValidationRuleViolation::CrossFormDependency)));
        assert!(asm
            .validation_rule_candidates
            .iter()
            .any(|candidate| candidate
                .violations
                .contains(&ValidationRuleViolation::SelfDependency)));
        assert!(asm
            .validation_rule_candidates
            .iter()
            .any(|candidate| candidate
                .violations
                .contains(&ValidationRuleViolation::IncompatibleType)));
    }

    #[test]
    fn reversed_files_preserve_validation_products_and_public_schema() {
        let first = ezc_parser::parse_file(
            "src/A.tsx",
            r#"@component("a-x") class A { @form() form!: Form; @validate(required()) @field(this.form) value = ""; render() { return <div />; } }"#,
        );
        let second = ezc_parser::parse_file(
            "src/B.tsx",
            r#"@component("b-x") class B { @form() form!: Form; @validate(min(1 + 1)) @field(this.form) value = 2; render() { return <div />; } }"#,
        );
        let forward =
            build_application_semantic_model_for_unit(&CompilationUnit::from_parsed_files(vec![
                first.clone(),
                second.clone(),
            ]));
        let reversed =
            build_application_semantic_model_for_unit(&CompilationUnit::from_parsed_files(vec![
                second, first,
            ]));
        assert_eq!(
            forward.validation_rule_candidates,
            reversed.validation_rule_candidates
        );
        assert_eq!(forward.validation_rules, reversed.validation_rules);
        assert_eq!(forward.validation_graph, reversed.validation_graph);
        assert_eq!(SEMANTIC_GRAPH_SCHEMA_VERSION, 6);
        let json = semantic_graph_json(&build_semantic_graph(&forward));
        assert!(json.contains("validation-rule"));
    }
}
