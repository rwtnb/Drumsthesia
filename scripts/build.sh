#!/usr/bin/env

RUSTFLAGS=--cfg=web_sys_unstable_apis cargo build --target wasm32-unknown-unknown
cargo install -f wasm-bindgen-cli && wasm-bindgen --out-dir generated --web target/wasm32-unknown-unknown/debug/drumsthesia.wasm
