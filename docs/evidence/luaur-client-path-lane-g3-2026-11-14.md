# Luau client path, lane G3: host memory the budget did not see (2026-11-14)

Design: `docs/luaur-client-path-safe.md`, stage 1 (`crates/nuxie-scripting`).
Branch `claude/luaur-lane-g3-budget-2cee9f`, base `32db50aba8`
(`levi/luaur-client-stage1c`). Not pushed.

The lane closes places where a script makes the host hold memory that the
per-VM `HostMemoryBudget` (64 MiB by default) does not see. It then makes the
crate deny `unsafe_code`.

## Commits

| Commit | Concern |
|---|---|
| `4b169c4b70` | A font assigned to a view-model font property keeps its decode charge |
| `8036891203` | A `DataValue` string is checked before it is copied (new facade `LuaString::len`) |
| `b745da4c5e` | A blob assigned to a view-model blob property is charged before the copy |
| `6cffead55f` | A view-model instance a script creates (`Data.X.new`, `model:new`/`model:instance`) is charged before it is built |
| `b5bead3eaa` | List `push` and `insert` are charged before the list grows |
| `8da3830f3a` | A gradient is charged for its stops before they are read |
| `ee6a3f6ab4` | `addToPath` refuses growth that would not fit the budget |
| `b589c0e732` | `artboard:instance()` is capped at 1024 live instances per VM |
| `26ecd59f47` | The `addToPath` test sizes its loop from the budget (so the old code fails it fast instead of timing out) |
| `817d5fde5e` | `#![deny(unsafe_code)]`, with `#[allow(unsafe_code)]` on the items that still hold `unsafe` |
| `d7c2b52647` | Codex round 1: gradient stops a stop's `__index` adds while the stops are read are charged |
| `6f39ec8c51` | Codex round 1: an instance stays charged while a property or method of it is held |
| `8a0596c5f7` | Codex round 2: a font or blob property drops its assigned charge once the model reads back empty |
| `d4c425cdfe` | Codex round 3: `Data.X.new(name)` is charged for the template it copies, before the script can keep it |
| `c14ae21a0e` | Codex round 3: an instance graph past the estimate's walk bound is refused, not estimated in part |

Each fix has a test that fails on the old code. I checked this by reverting
all fixes at once and keeping the tests: every new test failed. The
tests added for codex rounds 1–3 were each checked the same way, by reverting
only that fix.

## Items

### 1. Font assigned to a view-model font property: charged

Before, the setter took the native face and dropped the `ScriptedFont`, and
its decode charge was refunded with it. Now `ScriptedPropertyFont` keeps
`(FontRef, Rc<HostMemoryCharge>)` for the face it assigned. The getter hands
the same charge back through the new `create_charged_asset_font`. The pair
is matched by `Arc::ptr_eq` on the native face. The charge is dropped once
the model holds a different face, or none: a face the host cleared is refunded
when the property is next read (codex round 2; the blob property does the
same).

- Formula: the font's existing decode charge, shared rather than counted again.
- Test: `a_font_assigned_to_a_property_stays_charged` (226,004 B with `data_bind_font_test.riv`).
- Deviation from the brief: I did not use `push_charged_font`. That builds the font `from_native`. The getter has to keep the `from_asset` path (the font bytes), so `lua_font.rs` gained `create_charged_asset_font`.

### 2. View-model Blob setter, list `push`/`insert`, `Data.X.new`: charged before copy or growth

**Blob setter.** A String or Buffer is charged its `len()` before the bytes
are copied. A blob userdata shares the charge it already has.
`ScriptedPropertyBlob` keeps `(Arc<RuntimeBlobAsset>, Rc<HostMemoryCharge>)`
for the getter, the same way the font property does. `ScriptedBlob` in
`lua_blob.rs` now carries its charge.
- Test: `a_blob_assigned_to_a_property_is_charged`.

**Instance (`Data.X.new`, `model:new`, `model:instance`).** The charge is
`instance_bytes::of(model)`, taken before `named_instance` builds anything:

