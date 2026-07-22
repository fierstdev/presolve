use wasm_bindgen::prelude::*;

/// The sole public L12-C browser binding. Product validation and query
/// projection remain inside the compiler's Rust authority.
#[wasm_bindgen]
pub fn query_snapshot_v1(product_bytes: &[u8], request_bytes: &[u8]) -> Vec<u8> {
    crate::language_service::query_snapshot_v1(product_bytes, request_bytes)
}
