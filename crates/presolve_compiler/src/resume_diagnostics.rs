use crate::resume_instance::SerializableInstance;
use crate::resume_plan::ResumePlan;
use crate::semantic_id::SemanticId;
use crate::{
    build_resume_activation_plan, build_resume_anchor_plan, build_resume_boundary_graph,
    build_resume_capture_plan, build_resume_chunk_graph, build_resume_liveness_plan,
    build_resume_restore_plan, build_resume_schema_registry, validate_resume_activation_plan,
    validate_resume_anchor_plan, validate_resume_boundary_graph, validate_resume_capture_plan,
    validate_resume_chunk_graph, validate_resume_liveness_plan, validate_resume_restore_plan,
    validate_resume_schema_registry, ApplicationSemanticModel, ResumeActivationBlockReason,
    ResumeActivationIntegrityCode, ResumeActivationPlan, ResumeAnchorIntegrityCode,
    ResumeAnchorPlan, ResumeBoundaryGraph, ResumeBoundaryIntegrityCode, ResumeCaptureBlockReason,
    ResumeCaptureIntegrityCode, ResumeCapturePlan, ResumeChunkBlockReason, ResumeChunkGraph,
    ResumeChunkIntegrityCode, ResumeLivenessBlockReason, ResumeLivenessIntegrityCode,
    ResumeLivenessPlan, ResumeRestoreBlockReason, ResumeRestoreIntegrityCode, ResumeRestorePlan,
    ResumeSchemaBlockReason, ResumeSchemaIntegrityCode, ResumeSchemaRegistry, SourceProvenance,
};

/// A Phase J diagnostic code reserved before executable resumability products
/// are introduced. J19 alone projects these from immutable Phase J products.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ResumeDiagnosticReservation {
    pub code: &'static str,
    pub meaning: &'static str,
}

/// The contiguous public compiler diagnostic range reserved by J0.
pub const RESUME_DIAGNOSTIC_RESERVATIONS: [ResumeDiagnosticReservation; 16] = [
    ResumeDiagnosticReservation {
        code: "PSC1096",
        meaning: "Unsupported resume value",
    },
    ResumeDiagnosticReservation {
        code: "PSC1097",
        meaning: "Missing resume owner",
    },
    ResumeDiagnosticReservation {
        code: "PSC1098",
        meaning: "Resume boundary cycle",
    },
    ResumeDiagnosticReservation {
        code: "PSC1099",
        meaning: "Invalid resume retention",
    },
    ResumeDiagnosticReservation {
        code: "PSC1100",
        meaning: "Invalid resume recomputation",
    },
    ResumeDiagnosticReservation {
        code: "PSC1101",
        meaning: "Invalid activation policy",
    },
    ResumeDiagnosticReservation {
        code: "PSC1102",
        meaning: "Resume chunk cycle",
    },
    ResumeDiagnosticReservation {
        code: "PSC1103",
        meaning: "Missing resume program",
    },
    ResumeDiagnosticReservation {
        code: "PSC1104",
        meaning: "Invalid resume anchor",
    },
    ResumeDiagnosticReservation {
        code: "PSC1105",
        meaning: "Resume schema collision",
    },
    ResumeDiagnosticReservation {
        code: "PSC1106",
        meaning: "Invalid snapshot stable state",
    },
    ResumeDiagnosticReservation {
        code: "PSC1107",
        meaning: "Resume artifact mismatch",
    },
    ResumeDiagnosticReservation {
        code: "PSC1108",
        meaning: "Lazy event payload unsupported",
    },
    ResumeDiagnosticReservation {
        code: "PSC1109",
        meaning: "Missing resume chunk",
    },
    ResumeDiagnosticReservation {
        code: "PSC1110",
        meaning: "Invalid resume ordering",
    },
    ResumeDiagnosticReservation {
        code: "PSC1111",
        meaning: "Unsupported resume topology",
    },
];

/// J1-J21 must allocate internal integrity codes only within this J0-reserved range.
pub const RESUME_INTEGRITY_RESERVATION_START: u32 = 1289;
pub const RESUME_INTEGRITY_RESERVATION_END: u32 = 1384;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResumeDiagnostic {
    pub code: String,
    pub component: SemanticId,
    pub state: Option<SemanticId>,
}

