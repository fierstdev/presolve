use std::collections::BTreeSet;

use crate::component_graph::MethodSemanticRole;
use crate::{
    ApplicationSemanticModel, AuthoredDeclarationKind, ComponentDiagnostic,
    ComponentDiagnosticSeverity, ComponentInvocationResolutionStatus, CompositionCompatibility,
    DiagnosticSecondaryLabel, InstanceContextResolutionStatus, SemanticId, SlotBindingStatus,
    SlotContentFragmentViolation, SlotDeclarationViolation, SlotOutletViolation, SourceProvenance,
};

/// Executable H19 contract metadata. Tests use this table to prove that the
/// reserved catalog stays complete and that every code has an explicit
/// authority, evidence, identity, deduplication, suppression, and projection
/// contract.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ComponentDiagnosticContract {
    pub code: &'static str,
    pub message: &'static str,
    pub authority: &'static str,
    pub primary_role: &'static str,
    pub identities: &'static str,
    pub secondary_roles: &'static str,
    pub deduplication: &'static str,
    pub suppression: &'static str,
    pub projection: &'static str,
}

pub const COMPONENT_DIAGNOSTIC_CONTRACTS: [ComponentDiagnosticContract; 16] = [
    contract((
        "PSC1068",
        "Invalid slot declaration.",
        "H1 slot declaration candidates",
        "invalid authored slot declaration",
        "component_id",
        "related slot declarations",
        "candidate identity",
        "none",
        "shared diagnostic envelope",
    )),
    contract((
        "PSC1069",
        "Invalid component invocation.",
        "H2 invocation resolution",
        "component invocation",
        "component_id, invocation_id",
        "candidate definitions",
        "invocation_id",
        "inheritance",
        "shared diagnostic envelope",
    )),
    contract((
        "PSC1070",
        "Unresolved component symbol.",
        "H2 invocation resolution",
        "component invocation symbol",
        "component_id, invocation_id",
        "none",
        "invocation_id",
        "inheritance",
        "shared diagnostic envelope",
    )),
    contract((
        "PSC1071",
        "Component composition cycle.",
        "H9 composition analysis",
        "cycle invocation",
        "component_id, invocation_id",
        "cycle edges",
        "cycle invocation_id",
        "inheritance and unresolved invocation",
        "shared diagnostic envelope",
    )),
    contract((
        "PSC1072",
        "Component inheritance is unsupported.",
        "H1 normalized component heritage",
        "component base class",
        "component_id",
        "none",
        "component_id and base",
        "none",
        "shared diagnostic envelope",
    )),
    contract((
        "PSC1073",
        "Inherited semantic declaration is unsupported.",
        "H1 normalized heritage and semantic declarations",
        "component base class",
        "component_id",
        "inherited semantic declarations",
        "component_id and inherited declaration",
        "generic inheritance",
        "shared diagnostic envelope",
    )),
    contract((
        "PSC1074",
        "Unknown slot.",
        "H3 slot fragment/outlet registry",
        "unknown supplied slot",
        "component_id, invocation_id",
        "none",
        "fragment identity",
        "invocation",
        "shared diagnostic envelope",
    )),
    contract((
        "PSC1075",
        "Duplicate slot content.",
        "H3 slot fragment registry",
        "duplicate supplied content",
        "component_id, invocation_id, slot_id",
        "duplicate fragments",
        "fragment identity",
        "invocation",
        "shared diagnostic envelope",
    )),
    contract((
        "PSC1076",
        "Duplicate slot outlet.",
        "H3 slot outlet registry",
        "duplicate callee outlet",
        "component_id, slot_id",
        "other outlets",
        "slot_id and outlet provenance",
        "inheritance",
        "shared diagnostic envelope",
    )),
    contract((
        "PSC1077",
        "Missing slot outlet.",
        "H7 slot binding registry",
        "instance slot binding",
        "component_id, invocation_id, component_instance_id, slot_binding_id, slot_id",
        "slot declaration",
        "slot_binding_id",
        "invocation and invalid slot",
        "shared diagnostic envelope",
    )),
    contract((
        "PSC1078",
        "Invalid slot content ownership.",
        "H7 binding and H8 ownership typing",
        "instance slot binding",
        "component_id, invocation_id, component_instance_id, slot_binding_id, slot_id",
        "content and outlet owners",
        "slot_binding_id",
        "invocation and invalid slot",
        "shared diagnostic envelope",
    )),
    contract((
        "PSC1079",
        "Slot type or boundary incompatibility.",
        "H8 composition typing",
        "instance slot binding",
        "component_id, invocation_id, component_instance_id, slot_binding_id, slot_id",
        "declared slot type",
        "slot_binding_id",
        "invocation and invalid slot",
        "shared diagnostic envelope",
    )),
    contract((
        "PSC1080",
        "Component instance planning failure.",
        "H4 blocked instance plan",
        "blocked component instance",
        "component_id, invocation_id, component_instance_id",
        "parent instance",
        "component_instance_id",
        "invocation, cycle, and slot",
        "shared diagnostic envelope",
    )),
    contract((
        "PSC1081",
        "Instance-aware Context binding unavailable.",
        "H6 instance Context and H8 typing",
        "consumer instance binding",
        "component_id, component_instance_id, provider_instance_id, consumer_instance_id",
        "candidate Provider instances",
        "consumer_instance_id",
        "component and slot failures",
        "shared diagnostic envelope",
    )),
    contract((
        "PSC1082",
        "Structural region cannot be planned.",
        "H4 blocked structural instance plan",
        "structural region",
        "component_id, invocation_id, component_instance_id, structural_region_id",
        "parent instance",
        "component_instance_id and structural_region_id",
        "invocation, cycle, slot, and Context",
        "shared diagnostic envelope",
    )),
    contract((
        "PSC1083",
        "Component/slot source cannot be lowered.",
        "H10 initialization exclusions and H11 lowering",
        "blocked component or slot source",
        "component_id, invocation_id, component_instance_id, slot_binding_id",
        "none",
        "typed blocked source identity",
        "all earlier subject failures",
        "shared diagnostic envelope",
    )),
];

