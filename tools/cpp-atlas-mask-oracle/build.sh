#!/bin/bash
set -euo pipefail

script_dir="$(cd "$(dirname "$0")" && pwd)"
runtime="${RIVE_RUNTIME_DIR:-/Users/levi/dev/oss/rive-runtime}"
patch="$script_dir/runtime.patch"
dawn_patch="$script_dir/dawn-xcode26.patch"
dawn_dir="$runtime/renderer/dependencies/dawn"
injected_dir="$runtime/renderer/atlas_mask_oracle"
build_out="${RIVE_ATLAS_MASK_BUILD_OUT:-out/cpp-atlas-mask-oracle}"
jobs="${RIVE_ATLAS_MASK_JOBS:-2}"
output="${RIVE_ATLAS_MASK_OUTPUT:-$script_dir/out/atlas-mask.r16f}"
inputs_output="${RIVE_ATLAS_INPUT_OUTPUT:-$script_dir/out/atlas-inputs.bin}"
blit_output="${RIVE_ATLAS_BLIT_OUTPUT:-$script_dir/out/atlas-blit.rgba}"
clipped_blit_output="${RIVE_ATLAS_CLIPPED_BLIT_OUTPUT:-$script_dir/out/atlas-clipped-blit.rgba}"
path_clipped_blit_output="${RIVE_ATLAS_PATH_CLIPPED_BLIT_OUTPUT:-$script_dir/out/atlas-path-clipped-blit.rgba}"
changing_path_clipped_blit_output="${RIVE_ATLAS_CHANGING_PATH_CLIPPED_BLIT_OUTPUT:-$script_dir/out/atlas-changing-path-clipped-blit.rgba}"
nested_path_clipped_blit_output="${RIVE_ATLAS_NESTED_PATH_CLIPPED_BLIT_OUTPUT:-$script_dir/out/atlas-nested-path-clipped-blit.rgba}"
nested_evenodd_path_clipped_blit_output="${RIVE_ATLAS_NESTED_EVENODD_PATH_CLIPPED_BLIT_OUTPUT:-$script_dir/out/atlas-nested-evenodd-path-clipped-blit.rgba}"
nested_clockwise_path_clipped_blit_output="${RIVE_ATLAS_NESTED_CLOCKWISE_PATH_CLIPPED_BLIT_OUTPUT:-$script_dir/out/atlas-nested-clockwise-path-clipped-blit.rgba}"
advanced_blend_blit_output="${RIVE_ATLAS_ADVANCED_BLEND_BLIT_OUTPUT:-$script_dir/out/atlas-advanced-blend-blit.rgba}"
atomic_advanced_blend_output="${RIVE_ATOMIC_ADVANCED_BLEND_OUTPUT:-$script_dir/out/atomic-advanced-blend.rgba}"
atomic_colorburn_pair_output="${RIVE_ATOMIC_COLORBURN_PAIR_OUTPUT:-$script_dir/out/atomic-colorburn-pair.rgba}"
atomic_colorburn_pair_color_output="${RIVE_ATOMIC_COLORBURN_PAIR_COLOR_OUTPUT:-$script_dir/out/atomic-colorburn-pair.color}"
atomic_colorburn_pair_coverage_output="${RIVE_ATOMIC_COLORBURN_PAIR_COVERAGE_OUTPUT:-$script_dir/out/atomic-colorburn-pair.coverage}"
atomic_interleavedfeather_full_output="${RIVE_ATOMIC_INTERLEAVEDFEATHER_FULL_OUTPUT:-$script_dir/out/atomic-interleavedfeather-full.rgba}"
atomic_interleavedfeather_full_provenance="${RIVE_ATOMIC_INTERLEAVEDFEATHER_FULL_PROVENANCE:-$script_dir/out/atomic-interleavedfeather-full.provenance}"
atomic_dstreadshuffle_full_output="${RIVE_ATOMIC_DSTREADSHUFFLE_FULL_OUTPUT:-$script_dir/out/atomic-dstreadshuffle-full.rgba}"
atomic_dstreadshuffle_full_provenance="${RIVE_ATOMIC_DSTREADSHUFFLE_FULL_PROVENANCE:-$script_dir/out/atomic-dstreadshuffle-full.provenance}"
atomic_dstreadshuffle_srcover_output="${RIVE_ATOMIC_DSTREADSHUFFLE_SRCOVER_OUTPUT:-$script_dir/out/atomic-dstreadshuffle-srcover.rgba}"
atomic_dstreadshuffle_srcover_provenance="${RIVE_ATOMIC_DSTREADSHUFFLE_SRCOVER_PROVENANCE:-$script_dir/out/atomic-dstreadshuffle-srcover.provenance}"
atomic_spotify_kids_app_icon_full_output="${RIVE_ATOMIC_SPOTIFY_KIDS_APP_ICON_FULL_OUTPUT:-$script_dir/out/atomic-spotify-kids-app-icon-full.rgba}"
atomic_spotify_kids_app_icon_full_coverage="${RIVE_ATOMIC_SPOTIFY_KIDS_APP_ICON_FULL_COVERAGE_OUTPUT:-$script_dir/out/atomic-spotify-kids-app-icon-full.coverage}"
atomic_spotify_kids_app_icon_full_clip="${RIVE_ATOMIC_SPOTIFY_KIDS_APP_ICON_FULL_CLIP_OUTPUT:-$script_dir/out/atomic-spotify-kids-app-icon-full.clip}"
atomic_spotify_kids_app_icon_full_provenance="${RIVE_ATOMIC_SPOTIFY_KIDS_APP_ICON_FULL_PROVENANCE:-$script_dir/out/atomic-spotify-kids-app-icon-full.provenance}"
atomic_spotify_kids_app_icon_full_schedule=""
fill_output="${RIVE_ATLAS_FILL_MASK_OUTPUT:-$script_dir/out/atlas-fill-mask.r16f}"
fill_inputs_output="${RIVE_ATLAS_FILL_INPUT_OUTPUT:-$script_dir/out/atlas-fill-inputs.bin}"
fill_blit_output="${RIVE_ATLAS_FILL_BLIT_OUTPUT:-$script_dir/out/atlas-fill-blit.rgba}"
cusp_output="${RIVE_ATLAS_CUSP_MASK_OUTPUT:-$script_dir/out/atlas-cusp-mask.r16f}"
cusp_inputs_output="${RIVE_ATLAS_CUSP_INPUT_OUTPUT:-$script_dir/out/atlas-cusp-inputs.bin}"
cusp_blit_output="${RIVE_ATLAS_CUSP_BLIT_OUTPUT:-$script_dir/out/atlas-cusp-blit.rgba}"
large_feather_cusp_output="${RIVE_ATLAS_LARGE_FEATHER_CUSP_MASK_OUTPUT:-$script_dir/out/atlas-large-feather-cusp-mask.r16f}"
large_feather_cusp_inputs_output="${RIVE_ATLAS_LARGE_FEATHER_CUSP_INPUT_OUTPUT:-$script_dir/out/atlas-large-feather-cusp-inputs.bin}"
large_feather_cusp_blit_output="${RIVE_ATLAS_LARGE_FEATHER_CUSP_BLIT_OUTPUT:-$script_dir/out/atlas-large-feather-cusp-blit.rgba}"
large_feather_cusp_placement_output="${RIVE_ATLAS_LARGE_FEATHER_CUSP_PLACEMENT_OUTPUT:-$script_dir/out/atlas-large-feather-cusp-placement.bin}"
large_feather_shapes_cusp_output="${RIVE_ATLAS_LARGE_FEATHER_SHAPES_CUSP_MASK_OUTPUT:-$script_dir/out/atlas-large-feather-shapes-cusp-mask.r16f}"
large_feather_shapes_cusp_inputs_output="${RIVE_ATLAS_LARGE_FEATHER_SHAPES_CUSP_INPUT_OUTPUT:-$script_dir/out/atlas-large-feather-shapes-cusp-inputs.bin}"
large_feather_shapes_cusp_blit_output="${RIVE_ATLAS_LARGE_FEATHER_SHAPES_CUSP_BLIT_OUTPUT:-$script_dir/out/atlas-large-feather-shapes-cusp-blit.rgba}"
large_feather_shapes_cusp_placement_output="${RIVE_ATLAS_LARGE_FEATHER_SHAPES_CUSP_PLACEMENT_OUTPUT:-$script_dir/out/atlas-large-feather-shapes-cusp-placement.bin}"
empty_stroke_output="${RIVE_ATLAS_EMPTY_STROKE_MASK_OUTPUT:-$script_dir/out/atlas-empty-stroke-mask.r16f}"
empty_stroke_inputs_output="${RIVE_ATLAS_EMPTY_STROKE_INPUT_OUTPUT:-$script_dir/out/atlas-empty-stroke-inputs.bin}"
empty_stroke_blit_output="${RIVE_ATLAS_EMPTY_STROKE_BLIT_OUTPUT:-$script_dir/out/atlas-empty-stroke-blit.rgba}"
empty_stroke_overlap_blit_output="${RIVE_ATLAS_EMPTY_STROKE_OVERLAP_BLIT_OUTPUT:-$script_dir/out/atlas-empty-stroke-overlap-blit.rgba}"
softened_cusp_output="${RIVE_SOFTENED_CUSP_OUTPUT:-$script_dir/out/softened-cusp.bin}"
direct_cusp_inputs_output="${RIVE_DIRECT_CUSP_INPUT_OUTPUT:-$script_dir/out/direct-cusp-inputs.bin}"
direct_cusp_blit_output="${RIVE_DIRECT_CUSP_BLIT_OUTPUT:-$script_dir/out/direct-cusp-blit.rgba}"
direct_cusp_coverage_output="${RIVE_DIRECT_CUSP_COVERAGE_OUTPUT:-$script_dir/out/direct-cusp-coverage.bin}"
direct_polyshark_inputs_output="${RIVE_DIRECT_POLYSHARK_INPUT_OUTPUT:-$script_dir/out/direct-polyshark-inputs.bin}"
direct_grid_inputs_output="${RIVE_DIRECT_GRID_INPUT_OUTPUT:-$script_dir/out/direct-grid-inputs.bin}"
direct_flower_inputs_output="${RIVE_DIRECT_FLOWER_INPUT_OUTPUT:-$script_dir/out/direct-flower-inputs.bin}"
direct_bad_skin_inputs_output="${RIVE_DIRECT_BAD_SKIN_INPUT_OUTPUT:-$script_dir/out/direct-bad-skin-inputs.bin}"
direct_strokes_round_inputs_output="${RIVE_DIRECT_STROKES_ROUND_INPUT_OUTPUT:-$script_dir/out/direct-strokes-round-inputs.bin}"
direct_strokes_round_blit_output="${RIVE_DIRECT_STROKES_ROUND_BLIT_OUTPUT:-$script_dir/out/direct-strokes-round-blit.rgba}"
direct_strokes_round_spans_output="${RIVE_DIRECT_STROKES_ROUND_SPANS_OUTPUT:-$script_dir/out/direct-strokes-round-spans.bin}"
direct_rawtext_inputs_output="${RIVE_DIRECT_RAWTEXT_INPUT_OUTPUT:-$script_dir/out/direct-rawtext-inputs.bin}"
direct_rawtext_blit_output="${RIVE_DIRECT_RAWTEXT_BLIT_OUTPUT:-$script_dir/out/direct-rawtext-blit.rgba}"
direct_rawtext_spans_output="${RIVE_DIRECT_RAWTEXT_SPANS_OUTPUT:-$script_dir/out/direct-rawtext-spans.bin}"
polyshark_generator="$script_dir/generate_polyshark_stream_path.py"
polyshark_stream="$script_dir/../../fixtures/renderer/streams/gm/feather_polyshapes.rive-stream"
rawtext_generator="$script_dir/generate_rawtext_stream_path.py"
rawtext_stream="$script_dir/../../fixtures/renderer/streams/gm/rawtext.rive-stream"
colorburn_pair_generator="$script_dir/generate_interleaved_colorburn_pair_path.py"
colorburn_pair_stream="$script_dir/../../fixtures/renderer/streams/gm/interleavedfeather.rive-stream"
path_stream_generator="$script_dir/generate_path_stream_replay.py"
msaa_reference_generator="$script_dir/generate_msaa_reference_registry.py"
msaa_reference_manifest="$script_dir/msaa-reference-corpus.toml"
interleavedfeather_stream="$script_dir/../../fixtures/renderer/streams/gm/interleavedfeather.rive-stream"
dstreadshuffle_stream="$script_dir/../../fixtures/renderer/streams/gm/dstreadshuffle.rive-stream"
spotify_kids_app_icon_stream="$script_dir/../../fixtures/renderer/streams/riv/spotify_kids_app_icon.rive-stream"
hunter_x_stream="$script_dir/../../fixtures/renderer/streams/riv/hunter_x_demo.rive-stream"
ninja_bin="${RIVE_ATLAS_MASK_NINJA:-$dawn_dir/third_party/ninja/ninja}"
case "$(uname -s)" in
    Darwin) default_gn="$dawn_dir/buildtools/mac/gn" ;;
    Linux) default_gn="$dawn_dir/buildtools/linux64/gn" ;;
    *) default_gn="$script_dir/../../target/depot_tools/gn" ;;