/// Public J19 projection from immutable Phase J products. These records do not
/// manufacture semantic identities or re-read source/DOM output.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResumeProjectedDiagnostic {
    pub code: &'static str,
    pub message: String,
    /// The exact Phase J product identity when an earlier product established
    /// one. Unresolved candidates deliberately retain no fabricated identity.
    pub primary_identity: Option<String>,
    pub primary_provenance: SourceProvenance,
}

/// The immutable Phase J products that authorize J19 public diagnostics.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResumeDiagnosticProducts {
    pub liveness: ResumeLivenessPlan,
    pub boundaries: ResumeBoundaryGraph,
    pub activation: ResumeActivationPlan,
    pub chunks: ResumeChunkGraph,
    pub schemas: ResumeSchemaRegistry,
    pub capture: ResumeCapturePlan,
    pub restore: ResumeRestorePlan,
    pub anchors: ResumeAnchorPlan,
}

#[must_use]
pub fn build_resume_diagnostic_products(
    model: &ApplicationSemanticModel,
) -> ResumeDiagnosticProducts {
    ResumeDiagnosticProducts {
        liveness: build_resume_liveness_plan(model),
        boundaries: build_resume_boundary_graph(model),
        activation: build_resume_activation_plan(model),
        chunks: build_resume_chunk_graph(model),
        schemas: build_resume_schema_registry(model),
        capture: build_resume_capture_plan(model),
        restore: build_resume_restore_plan(model),
        anchors: build_resume_anchor_plan(model),
    }
}

#[must_use]
#[allow(clippy::too_many_lines)]
pub fn project_resume_diagnostics(
    model: &ApplicationSemanticModel,
) -> Vec<ResumeProjectedDiagnostic> {
    // Earlier compiler diagnostics mean the required canonical products were
    // never established. Public J19 diagnostics must not invent Phase J
    // identities while reporting the earlier failure.
    if !model.diagnostics.is_empty() {
        return Vec::new();
    }
    project_resume_diagnostics_from_products(model, &build_resume_diagnostic_products(model))
}

