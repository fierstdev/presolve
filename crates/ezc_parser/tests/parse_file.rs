use ezc_parser::{
    parse_file, ParseSeverity, ParsedJsxAttributeValue, ParsedJsxChild, ParsedJsxElement,
    ParsedJsxNode, ParsedSerializableValue, ParsedStateOperation,
};

fn jsx_root_element(root: &ParsedJsxNode) -> &ParsedJsxElement {
    let ParsedJsxNode::Element(element) = root else {
        panic!("expected JSX element root");
    };

    element
}

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
    let root = jsx_root_element(&render.jsx_roots[0]);
    assert_eq!(root.name, "button");
    assert_eq!(root.attributes.len(), 1);
    assert_eq!(root.attributes[0].name, "onClick");
    assert!(matches!(
        root.attributes[0].value,
        ParsedJsxAttributeValue::Expression(_)
    ));
    assert_eq!(root.event_handlers.len(), 1);
    assert_eq!(root.event_handlers[0].event, "click");
    assert_eq!(root.event_handlers[0].handler, "this.increment");
    assert_eq!(root.event_handlers[0].span.line, 12);
    assert_eq!(root.event_handlers[0].span.column, 15);

    assert_eq!(root.children.len(), 2);
    let ParsedJsxChild::Text { value, span } = &root.children[0] else {
        panic!("expected text child");
    };
    assert_eq!(value, "Count:");
    assert_eq!(span.line, 13);
    assert_eq!(span.column, 9);

    let ParsedJsxChild::Binding { expression, span } = &root.children[1] else {
        panic!("expected binding child");
    };
    assert_eq!(expression, "this.count");
    assert_eq!(span.line, 13);
    assert_eq!(span.column, 16);
    assert_eq!(render.bindings, vec!["this.count"]);

    let increment = class
        .methods
        .iter()
        .find(|method| method.name == "increment")
        .expect("expected increment method");

    assert_eq!(increment.state_updates.len(), 1);
    assert_eq!(increment.state_updates[0].field, "count");
    assert_eq!(
        increment.state_updates[0].operation,
        ParsedStateOperation::AddAssign(ParsedSerializableValue::Number("1".to_string()))
    );
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
fn parses_decrement_state_update() {
    let source = r#"
@component("x-decrement-counter")
class DecrementCounter extends Component {
  count = state(2);

  decrement() {
    this.count--;
  }

  render() {
    return <button onClick={() => this.decrement()}>Count: {this.count}</button>;
  }
}
"#;

    let parsed = parse_file("DecrementCounter.tsx", source);

    assert!(parsed.diagnostics.is_empty());

    let class = parsed.classes.first().expect("expected class");
    let decrement = class
        .methods
        .iter()
        .find(|method| method.name == "decrement")
        .expect("expected decrement method");

    assert_eq!(decrement.state_updates.len(), 1);
    assert_eq!(decrement.state_updates[0].field, "count");
    assert_eq!(
        decrement.state_updates[0].operation,
        ParsedStateOperation::Decrement
    );
}

#[test]
fn parses_add_and_subtract_assign_state_updates() {
    let source = r#"
@component("x-step-counter")
class StepCounter extends Component {
  count = state(4);

  addTwo() {
    this.count += 2;
  }

  subtractThree() {
    this.count -= 3;
  }

  render() {
    return <button onClick={() => this.addTwo()}>Count: {this.count}</button>;
  }
}
"#;

    let parsed = parse_file("StepCounter.tsx", source);

    assert!(parsed.diagnostics.is_empty());

    let class = parsed.classes.first().expect("expected class");
    let add_two = class
        .methods
        .iter()
        .find(|method| method.name == "addTwo")
        .expect("expected addTwo method");
    let subtract_three = class
        .methods
        .iter()
        .find(|method| method.name == "subtractThree")
        .expect("expected subtractThree method");

    assert_eq!(add_two.state_updates.len(), 1);
    assert_eq!(add_two.state_updates[0].field, "count");
    assert_eq!(
        add_two.state_updates[0].operation,
        ParsedStateOperation::AddAssign(ParsedSerializableValue::Number("2".to_string()))
    );

    assert_eq!(subtract_three.state_updates.len(), 1);
    assert_eq!(subtract_three.state_updates[0].field, "count");
    assert_eq!(
        subtract_three.state_updates[0].operation,
        ParsedStateOperation::SubtractAssign(ParsedSerializableValue::Number("3".to_string()))
    );
}

