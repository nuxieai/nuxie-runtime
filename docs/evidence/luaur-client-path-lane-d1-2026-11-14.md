# Luau client path, lane D1: audio files panic-free (2026-11-14)

Design: `docs/luaur-client-path-safe.md` section 6 (lane D1) and section
5 item 4 (browser audio host). Survey:
`docs/evidence/luaur-client-path-survey-2026-11-14.md` (S). Lane A's limits
and budgets in these files are kept unchanged.
Branch `claude/luaur-lane-d1-audio-4b5386`, from `9eb238d58f`
(= `levi/luaur-client-safe`).

## Result

Forced panic-lint sites (the seven lints), measured with
`tools/ratchets/panic-lints.sh --baseline <scratch> crates/nuxie-scripting
'crates/nuxie-scripting:nuxie-scripting__publisher@wasm'`, counted per file
from the clippy diagnostics files.

| File (`crates/nuxie-scripting/src/vm/`) | native before | native after | wasm before | wasm after |
|---|---:|---:|---:|---:|
| `luau_audio_shared.rs` (native only) | 64 | 0 | – | – |
| `luau_audio_shared_codec.rs` (included by both) | 2 | 0 | 2 | 0 |
| `luau_audio_shared_web.rs` (wasm only) | – | – | 14 | 0 |
| `luau_audio_samples.rs` | 27 | 0 | 27 | 0 |
| `lua_audio_samples.rs` | 12 | 0 | 12 | 0 |
| `lua_audio_processors.rs` | 24 | 0 | 24 | 0 |
| `lua_audio.rs` | 0 | 0 | 2 | 0 |
| `luau_audio_processor_engine.rs` | 12 | 0 | 17 | 0 |
| `luau_audio_processor_engine/safe.rs` | 14 | 0 | 14 | 0 |
| `luau_audio_processor_engine/browser_kernel.rs` (wasm only) | – | – | 3 | 0 |
| `luau_audio_web.rs` (wasm only) | – | – | 6 | 0 |
| **These files** | **155** | **0** | **121** | **0** |
| Other files of the crate | 321 | 321 | 320 | 320 |
| **Crate row** | **476** | **321** | **441** | **320** |

Ratchet rows to set at merge (not edited here): `crates/nuxie-scripting`
321, `crates/nuxie-scripting:nuxie-scripting__publisher@wasm` 320. The
baseline file in `tests/ratchets/` does not yet carry these rows, so the
tool prints them as untracked.

`unsafe` (ratchet pattern `unsafe (fn|\{|impl|extern)`), wasm variant:
`luau_audio_web.rs` 12 before, 0 after; the new boundary module
`vm/luau_audio_web/host.rs` holds 8 (below). No other audio file has any.

## Shape changes

One commit per file group:

