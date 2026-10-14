use semver::Version;

use crate::{CapabilityContract, CapabilityContractError};

/// Canonical capability interface identity for relational SQL databases.
pub const SQL_DATABASE_INTERFACE: &str = "presolve:sql/database";

/// Begins one explicit transaction and returns a transaction-scoped handle.
pub const SQL_DATABASE_OPERATION_BEGIN: &str = "begin";

/// Commits one previously begun transaction.
pub const SQL_DATABASE_OPERATION_COMMIT: &str = "commit";

/// Executes one SQL statement when row results are not required.
pub const SQL_DATABASE_OPERATION_EXECUTE: &str = "execute";

/// Executes one SQL statement and returns its result rows.
pub const SQL_DATABASE_OPERATION_QUERY: &str = "query";

/// Rolls back one previously begun transaction.
pub const SQL_DATABASE_OPERATION_ROLLBACK: &str = "rollback";

/// Statement parameters are bound independently of SQL source text.
///
/// Conforming providers must not satisfy this guarantee by interpolating
/// application values into SQL text.
pub const SQL_DATABASE_GUARANTEE_BOUND_PARAMETERS: &str = "bound-parameters";

/// Baseline parameter and result values preserve a portable scalar domain.
///
/// The baseline domain is null, signed 64-bit integer, IEEE-754 binary64,
/// UTF-8 text, and opaque bytes. Rich provider types remain outside the
/// baseline contract unless a future semantic extension defines them.
pub const SQL_DATABASE_GUARANTEE_PORTABLE_SCALAR_VALUES: &str = "portable-scalar-values";

/// One capability invocation executes exactly one SQL statement.
pub const SQL_DATABASE_GUARANTEE_SINGLE_STATEMENT_EXECUTION: &str = "single-statement-execution";

/// Query result columns preserve the order produced by the database.
///
/// Rows are represented positionally rather than as maps so duplicate column
/// labels remain representable.
pub const SQL_DATABASE_GUARANTEE_ORDERED_RESULT_COLUMNS: &str = "ordered-result-columns";

/// SQL null remains distinct from empty text, empty bytes, and numeric zero.
pub const SQL_DATABASE_GUARANTEE_NULL_DISTINCT_FROM_EMPTY: &str = "null-distinct-from-empty";

/// Presolve does not add an ordering guarantee to query rows.
///
/// Applications that require deterministic row order must express that order
/// in their SQL statement according to the selected SQL dialect.
pub const SQL_DATABASE_GUARANTEE_QUERY_DEFINED_ROW_ORDER: &str = "query-defined-row-order";

/// A committed transaction applies its baseline data mutations atomically and
/// a rolled-back transaction exposes none of its uncommitted data mutations.
///
/// Schema-definition behavior is intentionally outside the 0.1 baseline.
pub const SQL_DATABASE_GUARANTEE_TRANSACTION_ATOMICITY: &str = "transaction-atomicity";

/// Every operation issued through one transaction handle is executed through
/// the same logical transactional session.
pub const SQL_DATABASE_GUARANTEE_TRANSACTION_SESSION_AFFINITY: &str =
    "transaction-session-affinity";

/// Statements issued through one transaction handle are observed in invocation
/// order by that transaction.
pub const SQL_DATABASE_GUARANTEE_ORDERED_TRANSACTION_STATEMENTS: &str =
    "ordered-transaction-statements";

/// Dropping an open transaction without committing it rolls back all of its
/// uncommitted baseline data mutations.
///
/// This guarantee makes transaction-resource destruction safe across language
/// and transport bindings where explicit cleanup is not always observable.
pub const SQL_DATABASE_GUARANTEE_UNCOMMITTED_DROP_ROLLS_BACK: &str =
    "uncommitted-drop-rolls-back";

/// Provider accepts PostgreSQL-family SQL syntax and semantics.
pub const SQL_DATABASE_FEATURE_DIALECT_POSTGRESQL: &str = "dialect-postgresql";

/// Provider accepts SQLite-family SQL syntax and semantics.
pub const SQL_DATABASE_FEATURE_DIALECT_SQLITE: &str = "dialect-sqlite";

