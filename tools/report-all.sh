#!/usr/bin/env bash
# Run every check in a group and report every failure.
#
# A gate that stops at its first failing step reports one failure and hides the
# rest. When a cheap precondition runs ahead of an expensive check, that is not
# a cosmetic loss: the expensive check never executes, so a distinct real
# failure underneath stays invisible until the cheap one is fixed. This runner
# exists so a group of independent checks reports its whole truth in one pass.
#
# Usage:
#   tools/report-all.sh <group label> [<check label> <shell command>]...
#
# Every check runs even if an earlier one fails. Output is streamed under a
# per-check banner, and the trailing summary names each failing check. Exit
# status is 1 if any check failed, 0 otherwise.
#
# Only use this for checks that are genuinely independent. A check that cannot
# produce a trustworthy verdict unless an earlier one passed belongs in the
# same command as its precondition, not in a sibling slot here.

set -uo pipefail

if [ "$#" -lt 3 ]; then
  echo "usage: $0 <group label> <check label> <command> [<check label> <command>]..." >&2
  exit 2
fi

group=$1
shift

if [ $(($# % 2)) -ne 0 ]; then
  echo "$0: checks must be given as label/command pairs" >&2
  exit 2
fi

failed=()
total=0

while [ "$#" -gt 0 ]; do
  label=$1
  command=$2
  shift 2
  total=$((total + 1))

  echo "== ${group}: ${label} =="
  if bash -c "$command"; then
    echo "-- ${group}: ${label} OK"
  else
    status=$?
    echo "-- ${group}: ${label} FAILED (exit ${status})" >&2
    failed+=("$label")
  fi
  echo
done

if [ "${#failed[@]}" -eq 0 ]; then
  echo "${group}: all ${total} checks passed"
  exit 0
fi

echo "${group}: ${#failed[@]} of ${total} checks failed" >&2
for label in "${failed[@]}"; do
  echo "  - ${label}" >&2
done
exit 1
