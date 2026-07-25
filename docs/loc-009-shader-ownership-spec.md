# LOC-009: pinned shader lookup and backend-module ownership

## Objective

Restore the owners and lookup behavior in pinned C++ oracle
`d788e8ec6e8b598526607d6a1e8818e8b637b60c`.

1. File registration attempts the backend-neutral `ShaderAsset` decode and
   indexes the final last-descriptor-wins RSTB ranges once. The public C++ file
   importer ignores the decoder boolean and still succeeds, so Rust retains an
   invalid/empty state when neutral decoding fails or contents are absent.
   A foldered file-owned asset has one retained owner shared by its bare and
   `folderPath/name` lookup aliases.
2. Every successful `context:shader(name)` call creates one fresh
   `ScriptedShader` occurrence and one whole-module backend `ShaderModule`.
   All entries derived from that userdata share the module. A second same-name
   lookup creates another occurrence and module. Failed lookup pops the failed
   userdata, returns zero Lua values (therefore nil in assignment), and retains
   neither an occurrence nor a module.
3. Explicit vertex and fragment stages resolve independently. An omitted
   fragment stage falls back to the vertex shader userdata and therefore shares
   its occurrence/module.

An unused target-1-only or malformed shader asset must not poison unrelated
script boot. Requesting that exact name returns nil and execution continues.
WebGPU accepts only authored whole-module target-0 WGSL with mandatory
target-16 `BindingMap`; target-1 GLSL translation and fallback remain deleted.

## Pinned C++ evidence

- `src/assets/shader_asset.cpp:6-113`: `ShaderAsset::decode` validates
  SignedContent, RSTB v4, sections, last-descriptor-wins indexing, and every
  final descriptor range without selecting the active backend.
- `src/importers/file_asset_importer.cpp:28-50`: absent or asynchronously
  unavailable `FileAssetContents` returns `StatusCode::Ok`;
  `FileAssetImporter::resolve` also ignores the decoder boolean. Public file
  registration therefore does not fail for contentless or malformed neutral
  input.
- `src/lua/lua_scripted_context.cpp:516-539`: lookup scans one file-owned
  `ShaderAsset` and accepts either its bare `name` or its
  `folderPath/name`. Both aliases therefore resolve the same retained decode
  owner.
- `src/lua/lua_scripted_context.cpp:511-556`: the successful lookup seam
  constructs a fresh `ScriptedShader` at line 547. Lines 547-556 pop a
  `ScriptedShader` that cannot build entries and return zero Lua values, so
  `context:shader(name)` produces nil without raising a lookup exception.
- `src/lua/renderer/lua_gpu.cpp:519-561,629-660`: WebGPU selects target 0 and
  target 16 at lookup and creates one backend module for the whole authored
  module; every entry shares it.
- `src/lua/renderer/lua_gpu.cpp:1909-1945`: explicit vertex and fragment stage
  descriptors resolve independently and may use different shader userdata.
- `src/lua/renderer/lua_gpu.cpp:2079-2094`: omitted fragment falls back to the
  vertex userdata.
- `renderer/include/rive/renderer/ore/ore_pipeline.hpp:49-60,77-88` and
  `src/lua/renderer/lua_gpu.cpp:2164-2200`: each shader module owns target 16,
  but pipeline/auto-layout uses the vertex module's binding map whenever a
  vertex stage exists. It does not union the vertex and fragment maps.
- `src/lua/renderer/lua_gpu.cpp:1510-1562` and
  `renderer/include/rive/renderer/ore/ore_types.hpp:561-565`: target 16 records
  binding identity, resource kind, and stage visibility, but no uniform byte
  size.

## Public behavior and deep-module seam

Keep the existing public registration interface:

```rust
ScriptVm::register_gpu_canvas_shader_asset(name, payload)
```

Add only the narrow safe multi-alias interface needed by the file facade:

```rust
ScriptVm::register_gpu_canvas_shader_asset_aliases(aliases, payload)
```

The module behind that interface must:

1. preserve the existing stronger Rust duplicate-first/reject policy
   atomically, so a rejected duplicate never replaces the first source;
2. preflight every multi-alias registration, create one file-owned state, and
   bind every accepted alias to that owner without partial mutation;
3. preserve facade file order independently per alias: the first asset keeps a
   colliding alias, while a later asset is still registered once under any
   unique aliases it has;
4. decode/index the backend-neutral container once per reachable file-owned
   asset at registration, regardless of alias count;
5. treat absent contents as an empty retained-invalid state rather than a file
   preparation error, without weakening `ScriptAsset` payload requirements;
6. retain invalid/empty state when neutral decode/index validation fails;
7. select/decode target 0 and target 16 only when the exact name is requested;
8. return zero Lua values for a missing name, retained neutral-decode failure,
   target incompatibility, malformed selected backend data, or backend module
   creation failure, without aborting script execution;
9. allocate one fresh logical occurrence and physical module for each
   successful lookup while the renderer factory context is active;
10. clone that opaque occurrence handle, not name/content identity, through
   stage descriptors, pipelines, and completed passes until backend pipeline
   materialization;
11. retain explicit vertex and fragment occurrences separately, while the
   omitted-fragment combined form shares one `Arc`;
12. preserve declaration-order defaults, named entry selection, target-16
   validation, and canonical target-0/16 behavior.

Caching immutable CPU target-0/16 decoding on the file-owned asset is allowed.
Caching or interning a physical module by name, source, pipeline key, or global
content identity is forbidden.

