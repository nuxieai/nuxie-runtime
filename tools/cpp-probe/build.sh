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
    echo "usage: tools/cpp-probe/build.sh [debug|release|clean]" >&2
    exit 2
fi

premake5 gmake2
# Premake regenerates the project paths, but existing dependency files retain
# absolute source paths from the checkout that produced them.  Pin verification
# deliberately builds against a temporary checkout, so reusing those files can
# leave this ordinary build depending on a checkout that no longer exists.
make "config=$config" clean
make "config=$config" -j"$(sysctl -n hw.logicalcpu 2>/dev/null || nproc)"
