use std::path::{Path, PathBuf};

use oxc_allocator::Allocator;
use oxc_ast::ast::{
    Argument, ClassElement, Declaration, Expression, JSXAttributeItem, JSXAttributeName,
    JSXAttributeValue, JSXChild, JSXElementName, JSXExpression, Program, PropertyKey, Statement,
};
use oxc_diagnostics::Severity as OxcSeverity;
use oxc_parser::Parser;
use oxc_span::{SourceType, Span};

use crate::model::{
    ParseDiagnostic, ParseLabel, ParseSeverity, ParsedClass, ParsedDecorator, ParsedFile,
    ParsedJsxElement, ParsedMethod, ParsedProperty, SourceSpan,
};

pub fn parse_file(path: impl AsRef<Path>, source: &str) -> ParsedFile {
    let path = path.as_ref();
    let source_type = SourceType::from_path(path)
        .unwrap_or_default()
        .with_typescript(true)
        .with_jsx(true);

    let allocator = Allocator::default();
    let ret = Parser::new(&allocator, source, source_type).parse();

    let classes = parse_program(&ret.program, source);
    let diagnostics = ret
        .errors
        .iter()
        .map(|diagnostic| parse_diagnostic(source, diagnostic))
        .collect::<Vec<_>>();

    ParsedFile {
        path: PathBuf::from(path),
        classes,
        diagnostics,
    }
}

fn parse_program(program: &Program<'_>, source: &str) -> Vec<ParsedClass> {
    let mut classes = Vec::new();

    for statement in &program.body {
        if let Some(declaration) = statement.as_declaration() {
            if let Some(class) = parse_declaration(declaration, source) {
                classes.push(class);
            }
        }
    }

    classes
}

fn parse_declaration(declaration: &Declaration<'_>, source: &str) -> Option<ParsedClass> {
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
        .filter_map(|decorator| parse_decorator(decorator, source))
        .collect::<Vec<_>>();

    let mut properties = Vec::new();
    let mut methods = Vec::new();

    for element in &class.body.body {
        match element {
            ClassElement::PropertyDefinition(property) => {
                if let Some(property) = parse_property(property, source) {
                    properties.push(property);
                }
            }
            ClassElement::MethodDefinition(method) => {
                if let Some(method) = parse_method(method, source) {
                    methods.push(method);
                }
            }
            _ => {}
        }
    }

    Some(ParsedClass {
        name,
        span: source_span(source, class.span),
        decorators,
        properties,
        methods,
    })
}

fn parse_decorator(
    decorator: &oxc_ast::ast::Decorator<'_>,
    source: &str,
) -> Option<ParsedDecorator> {
    let Expression::CallExpression(call) = &decorator.expression else {
        return None;
    };

    let Expression::Identifier(callee) = &call.callee else {
        return None;
    };

    let argument = call.arguments.first().and_then(argument_string_value);

    Some(ParsedDecorator {
        name: callee.name.to_string(),
        argument,
        span: source_span(source, decorator.span),
    })
}

fn argument_string_value(argument: &Argument<'_>) -> Option<String> {
    match argument {
        Argument::StringLiteral(literal) => Some(literal.value.to_string()),
        _ => None,
    }
}

fn parse_property(
    property: &oxc_ast::ast::PropertyDefinition<'_>,
    source: &str,
) -> Option<ParsedProperty> {
    let name = property_key_name(&property.key)?;

    let initializer = property.value.as_ref().and_then(expression_summary);

    Some(ParsedProperty {
        name,
        initializer,
        span: source_span(source, property.span),
    })
}

fn parse_method(method: &oxc_ast::ast::MethodDefinition<'_>, source: &str) -> Option<ParsedMethod> {
    let name = property_key_name(&method.key)?;

    let mut jsx_roots = Vec::new();
    let mut bindings = Vec::new();

    if let Some(body) = &method.value.body {
        for statement in &body.statements {
            parse_statement_for_jsx(statement, source, &mut jsx_roots, &mut bindings);
        }
    }

    Some(ParsedMethod {
        name,
        span: source_span(source, method.span),
        jsx_roots,
        bindings,
    })
}

