use std::collections::{BTreeMap, BTreeSet};

use crate::{
    is_assignable, state_initializer_value_type, ComponentNode, ElementNode, FieldBindingId,
    FieldId, FormEntity, FormFieldBindingCandidateId, FormFieldDeclarationCandidate,
    FormFieldEntity, FormId, RenderAttribute, RenderAttributeValue, SemanticId, SemanticType,
    SerializableValue, SourceProvenance, TemplateChild, TemplateNode,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum FormInputKind {
    Text,
    Email,
    Password,
    Search,
    Tel,
    Url,
    Number,
    Checkbox,
    Radio,
    Date,
    Time,
    DateTimeLocal,
    Month,
    Week,
    Range,
    Hidden,
}

impl FormInputKind {
    fn from_static(value: &str) -> Option<Self> {
        Some(match value {
            "text" => Self::Text,
            "email" => Self::Email,
            "password" => Self::Password,
            "search" => Self::Search,
            "tel" => Self::Tel,
            "url" => Self::Url,
            "number" => Self::Number,
            "checkbox" => Self::Checkbox,
            "radio" => Self::Radio,
            "date" => Self::Date,
            "time" => Self::Time,
            "datetime-local" => Self::DateTimeLocal,
            "month" => Self::Month,
            "week" => Self::Week,
            "range" => Self::Range,
            "hidden" => Self::Hidden,
            _ => return None,
        })
    }

    const fn channel(self) -> FormControlChannel {
        match self {
            Self::Number | Self::Range => FormControlChannel::NumericValue,
            Self::Checkbox => FormControlChannel::Checked,
            Self::Radio => FormControlChannel::RadioValue,
            Self::Text
            | Self::Email
            | Self::Password
            | Self::Search
            | Self::Tel
            | Self::Url
            | Self::Date
            | Self::Time
            | Self::DateTimeLocal
            | Self::Month
            | Self::Week
            | Self::Hidden => FormControlChannel::Value,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum FormControlChannel {
    Value,
    NumericValue,
    Checked,
    RadioValue,
    SelectedValue,
    SelectedValues,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FormControlNormalization {
    Text,
    NullableText,
    Number,
    NullableNumber,
    Boolean,
    Scalar,
    ScalarArray,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FormControlCompatibility {
    Compatible(FormControlNormalization),
    Incompatible,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FormFieldBindingExpressionFact {
    DirectThisMember { name: String },
    Bare,
    Static(String),
    Unsupported { expression: Option<String> },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum FormFieldBindingEvidenceKind {
    InputType,
    RadioValue,
    Multiple,
    Value,
    Checked,
    DefaultValue,
    Selected,
    OnInput,
    OnChange,
    Spread,
    TextareaChildren,
    FormAttribute,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FormFieldBindingEvidence {
    pub kind: FormFieldBindingEvidenceKind,
    pub provenance: SourceProvenance,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum FormFieldBindingViolation {
    InvalidControlElement,
    InvalidInputType,
    DynamicInputType,
    InvalidBindingExpression,
    UnresolvedField,
    AmbiguousField,
    InvalidFieldDeclaration,
    CrossComponentField,
    DuplicateBindingAttribute,
    SpreadConflict,
    DuplicateFieldControl,
    InvalidRadioGroup,
    MissingRadioValue,
    DuplicateRadioValue,
    IncompatibleControlType,
    CompetingValueBinding,
    CompetingCheckedBinding,
    CompetingDefaultValue,
    CompetingSelectedState,
    CompetingChangeHandler,
    UnsupportedTextareaChildren,
    UnsupportedDynamicMultiple,
    UnsupportedFormAttribute,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FormFieldBindingCandidate {
    pub id: FormFieldBindingCandidateId,
    pub binding_id: Option<FieldBindingId>,
    pub owner_component: Option<SemanticId>,
    pub owner_template: Option<SemanticId>,
    pub control_entity: Option<SemanticId>,
    pub element_name: Option<String>,
    pub authored_input_type: Option<String>,
    pub input_kind: Option<FormInputKind>,
    pub field_expression: FormFieldBindingExpressionFact,
    pub authored_field_name: Option<String>,
    pub resolved_field: Option<FieldId>,
    pub resolved_form: Option<FormId>,
    pub channel: Option<FormControlChannel>,
    pub compatibility: Option<FormControlCompatibility>,
    pub field_type: Option<SemanticType>,
    pub radio_value: Option<SerializableValue>,
    pub authored_order: usize,
    pub provenance: SourceProvenance,
    pub attribute_provenance: SourceProvenance,
    pub expression_provenance: Option<SourceProvenance>,
    pub field_name_provenance: Option<SourceProvenance>,
    pub control_kind_provenance: Option<SourceProvenance>,
    pub radio_value_provenance: Option<SourceProvenance>,
    pub evidence: Vec<FormFieldBindingEvidence>,
    pub violations: Vec<FormFieldBindingViolation>,
}

impl FormFieldBindingCandidate {
    #[must_use]
    pub fn is_valid(&self) -> bool {
        self.violations.is_empty()
    }

    fn add_violation(&mut self, violation: FormFieldBindingViolation) {
        if !self.violations.contains(&violation) {
            self.violations.push(violation);
            self.violations.sort();
        }
    }
}

/// Immutable compiler-owned use-site binding between one template control and
/// one canonical Form Field declaration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FormFieldBinding {
    pub id: FieldBindingId,
    pub owner_template: SemanticId,
    pub control_entity: SemanticId,
    pub field: FieldId,
    pub form: FormId,
    pub component: SemanticId,
    pub element_name: String,
    pub input_kind: Option<FormInputKind>,
    pub channel: FormControlChannel,
    pub compatibility: FormControlCompatibility,
    pub field_type: SemanticType,
    pub radio_value: Option<SerializableValue>,
    pub authored_order: usize,
    pub provenance: SourceProvenance,
    pub attribute_provenance: SourceProvenance,
    pub expression_provenance: SourceProvenance,
    pub control_kind_provenance: SourceProvenance,
    pub radio_value_provenance: Option<SourceProvenance>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct FormFieldBindingProducts {
    pub candidates: Vec<FormFieldBindingCandidate>,
    pub bindings: BTreeMap<FieldBindingId, FormFieldBinding>,
}

#[must_use]
pub fn collect_form_field_binding_products(
    components: &[ComponentNode],
    templates: &[TemplateNode],
    forms: &BTreeMap<FormId, FormEntity>,
    fields: &BTreeMap<FieldId, FormFieldEntity>,
    field_candidates: &[FormFieldDeclarationCandidate],
) -> FormFieldBindingProducts {
    let components = components
        .iter()
        .map(|component| (component.id.clone(), component))
        .collect::<BTreeMap<_, _>>();
    let mut templates = templates.iter().collect::<Vec<_>>();
    templates.sort_by(|left, right| left.id.cmp(&right.id));

    let mut candidates = Vec::new();
    for template in templates {
        let component = template
            .owner
            .entity_id()
            .and_then(|component| components.get(component).copied());
        if let Some(root) = &template.root {
            collect_element_candidates(
                root,
                template,
                "root",
                component,
                forms,
                fields,
                field_candidates,
                &mut candidates,
            );
        }
        if let Some(root) = &template.root_fragment {
            collect_child_candidates(
                &root.children,
                template,
                "root",
                component,
                forms,
                fields,
                field_candidates,
                &mut candidates,
            );
        }
    }

    candidates.sort_by(candidate_source_order);
    let mut orders = BTreeMap::<SemanticId, usize>::new();
    for candidate in &mut candidates {
        if let Some(template) = &candidate.owner_template {
            let order = orders.entry(template.clone()).or_default();
            candidate.authored_order = *order;
            *order += 1;
        }
    }
    mark_binding_multiplicity(&mut candidates);

    let mut bindings = BTreeMap::new();
    for candidate in &mut candidates {
        if !candidate.is_valid() {
            candidate.binding_id = None;
            continue;
        }
        let binding = lower_valid_binding(candidate);
        candidate.binding_id = Some(binding.id.clone());
        bindings.insert(binding.id.clone(), binding);
    }

    FormFieldBindingProducts {
        candidates,
        bindings,
    }
}

#[allow(clippy::too_many_arguments)]
fn collect_element_candidates(
    element: &ElementNode,
    template: &TemplateNode,
    path: &str,
    component: Option<&ComponentNode>,
    forms: &BTreeMap<FormId, FormEntity>,
    fields: &BTreeMap<FieldId, FormFieldEntity>,
    field_candidates: &[FormFieldDeclarationCandidate],
    candidates: &mut Vec<FormFieldBindingCandidate>,
) {
    let control = template.id.template_entity("element", path);
    let field_attributes = element
        .authored_attributes
        .iter()
        .filter(|attribute| attribute.name == "field")
        .collect::<Vec<_>>();
    for attribute in &field_attributes {
        let mut candidate = binding_candidate(element, template, &control, component, attribute);
        if field_attributes.len() > 1 {
            candidate.add_violation(FormFieldBindingViolation::DuplicateBindingAttribute);
        }
        classify_control(element, template, &mut candidate);
        resolve_field(component, fields, field_candidates, forms, &mut candidate);
        classify_compatibility(&mut candidate);
        candidates.push(candidate);
    }

    collect_child_candidates(
        &element.children,
        template,
        path,
        component,
        forms,
        fields,
        field_candidates,
        candidates,
    );
}

#[allow(clippy::too_many_arguments)]
fn collect_child_candidates(
    children: &[TemplateChild],
    template: &TemplateNode,
    parent_path: &str,
    component: Option<&ComponentNode>,
    forms: &BTreeMap<FormId, FormEntity>,
    fields: &BTreeMap<FieldId, FormFieldEntity>,
    field_candidates: &[FormFieldDeclarationCandidate],
    candidates: &mut Vec<FormFieldBindingCandidate>,
) {
    for (index, child) in children.iter().enumerate() {
        let path = format!("{parent_path}.{index}");
        match child {
            TemplateChild::Element(element) => collect_element_candidates(
                element,
                template,
                &path,
                component,
                forms,
                fields,
                field_candidates,
                candidates,
            ),
            TemplateChild::Fragment(fragment) => collect_child_candidates(
                &fragment.children,
                template,
                &path,
                component,
                forms,
                fields,
                field_candidates,
                candidates,
            ),
            TemplateChild::Conditional(conditional) => {
                collect_child_candidates(
                    &conditional.when_true,
                    template,
                    &format!("{path}.true"),
                    component,
                    forms,
                    fields,
                    field_candidates,
                    candidates,
                );
                collect_child_candidates(
                    &conditional.when_false,
                    template,
                    &format!("{path}.false"),
                    component,
                    forms,
                    fields,
                    field_candidates,
                    candidates,
                );
            }
            TemplateChild::List(list) => collect_child_candidates(
                &list.item_template,
                template,
                &format!("{path}.item"),
                component,
                forms,
                fields,
                field_candidates,
                candidates,
            ),
            TemplateChild::Text { .. } | TemplateChild::Binding { .. } => {}
        }
    }
}

fn binding_candidate(
    element: &ElementNode,
    template: &TemplateNode,
    control: &SemanticId,
    component: Option<&ComponentNode>,
    attribute: &RenderAttribute,
) -> FormFieldBindingCandidate {
    let path = &template.provenance.path;
    let field_expression = match &attribute.value {
        RenderAttributeValue::Boolean => FormFieldBindingExpressionFact::Bare,
        RenderAttributeValue::Static(value) => {
            FormFieldBindingExpressionFact::Static(value.clone())
        }
        RenderAttributeValue::Expression(expression) => attribute.this_member.as_ref().map_or_else(
            || FormFieldBindingExpressionFact::Unsupported {
                expression: expression.clone(),
            },
            |member| FormFieldBindingExpressionFact::DirectThisMember {
                name: member.member.clone(),
            },
        ),
        RenderAttributeValue::Spread(expression) => FormFieldBindingExpressionFact::Unsupported {
            expression: expression.clone(),
        },
        RenderAttributeValue::Unsupported => {
            FormFieldBindingExpressionFact::Unsupported { expression: None }
        }
    };
    let authored_field_name = match &field_expression {
        FormFieldBindingExpressionFact::DirectThisMember { name } => Some(name.clone()),
        _ => None,
    };
    let mut candidate = FormFieldBindingCandidate {
        id: FormFieldBindingCandidateId::for_source_position(path, attribute.span.start),
        binding_id: None,
        owner_component: component.map(|component| component.id.clone()),
        owner_template: Some(template.id.clone()),
        control_entity: Some(control.clone()),
        element_name: Some(element.tag_name.clone()),
        authored_input_type: None,
        input_kind: None,
        field_expression,
        authored_field_name,
        resolved_field: None,
        resolved_form: None,
        channel: None,
        compatibility: None,
        field_type: None,
        radio_value: None,
        authored_order: 0,
        provenance: SourceProvenance::new(path, element.span),
        attribute_provenance: SourceProvenance::new(path, attribute.span),
        expression_provenance: attribute
            .expression_span
            .or(attribute.value_span)
            .map(|span| SourceProvenance::new(path, span)),
        field_name_provenance: attribute
            .this_member
            .as_ref()
            .map(|member| SourceProvenance::new(path, member.member_span)),
        control_kind_provenance: Some(SourceProvenance::new(path, element.tag_name_span)),
        radio_value_provenance: None,
        evidence: Vec::new(),
        violations: Vec::new(),
    };
    if !matches!(
        candidate.field_expression,
        FormFieldBindingExpressionFact::DirectThisMember { .. }
    ) {
        candidate.add_violation(FormFieldBindingViolation::InvalidBindingExpression);
    }
    candidate
}

fn classify_control(
    element: &ElementNode,
    template: &TemplateNode,
    candidate: &mut FormFieldBindingCandidate,
) {
    let path = &template.provenance.path;
    for attribute in &element.authored_attributes {
        let evidence_kind = match attribute.name.as_str() {
            "type" => Some(FormFieldBindingEvidenceKind::InputType),
            "value" => Some(FormFieldBindingEvidenceKind::Value),
            "checked" => Some(FormFieldBindingEvidenceKind::Checked),
            "defaultValue" => Some(FormFieldBindingEvidenceKind::DefaultValue),
            "multiple" => Some(FormFieldBindingEvidenceKind::Multiple),
            "onInput" => Some(FormFieldBindingEvidenceKind::OnInput),
            "onChange" => Some(FormFieldBindingEvidenceKind::OnChange),
            "form" => Some(FormFieldBindingEvidenceKind::FormAttribute),
            "{...}" => Some(FormFieldBindingEvidenceKind::Spread),
            _ => None,
        };
        if let Some(kind) = evidence_kind {
            candidate.evidence.push(FormFieldBindingEvidence {
                kind,
                provenance: SourceProvenance::new(path, attribute.span),
            });
        }
    }

    if element
        .authored_attributes
        .iter()
        .any(|attribute| matches!(attribute.value, RenderAttributeValue::Spread(_)))
    {
        candidate.add_violation(FormFieldBindingViolation::SpreadConflict);
    }
    if element
        .authored_attributes
        .iter()
        .any(|attribute| attribute.name == "form")
    {
        candidate.add_violation(FormFieldBindingViolation::UnsupportedFormAttribute);
    }
    if element
        .authored_attributes
        .iter()
        .any(|attribute| matches!(attribute.name.as_str(), "onInput" | "onChange"))
    {
        candidate.add_violation(FormFieldBindingViolation::CompetingChangeHandler);
    }

    match element.tag_name.as_str() {
        "input" => classify_input(element, path, candidate),
        "textarea" => {
            candidate.channel = Some(FormControlChannel::Value);
            if !element.children.is_empty() {
                candidate.evidence.push(FormFieldBindingEvidence {
                    kind: FormFieldBindingEvidenceKind::TextareaChildren,
                    provenance: SourceProvenance::new(path, element.span),
                });
                candidate.add_violation(FormFieldBindingViolation::UnsupportedTextareaChildren);
            }
            if has_attribute(element, "value") {
                candidate.add_violation(FormFieldBindingViolation::CompetingValueBinding);
            }
        }
        "select" => classify_select(element, path, candidate),
        _ => candidate.add_violation(FormFieldBindingViolation::InvalidControlElement),
    }
    if has_attribute(element, "defaultValue") {
        candidate.add_violation(FormFieldBindingViolation::CompetingDefaultValue);
    }
}

fn classify_input(
    element: &ElementNode,
    path: &std::path::Path,
    candidate: &mut FormFieldBindingCandidate,
) {
    let types = attributes_named(element, "type");
    let input_kind = match types.as_slice() {
        [] => Some(FormInputKind::Text),
        [attribute] => {
            if let RenderAttributeValue::Static(value) = &attribute.value {
                candidate.authored_input_type = Some(value.clone());
                FormInputKind::from_static(value).or_else(|| {
                    candidate.add_violation(FormFieldBindingViolation::InvalidInputType);
                    None
                })
            } else {
                candidate.add_violation(FormFieldBindingViolation::DynamicInputType);
                None
            }
        }
        _ => {
            candidate.add_violation(FormFieldBindingViolation::InvalidInputType);
            None
        }
    };
    candidate.input_kind = input_kind;
    candidate.channel = input_kind.map(FormInputKind::channel);
    if let Some(type_attribute) = types.first() {
        candidate.control_kind_provenance = Some(SourceProvenance::new(path, type_attribute.span));
    }

    match input_kind {
        Some(FormInputKind::Radio) => classify_radio_value(element, path, candidate),
        Some(FormInputKind::Checkbox) => {
            if has_attribute(element, "checked") {
                candidate.add_violation(FormFieldBindingViolation::CompetingCheckedBinding);
            }
        }
        Some(_) if has_attribute(element, "value") => {
            candidate.add_violation(FormFieldBindingViolation::CompetingValueBinding);
        }
        Some(_) | None => {}
    }
}

fn classify_radio_value(
    element: &ElementNode,
    path: &std::path::Path,
    candidate: &mut FormFieldBindingCandidate,
) {
    let values = attributes_named(element, "value");
    let Some(attribute) = values.first() else {
        candidate.add_violation(FormFieldBindingViolation::MissingRadioValue);
        candidate.add_violation(FormFieldBindingViolation::InvalidRadioGroup);
        return;
    };
    candidate.evidence.push(FormFieldBindingEvidence {
        kind: FormFieldBindingEvidenceKind::RadioValue,
        provenance: SourceProvenance::new(path, attribute.span),
    });
    if values.len() != 1 {
        candidate.add_violation(FormFieldBindingViolation::InvalidRadioGroup);
        return;
    }
    let value = attribute.constant_value.as_ref().filter(|value| {
        matches!(
            value,
            SerializableValue::String(_)
                | SerializableValue::Number(_)
                | SerializableValue::Boolean(_)
        )
    });
    if let Some(value) = value {
        candidate.radio_value = Some(value.clone());
        candidate.radio_value_provenance = Some(SourceProvenance::new(path, attribute.span));
    } else {
        candidate.add_violation(FormFieldBindingViolation::MissingRadioValue);
        candidate.add_violation(FormFieldBindingViolation::InvalidRadioGroup);
    }
}

fn classify_select(
    element: &ElementNode,
    path: &std::path::Path,
    candidate: &mut FormFieldBindingCandidate,
) {
    let multiple = attributes_named(element, "multiple");
    candidate.channel = match multiple.as_slice() {
        [] => Some(FormControlChannel::SelectedValue),
        [attribute] if matches!(attribute.value, RenderAttributeValue::Boolean) => {
            Some(FormControlChannel::SelectedValues)
        }
        _ => {
            candidate.add_violation(FormFieldBindingViolation::UnsupportedDynamicMultiple);
            None
        }
    };
    if has_attribute(element, "value") {
        candidate.add_violation(FormFieldBindingViolation::CompetingValueBinding);
    }
    if selected_option_provenance(&element.children, path).is_some() {
        if let Some(provenance) = selected_option_provenance(&element.children, path) {
            candidate.evidence.push(FormFieldBindingEvidence {
                kind: FormFieldBindingEvidenceKind::Selected,
                provenance,
            });
        }
        candidate.add_violation(FormFieldBindingViolation::CompetingSelectedState);
    }
}

fn resolve_field(
    component: Option<&ComponentNode>,
    fields: &BTreeMap<FieldId, FormFieldEntity>,
    field_candidates: &[FormFieldDeclarationCandidate],
    forms: &BTreeMap<FormId, FormEntity>,
    candidate: &mut FormFieldBindingCandidate,
) {
    let (Some(component), Some(name)) = (component, candidate.authored_field_name.as_deref())
    else {
        if component.is_none() {
            candidate.add_violation(FormFieldBindingViolation::CrossComponentField);
        }
        return;
    };
    let local = fields
        .values()
        .filter(|field| field.owner_component == component.id && field.name == name)
        .collect::<Vec<_>>();
    let local_invalid = field_candidates
        .iter()
        .filter(|field| {
            field.owner_component.as_ref() == Some(&component.id)
                && field.authored_name.as_deref() == Some(name)
                && !field.is_valid()
        })
        .count();
    let ordinary_collision = component
        .state_fields
        .iter()
        .any(|field| field.name == name)
        || component.methods.iter().any(|method| method.name == name);

    match local.as_slice() {
        [field] => {
            candidate.resolved_field = Some(field.id.clone());
            candidate.resolved_form = Some(field.owner_form.clone());
            candidate.field_type = Some(field.semantic_type.clone());
            let valid_form_owner = forms
                .get(&field.owner_form)
                .and_then(|form| form.owner.entity_id())
                == Some(&component.id);
            if field.owner_component != component.id || !valid_form_owner {
                candidate.add_violation(FormFieldBindingViolation::CrossComponentField);
            }
            if ordinary_collision || local_invalid > 0 {
                candidate.add_violation(FormFieldBindingViolation::AmbiguousField);
            }
        }
        [] => {
            if local_invalid > 0 {
                candidate.add_violation(FormFieldBindingViolation::InvalidFieldDeclaration);
                if local_invalid > 1 {
                    candidate.add_violation(FormFieldBindingViolation::AmbiguousField);
                }
            } else if fields
                .values()
                .any(|field| field.name == name && field.owner_component != component.id)
            {
                candidate.add_violation(FormFieldBindingViolation::CrossComponentField);
            } else {
                candidate.add_violation(FormFieldBindingViolation::UnresolvedField);
            }
        }
        _ => candidate.add_violation(FormFieldBindingViolation::AmbiguousField),
    }
}

fn classify_compatibility(candidate: &mut FormFieldBindingCandidate) {
    let (Some(channel), Some(field_type)) = (candidate.channel, candidate.field_type.as_ref())
    else {
        return;
    };
    let compatibility = match channel {
        FormControlChannel::Value => string_compatibility(field_type),
        FormControlChannel::NumericValue => numeric_compatibility(field_type),
        FormControlChannel::Checked => boolean_compatibility(field_type),
        FormControlChannel::RadioValue => scalar_compatibility(field_type, false),
        FormControlChannel::SelectedValue => scalar_compatibility(field_type, true),
        FormControlChannel::SelectedValues => array_scalar_compatibility(field_type),
    };
    candidate.compatibility = Some(compatibility);
    if compatibility == FormControlCompatibility::Incompatible {
        candidate.add_violation(FormFieldBindingViolation::IncompatibleControlType);
        return;
    }
    if channel == FormControlChannel::RadioValue {
        if let Some(value) = &candidate.radio_value {
            if !is_assignable(&state_initializer_value_type(value), field_type) {
                candidate.add_violation(FormFieldBindingViolation::IncompatibleControlType);
                candidate.add_violation(FormFieldBindingViolation::InvalidRadioGroup);
            }
        }
    }
}

fn string_compatibility(semantic_type: &SemanticType) -> FormControlCompatibility {
    nullable_family_compatibility(
        semantic_type,
        |member| {
            matches!(
                member,
                SemanticType::String | SemanticType::StringLiteral(_)
            )
        },
        FormControlNormalization::Text,
        FormControlNormalization::NullableText,
    )
}

fn numeric_compatibility(semantic_type: &SemanticType) -> FormControlCompatibility {
    nullable_family_compatibility(
        semantic_type,
        |member| {
            matches!(
                member,
                SemanticType::Number | SemanticType::NumberLiteral(_)
            )
        },
        FormControlNormalization::Number,
        FormControlNormalization::NullableNumber,
    )
}

fn nullable_family_compatibility(
    semantic_type: &SemanticType,
    matches_member: impl Fn(&SemanticType) -> bool,
    normal: FormControlNormalization,
    nullable: FormControlNormalization,
) -> FormControlCompatibility {
    let members = union_members(semantic_type);
    let has_null = members
        .iter()
        .any(|member| matches!(member, SemanticType::Null));
    let non_null = members
        .iter()
        .filter(|member| !matches!(member, SemanticType::Null))
        .collect::<Vec<_>>();
    if !non_null.is_empty() && non_null.iter().all(|member| matches_member(member)) {
        FormControlCompatibility::Compatible(if has_null { nullable } else { normal })
    } else {
        FormControlCompatibility::Incompatible
    }
}

fn boolean_compatibility(semantic_type: &SemanticType) -> FormControlCompatibility {
    let members = union_members(semantic_type);
    if !members.is_empty()
        && members.iter().all(|member| {
            matches!(
                member,
                SemanticType::Boolean | SemanticType::BooleanLiteral(_)
            )
        })
    {
        FormControlCompatibility::Compatible(FormControlNormalization::Boolean)
    } else {
        FormControlCompatibility::Incompatible
    }
}

fn scalar_compatibility(
    semantic_type: &SemanticType,
    allow_null: bool,
) -> FormControlCompatibility {
    let members = union_members(semantic_type);
    if !members.is_empty()
        && members.iter().all(|member| {
            matches!(
                member,
                SemanticType::String
                    | SemanticType::StringLiteral(_)
                    | SemanticType::Number
                    | SemanticType::NumberLiteral(_)
                    | SemanticType::Boolean
                    | SemanticType::BooleanLiteral(_)
            ) || (allow_null && matches!(member, SemanticType::Null))
        })
    {
        FormControlCompatibility::Compatible(FormControlNormalization::Scalar)
    } else {
        FormControlCompatibility::Incompatible
    }
}

fn array_scalar_compatibility(semantic_type: &SemanticType) -> FormControlCompatibility {
    let SemanticType::Array(element) = semantic_type else {
        return FormControlCompatibility::Incompatible;
    };
    if scalar_compatibility(element, true)
        == FormControlCompatibility::Compatible(FormControlNormalization::Scalar)
    {
        FormControlCompatibility::Compatible(FormControlNormalization::ScalarArray)
    } else {
        FormControlCompatibility::Incompatible
    }
}

fn union_members(semantic_type: &SemanticType) -> Vec<&SemanticType> {
    match semantic_type {
        SemanticType::Union(members) => members.iter().collect(),
        _ => vec![semantic_type],
    }
}

fn mark_binding_multiplicity(candidates: &mut [FormFieldBindingCandidate]) {
    let mut groups = BTreeMap::<(SemanticId, FieldId), Vec<usize>>::new();
    for (index, candidate) in candidates.iter().enumerate() {
        if !candidate.is_valid() {
            continue;
        }
        if let (Some(component), Some(field)) =
            (&candidate.owner_component, &candidate.resolved_field)
        {
            groups
                .entry((component.clone(), field.clone()))
                .or_default()
                .push(index);
        }
    }
    for indexes in groups.into_values().filter(|indexes| indexes.len() > 1) {
        let all_radio = indexes.iter().all(|index| {
            candidates[*index].channel == Some(FormControlChannel::RadioValue)
                && candidates[*index].input_kind == Some(FormInputKind::Radio)
        });
        if !all_radio {
            for index in indexes {
                candidates[index].add_violation(FormFieldBindingViolation::DuplicateFieldControl);
            }
            continue;
        }
        let mut values = BTreeMap::<String, Vec<usize>>::new();
        for index in &indexes {
            if let Some(value) = &candidates[*index].radio_value {
                values.entry(format!("{value:?}")).or_default().push(*index);
            }
        }
        let duplicate_indexes = values
            .into_values()
            .filter(|members| members.len() > 1)
            .flatten()
            .collect::<BTreeSet<_>>();
        if !duplicate_indexes.is_empty() {
            for index in indexes {
                candidates[index].add_violation(FormFieldBindingViolation::InvalidRadioGroup);
                if duplicate_indexes.contains(&index) {
                    candidates[index].add_violation(FormFieldBindingViolation::DuplicateRadioValue);
                }
            }
        }
    }
}

fn lower_valid_binding(candidate: &FormFieldBindingCandidate) -> FormFieldBinding {
    let control = candidate
        .control_entity
        .clone()
        .expect("valid binding has control identity");
    let field = candidate
        .resolved_field
        .clone()
        .expect("valid binding has Field identity");
    FormFieldBinding {
        id: FieldBindingId::for_control(&control, &field),
        owner_template: candidate
            .owner_template
            .clone()
            .expect("valid binding has template owner"),
        control_entity: control,
        field,
        form: candidate
            .resolved_form
            .clone()
            .expect("valid binding has Form identity"),
        component: candidate
            .owner_component
            .clone()
            .expect("valid binding has component owner"),
        element_name: candidate
            .element_name
            .clone()
            .expect("valid binding has element name"),
        input_kind: candidate.input_kind,
        channel: candidate
            .channel
            .expect("valid binding has control channel"),
        compatibility: candidate
            .compatibility
            .expect("valid binding has compatibility"),
        field_type: candidate
            .field_type
            .clone()
            .expect("valid binding has Field type"),
        radio_value: candidate.radio_value.clone(),
        authored_order: candidate.authored_order,
        provenance: candidate.provenance.clone(),
        attribute_provenance: candidate.attribute_provenance.clone(),
        expression_provenance: candidate
            .expression_provenance
            .clone()
            .expect("valid binding has expression provenance"),
        control_kind_provenance: candidate
            .control_kind_provenance
            .clone()
            .expect("valid binding has control-kind provenance"),
        radio_value_provenance: candidate.radio_value_provenance.clone(),
    }
}

fn attributes_named<'a>(element: &'a ElementNode, name: &str) -> Vec<&'a RenderAttribute> {
    element
        .authored_attributes
        .iter()
        .filter(|attribute| attribute.name == name)
        .collect()
}

fn has_attribute(element: &ElementNode, name: &str) -> bool {
    element
        .authored_attributes
        .iter()
        .any(|attribute| attribute.name == name)
}

fn selected_option_provenance(
    children: &[TemplateChild],
    path: &std::path::Path,
) -> Option<SourceProvenance> {
    for child in children {
        match child {
            TemplateChild::Element(element) => {
                if element.tag_name == "option" {
                    if let Some(selected) = element
                        .authored_attributes
                        .iter()
                        .find(|attribute| attribute.name == "selected")
                    {
                        return Some(SourceProvenance::new(path, selected.span));
                    }
                }
                if let Some(provenance) = selected_option_provenance(&element.children, path) {
                    return Some(provenance);
                }
            }
            TemplateChild::Fragment(fragment) => {
                if let Some(provenance) = selected_option_provenance(&fragment.children, path) {
                    return Some(provenance);
                }
            }
            TemplateChild::Conditional(conditional) => {
                if let Some(provenance) = selected_option_provenance(&conditional.when_true, path)
                    .or_else(|| selected_option_provenance(&conditional.when_false, path))
                {
                    return Some(provenance);
                }
            }
            TemplateChild::List(list) => {
                if let Some(provenance) = selected_option_provenance(&list.item_template, path) {
                    return Some(provenance);
                }
            }
            TemplateChild::Text { .. } | TemplateChild::Binding { .. } => {}
        }
    }
    None
}

fn candidate_source_order(
    left: &FormFieldBindingCandidate,
    right: &FormFieldBindingCandidate,
) -> std::cmp::Ordering {
    (
        left.provenance.path.as_path(),
        left.attribute_provenance.span.start,
        left.attribute_provenance.span.end,
        left.id.as_str(),
    )
        .cmp(&(
            right.provenance.path.as_path(),
            right.attribute_provenance.span.start,
            right.attribute_provenance.span.end,
            right.id.as_str(),
        ))
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use crate::{
        build_application_semantic_model, build_application_semantic_model_for_unit,
        build_semantic_graph, validate_application_semantic_model, CompilationUnit, FieldBindingId,
        FieldId, FormControlChannel, FormControlCompatibility, FormControlNormalization,
        FormFieldBindingViolation, FormId, SemanticEntityKind, SemanticOwner,
        SemanticReferenceKind,
    };

    #[test]
    #[allow(clippy::too_many_lines)]
    fn lowers_supported_controls_with_exact_fields_channels_ownership_and_references() {
        let source = r#"
@component("profile-editor")
class ProfileEditor {
  @form() profile!: Form;
  @form() preferences!: Form;
  @field(this.profile) displayName = "";
  @field(this.profile) email = "";
  @field(this.profile) password: string | null = null;
  @field(this.profile) age = 18;
  @field(this.profile) distance = 5;
  @field(this.profile) enabled = false;
  @field(this.profile) contact: "email" | "phone" = "email";
  @field(this.profile) biography = "";
  @field(this.preferences) country = "us";
  @field(this.preferences) tags: string[] = [];
  render() {
    return <main>
      <section><input field={this.displayName} /></section>
      <input type="email" field={this.email} />
      <input type="password" field={this.password} />
      <input type="number" field={this.age} />
      <input type="range" field={this.distance} />
      <input type="checkbox" field={this.enabled} />
      <input type="radio" value="email" field={this.contact} />
      <input type="radio" value="phone" field={this.contact} />
      <textarea field={this.biography} />
      <select field={this.country}><option value="us">US</option></select>
      <select multiple field={this.tags}><option value="compiler">Compiler</option></select>
    </main>;
  }
}
"#;
        let parsed = ezc_parser::parse_file("src/ProfileEditor.tsx", source);
        let asm = build_application_semantic_model(&parsed);

        assert_eq!(asm.form_field_binding_candidates().len(), 11);
        assert_eq!(asm.form_field_bindings().len(), 11);
        assert!(asm
            .form_field_binding_candidates()
            .iter()
            .all(|candidate| candidate.is_valid() && candidate.binding_id.is_some()));
        assert_eq!(
            asm.form_field_bindings()
                .iter()
                .map(|binding| binding.authored_order)
                .collect::<Vec<_>>(),
            (0..11).collect::<Vec<_>>()
        );

        let component = &asm.components[0].id;
        let profile = FormId::for_owner(component, "profile");
        let preferences = FormId::for_owner(component, "preferences");
        let display = FieldId::for_form(&profile, "displayName");
        let display_binding = asm
            .form_field_bindings()
            .into_iter()
            .find(|binding| binding.field == display)
            .expect("display binding");
        assert_eq!(display_binding.channel, FormControlChannel::Value);
        assert_eq!(
            display_binding.compatibility,
            FormControlCompatibility::Compatible(FormControlNormalization::Text)
        );
        assert_eq!(display_binding.form, profile);
        assert_eq!(display_binding.component, *component);
        assert_eq!(
            asm.owner(display_binding.id.as_semantic_id()),
            Some(&SemanticOwner::entity(
                display_binding.control_entity.clone()
            ))
        );
        assert!(asm
            .entity(display_binding.id.as_semantic_id())
            .is_some_and(|entity| entity.kind() == SemanticEntityKind::FormFieldBinding));
        assert_eq!(
            display_binding.id,
            FieldBindingId::for_control(&display_binding.control_entity, &display)
        );

        let password = asm
            .form_field_bindings()
            .into_iter()
            .find(|binding| binding.field == FieldId::for_form(&profile, "password"))
            .expect("password binding");
        assert_eq!(
            password.compatibility,
            FormControlCompatibility::Compatible(FormControlNormalization::NullableText)
        );
        assert!(asm.form_field_bindings().iter().any(|binding| {
            binding.form == preferences && binding.channel == FormControlChannel::SelectedValues
        }));
        assert_eq!(
            asm.references_of_kind(SemanticReferenceKind::FieldBindingField)
                .len(),
            11
        );
        assert_eq!(
            asm.references_of_kind(SemanticReferenceKind::FieldBindingForm)
                .len(),
            11
        );
        assert!(validate_application_semantic_model(&asm).is_empty());
        let graph = build_semantic_graph(&asm);
        assert!(graph
            .nodes
            .iter()
            .any(|node| node.id.as_str().contains("/field-binding:")));
        assert!(graph
            .edges
            .iter()
            .any(|edge| edge.kind == crate::SemanticGraphEdgeKind::FieldBindingBindsField));

        let executable_attributes = &asm.templates[0]
            .root
            .as_ref()
            .expect("root")
            .children
            .iter()
            .find_map(|child| match child {
                crate::TemplateChild::Element(section) if section.tag_name == "section" => {
                    section.children.iter().find_map(|child| match child {
                        crate::TemplateChild::Element(input) => Some(&input.attributes),
                        _ => None,
                    })
                }
                _ => None,
            })
            .expect("nested input attributes");
        assert!(executable_attributes
            .iter()
            .all(|attribute| attribute.name != "field"));
    }

    #[test]
    fn retains_invalid_syntax_controls_and_partially_resolved_evidence_without_binding_ids() {
        let source = r#"
@component("invalid-bindings")
class InvalidBindings {
  @form() form!: Form;
  @field(this.form) text = "";
  @field(this.form) tags: string[] = [];
  render() {
    return <main>
      <input field />
      <input field="text" />
      <input field={text} />
      <input field={this.form.text} />
      <input field={this["text"]} />
      <input field={getField()} />
      <input field={condition ? this.text : this.tags} />
      <input field={this.text} field={this.tags} />
      <input field={this.text} {...props} />
      <div field={this.text} />
      <button field={this.text} />
      <MyInput field={this.text} />
      <input type="file" field={this.text} />
      <input type={this.kind} field={this.text} />
      <textarea field={this.text}>Initial</textarea>
      <select multiple={this.multiple} field={this.tags} />
      <input field={this.text} />
    </main>;
  }
}
"#;
        let asm = build_application_semantic_model(&ezc_parser::parse_file(
            "src/InvalidBindings.tsx",
            source,
        ));
        let candidates = asm.form_field_binding_candidates();
        assert_eq!(candidates.len(), 18);
        assert_eq!(
            candidates
                .iter()
                .map(|candidate| candidate.id.clone())
                .collect::<BTreeSet<_>>()
                .len(),
            candidates.len()
        );
        assert!(candidates
            .iter()
            .filter(|candidate| !candidate.is_valid())
            .all(|candidate| candidate.binding_id.is_none()));
        assert!(candidates.iter().any(|candidate| candidate
            .violations
            .contains(&FormFieldBindingViolation::InvalidBindingExpression)));
        assert!(candidates.iter().any(|candidate| candidate
            .violations
            .contains(&FormFieldBindingViolation::DuplicateBindingAttribute)));
        assert!(candidates.iter().any(|candidate| candidate
            .violations
            .contains(&FormFieldBindingViolation::SpreadConflict)));
        assert!(candidates.iter().any(|candidate| candidate
            .violations
            .contains(&FormFieldBindingViolation::InvalidControlElement)));
        assert!(candidates.iter().any(|candidate| candidate
            .violations
            .contains(&FormFieldBindingViolation::InvalidInputType)));
        assert!(candidates.iter().any(|candidate| candidate
            .violations
            .contains(&FormFieldBindingViolation::DynamicInputType)));
        assert!(candidates.iter().any(|candidate| candidate
            .violations
            .contains(&FormFieldBindingViolation::UnsupportedTextareaChildren)));
        assert!(candidates.iter().any(|candidate| candidate
            .violations
            .contains(&FormFieldBindingViolation::UnsupportedDynamicMultiple)));
        let resolved_invalid = candidates
            .iter()
            .find(|candidate| {
                candidate
                    .violations
                    .contains(&FormFieldBindingViolation::InvalidControlElement)
                    && candidate.resolved_field.is_some()
            })
            .expect("partially resolved invalid control");
        assert!(resolved_invalid.resolved_form.is_some());
        assert!(!resolved_invalid.evidence.is_empty() || resolved_invalid.element_name.is_some());
        assert_eq!(asm.form_field_bindings().len(), 1);
    }

    #[test]
    #[allow(clippy::too_many_lines)]
    fn enforces_resolution_compatibility_multiplicity_radio_and_channel_conflicts() {
        let source = r#"
@component("binding-rules")
class BindingRules {
  @form() first!: Form;
  @form() second!: Form;
  @field(this.first) duplicate = "";
  @field(this.first) contact: "email" | "phone" = "email";
  @field(this.first) badRadio: 1 | 2 = 1;
  @field(this.first) missingRadio = "";
  @field(this.first) enabled: boolean | null = null;
  @field(this.first) scalar = "";
  @field(this.first) object = { value: "" };
  @field(this.first) nullableNumber: number | null = null;
  @field(this.first) conflictText = "";
  @field(this.first) conflictCheck = false;
  @field(this.first) duplicateCheck = false;
  @field(this.first) conflictSelect = "us";
  @field(this.first) eventText = "";
  @field(this.first) changeText = "";
  @field(this.first) dynamicValue = "";
  @field(this.first) clickText = "";
  @field(this.first) selectedText = "us";
  @field(this.first) ambiguous = "first";
  @field(this.second) ambiguous = "second";
  ordinary = state("");
  render() {
    return <main>
      <input field={this.duplicate} />
      <textarea field={this.duplicate} />
      <input type="radio" value="email" field={this.contact} />
      <input type="radio" value="phone" field={this.contact} />
      <input type="radio" value="email" field={this.contact} />
      <input type="radio" value="one" field={this.badRadio} />
      <input type="radio" field={this.missingRadio} />
      <input type="checkbox" field={this.enabled} />
      <select multiple field={this.scalar} />
      <select field={this.object} />
      <input type="number" field={this.nullableNumber} />
      <input value="authored" field={this.conflictText} />
      <input type="checkbox" checked field={this.conflictCheck} />
      <input type="checkbox" field={this.duplicateCheck} />
      <input type="checkbox" field={this.duplicateCheck} />
      <select defaultValue="us" field={this.conflictSelect} />
      <input onInput={this.track} field={this.eventText} />
      <input onChange={this.track} field={this.changeText} />
      <input value={this.ordinary} field={this.dynamicValue} />
      <input onClick={this.track} field={this.clickText} />
      <select field={this.selectedText}><option selected value="us">US</option></select>
      <input field={this.ordinary} />
      <input field={this.missing} />
      <input field={this.ambiguous} />
      <input field={this.conflictText} />
    </main>;
  }
  track() {}
}
"#;
        let asm = build_application_semantic_model(&ezc_parser::parse_file(
            "src/BindingRules.tsx",
            source,
        ));
        let candidates = asm.form_field_binding_candidates();
        assert!(
            candidates
                .iter()
                .filter(|candidate| {
                    candidate
                        .violations
                        .contains(&FormFieldBindingViolation::DuplicateFieldControl)
                })
                .count()
                >= 4
        );
        assert!(candidates.iter().any(|candidate| candidate
            .violations
            .contains(&FormFieldBindingViolation::DuplicateRadioValue)));
        assert!(candidates.iter().any(|candidate| candidate
            .violations
            .contains(&FormFieldBindingViolation::MissingRadioValue)));
        assert!(candidates.iter().any(|candidate| candidate
            .violations
            .contains(&FormFieldBindingViolation::IncompatibleControlType)));
        assert!(candidates.iter().any(|candidate| candidate
            .violations
            .contains(&FormFieldBindingViolation::CompetingValueBinding)));
        assert!(candidates.iter().any(|candidate| candidate
            .violations
            .contains(&FormFieldBindingViolation::CompetingCheckedBinding)));
        assert!(candidates.iter().any(|candidate| candidate
            .violations
            .contains(&FormFieldBindingViolation::CompetingDefaultValue)));
        assert!(candidates.iter().any(|candidate| candidate
            .violations
            .contains(&FormFieldBindingViolation::CompetingSelectedState)));
        assert!(candidates.iter().any(|candidate| candidate
            .violations
            .contains(&FormFieldBindingViolation::CompetingChangeHandler)));
        assert!(candidates.iter().any(|candidate| candidate
            .violations
            .contains(&FormFieldBindingViolation::UnresolvedField)));
        assert!(candidates.iter().any(|candidate| candidate
            .violations
            .contains(&FormFieldBindingViolation::AmbiguousField)));
        let nullable_number = asm
            .form_field_bindings()
            .into_iter()
            .find(|binding| binding.channel == FormControlChannel::NumericValue)
            .expect("valid nullable number binding");
        assert_eq!(
            nullable_number.compatibility,
            FormControlCompatibility::Compatible(FormControlNormalization::NullableNumber)
        );
        assert!(asm.form_field_bindings().iter().all(|binding| {
            binding.field
                != FieldId::for_form(
                    &FormId::for_owner(&asm.components[0].id, "first"),
                    "contact",
                )
        }));
        assert!(asm.form_field_bindings().iter().any(|binding| {
            binding.field
                == FieldId::for_form(
                    &FormId::for_owner(&asm.components[0].id, "first"),
                    "clickText",
                )
        }));
    }

    #[test]
    fn recognizes_the_complete_static_input_kind_contract() {
        let supported = [
            "text",
            "email",
            "password",
            "search",
            "tel",
            "url",
            "number",
            "checkbox",
            "radio",
            "date",
            "time",
            "datetime-local",
            "month",
            "week",
            "range",
            "hidden",
        ];
        assert!(supported
            .iter()
            .all(|kind| super::FormInputKind::from_static(kind).is_some()));
        assert!(super::FormInputKind::from_static("file").is_none());
        assert!(super::FormInputKind::from_static("color").is_none());
    }

    #[test]
    fn preserves_valid_radio_groups_and_allows_the_same_name_in_separate_components() {
        let first = r#"
@component("first") class First {
  @form() form!: Form;
  @field(this.form) choice: "a" | "b" = "a";
  render() { return <main><input type="radio" value="a" field={this.choice} /><input type="radio" value="b" field={this.choice} /></main>; }
}
"#;
        let second = r#"
@component("second") class Second {
  @form() form!: Form;
  @field(this.form) choice = "";
  render() { return <input field={this.choice} />; }
}
"#;
        let unit =
            CompilationUnit::parse_sources([("src/First.tsx", first), ("src/Second.tsx", second)]);
        let reversed =
            CompilationUnit::parse_sources([("src/Second.tsx", second), ("src/First.tsx", first)]);
        let asm = build_application_semantic_model_for_unit(&unit);
        let reversed = build_application_semantic_model_for_unit(&reversed);
        assert_eq!(
            asm.form_field_binding_candidates,
            reversed.form_field_binding_candidates
        );
        assert_eq!(asm.form_field_bindings, reversed.form_field_bindings);
        assert_eq!(asm.form_field_bindings().len(), 3);
        assert_eq!(
            asm.form_field_bindings()
                .iter()
                .filter(|binding| binding.channel == FormControlChannel::RadioValue)
                .count(),
            2
        );
    }

    #[test]
    fn retains_invalid_duplicate_cross_component_and_inherited_field_resolution() {
        let source = r#"
class BaseEditor {
  @form() baseForm!: Form;
  @field(this.baseForm) inherited = "";
}

@component("owner")
class Owner {
  @form() form!: Form;
  @field(this.form) remote = "";
  render() { return <main />; }
}

@component("consumer")
class Consumer extends BaseEditor {
  @form("invalid") invalidForm!: Form;
  @field(this.invalidForm) invalidField = "";
  @form() form!: Form;
  @field(this.form) duplicate = "first";
  @field(this.form) duplicate = "second";
  ordinary = state("");
  render() {
    return <main>
      <input field={this.invalidField} />
      <input field={this.duplicate} />
      <input field={this.remote} />
      <input field={this.inherited} />
      <input field={this.ordinary} />
    </main>;
  }
}
"#;
        let asm =
            build_application_semantic_model(&ezc_parser::parse_file("src/Resolution.tsx", source));
        let candidates = asm.form_field_binding_candidates();
        assert!(asm.form_field_bindings().is_empty());
        assert!(candidates.iter().any(|candidate| {
            candidate.authored_field_name.as_deref() == Some("invalidField")
                && candidate
                    .violations
                    .contains(&FormFieldBindingViolation::InvalidFieldDeclaration)
        }));
        assert!(candidates.iter().any(|candidate| {
            candidate.authored_field_name.as_deref() == Some("duplicate")
                && candidate
                    .violations
                    .contains(&FormFieldBindingViolation::AmbiguousField)
        }));
        assert!(candidates.iter().any(|candidate| {
            candidate.authored_field_name.as_deref() == Some("remote")
                && candidate
                    .violations
                    .contains(&FormFieldBindingViolation::CrossComponentField)
        }));
        assert!(candidates.iter().any(|candidate| {
            candidate.authored_field_name.as_deref() == Some("inherited")
                && candidate
                    .violations
                    .contains(&FormFieldBindingViolation::UnresolvedField)
        }));
        assert!(candidates.iter().any(|candidate| {
            candidate.authored_field_name.as_deref() == Some("ordinary")
                && candidate
                    .violations
                    .contains(&FormFieldBindingViolation::UnresolvedField)
        }));
    }
}
