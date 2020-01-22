#!/usr/bin/env bash

export RUSTFLAGS="${RUSTFLAGS:''}";

if [[ -z "$ALLOW_WARNINGS" ]]; then
    export RUSTFLAGS="$RUSTFLAGS -D warnings";
fi

if [[ "$TARGET" ]]; then
    rustup target add "$TARGET";
    TARGET_FLAG="--target=$TARGET"
fi

cargo build "$TARGET_FLAG"

if [[ -z "$SKIP_TESTS" ]]; then
    cargo test;
fi

if [[ "$DO_BENCHMARKS" ]]; then
    cargo test --benches;
fi