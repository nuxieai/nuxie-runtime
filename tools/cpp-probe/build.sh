#!/bin/bash
set -euo pipefail

script_dir="$(cd "$(dirname "$0")" && pwd)"
rive_runtime="${RIVE_RUNTIME_DIR:-/Users/levi/dev/oss/rive-runtime}"
provenance="$script_dir/../golden-runner/runtime-provenance.sh"

config="${1:-debug}"
if [[ "$config" == "clean" ]]; then
    export RIVE_CPP_PROBE_RUNTIME_LIBDIR="$rive_runtime/out/rive-rust-cpp-probe-debug"
    cd "$script_dir/build"
    premake5 gmake2
    make clean
    exit 0
fi

if [[ "$config" != "debug" && "$config" != "release" ]]; then
    echo "usage: tools/cpp-probe/build.sh [debug|release|clean]" >&2
    exit 2
fi

if [[ "$(uname -s)" == "Darwin" ]]; then
    : "${CC:=/usr/bin/clang}"
    : "${CXX:=/usr/bin/clang++}"
    export CC CXX
fi

jobs="$(sysctl -n hw.logicalcpu 2>/dev/null || nproc)"
runtime_out="${RIVE_CPP_PROBE_RUNTIME_OUT:-out/rive-rust-cpp-probe-$config}"
runtime_libdir="$rive_runtime/$runtime_out"
runtime_archive="$runtime_libdir/librive.a"
runtime_makefile="$runtime_libdir/rive.make"
runtime_stamp="$runtime_archive.provenance"

"$provenance" source "$rive_runtime"
if ! "$provenance" verify \
    "$rive_runtime" \
    "$runtime_archive" \
    "$runtime_makefile" \
    "$runtime_stamp" \
    "$config" \
    ordinary >/dev/null 2>&1; then
    echo "==== Building provenance-bound C++ probe librive ($config) ===="
    (
        cd "$rive_runtime"
        PREMAKE_PATH="$rive_runtime/build${PREMAKE_PATH:+:$PREMAKE_PATH}" \
            premake5 gmake2 \
            --file=premake5_v2.lua \
            --config="$config" \
            --out="$runtime_out" \
            --with_rive_text \
            --with_rive_layout
        make -C "$runtime_out" clean
        make -C "$runtime_out" -j"$jobs" \
            rive rive_harfbuzz rive_sheenbidi rive_yoga
    )
    "$provenance" write \
        "$rive_runtime" \
        "$runtime_archive" \
        "$runtime_makefile" \
        "$runtime_stamp" \
        "$config" \
        ordinary
fi
"$provenance" verify \
    "$rive_runtime" \
    "$runtime_archive" \
    "$runtime_makefile" \
    "$runtime_stamp" \
    "$config" \
    ordinary
export RIVE_CPP_PROBE_RUNTIME_LIBDIR="$runtime_libdir"
echo "C++ probe librive provenance: $runtime_stamp"

cd "$script_dir/build"
premake5 gmake2
# Premake regenerates the project paths, but existing dependency files retain
# absolute source paths from the checkout that produced them.  Pin verification
# deliberately builds against a temporary checkout, so reusing those files can
# leave this ordinary build depending on a checkout that no longer exists.
make "config=$config" clean
make "config=$config" -j"$jobs"
