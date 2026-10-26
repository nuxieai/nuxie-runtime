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
