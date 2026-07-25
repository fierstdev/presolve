import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import test from "node:test";
import { resolve } from "node:path";

import {
  analyzeTypeScriptProject,
  PRIMARY_TYPESCRIPT_VERSION,
  TYPESCRIPT_SEMANTIC_AUTHORITY_SCHEMA_VERSION,
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
      modules: [{ id: "package-export", file, position: position("@compat/library") }],
    },
  });

  assert.equal(result.schemaVersion, TYPESCRIPT_SEMANTIC_AUTHORITY_SCHEMA_VERSION);
  assert.equal(result.typeScriptVersion, PRIMARY_TYPESCRIPT_VERSION);
  assert.equal(result.diagnostics.length, 0);
  assert.equal(result.symbols[0].symbol.name, "overload");
  assert.equal(result.symbols[0].symbol.aliasTarget.name, "overload");
  assert.match(result.symbols[0].symbol.aliasTarget.declarationPaths[0], /@compat\/library\/types\/index\.d\.ts$/);
  assert.equal(result.types[0].type.text, "Box<LocalAlias>");
  assert.equal(result.contextualTypes[0].type.text, "number");
  assert.deepEqual(result.signatures[0].signature.parameterTypes.map(parameter => parameter.type.text), ["string"]);
  assert.equal(result.signatures[0].signature.returnType.text, "number");
  assert.equal(result.assignability[0].assignable, true);
  assert.equal(result.assignability[1].assignable, false);
  assert.match(result.modules[0].module.declarationPaths[0], /@compat\/library\/types\/index\.d\.ts$/);
});

test("the authority adapter preserves native TypeScript diagnostics", async () => {
  const diagnostics = await analyzeTypeScriptProject({
    configFile: resolve(root, "tests/typescript-compatibility/diagnostics/tsconfig.json"),
  });
  assert.deepEqual(diagnostics.diagnostics.map(diagnostic => diagnostic.code), [2322, 2345]);
  assert(diagnostics.diagnostics.every(diagnostic => diagnostic.source === "semantic"));
});
