#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd -P "$(dirname "${BASH_SOURCE[0]}")" && pwd -P)"
repo_root="$(cd -P "${script_dir}/.." && pwd -P)"
rust_toolchain="1.94.1"
ndk_version="29.0.14206865"
android_api="23"
features="android-vulkan,scripting,android-authored-wgsl"
targets=(aarch64-linux-android x86_64-linux-android)
abis=(arm64-v8a x86_64)

if [[ "${1:-}" == "--plan" ]]; then
    printf '%s\n' \
        'artifact: NuxieRuntimeAndroid.zip' \
        "release-tag: $(python3 "${script_dir}/android_runtime_contract.py" release-tag)" \
        'root-package: nux-capi' \
        'rust-toolchain: 1.94.1' \
        'build-system: Bazel 9.3.0 / rules_rust 0.74.0' \
        'bazel-target: //crates/nux-capi:nux-capi__android__cdylib' \
        'android-ndk: 29.0.14206865' \
        'android-api: 23' \
        'targets: aarch64-linux-android x86_64-linux-android' \
        'abis: arm64-v8a x86_64' \
        'features: android-vulkan,scripting,android-authored-wgsl' \
        'archive-tree: include/nux_capi.generated.h plus libnux_capi.so and libc++_shared.so for each ABI' \
        'qualification: ABI4 layout/header/exports, ELF architecture/DT_NEEDED/16-KiB LOAD alignment, provenance, checksums, size budget'
    exit 0
