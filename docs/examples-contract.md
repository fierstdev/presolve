# Presolve alpha example contract

**Status:** L14-A authoritative corpus contract. Examples consume frozen
compiler products and explicit inputs; they do not introduce language/runtime
semantics, source discovery, or a create command.

| Example | Existing evidence | Explicit authority | Public proof | Excluded behavior |
| --- | --- | --- | --- | --- |
| Counter | `fixtures/0001-source-summary`; `examples/counter` | one listed `Counter.tsx` | existing explicit build/check path | discovery, scaffolding |
| Components/Context/Slots | `fixtures/0059`, `0062`--`0065`; component/context contracts | listed component source files | frozen component/runtime fixture evidence | inheritance/slot behavior beyond frozen contracts |
| Forms | `docs/forms-contract.md`; `crates/ezc_core/src/form_submission_host.rs` | `examples/forms/src/Forms.tsx` | explicit `presolve check` proof | browser submit/network behavior |
| Explicit workspace | workspace fixtures `chain-v1` and `cycle-v1` | caller-supplied package/edge/source list | `presolve workspace` contract | manifests/dependency discovery |
| Production/resume | Phase K corpus and resumability contract | explicit compiler input and existing artifact fixtures | existing production/browser evidence | benchmarks, deployment, author-time edits |

Each L14-B example must declare its own supplied configuration and source
membership, run a public accepted command, cite only existing expected product
identities, and add a documentation snippet test. Runtime execution is required
only when an existing browser fixture proves that exact kind of behavior. The
alpha corpus has exactly these five examples; `presolve create`, template
generation, package installation, manifest discovery, remote data, and every
reserved capability are excluded until separately contracted after alpha.
