#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd "$(dirname "$0")" && pwd)"
repo_root="$(cd "$script_dir/.." && pwd)"
runtime_dir="${RIVE_RUNTIME_DIR:?RIVE_RUNTIME_DIR must point at the pinned rive-runtime checkout}"
runtime_ref="${RIVE_RUNTIME_REF:?RIVE_RUNTIME_REF must name the pinned rive-runtime revision}"
depot_tools_ref="${DEPOT_TOOLS_REF:?DEPOT_TOOLS_REF must name the pinned depot_tools revision}"
oracle_build="$repo_root/tools/cpp-atlas-mask-oracle/build.sh"

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

oracle_runtime_ref="$(assignment expected_runtime_revision "$oracle_build")"
oracle_dawn_ref="$(assignment expected_dawn_revision "$oracle_build")"
make_dawn_ref="$(awk '$1 == "git" && $2 == "checkout" && $3 ~ /^[0-9a-f]{40}$/ { print $3 }' "$runtime_dir/renderer/make_dawn.sh")"
actual_runtime_ref="$(git -C "$runtime_dir" rev-parse HEAD)"

require_revision RIVE_RUNTIME_REF "$runtime_ref"
require_revision DEPOT_TOOLS_REF "$depot_tools_ref"
require_revision cpp-atlas-runtime-pin "$oracle_runtime_ref"
require_revision cpp-atlas-dawn-pin "$oracle_dawn_ref"
require_revision make-dawn-pin "$make_dawn_ref"
require_revision checked-out-runtime "$actual_runtime_ref"

if [[ "$runtime_ref" != "$oracle_runtime_ref" || "$runtime_ref" != "$actual_runtime_ref" ]]; then
    echo "rive-runtime pin mismatch: workflow=$runtime_ref oracle=$oracle_runtime_ref checkout=$actual_runtime_ref" >&2
    exit 2
fi
if ! git -C "$runtime_dir" diff --quiet || ! git -C "$runtime_dir" diff --cached --quiet; then
    echo "rive-runtime checkout has tracked changes; refusing to key a non-pinned oracle" >&2
    exit 2
fi
if [[ "$oracle_dawn_ref" != "$make_dawn_ref" ]]; then
    echo "Dawn pin mismatch: oracle=$oracle_dawn_ref make_dawn=$make_dawn_ref" >&2
    exit 2
fi

for command in cargo clang git make premake5 python3 rustc shasum xcodebuild xcrun; do
    if ! command -v "$command" >/dev/null 2>&1; then
        echo "missing cache-key input tool: $command" >&2
        exit 2
    fi
done

# These are every workspace input that can enter the FFI-only release reference
# executable. The Rust renderer and vendored wgpu are intentionally absent: the
# reference is built without the default `rust-wgpu` feature. The pinned Rive
# and Dawn trees are represented by their exact revisions above; Cargo registry
# dependencies are represented by Cargo.lock.
source_roots=(
    .cargo
    .github/workflows/ci.yml
    Cargo.lock
    Cargo.toml
    Makefile
    crates/nuxie-render-api
    crates/nuxie-render-stream
    crates/nuxie-renderer-ffi
    tools/cpp-atlas-mask-oracle
    tools/perf-compare
    tools/pixel-compare
    tools/renderer-dawn-reference-bootstrap.sh
    tools/renderer-dawn-reference-cache-key.sh
    tools/renderer-replay
)

source_list="$(git -C "$repo_root" ls-files --cached --others --exclude-standard -- "${source_roots[@]}" | LC_ALL=C sort)"
if [[ -z "$source_list" ]]; then
    echo "reference replay cache key resolved no workspace sources" >&2
    exit 2
fi

naga_path="${RIVE_ATLAS_MASK_NAGA:-$HOME/.cargo/bin/naga}"
premake_path="$(command -v premake5)"
if [[ ! -x "$naga_path" || "$(basename "$naga_path")" != "naga" ]]; then
    echo "RIVE_ATLAS_MASK_NAGA must name an executable named naga: $naga_path" >&2
    exit 2
fi
input_digest="$({
    printf '%s\n' \
        'renderer-dawn-reference-cache-v2-ffi-only' \
        'cargo-profile=release' \
        'cargo-features=no-default,perf-dawn' \
        'macosx-deployment-target=12.0' \
        "runtime-revision=$runtime_ref" \
        "dawn-revision=$oracle_dawn_ref" \
        "depot-tools-revision=$depot_tools_ref" \
        "host-arch=$(uname -m)" \
        "macos-version=$(sw_vers -productVersion)" \
        "macos-build=$(sw_vers -buildVersion)" \
        "macos-sdk-version=$(xcrun --sdk macosx --show-sdk-version)" \
        "macos-sdk-build=$(xcrun --sdk macosx --show-sdk-build-version)" \
        "naga-sha256=$(shasum -a 256 "$naga_path" | awk '{ print $1 }')" \
        "premake5-sha256=$(shasum -a 256 "$premake_path" | awk '{ print $1 }')"
    xcodebuild -version | sed 's/^/xcode=/'
    rustc -Vv | sed 's/^/rustc=/'
    cargo -Vv | sed 's/^/cargo=/'
    clang --version | sed 's/^/clang=/'
    git --version | sed 's/^/git=/'
    make --version | sed 's/^/make=/'
    "$naga_path" --version | sed 's/^/naga=/'
    premake5 --version | sed 's/^/premake5=/'
    python3 --version | sed 's/^/python3=/'
    while IFS= read -r path; do
        [[ -n "$path" ]] || continue
        file="$repo_root/$path"
        if [[ ! -f "$file" ]]; then
            echo "cache-key source is not a regular file: $path" >&2
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

key="renderer-dawn-reference-v2-ffi-only-$(uname -m)-$input_digest"
printf 'reference runtime=%s dawn=%s inputs=%s\n' \
    "$runtime_ref" "$oracle_dawn_ref" "$input_digest" >&2
printf '%s\n' "$key"
