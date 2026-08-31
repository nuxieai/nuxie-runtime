#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd -P "$(dirname "${BASH_SOURCE[0]}")" && pwd -P)"
repo_root="$(cd -P "${script_dir}/.." && pwd -P)"
expected_tag="android-runtime-v0.3.7"

if [[ "${1:-}" == "--plan" ]]; then
    printf '%s\n' \
        '1. require clean source commit equal to origin/main' \
        '2. require local and remote android-runtime-v0.3.7 tags at that source commit' \
        '3. verify ABI4, ELF, provenance, checksums, and size evidence' \
        '4. create a GitHub draft with the immutable archive and three evidence assets' \
        '5. download every draft asset and compare it byte-for-byte with the qualified local asset' \
        '6. verify the downloaded artifact independently' \
        '7. publish the already-verified draft without replacing any asset'
    exit 0
fi

release_tag="${1:-}"
artifact_root="${2:-${repo_root}/target/nux-capi-android}"
if [[ -z "${release_tag}" || $# -gt 2 ]]; then
    echo "usage: $0 android-runtime-v0.3.7 [artifact-directory] | --plan" >&2
    exit 2
fi
if [[ "${release_tag}" != "${expected_tag}" ]]; then
    echo "release tag ${release_tag} does not match immutable ${expected_tag}" >&2
    exit 3
fi

archive="${artifact_root}/NuxieRuntimeAndroid.zip"
metadata="${artifact_root}/NuxieRuntimeAndroid.json"
build_inputs="${artifact_root}/NuxieRuntimeAndroid-BUILD_INPUTS.json"
size_report="${artifact_root}/NuxieRuntimeAndroid-SIZE_REPORT.json"
assets=("${archive}" "${metadata}" "${build_inputs}" "${size_report}")
for path in "${assets[@]}"; do
    if [[ ! -f "${path}" ]]; then
        echo "required Android release asset is missing: ${path}" >&2
        exit 3
    fi
done

source_revision="$(git -C "${repo_root}" rev-parse --verify HEAD)"
if [[ -n "$(git -C "${repo_root}" status --porcelain --untracked-files=all)" ]]; then
    echo "refusing to publish from a dirty runtime tree" >&2
    exit 4
fi
git -C "${repo_root}" fetch --no-tags origin refs/heads/main:refs/remotes/origin/main
if [[ "${source_revision}" != "$(git -C "${repo_root}" rev-parse refs/remotes/origin/main)" ]]; then
    echo "release source must be exactly origin/main" >&2
    exit 4
fi
tagged_revision="$(git -C "${repo_root}" rev-list -n 1 "refs/tags/${release_tag}")"
if [[ "${tagged_revision}" != "${source_revision}" ]]; then
    echo "local ${release_tag} does not resolve to the release source" >&2
    exit 4
fi
remote_tag_records="$(git -C "${repo_root}" ls-remote --exit-code origin \
    "refs/tags/${release_tag}" "refs/tags/${release_tag}^{}")"
remote_tag_revision="$(awk '$2 ~ /\^\{\}$/ { print $1 }' <<< "${remote_tag_records}")"
if [[ -z "${remote_tag_revision}" ]]; then
    remote_tag_revision="$(awk '$2 !~ /\^\{\}$/ { print $1 }' <<< "${remote_tag_records}")"
fi
if [[ "${remote_tag_revision}" != "${source_revision}" ]]; then
    echo "remote ${release_tag} does not resolve to the release source" >&2
    exit 4
fi

python3 "${script_dir}/android_runtime_contract.py" verify \
    --repo-root "${repo_root}" \
    --artifact-root "${artifact_root}" \
    --release-revision "${source_revision}"

if gh release view "${release_tag}" --repo nuxieai/nuxie-runtime >/dev/null 2>&1; then
    echo "release ${release_tag} already exists; immutable assets are never replaced" >&2
    exit 5
fi

notes="$(mktemp "${TMPDIR:-/tmp}/nuxie-android-release-notes.XXXXXX")"
downloads="$(mktemp -d "${TMPDIR:-/tmp}/nuxie-android-release-download.XXXXXX")"
cleanup() {
    rm -f "${notes}"
    rm -rf "${downloads}"
}
trap cleanup EXIT
python3 - "${metadata}" "${build_inputs}" "${size_report}" "${notes}" <<'PY'
import json
import pathlib
import sys

metadata_path, inputs_path, sizes_path, notes_path = map(pathlib.Path, sys.argv[1:])
metadata = json.loads(metadata_path.read_text())
inputs = json.loads(inputs_path.read_text())
sizes = json.loads(sizes_path.read_text())
configuration = inputs["configuration"]
measurements = sizes["measurements"]
lines = [
    f"Nuxie Android runtime {metadata['artifactVersion']}",
    "",
    f"Source: `{metadata['buildSourceRevision']}`",
    f"C ABI runtime: `{metadata['runtimeIdentity']}`",
    "",
    "Pinned release cut:",
    "",
    f"- Rust {configuration['rustToolchain']}",
    f"- cargo-ndk {configuration['cargoNdk'].removeprefix('cargo-ndk ')}",
    f"- Android NDK {configuration['androidNdk']}",
    f"- Android API {configuration['androidApiLevel']}",
    f"- ABIs: {', '.join(metadata['android']['abis'])}",
    f"- Features: {', '.join(metadata['android']['features'])}",
    "",
    f"Archive bytes: {measurements['archiveBytes']}",
    f"Expanded bytes: {measurements['expandedBytes']}",
    f"Archive SHA-256: `{metadata['artifact']['sha256']}`",
]
notes_path.write_text("\n".join(lines) + "\n")
PY

gh release create "${release_tag}" \
    "${archive}#NuxieRuntimeAndroid.zip" \
    "${metadata}#NuxieRuntimeAndroid.json" \
    "${build_inputs}#NuxieRuntimeAndroid-BUILD_INPUTS.json" \
    "${size_report}#NuxieRuntimeAndroid-SIZE_REPORT.json" \
    --repo nuxieai/nuxie-runtime \
    --draft \
    --verify-tag \
    --title "Nuxie Android runtime 0.3.7" \
    --notes-file "${notes}"

for asset in \
    NuxieRuntimeAndroid.zip \
    NuxieRuntimeAndroid.json \
    NuxieRuntimeAndroid-BUILD_INPUTS.json \
    NuxieRuntimeAndroid-SIZE_REPORT.json
do
    gh release download "${release_tag}" \
        --repo nuxieai/nuxie-runtime \
        --dir "${downloads}" \
        --pattern "${asset}"
    cmp "${artifact_root}/${asset}" "${downloads}/${asset}"
done

python3 "${script_dir}/android_runtime_contract.py" verify \
    --repo-root "${repo_root}" \
    --artifact-root "${downloads}" \
    --release-revision "${source_revision}"
gh release edit "${release_tag}" \
    --repo nuxieai/nuxie-runtime \
    --draft=false
echo "Published immutable ${release_tag} after download and independent verification"
