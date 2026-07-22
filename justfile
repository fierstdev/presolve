# Repository task recipes. These commands are intentionally simple.

check:
    ./scripts/verify-l12c4-language-service.sh
    ./scripts/verify-l12c3-wasm-binding.sh
    ./scripts/verify-l12c2-rust-query-projection.sh
    ./scripts/verify-l12c-wasm-binding-contract.sh
    ./scripts/verify-l12c-language-service-binding-audit.sh
    ./scripts/verify-l12c-query-snapshot-product.sh
    ./scripts/verify-l12b-query-snapshot-amendment.sh
    ./scripts/verify-l12a-editor-capability-audit.sh
    ./scripts/verify-l11g-artifact-graph-command.sh
    ./scripts/verify-l11g-profile-command.sh
    ./scripts/verify-l11g-trace-command.sh
    ./scripts/verify-l11f-tooling-products.sh
    ./scripts/verify-l11e-artifact-graph-contract.sh
    ./scripts/verify-l11d-trace-cost-contract.sh
    ./scripts/verify-l11c-tooling-commands.sh
    ./scripts/verify-l11b-tooling-product-readers.sh
    ./scripts/verify-l11a-tooling-capability-contract.sh
    ./scripts/verify-l10-schema-contract.sh
    cargo fmt --all --check
    cargo clippy --workspace --all-targets -- -D warnings
    cargo test --workspace
    ./scripts/verify-repository-layout.sh
    ./scripts/verify-public-identity.sh
    ./scripts/verify-l3-platform-contracts.sh
    ./scripts/verify-l4-service-contracts.sh
    ./scripts/verify-l5-incremental-contracts.sh
    ./scripts/verify-l6-persistent-cache-contracts.sh
    ./scripts/verify-l7-workspace-contracts.sh
    ./scripts/verify-l8-watch-contracts.sh
    ./scripts/verify-l9a1-configuration-codec-contracts.sh
    ./scripts/verify-l9b-command-framework-contracts.sh
    ./scripts/verify-l9c-compilation-adapter-contracts.sh
    ./scripts/verify-l9d-build-check-contracts.sh
    ./scripts/verify-l9e-cache-clean-contracts.sh
    ./scripts/verify-l9f-workspace-contracts.sh
    ./scripts/verify-l9g-command-dispatch-contracts.sh
    ./scripts/verify-l9-final-contracts.sh

repository-layout:
    ./scripts/verify-repository-layout.sh

phase-l-specifications:
    ./scripts/verify-phase-l-specifications.sh

l3-platform-contracts:
    ./scripts/verify-l3-platform-contracts.sh

l4-service-contracts:
    ./scripts/verify-l4-service-contracts.sh

l5-incremental-contracts:
    ./scripts/verify-l5-incremental-contracts.sh

l6-persistent-cache-contracts:
    ./scripts/verify-l6-persistent-cache-contracts.sh

l8-watch-contracts:
    ./scripts/verify-l8-watch-contracts.sh

l9a1-configuration-codec-contracts:
    ./scripts/verify-l9a1-configuration-codec-contracts.sh

l9b-command-framework-contracts:
    ./scripts/verify-l9b-command-framework-contracts.sh

l9c-compilation-adapter-contracts:
    ./scripts/verify-l9c-compilation-adapter-contracts.sh

l9d-build-check-contracts:
    ./scripts/verify-l9d-build-check-contracts.sh

l9e-cache-clean-contracts:
    ./scripts/verify-l9e-cache-clean-contracts.sh

l9f-workspace-contracts:
    ./scripts/verify-l9f-workspace-contracts.sh

l9g-command-dispatch-contracts:
    ./scripts/verify-l9g-command-dispatch-contracts.sh

e2e:
    cargo test -p presolve-cli --test runtime_browser -- --nocapture --test-threads=1

e2e-headed:
    cargo test -p presolve-cli --test runtime_browser -- --nocapture --test-threads=1

explain-counter:
    cargo run -p presolve-cli -- explain fixtures/0001-source-summary/input/Counter.tsx

explain-counter-json:
    cargo run -p presolve-cli -- explain fixtures/0001-source-summary/input/Counter.tsx --format json