/// Projects public J19 records only from already-established Phase J products.
#[must_use]
#[allow(clippy::too_many_lines)]
pub fn project_resume_diagnostics_from_products(
    model: &ApplicationSemanticModel,
    products: &ResumeDiagnosticProducts,
) -> Vec<ResumeProjectedDiagnostic> {
    if !model.diagnostics.is_empty() {
        return Vec::new();
    }
    let mut projected = Vec::new();
    let liveness = &products.liveness;
    for block in &liveness.blocked {
        let code = match block.reason {
            ResumeLivenessBlockReason::RequiredNonSerializableValue => "PSC1096",
            ResumeLivenessBlockReason::RecomputeProofUnavailable => "PSC1100",
            ResumeLivenessBlockReason::MissingCanonicalSource => "PSC1097",
            ResumeLivenessBlockReason::UnknownDependency => "PSC1099",
        };
        projected.push(ResumeProjectedDiagnostic {
            code,
            message: format!(
                "{}: {}",
                resume_diagnostic_meaning(code),
                block.slot.resume_slot_id
            ),
            primary_identity: Some(block.slot.resume_slot_id.to_string()),
            primary_provenance: block.slot.provenance.clone(),
        });
    }
    let schemas = &products.schemas;
    for block in &schemas.blocks {
        let code = match block.reason {
            ResumeSchemaBlockReason::UnsupportedValue => "PSC1096",
            ResumeSchemaBlockReason::UpstreamLivenessBlock => "PSC1099",
            ResumeSchemaBlockReason::MissingCanonicalSlotType
            | ResumeSchemaBlockReason::MalformedSemanticType => "PSC1107",
        };
        projected.push(ResumeProjectedDiagnostic {
            code,
            message: format!("{}: {:?}", resume_diagnostic_meaning(code), block.slot),
            primary_identity: Some(block.slot.resume_slot_id().to_string()),
            primary_provenance: block.provenance.clone(),
        });
    }
    let boundaries = &products.boundaries;
    for block in &boundaries.blocks {
        projected.push(ResumeProjectedDiagnostic {
            code: "PSC1111",
            message: format!(
                "Unsupported resume topology: {:?}",
                block.candidate_boundary
            ),
            primary_identity: block.candidate_boundary.as_ref().map(ToString::to_string),
            primary_provenance: block.provenance.clone(),
        });
    }
    let activation = &products.activation;
    for block in &activation.blocks {
        let code = match block.reason {
            ResumeActivationBlockReason::MissingInteractionReference => "PSC1109",
            ResumeActivationBlockReason::UnknownBoundary
            | ResumeActivationBlockReason::RequiredBoundaryBlocked
            | ResumeActivationBlockReason::NoValidEagerFallback => "PSC1101",
            ResumeActivationBlockReason::UnsupportedLazyEventPayload => "PSC1108",
        };
        projected.push(ResumeProjectedDiagnostic {
            code,
            message: format!("{}: {}", resume_diagnostic_meaning(code), block.boundary),
            primary_identity: Some(block.boundary.to_string()),
            primary_provenance: block.provenance.clone(),
        });
    }
    let chunks = &products.chunks;
    for block in &chunks.blocks {
        let code = match block.reason {
            ResumeChunkBlockReason::MissingActivationBoundary => "PSC1109",
            ResumeChunkBlockReason::MissingProgram => "PSC1103",
            ResumeChunkBlockReason::UnsupportedActivationPolicy => "PSC1101",
        };
        projected.push(ResumeProjectedDiagnostic {
            code,
            message: resume_diagnostic_meaning(code).to_string(),
            primary_identity: block.root_boundary.as_ref().map(ToString::to_string),
            primary_provenance: block.provenance.clone(),
        });
    }
    let capture = &products.capture;
    for block in &capture.blocks {
        let Some(primary_provenance) = block.provenance.clone() else {
            // This is an invalid earlier candidate with no source authority;
            // J19 must not fabricate a public Phase J identity for it.
            continue;
        };
        let code = match block.reason {
            ResumeCaptureBlockReason::MissingBoundarySchema => "PSC1097",
            ResumeCaptureBlockReason::MissingRetainedSlotSchema => "PSC1103",
        };
        projected.push(ResumeProjectedDiagnostic {
            code,
            message: resume_diagnostic_meaning(code).to_string(),
            primary_identity: block
                .slot
                .as_ref()
                .map(|slot| slot.resume_slot_id().to_string())
                .or_else(|| block.boundary.as_ref().map(ToString::to_string)),
            primary_provenance,
        });
    }
    let restore = &products.restore;
    for block in &restore.blocks {
        let code = match block.reason {
            ResumeRestoreBlockReason::MissingSlotSchema
            | ResumeRestoreBlockReason::MissingFormSlotOwner => "PSC1103",
            ResumeRestoreBlockReason::MissingComputedProgram
            | ResumeRestoreBlockReason::UnsupportedRecomputableSlot => "PSC1100",
        };
        projected.push(ResumeProjectedDiagnostic {
            code,
            message: resume_diagnostic_meaning(code).to_string(),
            primary_identity: Some(block.slot.resume_slot_id().to_string()),
            primary_provenance: block.provenance.clone(),
        });
    }
    project_resume_integrity_diagnostics(model, products, &mut projected);
    projected.sort_by(|left, right| {
        (
            left.code,
            &left.primary_identity,
            &left.primary_provenance.path,
            left.primary_provenance.span.start,
            left.primary_provenance.span.end,
            &left.message,
        )
            .cmp(&(
                right.code,
                &right.primary_identity,
                &right.primary_provenance.path,
                right.primary_provenance.span.start,
                right.primary_provenance.span.end,
                &right.message,
            ))
    });
    projected.dedup_by(|left, right| {
        left.code == right.code
            && left.primary_identity == right.primary_identity
            && left.primary_provenance == right.primary_provenance
    });
    projected
}

