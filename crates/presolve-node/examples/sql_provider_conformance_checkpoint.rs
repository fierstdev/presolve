use std::env;

use anyhow::{Context, Result, bail};
use presolve_provider_conformance::{SqlConformanceFixture, assert_sql_database_conformance};
use presolve_provider_realization::SecretValue;
use presolve_provider_sdk::SqlDatabaseProvider;
use presolve_provider_sql_postgres::PostgresSqlProvider;
use presolve_provider_sql_sqlite::SqliteSqlProvider;

const SQLITE_PATH_ENV: &str = "PRESOLVE_SQLITE_PATH";
const POSTGRES_HOST_ENV: &str = "PRESOLVE_POSTGRES_HOST";
const POSTGRES_PORT_ENV: &str = "PRESOLVE_POSTGRES_PORT";
const POSTGRES_DATABASE_ENV: &str = "PRESOLVE_POSTGRES_DATABASE";
const POSTGRES_USER_ENV: &str = "PRESOLVE_POSTGRES_USER";
const POSTGRES_PASSWORD_ENV: &str = "PRESOLVE_POSTGRES_PASSWORD";

const INSERT: &str = "INSERT INTO presolve_conformance (id, label, optional_text, bytes_value) VALUES ($1, $2, $3, $4)";
const SELECT_BY_ID: &str = "SELECT id, label, optional_text, bytes_value FROM presolve_conformance WHERE id = $1 ORDER BY id";
const UPDATE_LABEL: &str = "UPDATE presolve_conformance SET label = $1 WHERE id = $2";
const DELETE_BY_ID: &str = "DELETE FROM presolve_conformance WHERE id = $1";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Mode {
    Sqlite,
    Postgres,
}

impl Mode {
    fn parse(value: &str) -> Result<Self> {
        match value {
            "sqlite" => Ok(Self::Sqlite),
            "postgres" => Ok(Self::Postgres),
            _ => bail!("mode must be `sqlite` or `postgres`, got `{value}`"),
        }
    }

    const fn label(self) -> &'static str {
        match self {
            Self::Sqlite => "sqlite",
            Self::Postgres => "postgres",
        }
    }
}

fn main() -> Result<()> {
    let mut args = env::args().skip(1);
    let mode = Mode::parse(&args.next().context("missing SQL provider mode")?)?;
    if args.next().is_some() {
        bail!("usage: sql_provider_conformance_checkpoint <sqlite|postgres>");
    }

    match mode {
        Mode::Sqlite => {
            let path = required_env(SQLITE_PATH_ENV)?;
            let provider = SqliteSqlProvider::new(path).context("invalid SQLite configuration")?;
            provision_schema(
                &provider,
                "CREATE TABLE IF NOT EXISTS presolve_conformance (id INTEGER PRIMARY KEY, label TEXT NOT NULL, optional_text TEXT, bytes_value BLOB)",
            )?;
            run_conformance(&provider);
        }
        Mode::Postgres => {
            let host = required_env(POSTGRES_HOST_ENV)?;
            let port = env::var(POSTGRES_PORT_ENV)
                .unwrap_or_else(|_| "5432".to_owned())
                .parse::<u16>()
                .context("PRESOLVE_POSTGRES_PORT must be a valid TCP port")?;
            let database = required_env(POSTGRES_DATABASE_ENV)?;
            let user = required_env(POSTGRES_USER_ENV)?;
            let password = SecretValue::new(required_env(POSTGRES_PASSWORD_ENV)?);
            let provider = PostgresSqlProvider::new(host, port, database, user, password)
                .context("invalid PostgreSQL configuration")?;
            provision_schema(
                &provider,
                "CREATE TABLE IF NOT EXISTS presolve_conformance (id BIGINT PRIMARY KEY, label TEXT NOT NULL, optional_text TEXT, bytes_value BYTEA)",
            )?;
            run_conformance(&provider);
        }
    }

    println!("SQL provider conformance passed: {}", mode.label());
    Ok(())
}

fn run_conformance(provider: &dyn SqlDatabaseProvider) {
    assert_sql_database_conformance(
        provider,
        SqlConformanceFixture {
            insert: INSERT,
            select_by_id: SELECT_BY_ID,
            update_label: UPDATE_LABEL,
            delete_by_id: DELETE_BY_ID,
        },
    );
}

fn provision_schema(provider: &dyn SqlDatabaseProvider, statement: &str) -> Result<()> {
    provider
        .execute(statement, &[])
        .map(|_| ())
        .context("could not provision SQL conformance schema")
}

fn required_env(name: &str) -> Result<String> {
    env::var(name).with_context(|| format!("required environment variable `{name}` is not set"))
}
