#!/bin/sh
set -eu

repo_dir=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
expected="$repo_dir/crates/nux-capi/exports-v3.txt"
work_dir=$(mktemp -d)
trap 'rm -rf "$work_dir"' EXIT

cargo build --quiet --manifest-path "$repo_dir/Cargo.toml" -p nux-capi

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
diff -u "$expected" "$header_actual"

echo "nux-capi ABI-v3 export inventory ok"