const fn contract(
    details: (
        &'static str,
        &'static str,
        &'static str,
        &'static str,
        &'static str,
        &'static str,
        &'static str,
        &'static str,
        &'static str,
    ),
) -> ComponentDiagnosticContract {
    let (
        code,
        message,
        authority,
        primary_role,
        identities,
        secondary_roles,
        deduplication,
        suppression,
        projection,
    ) = details;
    ComponentDiagnosticContract {
        code,
        message,
        authority,
        primary_role,
        identities,
        secondary_roles,
        deduplication,
        suppression,
        projection,
    }
}

/// Project H19 diagnostics only from immutable H1-H17 products retained in the
/// ASM. This function has no parser, source-text, CLI, or runtime dependency.
#[must_use]
pub fn collect_component_diagnostics(model: &ApplicationSemanticModel) -> Vec<ComponentDiagnostic> {
    let mut diagnostics = Vec::new();
    collect_declarations_and_inheritance(model, &mut diagnostics);
    collect_invocations_and_cycles(model, &mut diagnostics);
    collect_slot_composition(model, &mut diagnostics);
    collect_slot_bindings(model, &mut diagnostics);
    collect_instance_context(model, &mut diagnostics);
    collect_planning_and_lowering(model, &mut diagnostics);
    suppress_derivative_cascades(&mut diagnostics);
    canonicalize(&mut diagnostics);
    diagnostics
}

fn collect_declarations_and_inheritance(
    model: &ApplicationSemanticModel,
    diagnostics: &mut Vec<ComponentDiagnostic>,
) {
    for component in &model.components {
        for candidate in &component.slot_declaration_candidates {
            let Some(violation) = candidate.violations.first() else {
                continue;
            };
            let primary = slot_declaration_primary(candidate, violation);
            let mut diagnostic = diagnostic("PSC1068", primary, Some(component.id.clone()));
            diagnostic.secondary_labels = component
                .slot_declaration_candidates
                .iter()
                .filter(|other| {
                    other.id != candidate.id && other.field_name == candidate.field_name
                })
                .map(|other| label(other.provenance.clone(), "Related Slot declaration."))
                .collect();
            diagnostics.push(diagnostic);
        }

        let Some(heritage) = &component.heritage else {
            continue;
        };
        if heritage.base == "Component" {
            continue;
        }
        let inherited = inherited_semantic_provenances(model, component, &heritage.base);
        if inherited.is_empty() {
            diagnostics.push(diagnostic(
                "PSC1072",
                heritage.provenance.clone(),
                Some(component.id.clone()),
            ));
        } else {
            for provenance in inherited {
                let mut item = diagnostic(
                    "PSC1073",
                    heritage.provenance.clone(),
                    Some(component.id.clone()),
                );
                item.secondary_labels
                    .push(label(provenance, "Inherited semantic declaration is here."));
                diagnostics.push(item);
            }
        }
    }
}

fn inherited_semantic_provenances(
    model: &ApplicationSemanticModel,
    component: &crate::ComponentNode,
    base: &str,
) -> Vec<SourceProvenance> {
    let component_path = model.provenance(&component.id).map(|item| &item.path);
    let Some(base_component) = model.components.iter().find(|candidate| {
        candidate.class_name == base
            && model.provenance(&candidate.id).map(|item| &item.path) == component_path
    }) else {
        return Vec::new();
    };
    let mut result = Vec::new();
    result.extend(
        base_component
            .state_fields
            .iter()
            .filter_map(|item| model.provenance(&item.id))
            .cloned(),
    );
    result.extend(
        base_component
            .context_declaration_candidates
            .iter()
            .map(|item| item.provenance.clone()),
    );
    result.extend(
        base_component
            .slot_declaration_candidates
            .iter()
            .map(|item| item.provenance.clone()),
    );
    result.extend(
        base_component
            .methods
            .iter()
            .filter(|method| method.semantic_role != MethodSemanticRole::Standard)
            .filter_map(|method| model.provenance(&method.id))
            .cloned(),
    );
    result.sort_by(provenance_order);
    result.dedup();
    result
}