#[allow(clippy::too_many_lines)]
fn project_resume_integrity_diagnostics(
    model: &ApplicationSemanticModel,
    products: &ResumeDiagnosticProducts,
    projected: &mut Vec<ResumeProjectedDiagnostic>,
) {
    let boundary = |id: Option<&crate::ResumeBoundaryId>| {
        id.and_then(|id| {
            products
                .boundaries
                .boundary(id)
                .map(|record| (id.to_string(), record.provenance.clone()))
        })
    };
    let fallback = || {
        products
            .boundaries
            .boundaries
            .iter()
            .find(|record| record.ownership_parent.is_none())
            .map(|record| (record.id.to_string(), record.provenance.clone()))
    };
    let slot = |slot: Option<&crate::ResumeExistingSlot>| {
        slot.and_then(|slot| {
            products
                .liveness
                .retained
                .iter()
                .map(|record| &record.slot)
                .chain(
                    products
                        .liveness
                        .recomputable
                        .iter()
                        .map(|record| &record.slot),
                )
                .chain(products.liveness.excluded.iter().map(|record| &record.slot))
                .chain(products.liveness.blocked.iter().map(|record| &record.slot))
                .find(|record| &record.existing_slot == slot)
                .map(|record| (record.resume_slot_id.to_string(), record.provenance.clone()))
        })
    };
    let mut push = |code, message: String, primary: Option<(String, SourceProvenance)>| {
        if let Some((primary_identity, primary_provenance)) = primary.or_else(fallback) {
            projected.push(ResumeProjectedDiagnostic {
                code,
                message: format!("{}: {message}", resume_diagnostic_meaning(code)),
                primary_identity: Some(primary_identity),
                primary_provenance,
            });
        }
    };

    for diagnostic in validate_resume_liveness_plan(model, &products.liveness) {
        let code = match diagnostic.code {
            ResumeLivenessIntegrityCode::RequiredUnsupportedValue => "PSC1096",
            ResumeLivenessIntegrityCode::MissingStorageOwner => "PSC1097",
            ResumeLivenessIntegrityCode::UnknownDependency
            | ResumeLivenessIntegrityCode::InvalidRetentionReason
            | ResumeLivenessIntegrityCode::DuplicateClassification
            | ResumeLivenessIntegrityCode::InvalidCandidatePromotion => "PSC1099",
            ResumeLivenessIntegrityCode::RecomputeWithoutProof => "PSC1100",
            ResumeLivenessIntegrityCode::ProvenanceOrderIndexDrift => "PSC1110",
        };
        push(code, diagnostic.message, slot(diagnostic.slot.as_ref()));
    }
    for diagnostic in validate_resume_boundary_graph(model, &products.boundaries) {
        let code = match diagnostic.code {
            ResumeBoundaryIntegrityCode::Cycle => "PSC1098",
            ResumeBoundaryIntegrityCode::DuplicateBoundary => "PSC1105",
            ResumeBoundaryIntegrityCode::PhaseCorrespondence => "PSC1107",
            ResumeBoundaryIntegrityCode::OrderingOrIndexDrift => "PSC1110",
            ResumeBoundaryIntegrityCode::InvalidOwner
            | ResumeBoundaryIntegrityCode::MissingOrMultipleParent
            | ResumeBoundaryIntegrityCode::Unreachable
            | ResumeBoundaryIntegrityCode::Reciprocity
            | ResumeBoundaryIntegrityCode::ProvenanceDrift => "PSC1111",
        };
        push(
            code,
            diagnostic.message,
            boundary(diagnostic.boundary.as_ref()),
        );
    }
    for diagnostic in validate_resume_activation_plan(model, &products.activation) {
        let code = match diagnostic.code {
            ResumeActivationIntegrityCode::UnsupportedLazyPayload => "PSC1108",
            ResumeActivationIntegrityCode::UnknownEventOrBoundary => "PSC1109",
            ResumeActivationIntegrityCode::OrderingOrIndexDrift => "PSC1110",
            ResumeActivationIntegrityCode::MissingOrDuplicatePolicy
            | ResumeActivationIntegrityCode::InvalidPrerequisite
            | ResumeActivationIntegrityCode::InvalidPolicyAuthority => "PSC1101",
        };
        push(
            code,
            diagnostic.message,
            boundary(diagnostic.boundary.as_ref()),
        );
    }
    for diagnostic in validate_resume_chunk_graph(model, &products.chunks) {
        let code = match diagnostic.code {
            ResumeChunkIntegrityCode::DependencyCycle => "PSC1102",
            ResumeChunkIntegrityCode::MissingProgram => "PSC1103",
            ResumeChunkIntegrityCode::RootCorrespondence => "PSC1109",
            ResumeChunkIntegrityCode::DuplicateInclusion
            | ResumeChunkIntegrityCode::UnrelatedProgram
            | ResumeChunkIntegrityCode::OrderingOrOutputDrift => "PSC1110",
        };
        let primary = diagnostic.chunk.as_ref().and_then(|id| {
            products
                .chunks
                .chunk(id)
                .and_then(|chunk| boundary(chunk.root_boundary.as_ref()))
        });
        push(code, diagnostic.message, primary);
    }
    if let Err(diagnostics) = validate_resume_schema_registry(model, &products.schemas) {
        for diagnostic in diagnostics {
            let code = match diagnostic.code {
                ResumeSchemaIntegrityCode::UnsupportedValue => "PSC1096",
                ResumeSchemaIntegrityCode::MissingSlot => "PSC1103",
                ResumeSchemaIntegrityCode::IdentityCollision => "PSC1105",
                ResumeSchemaIntegrityCode::MalformedSemanticType
                | ResumeSchemaIntegrityCode::DuplicateProperty => "PSC1107",
                ResumeSchemaIntegrityCode::OrderingOrIndexDrift => "PSC1110",
            };
            push(
                code,
                diagnostic.message,
                slot(diagnostic.slot.as_ref()).or_else(|| boundary(diagnostic.boundary.as_ref())),
            );
        }
    }
    if let Err(diagnostics) = validate_resume_capture_plan(model, &products.capture) {
        for diagnostic in diagnostics {
            let code = match diagnostic.code {
                ResumeCaptureIntegrityCode::InvalidCaptureState => "PSC1106",
                ResumeCaptureIntegrityCode::OrderingOrOutputDrift => "PSC1110",
                ResumeCaptureIntegrityCode::ProgramCorrespondence
                | ResumeCaptureIntegrityCode::InvalidInstruction => "PSC1103",
            };
            push(
                code,
                diagnostic.message,
                boundary(diagnostic.boundary.as_ref()),
            );
        }
    }
    if let Err(diagnostics) = validate_resume_restore_plan(model, &products.restore) {
        for diagnostic in diagnostics {
            let code = match diagnostic.code {
                ResumeRestoreIntegrityCode::PhaseOrDuplicateWrite
                | ResumeRestoreIntegrityCode::OrderingOrOutputDrift => "PSC1110",
                ResumeRestoreIntegrityCode::ProgramReference
                | ResumeRestoreIntegrityCode::MissingCompletion => "PSC1103",
            };
            push(
                code,
                diagnostic.message,
                boundary(diagnostic.boundary.as_ref()),
            );
        }
    }
    for diagnostic in validate_resume_anchor_plan(model, &products.anchors) {
        let code = match diagnostic.code {
            ResumeAnchorIntegrityCode::OrderingOrOutputDrift => "PSC1110",
            ResumeAnchorIntegrityCode::MissingTarget
            | ResumeAnchorIntegrityCode::UnstableTarget
            | ResumeAnchorIntegrityCode::DuplicateAnchor
            | ResumeAnchorIntegrityCode::WrongKind
            | ResumeAnchorIntegrityCode::StructuralPairMismatch => "PSC1104",
        };
        let primary = diagnostic.anchor_id.as_ref().and_then(|id| {
            products
                .anchors
                .anchors
                .iter()
                .find(|anchor| &anchor.anchor_id == id)
                .and_then(|anchor| boundary(Some(&anchor.boundary_id)))
        });
        push(code, diagnostic.message, primary);
    }
}

