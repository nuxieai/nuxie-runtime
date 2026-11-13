# Luau client path, lane A: crash paths a script can reach (2026-11-14)

Design: `docs/luaur-client-path-safe.md` section 3. Survey:
`docs/evidence/luaur-client-path-survey-2026-11-14.md` (cited as S).
Branch `claude/luaur-lane-a-crash-paths-86bf19`, from `7fd7c6f5f8`.

## What changed

| Commit | Item | Change |
|---|---|---|
| `2882f840b2` | A.1 | One limit set for a shared audio value (`luau_audio_shared_codec.rs`): depth, value count, encoded bytes. Charged before copying in `to_shared` (main VM) and `read` (processor VM), and again in `decode`, which now checks nesting, value count and every container count against the remaining input before recursing or reserving. |
| `23d3bc4f0f`, `b00e47e90a` | A.2 | `MeshInstances(count)` and `:resize(count)` charge a per-VM host budget before `try_reserve_exact`. The charge is the largest capacity asked for and is refunded only on drop (the second commit, from codex review: refunding on shrink let the renderer's mirror keep memory past the budget). |
| `8012ecbec7` | A.3 | Audio stream chunk size computed with checked arithmetic against `MAX_SHARED_BYTES`; sample and record buffers use `try_reserve_exact`. A main-VM PCM reader's record and scratch are charged to the VM's shared budget. |
| `6f3ba47dc8` | A.4 | `Drop` for `ScopedRendererAccess` and `ActiveRendererBorrow` cannot panic; a broken invariant is recorded and returned by the next renderer access. |
| (this commit) | — | This file. |

Script-visible behaviour: only crashes (stack overflow, abort on allocation
failure, debug overflow panic, panic in `Drop`) became script errors. Every
existing error text is unchanged; new texts are listed with each limit.

## A.1 Shared audio values: the limits and why

| Limit | Value | Where checked | Error |
|---|---|---|---|
| Depth (tables on one path) | 64 | `to_shared`, `read` (unchanged checks, now naming `MAX_SHARED_DEPTH`); `decode` (new) | builders: `audio message nesting exceeds 64 tables` (unchanged); decoder: `shared value nesting exceeds 64 tables` |
| Values (containers included) | 2^22 = 4,194,304 | `to_shared`, `read`, `decode` (new) | `audio message exceeds 4194304 values` |
| Encoded bytes | `MAX_SHARED_BYTES` = 256 MiB (unchanged) | `layout` at send/publish (unchanged); now also `to_shared`, `read` as the value is built, and `decode` on its input | `shared value too large` (the existing text) |

Why these numbers.

- Depth 64: the survey was wrong that `to_shared` and `read` had no depth
  limit; both already refused a 65th table (`lua_audio_processors.rs:684`,
  `safe.rs:684` at `7fd7c6f5f8`). Keeping 64 keeps what scripts observe. The
  unbounded paths were the decoder (any byte stream, 5 bytes per level: a
  256 MiB record is 50 million levels) and everything downstream of it
  (`from_shared`, `push`, `encode`, `contains_handles`, the derived `Drop`
  of `SharedValue`), which only ever see a value from a builder or the
  decoder and so are now bounded by 64 too.
- Values 2^22: the safe VM charges 16 bytes per table slot
  (`nuxie-luau-vm/src/heap.rs:317`) plus 64 per table. A value with no table
  reachable twice that fits in the default 64 MiB file VM
  (`DEFAULT_SCRIPT_VM_MEMORY_BYTES`, `vm.rs:854`; the main `ScriptVm::new`
  ceiling is 16 MiB, `vm.rs:279`) therefore has fewer than 2^22 values. So
  the limit refuses only values where a table is reached many times over,
  each reach copied into host memory: 2049 references to one 2048-slot table
  is 4.2 million host values from about 64 KiB of script memory. Before, that
  built the whole tree on the host and only then compared its encoded size;
  references to a 1 MiB string 300 times allocated 300 MiB before failing.
  At 2^22 values a host tree is at most about 130 to 300 MiB (a value is 24
  to 32 bytes, plus map nodes for records), the same order as the 256 MiB
  byte limit that already existed. A host that raises a VM's ceiling above
  64 MiB (`ScriptExecutionLimits::with_max_memory_bytes`) can hold flat
  tables past the limit; they are refused with the error above.
- Bytes: the existing limit, now charged before each string is copied rather
  than after the whole tree exists. For ring `send` and snapshot `publish`
  the result is unchanged (same text; the old path failed in `layout`). New
  for `Audio.start`'s argument and `Audio.streamReader`'s descriptor, which
  had no byte limit.

Real content: the only audio scripts in the repo
(`tools/audio-device-validation/src/luau_fixture.lua`,
`luau_oscillator.lua`, `tools/luau-audio-web/src/qualification_*.lua`) send
records of two to five fields, one level deep. The crate's tests send at
most a few levels. No fixture is near any limit.

Error codes: the host-command limits report through
`ScriptResourceGuard::fail` with `script.resource.*` codes, which trips the
script's resource guard. The task asks for an ordinary script error on the
posting side, and the existing depth text is script-observable, so the new
limits use plain runtime errors in the existing audio style instead of
codes. If codes are wanted, they change the depth text; that is a decision
for the coordinator.

## A.2 MeshInstances

Budget: 16 MiB of host memory per script VM across all live `MeshInstances`
(`MAX_IMAGE_MESH_INSTANCE_BYTES`), which is 349,525 instances of 48 bytes,
the same as the main VM's own 16 MiB ceiling, which these instances are
outside of. The renderer mirrors the staged instances once
(`make_image_mesh_instances(count)`, `edit(len)`). Growth doubles while the budget
allows it (the upstream test grows one instance at a time) and is exact
otherwise. A shrink refunds nothing, because the renderer's mirror
(`ImageMeshInstancesStorage::edit`, `Vec::resize`) keeps the capacity of the
largest count it was given; the whole charge is refunded on drop. Staging
capacity is exactly the charge; the mirror's `Vec` grows by doubling to at
most twice the largest count, so host memory for one VM's instances is at
most about three times the budget (48 MiB). Error: `MeshInstances may hold
at most 349525 instances per script VM`. No fixture uses `MeshInstances`; upstream tests use 100.

Before: `resize(4294967295)` asked for 206 GB (`Vec::resize`), which aborts
on allocation failure; `MeshInstances(n)` did the same through `vec![..; n]`
and the factory.

## A.3 Audio stream sizes

- `lua_audio_samples.rs` `stream`: `16 + chunkFrames * channels * 4` was
  computed in `u64` without checks, from two script-chosen `u32`s
  (`AudioSource:stream(rate, channels, chunk, capacity)`; `make_reader` keeps
  any nonzero `channels`). `(2^31, 2^31)` overflows `u64`: a panic in debug,
  a wrap to 16 bytes in release, which passed the size check and then
  allocated `2^62` samples. Now checked; stated maximum: one chunk with its
  header is at most `MAX_SHARED_BYTES` (unchanged). Samples and record use
  `try_reserve_exact`. Each stream's ring (at least one chunk) is charged to
  the VM's shared budget (512 MiB, `DEFAULT_SHARED_BUDGET`) before these, so
  a stream's two buffers are at most twice its charged ring.
- `lua_audio_processors.rs` `stream_reader_value`: the size now comes from
  `PcmStreamDescriptor::chunk_sample_bytes` (checked, at most
  `MAX_SHARED_BYTES`). The descriptor fields reach this from a script table,
  and a reader allocates a record and a scratch chunk of up to 256 MiB each,
  uncharged: `Audio.streamReader` in a loop allocated without bound. Both are
  now charged to the VM's shared budget first (error: `shared memory budget
  exceeded`, the budget's existing text).

## A.4 Renderer `Drop`

`ScopedRendererAccess::drop` removes its own frame wherever it is in the
stack (its borrow ends there, so its pointer must go), records "scoped
renderer stack order" if it was not on top and "scoped renderer stack
underflow" if absent. `ActiveRendererBorrow::drop` records "active renderer
borrow outlived its scope" if its frame is gone. Both use `try_with` and
`try_borrow_mut`; a busy stack (unreachable: no borrow of it is held across a
call) poisons renderer access on that thread rather than leave a stale
pointer reachable. `with_active_renderer` returns a recorded message once,
as a runtime error. Out-of-order removal is only possible for independent
renderers: a nested scope reborrowed from an outer one cannot outlive it
(`PhantomData<&'a mut dyn Renderer>`). The thread-local flag is shared by all
VMs on the thread, so the error reaches whichever renderer is accessed next.

## Item 5: allocations from S3.3 marked "not traced" or "none"

| Site (S3.3) | S3.3 said | Now |
|---|---|---|
| `vm/lua_mesh.rs:165,172,197` (`MeshInstances(n)`, `:resize(n)`) | none | Budget (A.2). |
| `vm/lua_audio_processors.rs:340-347` (reader scratch) | `try_reserve_exact`, unchecked multiply | Checked size from the descriptor (at most `MAX_SHARED_BYTES`); record and scratch charged to the VM's shared budget (A.3). |
| `vm/lua_audio_samples.rs:518,523` (stream samples, record) | not traced | Script-chosen `channels` and `chunkFrames`, checked against `MAX_SHARED_BYTES`, `try_reserve_exact`, after the ring is charged (A.3). |
| `vm/luau_audio_shared_codec.rs:174-178` (decode array) | `try_reserve_exact` | Count checked against remaining input and the value limit before reserving (A.1). Object maps are bounded the same way (5 bytes per entry). |
| `shader_asset.rs:653,654,752,801` | not traced | Entry counts are `u8` (at most 255), read with the cursor; bounded. |
| `shader_asset.rs:861` (binding map) | not traced | `u32` count, but `count * entry_size + groups * group_size` is checked (overflow and against `cursor.remaining()`, :851-859) before `with_capacity`; bounded by the asset's length. |
| `shader_asset.rs:928,977` (reflection) | not traced | `u8` / `u16` counts that must equal already-decoded lengths; bounded. |
| `shader_asset.rs:1020` (`decode_interface_variables`) | not traced | Called with `u8` counts (:957-960); bounded. Shader asset sites at 1277, 1282, 2060, 2803 are in `mod tests` (:1191). |
| `vm/luau_audio_web.rs:76` (`output.resize(required)`) | wasm only | Size is the browser host's response length (at most `i32::MAX`); not script-chosen directly, but a response can carry script data (shared values as JSON). Not changed: this file is lane C.4's. A failed allocation traps the kernel. |
| `vm/luau_audio_web.rs:151` (`alloc(length)`) | wasm only | Length chosen by the JS caller; same. Not changed. |

Found outside S3.3's regex (it matched `with_capacity`/`resize`/`reserve`,
not `push`), for lane E.3's list, not changed here:

