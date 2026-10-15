#![forbid(unsafe_code)]

//! PostgreSQL realization of `presolve:sql/database@0.1.0`.
//!
//! Connection identity and credentials are node-side realization concerns. The
//! initial adapter uses direct synchronous connections and `NoTls`; connection
//! pooling and TLS policy remain operational concerns outside the baseline SQL
//! capability semantics.

use std::{error::Error, sync::Arc};

use bytes::BytesMut;
use postgres::{
    Client, Config, NoTls, Row,
    types::{IsNull, ToSql, Type, to_sql_checked},
};
use presolve_capability::{
    SQL_DATABASE_FEATURE_DIALECT_POSTGRESQL, SQL_DATABASE_FEATURE_READ_ONLY_TRANSACTIONS,
    SQL_DATABASE_FEATURE_RETURNING, SQL_DATABASE_FEATURE_SAVEPOINTS,
    SQL_DATABASE_FEATURE_SERIALIZABLE_TRANSACTIONS,
};
use presolve_provider_realization::SecretValue;
use presolve_provider_sdk::{
    SQL_DATABASE_INTERFACE, SqlColumn, SqlDatabaseError, SqlDatabaseProvider, SqlExecuteResult,
    SqlQueryResult, SqlRow, SqlTransaction, SqlTransactionIsolation, SqlTransactionOptions,
    SqlValue,
};
use thiserror::Error;

/// Node-side adapter identity for PostgreSQL SQL.
pub const POSTGRES_SQL_ADAPTER_ID: &str = "postgres:sql";

/// Provider-realization setting containing the PostgreSQL host.
pub const POSTGRES_HOST_SETTING: &str = "host";
/// Provider-realization setting containing the PostgreSQL port.
pub const POSTGRES_PORT_SETTING: &str = "port";
/// Provider-realization setting containing the PostgreSQL database name.
pub const POSTGRES_DATABASE_SETTING: &str = "database";
/// Provider-realization setting containing the PostgreSQL user.
pub const POSTGRES_USER_SETTING: &str = "user";
/// Provider-realization secret binding containing the PostgreSQL password.
pub const POSTGRES_PASSWORD_SECRET: &str = "password";

/// PostgreSQL provider configuration failure.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum PostgresSqlConfigError {
    /// A required text setting is empty.
    #[error("PostgreSQL {field} must not be empty")]
    EmptyField { field: &'static str },
    /// Port is not a valid TCP port.
    #[error("PostgreSQL port `{value}` is invalid")]
    InvalidPort { value: String },
    /// Password is empty.
    #[error("PostgreSQL password must not be empty")]
    EmptyPassword,
}

/// Direct-connection PostgreSQL implementation of Presolve's portable SQL boundary.
#[derive(Clone)]
pub struct PostgresSqlProvider {
    host: String,
    port: u16,
    database: String,
    user: String,
    password: SecretValue,
}

impl std::fmt::Debug for PostgresSqlProvider {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("PostgresSqlProvider")
            .field("host", &self.host)
            .field("port", &self.port)
            .field("database", &self.database)
            .field("user", &self.user)
            .field("password", &"[redacted]")
            .finish()
    }
}

impl PostgresSqlProvider {
    /// Creates a PostgreSQL provider.
    ///
    /// # Errors
    ///
    /// Returns [`PostgresSqlConfigError`] for invalid connection settings.
    pub fn new(
        host: impl Into<String>,
        port: u16,
        database: impl Into<String>,
        user: impl Into<String>,
        password: SecretValue,
    ) -> Result<Self, PostgresSqlConfigError> {
        let host = host.into();
        let database = database.into();
        let user = user.into();
        for (field, value) in [("host", &host), ("database", &database), ("user", &user)] {
            if value.trim().is_empty() {
                return Err(PostgresSqlConfigError::EmptyField { field });
            }
        }
        if port == 0 {
            return Err(PostgresSqlConfigError::InvalidPort {
                value: port.to_string(),
            });
        }
        if password.expose_secret().is_empty() {
            return Err(PostgresSqlConfigError::EmptyPassword);
        }
        Ok(Self {
            host,
            port,
            database,
            user,
            password,
        })
    }

