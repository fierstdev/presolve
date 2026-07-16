use std::collections::BTreeMap;
use std::env;
use std::fmt::Write as _;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process;

use ezc_core::{
    build_application_semantic_model_for_unit, build_component_graph,
    build_context_inspection_registry, build_effect_inspection_registry,
    build_runtime_component_artifact, build_runtime_computed_artifact,
    build_runtime_context_artifact, build_runtime_effect_artifact, build_runtime_forms_artifact,
    build_semantic_graph, build_template_graph, build_template_manifest_from_asm, explain_json,
    explain_text, fold_component_graph, generate_runtime_stub,
    generate_standalone_page_with_component_runtime_and_forms, generate_static_html,
    lower_components_to_ir, optimize_context_ir, optimize_effect_ir,
    runtime_component_artifact_json, runtime_computed_artifact_json, runtime_context_artifact_json,
    runtime_effect_artifact_json, runtime_forms_artifact_json, semantic_graph_json,
    semantic_type_text, summarize_source, template_manifest_json,
    validate_application_semantic_model, ApplicationSemanticModel, AsmValidationDiagnostic,
    AttributeValue, CompilationUnit, ComponentGraph, ConstantFoldingPass, DeclaredStateTypeKind,
    EffectInspection, EffectInspectionRegistry, ImmutableAsmPass, RenderAttribute,
    RenderAttributeValue, SemanticEntity, SemanticEntityKind, SemanticId, SemanticOwner,
    SemanticReferenceKind, SerializableValue, SourceProvenance, StateOperation, TemplateChild,
    TemplateGraph, TemplateSemanticKind,
};
use ezc_parser::{
    parse_file, ParseDiagnostic, ParseSeverity, ParsedClass, ParsedFile, ParsedJsxAttribute,
    ParsedJsxAttributeValue, ParsedJsxChild, ParsedJsxNode, ParsedMethod, SourceSpan,
};
use serde::Serialize;

const ASM_INSPECTION_SCHEMA_VERSION: u32 = 8;
const CHECK_JSON_SCHEMA_VERSION: u32 = 4;

fn main() {
    let mut args = env::args().skip(1).collect::<Vec<_>>();

    if args.is_empty() {
        print_usage_and_exit();
    }

    let command = args.remove(0);

    match command.as_str() {
        "explain" => run_explain(args),
        "parse" => run_parse(args),
        "graph" => run_graph(args),
        "asm" => run_asm(&args),
        "check" => run_check(&args),
        "template" => run_template(args),
        "html" => run_html(args),
        "manifest" => run_manifest(args),
        "build" => run_build(args),
        _ => {
            eprintln!("unknown command: {command}");
            print_usage_and_exit();
        }
    }
}

fn run_explain(mut args: Vec<String>) {
    if args
        .iter()
        .any(|argument| is_asm_entity_inspection_option(argument))
    {
        run_asm_inspection(parse_asm_inputs(&args));
        return;
    }

    if args.is_empty() {
        eprintln!("missing file path");
        print_usage_and_exit();
    }

    let path = PathBuf::from(args.remove(0));

    let format = parse_format(&args);

    let source = fs::read_to_string(&path).unwrap_or_else(|error| {
        eprintln!("failed to read {}: {error}", path.display());
        process::exit(1);
    });

    let summary = summarize_source(&path, &source);

    match format.as_str() {
        "text" => print!("{}", explain_text(&summary)),
        "json" => print!("{}", explain_json(&summary)),
        _ => {
            eprintln!("unsupported format: {format}");
            process::exit(1);
        }
    }
}

fn run_graph(mut args: Vec<String>) {
    if args.is_empty() {
        eprintln!("missing file path");
        print_usage_and_exit();
    }

    let path = PathBuf::from(args.remove(0));

    let source = fs::read_to_string(&path).unwrap_or_else(|error| {
        eprintln!("failed to read {}: {error}", path.display());
        process::exit(1);
    });

    let parsed = parse_file(&path, &source);
    let graph = fold_component_graph(&build_component_graph(&parsed));

    print_component_graph(&path, &graph);
}

fn run_asm(args: &[String]) {
    run_asm_inspection(parse_asm_inputs(args));
}

fn run_asm_inspection(inputs: AsmInputs) {
    let sources = inputs
        .paths
        .into_iter()
        .map(|path| {
            let source = fs::read_to_string(&path).unwrap_or_else(|error| {
                eprintln!("failed to read {}: {error}", path.display());
                process::exit(1);
            });
            (path, source)
        })
        .collect::<Vec<_>>();
    let unit = CompilationUnit::parse_sources(sources);
    let paths = unit
        .files()
        .iter()
        .map(|file| file.path.clone())
        .collect::<Vec<_>>();
    let asm = build_application_semantic_model_for_unit(&unit);
    let asm = ConstantFoldingPass.transform(&asm);
    let validation = validate_application_semantic_model(&asm);

    if inputs.format == "graph" {
        if inputs.entity_id.is_some()
            || inputs.source_selection.is_some()
            || !inputs.filters.is_empty()
        {
            eprintln!("--format graph cannot be combined with ASM entity selection or filters");
            process::exit(1);
        }
        print!("{}", semantic_graph_json(&build_semantic_graph(&asm)));
        return;
    }

    let entity = match (inputs.entity_id, inputs.source_selection) {
        (Some(entity_id), None) => Some(find_asm_entity(&asm, &entity_id)),
        (None, Some((path, offset))) => Some(find_asm_entity_at(&asm, &path, offset)),
        (None, None) => None,
        (Some(_), Some(_)) => unreachable!("ASM input validation rejects conflicting selectors"),
    };
    if !inputs.filters.is_empty() && entity.is_none() {
        eprintln!("--child-kind and --reference-kind require an ASM entity selector");
        process::exit(1);
    }

    match (inputs.format.as_str(), entity) {
        ("text", Some(entity)) => {
            print_asm_entity_text(&asm, entity, &asm.diagnostics, inputs.filters);
        }
        ("json", Some(entity)) => print!(
            "{}",
            asm_entity_inspection_json(&asm, entity, &asm.diagnostics, inputs.filters)
        ),
        ("text", None) => print_asm_text(&paths, &asm, &validation),
        ("json", None) => print!("{}", asm_inspection_json(&paths, &asm, &validation)),
        _ => {
            eprintln!("unsupported format: {}", inputs.format);
            process::exit(1);
        }
    }
}

fn is_asm_entity_inspection_option(argument: &str) -> bool {
    matches!(
        argument,
        "--entity" | "--source" | "--offset" | "--child-kind" | "--reference-kind"
    )
}

fn run_check(args: &[String]) {
    let (input_paths, format, categories, fail_on) = parse_check_inputs(args);
    let sources = input_paths
        .into_iter()
        .map(|path| {
            let source = fs::read_to_string(&path).unwrap_or_else(|error| {
                eprintln!("failed to read {}: {error}", path.display());
                process::exit(1);
            });
            (path, source)
        })
        .collect::<Vec<_>>();
    let unit = CompilationUnit::parse_sources(sources);
    let asm = build_application_semantic_model_for_unit(&unit);
    let asm = ConstantFoldingPass.transform(&asm);
    let validation = validate_application_semantic_model(&asm);
    let parser_diagnostic_count = unit
        .files()
        .iter()
        .map(|file| file.diagnostics.len())
        .sum::<usize>();

    match format.as_str() {
        "text" => print_check_text(
            &unit,
            &asm,
            &validation,
            parser_diagnostic_count,
            &categories,
            &fail_on,
        ),
        "json" => print!(
            "{}",
            check_json(&unit, &asm, &validation, &categories, &fail_on)
        ),
        _ => {
            eprintln!("unsupported format: {format}");
            process::exit(1);
        }
    }

    if parser_diagnostics_fail(&unit, &fail_on)
        || !asm.diagnostics.is_empty()
        || !validation.is_empty()
    {
        process::exit(1);
    }
}

fn print_check_text(
    unit: &CompilationUnit,
    asm: &ApplicationSemanticModel,
    validation: &[AsmValidationDiagnostic],
    parser_diagnostic_count: usize,
    categories: &[String],
    fail_on: &ParseSeverity,
) {
    println!("Check:");
    println!("  files: {}", unit.files().len());
    println!("  parser diagnostics: {parser_diagnostic_count}");
    println!("  compiler diagnostics: {}", asm.diagnostics.len());
    println!("  ASM validation diagnostics: {}", validation.len());
    println!("  parser fail on: {}", diagnostic_severity_label(fail_on));
    if check_category_enabled(categories, "parser") {
        for file in unit.files() {
            for diagnostic in &file.diagnostics {
                println!(
                    "  parser {}: {}",
                    diagnostic_severity_label(&diagnostic.severity),
                    diagnostic.message
                );
                for label in &diagnostic.labels {
                    println!(
                        "    at {}:{}:{} span={}..{}",
                        file.path.display(),
                        label.span.line,
                        label.span.column,
                        label.span.start,
                        label.span.end
                    );
                }
            }
        }
    }
    if check_category_enabled(categories, "compiler") {
        print_compiler_diagnostics(&asm.diagnostics);
    }
    if check_category_enabled(categories, "validation") {
        if let Some(validation_text) = asm_validation_diagnostics_text(validation) {
            print!("{validation_text}");
        }
    }
}

fn check_json(
    unit: &CompilationUnit,
    asm: &ApplicationSemanticModel,
    validation: &[AsmValidationDiagnostic],
    categories: &[String],
    fail_on: &ParseSeverity,
) -> String {
    let parser_count = unit
        .files()
        .iter()
        .map(|file| file.diagnostics.len())
        .sum::<usize>();
    let parser_diagnostics = if check_category_enabled(categories, "parser") {
        unit.files()
            .iter()
            .flat_map(|file| {
                file.diagnostics
                    .iter()
                    .map(move |diagnostic| parser_diagnostic_json(&file.path, diagnostic))
            })
            .collect::<Vec<_>>()
    } else {
        Vec::new()
    };
    let compiler_diagnostics = if check_category_enabled(categories, "compiler") {
        asm.diagnostics
            .iter()
            .map(compiler_diagnostic_json)
            .collect::<Vec<_>>()
    } else {
        Vec::new()
    };
    let validation_diagnostics = if check_category_enabled(categories, "validation") {
        validation.iter().map(|diagnostic| serde_json::json!({"code": diagnostic.code, "message": diagnostic.message})).collect::<Vec<_>>()
    } else {
        Vec::new()
    };
    serde_json::to_string_pretty(&serde_json::json!({"schema_version": CHECK_JSON_SCHEMA_VERSION, "files": unit.files().iter().map(|file| file.path.display().to_string()).collect::<Vec<_>>(), "summary": {"parser_diagnostics": parser_count, "compiler_diagnostics": asm.diagnostics.len(), "validation": validation.len()}, "categories": categories, "fail_on": diagnostic_severity_label(fail_on), "parser_diagnostics": parser_diagnostics, "compiler_diagnostics": compiler_diagnostics, "validation": validation_diagnostics})).expect("check document should serialize") + "\n"
}