| Commit | Files | Change |
|---|---|---|
| `cef44bffaf` | `luau_audio_shared.rs`, `luau_audio_shared_codec.rs` | Ring storage is a `RingBytes` type: a position is below the capacity and every copy walks the storage from a position around the end (`split_at_checked` + `chain` + `zip`), so no wrapped index is computed. `RingState::push(parts)` checks the free space, writes and accounts in one place; `pop(size)` advances the head. Snapshot copies are chosen by a `Slot` enum decoded from `published` (0, 1, 2), with `of`/`of_mut` destructuring the two-element arrays, instead of `published - 1`. Region bytes are an iterator of (word, byte lane) from the offset; the per-byte write keeps `fetch_update` (always `Some`, result ignored) instead of `expect`. Budget charges and refunds use saturating arithmetic after the existing checks. The codec's byte reader reads fixed arrays (`array::<N>()`, `let [tag] = ...`). `receive()`/`latest()` (the two `expect`s, "internally encoded") were called only by tests and are now `#[cfg(test)]`. Blob copy is `try_reserve_exact` + `extend_from_slice` (no `copy_from_slice`). |
| `65e7c2bdb7` | `luau_audio_shared_web.rs` | `Value[...]` reads become `get(...)` (same result: missing is `Null`); writes (`args["op"] = ..`, whose `IndexMut` panics on a non-object) build a `Map`. Blob/region reads copy by `zip` after the existing length check. Same `#[cfg(test)]` for `receive`/`latest`. |
| `d6d61b1eb3` | `luau_audio_samples.rs`, `lua_audio_samples.rs` | A frame's byte size is `PcmStreamDescriptor::frame_bytes() -> NonZeroUsize`; divisions and remainders are by that type. A record is `record.get(..size)` then `split_first_chunk::<16>()` and a destructured `RecordHeader`; the sample copy zips the record's remaining bytes against the output's free bytes and counts what it copied. `prefix`/`prefix_mut` give a clamped prefix for the fills. `Registry::id()` returns the script error "audio request identity exhausted" instead of `expect`; `registry()` builds and returns the registry instead of installing then `unwrap`ping; the completion loop matches instead of two `unwrap`s. Stream `chunkFrames` is a `NonZeroU32` after its existing check (the `div_ceil` divisor), the record stride a `NonZeroUsize`. |
| `39593b503e` | `lua_audio_processors.rs` | App-data registries: the existing `shared_budget(lua)?` (its existing error text) instead of four `expect`s; module and owner registries are inserted on first use instead of `expect`. `single(&args)` returns the one argument with the existing arity error. `RegionHeld` is read once and inserted if absent. Array tables are filled from `(1..).zip(values)`. PCM reader scratch copies use the clamped prefixes. |
| `a1f903a996` | `luau_audio_web.rs`, `luau_audio_web/host.rs` (new), `lua_audio.rs` | Section 5 item 4, below. JSON response fields read with `get`. |
| `f36e6f9116` | `luau_audio_processor_engine.rs`, `.../safe.rs`, `.../browser_kernel.rs` | See "`catch_unwind` sites". The log ring (8 KiB, 2-byte length, 1 KiB lines) walks its atomics from a position like the shared ring; char truncation is `char_floor` (`(0..=max).rev().find(is_char_boundary)`). Deadlines are `Instant::checked_add` (std on native; the browser `Instant` gained the same method and lost its `Add` impl). Processor array pushes use `(1..).zip`. |

Not one-by-one: no site was replaced by `x.get(i).ok_or(..)?` with a new
error. The few new error texts are all for states that used to panic
(listed under "What a script observes").

## Lint-invisible sites in these files

- `copy_from_slice`: S listed `luau_audio_samples.rs:205`,
  `luau_audio_shared.rs:791,810`, `luau_audio_shared_web.rs:289,380`. All
  five are gone (zip copies, or `extend_from_slice` into an exactly reserved
  `Vec`). One new one, in `host.rs`'s host-absent `call`: the output is
  `output.get_mut(..N)` of the constant's own length N, equal by
  construction. The rest in these files are in `#[cfg(test)]` code.
- Integer division: every non-literal divisor in shipped code is a
  `NonZero*` (`frame_bytes`, `full_record_bytes`, sample rate) or a
  method on a checked value (`div_ceil(chunk_frames.get())`,
  `div_ceil(rate.get())`). Literal divisors left: `/ 4`, `% 4` in the
  region (word and lane of a byte), `% LOG_BYTES` (8192).
- `RefCell` borrows across a call into a VM: none in these files after the
  change. Found and fixed (trivial): the browser kernel held
  `KERNEL.borrow_mut()` across `BrowserKernel::start` and `render`, both of
  which run processor script; both now `try_borrow_mut` (a re-entrant
  request gets "processor kernel is busy"; a re-entrant render renders
  nothing). `RESPONSE` likewise. Checked and clean: promise settlement in
  `lua_audio_samples.rs` (no guard alive at `resolve`/`reject`/
  `set_on_cancel`), `region_exclusive`'s `callback.call` (no guard alive;
  the view's methods borrow only for the copy). Nothing reported for lane E
  from these files.

## `catch_unwind` sites

Both stay. What is around them now cannot panic:

- `safe.rs` render: the argument used to be `&mut out[first..end]` built
  outside the closure from `offset * channels` arithmetic. Now the output
  prefix for `frames * channels` (checked) is taken with `get_mut`, and each
  block is `split_at_mut_checked(count * channels)` off the rest. A short
  output, or `max_frames == 0` with frames to render, fails the processor
  with the existing "audio format exceeds initialized processor format".
  Before, a short output panicked outside the catch on the real-time
  thread, and `max_frames == 0` looped forever (count 0, offset never
  advanced). The failure paths zero only the failed block; the blocks after
  it were zeroed at entry and never written, so the output is the same as
  `out[first..].fill(0)`.
