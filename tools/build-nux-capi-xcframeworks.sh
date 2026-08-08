#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd -P "$(dirname "${BASH_SOURCE[0]}")" && pwd -P)"
repo_root="$(cd -P "${script_dir}/.." && pwd -P)"
targets=(
    aarch64-apple-ios
    aarch64-apple-ios-sim
    x86_64-apple-ios
    aarch64-apple-darwin
    x86_64-apple-darwin
)

if [[ "${1:-}" == "--plan" ]]; then
    printf '%s\n' \
        'root-package: nux-capi' \
        'feature-set: legacy-migration' \
        'thin-builds: aarch64-apple-ios aarch64-apple-ios-sim x86_64-apple-ios aarch64-apple-darwin x86_64-apple-darwin' \
        'artifact full-apple: all five thin builds' \
        'artifact ios-only: first three thin builds'
    exit 0
fi
if [[ $# -gt 1 ]]; then
    echo "usage: $0 [output-directory] | --plan" >&2
    exit 2
fi

requested_output_root="${1:-${repo_root}/target/nux-capi-apple}"
if [[ -z "${requested_output_root}" ]]; then
    echo "refusing an empty output path" >&2
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
macos_deployment_target="${NUX_APPLE_MACOS_DEPLOYMENT_TARGET:-12.0}"
rust_toolchain="${NUX_APPLE_RUST_TOOLCHAIN:-1.94.1}"
rust_cargo="$(rustup which --toolchain "${rust_toolchain}" cargo)"
rust_compiler="$(rustup which --toolchain "${rust_toolchain}" rustc)"
rust_host="$("${rust_compiler}" -vV | sed -n 's/^host: //p')"
rust_sysroot="$("${rust_compiler}" --print sysroot)"
rust_llvm_objcopy="${rust_sysroot}/lib/rustlib/${rust_host}/bin/llvm-objcopy"
xcodebuild_path="$(command -v xcodebuild)"
lipo_path="$(command -v lipo)"
ditto_path="$(command -v ditto)"
swift_path="$(command -v swift)"
clang_path="$(xcrun --find clang)"
rustc_version="$("${rust_compiler}" -vV | tr '\n' ' ' | sed 's/[[:space:]]*$//')"
cargo_version="$("${rust_cargo}" -Vv | tr '\n' ' ' | sed 's/[[:space:]]*$//')"
runtime_version="$(sed -n 's/^version = "\([^"]*\)"/\1/p' "${repo_root}/crates/nux-capi/Cargo.toml" | head -1)"
source_revision="$(git -C "${repo_root}" rev-parse --verify HEAD)"

if [[ -n "$(git -C "${repo_root}" status --porcelain --untracked-files=all)" ]]; then
    echo "refusing to package a dirty runtime tree" >&2
    exit 4
fi
if [[ ! -x "${rust_llvm_objcopy}" ]]; then
    echo "missing pinned llvm-objcopy for Rust ${rust_toolchain}" >&2
    exit 3
fi
for target in "${targets[@]}"; do
    if ! rustup target list --toolchain "${rust_toolchain}" --installed | grep -qx "${target}"; then
        echo "missing Rust target ${target} for toolchain ${rust_toolchain}" >&2
        exit 3
    fi
done

build_root="${output_root}/build"
cargo_target_dir="${build_root}/cargo"
stripped_root="${build_root}/stripped"
headers_dir="${build_root}/Headers"
universal_root="${build_root}/universal"
full_framework="${output_root}/full/NuxieRuntime.xcframework"
ios_framework="${output_root}/ios/NuxieRuntime.xcframework"
full_archive="${output_root}/NuxieRuntime.xcframework.zip"
ios_archive="${output_root}/NuxieRuntime-iOS.xcframework.zip"
metadata_path="${output_root}/artifact-set.json"
size_report_path="${output_root}/SIZE_REPORT.json"
build_inputs_path="${build_root}/BUILD_INPUTS.json"

rm -rf \
    "${stripped_root}" \
    "${headers_dir}" \
    "${universal_root}" \
    "${output_root}/full" \
    "${output_root}/ios" \
    "${full_archive}" \
    "${ios_archive}" \
    "${metadata_path}" \
    "${size_report_path}"
mkdir -p \
    "${stripped_root}" \
    "${headers_dir}" \
    "${universal_root}" \
    "$(dirname "${full_framework}")" \
    "$(dirname "${ios_framework}")"

contract_fingerprint="$({
    shasum -a 256 \
        "${repo_root}/crates/nux-capi/include/nux_capi.generated.h" \
        "${repo_root}/crates/nux-capi/abi-layout-v3.json" \
        "${repo_root}/crates/nux-capi/exports-v3-portable.txt" \
        "${repo_root}/crates/nux-capi/exports-v3-apple-metal-extension.txt" \
        "${repo_root}/crates/nux-capi/exports-v3-legacy-migration.txt"
} | shasum -a 256 | awk '{ print $1 }')"

xcode_version="$(xcodebuild -version | sed -n 's/^Xcode //p')"
xcode_build="$(xcodebuild -version | sed -n 's/^Build version //p')"
iphoneos_sdk="$(xcrun --sdk iphoneos --show-sdk-version) ($(xcrun --sdk iphoneos --show-sdk-build-version))"
iphonesimulator_sdk="$(xcrun --sdk iphonesimulator --show-sdk-version) ($(xcrun --sdk iphonesimulator --show-sdk-build-version))"
macos_sdk="$(xcrun --sdk macosx --show-sdk-version) ($(xcrun --sdk macosx --show-sdk-build-version))"

build_inputs_hash="$(
    python3 "${script_dir}/apple_runtime_input_digest.py" \
        write "${build_inputs_path}" \
        --repo-root "${repo_root}" \
        --cargo "${rust_cargo}" \
        --root-package nux-capi \
        --feature legacy-migration \
        --build-profile "${profile}" \
        --rust-toolchain "${rust_toolchain}" \
        --rustc-version "${rustc_version}" \
        --cargo-version "${cargo_version}" \
        --xcode-version "${xcode_version}" \
        --xcode-build "${xcode_build}" \
        --iphoneos-sdk "${iphoneos_sdk}" \
        --iphonesimulator-sdk "${iphonesimulator_sdk}" \
        --macos-sdk "${macos_sdk}" \
        --minimum-ios-version "${deployment_target}" \
        --minimum-macos-version "${macos_deployment_target}" \
        --tool "cargo=${rust_cargo}" \
        --tool "rustc=${rust_compiler}" \
        --tool "llvm-objcopy=${rust_llvm_objcopy}" \
        --tool "xcodebuild=${xcodebuild_path}" \
        --tool "lipo=${lipo_path}" \
        --tool "ditto=${ditto_path}" \
        --tool "swift=${swift_path}" \
        --tool "clang=${clang_path}"
)"

for target in "${targets[@]}"; do
    IPHONEOS_DEPLOYMENT_TARGET="${deployment_target}" \
    MACOSX_DEPLOYMENT_TARGET="${macos_deployment_target}" \
    NUX_RUNTIME_BUILD_INPUTS_HASH="${build_inputs_hash}" \
    NUX_RUNTIME_BUILD_PROFILE="${profile}" \
    NUX_RUNTIME_CONTRACT_FINGERPRINT="${contract_fingerprint}" \
    NUX_RUNTIME_RUSTC_VERSION="${rustc_version}" \
    NUX_RUNTIME_SOURCE_REVISION="${source_revision}" \
    CARGO_TARGET_DIR="${cargo_target_dir}" \
    RUSTC="${rust_compiler}" \
        "${rust_cargo}" build \
            --manifest-path "${repo_root}/Cargo.toml" \
            --locked \
            --package nux-capi \
            --no-default-features \
            --features legacy-migration \
            --profile "${profile}" \
            --target "${target}"
    mkdir -p "${stripped_root}/${target}"
    cp "${cargo_target_dir}/${target}/${profile}/libnux_capi.a" \
        "${stripped_root}/${target}/libnux_capi.a"
    "${rust_llvm_objcopy}" \
        --remove-section=__LLVM,__bitcode \
        --remove-section=__LLVM,__cmdline \
        "${stripped_root}/${target}/libnux_capi.a"
done

device_library="${stripped_root}/aarch64-apple-ios/libnux_capi.a"
arm_simulator_library="${stripped_root}/aarch64-apple-ios-sim/libnux_capi.a"
intel_simulator_library="${stripped_root}/x86_64-apple-ios/libnux_capi.a"
arm_macos_library="${stripped_root}/aarch64-apple-darwin/libnux_capi.a"
intel_macos_library="${stripped_root}/x86_64-apple-darwin/libnux_capi.a"
simulator_library="${universal_root}/libnux_capi-simulator.a"
macos_library="${universal_root}/libnux_capi-macos.a"
lipo -create "${arm_simulator_library}" "${intel_simulator_library}" -output "${simulator_library}"
lipo -create "${arm_macos_library}" "${intel_macos_library}" -output "${macos_library}"

cp "${repo_root}/crates/nux-capi/include/nux_capi.h" "${headers_dir}/"
cp "${repo_root}/crates/nux-capi/include/nux_capi.generated.h" "${headers_dir}/"
cp "${repo_root}/crates/nux-capi/include/nux_capi_apple.h" "${headers_dir}/"
cp "${repo_root}/crates/nux-apple-runtime/include/nux_runtime.h" "${headers_dir}/"
cp "${repo_root}/crates/nux-apple-runtime/include/nux_runtime.generated.h" "${headers_dir}/"
cp "${repo_root}/crates/nux-capi/include/module.migration.modulemap" "${headers_dir}/module.modulemap"

xcodebuild -create-xcframework \
    -library "${device_library}" -headers "${headers_dir}" \
    -library "${simulator_library}" -headers "${headers_dir}" \
    -library "${macos_library}" -headers "${headers_dir}" \
    -output "${full_framework}"
xcodebuild -create-xcframework \
    -library "${device_library}" -headers "${headers_dir}" \
    -library "${simulator_library}" -headers "${headers_dir}" \
    -output "${ios_framework}"

for framework in "${full_framework}" "${ios_framework}"; do
    cp "${repo_root}/LICENSE" "${framework}/LICENSE"
    cp "${repo_root}/THIRD_PARTY_NOTICES.md" "${framework}/THIRD_PARTY_NOTICES.md"
    cp "${build_inputs_path}" "${framework}/BUILD_INPUTS.json"
done
ditto -c -k --sequesterRsrc --keepParent "${full_framework}" "${full_archive}"
ditto -c -k --sequesterRsrc --keepParent "${ios_framework}" "${ios_archive}"
full_checksum="$(swift package compute-checksum "${full_archive}")"
ios_checksum="$(swift package compute-checksum "${ios_archive}")"

python3 - "${metadata_path}" "${runtime_version}" "${source_revision}" \
    "${contract_fingerprint}" "${build_inputs_hash}" "${full_checksum}" "${ios_checksum}" <<'PY'
import json
import pathlib
import sys

path, version, revision, fingerprint, inputs, full_checksum, ios_checksum = sys.argv[1:]
full_targets = sorted([
    "aarch64-apple-darwin", "aarch64-apple-ios", "aarch64-apple-ios-sim",
    "x86_64-apple-darwin", "x86_64-apple-ios",
])
ios_targets = sorted([
    "aarch64-apple-ios", "aarch64-apple-ios-sim", "x86_64-apple-ios",
])
document = {
    "schemaVersion": 6,
    "runtimeVersion": version,
    "buildSourceRevision": revision,
    "releaseRevision": revision,
    "runtimeIdentity": f"{version}@{revision}",
    "contractFingerprint": fingerprint,
    "buildInputsHash": inputs,
    "artifacts": [
        {
            "kind": "full-apple",
            "archiveName": "NuxieRuntime.xcframework.zip",
            "bundleName": "NuxieRuntime.xcframework",
            "swiftPackageChecksum": full_checksum,
            "targets": full_targets,
        },
        {
            "kind": "ios-only",
            "archiveName": "NuxieRuntime-iOS.xcframework.zip",
            "bundleName": "NuxieRuntime.xcframework",
            "swiftPackageChecksum": ios_checksum,
            "targets": ios_targets,
        },
    ],
}
pathlib.Path(path).write_text(json.dumps(document, indent=2) + "\n")
PY

python3 "${script_dir}/apple_runtime_contract.py" distribution "${metadata_path}"
"${script_dir}/verify-nux-capi-xcframeworks.sh" \
    "${output_root}" "${metadata_path}" "${build_inputs_hash}" "${size_report_path}"

printf 'Full Apple: %s\niOS only: %s\nMetadata: %s\nSizes: %s\n' \
    "${full_archive}" "${ios_archive}" "${metadata_path}" "${size_report_path}"
