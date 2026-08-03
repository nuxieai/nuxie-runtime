# P3A2 retained focus-tree integration report

## Result

The retained focus-tree lane was rebuilt on the exact current `origin/main`
commit `cc7effd0f8043abf0db925cea7ba4e6c19153537` as
`levi/p3a-focus-tree-v2`. The rebuilt branch contains only `[P3-a]` commits and
has a clean worktree.

The requested source lane ref `levi/p3a-focus-tree` was not present in the
provided worktree, so `git log --no-merges origin/main..levi/p3a-focus-tree`
could not resolve. The lane payload was recoverable as the worktree's committed
HEAD, `3e8564ae` (`[P3] Lane work (orchestrator commit)`). Its retained-focus
files were applied without committing to the exact main base. Lane scratch
files `MR2B-report.md` and `P3A-map.md` were dropped, and an unrelated
`simple_array` ledger-note hunk was restored to current-main state.

The repository metadata for the provided worktree is outside the writable
sandbox. The real rebuilt branch and commits therefore live in the writable
shared clone `/tmp/nuxie-p3a-v2.KlWl4s`; the final P3A-owned files are mirrored
into the provided worktree. `P3A2-map.md` records the exact branch recovery
command, including all intervening current-main files.

## Rebuilt commit series

From `cc7effd0`:

1. `678d1542 [P3-a] Rebuild retained focus tree on current main`
2. `55bd7181 [P3-a] Reconcile focus dirty cache with retained tree`
3. `6ae848c1 [P3-a] Preserve focus manager switch API`
4. `ab373e1f [P3-a] Promote retained focus correspondence`
5. `38b74b2b [P3-a] Reconcile retained focus gate ownership`
6. `84651c5d [P3-a] Fix retained focus domain sharing`
7. `7dd319a8 [P3-a] Attribute retained focus facades`
8. `6bb8d7b4 [P3-a] Pin retained focus coordinator docs`

## S4-7 reconciliation

Upstream commit `beb246e5` was studied in
`/Users/levi/dev/oss/rive-runtime`, including its manager/node changes and four
focus-test additions. No later focus-manager/node changes exist between that
commit and the candidate pin `4ac7b32798da0482e441ef09304dc3b480ed3ee5`, so
the dirty-cache contract at the pin is the combined upstream state.

The retained Rust architecture now implements that contract directly:

- `FocusManager` retains a lazy `Cell<Option<bool>>` cache for focusable
  content. A tree walk happens only on a cache miss.
- Topology mutations, `can_focus` changes, and Focusable-backing presence
  changes invalidate the cache. Unchanged retained updates preserve it.
- Artboard focus changes update stable retained nodes and lookup registries at
  mutation boundaries. Input dispatch and steady frame advance do not rebuild
  descriptors, lookup maps, or walk the full focus tree.
- External focus domains are moved into artboards rather than snapshot-cloned,
  preserving shared retained identity across component-list and nested-owner
  projections.
- Cycle rejection, owner-local node/target registries, exact keyboard
  capability publication, and the public focus-manager switch signature were
  preserved while resolving semantic conflicts with current main.

Green focused evidence:

- Five `focusable_content_cache_*` / unchanged-update manager tests cover
  `can_focus`, backing presence, add/remove, root migration, and steady-cache
  preservation.
- `cargo test -p nuxie-runtime --test focus_retained_tree` passes both public
  retained-node default/property and hierarchy/reparenting oracles.
- The complete runtime gate passes the domain-sharing, cycle, component-list,
  focus-state, and manager-switch regressions found during integration.
- Scripted golden exact focus cases include `bindable_focus_tree_swap`,
  `focus_collapsing`, `focus_test`, `focus_traversal`, `focusable_element`,
  `list_focus_order`, and `swappable_artboards_focus`.

## Ledger reconciliation

The `focus_data.cpp`, `focus_manager.cpp`, `focus_node.cpp`, and
`focusable.cpp` correspondence rows are `faithful` and
`orchestrator-verified`, each mapped to its single direct retained Rust owner
and citing green oracle evidence. The same four owners are represented in the
port manifest and runtime ownership atlas. The focus test row remains honestly
`partial` (85 upstream cases). P3A2 adds the four upstream dirty-cache cases to
the row's previously covered cases and does not claim the remaining cases.

The facade-only `focus.rs` and coordinator-only `input/mod.rs` are classified
as Rust additions instead of being attached to direct upstream ownership rows.
The ownership gate reports correspondence scatter `151/155`; no crate-boundary
exception was added.

## Required gates

All required gates are green:

| Gate | Result |
| --- | --- |
| `cargo test -p nuxie-runtime` | PASS: 929 library tests plus all integration and doc tests |
| `cargo test -p nuxie --features scripting` | PASS: all unit, integration, and doc tests |
| `make runtime-frame-loop-port-check` | PASS: 354 source files, 353 faithful files, 36 faithful member rows, scatter 151/155 |
| `make rust-attribution-check` | PASS: every in-scope Rust source classified |
| `make scripted-golden-compare` | PASS: 353 entries, 324 exact, 670 exact segments, 669 side-channel segments, 0 divergences, 29 honest not-yet rows |

The clean clone intentionally lacked ignored `.riv` fixtures. Required inputs
were copied from the provided checkout for testing only; no fixture binary is
tracked. The first full Cargo link and first scripted-golden runner build hit
the sandbox's disk ceiling. Final Cargo/golden invocations used one build job,
disabled incremental/debug data, and stripped symbols; after removing only
temporary build artifacts, the unchanged tests and comparisons completed
green. The final source diff passes `git diff --check` and contains no temporary
`[DEBUG-...]` instrumentation.

## Review

The required two-axis closeout reviewed `git diff cc7effd0...HEAD`. The
standards axis found no documented-standard violations. The spec axis found no
scope creep or retained dirty-cache behavior error. Its two documentation
findings were corrected before commit: this report and sandbox map are tracked,
and the focus test inventory/coverage wording above matches the ledger.