- Inside `process_block`: the length check is `checked_mul`; the hard-block
  span is saturating and divided by a `NonZeroU64` sample rate (a zero rate
  with hard blocks used to panic in `div_ceil`, caught as "audio processor
  panicked"; now "invalid processor sample rate"); a span past the clock's
  range sets no deadline instead of panicking in `Instant + Duration`.
- `luau_audio_processor_engine.rs` initialization: the code in the job
  outside the catch (cancel check, `weak.upgrade`, `lock` (poison-tolerant),
  state CAS, `notify`) has no panic site in this crate. It calls
  `AudioEngine::attach_renderer` in `nuxie-runtime`, which this lane did not
  audit. The `luau-audio-init` loop (`while let Ok(job) = receiver.recv() {
  job() }`) has none either.
- No allocation or lock was added to `process_block` or `render`. The
  render loop allocates nothing; the PCM reader's `read_into_bytes` (called
  from processor script) allocates nothing. `SharedRing::send` now builds
  the hold trailer in a small `Vec` before taking the lock, but only when
  the value carries shared handles, which a processor cannot send
  (`send_from_processor` refuses them first), so the processor path is
  unchanged.

## Section 5 item 4: browser audio host boundary

Boundary module: `crates/nuxie-scripting/src/vm/luau_audio_web/host.rs`
(module path `vm::luau_audio_web::host`). Eight `unsafe` sites, each with
its contract:

| Site | Contract the JS host keeps |
|---|---|
| `unsafe extern "C"` block, wasm module `nuxie_audio`: `host_call`, `now_ms`, `shared_raw` | Signatures match the host's (`createImports` in `tools/luau-audio-web/shared.mjs`); called only through the wrappers below. |
| `call(op, input: &[u8], output: &mut [u8]) -> i32` | The host reads at most `input.len()` bytes and writes at most `output.len()`, during the call only, and keeps no address. |
| `shared(op, owner, id, bytes: &mut [u8], offset, value) -> f64` | Same, for `bytes`. |
| `now() -> f64` | Plain values only. |
| `pub unsafe fn free(pointer, length)` + its `Box::from_raw` | `pointer`, `length` are exactly what `alloc` returned and was given, not freed before, not used after. |
| `pub unsafe fn kernel_request(pointer, length)` + its `slice::from_raw_parts` | `pointer` addresses `length` initialized bytes in this instance, unchanged during the call. |

