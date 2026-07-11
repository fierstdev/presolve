use std::collections::BTreeSet;
use std::path::{Component, Path, PathBuf};

use ezc_parser::{ParsedExport, ParsedExportKind, ParsedImport, SourceSpan};

use crate::compilation_unit::CompilationUnit;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModuleGraph {
    pub modules: Vec<ModuleNode>,
    pub edges: Vec<ModuleEdge>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModuleNode {
    pub path: PathBuf,
    pub imports: Vec<ParsedImport>,
    pub exports: Vec<ParsedExport>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModuleEdge {
    pub source: PathBuf,
    pub specifier: String,
    pub target: ModuleTarget,
    pub kind: ModuleEdgeKind,
    pub span: SourceSpan,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ModuleEdgeKind {
    Import,
    NamedReExport,
    ExportAll,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ModuleTarget {
    Resolved(PathBuf),
    External,
    Unresolved,
}

#[must_use]
pub fn build_module_graph(unit: &CompilationUnit) -> ModuleGraph {
    let paths = unit
        .files()
        .iter()
        .map(|file| file.path.clone())
        .collect::<BTreeSet<_>>();
    let mut edges = Vec::new();

    for file in unit.files() {
        for import in &file.imports {
            edges.push(ModuleEdge {
                source: file.path.clone(),
                specifier: import.source.clone(),
                target: resolve_module_target(&file.path, &import.source, &paths),
                kind: ModuleEdgeKind::Import,
                span: import.span,
            });
        }

        for export in &file.exports {
            let Some(specifier) = &export.source else {
                continue;
            };
            edges.push(ModuleEdge {
                source: file.path.clone(),
                specifier: specifier.clone(),
                target: resolve_module_target(&file.path, specifier, &paths),
                kind: match export.kind {
                    ParsedExportKind::All => ModuleEdgeKind::ExportAll,
                    ParsedExportKind::Named | ParsedExportKind::Default => {
                        ModuleEdgeKind::NamedReExport
                    }
                },
                span: export.span,
            });
        }
    }

    edges.sort_by(|left, right| {
        (&left.source, left.span.start, left.kind, &left.specifier).cmp(&(
            &right.source,
            right.span.start,
            right.kind,
            &right.specifier,
        ))
    });

    ModuleGraph {
        modules: unit
            .files()
            .iter()
            .map(|file| ModuleNode {
                path: file.path.clone(),
                imports: file.imports.clone(),
                exports: file.exports.clone(),
            })
            .collect(),
        edges,
    }
}

fn resolve_module_target(
    source_path: &Path,
    specifier: &str,
    paths: &BTreeSet<PathBuf>,
) -> ModuleTarget {
    if !is_relative_specifier(specifier) {
        return ModuleTarget::External;
    }

    let joined_path = source_path
        .parent()
        .unwrap_or_else(|| Path::new(""))
        .join(specifier);
    let base = normalize_path(&joined_path);
    let mut candidates = vec![base.clone()];
    if base.extension().is_none() {
        for extension in ["ts", "tsx", "js", "jsx"] {
            candidates.push(base.with_extension(extension));
        }
        for extension in ["ts", "tsx", "js", "jsx"] {
            candidates.push(base.join("index").with_extension(extension));
        }
    }

    candidates
        .into_iter()
        .find(|candidate| paths.contains(candidate))
        .map_or(ModuleTarget::Unresolved, ModuleTarget::Resolved)
}

fn is_relative_specifier(specifier: &str) -> bool {
    matches!(specifier, "." | "..") || specifier.starts_with("./") || specifier.starts_with("../")
}

fn normalize_path(path: &Path) -> PathBuf {
    let mut normalized = PathBuf::new();

    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                normalized.pop();
            }
            Component::Normal(segment) => normalized.push(segment),
            Component::RootDir | Component::Prefix(_) => normalized.push(component.as_os_str()),
        }
    }

    normalized
}

#[cfg(test)]
mod tests {
    use super::{build_module_graph, ModuleEdgeKind, ModuleTarget};
    use crate::CompilationUnit;

    #[test]
    fn derives_module_edges_from_imports_and_re_exports() {
        let unit = CompilationUnit::parse_sources([
            (
                "src/app/App.tsx",
                r#"
import { Card as AppCard } from "../ui/Card";
import "edgezero-runtime";
export { Status } from "./status";
export * from "./shared";
export default class App {}
"#,
            ),
            ("src/app/status.ts", "export const Status = 200;"),
            (
                "src/app/shared/index.ts",
                "export const label = \"shared\";",
            ),
            ("src/ui/Card.tsx", "export class Card {}"),
        ]);

        let graph = build_module_graph(&unit);
        let app = graph.modules.first().expect("expected app module");

        assert_eq!(graph.modules.len(), 4);
        assert_eq!(app.path.to_string_lossy(), "src/app/App.tsx");
        assert_eq!(app.imports[0].specifiers[0].imported, "Card");
        assert_eq!(app.imports[0].specifiers[0].local, "AppCard");
        assert_eq!(app.exports[0].specifiers[0].exported, "Status");
        assert_eq!(app.exports[2].specifiers[0].exported, "default");

        assert_eq!(graph.edges.len(), 4);
        assert_eq!(graph.edges[0].kind, ModuleEdgeKind::Import);
        assert_eq!(
            graph.edges[0].target,
            ModuleTarget::Resolved("src/ui/Card.tsx".into())
        );
        assert_eq!(graph.edges[1].target, ModuleTarget::External);
        assert_eq!(graph.edges[2].kind, ModuleEdgeKind::NamedReExport);
        assert_eq!(
            graph.edges[2].target,
            ModuleTarget::Resolved("src/app/status.ts".into())
        );
        assert_eq!(graph.edges[3].kind, ModuleEdgeKind::ExportAll);
        assert_eq!(
            graph.edges[3].target,
            ModuleTarget::Resolved("src/app/shared/index.ts".into())
        );
    }

    #[test]
    fn distinguishes_external_and_unresolved_relative_specifiers() {
        let unit = CompilationUnit::parse_sources([(
            "src/App.tsx",
            r#"
import { packageValue } from "package-value";
import { missing } from "./missing";
"#,
        )]);

        let graph = build_module_graph(&unit);

        assert_eq!(graph.edges[0].target, ModuleTarget::External);
        assert_eq!(graph.edges[1].target, ModuleTarget::Unresolved);
    }
}
