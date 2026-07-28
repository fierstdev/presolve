use std::collections::{BTreeMap, BTreeSet};

use crate::{
    infer_serializable_value_type, is_assignable, serialization_compatibility,
    state_initializer_value_type, ComponentNode, ExecutionBoundary, FieldId, FormEntity,
    FormFieldDeclarationCandidate, FormFieldDeclarationCandidateId, FormFieldDeclarationViolation,
    FormId, SemanticId, SemanticType, SemanticTypeAssignment, SemanticTypeId, SemanticTypeModel,
    SemanticTypeStatus, SerializableValue, SerializationCompatibility, SourceProvenance,
};

/// First-class immutable compiler-owned Form Field declaration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FormFieldEntity {
    pub id: FieldId,
    pub owner_form: FormId,
    pub owner_component: SemanticId,
    pub authored_field: SemanticId,
    pub name: String,
    /// Compiler-issued serialized leaf path. Current one-argument Fields use
    /// their name as a single root segment; N7-B will admit retained nested
    /// segments only with the complete artifact/runtime contract.
    pub path: Vec<String>,
    pub semantic_type: SemanticType,
    pub type_assignment: SemanticTypeAssignment,
    pub initial_value: SerializableValue,
    pub declaration_order: usize,
    pub provenance: SourceProvenance,
    pub form_designator_provenance: SourceProvenance,
    pub field_name_provenance: SourceProvenance,
    pub type_provenance: SourceProvenance,
    pub initializer_provenance: SourceProvenance,
    pub boundary: ExecutionBoundary,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FormFieldProducts {
    pub candidates: Vec<FormFieldDeclarationCandidate>,
    pub fields: BTreeMap<FieldId, FormFieldEntity>,
}

/// Resolve normalized I3 candidates exclusively through canonical I2 Forms and
/// the existing Phase C type/value authorities.
///
/// # Panics
///
/// Panics if a candidate with no retained violations lacks a canonical Form,
/// component, name, type, value, or provenance input. Such a candidate violates
/// the I3 staged-lowering invariant.
#[must_use]
pub fn collect_form_field_products(
    components: &[ComponentNode],
    forms: &BTreeMap<FormId, FormEntity>,
    semantic_types: &SemanticTypeModel,
    bindings: Option<&crate::BindingTable>,
) -> FormFieldProducts {
    let mut candidates = components
        .iter()
        .flat_map(|component| component.form_field_declaration_candidates.clone())
        .collect::<Vec<_>>();
    candidates.sort_by(candidate_source_order);

    let valid_forms = forms
        .values()
        .filter_map(|form| {
            Some((
                (form.owner.entity_id()?.clone(), form.name.clone()),
                form.id.clone(),
            ))
        })
        .collect::<BTreeMap<_, _>>();
    let components_by_id = components
        .iter()
        .map(|component| (component.id.clone(), component))
        .collect::<BTreeMap<_, _>>();
    let mut alias_origins = BTreeMap::<FormFieldDeclarationCandidateId, SemanticId>::new();

    for candidate in &mut candidates {
        resolve_candidate_form(
            candidate,
            components,
            &components_by_id,
            &valid_forms,
            forms,
        );
        resolve_candidate_type(candidate, semantic_types, bindings, &mut alias_origins);
    }

    mark_duplicate_names(&mut candidates);
    mark_conflicting_paths(&mut candidates);

    let mut fields = BTreeMap::new();
    let mut authored_orders = BTreeMap::<FormId, usize>::new();
    for candidate in &mut candidates {
        if !candidate.is_valid() {
            candidate.field_id = None;
            candidate.type_assignment = None;
            continue;
        }
        let (id, entity) = lower_valid_candidate(candidate, &alias_origins, &mut authored_orders);
        fields.insert(id, entity);
    }

    FormFieldProducts { candidates, fields }
}

