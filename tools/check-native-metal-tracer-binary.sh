#!/usr/bin/env bash
set -euo pipefail

if [[ "$(uname -s)" != "Darwin" ]]; then
  echo "error: native Metal tracer binary check requires macOS" >&2
  exit 1
fi

repo_root="$(cd "$(dirname "$0")/.." && pwd)"
artifact="$repo_root/target/release-size/native-metal-product-root"
dependency_tree="$(mktemp)"
trap 'rm -f "$dependency_tree"' EXIT

cargo tree \
  --manifest-path "$repo_root/Cargo.toml" \
  -p renderer-replay \
  --no-default-features \
  --features native-ore-metal \
  -e normal,build \
  --prefix none \
  --format '{p}' >"$dependency_tree"
python3 "$repo_root/tools/check-native-metal-product-dependencies.py" <"$dependency_tree"

cargo build \
  --manifest-path "$repo_root/Cargo.toml" \
  --profile release-size \
  -p renderer-replay \
  --no-default-features \
  --features native-ore-metal \
  --bin native-metal-product-root
"$artifact"

if ! file "$artifact" | grep -q 'Mach-O'; then
  echo "error: native Metal tracer root is not a Mach-O" >&2
  exit 1
fi

symbols="$(mktemp)"
strings_file="$(mktemp)"
trap 'rm -f "$dependency_tree" "$symbols" "$strings_file"' EXIT
nm -j "$artifact" >"$symbols"
strings "$artifact" >"$strings_file"

forbidden_symbols='wgpu|naga|wgsl'
# `strings` may concatenate adjacent literals without a separator. Require a
# token boundary so ordinary pairs such as `overflow` + `GPU resource` do not
# synthesize the false-positive substring `wGPU`.
forbidden_strings='(^|[^[:alnum:]_])(wgpu|naga|wgsl)([^[:alnum:]_]|$)'
if grep -Eiq "$forbidden_symbols" "$symbols" || grep -Eiq "$forbidden_strings" "$strings_file"; then
  echo "error: rooted native Metal tracer Mach-O retains a forbidden renderer dependency" >&2
  grep -Ei "$forbidden_symbols" "$symbols" | head -40 >&2 || true
  grep -Ei "$forbidden_strings" "$strings_file" | head -40 >&2 || true
  exit 1
fi

for required_product_symbol in CAMetalLayer nextDrawable presentDrawable:; do
  if ! grep -Fq "$required_product_symbol" "$strings_file"; then
    echo "error: rooted native Metal tracer does not retain Apple product-surface marker: $required_product_symbol" >&2
    exit 1
  fi
done

bytes="$(stat -f '%z' "$artifact")"
digest="$(shasum -a 256 "$artifact" | awk '{print $1}')"
echo "native Metal tracer Mach-O passed: bytes=$bytes sha256=$digest"