fn resume_diagnostic_meaning(code: &str) -> &'static str {
    RESUME_DIAGNOSTIC_RESERVATIONS
        .iter()
        .find(|entry| entry.code == code)
        .map_or("Unknown resumability diagnostic", |entry| entry.meaning)
}

#[must_use]
pub fn validate_resume_instances(
    plan: &ResumePlan,
    instances: &[SerializableInstance],
) -> Vec<ResumeDiagnostic> {
    let mut diagnostics = Vec::new();
    for component in &plan.components {
        let Some(instance) = instances
            .iter()
            .find(|instance| instance.component == component.component)
        else {
            diagnostics.push(ResumeDiagnostic {
                code: "PSRSM1001".to_string(),
                component: component.component.clone(),
                state: None,
            });
            continue;
        };
        for state in &component.state {
            if !instance.state.contains_key(state) {
                diagnostics.push(ResumeDiagnostic {
                    code: "PSRSM1002".to_string(),
                    component: component.component.clone(),
                    state: Some(state.clone()),
                });
            }
        }
    }
    diagnostics
}

#[cfg(test)]
mod tests {
    use super::{
        build_resume_diagnostic_products, project_resume_diagnostics,
        project_resume_diagnostics_from_products, RESUME_DIAGNOSTIC_RESERVATIONS,
        RESUME_INTEGRITY_RESERVATION_END, RESUME_INTEGRITY_RESERVATION_START,
    };