fn lower_valid_candidate(
    candidate: &mut FormFieldDeclarationCandidate,
    alias_origins: &BTreeMap<FormFieldDeclarationCandidateId, SemanticId>,
    authored_orders: &mut BTreeMap<FormId, usize>,
) -> (FieldId, FormFieldEntity) {
    let owner_form = candidate
        .resolved_form
        .clone()
        .expect("valid Form Field candidate has resolved Form");
    let name = candidate
        .authored_name
        .clone()
        .expect("valid Form Field candidate has authored name");
    let id = FieldId::for_form(&owner_form, &name);
    let semantic_type = candidate
        .semantic_type
        .clone()
        .expect("valid Form Field candidate has semantic type");
    let type_provenance = candidate
        .declared_type
        .as_ref()
        .map(|declared| declared.provenance.clone())
        .or_else(|| candidate.initializer_provenance.clone())
        .expect("valid Form Field candidate has type provenance");
    let type_assignment = SemanticTypeAssignment {
        id: SemanticTypeId::for_subject(id.as_semantic_id()),
        subject: id.as_semantic_id().clone(),
        semantic_type: semantic_type.clone(),
        origin: alias_origins
            .get(&candidate.id)
            .cloned()
            .unwrap_or_else(|| id.as_semantic_id().clone()),
        status: if candidate.declared_type.is_some() {
            SemanticTypeStatus::Declared
        } else {
            SemanticTypeStatus::Inferred
        },
        provenance: candidate.provenance.clone(),
    };
    let declaration_order = authored_orders.entry(owner_form.clone()).or_default();
    let entity = FormFieldEntity {
        id: id.clone(),
        owner_form,
        owner_component: candidate
            .owner_component
            .clone()
            .expect("valid Form Field candidate has component owner"),
        authored_field: candidate
            .declaration_field
            .clone()
            .expect("valid Form Field candidate has authored field identity"),
        path: candidate
            .nested_path_segments
            .clone()
            .unwrap_or_else(|| vec![name.clone()]),
        name,
        semantic_type,
        type_assignment: type_assignment.clone(),
        initial_value: candidate
            .initializer
            .clone()
            .expect("valid Form Field candidate has initial value"),
        declaration_order: *declaration_order,
        provenance: candidate.provenance.clone(),
        form_designator_provenance: candidate
            .form_designator
            .as_ref()
            .expect("valid Form Field candidate has designator")
            .provenance
            .clone(),
        field_name_provenance: candidate
            .name_provenance
            .clone()
            .expect("valid Form Field candidate has name provenance"),
        type_provenance,
        initializer_provenance: candidate
            .initializer_provenance
            .clone()
            .expect("valid Form Field candidate has initializer provenance"),
        boundary: ExecutionBoundary::Client,
    };
    *declaration_order += 1;
    candidate.field_id = Some(id.clone());
    candidate.type_assignment = Some(type_assignment);
    (id, entity)
}

fn resolve_candidate_form(
    candidate: &mut FormFieldDeclarationCandidate,
    components: &[ComponentNode],
    components_by_id: &BTreeMap<SemanticId, &ComponentNode>,
    valid_forms: &BTreeMap<(SemanticId, String), FormId>,
    forms: &BTreeMap<FormId, FormEntity>,
) {
    if let (Some(owner), Some(designator)) = (
        candidate.owner_component.as_ref(),
        candidate.form_designator.as_ref(),
    ) {
        if let Some(form) = valid_forms.get(&(owner.clone(), designator.authored_name.clone())) {
            candidate.resolved_form = Some(form.clone());
            return;
        }
        let has_invalid_local_form = components_by_id.get(owner).is_some_and(|component| {
            component
                .form_declaration_candidates
                .iter()
                .any(|form| form.authored_name.as_deref() == Some(&designator.authored_name))
        });
        if has_invalid_local_form {
            candidate.add_violation(FormFieldDeclarationViolation::InvalidForm);
            return;
        }
        let inherited = components_by_id.get(owner).is_some_and(|component| {
            component.heritage.as_ref().is_some_and(|heritage| {
                components.iter().any(|base| {
                    base.class_name == heritage.base
                        && base.form_declaration_candidates.iter().any(|form| {
                            form.authored_name.as_deref() == Some(&designator.authored_name)
                        })
                })
            })
        });
        if inherited {
            candidate.add_violation(FormFieldDeclarationViolation::InheritedDeclaration);
            candidate.add_violation(FormFieldDeclarationViolation::InvalidForm);
        } else {
            candidate.add_violation(FormFieldDeclarationViolation::UnresolvedForm);
        }
        return;
    }

    if let Some(unsupported) = &candidate.unsupported_form_designator {
        let cross_component = components.iter().any(|component| {
            component.class_name == unsupported.object
                && forms.values().any(|form| {
                    form.owner.entity_id() == Some(&component.id) && form.name == unsupported.member
                })
        });
        if cross_component {
            candidate.add_violation(FormFieldDeclarationViolation::CrossComponentForm);
        }
    }
}

