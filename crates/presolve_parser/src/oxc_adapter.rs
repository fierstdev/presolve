use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use oxc_allocator::Allocator;
use oxc_ast::ast::{
    Argument, AssignmentTarget, BindingPatternKind, ChainElement, ClassElement, Declaration,
    ExportDefaultDeclarationKind, Expression, ImportDeclarationSpecifier, JSXAttributeItem,
    JSXAttributeName, JSXAttributeValue, JSXChild, JSXElementName, JSXExpression, JSXFragment,
    JSXMemberExpression, JSXMemberExpressionObject, ModuleExportName, ObjectPropertyKind, Program,
    PropertyKey, PropertyKind, SimpleAssignmentTarget, Statement,
};
use oxc_ast_visit::{walk, Visit};
use oxc_diagnostics::Severity as OxcSeverity;
use oxc_estree::{CompactTSSerializer, ESTree};
use oxc_parser::Parser;
use oxc_span::{GetSpan, SourceType, Span};

use crate::model::{
    ParseDiagnostic, ParseLabel, ParseSeverity, ParsedArithmeticExpression,
    ParsedArithmeticExpressionKind, ParsedArithmeticOperator, ParsedCallArgument,
    ParsedCallExpression, ParsedClass, ParsedClassHeritage, ParsedComparisonOperator,
    ParsedComputedExpression, ParsedComputedExpressionKind, ParsedConstantExpression,
    ParsedConstantExpressionKind, ParsedDecorator, ParsedEffectBody, ParsedEffectCleanup,
    ParsedEffectExpression, ParsedEffectExpressionKind, ParsedEffectStatement,
    ParsedEffectStatementKind, ParsedEventHandler, ParsedExport, ParsedExportKind,
    ParsedExportSpecifier, ParsedFile, ParsedImport, ParsedImportSpecifier, ParsedInitializerCall,
    ParsedInlineHandler, ParsedJsxAttribute, ParsedJsxAttributeValue, ParsedJsxChild,
    ParsedJsxConditional, ParsedJsxElement, ParsedJsxFragment, ParsedJsxList, ParsedJsxNode,
    ParsedLocalVariable, ParsedLogicalOperator, ParsedMethod, ParsedMethodCall,
    ParsedMethodParameter, ParsedProperty, ParsedSerializableValue, ParsedSourceAst,
    ParsedStateOperation, ParsedStateUpdate, ParsedStaticMemberDesignator,
    ParsedThisMemberDesignator, ParsedTypeAlias, ParsedTypeAnnotation, ParsedUnaryOperator,
    ParsedUnsupportedEffectStatementKind, ParsedValidationRuleArgument,
    ParsedValidationRuleArgumentKind, ParsedValidationRuleExpression,
    ParsedValidationRuleExpressionKind, SourceSpan,
};

pub fn parse_file(path: impl AsRef<Path>, source: &str) -> ParsedFile {
    let path = path.as_ref();
    let source_type = SourceType::from_path(path)
        .unwrap_or_default()
        .with_typescript(true)
        .with_jsx(true);

    let allocator = Allocator::default();
    let ret = Parser::new(&allocator, source, source_type).parse();
    let syntax = parse_source_ast(&ret.program, source);
    let call_expressions = collect_call_expressions(&ret.program, source);

    let ParsedProgramFacts {
        classes,
        type_aliases,
        local_type_bindings,
        local_value_bindings,
        imports,
        exports,
    } = parse_program(&ret.program, source);
    let diagnostics = ret
        .errors
        .iter()
        .map(|diagnostic| parse_diagnostic(source, diagnostic))
        .collect::<Vec<_>>();

    ParsedFile {
        path: PathBuf::from(path),
        syntax,
        classes,
        type_aliases,
        local_type_bindings,
        local_value_bindings,
        imports,
        exports,
        call_expressions,
        diagnostics,
    }
}

fn collect_call_expressions(program: &Program<'_>, source: &str) -> Vec<ParsedCallExpression> {
    struct Collector<'a> {
        source: &'a str,
        calls: Vec<ParsedCallExpression>,
    }

    impl<'a> Visit<'a> for Collector<'a> {
        fn visit_call_expression(&mut self, call: &oxc_ast::ast::CallExpression<'a>) {
            let (member_object_span, member_property_span) = match &call.callee {
                Expression::StaticMemberExpression(member) => (
                    Some(source_span(self.source, member.object.span())),
                    Some(source_span(self.source, member.property.span)),
                ),
                _ => (None, None),
            };
            self.calls.push(ParsedCallExpression {
                callee_span: source_span(self.source, call.callee.span()),
                member_object_span,
                member_property_span,
                span: source_span(self.source, call.span),
                arguments: call
                    .arguments
                    .iter()
                    .map(|argument| match argument {
                        Argument::StringLiteral(value) => ParsedCallArgument::StringLiteral {
                            value: value.value.to_string(),
                            span: source_span(self.source, value.span),
                        },
                        _ => ParsedCallArgument::Other {
                            span: source_span(self.source, argument.span()),
                        },
                    })
                    .collect(),
            });
            walk::walk_call_expression(self, call);
        }
    }

    let mut collector = Collector {
        source,
        calls: Vec::new(),
    };
    collector.visit_program(program);
    collector
        .calls
        .sort_by_key(|call| (call.span.start, call.span.end));
    collector.calls
}

fn parse_source_ast(program: &Program<'_>, source: &str) -> ParsedSourceAst {
    let mut serializer = CompactTSSerializer::new(true);
    program.serialize(&mut serializer);
    ParsedSourceAst {
        source: source.to_string(),
        estree_json: serializer.into_string(),
        span: source_span(source, program.span),
    }
}

struct ParsedProgramFacts {
    classes: Vec<ParsedClass>,
    type_aliases: Vec<ParsedTypeAlias>,
    local_type_bindings: Vec<String>,
    local_value_bindings: Vec<String>,
    imports: Vec<ParsedImport>,
    exports: Vec<ParsedExport>,
}

