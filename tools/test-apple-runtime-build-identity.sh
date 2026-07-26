#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd -P "$(dirname "${BASH_SOURCE[0]}")" && pwd -P)"
repo_root="$(cd -P "${script_dir}/.." && pwd -P)"
temporary_root="$(mktemp -d "${TMPDIR:-/tmp}/nux-apple-runtime-identity.XXXXXX")"
target_dir="${temporary_root}/target"
probe_path="${repo_root}/crates/nuxie-runtime/src/.nux-apple-runtime-identity-probe.$$"

cleanup() {
    rm -f "${probe_path}"
    case "${temporary_root}" in
        "${TMPDIR:-/tmp}"/nux-apple-runtime-identity.*)
            rm -rf "${temporary_root}"
            ;;
        *)
            echo "refusing to remove unexpected temporary path: ${temporary_root}" >&2
            exit 1
            ;;
    esac
}
trap cleanup EXIT

run_check() {
    local log_path="$1"
    env -u NUX_RUNTIME_SOURCE_REVISION \
        CARGO_TARGET_DIR="${target_dir}" \
        cargo check \
            --locked \
            --package nux-apple-runtime \
            --no-default-features \
            --verbose > "${log_path}" 2>&1
}

embedded_revision() {
    local build_output
    build_output="$(
        find "${target_dir}/debug/build" \
            -path "${target_dir}/debug/build/nux-apple-runtime-*/output" \
            -type f \
            -print |
            LC_ALL=C sort |
            tail -1
    )"
    test -n "${build_output}"
    sed -n \
        's/^cargo:rustc-env=NUX_RUNTIME_SOURCE_REVISION=//p' \
        "${build_output}" |
        tail -1
}

first_log="${temporary_root}/first.log"
fresh_log="${temporary_root}/fresh.log"
created_log="${temporary_root}/created.log"
changed_log="${temporary_root}/changed.log"
removed_log="${temporary_root}/removed.log"
target_noise_log="${temporary_root}/target-noise.log"
spoof_log="${temporary_root}/spoof.log"

run_check "${first_log}"
baseline_revision="$(embedded_revision)"
test -n "${baseline_revision}"

run_check "${fresh_log}"
grep -Fq "Fresh nux-apple-runtime" "${fresh_log}"
test "$(embedded_revision)" = "${baseline_revision}"

printf 'first identity probe\n' > "${probe_path}"
run_check "${created_log}"
grep -Fq "Dirty nux-apple-runtime" "${created_log}"
created_revision="$(embedded_revision)"
test "${created_revision}" != "${baseline_revision}"

printf 'second identity probe\n' > "${probe_path}"
run_check "${changed_log}"
grep -Fq "Dirty nux-apple-runtime" "${changed_log}"
changed_revision="$(embedded_revision)"
test "${changed_revision}" != "${created_revision}"
test "${changed_revision}" != "${baseline_revision}"

rm -f "${probe_path}"
run_check "${removed_log}"
grep -Fq "Dirty nux-apple-runtime" "${removed_log}"
test "$(embedded_revision)" = "${baseline_revision}"

printf 'ignored build output\n' > "${target_dir}/identity-noise"
run_check "${target_noise_log}"
grep -Fq "Fresh nux-apple-runtime" "${target_noise_log}"
test "$(embedded_revision)" = "${baseline_revision}"

spoofed_revision="0000000000000000000000000000000000000000"
if NUX_RUNTIME_SOURCE_REVISION="${spoofed_revision}" \
    CARGO_TARGET_DIR="${target_dir}" \
    cargo check \
        --locked \
        --package nux-apple-runtime \
        --no-default-features > "${spoof_log}" 2>&1; then
    echo "a caller-supplied false source revision was accepted" >&2
    exit 1
fi
grep -Fq \
    "NUX_RUNTIME_SOURCE_REVISION must match the exact clean or content-bound dirty Git identity" \
    "${spoof_log}"

echo "Apple runtime build identity invalidation: pass"