fn resolve_candidate_type(
    candidate: &mut FormFieldDeclarationCandidate,
    semantic_types: &SemanticTypeModel,
    bindings: Option<&crate::BindingTable>,
    alias_origins: &mut BTreeMap<FormFieldDeclarationCandidateId, SemanticId>,
) {
    let Some(initializer) = candidate.initializer.clone() else {
        return;
    };
    let semantic_type = if let Some(authority_type) = candidate.authority_type.clone() {
        if !matches!(initializer, SerializableValue::Array(ref values) if values.is_empty()) {
            candidate.add_violation(FormFieldDeclarationViolation::InitializerTypeMismatch);
        }
        authority_type
    } else if let Some(declared_type) = &candidate.declared_type {
        let Some(resolved) = semantic_types.resolve_declared_type(declared_type, bindings) else {
            candidate.add_violation(FormFieldDeclarationViolation::InvalidDeclaredType);
            return;
        };
        if let Some(origin) = resolved.alias_origin {
            alias_origins.insert(candidate.id.clone(), origin);
        }
        if !is_supported_form_field_type(&resolved.semantic_type) {
            candidate.add_violation(FormFieldDeclarationViolation::InvalidDeclaredType);
        }
        let source = state_initializer_value_type(&initializer);
        if !is_assignable(&source, &resolved.semantic_type) {
            candidate.add_violation(FormFieldDeclarationViolation::InitializerTypeMismatch);
        }
        resolved.semantic_type
    } else {
        infer_serializable_value_type(&initializer)
    };
    if candidate.authority_type.is_none()
        && serialization_compatibility(&semantic_type) != SerializationCompatibility::Serializable
    {
        candidate.add_violation(FormFieldDeclarationViolation::NonSerializableType);
    }
    candidate.semantic_type = Some(semantic_type);
}

fn is_supported_form_field_type(semantic_type: &SemanticType) -> bool {
    match semantic_type {
        SemanticType::Null
        | SemanticType::Boolean
        | SemanticType::Number
        | SemanticType::String
        | SemanticType::BooleanLiteral(_)
        | SemanticType::NumberLiteral(_)
        | SemanticType::StringLiteral(_) => true,
        SemanticType::File => false,
        SemanticType::Array(element) => is_supported_form_field_type(element),
        SemanticType::Tuple(items) | SemanticType::Union(items) => {
            !items.is_empty() && items.iter().all(is_supported_form_field_type)
        }
        SemanticType::Object(object) => {
            object.properties.values().all(is_supported_form_field_type)
        }
        SemanticType::Unknown
        | SemanticType::Never
        | SemanticType::Form
        | SemanticType::SlotContent
        | SemanticType::Resource(_) => false,
    }
}

fn mark_duplicate_names(candidates: &mut [FormFieldDeclarationCandidate]) {
    let mut groups = BTreeMap::<(FormId, String), Vec<usize>>::new();
    for (index, candidate) in candidates.iter().enumerate() {
        if let (Some(form), Some(name)) = (&candidate.resolved_form, &candidate.authored_name) {
            groups
                .entry((form.clone(), name.clone()))
                .or_default()
                .push(index);
        }
    }
    let duplicate_groups = groups
        .into_values()
        .filter(|indexes| {
            indexes
                .iter()
                .map(|index| {
                    let provenance = &candidates[*index].provenance;
                    (
                        provenance.path.as_path(),
                        provenance.span.start,
                        provenance.span.end,
                    )
                })
                .collect::<BTreeSet<_>>()
                .len()
                > 1
        })
        .collect::<Vec<_>>();
    for indexes in duplicate_groups {
        for index in indexes {
            candidates[index].add_violation(FormFieldDeclarationViolation::DuplicateName);
        }
    }
}

