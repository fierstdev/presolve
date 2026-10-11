# Diagnostics reference

Presolve uses stable diagnostic codes for user-facing validation and resolution failures.

The code space is divided by subsystem.

## `PS1xxx` — Application Contract

The `PS1xxx` range is reserved for Application Contract diagnostics.

| Code | Meaning |
| --- | --- |
| `PS1001` | Application Contract parse error |
| `PS1002` | unsupported Application Contract version |
| `PS1003` | invalid application name |
| `PS1004` | invalid capability interface |
| `PS1005` | duplicate capability |
| `PS1006` | invalid resource requirement |
| `PS1007` | invalid network policy |
| `PS1008` | invalid workload name |
| `PS1009` | duplicate workload |
| `PS1010` | invalid application-internal interface name |
| `PS1011` | duplicate application-internal interface |
| `PS1012` | relationship references unknown workload |
| `PS1013` | relationship references unknown interface |
| `PS1014` | relationship connects a workload to itself |
| `PS1015` | duplicate relationship |
| `PS1016` | invalid capability feature |
| `PS1017` | duplicate capability feature |

Semantic Application Contract validation can return multiple diagnostics in one result.

## `PS2xxx` — resolution

| Code | Meaning |
| --- | --- |
| `PS2001` | required capability is missing |
| `PS2002` | insufficient environment memory |
| `PS2003` | insufficient environment CPU |
| `PS2004` | compatible capability exists but required semantic features are missing |
| `PS2005` | environment does not support a required workload kind |

## Environment Specification errors

Environment Specification validation currently uses typed Rust errors rather than assigned `PSxxxx` diagnostic codes.

This documentation will be extended when a stable user-facing diagnostic mapping is introduced for that subsystem.

## Stability

A diagnostic code is intended to be a more stable machine-facing identifier than its human-readable wording.

Presolve is still pre-stable, so the full diagnostic compatibility policy has not yet been declared stable.
