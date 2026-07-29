import {
  analyzeTypeScriptProject,
  classifyResolvedComponentHeritage,
  classifyResolvedIntrinsic,
  createCanonicalIntrinsicRegistry,
} from "./index.js";

export const V2_AUTHORED_AUTHORITY_SCHEMA_VERSION = 10;

/**
 * Resolves explicit source positions for the implemented decorator-free V2
 * authoring forms. Syntax selection belongs to the caller; this bridge owns
 * only TypeScript resolution and canonical-registry classification.
 */
export async function analyzeV2Authoring(request) {
  validateV2AuthoringRequest(request);
  const queries = {
    symbols: [
      ...(request.canonical.component ? [{ id: "canonical:component", ...request.canonical.component }] : []),
      ...(request.canonical.state ? [{ id: "canonical:state", ...request.canonical.state }] : []),
      ...(request.canonical.action ? [{ id: "canonical:action", ...request.canonical.action }] : []),
      ...(request.canonical.effect ? [{ id: "canonical:effect", ...request.canonical.effect }] : []),
      ...(request.canonical.slot ? [{ id: "canonical:slot", ...request.canonical.slot }] : []),
      ...(request.canonical.defineForm ? [{ id: "canonical:define-form", ...request.canonical.defineForm }] : []),
      ...(request.canonical.field ? [{ id: "canonical:field", ...request.canonical.field }] : []),
      ...request.canonical.validationRules.map(entry => ({
        id: `canonical-validation:${entry.name}`,
        file: entry.file,
        position: entry.position,
      })),
      ...(request.canonical.environment ? [{ id: "canonical:environment", ...request.canonical.environment }] : []),
      ...request.states.map(site => ({ id: `state:${site.id}`, file: site.file, position: site.position })),
      ...request.actions.map(site => ({ id: `action:${site.id}`, file: site.file, position: site.position })),
      ...request.effects.map(site => ({ id: `effect:${site.id}`, file: site.file, position: site.position })),
      ...request.slots.map(site => ({ id: `slot:${site.id}`, file: site.file, position: site.position })),
      ...request.forms.map(site => ({ id: `form:${site.id}`, file: site.file, position: site.position })),
      ...request.formFields.map(site => ({ id: `form-field:${site.id}`, file: site.file, position: site.position })),
      ...request.validations.map(site => ({ id: `validation:${site.id}`, file: site.file, position: site.position })),
      ...request.environmentPublic.flatMap(site => [
        { id: `environment-object:${site.id}`, file: site.file, position: site.objectPosition },
        { id: `environment-property:${site.id}`, file: site.file, position: site.propertyPosition },
      ]),
      ...request.packageInvocations.map(site => ({
        id: `package-invocation:${site.id}`,
        file: site.file,
        position: site.position,
      })),
      ...request.packageInvocations.map(site => ({
        id: `package-import:${site.id}`,
        file: site.file,
        position: site.importPosition,
      })),
    ],
    componentHeritage: request.components.map(site => ({
      id: `component:${site.id}`,
      file: site.file,
      position: site.position,
    })),
    signatures: request.formFields
      .map(site => ({
        id: `form-field-call:${site.id}`,
        file: site.file,
        position: site.position,
      })),
    standardSchemas: request.standardValidations.map(site => ({
      id: `standard-validation:${site.id}`,
      file: site.file,
      position: site.position,
    })),
  };
  const authority = await analyzeTypeScriptProject({
    configFile: request.configFile,
    ...(request.cwd === undefined ? {} : { cwd: request.cwd }),
    queries,
  });
  const symbols = new Map(authority.symbols.map(entry => [entry.id, entry.symbol]));
  const fieldSignatures = new Map(authority.signatures.map(entry => [entry.id, entry.signature]));
  const standardSchemas = new Map(
    authority.standardSchemas.map(entry => [entry.id, entry.standardSchema]),
  );
  const registry = createCanonicalIntrinsicRegistry([
    ...(request.canonical.component ? [{ kind: "component", symbol: symbols.get("canonical:component") }] : []),
    ...(request.canonical.state ? [{ kind: "state", symbol: symbols.get("canonical:state") }] : []),
    ...(request.canonical.action ? [{ kind: "action", symbol: symbols.get("canonical:action") }] : []),
    ...(request.canonical.effect ? [{ kind: "effect", symbol: symbols.get("canonical:effect") }] : []),
    ...(request.canonical.slot ? [{ kind: "slot", symbol: symbols.get("canonical:slot") }] : []),
    ...(request.canonical.defineForm ? [{ kind: "form", symbol: symbols.get("canonical:define-form") }] : []),
    ...(request.canonical.field ? [{ kind: "field", symbol: symbols.get("canonical:field") }] : []),
    ...request.canonical.validationRules.map(entry => ({
      kind: "validate",
      symbol: symbols.get(`canonical-validation:${entry.name}`),
    })),
    ...(request.canonical.environment ? [{ kind: "environment_public", symbol: symbols.get("canonical:environment") }] : []),
  ]);
  return {
    schemaVersion: V2_AUTHORED_AUTHORITY_SCHEMA_VERSION,
    diagnostics: authority.diagnostics,
    components: authority.componentHeritage.flatMap(site => {
      const intrinsic = classifyResolvedComponentHeritage(registry, site.bases);
      return intrinsic ? [{ id: stripPrefix(site.id, "component:"), identity: intrinsic.identity }] : [];
    }),
    states: request.states.flatMap(site => {
      const intrinsic = classifyResolvedIntrinsic(registry, symbols.get(`state:${site.id}`));
      return intrinsic?.kind === "state" ? [{ id: site.id, identity: intrinsic.identity }] : [];
    }),
    actions: request.actions.flatMap(site => {
      const intrinsic = classifyResolvedIntrinsic(registry, symbols.get(`action:${site.id}`));
      return intrinsic?.kind === "action" ? [{ id: site.id, identity: intrinsic.identity }] : [];
    }),
    effects: request.effects.flatMap(site => {
      const intrinsic = classifyResolvedIntrinsic(registry, symbols.get(`effect:${site.id}`));
      return intrinsic?.kind === "effect" ? [{ id: site.id, identity: intrinsic.identity }] : [];
    }),
    slots: request.slots.flatMap(site => {
      const intrinsic = classifyResolvedIntrinsic(registry, symbols.get(`slot:${site.id}`));
      return intrinsic?.kind === "slot" ? [{ id: site.id, identity: intrinsic.identity }] : [];
    }),
    forms: request.forms.flatMap(site => {
      const intrinsic = classifyResolvedIntrinsic(registry, symbols.get(`form:${site.id}`));
      return intrinsic?.kind === "form" ? [{ id: site.id, identity: intrinsic.identity }] : [];
    }),
    formFields: request.formFields.flatMap(site => {
      const intrinsic = classifyResolvedIntrinsic(registry, symbols.get(`form-field:${site.id}`));
      if (intrinsic?.kind !== "field") return [];
      const valueClassification = classifyFormFieldValue(
        fieldSignatures.get(`form-field-call:${site.id}`)?.returnType?.typeArguments?.[0],
      );
      return [{
        id: site.id,
        identity: intrinsic.identity,
        ...(valueClassification === undefined ? {} : { valueClassification }),
      }];
    }),
    validations: request.validations.flatMap(site => {
      const intrinsic = classifyResolvedIntrinsic(registry, symbols.get(`validation:${site.id}`));
      return intrinsic?.kind === "validate" ? [{ id: site.id, identity: intrinsic.identity }] : [];
    }),
    standardValidations: request.standardValidations.flatMap(site => {
      const schema = standardSchemas.get(`standard-validation:${site.id}`);
      const identity = resolvedIdentity(schema?.symbol);
      return schema?.version === 1
        && identity?.name === site.exportName
        && identity.declarationModules.length > 0
        ? [{
            id: site.id,
            identity,
            moduleSpecifier: site.moduleSpecifier,
            exportName: site.exportName,
            ...(schema.inputType === undefined ? {} : { inputType: schema.inputType.text }),
            ...(schema.outputType === undefined ? {} : { outputType: schema.outputType.text }),
          }]
        : [];
    }),
    packageInvocations: request.packageInvocations.flatMap(site => {
      const identity = resolvedIdentity(symbols.get(`package-invocation:${site.id}`));
      const importedIdentity = resolvedIdentity(symbols.get(`package-import:${site.id}`));
      return identity?.name === site.exportName
        && importedIdentity?.name === site.exportName
        && sameDeclarationModules(identity, importedIdentity)
        && identity.declarationModules.length > 0
        ? [{
            id: site.id,
            identity,
            moduleSpecifier: site.moduleSpecifier,
            exportName: site.exportName,
          }]
        : [];
    }),
    environmentPublic: request.environmentPublic.flatMap(site => {
      const receiver = classifyResolvedIntrinsic(registry, symbols.get(`environment-object:${site.id}`));
      const member = resolvedIdentity(symbols.get(`environment-property:${site.id}`));
      return receiver?.kind === "environment_public" && member?.name === "public"
        && sameDeclarationModules(member, receiver.identity)
        ? [{ id: site.id, identity: member }]
        : [];
    }),
  };
}