/// A JSON Field path is a compiler-owned shape, not a bag of author strings.
/// Therefore a leaf may neither duplicate another leaf nor become an object
/// prefix of another leaf in the same Form. Invalid candidates are excluded:
/// their primary error is more useful and they have no executable path.
fn mark_conflicting_paths(candidates: &mut [FormFieldDeclarationCandidate]) {
    let mut groups = BTreeMap::<(FormId, Vec<String>), Vec<usize>>::new();
    for (index, candidate) in candidates.iter().enumerate() {
        if !candidate.is_valid() {
            continue;
        }
        if let (Some(form), Some(name)) = (&candidate.resolved_form, &candidate.authored_name) {
            let path = candidate
                .nested_path_segments
                .clone()
                .unwrap_or_else(|| vec![name.clone()]);
            groups.entry((form.clone(), path)).or_default().push(index);
        }
    }
    let entries = groups.into_iter().collect::<Vec<_>>();
    let mut conflicting = BTreeSet::new();
    for (_, candidates) in &entries {
        if candidates.len() > 1 {
            conflicting.extend(candidates.iter().copied());
        }
    }
    for (left_index, ((left_form, left_path), left_candidates)) in entries.iter().enumerate() {
        for (right_form, right_path, right_candidates) in entries
            .iter()
            .skip(left_index + 1)
            .map(|((form, path), indexes)| (form, path, indexes))
        {
            if left_form != right_form || !paths_conflict(left_path, right_path) {
                continue;
            }
            conflicting.extend(left_candidates.iter().copied());
            conflicting.extend(right_candidates.iter().copied());
        }
    }
    for index in conflicting {
        candidates[index].add_violation(FormFieldDeclarationViolation::ConflictingPath);
    }
}

fn paths_conflict(left: &[String], right: &[String]) -> bool {
    (left.len() <= right.len() && left.iter().zip(right).all(|(left, right)| left == right))
        || (right.len() <= left.len() && right.iter().zip(left).all(|(left, right)| left == right))
}

fn candidate_source_order(
    left: &FormFieldDeclarationCandidate,
    right: &FormFieldDeclarationCandidate,
) -> std::cmp::Ordering {
    (
        left.provenance.path.as_path(),
        left.provenance.span.start,
        left.provenance.span.end,
        left.id.as_str(),
    )
        .cmp(&(
            right.provenance.path.as_path(),
            right.provenance.span.start,
            right.provenance.span.end,
            right.id.as_str(),
        ))
}

#[cfg(test)]
mod tests {
    use crate::{
        build_application_semantic_model, build_application_semantic_model_for_unit,
        build_semantic_graph, validate_application_semantic_model, CompilationUnit,
        ExecutionBoundary, FieldId, FormFieldDeclarationViolation, FormId, SemanticEntityKind,
        SemanticOwner, SemanticType, SemanticTypeStatus, SerializableValue,
        SEMANTIC_GRAPH_SCHEMA_VERSION,
    };

    #[test]
    fn lowers_valid_fields_with_exact_ownership_types_values_order_and_provenance() {
        let source = r#"
@component("profile-editor")
class ProfileEditor {
  @form() profileForm!: Form;
  @field(this.profileForm) displayName = "Austin";
  @field(this.profileForm) age: number = 30;
  @field(this.profileForm) selection: string | null = null;
  @field(this.profileForm) tags: string[] = [];
  @field(this.profileForm) point: [string, number] = ["x", 1];
  @field(this.profileForm) address: { city: string; postalCode: string } = { city: "", postalCode: "" };
  @field(this.profileForm) score: number = 1 + 2;
  render() { return <main />; }
}
"#;
        let parsed = presolve_parser::parse_file("src/ProfileEditor.tsx", source);
        let asm = build_application_semantic_model(&parsed);
        let component = &asm.components[0];
        let form_id = FormId::for_owner(&component.id, "profileForm");
        let display_id = FieldId::for_form(&form_id, "displayName");
        let display = asm.form_field(&display_id).expect("displayName Field");

        assert_eq!(asm.form_fields().len(), 7);
        assert_eq!(
            display.id.as_str(),
            "module:src/ProfileEditor.tsx/component:profile-editor/form:profileForm/field:displayName"
        );
        assert_eq!(display.owner_form, form_id);
        assert_eq!(display.owner_component, component.id);
        assert_eq!(display.semantic_type, SemanticType::String);
        assert_eq!(
            display.initial_value,
            SerializableValue::String("Austin".to_string())
        );
        assert_eq!(display.type_assignment.status, SemanticTypeStatus::Inferred);
        assert_eq!(display.boundary, ExecutionBoundary::Client);
        assert_eq!(display.declaration_order, 0);
        assert_eq!(
            asm.owner(display.id.as_semantic_id()),
            Some(&SemanticOwner::entity(
                display.owner_form.as_semantic_id().clone()
            ))
        );
        assert_eq!(
            asm.semantic_type_of(display.id.as_semantic_id()),
            Some(&SemanticType::String)
        );
        assert!(asm
            .entity(display.id.as_semantic_id())
            .is_some_and(|entity| entity.kind() == SemanticEntityKind::FormField));
        assert_eq!(
            asm.form_fields()
                .iter()
                .map(|field| field.declaration_order)
                .collect::<Vec<_>>(),
            (0..7).collect::<Vec<_>>()
        );
        assert_eq!(
            asm.form_field(&FieldId::for_form(&display.owner_form, "score"))
                .expect("score")
                .initial_value,
            SerializableValue::Number("3".to_string())
        );
        assert_eq!(
            display.provenance.span.start,
            source.find("@field(this.profileForm) displayName").unwrap()
        );
        assert!(
            display.form_designator_provenance.span.start
                < display.initializer_provenance.span.start
        );
        let validation = validate_application_semantic_model(&asm);
        assert!(validation.is_empty(), "{validation:#?}");
        let graph = build_semantic_graph(&asm);
        assert_eq!(graph.schema_version, SEMANTIC_GRAPH_SCHEMA_VERSION);
        assert!(graph
            .nodes
            .iter()
            .any(|node| node.id == *display.id.as_semantic_id()));
    }

