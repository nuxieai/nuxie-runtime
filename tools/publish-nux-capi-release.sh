#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd -P "$(dirname "${BASH_SOURCE[0]}")" && pwd -P)"
repo_root="$(cd -P "${script_dir}/.." && pwd -P)"
release_tag="${1:-}"
artifact_root="${2:-${repo_root}/target/nux-capi-apple}"
full_archive="${artifact_root}/NuxieRuntime.xcframework.zip"
ios_archive="${artifact_root}/NuxieRuntime-iOS.xcframework.zip"
metadata="${artifact_root}/artifact-set.json"
size_report="${artifact_root}/SIZE_REPORT.json"

if [[ -z "${release_tag}" || $# -gt 2 ]]; then
    echo "usage: $0 apple-runtime-v<crate-version> [artifact-directory]" >&2
    exit 2
fi
runtime_version="$(sed -n 's/^version = "\([^"]*\)"/\1/p' "${repo_root}/crates/nux-capi/Cargo.toml" | head -1)"
expected_tag="apple-runtime-v${runtime_version}"
if [[ "${release_tag}" != "${expected_tag}" ]]; then
    echo "release tag ${release_tag} does not match ${expected_tag}" >&2
    exit 3
fi

source_revision="$(git -C "${repo_root}" rev-parse --verify HEAD)"
if [[ -n "$(git -C "${repo_root}" status --porcelain --untracked-files=all)" ]]; then
    echo "refusing to publish from a dirty worktree" >&2
    exit 4
fi
tagged_revision="$(git -C "${repo_root}" rev-list -n 1 "refs/tags/${release_tag}")"
test "${tagged_revision}" = "${source_revision}"
git -C "${repo_root}" fetch --no-tags origin refs/heads/main:refs/remotes/origin/main
git -C "${repo_root}" merge-base --is-ancestor "${source_revision}" refs/remotes/origin/main
for path in "${full_archive}" "${ios_archive}" "${metadata}" "${size_report}"; do
    test -f "${path}"
done

build_source_revision="$(python3 "${script_dir}/json-scalar.py" "${metadata}" buildSourceRevision string)"
if [[ ! "${build_source_revision}" =~ ^[0-9a-f]{40}$ ]]; then
    echo "release artifacts were not built from a clean revision" >&2
    exit 5
fi
git -C "${repo_root}" merge-base --is-ancestor "${build_source_revision}" "${source_revision}"
test "${build_source_revision}" = "${source_revision}"

qualified_metadata="$(mktemp "${artifact_root}/.artifact-qualified.XXXXXX")"
trap 'rm -f "${qualified_metadata}"' EXIT
cp "${metadata}" "${qualified_metadata}"
python3 "${script_dir}/apple_runtime_contract.py" release "${qualified_metadata}" "${source_revision}"
python3 "${script_dir}/apple_runtime_contract.py" distribution "${qualified_metadata}"
"${script_dir}/verify-nux-capi-xcframeworks.sh" \
    "${artifact_root}" "${qualified_metadata}" \
    "$(python3 "${script_dir}/json-scalar.py" "${qualified_metadata}" buildInputsHash string)" \
    "${size_report}"
python3 "${script_dir}/apple_runtime_contract.py" sizes \
    "${size_report}" "${repo_root}/crates/nux-capi/size-budgets-v3.json" --release
mv "${qualified_metadata}" "${metadata}"
trap - EXIT

if gh release view "${release_tag}" --repo nuxieai/nuxie-runtime >/dev/null 2>&1; then
    echo "release ${release_tag} already exists; immutable assets are never replaced" >&2
    exit 6
fi

notes="$(mktemp "${TMPDIR:-/tmp}/nuxie-runtime-release.XXXXXX")"
downloads="$(mktemp -d "${TMPDIR:-/tmp}/nuxie-runtime-download.XXXXXX")"
cleanup() {
    rm -f "${notes}"
    rm -rf "${downloads}"
}
trap cleanup EXIT
printf 'Nuxie runtime %s\n\nSource: `%s`\n' "${runtime_version}" "${source_revision}" > "${notes}"
gh release create "${release_tag}" \
    "${full_archive}#NuxieRuntime.xcframework.zip" \
    "${ios_archive}#NuxieRuntime-iOS.xcframework.zip" \
    "${metadata}#artifact-set.json" \
    "${size_report}#SIZE_REPORT.json" \
    --repo nuxieai/nuxie-runtime \
    --verify-tag \
    --title "Nuxie runtime ${runtime_version}" \
    --notes-file "${notes}"

gh release download "${release_tag}" \
    --repo nuxieai/nuxie-runtime \
    --dir "${downloads}" \
    --pattern NuxieRuntime.xcframework.zip \
    --pattern NuxieRuntime-iOS.xcframework.zip \
    --pattern artifact-set.json \
    --pattern SIZE_REPORT.json
for asset in NuxieRuntime.xcframework.zip NuxieRuntime-iOS.xcframework.zip artifact-set.json SIZE_REPORT.json; do
    cmp "${artifact_root}/${asset}" "${downloads}/${asset}"
done

mkdir -p "${downloads}/full" "${downloads}/ios"
ditto -x -k "${downloads}/NuxieRuntime.xcframework.zip" "${downloads}/full"
ditto -x -k "${downloads}/NuxieRuntime-iOS.xcframework.zip" "${downloads}/ios"
"${script_dir}/verify-nux-capi-xcframeworks.sh" \
    "${downloads}" "${downloads}/artifact-set.json" \
    "$(python3 "${script_dir}/json-scalar.py" "${downloads}/artifact-set.json" buildInputsHash string)" \
    "${downloads}/SIZE_REPORT.json"
echo "Published immutable ${release_tag} with both Apple artifacts"