fn collect_invocations_and_cycles(
    model: &ApplicationSemanticModel,
    diagnostics: &mut Vec<ComponentDiagnostic>,
) {
    for invocation in model.component_invocations.values() {
        let code = match invocation.status {
            ComponentInvocationResolutionStatus::Resolved => continue,
            ComponentInvocationResolutionStatus::UnresolvedSymbol => "PSC1070",
            ComponentInvocationResolutionStatus::ResolvedNonComponent
            | ComponentInvocationResolutionStatus::Ambiguous
            | ComponentInvocationResolutionStatus::UnsupportedDynamicTarget => "PSC1069",
        };
        let mut item = diagnostic(
            code,
            invocation.provenance.clone(),
            Some(invocation.owner_component.clone()),
        );
        item.invocation_id = Some(invocation.id.clone());
        diagnostics.push(item);
    }
    for cycle in &model.component_composition.cycles {
        for invocation_id in &cycle.invocations {
            let Some(invocation) = model.component_invocations.get(invocation_id) else {
                continue;
            };
            let mut item = diagnostic(
                "PSC1071",
                invocation.provenance.clone(),
                Some(invocation.owner_component.clone()),
            );
            item.invocation_id = Some(invocation.id.clone());
            item.secondary_labels = cycle
                .invocations
                .iter()
                .filter(|other| *other != invocation_id)
                .filter_map(|other| model.component_invocations.get(other))
                .map(|other| label(other.provenance.clone(), "Composition cycle edge."))
                .collect();
            diagnostics.push(item);
        }
    }
}

fn collect_slot_composition(
    model: &ApplicationSemanticModel,
    diagnostics: &mut Vec<ComponentDiagnostic>,
) {
    for fragment in model.slot_content_fragments.values() {
        let code = if fragment
            .violations
            .contains(&SlotContentFragmentViolation::DuplicateFragment)
        {
            Some("PSC1075")
        } else if fragment.violations.iter().any(|item| {
            matches!(
                item,
                SlotContentFragmentViolation::MissingSlotDeclaration
                    | SlotContentFragmentViolation::UnsupportedDynamicSlotName
                    | SlotContentFragmentViolation::InvalidNestedWrapper
                    | SlotContentFragmentViolation::InvalidWrapperForm
            )
        }) {
            Some("PSC1074")
        } else {
            None
        };
        let Some(code) = code else { continue };
        let mut item = diagnostic(
            code,
            fragment.provenance.clone(),
            Some(fragment.owner_component.clone()),
        );
        item.invocation_id = Some(fragment.invocation.clone());
        item.slot_id.clone_from(&fragment.slot);
        item.secondary_labels = fragment
            .secondary_provenances
            .iter()
            .cloned()
            .map(|provenance| label(provenance, "Related Slot content."))
            .collect();
        diagnostics.push(item);
    }
    let mut emitted_outlet_groups = BTreeSet::new();
    for outlet in model.slot_outlets.values().filter(|outlet| {
        outlet
            .violations
            .contains(&SlotOutletViolation::DuplicateOutlet)
    }) {
        let group = (
            outlet.owner_component.clone(),
            outlet.requested_slot_name.clone(),
        );
        if !emitted_outlet_groups.insert(group) {
            continue;
        }
        let mut item = diagnostic(
            "PSC1076",
            outlet.provenance.clone(),
            Some(outlet.owner_component.clone()),
        );
        item.slot_id.clone_from(&outlet.slot);
        item.secondary_labels = model
            .slot_outlets
            .values()
            .filter(|other| {
                other.id != outlet.id
                    && other.owner_component == outlet.owner_component
                    && other.requested_slot_name == outlet.requested_slot_name
            })
            .map(|other| label(other.provenance.clone(), "Other Slot outlet."))
            .collect();
        diagnostics.push(item);
    }
}

fn collect_slot_bindings(
    model: &ApplicationSemanticModel,
    diagnostics: &mut Vec<ComponentDiagnostic>,
) {
    for binding in model.slot_bindings.bindings.values() {
        let code = match binding.status {
            SlotBindingStatus::MissingOutlet => Some("PSC1077"),
            SlotBindingStatus::InvalidOwnership => Some("PSC1078"),
            _ => None,
        };
        if let Some(code) = code {
            diagnostics.push(binding_diagnostic(model, binding, code));
        }
        if model
            .composition_types
            .slot_bindings
            .get(&binding.id)
            .is_some_and(|record| {
                record.type_compatibility == CompositionCompatibility::Incompatible
            })
        {
            diagnostics.push(binding_diagnostic(model, binding, "PSC1079"));
        }
    }
}

fn binding_diagnostic(
    model: &ApplicationSemanticModel,
    binding: &crate::SlotBinding,
    code: &str,
) -> ComponentDiagnostic {
    let component = model
        .component_instance_plan
        .instances
        .get(&binding.callee_instance)
        .map(|item| item.component.clone());
    let mut item = diagnostic(code, binding.provenance.clone(), component);
    item.invocation_id = Some(binding.invocation.clone());
    item.component_instance_id = Some(binding.callee_instance.clone());
    item.slot_binding_id = Some(binding.id.clone());
    item.slot_id.clone_from(&binding.slot);
    if let Some(slot) = binding.slot.as_ref().and_then(|id| model.slot(id)) {
        item.secondary_labels
            .push(label(slot.provenance.clone(), "Slot declaration."));
    }
    if code == "PSC1078" {
        if let Some(fragment) = binding
            .content_fragment
            .as_ref()
            .and_then(|id| model.slot_content_fragments.get(id))
        {
            item.secondary_labels.push(label(
                fragment.provenance.clone(),
                "Caller-owned Slot content.",
            ));
        }
        if let Some(outlet) = binding
            .outlet
            .as_ref()
            .and_then(|id| model.slot_outlets.get(id))
        {
            item.secondary_labels.push(label(
                outlet.provenance.clone(),
                "Callee-owned Slot outlet.",
            ));
        }
    }
    item
}