    #[test]
    fn scopes_names_to_forms_and_is_stable_under_reversed_files() {
        let first_source = r#"
@component("profile")
class Profile {
  @form() billing!: Form;
  @form() shipping!: Form;
  @field(this.billing) address = "billing";
  @field(this.shipping) address = "shipping";
  render() { return <main />; }
}
"#;
        let second_source = r#"
@component("account")
class Account {
  @form() billing!: Form;
  @field(this.billing) address = "account";
  render() { return <main />; }
}
"#;
        let first = CompilationUnit::parse_sources([
            ("src/Profile.tsx", first_source),
            ("src/Account.tsx", second_source),
        ]);
        let reversed = CompilationUnit::parse_sources([
            ("src/Account.tsx", second_source),
            ("src/Profile.tsx", first_source),
        ]);
        let first = build_application_semantic_model_for_unit(&first);
        let reversed = build_application_semantic_model_for_unit(&reversed);

        assert_eq!(first.form_fields, reversed.form_fields);
        assert_eq!(
            first.form_field_declaration_candidates,
            reversed.form_field_declaration_candidates
        );
        assert_eq!(first.form_fields.len(), 3);
        assert!(first
            .form_fields
            .values()
            .all(|field| field.name == "address" && field.path == ["address"]));
    }

    #[test]
    fn lowers_nested_paths_and_rejects_exact_or_prefix_conflicts() {
        let valid = presolve_parser::parse_file(
            "src/Profile.tsx",
            r#"
@component("profile") class Profile {
  @form() profile!: Form;
  @field(this.profile, "address.street") street = "South Congress";
  @field(this.profile, "address.city") city = "Austin";
  render() { return <main />; }
}
"#,
        );
        let valid = build_application_semantic_model(&valid);
        assert_eq!(valid.form_fields().len(), 2);
        assert!(valid
            .form_fields()
            .iter()
            .any(|field| field.name == "street" && field.path == ["address", "street"]));
        assert!(valid
            .form_fields()
            .iter()
            .any(|field| field.name == "city" && field.path == ["address", "city"]));
        let form = FormId::for_owner(&valid.components[0].id, "profile");
        assert_eq!(
            valid.serialization.plans[&crate::SerializationPlanId::for_form(&form)]
                .fields
                .iter()
                .map(|field| field.key.as_str())
                .collect::<Vec<_>>(),
            vec!["address.street", "address.city"]
        );

        let conflicting = presolve_parser::parse_file(
            "src/Conflicting.tsx",
            r#"
@component("conflicting") class Conflicting {
  @form() profile!: Form;
  @field(this.profile, "address") address = "flat";
  @field(this.profile, "address.street") street = "nested";
  render() { return <main />; }
}
"#,
        );
        let conflicting = build_application_semantic_model(&conflicting);
        assert!(conflicting.form_fields().is_empty());
        assert!(conflicting
            .form_field_declaration_candidates()
            .iter()
            .all(|candidate| candidate
                .violations
                .contains(&FormFieldDeclarationViolation::ConflictingPath)));

        let duplicate = presolve_parser::parse_file(
            "src/Duplicate.tsx",
            r#"
@component("duplicate") class Duplicate {
  @form() profile!: Form;
  @field(this.profile, "address.street") primary = "one";
  @field(this.profile, "address.street") secondary = "two";
  render() { return <main />; }
}
"#,
        );
        let duplicate = build_application_semantic_model(&duplicate);
        assert!(duplicate.form_fields().is_empty());
        assert!(duplicate
            .form_field_declaration_candidates()
            .iter()
            .all(|candidate| candidate
                .violations
                .contains(&FormFieldDeclarationViolation::ConflictingPath)));
    }

