pub mod model;
mod oxc_adapter;

pub use model::{
    ParseDiagnostic, ParseLabel, ParseSeverity, ParsedArithmeticExpression,
    ParsedArithmeticExpressionKind, ParsedArithmeticOperator, ParsedClass, ParsedClassHeritage,
    ParsedComparisonOperator, ParsedComputedExpression, ParsedComputedExpressionKind,
    ParsedConstantExpression, ParsedConstantExpressionKind, ParsedDecorator, ParsedEffectBody,
    ParsedEffectExpression, ParsedEffectExpressionKind, ParsedEffectStatement,
    ParsedEffectStatementKind, ParsedEventHandler, ParsedExport, ParsedExportKind,
    ParsedExportSpecifier, ParsedFile, ParsedImport, ParsedImportSpecifier, ParsedJsxAttribute,
    ParsedJsxAttributeValue, ParsedJsxChild, ParsedJsxConditional, ParsedJsxElement,
    ParsedJsxFragment, ParsedJsxList, ParsedJsxNode, ParsedLocalVariable, ParsedLogicalOperator,
    ParsedMethod, ParsedMethodCall, ParsedMethodParameter, ParsedProperty, ParsedSerializableValue,
    ParsedStateOperation, ParsedStateUpdate, ParsedStaticMemberDesignator,
    ParsedThisMemberDesignator, ParsedTypeAlias, ParsedTypeAnnotation, ParsedUnaryOperator,
    ParsedUnsupportedEffectStatementKind, SourceSpan,
};
pub use oxc_adapter::parse_file;

#[cfg(test)]
mod tests {
    use super::{
        parse_file, ParsedEffectStatementKind, ParsedSerializableValue,
        ParsedUnsupportedEffectStatementKind,
    };

    #[test]
    fn retains_source_faithful_class_heritage() {
        let source = "@component(\"x-child\") class Child extends Base.Component {}";
        let parsed = parse_file("src/Child.tsx", source);
        let heritage = parsed.classes[0].heritage.as_ref().expect("heritage");
        assert_eq!(heritage.base, "Base.Component");
        assert_eq!(
            &source[heritage.span.start..heritage.span.end],
            "Base.Component"
        );
    }

    #[test]
    fn retains_decorated_context_field_declaration_facts() {
        let source = r#"
@component("x-app-shell")
class AppShell extends Component {
  @context()
  locale: string = "en";
}
"#;
        let parsed = parse_file("src/AppShell.tsx", source);
        let property = &parsed.classes[0].properties[0];

        assert_eq!(property.name, "locale");
        assert_eq!(property.decorators[0].name, "context");
        assert_eq!(property.decorators[0].argument_count, 0);
        assert_eq!(property.type_annotation.as_ref().unwrap().text, "string");
        assert!(property.initializer_literal.is_some());
        assert!(!property.is_static);
        assert_eq!(property.span.start, source.find("@context()").unwrap());
        assert!(property.span.end > property.initializer_span.unwrap().end);
    }

    #[test]
    fn retains_slot_declaration_and_invalid_declaration_form_facts() {
        let source = r#"
@component("x-card")
class Card extends Component {
  @slot()
  children!: SlotContent;

  @slot("header")
  static invalid: string = "bad";

  @slot()
  outlet() {}

  attach(@slot() content: SlotContent) {}
}
"#;
        let parsed = parse_file("src/Card.tsx", source);
        assert!(parsed.diagnostics.is_empty(), "{:#?}", parsed.diagnostics);
        let class = &parsed.classes[0];
        let children = &class.properties[0];
        assert_eq!(children.decorators[0].name, "slot");
        assert_eq!(children.decorators[0].argument_count, 0);
        assert_eq!(
            children.type_annotation.as_ref().unwrap().text,
            "SlotContent"
        );
        assert!(children.is_definite_assignment);
        assert!(children.initializer.is_none());

        let invalid = &class.properties[1];
        assert_eq!(invalid.decorators[0].argument_count, 1);
        assert!(invalid.is_static);
        assert_eq!(invalid.initializer.as_deref(), Some("\"bad\""));

        assert_eq!(class.methods[0].decorators[0].name, "slot");
        assert_eq!(class.methods[1].parameters[0].decorators[0].name, "slot");
    }

