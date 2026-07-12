pub mod model;
mod oxc_adapter;

pub use model::{
    ParseDiagnostic, ParseLabel, ParseSeverity, ParsedArithmeticExpression,
    ParsedArithmeticExpressionKind, ParsedArithmeticOperator, ParsedClass,
    ParsedComparisonOperator, ParsedConstantExpression, ParsedConstantExpressionKind,
    ParsedDecorator, ParsedEventHandler, ParsedExport, ParsedExportKind, ParsedExportSpecifier,
    ParsedFile, ParsedImport, ParsedImportSpecifier, ParsedJsxAttribute, ParsedJsxAttributeValue,
    ParsedJsxChild, ParsedJsxConditional, ParsedJsxElement, ParsedJsxFragment, ParsedJsxList,
    ParsedJsxNode, ParsedLocalVariable, ParsedLogicalOperator, ParsedMethod, ParsedMethodParameter,
    ParsedProperty, ParsedSerializableValue, ParsedStateOperation, ParsedStateUpdate,
    ParsedTypeAnnotation, ParsedUnaryOperator, SourceSpan,
};
pub use oxc_adapter::parse_file;
