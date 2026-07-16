const RUNTIME_STUB: &str = r#"(() => {
  "use strict";

  const MANIFEST_ELEMENT_ID = "ez-template-manifest";
  const COMPUTED_ARTIFACT_ELEMENT_ID = "ez-computed-runtime";
  const EFFECT_ARTIFACT_ELEMENT_ID = "ez-effect-runtime";
  const CONTEXT_ARTIFACT_ELEMENT_ID = "ez-context-runtime";
  const COMPONENT_ARTIFACT_ELEMENT_ID = "ez-component-runtime";
  const FORMS_ARTIFACT_ELEMENT_ID = "ez-forms-runtime";
  const RUNTIME_VERSION = "0.0.0";
  const SUPPORTED_SCHEMA_VERSION = 4;
  const ACTION_MANIFEST_SCHEMA_VERSION = 2;
  const FORMS_MANIFEST_SCHEMA_VERSION = 3;
  const LEGACY_MANIFEST_SCHEMA_VERSION = 1;
  const SUPPORTED_COMPUTED_ARTIFACT_SCHEMA_VERSION = 3;
  const SUPPORTED_EFFECT_ARTIFACT_SCHEMA_VERSION = 1;
  const SUPPORTED_CONTEXT_ARTIFACT_SCHEMA_VERSION = 2;
  const SUPPORTED_COMPONENT_ARTIFACT_SCHEMA_VERSION = __EZ_COMPONENT_SCHEMA_VERSION__;
  const LEGACY_COMPONENT_ARTIFACT_SCHEMA_VERSION = 2;
  const SUPPORTED_FORMS_ARTIFACT_SCHEMA_VERSION = 1;

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

  function effectArtifactHasActionPlans(effectArtifact) {
    return (effectArtifact?.effects ?? []).some((effect) =>
      Array.isArray(effect.action_batch_triggers) && effect.action_batch_triggers.length > 0
    );
  }

  function validateManifestActionBindings(manifest, diagnostics) {
    for (const component of manifest.components ?? []) {
      const actionsByMethod = new Map();

      for (const action of component.actions ?? []) {
        if (typeof action.method_id !== "string" || typeof action.action_batch_id !== "string") {
          reportDiagnostic(
            diagnostics,
            "EZR_INVALID_ACTION_BINDING",
            "Schema-v2 template action was missing compiler action identities",
            { component: component.name, action },
            true
          );
          throw new EdgeZeroBootError("EZR_INVALID_ACTION_BINDING");
        }
        actionsByMethod.set(action.method_id, action.action_batch_id);
      }

      for (const event of component.template?.events ?? []) {
        if (event.kind !== "action") {
          reportDiagnostic(
            diagnostics,
            "EZR_INVALID_ACTION_BINDING",
            "Schema-v2 template event was not an explicit action binding",
            { component: component.name, event },
            true
          );
          throw new EdgeZeroBootError("EZR_INVALID_ACTION_BINDING");
        }
        if (typeof event.method_id !== "string" || typeof event.action_batch_id !== "string") {
          reportDiagnostic(
            diagnostics,
            "EZR_INVALID_ACTION_BINDING",
            "Schema-v2 template action binding was missing an action batch identity",
            { component: component.name, event },
            true
          );
          throw new EdgeZeroBootError("EZR_INVALID_ACTION_BINDING");
        }
        if (actionsByMethod.get(event.method_id) !== event.action_batch_id) {
          reportDiagnostic(
            diagnostics,
            "EZR_INVALID_ACTION_BINDING",
            "Template action binding did not match its compiler action implementation",
            { component: component.name, event },
            true
          );
          throw new EdgeZeroBootError("EZR_INVALID_ACTION_BINDING");
        }
      }
    }
  }

  function validateManifestSchema(manifest, effectArtifact, componentArtifact, diagnostics) {
    if (
      manifest?.schema_version !== SUPPORTED_SCHEMA_VERSION &&
      manifest?.schema_version !== FORMS_MANIFEST_SCHEMA_VERSION &&
      manifest?.schema_version !== ACTION_MANIFEST_SCHEMA_VERSION &&
      manifest?.schema_version !== LEGACY_MANIFEST_SCHEMA_VERSION
    ) {
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

    const isOrdinaryInstancePair = manifest.schema_version === SUPPORTED_SCHEMA_VERSION
      && componentArtifact?.schema_version === SUPPORTED_COMPONENT_ARTIFACT_SCHEMA_VERSION;
    const isLegacyColdPair = manifest.schema_version === FORMS_MANIFEST_SCHEMA_VERSION
      && componentArtifact?.schema_version === LEGACY_COMPONENT_ARTIFACT_SCHEMA_VERSION;
    if (!isOrdinaryInstancePair && !isLegacyColdPair) {
      reportDiagnostic(
        diagnostics,
        "EZR_UNSUPPORTED_SCHEMA",
        "Template manifest and component artifact are not an exact runtime contract pair",
        { manifest_schema_version: manifest.schema_version, component_schema_version: componentArtifact?.schema_version },
        true
      );
      throw new EdgeZeroBootError("EZR_UNSUPPORTED_SCHEMA");
    }

    if (
      manifest.schema_version === LEGACY_MANIFEST_SCHEMA_VERSION &&
      effectArtifactHasActionPlans(effectArtifact)
    ) {
      reportDiagnostic(
        diagnostics,
        "EZR_LEGACY_MANIFEST_EFFECT_ACTIONS",
        "A legacy template manifest cannot activate compiler-generated effect action batches",
        { schema_version: manifest.schema_version },
        true
      );
      throw new EdgeZeroBootError("EZR_LEGACY_MANIFEST_EFFECT_ACTIONS");
    }

    if (manifest.schema_version >= ACTION_MANIFEST_SCHEMA_VERSION) {
      validateManifestActionBindings(manifest, diagnostics);
    }
  }

  function readFormsArtifact(diagnostics) {
    const element = document.getElementById(FORMS_ARTIFACT_ELEMENT_ID);
    if (element === null) return null;
    if (!(element instanceof HTMLScriptElement)) {
      reportDiagnostic(diagnostics, "EZR_INVALID_FORMS_ARTIFACT", "Forms runtime metadata was not stored in a script element", { artifactElementId: FORMS_ARTIFACT_ELEMENT_ID }, true);
      throw new EdgeZeroBootError("EZR_INVALID_FORMS_ARTIFACT");
    }
    try { return JSON.parse(element.textContent ?? ""); } catch (error) {
      reportDiagnostic(diagnostics, "EZR_INVALID_FORMS_ARTIFACT", "Forms runtime metadata JSON could not be parsed", { message: error instanceof Error ? error.message : String(error) }, true);
      throw new EdgeZeroBootError("EZR_INVALID_FORMS_ARTIFACT");
    }
  }

  function validateFormsArtifact(formsArtifact, manifest, diagnostics) {
    if (formsArtifact === null) {
      if (manifest.schema_version === 3) {
        reportDiagnostic(diagnostics, "EZR_MISSING_FORMS_ARTIFACT", "A schema-v3 template manifest requires Forms runtime metadata", {}, true);
        throw new EdgeZeroBootError("EZR_MISSING_FORMS_ARTIFACT");
      }
      return;
    }
    if (formsArtifact.schema_version !== SUPPORTED_FORMS_ARTIFACT_SCHEMA_VERSION || !Array.isArray(formsArtifact.forms) || !Array.isArray(formsArtifact.instances) || !Array.isArray(formsArtifact.hosts)) {
      reportDiagnostic(diagnostics, "EZR_UNSUPPORTED_FORMS_ARTIFACT_SCHEMA", "Forms runtime metadata did not match the compiler artifact contract", { schema_version: formsArtifact.schema_version }, true);
      throw new EdgeZeroBootError("EZR_UNSUPPORTED_FORMS_ARTIFACT_SCHEMA");
    }
    const hasForms = formsArtifact.forms.length > 0 || formsArtifact.instances.length > 0;
    if (hasForms && manifest.schema_version !== 3) {
      reportDiagnostic(diagnostics, "EZR_FORMS_MANIFEST_MISMATCH", "Forms runtime metadata requires a schema-v3 template manifest", { schema_version: manifest.schema_version }, true);
      throw new EdgeZeroBootError("EZR_FORMS_MANIFEST_MISMATCH");
    }
    const instances = new Set(formsArtifact.instances.map((instance) => instance.id));
    for (const binding of manifest.form_bindings ?? []) {
      if (!instances.has(binding.form_instance_id)) {
        reportDiagnostic(diagnostics, "EZR_FORMS_MANIFEST_MISMATCH", "Forms manifest bridge referenced an unknown Form instance", { binding }, true);
        throw new EdgeZeroBootError("EZR_FORMS_MANIFEST_MISMATCH");
      }
    }
    const artifactHosts = new Map(formsArtifact.hosts.map((host) => [`${host.host_anchor}|${host.form_instance}`, host]));
    for (const host of manifest.form_hosts ?? []) {
      const artifact = artifactHosts.get(`${host.host_anchor}|${host.form_instance_id}`);
      if (artifact === undefined || artifact.id !== host.submission_host_id || artifact.event !== host.event || artifact.submit_action !== host.submit_action || artifact.action_batch !== host.action_batch || artifact.serialization_plan !== host.serialization_plan || artifact.prevent_default !== host.prevent_default) {
        reportDiagnostic(diagnostics, "EZR_FORMS_MANIFEST_MISMATCH", "Forms manifest host bridge did not match an exact compiler host record", { host }, true);
        throw new EdgeZeroBootError("EZR_FORMS_MANIFEST_MISMATCH");
      }
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

  function readEffectArtifact(diagnostics) {
    const element = document.getElementById(EFFECT_ARTIFACT_ELEMENT_ID);

    if (element === null) {
      return null;
    }

    if (!(element instanceof HTMLScriptElement)) {
      reportDiagnostic(
        diagnostics,
        "EZR_INVALID_EFFECT_ARTIFACT",
        "Effect runtime metadata was not stored in a script element",
        { artifactElementId: EFFECT_ARTIFACT_ELEMENT_ID },
        true
      );
      throw new EdgeZeroBootError("EZR_INVALID_EFFECT_ARTIFACT");
    }

    try {
      return JSON.parse(element.textContent ?? "");
    } catch (error) {
      reportDiagnostic(
        diagnostics,
        "EZR_INVALID_EFFECT_ARTIFACT",
        "Effect runtime metadata JSON could not be parsed",
        { message: error instanceof Error ? error.message : String(error) },
        true
      );
      throw new EdgeZeroBootError("EZR_INVALID_EFFECT_ARTIFACT");
    }
  }

  function validateEffectArtifactSchema(artifact, diagnostics) {
    if (artifact === null) {
      return;
    }

    if (artifact.schema_version !== SUPPORTED_EFFECT_ARTIFACT_SCHEMA_VERSION) {
      reportDiagnostic(
        diagnostics,
        "EZR_UNSUPPORTED_EFFECT_ARTIFACT_SCHEMA",
        `Unsupported effect runtime metadata schema version ${String(artifact.schema_version)}`,
        {
          schema_version: artifact.schema_version,
          supported_schema_version: SUPPORTED_EFFECT_ARTIFACT_SCHEMA_VERSION
        },
        true
      );
      throw new EdgeZeroBootError("EZR_UNSUPPORTED_EFFECT_ARTIFACT_SCHEMA");
    }
  }

  function readComponentArtifact(diagnostics) {
    const element = document.getElementById(COMPONENT_ARTIFACT_ELEMENT_ID);
    if (element === null) return null;
    if (!(element instanceof HTMLScriptElement)) {
      reportDiagnostic(diagnostics, "EZR_INVALID_COMPONENT_ARTIFACT", "Component runtime metadata was not stored in a script element", { artifactElementId: COMPONENT_ARTIFACT_ELEMENT_ID }, true);
      throw new EdgeZeroBootError("EZR_INVALID_COMPONENT_ARTIFACT");
    }
    try { return JSON.parse(element.textContent ?? ""); } catch (error) {
      reportDiagnostic(diagnostics, "EZR_INVALID_COMPONENT_ARTIFACT", "Component runtime metadata JSON could not be parsed", { message: error instanceof Error ? error.message : String(error) }, true);
      throw new EdgeZeroBootError("EZR_INVALID_COMPONENT_ARTIFACT");
    }
  }

  function validateComponentArtifactSchema(artifact, diagnostics) {
    if (artifact === null) return;
    if ((artifact.schema_version !== SUPPORTED_COMPONENT_ARTIFACT_SCHEMA_VERSION && artifact.schema_version !== LEGACY_COMPONENT_ARTIFACT_SCHEMA_VERSION) || !Array.isArray(artifact.instances) || !Array.isArray(artifact.initialization_batches)) {
      reportDiagnostic(diagnostics, "EZR_INVALID_COMPONENT_ARTIFACT", "Component runtime metadata did not match the compiler artifact contract", { schema_version: artifact.schema_version }, true);
      throw new EdgeZeroBootError("EZR_INVALID_COMPONENT_ARTIFACT");
    }
    const instances = new Set(artifact.instances.map((instance) => instance.instance));
    for (const instance of artifact.instances) if (instance.parent !== null && instance.parent !== undefined && !instances.has(instance.parent)) {
      reportDiagnostic(diagnostics, "EZR_INVALID_COMPONENT_ARTIFACT", "Component runtime metadata referenced an unknown parent instance", { instance: instance.instance, parent: instance.parent }, true);
      throw new EdgeZeroBootError("EZR_INVALID_COMPONENT_ARTIFACT");
    }
  }

  function readContextArtifact(diagnostics) {
    const element = document.getElementById(CONTEXT_ARTIFACT_ELEMENT_ID);
    if (!(element instanceof HTMLScriptElement)) {
      reportDiagnostic(diagnostics, "EZR_INVALID_CONTEXT_ARTIFACT", "Context runtime metadata was not stored in a script element", { artifactElementId: CONTEXT_ARTIFACT_ELEMENT_ID }, true);
      throw new EdgeZeroBootError("EZR_INVALID_CONTEXT_ARTIFACT");
    }
    try { return JSON.parse(element.textContent ?? ""); }
    catch (error) {
      reportDiagnostic(diagnostics, "EZR_INVALID_CONTEXT_ARTIFACT", "Context runtime metadata JSON could not be parsed", { message: error instanceof Error ? error.message : String(error) }, true);
      throw new EdgeZeroBootError("EZR_INVALID_CONTEXT_ARTIFACT");
    }
  }

  function validateContextArtifactSchema(artifact, diagnostics) {
    if (artifact.schema_version !== SUPPORTED_CONTEXT_ARTIFACT_SCHEMA_VERSION) {
      reportDiagnostic(diagnostics, "EZR_UNSUPPORTED_CONTEXT_ARTIFACT_SCHEMA", `Unsupported Context runtime metadata schema version ${String(artifact.schema_version)}`, { schema_version: artifact.schema_version, supported_schema_version: SUPPORTED_CONTEXT_ARTIFACT_SCHEMA_VERSION }, true);
      throw new EdgeZeroBootError("EZR_UNSUPPORTED_CONTEXT_ARTIFACT_SCHEMA");
    }
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
      const record = actionsByMethod.get(action.method_id) ?? {
        action_batch_id: action.action_batch_id,
        actions: []
      };
      record.actions.push(action);
      actionsByMethod.set(action.method_id, record);
    }

    return actionsByMethod;
  }

  function componentFieldKey(componentName, field) {
    return `${componentName}:${field}`;
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

  function createRuntimeStore(elementsByNode, diagnostics, computedArtifact, contextArtifact, effectArtifact) {
    const computedEvaluations = new Map();
    const storageValues = new Map();
    const storageByComponentField = new Map();
    const invalidationsByStorage = new Map();
    const computedDirty = new Map();

    for (const state of computedArtifact?.state ?? []) {
      storageValues.set(state.storage, state.initial_value);
      storageByComponentField.set(
        componentFieldKey(state.component, state.field),
        state.storage
      );
    }

    for (const invalidation of computedArtifact?.invalidations ?? []) {
      invalidationsByStorage.set(invalidation.storage, invalidation.dependents ?? []);
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
      contextArtifact,
      effectArtifact,
      contextSlots: new Map(),
      contextConsumerBindings: new Map(),
      contextInitialSourceRuns: [],
      contextUpdateSourceRuns: [],
      contextFailures: [],
      computedEvaluations,
      computedDirty,
      computedValues: new Map(),
      computedCaches: new Map(),
      computedSlotsByInstanceComputed: new Map(),
      computedDirtySlots: new Map(),
      storageValues,
      storageByComponentField,
      invalidationsByStorage,
      computedUpdateRuns: 0,
      initialEffectRuns: [],
      completedActionEffectRuns: [],
      activeActionBatch: null
    };
  }

  function computedSlotForExecution(store, computed) {
    const componentInstanceId = store.activeExecutionContext?.component_instance_id;
    if (componentInstanceId === undefined) return undefined;
    const slot = store.computedSlotsByInstanceComputed.get(`${componentInstanceId}|${computed}`);
    if (slot === undefined) throw new EdgeZeroBootError("EZR_INVALID_COMPONENT_ARTIFACT");
    return slot;
  }

  function isComputedDirty(store, computed) {
    const slot = computedSlotForExecution(store, computed);
    return slot === undefined
      ? store.computedDirty.get(computed) === true
      : store.computedDirtySlots.get(slot.dirty_slot_id) === true;
  }

  function setComputedDirty(store, computed, value) {
    const slot = computedSlotForExecution(store, computed);
    if (slot === undefined) store.computedDirty.set(computed, value);
    else store.computedDirtySlots.set(slot.dirty_slot_id, value);
  }

  function computedValue(store, computed) {
    const slot = computedSlotForExecution(store, computed);
    return slot === undefined
      ? store.computedValues.get(computed)
      : store.computedCaches.get(slot.cache_slot_id);
  }

  function storeComputedValue(store, evaluation, value) {
    const slot = computedSlotForExecution(store, evaluation.computed);
    if (slot === undefined) {
      store.computedValues.set(evaluation.computed, value);
      store.computedCaches.set(evaluation.cache_slot, value);
    } else {
      store.computedCaches.set(slot.cache_slot_id, value);
    }
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

  function executePureProgramInstruction(store, values, instruction, subject) {
    if (instruction.kind === "constant") {
      values.set(instruction.result, instruction.value);
      return true;
    }

    if (instruction.kind === "load-state") {
      values.set(instruction.result, store.storageValues.get(instruction.storage));
      return true;
    }

    if (instruction.kind === "load-computed") {
      if (isComputedDirty(store, instruction.computed)) {
        reportDiagnostic(
          store.diagnostics,
          "EZR_UNPLANNED_COMPUTED_DEPENDENCY",
          "Compiler program depended on a value not yet evaluated by the compiler plan",
          { subject, dependency: instruction.computed }
        );
        values.set(instruction.result, undefined);
      } else {
        values.set(instruction.result, computedValue(store, instruction.computed));
      }
      return true;
    }

    if (instruction.kind === "get-member") {
      const object = computedOperandValue(store, values, instruction.object);
      const value = object !== null && typeof object === "object"
        && Object.prototype.hasOwnProperty.call(object, instruction.property)
        ? object[instruction.property]
        : undefined;
      values.set(instruction.result, value);
      return true;
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
      return true;
    }

    if (instruction.kind === "unary") {
      values.set(
        instruction.result,
        computedUnary(
          instruction.operation,
          computedOperandValue(store, values, instruction.operand)
        )
      );
      return true;
    }

    return false;
  }

  function executeComputedProgram(store, evaluation) {
    const values = new Map();

    for (const instruction of evaluation.program?.instructions ?? []) {
      executePureProgramInstruction(store, values, instruction, evaluation.computed);
    }

    return values.get(evaluation.program?.result);
  }

  function initialEffectBatches(effectArtifact) {
    const batches = new Map();

    for (const effect of effectArtifact?.effects ?? []) {
      const trigger = effect.initial_trigger;

      if (trigger === null || trigger === undefined) {
        continue;
      }

      const effects = batches.get(trigger.effect_batch_index) ?? [];
      effects.push(effect);
      batches.set(trigger.effect_batch_index, effects);
    }

    return [...batches.entries()].sort(([left], [right]) => left - right);
  }

  function dispatchEffectCapability(store, effect, instruction, values, evidence) {
    const runtimeLowering = instruction.runtime_lowering;
    const arguments = (instruction.arguments ?? []).map((operand) =>
      computedOperandValue(store, values, operand)
    );
    const value = computedOperandValue(store, values, instruction.value);

    switch (runtimeLowering) {
      case "builtin.browser.document.title.assign":
        document.title = value;
        break;
      case "builtin.browser.console.log":
        console.log(...arguments);
        break;
      case "builtin.browser.console.info":
        console.info(...arguments);
        break;
      case "builtin.browser.console.warn":
        console.warn(...arguments);
        break;
      case "builtin.browser.console.error":
        console.error(...arguments);
        break;
      case "builtin.browser.local_storage.set_item":
        localStorage.setItem(...arguments);
        break;
      case "builtin.browser.local_storage.remove_item":
        localStorage.removeItem(...arguments);
        break;
      case "builtin.browser.session_storage.set_item":
        sessionStorage.setItem(...arguments);
        break;
      case "builtin.browser.session_storage.remove_item":
        sessionStorage.removeItem(...arguments);
        break;
      default:
        reportDiagnostic(
          store.diagnostics,
          "EZR_UNSUPPORTED_EFFECT_CAPABILITY",
          "Effect program referenced an unsupported compiler runtime lowering",
          { effect: effect.effect, runtime_lowering: runtimeLowering }
        );
        return;
    }

    evidence.capability_operations.push({
      operation: instruction.operation,
      runtime_lowering: runtimeLowering
    });
  }

  function executeEffectProgram(store, effect, evidence) {
    const values = new Map();

    for (const instruction of effect.program?.instructions ?? []) {
      if (executePureProgramInstruction(store, values, instruction, effect.effect)) {
        continue;
      }

      if (instruction.kind === "capability-call" || instruction.kind === "capability-assign") {
        dispatchEffectCapability(store, effect, instruction, values, evidence);
        continue;
      }

      reportDiagnostic(
        store.diagnostics,
        "EZR_UNSUPPORTED_EFFECT_INSTRUCTION",
        "Effect program contained an unsupported compiler instruction",
        { effect: effect.effect, kind: instruction.kind }
      );
    }
  }

  function executeInitialEffects(store) {
    for (const [effectBatchIndex, effects] of initialEffectBatches(store.effectArtifact)) {
      for (const effect of effects) {
        const evidence = {
          effect: effect.effect,
          effect_batch_index: effectBatchIndex,
          capability_operations: []
        };
        executeEffectProgram(store, effect, evidence);
        store.initialEffectRuns.push(evidence);
      }
    }
  }

  function actionEffectBatches(effectArtifact, actionBatchId) {
    const batches = new Map();

    for (const effect of effectArtifact?.effects ?? []) {
      const trigger = (effect.action_batch_triggers ?? []).find(
        (candidate) => candidate.action_batch === actionBatchId
      );

      if (trigger === undefined) {
        continue;
      }

      const effects = batches.get(trigger.effect_batch_index) ?? [];
      effects.push(effect);
      batches.set(trigger.effect_batch_index, effects);
    }

    return [...batches.entries()].sort(([left], [right]) => left - right);
  }

  function executeCompletedActionEffects(store, actionBatchId) {
    for (const [effectBatchIndex, effects] of actionEffectBatches(
      store.effectArtifact,
      actionBatchId
    )) {
      for (const effect of effects) {
        const evidence = {
          action_batch_id: actionBatchId,
          effect: effect.effect,
          effect_batch_index: effectBatchIndex,
          capability_operations: []
        };
        executeEffectProgram(store, effect, evidence);
        store.completedActionEffectRuns.push(evidence);
      }
    }
  }

  function executeComputedPlan(store, componentInstanceId = null) {
    if (store.computedArtifact === null) {
      return;
    }

    const priorExecutionContext = store.activeExecutionContext;
    if (componentInstanceId !== null) {
      store.activeExecutionContext = { component_instance_id: componentInstanceId };
    }

    try {
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

        if (!isComputedDirty(store, computed)) continue;

        storeComputedValue(store, evaluation, executeComputedProgram(store, evaluation));
        setComputedDirty(store, computed, false);
      }
    } finally {
      store.activeExecutionContext = priorExecutionContext;
    }
  }

  function executeInitialContext(store) {
    const sources = new Map((store.contextArtifact?.sources ?? []).map((source) => [source.source, source]));
    for (const batch of store.contextArtifact?.initial_batches ?? []) {
      for (const sourceId of batch.sources ?? []) {
        const source = sources.get(sourceId);
        if (source === undefined) { continue; }
        const unavailable = (source.required_computed ?? []).some((computed) => store.computedDirty.get(computed) === true);
        if (unavailable) {
          store.contextFailures.push({ source: source.source, failure: "unavailable-computed-prerequisite" });
          continue;
        }
        const values = new Map();
        let initialized = false;
        for (const instruction of source.program?.instructions ?? []) {
          if (executePureProgramInstruction(store, values, instruction, source.source)) { continue; }
          if (instruction.kind === "initialize_context_slot") {
            store.contextSlots.set(instruction.slot, computedOperandValue(store, values, instruction.value));
            initialized = true;
            continue;
          }
          store.contextFailures.push({ source: source.source, failure: `unsupported-instruction:${String(instruction.kind)}` });
          break;
        }
        if (initialized) { store.contextInitialSourceRuns.push(source.source); }
      }
    }
    for (const consumer of store.contextArtifact?.consumers ?? []) {
      store.contextConsumerBindings.set(consumer.consumer, consumer.slot);
      if (!store.contextSlots.has(consumer.slot)) {
        store.contextFailures.push({ consumer: consumer.consumer, failure: "source-slot-unavailable" });
      }
    }
  }

  function executeContextUpdates(store, actionBatchId) {
    const update = (store.contextArtifact?.action_updates ?? []).find(
      (candidate) => candidate.action_batch === actionBatchId
    );
    if (update === undefined || update.invalidated_sources.length === 0) { return; }
    const sources = new Map((store.contextArtifact?.sources ?? []).map((source) => [source.source, source]));
    for (const sourceId of update.invalidated_sources) {
      const source = sources.get(sourceId);
      if (source === undefined) { continue; }
      const values = new Map();
      let initialized = false;
      for (const instruction of source.program?.instructions ?? []) {
        if (executePureProgramInstruction(store, values, instruction, source.source)) { continue; }
        if (instruction.kind === "initialize_context_slot") {
          store.contextSlots.set(instruction.slot, computedOperandValue(store, values, instruction.value));
          initialized = true;
          continue;
        }
        store.contextFailures.push({ action_batch: actionBatchId, source: source.source, failure: `unsupported-update-instruction:${String(instruction.kind)}` });
        break;
      }
      if (initialized) { store.contextUpdateSourceRuns.push({ action_batch: actionBatchId, source: source.source }); }
    }
  }

  function executeComputedUpdateBatches(store) {
    if (store.computedArtifact === null) {
      return;
    }

    let executed = false;

    for (const batch of store.computedArtifact.update_batches ?? []) {
      for (const computed of batch) {
        if (!isComputedDirty(store, computed)) {
          continue;
        }

        const evaluation = store.computedEvaluations.get(computed);

        if (evaluation === undefined) {
          reportDiagnostic(
            store.diagnostics,
            "EZR_UNPLANNED_COMPUTED_DEPENDENCY",
            "Compiler update batch referenced a missing computed evaluation",
            { computed }
          );
          continue;
        }

        storeComputedValue(store, evaluation, executeComputedProgram(store, evaluation));
        setComputedDirty(store, computed, false);
        executed = true;
      }
    }

    if (executed) {
      store.computedUpdateRuns += 1;
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
    const storage = store.storageByComponentField.get(
      componentFieldKey(component.name, field)
    );

    if (storage !== undefined) {
      store.storageValues.set(storage, value);
      for (const computed of store.invalidationsByStorage.get(storage) ?? []) {
        setComputedDirty(store, computed, true);
      }
    }
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

    for (const [methodId, actionRecord] of actionsByMethod) {
      store.actionsByMethod.set(methodId, actionRecord);
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

    const actionRecord = store.actionsByMethod.get(event.method_id);

    if (
      actionRecord === undefined ||
      actionRecord.action_batch_id !== event.action_batch_id
    ) {
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
      action_batch_id: event.action_batch_id,
      actions: actionRecord.actions
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

  function executeActions(store, component, actionBatchId, actions, executionContext = null) {
    store.activeActionBatch = actionBatchId;
    store.activeExecutionContext = executionContext;

    try {
      for (const action of actions) {
        executeAction(store, component, action);
      }

      executeComputedUpdateBatches(store);
      executeContextUpdates(store, actionBatchId);
      executeCompletedActionEffects(store, actionBatchId);
    } finally {
      store.activeActionBatch = null;
      store.activeExecutionContext = null;
      refreshComputedDebugState(store);
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

    executeActions(store, record.component, record.action_batch_id, record.actions);
  }

  function installDelegatedEventListeners(store) {
    for (const eventType of store.eventsByType.keys()) {
      document.addEventListener(eventType, (event) => {
        dispatchDelegatedEvent(store, event);
      });
    }
  }

  function collectOrdinaryTargetAnchors() {
    const targets = new Map();
    const duplicates = new Set();
    for (const element of document.querySelectorAll("[data-ez-ti]")) {
      const id = element.getAttribute("data-ez-ti");
      if (id === null) continue;
      if (targets.has(id)) duplicates.add(id);
      targets.set(id, element);
    }
    return { targets, duplicates };
  }

  function ordinaryEventKey(targetId, eventType) {
    return `${targetId}\u001f${eventType}`;
  }

  function ordinaryTextBindingNode(bindingId) {
    const walker = document.createTreeWalker(document, NodeFilter.SHOW_COMMENT);
    const start = `ez-ti-binding-start:${bindingId}`;
    const end = `ez-ti-binding-end:${bindingId}`;
    let startMarker = null;
    while (walker.nextNode()) {
      if (walker.currentNode.data === start) { startMarker = walker.currentNode; continue; }
      if (startMarker !== null && walker.currentNode.data === end) {
        const text = startMarker.nextSibling;
        return text instanceof Text ? text : null;
      }
    }
    return null;
  }

  function registerOrdinaryBinding(store, binding) {
    const component = store.components.get(binding.component_instance_id);
    const field = fieldNameFromThisMember(binding.expression);
    if (component === undefined || field === null) throw new EdgeZeroBootError("EZR_INVALID_ORDINARY_BINDING");
    const target = store.templateTargetsById.get(binding.instance_target_id);
    const context = { component_instance_id: binding.component_instance_id };
    let update = null;
    if (binding.kind === "text") {
      const text = ordinaryTextBindingNode(binding.instance_binding_id);
      if (text !== null) update = (value) => { void context; text.data = formatBindingValue(value); };
    } else if ((binding.kind === "attribute" || binding.kind === "property") && target instanceof Element && typeof binding.attribute_name === "string") {
      update = (value) => { void context; updateAttributeBinding(target, binding.attribute_name, value); };
    }
    if (update === null) throw new EdgeZeroBootError("EZR_INVALID_ORDINARY_BINDING");
    update(component.state[field]);
    registerBinding(store, component, field, update);
  }

  function initializeOrdinaryInstanceRuntime(store, manifest, componentArtifact) {
    if (manifest.schema_version !== SUPPORTED_SCHEMA_VERSION) return;
    const anchors = collectOrdinaryTargetAnchors();
    const artifactTargets = new Map((componentArtifact.ordinary_template_targets ?? []).map((target) => [target.id, target]));
    const artifactBindings = new Map((componentArtifact.ordinary_template_bindings ?? []).map((binding) => [binding.id, binding]));
    const artifactEvents = new Map((componentArtifact.ordinary_template_events ?? []).map((event) => [ordinaryEventKey(event.target_id, event.event_type), event]));
    store.templateTargetsById = anchors.targets;
    store.ordinaryBindingsById = new Map();
    store.ordinaryEventsByTargetAndType = new Map();
    for (const target of manifest.ordinary_targets ?? []) {
      const artifactTarget = artifactTargets.get(target.id);
      if (artifactTarget === undefined || artifactTarget.component_instance_id !== target.component_instance_id || anchors.duplicates.has(target.id) || !anchors.targets.has(target.id)) {
        throw new EdgeZeroBootError("EZR_INVALID_ORDINARY_TARGET");
      }
    }
    for (const binding of manifest.ordinary_bindings ?? []) {
      const artifactBinding = artifactBindings.get(binding.instance_binding_id);
      if (artifactBinding === undefined || artifactBinding.component_instance_id !== binding.component_instance_id || artifactBinding.target_id !== binding.instance_target_id) {
        throw new EdgeZeroBootError("EZR_INVALID_ORDINARY_BINDING");
      }
      store.ordinaryBindingsById.set(binding.instance_binding_id, {
        ...binding,
        execution_context: { component_instance_id: binding.component_instance_id }
      });
      registerOrdinaryBinding(store, binding);
    }
    for (const event of manifest.ordinary_events ?? []) {
      const key = ordinaryEventKey(event.instance_target_id, event.event_type);
      const artifactEvent = artifactEvents.get(key);
      if (artifactEvent === undefined || artifactEvent.component_instance_id !== event.component_instance_id || store.ordinaryEventsByTargetAndType.has(key)) {
        throw new EdgeZeroBootError("EZR_INVALID_ORDINARY_EVENT");
      }
      store.ordinaryEventsByTargetAndType.set(key, event);
    }
  }

  function ordinaryTargetFromEvent(target) {
    let current = target instanceof Element ? target : target?.parentElement;
    while (current !== null && current !== undefined) {
      const targetId = current.getAttribute("data-ez-ti");
      if (targetId !== null) return targetId;
      current = current.parentElement;
    }
    return null;
  }

  function dispatchOrdinaryInstanceEvent(store, event) {
    const targetId = ordinaryTargetFromEvent(event.target);
    if (targetId === null) return;
    const record = store.ordinaryEventsByTargetAndType.get(ordinaryEventKey(targetId, event.type));
    if (record === undefined) return;
    const actionRecord = store.actionsByMethod.get(record.handler_method_id);
    const component = store.components.get(record.component_instance_id);
    if (actionRecord === undefined || component === undefined || actionRecord.action_batch_id !== record.action_batch_id) {
      throw new EdgeZeroBootError("EZR_INVALID_ORDINARY_EVENT");
    }
    const context = {
      component_instance_id: record.component_instance_id,
      trigger_target_id: record.instance_target_id,
      declaration_event_id: record.declaration_event_id,
      action_batch_id: record.action_batch_id
    };
    executeActions(store, component, record.action_batch_id, actionRecord.actions, context);
  }

  function installOrdinaryInstanceEventListeners(store) {
    for (const key of store.ordinaryEventsByTargetAndType.keys()) {
      const eventType = key.slice(key.lastIndexOf("\u001f") + 1);
      document.addEventListener(eventType, (event) => dispatchOrdinaryInstanceEvent(store, event));
    }
  }

  // Forms are initialized exclusively from the compiler artifact and manifest
  // bridge. The DOM contributes only the user event value for a known anchor.
  function initializeFormsRuntime(store, formsArtifact, manifest, elementsByNode, diagnostics) {
    store.forms = new Map();
    store.formInstances = new Map();
    store.formBindingsByAnchor = new Map();
    store.formBindingsByField = new Map();
    store.formHostsByAnchor = new Map();
    if (formsArtifact === null) return;

    const definitions = new Map(formsArtifact.forms.map((form) => [form.id, form]));
    for (const instance of formsArtifact.instances) {
      const definition = definitions.get(instance.form);
      if (definition === undefined) {
        reportDiagnostic(diagnostics, "EZR_UNKNOWN_FORM_INSTANCE", "Forms artifact referenced an unknown Form definition", { instance: instance.id, form: instance.form }, true);
        continue;
      }
      const fields = new Map(definition.fields.map((field) => [field.id, {
        value: field.initial_value,
        initial: field.initial_value,
        dirty: false,
        touched: false,
        validation: []
      }]));
      store.forms.set(definition.id, definition);
      store.formInstances.set(instance.id, {
        definition,
        instance,
        fields,
        aggregate_valid: true,
        submission: "Idle"
      });
    }

    for (const bridge of manifest.form_bindings ?? []) {
      const element = manifest.schema_version === SUPPORTED_SCHEMA_VERSION
        ? store.templateTargetsById.get(bridge.instance_target_id)
        : elementsByNode.get(bridge.control_anchor);
      const formInstance = store.formInstances.get(bridge.form_instance_id);
      if (element === undefined || formInstance === undefined) {
        reportDiagnostic(diagnostics, "EZR_FORMS_MANIFEST_MISMATCH", "Forms manifest bridge did not resolve an exact compiler anchor and instance", { bridge }, true);
        continue;
      }
      const binding = formInstance.definition.bindings.find((item) => item.id === bridge.field_binding_id);
      if (binding === undefined || binding.field === undefined || binding.channel !== bridge.channel) {
        reportDiagnostic(diagnostics, "EZR_UNKNOWN_FORM_BINDING", "Forms manifest bridge did not match an artifact binding", { bridge }, true);
        continue;
      }
      const record = { bridge, binding, element, formInstance };
      store.formBindingsByAnchor.set(manifest.schema_version === SUPPORTED_SCHEMA_VERSION ? bridge.instance_target_id : bridge.control_anchor, record);
      const key = `${bridge.form_instance_id}|${binding.field}`;
      const bindings = store.formBindingsByField.get(key) ?? [];
      bindings.push(record);
      store.formBindingsByField.set(key, bindings);
      writeFormControl(record, formInstance.fields.get(binding.field)?.value);
    }

    for (const bridge of manifest.form_hosts ?? []) {
      const element = manifest.schema_version === SUPPORTED_SCHEMA_VERSION
        ? store.templateTargetsById.get(bridge.instance_target_id)
        : elementsByNode.get(bridge.host_anchor);
      const formInstance = store.formInstances.get(bridge.form_instance_id);
      const host = (formsArtifact.hosts ?? []).find((candidate) => candidate.host_anchor === bridge.host_anchor && candidate.form_instance === bridge.form_instance_id);
      if (!(element instanceof HTMLFormElement) || formInstance === undefined || host === undefined || host.event !== "submit") {
        reportDiagnostic(diagnostics, "EZR_FORMS_MANIFEST_MISMATCH", "Forms host bridge did not resolve an exact compiler-owned form anchor", { bridge }, true);
        continue;
      }
      const anchor = manifest.schema_version === SUPPORTED_SCHEMA_VERSION ? bridge.instance_target_id : bridge.host_anchor;
      store.formHostsByAnchor.set(anchor, { bridge, host, element, formInstance });
      element.addEventListener(host.event, (event) => dispatchFormSubmit(store, event, anchor));
    }

    document.addEventListener("input", (event) => dispatchFormEvent(store, event, false));
    document.addEventListener("change", (event) => dispatchFormEvent(store, event, false));
    document.addEventListener("focusout", (event) => dispatchFormEvent(store, event, true));
    window.__EDGEZERO_FORMS__ = {
      resetForm: (instanceId) => resetForm(store, instanceId),
      resetField: (instanceId, fieldId) => resetField(store, instanceId, fieldId)
    };
  }

  function dispatchFormSubmit(store, event, anchor) {
    const record = store.formHostsByAnchor.get(anchor);
    if (record === undefined || event.type !== record.host.event) return;
    if (record.host.prevent_default === true) event.preventDefault();
    for (const fieldId of record.formInstance.fields.keys()) validateFormField(record.formInstance, fieldId);
    if (!record.formInstance.aggregate_valid) { record.formInstance.submission = "Invalid"; return; }
    const action = store.actionsByMethod.get(record.host.submit_action);
    const component = store.components.get(record.bridge.component_instance_id) ?? action?.component;
    if (action === undefined || action.action_batch_id !== record.host.action_batch || component === undefined) {
      reportDiagnostic(store.diagnostics, "EZR_UNRESOLVED_FORM_SUBMIT_ACTION", "Submission host did not resolve its exact compiler action", { host: record.host }, true);
      record.formInstance.submission = "Failed";
      return;
    }
    record.formInstance.submission = "Submitting";
    // Serialization is deliberately compiler-record driven: field values come
    // from the instance store, never DOM scanning or a form-element snapshot.
    record.formInstance.serialized = [...record.formInstance.fields.entries()].map(([field, state]) => ({ field, value: state.value }));
    executeActions(store, component, record.host.action_batch, action.actions, {
      component_instance_id: record.bridge.component_instance_id,
      trigger_target_id: record.bridge.instance_target_id,
      declaration_event_id: record.host.submit_action,
      action_batch_id: record.host.action_batch
    });
    record.formInstance.submission = "Completed";
  }

  function dispatchFormEvent(store, event, blur) {
    const element = event.target;
    if (!(element instanceof HTMLElement)) return;
    const anchor = store.templateTargetsById instanceof Map
      ? ordinaryTargetFromEvent(element)
      : element.getAttribute("data-ez-node");
    const record = anchor === null ? undefined : store.formBindingsByAnchor.get(anchor);
    if (record === undefined) return;
    if (blur) {
      const state = record.formInstance.fields.get(record.binding.field);
      state.touched = true;
      validateFormField(record.formInstance, record.binding.field);
      return;
    }
    const expected = record.binding.channel === "Checked" || record.binding.channel === "RadioValue" ? "change" : "input";
    if (event.type !== expected) return;
    const value = readFormControl(record);
    if (value === undefined) return;
    writeFormField(store, record.formInstance, record.binding.field, value);
  }

  function readFormControl(record) {
    const { element, binding } = record;
    if (binding.channel === "Checked") return element.checked === true;
    if (binding.channel === "NumericValue") {
      if (element.value === "") return binding.normalization === "NullableNumber" ? null : undefined;
      const value = Number(element.value);
      return Number.isFinite(value) ? value : undefined;
    }
    if (binding.channel === "SelectedValues") return [...element.selectedOptions].map((option) => option.value);
    return element.value;
  }

  function writeFormControl(record, value) {
    const { element, binding } = record;
    if (binding.channel === "Checked") element.checked = value === true;
    else if (binding.channel === "SelectedValues") {
      for (const option of element.options ?? []) option.selected = Array.isArray(value) && value.includes(option.value);
    } else element.value = value === null ? "" : String(value ?? "");
  }

  function writeFormField(store, formInstance, fieldId, value) {
    const state = formInstance.fields.get(fieldId);
    if (state === undefined) return;
    state.value = value;
    state.dirty = JSON.stringify(value) !== JSON.stringify(state.initial);
    validateFormField(formInstance, fieldId);
    for (const dependency of formInstance.definition.validation_dependencies ?? []) {
      if (dependency.source_field === fieldId) validateFormField(formInstance, dependency.target_field);
    }
    for (const record of store.formBindingsByField.get(`${formInstance.instance.id}|${fieldId}`) ?? []) writeFormControl(record, value);
  }

  function validateFormField(formInstance, fieldId) {
    const state = formInstance.fields.get(fieldId);
    if (state === undefined) return;
    const rules = (formInstance.definition.validation_rules ?? []).filter((rule) => rule.target_field === fieldId);
    state.validation = rules.filter((rule) => !validateFormRule(formInstance, state.value, rule)).map((rule) => rule.id);
    formInstance.aggregate_valid = [...formInstance.fields.values()].every((field) => field.validation.length === 0);
  }

  function validateFormRule(formInstance, value, rule) {
    const dependency = rule.dependency === undefined ? undefined : formInstance.fields.get(rule.dependency)?.value;
    if (rule.kind === "Required") return !(value === null || value === undefined || value === "" || (Array.isArray(value) && value.length === 0));
    if (rule.kind === "Email") return value === "" || /^[^@\s]+@[^@\s]+\.[^@\s]+$/.test(String(value));
    if (rule.kind === "Equals") return value === dependency;
    if (rule.kind === "NotEquals") return value !== dependency;
    return true;
  }

  function resetField(store, instanceId, fieldId) {
    const formInstance = store.formInstances?.get(instanceId);
    const state = formInstance?.fields.get(fieldId);
    if (state === undefined) return false;
    state.value = state.initial; state.dirty = false; state.touched = false; state.validation = [];
    for (const record of store.formBindingsByField.get(`${instanceId}|${fieldId}`) ?? []) writeFormControl(record, state.value);
    formInstance.aggregate_valid = true;
    return true;
  }

  function resetForm(store, instanceId) {
    const formInstance = store.formInstances?.get(instanceId);
    if (formInstance === undefined) return false;
    for (const fieldId of formInstance.fields.keys()) resetField(store, instanceId, fieldId);
    formInstance.submission = "Idle";
    return true;
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

  function refreshComputedDebugState(store) {
    if (window.__EDGEZERO__?.store !== store) {
      return;
    }

    window.__EDGEZERO__.computed = debugComputed(store);
    window.__EDGEZERO__.computed_update_runs = store.computedUpdateRuns;
  }

  function runtimeState({
    manifest = null,
    missingAnchors = [],
    store = null,
    components = [],
    computed = [],
    computed_update_runs = 0,
    initial_effect_runs = [],
    completed_action_effect_runs = [],
    context_initial_source_runs = [],
    context_slots = [],
    context_consumer_bindings = [],
    context_failures = [],
    context_update_source_runs = [],
    component_initialization_runs = [],
    component_instance_tree = [],
    slot_binding_runs = [],
    component_failures = [],
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
      computed,
      computed_update_runs,
      initial_effect_runs,
      completed_action_effect_runs,
      context_initial_source_runs,
      context_slots,
      context_consumer_bindings,
      context_failures,
      context_update_source_runs,
      component_initialization_runs,
      component_instance_tree,
      slot_binding_runs,
      component_failures
    };
  }

  function initializeRuntime(manifest, computedArtifact, contextArtifact, effectArtifact, componentArtifact, formsArtifact, diagnostics) {
    const bindingAnchors = collectBindingAnchors();
    const conditionalAnchors = collectConditionalAnchors();
    const listAnchors = collectListAnchors();
    const elementsByNode = collectElementAnchors();
    const store = createRuntimeStore(elementsByNode, diagnostics, computedArtifact, contextArtifact, effectArtifact);
    store.componentArtifact = componentArtifact;
    store.componentInstances = new Map((componentArtifact?.instances ?? []).map((instance) => [instance.instance, { ...instance, status: "created" }]));
    store.slotBindings = new Map((componentArtifact?.slot_binding_programs ?? []).map((binding) => [binding.binding, binding]));
    store.instanceContextBindings = new Map((componentArtifact?.instance_context_bindings ?? []).map((binding) => [binding.consumer_instance, binding]));
    store.componentRegions = new Map((componentArtifact?.structural_programs ?? []).map((program) => [program.region, program]));
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

    if (manifest.schema_version === SUPPORTED_SCHEMA_VERSION) {
      const definitions = new Map((manifest.components ?? []).map((component) => [component.component_id, component]));
      for (const instance of componentArtifact.instances ?? []) {
        const definition = definitions.get(instance.component);
        if (definition === undefined) throw new EdgeZeroBootError("EZR_INVALID_ORDINARY_COMPONENT");
        for (const slot of instance.computed_slots ?? []) {
          const pair = `${instance.instance}|${slot.computed_id}`;
          if (!slot.cache_slot_id.startsWith(`${instance.instance}/computed-cache:`)
            || !slot.dirty_slot_id.startsWith(`${instance.instance}/computed-dirty:`)
            || store.computedSlotsByInstanceComputed.has(pair)
            || store.computedDirtySlots.has(slot.dirty_slot_id)) {
            throw new EdgeZeroBootError("EZR_INVALID_COMPONENT_ARTIFACT");
          }
          store.computedSlotsByInstanceComputed.set(pair, slot);
          store.computedDirtySlots.set(slot.dirty_slot_id, slot.dirty_initial_value === true);
        }
        const component = { name: instance.component, manifest: definition, state: {} };
        for (const state of computedArtifact?.state ?? []) {
          if (state.component === instance.component) component.state[state.field] = state.initial_value;
        }
        store.components.set(instance.instance, component);
        registerActions(store, component, definition);
      }
      initializeOrdinaryInstanceRuntime(store, manifest, componentArtifact);
    } else {
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
    }

    if (manifest.schema_version === SUPPORTED_SCHEMA_VERSION) {
      for (const instance of componentArtifact.instances ?? []) {
        executeComputedPlan(store, instance.instance);
      }
    } else {
      executeComputedPlan(store);
    }

    executeInitialContext(store);

    initializeFormsRuntime(store, formsArtifact, manifest, elementsByNode, diagnostics);

    executeInitialEffects(store);

    if (manifest.schema_version === SUPPORTED_SCHEMA_VERSION) {
      installOrdinaryInstanceEventListeners(store);
    } else {
      installDelegatedEventListeners(store);
    }

    return runtimeState({
      manifest,
      missingAnchors,
      diagnostics,
      store,
      components: debugComponents(store),
      computed: debugComputed(store),
      computed_update_runs: store.computedUpdateRuns,
      initial_effect_runs: store.initialEffectRuns,
      completed_action_effect_runs: store.completedActionEffectRuns,
      context_initial_source_runs: store.contextInitialSourceRuns,
      context_slots: [...store.contextSlots.entries()],
      context_consumer_bindings: [...store.contextConsumerBindings.entries()],
      context_failures: store.contextFailures,
      context_update_source_runs: store.contextUpdateSourceRuns,
      component_initialization_runs: (componentArtifact?.initialization_batches ?? []).map((batch) => batch.index),
      component_instance_tree: [...store.componentInstances.values()],
      forms: [...store.formInstances.values()].map((instance) => ({
        id: instance.instance.id,
        form: instance.definition.id,
        aggregate_valid: instance.aggregate_valid,
        submission: instance.submission
      })),
      slot_binding_runs: [...store.slotBindings.keys()],
      component_failures: []
    });
  }

  function boot() {
    const diagnostics = [];

    try {
      const manifest = readManifest(diagnostics);
      const computedArtifact = readComputedArtifact(diagnostics);
      validateComputedArtifactSchema(computedArtifact, diagnostics);
      const contextArtifact = readContextArtifact(diagnostics);
      validateContextArtifactSchema(contextArtifact, diagnostics);
      const effectArtifact = readEffectArtifact(diagnostics);
      validateEffectArtifactSchema(effectArtifact, diagnostics);
      const componentArtifact = readComponentArtifact(diagnostics);
      validateComponentArtifactSchema(componentArtifact, diagnostics);
      validateManifestSchema(manifest, effectArtifact, componentArtifact, diagnostics);
      const formsArtifact = readFormsArtifact(diagnostics);
      validateFormsArtifact(formsArtifact, manifest, diagnostics);

      const state = initializeRuntime(manifest, computedArtifact, contextArtifact, effectArtifact, componentArtifact, formsArtifact, diagnostics);
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
    RUNTIME_STUB.replace(
        "__EZ_COMPONENT_SCHEMA_VERSION__",
        &crate::RUNTIME_COMPONENT_ARTIFACT_SCHEMA_VERSION.to_string(),
    )
}

#[cfg(test)]
mod tests {
    use super::generate_runtime_stub;

    #[test]
    fn emits_runtime_manifest_bootstrap() {
        let runtime = generate_runtime_stub();

        assert!(runtime.contains("ez-template-manifest"));
        assert!(runtime.contains("ez-effect-runtime"));
        assert!(runtime.contains("ez-context-runtime"));
        assert!(runtime.contains("executeInitialContext(store)"));
        assert!(runtime.contains("executeContextUpdates(store, actionBatchId)"));
        assert!(runtime.contains("contextSlots: new Map()"));
        assert!(runtime.contains("RUNTIME_VERSION = \"0.0.0\""));
        assert!(runtime.contains("SUPPORTED_SCHEMA_VERSION = 4"));
        assert!(runtime.contains("ez-forms-runtime"));
        assert!(runtime.contains("initializeFormsRuntime"));
        assert!(runtime.contains("dispatchFormSubmit"));
        assert!(runtime.contains("form_hosts"));
        assert!(!runtime.contains("FormData(formElement)"));
        assert!(runtime.contains("EZR_MISSING_MANIFEST"));
        assert!(runtime.contains("EZR_INVALID_MANIFEST_JSON"));
        assert!(runtime.contains("EZR_UNSUPPORTED_SCHEMA"));
        assert!(runtime.contains("data-ez-node"));
        assert!(runtime.contains("ordinaryEventsByTargetAndType"));
        assert!(runtime.contains("component_instance_id: record.component_instance_id"));
        assert!(runtime.contains("computedSlotsByInstanceComputed: new Map()"));
        assert!(runtime.contains("computedDirtySlots: new Map()"));
        assert!(runtime.contains("function computedSlotForExecution"));
        assert!(runtime.contains("/computed-cache:"));
        assert!(runtime.contains("/computed-dirty:"));
        assert!(runtime.contains("LEGACY_COMPONENT_ARTIFACT_SCHEMA_VERSION = 2"));
        assert!(runtime.contains("ez-binding:"));
        assert!(runtime.contains("reportDiagnostic"));
        assert!(runtime.contains("validateManifestSchema"));
        assert!(runtime.contains("validateEffectArtifactSchema"));
        assert!(runtime.contains("createRuntimeStore"));
        assert!(runtime.contains("readField"));
        assert!(runtime.contains("writeField"));
        assert!(runtime.contains("notifyField"));
        assert!(runtime.contains("actionDelta"));
        assert!(runtime.contains("isBooleanAttribute"));
        assert!(runtime.contains("isPropertyAttribute"));
        assert!(runtime.contains("updateAttributeBinding"));
        assert!(runtime.contains("actionsByMethod.set(action.method_id, action.action_batch_id)"));
        assert!(runtime.contains("const actionRecord = store.actionsByMethod.get(event.method_id)"));
        assert!(runtime.contains("actionRecord.action_batch_id !== event.action_batch_id"));
        assert!(runtime.contains("executeActions"));
        assert!(runtime.contains("executeCompletedActionEffects"));
        assert!(runtime.contains("activeActionBatch"));
        assert!(runtime.contains("executeInitialEffects"));
        assert!(runtime.contains("dispatchEffectCapability"));
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
