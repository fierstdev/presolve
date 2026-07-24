//! K5 immutable constant pooling for production-only executable products.

use std::collections::BTreeMap;

use crate::{ConstantPoolEntryId, ProductionOptimizationPolicyV1};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProductionConstantCandidate {
    pub consumer_id: String,
    pub type_tag: String,
    pub canonical_bytes: Vec<u8>,
    pub immutable: bool,
    pub identity_observable: bool,
    pub snapshot_or_public_artifact: bool,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProductionConstantPoolEntry {
    pub id: ConstantPoolEntryId,
    pub ordinal: usize,
    pub type_tag: String,
    pub canonical_bytes: Vec<u8>,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ConstantPoolConsumer {
    pub consumer_id: String,
    pub entry_id: ConstantPoolEntryId,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ConstantPoolingDecision {
    Pooled,
    Inline,
    Excluded,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ConstantPoolingReport {
    pub decisions: Vec<(String, ConstantPoolingDecision)>,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProductionConstantPool {
    pub entries: Vec<ProductionConstantPoolEntry>,
    pub consumers: Vec<ConstantPoolConsumer>,
}

#[must_use]
///
/// # Panics
///
/// Panics only if an internally validated canonical type tag becomes invalid.
pub fn pool_production_constants(
    candidates: &[ProductionConstantCandidate],
) -> (ProductionConstantPool, ConstantPoolingReport) {
    let mut groups = BTreeMap::<(String, Vec<u8>), Vec<&ProductionConstantCandidate>>::new();
    let mut decisions = Vec::new();
    for candidate in candidates {
        if !candidate.immutable
            || candidate.identity_observable
            || candidate.snapshot_or_public_artifact
            || ConstantPoolEntryId::for_canonical_value(
                &candidate.type_tag,
                &candidate.canonical_bytes,
            )
            .is_none()
        {
            decisions.push((
                candidate.consumer_id.clone(),
                ConstantPoolingDecision::Excluded,
            ));
        } else {
            groups
                .entry((
                    candidate.type_tag.clone(),
                    candidate.canonical_bytes.clone(),
                ))
                .or_default()
                .push(candidate);
        }
    }
    let mut pending = Vec::new();
    for ((type_tag, bytes), users) in groups {
        let count = users.len();
        let short = bytes.len() <= ProductionOptimizationPolicyV1::INLINE_LITERAL_MAX_UTF8_BYTES;
        let pooled_smaller =
            bytes.len().saturating_mul(count) > bytes.len() + count.saturating_mul(4);
        if count >= ProductionOptimizationPolicyV1::CONSTANT_POOL_MIN_REUSE_COUNT
            && (!short || pooled_smaller)
        {
            pending.push((
                ConstantPoolEntryId::for_canonical_value(&type_tag, &bytes)
                    .expect("validated type tag"),
                type_tag,
                bytes,
                users,
            ));
        } else {
            decisions.extend(
                users
                    .into_iter()
                    .map(|user| (user.consumer_id.clone(), ConstantPoolingDecision::Inline)),
            );
        }
    }
    pending.sort_by(|a, b| a.0.cmp(&b.0));
    let mut entries = Vec::new();
    let mut consumers = Vec::new();
    for (ordinal, (id, type_tag, canonical_bytes, users)) in pending.into_iter().enumerate() {
        entries.push(ProductionConstantPoolEntry {
            id: id.clone(),
            ordinal,
            type_tag,
            canonical_bytes,
        });
        for user in users {
            consumers.push(ConstantPoolConsumer {
                consumer_id: user.consumer_id.clone(),
                entry_id: id.clone(),
            });
            decisions.push((user.consumer_id.clone(), ConstantPoolingDecision::Pooled));
        }
    }
    consumers.sort_by(|a, b| a.consumer_id.cmp(&b.consumer_id));
    decisions.sort_by(|a, b| a.0.cmp(&b.0));
    (
        ProductionConstantPool { entries, consumers },
        ConstantPoolingReport { decisions },
    )
}

#[cfg(test)]
mod tests {
    use super::{pool_production_constants, ConstantPoolingDecision, ProductionConstantCandidate};
    fn candidate(id: &str, ty: &str, value: &[u8]) -> ProductionConstantCandidate {
        ProductionConstantCandidate {
            consumer_id: id.to_string(),
            type_tag: ty.to_string(),
            canonical_bytes: value.to_vec(),
            immutable: true,
            identity_observable: false,
            snapshot_or_public_artifact: false,
        }
    }
    #[test]
    fn k5_pools_only_safe_reused_constants_in_canonical_order() {
        let long = b"this canonical constant is longer than twenty-four bytes";
        let (pool, report) = pool_production_constants(&[
            candidate("z", "string", long),
            candidate("a", "string", long),
            candidate("number", "number", b"1"),
            candidate("number2", "number", b"1"),
        ]);
        assert_eq!(pool.entries.len(), 1);
        assert_eq!(pool.entries[0].ordinal, 0);
        assert_eq!(pool.consumers.len(), 2);
        assert!(report
            .decisions
            .iter()
            .any(|(id, decision)| id == "number" && *decision == ConstantPoolingDecision::Inline));
    }
    #[test]
    fn k5_excludes_mutable_identity_and_public_values() {
        let mut mutable = candidate("mutable", "string", b"same-long-value-for-pooling");
        mutable.immutable = false;
        let mut identity = candidate("identity", "string", b"same-long-value-for-pooling");
        identity.identity_observable = true;
        let (pool, report) = pool_production_constants(&[mutable, identity]);
        assert!(pool.entries.is_empty());
        assert!(report
            .decisions
            .iter()
            .all(|(_, decision)| *decision == ConstantPoolingDecision::Excluded));
    }
}
