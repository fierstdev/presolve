//! K11 canonical ordinal patch schedules and proof-gated binding coalescing.

use std::collections::BTreeSet;

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum ProductionPatchBatchKind {
    Action,
    Form,
    Reset,
    Restore,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[allow(clippy::struct_excessive_bools)]
pub struct ProductionPatchOperation {
    pub operation_id: String,
    pub batch: ProductionPatchBatchKind,
    pub program_ordinal: u32,
    pub binding_target_id: String,
    pub operation_kind: String,
    pub structural: bool,
    pub capability: bool,
    pub effect: bool,
    pub browser_read: bool,
    pub value_observed: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProductionPatchSchedule {
    pub batch: ProductionPatchBatchKind,
    pub ordinals: Vec<u32>,
    pub fixed_capacity: usize,
    pub generation_counter_required: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BindingWriteCoalescingDecision {
    pub removed_operation_id: String,
    pub retained_operation_id: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BindingWriteCoalescingReport {
    pub decisions: Vec<BindingWriteCoalescingDecision>,
    pub retained_operations: Vec<ProductionPatchOperation>,
}

/// Encodes a compiler-ordered batch as dense ordinals without runtime discovery.
#[must_use]
pub fn build_production_patch_schedule(
    operations: &[ProductionPatchOperation],
) -> Option<ProductionPatchSchedule> {
    let batch = operations.first()?.batch;
    (!operations.iter().any(|operation| operation.batch != batch)).then(|| {
        ProductionPatchSchedule {
            batch,
            ordinals: operations
                .iter()
                .map(|operation| operation.program_ordinal)
                .collect(),
            fixed_capacity: operations.len(),
            generation_counter_required: true,
        }
    })
}

/// Removes only earlier writes with complete closed-operation equivalence evidence.
#[must_use]
pub fn coalesce_production_binding_writes(
    operations: &[ProductionPatchOperation],
) -> BindingWriteCoalescingReport {
    let mut removed = BTreeSet::new();
    let mut decisions = Vec::new();
    for (left, earlier) in operations.iter().enumerate() {
        if earlier.structural
            || earlier.capability
            || earlier.effect
            || earlier.browser_read
            || earlier.value_observed
        {
            continue;
        }
        for (right, later) in operations.iter().enumerate().skip(left + 1) {
            if earlier.batch != later.batch
                || earlier.binding_target_id != later.binding_target_id
                || earlier.operation_kind != later.operation_kind
            {
                continue;
            }
            let interval_safe = operations[left + 1..right].iter().all(|operation| {
                !operation.structural
                    && !operation.capability
                    && !operation.effect
                    && !operation.browser_read
            });
            if interval_safe {
                removed.insert(left);
                decisions.push(BindingWriteCoalescingDecision {
                    removed_operation_id: earlier.operation_id.clone(),
                    retained_operation_id: later.operation_id.clone(),
                });
            }
            break;
        }
    }
    BindingWriteCoalescingReport {
        decisions,
        retained_operations: operations
            .iter()
            .enumerate()
            .filter(|(index, _)| !removed.contains(index))
            .map(|(_, operation)| operation.clone())
            .collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn write(id: &str, ordinal: u32) -> ProductionPatchOperation {
        ProductionPatchOperation {
            operation_id: id.to_string(),
            batch: ProductionPatchBatchKind::Action,
            program_ordinal: ordinal,
            binding_target_id: "binding:a".to_string(),
            operation_kind: "text".to_string(),
            structural: false,
            capability: false,
            effect: false,
            browser_read: false,
            value_observed: false,
        }
    }
    #[test]
    fn k11_coalesces_only_closed_equivalent_writes() {
        let mut blocked = write("first", 0);
        let last = write("last", 1);
        blocked.effect = true;
        assert_eq!(
            coalesce_production_binding_writes(&[write("first", 0), last.clone()])
                .decisions
                .len(),
            1
        );
        assert!(coalesce_production_binding_writes(&[blocked, last])
            .decisions
            .is_empty());
        assert_eq!(
            build_production_patch_schedule(&[write("a", 2), write("b", 3)])
                .expect("schedule")
                .ordinals,
            vec![2, 3]
        );
    }
}
