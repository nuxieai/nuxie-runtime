#!/bin/bash
# Builds (or reuses) a provenance-bound librive at the pinned upstream revision
# and prints its lib directory on stdout. All progress output goes to stderr so
# callers can capture the path directly:
#
#   libdir="$(tools/build-support/pinned-librive.sh release)"
#
# Why a tool cannot just link $RIVE_RUNTIME_DIR/tests/out/<config>: that tree is
# whatever the shared pinned checkout was last told to build. Here it carries a
# librive stamped at the audit ref (d788e8ec) rather than the pin, compiled
# against a separate tests/dependencies/ tree holding non-grid yoga. Linking it
# silently mixes upstream revisions -- and because rive/layout/layout_data.hpp
# stores YGNode/YGStyle by value, a yoga mismatch is an ABI mismatch.
#
# The provenance stamp (tools/golden-runner/runtime-provenance.sh) binds the
# archive to the pinned revision, the feature defines, the compiler, and the
# registered oracle patch set, and is re-verified on every invocation.
set -euo pipefail

script_dir="$(cd "$(dirname "$0")" && pwd)"
repo_root="$(cd "$script_dir/../.." && pwd)"
provenance="$repo_root/tools/golden-runner/runtime-provenance.sh"
rive_runtime="${RIVE_RUNTIME_DIR:-/Users/levi/dev/oss/rive-runtime}"

config="${1:-release}"
if [[ "$config" != "debug" && "$config" != "release" ]]; then
    echo "usage: $0 [debug|release]" >&2
    exit 2
fi

# Rive's macOS release archives contain LLVM bitcode. Letting PATH select a
# Homebrew LLVM while the final link targets the Xcode SDK can produce archives
# Apple ld cannot consume. Use the platform toolchain by default, preserving an
# explicit caller override.
if [[ "$(uname -s)" == "Darwin" ]]; then
    : "${CC:=/usr/bin/clang}"
    : "${CXX:=/usr/bin/clang++}"
    export CC CXX
fi

mode="ordinary"
libdir="${PINNED_LIBRIVE_OUT:-$repo_root/target/pinned-librive/$mode-$config}"
archive="$libdir/librive.a"
makefile="$libdir/rive.make"
stamp="$archive.provenance"
jobs="$(sysctl -n hw.logicalcpu 2>/dev/null || nproc)"

"$provenance" source "$rive_runtime" >&2

if ! "$provenance" verify "$rive_runtime" "$archive" "$makefile" "$stamp" "$config" "$mode" >/dev/null 2>&1; then
    echo "==== Building provenance-bound $mode librive ($config) ====" >&2

    # Oracle patches never touch the shared pinned checkout: they are applied
    # to an isolated `git archive` of the pin and librive compiles from there.
    # Downloaded third-party dependencies are reused from the shared checkout
    # via the DEPENDENCIES override honored by build/dependency.lua -- which is
    # also what makes the tool's include path and this archive agree on yoga.
    # The materialized tree is per-invocation so concurrent builds in the same
    # repo cannot delete each other's sources mid-compile.
    mkdir -p "$repo_root/target/pinned-librive"
    materialize_dest="$(mktemp -d "$repo_root/target/pinned-librive/patched-runtime-src.XXXXXX")"
    build_src="$("$provenance" materialize "$rive_runtime" "$materialize_dest")"
    build_out="$(python3 -c 'import os,sys; print(os.path.relpath(sys.argv[1], sys.argv[2]))' "$libdir" "$build_src")"
    (
        cd "$build_src"
        DEPENDENCIES="$rive_runtime/dependencies" \
            PREMAKE_PATH="$build_src/build${PREMAKE_PATH:+:$PREMAKE_PATH}" \
            premake5 gmake2 \
            --file=premake5_v2.lua \
            --config="$config" \
            --out="$build_out" \
            --with_rive_text --with_rive_layout >&2
        make -C "$build_out" clean >&2
        make -C "$build_out" -j"$jobs" rive rive_harfbuzz rive_sheenbidi rive_yoga >&2
    )
    "$provenance" write "$rive_runtime" "$archive" "$makefile" "$stamp" "$config" "$mode"
    rm -rf "$materialize_dest"
fi

"$provenance" verify "$rive_runtime" "$archive" "$makefile" "$stamp" "$config" "$mode" >&2
echo "pinned librive provenance: $stamp" >&2
printf '%s\n' "$libdir"
