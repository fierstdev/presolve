use std::collections::BTreeMap;

use crate::{
    AuthoredContextDeclarationCandidate, ConsumerEntity, ConsumerId, ContextDeclarationCandidateId,
    ContextDeclarationCandidateKind, ContextDeclarationViolation, ContextEntity, ContextId,
    ProviderEntity, ProviderId, SemanticId,
};

/// The only semantic links a valid Context-family declaration candidate may hold.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ContextSemanticEntityId {
    Context(ContextId),
    Provider(ProviderId),
    Consumer(ConsumerId),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ContextDeclarationStatus {
    Valid(ContextSemanticEntityId),
    Invalid(Vec<ContextDeclarationViolation>),
}

/// Immutable G18 diagnostic authority.  It is assembled from parser-retained
/// facts and already-lowered compiler products; no source is revisited.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContextDeclarationCandidate {
    pub authored: AuthoredContextDeclarationCandidate,
    pub status: ContextDeclarationStatus,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContextDeclarationCandidateRegistry {
    candidates: Vec<ContextDeclarationCandidate>,
    by_id: BTreeMap<ContextDeclarationCandidateId, usize>,
}

impl ContextDeclarationCandidateRegistry {
    #[must_use]
    pub fn candidates(&self) -> &[ContextDeclarationCandidate] {
        &self.candidates
    }

    #[must_use]
    pub fn candidate(
        &self,
        id: &ContextDeclarationCandidateId,
    ) -> Option<&ContextDeclarationCandidate> {
        self.by_id
            .get(id)
            .and_then(|index| self.candidates.get(*index))
    }

    #[must_use]
    pub fn invalid_candidates(&self) -> Vec<&ContextDeclarationCandidate> {
        self.candidates
            .iter()
            .filter(|candidate| matches!(candidate.status, ContextDeclarationStatus::Invalid(_)))
            .collect()
    }
}

/// Builds the immutable G18 diagnostic authority from retained declaration facts.
///
/// # Panics
///
/// Panics if an internally inconsistent candidate has no hard violation but
/// cannot be linked to its required semantic entity.
#[must_use]
pub fn collect_context_declaration_candidates(
    components: &[crate::ComponentNode],
    contexts: &BTreeMap<ContextId, ContextEntity>,
    providers: &BTreeMap<ProviderId, ProviderEntity>,
    consumers: &BTreeMap<ConsumerId, ConsumerEntity>,
) -> ContextDeclarationCandidateRegistry {
    let mut candidates = components
        .iter()
        .flat_map(|component| component.context_declaration_candidates.iter())
        .cloned()
        .map(|authored| {
            let mut violations = authored.violations.clone();
            let link = match authored.kind {
                ContextDeclarationCandidateKind::Context => entity_for_context(&authored, contexts),
                ContextDeclarationCandidateKind::Provider => {
                    entity_for_provider(&authored, providers)
                }
                ContextDeclarationCandidateKind::Consumer => {
                    entity_for_consumer(&authored, consumers)
                }
            };
            if violations.is_empty()
                && link.is_none()
                && matches!(
                    authored.kind,
                    ContextDeclarationCandidateKind::Provider
                        | ContextDeclarationCandidateKind::Consumer
                )
            {
                violations.push(ContextDeclarationViolation::UnresolvedContextDesignator);
            }
            let status = if violations.is_empty() {
                ContextDeclarationStatus::Valid(
                    link.unwrap_or_else(|| unreachable!("structurally valid candidate must lower")),
                )
            } else {
                violations.sort_by_key(violation_rank);
                violations.dedup();
                ContextDeclarationStatus::Invalid(violations)
            };
            ContextDeclarationCandidate { authored, status }
        })
        .collect::<Vec<_>>();

    // A duplicate Provider group has no winner.  Existing lowering keeps the
    // first entity for historical G4 recovery, so retain the candidate facts
    // and make all source candidates invalid for G18 without changing G4.
    let mut groups = BTreeMap::<(SemanticId, String, String), Vec<usize>>::new();
    for (index, candidate) in candidates.iter().enumerate() {
        if candidate.authored.kind == ContextDeclarationCandidateKind::Provider {
            if let Some(designator) = &candidate.authored.context_designator {
                groups
                    .entry((
                        candidate.authored.owner_component.clone(),
                        designator.component_symbol.clone(),
                        designator.context_member.clone(),
                    ))
                    .or_default()
                    .push(index);
            }
        }
    }
    for group in groups.values().filter(|group| group.len() > 1) {
        for index in group {
            let candidate = &mut candidates[*index];
            match &mut candidate.status {
                ContextDeclarationStatus::Valid(_) => {
                    candidate.status = ContextDeclarationStatus::Invalid(vec![
                        ContextDeclarationViolation::DuplicateProvider,
                    ]);
                }
                ContextDeclarationStatus::Invalid(violations) => {
                    violations.push(ContextDeclarationViolation::DuplicateProvider);
                    violations.sort_by_key(violation_rank);
                    violations.dedup();
                }
            }
        }
    }
    candidates.sort_by(|left, right| left.authored.id.cmp(&right.authored.id));
    let by_id = candidates
        .iter()
        .enumerate()
        .map(|(index, candidate)| (candidate.authored.id.clone(), index))
        .collect();
    ContextDeclarationCandidateRegistry { candidates, by_id }
}

