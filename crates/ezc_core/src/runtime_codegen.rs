const RUNTIME_STUB: &str = r#"(() => {
  "use strict";

  const MANIFEST_ELEMENT_ID = "ez-template-manifest";

  function readManifest() {
    const element = document.getElementById(MANIFEST_ELEMENT_ID);

    if (!(element instanceof HTMLScriptElement)) {
      throw new Error(
        `Missing template manifest script #${MANIFEST_ELEMENT_ID}`
      );
    }

    return JSON.parse(element.textContent ?? "");
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

  function collectElementAnchors() {
    const elementsByNode = new Map();

    for (const element of document.querySelectorAll("[data-ez-node]")) {
      elementsByNode.set(element.dataset.ezNode, element);
    }

    return elementsByNode;
  }

  function collectMissingAnchors(manifest, bindingAnchors, elementsByNode) {
    const missing = [];

    for (const component of manifest.components ?? []) {
      for (const node of component.template?.nodes ?? []) {
        if (node.kind === "element") {
          if (!elementsByNode.has(node.id)) {
            missing.push({
              id: node.id,
              kind: node.kind
            });
          }
        }

        if (
          node.kind === "binding" &&
          !bindingAnchors.has(node.id)
        ) {
          missing.push({
            id: node.id,
            kind: node.kind
          });
        }
      }
    }

    return missing;
  }

  function buildActionsByMethod(component) {
    const actionsByMethod = new Map();

    for (const action of component.actions ?? []) {
      actionsByMethod.set(action.method, action);
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

  function createRuntimeStore(elementsByNode) {
    return {
      components: new Map(),
      bindingsByField: new Map(),
      actionsByMethod: new Map(),
      eventsByType: new Map(),
      elementsByNode
    };
  }

  function readField(component, field) {
    if (!(field in component.state)) {
      console.error(
        "[EdgeZero] EZR_MISSING_FIELD",
        { component: component.name, field }
      );
      return undefined;
    }

    return component.state[field];
  }

  function writeField(store, component, field, value) {
    if (!(field in component.state)) {
      console.error(
        "[EdgeZero] EZR_MISSING_FIELD",
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
      console.error(
        "[EdgeZero] EZR_MISSING_BINDING",
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

    for (const [method, action] of actionsByMethod) {
      store.actionsByMethod.set(
        componentMethodKey(component.name, method),
        action
      );
    }
  }

  function registerEvent(store, component, event) {
    if (event.event !== "click") {
      console.error(
        "[EdgeZero] EZR_UNSUPPORTED_EVENT",
        event
      );
      return;
    }

    const method = normalizeHandlerReference(event.handler);
    const action = store.actionsByMethod.get(
      componentMethodKey(component.name, method)
    );

    if (action === undefined) {
      console.error(
        "[EdgeZero] EZR_MISSING_ACTION",
        event
      );
      return;
    }

    const eventsByNode = store.eventsByType.get(event.event) ?? new Map();

    if (eventsByNode.has(event.node)) {
      console.error(
        "[EdgeZero] EZR_DUPLICATE_EVENT",
        event
      );
      return;
    }

    eventsByNode.set(event.node, {
      component,
      action
    });
    store.eventsByType.set(event.event, eventsByNode);
  }

  function initializeComponentRuntime(store, manifestComponent, bindingAnchors) {
    const component = {
      name: manifestComponent.name,
      manifest: manifestComponent,
      state: {}
    };

    store.components.set(component.name, component);
    registerActions(store, component, manifestComponent);

    for (const node of manifestComponent.template?.nodes ?? []) {
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

      const anchor = bindingAnchors.get(node.id);

      if (anchor === undefined) {
        console.error(
          "[EdgeZero] EZR_MISSING_BINDING_ANCHOR",
          node
        );
        continue;
      }

      const textNode = anchor.marker.nextSibling;

      if (!(textNode instanceof Text)) {
        console.error(
          "[EdgeZero] EZR_MISSING_BINDING_TEXT",
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

  function executeAction(store, component, action) {
    if (action.operation !== "increment") {
      console.error(
        "[EdgeZero] EZR_UNSUPPORTED_ACTION",
        action
      );
      return;
    }

    const current = Number(readField(component, action.field));

    if (Number.isNaN(current)) {
      console.error(
        "[EdgeZero] EZR_NON_NUMERIC_FIELD",
        action
      );
      return;
    }

    writeField(store, component, action.field, current + 1);
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

    executeAction(store, record.component, record.action);
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

  function initializeRuntime(manifest) {
    const bindingAnchors = collectBindingAnchors();
    const elementsByNode = collectElementAnchors();
    const store = createRuntimeStore(elementsByNode);
    const missingAnchors = collectMissingAnchors(
      manifest,
      bindingAnchors,
      elementsByNode
    );

    for (const manifestComponent of manifest.components ?? []) {
      const component = initializeComponentRuntime(
        store,
        manifestComponent,
        bindingAnchors
      );
      registerComponentEvents(store, component);
    }

    installDelegatedEventListeners(store);

    return {
      manifest,
      missingAnchors,
      store,
      components: debugComponents(store)
    };
  }

  function boot() {
    try {
      const manifest = readManifest();
      const runtimeState = initializeRuntime(manifest);
      const status = runtimeState.missingAnchors.length === 0 ? "ready" : "error";

      document.documentElement.dataset.ezRuntime = status;
      window.__EDGEZERO__ = runtimeState;

      document.dispatchEvent(
        new CustomEvent("edgezero:ready", {
          detail: runtimeState
        })
      );

      if (runtimeState.missingAnchors.length > 0) {
        console.error(
          "[EdgeZero] Missing template anchors",
          runtimeState.missingAnchors
        );
      }
    } catch (error) {
      document.documentElement.dataset.ezRuntime = "error";

      console.error(
        "[EdgeZero] Runtime boot failed",
        error
      );
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
        assert!(runtime.contains("data-ez-node"));
        assert!(runtime.contains("ez-binding:"));
        assert!(runtime.contains("normalizeHandlerReference"));
        assert!(runtime.contains("createRuntimeStore"));
        assert!(runtime.contains("readField"));
        assert!(runtime.contains("writeField"));
        assert!(runtime.contains("notifyField"));
        assert!(runtime.contains("formatBindingValue"));
        assert!(runtime.contains("value === null ? \"\" : String(value)"));
        assert!(runtime.contains("component.state[field] = node.initial_value"));
        assert!(!runtime.contains("component.state[field] = Number(node.initial_value)"));
        assert!(runtime.contains("installDelegatedEventListeners"));
        assert!(runtime.contains("document.addEventListener(eventType"));
        assert!(!runtime.contains("element.addEventListener(\"click\""));
        assert!(runtime.contains("action.operation !== \"increment\""));
        assert!(runtime.contains("dataset.ezRuntime"));
        assert!(runtime.contains("edgezero:ready"));
        assert!(runtime.contains("window.__EDGEZERO__"));
    }
}
