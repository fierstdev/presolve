EdgeZero Agent Handoff

Repository state

* Branch: main
* Latest commit: feat: support string state literals
* Working tree: clean after committing this slice
* Date: 2026-07-09 19:05:54 PDT

Last completed slice

* Slice: 6E-A - String literals
* Summary: State initializers now accept string literals and preserve the semantic value through parser, component graph, template graph, HTML, manifest JSON, and browser runtime initialization.
* Key files: crates/ezc_parser/src/oxc_adapter.rs, crates/ezc_core/src/runtime_codegen.rs, crates/ezc_core/src/lib.rs, crates/ezc_cli/tests/explain.rs, crates/ezc_cli/tests/runtime_browser.rs
* New behavior: `state("Austin & <Zero>")` records `Austin & <Zero>` without source quotes, static HTML escapes it as text, manifest JSON emits it as a JSON string, and `window.__EDGEZERO__` initializes runtime state with the string instead of `Number(...)`.
* Tests added or changed: parser string literal assertion, core HTML/manifest assertions, CLI html/template/manifest fixture assertions, runtime-codegen assertion, and browser e2e string-state boot probe.
* Fixtures added or changed: fixtures/0006-string-state/input/StringGreeting.tsx plus expected HTML, template, and manifest outputs

Current in-progress slice

* Slice: 6E-B - Boolean literals
* Status: Not started
* Completed: None
* Remaining: Support `state(true)` and `state(false)`, preserve boolean type in compiler models and manifest, define text rendering as `true` / `false`, and add static plus browser coverage.

Verification

* cargo fmt --all --check: pass
* cargo test -p ezc_parser: pass
* cargo test -p ezc_core: pass
* cargo test -p ezc_cli --test explain: pass
* cargo test -p ezc_cli --test runtime_browser -- --nocapture: pass
* cargo test --workspace: pass
* pnpm test:e2e: pass
* just e2e: not run locally because `just` is not installed in this shell
* Known failures: `cargo clippy --workspace --all-targets -- -D warnings` still fails on pre-existing warnings outside this slice, so the new CI workflow intentionally does not add a clippy gate yet.

Architecture decisions made

* Decision: Keep 6E-A on the existing `Option<String>` initial-value path and defer the compiler-owned typed serializable value enum to 6E-D.
* Reason: The 6E-A requirement is string literal support, and the existing manifest field already serializes string values as JSON strings.
* Tradeoff: Numeric initial state now enters the browser store as the manifest string until numeric actions coerce with `Number(...)`; visible rendering and increment behavior remain covered.
* Follow-up: Slice 6E-D should replace this temporary scalar path with a typed `SerializableValue` model before arrays and objects.

Known limitations

* Item: Only `this.<field>++` is recognized as an action. `this.count += 1`, decrement, assignment, and multi-step plans are intentionally deferred.
* Item: The browser runtime supports only delegated click events, increment actions, numeric/string initial state, and binding callback text updates.
* Item: Boolean and null state literals are still unsupported until 6E-B and 6E-C.
* Item: Browser e2e requires a local Chrome binary or `EDGEZERO_CHROME=/path/to/chrome`.

Exact next step

Start Slice 6E-B by teaching the parser to capture boolean `state(...)` arguments, choosing the minimal typed representation needed before 6E-D, and adding a boolean fixture that asserts HTML text renders as `true` / `false` and manifest/runtime preserve booleans.

Useful commands

* `cargo fmt --all --check`
* `cargo test -p ezc_parser`
* `cargo test -p ezc_core`
* `cargo test -p ezc_cli`
* `cargo test -p ezc_cli --test runtime_browser -- --nocapture`
* `pnpm test:e2e`
* `cargo test --workspace`
* `cargo run -p ezc_cli -- build fixtures/0005-double-binding-counter/input/DoubleBindingCounter.tsx --out target/ezc-manual/double-binding-counter`

Changed but uncommitted files

* None
