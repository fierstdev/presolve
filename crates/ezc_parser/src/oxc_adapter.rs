use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use oxc_allocator::Allocator;
use oxc_ast::ast::{
    Argument, AssignmentTarget, BindingPatternKind, ClassElement, Declaration,
    ExportDefaultDeclarationKind, Expression, ImportDeclarationSpecifier, JSXAttributeItem,
    JSXAttributeName, JSXAttributeValue, JSXChild, JSXElementName, JSXExpression, JSXFragment,
    ModuleExportName, ObjectPropertyKind, Program, PropertyKey, PropertyKind,
    SimpleAssignmentTarget, Statement,
};
use oxc_diagnostics::Severity as OxcSeverity;
use oxc_parser::Parser;
use oxc_span::{GetSpan, SourceType, Span};

use crate::model::{
    ParseDiagnostic, ParseLabel, ParseSeverity, ParsedArithmeticExpression,
    ParsedArithmeticExpressionKind, ParsedArithmeticOperator, ParsedClass,
    ParsedComparisonOperator, ParsedConstantExpression, ParsedConstantExpressionKind,
    ParsedDecorator, ParsedEventHandler, ParsedExport, ParsedExportKind, ParsedExportSpecifier,
    ParsedFile, ParsedImport, ParsedImportSpecifier, ParsedJsxAttribute, ParsedJsxAttributeValue,
    ParsedJsxChild, ParsedJsxConditional, ParsedJsxElement, ParsedJsxFragment, ParsedJsxList,
    ParsedJsxNode, ParsedLocalVariable, ParsedLogicalOperator, ParsedMethod, ParsedMethodParameter,
    ParsedProperty, ParsedSerializableValue, ParsedStateOperation, ParsedStateUpdate,
    ParsedTypeAlias, ParsedTypeAnnotation, ParsedUnaryOperator, SourceSpan,
};

pub fn parse_file(path: impl AsRef<Path>, source: &str) -> ParsedFile {
    let path = path.as_ref();
    let source_type = SourceType::from_path(path)
        .unwrap_or_default()
        .with_typescript(true)
        .with_jsx(true);

    let allocator = Allocator::default();
    let ret = Parser::new(&allocator, source, source_type).parse();

    let (classes, type_aliases, imports, exports) = parse_program(&ret.program, source);
    let diagnostics = ret
        .errors
        .iter()
        .map(|diagnostic| parse_diagnostic(source, diagnostic))
        .collect::<Vec<_>>();

    ParsedFile {
        path: PathBuf::from(path),
        classes,
        type_aliases,
        imports,
        exports,
        diagnostics,
    }
}

fn parse_program(
    program: &Program<'_>,
    source: &str,
) -> (
    Vec<ParsedClass>,
    Vec<ParsedTypeAlias>,
    Vec<ParsedImport>,
    Vec<ParsedExport>,
) {
    let mut classes = Vec::new();
    let mut type_aliases = Vec::new();
    let mut imports = Vec::new();
    let mut exports = Vec::new();

    for statement in &program.body {
        match statement {
            Statement::ImportDeclaration(declaration) => {
                imports.push(parse_import_declaration(declaration, source));
                continue;
            }
            Statement::ExportNamedDeclaration(declaration) => {
                exports.push(parse_named_export_declaration(declaration, source));
                if let Some(declaration) = &declaration.declaration {
                    if let Some(class) = parse_declaration(declaration, source) {
                        classes.push(class);
                    }
                    if let Some(alias) = parse_type_alias_declaration(declaration, source) {
                        type_aliases.push(alias);
                    }
                }
                continue;
            }
            Statement::ExportDefaultDeclaration(declaration) => {
                exports.push(parse_default_export_declaration(declaration, source));
                if let ExportDefaultDeclarationKind::ClassDeclaration(class) =
                    &declaration.declaration
                {
                    if let Some(class) = parse_class(class, source) {
                        classes.push(class);
                    }
                }
                continue;
            }
            Statement::ExportAllDeclaration(declaration) => {
                exports.push(ParsedExport {
                    kind: ParsedExportKind::All,
                    source: Some(declaration.source.value.to_string()),
                    specifiers: declaration
                        .exported
                        .as_ref()
                        .map(|exported| {
                            vec![ParsedExportSpecifier {
                                local: None,
                                exported: module_export_name(exported),
                            }]
                        })
                        .unwrap_or_default(),
                    span: source_span(source, declaration.span),
                });
                continue;
            }
            _ => {}
        }

        if let Some(declaration) = statement.as_declaration() {
            if let Some(class) = parse_declaration(declaration, source) {
                classes.push(class);
            }
            if let Some(alias) = parse_type_alias_declaration(declaration, source) {
                type_aliases.push(alias);
            }
        }
    }

    (classes, type_aliases, imports, exports)
}

