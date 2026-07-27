#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd -P "$(dirname "${BASH_SOURCE[0]}")" && pwd -P)"
if [[ "$#" -gt 1 ]]; then
    echo "usage: $0 [repository-root]" >&2
    exit 2
fi
if [[ "$#" -eq 1 ]]; then
    repo_root="$(cd -P "${1}" && pwd -P)"
else
    repo_root="$(cd -P "${script_dir}/.." && pwd -P)"
fi
actual_repo_root="$(git -C "${repo_root}" rev-parse --show-toplevel)"
if [[ "${actual_repo_root}" != "${repo_root}" ]]; then
    echo "repository root must be the Git worktree root: ${repo_root}" >&2
    exit 2
fi

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
    (cd "${repo_root}" && env -u NUX_RUNTIME_SOURCE_REVISION \
        CARGO_TARGET_DIR="${target_dir}" \
        cargo check \
            --locked \
            --package nux-apple-runtime \
            --no-default-features \
            --verbose) > "${log_path}" 2>&1
}

expected_revision() {
    local head
    local tracked_diff="${temporary_root}/expected-tracked.diff"
    local untracked_paths="${temporary_root}/expected-untracked.paths"
    local identity_input="${temporary_root}/expected-identity.input"
    local source_path
    local digest

    head="$(git -C "${repo_root}" rev-parse --verify HEAD)"
    git -C "${repo_root}" \
        diff --binary --no-ext-diff HEAD -- > "${tracked_diff}"
    git -C "${repo_root}" \
        ls-files --others --exclude-standard -z -- crates vendor \
        > "${untracked_paths}"
    if [[ ! -s "${tracked_diff}" && ! -s "${untracked_paths}" ]]; then
        printf '%s\n' "${head}"
        return
    fi

    printf '%s\0' "${head}" > "${identity_input}"
    cat "${tracked_diff}" >> "${identity_input}"
    while IFS= read -r -d '' source_path; do
        printf '%s\0' "${source_path}" >> "${identity_input}"
        cat "${repo_root}/${source_path}" >> "${identity_input}"
        printf '\0' >> "${identity_input}"
    done < "${untracked_paths}"
    digest="$(shasum -a 256 "${identity_input}" | awk '{ print $1 }')"
    test -n "${digest}"
    printf '%s-dirty.%s\n' "${head}" "${digest}"
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

assert_embedded_revision() {
    local phase="$1"
    local expected="$2"
    local actual
    actual="$(embedded_revision)"
    if [[ "${actual}" != "${expected}" ]]; then
        echo "${phase} embedded revision mismatch" >&2
        echo "expected=${expected}" >&2
        echo "actual=${actual}" >&2
        exit 1
    fi
}

first_log="${temporary_root}/first.log"
no_op_log="${temporary_root}/no-op.log"
created_log="${temporary_root}/created.log"
changed_log="${temporary_root}/changed.log"
removed_log="${temporary_root}/removed.log"
target_noise_log="${temporary_root}/target-noise.log"
spoof_log="${temporary_root}/spoof.log"

baseline_revision="$(expected_revision)"
run_check "${first_log}"
assert_embedded_revision "first build" "${baseline_revision}"

run_check "${no_op_log}"
assert_embedded_revision "no-op build" "${baseline_revision}"

printf 'first identity probe\n' > "${probe_path}"
created_revision="$(expected_revision)"
test "${created_revision}" != "${baseline_revision}"
run_check "${created_log}"
assert_embedded_revision "source-create build" "${created_revision}"

printf 'second identity probe\n' > "${probe_path}"
changed_revision="$(expected_revision)"
test "${changed_revision}" != "${created_revision}"
test "${changed_revision}" != "${baseline_revision}"
run_check "${changed_log}"
assert_embedded_revision "source-change build" "${changed_revision}"

rm -f "${probe_path}"
removed_revision="$(expected_revision)"
test "${removed_revision}" = "${baseline_revision}"
run_check "${removed_log}"
assert_embedded_revision "source-remove build" "${removed_revision}"

printf 'ignored build output\n' > "${target_dir}/identity-noise"
target_noise_revision="$(expected_revision)"
test "${target_noise_revision}" = "${baseline_revision}"
run_check "${target_noise_log}"
assert_embedded_revision "target-noise build" "${target_noise_revision}"

spoofed_revision="0000000000000000000000000000000000000000"
if (cd "${repo_root}" && \
    NUX_RUNTIME_SOURCE_REVISION="${spoofed_revision}" \
        CARGO_TARGET_DIR="${target_dir}" \
        cargo check \
            --locked \
            --package nux-apple-runtime \
            --no-default-features) > "${spoof_log}" 2>&1; then
    echo "a caller-supplied false source revision was accepted" >&2
    exit 1
fi
grep -Fq \
    "NUX_RUNTIME_SOURCE_REVISION must match the exact clean or content-bound dirty Git identity" \
    "${spoof_log}"

echo "Apple runtime build identity invalidation: pass"
