# SQLite/PostgreSQL same-release SQL portability proof

This proof extends Presolve's deployment-decision coverage to relational SQL.

It asks a deliberately narrow question:

> Can one immutable application release declare one portable relational capability, resolve against materially different relational engines, and execute the same Component bytes through both without rebuilding the application or embedding provider configuration into its identity?

The proof uses `presolve:sql/database@0.1.0`, SQLite, and PostgreSQL.

## What is held constant

The application is `examples/sql-demo`.

Presolve builds `sql-demo.presolved` exactly once. The resulting archive is then reused for every planning and execution step. The proof records both the Presolve release digest and the raw archive SHA-256 before resolution and verifies both again after execution.

The application does not detect which database is selected. It uses a deliberately shared SQL subset and positional `$1`, `$2`, ... bound parameters. SQLite compatibility with that parameter form is tested directly.

Presolve does not rewrite SQL and does not claim that arbitrary SQLite and PostgreSQL dialects are interchangeable.

## Environments

The same detached release is resolved against two checked-in Environment Specifications:

| Environment | Selected provider | Realization |
| --- | --- | --- |
| `sql-sqlite` | SQLite SQL | node-side SQLite database path |
| `sql-postgres` | PostgreSQL SQL | node-side host/port/database/user plus password secret |

Those realization values are intentionally absent from the Application Contract, DeploymentPlan, and `.presolved` identity.

## Provider conformance

Before executing the application proof, both engines are exercised through the same reusable SQL conformance harness.

The harness verifies:

- integer, text, and byte round trips through typed relational columns
- bound values remain data rather than becoming statement source
- `null`, empty text, and empty bytes remain distinct
- ordered column metadata survives a query that returns zero rows
- explicit transaction rollback
- statement ordering followed by commit
- dropping an uncommitted transaction rolls it back

### A useful failure discovered during validation

The first live PostgreSQL conformance run exposed a flaw in the test fixture rather than in PostgreSQL or the provider API.

The original fixture used one `optional_value` column for both text and bytes. SQLite's dynamic typing accepted both representations, while PostgreSQL correctly rejected serializing text into a `BYTEA` parameter slot.

The fixture was corrected to use separate typed columns:

```text
id             integer
label          text
optional_text  nullable text
bytes_value    nullable bytes
```

That correction matters because the conformance suite is intended to validate Presolve semantics across engines with different type systems, not to encode assumptions that only a dynamically typed engine satisfies.

A second tightening requires result column metadata to remain available even when a query returns no rows. SQLite already obtained metadata from its prepared statement. The PostgreSQL adapter was changed to derive columns from the prepared statement rather than from the first returned row.

The corrected type-safe conformance model passes against both SQLite and a live PostgreSQL engine.

## Execution path

Run:

```bash
bash ./scripts/check-sql-portability.sh
```

The checkpoint performs these stages:

1. validates SQLite against the reusable SQL conformance suite
2. starts a digest-pinned PostgreSQL container and validates PostgreSQL against the same suite
3. builds the Presolve CLI/runtime tooling
4. builds `sql-demo.presolved` exactly once
5. records its release digest and archive SHA-256
6. resolves those exact bytes against the SQLite Environment Specification
7. resolves those exact bytes against the PostgreSQL Environment Specification
8. realizes the exact provider selected by each DeploymentPlan
9. provisions the environment-owned relational schema
10. executes the same embedded Component bytes through SQLite
11. executes the same embedded Component bytes through PostgreSQL
12. re-inspects and re-hashes the release and fails if its identity changed

The PostgreSQL test service is pinned as:

```text
postgres:18-alpine@sha256:77f585114c32fbca283dc835b0596f4e52b51b4c6662d7810b2f4084f60a1873
```

Schema creation is environment provisioning. DDL/migration lifecycle is deliberately outside `presolve:sql/database@0.1.0` and is not embedded into the application portability claim.

## What this proves

This proof establishes that:

- relational SQL is materialized through the same resolver → ProviderId → realization → runtime boundary used by other Presolve capabilities
- the same application release can bind to SQLite and PostgreSQL without an application rebuild
- provider connection details and credentials remain realization-time concerns
- the portable scalar/result/transaction semantics survive two engines with materially different typing and operational models
- Presolve can expose dialect assumptions explicitly instead of pretending all SQL syntax is portable
- provider conformance can identify false portability assumptions before they become part of the capability contract

## What this does not prove

This proof does not claim:

- arbitrary SQL dialect portability
- automatic SQL rewriting or migration translation
- a portable schema-migration system
- connection pooling, TLS policy, HA, replication, failover, or backup semantics
- equivalence of SQLite and PostgreSQL isolation/concurrency behavior
- that every provider-specific PostgreSQL or SQLite feature belongs in the portable baseline

Those differences remain explicit evidence for later semantic negotiation rather than being hidden behind the word "SQL".

## Public CI

The repository has a dedicated **SQL portability** CI job. It is additive to the existing KV flagship, object-storage, Docker, Core, TypeScript, and Architecture gates.

The proof is intentionally independent so a future SQL regression cannot be masked by unrelated portability checks.