esac
gn_bin="${RIVE_ATLAS_MASK_GN:-$default_gn}"
naga_bin="${RIVE_ATLAS_MASK_NAGA:-$HOME/.cargo/bin/naga}"
expected_naga_version="30.0.0"
expected_interleavedfeather_sha256="8868c228229b6708e4e46c947177bfd982c6e7a60ee9b1c3a7da43a7ec0ee17a"
expected_dstreadshuffle_sha256="0e08ecd19e6a9e1f89f3ae2291181cea3513edf5bbe8cadcd3e1e10a0c33f195"
expected_spotify_kids_app_icon_sha256="1c230de80579ddfc9953541ec3311c981e8f53d94c4d023c5429635186ebbd88"
expected_hunter_x_sha256="3544c61ac8a12790bc1b865074c5ea140caa05502a03f9bca7b5ca8e2f731b5f"
expected_runtime_revision="7c778d13c5d903b3b74eec1dd6bb68a811dea5f2"
expected_dawn_revision="211333b2e3e429c3508f25c81c547f602adf448c"

xcode_major() {
    xcodebuild -version 2>/dev/null | awk '/Xcode/ { split($2, version, "."); print version[1]; exit }'
}

sha256_file() {
    python3 -c 'import hashlib, sys; print(hashlib.sha256(open(sys.argv[1], "rb").read()).hexdigest())' "$1"
}

validate_spotify_artifacts() {
    python3 - "$atomic_spotify_kids_app_icon_full_output" \
        "$atomic_spotify_kids_app_icon_full_coverage" \
        "$atomic_spotify_kids_app_icon_full_clip" \
        "$atomic_spotify_kids_app_icon_full_schedule" <<'PY'
import pathlib
import struct
import sys

rgba_path, coverage_path, clip_path, schedule_path = map(pathlib.Path, sys.argv[1:])

def u32s(data, offset, count):
    return struct.unpack_from(f"<{count}I", data, offset)

rgba = rgba_path.read_bytes()
if rgba[:8] != b"RIVEABL\0" or u32s(rgba, 8, 3) != (1, 1024, 1436):
    raise SystemExit("Spotify RIVEABL header must be v1 1024x1436")
if len(rgba) != 20 + 1024 * 1436 * 4:
    raise SystemExit("Spotify RIVEABL payload size is not exact")

for name, path, magic in (
    ("coverage", coverage_path, b"RIVEAPC\0"),
    ("clip", clip_path, b"RIVEACL\0"),
):
    data = path.read_bytes()
    if data[:8] != magic or u32s(data, 8, 4) != (1, 1024, 1440, 1024 * 1440):
        raise SystemExit(f"Spotify {name} header must be v1 1024x1440 with every native word")
    if len(data) != 24 + 1024 * 1440 * 4:
        raise SystemExit(f"Spotify {name} payload size is not exact")

schedule = schedule_path.read_bytes()
if schedule[:8] != b"RIVEASL\0" or len(schedule) < 24:
    raise SystemExit("Spotify schedule artifact is missing its v1 header")
version, interlock_mode, fixed_function_color_output, count = u32s(schedule, 8, 4)
if version != 1 or interlock_mode != 1 or fixed_function_color_output != 1 or count != 24:
    raise SystemExit("Spotify schedule must pin atomic interlock and fixed-function color output")
if len(schedule) != 24 + count * 24:
    raise SystemExit("Spotify schedule payload size is not exact")
PY
}

needs_xcode26_patch() {
    [[ "$(uname -s)" == "Darwin" ]] && [[ "$(xcode_major)" =~ ^[0-9]+$ ]] && (( $(xcode_major) >= 26 ))
}

dawn_patch_needed() {
    grep -Fqx '      cflags = [ "-fvisibility-global-new-delete=force-hidden" ]' \
        "$dawn_dir/third_party/partition_alloc/src/partition_alloc/BUILD.gn"
}

dawn_target_exists() {
    (
        cd "$dawn_dir"
        "$ninja_bin" -C out/release -t targets all |
            awk -F: -v target="$1" '$1 == target { found = 1 } END { exit !found }'
    )
}

dawn_target_produces() {
    (
        cd "$dawn_dir"
        "$ninja_bin" -C out/release -t query "$1" | grep -Fq "$2"
    )
}

dawn_archive_is_declared() {
    grep -Fq "$1" "$dawn_dir/out/release/build.ninja"
}

dawn_args="$dawn_dir/out/release/args.gn"
dawn_args_snapshot=""
dawn_args_snapshot_candidate=""

