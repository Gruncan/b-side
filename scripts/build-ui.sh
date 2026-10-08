#!/bin/sh
set -eu
cd "$(dirname "$0")/.."
cargo build --manifest-path crates/deck-ui/Cargo.toml --target wasm32-unknown-unknown --release
mkdir -p ui/pkg
wasm-bindgen --target web --out-dir ui/pkg \
  target/wasm32-unknown-unknown/release/deck_ui.wasm
