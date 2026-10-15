use presolve_provider_sdk::{
    SqlColumn, SqlDatabaseProvider, SqlRow, SqlTransactionOptions, SqlValue,
};

/// Provider-specific SQL text used to exercise the portable relational
/// capability semantics against an already-provisioned conformance table.
///
/// The harness intentionally does not prescribe DDL or placeholder syntax.
/// Providers may therefore supply statements in the dialect they advertise
/// while the assertions remain focused on Presolve's provider-neutral
/// execution, value, result, and transaction semantics.
#[derive(Debug, Clone, Copy)]
pub struct SqlConformanceFixture {
    /// Inserts `(id, label, optional_text, bytes_value)` into the conformance table.
    pub insert: &'static str,

    /// Selects exactly `id, label, optional_text, bytes_value` for one bound `id`.
    pub select_by_id: &'static str,

    /// Updates `label` for one bound `id`.
    pub update_label: &'static str,

    /// Deletes the row for one bound `id`.
    pub delete_by_id: &'static str,
}

/// Exercises the mandatory `presolve:sql/database@0.1.0` provider semantics.
///
/// The caller must provision the schema expected by [`SqlConformanceFixture`]
/// before invoking this function. Schema lifecycle is deliberately outside the
/// SQL capability baseline.
///
/// This harness panics on semantic violations and is intended to be called from
/// provider test suites.
pub fn assert_sql_database_conformance(
    provider: &dyn SqlDatabaseProvider,
    fixture: SqlConformanceFixture,
) {
    const BASE_ID: i64 = 9_120_000;

    for offset in 0..8 {
        cleanup(provider, fixture, BASE_ID + offset);
    }

    assert_empty_result_preserves_columns(provider, fixture, BASE_ID + 7);
    assert_scalar_round_trip(provider, fixture, BASE_ID);
    assert_bound_values_remain_data(provider, fixture, BASE_ID + 1);
    assert_null_is_distinct_from_empty(provider, fixture, BASE_ID + 2, BASE_ID + 3);
    assert_explicit_rollback(provider, fixture, BASE_ID + 4);
    assert_ordered_transaction_commit(provider, fixture, BASE_ID + 5);
    assert_dropped_transaction_rolls_back(provider, fixture, BASE_ID + 6);

    for offset in 0..8 {
        cleanup(provider, fixture, BASE_ID + offset);
    }
}

fn assert_empty_result_preserves_columns(
    provider: &dyn SqlDatabaseProvider,
    fixture: SqlConformanceFixture,
    id: i64,
) {
    let (columns, rows) = query_rows(provider, fixture.select_by_id, &[SqlValue::Integer(id)]);
    assert_columns(&columns);
    assert!(rows.is_empty(), "absent row query should remain empty");
}

fn assert_scalar_round_trip(
    provider: &dyn SqlDatabaseProvider,
    fixture: SqlConformanceFixture,
    id: i64,
) {
    let inserted = provider
        .execute(
            fixture.insert,
            &[
                SqlValue::Integer(id),
                SqlValue::Text("baseline".to_owned()),
                SqlValue::Text("present".to_owned()),
                SqlValue::Bytes(vec![0, 1, 2, 255]),
            ],
        )
        .expect("baseline SQL insert should succeed");
    assert_eq!(inserted.affected_rows(), 1);

    let (columns, rows) = query_rows(provider, fixture.select_by_id, &[SqlValue::Integer(id)]);
    assert_columns(&columns);
    assert_eq!(rows.len(), 1);
    assert_eq!(
        rows[0].values(),
        [
            SqlValue::Integer(id),
            SqlValue::Text("baseline".to_owned()),
            SqlValue::Text("present".to_owned()),
            SqlValue::Bytes(vec![0, 1, 2, 255]),
        ]
    );
}

fn assert_bound_values_remain_data(
    provider: &dyn SqlDatabaseProvider,
    fixture: SqlConformanceFixture,
    id: i64,
) {
    let hostile = "value'); delete from presolve_conformance; --";

    provider
        .execute(
            fixture.insert,
            &[
                SqlValue::Integer(id),
                SqlValue::Text(hostile.to_owned()),
                SqlValue::Null,
                SqlValue::Null,
            ],
        )
        .expect("bound hostile-looking text should remain data");

    let (_, rows) = query_rows(provider, fixture.select_by_id, &[SqlValue::Integer(id)]);
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].values()[1], SqlValue::Text(hostile.to_owned()));
}

