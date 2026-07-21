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
# public WgpuFactory/WgpuFrame and Darwin presentation entry points,
# dead-strips the resulting closure, and applies `strip -S -x`. The result is
# a consumed-SDK link-closure proxy, not the size of the `.a` archive and not
# the callback-only Cargo cdylib.
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

for tool in "$CARGO_BIN" "$RUSTC_BIN" "$CC_BIN" git nm shasum strip size; do
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

ROOT_INVENTORY="tools/size-report-renderer-roots.txt"
RENDERER_SOURCE="crates/nuxie-renderer/src/lib.rs"
RENDERER_SURFACE_SOURCE="crates/nuxie-renderer/src/surface.rs"
ROOT_HARNESS="crates/nux-capi/src/size_report_roots.rs"
SOURCE_ROOTS="${REPORT_DIR}/renderer-public-api-from-source.txt"
HARNESS_ROOTS="${REPORT_DIR}/renderer-public-api-from-harness.txt"
ACTUAL_REVISION="$(git rev-parse --verify HEAD)"
MEASUREMENT_REVISION="${NUX_RUNTIME_SOURCE_REVISION:-$ACTUAL_REVISION}"
if [[ "$MEASUREMENT_REVISION" != "$ACTUAL_REVISION" ]]; then
  echo "size-report source revision override does not match HEAD: ${MEASUREMENT_REVISION} != ${ACTUAL_REVISION}" >&2
  exit 2
fi
if ! git diff --quiet HEAD --; then
  echo "size-report refuses a dirty tracked worktree; commit the measured sources first" >&2
  exit 2
fi
export NUX_RUNTIME_SOURCE_REVISION="$MEASUREMENT_REVISION"

extract_impl_block() { # exact impl header, source file
  local header="$1" source="$2"
  awk -v header="$header" '
    $0 == header { inside = 1; next }
    inside && $0 == "}" { exit }
    inside { print }
  ' "$source"
}

verify_renderer_root_inventory() {
  if [[ ! -s "$ROOT_INVENTORY" ]]; then
    echo "renderer public-root inventory is missing or empty: ${ROOT_INVENTORY}" >&2
    return 1
  fi
  if [[ "$(sort -u "$ROOT_INVENTORY" | wc -l | tr -d ' ')" != \
        "$(wc -l <"$ROOT_INVENTORY" | tr -d ' ')" ]]; then
    echo "renderer public-root inventory contains duplicate rows" >&2
    return 1
  fi

  {
    extract_impl_block 'impl WgpuFactory {' "$RENDERER_SOURCE" \
      | sed -En 's/^    pub ([^[:space:]]+[[:space:]]+)*fn ([a-zA-Z0-9_]+).*/inherent WgpuFactory::\2/p'
    extract_impl_block 'impl WgpuFrame {' "$RENDERER_SOURCE" \
      | sed -En 's/^    pub ([^[:space:]]+[[:space:]]+)*fn ([a-zA-Z0-9_]+).*/inherent WgpuFrame::\2/p'
    extract_impl_block 'impl Factory for WgpuFactory {' "$RENDERER_SOURCE" \
      | sed -En 's/^    ((async|const|unsafe)[[:space:]]+)*fn ([a-zA-Z0-9_]+).*/trait Factory::\3/p'
    extract_impl_block 'impl Renderer for WgpuFrame {' "$RENDERER_SOURCE" \
      | sed -En 's/^    ((async|const|unsafe)[[:space:]]+)*fn ([a-zA-Z0-9_]+).*/trait Renderer::\3/p'
    extract_impl_block 'impl ApplePresentationCompletion {' "$RENDERER_SURFACE_SOURCE" \
      | sed -En 's/^    pub ([^[:space:]]+[[:space:]]+)*fn ([a-zA-Z0-9_]+).*/inherent ApplePresentationCompletion::\2/p'
    extract_impl_block 'impl AppleSurface {' "$RENDERER_SURFACE_SOURCE" \
      | sed -En 's/^    pub ([^[:space:]]+[[:space:]]+)*fn ([a-zA-Z0-9_]+).*/inherent AppleSurface::\2/p'
  } >"$SOURCE_ROOTS"

  sed -nE 's/^[[:space:]]*[0-9_]+ => root!\("([^"]+)".*/\1/p' \
    "$ROOT_HARNESS" >"$HARNESS_ROOTS"

  if ! cmp -s "$ROOT_INVENTORY" "$SOURCE_ROOTS"; then
    echo "renderer public API drifted from the audited size-report inventory" >&2
    diff -u "$ROOT_INVENTORY" "$SOURCE_ROOTS" >&2 || true
    return 1
  fi
  if ! cmp -s "$ROOT_INVENTORY" "$HARNESS_ROOTS"; then
    echo "renderer size-report consumer does not cover the audited inventory" >&2
    diff -u "$ROOT_INVENTORY" "$HARNESS_ROOTS" >&2 || true
    return 1
  fi
}

