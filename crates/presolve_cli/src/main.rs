use std::collections::{BTreeMap, BTreeSet};
use std::env;
use std::fmt::Write as _;
use std::fs;
use std::io::{self, Read as _, Write as _};
use std::net::{TcpListener, TcpStream};
use std::path::{Component, Path, PathBuf};
use std::process;
use std::sync::atomic::{AtomicU64, Ordering};

use presolve_cli::{
    load_explicit_project_envelope_v1, load_explicit_source_inputs_v1,
    parse_explicit_source_spec_v1, run_explicit_build_or_check_v1, run_explicit_watch_once_v1,
    run_explicit_workspace_v1, run_project_cache_operation_v1, CliCacheOperationV1,
};

use presolve_compiler::tooling_reader::{read_tooling_product_v1, ToolingProductV1};
use presolve_compiler::{
    build_application_publication_product_v1, build_application_semantic_model_for_unit,
    build_application_semantic_model_for_unit_with_packages, build_component_graph,
    build_context_inspection_registry, build_effect_inspection_registry,
    build_form_inspection_registry, build_production_reachability_graph, build_production_reports,
    build_production_runtime_artifact, build_resume_chunk_graph, build_resume_manifest,
    build_runtime_component_artifact, build_runtime_computed_artifact,
    build_runtime_context_artifact, build_runtime_effect_artifact, build_runtime_forms_artifact,
    build_runtime_opaque_artifact_with_modules, build_runtime_resource_artifact_with_modules,
    build_semantic_graph, build_static_request_handoff_v1, build_template_graph,
    build_template_manifest_from_asm, build_validated_route_graph_v1, discover_project_v1,
    discover_semantic_packages_v1, embed_opaque_runtime_artifact, emit_production_modules,
    explain_json, explain_text, extract_production_chunk_graph, fold_component_graph,
    generate_ordinary_instance_html, generate_runtime_stub,
    generate_standalone_page_with_resume_runtime,
    generate_standalone_page_with_resume_runtime_and_resources, generate_static_html,
    lower_components_to_ir, optimization_report_json, optimize_context_ir, optimize_effect_ir,
    production_runtime_artifact_json, project_production_diagnostics, project_resume_diagnostics,
    resume_manifest_json, runtime_component_artifact_json, runtime_computed_artifact_json,
    runtime_context_artifact_json, runtime_cost_report_json, runtime_effect_artifact_json,
    runtime_forms_artifact_json, runtime_opaque_artifact_json, runtime_resource_artifact_json,
    semantic_capability_matrix_text, semantic_capability_migration_text,
    semantic_capability_registry_json, semantic_graph_json, semantic_type_text, summarize_source,
    template_manifest_json, validate_application_publication_request_v1,
    validate_application_semantic_model, validate_runtime_opaque_artifact,
    validate_runtime_resource_artifact, ApplicationPublicationProfileV1,
    ApplicationPublicationRequestV1, ApplicationPublicationSourceV1, ApplicationSemanticModel,
    AsmValidationDiagnostic, AttributeValue, CompilationUnit, ComponentGraph, ConstantFoldingPass,
    DeclaredStateTypeKind, EffectInspection, EffectInspectionRegistry,
    ExecutableProgramFingerprint, ImmutableAsmPass, ProductionDiagnosticFact,
    ProductionDiagnosticKind, ProductionProjectedDiagnostic, ProductionReportInputs,
    ProductionRootChunkInput, RenderAttribute, RenderAttributeValue, SemanticEntity,
    SemanticEntityKind, SemanticId, SemanticOwner, SemanticPackageResolutionTable,
    SemanticPackageRuntimeModuleKey, SemanticPackageRuntimeModuleTable, SemanticReferenceKind,
    SerializableValue, SharedChunkCandidatePlan, SourceProvenance, StateOperation, TemplateChild,
    TemplateGraph, TemplateSemanticKind,
};
use presolve_parser::{
    parse_file, ParseDiagnostic, ParseSeverity, ParsedClass, ParsedFile, ParsedJsxAttribute,
    ParsedJsxAttributeValue, ParsedJsxChild, ParsedJsxNode, ParsedMethod, SourceSpan,
};
use serde::Serialize;
use sha2::{Digest as _, Sha256};

const ASM_INSPECTION_SCHEMA_VERSION: u32 = 12;
const CHECK_JSON_SCHEMA_VERSION: u32 = 6;
static NEXT_APPLICATION_PUBLICATION_STAGE: AtomicU64 = AtomicU64::new(1);

fn supports_asm_inspection_schema(version: u32) -> bool {
    version == ASM_INSPECTION_SCHEMA_VERSION
}

fn main() {
    let mut args = env::args().skip(1).collect::<Vec<_>>();

    if args.is_empty() {
        print_usage_and_exit();
    }

    let command = args.remove(0);

    match command.as_str() {
        "version" => run_l9_version(&args),
        "help" | "--help" | "-h" => print_l9_usage(),
        "create" | "benchmark" | "doctor" => l9_reserved_command(&command),
        "explain" => run_explain(args),
        "parse" => run_parse(args),
        "graph" => {
            if args.first().is_some_and(|value| value == "workspace") {
                run_l11_workspace_graph(&args[1..]);
            } else if args.first().is_some_and(|value| value == "artifact") {
                run_l11_artifact_graph(&args[1..]);
            } else {
                run_graph(args);
            }
        }
        "asm" => l9_command_error("asm", "retired: use presolve explain", 6),
        "check" => {
            if args.is_empty() {
                run_ergonomic_check(Path::new("."));
            } else if args.iter().any(|argument| argument == "--config") {
                run_l9_build_or_check("check", &args);
            } else {
                run_check(&args);
            }
        }
        "template" => run_template(args),
        "html" => run_html(args),
        "manifest" => run_manifest(args),
        "build" => {
            if args.is_empty() {
                run_ergonomic_build(Path::new("."), ApplicationPublicationProfileV1::Production);
            } else if args.iter().any(|argument| argument == "--config") {
                run_l9_build_or_check("build", &args);
            } else {
                run_build(args);
            }
        }
        "dev" => run_ergonomic_dev(&args),
        "application" => run_application_command(args),
        "route" => run_route_command(args),
        "cache" => run_l9_cache(&args),
        "clean" => run_l9_clean(&args),
        "workspace" => run_l9_workspace(&args),
        "watch" => run_l9_watch(&args),
        "inspect" => run_l11_inspect(&args),
        "trace" => run_l11_trace(&args),
        "profile" => run_l11_profile(&args),
        _ => {
            eprintln!("unknown command: {command}");
            print_usage_and_exit();
        }
    }
}

fn run_ergonomic_build(root: &Path, profile: ApplicationPublicationProfileV1) {
    let project = discover_project_v1(root)
        .unwrap_or_else(|error| application_cli_error(error.code, &error.message));
    let entry_path = PathBuf::from("app/routes/index.tsx");
    if !project
        .sources
        .iter()
        .any(|source| source.logical_path == entry_path)
    {
        application_cli_error(
            "PSDISC1005_DEFAULT_ENTRY_MISSING",
            "expected app/routes/index.tsx; use `presolve application build` for a non-default entry",
        );
    }
    let output_root = project.root.join("dist");
    validate_application_output_root(&output_root);
    let discovery_unit = CompilationUnit::parse_sources(
        project
            .sources
            .iter()
            .map(|source| (source.logical_path.clone(), source.source.as_str())),
    );
    let (package_contracts, package_runtime_modules) =
        discover_imported_package_tables(&project.root, &discovery_unit);
    let model = build_application_semantic_model_for_unit_with_packages(
        &discovery_unit,
        &package_contracts,
    );
    presolve_compiler::build_validated_file_route_graph_v1(&model)
        .unwrap_or_else(|error| application_cli_error(error.code, &error.message));
    let request = ApplicationPublicationRequestV1 {
        configuration: presolve_compiler::platform::WorkspaceConfiguration::default(),
        sources: project
            .sources
            .into_iter()
            .map(|source| ApplicationPublicationSourceV1 {
                logical_path: source.logical_path,
                source: source.source,
            })
            .collect(),
        entry_path,
        package_contracts,
        package_runtime_modules,
        profile,
        output_root: output_root.clone(),
    };
    let validated = validate_application_publication_request_v1(request)
        .unwrap_or_else(|error| application_cli_error(error.code, &error.message));
    let product = build_application_publication_product_v1(validated)
        .unwrap_or_else(|error| application_cli_error(error.code, &error.message));
    publish_application_product(&output_root, &product)
        .unwrap_or_else(|error| application_cli_error("PSAPP3008_PUBLICATION_FAILED", &error));
    println!("Built {}", output_root.display());
}

fn run_ergonomic_dev(args: &[String]) {
    let mut port = 3000_u16;
    let mut once = false;
    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--once" => {
                once = true;
                index += 1;
            }
            "--port" => {
                let Some(value) = args.get(index + 1) else {
                    application_cli_error("PSDEV1001_INVALID_ARGUMENT", "--port requires a number");
                };
                port = value.parse::<u16>().unwrap_or_else(|_| {
                    application_cli_error("PSDEV1001_INVALID_ARGUMENT", "--port must be a u16")
                });
                index += 2;
            }
            option => application_cli_error(
                "PSDEV1001_INVALID_ARGUMENT",
                &format!("unknown dev option `{option}`"),
            ),
        }
    }
    run_ergonomic_build(Path::new("."), ApplicationPublicationProfileV1::Development);
    if once {
        return;
    }
    serve_ergonomic_development_output(&Path::new(".").join("dist"), port);
}

fn serve_ergonomic_development_output(output_root: &Path, port: u16) -> ! {
    let listener = TcpListener::bind(("127.0.0.1", port)).unwrap_or_else(|error| {
        application_cli_error(
            "PSDEV1002_LISTEN_FAILED",
            &format!("failed to bind 127.0.0.1:{port}: {error}"),
        )
    });
    let address = listener
        .local_addr()
        .expect("bound listener has an address");
    println!("Presolve dev ready at http://{address}");
    for connection in listener.incoming() {
        match connection {
            Ok(stream) => serve_ergonomic_development_connection(stream, output_root),
            Err(error) => eprintln!("PSDEV1003_CONNECTION_FAILED: {error}"),
        }
    }
    unreachable!("a TcpListener incoming iterator never completes")
}

fn serve_ergonomic_development_connection(mut stream: TcpStream, output_root: &Path) {
    let mut request = [0_u8; 16 * 1024];
    let Ok(length) = stream.read(&mut request) else {
        return;
    };
    let request = String::from_utf8_lossy(&request[..length]);
    let Some(target) = request
        .lines()
        .next()
        .and_then(|line| line.split_whitespace().nth(1))
    else {
        write_development_response(
            &mut stream,
            "400 Bad Request",
            "text/plain",
            b"Bad request\n",
        );
        return;
    };
    let path = target.split('?').next().unwrap_or(target);
    let relative = if path == "/" {
        PathBuf::from("index.html")
    } else {
        PathBuf::from(path.trim_start_matches('/'))
    };
    if !is_safe_development_asset_path(&relative) {
        write_development_response(&mut stream, "404 Not Found", "text/plain", b"Not found\n");
        return;
    }
    match fs::read(output_root.join(&relative)) {
        Ok(bytes) => write_development_response(
            &mut stream,
            "200 OK",
            development_content_type(&relative),
            &bytes,
        ),
        Err(_) => {
            write_development_response(&mut stream, "404 Not Found", "text/plain", b"Not found\n")
        }
    }
}

fn is_safe_development_asset_path(path: &Path) -> bool {
    !path.as_os_str().is_empty()
        && path
            .components()
            .all(|component| matches!(component, Component::Normal(_)))
}

fn development_content_type(path: &Path) -> &'static str {
    match path.extension().and_then(|extension| extension.to_str()) {
        Some("html") => "text/html; charset=utf-8",
        Some("js") => "text/javascript; charset=utf-8",
        Some("json") => "application/json; charset=utf-8",
        Some("css") => "text/css; charset=utf-8",
        _ => "application/octet-stream",
    }
}

