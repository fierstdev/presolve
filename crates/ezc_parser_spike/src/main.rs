use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::rc::Rc;

use oxc_allocator::Allocator;
use oxc_ast::ast::{
    Argument, ClassElement, Declaration, Expression, JSXAttributeItem, JSXAttributeName,
    JSXAttributeValue, JSXChild, JSXElementName, JSXExpression, Program, PropertyKey, Statement,
};
use oxc_parser::Parser;
use oxc_span::{SourceType, Span};

use swc_common::{FileName, SourceMap};
use swc_ecma_ast::{Decl, ModuleDecl, ModuleItem, Stmt};
use swc_ecma_parser::{lexer::Lexer, Parser as SwcParser, StringInput, Syntax, TsSyntax};

#[derive(Debug, Default)]
struct ParserProbe {
    classes: Vec<ClassProbe>,
}

#[derive(Debug)]
struct ClassProbe {
    name: String,
    span: Span,
    decorators: Vec<DecoratorProbe>,
    properties: Vec<PropertyProbe>,
    methods: Vec<MethodProbe>,
}

#[derive(Debug)]
struct DecoratorProbe {
    name: String,
    argument: Option<String>,
    span: Span,
}

#[derive(Debug)]
struct PropertyProbe {
    name: String,
    initializer: Option<String>,
    span: Span,
}

#[derive(Debug)]
struct MethodProbe {
    name: String,
    span: Span,
    jsx_roots: Vec<JsxElementProbe>,
    bindings: Vec<String>,
}

#[derive(Debug)]
struct JsxElementProbe {
    name: String,
    span: Span,
    attributes: Vec<String>,
}

fn main() {
    let mut args = env::args().skip(1).collect::<Vec<_>>();

    let use_swc = args.iter().any(|arg| arg == "--swc");
    args.retain(|arg| arg != "--swc");

    let path = args
        .first()
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("fixtures/0001-source-summary/input/Counter.tsx"));

    let source = fs::read_to_string(&path).expect("failed to read source file");

    if use_swc {
        run_swc(&path, &source);
    } else {
        run_oxc(&path, &source);
    }
}

fn run_swc(path: &Path, source: &str) {
    let source_map = Rc::new(SourceMap::default());
    let file = source_map.new_source_file(
        FileName::Real(path.to_path_buf()).into(),
        source.to_string(),
    );

    let lexer = Lexer::new(
        Syntax::Typescript(TsSyntax {
            tsx: true,
            decorators: true,
            ..Default::default()
        }),
        Default::default(),
        StringInput::from(&*file),
        None,
    );

    let mut parser = SwcParser::new_from(lexer);

    match parser.parse_module() {
        Ok(module) => {
            println!("File: {}", path.display());
            println!("SWC parsed module.");
            println!("Top-level items: {}", module.body.len());

            for item in &module.body {
                match item {
                    ModuleItem::Stmt(Stmt::Decl(Decl::Class(class_decl))) => {
                        println!(
                            "ClassDeclaration: {} span={:?}",
                            class_decl.ident.sym, class_decl.class.span
                        );
                        println!("  decorators: {}", class_decl.class.decorators.len());
                        println!("  body elements: {}", class_decl.class.body.len());
                    }
                    ModuleItem::ModuleDecl(ModuleDecl::ExportDecl(export_decl)) => {
                        println!("ExportDecl span={:?}", export_decl.span);
                    }
                    other => {
                        println!("ModuleItem: {:?}", other);
                    }
                }
            }
        }
        Err(error) => {
            println!("File: {}", path.display());
            println!("SWC parse error: {error:?}");
        }
    }
}

fn run_oxc(path: &Path, source: &str) {
    let source_type = SourceType::from_path(path)
        .unwrap_or_default()
        .with_typescript(true)
        .with_jsx(true);

    let allocator = Allocator::default();
    let ret = Parser::new(&allocator, source, source_type).parse();

    println!("File: {}", path.display());
    println!("Errors: {}", ret.errors.len());

    for error in &ret.errors {
        println!("  {error:?}");
    }

    let probe = probe_program(&ret.program);

    print_probe(&probe);

    if !ret.errors.is_empty() {
        print_diagnostic_probe(source, &ret.errors);
    }
}

fn probe_program(program: &Program<'_>) -> ParserProbe {
    let mut probe = ParserProbe::default();

    for statement in &program.body {
        if let Some(declaration) = statement.as_declaration() {
            if let Some(class_probe) = probe_declaration(declaration) {
                probe.classes.push(class_probe);
            }
        }
    }

    probe
}

