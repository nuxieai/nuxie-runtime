#!/usr/bin/env bash
set -euo pipefail

if [[ $# -ne 4 ]]; then
    echo "usage: $0 <distribution-root> <artifact-set.json> <build-inputs-hash> <size-report-output>" >&2
    exit 2
fi

distribution_root="$1"
metadata_path="$2"
build_inputs_hash="$3"
size_report_path="$4"
script_dir="$(cd -P "$(dirname "${BASH_SOURCE[0]}")" && pwd -P)"
repo_root="$(cd -P "${script_dir}/.." && pwd -P)"
full_framework="${distribution_root}/full/NuxieRuntime.xcframework"
ios_framework="${distribution_root}/ios/NuxieRuntime.xcframework"
full_archive="${distribution_root}/NuxieRuntime.xcframework.zip"
ios_archive="${distribution_root}/NuxieRuntime-iOS.xcframework.zip"
rust_toolchain="${NUX_APPLE_RUST_TOOLCHAIN:-1.94.1}"
rust_compiler="$(rustup which --toolchain "${rust_toolchain}" rustc)"
rust_host="$("${rust_compiler}" -vV | sed -n 's/^host: //p')"
rust_sysroot="$("${rust_compiler}" --print sysroot)"
rust_llvm_nm="${rust_sysroot}/lib/rustlib/${rust_host}/bin/llvm-nm"
rust_llvm_readobj="${rust_sysroot}/lib/rustlib/${rust_host}/bin/llvm-readobj"
verification_root="$(mktemp -d "${TMPDIR:-/tmp}/nux-capi-distribution.XXXXXX")"
trap 'rm -rf "${verification_root}"' EXIT

python3 "${script_dir}/apple_runtime_contract.py" distribution "${metadata_path}"
test -f "${full_archive}"
test -f "${ios_archive}"
test -d "${full_framework}"
test -d "${ios_framework}"
plutil -lint "${full_framework}/Info.plist"
plutil -lint "${ios_framework}/Info.plist"

expected_checksum() {
    python3 - "${metadata_path}" "$1" <<'PY'
import json
import pathlib
import sys
artifacts = json.loads(pathlib.Path(sys.argv[1]).read_text())["artifacts"]
print(next(artifact["swiftPackageChecksum"] for artifact in artifacts if artifact["kind"] == sys.argv[2]))
PY
}
test "$(swift package compute-checksum "${full_archive}")" = "$(expected_checksum full-apple)"
test "$(swift package compute-checksum "${ios_archive}")" = "$(expected_checksum ios-only)"

mkdir -p "${verification_root}/full" "${verification_root}/ios"
ditto -x -k "${full_archive}" "${verification_root}/full"
ditto -x -k "${ios_archive}" "${verification_root}/ios"
diff -rq "${full_framework}" "${verification_root}/full/NuxieRuntime.xcframework" >/dev/null
diff -rq "${ios_framework}" "${verification_root}/ios/NuxieRuntime.xcframework" >/dev/null

platform_directories() {
    find "$1" -mindepth 1 -maxdepth 1 -type d -exec basename {} \; | LC_ALL=C sort
}
root_entries() {
    find "$1" -mindepth 1 -maxdepth 1 -exec basename {} \; | LC_ALL=C sort
}
expected_full_platforms="$(printf '%s\n' ios-arm64 ios-arm64_x86_64-simulator macos-arm64_x86_64 | LC_ALL=C sort)"
expected_ios_platforms="$(printf '%s\n' ios-arm64 ios-arm64_x86_64-simulator | LC_ALL=C sort)"
test "$(platform_directories "${full_framework}")" = "${expected_full_platforms}"
test "$(platform_directories "${ios_framework}")" = "${expected_ios_platforms}"
expected_full_entries="$(printf '%s\n' BUILD_INPUTS.json Info.plist LICENSE THIRD_PARTY_NOTICES.md ${expected_full_platforms} | LC_ALL=C sort)"
expected_ios_entries="$(printf '%s\n' BUILD_INPUTS.json Info.plist LICENSE THIRD_PARTY_NOTICES.md ${expected_ios_platforms} | LC_ALL=C sort)"
test "$(root_entries "${full_framework}")" = "${expected_full_entries}"
test "$(root_entries "${ios_framework}")" = "${expected_ios_entries}"

single_library() {
    directory="$1"
    test -d "${directory}/Headers"
    entry_count="$(find "${directory}" -mindepth 1 -maxdepth 1 -print | wc -l | tr -d '[:space:]')"
    test "${entry_count}" = 2
    count="$(find "${directory}" -maxdepth 1 -type f -name '*.a' -print | wc -l | tr -d '[:space:]')"
    test "${count}" = 1
    find "${directory}" -maxdepth 1 -type f -name '*.a' -print
}
full_device="$(single_library "${full_framework}/ios-arm64")"
full_simulator="$(single_library "${full_framework}/ios-arm64_x86_64-simulator")"
full_macos="$(single_library "${full_framework}/macos-arm64_x86_64")"
ios_device="$(single_library "${ios_framework}/ios-arm64")"
ios_simulator="$(single_library "${ios_framework}/ios-arm64_x86_64-simulator")"
cmp "${full_device}" "${ios_device}"
cmp "${full_simulator}" "${ios_simulator}"
normalized_archs() {
    lipo -archs "$1" | tr ' ' '\n' | sed '/^$/d' | LC_ALL=C sort | paste -sd ' ' -
}
test "$(normalized_archs "${full_device}")" = "arm64"
test "$(normalized_archs "${ios_device}")" = "arm64"
for universal in "${full_simulator}" "${full_macos}" "${ios_simulator}"; do
    test "$(normalized_archs "${universal}")" = "arm64 x86_64"
done

declare -a thin_libraries=()
declare -a target_libraries=()
thin_libraries+=("${full_device}")
target_libraries+=("aarch64-apple-ios:${full_device}")
for specification in \
    "aarch64-apple-ios-sim:${full_simulator}:arm64" \
    "x86_64-apple-ios:${full_simulator}:x86_64" \
    "aarch64-apple-darwin:${full_macos}:arm64" \
    "x86_64-apple-darwin:${full_macos}:x86_64"; do
    target="${specification%%:*}"
    remainder="${specification#*:}"
    universal="${remainder%:*}"
    architecture="${specification##*:}"
    thin="${verification_root}/libnux_capi-${target}.a"
    lipo "${universal}" -thin "${architecture}" -output "${thin}"
    thin_libraries+=("${thin}")
    target_libraries+=("${target}:${thin}")
done

python3 "${script_dir}/apple_runtime_contract.py" header-symbols \
    "${repo_root}/crates/nux-capi/include/nux_capi.generated.h" \
    "portable=${repo_root}/crates/nux-capi/exports-v3-portable.txt" \
    "appleExtension=${repo_root}/crates/nux-capi/exports-v3-apple-metal-extension.txt"

for library in "${thin_libraries[@]}"; do
    sections="$("${rust_llvm_readobj}" --sections "${library}")"
    if grep -q 'Segment: __LLVM' <<< "${sections}"; then
        echo "embedded LLVM bitcode remains in ${library}" >&2
        exit 1
    fi
    symbols_path="${verification_root}/$(basename "${library}").symbols"
    "${rust_llvm_nm}" -gjU "${library}" > "${symbols_path}"
    python3 "${script_dir}/apple_runtime_contract.py" symbol-partitions \
        "${symbols_path}" \
        "portable=${repo_root}/crates/nux-capi/exports-v3-portable.txt" \
        "appleExtension=${repo_root}/crates/nux-capi/exports-v3-apple-metal-extension.txt"
done

for framework in "${full_framework}" "${ios_framework}"; do
    test -f "${framework}/LICENSE"
    test -f "${framework}/THIRD_PARTY_NOTICES.md"
    test -f "${framework}/BUILD_INPUTS.json"
    python3 "${script_dir}/apple_runtime_contract.py" inputs \
        "${framework}/BUILD_INPUTS.json" "${build_inputs_hash}"
done
cmp "${full_framework}/BUILD_INPUTS.json" "${ios_framework}/BUILD_INPUTS.json"

for target_and_library in "${target_libraries[@]}"; do
    target="${target_and_library%%:*}"
    library="${target_and_library#*:}"
    provenance_strings="${verification_root}/$(basename "${library}").provenance-strings"
    strings "${library}" > "${provenance_strings}"
    python3 "${script_dir}/apple_runtime_contract.py" slice-provenance \
        "${provenance_strings}" "${metadata_path}" \
        "${full_framework}/BUILD_INPUTS.json" "${target}"
done
"${script_dir}/check-nux-capi-layout.py"

headers_dir="$(dirname "${full_macos}")/Headers"
expected_headers="$(printf '%s\n' module.modulemap nux_capi.generated.h nux_capi.h nux_capi_apple.h)"
while IFS= read -r packaged_headers; do
    actual_headers="$(cd "${packaged_headers}" && find . -mindepth 1 -maxdepth 1 -print | sed 's#^./##' | LC_ALL=C sort)"
    test "${actual_headers}" = "${expected_headers}"
    for header in ${expected_headers}; do
        cmp "${headers_dir}/${header}" "${packaged_headers}/${header}"
    done
done < <(find "${full_framework}" "${ios_framework}" -type d -name Headers -print | LC_ALL=C sort)

consumer_root="${verification_root}/consumers"
mkdir -p "${consumer_root}"
composed_fixture="${verification_root}/composed_script_asset.riv"
python3 - "${repo_root}/crates/nux-capi/smoke/composed_script_asset.riv.base64" \
    "${composed_fixture}" <<'PY'
import base64
import pathlib
import sys

source, output = map(pathlib.Path, sys.argv[1:])
output.write_bytes(base64.b64decode(source.read_text().strip(), validate=True))
PY
macos_sdk_path="$(xcrun --sdk macosx --show-sdk-path)"
xcrun --sdk macosx clang \
    -std=c11 -Wall -Wextra -Werror \
    -isysroot "${macos_sdk_path}" \
    -mmacosx-version-min="${NUX_APPLE_MACOS_DEPLOYMENT_TARGET:-12.0}" \
    -I"${headers_dir}" \
    "${repo_root}/crates/nux-capi/smoke/distribution_consumer.c" \
    "${full_macos}" \
    -framework Foundation -framework QuartzCore -framework Metal \
    -framework CoreGraphics -framework ImageIO -framework Security \
    -o "${consumer_root}/c-consumer"
"${consumer_root}/c-consumer"

xcrun --sdk macosx swiftc \
    -parse-as-library \
    -sdk "${macos_sdk_path}" \
    -target "arm64-apple-macos${NUX_APPLE_MACOS_DEPLOYMENT_TARGET:-12.0}" \
    -I "${headers_dir}" \
    "${repo_root}/crates/nux-capi/smoke/distribution_consumer.swift" \
    "${full_macos}" \
    -o "${consumer_root}/swift-consumer"
"${consumer_root}/swift-consumer"

xcrun --sdk macosx clang \
    -std=c11 -Wall -Wextra -Werror \
    -target "arm64-apple-macos${NUX_APPLE_MACOS_DEPLOYMENT_TARGET:-12.0}" \
    -isysroot "${macos_sdk_path}" \
    -I"${headers_dir}" \
    "${repo_root}/crates/nux-capi/smoke/capi_metal_smoke.c" \
    "${full_macos}" \
    -framework Foundation -framework QuartzCore -framework Metal \
    -framework CoreFoundation -framework CoreGraphics -framework ImageIO \
    -framework Security -liconv \
    -o "${consumer_root}/c-behavior-consumer"
"${consumer_root}/c-behavior-consumer" "${composed_fixture}" --composed

xcrun --sdk macosx swiftc \
    -warnings-as-errors \
    -sdk "${macos_sdk_path}" \
    -target "arm64-apple-macos${NUX_APPLE_MACOS_DEPLOYMENT_TARGET:-12.0}" \
    -I "${headers_dir}" \
    "${repo_root}/crates/nux-capi/smoke/capi_metal_smoke.swift" \
    "${full_macos}" \
    -framework CoreFoundation -framework CoreGraphics -framework ImageIO \
    -framework QuartzCore -framework Metal -framework Foundation -framework Security \
    -Xlinker -liconv \
    -o "${consumer_root}/swift-behavior-consumer"
"${consumer_root}/swift-behavior-consumer" "${composed_fixture}" --composed

device_headers="$(dirname "${full_device}")/Headers"
iphoneos_sdk_path="$(xcrun --sdk iphoneos --show-sdk-path)"
xcrun --sdk iphoneos clang \
    -target "arm64-apple-ios${NUX_APPLE_DEPLOYMENT_TARGET:-15.0}" \
    -std=c11 -Wall -Wextra -Werror \
    -isysroot "${iphoneos_sdk_path}" \
    -I"${device_headers}" \
    "${repo_root}/crates/nux-capi/smoke/distribution_consumer.c" \
    "${full_device}" \
    -framework Foundation -framework QuartzCore -framework Metal \
    -framework CoreGraphics -framework ImageIO -framework Security \
    -o "${consumer_root}/c-consumer-ios"
xcrun --sdk iphoneos swiftc \
    -parse-as-library \
    -sdk "${iphoneos_sdk_path}" \
    -target "arm64-apple-ios${NUX_APPLE_DEPLOYMENT_TARGET:-15.0}" \
    -I "${device_headers}" \
    "${repo_root}/crates/nux-capi/smoke/distribution_consumer.swift" \
    "${full_device}" \
    -o "${consumer_root}/swift-consumer-ios"

xcrun --sdk iphoneos clang \
    -target "arm64-apple-ios${NUX_APPLE_DEPLOYMENT_TARGET:-15.0}" \
    -std=c11 -Wall -Wextra -Werror \
    -isysroot "${iphoneos_sdk_path}" \
    -I"${device_headers}" \
    "${repo_root}/crates/nux-capi/smoke/capi_metal_smoke.c" \
    "${full_device}" \
    -framework Foundation -framework QuartzCore -framework Metal \
    -framework CoreFoundation -framework CoreGraphics -framework ImageIO \
    -framework Security -liconv \
    -o "${consumer_root}/c-behavior-consumer-ios"
xcrun --sdk iphoneos swiftc \
    -warnings-as-errors \
    -sdk "${iphoneos_sdk_path}" \
    -target "arm64-apple-ios${NUX_APPLE_DEPLOYMENT_TARGET:-15.0}" \
    -I "${device_headers}" \
    "${repo_root}/crates/nux-capi/smoke/capi_metal_smoke.swift" \
    "${full_device}" \
    -framework CoreFoundation -framework CoreGraphics -framework ImageIO \
    -framework QuartzCore -framework Metal -framework Foundation -framework Security \
    -Xlinker -liconv \
    -o "${consumer_root}/swift-behavior-consumer-ios"

python3 - "${size_report_path}" \
    "${repo_root}/crates/nux-capi/size-baseline-apple-runtime-v0.4.0.json" \
    "${full_archive}" "${ios_archive}" \
    "${full_framework}" "${ios_framework}" \
    "${consumer_root}/c-behavior-consumer" "${consumer_root}/swift-behavior-consumer" \
    "${consumer_root}/c-behavior-consumer-ios" "${consumer_root}/swift-behavior-consumer-ios" \
    "${target_libraries[@]}" <<'PY'
import json
import pathlib
import sys

(
    output, baseline_path, full_archive, ios_archive, full_framework, ios_framework,
    c_macos, swift_macos, c_ios, swift_ios, *specs
) = sys.argv[1:]
slice_bytes = {}
for specification in specs:
    target, path = specification.split(":", 1)
    slice_bytes[target] = pathlib.Path(path).stat().st_size

def expanded(path):
    return sum(item.stat().st_size for item in pathlib.Path(path).rglob("*") if item.is_file())

current = {
        "full-apple": {
            "compressedBytes": pathlib.Path(full_archive).stat().st_size,
            "expandedBytes": expanded(full_framework),
            "representativeLinkedBytes": {
                "c-macos-arm64": pathlib.Path(c_macos).stat().st_size,
                "swift-macos-arm64": pathlib.Path(swift_macos).stat().st_size,
            },
            "sliceBytes": dict(sorted(slice_bytes.items())),
        },
        "ios-only": {
            "compressedBytes": pathlib.Path(ios_archive).stat().st_size,
            "expandedBytes": expanded(ios_framework),
            "representativeLinkedBytes": {
                "c-ios-arm64": pathlib.Path(c_ios).stat().st_size,
                "swift-ios-arm64": pathlib.Path(swift_ios).stat().st_size,
            },
            "sliceBytes": {key: slice_bytes[key] for key in sorted(slice_bytes) if "darwin" not in key},
        },
}
baseline_document = json.loads(pathlib.Path(baseline_path).read_text())
baseline = {
    key: baseline_document[key]
    for key in ("releaseTag", "sourceRevision", "sizeReportSha256", "artifacts")
}

def subtract(after, before):
    return {
        "compressedBytes": after["compressedBytes"] - before["compressedBytes"],
        "expandedBytes": after["expandedBytes"] - before["expandedBytes"],
        "representativeLinkedBytes": {
            key: value - before["representativeLinkedBytes"][key]
            for key, value in after["representativeLinkedBytes"].items()
        },
        "sliceBytes": {
            key: value - before["sliceBytes"][key]
            for key, value in after["sliceBytes"].items()
        },
    }

document = {
    "schemaVersion": 2,
    "baseline": baseline,
    "artifacts": current,
    "deltasFromBaseline": {
        kind: subtract(metrics, baseline["artifacts"][kind])
        for kind, metrics in current.items()
    },
}
pathlib.Path(output).write_text(json.dumps(document, indent=2) + "\n")
PY

python3 "${script_dir}/apple_runtime_contract.py" sizes \
    "${size_report_path}" "${repo_root}/crates/nux-capi/size-budgets-v3.json"
python3 "${script_dir}/check-nux-capi-surface.py"
echo "nux-capi dual-XCFramework verification passed"
