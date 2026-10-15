#![forbid(unsafe_code)]

//! SQLite realization of `presolve:sql/database@0.1.0`.
//!
//! Database paths are node-side realization configuration and never enter the
//! application contract, deployment plan, or immutable `.presolved` release.

use std::{path::PathBuf, sync::Arc};

use presolve_capability::{
    SQL_DATABASE_FEATURE_DIALECT_SQLITE, SQL_DATABASE_FEATURE_READ_ONLY_TRANSACTIONS,
    SQL_DATABASE_FEATURE_RETURNING, SQL_DATABASE_FEATURE_SAVEPOINTS,
    SQL_DATABASE_FEATURE_SERIALIZABLE_TRANSACTIONS,
};
use presolve_provider_sdk::{
    SQL_DATABASE_INTERFACE, SqlColumn, SqlDatabaseError, SqlDatabaseProvider, SqlExecuteResult,
    SqlQueryResult, SqlRow, SqlTransaction, SqlTransactionIsolation, SqlTransactionOptions,
    SqlValue,
};
use rusqlite::{Connection, Error as SqliteError, params_from_iter, types::ValueRef};
use thiserror::Error;

/// Node-side adapter identity for SQLite SQL.
pub const SQLITE_SQL_ADAPTER_ID: &str = "sqlite:sql";

/// Provider-realization setting containing the SQLite database path.
pub const SQLITE_PATH_SETTING: &str = "path";

/// SQLite provider configuration failure.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum SqliteSqlConfigError {
    /// Database path is empty.
    #[error("SQLite database path must not be empty")]
    EmptyPath,
}

/// File-backed SQLite implementation of Presolve's portable SQL boundary.
#[derive(Debug, Clone)]
pub struct SqliteSqlProvider {
    path: PathBuf,
}

impl SqliteSqlProvider {
    /// Creates a SQLite provider bound to one database file.
    ///
    /// # Errors
    ///
    /// Returns [`SqliteSqlConfigError`] if `path` is empty.
    pub fn new(path: impl Into<PathBuf>) -> Result<Self, SqliteSqlConfigError> {
        let path = path.into();
        if path.as_os_str().is_empty() {
            return Err(SqliteSqlConfigError::EmptyPath);
        }
        Ok(Self { path })
    }

    fn connect(&self) -> Result<Connection, SqlDatabaseError> {
        Connection::open(&self.path).map_err(map_sqlite_error)
    }
}

impl SqlDatabaseProvider for SqliteSqlProvider {
    fn execute(
        &self,
        statement: &str,
        parameters: &[SqlValue],
    ) -> Result<SqlExecuteResult, SqlDatabaseError> {
        execute_on(&self.connect()?, statement, parameters)
    }

    fn query(
        &self,
        statement: &str,
        parameters: &[SqlValue],
    ) -> Result<SqlQueryResult, SqlDatabaseError> {
        query_on(&self.connect()?, statement, parameters)
    }

    fn begin(
        &self,
        options: SqlTransactionOptions,
    ) -> Result<Box<dyn SqlTransaction>, SqlDatabaseError> {
        let connection = self.connect()?;

        if options.read_only() {
            connection
                .execute_batch("PRAGMA query_only = ON")
                .map_err(map_sqlite_error)?;
        }

        match options.isolation() {
            SqlTransactionIsolation::ProviderDefault | SqlTransactionIsolation::Serializable => {
                connection
                    .execute_batch("BEGIN")
                    .map_err(map_sqlite_error)?;
            }
        }

        Ok(Box::new(SqliteTransaction {
            connection: Some(connection),
            active: true,
        }))
    }
}

struct SqliteTransaction {
    connection: Option<Connection>,
    active: bool,
}

impl SqliteTransaction {
    fn connection(&self) -> Result<&Connection, SqlDatabaseError> {
        self.connection
            .as_ref()
            .filter(|_| self.active)
            .ok_or_else(|| SqlDatabaseError::transaction_state("SQLite transaction is closed"))
    }
}

impl SqlTransaction for SqliteTransaction {
    fn execute(
        &mut self,
        statement: &str,
        parameters: &[SqlValue],
    ) -> Result<SqlExecuteResult, SqlDatabaseError> {
        execute_on(self.connection()?, statement, parameters)
    }

    fn query(
        &mut self,
        statement: &str,
        parameters: &[SqlValue],
    ) -> Result<SqlQueryResult, SqlDatabaseError> {
        query_on(self.connection()?, statement, parameters)
    }

    fn commit(mut self: Box<Self>) -> Result<(), SqlDatabaseError> {
        self.connection()?
            .execute_batch("COMMIT")
            .map_err(map_sqlite_error)?;
        self.active = false;
        self.connection.take();
        Ok(())
    }

