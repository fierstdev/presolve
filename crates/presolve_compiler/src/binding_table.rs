use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use crate::compilation_unit::CompilationUnit;
use crate::module_graph::{ModuleEdgeKind, ModuleGraph, ModuleTarget};
use crate::semantic_package::{
    SemanticPackageFormSubmission, SemanticPackageKind, SemanticPackageOpaqueTerminal,
    SemanticPackagePureOperation, SemanticPackageResolutionTable, SemanticPackageResourceEndpoint,
    SemanticPackageRouteLoader, SemanticPackageServerAction,
};
use crate::symbol_table::{ModuleSymbol, SymbolKind, SymbolTable};

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
    SemanticPackage {
        package: String,
        version: String,
        integrity: String,
        export: String,
        kind: SemanticPackageKind,
        type_signature: String,
        runtime_module: String,
        resume_policy: String,
        pure_operation: Option<SemanticPackagePureOperation>,
        resource_endpoint: Option<SemanticPackageResourceEndpoint>,
        route_loader: Option<SemanticPackageRouteLoader>,
        server_action: Option<SemanticPackageServerAction>,
        form_submission: Option<SemanticPackageFormSubmission>,
        opaque_terminal: Option<SemanticPackageOpaqueTerminal>,
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
    build_binding_table_with_packages(
        unit,
        symbols,
        modules,
        &SemanticPackageResolutionTable::default(),
    )
}

