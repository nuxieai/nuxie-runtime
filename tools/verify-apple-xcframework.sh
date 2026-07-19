#!/usr/bin/env bash
set -euo pipefail

phase() {
    printf '  -> %s\n' "$1"
}

require_file() {
    local path="$1"
    if [[ ! -f "${path}" ]]; then
        echo "required Apple runtime artifact is missing: ${path}" >&2
        return 1
    fi
}

if [[ $# -ne 3 ]]; then
    echo "usage: verify-apple-xcframework.sh <xcframework> <zip> <artifact-metadata>" >&2
    exit 2
fi

xcframework_path="$1"
archive_path="$2"
metadata_path="$3"
info_plist="${xcframework_path}/Info.plist"
script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
repo_root="$(cd "${script_dir}/.." && pwd)"

metadata_scalar() {
    python3 "${script_dir}/json-scalar.py" "${metadata_path}" "$1" "$2"
}

phase "validate metadata and archive checksum"
require_file "${info_plist}"
require_file "${archive_path}"
require_file "${metadata_path}"
plutil -lint "${info_plist}"
if ! python3 -m json.tool "${metadata_path}" >/dev/null; then
    echo "artifact metadata is not valid JSON: ${metadata_path}" >&2
    sed -n '1,200p' "${metadata_path}" >&2
    exit 1
fi

expected_checksum="$(metadata_scalar swiftPackageChecksum string)"
actual_checksum="$(swift package compute-checksum "${archive_path}")"
if [[ "${actual_checksum}" != "${expected_checksum}" ]]; then
    echo "archive checksum does not match artifact metadata" >&2
    echo "expected: ${expected_checksum}" >&2
    echo "actual:   ${actual_checksum}" >&2
    exit 1
fi

phase "extract and compare the archive"
verification_temp_dir="$(mktemp -d "${TMPDIR:-/tmp}/nuxie-runtime-verify.XXXXXX")"
trap 'rm -rf "${verification_temp_dir}"' EXIT
ditto -x -k "${archive_path}" "${verification_temp_dir}"
archived_framework="$(find "${verification_temp_dir}" -maxdepth 1 -name '*.xcframework' -print -quit)"
test -n "${archived_framework}"
diff -rq "${xcframework_path}" "${archived_framework}" >/dev/null

device_library="$(find "${xcframework_path}" -path '*ios-arm64/libnux_apple_runtime.a' -print -quit)"
simulator_library="$(find "${xcframework_path}" -path '*ios-arm64_x86_64-simulator/libnux_apple_runtime.a' -print -quit)"

phase "validate architectures and exported ABI"
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

phase "validate toolchain and embedded provenance"
source_revision="$(metadata_scalar sourceRevision string)"
build_profile="$(metadata_scalar buildProfile string)"
minimum_ios_version="$(metadata_scalar minimumIOSVersion string)"
rust_toolchain="$(metadata_scalar rustToolchain string)"
runtime_version="$(metadata_scalar runtimeVersion string)"
xcode_version="$(metadata_scalar xcodeVersion string)"
xcode_build="$(metadata_scalar xcodeBuild string)"
iphoneos_sdk_version="$(metadata_scalar iphoneOSSDKVersion string)"
iphoneos_sdk_build="$(metadata_scalar iphoneOSSDKBuild string)"
iphonesimulator_sdk_version="$(metadata_scalar iphoneSimulatorSDKVersion string)"
iphonesimulator_sdk_build="$(metadata_scalar iphoneSimulatorSDKBuild string)"
test "$(metadata_scalar schemaVersion integer)" = "1"
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

expected_public_headers="$(printf '%s\n' \
    module.modulemap \
    nux_runtime.generated.h \
    nux_runtime.h)"

verify_public_header_allowlist() {
    local headers="$1"
    local actual_public_headers
    test -d "${headers}"
    actual_public_headers="$(
        cd "${headers}"
        find . -mindepth 1 -print | sed 's#^\./##' | LC_ALL=C sort
    )"
    if [[ "${actual_public_headers}" != "${expected_public_headers}" ]]; then
        echo "unexpected public header contents in ${headers}" >&2
        diff -u \
            <(printf '%s\n' "${expected_public_headers}") \
            <(printf '%s\n' "${actual_public_headers}") >&2 || true
        return 1
    fi
}

headers_dir="$(dirname "${device_library}")/Headers"
simulator_headers_dir="$(dirname "${simulator_library}")/Headers"
phase "validate the public header boundary"
verify_public_header_allowlist "${headers_dir}"
verify_public_header_allowlist "${simulator_headers_dir}"
for public_header in module.modulemap nux_runtime.generated.h nux_runtime.h; do
    cmp "${headers_dir}/${public_header}" "${simulator_headers_dir}/${public_header}"
done
header_abi_major="$(sed -n 's/^#define NUX_RUNTIME_ABI_MAJOR //p' "${headers_dir}/nux_runtime.generated.h")"
header_abi_minor="$(sed -n 's/^#define NUX_RUNTIME_ABI_MINOR //p' "${headers_dir}/nux_runtime.generated.h")"
test "$(metadata_scalar abiMajor integer)" = "${header_abi_major}"
test "$(metadata_scalar abiMinor integer)" = "${header_abi_minor}"

phase "compile the C header smoke test"
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

phase "link the Swift smoke tests"
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
