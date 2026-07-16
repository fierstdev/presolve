use std::collections::BTreeMap;

use crate::{
    ComponentNode, ExecutionBoundary, SemanticOwner, SemanticType, SlotId, SlotKind,
    SourceProvenance,
};

/// First-class compiler-owned semantic entity for one valid H1 `@slot()` field.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SlotEntity {
    pub id: SlotId,
    pub owner: crate::SemanticId,
    pub authored_field: crate::SemanticId,
    pub name: String,
    pub kind: SlotKind,
    pub semantic_type: SemanticType,
    pub boundary: ExecutionBoundary,
    pub declared_type_provenance: SourceProvenance,
    pub provenance: SourceProvenance,
}

impl SlotEntity {
    #[must_use]
    pub fn semantic_owner(&self) -> SemanticOwner {
        SemanticOwner::entity(self.owner.clone())
    }
}

/// Lower valid authored Slot declarations into stable canonical entities.
#[must_use]
pub fn collect_slot_entities(components: &[ComponentNode]) -> BTreeMap<SlotId, SlotEntity> {
    components
        .iter()
        .flat_map(|component| {
            component.slot_declarations.iter().map(move |declaration| {
                let id = SlotId::for_component(&component.id, &declaration.name);
                (
                    id.clone(),
                    SlotEntity {
                        id,
                        owner: component.id.clone(),
                        authored_field: declaration.authored_field.clone(),
                        name: declaration.name.clone(),
                        kind: declaration.kind,
                        semantic_type: SemanticType::SlotContent,
                        boundary: ExecutionBoundary::Client,
                        declared_type_provenance: declaration.declared_type.provenance.clone(),
                        provenance: declaration.provenance.clone(),
                    },
                )
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use crate::{
        build_application_semantic_model, build_semantic_graph,
        validate_application_semantic_model, AuthoredDeclarationKind, ExecutionBoundary,
        SemanticEntityKind, SemanticOwner, SemanticType, SlotDeclarationViolation, SlotId,
        SlotKind,
    };

    #[test]
    fn lowers_default_and_named_slots_with_canonical_identity_and_ownership() {
        let parsed = ezc_parser::parse_file(
            "src/Card.tsx",
            r#"
@component("x-card")
class Card extends Component {
  @slot()
  children!: SlotContent;

  @slot()
  header!: SlotContent;

  render() { return <section />; }
}
"#,
        );
        let asm = build_application_semantic_model(&parsed);
        let component = &asm.components[0];
        let children = asm
            .slot(&SlotId::for_component(&component.id, "children"))
            .expect("default slot");
        let header = asm
            .slot(&SlotId::for_component(&component.id, "header"))
            .expect("named slot");

        assert_eq!(asm.slots().len(), 2);
        assert_eq!(children.kind, SlotKind::Default);
        assert_eq!(header.kind, SlotKind::Named);
        assert_eq!(children.semantic_type, SemanticType::SlotContent);
        assert_eq!(children.boundary, ExecutionBoundary::Client);
        assert_eq!(children.owner, component.id);
        assert_eq!(children.authored_field, component.id.slot_field("children"));
        assert_eq!(
            children.id.as_str(),
            "module:src/Card.tsx/component:x-card/slot:children"
        );
        assert_eq!(
            asm.owner(children.id.as_semantic_id()),
            Some(&SemanticOwner::entity(component.id.clone()))
        );
        assert_eq!(
            asm.semantic_type_of(children.id.as_semantic_id()),
            Some(&SemanticType::SlotContent)
        );
        assert!(asm
            .entity(children.id.as_semantic_id())
            .is_some_and(|entity| { entity.kind() == SemanticEntityKind::Slot }));
        assert!(children.provenance.span.start < children.provenance.span.end);
        assert!(
            children.declared_type_provenance.span.start
                < children.declared_type_provenance.span.end
        );
        let diagnostics = validate_application_semantic_model(&asm);
        assert!(diagnostics.is_empty(), "{diagnostics:#?}");
    }

    #[test]
    fn retains_invalid_slot_field_shape_facts_without_creating_slot_ids() {
        let parsed = ezc_parser::parse_file(
            "src/InvalidSlots.tsx",
            r#"
@component("x-invalid-slots")
class InvalidSlots extends Component {
  @slot("header") argument!: SlotContent;
  @slot() static staticSlot!: SlotContent;
  @slot() missingType!;
  @slot() wrongType!: string;
  @slot() initialized: SlotContent = "bad";
  @slot() notDefinite: SlotContent;
  render() { return <main />; }
}
"#,
        );
        let asm = build_application_semantic_model(&parsed);
        let candidates = &asm.components[0].slot_declaration_candidates;

        assert!(asm.slots().is_empty());
        assert_eq!(candidates.len(), 6);
        assert!(candidates
            .iter()
            .any(
                |candidate| candidate.violations.iter().any(|violation| matches!(
                    violation,
                    SlotDeclarationViolation::DecoratorArity { .. }
                ))
            ));
        assert!(candidates
            .iter()
            .any(
                |candidate| candidate.violations.iter().any(|violation| matches!(
                    violation,
                    SlotDeclarationViolation::StaticDeclarationUnsupported
                ))
            ));
        assert_eq!(
            candidates
                .iter()
                .filter(
                    |candidate| candidate.violations.iter().any(|violation| matches!(
                        violation,
                        SlotDeclarationViolation::InvalidDeclaredType { .. }
                    ))
                )
                .count(),
            2
        );
        assert!(candidates.iter().any(|candidate| candidate
            .violations
            .iter()
            .any(|violation| matches!(violation, SlotDeclarationViolation::ForbiddenInitializer))));
        assert!(candidates
            .iter()
            .any(
                |candidate| candidate.violations.iter().any(|violation| matches!(
                    violation,
                    SlotDeclarationViolation::DefiniteAssignmentRequired
                ))
            ));
        assert!(candidates.iter().all(|candidate| {
            !asm.slots.contains_key(&SlotId::for_component(
                &candidate.owner_component,
                candidate.field_name.as_deref().unwrap_or("invalid"),
            ))
        }));
    }

    #[test]
    fn retains_slot_conflict_duplicate_and_invalid_declaration_kind_facts() {
        let parsed = ezc_parser::parse_file(
            "src/InvalidSlotKinds.tsx",
            r#"
@component("x-invalid-slot-kinds")
class InvalidSlotKinds extends Component {
  @slot() @context() conflict!: SlotContent;
  @slot() duplicate!: SlotContent;
  @slot() duplicate!: SlotContent;
  @slot() method() {}
  @slot() get accessor(): SlotContent { return this.value; }
  attach(@slot() content: SlotContent) {}
  render() { return <main />; }
}
"#,
        );
        let asm = build_application_semantic_model(&parsed);
        let candidates = &asm.components[0].slot_declaration_candidates;

        assert!(asm.slots().is_empty());
        assert_eq!(candidates.len(), 6);
        assert!(candidates.iter().any(|candidate| candidate
            .violations
            .contains(&SlotDeclarationViolation::ConflictingSemanticDecorators)));
        assert_eq!(
            candidates
                .iter()
                .filter(|candidate| candidate
                    .violations
                    .contains(&SlotDeclarationViolation::DuplicateSlot))
                .count(),
            2
        );
        assert!(candidates.iter().any(|candidate| {
            candidate
                .violations
                .contains(&SlotDeclarationViolation::InvalidDeclarationKind {
                    actual: AuthoredDeclarationKind::Method,
                })
        }));
        assert!(candidates.iter().any(|candidate| {
            candidate
                .violations
                .contains(&SlotDeclarationViolation::InvalidDeclarationKind {
                    actual: AuthoredDeclarationKind::Getter,
                })
        }));
        assert!(candidates.iter().any(|candidate| {
            candidate
                .violations
                .contains(&SlotDeclarationViolation::InvalidDeclarationKind {
                    actual: AuthoredDeclarationKind::Parameter,
                })
        }));
    }

    #[test]
    fn slot_order_is_canonical_across_components_and_fields() {
        let parsed = ezc_parser::parse_file(
            "src/Slots.tsx",
            r#"
@component("x-z") class Z extends Component {
  @slot() zebra!: SlotContent;
  @slot() alpha!: SlotContent;
  render() { return <main />; }
}
@component("x-a") class A extends Component {
  @slot() children!: SlotContent;
  render() { return <main />; }
}
"#,
        );
        let asm = build_application_semantic_model(&parsed);
        let ids = asm.slots.keys().map(SlotId::as_str).collect::<Vec<_>>();
        let mut sorted = ids.clone();
        sorted.sort_unstable();
        assert_eq!(ids, sorted);
    }

    #[test]
    fn keeps_slots_out_of_the_frozen_phase_g_semantic_graph_schema() {
        let parsed = ezc_parser::parse_file(
            "src/Card.tsx",
            r#"
@component("x-card")
class Card extends Component {
  @slot() children!: SlotContent;
  render() { return <section />; }
}
"#,
        );
        let asm = build_application_semantic_model(&parsed);
        let slot_id = asm
            .slots()
            .into_iter()
            .next()
            .expect("slot entity")
            .id
            .clone();
        let graph = build_semantic_graph(&asm);

        assert_eq!(graph.schema_version, 6);
        assert!(graph
            .nodes
            .iter()
            .all(|node| node.id != *slot_id.as_semantic_id()));
        assert!(graph.edges.iter().all(|edge| {
            edge.source != *slot_id.as_semantic_id() && edge.target != *slot_id.as_semantic_id()
        }));
    }
}
