#!/usr/bin/env bash
# test.sh — run the cargo test gate, then drop the debug tree it built.
#
# Usage: bash dstack-cli/test.sh [cargo test args...]
#
# The debug artifacts of one `cargo test` run weigh several hundred megabytes and nothing else
# reads them (install.sh and parity/run.sh build --release). So the tree is cleaned after every
# run, and the next run rebuilds it. The exit status is cargo test's, whatever the clean did.
set -u
CLI_DIR="$(cd "$(dirname "${BASH_SOURCE[0]:-$0}")" && pwd -P)"
cargo test --manifest-path "$CLI_DIR/Cargo.toml" "$@"
status=$?
cargo clean --manifest-path "$CLI_DIR/Cargo.toml" --profile dev
exit "$status"