fn parser_diagnostic_json(path: &Path, diagnostic: &ParseDiagnostic) -> serde_json::Value {
    serde_json::json!({
        "path": path,
        "severity": diagnostic_severity_label(&diagnostic.severity),
        "message": diagnostic.message,
        "labels": diagnostic.labels.iter().map(|label| serde_json::json!({
            "line": label.span.line,
            "column": label.span.column,
            "start": label.span.start,
            "end": label.span.end,
        })).collect::<Vec<_>>(),
    })
}

fn compiler_diagnostic_json(diagnostic: &ezc_core::ComponentDiagnostic) -> serde_json::Value {
    serde_json::to_value(AsmInspectionDiagnostic::from(diagnostic))
        .expect("compiler diagnostic projection should serialize")
}

fn parser_diagnostics_fail(unit: &CompilationUnit, fail_on: &ParseSeverity) -> bool {
    unit.files()
        .iter()
        .flat_map(|file| &file.diagnostics)
        .any(|diagnostic| severity_rank(&diagnostic.severity) >= severity_rank(fail_on))
}
fn severity_rank(severity: &ParseSeverity) -> u8 {
    match severity {
        ParseSeverity::Info => 0,
        ParseSeverity::Warning => 1,
        ParseSeverity::Error => 2,
    }
}

fn check_category_enabled(categories: &[String], category: &str) -> bool {
    categories.is_empty() || categories.iter().any(|item| item == category)
}

fn print_asm_text(
    paths: &[PathBuf],
    asm: &ApplicationSemanticModel,
    validation: &[AsmValidationDiagnostic],
) {
    let projected_entities = asm
        .ownership
        .keys()
        .filter(|id| is_phase_g_inspection_entity(asm, id))
        .collect::<Vec<_>>();
    if paths.len() == 1 {
        println!("File: {}", paths[0].display());
    } else {
        println!("Files:");
        for path in paths {
            println!("  {}", path.display());
        }
    }
    println!("ApplicationSemanticModel:");
    println!("  components: {}", asm.components.len());
    println!("  templates: {}", asm.templates.len());
    println!("  ownership: {}", projected_entities.len());
    println!(
        "  references: {}",
        asm.references
            .iter()
            .filter(|reference| {
                !matches!(
                    reference.kind,
                    SemanticReferenceKind::FieldBindingField
                        | SemanticReferenceKind::FieldBindingForm
                        | SemanticReferenceKind::ValidationRuleField
                )
            })
            .count()
    );
    println!(
        "  provenance: {}",
        projected_entities
            .iter()
            .filter(|id| asm.provenance(id).is_some())
            .count()
    );
    println!(
        "  semantic types: {}",
        projected_entities
            .iter()
            .filter(|id| asm.semantic_types.assignments.contains_key(*id))
            .count()
    );
    println!("  diagnostics: {}", asm.diagnostics.len());
    println!("  validation: {}", validation.len());

    print_compiler_diagnostics(&asm.diagnostics);

    if let Some(validation_text) = asm_validation_diagnostics_text(validation) {
        print!("{validation_text}");
    }
}

fn print_compiler_diagnostics(diagnostics: &[ezc_core::ComponentDiagnostic]) {
    let mut diagnostics = diagnostics.iter().collect::<Vec<_>>();
    diagnostics.sort_by(|left, right| {
        (left.code.as_str(), left.message.as_str())
            .cmp(&(right.code.as_str(), right.message.as_str()))
    });
    if !diagnostics.is_empty() {
        println!("  compiler diagnostics:");
        for diagnostic in diagnostics {
            println!(
                "    {}[{}]: {}",
                diagnostic.severity.as_str(),
                diagnostic.code,
                diagnostic.message
            );
            if let Some(provenance) = &diagnostic.provenance {
                println!(
                    "      at {}:{}:{} span={}..{}",
                    provenance.path.display(),
                    provenance.span.line,
                    provenance.span.column,
                    provenance.span.start,
                    provenance.span.end,
                );
            }
            if let Some(effect_id) = &diagnostic.effect_id {
                println!("      = effect: {effect_id}");
            }
            if let Some(statement_id) = &diagnostic.statement_id {
                println!("      = statement: {statement_id}");
            }
            for (role, identity) in diagnostic_component_identities(diagnostic) {
                println!("      = {role}: {identity}");
            }
            for label in &diagnostic.secondary_labels {
                println!(
                    "      = related: {} at {}:{}:{} span={}..{}",
                    label.message,
                    label.provenance.path.display(),
                    label.provenance.span.line,
                    label.provenance.span.column,
                    label.provenance.span.start,
                    label.provenance.span.end,
                );
            }
        }
    }
}

fn diagnostic_component_identities(
    diagnostic: &ezc_core::ComponentDiagnostic,
) -> Vec<(&'static str, String)> {
    let mut identities = Vec::new();
    macro_rules! push_identity {
        ($role:literal, $value:expr) => {
            if let Some(value) = $value {
                identities.push(($role, value.to_string()));
            }
        };
    }
    push_identity!("slot", diagnostic.slot_id.as_ref());
    push_identity!("invocation", diagnostic.invocation_id.as_ref());
    push_identity!(
        "component instance",
        diagnostic.component_instance_id.as_ref()
    );
    push_identity!("slot binding", diagnostic.slot_binding_id.as_ref());
    push_identity!(
        "structural region",
        diagnostic.structural_region_id.as_ref()
    );
    push_identity!("component", diagnostic.component_id.as_ref());
    push_identity!(
        "provider instance",
        diagnostic.provider_instance_id.as_ref()
    );
    push_identity!(
        "consumer instance",
        diagnostic.consumer_instance_id.as_ref()
    );
    identities
}

fn asm_validation_diagnostics_text(validation: &[AsmValidationDiagnostic]) -> Option<String> {
    if validation.is_empty() {
        return None;
    }

    let mut diagnostics = validation.iter().collect::<Vec<_>>();
    diagnostics.sort_by(|left, right| {
        (left.code.as_str(), left.message.as_str())
            .cmp(&(right.code.as_str(), right.message.as_str()))
    });

    let mut output = String::from("  ASM validation diagnostics:\n");
    for diagnostic in diagnostics {
        writeln!(output, "    {}: {}", diagnostic.code, diagnostic.message)
            .expect("writing to String should not fail");
    }

    Some(output)
}

fn asm_inspection_json(
    paths: &[PathBuf],
    asm: &ApplicationSemanticModel,
    validation: &[AsmValidationDiagnostic],
) -> String {
    let computed_functions = computed_evaluation_functions(asm);
    let effect_inspections = build_effect_inspection_registry(asm);
    let context_inspections = build_context_inspection_registry(asm);
    let mut references = asm
        .references
        .iter()
        .filter(|reference| {
            !matches!(
                reference.kind,
                SemanticReferenceKind::FieldBindingField
                    | SemanticReferenceKind::FieldBindingForm
                    | SemanticReferenceKind::ValidationRuleField
            )
        })
        .map(|reference| AsmInspectionReference {
            kind: semantic_reference_kind(reference.kind),
            source: reference.source.as_str(),
            target: reference.target.as_str(),
            provenance: (&reference.provenance).into(),
        })
        .collect::<Vec<_>>();
    references.sort_by(|left, right| {
        (left.source, left.target, left.kind).cmp(&(right.source, right.target, right.kind))
    });

    let diagnostics = asm
        .diagnostics
        .iter()
        .map(AsmInspectionDiagnostic::from)
        .collect::<Vec<_>>();

    let mut validation = validation
        .iter()
        .map(|diagnostic| AsmInspectionDiagnostic {
            code: &diagnostic.code,
            severity: None,
            message: &diagnostic.message,
            primary_provenance: None,
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
            component_id: None,
            provider_instance_id: None,
            consumer_instance_id: None,
            secondary_labels: Vec::new(),
        })
        .collect::<Vec<_>>();
    validation.sort_by(|left, right| (left.code, left.message).cmp(&(right.code, right.message)));

    let document = AsmInspectionDocument {
        schema_version: ASM_INSPECTION_SCHEMA_VERSION,
        file: paths[0].display().to_string(),
        files: (paths.len() > 1).then(|| {
            paths
                .iter()
                .map(|path| path.display().to_string())
                .collect()
        }),
        entities: asm
            .ownership
            .keys()
            .filter(|id| is_phase_g_inspection_entity(asm, id))
            .map(|id| {
                asm_inspection_entity(
                    asm,
                    id,
                    &computed_functions,
                    &effect_inspections,
                    &context_inspections,
                )
            })
            .collect(),
        references,
        diagnostics,
        validation,
    };

    serde_json::to_string_pretty(&document).expect("ASM inspection document should serialize")
        + "\n"
}

fn find_asm_entity<'a>(asm: &'a ApplicationSemanticModel, entity_id: &str) -> &'a SemanticId {
    asm.ownership
        .keys()
        .filter(|id| is_phase_g_inspection_entity(asm, id))
        .find(|id| id.as_str() == entity_id)
        .unwrap_or_else(|| {
            eprintln!("unknown ASM entity: {entity_id}");
            process::exit(1);
        })
}

fn find_asm_entity_at<'a>(
    asm: &'a ApplicationSemanticModel,
    path: &Path,
    offset: usize,
) -> &'a SemanticId {
    let mut candidates = asm.entities_at(path, offset);
    candidates.retain(|id| is_phase_g_inspection_entity(asm, id));
    if candidates.is_empty() {
        eprintln!("no ASM entity at {}:{offset}", path.display());
        process::exit(1);
    }
    candidates.sort_by(|left, right| {
        let left_span = &asm
            .provenance(left)
            .expect("ASM entities have provenance")
            .span;
        let right_span = &asm
            .provenance(right)
            .expect("ASM entities have provenance")
            .span;
        (left_span.end - left_span.start, left.as_str())
            .cmp(&(right_span.end - right_span.start, right.as_str()))
    });

    let entity = candidates[0];
    let entity_span = &asm
        .provenance(entity)
        .expect("ASM entities have provenance")
        .span;
    if candidates.get(1).is_some_and(|other| {
        let other_span = &asm
            .provenance(other)
            .expect("ASM entities have provenance")
            .span;
        other_span.end - other_span.start == entity_span.end - entity_span.start
    }) {
        eprintln!("ambiguous ASM entity at {}:{offset}", path.display());
        for candidate in candidates {
            eprintln!("  {}", candidate.as_str());
        }
        process::exit(1);
    }

    entity
}

