#!/bin/bash
set -euo pipefail

script_dir="$(cd "$(dirname "$0")" && pwd)"
rive_runtime="${RIVE_RUNTIME_DIR:-/Users/levi/dev/oss/rive-runtime}"
provenance="$script_dir/../golden-runner/runtime-provenance.sh"
with_scripting="${RIVE_CPP_PROBE_WITH_SCRIPTING:-0}"

config="${1:-debug}"
if [[ "$config" == "clean" ]]; then
    export RIVE_CPP_PROBE_RUNTIME_LIBDIR="$rive_runtime/out/rive-rust-cpp-probe-debug"
    cd "$script_dir/build"
    premake5 gmake2
    make clean
    rm -rf generated
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

# librive build trees default under this repo's own target/ rather than the
# shared pinned checkout's out/. Concurrent worktree batteries racing
# `make clean`+rebuild in one shared build tree can delete or partially
# rewrite librive.a between another battery's provenance verify and its
# link. Per-repo trees make the shared checkout read-only for builds;
# explicit *_OUT overrides still win.
repo_target="$(cd "$script_dir/../.." && pwd)/target"

if [[ "$with_scripting" == "1" ]]; then
    runtime_mode="scripted"
    runtime_out="${RIVE_CPP_PROBE_RUNTIME_OUT:-$repo_target/cpp-probe-librive/scripted-$config}"
    decoders_out="${RIVE_CPP_PROBE_DECODERS_OUT:-$repo_target/cpp-probe-librive/scripted-$config-decoders}"
    # Scripted librive carries the pinned Lua audio engine, so it builds with
    # --with_rive_audio=external and links miniaudio -- matching the scripted
    # golden runner and the `scripted` expectation in runtime-provenance.sh.
    runtime_premake_flags=(
        --with_rive_text
        --with_rive_layout
        --with_rive_scripting
        --with_rive_audio=external
    )
    runtime_targets=(rive rive_harfbuzz rive_sheenbidi rive_yoga luau_vm miniaudio)
    export RIVE_CPP_PROBE_RUNNER_NAME="${RIVE_CPP_PROBE_RUNNER_NAME:-rive_cpp_probe_scripted}"
else
    runtime_mode="audio"
    runtime_out="${RIVE_CPP_PROBE_RUNTIME_OUT:-$repo_target/cpp-probe-librive/audio-$config}"
    runtime_premake_flags=(--with_rive_text --with_rive_layout --with_rive_audio=external)
    runtime_targets=(rive miniaudio rive_harfbuzz rive_sheenbidi rive_yoga)
    export RIVE_CPP_PROBE_WITH_AUDIO=1
    export RIVE_CPP_PROBE_RUNNER_NAME="${RIVE_CPP_PROBE_RUNNER_NAME:-rive_cpp_probe}"
