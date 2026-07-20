#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd -P "$(dirname "${BASH_SOURCE[0]}")" && pwd -P)"
repo_root="$(cd -P "${script_dir}/.." && pwd -P)"
if [[ $# -gt 0 ]]; then
    requested_output_root="$1"
else
    requested_output_root="${repo_root}/target/apple-runtime"
fi
if [[ -z "${requested_output_root}" ]]; then
    echo "refusing unsafe output path: empty path" >&2
    exit 2
fi
mkdir -p "${requested_output_root}"
output_root="$(cd -P "${requested_output_root}" && pwd -P)"
case "${output_root}" in
    /|"${repo_root}"|"")
        echo "refusing unsafe output path: ${output_root}" >&2
        exit 2
        ;;
esac

profile="${NUX_APPLE_PROFILE:-release-apple}"
deployment_target="${NUX_APPLE_DEPLOYMENT_TARGET:-15.0}"
rust_toolchain="${NUX_APPLE_RUST_TOOLCHAIN:-1.94.1}"
rust_cargo="$(rustup which --toolchain "${rust_toolchain}" cargo)"
rust_compiler="$(rustup which --toolchain "${rust_toolchain}" rustc)"
rust_host="$("${rust_compiler}" -vV | sed -n 's/^host: //p')"
rust_sysroot="$("${rust_compiler}" --print sysroot)"
rust_llvm_nm="${rust_sysroot}/lib/rustlib/${rust_host}/bin/llvm-nm"
runtime_revision="${NUX_RUNTIME_SOURCE_REVISION:-}"
xcode_version="$(xcodebuild -version | sed -n 's/^Xcode //p')"
xcode_build="$(xcodebuild -version | sed -n 's/^Build version //p')"
iphoneos_sdk_version="$(xcrun --sdk iphoneos --show-sdk-version)"
iphoneos_sdk_build="$(xcrun --sdk iphoneos --show-sdk-build-version)"
iphonesimulator_sdk_version="$(xcrun --sdk iphonesimulator --show-sdk-version)"
iphonesimulator_sdk_build="$(xcrun --sdk iphonesimulator --show-sdk-build-version)"
build_root="${output_root}/build"
cargo_target_dir="${build_root}/cargo"
headers_dir="${build_root}/Headers"
simulator_dir="${build_root}/simulator"
xcframework_path="${output_root}/NuxieRuntime.xcframework"
archive_path="${output_root}/NuxieRuntime.xcframework.zip"
metadata_path="${output_root}/artifact.json"
license_path="${xcframework_path}/LICENSE"
third_party_notices_path="${xcframework_path}/THIRD_PARTY_NOTICES.md"
luaur_version="$(
    awk '
        $0 == "name = \"luaur-vm\"" { found = 1; next }
        found && /^version = / {
            value = $0
            sub(/^version = \"/, "", value)
            sub(/\"$/, "", value)
            print value
            exit
        }
        found && /^\[\[package\]\]/ { exit 1 }
    ' "${repo_root}/Cargo.lock"
)"

if [[ -z "${luaur_version}" ]]; then
    echo "cannot determine the pinned luaur-vm version from Cargo.lock" >&2
    exit 10
fi

if [[ ! -x "${rust_llvm_nm}" ]]; then
    echo "missing llvm-nm for Rust toolchain ${rust_toolchain}" >&2
    echo "install it with: rustup component add --toolchain ${rust_toolchain} llvm-tools" >&2
    exit 9
fi

phase() {
    printf '\n==> %s\n' "$1"
}

report_disk() {
    local available_kib
    available_kib="$(df -Pk "${output_root}" 2>/dev/null | awk 'NR == 2 { print $4 }' || true)"
    printf 'disk: available=%s KiB\n' "${available_kib:-unknown}"
}

if [[ -n "${NUX_APPLE_XCODE_VERSION:-}" && "${xcode_version}" != "${NUX_APPLE_XCODE_VERSION}" ]]; then
    echo "Xcode version ${xcode_version} does not match required ${NUX_APPLE_XCODE_VERSION}" >&2
    exit 6
fi
if [[ -n "${NUX_APPLE_XCODE_BUILD:-}" && "${xcode_build}" != "${NUX_APPLE_XCODE_BUILD}" ]]; then
    echo "Xcode build ${xcode_build} does not match required ${NUX_APPLE_XCODE_BUILD}" >&2
    exit 7
fi

if git -C "${repo_root}" rev-parse --is-inside-work-tree >/dev/null 2>&1; then
    if [[ -z "${runtime_revision}" ]]; then
        runtime_revision="$(git -C "${repo_root}" rev-parse --verify HEAD)"
    fi
    if [[ -n "$(git -C "${repo_root}" status --porcelain --untracked-files=all)" ]]; then
        if [[ "${NUX_APPLE_ALLOW_DIRTY:-0}" != "1" ]]; then
            echo "refusing to package a dirty runtime tree" >&2
            echo "commit the runtime or set NUX_APPLE_ALLOW_DIRTY=1 for a local prototype" >&2
            exit 4
        fi
        runtime_revision="${runtime_revision}-dirty"
    fi
elif [[ -z "${runtime_revision}" ]]; then
    echo "NUX_RUNTIME_SOURCE_REVISION is required outside a Git worktree" >&2
    exit 8
fi
if [[ ! "${runtime_revision}" =~ ^[A-Za-z0-9._+-]+$ ]]; then
    echo "runtime source revision is not metadata-safe: ${runtime_revision}" >&2
    exit 5
fi

# Keep Cargo's target directory as an incremental build cache, but recreate every
# directory that is copied into the published artifact. Without this boundary,
# headers and libraries from an older packaging layout can silently survive into
# a later XCFramework.
rm -rf \
    "${headers_dir}" \
    "${simulator_dir}" \
    "${xcframework_path}" \
    "${archive_path}" \
    "${metadata_path}"
mkdir -p "${output_root}" "${build_root}" "${headers_dir}" "${simulator_dir}"
phase "Prepare deterministic Apple runtime output"
report_disk

targets=(
    aarch64-apple-ios
    aarch64-apple-ios-sim
    x86_64-apple-ios
)

for target in "${targets[@]}"; do
    if ! rustup target list --toolchain "${rust_toolchain}" --installed | grep -qx "${target}"; then
        echo "missing Rust target ${target} for toolchain ${rust_toolchain}" >&2
        echo "install it with: rustup target add --toolchain ${rust_toolchain} ${target}" >&2
        exit 3
    fi
    phase "Build Apple runtime for ${target}"
    IPHONEOS_DEPLOYMENT_TARGET="${deployment_target}" \
    NUX_RUNTIME_BUILD_PROFILE="${profile}" \
    NUX_RUNTIME_SOURCE_REVISION="${runtime_revision}" \
    CARGO_TARGET_DIR="${cargo_target_dir}" \
    RUSTC="${rust_compiler}" \
        "${rust_cargo}" build \
            --manifest-path "${repo_root}/Cargo.toml" \
            --locked \
            --package nux-apple-runtime \
            --no-default-features \
            --features apple-product \
            --profile "${profile}" \
            --target "${target}"
    report_disk
done

device_library="${cargo_target_dir}/aarch64-apple-ios/${profile}/libnux_apple_runtime.a"
arm_simulator_library="${cargo_target_dir}/aarch64-apple-ios-sim/${profile}/libnux_apple_runtime.a"
intel_simulator_library="${cargo_target_dir}/x86_64-apple-ios/${profile}/libnux_apple_runtime.a"
simulator_library="${simulator_dir}/libnux_apple_runtime.a"

phase "Create the universal simulator library"
lipo -create \
    "${arm_simulator_library}" \
    "${intel_simulator_library}" \
    -output "${simulator_library}"

cp "${repo_root}/crates/nux-apple-runtime/include/nux_runtime.h" "${headers_dir}/"
cp "${repo_root}/crates/nux-apple-runtime/include/nux_runtime.generated.h" "${headers_dir}/"
cp "${repo_root}/crates/nux-apple-runtime/include/module.modulemap" "${headers_dir}/"

phase "Create the XCFramework"
xcodebuild -create-xcframework \
    -library "${device_library}" \
    -headers "${headers_dir}" \
    -library "${simulator_library}" \
    -headers "${headers_dir}" \
    -output "${xcframework_path}"

phase "Attach license notices"
cp "${repo_root}/LICENSE" "${license_path}"
cp "${repo_root}/THIRD_PARTY_NOTICES.md" "${third_party_notices_path}"
report_disk

phase "Archive the XCFramework"
ditto -c -k --sequesterRsrc --keepParent "${xcframework_path}" "${archive_path}"
checksum="$(swift package compute-checksum "${archive_path}")"
report_disk

phase "Write artifact provenance"
printf '{\n  "schemaVersion": 1,\n  "abiMajor": %s,\n  "abiMinor": %s,\n  "runtimeVersion": "%s",\n  "luaurVersion": "%s",\n  "sourceRevision": "%s",\n  "buildProfile": "%s",\n  "rustToolchain": "%s",\n  "xcodeVersion": "%s",\n  "xcodeBuild": "%s",\n  "iphoneOSSDKVersion": "%s",\n  "iphoneOSSDKBuild": "%s",\n  "iphoneSimulatorSDKVersion": "%s",\n  "iphoneSimulatorSDKBuild": "%s",\n  "minimumIOSVersion": "%s",\n  "thirdPartyNoticesPath": "NuxieRuntime.xcframework/THIRD_PARTY_NOTICES.md",\n  "swiftPackageChecksum": "%s"\n}\n' \
    "$(sed -n 's/^#define NUX_RUNTIME_ABI_MAJOR //p' "${headers_dir}/nux_runtime.generated.h")" \
    "$(sed -n 's/^#define NUX_RUNTIME_ABI_MINOR //p' "${headers_dir}/nux_runtime.generated.h")" \
    "$(sed -n 's/^version = "\([^"]*\)"/\1/p' "${repo_root}/crates/nux-apple-runtime/Cargo.toml" | head -1)" \
    "${luaur_version}" \
    "${runtime_revision}" \
    "${profile}" \
    "${rust_toolchain}" \
    "${xcode_version}" \
    "${xcode_build}" \
    "${iphoneos_sdk_version}" \
    "${iphoneos_sdk_build}" \
    "${iphonesimulator_sdk_version}" \
    "${iphonesimulator_sdk_build}" \
    "${deployment_target}" \
    "${checksum}" > "${metadata_path}"

phase "Verify the packaged XCFramework"
report_disk
"${script_dir}/verify-apple-xcframework.sh" "${xcframework_path}" "${archive_path}" "${metadata_path}"

echo "XCFramework: ${xcframework_path}"
echo "Archive: ${archive_path}"
echo "Checksum: ${checksum}"
