#!/usr/bin/env bash
set -euo pipefail

if [[ $# -ne 3 ]]; then
    echo "usage: verify-apple-xcframework.sh <xcframework> <zip> <artifact-metadata>" >&2
    exit 2
fi

xcframework_path="$1"
archive_path="$2"
metadata_path="$3"
info_plist="${xcframework_path}/Info.plist"

test -f "${info_plist}"
test -f "${archive_path}"
test -f "${metadata_path}"
plutil -lint "${info_plist}" >/dev/null
# `plutil -lint` only accepts plist syntax on older supported Xcode hosts,
# while `-p` validates both plist and JSON inputs.
plutil -p "${metadata_path}" >/dev/null

expected_checksum="$(plutil -extract swiftPackageChecksum raw "${metadata_path}")"
actual_checksum="$(swift package compute-checksum "${archive_path}")"
test "${actual_checksum}" = "${expected_checksum}"

verification_temp_dir="$(mktemp -d "${TMPDIR:-/tmp}/nuxie-runtime-verify.XXXXXX")"
trap 'rm -rf "${verification_temp_dir}"' EXIT
ditto -x -k "${archive_path}" "${verification_temp_dir}"
archived_framework="$(find "${verification_temp_dir}" -maxdepth 1 -name '*.xcframework' -print -quit)"
test -n "${archived_framework}"
diff -rq "${xcframework_path}" "${archived_framework}" >/dev/null

device_library="$(find "${xcframework_path}" -path '*ios-arm64/libnux_apple_runtime.a' -print -quit)"
simulator_library="$(find "${xcframework_path}" -path '*ios-arm64_x86_64-simulator/libnux_apple_runtime.a' -print -quit)"

test -n "${device_library}"
test -n "${simulator_library}"
test "$(lipo -archs "${device_library}")" = "arm64"
simulator_archs="$(lipo -archs "${simulator_library}")"
[[ "${simulator_archs}" == *arm64* ]]
[[ "${simulator_archs}" == *x86_64* ]]

for library in "${device_library}" "${simulator_library}"; do
    symbols="$(nm -gjU "${library}" 2>/dev/null)"
    grep -Fxq '_nux_runtime_abi_major' <<< "${symbols}"
    grep -Fxq '_nux_runtime_abi_minor' <<< "${symbols}"
    grep -Fxq '_nux_runtime_require_abi' <<< "${symbols}"
    grep -Fxq '_nux_runtime_build_provenance' <<< "${symbols}"
    grep -Fxq '_nux_flow_runtime_context_create' <<< "${symbols}"
    grep -Fxq '_nux_flow_render_session_create' <<< "${symbols}"
    grep -Fxq '_nux_flow_render_session_attach_apple_surface' <<< "${symbols}"
    grep -Fxq '_nux_apple_surface_reattach' <<< "${symbols}"
    grep -Fxq '_nux_flow_render_session_advance' <<< "${symbols}"
    grep -Fxq '_rust_eh_personality' <<< "${symbols}"
done

source_revision="$(plutil -extract sourceRevision raw "${metadata_path}")"
build_profile="$(plutil -extract buildProfile raw "${metadata_path}")"
minimum_ios_version="$(plutil -extract minimumIOSVersion raw "${metadata_path}")"
rust_toolchain="$(plutil -extract rustToolchain raw "${metadata_path}")"
runtime_version="$(plutil -extract runtimeVersion raw "${metadata_path}")"
xcode_version="$(plutil -extract xcodeVersion raw "${metadata_path}")"
xcode_build="$(plutil -extract xcodeBuild raw "${metadata_path}")"
iphoneos_sdk_version="$(plutil -extract iphoneOSSDKVersion raw "${metadata_path}")"
iphoneos_sdk_build="$(plutil -extract iphoneOSSDKBuild raw "${metadata_path}")"
iphonesimulator_sdk_version="$(plutil -extract iphoneSimulatorSDKVersion raw "${metadata_path}")"
iphonesimulator_sdk_build="$(plutil -extract iphoneSimulatorSDKBuild raw "${metadata_path}")"
test "$(plutil -extract schemaVersion raw "${metadata_path}")" = "1"
test "$(xcodebuild -version | sed -n 's/^Xcode //p')" = "${xcode_version}"
test "$(xcodebuild -version | sed -n 's/^Build version //p')" = "${xcode_build}"
test "$(xcrun --sdk iphoneos --show-sdk-version)" = "${iphoneos_sdk_version}"
test "$(xcrun --sdk iphoneos --show-sdk-build-version)" = "${iphoneos_sdk_build}"
test "$(xcrun --sdk iphonesimulator --show-sdk-version)" = "${iphonesimulator_sdk_version}"
test "$(xcrun --sdk iphonesimulator --show-sdk-build-version)" = "${iphonesimulator_sdk_build}"
for target_and_library in \
    "aarch64-apple-ios:${device_library}" \
    "aarch64-apple-ios-sim:${simulator_library}" \
    "x86_64-apple-ios:${simulator_library}"; do
    target="${target_and_library%%:*}"
    library="${target_and_library#*:}"
    provenance="$(strings "${library}" | grep -F "\"target\":\"${target}\"" | head -1)"
    test -n "${provenance}"
    grep -Fq "\"sourceRevision\":\"${source_revision}\"" <<< "${provenance}"
    grep -Fq "\"runtimeVersion\":\"${runtime_version}\"" <<< "${provenance}"
    grep -Fq "\"profile\":\"${build_profile}\"" <<< "${provenance}"
    grep -Fq "\"rustc\":\"rustc ${rust_toolchain}" <<< "${provenance}"
    grep -Fq '"features":"apple-product"' <<< "${provenance}"
    grep -Fq '"luaurVersion":null' <<< "${provenance}"
done

headers_dir="$(dirname "${device_library}")/Headers"
test -f "${headers_dir}/nux_runtime.h"
test -f "${headers_dir}/nux_runtime.generated.h"
test -f "${headers_dir}/module.modulemap"
header_abi_major="$(sed -n 's/^#define NUX_RUNTIME_ABI_MAJOR //p' "${headers_dir}/nux_runtime.generated.h")"
header_abi_minor="$(sed -n 's/^#define NUX_RUNTIME_ABI_MINOR //p' "${headers_dir}/nux_runtime.generated.h")"
test "$(plutil -extract abiMajor raw "${metadata_path}")" = "${header_abi_major}"
test "$(plutil -extract abiMinor raw "${metadata_path}")" = "${header_abi_minor}"

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
repo_root="$(cd "${script_dir}/.." && pwd)"
clang -std=c11 -Wall -Wextra -Werror \
    -I"${headers_dir}" \
    -fsyntax-only \
    "${repo_root}/crates/nux-apple-runtime/smoke/header_smoke.c"

swift_smoke_dir="${verification_temp_dir}/swift-link"
mkdir -p "${swift_smoke_dir}"

link_swift_smoke() {
    local sdk="$1"
    local target="$2"
    local headers="$3"
    local library="$4"
    local label="$5"
    local output="${swift_smoke_dir}/libNuxieRuntimeSmoke-${label}.dylib"
    local sdk_path
    sdk_path="$(xcrun --sdk "${sdk}" --show-sdk-path)"
    xcrun --sdk "${sdk}" swiftc \
        -emit-library \
        -parse-as-library \
        -sdk "${sdk_path}" \
        -target "${target}" \
        -I "${headers}" \
        -L "$(dirname "${library}")" \
        -lnux_apple_runtime \
        -framework Foundation \
        -framework QuartzCore \
        -framework Metal \
        -framework CoreGraphics \
        -framework Security \
        "${repo_root}/crates/nux-apple-runtime/smoke/swift_import_smoke.swift" \
        -o "${output}"

    local expected_arch="${target%%-*}"
    test "$(lipo -archs "${output}")" = "${expected_arch}"
    local linked_symbols
    linked_symbols="$(nm -gjU "${output}")"
    grep -Fxq '_nux_runtime_abi_major' <<< "${linked_symbols}"
    ! otool -L "${output}" | grep -Eiq 'rive|nuxie_runtime'
    local linked_minos
    linked_minos="$(otool -l "${output}" | awk '$1 == "minos" { print $2 }' | sort -u)"
    test "${linked_minos}" = "${minimum_ios_version}"
}

simulator_headers_dir="$(dirname "${simulator_library}")/Headers"
link_swift_smoke \
    iphoneos "arm64-apple-ios${minimum_ios_version}" \
    "${headers_dir}" "${device_library}" device-arm64
link_swift_smoke \
    iphonesimulator "arm64-apple-ios${minimum_ios_version}-simulator" \
    "${simulator_headers_dir}" "${simulator_library}" simulator-arm64
link_swift_smoke \
    iphonesimulator "x86_64-apple-ios${minimum_ios_version}-simulator" \
    "${simulator_headers_dir}" "${simulator_library}" simulator-x86_64

echo "apple-xcframework verification passed"
