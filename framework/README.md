# Presolve Framework

This private workspace is the Phase M framework boundary. It exposes
compiler-owned authoring forms through TypeScript declarations and focused
conformance fixtures. It does not own a parser, source transform, compiler
adapter, runtime, renderer, scheduler, artifact decoder, or generated output.

M2 contains only `@presolve/framework-types`: ambient declarations needed for
the existing Counter source. M6-B adds typed static Context declarations and
compiler-resolved qualified Context designators as a documented production
compiler-language contract, not a framework shim. M10-A adds the already
compiler-admitted Resource declaration/type surface; package contracts and
runtime mappings remain explicit compiler build inputs. Later slices may extend
this workspace only when a frozen compiler capability and Phase M conformance
contract authorize the exact form.

M10-B makes the bounded compiler-supported JSX aliases, accessibility
attributes, and Action event attributes visible to TypeScript without adding a
JSX transform or framework DOM vocabulary. All remaining JSX admission is still
decided by the compiler.