fn entity_for_context(
    candidate: &AuthoredContextDeclarationCandidate,
    contexts: &BTreeMap<ContextId, ContextEntity>,
) -> Option<ContextSemanticEntityId> {
    contexts
        .values()
        .find(|entity| {
            entity.owner.entity_id() == Some(&candidate.owner_component)
                && candidate.field_name.as_deref() == Some(&entity.name)
        })
        .map(|entity| ContextSemanticEntityId::Context(entity.id.clone()))
}
fn entity_for_provider(
    candidate: &AuthoredContextDeclarationCandidate,
    providers: &BTreeMap<ProviderId, ProviderEntity>,
) -> Option<ContextSemanticEntityId> {
    providers
        .values()
        .find(|entity| {
            entity.owner.entity_id() == Some(&candidate.owner_component)
                && candidate.field_name.as_deref() == Some(&entity.name)
        })
        .map(|entity| ContextSemanticEntityId::Provider(entity.id.clone()))
}
fn entity_for_consumer(
    candidate: &AuthoredContextDeclarationCandidate,
    consumers: &BTreeMap<ConsumerId, ConsumerEntity>,
) -> Option<ContextSemanticEntityId> {
    consumers
        .values()
        .find(|entity| {
            entity.owner.entity_id() == Some(&candidate.owner_component)
                && candidate.field_name.as_deref() == Some(&entity.name)
        })
        .map(|entity| ContextSemanticEntityId::Consumer(entity.id.clone()))
}
fn violation_rank(violation: &ContextDeclarationViolation) -> u8 {
    match violation {
        ContextDeclarationViolation::InvalidDeclarationKind { .. }
        | ContextDeclarationViolation::ConflictingSemanticDecorators => 0,
        ContextDeclarationViolation::StaticDeclarationUnsupported => 1,
        ContextDeclarationViolation::DecoratorArity { .. } => 2,
        ContextDeclarationViolation::ContextDesignatorUnsupported => 3,
        ContextDeclarationViolation::UnresolvedContextDesignator => 4,
        ContextDeclarationViolation::MissingDeclaredType => 5,
        ContextDeclarationViolation::MissingInitializer
        | ContextDeclarationViolation::ForbiddenInitializer
        | ContextDeclarationViolation::UnsupportedInitializer
        | ContextDeclarationViolation::DefiniteAssignmentRequired => 6,
        ContextDeclarationViolation::DuplicateProvider => 7,
    }
}
