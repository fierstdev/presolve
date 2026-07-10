const RUNTIME_STUB: &str = r#"(() => {
  "use strict";

  const MANIFEST_ELEMENT_ID = "ez-template-manifest";
  const RUNTIME_VERSION = "0.0.0";
  const SUPPORTED_SCHEMA_VERSION = 1;

  class EdgeZeroBootError extends Error {
    constructor(code) {
      super(code);
      this.name = "EdgeZeroBootError";
      this.code = code;
    }
  }

  function createDiagnostic(code, message, detail, fatal = false) {
    return {
      code,
      message,
      detail,
      fatal
    };
  }

  function reportDiagnostic(diagnostics, code, message, detail, fatal = false) {
    const diagnostic = createDiagnostic(code, message, detail, fatal);
    diagnostics.push(diagnostic);
    console.error(`[EdgeZero] ${code}`, diagnostic);
    return diagnostic;
  }

  function readManifest(diagnostics) {
    const element = document.getElementById(MANIFEST_ELEMENT_ID);

    if (!(element instanceof HTMLScriptElement)) {
      reportDiagnostic(
        diagnostics,
        "EZR_MISSING_MANIFEST",
        `Missing template manifest script #${MANIFEST_ELEMENT_ID}`,
        { manifestElementId: MANIFEST_ELEMENT_ID },
        true
      );
      throw new EdgeZeroBootError("EZR_MISSING_MANIFEST");
    }

    try {
      return JSON.parse(element.textContent ?? "");
    } catch (error) {
      reportDiagnostic(
        diagnostics,
        "EZR_INVALID_MANIFEST_JSON",
        "Template manifest JSON could not be parsed",
        { message: error instanceof Error ? error.message : String(error) },
        true
      );
      throw new EdgeZeroBootError("EZR_INVALID_MANIFEST_JSON");
    }
  }

  function validateManifestSchema(manifest, diagnostics) {
    if (manifest?.schema_version !== SUPPORTED_SCHEMA_VERSION) {
      reportDiagnostic(
        diagnostics,
        "EZR_UNSUPPORTED_SCHEMA",
        `Unsupported template manifest schema version ${String(manifest?.schema_version)}`,
        {
          schema_version: manifest?.schema_version,
          supported_schema_version: SUPPORTED_SCHEMA_VERSION
        },
        true
      );
      throw new EdgeZeroBootError("EZR_UNSUPPORTED_SCHEMA");
    }
  }

  function normalizeHandlerReference(reference) {
    return String(reference ?? "").replace(/^this\./, "");
  }

  function fieldNameFromThisMember(expression) {
    const match = /^this\.([A-Za-z_$][\w$]*)$/.exec(String(expression ?? ""));
    return match === null ? null : match[1];
  }

  function collectBindingAnchors() {
    const anchors = new Map();
    const walker = document.createTreeWalker(
      document.body,
      NodeFilter.SHOW_COMMENT
    );

    while (walker.nextNode()) {
      const value = (walker.currentNode.nodeValue ?? "").trim();
      const match = /^ez-binding:([^:]+):(.*)$/.exec(value);

      if (match !== null) {
        anchors.set(match[1], {
          id: match[1],
          expression: match[2],
          marker: walker.currentNode
        });
      }
    }

    return anchors;
  }

  function collectConditionalAnchors() {
    const starts = new Map();
    const ends = new Map();
    const walker = document.createTreeWalker(
      document.body,
      NodeFilter.SHOW_COMMENT
    );

    while (walker.nextNode()) {
      const value = (walker.currentNode.nodeValue ?? "").trim();
      const startMatch = /^ez-conditional-start:([^:]+):(.*)$/.exec(value);

      if (startMatch !== null) {
        starts.set(startMatch[1], {
          id: startMatch[1],
          condition: startMatch[2],
          marker: walker.currentNode
        });
        continue;
      }

      const endMatch = /^ez-conditional-end:([^:]+)$/.exec(value);

      if (endMatch !== null) {
        ends.set(endMatch[1], {
          id: endMatch[1],
          marker: walker.currentNode
        });
      }
    }

    return {
      starts,
      ends
    };
  }

  function collectListAnchors() {
    const starts = new Map();
    const ends = new Map();
    const walker = document.createTreeWalker(
      document.body,
      NodeFilter.SHOW_COMMENT
    );

    while (walker.nextNode()) {
      const value = (walker.currentNode.nodeValue ?? "").trim();
      const startMatch = /^ez-list-start:([^:]+):(.*)$/.exec(value);

      if (startMatch !== null) {
        starts.set(startMatch[1], {
          id: startMatch[1],
          iterable: startMatch[2],
          marker: walker.currentNode
        });
        continue;
      }

      const endMatch = /^ez-list-end:([^:]+)$/.exec(value);

      if (endMatch !== null) {
        ends.set(endMatch[1], {
          id: endMatch[1],
          marker: walker.currentNode
        });
      }
    }

    return {
      starts,
      ends
    };
  }

  function collectElementAnchors() {
    const elementsByNode = new Map();

    for (const element of document.querySelectorAll("[data-ez-node]")) {
      elementsByNode.set(element.dataset.ezNode, element);
    }

    return elementsByNode;
  }

  function collectMissingAnchors(
    manifest,
    bindingAnchors,
    conditionalAnchors,
    listAnchors,
    elementsByNode
  ) {
    const missing = [];

    for (const component of manifest.components ?? []) {
      for (const node of component.template?.nodes ?? []) {
        if (node.kind === "element") {
          if (!elementsByNode.has(node.id)) {
            missing.push({
              component: component.name,
              id: node.id,
              kind: node.kind,
              code: "EZR_MISSING_ELEMENT_ANCHOR"
            });
          }
        }

        if (
          node.kind === "binding" &&
          node.target !== "attribute" &&
          !bindingAnchors.has(node.id)
        ) {
          missing.push({
            component: component.name,
            id: node.id,
            kind: node.kind,
            code: "EZR_MISSING_BINDING_ANCHOR"
          });
        }

        if (node.kind === "conditional") {
          if (!conditionalAnchors.starts.has(node.start)) {
            missing.push({
              component: component.name,
              id: node.start,
              kind: node.kind,
              code: "EZR_MISSING_CONDITIONAL_ANCHOR"
            });
          }

          if (!conditionalAnchors.ends.has(node.end)) {
            missing.push({
              component: component.name,
              id: node.end,
              kind: node.kind,
              code: "EZR_MISSING_CONDITIONAL_ANCHOR"
            });
          }
        }

        if (node.kind === "list") {
          if (!listAnchors.starts.has(node.start)) {
            missing.push({
              component: component.name,
              id: node.start,
              kind: node.kind,
              code: "EZR_MISSING_LIST_ANCHOR"
            });
          }

          if (!listAnchors.ends.has(node.end)) {
            missing.push({
              component: component.name,
              id: node.end,
              kind: node.kind,
              code: "EZR_MISSING_LIST_ANCHOR"
            });
          }
        }
      }
    }

    return missing;
  }

  function buildActionsByMethod(component) {
    const actionsByMethod = new Map();

    for (const action of component.actions ?? []) {
      const actions = actionsByMethod.get(action.method) ?? [];
      actions.push(action);
      actionsByMethod.set(action.method, actions);
    }

    return actionsByMethod;
  }

  function componentFieldKey(componentName, field) {
    return `${componentName}:${field}`;
  }

  function componentMethodKey(componentName, method) {
    return `${componentName}:${method}`;
  }

  function formatBindingValue(value) {
    return value === null ? "" : String(value);
  }

  function isBooleanAttribute(attribute) {
    return new Set([
      "allowfullscreen",
      "async",
      "autofocus",
      "autoplay",
      "checked",
      "controls",
      "default",
      "defer",
      "disabled",
      "formnovalidate",
      "hidden",
      "inert",
      "loop",
      "multiple",
      "muted",
      "nomodule",
      "novalidate",
      "open",
      "readonly",
      "required",
      "reversed",
      "selected"
    ]).has(String(attribute).toLowerCase());
  }

  function isPropertyAttribute(attribute) {
    return new Set([
      "checked",
      "disabled",
      "selected",
      "value"
    ]).has(String(attribute).toLowerCase());
  }

  function updateAttributeBinding(element, attribute, value) {
    const normalizedAttribute = String(attribute);

    if (isBooleanAttribute(normalizedAttribute)) {
      const enabled = Boolean(value);
      element.toggleAttribute(normalizedAttribute, enabled);

      if (isPropertyAttribute(normalizedAttribute) && normalizedAttribute in element) {
        element[normalizedAttribute] = enabled;
      }

      return;
    }

    if (value === null || value === undefined) {
      element.removeAttribute(normalizedAttribute);

      if (isPropertyAttribute(normalizedAttribute) && normalizedAttribute in element) {
        element[normalizedAttribute] = "";
      }

      return;
    }

    const text = formatBindingValue(value);
    element.setAttribute(normalizedAttribute, text);

    if (isPropertyAttribute(normalizedAttribute) && normalizedAttribute in element) {
      element[normalizedAttribute] = text;
    }
  }

  function replaceConditionalBranch(store, startMarker, endMarker, html) {
    if (
      startMarker.parentNode === null ||
      endMarker.parentNode === null ||
      startMarker.parentNode !== endMarker.parentNode
    ) {
      reportDiagnostic(
        store.diagnostics,
        "EZR_MISSING_CONDITIONAL_ANCHOR",
        "Conditional anchor range was not contiguous in one parent",
        {}
      );
      return;
    }

    let current = startMarker.nextSibling;

    while (current !== null && current !== endMarker) {
      const next = current.nextSibling;
      current.remove();
      current = next;
    }

    const template = document.createElement("template");
    template.innerHTML = String(html ?? "");
    endMarker.parentNode.insertBefore(template.content, endMarker);
    store.elementsByNode = collectElementAnchors();
  }

  function listItems(value) {
    return Array.isArray(value) ? value : [];
  }

  function normalizeListKey(value) {
    return String(value).replaceAll("--", "—");
  }

  function listItemKey(node, item, index) {
    if (node.key_expression === node.item_variable) {
      return Array.isArray(item) ? String(index) : normalizeListKey(item);
    }

    if (node.index_variable === node.key_expression) {
      return String(index);
    }

    return String(index);
  }

  function escapeHtmlText(value) {
    return String(value)
      .replaceAll("&", "&amp;")
      .replaceAll("<", "&lt;")
      .replaceAll(">", "&gt;");
  }

  function escapeHtmlAttribute(value) {
    return escapeHtmlText(value).replaceAll('"', "&quot;");
  }

  function renderListItemHtml(node, item, index, key) {
    return String(node.item_template_html ?? "")
      .replaceAll("__ez_list_key__", escapeHtmlAttribute(key))
      .replaceAll("__ez_list_item__", escapeHtmlText(formatBindingValue(item)))
      .replaceAll("__ez_list_index__", String(index));
  }

  function initialListInstances(store, node, items) {
    const instances = new Map();

    for (const [index, item] of items.entries()) {
      const key = listItemKey(node, item, index);
      const element = store.elementsByNode.get(`${node.item_root}:${key}`);

      if (element !== undefined) {
        instances.set(key, { element, item, index });
      }
    }

    return instances;
  }

  function reconcileKeyedList(store, node, startMarker, endMarker, instances, value) {
    if (
      startMarker.parentNode === null ||
      endMarker.parentNode === null ||
      startMarker.parentNode !== endMarker.parentNode
    ) {
      reportDiagnostic(
        store.diagnostics,
        "EZR_MISSING_LIST_ANCHOR",
        "List anchor range was not contiguous in one parent",
        {}
      );
      return instances;
    }

    const parent = startMarker.parentNode;
    const nextInstances = new Map();
    const ordered = [];

    for (const [index, item] of listItems(value).entries()) {
      const key = listItemKey(node, item, index);

      if (nextInstances.has(key)) {
        reportDiagnostic(
          store.diagnostics,
          "EZR_DUPLICATE_LIST_KEY",
          "List update produced a duplicate key",
          { id: node.id, key }
        );
        continue;
      }

      let instance = instances.get(key);

      if (instance === undefined) {
        const template = document.createElement("template");
        template.innerHTML = renderListItemHtml(node, item, index, key);
        const element = template.content.firstElementChild;

        if (element === null) {
          reportDiagnostic(
            store.diagnostics,
            "EZR_INVALID_LIST_TEMPLATE",
            "List item template did not produce a root element",
            node
          );
          continue;
        }

        parent.insertBefore(element, endMarker);
        instance = { element, item, index };
      }

      instance.item = item;
      instance.index = index;
      nextInstances.set(key, instance);
      ordered.push(instance);
    }

    for (const [key, instance] of instances) {
      if (!nextInstances.has(key)) {
        instance.element.remove();
      }
    }

    let cursor = startMarker.nextSibling;

    for (const instance of ordered) {
      if (instance.element !== cursor) {
        parent.insertBefore(instance.element, cursor);
      }
      cursor = instance.element.nextSibling;
    }

    store.elementsByNode = collectElementAnchors();
    return nextInstances;
  }

  function createRuntimeStore(elementsByNode, diagnostics) {
    return {
      components: new Map(),
      bindingsByField: new Map(),
      actionsByMethod: new Map(),
      eventsByType: new Map(),
      elementsByNode,
      diagnostics
    };
  }

  function readField(store, component, field) {
    if (!(field in component.state)) {
      reportDiagnostic(
        store.diagnostics,
        "EZR_INVALID_STATE_OPERATION",
        "Action referenced a missing state field",
        { component: component.name, field }
      );
      return undefined;
    }

    return component.state[field];
  }

  function writeField(store, component, field, value) {
    if (!(field in component.state)) {
      reportDiagnostic(
        store.diagnostics,
        "EZR_INVALID_STATE_OPERATION",
        "Action referenced a missing state field",
        { component: component.name, field }
      );
      return;
    }

    component.state[field] = value;
    notifyField(store, component, field);
  }

  function notifyField(store, component, field) {
    const bindings = store.bindingsByField.get(
      componentFieldKey(component.name, field)
    );

    if (bindings === undefined) {
      reportDiagnostic(
        store.diagnostics,
        "EZR_MISSING_BINDING_ANCHOR",
        "State field has no registered binding anchor",
        { component: component.name, field }
      );
      return;
    }

    for (const updateBinding of bindings) {
      updateBinding(component.state[field]);
    }
  }

  function registerBinding(store, component, field, updateBinding) {
    const key = componentFieldKey(component.name, field);
    const bindings = store.bindingsByField.get(key) ?? [];
    bindings.push(updateBinding);
    store.bindingsByField.set(key, bindings);
  }

  function registerActions(store, component, manifestComponent) {
    const actionsByMethod = buildActionsByMethod(manifestComponent);

    for (const [method, actions] of actionsByMethod) {
      store.actionsByMethod.set(
        componentMethodKey(component.name, method),
        actions
      );
    }
  }

  function registerEvent(store, component, event) {
    if (event.event !== "click") {
      reportDiagnostic(
        store.diagnostics,
        "EZR_UNRESOLVED_EVENT",
        "Unsupported event type in template manifest",
        event
      );
      return;
    }

    const method = normalizeHandlerReference(event.handler);
    const actions = store.actionsByMethod.get(
      componentMethodKey(component.name, method)
    );

    if (actions === undefined) {
      reportDiagnostic(
        store.diagnostics,
        "EZR_UNRESOLVED_ACTION",
        "Event handler did not resolve to a compiler action",
        event
      );
      return;
    }

    const eventsByNode = store.eventsByType.get(event.event) ?? new Map();

    if (eventsByNode.has(event.node)) {
      reportDiagnostic(
        store.diagnostics,
        "EZR_UNRESOLVED_EVENT",
        "Duplicate event registration for template node",
        event
      );
      return;
    }

    eventsByNode.set(event.node, {
      component,
      actions
    });
    store.eventsByType.set(event.event, eventsByNode);
  }

  function initializeComponentRuntime(
    store,
    manifestComponent,
    bindingAnchors,
    conditionalAnchors,
    listAnchors
  ) {
    const component = {
      name: manifestComponent.name,
      manifest: manifestComponent,
      state: {}
    };

    store.components.set(component.name, component);
    registerActions(store, component, manifestComponent);

    for (const node of manifestComponent.template?.nodes ?? []) {
      if (node.kind === "list") {
        const field = fieldNameFromThisMember(node.iterable);

        if (field === null) {
          continue;
        }

        if (component.state[field] === undefined) {
          component.state[field] = node.initial_value;
        }

        const start = listAnchors.starts.get(node.start);
        const end = listAnchors.ends.get(node.end);

        if (start === undefined || end === undefined) {
          continue;
        }

        let instances = initialListInstances(
          store,
          node,
          listItems(component.state[field])
        );
        registerBinding(store, component, field, (value) => {
          instances = reconcileKeyedList(
            store,
            node,
            start.marker,
            end.marker,
            instances,
            value
          );
        });
        continue;
      }

      if (node.kind === "conditional") {
        const field = fieldNameFromThisMember(node.condition);

        if (field === null) {
          continue;
        }

        if (component.state[field] === undefined) {
          component.state[field] = node.initial_value;
        }

        const start = conditionalAnchors.starts.get(node.start);
        const end = conditionalAnchors.ends.get(node.end);

        if (start === undefined || end === undefined) {
          continue;
        }

        registerBinding(store, component, field, (value) => {
          replaceConditionalBranch(
            store,
            start.marker,
            end.marker,
            value === true ? node.when_true_html : node.when_false_html
          );
        });
        continue;
      }

      if (node.kind !== "binding") {
        continue;
      }

      const field = fieldNameFromThisMember(node.expression);

      if (field === null) {
        continue;
      }

      if (component.state[field] === undefined) {
        component.state[field] = node.initial_value;
      }

      if (node.target === "attribute") {
        const element = store.elementsByNode.get(node.element);

        if (element === undefined) {
          continue;
        }

        updateAttributeBinding(element, node.attribute, component.state[field]);
        registerBinding(store, component, field, (value) => {
          updateAttributeBinding(element, node.attribute, value);
        });
        continue;
      }

      const anchor = bindingAnchors.get(node.id);

      if (anchor === undefined) {
        continue;
      }

      const textNode = anchor.marker.nextSibling;

      if (!(textNode instanceof Text)) {
        reportDiagnostic(
          store.diagnostics,
          "EZR_MISSING_BINDING_ANCHOR",
          "Binding anchor was not followed by a text node",
          node
        );
        continue;
      }

      registerBinding(store, component, field, (value) => {
        textNode.textContent = formatBindingValue(value);
      });
    }

    return component;
  }

  function actionDelta(store, action) {
    if (action.operation === "increment") {
      return 1;
    }

    if (action.operation === "decrement") {
      return -1;
    }

    if (action.operation === "add_assign" || action.operation === "subtract_assign") {
      const operand = Number(action.operand);

      if (Number.isNaN(operand)) {
        reportDiagnostic(
          store.diagnostics,
          "EZR_INVALID_STATE_OPERATION",
          "Numeric state operation had a non-numeric operand",
          action
        );
        return null;
      }

      return action.operation === "add_assign" ? operand : -operand;
    }

    return null;
  }

  function executeAction(store, component, action) {
    if (
      action.operation !== "increment" &&
      action.operation !== "decrement" &&
      action.operation !== "add_assign" &&
      action.operation !== "subtract_assign" &&
      action.operation !== "assign" &&
      action.operation !== "toggle"
    ) {
      reportDiagnostic(
        store.diagnostics,
        "EZR_INVALID_STATE_OPERATION",
        "Action used an unsupported state operation",
        action
      );
      return;
    }

    if (action.operation === "toggle") {
      const current = readField(store, component, action.field);

      if (typeof current !== "boolean") {
        reportDiagnostic(
          store.diagnostics,
          "EZR_INVALID_STATE_OPERATION",
          "Toggle action requires a boolean state field",
          action
        );
        return;
      }

      writeField(store, component, action.field, !current);
      return;
    }

    if (action.operation === "assign") {
      writeField(store, component, action.field, action.operand);
      return;
    }

    const current = Number(readField(store, component, action.field));

    if (Number.isNaN(current)) {
      reportDiagnostic(
        store.diagnostics,
        "EZR_INVALID_STATE_OPERATION",
        "Numeric state operation requires a numeric state field",
        action
      );
      return;
    }

    const delta = actionDelta(store, action);

    if (delta === null) {
      return;
    }

    writeField(store, component, action.field, current + delta);
  }

  function executeActions(store, component, actions) {
    for (const action of actions) {
      executeAction(store, component, action);
    }
  }

  function registerComponentEvents(store, component) {
    for (const event of component.manifest.template?.events ?? []) {
      registerEvent(store, component, event);
    }
  }

  function delegatedEventRecord(store, eventType, target) {
    const eventsByNode = store.eventsByType.get(eventType);

    if (eventsByNode === undefined) {
      return null;
    }

    let current = target instanceof Element ? target : target?.parentElement;

    while (current !== null && current !== undefined) {
      const nodeId = current.dataset?.ezNode;

      if (nodeId !== undefined) {
        const record = eventsByNode.get(nodeId);

        if (record !== undefined) {
          return record;
        }
      }

      current = current.parentElement;
    }

    return null;
  }

  function dispatchDelegatedEvent(store, event) {
    const record = delegatedEventRecord(store, event.type, event.target);

    if (record === null) {
      return;
    }

    executeActions(store, record.component, record.actions);
  }

  function installDelegatedEventListeners(store) {
    for (const eventType of store.eventsByType.keys()) {
      document.addEventListener(eventType, (event) => {
        dispatchDelegatedEvent(store, event);
      });
    }
  }

  function debugComponents(store) {
    return [...store.components.values()].map((component) => ({
      name: component.name,
      state: component.state
    }));
  }

  function runtimeState({
    manifest = null,
    missingAnchors = [],
    store = null,
    components = [],
    diagnostics
  }) {
    return {
      runtime_version: RUNTIME_VERSION,
      supported_schema_version: SUPPORTED_SCHEMA_VERSION,
      manifest,
      missingAnchors,
      diagnostics,
      store,
      components
    };
  }

  function initializeRuntime(manifest, diagnostics) {
    const bindingAnchors = collectBindingAnchors();
    const conditionalAnchors = collectConditionalAnchors();
    const listAnchors = collectListAnchors();
    const elementsByNode = collectElementAnchors();
    const store = createRuntimeStore(elementsByNode, diagnostics);
    const missingAnchors = collectMissingAnchors(
      manifest,
      bindingAnchors,
      conditionalAnchors,
      listAnchors,
      elementsByNode
    );

    for (const anchor of missingAnchors) {
      reportDiagnostic(
        diagnostics,
        anchor.code,
        anchor.kind === "element"
          ? "Manifest element anchor was not found in the rendered DOM"
          : anchor.kind === "conditional"
            ? "Manifest conditional anchor was not found in the rendered DOM"
            : anchor.kind === "list"
              ? "Manifest list anchor was not found in the rendered DOM"
            : "Manifest binding anchor was not found in the rendered DOM",
        anchor
      );
    }

    for (const manifestComponent of manifest.components ?? []) {
      const component = initializeComponentRuntime(
        store,
        manifestComponent,
        bindingAnchors,
        conditionalAnchors,
        listAnchors
      );
      registerComponentEvents(store, component);
    }

    installDelegatedEventListeners(store);

    return runtimeState({
      manifest,
      missingAnchors,
      diagnostics,
      store,
      components: debugComponents(store)
    });
  }

  function boot() {
    const diagnostics = [];

    try {
      const manifest = readManifest(diagnostics);
      validateManifestSchema(manifest, diagnostics);

      const state = initializeRuntime(manifest, diagnostics);
      const status = state.diagnostics.some((diagnostic) => diagnostic.fatal)
        || state.missingAnchors.length > 0
        ? "error"
        : "ready";

      document.documentElement.dataset.ezRuntime = status;
      window.__EDGEZERO__ = state;

      document.dispatchEvent(
        new CustomEvent("edgezero:ready", {
          detail: state
        })
      );
    } catch (error) {
      document.documentElement.dataset.ezRuntime = "error";
      window.__EDGEZERO__ = runtimeState({
        diagnostics
      });

      document.dispatchEvent(
        new CustomEvent("edgezero:ready", {
          detail: window.__EDGEZERO__
        })
      );

      if (!(error instanceof EdgeZeroBootError)) {
        reportDiagnostic(
          diagnostics,
          "EZR_RUNTIME_BOOT_FAILED",
          "Runtime boot failed",
          { message: error instanceof Error ? error.message : String(error) },
          true
        );
      }
    }
  }

  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", boot, {
      once: true
    });
  } else {
    boot();
  }
})();
"#;

