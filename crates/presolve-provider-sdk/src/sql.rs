use std::fmt;

pub use presolve_capability::SQL_DATABASE_INTERFACE;
use thiserror::Error;

/// Portable scalar value accepted and returned by the SQL baseline.
#[derive(Debug, Clone, PartialEq)]
pub enum SqlValue {
    /// SQL `NULL`.
    Null,

    /// Signed 64-bit integer.
    Integer(i64),

    /// IEEE-754 binary64 floating-point value.
    Real(f64),

    /// UTF-8 text.
    Text(String),

    /// Opaque bytes.
    Bytes(Vec<u8>),
}

/// Portable metadata for one query-result column.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SqlColumn {
    name: String,
}

impl SqlColumn {
    /// Creates portable query-column metadata.
    #[must_use]
    pub fn new(name: impl Into<String>) -> Self {
        Self { name: name.into() }
    }

    /// Returns the column label exactly as reported by the provider.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }
}

/// One positional SQL result row.
///
/// Rows are vectors rather than maps because duplicate result-column labels are
/// valid SQL and must remain representable.
#[derive(Debug, Clone, PartialEq)]
pub struct SqlRow {
    values: Vec<SqlValue>,
}

impl SqlRow {
    /// Creates one positional row.
    #[must_use]
    pub const fn new(values: Vec<SqlValue>) -> Self {
        Self { values }
    }

    /// Returns row values in result-column order.
    #[must_use]
    pub fn values(&self) -> &[SqlValue] {
        &self.values
    }

    /// Consumes the row and returns its positional values.
    #[must_use]
    pub fn into_values(self) -> Vec<SqlValue> {
        self.values
    }
}

/// Stable categories for SQL provider failures.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SqlDatabaseErrorCode {
    /// The provider cannot currently service requests.
    Unavailable,

    /// The supplied SQL statement is invalid for the selected provider/dialect.
    InvalidStatement,

    /// One or more bound parameters cannot satisfy the statement.
    InvalidParameter,

    /// Statement execution violated a database constraint.
    ConstraintViolation,

    /// The requested operation is invalid for the transaction's current state.
    TransactionState,

    /// The provider does not implement a requested optional SQL semantic.
    Unsupported,

    /// The operation failed for a condition that callers may reasonably retry.
    Transient,

    /// The provider encountered an internal failure.
    Internal,
}

/// Failure returned by a relational SQL provider.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
#[error("{message}")]
pub struct SqlDatabaseError {
    code: SqlDatabaseErrorCode,
    message: String,
}

impl SqlDatabaseError {
    /// Creates an unavailable-provider error.
    #[must_use]
    pub fn unavailable(message: impl Into<String>) -> Self {
        Self::new(SqlDatabaseErrorCode::Unavailable, message)
    }

    /// Creates an invalid-statement error.
    #[must_use]
    pub fn invalid_statement(message: impl Into<String>) -> Self {
        Self::new(SqlDatabaseErrorCode::InvalidStatement, message)
    }

    /// Creates an invalid-parameter error.
    #[must_use]
    pub fn invalid_parameter(message: impl Into<String>) -> Self {
        Self::new(SqlDatabaseErrorCode::InvalidParameter, message)
    }

    /// Creates a constraint-violation error.
    #[must_use]
    pub fn constraint_violation(message: impl Into<String>) -> Self {
        Self::new(SqlDatabaseErrorCode::ConstraintViolation, message)
    }

    /// Creates a transaction-state error.
    #[must_use]
    pub fn transaction_state(message: impl Into<String>) -> Self {
        Self::new(SqlDatabaseErrorCode::TransactionState, message)
    }

    /// Creates an unsupported-semantic error.
    #[must_use]
    pub fn unsupported(message: impl Into<String>) -> Self {
        Self::new(SqlDatabaseErrorCode::Unsupported, message)
    }

    /// Creates a transient provider error.
    #[must_use]
    pub fn transient(message: impl Into<String>) -> Self {
        Self::new(SqlDatabaseErrorCode::Transient, message)
    }

    /// Creates an internal-provider error.
    #[must_use]
    pub fn internal(message: impl Into<String>) -> Self {
        Self::new(SqlDatabaseErrorCode::Internal, message)
    }

    fn new(code: SqlDatabaseErrorCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }

    /// Returns the stable provider error category.
    #[must_use]
    pub const fn code(&self) -> SqlDatabaseErrorCode {
        self.code
    }

    /// Returns the human-readable provider error message.
    #[must_use]
    pub fn message(&self) -> &str {
        &self.message
    }
}

/// Result metadata for a SQL statement executed without row results.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SqlExecuteResult {
    affected_rows: u64,
}

impl SqlExecuteResult {
    /// Creates execute-result metadata.
    #[must_use]
    pub const fn new(affected_rows: u64) -> Self {
        Self { affected_rows }
    }