    #[test]
    fn j0_reserves_the_public_and_internal_resumability_ranges_without_products() {
        let codes = RESUME_DIAGNOSTIC_RESERVATIONS
            .iter()
            .map(|reservation| reservation.code)
            .collect::<Vec<_>>();
        assert_eq!(codes.first(), Some(&"PSC1096"));
        assert_eq!(codes.last(), Some(&"PSC1111"));
        assert_eq!(codes.len(), 16);
        assert_eq!(RESUME_INTEGRITY_RESERVATION_START, 1289);
        assert_eq!(RESUME_INTEGRITY_RESERVATION_END, 1384);
        assert_eq!(
            RESUME_INTEGRITY_RESERVATION_END - RESUME_INTEGRITY_RESERVATION_START + 1,
            96
        );
        assert_eq!(crate::RESUME_MANIFEST_SCHEMA_VERSION, 7);
        assert_eq!(crate::SEMANTIC_GRAPH_SCHEMA_VERSION, 6);
        assert_eq!(crate::TEMPLATE_MANIFEST_SCHEMA_VERSION, 5);
    }

    #[test]
    fn j19_projects_unsupported_retained_values_with_exact_source_evidence() {
        let mut model = crate::build_application_semantic_model(&presolve_parser::parse_file(
            "src/ResumeDiagnostics.tsx",
            r#"
@component("x-resume-diagnostics") class ResumeDiagnostics {
  value = state(1);
  render() { return <main>{this.value}</main>; }
}"#,
        ));
        let state = model.components[0].state_fields[0].id.clone();
        model
            .semantic_types
            .assignments
            .get_mut(&state)
            .expect("state type assignment")
            .semantic_type = crate::SemanticType::Unknown;
        let diagnostics = project_resume_diagnostics(&model);
        let diagnostic = diagnostics
            .iter()
            .find(|diagnostic| diagnostic.code == "PSC1096")
            .unwrap_or_else(|| panic!("unsupported retained state value: {diagnostics:#?}"));
        assert!(diagnostic.primary_identity.is_some());
        assert_eq!(
            diagnostic.primary_provenance.path,
            std::path::PathBuf::from("src/ResumeDiagnostics.tsx")
        );
        assert!(diagnostics.windows(2).all(|pair| {
            (pair[0].code, &pair[0].primary_identity, &pair[0].message)
                <= (pair[1].code, &pair[1].primary_identity, &pair[1].message)
        }));
    }

    #[test]
    fn j19_projects_the_integrity_only_catalog_conditions_from_malformed_products() {
        let model = crate::build_application_semantic_model(&presolve_parser::parse_file(
            "src/ResumeIntegrity.tsx",
            r#"@component("x-resume-integrity") class ResumeIntegrity {
  value = state(1);
  render() { return <main>{this.value}</main>; }
}"#,
        ));
        let mut products = build_resume_diagnostic_products(&model);
        let child = products
            .boundaries
            .boundaries
            .iter_mut()
            .find(|boundary| boundary.ownership_parent.is_some())
            .expect("child boundary");
        child.ownership_parent = Some(child.id.clone());
        let eager_chunk = products.chunks.chunks[0].id.clone();
        products.chunks.chunks[0]
            .dependency_chunks
            .push(eager_chunk);
        products
            .schemas
            .schemas
            .push(products.schemas.schemas[0].clone());
        products.capture.envelope_writer.captured_at_is_null = false;
        products
            .anchors
            .anchors
            .push(products.anchors.anchors[0].clone());
        products.schemas.schema_index.clear();

        let codes = project_resume_diagnostics_from_products(&model, &products)
            .into_iter()
            .map(|diagnostic| diagnostic.code)
            .collect::<std::collections::BTreeSet<_>>();
        for code in [
            "PSC1098", "PSC1102", "PSC1104", "PSC1105", "PSC1106", "PSC1110",
        ] {
            assert!(codes.contains(code), "missing {code}: {codes:#?}");
        }
    }
}