function validateV2AuthoringRequest(request) {
  if (!request || typeof request !== "object" || typeof request.configFile !== "string") {
    throw new TypeError("V2 authoring authority requests require a configFile");
  }
  if (!request.canonical || typeof request.canonical !== "object") {
    throw new TypeError("V2 authoring authority requests require canonical framework positions");
  }
  if (request.schemaVersion !== V2_AUTHORED_AUTHORITY_SCHEMA_VERSION) {
    throw new TypeError(`unsupported V2 authoring authority schema version ${request.schemaVersion}`);
  }
  for (const kind of ["component", "state", "action", "effect", "slot", "defineForm", "field", "environment"]) {
    if (request.canonical[kind] !== undefined) {
      validatePosition(request.canonical[kind], `canonical ${kind}`);
    }
  }
  if (!Array.isArray(request.canonical.validationRules)) {
    throw new TypeError("V2 authoring canonical validation rules must be an array");
  }
  for (const entry of request.canonical.validationRules) {
    if (!entry || typeof entry.name !== "string" || !entry.name) {
      throw new TypeError("V2 authoring canonical validation rules require names");
    }
    validatePosition(entry, `canonical validation ${entry.name}`);
  }
  for (const [kind, sites, canonicalKind] of [
    ["component", request.components, "component"],
    ["state", request.states, "state"],
    ["action", request.actions, "action"],
    ["effect", request.effects, "effect"],
    ["slot", request.slots, "slot"],
    ["form", request.forms, "defineForm"],
    ["form field", request.formFields, "field"],
  ]) {
    if (!Array.isArray(sites)) throw new TypeError(`V2 authoring ${kind} sites must be an array`);
    if (sites.length > 0 && request.canonical[canonicalKind] === undefined) {
      throw new TypeError(`V2 authoring ${kind} sites require a canonical ${canonicalKind} position`);
    }
    const ids = new Set();
    for (const site of sites) {
      if (!site || typeof site.id !== "string" || !site.id || ids.has(site.id)) {
        throw new TypeError(`V2 authoring ${kind} sites require unique non-empty ids`);
      }
      ids.add(site.id);
      validatePosition(site, `${kind} site`);
      if (kind === "form field" && site.initialPosition !== undefined) {
        validatePosition(
          { file: site.file, position: site.initialPosition },
          "form field initial value",
        );
      }
    }
  }
  if (!Array.isArray(request.validations)) {
    throw new TypeError("V2 authoring validation sites must be an array");
  }
  if (!Array.isArray(request.standardValidations)) {
    throw new TypeError("V2 authoring Standard Schema validation sites must be an array");
  }
  if (!Array.isArray(request.packageInvocations)) {
    throw new TypeError("V2 authoring package invocation sites must be an array");
  }
  const standardValidationIds = new Set();
  for (const site of request.standardValidations) {
    if (!site || typeof site.id !== "string" || !site.id
      || standardValidationIds.has(site.id)
      || typeof site.moduleSpecifier !== "string" || !site.moduleSpecifier
      || typeof site.exportName !== "string" || !site.exportName) {
      throw new TypeError("V2 authoring Standard Schema validation sites require unique ids and module exports");
    }
    standardValidationIds.add(site.id);
    validatePosition(site, "Standard Schema validation site");
  }
  const packageInvocationIds = new Set();
  for (const site of request.packageInvocations) {
    if (!site || typeof site.id !== "string" || !site.id
      || packageInvocationIds.has(site.id)
      || typeof site.moduleSpecifier !== "string" || !site.moduleSpecifier
      || typeof site.exportName !== "string" || !site.exportName
      || !Number.isInteger(site.importPosition)) {
      throw new TypeError("V2 authoring package invocation sites require unique ids and named module exports");
    }
    packageInvocationIds.add(site.id);
    validatePosition(site, "package invocation site");
  }
  const validationIds = new Set();
  for (const site of request.validations) {
    if (!site || typeof site.id !== "string" || !site.id || validationIds.has(site.id)) {
      throw new TypeError("V2 authoring validation sites require unique non-empty ids");
    }
    validationIds.add(site.id);
    validatePosition(site, "validation site");
  }
  if (!Array.isArray(request.environmentPublic)) {
    throw new TypeError("V2 authoring environment public sites must be an array");
  }
  if (request.environmentPublic.length > 0 && request.canonical.environment === undefined) {
    throw new TypeError("V2 authoring environment public sites require a canonical environment position");
  }
  const ids = new Set();
  for (const site of request.environmentPublic) {
    if (!site || typeof site.id !== "string" || !site.id || ids.has(site.id)) {
      throw new TypeError("V2 authoring environment public sites require unique non-empty ids");
    }
    ids.add(site.id);
    validatePosition({ file: site.file, position: site.objectPosition }, "environment public object site");
    validatePosition({ file: site.file, position: site.propertyPosition }, "environment public property site");
  }
}

function classifyFormFieldValue(type) {
  const identity = type?.arrayElement?.symbol?.identity;
  if (identity?.name !== "File") return undefined;
  if (!identity.declarationModules.some(module => /(?:^|\/)lib\.dom\.d\.ts$/.test(module))) {
    return undefined;
  }
  return "file_array";
}

function validatePosition(value, label) {
  if (!value || typeof value.file !== "string" || !Number.isInteger(value.position)) {
    throw new TypeError(`V2 authoring ${label} requires file and integer position`);
  }
}

function stripPrefix(value, prefix) {
  return value.startsWith(prefix) ? value.slice(prefix.length) : value;
}

function resolvedIdentity(symbol) {
  return symbol?.aliasTarget?.identity ?? symbol?.identity;
}

function sameDeclarationModules(left, right) {
  if (!left || !right) return false;
  return JSON.stringify([...left.declarationModules].sort())
    === JSON.stringify([...right.declarationModules].sort());
}
