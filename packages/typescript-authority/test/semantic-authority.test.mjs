import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import test from "node:test";
import { resolve } from "node:path";
import { spawnSync } from "node:child_process";

import {
  analyzeTypeScriptProject,
  analyzeV2Authoring,
  classifyResolvedComponentHeritage,
  classifyResolvedIntrinsic,
  createCanonicalIntrinsicRegistry,
  PRIMARY_TYPESCRIPT_VERSION,
  TYPESCRIPT_SEMANTIC_AUTHORITY_SCHEMA_VERSION,
  V2_AUTHORED_AUTHORITY_SCHEMA_VERSION,
} from "../src/index.js";

const root = resolve(import.meta.dirname, "../../..");
const configFile = resolve(root, "tests/typescript-compatibility/tsconfig.json");
const file = resolve(root, "tests/typescript-compatibility/src/compatibility.tsx");
const source = readFileSync(file, "utf8");

function position(text, occurrence = 0) {
  let start = -1;
  for (let index = 0; index <= occurrence; index += 1) {
    start = source.indexOf(text, start + 1);
  }
  assert.notEqual(start, -1, `missing ${text}`);
  return start;
}

test("the authority adapter owns TypeScript semantic queries", async () => {
  const result = await analyzeTypeScriptProject({
    configFile,
    queries: {
      symbols: [{ id: "overload", file, position: position('overload("typed")') }],
      types: [{ id: "generic", file, position: position("box: Box<LocalAlias>") }],
      contextualTypes: [{ id: "callback", file, position: position("value => value.length") }],
      signatures: [{ id: "call", file, position: position('overload("typed")') }],
      assignability: [
        {
          id: "number-to-number",
          source: { file, position: position("selected =") },
          target: { file, position: position("selected =") },
        },
        {
          id: "string-to-number",
          source: { file, position: position("packageValue;") },
          target: { file, position: position("selected =") },
        },
      ],
      modules: [
        { id: "package-export", file, position: position("@compat/library") },
        { id: "re-export", file, position: position("@compat/library", 1) },
      ],
    },
  });

  assert.equal(result.schemaVersion, TYPESCRIPT_SEMANTIC_AUTHORITY_SCHEMA_VERSION);
  assert.equal(result.typeScriptVersion, PRIMARY_TYPESCRIPT_VERSION);
  assert.equal(result.diagnostics.length, 0);
  assert.equal(result.symbols[0].symbol.name, "overload");
  assert.equal(result.symbols[0].symbol.aliasTarget.name, "overload");
  assert.match(result.symbols[0].symbol.aliasTarget.declarationPaths[0], /@compat\/library\/types\/index\.d\.ts$/);
  assert.deepEqual(result.symbols[0].symbol.aliasTarget.identity.declarationModules, ["node_modules/@compat/library/types/index.d.ts"]);
  assert.equal(result.types[0].type.text, "Box<LocalAlias>");
  assert.equal(result.contextualTypes[0].type.text, "number");
  assert.deepEqual(result.signatures[0].signature.parameterTypes.map(parameter => parameter.type.text), ["string"]);
  assert.equal(result.signatures[0].signature.returnType.text, "number");
  assert.equal(result.assignability[0].assignable, true);
  assert.equal(result.assignability[1].assignable, false);
  assert.match(result.modules[0].module.declarationPaths[0], /@compat\/library\/types\/index\.d\.ts$/);
  assert.equal(result.modules[0].module.specifier, "@compat/library");
  assert.deepEqual(result.modules[0].module.resolvedModulePaths, ["node_modules/@compat/library/types/index.d.ts"]);
  assert.deepEqual(result.modules[1].module.resolvedModulePaths, result.modules[0].module.resolvedModulePaths);
});

test("the authority adapter preserves native TypeScript diagnostics", async () => {
  const diagnostics = await analyzeTypeScriptProject({
    configFile: resolve(root, "tests/typescript-compatibility/diagnostics/tsconfig.json"),
  });
  assert.deepEqual(diagnostics.diagnostics.map(diagnostic => diagnostic.code), [2322, 2345]);
  assert(diagnostics.diagnostics.every(diagnostic => diagnostic.source === "semantic"));
});

