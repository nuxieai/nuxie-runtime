#!/bin/bash
set -euo pipefail

script_dir="$(cd "$(dirname "$0")" && pwd)"
root="$(cd "$script_dir/../.." && pwd)"
binary="$script_dir/build/$(uname -s | tr A-Z a-z | sed 's/darwin/macosx/')/bin/release/gm_stream_capture"
output_dir="${1:-$root/fixtures/renderer/streams/gm}"
failures="$output_dir/gated.txt"

mkdir -p "$output_dir"
cp "$script_dir/generated-gates.txt" "$failures"
while read -r entry; do
    name="${entry#GM_ENTRY(}"
    name="${name%)}"
    if ! "$binary" "$name" "$output_dir/$name.rive-stream"; then
        rm -f "$output_dir/$name.rive-stream"
        printf '%s\tcapture-runtime-failure\n' "$name" >>"$failures"
    fi
done <"$script_dir/generated_registry.inc"