fn parse_type_alias_declaration(
    declaration: &Declaration<'_>,
    source: &str,
) -> Option<ParsedTypeAlias> {
    let Declaration::TSTypeAliasDeclaration(alias) = declaration else {
        return None;
    };
    let type_span = source_span(source, alias.type_annotation.span());

    Some(ParsedTypeAlias {
        name: alias.id.name.to_string(),
        type_text: source[type_span.start..type_span.end].trim().to_string(),
        span: source_span(source, alias.span),
        type_span,
    })
}

fn parse_import_declaration(
    declaration: &oxc_ast::ast::ImportDeclaration<'_>,
    source: &str,
) -> ParsedImport {
    let specifiers = declaration
        .specifiers
        .as_ref()
        .map(|specifiers| {
            specifiers
                .iter()
                .map(|specifier| match specifier {
                    ImportDeclarationSpecifier::ImportSpecifier(specifier) => {
                        ParsedImportSpecifier {
                            imported: module_export_name(&specifier.imported),
                            local: specifier.local.name.to_string(),
                        }
                    }
                    ImportDeclarationSpecifier::ImportDefaultSpecifier(specifier) => {
                        ParsedImportSpecifier {
                            imported: "default".to_string(),
                            local: specifier.local.name.to_string(),
                        }
                    }
                    ImportDeclarationSpecifier::ImportNamespaceSpecifier(specifier) => {
                        ParsedImportSpecifier {
                            imported: "*".to_string(),
                            local: specifier.local.name.to_string(),
                        }
                    }
                })
                .collect()
        })
        .unwrap_or_default();

    ParsedImport {
        source: declaration.source.value.to_string(),
        specifiers,
        span: source_span(source, declaration.span),
    }
}

fn parse_named_export_declaration(
    declaration: &oxc_ast::ast::ExportNamedDeclaration<'_>,
    source: &str,
) -> ParsedExport {
    let mut specifiers = declaration
        .specifiers
        .iter()
        .map(|specifier| ParsedExportSpecifier {
            local: Some(module_export_name(&specifier.local)),
            exported: module_export_name(&specifier.exported),
        })
        .collect::<Vec<_>>();

    if let Some(declaration) = &declaration.declaration {
        specifiers.extend(named_declaration_exports(declaration));
    }

    ParsedExport {
        kind: ParsedExportKind::Named,
        source: declaration
            .source
            .as_ref()
            .map(|source| source.value.to_string()),
        specifiers,
        span: source_span(source, declaration.span),
    }
}

fn parse_default_export_declaration(
    declaration: &oxc_ast::ast::ExportDefaultDeclaration<'_>,
    source: &str,
) -> ParsedExport {
    ParsedExport {
        kind: ParsedExportKind::Default,
        source: None,
        specifiers: vec![ParsedExportSpecifier {
            local: default_declaration_name(&declaration.declaration),
            exported: "default".to_string(),
        }],
        span: source_span(source, declaration.span),
    }
}

fn named_declaration_exports(declaration: &Declaration<'_>) -> Vec<ParsedExportSpecifier> {
    let name = match declaration {
        Declaration::ClassDeclaration(class) => class.id.as_ref().map(|id| id.name.to_string()),
        Declaration::FunctionDeclaration(function) => {
            function.id.as_ref().map(|id| id.name.to_string())
        }
        Declaration::TSTypeAliasDeclaration(alias) => Some(alias.id.name.to_string()),
        Declaration::VariableDeclaration(declaration) => {
            let names = declaration
                .declarations
                .iter()
                .filter_map(|declarator| binding_identifier_name(&declarator.id.kind))
                .collect::<Vec<_>>();

            return names
                .into_iter()
                .map(|name| ParsedExportSpecifier {
                    local: Some(name.clone()),
                    exported: name,
                })
                .collect();
        }
        _ => None,
    };

    name.map(|name| {
        vec![ParsedExportSpecifier {
            local: Some(name.clone()),
            exported: name,
        }]
    })
    .unwrap_or_default()
}

fn default_declaration_name(declaration: &ExportDefaultDeclarationKind<'_>) -> Option<String> {
    match declaration {
        ExportDefaultDeclarationKind::ClassDeclaration(class) => {
            class.id.as_ref().map(|id| id.name.to_string())
        }
        ExportDefaultDeclarationKind::FunctionDeclaration(function) => {
            function.id.as_ref().map(|id| id.name.to_string())
        }
        _ => None,
    }
}

fn module_export_name(name: &ModuleExportName<'_>) -> String {
    match name {
        ModuleExportName::IdentifierName(name) => name.name.to_string(),
        ModuleExportName::IdentifierReference(name) => name.name.to_string(),
        ModuleExportName::StringLiteral(name) => name.value.to_string(),
    }
}

fn parse_declaration(declaration: &Declaration<'_>, source: &str) -> Option<ParsedClass> {
    let Declaration::ClassDeclaration(class) = declaration else {
        return None;
    };

    parse_class(class, source)
}

