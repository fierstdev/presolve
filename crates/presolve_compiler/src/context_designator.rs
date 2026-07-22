use std::collections::BTreeMap;

use crate::{
    BindingTable, ComponentNode, ContextDesignator, ContextEntity, ContextId, ImportBindingTarget,
    SymbolKind,
};

/// Resolve one authored Context designator to its canonical G1 identity.
///
/// This is the sole raw designator-resolution authority shared by G2 Provider
/// and G3 Consumer lowering. It performs no G4 Provider visibility or selection.
pub(crate) fn resolve_context_designator(
    designator: &ContextDesignator,
    components: &[ComponentNode],
    contexts: &BTreeMap<ContextId, ContextEntity>,
    bindings: Option<&BindingTable>,
) -> Option<ContextId> {
    let local_component = components
        .iter()
        .find(|component| {
            component.class_name == designator.component_symbol
                && contexts.values().any(|context| {
                    context.owner.entity_id() == Some(&component.id)
                        && context.provenance.path == designator.provenance.path
                })
        })
        .map(|component| component.id.clone());
    let imported_component = bindings
        .and_then(|bindings| {
            bindings.resolve_import(&designator.provenance.path, &designator.component_symbol)
        })
        .and_then(|binding| match &binding.target {
            ImportBindingTarget::Symbol(symbol) if symbol.kind == SymbolKind::Component => {
                Some(symbol.id.clone())
            }
            ImportBindingTarget::Symbol(_) | ImportBindingTarget::Namespace { .. } => None,
        });
    let component = local_component.or(imported_component)?;
    contexts
        .values()
        .find(|context| {
            context.owner.entity_id() == Some(&component)
                && context.name == designator.context_member
        })
        .map(|context| context.id.clone())
}
