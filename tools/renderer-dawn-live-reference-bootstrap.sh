#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd "$(dirname "$0")" && pwd)"
repo_root="$(cd "$script_dir/.." && pwd)"
runtime_dir="${RIVE_RUNTIME_DIR:?RIVE_RUNTIME_DIR must point at the current rive-runtime checkout}"
jobs="${RIVE_DAWN_LIVE_JOBS:-2}"
dawn_dir="$runtime_dir/renderer/dependencies/dawn"
dawn_patch="$repo_root/tools/cpp-atlas-mask-oracle/dawn-apple-visibility.patch"
dependency_pins="$repo_root/tools/renderer-dawn-live-reference-dependencies.txt"
build_out="out/cpp-atlas-mask-oracle"
expected_runtime_revision="d788e8ec6e8b598526607d6a1e8818e8b637b60c"
expected_dawn_revision="211333b2e3e429c3508f25c81c547f602adf448c"
expected_naga_version="30.0.0"

for command in gclient git glslangValidator gn make premake5 python3 spirv-opt xcrun xxd; do
    if ! command -v "$command" >/dev/null 2>&1; then
        echo "missing live Dawn bootstrap tool: $command" >&2
        exit 2
    fi
done
if [[ ! "$jobs" =~ ^[1-9][0-9]*$ ]]; then
    echo "RIVE_DAWN_LIVE_JOBS must be a positive integer, got '$jobs'" >&2
    exit 2
fi
naga_bin="${RIVE_DAWN_LIVE_NAGA:-$(command -v naga || true)}"
if [[ ! -x "$naga_bin" || "$(basename "$naga_bin")" != "naga" ]]; then
    echo "RIVE_DAWN_LIVE_NAGA must name an executable named naga: ${naga_bin:-missing}" >&2
    exit 2
fi
naga_output="$("$naga_bin" --version 2>&1)"
naga_version="$(awk 'NR == 1 { print $NF }' <<< "$naga_output")"
if [[ "$naga_version" != "$expected_naga_version" ]]; then
    echo "unsupported Naga version at $naga_bin: expected $expected_naga_version, got ${naga_version:-unknown}" >&2
    exit 2
fi
if ! git -C "$runtime_dir" rev-parse --is-inside-work-tree >/dev/null 2>&1; then
    echo "missing RIVE_RUNTIME_DIR git checkout: $runtime_dir" >&2
    exit 2
fi
if [[ ! -f "$dependency_pins" ]]; then
    echo "missing live renderer dependency pin manifest: $dependency_pins" >&2
    exit 2
fi

actual_runtime_revision="$(git -C "$runtime_dir" rev-parse HEAD)"
if [[ "$actual_runtime_revision" != "$expected_runtime_revision" ]]; then
    echo "live renderer runtime drifted: expected $expected_runtime_revision, got $actual_runtime_revision" >&2
    exit 2
fi
if ! git -C "$runtime_dir" diff --quiet || ! git -C "$runtime_dir" diff --cached --quiet; then
    echo "live renderer runtime has tracked changes; refusing to build a non-pinned oracle" >&2
    exit 2
fi

make_dawn_revision="$(awk '$1 == "git" && $2 == "checkout" && $3 ~ /^[0-9a-f]{40}$/ { print $3 }' "$runtime_dir/renderer/make_dawn.sh")"
if [[ "$make_dawn_revision" != "$expected_dawn_revision" ]]; then
    echo "current runtime Dawn pin drifted: expected $expected_dawn_revision, got ${make_dawn_revision:-missing}" >&2
    exit 2
fi

mkdir -p "$runtime_dir/renderer/dependencies"
if [[ ! -d "$dawn_dir/.git" ]]; then
    if [[ -e "$dawn_dir" ]]; then
        echo "Dawn destination exists but is not a git checkout: $dawn_dir" >&2
        exit 2
    fi
    git clone --no-checkout https://dawn.googlesource.com/dawn "$dawn_dir"
    git -C "$dawn_dir" checkout --detach "$expected_dawn_revision"
else
    actual_dawn_revision="$(git -C "$dawn_dir" rev-parse HEAD)"
    if [[ "$actual_dawn_revision" != "$expected_dawn_revision" ]]; then
        echo "existing Dawn checkout is not pinned: expected $expected_dawn_revision, got $actual_dawn_revision" >&2
        exit 2
    fi
fi
if ! git -C "$dawn_dir" diff --quiet || ! git -C "$dawn_dir" diff --cached --quiet; then
    echo "Dawn checkout has tracked changes; refusing to build a non-pinned oracle" >&2
    exit 2