test("canonical intrinsics classify resolved exports rather than local spellings", async () => {
  const frameworkFile = resolve(root, "tests/framework-public-api/src/PublicCounter.tsx");
  const frameworkSource = readFileSync(frameworkFile, "utf8");
  const frameworkResult = await analyzeTypeScriptProject({
    configFile: resolve(root, "tests/framework-public-api/tsconfig.json"),
    queries: {
      symbols: [
        { id: "component-import", file: frameworkFile, position: frameworkSource.indexOf("component,") },
        { id: "component-use", file: frameworkFile, position: frameworkSource.indexOf("@component()") + 1 },
      ],
    },
  });
  const registry = createCanonicalIntrinsicRegistry([
    { kind: "component", symbol: frameworkResult.symbols[0].symbol },
  ]);
  assert.equal(classifyResolvedIntrinsic(registry, frameworkResult.symbols[1].symbol)?.kind, "component");
  assert.equal(classifyResolvedIntrinsic(registry, { identity: { name: "component", flags: 0, declarationModules: [] } }), undefined);
});

test("component heritage preserves aliases and indirect bases for registry classification", async () => {
  const frameworkFile = resolve(root, "tests/framework-public-api/src/V2Counter.tsx");
  const frameworkSource = readFileSync(frameworkFile, "utf8");
  const result = await analyzeTypeScriptProject({
    configFile: resolve(root, "tests/framework-public-api/tsconfig.json"),
    queries: {
      symbols: [{ id: "component-import", file: frameworkFile, position: frameworkSource.indexOf("Component") }],
      componentHeritage: [{ id: "counter", file: frameworkFile, position: frameworkSource.indexOf("V2Counter extends") }],
    },
  });
  const registry = createCanonicalIntrinsicRegistry([
    { kind: "component", symbol: result.symbols[0].symbol },
  ]);
  const heritage = result.componentHeritage[0];
  assert.deepEqual(heritage.bases.map(base => base.name), ["V2CounterBase", "Component"]);
  assert.equal(classifyResolvedComponentHeritage(registry, heritage.bases)?.kind, "component");
});

test("the V2 authoring bridge resolves canonical component, State, Action, Effect, Slot, and Environment evidence", async () => {
  const frameworkFile = resolve(root, "tests/framework-public-api/src/V2Counter.tsx");
  const frameworkSource = readFileSync(frameworkFile, "utf8");
  const result = await analyzeV2Authoring({
    schemaVersion: V2_AUTHORED_AUTHORITY_SCHEMA_VERSION,
    configFile: resolve(root, "tests/framework-public-api/tsconfig.json"),
    canonical: {
      component: { file: frameworkFile, position: frameworkSource.indexOf("Component") },
      state: { file: frameworkFile, position: frameworkSource.indexOf("state") },
      action: { file: frameworkFile, position: frameworkSource.indexOf("action") },
      effect: { file: frameworkFile, position: frameworkSource.indexOf("effect") },
      slot: { file: frameworkFile, position: frameworkSource.indexOf("slot") },
      environment: { file: frameworkFile, position: frameworkSource.indexOf("runtimeEnvironment") },
      validationRules: [],
    },
    components: [{ id: "counter", file: frameworkFile, position: frameworkSource.indexOf("V2Counter extends") }],
    states: [{ id: "count", file: frameworkFile, position: frameworkSource.indexOf("state(0)") }],
    actions: [{ id: "increment", file: frameworkFile, position: frameworkSource.indexOf("action(()") }],
    effects: [{ id: "syncTitle", file: frameworkFile, position: frameworkSource.indexOf("effect(()") }],
    slots: [{ id: "children", file: frameworkFile, position: frameworkSource.indexOf("slot()") }],
    forms: [],
    formFields: [],
    validations: [],
    standardValidations: [],
    packageInvocations: [],
    environmentPublic: [
      {
        id: "application-name",
        file: frameworkFile,
        objectPosition: frameworkSource.indexOf("runtimeEnvironment.public"),
        propertyPosition: frameworkSource.indexOf("runtimeEnvironment.public") + "runtimeEnvironment.".length,
      },
      {
        id: "lookalike",
        file: frameworkFile,
        objectPosition: frameworkSource.indexOf("lookalikeEnvironment.public"),
        propertyPosition: frameworkSource.indexOf("lookalikeEnvironment.public") + "lookalikeEnvironment.".length,
      },
    ],
  });
  assert.equal(result.schemaVersion, V2_AUTHORED_AUTHORITY_SCHEMA_VERSION);
  assert.equal(result.diagnostics.length, 0);
  assert.deepEqual(result.components.map(entry => entry.id), ["counter"]);
  assert.deepEqual(result.states.map(entry => entry.id), ["count"]);
  assert.deepEqual(result.actions.map(entry => entry.id), ["increment"]);
  assert.deepEqual(result.effects.map(entry => entry.id), ["syncTitle"]);
  assert.deepEqual(result.slots.map(entry => entry.id), ["children"]);
  assert.deepEqual(result.environmentPublic.map(entry => entry.id), ["application-name"]);
  assert.equal(result.components[0].identity.name, "Component");
  assert.equal(result.states[0].identity.name, "state");
  assert.equal(result.actions[0].identity.name, "action");
  assert.equal(result.effects[0].identity.name, "effect");
  assert.equal(result.slots[0].identity.name, "slot");
  assert.equal(result.environmentPublic[0].identity.name, "public");
});