fn parse_class(class: &oxc_ast::ast::Class<'_>, source: &str) -> Option<ParsedClass> {
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

    let state_initial_value = property.value.as_ref().and_then(state_initial_value);
    let state_initial_expression = property
        .value
        .as_ref()
        .and_then(|expression| state_initial_constant_expression(expression, source));
    let state_type_annotation = (initializer.as_deref() == Some("state(...)"))
        .then_some(property.type_annotation.as_ref())
        .flatten()
        .map(|annotation| parsed_type_annotation(annotation.span, source));

    Some(ParsedProperty {
        name,
        initializer,
        state_initial_value,
        state_initial_expression,
        state_type_annotation,
        span: source_span(source, property.span),
    })
}

fn parsed_type_annotation(span: Span, source: &str) -> ParsedTypeAnnotation {
    let span = source_span(source, span);
    let text = source[span.start..span.end]
        .strip_prefix(':')
        .expect("TypeScript annotation span should start with a colon")
        .trim()
        .to_string();

    ParsedTypeAnnotation { text, span }
}

fn parse_method(method: &oxc_ast::ast::MethodDefinition<'_>, source: &str) -> Option<ParsedMethod> {
    let name = property_key_name(&method.key)?;

    let mut jsx_roots = Vec::new();
    let mut bindings = Vec::new();
    let mut state_updates = Vec::new();
    let mut local_variables = Vec::new();
    let parameters = method
        .value
        .params
        .items
        .iter()
        .filter_map(|parameter| {
            Some(ParsedMethodParameter {
                name: binding_identifier_name(&parameter.pattern.kind)?,
                span: source_span(source, parameter.span),
            })
        })
        .collect();

    if let Some(body) = &method.value.body {
        for statement in &body.statements {
            parse_statement_for_jsx(statement, source, &mut jsx_roots, &mut bindings);
            if let Some(update) = parsed_state_update(statement, source) {
                state_updates.push(update);
            }
            local_variables.extend(parsed_local_variables(statement, source));
        }
    }

    Some(ParsedMethod {
        name,
        span: source_span(source, method.span),
        jsx_roots,
        bindings,
        state_updates,
        local_variables,
        parameters,
    })
}

fn parsed_local_variables(statement: &Statement<'_>, source: &str) -> Vec<ParsedLocalVariable> {
    let Some(Declaration::VariableDeclaration(declaration)) = statement.as_declaration() else {
        return Vec::new();
    };
    declaration
        .declarations
        .iter()
        .filter_map(|declarator| {
            let name = binding_identifier_name(&declarator.id.kind)?;
            let value = serializable_value_from_expression(declarator.init.as_ref()?)?;
            Some(ParsedLocalVariable {
                name,
                value,
                span: source_span(source, declarator.span),
            })
        })
        .collect()
}

fn parsed_state_update(statement: &Statement<'_>, source: &str) -> Option<ParsedStateUpdate> {
    let Statement::ExpressionStatement(statement) = statement else {
        return None;
    };

    match &statement.expression {
        Expression::UpdateExpression(update) => parsed_update_state_update(update, source),
        Expression::AssignmentExpression(assignment) => {
            parsed_assignment_state_update(assignment, source)
        }
        _ => None,
    }
}

fn parsed_update_state_update(
    update: &oxc_ast::ast::UpdateExpression<'_>,
    source: &str,
) -> Option<ParsedStateUpdate> {
    let operation = match update.operator.as_str() {
        "++" => ParsedStateOperation::Increment,
        "--" => ParsedStateOperation::Decrement,
        _ => return None,
    };

    let field = this_assignment_target_field(&update.argument)?;

    Some(ParsedStateUpdate {
        field,
        operation,
        span: source_span(source, update.span),
    })
}

fn parsed_assignment_state_update(
    assignment: &oxc_ast::ast::AssignmentExpression<'_>,
    source: &str,
) -> Option<ParsedStateUpdate> {
    let field = this_assignment_target_field_from_assignment_target(&assignment.left)?;

    let operation = match assignment.operator.as_str() {
        "+=" => {
            ParsedStateOperation::AddAssign(serializable_value_from_expression(&assignment.right)?)
        }
        "-=" => ParsedStateOperation::SubtractAssign(serializable_value_from_expression(
            &assignment.right,
        )?),
        "=" if toggled_this_field(&assignment.right).as_deref() == Some(field.as_str()) => {
            ParsedStateOperation::Toggle
        }
        "=" => ParsedStateOperation::Assign(serializable_value_from_expression(&assignment.right)?),
        _ => return None,
    };

    Some(ParsedStateUpdate {
        field,
        operation,
        span: source_span(source, assignment.span),
    })
}

fn this_assignment_target_field(target: &SimpleAssignmentTarget<'_>) -> Option<String> {
    let SimpleAssignmentTarget::StaticMemberExpression(member) = target else {
        return None;
    };

    let Expression::ThisExpression(_) = &member.object else {
        return None;
    };

    Some(member.property.name.to_string())
}