#[test]
fn parses_direct_literal_assignment_state_update() {
    let source = r#"
@component("x-reset-counter")
class ResetCounter extends Component {
  count = state(5);

  reset() {
    this.count = 0;
  }

  render() {
    return <button onClick={() => this.reset()}>Count: {this.count}</button>;
  }
}
"#;

    let parsed = parse_file("ResetCounter.tsx", source);

    assert!(parsed.diagnostics.is_empty());

    let class = parsed.classes.first().expect("expected class");
    let reset = class
        .methods
        .iter()
        .find(|method| method.name == "reset")
        .expect("expected reset method");

    assert_eq!(reset.state_updates.len(), 1);
    assert_eq!(reset.state_updates[0].field, "count");
    assert_eq!(
        reset.state_updates[0].operation,
        ParsedStateOperation::Assign(ParsedSerializableValue::Number("0".to_string()))
    );
}

#[test]
fn parses_boolean_toggle_state_update() {
    let source = r#"
@component("x-toggle-flag")
class ToggleFlag extends Component {
  enabled = state(false);

  toggle() {
    this.enabled = !this.enabled;
  }

  render() {
    return <button onClick={() => this.toggle()}>Enabled: {this.enabled}</button>;
  }
}
"#;

    let parsed = parse_file("ToggleFlag.tsx", source);

    assert!(parsed.diagnostics.is_empty());

    let class = parsed.classes.first().expect("expected class");
    let toggle = class
        .methods
        .iter()
        .find(|method| method.name == "toggle")
        .expect("expected toggle method");

    assert_eq!(toggle.state_updates.len(), 1);
    assert_eq!(toggle.state_updates[0].field, "enabled");
    assert_eq!(
        toggle.state_updates[0].operation,
        ParsedStateOperation::Toggle
    );
}

#[test]
fn parses_multi_step_state_updates_in_source_order() {
    let source =
        include_str!("../../../fixtures/0013-multi-step-action/input/BatchActionCounter.tsx");

    let parsed = parse_file(
        "fixtures/0013-multi-step-action/input/BatchActionCounter.tsx",
        source,
    );

    assert!(parsed.diagnostics.is_empty());

    let class = parsed.classes.first().expect("expected class");
    let apply = class
        .methods
        .iter()
        .find(|method| method.name == "apply")
        .expect("expected apply method");

    assert_eq!(apply.state_updates.len(), 5);

    assert_eq!(apply.state_updates[0].field, "count");
    assert_eq!(
        apply.state_updates[0].operation,
        ParsedStateOperation::AddAssign(ParsedSerializableValue::Number("2".to_string()))
    );
    assert_eq!(apply.state_updates[1].field, "count");
    assert_eq!(
        apply.state_updates[1].operation,
        ParsedStateOperation::Decrement
    );
    assert_eq!(apply.state_updates[2].field, "count");
    assert_eq!(
        apply.state_updates[2].operation,
        ParsedStateOperation::Assign(ParsedSerializableValue::Number("8".to_string()))
    );
    assert_eq!(apply.state_updates[3].field, "count");
    assert_eq!(
        apply.state_updates[3].operation,
        ParsedStateOperation::Increment
    );
    assert_eq!(apply.state_updates[4].field, "enabled");
    assert_eq!(
        apply.state_updates[4].operation,
        ParsedStateOperation::Toggle
    );
}

