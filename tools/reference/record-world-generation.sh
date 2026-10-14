#!/usr/bin/env bash
set -euo pipefail

script_dir=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)
repo_root=$(cd -- "$script_dir/../.." && pwd)

reference_root=
temporary_output=$(mktemp)
temporary_records=$(mktemp)
temporary_nexus_records=$(mktemp)
temporary_surface_records=$(mktemp)
temporary_manifest=$(mktemp)
run_root=$(mktemp -d "${TMPDIR:-/tmp}/atlantis-world-stats.XXXXXX")
cleanup() {
    rm -f -- "$temporary_output" "$temporary_records" "$temporary_nexus_records" \
        "$temporary_surface_records" "$temporary_manifest"
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
        if [[ "$seed" == 1 ]]; then
            final_setup_count=$(grep -c '^WORLDGEN_NEXUS_FINAL_SETUP ' "$temporary_output" || true)
            if [[ "$final_setup_count" != 1 ]]; then
                printf 'Expected one Nexus FinalSetup record for %s seed %s\n' "$ruleset" "$seed" >&2
                cat "$temporary_output" >&2
                exit 1
            fi
            final_setup_line=$(grep '^WORLDGEN_NEXUS_FINAL_SETUP ' "$temporary_output")
            printf '%s\t%s\t%s\n' "$ruleset" "$seed" "$final_setup_line" >>"$temporary_nexus_records"

            surface_setup_count=$(grep -c '^WORLDGEN_SURFACE_FINAL_SETUP ' "$temporary_output" || true)
            if [[ "$surface_setup_count" != 2048 ]]; then
                printf 'Expected 2048 surface FinalSetup records for %s seed %s, got %s\n' \
                    "$ruleset" "$seed" "$surface_setup_count" >&2
                cat "$temporary_output" >&2
                exit 1
            fi
            grep '^WORLDGEN_SURFACE_FINAL_SETUP ' "$temporary_output" |
                sed "s/^/$ruleset\t/" >>"$temporary_surface_records"
        fi
        rm -rf -- "$run_dir"
    done
done

fixture_dir="$repo_root/fixtures/world-generation"
mkdir -p "$fixture_dir"
python3 - "$temporary_records" "$temporary_nexus_records" \
    "$temporary_surface_records" \
    "$fixture_dir/classic-terrain-counts.json" \
    "$fixture_dir/classic-nexus-final-setup.json" \
    "$fixture_dir/classic-surface-final-setup.json" <<'PY'
import json
import sys

(
    records_path,
    nexus_records_path,
    surface_records_path,
    fixture_path,
    nexus_fixture_path,
    surface_fixture_path,
) = sys.argv[1:]
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

nexus_rulesets = {}
with open(nexus_records_path, encoding="utf-8") as records:
    for line in records:
        ruleset, seed_text, stats_text = line.rstrip("\n").split("\t")
        values = stats_text.split()
        if ruleset not in ("standard", "neworigins") or int(seed_text) != 1:
            raise SystemExit(f"unexpected Nexus FinalSetup record: {line.rstrip()}")
        if ruleset in nexus_rulesets or len(values) != 5 or values[0] != "WORLDGEN_NEXUS_FINAL_SETUP":
            raise SystemExit(f"invalid Nexus FinalSetup record: {line.rstrip()}")
        terrain, race, population, habitat = map(int, values[1:])
        nexus_rulesets[ruleset] = {
            "terrain": terrain,
            "race": race,
            "population": population,
            "habitat": habitat,
        }

if set(nexus_rulesets) != {"standard", "neworigins"}:
    raise SystemExit(f"missing Nexus FinalSetup records: {sorted(nexus_rulesets)}")

nexus_fixture = {
    "format": "atlantis-classic-nexus-final-setup-v1",
    "seed": 1,
    "rulesets": nexus_rulesets,
}
with open(nexus_fixture_path, "w", encoding="utf-8", newline="\n") as output:
    json.dump(nexus_fixture, output, indent=2)
    output.write("\n")

surface_fields = [
    "x",
    "y",
    "terrain",
    "race",
    "population",
    "base_population",
    "wages",
    "max_wages",
    "wealth",
    "habitat",
    "development",
    "max_development",
    "has_town",
    "town_population",
    "town_habitat",
    "town_development",
    "town_activity",
]
surface_regions = {"standard": [], "neworigins": []}
with open(surface_records_path, encoding="utf-8") as records:
    for line in records:
        ruleset, stats_text = line.rstrip("\n").split("\t", 1)
        values = stats_text.split()
        if (
            ruleset not in surface_regions
            or len(values) != len(surface_fields) + 1
            or values[0] != "WORLDGEN_SURFACE_FINAL_SETUP"
        ):
            raise SystemExit(f"invalid surface FinalSetup record: {line.rstrip()}")
        surface_regions[ruleset].append([int(value) for value in values[1:]])

expected_coordinates = [
    [x, y] for x in range(64) for y in range(64) if (x + y) % 2 == 0
]
for ruleset, rows in surface_regions.items():
    if len(rows) != 2048 or [row[:2] for row in rows] != expected_coordinates:
        raise SystemExit(f"invalid surface FinalSetup records for {ruleset}: {len(rows)} rows")

surface_fixture = {
    "format": "atlantis-classic-surface-final-setup-v1",
    "seed": 1,
    "width": 64,
    "height": 64,
    "fields": surface_fields,
    "rulesets": {
        ruleset: {"regions": [" ".join(map(str, row)) for row in rows]}
        for ruleset, rows in surface_regions.items()
    },
}
with open(surface_fixture_path, "w", encoding="utf-8", newline="\n") as output:
    json.dump(surface_fixture, output, indent=2)
    output.write("\n")
PY

manifest="$repo_root/fixtures/SHA256SUMS"
grep -Ev '  fixtures/world-generation/classic-(terrain-counts|nexus-final-setup|surface-final-setup)\.json$' "$manifest" \
    >"$temporary_manifest"
(
    cd "$repo_root"
    shasum -a 256 \
        fixtures/world-generation/classic-terrain-counts.json \
        fixtures/world-generation/classic-nexus-final-setup.json \
        fixtures/world-generation/classic-surface-final-setup.json
) >>"$temporary_manifest"
awk '{
    if ($2 ~ /^fixtures\/standard\//) group = "a"
    else if ($2 ~ /^fixtures\/neworigins\//) group = "b"
    else group = "c"
    print group, $0
}' "$temporary_manifest" |
    LC_ALL=C sort -k1,1 -k3,3 |
    cut -d ' ' -f2- >"$manifest"
printf 'Recorded classic terrain-count and FinalSetup fixtures from the C++ reference\n'