fn this_assignment_target_field_from_assignment_target(
    target: &AssignmentTarget<'_>,
) -> Option<String> {
    let AssignmentTarget::StaticMemberExpression(member) = target else {
        return None;
    };

    let Expression::ThisExpression(_) = &member.object else {
        return None;
    };

    Some(member.property.name.to_string())
}

fn parse_statement_for_jsx(
    statement: &Statement<'_>,
    source: &str,
    jsx_roots: &mut Vec<ParsedJsxNode>,
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
    jsx_roots: &mut Vec<ParsedJsxNode>,
    bindings: &mut Vec<String>,
) {
    match expression {
        Expression::ParenthesizedExpression(parenthesized) => {
            parse_expression_for_jsx(&parenthesized.expression, source, jsx_roots, bindings);
        }
        Expression::JSXElement(element) => {
            for child in &element.children {
                parse_jsx_child(child, bindings);
            }

            if let Some(element) = parsed_jsx_element(element, source) {
                jsx_roots.push(ParsedJsxNode::Element(element));
            }
        }
        Expression::JSXFragment(fragment) => {
            for child in &fragment.children {
                parse_jsx_child(child, bindings);
            }

            jsx_roots.push(ParsedJsxNode::Fragment(parsed_jsx_fragment(
                fragment, source,
            )));
        }
        _ => {}
    }
}

fn parse_jsx_child(child: &JSXChild<'_>, bindings: &mut Vec<String>) {
    match child {
        JSXChild::ExpressionContainer(container) => {
            if let Some(binding) = jsx_expression_binding_summary(&container.expression) {
                bindings.push(binding);
            }
        }
        JSXChild::Element(element) => {
            for child in &element.children {
                parse_jsx_child(child, bindings);
            }
        }
        JSXChild::Fragment(fragment) => {
            for child in &fragment.children {
                parse_jsx_child(child, bindings);
            }
        }
        _ => {}
    }
}

fn parsed_jsx_child(child: &JSXChild<'_>, source: &str) -> Option<ParsedJsxChild> {
    match child {
        JSXChild::Text(text) => {
            let normalized = normalize_jsx_text(&text.value);

            if normalized.is_empty() {
                None
            } else {
                Some(ParsedJsxChild::Text {
                    value: normalized,
                    span: jsx_text_value_span(source, text.value.as_str(), text.span),
                })
            }
        }
        JSXChild::ExpressionContainer(container) => parsed_jsx_expression_child(
            &container.expression,
            source,
            source_span(source, container.span),
        ),
        JSXChild::Element(element) => {
            parsed_jsx_element(element, source).map(ParsedJsxChild::Element)
        }
        JSXChild::Fragment(fragment) => Some(ParsedJsxChild::Fragment(parsed_jsx_fragment(
            fragment, source,
        ))),
        _ => None,
    }
}

fn parsed_jsx_expression_child(
    expression: &JSXExpression<'_>,
    source: &str,
    span: SourceSpan,
) -> Option<ParsedJsxChild> {
    let expression = expression.as_expression()?;

    if let Expression::CallExpression(call) = expression {
        if let Some(list) = parsed_jsx_list(call, source) {
            return Some(ParsedJsxChild::List(list));
        }
    }

    if let Expression::ConditionalExpression(conditional) = expression {
        return parsed_jsx_conditional(conditional, source).map(ParsedJsxChild::Conditional);
    }

    if let Expression::LogicalExpression(logical) = expression {
        return parsed_jsx_logical_and(logical, source).map(ParsedJsxChild::Conditional);
    }

    expression_summary(expression).map(|expression| ParsedJsxChild::Binding { expression, span })
}

fn parsed_jsx_list(call: &oxc_ast::ast::CallExpression<'_>, source: &str) -> Option<ParsedJsxList> {
    let Expression::StaticMemberExpression(member) = &call.callee else {
        return None;
    };

    if member.property.name != "map" || call.arguments.len() != 1 {
        return None;
    }

    let iterable = expression_summary(&member.object)?;
    let Argument::ArrowFunctionExpression(callback) = &call.arguments[0] else {
        return None;
    };

    if callback.params.rest.is_some() || !(1..=2).contains(&callback.params.items.len()) {
        return None;
    }

    let item_variable = binding_identifier_name(&callback.params.items[0].pattern.kind)?;
    let index_variable = match callback.params.items.get(1) {
        Some(parameter) => Some(binding_identifier_name(&parameter.pattern.kind)?),
        None => None,
    };
    let item_template = parsed_jsx_node_from_expression(callback.get_expression()?, source)?;
    let key_expression = key_expression_from_jsx_node(&item_template).unwrap_or_default();

    Some(ParsedJsxList {
        iterable,
        item_variable,
        index_variable,
        key_expression,
        span: source_span(source, call.span),
        item_template,
    })
}

