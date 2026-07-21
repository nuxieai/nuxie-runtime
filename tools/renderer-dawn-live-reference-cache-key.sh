#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd "$(dirname "$0")" && pwd)"
repo_root="$(cd "$script_dir/.." && pwd)"
runtime_dir="${RIVE_RUNTIME_DIR:?RIVE_RUNTIME_DIR must point at the current rive-runtime checkout}"
runtime_ref="${RIVE_SAME_RUNNER_RUNTIME_REF:?RIVE_SAME_RUNNER_RUNTIME_REF must name the current rive-runtime revision}"
depot_tools_ref="${DEPOT_TOOLS_REF:?DEPOT_TOOLS_REF must name the pinned depot_tools revision}"
bootstrap="$repo_root/tools/renderer-dawn-live-reference-bootstrap.sh"
dependency_pins="$repo_root/tools/renderer-dawn-live-reference-dependencies.txt"

assignment() {
    local name="$1"
    local file="$2"
    awk -F '"' -v prefix="$name=" '$1 == prefix { print $2 }' "$file"
}

require_revision() {
    local name="$1"
    local revision="$2"
    if [[ ! "$revision" =~ ^[0-9a-f]{40}$ ]]; then
        echo "$name must resolve to one lowercase 40-character revision, got '$revision'" >&2
        exit 2
    fi
}

bootstrap_runtime_ref="$(assignment expected_runtime_revision "$bootstrap")"
bootstrap_dawn_ref="$(assignment expected_dawn_revision "$bootstrap")"
bootstrap_naga_version="$(assignment expected_naga_version "$bootstrap")"
make_dawn_ref="$(awk '$1 == "git" && $2 == "checkout" && $3 ~ /^[0-9a-f]{40}$/ { print $3 }' "$runtime_dir/renderer/make_dawn.sh")"
actual_runtime_ref="$(git -C "$runtime_dir" rev-parse HEAD)"

require_revision RIVE_SAME_RUNNER_RUNTIME_REF "$runtime_ref"
require_revision DEPOT_TOOLS_REF "$depot_tools_ref"
require_revision live-bootstrap-runtime-pin "$bootstrap_runtime_ref"
require_revision live-bootstrap-dawn-pin "$bootstrap_dawn_ref"
require_revision make-dawn-pin "$make_dawn_ref"
require_revision checked-out-runtime "$actual_runtime_ref"

if [[ "$runtime_ref" != "$bootstrap_runtime_ref" || "$runtime_ref" != "$actual_runtime_ref" ]]; then
    echo "live renderer runtime pin mismatch: workflow=$runtime_ref bootstrap=$bootstrap_runtime_ref checkout=$actual_runtime_ref" >&2
    exit 2
fi
if ! git -C "$runtime_dir" diff --quiet || ! git -C "$runtime_dir" diff --cached --quiet; then
    echo "live renderer runtime has tracked changes; refusing to key a non-pinned oracle" >&2
    exit 2
fi
if [[ "$bootstrap_dawn_ref" != "$make_dawn_ref" ]]; then
    echo "live renderer Dawn pin mismatch: bootstrap=$bootstrap_dawn_ref make_dawn=$make_dawn_ref" >&2
    exit 2
fi
if [[ ! -f "$dependency_pins" ]]; then
    echo "missing live renderer dependency pin manifest: $dependency_pins" >&2
    exit 2
fi

