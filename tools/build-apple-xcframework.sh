#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd -P "$(dirname "${BASH_SOURCE[0]}")" && pwd -P)"
repo_root="$(cd -P "${script_dir}/.." && pwd -P)"
cd "${repo_root}"
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
macos_deployment_target="${NUX_APPLE_MACOS_DEPLOYMENT_TARGET:-12.0}"
rust_toolchain="${NUX_APPLE_RUST_TOOLCHAIN:-1.94.1}"
rust_cargo="$(rustup which --toolchain "${rust_toolchain}" cargo)"
rust_compiler="$(rustup which --toolchain "${rust_toolchain}" rustc)"
rust_host="$("${rust_compiler}" -vV | sed -n 's/^host: //p')"
rust_sysroot="$("${rust_compiler}" --print sysroot)"
rust_llvm_nm="${rust_sysroot}/lib/rustlib/${rust_host}/bin/llvm-nm"
rust_llvm_objcopy="${rust_sysroot}/lib/rustlib/${rust_host}/bin/llvm-objcopy"
xcodebuild_path="$(command -v xcodebuild)"
lipo_path="$(command -v lipo)"
ditto_path="$(command -v ditto)"
swift_path="$(command -v swift)"
clang_path="$(xcrun --find clang)"
rustc_version="$("${rust_compiler}" -vV)"
cargo_version="$("${rust_cargo}" -Vv)"
runtime_revision="${NUX_RUNTIME_SOURCE_REVISION:-}"
runtime_version="$(
    sed -n 's/^version = "\([^"]*\)"/\1/p' \
        "${repo_root}/crates/nux-apple-runtime/Cargo.toml" |
        head -1
)"
xcode_version="$(xcodebuild -version | sed -n 's/^Xcode //p')"
xcode_build="$(xcodebuild -version | sed -n 's/^Build version //p')"
iphoneos_sdk_version="$(xcrun --sdk iphoneos --show-sdk-version)"
iphoneos_sdk_build="$(xcrun --sdk iphoneos --show-sdk-build-version)"
iphonesimulator_sdk_version="$(xcrun --sdk iphonesimulator --show-sdk-version)"
iphonesimulator_sdk_build="$(xcrun --sdk iphonesimulator --show-sdk-build-version)"
macosx_sdk_version="$(xcrun --sdk macosx --show-sdk-version)"
macosx_sdk_build="$(xcrun --sdk macosx --show-sdk-build-version)"
build_root="${output_root}/build"
cargo_target_dir="${build_root}/cargo"
build_inputs_manifest_path="${build_root}/build-inputs.json"
headers_dir="${build_root}/Headers"
simulator_dir="${build_root}/simulator"
macos_dir="${build_root}/macos"
stripped_root="${build_root}/stripped"
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
if [[ -z "${runtime_version}" ]]; then
    echo "cannot determine the Apple runtime version from Cargo.toml" >&2
    exit 11
fi

for rust_llvm_tool in "${rust_llvm_nm}" "${rust_llvm_objcopy}"; do
    if [[ ! -x "${rust_llvm_tool}" ]]; then
        echo "missing $(basename "${rust_llvm_tool}") for Rust toolchain ${rust_toolchain}" >&2
        echo "install it with: rustup component add --toolchain ${rust_toolchain} llvm-tools" >&2
        exit 9
    fi
done

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

if ! git -C "${repo_root}" rev-parse --is-inside-work-tree >/dev/null 2>&1; then
    echo "refusing to package unverifiable source outside a Git worktree" >&2
    exit 8
fi
git_revision="$(git -C "${repo_root}" rev-parse --verify HEAD)"
if [[ -z "${runtime_revision}" ]]; then
    runtime_revision="${git_revision}"
elif [[ "${runtime_revision}" != "${git_revision}" ]]; then
    echo "requested runtime source revision does not match the checked-out commit" >&2
    echo "requested: ${runtime_revision}" >&2
    echo "checkout:  ${git_revision}" >&2
    exit 12
fi
if [[ -n "$(git -C "${repo_root}" status --porcelain --untracked-files=all)" ]]; then
    if [[ "${NUX_APPLE_ALLOW_DIRTY:-0}" != "1" ]]; then
        echo "refusing to package a dirty runtime tree" >&2
        echo "commit the runtime or set NUX_APPLE_ALLOW_DIRTY=1 for a local prototype" >&2
        exit 4
    fi
    if ! git -C "${repo_root}" diff --quiet --no-ext-diff HEAD -- ||
        [[ -n "$(
            git -C "${repo_root}" ls-files \
                --others \
                --exclude-standard \
                -- \
                crates \
                vendor \
                .cargo
        )" ]]; then
        dirty_fingerprint="$(
            {
                printf '%s\0' "${git_revision}"
                git -C "${repo_root}" diff --binary --no-ext-diff HEAD --
                git -C "${repo_root}" ls-files \
                    --others \
                    --exclude-standard \
                    -z \
                    -- \
                    crates \
                    vendor \
                    .cargo |
                    while IFS= read -r -d '' untracked_path; do
                        printf '%s\0' "${untracked_path}"
                        cat "${repo_root}/${untracked_path}"
                        printf '\0'
                    done
            } | shasum -a 256 | awk '{ print $1 }'
        )"
        runtime_revision="${git_revision}-dirty.${dirty_fingerprint}"
    fi
