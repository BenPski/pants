#/usr/bin/env sh

cargo fmt --check && cargo clippy && cargo test
