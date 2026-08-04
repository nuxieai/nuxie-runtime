#!/bin/bash
# Parallel landing gate with per-gate memoization.
# - All gates run concurrently as background jobs (cargo-based gates serialize
#   naturally on cargo's own target-dir lock; C++ and python gates overlap).
# - NOT fail-fast: every gate runs to completion and all failures report at
#   once, with per-gate logs under .land-cache/.
# - Memoized: a gate that passed for this exact tree (git tree hash) is
#   skipped on retry, so a single red gate never costs a full re-run.
# Usage: tools/land.sh <branch> <pr-title-file> [extra-gate ...]
set -uo pipefail
branch="$1"; body="$2"; shift 2
[[ -f "$body" ]] || { echo "land.sh: body file $body missing" >&2; exit 1; }
if ! git diff --quiet || ! git diff --cached --quiet; then
    echo "land.sh: dirty tree — commit or stash before landing" >&2; exit 1
fi
git fetch origin main
git merge origin/main --no-edit || { echo "land.sh: merge with origin/main failed" >&2; exit 1; }

tree=$(git rev-parse 'HEAD^{tree}')
cache=".land-cache/$tree"
mkdir -p "$cache"
echo "land.sh: gate cache $cache"

# Sources must be fresh before anything builds; cheap, always run.
make rust-sources-fresh || exit 1

gates=(cpp-probe runtime-frame-loop-port-check rust-attribution-check
       cargo-test-runtime cargo-test-scripting scripted-golden-compare
       silver-corpus-test)
for extra in "$@"; do gates+=("$extra"); done

run_gate() {
    local g="$1"
    if [[ -f "$cache/$g.ok" ]]; then
        echo "gate $g: cached pass"
        return 0
    fi
    local rc
    case "$g" in
        cargo-test-runtime)  cargo test -p nuxie-runtime > "$cache/$g.log" 2>&1; rc=$? ;;
        cargo-test-scripting) cargo test -p nuxie --features scripting > "$cache/$g.log" 2>&1; rc=$? ;;
        *)                   make "$g" > "$cache/$g.log" 2>&1; rc=$? ;;
    esac
    if [[ $rc -eq 0 ]]; then
        touch "$cache/$g.ok"
        echo "gate $g: PASS"
    else
        echo "gate $g: FAIL (log: $cache/$g.log)"
    fi
    return $rc
}

# scripted-golden-compare and silver need cpp-probe's oracle; run probe first
# if not cached, everything else fully parallel.
if [[ ! -f "$cache/cpp-probe.ok" ]]; then
    run_gate cpp-probe || { echo "land.sh: cpp-probe failed — aborting (oracle prerequisite)"; exit 1; }
fi

pids=(); names=()
for g in "${gates[@]}"; do
    [[ "$g" == "cpp-probe" ]] && continue
    run_gate "$g" &
    pids+=($!); names+=("$g")
done
failed=()
for i in "${!pids[@]}"; do
    wait "${pids[$i]}" || failed+=("${names[$i]}")
done
if [[ ${#failed[@]} -gt 0 ]]; then
    echo "land.sh: FAILED gates: ${failed[*]}" >&2
    for g in "${failed[@]}"; do echo "--- $g (last 15 lines) ---"; tail -15 "$cache/$g.log"; done >&2
    exit 1
fi

git push -u origin "$branch" || exit 1
# A prior land.sh run may have created the PR and then lost the merge race with
# main advancing; reuse the branch's open PR instead of aborting the retry.
if ! gh pr create --base main --head "$branch" --title "$(head -1 "$body")" --body "$(tail -n +2 "$body")"; then
    existing=$(gh pr list --head "$branch" --state open --json number --jq '.[0].number')
    if [[ -z "$existing" ]]; then
        echo "land.sh: gh pr create failed and no open PR exists for $branch" >&2; exit 1
    fi
    echo "land.sh: reusing existing open PR #$existing for $branch"
fi
gh pr merge --merge || exit 1
echo "LANDED: $branch"
