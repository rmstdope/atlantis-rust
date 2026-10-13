#!/usr/bin/env bash
set -euo pipefail

usage() {
    printf 'Usage: %s [--fast]\n' "$0" >&2
}

fast=false
if [[ $# -gt 1 ]]; then
    usage
    exit 2
fi

if [[ $# -eq 1 ]]; then
    if [[ "$1" != "--fast" ]]; then
        usage
        exit 2
    fi
    fast=true
fi

printf '%s\n' '==> cargo fmt --all -- --check'
cargo fmt --all -- --check

printf '%s\n' '==> cargo clippy --workspace --all-targets --locked -- -D warnings'
cargo clippy --workspace --all-targets --locked -- -D warnings

printf '%s\n' '==> cargo test --workspace --lib --bins --locked'
cargo test --workspace --lib --bins --locked

if [[ "$fast" == true ]]; then
    exit 0
fi

printf '%s\n' '==> cargo test -p atlantis-engine --locked replay'
cargo test -p atlantis-engine --locked replay

printf '%s\n' '==> checking atlantis-engine for filesystem, stdin, and clock APIs'
if grep -R -n -E 'std::fs|std::io::stdin|SystemTime|Instant' crates/atlantis-engine/src; then
    printf '%s\n' 'Forbidden filesystem, stdin, or clock API found in atlantis-engine source' >&2
    exit 1
else
    status=$?
    if [[ "$status" -ne 1 ]]; then
        printf 'Could not inspect atlantis-engine source (grep exited %s)\n' "$status" >&2
        exit "$status"
    fi
fi