configure_xcode26_dawn_args() {
    if ! needs_xcode26_patch; then
        return
    fi
    if grep -Eq '^treat_warnings_as_errors[[:space:]]*=' "$dawn_args"; then
        if ! grep -Eq '^treat_warnings_as_errors[[:space:]]*=[[:space:]]*false[[:space:]]*$' "$dawn_args"; then
            echo "Dawn args explicitly set treat_warnings_as_errors; refusing to override it" >&2
            return 2
        fi
    else
        printf '\n# cpp-atlas-mask-oracle: Xcode 26 promotes legacy unsafe-buffer warnings.\ntreat_warnings_as_errors=false\n' >> "$dawn_args"
    fi
    grep -Eq '^treat_warnings_as_errors[[:space:]]*=[[:space:]]*false[[:space:]]*$' "$dawn_args"

    if grep -Eq '^use_lld[[:space:]]*=' "$dawn_args"; then
        if ! grep -Eq '^use_lld[[:space:]]*=[[:space:]]*false[[:space:]]*$' "$dawn_args"; then
            echo "Dawn args explicitly set use_lld; refusing to override it" >&2
            return 2
        fi
    else
        printf '# cpp-atlas-mask-oracle: Apple ld cannot consume Dawn thin archives.\nuse_lld=false\n' >> "$dawn_args"
    fi
}

usage() {
    echo "usage: tools/cpp-atlas-mask-oracle/build.sh [--preflight|--build-only]" >&2
}

preflight() {
    local missing=0
    local naga_output
    local naga_version
    if [[ "${run_mode:-full}" == build-only ]]; then
        RIVE_MSAA_REFERENCE_BOOTSTRAP=1 python3 "$script_dir/format_test.py"
    else
        python3 "$script_dir/format_test.py"
    fi
    PYTHONDONTWRITEBYTECODE=1 python3 "$script_dir/test_capture_msaa_references.py"
    python3 "$polyshark_generator" --stream "$polyshark_stream" --check
    python3 "$rawtext_generator" --stream "$rawtext_stream" --check
    python3 "$colorburn_pair_generator" --stream "$colorburn_pair_stream" --check
    python3 "$msaa_reference_generator" \
        --manifest "$msaa_reference_manifest" \
        --repo-root "$script_dir/../.." \
        --runtime-revision "$expected_runtime_revision" \
        --dawn-revision "$expected_dawn_revision" \
        --check
    python3 "$path_stream_generator" \
        --stream "$interleavedfeather_stream" \
        --expected-sha256 "$expected_interleavedfeather_sha256" \
        --expected-source gm:interleavedfeather \
        --expected-width 1000 --expected-height 1000 \
        --expected-count save=301 --expected-count restore=301 \
        --expected-count transform=900 --expected-count makeEmptyRenderPath=1 \
        --expected-count makeRenderPaint=1 --expected-count drawPath=451 \
        --function replayInterleavedFeatherFull --check
    python3 "$path_stream_generator" \
        --profile riv \
        --stream "$spotify_kids_app_icon_stream" \
        --expected-sha256 "$expected_spotify_kids_app_icon_sha256" \
        --expected-source-suffix tests/unit_tests/assets/spotify_kids_app_icon.riv \
        --expected-artboard artboard --expected-scene "State Machine 1" \
        --expected-width 1024 --expected-height 1436 \
        --expected-sample-seconds 0 \
        --expected-count save=18 --expected-count restore=18 \
        --expected-count transform=15 --expected-count makeEmptyRenderPath=20 \
        --expected-count makeRenderPaint=48 --expected-count clipPath=6 \
        --expected-count drawPath=14 \
        --function replaySpotifyKidsAppIconFull --check
    for command in cmp git make mktemp premake5 python3; do
        if ! command -v "$command" >/dev/null; then
            echo "missing required tool: $command" >&2
            missing=1
        fi
    done
    if [[ ! -x "$naga_bin" ]]; then
        echo "missing required tool: Naga $expected_naga_version ($naga_bin)" >&2
        missing=1
    elif [[ "$(basename "$naga_bin")" != "naga" ]]; then
        echo "RIVE_ATLAS_MASK_NAGA must name an executable named naga: $naga_bin" >&2
        missing=1
    elif ! naga_output="$("$naga_bin" --version 2>&1)"; then
        echo "could not query Naga version at $naga_bin: $naga_output" >&2
        missing=1
    else
        naga_version="$(awk 'NR == 1 { print $NF }' <<< "$naga_output")"
        if [[ "$naga_version" != "$expected_naga_version" ]]; then
            echo "unsupported Naga version at $naga_bin: expected $expected_naga_version, got ${naga_version:-unknown}" >&2
            missing=1
        else
            echo "Naga: $naga_bin ($naga_version)"
        fi
    fi
    if [[ ! -d "$runtime/.git" ]]; then
        echo "missing RIVE_RUNTIME_DIR git checkout: $runtime" >&2
        return 2
    fi
    if [[ "$(git -C "$runtime" rev-parse HEAD)" != "$expected_runtime_revision" ]]; then
        echo "Rive runtime revision drifted; review and update the full-stream oracle pin" >&2
        return 2
    fi
    if ! git -C "$runtime" diff --quiet || ! git -C "$runtime" diff --cached --quiet; then
        echo "Rive runtime has tracked changes; refusing to label a non-pinned oracle artifact" >&2
        return 2
    fi
    if [[ "$(git -C "$dawn_dir" rev-parse HEAD)" != "$expected_dawn_revision" ]]; then
        echo "Dawn revision drifted; review and update the full-stream oracle pin" >&2
        return 2
    fi
    if ! git -C "$dawn_dir" diff --quiet || ! git -C "$dawn_dir" diff --cached --quiet; then
        echo "Dawn has tracked changes; refusing to label a non-pinned oracle artifact" >&2
        return 2
    fi
    if ! git -C "$runtime" apply --check "$patch"; then
        echo "runtime patch does not apply cleanly: $runtime" >&2
        return 2
    fi
    if [[ ! -f "$dawn_dir/include/webgpu/webgpu.h" ]]; then
        echo "missing Dawn prerequisite: renderer/dependencies/dawn/include/webgpu/webgpu.h" >&2
        missing=1
    fi
    if [[ ! -x "$ninja_bin" ]]; then
        echo "missing required tool: pinned Dawn ninja ($ninja_bin)" >&2
        missing=1
    fi
    if [[ ! -x "$gn_bin" ]]; then
        echo "missing required tool: pinned Dawn gn ($gn_bin)" >&2
        missing=1
    fi
    for target in webgpu_dawn_static cpp proc_static; do
        if ! dawn_target_exists "$target"; then
            echo "missing pinned Dawn Ninja target: $target" >&2
            missing=1
        fi
    done
    if ! dawn_target_produces webgpu_dawn_static 'obj/src/dawn/native/libwebgpu_dawn.a'; then
        echo "webgpu_dawn_static does not produce libwebgpu_dawn.a" >&2
        missing=1
    fi
    if ! dawn_target_produces proc_static 'obj/src/dawn/libdawn_proc_static.a'; then
        echo "proc_static does not produce libdawn_proc_static.a" >&2
        missing=1
    fi
    for library in \
        'obj/src/dawn/native/libwebgpu_dawn.a' \
        'obj/src/dawn/native/libdawn_native_static.a' \
        'obj/src/dawn/libdawn_proc_static.a'; do
        if ! dawn_archive_is_declared "$library"; then
            echo "missing pinned Dawn archive rule: $library" >&2
            missing=1
        fi
    done
    if needs_xcode26_patch; then
        if dawn_patch_needed; then
            if ! git -C "$dawn_dir" apply --check "$dawn_patch"; then
                echo "Dawn Xcode-26 compatibility patch does not apply cleanly" >&2
                return 2
            fi
        else
            echo "Dawn Xcode-26 compatibility patch: already present"
        fi
        if grep -Eq '^treat_warnings_as_errors[[:space:]]*=' "$dawn_args" &&
            ! grep -Eq '^treat_warnings_as_errors[[:space:]]*=[[:space:]]*false[[:space:]]*$' "$dawn_args"; then
            echo "Dawn args explicitly enable warnings-as-errors; refusing to override them" >&2
            return 2
        fi
        if grep -Eq '^use_lld[[:space:]]*=' "$dawn_args" &&
            ! grep -Eq '^use_lld[[:space:]]*=[[:space:]]*false[[:space:]]*$' "$dawn_args"; then
            echo "Dawn args explicitly enable lld; refusing to produce Apple-ld-incompatible thin archives" >&2
            return 2
        fi
        echo "Dawn Xcode-26 compatibility: treat_warnings_as_errors=false and use_lld=false will be verified before GN generation"
    fi
    if (( missing )); then
        echo "preflight: BLOCKED (patch applies; Dawn build/runtime execution was not attempted)" >&2
        return 3
    fi
    echo "preflight: READY"
}