fi

cp "$dawn_dir/scripts/standalone.gclient" "$dawn_dir/.gclient"
(
    cd "$dawn_dir"
    DEPOT_TOOLS_UPDATE=0 gclient sync -f -D
)

ninja_bin="$dawn_dir/third_party/ninja/ninja"
if [[ ! -x "$ninja_bin" ]]; then
    echo "pinned Dawn ninja is missing after gclient sync: $ninja_bin" >&2
    exit 2
fi

dawn_patch_applied=0
cleanup() {
    local status=$?
    trap - EXIT INT TERM
    set +e
    if (( dawn_patch_applied )); then
        if ! git -C "$dawn_dir" apply --reverse "$dawn_patch"; then
            echo "failed to reverse Dawn Apple visibility patch" >&2
            status=1
        fi
    fi
    exit "$status"
}
trap cleanup EXIT
trap 'exit 130' INT
trap 'exit 143' TERM

dawn_args="is_debug=false dawn_complete_static_libs=true use_custom_libcxx=false dawn_use_swiftshader=false angle_enable_swiftshader=false"
if [[ "$(uname -s)" == "Darwin" ]]; then
    if git -C "$dawn_dir" apply --check "$dawn_patch"; then
        git -C "$dawn_dir" apply "$dawn_patch"
        dawn_patch_applied=1
    elif ! git -C "$dawn_dir" apply --reverse --check "$dawn_patch"; then
        echo "Dawn Apple visibility patch does not match pinned Dawn" >&2
        exit 2
    fi
    dawn_args="$dawn_args treat_warnings_as_errors=false use_lld=false"
fi

(
    cd "$dawn_dir"
    gn gen --args="$dawn_args" out/release
    "$ninja_bin" -C out/release -j"$jobs" webgpu_dawn_static cpp proc_static
)

(
    cd "$runtime_dir/renderer"
    DEPENDENCIES="$runtime_dir/dependencies" \
    PATH="$(dirname "$naga_bin"):$PATH" \
    PREMAKE_PATH="$runtime_dir/build${PREMAKE_PATH:+:$PREMAKE_PATH}" \
        premake5 gmake2 --file=premake5.lua --config=release \
        --out="$build_out" --with-dawn --no-lto
)

dependency_count=0
while read -r dependency_name dependency_revision trailing; do
    if [[ -z "$dependency_name" || "$dependency_name" == \#* ]]; then
        continue
    fi
    if [[ -n "${trailing:-}" || ! "$dependency_name" =~ ^[A-Za-z0-9._-]+$ ||
        ! "$dependency_revision" =~ ^[0-9a-f]{40}$ ]]; then
        echo "malformed live renderer dependency pin: $dependency_name ${dependency_revision:-} ${trailing:-}" >&2
        exit 2
    fi
    dependency_dir="$runtime_dir/dependencies/$dependency_name"
    if [[ ! -d "$dependency_dir/.git" ]]; then
        echo "missing live renderer dependency checkout: $dependency_name" >&2
        exit 2
    fi
    actual_dependency_revision="$(git -C "$dependency_dir" rev-parse HEAD)"
    if [[ "$actual_dependency_revision" != "$dependency_revision" ]]; then
        echo "live renderer dependency drifted: $dependency_name expected $dependency_revision, got $actual_dependency_revision" >&2
        exit 2
    fi
    if [[ -n "$(git -C "$dependency_dir" status --porcelain --untracked-files=all)" ]]; then
        echo "live renderer dependency has local changes: $dependency_name" >&2
        exit 2
    fi
    dependency_count=$((dependency_count + 1))
done < "$dependency_pins"
if (( dependency_count == 0 )); then
    echo "live renderer dependency pin manifest is empty" >&2
    exit 2
fi

(
    cd "$runtime_dir/renderer"
    make -C "$build_out" -j"$jobs" \
        rive rive_pls_renderer rive_decoders libpng zlib libjpeg libwebp \
        rive_harfbuzz rive_sheenbidi rive_yoga
)

for archive in \
    librive.a \
    librive_pls_renderer.a \
    librive_decoders.a \
    liblibpng.a \
    libzlib.a \
    liblibjpeg.a \
    liblibwebp.a \
    librive_harfbuzz.a \
    librive_sheenbidi.a \
    librive_yoga.a; do
    if [[ ! -f "$runtime_dir/renderer/$build_out/$archive" ]]; then
        echo "live renderer archive is missing after build: $archive" >&2
        exit 2
    fi
done

echo "built current-runtime Dawn archives at $runtime_dir/renderer/$build_out"
