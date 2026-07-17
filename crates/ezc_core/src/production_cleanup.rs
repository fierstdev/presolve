//! K12 reverse-order cleanup closure for compiler-owned component runtime state.

use std::collections::BTreeSet;

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum ProductionCleanupKind {
    ActivationDispatch,
    EventAndBindingIndex,
    SlotAndStructuralRegistry,
    ComputedCache,
    StateStorage,
    ComponentInstance,
    DomAnchor,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProductionOwnedRuntimeRecord {
    pub owner_id: String,
    pub initialization_ordinal: u32,
    pub kind: ProductionCleanupKind,
    pub record_id: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProductionDestroyPlan {
    pub detached_activation_ids: Vec<String>,
    pub cleanup_records: Vec<ProductionOwnedRuntimeRecord>,
}

/// Builds a reverse-initialization cleanup plan without authored callbacks.
#[must_use]
pub fn build_production_destroy_plan(
    destroyed_owner_ids: &[String],
    records: &[ProductionOwnedRuntimeRecord],
    pending_activation_ids: &[String],
) -> ProductionDestroyPlan {
    let owners = destroyed_owner_ids.iter().collect::<BTreeSet<_>>();
    let mut cleanup_records = records
        .iter()
        .filter(|record| owners.contains(&record.owner_id))
        .cloned()
        .collect::<Vec<_>>();
    cleanup_records.sort_by(|left, right| {
        (right.initialization_ordinal, right.kind, &right.record_id).cmp(&(
            left.initialization_ordinal,
            left.kind,
            &left.record_id,
        ))
    });
    let mut detached_activation_ids = pending_activation_ids.to_vec();
    detached_activation_ids.sort();
    detached_activation_ids.dedup();
    ProductionDestroyPlan {
        detached_activation_ids,
        cleanup_records,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn k12_releases_only_destroyed_owners_in_reverse_initialization_order() {
        let records = vec![
            ProductionOwnedRuntimeRecord {
                owner_id: "a".to_string(),
                initialization_ordinal: 1,
                kind: ProductionCleanupKind::StateStorage,
                record_id: "state".to_string(),
            },
            ProductionOwnedRuntimeRecord {
                owner_id: "a".to_string(),
                initialization_ordinal: 2,
                kind: ProductionCleanupKind::DomAnchor,
                record_id: "anchor".to_string(),
            },
            ProductionOwnedRuntimeRecord {
                owner_id: "b".to_string(),
                initialization_ordinal: 3,
                kind: ProductionCleanupKind::ComponentInstance,
                record_id: "other".to_string(),
            },
        ];
        let plan = build_production_destroy_plan(
            &["a".to_string()],
            &records,
            &["activation:a".to_string(), "activation:a".to_string()],
        );
        assert_eq!(
            plan.cleanup_records
                .iter()
                .map(|record| record.record_id.as_str())
                .collect::<Vec<_>>(),
            vec!["anchor", "state"]
        );
        assert_eq!(plan.detached_activation_ids, vec!["activation:a"]);
    }
}
