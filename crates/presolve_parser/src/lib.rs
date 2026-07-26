pub mod model;
mod oxc_adapter;

pub use model::{
    ParseDiagnostic, ParseLabel, ParseSeverity, ParsedArithmeticExpression,
    ParsedArithmeticExpressionKind, ParsedArithmeticOperator, ParsedClass, ParsedClassHeritage,
    ParsedComparisonOperator, ParsedComputedExpression, ParsedComputedExpressionKind,
    ParsedConstantExpression, ParsedConstantExpressionKind, ParsedDecorator, ParsedEffectBody,
    ParsedEffectExpression, ParsedEffectExpressionKind, ParsedEffectStatement,
    ParsedEffectStatementKind, ParsedEventHandler, ParsedExport, ParsedExportKind,
    ParsedExportSpecifier, ParsedFile, ParsedImport, ParsedImportSpecifier, ParsedInlineHandler,
    ParsedJsxAttribute, ParsedJsxAttributeValue, ParsedJsxChild, ParsedJsxConditional,
    ParsedJsxElement, ParsedJsxFragment, ParsedJsxList, ParsedJsxNode, ParsedLocalVariable,
    ParsedLogicalOperator, ParsedMethod, ParsedMethodCall, ParsedMethodParameter, ParsedProperty,
    ParsedSerializableValue, ParsedSourceAst, ParsedStateOperation, ParsedStateUpdate,
    ParsedStaticMemberDesignator, ParsedThisMemberDesignator, ParsedTypeAlias,
    ParsedTypeAnnotation, ParsedUnaryOperator, ParsedUnsupportedEffectStatementKind,
    ParsedValidationRuleArgument, ParsedValidationRuleArgumentKind, ParsedValidationRuleExpression,
    ParsedValidationRuleExpressionKind, SourceSpan,
};
pub use oxc_adapter::parse_file;

/// Validates one flags-free ECMAScript pattern using the compiler frontend's
/// pinned ECMAScript grammar authority.
#[must_use]
pub fn is_valid_ecmascript_pattern(pattern: &str) -> bool {
    let allocator = oxc_allocator::Allocator::default();
    oxc_regular_expression::LiteralParser::new(
        &allocator,
        pattern,
        None,
        oxc_regular_expression::Options {
            pattern_span_offset: 0,
            flags_span_offset: 0,
        },
    )
    .parse()
    .is_ok()
}

#[cfg(test)]
mod tests {
    use super::{
        parse_file, ParsedEffectStatementKind, ParsedJsxAttributeValue, ParsedSerializableValue,
        ParsedUnsupportedEffectStatementKind, ParsedValidationRuleArgumentKind,
        ParsedValidationRuleExpressionKind,
    };

    #[test]
    fn retains_validation_rule_calls_constants_and_direct_field_designators() {
        let source = r#"
@component("profile")
class Profile {
  @form() profile!: Form;

  @validate(min(1 + 2))
  @validate(pattern("^[a-z]+$"))
  @field(this.profile)
  name = "";

  @validate(equals(this.name))
  @field(this.profile)
  confirmation = "";
}
"#;
        let parsed = parse_file("src/Profile.tsx", source);
        let name = &parsed.classes[0].properties[1];
        let validations = name
            .decorators
            .iter()
            .filter(|decorator| decorator.name == "validate")
            .collect::<Vec<_>>();
        assert_eq!(validations.len(), 2);
        let expression = validations[0]
            .validation_rule_expression
            .as_ref()
            .expect("rule expression");
        let ParsedValidationRuleExpressionKind::Call { callee, arguments } = &expression.kind
        else {
            panic!("expected validation call");
        };
        assert_eq!(callee.as_deref(), Some("min"));
        assert!(matches!(
            arguments[0].kind,
            ParsedValidationRuleArgumentKind::Constant(_)
        ));
        let equals = parsed.classes[0].properties[2]
            .decorators
            .iter()
            .find(|decorator| decorator.name == "validate")
            .unwrap()
            .validation_rule_expression
            .as_ref()
            .unwrap();
        let ParsedValidationRuleExpressionKind::Call { arguments, .. } = &equals.kind else {
            panic!("expected equals call");
        };
        assert!(matches!(
            &arguments[0].kind,
            ParsedValidationRuleArgumentKind::ThisMember(designator)
                if designator.member == "name"
        ));
    }