    fn connect(&self) -> Result<Client, SqlDatabaseError> {
        let mut config = Config::new();
        config
            .host(&self.host)
            .port(self.port)
            .dbname(&self.database)
            .user(&self.user)
            .password(self.password.expose_secret());
        config
            .connect(NoTls)
            .map_err(|error| map_postgres_error(&error))
    }
}

impl SqlDatabaseProvider for PostgresSqlProvider {
    fn execute(
        &self,
        statement: &str,
        parameters: &[SqlValue],
    ) -> Result<SqlExecuteResult, SqlDatabaseError> {
        execute_on(&mut self.connect()?, statement, parameters)
    }

    fn query(
        &self,
        statement: &str,
        parameters: &[SqlValue],
    ) -> Result<SqlQueryResult, SqlDatabaseError> {
        query_on(&mut self.connect()?, statement, parameters)
    }

    fn begin(
        &self,
        options: SqlTransactionOptions,
    ) -> Result<Box<dyn SqlTransaction>, SqlDatabaseError> {
        let mut client = self.connect()?;
        let mut begin = String::from("BEGIN");
        if matches!(options.isolation(), SqlTransactionIsolation::Serializable) {
            begin.push_str(" ISOLATION LEVEL SERIALIZABLE");
        }
        if options.read_only() {
            begin.push_str(" READ ONLY");
        }
        client
            .batch_execute(&begin)
            .map_err(|error| map_postgres_error(&error))?;
        Ok(Box::new(PostgresTransaction {
            client: Some(client),
            active: true,
        }))
    }
}

struct PostgresTransaction {
    client: Option<Client>,
    active: bool,
}

impl PostgresTransaction {
    fn client(&mut self) -> Result<&mut Client, SqlDatabaseError> {
        self.client
            .as_mut()
            .filter(|_| self.active)
            .ok_or_else(|| SqlDatabaseError::transaction_state("PostgreSQL transaction is closed"))
    }
}

impl SqlTransaction for PostgresTransaction {
    fn execute(
        &mut self,
        statement: &str,
        parameters: &[SqlValue],
    ) -> Result<SqlExecuteResult, SqlDatabaseError> {
        execute_on(self.client()?, statement, parameters)
    }

    fn query(
        &mut self,
        statement: &str,
        parameters: &[SqlValue],
    ) -> Result<SqlQueryResult, SqlDatabaseError> {
        query_on(self.client()?, statement, parameters)
    }

    fn commit(mut self: Box<Self>) -> Result<(), SqlDatabaseError> {
        self.client()?
            .batch_execute("COMMIT")
            .map_err(|error| map_postgres_error(&error))?;
        self.active = false;
        self.client.take();
        Ok(())
    }

    fn rollback(mut self: Box<Self>) -> Result<(), SqlDatabaseError> {
        self.client()?
            .batch_execute("ROLLBACK")
            .map_err(|error| map_postgres_error(&error))?;
        self.active = false;
        self.client.take();
        Ok(())
    }
}

impl Drop for PostgresTransaction {
    fn drop(&mut self) {
        if self.active {
            if let Some(client) = self.client.as_mut() {
                let _ = client.batch_execute("ROLLBACK");
            }
            self.active = false;
        }
    }
}

#[derive(Debug)]
struct PgNull;

impl ToSql for PgNull {
    fn to_sql(
        &self,
        _ty: &Type,
        _out: &mut BytesMut,
    ) -> Result<IsNull, Box<dyn Error + Sync + Send>> {
        Ok(IsNull::Yes)
    }

    fn accepts(_ty: &Type) -> bool {
        true
    }