fn probe_declaration(declaration: &Declaration<'_>) -> Option<ClassProbe> {
    let Declaration::ClassDeclaration(class) = declaration else {
        return None;
    };

    let name = class
        .id
        .as_ref()
        .map(|id| id.name.to_string())
        .unwrap_or_else(|| "<anonymous>".to_string());

    let decorators = class
        .decorators
        .iter()
        .filter_map(probe_decorator)
        .collect::<Vec<_>>();

    let mut properties = Vec::new();
    let mut methods = Vec::new();

    for element in &class.body.body {
        match element {
            ClassElement::PropertyDefinition(property) => {
                if let Some(property_probe) = probe_property(property) {
                    properties.push(property_probe);
                }
            }
            ClassElement::MethodDefinition(method) => {
                if let Some(method_probe) = probe_method(method) {
                    methods.push(method_probe);
                }
            }
            _ => {}
        }
    }

    Some(ClassProbe {
        name,
        span: class.span,
        decorators,
        properties,
        methods,
    })
}

fn probe_decorator(decorator: &oxc_ast::ast::Decorator<'_>) -> Option<DecoratorProbe> {
    let Expression::CallExpression(call) = &decorator.expression else {
        return None;
    };

    let Expression::Identifier(callee) = &call.callee else {
        return None;
    };

    let argument = call.arguments.first().and_then(argument_string_value);

    Some(DecoratorProbe {
        name: callee.name.to_string(),
        argument,
        span: decorator.span,
    })
}

fn argument_string_value(argument: &Argument<'_>) -> Option<String> {
    match argument {
        Argument::StringLiteral(literal) => Some(literal.value.to_string()),
        _ => None,
    }
}

fn probe_property(property: &oxc_ast::ast::PropertyDefinition<'_>) -> Option<PropertyProbe> {
    let name = property_key_name(&property.key)?;

    let initializer = property.value.as_ref().and_then(expression_summary);

    Some(PropertyProbe {
        name,
        initializer,
        span: property.span,
    })
}

fn probe_method(method: &oxc_ast::ast::MethodDefinition<'_>) -> Option<MethodProbe> {
    let name = property_key_name(&method.key)?;

    let mut jsx_roots = Vec::new();
    let mut bindings = Vec::new();

    if let Some(body) = &method.value.body {
        for statement in &body.statements {
            probe_statement_for_jsx(statement, &mut jsx_roots, &mut bindings);
        }
    }

    Some(MethodProbe {
        name,
        span: method.span,
        jsx_roots,
        bindings,
    })
}

fn probe_statement_for_jsx(
    statement: &Statement<'_>,
    jsx_roots: &mut Vec<JsxElementProbe>,
    bindings: &mut Vec<String>,
) {
    if let Statement::ReturnStatement(return_statement) = statement {
        if let Some(argument) = &return_statement.argument {
            probe_expression_for_jsx(argument, jsx_roots, bindings);
        }
    }
}

fn probe_expression_for_jsx(
    expression: &Expression<'_>,
    jsx_roots: &mut Vec<JsxElementProbe>,
    bindings: &mut Vec<String>,
) {
    match expression {
        Expression::ParenthesizedExpression(parenthesized) => {
            probe_expression_for_jsx(&parenthesized.expression, jsx_roots, bindings);
        }
        Expression::JSXElement(element) => {
            let name = jsx_element_name(&element.opening_element.name)
                .unwrap_or_else(|| "<unknown>".to_string());

            let attributes = element
                .opening_element
                .attributes
                .iter()
                .filter_map(jsx_attribute_name)
                .collect::<Vec<_>>();

            for child in &element.children {
                probe_jsx_child(child, bindings);
            }

            jsx_roots.push(JsxElementProbe {
                name,
                span: element.span,
                attributes,
            });
        }
        _ => {}
    }
}

fn probe_jsx_child(child: &JSXChild<'_>, bindings: &mut Vec<String>) {
    match child {
        JSXChild::ExpressionContainer(container) => {
            if let Some(binding) = jsx_expression_summary(&container.expression) {
                bindings.push(binding);
            }
        }
        JSXChild::Element(element) => {
            for child in &element.children {
                probe_jsx_child(child, bindings);
            }
        }
        _ => {}
    }
}

fn property_key_name(key: &PropertyKey<'_>) -> Option<String> {
    match key {
        PropertyKey::StaticIdentifier(identifier) => Some(identifier.name.to_string()),
        PropertyKey::PrivateIdentifier(identifier) => Some(format!("#{}", identifier.name)),
        PropertyKey::StringLiteral(literal) => Some(literal.value.to_string()),
        PropertyKey::NumericLiteral(literal) => Some(literal.raw.as_ref()?.to_string()),
        _ => None,
    }
}

