# Presolve identity transition

Phase L1 establishes Presolve as the public product, compiler, executable, and
package namespace. The canonical command is `presolve`; the public package
scope is `@presolve`; the public repository destination is
`github.com/fierstdev/presolve`; and the website is `presolve.dev`.

The transition is identity-only. It neither changes authored language
semantics nor changes diagnostics, runtime behavior, compiler products, or
generated artifacts.

## Retained historical identity

The historical engineering name remains only where the Phase A-K freeze makes
replacement incorrect. The retained occurrences are deliberately classified:

| Location class | Reason | Rule |
| --- | --- | --- |
| `notes/progress/`, `docs/planning/`, ADRs, and RFCs | Historical engineering record | Preserve the record verbatim. |
| Frozen Phase A-K contracts | Frozen provenance and contract terminology | Preserve until the historical archive is established in L2. |
| `crates/ezc_*` paths and Rust import aliases | Private implementation layout retained until L2 repository restructuring | Never present these as a public package or executable. |
| Generated runtime globals and diagnostics | Frozen generated-artifact/runtime contract | Do not rename in L1; generated output must remain byte-equivalent. |
| Fixture source, browser probes, and assertion labels | Authored test data and test-only expectations | Preserve unless a later authorized artifact migration changes the frozen contract. |

`scripts/verify-public-identity.sh` verifies the active public metadata,
documentation, workflow, package, schema, and command surfaces. It fails when
the historical public identity is reintroduced there.
