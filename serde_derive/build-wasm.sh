#!/usr/bin/env bash

cargo +nightly build --release --target wasm32-unknown-unknown
wasm-strip target/wasm32-unknown-unknown/release/serde_derive.wasm
wasm-opt -Oz -o target/wasm32-unknown-unknown/release/serde_derive-min.wasm target/wasm32-unknown-unknown/release/serde_derive.wasm
cp target/wasm32-unknown-unknown/release/serde_derive-min.wasm ../wa-serde-derive/src/serde_derive.wasm