fi
if [[ ! "${runtime_revision}" =~ ^[0-9a-f]{40}(-dirty\.[0-9a-f]{64})?$ ]]; then
    echo "runtime source revision is not an exact clean or diagnostic-dirty identity: ${runtime_revision}" >&2
    exit 5
fi
runtime_identity="${runtime_version}@${runtime_revision}"

# Keep Cargo's target directory as an incremental build cache, but recreate every
# directory that is copied into the published artifact. Without this boundary,
# headers and libraries from an older packaging layout can silently survive into
# a later XCFramework.
rm -rf \
    "${headers_dir}" \
    "${simulator_dir}" \
    "${macos_dir}" \
    "${stripped_root}" \
    "${xcframework_path}" \
    "${archive_path}" \
    "${metadata_path}"
mkdir -p "${output_root}" "${build_root}" "${headers_dir}" "${simulator_dir}" "${macos_dir}" "${stripped_root}"
phase "Prepare deterministic Apple runtime output"
report_disk

phase "Audit the exact Apple dependency closure and build inputs"
build_inputs_hash="$(
    python3 "${script_dir}/apple_runtime_input_digest.py" \
        write "${build_inputs_manifest_path}" \
        --repo-root "${repo_root}" \
        --cargo "${rust_cargo}" \
        --build-profile "${profile}" \
        --rust-toolchain "${rust_toolchain}" \
        --rustc-version "${rustc_version}" \
        --cargo-version "${cargo_version}" \
        --xcode-version "${xcode_version}" \
        --xcode-build "${xcode_build}" \
        --iphoneos-sdk "${iphoneos_sdk_version} (${iphoneos_sdk_build})" \
        --iphonesimulator-sdk "${iphonesimulator_sdk_version} (${iphonesimulator_sdk_build})" \
        --macos-sdk "${macosx_sdk_version} (${macosx_sdk_build})" \
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
if [[ ! "${build_inputs_hash}" =~ ^[0-9a-f]{64}$ ]]; then
    echo "Apple build-input digest is not a lowercase SHA-256: ${build_inputs_hash}" >&2
    exit 13
fi
echo "Build inputs: ${build_inputs_hash}"

targets=(
    aarch64-apple-ios
    aarch64-apple-ios-sim
    x86_64-apple-ios
    aarch64-apple-darwin
    x86_64-apple-darwin
)

for target in "${targets[@]}"; do
    if ! rustup target list --toolchain "${rust_toolchain}" --installed | grep -qx "${target}"; then
        echo "missing Rust target ${target} for toolchain ${rust_toolchain}" >&2
        echo "install it with: rustup target add --toolchain ${rust_toolchain} ${target}" >&2
        exit 3
    fi
    phase "Build Apple runtime for ${target}"
    IPHONEOS_DEPLOYMENT_TARGET="${deployment_target}" \
    MACOSX_DEPLOYMENT_TARGET="${macos_deployment_target}" \
    NUX_RUNTIME_BUILD_PROFILE="${profile}" \
    NUX_RUNTIME_BUILD_INPUTS_HASH="${build_inputs_hash}" \
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

# Fat-LTO Rust staticlibs embed LLVM bitcode (`__LLVM,__bitcode` /
# `__LLVM,__cmdline`) in some archive members. Apple deprecated embedded
# bitcode: shipping it bloats the artifact and Apple LLVM older than the
# bitcode's producer cannot parse those members (`llvm-nm` fails with
# "Unknown attribute kind"). Strip the sections with the pinned Rust
# toolchain's llvm-objcopy (Xcode's `bitcode_strip` is broken in 26.6) on
# copies, so Cargo's incremental cache keeps its original objects.
phase "Strip embedded LLVM bitcode from the runtime libraries"
for target in "${targets[@]}"; do
    mkdir -p "${stripped_root}/${target}"
    cp "${cargo_target_dir}/${target}/${profile}/libnux_apple_runtime.a" \
        "${stripped_root}/${target}/libnux_apple_runtime.a"
    "${rust_llvm_objcopy}" \
        --remove-section=__LLVM,__bitcode \
        --remove-section=__LLVM,__cmdline \
        "${stripped_root}/${target}/libnux_apple_runtime.a"