run_mode=full
case "${1:-}" in
    --preflight)
        preflight
        exit $?
        ;;
    --build-only)
        run_mode=build-only
        ;;
    "")
        ;;
    *)
        usage
        exit 2
        ;;
esac

preflight
export PATH="$(dirname "$naga_bin"):$PATH"
applied=0
dawn_patch_applied=0
cleanup() {
    local status=$?
    trap - EXIT INT TERM
    set +e
    if [[ -n "$dawn_args_snapshot" ]]; then
        if [[ ! -f "$dawn_args_snapshot" ]]; then
            echo "Dawn args snapshot disappeared before cleanup: $dawn_args_snapshot" >&2
            status=1
        elif ! cp "$dawn_args_snapshot" "$dawn_args" ||
            ! cmp -s "$dawn_args_snapshot" "$dawn_args"; then
            echo "failed to restore Dawn args byte-for-byte; snapshot retained at $dawn_args_snapshot" >&2
            status=1
        else
            echo "Dawn args restored byte-for-byte: $dawn_args"
            rm -f "$dawn_args_snapshot"
        fi
    elif [[ -n "$dawn_args_snapshot_candidate" ]]; then
        rm -f "$dawn_args_snapshot_candidate"
    fi
    if (( dawn_patch_applied )); then
        if ! git -C "$dawn_dir" apply --reverse "$dawn_patch"; then
            echo "failed to reverse Dawn compatibility patch" >&2
            status=1
        fi
    fi
    if (( applied )); then
        if ! git -C "$runtime" apply --reverse "$patch"; then
            echo "failed to reverse runtime oracle patch" >&2
            status=1
        fi
    fi
    rm -rf "$injected_dir"
    if [[ -n "$atomic_spotify_kids_app_icon_full_schedule" ]]; then
        rm -f "$atomic_spotify_kids_app_icon_full_schedule"
    fi
    exit "$status"
}
trap cleanup EXIT
trap 'exit 130' INT
trap 'exit 143' TERM
mkdir -p "$injected_dir" \
    "$(dirname "$output")" \
    "$(dirname "$inputs_output")" \
    "$(dirname "$blit_output")" \
    "$(dirname "$fill_output")" \
    "$(dirname "$fill_inputs_output")" \
    "$(dirname "$fill_blit_output")" \
    "$(dirname "$cusp_output")" \
    "$(dirname "$cusp_inputs_output")" \
    "$(dirname "$cusp_blit_output")" \
    "$(dirname "$empty_stroke_output")" \
    "$(dirname "$empty_stroke_inputs_output")" \
    "$(dirname "$empty_stroke_blit_output")" \
    "$(dirname "$empty_stroke_overlap_blit_output")" \
    "$(dirname "$softened_cusp_output")" \
    "$(dirname "$direct_cusp_inputs_output")" \
    "$(dirname "$direct_cusp_blit_output")" \
    "$(dirname "$direct_cusp_coverage_output")" \
    "$(dirname "$direct_polyshark_inputs_output")" \
    "$(dirname "$direct_grid_inputs_output")" \
    "$(dirname "$direct_flower_inputs_output")" \
    "$(dirname "$direct_bad_skin_inputs_output")" \
    "$(dirname "$direct_strokes_round_inputs_output")" \
    "$(dirname "$direct_strokes_round_blit_output")" \
    "$(dirname "$direct_strokes_round_spans_output")" \
    "$(dirname "$direct_rawtext_inputs_output")" \
    "$(dirname "$direct_rawtext_blit_output")" \
    "$(dirname "$direct_rawtext_spans_output")" \
    "$(dirname "$advanced_blend_blit_output")" \
    "$(dirname "$atomic_advanced_blend_output")" \
    "$(dirname "$atomic_colorburn_pair_output")" \
    "$(dirname "$atomic_colorburn_pair_color_output")" \
    "$(dirname "$atomic_colorburn_pair_coverage_output")" \
    "$(dirname "$atomic_interleavedfeather_full_output")" \
    "$(dirname "$atomic_interleavedfeather_full_provenance")" \
    "$(dirname "$atomic_dstreadshuffle_full_output")" \
    "$(dirname "$atomic_dstreadshuffle_full_provenance")" \
    "$(dirname "$atomic_dstreadshuffle_srcover_output")" \
    "$(dirname "$atomic_dstreadshuffle_srcover_provenance")" \
    "$(dirname "$atomic_spotify_kids_app_icon_full_output")" \
    "$(dirname "$atomic_spotify_kids_app_icon_full_coverage")" \
    "$(dirname "$atomic_spotify_kids_app_icon_full_clip")" \
    "$(dirname "$atomic_spotify_kids_app_icon_full_provenance")"
cp "$script_dir/runtime-src/main.cpp" "$injected_dir/main.cpp"
python3 "$polyshark_generator" --stream "$polyshark_stream" \
    --output "$injected_dir/generated_polyshark_path.inc"
python3 "$rawtext_generator" --stream "$rawtext_stream" \
    --output "$injected_dir/generated_rawtext_path.inc"
python3 "$colorburn_pair_generator" --stream "$colorburn_pair_stream" \
    --output "$injected_dir/generated_interleaved_colorburn_pair_path.inc"
python3 "$path_stream_generator" \
    --stream "$interleavedfeather_stream" \
    --expected-sha256 "$expected_interleavedfeather_sha256" \
    --expected-source gm:interleavedfeather \
    --expected-width 1000 --expected-height 1000 \
    --expected-clear-color 0x00000000 \
    --expected-count save=301 --expected-count restore=301 \
    --expected-count transform=900 --expected-count makeEmptyRenderPath=1 \
    --expected-count makeRenderPaint=1 --expected-count drawPath=451 \
    --function replayInterleavedFeatherFull \
    --output "$injected_dir/generated_interleavedfeather_full.inc"
python3 "$path_stream_generator" \
    --stream "$dstreadshuffle_stream" \
    --expected-sha256 "$expected_dstreadshuffle_sha256" \
    --expected-source gm:dstreadshuffle \
    --expected-width 530 --expected-height 690 \
    --expected-clear-color 0xffffffff \
    --expected-count save=97 --expected-count restore=97 \
    --expected-count transform=96 --expected-count makeEmptyRenderPath=193 \
    --expected-count makeRenderPaint=97 --expected-count drawPath=97 \
    --function replayDstReadShuffleFull \
    --output "$injected_dir/generated_dstreadshuffle_full.inc"
python3 "$path_stream_generator" \
    --stream "$dstreadshuffle_stream" \
    --expected-sha256 "$expected_dstreadshuffle_sha256" \
    --expected-source gm:dstreadshuffle \
    --expected-width 530 --expected-height 690 \
    --expected-clear-color 0xffffffff \
    --expected-count save=97 --expected-count restore=97 \
    --expected-count transform=96 --expected-count makeEmptyRenderPath=193 \
    --expected-count makeRenderPaint=97 --expected-count drawPath=97 \
    --override-blend-mode 3 \
    --function replayDstReadShuffleSrcOverControl \
    --output "$injected_dir/generated_dstreadshuffle_srcover_control.inc"
python3 "$path_stream_generator" \
    --profile riv \
    --stream "$spotify_kids_app_icon_stream" \
    --expected-sha256 "$expected_spotify_kids_app_icon_sha256" \
    --expected-source-suffix tests/unit_tests/assets/spotify_kids_app_icon.riv \
    --expected-artboard artboard --expected-scene "State Machine 1" \
    --expected-width 1024 --expected-height 1436 \
    --expected-sample-seconds 0 \
    --expected-count save=18 --expected-count restore=18 \
    --expected-count transform=15 --expected-count makeEmptyRenderPath=20 \
    --expected-count makeRenderPaint=48 --expected-count clipPath=6 \
    --expected-count drawPath=14 \
    --function replaySpotifyKidsAppIconFull \
    --output "$injected_dir/generated_spotify_kids_app_icon_full.inc"
python3 "$path_stream_generator" \
    --profile riv \
    --stream "$hunter_x_stream" \
    --expected-sha256 "$expected_hunter_x_sha256" \
    --expected-source-suffix tests/unit_tests/assets/hunter_x_demo.riv \
    --expected-artboard "Main Menu" --expected-scene "State Machine 1" \
    --expected-width 3387 --expected-height 1906 \
    --expected-sample-seconds 0 \
    --expected-count makeRenderPaint=1529 \
    --expected-count makeLinearGradient=182 \
    --expected-count makeRadialGradient=6 \
    --expected-count save=356 --expected-count restore=356 \
    --expected-count transform=343 --expected-count makeEmptyRenderPath=342 \
    --expected-count clipPath=130 --expected-count drawPath=318 \
    --function replayHunterXFull \
    --output "$injected_dir/generated_hunter_x_full.inc"