    /// Returns the number of rows affected by the statement.
    #[must_use]
    pub const fn affected_rows(self) -> u64 {
        self.affected_rows
    }
}

/// Stream-capable result of one SQL query.
///
/// Providers may buffer rows internally, but the provider SDK does not require
/// the complete result set to exist in memory before the consumer can process
/// it.
pub struct SqlQueryResult {
    columns: Vec<SqlColumn>,
    rows: Box<dyn Iterator<Item = Result<SqlRow, SqlDatabaseError>> + Send>,
}

impl SqlQueryResult {
    /// Creates a query result from ordered columns and a row iterator.
    #[must_use]
    pub fn new<I>(columns: Vec<SqlColumn>, rows: I) -> Self
    where
        I: Iterator<Item = Result<SqlRow, SqlDatabaseError>> + Send + 'static,
    {
        Self {
            columns,
            rows: Box::new(rows),
        }
    }

    /// Returns ordered query-result column metadata.
    #[must_use]
    pub fn columns(&self) -> &[SqlColumn] {
        &self.columns
    }

    /// Returns mutable access to the row stream.
    #[must_use]
    pub fn rows(
        &mut self,
    ) -> &mut (dyn Iterator<Item = Result<SqlRow, SqlDatabaseError>> + Send + 'static) {
        self.rows.as_mut()
    }

    /// Splits the result into column metadata and its row stream.
    pub fn into_parts(
        self,
    ) -> (
        Vec<SqlColumn>,
        Box<dyn Iterator<Item = Result<SqlRow, SqlDatabaseError>> + Send>,
    ) {
        (self.columns, self.rows)
    }
}

impl fmt::Debug for SqlQueryResult {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("SqlQueryResult")
            .field("columns", &self.columns)
            .field("rows", &"<stream>")
            .finish()
    }
}

/// Transaction isolation modes exposed by the current portable SQL boundary.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum SqlTransactionIsolation {
    /// Use the provider's baseline/default isolation semantics.
    #[default]
    ProviderDefault,

    /// Require serializable transaction semantics.
    ///
    /// The selected provider must advertise the
    /// `serializable-transactions` capability feature.
    Serializable,
}

/// Options used when beginning an explicit SQL transaction.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct SqlTransactionOptions {
    read_only: bool,
    isolation: SqlTransactionIsolation,
}

impl SqlTransactionOptions {
    /// Creates baseline transaction options.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            read_only: false,
            isolation: SqlTransactionIsolation::ProviderDefault,
        }
    }

    /// Requests read-only transaction intent.
    #[must_use]
    pub const fn with_read_only(mut self, read_only: bool) -> Self {
        self.read_only = read_only;
        self
    }

    /// Requests a portable transaction isolation mode.
    #[must_use]
    pub const fn with_isolation(mut self, isolation: SqlTransactionIsolation) -> Self {
        self.isolation = isolation;
        self
    }

    /// Returns whether the transaction requests read-only intent.
    #[must_use]
    pub const fn read_only(self) -> bool {
        self.read_only
    }

    /// Returns the requested transaction isolation mode.
    #[must_use]
    pub const fn isolation(self) -> SqlTransactionIsolation {
        self.isolation
    }
}

/// One explicit relational SQL transaction.
///
/// The transaction object owns the provider-specific logical session. Runtime
/// bindings may expose this object through an opaque resource handle without
/// leaking driver connections or pool identities to application code.
pub trait SqlTransaction: Send {
    /// Executes exactly one statement in this transaction.
    ///
    /// # Errors
    ///
    /// Returns [`SqlDatabaseError`] when execution fails, parameters are
    /// invalid, or the transaction is no longer usable.
    fn execute(
        &mut self,
        statement: &str,
        parameters: &[SqlValue],
    ) -> Result<SqlExecuteResult, SqlDatabaseError>;

    /// Executes exactly one row-producing statement in this transaction.
    ///
    /// # Errors
    ///
    /// Returns [`SqlDatabaseError`] when execution fails, parameters are
    /// invalid, or the transaction is no longer usable.
    fn query(
        &mut self,
        statement: &str,
        parameters: &[SqlValue],
    ) -> Result<SqlQueryResult, SqlDatabaseError>;

    /// Commits this transaction and consumes the transaction object.
    ///
    /// # Errors
    ///
    /// Returns [`SqlDatabaseError`] if commit fails.
    fn commit(self: Box<Self>) -> Result<(), SqlDatabaseError>;

    /// Rolls back this transaction and consumes the transaction object.
    ///
    /// # Errors
    ///
    /// Returns [`SqlDatabaseError`] if rollback fails.
    fn rollback(self: Box<Self>) -> Result<(), SqlDatabaseError>;
}

