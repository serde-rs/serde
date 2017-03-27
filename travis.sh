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
    cargo clippy --features unstable-testing -- -Dclippy

    cd "$DIR/serde_derive"
    cargo clippy --features unstable-testing -- -Dclippy

    cd "$DIR/test_suite"
    cargo clippy --features unstable -- -Dclippy

    cd "$DIR/test_suite/no_std"
    cargo clippy -- -Dclippy
else
    CHANNEL=nightly
    cargo clean
    cd "$DIR/serde"
    channel build
    channel build --no-default-features
    channel build --no-default-features --features alloc
    channel build --no-default-features --features collections
    channel test --features unstable-testing
    cd "$DIR/test_suite/deps"
    channel build
    cd "$DIR/test_suite"
    channel test --features unstable
    cd "$DIR/test_suite/no_std"
    channel build

    CHANNEL=beta
    cargo clean
    cd "$DIR/serde"
    channel build
    cd "$DIR/test_suite"
    channel test

    CHANNEL=stable
    cargo clean
    cd "$DIR/serde"
    channel build
    channel build --no-default-features

    CHANNEL=1.13.0
    cargo clean
    cd "$DIR/serde"
    channel build
    channel build --no-default-features
fi
