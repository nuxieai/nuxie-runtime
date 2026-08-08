#!/bin/sh
set -eu

repo_dir=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
features="${NUX_CAPI_FEATURES:-}"
if [ "$features" = "apple-metal" ]; then
    expected="$repo_dir/crates/nux-capi/exports-v3-apple-metal.txt"
    feature_args="--features apple-metal"
else
    expected="$repo_dir/crates/nux-capi/exports-v3.txt"
    feature_args=""
fi
work_dir=$(mktemp -d)
trap 'rm -rf "$work_dir"' EXIT

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
if [ "$features" != "apple-metal" ]; then
    # cbindgen retains feature-gated declarations in the generated header. The
    # portable ABI excludes the Apple renderer family and this one Apple-only
    # import entry point; keep the latter exact so future imports fail closed.
    grep -Ev '^(nux_renderer_|nux_file_import_(configured|with_apple_assets)$)' \
        "$header_actual" > "$work_dir/header-portable.txt"
    header_actual="$work_dir/header-portable.txt"
fi
diff -u "$expected" "$header_actual"

echo "nux-capi ABI-v3 ${features:-portable} export inventory ok"
