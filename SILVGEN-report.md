Implemented the advanced-pin corpus update. No runtime code was changed.

Classifications:

- `bidirectional_stateful_property`: `diverges` — frame 3, op 180, transform `tx`: 150 vs 100.
- `paused_nested_artboard_opacity`: `diverges` — frame 1, op 103: `rewind` vs `drawPath`.
- `layout_text_match`: `diverges` — frame 0, op 61: `save` vs `frame`.

The generator now ignores commented-out C++ call sites, handles six new dynamic grid-stack baselines, and generates 252 entries. Evidence is in [SILVGEN-report.md](/Users/levi/dev/worktrees/nuxie-mr-c12/SILVGEN-report.md).

Passed:

- `make silver-corpus-test` — 19 Python tests plus Rust tests
- `make silver-corpus-manifest-check`
- `cargo test -p nuxie-runtime` — 943 library tests plus integration targets
- `git diff --check`

I could not create the commit because Git metadata is read-only in this environment (`index.lock: Operation not permitted`). `tools/land.sh` remains untouched and unstaged. To finish:

```sh
git add .gitignore SILVGEN-report.md silver-corpus.toml \
  tools/silver-corpus/README.md \
  tools/silver-corpus/generate_manifest.py \
  tools/silver-corpus/src/lib.rs \
  tools/silver-corpus/test_generate_manifest.py \
  tools/silver-corpus/tests/runtime_frame_loop_backfill_bc.rs

git commit -m '[silver] Regenerate corpus at the advanced pin: dedup S4 call sites, classify new fixtures'
```