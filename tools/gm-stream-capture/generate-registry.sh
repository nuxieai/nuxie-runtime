#!/bin/bash
set -euo pipefail

script_dir="$(cd "$(dirname "$0")" && pwd)"
rive_runtime="${RIVE_RUNTIME_DIR:-/Users/levi/dev/oss/rive-runtime}"
registry="$script_dir/generated_registry.inc"
files="$script_dir/build/gm-files.txt"
gates="$script_dir/generated-gates.txt"

: >"$registry"
: >"$files"
: >"$gates"

for file in "$rive_runtime"/tests/gm/*.cpp; do
    base="$(basename "$file")"
    case "$base" in
        gm.cpp|gmmain.cpp|gmutils.cpp|ore_*.cpp|render_canvas*.cpp|gamma_texture.cpp|lots_of_squares.cpp)
            printf '%s\tdirect-render-context-or-harness\n' "$base" >>"$gates"
            continue
            ;;
    esac
    if rg -q 'renderContext\(|renderContextGLImpl\(|flushPLSContext\(|beginOreFrame\(|endOreFrame\(|makeOffscreenRenderTarget\(|beginFrame\(|endFrame\(' "$file"; then
        printf '%s\tdirect-render-context-or-harness\n' "$base" >>"$gates"
        continue
    fi
    printf '%s\n' "$file" >>"$files"
    perl -ne '
        if (/^\s*GMREGISTER\s*\(\s*([A-Za-z_][A-Za-z0-9_]*)/) { print "$1\n" }
        if (/^\s*DEF_SIMPLE_GM(?:_WITH_CLEAR_COLOR)?\s*\(\s*([A-Za-z_][A-Za-z0-9_]*)/) { print "$1\n" }
    ' "$file"
done | sort -u | sed 's/^/GM_ENTRY(/; s/$/)/' >"$registry"