fn write_development_response(
    stream: &mut TcpStream,
    status: &str,
    content_type: &str,
    body: &[u8],
) {
    let _ = write!(
        stream,
        "HTTP/1.1 {status}\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        body.len()
    );
    let _ = stream.write_all(body);
}

fn run_ergonomic_check(root: &Path) {
    let project = discover_project_v1(root)
        .unwrap_or_else(|error| application_cli_error(error.code, &error.message));
    let unit = CompilationUnit::parse_sources(
        project
            .sources
            .iter()
            .map(|source| (source.logical_path.clone(), source.source.as_str())),
    );
    let (package_contracts, _) = discover_imported_package_tables(&project.root, &unit);
    let asm = build_application_semantic_model_for_unit_with_packages(&unit, &package_contracts);
    presolve_compiler::build_validated_file_route_graph_v1(&asm)
        .unwrap_or_else(|error| application_cli_error(error.code, &error.message));
    let diagnostics = asm
        .diagnostics
        .iter()
        .filter(|diagnostic| {
            diagnostic.code.starts_with("PSC") || diagnostic.code.starts_with("PSBIND")
        })
        .collect::<Vec<_>>();
    if diagnostics.is_empty() {
        println!("Checked {} source file(s)", unit.files().len());
        return;
    }
    for diagnostic in diagnostics {
        eprintln!("{}: {}", diagnostic.code, diagnostic.message);
    }
    process::exit(2);
}

fn discover_imported_package_tables(
    root: &Path,
    unit: &CompilationUnit,
) -> (
    SemanticPackageResolutionTable,
    SemanticPackageRuntimeModuleTable,
) {
    let package_specifiers = unit
        .files()
        .iter()
        .flat_map(|file| file.imports.iter().map(|import| import.source.as_str()))
        .filter(|source| !source.starts_with('.') && !source.starts_with('/'))
        .map(str::to_owned)
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    discover_semantic_packages_v1(root, &package_specifiers)
        .unwrap_or_else(|error| application_cli_error(error.code, &error.message))
}

fn run_l9_version(args: &[String]) {
    if args.iter().any(|argument| argument == "--format") {
        if args.len() != 2 || args[0] != "--format" || args[1] != "json" {
            l9_command_error("version", "only --format json is supported", 2);
        }
        println!(
            "{}",
            serde_json::json!({
                "schema": "presolve.cli-version",
                "version": 1,
                "presolve_version": env!("CARGO_PKG_VERSION"),
            })
        );
    } else if args.is_empty() {
        println!("presolve {}", env!("CARGO_PKG_VERSION"));
    } else {
        l9_command_error("version", "unknown option", 2);
    }
}

fn l9_reserved_command(command: &str) -> ! {
    l9_command_error(
        command,
        "reserved: no canonical L3-L8 product adapter is available for this command",
        6,
    )
}

fn print_l9_usage() -> ! {
    println!("presolve <command> [options]");
    println!("commands: version, build, check, clean, cache, workspace, watch, dev, create, explain, inspect, graph, trace, profile, benchmark, doctor");
    println!(
        "explicit project commands require --config <file> and --source <logical=relative-file>"
    );
    process::exit(0);
}

fn run_l9_build_or_check(command: &str, args: &[String]) {
    let mut configuration_path = None;
    let mut sources = Vec::new();
    let mut verify_clean_equivalence = false;
    let mut json = false;
    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--config" => {
                let Some(value) = args.get(index + 1) else {
                    l9_command_error(command, "missing value for --config", 2);
                };
                configuration_path = Some(PathBuf::from(value));
                index += 2;
            }
            "--source" => {
                let Some(value) = args.get(index + 1) else {
                    l9_command_error(command, "missing value for --source", 2);
                };
                let source = parse_explicit_source_spec_v1(value).unwrap_or_else(|error| {
                    l9_command_error(command, &error.to_string(), 2);
                });
                sources.push(source);
                index += 2;
            }
            "--verify-clean-equivalence" => {
                verify_clean_equivalence = true;
                index += 1;
            }
            "--format" => {
                let Some(value) = args.get(index + 1) else {
                    l9_command_error(command, "missing value for --format", 2);
                };
                match value.as_str() {
                    "human" => json = false,
                    "json" => json = true,
                    _ => l9_command_error(command, "--format must be human or json", 2),
                }
                index += 2;
            }
            value => l9_command_error(command, &format!("unknown option: {value}"), 2),
        }
    }
    let Some(configuration_path) = configuration_path else {
        l9_command_error(command, "--config is required", 2);
    };
    let result =
        run_explicit_build_or_check_v1(&configuration_path, &sources, verify_clean_equivalence)
            .unwrap_or_else(|error| {
                let exit_code = if error.code.starts_with("L9D") || error.code.starts_with("L9P") {
                    2
                } else {
                    4
                };
                l9_command_error(command, &error.to_string(), exit_code);
            });
    if json {
        println!(
            "{}",
            serde_json::json!({
                "schema": "presolve.cli-result",
                "version": 1,
                "command": command,
                "status": "succeeded",
                "exit_code": 0,
                "result": {
                    "workspace_id": result.workspace_id,
                    "commit_sequence": result.commit_sequence,
                    "snapshot_id": result.snapshot_id,
                    "graph_snapshot_id": result.graph_snapshot_id,
                    "mode": result.mode,
                },
                "diagnostics": [],
                "errors": [],
            })
        );
    } else {
        println!(
            "{command} succeeded: workspace={} snapshot={} mode={}",
            result.workspace_id, result.snapshot_id, result.mode
        );
    }
}

fn l9_command_error(command: &str, message: &str, exit_code: i32) -> ! {
    eprintln!("{command}: {message}");
    process::exit(exit_code);
}

struct L11ProductInput {
    schema: String,
    product: PathBuf,
    format: String,
}

fn l11_error(command: &str, code: &str, message: &str) -> ! {
    l9_command_error(command, &format!("{code}: {message}"), 6)
}

fn parse_l11_product_input(command: &str, args: &[String], allow_dot: bool) -> L11ProductInput {
    let mut schema = None;
    let mut product = None;
    let mut format = "human".to_owned();
    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--schema" => {
                let Some(value) = args.get(index + 1) else {
                    l11_error(command, "L11T002", "missing value for --schema");
                };
                schema = Some(value.clone());
                index += 2;
            }
            "--product" => {
                let Some(value) = args.get(index + 1) else {
                    l11_error(command, "L11T002", "missing value for --product");
                };
                product = Some(PathBuf::from(value));
                index += 2;
            }
            "--format" => {
                let Some(value) = args.get(index + 1) else {
                    l11_error(command, "L11T002", "missing value for --format");
                };
                if value != "human" && value != "json" && (!allow_dot || value != "dot") {
                    l11_error(
                        command,
                        "L11T006",
                        "unsupported format for this tooling view",
                    );
                }
                format.clone_from(value);
                index += 2;
            }
            value => l11_error(command, "L11T002", &format!("unknown option: {value}")),
        }
    }
    L11ProductInput {
        schema: schema.unwrap_or_else(|| l11_error(command, "L11T002", "--schema is required")),
        product: product.unwrap_or_else(|| l11_error(command, "L11T002", "--product is required")),
        format,
    }
}

fn read_l11_product(command: &str, input: &L11ProductInput) -> ToolingProductV1 {
    let bytes = fs::read(&input.product).unwrap_or_else(|error| {
        l11_error(
            command,
            "L11T002",
            &format!("failed to read product: {error}"),
        );
    });
    read_tooling_product_v1(&input.schema, &[1], &bytes)
        .unwrap_or_else(|error| l11_error(command, error.code, &error.message))
}

fn run_l11_inspect(args: &[String]) {
    let Some(view) = args.first() else {
        l11_error(
            "inspect",
            "L11T005",
            "a supported inspection view is required",
        );
    };
    let input = parse_l11_product_input("inspect", &args[1..], false);
    let product = read_l11_product("inspect", &input);
    match (view.as_str(), product) {
        ("workspace-snapshot", ToolingProductV1::WorkspaceSnapshot(snapshot)) => {
            if input.format == "json" {
                print!(
                    "{}",
                    String::from_utf8(snapshot.to_canonical_json().unwrap()).unwrap()
                );
            } else {
                println!(
                    "workspace snapshot: workspace={} snapshot={} units={}",
                    snapshot.workspace_id.as_str(),
                    snapshot.snapshot_id.as_str(),
                    snapshot.units.len()
                );
            }
        }
        ("workspace-graph", ToolingProductV1::WorkspaceGraph(graph)) => {
            if input.format == "json" {
                print!(
                    "{}",
                    String::from_utf8(graph.to_canonical_json().unwrap()).unwrap()
                );
            } else {
                println!(
                    "workspace graph: workspace={} snapshot={} units={} compile_edges={}",
                    graph.workspace_id.as_str(),
                    graph.snapshot_id.as_str(),
                    graph.units.len(),
                    graph.dependency_edges.len()
                );
            }
        }
        _ => l11_error(
            "inspect",
            "L11T006",
            "view does not match the validated product schema",
        ),
    }
}

fn run_l11_workspace_graph(args: &[String]) {
    let input = parse_l11_product_input("graph", args, true);
    let ToolingProductV1::WorkspaceGraph(graph) = read_l11_product("graph", &input) else {
        l11_error(
            "graph",
            "L11T006",
            "workspace graph view requires workspace-graph schema",
        );
    };
    match input.format.as_str() {
        "json" => print!(
            "{}",
            String::from_utf8(graph.to_canonical_json().unwrap()).unwrap()
        ),
        "dot" => {
            println!("digraph \"presolve.workspace-graph\" {{");
            for edge in &graph.dependency_edges {
                println!(
                    "  \"{}\" -> \"{}\";",
                    edge.source.as_str(),
                    edge.target.as_str()
                );
            }
            println!("}}");
        }
        _ => println!(
            "workspace graph: workspace={} snapshot={} units={} compile_edges={}",
            graph.workspace_id.as_str(),
            graph.snapshot_id.as_str(),
            graph.units.len(),
            graph.dependency_edges.len()
        ),
    }
}

fn run_l11_artifact_graph(args: &[String]) {
    let input = parse_l11_product_input("graph", args, true);
    let ToolingProductV1::ArtifactGraph(graph) = read_l11_product("graph", &input) else {
        l11_error(
            "graph",
            "L11T006",
            "artifact graph requires artifact-graph schema",
        );
    };
    match input.format.as_str() {
        "json" => print!(
            "{}",
            presolve_compiler::tooling_artifact_graph_json_v1(&graph)
        ),
        "dot" => {
            println!("digraph \"presolve.artifact-graph\" {{");
            for chunk in &graph.chunks {
                println!("  \"{}\";", chunk.chunk_id);
            }
            for edge in &graph.dependencies {
                println!(
                    "  \"{}\" -> \"{}\";",
                    edge.dependent_chunk_id, edge.dependency_chunk_id
                );
            }
            println!("}}");
        }
        _ => println!(
            "artifact graph: graph={} build={} chunks={} dependencies={} activations={}",
            graph.graph_id,
            graph.build_id,
            graph.chunks.len(),
            graph.dependencies.len(),
            graph.activations.len()
        ),
    }
}

fn run_l11_trace(args: &[String]) {
    let input = parse_l11_product_input("trace", args, false);
    let ToolingProductV1::BuildTrace(trace) = read_l11_product("trace", &input) else {
        l11_error(
            "trace",
            "L11T006",
            "trace requires a validated build-trace product",
        );
    };
    if input.format == "json" {
        print!("{}", presolve_compiler::tooling_build_trace_json_v1(&trace));
    } else {
        println!(
            "build trace: trace={} workspace={} outcome={:?} stages={}",
            trace.trace_id,
            trace.workspace_id,
            trace.outcome,
            trace.stages.len()
        );
    }
}