- `VertexBuffer:add(...)` and `TriangleBuffer:add(a, b, c)`
  (`vm/lua_mesh.rs`) push into host `Vec`s with no bound; `update` mirrors
  them into a render buffer of the same size. Each callback is limited by
  its interrupt budget, but the buffers persist across callbacks, so a script
  can grow them without limit over frames.
- The processor VM's `pcmReader` (`luau_audio_processor_engine/safe.rs`
  `pcm_reader`) allocates a reader record of up to `MAX_SHARED_BYTES` per
  call and keeps every reader in `host.pcm_readers`; uncharged, and on the
  audio thread. Charging it needs a `SharedBudget` reachable from the
  processor host (none today) or from the ring (the web ring has none). Not
  fixed: it needs a design choice about which budget a processor charges.


## Item 5: recursion over script-built or file-built data

Method: a per-file call-graph search for strongly connected components
(direct calls, `self.` calls, function items passed as values such as
`.map(f)`), then reading each candidate; the facade, view-model, JSON,
`pairs`, `require` and shader paths were also read by hand.

| Walk | Depth driven by | Bound | Verdict |
|---|---|---|---|
| `to_shared` / `from_shared` (`vm/lua_audio_processors.rs`), `read` / `push` (`luau_audio_processor_engine/safe.rs`) | script tables, shared values | 64 tables (A.1) | bounded |
| codec `write`, `value`, `contains_handles` (recursion through `.any(Self::contains_handles)`), derived `Drop`/`Clone`/`Debug` of `SharedValue` | shared values, byte streams | 64 tables (A.1; `decode` checks it now) | bounded (was unbounded through `decode`) |
| `host_commands.rs` `value_at_depth` ↔ `table_value`; `host_value_bytes`, `host_value_nodes` (through `.map(f)`) | script tables | `max_value_depth`, default 32, cap 64 | bounded |
| Host → Luau → host re-entry: `__index`/`__namecall` dispatchers (`lua_font.rs`, `lua_rive_file.rs`), `__tostring` from `Value::to_string`, view-model `get`, property listeners, artboard draw visitor, `pcall`/`xpcall` guards | script metatables and callbacks | `nuxie-luau-vm` `max_reentry` 16 (`hosted.rs:541`, `limits.rs:50`) and `max_c_calls` 200; the crate does not override `Limits` | bounded |
| Facade `FromLua`/`IntoLua` for `Option`, `Vec`, arrays, maps, sets, tuples, `Variadic` (`luau/facade/conversion.rs`) | Rust type parameters | fixed at compile time; tables stay handles | not data-driven |
| View models (`vm/view_model.rs`): nested and list view models | script access | created lazily, one level per access; `parent()` lazy, `same_identity` a loop | not recursive here (walks inside `nuxie_runtime` not checked) |
| `serde_json::Value` (web audio, browser kernel) | browser host JSON | serde_json's default recursion limit 128 | bounded |
| **`Error::CallbackError { cause: Arc<Error> }`** (`luau/facade/error.rs:17`): `is_structured` (:124), `Display` (:241), derived `Debug` and `Drop`, `log_error` (`vm/logging_scripting_context.rs:13`), `observe_vm_error` (`vm/resource_limits.rs:132`) | a script that catches a structured error object and re-raises it through a host callback that calls script code, in a loop: each pass adds a layer (`recover_error_object` :156 wraps it, `raise` :139 re-raises it) | none; iterative on the Luau side, so `max_reentry` does not apply. Needs a structured error to start (`CallbackDestructed`, `UserDataDestructed`, `RecursiveMutCallback`) | **unbounded, plausible, not reproduced** |
| **`sort`** (`vm/native_registration.rs:77`, recursing at :99 per dependency and :114 for each remaining pending script) | file data: the number of pending script assets | a cycle guard only (:90-93); depth grows with the number of pending scripts, no cap found | **unbounded in file size** (signed files only on devices) |

