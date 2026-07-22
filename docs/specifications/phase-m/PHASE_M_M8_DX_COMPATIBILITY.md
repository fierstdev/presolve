# Phase M M8 compiler-backed DX and compatibility

**Status:** M8 implementation authority.

## Explanation surface

Phase M adds no framework `explain` command and no JavaScript source analysis.
The supported explanation surface is the existing canonical CLI inspection
product:

```sh
presolve explain --inspect src/Component.tsx --format json
```

Framework documentation may label records for component, State, Action,
Computed, Effect, Context, Slot, Form, Field, and submission concepts, but it
must project the compiler result unchanged. It cannot derive dependencies,
reconstruct identities, decode artifacts, or replace compiler diagnostics.

## Examples

The Phase M examples are compiler source, not framework templates:

| Example | Conformance family | Canonical proof |
| --- | --- | --- |
| `examples/counter` | Component, State, Action, direct binding | M4 artifact/public browser proof |
| `framework/tests/computed-types` | Computed | M5 compiler/browser proof |
| `framework/tests/context-types` | static Context and qualified designators | M6-B compiler/browser proof |
| `framework/tests/forms-types` | explicit Form host and submission | M7 browser proof |
| `framework/tests/forms-resume-types` | Form resume/fail-closed fallback | M7 browser proof |

They require explicit source/configuration inputs. M8 adds no project discovery,
generator, package-manager integration, or dev server.

## Compatibility matrix

| Input | Status | Evidence |
| --- | --- | --- |
| Presolve compiler language | Supported | M0–M7 fixtures and canonical CLI checks |
| Compiler artifacts/runtime/resume | Supported opaquely | M4–M7 real-browser products; framework has no decoder |
| TypeScript 7.0 native CLI | Supported | declaration fixtures for M2, M5, M6, and M7 |
| TypeScript 7.1 | Deferred | install the pinned 7.1 toolchain and rerun every declaration fixture before declaring support |

An unsupported tuple fails closed. No compatibility layer may reinterpret old
source syntax, artifact bytes, or compiler diagnostics.

## Boundary

M8 excludes a framework language service, LSP implementation, source parser,
JSX transform, router, dev server, HMR, scaffolder, deployment adapter, and
package publication. Existing compiler-owned WASM/LSP products remain the only
future editor integration authority.
