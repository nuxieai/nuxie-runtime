#!/bin/bash
set -euo pipefail

root="$(cd "$(dirname "$0")/.." && pwd)"
committed="$root/crates/nuxie-renderer/src/generated"
work="$(mktemp -d "${TMPDIR:-/tmp}/nuxie-renderer-shaders.XXXXXX")"
trap 'rm -rf "$work"' EXIT

expected_module_count="66"
expected_module_digest="03867c8c0a2240066043aef6d54d05cd4f02008295a70bb4d244e78f139ca953"
expected_cpp_header_count="56"
expected_cpp_header_digest="3089dd3b0baea38b2c9cabb4062e3908dc18f4debe46f3158ab9ac6678abd7d0"

RIVE_RUNTIME_DIR="${RIVE_RUNTIME_DIR:-/Users/levi/dev/oss/rive-runtime}" \
RENDERER_SHADER_OUTPUT_DIR="$work/generated" \
RENDERER_SHADER_UPSTREAM_OUT="$work/upstream" \
    "$root/tools/generate-renderer-shaders.sh"

diff -ru "$committed" "$work/generated"

module_count="$(find "$work/generated" -name '*.wgsl' -type f | wc -l | tr -d ' ')"
module_digest="$(cd "$work/generated" && find . -name '*.wgsl' -type f -print0 \
    | sort -z \
    | xargs -0 shasum -a 256 \
    | shasum -a 256 \
    | awk '{print $1}')"
cpp_header_count="$(find "$work/upstream/wgsl" -name '*.hpp' -type f | wc -l | tr -d ' ')"
cpp_header_digest="$(cd "$work/upstream" && find wgsl -name '*.hpp' -type f -print0 \
    | sort -z \
    | xargs -0 shasum -a 256 \
    | shasum -a 256 \
    | awk '{print $1}')"

if [[ "$module_count" != "$expected_module_count" ]]; then
    echo "wrong Rust WGSL module count: expected $expected_module_count, got $module_count" >&2
    exit 1
fi
if [[ "$module_digest" != "$expected_module_digest" ]]; then
    echo "wrong Rust WGSL digest: expected $expected_module_digest, got $module_digest" >&2
    exit 1
fi
if [[ "$cpp_header_count" != "$expected_cpp_header_count" ]]; then
    echo "wrong C++ WGSL header count: expected $expected_cpp_header_count, got $cpp_header_count" >&2
    exit 1
fi
if [[ "$cpp_header_digest" != "$expected_cpp_header_digest" ]]; then
    echo "wrong C++ WGSL header digest: expected $expected_cpp_header_digest, got $cpp_header_digest" >&2
    exit 1
fi

echo "renderer shaders reproducible: rust-modules=$module_count rust-digest=$module_digest cpp-headers=$cpp_header_count cpp-digest=$cpp_header_digest"
