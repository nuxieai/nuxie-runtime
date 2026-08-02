#!/bin/bash
set -euo pipefail

probe_dir="$(cd "$(dirname "$0")" && pwd)"
cd "$probe_dir/build"

config="${1:-debug}"
if [[ "$config" == "clean" ]]; then
    premake5 gmake2
    make clean
    rm -rf generated
    exit 0
fi

if [[ "$config" != "debug" && "$config" != "release" ]]; then
    echo "usage: tools/cpp-probe/build.sh [debug|release|clean]" >&2
    exit 2
fi

# Embed a fingerprint of the probe's build inputs so the Rust differential
# harness can reject a stale binary (`rive_cpp_probe --fingerprint`). The
# input list and hash construction must stay in lockstep with
# expected_probe_fingerprint() in the Rust cpp-probe test harnesses.
fingerprint="$(
    {
        echo "nuxie-cpp-probe-source/v1"
        for input in main.cpp testing_random_provider.cpp build/premake5.lua build.sh; do
            printf '%s:%s\n' "$input" "$(shasum -a 256 "$probe_dir/$input" | cut -d' ' -f1)"
        done
    } | shasum -a 256 | cut -d' ' -f1
)"
mkdir -p generated
header="generated/probe_source_fingerprint.h"
header_content="#pragma once
#define PROBE_SOURCE_FINGERPRINT \"$fingerprint\""
if [[ ! -f "$header" || "$(cat "$header")" != "$header_content" ]]; then
    printf '%s\n' "$header_content" > "$header"
fi

premake5 gmake2
make "config=$config" -j"$(sysctl -n hw.logicalcpu 2>/dev/null || nproc)"