fn parse_statement_for_jsx(
    statement: &Statement<'_>,
    source: &str,
    jsx_roots: &mut Vec<ParsedJsxElement>,
    bindings: &mut Vec<String>,
) {
    if let Statement::ReturnStatement(return_statement) = statement {
        if let Some(argument) = &return_statement.argument {
            parse_expression_for_jsx(argument, source, jsx_roots, bindings);
        }
    }
}

fn parse_expression_for_jsx(
    expression: &Expression<'_>,
    source: &str,
    jsx_roots: &mut Vec<ParsedJsxElement>,
    bindings: &mut Vec<String>,
) {
    match expression {
        Expression::ParenthesizedExpression(parenthesized) => {
            parse_expression_for_jsx(&parenthesized.expression, source, jsx_roots, bindings);
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

            let event_handler_refs = element
                .opening_element
                .attributes
                .iter()
                .filter_map(jsx_event_handler_ref)
                .collect::<Vec<_>>();

            for child in &element.children {
                parse_jsx_child(child, bindings);
            }

            jsx_roots.push(ParsedJsxElement {
                name,
                span: source_span(source, element.span),
                attributes,
                event_handler_refs,
            });
        }
        _ => {}
    }
}

fn parse_jsx_child(child: &JSXChild<'_>, bindings: &mut Vec<String>) {
    match child {
        JSXChild::ExpressionContainer(container) => {
            if let Some(binding) = jsx_expression_summary(&container.expression) {
                bindings.push(binding);
            }
        }
        JSXChild::Element(element) => {
            for child in &element.children {
                parse_jsx_child(child, bindings);
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

fn jsx_event_handler_ref(attribute: &JSXAttributeItem<'_>) -> Option<String> {
    let JSXAttributeItem::Attribute(attribute) = attribute else {
        return None;
    };

    let attribute_name = match &attribute.name {
        JSXAttributeName::Identifier(identifier) => identifier.name.as_str(),
        JSXAttributeName::NamespacedName(_) => return None,
    };

    if !attribute_name.starts_with("on") {
        return None;
    }

    let Some(JSXAttributeValue::ExpressionContainer(container)) = &attribute.value else {
        return None;
    };

    jsx_expression_event_handler_ref(&container.expression)
}

fn jsx_expression_event_handler_ref(expression: &JSXExpression<'_>) -> Option<String> {
    let expression = expression.as_expression()?;
    expression_event_handler_ref(expression)
}

fn expression_event_handler_ref(expression: &Expression<'_>) -> Option<String> {
    match expression {
        Expression::ArrowFunctionExpression(arrow) => {
            for statement in &arrow.body.statements {
                if let Statement::ExpressionStatement(statement) = statement {
                    if let Some(reference) = expression_event_handler_ref(&statement.expression) {
                        return Some(reference);
                    }
                }
            }

            None
        }
        Expression::CallExpression(call) => expression_summary(&call.callee),
        _ => None,
    }
}

fn parse_diagnostic(source: &str, diagnostic: &oxc_diagnostics::OxcDiagnostic) -> ParseDiagnostic {
    let severity = match diagnostic.severity {
        OxcSeverity::Advice => ParseSeverity::Info,
        OxcSeverity::Warning => ParseSeverity::Warning,
        OxcSeverity::Error => ParseSeverity::Error,
    };

    let labels = diagnostic
        .labels
        .as_ref()
        .map(|labels| {
            labels
                .iter()
                .map(|label| {
                    let start = label.offset();
                    let end = start + label.len();

                    ParseLabel {
                        span: source_span_from_offsets(source, start, end),
                    }
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    ParseDiagnostic {
        message: diagnostic.message.to_string(),
        severity,
        labels,
    }
}

fn source_span(source: &str, span: Span) -> SourceSpan {
    source_span_from_offsets(source, span.start as usize, span.end as usize)
}

fn source_span_from_offsets(source: &str, start: usize, end: usize) -> SourceSpan {
    let (line, column) = line_column_at(source, start);

    SourceSpan {
        start,
        end,
        line,
        column,
    }
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
