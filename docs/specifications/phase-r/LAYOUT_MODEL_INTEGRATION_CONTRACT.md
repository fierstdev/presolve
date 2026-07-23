# Phase R Layout Model Integration Contract

Automatic layout composition enters the compiler before component instance
planning. The application-model assembly sequence is:

```text
components + templates + authored invocation facts
  → file-route graph + validated layout composition edges
  → authored and virtual invocation registry
  → component instance plan
  → Slot bindings, Context, forms, resources, actions, runtime, resume
```

The virtual invocation registry uses `ComponentInvocationId::for_layout_composition`.
It records an outer layout caller and the next layout/page callee, but has no
authored template position or source-content fragment. The instance planner
must treat it as an ordinary resolved non-structural child edge.

Slot binding gains an explicit direct-child content variant for this edge. It
binds the callee instance to the caller layout's unique default outlet. Ordinary
HTML renders that same planned child at the outlet; it does not copy or wrap
the child HTML. Runtime/resume products observe the composed plan because they
are derived only after this insertion.

The existing generic application-model entry points remain authored-only. A
new compiler-owned file-route application-model entry point owns this expanded
assembly; framework and CLI code may only request its final products.