fn parse_program(program: &Program<'_>, source: &str) -> ParsedProgramFacts {
    let mut classes = Vec::new();
    let mut type_aliases = Vec::new();
    let mut local_type_bindings = Vec::new();
    let mut local_value_bindings = Vec::new();
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
                    retain_local_type_binding(declaration, &mut local_type_bindings);
                    retain_local_value_binding(declaration, &mut local_value_bindings);
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
                        local_type_bindings.push(class.name.clone());
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
            retain_local_type_binding(declaration, &mut local_type_bindings);
            retain_local_value_binding(declaration, &mut local_value_bindings);
            if let Some(class) = parse_declaration(declaration, source) {
                classes.push(class);
            }
            if let Some(alias) = parse_type_alias_declaration(declaration, source) {
                type_aliases.push(alias);
            }
        }
    }

    local_type_bindings.sort();
    local_type_bindings.dedup();
    local_value_bindings.sort();
    local_value_bindings.dedup();
    ParsedProgramFacts {
        classes,
        type_aliases,
        local_type_bindings,
        local_value_bindings,
        imports,
        exports,
    }
}

fn retain_local_value_binding(declaration: &Declaration<'_>, bindings: &mut Vec<String>) {
    if let Declaration::FunctionDeclaration(function) = declaration {
        if let Some(id) = &function.id {
            bindings.push(id.name.to_string());
        }
    }
}

fn retain_local_type_binding(declaration: &Declaration<'_>, bindings: &mut Vec<String>) {
    let name = match declaration {
        Declaration::ClassDeclaration(class) => class.id.as_ref().map(|id| id.name.as_str()),
        Declaration::TSTypeAliasDeclaration(alias) => Some(alias.id.name.as_str()),
        Declaration::TSInterfaceDeclaration(interface) => Some(interface.id.name.as_str()),
        Declaration::TSEnumDeclaration(r#enum) => Some(r#enum.id.name.as_str()),
        Declaration::TSImportEqualsDeclaration(import) => Some(import.id.name.as_str()),
        Declaration::VariableDeclaration(_)
        | Declaration::FunctionDeclaration(_)
        | Declaration::TSModuleDeclaration(_) => None,
    };
    if let Some(name) = name {
        bindings.push(name.to_string());
    }
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
                            local_span: source_span(source, specifier.local.span),
                        }
                    }
                    ImportDeclarationSpecifier::ImportDefaultSpecifier(specifier) => {
                        ParsedImportSpecifier {
                            imported: "default".to_string(),
                            local: specifier.local.name.to_string(),
                            local_span: source_span(source, specifier.local.span),
                        }
                    }
                    ImportDeclarationSpecifier::ImportNamespaceSpecifier(specifier) => {
                        ParsedImportSpecifier {
                            imported: "*".to_string(),
                            local: specifier.local.name.to_string(),
                            local_span: source_span(source, specifier.local.span),
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

    let decorators = normalized_decorators(
        class
            .decorators
            .iter()
            .filter_map(|decorator| parse_decorator(decorator, source))
            .collect::<Vec<_>>(),
    );

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
        heritage: class.super_class.as_ref().map(|base| {
            let span = base.span();
            ParsedClassHeritage {
                base: source
                    .get(span.start as usize..span.end as usize)
                    .unwrap_or_default()
                    .trim()
                    .to_string(),
                span: source_span(source, span),
            }
        }),
        decorators,
        properties,
        methods,
    })
}

fn parse_decorator(
    decorator: &oxc_ast::ast::Decorator<'_>,
    source: &str,
) -> Option<ParsedDecorator> {
    match &decorator.expression {
        Expression::CallExpression(call) => {
            let Expression::Identifier(callee) = &call.callee else {
                return None;
            };
            Some(ParsedDecorator {
                name: callee.name.to_string(),
                is_invoked: true,
                arguments: call.arguments.iter().map(argument_string_value).collect(),
                argument: call.arguments.first().and_then(argument_string_value),
                argument_count: call.arguments.len(),
                argument_spans: call
                    .arguments
                    .iter()
                    .map(|argument| source_span(source, argument.span()))
                    .collect(),
                static_member_argument: call
                    .arguments
                    .first()
                    .and_then(|argument| parsed_static_member_designator(argument, source)),
                this_member_argument: call
                    .arguments
                    .first()
                    .and_then(|argument| parsed_this_member_designator(argument, source)),
                validation_rule_expression: (callee.name == "validate")
                    .then(|| call.arguments.first()?.as_expression())
                    .flatten()
                    .map(|expression| parsed_validation_rule_expression(expression, source)),
                span: source_span(source, decorator.span),
            })
        }
        Expression::Identifier(identifier) => Some(ParsedDecorator {
            name: identifier.name.to_string(),
            is_invoked: false,
            arguments: Vec::new(),
            argument: None,
            argument_count: 0,
            argument_spans: Vec::new(),
            static_member_argument: None,
            this_member_argument: None,
            validation_rule_expression: None,
            span: source_span(source, decorator.span),
        }),
        _ => None,
    }
}

fn normalized_decorators(mut decorators: Vec<ParsedDecorator>) -> Vec<ParsedDecorator> {
    if !decorators.iter().any(|decorator| {
        matches!(
            decorator.name.as_str(),
            "form" | "field" | "validate" | "submit" | "serialize"
        )
    }) {
        decorators.retain(|decorator| decorator.is_invoked);
    }
    decorators
}

fn parsed_validation_rule_expression(
    expression: &Expression<'_>,
    source: &str,
) -> ParsedValidationRuleExpression {
    let kind = match expression {
        Expression::CallExpression(call) => ParsedValidationRuleExpressionKind::Call {
            callee: match &call.callee {
                Expression::Identifier(identifier) => Some(identifier.name.to_string()),
                _ => None,
            },
            arguments: call
                .arguments
                .iter()
                .map(|argument| parsed_validation_rule_argument(argument, source))
                .collect(),
        },
        Expression::Identifier(identifier) => {
            ParsedValidationRuleExpressionKind::Identifier(identifier.name.to_string())
        }
        _ => ParsedValidationRuleExpressionKind::Unsupported,
    };
    ParsedValidationRuleExpression {
        kind,
        span: source_span(source, expression.span()),
    }
}