python3 "$msaa_reference_generator" \
    --manifest "$msaa_reference_manifest" \
    --repo-root "$script_dir/../.." \
    --runtime-revision "$expected_runtime_revision" \
    --dawn-revision "$expected_dawn_revision" \
    --output "$injected_dir/generated_msaa_reference_registry.inc"
git -C "$runtime" apply "$patch"
applied=1
if needs_xcode26_patch && dawn_patch_needed; then
    git -C "$dawn_dir" apply "$dawn_patch"
    dawn_patch_applied=1
fi
dawn_args_snapshot_candidate="$(mktemp "${TMPDIR:-/tmp}/cpp-atlas-mask-dawn-args.XXXXXX")"
cp "$dawn_args" "$dawn_args_snapshot_candidate"
dawn_args_snapshot="$dawn_args_snapshot_candidate"
configure_xcode26_dawn_args

(
    cd "$dawn_dir"
    "$gn_bin" gen out/release
    "$ninja_bin" -C out/release -j"$jobs" webgpu_dawn_static cpp proc_static
)

(
    cd "$runtime/renderer"
    PREMAKE_PATH="$runtime/build${PREMAKE_PATH:+:$PREMAKE_PATH}" \
        premake5 gmake2 --file=premake5.lua --config=release --out="$build_out" --with-dawn --no-lto
    make -C "$build_out" -j"$jobs" rive_atlas_mask_oracle
)

if [[ "$run_mode" == build-only ]]; then
    echo "built $runtime/renderer/$build_out/rive_atlas_mask_oracle"
    exit 0
fi

rm -f "$output" "$inputs_output" "$blit_output" "$clipped_blit_output" "$path_clipped_blit_output" "$changing_path_clipped_blit_output" "$nested_path_clipped_blit_output" "$nested_evenodd_path_clipped_blit_output" "$nested_clockwise_path_clipped_blit_output" "$advanced_blend_blit_output" "$atomic_advanced_blend_output" "$atomic_colorburn_pair_output" "$atomic_colorburn_pair_color_output" "$atomic_colorburn_pair_coverage_output" "$atomic_interleavedfeather_full_output" "$atomic_interleavedfeather_full_provenance" "$atomic_dstreadshuffle_full_output" "$atomic_dstreadshuffle_full_provenance" "$atomic_dstreadshuffle_srcover_output" "$atomic_dstreadshuffle_srcover_provenance" "$atomic_spotify_kids_app_icon_full_output" "$atomic_spotify_kids_app_icon_full_coverage" "$atomic_spotify_kids_app_icon_full_clip" "$atomic_spotify_kids_app_icon_full_provenance" "$fill_output" "$fill_inputs_output" "$fill_blit_output" "$cusp_output" "$cusp_inputs_output" "$cusp_blit_output" "$empty_stroke_output" "$empty_stroke_inputs_output" "$empty_stroke_blit_output" "$empty_stroke_overlap_blit_output" "$softened_cusp_output" "$direct_cusp_inputs_output" "$direct_cusp_blit_output" "$direct_cusp_coverage_output" "$direct_polyshark_inputs_output" "$direct_grid_inputs_output" "$direct_flower_inputs_output" "$direct_bad_skin_inputs_output" "$direct_strokes_round_inputs_output" "$direct_rawtext_inputs_output" "$direct_rawtext_blit_output" "$direct_rawtext_spans_output"
"$runtime/renderer/$build_out/rive_atlas_mask_oracle" "$output" "$inputs_output" "$blit_output"
"$runtime/renderer/$build_out/rive_atlas_mask_oracle" /dev/null /dev/null "$clipped_blit_output" clipped
"$runtime/renderer/$build_out/rive_atlas_mask_oracle" /dev/null /dev/null "$path_clipped_blit_output" path-clipped
"$runtime/renderer/$build_out/rive_atlas_mask_oracle" /dev/null /dev/null "$changing_path_clipped_blit_output" changing-path-clipped
"$runtime/renderer/$build_out/rive_atlas_mask_oracle" /dev/null /dev/null "$nested_path_clipped_blit_output" nested-path-clipped
"$runtime/renderer/$build_out/rive_atlas_mask_oracle" /dev/null /dev/null "$nested_evenodd_path_clipped_blit_output" nested-evenodd-path-clipped
"$runtime/renderer/$build_out/rive_atlas_mask_oracle" /dev/null /dev/null "$nested_clockwise_path_clipped_blit_output" nested-clockwise-path-clipped
"$runtime/renderer/$build_out/rive_atlas_mask_oracle" /dev/null /dev/null "$advanced_blend_blit_output" advanced-blend
"$runtime/renderer/$build_out/rive_atlas_mask_oracle" /dev/null /dev/null "$atomic_advanced_blend_output" atomic-advanced-blend
"$runtime/renderer/$build_out/rive_atlas_mask_oracle" /dev/null /dev/null "$atomic_colorburn_pair_output" atomic-colorburn-pair "$atomic_colorburn_pair_color_output" "$atomic_colorburn_pair_coverage_output"
"$runtime/renderer/$build_out/rive_atlas_mask_oracle" /dev/null /dev/null "$atomic_interleavedfeather_full_output" atomic-interleavedfeather-full "$atomic_interleavedfeather_full_provenance"
atomic_interleavedfeather_full_sha256="$(sha256_file "$atomic_interleavedfeather_full_output")"
printf 'artifact_sha256=%s\nstream_sha256=%s\nruntime_revision=%s\ndawn_revision=%s\n' \
    "$atomic_interleavedfeather_full_sha256" \
    "$expected_interleavedfeather_sha256" \
    "$expected_runtime_revision" \
    "$expected_dawn_revision" >> "$atomic_interleavedfeather_full_provenance"
"$runtime/renderer/$build_out/rive_atlas_mask_oracle" /dev/null /dev/null "$atomic_dstreadshuffle_full_output" atomic-dstreadshuffle-full "$atomic_dstreadshuffle_full_provenance"
atomic_dstreadshuffle_full_sha256="$(sha256_file "$atomic_dstreadshuffle_full_output")"
printf 'artifact_sha256=%s\nstream_sha256=%s\nruntime_revision=%s\ndawn_revision=%s\n' \
    "$atomic_dstreadshuffle_full_sha256" \
    "$expected_dstreadshuffle_sha256" \
    "$expected_runtime_revision" \
    "$expected_dawn_revision" >> "$atomic_dstreadshuffle_full_provenance"
"$runtime/renderer/$build_out/rive_atlas_mask_oracle" /dev/null /dev/null "$atomic_dstreadshuffle_srcover_output" atomic-dstreadshuffle-srcover-full "$atomic_dstreadshuffle_srcover_provenance"
atomic_dstreadshuffle_srcover_sha256="$(sha256_file "$atomic_dstreadshuffle_srcover_output")"
printf 'artifact_sha256=%s\nstream_sha256=%s\nruntime_revision=%s\ndawn_revision=%s\nblend_mode_override=srcOver\n' \
    "$atomic_dstreadshuffle_srcover_sha256" \
    "$expected_dstreadshuffle_sha256" \
    "$expected_runtime_revision" \
    "$expected_dawn_revision" >> "$atomic_dstreadshuffle_srcover_provenance"
atomic_spotify_kids_app_icon_full_schedule="$(mktemp "${TMPDIR:-/tmp}/cpp-atlas-mask-spotify-schedule.XXXXXX")"
"$runtime/renderer/$build_out/rive_atlas_mask_oracle" "$atomic_spotify_kids_app_icon_full_schedule" "$atomic_spotify_kids_app_icon_full_coverage" "$atomic_spotify_kids_app_icon_full_output" atomic-spotify-kids-app-icon-full "$atomic_spotify_kids_app_icon_full_provenance" "$atomic_spotify_kids_app_icon_full_clip"
atomic_spotify_kids_app_icon_full_replay_sha256="$(sha256_file "$injected_dir/generated_spotify_kids_app_icon_full.inc")"
atomic_spotify_kids_app_icon_full_sha256="$(sha256_file "$atomic_spotify_kids_app_icon_full_output")"
atomic_spotify_kids_app_icon_full_coverage_sha256="$(sha256_file "$atomic_spotify_kids_app_icon_full_coverage")"
atomic_spotify_kids_app_icon_full_clip_sha256="$(sha256_file "$atomic_spotify_kids_app_icon_full_clip")"
atomic_spotify_kids_app_icon_full_schedule_sha256="$(sha256_file "$atomic_spotify_kids_app_icon_full_schedule")"
printf 'replay_sha256=%s\nartifact_sha256=%s\ncoverage_sha256=%s\nclip_sha256=%s\ndraw_schedule_sha256=%s\nstream_sha256=%s\nruntime_revision=%s\ndawn_revision=%s\nframe_width=1024\nframe_height=1436\nstorage_width=1024\nstorage_height=1440\nsample_seconds_bits=00000000\ndraw_batch_count=24\nfixed_function_color_output=true\npacked_color_backing=absent\n' \
    "$atomic_spotify_kids_app_icon_full_replay_sha256" \
    "$atomic_spotify_kids_app_icon_full_sha256" \
    "$atomic_spotify_kids_app_icon_full_coverage_sha256" \
    "$atomic_spotify_kids_app_icon_full_clip_sha256" \
    "$atomic_spotify_kids_app_icon_full_schedule_sha256" \
    "$expected_spotify_kids_app_icon_sha256" \
    "$expected_runtime_revision" \
    "$expected_dawn_revision" >> "$atomic_spotify_kids_app_icon_full_provenance"