fi
if [[ "$runtime_out" = /* ]]; then
    runtime_libdir="$runtime_out"
else
    runtime_libdir="$rive_runtime/$runtime_out"
fi
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
    "$runtime_mode" >/dev/null 2>&1; then
    echo "==== Building provenance-bound C++ probe librive ($config) ===="
    # Registered oracle patches build from an isolated patched copy of the
    # pin; the shared checkout is never written. The copy is per-invocation
    # so concurrent batteries cannot delete each other's build sources
    # mid-compile. See runtime-provenance.sh.
    mkdir -p "$repo_target/cpp-probe-librive"
    materialize_dest="$(mktemp -d "$repo_target/cpp-probe-librive/patched-runtime-src.XXXXXX")"
    runtime_build_src="$("$provenance" materialize "$rive_runtime" "$materialize_dest")"
    runtime_build_out_from_src="$(python3 -c 'import os,sys; print(os.path.relpath(sys.argv[1], sys.argv[2]))' "$runtime_libdir" "$runtime_build_src")"
    (
        cd "$runtime_build_src"
        DEPENDENCIES="$rive_runtime/dependencies" \
            PREMAKE_PATH="$runtime_build_src/build${PREMAKE_PATH:+:$PREMAKE_PATH}" \
            premake5 gmake2 \
            --file=premake5_v2.lua \
            --config="$config" \
            --out="$runtime_build_out_from_src" \
            "${runtime_premake_flags[@]}"
        make -C "$runtime_build_out_from_src" clean
        make -C "$runtime_build_out_from_src" -j"$jobs" "${runtime_targets[@]}"
    )
    "$provenance" write \
        "$rive_runtime" \
        "$runtime_archive" \
        "$runtime_makefile" \
        "$runtime_stamp" \
        "$config" \
        "$runtime_mode"
    rm -rf "$materialize_dest"
fi
"$provenance" verify \
    "$rive_runtime" \
    "$runtime_archive" \
    "$runtime_makefile" \
    "$runtime_stamp" \
    "$config" \
    "$runtime_mode"
export RIVE_CPP_PROBE_RUNTIME_LIBDIR="$runtime_libdir"
echo "C++ probe librive provenance: $runtime_stamp"

if [[ "$with_scripting" == "1" ]]; then
    echo "==== Building scripted C++ probe decoders ($config) ===="
    if [[ "$decoders_out" = /* ]]; then
        decoders_libdir="$decoders_out"
        decoders_build_out="$(python3 -c 'import os,sys; print(os.path.relpath(sys.argv[1], sys.argv[2]))' "$decoders_libdir" "$rive_runtime/decoders")"
    else
        decoders_libdir="$rive_runtime/decoders/$decoders_out"
        decoders_build_out="$decoders_out"
    fi
    decoder_archives=(
        "$decoders_libdir/librive_decoders.a"
        "$decoders_libdir/liblibpng.a"
        "$decoders_libdir/libzlib.a"
        "$decoders_libdir/liblibjpeg.a"
        "$decoders_libdir/liblibwebp.a"
    )
    decoder_set_complete=1
    for archive in "${decoder_archives[@]}"; do
        if [[ ! -s "$archive" ]]; then
            decoder_set_complete=0
            break
        fi
    done
    if [[ "$decoder_set_complete" == "1" ]]; then
        echo "reusing complete scripted decoder archive set: $decoders_libdir"
    else
        (
            cd "$rive_runtime/decoders"
            PREMAKE_PATH="$rive_runtime/build${PREMAKE_PATH:+:$PREMAKE_PATH}" \
                premake5 gmake2 \
                --file=premake5_v2.lua \
                --config="$config" \
                --out="$decoders_build_out"
            make -C "$decoders_build_out" -j"$jobs" \
                rive_decoders libpng zlib libjpeg libwebp
        )
    fi
    export RIVE_CPP_PROBE_DECODERS_LIBDIR="$decoders_libdir"
fi

# Embed a fingerprint of the probe's build inputs so the Rust differential
# harness can reject a stale binary (`rive_cpp_probe --fingerprint`). The
# input list and hash construction must stay in lockstep with
# expected_probe_fingerprint() in the Rust cpp-probe test harnesses.
fingerprint="$(
    {
        echo "nuxie-cpp-probe-source/v1"
        for input in main.cpp testing_random_provider.cpp build/premake5.lua build.sh; do
            printf '%s:%s\n' "$input" "$(shasum -a 256 "$script_dir/$input" | cut -d' ' -f1)"
        done
    } | shasum -a 256 | cut -d' ' -f1
)"
mkdir -p "$script_dir/build/generated"
fingerprint_header="$script_dir/build/generated/probe_source_fingerprint.h"
fingerprint_content="#pragma once
#define PROBE_SOURCE_FINGERPRINT \"$fingerprint\""
if [[ ! -f "$fingerprint_header" || "$(cat "$fingerprint_header")" != "$fingerprint_content" ]]; then
    printf '%s\n' "$fingerprint_content" > "$fingerprint_header"
fi

cd "$script_dir/build"
premake5 gmake2
# Premake regenerates the project paths, but existing dependency files retain
# absolute source paths from the checkout that produced them.  Pin verification
# deliberately builds against a temporary checkout, so reusing those files can
# leave this ordinary build depending on a checkout that no longer exists.
make "config=$config" clean
make "config=$config" -j"$jobs"
