use std::path::{Path, PathBuf};

use ezc_parser::SourceSpan;

/// Compiler source location for a semantic entity or relationship.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceProvenance {
    pub path: PathBuf,
    pub span: SourceSpan,
}

impl SourceProvenance {
    #[must_use]
    pub fn new(path: impl AsRef<Path>, span: SourceSpan) -> Self {
        Self {
            path: path.as_ref().to_path_buf(),
            span,
        }
    }
}
