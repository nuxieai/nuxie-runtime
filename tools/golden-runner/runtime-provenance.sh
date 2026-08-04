#!/bin/bash
set -euo pipefail

expected_runtime_revision="4ac7b32798da0482e441ef09304dc3b480ed3ee5"
schema="nuxie-golden-librive-provenance-v3"

# Registered local oracle patches applied on top of the pinned revision when
# building librive. The pinned checkout itself must stay pristine: build.sh
# materializes `git archive` of the pin into an isolated tree, applies these,
# and compiles there. The stamp binds the archive to the exact patch set so a
# patch edit (or removal) invalidates previously built archives.
librive_patch_dir="$(cd "$(dirname "$0")/.." && pwd)/rive-runtime-patches"

librive_patches() {
    local patch
    for patch in "$librive_patch_dir"/librive-*.patch; do
        [[ -e "$patch" ]] || continue
        printf '%s\n' "$patch"
    done | LC_ALL=C sort
}

patches_digest() {
    local patch digest out=""
    while IFS= read -r patch; do
        [[ -n "$patch" ]] || continue
        digest="$(sha256_file "$patch")"
        out+="${out:+,}$(basename "$patch"):$digest"
    done < <(librive_patches)
    if [[ -z "$out" ]]; then
        out="none"
    fi
    printf '%s\n' "$out"
}

usage() {
    echo "usage: $0 source <runtime-dir>" >&2
    echo "       $0 patches" >&2
    echo "       $0 materialize <runtime-dir> <dest-dir>" >&2
    echo "       $0 write|verify <runtime-dir> <archive> <rive.make> <stamp> <debug|release> <ordinary|scripted|audio>" >&2
    exit 2
}

sha256_file() {
    if command -v sha256sum >/dev/null 2>&1; then
        sha256sum "$1" | awk '{print $1}'
    else
        shasum -a 256 "$1" | awk '{print $1}'
    fi
}

compiler_path() {
    command -v "${CXX:-clang++}"
}

compiler_version() {
    "$(compiler_path)" --version | sed -n '1p'
}

normalize_defines() {
    local makefile="$1"
    local defines_line
    defines_line="$(grep -m1 '^DEFINES +=' "$makefile" || true)"
    if [[ -z "$defines_line" ]]; then
        echo "golden runner provenance: missing DEFINES line in $makefile" >&2
        return 1
    fi
    printf '%s\n' "$defines_line" \
        | sed 's/^DEFINES +=[[:space:]]*//' \
        | tr ' ' '\n' \
        | sed -n 's/^-D//p' \
        | LC_ALL=C sort \
        | paste -sd, -
}

expected_defines() {
    local config="$1"
    local mode="$2"
    local defines=(
        "RIVE_MACOSX"
        "WITH_RIVE_LAYOUT"
        "WITH_RIVE_TEXT"
        "YOGA_EXPORT="
        "_RIVE_INTERNAL_"
    )
    if [[ "$config" == "debug" ]]; then
        defines+=("DEBUG")
    else
        defines+=("NDEBUG" "RELEASE")
    fi
    if [[ "$mode" == "scripted" ]]; then
        defines+=(
            "EXTERNAL_RIVE_AUDIO_ENGINE"
            "HYDRO_SIGN_VERIFY_ONLY=1"
            "MA_NO_DEVICE_IO"
            "MA_NO_RESOURCE_MANAGER"
            "RIVE_DECODERS"
            "WITH_RIVE_AUDIO"
            "WITH_RIVE_SCRIPTING"
        )
    elif [[ "$mode" == "audio" ]]; then
        defines+=(
            "EXTERNAL_RIVE_AUDIO_ENGINE"
            "MA_NO_DEVICE_IO"
            "MA_NO_RESOURCE_MANAGER"
            "WITH_RIVE_AUDIO"
        )
    fi
    printf '%s\n' "${defines[@]}" | LC_ALL=C sort | paste -sd, -
}

# Produce the librive build source tree. With no registered patches this is
# the pinned checkout itself. With patches, an isolated `git archive` of the
# pin is extracted into <dest-dir>, patches are applied there, and <dest-dir>
# is the build source — the shared checkout is never written. Prints the
# directory to build from on stdout.
materialize_source() {
    local runtime_dir="$1"
    local dest="$2"
    validate_source "$runtime_dir"
    local patches
    patches="$(librive_patches)"
    if [[ -z "$patches" ]]; then
        printf '%s\n' "$runtime_dir"
        return 0
    fi
    rm -rf "$dest"
    mkdir -p "$dest"
    git -C "$runtime_dir" archive HEAD | tar -x -C "$dest"
    # The destination usually lives under the consuming repo's (gitignored)
    # target/ tree. `git apply` run there would resolve the ENCLOSING repo and
    # silently skip every path (exit 0, no changes) — so pin the destination
    # as its own throwaway repo, apply against it, and hard-verify each patch
    # actually landed before reporting the tree as patched.
    git -C "$dest" init --quiet
    local patch
    while IFS= read -r patch; do
        [[ -n "$patch" ]] || continue
        echo "applying oracle patch: $(basename "$patch")" >&2
        git -C "$dest" apply "$patch"
        if ! git -C "$dest" apply --reverse --check "$patch"; then
            echo "oracle patch did not take effect: $patch" >&2
            return 1
        fi
    done <<<"$patches"
    rm -rf "$dest/.git"
    printf '%s\n' "$dest"
}