fn run_l11_profile(args: &[String]) {
    let input = parse_l11_product_input("profile", args, false);
    let ToolingProductV1::CompileCostReport(report) = read_l11_product("profile", &input) else {
        l11_error(
            "profile",
            "L11T006",
            "profile requires a validated compile-cost-report product",
        );
    };
    if input.format == "json" {
        print!(
            "{}",
            presolve_compiler::tooling_compile_cost_report_json_v1(&report)
        );
    } else {
        println!(
            "structural profile: report={} build={} production_bytes={} artifact_bytes={} static_operations={}",
            report.report_id,
            report.build_id,
            report.optimization_report.production_bytes,
            report.runtime_cost_report.production_artifact_bytes,
            report.runtime_cost_report.estimated_cold_init_operation_count
                + report.runtime_cost_report.estimated_resume_restore_operation_count
        );
    }
}

fn run_l9_cache(args: &[String]) {
    let (operation, start) = match args.first().map(String::as_str) {
        Some("inspect") => (CliCacheOperationV1::Inspect, 1),
        Some("verify") => (CliCacheOperationV1::Verify, 1),
        Some("clean") => (CliCacheOperationV1::Clean, 1),
        Some(value) if value.starts_with('-') => (CliCacheOperationV1::Inspect, 0),
        None => (CliCacheOperationV1::Inspect, 0),
        Some(value) => {
            l9_command_error("cache", &format!("unsupported cache operation: {value}"), 6)
        }
    };
    let (configuration_path, json) = l9_config_and_format("cache", &args[start..]);
    let result =
        run_project_cache_operation_v1(&configuration_path, operation).unwrap_or_else(|error| {
            let exit_code = if error.code.starts_with("L9P") || error.code.starts_with("L9E") {
                2
            } else {
                5
            };
            l9_command_error("cache", &error.to_string(), exit_code);
        });
    if json {
        if let Some(report) = result.report {
            print!("{}", String::from_utf8_lossy(&report.to_canonical_json()));
        } else {
            println!(
                "{}",
                serde_json::json!({
                    "schema": "presolve.cli-cache-clean",
                    "version": 1,
                    "removed_keys": result.removed_keys,
                })
            );
        }
    } else if let Some(report) = result.report {
        println!(
            "cache {}: valid_entries={} payload_bytes={} artifact_bytes={}",
            result.operation.as_str(),
            report.valid_keys.len(),
            report.total_payload_bytes,
            report.total_artifact_bytes
        );
    } else {
        println!("cache clean: removed_entries={}", result.removed_keys.len());
    }
}

fn run_l9_clean(args: &[String]) {
    let (configuration_path, json) = l9_config_and_format("clean", args);
    let result = run_project_cache_operation_v1(&configuration_path, CliCacheOperationV1::Clean)
        .unwrap_or_else(|error| {
            let exit_code = if error.code.starts_with("L9P") || error.code.starts_with("L9E") {
                2
            } else {
                5
            };
            l9_command_error("clean", &error.to_string(), exit_code);
        });
    if json {
        println!(
            "{}",
            serde_json::json!({
                "schema": "presolve.cli-cache-clean",
                "version": 1,
                "removed_keys": result.removed_keys,
            })
        );
    } else {
        println!(
            "clean succeeded: removed_cache_entries={}",
            result.removed_keys.len()
        );
    }
}

fn run_l9_workspace(args: &[String]) {
    let mut configuration_path = None;
    let mut sources = Vec::new();
    let mut verify_clean_equivalence = false;
    let mut json = false;
    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--config" => {
                let Some(value) = args.get(index + 1) else {
                    l9_command_error("workspace", "missing value for --config", 2);
                };
                configuration_path = Some(PathBuf::from(value));
                index += 2;
            }
            "--source" => {
                let Some(value) = args.get(index + 1) else {
                    l9_command_error("workspace", "missing value for --source", 2);
                };
                sources.push(
                    parse_explicit_source_spec_v1(value).unwrap_or_else(|error| {
                        l9_command_error("workspace", &error.to_string(), 2);
                    }),
                );
                index += 2;
            }
            "--verify-clean-equivalence" => {
                verify_clean_equivalence = true;
                index += 1;
            }
            "--format" => {
                let Some(value) = args.get(index + 1) else {
                    l9_command_error("workspace", "missing value for --format", 2);
                };
                match value.as_str() {
                    "human" => json = false,
                    "json" => json = true,
                    _ => l9_command_error("workspace", "--format must be human or json", 2),
                }
                index += 2;
            }
            value => l9_command_error("workspace", &format!("unknown option: {value}"), 2),
        }
    }
    let Some(configuration_path) = configuration_path else {
        l9_command_error("workspace", "--config is required", 2);
    };
    let result = run_explicit_workspace_v1(&configuration_path, &sources, verify_clean_equivalence)
        .unwrap_or_else(|error| {
            let exit_code = if error.code.starts_with("L9") {
                2
            } else if error.code.starts_with("L7") || error.code == "workspace_not_found" {
                3
            } else {
                4
            };
            l9_command_error("workspace", &error.to_string(), exit_code);
        });
    if json {
        println!(
            "{}",
            serde_json::json!({
                "schema": "presolve.cli-workspace-result",
                "version": 1,
                "workspace_id": result.workspace_id,
                "status": result.status,
                "manifest_identity": result.manifest_identity,
                "graph_identity": result.graph_identity,
                "plan_identity": result.plan_identity,
                "package_snapshot_id": result.package_snapshot_id,
            })
        );
    } else {
        println!(
            "workspace succeeded: workspace={} plan={}",
            result.workspace_id, result.plan_identity
        );
    }
}

fn run_l9_watch(args: &[String]) {
    let mut configuration_path = None;
    let mut sources = Vec::new();
    let mut json = false;
    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--once" => index += 1,
            "--config" => {
                let Some(value) = args.get(index + 1) else {
                    l9_command_error("watch", "missing value for --config", 2);
                };
                configuration_path = Some(PathBuf::from(value));
                index += 2;
            }
            "--source" => {
                let Some(value) = args.get(index + 1) else {
                    l9_command_error("watch", "missing value for --source", 2);
                };
                sources.push(
                    parse_explicit_source_spec_v1(value)
                        .unwrap_or_else(|error| l9_command_error("watch", &error.to_string(), 2)),
                );
                index += 2;
            }
            "--format" => {
                let Some(value) = args.get(index + 1) else {
                    l9_command_error("watch", "missing value for --format", 2);
                };
                json = match value.as_str() {
                    "human" => false,
                    "json" => true,
                    _ => l9_command_error("watch", "--format must be human or json", 2),
                };
                index += 2;
            }
            value => l9_command_error("watch", &format!("unknown option: {value}"), 2),
        }
    }
    let Some(configuration_path) = configuration_path else {
        l9_command_error("watch", "--config is required", 2);
    };
    let outcome =
        run_explicit_watch_once_v1(&configuration_path, &sources).unwrap_or_else(|error| {
            l9_command_error(
                "watch",
                &error.to_string(),
                if error.code.starts_with("L9") { 2 } else { 3 },
            )
        });
    if json {
        println!(
            "{}",
            serde_json::json!({"schema":"presolve.cli-watch-once","version":1,"outcome":outcome})
        );
    } else {
        println!("watch once succeeded: outcome={outcome}");
    }
}

fn l9_config_and_format(command: &str, args: &[String]) -> (PathBuf, bool) {
    let mut configuration_path = None;
    let mut json = false;
    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--config" => {
                let Some(value) = args.get(index + 1) else {
                    l9_command_error(command, "missing value for --config", 2);
                };
                configuration_path = Some(PathBuf::from(value));
                index += 2;
            }
            "--format" => {
                let Some(value) = args.get(index + 1) else {
                    l9_command_error(command, "missing value for --format", 2);
                };
                match value.as_str() {
                    "human" => json = false,
                    "json" => json = true,
                    _ => l9_command_error(command, "--format must be human or json", 2),
                }
                index += 2;
            }
            value => l9_command_error(command, &format!("unknown option: {value}"), 2),
        }
    }
    let Some(configuration_path) = configuration_path else {
        l9_command_error(command, "--config is required", 2);
    };
    (configuration_path, json)
}

fn run_explain(mut args: Vec<String>) {
    if args
        .first()
        .is_some_and(|argument| argument == "--capabilities")
    {
        args.remove(0);
        let format = parse_format(&args);
        match format.as_str() {
            "json" => print!("{}", semantic_capability_registry_json()),
            "human" => print!("{}", semantic_capability_matrix_text()),
            "migration" => print!("{}", semantic_capability_migration_text()),
            _ => {
                eprintln!("semantic capability inspection supports only --format human, json, or migration");
                process::exit(1);
            }
        }
        return;
    }

    let semantic_inspection = args
        .iter()
        .any(|argument| argument == "--inspect" || is_asm_entity_inspection_option(argument))
        || args
            .windows(2)
            .any(|pair| pair[0] == "--format" && pair[1] == "graph");
    if semantic_inspection {
        args.retain(|argument| argument != "--inspect");
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
    let resume_diagnostics = if unit.files().iter().any(|file| !file.diagnostics.is_empty()) {
        Vec::new()
    } else {
        project_resume_diagnostics(&asm)
    };
    let mut validation = validate_application_semantic_model(&asm);
    validation.extend(resume_diagnostics.iter().cloned().map(|diagnostic| {
        AsmValidationDiagnostic {
            code: diagnostic.code.to_string(),
            message: diagnostic.message,
        }
    }));

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
            print_asm_entity_text(
                &asm,
                entity,
                &asm.diagnostics,
                &resume_diagnostics,
                inputs.filters,
            );
        }
        ("json", Some(entity)) => print!(
            "{}",
            asm_entity_inspection_json(
                &asm,
                entity,
                &asm.diagnostics,
                &resume_diagnostics,
                inputs.filters,
            )
        ),
        ("text", None) => print_asm_text(&paths, &asm, &validation),
        ("json", None) => print!(
            "{}",
            asm_inspection_json(&paths, &asm, &validation, &resume_diagnostics)
        ),
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
    let resume_diagnostics = if unit.files().iter().any(|file| !file.diagnostics.is_empty()) {
        Vec::new()
    } else {
        project_resume_diagnostics(&asm)
    };
    let mut validation = validate_application_semantic_model(&asm);
    validation.extend(resume_diagnostics.iter().cloned().map(|diagnostic| {
        AsmValidationDiagnostic {
            code: diagnostic.code.to_string(),
            message: diagnostic.message,
        }
    }));
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
            check_json(
                &unit,
                &asm,
                &validation,
                &resume_diagnostics,
                &categories,
                &fail_on,
            )
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
        print_production_diagnostics_text(&asm_production_diagnostics(asm));
    }
}

