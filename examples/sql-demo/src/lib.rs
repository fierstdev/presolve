mod bindings {
    wit_bindgen::generate!({
        path: "../../wit/presolve-app",
        world: "sql-application",
        with: {
            "presolve:sql/database@0.1.0": generate,
        },
    });
}

use bindings::presolve::sql::database;

struct SqlComponent;

impl bindings::Guest for SqlComponent {
    fn run(input: String) -> String {
        const ID: i64 = 7_301;
        const PAYLOAD: &[u8] = &[0, 1, 2, 255];

        if let Err(error) = database::execute(
            "DELETE FROM presolve_demo WHERE id = $1",
            &[database::Value::Integer(ID)],
        ) {
            return format!("error:cleanup:{}", error_code(error));
        }

        let transaction = match database::begin(database::TransactionOptions {
            read_only: false,
            isolation: database::Isolation::ProviderDefault,
        }) {
            Ok(transaction) => transaction,
            Err(error) => return format!("error:begin:{}", error_code(error)),
        };

        if let Err(error) = transaction.execute(
            "INSERT INTO presolve_demo (id, label, optional_value) VALUES ($1, $2, $3)",
            &[
                database::Value::Integer(ID),
                database::Value::Text(input.clone()),
                database::Value::Bytes(PAYLOAD.to_vec()),
            ],
        ) {
            let _ = database::rollback(transaction);
            return format!("error:insert:{}", error_code(error));
        }

        let in_transaction = match transaction.query(
            "SELECT id, label, optional_value FROM presolve_demo WHERE id = $1 ORDER BY id",
            &[database::Value::Integer(ID)],
        ) {
            Ok(result) => result,
            Err(error) => {
                let _ = database::rollback(transaction);
                return format!("error:tx-query:{}", error_code(error));
            }
        };

        if let Err(message) = verify_result(&in_transaction, ID, &input, PAYLOAD) {
            drop(in_transaction);
            let _ = database::rollback(transaction);
            return message;
        }
        drop(in_transaction);

        if let Err(error) = database::commit(transaction) {
            return format!("error:commit:{}", error_code(error));
        }

        let committed = match database::query(
            "SELECT id, label, optional_value FROM presolve_demo WHERE id = $1 ORDER BY id",
            &[database::Value::Integer(ID)],
        ) {
            Ok(result) => result,
            Err(error) => return format!("error:committed-query:{}", error_code(error)),
        };
        if let Err(message) = verify_result(&committed, ID, &input, PAYLOAD) {
            return message;
        }
        drop(committed);

        let rollback_transaction = match database::begin(database::TransactionOptions {
            read_only: false,
            isolation: database::Isolation::ProviderDefault,
        }) {
            Ok(transaction) => transaction,
            Err(error) => return format!("error:rollback-begin:{}", error_code(error)),
        };

        if let Err(error) = rollback_transaction.execute(
            "UPDATE presolve_demo SET label = $1 WHERE id = $2",
            &[
                database::Value::Text("must-not-persist".to_owned()),
                database::Value::Integer(ID),
            ],
        ) {
            let _ = database::rollback(rollback_transaction);
            return format!("error:rollback-update:{}", error_code(error));
        }

        if let Err(error) = database::rollback(rollback_transaction) {
            return format!("error:rollback:{}", error_code(error));
        }

        let after_rollback = match database::query(
            "SELECT id, label, optional_value FROM presolve_demo WHERE id = $1 ORDER BY id",
            &[database::Value::Integer(ID)],
        ) {
            Ok(result) => result,
            Err(error) => return format!("error:rollback-query:{}", error_code(error)),
        };
        if let Err(message) = verify_result(&after_rollback, ID, &input, PAYLOAD) {
            return message;
        }
        drop(after_rollback);

        if let Err(error) = database::execute(
            "DELETE FROM presolve_demo WHERE id = $1",
            &[database::Value::Integer(ID)],
        ) {
            return format!("error:final-cleanup:{}", error_code(error));
        }

        format!("sql:{input}")
    }
}

fn verify_result(
    result: &database::QueryResult,
    id: i64,
    label: &str,
    payload: &[u8],
) -> Result<(), String> {
    let columns = result.columns();
    if columns.len() != 3
        || columns[0].name != "id"
        || columns[1].name != "label"
        || columns[2].name != "optional_value"
    {
        return Err("error:columns".to_owned());
    }

    let row = match result.next_row() {
        Ok(Some(row)) => row,
        Ok(None) => return Err("error:row-missing".to_owned()),
        Err(error) => return Err(format!("error:row:{}", error_code(error))),
    };

    let [actual_id, actual_label, actual_payload] = row.values.as_slice() else {
        return Err("error:row-values".to_owned());
    };
    match (actual_id, actual_label, actual_payload) {
        (
            database::Value::Integer(actual_id),
            database::Value::Text(actual_label),
            database::Value::Bytes(actual_payload),
        ) if *actual_id == id && actual_label == label && actual_payload.as_slice() == payload => {}
        _ => return Err("error:row-values".to_owned()),
    }

    match result.next_row() {
        Ok(None) => Ok(()),
        Ok(Some(_)) => Err("error:extra-row".to_owned()),
        Err(error) => Err(format!("error:extra-row:{}", error_code(error))),
    }
}

fn error_code(error: database::Error) -> &'static str {
    match error {
        database::Error::Unavailable(_) => "unavailable",
        database::Error::InvalidStatement(_) => "invalid-statement",
        database::Error::InvalidParameter(_) => "invalid-parameter",
        database::Error::ConstraintViolation(_) => "constraint-violation",
        database::Error::TransactionState(_) => "transaction-state",
        database::Error::Unsupported(_) => "unsupported",
        database::Error::Transient(_) => "transient",
        database::Error::Internal(_) => "internal",
    }
}

bindings::export!(SqlComponent with_types_in bindings);
