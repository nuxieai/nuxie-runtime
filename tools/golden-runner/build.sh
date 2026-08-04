#!/bin/bash
set -euo pipefail

script_dir="$(cd "$(dirname "$0")" && pwd)"
rive_runtime="${RIVE_RUNTIME_DIR:-/Users/levi/dev/oss/rive-runtime}"
runner_name="${RIVE_GOLDEN_RUNNER_NAME:-rive_golden_runner}"
provenance="$script_dir/runtime-provenance.sh"

config="${1:-debug}"
if [[ "$config" == "clean" ]]; then
    cd "$script_dir/build"
    premake5 gmake2
    make clean
    exit 0
fi

if [[ "$config" != "debug" && "$config" != "release" ]]; then
    echo "usage: tools/golden-runner/build.sh [debug|release|clean]" >&2
    exit 2
fi

# Rive's macOS release archives contain LLVM bitcode. Letting PATH select a
# Homebrew LLVM while the final link targets the Xcode SDK can produce archives
# that Apple ld cannot consume (and has failed here as "Unsupported stack
# probing method"). Use the platform toolchain by default while preserving an
# explicit caller override.
if [[ "$(uname -s)" == "Darwin" ]]; then
    : "${CC:=/usr/bin/clang}"
    : "${CXX:=/usr/bin/clang++}"
    export CC CXX
fi

jobs="$(sysctl -n hw.logicalcpu 2>/dev/null || nproc)"
"$provenance" source "$rive_runtime"

# librive build trees default under this repo's own target/ rather than the
# shared pinned checkout's out/. The pinned checkout is shared by every
# worktree; concurrent batteries racing `make clean`+rebuild in one shared
# build tree can delete or partially rewrite librive.a between another
# battery's provenance verify and its link. Per-repo trees make the shared
# checkout read-only for builds; explicit *_OUT overrides still win.
repo_target="$(cd "$script_dir/../.." && pwd)/target"

if [[ "${RIVE_GOLDEN_WITH_SCRIPTING:-0}" == "1" ]]; then
    runtime_mode="scripted"
    runtime_out="${RIVE_GOLDEN_SCRIPTING_OUT:-$repo_target/golden-runner-librive/scripted-$config}"
    decoders_out="${RIVE_GOLDEN_DECODERS_OUT:-$repo_target/golden-runner-librive/scripted-$config-decoders}"
    runtime_premake_flags=(--with_rive_text --with_rive_layout --with_rive_scripting --with_rive_audio=external)
    runtime_targets=(rive rive_harfbuzz rive_sheenbidi rive_yoga luau_vm miniaudio)
else
    runtime_mode="ordinary"
    runtime_out="${RIVE_GOLDEN_RUNTIME_OUT:-$repo_target/golden-runner-librive/ordinary-$config}"
    runtime_premake_flags=(--with_rive_text --with_rive_layout)
    runtime_targets=(rive rive_harfbuzz rive_sheenbidi rive_yoga)
fi

if [[ "$runtime_out" = /* ]]; then
    runtime_libdir="$runtime_out"
else
    runtime_libdir="$rive_runtime/$runtime_out"
fi
runtime_archive="$runtime_libdir/librive.a"
runtime_makefile="$runtime_libdir/rive.make"
runtime_stamp="$runtime_archive.provenance"

if ! "$provenance" verify "$rive_runtime" "$runtime_archive" "$runtime_makefile" "$runtime_stamp" "$config" "$runtime_mode" >/dev/null 2>&1; then
    echo "==== Building provenance-bound $runtime_mode librive ($config) ===="

    # Registered local oracle patches (tools/rive-runtime-patches/librive-*)
    # never touch the shared pinned checkout: like the shader overlay in
    # generate-renderer-shaders.sh, they are applied to an isolated `git
    # archive` of the pin, and librive compiles from that tree. Downloaded
    # third-party dependencies are reused from the shared checkout via the
    # DEPENDENCIES override honored by build/dependency.lua. The provenance
    # stamp binds the archive to the exact patch set (see runtime-provenance.sh).
    # The materialized tree is per-invocation so concurrent batteries in the
    # same repo cannot delete each other's build sources mid-compile.
    mkdir -p "$repo_target/golden-runner-librive"
    materialize_dest="$(mktemp -d "$repo_target/golden-runner-librive/patched-runtime-src.XXXXXX")"
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
    "$provenance" write "$rive_runtime" "$runtime_archive" "$runtime_makefile" "$runtime_stamp" "$config" "$runtime_mode"
    rm -rf "$materialize_dest"
fi
"$provenance" verify "$rive_runtime" "$runtime_archive" "$runtime_makefile" "$runtime_stamp" "$config" "$runtime_mode"
export RIVE_GOLDEN_RUNTIME_LIBDIR="$runtime_libdir"
echo "golden runner librive provenance: $runtime_stamp"

if [[ "$runtime_mode" == "scripted" ]]; then
    echo "==== Building scripted rive_decoders ($config) ===="
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
        # The decoder archives live beside the provenance-bound runtime in the
        # pinned C++ checkout. Reuse a complete set: managed verification
        # worktrees can be intentionally read-only, and deleting those inputs
        # is neither necessary nor permitted there. The live corpus comparison
        # below remains the behavioral compatibility check for the linked set.
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
    export RIVE_GOLDEN_DECODERS_LIBDIR="$decoders_libdir"
fi

cd "$script_dir/build"
premake5 gmake2

# Premake's generated dependencies do not capture changes to compiler flags or
# include paths. Reusing these two objects after RIVE_RUNTIME_DIR changes can
# therefore compile against one upstream ABI and link against another. Clean
# the current mode/config before every build; the runner has only two sources.
make "config=$config" clean
make "config=$config" -j"$jobs"