    #[test]
    fn retains_a_source_faithful_general_estree_product() {
        let source = r#"
import type { CardProps } from "./types";
export const Card = <section aria-label="card">{1 + 2}</section>;
"#;
        let parsed = parse_file("src/Card.tsx", source);
        assert_eq!(parsed.syntax.source, source);
        assert_eq!(parsed.syntax.span.start, 0);
        assert_eq!(parsed.syntax.span.end, source.len());
        assert!(parsed.syntax.estree_json.contains("\"Program\""));
        assert!(parsed.syntax.estree_json.contains("\"ImportDeclaration\""));
        assert!(parsed.syntax.estree_json.contains("\"JSXElement\""));
        assert!(
            parsed.syntax.estree_json.contains("\"TSImportType\"")
                || parsed
                    .syntax
                    .estree_json
                    .contains("\"importKind\":\"type\"")
        );
    }

    #[test]
    fn retains_invalid_outer_validation_invocation_and_expression_shapes() {
        let source = r#"
@component("profile")
class Profile {
  @validate
  first = "";
  @validate()
  second = "";
  @validate(required(), email())
  third = "";
  @validate(schema.required())
  fourth = "";
}
"#;
        let parsed = parse_file("src/Profile.tsx", source);
        let decorators = parsed.classes[0]
            .properties
            .iter()
            .map(|property| &property.decorators[0])
            .collect::<Vec<_>>();
        assert!(!decorators[0].is_invoked);
        assert_eq!(decorators[1].argument_count, 0);
        assert_eq!(decorators[2].argument_count, 2);
        assert!(matches!(
            decorators[3]
                .validation_rule_expression
                .as_ref()
                .unwrap()
                .kind,
            ParsedValidationRuleExpressionKind::Call { callee: None, .. }
        ));
    }

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
    fn retains_source_faithful_import_binding_spans() {
        let source = "import { Component as FrameworkBase } from \"presolve\";";
        let parsed = parse_file("src/Card.tsx", source);
        let specifier = &parsed.imports[0].specifiers[0];
        assert_eq!(specifier.imported, "Component");
        assert_eq!(specifier.local, "FrameworkBase");
        assert_eq!(
            &source[specifier.local_span.start..specifier.local_span.end],
            "FrameworkBase"
        );
    }

    #[test]
    fn retains_direct_initializer_call_spans_without_intrinsic_classification() {
        let source = "class Counter { count = reactiveCell(0); plain = 1; }";
        let parsed = parse_file("src/Counter.tsx", source);
        let property = &parsed.classes[0].properties[0];
        let call = property
            .initializer_call
            .as_ref()
            .expect("direct calls are retained as syntax facts");
        assert_eq!(
            &source[call.callee_span.start..call.callee_span.end],
            "reactiveCell"
        );
        assert_eq!(&source[call.span.start..call.span.end], "reactiveCell(0)");
        assert!(parsed.classes[0].properties[1].initializer_call.is_none());
    }

    #[test]
    fn retains_inline_initializer_handler_updates_without_classifying_the_call() {
        let source = r#"
class Counter {
  increment = activate(() => { this.count += 1; unrelated(); });
  reset = activate(async function () { this.count = 0; });
}
"#;
        let parsed = parse_file("src/Counter.tsx", source);
        let increment = parsed.classes[0].properties[0]
            .initializer_call
            .as_ref()
            .and_then(|call| call.inline_handler.as_ref())
            .expect("inline arrow handler should remain a syntax fact");
        assert!(!increment.is_async);
        assert!(!increment.is_expression_body);
        assert_eq!(increment.state_updates.len(), 1);
        assert_eq!(increment.state_updates[0].field, "count");
        assert_eq!(increment.unsupported_statement_spans.len(), 1);
        assert_eq!(
            increment
                .effect_body
                .as_ref()
                .expect("inline block body")
                .statements
                .len(),
            2
        );
        assert_eq!(
            &source[increment.unsupported_statement_spans[0].start
                ..increment.unsupported_statement_spans[0].end],
            "unrelated();"
        );

        let reset = parsed.classes[0].properties[1]
            .initializer_call
            .as_ref()
            .and_then(|call| call.inline_handler.as_ref())
            .expect("inline function handler should remain a syntax fact");
        assert!(reset.is_async);
        assert_eq!(reset.state_updates.len(), 1);
        assert!(reset.unsupported_statement_spans.is_empty());
        assert!(reset.effect_body.is_some());
    }

