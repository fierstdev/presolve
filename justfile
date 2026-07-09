# Install `just` later if desired. These commands are intentionally simple.

check:
    cargo fmt --all --check
    cargo clippy --workspace --all-targets -- -D warnings
    cargo test --workspace

explain-counter:
    cargo run -p ezc_cli -- explain fixtures/0001-source-summary/input/Counter.tsx

explain-counter-json:
    cargo run -p ezc_cli -- explain fixtures/0001-source-summary/input/Counter.tsx --format json
