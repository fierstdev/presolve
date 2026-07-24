//! K7 deterministic production chunk topology from accepted K6 candidates.

use std::collections::{BTreeMap, BTreeSet};

use crate::{
    ExecutableProgramFingerprint, ProductionChunkId, SharedChunkCandidate, SharedChunkCandidateId,
    SharedChunkCandidatePlan,
};

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum ProductionChunkKind {
    Eager,
    Root,
    Shared,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProductionRootChunkInput {
    pub activation_root_id: String,
    pub root_kind: String,
    pub programs: Vec<ExecutableProgramFingerprint>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProductionChunkRecord {
    pub id: ProductionChunkId,
    pub kind: ProductionChunkKind,
    pub activation_roots: Vec<String>,
    pub root_kind: Option<String>,
    pub programs: Vec<ExecutableProgramFingerprint>,
    pub registration_only: bool,
    pub provisional_module_filename: String,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct ProductionChunkDependency {
    pub dependent_chunk_id: ProductionChunkId,
    pub dependency_chunk_id: ProductionChunkId,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProductionSharedChunkFailurePolicy {
    FailDependentActivationWithoutRetry,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProductionActivationChunkPlan {
    pub activation_root_id: String,
    pub root_chunk_id: ProductionChunkId,
    pub shared_chunk_ids: Vec<ProductionChunkId>,
    pub shared_chunk_failure_policy: ProductionSharedChunkFailurePolicy,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProductionChunkGraph {
    pub eager_chunk_id: ProductionChunkId,
    pub chunks: Vec<ProductionChunkRecord>,
    pub dependencies: Vec<ProductionChunkDependency>,
    pub activation_plans: Vec<ProductionActivationChunkPlan>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProductionChunkExtractionReport {
    pub extracted_candidate_ids: Vec<SharedChunkCandidateId>,
    pub extracted_program_count: usize,
    pub root_chunk_count: usize,
    pub shared_chunk_count: usize,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum ProductionChunkGraphValidationError {
    DuplicateActivationRoot(String),
    DuplicateChunkId(ProductionChunkId),
    DuplicateDependency(ProductionChunkDependency),
    EagerChunkHasDependency(ProductionChunkId),
    ExpectedExactlyOneEagerChunk,
    InvalidActivationPlanRoot(ProductionChunkId),
    InvalidChunkDependency(ProductionChunkDependency),
    InvalidSharedChunkCandidate(SharedChunkCandidateId),
    InvalidSharedChunkOrder(String),
    MissingDependencyChunk(ProductionChunkId),
    ProductionChunkCycle,
    SharedChunkIsNotRegistrationOnly(ProductionChunkId),
    UnknownActivationPlanChunk(ProductionChunkId),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProductionChunkExtractionError {
    pub validation_errors: Vec<ProductionChunkGraphValidationError>,
}

/// Creates the exact one-eager, root-to-shared depth-one topology allowed by K7.
///
/// # Errors
///
/// Returns all deterministic validation errors when candidates or roots cannot
/// form that topology without changing their exact identities.
#[allow(clippy::too_many_lines)]
pub fn extract_production_chunk_graph(
    candidate_plan: &SharedChunkCandidatePlan,
    roots: &[ProductionRootChunkInput],
) -> Result<(ProductionChunkGraph, ProductionChunkExtractionReport), ProductionChunkExtractionError>
{
    let mut root_inputs = roots.to_vec();
    root_inputs.sort_by(|left, right| left.activation_root_id.cmp(&right.activation_root_id));
    let mut extraction_errors = Vec::new();
    if root_inputs
        .windows(2)
        .any(|pair| pair[0].activation_root_id == pair[1].activation_root_id)
    {
        extraction_errors.extend(
            root_inputs
                .windows(2)
                .filter(|pair| pair[0].activation_root_id == pair[1].activation_root_id)
                .map(|pair| {
                    ProductionChunkGraphValidationError::DuplicateActivationRoot(
                        pair[0].activation_root_id.clone(),
                    )
                }),
        );
    }

    let root_programs = root_inputs
        .iter()
        .map(|root| {
            (
                root.activation_root_id.clone(),
                root.programs.iter().cloned().collect::<BTreeSet<_>>(),
            )
        })
        .collect::<BTreeMap<_, _>>();
    let root_ids = root_programs.keys().cloned().collect::<BTreeSet<_>>();
    let mut candidates = candidate_plan.candidates.clone();
    candidates.sort_by(|left, right| left.id.cmp(&right.id));
    let mut extracted_by_root = BTreeMap::<String, BTreeSet<ExecutableProgramFingerprint>>::new();
    for candidate in &candidates {
        validate_candidate_consumers(
            candidate,
            &root_ids,
            &root_programs,
            &mut extracted_by_root,
            &mut extraction_errors,
        );
    }
    if !extraction_errors.is_empty() {
        extraction_errors.sort();
        extraction_errors.dedup();
        return Err(ProductionChunkExtractionError {
            validation_errors: extraction_errors,
        });
    }

    let eager_chunk_id = ProductionChunkId::eager_runtime_v1();
    let mut chunks = vec![ProductionChunkRecord {
        id: eager_chunk_id.clone(),
        kind: ProductionChunkKind::Eager,
        activation_roots: vec!["eager-runtime".to_string()],
        root_kind: None,
        programs: Vec::new(),
        registration_only: true,
        provisional_module_filename: "boot.pending-content-hash.js".to_string(),
    }];
    let mut dependencies = Vec::new();
    let mut shared_chunk_ids_by_root = BTreeMap::<String, Vec<ProductionChunkId>>::new();
    for candidate in &candidates {
        let Some(shared_chunk_id) = ProductionChunkId::for_activation_roots_and_programs(
            "shared",
            &candidate
                .consumers
                .iter()
                .map(|consumer| consumer.root_id.clone())
                .collect::<Vec<_>>(),
            &candidate.programs,
        ) else {
            return Err(ProductionChunkExtractionError {
                validation_errors: vec![
                    ProductionChunkGraphValidationError::InvalidSharedChunkCandidate(
                        candidate.id.clone(),
                    ),
                ],
            });
        };
        for consumer in &candidate.consumers {
            shared_chunk_ids_by_root
                .entry(consumer.root_id.clone())
                .or_default()
                .push(shared_chunk_id.clone());
        }
        chunks.push(ProductionChunkRecord {
            id: shared_chunk_id.clone(),
            kind: ProductionChunkKind::Shared,
            activation_roots: candidate
                .consumers
                .iter()
                .map(|consumer| consumer.root_id.clone())
                .collect(),
            root_kind: None,
            programs: candidate.programs.clone(),
            registration_only: true,
            provisional_module_filename: provisional_filename("shared", &shared_chunk_id),
        });
        dependencies.push(ProductionChunkDependency {
            dependent_chunk_id: shared_chunk_id,
            dependency_chunk_id: eager_chunk_id.clone(),
        });
    }

    let mut activation_plans = Vec::new();
    for root in root_inputs {
        let extracted = extracted_by_root
            .get(&root.activation_root_id)
            .cloned()
            .unwrap_or_default();
        let mut programs = root
            .programs
            .into_iter()
            .filter(|program| !extracted.contains(program))
            .collect::<Vec<_>>();
        programs.sort();
        programs.dedup();
        let root_chunk_id = ProductionChunkId::for_activation_roots_and_programs(
            &root.root_kind,
            std::slice::from_ref(&root.activation_root_id),
            &programs,
        )
        .ok_or_else(|| ProductionChunkExtractionError {
            validation_errors: vec![
                ProductionChunkGraphValidationError::InvalidActivationPlanRoot(
                    ProductionChunkId::eager_runtime_v1(),
                ),
            ],
        })?;
        let mut shared_chunk_ids = shared_chunk_ids_by_root
            .remove(&root.activation_root_id)
            .unwrap_or_default();
        shared_chunk_ids.sort();
        shared_chunk_ids.dedup();
        chunks.push(ProductionChunkRecord {
            id: root_chunk_id.clone(),
            kind: ProductionChunkKind::Root,
            activation_roots: vec![root.activation_root_id.clone()],
            root_kind: Some(root.root_kind.clone()),
            programs,
            registration_only: false,
            provisional_module_filename: provisional_filename(
                &format!("root.{}", root.root_kind),
                &root_chunk_id,
            ),
        });
        dependencies.push(ProductionChunkDependency {
            dependent_chunk_id: root_chunk_id.clone(),
            dependency_chunk_id: eager_chunk_id.clone(),
        });
        dependencies.extend(shared_chunk_ids.iter().cloned().map(|dependency_chunk_id| {
            ProductionChunkDependency {
                dependent_chunk_id: root_chunk_id.clone(),
                dependency_chunk_id,
            }
        }));
        activation_plans.push(ProductionActivationChunkPlan {
            activation_root_id: root.activation_root_id,
            root_chunk_id,
            shared_chunk_ids,
            shared_chunk_failure_policy:
                ProductionSharedChunkFailurePolicy::FailDependentActivationWithoutRetry,
        });
    }
    chunks.sort_by(|left, right| left.id.cmp(&right.id));
    dependencies.sort();
    dependencies.dedup();
    activation_plans.sort_by(|left, right| left.activation_root_id.cmp(&right.activation_root_id));
    let graph = ProductionChunkGraph {
        eager_chunk_id,
        chunks,
        dependencies,
        activation_plans,
    };
    validate_production_chunk_graph(&graph)
        .map_err(|validation_errors| ProductionChunkExtractionError { validation_errors })?;
    let report = ProductionChunkExtractionReport {
        extracted_candidate_ids: candidates
            .iter()
            .map(|candidate| candidate.id.clone())
            .collect(),
        extracted_program_count: candidates
            .iter()
            .map(|candidate| candidate.programs.len())
            .sum(),
        root_chunk_count: roots.len(),
        shared_chunk_count: candidates.len(),
    };
    Ok((graph, report))
}

/// Validates ordering, identity, dependency depth, and activation safety.
///
/// # Errors
///
/// Returns stable validation evidence when the graph is not Section 12 topology.
pub fn validate_production_chunk_graph(
    graph: &ProductionChunkGraph,
) -> Result<(), Vec<ProductionChunkGraphValidationError>> {
    let mut errors = Vec::new();
    let chunks = graph
        .chunks
        .iter()
        .map(|chunk| (chunk.id.clone(), chunk))
        .collect::<BTreeMap<_, _>>();
    if graph.chunks.windows(2).any(|pair| pair[0].id == pair[1].id) {
        errors.extend(
            graph
                .chunks
                .windows(2)
                .filter(|pair| pair[0].id == pair[1].id)
                .map(|pair| {
                    ProductionChunkGraphValidationError::DuplicateChunkId(pair[0].id.clone())
                }),
        );
    }
    let eager_chunks = graph
        .chunks
        .iter()
        .filter(|chunk| chunk.kind == ProductionChunkKind::Eager)
        .collect::<Vec<_>>();
    if eager_chunks.len() != 1
        || eager_chunks.first().map(|chunk| &chunk.id) != Some(&graph.eager_chunk_id)
    {
        errors.push(ProductionChunkGraphValidationError::ExpectedExactlyOneEagerChunk);
    }
    errors.extend(
        graph
            .chunks
            .iter()
            .filter(|chunk| chunk.kind == ProductionChunkKind::Shared && !chunk.registration_only)
            .map(|chunk| {
                ProductionChunkGraphValidationError::SharedChunkIsNotRegistrationOnly(
                    chunk.id.clone(),
                )
            }),
    );
    let mut seen_dependencies = BTreeSet::new();
    for dependency in &graph.dependencies {
        if !seen_dependencies.insert(dependency.clone()) {
            errors.push(ProductionChunkGraphValidationError::DuplicateDependency(
                dependency.clone(),
            ));
        }
        let Some(dependent) = chunks.get(&dependency.dependent_chunk_id) else {
            errors.push(ProductionChunkGraphValidationError::MissingDependencyChunk(
                dependency.dependent_chunk_id.clone(),
            ));
            continue;
        };
        let Some(target) = chunks.get(&dependency.dependency_chunk_id) else {
            errors.push(ProductionChunkGraphValidationError::MissingDependencyChunk(
                dependency.dependency_chunk_id.clone(),
            ));
            continue;
        };
        let valid = matches!(
            (dependent.kind, target.kind),
            (ProductionChunkKind::Shared, ProductionChunkKind::Eager)
                | (
                    ProductionChunkKind::Root,
                    ProductionChunkKind::Eager | ProductionChunkKind::Shared
                )
        );
        if !valid {
            errors.push(if dependent.kind == ProductionChunkKind::Eager {
                ProductionChunkGraphValidationError::EagerChunkHasDependency(dependent.id.clone())
            } else {
                ProductionChunkGraphValidationError::InvalidChunkDependency(dependency.clone())
            });
        }
    }
    errors.extend(validate_activation_plans(graph, &chunks));
    errors.extend(validate_cycle(graph, &chunks));
    errors.sort();
    errors.dedup();
    errors.is_empty().then_some(()).ok_or(errors)
}

fn validate_candidate_consumers(
    candidate: &SharedChunkCandidate,
    root_ids: &BTreeSet<String>,
    root_programs: &BTreeMap<String, BTreeSet<ExecutableProgramFingerprint>>,
    extracted_by_root: &mut BTreeMap<String, BTreeSet<ExecutableProgramFingerprint>>,
    errors: &mut Vec<ProductionChunkGraphValidationError>,
) {
    for consumer in &candidate.consumers {
        if !root_ids.contains(&consumer.root_id) {
            errors.push(
                ProductionChunkGraphValidationError::UnknownActivationPlanChunk(
                    ProductionChunkId::eager_runtime_v1(),
                ),
            );
            continue;
        }
        let Some(programs) = root_programs.get(&consumer.root_id) else {
            continue;
        };
        if !candidate
            .programs
            .iter()
            .all(|program| programs.contains(program))
        {
            errors.push(
                ProductionChunkGraphValidationError::InvalidActivationPlanRoot(
                    ProductionChunkId::eager_runtime_v1(),
                ),
            );
            continue;
        }
        extracted_by_root
            .entry(consumer.root_id.clone())
            .or_default()
            .extend(candidate.programs.iter().cloned());
    }
}

fn validate_activation_plans(
    graph: &ProductionChunkGraph,
    chunks: &BTreeMap<ProductionChunkId, &ProductionChunkRecord>,
) -> Vec<ProductionChunkGraphValidationError> {
    let mut errors = Vec::new();
    let mut roots = BTreeSet::new();
    for plan in &graph.activation_plans {
        if !roots.insert(plan.activation_root_id.clone()) {
            errors.push(
                ProductionChunkGraphValidationError::DuplicateActivationRoot(
                    plan.activation_root_id.clone(),
                ),
            );
        }
        if chunks.get(&plan.root_chunk_id).map(|chunk| chunk.kind)
            != Some(ProductionChunkKind::Root)
        {
            errors.push(
                ProductionChunkGraphValidationError::InvalidActivationPlanRoot(
                    plan.root_chunk_id.clone(),
                ),
            );
        }
        let mut shared = plan.shared_chunk_ids.clone();
        shared.sort();
        shared.dedup();
        if shared != plan.shared_chunk_ids {
            errors.push(
                ProductionChunkGraphValidationError::InvalidSharedChunkOrder(
                    plan.activation_root_id.clone(),
                ),
            );
        }
        for shared_chunk_id in &plan.shared_chunk_ids {
            match chunks.get(shared_chunk_id) {
                Some(chunk)
                    if chunk.kind == ProductionChunkKind::Shared && chunk.registration_only => {}
                Some(_) => errors.push(
                    ProductionChunkGraphValidationError::SharedChunkIsNotRegistrationOnly(
                        shared_chunk_id.clone(),
                    ),
                ),
                None => errors.push(
                    ProductionChunkGraphValidationError::UnknownActivationPlanChunk(
                        shared_chunk_id.clone(),
                    ),
                ),
            }
        }
    }
    errors
}

fn validate_cycle(
    graph: &ProductionChunkGraph,
    chunks: &BTreeMap<ProductionChunkId, &ProductionChunkRecord>,
) -> Vec<ProductionChunkGraphValidationError> {
    fn visit(
        current: &ProductionChunkId,
        dependencies: &BTreeMap<ProductionChunkId, Vec<ProductionChunkId>>,
        visiting: &mut BTreeSet<ProductionChunkId>,
        visited: &mut BTreeSet<ProductionChunkId>,
    ) -> bool {
        if !visiting.insert(current.clone()) {
            return true;
        }
        if let Some(next) = dependencies.get(current) {
            for dependency in next {
                if !visited.contains(dependency)
                    && visit(dependency, dependencies, visiting, visited)
                {
                    return true;
                }
            }
        }
        visiting.remove(current);
        visited.insert(current.clone());
        false
    }

    let mut dependencies = BTreeMap::<ProductionChunkId, Vec<ProductionChunkId>>::new();
    for dependency in &graph.dependencies {
        dependencies
            .entry(dependency.dependent_chunk_id.clone())
            .or_default()
            .push(dependency.dependency_chunk_id.clone());
    }
    let mut visiting = BTreeSet::new();
    let mut visited = BTreeSet::new();
    for chunk_id in chunks.keys() {
        if !visited.contains(chunk_id)
            && visit(chunk_id, &dependencies, &mut visiting, &mut visited)
        {
            return vec![ProductionChunkGraphValidationError::ProductionChunkCycle];
        }
    }
    Vec::new()
}

fn provisional_filename(prefix: &str, chunk_id: &ProductionChunkId) -> String {
    let short_id = chunk_id
        .as_str()
        .rsplit_once(':')
        .map_or(chunk_id.as_str(), |(_, suffix)| suffix)
        .chars()
        .take(12)
        .collect::<String>();
    format!("{prefix}.{short_id}.pending-content-hash.js")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{plan_shared_lazy_chunk_candidates, SharedChunkProgramOccurrence};

    fn fingerprint(value: &str) -> ExecutableProgramFingerprint {
        ExecutableProgramFingerprint::for_canonical_opcode_stream(value.as_bytes())
    }

    fn root(
        root_id: &str,
        programs: Vec<ExecutableProgramFingerprint>,
    ) -> ProductionRootChunkInput {
        ProductionRootChunkInput {
            activation_root_id: root_id.to_string(),
            root_kind: "interaction".to_string(),
            programs,
        }
    }

    fn occurrence(root_id: &str, program: &str) -> SharedChunkProgramOccurrence {
        SharedChunkProgramOccurrence {
            root_id: root_id.to_string(),
            fingerprint: fingerprint(program),
            canonical_bytes: vec![b'x'; 300],
            eager_required: false,
            root_specific: false,
            captures_mutable_identity: false,
            registration_only: true,
            runtime_protocol: "v1".to_string(),
        }
    }

    #[test]
    fn k7_extracts_shared_programs_and_preserves_root_identity() {
        let shared = fingerprint("shared");
        let candidates = plan_shared_lazy_chunk_candidates(
            &[
                occurrence("root-a", "shared"),
                occurrence("root-b", "shared"),
            ],
            0,
            0,
        );
        let (graph, report) = extract_production_chunk_graph(
            &candidates,
            &[
                root("root-b", vec![shared.clone(), fingerprint("b-only")]),
                root("root-a", vec![shared.clone(), fingerprint("a-only")]),
            ],
        )
        .expect("eligible candidate extracts");
        assert_eq!(report.shared_chunk_count, 1);
        assert_eq!(graph.activation_plans.len(), 2);
        assert_ne!(
            graph.activation_plans[0].root_chunk_id,
            graph.activation_plans[1].root_chunk_id
        );
        assert_eq!(
            graph.activation_plans[0].shared_chunk_failure_policy,
            ProductionSharedChunkFailurePolicy::FailDependentActivationWithoutRetry
        );
        assert_eq!(
            graph.activation_plans[0].shared_chunk_ids,
            graph.activation_plans[1].shared_chunk_ids
        );
        assert!(graph
            .chunks
            .iter()
            .filter(|chunk| chunk.kind == ProductionChunkKind::Root)
            .all(|chunk| !chunk.programs.contains(&shared)));
        assert!(graph.chunks.iter().any(|chunk| {
            chunk.kind == ProductionChunkKind::Shared
                && chunk.registration_only
                && chunk.programs == vec![shared.clone()]
        }));
        assert_eq!(
            graph
                .chunks
                .iter()
                .filter(|chunk| chunk.kind == ProductionChunkKind::Shared)
                .count(),
            1
        );
    }

    #[test]
    fn k7_keeps_roots_independent_when_no_candidate_extracts() {
        let (graph, report) = extract_production_chunk_graph(
            &SharedChunkCandidatePlan {
                candidates: Vec::new(),
                rejections: Vec::new(),
            },
            &[
                root("root-a", vec![fingerprint("a")]),
                root("root-b", vec![fingerprint("b")]),
            ],
        )
        .expect("roots without sharing remain valid");
        assert_eq!(report.shared_chunk_count, 0);
        assert!(graph
            .activation_plans
            .iter()
            .all(|plan| plan.shared_chunk_ids.is_empty()));
        assert_eq!(
            graph
                .chunks
                .iter()
                .filter(|chunk| chunk.kind == ProductionChunkKind::Eager)
                .count(),
            1
        );
    }

    #[test]
    fn k7_rejects_cycle_and_depth_two_dependencies() {
        let (mut graph, _) = extract_production_chunk_graph(
            &SharedChunkCandidatePlan {
                candidates: Vec::new(),
                rejections: Vec::new(),
            },
            &[root("root-a", vec![fingerprint("a")])],
        )
        .expect("base graph");
        let root_chunk = graph.activation_plans[0].root_chunk_id.clone();
        graph.dependencies.push(ProductionChunkDependency {
            dependent_chunk_id: graph.eager_chunk_id.clone(),
            dependency_chunk_id: root_chunk,
        });
        let errors =
            validate_production_chunk_graph(&graph).expect_err("cycle and eager edge reject");
        assert!(errors.contains(&ProductionChunkGraphValidationError::ProductionChunkCycle));
        assert!(errors.iter().any(|error| matches!(
            error,
            ProductionChunkGraphValidationError::EagerChunkHasDependency(_)
        )));
    }

    #[test]
    fn k7_rejects_shared_to_shared_depth() {
        let shared_one = fingerprint("shared-one");
        let shared_two = fingerprint("shared-two");
        let candidates = plan_shared_lazy_chunk_candidates(
            &[
                occurrence("root-a", "shared-one"),
                occurrence("root-b", "shared-one"),
                occurrence("root-b", "shared-two"),
                occurrence("root-c", "shared-two"),
            ],
            0,
            0,
        );
        let (mut graph, _) = extract_production_chunk_graph(
            &candidates,
            &[
                root("root-a", vec![shared_one]),
                root("root-b", vec![shared_two, fingerprint("shared-one")]),
                root("root-c", vec![fingerprint("shared-two")]),
            ],
        )
        .expect("two exact candidate root sets");
        let shared_chunks = graph
            .chunks
            .iter()
            .filter(|chunk| chunk.kind == ProductionChunkKind::Shared)
            .map(|chunk| chunk.id.clone())
            .collect::<Vec<_>>();
        assert_eq!(shared_chunks.len(), 2);
        graph.dependencies.push(ProductionChunkDependency {
            dependent_chunk_id: shared_chunks[0].clone(),
            dependency_chunk_id: shared_chunks[1].clone(),
        });
        let errors = validate_production_chunk_graph(&graph).expect_err("depth two rejects");
        assert!(errors.iter().any(|error| matches!(
            error,
            ProductionChunkGraphValidationError::InvalidChunkDependency(_)
        )));
    }

    #[test]
    fn k7_graph_is_deterministic_under_reversed_input_order() {
        let shared = fingerprint("shared");
        let candidates = plan_shared_lazy_chunk_candidates(
            &[
                occurrence("root-a", "shared"),
                occurrence("root-b", "shared"),
            ],
            0,
            0,
        );
        let first = extract_production_chunk_graph(
            &candidates,
            &[
                root("root-a", vec![shared.clone()]),
                root("root-b", vec![shared.clone()]),
            ],
        );
        let second = extract_production_chunk_graph(
            &candidates,
            &[
                root("root-b", vec![shared.clone()]),
                root("root-a", vec![shared]),
            ],
        );
        assert_eq!(first, second);
    }
}
