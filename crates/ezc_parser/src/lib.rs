pub mod model;
mod oxc_adapter;

pub use model::{
    ParseDiagnostic, ParseLabel, ParseSeverity, ParsedClass, ParsedDecorator, ParsedFile,
    ParsedJsxChild, ParsedJsxElement, ParsedMethod, ParsedProperty, ParsedStateOperation,
    ParsedStateUpdate, SourceSpan,
};
pub use oxc_adapter::parse_file;
