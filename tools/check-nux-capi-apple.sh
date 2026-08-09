#!/usr/bin/env bash
set -euo pipefail

repo_dir=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
runtime_dir="${NUX_RUNTIME_DIR:-${RIVE_RUNTIME_DIR:-/Users/levi/dev/oss/rive-runtime}}"
fixture="$runtime_dir/tests/unit_tests/assets/in_band_asset.riv"
profile="${NUX_CAPI_APPLE_PROFILE:-dev}"
target_root="${CARGO_TARGET_DIR:-${repo_dir}/target}"
work_dir=$(mktemp -d)
trap 'rm -rf "$work_dir"' EXIT

if [[ ! -f "$fixture" ]]; then
    echo "missing Apple smoke fixture: $fixture" >&2
    exit 2
fi

# This is a compile/link matrix, not a packaging path. Its default dev archives
# stay in Cargo's ignored target directory and are only intermediates for the
# exact C and Swift smoke hosts below. Distribution sizing and XCFramework
# assembly use optimized artifacts in the release pipeline.

# The workspace may be entered with a Homebrew rustc, whose sysroot does not
# contain rustup-installed Apple cross targets. Pin all matrix work to one
# rustup toolchain unless the caller supplies an explicit Cargo binary.
if [[ -n "${CARGO_BIN:-}" ]]; then
    cargo_cmd=("$CARGO_BIN")
    rustc_cmd=("${RUSTC_BIN:-rustc}")
else
    cargo_cmd=(rustup run stable cargo)
    rustc_cmd=(rustup run stable rustc)
fi
rust_sysroot=$("${rustc_cmd[@]}" --print sysroot)
rustc_path="$rust_sysroot/bin/rustc"

if [[ "$profile" == dev ]]; then
    artifact_profile=debug
else
    artifact_profile="$profile"
fi

frameworks=(
    -framework CoreFoundation
    -framework CoreGraphics
    -framework ImageIO
    -framework QuartzCore
    -framework Metal
    -framework Foundation
    -framework Security
    -liconv
)

targets=(
    aarch64-apple-ios
    aarch64-apple-ios-sim
    x86_64-apple-ios
    aarch64-apple-darwin
    x86_64-apple-darwin
)

for target in "${targets[@]}"; do
    if [[ ! -d "$rust_sysroot/lib/rustlib/$target/lib" ]]; then
        echo "missing Rust target: $target" >&2
        exit 2
    fi
done

host_target=$("${rustc_cmd[@]}" -vV | awk '/^host:/ {print $2}')

for target in "${targets[@]}"; do
    case "$target" in
        aarch64-apple-ios)
            sdk=iphoneos
            clang_target=arm64-apple-ios15.0
            swift_target=arm64-apple-ios15.0
            ;;
        aarch64-apple-ios-sim)
            sdk=iphonesimulator
            clang_target=arm64-apple-ios15.0-simulator
            swift_target=arm64-apple-ios15.0-simulator
            ;;
        x86_64-apple-ios)
            sdk=iphonesimulator
            clang_target=x86_64-apple-ios15.0-simulator
            swift_target=x86_64-apple-ios15.0-simulator
            ;;
        aarch64-apple-darwin)
            sdk=macosx
            clang_target=arm64-apple-macos12.0
            swift_target=arm64-apple-macos12.0
            ;;
        x86_64-apple-darwin)
            sdk=macosx
            clang_target=x86_64-apple-macos12.0
            swift_target=x86_64-apple-macos12.0
            ;;
    esac

    echo "building nux-capi Apple extension for $target"
    IPHONEOS_DEPLOYMENT_TARGET=15.0 MACOSX_DEPLOYMENT_TARGET=12.0 \
        RUSTC="$rustc_path" \
        "${cargo_cmd[@]}" build --locked --manifest-path "$repo_dir/Cargo.toml" \
        -p nux-capi --no-default-features --features apple-metal,scripting \
        --profile "$profile" --target "$target"

    artifact_dir="$target_root/$target/$artifact_profile"
    archive="$artifact_dir/libnux_capi.a"
    test -f "$archive"
    archive_count=$(find "$artifact_dir" -maxdepth 1 -type f -name 'libnux*.a' | wc -l | tr -d ' ')
    if [[ "$archive_count" != 1 ]]; then
        echo "$target contains $archive_count Rust static archives; expected only libnux_capi.a" >&2
        exit 3
    fi
    if "${cargo_cmd[@]}" tree --locked --manifest-path "$repo_dir/Cargo.toml" -p nux-capi \
        --no-default-features --features apple-metal,scripting --target "$target" --edges normal \
        | grep -Eq '(^| )(?:nux-apple-runtime|nux-container|nuxie-apple-adapter|nuxie-product|nuxie-product-scripting|nuxie-project-data) v'; then
        echo "retired product/runtime package leaked into nux-capi distribution for $target" >&2
        exit 4
    fi

    sdk_path=$(xcrun --sdk "$sdk" --show-sdk-path)
    c_output="$work_dir/capi-metal-$target"
    swift_output="$work_dir/swift-capi-metal-$target"
    xcrun --sdk "$sdk" clang -std=c11 -Wall -Wextra -Werror \
        -target "$clang_target" -isysroot "$sdk_path" \
        -I "$repo_dir/crates/nux-capi/include" \
        "$repo_dir/crates/nux-capi/smoke/capi_metal_smoke.c" "$archive" \
        "${frameworks[@]}" -o "$c_output"
    xcrun --sdk "$sdk" swiftc -warnings-as-errors \
        -target "$swift_target" -sdk "$sdk_path" \
        -I "$repo_dir/crates/nux-capi/include" \
        "$repo_dir/crates/nux-capi/smoke/capi_metal_smoke.swift" "$archive" \
        -framework CoreFoundation -framework CoreGraphics -framework ImageIO \
        -framework QuartzCore -framework Metal -framework Foundation -framework Security \
        -Xlinker -liconv -o "$swift_output"

    if [[ "$target" == "$host_target" ]]; then
        "$c_output" "$fixture"
        "$swift_output" "$fixture"
        echo "$target: C and Swift link-and-render hosts executed natively"
    elif [[ "$target" == aarch64-apple-ios-sim ]] \
        && xcrun simctl list devices booted | grep -q '(Booted)'; then
        xcrun simctl spawn booted "$c_output" "$fixture"
        xcrun simctl spawn booted "$swift_output" "$fixture"
        echo "$target: C and Swift link-and-render hosts executed in the booted iOS simulator"
    else
        echo "$target: C and Swift render hosts compiled and statically linked; execution unavailable without a matching connected/booted runtime or host architecture"
    fi
    printf '%s %s cross-link archive bytes (not distribution): %s\n' \
        "$target" "$profile" "$(stat -f %z "$archive")"
done

NUX_CAPI_FEATURES=apple-metal,scripting "$repo_dir/tools/check-nux-capi-exports.sh"
"$repo_dir/tools/check-nux-capi-surface.py"
echo "nux-capi Apple five-slice C/Swift link and executable render smoke ok"
