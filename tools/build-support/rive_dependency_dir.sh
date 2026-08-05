#!/bin/bash
# Resolve a pinned rive-runtime C++ dependency to the exact revision the pinned
# librive was built from.
#
# Shell counterpart to the `dependency_dir` helper in
# tools/cpp-probe/build/premake5.lua, for tools that drive the compiler directly
# instead of going through premake. Keep the two in sync.
#
# Upstream fetches dependencies through build/dependency.lua's
# `dependency.github(project, tag)`, which clones to dependencies/<project>_<tag>
# with '/' replaced by '_'. Read the tag out of the runtime's own premake file
# rather than globbing that directory or hardcoding it: several projects have
# more than one tag checked out side by side -- rive-app_yoga_rive_changes_v2_0_1_2
# next to ..._v2_0_1_2_grid, three luigi-rosso_luau_* revisions -- so a glob
# picks whichever sorts first and a hardcoded path silently goes stale on the
# next runtime roll. Either way the tool compiles against different headers than
# the archive it links, which is an ABI/behavior mismatch hidden inside the thing
# that decides what "correct" means.
#
# Usage: rive_dependency_dir <rive_runtime> <project> <premake_file>
#   rive_dependency_dir "$rive_runtime" rive-app/yoga dependencies/premake5_yoga_v2.lua
# Prints the absolute dependency directory on stdout; exits non-zero with a
# diagnostic on stderr if the revision cannot be resolved or is not checked out.
# There is deliberately no fallback: a stale path is worse than a hard failure.
rive_dependency_dir() {
    local rive_runtime="$1"
    local project="$2"
    local premake_file="$3"
    local premake_path="$rive_runtime/$premake_file"

    if [[ ! -f "$premake_path" ]]; then
        echo "rive_dependency_dir: $premake_path does not exist;" \
            "cannot determine the $project revision the pinned librive was built with" >&2
        return 1
    fi

    # Escape BRE metacharacters in the project name ('.' in tag-bearing names,
    # and the '/' that also separates owner from repo).
    local project_re
    project_re="$(printf '%s' "$project" | sed 's/[][\.*^$\\]/\\&/g')"

    local tag
    tag="$(sed -n \
        "s#.*dependency\.github([[:space:]]*'${project_re}'[[:space:]]*,[[:space:]]*'\([^']*\)'.*#\1#p" \
        "$premake_path" | head -n 1)"

    if [[ -z "$tag" ]]; then
        echo "rive_dependency_dir: no dependency.github('$project', ...) in $premake_path;" \
            "cannot determine the revision the pinned librive was built with" >&2
        return 1
    fi

    local dir="$rive_runtime/dependencies/${project//\//_}_${tag}"
    if [[ ! -d "$dir" ]]; then
        echo "rive_dependency_dir: the pinned runtime builds $project at $tag," \
            "but $dir does not exist. Build librive first so the dependency is fetched;" \
            "another ${project//\//_}_* directory holds a different revision and is not a substitute." >&2
        return 1
    fi

    printf '%s\n' "$dir"
}