`alloc(length)` is safe code (it leaks a `Box<[u8]>`); it now reserves with
`try_reserve_exact` and returns null on failure, which `worklet.mjs`
already handles ("audio kernel allocation failed"); before, a failed
allocation aborted the kernel. Without `browser-audio-host` the three
wrappers are ordinary safe functions (the canned "audio needs the browser
audio host" response, `-1.0`, `0.0`). Everything else in the audio code
calls `host::call`/`shared`/`now` through `luau_audio_web::request`,
`raw_shared` and `Instant::now`. `luau_audio_web.rs` re-exports `alloc`,
`free`, `kernel_request` so the kernel's import path is unchanged. The
response serialization `expect` is a fallback response.

At merge: `#![deny(unsafe_code)]` on the crate with `#[allow(unsafe_code)]`
on `mod host;` in `vm/luau_audio_web.rs`.

### `tools/luau-audio-web/src/lib.rs` (the kernel's exports)

Not changed. Its 9 `unsafe` tokens are 5 `#[unsafe(no_mangle)]`
attributes (required for any export in edition 2024; the ratchet pattern
does not count them), 2 `unsafe extern "C" fn` exports
(`nuxie_audio_free`, `nuxie_audio_request`, which receive raw parts from
JS) and the 2 `unsafe` calls into `host::free` / `host::kernel_request`.
They are already a forwarding shim over the boundary; there is nothing
behind them to contain. Removing them would mean defining the
`#[no_mangle]` exports inside `nuxie-scripting` itself, which would make
every wasm artifact that links `nuxie-scripting__publisher` export
`nuxie_audio_*`. Not the same small pattern; left as the second (tools,
not crate) boundary, 4 ratchet-counted sites.

## What a script observes

Unchanged except where a panic or hang became an error:

| Before | After |
|---|---|
| Panic (`expect`) when audio request ids are exhausted (2^64) | Script error "audio request identity exhausted" |
| Panic on missing app-data registries (not reachable after `install`) | Existing errors ("shared audio budget is not installed", "audio processor module '..' is not registered") |
| Short render buffer: panic on the audio thread outside the catch | Processor fails: "audio format exceeds initialized processor format" |
| `max_frames == 0` with frames requested: infinite loop | Same failure text |
| Zero sample rate with hard blocks: caught panic, "audio processor panicked" | "invalid processor sample rate" |
| Browser kernel re-entrant request: `RefCell` panic (wasm trap) | Response error "processor kernel is busy" |
| Kernel `alloc` failure: abort | Null, which JS reports |
| Browser response buffer growth failure: abort | "cannot allocate browser audio response" |

The region byte read now loads each word once for its four bytes instead
of once per byte; under concurrent processor atomics this can only make a
read more consistent, not less.

## Checks

| Check | Result | Load average (1 min) start → end |
|---|---|---|
| Panic-lint ratchet, native row | 321, my files 0, PASS | 8.81 → 20.71 (baseline run); after-run ~17 |
| Panic-lint ratchet, wasm row | 320, my files 0, PASS | ~17 |
| `python3 tools/bazel/runtime.py test -p nuxie-scripting` | 39 / 39 targets pass | 23.25 → 11.04 |
| `make luau-vm-diff` | 409 runs, 0 disagreements, every admitted opcode reached | 45.03 → 40.02 |
| `make luau-conformance` | 37 / 37 required OK, 0 panics, PASS | 40.02 → 21.50 |
| `tools/luau-audio-web/pool.py build --release` | builds before and after; imports and exports identical (`wasm-objdump`) | 26.25 (after build), 40.91 → 34.85 (before build) |
| `codex review --base levi/luaur-client-safe` | round 1: no findings | |

Kernel sizes (release, pool build):

| Artifact | before (`9eb238d58f`) | after |
|---|---:|---:|
| `nuxie_luau_audio_kernel.wasm` | 976,600 | 970,331 (−6,269) |
| `nuxie_luau_audio_probe.wasm` | 16,666,420 | 16,661,303 (−5,117) |

`python3 tools/bazel/runtime.py test -p nuxie --features scripting,test-support`
was not run: nothing outside the audio files changed.

## Not verified

- The browser itself: no AudioWorklet run. `worklet.test.mjs` (node, with
  stubbed exports) passed in codex's review run; the kernel was built but
  not loaded.
- Real-time behaviour on a device: no audio thread timing was measured;
  the claim that the render path gained no allocation or lock is from
  reading the code.
- `AudioEngine::attach_renderer` (`nuxie-runtime`) was not audited for
  panics.
- No test was added for the new failure paths (short render buffer,
  `max_frames == 0`, kernel re-entrancy); they are covered by reading only.

## Where the design was wrong or thin

- "One checked byte cursor for the shared-memory codecs and ring layouts":
  the codec already had a cursor (lane A's `Reader::take`); it needed only
  fixed-array reads. The ring is not a cursor problem: it is wrapped
  storage, and the shape that removes the arithmetic is iteration from a
  position (`split_at` + `chain`), shared by the shared ring and the log
  ring.
- "A validated layout is a type built by validation": the PCM descriptor's
  fields are public and also built from a shared value, so a validated type
  would have meant changing every reader of `channels`. A `frame_bytes()`
  accessor returning `NonZeroUsize` (the one place a zero matters) did the
  job with less churn.
- `serde_json::Value` indexing is a lint site the survey counted under
  `indexing_slicing` without noting that reads cannot panic but writes can;
  the writes were the real panics.
- The survey's "about 30 wasm-only" was 121 − (155 − 66 native-only) = 32
  here, consistent.
- The kernel's export shim (`tools/luau-audio-web/src/lib.rs`) cannot
  join the crate's boundary module without exporting kernel symbols from
  every publisher artifact; the design should name it as a separate,
  tools-side boundary.
