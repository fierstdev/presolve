use std::env;
use std::fs;
use std::path::PathBuf;

use oxc_allocator::Allocator;
use oxc_ast::ast::{
    ClassElement, Declaration, Expression, JSXElementName, MethodDefinitionKind, Program, Statement,
};
use oxc_parser::Parser;
use oxc_span::{GetSpan, SourceType};

fn main() {
    let path = env::args()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("fixtures/0001-source-summary/input/Counter.tsx"));

    let source = fs::read_to_string(&path).expect("failed to read source file");

    let source_type = SourceType::from_path(&path)
        .unwrap_or_default()
        .with_typescript(true)
        .with_jsx(true);

    let allocator = Allocator::default();
    let ret = Parser::new(&allocator, &source, source_type).parse();

    println!("File: {}", path.display());
    println!("Errors: {}", ret.errors.len());

    for error in &ret.errors {
        println!("  {error:?}");
    }

    inspect_program(&ret.program);
}

fn inspect_program(program: &Program<'_>) {
    println!("Top-level statements: {}", program.body.len());

    for statement in &program.body {
        inspect_statement(statement);
    }
}

fn inspect_statement(statement: &Statement<'_>) {
    if let Some(declaration) = statement.as_declaration() {
        inspect_declaration(declaration);
        return;
    }

    println!("Statement: {:?} span={:?}", statement, statement.span());
}

fn inspect_declaration(declaration: &Declaration<'_>) {
    match declaration {
        Declaration::ClassDeclaration(class) => {
            let name = class
                .id
                .as_ref()
                .map(|id| id.name.as_str())
                .unwrap_or("<anonymous>");

            println!("ClassDeclaration: {name} span={:?}", class.span);

            println!("  decorators: {}", class.decorators.len());
            for decorator in &class.decorators {
                println!(
                    "    decorator span={:?} expression={:?}",
                    decorator.span, decorator.expression
                );
            }

            println!("  body elements: {}", class.body.body.len());
            for element in &class.body.body {
                inspect_class_element(element);
            }
        }
        other => {
            println!("Declaration: {:?} span={:?}", other, other.span());
        }
    }
}

fn inspect_class_element(element: &ClassElement<'_>) {
    match element {
        ClassElement::MethodDefinition(method) => {
            let kind = match method.kind {
                MethodDefinitionKind::Constructor => "constructor",
                MethodDefinitionKind::Method => "method",
                MethodDefinitionKind::Get => "get",
                MethodDefinitionKind::Set => "set",
            };

            println!(
                "  MethodDefinition: kind={} span={:?} key={:?}",
                kind, method.span, method.key
            );

            if let Some(body) = &method.value.body {
                for statement in &body.statements {
                    inspect_method_statement(statement);
                }
            }
        }
        other => {
            println!("  ClassElement: {:?} span={:?}", other, other.span());
        }
    }
}

fn inspect_method_statement(statement: &Statement<'_>) {
    match statement {
        Statement::ReturnStatement(return_statement) => {
            println!("    ReturnStatement span={:?}", return_statement.span);

            if let Some(argument) = &return_statement.argument {
                inspect_expression(argument, 6);
            }
        }
        other => {
            println!("    Statement: {:?} span={:?}", other, other.span());
        }
    }
}

fn inspect_expression(expression: &Expression<'_>, indent: usize) {
    let pad = " ".repeat(indent);

    match expression {
        Expression::ParenthesizedExpression(parenthesized) => {
            println!("{pad}ParenthesizedExpression span={:?}", parenthesized.span);
            inspect_expression(&parenthesized.expression, indent + 2);
        }
        Expression::JSXElement(element) => {
            println!("{pad}JSXElement span={:?}", element.span);
            print_jsx_name(&element.opening_element.name, indent + 2);

            println!(
                "{pad}  attributes: {}",
                element.opening_element.attributes.len()
            );

            println!("{pad}  children: {}", element.children.len());
        }
        Expression::JSXFragment(fragment) => {
            println!("{pad}JSXFragment span={:?}", fragment.span);
        }
        other => {
            println!("{pad}Expression: {:?} span={:?}", other, other.span());
        }
    }
}

fn print_jsx_name(name: &JSXElementName<'_>, indent: usize) {
    let pad = " ".repeat(indent);

    match name {
        JSXElementName::Identifier(identifier) => {
            println!("{pad}JSX name: {}", identifier.name);
        }
        JSXElementName::IdentifierReference(identifier) => {
            println!("{pad}JSX identifier reference: {}", identifier.name);
        }
        JSXElementName::NamespacedName(namespaced) => {
            println!("{pad}JSX namespaced name: {:?}", namespaced);
        }
        JSXElementName::MemberExpression(member) => {
            println!("{pad}JSX member name: {:?}", member);
        }
        JSXElementName::ThisExpression(_) => {
            println!("{pad}JSX this expression");
        }
    }
}