fn print_asm_entity_text(
    asm: &ApplicationSemanticModel,
    id: &SemanticId,
    diagnostics: &[ezc_core::ComponentDiagnostic],
    filters: AsmEntityFilters,
) {
    let computed_functions = computed_evaluation_functions(asm);
    let effect_inspections = build_effect_inspection_registry(asm);
    let entity = asm
        .entity(id)
        .expect("ASM ownership should contain entities");
    let provenance = asm.provenance(id).expect("ASM entities have provenance");
    println!("ASM Entity: {}", id.as_str());
    println!("  kind: {}", semantic_entity_kind(entity));
    println!(
        "  owner: {}",
        semantic_owner_id(asm.owner(id).expect("ASM entities have owners"))
            .unwrap_or("application")
    );
    println!(
        "  provenance: {}:{}:{} span={}..{}",
        provenance.path.display(),
        provenance.span.line,
        provenance.span.column,
        provenance.span.start,
        provenance.span.end
    );
    if let Some(semantic_type) = asm_semantic_type(asm, id) {
        println!("  semantic type: {}", semantic_type.type_text);
        println!("    status: {}", semantic_type.status);
        println!("    origin: {}", semantic_type.origin);
    }
    if let Some(computed) = asm_computed_inspection(asm, id, &computed_functions) {
        println!("  computed:");
        println!("    type: {}", computed.computed_type);
        println!("    dependencies: {:?}", computed.dependencies);
        println!("    dependents: {:?}", computed.dependents);
        println!("    evaluation order: {:?}", computed.evaluation_order);
        println!("    evaluation batch: {:?}", computed.evaluation_batch);
        println!("    purity: {}", computed.purity);
        println!("    serializability: {}", computed.serializability);
        println!("    IR function: {:?}", computed.ir_function);
    }
    if let Some(effect) = effect_inspections.records.get(id) {
        print_effect_inspection_text(effect);
    }
    let parents = asm.ancestors_of(id);
    println!("  parents: {}", parents.len());
    for parent in parents {
        println!("    {}", parent.as_str());
    }
    let children = filtered_entity_children(asm, id, filters);
    println!("  children: {}", children.len());
    for child in children {
        println!("    {}", child.as_str());
    }
    println!("  descendants: {}", projected_descendants(asm, id).len());
    print_entity_references(
        "outgoing",
        filtered_entity_references(asm.references_from(id), filters),
    );
    print_entity_references(
        "incoming",
        filtered_entity_references(asm.references_to(id), filters),
    );
    print_entity_diagnostics(diagnostics, id, provenance);
}

fn print_entity_references(label: &str, references: Vec<&ezc_core::SemanticReference>) {
    println!("  {label} references: {}", references.len());
    for reference in references {
        println!(
            "    {}: {} -> {}",
            semantic_reference_kind(reference.kind),
            reference.source.as_str(),
            reference.target.as_str()
        );
    }
}

fn print_effect_inspection_text(effect: &EffectInspection) {
    println!("  Effect:");
    println!("    Validation: {}", effect.validation.status);
    println!("    Violations: {:?}", effect.validation.violations);
    println!("    Dependencies:");
    println!("      state: {:?}", effect.direct_dependencies.state);
    println!("      computed: {:?}", effect.direct_dependencies.computed);
    println!(
        "      transitive state: {:?}",
        effect.transitive_dependencies.state
    );
    println!(
        "      transitive computed: {:?}",
        effect.transitive_dependencies.computed
    );
    println!("      dependents: {:?}", effect.dependents);
    println!("    Initial trigger: {:?}", effect.initial_trigger);
    println!("    Action triggers: {:?}", effect.action_triggers);
    println!("    Schedule: {:?}", effect.schedule);
    println!("    Capabilities: {:?}", effect.capabilities);
    println!("    IR: {:?}", effect.ir);
    println!("    Runtime: {:?}", effect.runtime);
    println!("    Resumability: {:?}", effect.resumability);
}

fn print_entity_diagnostics(
    diagnostics: &[ezc_core::ComponentDiagnostic],
    id: &SemanticId,
    provenance: &SourceProvenance,
) {
    let diagnostics = related_entity_diagnostics(diagnostics, id, provenance);
    println!("  diagnostics: {}", diagnostics.len());
    for diagnostic in diagnostics {
        println!("    {}: {}", diagnostic.code, diagnostic.message);
    }
}

fn asm_entity_inspection_json(
    asm: &ApplicationSemanticModel,
    id: &SemanticId,
    diagnostics: &[ezc_core::ComponentDiagnostic],
    filters: AsmEntityFilters,
) -> String {
    let computed_functions = computed_evaluation_functions(asm);
    let effect_inspections = build_effect_inspection_registry(asm);
    let context_inspections = build_context_inspection_registry(asm);
    let provenance = asm.provenance(id).expect("ASM entities have provenance");
    let document = AsmEntityInspectionDocument {
        schema_version: ASM_INSPECTION_SCHEMA_VERSION,
        entity: asm_inspection_entity(
            asm,
            id,
            &computed_functions,
            &effect_inspections,
            &context_inspections,
        ),
        parents: asm
            .ancestors_of(id)
            .into_iter()
            .map(SemanticId::as_str)
            .collect(),
        children: filtered_entity_children(asm, id, filters)
            .into_iter()
            .map(SemanticId::as_str)
            .collect(),
        descendant_count: projected_descendants(asm, id).len(),
        outgoing_references: filtered_entity_references(asm.references_from(id), filters)
            .into_iter()
            .map(AsmInspectionReference::from)
            .collect(),
        incoming_references: filtered_entity_references(asm.references_to(id), filters)
            .into_iter()
            .map(AsmInspectionReference::from)
            .collect(),
        diagnostics: related_entity_diagnostics(diagnostics, id, provenance)
            .into_iter()
            .map(AsmInspectionDiagnostic::from)
            .collect(),
    };

    serde_json::to_string_pretty(&document).expect("ASM entity inspection should serialize") + "\n"
}

fn filtered_entity_children<'a>(
    asm: &'a ApplicationSemanticModel,
    id: &SemanticId,
    filters: AsmEntityFilters,
) -> Vec<&'a SemanticId> {
    asm.children_of(id)
        .into_iter()
        .filter(|child| is_phase_g_inspection_entity(asm, child))
        .filter(|child| {
            filters.child_kind.is_none_or(|kind| {
                asm.entity(child)
                    .is_some_and(|entity| entity.kind() == kind)
            })
        })
        .collect()
}

fn projected_descendants<'a>(
    asm: &'a ApplicationSemanticModel,
    id: &SemanticId,
) -> Vec<&'a SemanticId> {
    asm.descendants_of(id)
        .into_iter()
        .filter(|descendant| is_phase_g_inspection_entity(asm, descendant))
        .collect()
}

fn is_phase_g_inspection_entity(asm: &ApplicationSemanticModel, id: &SemanticId) -> bool {
    !matches!(
        asm.entity(id),
        Some(
            SemanticEntity::Form(_)
                | SemanticEntity::FormField(_)
                | SemanticEntity::FormFieldBinding(_)
                | SemanticEntity::ValidationRule(_)
                | SemanticEntity::Slot(_)
                | SemanticEntity::ComponentInvocation(_)
                | SemanticEntity::ComponentInstance(_)
                | SemanticEntity::BlockedComponentInstance(_)
                | SemanticEntity::SlotContentFragment(_)
                | SemanticEntity::SlotOutlet(_)
        )
    )
}

fn filtered_entity_references(
    references: Vec<&ezc_core::SemanticReference>,
    filters: AsmEntityFilters,
) -> Vec<&ezc_core::SemanticReference> {
    let mut references = references
        .into_iter()
        .filter(|reference| {
            !matches!(
                reference.kind,
                SemanticReferenceKind::FieldBindingField
                    | SemanticReferenceKind::FieldBindingForm
                    | SemanticReferenceKind::ValidationRuleField
            )
        })
        .filter(|reference| {
            filters
                .reference_kind
                .is_none_or(|kind| reference.kind == kind)
        })
        .collect::<Vec<_>>();
    references.sort_by(|left, right| {
        (left.source.as_str(), left.target.as_str())
            .cmp(&(right.source.as_str(), right.target.as_str()))
    });
    references
}

fn related_entity_diagnostics<'a>(
    diagnostics: &'a [ezc_core::ComponentDiagnostic],
    id: &SemanticId,
    provenance: &SourceProvenance,
) -> Vec<&'a ezc_core::ComponentDiagnostic> {
    let id = id.as_str();
    diagnostics
        .iter()
        .filter(|diagnostic| {
            diagnostic
                .effect_id
                .as_ref()
                .is_some_and(|item| item.as_str() == id)
                || diagnostic
                    .context_declaration_candidate_id
                    .as_ref()
                    .is_some_and(|item| item.as_str() == id)
                || diagnostic
                    .context_id
                    .as_ref()
                    .is_some_and(|item| item.as_str() == id)
                || diagnostic
                    .provider_id
                    .as_ref()
                    .is_some_and(|item| item.as_str() == id)
                || diagnostic
                    .consumer_id
                    .as_ref()
                    .is_some_and(|item| item.as_str() == id)
                || diagnostic
                    .slot_id
                    .as_ref()
                    .is_some_and(|item| item.as_str() == id)
                || diagnostic
                    .invocation_id
                    .as_ref()
                    .is_some_and(|item| item.as_str() == id)
                || diagnostic
                    .component_instance_id
                    .as_ref()
                    .is_some_and(|item| item.as_str() == id)
                || diagnostic
                    .slot_binding_id
                    .as_ref()
                    .is_some_and(|item| item.as_str() == id)
                || diagnostic
                    .structural_region_id
                    .as_ref()
                    .is_some_and(|item| item.as_str() == id)
                || diagnostic
                    .component_id
                    .as_ref()
                    .is_some_and(|item| item.as_str() == id)
                || diagnostic
                    .provider_instance_id
                    .as_ref()
                    .is_some_and(|item| item.to_string() == id)
                || diagnostic
                    .consumer_instance_id
                    .as_ref()
                    .is_some_and(|item| item.to_string() == id)
                || diagnostic
                    .provenance
                    .as_ref()
                    .is_some_and(|diagnostic_provenance| {
                        diagnostic_provenance.path == provenance.path
                            && diagnostic_provenance.span.start < provenance.span.end
                            && provenance.span.start < diagnostic_provenance.span.end
                    })
        })
        .collect()
}

struct AsmInputs {
    paths: Vec<PathBuf>,
    format: String,
    entity_id: Option<String>,
    source_selection: Option<(PathBuf, usize)>,
    filters: AsmEntityFilters,
}

#[derive(Clone, Copy, Default)]
struct AsmEntityFilters {
    child_kind: Option<SemanticEntityKind>,
    reference_kind: Option<SemanticReferenceKind>,
}

impl AsmEntityFilters {
    fn is_empty(self) -> bool {
        self.child_kind.is_none() && self.reference_kind.is_none()
    }
}

