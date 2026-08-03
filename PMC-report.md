# Port-manifest command-transport reconciliation

## Adjudication

The honest legacy port status for both `src/command_queue.cpp` and
`src/command_server.cpp` is **partial**. P3F landed the full structural queue
and server owners and 14 focused Rust tests, but only 4 of the 83 upstream
`command_queue_test.cpp` cases have complete equivalent assertion coverage.
The remaining fixture-specific, semantic, and S4-45 blob cases prevent a
whole-file faithful claim.

The ledgers use different status vocabularies, so the same adjudication is
encoded as follows:

| Ledger | Command transport classification |
|---|---|
| register seed in `tools/port-manifest/port_manifest.py` | `partial` |
| `port-manifest.toml` | `partial` |
| `file-correspondence-manifest.toml` | `status = "pending"`, `verification = "pending-verification"`, `legacy_port_status = "partial"` |

`partial` is not a valid v1 whole-file correspondence status; that schema
permits only `faithful`, `divergent-by-decision`, and `pending`.
`pending-verification` is an independent verification field, not a behavioral
status. RF-33 also requires a file to remain `pending` while its member/test
evidence is incomplete. Promoting either correspondence row to `faithful`
would therefore overstate the landed evidence, while adding `partial` to that
schema would conflate the legacy port inventory with whole-file correspondence.

## Reconciled ownership and evidence

- `src/command_queue.cpp` maps to
  `crates/nuxie/src/command_queue.rs`.
- `src/command_server.cpp` maps to
  `crates/nuxie/src/command_server.rs; crates/nuxie-scripting/src/vm/command_server.rs`.
- The seed, port-manifest row, and correspondence row now use byte-identical
  Rust module lists and P3F evidence notes for each upstream file.
- B6-0121/B6-0122 verdict, cluster, and audit-record fields are unchanged.
  The B6-0122 note retains the FL-E1 fragment and the crate-boundary MR
  exception.
- `docs/p3f-command-queue-test-ledger.md` now records the advanced
  `rive-runtime@4ac7b327` total: 4 complete + 62 pending non-F6 + 13 pending
  F6 + 4 pending S4-45 blob WATCH = 83.

The port-manifest gate also exposed stale behavior-neutral source paths left by
earlier file moves. The seed and generated manifest now point at the existing
`audio_engine.rs`, `audio_sound.rs`, and `lua_promise.rs` owners and carry the
already-landed P3-b DataContext classification. These repairs do not change a
port verdict; they make the declared Rust attribution checkable.

## Verification

- `RIVE_RUNTIME_DIR=/Users/levi/dev/worktrees/rive-runtime-b73bc675 make port-manifest-check`
  - 20/20 unit tests passed.
  - 447/447 manifest rows passed; all Rust module paths verified.
- `make runtime-frame-loop-port-check` passed.
- `make rust-attribution-check`
  - 10/10 unit tests passed.
  - Every in-scope Rust source is classified.
- `git diff --check` passed.