verify_renderer_root_inventory
PUBLIC_ROOT_COUNT=$(wc -l <"$ROOT_INVENTORY" | tr -d ' ')

fsize() { stat -f%z "$1"; }
mib()   { awk -v b="$1" 'BEGIN{printf "%.2f", b/1048576}'; }
sha256() { shasum -a 256 "$1" | awk '{print $1}'; }

write_symbol_breakdown() { # unstripped dylib, output
  local artifact="$1" output="$2" estimates="${output}.all"
  nm -nm "$artifact" 2>/dev/null | awk '
    $1 ~ /^[0-9a-f]+$/ && $2 == "(__TEXT,__text)" {
      address = ("0x" $1) + 0
      if (previous != "" && address > previous) {
        print address - previous "\t" symbol
      }
      if (previous == "" || address > previous) {
        previous = address
        symbol = $NF
      }
    }
  ' >"$estimates"
  {
    echo "# Approximate __text bytes by symbol family (next-address deltas)"
    awk -F '\t' '
      {
        symbol = $2
        if (symbol ~ /nuxie_renderer/) group = "nuxie-renderer"
        else if (symbol ~ /wgpu_core|wgpu_hal|wgpu_types|naga/) group = "wgpu+naga"
        else if (symbol ~ /nuxie_runtime/) group = "nuxie-runtime"
        else if (symbol ~ /nuxie_binary/) group = "nuxie-binary"
        else if (symbol ~ /nuxie_graph/) group = "nuxie-graph"
        else if (symbol ~ /nuxie_schema/) group = "nuxie-schema"
        else if (symbol ~ /nuxie_scripting|luaur/) group = "scripting+Luau"
        else if (symbol ~ /skrifa|harfrust|read_fonts/) group = "text-shaping"
        else if (symbol ~ /taffy/) group = "taffy"
        else if (symbol ~ /alloc|core|std/) group = "Rust std/core/alloc"
        else group = "other"
        bytes[group] += $1
      }
      END { for (group in bytes) print bytes[group] "\t" group }
    ' "$estimates" | sort -nr -k1,1
    echo
    echo "# Largest approximate symbols (bytes, mangled symbol)"
    sort -nr -k1,1 "$estimates" | sed -n '1,25p'
  } >"$output"
  rm "$estimates"
}

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
  local renderer_root_symbol_file="${variant_dir}/renderer-root-symbol.txt"
  local linked_symbols="${variant_dir}/linked-symbols.txt"
  local symbol_breakdown="${variant_dir}/symbol-breakdown.txt"
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
  nm -gjU "$archive" 2>/dev/null | grep -E '_+nuxie_size_report_renderer_roots$' \
    | sort -u >"$renderer_root_symbol_file"

  local export_count renderer_root_symbol renderer_root_symbol_count
  export_count=$(wc -l <"$exports" | tr -d ' ')
  renderer_root_symbol_count=$(wc -l <"$renderer_root_symbol_file" | tr -d ' ')
  if [[ "$export_count" -lt 30 ]]; then
    echo "refusing incomplete C ABI closure: only ${export_count} nux-capi exports" >&2
    return 1
  fi
  if [[ "$renderer_root_symbol_count" != 1 ]]; then
    echo "expected exactly one compiled renderer consumer root; found ${renderer_root_symbol_count}" >&2
    return 1
  fi
  renderer_root_symbol=$(<"$renderer_root_symbol_file")
  renderer_link_args+=("-Wl,-u,${renderer_root_symbol}")

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
  if ! grep -Fq "$renderer_root_symbol" "$linked_symbols"; then
    echo "linked artifact does not retain the exact renderer consumer root" >&2
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
  write_symbol_breakdown "$unstripped" "$symbol_breakdown"

  LAST_SIZE=$(fsize "$output")
  LAST_PATH="$output"
  LAST_RENDERER_ROOTS="$PUBLIC_ROOT_COUNT"
}

echo "=================================================================="
echo " Nuxie full SDK link-closure size report"
echo " Rust target: ${RUST_TARGET} (Mach-O ${HOST_ARCH})"
echo " Source revision: ${MEASUREMENT_REVISION}"
echo " closure: all nux-capi exports + complete public Darwin renderer roots"
echo " link: dead_strip + dead_strip_dylibs; post-link strip -S -x"
echo "=================================================================="
echo

build_full_link release-size renderer-on-scripting-off size-report-roots
SIZE_OFF="$LAST_SIZE"
OFF_PATH="$LAST_PATH"
OFF_ROOTS="$LAST_RENDERER_ROOTS"

build_full_link release-size renderer-on-scripting-on size-report-roots,nuxie/scripting
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
  build_full_link release renderer-on-scripting-off size-report-roots
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
printf "Artifact SHA-256: scripting OFF=%s\n" "$(sha256 "$OFF_PATH")"
printf "Artifact SHA-256: scripting ON=%s\n" "$(sha256 "$ON_PATH")"

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