fi
if [[ $# -gt 1 ]]; then
    echo "usage: $0 [output-directory] | --plan" >&2
    exit 2
fi

requested_output_root="${1:-${repo_root}/target/nux-capi-android}"
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

source_revision="$(git -C "${repo_root}" rev-parse --verify HEAD)"
if [[ ! "${source_revision}" =~ ^[0-9a-f]{40}$ ]]; then
    echo "source revision is not a full Git SHA: ${source_revision}" >&2
    exit 3
fi
if [[ -n "$(git -C "${repo_root}" status --porcelain --untracked-files=all)" ]]; then
    echo "refusing to package a dirty runtime tree" >&2
    exit 4
fi

for variable in \
    AR ARFLAGS CC CFLAGS CPPFLAGS CXX CXXFLAGS LD LDFLAGS LIBRARY_PATH \
    RANLIB RUSTC RUSTC_BOOTSTRAP RUSTC_LINKER RUSTC_WRAPPER \
    RUSTC_WORKSPACE_WRAPPER RUSTFLAGS CARGO_BUILD_RUSTFLAGS \
    CARGO_ENCODED_RUSTFLAGS CARGO_INCREMENTAL NUX_CAPI_UPDATE_HEADER
do
    if [[ -n "${!variable:-}" ]]; then
        echo "release build has unaudited compiler override: ${variable}" >&2
        exit 4
    fi
done
while IFS='=' read -r variable _; do
    case "${variable}" in
        CARGO_TARGET_*_LINKER|CARGO_TARGET_*_RUNNER|CARGO_TARGET_*_RUSTFLAGS|\
        HOST_AR|HOST_AR_*|HOST_CC|HOST_CC_*|HOST_CFLAGS|HOST_CFLAGS_*|\
        HOST_CXX|HOST_CXX_*|HOST_CXXFLAGS|HOST_CXXFLAGS_*|HOST_RANLIB|HOST_RANLIB_*|\
        TARGET_AR|TARGET_AR_*|TARGET_CC|TARGET_CC_*|TARGET_CFLAGS|TARGET_CFLAGS_*|\
        TARGET_CXX|TARGET_CXX_*|TARGET_CXXFLAGS|TARGET_CXXFLAGS_*|TARGET_RANLIB|TARGET_RANLIB_*|\
        AR_*|CC_*|CFLAGS_*|CXX_*|CXXFLAGS_*|RANLIB_*)
            echo "release build has unaudited target compiler override: ${variable}" >&2
            exit 4
            ;;
    esac
done < <(env)

ndk_root="${ANDROID_NDK_HOME:-${ANDROID_NDK_ROOT:-}}"
if [[ -z "${ndk_root}" && -n "${ANDROID_HOME:-}" ]]; then
    ndk_root="${ANDROID_HOME}/ndk/${ndk_version}"
fi
if [[ -z "${ndk_root}" || ! -f "${ndk_root}/source.properties" ]]; then
    echo "set ANDROID_NDK_HOME to Android NDK ${ndk_version}" >&2
    exit 3
fi
resolved_ndk_root="$(cd -P "${ndk_root}" && pwd -P)"
actual_ndk_version="$(sed -n 's/^Pkg\.Revision[[:space:]]*=[[:space:]]*//p' "${resolved_ndk_root}/source.properties")"
if [[ "${actual_ndk_version}" != "${ndk_version}" ]]; then
    echo "Android NDK must be ${ndk_version}, found ${actual_ndk_version:-unknown}" >&2
    exit 3
fi

shopt -s nullglob
ndk_prebuilts=("${resolved_ndk_root}"/toolchains/llvm/prebuilt/*)
if [[ ${#ndk_prebuilts[@]} -ne 1 || ! -d "${ndk_prebuilts[0]}" ]]; then
    echo "Android NDK must expose exactly one host prebuilt" >&2
    exit 3
fi
ndk_prebuilt="$(cd -P "${ndk_prebuilts[0]}" && pwd -P)"
ndk_host_tag="$(basename "${ndk_prebuilt}")"
ndk_bin="${ndk_prebuilt}/bin"
ndk_sysroot_lib="${ndk_prebuilt}/sysroot/usr/lib"

export ANDROID_NDK_HOME="${resolved_ndk_root}"
export ANDROID_NDK_ROOT="${resolved_ndk_root}"

for path in \
    "${ndk_bin}/aarch64-linux-android${android_api}-clang" \
    "${ndk_bin}/x86_64-linux-android${android_api}-clang" \
    "${ndk_bin}/llvm-ar" \
    "${ndk_bin}/llvm-nm" \
    "${ndk_bin}/llvm-readelf" \
    "${ndk_bin}/llvm-strings" \
    "${ndk_bin}/llvm-strip" \
    "${ndk_sysroot_lib}/aarch64-linux-android/libc++_shared.so" \
    "${ndk_sysroot_lib}/x86_64-linux-android/libc++_shared.so"
do
    if [[ ! -f "${path}" ]]; then
        echo "pinned Android toolchain input is missing: ${path}" >&2
        exit 3
    fi
done

for relative in \
    crates/nux-capi/abi-layout-v4.json \
    crates/nux-capi/exports-v4-portable.txt \
    crates/nux-capi/exports-v4-apple-metal-extension.txt \
    crates/nux-capi/exports-v4-android-vulkan-extension.txt \
    crates/nux-capi/exports-v4-android-authored-wgsl-extension.txt
do
    if [[ ! -f "${repo_root}/${relative}" ]]; then
        echo "ABI-v4 release contract is missing: ${relative}" >&2
        exit 3
    fi
done

runtime_version="$(sed -n 's/^version = "\([^"]*\)"/\1/p' "${repo_root}/crates/nux-capi/Cargo.toml" | head -1)"
source_date_epoch="$(git -C "${repo_root}" show -s --format=%ct HEAD)"
contract_fingerprint="$(python3 "${script_dir}/android_runtime_contract.py" fingerprint --repo-root "${repo_root}")"
rustc_version="$(python3 "${script_dir}/bazel_runtime_build.py" toolchain \
    --repo-root "${repo_root}" --label //crates/nux-capi:nux-capi__android__cdylib \
    --target aarch64-linux-android | python3 -c 'import json,sys; print(json.load(sys.stdin)["version"])')"

build_root="${output_root}/build"
prebuilt_root="${build_root}/prebuilt"
build_inputs="${build_root}/BUILD_INPUTS.json"
archive="${output_root}/NuxieRuntimeAndroid.zip"
metadata="${output_root}/NuxieRuntimeAndroid.json"
published_inputs="${output_root}/NuxieRuntimeAndroid-BUILD_INPUTS.json"
size_report="${output_root}/NuxieRuntimeAndroid-SIZE_REPORT.json"

rm -rf "${build_root}"
rm -f "${archive}" "${metadata}" "${published_inputs}" "${size_report}"
mkdir -p \
    "${prebuilt_root}/include" \
    "${prebuilt_root}/jniLibs/arm64-v8a" \
    "${prebuilt_root}/jniLibs/x86_64"

build_inputs_hash="$(
    python3 "${script_dir}/android_runtime_contract.py" inputs "${build_inputs}" \
        --repo-root "${repo_root}" \
        --source-revision "${source_revision}" \
        --runtime-version "${runtime_version}" \
        --source-date-epoch "${source_date_epoch}" \
        --ndk-root "${resolved_ndk_root}" \
        --ndk-host-tag "${ndk_host_tag}"
)"

for index in "${!targets[@]}"; do
    NUX_RUNTIME_SOURCE_REVISION="${source_revision}" \
    NUX_RUNTIME_BUILD_INPUTS_HASH="${build_inputs_hash}" \
    NUX_RUNTIME_CONTRACT_FINGERPRINT="${contract_fingerprint}" \
    NUX_RUNTIME_BUILD_PROFILE="release" \
    NUX_RUNTIME_RUSTC_VERSION="${rustc_version}" \
    NUX_RUNTIME_DISTRIBUTION_ROOT_PACKAGE="nux-capi" \
        python3 "${script_dir}/bazel_runtime_build.py" build \
            --repo-root "${repo_root}" \
            --label //crates/nux-capi:nux-capi__android__cdylib \
            --target "${targets[index]}" \
            --output "${prebuilt_root}/jniLibs/${abis[index]}/libnux_capi.so"
done

cp "${repo_root}/crates/nux-capi/include/nux_capi.generated.h" \
    "${prebuilt_root}/include/nux_capi.generated.h"
cp "${ndk_sysroot_lib}/aarch64-linux-android/libc++_shared.so" \
    "${prebuilt_root}/jniLibs/arm64-v8a/libc++_shared.so"
cp "${ndk_sysroot_lib}/x86_64-linux-android/libc++_shared.so" \
    "${prebuilt_root}/jniLibs/x86_64/libc++_shared.so"
for abi in "${abis[@]}"; do
    "${ndk_bin}/llvm-strip" --strip-unneeded \
        "${prebuilt_root}/jniLibs/${abi}/libnux_capi.so"
done

python3 "${script_dir}/android_runtime_contract.py" package \
    --repo-root "${repo_root}" \
    --artifact-root "${output_root}" \
    --prebuilt-root "${prebuilt_root}" \
    --build-inputs "${build_inputs}" \
    --source-revision "${source_revision}" \
    --runtime-version "${runtime_version}"
python3 "${script_dir}/android_runtime_contract.py" verify \
    --repo-root "${repo_root}" \
    --artifact-root "${output_root}" \
    --ndk-root "${resolved_ndk_root}"

printf 'Android archive: %s\nMetadata: %s\nBuild inputs: %s\nSizes: %s\n' \
    "${archive}" "${metadata}" "${published_inputs}" "${size_report}"
