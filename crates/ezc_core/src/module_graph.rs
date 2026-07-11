use std::collections::BTreeMap;
use std::path::PathBuf;

use crate::application_semantic_model::ApplicationSemanticModel;
use crate::semantic_id::SemanticId;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModuleGraph {
    pub modules: Vec<ModuleNode>,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModuleNode {
    pub path: PathBuf,
    pub components: Vec<SemanticId>,
}

#[must_use]
pub fn build_module_graph(model: &ApplicationSemanticModel) -> ModuleGraph {
    let mut modules = BTreeMap::<PathBuf, Vec<SemanticId>>::new();
    for component in &model.components {
        if let Some(provenance) = model.provenance(&component.id) {
            modules
                .entry(provenance.path.clone())
                .or_default()
                .push(component.id.clone());
        }
    }
    ModuleGraph {
        modules: modules
            .into_iter()
            .map(|(path, components)| ModuleNode { path, components })
            .collect(),
    }
}
