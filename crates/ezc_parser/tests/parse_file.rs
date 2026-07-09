use ezc_parser::{parse_file, ParseSeverity, ParsedJsxChild};

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
        render.jsx_roots[0].event_handler_refs,
        vec!["this.increment"]
    );
    assert_eq!(
        render.jsx_roots[0].children,
        vec![
            ParsedJsxChild::Text("Count:".to_string()),
            ParsedJsxChild::Binding("this.count".to_string()),
        ]
    );
    assert_eq!(render.bindings, vec!["this.count"]);
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
