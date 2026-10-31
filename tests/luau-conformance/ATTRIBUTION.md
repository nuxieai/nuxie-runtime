# Upstream Luau conformance corpus

`upstream/` holds the 55 scripts of `tests/conformance/` from the Luau
repository, copied byte for byte. They are the corpus that
`tools/luau-conformance` runs against the vendored luaur VM, scored by
`scope.txt` (`make luau-conformance`).

| Field | Value |
|---|---|
| Repository | https://github.com/luau-lang/luau |
| Commit | `3fc82b1071ab387531175869afc4fb528464afa4` |
| Tag | `0.734` |
| Path | `tests/conformance/*.luau` |
| License | MIT (`upstream/LICENSE.txt`, Roblox Corporation); several scripts derive from the Lua 5.1 test suite (`upstream/lua_LICENSE.txt`, Lua.org, PUC-Rio) |
| Vendored | 2026-10-31 |

The files are unmodified. `literals.luau`, `pm.luau` and `sort.luau` are
Latin-1 (ISO-8859-1), not UTF-8; keep their bytes as they are.

Copy command used (from a checkout of the commit above):

```sh
cp -p "$LUAU/tests/conformance/"*.luau tests/luau-conformance/upstream/
cp -p "$LUAU/LICENSE.txt" "$LUAU/lua_LICENSE.txt" tests/luau-conformance/upstream/
```

To re-vendor at a pinned commit (a pin roll), run
`tools/luau-conformance/fetch-upstream.sh <commit>`; it does the same copy
from a depth-1 fetch. Then update the commit and tag here and in the
script, run `make luau-conformance`, and classify any new script in
`scope.txt`.
