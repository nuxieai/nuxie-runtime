# P3-b RB-1 remainder report

## Outcome

P3-b ports the `4ac7b32798da0482e441ef09304dc3b480ed3ee5`
pin-advance remainder as one runtime-state spine:

- Pinned `src/core.cpp` property observers now have a direct Rust owner in
  `crates/nuxie-runtime/src/core.rs`. The arena adaptation retains observers by
  exact `(local_id, property_key)`, keeps the empty-observer fast path, and
  pushes dirt only to the subscribed authored DataBind occurrences.
- Artboard target setters notify the retained observer owner synchronously.
  Nested-host bindings also retain authored-index/path queue membership, so
  initial and subsequent source-to-target work drains sparsely instead of
  relying on a whole-context pass.
- The Artboard mutation/processed epoch pair and its dirty-mark facade are
  deleted. The compatibility `stateful_nested_view_model_contexts_dirty` flag
  is not a scheduler; it is consumed only inside a pass opened by retained
  queue dirt or one of the named conservative-poll families.
- Lua DataContext chains now retain `Option<ScriptViewModel>` per context node.
  `parent()` therefore preserves a real parent DataContext whose independent
  `viewModel()` result is nil. Detached advancement queries runtime-owned
  `has_parents()` directly; the scripting parent graph, mirror rewrites, and
  rescans are deleted.

The conservative polling residue is limited to converter, layout-computed,
Solo, and shape-length/numeric families whose pinned dependency edges are not
yet modeled. It is explicit crate-boundary residue, not the general RB-1
compensation scheduler.

## Oracle localization

The first observer cut made `scripted_data_context` exact but exposed a late
`stateful_nested` mismatch. A fresh runner showed that a compatibility rescan
flag was incorrectly being treated as retained queue dirt after layout
dimension changes. Excluding that flag from `has_artboard_data_bind_queue_work`
restored exact C++ scheduling. The full runtime suite then exposed the dual
edge: a null nested-artboard source had no initial retained enrollment. Adding
nested-host occurrences to `RuntimeArtboardDataBindTargetQueues` fixed that
producer without re-enabling the compensation flag.

Direct regressions cover exact-property fanout/removal, a clean owned-context
queue gate even when the compatibility flag is set, and the null nested-artboard
initial swap. The scripted golden corpus proves the integrated result.

## Correspondence and residue

- `file-correspondence-manifest.toml` promotes only B6-0146 `src/core.cpp` and
  B6-0264 `src/lua/lua_data_context.cpp` to faithful/orchestrator-verified.
  The generated scorecard is faithful 438, pending 14,
  divergent-by-decision 4; the pending floor only tightened.
- `port-manifest.toml` names the direct Core owner/integration seam and moves
  `lua_data_context.cpp` from partial to ported.
- `test-correspondence-manifest.toml` moves
  `scripting_detached_viewmodel_advance_test.cpp` only from pending to partial.
  Two non-null lifecycle cases are named explicitly; the upstream null-wrapper
  case has no constructible safe-Rust counterpart and remains uncovered.
- `docs/runtime-frame-loop-ownership.toml` has no `src/core.cpp` or
  `src/lua/lua_data_context.cpp` source set or member row: it remains the frozen
  FL-E8 frame-loop ledger, so there was no stale ownership path to move. The C6
  public API inventory receipt in `docs/runtime-frame-loop-gaps.toml` was
  refreshed for the intentional `Vec<Option<ScriptViewModel>>` signature.
- Owner comments live with `RuntimeCorePropertyObservers`,
  `RuntimeOwnedDataContext::main_context_slots`, `ScriptViewModel::has_parents`,
  and the detached root registry. No deleted module or copied-parent attribution
  residue remains.
- Rust attribution is complete. Frame-loop scatter is 153/155. The Core row's
  three modules and the DataContext row's two crates carry explicit arena/crate
  boundary justifications; no scatter ratchet was relaxed.
- The umbrella `lua_properties.cpp` and `rive_lua_libs.cpp` rows remain pending
  for their unrelated whole-file residue.

## Commits and sandbox handoff

Commits completed before the shared Git metadata became read-only:

- `4ace1ac1` — `Port runtime-owned Lua parent edges`
- `e29adba7` — `Port Core property observer pushes`

The final queue-gate/nested-host correction, ledgers, scorecard, status/map,
and this report remain as a clean worktree diff because the sandbox rejects
creation of the shared `.git/index.lock` (and rejects an alternate index when
Git tries to write objects under the shared worktree metadata). The landing
worker should commit the remaining worktree diff as one P3-b closeout commit;
no file was written outside this worktree and no `/tmp` worktree was used.

## Required gates

- `cargo test -p nuxie-runtime` — green: 931/931 library tests plus all
  integration and doc tests.
- `cargo test -p nuxie --features scripting` — green: 246/246 library tests
  (one fixture generator ignored) plus all integration and doc tests.
- `make runtime-frame-loop-port-check` — green: 112 checker tests; test ledger
  157 files / 1,404 TEST_CASEs; frame-loop files 353 faithful + 1 approved
  divergence; scatter 153/155.
- `make rust-attribution-check` — green: 10/10 checker tests and every in-scope
  Rust source classified.
- `make scripted-golden-compare` — green: 353 entries, 324 exact, 670 exact
  segments, 669 side-channel segments, 29 not-yet, zero divergences and zero
  unsupported features.
- `make parity-scorecard` — green: 26/26 generator tests; snapshot regenerated.
- `git diff --check` — green.

