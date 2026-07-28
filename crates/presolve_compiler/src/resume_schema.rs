//! J6 exact per-boundary resume schemas and closed value codecs.

use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

use crate::{
    build_computed_instance_slot_registry, build_resume_boundary_graph, build_resume_liveness_plan,
    build_runtime_component_registry, build_state_instance_storage_registry,
    lower_components_to_ir, ApplicationSemanticModel, ResumeBoundaryId, ResumeExistingSlot,
    ResumeLivenessPlan, ResumeSchemaId, ResumeSlotId, SemanticType, SourceProvenance,
};

pub const RESUME_SCHEMA_REGISTRY_VERSION: u32 = 1;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct ResumeObjectPropertyCodec {
    pub name: String,
    pub codec: ResumeValueCodec,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(tag = "kind", content = "value", rename_all = "snake_case")]
pub enum ResumeValueCodec {
    NullCodec,
    BooleanCodec,
    NumberCodec,
    StringCodec,
    ArrayCodec(Box<ResumeValueCodec>),
    ObjectCodec(Vec<ResumeObjectPropertyCodec>),
    NullableCodec(Box<ResumeValueCodec>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResumeSlotSchema {
    pub resume_slot_id: ResumeSlotId,
    pub existing_slot: ResumeExistingSlot,
    pub semantic_type: SemanticType,
    pub codec: ResumeValueCodec,
    pub provenance: SourceProvenance,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResumeBoundarySchema {
    pub id: ResumeSchemaId,
    pub boundary: ResumeBoundaryId,
    pub slots: Vec<ResumeSlotSchema>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ResumeSchemaBlockReason {
    UpstreamLivenessBlock,
    MissingCanonicalSlotType,
    MalformedSemanticType,
    UnsupportedValue,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResumeSchemaBlock {
    pub boundary: Option<ResumeBoundaryId>,
    pub slot: ResumeExistingSlot,
    pub reason: ResumeSchemaBlockReason,
    pub semantic_type: Option<SemanticType>,
    pub provenance: SourceProvenance,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResumeSchemaRegistry {
    pub version: u32,
    pub schemas: Vec<ResumeBoundarySchema>,
    pub blocks: Vec<ResumeSchemaBlock>,
    pub schema_index: BTreeMap<ResumeBoundaryId, usize>,
    pub slot_index: BTreeMap<ResumeExistingSlot, ResumeSchemaId>,
}

impl ResumeSchemaRegistry {
    #[must_use]
    pub fn schema(&self, boundary: &ResumeBoundaryId) -> Option<&ResumeBoundarySchema> {
        self.schema_index
            .get(boundary)
            .and_then(|index| self.schemas.get(*index))
    }

    #[must_use]
    pub fn schema_for_slot(&self, slot: &ResumeExistingSlot) -> Option<&ResumeBoundarySchema> {
        self.slot_index.get(slot).and_then(|schema| {
            self.schemas
                .iter()
                .find(|candidate| &candidate.id == schema)
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ResumeSchemaIntegrityCode {
    MalformedSemanticType,
    DuplicateProperty,
    UnsupportedValue,
    MissingSlot,
    IdentityCollision,
    OrderingOrIndexDrift,
}

impl ResumeSchemaIntegrityCode {
    #[must_use]
    pub const fn code(self) -> &'static str {
        match self {
            Self::MalformedSemanticType => "PSASM1349",
            Self::DuplicateProperty => "PSASM1350",
            Self::UnsupportedValue => "PSASM1351",
            Self::MissingSlot => "PSASM1352",
            Self::IdentityCollision => "PSASM1353",
            Self::OrderingOrIndexDrift => "PSASM1354",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResumeSchemaIntegrityDiagnostic {
    pub code: ResumeSchemaIntegrityCode,
    pub boundary: Option<ResumeBoundaryId>,
    pub slot: Option<ResumeExistingSlot>,
    pub message: String,
}

/// # Errors
///
/// Returns a closed schema-block reason when the semantic type is malformed or
/// cannot be represented by the J6 codec vocabulary.
pub fn resume_value_codec(
    semantic_type: &SemanticType,
) -> Result<ResumeValueCodec, ResumeSchemaBlockReason> {
    match semantic_type {
        SemanticType::Null => Ok(ResumeValueCodec::NullCodec),
        SemanticType::Boolean | SemanticType::BooleanLiteral(_) => {
            Ok(ResumeValueCodec::BooleanCodec)
        }
        SemanticType::Number | SemanticType::NumberLiteral(_) => Ok(ResumeValueCodec::NumberCodec),
        SemanticType::String | SemanticType::StringLiteral(_) => Ok(ResumeValueCodec::StringCodec),
        SemanticType::Array(element) => resume_value_codec(element)
            .map(Box::new)
            .map(ResumeValueCodec::ArrayCodec),
        SemanticType::Object(object) => object
            .properties
            .iter()
            .map(|(name, property)| {
                Ok(ResumeObjectPropertyCodec {
                    name: name.clone(),
                    codec: resume_value_codec(property)?,
                })
            })
            .collect::<Result<Vec<_>, ResumeSchemaBlockReason>>()
            .map(ResumeValueCodec::ObjectCodec),
        SemanticType::Union(members) => nullable_codec(members),
        SemanticType::Unknown
        | SemanticType::Never
        | SemanticType::Form
        | SemanticType::SlotContent => Err(ResumeSchemaBlockReason::MalformedSemanticType),
        SemanticType::File => Err(ResumeSchemaBlockReason::UnsupportedValue),
        SemanticType::Tuple(_) | SemanticType::Resource(_) => {
            Err(ResumeSchemaBlockReason::UnsupportedValue)
        }
    }
}

fn nullable_codec(members: &[SemanticType]) -> Result<ResumeValueCodec, ResumeSchemaBlockReason> {
    if members.is_empty() {
        return Err(ResumeSchemaBlockReason::MalformedSemanticType);
    }
    let non_null = members
        .iter()
        .filter(|member| !matches!(member, SemanticType::Null))
        .collect::<Vec<_>>();
    let null_count = members.len() - non_null.len();
    if null_count != 1 || non_null.len() != 1 {
        return Err(ResumeSchemaBlockReason::UnsupportedValue);
    }
    resume_value_codec(non_null[0])
        .map(Box::new)
        .map(ResumeValueCodec::NullableCodec)
}

#[must_use]
#[allow(clippy::too_many_lines)]
pub fn build_resume_schema_registry(model: &ApplicationSemanticModel) -> ResumeSchemaRegistry {
    let boundaries = build_resume_boundary_graph(model);
    let liveness = build_resume_liveness_plan(model);
    let slot_types = ResumeSlotTypeAuthority::new(model);
    let mut schemas = boundaries
        .boundaries
        .iter()
        .map(|boundary| ResumeBoundarySchema {
            id: ResumeSchemaId::for_boundary(&boundary.id),
            boundary: boundary.id.clone(),
            slots: Vec::new(),
        })
        .collect::<Vec<_>>();
    let schema_index = schemas
        .iter()
        .enumerate()
        .map(|(index, schema)| (schema.boundary.clone(), index))
        .collect::<BTreeMap<_, _>>();
    let mut blocks = Vec::new();

    for slot in capture_eligible_slots(&liveness) {
        let boundary = slot.boundary_candidate.clone();
        let Some(boundary_id) = boundary.as_ref() else {
            blocks.push(ResumeSchemaBlock {
                boundary,
                slot: slot.existing_slot.clone(),
                reason: ResumeSchemaBlockReason::MissingCanonicalSlotType,
                semantic_type: None,
                provenance: slot.provenance.clone(),
            });
            continue;
        };
        let Some(index) = schema_index.get(boundary_id).copied() else {
            blocks.push(ResumeSchemaBlock {
                boundary,
                slot: slot.existing_slot.clone(),
                reason: ResumeSchemaBlockReason::MissingCanonicalSlotType,
                semantic_type: None,
                provenance: slot.provenance.clone(),
            });
            continue;
        };
        let Some(semantic_type) = slot_types.semantic_type(&slot.existing_slot) else {
            blocks.push(ResumeSchemaBlock {
                boundary: Some(boundary_id.clone()),
                slot: slot.existing_slot.clone(),
                reason: ResumeSchemaBlockReason::MissingCanonicalSlotType,
                semantic_type: None,
                provenance: slot.provenance.clone(),
            });
            continue;
        };
        match resume_value_codec(&semantic_type) {
            Ok(codec) => schemas[index].slots.push(ResumeSlotSchema {
                resume_slot_id: slot.resume_slot_id.clone(),
                existing_slot: slot.existing_slot.clone(),
                semantic_type,
                codec,
                provenance: slot.provenance.clone(),
            }),
            Err(reason) => blocks.push(ResumeSchemaBlock {
                boundary: Some(boundary_id.clone()),
                slot: slot.existing_slot.clone(),
                reason,
                semantic_type: Some(semantic_type),
                provenance: slot.provenance.clone(),
            }),
        }
    }

    for blocked in &liveness.blocked {
        blocks.push(ResumeSchemaBlock {
            boundary: blocked.slot.boundary_candidate.clone(),
            slot: blocked.slot.existing_slot.clone(),
            reason: ResumeSchemaBlockReason::UpstreamLivenessBlock,
            semantic_type: slot_types.semantic_type(&blocked.slot.existing_slot),
            provenance: blocked.slot.provenance.clone(),
        });
    }

    for schema in &mut schemas {
        schema.slots.sort_by(|left, right| {
            (&left.resume_slot_id, &left.existing_slot)
                .cmp(&(&right.resume_slot_id, &right.existing_slot))
        });
    }
    blocks.sort_by(|left, right| {
        (&left.boundary, &left.slot, left.reason).cmp(&(&right.boundary, &right.slot, right.reason))
    });
    let slot_index = schemas
        .iter()
        .flat_map(|schema| {
            schema
                .slots
                .iter()
                .map(|slot| (slot.existing_slot.clone(), schema.id.clone()))
        })
        .collect();

    ResumeSchemaRegistry {
        version: RESUME_SCHEMA_REGISTRY_VERSION,
        schemas,
        blocks,
        schema_index,
        slot_index,
    }
}

fn capture_eligible_slots(liveness: &ResumeLivenessPlan) -> Vec<&crate::ResumeLivenessSlot> {
    let mut slots = liveness
        .retained
        .iter()
        .map(|record| &record.slot)
        .chain(liveness.recomputable.iter().map(|record| &record.slot))
        .collect::<Vec<_>>();
    slots.sort_by(|left, right| left.existing_slot.cmp(&right.existing_slot));
    slots
}

struct ResumeSlotTypeAuthority {
    types: BTreeMap<ResumeExistingSlot, SemanticType>,
}

impl ResumeSlotTypeAuthority {
    fn new(model: &ApplicationSemanticModel) -> Self {
        let ir = lower_components_to_ir(model);
        let mut types = BTreeMap::new();
        for record in build_state_instance_storage_registry(model, &ir).records {
            types.insert(
                ResumeExistingSlot::State(record.slot_id),
                record.semantic_type,
            );
        }
        for activation in model.resource_activations.values() {
            let Some(declaration) = model.resource_declarations.get(&activation.declaration) else {
                continue;
            };
            let snapshot_policy = model
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
                });
            if !snapshot_policy {
                continue;
            }
            types.insert(
                ResumeExistingSlot::ResourceState(activation.id.state_slot()),
                SemanticType::Object(crate::ObjectType {
                    properties: BTreeMap::from([
                        ("generation".to_string(), SemanticType::Number),
                        ("state".to_string(), SemanticType::String),
                    ]),
                }),
            );
            types.insert(
                ResumeExistingSlot::ResourceData(activation.id.data_slot()),
                SemanticType::Union(vec![SemanticType::Null, declaration.data_type.clone()]),
            );
            types.insert(
                ResumeExistingSlot::ResourceError(activation.id.error_slot()),
                SemanticType::Union(vec![SemanticType::Null, declaration.error_type.clone()]),
            );
        }
        for record in build_computed_instance_slot_registry(model, &ir).records {
            if let Some(semantic_type) = model.semantic_type_of(&record.computed_id) {
                types.insert(
                    ResumeExistingSlot::ComputedCache(record.cache_slot_id),
                    semantic_type.clone(),
                );
            }
            types.insert(
                ResumeExistingSlot::ComputedDirty(record.dirty_slot_id),
                SemanticType::Boolean,
            );
        }
        let runtime_components =
            build_runtime_component_registry(model, &model.component_ir_optimization);
        for binding in runtime_components.instance_context_bindings {
            if let Some(semantic_type) =
                model.semantic_type_of(binding.selected_source.context.as_semantic_id())
            {
                types.insert(
                    ResumeExistingSlot::Context(binding.runtime_slot),
                    semantic_type.clone(),
                );
            }
        }
        for instance in model.optimized_form_ir.optimized.instances.values() {
            for (field_id, slot) in &instance.storage.value {
                if let Some(field) = model.form_fields.get(field_id) {
                    types.insert(
                        ResumeExistingSlot::FormFieldValue(slot.clone()),
                        field.semantic_type.clone(),
                    );
                }
            }
            for slot in instance.storage.dirty.values() {
                types.insert(
                    ResumeExistingSlot::FormFieldDirty(slot.clone()),
                    SemanticType::Boolean,
                );
            }
            for slot in instance.storage.touched.values() {
                types.insert(
                    ResumeExistingSlot::FormFieldTouched(slot.clone()),
                    SemanticType::Boolean,
                );
            }
            for slot in instance.storage.validation.values() {
                types.insert(
                    ResumeExistingSlot::FormFieldValidation(slot.clone()),
                    SemanticType::Array(Box::new(SemanticType::String)),
                );
            }
            types.insert(
                ResumeExistingSlot::FormValidationAggregate(instance.storage.aggregate.clone()),
                SemanticType::Boolean,
            );
            types.insert(
                ResumeExistingSlot::FormSubmission(instance.storage.submission.clone()),
                SemanticType::String,
            );
        }
        Self { types }
    }

    fn semantic_type(&self, slot: &ResumeExistingSlot) -> Option<SemanticType> {
        self.types.get(slot).cloned()
    }
}

/// # Errors
///
/// Returns exact J6 integrity diagnostics for malformed codecs, missing schema
/// reciprocity, identity collisions, or canonical ordering/index drift.
pub fn validate_resume_schema_registry(
    model: &ApplicationSemanticModel,
    registry: &ResumeSchemaRegistry,
) -> Result<(), Vec<ResumeSchemaIntegrityDiagnostic>> {
    let mut diagnostics = Vec::new();
    let boundaries = build_resume_boundary_graph(model);
    let liveness = build_resume_liveness_plan(model);
    let expected_boundary_order = boundaries
        .boundaries
        .iter()
        .map(|boundary| boundary.id.clone())
        .collect::<Vec<_>>();
    let actual_boundary_order = registry
        .schemas
        .iter()
        .map(|schema| schema.boundary.clone())
        .collect::<Vec<_>>();
    if expected_boundary_order != actual_boundary_order {
        diagnostics.push(diagnostic(
            ResumeSchemaIntegrityCode::OrderingOrIndexDrift,
            None,
            None,
            "resume schemas do not preserve canonical parent-before-child boundary order",
        ));
    }

    let mut schema_ids = BTreeSet::new();
    let mut resume_slot_ids = BTreeSet::new();
    let mut existing_slots = BTreeSet::new();
    for schema in &registry.schemas {
        if schema.id != ResumeSchemaId::for_boundary(&schema.boundary)
            || !schema_ids.insert(schema.id.clone())
        {
            diagnostics.push(diagnostic(
                ResumeSchemaIntegrityCode::IdentityCollision,
                Some(schema.boundary.clone()),
                None,
                "resume schema identity is not unique and boundary-derived",
            ));
        }
        for slot in &schema.slots {
            if slot.resume_slot_id != slot.existing_slot.resume_slot_id()
                || !resume_slot_ids.insert(slot.resume_slot_id.clone())
                || !existing_slots.insert(slot.existing_slot.clone())
            {
                diagnostics.push(diagnostic(
                    ResumeSchemaIntegrityCode::IdentityCollision,
                    Some(schema.boundary.clone()),
                    Some(slot.existing_slot.clone()),
                    "resume slot identity is not unique and storage-derived",
                ));
            }
            validate_codec(
                &slot.codec,
                &schema.boundary,
                &slot.existing_slot,
                &mut diagnostics,
            );
        }
    }

    let expected_slots = capture_eligible_slots(&liveness)
        .into_iter()
        .map(|slot| slot.existing_slot.clone())
        .collect::<BTreeSet<_>>();
    let blocked_slots = registry
        .blocks
        .iter()
        .map(|block| block.slot.clone())
        .collect::<BTreeSet<_>>();
    if !expected_slots
        .iter()
        .all(|slot| existing_slots.contains(slot) || blocked_slots.contains(slot))
    {
        diagnostics.push(diagnostic(
            ResumeSchemaIntegrityCode::MissingSlot,
            None,
            None,
            "capture-eligible liveness slot lacks a schema or explicit schema block",
        ));
    }
    if registry.version != RESUME_SCHEMA_REGISTRY_VERSION
        || registry != &build_resume_schema_registry(model)
    {
        diagnostics.push(diagnostic(
            ResumeSchemaIntegrityCode::OrderingOrIndexDrift,
            None,
            None,
            "resume schema registry drifted from canonical ordering or indexes",
        ));
    }

    if diagnostics.is_empty() {
        Ok(())
    } else {
        Err(diagnostics)
    }
}

fn validate_codec(
    codec: &ResumeValueCodec,
    boundary: &ResumeBoundaryId,
    slot: &ResumeExistingSlot,
    diagnostics: &mut Vec<ResumeSchemaIntegrityDiagnostic>,
) {
    match codec {
        ResumeValueCodec::ArrayCodec(element) | ResumeValueCodec::NullableCodec(element) => {
            validate_codec(element, boundary, slot, diagnostics);
        }
        ResumeValueCodec::ObjectCodec(properties) => {
            let names = properties
                .iter()
                .map(|property| property.name.as_str())
                .collect::<Vec<_>>();
            if names.windows(2).any(|pair| pair[0] == pair[1]) {
                diagnostics.push(diagnostic(
                    ResumeSchemaIntegrityCode::DuplicateProperty,
                    Some(boundary.clone()),
                    Some(slot.clone()),
                    "resume object codec contains a duplicate property",
                ));
            }
            if names.windows(2).any(|pair| pair[0] > pair[1]) {
                diagnostics.push(diagnostic(
                    ResumeSchemaIntegrityCode::OrderingOrIndexDrift,
                    Some(boundary.clone()),
                    Some(slot.clone()),
                    "resume object codec properties are not in canonical order",
                ));
            }
            for property in properties {
                validate_codec(&property.codec, boundary, slot, diagnostics);
            }
        }
        ResumeValueCodec::NullCodec
        | ResumeValueCodec::BooleanCodec
        | ResumeValueCodec::NumberCodec
        | ResumeValueCodec::StringCodec => {}
    }
}

fn diagnostic(
    code: ResumeSchemaIntegrityCode,
    boundary: Option<ResumeBoundaryId>,
    slot: Option<ResumeExistingSlot>,
    message: &str,
) -> ResumeSchemaIntegrityDiagnostic {
    ResumeSchemaIntegrityDiagnostic {
        code,
        boundary,
        slot,
        message: message.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        build_application_semantic_model_for_unit_with_packages, parse_semantic_package_contract,
        CompilationUnit, ObjectType, SemanticPackageResolutionTable,
    };

    fn model(source: &str) -> ApplicationSemanticModel {
        crate::build_application_semantic_model(&presolve_parser::parse_file(
            "src/ResumeSchema.tsx",
            source,
        ))
    }

    fn resource_model(resume_policy: &str) -> ApplicationSemanticModel {
        let unit = CompilationUnit::parse_sources([(
            "src/Profile.tsx",
            r#"
import { loadProfile } from "profile-service";
@component("x-profile") @route("/") class Profile extends Component {
  @resource("loadProfile") profile!: Resource<string, string>;
  @computed() get profileName(): string | null { return this.profile.data; }
  render() { return <main>{this.profileName}</main>; }
}
"#,
        )]);
        let contract = parse_semantic_package_contract(&format!(
            r#"{{"schema_version":1,"package":"profile-service","version":"1.2.3","integrity":"sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","exports":{{"loadProfile":{{"kind":"resource","type_signature":"() -> Resource<string, string>","runtime_module":"dist/load-profile.js","resume_policy":"{resume_policy}","resource_endpoint":{{"execution_boundary":"shared","cancellation":"abort","resume":"{resume_policy}"}}}}}}}}"#
        ))
        .expect("resource contract");
        let mut packages = SemanticPackageResolutionTable::default();
        packages.insert("profile-service".into(), contract).unwrap();
        build_application_semantic_model_for_unit_with_packages(&unit, &packages)
    }

    #[test]
    fn derives_closed_codecs_with_canonical_object_order_and_explicit_nullable() {
        let object = SemanticType::Object(ObjectType {
            properties: BTreeMap::from([
                ("b".to_string(), SemanticType::String),
                ("a".to_string(), SemanticType::Number),
            ]),
        });
        assert_eq!(
            resume_value_codec(&object),
            Ok(ResumeValueCodec::ObjectCodec(vec![
                ResumeObjectPropertyCodec {
                    name: "a".to_string(),
                    codec: ResumeValueCodec::NumberCodec,
                },
                ResumeObjectPropertyCodec {
                    name: "b".to_string(),
                    codec: ResumeValueCodec::StringCodec,
                },
            ]))
        );
        assert_eq!(
            resume_value_codec(&SemanticType::Union(vec![
                SemanticType::Null,
                SemanticType::String,
            ])),
            Ok(ResumeValueCodec::NullableCodec(Box::new(
                ResumeValueCodec::StringCodec
            )))
        );
        assert_eq!(
            resume_value_codec(&SemanticType::Union(vec![
                SemanticType::String,
                SemanticType::Number,
            ])),
            Err(ResumeSchemaBlockReason::UnsupportedValue)
        );
    }

    #[test]
    fn creates_one_schema_per_boundary_with_exact_slot_reciprocity() {
        let model = model(
            r#"@component("x-counter") class Counter { count = state(1); render() { return <button>{this.count}</button>; } }"#,
        );
        let graph = build_resume_boundary_graph(&model);
        let liveness = build_resume_liveness_plan(&model);
        let registry = build_resume_schema_registry(&model);
        assert_eq!(registry.schemas.len(), graph.boundaries.len());
        assert_eq!(
            registry
                .schemas
                .iter()
                .map(|schema| &schema.boundary)
                .collect::<Vec<_>>(),
            graph
                .boundaries
                .iter()
                .map(|boundary| &boundary.id)
                .collect::<Vec<_>>()
        );
        let expected = capture_eligible_slots(&liveness)
            .into_iter()
            .map(|slot| slot.existing_slot.clone())
            .collect::<BTreeSet<_>>();
        let actual = registry.slot_index.keys().cloned().collect::<BTreeSet<_>>();
        assert_eq!(actual, expected);
        assert!(registry.blocks.is_empty());
        assert_eq!(validate_resume_schema_registry(&model, &registry), Ok(()));
    }

    #[test]
    fn freezes_form_runtime_slot_codecs_without_runtime_guessing() {
        let model = model(
            r#"@component("x-profile") class Profile {
  @form() profile!: Form;
  @field(this.profile) name = "";
  render() { return <input field={this.name} />; }
}"#,
        );
        let registry = build_resume_schema_registry(&model);
        let codecs = registry
            .schemas
            .iter()
            .flat_map(|schema| &schema.slots)
            .map(|slot| (&slot.existing_slot, &slot.codec))
            .collect::<BTreeMap<_, _>>();
        assert!(codecs.iter().any(|(slot, codec)| matches!(
            (slot, codec),
            (
                ResumeExistingSlot::FormFieldValidation(_),
                ResumeValueCodec::ArrayCodec(element)
            ) if **element == ResumeValueCodec::StringCodec
        )));
        assert!(codecs.iter().any(|(slot, codec)| matches!(
            (slot, codec),
            (
                ResumeExistingSlot::FormValidationAggregate(_),
                ResumeValueCodec::BooleanCodec
            )
        )));
        assert!(codecs.iter().any(|(slot, codec)| matches!(
            (slot, codec),
            (
                ResumeExistingSlot::FormSubmission(_),
                ResumeValueCodec::StringCodec
            )
        )));
    }

    #[test]
    fn snapshot_resources_publish_instance_qualified_terminal_slots_before_computed_recompute() {
        let model = resource_model("snapshot");
        let liveness = build_resume_liveness_plan(&model);
        let resource_slots =
            liveness.slots_for_reason(crate::ResumeRetentionReason::ResourceSnapshotValue);
        assert_eq!(resource_slots.len(), 3);
        assert!(resource_slots.iter().all(|slot| matches!(
            slot.existing_slot,
            ResumeExistingSlot::ResourceState(_)
                | ResumeExistingSlot::ResourceData(_)
                | ResumeExistingSlot::ResourceError(_)
        )));
        let computed = liveness
            .recomputable
            .iter()
            .find(|record| {
                matches!(
                    record.slot.existing_slot,
                    ResumeExistingSlot::ComputedCache(_)
                )
            })
            .expect("profile computed cache");
        assert!(matches!(
            computed.slot.direct_dependencies.as_slice(),
            [ResumeExistingSlot::ResourceData(_)]
        ));

        let registry = build_resume_schema_registry(&model);
        let codecs = registry
            .schemas
            .iter()
            .flat_map(|schema| &schema.slots)
            .map(|slot| (&slot.existing_slot, &slot.codec))
            .collect::<BTreeMap<_, _>>();
        assert!(codecs.iter().any(|(slot, codec)| matches!(
            (slot, codec),
            (
                ResumeExistingSlot::ResourceState(_),
                ResumeValueCodec::ObjectCodec(properties)
            ) if properties == &vec![
                ResumeObjectPropertyCodec {
                    name: "generation".to_string(),
                    codec: ResumeValueCodec::NumberCodec,
                },
                ResumeObjectPropertyCodec {
                    name: "state".to_string(),
                    codec: ResumeValueCodec::StringCodec,
                },
            ]
        )));
        assert_eq!(
            codecs
                .iter()
                .filter(|(slot, codec)| matches!(
                    (slot, codec),
                    (ResumeExistingSlot::ResourceData(_), ResumeValueCodec::NullableCodec(inner))
                        if **inner == ResumeValueCodec::StringCodec
                ) || matches!(
                    (slot, codec),
                    (ResumeExistingSlot::ResourceError(_), ResumeValueCodec::NullableCodec(inner))
                        if **inner == ResumeValueCodec::StringCodec
                ))
                .count(),
            2
        );
        let resource_slot_ids = registry
            .schemas
            .iter()
            .flat_map(|schema| &schema.slots)
            .filter(|slot| {
                matches!(
                    slot.existing_slot,
                    ResumeExistingSlot::ResourceState(_)
                        | ResumeExistingSlot::ResourceData(_)
                        | ResumeExistingSlot::ResourceError(_)
                )
            })
            .map(|slot| slot.resume_slot_id.clone())
            .collect::<BTreeSet<_>>();
        let capture = crate::build_resume_capture_plan(&model);
        let captured = capture
            .programs
            .iter()
            .flat_map(|program| &program.instructions)
            .filter_map(|instruction| match instruction {
                crate::ResumeCaptureInstruction::ReadSlot { slot_id } => Some(slot_id.clone()),
                _ => None,
            })
            .collect::<BTreeSet<_>>();
        assert!(resource_slot_ids.is_subset(&captured));
        let restore = crate::build_resume_restore_plan(&model);
        assert_eq!(
            restore
                .programs
                .iter()
                .flat_map(|program| &program.slot_assignments)
                .filter(|assignment| matches!(
                    assignment.slot,
                    ResumeExistingSlot::ResourceState(_)
                        | ResumeExistingSlot::ResourceData(_)
                        | ResumeExistingSlot::ResourceError(_)
                ))
                .map(|assignment| assignment.phase)
                .collect::<BTreeSet<_>>(),
            BTreeSet::from([crate::ResumeRestorePhase::R3RestoreMutableStateAndResources])
        );
        let manifest = crate::build_resume_manifest(&model);
        assert_eq!(
            manifest
                .slot_schemas
                .iter()
                .filter(|slot| matches!(
                    slot.retention_reason,
                    crate::resume_manifest::ResumeManifestRetentionReason::SerializableResourceValue
                ))
                .count(),
            3
        );
        assert!(validate_resume_schema_registry(&model, &registry).is_ok());
    }

    #[test]
    fn reload_resources_do_not_publish_snapshot_slots() {
        let model = resource_model("reload");
        let liveness = build_resume_liveness_plan(&model);
        assert!(liveness
            .slots_for_reason(crate::ResumeRetentionReason::ResourceSnapshotValue)
            .is_empty());
        assert!(build_resume_schema_registry(&model)
            .schemas
            .iter()
            .flat_map(|schema| &schema.slots)
            .all(|slot| !matches!(
                slot.existing_slot,
                ResumeExistingSlot::ResourceState(_)
                    | ResumeExistingSlot::ResourceData(_)
                    | ResumeExistingSlot::ResourceError(_)
            )));
    }

    #[test]
    fn integrity_rejects_identity_and_ordering_drift() {
        let model = model(
            r#"@component("x-counter") class Counter { count = state(1); render() { return <button>{this.count}</button>; } }"#,
        );
        let mut registry = build_resume_schema_registry(&model);
        registry.schemas.reverse();
        let diagnostics = validate_resume_schema_registry(&model, &registry).expect_err("drift");
        assert!(diagnostics.iter().any(|diagnostic| {
            diagnostic.code == ResumeSchemaIntegrityCode::OrderingOrIndexDrift
        }));
    }

    #[test]
    fn schema_registry_is_deterministic_under_source_reversal() {
        let first = presolve_parser::parse_file(
            "src/A.tsx",
            r#"@component("x-a") @route("/a") class A { value = state({ b: "two", a: 1 }); render() { return <button>{this.value}</button>; } }"#,
        );
        let second = presolve_parser::parse_file(
            "src/B.tsx",
            r#"@component("x-b") @route("/b") class B { enabled = state(true); render() { return <button>{this.enabled}</button>; } }"#,
        );
        let forward = crate::build_application_semantic_model_for_unit(
            &crate::CompilationUnit::from_parsed_files(vec![first.clone(), second.clone()]),
        );
        let reverse = crate::build_application_semantic_model_for_unit(
            &crate::CompilationUnit::from_parsed_files(vec![second, first]),
        );
        assert_eq!(
            build_resume_schema_registry(&forward),
            build_resume_schema_registry(&reverse)
        );
    }

    #[test]
    fn schema_order_keeps_nested_boundaries_parent_before_child() {
        let model = model(
            r#"@component("x-child") class Child { value = state(1); render() { return <span>{this.value}</span>; } }
@component("x-parent") @route("/") class Parent { render() { return <Child />; } }"#,
        );
        let graph = build_resume_boundary_graph(&model);
        let registry = build_resume_schema_registry(&model);
        assert_eq!(
            registry
                .schemas
                .iter()
                .map(|schema| &schema.boundary)
                .collect::<Vec<_>>(),
            graph
                .boundaries
                .iter()
                .map(|boundary| &boundary.id)
                .collect::<Vec<_>>()
        );
    }

    #[test]
    fn reserves_the_complete_j6_integrity_range() {
        assert_eq!(
            [
                ResumeSchemaIntegrityCode::MalformedSemanticType,
                ResumeSchemaIntegrityCode::DuplicateProperty,
                ResumeSchemaIntegrityCode::UnsupportedValue,
                ResumeSchemaIntegrityCode::MissingSlot,
                ResumeSchemaIntegrityCode::IdentityCollision,
                ResumeSchemaIntegrityCode::OrderingOrIndexDrift,
            ]
            .map(ResumeSchemaIntegrityCode::code),
            [
                "PSASM1349",
                "PSASM1350",
                "PSASM1351",
                "PSASM1352",
                "PSASM1353",
                "PSASM1354",
            ]
        );
    }
}