Neither unbounded walk is in a file this lane touches, so both are
reported, not fixed. Side note from the same read: `sort` returns early for
an already visited `next` without continuing through `remaining`, so later
pending scripts can get no order; possibly a separate correctness bug.


## Tests and the before-state

Each fix's test, adapted to the pre-fix API (limits written as numbers),
was applied to `7fd7c6f5f8` and run: all eight fail there.

| Test (on this branch) | On `7fd7c6f5f8` |
|---|---|
| `luau_audio_shared::decode_refuses_nesting_past_the_shared_depth_limit` (65 levels; the overflow itself is never run) | fails: 65 levels decode |
| `luau_audio_shared::decoding_a_raw_record_fails_that_message_only` | fails: a raw record decodes as a value |
| `luau_audio_shared::decode_refuses_more_values_than_the_shared_node_limit`, `decode_refuses_container_counts_the_input_cannot_hold` | not run on the old code (no limit existed; the second reserved up to 4G elements) |
| `lua_audio_processors::audio_message_past_the_value_limit_is_a_script_error` | fails: send built 4.2 million host values and returned false |
| `luau_audio_processor_engine::processor_message_past_the_value_limit_is_a_script_error` | fails: same, in the processor VM |
| `lua_audio_processors::shared_value_limits_are_charged_before_copying` | wiring test with small limits; new API |
| `lua_audio_processors::pcm_readers_are_charged_to_the_shared_budget` | fails: 200 readers created, no error |
| `lua_audio_samples::stream_formats_past_the_shared_limit_are_script_errors` | fails: "attempt to multiply with overflow" (debug); the release build wraps |
| `lua_renderer::mesh_instances_past_the_vm_budget_are_a_script_error` | fails: `resize(349526)` succeeded (the 2^32 case is never run on old code: it aborts) |
| `lua_renderer::renderer_scopes_ended_out_of_order_report_an_error_instead_of_panicking` | fails and **aborts the test process**: the `Drop` panicked, then a second panic during unwinding gave "thread caused non-unwinding panic. aborting" |