fn parsed_validation_rule_argument(
    argument: &Argument<'_>,
    source: &str,
) -> ParsedValidationRuleArgument {
    let span = source_span(source, argument.span());
    let kind = argument.as_expression().map_or(
        ParsedValidationRuleArgumentKind::Unsupported,
        |expression| {
            if let Expression::StringLiteral(literal) = expression {
                return ParsedValidationRuleArgumentKind::StringLiteral(literal.value.to_string());
            }
            if let Some(designator) = parsed_this_member_expression(expression, source) {
                return ParsedValidationRuleArgumentKind::ThisMember(designator);
            }
            parsed_constant_expression(expression, source).map_or(
                ParsedValidationRuleArgumentKind::Unsupported,
                ParsedValidationRuleArgumentKind::Constant,
            )
        },
    );
    ParsedValidationRuleArgument { kind, span }
}

fn parsed_this_member_designator(
    argument: &Argument<'_>,
    source: &str,
) -> Option<ParsedThisMemberDesignator> {
    parsed_this_member_expression(argument.as_expression()?, source)
}

fn parsed_this_member_expression(
    expression: &Expression<'_>,
    source: &str,
) -> Option<ParsedThisMemberDesignator> {
    let Expression::StaticMemberExpression(member) = expression else {
        return None;
    };
    let Expression::ThisExpression(this) = &member.object else {
        return None;
    };
    Some(ParsedThisMemberDesignator {
        member: member.property.name.to_string(),
        span: source_span(source, member.span),
        this_span: source_span(source, this.span),
        member_span: source_span(source, member.property.span),
    })
}

