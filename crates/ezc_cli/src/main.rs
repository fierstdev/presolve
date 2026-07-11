use std::env;
use std::fmt::Write as _;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process;

use ezc_core::{
    build_application_semantic_model_for_unit, build_component_graph, build_template_graph,
    build_template_manifest, explain_json, explain_text, generate_runtime_stub,
    generate_standalone_page, generate_static_html, summarize_source, template_manifest_json,
    validate_application_semantic_model, ApplicationSemanticModel, AsmValidationDiagnostic,
    AttributeValue, CompilationUnit, ComponentGraph, DeclaredStateTypeKind, RenderAttribute,
    RenderAttributeValue, SemanticEntity, SemanticOwner, SemanticReferenceKind, SerializableValue,
    SourceProvenance, StateOperation, TemplateChild, TemplateGraph, TemplateSemanticKind,
};
use ezc_parser::{
    parse_file, ParseSeverity, ParsedClass, ParsedFile, ParsedJsxAttribute,
    ParsedJsxAttributeValue, ParsedJsxChild, ParsedJsxNode, ParsedMethod, SourceSpan,
};
use serde::Serialize;

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
    let graph = build_component_graph(&parsed);

    print_component_graph(&path, &graph);
}

fn run_asm(args: &[String]) {
    let (input_paths, format) = parse_asm_inputs(args);
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
    let paths = unit
        .files()
        .iter()
        .map(|file| file.path.clone())
        .collect::<Vec<_>>();
    let asm = build_application_semantic_model_for_unit(&unit);
    let validation = validate_application_semantic_model(&asm);

    match format.as_str() {
        "text" => print_asm_text(&paths, &asm, &validation),
        "json" => print!("{}", asm_inspection_json(&paths, &asm, &validation)),
        _ => {
            eprintln!("unsupported format: {format}");
            process::exit(1);
        }
    }
}

fn run_check(args: &[String]) {
    let (input_paths, format, categories) = parse_check_inputs(args);
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
        ),
        "json" => print!("{}", check_json(&unit, &asm, &validation, &categories)),
        _ => {
            eprintln!("unsupported format: {format}");
            process::exit(1);
        }
    }

    if parser_diagnostic_count + asm.diagnostics.len() + validation.len() > 0 {
        process::exit(1);
    }
}