No test uses `catch_unwind`. The stack-overflow case is shown by depth 65
against the limit of 64, never by overflowing.


## Checks

Load averages are the 1, 5 and 15 minute figures at start and end. The host
was shared with other sessions throughout.

| Check | Result | Load (start → end) |
|---|---|---|
| `python3 tools/bazel/runtime.py test -p nuxie-scripting` (at `6f3ba47dc8`) | 39 of 39 targets pass | 17.0/22.2/16.1 → 12.0/19.3/15.7 |
| same, at the final tip `b00e47e90a` | 39 of 39 targets pass | 7.3/9.3/12.1 → 7.8/9.5/11.8 |
| `make luau-vm-diff` (routed to the pool) | 409 runs, 0 disagreements; corpus reaches every admitted opcode | 12.0/19.3/15.7 → 9.0/13.0/14.1 |
| `make luau-conformance` | PASS: 55 scripts, 37 required OK, 0 required not passing, 0 panics | 9.0/13.0/14.1 → 9.7/12.2/13.6 |
| `python3 tools/bazel/runtime.py test -p nuxie --features scripting,test-support` | 41 of 42 targets pass (one cached); the failure is `stateful_component_image_bind` in `upstream_data_binding_images_direct`, the known red on main | 9.6/12.1/13.6 → 9.6/12.1/13.4 |
| `make scripted-golden-compare` | exit 0: 364 entries, 348 exact, 16 diverge | 9.6/12.1/13.4 → 12.6/16.3/15.3 |
| `codex review --base levi/luaur-client-safe` | round 1: one finding (mesh shrink refund), fixed in `b00e47e90a`; round 2: no findings | — |