test("the V2 authoring bridge supports a component-only discovery phase", async () => {
  const frameworkFile = resolve(root, "tests/framework-public-api/src/V2Counter.tsx");
  const frameworkSource = readFileSync(frameworkFile, "utf8");
  const result = await analyzeV2Authoring({
    schemaVersion: V2_AUTHORED_AUTHORITY_SCHEMA_VERSION,
    configFile: resolve(root, "tests/framework-public-api/tsconfig.json"),
    canonical: {
      component: { file: frameworkFile, position: frameworkSource.indexOf("Component") },
      validationRules: [],
    },
    components: [{ id: "counter", file: frameworkFile, position: frameworkSource.indexOf("V2Counter extends") }],
    states: [],
    actions: [],
    effects: [],
    slots: [],
    forms: [],
    formFields: [],
    validations: [],
    standardValidations: [],
    packageInvocations: [],
    environmentPublic: [],
  });
  assert.deepEqual(result.components.map(entry => entry.id), ["counter"]);
  assert.deepEqual(result.states, []);
  assert.deepEqual(result.actions, []);
  assert.deepEqual(result.effects, []);
});

test("the V2 authoring bridge proves exact named-import package invocations", async () => {
  const packageFile = resolve(root, "tests/framework-public-api/src/V2Package.tsx");
  const packageSource = readFileSync(packageFile, "utf8");
  const importedCall = packageSource.lastIndexOf("emitMetric(category");
  const importedAsyncCall = packageSource.lastIndexOf("emitMetricAsync(category");
  const localCall = packageSource.lastIndexOf("recordMetric(category");
  const result = await analyzeV2Authoring({
    schemaVersion: V2_AUTHORED_AUTHORITY_SCHEMA_VERSION,
    configFile: resolve(root, "tests/framework-public-api/tsconfig.json"),
    canonical: {
      component: { file: packageFile, position: packageSource.indexOf("Component") },
      action: { file: packageFile, position: packageSource.indexOf("action") },
      validationRules: [],
    },
    components: [{ id: "package", file: packageFile, position: packageSource.indexOf("V2Package extends") }],
    states: [],
    actions: [
      { id: "send", file: packageFile, position: packageSource.indexOf("action((category") },
      { id: "async", file: packageFile, position: packageSource.indexOf("action(async") },
      { id: "local", file: packageFile, position: packageSource.lastIndexOf("action((") },
    ],
    effects: [],
    slots: [],
    forms: [],
    formFields: [],
    validations: [],
    standardValidations: [],
    packageInvocations: [
      {
        id: "imported",
        file: packageFile,
        position: importedCall,
        importPosition: packageSource.indexOf("emitMetric"),
        moduleSpecifier: "./V2PackageHelper.js",
        exportName: "recordMetric",
        argumentTypes: ["string", "number"],
        completion: "synchronous",
      },
      {
        id: "wrong-signature",
        file: packageFile,
        position: importedCall,
        importPosition: packageSource.indexOf("emitMetric"),
        moduleSpecifier: "./V2PackageHelper.js",
        exportName: "recordMetric",
        argumentTypes: ["string", "boolean"],
        completion: "synchronous",
      },
      {
        id: "imported-async",
        file: packageFile,
        position: importedAsyncCall,
        importPosition: packageSource.indexOf("recordMetricAsync as emitMetricAsync"),
        moduleSpecifier: "./V2PackageHelper.js",
        exportName: "recordMetricAsync",
        argumentTypes: ["string"],
        completion: "promise",
        signalPosition: packageSource.indexOf("AbortSignal"),
      },
      {
        id: "lookalike",
        file: packageFile,
        position: localCall,
        importPosition: packageSource.indexOf("emitMetric"),
        moduleSpecifier: "./V2PackageHelper.js",
        exportName: "recordMetric",
        argumentTypes: ["string", "number"],
        completion: "synchronous",
      },
    ],
    environmentPublic: [],
  });
  assert.equal(result.diagnostics.length, 0);
  assert.deepEqual(result.packageInvocations.map(entry => entry.id), ["imported", "imported-async"]);
  assert.equal(result.packageInvocations[0].identity.name, "recordMetric");
  assert.equal(result.packageInvocations[0].moduleSpecifier, "./V2PackageHelper.js");
  assert.deepEqual(result.packageInvocations[0].argumentTypes, ["string", "number"]);
  assert.equal(result.packageInvocations[0].completion, "synchronous");
  assert.equal(result.packageInvocations[0].injectAbortSignal, false);
  assert.deepEqual(result.packageInvocations[1].argumentTypes, ["string"]);
  assert.equal(result.packageInvocations[1].completion, "promise");
  assert.equal(result.packageInvocations[1].injectAbortSignal, true);
});

