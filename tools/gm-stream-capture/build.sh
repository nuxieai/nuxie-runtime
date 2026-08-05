#!/bin/bash
set -euo pipefail

script_dir="$(cd "$(dirname "$0")" && pwd)"
config="${1:-release}"
jobs="$(sysctl -n hw.logicalcpu 2>/dev/null || nproc)"

if [[ "$config" != "debug" && "$config" != "release" ]]; then
    echo "usage: tools/gm-stream-capture/build.sh [debug|release]" >&2
    exit 2
fi

"$script_dir/generate-registry.sh"

# The captured streams are oracle output, so the archive behind them has to be
# the pinned runtime -- not whatever the shared checkout last built into
# tests/out. This verifies or rebuilds a provenance-bound librive at the pin,
# compiled from the same dependencies tree the tool's include path resolves.
if [[ "$(uname -s)" == "Darwin" ]]; then
    : "${CC:=/usr/bin/clang}"
    : "${CXX:=/usr/bin/clang++}"
    export CC CXX
fi
RIVE_GM_CAPTURE_RUNTIME_LIBDIR="$("$script_dir/../build-support/pinned-librive.sh" "$config")"
export RIVE_GM_CAPTURE_RUNTIME_LIBDIR

cd "$script_dir/build"
premake5 gmake2

# Premake's generated dependencies track headers, not the flags used to find
# them. An existing object tree therefore survives a change to INCLUDES or
# DEFINES and links objects compiled against two different upstream ABIs --
# exactly the failure the pinned dependency resolution exists to prevent.
# Clean only when those flags actually move, so ordinary rebuilds stay
# incremental across ~60 translation units.
stamp="macosx/obj/$config/.buildflags"
flags="$(grep -E '^(INCLUDES|DEFINES|FORCE_INCLUDE|ALL_CXXFLAGS) ' gm_stream_capture.make)"
if [[ ! -f "$stamp" || "$flags" != "$(cat "$stamp")" ]]; then
    make "config=$config" clean
fi

make "config=$config" -j"$jobs"

mkdir -p "$(dirname "$stamp")"
printf '%s\n' "$flags" >"$stamp"