dependency_inputs=()
while read -r dependency_name dependency_revision trailing; do
    if [[ -z "$dependency_name" || "$dependency_name" == \#* ]]; then
        continue
    fi
    if [[ -n "${trailing:-}" || ! "$dependency_name" =~ ^[A-Za-z0-9._-]+$ ||
        ! "$dependency_revision" =~ ^[0-9a-f]{40}$ ]]; then
        echo "malformed live renderer dependency pin: $dependency_name ${dependency_revision:-} ${trailing:-}" >&2
        exit 2
    fi
    dependency_inputs+=("$dependency_name@$dependency_revision")
done < "$dependency_pins"
if (( ${#dependency_inputs[@]} == 0 )); then
    echo "live renderer dependency pin manifest is empty" >&2
    exit 2
fi

for command in cargo clang git glslangValidator make premake5 python3 rustc shasum spirv-opt xcodebuild xcrun xxd; do
    if ! command -v "$command" >/dev/null 2>&1; then
        echo "missing live reference cache-key input tool: $command" >&2
        exit 2
    fi
done

# These are every workspace input that can enter the uninstrumented FFI-only
# live replay. Historical atlas instrumentation is intentionally absent; its
# immutable 7c cache and references have a separate namespace and source set.
source_roots=(
    .cargo
    .github/workflows/ci.yml
    Cargo.lock
    Cargo.toml
    Makefile
    crates/nuxie-render-api
    crates/nuxie-render-stream
    crates/nuxie-renderer-ffi
    tools/cpp-atlas-mask-oracle/dawn-apple-visibility.patch
    tools/perf-compare
    tools/pixel-compare
    tools/renderer-dawn-live-reference-bootstrap.sh
    tools/renderer-dawn-live-reference-cache-key.sh
    tools/renderer-dawn-live-reference-dependencies.txt
    tools/renderer-replay
)

source_list="$(git -C "$repo_root" ls-files --cached --others --exclude-standard -- "${source_roots[@]}" | LC_ALL=C sort)"
if [[ -z "$source_list" ]]; then
    echo "live reference replay cache key resolved no workspace sources" >&2
    exit 2
fi

naga_path="${RIVE_DAWN_LIVE_NAGA:-$(command -v naga || true)}"
premake_path="$(command -v premake5)"
if [[ ! -x "$naga_path" || "$(basename "$naga_path")" != "naga" ]]; then
    echo "RIVE_DAWN_LIVE_NAGA must name an executable named naga: ${naga_path:-missing}" >&2
    exit 2
fi
naga_version="$("$naga_path" --version 2>&1 | awk 'NR == 1 { print $NF }')"
if [[ "$naga_version" != "$bootstrap_naga_version" ]]; then
    echo "unsupported Naga version at $naga_path: expected $bootstrap_naga_version, got ${naga_version:-unknown}" >&2
    exit 2
fi
glslang_path="$(command -v glslangValidator)"
spirv_opt_path="$(command -v spirv-opt)"
input_digest="$({
    printf '%s\n' \
        'renderer-dawn-live-reference-cache-v1-ffi-only' \
        'cargo-profile=release' \
        'cargo-features=no-default,perf-dawn' \
        'macosx-deployment-target=12.0' \
        "runtime-revision=$runtime_ref" \
        "dawn-revision=$bootstrap_dawn_ref" \
        "depot-tools-revision=$depot_tools_ref" \
        "host-arch=$(uname -m)" \
        "macos-version=$(sw_vers -productVersion)" \
        "macos-build=$(sw_vers -buildVersion)" \
        "macos-sdk-version=$(xcrun --sdk macosx --show-sdk-version)" \
        "macos-sdk-build=$(xcrun --sdk macosx --show-sdk-build-version)" \
        "glslang-sha256=$(shasum -a 256 "$glslang_path" | awk '{ print $1 }')" \
        "naga-sha256=$(shasum -a 256 "$naga_path" | awk '{ print $1 }')" \
        "premake5-sha256=$(shasum -a 256 "$premake_path" | awk '{ print $1 }')" \
        "spirv-opt-sha256=$(shasum -a 256 "$spirv_opt_path" | awk '{ print $1 }')"
    for dependency_input in "${dependency_inputs[@]}"; do
        printf 'dependency=%s\n' "$dependency_input"
    done
    xcodebuild -version | sed 's/^/xcode=/'
    rustc -Vv | sed 's/^/rustc=/'
    cargo -Vv | sed 's/^/cargo=/'
    clang --version | sed 's/^/clang=/'
    git --version | sed 's/^/git=/'
    "$glslang_path" --version | sed 's/^/glslang=/'
    make --version | sed 's/^/make=/'
    "$naga_path" --version | sed 's/^/naga=/'
    premake5 --version | sed 's/^/premake5=/'
    python3 --version | sed 's/^/python3=/'
    "$spirv_opt_path" --version 2>&1 | sed 's/^/spirv-opt=/'
    xxd -v 2>&1 | sed 's/^/xxd=/'
    while IFS= read -r path; do
        [[ -n "$path" ]] || continue
        file="$repo_root/$path"
        if [[ ! -f "$file" ]]; then
            echo "live cache-key source is not a regular file: $path" >&2
            exit 2
        fi
        mode="$(git -C "$repo_root" ls-files --stage -- "$path" | awk 'NR == 1 { print $1 }')"
        if [[ -z "$mode" ]]; then
            mode="untracked-$(stat -f '%Lp' "$file")"
        fi
        printf 'source=%s mode=%s sha256=%s\n' \
            "$path" "$mode" "$(shasum -a 256 "$file" | awk '{ print $1 }')"
    done <<< "$source_list"
} | shasum -a 256 | awk '{ print $1 }')"

key="renderer-dawn-live-reference-v1-ffi-only-$(uname -m)-$input_digest"
printf 'live reference runtime=%s dawn=%s inputs=%s\n' \
    "$runtime_ref" "$bootstrap_dawn_ref" "$input_digest" >&2
printf '%s\n' "$key"