fn binding_identifier_name(pattern: &BindingPatternKind<'_>) -> Option<String> {
    let BindingPatternKind::BindingIdentifier(identifier) = pattern else {
        return None;
    };

    Some(identifier.name.to_string())
}

fn key_expression_from_jsx_node(node: &ParsedJsxNode) -> Option<String> {
    let ParsedJsxNode::Element(element) = node else {
        return None;
    };

    element.attributes.iter().find_map(|attribute| {
        if attribute.name != "key" {
            return None;
        }

        let ParsedJsxAttributeValue::Expression(Some(expression)) = &attribute.value else {
            return None;
        };

        Some(expression.clone())
    })
}

fn parsed_jsx_conditional(
    conditional: &oxc_ast::ast::ConditionalExpression<'_>,
    source: &str,
) -> Option<ParsedJsxConditional> {
    let condition = expression_summary(&conditional.test)?;
    let when_true = parsed_jsx_node_from_expression(&conditional.consequent, source)?;
    let when_false = parsed_jsx_node_from_expression(&conditional.alternate, source)?;

    Some(ParsedJsxConditional {
        condition,
        span: source_span(source, conditional.span),
        when_true,
        when_false: Some(when_false),
    })
}

fn parsed_jsx_logical_and(
    logical: &oxc_ast::ast::LogicalExpression<'_>,
    source: &str,
) -> Option<ParsedJsxConditional> {
    if logical.operator.as_str() != "&&" {
        return None;
    }

    let condition = expression_summary(&logical.left)?;
    let when_true = parsed_jsx_node_from_expression(&logical.right, source)?;

    Some(ParsedJsxConditional {
        condition,
        span: source_span(source, logical.span),
        when_true,
        when_false: None,
    })
}

fn parsed_jsx_node_from_expression(
    expression: &Expression<'_>,
    source: &str,
) -> Option<ParsedJsxNode> {
    match expression {
        Expression::ParenthesizedExpression(parenthesized) => {
            parsed_jsx_node_from_expression(&parenthesized.expression, source)
        }
        Expression::JSXElement(element) => {
            parsed_jsx_element(element, source).map(ParsedJsxNode::Element)
        }
        Expression::JSXFragment(fragment) => Some(ParsedJsxNode::Fragment(parsed_jsx_fragment(
            fragment, source,
        ))),
        _ => None,
    }
}

fn parsed_jsx_fragment(fragment: &JSXFragment<'_>, source: &str) -> ParsedJsxFragment {
    ParsedJsxFragment {
        span: source_span(source, fragment.span),
        children: fragment
            .children
            .iter()
            .filter_map(|child| parsed_jsx_child(child, source))
            .collect(),
    }
}

fn parsed_jsx_element(
    element: &oxc_ast::ast::JSXElement<'_>,
    source: &str,
) -> Option<ParsedJsxElement> {
    let name =
        jsx_element_name(&element.opening_element.name).unwrap_or_else(|| "<unknown>".to_string());

    let attributes = element
        .opening_element
        .attributes
        .iter()
        .map(|attribute| parsed_jsx_attribute(attribute, source))
        .collect::<Vec<_>>();

    let event_handlers = element
        .opening_element
        .attributes
        .iter()
        .filter_map(|attribute| jsx_event_handler(attribute, source))
        .collect::<Vec<_>>();

    let children = element
        .children
        .iter()
        .filter_map(|child| parsed_jsx_child(child, source))
        .collect::<Vec<_>>();

    Some(ParsedJsxElement {
        name,
        span: source_span(source, element.span),
        attributes,
        event_handlers,
        children,
    })
}