fn collect_instance_context(
    model: &ApplicationSemanticModel,
    diagnostics: &mut Vec<ComponentDiagnostic>,
) {
    for resolution in model.instance_context.resolutions.values() {
        let incompatible = model
            .composition_types
            .instance_context_bindings
            .get(&resolution.consumer_instance)
            .is_some_and(|record| record.overall != CompositionCompatibility::Compatible);
        if !incompatible
            && matches!(
                resolution.status,
                InstanceContextResolutionStatus::ProviderSelected
                    | InstanceContextResolutionStatus::ContextDefaultSelected
            )
        {
            continue;
        }
        let component = model
            .instance_context
            .consumer_instances
            .get(&resolution.consumer_instance)
            .map(|item| item.component.clone());
        let mut item = diagnostic("PSC1081", resolution.provenance.clone(), component);
        item.context_id.clone_from(&resolution.context);
        item.provider_id = resolution
            .provider_instance
            .as_ref()
            .map(|id| id.provider.clone());
        item.consumer_id = Some(resolution.consumer_instance.consumer.clone());
        item.component_instance_id = Some(resolution.consumer_instance.component_instance.clone());
        item.provider_instance_id
            .clone_from(&resolution.provider_instance);
        item.consumer_instance_id = Some(resolution.consumer_instance.clone());
        item.secondary_labels = resolution
            .candidate_provider_instances
            .iter()
            .filter_map(|id| model.instance_context.provider_instances.get(id))
            .map(|record| label(record.provenance.clone(), "Candidate Provider instance."))
            .collect();
        diagnostics.push(item);
    }
}

fn collect_planning_and_lowering(
    model: &ApplicationSemanticModel,
    diagnostics: &mut Vec<ComponentDiagnostic>,
) {
    let blocked_ids = model
        .component_initialization
        .blocked_instances
        .iter()
        .collect::<BTreeSet<_>>();
    for blocked in model.component_instance_plan.blocked.values() {
        let code = if blocked.structural_region.is_some() {
            "PSC1082"
        } else {
            "PSC1080"
        };
        let component = blocked.target_component.clone().or_else(|| {
            model
                .component_invocations
                .get(&blocked.invocation)
                .map(|item| item.owner_component.clone())
        });
        let mut item = diagnostic(code, blocked.provenance.clone(), component.clone());
        item.invocation_id = Some(blocked.invocation.clone());
        item.component_instance_id = Some(blocked.id.clone());
        item.structural_region_id
            .clone_from(&blocked.structural_region);
        if let Some(parent) = model
            .component_instance_plan
            .instances
            .get(&blocked.parent_instance)
        {
            item.secondary_labels
                .push(label(parent.provenance.clone(), "Planned parent instance."));
        }
        diagnostics.push(item);
        if blocked_ids.contains(&blocked.id) {
            let mut lowering = diagnostic("PSC1083", blocked.provenance.clone(), component);
            lowering.invocation_id = Some(blocked.invocation.clone());
            lowering.component_instance_id = Some(blocked.id.clone());
            lowering
                .structural_region_id
                .clone_from(&blocked.structural_region);
            diagnostics.push(lowering);
        }
    }
    for binding in model.slot_bindings.bindings.values().filter(|binding| {
        !matches!(
            binding.status,
            SlotBindingStatus::Bound | SlotBindingStatus::Empty
        )
    }) {
        let mut item = binding_diagnostic(model, binding, "PSC1083");
        item.secondary_labels.clear();
        diagnostics.push(item);
    }
}

fn suppress_derivative_cascades(diagnostics: &mut Vec<ComponentDiagnostic>) {
    let invalid_components = diagnostics
        .iter()
        .filter(|item| matches!(item.code.as_str(), "PSC1072" | "PSC1073"))
        .filter_map(|item| item.component_id.clone())
        .collect::<BTreeSet<_>>();
    let unresolved_invocations = diagnostics
        .iter()
        .filter(|item| matches!(item.code.as_str(), "PSC1069" | "PSC1070"))
        .filter_map(|item| item.invocation_id.clone())
        .collect::<BTreeSet<_>>();
    let cycle_invocations = diagnostics
        .iter()
        .filter(|item| item.code == "PSC1071")
        .filter_map(|item| item.invocation_id.clone())
        .collect::<BTreeSet<_>>();
    let invalid_bindings = diagnostics
        .iter()
        .filter(|item| {
            matches!(
                item.code.as_str(),
                "PSC1074" | "PSC1075" | "PSC1076" | "PSC1077" | "PSC1078" | "PSC1079"
            )
        })
        .filter_map(|item| item.slot_binding_id.clone())
        .collect::<BTreeSet<_>>();
    let invalid_slot_invocations = diagnostics
        .iter()
        .filter(|item| matches!(item.code.as_str(), "PSC1074" | "PSC1075"))
        .filter_map(|item| item.invocation_id.clone())
        .collect::<BTreeSet<_>>();
    let context_instances = diagnostics
        .iter()
        .filter(|item| item.code == "PSC1081")
        .filter_map(|item| item.component_instance_id.clone())
        .collect::<BTreeSet<_>>();
    let planning_failures = diagnostics
        .iter()
        .filter(|item| matches!(item.code.as_str(), "PSC1080" | "PSC1082"))
        .filter_map(|item| item.component_instance_id.clone())
        .collect::<BTreeSet<_>>();
    diagnostics.retain(|item| {
        let rank = suppression_rank(&item.code);
        let invalid_component = rank > 1
            && item
                .component_id
                .as_ref()
                .is_some_and(|id| invalid_components.contains(id));
        let unresolved_invocation = rank > 2
            && item
                .invocation_id
                .as_ref()
                .is_some_and(|id| unresolved_invocations.contains(id));
        let cyclic_invocation = rank > 3
            && item
                .invocation_id
                .as_ref()
                .is_some_and(|id| cycle_invocations.contains(id));
        let invalid_binding = rank > 4
            && item
                .slot_binding_id
                .as_ref()
                .is_some_and(|id| invalid_bindings.contains(id));
        let invalid_slot_invocation = rank > 4
            && item
                .invocation_id
                .as_ref()
                .is_some_and(|id| invalid_slot_invocations.contains(id));
        let unavailable_context = rank > 5
            && item
                .component_instance_id
                .as_ref()
                .is_some_and(|id| context_instances.contains(id));
        let failed_planning = item.code == "PSC1083"
            && item
                .component_instance_id
                .as_ref()
                .is_some_and(|id| planning_failures.contains(id));
        !(invalid_component
            || unresolved_invocation
            || cyclic_invocation
            || invalid_binding
            || invalid_slot_invocation
            || unavailable_context
            || failed_planning)
    });
}

