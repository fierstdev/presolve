# Application Contract reference

The Application Contract is the versioned, environment-independent declaration of application semantics and requirements.

Canonical filename:

```text
presolve.toml
```

Current schema version:

```text
0.1
```

Unknown fields are rejected.

## Complete example

```toml
contract_version = "0.1"

[application]
name = "commerce"
version = "1.0.0"
description = "Example multi-workload application"

[[workloads]]
name = "api"
kind = "component"

[[workloads]]
name = "worker"
kind = "component"

[[interfaces]]
name = "jobs"
kind = "event"

[[relationships]]
from = "api"
to = "worker"
interface = "jobs"

[[capabilities]]
interface = "presolve:kv/store"
version = "^0.1"

[[capabilities]]
interface = "presolve:objects/store"
version = "^0.1"
required_features = ["range-read"]
preferred_features = ["user-metadata"]

[[capabilities]]
interface = "presolve:sql/database"
version = "^0.1"
preferred_features = ["returning"]

[[capabilities]]
interface = "presolve:queue/work-queue"
version = "^0.1"

[resources]
memory_mib = 256
cpu_millis = 500

[network]
outbound = "deny"
```

The example intentionally names semantic requirements rather than Redis, S3, SQLite, PostgreSQL, Cloudflare, queue vendors, endpoints, credentials, or deployment placement.

## `contract_version`

Required.

The current accepted value is:

```toml
contract_version = "0.1"
```

Application Contract schema versions evolve independently from the Presolve package version.

## `[application]`

Required fields:

```toml
[application]
name = "commerce"
version = "1.0.0"
```

Optional:

```toml
description = "..."
```

Application names use 1–63 lowercase ASCII letters, digits, or hyphens, must begin with a letter, and must end with a letter or digit.

## `[[workloads]]`

A workload is one logical executable constituent.

```toml
[[workloads]]
name = "api"
kind = "component"
```

Recognized workload kinds:

| Value | Meaning | Current execution status |
| --- | --- | --- |
| `component` | WebAssembly Component workload | implemented |
| `container` | OCI-compatible container workload | immutable identity, resolution, and Docker execution path implemented |

Workload kind is application semantics. Build artifact paths, image registry locations, and deployment placement do not belong in `presolve.toml`.

Workload names are unique within the application and follow the same name rules as the application name.

## `[[interfaces]]`

Application-local communication semantics:

```toml
[[interfaces]]
name = "jobs"
kind = "event"
```

Implemented kinds:

| Value | Meaning |
| --- | --- |
| `request_response` | caller expects a response |
| `event` | producer emits information without requiring a direct response |

An interface does not prescribe transport.

## `[[relationships]]`

A relationship connects two declared workloads through a declared interface:

```toml
[[relationships]]
from = "api"
to = "worker"
interface = "jobs"
```

Relationships must reference existing workloads and interfaces, may not connect a workload to itself, and may not be duplicated.

## `[[capabilities]]`

Capability requirements use `<namespace>:<package>/<interface>` identifiers:

```toml
[[capabilities]]
interface = "presolve:objects/store"
version = "^0.1"
```

Fields:

| Field | Required | Meaning |
| --- | --- | --- |
| `interface` | yes | capability semantic identity |
| `version` | yes | compatible semantic-version requirement |
| `optional` | no | whether the capability may remain unbound |
| `required_features` | no | semantic features every selected provider must support |
| `preferred_features` | no | features used to rank compatible providers |

Example:

```toml
[[capabilities]]
interface = "presolve:objects/store"
version = "^0.1"
optional = false
required_features = ["range-read"]
preferred_features = ["user-metadata"]
```

A feature may not be repeated across required and preferred feature lists.

### Current built-in semantic contracts

```text
presolve:kv/store@0.1.0
presolve:objects/store@0.1.0
presolve:sql/database@0.1.0
presolve:queue/work-queue@0.1.0
```

These contracts standardize application-visible semantics, not one concrete transport or vendor implementation.

For example, an application that depends on PostgreSQL-family SQL syntax can declare that assumption explicitly:

```toml
[[capabilities]]
interface = "presolve:sql/database"
version = "^0.1"
required_features = ["dialect-postgresql"]
preferred_features = ["returning"]
```

Presolve does not claim that arbitrary SQLite and PostgreSQL SQL source is interchangeable. Dialect dependence remains an explicit semantic requirement when needed.

Similarly, the baseline work-queue contract guarantees at-least-once leased delivery with explicit acknowledgement and retry. FIFO ordering, dead-letter routing, configurable lease duration, and delayed delivery are optional features rather than implicit baseline behavior.

See [Capability contracts](../architecture/capability-contracts.md), [SQL capability semantics](../architecture/sql-capability.md), and [Work-queue capability semantics](../architecture/work-queue-capability.md).

## `[resources]`

Resource requirements are minimums:

```toml
[resources]
memory_mib = 128
cpu_millis = 250
```

Each field is optional. If present, its value must be greater than zero.

## `[network]`

Outbound modes:

```text
deny
allow_list
allow_all
```

Default behavior is `deny`.

An allow list is valid only with `allow_list`:

```toml
[network]
outbound = "allow_list"
allow = ["api.example.com", "storage.example.com:443"]
```

Targets are hosts or host-and-port values, not URLs or paths.

## Validation

`parse_contract` deserializes TOML and then performs semantic validation.

Validation returns all detected Application Contract diagnostics rather than only the first semantic error.

See [Diagnostics](diagnostics.md).

## Application vs deployment

Do not place these in `presolve.toml`:

- environment selection
- provider IDs
- provider credentials
- provider endpoints or database paths
- build artifact paths
- deployment placement
- infrastructure resource names

Those are not application semantics.
