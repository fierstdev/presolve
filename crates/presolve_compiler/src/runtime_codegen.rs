const RUNTIME_STUB: &str = r#"(() => {
  "use strict";

  const MANIFEST_ELEMENT_ID = "presolve-template-manifest";
  const COMPUTED_ARTIFACT_ELEMENT_ID = "presolve-computed-runtime";
  const EFFECT_ARTIFACT_ELEMENT_ID = "presolve-effect-runtime";
  const CONTEXT_ARTIFACT_ELEMENT_ID = "presolve-context-runtime";
  const COMPONENT_ARTIFACT_ELEMENT_ID = "presolve-component-runtime";
  const FORMS_ARTIFACT_ELEMENT_ID = "presolve-forms-runtime";
  const RESOURCES_ARTIFACT_ELEMENT_ID = "presolve-resources-runtime";
  const OPAQUE_ARTIFACT_ELEMENT_ID = "presolve-opaque-runtime";
  const RESUME_MANIFEST_ELEMENT_ID = "presolve-resume-runtime";
  const RESUME_SNAPSHOT_ELEMENT_ID = "presolve-resume-snapshot";
  const PRODUCTION_RUNTIME_ELEMENT_ID = "presolve-production-runtime";
  const RUNTIME_VERSION = "0.0.0";
  const SUPPORTED_SCHEMA_VERSION = 5;
  const ACTION_MANIFEST_SCHEMA_VERSION = 2;
  const FORMS_MANIFEST_SCHEMA_VERSION = 3;
  const LEGACY_MANIFEST_SCHEMA_VERSION = 1;
  const SUPPORTED_COMPUTED_ARTIFACT_SCHEMA_VERSION = 12;
  const SUPPORTED_EFFECT_ARTIFACT_SCHEMA_VERSION = __EZ_EFFECT_SCHEMA_VERSION__;
  const SUPPORTED_CONTEXT_ARTIFACT_SCHEMA_VERSION = 2;
  const SUPPORTED_COMPONENT_ARTIFACT_SCHEMA_VERSION = __EZ_COMPONENT_SCHEMA_VERSION__;
  const LEGACY_COMPONENT_ARTIFACT_SCHEMA_VERSION = 2;
  const SUPPORTED_FORMS_ARTIFACT_SCHEMA_VERSION = 2;
  const SUPPORTED_RESOURCES_ARTIFACT_SCHEMA_VERSION = 1;
  const SUPPORTED_OPAQUE_ARTIFACT_SCHEMA_VERSION = 1;
  const SUPPORTED_RESUME_MANIFEST_SCHEMA_VERSION = 6;
  const SUPPORTED_RESUME_SNAPSHOT_SCHEMA_VERSION = 1;
  const SUPPORTED_RESUME_RUNTIME_PROTOCOL_VERSION = 1;
  const RESUME_REGISTRY_CONTRACT_VERSION = 1;

  class PresolveBootError extends Error {
    constructor(code) {
      super(code);
      this.name = "PresolveBootError";
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
    console.error(`[Presolve] ${code}`, diagnostic);
    return diagnostic;
  }

  class ResumeBootError extends Error {
    constructor(failure, detail = {}) {
      super(failure);
      this.name = "ResumeBootError";
      this.failure = failure;
      this.detail = detail;
    }
  }

  const resumeBootstrapState = {
    phase: "idle",
    manifest: null,
    registry: null,
    result: null,
    debug: []
  };

  function exactObjectKeys(value, expected) {
    if (value === null || typeof value !== "object" || Array.isArray(value)) return false;
    const actual = Object.keys(value).sort();
    const canonical = [...expected].sort();
    return actual.length === canonical.length
      && actual.every((key, index) => key === canonical[index]);
  }

  function readResumeManifest(diagnostics) {
    const element = document.getElementById(RESUME_MANIFEST_ELEMENT_ID);
    if (!(element instanceof HTMLScriptElement)) {
      reportDiagnostic(diagnostics, "PSR_RESUME_MANIFEST_MISSING", "Resume manifest v6 is missing", { artifactElementId: RESUME_MANIFEST_ELEMENT_ID });
      throw new ResumeBootError("ManifestVersionMismatch");
    }
    try {
      return JSON.parse(element.textContent ?? "");
    } catch (error) {
      reportDiagnostic(diagnostics, "PSR_RESUME_MANIFEST_PARSE", "Resume manifest v6 could not be parsed", { message: error instanceof Error ? error.message : String(error) });
      throw new ResumeBootError("ManifestVersionMismatch");
    }
  }

  function readProductionRuntimeArtifact() {
    const element = document.getElementById(PRODUCTION_RUNTIME_ELEMENT_ID);
    if (element === null) return null;
    if (!(element instanceof HTMLScriptElement)) throw new ResumeBootError("ProductionArtifactMismatch");
    let artifact;
    try { artifact = JSON.parse(element.textContent ?? ""); } catch (_) { throw new ResumeBootError("ProductionArtifactMismatch"); }
    if (artifact === null || typeof artifact !== "object"
      || artifact.schemaVersion !== 1 || artifact.runtimeProtocolVersion !== 1
      || !Array.isArray(artifact.tables?.tables) || !Array.isArray(artifact.chunks)) {
      throw new ResumeBootError("ProductionArtifactMismatch");
    }
    return artifact;
  }

  function productionOrdinalIndexes(artifact) {
    if (artifact === null) return null;
    const indexes = new Map();
    for (const table of artifact.tables.tables) {
      if (typeof table.table_kind !== "string" || !Array.isArray(table.mappings)) {
        throw new ResumeBootError("ProductionArtifactMismatch");
      }
      const values = new Map();
      for (const mapping of table.mappings) {
        if (typeof mapping.canonical_id !== "string" || !Number.isInteger(mapping.ordinal)
          || mapping.ordinal < 0 || values.has(mapping.canonical_id)) {
          throw new ResumeBootError("ProductionArtifactMismatch");
        }
        values.set(mapping.canonical_id, mapping.ordinal);
      }
      indexes.set(table.table_kind, values);
    }
    for (const required of ["anchors", "events", "activations", "activation_roots"]) {
      if (!indexes.has(required)) throw new ResumeBootError("ProductionArtifactMismatch");
    }
    return indexes;
  }

  function readOptionalResumeSnapshot(diagnostics, explicitSnapshot) {
    if (explicitSnapshot !== undefined) return explicitSnapshot;
    if (window.__PRESOLVE_RESUME_SNAPSHOT__ !== undefined) {
      return window.__PRESOLVE_RESUME_SNAPSHOT__;
    }
    const element = document.getElementById(RESUME_SNAPSHOT_ELEMENT_ID);
    if (element === null) return null;
    if (!(element instanceof HTMLScriptElement)) {
      reportDiagnostic(diagnostics, "PSR_RESUME_SNAPSHOT_PARSE", "Resume snapshot was not stored in a JSON script element", { artifactElementId: RESUME_SNAPSHOT_ELEMENT_ID });
      throw new ResumeBootError("SnapshotParseFailure");
    }
    try {
      return JSON.parse(element.textContent ?? "");
    } catch (error) {
      reportDiagnostic(diagnostics, "PSR_RESUME_SNAPSHOT_PARSE", "Resume snapshot v1 could not be parsed", { message: error instanceof Error ? error.message : String(error) });
      throw new ResumeBootError("SnapshotParseFailure");
    }
  }

  function uniqueRecordIndex(records, key, failure = "DuplicateIdentity") {
    if (!Array.isArray(records)) throw new ResumeBootError("ResumeArtifactMismatch");
    const index = new Map();
    for (const record of records) {
      const id = record?.[key];
      if (typeof id !== "string" || index.has(id)) throw new ResumeBootError(failure, { id });
      index.set(id, record);
    }
    return index;
  }

  function validateResumeManifest(manifest) {
    const topLevelKeys = [
      "schema_version", "build_id", "snapshot_schema_version", "runtime_protocol_version",
      "application_root_boundary_id", "boundaries", "slot_schemas", "capture_programs",
      "restore_programs", "chunks", "activations", "anchors", "events",
      "phase_i_component_resume_records", "phase_i_form_resume_records"
    ];
    if (!exactObjectKeys(manifest, topLevelKeys)) throw new ResumeBootError("ResumeArtifactMismatch");
    if (manifest.schema_version !== SUPPORTED_RESUME_MANIFEST_SCHEMA_VERSION) {
      throw new ResumeBootError("ManifestVersionMismatch");
    }
    if (manifest.snapshot_schema_version !== SUPPORTED_RESUME_SNAPSHOT_SCHEMA_VERSION) {
      throw new ResumeBootError("SnapshotSchemaMismatch");
    }
    if (manifest.runtime_protocol_version !== SUPPORTED_RESUME_RUNTIME_PROTOCOL_VERSION) {
      throw new ResumeBootError("RuntimeProtocolMismatch");
    }
    if (typeof manifest.build_id !== "string" || !/^resume-build:[0-9a-f]{64}$/.test(manifest.build_id)) {
      throw new ResumeBootError("ResumeArtifactMismatch");
    }
    const boundaries = uniqueRecordIndex(manifest.boundaries, "boundary_id");
    const slots = uniqueRecordIndex(manifest.slot_schemas, "slot_id");
    const capturePrograms = uniqueRecordIndex(manifest.capture_programs, "program_id");
    const restorePrograms = uniqueRecordIndex(manifest.restore_programs, "program_id");
    const chunks = uniqueRecordIndex(manifest.chunks, "chunk_id");
    const activations = uniqueRecordIndex(manifest.activations, "activation_id");
    const anchors = uniqueRecordIndex(manifest.anchors, "anchor_id");
    const events = uniqueRecordIndex(manifest.events, "resume_event_id");
    if (!boundaries.has(manifest.application_root_boundary_id)) {
      throw new ResumeBootError("UnknownBoundary");
    }
    for (const boundary of boundaries.values()) {
      if (!capturePrograms.has(boundary.capture_program_id)
        || !restorePrograms.has(boundary.restore_program_id)
        || (boundary.parent_boundary_id !== undefined && !boundaries.has(boundary.parent_boundary_id))
        || !boundary.child_boundary_ids.every((id) => boundaries.has(id))
        || !boundary.anchor_ids.every((id) => anchors.has(id))
        || !boundary.event_ids.every((id) => events.has(id))) {
        throw new ResumeBootError("ResumeArtifactMismatch", { boundary: boundary.boundary_id });
      }
    }
    for (const slot of slots.values()) {
      if (!boundaries.has(slot.owner_boundary_id)) throw new ResumeBootError("UnknownBoundary");
    }
    for (const activation of activations.values()) {
      if (!boundaries.has(activation.boundary_id)
        || !chunks.has(activation.chunk_id)
        || !activation.prerequisite_boundary_ids.every((id) => boundaries.has(id))
        || (activation.event_id !== undefined && !events.has(activation.event_id))) {
        throw new ResumeBootError("ResumeArtifactMismatch", { activation: activation.activation_id });
      }
    }
    for (const anchor of anchors.values()) {
      if (!boundaries.has(anchor.boundary_id)) throw new ResumeBootError("UnknownBoundary");
    }
    for (const event of events.values()) {
      if (!anchors.has(event.exact_target_anchor_id)
        || !boundaries.has(event.owner_boundary_id)
        || !chunks.has(event.chunk_id)) {
        throw new ResumeBootError("ResumeArtifactMismatch", { event: event.resume_event_id });
      }
    }
    return { boundaries, slots, capturePrograms, restorePrograms, chunks, activations, anchors, events };
  }

  function validateResumeSnapshot(snapshot, manifest, definitions) {
    if (!exactObjectKeys(snapshot, ["schemaVersion", "buildId", "snapshotId", "manifestVersion", "capturedAt", "boundaries"])) {
      throw new ResumeBootError("SnapshotSchemaMismatch");
    }
    if (snapshot.schemaVersion !== SUPPORTED_RESUME_SNAPSHOT_SCHEMA_VERSION
      || snapshot.manifestVersion !== SUPPORTED_RESUME_MANIFEST_SCHEMA_VERSION
      || snapshot.capturedAt !== null
      || typeof snapshot.snapshotId !== "string") {
      throw new ResumeBootError("SnapshotSchemaMismatch");
    }
    if (snapshot.buildId !== manifest.build_id) throw new ResumeBootError("BuildIdMismatch");
    const seenBoundaries = new Set();
    const seenSlots = new Set();
    if (!Array.isArray(snapshot.boundaries)) throw new ResumeBootError("SnapshotSchemaMismatch");
    for (const boundary of snapshot.boundaries) {
      if (!exactObjectKeys(boundary, ["boundaryId", "schemaId", "values"])) {
        throw new ResumeBootError("SnapshotSchemaMismatch");
      }
      const definition = definitions.boundaries.get(boundary.boundaryId);
      if (definition === undefined) throw new ResumeBootError("UnknownBoundary");
      if (seenBoundaries.has(boundary.boundaryId)) throw new ResumeBootError("DuplicateIdentity");
      if (boundary.schemaId !== definition.schema_id || !Array.isArray(boundary.values)) {
        throw new ResumeBootError("ResumeArtifactMismatch");
      }
      seenBoundaries.add(boundary.boundaryId);
      for (const value of boundary.values) {
        if (!exactObjectKeys(value, ["valueRecordId", "slotId", "value"])) {
          throw new ResumeBootError("SnapshotSchemaMismatch");
        }
        const slot = definitions.slots.get(value.slotId);
        if (slot === undefined) throw new ResumeBootError("UnknownSlot");
        if (slot.owner_boundary_id !== boundary.boundaryId || seenSlots.has(value.slotId)) {
          throw new ResumeBootError("DuplicateIdentity");
        }
        seenSlots.add(value.slotId);
      }
    }
    return { seenBoundaries, seenSlots };
  }

  function allocateResumeRegistry(manifest, definitions) {
    return {
      contract_version: RESUME_REGISTRY_CONTRACT_VERSION,
      build_id: manifest.build_id,
      definitions,
      boundary_records: new Map(),
      slot_values: new Map(),
      context_bindings: new Map(),
      component_records: new Map(),
      form_records: new Map(),
      structural_records: new Map(),
      effect_subscriptions: new Map(),
      activation_states: new Map(),
      debug: []
    };
  }

  function resumeDebugEvidence(result) {
    return {
      contract_version: RESUME_REGISTRY_CONTRACT_VERSION,
      mode: result.mode,
      failure: result.failure ?? null,
      build_id: resumeBootstrapState.manifest?.build_id ?? null,
      boundary_ids: resumeBootstrapState.registry === null
        ? []
        : [...resumeBootstrapState.registry.definitions.boundaries.keys()],
      slot_ids: resumeBootstrapState.registry === null
        ? []
        : [...resumeBootstrapState.registry.definitions.slots.keys()]
    };
  }

  async function bootstrapResume(input = {}) {
    if (resumeBootstrapState.phase !== "idle") {
      throw new ResumeBootError("DoubleBootstrap");
    }
    resumeBootstrapState.phase = "booting";
    const diagnostics = input.diagnostics ?? [];
    let snapshot = null;
    let coldAttempted = false;
    try {
      const manifest = input.resumeManifest ?? readResumeManifest(diagnostics);
      const definitions = validateResumeManifest(manifest);
      snapshot = readOptionalResumeSnapshot(diagnostics, input.snapshot);
      resumeBootstrapState.manifest = manifest;
      if (snapshot === null) {
        coldAttempted = true;
        const cold = await input.coldBoot?.("NoSnapshot");
        const result = { mode: "cold", failure: null, cold };
        resumeBootstrapState.phase = "ready";
        resumeBootstrapState.result = result;
        resumeBootstrapState.debug = [resumeDebugEvidence(result)];
        return result;
      }
      validateResumeSnapshot(snapshot, manifest, definitions);
      resumeBootstrapState.registry = allocateResumeRegistry(manifest, definitions);
      const resume = await input.resumeBoot?.({
        manifest,
        snapshot,
        registry: resumeBootstrapState.registry
      });
      const result = {
        mode: "resume",
        failure: null,
        registry: resumeBootstrapState.registry,
        resume
      };
      resumeBootstrapState.phase = "ready";
      resumeBootstrapState.result = result;
      resumeBootstrapState.debug = [resumeDebugEvidence(result)];
      return result;
    } catch (error) {
      if (coldAttempted) {
        resumeBootstrapState.phase = "failed";
        throw error;
      }
      const failure = error instanceof ResumeBootError ? error.failure : "RestoreInstructionFailure";
      resumeBootstrapState.registry = null;
      resumeBootstrapState.debug = [{ contract_version: RESUME_REGISTRY_CONTRACT_VERSION, mode: "cold", failure }];
      try {
        coldAttempted = true;
        const cold = await input.coldBoot?.(failure);
        const result = { mode: "cold", failure, cold };
        resumeBootstrapState.phase = "ready";
        resumeBootstrapState.result = result;
        return result;
      } catch (coldError) {
        resumeBootstrapState.phase = "failed";
        throw coldError;
      }
    }
  }

  function captureSnapshot() {
    if (resumeBootstrapState.phase !== "ready" || resumeBootstrapState.manifest === null) {
      return { ok: false, failure: "NotQuiescent" };
    }
    return { ok: false, failure: "NotQuiescent" };
  }

  async function activateBoundary(boundaryId) {
    const registry = resumeBootstrapState.registry;
    if (registry === null || !registry.definitions.boundaries.has(boundaryId)) {
      throw new ResumeBootError("UnknownBoundary", { boundaryId });
    }
    const activation = [...registry.definitions.activations.values()]
      .find((record) => record.boundary_id === boundaryId);
    if (activation === undefined) throw new ResumeBootError("ActivationFailure", { boundaryId });
    const prior = registry.activation_states.get(activation.activation_id);
    if (prior?.status === "active") return prior.result;
    if (prior?.status === "failed") throw new ResumeBootError("ActivationFailure", { boundaryId });
    if (prior?.promise !== undefined) return prior.promise;
    const chunk = registry.definitions.chunks.get(activation.chunk_id);
    if (chunk === undefined || typeof chunk.module_path !== "string") {
      throw new ResumeBootError("ActivationFailure", { boundaryId });
    }
    const promise = import(new URL(chunk.module_path, document.baseURI).href)
      .then(() => {
        const result = { boundary_id: boundaryId, activation_id: activation.activation_id, chunk_id: activation.chunk_id, status: "active" };
        registry.activation_states.set(activation.activation_id, { status: "active", result });
        return result;
      })
      .catch(() => {
        registry.activation_states.set(activation.activation_id, { status: "failed" });
        throw new ResumeBootError("ActivationFailure", { boundaryId });
      });
    registry.activation_states.set(activation.activation_id, { status: "loading", promise });
    return promise;
  }

  async function activateByEvent(resumeEventId) {
    const registry = resumeBootstrapState.registry;
    if (registry === null || !registry.definitions.events.has(resumeEventId)) {
      throw new ResumeBootError("ActivationFailure", { resumeEventId });
    }
    const activation = [...registry.definitions.activations.values()]
      .find((record) => record.event_id === resumeEventId);
    if (activation === undefined) throw new ResumeBootError("ActivationFailure", { resumeEventId });
    return activateBoundary(activation.boundary_id);
  }

  function resumeEventMarker(event) {
    let current = event.target instanceof Element ? event.target : event.target?.parentElement;
    while (current !== null && current !== undefined) {
      const marker = current.getAttribute("data-presolve-e");
      if (marker !== null) return marker;
      current = current.parentElement;
    }
    return null;
  }

  function installResumeActivationListeners(registry, store) {
    const events = new Map([...registry.definitions.events.values()].map((event) => [event.resume_event_id, event]));
    const eventTypes = new Set([...events.values()].map((event) => event.event_type));
    for (const eventType of eventTypes) {
      document.addEventListener(eventType, async (event) => {
        const marker = resumeEventMarker(event);
        if (marker === null) return;
        const record = events.get(marker);
        if (record === undefined || record.event_type !== event.type) return;
        try {
          await activateByEvent(marker);
          dispatchOrdinaryInstanceEvent(store, event);
        } catch (error) {
          registry.debug.push({ event_id: marker, failure: error instanceof ResumeBootError ? error.failure : "ActivationFailure" });
        }
      });
    }
  }

  function readManifest(diagnostics) {
    const element = document.getElementById(MANIFEST_ELEMENT_ID);

    if (!(element instanceof HTMLScriptElement)) {
      reportDiagnostic(
        diagnostics,
        "PSR_MISSING_MANIFEST",
        `Missing template manifest script #${MANIFEST_ELEMENT_ID}`,
        { manifestElementId: MANIFEST_ELEMENT_ID },
        true
      );
      throw new PresolveBootError("PSR_MISSING_MANIFEST");
    }

    try {
      return JSON.parse(element.textContent ?? "");
    } catch (error) {
      reportDiagnostic(
        diagnostics,
        "PSR_INVALID_MANIFEST_JSON",
        "Template manifest JSON could not be parsed",
        { message: error instanceof Error ? error.message : String(error) },
        true
      );
      throw new PresolveBootError("PSR_INVALID_MANIFEST_JSON");
    }
  }

  function effectArtifactHasActionPlans(effectArtifact) {
    return (effectArtifact?.effects ?? []).some((effect) =>
      Array.isArray(effect.action_batch_triggers) && effect.action_batch_triggers.length > 0
    );
  }

  function validateManifestActionBindings(manifest, opaqueArtifact, diagnostics) {
    const opaqueMethods = new Set((opaqueArtifact?.activations ?? []).map((activation) => activation.method));
    for (const component of manifest.components ?? []) {
      const actionsByMethod = new Map();

      for (const action of component.actions ?? []) {
        if (
          typeof action.method_id !== "string"
          || typeof action.action_batch_id !== "string"
          || (manifest.schema_version === SUPPORTED_SCHEMA_VERSION && typeof action.storage_id !== "string")
        ) {
          reportDiagnostic(
            diagnostics,
            "PSR_INVALID_ACTION_BINDING",
            "Schema-v2 template action was missing compiler action identities",
            { component: component.name, action },
            true
          );
          throw new PresolveBootError("PSR_INVALID_ACTION_BINDING");
        }
        actionsByMethod.set(action.method_id, action.action_batch_id);
      }

      for (const event of component.template?.events ?? []) {
        if (event.kind !== "action") {
          reportDiagnostic(
            diagnostics,
            "PSR_INVALID_ACTION_BINDING",
            "Schema-v2 template event was not an explicit action binding",
            { component: component.name, event },
            true
          );
          throw new PresolveBootError("PSR_INVALID_ACTION_BINDING");
        }
        if (typeof event.method_id !== "string" || typeof event.action_batch_id !== "string") {
          reportDiagnostic(
            diagnostics,
            "PSR_INVALID_ACTION_BINDING",
            "Schema-v2 template action binding was missing an action batch identity",
            { component: component.name, event },
            true
          );
          throw new PresolveBootError("PSR_INVALID_ACTION_BINDING");
        }
        if (actionsByMethod.get(event.method_id) !== event.action_batch_id
          && !opaqueMethods.has(event.method_id)) {
          reportDiagnostic(
            diagnostics,
            "PSR_INVALID_ACTION_BINDING",
            "Template action binding did not match its compiler action implementation",
            { component: component.name, event },
            true
          );
          throw new PresolveBootError("PSR_INVALID_ACTION_BINDING");
        }
      }
    }
  }

  function validateManifestSchema(manifest, effectArtifact, componentArtifact, opaqueArtifact, diagnostics) {
    if (
      manifest?.schema_version !== SUPPORTED_SCHEMA_VERSION &&
      manifest?.schema_version !== FORMS_MANIFEST_SCHEMA_VERSION &&
      manifest?.schema_version !== ACTION_MANIFEST_SCHEMA_VERSION &&
      manifest?.schema_version !== LEGACY_MANIFEST_SCHEMA_VERSION
    ) {
      reportDiagnostic(
        diagnostics,
        "PSR_UNSUPPORTED_SCHEMA",
        `Unsupported template manifest schema version ${String(manifest?.schema_version)}`,
        {
          schema_version: manifest?.schema_version,
          supported_schema_version: SUPPORTED_SCHEMA_VERSION
        },
        true
      );
      throw new PresolveBootError("PSR_UNSUPPORTED_SCHEMA");
    }

    const isOrdinaryInstancePair = manifest.schema_version === SUPPORTED_SCHEMA_VERSION
      && componentArtifact?.schema_version === SUPPORTED_COMPONENT_ARTIFACT_SCHEMA_VERSION;
    const isLegacyColdPair = manifest.schema_version === FORMS_MANIFEST_SCHEMA_VERSION
      && componentArtifact?.schema_version === LEGACY_COMPONENT_ARTIFACT_SCHEMA_VERSION;
    if (!isOrdinaryInstancePair && !isLegacyColdPair) {
      reportDiagnostic(
        diagnostics,
        "PSR_UNSUPPORTED_SCHEMA",
        "Template manifest and component artifact are not an exact runtime contract pair",
        { manifest_schema_version: manifest.schema_version, component_schema_version: componentArtifact?.schema_version },
        true
      );
      throw new PresolveBootError("PSR_UNSUPPORTED_SCHEMA");
    }

    if (isOrdinaryInstancePair) {
      for (const program of componentArtifact.structural_programs ?? []) {
        const component = (manifest.components ?? []).find(
          (candidate) => candidate.component_id === program.host_component
        );
        const host = component?.template?.nodes?.find((node) => node.id === program.host_node);
        if (component === undefined || host === undefined
          || (host.kind !== "conditional" && host.kind !== "list")) {
          reportDiagnostic(
            diagnostics,
            "PSR_INVALID_COMPONENT_ARTIFACT",
            "Structural component metadata did not resolve to an emitted conditional or keyed-list host",
            { region: program.region, host_component: program.host_component, host_node: program.host_node },
            true
          );
          throw new PresolveBootError("PSR_INVALID_COMPONENT_ARTIFACT");
        }
        if (host.kind === "list" && program.conditional_host_fragments.length !== 0) {
          reportDiagnostic(
            diagnostics,
            "PSR_INVALID_COMPONENT_ARTIFACT",
            "Conditional host fragments were attached to a keyed-list host",
            { region: program.region, host_component: program.host_component, host_node: program.host_node },
            true
          );
          throw new PresolveBootError("PSR_INVALID_COMPONENT_ARTIFACT");
        }
        if (host.kind === "conditional" && program.keyed_host_fragments.length !== 0) {
          reportDiagnostic(
            diagnostics,
            "PSR_INVALID_COMPONENT_ARTIFACT",
            "Keyed host fragments were attached to a conditional host",
            { region: program.region, host_component: program.host_component, host_node: program.host_node },
            true
          );
          throw new PresolveBootError("PSR_INVALID_COMPONENT_ARTIFACT");
        }
      }
    }

    if (
      manifest.schema_version === LEGACY_MANIFEST_SCHEMA_VERSION &&
      effectArtifactHasActionPlans(effectArtifact)
    ) {
      reportDiagnostic(
        diagnostics,
        "PSR_LEGACY_MANIFEST_EFFECT_ACTIONS",
        "A legacy template manifest cannot activate compiler-generated effect action batches",
        { schema_version: manifest.schema_version },
        true
      );
      throw new PresolveBootError("PSR_LEGACY_MANIFEST_EFFECT_ACTIONS");
    }

    if (manifest.schema_version >= ACTION_MANIFEST_SCHEMA_VERSION) {
      validateManifestActionBindings(manifest, opaqueArtifact, diagnostics);
    }
  }

  function readFormsArtifact(diagnostics) {
    const element = document.getElementById(FORMS_ARTIFACT_ELEMENT_ID);
    if (element === null) return null;
    if (!(element instanceof HTMLScriptElement)) {
      reportDiagnostic(diagnostics, "PSR_INVALID_FORMS_ARTIFACT", "Forms runtime metadata was not stored in a script element", { artifactElementId: FORMS_ARTIFACT_ELEMENT_ID }, true);
      throw new PresolveBootError("PSR_INVALID_FORMS_ARTIFACT");
    }
    try { return JSON.parse(element.textContent ?? ""); } catch (error) {
      reportDiagnostic(diagnostics, "PSR_INVALID_FORMS_ARTIFACT", "Forms runtime metadata JSON could not be parsed", { message: error instanceof Error ? error.message : String(error) }, true);
      throw new PresolveBootError("PSR_INVALID_FORMS_ARTIFACT");
    }
  }

  function readResourcesArtifact(diagnostics) {
    const element = document.getElementById(RESOURCES_ARTIFACT_ELEMENT_ID);
    if (element === null) return null;
    if (!(element instanceof HTMLScriptElement)) {
      reportDiagnostic(diagnostics, "PSR_INVALID_RESOURCES_ARTIFACT", "Resource runtime metadata was not stored in a script element", { artifactElementId: RESOURCES_ARTIFACT_ELEMENT_ID }, true);
      throw new PresolveBootError("PSR_INVALID_RESOURCES_ARTIFACT");
    }
    try { return JSON.parse(element.textContent ?? ""); } catch (error) {
      reportDiagnostic(diagnostics, "PSR_INVALID_RESOURCES_ARTIFACT", "Resource runtime metadata JSON could not be parsed", { message: error instanceof Error ? error.message : String(error) }, true);
      throw new PresolveBootError("PSR_INVALID_RESOURCES_ARTIFACT");
    }
  }

  function readOpaqueArtifact(diagnostics) {
    const element = document.getElementById(OPAQUE_ARTIFACT_ELEMENT_ID);
    if (element === null) return null;
    if (!(element instanceof HTMLScriptElement)) {
      reportDiagnostic(diagnostics, "PSR_INVALID_OPAQUE_ARTIFACT", "Opaque terminal metadata was not stored in a script element", { artifactElementId: OPAQUE_ARTIFACT_ELEMENT_ID }, true);
      throw new PresolveBootError("PSR_INVALID_OPAQUE_ARTIFACT");
    }
    try { return JSON.parse(element.textContent ?? ""); } catch (error) {
      reportDiagnostic(diagnostics, "PSR_INVALID_OPAQUE_ARTIFACT", "Opaque terminal metadata JSON could not be parsed", { message: error instanceof Error ? error.message : String(error) }, true);
      throw new PresolveBootError("PSR_INVALID_OPAQUE_ARTIFACT");
    }
  }

  function validateOpaqueArtifact(opaqueArtifact, diagnostics) {
    if (opaqueArtifact === null) return;
    if (opaqueArtifact.schema_version !== SUPPORTED_OPAQUE_ARTIFACT_SCHEMA_VERSION
      || !Array.isArray(opaqueArtifact.activations)) {
      reportDiagnostic(diagnostics, "PSR_UNSUPPORTED_OPAQUE_ARTIFACT_SCHEMA", "Opaque terminal metadata did not match the compiler artifact contract", { schema_version: opaqueArtifact.schema_version }, true);
      throw new PresolveBootError("PSR_UNSUPPORTED_OPAQUE_ARTIFACT_SCHEMA");
    }
    const ids = new Set();
    const methods = new Set();
    for (const activation of opaqueArtifact.activations) {
      if (typeof activation?.id !== "string" || ids.has(activation.id)
        || typeof activation?.method !== "string" || methods.has(activation.method)
        || typeof activation?.owner_component !== "string"
        || typeof activation?.package !== "string" || typeof activation?.version !== "string"
        || typeof activation?.integrity !== "string" || typeof activation?.export !== "string"
        || typeof activation?.runtime_module !== "string" || typeof activation?.runtime_location !== "string"
        || activation?.type_signature !== "() -> void"
        || activation?.execution_boundary !== "client" || activation?.resume_policy !== "cold_fallback") {
        reportDiagnostic(diagnostics, "PSR_INVALID_OPAQUE_ARTIFACT", "Opaque terminal activation did not retain one exact callable client boundary", { activation }, true);
        throw new PresolveBootError("PSR_INVALID_OPAQUE_ARTIFACT");
      }
      ids.add(activation.id);
      methods.add(activation.method);
    }
  }

  function validateResourcesArtifact(resourcesArtifact, diagnostics) {
    if (resourcesArtifact === null) return;
    if (resourcesArtifact.schema_version !== SUPPORTED_RESOURCES_ARTIFACT_SCHEMA_VERSION
      || !Array.isArray(resourcesArtifact.declarations)
      || !Array.isArray(resourcesArtifact.activations)) {
      reportDiagnostic(diagnostics, "PSR_UNSUPPORTED_RESOURCES_ARTIFACT_SCHEMA", "Resource runtime metadata did not match the compiler artifact contract", { schema_version: resourcesArtifact.schema_version }, true);
      throw new PresolveBootError("PSR_UNSUPPORTED_RESOURCES_ARTIFACT_SCHEMA");
    }
    const declarations = new Set();
    for (const declaration of resourcesArtifact.declarations) {
      const endpoint = declaration?.endpoint;
      if (typeof declaration?.id !== "string" || declarations.has(declaration.id)
        || typeof endpoint?.package !== "string" || typeof endpoint?.version !== "string"
        || typeof endpoint?.integrity !== "string" || typeof endpoint?.export !== "string"
        || typeof endpoint?.runtime_module !== "string" || typeof endpoint?.runtime_location !== "string") {
        reportDiagnostic(diagnostics, "PSR_INVALID_RESOURCES_ARTIFACT", "Resource declaration did not retain one exact executable endpoint", { declaration }, true);
        throw new PresolveBootError("PSR_INVALID_RESOURCES_ARTIFACT");
      }
      declarations.add(declaration.id);
    }
    const activations = new Set();
    for (const activation of resourcesArtifact.activations) {
      const generationRequired = ["pending", "ready", "failed", "cancelled"].includes(activation?.state);
      if (typeof activation?.id !== "string" || activations.has(activation.id)
        || !declarations.has(activation?.declaration)
        || generationRequired !== Number.isInteger(activation?.generation)) {
        reportDiagnostic(diagnostics, "PSR_INVALID_RESOURCES_ARTIFACT", "Resource activation did not retain canonical lifecycle linkage", { activation }, true);
        throw new PresolveBootError("PSR_INVALID_RESOURCES_ARTIFACT");
      }
      activations.add(activation.id);
    }
  }

  function validateFormsArtifact(formsArtifact, manifest, diagnostics) {
    if (formsArtifact === null) {
      if (manifest.schema_version >= 3) {
        reportDiagnostic(diagnostics, "PSR_MISSING_FORMS_ARTIFACT", "A schema-v3 template manifest requires Forms runtime metadata", {}, true);
        throw new PresolveBootError("PSR_MISSING_FORMS_ARTIFACT");
      }
      return;
    }
    if (formsArtifact.schema_version !== SUPPORTED_FORMS_ARTIFACT_SCHEMA_VERSION || !Array.isArray(formsArtifact.forms) || !Array.isArray(formsArtifact.instances) || !Array.isArray(formsArtifact.hosts)) {
      reportDiagnostic(diagnostics, "PSR_UNSUPPORTED_FORMS_ARTIFACT_SCHEMA", "Forms runtime metadata did not match the compiler artifact contract", { schema_version: formsArtifact.schema_version }, true);
      throw new PresolveBootError("PSR_UNSUPPORTED_FORMS_ARTIFACT_SCHEMA");
    }
    for (const form of formsArtifact.forms) {
      const paths = new Set();
      for (const field of form?.fields ?? []) {
        if (!Array.isArray(field?.path) || field.path.length === 0 || field.path.length > 16
          || !field.path.every((segment) => typeof segment === "string" && /^[A-Za-z_][A-Za-z0-9_]*$/.test(segment))) {
          reportDiagnostic(diagnostics, "PSR_INVALID_FORMS_ARTIFACT", "Form Field path was not compiler-canonical", { field }, true);
          throw new PresolveBootError("PSR_INVALID_FORMS_ARTIFACT");
        }
        const path = field.path.join(".");
        if ([...paths].some((other) => path === other || path.startsWith(`${other}.`) || other.startsWith(`${path}.`))) {
          reportDiagnostic(diagnostics, "PSR_INVALID_FORMS_ARTIFACT", "Form Field paths conflicted", { form: form.id, path }, true);
          throw new PresolveBootError("PSR_INVALID_FORMS_ARTIFACT");
        }
        paths.add(path);
      }
    }
    const hasForms = formsArtifact.forms.length > 0 || formsArtifact.instances.length > 0;
    if (hasForms && manifest.schema_version < 3) {
      reportDiagnostic(diagnostics, "PSR_FORMS_MANIFEST_MISMATCH", "Forms runtime metadata requires a schema-v3 template manifest", { schema_version: manifest.schema_version }, true);
      throw new PresolveBootError("PSR_FORMS_MANIFEST_MISMATCH");
    }
    const instances = new Set(formsArtifact.instances.map((instance) => instance.id));
    for (const binding of manifest.form_bindings ?? []) {
      if (!instances.has(binding.form_instance_id)) {
        reportDiagnostic(diagnostics, "PSR_FORMS_MANIFEST_MISMATCH", "Forms manifest bridge referenced an unknown Form instance", { binding }, true);
        throw new PresolveBootError("PSR_FORMS_MANIFEST_MISMATCH");
      }
    }
    const artifactHosts = new Map(formsArtifact.hosts.map((host) => [`${host.host_anchor}|${host.form_instance}`, host]));
    for (const host of manifest.form_hosts ?? []) {
      const artifact = artifactHosts.get(`${host.host_anchor}|${host.form_instance_id}`);
      if (artifact === undefined || artifact.id !== host.submission_host_id || artifact.event !== host.event || artifact.submit_action !== host.submit_action || artifact.action_batch !== host.action_batch || artifact.serialization_plan !== host.serialization_plan || artifact.prevent_default !== host.prevent_default) {
        reportDiagnostic(diagnostics, "PSR_FORMS_MANIFEST_MISMATCH", "Forms manifest host bridge did not match an exact compiler host record", { host }, true);
        throw new PresolveBootError("PSR_FORMS_MANIFEST_MISMATCH");
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
        "PSR_INVALID_COMPUTED_ARTIFACT",
        "Computed runtime metadata was not stored in a script element",
        { artifactElementId: COMPUTED_ARTIFACT_ELEMENT_ID },
        true
      );
      throw new PresolveBootError("PSR_INVALID_COMPUTED_ARTIFACT");
    }

    try {
      return JSON.parse(element.textContent ?? "");
    } catch (error) {
      reportDiagnostic(
        diagnostics,
        "PSR_INVALID_COMPUTED_ARTIFACT",
        "Computed runtime metadata JSON could not be parsed",
        { message: error instanceof Error ? error.message : String(error) },
        true
      );
      throw new PresolveBootError("PSR_INVALID_COMPUTED_ARTIFACT");
    }
  }

  function validateComputedArtifactSchema(artifact, diagnostics) {
    if (artifact === null) {
      return;
    }

    if (artifact.schema_version !== SUPPORTED_COMPUTED_ARTIFACT_SCHEMA_VERSION) {
      reportDiagnostic(
        diagnostics,
        "PSR_UNSUPPORTED_COMPUTED_ARTIFACT_SCHEMA",
        `Unsupported computed runtime metadata schema version ${String(artifact.schema_version)}`,
        {
          schema_version: artifact.schema_version,
          supported_schema_version: SUPPORTED_COMPUTED_ARTIFACT_SCHEMA_VERSION
        },
        true
      );
      throw new PresolveBootError("PSR_UNSUPPORTED_COMPUTED_ARTIFACT_SCHEMA");
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
        "PSR_INVALID_EFFECT_ARTIFACT",
        "Effect runtime metadata was not stored in a script element",
        { artifactElementId: EFFECT_ARTIFACT_ELEMENT_ID },
        true
      );
      throw new PresolveBootError("PSR_INVALID_EFFECT_ARTIFACT");
    }

    try {
      return JSON.parse(element.textContent ?? "");
    } catch (error) {
      reportDiagnostic(
        diagnostics,
        "PSR_INVALID_EFFECT_ARTIFACT",
        "Effect runtime metadata JSON could not be parsed",
        { message: error instanceof Error ? error.message : String(error) },
        true
      );
      throw new PresolveBootError("PSR_INVALID_EFFECT_ARTIFACT");
    }
  }

  function validateEffectArtifactSchema(artifact, diagnostics) {
    if (artifact === null) {
      return;
    }

    if (artifact.schema_version !== SUPPORTED_EFFECT_ARTIFACT_SCHEMA_VERSION) {
      reportDiagnostic(
        diagnostics,
        "PSR_UNSUPPORTED_EFFECT_ARTIFACT_SCHEMA",
        `Unsupported effect runtime metadata schema version ${String(artifact.schema_version)}`,
        {
          schema_version: artifact.schema_version,
          supported_schema_version: SUPPORTED_EFFECT_ARTIFACT_SCHEMA_VERSION
        },
        true
      );
      throw new PresolveBootError("PSR_UNSUPPORTED_EFFECT_ARTIFACT_SCHEMA");
    }
  }

  function validateEffectArtifactInstances(effectArtifact, componentArtifact, diagnostics) {
    if (effectArtifact === null) return;
    if (!Array.isArray(effectArtifact.instances) || !Array.isArray(effectArtifact.structural_templates)) {
      reportDiagnostic(diagnostics, "PSR_INVALID_EFFECT_INSTANCE_ARTIFACT", "Effect runtime metadata omitted instance ownership records", {}, true);
      throw new PresolveBootError("PSR_INVALID_EFFECT_INSTANCE_ARTIFACT");
    }
    if (effectArtifact.instances.length === 0) return;
    if (componentArtifact === null) {
      reportDiagnostic(diagnostics, "PSR_INVALID_EFFECT_INSTANCE_ARTIFACT", "Instance-owned effects require a component runtime artifact", {}, true);
      throw new PresolveBootError("PSR_INVALID_EFFECT_INSTANCE_ARTIFACT");
    }
    const declarations = new Set((effectArtifact.effects ?? []).map((effect) => effect.effect));
    const instances = new Map((componentArtifact.instances ?? []).map((instance) => [instance.instance, instance]));
    const identities = new Set();
    for (const record of effectArtifact.instances) {
      const component = instances.get(record.component_instance);
      if (typeof record.effect_instance !== "string" || identities.has(record.effect_instance)
        || !declarations.has(record.effect) || component === undefined
        || record.parent_instance !== component.parent || record.depth !== component.depth
        || !Number.isInteger(record.declaration_order) || record.declaration_order < 0) {
        reportDiagnostic(diagnostics, "PSR_INVALID_EFFECT_INSTANCE_ARTIFACT", "Effect instance ownership did not match compiler component metadata", { effect_instance: record.effect_instance }, true);
        throw new PresolveBootError("PSR_INVALID_EFFECT_INSTANCE_ARTIFACT");
      }
      identities.add(record.effect_instance);
    }
    const regions = new Map((componentArtifact.structural_programs ?? []).map((program) => [program.region, program]));
    const templateIds = new Set([...regions.values()].flatMap((program) => program.template_instances ?? []));
    const templateIdentities = new Set();
    for (const record of effectArtifact.structural_templates) {
      const region = regions.get(record.structural_region);
      if (typeof record.template_instance !== "string" || templateIdentities.has(record.template_instance)
        || !templateIds.has(record.template_instance) || region === undefined
        || !declarations.has(record.effect) || !Number.isInteger(record.depth) || record.depth < 0
        || !Number.isInteger(record.declaration_order) || record.declaration_order < 0) {
        reportDiagnostic(diagnostics, "PSR_INVALID_EFFECT_INSTANCE_ARTIFACT", "Structural effect template metadata did not match compiler component metadata", { template_instance: record.template_instance }, true);
        throw new PresolveBootError("PSR_INVALID_EFFECT_INSTANCE_ARTIFACT");
      }
      templateIdentities.add(record.template_instance);
    }
  }

  function readComponentArtifact(diagnostics) {
    const element = document.getElementById(COMPONENT_ARTIFACT_ELEMENT_ID);
    if (element === null) return null;
    if (!(element instanceof HTMLScriptElement)) {
      reportDiagnostic(diagnostics, "PSR_INVALID_COMPONENT_ARTIFACT", "Component runtime metadata was not stored in a script element", { artifactElementId: COMPONENT_ARTIFACT_ELEMENT_ID }, true);
      throw new PresolveBootError("PSR_INVALID_COMPONENT_ARTIFACT");
    }
    try { return JSON.parse(element.textContent ?? ""); } catch (error) {
      reportDiagnostic(diagnostics, "PSR_INVALID_COMPONENT_ARTIFACT", "Component runtime metadata JSON could not be parsed", { message: error instanceof Error ? error.message : String(error) }, true);
      throw new PresolveBootError("PSR_INVALID_COMPONENT_ARTIFACT");
    }
  }

  function canonicalStateSlotId(componentInstanceId, storageId) {
    const encoded = [...new TextEncoder().encode(storageId)].map((byte) => {
      const unreserved = (byte >= 65 && byte <= 90)
        || (byte >= 97 && byte <= 122)
        || (byte >= 48 && byte <= 57)
        || byte === 45 || byte === 46 || byte === 95 || byte === 126;
      return unreserved ? String.fromCharCode(byte) : `%${byte.toString(16).toUpperCase().padStart(2, "0")}`;
    }).join("");
    return `${componentInstanceId}/state-slot:${encoded}`;
  }

  function validateComponentArtifactSchema(artifact, diagnostics) {
    if (artifact === null) return;
    if ((artifact.schema_version !== SUPPORTED_COMPONENT_ARTIFACT_SCHEMA_VERSION && artifact.schema_version !== LEGACY_COMPONENT_ARTIFACT_SCHEMA_VERSION) || !Array.isArray(artifact.instances) || !Array.isArray(artifact.initialization_batches)) {
      reportDiagnostic(diagnostics, "PSR_INVALID_COMPONENT_ARTIFACT", "Component runtime metadata did not match the compiler artifact contract", { schema_version: artifact.schema_version }, true);
      throw new PresolveBootError("PSR_INVALID_COMPONENT_ARTIFACT");
    }
    const instances = new Set(artifact.instances.map((instance) => instance.instance));
    const instancesById = new Map(artifact.instances.map((instance) => [instance.instance, instance]));
    const structuralTemplates = new Set(
      (artifact.structural_programs ?? [])
        .flatMap((program) => program.template_instances ?? [])
    );
    const structuralTemplateComponents = new Map(
      (artifact.structural_programs ?? [])
        .flatMap((program) => (program.template_occurrences ?? []).map((occurrence) => [
          occurrence.template_instance,
          occurrence.component
        ]))
    );
    const stateSlots = new Set();
    const statePairs = new Set();
    for (const instance of artifact.instances) if (
      instance.parent !== null
      && instance.parent !== undefined
      && !instances.has(instance.parent)
      && !structuralTemplates.has(instance.parent)
    ) {
      reportDiagnostic(diagnostics, "PSR_INVALID_COMPONENT_ARTIFACT", "Component runtime metadata referenced an unknown parent instance", { instance: instance.instance, parent: instance.parent }, true);
      throw new PresolveBootError("PSR_INVALID_COMPONENT_ARTIFACT");
    }
    if (artifact.schema_version === SUPPORTED_COMPONENT_ARTIFACT_SCHEMA_VERSION) {
      if (!Array.isArray(artifact.structural_programs)) {
        throw new PresolveBootError("PSR_INVALID_COMPONENT_ARTIFACT");
      }
      if (!Array.isArray(artifact.ordinary_template_targets)
        || !Array.isArray(artifact.ordinary_template_bindings)
        || !Array.isArray(artifact.ordinary_template_events)) {
        throw new PresolveBootError("PSR_INVALID_COMPONENT_ARTIFACT");
      }
      const structuralRegions = new Set();
      const structuralHosts = new Set();
      for (const program of artifact.structural_programs) {
        const host = `${String(program.host_component)}\u001f${String(program.host_node)}`;
        if (typeof program.region !== "string" || program.region.length === 0
          || typeof program.host_component !== "string" || program.host_component.length === 0
          || typeof program.host_node !== "string" || program.host_node.length === 0
          || typeof program.host_template_entity !== "string" || program.host_template_entity.length === 0
          || structuralRegions.has(program.region) || structuralHosts.has(host)
          || !Array.isArray(program.conditional_host_fragments)
          || !Array.isArray(program.keyed_host_fragments)
          || !Array.isArray(program.template_instances)
          || !Array.isArray(program.template_occurrences)
          || program.template_occurrences.length !== program.template_instances.length
          || new Set(program.conditional_host_fragments.map((fragments) => fragments?.host_instance)).size !== program.conditional_host_fragments.length
          || program.conditional_host_fragments.some((fragments) => typeof fragments?.host_instance !== "string"
            || typeof fragments.host_scope !== "string"
            || (fragments.host_scope === "static-instance"
              ? instancesById.get(fragments.host_instance)?.component !== program.host_component
              : fragments.host_scope === "structural-occurrence"
                ? structuralTemplateComponents.get(fragments.host_instance) !== program.host_component
                : true)
            || typeof fragments.when_true_html !== "string" || fragments.when_true_html.length === 0
            || typeof fragments.when_false_html !== "string" || fragments.when_false_html.length === 0)
          || new Set(program.keyed_host_fragments.map((fragments) => fragments?.host_instance)).size !== program.keyed_host_fragments.length
          || program.keyed_host_fragments.some((fragments) => typeof fragments?.host_instance !== "string"
            || typeof fragments.host_scope !== "string"
            || (fragments.host_scope === "static-instance"
              ? instancesById.get(fragments.host_instance)?.component !== program.host_component
              : fragments.host_scope === "structural-occurrence"
                ? structuralTemplateComponents.get(fragments.host_instance) !== program.host_component
                : true)
            || typeof fragments.item_template_html !== "string" || fragments.item_template_html.length === 0)
          || program.template_occurrences.some((occurrence, index) => typeof occurrence?.template_instance !== "string"
            || occurrence.template_instance !== program.template_instances[index]
            || typeof occurrence.invocation !== "string" || typeof occurrence.invocation_template_entity !== "string"
            || occurrence.invocation_template_entity.length === 0 || typeof occurrence.component !== "string"
            || typeof occurrence.template_html !== "string" || occurrence.template_html.length === 0
            || !Array.isArray(occurrence.state_slots)
            || !Array.isArray(occurrence.computed_slots)
            || !Array.isArray(occurrence.ordinary_template_targets)
            || !Array.isArray(occurrence.ordinary_template_bindings)
            || !Array.isArray(occurrence.ordinary_template_events)
            || JSON.stringify(occurrence.ordinary_template_targets) !== JSON.stringify(
              artifact.ordinary_template_targets
                .filter((target) => target.component_instance_id === occurrence.template_instance)
                .map((target) => target.id)
            )
            || JSON.stringify(occurrence.ordinary_template_bindings) !== JSON.stringify(
              artifact.ordinary_template_bindings
                .filter((binding) => binding.component_instance_id === occurrence.template_instance)
                .map((binding) => binding.id)
            )
            || JSON.stringify(occurrence.ordinary_template_events) !== JSON.stringify(
              artifact.ordinary_template_events
                .filter((event) => event.component_instance_id === occurrence.template_instance)
                .map((event) => event.declaration_event_id)
            ))
          || JSON.stringify(program.create_order) !== JSON.stringify(program.template_instances)
          || JSON.stringify([...(program.destroy_order ?? [])].reverse()) !== JSON.stringify(program.template_instances)) {
          throw new PresolveBootError("PSR_INVALID_COMPONENT_ARTIFACT");
        }
        structuralRegions.add(program.region);
        structuralHosts.add(host);
      }
      for (const instance of artifact.instances) {
        if (!Array.isArray(instance.state_slots)) throw new PresolveBootError("PSR_INVALID_COMPONENT_ARTIFACT");
        for (const slot of instance.state_slots) {
          const pair = `${instance.instance}|${slot.storage_id}`;
          if (
            typeof slot.slot_id !== "string"
            || slot.slot_id !== canonicalStateSlotId(instance.instance, slot.storage_id)
            || typeof slot.state_id !== "string"
            || typeof slot.storage_id !== "string"
            || slot.storage_id !== `storage:${slot.state_id}`
            || stateSlots.has(slot.slot_id)
            || statePairs.has(pair)
          ) {
            throw new PresolveBootError("PSR_INVALID_COMPONENT_ARTIFACT");
          }
          stateSlots.add(slot.slot_id);
          statePairs.add(pair);
        }
      }
    }
  }

  function readContextArtifact(diagnostics) {
    const element = document.getElementById(CONTEXT_ARTIFACT_ELEMENT_ID);
    if (!(element instanceof HTMLScriptElement)) {
      reportDiagnostic(diagnostics, "PSR_INVALID_CONTEXT_ARTIFACT", "Context runtime metadata was not stored in a script element", { artifactElementId: CONTEXT_ARTIFACT_ELEMENT_ID }, true);
      throw new PresolveBootError("PSR_INVALID_CONTEXT_ARTIFACT");
    }
    try { return JSON.parse(element.textContent ?? ""); }
    catch (error) {
      reportDiagnostic(diagnostics, "PSR_INVALID_CONTEXT_ARTIFACT", "Context runtime metadata JSON could not be parsed", { message: error instanceof Error ? error.message : String(error) }, true);
      throw new PresolveBootError("PSR_INVALID_CONTEXT_ARTIFACT");
    }
  }

  function validateContextArtifactSchema(artifact, diagnostics) {
    if (artifact.schema_version !== SUPPORTED_CONTEXT_ARTIFACT_SCHEMA_VERSION) {
      reportDiagnostic(diagnostics, "PSR_UNSUPPORTED_CONTEXT_ARTIFACT_SCHEMA", `Unsupported Context runtime metadata schema version ${String(artifact.schema_version)}`, { schema_version: artifact.schema_version, supported_schema_version: SUPPORTED_CONTEXT_ARTIFACT_SCHEMA_VERSION }, true);
      throw new PresolveBootError("PSR_UNSUPPORTED_CONTEXT_ARTIFACT_SCHEMA");
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
      const match = /^presolve-binding:([^:]+):(.*)$/.exec(value);

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
      const startMatch = /^presolve-conditional-start:([^:]+):(.*)$/.exec(value);

      if (startMatch !== null) {
        starts.set(startMatch[1], {
          id: startMatch[1],
          condition: startMatch[2],
          marker: walker.currentNode
        });
        continue;
      }

      const endMatch = /^presolve-conditional-end:([^:]+)$/.exec(value);

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
      const startMatch = /^presolve-list-start:([^:]+):(.*)$/.exec(value);

      if (startMatch !== null) {
        starts.set(startMatch[1], {
          id: startMatch[1],
          iterable: startMatch[2],
          marker: walker.currentNode
        });
        continue;
      }

      const endMatch = /^presolve-list-end:([^:]+)$/.exec(value);

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

    for (const element of document.querySelectorAll("[data-presolve-node]")) {
      elementsByNode.set(element.dataset.presolveNode, element);
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
              code: "PSR_MISSING_ELEMENT_ANCHOR"
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
            code: "PSR_MISSING_BINDING_ANCHOR"
          });
        }

        if (node.kind === "conditional") {
          if (!conditionalAnchors.starts.has(node.start)) {
            missing.push({
              component: component.name,
              id: node.start,
              kind: node.kind,
              code: "PSR_MISSING_CONDITIONAL_ANCHOR"
            });
          }

          if (!conditionalAnchors.ends.has(node.end)) {
            missing.push({
              component: component.name,
              id: node.end,
              kind: node.kind,
              code: "PSR_MISSING_CONDITIONAL_ANCHOR"
            });
          }
        }

        if (node.kind === "list") {
          if (!listAnchors.starts.has(node.start)) {
            missing.push({
              component: component.name,
              id: node.start,
              kind: node.kind,
              code: "PSR_MISSING_LIST_ANCHOR"
            });
          }

          if (!listAnchors.ends.has(node.end)) {
            missing.push({
              component: component.name,
              id: node.end,
              kind: node.kind,
              code: "PSR_MISSING_LIST_ANCHOR"
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
        "PSR_MISSING_CONDITIONAL_ANCHOR",
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

  const STRUCTURAL_OCCURRENCE_IDENTITY_PREFIX = "presolve-structural-occurrence:v1:";

  function structuralOccurrenceHex(value) {
    if (typeof value !== "string" || value.length === 0) {
      throw new PresolveBootError("PSR_INVALID_COMPONENT_ARTIFACT");
    }
    return [...new TextEncoder().encode(value)]
      .map((byte) => byte.toString(16).toUpperCase().padStart(2, "0"))
      .join("");
  }

  function structuralOccurrenceIdentity(parentScope, region, templateInstance, localOccurrence) {
    return `${STRUCTURAL_OCCURRENCE_IDENTITY_PREFIX}${structuralOccurrenceHex(parentScope)}.${structuralOccurrenceHex(region)}.${structuralOccurrenceHex(templateInstance)}.${structuralOccurrenceHex(localOccurrence)}`;
  }

  function decodeStructuralOccurrenceIdentity(value) {
    if (typeof value !== "string" || !value.startsWith(STRUCTURAL_OCCURRENCE_IDENTITY_PREFIX)) {
      throw new PresolveBootError("PSR_INVALID_COMPONENT_ARTIFACT");
    }
    const fields = value.slice(STRUCTURAL_OCCURRENCE_IDENTITY_PREFIX.length).split(".");
    if (fields.length !== 4 || fields.some((field) => field.length === 0 || field.length % 2 !== 0 || !/^[0-9a-f]+$/i.test(field))) {
      throw new PresolveBootError("PSR_INVALID_COMPONENT_ARTIFACT");
    }
    let decoded;
    try {
      decoded = fields.map((field) => new TextDecoder("utf-8", { fatal: true }).decode(Uint8Array.from(
        field.match(/../g).map((pair) => Number.parseInt(pair, 16))
      )));
    } catch (_) {
      throw new PresolveBootError("PSR_INVALID_COMPONENT_ARTIFACT");
    }
    if (decoded.some((field) => field.length === 0)) {
      throw new PresolveBootError("PSR_INVALID_COMPONENT_ARTIFACT");
    }
    return Object.freeze({
      parent_scope: decoded[0],
      region: decoded[1],
      template_instance: decoded[2],
      local_occurrence: decoded[3]
    });
  }

  function instantiateStructuralTemplateSlots(occurrence, occurrenceIdentity) {
    decodeStructuralOccurrenceIdentity(occurrenceIdentity);
    const templateInstance = String(occurrence?.template_instance ?? "");
    const prefix = `${templateInstance}/`;
    if (templateInstance.length === 0
      || !Array.isArray(occurrence?.state_slots)
      || !Array.isArray(occurrence?.computed_slots)) {
      throw new PresolveBootError("PSR_INVALID_COMPONENT_ARTIFACT");
    }
    const ids = new Set();
    const stateSlots = occurrence.state_slots.map((slot) => {
      if (typeof slot?.slot_id !== "string" || !slot.slot_id.startsWith(`${prefix}state-slot:`)
        || typeof slot.storage_id !== "string" || !slot.storage_id.startsWith("storage:")) {
        throw new PresolveBootError("PSR_INVALID_COMPONENT_ARTIFACT");
      }
      const slotId = `${occurrenceIdentity}/${slot.slot_id.slice(prefix.length)}`;
      if (ids.has(slotId)) throw new PresolveBootError("PSR_INVALID_COMPONENT_ARTIFACT");
      ids.add(slotId);
      return Object.freeze({ ...slot, slot_id: slotId });
    });
    const computedSlots = occurrence.computed_slots.map((slot) => {
      if (typeof slot?.cache_slot_id !== "string" || !slot.cache_slot_id.startsWith(`${prefix}computed-cache:`)
        || typeof slot.dirty_slot_id !== "string" || !slot.dirty_slot_id.startsWith(`${prefix}computed-dirty:`)) {
        throw new PresolveBootError("PSR_INVALID_COMPONENT_ARTIFACT");
      }
      const cacheSlotId = `${occurrenceIdentity}/${slot.cache_slot_id.slice(prefix.length)}`;
      const dirtySlotId = `${occurrenceIdentity}/${slot.dirty_slot_id.slice(prefix.length)}`;
      if (ids.has(cacheSlotId) || ids.has(dirtySlotId)) throw new PresolveBootError("PSR_INVALID_COMPONENT_ARTIFACT");
      ids.add(cacheSlotId);
      ids.add(dirtySlotId);
      return Object.freeze({ ...slot, cache_slot_id: cacheSlotId, dirty_slot_id: dirtySlotId });
    });
    return Object.freeze({ state_slots: stateSlots, computed_slots: computedSlots });
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
        String(textNode.nodeValue ?? "").startsWith("presolve-list-binding-end:")
      ) {
        textNode.before(document.createTextNode(text));
      }
    }
  }

  function listItemElements(root) {
    return [root, ...root.querySelectorAll("[data-presolve-list-bindings]")];
  }

  function updateListItemAttributes(node, instance) {
    for (const element of listItemElements(instance.element)) {
      const bindings = String(element.dataset.presolveListBindings ?? "");

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
    return [root, ...root.querySelectorAll("[data-presolve-on-click]")];
  }

  function registerListItemEvents(store, component, instance) {
    for (const element of listItemEventElements(instance.element)) {
      const node = element.dataset.presolveNode;
      const handler = element.dataset.presolveOnClick;

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
      const node = element.dataset.presolveNode;

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
        "PSR_MISSING_LIST_ANCHOR",
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
          "PSR_DUPLICATE_LIST_KEY",
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
            "PSR_INVALID_LIST_TEMPLATE",
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

  function createRuntimeStore(elementsByNode, diagnostics, computedArtifact, contextArtifact, effectArtifact, componentArtifact, opaqueArtifact) {
    const computedEvaluations = new Map();
    const storageValues = new Map();
    const storageByComponentField = new Map();
    const instanceQualifiedState = componentArtifact?.schema_version === SUPPORTED_COMPONENT_ARTIFACT_SCHEMA_VERSION;
    const invalidationsByStorage = new Map();
    const resourceInvalidationsByDeclaration = new Map();
    const computedDirty = new Map();

    if (!instanceQualifiedState) {
      for (const state of computedArtifact?.state ?? []) {
        storageValues.set(state.storage, state.initial_value);
        storageByComponentField.set(
          componentFieldKey(state.component, state.field),
          state.storage
        );
      }
    }

    for (const invalidation of computedArtifact?.invalidations ?? []) {
      invalidationsByStorage.set(invalidation.storage, invalidation.dependents ?? []);
    }

    for (const invalidation of computedArtifact?.resource_invalidations ?? []) {
      resourceInvalidationsByDeclaration.set(invalidation.declaration, invalidation.dependents ?? []);
    }

    for (const evaluation of computedArtifact?.evaluations ?? []) {
      computedEvaluations.set(evaluation.computed, evaluation);
      computedDirty.set(evaluation.computed, evaluation.dirty_flag?.initial_value === true);
    }

    return {
      components: new Map(),
      bindingsByField: new Map(),
      bindingsByStateSlot: new Map(),
      bindingsByInstanceComputed: new Map(),
      actionsByMethod: new Map(),
      opaqueTerminalsByMethod: new Map((opaqueArtifact?.activations ?? []).map((activation) => [activation.method, activation])),
      opaqueActivations: [],
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
      stateSlotsByInstanceStorage: new Map(),
      instanceQualifiedState,
      invalidationsByStorage,
      resourceInvalidationsByDeclaration,
      computedUpdateRuns: 0,
      initialEffectRuns: [],
      completedActionEffectRuns: [],
      effectCleanupRuns: [],
      effectSubscriptions: new Map(),
      effectCleanups: new Map(),
      resources: new Map(),
      resourceActivationsByInstanceDeclaration: new Map(),
      activeActionBatch: null
    };
  }

  function computedSlotForExecution(store, computed) {
    const componentInstanceId = store.activeExecutionContext?.component_instance_id;
    if (componentInstanceId === undefined) return undefined;
    const slot = store.computedSlotsByInstanceComputed.get(`${componentInstanceId}|${computed}`);
    if (slot === undefined) throw new PresolveBootError("PSR_INVALID_COMPONENT_ARTIFACT");
    return slot;
  }

  function stateSlotForInstanceStorage(store, componentInstanceId, storage) {
    const slot = store.stateSlotsByInstanceStorage.get(`${componentInstanceId}|${storage}`);
    if (slot === undefined) throw new PresolveBootError("PSR_INVALID_COMPONENT_ARTIFACT");
    return slot;
  }

  function stateValueForStorage(store, storage) {
    if (!store.instanceQualifiedState) return store.storageValues.get(storage);
    const componentInstanceId = store.activeExecutionContext?.component_instance_id;
    if (componentInstanceId === undefined) throw new PresolveBootError("PSR_MISSING_EXECUTION_CONTEXT");
    return store.storageValues.get(
      stateSlotForInstanceStorage(store, componentInstanceId, storage).slot_id
    );
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

  function computedValueForInstance(store, componentInstanceId, computed) {
    const slot = store.computedSlotsByInstanceComputed.get(`${componentInstanceId}|${computed}`);
    if (slot === undefined) throw new PresolveBootError("PSR_INVALID_COMPONENT_ARTIFACT");
    return store.computedCaches.get(slot.cache_slot_id);
  }

  function notifyComputed(store, computed, value) {
    const componentInstanceId = store.activeExecutionContext?.component_instance_id;
    if (componentInstanceId === undefined) return;
    for (const updateBinding of store.bindingsByInstanceComputed.get(`${componentInstanceId}|${computed}`) ?? []) {
      updateBinding(value);
    }
  }

  function registerComputedBinding(store, componentInstanceId, computed, updateBinding) {
    const key = `${componentInstanceId}|${computed}`;
    const bindings = store.bindingsByInstanceComputed.get(key) ?? [];
    bindings.push(updateBinding);
    store.bindingsByInstanceComputed.set(key, bindings);
  }

  function storeComputedValue(store, evaluation, value) {
    const slot = computedSlotForExecution(store, evaluation.computed);
    if (slot === undefined) {
      store.computedValues.set(evaluation.computed, value);
      store.computedCaches.set(evaluation.cache_slot, value);
    } else {
      store.computedCaches.set(slot.cache_slot_id, value);
    }
    notifyComputed(store, evaluation.computed, value);
  }

  function computedOperandValue(store, values, operand) {
    if (operand?.kind === "value") {
      return values.get(operand.value);
    }

    if (operand?.kind === "constant") {
      return operand.value;
    }

    if (operand?.kind === "storage") {
      return stateValueForStorage(store, operand.storage);
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
      case "min": return typeof left === "number" && typeof right === "number" ? Math.min(left, right) : undefined;
      case "max": return typeof left === "number" && typeof right === "number" ? Math.max(left, right) : undefined;
      default: return undefined;
    }
  }

  function computedUnary(operation, value) {
    switch (operation) {
      case "not": return !value;
      case "identity": return +value;
      case "negate": return -value;
      case "abs": return typeof value === "number" ? Math.abs(value) : undefined;
      case "floor": return typeof value === "number" ? Math.floor(value) : undefined;
      case "ceil": return typeof value === "number" ? Math.ceil(value) : undefined;
      case "round": return typeof value === "number" ? Math.round(value) : undefined;
      default: return undefined;
    }
  }

  function executePureProgramInstruction(store, values, instruction, subject) {
    if (instruction.kind === "constant") {
      values.set(instruction.result, instruction.value);
      return true;
    }

    if (instruction.kind === "load-state") {
      values.set(instruction.result, stateValueForStorage(store, instruction.storage));
      return true;
    }

    if (instruction.kind === "load-computed") {
      if (isComputedDirty(store, instruction.computed)) {
        reportDiagnostic(
          store.diagnostics,
          "PSR_UNPLANNED_COMPUTED_DEPENDENCY",
          "Compiler program depended on a value not yet evaluated by the compiler plan",
          { subject, dependency: instruction.computed }
        );
        values.set(instruction.result, undefined);
      } else {
        values.set(instruction.result, computedValue(store, instruction.computed));
      }
      return true;
    }

    if (instruction.kind === "load-resource") {
      const componentInstanceId = store.activeExecutionContext?.component_instance_id;
      const activationId = typeof componentInstanceId === "string"
        ? store.resourceActivationsByInstanceDeclaration.get(`${componentInstanceId}\u001f${instruction.declaration}`)
        : undefined;
      const resource = activationId === undefined ? undefined : store.resources.get(activationId);
      if (resource === undefined) {
        reportDiagnostic(
          store.diagnostics,
          "PSR_RESOURCE_ACTIVATION_MISSING",
          "Computed Resource projection lacked exactly one compiler-selected activation",
          { subject, component_instance_id: componentInstanceId, declaration: instruction.declaration },
          true
        );
        throw new PresolveBootError("PSR_RESOURCE_ACTIVATION_MISSING");
      }
      values.set(instruction.result, { data: resource.data, error: resource.error, state: resource.state });
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

    if (instruction.kind === "get-index") {
      const object = computedOperandValue(store, values, instruction.object);
      const index = computedOperandValue(store, values, instruction.index);
      const key = typeof index === "string" || (typeof index === "number" && Number.isInteger(index) && index >= 0)
        ? String(index)
        : null;
      const value = key !== null && object !== null && typeof object === "object"
        && Object.prototype.hasOwnProperty.call(object, key)
        ? object[key]
        : undefined;
      values.set(instruction.result, value);
      return true;
    }

    if (instruction.kind === "select") {
      const condition = computedOperandValue(store, values, instruction.condition);
      const value = condition === true
        ? computedOperandValue(store, values, instruction.when_true)
        : computedOperandValue(store, values, instruction.when_false);
      values.set(instruction.result, value);
      return true;
    }

    if (instruction.kind === "template") {
      const quasis = instruction.quasis ?? [];
      const expressions = instruction.expressions ?? [];
      if (quasis.length !== expressions.length + 1) {
        reportDiagnostic(
          store.diagnostics,
          "PSR_INVALID_TEMPLATE_PROGRAM",
          "Compiler artifact contained an invalid template interpolation program",
          { subject }
        );
        values.set(instruction.result, undefined);
      } else {
        let value = quasis[0];
        for (let index = 0; index < expressions.length; index += 1) {
          value += String(values.get(expressions[index]));
          value += quasis[index + 1];
        }
        values.set(instruction.result, value);
      }
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

    if (instruction.kind === "pure-package-call") {
      if (instruction.operation === "identity" && instruction.arguments?.length === 1) {
        values.set(instruction.result, values.get(instruction.arguments[0]));
      } else {
        reportDiagnostic(
          store.diagnostics,
          "PSR_INVALID_PURE_PACKAGE_OPERATION",
          "Compiler artifact contained an unsupported pure package operation",
          { subject, package: instruction.package, export: instruction.export, operation: instruction.operation }
        );
        values.set(instruction.result, undefined);
      }
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

    return [...batches.entries()]
      .sort(([left], [right]) => left - right)
      .map(([batchIndex, effects]) => [
        batchIndex,
        effects.sort((left, right) =>
          (left.declaration_order ?? Number.MAX_SAFE_INTEGER)
            - (right.declaration_order ?? Number.MAX_SAFE_INTEGER)
          || left.effect.localeCompare(right.effect)
        )
      ]);
  }

  function dispatchEffectCapability(store, effect, instruction, values, evidence) {
    const runtimeLowering = instruction.runtime_lowering;
    const capabilityArguments = (instruction.arguments ?? []).map((operand) =>
      computedOperandValue(store, values, operand)
    );
    const value = computedOperandValue(store, values, instruction.value);

    switch (runtimeLowering) {
      case "builtin.browser.document.title.assign":
        document.title = value;
        break;
      case "builtin.browser.console.log":
        console.log(...capabilityArguments);
        break;
      case "builtin.browser.console.info":
        console.info(...capabilityArguments);
        break;
      case "builtin.browser.console.warn":
        console.warn(...capabilityArguments);
        break;
      case "builtin.browser.console.error":
        console.error(...capabilityArguments);
        break;
      case "builtin.browser.local_storage.set_item":
        localStorage.setItem(...capabilityArguments);
        break;
      case "builtin.browser.local_storage.remove_item":
        localStorage.removeItem(...capabilityArguments);
        break;
      case "builtin.browser.session_storage.set_item":
        sessionStorage.setItem(...capabilityArguments);
        break;
      case "builtin.browser.session_storage.remove_item":
        sessionStorage.removeItem(...capabilityArguments);
        break;
      default:
        reportDiagnostic(
          store.diagnostics,
          "PSR_UNSUPPORTED_EFFECT_CAPABILITY",
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

  function effectInstanceTargets(effectArtifact, effect, actionInstanceId = null) {
    const records = (effectArtifact?.instances ?? [])
      .filter((record) => record.effect === effect.effect)
      .filter((record) => actionInstanceId === null || record.component_instance === actionInstanceId)
      .sort((left, right) => left.depth - right.depth
        || left.declaration_order - right.declaration_order
        || left.effect_instance.localeCompare(right.effect_instance));
    return records.length === 0 ? [null] : records;
  }

  function executeEffectProgram(store, effect, evidence, program = effect.program) {
    const values = new Map();

    for (const instruction of program?.instructions ?? []) {
      if (executePureProgramInstruction(store, values, instruction, effect.effect)) {
        continue;
      }

      if (instruction.kind === "capability-call" || instruction.kind === "capability-assign") {
        dispatchEffectCapability(store, effect, instruction, values, evidence);
        continue;
      }

      reportDiagnostic(
        store.diagnostics,
        "PSR_UNSUPPORTED_EFFECT_INSTRUCTION",
        "Effect program contained an unsupported compiler instruction",
        { effect: effect.effect, kind: instruction.kind }
      );
    }
  }

  function runEffect(store, effect, evidence, instance = null) {
    const key = instance?.effect_instance ?? effect.effect;
    const priorExecutionContext = store.activeExecutionContext;
    if (instance !== null) store.activeExecutionContext = { component_instance_id: instance.component_instance };
    try {
      const cleanup = store.effectCleanups.get(key);
      if (cleanup !== undefined) {
        const cleanupEvidence = { capability_operations: [] };
        executeEffectProgram(store, effect, cleanupEvidence, cleanup);
        evidence.cleanup_capability_operations = cleanupEvidence.capability_operations;
        store.effectCleanups.delete(key);
      }
      executeEffectProgram(store, effect, evidence);
      if (effect.cleanup_program !== null && effect.cleanup_program !== undefined) {
        store.effectCleanups.set(key, effect.cleanup_program);
      }
    } finally {
      store.activeExecutionContext = priorExecutionContext;
    }
  }

  function disposeEffectInstances(store) {
    const effects = new Map((store.effectArtifact?.effects ?? []).map((effect) => [effect.effect, effect]));
    const instances = [...(store.effectArtifact?.instances ?? [])]
      .sort((left, right) => right.depth - left.depth
        || right.declaration_order - left.declaration_order
        || right.effect_instance.localeCompare(left.effect_instance));
    for (const instance of instances) {
      const cleanup = store.effectCleanups.get(instance.effect_instance);
      const effect = effects.get(instance.effect);
      if (cleanup === undefined || effect === undefined) continue;
      const priorExecutionContext = store.activeExecutionContext;
      store.activeExecutionContext = { component_instance_id: instance.component_instance };
      try {
        const evidence = {
          effect: effect.effect,
          effect_instance: instance.effect_instance,
          capability_operations: []
        };
        executeEffectProgram(store, effect, evidence, cleanup);
        store.effectCleanupRuns.push(evidence);
        store.effectCleanups.delete(instance.effect_instance);
      } finally {
        store.activeExecutionContext = priorExecutionContext;
      }
    }
  }

  function installEffectDisposal(store) {
    window.addEventListener("pagehide", () => disposeEffectInstances(store), { once: true });
  }

  function executeInitialEffects(store) {
    for (const [effectBatchIndex, effects] of initialEffectBatches(store.effectArtifact)) {
      for (const effect of effects) {
        for (const instance of effectInstanceTargets(store.effectArtifact, effect)) {
          const evidence = {
            effect: effect.effect,
            effect_instance: instance?.effect_instance,
            effect_batch_index: effectBatchIndex,
            capability_operations: []
          };
          runEffect(store, effect, evidence, instance);
          store.initialEffectRuns.push(evidence);
        }
      }
    }
  }

  // Resume is an explicit V2 lifecycle phase. It intentionally excludes the
  // legacy decorator effects whose restore boundary is frozen by the existing
  // resumability contract.
  function executeResumeEffects(store) {
    for (const [effectBatchIndex, effects] of initialEffectBatches(store.effectArtifact)) {
      for (const effect of effects) {
        if (effect.run_on_resume !== true) continue;
        for (const instance of effectInstanceTargets(store.effectArtifact, effect)) {
          const evidence = {
            effect: effect.effect,
            effect_instance: instance?.effect_instance,
            effect_batch_index: effectBatchIndex,
            capability_operations: []
          };
          runEffect(store, effect, evidence, instance);
          store.initialEffectRuns.push(evidence);
        }
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

    return [...batches.entries()]
      .sort(([left], [right]) => left - right)
      .map(([batchIndex, effects]) => [
        batchIndex,
        effects.sort((left, right) =>
          (left.declaration_order ?? Number.MAX_SAFE_INTEGER)
            - (right.declaration_order ?? Number.MAX_SAFE_INTEGER)
          || left.effect.localeCompare(right.effect)
        )
      ]);
  }

  function executeCompletedActionEffects(store, actionBatchId) {
    for (const [effectBatchIndex, effects] of actionEffectBatches(
      store.effectArtifact,
      actionBatchId
    )) {
      for (const effect of effects) {
        const actionInstanceId = store.activeExecutionContext?.component_instance_id ?? null;
        for (const instance of effectInstanceTargets(store.effectArtifact, effect, actionInstanceId)) {
          const evidence = {
            action_batch_id: actionBatchId,
            effect: effect.effect,
            effect_instance: instance?.effect_instance,
            effect_batch_index: effectBatchIndex,
            capability_operations: []
          };
          runEffect(store, effect, evidence, instance);
          store.completedActionEffectRuns.push(evidence);
        }
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
            "PSR_UNPLANNED_COMPUTED_DEPENDENCY",
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
            "PSR_UNPLANNED_COMPUTED_DEPENDENCY",
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

  function invalidateResourceComputeds(store, activation) {
    const priorExecutionContext = store.activeExecutionContext;
    store.activeExecutionContext = { component_instance_id: activation.component_instance };
    try {
      for (const computed of store.resourceInvalidationsByDeclaration.get(activation.declaration) ?? []) {
        setComputedDirty(store, computed, true);
      }
      executeComputedUpdateBatches(store);
    } finally {
      store.activeExecutionContext = priorExecutionContext;
    }
  }

  function readField(store, component, field, storageId = null) {
    if (store.instanceQualifiedState) {
      if (typeof component.instance_id !== "string" || typeof storageId !== "string") {
        throw new PresolveBootError("PSR_INVALID_STATE_OPERATION");
      }
      return store.storageValues.get(
        stateSlotForInstanceStorage(store, component.instance_id, storageId).slot_id
      );
    }
    if (!(field in component.state)) {
      reportDiagnostic(
        store.diagnostics,
        "PSR_INVALID_STATE_OPERATION",
        "Action referenced a missing state field",
        { component: component.name, field }
      );
      return undefined;
    }

    return component.state[field];
  }

  function writeField(store, component, field, value, storageId = null) {
    if (store.instanceQualifiedState) {
      if (typeof component.instance_id !== "string" || typeof storageId !== "string") {
        throw new PresolveBootError("PSR_INVALID_STATE_OPERATION");
      }
      const slot = stateSlotForInstanceStorage(store, component.instance_id, storageId);
      store.storageValues.set(slot.slot_id, value);
      component.state[field] = value;
      for (const computed of store.invalidationsByStorage.get(storageId) ?? []) {
        setComputedDirty(store, computed, true);
      }
      notifyField(store, component, field, slot.slot_id);
      return;
    }
    if (!(field in component.state)) {
      reportDiagnostic(
        store.diagnostics,
        "PSR_INVALID_STATE_OPERATION",
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
    notifyField(store, component, field, null);
  }

  function notifyField(store, component, field, stateSlotId) {
    const bindings = stateSlotId === null
      ? store.bindingsByField.get(componentFieldKey(component.name, field))
      : store.bindingsByStateSlot.get(stateSlotId);

    if (bindings === undefined) {
      reportDiagnostic(
        store.diagnostics,
        "PSR_MISSING_BINDING_ANCHOR",
        "State field has no registered binding anchor",
        { component: component.name, field }
      );
      return;
    }

    for (const updateBinding of bindings) {
      updateBinding(stateSlotId === null ? component.state[field] : store.storageValues.get(stateSlotId));
    }
  }

  function registerBinding(store, component, field, updateBinding, storageId = null) {
    if (store.instanceQualifiedState) {
      if (typeof component.instance_id !== "string" || typeof storageId !== "string") {
        throw new PresolveBootError("PSR_INVALID_ORDINARY_BINDING");
      }
      const slot = stateSlotForInstanceStorage(store, component.instance_id, storageId);
      const bindings = store.bindingsByStateSlot.get(slot.slot_id) ?? [];
      bindings.push(updateBinding);
      store.bindingsByStateSlot.set(slot.slot_id, bindings);
      return;
    }
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
        "PSR_UNRESOLVED_EVENT",
        "Unsupported event type in template manifest",
        event
      );
      return;
    }

    const actionRecord = store.actionsByMethod.get(event.method_id)
      ?? (store.opaqueTerminalsByMethod.has(event.method_id)
        ? { action_batch_id: event.action_batch_id, actions: [] }
        : undefined);

    if (
      actionRecord === undefined ||
      actionRecord.action_batch_id !== event.action_batch_id
    ) {
      reportDiagnostic(
        store.diagnostics,
        "PSR_UNRESOLVED_ACTION",
        "Event handler did not resolve to a compiler action",
        event
      );
      return;
    }

    const eventsByNode = store.eventsByType.get(event.event) ?? new Map();

    if (eventsByNode.has(event.node)) {
      reportDiagnostic(
        store.diagnostics,
        "PSR_UNRESOLVED_EVENT",
        "Duplicate event registration for template node",
        event
      );
      return;
    }

    eventsByNode.set(event.node, {
      component,
      method_id: event.method_id,
      action_batch_id: event.action_batch_id,
      actions: actionRecord.actions,
      arguments: Array.isArray(event.arguments) ? event.arguments : []
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
          "PSR_MISSING_BINDING_ANCHOR",
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
          "PSR_INVALID_STATE_OPERATION",
          "Numeric state operation had a non-numeric operand",
          action
        );
        return null;
      }

      return action.operation === "add_assign" ? operand : -operand;
    }

    return null;
  }

  function executeAction(store, component, action, executionContext = null) {
    if (
      action.operation !== "increment" &&
      action.operation !== "decrement" &&
      action.operation !== "add_assign" &&
      action.operation !== "subtract_assign" &&
      action.operation !== "assign" &&
      action.operation !== "assign_parameter" &&
      action.operation !== "toggle"
    ) {
      reportDiagnostic(
        store.diagnostics,
        "PSR_INVALID_STATE_OPERATION",
        "Action used an unsupported state operation",
        action
      );
      return;
    }

    if (action.operation === "toggle") {
      const current = readField(store, component, action.field, action.storage_id);

      if (typeof current !== "boolean") {
        reportDiagnostic(
          store.diagnostics,
          "PSR_INVALID_STATE_OPERATION",
          "Toggle action requires a boolean state field",
          action
        );
        return;
      }

      writeField(store, component, action.field, !current, action.storage_id);
      return;
    }

    if (action.operation === "assign") {
      writeField(store, component, action.field, action.operand, action.storage_id);
      return;
    }

    if (action.operation === "assign_parameter") {
      const parameterIndex = Number(action.operand);
      const value = executionContext?.arguments?.[parameterIndex];
      if (value === undefined || !Number.isInteger(parameterIndex)) {
        reportDiagnostic(store.diagnostics, "PSR_INVALID_STATE_OPERATION", "Action parameter assignment had no compiler-projected argument", action);
        return;
      }
      writeField(store, component, action.field, value, action.storage_id);
      return;
    }

    const current = Number(readField(store, component, action.field, action.storage_id));

    if (Number.isNaN(current)) {
      reportDiagnostic(
        store.diagnostics,
        "PSR_INVALID_STATE_OPERATION",
        "Numeric state operation requires a numeric state field",
        action
      );
      return;
    }

    const delta = actionDelta(store, action);

    if (delta === null) {
      return;
    }

    writeField(store, component, action.field, current + delta, action.storage_id);
  }

  function executeActions(store, component, actionBatchId, actions, executionContext = null) {
    store.activeActionBatch = actionBatchId;
    store.activeExecutionContext = executionContext;

    try {
      for (const action of actions) {
        executeAction(store, component, action, executionContext);
      }

      executeComputedUpdateBatches(store);
      executeContextUpdates(store, actionBatchId);
      executeCompletedActionEffects(store, actionBatchId);
      const methodId = executionContext?.method_id;
      if (typeof methodId === "string") executeOpaqueTerminal(store, methodId);
    } finally {
      store.activeActionBatch = null;
      store.activeExecutionContext = null;
      refreshComputedDebugState(store);
    }
  }

  function executeOpaqueTerminal(store, methodId) {
    const terminal = store.opaqueTerminalsByMethod.get(methodId);
    if (terminal === undefined) return;
    const evidence = { activation: terminal.id, method: methodId, status: "loading" };
    store.opaqueActivations.push(evidence);
    import(new URL(terminal.runtime_location, document.baseURI).href)
      .then((module) => {
        const callable = module?.[terminal.export];
        if (typeof callable !== "function") throw new Error("declared export is not callable");
        return callable();
      })
      .then(() => { evidence.status = "complete"; })
      .catch((error) => {
        evidence.status = "failed";
        reportDiagnostic(store.diagnostics, "PSR_OPAQUE_TERMINAL_FAILURE", "A compiler-authorized opaque terminal failed", { activation: terminal.id, message: error instanceof Error ? error.message : String(error) });
      });
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
      const nodeId = current.dataset?.presolveNode;

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

    executeActions(store, record.component, record.action_batch_id, record.actions, { arguments: record.arguments, method_id: record.method_id });
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
    const register = (id, target) => {
      if (targets.has(id)) duplicates.add(id);
      targets.set(id, target);
    };
    for (const element of document.querySelectorAll("[data-presolve-ti]")) {
      const id = element.getAttribute("data-presolve-ti");
      if (id === null) continue;
      register(id, element);
    }
    const conditionalStarts = new Map();
    const listStarts = new Map();
    const walker = document.createTreeWalker(document, NodeFilter.SHOW_COMMENT);
    while (walker.nextNode()) {
      const marker = walker.currentNode;
      const value = String(marker.nodeValue ?? "");
      const conditionalStart = /^presolve-conditional-start:[^:]+:ti:(.+)$/.exec(value);
      if (conditionalStart !== null) {
        conditionalStarts.set(conditionalStart[1], marker);
        continue;
      }
      const conditionalEnd = /^presolve-conditional-end:[^:]+:ti:(.+)$/.exec(value);
      if (conditionalEnd !== null) {
        const start = conditionalStarts.get(conditionalEnd[1]);
        if (start !== undefined) {
          register(conditionalEnd[1], { kind: "conditional", start, end: marker });
          conditionalStarts.delete(conditionalEnd[1]);
        }
        continue;
      }
      const listStart = /^presolve-ti-target-start:(.+)$/.exec(value);
      if (listStart !== null) {
        listStarts.set(listStart[1], marker);
        continue;
      }
      const listEnd = /^presolve-ti-target-end:(.+)$/.exec(value);
      if (listEnd !== null) {
        const start = listStarts.get(listEnd[1]);
        if (start !== undefined) {
          register(listEnd[1], { kind: "list", start, end: marker });
          listStarts.delete(listEnd[1]);
        }
      }
    }
    return { targets, duplicates };
  }

  function ordinaryEventKey(targetId, eventType) {
    return `${targetId}\u001f${eventType}`;
  }

  function ordinaryTextBindingNode(bindingId) {
    const walker = document.createTreeWalker(document, NodeFilter.SHOW_COMMENT);
    const start = `presolve-ti-binding-start:${bindingId}`;
    const end = `presolve-ti-binding-end:${bindingId}`;
    let startMarker = null;
    while (walker.nextNode()) {
      if (walker.currentNode.data === start) { startMarker = walker.currentNode; continue; }
      if (startMarker !== null && walker.currentNode.data === end) {
        const text = startMarker.nextSibling;
        if (text instanceof Text) return text;
        if (text === walker.currentNode && startMarker.parentNode !== null) {
          const empty = document.createTextNode("");
          startMarker.parentNode.insertBefore(empty, walker.currentNode);
          return empty;
        }
        return null;
      }
    }
    return null;
  }

  function registerOrdinaryBinding(store, binding, artifactBinding) {
    const component = store.components.get(binding.component_instance_id);
    const field = fieldNameFromThisMember(binding.expression);
    const storageId = artifactBinding.state_storage_ids?.length === 1
      ? artifactBinding.state_storage_ids[0]
      : null;
    const computedId = artifactBinding.computed_ids?.length === 1
      ? artifactBinding.computed_ids[0]
      : null;
    if (component === undefined || field === null || (storageId === null) === (computedId === null)) {
      throw new PresolveBootError("PSR_INVALID_ORDINARY_BINDING");
    }
    const target = store.templateTargetsById.get(binding.instance_target_id);
    const slot = storageId === null
      ? null
      : stateSlotForInstanceStorage(store, binding.component_instance_id, storageId);
    let update = null;
    if (binding.kind === "text") {
      const text = ordinaryTextBindingNode(binding.instance_binding_id);
      if (text !== null) update = (value) => { text.data = formatBindingValue(value); };
    } else if ((binding.kind === "attribute" || binding.kind === "property") && target instanceof Element && typeof binding.attribute_name === "string") {
      update = (value) => { updateAttributeBinding(target, binding.attribute_name, value); };
    } else if (binding.kind === "conditional" && target?.kind === "conditional") {
      const nodes = (component.manifest.template?.nodes ?? []).filter(
        (node) => node.kind === "conditional" && node.condition === binding.expression
      );
      if (nodes.length === 1) {
        const node = nodes[0];
        update = (value) => {
          replaceConditionalBranch(
            store,
            target.start,
            target.end,
            value === true ? node.when_true_html : node.when_false_html
          );
        };
      }
    } else if (binding.kind === "list" && target?.kind === "list") {
      if (slot === null) throw new PresolveBootError("PSR_INVALID_ORDINARY_BINDING");
      const nodes = (component.manifest.template?.nodes ?? []).filter(
        (node) => node.kind === "list" && node.iterable === binding.expression
      );
      if (nodes.length === 1) {
        const node = nodes[0];
        let instances = initialListInstances(
          store,
          node,
          listItems(store.storageValues.get(slot.slot_id))
        );
        for (const instance of instances.values()) {
          registerListItemEvents(store, component, instance);
        }
        update = (value) => {
          instances = reconcileKeyedList(
            store,
            component,
            node,
            target.start,
            target.end,
            instances,
            value
          );
        };
      }
    }
    if (update === null) throw new PresolveBootError("PSR_INVALID_ORDINARY_BINDING");
    if (slot !== null) {
      update(store.storageValues.get(slot.slot_id));
      registerBinding(store, component, field, update, storageId);
    } else {
      update(computedValueForInstance(store, binding.component_instance_id, computedId));
      registerComputedBinding(store, binding.component_instance_id, computedId, update);
    }
  }

  function initializeOrdinaryInstanceRuntime(store, manifest, componentArtifact) {
    if (manifest.schema_version !== SUPPORTED_SCHEMA_VERSION) return;
    const anchors = collectOrdinaryTargetAnchors();
    for (const binding of manifest.ordinary_bindings ?? []) {
      if (binding.kind !== "text" || anchors.targets.has(binding.instance_target_id)) continue;
      const text = ordinaryTextBindingNode(binding.instance_binding_id);
      if (text !== null) anchors.targets.set(binding.instance_target_id, text);
    }
    const artifactTargets = new Map((componentArtifact.ordinary_template_targets ?? []).map((target) => [target.id, target]));
    const artifactBindings = new Map((componentArtifact.ordinary_template_bindings ?? []).map((binding) => [binding.id, binding]));
    const artifactEvents = new Map((componentArtifact.ordinary_template_events ?? []).map((event) => [ordinaryEventKey(event.target_id, event.event_type), event]));
    store.templateTargetsById = anchors.targets;
    store.ordinaryBindingsById = new Map();
    store.ordinaryEventsByTargetAndType = new Map();
    for (const target of manifest.ordinary_targets ?? []) {
      const artifactTarget = artifactTargets.get(target.id);
      if (artifactTarget === undefined || artifactTarget.component_instance_id !== target.component_instance_id || anchors.duplicates.has(target.id) || !anchors.targets.has(target.id)) {
        throw new PresolveBootError("PSR_INVALID_ORDINARY_TARGET");
      }
    }
    for (const binding of manifest.ordinary_bindings ?? []) {
      const artifactBinding = artifactBindings.get(binding.instance_binding_id);
      if (artifactBinding === undefined || artifactBinding.component_instance_id !== binding.component_instance_id || artifactBinding.target_id !== binding.instance_target_id) {
        throw new PresolveBootError("PSR_INVALID_ORDINARY_BINDING");
      }
      store.ordinaryBindingsById.set(binding.instance_binding_id, {
        ...binding,
        execution_context: { component_instance_id: binding.component_instance_id }
      });
      registerOrdinaryBinding(store, binding, artifactBinding);
    }
    for (const event of manifest.ordinary_events ?? []) {
      const key = ordinaryEventKey(event.instance_target_id, event.event_type);
      const artifactEvent = artifactEvents.get(key);
      if (
        artifactEvent === undefined
        || artifactEvent.component_instance_id !== event.component_instance_id
        || JSON.stringify(artifactEvent.arguments ?? []) !== JSON.stringify(event.arguments ?? [])
        || store.ordinaryEventsByTargetAndType.has(key)
      ) {
        throw new PresolveBootError("PSR_INVALID_ORDINARY_EVENT");
      }
      store.ordinaryEventsByTargetAndType.set(key, event);
    }
  }

  function installResumeDomBindings(store, manifest, componentArtifact) {
    const stateBindingIds = new Set(
      (componentArtifact.ordinary_template_bindings ?? [])
        .filter((binding) => binding.state_storage_ids?.length === 1)
        .map((binding) => binding.id)
    );
    const bindings = (manifest.ordinary_bindings ?? []).filter((binding) =>
      stateBindingIds.has(binding.instance_binding_id)
      && (binding.kind === "text" || binding.kind === "attribute" || binding.kind === "property")
    );
    const targetIds = new Set(bindings.map((binding) => binding.instance_target_id));
    for (const event of manifest.ordinary_events ?? []) targetIds.add(event.instance_target_id);
    const targets = (manifest.ordinary_targets ?? []).filter((target) => targetIds.has(target.id));
    initializeOrdinaryInstanceRuntime(
      store,
      { ...manifest, ordinary_targets: targets, ordinary_bindings: bindings },
      componentArtifact
    );
  }

  function establishResumeEffects(registry, store, effectArtifact) {
    for (const effect of effectArtifact?.effects ?? []) {
      if (store.effectSubscriptions.has(effect.effect) || registry.effect_subscriptions.has(effect.effect)) {
        throw new ResumeBootError("DuplicateIdentity");
      }
      const subscription = {
        effect_instance_id: effect.effect,
        scheduler_order: effect.initial_trigger?.effect_batch_index ?? null,
        active_after_restore: true,
        run_on_restore: effect.run_on_resume === true
      };
      store.effectSubscriptions.set(effect.effect, subscription);
      registry.effect_subscriptions.set(effect.effect, subscription);
    }
    executeResumeEffects(store);
  }

  function ordinaryTargetFromEvent(target) {
    let current = target instanceof Element ? target : target?.parentElement;
    while (current !== null && current !== undefined) {
      const targetId = current.getAttribute("data-presolve-ti");
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
    const actionRecord = store.actionsByMethod.get(record.handler_method_id)
      ?? (store.opaqueTerminalsByMethod.has(record.handler_method_id)
        ? { action_batch_id: record.action_batch_id, actions: [] }
        : undefined);
    const component = store.components.get(record.component_instance_id);
    if (actionRecord === undefined || component === undefined || actionRecord.action_batch_id !== record.action_batch_id) {
      throw new PresolveBootError("PSR_INVALID_ORDINARY_EVENT");
    }
    const context = {
      component_instance_id: record.component_instance_id,
      trigger_target_id: record.instance_target_id,
      declaration_event_id: record.declaration_event_id,
      action_batch_id: record.action_batch_id,
      method_id: record.handler_method_id,
      arguments: Array.isArray(record.arguments) ? record.arguments : []
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
        reportDiagnostic(diagnostics, "PSR_UNKNOWN_FORM_INSTANCE", "Forms artifact referenced an unknown Form definition", { instance: instance.id, form: instance.form }, true);
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
        reportDiagnostic(diagnostics, "PSR_FORMS_MANIFEST_MISMATCH", "Forms manifest bridge did not resolve an exact compiler anchor and instance", { bridge }, true);
        continue;
      }
      const binding = formInstance.definition.bindings.find((item) => item.id === bridge.field_binding_id);
      if (binding === undefined || binding.field === undefined || binding.channel !== bridge.channel) {
        reportDiagnostic(diagnostics, "PSR_UNKNOWN_FORM_BINDING", "Forms manifest bridge did not match an artifact binding", { bridge }, true);
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
        reportDiagnostic(diagnostics, "PSR_FORMS_MANIFEST_MISMATCH", "Forms host bridge did not resolve an exact compiler-owned form anchor", { bridge }, true);
        continue;
      }
      const anchor = manifest.schema_version === SUPPORTED_SCHEMA_VERSION ? bridge.instance_target_id : bridge.host_anchor;
      store.formHostsByAnchor.set(anchor, { bridge, host, element, formInstance });
      element.addEventListener(host.event, (event) => dispatchFormSubmit(store, event, anchor));
    }

    document.addEventListener("input", (event) => dispatchFormEvent(store, event, false));
    document.addEventListener("change", (event) => dispatchFormEvent(store, event, false));
    document.addEventListener("focusout", (event) => dispatchFormEvent(store, event, true));
    window.__PRESOLVE_FORMS__ = {
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
      reportDiagnostic(store.diagnostics, "PSR_UNRESOLVED_FORM_SUBMIT_ACTION", "Submission host did not resolve its exact compiler action", { host: record.host }, true);
      record.formInstance.submission = "Failed";
      return;
    }
    record.formInstance.submission = "Submitting";
    // Serialization is deliberately compiler-record driven: field values and
    // path shape come from the compiler artifact, never DOM scanning.
    record.formInstance.serialized = serializeFormInstance(record.formInstance);
    executeActions(store, component, record.host.action_batch, action.actions, {
      component_instance_id: record.bridge.component_instance_id,
      trigger_target_id: record.bridge.instance_target_id,
      declaration_event_id: record.host.submit_action,
      action_batch_id: record.host.action_batch
    });
    record.formInstance.submission = "Completed";
  }

  function serializeFormInstance(formInstance) {
    const definition = formInstance.definition;
    const fields = new Map((definition.fields ?? []).map((field) => [field.id, field]));
    if (definition.serialization?.format === "Json") {
      const result = {};
      for (const [fieldId, state] of formInstance.fields.entries()) {
        const path = fields.get(fieldId)?.path;
        if (!Array.isArray(path) || path.length === 0) continue;
        let target = result;
        for (const segment of path.slice(0, -1)) target = target[segment] ??= {};
        target[path[path.length - 1]] = state.value;
      }
      return result;
    }
    return [...formInstance.fields.entries()].map(([field, state]) => ({
      key: fields.get(field)?.path?.join(".") ?? field,
      value: state.value
    }));
  }

  function dispatchFormEvent(store, event, blur) {
    const element = event.target;
    if (!(element instanceof HTMLElement)) return;
    const anchor = store.templateTargetsById instanceof Map
      ? ordinaryTargetFromEvent(element)
      : element.getAttribute("data-presolve-node");
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
    if (store.instanceQualifiedState) {
      return [...store.computedSlotsByInstanceComputed.entries()].map(([key, slot]) => ({
        component_instance_id: key.slice(0, key.lastIndexOf("|")),
        computed: slot.computed_id,
        cache_slot: slot.cache_slot_id,
        dirty: store.computedDirtySlots.get(slot.dirty_slot_id) === true,
        value: store.computedCaches.get(slot.cache_slot_id)
      }));
    }
    return [...store.computedEvaluations.values()].map((evaluation) => ({
      computed: evaluation.computed,
      cache_slot: evaluation.cache_slot,
      dirty: store.computedDirty.get(evaluation.computed) === true,
      value: store.computedCaches.get(evaluation.cache_slot)
    }));
  }

  function refreshComputedDebugState(store) {
    if (window.__PRESOLVE__?.store !== store) {
      return;
    }

    window.__PRESOLVE__.computed = debugComputed(store);
    window.__PRESOLVE__.computed_update_runs = store.computedUpdateRuns;
  }

  function initialStateSlotValue(slot) {
    if (slot.semantic_type !== "number") return slot.initial_value;
    const value = Number(slot.initial_value);
    if (!Number.isFinite(value)) throw new PresolveBootError("PSR_INVALID_COMPONENT_ARTIFACT");
    return value;
  }

  function decodeResumeValue(value, codec) {
    switch (codec?.kind) {
      case "null_codec":
        if (value !== null) throw new ResumeBootError("ValueTypeMismatch");
        return null;
      case "boolean_codec":
        if (typeof value !== "boolean") throw new ResumeBootError("ValueTypeMismatch");
        return value;
      case "number_codec":
        if (typeof value !== "number" || !Number.isFinite(value) || Object.is(value, -0)) {
          throw new ResumeBootError("ValueTypeMismatch");
        }
        return value;
      case "string_codec":
        if (typeof value !== "string") throw new ResumeBootError("ValueTypeMismatch");
        return value;
      case "array_codec":
        if (!Array.isArray(value)) throw new ResumeBootError("ValueTypeMismatch");
        return value.map((entry) => decodeResumeValue(entry, codec.value));
      case "object_codec": {
        if (value === null || typeof value !== "object" || Array.isArray(value)) {
          throw new ResumeBootError("ValueTypeMismatch");
        }
        const properties = codec.value ?? [];
        if (!exactObjectKeys(value, properties.map((property) => property.name))) {
          throw new ResumeBootError("ValueTypeMismatch");
        }
        return Object.fromEntries(properties.map((property) => [
          property.name,
          decodeResumeValue(value[property.name], property.codec)
        ]));
      }
      case "nullable_codec":
        return value === null ? null : decodeResumeValue(value, codec.value);
      default:
        throw new ResumeBootError("ValueCodecFailure");
    }
  }

  function snapshotValuesBySlot(snapshot) {
    const values = new Map();
    for (const boundary of snapshot.boundaries) {
      for (const record of boundary.values) values.set(record.slotId, record.value);
    }
    return values;
  }

  function allocateResumeStateComputedStore(
    registry,
    manifest,
    templateManifest,
    computedArtifact,
    contextArtifact,
    effectArtifact,
    componentArtifact,
    diagnostics
  ) {
    const store = createRuntimeStore(
      collectElementAnchors(),
      diagnostics,
      computedArtifact,
      contextArtifact,
      effectArtifact,
      componentArtifact
    );
    store.componentArtifact = componentArtifact;
    store.componentInstances = new Map();
    store.slotBindings = new Map();
    store.componentRegions = new Map();
    store.instanceContextBindings = new Map(
      (componentArtifact?.instance_context_bindings ?? [])
        .map((binding) => [binding.consumer_instance, binding])
    );
    const definitions = new Map(
      (templateManifest.components ?? []).map((component) => [component.component_id, component])
    );
    for (const boundary of manifest.boundaries) {
      registry.boundary_records.set(boundary.boundary_id, {
        boundary_id: boundary.boundary_id,
        kind: boundary.kind,
        status: "allocated"
      });
    }
    for (const instance of componentArtifact.instances ?? []) {
      const definition = definitions.get(instance.component);
      if (definition === undefined) throw new ResumeBootError("ResumeArtifactMismatch");
      for (const slot of instance.state_slots ?? []) {
        const key = `${instance.instance}|${slot.storage_id}`;
        if (store.stateSlotsByInstanceStorage.has(key)) {
          throw new ResumeBootError("DuplicateIdentity");
        }
        store.stateSlotsByInstanceStorage.set(key, slot);
      }
      for (const slot of instance.computed_slots ?? []) {
        const key = `${instance.instance}|${slot.computed_id}`;
        if (store.computedSlotsByInstanceComputed.has(key)) {
          throw new ResumeBootError("DuplicateIdentity");
        }
        store.computedSlotsByInstanceComputed.set(key, slot);
        store.computedDirtySlots.set(slot.dirty_slot_id, false);
      }
      const component = {
        instance_id: instance.instance,
        name: instance.component,
        manifest: definition,
        state: {}
      };
      store.components.set(instance.instance, component);
      registerActions(store, component, definition);
    }
    return store;
  }

  function writeResumeSlot(store, registry, slotSchema, value) {
    registry.slot_values.set(slotSchema.slot_id, value);
    const existing = slotSchema.existing_storage_slot_id;
    if (slotSchema.restore_phase === "R3") {
      store.storageValues.set(existing, value);
      return;
    }
    if (slotSchema.restore_phase === "R4") {
      const computedSlot = [...store.computedSlotsByInstanceComputed.values()]
        .find((slot) => slot.cache_slot_id === existing || slot.dirty_slot_id === existing);
      if (computedSlot === undefined) throw new ResumeBootError("UnknownSlot");
      if (computedSlot.cache_slot_id === existing) store.computedCaches.set(existing, value);
      else store.computedDirtySlots.set(existing, value);
      return;
    }
    if (slotSchema.restore_phase === "R6") {
      store.contextSlots.set(existing, value);
      return;
    }
    throw new ResumeBootError("RestoreInstructionFailure");
  }

  function executeResumeDecodeAndWrites(store, registry, snapshot, phases) {
    const snapshotValues = snapshotValuesBySlot(snapshot);
    const decoded = new Map();
    for (const program of registry.definitions.restorePrograms.values()) {
      for (const record of program.instructions ?? []) {
        if (!phases.has(record.phase)) continue;
        const instruction = record.instruction;
        if (instruction.kind === "decode_value") {
          const schema = registry.definitions.slots.get(instruction.slot_id);
          if (schema === undefined || schema.restore_phase !== record.phase) {
            throw new ResumeBootError("RestoreInstructionFailure");
          }
          if (!snapshotValues.has(instruction.slot_id)) throw new ResumeBootError("UnknownSlot");
          decoded.set(
            instruction.slot_id,
            decodeResumeValue(snapshotValues.get(instruction.slot_id), instruction.codec)
          );
        } else if (instruction.kind === "write_slot") {
          const schema = registry.definitions.slots.get(instruction.slot_id);
          if (schema === undefined || !decoded.has(instruction.slot_id)) {
            throw new ResumeBootError("RestoreInstructionFailure");
          }
          writeResumeSlot(store, registry, schema, decoded.get(instruction.slot_id));
        } else {
          throw new ResumeBootError("RestoreInstructionFailure");
        }
      }
    }
  }

  function synchronizeRestoredComponentState(store, computedArtifact, componentArtifact) {
    const fieldsByStorage = new Map(
      (computedArtifact?.state ?? []).map((state) => [state.storage, state.field])
    );
    for (const instance of componentArtifact.instances ?? []) {
      const component = store.components.get(instance.instance);
      if (component === undefined) throw new ResumeBootError("ResumeArtifactMismatch");
      for (const slot of instance.state_slots ?? []) {
        const field = fieldsByStorage.get(slot.storage_id);
        if (field === undefined || !store.storageValues.has(slot.slot_id)) {
          throw new ResumeBootError("UnknownSlot");
        }
        component.state[field] = store.storageValues.get(slot.slot_id);
      }
    }
  }

  function executeResumeComputedRecomputation(store, registry) {
    for (const schema of registry.definitions.slots.values()) {
      if (schema.restore_phase !== "R5") continue;
      const computedSlot = [...store.computedSlotsByInstanceComputed.values()]
        .find((slot) =>
          slot.cache_slot_id === schema.existing_storage_slot_id
          || slot.dirty_slot_id === schema.existing_storage_slot_id
        );
      if (computedSlot === undefined) throw new ResumeBootError("UnknownSlot");
      if (computedSlot.dirty_slot_id === schema.existing_storage_slot_id) {
        store.computedDirtySlots.set(computedSlot.dirty_slot_id, true);
      }
    }
    const recomputed = new Set();
    for (const program of registry.definitions.restorePrograms.values()) {
      for (const record of program.instructions ?? []) {
        if (record.phase !== "R5") continue;
        const instruction = record.instruction;
        if (instruction.kind !== "recompute_computed") {
          throw new ResumeBootError("RestoreInstructionFailure");
        }
        const key = `${instruction.component_instance_id}|${instruction.computed_id}`;
        const slot = store.computedSlotsByInstanceComputed.get(key);
        const evaluation = store.computedEvaluations.get(instruction.computed_id);
        if (slot === undefined || evaluation === undefined || recomputed.has(key)) {
          throw new ResumeBootError("RestoreInstructionFailure");
        }
        const prior = store.activeExecutionContext;
        store.activeExecutionContext = {
          component_instance_id: instruction.component_instance_id
        };
        try {
          const value = executeComputedProgram(store, evaluation);
          storeComputedValue(store, evaluation, value);
          setComputedDirty(store, instruction.computed_id, false);
          const cacheSchema = [...registry.definitions.slots.values()]
            .find((schema) => schema.existing_storage_slot_id === slot.cache_slot_id);
          const dirtySchema = [...registry.definitions.slots.values()]
            .find((schema) => schema.existing_storage_slot_id === slot.dirty_slot_id);
          if (cacheSchema === undefined || dirtySchema === undefined) {
            throw new ResumeBootError("UnknownSlot");
          }
          registry.slot_values.set(cacheSchema.slot_id, value);
          registry.slot_values.set(dirtySchema.slot_id, false);
          recomputed.add(key);
        } finally {
          store.activeExecutionContext = prior;
        }
      }
    }
    return [...recomputed];
  }

  function executeResumeContextBindings(store, registry, componentArtifact) {
    const expected = new Map(
      (componentArtifact?.instance_context_bindings ?? [])
        .map((binding) => [binding.consumer_instance, binding])
    );
    for (const program of registry.definitions.restorePrograms.values()) {
      for (const record of program.instructions ?? []) {
        if (record.phase !== "R7") continue;
        const instruction = record.instruction;
        if (instruction.kind !== "bind_context_consumer") {
          throw new ResumeBootError("RestoreInstructionFailure");
        }
        const binding = expected.get(instruction.consumer_instance_id);
        if (
          binding === undefined
          || binding.selected_source !== instruction.selected_source
          || (binding.provider_source ?? null) !== (instruction.provider_instance_id ?? null)
          || binding.runtime_slot !== instruction.value_slot_id
          || !store.contextSlots.has(instruction.value_slot_id)
          || store.contextConsumerBindings.has(instruction.consumer_instance_id)
        ) {
          throw new ResumeBootError("ResumeArtifactMismatch");
        }
        const installed = {
          consumer_instance_id: instruction.consumer_instance_id,
          selected_source: instruction.selected_source,
          provider_instance_id: instruction.provider_instance_id ?? null,
          value_slot_id: instruction.value_slot_id
        };
        store.contextConsumerBindings.set(
          instruction.consumer_instance_id,
          instruction.value_slot_id
        );
        registry.context_bindings.set(instruction.consumer_instance_id, installed);
      }
    }
    if (store.contextConsumerBindings.size !== expected.size) {
      throw new ResumeBootError("ResumeArtifactMismatch");
    }
  }

  function collectExactResumeAnchors(manifest) {
    const elements = new Map();
    for (const element of document.querySelectorAll("[data-presolve-r]")) {
      const id = element.getAttribute("data-presolve-r");
      if (elements.has(id)) throw new ResumeBootError("DuplicateIdentity");
      elements.set(id, element);
    }
    const comments = new Map();
    const walker = document.createTreeWalker(document, NodeFilter.SHOW_COMMENT);
    for (let node = walker.nextNode(); node !== null; node = walker.nextNode()) {
      const value = node.nodeValue ?? "";
      if (!value.startsWith("presolve-r-start:") && !value.startsWith("presolve-r-end:")) continue;
      const id = value.slice(value.indexOf(":") + 1);
      if (comments.has(id)) throw new ResumeBootError("DuplicateIdentity");
      comments.set(id, node);
    }
    const anchors = new Map();
    for (const anchor of manifest.anchors) {
      const node = anchor.kind === "structural_start" || anchor.kind === "structural_end"
        ? comments.get(anchor.anchor_id)
        : elements.get(anchor.anchor_id);
      if (anchor.required && node === undefined) throw new ResumeBootError("MissingAnchor");
      if (node !== undefined) anchors.set(anchor.anchor_id, node);
    }
    return anchors;
  }

  function canonicalResumeValueEqual(left, right) {
    return JSON.stringify(left) === JSON.stringify(right);
  }

  function restoreResumeComponentsSlotsAndStructure(
    manifest,
    registry,
    store,
    componentArtifact
  ) {
    const componentRecords = new Map();
    const slotRecords = new Map();
    const structuralRecords = new Map();
    for (const item of manifest.phase_i_component_resume_records ?? []) {
      if (item.record_kind === "component_instance") {
        const record = item.component_instance;
        if (componentRecords.has(record.instance)) throw new ResumeBootError("DuplicateIdentity");
        componentRecords.set(record.instance, record);
      } else if (item.record_kind === "slot_binding") {
        const record = item.slot_binding;
        if (slotRecords.has(record.binding)) throw new ResumeBootError("DuplicateIdentity");
        slotRecords.set(record.binding, record);
      } else if (item.record_kind === "structural_region") {
        const record = item.structural_region;
        if (structuralRecords.has(record.region)) throw new ResumeBootError("DuplicateIdentity");
        structuralRecords.set(record.region, record);
      }
    }

    const planned = new Map(
      (componentArtifact.instances ?? []).map((instance) => [instance.instance, instance])
    );
    const templateInstances = new Set(
      (componentArtifact.structural_programs ?? [])
        .flatMap((program) => program.template_instances ?? [])
    );
    if (componentRecords.size !== planned.size + templateInstances.size) {
      throw new ResumeBootError("ResumeArtifactMismatch");
    }
    for (const record of componentRecords.values()) {
      const instance = planned.get(record.instance);
      if (instance !== undefined) {
        if (
          record.component !== instance.component
          || (record.parent_instance ?? null) !== (instance.parent ?? null)
          || (record.structural_region ?? null) !== (instance.structural_region ?? null)
          || record.active_status !== "active"
        ) {
          throw new ResumeBootError("ResumeArtifactMismatch");
        }
      } else if (!templateInstances.has(record.instance) || record.active_status !== "inactive") {
        throw new ResumeBootError("ResumeArtifactMismatch");
      }
      const runtimeRecord = { ...record, status: record.active_status };
      registry.component_records.set(record.instance, runtimeRecord);
      store.componentInstances.set(record.instance, runtimeRecord);
    }

    const expectedSlots = new Map(
      (componentArtifact.slot_binding_programs ?? []).map((binding) => [binding.binding, binding])
    );
    if (slotRecords.size !== expectedSlots.size) throw new ResumeBootError("ResumeArtifactMismatch");
    for (const record of slotRecords.values()) {
      const binding = expectedSlots.get(record.binding);
      if (
        binding === undefined
        || record.caller_instance !== binding.caller_instance
        || record.callee_instance !== binding.callee_instance
      ) {
        throw new ResumeBootError("ResumeArtifactMismatch");
      }
      store.slotBindings.set(record.binding, binding);
    }

    const expectedRegions = new Map(
      (componentArtifact.structural_programs ?? []).map((program) => [program.region, program])
    );
    if (structuralRecords.size !== expectedRegions.size) {
      throw new ResumeBootError("ResumeArtifactMismatch");
    }
    for (const record of structuralRecords.values()) {
      const program = expectedRegions.get(record.region);
      if (program === undefined || record.active_status !== "inactive") {
        throw new ResumeBootError("ResumeArtifactMismatch");
      }
      const runtimeRecord = { ...record, program, selection_value: undefined };
      registry.structural_records.set(record.region, runtimeRecord);
      store.componentRegions.set(record.region, runtimeRecord);
    }

    const restoredRegions = new Set();
    for (const program of registry.definitions.restorePrograms.values()) {
      for (const record of program.instructions ?? []) {
        if (record.phase !== "R9") continue;
        const instruction = record.instruction;
        if (
          instruction.kind !== "restore_structural_selection"
          || restoredRegions.has(instruction.region_id)
        ) {
          throw new ResumeBootError("RestoreInstructionFailure");
        }
        const runtimeRecord = registry.structural_records.get(instruction.region_id);
        const schema = registry.definitions.slots.get(instruction.slot_id);
        const value = registry.slot_values.get(instruction.slot_id);
        if (runtimeRecord === undefined || schema === undefined || value === undefined) {
          throw new ResumeBootError("ResumeArtifactMismatch");
        }
        const stateSlot = (componentArtifact.instances ?? [])
          .flatMap((instance) => instance.state_slots ?? [])
          .find((slot) => slot.slot_id === schema.existing_storage_slot_id);
        if (stateSlot === undefined || !canonicalResumeValueEqual(value, stateSlot.initial_value)) {
          throw new ResumeBootError("StructuralStateMismatch");
        }
        runtimeRecord.selection_value = value;
        restoredRegions.add(instruction.region_id);
      }
    }
    if (restoredRegions.size !== expectedRegions.size) {
      throw new ResumeBootError("ResumeArtifactMismatch");
    }
    store.resumeAnchors = collectExactResumeAnchors(manifest);
  }

  function restoreResumeForms(templateManifest, snapshot, registry, store, formsArtifact) {
    const restoreRecords = [...registry.definitions.restorePrograms.values()]
      .flatMap((program) => program.instructions ?? [])
      .filter((record) => ["R11", "R12", "R13", "R14", "R15"].includes(record.phase));
    if (formsArtifact === null) {
      if (restoreRecords.length !== 0) throw new ResumeBootError("ResumeArtifactMismatch");
      return;
    }
    const targets = collectOrdinaryTargetAnchors();
    const definitions = new Map((formsArtifact.forms ?? []).map((form) => [form.id, form]));
    const instances = new Map((formsArtifact.instances ?? []).map((instance) => [instance.id, instance]));
    if (definitions.size !== (formsArtifact.forms ?? []).length || instances.size !== (formsArtifact.instances ?? []).length) {
      throw new ResumeBootError("DuplicateIdentity");
    }
    store.forms = new Map();
    store.formInstances = new Map();
    store.formBindingsByAnchor = new Map();
    store.formBindingsByField = new Map();
    store.formHostsByAnchor = new Map();
    const slotOwners = new Map();
    for (const instance of instances.values()) {
      const definition = definitions.get(instance.form);
      if (definition === undefined || store.formInstances.has(instance.id)) {
        throw new ResumeBootError("ResumeArtifactMismatch");
      }
      const fields = new Map((definition.fields ?? []).map((field) => [field.id, {
        value: field.initial_value,
        initial: field.initial_value,
        dirty: false,
        touched: false,
        validation: []
      }]));
      if (fields.size !== (definition.fields ?? []).length) throw new ResumeBootError("DuplicateIdentity");
      const runtime = { definition, instance, fields, aggregate_valid: true, submission: "Idle" };
      store.forms.set(definition.id, definition);
      store.formInstances.set(instance.id, runtime);
      registry.form_records.set(instance.id, runtime);
      for (const slots of instance.field_slots ?? []) {
        if (!fields.has(slots.field)) throw new ResumeBootError("ResumeArtifactMismatch");
        for (const [kind, slot] of [["value", slots.value], ["dirty", slots.dirty], ["touched", slots.touched], ["validation", slots.validation]]) {
          if (slotOwners.has(slot)) throw new ResumeBootError("DuplicateIdentity");
          slotOwners.set(slot, { instance: runtime, field: slots.field, kind });
        }
      }
      if (slotOwners.has(instance.aggregate_validation_slot) || slotOwners.has(instance.submission_slot)) {
        throw new ResumeBootError("DuplicateIdentity");
      }
      slotOwners.set(instance.aggregate_validation_slot, { instance: runtime, field: null, kind: "aggregate" });
      slotOwners.set(instance.submission_slot, { instance: runtime, field: null, kind: "submission" });
    }

    const snapshotValues = snapshotValuesBySlot(snapshot);
    const restored = new Set();
    for (const record of restoreRecords) {
      if (record.phase === "R15") continue;
      const instruction = record.instruction;
      if (instruction.kind !== "decode_value") continue;
      const schema = registry.definitions.slots.get(instruction.slot_id);
      if (schema === undefined || schema.restore_phase !== record.phase || !snapshotValues.has(instruction.slot_id)) {
        throw new ResumeBootError("RestoreInstructionFailure");
      }
      const owner = slotOwners.get(schema.existing_storage_slot_id);
      if (owner === undefined || restored.has(instruction.slot_id)) throw new ResumeBootError("ResumeArtifactMismatch");
      const value = decodeResumeValue(snapshotValues.get(instruction.slot_id), instruction.codec);
      const field = owner.field === null ? undefined : owner.instance.fields.get(owner.field);
      if (owner.kind === "value" && field !== undefined) field.value = value;
      else if (owner.kind === "dirty" && field !== undefined && typeof value === "boolean") field.dirty = value;
      else if (owner.kind === "touched" && field !== undefined && typeof value === "boolean") field.touched = value;
      else if (owner.kind === "validation" && field !== undefined && Array.isArray(value) && value.every((item) => typeof item === "string")) field.validation = value;
      else if (owner.kind === "aggregate" && typeof value === "boolean") owner.instance.aggregate_valid = value;
      else if (owner.kind === "submission" && ["Idle", "Completed", "Failed", "Invalid"].includes(value)) owner.instance.submission = value;
      else throw new ResumeBootError(owner.kind === "submission" ? "UnstableFormSubmission" : "ValueTypeMismatch");
      registry.slot_values.set(instruction.slot_id, value);
      restored.add(instruction.slot_id);
    }
    const expectedSlots = [...registry.definitions.slots.values()]
      .filter((schema) => ["R11", "R12", "R13", "R14"].includes(schema.restore_phase));
    if (restored.size !== expectedSlots.length) throw new ResumeBootError("RestoreInstructionFailure");

    for (const bridge of templateManifest.form_bindings ?? []) {
      const target = targets.targets.get(bridge.instance_target_id);
      const formInstance = store.formInstances.get(bridge.form_instance_id);
      if (!(target instanceof Element) || targets.duplicates.has(bridge.instance_target_id) || formInstance === undefined) {
        throw new ResumeBootError("MissingAnchor");
      }
      const binding = formInstance.definition.bindings.find((item) => item.id === bridge.field_binding_id);
      if (binding === undefined || binding.field === undefined || binding.channel !== bridge.channel || store.formBindingsByAnchor.has(bridge.instance_target_id)) {
        throw new ResumeBootError("ResumeArtifactMismatch");
      }
      const runtimeBinding = { bridge, binding, element: target, formInstance };
      store.formBindingsByAnchor.set(bridge.instance_target_id, runtimeBinding);
      const key = `${bridge.form_instance_id}|${binding.field}`;
      const bindings = store.formBindingsByField.get(key) ?? [];
      bindings.push(runtimeBinding);
      store.formBindingsByField.set(key, bindings);
      writeFormControl(runtimeBinding, formInstance.fields.get(binding.field)?.value);
    }
    const expectedBindings = [...instances.values()].reduce((count, instance) => count + (definitions.get(instance.form)?.bindings?.length ?? 0), 0);
    if (store.formBindingsByAnchor.size !== expectedBindings) throw new ResumeBootError("ResumeArtifactMismatch");
    window.__PRESOLVE_FORMS__ = {
      resetForm: (instanceId) => resetForm(store, instanceId),
      resetField: (instanceId, fieldId) => resetField(store, instanceId, fieldId)
    };
  }

  function restoreResumeRuntimeThroughForms(
    manifest,
    snapshot,
    registry,
    templateManifest,
    computedArtifact,
    contextArtifact,
    effectArtifact,
    componentArtifact,
    formsArtifact,
    diagnostics
  ) {
    if ((computedArtifact?.evaluations ?? []).some((evaluation) =>
      (evaluation.program?.instructions ?? []).some((instruction) => instruction.kind === "load-resource")
    )) {
      throw new ResumeBootError("ResourceComputedReadUnsupported");
    }
    const store = allocateResumeStateComputedStore(
      registry,
      manifest,
      templateManifest,
      computedArtifact,
      contextArtifact,
      effectArtifact,
      componentArtifact,
      diagnostics
    );
    executeResumeDecodeAndWrites(store, registry, snapshot, new Set(["R3", "R4"]));
    synchronizeRestoredComponentState(store, computedArtifact, componentArtifact);
    const recomputationRuns = executeResumeComputedRecomputation(store, registry);
    executeResumeDecodeAndWrites(store, registry, snapshot, new Set(["R6"]));
    executeResumeContextBindings(store, registry, componentArtifact);
    restoreResumeComponentsSlotsAndStructure(manifest, registry, store, componentArtifact);
    restoreResumeForms(templateManifest, snapshot, registry, store, formsArtifact);
    installResumeDomBindings(store, templateManifest, componentArtifact);
    establishResumeEffects(registry, store, effectArtifact);
    installEffectDisposal(store);
    installResumeActivationListeners(registry, store);
    const state = runtimeState({
      manifest: templateManifest,
      diagnostics,
      store,
      components: debugComponents(store),
      computed: debugComputed(store),
      computed_update_runs: 0,
      context_initial_source_runs: store.contextInitialSourceRuns,
      context_slots: [...store.contextSlots.entries()],
      context_consumer_bindings: [...store.contextConsumerBindings.entries()],
      context_failures: store.contextFailures,
      context_update_source_runs: store.contextUpdateSourceRuns,
      component_initialization_runs: [],
      component_instance_tree: [...store.componentInstances.values()],
      forms: [...store.formInstances.values()].map((instance) => ({
        id: instance.instance.id,
        form: instance.definition.id,
        aggregate_valid: instance.aggregate_valid,
        submission: instance.submission
      })),
      slot_binding_runs: [],
      component_failures: []
    });
    state.resume_recomputation_runs = recomputationRuns;
    return state;
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
    forms = [],
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
      forms,
      slot_binding_runs,
      component_failures
    };
  }

  async function initializeResourcesRuntime(store, resourcesArtifact, diagnostics) {
    if (resourcesArtifact === null) return;
    window.addEventListener("pagehide", () => {
      for (const resource of store.resources.values()) resource.controller.abort();
    }, { once: true });
    const declarations = new Map(resourcesArtifact.declarations.map((declaration) => [declaration.id, declaration]));
    const records = [];
    for (const activation of resourcesArtifact.activations) {
      const declaration = declarations.get(activation.declaration);
      if (declaration === undefined) throw new PresolveBootError("PSR_INVALID_RESOURCES_ARTIFACT");
      if (!["Client", "Shared"].includes(declaration.execution_boundary)) {
        reportDiagnostic(diagnostics, "PSR_RESOURCE_SERVER_UNAVAILABLE", "A server-only Resource cannot activate in the browser runtime", { activation, declaration }, true);
        throw new PresolveBootError("PSR_RESOURCE_SERVER_UNAVAILABLE");
      }
      const controller = new AbortController();
      const record = { activation, declaration, controller, state: "pending", generation: 1, data: null, error: null };
      const activationKey = `${activation.component_instance}\u001f${activation.declaration}`;
      if (store.resources.has(activation.id) || store.resourceActivationsByInstanceDeclaration.has(activationKey)) {
        throw new PresolveBootError("PSR_INVALID_RESOURCES_ARTIFACT");
      }
      store.resources.set(activation.id, record);
      store.resourceActivationsByInstanceDeclaration.set(activationKey, activation.id);
      records.push(record);
    }
    await Promise.all(records.map(async (record) => {
      try {
        const module = await import(record.declaration.endpoint.runtime_location);
        const endpoint = module[record.declaration.endpoint.export];
        if (typeof endpoint !== "function") throw new Error("endpoint-export-missing");
        const result = await endpoint({ signal: record.controller.signal, inputs: Object.freeze({}) });
        if (record.controller.signal.aborted) {
          record.state = "cancelled";
        } else {
          JSON.stringify(result);
          record.state = "ready";
          record.data = result;
        }
      } catch (error) {
        if (record.controller.signal.aborted) {
          record.state = "cancelled";
        } else {
          record.state = "failed";
          record.error = error instanceof Error ? error.message : String(error);
          reportDiagnostic(diagnostics, "PSR_RESOURCE_ENDPOINT_FAILURE", "A compiler-authorized Resource endpoint failed", { activation: record.activation.id, error: record.error });
        }
      }
      invalidateResourceComputeds(store, record.activation);
    }));
  }

  async function initializeRuntime(manifest, computedArtifact, contextArtifact, effectArtifact, componentArtifact, formsArtifact, resourcesArtifact, opaqueArtifact, diagnostics) {
    const bindingAnchors = collectBindingAnchors();
    const conditionalAnchors = collectConditionalAnchors();
    const listAnchors = collectListAnchors();
    const elementsByNode = collectElementAnchors();
    const store = createRuntimeStore(elementsByNode, diagnostics, computedArtifact, contextArtifact, effectArtifact, componentArtifact, opaqueArtifact);
    store.componentArtifact = componentArtifact;
    store.componentInstances = new Map((componentArtifact?.instances ?? []).map((instance) => [instance.instance, { ...instance, status: "created" }]));
    store.slotBindings = new Map((componentArtifact?.slot_binding_programs ?? []).map((binding) => [binding.binding, binding]));
    store.instanceContextBindings = new Map((componentArtifact?.instance_context_bindings ?? []).map((binding) => [binding.consumer_instance, binding]));
    store.componentRegions = new Map((componentArtifact?.structural_programs ?? []).map((program) => [program.region, program]));
    // Inactive compiler-issued occurrence templates. Dynamic materialization
    // must consume this table; it must not rediscover component structure from
    // DOM nodes or selectors.
    store.structuralOccurrences = new Map((componentArtifact?.structural_programs ?? []).map(
      (program) => [program.region, program.template_occurrences]
    ));
    store.structuralOccurrencesByInvocation = new Map();
    for (const [region, occurrences] of store.structuralOccurrences) {
      for (const occurrence of occurrences) {
        if (store.structuralOccurrencesByInvocation.has(occurrence.invocation)) {
          throw new PresolveBootError("PSR_INVALID_COMPONENT_ARTIFACT");
        }
        store.structuralOccurrencesByInvocation.set(occurrence.invocation, Object.freeze({
          ...occurrence,
          structural_region: region,
        }));
      }
    }
    const missingAnchors = manifest.schema_version === SUPPORTED_SCHEMA_VERSION
      ? []
      : collectMissingAnchors(
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

    const resourceInitialization = initializeResourcesRuntime(store, resourcesArtifact, diagnostics);

    if (manifest.schema_version === SUPPORTED_SCHEMA_VERSION) {
      const definitions = new Map((manifest.components ?? []).map((component) => [component.component_id, component]));
      for (const instance of componentArtifact.instances ?? []) {
        const definition = definitions.get(instance.component);
        if (definition === undefined) throw new PresolveBootError("PSR_INVALID_ORDINARY_COMPONENT");
        const definitionStates = (computedArtifact?.state ?? []).filter(
          (state) => state.component === definition.name
        );
        if ((instance.state_slots ?? []).length !== definitionStates.length) {
          throw new PresolveBootError("PSR_INVALID_COMPONENT_ARTIFACT");
        }
        for (const slot of instance.state_slots ?? []) {
          const pair = `${instance.instance}|${slot.storage_id}`;
          if (
            store.stateSlotsByInstanceStorage.has(pair)
            || store.storageValues.has(slot.slot_id)
          ) {
            throw new PresolveBootError("PSR_INVALID_COMPONENT_ARTIFACT");
          }
          store.stateSlotsByInstanceStorage.set(pair, slot);
          store.storageValues.set(slot.slot_id, initialStateSlotValue(slot));
        }
        for (const state of definitionStates) {
          if (!store.stateSlotsByInstanceStorage.has(`${instance.instance}|${state.storage}`)) {
            throw new PresolveBootError("PSR_INVALID_COMPONENT_ARTIFACT");
          }
        }
        for (const slot of instance.computed_slots ?? []) {
          const pair = `${instance.instance}|${slot.computed_id}`;
          if (!slot.cache_slot_id.startsWith(`${instance.instance}/computed-cache:`)
            || !slot.dirty_slot_id.startsWith(`${instance.instance}/computed-dirty:`)
            || store.computedSlotsByInstanceComputed.has(pair)
            || store.computedDirtySlots.has(slot.dirty_slot_id)) {
            throw new PresolveBootError("PSR_INVALID_COMPONENT_ARTIFACT");
          }
          store.computedSlotsByInstanceComputed.set(pair, slot);
          store.computedDirtySlots.set(slot.dirty_slot_id, slot.dirty_initial_value === true);
        }
        const component = { instance_id: instance.instance, name: instance.component, manifest: definition, state: {} };
        for (const state of computedArtifact?.state ?? []) {
          if (state.component !== definition.name) continue;
          const slot = store.stateSlotsByInstanceStorage.get(`${instance.instance}|${state.storage}`);
          if (slot !== undefined) component.state[state.field] = store.storageValues.get(slot.slot_id);
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

    await resourceInitialization;

    executeInitialEffects(store);
    installEffectDisposal(store);

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
      resources: [...store.resources.entries()].map(([id, resource]) => ({ id, state: resource.state, generation: resource.generation })),
      opaque_terminals: [...store.opaqueActivations],
      slot_binding_runs: [...store.slotBindings.keys()],
      component_failures: []
    });
  }

  async function boot() {
    const diagnostics = [];

    try {
      const productionArtifact = readProductionRuntimeArtifact();
      const productionIndexes = productionOrdinalIndexes(productionArtifact);
      const manifest = readManifest(diagnostics);
      const computedArtifact = readComputedArtifact(diagnostics);
      validateComputedArtifactSchema(computedArtifact, diagnostics);
      const contextArtifact = readContextArtifact(diagnostics);
      validateContextArtifactSchema(contextArtifact, diagnostics);
      const effectArtifact = readEffectArtifact(diagnostics);
      validateEffectArtifactSchema(effectArtifact, diagnostics);
      const componentArtifact = readComponentArtifact(diagnostics);
      validateComponentArtifactSchema(componentArtifact, diagnostics);
      validateEffectArtifactInstances(effectArtifact, componentArtifact, diagnostics);
      const opaqueArtifact = readOpaqueArtifact(diagnostics);
      validateOpaqueArtifact(opaqueArtifact, diagnostics);
      validateManifestSchema(manifest, effectArtifact, componentArtifact, opaqueArtifact, diagnostics);
      const formsArtifact = readFormsArtifact(diagnostics);
      validateFormsArtifact(formsArtifact, manifest, diagnostics);
      const resourcesArtifact = readResourcesArtifact(diagnostics);
      validateResourcesArtifact(resourcesArtifact, diagnostics);

      const result = await bootstrapResume({
        diagnostics,
        resumeBoot: ({ manifest: resumeManifest, snapshot, registry }) => {
          if ((opaqueArtifact?.activations ?? []).length > 0) {
            throw new ResumeBootError("OpaqueTerminalColdFallback");
          }
          return restoreResumeRuntimeThroughForms(
            resumeManifest,
            snapshot,
            registry,
            manifest,
            computedArtifact,
            contextArtifact,
            effectArtifact,
            componentArtifact,
            formsArtifact,
            diagnostics
          );
        },
        coldBoot: () => initializeRuntime(
          manifest,
          computedArtifact,
          contextArtifact,
          effectArtifact,
          componentArtifact,
          formsArtifact,
          resourcesArtifact,
          opaqueArtifact,
          diagnostics
        )
      });
      const state = result.mode === "cold"
        ? result.cold
        : result.resume;
      state.resume = {
        mode: result.mode,
        failure: result.failure,
        contract_version: RESUME_REGISTRY_CONTRACT_VERSION
      };
      state.disposeEffects = () => disposeEffectInstances(state.store);
      state.resume_registry = result.mode === "resume" ? result.registry : null;
      state.resume_debug = [...resumeBootstrapState.debug];
      state.production = productionIndexes === null ? null : {
        artifact_checksum: productionArtifact.integrity?.artifact_checksum ?? null,
        table_kinds: [...productionIndexes.keys()]
      };
      const status = state.diagnostics.some((diagnostic) => diagnostic.fatal)
        || state.missingAnchors.length > 0
        ? "error"
        : "ready";

      document.documentElement.dataset.presolveRuntime = status;
      window.__PRESOLVE__ = state;

      document.dispatchEvent(
        new CustomEvent("presolve:ready", {
          detail: state
        })
      );
    } catch (error) {
      document.documentElement.dataset.presolveRuntime = "error";
      if (error instanceof PresolveBootError) {
        reportDiagnostic(
          diagnostics,
          error.code,
          "Runtime boot failed at a closed compiler contract boundary",
          { code: error.code },
          true
        );
      } else {
        reportDiagnostic(
          diagnostics,
          "PSR_RUNTIME_BOOT_FAILED",
          "Runtime boot failed",
          { message: error instanceof Error ? error.message : String(error) },
          true
        );
      }

      window.__PRESOLVE__ = runtimeState({
        diagnostics
      });

      document.dispatchEvent(
        new CustomEvent("presolve:ready", {
          detail: window.__PRESOLVE__
        })
      );
    }
  }

  window.__PRESOLVE_RESUME__ = Object.freeze({
    bootstrapResume,
    captureSnapshot,
    activateByEvent,
    activateBoundary,
    debugEvidence: () => [...resumeBootstrapState.debug]
  });

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
    RUNTIME_STUB
        .replace(
            "__EZ_COMPONENT_SCHEMA_VERSION__",
            &crate::RUNTIME_COMPONENT_ARTIFACT_SCHEMA_VERSION.to_string(),
        )
        .replace(
            "__EZ_EFFECT_SCHEMA_VERSION__",
            &crate::RUNTIME_EFFECT_ARTIFACT_SCHEMA_VERSION.to_string(),
        )
}

#[cfg(test)]
mod tests {
    use super::generate_runtime_stub;

    #[test]
    fn emits_runtime_manifest_bootstrap() {
        let runtime = generate_runtime_stub();

        assert!(runtime.contains("presolve-template-manifest"));
        assert!(runtime.contains("presolve-effect-runtime"));
        assert!(runtime.contains("presolve-context-runtime"));
        assert!(runtime.contains("executeInitialContext(store)"));
        assert!(runtime.contains("executeContextUpdates(store, actionBatchId)"));
        assert!(runtime.contains("contextSlots: new Map()"));
        assert!(runtime.contains("RUNTIME_VERSION = \"0.0.0\""));
        assert!(runtime.contains("SUPPORTED_SCHEMA_VERSION = 5"));
        assert!(runtime.contains("SUPPORTED_COMPUTED_ARTIFACT_SCHEMA_VERSION = 12"));
        assert!(runtime.contains("instruction.kind === \"load-resource\""));
        assert!(runtime.contains("resourceInvalidationsByDeclaration"));
        assert!(runtime.contains("case \"abs\""));
        assert!(runtime.contains("instruction.kind === \"select\""));
        assert!(runtime.contains("instruction.kind === \"get-index\""));
        assert!(runtime.contains("presolve-forms-runtime"));
        assert!(runtime.contains("presolve-resources-runtime"));
        assert!(runtime.contains("presolve-opaque-runtime"));
        assert!(runtime.contains("validateOpaqueArtifact"));
        assert!(runtime.contains("executeOpaqueTerminal"));
        assert!(runtime.contains("PSR_OPAQUE_TERMINAL_FAILURE"));
        assert!(runtime.contains("OpaqueTerminalColdFallback"));
        assert!(runtime.contains("validateResourcesArtifact"));
        assert!(runtime.contains("initializeResourcesRuntime"));
        assert!(runtime.contains("AbortController"));
        assert!(runtime.contains("pagehide"));
        assert!(runtime.contains("initializeFormsRuntime"));
        assert!(runtime.contains("dispatchFormSubmit"));
        assert!(runtime.contains("form_hosts"));
        assert!(runtime.contains("structuralOccurrences = new Map"));
        assert!(runtime.contains("structuralOccurrencesByInvocation"));
        assert!(runtime.contains("function structuralOccurrenceIdentity"));
        assert!(runtime.contains("function decodeStructuralOccurrenceIdentity"));
        assert!(runtime.contains("function instantiateStructuralTemplateSlots"));
        assert!(runtime.contains("Conditional host fragments were attached to a keyed-list host"));
        assert!(runtime.contains("occurrence.ordinary_template_targets"));
        assert!(runtime.contains("occurrence.ordinary_template_bindings"));
        assert!(runtime.contains("occurrence.ordinary_template_events"));
        assert!(!runtime.contains("FormData(formElement)"));
        assert!(runtime.contains("PSR_MISSING_MANIFEST"));
        assert!(runtime.contains("PSR_INVALID_MANIFEST_JSON"));
        assert!(runtime.contains("PSR_UNSUPPORTED_SCHEMA"));
        assert!(runtime.contains("data-presolve-node"));
        assert!(runtime.contains("ordinaryEventsByTargetAndType"));
        assert!(runtime.contains("component_instance_id: record.component_instance_id"));
        assert!(runtime.contains("computedSlotsByInstanceComputed: new Map()"));
        assert!(runtime.contains("computedDirtySlots: new Map()"));
        assert!(runtime.contains("function computedSlotForExecution"));
        assert!(runtime.contains("/computed-cache:"));
        assert!(runtime.contains("/computed-dirty:"));
        assert!(runtime.contains("LEGACY_COMPONENT_ARTIFACT_SCHEMA_VERSION = 2"));
        assert!(runtime.contains("RESUME_REGISTRY_CONTRACT_VERSION = 1"));
        assert!(runtime.contains("function validateResumeManifest"));
        assert!(runtime.contains("function validateResumeSnapshot"));
        assert!(runtime.contains("async function bootstrapResume"));
        assert!(runtime.contains("function captureSnapshot"));
        assert!(runtime.contains("async function activateByEvent"));
        assert!(runtime.contains("async function activateBoundary"));
        assert!(runtime.contains("function decodeResumeValue"));
        assert!(runtime.contains("function restoreResumeRuntimeThroughForms"));
        assert!(runtime.contains("ResourceComputedReadUnsupported"));
        assert!(runtime.contains("executeResumeComputedRecomputation"));
        assert!(runtime.contains("function executeResumeContextBindings"));
        assert!(runtime.contains("function restoreResumeComponentsSlotsAndStructure"));
        assert!(runtime.contains("function restoreResumeForms"));
        assert!(runtime.contains("function installResumeDomBindings"));
        assert!(runtime.contains("function establishResumeEffects"));
        assert!(runtime.contains("function collectExactResumeAnchors"));
        assert!(runtime.contains("window.__PRESOLVE_RESUME__ = Object.freeze"));
        assert!(runtime.contains("throw new ResumeBootError(\"DoubleBootstrap\")"));
        assert!(runtime.contains("presolve-binding:"));
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
        assert!(runtime.contains("function executeResumeEffects"));
        assert!(runtime.contains("function validateEffectArtifactInstances"));
        assert!(runtime.contains("effectArtifact.structural_templates"));
        assert!(runtime.contains("function disposeEffectInstances"));
        assert!(runtime.contains("effect.run_on_resume !== true"));
        assert!(runtime.contains("dispatchEffectCapability"));
        assert!(!runtime.contains("const arguments ="));
        assert!(runtime.contains("formatBindingValue"));
        assert!(runtime.contains("value === null ? \"\" : String(value)"));
        assert!(runtime.contains("listItemMemberPath"));
        assert!(runtime.contains("populateListItemMemberBindings"));
        assert!(runtime.contains("updateListItemTextBindings"));
        assert!(runtime.contains("updateListItemAttributes"));
        assert!(runtime.contains("registerListItemEvents"));
        assert!(runtime.contains("unregisterListItemEvents"));
        assert!(runtime.contains("presolve-list-binding-end:"));
        assert!(runtime.contains("renderListItemElement"));
        assert!(runtime.contains("component.state[field] = node.initial_value"));
        assert!(!runtime.contains("component.state[field] = Number(node.initial_value)"));
        assert!(runtime.contains("installDelegatedEventListeners"));
        assert!(runtime.contains("document.addEventListener(eventType"));
        assert!(!runtime.contains("element.addEventListener(\"click\""));
        assert!(runtime.contains("action.operation !== \"toggle\""));
        assert!(runtime.contains("action.operation === \"assign\""));
        assert!(runtime.contains("action.operation === \"toggle\""));
        assert!(runtime.contains("PSR_MISSING_ELEMENT_ANCHOR"));
        assert!(runtime.contains("PSR_MISSING_BINDING_ANCHOR"));
        assert!(runtime.contains("PSR_UNRESOLVED_EVENT"));
        assert!(runtime.contains("PSR_UNRESOLVED_ACTION"));
        assert!(runtime.contains("PSR_INVALID_STATE_OPERATION"));
        assert!(runtime.contains("current + delta"));
        assert!(runtime.contains("dataset.presolveRuntime"));
        assert!(runtime.contains("presolve:ready"));
        assert!(runtime.contains("runtime_version"));
        assert!(runtime.contains("diagnostics"));
        assert!(runtime.contains("window.__PRESOLVE__"));
    }
}
