# Upstream Luau benchmark scripts

These files are copied unchanged from the Luau repository's `bench/`
directory. They are the timing workload for `tools/luau-bench`.

| Field | Value |
|---|---|
| Repository | https://github.com/luau-lang/luau |
| Commit | `3fc82b1071ab387531175869afc4fb528464afa4` |
| Tag | `0.734` |
| License | MIT, see `LICENSE.txt` in this folder (copied from the same commit) |

What was copied:

- `bench/bench_support.lua` to `bench_support.lua`
- `bench/tests/*.lua` (top level only) to `tests/`
- `bench/micro_tests/*.lua` to `micro_tests/`

Upstream's subfolders (`bench/tests/shootout`, `sunspider`, `integers`,
`vibemark67`, `zefbench`, `bench/gc`, `bench/other`) and its Python drivers
were not copied.

The scripts load `bench_support.lua` through `require`, which the Rive Luau
profile does not provide. `tests/luau-bench/merge.py` writes self-contained
copies into `target/luau-bench/scripts/` at run time, with the support module
inlined. The merged copies are build output and are never committed. That
script also holds the list of benches that are left out and why.

To update: copy the same set of files from a new upstream commit, update the
table above and the entry in `THIRD_PARTY_NOTICES.md`, then regenerate
`tests/luau-bench/baseline-luaur.json` on a quiet host.
