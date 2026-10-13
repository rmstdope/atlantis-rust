#!/usr/bin/env bash
set -euo pipefail

if [[ "$#" -ne 2 ]]; then
	printf 'Usage: %s <standard|neworigins> <area>\n' "${0##*/}" >&2
	exit 2
fi

ruleset=$1
area=$2
case "$ruleset" in
	standard|neworigins) ;;
	*) printf 'Unsupported ruleset: %s\n' "$ruleset" >&2; exit 2 ;;
esac

if [[ ! "$area" =~ ^[A-Za-z0-9][A-Za-z0-9_-]*$ ]]; then
	printf 'Invalid area name: %s\n' "$area" >&2
	exit 2
fi

script_dir=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)
project_root=$(cd -- "$script_dir/../.." && pwd)
fixture_dir="$project_root/fixtures/$ruleset/$area"
if [[ ! -d "$fixture_dir" ]]; then
	printf 'No fixture area exists at %s; add a complete baseline turn first.\n' \
			"$fixture_dir" >&2
	exit 1
fi

shopt -s nullglob

is_complete_turn()
{
	local dir=$1
	local orders=("$dir"/orders.[0-9]*)
	local templates=("$dir"/template.[0-9]*)
	local reports=("$dir"/report.[0-9]*)

	[[ -f "$dir/game.in" && -f "$dir/game.out" &&
			-f "$dir/players.in" && -f "$dir/players.out" &&
			-f "$dir/engine-output.txt" ]] || return 1
	(( ${#orders[@]} > 0 && ${#templates[@]} > 0 && ${#reports[@]} > 0 )) ||
			return 1

	local path filename
	for path in "${orders[@]}" "${templates[@]}" "${reports[@]}"; do
		filename=${path##*/}
		[[ "$filename" =~ ^(orders|template|report)\.[0-9]+$ ]] || return 1
	done
}

remove_generated_outputs()
{
	local dir=$1
	local output
	local generated_outputs=("$dir/game.out" "$dir/players.out" \
			"$dir/engine-output.txt" "$dir"/report.* \
			"$dir"/template.* "$dir"/times.*)
	for output in "${generated_outputs[@]}"; do
		if [[ -e "$output" ]]; then
			rm -f -- "$output"
		fi
	done
}

turn_dirs=("$fixture_dir"/turn_[0-9]*)
latest_number=-1
latest_dir=
pending_dirs=()
for turn_dir in "${turn_dirs[@]}"; do
	[[ -e "$turn_dir" ]] || continue
	if [[ ! -d "$turn_dir" ]]; then
		printf 'Turn path is not a directory: %s\n' "$turn_dir" >&2
		exit 1
	fi

	turn_name=${turn_dir##*/}
	if [[ ! "$turn_name" =~ ^turn_(0|[1-9][0-9]*)$ ]]; then
		printf 'Invalid turn directory name: %s\n' "$turn_name" >&2
		exit 1
	fi
	turn_number=$((10#${BASH_REMATCH[1]}))

	if is_complete_turn "$turn_dir"; then
		if (( turn_number > latest_number )); then
			latest_number=$turn_number
			latest_dir=$turn_dir
		fi
	else
		pending_dirs+=("$turn_dir")
	fi
done

if (( latest_number < 0 )); then
	printf 'No complete fixture turn exists in %s.\n' "$fixture_dir" >&2
	exit 1
fi
if (( ${#pending_dirs[@]} > 1 )); then
	printf 'Multiple incomplete turns exist in %s; resolve them before recording.\n' \
			"$fixture_dir" >&2
	exit 1
fi

next_number=$((latest_number + 1))
target_dir="$fixture_dir/turn_$next_number"
if (( ${#pending_dirs[@]} == 1 )); then
	target_dir=${pending_dirs[0]}
	if [[ "$target_dir" != "$fixture_dir/turn_$next_number" ]]; then
		printf 'Incomplete turn %s is not after the latest complete turn %s.\n' \
				"${target_dir##*/}" "${latest_dir##*/}" >&2
		exit 1
	fi

	partial_outputs=("$target_dir/game.out" "$target_dir/players.out" \
			"$target_dir/engine-output.txt" "$target_dir"/report.* \
			"$target_dir"/template.* "$target_dir"/times.*)
	for output in "${partial_outputs[@]}"; do
		if [[ -e "$output" ]]; then
			printf 'Incomplete turn contains generated output: %s\n' "$output" >&2
			exit 1
		fi
	done
else
	if [[ -e "$target_dir" ]]; then
		printf 'Next turn path already exists: %s\n' "$target_dir" >&2
		exit 1
	fi

	templates=("$latest_dir"/template.[0-9]*)
	mkdir "$target_dir"
	cp "$latest_dir/game.out" "$target_dir/game.in"
	cp "$latest_dir/players.out" "$target_dir/players.in"
	for template in "${templates[@]}"; do
		filename=${template##*/}
		if [[ ! "$filename" =~ ^template\.([0-9]+)$ ]]; then
			printf 'Invalid template filename: %s\n' "$filename" >&2
			exit 1
		fi
		faction=${BASH_REMATCH[1]}
		cp "$template" "$target_dir/orders.$faction"
	done

	printf 'Staged %s from %s. Edit orders.* and rerun this command to record it.\n' \
			"${target_dir##*/}" "${latest_dir##*/}"
	exit 0
fi

if [[ ! -f "$target_dir/game.in" || ! -f "$target_dir/players.in" ]]; then
	printf 'Incomplete staged inputs in %s.\n' "$target_dir" >&2
	exit 1
fi
if ! cmp -s "$latest_dir/game.out" "$target_dir/game.in"; then
	printf 'Staged game.in differs from %s/game.out.\n' "${latest_dir##*/}" >&2
	exit 1
fi
if ! cmp -s "$latest_dir/players.out" "$target_dir/players.in"; then
	printf 'Staged players.in differs from %s/players.out.\n' "${latest_dir##*/}" >&2
	exit 1
fi

templates=("$latest_dir"/template.[0-9]*)
orders=("$target_dir"/orders.[0-9]*)
if (( ${#templates[@]} == 0 || ${#orders[@]} == 0 )); then
	printf 'Staged turn is missing template or orders files.\n' >&2
	exit 1
fi

orders_changed=0
for template in "${templates[@]}"; do
	filename=${template##*/}
	if [[ ! "$filename" =~ ^template\.([0-9]+)$ ]]; then
		printf 'Invalid template filename: %s\n' "$filename" >&2
		exit 1
	fi
	faction=${BASH_REMATCH[1]}
	order_file="$target_dir/orders.$faction"
	if [[ ! -f "$order_file" ]]; then
		printf 'Missing staged order file: %s\n' "$order_file" >&2
		exit 1
	fi
	if ! cmp -s "$template" "$order_file"; then
		orders_changed=1
	fi
done

for order_file in "${orders[@]}"; do
	filename=${order_file##*/}
	if [[ ! "$filename" =~ ^orders\.([0-9]+)$ ]]; then
		printf 'Invalid order filename: %s\n' "$filename" >&2
		exit 1
	fi
	faction=${BASH_REMATCH[1]}
	if [[ ! -f "$latest_dir/template.$faction" ]]; then
		printf 'No source template for %s.\n' "$filename" >&2
		exit 1
	fi
done

if (( orders_changed == 0 )); then
	printf 'No order changes in %s; refusing to record an unchanged turn.\n' \
			"$target_dir" >&2
	exit 1
fi

reference_build_root=$("$script_dir/build.sh")
printf 'Using reference binaries at %s\n' "$reference_build_root" >&2
reference_binary="$reference_build_root/Atlantis/$ruleset/$ruleset"
if [[ ! -x "$reference_binary" ]]; then
	printf 'Reference binary not found: %s\n' "$reference_binary" >&2
	exit 1
fi

if (cd "$target_dir" && "$reference_binary" run > engine-output.txt); then
	:
else
	status=$?
	printf 'Reference engine failed with status %s while recording %s.\n' \
			"$status" "${target_dir##*/}" >&2
	if [[ -f "$target_dir/engine-output.txt" ]]; then
		cat "$target_dir/engine-output.txt" >&2
	fi
	remove_generated_outputs "$target_dir"
	exit "$status"
fi

if ! is_complete_turn "$target_dir"; then
	printf 'Reference engine returned success but left an incomplete turn in %s.\n' \
			"$target_dir" >&2
	remove_generated_outputs "$target_dir"
	exit 1
fi

if ! replay_dir=$(mktemp -d "${TMPDIR:-/tmp}/atlantis-reference-replay.XXXXXX"); then
	printf 'Unable to create a temporary directory to verify %s.\n' \
			"${target_dir##*/}" >&2
	remove_generated_outputs "$target_dir"
	exit 1
fi
trap 'rm -rf -- "$replay_dir"' EXIT

replay_inputs=("$target_dir/game.in" "$target_dir/players.in" \
		"$target_dir"/orders.[0-9]*)
if ! cp "${replay_inputs[@]}" "$replay_dir/"; then
	printf 'Unable to stage inputs for replay verification of %s.\n' \
			"${target_dir##*/}" >&2
	remove_generated_outputs "$target_dir"
	exit 1
fi

if (cd "$replay_dir" && "$reference_binary" run > engine-output.txt); then
	:
else
	status=$?
	printf 'Reference replay failed with status %s while verifying %s.\n' \
			"$status" "${target_dir##*/}" >&2
	if [[ -f "$replay_dir/engine-output.txt" ]]; then
		cat "$replay_dir/engine-output.txt" >&2
	fi
	remove_generated_outputs "$target_dir"
	exit "$status"
fi

if ! is_complete_turn "$replay_dir"; then
	printf 'Reference replay returned success but left an incomplete turn while verifying %s.\n' \
			"${target_dir##*/}" >&2
	if [[ -f "$replay_dir/engine-output.txt" ]]; then
		cat "$replay_dir/engine-output.txt" >&2
	fi
	remove_generated_outputs "$target_dir"
	exit 1
fi

recorded_outputs=("$target_dir/game.out" "$target_dir/players.out" \
		"$target_dir/engine-output.txt" "$target_dir"/report.* \
		"$target_dir"/template.* "$target_dir"/times.*)
replayed_outputs=("$replay_dir/game.out" "$replay_dir/players.out" \
		"$replay_dir/engine-output.txt" "$replay_dir"/report.* \
		"$replay_dir"/template.* "$replay_dir"/times.*)
if (( ${#recorded_outputs[@]} != ${#replayed_outputs[@]} )); then
	printf 'Reference replay produced a different set of outputs for %s.\n' \
			"${target_dir##*/}" >&2
	remove_generated_outputs "$target_dir"
	exit 1
fi

for index in "${!recorded_outputs[@]}"; do
	recorded=${recorded_outputs[$index]}
	replayed=${replayed_outputs[$index]}
	if [[ "${recorded##*/}" != "${replayed##*/}" ]] ||
			! cmp -s "$recorded" "$replayed"; then
		printf 'Reference replay mismatch in %s while verifying %s.\n' \
				"${recorded##*/}" "${target_dir##*/}" >&2
		remove_generated_outputs "$target_dir"
		exit 1
	fi
done

printf 'Recorded %s/%s/%s from %s.\n' \
		"$ruleset" "$area" "${target_dir##*/}" "${latest_dir##*/}"
