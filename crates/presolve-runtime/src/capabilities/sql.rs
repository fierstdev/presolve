use presolve_provider_sdk::{
    SqlDatabaseError, SqlDatabaseErrorCode, SqlExecuteResult, SqlRow, SqlTransactionIsolation,
    SqlTransactionOptions, SqlValue,
};
use wasmtime::component::Resource;

use crate::{bindings::sql::presolve::sql::database, state::RuntimeState};

impl database::Host for RuntimeState {
    fn execute(
        &mut self,
        statement: String,
        parameters: Vec<database::Value>,
    ) -> Result<database::ExecuteResult, database::Error> {
        let parameters = parameters
            .into_iter()
            .map(map_value_from_wit)
            .collect::<Vec<_>>();
        self.sql_database_provider()
            .execute(&statement, &parameters)
            .map(map_execute_result)
            .map_err(|error| map_error(&error))
    }

    fn query(
        &mut self,
        statement: String,
        parameters: Vec<database::Value>,
    ) -> Result<Resource<database::QueryResult>, database::Error> {
        if !self.can_open_sql_session() {
            return Err(database::Error::Internal(
                "too many open SQL runtime sessions".to_owned(),
            ));
        }

        let parameters = parameters
            .into_iter()
            .map(map_value_from_wit)
            .collect::<Vec<_>>();
        let result = self
            .sql_database_provider()
            .query(&statement, &parameters)
            .map_err(|error| map_error(&error))?;
        let handle = self.allocate_sql_handle().ok_or_else(|| {
            database::Error::Internal("SQL runtime handle space exhausted".to_owned())
        })?;

        self.insert_sql_query_result(handle, result);
        Ok(Resource::new_own(handle))
    }

    fn begin(
        &mut self,
        options: database::TransactionOptions,
    ) -> Result<Resource<database::Transaction>, database::Error> {
        if !self.can_open_sql_session() {
            return Err(database::Error::Internal(
                "too many open SQL runtime sessions".to_owned(),
            ));
        }

        let transaction = self
            .sql_database_provider()
            .begin(map_transaction_options(options))
            .map_err(|error| map_error(&error))?;
        let handle = self.allocate_sql_handle().ok_or_else(|| {
            database::Error::Internal("SQL runtime handle space exhausted".to_owned())
        })?;

        self.insert_sql_transaction(handle, transaction);
        Ok(Resource::new_own(handle))
    }

    fn commit(
        &mut self,
        transaction: Resource<database::Transaction>,
    ) -> Result<(), database::Error> {
        let handle = transaction.rep();
        let transaction = self.remove_sql_transaction(handle).ok_or_else(|| {
            database::Error::TransactionState("unknown SQL transaction resource".to_owned())
        })?;

        transaction.commit().map_err(|error| map_error(&error))
    }

    fn rollback(
        &mut self,
        transaction: Resource<database::Transaction>,
    ) -> Result<(), database::Error> {
        let handle = transaction.rep();
        let transaction = self.remove_sql_transaction(handle).ok_or_else(|| {
            database::Error::TransactionState("unknown SQL transaction resource".to_owned())
        })?;

        transaction.rollback().map_err(|error| map_error(&error))
    }
}

impl database::HostTransaction for RuntimeState {
    fn execute(
        &mut self,
        transaction: Resource<database::Transaction>,
        statement: String,
        parameters: Vec<database::Value>,
    ) -> Result<database::ExecuteResult, database::Error> {
        let parameters = parameters
            .into_iter()
            .map(map_value_from_wit)
            .collect::<Vec<_>>();
        let transaction = self.sql_transaction_mut(transaction.rep()).ok_or_else(|| {
            database::Error::TransactionState("unknown SQL transaction resource".to_owned())
        })?;

        transaction
            .execute(&statement, &parameters)
            .map(map_execute_result)
            .map_err(|error| map_error(&error))
    }

    fn query(
        &mut self,
        transaction: Resource<database::Transaction>,
        statement: String,
        parameters: Vec<database::Value>,
    ) -> Result<Resource<database::QueryResult>, database::Error> {
        if !self.can_open_sql_session() {
            return Err(database::Error::Internal(
                "too many open SQL runtime sessions".to_owned(),
            ));
        }

        let parameters = parameters
            .into_iter()
            .map(map_value_from_wit)
            .collect::<Vec<_>>();
        let result = {
            let transaction = self.sql_transaction_mut(transaction.rep()).ok_or_else(|| {
                database::Error::TransactionState("unknown SQL transaction resource".to_owned())
            })?;

            transaction
                .query(&statement, &parameters)
                .map_err(|error| map_error(&error))?
        };
        let handle = self.allocate_sql_handle().ok_or_else(|| {
            database::Error::Internal("SQL runtime handle space exhausted".to_owned())
        })?;

        self.insert_sql_query_result(handle, result);
        Ok(Resource::new_own(handle))
    }