done
report_disk

device_library="${stripped_root}/aarch64-apple-ios/libnux_apple_runtime.a"
arm_simulator_library="${stripped_root}/aarch64-apple-ios-sim/libnux_apple_runtime.a"
intel_simulator_library="${stripped_root}/x86_64-apple-ios/libnux_apple_runtime.a"
simulator_library="${simulator_dir}/libnux_apple_runtime.a"
arm_macos_library="${stripped_root}/aarch64-apple-darwin/libnux_apple_runtime.a"
intel_macos_library="${stripped_root}/x86_64-apple-darwin/libnux_apple_runtime.a"
macos_library="${macos_dir}/libnux_apple_runtime.a"

phase "Create the universal simulator library"
lipo -create \
    "${arm_simulator_library}" \
    "${intel_simulator_library}" \
    -output "${simulator_library}"

phase "Create the universal macOS library"
lipo -create \
    "${arm_macos_library}" \
    "${intel_macos_library}" \
    -output "${macos_library}"

cp "${repo_root}/crates/nux-apple-runtime/include/nux_runtime.h" "${headers_dir}/"
cp "${repo_root}/crates/nux-apple-runtime/include/nux_runtime.generated.h" "${headers_dir}/"
cp "${repo_root}/crates/nux-apple-runtime/include/module.modulemap" "${headers_dir}/"
contract_fingerprint="$(shasum -a 256 "${headers_dir}/nux_runtime.generated.h" | awk '{ print $1 }')"

phase "Create the XCFramework"
xcodebuild -create-xcframework \
    -library "${device_library}" \
    -headers "${headers_dir}" \
    -library "${simulator_library}" \
    -headers "${headers_dir}" \
    -library "${macos_library}" \
    -headers "${headers_dir}" \
    -output "${xcframework_path}"

phase "Attach license notices"
cp "${repo_root}/LICENSE" "${license_path}"
cp "${repo_root}/THIRD_PARTY_NOTICES.md" "${third_party_notices_path}"
cp "${build_inputs_manifest_path}" "${xcframework_path}/BUILD_INPUTS.json"
report_disk

phase "Archive the XCFramework"
ditto -c -k --sequesterRsrc --keepParent "${xcframework_path}" "${archive_path}"
checksum="$(swift package compute-checksum "${archive_path}")"
report_disk

phase "Write artifact provenance"
printf '{\n  "schemaVersion": 5,\n  "runtimeVersion": "%s",\n  "buildSourceRevision": "%s",\n  "releaseRevision": "%s",\n  "buildInputsHash": "%s",\n  "buildInputsManifestPath": "NuxieRuntime.xcframework/BUILD_INPUTS.json",\n  "runtimeIdentity": "%s",\n  "contractFingerprint": "%s",\n  "luaurVersion": "%s",\n  "buildProfile": "%s",\n  "rustToolchain": "%s",\n  "xcodeVersion": "%s",\n  "xcodeBuild": "%s",\n  "iphoneOSSDKVersion": "%s",\n  "iphoneOSSDKBuild": "%s",\n  "iphoneSimulatorSDKVersion": "%s",\n  "iphoneSimulatorSDKBuild": "%s",\n  "macOSSDKVersion": "%s",\n  "macOSSDKBuild": "%s",\n  "minimumIOSVersion": "%s",\n  "minimumMacOSVersion": "%s",\n  "thirdPartyNoticesPath": "NuxieRuntime.xcframework/THIRD_PARTY_NOTICES.md",\n  "swiftPackageChecksum": "%s"\n}\n' \
    "${runtime_version}" \
    "${runtime_revision}" \
    "${git_revision}" \
    "${build_inputs_hash}" \
    "${runtime_identity}" \
    "${contract_fingerprint}" \
    "${luaur_version}" \
    "${profile}" \
    "${rust_toolchain}" \
    "${xcode_version}" \
    "${xcode_build}" \
    "${iphoneos_sdk_version}" \
    "${iphoneos_sdk_build}" \
    "${iphonesimulator_sdk_version}" \
    "${iphonesimulator_sdk_build}" \
    "${macosx_sdk_version}" \
    "${macosx_sdk_build}" \
    "${deployment_target}" \
    "${macos_deployment_target}" \
    "${checksum}" > "${metadata_path}"

phase "Verify the packaged XCFramework"
report_disk
"${script_dir}/verify-apple-xcframework.sh" "${xcframework_path}" "${archive_path}" "${metadata_path}"

echo "XCFramework: ${xcframework_path}"
echo "Archive: ${archive_path}"
echo "Checksum: ${checksum}"
