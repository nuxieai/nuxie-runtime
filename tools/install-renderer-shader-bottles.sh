#!/bin/bash
set -euo pipefail

# Install the pinned renderer shader toolchain (spirv-headers, spirv-tools,
# glslang) from Homebrew's bottle registry by content digest.
#
# CI used to install these by copying pinned homebrew-core formula revisions
# into a synthetic tap, but Homebrew republished these bottles in February
# 2026: the blobs referenced by the pinned formulas were garbage collected,
# so `brew install` downloads the bottle manifest, finds no matching blob,
# and dies mid-pour ("<keg> is not a directory"). Fetching the current blobs
# by sha256 keeps the toolchain bit-exact without depending on formula
# resolution, tap trust, or other Homebrew install machinery.
# generate-renderer-shaders.sh still enforces the exact tool version strings.
#
# The digests are the arm64_sonoma bottles (macos-14 CI runners); the
# spirv-headers bottle is platform-independent ("all"). To refresh a digest:
#   curl -fsSL -H "Authorization: Bearer $(curl -fsSL 'https://ghcr.io/token?service=ghcr.io&scope=repository:homebrew/core/<name>:pull' | jq -r .token)" \
#       -H "Accept: application/vnd.oci.image.index.v1+json" \
#       https://ghcr.io/v2/homebrew/core/<name>/manifests/<version> \
#       | jq -r '.manifests[].annotations | .["org.opencontainers.image.ref.name"] + " " + .["sh.brew.bottle.digest"]'

if [[ "$(uname -sm)" != "Darwin arm64" ]]; then
    echo "pinned bottle digests are for arm64 macOS; refusing to install on $(uname -sm)" >&2
    exit 1
fi

prefix="${RENDERER_SHADER_TOOLS_PREFIX:-$(brew --prefix)}"
cellar="$prefix/Cellar"
work="$(mktemp -d "${TMPDIR:-/tmp}/renderer-shader-bottles.XXXXXX")"
trap 'rm -rf "$work"' EXIT

substitute_placeholders() {
    local value="$1"
    value="${value//@@HOMEBREW_PREFIX@@/$prefix}"
    value="${value//@@HOMEBREW_CELLAR@@/$cellar}"
    printf '%s' "$value"
}

# Bottle relocation, normally done by `brew install` at pour time: bottles
# built as `cellar :any` carry @@HOMEBREW_PREFIX@@ placeholders in text
# files and Mach-O load commands.
relocate_keg() {
    local keg="$1"
    local file id dep

    while IFS= read -r file; do
        LC_ALL=C sed -i '' \
            -e "s|@@HOMEBREW_PREFIX@@|$prefix|g" \
            -e "s|@@HOMEBREW_CELLAR@@|$cellar|g" \
            "$file"
    done < <(grep -rlI '@@HOMEBREW_' "$keg" || true)

    while IFS= read -r file; do
        file -b "$file" | grep -q 'Mach-O' || continue
        local changed=0
        id="$(otool -D "$file" 2>/dev/null | sed -n '2p' || true)"
        if [[ "$id" == *@@HOMEBREW_* ]]; then
            install_name_tool -id "$(substitute_placeholders "$id")" "$file"
            changed=1
        fi
        while IFS= read -r dep; do
            install_name_tool -change "$dep" "$(substitute_placeholders "$dep")" "$file"
            changed=1
        done < <(otool -L "$file" | awk 'NR > 1 { print $1 }' | grep '@@HOMEBREW_' || true)
        if [[ "$changed" == 1 ]]; then
            codesign --force --sign - "$file"
        fi
    done < <(find "$keg" -type f)
}

install_bottle() {
    local name="$1" version="$2" digest="$3"
    local token blob keg bin

    token="$(curl -fsSL "https://ghcr.io/token?service=ghcr.io&scope=repository:homebrew/core/${name}:pull" \
        | /usr/bin/python3 -c 'import sys, json; print(json.load(sys.stdin)["token"])')"
    blob="$work/$name.bottle.tar.gz"
    curl -fsSL -H "Authorization: Bearer $token" -o "$blob" \
        "https://ghcr.io/v2/homebrew/core/${name}/blobs/sha256:${digest}"
    echo "$digest  $blob" | shasum -a 256 --check --status

    keg="$cellar/$name/$version"
    rm -rf "${cellar:?}/${name:?}"
    mkdir -p "$cellar"
    tar -xzf "$blob" -C "$cellar"
    if [[ ! -d "$keg" ]]; then
        echo "bottle for $name did not contain keg $name/$version" >&2
        exit 1
    fi

    relocate_keg "$keg"

    mkdir -p "$prefix/opt" "$prefix/bin"
    ln -sfn "$keg" "$prefix/opt/$name"
    if [[ -d "$keg/bin" ]]; then
        for bin in "$keg/bin/"*; do
            ln -sf "$bin" "$prefix/bin/$(basename "$bin")"
        done
    fi
    echo "installed $name $version -> $keg"
}

install_bottle spirv-headers 1.4.341.0 \
    efb9f1b78eb2c873093671ddcace25f80d411a5dc4eb53c83f96e8a433bfc6ec
install_bottle spirv-tools 1.4.341.0 \
    912dd89569602634bb84ddc2ce48102aafa8567499d30843f2f58b4e3760c86c
install_bottle glslang 16.2.0 \
    a2fab91a8da94e119a37d698d28bb924c7fb30c03413f77bd371a8eb6fe9b952

"$prefix/bin/glslangValidator" --version | head -n 1
"$prefix/bin/spirv-opt" --version
