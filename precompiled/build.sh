#!/bin/bash

cd -- "$(dirname -- "${BASH_SOURCE[0]}")" &>/dev/null
set -e -x

# TODO: Sanitize host filesystem paths. https://github.com/rust-lang/cargo/issues/12137

build () {
    local target=$1
    local output_target=$target
    local -a opts=()
    local nightly=
    local ext=

    case $target in
        x86_64-unknown-linux-musl)
            output_target=x86_64-unknown-linux-gnu
            nightly=+nightly
            opts=(
                -Z unstable-options
                -Z build-std=std,panic_abort
                -Z build-std-features=panic_immediate_abort
            )
            ;;
        wasm32-wasi)
            ext=.wasm
            ;;
    esac

    cargo $nightly build \
        --manifest-path bin/Cargo.toml \
        --bin serde_derive \
        --profile precompiled \
        --target $target \
        ${opts[@]}

    rm -f serde_derive/serde_derive-$output_target$ext
    mv target/$target/precompiled/serde_derive$ext serde_derive/serde_derive-$output_target$ext
}

build x86_64-unknown-linux-musl
build wasm32-wasi

#upx --best --lzma serde_derive/serde_derive-x86_64-unknown-linux-gnu
