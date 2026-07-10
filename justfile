# Install `just` later if desired. These commands are intentionally simple.

check:
    cargo fmt --all --check
    cargo clippy --workspace --all-targets -- -D warnings
    cargo test --workspace

e2e:
    cargo test -p ezc_cli --test runtime_browser -- --nocapture

e2e-headed:
    cargo test -p ezc_cli --test runtime_browser -- --nocapture

explain-counter:
    cargo run -p ezc_cli -- explain fixtures/0001-source-summary/input/Counter.tsx

explain-counter-json:
    cargo run -p ezc_cli -- explain fixtures/0001-source-summary/input/Counter.tsx --format json
