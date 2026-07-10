EdgeZero Agent Handoff

Repository state

* Branch: main
* Latest commit: runtime: version manifest contract
* Working tree: clean after committing this slice
* Date: 2026-07-10 06:37:16 PDT

Last completed slice

* Slice: 6G - Runtime contract versioning and diagnostics
* Summary: Added manifest schema versioning, runtime version exposure, schema compatibility checks, stable runtime diagnostic codes, and fatal boot diagnostics.
* Key files: crates/ezc_core/src/template_manifest.rs, crates/ezc_core/src/runtime_codegen.rs, crates/ezc_cli/tests/runtime_browser.rs, docs/runtime-contract.md
* New behavior: Manifests serialize with `"schema_version": 1`; runtime exposes `runtime_version`, `supported_schema_version`, and `diagnostics` in `window.__EDGEZERO__`; missing/invalid/future manifests fail closed with `data-ez-runtime="error"`.
* Tests added or changed: manifest schema assertions, fixture manifest updates, runtime string assertions, valid-boot diagnostics assertions, and real-browser fatal diagnostic probes.
* Fixtures added or changed: all `fixtures/*/expected/manifest.json` files now include `schema_version`.

Current in-progress slice

* Slice: 7A - Preserve static JSX attributes
* Status: Not started
* Completed: None
* Remaining: Replace parser attribute summaries with structured attributes, preserve ordinary static attributes through graph/HTML output, and add duplicate/static attribute diagnostics.

Verification

* cargo fmt --all --check: pass
* cargo test -p ezc_parser: pass
* cargo test -p ezc_core: pass
* cargo test -p ezc_cli --test explain: pass
* cargo test -p ezc_cli --test runtime_browser -- --nocapture: pass
* CI=true cargo test -p ezc_cli --test runtime_browser -- --nocapture: pass
* cargo clippy --workspace --all-targets -- -D warnings: pass
* cargo test --workspace: pass
* pnpm test:e2e: pass
* just e2e: pass

Architecture decisions made

* Decision: Manifest schema compatibility is exact-match only for now: runtime accepts `schema_version === 1` and rejects missing, older, or future schemas.
* Reason: There is no migration layer yet, so exact matching is clearer than silently accepting unknown manifests.
* Tradeoff: Older manifests without `schema_version` now fail at runtime until rebuilt with the current compiler.
* Follow-up: Slice 7A should preserve this manifest contract while adding structured static attribute data where needed.

Known limitations

* Item: Only `this.<field>++`, `this.<field>--`, `this.<field> += <literal>`, `this.<field> -= <literal>`, `this.<field> = <literal>`, and `this.<field> = !this.<field>` are recognized as action steps.
* Item: The browser runtime supports only delegated click events, ordered closed action steps, numeric/string/boolean/null initial state, and binding callback text updates.
* Item: The serializable value model is still primitive-only until later array/object slices.
* Item: Runtime schema compatibility is exact-match only; no backward/forward manifest migration exists yet.
* Item: Browser e2e requires a local Chrome binary or `EDGEZERO_CHROME=/path/to/chrome`.
* Item: GitHub Actions Chrome e2e repair is locally validated with `CI=true` but not yet confirmed by a new hosted run.

Exact next step

Start Slice 7A by replacing parser attribute string summaries with structured static JSX attributes and preserving ordinary attributes in generated HTML.

Useful commands

* `cargo fmt --all --check`
* `cargo test -p ezc_parser`
* `cargo test -p ezc_core`
* `cargo test -p ezc_cli`
* `cargo test -p ezc_cli --test runtime_browser -- --nocapture`
* `cargo clippy --workspace --all-targets -- -D warnings`
* `pnpm test:e2e`
* `just e2e`
* `cargo test --workspace`
* `cargo run -p ezc_cli -- build fixtures/0005-double-binding-counter/input/DoubleBindingCounter.tsx --out target/ezc-manual/double-binding-counter`
* `cargo run -p ezc_cli -- build fixtures/0009-decrement-counter/input/DecrementCounter.tsx --out target/ezc-manual/decrement-counter`
* `cargo run -p ezc_cli -- build fixtures/0010-add-subtract-assign/input/StepCounter.tsx --out target/ezc-manual/step-counter`
* `cargo run -p ezc_cli -- build fixtures/0011-direct-assignment/input/ResetCounter.tsx --out target/ezc-manual/reset-counter`
* `cargo run -p ezc_cli -- build fixtures/0012-boolean-toggle/input/ToggleFlag.tsx --out target/ezc-manual/toggle-flag`
* `cargo run -p ezc_cli -- build fixtures/0013-multi-step-action/input/BatchActionCounter.tsx --out target/ezc-manual/batch-action-counter`
* `cargo run -p ezc_cli -- build fixtures/0004-nested-jsx/input/NestedCounter.tsx --out target/ezc-manual/runtime-contract`

Changed but uncommitted files

* None
