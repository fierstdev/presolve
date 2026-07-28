import assert from "node:assert/strict";
import test from "node:test";

import { unsupportedPlatformDiagnostic } from "../bin/diagnostic.mjs";
import manifest from "../package.json" with { type: "json" };

test("unsupported-platform diagnostics identify the installed CLI release", () => {
  assert.equal(
    unsupportedPlatformDiagnostic(manifest.version, "linux", "arm64"),
    `Presolve ${manifest.version} does not include a CLI binary for linux-arm64. ` +
      "See https://github.com/fierstdev/presolve#supported-platforms."
  );
});
