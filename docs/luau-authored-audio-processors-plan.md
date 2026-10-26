# Luau-authored audio processors

This replaces the fixed Rust-oscillator scripting interface with author-defined
Luau DSP following upstream Wasm AudioProcessor semantics. Wasm remains excluded. The
previous docs/luau-audio-processors-plan.md records the already shipped iteration;
its prohibition on scripted sample generation is superseded by this request.

## Source reference

The public processor contract is read from upstream commit
`4252d73093af032d8c59b01ab3a7aa40d3c24add`, including
`include/rive/wasm/wasm_processors.hpp`, `src/wasm/wasm_processors.cpp`, lane policy,
shared transport and `tests/web_scripting/fixture/audio/main.as`. The processor
implementation is unchanged through the locally fetched upstream checkpoint
`0f4fa96fc5e9fe4bc41a398bff8275e59cfb86e2`.

## Intended contract

- Authors implement processor initialization and block processing in Luau.
  Initialization receives sample rate, maximum block frames, channel count and
  explicitly transferred input data. Processing receives an output sample buffer,
  frame count and absolute engine start frame.
- Each processor has an isolated VM. Never pass main-VM closures, Lua handles,
  artboard objects or arbitrary host userdata across threads. The module/entry
  packaging must support precompiled production assets without requiring a runtime
  source compiler. Do not pretend a closure with captured state can be transferred.
- Provide bounded messages from the main script and custom snapshots back. Define
  the supported data schema, ownership, overflow and snapshot consistency explicitly.
- Expose starting/running/stopped/failed state, failure details and stop. Stop is
  terminal and waits for an active render; it does not supply a host-authored fade. Preserve
  instance/VM ownership cleanup, including retained handles and failed initialization.
- A failed processor must produce silence without stopping other processors.
  Establish memory, initialization and block-execution limits. Interrupt safepoint
  quotas alone do not establish a hard wall-clock bound for native builtins or GC.
- Replace the hardcoded oscillator example with an oscillator authored in Luau.
  Ordinary Audio.play asset playback remains intact. No compatibility shim is
  required for the newly introduced fixed-oscillator scripting API.
- Preserve host-owned audio format and configurable blocks. No effects graph,
  new scripting engine, SDK release or broad consumer migration is implied.

## Approved execution model

Use direct audio-callback execution in isolated Luau processor VMs, matching
upstream's architecture. Initialize off the audio callback. No buffered-worker
alternative. Upstream lifecycle, message/snapshot behavior, limits and failure
handling are the specification; differences must be justified by concrete Luau
or Rust requirements. Do not invent restrictions such as forbidding all sample
callback allocations or GC: upstream runs collection as part of processing.

The current luaur-rt defaults to non-Send Lua handles. Its optional send feature
changes callback bounds globally through dependency feature unification. Do not
turn it on globally or add an unsafe Send wrapper around the existing main VM as
a shortcut. Audit exclusive processor ownership and initialization/destruction
threads before choosing a transport mechanism.

Native processing uses a dedicated serial initialization queue and per-processor
VM ownership. Default hardBlocks is two, first-block floor100ms and later floor1ms;
zero intentionally disables the deadline for host-controlled offline use. Module
initialization has a separate1000ms budget. Processor memory inherits the existing
host Luau VM ceiling. Shared objects use the upstream512MiB per-owner budget.

## Delivery phases

1. Inspect upstream processor lifecycle, message/snapshot semantics and failure
   budgets; apply the approved direct execution model and specify the author-facing module API.
2. Implement isolated processor execution and bounded communication, then bindings
   and a Luau-authored oscillator. Keep translation/implementation separate from
   the reviews. Use existing pool builds; no local compile fallback.
3. Freeze changes and run independent API/lifecycle review, followed by a separate
   Rust ownership, threading, resource-limit and integration review. Correct and
   rereview findings without weakening tests.
4. Validate actual Luau DSP: sample output, messages/snapshots, stop, retained-handle
   teardown, initialization failures, process exceptions, runaway code, memory
   exhaustion and queue pressure. Prove one failed processor does not mute another.
5. Run the existing Mac, physical iPhone and Android-emulator hosts with the real
   processor implementation. Report unavailable browser coverage separately; do
   not claim an AudioWorklet integration without implementing and validating it.
6. Save evidence, open a narrow PR and merge only after the required validation.

Do not load implement/TDD skills or expand this into migration ledgers.

## Completion pass: shared data, offline rendering and browser audio

Authorized 2026-10-26: implement all three remaining upstream capabilities with
Luau. Source authority remains the public upstream tree pinned above. Translate
shared Blob/Region semantics, offline format/frame clock, and AudioWorklet lane
execution. Region locking is forbidden on processor lanes; preserve permitted
atomics and immutable Blob reads. Preserve owner budgets and lifetime semantics.

