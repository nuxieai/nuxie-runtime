#!/usr/bin/env bash
set -euo pipefail

if [[ "$(uname -s)" != "Darwin" ]]; then
  echo "error: the ORE Metal binding witness requires macOS" >&2
  exit 2
fi

repo_root="$(cd "$(dirname "$0")/.." && pwd)"
tree_file="$(mktemp "${TMPDIR:-/tmp}/ore-metal-binding-witness-tree.XXXXXX")"
artifacts_file="$(mktemp "${TMPDIR:-/tmp}/ore-metal-binding-witness-artifacts.XXXXXX")"
symbols_file="$(mktemp "${TMPDIR:-/tmp}/ore-metal-binding-witness-symbols.XXXXXX")"
strings_file="$(mktemp "${TMPDIR:-/tmp}/ore-metal-binding-witness-strings.XXXXXX")"
trap 'rm -f "$tree_file" "$artifacts_file" "$symbols_file" "$strings_file"' EXIT

cd "$repo_root"
host_target="$(rustc -vV | sed -n 's/^host: //p')"
if [[ -z "$host_target" ]]; then
  echo "error: rustc did not report its host target" >&2
  exit 1
fi

cargo tree --locked -p nuxie-ore-metal --target "$host_target" \
  -e normal,build,dev >"$tree_file"
if grep -Eiq '(^|[[:space:]│├└─])(wgpu($|[-_ @v])|naga($|[ @v])|nuxie-renderer($|[ @v])|apple-msl-catalog($|[ @v]))' "$tree_file"; then
  echo "error: the ORE Metal witness resolves a forbidden renderer dependency" >&2
  grep -Ei 'wgpu|naga|nuxie-renderer|apple-msl-catalog' "$tree_file" >&2
  exit 1
fi

MTL_DEBUG_LAYER=1 MTL_SHADER_VALIDATION=1 cargo test --locked \
  -p nuxie-ore-metal --test ore_binding_witness --target "$host_target" --no-run \
  --message-format=json-render-diagnostics >"$artifacts_file"
artifact="$({
  jq -r 'select(.reason == "compiler-artifact" and .target.name == "ore_binding_witness" and .profile.test == true) | .executable // empty' "$artifacts_file"
} | tail -n 1)"
if [[ -z "$artifact" || ! -x "$artifact" ]]; then
  echo "error: cargo did not report the ORE Metal witness executable" >&2
  exit 1
fi

MTL_DEBUG_LAYER=1 MTL_SHADER_VALIDATION=1 "$artifact" --nocapture

if ! file "$artifact" | grep -q 'Mach-O'; then
  echo "error: ORE Metal binding witness is not a Mach-O executable" >&2
  exit 1
fi
nm -j "$artifact" >"$symbols_file"
strings "$artifact" >"$strings_file"
forbidden_marker='wgpu_(core|hal|types)|(^|[^[:alpha:]])wgpu([^[:alpha:]]|$)|naga(::|_)|(^|[^[:alpha:]])naga([^[:alpha:]]|$)|wgsl'
if grep -Eiq "$forbidden_marker" "$symbols_file" "$strings_file"; then
  echo "error: ORE Metal binding witness retains a forbidden renderer marker" >&2
  grep -Ei "$forbidden_marker" "$symbols_file" "$strings_file" | head -40 >&2
  exit 1
fi
if otool -L "$artifact" | tail -n +2 | grep -Eiq 'dawn|webgpu'; then
  echo "error: ORE Metal binding witness links a Dawn/WebGPU dynamic library" >&2
  otool -L "$artifact" >&2
  exit 1
fi
for required_marker in ore_binding_witness vs_main fs_main; do
  if ! grep -Fq "$required_marker" "$strings_file"; then
    echo "error: ORE Metal binding witness is missing rooted marker: $required_marker" >&2
    exit 1
  fi
done

bytes="$(stat -f '%z' "$artifact")"
digest="$(shasum -a 256 "$artifact" | awk '{print $1}')"
echo "ORE Metal binding witness passed: bytes=$bytes sha256=$digest"
