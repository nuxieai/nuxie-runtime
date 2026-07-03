#!/bin/bash
set -euo pipefail

cd "$(dirname "$0")/build"

config="${1:-debug}"
if [[ "$config" == "clean" ]]; then
    premake5 gmake2
    make clean
    exit 0
fi

if [[ "$config" != "debug" && "$config" != "release" ]]; then
    echo "usage: tools/golden-runner/build.sh [debug|release|clean]" >&2
    exit 2
fi

premake5 gmake2
make "config=$config" -j"$(sysctl -n hw.logicalcpu 2>/dev/null || nproc)"
