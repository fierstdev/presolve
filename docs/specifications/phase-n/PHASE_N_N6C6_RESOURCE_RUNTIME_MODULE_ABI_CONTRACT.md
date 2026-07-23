# Phase N N6-C6 resource runtime-module ABI contract

N6-C6 defines the sole accepted resolution of third-party Resource runtime bytes.
`SemanticPackageRuntimeModuleTable` maps an exact package, version, integrity,
and contract-declared runtime-module coordinate to one explicit host location.
It rejects empty locations and duplicate coordinates, and it cannot resolve a
changed module path or integrity value.

The table is supplied by the application host or future metaframework. The
compiler does not inspect a package, discover `node_modules`, read a lockfile,
install dependencies, infer a URL, or fall back to a global module registry.

This establishes a production-capable handoff for later artifact publication;
it does not yet load a module or make Resource source executable. `PSC1046`
remains in force until the compiler wires the location through its artifact,
runtime activation, cancellation, snapshot, resume, and browser proof.

Verification is `scripts/verify-n6c6-resource-runtime-module-abi.sh`.
