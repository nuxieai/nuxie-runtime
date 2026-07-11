#!/bin/bash
set -euo pipefail

script_dir="$(cd "$(dirname "$0")" && pwd)"
config="${1:-release}"
jobs="$(sysctl -n hw.logicalcpu 2>/dev/null || nproc)"

if [[ "$config" != "debug" && "$config" != "release" ]]; then
    echo "usage: tools/gm-stream-capture/build.sh [debug|release]" >&2
    exit 2
fi

"$script_dir/generate-registry.sh"

cd "$script_dir/build"
premake5 gmake2
make "config=$config" -j"$jobs"
