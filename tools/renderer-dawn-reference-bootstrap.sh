#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd "$(dirname "$0")" && pwd)"
repo_root="$(cd "$script_dir/.." && pwd)"
runtime_dir="${RIVE_RUNTIME_DIR:?RIVE_RUNTIME_DIR must point at the pinned rive-runtime checkout}"
jobs="${RIVE_ATLAS_MASK_JOBS:-2}"
oracle_build="$repo_root/tools/cpp-atlas-mask-oracle/build.sh"
dawn_dir="$runtime_dir/renderer/dependencies/dawn"
naga_bin="${RIVE_ATLAS_MASK_NAGA:-$HOME/.cargo/bin/naga}"

for command in gclient git gn; do
    if ! command -v "$command" >/dev/null 2>&1; then
        echo "missing Dawn bootstrap tool: $command" >&2
        exit 2
    fi
done
if [[ ! "$jobs" =~ ^[1-9][0-9]*$ ]]; then
    echo "RIVE_ATLAS_MASK_JOBS must be a positive integer, got '$jobs'" >&2
    exit 2
fi
if [[ ! -x "$naga_bin" || "$(basename "$naga_bin")" != "naga" ]]; then
    echo "RIVE_ATLAS_MASK_NAGA must name an executable named naga: $naga_bin" >&2
    exit 2
fi

runtime_ref="$(awk -F '"' '$1 == "expected_runtime_revision=" { print $2 }' "$oracle_build")"
dawn_ref="$(awk -F '"' '$1 == "expected_dawn_revision=" { print $2 }' "$oracle_build")"
actual_runtime_ref="$(git -C "$runtime_dir" rev-parse HEAD)"
if [[ "$actual_runtime_ref" != "$runtime_ref" ]]; then
    echo "rive-runtime checkout drifted: expected $runtime_ref, got $actual_runtime_ref" >&2
    exit 2
fi
if ! git -C "$runtime_dir" diff --quiet || ! git -C "$runtime_dir" diff --cached --quiet; then
    echo "rive-runtime checkout has tracked changes; refusing to bootstrap a non-pinned oracle" >&2
    exit 2
fi

mkdir -p "$runtime_dir/renderer/dependencies"
if [[ ! -d "$dawn_dir/.git" ]]; then
    if [[ -e "$dawn_dir" ]]; then
        echo "Dawn destination exists but is not a git checkout: $dawn_dir" >&2
        exit 2
    fi
    git clone --no-checkout https://dawn.googlesource.com/dawn "$dawn_dir"
    git -C "$dawn_dir" checkout --detach "$dawn_ref"
else
    actual_dawn_ref="$(git -C "$dawn_dir" rev-parse HEAD)"
    if [[ "$actual_dawn_ref" != "$dawn_ref" ]]; then
        echo "existing Dawn checkout is not pinned: expected $dawn_ref, got $actual_dawn_ref" >&2
        exit 2
    fi
fi
if ! git -C "$dawn_dir" diff --quiet || ! git -C "$dawn_dir" diff --cached --quiet; then
    echo "Dawn checkout has tracked changes; refusing to bootstrap a non-pinned oracle" >&2
    exit 2
fi

cp "$dawn_dir/scripts/standalone.gclient" "$dawn_dir/.gclient"
(
    cd "$dawn_dir"
    DEPOT_TOOLS_UPDATE=0 gclient sync -f -D
    gn gen --args='is_debug=false dawn_complete_static_libs=true use_custom_libcxx=false dawn_use_swiftshader=false angle_enable_swiftshader=false' out/release
)

RIVE_RUNTIME_DIR="$runtime_dir" RIVE_ATLAS_MASK_JOBS="$jobs" \
    RIVE_ATLAS_MASK_NAGA="$naga_bin" \
    "$oracle_build" --build-only
