#!/bin/bash

set -e

DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

channel() {
    if [ -n "${TRAVIS}" ]; then
        if [ "${TRAVIS_RUST_VERSION}" = "${CHANNEL}" ]; then
            pwd
            (set -x; cargo "$@")
        fi
    elif [ -n "${APPVEYOR}" ]; then
        if [ "${APPVEYOR_RUST_CHANNEL}" = "${CHANNEL}" ]; then
            pwd
            (set -x; cargo "$@")
        fi
    else
        pwd
        (set -x; cargo "+${CHANNEL}" "$@")
    fi
}

if [ -n "${CLIPPY}" ]; then
    # cached installation will not work on a later nightly
    if [ -n "${TRAVIS}" ] && ! cargo install clippy --debug --force; then
        echo "COULD NOT COMPILE CLIPPY, IGNORING CLIPPY TESTS"
        exit
    fi

    cd "$DIR/serde"
    cargo clippy --features 'rc unstable' -- -Dclippy

    cd "$DIR/serde_derive"
    cargo clippy -- -Dclippy

    cd "$DIR/serde_test"
    cargo clippy -- -Dclippy

    cd "$DIR/test_suite"
    cargo clippy --features unstable -- -Dclippy

    cd "$DIR/test_suite/no_std"
    cargo clippy -- -Dclippy
elif [ -n "${EMSCRIPTEN}" ]; then
    CARGO_WEB_RELEASE=$(curl -L -s -H 'Accept: application/json' https://github.com/koute/cargo-web/releases/latest)
    CARGO_WEB_VERSION=$(echo "${CARGO_WEB_RELEASE}" | sed -e 's/.*"tag_name":"\([^"]*\)".*/\1/')
    CARGO_WEB_URL="https://github.com/koute/cargo-web/releases/download/${CARGO_WEB_VERSION}/cargo-web-x86_64-unknown-linux-gnu.gz"

    mkdir -p ~/.cargo/bin
    echo "Downloading cargo-web from: ${CARGO_WEB_URL}"
    curl -L "${CARGO_WEB_URL}" | gzip -d > ~/.cargo/bin/cargo-web
    chmod +x ~/.cargo/bin/cargo-web

    cd "$DIR/test_suite"
    cargo web test --target=asmjs-unknown-emscripten --nodejs
    cargo web test --target=wasm32-unknown-emscripten --nodejs
else
    CHANNEL=nightly
    cd "$DIR"
    cargo clean
    cd "$DIR/serde"
    channel build
    channel build --no-default-features
    channel build --no-default-features --features alloc
    channel build --no-default-features --features 'rc alloc'
    channel test --features 'rc unstable'
    cd "$DIR/test_suite/deps"
    channel build
    cd "$DIR/test_suite"
    channel test --features unstable
    channel build --tests --features proc-macro2/nightly
    if [ -z "${APPVEYOR}" ]; then
        cd "$DIR/test_suite/no_std"
        channel build
    fi

    CHANNEL=beta
    cd "$DIR"
    cargo clean
    cd "$DIR/serde"
    channel build --features rc
    cd "$DIR/test_suite"
    channel test

    CHANNEL=stable
    cd "$DIR"
    cargo clean
    cd "$DIR/serde"
    channel build --features rc
    channel build --no-default-features
    cd "$DIR/serde_test"
    channel build
    channel test

    CHANNEL=1.13.0
    cd "$DIR"
    cargo clean
    cd "$DIR/serde"
    channel build --features rc
    channel build --no-default-features
    cd "$DIR/serde_test"
    channel build

    CHANNEL=1.15.0
    cd "$DIR"
    cargo clean
    cd "$DIR/serde_derive"
    channel build

    for CHANNEL in 1.20.0 1.21.0 1.25.0 1.26.0; do
        cd "$DIR"
        cargo clean
        cd "$DIR/serde"
        channel build --no-default-features
        channel build
    done
fi
