#!/usr/bin/env bash
set -euo pipefail

script_dir=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)
repo_root=$(cd -- "$script_dir/../.." && pwd)

reference_root=$("$script_dir/build.sh")
temp_dir=$(mktemp -d "${TMPDIR:-/tmp}/atlantis-region-setup.XXXXXX")
trap 'rm -rf "$temp_dir"' EXIT
fixture_dir="$repo_root/fixtures/economy/region-setup"

for ruleset in standard neworigins; do
    "$reference_root/Atlantis/$ruleset/$ruleset" dump-region-setup \
        > "$temp_dir/$ruleset.jsonl"
done

mkdir -p "$fixture_dir"
for ruleset in standard neworigins; do
    mv "$temp_dir/$ruleset.jsonl" "$fixture_dir/$ruleset.jsonl"
done

(
    cd "$repo_root"
    awk '!index($0, "  fixtures/economy/region-setup/")' fixtures/SHA256SUMS
    find fixtures/economy/region-setup -type f -print | LC_ALL=C sort |
        while IFS= read -r fixture; do
            shasum -a 256 "$fixture"
        done
) > "$temp_dir/SHA256SUMS"
mv "$temp_dir/SHA256SUMS" "$repo_root/fixtures/SHA256SUMS"

printf 'Recorded region setup fixtures at %s\n' "$fixture_dir" >&2
printf 'Reference build retained at %s\n' "$reference_root" >&2
