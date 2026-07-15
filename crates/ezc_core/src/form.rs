use std::collections::BTreeMap;

use crate::{
    ComponentNode, ExecutionBoundary, FormDeclarationStatus, FormId, SemanticId, SemanticOwner,
    SourceProvenance,
};

/// First-class immutable compiler-owned Form declaration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FormEntity {
    pub id: FormId,
    pub owner: SemanticOwner,
    pub authored_field: SemanticId,
    pub name: String,
    pub provenance: SourceProvenance,
    pub boundary: ExecutionBoundary,
}

/// Lower only valid normalized I2 candidates into canonical Form entities.
/// Invalid and duplicate candidates remain on their owning component as
/// immutable diagnostic inputs and never become executable products.
///
/// # Panics
///
/// Panics if a candidate marked valid is missing canonical owner, Form, or
/// authored-field identity. That would violate the I2 staging invariant.
#[must_use]
pub fn collect_form_entities(components: &[ComponentNode]) -> BTreeMap<FormId, FormEntity> {
    components
        .iter()
        .flat_map(|component| component.form_declaration_candidates.iter())
        .filter(|candidate| candidate.status == FormDeclarationStatus::Valid)
        .map(|candidate| {
            let id = candidate
                .form_id
                .clone()
                .expect("valid Form candidate has canonical identity");
            let owner = candidate
                .owner_component
                .clone()
                .expect("valid Form candidate has canonical owner");
            (
                id.clone(),
                FormEntity {
                    id,
                    owner: SemanticOwner::entity(owner),
                    authored_field: candidate
                        .authored_field
                        .clone()
                        .expect("valid Form candidate has authored field identity"),
                    name: candidate
                        .authored_name
                        .clone()
                        .expect("valid Form candidate has authored name"),
                    provenance: candidate.provenance.clone(),
                    boundary: ExecutionBoundary::Client,
                },
            )
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use crate::{
        build_application_semantic_model, build_application_semantic_model_for_unit,
        build_semantic_graph, validate_application_semantic_model, AuthoredDeclarationKind,
        CompilationUnit, ExecutionBoundary, FormDeclarationStatus, FormDeclarationViolation,
        FormId, SemanticEntityKind, SemanticOwner, SemanticType, SEMANTIC_GRAPH_SCHEMA_VERSION,
    };

    #[test]
    fn lowers_valid_form_fields_with_identity_ownership_type_and_provenance() {
        let source = r#"
@component("user-profile")
class UserProfile {
  @form()
  profileForm!: Form;

  @form()
  declare settingsForm: Form;

  render() { return <main />; }
}
"#;
        let parsed = ezc_parser::parse_file("src/UserProfile.tsx", source);
        let asm = build_application_semantic_model(&parsed);
        let component = &asm.components[0];
        let profile_id = FormId::for_owner(&component.id, "profileForm");
        let profile = asm.form(&profile_id).expect("profile Form");

        assert_eq!(asm.forms().len(), 2);
        assert_eq!(
            profile.id.as_str(),
            "module:src/UserProfile.tsx/component:user-profile/form:profileForm"
        );
        assert_eq!(profile.owner, SemanticOwner::entity(component.id.clone()));
        assert_eq!(
            profile.authored_field.as_str(),
            "module:src/UserProfile.tsx/component:user-profile/form-field:profileForm"
        );
        assert_eq!(profile.boundary, ExecutionBoundary::Client);
        assert_eq!(
            profile.provenance.span.start,
            source.find("@form()\n  profileForm").unwrap()
        );
        assert_eq!(
            asm.owner(profile.id.as_semantic_id()),
            Some(&SemanticOwner::entity(component.id.clone()))
        );
        assert_eq!(
            asm.semantic_type_of(profile.id.as_semantic_id()),
            Some(&SemanticType::Form)
        );
        assert!(asm
            .entity(profile.id.as_semantic_id())
            .is_some_and(|entity| entity.kind() == SemanticEntityKind::Form));
        assert!(asm
            .form_declaration_candidates()
            .iter()
            .all(|candidate| candidate.status == FormDeclarationStatus::Valid));
        assert!(validate_application_semantic_model(&asm).is_empty());
        let graph = build_semantic_graph(&asm);
        assert_eq!(graph.schema_version, SEMANTIC_GRAPH_SCHEMA_VERSION);
        assert!(graph
            .nodes
            .iter()
            .all(|node| !node.id.as_str().contains("/form:")));
    }

    #[test]
    fn keeps_form_names_component_scoped_and_multi_file_order_deterministic() {
        let login = r#"
@component("login-panel")
class LoginPanel {
  @form() credentials!: Form;
  render() { return <main />; }
}
"#;
        let signup = r#"
@component("signup-panel")
class SignupPanel {
  @form() credentials!: Form;
  render() { return <main />; }
}
"#;
        let first =
            CompilationUnit::parse_sources([("src/Signup.tsx", signup), ("src/Login.tsx", login)]);
        let second =
            CompilationUnit::parse_sources([("src/Login.tsx", login), ("src/Signup.tsx", signup)]);
        let first = build_application_semantic_model_for_unit(&first);
        let second = build_application_semantic_model_for_unit(&second);

        assert_eq!(first.forms, second.forms);
        assert_eq!(
            first.form_declaration_candidates(),
            second.form_declaration_candidates()
        );
        assert_eq!(first.forms.len(), 2);
        assert_ne!(first.forms.keys().next(), first.forms.keys().nth(1));
        assert!(first.forms.values().all(|form| form.name == "credentials"));
    }

    #[test]
    #[allow(clippy::too_many_lines)]
    fn retains_every_invalid_form_candidate_without_fabricating_identities() {
        let source = r#"
@form()
class ClassTarget {
  @form() orphan!: Form;
}

@form()
@component("profile")
class Profile {
  @form() good!: Form;
  @form() static staticForm!: Form;
  @form() initialized: Form = createForm();
  @form() notDeclarationOnly: Form;
  @form() missingType!;
  @form() wrong!: string;
  @form() nullable!: Form | null;
  @form() generic!: Form<any>;
  @form
  bare!: Form;
  @form("named") named!: Form;
  @form({}, true) many!: Form;
  @form() @slot() conflicting!: Form;
  @form() duplicate!: Form;
  @form() duplicate!: Form;
  @form() ["computed"]!: Form;
  @form() submit() {}
  parameter(@form() value: Form) {}
  @form() get current(): Form { return this.good; }
  @form() set current(value: Form) {}
  render() { return <main />; }
}
"#;
        let parsed = ezc_parser::parse_file("src/InvalidForms.tsx", source);
        let asm = build_application_semantic_model(&parsed);
        let candidates = asm.form_declaration_candidates();

        assert_eq!(asm.forms().len(), 1);
        assert_eq!(asm.forms()[0].name, "good");
        assert_eq!(candidates.len(), 22);
        assert_eq!(
            candidates
                .iter()
                .filter(|candidate| candidate.status == FormDeclarationStatus::Valid)
                .count(),
            1
        );
        assert!(candidates.iter().all(|candidate| {
            candidate.status == FormDeclarationStatus::Valid || !candidate.violations().is_empty()
        }));

        for name in [
            "staticForm",
            "initialized",
            "notDeclarationOnly",
            "missingType",
            "wrong",
            "nullable",
            "generic",
            "bare",
            "named",
            "many",
            "conflicting",
            "duplicate",
        ] {
            assert!(
                candidates.iter().any(|candidate| {
                    candidate.authored_name.as_deref() == Some(name) && candidate.form_id.is_some()
                }),
                "{name}"
            );
        }
        assert!(candidates
            .iter()
            .filter(|candidate| { candidate.authored_name.as_deref() == Some("duplicate") })
            .all(|candidate| candidate
                .violations()
                .contains(&FormDeclarationViolation::DuplicateName)));
        assert_eq!(
            candidates
                .iter()
                .filter(|candidate| candidate.authored_name.as_deref() == Some("duplicate"))
                .count(),
            2
        );
        assert!(candidates.iter().any(|candidate| {
            candidate.authored_name.as_deref() == Some("initialized")
                && candidate
                    .violations()
                    .contains(&FormDeclarationViolation::InitializedField)
        }));
        assert!(candidates.iter().any(|candidate| {
            candidate.authored_name.as_deref() == Some("bare")
                && candidate
                    .violations()
                    .contains(&FormDeclarationViolation::InvalidDecoratorInvocation)
        }));
        assert!(candidates.iter().any(|candidate| {
            candidate.authored_name.as_deref() == Some("conflicting")
                && candidate
                    .violations()
                    .contains(&FormDeclarationViolation::ConflictingSemanticDecorator)
        }));
        assert_eq!(
            candidates
                .iter()
                .find(|candidate| candidate.authored_name.as_deref() == Some("many"))
                .expect("many-argument candidate")
                .decorator_argument_provenance
                .len(),
            2
        );
        assert!(candidates
            .iter()
            .filter(|candidate| {
                matches!(
                    candidate.declaration_kind,
                    AuthoredDeclarationKind::Class
                        | AuthoredDeclarationKind::Method
                        | AuthoredDeclarationKind::Getter
                        | AuthoredDeclarationKind::Setter
                        | AuthoredDeclarationKind::Parameter
                ) || candidate
                    .violations()
                    .contains(&FormDeclarationViolation::InvalidOwner)
                    || candidate
                        .violations()
                        .contains(&FormDeclarationViolation::UnsupportedFieldName)
            })
            .all(|candidate| candidate.form_id.is_none()));
    }

    #[test]
    fn resolves_form_only_through_the_builtin_type_authority() {
        let cases = [
            r#"
import { Form } from "./user-form";
@component("profile")
class Profile {
  @form() profile!: Form;
  render() { return <main />; }
}
"#,
            r#"
interface Form {}
@component("profile")
class Profile {
  @form() profile!: Form;
  render() { return <main />; }
}
"#,
            r#"
class Form {}
@component("profile")
class Profile {
  @form() profile!: Form;
  render() { return <main />; }
}
"#,
            r#"
class CustomForm extends Form {}
@component("profile")
class Profile {
  @form() profile!: CustomForm;
  render() { return <main />; }
}
"#,
            r#"
type FormAlias = Form;
@component("profile")
class Profile {
  @form() profile!: FormAlias;
  render() { return <main />; }
}
"#,
        ];

        for (index, source) in cases.into_iter().enumerate() {
            let parsed = ezc_parser::parse_file(format!("src/InvalidType{index}.tsx"), source);
            let asm = build_application_semantic_model(&parsed);
            let candidates = asm.form_declaration_candidates();
            assert!(asm.forms().is_empty(), "case {index}");
            assert_eq!(candidates.len(), 1, "case {index}");
            assert!(
                candidates[0].violations().iter().any(|violation| matches!(
                    violation,
                    FormDeclarationViolation::InvalidType { .. }
                )),
                "case {index}"
            );
            assert!(candidates[0].form_id.is_some(), "case {index}");
        }
    }

    #[test]
    fn rejects_repeated_form_decorators_without_selecting_a_winner() {
        let parsed = ezc_parser::parse_file(
            "src/DuplicateDecorator.tsx",
            r#"
@component("profile")
class Profile {
  @form()
  @form()
  profile!: Form;
  render() { return <main />; }
}
"#,
        );
        let asm = build_application_semantic_model(&parsed);
        let candidates = asm.form_declaration_candidates();

        assert!(asm.forms().is_empty());
        assert_eq!(candidates.len(), 2);
        assert!(candidates.iter().all(|candidate| {
            candidate.form_id.is_some()
                && candidate
                    .violations()
                    .contains(&FormDeclarationViolation::DuplicateFormDecorator)
                && !candidate
                    .violations()
                    .contains(&FormDeclarationViolation::DuplicateName)
        }));
    }

    #[test]
    fn does_not_copy_inherited_form_declarations_into_derived_components() {
        let parsed = ezc_parser::parse_file(
            "src/Inheritance.tsx",
            r#"
@component("base-panel")
class BasePanel {
  @form() baseForm!: Form;
  render() { return <main />; }
}

@component("derived-panel")
class DerivedPanel extends BasePanel {
  render() { return <main />; }
}
"#,
        );
        let asm = build_application_semantic_model(&parsed);
        let derived = asm
            .components
            .iter()
            .find(|component| component.class_name == "DerivedPanel")
            .expect("derived component");

        assert_eq!(asm.forms().len(), 1);
        assert!(derived.form_declaration_candidates.is_empty());
        assert!(asm.forms().iter().all(|form| {
            form.owner
                .entity_id()
                .is_some_and(|owner| owner != &derived.id)
        }));
    }
}
