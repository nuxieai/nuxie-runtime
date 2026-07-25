#!/bin/sh
set -eu

script_dir=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
repo_root=$(CDPATH= cd -- "$script_dir/../.." && pwd)
source_root=${EDITOR_NEXT_PLAN_DIR:-/Users/levi/.codex/worktrees/7189/nuxie-dev/worktrees/editor-next-cutover-assembly/plans}
rive_runtime_dir=${RIVE_RUNTIME_DIR:-/Users/levi/dev/oss/rive-runtime}
expected_ref=d788e8ec6e8b598526607d6a1e8818e8b637b60c

python3 "$script_dir/test_check.py"
probe=$("$script_dir/cpp_probe/build.sh")
python3 "$script_dir/check.py" \
    --repo-root "$repo_root" \
    --atlas "$repo_root/docs/editor-next-runtime-defect-atlas.toml" \
    --corrections "$repo_root/docs/editor-next-runtime-defect-corrections.toml" \
    --fixtures "$script_dir/fixtures.toml" \
    --source-root "$source_root" \
    --rive-runtime-dir "$rive_runtime_dir" \
    --cpp-probe "$probe" \
    --expected-upstream-ref "$expected_ref"

echo "editor-next-runtime-defect-probe: registry and provenance green"
