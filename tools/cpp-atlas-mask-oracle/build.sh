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
ninja_bin="${RIVE_ATLAS_MASK_NINJA:-$script_dir/../../target/depot_tools/ninja}"
gn_bin="${RIVE_ATLAS_MASK_GN:-$script_dir/../../target/depot_tools/gn}"

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
dawn_warning_arg_added=0
dawn_lld_arg_added=0

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
        dawn_warning_arg_added=1
    fi
    grep -Eq '^treat_warnings_as_errors[[:space:]]*=[[:space:]]*false[[:space:]]*$' "$dawn_args"

    if grep -Eq '^use_lld[[:space:]]*=' "$dawn_args"; then
        if ! grep -Eq '^use_lld[[:space:]]*=[[:space:]]*false[[:space:]]*$' "$dawn_args"; then
            echo "Dawn args explicitly set use_lld; refusing to override it" >&2
            return 2
        fi
    else
        printf '# cpp-atlas-mask-oracle: Apple ld cannot consume Dawn thin archives.\nuse_lld=false\n' >> "$dawn_args"
        dawn_lld_arg_added=1
    fi
}

usage() {
    echo "usage: tools/cpp-atlas-mask-oracle/build.sh [--preflight]" >&2
}

preflight() {
    local missing=0
    python3 "$script_dir/format_test.py"
    for command in git premake5 make naga python3; do
        if ! command -v "$command" >/dev/null; then
            echo "missing required tool: $command" >&2
            missing=1
        fi
    done
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
        echo "missing required tool: depot_tools ninja ($ninja_bin)" >&2
        missing=1
    fi
    if [[ ! -x "$gn_bin" ]]; then
        echo "missing required tool: depot_tools gn ($gn_bin)" >&2
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
mkdir -p "$injected_dir" "$(dirname "$output")"
cp "$script_dir/runtime-src/main.cpp" "$injected_dir/main.cpp"
applied=0
dawn_patch_applied=0
cleanup() {
    if (( dawn_lld_arg_added )); then
        sed -i.bak '/^# cpp-atlas-mask-oracle: Apple ld cannot consume Dawn thin archives\.$/d; /^use_lld=false$/d' "$dawn_args"
        rm -f "$dawn_args.bak"
    fi
    if (( dawn_warning_arg_added )); then
        sed -i.bak '/^# cpp-atlas-mask-oracle: Xcode 26 promotes legacy unsafe-buffer warnings\.$/d; /^treat_warnings_as_errors=false$/d' "$dawn_args"
        rm -f "$dawn_args.bak"
    fi
    if (( dawn_patch_applied )); then
        git -C "$dawn_dir" apply --reverse "$dawn_patch"
    fi
    if (( applied )); then
        git -C "$runtime" apply --reverse "$patch"
    fi
    rm -rf "$injected_dir"
}
trap cleanup EXIT INT TERM
git -C "$runtime" apply "$patch"
applied=1
if needs_xcode26_patch && dawn_patch_needed; then
    git -C "$dawn_dir" apply "$dawn_patch"
    dawn_patch_applied=1
fi
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

"$runtime/renderer/$build_out/rive_atlas_mask_oracle" "$output"
echo "atlas mask: $output"
