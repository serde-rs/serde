#!/bin/bash
set -ev
if [ "${CLIPPY}" = "true" ]; then
    (cd serde && travis-cargo clippy -- --features unstable-testing)
    (cd serde_codegen && travis-cargo clippy -- --features unstable-testing)
    (cd serde_derive && travis-cargo clippy -- --features unstable-testing)
    (cd test_suite && travis-cargo clippy -- --features unstable-testing)
    (cd test_suite/deps && travis-cargo clippy -- --features unstable-testing)
    (cd test_suite/no_std && travis-cargo clippy -- --features unstable-testing)
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