/// Host implementation of the `presolve:sql/database` capability.
///
/// SQL text remains application-provided. The provider implements the portable
/// execution/value/result/transaction boundary and the dialect/features it
/// advertised during resolution.
pub trait SqlDatabaseProvider: Send + Sync {
    /// Executes exactly one SQL statement outside an explicit transaction.
    ///
    /// # Errors
    ///
    /// Returns [`SqlDatabaseError`] when execution fails or parameters are
    /// invalid.
    fn execute(
        &self,
        statement: &str,
        parameters: &[SqlValue],
    ) -> Result<SqlExecuteResult, SqlDatabaseError>;

    /// Executes exactly one row-producing SQL statement outside an explicit
    /// transaction.
    ///
    /// # Errors
    ///
    /// Returns [`SqlDatabaseError`] when execution fails or parameters are
    /// invalid.
    fn query(
        &self,
        statement: &str,
        parameters: &[SqlValue],
    ) -> Result<SqlQueryResult, SqlDatabaseError>;

    /// Begins one explicit transaction.
    ///
    /// The returned object owns the provider-specific logical transactional
    /// session, structurally preserving session affinity.
    ///
    /// # Errors
    ///
    /// Returns [`SqlDatabaseError`] when the provider cannot begin a transaction
    /// with the requested semantics.
    fn begin(
        &self,
        options: SqlTransactionOptions,
    ) -> Result<Box<dyn SqlTransaction>, SqlDatabaseError>;
}

/// Provider used when no SQL database capability implementation is configured.
#[derive(Debug, Default)]
pub struct UnavailableSqlDatabaseProvider;

impl SqlDatabaseProvider for UnavailableSqlDatabaseProvider {
    fn execute(
        &self,
        _statement: &str,
        _parameters: &[SqlValue],
    ) -> Result<SqlExecuteResult, SqlDatabaseError> {
        Err(SqlDatabaseError::unavailable(
            "SQL database capability is not configured",
        ))
    }

    fn query(
        &self,
        _statement: &str,
        _parameters: &[SqlValue],
    ) -> Result<SqlQueryResult, SqlDatabaseError> {
        Err(SqlDatabaseError::unavailable(
            "SQL database capability is not configured",
        ))
    }

    fn begin(
        &self,
        _options: SqlTransactionOptions,
    ) -> Result<Box<dyn SqlTransaction>, SqlDatabaseError> {
        Err(SqlDatabaseError::unavailable(
            "SQL database capability is not configured",
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scalar_values_preserve_baseline_domain() {
        let values = [
            SqlValue::Null,
            SqlValue::Integer(-7),
            SqlValue::Real(3.5),
            SqlValue::Text("hello".to_owned()),
            SqlValue::Bytes(vec![0, 1, 2]),
        ];

        assert_eq!(values[0], SqlValue::Null);
        assert_eq!(values[1], SqlValue::Integer(-7));
        assert_eq!(values[2], SqlValue::Real(3.5));
        assert_eq!(values[3], SqlValue::Text("hello".to_owned()));
        assert_eq!(values[4], SqlValue::Bytes(vec![0, 1, 2]));
    }

    #[test]
    fn query_result_preserves_duplicate_column_labels_and_position() {
        let columns = vec![SqlColumn::new("id"), SqlColumn::new("id")];
        let rows = vec![Ok(SqlRow::new(vec![
            SqlValue::Integer(1),
            SqlValue::Integer(2),
        ]))]
        .into_iter();
        let mut result = SqlQueryResult::new(columns, rows);

        assert_eq!(result.columns()[0].name(), "id");
        assert_eq!(result.columns()[1].name(), "id");

        let row = result
            .rows()
            .next()
            .expect("one row should exist")
            .expect("row should succeed");
        assert_eq!(row.values(), [SqlValue::Integer(1), SqlValue::Integer(2)]);
    }

    #[test]
    fn transaction_options_default_to_baseline_semantics() {
        let options = SqlTransactionOptions::default();

        assert!(!options.read_only());
        assert_eq!(
            options.isolation(),
            SqlTransactionIsolation::ProviderDefault
        );
    }

    #[test]
    fn error_preserves_code_and_message() {
        let error = SqlDatabaseError::constraint_violation("duplicate key");

        assert_eq!(error.code(), SqlDatabaseErrorCode::ConstraintViolation);
        assert_eq!(error.message(), "duplicate key");
    }

    #[test]
    fn unavailable_provider_fails_explicitly() {
        let provider = UnavailableSqlDatabaseProvider;

        let error = provider
            .execute("update example set value = ?", &[SqlValue::Integer(1)])
            .expect_err("provider should be unavailable");

        assert_eq!(error.code(), SqlDatabaseErrorCode::Unavailable);
    }
}