validate_spotify_artifacts
"$runtime/renderer/$build_out/rive_atlas_mask_oracle" /dev/null /dev/null /dev/null msaa-intersection-groups
"$runtime/renderer/$build_out/rive_atlas_mask_oracle" "$fill_output" "$fill_inputs_output" "$fill_blit_output" fill
"$runtime/renderer/$build_out/rive_atlas_mask_oracle" "$cusp_output" "$cusp_inputs_output" "$cusp_blit_output" cusp "$softened_cusp_output"
"$runtime/renderer/$build_out/rive_atlas_mask_oracle" "$large_feather_cusp_output" "$large_feather_cusp_inputs_output" "$large_feather_cusp_blit_output" large-feather-cusp "$large_feather_cusp_placement_output"
"$runtime/renderer/$build_out/rive_atlas_mask_oracle" "$large_feather_shapes_cusp_output" "$large_feather_shapes_cusp_inputs_output" "$large_feather_shapes_cusp_blit_output" large-feather-shapes-cusp "$large_feather_shapes_cusp_placement_output"
"$runtime/renderer/$build_out/rive_atlas_mask_oracle" "$empty_stroke_output" "$empty_stroke_inputs_output" "$empty_stroke_blit_output" empty-stroke
"$runtime/renderer/$build_out/rive_atlas_mask_oracle" /dev/null /dev/null "$empty_stroke_overlap_blit_output" empty-stroke-overlap
"$runtime/renderer/$build_out/rive_atlas_mask_oracle" /dev/null "$direct_cusp_inputs_output" "$direct_cusp_blit_output" direct-cusp "$direct_cusp_coverage_output"
"$runtime/renderer/$build_out/rive_atlas_mask_oracle" /dev/null "$direct_polyshark_inputs_output" /dev/null direct-polyshark
"$runtime/renderer/$build_out/rive_atlas_mask_oracle" /dev/null "$direct_grid_inputs_output" /dev/null direct-grid
"$runtime/renderer/$build_out/rive_atlas_mask_oracle" /dev/null "$direct_flower_inputs_output" /dev/null direct-flower
"$runtime/renderer/$build_out/rive_atlas_mask_oracle" /dev/null "$direct_bad_skin_inputs_output" /dev/null direct-bad-skin
"$runtime/renderer/$build_out/rive_atlas_mask_oracle" /dev/null "$direct_strokes_round_inputs_output" "$direct_strokes_round_blit_output" direct-strokes-round "$direct_strokes_round_spans_output"
"$runtime/renderer/$build_out/rive_atlas_mask_oracle" /dev/null "$direct_rawtext_inputs_output" "$direct_rawtext_blit_output" direct-rawtext "$direct_rawtext_spans_output"
python3 "$script_dir/format_test.py" --validate-direct-grid "$direct_grid_inputs_output"
python3 "$script_dir/format_test.py" --validate-direct-flower "$direct_flower_inputs_output"
python3 "$script_dir/format_test.py" --validate-direct-bad-skin "$direct_bad_skin_inputs_output"
python3 "$script_dir/format_test.py" --validate-direct-cusp-coverage "$direct_cusp_coverage_output"
python3 "$script_dir/format_test.py" --validate-atomic-colorburn-pair "$atomic_colorburn_pair_color_output"
python3 "$script_dir/format_test.py" --validate-atomic-colorburn-pair-coverage "$atomic_colorburn_pair_coverage_output"
python3 "$script_dir/format_test.py" --validate-direct-strokes-round "$direct_strokes_round_inputs_output"
python3 "$script_dir/format_test.py" --validate-direct-strokes-round-spans "$direct_strokes_round_spans_output"
python3 "$script_dir/format_test.py" --validate-direct-rawtext "$direct_rawtext_inputs_output"
python3 "$script_dir/format_test.py" --validate-direct-rawtext-spans "$direct_rawtext_spans_output"
output_bytes="$(wc -c < "$output" | tr -d ' ')"
if [[ "$output_bytes" != "4628" ]]; then
    echo "atlas mask must be exactly 4628 bytes, got $output_bytes: $output" >&2
    exit 1
fi
inputs_bytes="$(wc -c < "$inputs_output" | tr -d ' ')"
if (( inputs_bytes <= 56 )); then
    echo "atlas inputs must contain a 40-byte header, contour, and tessellation payload: $inputs_output" >&2
    exit 1
fi
blit_bytes="$(wc -c < "$blit_output" | tr -d ' ')"
if [[ "$blit_bytes" != "16404" ]]; then
    echo "atlas blit must be exactly 16404 bytes, got $blit_bytes: $blit_output" >&2
    exit 1
fi
large_feather_cusp_mask_bytes="$(wc -c < "$large_feather_cusp_output" | tr -d ' ')"
large_feather_cusp_inputs_bytes="$(wc -c < "$large_feather_cusp_inputs_output" | tr -d ' ')"
large_feather_cusp_placement_bytes="$(wc -c < "$large_feather_cusp_placement_output" | tr -d ' ')"
large_feather_cusp_blit_bytes="$(wc -c < "$large_feather_cusp_blit_output" | tr -d ' ')"
if [[ "$large_feather_cusp_mask_bytes" != "2600" ||
      "$large_feather_cusp_inputs_bytes" != "32840" ||
      "$large_feather_cusp_placement_bytes" != "88" ||
      "$large_feather_cusp_blit_bytes" != "14385172" ]]; then
    echo "large-feather cusp artifacts have unexpected sizes" >&2
    exit 1
fi
large_feather_shapes_cusp_mask_bytes="$(wc -c < "$large_feather_shapes_cusp_output" | tr -d ' ')"
large_feather_shapes_cusp_inputs_bytes="$(wc -c < "$large_feather_shapes_cusp_inputs_output" | tr -d ' ')"
large_feather_shapes_cusp_placement_bytes="$(wc -c < "$large_feather_shapes_cusp_placement_output" | tr -d ' ')"
large_feather_shapes_cusp_blit_bytes="$(wc -c < "$large_feather_shapes_cusp_blit_output" | tr -d ' ')"
if [[ "$large_feather_shapes_cusp_mask_bytes" != "2600" ||
      "$large_feather_shapes_cusp_inputs_bytes" != "32824" ||
      "$large_feather_shapes_cusp_placement_bytes" != "88" ||
      "$large_feather_shapes_cusp_blit_bytes" != "14385172" ]]; then
    echo "large-feather shapes-cusp artifacts have unexpected sizes" >&2
    exit 1
fi
clipped_blit_bytes="$(wc -c < "$clipped_blit_output" | tr -d ' ')"
if [[ "$clipped_blit_bytes" != "16404" ]]; then
    echo "clipped atlas blit must be exactly 16404 bytes, got $clipped_blit_bytes: $clipped_blit_output" >&2
    exit 1
fi
path_clipped_blit_bytes="$(wc -c < "$path_clipped_blit_output" | tr -d ' ')"
if [[ "$path_clipped_blit_bytes" != "16404" ]]; then
    echo "path-clipped atlas blit must be exactly 16404 bytes, got $path_clipped_blit_bytes: $path_clipped_blit_output" >&2
    exit 1
fi
changing_path_clipped_blit_bytes="$(wc -c < "$changing_path_clipped_blit_output" | tr -d ' ')"
if [[ "$changing_path_clipped_blit_bytes" != "16404" ]]; then
    echo "changing path-clipped atlas blit must be exactly 16404 bytes, got $changing_path_clipped_blit_bytes: $changing_path_clipped_blit_output" >&2
    exit 1
fi
nested_path_clipped_blit_bytes="$(wc -c < "$nested_path_clipped_blit_output" | tr -d ' ')"
if [[ "$nested_path_clipped_blit_bytes" != "16404" ]]; then
    echo "nested path-clipped atlas blit must be exactly 16404 bytes, got $nested_path_clipped_blit_bytes: $nested_path_clipped_blit_output" >&2
    exit 1
fi
nested_evenodd_path_clipped_blit_bytes="$(wc -c < "$nested_evenodd_path_clipped_blit_output" | tr -d ' ')"
if [[ "$nested_evenodd_path_clipped_blit_bytes" != "16404" ]]; then
    echo "nested even-odd path-clipped atlas blit must be exactly 16404 bytes, got $nested_evenodd_path_clipped_blit_bytes: $nested_evenodd_path_clipped_blit_output" >&2
    exit 1
