# Presolve Framework

This private workspace is the Phase M framework boundary. It exposes frozen
compiler authoring forms through TypeScript declarations and focused
conformance fixtures. It does not own a parser, source transform, compiler
adapter, runtime, renderer, scheduler, artifact decoder, or generated output.

M2 contains only `@presolve/framework-types`: ambient declarations needed for
the existing Counter source. Later slices may extend this workspace only when
the Phase M roadmap authorizes their frozen compiler conformance family.
