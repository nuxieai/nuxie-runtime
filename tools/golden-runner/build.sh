#!/bin/bash
set -euo pipefail

script_dir="$(cd "$(dirname "$0")" && pwd)"
rive_runtime="${RIVE_RUNTIME_DIR:-/Users/levi/dev/oss/rive-runtime}"
runner_name="${RIVE_GOLDEN_RUNNER_NAME:-rive_golden_runner}"

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

jobs="$(sysctl -n hw.logicalcpu 2>/dev/null || nproc)"

if [[ "${RIVE_GOLDEN_WITH_SCRIPTING:-0}" == "1" ]]; then
    runtime_out="${RIVE_GOLDEN_SCRIPTING_OUT:-out/rive-rust-golden-scripting-$config}"
    decoders_out="${RIVE_GOLDEN_DECODERS_OUT:-out/rive-rust-golden-scripting-$config}"
    echo "==== Building scripted librive ($config) ===="
    (
        cd "$rive_runtime"
        PREMAKE_PATH="$rive_runtime/build${PREMAKE_PATH:+:$PREMAKE_PATH}" \
            premake5 gmake2 \
            --file=premake5_v2.lua \
            --config="$config" \
            --out="$runtime_out" \
            --with_rive_text \
            --with_rive_layout \
            --with_rive_scripting
        make -C "$runtime_out" -j"$jobs" \
            rive rive_harfbuzz rive_sheenbidi rive_yoga luau_vm
    )
    export RIVE_GOLDEN_SCRIPTING_LIBDIR="$rive_runtime/$runtime_out"
    echo "==== Building scripted rive_decoders ($config) ===="
    (
        cd "$rive_runtime/decoders"
        PREMAKE_PATH="$rive_runtime/build${PREMAKE_PATH:+:$PREMAKE_PATH}" \
            premake5 gmake2 \
            --file=premake5_v2.lua \
            --config="$config" \
            --out="$decoders_out"
        make -C "$decoders_out" -j"$jobs" \
            rive_decoders libpng zlib libjpeg libwebp
    )
    export RIVE_GOLDEN_DECODERS_LIBDIR="$rive_runtime/decoders/$decoders_out"
fi

cd "$script_dir/build"
premake5 gmake2

# Premake's generated dependencies do not capture changes to compiler flags or
# include paths. Reusing these two objects after RIVE_RUNTIME_DIR changes can
# therefore compile against one upstream ABI and link against another. Clean
# the current mode/config before every build; the runner has only two sources.
make "config=$config" clean
make "config=$config" -j"$jobs"
