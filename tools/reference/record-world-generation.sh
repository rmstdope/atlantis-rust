#!/usr/bin/env bash
set -euo pipefail

script_dir=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)
repo_root=$(cd -- "$script_dir/../.." && pwd)

reference_root=
temporary_output=$(mktemp)
temporary_records=$(mktemp)
temporary_manifest=$(mktemp)
run_root=$(mktemp -d "${TMPDIR:-/tmp}/atlantis-world-stats.XXXXXX")
cleanup() {
    rm -f -- "$temporary_output" "$temporary_records" "$temporary_manifest"
    rm -rf -- "$run_root"
    if [[ -n "$reference_root" ]]; then
        rm -rf -- "$reference_root"
    fi
}
trap cleanup EXIT

reference_root=$("$script_dir/build.sh")
for ruleset in standard neworigins; do
    reference_binary="$reference_root/Atlantis/$ruleset/$ruleset"
    for seed in $(seq 1 50); do
        run_dir="$run_root/$ruleset-$seed"
        mkdir "$run_dir"
        if [[ "$ruleset" == neworigins ]]; then
            (
                cd "$run_dir"
                printf '1\n64\n64\n' |
                    ATLANTIS_SEED="$seed" ATLANTIS_WORLD_STATS=1 \
                        "$reference_binary" new >"$temporary_output" 2>&1
            )
        else
            (
                cd "$run_dir"
                ATLANTIS_SEED="$seed" ATLANTIS_WORLD_STATS=1 \
                    "$reference_binary" new </dev/null >"$temporary_output" 2>&1
            )
        fi

        stats_count=$(grep -c '^WORLDGEN_TERRAIN_COUNTS ' "$temporary_output" || true)
        if [[ "$stats_count" != 1 ]]; then
            printf 'Expected one terrain-count record for %s seed %s\n' "$ruleset" "$seed" >&2
            cat "$temporary_output" >&2
            exit 1
        fi
        stats_line=$(grep '^WORLDGEN_TERRAIN_COUNTS ' "$temporary_output")
        printf '%s\t%s\t%s\n' "$ruleset" "$seed" "$stats_line" >>"$temporary_records"
        rm -rf -- "$run_dir"
    done
done

fixture_dir="$repo_root/fixtures/world-generation"
mkdir -p "$fixture_dir"
python3 - "$temporary_records" "$fixture_dir/classic-terrain-counts.json" <<'PY'
import json
import sys

records_path, fixture_path = sys.argv[1:]
seeds = list(range(1, 51))
rulesets = {"standard": [], "neworigins": []}

with open(records_path, encoding="utf-8") as records:
    for line in records:
        ruleset, seed_text, stats_text = line.rstrip("\n").split("\t")
        seed = int(seed_text)
        counts = [int(value) for value in stats_text.split()[1:]]
        if ruleset not in rulesets or seed != len(rulesets[ruleset]) + 1:
            raise SystemExit(f"unexpected reference record: {line.rstrip()}")
        if len(counts) != 61 or sum(counts) != 2048:
            raise SystemExit(f"invalid terrain counts for {ruleset} seed {seed}")
        rulesets[ruleset].append(counts)

for name, rows in rulesets.items():
    if len(rows) != len(seeds):
        raise SystemExit(f"missing {name} reference records: got {len(rows)}")

fixture = {
    "format": "atlantis-classic-terrain-counts-v1",
    "width": 64,
    "height": 64,
    "seeds": seeds,
    "rulesets": rulesets,
}
with open(fixture_path, "w", encoding="utf-8", newline="\n") as output:
    json.dump(fixture, output, indent=2)
    output.write("\n")
PY

manifest="$repo_root/fixtures/SHA256SUMS"
grep -v '  fixtures/world-generation/classic-terrain-counts.json$' "$manifest" \
    >"$temporary_manifest"
(
    cd "$repo_root"
    shasum -a 256 fixtures/world-generation/classic-terrain-counts.json
) >>"$temporary_manifest"
awk '{
    if ($2 ~ /^fixtures\/standard\//) group = "a"
    else if ($2 ~ /^fixtures\/neworigins\//) group = "b"
    else group = "c"
    print group, $0
}' "$temporary_manifest" |
    LC_ALL=C sort -k1,1 -k3,3 |
    cut -d ' ' -f2- >"$manifest"
printf 'Recorded classic terrain-count fixture from the C++ reference\n'
