#!/bin/bash
set -euo pipefail

expected_runtime_revision="d788e8ec6e8b598526607d6a1e8818e8b637b60c"
schema="nuxie-golden-librive-provenance-v2"

usage() {
    echo "usage: $0 source <runtime-dir>" >&2
    echo "       $0 write|verify <runtime-dir> <archive> <rive.make> <stamp> <debug|release> <ordinary|scripted>" >&2
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
            "HYDRO_SIGN_VERIFY_ONLY=1"
            "RIVE_DECODERS"
            "WITH_RIVE_SCRIPTING"
        )
    fi
    printf '%s\n' "${defines[@]}" | LC_ALL=C sort | paste -sd, -
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
    if [[ "$mode" != "ordinary" && "$mode" != "scripted" ]]; then
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
    for field in schema runtime_revision config mode defines compiler_path compiler_version archive_sha256; do
        case "$field" in
            schema) expected="$schema" ;;
            runtime_revision) expected="$expected_runtime_revision" ;;
            config) expected="$config" ;;
            mode) expected="$mode" ;;
            defines) expected="$(normalize_defines "$makefile")" ;;
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
