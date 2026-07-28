//! J2 canonical resumability liveness over existing compiler-owned runtime slots.

use std::collections::{BTreeMap, BTreeSet};

use crate::{
    build_computed_instance_slot_registry, build_effect_resume_plan,
    build_runtime_component_registry, build_runtime_computed_registry,
    build_state_instance_storage_registry, lower_components_to_ir, optimize_effect_ir,
    ApplicationSemanticModel, ComponentInstanceId, ComponentInstanceStatus, ComponentRootId,
    ComputedInstanceCacheSlotId, ComputedInstanceDirtySlotId, ComputedPurity,
    ContextSourceInstanceOwner, ContextValueSourceId, EffectActivationSlotId, FormFieldDirtySlotId,
    FormFieldTouchedSlotId, FormFieldValidationSlotId, FormFieldValueSlotId, FormInstanceId,
    FormSubmissionStateSlotId, FormValidationAggregateSlotId, InstanceContextValueSlotId,
    IrStorageId, ResumeBoundaryId, ResumeSlotId, SemanticId, SerializationCompatibility,
    SourceProvenance, StateInstanceSlotId,
};

pub const RESUME_LIVENESS_PLAN_VERSION: u32 = 1;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum ResumeExistingSlot {
    State(StateInstanceSlotId),
    ComputedCache(ComputedInstanceCacheSlotId),
    ComputedDirty(ComputedInstanceDirtySlotId),
    Context(InstanceContextValueSlotId),
    FormFieldValue(FormFieldValueSlotId),
    FormFieldDirty(FormFieldDirtySlotId),
    FormFieldTouched(FormFieldTouchedSlotId),
    FormFieldValidation(FormFieldValidationSlotId),
    FormValidationAggregate(FormValidationAggregateSlotId),
    FormSubmission(FormSubmissionStateSlotId),
    ResourceState(SemanticId),
    ResourceData(SemanticId),
    ResourceError(SemanticId),
    EffectActivation(EffectActivationSlotId),
}

impl ResumeExistingSlot {
    #[must_use]
    pub fn text(&self) -> String {
        match self {
            Self::State(slot) => slot.to_string(),
            Self::ComputedCache(slot) => slot.to_string(),
            Self::ComputedDirty(slot) => slot.to_string(),
            Self::Context(slot) => slot.to_string(),
            Self::FormFieldValue(slot) => slot.as_str().to_string(),
            Self::FormFieldDirty(slot) => slot.as_str().to_string(),
            Self::FormFieldTouched(slot) => slot.as_str().to_string(),
            Self::FormFieldValidation(slot) => slot.as_str().to_string(),
            Self::FormValidationAggregate(slot) => slot.as_str().to_string(),
            Self::FormSubmission(slot) => slot.as_str().to_string(),
            Self::ResourceState(slot) | Self::ResourceData(slot) | Self::ResourceError(slot) => {
                slot.as_str().to_string()
            }
            Self::EffectActivation(slot) => slot.as_str().to_string(),
        }
    }

