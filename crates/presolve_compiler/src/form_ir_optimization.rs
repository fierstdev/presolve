//! I13 immutable Form IR optimization boundary.
use crate::FormIrReport;
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct FormIrOptimizationMetrics {
    pub constant_folds: usize,
    pub deduplicated_pure_reads: usize,
    pub removed_dead_pure_values: usize,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OptimizedFormIrReport {
    pub source: FormIrReport,
    pub optimized: FormIrReport,
    pub metrics: FormIrOptimizationMetrics,
    pub immutable_input: bool,
}
#[must_use]
pub fn optimize_form_ir(source: &FormIrReport) -> OptimizedFormIrReport {
    OptimizedFormIrReport {
        source: source.clone(),
        optimized: source.clone(),
        metrics: FormIrOptimizationMetrics::default(),
        immutable_input: true,
    }
}
#[cfg(test)]
mod tests {
    #[test]
    fn preserves_every_instance_and_operation() {
        let parsed = presolve_parser::parse_file(
            "src/X.tsx",
            r#"@component("x") class X{@form() form!:Form; @field(this.form) value="";render(){return <input field={this.value}/>;}}"#,
        );
        let model = crate::build_application_semantic_model(&parsed);
        assert_eq!(model.optimized_form_ir.source, model.form_ir);
        assert_eq!(model.optimized_form_ir.optimized, model.form_ir);
    }
}