fn parse_asm_inputs(args: &[String]) -> AsmInputs {
    let mut paths = Vec::new();
    let mut format = "text".to_string();
    let mut entity_id = None;
    let mut source_path = None;
    let mut source_offset = None;
    let mut filters = AsmEntityFilters::default();
    let mut index = 0;

    while index < args.len() {
        match args[index].as_str() {
            "--format" => {
                let Some(value) = args.get(index + 1) else {
                    eprintln!("missing value for --format");
                    process::exit(1);
                };
                format.clone_from(value);
                index += 2;
            }
            "--entity" => {
                let Some(value) = args.get(index + 1) else {
                    eprintln!("missing value for --entity");
                    process::exit(1);
                };
                entity_id = Some(value.clone());
                index += 2;
            }
            "--source" => {
                let Some(value) = args.get(index + 1) else {
                    eprintln!("missing value for --source");
                    process::exit(1);
                };
                source_path = Some(PathBuf::from(value));
                index += 2;
            }
            "--offset" => {
                let Some(value) = args.get(index + 1) else {
                    eprintln!("missing value for --offset");
                    process::exit(1);
                };
                source_offset = Some(value.parse::<usize>().unwrap_or_else(|_| {
                    eprintln!("invalid byte offset: {value}");
                    process::exit(1);
                }));
                index += 2;
            }
            "--child-kind" => {
                let Some(value) = args.get(index + 1) else {
                    eprintln!("missing value for --child-kind");
                    process::exit(1);
                };
                filters.child_kind = Some(parse_asm_entity_kind(value));
                index += 2;
            }
            "--reference-kind" => {
                let Some(value) = args.get(index + 1) else {
                    eprintln!("missing value for --reference-kind");
                    process::exit(1);
                };
                filters.reference_kind = Some(parse_asm_reference_kind(value));
                index += 2;
            }
            option if option.starts_with('-') => {
                eprintln!("unknown option: {option}");
                process::exit(1);
            }
            path => {
                paths.push(PathBuf::from(path));
                index += 1;
            }
        }
    }

    if paths.is_empty() {
        eprintln!("missing file path");
        print_usage_and_exit();
    }

    let source_selection = match (source_path, source_offset) {
        (Some(path), Some(offset)) => Some((path, offset)),
        (None, None) => None,
        _ => {
            eprintln!("--source and --offset must be used together");
            process::exit(1);
        }
    };
    if entity_id.is_some() && source_selection.is_some() {
        eprintln!("--entity cannot be combined with --source or --offset");
        process::exit(1);
    }

    AsmInputs {
        paths,
        format,
        entity_id,
        source_selection,
        filters,
    }
}

fn parse_asm_entity_kind(value: &str) -> SemanticEntityKind {
    match value {
        "component" => SemanticEntityKind::Component,
        "state-field" => SemanticEntityKind::StateField,
        "method" => SemanticEntityKind::Method,
        "context" => SemanticEntityKind::Context,
        "provider" => SemanticEntityKind::Provider,
        "consumer" => SemanticEntityKind::Consumer,
        "computed" => SemanticEntityKind::Computed,
        "effect" => SemanticEntityKind::Effect,
        "parameter" => SemanticEntityKind::Parameter,
        "local-variable" => SemanticEntityKind::LocalVariable,
        "action" => SemanticEntityKind::Action,
        "event-handler" => SemanticEntityKind::EventHandler,
        "template" => SemanticEntityKind::Template,
        "template-entity" => SemanticEntityKind::TemplateEntity,
        _ => {
            eprintln!("unsupported ASM child kind: {value}");
            process::exit(1);
        }
    }
}

fn parse_asm_reference_kind(value: &str) -> SemanticReferenceKind {
    match value {
        "action-state" => SemanticReferenceKind::ActionState,
        "computed-state" => SemanticReferenceKind::ComputedState,
        "computed-computed" => SemanticReferenceKind::ComputedComputed,
        "effect-state" => SemanticReferenceKind::EffectState,
        "effect-computed" => SemanticReferenceKind::EffectComputed,
        "provides-context" => SemanticReferenceKind::ProvidesContext,
        "consumes-context" => SemanticReferenceKind::ConsumesContext,
        "resolves-to-provider" => SemanticReferenceKind::ResolvesToProvider,
        "event-method" => SemanticReferenceKind::EventMethod,
        "template-state" => SemanticReferenceKind::TemplateState,
        "template-computed" => SemanticReferenceKind::TemplateComputed,
        "template-local" => SemanticReferenceKind::TemplateLocal,
        _ => {
            eprintln!("unsupported ASM reference kind: {value}");
            process::exit(1);
        }
    }
}

fn parse_check_inputs(args: &[String]) -> (Vec<PathBuf>, String, Vec<String>, ParseSeverity) {
    if args.is_empty() {
        eprintln!("missing file path");
        print_usage_and_exit();
    }

    let mut paths = Vec::new();
    let mut format = "text".to_string();
    let mut categories = Vec::new();
    let mut fail_on = ParseSeverity::Error;
    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--format" => {
                let Some(value) = args.get(index + 1) else {
                    eprintln!("missing value for --format");
                    process::exit(1);
                };
                format.clone_from(value);
                index += 2;
            }
            "--category" => {
                let Some(value) = args.get(index + 1) else {
                    eprintln!("missing value for --category");
                    process::exit(1);
                };
                if !matches!(value.as_str(), "parser" | "compiler" | "validation") {
                    eprintln!("unsupported check category: {value}");
                    process::exit(1);
                }
                categories.push(value.clone());
                index += 2;
            }
            "--fail-on" => {
                let Some(value) = args.get(index + 1) else {
                    eprintln!("missing value for --fail-on");
                    process::exit(1);
                };
                fail_on = match value.as_str() {
                    "error" => ParseSeverity::Error,
                    "warning" => ParseSeverity::Warning,
                    "info" => ParseSeverity::Info,
                    _ => {
                        eprintln!("unsupported fail policy: {value}");
                        process::exit(1);
                    }
                };
                index += 2;
            }
            option if option.starts_with('-') => {
                eprintln!("unknown option: {option}");
                process::exit(1);
            }
            path => {
                paths.push(PathBuf::from(path));
                index += 1;
            }
        }
    }
    (paths, format, categories, fail_on)
}

fn semantic_entity_kind(entity: SemanticEntity<'_>) -> &'static str {
    match entity {
        SemanticEntity::Component(_) => "component",
        SemanticEntity::StateField(_) => "state-field",
        SemanticEntity::Method(_) => "method",
        SemanticEntity::Context(_) => "context",
        SemanticEntity::Provider(_) => "provider",
        SemanticEntity::Consumer(_) => "consumer",
        SemanticEntity::Form(_) => "form",
        SemanticEntity::FormField(_) => "form-field",
        SemanticEntity::FormFieldBinding(_) => "form-field-binding",
        SemanticEntity::ValidationRule(_) => "validation-rule",
        SemanticEntity::Slot(_) => "slot",
        SemanticEntity::ComponentInvocation(_) => "component-invocation",
        SemanticEntity::ComponentInstance(_) => "component-instance",
        SemanticEntity::BlockedComponentInstance(_) => "blocked-component-instance",
        SemanticEntity::SlotContentFragment(_) => "slot-content-fragment",
        SemanticEntity::SlotOutlet(_) => "slot-outlet",
        SemanticEntity::Computed(_) => "computed",
        SemanticEntity::Effect(_) => "effect",
        SemanticEntity::Parameter(_) => "parameter",
        SemanticEntity::LocalVariable(_) => "local-variable",
        SemanticEntity::Action(_) => "action",
        SemanticEntity::EventHandler(_) => "event-handler",
        SemanticEntity::Template(_) => "template",
        SemanticEntity::TemplateEntity(entity) => match entity.kind {
            TemplateSemanticKind::Element => "template-element",
            TemplateSemanticKind::Fragment => "template-fragment",
            TemplateSemanticKind::Text => "template-text",
            TemplateSemanticKind::Binding => "template-binding",
            TemplateSemanticKind::Attribute => "template-attribute",
            TemplateSemanticKind::AttributeBinding => "template-attribute-binding",
            TemplateSemanticKind::EventAttribute => "template-event-attribute",
            TemplateSemanticKind::Conditional => "template-conditional",
            TemplateSemanticKind::List => "template-list",
        },
    }
}

fn semantic_owner_id(owner: &SemanticOwner) -> Option<&str> {
    owner.entity_id().map(ezc_core::SemanticId::as_str)
}

fn semantic_reference_kind(kind: SemanticReferenceKind) -> &'static str {
    match kind {
        SemanticReferenceKind::ActionState => "action-state",
        SemanticReferenceKind::ComputedState => "computed-state",
        SemanticReferenceKind::ComputedComputed => "computed-computed",
        SemanticReferenceKind::EffectState => "effect-state",
        SemanticReferenceKind::EffectComputed => "effect-computed",
        SemanticReferenceKind::ProvidesContext => "provides-context",
        SemanticReferenceKind::ConsumesContext => "consumes-context",
        SemanticReferenceKind::ResolvesToProvider => "resolves-to-provider",
        SemanticReferenceKind::EventMethod => "event-method",
        SemanticReferenceKind::TemplateState => "template-state",
        SemanticReferenceKind::TemplateComputed => "template-computed",
        SemanticReferenceKind::TemplateLocal => "template-local",
        SemanticReferenceKind::FieldBindingField => "field-binding-field",
        SemanticReferenceKind::FieldBindingForm => "field-binding-form",
        SemanticReferenceKind::ValidationRuleField => "validation-rule-field",
    }
}

fn declared_state_type(entity: SemanticEntity<'_>) -> Option<AsmInspectionDeclaredType<'_>> {
    let SemanticEntity::StateField(field) = entity else {
        return None;
    };
    let declared_type = field.declared_type.as_ref()?;

    Some(AsmInspectionDeclaredType {
        text: &declared_type.text,
        kind: declared_type.kind.map(asm_declared_state_type_kind),
        provenance: (&declared_type.provenance).into(),
    })
}

fn asm_semantic_type(
    asm: &ApplicationSemanticModel,
    id: &SemanticId,
) -> Option<AsmInspectionSemanticType> {
    let assignment = asm.semantic_types.assignments.get(id)?;
    Some(AsmInspectionSemanticType {
        type_text: semantic_type_text(&assignment.semantic_type),
        origin: assignment.origin.as_str().to_string(),
        status: match assignment.status {
            ezc_core::SemanticTypeStatus::Declared => "declared",
            ezc_core::SemanticTypeStatus::Inferred => "inferred",
        },
        provenance: (&assignment.provenance).into(),
    })
}

fn computed_evaluation_functions(
    asm: &ApplicationSemanticModel,
) -> BTreeMap<SemanticId, SemanticId> {
    lower_components_to_ir(asm)
        .modules
        .into_iter()
        .flat_map(|module| module.computed_evaluations)
        .map(|evaluation| (evaluation.computed, evaluation.function))
        .collect()
}

fn asm_inspection_entity<'a>(
    asm: &'a ApplicationSemanticModel,
    id: &'a SemanticId,
    computed_functions: &BTreeMap<SemanticId, SemanticId>,
    effect_inspections: &EffectInspectionRegistry,
    context_inspections: &ezc_core::ContextInspectionRegistry,
) -> AsmInspectionEntity<'a> {
    let entity = asm
        .entity(id)
        .expect("ASM ownership should contain semantic entities");
    let provenance = asm.provenance(id).expect("ASM entities have provenance");

    AsmInspectionEntity {
        id: id.as_str(),
        kind: semantic_entity_kind(entity),
        owner: semantic_owner_id(asm.owner(id).expect("ASM entities have owners")),
        provenance: provenance.into(),
        declared_type: declared_state_type(entity),
        initial_expression: initial_expression(asm, entity),
        local_variables: method_local_variables(entity),
        parameters: method_parameters(entity, provenance),
        semantic_type: asm_semantic_type(asm, id),
        computed: asm_computed_inspection(asm, id, computed_functions),
        effect: effect_inspections.records.get(id).cloned(),
        context: context_inspections.records.get(id).cloned(),
        component: asm_component_inspection(asm, id),
    }
}

