#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd -P "$(dirname "${BASH_SOURCE[0]}")" && pwd -P)"
repo_root="$(cd -P "${script_dir}/.." && pwd -P)"
release_tag="${1:-}"
artifact_root="${2:-${repo_root}/target/apple-runtime}"
archive="${artifact_root}/NuxieRuntime.xcframework.zip"
metadata="${artifact_root}/artifact.json"
xcframework="${artifact_root}/NuxieRuntime.xcframework"

if [[ -z "${release_tag}" || $# -gt 2 ]]; then
    echo "usage: $0 apple-runtime-v<crate-version> [artifact-directory]" >&2
    exit 2
fi

runtime_version="$(
    sed -n 's/^version = "\([^"]*\)"/\1/p' \
        "${repo_root}/crates/nux-apple-runtime/Cargo.toml" |
        head -1
)"
expected_tag="apple-runtime-v${runtime_version}"
if [[ "${release_tag}" != "${expected_tag}" ]]; then
    echo "release tag ${release_tag} does not match crate version; expected ${expected_tag}" >&2
    exit 3
fi

source_revision="$(git -C "${repo_root}" rev-parse --verify HEAD)"
if [[ -n "$(git -C "${repo_root}" status --porcelain --untracked-files=all)" ]]; then
    echo "refusing to publish from a dirty runtime worktree" >&2
    exit 4
fi
tagged_revision="$(git -C "${repo_root}" rev-list -n 1 "refs/tags/${release_tag}")"
test "${source_revision}" = "${tagged_revision}"
git -C "${repo_root}" fetch --no-tags origin \
    refs/heads/main:refs/remotes/origin/main
git -C "${repo_root}" merge-base --is-ancestor \
    "${source_revision}" refs/remotes/origin/main

for path in "${xcframework}" "${archive}" "${metadata}"; do
    if [[ ! -e "${path}" ]]; then
        echo "release input is missing: ${path}" >&2
        exit 5
    fi
done

"${script_dir}/verify-apple-xcframework.sh" \
    "${xcframework}" "${archive}" "${metadata}"
test "$("${script_dir}/json-scalar.py" "${metadata}" runtimeVersion string)" = "${runtime_version}"
test "$("${script_dir}/json-scalar.py" "${metadata}" sourceRevision string)" = "${source_revision}"
test "$("${script_dir}/json-scalar.py" "${metadata}" buildProfile string)" = "release-apple"
test "$("${script_dir}/json-scalar.py" "${metadata}" rustToolchain string)" = "1.94.1"
test "$("${script_dir}/json-scalar.py" "${metadata}" minimumIOSVersion string)" = "15.0"
test "$("${script_dir}/json-scalar.py" "${metadata}" minimumMacOSVersion string)" = "12.0"
checksum="$(swift package compute-checksum "${archive}")"
test "$("${script_dir}/json-scalar.py" "${metadata}" swiftPackageChecksum string)" = "${checksum}"

if gh release view "${release_tag}" --repo nuxieai/nuxie-runtime >/dev/null 2>&1; then
    echo "release ${release_tag} already exists; published assets are never replaced" >&2
    exit 6
fi

notes_path="$(mktemp "${TMPDIR:-/tmp}/nuxie-apple-release-notes.XXXXXX")"
download_root="$(mktemp -d "${TMPDIR:-/tmp}/nuxie-apple-release-download.XXXXXX")"
cleanup() {
    rm -f "${notes_path}"
    rm -rf "${download_root}"
}
trap cleanup EXIT
{
    echo "Nuxie Apple runtime ${runtime_version}"
    echo
    echo "Source: \`${source_revision}\`"
    echo "SwiftPM checksum: \`${checksum}\`"
    echo "Minimum platforms: iOS 15.0, macOS 12.0"
} > "${notes_path}"

gh release create "${release_tag}" \
    "${archive}#NuxieRuntime.xcframework.zip" \
    "${metadata}#artifact.json" \
    --repo nuxieai/nuxie-runtime \
    --verify-tag \
    --title "Nuxie Apple runtime ${runtime_version}" \
    --notes-file "${notes_path}"

gh release download "${release_tag}" \
    --repo nuxieai/nuxie-runtime \
    --dir "${download_root}" \
    --pattern NuxieRuntime.xcframework.zip \
    --pattern artifact.json
cmp "${archive}" "${download_root}/NuxieRuntime.xcframework.zip"
cmp "${metadata}" "${download_root}/artifact.json"

unpacked_root="${download_root}/unpacked"
mkdir -p "${unpacked_root}"
ditto -x -k "${download_root}/NuxieRuntime.xcframework.zip" "${unpacked_root}"
"${script_dir}/verify-apple-xcframework.sh" \
    "${unpacked_root}/NuxieRuntime.xcframework" \
    "${download_root}/NuxieRuntime.xcframework.zip" \
    "${download_root}/artifact.json"

echo "Published ${release_tag}"
echo "URL: https://github.com/nuxieai/nuxie-runtime/releases/download/${release_tag}/NuxieRuntime.xcframework.zip"
echo "Checksum: ${checksum}"
