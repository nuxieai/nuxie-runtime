#!/bin/sh
set -eu

expected_ref=d788e8ec6e8b598526607d6a1e8818e8b637b60c
script_dir=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
tool_root=$(CDPATH= cd -- "$script_dir/.." && pwd)
rive_runtime_dir=${RIVE_RUNTIME_DIR:-/Users/levi/dev/oss/rive-runtime}
build_dir=${EDITOR_NEXT_DEFECT_BUILD_DIR:-"$tool_root/build"}
compiler=${CXX:-/usr/bin/clang++}

actual_ref=$(git -C "$rive_runtime_dir" rev-parse HEAD)
if [ "$actual_ref" != "$expected_ref" ]; then
    echo "pinned C++ checkout mismatch: expected $expected_ref, found $actual_ref" >&2
    exit 1
fi

mkdir -p "$build_dir"
"$compiler" \
    -std=c++20 \
    -Wall \
    -Wextra \
    -Werror \
    "$script_dir/registry.cpp" \
    -o "$build_dir/editor-next-runtime-defect-probe"

stamp="$build_dir/editor-next-runtime-defect-probe.provenance"
source_sha256=$(shasum -a 256 "$script_dir/registry.cpp" | awk '{print $1}')
executable_sha256=$(shasum -a 256 "$build_dir/editor-next-runtime-defect-probe" | awk '{print $1}')
{
    echo "upstream_ref=$actual_ref"
    echo "compiler=$compiler"
    echo "flags=-std=c++20 -Wall -Wextra -Werror"
    echo "source=tools/editor-next-runtime-defects/cpp_probe/registry.cpp"
    echo "source_sha256=$source_sha256"
    echo "executable_sha256=$executable_sha256"
} >"$stamp"

echo "$build_dir/editor-next-runtime-defect-probe"
