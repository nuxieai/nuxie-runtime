#!/usr/bin/env bash
set -euo pipefail
proof_dir=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
repo_dir=$(CDPATH= cd -- "$proof_dir/../../.." && pwd)
video_profile=${VIDEO_PROOF_PROFILE:-dev}
case "$video_profile" in
  dev) video_profile_dir=debug ;;
  release) video_profile_dir=release ;;
  *) printf 'Unsupported VIDEO_PROOF_PROFILE: %s\n' "$video_profile" >&2; exit 1 ;;
esac
video_sdk=${ANDROID_SDK_ROOT:-${ANDROID_HOME:?set ANDROID_HOME or ANDROID_SDK_ROOT}}
video_ndk=${ANDROID_NDK_HOME:-$video_sdk/ndk/29.0.14206865}
video_toolchain="$video_ndk/toolchains/llvm/prebuilt/darwin-x86_64"
video_buildtools="$video_sdk/build-tools/36.1.0"
video_android_jar="$video_sdk/platforms/android-36/android.jar"
video_output="$repo_dir/target/video-android-proof"
mkdir -p "$video_output/classes" "$video_output/dex" "$video_output/assets" "$video_output/lib/arm64-v8a"
cd "$repo_dir"
CC_aarch64_linux_android="$video_toolchain/bin/aarch64-linux-android23-clang" \
CXX_aarch64_linux_android="$video_toolchain/bin/aarch64-linux-android23-clang++" \
AR_aarch64_linux_android="$video_toolchain/bin/llvm-ar" \
CARGO_TARGET_AARCH64_LINUX_ANDROID_LINKER="$video_toolchain/bin/aarch64-linux-android23-clang" \
BINDGEN_EXTRA_CLANG_ARGS="--target=aarch64-linux-android23 --sysroot=$video_toolchain/sysroot" \
RUSTC="$(rustup which --toolchain stable rustc)" CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 \
"$(rustup which --toolchain stable cargo)" build -p video-qualification --lib --target aarch64-linux-android --profile "$video_profile"
javac -source 17 -target 17 -classpath "$video_android_jar" -d "$video_output/classes" \
  "$repo_dir/crates/nuxie-video-host/android/ai/nuxie/runtime/VideoPlayer.java" \
  "$proof_dir/ai/nuxie/videoqualification/MainActivity.java"
jar cf "$video_output/classes.jar" -C "$video_output/classes" .
"$video_buildtools/d8" --lib "$video_android_jar" --min-api 23 --output "$video_output/dex" "$video_output/classes.jar"
cp "$repo_dir/crates/nuxie-video-host/tests/fixtures/red-blue-720p.mp4" "$video_output/assets/"
cp "$repo_dir/crates/nuxie-video-host/tests/fixtures/red-blue-sync.mp4" "$video_output/assets/"
cp "$repo_dir/crates/nuxie-video-host/tests/fixtures/red-blue-audio.mp4" "$video_output/assets/"
cp "$repo_dir/target/aarch64-linux-android/$video_profile_dir/libvideo_qualification.so" "$video_output/lib/arm64-v8a/"
cp "$video_toolchain/sysroot/usr/lib/aarch64-linux-android/libc++_shared.so" "$video_output/lib/arm64-v8a/"
"$video_toolchain/bin/llvm-strip" --strip-debug "$video_output/lib/arm64-v8a/libvideo_qualification.so"
"$video_buildtools/aapt2" link -o "$video_output/unsigned.apk" -I "$video_android_jar" --manifest "$proof_dir/AndroidManifest.xml" -A "$video_output/assets"
(cd "$video_output/dex" && zip -q "$video_output/unsigned.apk" classes.dex)
(cd "$video_output" && zip -q -r unsigned.apk lib)
"$video_buildtools/zipalign" -f -p 4 "$video_output/unsigned.apk" "$video_output/aligned.apk"
# Disposable, fixture-only Android debug signing key; never a release key.
if [[ ! -f "$video_output/debug.keystore" ]]; then
  keytool -genkeypair -keystore "$video_output/debug.keystore" -storepass android -keypass android \
    -alias androiddebugkey -dname 'CN=Nuxie Video Proof' -keyalg RSA -keysize 2048 -validity 3650
fi
"$video_buildtools/apksigner" sign --ks "$video_output/debug.keystore" --ks-pass pass:android \
  --out "$video_output/video-proof.apk" "$video_output/aligned.apk"
printf 'Qualification APK: %s\n' "$video_output/video-proof.apk"