test("the V2 authoring bridge resolves canonical decorator-free Form evidence", async () => {
  const formFile = resolve(root, "tests/framework-public-api/src/V2Form.tsx");
  const formSource = readFileSync(formFile, "utf8");
  const result = await analyzeV2Authoring({
    schemaVersion: V2_AUTHORED_AUTHORITY_SCHEMA_VERSION,
    configFile: resolve(root, "tests/framework-public-api/tsconfig.json"),
    canonical: {
      component: { file: formFile, position: formSource.indexOf("Component") },
      defineForm: { file: formFile, position: formSource.indexOf("defineForm") },
      field: { file: formFile, position: formSource.indexOf("field") },
      validationRules: [{
        name: "required",
        file: formFile,
        position: formSource.indexOf("required"),
      }],
    },
    components: [{
      id: "profile",
      file: formFile,
      position: formSource.indexOf("V2ProfileForm extends"),
    }],
    states: [],
    actions: [],
    effects: [],
    slots: [],
    forms: [{
      id: "profile-form",
      file: formFile,
      position: formSource.indexOf("defineForm({"),
    }],
    formFields: [{
      id: "profile-name",
      file: formFile,
      position: formSource.indexOf("field({"),
      initialPosition: formSource.indexOf('initial: ""') + "initial: ".length,
    }, {
      id: "profile-attachments",
      file: formFile,
      position: formSource.indexOf("field<Uploads>"),
      initialPosition: formSource.indexOf("initial: []") + "initial: ".length,
    }],
    validations: [{
      id: "profile-name-required",
      file: formFile,
      position: formSource.indexOf("required()"),
    }],
    standardValidations: [{
      id: "profile-name-standard",
      file: formFile,
      position: formSource.lastIndexOf("displayNameSchema"),
      moduleSpecifier: "./V2Schemas.js",
      exportName: "displayNameSchema",
    }],
    packageInvocations: [],
    environmentPublic: [],
  });
  assert.deepEqual(result.forms.map(entry => entry.id), ["profile-form"]);
  assert.equal(result.forms[0].identity.name, "defineForm");
  assert.deepEqual(result.formFields.map(entry => entry.id), ["profile-name", "profile-attachments"]);
  assert.equal(result.formFields[0].identity.name, "field");
  assert.equal(result.formFields[0].valueClassification, undefined);
  assert.equal(result.formFields[1].valueClassification, "file_array");
  assert.deepEqual(result.validations.map(entry => entry.id), ["profile-name-required"]);
  assert.equal(result.validations[0].identity.name, "required");
  assert.deepEqual(result.standardValidations, [{
    id: "profile-name-standard",
    identity: {
      name: "displayNameSchema",
      flags: result.standardValidations[0].identity.flags,
      declarationModules: ["src/v2schemas.ts"],
    },
    moduleSpecifier: "./V2Schemas.js",
    exportName: "displayNameSchema",
    inputType: "string",
    outputType: "string",
  }]);
});