Implement disjoint owners in parallel, then freeze implementation before the
independent source pass and subsequent Rust/browser integration pass. Browser
processors execute on the AudioWorklet thread, initialize off that callback, and
use wasm32-unknown-unknown Rust/Luau code rather than introducing Wasm guest
execution or Emscripten. Do not replace direct processing with main-thread or
buffered-worker DSP. Offline rendering must expose the upstream format and clock
contract, without an audio device or realtime block deadline.

Validate focused shared-data/ownership and offline tests on the pool, compare
actual Luau sample output across native offline and browser processing, exercise
browser controls/failures/stop/teardown, and rerun applicable Mac/iPhone/Android
qualification. Keep failures and platform limitations explicit. Land only after
both reviews and applicable validation; no compatibility fallback, assertion
weakening, SDK release, or unrelated consumer migration.

### Shared-data Luau surface

`Audio.Blob.from(bufferOrString)` copies immutable bytes. `blob.size`,
`blob:read(offset, outputBuffer)` (returns copied count), and `blob:toBuffer()`
expose reads. Reading past the end returns zero; partial reads leave the unused
output suffix unchanged. Empty blobs are valid.

`Audio.Region.create(byteLength)` creates zeroed shared storage. `region.size`
and `region.poisoned` are readable fields. `atomicLoad(offset)`,
`atomicStore(offset, value)`, `atomicAdd(offset, value)`, and
`atomicCas(offset, expected, desired)` require aligned, in-bounds U32 words.
Store returns zero; add and CAS return the observed previous word. Arithmetic
wraps at U32 width. These operations are sequentially consistent.

Main scripts use `region:tryExclusive(callback, timeoutSeconds)`. The timeout
is required and expressed in seconds, matching the public upstream host
primitive; zero makes one immediate acquisition attempt. The callback receives
a scoped view with `read(offset, outputBuffer)`, `write(offset, inputBuffer)`,
`getU32(offset)`, and `setU32(offset, value)`. Byte operations require the entire
range to fit. The view expires when the callback returns. Nested acquisitions
are rejected. A callback error poisons and unlocks the region; poison remains
observable and does not prevent later acquisition or atomic access.

This is an explicit Luau language adaptation. We do not claim equivalence to
private AssemblyScript convenience methods or guess their default timeouts or
serialization helpers. The main Luau host has no wall-clock call deadline to
clamp acquisition waits; it honors the explicit timeout. Processor lanes cannot
create shared objects or acquire Region locks. They can read immutable blobs
and perform Region atomics. Shared handles reach processors through initial
`Audio.start` input; processor Ring/Snapshot transfers cannot acquire or publish
shared-object holds. Main-lane Ring/Snapshot messages can carry these handles.

Browser main-script `Audio.sampleRate()`, `Audio.timeFrame()`, and `Audio.time()`
query the AudioWorklet host's sample rate and shared playback frame counter.
Offline host format/frame configuration takes precedence. Browser processor
startup does not instantiate a native audio engine. The completion-pass reviews and device/browser qualification below cover these
contracts.

### Completion-pass checkpoint

Implementation is written for all three capabilities. Independent source review
passed after correcting retired Blob/Region properties; independent Rust/browser
integration review passed after correcting browser owner metadata reclamation.
Pool transport/lifecycle tests passed 14/14; the final offline probe passed in
`bf6dd648-1b9f-473f-9953-4d7dcf3fb3a8`. Native scripting tests passed 411 cases,
with one known unrelated GPU sampler baseline failure and one ignored case.

Dependency metadata was refreshed locally with explicit user approval, without
local compilation/tests or crate version changes. Pool-only browser builds
required a narrow rules_rust executable-copy patch and the existing browser
feature graphs; these corrections received both reviews. A qualification loader
API mistake was caught by pool compilation and corrected to the existing trusted
bytecode loader, then rereviewed and tested. Assertions were not weakened.

Wireless iPhone qualification passed in pool build
`da898f40-c187-422e-a52b-82c930b6abaa`: 1,178 callbacks, 48 kHz stereo, 256-frame
blocks, Blob/Region controls, stop and teardown, and final silence. Mac release
qualification passed at 256/1024-frame callbacks in build
`690943b7-4249-4971-9ced-c021ba4289a1`. Android release qualification passed twice,
including cold restart, in build `48feb1f0-25a8-4fd6-97b6-038c22d84dcc`.

Browser release build `1f440a27-e06e-4a27-90c7-cc77316bbc4e`, packaged by
`9b3242ae-52af-4f86-a310-472356e2c46a`, passed all nine actual Chromium PCM/control/
failure/teardown stages. Qualification found that private worklet VMs needed the
same pinned bytecode flag profile initialized by Lua::new; the correction passed
both reviews and the full browser qualification. See the completion results in
`tools/audio-device-validation/RESULTS_LUAU_AUTHORED.md`. No SDK release or editor
consumer migration is included.
