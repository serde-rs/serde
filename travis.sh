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
else
    CHANNEL=nightly
    cd "$DIR"
    cargo clean
    cd "$DIR/serde"
    channel build
    channel build --no-default-features
    channel build --no-default-features --features alloc
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
    cd "$DIR/serde_state"
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
fi
