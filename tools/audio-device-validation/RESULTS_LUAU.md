# Luau audio processor validation — 2026-10-26

This change implements the plan in `docs/luau-audio-processors-plan.md` on top of
`b4e643fc8d1e70fbdba03ffb43aeda8aeb63007d`. It adds a sine oscillator and the
shared Luau creation/control/status/meter interface without Wasm or new dependencies.

Independent review one checked API/behavior and found a fixture mistake: an empty
mixer after teardown returns zero frames with silence, not a full block. The test
was corrected without changing production behavior. Independent review two checked
Rust atomics, bounded storage, callback ownership, and script/VM lifetimes. Both
passed after corrections. Follow-up reviews covered portability fixes and test-host
linking.

## Pool checks

- Audio unit suite: **41 passed**, including seven new processor tests. Job
  `1acca5a3-0406-404f-a83a-62b0a2932081` also passed the initial real-Luau probe.
- Focused Luau control tests: **5 passed**, job
  `d8bac408-3657-4202-8cb5-270f8ec2d977`.
- Final scripting unit suite: **376 passed, 1 failed, 1 ignored**. Final real-Luau
  probe passed in the same job, `3135db24-e67f-4f54-8ced-0aa07ffd4670`.
- The sole scripting failure is
  `gpu_canvas::ore::tests::shared_script_guards_preserve_parser_order_and_unknown_device_limits`.
  It asserts an anisotropic sampler can omit linear filters, while the existing
  guard rejects it. The identical failure was reproduced on unchanged merged main
  in job `5a4515d3-f618-4389-ad53-398158f56106`. This is baseline, not a new audio
  regression; neither its assertion nor the GPU guard was changed.
- Focused Rust formatting, whitespace checks, and generated Bazel metadata checks
  passed.

## Device execution

The fixture uses the shipping ScriptVm and actual Luau API. It schedules a tone,
changes frequency/gain with a ramp, fades/stops it, starts another owned tone, and
then destroys the instance while its Lua table and userdata remain reachable.
Final output must be silent. Binary-string and percent-interpolation smoke checks
also execute before audio initialization.

| Target | Outcome |
| --- | --- |
| Mac built-in output, CPAL, 48 kHz stereo | Passed at requested/actual blocks 256 and 1024. All phases, final silence, and retained-handle instance teardown passed. |
| Android arm64 API 36 emulator, AudioTrack, 48 kHz stereo | Two fresh launches passed with 256-frame mixer blocks. Each wrote/played 192,000 frames; zero underruns at the final-write sample and one after final drain. |
| Wireless Bat Phone, physical iPhone | Passed on iOS 26.6.2, speaker output, 48 kHz stereo and 256-frame callbacks. All 1,175 callbacks had valid samples/layout; scheduled stop, retained-handle instance teardown, and final silence passed. |

Final Mac build job: `7c825694-b953-4cfe-b418-5f6d642fd42a`.
Android library job: `56aeea93-68fc-4658-9b64-5cce86096133`; APK job:
`9bae8067-bd24-4e16-8442-cd507f1a2ba6`.
iOS build job: `260635b4-aa99-4698-9bcb-380983a8127f`.
Measured records are `results/luau-macos.json`,
`results/luau-android-run{1,2}.json`, and `results/luau-ios.json`. Android's test app
was removed and the emulator stopped. The iPhone test app was removed after its
successful run. All three requested device targets have completed validation.

## Blockers fixed

Android exposed hardcoded signed C-character types in the vendored Luau AST and
compiler, while this target uses unsigned C char. Eight files now use the existing
platform `c_char` contract consistently, preserving bytes, explicit string lengths,
ASCII comparisons, and ABI layout. These are portability fixes, not parser behavior
changes. The iOS Luau host additionally links ImageIO for the existing image codec.
Android test packaging accepts pool-normalized timestamps by clamping ZIP metadata
to its supported range; library contents are unchanged.

## Limits

These are standalone native hosts, not full SDK/view lifecycle tests. Android was
an emulator, not physical hardware. The runs verify digital output paths, controls,
and normal teardown; they do not measure acoustic fidelity, long-duration stability,
interruptions, Bluetooth routing, or background behavior. The new oscillator callback
uses fixed storage and atomics, but existing mixer allocations/locks remain baseline;
these results are not a hard realtime guarantee. No SDK releases or consumer changes
were made.
