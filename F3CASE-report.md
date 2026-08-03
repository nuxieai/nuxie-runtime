# F3CASE lane report

## Status

Complete for the requested F3 case-promotion scope.

- Started on `levi/endgame-f3-cases` at `fb8b7afd0720`, exactly matching
  `origin/main` before work.
- Verified the pinned C++ oracle at
  `/Users/levi/dev/oss/rive-runtime@4ac7b32798da0482e441ef09304dc3b480ed3ee5`.
- Ran the required fixture refresh from
  `/Users/levi/dev/nuxie-runtime/fixtures/` and `make fixtures` successfully.
- Promoted all 62 requested non-F6 pending cases as faithful Rust test
  equivalents. Together with the four already-complete baseline cases, the
  ledger now records 66 complete current-pin cases and zero pending non-F6
  cases.
- Kept all 13 semantics cases pending with an explicit F6 dependency and kept
  exactly four blob cases on the S4-45 WATCH.

## Commits

1. `1e1b93ef` — `[F3CASE] Port command-loop lifecycle cases`
2. `c269442c` — `[F3CASE] Port resource and view-model cases`
3. `0e3f3b03` — `[F3CASE] Complete non-F6 command queue case ports`
4. `ebe72deb` — `[F3CASE] Tighten command queue oracle assertions`
5. `3bd9e5ec` — `[F3CASE] Preserve external image handle identity`
6. `fa898109` — `[F3CASE] Tighten final ledger evidence`

## Evidence

- `cargo check -p nuxie` — PASS.
- `cargo test -p nuxie --test command_queue --no-fail-fast` — PASS,
  73 passed, 0 failed.
- The final 15 promoted cases were also run independently with exact-name
  filters; all passed. Earlier coherent slices were likewise proven with
  focused exact-name runs before their commits.
- `docs/p3f-command-queue-test-ledger.md` has one cited Rust-evidence row per
  complete upstream case and reconciles to
  `66 complete + 0 pending non-F6 + 13 pending F6 + 4 S4-45 WATCH = 83`.
- `git diff --check` — PASS before the completion commit.
- No tolerance was widened, and the Luau engine pin was not changed.
- Two independent closeout reviews checked standards and the F3CASE spec.
  Their findings were resolved with direct loader dispatch, bind identity,
  exact typed callback/list ordering, clear-value assertions, global settled
  delivery, precise ledger wording, and stable external-image handle
  round-tripping. The focused gate above passed after the final fixes.

## Residue and correspondence

- No port implementation/test file was added or moved, so the four-place
  residue rule has no new source-file residue to reconcile. The required lane
  report and sandbox recovery map are the only new files.
- `port-manifest.toml` is unchanged; the recorded scatter remains 154, below
  the lane ceiling of 155. The narrow `#[doc(hidden)]` state-machine,
  data-context, and list-inspection methods are crate-boundary test/command
  bridges: `CommandServer` lives in `nuxie` while the pinned operations and
  retained identities are owned by `nuxie-runtime`. They add no new manifest
  scatter row or exception.
- The `command_queue.cpp` and `command_server.cpp` correspondence rows were not
  promoted to complete because the 13 F6-gated command cases remain. The
  `lua_scripted_context.cpp` row was untouched and remains pending; this lane
  did not produce genuine whole-file completion for any of those rows.

## Queued items

- F6 owner: the 13 semantics manager/action/focus/diff command cases listed in
  the ledger.
- S4-45 owner: the four blob handle/message protocol cases on WATCH.
- Landing orchestrator: full batteries, golden comparisons, and any aggregate
  port-manifest gate; these were intentionally not run under the lane disk and
  test gate.