read_stamp_value() {
    local stamp="$1"
    local key="$2"
    local count
    count="$(grep -c "^${key}=" "$stamp" || true)"
    if [[ "$count" != "1" ]]; then
        echo "golden runner provenance: expected exactly one $key field in $stamp, found $count" >&2
        return 1
    fi
    sed -n "s/^${key}=//p" "$stamp"
}

validate_source() {
    local runtime_dir="$1"
    local actual_revision
    actual_revision="$(git -C "$runtime_dir" rev-parse HEAD 2>/dev/null || true)"
    if [[ "$actual_revision" != "$expected_runtime_revision" ]]; then
        echo "golden runner provenance: expected runtime $expected_runtime_revision, found ${actual_revision:-not-a-git-checkout}" >&2
        return 1
    fi
    if ! git -C "$runtime_dir" diff --quiet --ignore-submodules -- ||
        ! git -C "$runtime_dir" diff --cached --quiet --ignore-submodules --; then
        echo "golden runner provenance: tracked runtime sources are dirty at $runtime_dir" >&2
        return 1
    fi
}

validate_inputs() {
    local runtime_dir="$1"
    local archive="$2"
    local makefile="$3"
    local config="$4"
    local mode="$5"

    if [[ "$config" != "debug" && "$config" != "release" ]]; then
        usage
    fi
    if [[ "$mode" != "ordinary" && "$mode" != "scripted" && "$mode" != "audio" ]]; then
        usage
    fi
    if [[ ! -f "$archive" ]]; then
        echo "golden runner provenance: missing librive archive $archive" >&2
        return 1
    fi
    if [[ ! -f "$makefile" ]]; then
        echo "golden runner provenance: missing generated makefile $makefile" >&2
        return 1
    fi
    validate_source "$runtime_dir"

    local actual_defines expected
    actual_defines="$(normalize_defines "$makefile")"
    expected="$(expected_defines "$config" "$mode")"
    if [[ "$actual_defines" != "$expected" ]]; then
        echo "golden runner provenance: librive feature flags do not match" >&2
        echo "  expected: $expected" >&2
        echo "  actual:   $actual_defines" >&2
        return 1
    fi
}

write_stamp() {
    local runtime_dir="$1"
    local archive="$2"
    local makefile="$3"
    local stamp="$4"
    local config="$5"
    local mode="$6"
    validate_inputs "$runtime_dir" "$archive" "$makefile" "$config" "$mode"

    local temporary
    temporary="$(mktemp "${stamp}.tmp.XXXXXX")"
    {
        echo "schema=$schema"
        echo "runtime_revision=$expected_runtime_revision"
        echo "config=$config"
        echo "mode=$mode"
        echo "defines=$(normalize_defines "$makefile")"
        echo "patches=$(patches_digest)"
        echo "compiler_path=$(compiler_path)"
        echo "compiler_version=$(compiler_version)"
        echo "archive_sha256=$(sha256_file "$archive")"
    } >"$temporary"
    mv "$temporary" "$stamp"
}

verify_stamp() {
    local runtime_dir="$1"
    local archive="$2"
    local makefile="$3"
    local stamp="$4"
    local config="$5"
    local mode="$6"
    validate_inputs "$runtime_dir" "$archive" "$makefile" "$config" "$mode"
    if [[ ! -f "$stamp" ]]; then
        echo "golden runner provenance: missing stamp $stamp" >&2
        return 1
    fi

    local field expected
    for field in schema runtime_revision config mode defines patches compiler_path compiler_version archive_sha256; do
        case "$field" in
            schema) expected="$schema" ;;
            runtime_revision) expected="$expected_runtime_revision" ;;
            config) expected="$config" ;;
            mode) expected="$mode" ;;
            defines) expected="$(normalize_defines "$makefile")" ;;
            patches) expected="$(patches_digest)" ;;
            compiler_path) expected="$(compiler_path)" ;;
            compiler_version) expected="$(compiler_version)" ;;
            archive_sha256) expected="$(sha256_file "$archive")" ;;
        esac
        local actual
        actual="$(read_stamp_value "$stamp" "$field")"
        if [[ "$actual" != "$expected" ]]; then
            echo "golden runner provenance: $field mismatch in $stamp" >&2
            echo "  expected: $expected" >&2
            echo "  actual:   $actual" >&2
            return 1
        fi
    done
}

[[ "$#" -ge "1" ]] || usage
action="$1"
shift
case "$action" in
    source)
        [[ "$#" == "1" ]] || usage
        validate_source "$@"
        ;;
    patches)
        [[ "$#" == "0" ]] || usage
        librive_patches
        ;;
    materialize)
        [[ "$#" == "2" ]] || usage
        materialize_source "$@"
        ;;
    write)
        [[ "$#" == "6" ]] || usage
        write_stamp "$@"
        ;;
    verify)
        [[ "$#" == "6" ]] || usage
        verify_stamp "$@"
        ;;
    *) usage ;;
esac
