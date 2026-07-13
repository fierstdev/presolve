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
    ParsedStateOperation, ParsedStateUpdate, ParsedTypeAlias, ParsedTypeAnnotation,
    ParsedUnaryOperator, ParsedUnsupportedEffectStatementKind, SourceSpan,
};
pub use oxc_adapter::parse_file;

#[cfg(test)]
mod tests {
    use super::{parse_file, ParsedEffectStatementKind, ParsedUnsupportedEffectStatementKind};

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
