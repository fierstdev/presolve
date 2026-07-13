const RUNTIME_STUB: &str = r#"(() => {
  "use strict";

  const MANIFEST_ELEMENT_ID = "ez-template-manifest";
  const COMPUTED_ARTIFACT_ELEMENT_ID = "ez-computed-runtime";
  const RUNTIME_VERSION = "0.0.0";
  const SUPPORTED_SCHEMA_VERSION = 1;
  const SUPPORTED_COMPUTED_ARTIFACT_SCHEMA_VERSION = 2;

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

  function readComputedArtifact(diagnostics) {
    const element = document.getElementById(COMPUTED_ARTIFACT_ELEMENT_ID);

    if (element === null) {
      return null;
    }

    if (!(element instanceof HTMLScriptElement)) {
      reportDiagnostic(
        diagnostics,
        "EZR_INVALID_COMPUTED_ARTIFACT",
        "Computed runtime metadata was not stored in a script element",
        { artifactElementId: COMPUTED_ARTIFACT_ELEMENT_ID },
        true
      );
      throw new EdgeZeroBootError("EZR_INVALID_COMPUTED_ARTIFACT");
    }

    try {
      return JSON.parse(element.textContent ?? "");
    } catch (error) {
      reportDiagnostic(
        diagnostics,
        "EZR_INVALID_COMPUTED_ARTIFACT",
        "Computed runtime metadata JSON could not be parsed",
        { message: error instanceof Error ? error.message : String(error) },
        true
      );
      throw new EdgeZeroBootError("EZR_INVALID_COMPUTED_ARTIFACT");
    }
  }

  function validateComputedArtifactSchema(artifact, diagnostics) {
    if (artifact === null) {
      return;
    }

    if (artifact.schema_version !== SUPPORTED_COMPUTED_ARTIFACT_SCHEMA_VERSION) {
      reportDiagnostic(
        diagnostics,
        "EZR_UNSUPPORTED_COMPUTED_ARTIFACT_SCHEMA",
        `Unsupported computed runtime metadata schema version ${String(artifact.schema_version)}`,
        {
          schema_version: artifact.schema_version,
          supported_schema_version: SUPPORTED_COMPUTED_ARTIFACT_SCHEMA_VERSION
        },
        true
      );
      throw new EdgeZeroBootError("EZR_UNSUPPORTED_COMPUTED_ARTIFACT_SCHEMA");
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

  function listItemMemberPath(node, expression) {
    const prefix = `${String(node.item_variable ?? "")}.`;
    const value = String(expression ?? "");

    if (!value.startsWith(prefix)) {
      return null;
    }

    const path = value.slice(prefix.length).split(".");
    return path.length === 0 || path.some((member) => member === "") ? null : path;
  }

  function listItemMemberValue(node, item, expression) {
    const path = listItemMemberPath(node, expression);

    if (path === null) {
      return undefined;
    }

    let value = item;

    for (const member of path) {
      if (
        value === null ||
        typeof value !== "object" ||
        Array.isArray(value) ||
        !Object.prototype.hasOwnProperty.call(value, member)
      ) {
        return undefined;
      }

      value = value[member];
    }

    return value;
  }

  function listItemBindingValue(node, item, index, expression) {
    if (expression === node.item_variable) {
      return item;
    }

    if (expression === node.index_variable) {
      return index;
    }

    if (listItemMemberPath(node, expression) !== null) {
      return listItemMemberValue(node, item, expression);
    }

    return undefined;
  }

  function isListKeyPrimitive(value) {
    return value === null || (typeof value !== "object" && typeof value !== "undefined");
  }

  function listItemKey(node, item, index) {
    if (node.key_expression === node.item_variable) {
      return isListKeyPrimitive(item) ? normalizeListKey(item) : String(index);
    }

    if (listItemMemberPath(node, node.key_expression) !== null) {
      const value = listItemMemberValue(node, item, node.key_expression);
      return isListKeyPrimitive(value) ? normalizeListKey(value) : String(index);
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

  function populateListItemMemberBindings(node, item, fragment) {
    const walker = document.createTreeWalker(fragment, NodeFilter.SHOW_COMMENT);
    const markers = [];

    while (walker.nextNode()) {
      markers.push(walker.currentNode);
    }

    const memberPrefix = `:${String(node.item_variable ?? "")}.`;

    for (const marker of markers) {
      const comment = String(marker.nodeValue ?? "");
      const expressionStart = comment.lastIndexOf(memberPrefix);

      if (expressionStart < 0) {
        continue;
      }

      const expression = comment.slice(expressionStart + 1);
      const value = listItemMemberValue(node, item, expression);
      marker.after(document.createTextNode(value === undefined ? "" : formatBindingValue(value)));
    }
  }

  function listItemBindingExpression(node, comment) {
    const value = String(comment ?? "");
    const memberPrefix = `:${String(node.item_variable ?? "")}.`;
    const memberStart = value.lastIndexOf(memberPrefix);

    if (memberStart >= 0) {
      return value.slice(memberStart + 1);
    }

    const itemSuffix = `:${String(node.item_variable ?? "")}`;
    if (value.endsWith(itemSuffix)) {
      return String(node.item_variable ?? "");
    }

    const indexSuffix = `:${String(node.index_variable ?? "")}`;
    if (node.index_variable !== null && node.index_variable !== undefined && value.endsWith(indexSuffix)) {
      return String(node.index_variable);
    }

    return null;
  }

  function updateListItemTextBindings(node, instance) {
    const walker = document.createTreeWalker(instance.element, NodeFilter.SHOW_COMMENT);

    while (walker.nextNode()) {
      const marker = walker.currentNode;
      const expression = listItemBindingExpression(node, marker.nodeValue);

      if (expression === null) {
        continue;
      }

      const textNode = marker.nextSibling;
      const value = listItemBindingValue(node, instance.item, instance.index, expression);
      const text = value === undefined ? "" : formatBindingValue(value);

      if (textNode instanceof Text) {
        textNode.textContent = text;
      } else if (
        textNode instanceof Comment &&
        String(textNode.nodeValue ?? "").startsWith("ez-list-binding-end:")
      ) {
        textNode.before(document.createTextNode(text));
      }
    }
  }

  function listItemElements(root) {
    return [root, ...root.querySelectorAll("[data-ez-list-bindings]")];
  }

  function updateListItemAttributes(node, instance) {
    for (const element of listItemElements(instance.element)) {
      const bindings = String(element.dataset.ezListBindings ?? "");

      for (const binding of bindings.split(";")) {
        const separator = binding.indexOf("=");

        if (separator < 1) {
          continue;
        }

        const attribute = binding.slice(0, separator);
        const expression = binding.slice(separator + 1);
        const value = listItemBindingValue(node, instance.item, instance.index, expression);
        updateAttributeBinding(element, attribute, value);
      }
    }
  }

  function listItemEventElements(root) {
    return [root, ...root.querySelectorAll("[data-ez-on-click]")];
  }

  function registerListItemEvents(store, component, instance) {
    for (const element of listItemEventElements(instance.element)) {
      const node = element.dataset.ezNode;
      const handler = element.dataset.ezOnClick;

      if (node !== undefined && handler !== undefined) {
        registerEvent(store, component, { node, event: "click", handler });
      }
    }
  }

  function unregisterListItemEvents(store, instance) {
    const eventsByNode = store.eventsByType.get("click");

    if (eventsByNode === undefined) {
      return;
    }

    for (const element of listItemEventElements(instance.element)) {
      const node = element.dataset.ezNode;

      if (node !== undefined) {
        eventsByNode.delete(node);
      }
    }
  }

  function renderListItemElement(node, item, index, key) {
    const template = document.createElement("template");
    template.innerHTML = renderListItemHtml(node, item, index, key);
    populateListItemMemberBindings(node, item, template.content);
    return template.content.firstElementChild;
  }

  function initialListInstances(store, node, items) {
    const instances = new Map();

    for (const [index, item] of items.entries()) {
      const key = listItemKey(node, item, index);
      const element = store.elementsByNode.get(`${node.item_root}:${key}`);

      if (element !== undefined) {
        instances.set(key, { element, item, index, key });
      }
    }

    return instances;
  }

  function reconcileKeyedList(store, component, node, startMarker, endMarker, instances, value) {
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
        const element = renderListItemElement(node, item, index, key);

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
        instance = { element, item, index, key };
        registerListItemEvents(store, component, instance);
      }

      instance.item = item;
      instance.index = index;
      updateListItemTextBindings(node, instance);
      updateListItemAttributes(node, instance);
      nextInstances.set(key, instance);
      ordered.push(instance);
    }

    for (const [key, instance] of instances) {
      if (!nextInstances.has(key)) {
        unregisterListItemEvents(store, instance);
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

  function createRuntimeStore(elementsByNode, diagnostics, computedArtifact) {
    const computedEvaluations = new Map();
    const storageValues = new Map();
    const computedDirty = new Map();

    for (const state of computedArtifact?.state ?? []) {
      storageValues.set(state.storage, state.initial_value);
    }

    for (const evaluation of computedArtifact?.evaluations ?? []) {
      computedEvaluations.set(evaluation.computed, evaluation);
      computedDirty.set(evaluation.computed, evaluation.dirty_flag?.initial_value === true);
    }

    return {
      components: new Map(),
      bindingsByField: new Map(),
      actionsByMethod: new Map(),
      eventsByType: new Map(),
      elementsByNode,
      diagnostics,
      computedArtifact,
      computedEvaluations,
      computedDirty,
      computedValues: new Map(),
      computedCaches: new Map(),
      storageValues
    };
  }

  function computedOperandValue(store, values, operand) {
    if (operand?.kind === "value") {
      return values.get(operand.value);
    }

    if (operand?.kind === "constant") {
      return operand.value;
    }

    if (operand?.kind === "storage") {
      return store.storageValues.get(operand.storage);
    }

    return undefined;
  }

  function computedBinary(operation, left, right) {
    switch (operation) {
      case "add": return left + right;
      case "subtract": return left - right;
      case "multiply": return left * right;
      case "divide": return left / right;
      case "remainder": return left % right;
      case "equal": return left === right;
      case "not-equal": return left !== right;
      case "less-than": return left < right;
      case "less-than-or-equal": return left <= right;
      case "greater-than": return left > right;
      case "greater-than-or-equal": return left >= right;
      case "and": return left && right;
      case "or": return left || right;
      case "nullish-coalesce": return left ?? right;
      default: return undefined;
    }
  }

  function computedUnary(operation, value) {
    switch (operation) {
      case "not": return !value;
      case "identity": return +value;
      case "negate": return -value;
      default: return undefined;
    }
  }

  function executeComputedProgram(store, evaluation) {
    const values = new Map();

    for (const instruction of evaluation.program?.instructions ?? []) {
      if (instruction.kind === "constant") {
        values.set(instruction.result, instruction.value);
        continue;
      }

      if (instruction.kind === "load-state") {
        values.set(instruction.result, store.storageValues.get(instruction.storage));
        continue;
      }

      if (instruction.kind === "load-computed") {
        if (store.computedDirty.get(instruction.computed) === true) {
          reportDiagnostic(
            store.diagnostics,
            "EZR_UNPLANNED_COMPUTED_DEPENDENCY",
            "Computed evaluation depended on a value not yet evaluated by the compiler plan",
            { computed: evaluation.computed, dependency: instruction.computed }
          );
          values.set(instruction.result, undefined);
        } else {
          values.set(instruction.result, store.computedValues.get(instruction.computed));
        }
        continue;
      }

      if (instruction.kind === "get-member") {
        const object = computedOperandValue(store, values, instruction.object);
        const value = object !== null && typeof object === "object"
          && Object.prototype.hasOwnProperty.call(object, instruction.property)
          ? object[instruction.property]
          : undefined;
        values.set(instruction.result, value);
        continue;
      }

      if (instruction.kind === "binary") {
        values.set(
          instruction.result,
          computedBinary(
            instruction.operation,
            computedOperandValue(store, values, instruction.left),
            computedOperandValue(store, values, instruction.right)
          )
        );
        continue;
      }

      if (instruction.kind === "unary") {
        values.set(
          instruction.result,
          computedUnary(
            instruction.operation,
            computedOperandValue(store, values, instruction.operand)
          )
        );
      }
    }

    return values.get(evaluation.program?.result);
  }

  function executeComputedPlan(store) {
    if (store.computedArtifact === null) {
      return;
    }

    for (const computed of store.computedArtifact.evaluation_order ?? []) {
      const evaluation = store.computedEvaluations.get(computed);

      if (evaluation === undefined) {
        reportDiagnostic(
          store.diagnostics,
          "EZR_UNPLANNED_COMPUTED_DEPENDENCY",
          "Compiler plan referenced a missing computed evaluation",
          { computed }
        );
        continue;
      }

      if (store.computedDirty.get(computed) !== true) {
        continue;
      }

      const value = executeComputedProgram(store, evaluation);
      store.computedValues.set(computed, value);
      store.computedCaches.set(evaluation.cache_slot, value);
      store.computedDirty.set(computed, false);
    }
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
        for (const instance of instances.values()) {
          registerListItemEvents(store, component, instance);
        }
        registerBinding(store, component, field, (value) => {
          instances = reconcileKeyedList(
            store,
            component,
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

  function debugComputed(store) {
    return [...store.computedEvaluations.values()].map((evaluation) => ({
      computed: evaluation.computed,
      cache_slot: evaluation.cache_slot,
      dirty: store.computedDirty.get(evaluation.computed) === true,
      value: store.computedCaches.get(evaluation.cache_slot)
    }));
  }

  function runtimeState({
    manifest = null,
    missingAnchors = [],
    store = null,
    components = [],
    computed = [],
    diagnostics
  }) {
    return {
      runtime_version: RUNTIME_VERSION,
      supported_schema_version: SUPPORTED_SCHEMA_VERSION,
      manifest,
      missingAnchors,
      diagnostics,
      store,
      components,
      computed
    };
  }

  function initializeRuntime(manifest, computedArtifact, diagnostics) {
    const bindingAnchors = collectBindingAnchors();
    const conditionalAnchors = collectConditionalAnchors();
    const listAnchors = collectListAnchors();
    const elementsByNode = collectElementAnchors();
    const store = createRuntimeStore(elementsByNode, diagnostics, computedArtifact);
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

    executeComputedPlan(store);

    installDelegatedEventListeners(store);

    return runtimeState({
      manifest,
      missingAnchors,
      diagnostics,
      store,
      components: debugComponents(store),
      computed: debugComputed(store)
    });
  }

  function boot() {
    const diagnostics = [];

    try {
      const manifest = readManifest(diagnostics);
      validateManifestSchema(manifest, diagnostics);
      const computedArtifact = readComputedArtifact(diagnostics);
      validateComputedArtifactSchema(computedArtifact, diagnostics);

      const state = initializeRuntime(manifest, computedArtifact, diagnostics);
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
        assert!(runtime.contains("listItemMemberPath"));
        assert!(runtime.contains("populateListItemMemberBindings"));
        assert!(runtime.contains("updateListItemTextBindings"));
        assert!(runtime.contains("updateListItemAttributes"));
        assert!(runtime.contains("registerListItemEvents"));
        assert!(runtime.contains("unregisterListItemEvents"));
        assert!(runtime.contains("ez-list-binding-end:"));
        assert!(runtime.contains("renderListItemElement"));
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