    fn rollback(mut self: Box<Self>) -> Result<(), SqlDatabaseError> {
        self.connection()?
            .execute_batch("ROLLBACK")
            .map_err(map_sqlite_error)?;
        self.active = false;
        self.connection.take();
        Ok(())
    }
}

impl Drop for SqliteTransaction {
    fn drop(&mut self) {
        if self.active {
            if let Some(connection) = self.connection.as_ref() {
                let _ = connection.execute_batch("ROLLBACK");
            }
            self.active = false;
        }
    }
}

fn execute_on(
    connection: &Connection,
    statement: &str,
    parameters: &[SqlValue],
) -> Result<SqlExecuteResult, SqlDatabaseError> {
    let values = parameters.iter().map(sqlite_value).collect::<Vec<_>>();
    let affected = connection
        .execute(statement, params_from_iter(values))
        .map_err(map_sqlite_error)?;
    Ok(SqlExecuteResult::new(affected as u64))
}

fn query_on(
    connection: &Connection,
    statement: &str,
    parameters: &[SqlValue],
) -> Result<SqlQueryResult, SqlDatabaseError> {
    let values = parameters.iter().map(sqlite_value).collect::<Vec<_>>();
    let mut prepared = connection.prepare(statement).map_err(map_sqlite_error)?;
    let columns = prepared
        .column_names()
        .iter()
        .map(|name| SqlColumn::new(*name))
        .collect::<Vec<_>>();
    let column_count = columns.len();
    let mut rows = prepared
        .query(params_from_iter(values))
        .map_err(map_sqlite_error)?;
    let mut buffered = Vec::new();

    while let Some(row) = rows.next().map_err(map_sqlite_error)? {
        let mut values = Vec::with_capacity(column_count);
        for index in 0..column_count {
            values.push(sqlite_row_value(
                row.get_ref(index).map_err(map_sqlite_error)?,
            )?);
        }
        buffered.push(Ok(SqlRow::new(values)));
    }

    Ok(SqlQueryResult::new(columns, buffered.into_iter()))
}

fn sqlite_value(value: &SqlValue) -> rusqlite::types::Value {
    match value {
        SqlValue::Null => rusqlite::types::Value::Null,
        SqlValue::Integer(value) => rusqlite::types::Value::Integer(*value),
        SqlValue::Real(value) => rusqlite::types::Value::Real(*value),
        SqlValue::Text(value) => rusqlite::types::Value::Text(value.clone()),
        SqlValue::Bytes(value) => rusqlite::types::Value::Blob(value.clone()),
    }
}

fn sqlite_row_value(value: ValueRef<'_>) -> Result<SqlValue, SqlDatabaseError> {
    match value {
        ValueRef::Null => Ok(SqlValue::Null),
        ValueRef::Integer(value) => Ok(SqlValue::Integer(value)),
        ValueRef::Real(value) => Ok(SqlValue::Real(value)),
        ValueRef::Text(value) => std::str::from_utf8(value)
            .map(|value| SqlValue::Text(value.to_owned()))
            .map_err(|error| {
                SqlDatabaseError::internal(format!("SQLite returned invalid UTF-8 text: {error}"))
            }),
        ValueRef::Blob(value) => Ok(SqlValue::Bytes(value.to_vec())),
    }
}

fn map_sqlite_error(error: SqliteError) -> SqlDatabaseError {
    use rusqlite::ErrorCode;

    match error.sqlite_error_code() {
        Some(ErrorCode::ConstraintViolation) => {
            SqlDatabaseError::constraint_violation(error.to_string())
        }
        Some(ErrorCode::DatabaseBusy | ErrorCode::DatabaseLocked) => {
            SqlDatabaseError::transient(error.to_string())
        }
        Some(ErrorCode::ReadOnly) => SqlDatabaseError::constraint_violation(error.to_string()),
        _ => match error {
            SqliteError::InvalidParameterName(_) | SqliteError::InvalidParameterCount(_, _) => {
                SqlDatabaseError::invalid_parameter(error.to_string())
            }
            SqliteError::SqlInputError { .. } => {
                SqlDatabaseError::invalid_statement(error.to_string())
            }
            _ => SqlDatabaseError::internal(error.to_string()),
        },
    }
}

fn supported_feature(feature: &str) -> bool {
    matches!(
        feature,
        SQL_DATABASE_FEATURE_DIALECT_SQLITE
            | SQL_DATABASE_FEATURE_READ_ONLY_TRANSACTIONS
            | SQL_DATABASE_FEATURE_RETURNING
            | SQL_DATABASE_FEATURE_SAVEPOINTS
            | SQL_DATABASE_FEATURE_SERIALIZABLE_TRANSACTIONS
    )
}