    #[test]
    fn retains_normalized_form_declaration_targets_and_invocation_facts() {
        let source = r#"
@form()
class NotAFormField {}

@component("x-profile")
class Profile extends Component {
  @form()
  profile!: Form;

  @form()
  declare settings: Form;

  @form
  bare!: Form;

  @form("named")
  named!: Form;

  @form()
  @state
  conflicting!: Form;

  @form()
  get current(): Form { return this.profile; }

  @form()
  set current(value: Form) {}

  parameter(@form() value: Form) {}
}
"#;
        let parsed = parse_file("src/Profile.tsx", source);

        assert!(parsed.diagnostics.is_empty(), "{:#?}", parsed.diagnostics);
        assert!(parsed.classes[0].decorators[0].is_invoked);
        let profile = &parsed.classes[1];
        assert!(profile.properties[0].is_identifier_name);
        assert!(profile.properties[0].is_definite_assignment);
        assert!(!profile.properties[0].is_declare);
        assert!(profile.properties[1].is_declare);
        assert!(!profile.properties[2].decorators[0].is_invoked);
        assert_eq!(profile.properties[2].decorators[0].argument_count, 0);
        assert!(profile.properties[3].decorators[0].is_invoked);
        assert_eq!(profile.properties[3].decorators[0].argument_count, 1);
        assert_eq!(profile.properties[3].decorators[0].argument_spans.len(), 1);
        let named_argument = profile.properties[3].decorators[0].argument_spans[0];
        assert_eq!(
            &source[named_argument.start..named_argument.end],
            "\"named\""
        );
        assert_eq!(profile.properties[4].decorators.len(), 2);
        assert_eq!(profile.properties[4].decorators[1].name, "state");
        assert!(!profile.properties[4].decorators[1].is_invoked);
        assert!(profile.methods[0].is_getter);
        assert!(!profile.methods[0].is_setter);
        assert!(profile.methods[1].is_setter);
        assert_eq!(profile.methods[2].parameters[0].decorators[0].name, "form");
        assert_eq!(
            profile.properties[0].span.start,
            source.find("@form()\n  profile").unwrap()
        );
        assert!(parsed.local_type_bindings.contains(&"Profile".to_string()));
    }

    #[test]
    fn retains_module_local_type_namespace_bindings() {
        let parsed = parse_file(
            "src/types.ts",
            r#"
class ClassType {}
type AliasType = string;
interface InterfaceType {}
enum EnumType { Value }
import ImportEqualsType = require("./type");
"#,
        );

        assert_eq!(
            parsed.local_type_bindings,
            [
                "AliasType",
                "ClassType",
                "EnumType",
                "ImportEqualsType",
                "InterfaceType",
            ]
        );
    }

    #[test]
    fn retains_normalized_form_field_designators_targets_values_and_provenance() {
        let source = r#"
@field(this.profileForm)
class InvalidTarget {}

@component("profile-editor")
class ProfileEditor {
  @form() profileForm!: Form;
  @field(this.profileForm) displayName = "Austin";
  @field(this.profileForm) address = { city: "", postalCode: "" };
  @field
  bare = "";
  @field("profileForm") stringDesignator = "";
  @field(this.forms.profile) chained = "";
  @field(this.profileForm) ["computed"] = "";
  @field(this.profileForm) #privateName = "";
  @field(this.profileForm) method() {}
  parameter(@field(this.profileForm) value: string) {}
}
"#;
        let parsed = parse_file("src/ProfileEditor.tsx", source);
        let editor = &parsed.classes[1];
        let display = &editor.properties[1];

        assert_eq!(display.decorators[0].argument_count, 1);
        assert_eq!(
            display.decorators[0]
                .this_member_argument
                .as_ref()
                .map(|designator| designator.member.as_str()),
            Some("profileForm")
        );
        assert_eq!(
            display.initializer_literal,
            Some(ParsedSerializableValue::String("Austin".to_string()))
        );
        assert!(matches!(
            editor.properties[2].initializer_literal,
            Some(ParsedSerializableValue::Object(_))
        ));
        assert!(!editor.properties[3].decorators[0].is_invoked);
        assert!(editor.properties[4].decorators[0]
            .this_member_argument
            .is_none());
        assert!(editor.properties[5].decorators[0]
            .this_member_argument
            .is_none());
        assert!(!editor.properties[6].is_identifier_name);
        assert!(!editor.properties[7].is_identifier_name);
        assert_eq!(editor.methods[0].decorators[0].name, "field");
        assert_eq!(editor.methods[1].parameters[0].decorators[0].name, "field");
        assert_eq!(
            display.span.start,
            source.find("@field(this.profileForm) displayName").unwrap()
        );
    }

