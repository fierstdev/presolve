use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use crate::compilation_unit::CompilationUnit;
use crate::module_graph::{ModuleEdgeKind, ModuleGraph, ModuleTarget};
use crate::symbol_table::{ModuleSymbol, SymbolTable};

/// Resolved local exports and relative-module imports.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BindingTable {
    pub modules: Vec<ModuleBindingTable>,
    pub diagnostics: Vec<BindingDiagnostic>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModuleBindingTable {
    pub path: PathBuf,
    pub exports: BTreeMap<String, ExportBinding>,
    pub imports: BTreeMap<String, ImportBinding>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExportBinding {
    pub exported_name: String,
    pub local_name: String,
    pub symbol: ModuleSymbol,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImportBinding {
    pub local_name: String,
    pub imported_name: String,
    pub source_module: PathBuf,
    pub target: ImportBindingTarget,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ImportBindingTarget {
    Symbol(ModuleSymbol),
    Namespace {
        module: PathBuf,
        exports: BTreeMap<String, ModuleSymbol>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BindingDiagnostic {
    pub code: String,
    pub module: PathBuf,
    pub name: String,
    pub message: String,
}

impl BindingTable {
    #[must_use]
    pub fn module(&self, path: impl AsRef<Path>) -> Option<&ModuleBindingTable> {
        self.modules
            .iter()
            .find(|module| module.path == path.as_ref())
    }

    #[must_use]
    pub fn resolve_import(
        &self,
        module_path: impl AsRef<Path>,
        local_name: &str,
    ) -> Option<&ImportBinding> {
        self.module(module_path)
            .and_then(|module| module.imports.get(local_name))
    }
}

#[must_use]
pub fn build_binding_table(
    unit: &CompilationUnit,
    symbols: &SymbolTable,
    modules: &ModuleGraph,
) -> BindingTable {
    let mut tables = unit
        .files()
        .iter()
        .map(|file| ModuleBindingTable {
            path: file.path.clone(),
            exports: BTreeMap::new(),
            imports: BTreeMap::new(),
        })
        .collect::<Vec<_>>();
    let mut diagnostics = Vec::new();

    collect_local_exports(unit, symbols, &mut tables, &mut diagnostics);

    let exports_by_path = tables
        .iter()
        .map(|table| (table.path.clone(), table.exports.clone()))
        .collect::<BTreeMap<_, _>>();

    resolve_relative_imports(
        unit,
        modules,
        &exports_by_path,
        &mut tables,
        &mut diagnostics,
    );

    BindingTable {
        modules: tables,
        diagnostics,
    }
}

fn collect_local_exports(
    unit: &CompilationUnit,
    symbols: &SymbolTable,
    tables: &mut [ModuleBindingTable],
    diagnostics: &mut Vec<BindingDiagnostic>,
) {
    for (file, table) in unit.files().iter().zip(tables.iter_mut()) {
        let Some(module_symbols) = symbols.module(&file.path) else {
            continue;
        };

        for export in file.exports.iter().filter(|export| export.source.is_none()) {
            for specifier in &export.specifiers {
                let Some(local_name) = &specifier.local else {
                    diagnostics.push(BindingDiagnostic {
                        code: "EZBIND1001".to_string(),
                        module: file.path.clone(),
                        name: specifier.exported.clone(),
                        message: format!(
                            "export `{}` has no local declaration to bind",
                            specifier.exported
                        ),
                    });
                    continue;
                };
                let Some(symbol) = module_symbols.symbols.get(local_name) else {
                    diagnostics.push(BindingDiagnostic {
                        code: "EZBIND1001".to_string(),
                        module: file.path.clone(),
                        name: specifier.exported.clone(),
                        message: format!(
                            "export `{}` refers to unsupported or missing local symbol `{local_name}`",
                            specifier.exported
                        ),
                    });
                    continue;
                };

                insert_export(
                    table,
                    diagnostics,
                    ExportBinding {
                        exported_name: specifier.exported.clone(),
                        local_name: local_name.clone(),
                        symbol: symbol.clone(),
                    },
                );
            }
        }
    }
}

fn resolve_relative_imports(
    unit: &CompilationUnit,
    modules: &ModuleGraph,
    exports_by_path: &BTreeMap<PathBuf, BTreeMap<String, ExportBinding>>,
    tables: &mut [ModuleBindingTable],
    diagnostics: &mut Vec<BindingDiagnostic>,
) {
    for (file, table) in unit.files().iter().zip(tables.iter_mut()) {
        for import in &file.imports {
            let target = module_target(modules, &file.path, import.span);
            let ModuleTarget::Resolved(target_path) = target else {
                if matches!(target, ModuleTarget::Unresolved) {
                    diagnostics.push(BindingDiagnostic {
                        code: "EZBIND1002".to_string(),
                        module: file.path.clone(),
                        name: import.source.clone(),
                        message: format!(
                            "relative import `{}` does not resolve to a module",
                            import.source
                        ),
                    });
                }
                continue;
            };
            let target_exports = exports_by_path
                .get(&target_path)
                .cloned()
                .unwrap_or_default();

            for specifier in &import.specifiers {
                let binding = resolve_import_binding(specifier, &target_path, &target_exports);

                let Some(binding) = binding else {
                    diagnostics.push(BindingDiagnostic {
                        code: "EZBIND1003".to_string(),
                        module: file.path.clone(),
                        name: specifier.local.clone(),
                        message: format!(
                            "import `{}` is not exported by `{}`",
                            specifier.imported,
                            target_path.display()
                        ),
                    });
                    continue;
                };

                insert_import(table, diagnostics, binding);
            }
        }
    }
}

fn resolve_import_binding(
    specifier: &ezc_parser::ParsedImportSpecifier,
    target_path: &Path,
    target_exports: &BTreeMap<String, ExportBinding>,
) -> Option<ImportBinding> {
    if specifier.imported == "*" {
        return Some(ImportBinding {
            local_name: specifier.local.clone(),
            imported_name: specifier.imported.clone(),
            source_module: target_path.to_path_buf(),
            target: ImportBindingTarget::Namespace {
                module: target_path.to_path_buf(),
                exports: target_exports
                    .iter()
                    .map(|(name, export)| (name.clone(), export.symbol.clone()))
                    .collect(),
            },
        });
    }

    target_exports
        .get(&specifier.imported)
        .map(|export| ImportBinding {
            local_name: specifier.local.clone(),
            imported_name: specifier.imported.clone(),
            source_module: target_path.to_path_buf(),
            target: ImportBindingTarget::Symbol(export.symbol.clone()),
        })
}

fn module_target(
    modules: &ModuleGraph,
    source: &Path,
    span: ezc_parser::SourceSpan,
) -> ModuleTarget {
    modules
        .edges
        .iter()
        .find(|edge| {
            edge.kind == ModuleEdgeKind::Import && edge.source == source && edge.span == span
        })
        .map_or(ModuleTarget::Unresolved, |edge| edge.target.clone())
}

fn insert_export(
    table: &mut ModuleBindingTable,
    diagnostics: &mut Vec<BindingDiagnostic>,
    binding: ExportBinding,
) {
    if table.exports.contains_key(&binding.exported_name) {
        diagnostics.push(BindingDiagnostic {
            code: "EZBIND1004".to_string(),
            module: table.path.clone(),
            name: binding.exported_name.clone(),
            message: format!("duplicate export binding `{}`", binding.exported_name),
        });
        return;
    }

    table.exports.insert(binding.exported_name.clone(), binding);
}

fn insert_import(
    table: &mut ModuleBindingTable,
    diagnostics: &mut Vec<BindingDiagnostic>,
    binding: ImportBinding,
) {
    if table.imports.contains_key(&binding.local_name) {
        diagnostics.push(BindingDiagnostic {
            code: "EZBIND1005".to_string(),
            module: table.path.clone(),
            name: binding.local_name.clone(),
            message: format!("duplicate import binding `{}`", binding.local_name),
        });
        return;
    }

    table.imports.insert(binding.local_name.clone(), binding);
}

#[cfg(test)]
mod tests {
    use super::{build_binding_table, ImportBindingTarget};
    use crate::{build_module_graph, build_symbol_table, CompilationUnit, SemanticId};

    #[test]
    fn resolves_relative_named_default_and_namespace_imports() {
        let unit = CompilationUnit::parse_sources([
            (
                "src/app/App.tsx",
                r#"
import { Widget as CounterWidget } from "../ui/Counter";
import StatusCard from "../ui/Status";
import * as counter from "../ui/Counter";
import { packageValue } from "edgezero-runtime";
"#,
            ),
            (
                "src/ui/Counter.tsx",
                r#"
@component("x-counter")
export class Counter extends Component {
  render() {
    return <div>Counter</div>;
  }
}

export { Counter as Widget };
"#,
            ),
            (
                "src/ui/Status.tsx",
                r#"
@component("x-status")
export default class Status extends Component {
  render() {
    return <div>Status</div>;
  }
}
"#,
            ),
        ]);
        let symbols = build_symbol_table(&unit);
        let modules = build_module_graph(&unit);
        let bindings = build_binding_table(&unit, &symbols, &modules);
        let app = bindings.module("src/app/App.tsx").expect("app module");

        assert!(bindings.diagnostics.is_empty());
        assert_eq!(app.imports.len(), 3);
        assert_eq!(app.exports.len(), 0);
        assert_eq!(
            app.imports["CounterWidget"].target,
            ImportBindingTarget::Symbol(
                bindings
                    .module("src/ui/Counter.tsx")
                    .expect("counter module")
                    .exports["Widget"]
                    .symbol
                    .clone()
            )
        );
        assert_eq!(
            app.imports["StatusCard"].target,
            ImportBindingTarget::Symbol(
                bindings
                    .module("src/ui/Status.tsx")
                    .expect("status module")
                    .exports["default"]
                    .symbol
                    .clone()
            )
        );

        let ImportBindingTarget::Namespace { module, exports } = &app.imports["counter"].target
        else {
            panic!("expected namespace binding");
        };
        assert_eq!(module, &std::path::PathBuf::from("src/ui/Counter.tsx"));
        assert_eq!(
            exports["Widget"].id,
            SemanticId::component(Some("x-counter"), "Counter")
        );
        assert!(bindings
            .resolve_import("src/app/App.tsx", "packageValue")
            .is_none());
    }

    #[test]
    fn reports_unresolved_relative_and_missing_export_bindings() {
        let unit = CompilationUnit::parse_sources([
            (
                "src/App.tsx",
                r#"
import { Missing } from "./Counter";
import { MissingModule } from "./missing";
"#,
            ),
            (
                "src/Counter.tsx",
                r#"
@component("x-counter")
export class Counter extends Component {
  render() {
    return <div>Counter</div>;
  }
}
"#,
            ),
        ]);
        let symbols = build_symbol_table(&unit);
        let modules = build_module_graph(&unit);
        let bindings = build_binding_table(&unit, &symbols, &modules);

        let codes = bindings
            .diagnostics
            .iter()
            .map(|diagnostic| diagnostic.code.as_str())
            .collect::<Vec<_>>();

        assert_eq!(codes, vec!["EZBIND1003", "EZBIND1002"]);
    }
}