    to_sql_checked!();
}

fn postgres_parameters(parameters: &[SqlValue]) -> Vec<Box<dyn ToSql + Sync>> {
    parameters
        .iter()
        .map(|value| -> Box<dyn ToSql + Sync> {
            match value {
                SqlValue::Null => Box::new(PgNull),
                SqlValue::Integer(value) => Box::new(*value),
                SqlValue::Real(value) => Box::new(*value),
                SqlValue::Text(value) => Box::new(value.clone()),
                SqlValue::Bytes(value) => Box::new(value.clone()),
            }
        })
        .collect()
}

fn execute_on(
    client: &mut Client,
    statement: &str,
    parameters: &[SqlValue],
) -> Result<SqlExecuteResult, SqlDatabaseError> {
    let owned = postgres_parameters(parameters);
    let refs = owned
        .iter()
        .map(|value| value.as_ref() as &(dyn ToSql + Sync))
        .collect::<Vec<_>>();
    let affected = client
        .execute(statement, &refs)
        .map_err(|error| map_postgres_error(&error))?;
    Ok(SqlExecuteResult::new(affected))
}

fn query_on(
    client: &mut Client,
    statement: &str,
    parameters: &[SqlValue],
) -> Result<SqlQueryResult, SqlDatabaseError> {
    let owned = postgres_parameters(parameters);
    let refs = owned
        .iter()
        .map(|value| value.as_ref() as &(dyn ToSql + Sync))
        .collect::<Vec<_>>();
    let rows = client
        .query(statement, &refs)
        .map_err(|error| map_postgres_error(&error))?;
    let columns = rows.first().map_or_else(Vec::new, |row| {
        row.columns()
            .iter()
            .map(|column| SqlColumn::new(column.name()))
            .collect()
    });
    let buffered = rows
        .into_iter()
        .map(|row| postgres_row(&row))
        .collect::<Vec<_>>();
    Ok(SqlQueryResult::new(columns, buffered.into_iter()))
}

fn postgres_row(row: &Row) -> Result<SqlRow, SqlDatabaseError> {
    let mut values = Vec::with_capacity(row.len());
    for (index, column) in row.columns().iter().enumerate() {
        values.push(postgres_row_value(row, index, column.type_())?);
    }
    Ok(SqlRow::new(values))
}

fn postgres_row_value(row: &Row, index: usize, ty: &Type) -> Result<SqlValue, SqlDatabaseError> {
    macro_rules! optional {
        ($rust:ty, $map:expr) => {{
            let value: Option<$rust> = row
                .try_get(index)
                .map_err(|error| map_postgres_error(&error))?;
            Ok(value.map_or(SqlValue::Null, $map))
        }};
    }

    match *ty {
        Type::INT2 => optional!(i16, |value| SqlValue::Integer(i64::from(value))),
        Type::INT4 => optional!(i32, |value| SqlValue::Integer(i64::from(value))),
        Type::INT8 => optional!(i64, SqlValue::Integer),
        Type::FLOAT4 => optional!(f32, |value| SqlValue::Real(f64::from(value))),
        Type::FLOAT8 => optional!(f64, SqlValue::Real),
        Type::TEXT | Type::VARCHAR | Type::BPCHAR | Type::NAME => optional!(String, SqlValue::Text),
        Type::BYTEA => optional!(Vec<u8>, SqlValue::Bytes),
        _ => Err(SqlDatabaseError::unsupported(format!(
            "PostgreSQL result type `{}` is outside the portable SQL scalar baseline",
            ty.name()
        ))),
    }
}