fn check_json(
    unit: &CompilationUnit,
    asm: &ApplicationSemanticModel,
    validation: &[AsmValidationDiagnostic],
    resume_diagnostics: &[presolve_compiler::ResumeProjectedDiagnostic],
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
    let resume_diagnostics = if check_category_enabled(categories, "validation") {
        asm_resume_diagnostics(resume_diagnostics)
    } else {
        Vec::new()
    };
    let production_diagnostics = if check_category_enabled(categories, "validation") {
        asm_production_diagnostics(asm)
    } else {
        Vec::new()
    };
    serde_json::to_string_pretty(&serde_json::json!({"schema_version": CHECK_JSON_SCHEMA_VERSION, "files": unit.files().iter().map(|file| file.path.display().to_string()).collect::<Vec<_>>(), "summary": {"parser_diagnostics": parser_count, "compiler_diagnostics": asm.diagnostics.len(), "validation": validation.len()}, "categories": categories, "fail_on": diagnostic_severity_label(fail_on), "parser_diagnostics": parser_diagnostics, "compiler_diagnostics": compiler_diagnostics, "validation": validation_diagnostics, "resume_diagnostics": resume_diagnostics, "production_diagnostics": production_diagnostics})).expect("check document should serialize") + "\n"
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

fn compiler_diagnostic_json(
    diagnostic: &presolve_compiler::ComponentDiagnostic,
) -> serde_json::Value {
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
    let production = asm_production_inspection(asm);
    println!(
        "  production optimization: {}",
        if production["status"] == "available" {
            "available"
        } else {
            "blocked"
        }
    );
    print_production_diagnostics_text(&asm_production_diagnostics(asm));

    print_compiler_diagnostics(&asm.diagnostics);

    if let Some(validation_text) = asm_validation_diagnostics_text(validation) {
        print!("{validation_text}");
    }
}

fn print_compiler_diagnostics(diagnostics: &[presolve_compiler::ComponentDiagnostic]) {
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
    diagnostic: &presolve_compiler::ComponentDiagnostic,
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
    resume_diagnostics: &[presolve_compiler::ResumeProjectedDiagnostic],
) -> String {
    debug_assert!(supports_asm_inspection_schema(
        ASM_INSPECTION_SCHEMA_VERSION
    ));
    let computed_functions = computed_evaluation_functions(asm);
    let effect_inspections = build_effect_inspection_registry(asm);
    let context_inspections = build_context_inspection_registry(asm);
    let form_inspections = build_form_inspection_registry(asm);
    let resume = if asm.diagnostics.is_empty() {
        serde_json::from_str(&resume_manifest_json(&build_resume_manifest(asm)))
            .expect("resume manifest inspection JSON should parse")
    } else {
        serde_json::json!({
            "resumeFailures": asm.diagnostics.iter().map(|diagnostic| &diagnostic.code).collect::<Vec<_>>()
        })
    };
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
                    &form_inspections,
                )
            })
            .collect(),
        references,
        diagnostics,
        validation,
        resume_diagnostics: asm_resume_diagnostics(resume_diagnostics),
        production_diagnostics: asm_production_diagnostics(asm),
        resume,
        production: asm_production_inspection(asm),
    };

    serde_json::to_string_pretty(&document).expect("ASM inspection document should serialize")
        + "\n"
}

#[allow(clippy::too_many_lines)]
fn asm_production_inspection(asm: &ApplicationSemanticModel) -> serde_json::Value {
    if !asm.diagnostics.is_empty() {
        return serde_json::json!({
            "status": "blocked",
            "policy": {"id": "optimization-policy:production-v1", "version": 1},
            "blocks": asm.diagnostics.iter().map(|diagnostic| serde_json::json!({
                "code": diagnostic.code,
                "reason": diagnostic.message,
            })).collect::<Vec<_>>(),
            "production_ids": [],
        });
    }

    let ir = lower_components_to_ir(asm);
    let component = build_runtime_component_artifact(asm, &asm.component_ir_optimization);
    let computed = build_runtime_computed_artifact(asm, &ir);
    let context = build_runtime_context_artifact(asm, &optimize_context_ir(&ir));
    let effect = build_runtime_effect_artifact(asm, &optimize_effect_ir(&ir).output);
    let forms = build_runtime_forms_artifact(asm);
    let resume = build_resume_manifest(asm);
    let reachability = build_production_reachability_graph(
        &resume, &component, &computed, &context, &effect, &forms,
    );
    let candidates = SharedChunkCandidatePlan {
        candidates: Vec::new(),
        rejections: Vec::new(),
    };
    let (graph, extraction) =
        extract_production_chunk_graph(&candidates, &production_root_chunk_inputs(&resume))
            .expect("valid immutable resume products form a production graph");
    let artifact = build_production_runtime_artifact(&resume, &graph)
        .expect("valid immutable production graph packs");
    let artifact_json = production_runtime_artifact_json(&artifact);
    let layout = emit_production_modules(&graph);
    let modules = std::iter::once(&layout.eager)
        .chain(layout.shared.iter())
        .chain(layout.roots.iter())
        .map(|module| {
            serde_json::json!({
                "filename": module.filename,
                "byte_count": module.source.len(),
                "exports": module.exports,
            })
        })
        .collect::<Vec<_>>();
    let runtime_record_count = artifact
        .tables
        .tables
        .iter()
        .map(|table| table.mappings.len())
        .sum::<usize>()
        + graph.chunks.len();
    let static_operation_count = resume
        .capture_programs
        .iter()
        .map(|program| program.instructions.len())
        .sum::<usize>()
        + resume
            .restore_programs
            .iter()
            .map(|program| program.instructions.len())
            .sum::<usize>();

    serde_json::json!({
        "status": "available",
        "policy": {"id": artifact.optimization_policy, "version": 1},
        "reachability": {
            "roots": reachability.roots.iter().map(|root| serde_json::json!({
                "subject_id": root.subject_id,
                "reason": format!("{:?}", root.reason),
            })).collect::<Vec<_>>(),
            "edges": reachability.edges.iter().map(|edge| serde_json::json!({
                "from": edge.from,
                "to": edge.to,
                "reason": format!("{:?}", edge.reason),
            })).collect::<Vec<_>>(),
            "unreachable_records": reachability.unreachable.iter().map(|record| serde_json::json!({
                "subject_id": record.subject_id,
                "reason": record.reason,
            })).collect::<Vec<_>>(),
        },
        "programs": {
            "fingerprints": graph.chunks.iter().flat_map(|chunk| chunk.programs.iter()).map(ToString::to_string).collect::<Vec<_>>(),
            "aliases": [],
        },
        "constant_pool": {"entries": [], "consumers": []},
        "shared_candidates": {"calculations": [], "rejections": []},
        "chunk_graph": {
            "eager_chunk_id": graph.eager_chunk_id,
            "chunks": graph.chunks.iter().map(|chunk| serde_json::json!({
                "id": chunk.id,
                "kind": format!("{:?}", chunk.kind),
                "activation_roots": chunk.activation_roots,
                "root_kind": chunk.root_kind,
                "program_fingerprints": chunk.programs.iter().map(ToString::to_string).collect::<Vec<_>>(),
                "registration_only": chunk.registration_only,
                "module_filename": chunk.provisional_module_filename,
            })).collect::<Vec<_>>(),
            "dependencies": graph.dependencies.iter().map(|dependency| serde_json::json!({
                "dependent_chunk_id": dependency.dependent_chunk_id,
                "dependency_chunk_id": dependency.dependency_chunk_id,
            })).collect::<Vec<_>>(),
            "activation_plans": graph.activation_plans.iter().map(|plan| serde_json::json!({
                "activation_root_id": plan.activation_root_id,
                "root_chunk_id": plan.root_chunk_id,
                "shared_chunk_ids": plan.shared_chunk_ids,
            })).collect::<Vec<_>>(),
        },
        "runtime_tables": artifact.tables,
        "artifact_identity": {
            "build_id": artifact.build_id,
            "runtime_protocol_version": artifact.runtime_protocol_version,
            "artifact_checksum": artifact.integrity.artifact_checksum,
        },
        "cleanup_closures": {
            "ordering": "reverse-initialization",
            "owned_kinds": ["activation-dispatch", "event-binding-index", "form-index", "effect-subscription", "context-binding", "slot-structural-registry", "computed-cache", "state-storage", "form-storage", "context-provider", "resume-boundary-anchor", "component-instance", "dom-anchor"],
        },
        "validation_phases": ["V0","V1","V2","V3","V4","V5","V6","V7","V8","V9","V10"],
        "size_and_static_cost": {
            "production_artifact_bytes": artifact_json.len(),
            "production_executable_bytes": modules.iter().map(|module| module["byte_count"].as_u64().unwrap_or(0)).sum::<u64>(),
            "runtime_table_count": artifact.tables.tables.len(),
            "runtime_record_count": runtime_record_count,
            "static_operation_count": static_operation_count,
            "extracted_program_count": extraction.extracted_program_count,
            "root_chunk_count": extraction.root_chunk_count,
            "shared_chunk_count": extraction.shared_chunk_count,
        },
        "modules": modules,
        "blocks": reachability.blocks.iter().map(|block| serde_json::json!({
            "subject_id": block.subject_id,
            "reason": block.reason,
        })).collect::<Vec<_>>(),
        "exclusions": ["cryptographic-signing", "wall-clock-timing"],
    })
}

fn asm_production_diagnostics(
    asm: &ApplicationSemanticModel,
) -> Vec<ProductionProjectedDiagnostic> {
    project_production_diagnostics(&asm_production_diagnostic_facts(asm))
}

fn asm_production_diagnostic_facts(
    asm: &ApplicationSemanticModel,
) -> Vec<ProductionDiagnosticFact> {
    asm.diagnostics
        .iter()
        .map(|diagnostic| {
            let primary_identity = diagnostic
                .effect_id
                .as_ref()
                .map(ToString::to_string)
                .or_else(|| diagnostic.statement_id.as_ref().map(ToString::to_string))
                .or_else(|| {
                    diagnostic
                        .context_declaration_candidate_id
                        .as_ref()
                        .map(ToString::to_string)
                })
                .or_else(|| diagnostic.context_id.as_ref().map(ToString::to_string))
                .or_else(|| diagnostic.provider_id.as_ref().map(ToString::to_string))
                .or_else(|| diagnostic.consumer_id.as_ref().map(ToString::to_string))
                .or_else(|| {
                    diagnostic_component_identities(diagnostic)
                        .into_iter()
                        .next()
                        .map(|(_, identity)| identity)
                });
            ProductionDiagnosticFact {
                kind: ProductionDiagnosticKind::InvalidOptimizationRoot,
                actionable: true,
                primary_identity,
                primary_provenance: diagnostic.provenance.clone(),
                secondary_evidence: diagnostic
                    .secondary_labels
                    .iter()
                    .map(|label| label.message.clone())
                    .collect(),
            }
        })
        .collect()
}

fn print_production_diagnostics_text(diagnostics: &[ProductionProjectedDiagnostic]) {
    if diagnostics.is_empty() {
        return;
    }
    println!("  production diagnostics: {}", diagnostics.len());
    for diagnostic in diagnostics {
        println!(
            "    {} {}: {}",
            diagnostic.code, diagnostic.name, diagnostic.message
        );
        if let Some(identity) = &diagnostic.primary_identity {
            println!("      = subject: {identity}");
        }
        if let Some(provenance) = &diagnostic.primary_provenance {
            println!(
                "      at {}:{}:{} span={}..{}",
                provenance.path,
                provenance.line,
                provenance.column,
                provenance.start,
                provenance.end
            );
        }
        for evidence in &diagnostic.secondary_evidence {
            println!("      = evidence: {evidence}");
        }
    }
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
    diagnostics: &[presolve_compiler::ComponentDiagnostic],
    resume_diagnostics: &[presolve_compiler::ResumeProjectedDiagnostic],
    filters: AsmEntityFilters,
) {
    let computed_functions = computed_evaluation_functions(asm);
    let effect_inspections = build_effect_inspection_registry(asm);
    let form_inspections = build_form_inspection_registry(asm);
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
    if let Some(form) = form_inspections.records.get(id) {
        println!("  Form:");
        println!("    role: {}", form.role);
        println!("    form: {}", form.form);
        println!("    fields: {:?}", form.field_order);
        println!("    bindings: {:?}", form.bindings);
        println!("    rules: {:?}", form.validation_rules);
        println!(
            "    instances: {:?}",
            form.instances
                .iter()
                .map(|instance| &instance.form_instance)
                .collect::<Vec<_>>()
        );
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
    print_resume_diagnostics_text(&related_resume_diagnostics(resume_diagnostics, provenance));
    let production = asm_production_diagnostics(asm)
        .into_iter()
        .filter(|diagnostic| {
            diagnostic.primary_identity.as_deref() == Some(id.as_str())
                || diagnostic
                    .primary_provenance
                    .as_ref()
                    .is_some_and(|primary| {
                        primary.path == provenance.path.to_string_lossy()
                            && primary.start == provenance.span.start
                            && primary.end == provenance.span.end
                    })
        })
        .collect::<Vec<_>>();
    print_production_diagnostics_text(&production);
}

fn print_entity_references(label: &str, references: Vec<&presolve_compiler::SemanticReference>) {
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
    diagnostics: &[presolve_compiler::ComponentDiagnostic],
    id: &SemanticId,
    provenance: &SourceProvenance,
) {
    let diagnostics = related_entity_diagnostics(diagnostics, id, provenance);
    println!("  diagnostics: {}", diagnostics.len());
    for diagnostic in diagnostics {
        println!("    {}: {}", diagnostic.code, diagnostic.message);
    }
}

fn related_resume_diagnostics<'a>(
    diagnostics: &'a [presolve_compiler::ResumeProjectedDiagnostic],
    provenance: &SourceProvenance,
) -> Vec<&'a presolve_compiler::ResumeProjectedDiagnostic> {
    diagnostics
        .iter()
        .filter(|diagnostic| {
            diagnostic.primary_provenance.path == provenance.path
                && diagnostic.primary_provenance.span.start < provenance.span.end
                && provenance.span.start < diagnostic.primary_provenance.span.end
        })
        .collect()
}

