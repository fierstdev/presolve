//! K12 reverse-order cleanup closure for compiler-owned component runtime state.

use std::collections::BTreeSet;

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum ProductionCleanupKind {
    ActivationDispatch,
    EventAndBindingIndex,
    FormControlAndBindingIndex,
    EffectSubscription,
    ContextConsumerBinding,
    SlotAndStructuralRegistry,
    ComputedCache,
    StateStorage,
    FormInstanceStorage,
    ContextProviderSlot,
    ResumeBoundaryAndAnchor,
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

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum ProductionCleanupClosureViolation {
    DuplicateRecord(String),
    ForeignOwner(String),
    MissingRecord(String),
    OrderingMismatch,
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

/// Validates complete exact-owner coverage and reverse initialization order.
#[must_use]
pub fn validate_production_cleanup_closure(
    destroyed_owner_ids: &[String],
    required_records: &[ProductionOwnedRuntimeRecord],
    plan: &ProductionDestroyPlan,
) -> Vec<ProductionCleanupClosureViolation> {
    let owners = destroyed_owner_ids.iter().collect::<BTreeSet<_>>();
    let required = required_records
        .iter()
        .filter(|record| owners.contains(&record.owner_id))
        .map(|record| record.record_id.clone())
        .collect::<BTreeSet<_>>();
    let mut actual = BTreeSet::new();
    let mut violations = Vec::new();
    for record in &plan.cleanup_records {
        if !owners.contains(&record.owner_id) {
            violations.push(ProductionCleanupClosureViolation::ForeignOwner(
                record.record_id.clone(),
            ));
        }
        if !actual.insert(record.record_id.clone()) {
            violations.push(ProductionCleanupClosureViolation::DuplicateRecord(
                record.record_id.clone(),
            ));
        }
    }
    violations.extend(
        required
            .difference(&actual)
            .cloned()
            .map(ProductionCleanupClosureViolation::MissingRecord),
    );
    if plan.cleanup_records.windows(2).any(|pair| {
        (
            pair[0].initialization_ordinal,
            pair[0].kind,
            &pair[0].record_id,
        ) < (
            pair[1].initialization_ordinal,
            pair[1].kind,
            &pair[1].record_id,
        )
    }) {
        violations.push(ProductionCleanupClosureViolation::OrderingMismatch);
    }
    violations.sort();
    violations.dedup();
    violations
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

    #[test]
    fn k13_covers_forms_context_effect_and_resume_without_authored_cleanup() {
        let records = [
            (
                ProductionCleanupKind::FormControlAndBindingIndex,
                "form-control",
            ),
            (ProductionCleanupKind::EffectSubscription, "effect"),
            (ProductionCleanupKind::ContextConsumerBinding, "consumer"),
            (ProductionCleanupKind::FormInstanceStorage, "form-state"),
            (ProductionCleanupKind::ContextProviderSlot, "provider"),
            (ProductionCleanupKind::ResumeBoundaryAndAnchor, "resume"),
        ]
        .into_iter()
        .enumerate()
        .map(
            |(ordinal, (kind, record_id))| ProductionOwnedRuntimeRecord {
                owner_id: "destroyed".to_string(),
                initialization_ordinal: u32::try_from(ordinal).expect("fixture ordinal"),
                kind,
                record_id: record_id.to_string(),
            },
        )
        .collect::<Vec<_>>();
        let plan = build_production_destroy_plan(
            &["destroyed".to_string()],
            &records,
            &["activation:destroyed".to_string()],
        );
        assert!(
            validate_production_cleanup_closure(&["destroyed".to_string()], &records, &plan)
                .is_empty()
        );
        assert_eq!(
            plan.cleanup_records.first().expect("record").record_id,
            "resume"
        );
    }
}
