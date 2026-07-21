#!/usr/bin/env bash
# SDK binary-size report for the post-Phase-R renderer link closure.
#
# Cargo's ordinary `nux-capi` cdylib is callback-renderer-only. Enabling its
# `apple-renderer` feature compiles nuxie-renderer and wgpu, but fat LTO removes
# almost all of that backend because no C ABI export references WgpuFactory.
# Measuring that cdylib would therefore repeat the stale pre-Phase-R metric.
#
# This report builds the nux-capi staticlib, then performs the final Darwin
# link explicitly. It retains every public nux-capi C ABI export plus the
# public WgpuFactory/WgpuFrame entry points, dead-strips the resulting closure,
# and applies `strip -S -x`. The result is a consumed-SDK link-closure proxy,
# not the size of the `.a` archive and not the callback-only Cargo cdylib.
#
# Usage:
#   tools/size-report.sh              # release-size, renderer + scripting off/on
#   tools/size-report.sh --baseline   # also link the opt-level=3 release closure
#
# See docs/SIZE.md for the artifact contract, recorded measurements, and the
# pending size-budget USER-GATE.
set -euo pipefail

cd "$(dirname "$0")/.."
export LC_ALL=C

if [[ "$(uname -s)" != "Darwin" ]]; then
  echo "size-report currently measures the Darwin SDK link closure; host is $(uname -s)" >&2
  exit 2
fi

WANT_BASELINE=0
case "${1:-}" in
  "") ;;
  --baseline) WANT_BASELINE=1 ;;
  *)
    echo "usage: tools/size-report.sh [--baseline]" >&2
    exit 2
    ;;
esac

CARGO_BIN="${CARGO:-cargo}"
RUSTC_BIN="${RUSTC:-rustc}"
CC_BIN="${CC:-clang}"
TARGET_DIR="${CARGO_TARGET_DIR:-target}"
REPORT_DIR="${TARGET_DIR}/size-report"
HOST_ARCH="$(uname -m)"
LEGACY_BUDGET_BYTES=2883584 # 2.75 MiB; informational only, never enforced here.

for tool in "$CARGO_BIN" "$RUSTC_BIN" "$CC_BIN" nm strip size; do
  if ! command -v "$tool" >/dev/null 2>&1; then
    echo "size-report requires '$tool'" >&2
    exit 2
  fi
done
RUST_TARGET="$($RUSTC_BIN -vV | sed -n 's/^host: //p')"
if [[ -z "$RUST_TARGET" || "$RUST_TARGET" != *-apple-darwin ]]; then
  echo "size-report could not resolve a Darwin Rust host target" >&2
  exit 2
fi

mkdir -p "$REPORT_DIR"

fsize() { stat -f%z "$1"; }
mib()   { awk -v b="$1" 'BEGIN{printf "%.2f", b/1048576}'; }

row() { # label bytes [baseline_bytes]
  local label="$1" bytes="$2" base="${3:-}"
  if [[ -n "$base" && "$base" -gt 0 ]]; then
    local delta=$((bytes - base))
    local pct
    pct=$(awk -v d="$delta" -v b="$base" 'BEGIN{printf "%+.1f", (d/b)*100}')
    printf "  %-47s %10d B  %6s MiB  (%+d B, %s%%)\n" \
      "$label" "$bytes" "$(mib "$bytes")" "$delta" "$pct"
  else
    printf "  %-47s %10d B  %6s MiB\n" "$label" "$bytes" "$(mib "$bytes")"
  fi
}

# Outputs from the most recent build_full_link invocation.
LAST_SIZE=""
LAST_PATH=""
LAST_RENDERER_ROOTS=""