fi
nested_clockwise_path_clipped_blit_bytes="$(wc -c < "$nested_clockwise_path_clipped_blit_output" | tr -d ' ')"
if [[ "$nested_clockwise_path_clipped_blit_bytes" != "16404" ]]; then
    echo "nested clockwise path-clipped atlas blit must be exactly 16404 bytes, got $nested_clockwise_path_clipped_blit_bytes: $nested_clockwise_path_clipped_blit_output" >&2
    exit 1
fi
advanced_blend_blit_bytes="$(wc -c < "$advanced_blend_blit_output" | tr -d ' ')"
if [[ "$advanced_blend_blit_bytes" != "16404" ]]; then
    echo "advanced-blend atlas blit must be exactly 16404 bytes, got $advanced_blend_blit_bytes: $advanced_blend_blit_output" >&2
    exit 1
fi
atomic_advanced_blend_bytes="$(wc -c < "$atomic_advanced_blend_output" | tr -d ' ')"
if [[ "$atomic_advanced_blend_bytes" != "16404" ]]; then
    echo "atomic advanced-blend output must be exactly 16404 bytes, got $atomic_advanced_blend_bytes: $atomic_advanced_blend_output" >&2
    exit 1
fi
atomic_colorburn_pair_bytes="$(wc -c < "$atomic_colorburn_pair_output" | tr -d ' ')"
if [[ "$atomic_colorburn_pair_bytes" != "4194324" ]]; then
    echo "atomic colorburn pair output must be exactly 4194324 bytes, got $atomic_colorburn_pair_bytes: $atomic_colorburn_pair_output" >&2
    exit 1
fi
atomic_colorburn_pair_color_bytes="$(wc -c < "$atomic_colorburn_pair_color_output" | tr -d ' ')"
if [[ "$atomic_colorburn_pair_color_bytes" != "4194328" ]]; then
    echo "atomic colorburn pair color storage must be exactly 4194328 bytes, got $atomic_colorburn_pair_color_bytes: $atomic_colorburn_pair_color_output" >&2
    exit 1
fi
atomic_colorburn_pair_coverage_bytes="$(wc -c < "$atomic_colorburn_pair_coverage_output" | tr -d ' ')"
if [[ "$atomic_colorburn_pair_coverage_bytes" != "4194328" ]]; then
    echo "atomic colorburn pair coverage must be exactly 4194328 bytes, got $atomic_colorburn_pair_coverage_bytes: $atomic_colorburn_pair_coverage_output" >&2
    exit 1
fi
atomic_interleavedfeather_full_bytes="$(wc -c < "$atomic_interleavedfeather_full_output" | tr -d ' ')"
if [[ "$atomic_interleavedfeather_full_bytes" != "4000020" ]]; then
    echo "atomic interleavedfeather full output must be exactly 4000020 bytes, got $atomic_interleavedfeather_full_bytes: $atomic_interleavedfeather_full_output" >&2
    exit 1
fi
for provenance in \
    "backend=metal" \
    "artifact_sha256=$atomic_interleavedfeather_full_sha256" \
    "stream_sha256=$expected_interleavedfeather_sha256" \
    "runtime_revision=$expected_runtime_revision" \
    "dawn_revision=$expected_dawn_revision"; do
    if ! grep -Fqx "$provenance" "$atomic_interleavedfeather_full_provenance"; then
        echo "atomic interleavedfeather provenance is missing $provenance: $atomic_interleavedfeather_full_provenance" >&2
        exit 1
    fi
done
for provenance in \
    "backend=metal" \
    "replay_sha256=$atomic_spotify_kids_app_icon_full_replay_sha256" \
    "artifact_sha256=$atomic_spotify_kids_app_icon_full_sha256" \
    "coverage_sha256=$atomic_spotify_kids_app_icon_full_coverage_sha256" \
    "clip_sha256=$atomic_spotify_kids_app_icon_full_clip_sha256" \
    "draw_schedule_sha256=$atomic_spotify_kids_app_icon_full_schedule_sha256" \
    "stream_sha256=$expected_spotify_kids_app_icon_sha256" \
    "runtime_revision=$expected_runtime_revision" \
    "dawn_revision=$expected_dawn_revision" \
    "frame_width=1024" \
    "frame_height=1436" \
    "storage_width=1024" \
    "storage_height=1440" \
    "sample_seconds_bits=00000000" \
    "draw_batch_count=24" \
    "fixed_function_color_output=true" \
    "packed_color_backing=absent"; do
    if ! grep -Fqx "$provenance" "$atomic_spotify_kids_app_icon_full_provenance"; then
        echo "atomic Spotify provenance is missing $provenance: $atomic_spotify_kids_app_icon_full_provenance" >&2
        exit 1
    fi
done
atomic_dstreadshuffle_srcover_bytes="$(wc -c < "$atomic_dstreadshuffle_srcover_output" | tr -d ' ')"
if [[ "$atomic_dstreadshuffle_srcover_bytes" != "1462820" ]]; then
    echo "atomic dstreadshuffle SrcOver output must be exactly 1462820 bytes, got $atomic_dstreadshuffle_srcover_bytes: $atomic_dstreadshuffle_srcover_output" >&2
    exit 1
fi
for provenance in \
    "backend=metal" \
    "artifact_sha256=$atomic_dstreadshuffle_srcover_sha256" \
    "stream_sha256=$expected_dstreadshuffle_sha256" \
    "runtime_revision=$expected_runtime_revision" \
    "dawn_revision=$expected_dawn_revision" \
    "blend_mode_override=srcOver"; do
    if ! grep -Fqx "$provenance" "$atomic_dstreadshuffle_srcover_provenance"; then
        echo "atomic dstreadshuffle SrcOver provenance is missing $provenance: $atomic_dstreadshuffle_srcover_provenance" >&2
        exit 1
    fi
done
atomic_dstreadshuffle_full_bytes="$(wc -c < "$atomic_dstreadshuffle_full_output" | tr -d ' ')"
if [[ "$atomic_dstreadshuffle_full_bytes" != "1462820" ]]; then
    echo "atomic dstreadshuffle full output must be exactly 1462820 bytes, got $atomic_dstreadshuffle_full_bytes: $atomic_dstreadshuffle_full_output" >&2
    exit 1
fi
for provenance in \
    "backend=metal" \
    "artifact_sha256=$atomic_dstreadshuffle_full_sha256" \
    "stream_sha256=$expected_dstreadshuffle_sha256" \
    "runtime_revision=$expected_runtime_revision" \
    "dawn_revision=$expected_dawn_revision"; do
    if ! grep -Fqx "$provenance" "$atomic_dstreadshuffle_full_provenance"; then
        echo "atomic dstreadshuffle provenance is missing $provenance: $atomic_dstreadshuffle_full_provenance" >&2
        exit 1
    fi
done
fill_output_bytes="$(wc -c < "$fill_output" | tr -d ' ')"
if [[ "$fill_output_bytes" != "4628" ]]; then
    echo "atlas fill mask must be exactly 4628 bytes, got $fill_output_bytes: $fill_output" >&2
    exit 1
fi
fill_inputs_bytes="$(wc -c < "$fill_inputs_output" | tr -d ' ')"
if (( fill_inputs_bytes <= 56 )); then
    echo "atlas fill inputs must contain a header, contour, and tessellation payload: $fill_inputs_output" >&2
    exit 1
fi
fill_blit_bytes="$(wc -c < "$fill_blit_output" | tr -d ' ')"
if [[ "$fill_blit_bytes" != "16404" ]]; then
    echo "atlas fill blit must be exactly 16404 bytes, got $fill_blit_bytes: $fill_blit_output" >&2
    exit 1
fi
cusp_output_bytes="$(wc -c < "$cusp_output" | tr -d ' ')"
if [[ "$cusp_output_bytes" != "4628" ]]; then
    echo "atlas cusp mask must be exactly 4628 bytes, got $cusp_output_bytes: $cusp_output" >&2
    exit 1
fi
cusp_inputs_bytes="$(wc -c < "$cusp_inputs_output" | tr -d ' ')"
if (( cusp_inputs_bytes <= 56 )); then
    echo "atlas cusp inputs must contain a header, contour, and tessellation payload: $cusp_inputs_output" >&2
    exit 1
fi
cusp_blit_bytes="$(wc -c < "$cusp_blit_output" | tr -d ' ')"
if [[ "$cusp_blit_bytes" != "16404" ]]; then
    echo "atlas cusp blit must be exactly 16404 bytes, got $cusp_blit_bytes: $cusp_blit_output" >&2
    exit 1
fi
empty_stroke_output_bytes="$(wc -c < "$empty_stroke_output" | tr -d ' ')"
if [[ "$empty_stroke_output_bytes" != "4628" ]]; then
    echo "atlas empty-stroke mask must be exactly 4628 bytes, got $empty_stroke_output_bytes: $empty_stroke_output" >&2
    exit 1
