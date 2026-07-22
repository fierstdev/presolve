# Phase N N1-A semantic package contract

N1-A introduces a versioned, caller-supplied semantic package contract. A
contract names the resolved package/version, SHA-256 integrity, public exports,
semantic kind, type signature, runtime module, and resume policy. The compiler
validates the contract without reading package source or resolving/installing
npm dependencies.

The initial resolution table maps an import specifier to exactly one validated
contract and resolves only declared exports. This is foundation work: source
imports will consume the table in the next N1-A integration slice. Unknown
exports, invalid integrity, unsupported schemas, and duplicate specifiers fail
closed.