    #[test]
    fn retains_canonical_component_tag_names_and_exact_name_spans() {
        let source = r#"
@component("x-page")
class Page extends Component {
  render() { return <main><Card /><Registry.Card /></main>; }
}
"#;
        let parsed = parse_file("src/Page.tsx", source);
        let super::ParsedJsxNode::Element(root) = &parsed.classes[0].methods[0].jsx_roots[0] else {
            panic!("element root");
        };
        let elements = root
            .children
            .iter()
            .filter_map(|child| match child {
                super::ParsedJsxChild::Element(element) => Some(element),
                _ => None,
            })
            .collect::<Vec<_>>();

        assert_eq!(elements[0].name, "Card");
        assert_eq!(elements[1].name, "Registry.Card");
        assert_eq!(
            &source[elements[0].name_span.start..elements[0].name_span.end],
            "Card"
        );
        assert_eq!(
            &source[elements[1].name_span.start..elements[1].name_span.end],
            "Registry.Card"
        );
    }

    #[test]
    fn retains_static_member_provider_designators_and_value_expressions() {
        let parsed = parse_file(
            "src/AppShell.tsx",
            r#"
@component("x-app-shell")
class AppShell extends Component {
  @provide(AppShell.theme)
  providedTheme: string = this.theme ?? "light";
}
"#,
        );
        let property = &parsed.classes[0].properties[0];
        let decorator = &property.decorators[0];
        let designator = decorator.static_member_argument.as_ref().unwrap();

        assert_eq!(decorator.name, "provide");
        assert_eq!(decorator.argument_count, 1);
        assert_eq!(designator.object, "AppShell");
        assert_eq!(designator.member, "theme");
        assert!(property.initializer_expression.is_some());
    }

    #[test]
    fn retains_ordered_effect_statement_syntax_and_unsupported_forms() {
        let parsed = parse_file(
            "src/Effects.tsx",
            r#"
@component("x-effects")
class Effects extends Component {
  @effect()
  sync() {
    document.title = this.title;
    analytics.track("view", this.total + this.tax);
    return;
  }

  @effect()
  invalid() {
    const title = this.title;
    if (this.enabled) { analytics.track("enabled"); }
  }
}
"#,
        );
        let methods = &parsed.classes[0].methods;
        let body = methods[0].effect_body.as_ref().expect("effect body");
        assert_eq!(body.statements.len(), 3);
        assert!(matches!(
            body.statements[0].kind,
            ParsedEffectStatementKind::StaticMemberAssignment { .. }
        ));
        assert!(matches!(
            body.statements[1].kind,
            ParsedEffectStatementKind::CapabilityCall { .. }
        ));
        assert!(matches!(
            body.statements[2].kind,
            ParsedEffectStatementKind::EffectReturn { value: None }
        ));

        let invalid = methods[1].effect_body.as_ref().expect("effect body");
        assert!(matches!(
            invalid.statements[0].kind,
            ParsedEffectStatementKind::Unsupported(
                ParsedUnsupportedEffectStatementKind::LocalDeclaration
            )
        ));
        assert!(matches!(
            invalid.statements[1].kind,
            ParsedEffectStatementKind::Unsupported(ParsedUnsupportedEffectStatementKind::Branch)
        ));
    }
}
