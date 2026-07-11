use std::env;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process;

use ezc_core::{
    build_application_semantic_model, build_component_graph, build_template_graph,
    build_template_manifest, explain_json, explain_text, generate_runtime_stub,
    generate_standalone_page, generate_static_html, summarize_source, template_manifest_json,
    validate_application_semantic_model, AttributeValue, ComponentGraph, RenderAttribute,
    RenderAttributeValue, SerializableValue, StateOperation, TemplateChild, TemplateGraph,
};
use ezc_parser::{
    parse_file, ParseSeverity, ParsedClass, ParsedFile, ParsedJsxAttribute,
    ParsedJsxAttributeValue, ParsedJsxChild, ParsedJsxNode, ParsedMethod, SourceSpan,
};

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
        "asm" => run_asm(args),
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

fn run_asm(mut args: Vec<String>) {
    if args.is_empty() {
        eprintln!("missing file path");
        print_usage_and_exit();
    }
    let path = PathBuf::from(args.remove(0));
    let source = fs::read_to_string(&path).unwrap_or_else(|error| {
        eprintln!("failed to read {}: {error}", path.display());
        process::exit(1);
    });
    let asm = build_application_semantic_model(&parse_file(&path, &source));
    let validation = validate_application_semantic_model(&asm);
    println!("File: {}", path.display());
    println!("ApplicationSemanticModel:");
    println!("  components: {}", asm.components.len());
    println!("  templates: {}", asm.templates.len());
    println!("  ownership: {}", asm.ownership.len());
    println!("  references: {}", asm.references.len());
    println!("  provenance: {}", asm.provenance.len());
    println!("  diagnostics: {}", asm.diagnostics.len());
    println!("  validation: {}", validation.len());
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
    eprintln!("  ezc_cli parse <file>");
    eprintln!("  ezc_cli graph <file>");
    eprintln!("  ezc_cli template <file>");
    eprintln!("  ezc_cli html <file>");
    eprintln!("  ezc_cli manifest <file>");
    eprintln!("  ezc_cli build <file> [--out dir]");
    process::exit(1);
}