fn normalize_jsx_text(value: &str) -> String {
    value.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn jsx_text_value_span(source: &str, value: &str, span: Span) -> SourceSpan {
    let leading_whitespace = value
        .char_indices()
        .find(|(_, character)| !character.is_whitespace())
        .map_or(value.len(), |(index, _)| index);
    let trailing_whitespace = value
        .char_indices()
        .rev()
        .find(|(_, character)| !character.is_whitespace())
        .map_or(value.len(), |(index, character)| {
            index + character.len_utf8()
        });

    source_span_from_offsets(
        source,
        span.start as usize + leading_whitespace,
        span.start as usize + trailing_whitespace,
    )
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

fn jsx_expression_binding_summary(expression: &JSXExpression<'_>) -> Option<String> {
    let expression = expression.as_expression()?;

    if let Expression::ConditionalExpression(conditional) = expression {
        return expression_summary(&conditional.test);
    }

    if let Expression::LogicalExpression(logical) = expression {
        if logical.operator.as_str() == "&&" {
            return expression_summary(&logical.left);
        }
    }

    if let Expression::CallExpression(call) = expression {
        if let Some(iterable) = list_iterable_dependency(call) {
            return Some(iterable);
        }
    }

    expression_summary(expression)
}

fn list_iterable_dependency(call: &oxc_ast::ast::CallExpression<'_>) -> Option<String> {
    let Expression::StaticMemberExpression(member) = &call.callee else {
        return None;
    };

    if member.property.name != "map" {
        return None;
    }

    expression_summary(&member.object)
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

fn state_initial_value(expression: &Expression<'_>) -> Option<ParsedSerializableValue> {
    let Expression::CallExpression(call) = expression else {
        return None;
    };

    let Expression::Identifier(callee) = &call.callee else {
        return None;
    };

    if callee.name != "state" {
        return None;
    }

    call.arguments.first().and_then(state_argument_literal)
}

fn state_initial_constant_expression(
    expression: &Expression<'_>,
    source: &str,
) -> Option<ParsedConstantExpression> {
    let Expression::CallExpression(call) = expression else {
        return None;
    };
    let Expression::Identifier(callee) = &call.callee else {
        return None;
    };
    if callee.name != "state" || call.arguments.len() != 1 {
        return None;
    }

    let expression = parsed_constant_expression(call.arguments[0].as_expression()?, source)?;
    (!matches!(
        expression.kind,
        ParsedConstantExpressionKind::Primitive(_) | ParsedConstantExpressionKind::Boolean(_)
    ))
    .then_some(expression)
}

fn parsed_constant_expression(
    expression: &Expression<'_>,
    source: &str,
) -> Option<ParsedConstantExpression> {
    if let Expression::ParenthesizedExpression(parenthesized) = expression {
        return parsed_constant_expression(&parenthesized.expression, source);
    }
    if let Expression::BooleanLiteral(literal) = expression {
        return Some(ParsedConstantExpression {
            kind: ParsedConstantExpressionKind::Boolean(literal.value),
            span: source_span(source, literal.span),
        });
    }
    if let Some(primitive) = parsed_primitive_constant_expression(expression, source) {
        return Some(primitive);
    }
    if let Some(unary) = parsed_unary_constant_expression(expression, source) {
        return Some(unary);
    }
    if let Some(logical) = parsed_logical_expression(expression, source) {
        return Some(logical);
    }
    if let Some(comparison) = parsed_comparison_expression(expression, source) {
        return Some(comparison);
    }
    if let Some(nullish) = parsed_nullish_coalescing_expression(expression, source) {
        return Some(nullish);
    }

    let arithmetic = parsed_arithmetic_expression(expression, source)?;
    matches!(
        arithmetic.kind,
        ParsedArithmeticExpressionKind::Binary { .. }
    )
    .then_some(ParsedConstantExpression {
        span: arithmetic.span,
        kind: ParsedConstantExpressionKind::Arithmetic(arithmetic),
    })
}

fn parsed_unary_constant_expression(
    expression: &Expression<'_>,
    source: &str,
) -> Option<ParsedConstantExpression> {
    let Expression::UnaryExpression(unary) = expression else {
        return None;
    };
    let operator = match unary.operator.as_str() {
        "!" => ParsedUnaryOperator::Not,
        "+" => ParsedUnaryOperator::Plus,
        "-" => ParsedUnaryOperator::Minus,
        _ => return None,
    };
    let operand = parsed_constant_expression(&unary.argument, source)?;
    let valid = match operator {
        ParsedUnaryOperator::Not => matches!(
            operand.kind,
            ParsedConstantExpressionKind::Boolean(_)
                | ParsedConstantExpressionKind::Comparison { .. }
                | ParsedConstantExpressionKind::Logical { .. }
        ),
        ParsedUnaryOperator::Plus | ParsedUnaryOperator::Minus => matches!(
            operand.kind,
            ParsedConstantExpressionKind::Primitive(ParsedSerializableValue::Number(_))
                | ParsedConstantExpressionKind::Arithmetic(_)
        ),
    };
    valid.then_some(ParsedConstantExpression {
        kind: ParsedConstantExpressionKind::Unary {
            operator,
            operand: Box::new(operand),
        },
        span: source_span(source, unary.span),
    })
}

fn parsed_primitive_constant_expression(
    expression: &Expression<'_>,
    source: &str,
) -> Option<ParsedConstantExpression> {
    let (value, span) = match expression {
        Expression::NullLiteral(literal) => (ParsedSerializableValue::Null, literal.span),
        Expression::NumericLiteral(literal) => (
            ParsedSerializableValue::Number(literal.raw.as_ref()?.to_string()),
            literal.span,
        ),
        Expression::StringLiteral(literal) => (
            ParsedSerializableValue::String(literal.value.to_string()),
            literal.span,
        ),
        _ => return None,
    };

    Some(ParsedConstantExpression {
        kind: ParsedConstantExpressionKind::Primitive(value),
        span: source_span(source, span),
    })
}

fn parsed_logical_expression(
    expression: &Expression<'_>,
    source: &str,
) -> Option<ParsedConstantExpression> {
    let Expression::LogicalExpression(logical) = expression else {
        return None;
    };
    let operator = match logical.operator.as_str() {
        "&&" => ParsedLogicalOperator::And,
        "||" => ParsedLogicalOperator::Or,
        _ => return None,
    };

    Some(ParsedConstantExpression {
        kind: ParsedConstantExpressionKind::Logical {
            operator,
            left: Box::new(parsed_boolean_constant_expression(&logical.left, source)?),
            right: Box::new(parsed_boolean_constant_expression(&logical.right, source)?),
        },
        span: source_span(source, logical.span),
    })
}

fn parsed_nullish_coalescing_expression(
    expression: &Expression<'_>,
    source: &str,
) -> Option<ParsedConstantExpression> {
    let Expression::LogicalExpression(logical) = expression else {
        return None;
    };
    if logical.operator.as_str() != "??" {
        return None;
    }

    Some(ParsedConstantExpression {
        kind: ParsedConstantExpressionKind::NullishCoalescing {
            left: Box::new(parsed_nullish_constant_expression(&logical.left, source)?),
            right: Box::new(parsed_nullish_constant_expression(&logical.right, source)?),
        },
        span: source_span(source, logical.span),
    })
}

fn parsed_boolean_constant_expression(
    expression: &Expression<'_>,
    source: &str,
) -> Option<ParsedConstantExpression> {
    let expression = parsed_constant_expression(expression, source)?;
    matches!(
        expression.kind,
        ParsedConstantExpressionKind::Boolean(_)
            | ParsedConstantExpressionKind::Comparison { .. }
            | ParsedConstantExpressionKind::Logical { .. }
    )
    .then_some(expression)
}

fn parsed_nullish_constant_expression(
    expression: &Expression<'_>,
    source: &str,
) -> Option<ParsedConstantExpression> {
    let expression = parsed_constant_expression(expression, source)?;
    matches!(
        expression.kind,
        ParsedConstantExpressionKind::Primitive(_)
            | ParsedConstantExpressionKind::Boolean(_)
            | ParsedConstantExpressionKind::Arithmetic(_)
            | ParsedConstantExpressionKind::Comparison { .. }
            | ParsedConstantExpressionKind::Logical { .. }
            | ParsedConstantExpressionKind::NullishCoalescing { .. }
    )
    .then_some(expression)
}

fn parsed_comparison_expression(
    expression: &Expression<'_>,
    source: &str,
) -> Option<ParsedConstantExpression> {
    if let Expression::ParenthesizedExpression(parenthesized) = expression {
        return parsed_comparison_expression(&parenthesized.expression, source);
    }
    let Expression::BinaryExpression(binary) = expression else {
        return None;
    };
    let operator = match binary.operator.as_str() {
        "===" => ParsedComparisonOperator::Equal,
        "!==" => ParsedComparisonOperator::NotEqual,
        "<" => ParsedComparisonOperator::LessThan,
        "<=" => ParsedComparisonOperator::LessThanOrEqual,
        ">" => ParsedComparisonOperator::GreaterThan,
        ">=" => ParsedComparisonOperator::GreaterThanOrEqual,
        _ => return None,
    };

    Some(ParsedConstantExpression {
        kind: ParsedConstantExpressionKind::Comparison {
            operator,
            left: parsed_arithmetic_expression(&binary.left, source)?,
            right: parsed_arithmetic_expression(&binary.right, source)?,
        },
        span: source_span(source, binary.span),
    })
}

fn parsed_arithmetic_expression(
    expression: &Expression<'_>,
    source: &str,
) -> Option<ParsedArithmeticExpression> {
    match expression {
        Expression::ParenthesizedExpression(parenthesized) => {
            parsed_arithmetic_expression(&parenthesized.expression, source)
        }
        Expression::NumericLiteral(literal) => Some(ParsedArithmeticExpression {
            kind: ParsedArithmeticExpressionKind::Number(literal.raw.as_ref()?.to_string()),
            span: source_span(source, literal.span),
        }),
        Expression::BinaryExpression(binary) => {
            let operator = match binary.operator.as_str() {
                "+" => ParsedArithmeticOperator::Add,
                "-" => ParsedArithmeticOperator::Subtract,
                "*" => ParsedArithmeticOperator::Multiply,
                "/" => ParsedArithmeticOperator::Divide,
                "%" => ParsedArithmeticOperator::Remainder,
                _ => return None,
            };
            let left = parsed_arithmetic_expression(&binary.left, source)?;
            let right = parsed_arithmetic_expression(&binary.right, source)?;

            Some(ParsedArithmeticExpression {
                kind: ParsedArithmeticExpressionKind::Binary {
                    operator,
                    left: Box::new(left),
                    right: Box::new(right),
                },
                span: source_span(source, binary.span),
            })
        }
        _ => None,
    }
}

fn state_argument_literal(argument: &Argument<'_>) -> Option<ParsedSerializableValue> {
    serializable_value_from_expression(argument.as_expression()?)
}

fn serializable_value_from_expression(
    expression: &Expression<'_>,
) -> Option<ParsedSerializableValue> {
    match expression {
        Expression::NullLiteral(_) => Some(ParsedSerializableValue::Null),
        Expression::NumericLiteral(literal) => literal
            .raw
            .as_ref()
            .map(ToString::to_string)
            .map(ParsedSerializableValue::Number),
        Expression::StringLiteral(literal) => {
            Some(ParsedSerializableValue::String(literal.value.to_string()))
        }
        Expression::BooleanLiteral(literal) => {
            Some(ParsedSerializableValue::Boolean(literal.value))
        }
        Expression::ArrayExpression(array) => array
            .elements
            .iter()
            .map(|element| serializable_value_from_expression(element.as_expression()?))
            .collect::<Option<Vec<_>>>()
            .map(ParsedSerializableValue::Array),
        Expression::ObjectExpression(object) => object
            .properties
            .iter()
            .map(|property| {
                let ObjectPropertyKind::ObjectProperty(property) = property else {
                    return None;
                };

                if property.computed || property.method || property.kind != PropertyKind::Init {
                    return None;
                }

                let key = object_property_key_name(&property.key)?;
                let value = serializable_value_from_expression(&property.value)?;
                Some((key, value))
            })
            .collect::<Option<BTreeMap<_, _>>>()
            .map(ParsedSerializableValue::Object),
        _ => None,
    }
}

fn object_property_key_name(key: &PropertyKey<'_>) -> Option<String> {
    match key {
        PropertyKey::StaticIdentifier(_)
        | PropertyKey::StringLiteral(_)
        | PropertyKey::NumericLiteral(_) => property_key_name(key),
        PropertyKey::PrivateIdentifier(_) => None,
        _ => None,
    }
}

fn toggled_this_field(expression: &Expression<'_>) -> Option<String> {
    let Expression::UnaryExpression(unary) = expression else {
        return None;
    };

    if unary.operator.as_str() != "!" {
        return None;
    }

    this_member_expression_field(&unary.argument)
}

fn this_member_expression_field(expression: &Expression<'_>) -> Option<String> {
    let Expression::StaticMemberExpression(member) = expression else {
        return None;
    };

    let Expression::ThisExpression(_) = &member.object else {
        return None;
    };

    Some(member.property.name.to_string())
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

fn parsed_jsx_attribute(attribute: &JSXAttributeItem<'_>, source: &str) -> ParsedJsxAttribute {
    match attribute {
        JSXAttributeItem::Attribute(attribute) => {
            let name = jsx_attribute_name(&attribute.name);
            let value = match &attribute.value {
                Some(JSXAttributeValue::StringLiteral(literal)) => {
                    ParsedJsxAttributeValue::Static(literal.value.to_string())
                }
                Some(JSXAttributeValue::ExpressionContainer(container)) => {
                    ParsedJsxAttributeValue::Expression(jsx_expression_summary(
                        &container.expression,
                    ))
                }
                Some(JSXAttributeValue::Element(_)) | Some(JSXAttributeValue::Fragment(_)) => {
                    ParsedJsxAttributeValue::Unsupported
                }
                None => ParsedJsxAttributeValue::Boolean,
            };

            ParsedJsxAttribute {
                name,
                value,
                span: source_span(source, attribute.span),
            }
        }
        JSXAttributeItem::SpreadAttribute(spread) => ParsedJsxAttribute {
            name: "{...}".to_string(),
            value: ParsedJsxAttributeValue::Spread(expression_summary(&spread.argument)),
            span: source_span(source, spread.span),
        },
    }
}

fn jsx_attribute_name(name: &JSXAttributeName<'_>) -> String {
    match name {
        JSXAttributeName::Identifier(identifier) => identifier.name.to_string(),
        JSXAttributeName::NamespacedName(namespaced) => {
            format!("{}:{}", namespaced.namespace.name, namespaced.name.name)
        }
    }
}

fn jsx_event_handler(attribute: &JSXAttributeItem<'_>, source: &str) -> Option<ParsedEventHandler> {
    let JSXAttributeItem::Attribute(attribute) = attribute else {
        return None;
    };

    let attribute_name = match &attribute.name {
        JSXAttributeName::Identifier(identifier) => identifier.name.as_str(),
        JSXAttributeName::NamespacedName(_) => return None,
    };

    let event = jsx_event_type(attribute_name)?;

    let Some(JSXAttributeValue::ExpressionContainer(container)) = &attribute.value else {
        return None;
    };

    let handler = jsx_expression_event_handler_ref(&container.expression)?;

    Some(ParsedEventHandler {
        event,
        handler,
        span: source_span(source, attribute.span),
    })
}

fn jsx_event_type(attribute_name: &str) -> Option<String> {
    let name = attribute_name.strip_prefix("on")?;

    if !name.chars().next().is_some_and(char::is_uppercase) {
        return None;
    }

    let mut chars = name.chars();
    let first = chars.next()?.to_ascii_lowercase();
    let rest = chars.collect::<String>();

    Some(format!("{first}{rest}"))
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
