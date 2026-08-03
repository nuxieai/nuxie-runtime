#!/bin/bash
# Strict landing gate: runs the full battery fail-fast, then pushes and
# opens/merges the PR. No output-parsing, no chains — bare exit codes only.
# Usage: tools/land.sh <branch> <pr-title-file> [extra-gate ...]
set -euo pipefail
branch="$1"; body="$2"; shift 2
[[ -f "$body" ]] || { echo "land.sh: body file $body missing" >&2; exit 1; }
# merge-guard: never run the battery on a stale or dirty tree
if ! git diff --quiet || ! git diff --cached --quiet; then
    echo "land.sh: dirty tree — commit or stash before landing" >&2; exit 1
fi
git fetch origin main
git merge origin/main --no-edit || { echo "land.sh: merge with origin/main failed" >&2; exit 1; }
# Invalidate any cargo artifacts whose sources changed without a fresh mtime
# (regenerated codegen racing a concurrent build) so every step below builds
# from the sources being landed, not a poisoned cache.
make rust-sources-fresh
make cpp-probe
cargo test -p nuxie-runtime
cargo test -p nuxie --features scripting
make runtime-frame-loop-port-check
make rust-attribution-check
make scripted-golden-compare
make silver-corpus-test
for extra in "$@"; do make "$extra"; done
git push -u origin "$branch"
gh pr create --base main --head "$branch" --title "$(head -1 "$body")" --body "$(tail -n +2 "$body")"
gh pr merge --merge
echo "LANDED: $branch"