fn print_resume_diagnostics_text(diagnostics: &[&presolve_compiler::ResumeProjectedDiagnostic]) {
    if diagnostics.is_empty() {
        return;
    }
    println!("  resumability diagnostics: {}", diagnostics.len());
    for diagnostic in diagnostics {
        println!("    {}: {}", diagnostic.code, diagnostic.message);
    }
}

fn asm_resume_diagnostics(
    diagnostics: &[presolve_compiler::ResumeProjectedDiagnostic],
) -> Vec<AsmResumeDiagnostic<'_>> {
    diagnostics
        .iter()
        .map(|diagnostic| AsmResumeDiagnostic {
            code: diagnostic.code,
            message: &diagnostic.message,
            primary_identity: diagnostic.primary_identity.as_deref(),
            primary_provenance: (&diagnostic.primary_provenance).into(),
        })
        .collect()
}

fn asm_resume_diagnostics_from_refs<'a>(
    diagnostics: &[&'a presolve_compiler::ResumeProjectedDiagnostic],
) -> Vec<AsmResumeDiagnostic<'a>> {
    diagnostics
        .iter()
        .map(|diagnostic| AsmResumeDiagnostic {
            code: diagnostic.code,
            message: &diagnostic.message,
            primary_identity: diagnostic.primary_identity.as_deref(),
            primary_provenance: (&diagnostic.primary_provenance).into(),
        })
        .collect()
}

fn asm_entity_inspection_json(
    asm: &ApplicationSemanticModel,
    id: &SemanticId,
    diagnostics: &[presolve_compiler::ComponentDiagnostic],
    resume_diagnostics: &[presolve_compiler::ResumeProjectedDiagnostic],
    filters: AsmEntityFilters,
) -> String {
    debug_assert!(supports_asm_inspection_schema(
        ASM_INSPECTION_SCHEMA_VERSION
    ));
    let computed_functions = computed_evaluation_functions(asm);
    let effect_inspections = build_effect_inspection_registry(asm);
    let context_inspections = build_context_inspection_registry(asm);
    let form_inspections = build_form_inspection_registry(asm);
    let provenance = asm.provenance(id).expect("ASM entities have provenance");
    let document = AsmEntityInspectionDocument {
        schema_version: ASM_INSPECTION_SCHEMA_VERSION,
        entity: asm_inspection_entity(
            asm,
            id,
            &computed_functions,
            &effect_inspections,
            &context_inspections,
            &form_inspections,
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
        resume_diagnostics: asm_resume_diagnostics_from_refs(&related_resume_diagnostics(
            resume_diagnostics,
            provenance,
        )),
        production_diagnostics: asm_production_diagnostics(asm),
        production: asm_production_inspection(asm),
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
            SemanticEntity::Slot(_)
                | SemanticEntity::ComponentInvocation(_)
                | SemanticEntity::ComponentInstance(_)
                | SemanticEntity::BlockedComponentInstance(_)
                | SemanticEntity::SlotContentFragment(_)
                | SemanticEntity::SlotOutlet(_)
        )
    )
}

fn filtered_entity_references(
    references: Vec<&presolve_compiler::SemanticReference>,
    filters: AsmEntityFilters,
) -> Vec<&presolve_compiler::SemanticReference> {
    let mut references = references
        .into_iter()
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
    diagnostics: &'a [presolve_compiler::ComponentDiagnostic],
    id: &SemanticId,
    provenance: &SourceProvenance,
) -> Vec<&'a presolve_compiler::ComponentDiagnostic> {
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
        "computed-resource" => SemanticReferenceKind::ComputedResource,
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
    owner.entity_id().map(presolve_compiler::SemanticId::as_str)
}

fn semantic_reference_kind(kind: SemanticReferenceKind) -> &'static str {
    match kind {
        SemanticReferenceKind::ActionState => "action-state",
        SemanticReferenceKind::ComputedState => "computed-state",
        SemanticReferenceKind::ComputedComputed => "computed-computed",
        SemanticReferenceKind::ComputedResource => "computed-resource",
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
            presolve_compiler::SemanticTypeStatus::Declared => "declared",
            presolve_compiler::SemanticTypeStatus::Inferred => "inferred",
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
    context_inspections: &presolve_compiler::ContextInspectionRegistry,
    form_inspections: &presolve_compiler::FormInspectionRegistry,
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
        form: form_inspections.records.get(id).cloned(),
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
            presolve_compiler::ComputedPurity::Unclassified => "unclassified",
            presolve_compiler::ComputedPurity::Pure => "pure",
            presolve_compiler::ComputedPurity::Impure => "impure",
        },
        serializability: match computed_type.serialization {
            presolve_compiler::SerializationCompatibility::Serializable => "serializable",
            presolve_compiler::SerializationCompatibility::NotSerializable => "not-serializable",
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
    resume_diagnostics: Vec<AsmResumeDiagnostic<'a>>,
    production_diagnostics: Vec<ProductionProjectedDiagnostic>,
    resume: serde_json::Value,
    production: serde_json::Value,
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
    resume_diagnostics: Vec<AsmResumeDiagnostic<'a>>,
    production_diagnostics: Vec<ProductionProjectedDiagnostic>,
    production: serde_json::Value,
}

#[derive(Serialize)]
struct AsmResumeDiagnostic<'a> {
    code: &'static str,
    message: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    primary_identity: Option<&'a str>,
    primary_provenance: AsmInspectionProvenance,
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
    context: Option<presolve_compiler::ContextInspection>,
    #[serde(skip_serializing_if = "Option::is_none")]
    form: Option<presolve_compiler::FormInspection>,
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