fn asm_component_inspection(
    asm: &ApplicationSemanticModel,
    id: &SemanticId,
) -> Option<AsmInspectionComponent> {
    if let Some(slot) = asm
        .slots
        .values()
        .find(|slot| slot.id.as_semantic_id() == id)
    {
        return Some(AsmInspectionComponent {
            role: "slot",
            slots: vec![slot.id.to_string()],
            invocations: Vec::new(),
            instances: Vec::new(),
            initialization_batches: Vec::new(),
            structural_regions: Vec::new(),
        });
    }
    if let Some(invocation) = asm
        .component_invocations
        .values()
        .find(|invocation| invocation.id.as_semantic_id() == id)
    {
        let instances = asm
            .component_instance_plan
            .instances
            .values()
            .filter(|instance| instance.invocation.as_ref() == Some(&invocation.id))
            .map(|instance| instance.id.to_string())
            .collect();
        return Some(AsmInspectionComponent {
            role: "invocation",
            slots: Vec::new(),
            invocations: vec![invocation.id.to_string()],
            instances,
            initialization_batches: Vec::new(),
            structural_regions: Vec::new(),
        });
    }
    let component = asm
        .components
        .iter()
        .find(|component| component.id == *id)?;
    Some(AsmInspectionComponent {
        role: "definition",
        slots: asm
            .slots
            .values()
            .filter(|slot| slot.owner == component.id)
            .map(|slot| slot.id.to_string())
            .collect(),
        invocations: asm
            .component_invocations
            .values()
            .filter(|invocation| invocation.owner_component == component.id)
            .map(|invocation| invocation.id.to_string())
            .collect(),
        instances: asm
            .component_instance_plan
            .instances
            .values()
            .filter(|instance| instance.component == component.id)
            .map(|instance| instance.id.to_string())
            .collect(),
        initialization_batches: asm
            .component_initialization
            .instance_batches
            .iter()
            .filter(|batch| {
                batch.instances.iter().any(|instance| {
                    asm.component_instance_plan
                        .instances
                        .get(instance)
                        .is_some_and(|record| record.component == component.id)
                })
            })
            .map(|batch| batch.index)
            .collect(),
        structural_regions: asm
            .component_instance_plan
            .instances
            .values()
            .filter(|instance| instance.component == component.id)
            .filter_map(|instance| instance.structural_region.as_ref())
            .map(ToString::to_string)
            .collect(),
    })
}

fn asm_computed_inspection(
    asm: &ApplicationSemanticModel,
    id: &SemanticId,
    computed_functions: &BTreeMap<SemanticId, SemanticId>,
) -> Option<AsmInspectionComputed> {
    let computed = asm.computed_value(id)?;
    let computed_type = asm
        .semantic_types
        .computed_values
        .get(id)
        .expect("computed semantic entities should have canonical type metadata");
    let evaluation_order = asm
        .computed_evaluation_plan
        .evaluation_order
        .iter()
        .position(|computed_id| computed_id == id.as_str());
    let evaluation_batch = asm
        .computed_evaluation_plan
        .update_batches
        .iter()
        .position(|batch| batch.iter().any(|computed_id| computed_id == id.as_str()));

    Some(AsmInspectionComputed {
        computed_type: semantic_type_text(&computed_type.semantic_type),
        dependencies: asm
            .reactive_transitive_analysis
            .dependencies_of(id.as_str())
            .to_vec(),
        dependents: asm
            .reactive_transitive_analysis
            .dependents_of(id.as_str())
            .to_vec(),
        evaluation_order,
        evaluation_batch,
        purity: match computed.purity {
            ezc_core::ComputedPurity::Unclassified => "unclassified",
            ezc_core::ComputedPurity::Pure => "pure",
            ezc_core::ComputedPurity::Impure => "impure",
        },
        serializability: match computed_type.serialization {
            ezc_core::SerializationCompatibility::Serializable => "serializable",
            ezc_core::SerializationCompatibility::NotSerializable => "not-serializable",
        },
        ir_function: computed_functions
            .get(id)
            .map(|function| function.as_str().to_string()),
    })
}

fn initial_expression(
    asm: &ApplicationSemanticModel,
    entity: SemanticEntity<'_>,
) -> Option<String> {
    let SemanticEntity::StateField(field) = entity else {
        return None;
    };
    asm.expression_graph.render(&field.id)
}

fn method_local_variables(entity: SemanticEntity<'_>) -> Option<Vec<String>> {
    let SemanticEntity::Method(method) = entity else {
        return None;
    };
    Some(
        method
            .local_variables
            .iter()
            .map(|local| format!("{} = {:?}", local.name, local.value))
            .collect(),
    )
}

fn method_parameters<'a>(
    entity: SemanticEntity<'a>,
    method_provenance: &SourceProvenance,
) -> Option<Vec<AsmInspectionMethodParameter<'a>>> {
    let SemanticEntity::Method(method) = entity else {
        return None;
    };

    Some(
        method
            .parameters
            .iter()
            .map(|parameter| AsmInspectionMethodParameter {
                name: &parameter.name,
                provenance: AsmInspectionProvenance::with_span(method_provenance, parameter.span),
            })
            .collect(),
    )
}

fn asm_declared_state_type_kind(kind: DeclaredStateTypeKind) -> &'static str {
    match kind {
        DeclaredStateTypeKind::String => "string",
        DeclaredStateTypeKind::Number => "number",
        DeclaredStateTypeKind::Boolean => "boolean",
        DeclaredStateTypeKind::Null => "null",
    }
}

#[derive(Serialize)]
struct AsmInspectionDocument<'a> {
    schema_version: u32,
    file: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    files: Option<Vec<String>>,
    entities: Vec<AsmInspectionEntity<'a>>,
    references: Vec<AsmInspectionReference<'a>>,
    diagnostics: Vec<AsmInspectionDiagnostic<'a>>,
    validation: Vec<AsmInspectionDiagnostic<'a>>,
}

#[derive(Serialize)]
struct AsmEntityInspectionDocument<'a> {
    schema_version: u32,
    entity: AsmInspectionEntity<'a>,
    parents: Vec<&'a str>,
    children: Vec<&'a str>,
    descendant_count: usize,
    outgoing_references: Vec<AsmInspectionReference<'a>>,
    incoming_references: Vec<AsmInspectionReference<'a>>,
    diagnostics: Vec<AsmInspectionDiagnostic<'a>>,
}

#[derive(Serialize)]
struct AsmInspectionEntity<'a> {
    id: &'a str,
    kind: &'static str,
    owner: Option<&'a str>,
    provenance: AsmInspectionProvenance,
    #[serde(skip_serializing_if = "Option::is_none")]
    declared_type: Option<AsmInspectionDeclaredType<'a>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    initial_expression: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    local_variables: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    parameters: Option<Vec<AsmInspectionMethodParameter<'a>>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    semantic_type: Option<AsmInspectionSemanticType>,
    #[serde(skip_serializing_if = "Option::is_none")]
    computed: Option<AsmInspectionComputed>,
    #[serde(skip_serializing_if = "Option::is_none")]
    effect: Option<EffectInspection>,
    #[serde(skip_serializing_if = "Option::is_none")]
    context: Option<ezc_core::ContextInspection>,
    #[serde(skip_serializing_if = "Option::is_none")]
    component: Option<AsmInspectionComponent>,
}

#[derive(Serialize)]
struct AsmInspectionComponent {
    role: &'static str,
    slots: Vec<String>,
    invocations: Vec<String>,
    instances: Vec<String>,
    initialization_batches: Vec<usize>,
    structural_regions: Vec<String>,
}

#[derive(Serialize)]
struct AsmInspectionComputed {
    computed_type: String,
    dependencies: Vec<String>,
    dependents: Vec<String>,
    evaluation_order: Option<usize>,
    evaluation_batch: Option<usize>,
    purity: &'static str,
    serializability: &'static str,
    ir_function: Option<String>,
}

#[derive(Serialize)]
struct AsmInspectionSemanticType {
    type_text: String,
    origin: String,
    status: &'static str,
    provenance: AsmInspectionProvenance,
}

#[derive(Serialize)]
struct AsmInspectionMethodParameter<'a> {
    name: &'a str,
    provenance: AsmInspectionProvenance,
}

#[derive(Serialize)]
struct AsmInspectionDeclaredType<'a> {
    text: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    kind: Option<&'static str>,
    provenance: AsmInspectionProvenance,
}

#[derive(Serialize)]
struct AsmInspectionReference<'a> {
    kind: &'static str,
    source: &'a str,
    target: &'a str,
    provenance: AsmInspectionProvenance,
}

impl<'a> From<&'a ezc_core::SemanticReference> for AsmInspectionReference<'a> {
    fn from(reference: &'a ezc_core::SemanticReference) -> Self {
        Self {
            kind: semantic_reference_kind(reference.kind),
            source: reference.source.as_str(),
            target: reference.target.as_str(),
            provenance: (&reference.provenance).into(),
        }
    }
}

#[derive(Serialize)]
struct AsmInspectionDiagnostic<'a> {
    code: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    severity: Option<&'static str>,
    message: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    primary_provenance: Option<AsmInspectionProvenance>,
    #[serde(skip_serializing_if = "Option::is_none")]
    effect_id: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    statement_id: Option<&'a str>,
    context_declaration_candidate_id: Option<&'a str>,
    context_id: Option<&'a str>,
    provider_id: Option<&'a str>,
    consumer_id: Option<&'a str>,
    slot_id: Option<&'a str>,
    invocation_id: Option<&'a str>,
    component_instance_id: Option<&'a str>,
    slot_binding_id: Option<&'a str>,
    structural_region_id: Option<&'a str>,
    component_id: Option<&'a str>,
    provider_instance_id: Option<String>,
    consumer_instance_id: Option<String>,
    secondary_labels: Vec<AsmInspectionSecondaryLabel>,
}

