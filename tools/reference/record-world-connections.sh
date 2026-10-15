#!/usr/bin/env bash
set -euo pipefail

script_dir=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)
repo_root=$(cd -- "$script_dir/../.." && pwd)

reference_root=
temporary_output=$(mktemp)
temporary_records=$(mktemp)
temporary_manifest=$(mktemp)
run_root=$(mktemp -d "${TMPDIR:-/tmp}/atlantis-world-connections.XXXXXX")
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
    run_dir="$run_root/$ruleset"
    mkdir "$run_dir"
    if [[ "$ruleset" == neworigins ]]; then
        (
            cd "$run_dir"
            printf '1\n64\n64\n' |
                ATLANTIS_SEED=1 ATLANTIS_WORLD_CONNECTIONS=1 \
                    "$reference_binary" new >"$temporary_output" 2>&1
        )
    else
        (
            cd "$run_dir"
            ATLANTIS_SEED=1 ATLANTIS_WORLD_CONNECTIONS=1 \
                "$reference_binary" new </dev/null >"$temporary_output" 2>&1
        )
    fi

    summary_count=$(grep -c '^WORLDGEN_CONNECTION_SUMMARY ' "$temporary_output" || true)
    if [[ "$summary_count" != 1 ]]; then
        printf 'Expected one connection summary for %s\n' "$ruleset" >&2
        cat "$temporary_output" >&2
        exit 1
    fi
    grep '^WORLDGEN_CONNECTION_' "$temporary_output" |
        sed "s/^/$ruleset\t/" >>"$temporary_records"
    rm -rf -- "$run_dir"
done

fixture="$repo_root/fixtures/world-generation/classic-connections.json"
python3 - "$temporary_records" "$fixture" <<'PY'
import json
import sys

records_path, fixture_path = sys.argv[1:]
rulesets = {"standard": [], "neworigins": []}
with open(records_path, encoding="utf-8") as records:
    for line in records:
        ruleset, record = line.rstrip("\n").split("\t", 1)
        if ruleset not in rulesets or not record.startswith("WORLDGEN_CONNECTION_"):
            raise SystemExit(f"unexpected connection record: {line.rstrip()}")
        rulesets[ruleset].append(record)

for ruleset, rows in rulesets.items():
    summaries = [row for row in rows if row.startswith("WORLDGEN_CONNECTION_SUMMARY ")]
    region_rows = [row for row in rows if row.startswith("WORLDGEN_CONNECTION_REGION ")]
    if len(summaries) != 1:
        raise SystemExit(f"missing or duplicate connection summary for {ruleset}")
    region_count = int(summaries[0].split()[1])
    if len(region_rows) != region_count:
        raise SystemExit(
            f"region record count mismatch for {ruleset}: {len(region_rows)} != {region_count}"
        )

fixture = {
    "format": "atlantis-classic-world-connections-v1",
    "seed": 1,
    "width": 64,
    "height": 64,
    "rulesets": {name: {"records": rows} for name, rows in rulesets.items()},
}
with open(fixture_path, "w", encoding="utf-8", newline="\n") as output:
    json.dump(fixture, output, indent=2)
    output.write("\n")
PY

manifest="$repo_root/fixtures/SHA256SUMS"
grep -Ev '  fixtures/world-generation/classic-connections\.json$' "$manifest" \
    >"$temporary_manifest"
(
    cd "$repo_root"
    shasum -a 256 fixtures/world-generation/classic-connections.json
) >>"$temporary_manifest"
awk '{
    if ($2 ~ /^fixtures\/standard\//) group = "a"
    else if ($2 ~ /^fixtures\/neworigins\//) group = "b"
    else group = "c"
    print group, $0
}' "$temporary_manifest" |
    LC_ALL=C sort -k1,1 -k3,3 |
    cut -d ' ' -f2- >"$manifest"
printf 'Recorded immutable connection fixtures from the C++ reference\n'