- `INSTANCE + Σ properties (PROPERTY + name length)`, plus nested view models and list items (`LIST_ITEM` each).
- The walk uses an explicit stack, capped at 4096 visited models. A graph past the cap estimates as `usize::MAX`, which no budget admits; it used to return the part it had walked (codex round 3).
- `object::<T>() = size_of::<T>() + 64`. The 64 B is a stated bound on the core-object overhead, not a measurement.
- `PROPERTY` is the largest of the 12 `ViewModelInstance*` value types.

This shape-based charge is taken before the copy. After the copy, the charge
is raised to `of(copied instance)` before the script can keep it, and if it
does not fit, the copy is dropped. An authored named template copies its
populated lists and nested instances, which the model's shape does not show.
The fixtures measured up to 136× the shape: `component_list_virtualized`
`MainVM "Instance 1"` is 733 B by shape and 32,953 B as copied. The runtime
has no way to measure a template before copying it, so the copy exists for
that moment (codex round 3).

The charge is shared (`Rc`) by:

- the table handle
- every method closure (`ChargedModel`)
- every property wrapper created from it (`owning_charge`)

So the charge lives as long as anything the script holds keeps the instance.

- Tests:
  - `data_new_is_charged_for_the_instance` (including a kept property and a kept method)
  - `model_new_is_charged_for_the_instance`
  - `data_new_is_charged_for_the_named_template`
  - `an_instance_graph_past_the_walk_bound_is_not_admitted`

**List `push`/`insert`.** Before the list grows, the list wrapper's charge is
raised to `(len + 1) × rate`. The rate is the largest of
`LIST_ITEM + of(item)` seen so far, starting at `LIST_ITEM`.

- The charge is a high-water mark: it does not fall when the list shrinks.
- Test: `list_growth_is_charged_before_it_happens`.

### 3. `DataValue` string: checked before copy

`LuaString::len()` is new on the facade (`luau/facade/handles.rs`). It reads
`vm.objlen` and does not copy. `set_value` checks
`check_fits(value.len())` before the bytes are copied.
`string_prefix` (cut at the first NUL) now shrinks the allocation, so the
string keeps only its bytes.

- Tests:
  - `a_data_value_string_is_checked_before_it_is_copied`
  - `a_data_value_string_cut_at_nul_keeps_only_its_bytes`
  - `string_length_matches_its_bytes` (facade)

### 4. Gradients: charged

The gradient is built in `vm/renderer.rs` (`Gradient.linear/radial`), not in
`lua_paint.rs`. `ScriptedGradient` holds the shader plus its charge, and
`lua_paint.rs` stores a `ScriptedGradient`, so a paint keeps the charge.

- Formula: `gradient_bytes(n) = n × (size_of ColorInt + size_of f32)`.
- It is charged for the table's length before the stops are read. It is then raised before each push to the vectors' grown capacity, because a stop's `__index` can add stops (codex round 1).
- Tests:
  - `a_gradient_is_charged_for_its_stops`
  - `gradient_stops_added_while_reading_are_charged`

### 5. `addToPath`: checked before growth; failure is an ordinary Lua error

`addToPath` first measures what the artboard would add, using an empty
`RawPath`. Then it checks that the whole grown path fits what the budget has
free, and only then adds. A refusal is the ordinary budget error, which
`pcall` catches.

- Formula: `(verbs × size_of PathVerb + points × size_of Vec2D) × 3`. This duplicates `lua_path.rs`'s private `path_bytes`.
- Test: `add_to_path_stops_at_the_budget`.
- Limit: this is a check, not a reservation (see "Where the design was wrong").

### 6. `artboard:instance()`: bounded, not charged

Capped at 1024 live instances per VM. Each `ScriptedArtboardOwner` holds a
slot, and dropping the slot frees it. Past the cap the error is
"script VM may hold at most 1024 artboard instances".

Why a cap rather than a charge: the runtime does not report an artboard
instance's size, and a size worked out in this crate would be a guess that
drifts. The cap bounds the count. Each instance is a copy of authored
content, so its size is bounded by the file.

- The largest count a fixture reaches is 30.
- Test: `artboard_instances_are_capped_while_held`.

### Deny `unsafe_code`

