# Luau client path, lane G1: audio follow-ups (2026-11-14)

Branch `claude/luaur-lane-g1-audio-78de62`, based on
`levi/luaur-client-stage1c` (`32db50aba8`). Scope: the audio items the E1
inventory (`luaur-client-path-lane-e1-inventory-2026-11-14.md`) left open
and design section 12.2 assigns to the audio lane. Files touched:
`vm/luau_audio_shared.rs`, `vm/luau_audio_shared_codec.rs`,
`vm/lua_audio_samples.rs`, `vm/lua_audio_processors.rs`,
`vm/luau_audio_processor_engine.rs`,
`vm/luau_audio_processor_engine/safe.rs`. No other lane's files.

## Rows

Every row was read against the code at the base first; all five were as
the inventory described, with the two corrections under "Where the
inventory or design was wrong".

| # | Row | Outcome | Commit |
|---|---|---|---|
| 1 | Snapshot / ring hold chains drop recursively (E1 4.4, reproduced abort) | Fixed by construction: the holds list (`Holds`) owns a drop that pops each held value, and for a ring or snapshot it owned outright (`Arc::into_inner` on the handle and on its state) moves that link's own holds onto the list before the link drops. Every nested drop then finds nothing to drop; no depth cap. Arrays and objects inside holds are flattened the same way. A link someone else still owns only loses a reference and keeps its publication. `SharedBudget::close` drops through the same type. | `d6e2473ea0` |
| 2 | `HostStream::restart` spins on an oversized record (E1 3) | Fixed. The loop condition was `read_record(..) != 0`, and `read_record` reports a record larger than `out` as `size + 1` without consuming it. The drain is now `discard_queued_records`: at most `capacity / 4` reads (each record takes at least its length word, so that many empty the ring even with a concurrent sender), and an unconsumed record returns an error. | `84f148eb45` |
| 3 | Processor `pcmReader` uncharged, never freed (E1 5 row 14) | Fixed. A reader is now a slot charged its record capacity, released when the collector has freed both of the reader's natives (`readInto` and the `__index` function): their host entries are `owns_data`, and `drain_freed_natives` runs after init, after every block and at the start of every `Audio.streamReader` call. Freed slots are reused. The slot list itself is charged by the capacity each growth reaches (doubling, at least 4, reserved exactly) and keeps that charge, since it never shrinks (codex round 1). | `091672f9cf`, `34be338bde` |
| 4 | No cap on processors per VM or on the init queue (E1 5 rows 15, 16) | Fixed with fixed caps, below. | `ff8657c353`, `ada422ab91` |
| 5 | Holds charged at a nominal size (E1 5 row 17) | Fixed. A record's or publication's holds list is charged to the ring's or snapshot's `SharedBudget` at `capacity × size_of::<SharedValue>()`, plus, for a ring, one map entry (`size_of::<usize>() + size_of::<Holds>()`). Encoding reserves the list exactly (`layout` already counts handles), so capacity equals the handle count. Charged before the ring or writer lock, refunded when the record is read, the publication is replaced, or the owner closes. | `6d3e01c3b4` |

## Caps

| Cap | Value | Where | What tests and tools use | Headroom |
|---|---|---|---|---|
| Processors starting or running per script VM | 64 (`MAX_LIVE_PROCESSORS`) | `lua_audio_processors.rs`, checked in `Audio.start` after `admit` | At most 3 per VM (`lua_audio_processors` tests; `tools/luau-audio-web/src/qualification_main.lua` 3 starts; `tools/audio-device-validation/src/luau_fixture.lua` 2). No `.riv` fixture starts a processor. | ≥ 21× |
| Queued processor initializations, process-wide | 256 (`MAX_QUEUED_INITIALIZATIONS`) | `luau_audio_processor_engine.rs`, `sync_channel` + `try_send` | The queue drains as fast as inits run; tests start a handful at once (the per-VM cap test queues 65). | ≥ 3.9× over the largest test burst, ≥ 10× over every other test and tool. |
| Processor host memory (readers) | the processor VM's memory ceiling (16 MiB default, `ProcessorMemoryLimit`) | `safe.rs`, `AudioHost.host_memory_limit` | Tests' readers are 2 KiB records (≈ 8,100 fit); the largest test reader is the new 512 KiB one. | n/a (a byte budget, not a count) |

The queue cap is per process, the processor cap per VM: 256 is four VMs'
worth of full processor caps arriving in one burst while the init thread is
busy, which no test or tool comes near. A stopped or failed processor does
not count against the per-VM cap.