#[must_use]
pub fn build_binding_table_with_packages(
    unit: &CompilationUnit,
    symbols: &SymbolTable,
    modules: &ModuleGraph,
    packages: &SemanticPackageResolutionTable,
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
    resolve_relative_reexports(unit, modules, &mut tables, &mut diagnostics);
    collect_identity_collisions(symbols, &mut diagnostics);

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
        packages,
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
                        code: "PSBIND1001".to_string(),
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
                        code: "PSBIND1001".to_string(),
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
    packages: &SemanticPackageResolutionTable,
) {
    for (file, table) in unit.files().iter().zip(tables.iter_mut()) {
        for import in &file.imports {
            let target = module_target(modules, &file.path, import.span, ModuleEdgeKind::Import);
            if matches!(target, ModuleTarget::External) {
                resolve_semantic_package_import(import, packages, table, diagnostics, &file.path);
                continue;
            }
            let ModuleTarget::Resolved(target_path) = target else {
                if matches!(target, ModuleTarget::Unresolved) {
                    diagnostics.push(BindingDiagnostic {
                        code: "PSBIND1002".to_string(),
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
                        code: "PSBIND1003".to_string(),
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

fn resolve_semantic_package_import(
    import: &presolve_parser::ParsedImport,
    packages: &SemanticPackageResolutionTable,
    table: &mut ModuleBindingTable,
    diagnostics: &mut Vec<BindingDiagnostic>,
    module: &Path,
) {
    // `presolve` is the public declaration-only authoring vocabulary. Its
    // imports select compiler intrinsics that are already recognized by the
    // parser/component model; unlike third-party packages, they have no
    // runtime module, integrity coordinate, or semantic capability to bind.
    if import.source == "presolve" {
        return;
    }
    let Some(contract) = packages.contract(&import.source) else {
        diagnostics.push(BindingDiagnostic {
            code: "PSBIND1009".into(),
            module: module.to_path_buf(),
            name: import.source.clone(),
            message: format!(
                "external import `{}` has no semantic package contract",
                import.source
            ),
        });
        return;
    };
    for specifier in &import.specifiers {
        let Some(export) = contract.exports.get(&specifier.imported) else {
            diagnostics.push(BindingDiagnostic {
                code: "PSBIND1010".into(),
                module: module.to_path_buf(),
                name: specifier.imported.clone(),
                message: format!(
                    "semantic package `{}` does not declare export `{}`",
                    import.source, specifier.imported
                ),
            });
            continue;
        };
        insert_import(
            table,
            diagnostics,
            ImportBinding {
                local_name: specifier.local.clone(),
                imported_name: specifier.imported.clone(),
                source_module: PathBuf::from(&import.source),
                target: ImportBindingTarget::SemanticPackage {
                    package: contract.package.clone(),
                    version: contract.version.clone(),
                    integrity: contract.integrity.clone(),
                    export: specifier.imported.clone(),
                    kind: export.kind.clone(),
                    type_signature: export.type_signature.clone(),
                    runtime_module: export.runtime_module.clone(),
                    resume_policy: export.resume_policy.clone(),
                    pure_operation: export.pure_operation,
                    resource_endpoint: export.resource_endpoint.clone(),
                    route_loader: export.route_loader.clone(),
                    server_action: export.server_action.clone(),
                    form_submission: export.form_submission.clone(),
                    opaque_terminal: export.opaque_terminal.clone(),
                },
            },
        );
    }
}

fn resolve_relative_reexports(
    unit: &CompilationUnit,
    modules: &ModuleGraph,
    tables: &mut [ModuleBindingTable],
    diagnostics: &mut Vec<BindingDiagnostic>,
) {
    for _ in 0..unit.files().len() {
        let exports_by_path = tables
            .iter()
            .map(|table| (table.path.clone(), table.exports.clone()))
            .collect::<BTreeMap<_, _>>();
        let mut changed = false;

        for (file, table) in unit.files().iter().zip(tables.iter_mut()) {
            for export in file.exports.iter().filter(|export| export.source.is_some()) {
                let edge_kind = match export.kind {
                    presolve_parser::ParsedExportKind::Named => ModuleEdgeKind::NamedReExport,
                    presolve_parser::ParsedExportKind::All => ModuleEdgeKind::ExportAll,
                    presolve_parser::ParsedExportKind::Default => continue,
                };
                let target = module_target(modules, &file.path, export.span, edge_kind);
                let ModuleTarget::Resolved(target_path) = target else {
                    if matches!(target, ModuleTarget::Unresolved) {
                        push_diagnostic_once(
                            diagnostics,
                            BindingDiagnostic {
                                code: "PSBIND1006".to_string(),
                                module: file.path.clone(),
                                name: export.source.clone().unwrap_or_default(),
                                message: "relative re-export does not resolve to a module"
                                    .to_string(),
                            },
                        );
                    }
                    continue;
                };
                let target_exports = exports_by_path
                    .get(&target_path)
                    .cloned()
                    .unwrap_or_default();

                match export.kind {
                    presolve_parser::ParsedExportKind::Named => {
                        for specifier in &export.specifiers {
                            let Some(local_name) = &specifier.local else {
                                continue;
                            };
                            let Some(target) = target_exports.get(local_name) else {
                                continue;
                            };
                            changed |= insert_reexport(
                                table,
                                ExportBinding {
                                    exported_name: specifier.exported.clone(),
                                    local_name: local_name.clone(),
                                    symbol: target.symbol.clone(),
                                },
                            );
                        }
                    }
                    presolve_parser::ParsedExportKind::All if export.specifiers.is_empty() => {
                        for (name, target) in target_exports {
                            if name != "default" {
                                changed |= insert_reexport(
                                    table,
                                    ExportBinding {
                                        exported_name: name.clone(),
                                        local_name: target.local_name,
                                        symbol: target.symbol,
                                    },
                                );
                            }
                        }
                    }
                    presolve_parser::ParsedExportKind::All => {
                        push_diagnostic_once(
                            diagnostics,
                            BindingDiagnostic {
                                code: "PSBIND1007".to_string(),
                                module: file.path.clone(),
                                name: export.source.clone().unwrap_or_default(),
                                message: "namespace re-export bindings are not supported yet"
                                    .to_string(),
                            },
                        );
                    }
                    presolve_parser::ParsedExportKind::Default => {}
                }
            }
        }

        if !changed {
            break;
        }
    }
}

fn insert_reexport(table: &mut ModuleBindingTable, binding: ExportBinding) -> bool {
    if table.exports.contains_key(&binding.exported_name) {
        return false;
    }

    table.exports.insert(binding.exported_name.clone(), binding);
    true
}

fn collect_identity_collisions(symbols: &SymbolTable, diagnostics: &mut Vec<BindingDiagnostic>) {
    let mut paths_by_id = BTreeMap::<String, Vec<PathBuf>>::new();

    for module in &symbols.modules {
        for symbol in module
            .symbols
            .values()
            .filter(|symbol| symbol.kind == SymbolKind::Component)
        {
            paths_by_id
                .entry(symbol.id.as_str().to_string())
                .or_default()
                .push(module.path.clone());
        }
    }

    for paths in paths_by_id.values_mut() {
        paths.sort();
        paths.dedup();
    }

    for (id, paths) in paths_by_id.into_iter().filter(|(_, paths)| paths.len() > 1) {
        diagnostics.push(BindingDiagnostic {
            code: "PSBIND1008".to_string(),
            module: paths[0].clone(),
            name: id.clone(),
            message: format!(
                "semantic identity `{id}` collides across modules: {}",
                paths
                    .iter()
                    .map(|path| path.display().to_string())
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
        });
    }
}

fn push_diagnostic_once(diagnostics: &mut Vec<BindingDiagnostic>, diagnostic: BindingDiagnostic) {
    if diagnostics.iter().any(|existing| existing == &diagnostic) {
        return;
    }

    diagnostics.push(diagnostic);
}

fn resolve_import_binding(
    specifier: &presolve_parser::ParsedImportSpecifier,
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
    span: presolve_parser::SourceSpan,
    kind: ModuleEdgeKind,
) -> ModuleTarget {
    modules
        .edges
        .iter()
        .find(|edge| edge.kind == kind && edge.source == source && edge.span == span)
        .map_or(ModuleTarget::Unresolved, |edge| edge.target.clone())
}

fn insert_export(
    table: &mut ModuleBindingTable,
    diagnostics: &mut Vec<BindingDiagnostic>,
    binding: ExportBinding,
) {
    if table.exports.contains_key(&binding.exported_name) {
        diagnostics.push(BindingDiagnostic {
            code: "PSBIND1004".to_string(),
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
            code: "PSBIND1005".to_string(),
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
    use super::{build_binding_table, build_binding_table_with_packages, ImportBindingTarget};
    use crate::semantic_package::{
        parse_semantic_package_contract, SemanticPackageResolutionTable,
    };
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
import { packageValue } from "presolve-runtime";
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

        assert_eq!(
            bindings
                .diagnostics
                .iter()
                .map(|diagnostic| diagnostic.code.as_str())
                .collect::<Vec<_>>(),
            vec!["PSBIND1009"]
        );
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
            SemanticId::component_in_module("src/ui/Counter.tsx", Some("x-counter"), "Counter")
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

        assert_eq!(codes, vec!["PSBIND1003", "PSBIND1002"]);
    }

    #[test]
    fn resolves_named_and_export_all_reexport_chains() {
        let unit = CompilationUnit::parse_sources([
            (
                "src/App.tsx",
                r#"
import { PrimaryButton } from "./index";
"#,
            ),
            (
                "src/base.tsx",
                r#"
@component("x-button")
export class Button extends Component {
  render() {
    return <button>Button</button>;
  }
}
"#,
            ),
            (
                "src/bridge.ts",
                r#"
export { Button as PrimaryButton } from "./base";
"#,
            ),
            (
                "src/index.ts",
                r#"
export * from "./bridge";
"#,
            ),
        ]);
        let symbols = build_symbol_table(&unit);
        let modules = build_module_graph(&unit);
        let bindings = build_binding_table(&unit, &symbols, &modules);

        assert!(bindings.diagnostics.is_empty());
        assert_eq!(
            bindings
                .resolve_import("src/App.tsx", "PrimaryButton")
                .expect("resolved re-export")
                .target,
            ImportBindingTarget::Symbol(
                bindings
                    .module("src/index.ts")
                    .expect("index module")
                    .exports["PrimaryButton"]
                    .symbol
                    .clone()
            )
        );
    }

    #[test]
    fn module_qualified_component_ids_do_not_collide_across_modules() {
        let unit = CompilationUnit::parse_sources([
            (
                "src/First.tsx",
                r#"
@component("x-duplicate")
class First extends Component {
  render() {
    return <div>First</div>;
  }
}
"#,
            ),
            (
                "src/Second.tsx",
                r#"
@component("x-duplicate")
class Second extends Component {
  render() {
    return <div>Second</div>;
  }
}
"#,
            ),
        ]);
        let symbols = build_symbol_table(&unit);
        let modules = build_module_graph(&unit);
        let bindings = build_binding_table(&unit, &symbols, &modules);

        assert!(bindings.diagnostics.is_empty());
        assert_ne!(
            symbols
                .module("src/First.tsx")
                .expect("first module")
                .symbols["First"]
                .id,
            symbols
                .module("src/Second.tsx")
                .expect("second module")
                .symbols["Second"]
                .id
        );
    }

    #[test]
    fn resolves_external_imports_only_through_semantic_package_contracts() {
        let unit = CompilationUnit::parse_sources([(
            "src/App.tsx",
            "import { format } from \"date-kit\"; export class App {}",
        )]);
        let symbols = build_symbol_table(&unit);
        let modules = build_module_graph(&unit);
        let contract = parse_semantic_package_contract(r#"{"schema_version":1,"package":"date-kit","version":"1.0.0","integrity":"sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","exports":{"format":{"kind":"pure","type_signature":"(Date)->string","runtime_module":"dist/format.js","resume_policy":"input_only"}}}"#).unwrap();
        let mut packages = SemanticPackageResolutionTable::default();
        packages.insert("date-kit".into(), contract).unwrap();
        let bindings = build_binding_table_with_packages(&unit, &symbols, &modules, &packages);
        assert!(bindings.diagnostics.is_empty());
        let ImportBindingTarget::SemanticPackage {
            package,
            version,
            integrity,
            export,
            kind,
            type_signature,
            runtime_module,
            resume_policy,
            pure_operation: _,
            resource_endpoint: _,
            route_loader: _,
            server_action: _,
            form_submission: _,
            opaque_terminal: _,
        } = &bindings
            .resolve_import("src/App.tsx", "format")
            .unwrap()
            .target
        else {
            panic!("expected semantic package binding");
        };
        assert_eq!(package, "date-kit");
        assert_eq!(version, "1.0.0");
        assert_eq!(
            integrity,
            "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
        );
        assert_eq!(export, "format");
        assert_eq!(kind, &crate::semantic_package::SemanticPackageKind::Pure);
        assert_eq!(type_signature, "(Date)->string");
        assert_eq!(runtime_module, "dist/format.js");
        assert_eq!(resume_policy, "input_only");
    }

    #[test]
    fn accepts_the_public_presolve_authoring_import_without_a_package_capability_contract() {
        let unit = CompilationUnit::parse_sources([(
            "src/App.tsx",
            "import { component, state } from \"presolve\"; export class App {}",
        )]);
        let symbols = build_symbol_table(&unit);
        let modules = build_module_graph(&unit);
        let bindings = build_binding_table(&unit, &symbols, &modules);
        assert!(bindings.diagnostics.is_empty());
        assert!(bindings.module("src/App.tsx").unwrap().imports.is_empty());
    }

    #[test]
    fn rejects_external_imports_without_a_matching_contract_or_export() {
        let unit = CompilationUnit::parse_sources([(
            "src/App.tsx",
            "import present from \"date-kit\"; import { absent } from \"date-kit\"; import { other } from \"other-kit\"; export class App {}",
        )]);
        let symbols = build_symbol_table(&unit);
        let modules = build_module_graph(&unit);
        let contract = parse_semantic_package_contract(r#"{"schema_version":1,"package":"date-kit","version":"1.0.0","integrity":"sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","exports":{"default":{"kind":"pure","type_signature":"(Date)->string","runtime_module":"dist/default.js","resume_policy":"input_only"}}}"#).unwrap();
        let mut packages = SemanticPackageResolutionTable::default();
        packages.insert("date-kit".into(), contract).unwrap();

        let bindings = build_binding_table_with_packages(&unit, &symbols, &modules, &packages);
        assert_eq!(
            bindings
                .diagnostics
                .iter()
                .map(|diagnostic| diagnostic.code.as_str())
                .collect::<Vec<_>>(),
            vec!["PSBIND1010", "PSBIND1009"]
        );
        assert!(bindings.resolve_import("src/App.tsx", "present").is_some());
        assert!(bindings.resolve_import("src/App.tsx", "absent").is_none());
        assert!(bindings.resolve_import("src/App.tsx", "other").is_none());
    }
}
