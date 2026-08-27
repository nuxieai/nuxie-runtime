#!/usr/bin/env bash
set -euo pipefail

if [[ $# -ne 1 ]]; then
    echo "usage: $0 /path/to/rive-runtime" >&2
    exit 2
fi

upstream_checkout=$1
script_directory=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
repository_root=$(dirname -- "$script_directory")
overrides_file="$repository_root/docs/runtime-source-map-overrides.tsv"
scratch_directory=$(mktemp -d)

cleanup() {
    rm -rf -- "$scratch_directory"
}
trap cleanup EXIT

upstream_sources="$scratch_directory/upstream-sources"
rust_sources="$scratch_directory/rust-sources"

rg --files "$upstream_checkout/src" -g '*.cpp' | LC_ALL=C sort > "$upstream_sources"
rg --files "$repository_root/crates" -g '*.rs' | LC_ALL=C sort > "$rust_sources"

printf 'source\tstatus\ttarget\tnote\n'
while IFS= read -r source_file; do
    relative_source=${source_file#"$upstream_checkout/src/"}
    override=$(awk -F '\t' -v source="$relative_source" '
        $1 == source { print; exit }
    ' "$overrides_file")
    if [[ -n "$override" ]]; then
        printf '%s\n' "$override"
        continue
    fi

    case "$relative_source" in
        generated/*)
            printf '%s\t%s\t%s\t%s\n' \
                "$relative_source" \
                'blocked by an approved adaptation decision' \
                'crates/nuxie-schema' \
                'schema-generated Rust owner'
            continue
            ;;
        audio/*)
            printf '%s\t%s\t%s\t%s\n' \
                "$relative_source" \
                'blocked by an approved adaptation decision' \
                'Rust-native audio backend' \
                'approved backend adaptation'
            continue
            ;;
        lua/*|scripted/*)
            printf '%s\t%s\t%s\t%s\n' \
                "$relative_source" \
                'blocked by an approved adaptation decision' \
                'crates/nuxie-scripting' \
                'approved Rust-native scripting adaptation'
            continue
            ;;
    esac

    source_basename=$(basename -- "$relative_source" .cpp)
    target_basename="$source_basename.rs"
    targets=$(awk -F/ -v target="$target_basename" '
        $NF == target { print }
    ' "$rust_sources" | paste -sd ',' -)

    if [[ -z "$targets" ]]; then
        printf '%s\t%s\t\t%s\n' \
            "$relative_source" \
            'not started' \
            'no same-named Rust source owner'
    else
        printf '%s\t%s\t%s\t%s\n' \
            "$relative_source" \
            'needs source correction' \
            "$targets" \
            'candidate exists; complete pair not yet mechanically confirmed'
    fi
done < "$upstream_sources"