build_full_link() { # profile variant features
  local profile="$1" variant="$2" features="$3"
  local profile_dir="$profile"
  local variant_dir="${REPORT_DIR}/${profile}-${variant}"
  local archive="${TARGET_DIR}/${profile_dir}/libnux_capi.a"
  local cargo_dylib="${TARGET_DIR}/${profile_dir}/libnux_capi.dylib"
  local output="${variant_dir}/libnux_capi_full.dylib"
  local unstripped="${variant_dir}/libnux_capi_full.unstripped.dylib"
  local exports="${variant_dir}/exports.txt"
  local actual_exports="${variant_dir}/actual-exports.txt"
  local renderer_roots="${variant_dir}/renderer-roots.txt"
  local linked_symbols="${variant_dir}/linked-symbols.txt"
  local dependency_tree="${variant_dir}/cargo-tree.txt"
  local build_log="${variant_dir}/cargo-build.log"
  local cargo_profile_args=()
  local renderer_link_args=()

  mkdir -p "$variant_dir"
  if [[ "$profile" == "release" ]]; then
    cargo_profile_args=(--release)
  else
    cargo_profile_args=(--profile "$profile")
  fi

  echo "Building ${profile} full SDK closure (${variant//-/ }, features=${features})..."
  if ! "$CARGO_BIN" build --locked -p nux-capi --no-default-features \
      --features "$features" "${cargo_profile_args[@]}" >"$build_log" 2>&1; then
    sed -n '1,240p' "$build_log" >&2
    return 1
  fi

  if [[ ! -f "$archive" || ! -f "$cargo_dylib" ]]; then
    echo "cargo did not produce the expected nux-capi artifacts for ${profile}" >&2
    return 1
  fi

  "$CARGO_BIN" tree --locked -p nux-capi --no-default-features \
    --features "$features" --edges normal --prefix none >"$dependency_tree"
  if ! grep -Eq '^nuxie-renderer v' "$dependency_tree"; then
    echo "renderer dependency is absent from ${variant} feature graph" >&2
    return 1
  fi
  if ! grep -Eq '^wgpu v30\.0\.0 \(.*/vendor/wgpu-30\.0\.0\)$' "$dependency_tree"; then
    echo "vendored wgpu 30.0.0 is absent from ${variant} feature graph" >&2
    return 1
  fi
  if [[ "$features" == *nuxie/scripting* ]]; then
    if ! grep -Eq '^nuxie-scripting v' "$dependency_tree"; then
      echo "scripting dependency is absent from ${variant} feature graph" >&2
      return 1
    fi
  elif grep -Eq '^nuxie-scripting v' "$dependency_tree"; then
    echo "scripting dependency leaked into ${variant} feature graph" >&2
    return 1
  fi

  nm -gjU "$cargo_dylib" | grep '^_nux_' | sort -u >"$exports"
  nm -g "$archive" 2>/dev/null | sed 's/.* //' \
    | grep -E 'nuxie_renderer(11WgpuFactory|9WgpuFrame)|LT.*nuxie_renderer.*Wgpu(Factory|Frame)' \
    | sort -u >"$renderer_roots"

  local export_count renderer_root_count
  export_count=$(wc -l <"$exports" | tr -d ' ')
  renderer_root_count=$(wc -l <"$renderer_roots" | tr -d ' ')
  if [[ "$export_count" -lt 30 ]]; then
    echo "refusing incomplete C ABI closure: only ${export_count} nux-capi exports" >&2
    return 1
  fi
  if [[ "$renderer_root_count" -lt 20 ]]; then
    echo "refusing incomplete renderer closure: only ${renderer_root_count} renderer roots" >&2
    return 1
  fi

  while IFS= read -r symbol; do
    renderer_link_args+=("-Wl,-u,${symbol}")
  done <"$renderer_roots"

  "$CC_BIN" -dynamiclib -arch "$HOST_ARCH" -o "$unstripped" \
    -Wl,-dead_strip -Wl,-dead_strip_dylibs \
    -Wl,-force_load,"$archive" \
    -Wl,-exported_symbols_list,"$exports" \
    -Wl,-install_name,@rpath/libnux_capi_full.dylib \
    "${renderer_link_args[@]}" \
    -framework CoreFoundation \
    -framework CoreGraphics \
    -framework ImageIO \
    -framework QuartzCore \
    -framework Metal \
    -framework Foundation \
    -framework Security \
    -liconv

  nm -gjU "$unstripped" | grep '^_nux_' | sort -u >"$actual_exports"
  if ! cmp -s "$exports" "$actual_exports"; then
    echo "linked C ABI export set differs from the Cargo nux-capi artifact" >&2
    diff -u "$exports" "$actual_exports" >&2 || true
    return 1
  fi
  nm "$unstripped" >"$linked_symbols"
  if ! grep -Eq 'nuxie_renderer.*WgpuFactory.*new_with_mode' "$linked_symbols"; then
    echo "linked artifact does not retain WgpuFactory::new_with_mode" >&2
    return 1
  fi
  if ! grep -Eq 'wgpu_core' "$linked_symbols"; then
    echo "linked artifact does not retain the vendored wgpu core" >&2
    return 1
  fi
  if [[ "$features" == *nuxie/scripting* ]]; then
    if ! grep -Eq 'nuxie_scripting' "$linked_symbols" || ! grep -Eq 'luaur_vm' "$linked_symbols"; then
      echo "linked scripting-on artifact does not retain nuxie-scripting + luaur-vm" >&2
      return 1
    fi
  elif grep -Eq 'nuxie_scripting|luaur_vm' "$linked_symbols"; then
    echo "linked scripting-off artifact unexpectedly retains the scripting engine" >&2
    return 1
  fi

  cp "$unstripped" "$output"
  strip -S -x "$output"

  LAST_SIZE=$(fsize "$output")
  LAST_PATH="$output"
  LAST_RENDERER_ROOTS="$renderer_root_count"
}

