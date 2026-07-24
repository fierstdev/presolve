use std::path::Path;

use presolve_parser::{parse_file, ParsedFile};

/// The parsed source files that participate in one compiler invocation.
///
/// Files are stored in path order so every frontend consumer observes a
/// deterministic application input before module resolution is introduced.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompilationUnit {
    files: Vec<ParsedFile>,
}

impl CompilationUnit {
    #[must_use]
    pub fn from_parsed_files(mut files: Vec<ParsedFile>) -> Self {
        files.sort_by(|left, right| left.path.cmp(&right.path));
        Self { files }
    }

    #[must_use]
    pub fn parse_sources<I, P, S>(sources: I) -> Self
    where
        I: IntoIterator<Item = (P, S)>,
        P: AsRef<Path>,
        S: AsRef<str>,
    {
        let files = sources
            .into_iter()
            .map(|(path, source)| parse_file(path, source.as_ref()))
            .collect();

        Self::from_parsed_files(files)
    }

    #[must_use]
    pub fn files(&self) -> &[ParsedFile] {
        &self.files
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.files.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::CompilationUnit;

    #[test]
    fn sorts_parsed_sources_by_path() {
        let unit = CompilationUnit::parse_sources([
            ("src/Zeta.tsx", "class Zeta {}"),
            ("src/Alpha.tsx", "class Alpha {}"),
        ]);

        let paths = unit
            .files()
            .iter()
            .map(|file| file.path.to_string_lossy().into_owned())
            .collect::<Vec<_>>();

        assert_eq!(paths, vec!["src/Alpha.tsx", "src/Zeta.tsx"]);
    }
}