fn map_postgres_error(error: &postgres::Error) -> SqlDatabaseError {
    let message = error.to_string();
    let Some(db_error) = error.as_db_error() else {
        return SqlDatabaseError::unavailable(message);
    };
    let code = db_error.code().code();
    if code.starts_with("23") {
        SqlDatabaseError::constraint_violation(message)
    } else if code.starts_with("40") || code == "55P03" {
        SqlDatabaseError::transient(message)
    } else if code.starts_with("08") {
        SqlDatabaseError::unavailable(message)
    } else if matches!(code, "42601" | "42P01" | "42703") {
        SqlDatabaseError::invalid_statement(message)
    } else {
        SqlDatabaseError::internal(message)
    }
}

fn supported_feature(feature: &str) -> bool {
    matches!(
        feature,
        SQL_DATABASE_FEATURE_DIALECT_POSTGRESQL
            | SQL_DATABASE_FEATURE_READ_ONLY_TRANSACTIONS
            | SQL_DATABASE_FEATURE_RETURNING
            | SQL_DATABASE_FEATURE_SAVEPOINTS
            | SQL_DATABASE_FEATURE_SERIALIZABLE_TRANSACTIONS
    )
}

/// Provider-realization adapter for PostgreSQL SQL.
#[derive(Debug, Default, Clone, Copy)]
pub struct PostgresSqlAdapter;

impl presolve_provider_realization::ProviderAdapter for PostgresSqlAdapter {
    fn realize(
        &self,
        profile: &presolve_provider_realization::ProviderRealizationProfile,
        requested: &[presolve_provider_realization::ProviderCapabilityRequest],
        secrets: &dyn presolve_provider_realization::SecretResolver,
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

        let host = required_setting(profile, POSTGRES_HOST_SETTING)?;
        let database = required_setting(profile, POSTGRES_DATABASE_SETTING)?;
        let user = required_setting(profile, POSTGRES_USER_SETTING)?;
        let port_text = profile.setting(POSTGRES_PORT_SETTING).unwrap_or("5432");
        let port = port_text
            .parse::<u16>()
            .map_err(|_| ProviderAdapterError::Configuration {
                message: PostgresSqlConfigError::InvalidPort {
                    value: port_text.to_owned(),
                }
                .to_string(),
            })?;
        let password_ref = profile.secret(POSTGRES_PASSWORD_SECRET).ok_or_else(|| {
            ProviderAdapterError::MissingSecret {
                name: POSTGRES_PASSWORD_SECRET.to_owned(),
            }
        })?;
        let password = secrets.resolve(password_ref).map_err(|error| {
            ProviderAdapterError::SecretResolution {
                name: POSTGRES_PASSWORD_SECRET.to_owned(),
                message: error.to_string(),
            }
        })?;
        let provider =
            PostgresSqlProvider::new(host, port, database, user, password).map_err(|error| {
                ProviderAdapterError::Configuration {
                    message: error.to_string(),
                }
            })?;

        Ok(
            presolve_provider_realization::RealizedProvider::new(*profile.provider_id())
                .with_sql_database(Arc::new(provider)),
        )
    }
}

fn required_setting<'a>(
    profile: &'a presolve_provider_realization::ProviderRealizationProfile,
    key: &str,
) -> Result<&'a str, presolve_provider_realization::ProviderAdapterError> {
    profile.setting(key).ok_or_else(|| {
        presolve_provider_realization::ProviderAdapterError::MissingSetting {
            key: key.to_owned(),
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn debug_output_redacts_password() {
        let provider = PostgresSqlProvider::new(
            "127.0.0.1",
            5432,
            "presolve",
            "presolve",
            SecretValue::new("do-not-print"),
        )
        .expect("provider should configure");
        let debug = format!("{provider:?}");
        assert!(!debug.contains("do-not-print"));
        assert!(debug.contains("[redacted]"));
    }

    #[test]
    fn rejects_zero_port() {
        assert!(matches!(
            PostgresSqlProvider::new(
                "127.0.0.1",
                0,
                "presolve",
                "presolve",
                SecretValue::new("secret"),
            ),
            Err(PostgresSqlConfigError::InvalidPort { .. })
        ));
    }
}
