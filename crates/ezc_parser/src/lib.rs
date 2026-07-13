pub mod model;
mod oxc_adapter;

pub use model::{
    ParseDiagnostic, ParseLabel, ParseSeverity, ParsedArithmeticExpression,
    ParsedArithmeticExpressionKind, ParsedArithmeticOperator, ParsedClass,
    ParsedComparisonOperator, ParsedComputedExpression, ParsedComputedExpressionKind,
    ParsedConstantExpression, ParsedConstantExpressionKind, ParsedDecorator, ParsedEffectBody,
    ParsedEffectExpression, ParsedEffectExpressionKind, ParsedEffectStatement,
    ParsedEffectStatementKind, ParsedEventHandler, ParsedExport, ParsedExportKind,
    ParsedExportSpecifier, ParsedFile, ParsedImport, ParsedImportSpecifier, ParsedJsxAttribute,
    ParsedJsxAttributeValue, ParsedJsxChild, ParsedJsxConditional, ParsedJsxElement,
    ParsedJsxFragment, ParsedJsxList, ParsedJsxNode, ParsedLocalVariable, ParsedLogicalOperator,
    ParsedMethod, ParsedMethodCall, ParsedMethodParameter, ParsedProperty, ParsedSerializableValue,
    ParsedStateOperation, ParsedStateUpdate, ParsedStaticMemberDesignator, ParsedTypeAlias,
    ParsedTypeAnnotation, ParsedUnaryOperator, ParsedUnsupportedEffectStatementKind, SourceSpan,
};
pub use oxc_adapter::parse_file;

#[cfg(test)]
mod tests {
    use super::{parse_file, ParsedEffectStatementKind, ParsedUnsupportedEffectStatementKind};

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