const fn suppression_rank(code: &str) -> u8 {
    match code.as_bytes() {
        b"PSC1068" | b"PSC1072" | b"PSC1073" => 1,
        b"PSC1069" | b"PSC1070" => 2,
        b"PSC1071" => 3,
        b"PSC1074" | b"PSC1075" | b"PSC1076" | b"PSC1077" | b"PSC1078" | b"PSC1079" => 4,
        b"PSC1081" => 5,
        _ => 6,
    }
}

fn canonicalize(diagnostics: &mut Vec<ComponentDiagnostic>) {
    for item in diagnostics.iter_mut() {
        item.secondary_labels.sort_by_key(secondary_order);
        item.secondary_labels.dedup();
        if let Some(primary) = &item.provenance {
            item.secondary_labels
                .retain(|label| &label.provenance != primary);
        }
    }
    diagnostics.sort_by(diagnostic_order);
    diagnostics.dedup();
}

fn diagnostic_order(left: &ComponentDiagnostic, right: &ComponentDiagnostic) -> std::cmp::Ordering {
    diagnostic_key(left).cmp(&diagnostic_key(right))
}

fn diagnostic_key(item: &ComponentDiagnostic) -> (String, String, usize, usize, String) {
    let provenance = item.provenance.as_ref();
    (
        item.code.clone(),
        provenance
            .map(|item| item.path.display().to_string())
            .unwrap_or_default(),
        provenance.map_or(0, |item| item.span.start),
        provenance.map_or(0, |item| item.span.end),
        identity_key(item),
    )
}

fn identity_key(item: &ComponentDiagnostic) -> String {
    format!(
        "{:?}|{:?}|{:?}|{:?}|{:?}|{:?}|{:?}|{:?}",
        item.component_id,
        item.slot_id,
        item.invocation_id,
        item.component_instance_id,
        item.slot_binding_id,
        item.structural_region_id,
        item.provider_instance_id,
        item.consumer_instance_id
    )
}

fn secondary_order(label: &DiagnosticSecondaryLabel) -> (String, usize, usize, String) {
    (
        label.provenance.path.display().to_string(),
        label.provenance.span.start,
        label.provenance.span.end,
        label.message.clone(),
    )
}

fn provenance_order(left: &SourceProvenance, right: &SourceProvenance) -> std::cmp::Ordering {
    (left.path.as_path(), left.span.start, left.span.end).cmp(&(
        right.path.as_path(),
        right.span.start,
        right.span.end,
    ))
}

fn slot_declaration_primary(
    candidate: &crate::AuthoredSlotDeclarationCandidate,
    violation: &SlotDeclarationViolation,
) -> SourceProvenance {
    match violation {
        SlotDeclarationViolation::StaticDeclarationUnsupported => {
            candidate.static_modifier_provenance.as_ref()
        }
        SlotDeclarationViolation::ForbiddenInitializer => candidate.initializer_provenance.as_ref(),
        SlotDeclarationViolation::InvalidDeclarationKind {
            actual:
                AuthoredDeclarationKind::Method
                | AuthoredDeclarationKind::Getter
                | AuthoredDeclarationKind::Parameter,
        } => Some(&candidate.provenance),
        _ => Some(&candidate.decorator_provenance),
    }
    .unwrap_or(&candidate.provenance)
    .clone()
}

fn diagnostic(
    code: &str,
    provenance: SourceProvenance,
    component_id: Option<SemanticId>,
) -> ComponentDiagnostic {
    let message = COMPONENT_DIAGNOSTIC_CONTRACTS
        .iter()
        .find(|item| item.code == code)
        .expect("H19 code has a contract")
        .message;
    ComponentDiagnostic {
        code: code.to_string(),
        severity: ComponentDiagnosticSeverity::Error,
        message: message.to_string(),
        provenance: Some(provenance),
        effect_id: None,
        statement_id: None,
        context_declaration_candidate_id: None,
        context_id: None,
        provider_id: None,
        consumer_id: None,
        slot_id: None,
        invocation_id: None,
        component_instance_id: None,
        slot_binding_id: None,
        structural_region_id: None,
        component_id,
        provider_instance_id: None,
        consumer_instance_id: None,
        secondary_labels: Vec::new(),
    }
}

