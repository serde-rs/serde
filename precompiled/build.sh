#!/usr/bin/env bash

cd -- "$(dirname -- "${BASH_SOURCE[0]}")" &>/dev/null
set -e -x

# TODO: Sanitize host filesystem paths. https://github.com/rust-lang/cargo/issues/12137

cargo +nightly build \
    --manifest-path bin/Cargo.toml \
    --bin serde_derive \
    --profile precompiled \
    -Z unstable-options \
    -Z build-std=std,panic_abort \
    -Z build-std-features=panic_immediate_abort \
    --target x86_64-unknown-linux-musl \
    --out-dir serde_derive

rm -f serde_derive/serde_derive-x86_64-unknown-linux-gnu
mv serde_derive/serde_derive{,-x86_64-unknown-linux-gnu}
