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
  `viewModel()` result is nil. Generator-time contexts use the retained parent
  slots as independent DataContext-presence evidence, and `rootViewModel()`
  projects the terminal context's possibly-nil main model instead of skipping
  nil nodes. Detached advancement queries runtime-owned `has_parents()`
  directly; the scripting parent graph, mirror rewrites, and rescans are
  deleted.

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
initial swap. C6 regressions also cover generator-time retention of a nil-main
context with a parent slot and a terminal nil root. The scripted golden corpus
proves the integrated result.

## Review closeout

The required standards and spec reviews found three actionable edges, all
fixed in `bf44e59d`:

- C6 production construction had derived DataContext presence only from its
  main model. It now also uses retained parent slots, with a regression through
  `instantiate_registered_script_with_context` rather than only a direct
  userdata fixture.
- `rootViewModel()` had skipped nil parent nodes. It now selects the terminal
  context slot exactly, including a nil main model.
- String setters notified Core observers before their concrete generated
  changed callback, and two unused list-rescan helpers survived the registry
  demolition. Setter order now matches generated C++, and those facades are
  deleted.

Both review lanes were rerun through the affected focused tests and the entire
required gate set below.

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

## Commits

- `4ace1ac1` — `Port runtime-owned Lua parent edges`
- `e29adba7` — `Port Core property observer pushes`
- `e68b11f5` — `Close RB-1 observer remainder`
- `76ec5680` — `Record P3-b closeout evidence`
- `bf44e59d` — `Address P3-b closeout review`

The sandbox temporarily rejected creation of the shared `.git/index.lock`;
later retries succeeded, so no mapped diff remains.

No file was written outside this worktree and no `/tmp` worktree was used.

## Required gates

- `cargo test -p nuxie-runtime` — green: 931/931 library tests plus all
  integration and doc tests.
- `cargo test -p nuxie --features scripting` — green: 247/247 library tests
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

The supplemental, non-required `make port-manifest-check` still has four
pre-existing test failures around the already-landed command-queue/server rows;
the P3-b production rows themselves parse and are limited to Core and Lua
DataContext. No required P3-b gate depends on that stale generator expectation.
