#!/usr/bin/env bash
set -euo pipefail

script_dir=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)
repo_root=$(cd -- "$script_dir/../.." && pwd)
case "$repo_root" in
    */.cerebro/worktrees/*)
        repo_root=$(cd -- "$repo_root/../../.." && pwd)
        ;;
esac
reference_source="$repo_root/../Atlantis"

if [[ ! -f "$reference_source/Makefile" ]]; then
    printf 'C++ reference checkout not found: %s\n' "$reference_source" >&2
    exit 1
fi

build_root=$(mktemp -d "${TMPDIR:-/tmp}/atlantis-reference.XXXXXX")
reference_copy="$build_root/Atlantis"
printf 'Copying reference source to %s\n' "$reference_copy" >&2
cp -R "$reference_source" "$reference_copy"
rm -rf "$reference_copy/.git"

report_failure() {
    status=$?
    if [[ "$status" -ne 0 ]]; then
        printf 'Reference build failed; copied source retained at %s\n' "$reference_copy" >&2
    fi
}
trap report_failure EXIT

make -C "$reference_copy" GAME=standard clean >&2
make -C "$reference_copy" GAME=neworigins clean >&2

shopt -s nullglob
patches=("$script_dir"/patches/*.patch)
for patch_file in "${patches[@]}"; do
    printf 'Applying %s\n' "${patch_file##*/}" >&2
    patch -p1 -d "$reference_copy" < "$patch_file" >&2
done

compiler=${CXX:-g++}
compiler_flags='-g -I. -I.. -Wall -Werror -std=c++20'
if "$compiler" --version 2>&1 | grep -qi clang; then
    compiler_flags+=' -Wno-error=vla-cxx-extension'
fi

printf '%s\n' 'Building standard reference' >&2
make -C "$reference_copy" CPLUS="$compiler" CFLAGS="$compiler_flags" standard >&2
printf '%s\n' 'Building neworigins reference' >&2
make -C "$reference_copy" CPLUS="$compiler" CFLAGS="$compiler_flags" neworigins >&2

if [[ ! -x "$reference_copy/standard/standard" || ! -x "$reference_copy/neworigins/neworigins" ]]; then
    printf '%s\n' 'Reference build completed without both expected executables' >&2
    exit 1
fi

trap - EXIT
printf '%s\n' "$build_root"