fi
empty_stroke_inputs_bytes="$(wc -c < "$empty_stroke_inputs_output" | tr -d ' ')"
if (( empty_stroke_inputs_bytes <= 56 )); then
    echo "atlas empty-stroke inputs must contain a header, contour, and tessellation payload: $empty_stroke_inputs_output" >&2
    exit 1
fi
empty_stroke_blit_bytes="$(wc -c < "$empty_stroke_blit_output" | tr -d ' ')"
if [[ "$empty_stroke_blit_bytes" != "16404" ]]; then
    echo "atlas empty-stroke blit must be exactly 16404 bytes, got $empty_stroke_blit_bytes: $empty_stroke_blit_output" >&2
    exit 1
fi
empty_stroke_overlap_blit_bytes="$(wc -c < "$empty_stroke_overlap_blit_output" | tr -d ' ')"
if [[ "$empty_stroke_overlap_blit_bytes" != "16404" ]]; then
    echo "atlas empty-stroke overlap blit must be exactly 16404 bytes, got $empty_stroke_overlap_blit_bytes: $empty_stroke_overlap_blit_output" >&2
    exit 1
fi
softened_cusp_bytes="$(wc -c < "$softened_cusp_output" | tr -d ' ')"
if (( softened_cusp_bytes <= 20 )); then
    echo "softened cusp must contain a header and path payload: $softened_cusp_output" >&2
    exit 1
fi
direct_cusp_inputs_bytes="$(wc -c < "$direct_cusp_inputs_output" | tr -d ' ')"
if (( direct_cusp_inputs_bytes <= 56 )); then
    echo "direct cusp inputs must contain a header, contour, and tessellation payload: $direct_cusp_inputs_output" >&2
    exit 1
fi
direct_cusp_blit_bytes="$(wc -c < "$direct_cusp_blit_output" | tr -d ' ')"
if [[ "$direct_cusp_blit_bytes" != "16404" ]]; then
    echo "direct cusp blit must be exactly 16404 bytes, got $direct_cusp_blit_bytes: $direct_cusp_blit_output" >&2
    exit 1
fi
direct_cusp_coverage_bytes="$(wc -c < "$direct_cusp_coverage_output" | tr -d ' ')"
if [[ "$direct_cusp_coverage_bytes" != "16408" ]]; then
    echo "direct cusp atomic coverage must be exactly 16408 bytes, got $direct_cusp_coverage_bytes: $direct_cusp_coverage_output" >&2
    exit 1
fi
direct_polyshark_inputs_bytes="$(wc -c < "$direct_polyshark_inputs_output" | tr -d ' ')"
if [[ "$direct_polyshark_inputs_bytes" != "163896" ]]; then
    echo "direct polyshark inputs must be exactly 163896 bytes: $direct_polyshark_inputs_output" >&2
    exit 1
fi
direct_bad_skin_inputs_bytes="$(wc -c < "$direct_bad_skin_inputs_output" | tr -d ' ')"
if (( direct_bad_skin_inputs_bytes <= 56 )); then
    echo "direct bad-skin inputs must contain a header, contour, triangle, and tessellation payload: $direct_bad_skin_inputs_output" >&2
    exit 1
fi
direct_strokes_round_blit_bytes="$(wc -c < "$direct_strokes_round_blit_output" | tr -d ' ')"
if [[ "$direct_strokes_round_blit_bytes" != "640020" ]]; then
    echo "direct strokes-round blit must be exactly 640020 bytes, got $direct_strokes_round_blit_bytes: $direct_strokes_round_blit_output" >&2
    exit 1
fi
direct_rawtext_inputs_bytes="$(wc -c < "$direct_rawtext_inputs_output" | tr -d ' ')"
if [[ "$direct_rawtext_inputs_bytes" != "66152" ]]; then
    echo "direct rawtext inputs must be exactly 66152 bytes, got $direct_rawtext_inputs_bytes: $direct_rawtext_inputs_output" >&2
    exit 1
fi
direct_rawtext_spans_bytes="$(wc -c < "$direct_rawtext_spans_output" | tr -d ' ')"
if [[ "$direct_rawtext_spans_bytes" != "28060" ]]; then
    echo "direct rawtext spans must be exactly 28060 bytes, got $direct_rawtext_spans_bytes: $direct_rawtext_spans_output" >&2
    exit 1
fi
direct_rawtext_blit_bytes="$(wc -c < "$direct_rawtext_blit_output" | tr -d ' ')"
if [[ "$direct_rawtext_blit_bytes" != "536020" ]]; then
    echo "direct rawtext blit must be exactly 536020 bytes, got $direct_rawtext_blit_bytes: $direct_rawtext_blit_output" >&2
    exit 1
fi
echo "atlas mask: $output"
echo "atlas inputs: $inputs_output"
echo "atlas blit: $blit_output"
echo "atlas clipped blit: $clipped_blit_output"
echo "atlas path-clipped blit: $path_clipped_blit_output"
echo "atlas changing path-clipped blit: $changing_path_clipped_blit_output"
echo "atlas nested path-clipped blit: $nested_path_clipped_blit_output"
echo "atlas nested even-odd path-clipped blit: $nested_evenodd_path_clipped_blit_output"
echo "atlas nested clockwise path-clipped blit: $nested_clockwise_path_clipped_blit_output"
echo "atlas advanced-blend blit: $advanced_blend_blit_output"
echo "atomic advanced-blend output: $atomic_advanced_blend_output"
echo "atomic colorburn-pair output: $atomic_colorburn_pair_output"
echo "atomic colorburn-pair color storage: $atomic_colorburn_pair_color_output"
echo "atomic colorburn-pair coverage storage: $atomic_colorburn_pair_coverage_output"
echo "atomic interleavedfeather full output: $atomic_interleavedfeather_full_output"
echo "atomic interleavedfeather full provenance: $atomic_interleavedfeather_full_provenance"
echo "atomic dstreadshuffle full output: $atomic_dstreadshuffle_full_output"
echo "atomic dstreadshuffle full provenance: $atomic_dstreadshuffle_full_provenance"
echo "atomic dstreadshuffle SrcOver output: $atomic_dstreadshuffle_srcover_output"
echo "atomic dstreadshuffle SrcOver provenance: $atomic_dstreadshuffle_srcover_provenance"
echo "atomic Spotify Kids app-icon full output: $atomic_spotify_kids_app_icon_full_output"
echo "atomic Spotify Kids app-icon full coverage: $atomic_spotify_kids_app_icon_full_coverage"
echo "atomic Spotify Kids app-icon full clip: $atomic_spotify_kids_app_icon_full_clip"
echo "atomic Spotify Kids app-icon full provenance: $atomic_spotify_kids_app_icon_full_provenance"
echo "atlas fill mask: $fill_output"
echo "atlas fill inputs: $fill_inputs_output"
echo "atlas fill blit: $fill_blit_output"
echo "atlas cusp mask: $cusp_output"
echo "atlas cusp inputs: $cusp_inputs_output"
echo "atlas cusp blit: $cusp_blit_output"
echo "atlas large-feather cusp mask: $large_feather_cusp_output"
echo "atlas large-feather cusp inputs: $large_feather_cusp_inputs_output"
echo "atlas large-feather cusp placement: $large_feather_cusp_placement_output"
echo "atlas large-feather cusp blit: $large_feather_cusp_blit_output"
echo "atlas large-feather shapes-cusp mask: $large_feather_shapes_cusp_output"
echo "atlas large-feather shapes-cusp inputs: $large_feather_shapes_cusp_inputs_output"
echo "atlas large-feather shapes-cusp placement: $large_feather_shapes_cusp_placement_output"
echo "atlas large-feather shapes-cusp blit: $large_feather_shapes_cusp_blit_output"
echo "atlas empty-stroke mask: $empty_stroke_output"
echo "atlas empty-stroke inputs: $empty_stroke_inputs_output"
echo "atlas empty-stroke blit: $empty_stroke_blit_output"
echo "atlas empty-stroke overlap blit: $empty_stroke_overlap_blit_output"
echo "softened cusp: $softened_cusp_output"
echo "direct cusp inputs: $direct_cusp_inputs_output"
echo "direct cusp blit: $direct_cusp_blit_output"
echo "direct cusp atomic coverage: $direct_cusp_coverage_output"
echo "direct polyshark inputs: $direct_polyshark_inputs_output"
echo "direct grid inputs: $direct_grid_inputs_output"
echo "direct flower inputs: $direct_flower_inputs_output"
echo "direct bad-skin inputs: $direct_bad_skin_inputs_output"
echo "direct strokes-round inputs: $direct_strokes_round_inputs_output"
echo "direct strokes-round blit: $direct_strokes_round_blit_output"
echo "direct strokes-round spans: $direct_strokes_round_spans_output"
echo "direct rawtext inputs: $direct_rawtext_inputs_output"
echo "direct rawtext blit: $direct_rawtext_blit_output"
echo "direct rawtext spans: $direct_rawtext_spans_output"
