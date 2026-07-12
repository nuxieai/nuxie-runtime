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
fill_output="${RIVE_ATLAS_FILL_MASK_OUTPUT:-$script_dir/out/atlas-fill-mask.r16f}"
fill_inputs_output="${RIVE_ATLAS_FILL_INPUT_OUTPUT:-$script_dir/out/atlas-fill-inputs.bin}"
fill_blit_output="${RIVE_ATLAS_FILL_BLIT_OUTPUT:-$script_dir/out/atlas-fill-blit.rgba}"
ninja_bin="${RIVE_ATLAS_MASK_NINJA:-$dawn_dir/third_party/ninja/ninja}"
case "$(uname -s)" in
    Darwin) default_gn="$dawn_dir/buildtools/mac/gn" ;;
    Linux) default_gn="$dawn_dir/buildtools/linux64/gn" ;;
    *) default_gn="$script_dir/../../target/depot_tools/gn" ;;
esac
gn_bin="${RIVE_ATLAS_MASK_GN:-$default_gn}"
naga_bin="${RIVE_ATLAS_MASK_NAGA:-$HOME/.cargo/bin/naga}"
expected_naga_version="30.0.0"

xcode_major() {
    xcodebuild -version 2>/dev/null | awk '/Xcode/ { split($2, version, "."); print version[1]; exit }'
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
    echo "usage: tools/cpp-atlas-mask-oracle/build.sh [--preflight]" >&2
}

preflight() {
    local missing=0
    local naga_output
    local naga_version
    python3 "$script_dir/format_test.py"
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

case "${1:-}" in
    --preflight)
        preflight
        exit $?
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
    "$(dirname "$fill_blit_output")"
cp "$script_dir/runtime-src/main.cpp" "$injected_dir/main.cpp"
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
        premake5 gmake2 --file=premake5.lua --config=release --out="$build_out" --with-dawn
    make -C "$build_out" -j"$jobs" rive_atlas_mask_oracle
)

rm -f "$output" "$inputs_output" "$blit_output" "$fill_output" "$fill_inputs_output" "$fill_blit_output"
"$runtime/renderer/$build_out/rive_atlas_mask_oracle" "$output" "$inputs_output" "$blit_output"
"$runtime/renderer/$build_out/rive_atlas_mask_oracle" "$fill_output" "$fill_inputs_output" "$fill_blit_output" fill
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
echo "atlas mask: $output"
echo "atlas inputs: $inputs_output"
echo "atlas blit: $blit_output"
echo "atlas fill mask: $fill_output"
echo "atlas fill inputs: $fill_inputs_output"
echo "atlas fill blit: $fill_blit_output"
