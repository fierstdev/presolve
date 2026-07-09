use std::env;
use std::fs;
use std::path::PathBuf;
use std::process;

use ezc_core::{
    build_component_graph, build_template_graph, explain_json, explain_text, generate_static_html,
    summarize_source, AttributeValue, ComponentGraph, TemplateChild, TemplateGraph,
};
use ezc_parser::{parse_file, ParseSeverity, ParsedFile};

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
        "template" => run_template(args),
        "html" => run_html(args),
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

                format = value.clone();
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

fn print_parsed_file(parsed: &ParsedFile) {
    println!("File: {}", parsed.path.display());

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

    println!();
    println!("Classes:");
    if parsed.classes.is_empty() {
        println!("  none");
        return;
    }

    for class in &parsed.classes {
        println!(
            "  class {} at {}:{}",
            class.name, class.span.line, class.span.column
        );

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

        println!("    methods:");
        if class.methods.is_empty() {
            println!("      none");
        } else {
            for method in &class.methods {
                println!(
                    "      {} at {}:{}",
                    method.name, method.span.line, method.span.column
                );

                if method.jsx_roots.is_empty() {
                    println!("        jsx roots: none");
                } else {
                    for jsx in &method.jsx_roots {
                        println!(
                            "        jsx root <{}> at {}:{}",
                            jsx.name, jsx.span.line, jsx.span.column
                        );

                        if jsx.attributes.is_empty() {
                            println!("          attributes: none");
                        } else {
                            println!("          attributes: {}", jsx.attributes.join(", "));
                        }

                        if jsx.event_handler_refs.is_empty() {
                            println!("          event handlers: none");
                        } else {
                            println!(
                                "          event handlers: {}",
                                jsx.event_handler_refs.join(", ")
                            );
                        }

                        if jsx.children.is_empty() {
                            println!("          children: none");
                        } else {
                            println!("          children:");
                            for child in &jsx.children {
                                println!("            {:?}", child);
                            }
                        }
                    }
                }

                if method.bindings.is_empty() {
                    println!("        bindings: none");
                } else {
                    println!("        bindings: {}", method.bindings.join(", "));
                }
            }
        }
    }
}

fn print_component_graph(path: &PathBuf, graph: &ComponentGraph) {
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

        println!("      render:");
        match &component.render {
            Some(render) => {
                match &render.root_element {
                    Some(root) => println!("        root: <{root}>"),
                    None => println!("        root: none"),
                }

                if render.attributes.is_empty() {
                    println!("        attributes: none");
                } else {
                    println!("        attributes: {}", render.attributes.join(", "));
                }

                if render.event_handler_refs.is_empty() {
                    println!("        event handlers: none");
                } else {
                    println!(
                        "        event handlers: {}",
                        render.event_handler_refs.join(", ")
                    );
                }

                if render.children.is_empty() {
                    println!("        children: none");
                } else {
                    println!("        children:");
                    for child in &render.children {
                        println!("          {:?}", child);
                    }
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

fn print_template_graph(path: &PathBuf, graph: &TemplateGraph) {
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
                println!("      root: <{}> id={}", root.tag_name, root.id.0);

                println!("      attributes:");
                if root.attributes.is_empty() {
                    println!("        none");
                } else {
                    for attribute in &root.attributes {
                        println!(
                            "        {} = {}",
                            attribute.name,
                            format_attribute_value(&attribute.value)
                        );
                    }
                }

                println!("      children:");
                if root.children.is_empty() {
                    println!("        none");
                } else {
                    for child in &root.children {
                        print_template_child(child, 8);
                    }
                }
            }
            None => {
                println!("      root: none");
            }
        }
    }
}

fn print_template_child(child: &TemplateChild, indent: usize) {
    let padding = " ".repeat(indent);

    match child {
        TemplateChild::Text(text) => println!("{padding}Text({text:?})"),
        TemplateChild::Binding { id, expression } => {
            println!("{padding}Binding id={} expression={expression:?}", id.0);
        }
        TemplateChild::Element(element) => {
            println!(
                "{padding}Element <{}> id={}",
                element.tag_name, element.id.0
            );

            let child_padding = " ".repeat(indent + 2);

            println!("{child_padding}attributes:");
            if element.attributes.is_empty() {
                println!("{child_padding}  none");
            } else {
                for attribute in &element.attributes {
                    println!(
                        "{child_padding}  {} = {}",
                        attribute.name,
                        format_attribute_value(&attribute.value)
                    );
                }
            }

            println!("{child_padding}children:");
            if element.children.is_empty() {
                println!("{child_padding}  none");
            } else {
                for child in &element.children {
                    print_template_child(child, indent + 4);
                }
            }
        }
    }
}

fn format_attribute_value(value: &AttributeValue) -> String {
    match value {
        AttributeValue::Static(value) => format!("{value:?}"),
        AttributeValue::EventHandler(handler) => format!("event-handler({handler})"),
        AttributeValue::BindingList(bindings) => format!("bindings({})", bindings.join(", ")),
    }
}

fn diagnostic_severity_label(severity: &ParseSeverity) -> &'static str {
    match severity {
        ParseSeverity::Info => "Info",
        ParseSeverity::Warning => "Warning",
        ParseSeverity::Error => "Error",
    }
}

fn print_usage_and_exit() -> ! {
    eprintln!("usage:");
    eprintln!("  ezc_cli explain <file> [--format text|json]");
    eprintln!("  ezc_cli parse <file>");
    eprintln!("  ezc_cli graph <file>");
    eprintln!("  ezc_cli template <file>");
    eprintln!("  ezc_cli html <file>");
    process::exit(1);
}
