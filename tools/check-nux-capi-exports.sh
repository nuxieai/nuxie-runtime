#!/bin/sh
set -eu

repo_dir=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
features="${NUX_CAPI_FEATURES:-}"
work_dir=$(mktemp -d)
trap 'rm -rf "$work_dir"' EXIT

expected="$work_dir/expected.txt"
case "$features" in
    "")
        cp "$repo_dir/crates/nux-capi/exports-v3-portable.txt" "$expected"
        feature_args=""
        ;;
    scripting)
        cp "$repo_dir/crates/nux-capi/exports-v3-portable.txt" "$expected"
        feature_args="--features scripting"
        ;;
    apple-metal|apple-metal,scripting)
        cat \
            "$repo_dir/crates/nux-capi/exports-v3-portable.txt" \
            "$repo_dir/crates/nux-capi/exports-v3-apple-metal-extension.txt" | \
            LC_ALL=C sort -u > "$expected"
        feature_args="--features $features"
        ;;
    android-vulkan|android-vulkan,scripting)
        cat \
            "$repo_dir/crates/nux-capi/exports-v3-portable.txt" \
            "$repo_dir/crates/nux-capi/exports-v3-android-vulkan-extension.txt" | \
            LC_ALL=C sort -u > "$expected"
        feature_args="--features $features"
        ;;
    android-vulkan,scripting,android-authored-wgsl)
        cat \
            "$repo_dir/crates/nux-capi/exports-v3-portable.txt" \
            "$repo_dir/crates/nux-capi/exports-v3-android-vulkan-extension.txt" \
            "$repo_dir/crates/nux-capi/exports-v3-android-authored-wgsl-extension.txt" | \
            LC_ALL=C sort -u > "$expected"
        feature_args="--features $features"
        ;;
    *)
        echo "unsupported NUX_CAPI_FEATURES value: $features" >&2
        exit 2
        ;;
esac

# shellcheck disable=SC2086 # an empty or one-feature Cargo argument pair
cargo build --quiet --manifest-path "$repo_dir/Cargo.toml" -p nux-capi $feature_args

case "$(uname -s)" in
    Darwin)
        dynamic_artifact="$repo_dir/target/debug/libnux_capi.dylib"
        nm_defined() { nm -gU "$1" 2>/dev/null; }
        ;;
    *)
        dynamic_artifact="$repo_dir/target/debug/libnux_capi.so"
        nm_defined() { nm -g --defined-only "$1" 2>/dev/null; }
        ;;
esac

for artifact in "$dynamic_artifact" "$repo_dir/target/debug/libnux_capi.a"
do
    actual="$work_dir/$(basename "$artifact").txt"
    nm_defined "$artifact" | awk '{print $NF}' | sed 's/^_//' | \
        awk '/^nux_[A-Za-z0-9_]+$/ { print }' | sort -u > "$actual"
    diff -u "$expected" "$actual"
done

header_actual="$work_dir/header.txt"
grep -Eo 'nux_[A-Za-z0-9_]+[[:space:]]*\(' \
    "$repo_dir/crates/nux-capi/include/nux_capi.generated.h" | \
    sed -E 's/[[:space:]]*\($//' | sort -u > "$header_actual"
# cbindgen retains every feature-gated declaration in the generated header.
# Compare only the selected extension while keeping each family exact enough
# that a new symbol fails its own feature inventory closed.
asset_hooks_extension='^nux_file_import_(configured|with_assets)$'
apple_extension='^nux_renderer_(copy_metal_device|detach|free|info|new_metal|reattach|render_player|reset_player_domain|resize)$'
android_extension='^(nux_android_vulkan_frame_|nux_renderer_(android_vulkan_|new_android_vulkan$))'
android_authored_wgsl_extension='^nux_file_import_configured_with_trusted_wgsl$'
case "$features" in
    apple-metal|apple-metal,scripting)
        grep -Ev "$android_extension|$android_authored_wgsl_extension" "$header_actual" > "$work_dir/header-selected.txt"
        ;;
    android-vulkan|android-vulkan,scripting)
        grep -Ev "$apple_extension|$android_authored_wgsl_extension" "$header_actual" > "$work_dir/header-selected.txt"
        ;;
    android-vulkan,scripting,android-authored-wgsl)
        grep -Ev "$apple_extension" "$header_actual" > "$work_dir/header-selected.txt"
        ;;
    *)
        grep -Ev "$asset_hooks_extension|$apple_extension|$android_extension|$android_authored_wgsl_extension" \
            "$header_actual" > "$work_dir/header-selected.txt"
        ;;
esac
header_actual="$work_dir/header-selected.txt"
diff -u "$expected" "$header_actual"

echo "nux-capi ABI-v3 ${features:-portable} export inventory ok"