fn jsx_expression_summary(expression: &JSXExpression<'_>) -> Option<String> {
    if let Some(expression) = expression.as_expression() {
        return expression_summary(expression);
    }

    None
}

fn expression_summary(expression: &Expression<'_>) -> Option<String> {
    match expression {
        Expression::CallExpression(call) => {
            let callee = expression_summary(&call.callee)?;
            Some(format!("{callee}(...)"))
        }
        Expression::Identifier(identifier) => Some(identifier.name.to_string()),
        Expression::ThisExpression(_) => Some("this".to_string()),
        Expression::StaticMemberExpression(member) => {
            let object = expression_summary(&member.object)?;
            Some(format!("{object}.{}", member.property.name))
        }
        Expression::NumericLiteral(literal) => Some(literal.raw.as_ref()?.to_string()),
        Expression::StringLiteral(literal) => Some(format!("{:?}", literal.value.as_str())),
        _ => None,
    }
}

fn jsx_element_name(name: &JSXElementName<'_>) -> Option<String> {
    match name {
        JSXElementName::Identifier(identifier) => Some(identifier.name.to_string()),
        JSXElementName::IdentifierReference(identifier) => Some(identifier.name.to_string()),
        JSXElementName::NamespacedName(namespaced) => Some(format!("{namespaced:?}")),
        JSXElementName::MemberExpression(member) => Some(format!("{member:?}")),
        JSXElementName::ThisExpression(_) => Some("this".to_string()),
    }
}

fn jsx_attribute_name(attribute: &JSXAttributeItem<'_>) -> Option<String> {
    let JSXAttributeItem::Attribute(attribute) = attribute else {
        return None;
    };

    let name = match &attribute.name {
        JSXAttributeName::Identifier(identifier) => identifier.name.to_string(),
        JSXAttributeName::NamespacedName(namespaced) => format!("{namespaced:?}"),
    };

    let value_suffix = match &attribute.value {
        Some(JSXAttributeValue::StringLiteral(literal)) => format!("={:?}", literal.value.as_str()),
        Some(JSXAttributeValue::ExpressionContainer(_)) => "={...}".to_string(),
        Some(_) => "=<complex>".to_string(),
        None => String::new(),
    };

    Some(format!("{name}{value_suffix}"))
}

fn line_column_at(source: &str, offset: usize) -> (usize, usize) {
    let clamped = offset.min(source.len());

    let prefix = &source[..clamped];

    let line = prefix.bytes().filter(|byte| *byte == b'\n').count() + 1;

    let column = prefix
        .rsplit_once('\n')
        .map_or(prefix.len() + 1, |(_, tail)| tail.len() + 1);

    (line, column)
}

fn print_diagnostic_probe(source: &str, errors: &[oxc_diagnostics::OxcDiagnostic]) {
    println!("Diagnostic probe:");

    for error in errors {
        println!("  message: {}", error.message);
        println!("  severity: {:?}", error.severity);

        match &error.labels {
            Some(labels) if !labels.is_empty() => {
                for label in labels {
                    let offset = label.offset();
                    let length = label.len();
                    let (line, column) = line_column_at(source, offset);
                    println!("  label: offset={offset} length={length} location={line}:{column}");
                }
            }
            _ => {
                println!("  labels: none");
            }
        }
    }
}

fn print_probe(probe: &ParserProbe) {
    println!("ParserProbe:");

    if probe.classes.is_empty() {
        println!("  classes: none");
        return;
    }

    for class in &probe.classes {
        println!("  class {} span={:?}", class.name, class.span);

        println!("    decorators:");
        if class.decorators.is_empty() {
            println!("      none");
        } else {
            for decorator in &class.decorators {
                match &decorator.argument {
                    Some(argument) => {
                        println!(
                            "      @{}({argument:?}) span={:?}",
                            decorator.name, decorator.span
                        );
                    }
                    None => {
                        println!("      @{} span={:?}", decorator.name, decorator.span);
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
                            "      {} = {} span={:?}",
                            property.name, initializer, property.span
                        );
                    }
                    None => {
                        println!("      {} span={:?}", property.name, property.span);
                    }
                }
            }
        }

        println!("    methods:");
        if class.methods.is_empty() {
            println!("      none");
        } else {
            for method in &class.methods {
                println!("      {} span={:?}", method.name, method.span);

                for jsx in &method.jsx_roots {
                    println!("        jsx root <{}> span={:?}", jsx.name, jsx.span);
                    if jsx.attributes.is_empty() {
                        println!("          attributes: none");
                    } else {
                        println!("          attributes: {}", jsx.attributes.join(", "));
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
