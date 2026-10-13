#!/usr/bin/env bash
set -euo pipefail

usage() {
    printf 'Usage: %s <turn-dir> <engine>\n' "${0##*/}" >&2
    exit 2
}

if [[ $# -ne 2 ]]; then
    usage
fi

turn_dir=$1
engine=$2

if [[ ! -d "$turn_dir" ]]; then
    printf 'Turn directory does not exist: %s\n' "$turn_dir" >&2
    exit 2
fi
turn_dir=$(cd "$turn_dir" && pwd -P)

if [[ "$engine" == */* ]]; then
    engine_dir=$(cd "$(dirname "$engine")" && pwd -P) || {
        printf 'Engine directory does not exist: %s\n' "$(dirname "$engine")" >&2
        exit 2
    }
    engine="$engine_dir/$(basename "$engine")"
else
    engine=$(command -v "$engine") || {
        printf 'Engine executable not found: %s\n' "$2" >&2
        exit 2
    }
fi

if [[ ! -x "$engine" ]]; then
    printf 'Engine is not executable: %s\n' "$engine" >&2
    exit 2
fi

shopt -s nullglob
orders=("$turn_dir"/orders.*)
if [[ ! -f "$turn_dir/game.in" || ! -f "$turn_dir/players.in" || ${#orders[@]} -eq 0 ]]; then
    printf 'Turn directory must contain game.in, players.in, and at least one orders.* file: %s\n' "$turn_dir" >&2
    exit 2
fi

if [[ ! -f "$turn_dir/game.out" || ! -f "$turn_dir/players.out" || ! -f "$turn_dir/engine-output.txt" ]]; then
    printf 'Turn directory is missing a required expected output: %s\n' "$turn_dir" >&2
    exit 2
fi

work_dir=$(mktemp -d "${TMPDIR:-/tmp}/atlantis-replay.XXXXXX")
work_dir=$(cd "$work_dir" && pwd -P)
trap 'rm -rf "$work_dir"' EXIT

cp "$turn_dir/game.in" "$work_dir/game.in"
cp "$turn_dir/players.in" "$work_dir/players.in"
for order_file in "${orders[@]}"; do
    cp "$order_file" "$work_dir/${order_file##*/}"
done

if (cd "$work_dir" && "$engine" run >engine-output.txt 2>&1); then
    :
else
    engine_status=$?
    printf 'Engine exited with status %d for %s\n' "$engine_status" "$turn_dir" >&2
    if [[ -f "$work_dir/engine-output.txt" ]]; then
        cat "$work_dir/engine-output.txt" >&2
    fi
    exit "$engine_status"
fi

expected_outputs=(
    "$turn_dir/game.out"
    "$turn_dir/players.out"
    "$turn_dir"/report.*
    "$turn_dir"/template.*
    "$turn_dir"/times.*
    "$turn_dir/engine-output.txt"
)

for expected in "${expected_outputs[@]}"; do
    filename=${expected##*/}
    actual="$work_dir/$filename"

    if [[ ! -f "$expected" ]]; then
        printf 'Expected output is not a file: %s\n' "$expected" >&2
        exit 2
    fi
    if [[ ! -f "$actual" ]]; then
        printf 'First output difference in %s: engine output is missing\n' "$filename" >&2
        exit 1
    fi

    if diff_output=$(diff -U 3 -L "$filename (expected)" -L "$filename (actual)" "$expected" "$actual"); then
        continue
    else
        diff_status=$?
        if [[ $diff_status -ne 1 ]]; then
            printf 'Could not compare output file %s (diff exited %d)\n' "$filename" "$diff_status" >&2
            exit "$diff_status"
        fi
        printf 'First output difference in %s\n' "$filename" >&2
        printf '%s\n' "$diff_output" | awk '
            /^@@ / {
                if (hunk++) {
                    exit
                }
            }
            { print }
        ' >&2
        exit 1
    fi
done

actual_outputs=(
    "$work_dir/game.out"
    "$work_dir/players.out"
    "$work_dir"/report.*
    "$work_dir"/template.*
    "$work_dir"/times.*
    "$work_dir/engine-output.txt"
)

for actual in "${actual_outputs[@]}"; do
    [[ -f "$actual" ]] || continue
    filename=${actual##*/}
    if [[ ! -f "$turn_dir/$filename" ]]; then
        printf 'First output difference in %s: engine produced an unrecorded file\n' "$filename" >&2
        exit 1
    fi
done

printf 'Identical: %s\n' "$turn_dir"
