# Luau-authored audio validation — 2026-10-26

Candidate implementation: `b2b6d40039`, based on merged main `23145b07a0`.
This replaces the fixed Rust oscillator API with isolated Luau processors that
initialize off the audio callback and run sample-generation code directly on it.
Ordinary Audio.play remains unchanged. No Wasm engine or new production dependency
is added. Subsequent Rust2024 formatting changes only import ordering and test
formatting; both independent reviewers verified no production expression or test
assertion changed, so the candidate's executable validation remains applicable.

## Reviews

Independent source pass compared the native processor and Ring/Snapshot contract
against public upstream `4252d730`. It found and verified corrections for async
wake/poll integration, failure delivery and retained-handle ownership, excessive
ring storage, snapshot publisher blocking, and stale receiveInto fields. That
scoped source review passes at the candidate commit.

The separate independent Rust safety/integration review also passes at the same
commit. It checked exclusive VM ownership, stable allocator/callback pointers,
rooted sample buffers, GC interrupts, sticky failures, threading/teardown races,
shared publication, event retirement and precompiled registration. Device results
below must not be inferred from the earlier fixed-oscillator qualification.

## Pool validation

- 31 authored-audio tests passed across bindings, isolated VM execution and shared
  transport, including thread migration, caught deadline/OOM errors, precompiled
  dependencies, ScriptAsset registration, teardown and paused snapshot publication.
- Full scripting suite: 402 passed, 1 failed, 1 ignored. The only failure remains
  the previously reproduced baseline GPU sampler guard mismatch:
  `shared_script_guards_preserve_parser_order_and_unknown_device_limits`.
- The actual-script oscillator probe passed in the same pool job:
  `2772a1c3-06a6-4953-9aa0-6369b5684e8b`.
- All 34 ordinary audio unit tests passed in job
  `ecd6462c-8d77-44fc-8856-c5276deca0ad`.
- Generated Bazel metadata and whitespace checks passed. Compilation/tests ran on
  the build pool; no local compilation fallback was used.

## Device execution

- Mac built-in CPAL output passed at actual/requested256 and1024-frame blocks,
 48kHz stereo. Pool build `428437e0-725d-4d7f-85ce-f3fb6cae45b6`.
- Physical wireless iPhone on iOS26.6.2 passed,48kHz stereo,1,169 callbacks of
 256frames, zero invalid samples/layout errors. Pool build
 `2d492909-74ae-45d6-8b75-8d09fe7861f5`. Test app removed.
- Both targets passed custom controls/snapshots, processor stop, retained-handle
 instance teardown and final silence. Evidence: `results/luau-authored-macos.json`
 and `results/luau-authored-ios.json`.
- Android emulator debug qualification failed on cold restart: the processor
  correctly reported `audio processor execution budget exceeded`. The first run
  passed, but the second stopped at engine frame 22528. Diagnostic pool build
  `0fc55815-d680-4d0b-964f-eb1de41029aa`; both results are retained in
  `results/luau-authored-android-debug.json`. The earlier debug build also failed
  its second run. These debug failures remain failures; the optimized result below
  does not retroactively make them pass.
- The diagnostic host now preserves the first control failure in its JSON report.
  An optional `pool.py --release` selects the existing repository release profile.
  Both changes passed both review followups; neither changes deadlines, production
  processor logic or validation assertions.

- Android emulator release qualification passed twice, including a cold restart,
  using the same execution budgets, assertions and emulator settings. Both runs
  passed controls, snapshots, stop, retained-handle teardown and final silence;
  192,000 frames were written and played at 48 kHz stereo, with zero invalid
  samples and zero underruns before drain (one reported after drain). Build job
  `2d9b14a5-44b2-4dba-ade6-dfc1ac842c43`; packaging job
  `15fa9f0e-492d-445b-bf6e-673cc2aba52d`. Evidence and artifact hashes:
  `results/luau-authored-android-release.json`. Mac/iOS used debug builds; Android
  qualification applies to the tested optimized build.

## Coverage boundary

This evidence covers native processors with reusable Ring/Snapshot communication.
It does not establish full upstream audio capability parity: processor Blob/Region
APIs, the explicit upstream offline format/clock API, and browser AudioWorklet
integration are not implemented here. Module names and dynamic Luau records replace
upstream's compiled entry references and generated typed shared-record schemas.

Host timing budgets are checked at VM interrupts and after native processing/GC.
This is not an operating-system hard realtime guarantee. The device hosts verify
digital output paths, controls and teardown, not acoustic fidelity, long-duration
stability, Bluetooth/background behavior or full SDK/view integration. Android
qualification uses an emulator. No consumer migration or SDK release is included.