    #[must_use]
    pub fn resume_slot_id(&self) -> ResumeSlotId {
        ResumeSlotId::for_existing_storage(&self.text())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum ResumeLivenessOwner {
    ApplicationRoot(ComponentRootId),
    ComponentInstance(ComponentInstanceId),
    FormInstance(FormInstanceId),
    Semantic(SemanticId),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ResumeRetentionReason {
    MutableState,
    NonRecomputableComputedCache,
    ComputedDirtyState,
    PureEagerComputedCache,
    ComputedDirtyAfterRecompute,
    ContextConsumerDependency,
    FormFieldValue,
    FormFieldDirty,
    FormFieldTouched,
    FormRuleResult,
    FormAggregateValidation,
    StableFormSubmission,
    ResourceSnapshotValue,
    EffectSchedulerMetadata,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ResumeLivenessBlockReason {
    RequiredNonSerializableValue,
    MissingCanonicalSource,
    UnknownDependency,
    RecomputeProofUnavailable,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ResumeLivenessClassificationKind {
    Retained,
    Recomputable,
    Excluded,
    Blocked,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResumeLivenessSlot {
    pub resume_slot_id: ResumeSlotId,
    pub existing_slot: ResumeExistingSlot,
    pub owner: ResumeLivenessOwner,
    pub boundary_candidate: Option<ResumeBoundaryId>,
    pub direct_dependencies: Vec<ResumeExistingSlot>,
    pub transitive_dependencies: Vec<ResumeExistingSlot>,
    pub provenance: SourceProvenance,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResumeRetainedSlot {
    pub slot: ResumeLivenessSlot,
    pub reason: ResumeRetentionReason,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(clippy::struct_excessive_bools)]
pub struct ResumeRecomputationProof {
    pub pure: bool,
    pub deterministic: bool,
    pub eager_evaluator: bool,
    pub fixed_post_restore_order: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResumeRecomputableSlot {
    pub slot: ResumeLivenessSlot,
    pub reason: ResumeRetentionReason,
    pub proof: ResumeRecomputationProof,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResumeExcludedSlot {
    pub slot: ResumeLivenessSlot,
    pub reason: ResumeRetentionReason,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResumeLivenessBlock {
    pub slot: ResumeLivenessSlot,
    pub reason: ResumeLivenessBlockReason,
    pub integrity_code: ResumeLivenessIntegrityCode,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResumeLivenessPlan {
    pub version: u32,
    pub retained: Vec<ResumeRetainedSlot>,
    pub recomputable: Vec<ResumeRecomputableSlot>,
    pub excluded: Vec<ResumeExcludedSlot>,
    pub blocked: Vec<ResumeLivenessBlock>,
    pub slot_index: BTreeMap<ResumeExistingSlot, ResumeLivenessClassificationKind>,
}

pub enum ResumeLivenessClassificationRef<'a> {
    Retained(&'a ResumeRetainedSlot),
    Recomputable(&'a ResumeRecomputableSlot),
    Excluded(&'a ResumeExcludedSlot),
    Blocked(&'a ResumeLivenessBlock),
}

impl ResumeLivenessPlan {
    #[must_use]
    pub fn classification(
        &self,
        existing_slot: &ResumeExistingSlot,
    ) -> Option<ResumeLivenessClassificationRef<'_>> {
        self.retained
            .iter()
            .find(|record| &record.slot.existing_slot == existing_slot)
            .map(ResumeLivenessClassificationRef::Retained)
            .or_else(|| {
                self.recomputable
                    .iter()
                    .find(|record| &record.slot.existing_slot == existing_slot)
                    .map(ResumeLivenessClassificationRef::Recomputable)
            })
            .or_else(|| {
                self.excluded
                    .iter()
                    .find(|record| &record.slot.existing_slot == existing_slot)
                    .map(ResumeLivenessClassificationRef::Excluded)
            })
            .or_else(|| {
                self.blocked
                    .iter()
                    .find(|record| &record.slot.existing_slot == existing_slot)
                    .map(ResumeLivenessClassificationRef::Blocked)
            })
    }

    #[must_use]
    pub fn slots_for_owner(&self, owner: &ResumeLivenessOwner) -> Vec<&ResumeLivenessSlot> {
        self.all_slots()
            .into_iter()
            .filter(|slot| &slot.owner == owner)
            .collect()
    }

    #[must_use]
    pub fn slots_for_boundary(&self, boundary: &ResumeBoundaryId) -> Vec<&ResumeLivenessSlot> {
        self.all_slots()
            .into_iter()
            .filter(|slot| slot.boundary_candidate.as_ref() == Some(boundary))
            .collect()
    }

    #[must_use]
    pub fn slots_for_reason(&self, reason: ResumeRetentionReason) -> Vec<&ResumeLivenessSlot> {
        let mut slots = self
            .retained
            .iter()
            .filter(|record| record.reason == reason)
            .map(|record| &record.slot)
            .chain(
                self.recomputable
                    .iter()
                    .filter(|record| record.reason == reason)
                    .map(|record| &record.slot),
            )
            .chain(
                self.excluded
                    .iter()
                    .filter(|record| record.reason == reason)
                    .map(|record| &record.slot),
            )
            .collect::<Vec<_>>();
        slots.sort_by(|left, right| left.existing_slot.cmp(&right.existing_slot));
        slots
    }

    fn all_slots(&self) -> Vec<&ResumeLivenessSlot> {
        let mut slots = self
            .retained
            .iter()
            .map(|record| &record.slot)
            .chain(self.recomputable.iter().map(|record| &record.slot))
            .chain(self.excluded.iter().map(|record| &record.slot))
            .chain(self.blocked.iter().map(|record| &record.slot))
            .collect::<Vec<_>>();
        slots.sort_by(|left, right| left.existing_slot.cmp(&right.existing_slot));
        slots
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ResumeLivenessIntegrityCode {
    DuplicateClassification,
    MissingStorageOwner,
    UnknownDependency,
    InvalidRetentionReason,
    RecomputeWithoutProof,
    RequiredUnsupportedValue,
    InvalidCandidatePromotion,
    ProvenanceOrderIndexDrift,
}

impl ResumeLivenessIntegrityCode {
    #[must_use]
    pub const fn code(self) -> &'static str {
        match self {
            Self::DuplicateClassification => "PSASM1320",
            Self::MissingStorageOwner => "PSASM1321",
            Self::UnknownDependency => "PSASM1322",
            Self::InvalidRetentionReason => "PSASM1323",
            Self::RecomputeWithoutProof => "PSASM1324",
            Self::RequiredUnsupportedValue => "PSASM1325",
            Self::InvalidCandidatePromotion => "PSASM1326",
            Self::ProvenanceOrderIndexDrift => "PSASM1327",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResumeLivenessIntegrityDiagnostic {
    pub code: ResumeLivenessIntegrityCode,
    pub slot: Option<ResumeExistingSlot>,
    pub message: String,
}

#[derive(Debug, Clone)]
struct ClassificationEvidence {
    kind: ResumeLivenessClassificationKind,
    transitive_dependencies: Vec<ResumeExistingSlot>,
}

#[must_use]
#[allow(clippy::too_many_lines)]
pub fn build_resume_liveness_plan(model: &ApplicationSemanticModel) -> ResumeLivenessPlan {
    let ir = lower_components_to_ir(model);
    let state = build_state_instance_storage_registry(model, &ir);
    let computed = build_computed_instance_slot_registry(model, &ir);
    let computed_runtime = build_runtime_computed_registry(model, &ir);
    let component_registry =
        build_runtime_component_registry(model, &model.component_ir_optimization);
    let effect_registry =
        crate::build_runtime_effect_registry(model, &optimize_effect_ir(&ir).output);
    let effect_resume = build_effect_resume_plan(model, &effect_registry);

    let mut retained = Vec::new();
    let mut recomputable = Vec::new();
    let mut excluded = Vec::new();
    let mut blocked = Vec::new();
    let mut evidence = BTreeMap::<ResumeExistingSlot, ClassificationEvidence>::new();

    for record in &state.records {
        let existing_slot = ResumeExistingSlot::State(record.slot_id.clone());
        let slot = liveness_slot(
            existing_slot.clone(),
            ResumeLivenessOwner::ComponentInstance(record.component_instance_id.clone()),
            Some(ResumeBoundaryId::component_instance(
                &record.component_instance_id,
            )),
            Vec::new(),
            Vec::new(),
            record.provenance.clone(),
        );
        if record.serialization == SerializationCompatibility::Serializable {
            retained.push(ResumeRetainedSlot {
                slot,
                reason: ResumeRetentionReason::MutableState,
            });
            insert_evidence(
                &mut evidence,
                existing_slot,
                ResumeLivenessClassificationKind::Retained,
                Vec::new(),
            );
        } else {
            blocked.push(ResumeLivenessBlock {
                slot,
                reason: ResumeLivenessBlockReason::RequiredNonSerializableValue,
                integrity_code: ResumeLivenessIntegrityCode::RequiredUnsupportedValue,
            });
            insert_evidence(
                &mut evidence,
                existing_slot,
                ResumeLivenessClassificationKind::Blocked,
                Vec::new(),
            );
        }
    }

    // Snapshot-policy Resources always reserve all three terminal slots. Data
    // and error are nullable because exactly one is populated by the runtime
    // according to the restored state; this keeps capture shape closed and
    // instance-qualified without source or endpoint inference.
    for activation in snapshot_resource_activations(model) {
        let declaration = &model.resource_declarations[&activation.declaration];
        let owner = ResumeLivenessOwner::ComponentInstance(activation.component_instance.clone());
        let boundary = Some(ResumeBoundaryId::component_instance(
            &activation.component_instance,
        ));
        for existing_slot in [
            ResumeExistingSlot::ResourceState(activation.id.state_slot()),
            ResumeExistingSlot::ResourceData(activation.id.data_slot()),
            ResumeExistingSlot::ResourceError(activation.id.error_slot()),
        ] {
            retain_slot(
                existing_slot,
                owner.clone(),
                boundary.clone(),
                declaration.provenance.clone(),
                ResumeRetentionReason::ResourceSnapshotValue,
                &mut retained,
                &mut evidence,
            );
        }
    }

    let computed_by_pair = computed
        .records
        .iter()
        .map(|record| {
            (
                (
                    record.component_instance_id.clone(),
                    record.computed_id.as_str().to_string(),
                ),
                record,
            )
        })
        .collect::<BTreeMap<_, _>>();
    let mut ordered_computed = Vec::new();
    for instance in model.component_instance_plan.instances.values() {
        if instance.status != ComponentInstanceStatus::Planned {
            continue;
        }
        for computed_id in &model.computed_evaluation_plan.evaluation_order {
            if let Some(record) = computed_by_pair.get(&(instance.id.clone(), computed_id.clone()))
            {
                ordered_computed.push(*record);
            }
        }
    }
    for record in &computed.records {
        if !ordered_computed
            .iter()
            .any(|candidate| candidate.cache_slot_id == record.cache_slot_id)
        {
            ordered_computed.push(record);
        }
    }

    for record in ordered_computed {
        let Some(declaration) = computed_runtime.record(&record.computed_id) else {
            let provenance = model.computed_values[&record.computed_id]
                .provenance
                .clone();
            classify_computed_block(
                record,
                &provenance,
                &mut blocked,
                &mut evidence,
                ResumeLivenessBlockReason::MissingCanonicalSource,
            );
            continue;
        };
        let (direct_dependencies, unknown_dependency) = dependency_slots(
            model,
            &record.component_instance_id,
            &declaration.dependencies,
            &state,
            &computed,
            &model.resource_activations,
        );
        let transitive_dependencies = transitive_dependencies(&direct_dependencies, &evidence);
        let dependency_ready = !unknown_dependency
            && direct_dependencies.iter().all(|dependency| {
                evidence.get(dependency).is_some_and(|evidence| {
                    matches!(
                        evidence.kind,
                        ResumeLivenessClassificationKind::Retained
                            | ResumeLivenessClassificationKind::Recomputable
                    )
                })
            });
        let pure = model
            .computed_values
            .get(&record.computed_id)
            .is_some_and(|value| value.purity == ComputedPurity::Pure);
        let eager = model
            .computed_evaluation_plan
            .evaluation_order
            .iter()
            .any(|computed| computed == record.computed_id.as_str());
        let cache_existing = ResumeExistingSlot::ComputedCache(record.cache_slot_id.clone());
        let cache_slot = liveness_slot(
            cache_existing.clone(),
            ResumeLivenessOwner::ComponentInstance(record.component_instance_id.clone()),
            Some(ResumeBoundaryId::component_instance(
                &record.component_instance_id,
            )),
            direct_dependencies,
            transitive_dependencies.clone(),
            declaration.provenance.clone(),
        );
        let cache_kind = if pure && eager && dependency_ready {
            recomputable.push(ResumeRecomputableSlot {
                slot: cache_slot,
                reason: ResumeRetentionReason::PureEagerComputedCache,
                proof: complete_recomputation_proof(),
            });
            ResumeLivenessClassificationKind::Recomputable
        } else if declaration.serialization == SerializationCompatibility::Serializable {
            retained.push(ResumeRetainedSlot {
                slot: cache_slot,
                reason: ResumeRetentionReason::NonRecomputableComputedCache,
            });
            ResumeLivenessClassificationKind::Retained
        } else {
            blocked.push(ResumeLivenessBlock {
                slot: cache_slot,
                reason: if unknown_dependency {
                    ResumeLivenessBlockReason::UnknownDependency
                } else {
                    ResumeLivenessBlockReason::RecomputeProofUnavailable
                },
                integrity_code: if unknown_dependency {
                    ResumeLivenessIntegrityCode::UnknownDependency
                } else {
                    ResumeLivenessIntegrityCode::RequiredUnsupportedValue
                },
            });
            ResumeLivenessClassificationKind::Blocked
        };
        insert_evidence(
            &mut evidence,
            cache_existing.clone(),
            cache_kind,
            transitive_dependencies,
        );

        let dirty_existing = ResumeExistingSlot::ComputedDirty(record.dirty_slot_id.clone());
        let dirty_slot = liveness_slot(
            dirty_existing.clone(),
            ResumeLivenessOwner::ComponentInstance(record.component_instance_id.clone()),
            Some(ResumeBoundaryId::component_instance(
                &record.component_instance_id,
            )),
            vec![cache_existing.clone()],
            vec![cache_existing],
            declaration.provenance.clone(),
        );
        match cache_kind {
            ResumeLivenessClassificationKind::Recomputable => {
                recomputable.push(ResumeRecomputableSlot {
                    slot: dirty_slot,
                    reason: ResumeRetentionReason::ComputedDirtyAfterRecompute,
                    proof: complete_recomputation_proof(),
                });
            }
            ResumeLivenessClassificationKind::Retained => {
                retained.push(ResumeRetainedSlot {
                    slot: dirty_slot,
                    reason: ResumeRetentionReason::ComputedDirtyState,
                });
            }
            ResumeLivenessClassificationKind::Blocked
            | ResumeLivenessClassificationKind::Excluded => {
                blocked.push(ResumeLivenessBlock {
                    slot: dirty_slot,
                    reason: ResumeLivenessBlockReason::RecomputeProofUnavailable,
                    integrity_code: ResumeLivenessIntegrityCode::RequiredUnsupportedValue,
                });
            }
        }
        insert_evidence(&mut evidence, dirty_existing, cache_kind, Vec::new());
    }

    let context_bindings = component_registry
        .instance_context_bindings
        .iter()
        .map(|binding| (binding.runtime_slot.clone(), binding))
        .collect::<BTreeMap<_, _>>();
    for (runtime_slot, binding) in context_bindings {
        let existing_slot = ResumeExistingSlot::Context(runtime_slot);
        let (owner, component_instance, boundary) = match &binding.selected_source.owner {
            ContextSourceInstanceOwner::Provider(provider) => (
                ResumeLivenessOwner::ComponentInstance(provider.component_instance.clone()),
                provider.component_instance.clone(),
                Some(ResumeBoundaryId::component_instance(
                    &provider.component_instance,
                )),
            ),
            ContextSourceInstanceOwner::RootDefault(default) => {
                let instance = ComponentInstanceId::for_root(&default.owner_root);
                (
                    ResumeLivenessOwner::ApplicationRoot(default.owner_root.clone()),
                    instance,
                    Some(ResumeBoundaryId::application_root(&default.owner_root)),
                )
            }
        };
        let declaration_source = match &binding.selected_source.owner {
            ContextSourceInstanceOwner::Provider(provider) => {
                ContextValueSourceId::Provider(provider.provider.clone())
            }
            ContextSourceInstanceOwner::RootDefault(default) => {
                ContextValueSourceId::ContextDefault(default.context.clone())
            }
        };
        let Some(source) = model
            .context_evaluation
            .context_source_plan(&declaration_source)
        else {
            let slot = liveness_slot(
                existing_slot.clone(),
                owner,
                boundary,
                Vec::new(),
                Vec::new(),
                binding.provenance.clone(),
            );
            blocked.push(ResumeLivenessBlock {
                slot,
                reason: ResumeLivenessBlockReason::MissingCanonicalSource,
                integrity_code: ResumeLivenessIntegrityCode::MissingStorageOwner,
            });
            insert_evidence(
                &mut evidence,
                existing_slot,
                ResumeLivenessClassificationKind::Blocked,
                Vec::new(),
            );
            continue;
        };
        let mut semantic_dependencies = source.required_state.clone();
        semantic_dependencies.extend(source.required_computed.clone());
        let (direct_dependencies, unknown_dependency) = dependency_slots(
            model,
            &component_instance,
            &semantic_dependencies,
            &state,
            &computed,
            &model.resource_activations,
        );
        let transitive_dependencies = transitive_dependencies(&direct_dependencies, &evidence);
        let slot = liveness_slot(
            existing_slot.clone(),
            owner,
            boundary,
            direct_dependencies,
            transitive_dependencies.clone(),
            binding.provenance.clone(),
        );
        if unknown_dependency {
            blocked.push(ResumeLivenessBlock {
                slot,
                reason: ResumeLivenessBlockReason::UnknownDependency,
                integrity_code: ResumeLivenessIntegrityCode::UnknownDependency,
            });
            insert_evidence(
                &mut evidence,
                existing_slot,
                ResumeLivenessClassificationKind::Blocked,
                transitive_dependencies,
            );
        } else {
            retained.push(ResumeRetainedSlot {
                slot,
                reason: ResumeRetentionReason::ContextConsumerDependency,
            });
            insert_evidence(
                &mut evidence,
                existing_slot,
                ResumeLivenessClassificationKind::Retained,
                transitive_dependencies,
            );
        }
    }

    for instance in model.optimized_form_ir.optimized.instances.values() {
        let owner = ResumeLivenessOwner::FormInstance(instance.id.clone());
        let boundary = Some(ResumeBoundaryId::form_instance(&instance.id));
        let has_file_field = instance.storage.value.keys().any(|field_id| {
            model
                .form_fields
                .get(field_id)
                .is_some_and(|field| is_file_array(&field.semantic_type))
        });
        for (field_id, value_slot) in &instance.storage.value {
            let Some(field) = model.form_fields.get(field_id) else {
                continue;
            };
            let file_field = is_file_array(&field.semantic_type);
            if file_field {
                for (existing_slot, reason) in [
                    (
                        ResumeExistingSlot::FormFieldValue(value_slot.clone()),
                        ResumeRetentionReason::FormFieldValue,
                    ),
                    (
                        ResumeExistingSlot::FormFieldDirty(
                            instance.storage.dirty[field_id].clone(),
                        ),
                        ResumeRetentionReason::FormFieldDirty,
                    ),
                    (
                        ResumeExistingSlot::FormFieldTouched(
                            instance.storage.touched[field_id].clone(),
                        ),
                        ResumeRetentionReason::FormFieldTouched,
                    ),
                    (
                        ResumeExistingSlot::FormFieldValidation(
                            instance.storage.validation[field_id].clone(),
                        ),
                        ResumeRetentionReason::FormRuleResult,
                    ),
                ] {
                    exclude_slot(
                        existing_slot,
                        owner.clone(),
                        boundary.clone(),
                        field.provenance.clone(),
                        reason,
                        &mut excluded,
                        &mut evidence,
                    );
                }
                continue;
            }
            classify_form_value(
                value_slot,
                field,
                &owner,
                boundary.as_ref(),
                &mut retained,
                &mut blocked,
                &mut evidence,
            );
            for (existing_slot, reason) in [
                (
                    ResumeExistingSlot::FormFieldDirty(instance.storage.dirty[field_id].clone()),
                    ResumeRetentionReason::FormFieldDirty,
                ),
                (
                    ResumeExistingSlot::FormFieldTouched(
                        instance.storage.touched[field_id].clone(),
                    ),
                    ResumeRetentionReason::FormFieldTouched,
                ),
            ] {
                retain_slot(
                    existing_slot,
                    owner.clone(),
                    boundary.clone(),
                    field.provenance.clone(),
                    reason,
                    &mut retained,
                    &mut evidence,
                );
            }
            let validation_slot = ResumeExistingSlot::FormFieldValidation(
                instance.storage.validation[field_id].clone(),
            );
            if has_file_field {
                exclude_slot(
                    validation_slot,
                    owner.clone(),
                    boundary.clone(),
                    field.provenance.clone(),
                    ResumeRetentionReason::FormRuleResult,
                    &mut excluded,
                    &mut evidence,
                );
            } else {
                retain_slot(
                    validation_slot,
                    owner.clone(),
                    boundary.clone(),
                    field.provenance.clone(),
                    ResumeRetentionReason::FormRuleResult,
                    &mut retained,
                    &mut evidence,
                );
            }
        }
        let form_provenance = model.forms[&instance.form].provenance.clone();
        let aggregate =
            ResumeExistingSlot::FormValidationAggregate(instance.storage.aggregate.clone());
        if has_file_field {
            exclude_slot(
                aggregate,
                owner.clone(),
                boundary.clone(),
                form_provenance.clone(),
                ResumeRetentionReason::FormAggregateValidation,
                &mut excluded,
                &mut evidence,
            );
        } else {
            retain_slot(
                aggregate,
                owner.clone(),
                boundary.clone(),
                form_provenance.clone(),
                ResumeRetentionReason::FormAggregateValidation,
                &mut retained,
                &mut evidence,
            );
        }
        retain_slot(
            ResumeExistingSlot::FormSubmission(instance.storage.submission.clone()),
            owner,
            boundary,
            form_provenance,
            ResumeRetentionReason::StableFormSubmission,
            &mut retained,
            &mut evidence,
        );
    }

    for record in effect_resume.records {
        let Some(activation_slot) = record.activation_slot else {
            continue;
        };
        let existing_slot = ResumeExistingSlot::EffectActivation(activation_slot);
        excluded.push(ResumeExcludedSlot {
            slot: liveness_slot(
                existing_slot.clone(),
                ResumeLivenessOwner::Semantic(record.effect),
                None,
                Vec::new(),
                Vec::new(),
                record.provenance,
            ),
            reason: ResumeRetentionReason::EffectSchedulerMetadata,
        });
        insert_evidence(
            &mut evidence,
            existing_slot,
            ResumeLivenessClassificationKind::Excluded,
            Vec::new(),
        );
    }

    retained.sort_by(|left, right| left.slot.existing_slot.cmp(&right.slot.existing_slot));
    recomputable.sort_by(|left, right| left.slot.existing_slot.cmp(&right.slot.existing_slot));
    excluded.sort_by(|left, right| left.slot.existing_slot.cmp(&right.slot.existing_slot));
    blocked.sort_by(|left, right| left.slot.existing_slot.cmp(&right.slot.existing_slot));
    let slot_index = retained
        .iter()
        .map(|record| {
            (
                record.slot.existing_slot.clone(),
                ResumeLivenessClassificationKind::Retained,
            )
        })
        .chain(recomputable.iter().map(|record| {
            (
                record.slot.existing_slot.clone(),
                ResumeLivenessClassificationKind::Recomputable,
            )
        }))
        .chain(excluded.iter().map(|record| {
            (
                record.slot.existing_slot.clone(),
                ResumeLivenessClassificationKind::Excluded,
            )
        }))
        .chain(blocked.iter().map(|record| {
            (
                record.slot.existing_slot.clone(),
                ResumeLivenessClassificationKind::Blocked,
            )
        }))
        .collect();
    ResumeLivenessPlan {
        version: RESUME_LIVENESS_PLAN_VERSION,
        retained,
        recomputable,
        excluded,
        blocked,
        slot_index,
    }
}

fn complete_recomputation_proof() -> ResumeRecomputationProof {
    ResumeRecomputationProof {
        pure: true,
        deterministic: true,
        eager_evaluator: true,
        fixed_post_restore_order: true,
    }
}

fn insert_evidence(
    evidence: &mut BTreeMap<ResumeExistingSlot, ClassificationEvidence>,
    slot: ResumeExistingSlot,
    kind: ResumeLivenessClassificationKind,
    transitive_dependencies: Vec<ResumeExistingSlot>,
) {
    evidence.insert(
        slot,
        ClassificationEvidence {
            kind,
            transitive_dependencies,
        },
    );
}

fn classify_computed_block(
    record: &crate::ComputedInstanceSlotRecord,
    provenance: &SourceProvenance,
    blocked: &mut Vec<ResumeLivenessBlock>,
    evidence: &mut BTreeMap<ResumeExistingSlot, ClassificationEvidence>,
    reason: ResumeLivenessBlockReason,
) {
    for existing_slot in [
        ResumeExistingSlot::ComputedCache(record.cache_slot_id.clone()),
        ResumeExistingSlot::ComputedDirty(record.dirty_slot_id.clone()),
    ] {
        blocked.push(ResumeLivenessBlock {
            slot: liveness_slot(
                existing_slot.clone(),
                ResumeLivenessOwner::ComponentInstance(record.component_instance_id.clone()),
                Some(ResumeBoundaryId::component_instance(
                    &record.component_instance_id,
                )),
                Vec::new(),
                Vec::new(),
                provenance.clone(),
            ),
            reason,
            integrity_code: ResumeLivenessIntegrityCode::MissingStorageOwner,
        });
        insert_evidence(
            evidence,
            existing_slot,
            ResumeLivenessClassificationKind::Blocked,
            Vec::new(),
        );
    }
}

fn classify_form_value(
    value_slot: &FormFieldValueSlotId,
    field: &crate::FormFieldEntity,
    owner: &ResumeLivenessOwner,
    boundary: Option<&ResumeBoundaryId>,
    retained: &mut Vec<ResumeRetainedSlot>,
    blocked: &mut Vec<ResumeLivenessBlock>,
    evidence: &mut BTreeMap<ResumeExistingSlot, ClassificationEvidence>,
) {
    let existing_slot = ResumeExistingSlot::FormFieldValue(value_slot.clone());
    let slot = liveness_slot(
        existing_slot.clone(),
        owner.clone(),
        boundary.cloned(),
        Vec::new(),
        Vec::new(),
        field.provenance.clone(),
    );
    if crate::serialization_compatibility(&field.semantic_type)
        == SerializationCompatibility::Serializable
    {
        retained.push(ResumeRetainedSlot {
            slot,
            reason: ResumeRetentionReason::FormFieldValue,
        });
        insert_evidence(
            evidence,
            existing_slot,
            ResumeLivenessClassificationKind::Retained,
            Vec::new(),
        );
    } else {
        blocked.push(ResumeLivenessBlock {
            slot,
            reason: ResumeLivenessBlockReason::RequiredNonSerializableValue,
            integrity_code: ResumeLivenessIntegrityCode::RequiredUnsupportedValue,
        });
        insert_evidence(
            evidence,
            existing_slot,
            ResumeLivenessClassificationKind::Blocked,
            Vec::new(),
        );
    }
}

fn retain_slot(
    existing_slot: ResumeExistingSlot,
    owner: ResumeLivenessOwner,
    boundary: Option<ResumeBoundaryId>,
    provenance: SourceProvenance,
    reason: ResumeRetentionReason,
    retained: &mut Vec<ResumeRetainedSlot>,
    evidence: &mut BTreeMap<ResumeExistingSlot, ClassificationEvidence>,
) {
    retained.push(ResumeRetainedSlot {
        slot: liveness_slot(
            existing_slot.clone(),
            owner,
            boundary,
            Vec::new(),
            Vec::new(),
            provenance,
        ),
        reason,
    });
    insert_evidence(
        evidence,
        existing_slot,
        ResumeLivenessClassificationKind::Retained,
        Vec::new(),
    );
}

fn exclude_slot(
    existing_slot: ResumeExistingSlot,
    owner: ResumeLivenessOwner,
    boundary: Option<ResumeBoundaryId>,
    provenance: SourceProvenance,
    reason: ResumeRetentionReason,
    excluded: &mut Vec<ResumeExcludedSlot>,
    evidence: &mut BTreeMap<ResumeExistingSlot, ClassificationEvidence>,
) {
    excluded.push(ResumeExcludedSlot {
        slot: liveness_slot(
            existing_slot.clone(),
            owner,
            boundary,
            Vec::new(),
            Vec::new(),
            provenance,
        ),
        reason,
    });
    insert_evidence(
        evidence,
        existing_slot,
        ResumeLivenessClassificationKind::Excluded,
        Vec::new(),
    );
}

fn is_file_array(semantic_type: &crate::SemanticType) -> bool {
    matches!(
        semantic_type,
        crate::SemanticType::Array(element)
            if element.as_ref() == &crate::SemanticType::File
    )
}

fn dependency_slots(
    model: &ApplicationSemanticModel,
    component_instance: &ComponentInstanceId,
    dependencies: &[SemanticId],
    state: &crate::StateInstanceStorageRegistry,
    computed: &crate::ComputedInstanceSlotRegistry,
    resources: &BTreeMap<crate::ResourceActivationId, crate::ResourceActivation>,
) -> (Vec<ResumeExistingSlot>, bool) {
    let mut slots = Vec::new();
    let mut unknown = false;
    for dependency in dependencies {
        if model
            .components
            .iter()
            .flat_map(|component| &component.state_fields)
            .any(|state| state.id == *dependency)
        {
            let storage = IrStorageId::for_semantic_origin(dependency);
            if let Some(record) = state.record(component_instance, &storage) {
                slots.push(ResumeExistingSlot::State(record.slot_id.clone()));
            } else {
                unknown = true;
            }
        } else if let Some(record) = computed.records.iter().find(|record| {
            record.component_instance_id == *component_instance && record.computed_id == *dependency
        }) {
            slots.push(ResumeExistingSlot::ComputedCache(
                record.cache_slot_id.clone(),
            ));
        } else if let Some(resource) = model
            .resource_declarations
            .values()
            .find(|resource| resource.id.as_semantic_id() == dependency)
        {
            if let Some(activation) = resources.values().find(|activation| {
                activation.component_instance == *component_instance
                    && activation.declaration == resource.id
            }) {
                slots.push(ResumeExistingSlot::ResourceData(activation.id.data_slot()));
            } else {
                unknown = true;
            }
        } else {
            unknown = true;
        }
    }
    slots.sort();
    slots.dedup();
    (slots, unknown)
}

fn snapshot_resource_activations(
    model: &ApplicationSemanticModel,
) -> Vec<&crate::ResourceActivation> {
    model
        .resource_activations
        .values()
        .filter(|activation| {
            let Some(declaration) = model.resource_declarations.get(&activation.declaration) else {
                return false;
            };
            model
                .resource_endpoint_resolutions
                .iter()
                .any(|resolution| {
                    resolution.owner_component == declaration.owner_component
                        && resolution.field == declaration.name
                        && matches!(
                            resolution.outcome,
                            crate::ResourceEndpointResolutionOutcome::Resolved(ref endpoint)
                                if matches!(
                                    endpoint.endpoint.resume,
                                    crate::SemanticPackageResourceResumePolicy::Snapshot
                                )
                        )
                })
        })
        .collect()
}

fn transitive_dependencies(
    direct: &[ResumeExistingSlot],
    evidence: &BTreeMap<ResumeExistingSlot, ClassificationEvidence>,
) -> Vec<ResumeExistingSlot> {
    let mut transitive = direct.iter().cloned().collect::<BTreeSet<_>>();
    for dependency in direct {
        if let Some(dependency_evidence) = evidence.get(dependency) {
            transitive.extend(dependency_evidence.transitive_dependencies.iter().cloned());
        }
    }
    transitive.into_iter().collect()
}

fn liveness_slot(
    existing_slot: ResumeExistingSlot,
    owner: ResumeLivenessOwner,
    boundary_candidate: Option<ResumeBoundaryId>,
    direct_dependencies: Vec<ResumeExistingSlot>,
    transitive_dependencies: Vec<ResumeExistingSlot>,
    provenance: SourceProvenance,
) -> ResumeLivenessSlot {
    ResumeLivenessSlot {
        resume_slot_id: existing_slot.resume_slot_id(),
        existing_slot,
        owner,
        boundary_candidate,
        direct_dependencies,
        transitive_dependencies,
        provenance,
    }
}

#[must_use]
pub fn validate_resume_liveness_plan(
    model: &ApplicationSemanticModel,
    plan: &ResumeLivenessPlan,
) -> Vec<ResumeLivenessIntegrityDiagnostic> {
    let mut diagnostics = Vec::new();
    let canonical = build_resume_liveness_plan(model);
    let mut classified = BTreeSet::new();
    for slot in plan.all_slots() {
        if !classified.insert(slot.existing_slot.clone()) {
            diagnostics.push(integrity(
                ResumeLivenessIntegrityCode::DuplicateClassification,
                Some(slot.existing_slot.clone()),
                "runtime slot received more than one liveness classification",
            ));
        }
        for dependency in slot
            .direct_dependencies
            .iter()
            .chain(&slot.transitive_dependencies)
        {
            if !plan.slot_index.contains_key(dependency) {
                diagnostics.push(integrity(
                    ResumeLivenessIntegrityCode::UnknownDependency,
                    Some(slot.existing_slot.clone()),
                    "liveness evidence referenced an unknown runtime slot",
                ));
            }
        }
        let Some(expected) = canonical.classification(&slot.existing_slot) else {
            diagnostics.push(integrity(
                ResumeLivenessIntegrityCode::MissingStorageOwner,
                Some(slot.existing_slot.clone()),
                "liveness classification does not correspond to canonical runtime storage",
            ));
            continue;
        };
        let Some(actual) = plan.classification(&slot.existing_slot) else {
            continue;
        };
        if classification_slot(&actual).owner != classification_slot(&expected).owner {
            diagnostics.push(integrity(
                ResumeLivenessIntegrityCode::MissingStorageOwner,
                Some(slot.existing_slot.clone()),
                "liveness classification owner disagrees with canonical instance storage",
            ));
        }
        if classification_slot(&actual).boundary_candidate
            != classification_slot(&expected).boundary_candidate
        {
            diagnostics.push(integrity(
                ResumeLivenessIntegrityCode::InvalidCandidatePromotion,
                Some(slot.existing_slot.clone()),
                "runtime slot was promoted to a noncanonical resume boundary candidate",
            ));
        }
        if classification_kind(&expected) == ResumeLivenessClassificationKind::Blocked
            && classification_kind(&actual) != ResumeLivenessClassificationKind::Blocked
        {
            diagnostics.push(integrity(
                ResumeLivenessIntegrityCode::RequiredUnsupportedValue,
                Some(slot.existing_slot.clone()),
                "required unsupported value was classified as resumable",
            ));
        }
        if classification_kind(&actual) != classification_kind(&expected)
            || classification_reason(&actual) != classification_reason(&expected)
        {
            diagnostics.push(integrity(
                ResumeLivenessIntegrityCode::InvalidRetentionReason,
                Some(slot.existing_slot.clone()),
                "runtime slot liveness classification or reason disagrees with canonical policy",
            ));
        }
    }
    for record in &plan.recomputable {
        if !record.proof.pure
            || !record.proof.deterministic
            || !record.proof.eager_evaluator
            || !record.proof.fixed_post_restore_order
        {
            diagnostics.push(integrity(
                ResumeLivenessIntegrityCode::RecomputeWithoutProof,
                Some(record.slot.existing_slot.clone()),
                "recomputable slot was missing one required canonical proof",
            ));
        }
    }
    if plan.version != RESUME_LIVENESS_PLAN_VERSION || plan != &canonical {
        diagnostics.push(integrity(
            ResumeLivenessIntegrityCode::ProvenanceOrderIndexDrift,
            None,
            "liveness plan drifted from canonical storage, dependency, type, and instance products",
        ));
    }
    diagnostics.sort_by(|left, right| {
        (left.code, &left.slot, left.message.as_str()).cmp(&(
            right.code,
            &right.slot,
            right.message.as_str(),
        ))
    });
    diagnostics.dedup();
    diagnostics
}

fn classification_slot<'a>(
    classification: &ResumeLivenessClassificationRef<'a>,
) -> &'a ResumeLivenessSlot {
    match classification {
        ResumeLivenessClassificationRef::Retained(record) => &record.slot,
        ResumeLivenessClassificationRef::Recomputable(record) => &record.slot,
        ResumeLivenessClassificationRef::Excluded(record) => &record.slot,
        ResumeLivenessClassificationRef::Blocked(record) => &record.slot,
    }
}

fn classification_kind(
    classification: &ResumeLivenessClassificationRef<'_>,
) -> ResumeLivenessClassificationKind {
    match classification {
        ResumeLivenessClassificationRef::Retained(_) => ResumeLivenessClassificationKind::Retained,
        ResumeLivenessClassificationRef::Recomputable(_) => {
            ResumeLivenessClassificationKind::Recomputable
        }
        ResumeLivenessClassificationRef::Excluded(_) => ResumeLivenessClassificationKind::Excluded,
        ResumeLivenessClassificationRef::Blocked(_) => ResumeLivenessClassificationKind::Blocked,
    }
}

fn classification_reason(
    classification: &ResumeLivenessClassificationRef<'_>,
) -> Option<ResumeRetentionReason> {
    match classification {
        ResumeLivenessClassificationRef::Retained(record) => Some(record.reason),
        ResumeLivenessClassificationRef::Recomputable(record) => Some(record.reason),
        ResumeLivenessClassificationRef::Excluded(record) => Some(record.reason),
        ResumeLivenessClassificationRef::Blocked(_) => None,
    }
}

fn integrity(
    code: ResumeLivenessIntegrityCode,
    slot: Option<ResumeExistingSlot>,
    message: &str,
) -> ResumeLivenessIntegrityDiagnostic {
    ResumeLivenessIntegrityDiagnostic {
        code,
        slot,
        message: message.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classifies_every_exact_state_computed_context_and_form_slot_once() {
        let model = crate::build_application_semantic_model(&presolve_parser::parse_file(
            "src/Liveness.tsx",
            r#"
@component("x-liveness-theme") class Theme extends Component {
  @context() color!: string;
  render() { return <span />; }
}
@component("x-liveness-child") class Child extends Component {
  count = state(1);
  @computed() get doubled() { return this.count * 2; }
  @consume(Theme.color) color!: string;
  render() { return <span>{this.count}</span>; }
}
@component("x-liveness-page") @route("/") class Page extends Component {
  theme = state("blue");
  @provide(Theme.color) providedTheme: string = this.theme;
  @form() profile!: Form;
  @field(this.profile) name = "";
  render() { return <><Child /><Child /><input field={this.name}/></>; }
}"#,
        ));
        let plan = build_resume_liveness_plan(&model);
        assert!(validate_resume_liveness_plan(&model, &plan).is_empty());
        assert_eq!(
            plan.slot_index.len(),
            plan.retained.len()
                + plan.recomputable.len()
                + plan.excluded.len()
                + plan.blocked.len()
        );
        assert_eq!(
            plan.slots_for_reason(ResumeRetentionReason::MutableState)
                .len(),
            3
        );
        assert_eq!(
            plan.slots_for_reason(ResumeRetentionReason::PureEagerComputedCache)
                .len(),
            2
        );
        let context_slots = plan.slots_for_reason(ResumeRetentionReason::ContextConsumerDependency);
        assert_eq!(context_slots.len(), 1);
        assert!(context_slots.iter().all(|slot| {
            matches!(slot.existing_slot, ResumeExistingSlot::Context(_))
                && slot.direct_dependencies.len() == 1
                && slot.transitive_dependencies.len() == 1
        }));
        assert!(plan
            .retained
            .iter()
            .filter(|record| record.reason == ResumeRetentionReason::FormFieldValue)
            .all(|record| matches!(
                record.slot.existing_slot,
                ResumeExistingSlot::FormFieldValue(_)
            )));
        assert_eq!(
            [
                ResumeRetentionReason::FormFieldValue,
                ResumeRetentionReason::FormFieldDirty,
                ResumeRetentionReason::FormFieldTouched,
                ResumeRetentionReason::FormRuleResult,
                ResumeRetentionReason::FormAggregateValidation,
                ResumeRetentionReason::StableFormSubmission,
            ]
            .into_iter()
            .map(|reason| plan.slots_for_reason(reason).len())
            .sum::<usize>(),
            6
        );
    }

    #[test]
    fn computed_recomputation_carries_direct_and_transitive_exact_slot_evidence() {
        let model = crate::build_application_semantic_model(&presolve_parser::parse_file(
            "src/ComputedLiveness.tsx",
            r#"
@component("x-computed-liveness") class Example {
  count = state(1);
  @computed() get doubled() { return this.count * 2; }
  @computed() get label() { return this.doubled + 1; }
  render() { return <main />; }
}"#,
        ));
        let plan = build_resume_liveness_plan(&model);
        let label = plan
            .recomputable
            .iter()
            .find(|record| {
                record
                    .slot
                    .existing_slot
                    .text()
                    .contains("computed%3Alabel")
            })
            .expect("label cache");
        assert_eq!(label.slot.direct_dependencies.len(), 1);
        assert_eq!(label.slot.transitive_dependencies.len(), 2);
        assert!(label.proof.pure && label.proof.deterministic);
    }

    #[test]
    fn liveness_plan_is_deterministic_and_integrity_rejects_drift() {
        let first = presolve_parser::parse_file(
            "src/A.tsx",
            r#"@component("x-a") class A { value = state(1); render() { return <b>{this.value}</b>; } }"#,
        );
        let second = presolve_parser::parse_file(
            "src/B.tsx",
            r#"@component("x-b") class B { value = state(2); render() { return <i>{this.value}</i>; } }"#,
        );
        let forward = crate::build_application_semantic_model_for_unit(
            &crate::CompilationUnit::from_parsed_files(vec![first.clone(), second.clone()]),
        );
        let reverse = crate::build_application_semantic_model_for_unit(
            &crate::CompilationUnit::from_parsed_files(vec![second, first]),
        );
        assert_eq!(
            build_resume_liveness_plan(&forward),
            build_resume_liveness_plan(&reverse)
        );
        let mut drifted = build_resume_liveness_plan(&forward);
        drifted.slot_index.clear();
        assert!(validate_resume_liveness_plan(&forward, &drifted)
            .iter()
            .any(|diagnostic| {
                diagnostic.code == ResumeLivenessIntegrityCode::ProvenanceOrderIndexDrift
            }));

        let mut invalid_policy = build_resume_liveness_plan(&forward);
        invalid_policy.retained[0].reason = ResumeRetentionReason::FormFieldValue;
        invalid_policy.retained[0].slot.boundary_candidate = None;
        let diagnostics = validate_resume_liveness_plan(&forward, &invalid_policy);
        assert!(diagnostics.iter().any(|diagnostic| {
            diagnostic.code == ResumeLivenessIntegrityCode::InvalidRetentionReason
        }));
        assert!(diagnostics.iter().any(|diagnostic| {
            diagnostic.code == ResumeLivenessIntegrityCode::InvalidCandidatePromotion
        }));
    }

    #[test]
    fn reserves_the_complete_j2_integrity_range() {
        assert_eq!(
            [
                ResumeLivenessIntegrityCode::DuplicateClassification,
                ResumeLivenessIntegrityCode::MissingStorageOwner,
                ResumeLivenessIntegrityCode::UnknownDependency,
                ResumeLivenessIntegrityCode::InvalidRetentionReason,
                ResumeLivenessIntegrityCode::RecomputeWithoutProof,
                ResumeLivenessIntegrityCode::RequiredUnsupportedValue,
                ResumeLivenessIntegrityCode::InvalidCandidatePromotion,
                ResumeLivenessIntegrityCode::ProvenanceOrderIndexDrift,
            ]
            .map(ResumeLivenessIntegrityCode::code),
            [
                "PSASM1320",
                "PSASM1321",
                "PSASM1322",
                "PSASM1323",
                "PSASM1324",
                "PSASM1325",
                "PSASM1326",
                "PSASM1327",
            ]
        );
    }
}
