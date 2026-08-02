#!/bin/bash
# Strict landing gate: runs the full battery fail-fast, then pushes and
# opens/merges the PR. No output-parsing, no chains — bare exit codes only.
# Usage: tools/land.sh <branch> <pr-title-file> [extra-gate ...]
set -euo pipefail
branch="$1"; body="$2"; shift 2
[[ -f "$body" ]] || { echo "land.sh: body file $body missing" >&2; exit 1; }
make cpp-probe
cargo test -p nuxie-runtime
cargo test -p nuxie --features scripting
make runtime-frame-loop-port-check
make rust-attribution-check
make scripted-golden-compare
for extra in "$@"; do make "$extra"; done
git push -u origin "$branch"
gh pr create --base main --head "$branch" --title "$(head -1 "$body")" --body "$(tail -n +2 "$body")"
gh pr merge --merge
echo "LANDED: $branch"