test("the V2 authoring bridge recognizes a direct Component heritage-expression query", async () => {
  const frameworkFile = resolve(root, "tests/framework-public-api/src/V2Counter.tsx");
  const frameworkSource = readFileSync(frameworkFile, "utf8");
  const directFile = resolve(root, "tests/framework-public-api/src/DirectV2.tsx");
  const directSource = readFileSync(directFile, "utf8");
  const result = await analyzeV2Authoring({
    schemaVersion: V2_AUTHORED_AUTHORITY_SCHEMA_VERSION,
    configFile: resolve(root, "tests/framework-public-api/tsconfig.json"),
    canonical: {
      component: { file: frameworkFile, position: frameworkSource.indexOf("Component") },
      validationRules: [],
    },
    components: [{ id: "direct", file: directFile, position: directSource.lastIndexOf("Component") }],
    states: [],
    actions: [],
    effects: [],
    slots: [],
    forms: [],
    formFields: [],
    validations: [],
    standardValidations: [],
    packageInvocations: [],
    environmentPublic: [],
  });
  assert.deepEqual(result.components.map(entry => entry.id), ["direct"]);
  assert.equal(result.components[0].identity.name, "Component");
});

test("the V2 authoring bridge resolves environment evidence without a component query", async () => {
  const frameworkFile = resolve(root, "tests/framework-public-api/src/V2Counter.tsx");
  const frameworkSource = readFileSync(frameworkFile, "utf8");
  const result = await analyzeV2Authoring({
    schemaVersion: V2_AUTHORED_AUTHORITY_SCHEMA_VERSION,
    configFile: resolve(root, "tests/framework-public-api/tsconfig.json"),
    canonical: {
      environment: { file: frameworkFile, position: frameworkSource.indexOf("runtimeEnvironment") },
      validationRules: [],
    },
    components: [],
    states: [],
    actions: [],
    effects: [],
    slots: [],
    forms: [],
    formFields: [],
    validations: [],
    standardValidations: [],
    packageInvocations: [],
    environmentPublic: [{
      id: "application-name",
      file: frameworkFile,
      objectPosition: frameworkSource.indexOf("runtimeEnvironment.public"),
      propertyPosition: frameworkSource.indexOf("runtimeEnvironment.public") + "runtimeEnvironment.".length,
    }],
  });
  assert.deepEqual(result.components, []);
  assert.deepEqual(result.environmentPublic.map(entry => entry.id), ["application-name"]);
});

test("the V2 authoring executable speaks the versioned stdin/stdout bridge protocol", () => {
  const frameworkFile = resolve(root, "tests/framework-public-api/src/V2Counter.tsx");
  const frameworkSource = readFileSync(frameworkFile, "utf8");
  const result = spawnSync(
    process.execPath,
    [resolve(import.meta.dirname, "../bin/presolve-typescript-authority.mjs")],
    {
      input: JSON.stringify({
        schemaVersion: V2_AUTHORED_AUTHORITY_SCHEMA_VERSION,
        configFile: resolve(root, "tests/framework-public-api/tsconfig.json"),
        canonical: {
          component: { file: frameworkFile, position: frameworkSource.indexOf("Component") },
          state: { file: frameworkFile, position: frameworkSource.indexOf("state") },
          action: { file: frameworkFile, position: frameworkSource.indexOf("action") },
          effect: { file: frameworkFile, position: frameworkSource.indexOf("effect") },
          slot: { file: frameworkFile, position: frameworkSource.indexOf("slot") },
          environment: { file: frameworkFile, position: frameworkSource.indexOf("runtimeEnvironment") },
          validationRules: [],
        },
        components: [{ id: "counter", file: frameworkFile, position: frameworkSource.indexOf("V2Counter extends") }],
        states: [{ id: "count", file: frameworkFile, position: frameworkSource.indexOf("state(0)") }],
        actions: [{ id: "increment", file: frameworkFile, position: frameworkSource.indexOf("action(()") }],
        effects: [{ id: "syncTitle", file: frameworkFile, position: frameworkSource.indexOf("effect(()") }],
        slots: [{ id: "children", file: frameworkFile, position: frameworkSource.indexOf("slot()") }],
        forms: [],
        formFields: [],
        validations: [],
        standardValidations: [],
        packageInvocations: [],
        environmentPublic: [],
      }),
      encoding: "utf8",
    },
  );
  assert.equal(result.status, 0, result.stderr);
  const response = JSON.parse(result.stdout);
  assert.equal(response.schemaVersion, V2_AUTHORED_AUTHORITY_SCHEMA_VERSION);
  assert.deepEqual(response.components.map(entry => entry.id), ["counter"]);
  assert.deepEqual(response.slots.map(entry => entry.id), ["children"]);
});
