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

full_device="$(find "${full_framework}" -path '*ios-arm64/libnux_capi.a' -print -quit)"
full_simulator="$(find "${full_framework}" -path '*ios-arm64_x86_64-simulator/libnux_capi.a' -print -quit)"
full_macos="$(find "${full_framework}" -path '*macos-arm64_x86_64/libnux_capi.a' -print -quit)"
ios_device="$(find "${ios_framework}" -path '*ios-arm64/libnux_capi.a' -print -quit)"
ios_simulator="$(find "${ios_framework}" -path '*ios-arm64_x86_64-simulator/libnux_capi.a' -print -quit)"
for library in "${full_device}" "${full_simulator}" "${full_macos}" "${ios_device}" "${ios_simulator}"; do
    test -f "${library}"
done
test "$(lipo -archs "${full_device}")" = "arm64"
test "$(lipo -archs "${ios_device}")" = "arm64"
for universal in "${full_simulator}" "${full_macos}" "${ios_simulator}"; do
    archs="$(lipo -archs "${universal}")"
    [[ "${archs}" == *arm64* ]]
    [[ "${archs}" == *x86_64* ]]
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
        "appleExtension=${repo_root}/crates/nux-capi/exports-v3-apple-metal-extension.txt" \
        "legacyMigration=${repo_root}/crates/nux-capi/exports-v3-legacy-migration.txt"
done

for target_and_library in "${target_libraries[@]}"; do
    target="${target_and_library%%:*}"
    library="${target_and_library#*:}"
    provenance="$(strings "${library}" | grep -F "\"target\":\"${target}\"" | head -1)"
    test -n "${provenance}"
    grep -Fq '"schemaVersion":6' <<< "${provenance}"
    grep -Fq '"rootPackage":"nux-capi"' <<< "${provenance}"
    grep -Fq "\"buildInputsHash\":\"${build_inputs_hash}\"" <<< "${provenance}"
done

for framework in "${full_framework}" "${ios_framework}"; do
    test -f "${framework}/LICENSE"
    test -f "${framework}/THIRD_PARTY_NOTICES.md"
    test -f "${framework}/BUILD_INPUTS.json"
    python3 "${script_dir}/apple_runtime_contract.py" inputs \
        "${framework}/BUILD_INPUTS.json" "${build_inputs_hash}"
done
cmp "${full_framework}/BUILD_INPUTS.json" "${ios_framework}/BUILD_INPUTS.json"
"${script_dir}/check-nux-capi-layout.py"

headers_dir="$(dirname "${full_macos}")/Headers"
expected_headers="$(printf '%s\n' module.modulemap nux_capi.generated.h nux_capi.h nux_capi_apple.h nux_runtime.generated.h nux_runtime.h)"
actual_headers="$(cd "${headers_dir}" && find . -mindepth 1 -maxdepth 1 -print | sed 's#^./##' | LC_ALL=C sort)"
test "${actual_headers}" = "${expected_headers}"

consumer_root="${verification_root}/consumers"
mkdir -p "${consumer_root}"
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
    -L "$(dirname "${full_macos}")" \
    -lnux_capi \
    "${repo_root}/crates/nux-capi/smoke/distribution_consumer.swift" \
    -o "${consumer_root}/swift-consumer"
"${consumer_root}/swift-consumer"

python3 - "${size_report_path}" "${full_archive}" "${ios_archive}" \
    "${full_framework}" "${ios_framework}" "${consumer_root}/swift-consumer" \
    "${target_libraries[@]}" <<'PY'
import json
import pathlib
import sys

output, full_archive, ios_archive, full_framework, ios_framework, linked, *specs = sys.argv[1:]
slice_bytes = {}
for specification in specs:
    target, path = specification.split(":", 1)
    slice_bytes[target] = pathlib.Path(path).stat().st_size

def expanded(path):
    return sum(item.stat().st_size for item in pathlib.Path(path).rglob("*") if item.is_file())

linked_bytes = pathlib.Path(linked).stat().st_size
document = {
    "schemaVersion": 1,
    "artifacts": {
        "full-apple": {
            "compressedBytes": pathlib.Path(full_archive).stat().st_size,
            "expandedBytes": expanded(full_framework),
            "representativeLinkedBytes": linked_bytes,
            "sliceBytes": dict(sorted(slice_bytes.items())),
        },
        "ios-only": {
            "compressedBytes": pathlib.Path(ios_archive).stat().st_size,
            "expandedBytes": expanded(ios_framework),
            "representativeLinkedBytes": linked_bytes,
            "sliceBytes": {key: slice_bytes[key] for key in sorted(slice_bytes) if "darwin" not in key},
        },
    },
}
pathlib.Path(output).write_text(json.dumps(document, indent=2) + "\n")
PY

python3 "${script_dir}/apple_runtime_contract.py" sizes \
    "${size_report_path}" "${repo_root}/crates/nux-capi/size-budgets-v3.json"
echo "nux-capi dual-XCFramework verification passed"
