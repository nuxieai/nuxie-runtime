#!/usr/bin/env bash
# SDK binary-size report for the rive-capi cdylib.
#
# Builds the dedicated size-optimized profile (`release-size`: opt-level "z",
# strip = "symbols", fat LTO + codegen-units=1 + panic=abort inherited from
# `release`) and prints the tracked size metrics. The default `release`
# profile — which the perf gate depends on — is never modified or built here.
#
# Usage:
#   tools/size-report.sh              # size-profile cdylib, scripting off + on
#   tools/size-report.sh --baseline   # also build the unmodified `release`
#                                      # cdylib to show the size-profile delta
#
# See docs/SIZE.md for the recorded baseline table and budget rationale.
set -euo pipefail

cd "$(dirname "$0")/.."

DEFAULT_DY="target/release-size/librive_capi.dylib"
BASELINE_DY="target/release/librive_capi.dylib"
WANT_BASELINE=0
[ "${1:-}" = "--baseline" ] && WANT_BASELINE=1

fsize() { stat -f%z "$1" 2>/dev/null || stat -c%s "$1" 2>/dev/null; }
mib()   { awk -v b="$1" 'BEGIN{printf "%.2f", b/1048576}'; }

row() { # label bytes [baseline_bytes]
  local label="$1" bytes="$2" base="${3:-}"
  if [ -n "$base" ] && [ "$base" -gt 0 ] 2>/dev/null; then
    local delta=$((bytes - base))
    local pct
    pct=$(awk -v d="$delta" -v b="$base" 'BEGIN{printf "%+.1f", (d/b)*100}')
    printf "  %-34s %10d  %6s MiB  (%+d B, %s%%)\n" "$label" "$bytes" "$(mib "$bytes")" "$delta" "$pct"
  else
    printf "  %-34s %10d  %6s MiB\n" "$label" "$bytes" "$(mib "$bytes")"
  fi
}

echo "=================================================================="
echo " rive-capi SDK cdylib size report"
echo " profile: release-size (opt-level=z, strip=symbols, fat LTO,"
echo "          codegen-units=1, panic=abort)"
echo "=================================================================="

echo
echo "Building size-profile cdylib (scripting OFF, default features)..."
cargo build --profile release-size -p rive-capi >/dev/null 2>&1
SIZE_OFF=$(fsize "$DEFAULT_DY")

echo "Building size-profile cdylib (scripting ON)..."
cargo build --profile release-size -p rive-capi --features rive/scripting >/dev/null 2>&1
SIZE_ON=$(fsize "$DEFAULT_DY")

# Restore the scripting-off artifact as the canonical release-size output.
cargo build --profile release-size -p rive-capi >/dev/null 2>&1

BASE_OFF=""
if [ "$WANT_BASELINE" = "1" ]; then
  echo "Building baseline (release, opt-level=3, unstripped) for comparison..."
  cargo build --release -p rive-capi >/dev/null 2>&1
  BASE_OFF=$(fsize "$BASELINE_DY")
fi

echo
echo "Tracked sizes:"
if [ -n "$BASE_OFF" ]; then
  row "release (opt=3, unstripped)"     "$BASE_OFF"
  row "release-size (scripting OFF)"    "$SIZE_OFF" "$BASE_OFF"
else
  row "release-size (scripting OFF)"    "$SIZE_OFF"
fi
row "release-size (scripting ON)"       "$SIZE_ON" "$SIZE_OFF"

echo
echo "Section breakdown (release-size, scripting OFF):"
if command -v size >/dev/null 2>&1; then
  size -m "$DEFAULT_DY" 2>/dev/null | grep -E "Segment __(TEXT|DATA|LINKEDIT)|Section __(text|const|cstring|eh_frame|unwind)" || true
fi
echo
echo "Metric to track alongside the perf ratio: release-size scripting-OFF"
echo "cdylib = $SIZE_OFF bytes ($(mib "$SIZE_OFF") MiB). See docs/SIZE.md."
