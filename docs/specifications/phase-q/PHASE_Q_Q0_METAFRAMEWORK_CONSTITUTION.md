# Phase Q Q0 metaframework constitution

**Status:** Q0 implementation authority.

The metaframework is a compiler product over the frozen framework. Its public
source vocabulary begins with the existing compiler-recognized
`@route("/path")` component declaration; it does not add file-system routing
or a JavaScript route registry.

Every route publication request supplies the complete source set, an explicit
ordered route-entry list, package contracts/runtime mappings, and output root.
The compiler validates path identity and produces route manifests/artifacts.
The application package only projects the command.

Browser navigation is document navigation for the initial product. Layouts,
parameters, server code, deployment, and provider adapters each require their
own compiler product under Q2-Q4. No generic runtime router, server, source
discovery, or artifact merger is permitted.
