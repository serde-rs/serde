#!/bin/bash

set -xeuo pipefail

DIR=$(cd "$(dirname "$0")" && pwd)

export RUSTC=${RUSTC:-$HOME/.local/bin/rustc}

cargo build || true

"$RUSTC" \
    "$DIR"/../serde_derive/src/lib.rs \
    --crate-name serde_derive \
    --crate-type rustc-macro \
    -C prefer-dynamic \
    -g \
    --out-dir "$DIR"/target/debug/deps \
    --emit=dep-info,link \
    -L dependency="$DIR"/target/debug/deps \
    --extern serde_codegen="$DIR"/target/debug/deps/libserde_codegen.rlib

"$RUSTC" \
    src/main.rs \
    --crate-name tmp_test \
    --crate-type bin \
    -g \
    --out-dir "$DIR"/target/debug \
    --emit=dep-info,link \
    -L dependency="$DIR"/target/debug/deps \
    --extern serde_json=$(echo "$DIR"/target/debug/deps/libserde_json-*.rlib) \
    --extern serde=$(echo "$DIR"/target/debug/deps/libserde-*.rlib) \
    --extern serde_derive="$DIR"/target/debug/deps/libserde_derive.so
