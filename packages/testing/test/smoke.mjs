import { declaredTest, equalCanonicalBytes } from "../src/index.js";

if (!equalCanonicalBytes(new Uint8Array([1, 2]), new Uint8Array([1, 2]))) throw new Error("equal bytes must compare");
if (equalCanonicalBytes(new Uint8Array([1]), new Uint8Array([2]))) throw new Error("different bytes must not compare");
const test = declaredTest({ name: "compiler", command: "cargo test -p presolve-compiler --lib", lane: "deterministic" });
if (test.lane !== "deterministic" || !Object.isFrozen(test)) throw new Error("declared test metadata must be immutable");
