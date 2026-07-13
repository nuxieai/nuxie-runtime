#!/usr/bin/env bash
set -euo pipefail

expected_runtime_ref=7c778d13c5d903b3b74eec1dd6bb68a811dea5f2
runtime_dir=${RIVE_RUNTIME_DIR:-/Users/levi/dev/oss/rive-runtime}
decoder_archive="$runtime_dir/renderer/out/debug/librive_decoders.a"

actual_runtime_ref=$(git -C "$runtime_dir" rev-parse HEAD)
if [[ "$actual_runtime_ref" != "$expected_runtime_ref" ]]; then
    echo "decoder oracle requires runtime $expected_runtime_ref, found $actual_runtime_ref" >&2
    exit 1
fi

dirty=$(git -C "$runtime_dir" status --porcelain --untracked-files=no -- \
    decoders renderer/src/render_context.cpp)
untracked=$(git -C "$runtime_dir" ls-files --others --exclude-standard -- \
    decoders renderer/src/render_context.cpp | \
    grep -Ev '^decoders/(dependencies|out)/' || true)
dirty+="$untracked"
if [[ -n "$dirty" ]]; then
    echo "decoder oracle requires clean pinned decoder sources:" >&2
    echo "$dirty" >&2
    exit 1
fi

if [[ ! -f "$decoder_archive" ]]; then
    echo "missing $decoder_archive; build the pinned C++ renderer decoders first" >&2
    exit 1
fi

newer_source=$(find "$runtime_dir/decoders/src" \
    "$runtime_dir/decoders/include" \
    "$runtime_dir/decoders/build" \
    "$runtime_dir/decoders/premake5_v2.lua" -type f \
    -newer "$decoder_archive" -print -quit)
if [[ -n "$newer_source" ]]; then
    echo "decoder archive is older than $newer_source; rebuild the pinned C++ renderer decoders" >&2
    exit 1
fi

echo "renderer decoder provenance: runtime=$actual_runtime_ref archive=$decoder_archive"