/// Provider-realization adapter for SQLite SQL.
#[derive(Debug, Default, Clone, Copy)]
pub struct SqliteSqlAdapter;

impl presolve_provider_realization::ProviderAdapter for SqliteSqlAdapter {
    fn realize(
        &self,
        profile: &presolve_provider_realization::ProviderRealizationProfile,
        requested: &[presolve_provider_realization::ProviderCapabilityRequest],
        _secrets: &dyn presolve_provider_realization::SecretResolver,
    ) -> Result<
        presolve_provider_realization::RealizedProvider,
        presolve_provider_realization::ProviderAdapterError,
    > {
        use presolve_provider_realization::ProviderAdapterError;
        use semver::Version;

        let supported_version = Version::new(0, 1, 0);
        for capability in requested {
            if capability.interface() != SQL_DATABASE_INTERFACE {
                return Err(ProviderAdapterError::UnsupportedCapability {
                    interface: capability.interface().to_owned(),
                });
            }
            if capability.version() != &supported_version {
                return Err(ProviderAdapterError::UnsupportedVersion {
                    interface: SQL_DATABASE_INTERFACE.to_owned(),
                    selected_version: capability.version().clone(),
                    supported_version: supported_version.clone(),
                });
            }
            let unsupported = capability
                .negotiated_features()
                .iter()
                .filter(|feature| !supported_feature(feature))
                .cloned()
                .collect::<Vec<_>>();
            if !unsupported.is_empty() {
                return Err(ProviderAdapterError::UnsupportedFeatures {
                    interface: SQL_DATABASE_INTERFACE.to_owned(),
                    features: unsupported.join(", "),
                });
            }
        }

        let path = profile.setting(SQLITE_PATH_SETTING).ok_or_else(|| {
            ProviderAdapterError::MissingSetting {
                key: SQLITE_PATH_SETTING.to_owned(),
            }
        })?;
        let provider =
            SqliteSqlProvider::new(path).map_err(|error| ProviderAdapterError::Configuration {
                message: error.to_string(),
            })?;

        Ok(
            presolve_provider_realization::RealizedProvider::new(*profile.provider_id())
                .with_sql_database(Arc::new(provider)),
        )
    }
}

#[cfg(test)]
mod tests {
    use presolve_provider_conformance::{SqlConformanceFixture, assert_sql_database_conformance};

    use super::*;

    #[test]
    fn sqlite_accepts_shared_postgres_style_parameter_markers() {
        let path =
            std::env::temp_dir().join(format!("presolve-sqlite-marker-{}.db", std::process::id()));
        let _ = std::fs::remove_file(&path);
        let provider = SqliteSqlProvider::new(&path).expect("provider should configure");
        provider
            .execute(
                "CREATE TABLE marker_test (id INTEGER PRIMARY KEY, label TEXT NOT NULL)",
                &[],
            )
            .expect("schema should create");
        provider
            .execute(
                "INSERT INTO marker_test (id, label) VALUES ($1, $2)",
                &[SqlValue::Integer(1), SqlValue::Text("ok".to_owned())],
            )
            .expect("$N markers should bind on SQLite");
        let result = provider
            .query(
                "SELECT label FROM marker_test WHERE id = $1",
                &[SqlValue::Integer(1)],
            )
            .expect("query should succeed");
        let (_, mut rows) = result.into_parts();
        assert_eq!(
            rows.next()
                .expect("row should exist")
                .expect("row should decode")
                .values(),
            [SqlValue::Text("ok".to_owned())]
        );
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn sqlite_satisfies_sql_conformance_suite() {
        let path = std::env::temp_dir().join(format!(
            "presolve-sqlite-conformance-{}.db",
            std::process::id()
        ));
        let _ = std::fs::remove_file(&path);
        let provider = SqliteSqlProvider::new(&path).expect("provider should configure");
        provider
            .execute(
                "CREATE TABLE presolve_conformance (id INTEGER PRIMARY KEY, label TEXT NOT NULL, optional_value BLOB)",
                &[],
            )
            .expect("schema should create");
        assert_sql_database_conformance(
            &provider,
            SqlConformanceFixture {
                insert: "INSERT INTO presolve_conformance (id, label, optional_value) VALUES ($1, $2, $3)",
                select_by_id: "SELECT id, label, optional_value FROM presolve_conformance WHERE id = $1 ORDER BY id",
                update_label: "UPDATE presolve_conformance SET label = $1 WHERE id = $2",
                delete_by_id: "DELETE FROM presolve_conformance WHERE id = $1",
            },
        );
        let _ = std::fs::remove_file(path);
    }
}
