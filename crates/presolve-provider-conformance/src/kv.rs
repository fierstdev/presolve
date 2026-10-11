use presolve_provider_sdk::KeyValueProvider;

/// Asserts portable baseline key/value semantics against freshly-created providers.
///
/// `factory` is invoked separately for each semantic case so one failure cannot
/// depend on state left by another case.
pub fn assert_key_value_conformance<P, F>(factory: F)
where
    P: KeyValueProvider,
    F: Fn() -> P,
{
    missing_is_distinct_from_empty(&factory);
    set_replaces_complete_value(&factory);
    keys_use_exact_identity(&factory);
    delete_is_safe_and_reports_existence(&factory);
}

fn missing_is_distinct_from_empty<P, F>(factory: &F)
where
    P: KeyValueProvider,
    F: Fn() -> P,
{
    let provider = factory();
    let key = "presolve-conformance/missing-empty";

    assert_eq!(
        provider.get(key).expect("conformance get should succeed"),
        None,
        "KV conformance: a missing key must return None"
    );

    provider
        .set(key, "")
        .expect("conformance empty set should succeed");

    assert_eq!(
        provider.get(key).expect("conformance get should succeed"),
        Some(String::new()),
        "KV conformance: an empty value must remain distinct from a missing key"
    );
}

fn set_replaces_complete_value<P, F>(factory: &F)
where
    P: KeyValueProvider,
    F: Fn() -> P,
{
    let provider = factory();
    let key = "presolve-conformance/replacement";

    provider
        .set(key, "first-value")
        .expect("conformance initial set should succeed");
    provider
        .set(key, "second")
        .expect("conformance replacement set should succeed");

    assert_eq!(
        provider.get(key).expect("conformance get should succeed"),
        Some("second".to_owned()),
        "KV conformance: set must replace the complete previous value"
    );
}

fn keys_use_exact_identity<P, F>(factory: &F)
where
    P: KeyValueProvider,
    F: Fn() -> P,
{
    let provider = factory();

    provider
        .set("presolve-conformance/Case-Key", "value")
        .expect("conformance set should succeed");

    assert_eq!(
        provider
            .get("presolve-conformance/case-key")
            .expect("conformance get should succeed"),
        None,
        "KV conformance: keys must use exact identity"
    );

    assert_eq!(
        provider
            .get("presolve-conformance/Case-Key")
            .expect("conformance get should succeed"),
        Some("value".to_owned()),
        "KV conformance: the exact key must remain addressable"
    );
}

fn delete_is_safe_and_reports_existence<P, F>(factory: &F)
where
    P: KeyValueProvider,
    F: Fn() -> P,
{
    let provider = factory();
    let key = "presolve-conformance/delete";

    provider
        .set(key, "value")
        .expect("conformance set should succeed");

    assert!(
        provider
            .delete(key)
            .expect("conformance delete should succeed"),
        "KV conformance: deleting an existing key must report true"
    );

    assert!(
        !provider
            .delete(key)
            .expect("conformance repeated delete should succeed"),
        "KV conformance: deleting a missing key must report false"
    );

    assert_eq!(
        provider.get(key).expect("conformance get should succeed"),
        None,
        "KV conformance: deleted values must not remain visible"
    );
}
