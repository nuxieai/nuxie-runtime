# GPUCEIL final ledger commit map

The implementation commits exist on `levi/pend9-gpu-ceilings`, but this
sandbox cannot write the linked worktree index at
`/Users/levi/dev/nuxie-runtime/.git/worktrees/nuxie-mr-c15/index.lock`.
The orchestrator should reconstruct the final documentation commit with:

```text
Document Lua GPU ceiling candidate
```

## Exact files and hunks

- `GPUCEIL-report.md` (ignored by `*-report.md`; add with `git add -f`): final
  status, pinned anchors, authored/focused evidence, proposed D18 wording,
  four-place residue record, and the three-crate boundary exception.
- `GPUCEIL-map.md`: this sandbox reconstruction map.
- `docs/parity-gap-register.md`: narrow existing F7/F8 prose to distinguish
  the implemented GPU-prefixed candidate from Canvas 2D/`Image:view` residue
  and the still-unadopted wgpu adaptation. No D-row is added.
- `file-correspondence-manifest.toml`: update only `B6-0280`'s audit record and
  evidence note. Keep `status = "pending"` and
  `verification = "pending-verification"`.
- `port-manifest.toml`: narrow the existing partial `lua_gpu.cpp` note to the
  GPUCEIL candidate and retained mixed-file residue.
- `tools/port-manifest/port_manifest.py`: mirror that exact manifest note in
  the checker expectation.

There is no staging-manifest ownership in this lane. Exclude every other
working-tree path and do not add the proposed D-row or flip `B6-0280` during
reconstruction.

## Existing implementation commits

- `862b4f63` — retain Lua GPU submission pipeline snapshots and authored oracle.
- `649461f3` — execute indexed submissions and persistent resources on wgpu.
- `293ed573` — isolate concurrent mutable resource snapshots.
- `67ac6abb` — tie sidecars to occurrence lifetime, pin active cache entries,
  and reject hidden orphan passes.

## Focused gate results

- `cargo test -p nuxie-scripting authored_lua_gpu_ceiling_fixture -- --nocapture`:
  4 passed, 0 failed.
- Focused renderer unit regressions for indexed texture identity, concurrent
  snapshot isolation, cache pinning, and lifetime-token expiry: 1 passed each.
- Both adapter-aware wgpu tests compiled and passed their harness, but printed
  `GPU adapter unavailable`; no live execution/pixel evidence exists on this
  host, so the row is not promotable from this lane.
- `cargo check -p nuxie-render-api -p nuxie-scripting -p nuxie-renderer -p nuxie`:
  passed with existing warnings only.
- Pinned port-manifest check: 456/456 rows and Rust module paths verified.
- `git diff --check`: passed before adding this map.

No full battery, corpus/golden run, tolerance change, or verification/status
promotion was performed.
