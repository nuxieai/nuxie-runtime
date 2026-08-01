#!/bin/bash
set -euo pipefail

script_dir="$(cd "$(dirname "$0")" && pwd)"
repo_root="$(cd "$script_dir/../.." && pwd)"
rive_runtime="${RIVE_RUNTIME_DIR:-/Users/levi/dev/oss/rive-runtime}"
config="${1:-release}"
runtime_out="$rive_runtime/out/rive-rust-golden-scripting-$config"
luau_root="$rive_runtime/dependencies/luigi-rosso_luau_rive_0_728_vec3"
output_dir="$repo_root/target/promise-oracle"
output="$output_dir/rive_cpp_promise_oracle"
provenance="$repo_root/tools/golden-runner/runtime-provenance.sh"
runtime_archive="$runtime_out/librive.a"
runtime_makefile="$runtime_out/rive.make"
runtime_stamp="$runtime_archive.provenance"

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
    -I"$rive_runtime/dependencies/rive-app_harfbuzz_rive_13.1.1/src" \
    -I"$rive_runtime/dependencies/Tehreer_SheenBidi_v2.6/Headers" \
    -I"$rive_runtime/dependencies/rive-app_miniaudio_rive_changes_5" \
    -I"$rive_runtime/dependencies/rive-app_yoga_rive_changes_v2_0_1_2" \
    -I"$luau_root/VM/include" \
    "$script_dir/main.cpp" \
    "$output_dir/lua_promise.o" \
    "$runtime_out/libluau_vm.a" \
    -o "$output"

echo "$output"