The renderer interface exposes an opaque `RenderGpuCanvasShader` sidecar and a
factory creation method. A WGPU sidecar owns the parsed WGSL, target-16
requirements, backend module, factory/device domain, and a per-occurrence ID.
The ordered vertex/fragment occurrence IDs participate in the renderer pipeline
cache key. The cache key must not retain `Arc` handles or modules. At the pinned
native seam, `PipelineDesc` carries the ordered VS/FS raw module pointers into
`CreateRenderPipeline`; `PipelineWGPU` retains compiled pipeline/layout state
rather than `ShaderModule` rcps
(`renderer/include/rive/renderer/ore/ore_pipeline.hpp:49-60,77-88`;
`renderer/src/ore/wgpu/ore_context_wgpu.cpp:728-913`). Rust `Arc` retention is
required only through delayed materialization; after native pipeline creation,
the compiled state outlives the module handles.

Each source's WGSL and target-16 map are validated separately. During pipeline
materialization Rust deliberately preserves the pinned native stale assumption:
the vertex occurrence's target-16 requirements are authoritative for layout.
Rust must not union the fragment map. Fragment resources incompatible with the
vertex-authoritative binding existence, kind, or stage visibility fail closed.
Target 16 has no uniform-size field, so Rust's Naga preflight size for an
explicit same binding is the maximum of the vertex and fragment WGSL
requirements while layout identity and visibility remain vertex-authoritative.
The explicit Rust factory/device-domain check is an early deterministic mirror
of Dawn device `ValidateObject`.

Default `Factory` implementations remain fail-closed. `RecordingFactory` and
`NullFactory` do not gain GPU shader support or GPU domain fields.
`WgpuFactory` implements shader creation and image materialization.
`BrowserFactory` forwards both methods.

## TDD slices

1. Register a valid target-1-only asset, boot and execute an unrelated public
   script, and prove no module is created.
2. Register malformed neutral data successfully, boot unrelated code, then
   request it and prove lookup returns nil, the script continues, and no module
   is created.
3. Request missing or target-1-only data and prove lookup returns zero Lua
   values (nil in assignment), execution continues, and no module is created.
4. Force backend module creation to fail and prove lookup returns nil,
   execution continues, and no module is retained.
5. Look up one shader without constructing a pipeline and prove one module is
   created.
6. Feed one occurrence through two pipeline keys and prove one module creation
   and stable occurrence identity.
7. Perform two same-name lookups and prove two module creations and distinct
   identities.
8. Use differently named and sourced explicit vertex/fragment shaders and
   prove both exact handles reach materialization; prove omitted fragment passes
   one shared handle.
9. Reject cross-factory/device-domain use.
10. Prove the vertex target-16 map remains authoritative for binding identity
    and visibility, while explicit same-binding Naga preflight uses the larger
    vertex/fragment uniform size.
11. Pass a lookup nil to `GPUPipeline` and prove construction still fails
    closed.
12. Keep canonical target-0/16 renderer behavior unchanged.
13. Register one foldered file asset under bare and qualified aliases, prove
    both lookups create fresh backend occurrences, and prove the aliases share
    one retained owner.
14. Prove facade alias collisions are first-file-wins independently per alias.
15. Prove an unused contentless shader boots and a requested contentless shader
    returns nil/continues, both with zero backend module creations.

Tests assert public behavior or renderer semantics, not source text.

## Allowed scope

Production/evidence files:

- `crates/nuxie-render-api/src/lib.rs`
- `crates/nuxie-renderer/src/{gpu_canvas.rs,lib.rs,browser.rs}`
- `crates/nuxie-scripting/src/{gpu_canvas.rs,shader_asset.rs,vm.rs}`
- `crates/nuxie-scripting/src/vm/renderer.rs` (visibility only)
- `crates/nuxie-scripting` tests and Cargo metadata
- `crates/nuxie/src/lib.rs`
- `crates/nuxie/tests/imported_gpu_canvas.rs`
- `crates/nux-capi/src/size_report_roots.rs`
- `tools/browser-renderer-smoke/src/lib.rs`
- `tools/size-report-renderer-roots.txt`
- `docs/loc-009-shader-ownership-spec.md`
- `Cargo.lock`

The approved corrective lease includes the facade registration path, its
focused imported-GPU-canvas fixture/tests, and the browser renderer smoke
forwarder listed above. Do not touch `crates/nuxie-runtime`,
`crates/nuxie-graph`, other facade tests or fixtures, FL docs/gates, budgets,
scripts, or parity fences. Size evidence may only add the new `Factory` root
and increment its root count.

## Acceptance

Acceptance requires nil/no-value lookup semantics, continued script execution,
and zero module creations for missing, malformed, target-incompatible, and
backend-creation-failed lookups. It also requires the same-owner alias and lazy
contentless-import behavior above. Lookup exceptions do not satisfy LOC-009.
GPUPipeline must continue to reject nil when a caller uses it as a module. The
repository closeout pixel, full-renderer, required-browser, and size floors
remain mandatory.

```bash
cargo fmt --all -- --check
cargo test -p nuxie-scripting
cargo test -p nuxie-renderer --lib
cargo test -p nuxie --features scripting --test imported_gpu_canvas
cargo check --workspace
cargo check -p nuxie-renderer --target wasm32-unknown-unknown
make renderer-golden
make browser-webgpu-only-check
make size-report
git diff --check
```

If full renderer tests require unavailable GPU hardware, run and report focused
exact internal tests, but they do not replace or waive any required floor. Leave
the corrected files dirty; do not commit or push.
