#!/usr/bin/env bash
set -euo pipefail

script_dir=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)
repo_root=$(cd -- "$script_dir/../.." && pwd)

ruleset=${1:-}
case "$ruleset" in
    standard|neworigins) ;;
    *)
        printf 'Usage: %s <standard|neworigins>\n' "${BASH_SOURCE[0]}" >&2
        exit 2
        ;;
esac

reference_root=
temporary_output=$(mktemp)
temporary_trace=$(mktemp)
cleanup() {
    rm -f -- "$temporary_output" "$temporary_trace"
    if [[ -n "$reference_root" ]]; then
        rm -rf -- "$reference_root"
    fi
}
trap cleanup EXIT

reference_root=$("$script_dir/build.sh")
reference_binary="$reference_root/Atlantis/$ruleset/$ruleset"
ATLANTIS_RNG_TRACE="$temporary_trace" "$reference_binary" new-game-fixture \
    >"$temporary_output"

fixture_dir="$repo_root/fixtures/new-game"
mkdir -p "$fixture_dir"
if ! grep -q '^NPC_FIXTURE ' "$temporary_output"; then
    printf 'Reference produced no new-game fixture records\n' >&2
    exit 1
fi
sed -n 's/^NPC_FIXTURE //p' "$temporary_output" >"$fixture_dir/$ruleset.txt"
cp "$temporary_trace" "$fixture_dir/$ruleset.rng"

manifest="$repo_root/fixtures/SHA256SUMS"
temporary_manifest=$(mktemp)
trap 'cleanup; rm -f -- "$temporary_manifest"' EXIT
grep -v "^.*  fixtures/new-game/$ruleset\\." "$manifest" >"$temporary_manifest"
(
    cd "$repo_root"
    shasum -a 256 "fixtures/new-game/$ruleset.txt" "fixtures/new-game/$ruleset.rng"
) >>"$temporary_manifest"
awk '{
    if ($2 ~ /^fixtures\/standard\//) group = "a"
    else if ($2 ~ /^fixtures\/neworigins\//) group = "b"
    else group = "c"
    print group, $0
}' "$temporary_manifest" |
    LC_ALL=C sort -k1,1 -k3,3 |
    cut -d ' ' -f2- >"$manifest"
printf 'Recorded %s new-game fixture and RNG trace\n' "$ruleset"