impl<'a> From<&'a ezc_core::ComponentDiagnostic> for AsmInspectionDiagnostic<'a> {
    fn from(diagnostic: &'a ezc_core::ComponentDiagnostic) -> Self {
        Self {
            code: &diagnostic.code,
            severity: Some(diagnostic.severity.as_str()),
            message: &diagnostic.message,
            primary_provenance: diagnostic
                .provenance
                .as_ref()
                .map(AsmInspectionProvenance::from),
            effect_id: diagnostic
                .effect_id
                .as_ref()
                .map(ezc_core::EffectId::as_str),
            statement_id: diagnostic
                .statement_id
                .as_ref()
                .map(ezc_core::EffectStatementId::as_str),
            context_declaration_candidate_id: diagnostic
                .context_declaration_candidate_id
                .as_ref()
                .map(ezc_core::ContextDeclarationCandidateId::as_str),
            context_id: diagnostic
                .context_id
                .as_ref()
                .map(ezc_core::ContextId::as_str),
            provider_id: diagnostic
                .provider_id
                .as_ref()
                .map(ezc_core::ProviderId::as_str),
            consumer_id: diagnostic
                .consumer_id
                .as_ref()
                .map(ezc_core::ConsumerId::as_str),
            slot_id: diagnostic.slot_id.as_ref().map(ezc_core::SlotId::as_str),
            invocation_id: diagnostic
                .invocation_id
                .as_ref()
                .map(ezc_core::ComponentInvocationId::as_str),
            component_instance_id: diagnostic
                .component_instance_id
                .as_ref()
                .map(ezc_core::ComponentInstanceId::as_str),
            slot_binding_id: diagnostic
                .slot_binding_id
                .as_ref()
                .map(ezc_core::SlotBindingId::as_str),
            structural_region_id: diagnostic
                .structural_region_id
                .as_ref()
                .map(ezc_core::ComponentStructuralRegionId::as_str),
            component_id: diagnostic
                .component_id
                .as_ref()
                .map(ezc_core::SemanticId::as_str),
            provider_instance_id: diagnostic
                .provider_instance_id
                .as_ref()
                .map(ToString::to_string),
            consumer_instance_id: diagnostic
                .consumer_instance_id
                .as_ref()
                .map(ToString::to_string),
            secondary_labels: diagnostic
                .secondary_labels
                .iter()
                .map(AsmInspectionSecondaryLabel::from)
                .collect(),
        }
    }
}

#[derive(Serialize)]
struct AsmInspectionSecondaryLabel {
    provenance: AsmInspectionProvenance,
    message: String,
}

impl From<&ezc_core::DiagnosticSecondaryLabel> for AsmInspectionSecondaryLabel {
    fn from(label: &ezc_core::DiagnosticSecondaryLabel) -> Self {
        Self {
            provenance: (&label.provenance).into(),
            message: label.message.clone(),
        }
    }
}

#[derive(Serialize)]
struct AsmInspectionProvenance {
    path: String,
    start: usize,
    end: usize,
    line: usize,
    column: usize,
}

impl From<&SourceProvenance> for AsmInspectionProvenance {
    fn from(provenance: &SourceProvenance) -> Self {
        Self {
            path: provenance.path.display().to_string(),
            start: provenance.span.start,
            end: provenance.span.end,
            line: provenance.span.line,
            column: provenance.span.column,
        }
    }
}

impl AsmInspectionProvenance {
    fn with_span(provenance: &SourceProvenance, span: ezc_parser::SourceSpan) -> Self {
        Self {
            path: provenance.path.display().to_string(),
            start: span.start,
            end: span.end,
            line: span.line,
            column: span.column,
        }
    }
}

fn run_template(mut args: Vec<String>) {
    if args.is_empty() {
        eprintln!("missing file path");
        print_usage_and_exit();
    }

    let path = PathBuf::from(args.remove(0));

    let source = fs::read_to_string(&path).unwrap_or_else(|error| {
        eprintln!("failed to read {}: {error}", path.display());
        process::exit(1);
    });

    let parsed = parse_file(&path, &source);
    let component_graph = fold_component_graph(&build_component_graph(&parsed));
    let template_graph = build_template_graph(&component_graph);

    print_template_graph(&path, &template_graph);
}

fn run_html(mut args: Vec<String>) {
    if args.is_empty() {
        eprintln!("missing file path");
        print_usage_and_exit();
    }

    let path = PathBuf::from(args.remove(0));

    let source = fs::read_to_string(&path).unwrap_or_else(|error| {
        eprintln!("failed to read {}: {error}", path.display());
        process::exit(1);
    });

    let parsed = parse_file(&path, &source);
    let component_graph = fold_component_graph(&build_component_graph(&parsed));
    let template_graph = build_template_graph(&component_graph);
    let html = generate_static_html(&template_graph);

    print!("{html}");
}

fn run_manifest(mut args: Vec<String>) {
    if args.is_empty() {
        eprintln!("missing file path");
        print_usage_and_exit();
    }

    let path = PathBuf::from(args.remove(0));

    let source = fs::read_to_string(&path).unwrap_or_else(|error| {
        eprintln!("failed to read {}: {error}", path.display());
        process::exit(1);
    });

    let parsed = parse_file(&path, &source);
    let asm = build_application_semantic_model_for_unit(&CompilationUnit::from_parsed_files(vec![
        parsed,
    ]));
    let manifest = build_template_manifest_from_asm(&asm);

    println!("{}", template_manifest_json(&manifest));
}

fn run_build(mut args: Vec<String>) {
    if args.is_empty() {
        eprintln!("missing file path");
        print_usage_and_exit();
    }

    let input_path = PathBuf::from(args.remove(0));
    let out_dir = parse_out_dir(&args);

    let source = fs::read_to_string(&input_path).unwrap_or_else(|error| {
        eprintln!("failed to read {}: {error}", input_path.display());
        process::exit(1);
    });

    let parsed = parse_file(&input_path, &source);
    let unit = CompilationUnit::from_parsed_files(vec![parsed.clone()]);
    let asm = ConstantFoldingPass.transform(&build_application_semantic_model_for_unit(&unit));
    let ir = lower_components_to_ir(&asm);
    let computed_runtime_artifact = build_runtime_computed_artifact(&asm, &ir);
    let computed_runtime_json = runtime_computed_artifact_json(&computed_runtime_artifact);
    let effect_ir = optimize_effect_ir(&ir).output;
    let effect_runtime_artifact = build_runtime_effect_artifact(&asm, &effect_ir);
    let effect_runtime_json = runtime_effect_artifact_json(&effect_runtime_artifact);
    let context_ir = optimize_context_ir(&ir);
    let context_runtime_artifact = build_runtime_context_artifact(&asm, &context_ir);
    let context_runtime_json = runtime_context_artifact_json(&context_runtime_artifact);
    let component_runtime_artifact =
        build_runtime_component_artifact(&asm, &asm.component_ir_optimization);
    let component_runtime_json = runtime_component_artifact_json(&component_runtime_artifact);
    let forms_runtime_artifact = build_runtime_forms_artifact(&asm);
    let forms_runtime_json = runtime_forms_artifact_json(&forms_runtime_artifact);
    let component_graph = fold_component_graph(&build_component_graph(&parsed));
    let template_graph = build_template_graph(&component_graph);
    let html_fragment = generate_static_html(&template_graph);
    let manifest = build_template_manifest_from_asm(&asm);
    let manifest_json = template_manifest_json(&manifest);
    let page_title = page_title_from_graph(&template_graph);
    let page_html = generate_standalone_page_with_component_runtime_and_forms(
        &page_title,
        &html_fragment,
        &manifest,
        &computed_runtime_artifact,
        &context_runtime_artifact,
        &effect_runtime_artifact,
        &component_runtime_artifact,
        &forms_runtime_artifact,
    );
    let runtime_js = generate_runtime_stub();

    write_build_artifacts(
        &out_dir,
        &page_html,
        &manifest_json,
        &computed_runtime_json,
        &context_runtime_json,
        &effect_runtime_json,
        &component_runtime_json,
        &forms_runtime_json,
        &runtime_js,
    )
    .unwrap_or_else(|error| {
        eprintln!(
            "failed to write build artifacts to {}: {error}",
            out_dir.display()
        );

        process::exit(1);
    });

    println!("Wrote {}", out_dir.join("index.html").display());
    println!("Wrote {}", out_dir.join("template.manifest.json").display());
    println!("Wrote {}", out_dir.join("computed.runtime.json").display());
    println!("Wrote {}", out_dir.join("context.runtime.json").display());
    println!("Wrote {}", out_dir.join("effect.runtime.json").display());
    println!("Wrote {}", out_dir.join("component.runtime.json").display());
    println!("Wrote {}", out_dir.join("forms.runtime.json").display());
    println!("Wrote {}", out_dir.join("runtime.js").display());
}

fn run_parse(mut args: Vec<String>) {
    if args.is_empty() {
        eprintln!("missing file path");
        print_usage_and_exit();
    }

    let path = PathBuf::from(args.remove(0));

    let source = fs::read_to_string(&path).unwrap_or_else(|error| {
        eprintln!("failed to read {}: {error}", path.display());
        process::exit(1);
    });

    let parsed = parse_file(&path, &source);

    print_parsed_file(&parsed);
}

fn page_title_from_graph(graph: &TemplateGraph) -> String {
    graph.templates.first().map_or_else(
        || "EdgeZero App".to_string(),
        |template| template.component_name.clone(),
    )
}

fn parse_format(args: &[String]) -> String {
    let mut format = "text".to_string();

    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--format" => {
                let Some(value) = args.get(index + 1) else {
                    eprintln!("missing value for --format");
                    process::exit(1);
                };

                format.clone_from(value);
                index += 2;
            }
            unknown => {
                eprintln!("unknown option: {unknown}");
                process::exit(1);
            }
        }
    }

    format
}

fn parse_out_dir(args: &[String]) -> PathBuf {
    let mut out_dir = None;

    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--out" => {
                let Some(value) = args.get(index + 1) else {
                    eprintln!("missing value for --out");
                    process::exit(1);
                };

                out_dir = Some(PathBuf::from(value));
                index += 2;
            }
            unknown => {
                eprintln!("unknown option: {unknown}");
                process::exit(1);
            }
        }
    }

    out_dir.unwrap_or_else(|| PathBuf::from("dist"))
}

fn print_parsed_file(parsed: &ParsedFile) {
    println!("File: {}", parsed.path.display());
    print_parse_diagnostics(parsed);
    print_parsed_classes(parsed);
}

fn print_parse_diagnostics(parsed: &ParsedFile) {
    println!("Diagnostics:");
    if parsed.diagnostics.is_empty() {
        println!("  none");
    } else {
        for diagnostic in &parsed.diagnostics {
            println!(
                "  {}: {}",
                diagnostic_severity_label(&diagnostic.severity),
                diagnostic.message
            );

            for label in &diagnostic.labels {
                println!(
                    "    at {}:{} span={}..{}",
                    label.span.line, label.span.column, label.span.start, label.span.end
                );
            }
        }
    }
}