#[must_use]
pub fn generate_runtime_stub() -> String {
    RUNTIME_STUB.to_string()
}

#[cfg(test)]
mod tests {
    use super::generate_runtime_stub;

    #[test]
    fn emits_runtime_manifest_bootstrap() {
        let runtime = generate_runtime_stub();

        assert!(runtime.contains("ez-template-manifest"));
        assert!(runtime.contains("RUNTIME_VERSION = \"0.0.0\""));
        assert!(runtime.contains("SUPPORTED_SCHEMA_VERSION = 1"));
        assert!(runtime.contains("EZR_MISSING_MANIFEST"));
        assert!(runtime.contains("EZR_INVALID_MANIFEST_JSON"));
        assert!(runtime.contains("EZR_UNSUPPORTED_SCHEMA"));
        assert!(runtime.contains("data-ez-node"));
        assert!(runtime.contains("ez-binding:"));
        assert!(runtime.contains("reportDiagnostic"));
        assert!(runtime.contains("validateManifestSchema"));
        assert!(runtime.contains("normalizeHandlerReference"));
        assert!(runtime.contains("createRuntimeStore"));
        assert!(runtime.contains("readField"));
        assert!(runtime.contains("writeField"));
        assert!(runtime.contains("notifyField"));
        assert!(runtime.contains("actionDelta"));
        assert!(runtime.contains("isBooleanAttribute"));
        assert!(runtime.contains("isPropertyAttribute"));
        assert!(runtime.contains("updateAttributeBinding"));
        assert!(runtime.contains("const actions = actionsByMethod.get(action.method) ?? []"));
        assert!(runtime.contains("actions.push(action)"));
        assert!(runtime.contains("executeActions"));
        assert!(runtime.contains("formatBindingValue"));
        assert!(runtime.contains("value === null ? \"\" : String(value)"));
        assert!(runtime.contains("component.state[field] = node.initial_value"));
        assert!(!runtime.contains("component.state[field] = Number(node.initial_value)"));
        assert!(runtime.contains("installDelegatedEventListeners"));
        assert!(runtime.contains("document.addEventListener(eventType"));
        assert!(!runtime.contains("element.addEventListener(\"click\""));
        assert!(runtime.contains("action.operation !== \"toggle\""));
        assert!(runtime.contains("action.operation === \"assign\""));
        assert!(runtime.contains("action.operation === \"toggle\""));
        assert!(runtime.contains("EZR_MISSING_ELEMENT_ANCHOR"));
        assert!(runtime.contains("EZR_MISSING_BINDING_ANCHOR"));
        assert!(runtime.contains("EZR_UNRESOLVED_EVENT"));
        assert!(runtime.contains("EZR_UNRESOLVED_ACTION"));
        assert!(runtime.contains("EZR_INVALID_STATE_OPERATION"));
        assert!(runtime.contains("current + delta"));
        assert!(runtime.contains("dataset.ezRuntime"));
        assert!(runtime.contains("edgezero:ready"));
        assert!(runtime.contains("runtime_version"));
        assert!(runtime.contains("diagnostics"));
        assert!(runtime.contains("window.__EDGEZERO__"));
    }
}