`lib.rs` has `#![deny(unsafe_code)]`. `#[allow(unsafe_code)]` sits only on
the smallest item or statement that holds remaining `unsafe`:

- `shader_asset.rs:316` `decode_vulkan_spirv`, `:376` `decode_apple_metal`, `:1413` test `provenance`
- `gpu_canvas.rs:3608` test `compile_source` (G2's file)
- `gpu_canvas_ore/canvas.rs:66` the `let color_view = unsafe {…}` statement (G2)
- `gpu_canvas_ore/mod.rs:195` the `let view = if ctx.isRecording() {…}` statement (G2)
- `vm/lua_audio_processors.rs:978` test `compile` (G1)
- `vm/luau_audio_processor_engine.rs:674` test `compile` (G1)
- `vm/luau_audio_web.rs:5` `mod host;` (G1)
- `vm/lua_path.rs:752` test `compile_source` (G4)
- `vm/upstream_scripting_gpu_features.rs:78,82` and `vm/script_call_gpu_scope_tests.rs:140,144` `unsafe fn wrapCanvasTexture` / `wrapRiveTexture`

The G1, G2 and G4 files got the attribute line only, nothing else.

`upstream_test_scripted_artboard_render` in `lua_artboards.rs` had a safe
body and is now a safe `pub fn`. Its caller
`tools/silver-corpus/src/scripted_artboard_replay.rs:203` now has a redundant
`unsafe` block. I left it alone because it is outside this lane.

Unsafe count for `crates/nuxie-scripting`: 47 → 46. The ratchet baseline was
not edited.

## Largest charges

Measured with a temporary high-water hook, since removed, that recorded each
budget's largest `used` and the largest single charge of each kind.

| Measure | Golden runner (`scripted-golden-compare`) | `nuxie` tests |
|---|---|---|
| Budget `used` | 1,916,738 B (`data_bind_blob_test.riv`) | 2,045,548 B (`nuxie__test_upstream_wave_a_final`) |
| Single list charge | 480,220 B (`data_bind_blob_test`) | — |
| Instance | 1,510 B (`list_focus_order.riv`) | 2,619 B (`upstream_scripting_context`) |
| Gradient | 40 B (`script_paths_opacity_test.riv`) | — |
| Live artboard instances | 30 (`script_artboard_opacity_test.riv`) | — |

No fixture assigned a blob or font through a property. These numbers were
measured at `817d5fde5e`, before the codex round 3 template fix. Instance
charges for named templates are larger now (up to the measured copy, e.g.
32,953 B). The high-water hook was not re-run; the golden compare and the
`nuxie` tests on the tip still pass under the 64 MiB default. The largest `used`
is about 3% of 64 MiB. The previous largest charge (E2b) was 46 KB; the new
instance and list charges make most of the rise.

## What a script now observes differently

All of these happen only at extremes. In-budget content renders the same: the
golden compare is unchanged.

- New budget refusals ("script VM exceeded its N-byte host memory budget") for:
  - a too-large blob, string or `DataValue` string
  - too many view-model instances
  - list growth
  - many-stop gradients
  - `addToPath` past the budget
- A `DataValue` string over the budget is refused even when a NUL would have cut it short, because the check happens before the cut.
- `push`/`insert` on a list that is already being changed (re-entry through a metamethod) raises "list is already being changed".
- "script VM may hold at most 1024 artboard instances".
- `Data.X.new` checks the budget before it builds the instance, so a budget error can come before an error the build would have raised.
- A list's charge does not fall when the list shrinks.
- A view-model graph past 4096 instances (for example a list holding 4096 items, or a script-made cycle) is refused when it is instanced or pushed into a list.
- A font or blob the host cleared stops counting against the budget once the property is read back as `nil`.

## Checks

All checks ran on `c14ae21a0e` with `NUXIE_EXECUTION=local` and
`RIVE_RUNTIME_DIR=/Users/levi/dev/worktrees/rive-runtime-pin-cd5351c3`.
Load averages are shown as 1/5/15 min.

| Check | Result | Load at start → end |
|---|---|---|
| `runtime.py test -p nuxie-scripting` | 39/39 targets pass (`--lib vm::` 451 passed, 1 ignored) | 19.33/18.97/17.92 → 14.80/16.83/17.21 |
| `tools/ratchets/panic-lints.sh` | `crates/nuxie-scripting` and `crates/nuxie-scripting:nuxie-scripting__publisher@wasm` are 0 for all 7 lints (+0) | 24.00/19.42/17.49 → 21.31/19.06/17.39 |
| `tools/ratchets/unsafe-count.sh` | PASS; `crates/nuxie-scripting` 47 → 46 (baseline not updated) | 21.31/19.06/17.39 |
| `runtime.py test -p nuxie --features scripting,test-support` | 41/42 pass; only the known red `stateful_component_image_bind` (`nuxie__test_upstream_data_binding_images_direct`) | 21.31/19.06/17.39 → 14.58/19.42/18.00 |
| `make scripted-golden-compare` | entries 364, exact 348, diverges 16, unsupported 0, not-yet 0; rc 0 | 14.58/19.42/18.00 → 14.67/18.15/17.62 |

Load was high because other lanes were building at the same time. None of
these checks measure time.

Free disk stayed at 122–137 GB.

## Not verified

- Apple and Android rows (no device run).
- Intermediate commits were not built one by one; only the tips named above were.
- `make golden-compare` (the non-scripted compare) was not run.

## Where the design was wrong, or what is left

1. **Gradient location.** The brief put gradients in `lua_paint.rs`. They are built in `vm/renderer.rs`, so I edited `renderer.rs`, which is not on the lane's list. `lua_blob.rs` (no owner) was also edited, for the blob charge.
2. **`addToPath` needs `lua_path.rs`.** The fix is a bounded check, not a reservation. A path filled only through `addToPath` therefore holds memory nothing has charged, so several such paths together can exceed the budget, and a path that already holds a charge is counted twice in the check. The real fix keeps the reservation on the path: make `ScriptedPath::reserve` `pub(super)` in `lua_path.rs` (G4) and call it from `addToPath`. Reported to G4, not done here.
3. **Charges on Lua wrappers cannot follow ownership in the native graph.** The font, blob, list and instance charges live on the Lua-side wrappers. When the native graph keeps a value alive without a wrapper, the charge is refunded while the memory stays. This happens when a sibling property keeps the native model, or when an instance is held as a nested view model, a list item, `list:pop`'s result or `context:viewModel`. Tying the charge to the native property's lifetime needs runtime support (G4, `crates/nuxie-runtime`).
4. **`Text:append` keeps `render_paint`, and so the shader**, after the Paint and Gradient wrappers are gone. The gradient charge does not follow it. The fix is in `lua_text.rs` (G4).
5. The brief named `push_charged_font`. It does not fit, because it rebuilds the font `from_native`, so I added `create_charged_asset_font` (item 1).
6. The 64-byte core-object overhead in `instance_bytes` is a stated bound, not a measurement. The runtime does not report artboard-instance size, hence the cap in item 6.

## Codex review (`codex review --base levi/luaur-client-stage1c`, 4 rounds)

| Round | Findings | Outcome |
|---|---|---|
| 1 | 5 × P1 | Gradient stops added during the read: fixed (`d7c2b52647`). Instance retained by properties and methods: fixed (`6f39ec8c51`). addToPath reservation, native-ownership charges, and Text paint retention: not fixable in this lane (items 2–4 above). |
| 2 | 3 × P1 (repeats), 1 × P2 | P2, an assigned font/blob staying charged after a native clear: fixed (`8a0596c5f7`). |
| 3 | 3 × P1 (repeats), 2 × P2 | Named template under-charged: fixed (`d4c425cdfe`). Partial estimate past the walk bound: fixed (`c14ae21a0e`). |
| 4 | 3 × P1 (repeats) | Nothing new. Codex's probes confirm the blob-alias and Text-gradient gaps, which are reported under items 3 and 4 above. |

Round 3 was run once by mistake with a prompt alongside `--base`, which this
codex CLI rejects, so no review happened in that run. It was re-run plain.
