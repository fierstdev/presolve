pub mod model;
mod oxc_adapter;

pub use model::{
    ParseDiagnostic, ParseLabel, ParseSeverity, ParsedClass, ParsedDecorator, ParsedEventHandler,
    ParsedExport, ParsedExportKind, ParsedExportSpecifier, ParsedFile, ParsedImport,
    ParsedImportSpecifier, ParsedJsxAttribute, ParsedJsxAttributeValue, ParsedJsxChild,
    ParsedJsxConditional, ParsedJsxElement, ParsedJsxFragment, ParsedJsxList, ParsedJsxNode,
    ParsedMethod, ParsedProperty, ParsedSerializableValue, ParsedStateOperation, ParsedStateUpdate,
    ParsedTypeAnnotation, SourceSpan,
};
pub use oxc_adapter::parse_file;
