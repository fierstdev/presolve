use ezc_parser::{
    parse_file, ParseSeverity, ParsedEventHandler, ParsedJsxChild, ParsedSerializableValue,
    ParsedStateOperation,
};

#[test]
fn parses_counter_fixture() {
    let source = include_str!("../../../fixtures/0001-source-summary/input/Counter.tsx");

    let parsed = parse_file("fixtures/0001-source-summary/input/Counter.tsx", source);

    assert!(parsed.diagnostics.is_empty());

    let class = parsed.classes.first().expect("expected class");
    assert_eq!(class.name, "Counter");

    assert_eq!(class.decorators.len(), 2);
    assert_eq!(class.decorators[0].name, "route");
    assert_eq!(class.decorators[0].argument.as_deref(), Some("/counter"));
    assert_eq!(class.decorators[1].name, "component");
    assert_eq!(class.decorators[1].argument.as_deref(), Some("x-counter"));

    assert_eq!(class.properties.len(), 1);
    assert_eq!(class.properties[0].name, "count");
    assert_eq!(
        class.properties[0].initializer.as_deref(),
        Some("state(...)")
    );
    assert_eq!(
        class.properties[0].state_initial_value,
        Some(ParsedSerializableValue::Number("0".to_string()))
    );

    let method_names = class
        .methods
        .iter()
        .map(|method| method.name.as_str())
        .collect::<Vec<_>>();

    assert_eq!(method_names, vec!["increment", "render"]);

    let render = class
        .methods
        .iter()
        .find(|method| method.name == "render")
        .expect("expected render method");

    assert_eq!(render.jsx_roots.len(), 1);
    assert_eq!(render.jsx_roots[0].name, "button");
    assert_eq!(render.jsx_roots[0].attributes, vec!["onClick={...}"]);
    assert_eq!(
        render.jsx_roots[0].event_handlers,
        vec![ParsedEventHandler {
            event: "click".to_string(),
            handler: "this.increment".to_string(),
        }]
    );
    assert_eq!(
        render.jsx_roots[0].children,
        vec![
            ParsedJsxChild::Text("Count:".to_string()),
            ParsedJsxChild::Binding("this.count".to_string()),
        ]
    );
    assert_eq!(render.bindings, vec!["this.count"]);

    let increment = class
        .methods
        .iter()
        .find(|method| method.name == "increment")
        .expect("expected increment method");

    assert!(increment.state_updates.is_empty());
}

#[test]
fn parses_string_state_literal_without_source_quotes() {
    let source = include_str!("../../../fixtures/0006-string-state/input/StringGreeting.tsx");

    let parsed = parse_file(
        "fixtures/0006-string-state/input/StringGreeting.tsx",
        source,
    );

    assert!(parsed.diagnostics.is_empty());

    let class = parsed.classes.first().expect("expected class");
    assert_eq!(class.properties.len(), 1);
    assert_eq!(class.properties[0].name, "name");
    assert_eq!(
        class.properties[0].initializer.as_deref(),
        Some("state(...)")
    );
    assert_eq!(
        class.properties[0].state_initial_value,
        Some(ParsedSerializableValue::String(
            "Austin & <Zero>".to_string()
        ))
    );
}

#[test]
fn parses_boolean_state_literals() {
    let source = include_str!("../../../fixtures/0007-boolean-state/input/BooleanFlags.tsx");

    let parsed = parse_file("fixtures/0007-boolean-state/input/BooleanFlags.tsx", source);

    assert!(parsed.diagnostics.is_empty());

    let class = parsed.classes.first().expect("expected class");
    assert_eq!(class.properties.len(), 2);
    assert_eq!(class.properties[0].name, "enabled");
    assert_eq!(
        class.properties[0].state_initial_value,
        Some(ParsedSerializableValue::Boolean(true))
    );
    assert_eq!(class.properties[1].name, "disabled");
    assert_eq!(
        class.properties[1].state_initial_value,
        Some(ParsedSerializableValue::Boolean(false))
    );
}

#[test]
fn parses_null_state_literal() {
    let source = include_str!("../../../fixtures/0008-null-state/input/NullSelection.tsx");

    let parsed = parse_file("fixtures/0008-null-state/input/NullSelection.tsx", source);

    assert!(parsed.diagnostics.is_empty());

    let class = parsed.classes.first().expect("expected class");
    assert_eq!(class.properties.len(), 1);
    assert_eq!(class.properties[0].name, "selection");
    assert_eq!(
        class.properties[0].state_initial_value,
        Some(ParsedSerializableValue::Null)
    );
}

#[test]
fn reports_broken_tsx_diagnostic() {
    let source = include_str!("../../../fixtures/0002-broken-tsx/input/BrokenCounter.tsx");

    let parsed = parse_file("fixtures/0002-broken-tsx/input/BrokenCounter.tsx", source);

    assert_eq!(parsed.classes.len(), 0);
    assert_eq!(parsed.diagnostics.len(), 1);

    let diagnostic = &parsed.diagnostics[0];

    assert_eq!(diagnostic.message, "Unexpected token");
    assert_eq!(diagnostic.severity, ParseSeverity::Error);
    assert_eq!(diagnostic.labels.len(), 1);
    assert_eq!(diagnostic.labels[0].span.start, 198);
    assert_eq!(diagnostic.labels[0].span.end, 199);
    assert_eq!(diagnostic.labels[0].span.line, 9);
    assert_eq!(diagnostic.labels[0].span.column, 16);
}

#[test]
fn parses_nested_jsx_fixture() {
    let source = include_str!("../../../fixtures/0004-nested-jsx/input/NestedCounter.tsx");

    let parsed = parse_file("fixtures/0004-nested-jsx/input/NestedCounter.tsx", source);

    assert!(parsed.diagnostics.is_empty());

    let class = parsed.classes.first().expect("expected class");
    assert_eq!(class.name, "NestedCounter");

    let increment = class
        .methods
        .iter()
        .find(|method| method.name == "increment")
        .expect("expected increment method");

    assert_eq!(increment.state_updates.len(), 1);
    assert_eq!(increment.state_updates[0].field, "count");
    assert_eq!(
        increment.state_updates[0].operation,
        ParsedStateOperation::Increment
    );

    let render = class
        .methods
        .iter()
        .find(|method| method.name == "render")
        .expect("expected render method");

    assert_eq!(render.jsx_roots.len(), 1);

    let section = &render.jsx_roots[0];
    assert_eq!(section.name, "section");

    assert_eq!(section.children.len(), 1);

    let ParsedJsxChild::Element(button) = &section.children[0] else {
        panic!("expected nested button element");
    };

    assert_eq!(button.name, "button");
    assert_eq!(button.attributes, vec!["onClick={...}"]);
    assert_eq!(
        button.event_handlers,
        vec![ParsedEventHandler {
            event: "click".to_string(),
            handler: "this.increment".to_string(),
        }]
    );

    assert_eq!(
        button.children,
        vec![
            ParsedJsxChild::Text("Count:".to_string()),
            ParsedJsxChild::Binding("this.count".to_string()),
        ]
    );
}