fn print_parsed_classes(parsed: &ParsedFile) {
    println!();
    println!("Classes:");
    if parsed.classes.is_empty() {
        println!("  none");
        return;
    }

    for class in &parsed.classes {
        print_parsed_class(class);
    }
}

fn print_parsed_class(class: &ParsedClass) {
    println!(
        "  class {} at {}:{}",
        class.name, class.span.line, class.span.column
    );

    print_parsed_decorators(class);
    print_parsed_properties(class);
    print_parsed_methods(&class.methods);
}

fn print_parsed_decorators(class: &ParsedClass) {
    println!("    decorators:");
    if class.decorators.is_empty() {
        println!("      none");
    } else {
        for decorator in &class.decorators {
            match &decorator.argument {
                Some(argument) => {
                    println!(
                        "      @{}({argument:?}) at {}:{}",
                        decorator.name, decorator.span.line, decorator.span.column
                    );
                }
                None => {
                    println!(
                        "      @{} at {}:{}",
                        decorator.name, decorator.span.line, decorator.span.column
                    );
                }
            }
        }
    }
}

fn print_parsed_properties(class: &ParsedClass) {
    println!("    properties:");
    if class.properties.is_empty() {
        println!("      none");
    } else {
        for property in &class.properties {
            match &property.initializer {
                Some(initializer) => {
                    println!(
                        "      {} = {} at {}:{}",
                        property.name, initializer, property.span.line, property.span.column
                    );
                }
                None => {
                    println!(
                        "      {} at {}:{}",
                        property.name, property.span.line, property.span.column
                    );
                }
            }
        }
    }
}

fn print_parsed_methods(methods: &[ParsedMethod]) {
    println!("    methods:");
    if methods.is_empty() {
        println!("      none");
    } else {
        for method in methods {
            print_parsed_method(method);
        }
    }
}

fn print_parsed_method(method: &ParsedMethod) {
    println!(
        "      {} at {}:{}",
        method.name, method.span.line, method.span.column
    );

    if method.jsx_roots.is_empty() {
        println!("        jsx roots: none");
    } else {
        for jsx in &method.jsx_roots {
            print_parsed_jsx_root(jsx);
        }
    }

    if method.bindings.is_empty() {
        println!("        bindings: none");
    } else {
        println!("        bindings: {}", method.bindings.join(", "));
    }
}

fn print_parsed_jsx_root(root: &ParsedJsxNode) {
    match root {
        ParsedJsxNode::Element(element) => {
            println!(
                "        jsx root <{}> at {}:{}",
                element.name, element.span.line, element.span.column
            );
            print_parsed_jsx_element_details(element, 10);
        }
        ParsedJsxNode::Fragment(fragment) => {
            println!(
                "        jsx root <> at {}:{}",
                fragment.span.line, fragment.span.column
            );
            print_parsed_jsx_children(&fragment.children, 10);
        }
    }
}

fn print_parsed_jsx_element_details(element: &ezc_parser::ParsedJsxElement, indent: usize) {
    let padding = " ".repeat(indent);

    if element.attributes.is_empty() {
        println!("{padding}attributes: none");
    } else {
        println!(
            "{padding}attributes: {}",
            element
                .attributes
                .iter()
                .map(format_parsed_attribute)
                .collect::<Vec<_>>()
                .join(", ")
        );
    }

    if element.event_handlers.is_empty() {
        println!("{padding}event handlers: none");
    } else {
        println!(
            "{padding}event handlers: {}",
            format_parsed_event_handlers(&element.event_handlers).join(", ")
        );
    }

    print_parsed_jsx_children(&element.children, indent);
}

fn print_parsed_jsx_children(children: &[ParsedJsxChild], indent: usize) {
    let padding = " ".repeat(indent);

    if children.is_empty() {
        println!("{padding}children: none");
    } else {
        println!("{padding}children:");
        for child in children {
            println!("{padding}  {}", format_parsed_child(child));
        }
    }
}

fn print_component_graph(path: &Path, graph: &ComponentGraph) {
    println!("File: {}", path.display());

    println!("ComponentGraph:");

    println!("  diagnostics:");
    if graph.diagnostics.is_empty() {
        println!("    none");
    } else {
        for diagnostic in &graph.diagnostics {
            println!("    {}: {}", diagnostic.code, diagnostic.message);
        }
    }

    println!("  components:");
    if graph.components.is_empty() {
        println!("    none");
        return;
    }

    for component in &graph.components {
        println!("    component {}", component.class_name);

        match &component.element_name {
            Some(element_name) => println!("      element: {element_name}"),
            None => println!("      element: <missing>"),
        }

        match &component.route_path {
            Some(route_path) => println!("      route: {route_path}"),
            None => println!("      route: none"),
        }

        println!("      state:");
        if component.state_fields.is_empty() {
            println!("        none");
        } else {
            for state in &component.state_fields {
                if let Some(expression) = &state.initial_expression {
                    println!("        {} = {expression}", state.name);
                } else {
                    println!("        {}", state.name);
                }
            }
        }

        println!("      methods:");
        if component.methods.is_empty() {
            println!("        none");
        } else {
            for method in &component.methods {
                println!("        {}", method.name);
            }
        }

        if !component.actions.is_empty() {
            println!("      actions:");
            for action in &component.actions {
                println!(
                    "        {}: {} {}",
                    action.method,
                    format_state_operation(&action.operation),
                    action.field
                );
            }
        }

        println!("      render:");
        match &component.render {
            Some(render) => {
                print_render_root(render);

                if let Some(fragment) = &render.root_fragment {
                    println!("        attributes: none");
                    println!("        event handlers: none");
                    print_render_children(&fragment.children, 8);
                } else {
                    if render.attributes.is_empty() {
                        println!("        attributes: none");
                    } else {
                        println!(
                            "        attributes: {}",
                            render
                                .attributes
                                .iter()
                                .map(format_render_attribute)
                                .collect::<Vec<_>>()
                                .join(", ")
                        );
                    }

                    if render.event_handlers.is_empty() {
                        println!("        event handlers: none");
                    } else {
                        println!(
                            "        event handlers: {}",
                            format_render_event_handlers(&render.event_handlers).join(", ")
                        );
                    }

                    print_render_children(&render.children, 8);
                }

                if render.bindings.is_empty() {
                    println!("        bindings: none");
                } else {
                    println!("        bindings: {}", render.bindings.join(", "));
                }
            }
            None => {
                println!("        none");
            }
        }
    }
}

fn print_render_children(children: &[ezc_core::RenderChild], indent: usize) {
    let padding = " ".repeat(indent);

    if children.is_empty() {
        println!("{padding}children: none");
    } else {
        println!("{padding}children:");
        for child in children {
            println!("{padding}  {}", format_render_child(child));
        }
    }
}

fn print_render_root(render: &ezc_core::RenderModel) {
    match (&render.root_element, &render.root_fragment) {
        (Some(root), _) => println!("        root: <{root}>"),
        (None, Some(fragment)) => println!(
            "        root: <> {}",
            format_line_column_span(&fragment.span)
        ),
        (None, None) => println!("        root: none"),
    }
}

fn format_state_operation(operation: &StateOperation) -> &'static str {
    match operation {
        StateOperation::Increment => "increment",
        StateOperation::Decrement => "decrement",
        StateOperation::AddAssign(_) => "add-assign",
        StateOperation::SubtractAssign(_) => "subtract-assign",
        StateOperation::Assign(_) => "assign",
        StateOperation::Toggle => "toggle",
    }
}

fn format_parsed_event_handlers(event_handlers: &[ezc_parser::ParsedEventHandler]) -> Vec<String> {
    event_handlers
        .iter()
        .map(|event_handler| format!("{} -> {}", event_handler.event, event_handler.handler))
        .collect()
}

fn format_render_event_handlers(event_handlers: &[ezc_core::RenderEventHandler]) -> Vec<String> {
    event_handlers
        .iter()
        .map(|event_handler| format!("{} -> {}", event_handler.event, event_handler.handler))
        .collect()
}

fn format_parsed_child(child: &ParsedJsxChild) -> String {
    match child {
        ParsedJsxChild::Text { value, span } => {
            format!("Text({value:?}) {}", format_line_column_span(span))
        }
        ParsedJsxChild::Binding { expression, span } => {
            format!("Binding({expression:?}) {}", format_line_column_span(span))
        }
        ParsedJsxChild::Element(element) => format!(
            "Element <{}> {}",
            element.name,
            format_line_column_span(&element.span)
        ),
        ParsedJsxChild::Fragment(fragment) => {
            format!("Fragment <> {}", format_line_column_span(&fragment.span))
        }
        ParsedJsxChild::Conditional(conditional) => format!(
            "Conditional({:?}) {}",
            conditional.condition,
            format_line_column_span(&conditional.span)
        ),
        ParsedJsxChild::List(list) => format!(
            "List(iterable={:?}, item={:?}, index={:?}, key={:?}) {}",
            list.iterable,
            list.item_variable,
            list.index_variable,
            list.key_expression,
            format_line_column_span(&list.span)
        ),
    }
}

fn format_render_child(child: &ezc_core::RenderChild) -> String {
    match child {
        ezc_core::RenderChild::Text { value, span } => {
            format!("Text({value:?}) {}", format_line_column_span(span))
        }
        ezc_core::RenderChild::Binding { expression, span } => {
            format!("Binding({expression:?}) {}", format_line_column_span(span))
        }
        ezc_core::RenderChild::Element(element) => format!(
            "Element <{}> {}",
            element.tag_name,
            format_line_column_span(&element.span)
        ),
        ezc_core::RenderChild::Fragment(fragment) => {
            format!("Fragment <> {}", format_line_column_span(&fragment.span))
        }
        ezc_core::RenderChild::Conditional(conditional) => format!(
            "Conditional({:?}) {}",
            conditional.condition,
            format_line_column_span(&conditional.span)
        ),
        ezc_core::RenderChild::List(list) => format!(
            "List(iterable={:?}, item={:?}, index={:?}, key={:?}) {}",
            list.iterable,
            list.item_variable,
            list.index_variable,
            list.key_expression,
            format_line_column_span(&list.span)
        ),
    }
}

fn print_template_graph(path: &Path, graph: &TemplateGraph) {
    println!("File: {}", path.display());

    println!("TemplateGraph:");
    println!("  templates:");

    if graph.templates.is_empty() {
        println!("    none");
        return;
    }

    for template in &graph.templates {
        println!("    template {}", template.component_name);

        match &template.root {
            Some(root) => {
                println!(
                    "      root: <{}> id={} {}",
                    root.tag_name,
                    root.id.0,
                    format_source_span(path, &root.span)
                );

                println!("      attributes:");
                if root.attributes.is_empty() {
                    println!("        none");
                } else {
                    for attribute in &root.attributes {
                        println!(
                            "        {} = {} {}",
                            attribute.name,
                            format_attribute_value(&attribute.value),
                            format_optional_source_span(path, attribute.span.as_ref())
                        );
                    }
                }

                println!("      children:");
                if root.children.is_empty() {
                    println!("        none");
                } else {
                    for child in &root.children {
                        print_template_child(path, child, 8);
                    }
                }
            }
            None => {
                if let Some(fragment) = &template.root_fragment {
                    println!(
                        "      root: <> id={} {}",
                        fragment.id.0,
                        format_source_span(path, &fragment.span)
                    );

                    println!("      children:");
                    if fragment.children.is_empty() {
                        println!("        none");
                    } else {
                        for child in &fragment.children {
                            print_template_child(path, child, 8);
                        }
                    }
                } else {
                    println!("      root: none");
                }
            }
        }
    }
}

