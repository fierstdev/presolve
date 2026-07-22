use crate::application_semantic_model::ApplicationSemanticModel;
use crate::semantic_id::SemanticId;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RouteGraph {
    pub routes: Vec<RouteNode>,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RouteNode {
    pub path: String,
    pub component: SemanticId,
}

#[must_use]
pub fn build_route_graph(model: &ApplicationSemanticModel) -> RouteGraph {
    RouteGraph {
        routes: model
            .components
            .iter()
            .filter_map(|component| {
                component.route_path.as_ref().map(|path| RouteNode {
                    path: path.clone(),
                    component: component.id.clone(),
                })
            })
            .collect(),
    }
}
