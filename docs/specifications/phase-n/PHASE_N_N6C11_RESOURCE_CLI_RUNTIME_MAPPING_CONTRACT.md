# Phase N N6-C11 resource CLI runtime mapping contract

`presolve build` accepts `--package-runtime specifier=runtime-location` only
when the same invocation supplied a validated `--package-contract` for that
specifier. The CLI expands the contract's declared runtime modules into exact
package/version/integrity/module keys in the compiler-owned runtime-module
table. Repeated exports sharing a module retain one key; missing contracts or
malformed mappings fail with exit status 2.

This is explicit configuration, not package discovery. The option does not
read package files, lockfiles, or `node_modules`. N6-C13 consumes the resulting
table only for resolved Resource declarations, publishes
`resources.runtime.json`, and embeds the same artifact in the page. A Resource
build with an omitted exact mapping fails with `PSRES1001` before publication.

Verification is `scripts/verify-n6c11-resource-cli-runtime-mapping.sh`.
