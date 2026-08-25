# `lua_scripted_context.cpp` paired audit

Upstream owner: `src/lua/lua_scripted_context.cpp` at
`4ac7b32798da0482e441ef09304dc3b480ed3ee5`.

Rust owners:

- `crates/nuxie-scripting/src/vm/view_model.rs` owns the Context userdata,
  DataContext projection, lifetime checks, and method dispatch.
- `crates/nuxie-scripting/src/vm.rs` owns the VM lifetime, imported file
  registries, and the Context-to-ScriptHost update request.
- `crates/nuxie-scripting/src/vm/lua_image.rs`, `lua_blob.rs`, `lua_audio.rs`,
  and `lua_image_decode.rs` own their corresponding retained asset userdata.
- `crates/nuxie-scripting/src/gpu_canvas.rs` owns GPU-canvas and shader
  occurrences.
- `crates/nuxie-runtime/src/scripting.rs` owns file-scoped asset identities.
- `crates/nuxie/src/lib.rs` installs the importing file and routes component
  owners through the runtime ScriptHost.

Verdict: adapted and behaviorally equivalent under the fixed Rust-native
renderer and scripting boundary.

The paired audit covered `viewModel`, `rootViewModel`, `dataContext`,
`markNeedsUpdate`, `image`, `blob`, `decodeImage`, `audio`, `canvas`,
`gpuCanvas`, `features`, and `shader`; disposed-context errors; Context
registration and invalid-method dispatch; the conservative 19-field headless
feature table; independent GPU-canvas descriptors; and fresh-VM render-state
ownership. The Context update request is drained by `LuaScriptInstance` into
the active `ScriptHost`, then routed to the component dirt owner with the
pinned update-phase recursion guard.

The audit found one real ownership error. Rust had implemented
`context:image()` through the optional data-context `ScriptViewModel`. Pinned
C++ instead walks `scriptAsset()->file()->assets()`, so the lookup works even
when the scripted object has no ViewModel. The correction installs a
file-scoped image identity catalog in the VM and makes Context consult it
directly. The converted `walle.riv` case now proves lookup and retained decoded
image dimensions with a null ViewModel, matching the upstream test shape.

Six converted upstream cases had remained disabled because their assertions
encoded adapter placeholders rather than the C++ behavior. Both
`markNeedsUpdate` cases now inspect the live request; the image case installs
the same file-owned decoded-image state as C++ `SerializingFactory`; invalid
methods assert the Rust-native diagnostic names the requested method; and the
C++ null ORE/render pointers are represented by two fresh VMs owning distinct
empty Rust GPU/render state. All 17 private binding-owner cases are active and
green. The three remaining ignored cases among the five public facade silvers
are denominators for ScriptInput hydration or shared raw-path rendering, not
gaps in this Context owner.

Canvas 2D is the one approved adaptation. The Rust renderer boundary has no
C++ `RenderContext`/ORE Canvas userdata, so the method remains present and
fails deterministically with
`unsupported: scripted-context-canvas binding is unavailable`. This is counted
as exact parity only under that explicit immutable adaptation, never as a
literal implementation of the C++ Canvas surface.
