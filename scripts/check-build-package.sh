#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
PRESOLVE="$ROOT/target/debug/presolve"
TMP="$(mktemp -d "${TMPDIR:-/tmp}/presolve-build-checkpoint.XXXXXX")"

cleanup() {
  rm -rf "$TMP"
}
trap cleanup EXIT

echo "== build/package Rust tests =="
cargo test --manifest-path "$ROOT/Cargo.toml" \
  -p presolve-bundle \
  -p presolve-cli

echo "== build/package Clippy =="
cargo clippy --manifest-path "$ROOT/Cargo.toml" \
  -p presolve-bundle \
  -p presolve-cli \
  --all-targets \
  -- \
  -D warnings

echo "== TypeScript SDK tests =="
pnpm --dir "$ROOT" --filter @presolve/sdk check

echo "== build Presolve CLI =="
cargo build --manifest-path "$ROOT/Cargo.toml" -p presolve-cli

component_checkpoint() {
  local app="$TMP/component"
  mkdir -p "$app"

  cat > "$app/presolve.toml" <<'EOF'
contract_version = "0.1"

[application]
name = "component-repro"
version = "1.0.0"
EOF

  printf 'presolve deterministic component bytes\n' > "$app/app.wasm"

  (
    cd "$app"
    "$PRESOLVE" build \
      --component app.wasm \
      --output first.presolved
    "$PRESOLVE" build \
      --component app.wasm \
      --output second.presolved
  )

  cmp "$app/first.presolved" "$app/second.presolved"
}

multi_component_checkpoint() {
  local app="$TMP/multi"
  mkdir -p "$app"

  cat > "$app/presolve.toml" <<'EOF'
contract_version = "0.1"

[application]
name = "multi-repro"
version = "1.0.0"

[[workloads]]
name = "api"
kind = "component"

[[workloads]]
name = "worker"
kind = "component"
EOF

  printf 'api component bytes\n' > "$app/api.wasm"
  printf 'worker component bytes\n' > "$app/worker.wasm"

  (
    cd "$app"
    "$PRESOLVE" build \
      --component-for api=api.wasm \
      --component-for worker=worker.wasm \
      --output first.presolved

    "$PRESOLVE" build \
      --component-for worker=worker.wasm \
      --component-for api=api.wasm \
      --output second.presolved
  )

  cmp "$app/first.presolved" "$app/second.presolved"
}

oci_checkpoint() {
  local app="$TMP/oci"
  local digest="sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
  mkdir -p "$app"

  cat > "$app/presolve.toml" <<'EOF'
contract_version = "0.1"

[application]
name = "oci-repro"
version = "1.0.0"

[[workloads]]
name = "api"
kind = "container"
EOF

  (
    cd "$app"
    "$PRESOLVE" build \
      --oci-image-for "api=manifest,$digest,1024" \
      --output first.presolved

    "$PRESOLVE" build \
      --oci-image-for "api=manifest,$digest,1024" \
      --output second.presolved
  )

  cmp "$app/first.presolved" "$app/second.presolved"
}

mixed_checkpoint() {
  local app="$TMP/mixed"
  local digest="sha256:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"
  mkdir -p "$app"

  cat > "$app/presolve.toml" <<'EOF'
contract_version = "0.1"

[application]
name = "mixed-repro"
version = "1.0.0"

[[workloads]]
name = "api"
kind = "component"

[[workloads]]
name = "worker"
kind = "container"
EOF

  printf 'mixed api component bytes\n' > "$app/api.wasm"

  (
    cd "$app"
    "$PRESOLVE" build \
      --component-for api=api.wasm \
      --oci-image-for "worker=index,$digest,2048" \
      --output first.presolved

    "$PRESOLVE" build \
      --oci-image-for "worker=index,$digest,2048" \
      --component-for api=api.wasm \
      --output second.presolved
  )

  cmp "$app/first.presolved" "$app/second.presolved"
}

echo "== explicit Component reproducibility =="
component_checkpoint

echo "== multi-Component mapping-order reproducibility =="
multi_component_checkpoint

echo "== OCI descriptor reproducibility =="
oci_checkpoint

echo "== mixed workload mapping-order reproducibility =="
mixed_checkpoint

echo "== TypeScript source-build boundary =="
"$ROOT/scripts/smoke-typescript-component.sh"

echo "Chunk 19 build/package checkpoint passed"