fn print_check_text(
    unit: &CompilationUnit,
    asm: &ApplicationSemanticModel,
    validation: &[AsmValidationDiagnostic],
    parser_diagnostic_count: usize,
    categories: &[String],
) {
    println!("Check:");
    println!("  files: {}", unit.files().len());
    println!("  parser diagnostics: {parser_diagnostic_count}");
    println!("  compiler diagnostics: {}", asm.diagnostics.len());
    println!("  ASM validation diagnostics: {}", validation.len());
    if check_category_enabled(categories, "parser") {
        for file in unit.files() {
            for diagnostic in &file.diagnostics {
                println!(
                    "  parser {}: {}",
                    diagnostic_severity_label(&diagnostic.severity),
                    diagnostic.message
                );
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
) -> String {
    let parser_count = unit
        .files()
        .iter()
        .map(|file| file.diagnostics.len())
        .sum::<usize>();
    let parser_diagnostics = if check_category_enabled(categories, "parser") {
        unit.files().iter().flat_map(|file| file.diagnostics.iter().map(move |diagnostic| serde_json::json!({"path": file.path, "severity": diagnostic_severity_label(&diagnostic.severity), "message": diagnostic.message}))).collect::<Vec<_>>()
    } else {
        Vec::new()
    };
    let compiler_diagnostics = if check_category_enabled(categories, "compiler") {
        asm.diagnostics.iter().map(|diagnostic| serde_json::json!({"code": diagnostic.code, "message": diagnostic.message})).collect::<Vec<_>>()
    } else {
        Vec::new()
    };
    let validation_diagnostics = if check_category_enabled(categories, "validation") {
        validation.iter().map(|diagnostic| serde_json::json!({"code": diagnostic.code, "message": diagnostic.message})).collect::<Vec<_>>()
    } else {
        Vec::new()
    };
    serde_json::to_string_pretty(&serde_json::json!({"schema_version": 1, "files": unit.files().iter().map(|file| file.path.display().to_string()).collect::<Vec<_>>(), "summary": {"parser_diagnostics": parser_count, "compiler_diagnostics": asm.diagnostics.len(), "validation": validation.len()}, "categories": categories, "parser_diagnostics": parser_diagnostics, "compiler_diagnostics": compiler_diagnostics, "validation": validation_diagnostics})).expect("check document should serialize") + "\n"
}

fn check_category_enabled(categories: &[String], category: &str) -> bool {
    categories.is_empty() || categories.iter().any(|item| item == category)
}

fn print_asm_text(
    paths: &[PathBuf],
    asm: &ApplicationSemanticModel,
    validation: &[AsmValidationDiagnostic],
) {
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
    println!("  ownership: {}", asm.ownership.len());
    println!("  references: {}", asm.references.len());
    println!("  provenance: {}", asm.provenance.len());
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
            println!("    {}: {}", diagnostic.code, diagnostic.message);
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
        }
    }
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
    let mut references = asm
        .references
        .iter()
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

    let mut diagnostics = asm
        .diagnostics
        .iter()
        .map(|diagnostic| AsmInspectionDiagnostic {
            code: &diagnostic.code,
            message: &diagnostic.message,
            provenance: diagnostic
                .provenance
                .as_ref()
                .map(AsmInspectionProvenance::from),
        })
        .collect::<Vec<_>>();
    diagnostics.sort_by(|left, right| (left.code, left.message).cmp(&(right.code, right.message)));

    let mut validation = validation
        .iter()
        .map(|diagnostic| AsmInspectionDiagnostic {
            code: &diagnostic.code,
            message: &diagnostic.message,
            provenance: None,
        })
        .collect::<Vec<_>>();
    validation.sort_by(|left, right| (left.code, left.message).cmp(&(right.code, right.message)));

    let document = AsmInspectionDocument {
        schema_version: 1,
        file: paths[0].display().to_string(),
        files: (paths.len() > 1).then(|| {
            paths
                .iter()
                .map(|path| path.display().to_string())
                .collect()
        }),
        entities: asm
            .ownership
            .iter()
            .map(|(id, owner)| {
                let entity = asm
                    .entity(id)
                    .expect("ASM ownership should only contain semantic entities");
                let provenance = asm
                    .provenance(id)
                    .expect("ASM ownership should have source provenance");

                AsmInspectionEntity {
                    id: id.as_str(),
                    kind: semantic_entity_kind(entity),
                    owner: semantic_owner_id(owner),
                    provenance: provenance.into(),
                    declared_type: declared_state_type(entity),
                }
            })
            .collect(),
        references,
        diagnostics,
        validation,
    };

    serde_json::to_string_pretty(&document).expect("ASM inspection document should serialize")
        + "\n"
}

fn parse_asm_inputs(args: &[String]) -> (Vec<PathBuf>, String) {
    let mut paths = Vec::new();
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

    (paths, format)
}

fn parse_check_inputs(args: &[String]) -> (Vec<PathBuf>, String, Vec<String>) {
    if args.is_empty() {
        eprintln!("missing file path");
        print_usage_and_exit();
    }

    let mut paths = Vec::new();
    let mut format = "text".to_string();
    let mut categories = Vec::new();
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
    (paths, format, categories)
}

fn semantic_entity_kind(entity: SemanticEntity<'_>) -> &'static str {
    match entity {
        SemanticEntity::Component(_) => "component",
        SemanticEntity::StateField(_) => "state-field",
        SemanticEntity::Method(_) => "method",
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
        SemanticReferenceKind::EventMethod => "event-method",
        SemanticReferenceKind::TemplateState => "template-state",
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
struct AsmInspectionEntity<'a> {
    id: &'a str,
    kind: &'static str,
    owner: Option<&'a str>,
    provenance: AsmInspectionProvenance,
    #[serde(skip_serializing_if = "Option::is_none")]
    declared_type: Option<AsmInspectionDeclaredType<'a>>,
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

#[derive(Serialize)]
struct AsmInspectionDiagnostic<'a> {
    code: &'a str,
    message: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    provenance: Option<AsmInspectionProvenance>,
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
    let component_graph = build_component_graph(&parsed);
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
    let component_graph = build_component_graph(&parsed);
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
    let component_graph = build_component_graph(&parsed);
    let template_graph = build_template_graph(&component_graph);
    let manifest = build_template_manifest(&component_graph, &template_graph);

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
    let component_graph = build_component_graph(&parsed);
    let template_graph = build_template_graph(&component_graph);
    let html_fragment = generate_static_html(&template_graph);
    let manifest = build_template_manifest(&component_graph, &template_graph);
    let manifest_json = template_manifest_json(&manifest);
    let page_title = page_title_from_graph(&template_graph);
    let page_html = generate_standalone_page(&page_title, &html_fragment, &manifest);
    let runtime_js = generate_runtime_stub();

    write_build_artifacts(&out_dir, &page_html, &manifest_json, &runtime_js).unwrap_or_else(
        |error| {
            eprintln!(
                "failed to write build artifacts to {}: {error}",
                out_dir.display()
            );

            process::exit(1);
        },
    );

    println!("Wrote {}", out_dir.join("index.html").display());
    println!("Wrote {}", out_dir.join("template.manifest.json").display());
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
                println!("        {}", state.name);
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

fn write_build_artifacts(
    out_dir: &PathBuf,
    html: &str,
    manifest_json: &str,
    runtime_js: &str,
) -> io::Result<()> {
    fs::create_dir_all(out_dir)?;

    fs::write(out_dir.join("index.html"), html)?;

    fs::write(out_dir.join("template.manifest.json"), manifest_json)?;

    fs::write(out_dir.join("runtime.js"), runtime_js)?;

    Ok(())
}

fn print_usage_and_exit() -> ! {
    eprintln!("usage:");
    eprintln!("  ezc_cli explain <file> [--format text|json]");
    eprintln!("  ezc_cli asm <file> [--format text|json]");
    eprintln!("  ezc_cli check <file> [file...] [--format text|json]");
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
}
