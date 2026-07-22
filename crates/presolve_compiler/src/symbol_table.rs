use std::collections::BTreeMap;
use std::path::Path;

use crate::compilation_unit::CompilationUnit;
use crate::component_graph::build_component_graph_for_module;
use crate::semantic_id::{SemanticId, SemanticOwner};
use crate::semantic_provenance::SourceProvenance;

/// Local declarations indexed by source module before import resolution.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SymbolTable {
    pub modules: Vec<ModuleSymbolTable>,
    pub diagnostics: Vec<SymbolDiagnostic>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModuleSymbolTable {
    pub path: std::path::PathBuf,
    pub symbols: BTreeMap<String, ModuleSymbol>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModuleSymbol {
    pub name: String,
    pub kind: SymbolKind,
    pub id: SemanticId,
    pub owner: SemanticOwner,
    pub provenance: SourceProvenance,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SymbolKind {
    Component,
    StateField,
    Method,
    TypeAlias,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SymbolDiagnostic {
    pub code: String,
    pub module: std::path::PathBuf,
    pub name: String,
    pub message: String,
}

impl SymbolTable {
    #[must_use]
    pub fn module(&self, path: impl AsRef<Path>) -> Option<&ModuleSymbolTable> {
        self.modules
            .iter()
            .find(|module| module.path == path.as_ref())
    }

    #[must_use]
    pub fn resolve(&self, module_path: impl AsRef<Path>, name: &str) -> Option<&ModuleSymbol> {
        self.module(module_path)
            .and_then(|module| module.symbols.get(name))
    }
}

#[must_use]
pub fn build_symbol_table(unit: &CompilationUnit) -> SymbolTable {
    let mut modules = Vec::new();
    let mut diagnostics = Vec::new();

    for file in unit.files() {
        let graph = build_component_graph_for_module(file);
        let mut symbols = BTreeMap::new();

        for alias in &file.type_aliases {
            let id = SemanticId::type_alias_in_module(&file.path, &alias.name);
            insert_symbol(
                &mut symbols,
                &mut diagnostics,
                &file.path,
                ModuleSymbol {
                    name: alias.name.clone(),
                    kind: SymbolKind::TypeAlias,
                    id,
                    owner: SemanticOwner::Application,
                    provenance: SourceProvenance::new(&file.path, alias.type_span),
                },
            );
        }

        for component in graph.components {
            insert_symbol(
                &mut symbols,
                &mut diagnostics,
                &file.path,
                ModuleSymbol {
                    name: component.class_name.clone(),
                    kind: SymbolKind::Component,
                    id: component.id.clone(),
                    owner: component.owner.clone(),
                    provenance: graph.provenance[&component.id].clone(),
                },
            );

            for field in component.state_fields {
                let id = field.id;
                let provenance = graph.provenance[&id].clone();
                insert_symbol(
                    &mut symbols,
                    &mut diagnostics,
                    &file.path,
                    ModuleSymbol {
                        name: format!("{}.{}", component.class_name, field.name),
                        kind: SymbolKind::StateField,
                        id,
                        owner: field.owner,
                        provenance,
                    },
                );
            }

            for method in component.methods {
                let id = method.id;
                let provenance = graph.provenance[&id].clone();
                insert_symbol(
                    &mut symbols,
                    &mut diagnostics,
                    &file.path,
                    ModuleSymbol {
                        name: format!("{}.{}", component.class_name, method.name),
                        kind: SymbolKind::Method,
                        id,
                        owner: method.owner,
                        provenance,
                    },
                );
            }
        }

        modules.push(ModuleSymbolTable {
            path: file.path.clone(),
            symbols,
        });
    }

    SymbolTable {
        modules,
        diagnostics,
    }
}

fn insert_symbol(
    symbols: &mut BTreeMap<String, ModuleSymbol>,
    diagnostics: &mut Vec<SymbolDiagnostic>,
    module: &Path,
    symbol: ModuleSymbol,
) {
    if symbols.contains_key(&symbol.name) {
        diagnostics.push(SymbolDiagnostic {
            code: "PSSYM1001".to_string(),
            module: module.to_path_buf(),
            name: symbol.name.clone(),
            message: format!("duplicate local symbol `{}`", symbol.name),
        });
        return;
    }

    symbols.insert(symbol.name.clone(), symbol);
}

#[cfg(test)]
mod tests {
    use super::{build_symbol_table, SymbolKind};
    use crate::{CompilationUnit, SemanticId, SemanticOwner};

    #[test]
    fn indexes_local_component_members_by_module() {
        let unit = CompilationUnit::parse_sources([
            (
                "src/Counter.tsx",
                r#"
import { shared } from "./shared";

@component("x-counter")
class Counter extends Component {
  count = state(0);

  increment() {
    this.count++;
  }

  render() {
    return <button>{this.count}</button>;
  }
}
"#,
            ),
            (
                "src/Status.tsx",
                r#"
@component("x-status")
class Status extends Component {
  render() {
    return <div>Status</div>;
  }
}
"#,
            ),
        ]);

        let symbols = build_symbol_table(&unit);
        let counter = symbols.module("src/Counter.tsx").expect("counter module");

        assert_eq!(symbols.modules.len(), 2);
        assert!(symbols.diagnostics.is_empty());
        assert_eq!(counter.symbols.len(), 4);
        assert_eq!(counter.symbols["Counter"].kind, SymbolKind::Component);
        assert_eq!(
            counter.symbols["Counter"].id,
            SemanticId::component_in_module("src/Counter.tsx", Some("x-counter"), "Counter")
        );
        assert_eq!(
            counter.symbols["Counter.count"].kind,
            SymbolKind::StateField
        );
        assert_eq!(
            counter.symbols["Counter.count"].owner,
            SemanticOwner::entity(SemanticId::component_in_module(
                "src/Counter.tsx",
                Some("x-counter"),
                "Counter"
            ))
        );
        assert_eq!(
            counter.symbols["Counter.increment"].kind,
            SymbolKind::Method
        );
        assert_eq!(counter.symbols["Counter.increment"].provenance.span.line, 8);
        assert!(symbols.resolve("src/Counter.tsx", "shared").is_none());
        assert!(symbols.resolve("src/Status.tsx", "Status.render").is_some());
    }

    #[test]
    fn reports_duplicate_local_member_names() {
        let unit = CompilationUnit::parse_sources([(
            "src/Duplicate.tsx",
            r#"
@component("x-duplicate")
class Duplicate extends Component {
  first() {}
  first() {}

  render() {
    return <div>Duplicate</div>;
  }
}
"#,
        )]);

        let symbols = build_symbol_table(&unit);

        assert_eq!(symbols.diagnostics.len(), 1);
        assert_eq!(symbols.diagnostics[0].code, "PSSYM1001");
        assert_eq!(symbols.diagnostics[0].name, "Duplicate.first");
    }
}
