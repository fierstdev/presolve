//! K9 repository-owned deterministic production module emission.

use std::collections::BTreeMap;
use std::fmt::Write as _;

use sha2::{Digest, Sha256};

use crate::{ProductionChunkGraph, ProductionChunkId, ProductionChunkKind};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProductionModuleRecord {
    pub chunk_id: ProductionChunkId,
    pub filename: String,
    pub source: String,
    pub exports: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProductionModuleLayout {
    pub eager: ProductionModuleRecord,
    pub shared: Vec<ProductionModuleRecord>,
    pub roots: Vec<ProductionModuleRecord>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ProductionModuleValidationError {
    DynamicCode,
    InvalidExport,
    NonCanonicalImportOrder,
    SourceComment,
}

/// Emits a fixed, content-addressed module layout from the validated K7 graph.
///
/// # Panics
///
/// Panics only when a caller supplies a graph lacking its compiler-owned eager
/// chunk, which is an earlier graph-integrity failure.
#[must_use]
pub fn emit_production_modules(graph: &ProductionChunkGraph) -> ProductionModuleLayout {
    let eager_chunk = graph
        .chunks
        .iter()
        .find(|chunk| chunk.kind == ProductionChunkKind::Eager)
        .expect("validated graph contains one eager chunk");
    let eager = module(eager_chunk.id.clone(), "boot", Vec::new());
    let shared_by_id = graph
        .chunks
        .iter()
        .filter(|chunk| chunk.kind == ProductionChunkKind::Shared)
        .map(|chunk| {
            (
                chunk.id.clone(),
                module(chunk.id.clone(), "shared", vec![eager.filename.clone()]),
            )
        })
        .collect::<BTreeMap<_, _>>();
    let shared = shared_by_id.values().cloned().collect::<Vec<_>>();
    let mut roots = graph
        .chunks
        .iter()
        .filter(|chunk| chunk.kind == ProductionChunkKind::Root)
        .map(|chunk| {
            let mut imports = graph
                .dependencies
                .iter()
                .filter(|edge| edge.dependent_chunk_id == chunk.id)
                .filter_map(|edge| shared_by_id.get(&edge.dependency_chunk_id))
                .map(|shared| shared.filename.clone())
                .collect::<Vec<_>>();
            imports.push(eager.filename.clone());
            module(chunk.id.clone(), "root", imports)
        })
        .collect::<Vec<_>>();
    roots.sort_by(|left, right| left.chunk_id.cmp(&right.chunk_id));
    ProductionModuleLayout {
        eager,
        shared,
        roots,
    }
}

/// Rejects unsupported generated-JavaScript constructs before production output.
#[must_use]
pub fn validate_production_module(
    module: &ProductionModuleRecord,
) -> Vec<ProductionModuleValidationError> {
    let mut errors = Vec::new();
    if module.source.contains("eval(")
        || module.source.contains("Function(")
        || module.source.contains("import(")
    {
        errors.push(ProductionModuleValidationError::DynamicCode);
    }
    if module.source.contains("//") || module.source.contains("/*") {
        errors.push(ProductionModuleValidationError::SourceComment);
    }
    if module.exports
        != vec![
            "productionChunkId".to_string(),
            "registerProductionChunk".to_string(),
        ]
    {
        errors.push(ProductionModuleValidationError::InvalidExport);
    }
    let imports = module
        .source
        .lines()
        .filter_map(|line| {
            line.strip_prefix("import{registerProductionChunk as a}from\"./")
                .and_then(|line| line.strip_suffix("\";"))
        })
        .collect::<Vec<_>>();
    if imports.windows(2).any(|pair| pair[0] > pair[1]) {
        errors.push(ProductionModuleValidationError::NonCanonicalImportOrder);
    }
    errors
}

fn module(
    chunk_id: ProductionChunkId,
    prefix: &str,
    mut imports: Vec<String>,
) -> ProductionModuleRecord {
    imports.sort();
    imports.dedup();
    let mut import_text = String::new();
    for filename in &imports {
        write!(
            import_text,
            "import{{registerProductionChunk as a}}from\"./{filename}\";"
        )
        .expect("writing to a string should not fail");
    }
    let source = format!(
        "{import_text}export const productionChunkId=\"{chunk_id}\";export function registerProductionChunk(){{return productionChunkId;}}\n"
    );
    let filename = format!("{prefix}.{}.js", short_hash(&source));
    ProductionModuleRecord {
        chunk_id,
        filename,
        source,
        exports: vec![
            "productionChunkId".to_string(),
            "registerProductionChunk".to_string(),
        ],
    }
}

fn short_hash(source: &str) -> String {
    format!("{:x}", Sha256::digest(source.as_bytes()))[..16].to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        extract_production_chunk_graph, ExecutableProgramFingerprint, ProductionRootChunkInput,
        SharedChunkCandidatePlan,
    };

    #[test]
    fn k9_emits_deterministic_syntax_safe_modules() {
        let graph = extract_production_chunk_graph(
            &SharedChunkCandidatePlan {
                candidates: Vec::new(),
                rejections: Vec::new(),
            },
            &[ProductionRootChunkInput {
                activation_root_id: "root".to_string(),
                root_kind: "interaction".to_string(),
                programs: vec![ExecutableProgramFingerprint::for_canonical_opcode_stream(
                    b"return",
                )],
            }],
        )
        .expect("graph")
        .0;
        let first = emit_production_modules(&graph);
        let second = emit_production_modules(&graph);
        assert_eq!(first, second);
        assert!(first.eager.filename.starts_with("boot."));
        assert!(first
            .roots
            .iter()
            .all(|module| validate_production_module(module).is_empty()));
        assert!(validate_production_module(&first.eager).is_empty());
    }
}
