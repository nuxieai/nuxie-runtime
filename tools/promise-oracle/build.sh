#!/bin/bash
set -euo pipefail

script_dir="$(cd "$(dirname "$0")" && pwd)"
repo_root="$(cd "$script_dir/../.." && pwd)"
rive_runtime="${RIVE_RUNTIME_DIR:-/Users/levi/dev/oss/rive-runtime}"
config="${1:-release}"
# Reuses the golden runner's per-repo librive tree (tools/golden-runner/
# build.sh default); the shared pinned checkout stays read-only for builds.
runtime_out="$repo_root/target/golden-runner-librive/scripted-$config"
output_dir="$repo_root/target/promise-oracle"
output="$output_dir/rive_cpp_promise_oracle"
provenance="$repo_root/tools/golden-runner/runtime-provenance.sh"
runtime_archive="$runtime_out/librive.a"
runtime_makefile="$runtime_out/rive.make"
runtime_stamp="$runtime_archive.provenance"

# Every header the oracle compiles against must come from the revision the
# pinned librive (and the libluau_vm.a it links below) was built from, so the
# revisions are read out of the pinned runtime's own premake files rather than
# hardcoded here. `rive_dependency_dir` hard-errors instead of falling back, and
# `set -e` turns that into a failed build.
# shellcheck source=../build-support/rive_dependency_dir.sh
source "$repo_root/tools/build-support/rive_dependency_dir.sh"
luau_root="$(rive_dependency_dir "$rive_runtime" luigi-rosso/luau scripting/premake5.lua)"
harfbuzz_root="$(rive_dependency_dir "$rive_runtime" rive-app/harfbuzz dependencies/premake5_harfbuzz_v2.lua)"
sheenbidi_root="$(rive_dependency_dir "$rive_runtime" Tehreer/SheenBidi dependencies/premake5_sheenbidi_v2.lua)"
miniaudio_root="$(rive_dependency_dir "$rive_runtime" rive-app/miniaudio dependencies/premake5_miniaudio_v2.lua)"
# Not reached by main.cpp's include closure today, but rive/layout/layout_data.hpp
# holds a YGNode and a YGStyle by value and those differ by 92 bytes between the
# grid and non-grid yoga tags -- one new rive header in the closure turns a stale
# path here into a silent LayoutData size skew against librive.
yoga_root="$(rive_dependency_dir "$rive_runtime" rive-app/yoga dependencies/premake5_yoga_v2.lua)"

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
    scripted >/dev/null 2>&1; then
    echo "==== Building provenance-bound scripted librive ($config) ===="
    RIVE_RUNTIME_DIR="$rive_runtime" \
        RIVE_GOLDEN_WITH_SCRIPTING=1 \
        RIVE_GOLDEN_RUNNER_NAME=rive_cpp_promise_provenance \
        bash "$repo_root/tools/golden-runner/build.sh" "$config"
fi
"$provenance" verify \
    "$rive_runtime" \
    "$runtime_archive" \
    "$runtime_makefile" \
    "$runtime_stamp" \
    "$config" \
    scripted

test -s "$runtime_out/libluau_vm.a"
mkdir -p "$output_dir"
(
    cd "$output_dir"
    ar -x "$runtime_archive" lua_promise.o
)

"${CXX:-/usr/bin/clang++}" \
    -std=c++17 \
    -DWITH_RIVE_SCRIPTING \
    -DWITH_RIVE_TEXT \
    -DWITH_RIVE_LAYOUT \
    -D_RIVE_INTERNAL_ \
    -I"$rive_runtime/include" \
    -I"$rive_runtime/scripting" \
    -I"$rive_runtime/dependencies" \
    -I"$harfbuzz_root/src" \
    -I"$sheenbidi_root/Headers" \
    -I"$miniaudio_root" \
    -I"$yoga_root" \
    -I"$luau_root/VM/include" \
    "$script_dir/main.cpp" \
    "$output_dir/lua_promise.o" \
    "$runtime_out/libluau_vm.a" \
    -o "$output"

echo "$output"
