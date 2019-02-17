#!/bin/bash
#
# Copyright 2015 The Rust Project Developers. See the COPYRIGHT
# file at the top-level directory of this distribution and at
# http://rust-lang.org/COPYRIGHT.
#
# Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
# http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
# <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
# option. This file may not be copied, modified, or distributed
# except according to those terms.

# A script to update the references for particular tests. The idea is
# that you do a run, which will generate files in the build directory
# containing the (normalized) actual output of the compiler. This
# script will then copy that output and replace the "expected output"
# files. You can then commit the changes.
#
# If you find yourself manually editing a foo.stderr file, you're
# doing it wrong.

cd "$(dirname "${BASH_SOURCE[0]}")"
BUILD_DIR="../../../target/ui"

for testcase in */*.rs; do
    STDERR_NAME="${testcase/%.rs/.stderr}"
    STDOUT_NAME="${testcase/%.rs/.stdout}"
    if [ -f "$BUILD_DIR/$STDOUT_NAME" ] && \
           ! (diff "$BUILD_DIR/$STDOUT_NAME" "$STDOUT_NAME" >& /dev/null); then
        echo "updating $STDOUT_NAME"
        cp "$BUILD_DIR/$STDOUT_NAME" "$STDOUT_NAME"
    fi
    if [ -f "$BUILD_DIR/$STDERR_NAME" ] && \
           ! (diff "$BUILD_DIR/$STDERR_NAME" "$STDERR_NAME" >& /dev/null); then
        echo "updating $STDERR_NAME"
        cp "$BUILD_DIR/$STDERR_NAME" "$STDERR_NAME"
    fi
done
