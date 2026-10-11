use std::{collections::BTreeSet, io::Cursor};

use presolve_provider_sdk::{ObjectStoreErrorCode, ObjectStoreProvider};

/// Asserts portable baseline object-storage semantics against freshly-created providers.
///
/// Optional features such as range reads, conditional writes, and user metadata
/// are intentionally outside this baseline suite.
pub fn assert_object_store_conformance<P, F>(factory: F)
where
    P: ObjectStoreProvider,
    F: Fn() -> P,
{
    opaque_bytes_round_trip_after_write(&factory);
    zero_length_is_distinct_from_missing(&factory);
    put_replaces_complete_object(&factory);
    head_reports_portable_metadata(&factory);
    delete_is_idempotent_and_reports_existence(&factory);
    list_uses_exact_prefix_semantics(&factory);
    short_body_is_rejected_atomically(&factory);
    long_body_is_rejected_atomically(&factory);
}

fn opaque_bytes_round_trip_after_write<P, F>(factory: &F)
where
    P: ObjectStoreProvider,
    F: Fn() -> P,
{
    let provider = factory();
    let key = "presolve-conformance/binary";
    let expected = [0_u8, 1, 2, 127, 128, 254, 255];

    put(&provider, key, &expected);
    let actual = get_body(&provider, key).expect("object should exist after put");

    assert_eq!(
        actual, expected,
        "object conformance: content must remain opaque bytes and be visible after write"
    );
}

fn zero_length_is_distinct_from_missing<P, F>(factory: &F)
where
    P: ObjectStoreProvider,
    F: Fn() -> P,
{
    let provider = factory();
    let key = "presolve-conformance/empty";

    match provider.get(key).expect("conformance get should succeed") {
        None => {}
        Some(_) => panic!("object conformance: a missing object must return None"),
    }

    put(&provider, key, &[]);

    let read = provider
        .get(key)
        .expect("conformance get should succeed")
        .expect("zero-length object must exist");

    assert_eq!(read.info().size(), 0);
}

fn put_replaces_complete_object<P, F>(factory: &F)
where
    P: ObjectStoreProvider,
    F: Fn() -> P,
{
    let provider = factory();
    let key = "presolve-conformance/replacement";

    put(&provider, key, b"first-object-body");
    put(&provider, key, b"new");

    assert_eq!(
        get_body(&provider, key).expect("replacement object should exist"),
        b"new",
        "object conformance: put must replace the complete previous object"
    );
}

fn head_reports_portable_metadata<P, F>(factory: &F)
where
    P: ObjectStoreProvider,
    F: Fn() -> P,
{
    let provider = factory();
    let key = "presolve-conformance/head";
    let body = b"metadata";

    put(&provider, key, body);

    let info = provider
        .head(key)
        .expect("conformance head should succeed")
        .expect("object should exist");

    let expected_size = u64::try_from(body.len()).expect("body length should fit u64");
    assert_eq!(info.key(), key);
    assert_eq!(info.size(), expected_size);
}

fn delete_is_idempotent_and_reports_existence<P, F>(factory: &F)
where
    P: ObjectStoreProvider,
    F: Fn() -> P,
{
    let provider = factory();
    let key = "presolve-conformance/delete";

    put(&provider, key, b"value");

    assert!(
        provider
            .delete(key)
            .expect("conformance delete should succeed"),
        "object conformance: deleting an existing object must report true"
    );

    assert!(
        !provider
            .delete(key)
            .expect("conformance repeated delete should succeed"),
        "object conformance: deleting a missing object must report false"
    );

    match provider.get(key).expect("conformance get should succeed") {
        None => {}
        Some(_) => panic!("object conformance: deleted objects must not remain visible"),
    }
}

fn list_uses_exact_prefix_semantics<P, F>(factory: &F)
where
    P: ObjectStoreProvider,
    F: Fn() -> P,
{
    let provider = factory();

    put(&provider, "presolve-conformance/a/one", b"1");
    put(&provider, "presolve-conformance/a/two", b"22");
    put(&provider, "presolve-conformance/b/three", b"333");

    let listed = provider
        .list("presolve-conformance/a/")
        .expect("conformance list should succeed");

    let keys = listed
        .iter()
        .map(|info| info.key().to_owned())
        .collect::<BTreeSet<_>>();

    assert_eq!(
        keys,
        BTreeSet::from([
            "presolve-conformance/a/one".to_owned(),
            "presolve-conformance/a/two".to_owned(),
        ]),
        "object conformance: list must match the exact requested key prefix"
    );

    assert!(
        listed.iter().any(|info| info.size() == 1) && listed.iter().any(|info| info.size() == 2),
        "object conformance: list metadata must report portable object sizes"
    );
}

fn short_body_is_rejected_atomically<P, F>(factory: &F)
where
    P: ObjectStoreProvider,
    F: Fn() -> P,
{
    let provider = factory();
    let key = "presolve-conformance/short-body";

    put(&provider, key, b"stable");

    let mut body = Cursor::new(b"new".to_vec());
    let error = provider
        .put(key, 4, &mut body)
        .expect_err("short body must be rejected");

    assert_eq!(
        error.code(),
        ObjectStoreErrorCode::InvalidBody,
        "object conformance: short body must report InvalidBody"
    );

    assert_eq!(
        get_body(&provider, key).expect("previous object should remain"),
        b"stable",
        "object conformance: malformed put must not partially replace an object"
    );
}

fn long_body_is_rejected_atomically<P, F>(factory: &F)
where
    P: ObjectStoreProvider,
    F: Fn() -> P,
{
    let provider = factory();
    let key = "presolve-conformance/long-body";

    put(&provider, key, b"stable");

    let mut body = Cursor::new(b"toolong".to_vec());
    let error = provider
        .put(key, 3, &mut body)
        .expect_err("long body must be rejected");

    assert_eq!(
        error.code(),
        ObjectStoreErrorCode::InvalidBody,
        "object conformance: long body must report InvalidBody"
    );

    assert_eq!(
        get_body(&provider, key).expect("previous object should remain"),
        b"stable",
        "object conformance: malformed put must not partially replace an object"
    );
}

fn put<P: ObjectStoreProvider>(provider: &P, key: &str, body: &[u8]) {
    let size = u64::try_from(body.len()).expect("conformance body length should fit u64");
    let mut reader = Cursor::new(body);

    let info = provider
        .put(key, size, &mut reader)
        .expect("conformance put should succeed");

    assert_eq!(info.key(), key);
    assert_eq!(info.size(), size);
}

fn get_body<P: ObjectStoreProvider>(provider: &P, key: &str) -> Option<Vec<u8>> {
    let mut read = provider.get(key).expect("conformance get should succeed")?;

    let mut body = Vec::new();
    read.body()
        .read_to_end(&mut body)
        .expect("conformance object body read should succeed");

    Some(body)
}
