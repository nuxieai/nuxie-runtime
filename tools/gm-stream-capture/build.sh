#!/bin/bash
set -euo pipefail

script_dir="$(cd "$(dirname "$0")" && pwd)"
repo_root="$(cd "$script_dir/../.." && pwd)"
rive_runtime="${RIVE_RUNTIME_DIR:-/Users/levi/dev/oss/rive-runtime}"
config="${1:-release}"
jobs="$(sysctl -n hw.logicalcpu 2>/dev/null || nproc)"

if [[ "$config" != "debug" && "$config" != "release" ]]; then
    echo "usage: tools/gm-stream-capture/build.sh [debug|release]" >&2
    exit 2
fi

"$script_dir/generate-registry.sh"

# The captured streams are oracle output, so the archive behind them has to be
# the pinned runtime -- not whatever the shared checkout last built into
# tests/out (here: stamped at the audit ref, compiled against non-grid yoga).
# Reuses the golden runner's per-repo ordinary librive tree, exactly like
# tools/promise-oracle/build.sh reuses its scripted one; the provenance stamp
# binds the archive to the pinned revision, defines, compiler, and oracle
# patch set.
runtime_out="$repo_root/target/golden-runner-librive/ordinary-$config"
runtime_archive="$runtime_out/librive.a"
runtime_makefile="$runtime_out/rive.make"
runtime_stamp="$runtime_archive.provenance"
provenance="$repo_root/tools/golden-runner/runtime-provenance.sh"

if [[ "$(uname -s)" == "Darwin" ]]; then
    : "${CC:=/usr/bin/clang}"
    : "${CXX:=/usr/bin/clang++}"
    export CC CXX
fi

"$provenance" source "$rive_runtime"
if ! "$provenance" verify \
    "$rive_runtime" \
    "$runtime_archive" \
    "$runtime_makefile" \
    "$runtime_stamp" \
    "$config" \
    ordinary >/dev/null 2>&1; then
    echo "==== Building provenance-bound ordinary librive ($config) ===="
    RIVE_RUNTIME_DIR="$rive_runtime" \
        bash "$repo_root/tools/golden-runner/build.sh" "$config"
fi
"$provenance" verify \
    "$rive_runtime" \
    "$runtime_archive" \
    "$runtime_makefile" \
    "$runtime_stamp" \
    "$config" \
    ordinary >/dev/null
echo "gm-stream-capture librive provenance: $runtime_stamp"

export RIVE_GM_CAPTURE_RUNTIME_LIBDIR="$runtime_out"

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