fn assert_null_is_distinct_from_empty(
    provider: &dyn SqlDatabaseProvider,
    fixture: SqlConformanceFixture,
    null_id: i64,
    empty_id: i64,
) {
    provider
        .execute(
            fixture.insert,
            &[
                SqlValue::Integer(null_id),
                SqlValue::Text("null".to_owned()),
                SqlValue::Null,
                SqlValue::Null,
            ],
        )
        .expect("null insert should succeed");

    provider
        .execute(
            fixture.insert,
            &[
                SqlValue::Integer(empty_id),
                SqlValue::Text("empty".to_owned()),
                SqlValue::Text(String::new()),
                SqlValue::Bytes(Vec::new()),
            ],
        )
        .expect("empty scalar insert should succeed");

    let (_, null_rows) = query_rows(
        provider,
        fixture.select_by_id,
        &[SqlValue::Integer(null_id)],
    );
    let (_, empty_rows) = query_rows(
        provider,
        fixture.select_by_id,
        &[SqlValue::Integer(empty_id)],
    );

    assert_eq!(null_rows[0].values()[2], SqlValue::Null);
    assert_eq!(null_rows[0].values()[3], SqlValue::Null);
    assert_eq!(empty_rows[0].values()[2], SqlValue::Text(String::new()));
    assert_eq!(empty_rows[0].values()[3], SqlValue::Bytes(Vec::new()));
}

fn assert_explicit_rollback(
    provider: &dyn SqlDatabaseProvider,
    fixture: SqlConformanceFixture,
    id: i64,
) {
    let mut transaction = provider
        .begin(SqlTransactionOptions::default())
        .expect("transaction should begin");

    transaction
        .execute(
            fixture.insert,
            &[
                SqlValue::Integer(id),
                SqlValue::Text("rollback".to_owned()),
                SqlValue::Null,
                SqlValue::Null,
            ],
        )
        .expect("transactional insert should succeed");

    transaction.rollback().expect("rollback should succeed");

    let (_, rows) = query_rows(provider, fixture.select_by_id, &[SqlValue::Integer(id)]);
    assert!(rows.is_empty(), "rolled-back row must not be visible");
}

fn assert_ordered_transaction_commit(
    provider: &dyn SqlDatabaseProvider,
    fixture: SqlConformanceFixture,
    id: i64,
) {
    let mut transaction = provider
        .begin(SqlTransactionOptions::default())
        .expect("transaction should begin");

    transaction
        .execute(
            fixture.insert,
            &[
                SqlValue::Integer(id),
                SqlValue::Text("first".to_owned()),
                SqlValue::Null,
                SqlValue::Null,
            ],
        )
        .expect("transactional insert should succeed");

    transaction
        .execute(
            fixture.update_label,
            &[SqlValue::Text("second".to_owned()), SqlValue::Integer(id)],
        )
        .expect("ordered transactional update should succeed");

    transaction.commit().expect("commit should succeed");

    let (_, rows) = query_rows(provider, fixture.select_by_id, &[SqlValue::Integer(id)]);
    assert_eq!(rows.len(), 1);
    assert_eq!(
        rows[0].values()[1],
        SqlValue::Text("second".to_owned()),
        "transaction statements must be observed in invocation order"
    );
}

fn assert_dropped_transaction_rolls_back(
    provider: &dyn SqlDatabaseProvider,
    fixture: SqlConformanceFixture,
    id: i64,
) {
    let mut transaction = provider
        .begin(SqlTransactionOptions::default())
        .expect("transaction should begin");

    transaction
        .execute(
            fixture.insert,
            &[
                SqlValue::Integer(id),
                SqlValue::Text("drop".to_owned()),
                SqlValue::Null,
                SqlValue::Null,
            ],
        )
        .expect("transactional insert should succeed");

    drop(transaction);

    let (_, rows) = query_rows(provider, fixture.select_by_id, &[SqlValue::Integer(id)]);
    assert!(
        rows.is_empty(),
        "dropping an uncommitted transaction must roll it back"
    );
}

fn cleanup(provider: &dyn SqlDatabaseProvider, fixture: SqlConformanceFixture, id: i64) {
    provider
        .execute(fixture.delete_by_id, &[SqlValue::Integer(id)])
        .expect("conformance cleanup should succeed");
}

fn query_rows(
    provider: &dyn SqlDatabaseProvider,
    statement: &str,
    parameters: &[SqlValue],
) -> (Vec<SqlColumn>, Vec<SqlRow>) {
    let result = provider
        .query(statement, parameters)
        .expect("conformance query should succeed");
    let (columns, rows) = result.into_parts();
    let rows = rows
        .collect::<Result<Vec<_>, _>>()
        .expect("conformance query rows should stream successfully");

    (columns, rows)
}

fn assert_columns(columns: &[SqlColumn]) {
    assert_eq!(columns.len(), 4);
    assert_eq!(columns[0].name(), "id");
    assert_eq!(columns[1].name(), "label");
    assert_eq!(columns[2].name(), "optional_text");
    assert_eq!(columns[3].name(), "bytes_value");
}
