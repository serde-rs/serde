#!/bin/bash

cd -- "$(dirname -- "${BASH_SOURCE[0]}")" &>/dev/null

# TODO: Sanitize host filesystem paths. https://github.com/rust-lang/cargo/issues/12137

cargo +nightly build \
    --manifest-path serde_derive/Cargo.toml \
    --bin serde_derive \
    --profile precompiled \
    -Z unstable-options \
    -Z build-std=std,panic_abort \
    -Z build-std-features=panic_immediate_abort \
    --target x86_64-unknown-linux-musl \
    --out-dir x86_64-unknown-linux-gnu

#upx --best --lzma x86_64-unknown-linux-gnu/serde_derive
