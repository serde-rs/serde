#!/bin/bash
set -ev
if [ "${CLIPPY}" = "true" ]; then
    (cd serde && travis-cargo clippy -- --features unstable-testing)
    (cd serde_codegen && travis-cargo clippy -- --features unstable-testing)
    (cd serde_derive && travis-cargo clippy -- --features unstable-testing)
    (cd test_suite && travis-cargo clippy -- --features unstable-testing)
    (cd test_suite/deps && travis-cargo clippy -- --features unstable-testing)
    (cd test_suite/no_std && travis-cargo clippy -- --features unstable-testing)
fi