#[test]
fn parses_jsx_attributes_as_structured_values() {
    let source = r#"
@component("x-attrs")
class Attrs extends Component {
  disabled = state(false);

  render() {
    return <button type="button" disabled data-mode="safe" title={this.disabled} {...props}>Go</button>;
  }
}
"#;

    let parsed = parse_file("Attrs.tsx", source);

    assert!(parsed.diagnostics.is_empty());

    let class = parsed.classes.first().expect("expected class");
    let render = class
        .methods
        .iter()
        .find(|method| method.name == "render")
        .expect("expected render method");
    let root = jsx_root_element(render.jsx_roots.first().expect("expected JSX root"));

    assert_eq!(root.attributes.len(), 5);

    assert_eq!(root.attributes[0].name, "type");
    assert_eq!(
        root.attributes[0].value,
        ParsedJsxAttributeValue::Static("button".to_string())
    );

    assert_eq!(root.attributes[1].name, "disabled");
    assert_eq!(root.attributes[1].value, ParsedJsxAttributeValue::Boolean);

    assert_eq!(root.attributes[2].name, "data-mode");
    assert_eq!(
        root.attributes[2].value,
        ParsedJsxAttributeValue::Static("safe".to_string())
    );

    assert_eq!(root.attributes[3].name, "title");
    assert_eq!(
        root.attributes[3].value,
        ParsedJsxAttributeValue::Expression(Some("this.disabled".to_string()))
    );

    assert_eq!(root.attributes[4].name, "{...}");
    assert_eq!(
        root.attributes[4].value,
        ParsedJsxAttributeValue::Spread(Some("props".to_string()))
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

    let section = jsx_root_element(&render.jsx_roots[0]);
    assert_eq!(section.name, "section");

    assert_eq!(section.children.len(), 1);

    let ParsedJsxChild::Element(button) = &section.children[0] else {
        panic!("expected nested button element");
    };

    assert_eq!(button.name, "button");
    assert_eq!(button.attributes.len(), 1);
    assert_eq!(button.attributes[0].name, "onClick");
    assert!(matches!(
        button.attributes[0].value,
        ParsedJsxAttributeValue::Expression(_)
    ));
    assert_eq!(button.event_handlers.len(), 1);
    assert_eq!(button.event_handlers[0].event, "click");
    assert_eq!(button.event_handlers[0].handler, "this.increment");
    assert_eq!(button.event_handlers[0].span.line, 13);
    assert_eq!(button.event_handlers[0].span.column, 17);

    assert_eq!(button.children.len(), 2);
    let ParsedJsxChild::Text { value, span } = &button.children[0] else {
        panic!("expected nested text child");
    };
    assert_eq!(value, "Count:");
    assert_eq!(span.line, 13);
    assert_eq!(span.column, 50);

    let ParsedJsxChild::Binding { expression, span } = &button.children[1] else {
        panic!("expected nested binding child");
    };
    assert_eq!(expression, "this.count");
    assert_eq!(span.line, 13);
    assert_eq!(span.column, 57);
}

#[test]
fn parses_jsx_fragments() {
    let source = include_str!("../../../fixtures/0016-fragments/input/FragmentPanel.tsx");

    let parsed = parse_file("fixtures/0016-fragments/input/FragmentPanel.tsx", source);

    assert!(parsed.diagnostics.is_empty());

    let class = parsed.classes.first().expect("expected class");
    let render = class
        .methods
        .iter()
        .find(|method| method.name == "render")
        .expect("expected render method");

    assert_eq!(render.jsx_roots.len(), 1);

    let ParsedJsxNode::Fragment(root) = &render.jsx_roots[0] else {
        panic!("expected fragment root");
    };

    assert_eq!(root.span.line, 8);
    assert_eq!(root.span.column, 7);
    assert_eq!(root.children.len(), 2);

    let ParsedJsxChild::Element(heading) = &root.children[0] else {
        panic!("expected heading child");
    };
    assert_eq!(heading.name, "h1");

    let ParsedJsxChild::Fragment(nested) = &root.children[1] else {
        panic!("expected nested fragment child");
    };
    assert_eq!(nested.span.line, 10);
    assert_eq!(nested.children.len(), 2);

    let ParsedJsxChild::Element(paragraph) = &nested.children[0] else {
        panic!("expected paragraph child");
    };
    assert_eq!(paragraph.name, "p");
    assert_eq!(paragraph.children.len(), 2);

    let ParsedJsxChild::Binding { expression, .. } = &paragraph.children[1] else {
        panic!("expected paragraph binding");
    };
    assert_eq!(expression, "this.label");
    assert_eq!(render.bindings, vec!["this.label"]);
}