`nuxie`, vm-diff, conformance and golden ran at `6f3ba47dc8`; the final
commit changes only `vm/lua_mesh.rs` and its test, which the second
`nuxie-scripting` run covers.


## Not verified

- The release-build wrap in the stream size was derived from the code, not
  run (tests build with overflow checks).
- The 2^32 `MeshInstances` resize and the deep decode were never run on the
  old code; their failure modes (abort on allocation, stack overflow) are
  derived.
- The wasm variant (`luau_audio_shared_web.rs` includes the same codec) was
  not built; the change there is the shared codec file only.
- Under `-Cpanic=abort`: not run (lane F's gate).
- The `CallbackError` chain was found by reading; a script that starts it
  was not written.
- Walks inside `nuxie_runtime` (view-model internals, draw visitors) and the
  VM's own GC traversal of deeply nested tables were not checked.


## Where the design was wrong or incomplete

1. S3.3 says `to_shared` and the processor's `read` had no depth limit.
   Both already refused a 65th table. The unbounded recursion was the
   decoder, and everything downstream of what it returns. The design's fix
   (one limit set at build and at decode) was right; its premise about the
   builders was not.
2. The real gap at the builders was not depth but copying: a table reached
   many times over is copied once per reach, with no value or byte limit
   until the whole tree existed (and none at all for `Audio.start` and
   `streamReader`). The value limit and byte charge address that.
3. Error codes: modelling the limits on `host_commands.rs` would route them
   through `ScriptResourceGuard::fail`, which trips the script's resource
   guard. That is not "an ordinary script error", and the depth text
   already exists. The new limits use plain errors; codes are the
   coordinator's call.
4. A.2 "a host-side budget like the GPU canvas": the GPU canvas budget never
   refunds. Copied as is, a script that resizes repeatedly would run out.
   This budget refunds on drop. It cannot refund on shrink because the
   renderer mirror keeps its capacity (found by codex review).
5. A.3 lists two sites; the reader buffers behind them were the larger
   problem (uncharged, repeatable from a script loop). The main-VM reader is
   charged now; the processor's `pcmReader` needs a budget it does not have.
6. Section 7.3's count of 54 allocation sites comes from a regex that misses
   `push` growth (`VertexBuffer`, `TriangleBuffer`); lane E.3 should list by
   data flow, not by call name.
7. Section 7.2 (lane E): two unbounded recursions found, the
   `CallbackError` chain and `native_registration.rs` `sort`, outside this
   lane's files.