fn parsed_static_member_designator(
    argument: &Argument<'_>,
    source: &str,
) -> Option<ParsedStaticMemberDesignator> {
    let Expression::StaticMemberExpression(member) = argument.as_expression()? else {
        return None;
    };
    let Expression::Identifier(object) = &member.object else {
        return None;
    };
    Some(ParsedStaticMemberDesignator {
        object: object.name.to_string(),
        member: member.property.name.to_string(),
        span: source_span(source, member.span),
        object_span: source_span(source, object.span),
        member_span: source_span(source, member.property.span),
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
    let decorators = normalized_decorators(
        property
            .decorators
            .iter()
            .filter_map(|decorator| parse_decorator(decorator, source))
            .collect::<Vec<_>>(),
    );
    let (name, is_identifier_name) = match &property.key {
        PropertyKey::StaticIdentifier(identifier) => (identifier.name.to_string(), true),
        key => match property_key_name(key) {
            Some(name) => (name, false),
            None if decorators
                .iter()
                .any(|decorator| matches!(decorator.name.as_str(), "form" | "field")) =>
            {
                (
                    format!("<unsupported:{}>", property.key.span().start),
                    false,
                )
            }
            None => return None,
        },
    };

    let initializer = property.value.as_ref().and_then(expression_summary);
    let initializer_call = property.value.as_ref().and_then(|expression| {
        let Expression::CallExpression(call) = expression else {
            return None;
        };
        Some(ParsedInitializerCall {
            callee_span: source_span(source, call.callee.span()),
            span: source_span(source, call.span),
            argument_count: call.arguments.len(),
            inline_handler: parsed_inline_handler(call, source),
        })
    });
    let initializer_literal = property
        .value
        .as_ref()
        .and_then(serializable_value_from_expression);
    let initializer_expression = property
        .value
        .as_ref()
        .and_then(|expression| parsed_computed_expression(expression, source));
    let initializer_constant_expression = property
        .value
        .as_ref()
        .and_then(|expression| parsed_constant_expression(expression, source));
    let initializer_span = property
        .value
        .as_ref()
        .map(|value| source_span(source, value.span()));

    let state_initial_value = property.value.as_ref().and_then(state_initial_value);
    let state_initial_expression = property
        .value
        .as_ref()
        .and_then(|expression| state_initial_constant_expression(expression, source));
    let type_annotation = property
        .type_annotation
        .as_ref()
        .map(|annotation| parsed_type_annotation(annotation.span, source));
    let state_type_annotation = (initializer.as_deref() == Some("state(...)"))
        .then_some(type_annotation.clone())
        .flatten();
    let declaration_start = decorators
        .first()
        .map_or(property.span.start as usize, |decorator| {
            decorator.span.start
        });

    Some(ParsedProperty {
        name,
        is_identifier_name,
        decorators,
        initializer_call,
        initializer,
        initializer_literal,
        initializer_expression,
        initializer_constant_expression,
        initializer_span,
        state_initial_value,
        state_initial_expression,
        state_type_annotation,
        type_annotation,
        name_span: source_span(source, property.key.span()),
        is_static: property.r#static,
        is_definite_assignment: property.definite,
        is_declare: property.declare,
        span: source_span_from_offsets(source, declaration_start, property.span.end as usize),
    })
}

/// Retain general syntax facts for a single inline function argument.  This
/// deliberately makes no inference from the callee spelling or field name.
fn parsed_inline_handler(
    call: &oxc_ast::ast::CallExpression<'_>,
    source: &str,
) -> Option<ParsedInlineHandler> {
    let [argument] = call.arguments.as_slice() else {
        return None;
    };
    match argument {
        Argument::ArrowFunctionExpression(handler) => Some(parsed_inline_handler_body(
            handler.span,
            &handler.body,
            handler.r#async,
            handler.expression,
            parsed_inline_handler_parameters(&handler.params, source),
            source,
        )),
        Argument::FunctionExpression(handler) => Some(parsed_inline_handler_body(
            handler.span,
            handler.body.as_deref()?,
            handler.r#async,
            false,
            parsed_inline_handler_parameters(&handler.params, source),
            source,
        )),
        _ => None,
    }
}

fn parsed_inline_handler_body(
    span: Span,
    body: &oxc_ast::ast::FunctionBody<'_>,
    is_async: bool,
    is_expression_body: bool,
    parameters: Vec<ParsedMethodParameter>,
    source: &str,
) -> ParsedInlineHandler {
    let mut state_updates = Vec::new();
    let mut unsupported_statement_spans = Vec::new();
    for statement in &body.statements {
        if let Some(update) = parsed_state_update(statement, source) {
            state_updates.push(update);
        } else if !matches!(statement, Statement::EmptyStatement(_)) {
            unsupported_statement_spans.push(source_span(source, statement.span()));
        }
    }
    ParsedInlineHandler {
        span: source_span(source, span),
        body_span: source_span(source, body.span),
        is_async,
        is_expression_body,
        parameters,
        state_updates,
        unsupported_statement_spans,
        effect_body: (!is_expression_body).then(|| parsed_inline_effect_body(body, source)),
    }
}

fn parsed_inline_handler_parameters(
    parameters: &oxc_ast::ast::FormalParameters<'_>,
    source: &str,
) -> Vec<ParsedMethodParameter> {
    parameters
        .items
        .iter()
        .filter_map(|parameter| {
            Some(ParsedMethodParameter {
                name: binding_identifier_name(&parameter.pattern.kind)?,
                decorators: normalized_decorators(
                    parameter
                        .decorators
                        .iter()
                        .filter_map(|decorator| parse_decorator(decorator, source))
                        .collect(),
                ),
                span: source_span(source, parameter.span),
                type_annotation: parameter
                    .pattern
                    .type_annotation
                    .as_ref()
                    .map(|annotation| parsed_type_annotation(annotation.span, source)),
            })
        })
        .collect()
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
    let decorators = normalized_decorators(
        method
            .decorators
            .iter()
            .filter_map(|decorator| parse_decorator(decorator, source))
            .collect::<Vec<_>>(),
    );

    let mut jsx_roots = Vec::new();
    let mut bindings = Vec::new();
    let mut state_updates = Vec::new();
    let mut local_variables = Vec::new();
    let mut return_values = Vec::new();
    let mut calls = Vec::new();
    let parameters = method
        .value
        .params
        .items
        .iter()
        .filter_map(|parameter| {
            Some(ParsedMethodParameter {
                name: binding_identifier_name(&parameter.pattern.kind)?,
                decorators: normalized_decorators(
                    parameter
                        .decorators
                        .iter()
                        .filter_map(|decorator| parse_decorator(decorator, source))
                        .collect(),
                ),
                span: source_span(source, parameter.span),
                type_annotation: parameter
                    .pattern
                    .type_annotation
                    .as_ref()
                    .map(|annotation| parsed_type_annotation(annotation.span, source)),
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
            if let Some(value) = parsed_return_value(statement) {
                return_values.push(value);
            }
            collect_method_calls(statement, source, &mut calls);
        }
    }

    // Getter expression retention is syntax only. A later canonical V2
    // analysis decides whether a getter is computed; decorators are not a
    // parser recognition condition.
    let computed_expression = (method.kind == oxc_ast::ast::MethodDefinitionKind::Get)
        .then(|| {
            let body = method.value.body.as_ref()?;
            let [Statement::ReturnStatement(return_statement)] = body.statements.as_slice() else {
                return None;
            };
            parsed_computed_expression(return_statement.argument.as_ref()?, source)
        })
        .flatten();
    let effect_body = decorators
        .iter()
        .any(|decorator| decorator.name == "effect")
        .then(|| {
            method
                .value
                .body
                .as_ref()
                .map(|body| parsed_effect_body(body, source))
        })
        .flatten();

    Some(ParsedMethod {
        name,
        span: source_span(source, method.span),
        decorators,
        is_getter: method.kind == oxc_ast::ast::MethodDefinitionKind::Get,
        is_setter: method.kind == oxc_ast::ast::MethodDefinitionKind::Set,
        is_async: method.value.r#async,
        is_static: method.r#static,
        jsx_roots,
        bindings,
        state_updates,
        local_variables,
        parameters,
        return_type_annotation: method
            .value
            .return_type
            .as_ref()
            .map(|annotation| parsed_type_annotation(annotation.span, source)),
        return_values,
        computed_expression,
        effect_body,
        calls,
    })
}

fn parsed_effect_body(body: &oxc_ast::ast::FunctionBody<'_>, source: &str) -> ParsedEffectBody {
    parsed_effect_body_with_cleanup(body, source, false)
}

fn parsed_inline_effect_body(
    body: &oxc_ast::ast::FunctionBody<'_>,
    source: &str,
) -> ParsedEffectBody {
    parsed_effect_body_with_cleanup(body, source, true)
}

fn parsed_effect_body_with_cleanup(
    body: &oxc_ast::ast::FunctionBody<'_>,
    source: &str,
    allow_cleanup: bool,
) -> ParsedEffectBody {
    let final_statement = body.statements.len().saturating_sub(1);
    let cleanup = allow_cleanup
        .then(|| body.statements.last())
        .flatten()
        .and_then(|statement| parsed_effect_cleanup(statement, source));
    ParsedEffectBody {
        statements: body
            .statements
            .iter()
            .enumerate()
            .filter(|(index, _)| cleanup.is_none() || *index != final_statement)
            .map(|(index, statement)| {
                parsed_effect_statement(statement, index == final_statement, source)
            })
            .collect(),
        cleanup,
    }
}

fn parsed_effect_cleanup(statement: &Statement<'_>, source: &str) -> Option<ParsedEffectCleanup> {
    let Statement::ReturnStatement(return_statement) = statement else {
        return None;
    };
    let expression = return_statement.argument.as_ref()?;
    let (span, is_async, body) = match expression {
        Expression::ArrowFunctionExpression(handler) if !handler.expression => {
            (handler.span, handler.r#async, handler.body.as_ref())
        }
        Expression::FunctionExpression(handler) => {
            (handler.span, handler.r#async, handler.body.as_deref()?)
        }
        _ => return None,
    };
    Some(ParsedEffectCleanup {
        span: source_span(source, span),
        is_async,
        body: Box::new(parsed_effect_body(body, source)),
    })
}

fn parsed_effect_statement(
    statement: &Statement<'_>,
    is_final: bool,
    source: &str,
) -> ParsedEffectStatement {
    let span = source_span(source, statement.span());
    let kind = match statement {
        Statement::EmptyStatement(_) => ParsedEffectStatementKind::Empty,
        Statement::ExpressionStatement(statement) => match &statement.expression {
            Expression::AssignmentExpression(assignment) if assignment.operator.as_str() == "=" => {
                match (
                    parsed_effect_assignment_target(&assignment.left, source),
                    parsed_effect_expression(&assignment.right, source),
                ) {
                    (Some(target), Some(value)) => {
                        ParsedEffectStatementKind::StaticMemberAssignment { target, value }
                    }
                    _ => ParsedEffectStatementKind::Unsupported(
                        ParsedUnsupportedEffectStatementKind::UnsupportedExpression,
                    ),
                }
            }
            Expression::AssignmentExpression(_) | Expression::UpdateExpression(_) => {
                ParsedEffectStatementKind::Unsupported(
                    ParsedUnsupportedEffectStatementKind::CompoundAssignment,
                )
            }
            Expression::CallExpression(call) => match (
                parsed_effect_expression(&call.callee, source),
                call.arguments
                    .iter()
                    .map(|argument| {
                        argument
                            .as_expression()
                            .and_then(|expression| parsed_effect_expression(expression, source))
                    })
                    .collect::<Option<Vec<_>>>(),
            ) {
                (Some(callee), Some(arguments)) => {
                    ParsedEffectStatementKind::CapabilityCall { callee, arguments }
                }
                _ => ParsedEffectStatementKind::Unsupported(
                    ParsedUnsupportedEffectStatementKind::UnsupportedExpression,
                ),
            },
            _ => ParsedEffectStatementKind::Unsupported(
                ParsedUnsupportedEffectStatementKind::UnsupportedExpression,
            ),
        },
        Statement::ReturnStatement(statement) => match &statement.argument {
            None if is_final => ParsedEffectStatementKind::EffectReturn { value: None },
            Some(value) => parsed_effect_expression(value, source).map_or_else(
                || {
                    ParsedEffectStatementKind::Unsupported(
                        ParsedUnsupportedEffectStatementKind::CleanupReturnCandidate,
                    )
                },
                |value| ParsedEffectStatementKind::EffectReturn { value: Some(value) },
            ),
            None => ParsedEffectStatementKind::Unsupported(
                ParsedUnsupportedEffectStatementKind::UnsupportedExpression,
            ),
        },
        statement if statement.as_declaration().is_some() => {
            ParsedEffectStatementKind::Unsupported(
                ParsedUnsupportedEffectStatementKind::LocalDeclaration,
            )
        }
        Statement::IfStatement(_) | Statement::SwitchStatement(_) => {
            ParsedEffectStatementKind::Unsupported(ParsedUnsupportedEffectStatementKind::Branch)
        }
        Statement::ForInStatement(_)
        | Statement::ForOfStatement(_)
        | Statement::ForStatement(_)
        | Statement::WhileStatement(_)
        | Statement::DoWhileStatement(_) => {
            ParsedEffectStatementKind::Unsupported(ParsedUnsupportedEffectStatementKind::Loop)
        }
        Statement::BlockStatement(_) => ParsedEffectStatementKind::Unsupported(
            ParsedUnsupportedEffectStatementKind::NestedBlock,
        ),
        Statement::TryStatement(_) | Statement::ThrowStatement(_) => {
            ParsedEffectStatementKind::Unsupported(
                ParsedUnsupportedEffectStatementKind::ExceptionHandling,
            )
        }
        _ => ParsedEffectStatementKind::Unsupported(
            ParsedUnsupportedEffectStatementKind::UnsupportedExpression,
        ),
    };
    ParsedEffectStatement { kind, span }
}

fn parsed_effect_assignment_target(
    target: &AssignmentTarget<'_>,
    source: &str,
) -> Option<ParsedEffectExpression> {
    let AssignmentTarget::StaticMemberExpression(member) = target else {
        return None;
    };
    parsed_effect_static_member(
        &member.object,
        member.property.name.as_str(),
        member.span,
        source,
    )
}

fn parsed_effect_expression(
    expression: &Expression<'_>,
    source: &str,
) -> Option<ParsedEffectExpression> {
    if let Expression::ParenthesizedExpression(parenthesized) = expression {
        return parsed_effect_expression(&parenthesized.expression, source);
    }
    if let Some(value) = serializable_value_from_expression(expression) {
        return Some(ParsedEffectExpression {
            kind: ParsedEffectExpressionKind::Literal(value),
            span: source_span(source, expression.span()),
        });
    }
    match expression {
        Expression::Identifier(identifier) => Some(ParsedEffectExpression {
            kind: ParsedEffectExpressionKind::Identifier(identifier.name.to_string()),
            span: source_span(source, identifier.span),
        }),
        Expression::ThisExpression(this) => Some(ParsedEffectExpression {
            kind: ParsedEffectExpressionKind::Identifier("this".to_string()),
            span: source_span(source, this.span),
        }),
        Expression::StaticMemberExpression(member) => parsed_effect_static_member(
            &member.object,
            member.property.name.as_str(),
            member.span,
            source,
        ),
        Expression::BinaryExpression(binary) => {
            let kind = match binary.operator.as_str() {
                "+" => ParsedEffectExpressionKind::Arithmetic {
                    left: Box::new(parsed_effect_expression(&binary.left, source)?),
                    right: Box::new(parsed_effect_expression(&binary.right, source)?),
                    operator: ParsedArithmeticOperator::Add,
                },
                "-" => ParsedEffectExpressionKind::Arithmetic {
                    left: Box::new(parsed_effect_expression(&binary.left, source)?),
                    right: Box::new(parsed_effect_expression(&binary.right, source)?),
                    operator: ParsedArithmeticOperator::Subtract,
                },
                "*" => ParsedEffectExpressionKind::Arithmetic {
                    left: Box::new(parsed_effect_expression(&binary.left, source)?),
                    right: Box::new(parsed_effect_expression(&binary.right, source)?),
                    operator: ParsedArithmeticOperator::Multiply,
                },
                "/" => ParsedEffectExpressionKind::Arithmetic {
                    left: Box::new(parsed_effect_expression(&binary.left, source)?),
                    right: Box::new(parsed_effect_expression(&binary.right, source)?),
                    operator: ParsedArithmeticOperator::Divide,
                },
                "%" => ParsedEffectExpressionKind::Arithmetic {
                    left: Box::new(parsed_effect_expression(&binary.left, source)?),
                    right: Box::new(parsed_effect_expression(&binary.right, source)?),
                    operator: ParsedArithmeticOperator::Remainder,
                },
                "===" => ParsedEffectExpressionKind::Comparison {
                    left: Box::new(parsed_effect_expression(&binary.left, source)?),
                    right: Box::new(parsed_effect_expression(&binary.right, source)?),
                    operator: ParsedComparisonOperator::Equal,
                },
                "!==" => ParsedEffectExpressionKind::Comparison {
                    left: Box::new(parsed_effect_expression(&binary.left, source)?),
                    right: Box::new(parsed_effect_expression(&binary.right, source)?),
                    operator: ParsedComparisonOperator::NotEqual,
                },
                "<" => ParsedEffectExpressionKind::Comparison {
                    left: Box::new(parsed_effect_expression(&binary.left, source)?),
                    right: Box::new(parsed_effect_expression(&binary.right, source)?),
                    operator: ParsedComparisonOperator::LessThan,
                },
                "<=" => ParsedEffectExpressionKind::Comparison {
                    left: Box::new(parsed_effect_expression(&binary.left, source)?),
                    right: Box::new(parsed_effect_expression(&binary.right, source)?),
                    operator: ParsedComparisonOperator::LessThanOrEqual,
                },
                ">" => ParsedEffectExpressionKind::Comparison {
                    left: Box::new(parsed_effect_expression(&binary.left, source)?),
                    right: Box::new(parsed_effect_expression(&binary.right, source)?),
                    operator: ParsedComparisonOperator::GreaterThan,
                },
                ">=" => ParsedEffectExpressionKind::Comparison {
                    left: Box::new(parsed_effect_expression(&binary.left, source)?),
                    right: Box::new(parsed_effect_expression(&binary.right, source)?),
                    operator: ParsedComparisonOperator::GreaterThanOrEqual,
                },
                _ => return None,
            };
            Some(ParsedEffectExpression {
                kind,
                span: source_span(source, binary.span),
            })
        }
        Expression::LogicalExpression(logical) => {
            let kind = match logical.operator.as_str() {
                "&&" => ParsedEffectExpressionKind::Logical {
                    left: Box::new(parsed_effect_expression(&logical.left, source)?),
                    right: Box::new(parsed_effect_expression(&logical.right, source)?),
                    operator: ParsedLogicalOperator::And,
                },
                "||" => ParsedEffectExpressionKind::Logical {
                    left: Box::new(parsed_effect_expression(&logical.left, source)?),
                    right: Box::new(parsed_effect_expression(&logical.right, source)?),
                    operator: ParsedLogicalOperator::Or,
                },
                "??" => ParsedEffectExpressionKind::NullishCoalescing {
                    left: Box::new(parsed_effect_expression(&logical.left, source)?),
                    right: Box::new(parsed_effect_expression(&logical.right, source)?),
                },
                _ => return None,
            };
            Some(ParsedEffectExpression {
                kind,
                span: source_span(source, logical.span),
            })
        }
        Expression::UnaryExpression(unary) => {
            let operator = match unary.operator.as_str() {
                "!" => ParsedUnaryOperator::Not,
                "+" => ParsedUnaryOperator::Plus,
                "-" => ParsedUnaryOperator::Minus,
                _ => return None,
            };
            Some(ParsedEffectExpression {
                kind: ParsedEffectExpressionKind::Unary {
                    operand: Box::new(parsed_effect_expression(&unary.argument, source)?),
                    operator,
                },
                span: source_span(source, unary.span),
            })
        }
        _ => None,
    }
}

fn parsed_effect_static_member(
    object: &Expression<'_>,
    property: &str,
    span: Span,
    source: &str,
) -> Option<ParsedEffectExpression> {
    let object = parsed_effect_expression(object, source)?;
    let kind = if matches!(&object.kind, ParsedEffectExpressionKind::Identifier(name) if name == "this")
    {
        ParsedEffectExpressionKind::ThisMember(property.to_string())
    } else {
        ParsedEffectExpressionKind::MemberAccess {
            object: Box::new(object),
            property: property.to_string(),
        }
    };
    Some(ParsedEffectExpression {
        kind,
        span: source_span(source, span),
    })
}

fn collect_method_calls(
    statement: &Statement<'_>,
    source: &str,
    calls: &mut Vec<ParsedMethodCall>,
) {
    let expression = match statement {
        Statement::ExpressionStatement(statement) => Some(&statement.expression),
        Statement::ReturnStatement(statement) => statement.argument.as_ref(),
        _ => None,
    };
    if let Some(expression) = expression {
        collect_calls_from_expression(expression, source, calls);
    }
}

fn collect_calls_from_expression(
    expression: &Expression<'_>,
    source: &str,
    calls: &mut Vec<ParsedMethodCall>,
) {
    match expression {
        Expression::ParenthesizedExpression(parenthesized) => {
            collect_calls_from_expression(&parenthesized.expression, source, calls);
        }
        Expression::CallExpression(call) => {
            if let Some(callee) = expression_summary(&call.callee) {
                calls.push(ParsedMethodCall {
                    callee,
                    span: source_span(source, call.span),
                });
            }
            collect_calls_from_expression(&call.callee, source, calls);
            for argument in &call.arguments {
                if let Some(expression) = argument.as_expression() {
                    collect_calls_from_expression(expression, source, calls);
                }
            }
        }
        Expression::BinaryExpression(binary) => {
            collect_calls_from_expression(&binary.left, source, calls);
            collect_calls_from_expression(&binary.right, source, calls);
        }
        Expression::LogicalExpression(logical) => {
            collect_calls_from_expression(&logical.left, source, calls);
            collect_calls_from_expression(&logical.right, source, calls);
        }
        Expression::UnaryExpression(unary) => {
            collect_calls_from_expression(&unary.argument, source, calls);
        }
        _ => {}
    }
}

fn parsed_computed_expression(
    expression: &Expression<'_>,
    source: &str,
) -> Option<ParsedComputedExpression> {
    if let Expression::ParenthesizedExpression(parenthesized) = expression {
        return parsed_computed_expression(&parenthesized.expression, source);
    }

    if let Some(value) = serializable_value_from_expression(expression) {
        return Some(ParsedComputedExpression {
            kind: ParsedComputedExpressionKind::Literal(value),
            span: source_span(source, expression.span()),
        });
    }

    match expression {
        Expression::TemplateLiteral(template) => Some(ParsedComputedExpression {
            kind: ParsedComputedExpressionKind::Template {
                quasis: template
                    .quasis
                    .iter()
                    .map(|quasi| quasi.value.cooked.as_ref().map(ToString::to_string))
                    .collect::<Option<Vec<_>>>()?,
                expressions: template
                    .expressions
                    .iter()
                    .map(|expression| parsed_computed_expression(expression, source))
                    .collect::<Option<Vec<_>>>()?,
            },
            span: source_span(source, template.span),
        }),
        Expression::CallExpression(call) => {
            let callee = expression_summary(&call.callee)?;
            let arguments = call
                .arguments
                .iter()
                .map(|argument| parsed_computed_expression(argument.as_expression()?, source))
                .collect::<Option<Vec<_>>>()?;
            Some(ParsedComputedExpression {
                kind: ParsedComputedExpressionKind::Call { callee, arguments },
                span: source_span(source, call.span),
            })
        }
        Expression::StaticMemberExpression(member) => {
            let property = member.property.name.to_string();
            let kind = if matches!(&member.object, Expression::ThisExpression(_)) {
                ParsedComputedExpressionKind::ThisMember(property)
            } else {
                ParsedComputedExpressionKind::MemberAccess {
                    object: Box::new(parsed_computed_expression(&member.object, source)?),
                    property,
                    optional: member.optional,
                }
            };
            Some(ParsedComputedExpression {
                kind,
                span: source_span(source, member.span),
            })
        }
        Expression::ChainExpression(chain) => match &chain.expression {
            ChainElement::StaticMemberExpression(member) => Some(ParsedComputedExpression {
                kind: ParsedComputedExpressionKind::MemberAccess {
                    object: Box::new(parsed_computed_expression(&member.object, source)?),
                    property: member.property.name.to_string(),
                    optional: true,
                },
                span: source_span(source, chain.span),
            }),
            _ => None,
        },
        Expression::ComputedMemberExpression(member) => {
            let index = parsed_computed_expression(&member.expression, source)?;
            let supported_index = match &index.kind {
                ParsedComputedExpressionKind::Literal(ParsedSerializableValue::String(_)) => true,
                ParsedComputedExpressionKind::Literal(ParsedSerializableValue::Number(value)) => {
                    value.parse::<u64>().is_ok()
                }
                _ => false,
            };
            if !supported_index {
                return None;
            }
            Some(ParsedComputedExpression {
                kind: ParsedComputedExpressionKind::IndexAccess {
                    object: Box::new(parsed_computed_expression(&member.object, source)?),
                    index: Box::new(index),
                },
                span: source_span(source, member.span),
            })
        }
        Expression::ConditionalExpression(conditional) => Some(ParsedComputedExpression {
            kind: ParsedComputedExpressionKind::Conditional {
                condition: Box::new(parsed_computed_expression(&conditional.test, source)?),
                when_true: Box::new(parsed_computed_expression(&conditional.consequent, source)?),
                when_false: Box::new(parsed_computed_expression(&conditional.alternate, source)?),
            },
            span: source_span(source, conditional.span),
        }),
        Expression::BinaryExpression(binary) => {
            let operator = match binary.operator.as_str() {
                "+" => ParsedComputedExpressionKind::Arithmetic {
                    left: Box::new(parsed_computed_expression(&binary.left, source)?),
                    right: Box::new(parsed_computed_expression(&binary.right, source)?),
                    operator: ParsedArithmeticOperator::Add,
                },
                "-" => ParsedComputedExpressionKind::Arithmetic {
                    left: Box::new(parsed_computed_expression(&binary.left, source)?),
                    right: Box::new(parsed_computed_expression(&binary.right, source)?),
                    operator: ParsedArithmeticOperator::Subtract,
                },
                "*" => ParsedComputedExpressionKind::Arithmetic {
                    left: Box::new(parsed_computed_expression(&binary.left, source)?),
                    right: Box::new(parsed_computed_expression(&binary.right, source)?),
                    operator: ParsedArithmeticOperator::Multiply,
                },
                "/" => ParsedComputedExpressionKind::Arithmetic {
                    left: Box::new(parsed_computed_expression(&binary.left, source)?),
                    right: Box::new(parsed_computed_expression(&binary.right, source)?),
                    operator: ParsedArithmeticOperator::Divide,
                },
                "%" => ParsedComputedExpressionKind::Arithmetic {
                    left: Box::new(parsed_computed_expression(&binary.left, source)?),
                    right: Box::new(parsed_computed_expression(&binary.right, source)?),
                    operator: ParsedArithmeticOperator::Remainder,
                },
                "===" => ParsedComputedExpressionKind::Comparison {
                    left: Box::new(parsed_computed_expression(&binary.left, source)?),
                    right: Box::new(parsed_computed_expression(&binary.right, source)?),
                    operator: ParsedComparisonOperator::Equal,
                },
                "!==" => ParsedComputedExpressionKind::Comparison {
                    left: Box::new(parsed_computed_expression(&binary.left, source)?),
                    right: Box::new(parsed_computed_expression(&binary.right, source)?),
                    operator: ParsedComparisonOperator::NotEqual,
                },
                "<" => ParsedComputedExpressionKind::Comparison {
                    left: Box::new(parsed_computed_expression(&binary.left, source)?),
                    right: Box::new(parsed_computed_expression(&binary.right, source)?),
                    operator: ParsedComparisonOperator::LessThan,
                },
                "<=" => ParsedComputedExpressionKind::Comparison {
                    left: Box::new(parsed_computed_expression(&binary.left, source)?),
                    right: Box::new(parsed_computed_expression(&binary.right, source)?),
                    operator: ParsedComparisonOperator::LessThanOrEqual,
                },
                ">" => ParsedComputedExpressionKind::Comparison {
                    left: Box::new(parsed_computed_expression(&binary.left, source)?),
                    right: Box::new(parsed_computed_expression(&binary.right, source)?),
                    operator: ParsedComparisonOperator::GreaterThan,
                },
                ">=" => ParsedComputedExpressionKind::Comparison {
                    left: Box::new(parsed_computed_expression(&binary.left, source)?),
                    right: Box::new(parsed_computed_expression(&binary.right, source)?),
                    operator: ParsedComparisonOperator::GreaterThanOrEqual,
                },
                _ => return None,
            };
            Some(ParsedComputedExpression {
                kind: operator,
                span: source_span(source, binary.span),
            })
        }
        Expression::LogicalExpression(logical) => {
            let kind = match logical.operator.as_str() {
                "&&" => ParsedComputedExpressionKind::Logical {
                    left: Box::new(parsed_computed_expression(&logical.left, source)?),
                    right: Box::new(parsed_computed_expression(&logical.right, source)?),
                    operator: ParsedLogicalOperator::And,
                },
                "||" => ParsedComputedExpressionKind::Logical {
                    left: Box::new(parsed_computed_expression(&logical.left, source)?),
                    right: Box::new(parsed_computed_expression(&logical.right, source)?),
                    operator: ParsedLogicalOperator::Or,
                },
                "??" => ParsedComputedExpressionKind::NullishCoalescing {
                    left: Box::new(parsed_computed_expression(&logical.left, source)?),
                    right: Box::new(parsed_computed_expression(&logical.right, source)?),
                },
                _ => return None,
            };
            Some(ParsedComputedExpression {
                kind,
                span: source_span(source, logical.span),
            })
        }
        Expression::UnaryExpression(unary) => {
            let operator = match unary.operator.as_str() {
                "!" => ParsedUnaryOperator::Not,
                "+" => ParsedUnaryOperator::Plus,
                "-" => ParsedUnaryOperator::Minus,
                _ => return None,
            };
            Some(ParsedComputedExpression {
                kind: ParsedComputedExpressionKind::Unary {
                    operand: Box::new(parsed_computed_expression(&unary.argument, source)?),
                    operator,
                },
                span: source_span(source, unary.span),
            })
        }
        _ => None,
    }
}

fn parsed_return_value(statement: &Statement<'_>) -> Option<ParsedSerializableValue> {
    let Statement::ReturnStatement(return_statement) = statement else {
        return None;
    };
    serializable_value_from_expression(return_statement.argument.as_ref()?)
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
        "=" => match &assignment.right {
            Expression::Identifier(identifier) => {
                ParsedStateOperation::AssignParameter(identifier.name.to_string())
            }
            expression => {
                ParsedStateOperation::Assign(serializable_value_from_expression(expression)?)
            }
        },
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
        name_span: source_span(source, element.opening_element.name.span()),
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
        JSXElementName::NamespacedName(namespaced) => Some(format!(
            "{}:{}",
            namespaced.namespace.name, namespaced.name.name
        )),
        JSXElementName::MemberExpression(member) => Some(jsx_member_expression_name(member)),
        JSXElementName::ThisExpression(_) => Some("this".to_string()),
    }
}

fn jsx_member_expression_name(member: &JSXMemberExpression<'_>) -> String {
    let object = match &member.object {
        JSXMemberExpressionObject::IdentifierReference(identifier) => identifier.name.to_string(),
        JSXMemberExpressionObject::MemberExpression(member) => jsx_member_expression_name(member),
        JSXMemberExpressionObject::ThisExpression(_) => "this".to_string(),
    };
    format!("{object}.{}", member.property.name)
}

fn parsed_jsx_attribute(attribute: &JSXAttributeItem<'_>, source: &str) -> ParsedJsxAttribute {
    match attribute {
        JSXAttributeItem::Attribute(attribute) => {
            let name = jsx_attribute_name(&attribute.name);
            let expression = attribute.value.as_ref().and_then(|value| match value {
                JSXAttributeValue::ExpressionContainer(container) => {
                    container.expression.as_expression()
                }
                _ => None,
            });
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
                name_span: source_span(source, attribute.name.span()),
                value_span: attribute
                    .value
                    .as_ref()
                    .map(|value| source_span(source, value.span())),
                expression_span: expression
                    .map(|expression| source_span(source, expression.span())),
                this_member: expression
                    .and_then(|expression| parsed_this_member_expression(expression, source)),
                constant_value: match &attribute.value {
                    Some(JSXAttributeValue::StringLiteral(literal)) => {
                        Some(ParsedSerializableValue::String(literal.value.to_string()))
                    }
                    Some(JSXAttributeValue::ExpressionContainer(container)) => container
                        .expression
                        .as_expression()
                        .and_then(serializable_value_from_expression),
                    _ => None,
                },
                span: source_span(source, attribute.span),
            }
        }
        JSXAttributeItem::SpreadAttribute(spread) => ParsedJsxAttribute {
            name: "{...}".to_string(),
            value: ParsedJsxAttributeValue::Spread(expression_summary(&spread.argument)),
            name_span: source_span(source, spread.span),
            value_span: Some(source_span(source, spread.argument.span())),
            expression_span: Some(source_span(source, spread.argument.span())),
            this_member: parsed_this_member_expression(&spread.argument, source),
            constant_value: serializable_value_from_expression(&spread.argument),
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

    let (handler, arguments) = jsx_expression_event_handler_ref(&container.expression)?;

    Some(ParsedEventHandler {
        event,
        handler,
        arguments,
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

fn jsx_expression_event_handler_ref(
    expression: &JSXExpression<'_>,
) -> Option<(String, Vec<ParsedSerializableValue>)> {
    let expression = expression.as_expression()?;
    expression_event_handler_ref(expression)
}

fn expression_event_handler_ref(
    expression: &Expression<'_>,
) -> Option<(String, Vec<ParsedSerializableValue>)> {
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
        Expression::CallExpression(call) => Some((
            expression_summary(&call.callee)?,
            call.arguments
                .iter()
                .map(|argument| serializable_value_from_expression(argument.as_expression()?))
                .collect::<Option<Vec<_>>>()?,
        )),
        Expression::StaticMemberExpression(_) => {
            Some((expression_summary(expression)?, Vec::new()))
        }
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