/// Provider supports read-only transaction intent as a portable transaction
/// option.
pub const SQL_DATABASE_FEATURE_READ_ONLY_TRANSACTIONS: &str = "read-only-transactions";

/// Provider supports SQL statements that return mutation result rows.
pub const SQL_DATABASE_FEATURE_RETURNING: &str = "returning";

/// Provider supports savepoint semantics within an explicit transaction.
pub const SQL_DATABASE_FEATURE_SAVEPOINTS: &str = "savepoints";

/// Provider can satisfy an explicitly requested serializable transaction mode.
pub const SQL_DATABASE_FEATURE_SERIALIZABLE_TRANSACTIONS: &str = "serializable-transactions";

/// Returns the canonical relational SQL capability contract.
///
/// The baseline deliberately standardizes execution, result, parameter, and
/// transaction semantics without claiming that arbitrary SQL source text is
/// portable across dialects. Applications that depend on dialect-specific SQL
/// must request the corresponding dialect feature.
///
/// # Errors
///
/// Returns an error only if Presolve's built-in static contract definition is
/// internally invalid.
pub fn sql_database_contract() -> Result<CapabilityContract, CapabilityContractError> {
    CapabilityContract::new(
        SQL_DATABASE_INTERFACE,
        Version::new(0, 1, 0),
        [
            SQL_DATABASE_OPERATION_BEGIN,
            SQL_DATABASE_OPERATION_COMMIT,
            SQL_DATABASE_OPERATION_EXECUTE,
            SQL_DATABASE_OPERATION_QUERY,
            SQL_DATABASE_OPERATION_ROLLBACK,
        ],
        [
            SQL_DATABASE_GUARANTEE_BOUND_PARAMETERS,
            SQL_DATABASE_GUARANTEE_PORTABLE_SCALAR_VALUES,
            SQL_DATABASE_GUARANTEE_SINGLE_STATEMENT_EXECUTION,
            SQL_DATABASE_GUARANTEE_ORDERED_RESULT_COLUMNS,
            SQL_DATABASE_GUARANTEE_NULL_DISTINCT_FROM_EMPTY,
            SQL_DATABASE_GUARANTEE_QUERY_DEFINED_ROW_ORDER,
            SQL_DATABASE_GUARANTEE_TRANSACTION_ATOMICITY,
            SQL_DATABASE_GUARANTEE_TRANSACTION_SESSION_AFFINITY,
            SQL_DATABASE_GUARANTEE_ORDERED_TRANSACTION_STATEMENTS,
            SQL_DATABASE_GUARANTEE_UNCOMMITTED_DROP_ROLLS_BACK,
        ],
        [
            SQL_DATABASE_FEATURE_DIALECT_POSTGRESQL,
            SQL_DATABASE_FEATURE_DIALECT_SQLITE,
            SQL_DATABASE_FEATURE_READ_ONLY_TRANSACTIONS,
            SQL_DATABASE_FEATURE_RETURNING,
            SQL_DATABASE_FEATURE_SAVEPOINTS,
            SQL_DATABASE_FEATURE_SERIALIZABLE_TRANSACTIONS,
        ],
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn built_in_sql_database_contract_is_valid_and_canonical() {
        let contract = sql_database_contract().expect("SQL database contract should be valid");

        assert_eq!(contract.interface(), SQL_DATABASE_INTERFACE);
        assert_eq!(contract.version(), &Version::new(0, 1, 0));
        assert_eq!(
            contract.operations(),
            ["begin", "commit", "execute", "query", "rollback"]
        );
        assert_eq!(
            contract.guarantees(),
            [
                "bound-parameters",
                "null-distinct-from-empty",
                "ordered-result-columns",
                "ordered-transaction-statements",
                "portable-scalar-values",
                "query-defined-row-order",
                "single-statement-execution",
                "transaction-atomicity",
                "transaction-session-affinity",
                "uncommitted-drop-rolls-back"
            ]
        );
        assert_eq!(
            contract.optional_features(),
            [
                "dialect-postgresql",
                "dialect-sqlite",
                "read-only-transactions",
                "returning",
                "savepoints",
                "serializable-transactions"
            ]
        );
    }
}
