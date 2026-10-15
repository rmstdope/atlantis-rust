#!/usr/bin/env bash
set -euo pipefail

script_dir=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)
repo_root=$(cd -- "$script_dir/../.." && pwd)

reference_root=
temporary_output=$(mktemp)
temporary_records=$(mktemp)
temporary_manifest=$(mktemp)
run_root=$(mktemp -d "${TMPDIR:-/tmp}/atlantis-natural-world-stats.XXXXXX")
cleanup() {
    rm -f -- "$temporary_output" "$temporary_records" "$temporary_manifest"
    rm -rf -- "$run_root"
    if [[ -n "$reference_root" ]]; then
        rm -rf -- "$reference_root"
    fi
}
trap cleanup EXIT

reference_root=$(ATLANTIS_REFERENCE_OPTIMIZE=1 "$script_dir/build.sh")
reference_binary="$reference_root/Atlantis/neworigins/neworigins"
for seed in $(seq 1 50); do
    run_dir="$run_root/neworigins-$seed"
    mkdir "$run_dir"
    printf 'Recording NewOrigins natural world %s/50\n' "$seed" >&2
    if ! (
        cd "$run_dir"
        printf '2\n64\n64\n' |
            ATLANTIS_SEED="$seed" \
                ATLANTIS_NATURAL_SEED="$seed" \
                ATLANTIS_NATURAL_STATS=1 \
                "$reference_binary" new >"$temporary_output" 2>&1
    ); then
        printf 'C++ natural world generation failed for seed %s\n' "$seed" >&2
        cat "$temporary_output" >&2
        exit 1
    fi

    stats_count=$(grep -c '^WORLDGEN_NATURAL_STATS ' "$temporary_output" || true)
    if [[ "$stats_count" != 1 ]]; then
        printf 'Expected one natural-statistics record for seed %s\n' "$seed" >&2
        cat "$temporary_output" >&2
        exit 1
    fi
    stats_line=$(grep '^WORLDGEN_NATURAL_STATS ' "$temporary_output")
    printf '%s\t%s\n' "$seed" "$stats_line" >>"$temporary_records"
    rm -rf -- "$run_dir"
done

fixture_dir="$repo_root/fixtures/world-generation"
mkdir -p "$fixture_dir"
python3 - "$temporary_records" "$fixture_dir/neworigins-natural-stats.json" <<'PY'
import json
import sys

records_path, fixture_path = sys.argv[1:]
biomes = [
    "tundra",
    "mountains",
    "swamp",
    "forest",
    "plains",
    "jungle",
    "desert",
    "water",
]
worlds = []
with open(records_path, encoding="utf-8") as records:
    for line in records:
        seed_text, stats_text = line.rstrip("\n").split("\t")
        values = stats_text.split()
        seed = int(seed_text)
        if len(values) != 12 or values[0] != "WORLDGEN_NATURAL_STATS":
            raise SystemExit(f"invalid reference record: {line.rstrip()}")
        counts = [int(value) for value in values[1:9]]
        river_count, river_region_count, settlement_count = map(int, values[9:])
        if seed != len(worlds) + 1 or sum(counts) != 2048:
            raise SystemExit(f"invalid biome counts for seed {seed}: {counts}")
        if min(river_count, river_region_count, settlement_count) < 0:
            raise SystemExit(f"negative world-generation statistic for seed {seed}")
        worlds.append(
            {
                "seed": seed,
                "biome_counts": counts,
                "river_count": river_count,
                "river_region_count": river_region_count,
                "settlement_count": settlement_count,
            }
        )

if len(worlds) != 50:
    raise SystemExit(f"expected 50 reference worlds, got {len(worlds)}")

fixture = {
    "format": "atlantis-neworigins-natural-world-stats-v1",
    "width": 64,
    "height": 64,
    "biomes": biomes,
    "worlds": worlds,
}
with open(fixture_path, "w", encoding="utf-8", newline="\n") as output:
    json.dump(fixture, output, indent=2)
    output.write("\n")
PY

manifest="$repo_root/fixtures/SHA256SUMS"
grep -v '  fixtures/world-generation/neworigins-natural-stats\.json$' "$manifest" \
    >"$temporary_manifest"
(
    cd "$repo_root"
    shasum -a 256 fixtures/world-generation/neworigins-natural-stats.json
) >>"$temporary_manifest"
awk '{
    if ($2 ~ /^fixtures\/standard\//) group = "a"
    else if ($2 ~ /^fixtures\/neworigins\//) group = "b"
    else group = "c"
    print group, $0
}' "$temporary_manifest" |
    LC_ALL=C sort -k1,1 -k3,3 |
    cut -d ' ' -f2- >"$manifest"
printf 'Recorded NewOrigins natural-world statistics from the C++ reference\n'