fn print_template_child(path: &Path, child: &TemplateChild, indent: usize) {
    let padding = " ".repeat(indent);

    match child {
        TemplateChild::Text { value, span } => {
            println!(
                "{padding}Text({value:?}) {}",
                format_source_span(path, span)
            );
        }
        TemplateChild::Binding {
            id,
            expression,
            initial_value,
            span,
        } => {
            println!(
                "{padding}Binding id={} expression={expression:?} initial={} {}",
                id.0,
                format_serializable_value(initial_value.as_ref()),
                format_source_span(path, span)
            );
        }
        TemplateChild::Element(element) => {
            println!(
                "{padding}Element <{}> id={} {}",
                element.tag_name,
                element.id.0,
                format_source_span(path, &element.span)
            );

            let child_padding = " ".repeat(indent + 2);

            println!("{child_padding}attributes:");
            if element.attributes.is_empty() {
                println!("{child_padding}  none");
            } else {
                for attribute in &element.attributes {
                    println!(
                        "{child_padding}  {} = {} {}",
                        attribute.name,
                        format_attribute_value(&attribute.value),
                        format_optional_source_span(path, attribute.span.as_ref())
                    );
                }
            }

            println!("{child_padding}children:");
            if element.children.is_empty() {
                println!("{child_padding}  none");
            } else {
                for child in &element.children {
                    print_template_child(path, child, indent + 4);
                }
            }
        }
        TemplateChild::Fragment(fragment) => {
            println!(
                "{padding}Fragment <> id={} {}",
                fragment.id.0,
                format_source_span(path, &fragment.span)
            );

            let child_padding = " ".repeat(indent + 2);
            println!("{child_padding}children:");
            if fragment.children.is_empty() {
                println!("{child_padding}  none");
            } else {
                for child in &fragment.children {
                    print_template_child(path, child, indent + 4);
                }
            }
        }
        TemplateChild::Conditional(conditional) => {
            println!(
                "{padding}Conditional id={} start={} end={} condition={:?} initial={} {}",
                conditional.id.0,
                conditional.start_id.0,
                conditional.end_id.0,
                conditional.condition,
                format_serializable_value(conditional.initial_value.as_ref()),
                format_source_span(path, &conditional.span)
            );

            let child_padding = " ".repeat(indent + 2);
            println!("{child_padding}true:");
            if conditional.when_true.is_empty() {
                println!("{child_padding}  none");
            } else {
                for child in &conditional.when_true {
                    print_template_child(path, child, indent + 4);
                }
            }

            println!("{child_padding}false:");
            if conditional.when_false.is_empty() {
                println!("{child_padding}  none");
            } else {
                for child in &conditional.when_false {
                    print_template_child(path, child, indent + 4);
                }
            }
        }
        TemplateChild::List(list) => print_template_list(path, list, indent),
    }
}

fn print_template_list(path: &Path, list: &ezc_core::ListNode, indent: usize) {
    let padding = " ".repeat(indent);
    println!(
                "{padding}List id={} start={} end={} iterable={:?} initial={} item={:?} index={:?} key={:?} {}",
                list.id.0,
                list.start_id.0,
                list.end_id.0,
                list.iterable,
                format_serializable_value(list.initial_value.as_ref()),
                list.item_variable,
        list.index_variable,
        list.key_expression,
        format_source_span(path, &list.span)
    );

    let child_padding = " ".repeat(indent + 2);
    println!("{child_padding}item template:");
    if list.item_template.is_empty() {
        println!("{child_padding}  none");
    } else {
        for child in &list.item_template {
            print_template_child(path, child, indent + 4);
        }
    }
}

fn format_source_span(path: &Path, span: &SourceSpan) -> String {
    format!(
        "@ {}:{}:{} span={}..{}",
        path.display(),
        span.line,
        span.column,
        span.start,
        span.end
    )
}

fn format_line_column_span(span: &SourceSpan) -> String {
    format!(
        "@ {}:{} span={}..{}",
        span.line, span.column, span.start, span.end
    )
}

fn format_optional_source_span(path: &Path, span: Option<&SourceSpan>) -> String {
    span.map_or_else(
        || "@ generated".to_string(),
        |span| format_source_span(path, span),
    )
}

fn format_serializable_value(value: Option<&SerializableValue>) -> String {
    match value {
        Some(SerializableValue::Null) => "Some(null)".to_string(),
        Some(SerializableValue::Number(value) | SerializableValue::String(value)) => {
            format!("Some({value:?})")
        }
        Some(SerializableValue::Boolean(value)) => format!("Some({value})"),
        Some(SerializableValue::Array(values)) => format!("Some({values:?})"),
        Some(SerializableValue::Object(values)) => format!("Some({values:?})"),
        None => "None".to_string(),
    }
}

fn format_attribute_value(value: &AttributeValue) -> String {
    match value {
        AttributeValue::Boolean => "boolean".to_string(),
        AttributeValue::Static(value) => format!("{value:?}"),
        AttributeValue::Binding {
            id,
            expression,
            initial_value,
        } => format!(
            "binding(id={} expression={expression:?} initial={})",
            id.0,
            format_serializable_value(initial_value.as_ref())
        ),
        AttributeValue::EventHandler { event, handler } => {
            format!("event-handler({event} -> {handler})")
        }
        AttributeValue::BindingList(bindings) => format!("bindings({})", bindings.join(", ")),
    }
}

fn format_parsed_attribute(attribute: &ParsedJsxAttribute) -> String {
    match &attribute.value {
        ParsedJsxAttributeValue::Boolean => attribute.name.clone(),
        ParsedJsxAttributeValue::Static(value) => format!("{}={value:?}", attribute.name),
        ParsedJsxAttributeValue::Expression(_) => format!("{}={{...}}", attribute.name),
        ParsedJsxAttributeValue::Spread(_) => "{...}".to_string(),
        ParsedJsxAttributeValue::Unsupported => format!("{}=<complex>", attribute.name),
    }
}

fn format_render_attribute(attribute: &RenderAttribute) -> String {
    match &attribute.value {
        RenderAttributeValue::Boolean => attribute.name.clone(),
        RenderAttributeValue::Static(value) => format!("{}={value:?}", attribute.name),
        RenderAttributeValue::Expression(_) => format!("{}={{...}}", attribute.name),
        RenderAttributeValue::Spread(_) => "{...}".to_string(),
        RenderAttributeValue::Unsupported => format!("{}=<complex>", attribute.name),
    }
}

fn diagnostic_severity_label(severity: &ParseSeverity) -> &'static str {
    match severity {
        ParseSeverity::Info => "Info",
        ParseSeverity::Warning => "Warning",
        ParseSeverity::Error => "Error",
    }
}

#[allow(clippy::too_many_arguments)]
fn write_build_artifacts(
    out_dir: &PathBuf,
    html: &str,
    manifest_json: &str,
    computed_runtime_json: &str,
    context_runtime_json: &str,
    effect_runtime_json: &str,
    component_runtime_json: &str,
    forms_runtime_json: &str,
    runtime_js: &str,
) -> io::Result<()> {
    fs::create_dir_all(out_dir)?;

    fs::write(out_dir.join("index.html"), html)?;

    fs::write(out_dir.join("template.manifest.json"), manifest_json)?;

    fs::write(out_dir.join("computed.runtime.json"), computed_runtime_json)?;

    fs::write(out_dir.join("context.runtime.json"), context_runtime_json)?;

    fs::write(out_dir.join("effect.runtime.json"), effect_runtime_json)?;
    fs::write(
        out_dir.join("component.runtime.json"),
        component_runtime_json,
    )?;
    fs::write(out_dir.join("forms.runtime.json"), forms_runtime_json)?;

    fs::write(out_dir.join("runtime.js"), runtime_js)?;

    Ok(())
}

fn print_usage_and_exit() -> ! {
    eprintln!("usage:");
    eprintln!("  ezc_cli explain <file> [--format text|json]");
    eprintln!("  ezc_cli explain <file> [--entity semantic-id | --source path --offset byte] [--child-kind kind] [--reference-kind kind] [--format text|json]");
    eprintln!("  ezc_cli asm <file> [--entity semantic-id | --source path --offset byte] [--child-kind kind] [--reference-kind kind] [--format text|json|graph]");
    eprintln!(
        "  ezc_cli check <file> [file...] [--format text|json] [--category parser|compiler|validation] [--fail-on error|warning|info]"
    );
    eprintln!("  ezc_cli parse <file>");
    eprintln!("  ezc_cli graph <file>");
    eprintln!("  ezc_cli template <file>");
    eprintln!("  ezc_cli html <file>");
    eprintln!("  ezc_cli manifest <file>");
    eprintln!("  ezc_cli build <file> [--out dir]");
    process::exit(1);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formats_sorted_asm_validation_diagnostics_only_when_present() {
        assert!(asm_validation_diagnostics_text(&[]).is_none());

        let diagnostics = vec![
            AsmValidationDiagnostic {
                code: "EZASM1002".to_string(),
                message: "second".to_string(),
            },
            AsmValidationDiagnostic {
                code: "EZASM1001".to_string(),
                message: "first".to_string(),
            },
        ];

        assert_eq!(
            asm_validation_diagnostics_text(&diagnostics),
            Some(
                "  ASM validation diagnostics:\n    EZASM1001: first\n    EZASM1002: second\n"
                    .to_string()
            )
        );
    }

    #[test]
    fn i6_validation_products_do_not_leak_into_asm_inspection_schema_v8() {
        let path = PathBuf::from("src/Profile.tsx");
        let parsed = ezc_parser::parse_file(
            &path,
            r#"
@component("profile")
class Profile {
  @form() profile!: Form;
  @validate(required())
  @field(this.profile)
  name = "";
  render() { return <div />; }
}
"#,
        );
        let asm =
            build_application_semantic_model_for_unit(&CompilationUnit::from_parsed_files(vec![
                parsed,
            ]));
        assert_eq!(asm.validation_rules.len(), 1);
        let document = asm_inspection_json(&[path], &asm, &[]);
        let json: serde_json::Value = serde_json::from_str(&document).unwrap();
        assert_eq!(json["schema_version"], ASM_INSPECTION_SCHEMA_VERSION);
        assert!(!document.contains("validation-rule"));
        assert!(!document.contains("validation_rule"));
    }
}