echo "=================================================================="
echo " Nuxie full SDK link-closure size report"
echo " Rust target: ${RUST_TARGET} (Mach-O ${HOST_ARCH})"
echo " closure: all nux-capi exports + public WgpuFactory/WgpuFrame roots"
echo " link: dead_strip + dead_strip_dylibs; post-link strip -S -x"
echo "=================================================================="
echo

build_full_link release-size renderer-on-scripting-off apple-renderer
SIZE_OFF="$LAST_SIZE"
OFF_PATH="$LAST_PATH"
OFF_ROOTS="$LAST_RENDERER_ROOTS"

build_full_link release-size renderer-on-scripting-on apple-renderer,nuxie/scripting
SIZE_ON="$LAST_SIZE"
ON_PATH="$LAST_PATH"
ON_ROOTS="$LAST_RENDERER_ROOTS"

# Restore the renderer-on/scripting-off Cargo output as the canonical
# release-size artifact. The measured closure above is already copied aside.
RESTORE_LOG="${REPORT_DIR}/restore-release-size-off.log"
if ! "$CARGO_BIN" build --locked -p nux-capi --no-default-features \
    --features apple-renderer --profile release-size >"$RESTORE_LOG" 2>&1; then
  sed -n '1,240p' "$RESTORE_LOG" >&2
  exit 1
fi

BASE_OFF=""
BASE_PATH=""
if [[ "$WANT_BASELINE" == "1" ]]; then
  build_full_link release renderer-on-scripting-off apple-renderer
  BASE_OFF="$LAST_SIZE"
  BASE_PATH="$LAST_PATH"
fi

echo
echo "Tracked stripped link-closure sizes:"
if [[ -n "$BASE_OFF" ]]; then
  row "release opt=3, renderer ON, scripting OFF" "$BASE_OFF"
  row "release-size opt=z, renderer ON, scripting OFF" "$SIZE_OFF" "$BASE_OFF"
else
  row "release-size opt=z, renderer ON, scripting OFF" "$SIZE_OFF"
fi
row "release-size opt=z, renderer ON, scripting ON" "$SIZE_ON" "$SIZE_OFF"

LEGACY_DELTA=$((SIZE_OFF - LEGACY_BUDGET_BYTES))
LEGACY_PCT=$(awk -v d="$LEGACY_DELTA" -v b="$LEGACY_BUDGET_BYTES" \
  'BEGIN{printf "%+.1f", (d/b)*100}')
echo
printf "Historical 2.75 MiB budget delta (informational; not enforced): %+d B (%s%%)\n" \
  "$LEGACY_DELTA" "$LEGACY_PCT"
printf "Retained renderer roots: scripting OFF=%s, ON=%s\n" "$OFF_ROOTS" "$ON_ROOTS"

echo
echo "Artifacts:"
echo "  scripting OFF: $OFF_PATH"
echo "  scripting ON:  $ON_PATH"
if [[ -n "$BASE_PATH" ]]; then
  echo "  release base:  $BASE_PATH"
fi

echo
echo "Section breakdown (release-size, renderer ON, scripting OFF):"
size -m "$OFF_PATH" 2>/dev/null \
  | grep -E 'Segment __(TEXT|DATA|LINKEDIT)|Section __(text|const|cstring|eh_frame|unwind)' \
  || true

echo
echo "Metric awaiting the #B-3 USER-GATE: release-size renderer-ON scripting-OFF"
echo "link closure = $SIZE_OFF bytes ($(mib "$SIZE_OFF") MiB). See docs/SIZE.md."
