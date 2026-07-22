# Presolve launch content contract

L18 owns versioned, repository-local launch content only. Its source is
site/content/; site/README.md indexes the content and explicitly disclaims
deployment. No web server, deployment configuration, analytics, hosted
playground, product API, benchmark result, or new compiler/runtime behavior is
introduced by this contract.

Every L18 page declares the presolve.launch-content schema at version 1 and the
current repository release version. The required routes are home, documentation,
architecture, examples, benchmark methodology, roadmap, and playground. Local
links must resolve in the repository; the public repository link is the one
external destination required by this slice.

The playground route is a clearly non-functional placeholder. Benchmark content
may describe the committed methodology and observation boundary, but must not
claim comparative numbers, a performance gate, or a hosted benchmark service.