    #[test]
    #[allow(clippy::too_many_lines)]
    fn retains_invalid_decorators_targets_designators_and_owning_forms_without_field_ids() {
        let source = r#"
@field(this.profileForm)
class ClassTarget {}

class BaseEditor {
  @form() baseForm!: Form;
}

@component("other")
class Other {
  @form() otherForm!: Form;
  render() { return <main />; }
}

@component("profile")
class Profile extends BaseEditor {
  @form() validForm!: Form;
  @form("bad") invalidForm!: Form;
  normalProperty = {};
  @field validBare = "";
  @field() zero = "";
  @field(this.validForm, "invalid-path") many = "";
  @field("validForm") stringArg = "";
  @field(validForm) identifierArg = "";
  @field(this.forms.validForm) chain = "";
  @field(getForm()) call = "";
  @field(this.missing) missing = "";
  @field(this.normalProperty) ordinary = "";
  @field(this.invalidForm) invalid = "";
  @field(this.baseForm) inherited = "";
  @field(Other.otherForm) cross = "";
  @field(this.validForm) static staticField = "";
  @field(this.validForm) ["computed"] = "";
  @field(this.validForm) #privateField = "";
  @field(this.validForm) method() {}
  @field(this.validForm) get getter() { return ""; }
  @field(this.validForm) set setter(value: string) {}
  parameter(@field(this.validForm) value: string) {}
  render() { return <main />; }
}
"#;
        let parsed = presolve_parser::parse_file("src/InvalidFields.tsx", source);
        let asm = build_application_semantic_model(&parsed);
        let candidates = asm.form_field_declaration_candidates();

        assert_eq!(asm.form_fields().len(), 1);
        assert_eq!(asm.form_fields()[0].name, "stringArg");
        assert_eq!(candidates.len(), 20);
        assert_eq!(
            candidates
                .iter()
                .map(|candidate| candidate.id.clone())
                .collect::<std::collections::BTreeSet<_>>()
                .len(),
            candidates.len()
        );
        assert!(candidates.iter().any(|candidate| {
            candidate.authored_name.as_deref() == Some("stringArg")
                && candidate.field_id.is_some()
                && candidate.type_assignment.is_some()
                && candidate.violations.is_empty()
        }));
        assert!(candidates
            .iter()
            .filter(|candidate| { candidate.authored_name.as_deref() != Some("stringArg") })
            .all(|candidate| {
                candidate.field_id.is_none()
                    && candidate.type_assignment.is_none()
                    && !candidate.violations.is_empty()
            }));
        assert!(candidates.iter().any(|candidate| candidate
            .violations
            .contains(&FormFieldDeclarationViolation::InvalidDecoratorInvocation)));
        assert!(candidates.iter().any(|candidate| candidate
            .violations
            .contains(&FormFieldDeclarationViolation::InvalidPath)));
        assert!(candidates
            .iter()
            .any(|candidate| candidate.violations.contains(
                &FormFieldDeclarationViolation::InvalidDecoratorArity {
                    actual: 0,
                    expected: 1,
                }
            )));
        assert!(candidates.iter().any(|candidate| candidate
            .violations
            .contains(&FormFieldDeclarationViolation::UnresolvedForm)));
        assert!(candidates.iter().any(|candidate| candidate
            .violations
            .contains(&FormFieldDeclarationViolation::InvalidForm)));
        assert!(candidates.iter().any(|candidate| candidate
            .violations
            .contains(&FormFieldDeclarationViolation::InheritedDeclaration)));
        assert!(candidates.iter().any(|candidate| candidate
            .violations
            .contains(&FormFieldDeclarationViolation::CrossComponentForm)));
        assert!(candidates.iter().any(|candidate| candidate
            .violations
            .contains(&FormFieldDeclarationViolation::StaticField)));
        assert!(candidates.iter().any(|candidate| candidate
            .violations
            .contains(&FormFieldDeclarationViolation::UnsupportedFieldName)));
    }