    fn drop(&mut self, transaction: Resource<database::Transaction>) -> wasmtime::Result<()> {
        let Some(transaction) = self.remove_sql_transaction(transaction.rep()) else {
            return Ok(());
        };

        transaction
            .rollback()
            .map_err(|error| wasmtime::Error::msg(format!("implicit SQL rollback failed: {error}")))
    }
}

impl database::HostQueryResult for RuntimeState {
    fn columns(
        &mut self,
        result: Resource<database::QueryResult>,
    ) -> wasmtime::Result<Vec<database::Column>> {
        let result = self
            .sql_query_result_mut(result.rep())
            .ok_or_else(|| wasmtime::Error::msg("unknown SQL query-result resource"))?;

        Ok(result
            .columns()
            .iter()
            .map(|column| database::Column {
                name: column.name().to_owned(),
            })
            .collect())
    }

    fn next_row(
        &mut self,
        result: Resource<database::QueryResult>,
    ) -> Result<Option<database::Row>, database::Error> {
        let result = self.sql_query_result_mut(result.rep()).ok_or_else(|| {
            database::Error::Internal("unknown SQL query-result resource".to_owned())
        })?;

        match result.rows().next() {
            Some(Ok(row)) => Ok(Some(map_row(row))),
            Some(Err(error)) => Err(map_error(&error)),
            None => Ok(None),
        }
    }

    fn drop(&mut self, result: Resource<database::QueryResult>) -> wasmtime::Result<()> {
        self.remove_sql_query_result(result.rep());
        Ok(())
    }
}

fn map_value_from_wit(value: database::Value) -> SqlValue {
    match value {
        database::Value::Null => SqlValue::Null,
        database::Value::Integer(value) => SqlValue::Integer(value),
        database::Value::Real(value) => SqlValue::Real(value),
        database::Value::Text(value) => SqlValue::Text(value),
        database::Value::Bytes(value) => SqlValue::Bytes(value),
    }
}

fn map_value_to_wit(value: SqlValue) -> database::Value {
    match value {
        SqlValue::Null => database::Value::Null,
        SqlValue::Integer(value) => database::Value::Integer(value),
        SqlValue::Real(value) => database::Value::Real(value),
        SqlValue::Text(value) => database::Value::Text(value),
        SqlValue::Bytes(value) => database::Value::Bytes(value),
    }
}

fn map_row(row: SqlRow) -> database::Row {
    database::Row {
        values: row
            .into_values()
            .into_iter()
            .map(map_value_to_wit)
            .collect(),
    }
}

fn map_execute_result(result: SqlExecuteResult) -> database::ExecuteResult {
    database::ExecuteResult {
        affected_rows: result.affected_rows(),
    }
}

fn map_transaction_options(options: database::TransactionOptions) -> SqlTransactionOptions {
    let isolation = match options.isolation {
        database::Isolation::ProviderDefault => SqlTransactionIsolation::ProviderDefault,
        database::Isolation::Serializable => SqlTransactionIsolation::Serializable,
    };

    SqlTransactionOptions::new()
        .with_read_only(options.read_only)
        .with_isolation(isolation)
}

fn map_error(error: &SqlDatabaseError) -> database::Error {
    let message = error.message().to_owned();

    match error.code() {
        SqlDatabaseErrorCode::Unavailable => database::Error::Unavailable(message),
        SqlDatabaseErrorCode::InvalidStatement => database::Error::InvalidStatement(message),
        SqlDatabaseErrorCode::InvalidParameter => database::Error::InvalidParameter(message),
        SqlDatabaseErrorCode::ConstraintViolation => database::Error::ConstraintViolation(message),
        SqlDatabaseErrorCode::TransactionState => database::Error::TransactionState(message),
        SqlDatabaseErrorCode::Unsupported => database::Error::Unsupported(message),
        SqlDatabaseErrorCode::Transient => database::Error::Transient(message),
        SqlDatabaseErrorCode::Internal => database::Error::Internal(message),
    }
}
