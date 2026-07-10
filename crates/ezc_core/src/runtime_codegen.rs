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

  function initializeComponentRuntime(component, bindingAnchors, elementsByNode) {
    const state = {};
    const bindingsByField = new Map();
    const actionsByMethod = buildActionsByMethod(component);

    for (const node of component.template?.nodes ?? []) {
      if (node.kind !== "binding") {
        continue;
      }

      const field = fieldNameFromThisMember(node.expression);

      if (field === null) {
        continue;
      }

      if (node.initial_value !== null && state[field] === undefined) {
        state[field] = Number(node.initial_value);
      }

      const anchor = bindingAnchors.get(node.id);

      if (anchor === undefined) {
        console.error(
          "[EdgeZero] Missing binding anchor",
          node
        );
        continue;
      }

      const textNode = anchor.marker.nextSibling;

      if (!(textNode instanceof Text)) {
        console.error(
          "[EdgeZero] Missing binding text node",
          node
        );
        continue;
      }

      const bindings = bindingsByField.get(field) ?? [];
      bindings.push({
        node,
        textNode
      });
      bindingsByField.set(field, bindings);
    }

    return {
      component,
      state,
      actionsByMethod,
      bindingsByField,
      elementsByNode
    };
  }

  function updateFieldBindings(runtimeComponent, field) {
    const bindings = runtimeComponent.bindingsByField.get(field);

    if (bindings === undefined) {
      console.error(
        "[EdgeZero] Missing binding for field",
        field
      );
      return;
    }

    for (const binding of bindings) {
      binding.textNode.textContent = String(runtimeComponent.state[field]);
    }
  }

  function executeAction(runtimeComponent, action) {
    if (action.operation !== "increment") {
      console.error(
        "[EdgeZero] Unsupported action operation",
        action
      );
      return;
    }

    if (!(action.field in runtimeComponent.state)) {
      console.error(
        "[EdgeZero] Missing state field",
        action
      );
      return;
    }

    const current = Number(runtimeComponent.state[action.field]);

    if (Number.isNaN(current)) {
      console.error(
        "[EdgeZero] State field is not numeric",
        action
      );
      return;
    }

    runtimeComponent.state[action.field] = current + 1;
    updateFieldBindings(runtimeComponent, action.field);
  }

  function attachEventListeners(runtimeComponent) {
    for (const event of runtimeComponent.component.template?.events ?? []) {
      const element = runtimeComponent.elementsByNode.get(event.node);

      if (element === undefined) {
        console.error(
          "[EdgeZero] Missing event anchor",
          event
        );
        continue;
      }

      const method = normalizeHandlerReference(event.handler);
      const action = runtimeComponent.actionsByMethod.get(method);

      if (action === undefined) {
        console.error(
          "[EdgeZero] Missing action for handler",
          event
        );
        continue;
      }

      element.addEventListener("click", () => {
        executeAction(runtimeComponent, action);
      });
    }
  }

  function initializeRuntime(manifest) {
    const bindingAnchors = collectBindingAnchors();
    const elementsByNode = collectElementAnchors();
    const missingAnchors = collectMissingAnchors(
      manifest,
      bindingAnchors,
      elementsByNode
    );
    const components = [];

    for (const component of manifest.components ?? []) {
      const runtimeComponent = initializeComponentRuntime(
        component,
        bindingAnchors,
        elementsByNode
      );
      attachEventListeners(runtimeComponent);
      components.push({
        name: component.name,
        state: runtimeComponent.state
      });
    }

    return {
      manifest,
      missingAnchors,
      components
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
        assert!(runtime.contains("addEventListener(\"click\""));
        assert!(runtime.contains("action.operation !== \"increment\""));
        assert!(runtime.contains("dataset.ezRuntime"));
        assert!(runtime.contains("edgezero:ready"));
        assert!(runtime.contains("window.__EDGEZERO__"));
    }
}
