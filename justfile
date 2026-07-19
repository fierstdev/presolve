# Repository task recipes. These commands are intentionally simple.

check:
    cargo fmt --all --check
    cargo clippy --workspace --all-targets -- -D warnings
    cargo test --workspace

e2e:
    cargo test -p presolve-cli --test runtime_browser -- --nocapture --test-threads=1

e2e-headed:
    cargo test -p presolve-cli --test runtime_browser -- --nocapture --test-threads=1

explain-counter:
    cargo run -p presolve-cli -- explain fixtures/0001-source-summary/input/Counter.tsx

explain-counter-json:
    cargo run -p presolve-cli -- explain fixtures/0001-source-summary/input/Counter.tsx --format json