    #[test]
    fn rejects_invalid_values_types_duplicates_and_conflicts_without_poisoning_valid_fields() {
        let source = r#"
@component("profile")
class Profile {
  @form() profileForm!: Form;
  @form() otherForm!: Form;
  @field(this.profileForm) good = "ok";
  @field(this.profileForm) missing!: string;
  @field(this.profileForm) declare declared: string;
  @field(this.profileForm) noInitializer: string;
  @field(this.profileForm) call = loadName();
  @field(this.profileForm) wrapped = state("");
  @field(this.profileForm) mismatch: number = "bad";
  @field(this.profileForm) unresolved: MissingType = "";
  @field(this.profileForm) nonSerializable: SlotContent = "";
  @field(this.profileForm) duplicate = "first";
  @field(this.profileForm) duplicate = "second";
  @field(this.otherForm) duplicate = "other";
  @field(this.profileForm) @state() stateConflict = "";
  @form() @field(this.profileForm) formConflict = "";
  @slot() @field(this.profileForm) slotConflict = "";
  @custom() @field(this.profileForm) customConflict = "";
  @field(this.profileForm) @field(this.profileForm) repeatedDecorator = "";
  render() { return <main />; }
}
"#;
        let parsed = presolve_parser::parse_file("src/InvalidValues.tsx", source);
        let asm = build_application_semantic_model(&parsed);
        let candidates = asm.form_field_declaration_candidates();

        assert_eq!(asm.form_fields().len(), 2);
        assert!(asm.form_fields().iter().any(|field| field.name == "good"));
        assert!(asm.form_fields().iter().any(|field| {
            field.name == "duplicate" && field.owner_form.as_str().contains("form:otherForm")
        }));
        assert!(candidates
            .iter()
            .filter(|candidate| {
                candidate.authored_name.as_deref() == Some("duplicate")
                    && candidate
                        .resolved_form
                        .as_ref()
                        .is_some_and(|form| form.as_str().contains("form:profileForm"))
            })
            .all(|candidate| candidate
                .violations
                .contains(&FormFieldDeclarationViolation::DuplicateName)
                && candidate.field_id.is_none()));
        for violation in [
            FormFieldDeclarationViolation::MissingInitializer,
            FormFieldDeclarationViolation::UnsupportedInitializer,
            FormFieldDeclarationViolation::InitializerTypeMismatch,
            FormFieldDeclarationViolation::InvalidDeclaredType,
            FormFieldDeclarationViolation::NonSerializableType,
            FormFieldDeclarationViolation::ConflictingSemanticDecorator,
        ] {
            assert!(
                candidates
                    .iter()
                    .any(|candidate| candidate.violations.contains(&violation)),
                "{violation:?}"
            );
        }
        assert!(candidates
            .iter()
            .filter(|candidate| candidate.authored_name.as_deref() == Some("repeatedDecorator"))
            .all(|candidate| {
                candidate
                    .violations
                    .contains(&FormFieldDeclarationViolation::DuplicateFieldDecorator)
                    && !candidate
                        .violations
                        .contains(&FormFieldDeclarationViolation::DuplicateName)
                    && candidate.field_id.is_none()
            }));
    }

    #[test]
    fn resolves_local_and_imported_aliases_through_existing_type_authorities() {
        let types = r"
export type NullableName = string | null;
";
        let editor = r#"
import { NullableName as Name } from "./types";
type LocalAge = number;
@component("profile")
class Profile {
  @form() profileForm!: Form;
  @field(this.profileForm) name: Name = null;
  @field(this.profileForm) age: LocalAge = 18;
  render() { return <main />; }
}
"#;
        let unit =
            CompilationUnit::parse_sources([("src/types.ts", types), ("src/Profile.tsx", editor)]);
        let asm = build_application_semantic_model_for_unit(&unit);

        assert_eq!(asm.form_fields().len(), 2);
        assert!(asm.form_fields().iter().all(|field| {
            field.type_assignment.status == SemanticTypeStatus::Declared
                && field.type_assignment.origin != field.id.as_semantic_id().clone()
        }));
    }
}
