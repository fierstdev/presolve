# SQL capability semantics

Presolve's relational SQL capability is the first capability family whose portability boundary cannot be described honestly as a simple vendor substitution.

The built-in semantic contract is:

```text
presolve:sql/database@0.1.0
```

The contract standardizes how an application invokes a relational database, how values and results cross the capability boundary, and what an explicit transaction means. It does **not** claim that arbitrary SQL source text is portable across database dialects.

## Why SQL needs a stricter boundary

Key-value and object-storage operations have small operation vocabularies whose semantics Presolve can define directly. SQL is different because the application supplies a query language that varies materially between engines.

A provider-neutral SQL capability therefore has two distinct compatibility questions:

1. Can the provider satisfy Presolve's execution, value, result, and transaction semantics?
2. Can the provider execute the SQL dialect used by this application?

Presolve must not collapse those questions into one vague claim of "SQL compatibility."

## Baseline operations

Version `0.1.0` defines five mandatory operations:

| Operation | Meaning |
| --- | --- |
| `begin` | Begin one explicit transaction and return an opaque transaction handle. |
| `execute` | Execute exactly one SQL statement when row results are not required. |
| `query` | Execute exactly one SQL statement and return its result rows. |
| `commit` | Atomically commit one open transaction. |
| `rollback` | Roll back one open transaction. |

`execute` and `query` may be invoked outside a transaction or through a transaction handle. Operations using one transaction handle must remain on the same logical transactional session.

The application-facing binding should not expose connection objects, pools, hostnames, driver handles, or vendor client types.

## Portable scalar value model

The baseline parameter/result domain is intentionally small:

```text
null
signed 64-bit integer
IEEE-754 binary64
UTF-8 text
opaque bytes
```

This is the minimum domain the initial SQLite and PostgreSQL providers are expected to round-trip without inventing provider-specific application types.

The baseline does not currently give first-class portable representations to:

- arbitrary-precision decimal values
- UUIDs
- dates, times, or timestamps
- JSON documents
- arrays
- enums
- engine-specific extension types

Applications may encode richer values into baseline text/bytes when appropriate, or a future capability version may add explicit semantic types. Presolve should not silently coerce rich provider types into lossy baseline values.

## Parameters

Statement values are bound independently from SQL source text.

A conforming provider must not implement parameter binding by string interpolation. This is both a semantic and security boundary: application values remain values, not executable query text.

The eventual provider SDK/WIT binding should use an ordered parameter sequence. Named-parameter syntax is dialect-specific and is not part of the `0.1` baseline.

## Results

Rows are positional, not maps.

A query result preserves column order exactly as returned by the database. This is necessary because SQL permits duplicate column labels:

```sql
SELECT left_table.id, right_table.id
```

A map-shaped result would erase that distinction.

SQL `NULL` is distinct from empty text, empty bytes, and numeric zero.

Presolve adds no row-order guarantee. Applications that depend on deterministic ordering must express ordering in their SQL according to the selected dialect.

## Transactions

An explicit Presolve SQL transaction guarantees:

- operations issued through the same handle use the same logical transactional session;
- statements are observed by that transaction in invocation order;
- successful commit applies baseline data mutations atomically; and
- rollback exposes none of that transaction's uncommitted baseline data mutations.

The baseline deliberately does **not** promise a particular isolation level. If an application requires serializable execution, it must request the `serializable-transactions` semantic feature.

Schema-definition behavior is outside the `0.1` transaction guarantee. Database migrations and schema lifecycle are a separate concern and should not be smuggled into the runtime capability contract merely to make a demonstration self-contained.

## SQL text and dialects

Presolve does not parse, rewrite, transpile, or normalize application SQL in the `0.1` capability.

Providers may advertise dialect features:

```text
dialect-postgresql
dialect-sqlite
```

An application that uses dialect-specific syntax must request the corresponding feature:

```toml
[[capabilities]]
interface = "presolve:sql/database"
version = "^0.1"
required_features = ["dialect-postgresql"]
```

That makes the deployment requirement explicit and lets the resolver reject a SQLite-only provider before execution.

An application may omit a dialect requirement when it intentionally restricts itself to SQL accepted by every target provider it intends to use. Presolve does not currently statically prove that query text belongs to such a shared subset. Runtime proof and application tests remain necessary evidence.

This is an intentional non-claim rather than hidden incompatibility.

## Optional semantic features

The first contract vocabulary includes:

| Feature | Meaning |
| --- | --- |
| `dialect-postgresql` | PostgreSQL-family SQL syntax/semantics are accepted. |
| `dialect-sqlite` | SQLite-family SQL syntax/semantics are accepted. |
| `read-only-transactions` | Read-only transaction intent can be requested portably. |
| `returning` | Mutation statements may produce result rows through SQL `RETURNING`-style semantics. |
| `savepoints` | Savepoint semantics are available inside an explicit transaction. |
| `serializable-transactions` | The provider can satisfy an explicitly requested serializable transaction mode. |

These are semantic promises, not implementation labels. For example, a managed PostgreSQL-compatible service may advertise `dialect-postgresql`; the feature does not mean the provider process itself must be PostgreSQL.

## What the resolver can and cannot prove

Given:

```text
application SQL requirement
        +
environment SQL supply
        ↓
interface/version/feature negotiation
```

Presolve can prove that a provider advertises the required SQL semantic contract and requested features.

For `0.1`, Presolve cannot prove arbitrary SQL source compatibility when the application deliberately omits a dialect requirement. That would require either a standardized Presolve query IR, SQL parsing/static analysis, or another explicit portability mechanism. None of those are implied by this contract.

## First implementation proof

The intended first end-to-end proof is one immutable SQL application release realized against:

```text
SQLite provider
      ↕
 presolve:sql/database@0.1.0
      ↕
PostgreSQL provider
```

The proof application should use only a deliberately shared SQL subset and exercise:

1. parameterized mutation outside a transaction;
2. ordered-column query results and null handling;
3. an explicit transaction with multiple ordered mutations;
4. rollback and post-rollback observation;
5. commit and post-commit observation; and
6. unchanged `.presolved` identity across both realizations.

Schema setup should happen outside the application capability path so the proof tests runtime SQL semantics rather than cross-engine DDL portability.

## Deferred questions

Real provider work is expected to expose additional semantic dimensions. In particular, the current boolean feature vocabulary may be insufficient for values such as:

- maximum parameter count
- maximum statement/result size
- transaction isolation families beyond serializable support
- numeric precision/scale support
- timeout/cancellation guarantees
- connection/session limits

Those should become typed semantic properties only when provider evidence demonstrates the required shape. SQL is a reason to evolve the capability IR carefully, not a reason to invent a large generic property system in advance.

## Non-goals for `0.1`

This contract is not:

- an ORM;
- a query builder;
- a migration framework;
- a universal SQL transpiler;
- a common-denominator replacement for PostgreSQL or SQLite features; or
- a claim that all relational databases are interchangeable.

The goal is narrower: make the application/provider boundary explicit enough that Presolve can reason honestly about which relational database realizations can satisfy a release.
