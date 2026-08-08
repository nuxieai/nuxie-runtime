#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd -P "$(dirname "${BASH_SOURCE[0]}")" && pwd -P)"
repo_root="$(cd -P "${script_dir}/.." && pwd -P)"
profile="${NUX_APPLE_PROFILE:-release-apple}"
rust_toolchain="${NUX_APPLE_RUST_TOOLCHAIN:-1.94.1}"
rust_cargo="$(rustup which --toolchain "${rust_toolchain}" cargo)"
rust_compiler="$(rustup which --toolchain "${rust_toolchain}" rustc)"
rust_host="$("${rust_compiler}" -vV | sed -n 's/^host: //p')"
rust_sysroot="$("${rust_compiler}" --print sysroot)"
rust_llvm_nm="${rust_sysroot}/lib/rustlib/${rust_host}/bin/llvm-nm"
rust_llvm_objcopy="${rust_sysroot}/lib/rustlib/${rust_host}/bin/llvm-objcopy"
target_dir="${NUX_CAPI_MIGRATION_TARGET_DIR:-${repo_root}/target/nux-capi-migration-contract}"
work_dir="$(mktemp -d "${TMPDIR:-/tmp}/nux-capi-migration-contract.XXXXXX")"
trap 'rm -rf "${work_dir}"' EXIT

RUSTC="${rust_compiler}" "${rust_cargo}" build \
    --manifest-path "${repo_root}/Cargo.toml" \
    --locked \
    --package nux-capi \
    --no-default-features \
    --features legacy-migration \
    --profile "${profile}" \
    --target-dir "${target_dir}"

library="${target_dir}/${profile}/libnux_capi.a"
test -f "${library}"
stripped_library="${work_dir}/libnux_capi.a"
cp "${library}" "${stripped_library}"
"${rust_llvm_objcopy}" \
    --remove-section=__LLVM,__bitcode \
    --remove-section=__LLVM,__cmdline \
    "${stripped_library}"

cat \
    "${repo_root}/crates/nux-capi/exports-v3-portable.txt" \
    "${repo_root}/crates/nux-capi/exports-v3-apple-metal-extension.txt" \
    "${repo_root}/crates/nux-capi/exports-v3-legacy-migration.txt" |
    LC_ALL=C sort -u > "${work_dir}/expected-symbols.txt"
"${rust_llvm_nm}" -gjU "${stripped_library}" |
    sed 's/^_//' |
    awk '/^nux_[A-Za-z0-9_]+$/ { print }' |
    LC_ALL=C sort -u > "${work_dir}/actual-symbols.txt"
diff -u "${work_dir}/expected-symbols.txt" "${work_dir}/actual-symbols.txt"

headers="${work_dir}/Headers"
mkdir -p "${headers}"
cp "${repo_root}/crates/nux-capi/include/nux_capi.h" "${headers}/"
cp "${repo_root}/crates/nux-capi/include/nux_capi.generated.h" "${headers}/"
cp "${repo_root}/crates/nux-capi/include/nux_capi_apple.h" "${headers}/"
cp "${repo_root}/crates/nux-apple-runtime/include/nux_runtime.h" "${headers}/"
cp "${repo_root}/crates/nux-apple-runtime/include/nux_runtime.generated.h" "${headers}/"
cp "${repo_root}/crates/nux-capi/include/module.migration.modulemap" \
    "${headers}/module.modulemap"

swift_source="${work_dir}/migration_import.swift"
printf '%s\n' \
    'import NuxieRuntimeC' \
    'import NuxieRuntimeFFI' \
    'let mature = nux_capi_abi_version()' \
    'let legacy = nux_screen_session_result_is_settled(nil)' \
    '_ = (mature, legacy)' > "${swift_source}"
xcrun swiftc -typecheck -I "${headers}" "${swift_source}"

xcrun clang \
    -std=c11 -Wall -Wextra -Werror \
    -I "${headers}" \
    "${repo_root}/crates/nux-capi/smoke/distribution_consumer.c" \
    "${stripped_library}" \
    -framework Foundation -framework QuartzCore -framework Metal \
    -framework CoreGraphics -framework ImageIO -framework Security \
    -o "${work_dir}/c-consumer"
"${work_dir}/c-consumer"

xcrun clang \
    -std=c11 -Wall -Wextra -Werror \
    -I "${headers}" \
    "${repo_root}/crates/nux-capi/smoke/distribution_legacy_consumer.c" \
    "${stripped_library}" \
    -framework Foundation -framework QuartzCore -framework Metal \
    -framework CoreGraphics -framework ImageIO -framework Security \
    -o "${work_dir}/c-legacy-consumer"
"${work_dir}/c-legacy-consumer"

xcrun swiftc \
    -parse-as-library \
    -I "${headers}" \
    "${repo_root}/crates/nux-capi/smoke/distribution_consumer.swift" \
    "${stripped_library}" \
    -o "${work_dir}/swift-consumer"
"${work_dir}/swift-consumer"

echo "nux-capi migration archive and dual-module contract passed"