fn label(provenance: SourceProvenance, message: &str) -> DiagnosticSecondaryLabel {
    DiagnosticSecondaryLabel {
        provenance,
        message: message.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use super::{collect_component_diagnostics, COMPONENT_DIAGNOSTIC_CONTRACTS};
    use crate::{
        build_application_semantic_model, validate_application_semantic_model,
        BlockedComponentInstancePlan, BlockedComponentInstanceReason, ComponentInstanceStatus,
        ComponentInvocationId, ComponentInvocationResolutionStatus, CompositionCompatibility,
        SemanticId, SlotBindingStatus,
    };

    #[test]
    fn contract_table_covers_the_complete_reserved_catalog() {
        let codes = COMPONENT_DIAGNOSTIC_CONTRACTS
            .iter()
            .map(|item| item.code)
            .collect::<Vec<_>>();
        assert_eq!(
            codes,
            (1068..=1083)
                .map(|code| format!("PSC{code}"))
                .collect::<Vec<_>>()
        );
        for item in COMPONENT_DIAGNOSTIC_CONTRACTS {
            assert!(item.message.ends_with('.'));
            assert!(!item.authority.is_empty());
            assert!(!item.primary_role.is_empty());
            assert!(!item.identities.is_empty());
            assert!(!item.secondary_roles.is_empty());
            assert!(!item.deduplication.is_empty());
            assert!(!item.suppression.is_empty());
            assert_eq!(item.projection, "shared diagnostic envelope");
        }
    }

    #[test]
    fn projects_declaration_invocation_cycle_slot_and_context_diagnostics_deterministically() {
        let source = r#"
class Base { @slot() inherited!: SlotContent; render() { return <div />; } }
@component("x-card") class Card extends Component {
  @slot("bad") bad!: SlotContent;
  @slot() children!: SlotContent;
  render() { return <article><slot /><slot /></article>; }
}
@component("x-cycle") class Cycle extends Component { render() { return <Cycle />; } }
@component("x-page") class Page extends Base {
  render() { return <main />; }
}
@component("x-app") class App extends Component {
  render() { return <main><Missing /><Card><template slot="unknown"><b /></template></Card></main>; }
}
"#;
        let model =
            build_application_semantic_model(&presolve_parser::parse_file("src/H19.tsx", source));
        let first = collect_component_diagnostics(&model);
        let second = collect_component_diagnostics(&model);
        assert_eq!(first, second);
        let codes = first
            .iter()
            .map(|item| item.code.as_str())
            .collect::<BTreeSet<_>>();
        for code in [
            "PSC1068", "PSC1070", "PSC1071", "PSC1073", "PSC1074", "PSC1076",
        ] {
            assert!(codes.contains(code), "missing {code}: {first:#?}");
        }
        assert!(first
            .iter()
            .all(|item| item.severity == crate::ComponentDiagnosticSeverity::Error));
        let validation = validate_application_semantic_model(&model);
        assert!(validation.is_empty(), "{validation:#?}");
    }

    #[test]
    fn nearby_valid_component_composition_emits_no_h19_diagnostic() {
        let model = build_application_semantic_model(&presolve_parser::parse_file(
            "src/ValidH19.tsx",
            r#"@component("x-card") class Card extends Component { @slot() children!: SlotContent; render() { return <article><slot /></article>; } } @component("x-page") class Page extends Component { render() { return <Card><p /></Card>; } }"#,
        ));
        assert!(collect_component_diagnostics(&model).is_empty());
    }

    #[test]
    #[allow(clippy::too_many_lines)]
    fn every_code_has_canonical_identity_provenance_label_order_and_deduplication() {
        for contract in COMPONENT_DIAGNOSTIC_CONTRACTS {
            let item = example_diagnostic(contract.code);
            assert_eq!(item.code, contract.code);
            assert_eq!(item.message, contract.message);
            assert_eq!(item.severity, crate::ComponentDiagnosticSeverity::Error);
            assert!(item.provenance.is_some(), "{} primary", contract.code);
            assert_required_identities(&item, contract.identities);
            let mut labels = item.secondary_labels.clone();
            labels.sort_by(|left, right| {
                super::secondary_order(left).cmp(&super::secondary_order(right))
            });
            labels.dedup();
            assert_eq!(labels, item.secondary_labels, "{} labels", contract.code);
            assert!(item
                .secondary_labels
                .iter()
                .all(|label| Some(&label.provenance) != item.provenance.as_ref()));
        }
    }

    #[test]
    fn suppression_removes_only_derivative_planning_and_lowering_failures() {
        let unresolved = model(
            r#"@component("x-page") class Page extends Component { render() { return <main><Missing /></main>; } }"#,
        );
        let diagnostics = collect_component_diagnostics(&unresolved);
        assert!(diagnostics.iter().any(|item| item.code == "PSC1070"));
        assert!(!diagnostics
            .iter()
            .any(|item| matches!(item.code.as_str(), "PSC1080" | "PSC1082" | "PSC1083")));

        let independent = model(
            r#"@component("x-a") class A extends Component { @slot("bad") invalid!: SlotContent; render() { return <B />; } } @component("x-b") class B extends Component { render() { return <A />; } }"#,
        );
        let codes = collect_component_diagnostics(&independent)
            .into_iter()
            .map(|item| item.code)
            .collect::<BTreeSet<_>>();
        assert!(codes.contains("PSC1068"));
        assert!(codes.contains("PSC1071"));
    }

    #[test]
    fn validation_rejects_every_noncanonical_component_diagnostic_mutation() {
        let mut unknown_identity = model(
            r#"@component("x-page") class Page extends Component { render() { return <Missing />; } }"#,
        );
        let index = unknown_identity
            .diagnostics
            .iter()
            .position(|item| item.code == "PSC1070")
            .expect("component diagnostic");
        unknown_identity.diagnostics[index].component_id =
            Some(SemanticId::component(Some("x-fabricated"), "Fabricated"));
        unknown_identity.diagnostics[index].invocation_id =
            Some(ComponentInvocationId::for_template_entity(
                &SemanticId::component(Some("x-fabricated"), "Fabricated"),
                "missing",
            ));
        assert!(validate_application_semantic_model(&unknown_identity)
            .iter()
            .any(|item| item.code == "PSASM1201"));

        let mut cross_owner = model(
            r#"@component("x-a") class A extends Component { render() { return <Missing />; } } @component("x-b") class B extends Component { render() { return <main />; } }"#,
        );
        let other = cross_owner
            .components
            .iter()
            .find(|item| item.element_name.as_deref() == Some("x-b"))
            .unwrap()
            .id
            .clone();
        let index = cross_owner
            .diagnostics
            .iter()
            .position(|item| item.code == "PSC1070")
            .unwrap();
        cross_owner.diagnostics[index].component_id = Some(other);
        assert!(validate_application_semantic_model(&cross_owner)
            .iter()
            .any(|item| item.code == "PSASM1201"));

        let mut bad_provenance = model(
            r#"@component("x-page") class Page extends Component { render() { return <Missing />; } }"#,
        );
        let index = bad_provenance
            .diagnostics
            .iter()
            .position(|item| item.code == "PSC1070")
            .unwrap();
        bad_provenance.diagnostics[index]
            .provenance
            .as_mut()
            .unwrap()
            .span
            .start += 1;
        assert!(validate_application_semantic_model(&bad_provenance)
            .iter()
            .any(|item| item.code == "PSASM1201"));

        let mut bad_labels = model(
            r#"@component("x-card") class Card extends Component { @slot() children!: SlotContent; render() { return <main><slot /><slot /></main>; } }"#,
        );
        let index = bad_labels
            .diagnostics
            .iter()
            .position(|item| item.code == "PSC1076")
            .unwrap();
        let duplicate = bad_labels.diagnostics[index].secondary_labels[0].clone();
        bad_labels.diagnostics[index]
            .secondary_labels
            .push(duplicate);
        assert!(validate_application_semantic_model(&bad_labels)
            .iter()
            .any(|item| item.code == "PSASM1201"));

        let mut primary_repeated = model(
            r#"@component("x-card") class Card extends Component { @slot() children!: SlotContent; render() { return <main><slot /><slot /></main>; } }"#,
        );
        let index = primary_repeated
            .diagnostics
            .iter()
            .position(|item| item.code == "PSC1076")
            .unwrap();
        primary_repeated.diagnostics[index].secondary_labels[0].provenance = primary_repeated
            .diagnostics[index]
            .provenance
            .clone()
            .unwrap();
        assert!(validate_application_semantic_model(&primary_repeated)
            .iter()
            .any(|item| item.code == "PSASM1201"));

        let mut unsupported = model(
            r#"@component("x-page") class Page extends Component { render() { return <Missing />; } }"#,
        );
        let mut fabricated = unsupported
            .diagnostics
            .iter()
            .find(|item| item.code == "PSC1070")
            .unwrap()
            .clone();
        fabricated.code = "PSC1083".to_string();
        fabricated.message = "Component/slot source cannot be lowered.".to_string();
        unsupported.diagnostics.push(fabricated);
        assert!(validate_application_semantic_model(&unsupported)
            .iter()
            .any(|item| item.code == "PSASM1201"));
    }

    fn example_model(code: &str) -> crate::ApplicationSemanticModel {
        match code {
            "PSC1068" => model(
                r#"@component("x-page") class Page extends Component { @slot("bad") value!: SlotContent; render() { return <main />; } }"#,
            ),
            "PSC1069" => model(
                r#"@component("x-page") class Page extends Component { render() { return <Registry.Card />; } }"#,
            ),
            "PSC1070" => model(
                r#"@component("x-page") class Page extends Component { render() { return <Missing />; } }"#,
            ),
            "PSC1071" => model(
                r#"@component("x-a") class A extends Component { render() { return <B />; } } @component("x-b") class B extends Component { render() { return <A />; } }"#,
            ),
            "PSC1072" => model(
                r#"class Base { render() { return <div />; } } @component("x-page") class Page extends Base { render() { return <main />; } }"#,
            ),
            "PSC1073" => model(
                r#"class Base { value = state(0); render() { return <div />; } } @component("x-page") class Page extends Base { render() { return <main />; } }"#,
            ),
            "PSC1074" => slot_model(r#"<Card><template slot="missing"><b /></template></Card>"#),
            "PSC1075" => slot_model(
                r#"<Card><template slot="header"><b /></template><template slot="header"><i /></template></Card>"#,
            ),
            "PSC1076" => model(
                r#"@component("x-card") class Card extends Component { @slot() children!: SlotContent; render() { return <article><slot /><slot /></article>; } }"#,
            ),
            "PSC1077" => slot_model(r#"<Card><template slot="header"><b /></template></Card>"#),
            "PSC1078" | "PSC1079" | "PSC1083" => {
                slot_model(r#"<Card><template slot="header"><b /></template></Card>"#)
            }
            "PSC1080" => model(
                r#"@component("x-page") class Page extends Component { render() { return <Missing />; } } @component("x-card") class Card extends Component { render() { return <div />; } }"#,
            ),
            "PSC1081" => model(
                r#"@component("x-theme") class Theme extends Component { @context() color!: string; render() { return <div />; } } @component("x-leaf") class Leaf extends Component { @consume(Theme.color) color!: number; render() { return <span />; } } @component("x-card") class Card extends Component { @provide(Theme.color) color: string = "blue"; render() { return <Leaf />; } } @component("x-page") class Page extends Component { render() { return <Card />; } }"#,
            ),
            "PSC1082" => model(
                r#"@component("x-card") class Card extends Component { render() { return <div />; } } @component("x-page") class Page extends Component { visible = state(true); render() { return <main>{this.visible ? <Card /> : <span />}</main>; } }"#,
            ),
            _ => unreachable!(),
        }
    }

    fn example_diagnostic(code: &str) -> crate::ComponentDiagnostic {
        let mut model = example_model(code);

        match code {
            "PSC1078" => {
                model
                    .slot_bindings
                    .bindings
                    .values_mut()
                    .next()
                    .unwrap()
                    .status = SlotBindingStatus::InvalidOwnership;
            }
            "PSC1079" => {
                let record = model
                    .composition_types
                    .slot_bindings
                    .values_mut()
                    .next()
                    .unwrap();
                record.type_compatibility = CompositionCompatibility::Incompatible;
                record.overall = CompositionCompatibility::Incompatible;
            }
            "PSC1080" => {
                let target = model
                    .components
                    .iter()
                    .find(|item| item.element_name.as_deref() == Some("x-card"))
                    .unwrap()
                    .id
                    .clone();
                let invocation_id = model
                    .component_instance_plan
                    .blocked
                    .values()
                    .next()
                    .unwrap()
                    .invocation
                    .clone();
                let invocation = model.component_invocations.get_mut(&invocation_id).unwrap();
                invocation.status = ComponentInvocationResolutionStatus::Resolved;
                invocation.target_component = Some(target.clone());
                model
                    .component_instance_plan
                    .blocked
                    .values_mut()
                    .next()
                    .unwrap()
                    .target_component = Some(target);
            }
            "PSC1082" => {
                let id = model
                    .component_instance_plan
                    .instances
                    .values()
                    .find(|item| item.status == ComponentInstanceStatus::StructuralTemplate)
                    .unwrap()
                    .id
                    .clone();
                let instance = model.component_instance_plan.instances.remove(&id).unwrap();
                model.component_instance_plan.blocked.insert(
                    id.clone(),
                    BlockedComponentInstancePlan {
                        id,
                        invocation: instance.invocation.unwrap(),
                        parent_instance: instance.parent_instance.unwrap(),
                        owner_root: instance.owner_root,
                        target_component: Some(instance.component),
                        structural_region: instance.structural_region,
                        depth: instance.depth,
                        reason: BlockedComponentInstanceReason::InvalidParentPlan,
                        provenance: instance.provenance,
                    },
                );
            }
            "PSC1083" => {
                model
                    .slot_bindings
                    .bindings
                    .values_mut()
                    .next()
                    .unwrap()
                    .status = SlotBindingStatus::BlockedInvocation;
            }
            _ => {}
        }
        collect_component_diagnostics(&model)
            .into_iter()
            .find(|item| item.code == code)
            .unwrap_or_else(|| panic!("missing {code}"))
    }

    fn model(source: &str) -> crate::ApplicationSemanticModel {
        build_application_semantic_model(&presolve_parser::parse_file(
            "src/DiagnosticMatrix.tsx",
            source,
        ))
    }

    fn slot_model(invocation: &str) -> crate::ApplicationSemanticModel {
        model(&format!(
            r#"@component("x-card") class Card extends Component {{ @slot() children!: SlotContent; @slot() header!: SlotContent; render() {{ return <article><slot /></article>; }} }} @component("x-page") class Page extends Component {{ render() {{ return <main>{invocation}</main>; }} }}"#
        ))
    }

    fn assert_required_identities(item: &crate::ComponentDiagnostic, identities: &str) {
        if identities.contains("component_id") {
            assert!(item.component_id.is_some(), "{} component", item.code);
        }
        if identities.contains("slot_id") {
            assert!(item.slot_id.is_some(), "{} slot", item.code);
        }
        if identities.contains("invocation_id") {
            assert!(item.invocation_id.is_some(), "{} invocation", item.code);
        }
        if identities.contains("component_instance_id") {
            assert!(
                item.component_instance_id.is_some(),
                "{} instance",
                item.code
            );
        }
        if identities.contains("slot_binding_id") {
            assert!(item.slot_binding_id.is_some(), "{} binding", item.code);
        }
        if identities.contains("structural_region_id") {
            assert!(item.structural_region_id.is_some(), "{} region", item.code);
        }
        if identities.contains("provider_instance_id") {
            assert!(
                item.provider_instance_id.is_some(),
                "{} provider instance",
                item.code
            );
        }
        if identities.contains("consumer_instance_id") {
            assert!(
                item.consumer_instance_id.is_some(),
                "{} consumer instance",
                item.code
            );
        }
    }
}
