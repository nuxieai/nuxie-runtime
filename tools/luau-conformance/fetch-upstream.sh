#!/usr/bin/env bash
# Re-vendor the upstream Luau conformance corpus at a pinned commit.
#
#   tools/luau-conformance/fetch-upstream.sh [<commit>]
#
# Fetches luau-lang/luau at <commit> (default: the current pin below) with a
# depth-1 fetch into a throwaway clone and copies tests/conformance/*.luau plus
# the two license files byte for byte into tests/luau-conformance/upstream/.
# Set LUAU_SRC_TREE to an existing checkout to copy from it instead of
# fetching (no commit check is possible then; the caller vouches for it).
#
# After a pin roll: update the commit and tag in
# tests/luau-conformance/ATTRIBUTION.md and LUAU_CONFORMANCE_PIN below, run
# `make luau-conformance`, and classify every new or changed script in
# tests/luau-conformance/scope.txt (the tool refuses unscoped scripts).
set -euo pipefail

LUAU_CONFORMANCE_PIN=3fc82b1071ab387531175869afc4fb528464afa4 # tag 0.734
LUAU_REPO="${LUAU_REPO:-https://github.com/luau-lang/luau.git}"
commit="${1:-$LUAU_CONFORMANCE_PIN}"

root="$(cd "$(dirname "$0")/../.." && pwd)"
dest="${LUAU_CONFORMANCE_DEST:-$root/tests/luau-conformance/upstream}"

if [ -n "${LUAU_SRC_TREE:-}" ]; then
  src="$LUAU_SRC_TREE"
else
  scratch="$(mktemp -d)"
  trap 'rm -rf "$scratch"' EXIT
  git -C "$scratch" init -q
  git -C "$scratch" fetch -q --depth 1 "$LUAU_REPO" "$commit"
  git -C "$scratch" checkout -q FETCH_HEAD
  fetched="$(git -C "$scratch" rev-parse HEAD)"
  if [ "$fetched" != "$commit" ]; then
    echo "fetched $fetched, expected $commit (pass a full commit SHA)" >&2
    exit 1
  fi
  src="$scratch"
fi

if ! ls "$src"/tests/conformance/*.luau >/dev/null 2>&1; then
  echo "no tests/conformance/*.luau under $src" >&2
  exit 1
fi

mkdir -p "$dest"
rm -f "$dest"/*.luau
# cp -p copies bytes verbatim; three scripts (literals, pm, sort) are Latin-1.
cp -p "$src"/tests/conformance/*.luau "$dest"/
cp -p "$src"/LICENSE.txt "$src"/lua_LICENSE.txt "$dest"/
count="$(ls "$dest"/*.luau | wc -l | tr -d ' ')"
echo "vendored $count scripts from luau $commit into $dest"