    #[test]
    fn retains_inline_effect_cleanup_without_recognizing_the_callee() {
        let source = r#"
class Counter {
  sync = observe(() => {
    document.title = this.title;
    return () => { document.title = ""; };
  });
}
"#;
        let parsed = parse_file("src/Counter.tsx", source);
        let body = parsed.classes[0].properties[0]
            .initializer_call
            .as_ref()
            .and_then(|call| call.inline_handler.as_ref())
            .and_then(|handler| handler.effect_body.as_ref())
            .expect("general inline block body");
        assert_eq!(body.statements.len(), 1);
        let cleanup = body.cleanup.as_ref().expect("cleanup callback");
        assert!(!cleanup.is_async);
        assert_eq!(cleanup.body.statements.len(), 1);
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
        assert_eq!(
            editor.properties[4].decorators[0].argument.as_deref(),
            Some("profileForm")
        );
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
    fn retains_every_static_decorator_argument_without_admitting_its_semantics() {
        let parsed = parse_file(
            "src/NestedField.tsx",
            r#"@component("x-nested") class NestedField {
  @field("profile", "address.street") street = "";
  render() { return <div />; }
}"#,
        );
        let decorator = &parsed.classes[0].properties[0].decorators[0];
        assert_eq!(decorator.name, "field");
        assert_eq!(decorator.argument_count, 2);
        assert_eq!(
            decorator.arguments,
            vec![
                Some("profile".to_string()),
                Some("address.street".to_string())
            ]
        );
        assert_eq!(decorator.argument, Some("profile".to_string()));
    }

    #[test]
    fn retains_normalized_form_control_attribute_facts() {
        let source = r#"
@component("profile-editor")
class ProfileEditor {
  render() {
    return <main>
      <input type="radio" value="email" field={this.contact} />
      <input type={this.kind} field={this.contact} {...props} />
      <input field={this["contact"]} />
      <select multiple={this.multiple} field={this.tags} />
    </main>;
  }
}
"#;
        let parsed = parse_file("src/ProfileEditor.tsx", source);
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

        let radio = &elements[0];
        let value = radio
            .attributes
            .iter()
            .find(|attribute| attribute.name == "value")
            .expect("radio value");
        let field = radio
            .attributes
            .iter()
            .find(|attribute| attribute.name == "field")
            .expect("field binding");
        assert_eq!(
            value.constant_value,
            Some(ParsedSerializableValue::String("email".to_string()))
        );
        assert_eq!(
            field
                .this_member
                .as_ref()
                .map(|member| member.member.as_str()),
            Some("contact")
        );
        assert_eq!(
            &source[field.expression_span.unwrap().start..field.expression_span.unwrap().end],
            "this.contact"
        );
        assert!(elements[1]
            .attributes
            .iter()
            .any(|attribute| matches!(attribute.value, ParsedJsxAttributeValue::Spread(_))));
        assert!(elements[2].attributes[0].this_member.is_none());
        assert!(elements[3]
            .attributes
            .iter()
            .find(|attribute| attribute.name == "multiple")
            .expect("multiple")
            .this_member
            .is_some());
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

    #[test]
    fn retains_submit_decorator_designator_and_method_signature_facts() {
        let parsed = parse_file(
            "src/Profile.tsx",
            r#"
@component("profile")
class Profile {
  @action() @submit(this.profileForm) save(): void {}
  @submit invalid(value: string): string { return ""; }
  @action() @submit(this.profileForm) static saveStatic(): void {}
}
"#,
        );
        let methods = &parsed.classes[0].methods;
        let submit = methods[0]
            .decorators
            .iter()
            .find(|decorator| decorator.name == "submit")
            .expect("submit decorator");
        assert!(submit.is_invoked);
        assert_eq!(submit.argument_count, 1);
        assert_eq!(
            submit
                .this_member_argument
                .as_ref()
                .map(|value| value.member.as_str()),
            Some("profileForm")
        );
        assert_eq!(
            methods[0]
                .return_type_annotation
                .as_ref()
                .map(|annotation| annotation.text.as_str()),
            Some("void")
        );
        assert_eq!(methods[1].parameters.len(), 1);
        assert!(
            !methods[1]
                .decorators
                .iter()
                .find(|decorator| decorator.name == "submit")
                .expect("bare submit")
                .is_invoked
        );
        assert!(methods[2].is_static);
    }
}
