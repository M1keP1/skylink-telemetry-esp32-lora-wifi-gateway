#!/bin/bash
set -e

echo "=== Running Cargo Fmt (Formatter) ==="
cargo fmt -- --check

echo -e "\n=== Running Cargo Clippy (Linter) ==="
cargo clippy --all-targets --all-features -- -D warnings

echo -e "\n=== Running Tests ==="
cargo test

echo -e "\n Code Quality Check Passed!"
