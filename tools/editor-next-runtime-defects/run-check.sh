#!/bin/sh
set -eu

script_dir=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
repo_root=$(CDPATH= cd -- "$script_dir/../.." && pwd)
rive_runtime_dir=${RIVE_RUNTIME_DIR:-/Users/levi/dev/oss/rive-runtime}
expected_ref=d788e8ec6e8b598526607d6a1e8818e8b637b60c
atlas="$repo_root/docs/editor-next-runtime-defect-atlas.toml"
editor_repo=${EDITOR_NEXT_REPO_DIR:-/Users/levi/.codex/worktrees/7189/nuxie-dev/worktrees/editor-next-cutover-assembly}

if [ -n "${EDITOR_NEXT_PLAN_DIR:-}" ]; then
    source_root=$EDITOR_NEXT_PLAN_DIR
    newest_source_root=${EDITOR_NEXT_NEWEST_PLAN_DIR:-"$source_root"}
else
    refs=$(python3 - "$atlas" <<'PY'
import pathlib
import sys
import tomllib

atlas = tomllib.loads(pathlib.Path(sys.argv[1]).read_text())
print(atlas["source_snapshot_ref"])
print(atlas["inbox"]["newest_available_editor_ref"])
PY
)
    source_ref=$(printf '%s\n' "$refs" | sed -n '1p')
    newest_source_ref=$(printf '%s\n' "$refs" | sed -n '2p')
    snapshot_parent=$(mktemp -d "${TMPDIR:-/tmp}/nuxie-editor-runtime-source.XXXXXX")
    source_worktree="$snapshot_parent/editor"
    newest_source_worktree=
    cleanup_source_worktree() {
        if [ -n "$newest_source_worktree" ]; then
            git -C "$editor_repo" worktree remove --force "$newest_source_worktree" >/dev/null 2>&1 || true
        fi
        git -C "$editor_repo" worktree remove --force "$source_worktree" >/dev/null 2>&1 || true
        rmdir "$snapshot_parent" >/dev/null 2>&1 || true
    }
    trap cleanup_source_worktree EXIT HUP INT TERM
    git -C "$editor_repo" worktree add --quiet --detach "$source_worktree" "$source_ref"
    source_root="$source_worktree/plans"
    if [ "$newest_source_ref" = "$source_ref" ]; then
        newest_source_root=$source_root
    else
        newest_source_worktree="$snapshot_parent/newest-editor"
        git -C "$editor_repo" worktree add --quiet --detach "$newest_source_worktree" "$newest_source_ref"
        newest_source_root="$newest_source_worktree/plans"
    fi
fi

python3 "$script_dir/test_check.py"
probe=$("$script_dir/cpp_probe/build.sh")
python3 "$script_dir/check.py" \
    --repo-root "$repo_root" \
    --atlas "$atlas" \
    --corrections "$repo_root/docs/editor-next-runtime-defect-corrections.toml" \
    --fixtures "$script_dir/fixtures.toml" \
    --source-root "$source_root" \
    --newest-source-root "$newest_source_root" \
    --editor-repo-dir "$editor_repo" \
    --rive-runtime-dir "$rive_runtime_dir" \
    --cpp-probe "$probe" \
    --expected-upstream-ref "$expected_ref"

echo "editor-next-runtime-defect-probe: registry and provenance green"
