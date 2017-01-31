#!/bin/bash
set -ev
if [ "${CLIPPY}" = "true" ]; then
    if cargo install clippy; then
        (cd serde && cargo clippy -- -Dclippy --features unstable-testing)
        (cd serde_derive && cargo clippy -- -Dclippy --features unstable-testing)
        (cd test_suite && cargo clippy -- -Dclippy --features unstable-testing)
        (cd test_suite/deps && cargo clippy -- -Dclippy)
        (cd test_suite/no_std && cargo clippy -- -Dclippy)
    else
        echo "could not compile clippy, ignoring clippy tests"
    fi
else
    (cd serde && travis-cargo build)
    (cd serde && travis-cargo --only beta test)
    (cd serde && travis-cargo --only nightly test -- --features unstable-testing)
    (cd serde && travis-cargo build -- --no-default-features)
    (cd serde && travis-cargo --only nightly build -- --no-default-features --features alloc)
    (cd serde && travis-cargo --only nightly build -- --no-default-features --features collections)
    (cd test_suite && travis-cargo --only beta test)
    (cd test_suite/deps && travis-cargo --only nightly build)
    (cd test_suite travis-cargo --only nightly test -- --features unstable-testing)
    (cd test_suite/no_std && travis-cargo --only nightly build)
fi