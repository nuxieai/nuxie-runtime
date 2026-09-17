#!/usr/bin/env bash
set -euo pipefail
proof_dir=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
repo_dir=$(CDPATH= cd -- "$proof_dir/../../.." && pwd)
video_profile=${VIDEO_PROOF_PROFILE:-dev}
case "$video_profile" in
  dev) video_profile_dir=debug ;;
  release-apple) video_profile_dir=release-apple ;;
  *) printf 'Unsupported VIDEO_PROOF_PROFILE: %s\n' "$video_profile" >&2; exit 1 ;;
esac
video_sdk=$(xcrun --sdk iphoneos --show-sdk-path)
IPHONEOS_DEPLOYMENT_TARGET=16.0 SDKROOT="$video_sdk" \
BINDGEN_EXTRA_CLANG_ARGS="--target=arm64-apple-ios16.0 --sysroot=$video_sdk" \
RUSTC="$(rustup which --toolchain stable rustc)" \
CARGO_TARGET_DIR="$repo_dir/target/video-qualification" \
"$(rustup which --toolchain stable cargo)" build --manifest-path "$repo_dir/Cargo.toml" -p video-qualification --lib --target aarch64-apple-ios --profile "$video_profile"
xcodegen generate --spec "$proof_dir/project.yml"
if [[ -n "${VIDEO_PROOF_DEVELOPMENT_TEAM:-}" ]]; then
  xcodebuild -project "$proof_dir/NuxieVideoProof.xcodeproj" -scheme NuxieVideoProof \
    -configuration Debug -destination 'generic/platform=iOS' \
    -derivedDataPath "$proof_dir/DerivedData" -allowProvisioningUpdates \
    "DEVELOPMENT_TEAM=$VIDEO_PROOF_DEVELOPMENT_TEAM" "NUX_VIDEO_PROOF_PROFILE=$video_profile_dir" build
fi
