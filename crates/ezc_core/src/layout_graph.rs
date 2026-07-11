use crate::application_semantic_model::ApplicationSemanticModel;
use crate::semantic_id::SemanticId;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LayoutGraph {
    pub routes: Vec<RouteLayoutNode>,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RouteLayoutNode {
    pub path: String,
    pub component: SemanticId,
    pub layouts: Vec<SemanticId>,
}

#[must_use]
pub fn build_layout_graph(model: &ApplicationSemanticModel) -> LayoutGraph {
    LayoutGraph {
        routes: model
            .components
            .iter()
            .filter_map(|component| {
                component.route_path.as_ref().map(|path| RouteLayoutNode {
                    path: path.clone(),
                    component: component.id.clone(),
                    layouts: Vec::new(),
                })
            })
            .collect(),
    }
}