impl<'a> From<&'a presolve_compiler::SemanticReference> for AsmInspectionReference<'a> {
    fn from(reference: &'a presolve_compiler::SemanticReference) -> Self {
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

impl<'a> From<&'a presolve_compiler::ComponentDiagnostic> for AsmInspectionDiagnostic<'a> {
    fn from(diagnostic: &'a presolve_compiler::ComponentDiagnostic) -> Self {
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
                .map(presolve_compiler::EffectId::as_str),
            statement_id: diagnostic
                .statement_id
                .as_ref()
                .map(presolve_compiler::EffectStatementId::as_str),
            context_declaration_candidate_id: diagnostic
                .context_declaration_candidate_id
                .as_ref()
                .map(presolve_compiler::ContextDeclarationCandidateId::as_str),
            context_id: diagnostic
                .context_id
                .as_ref()
                .map(presolve_compiler::ContextId::as_str),
            provider_id: diagnostic
                .provider_id
                .as_ref()
                .map(presolve_compiler::ProviderId::as_str),
            consumer_id: diagnostic
                .consumer_id
                .as_ref()
                .map(presolve_compiler::ConsumerId::as_str),
            slot_id: diagnostic
                .slot_id
                .as_ref()
                .map(presolve_compiler::SlotId::as_str),
            invocation_id: diagnostic
                .invocation_id
                .as_ref()
                .map(presolve_compiler::ComponentInvocationId::as_str),
            component_instance_id: diagnostic
                .component_instance_id
                .as_ref()
                .map(presolve_compiler::ComponentInstanceId::as_str),
            slot_binding_id: diagnostic
                .slot_binding_id
                .as_ref()
                .map(presolve_compiler::SlotBindingId::as_str),
            structural_region_id: diagnostic
                .structural_region_id
                .as_ref()
                .map(presolve_compiler::ComponentStructuralRegionId::as_str),
            component_id: diagnostic
                .component_id
                .as_ref()
                .map(presolve_compiler::SemanticId::as_str),
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

impl From<&presolve_compiler::DiagnosticSecondaryLabel> for AsmInspectionSecondaryLabel {
    fn from(label: &presolve_compiler::DiagnosticSecondaryLabel) -> Self {
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
    fn with_span(provenance: &SourceProvenance, span: presolve_parser::SourceSpan) -> Self {
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

#[allow(clippy::too_many_lines)]
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
    let packages = parse_semantic_package_contracts(&args);
    let resource_runtime_modules = parse_semantic_package_runtime_modules(&args, &packages);
    let asm = ConstantFoldingPass.transform(
        &build_application_semantic_model_for_unit_with_packages(&unit, &packages),
    );
    let package_diagnostics = asm
        .diagnostics
        .iter()
        .filter(|diagnostic| {
            diagnostic.code.starts_with("PSBIND10")
                || matches!(diagnostic.code.as_str(), "PSC1128" | "PSC1130" | "PSC1131")
        })
        .collect::<Vec<_>>();
    if !package_diagnostics.is_empty() {
        for diagnostic in package_diagnostics {
            eprintln!("{}: {}", diagnostic.code, diagnostic.message);
        }
        process::exit(2);
    }
    let resource_runtime_artifact = if asm.resource_declarations.is_empty() {
        None
    } else {
        let artifact =
            build_runtime_resource_artifact_with_modules(&asm, &resource_runtime_modules)
                .unwrap_or_else(|error| {
                    eprintln!(
                        "PSRES1001: resource runtime module mapping is incomplete: {error:?}"
                    );
                    process::exit(2);
                });
        let validation = validate_runtime_resource_artifact(&artifact);
        if !validation.is_empty() {
            eprintln!("PSRES1002: generated resource artifact is invalid: {validation:?}");
            process::exit(2);
        }
        if let Some(declaration) = artifact
            .declarations
            .iter()
            .find(|declaration| declaration.execution_boundary == "Server")
        {
            eprintln!(
                "PSRES1003: resource `{}` uses a server endpoint and cannot be published in a browser build",
                declaration.id
            );
            process::exit(2);
        }
        Some(artifact)
    };
    let opaque_runtime_artifact = if asm.opaque_action_resolutions.is_empty() {
        None
    } else {
        let artifact = build_runtime_opaque_artifact_with_modules(&asm, &resource_runtime_modules)
            .unwrap_or_else(|error| {
                eprintln!("PSOPA1001: opaque runtime module mapping is incomplete: {error:?}");
                process::exit(2);
            });
        let validation = validate_runtime_opaque_artifact(&artifact);
        if !validation.is_empty() {
            eprintln!("PSOPA1002: generated opaque artifact is invalid: {validation:?}");
            process::exit(2);
        }
        Some(artifact)
    };
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
    let resume_runtime_artifact = build_resume_manifest(&asm);
    let resume_runtime_json = resume_manifest_json(&resume_runtime_artifact);
    let resource_runtime_json = resource_runtime_artifact
        .as_ref()
        .map(runtime_resource_artifact_json);
    let opaque_runtime_json = opaque_runtime_artifact
        .as_ref()
        .map(runtime_opaque_artifact_json);
    let (production_chunk_graph, _) = extract_production_chunk_graph(
        &SharedChunkCandidatePlan {
            candidates: Vec::new(),
            rejections: Vec::new(),
        },
        &production_root_chunk_inputs(&resume_runtime_artifact),
    )
    .expect("frozen resume manifest should form a production root graph");
    let production_runtime_artifact =
        build_production_runtime_artifact(&resume_runtime_artifact, &production_chunk_graph)
            .expect("validated production root graph should pack");
    let production_runtime_json = production_runtime_artifact_json(&production_runtime_artifact);
    let resume_chunks = build_resume_chunk_graph(&asm);
    let component_graph = fold_component_graph(&build_component_graph(&parsed));
    let template_graph = build_template_graph(&component_graph);
    let html_fragment = generate_ordinary_instance_html(&asm);
    let manifest = build_template_manifest_from_asm(&asm);
    let manifest_json = template_manifest_json(&manifest);
    let page_title = page_title_from_graph(&template_graph);
    let page_html = generate_standalone_page_with_resume_runtime(
        &page_title,
        &html_fragment,
        &manifest,
        &computed_runtime_artifact,
        &context_runtime_artifact,
        &effect_runtime_artifact,
        &component_runtime_artifact,
        &forms_runtime_artifact,
        &resume_runtime_artifact,
    );
    let page_html = resource_runtime_artifact
        .as_ref()
        .map_or(page_html, |resources| {
            generate_standalone_page_with_resume_runtime_and_resources(
                &page_title,
                &html_fragment,
                &manifest,
                &computed_runtime_artifact,
                &context_runtime_artifact,
                &effect_runtime_artifact,
                &component_runtime_artifact,
                &forms_runtime_artifact,
                &resume_runtime_artifact,
                resources,
            )
        });
    let page_html = if let Some(opaque) = &opaque_runtime_artifact {
        embed_opaque_runtime_artifact(page_html, opaque)
    } else {
        page_html
    };
    let page_html = production_mode_page_html(
        page_html,
        args.iter().any(|argument| argument == "--production"),
        &production_runtime_json,
    );
    let runtime_js = generate_runtime_stub();
    let production_layout = emit_production_modules(&production_chunk_graph);
    let development_bytes = byte_count([
        &page_html,
        &manifest_json,
        &computed_runtime_json,
        &context_runtime_json,
        &effect_runtime_json,
        &component_runtime_json,
        &forms_runtime_json,
        &resume_runtime_json,
        &runtime_js,
    ]) + resource_runtime_json.as_ref().map_or(0, |json| {
        u64::try_from(json.len()).expect("resource artifact byte count exceeds u64")
    }) + opaque_runtime_json.as_ref().map_or(0, |json| {
        u64::try_from(json.len()).expect("opaque artifact byte count exceeds u64")
    });
    let production_bytes = byte_count(
        std::iter::once(&production_runtime_json).chain(
            std::iter::once(&production_layout.eager)
                .chain(production_layout.shared.iter())
                .chain(production_layout.roots.iter())
                .map(|module| &module.source),
        ),
    );
    let report_inputs = ProductionReportInputs {
        dead_products_removed: 0,
        constants_pooled: 0,
        programs_deduplicated: 0,
        shared_candidates_rejected: 0,
        binding_writes_coalesced: 0,
        development_bytes,
        production_bytes,
        cold_init_operation_count: report_count(
            resume_runtime_artifact
                .capture_programs
                .iter()
                .map(|program| program.instructions.len())
                .sum(),
        ),
        resume_restore_operation_count: report_count(
            resume_runtime_artifact
                .restore_programs
                .iter()
                .map(|program| program.instructions.len())
                .sum(),
        ),
        max_action_batch_operation_count: 0,
        max_scheduler_batch_width: 0,
        max_dom_patch_count_per_action: 0,
        retained_slot_count: report_count(resume_runtime_artifact.slot_schemas.len()),
    };
    let (optimization_report, runtime_cost_report) = build_production_reports(
        &production_runtime_artifact,
        &production_chunk_graph,
        &report_inputs,
    );
    let optimization_report_json = optimization_report_json(&optimization_report);
    let runtime_cost_report_json = runtime_cost_report_json(&runtime_cost_report);
    write_build_artifacts(
        &out_dir,
        &page_html,
        &manifest_json,
        &computed_runtime_json,
        &context_runtime_json,
        &effect_runtime_json,
        &component_runtime_json,
        &forms_runtime_json,
        &resume_runtime_json,
        resource_runtime_json.as_deref(),
        opaque_runtime_json.as_deref(),
        &production_runtime_json,
        &optimization_report_json,
        &runtime_cost_report_json,
        &runtime_js,
        &resume_chunks,
    )
    .unwrap_or_else(|error| {
        eprintln!(
            "failed to write build artifacts to {}: {error}",
            out_dir.display()
        );

        process::exit(1);
    });
    maybe_write_production_modules(
        args.iter().any(|argument| argument == "--production"),
        &out_dir,
        &production_chunk_graph,
    );

    print_build_artifact_paths(
        &out_dir,
        &resume_chunks,
        resource_runtime_json.is_some(),
        opaque_runtime_json.is_some(),
    );
}

fn run_application_command(args: Vec<String>) {
    let Some((subcommand, options)) = args.split_first() else {
        eprintln!("usage: presolve application build --config <path> --source <logical=relative> [--source ...] --entry <logical> --out <directory> [--package-contract specifier=contract.json] [--package-runtime specifier=runtime-location] [--production]");
        process::exit(1);
    };
    if subcommand != "build" {
        eprintln!("PSAPP3001_UNSUPPORTED_APPLICATION_COMMAND: application supports only `build`");
        process::exit(2);
    }
    let parsed = parse_application_build_options(options);
    let project_root = parsed
        .configuration_path
        .parent()
        .filter(|path| !path.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    let envelope = load_explicit_project_envelope_v1(project_root, &parsed.configuration_path)
        .unwrap_or_else(|error| application_cli_error(error.code, &error.message));
    let sources = load_explicit_source_inputs_v1(&envelope.project_root, &parsed.source_specs)
        .unwrap_or_else(|error| application_cli_error(error.code, &error.message));
    validate_application_output_root(&parsed.output_root);
    let packages = parse_semantic_package_contracts(options);
    let runtime_modules = parse_semantic_package_runtime_modules(options, &packages);
    let request = ApplicationPublicationRequestV1 {
        configuration: envelope.configuration,
        sources: sources
            .into_iter()
            .map(|source| ApplicationPublicationSourceV1 {
                logical_path: PathBuf::from(source.logical_path),
                source: source.content,
            })
            .collect(),
        entry_path: parsed.entry_path,
        package_contracts: packages,
        package_runtime_modules: runtime_modules,
        profile: parsed.profile,
        output_root: parsed.output_root.clone(),
    };
    let validated = validate_application_publication_request_v1(request)
        .unwrap_or_else(|error| application_cli_error(error.code, &error.message));
    let product = build_application_publication_product_v1(validated)
        .unwrap_or_else(|error| application_cli_error(error.code, &error.message));
    publish_application_product(&parsed.output_root, &product)
        .unwrap_or_else(|error| application_cli_error("PSAPP3008_PUBLICATION_FAILED", &error));
    for path in product.artifacts.keys() {
        println!("Wrote {}", parsed.output_root.join(path).display());
    }
}

fn run_route_command(args: Vec<String>) {
    let Some((subcommand, options)) = args.split_first() else {
        application_cli_error(
            "PSROUTE3001_UNSUPPORTED_ROUTE_COMMAND",
            "route supports only `graph` or `request`",
        );
    };
    if subcommand != "graph" && subcommand != "request" {
        application_cli_error(
            "PSROUTE3001_UNSUPPORTED_ROUTE_COMMAND",
            "route supports only `graph` or `request`",
        );
    }
    let mut config = None;
    let mut sources = Vec::new();
    let mut index = 0;
    while index < options.len() {
        match options[index].as_str() {
            "--config" => {
                let Some(value) = options.get(index + 1) else {
                    application_cli_error(
                        "PSROUTE3002_INVALID_ARGUMENT",
                        "--config requires a path",
                    );
                };
                if config.replace(PathBuf::from(value)).is_some() {
                    application_cli_error(
                        "PSROUTE3002_INVALID_ARGUMENT",
                        "--config may appear only once",
                    );
                }
                index += 2;
            }
            "--source" => {
                let Some(value) = options.get(index + 1) else {
                    application_cli_error(
                        "PSROUTE3002_INVALID_ARGUMENT",
                        "--source requires logical=relative-path",
                    );
                };
                sources.push(
                    parse_explicit_source_spec_v1(value)
                        .unwrap_or_else(|error| application_cli_error(error.code, &error.message)),
                );
                index += 2;
            }
            "--package-contract" | "--package-runtime" => {
                if options.get(index + 1).is_none() {
                    application_cli_error(
                        "PSROUTE3002_INVALID_ARGUMENT",
                        "package mapping requires a value",
                    );
                }
                index += 2;
            }
            value => application_cli_error(
                "PSROUTE3002_INVALID_ARGUMENT",
                &format!("unknown route graph option `{value}`"),
            ),
        }
    }
    let config = config.unwrap_or_else(|| {
        application_cli_error("PSROUTE3002_INVALID_ARGUMENT", "--config is required")
    });
    let root = config
        .parent()
        .filter(|path| !path.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    let envelope = load_explicit_project_envelope_v1(root, &config)
        .unwrap_or_else(|error| application_cli_error(error.code, &error.message));
    let sources = load_explicit_source_inputs_v1(&envelope.project_root, &sources)
        .unwrap_or_else(|error| application_cli_error(error.code, &error.message));
    let packages = parse_semantic_package_contracts(options);
    let unit = CompilationUnit::parse_sources(
        sources
            .iter()
            .map(|source| (PathBuf::from(&source.logical_path), source.content.as_str())),
    );
    let model = build_application_semantic_model_for_unit_with_packages(&unit, &packages);
    let graph = build_validated_route_graph_v1(&model)
        .unwrap_or_else(|error| application_cli_error(error.code, &error.message));
    let manifest = presolve_compiler::route_manifest_v1(&graph);
    if subcommand == "graph" {
        print!("{}", presolve_compiler::route_manifest_json_v1(&manifest));
    } else {
        print!(
            "{}",
            presolve_compiler::static_request_handoff_json_v1(&build_static_request_handoff_v1(
                &manifest
            ))
        );
    }
}

struct ApplicationBuildOptions {
    configuration_path: PathBuf,
    source_specs: Vec<presolve_cli::CliExplicitSourceSpecV1>,
    entry_path: PathBuf,
    output_root: PathBuf,
    profile: ApplicationPublicationProfileV1,
}

fn parse_application_build_options(args: &[String]) -> ApplicationBuildOptions {
    let mut configuration_path = None;
    let mut source_specs = Vec::new();
    let mut entry_path = None;
    let mut output_root = None;
    let mut profile = ApplicationPublicationProfileV1::Development;
    let mut index = 0;
    while index < args.len() {
        let option = &args[index];
        match option.as_str() {
            "--config" => {
                let Some(value) = args.get(index + 1) else {
                    application_cli_error("PSAPP3002_INVALID_ARGUMENT", "--config requires a path");
                };
                if configuration_path.replace(PathBuf::from(value)).is_some() {
                    application_cli_error(
                        "PSAPP3002_INVALID_ARGUMENT",
                        "--config may appear only once",
                    );
                }
                index += 2;
            }
            "--source" => {
                let Some(value) = args.get(index + 1) else {
                    application_cli_error(
                        "PSAPP3002_INVALID_ARGUMENT",
                        "--source requires logical=relative-path",
                    );
                };
                source_specs.push(
                    parse_explicit_source_spec_v1(value)
                        .unwrap_or_else(|error| application_cli_error(error.code, &error.message)),
                );
                index += 2;
            }
            "--entry" => {
                let Some(value) = args.get(index + 1) else {
                    application_cli_error(
                        "PSAPP3002_INVALID_ARGUMENT",
                        "--entry requires a logical path",
                    );
                };
                if entry_path.replace(PathBuf::from(value)).is_some() {
                    application_cli_error(
                        "PSAPP3002_INVALID_ARGUMENT",
                        "--entry may appear only once",
                    );
                }
                index += 2;
            }
            "--out" => {
                let Some(value) = args.get(index + 1) else {
                    application_cli_error(
                        "PSAPP3002_INVALID_ARGUMENT",
                        "--out requires a directory path",
                    );
                };
                if output_root.replace(PathBuf::from(value)).is_some() {
                    application_cli_error(
                        "PSAPP3002_INVALID_ARGUMENT",
                        "--out may appear only once",
                    );
                }
                index += 2;
            }
            "--production" => {
                profile = ApplicationPublicationProfileV1::Production;
                index += 1;
            }
            "--package-contract" | "--package-runtime" => {
                if args.get(index + 1).is_none() {
                    application_cli_error(
                        "PSAPP3002_INVALID_ARGUMENT",
                        &format!("{option} requires a value"),
                    );
                }
                index += 2;
            }
            _ => application_cli_error(
                "PSAPP3002_INVALID_ARGUMENT",
                &format!("unknown application build option `{option}`"),
            ),
        }
    }
    ApplicationBuildOptions {
        configuration_path: configuration_path.unwrap_or_else(|| {
            application_cli_error("PSAPP3002_INVALID_ARGUMENT", "--config is required")
        }),
        source_specs,
        entry_path: entry_path.unwrap_or_else(|| {
            application_cli_error("PSAPP3002_INVALID_ARGUMENT", "--entry is required")
        }),
        output_root: output_root.unwrap_or_else(|| {
            application_cli_error("PSAPP3002_INVALID_ARGUMENT", "--out is required")
        }),
        profile,
    }
}

fn application_cli_error(code: &str, message: &str) -> ! {
    eprintln!("{code}: {message}");
    process::exit(2);
}

fn validate_application_output_root(output_root: &Path) {
    let Some(parent) = output_root.parent() else {
        application_cli_error(
            "PSAPP3003_INVALID_OUTPUT_ROOT",
            "--out must have a caller-owned parent directory",
        );
    };
    if output_root.file_name().is_none() || output_root.as_os_str().is_empty() {
        application_cli_error(
            "PSAPP3003_INVALID_OUTPUT_ROOT",
            "--out must name a non-empty output root",
        );
    }
    if let Err(error) = fs::create_dir_all(parent) {
        application_cli_error(
            "PSAPP3003_INVALID_OUTPUT_ROOT",
            &format!(
                "failed to prepare output parent {}: {error}",
                parent.display()
            ),
        );
    }
    if let Ok(metadata) = fs::symlink_metadata(output_root) {
        if !metadata.file_type().is_symlink() {
            application_cli_error(
                "PSAPP3004_OUTPUT_ROOT_NOT_PUBLICATION_POINTER",
                "--out already exists and is not a Presolve application publication pointer",
            );
        }
    }
}

fn publish_application_product(
    output_root: &Path,
    product: &presolve_compiler::ApplicationPublicationProductV1,
) -> Result<(), String> {
    let stage = create_application_publication_stage(output_root)?;
    let publication = (|| {
        for (relative_path, bytes) in &product.artifacts {
            validate_publication_relative_path(relative_path)?;
            let path = stage.join(relative_path);
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent).map_err(|error| error.to_string())?;
            }
            fs::write(path, bytes).map_err(|error| error.to_string())?;
        }
        validate_staged_application_product(&stage, product)?;
        let pointer = stage.with_extension(format!(
            "publish-{}",
            NEXT_APPLICATION_PUBLICATION_STAGE.fetch_add(1, Ordering::Relaxed)
        ));
        create_application_publication_pointer(
            stage
                .file_name()
                .ok_or_else(|| "stage path has no file name".to_string())?,
            &pointer,
        )?;
        fs::rename(&pointer, output_root).map_err(|error| {
            let _ = fs::remove_file(&pointer);
            format!(
                "failed to atomically replace {}: {error}",
                output_root.display()
            )
        })?;
        Ok(())
    })();
    if publication.is_err() {
        let _ = fs::remove_dir_all(&stage);
    }
    publication
}

fn create_application_publication_stage(output_root: &Path) -> Result<PathBuf, String> {
    let parent = output_root
        .parent()
        .filter(|path| !path.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    let name = output_root
        .file_name()
        .ok_or_else(|| "output root has no file name".to_string())?
        .to_string_lossy();
    for _ in 0..64 {
        let id = NEXT_APPLICATION_PUBLICATION_STAGE.fetch_add(1, Ordering::Relaxed);
        let stage = parent.join(format!(".{name}.presolve-release-{}-{id}", process::id()));
        match fs::create_dir(&stage) {
            Ok(()) => return Ok(stage),
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => continue,
            Err(error) => {
                return Err(format!(
                    "failed to create publication staging directory: {error}"
                ))
            }
        }
    }
    Err("failed to allocate a unique publication staging directory".into())
}

fn validate_publication_relative_path(path: &Path) -> Result<(), String> {
    if path.as_os_str().is_empty()
        || path.is_absolute()
        || path.components().any(|component| {
            matches!(
                component,
                std::path::Component::ParentDir
                    | std::path::Component::RootDir
                    | std::path::Component::Prefix(_)
            )
        })
    {
        return Err(format!(
            "compiler product contains an invalid artifact path {}",
            path.display()
        ));
    }
    Ok(())
}

fn validate_staged_application_product(
    stage: &Path,
    product: &presolve_compiler::ApplicationPublicationProductV1,
) -> Result<(), String> {
    let manifest_path = Path::new("application.manifest.json");
    let manifest_bytes = fs::read(stage.join(manifest_path)).map_err(|error| error.to_string())?;
    let expected_manifest =
        presolve_compiler::application_publication_manifest_json_v1(&product.manifest);
    if manifest_bytes != expected_manifest.as_bytes() {
        return Err("staged application manifest differs from the compiler product".into());
    }
    if product.manifest.artifacts.len() + 1 != product.artifacts.len() {
        return Err("compiler product manifest inventory is incomplete".into());
    }
    for artifact in &product.manifest.artifacts {
        let path = PathBuf::from(&artifact.path);
        validate_publication_relative_path(&path)?;
        let bytes = fs::read(stage.join(&path)).map_err(|error| error.to_string())?;
        let digest = format!("sha256:{:x}", Sha256::digest(bytes));
        if digest != artifact.digest {
            return Err(format!(
                "staged artifact digest mismatch for {}",
                artifact.path
            ));
        }
    }
    Ok(())
}

#[cfg(unix)]
fn create_application_publication_pointer(
    target: &std::ffi::OsStr,
    pointer: &Path,
) -> Result<(), String> {
    std::os::unix::fs::symlink(target, pointer).map_err(|error| error.to_string())
}

#[cfg(windows)]
fn create_application_publication_pointer(
    target: &std::ffi::OsStr,
    pointer: &Path,
) -> Result<(), String> {
    std::os::windows::fs::symlink_dir(target, pointer).map_err(|error| error.to_string())
}

#[cfg(not(any(unix, windows)))]
fn create_application_publication_pointer(
    _target: &std::ffi::OsStr,
    _pointer: &Path,
) -> Result<(), String> {
    Err("atomic application publication pointers are unsupported on this platform".into())
}

fn parse_semantic_package_contracts(args: &[String]) -> SemanticPackageResolutionTable {
    let mut packages = SemanticPackageResolutionTable::default();
    let mut index = 0;
    while index < args.len() {
        if args[index] != "--package-contract" {
            index += 1;
            continue;
        }
        let Some(specification) = args.get(index + 1) else {
            eprintln!("--package-contract requires <specifier>=<contract-path>");
            process::exit(2);
        };
        let Some((specifier, contract_path)) = specification.split_once('=') else {
            eprintln!("--package-contract requires <specifier>=<contract-path>");
            process::exit(2);
        };
        if specifier.is_empty() || contract_path.is_empty() {
            eprintln!("--package-contract requires non-empty specifier and contract path");
            process::exit(2);
        }
        let source = fs::read_to_string(contract_path).unwrap_or_else(|error| {
            eprintln!("failed to read semantic package contract {contract_path}: {error}");
            process::exit(1);
        });
        let contract =
            presolve_compiler::parse_semantic_package_contract(&source).unwrap_or_else(|error| {
                eprintln!("invalid semantic package contract {contract_path}: {error:?}");
                process::exit(2);
            });
        packages
            .insert(specifier.to_string(), contract)
            .unwrap_or_else(|error| {
                eprintln!("invalid semantic package contract resolution `{specifier}`: {error:?}");
                process::exit(2);
            });
        index += 2;
    }
    packages
}

fn parse_semantic_package_runtime_modules(
    args: &[String],
    packages: &SemanticPackageResolutionTable,
) -> SemanticPackageRuntimeModuleTable {
    let mut modules = SemanticPackageRuntimeModuleTable::default();
    let mut index = 0;
    while index < args.len() {
        if args[index] != "--package-runtime" {
            index += 1;
            continue;
        }
        let Some(specification) = args.get(index + 1) else {
            eprintln!("--package-runtime requires <specifier>=<runtime-location>");
            process::exit(2);
        };
        let Some((specifier, location)) = specification.split_once('=') else {
            eprintln!("--package-runtime requires <specifier>=<runtime-location>");
            process::exit(2);
        };
        let Some(contract) = packages.contract(specifier) else {
            eprintln!("--package-runtime requires a matching --package-contract for `{specifier}`");
            process::exit(2);
        };
        for export in contract.exports.values() {
            let key = SemanticPackageRuntimeModuleKey {
                package: contract.package.clone(),
                version: contract.version.clone(),
                integrity: contract.integrity.clone(),
                runtime_module: export.runtime_module.clone(),
            };
            if modules.contains(&key) {
                continue;
            }
            modules
                .insert(key, location.to_string())
                .unwrap_or_else(|error| {
                    eprintln!("invalid --package-runtime for `{specifier}`: {error:?}");
                    process::exit(2);
                });
        }
        index += 2;
    }
    modules
}

fn print_build_artifact_paths(
    out_dir: &Path,
    resume_chunks: &presolve_compiler::ResumeChunkGraph,
    includes_resources: bool,
    includes_opaque: bool,
) {
    for artifact in [
        "index.html",
        "template.manifest.json",
        "computed.runtime.json",
        "context.runtime.json",
        "effect.runtime.json",
        "component.runtime.json",
        "forms.runtime.json",
        "resume.runtime.json",
        "production.runtime.json",
        "optimization-report.json",
        "runtime-cost-report.json",
        "runtime.js",
    ] {
        println!("Wrote {}", out_dir.join(artifact).display());
    }
    if includes_resources {
        println!("Wrote {}", out_dir.join("resources.runtime.json").display());
    }
    if includes_opaque {
        println!("Wrote {}", out_dir.join("opaque.runtime.json").display());
    }
    for chunk in &resume_chunks.chunks {
        println!(
            "Wrote {}",
            out_dir.join(&chunk.module.module_path).display()
        );
    }
}

fn byte_count<'a>(values: impl IntoIterator<Item = &'a String>) -> u64 {
    u64::try_from(values.into_iter().map(String::len).sum::<usize>())
        .expect("build byte count exceeds u64")
}

fn report_count(value: usize) -> u32 {
    u32::try_from(value).expect("build report count exceeds u32")
}

fn production_root_chunk_inputs(
    resume: &presolve_compiler::ResumeManifest,
) -> Vec<ProductionRootChunkInput> {
    let mut roots = resume
        .chunks
        .iter()
        .filter_map(|chunk| {
            let root_kind = match chunk.root_kind {
                presolve_compiler::resume_manifest::ResumeManifestChunkRootKind::Eager => {
                    return None
                }
                presolve_compiler::resume_manifest::ResumeManifestChunkRootKind::Interaction => {
                    "interaction"
                }
                presolve_compiler::resume_manifest::ResumeManifestChunkRootKind::Visible => {
                    "visible"
                }
                presolve_compiler::resume_manifest::ResumeManifestChunkRootKind::Manual => "manual",
            };
            Some(ProductionRootChunkInput {
                activation_root_id: chunk.root_id.clone(),
                root_kind: root_kind.to_string(),
                programs: chunk
                    .provided_program_ids
                    .iter()
                    .map(|program| {
                        ExecutableProgramFingerprint::for_canonical_opcode_stream(
                            program.as_bytes(),
                        )
                    })
                    .collect(),
            })
        })
        .collect::<Vec<_>>();
    roots.sort_by(|left, right| left.activation_root_id.cmp(&right.activation_root_id));
    roots
}

fn write_production_module_layout(
    out_dir: &Path,
    layout: &presolve_compiler::ProductionModuleLayout,
) -> io::Result<()> {
    let production_dir = out_dir.join("production");
    fs::create_dir_all(&production_dir)?;
    for module in std::iter::once(&layout.eager)
        .chain(layout.shared.iter())
        .chain(layout.roots.iter())
    {
        fs::write(production_dir.join(&module.filename), &module.source)?;
    }
    Ok(())
}

fn maybe_write_production_modules(
    production_mode: bool,
    out_dir: &Path,
    graph: &presolve_compiler::ProductionChunkGraph,
) {
    if !production_mode {
        return;
    }
    let layout = emit_production_modules(graph);
    write_production_module_layout(out_dir, &layout).unwrap_or_else(|error| {
        eprintln!(
            "failed to write production modules to {}: {error}",
            out_dir.display()
        );
        process::exit(1);
    });
    println!("Wrote {}", out_dir.join("production").display());
}

fn production_mode_page_html(
    page_html: String,
    production_mode: bool,
    artifact_json: &str,
) -> String {
    if !production_mode {
        return page_html;
    }
    let artifact = artifact_json.replace("</script", "<\\/script");
    page_html.replacen(
        "    <script src=\"./runtime.js\" defer></script>",
        &format!(
            "    <script type=\"application/json\" id=\"presolve-production-runtime\">{artifact}    </script>\n    <script src=\"./runtime.js\" defer></script>"
        ),
        1,
    )
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
        || "Presolve App".to_string(),
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
            "--production" => {
                index += 1;
            }
            "--package-contract" => {
                if args.get(index + 1).is_none() {
                    eprintln!("missing value for --package-contract");
                    process::exit(1);
                }
                index += 2;
            }
            "--package-runtime" => {
                if args.get(index + 1).is_none() {
                    eprintln!("missing value for --package-runtime");
                    process::exit(1);
                }
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

fn print_parsed_jsx_element_details(element: &presolve_parser::ParsedJsxElement, indent: usize) {
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

fn print_render_children(children: &[presolve_compiler::RenderChild], indent: usize) {
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

fn print_render_root(render: &presolve_compiler::RenderModel) {
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
        StateOperation::AssignParameter(_) => "assign-parameter",
        StateOperation::Toggle => "toggle",
    }
}

fn format_parsed_event_handlers(
    event_handlers: &[presolve_parser::ParsedEventHandler],
) -> Vec<String> {
    event_handlers
        .iter()
        .map(|event_handler| format!("{} -> {}", event_handler.event, event_handler.handler))
        .collect()
}

fn format_render_event_handlers(
    event_handlers: &[presolve_compiler::RenderEventHandler],
) -> Vec<String> {
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

fn format_render_child(child: &presolve_compiler::RenderChild) -> String {
    match child {
        presolve_compiler::RenderChild::Text { value, span } => {
            format!("Text({value:?}) {}", format_line_column_span(span))
        }
        presolve_compiler::RenderChild::Binding { expression, span } => {
            format!("Binding({expression:?}) {}", format_line_column_span(span))
        }
        presolve_compiler::RenderChild::Element(element) => format!(
            "Element <{}> {}",
            element.tag_name,
            format_line_column_span(&element.span)
        ),
        presolve_compiler::RenderChild::Fragment(fragment) => {
            format!("Fragment <> {}", format_line_column_span(&fragment.span))
        }
        presolve_compiler::RenderChild::Conditional(conditional) => format!(
            "Conditional({:?}) {}",
            conditional.condition,
            format_line_column_span(&conditional.span)
        ),
        presolve_compiler::RenderChild::List(list) => format!(
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

fn print_template_list(path: &Path, list: &presolve_compiler::ListNode, indent: usize) {
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
        AttributeValue::EventHandler { event, handler, .. } => {
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
    resume_runtime_json: &str,
    resource_runtime_json: Option<&str>,
    opaque_runtime_json: Option<&str>,
    production_runtime_json: &str,
    optimization_report_json: &str,
    runtime_cost_report_json: &str,
    runtime_js: &str,
    resume_chunks: &presolve_compiler::ResumeChunkGraph,
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
    fs::write(
        out_dir.join("optimization-report.json"),
        optimization_report_json,
    )?;
    fs::write(
        out_dir.join("runtime-cost-report.json"),
        runtime_cost_report_json,
    )?;
    fs::write(out_dir.join("forms.runtime.json"), forms_runtime_json)?;
    fs::write(out_dir.join("resume.runtime.json"), resume_runtime_json)?;
    if let Some(resource_runtime_json) = resource_runtime_json {
        fs::write(
            out_dir.join("resources.runtime.json"),
            resource_runtime_json,
        )?;
    }
    if let Some(opaque_runtime_json) = opaque_runtime_json {
        fs::write(out_dir.join("opaque.runtime.json"), opaque_runtime_json)?;
    }
    fs::write(
        out_dir.join("production.runtime.json"),
        production_runtime_json,
    )?;

    fs::write(out_dir.join("runtime.js"), runtime_js)?;
    for chunk in &resume_chunks.chunks {
        fs::write(
            out_dir.join(&chunk.module.module_path),
            &chunk.module.canonical_module_bytes,
        )?;
    }

    Ok(())
}

fn print_usage_and_exit() -> ! {
    eprintln!("usage:");
    eprintln!("  presolve explain --capabilities --format human|json|migration");
    eprintln!("  presolve explain <file> [--format text|json]");
    eprintln!("  presolve explain <file> [--inspect] [--entity semantic-id | --source path --offset byte] [--child-kind kind] [--reference-kind kind] [--format text|json|graph]");
    eprintln!(
        "  presolve check <file> [file...] [--format text|json] [--category parser|compiler|validation] [--fail-on error|warning|info]"
    );
    eprintln!(
        "  presolve check --config <file> --source <logical=relative-file> [--source ...] [--verify-clean-equivalence] [--format human|json]"
    );
    eprintln!("  presolve parse <file>");
    eprintln!("  presolve graph <file>");
    eprintln!("  presolve template <file>");
    eprintln!("  presolve html <file>");
    eprintln!("  presolve manifest <file>");
    eprintln!("  presolve build <file> [--package-contract specifier=contract.json] [--package-runtime specifier=runtime-location] [--out dir] [--production]");
    eprintln!("  presolve application build --config <file> --source <logical=relative-file> [--source ...] --entry <logical> --out <publication-pointer> [--package-contract specifier=contract.json] [--package-runtime specifier=runtime-location] [--production]");
    eprintln!("  presolve route graph --config <file> --source <logical=relative-file> [--source ...] [--package-contract specifier=contract.json] [--package-runtime specifier=runtime-location]");
    eprintln!("  presolve route request --config <file> --source <logical=relative-file> [--source ...] [--package-contract specifier=contract.json] [--package-runtime specifier=runtime-location]");
    eprintln!(
        "  presolve build --config <file> --source <logical=relative-file> [--source ...] [--verify-clean-equivalence] [--format human|json]"
    );
    process::exit(1);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn k17_accepts_only_asm_inspection_schema_v12() {
        assert!(supports_asm_inspection_schema(12));
        assert!(!supports_asm_inspection_schema(11));
        assert!(!supports_asm_inspection_schema(13));
    }

    #[test]
    fn formats_sorted_asm_validation_diagnostics_only_when_present() {
        assert!(asm_validation_diagnostics_text(&[]).is_none());

        let diagnostics = vec![
            AsmValidationDiagnostic {
                code: "PSASM1002".to_string(),
                message: "second".to_string(),
            },
            AsmValidationDiagnostic {
                code: "PSASM1001".to_string(),
                message: "first".to_string(),
            },
        ];

        assert_eq!(
            asm_validation_diagnostics_text(&diagnostics),
            Some(
                "  ASM validation diagnostics:\n    PSASM1001: first\n    PSASM1002: second\n"
                    .to_string()
            )
        );
    }

    #[test]
    fn j19_resume_diagnostics_extend_inspection_to_schema_v11() {
        let path = PathBuf::from("src/Profile.tsx");
        let parsed = presolve_parser::parse_file(
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
        let document = asm_inspection_json(&[path], &asm, &[], &[]);
        let json: serde_json::Value = serde_json::from_str(&document).unwrap();
        assert_eq!(json["schema_version"], ASM_INSPECTION_SCHEMA_VERSION);
        assert_eq!(json["schema_version"], 12);
        assert!(document.contains("validation-rule"));
        assert!(document.contains("validation_rule"));
    }

    #[test]
    fn j19_resume_diagnostics_share_full_selected_and_check_json_evidence() {
        let path = PathBuf::from("src/ResumeDiagnostic.tsx");
        let parsed = presolve_parser::parse_file(
            &path,
            r#"@component("x-resume-diagnostic") class ResumeDiagnostic {
  value = state(1);
  render() { return <main>{this.value}</main>; }
}"#,
        );
        let unit = CompilationUnit::from_parsed_files(vec![parsed]);
        let mut asm = build_application_semantic_model_for_unit(&unit);
        let state = asm.components[0].state_fields[0].id.clone();
        asm.semantic_types
            .assignments
            .get_mut(&state)
            .expect("state type")
            .semantic_type = presolve_compiler::SemanticType::Unknown;
        let resume_diagnostics = project_resume_diagnostics(&asm);
        assert!(resume_diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "PSC1096"));

        let full: serde_json::Value = serde_json::from_str(&asm_inspection_json(
            std::slice::from_ref(&path),
            &asm,
            &[],
            &resume_diagnostics,
        ))
        .expect("full ASM JSON");
        assert_eq!(full["schema_version"], 12);
        let full_diagnostic = full["resume_diagnostics"]
            .as_array()
            .expect("resume diagnostics")
            .iter()
            .find(|diagnostic| diagnostic["code"] == "PSC1096")
            .expect("PSC1096");
        assert!(full_diagnostic["primary_identity"].as_str().is_some());

        let entity = asm
            .ownership
            .keys()
            .find(|id| id.as_str() == state.as_str())
            .expect("state entity");
        let selected: serde_json::Value = serde_json::from_str(&asm_entity_inspection_json(
            &asm,
            entity,
            &asm.diagnostics,
            &resume_diagnostics,
            AsmEntityFilters::default(),
        ))
        .expect("selected ASM JSON");
        assert_eq!(selected["schema_version"], 12);
        assert!(selected["resume_diagnostics"]
            .as_array()
            .expect("selected resume diagnostics")
            .iter()
            .any(|diagnostic| diagnostic["code"] == "PSC1096"));

        let check: serde_json::Value = serde_json::from_str(&check_json(
            &unit,
            &asm,
            &[],
            &resume_diagnostics,
            &["validation".to_string()],
            &ParseSeverity::Error,
        ))
        .expect("check JSON");
        assert_eq!(check["schema_version"], 6);
        assert!(check["resume_diagnostics"]
            .as_array()
            .expect("check resume diagnostics")
            .iter()
            .any(|diagnostic| diagnostic["code"] == "PSC1096"));
    }
}
