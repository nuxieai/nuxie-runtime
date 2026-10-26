# Luau browser audio

`host.mjs` supplies the browser platform boundary for the shipping Luau `Audio`
API. `worklet.mjs` executes the same Rust/Luau processor kernel as native audio,
inside private wasm32-unknown-unknown instances. Only explicitly shared audio
objects use SharedArrayBuffer; runtime and Luau heaps are never shared.

Initialize `createAudioHost(kernelURL, options)`, supply its `imports(memoryGetter)`
when instantiating the main Wasm runtime, then call `host.attach(exports)` with
an exported `nuxie_audio_notify(id)` forwarding to
`nuxie_scripting::vm::luau_audio_web::notify(id)`. The notification wakes idle
script runtimes for asynchronous state and log delivery. `host.resume()` belongs
in the embedding application's user-activation handler. The browser requires a
secure context and cross-origin isolation. An externally supplied AudioContext
remains the caller's property; `dispose()` closes only an internally owned one.

The kernel artifact exports allocation/request/render functions. Processor
initialization runs in the worklet message handler; sample generation runs in
`AudioWorkletProcessor.process`. This Wasm artifact contains our Rust Luau
interpreter, not an upstream Wasm guest-script execution engine.

## Qualification

Build and test on the configured pool:

```sh
python3 tools/luau-audio-web/pool.py test
python3 tools/luau-audio-web/pool.py build --release
python3 tools/luau-audio-web/pool.py package --probe /path/to/pool/probe.wasm --kernel /path/to/pool/kernel.wasm
```

`package` invokes the pinned wasm-bindgen tool on the pool, then assembles the
already-built artifacts and adds the explicit audio-import initialization seam.
The artifact paths are printed by `build`. Local compilation is not used.

Run the packaged qualification with an installed Playwright module:

```sh
PLAYWRIGHT_PATH=/path/to/playwright/index.mjs node tools/luau-audio-web/test.mjs tools/luau-audio-web/pkg /tmp/luau-audio-browser-result.json
```

The host compares captured browser PCM exactly with an offline fixture, exercises
Ring controls, Blob reads, Region atomics, processor failure isolation, and
stop/instance/VM teardown while retaining handles. This page is an integration
test, not an editor migration or SDK release.

## Current validation state

The independent source-equivalence pass passed after correcting retired shared
handle properties. The separate Rust/browser integration pass also passed after correcting closed
owner metadata reclamation. Pool Node transport/lifecycle tests pass 14/14,
including retained-handle and repeated-owner-retirement regressions. Final pool
run: `6fc828bd-5328-4cc8-82e6-e98bfc1ca825`.
The initial browser build stopped before compilation because the new direct
serde_json edge requires dependency lockfile refresh; full browser execution
and native/device validation have not passed for this change yet.
