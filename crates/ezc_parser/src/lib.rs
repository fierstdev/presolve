pub mod model;
mod oxc_adapter;

pub use model::{
    ParseDiagnostic, ParseLabel, ParseSeverity, ParsedClass, ParsedDecorator, ParsedEventHandler,
    ParsedFile, ParsedJsxAttribute, ParsedJsxAttributeValue, ParsedJsxChild, ParsedJsxConditional,
    ParsedJsxElement, ParsedJsxFragment, ParsedJsxList, ParsedJsxNode, ParsedMethod,
    ParsedProperty, ParsedSerializableValue, ParsedStateOperation, ParsedStateUpdate, SourceSpan,
};
pub use oxc_adapter::parse_file;