## What a script observes differently

All of these are reached only past a limit no fixture or test reaches;
in-budget content is unchanged.

- Dropping a long hold chain no longer aborts the process. Nothing else
  about publication or receipt changes.
- `controller:seekFrame(n)` with a record larger than the stream's chunk at
  the head of its ring raises `audio stream ring holds a record larger than
  its chunk` where it used to hang the main thread. The oversized record
  stays queued; other records before it were discarded as before.
- `Audio.streamReader` in a processor raises `script VM exceeded its
  N-byte host memory budget` (N = the processor's memory limit) when held
  readers would pass it. Before refusing, it finishes the collector cycle
  under way and runs one more (`collect_step(-1)` twice), so a script that
  drops readers as it makes them is never refused: the host bytes are
  invisible to the collector's pacing, and without this a reader per block
  filled the budget before any cycle ran (the first version of the test
  showed it). The reader's natives being freed also means a script that
  keeps only `reader.readInto` and drops the table keeps that function
  working; a reader goes only when both natives are gone.
- `Audio.start` raises `Audio.start: at most 64 processors may run at once`
  at the 65th live processor of a VM, and `Audio.start: at most 256
  processors may wait to initialize at once` when the init queue is full.
- `ring:send` / `snapshot:publish` of a message carrying shared handles can
  raise `shared memory budget exceeded` when the owner's 512 MiB shared
  budget cannot take the holds list (about 32 B per handle plus a map entry
  per ring record). On a released owner they still return `false`, as before.

## Tests (each fails on the old code)

The old behaviour was restored in the working tree for each item (old
recursive drop, old `while read_record != 0 {}`, no limit and no release,
no caps / an unbounded-in-effect queue, zero-byte hold charges), the new
tests run, and the tree restored.

| Test | Old code |
|---|---|
| `luau_audio_shared::dropping_a_long_snapshot_hold_chain_does_not_recurse` (20,000 links, 256 KiB stack) | stack overflow, process abort |
| `luau_audio_shared::dropping_a_long_ring_hold_chain_does_not_recurse` (ring inside object inside array) | stack overflow, process abort |
| `luau_audio_shared::closing_a_budget_with_a_long_hold_chain_does_not_recurse` (head registered first, so `close` reaches it while every other link is owned by its predecessor) | stack overflow, process abort |
| `luau_audio_shared::a_dropped_chain_keeps_the_holds_of_links_still_owned_elsewhere` | passes (guard for the fix, not a repro) |
| `lua_audio_samples::seek_with_an_oversized_record_in_the_ring_is_a_script_error` (the inventory's sketch, in Luau) | `seekFrame did not return: Timeout` (20 s) |
| `luau_audio_processor_engine::processor_stream_readers_are_charged_to_the_processor_budget` | 64 readers of 512 KiB all accepted |
| `luau_audio_processor_engine::processor_stream_readers_are_released_when_collected` | reader count 100 after collection, not 0; against `091672f9cf` (before the slot-list charge), 0 bytes held after collection where the list's storage is still allocated |
| `lua_audio_processors::authored_audio_caps_live_processors_per_vm` | 100 processors started |
| `luau_audio_processor_engine::the_initialization_queue_refuses_jobs_past_its_bound` | fifth queued job accepted |
| `luau_audio_shared::ring_and_snapshot_holds_are_charged_at_their_size` | 0 bytes charged for 100 handles |
| `luau_audio_shared::holds_past_the_shared_budget_are_refused` | send accepted |

One existing test changed: `shared_budget_refunds_and_close_breaks_payload_cycles`
set the budget to exactly the ring's 64 bytes and then sent a self-hold;
the hold now costs its size, so the budget gets room for one hold and the
"snapshot does not fit" check moves after the send (the budget is full
there).

## Checks

Run on this Mac with `NUXIE_EXECUTION=local`, `RIVE_RUNTIME_DIR` at the
`cd5351c3` pin worktree. Load averages (1/5/15 min) at start and end of
each run; other lanes were building in parallel throughout.

| Check | Tip | Result | Load |
|---|---|---|---|
| `python3 tools/bazel/runtime.py test -p nuxie-scripting` | `34be338bde` | 39/39 targets pass; lib 529 passed, 1 ignored | 12.19 10.99 11.81 → 13.16 11.44 11.80 |
| same | `6d3e01c3b4` | 39/39 targets pass | 5.02 8.70 8.28 → 7.27 8.63 8.42 |
| `tools/ratchets/panic-lints.sh crates/nuxie-scripting crates/nuxie-scripting:nuxie-scripting__publisher@wasm` | `34be338bde` | PASS: 0 forced sites for all seven lints on both rows (19 / 18 other clippy diagnostics, not ratcheted, unchanged) | 13.16 11.44 11.80 → 13.16 12.77 12.34 |
| `tools/ratchets/unsafe-count.sh crates/nuxie-scripting` | `34be338bde` | PASS: 47 = baseline | same window |
| `python3 tools/bazel/runtime.py test -p nuxie --features scripting,test-support` (after `make fixtures` and a copy of the main checkout's `crates/nuxie/tests/fixtures/`) | `6d3e01c3b4` | 41 of 42 targets pass (one of them cached); the one red is `stateful_component_image_bind` (`upstream_data_binding_images_direct.rs:191`), the known red | 18.04 16.00 12.73 → 8.06 11.14 11.55 |
| `make scripted-golden-compare` | `6d3e01c3b4` | `entries=364 exact=348 diverges=16 unsupported-feature=0 not-yet=0`, as expected | 20.59 11.67 11.19 → 12.58 13.17 13.32 |
| `codex review --base levi/luaur-client-stage1c` | rounds 1-2 | see below | n/a |

The `nuxie` and golden runs are on `6d3e01c3b4`, before `34be338bde`;
that commit changes only the processor reader slot accounting, which no
fixture reaches (no fixture starts a processor), so they were not rerun.

### codex review

Two rounds; stopped after round 2 raised nothing in this lane's diff.

- Round 1, in scope, right: "Keep retained PCM slot capacity charged"
  (`safe.rs`): freeing a slot refunded the slot's size although the list
  keeps its storage, and `try_reserve(1)` could double the list uncharged.
  Fixed in `34be338bde` with a test that fails without it.
- Rounds 1 and 2, outside this lane's files (G4's `lua_text.rs`,
  `lua_path.rs`; E2b's `lua_font.rs`, `host_memory_budget.rs`): text
  outline geometry uncharged; path-measure storage uncharged; decoded-font
  charge lost through view-model font properties; the main budget refuses
  without letting the collector reclaim. Not verified or changed here;
  written up for their owners in the shared notes folder
  (`luaur-lane-g1-audio-out-of-scope-review.md`). The last one is the same
  pacing gap this lane hit in the processor reader and solved locally.

## Not verified

- wasm: on the browser transport, rings and snapshots are JS handles and
  their holds live in JS (`luau_audio_shared_web.rs`); rows 1 and 5 do not
  apply to that path and were not looked at there. The wasm row was linted
  (panic lints), not run. `submit_initialization` on wasm runs the job
  inline, so the queue cap is native-only; the per-VM processor cap applies
  to both.
- Release-build thresholds: the abort repro ran in the Bazel test build
  (debug) at 20,000 links on a 256 KiB stack.
- A real-time device run: the collection a full reader budget triggers
  runs on whichever thread calls `Audio.streamReader` (the audio thread when
  called from `process`), bounded by the processor heap (≤ the processor
  memory limit). Not measured for latency. Freed readers' buffers are also
  deallocated on that thread.
- The init input copy queued with each job is still uncharged (E1 row 16
  "charge the input"); the queue bound limits how many exist. Not in this
  lane's brief.
- E1 2(a) rows 12, 13 (borrows in `StreamController` / `Audio.start`) are
  latent and were not part of this brief; untouched.

## Where the inventory or design was wrong

- Design 12.2 / brief: "charge it to the `HostMemoryBudget` of the VM that
  owns it". The processor VM is a `nuxie_luau_vm::HostedVm` on the audio
  thread with no `HostMemoryBudget`; that type is `Rc<Cell>` and the
  processor must be `Send` (checked at `safe.rs`). The processor got the
  same rule as a plain counter in its host state: host budget = its VM's
  memory ceiling, same error text. The main VM's `ScriptedPcmReader` stays
  charged to the shared budget, as before.
- E1 row 17: "a snapshot keeps 2 copies" of holds. It keeps one: `write`
  takes the previous slot's holds out and drops them after the lock; only
  the live publication's holds stay. (Readers clone the list transiently.)
- E1 row 17: "9 B per handle" is the handle's tag and index inside the
  ring's already-charged bytes, not a holds charge; the holds list itself
  was charged nothing.
- E1 4.4: confirmed, including `close` recursing when it reaches a link
  whose successors are owned only through holds.
