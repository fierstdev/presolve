#!/usr/bin/env bash
set -euo pipefail

repository_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repository_root"

cargo build -p presolve-compiler --target wasm32-unknown-unknown --release --features wasm
mkdir -p packages/compiler-wasm/dist
wasm-bindgen --target web --out-dir packages/compiler-wasm/dist --out-name presolve_compiler_wasm target/wasm32-unknown-unknown/release/presolve_compiler.wasm
