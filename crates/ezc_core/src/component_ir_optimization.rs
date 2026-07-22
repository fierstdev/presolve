use crate::{ComponentIrReport, SourceProvenance};

/// Immutable H12 component-IR optimization result.
///
/// Component creation, Context initialization, template materialization, and
/// Slot placement are observable roots. The H11 lowering already excludes
/// blocked and statically empty work, so this first optimization projection
/// intentionally retains the complete operation stream unchanged.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct OptimizedComponentIrReport {
    pub source_report: ComponentIrReport,
    pub optimized_report: ComponentIrReport,
    pub eliminated_instruction_provenance: Vec<SourceProvenance>,
}

/// Optimizes only semantically unobservable component work.
///
/// No current H11 instruction satisfies that condition. Keeping the identity
/// projection explicit makes the H12 boundary authoritative while preserving
/// every instance, Context slot, Slot binding, and observable order.
#[must_use]
pub fn optimize_component_ir(report: &ComponentIrReport) -> OptimizedComponentIrReport {
    OptimizedComponentIrReport {
        source_report: report.clone(),
        optimized_report: report.clone(),
        eliminated_instruction_provenance: Vec::new(),
    }
}

#[must_use]
pub fn validate_optimized_component_ir(report: &OptimizedComponentIrReport) -> Vec<String> {
    let expected = optimize_component_ir(&report.source_report);
    (report != &expected)
        .then(|| {
            "optimized component IR does not preserve the canonical observable stream".to_string()
        })
        .into_iter()
        .collect()
}

#[cfg(test)]
mod tests {
    use crate::{
        build_application_semantic_model, optimize_component_ir, validate_optimized_component_ir,
    };

    #[test]
    fn preserves_every_observable_component_identity_and_order() {
        let model = build_application_semantic_model(&ezc_parser::parse_file(
            "src/Optimize.tsx",
            r#"
@component("x-card") class Card extends Component { @slot() children!: SlotContent; render() { return <article><slot /></article>; } }
@component("x-page") class Page extends Component { render() { return <Card><p /></Card>; } }
"#,
        ));
        let optimized = optimize_component_ir(&model.component_ir);
        assert_eq!(optimized.source_report, model.component_ir);
        assert_eq!(optimized.optimized_report, model.component_ir);
        assert!(optimized.eliminated_instruction_provenance.is_empty());
        assert!(validate_optimized_component_ir(&optimized).is_empty());
    }

    #[test]
    fn asm_validation_rejects_mutated_optimized_component_ir() {
        let mut model = build_application_semantic_model(&ezc_parser::parse_file(
            "src/OptimizeValidate.tsx",
            r#"@component("x-page") class Page extends Component { render() { return <main />; } }"#,
        ));
        model
            .component_ir_optimization
            .optimized_report
            .instructions
            .clear();
        assert!(crate::validate_application_semantic_model(&model)
            .iter()
            .any(|diagnostic| diagnostic.code == "PSASM1200"));
    }
}
