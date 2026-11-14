# Luau client path, lane E2b: host memory budget (2026-11-14)

Design: `docs/luaur-client-path-safe.md` 12.2, "E2b". Inputs: the E1
inventory (`docs/evidence/luaur-client-path-lane-e1-inventory-2026-11-14.md`)
section 5 (host allocations a script controls) and the lint rows for the
owned files. Branch `claude/luaur-lane-e2b-host-budget-877d5b`, based on
`levi/luaur-client-stage1c` (`d15e2cfbeb`). One concern per commit:

| Commit | Change |
|---|---|
| `7b0c990c2e` | `HostMemoryBudget` module and its wiring in `vm.rs` |
| `2d06f57265` | `print` lines are truncated at 1 MiB (row 2) |
| `7954d232aa` | `decodeImage` uses the bounded decoder, is charged and capped in flight (rows 3, 4) |
| `e2d93adce4` | mesh buffers charge the budget; the mesh instance cap stays (rows 10, 11) |
| `82fbfc04df` | paths charge before growing (rows 1, 12) |
| `65606a181a` | Text handles check a `u64` epoch; appends are charged; style ids cannot wrap (row 13, lint row 9) |
| `ae897b22cd` | fonts and data-value strings charge the budget; binding tests (rows 20, 24) |
| `e467f91e5e` | a decoded font stays charged while a derived font or a Text keeps its face (review) |
| `61877e5c90` | `Path:reset` drops the render path so the refund is exact (review) |
| `a1f50a960a` | a cancelled decode stays charged until its task lets go of the buffers (review) |
| `80a263aa56` | the print cap counts the shared line buffer, across nested prints (review) |
| `823803a5cd` | `glyph.font` shares the decoded charge; a Text keeps its font charges for life (review) |
| `766877e8f4` | an empty measure extract needs no budget room (review) |
| `d51435146f` | a data-value string cut at NUL drops the rest of its copy (review) |
| `2310d4300a` | a Text's font list is charged by capacity (review) |
| `9608f5dbb6` | `Path:reset` keeps the render path and its charge (reverts the render-path drop of `61877e5c90`, see Checks) |

## The budget as built

`crates/nuxie-scripting/src/vm/host_memory_budget.rs`.

- One `HostMemoryBudget` per script VM: an `Rc` of a limit and a used count
  (`Cell<usize>` each), kept in the VM's app data, so any binding that has
  the `Lua` reaches it with one app-data lookup; no thread-local.
- `ScriptVm::new` installs it with `DEFAULT_SCRIPT_HOST_MEMORY_BYTES`
  (= `DEFAULT_SCRIPT_VM_MEMORY_BYTES`, 64 MiB);
  `ScriptVm::new_with_execution_limits` installs
  `ScriptExecutionLimits::max_host_memory_bytes()`, set with
  `with_max_host_memory_bytes`. A state that never went through either
  (a bare `Lua` in tests) gets the default on first use.
- A binding holds a `HostMemoryCharge`: an RAII amount. `reserve_to(n)`
  raises it to `n` (never lowers), `grow(n)` adds, `set(n)` raises or
  refunds; drop refunds all. A charge is taken before the allocation it
  covers, and every refusal is one text:
  `script VM exceeded its {limit}-byte host memory budget`.
  Each charge is O(1): one compare and two `Cell` writes.
- Charges count capacity, not length. Growing containers charge the
  capacity the next growth will reach, `max(required, 2 × capacity, 4)`
  (`grown_capacity`), then `try_reserve_exact` that much, so the charge
  equals what the allocator was asked for and an allocator refusal is a
  Lua error, not an abort.
- A clear keeps the capacity, so it keeps the charge (high water). Only a
  release (drop, `Path:reset`, a GC'd handle) refunds.

Nothing is cached, pooled or retained by the budget; it is two counters.

## Charge formula per row

| Row | Binding | Charge |
|---|---|---|
| 1, 12 | `Path` commands, `add` (incl. self-add), measure extracts, `Font:glyphPath`, `Path.new` from a source | `3 × (verbs × 1 + points × 8)` bytes for the raw path's verb and point vectors plus the render path built from them; charged at the counts after the operation, before it. Extracts check the worst case (`2·pieces` verbs, `4·pieces` points, `pieces = 2V + 2`) fits, then charge the actual result. `reset` refunds to 0 if no render path was built; once one is, it is kept (a draw rewinds it in place) and the charge stays as a high-water mark. |
| 2 | `print` | not charged: a line is cut at 1 MiB (`MAX_PRINT_LINE_BYTES`) and ends with ` [truncated]`. The count is kept for the logging context's one shared line buffer, not per call, so a `__tostring` that prints and fails part-way (leaving bytes in the buffer) counts against the outer line. Why truncate: the line is handed to the host logger and freed when the call returns, so a charge would be refunded at once; the danger was only the size of one line (8,000 × 8 MiB args). Every argument is still `tostring`'d, in order, so `__tostring` side effects are unchanged. |
| 3, 4 | `context:decodeImage` | `len(encoded)` before the copy, plus `width × height × 4` from the header preflight before submit; held until the promise settles (the result is then a VM buffer the VM's own allocator charges) or, if cancelled, until the task drops its buffers. The decoder is the bounded one (8,192 px a side, 64 MiB decoded). At most 16 decodes in flight per VM: `decodeImage: at most 16 images may be decoding at once`. |
| 10 | `VertexBuffer:add` | `grown_capacity × 16` (the `Vec2D` values plus the byte mirror `update` builds) |
| 11 | `TriangleBuffer:add` | `grown_capacity × 4` (the `u16` values plus the mirror) |
| 10, 11 | `ImageMeshInstance` | `144` bytes per instance (3 × 48 B `ImageMeshInstanceData`: staged, committed, renderer copy); the per-VM count cap of 349,525 and its error text are unchanged, and the count cap still wins first in every existing test. |
| 13 | `Text:append` | `24 × input bytes + 64` per run, `+ 256` per new style, high water across `clear`; plus the font list (one 24 B entry per change of font between appends, charged by grown capacity, kept across clears). A 65,537th distinct style is an error (`Text may hold at most 65536 styles`) where it used to wrap the `u16` id and draw with another style's paint. |
| 20 | `Font.decode` | `len(bytes)` (the face keeps its bytes); one `Rc` charge shared by every font derived with `withOptions`, every Text it was appended to (for the Text's life: shaped runs keep the face past a clear until the next layout) and every font `glyph.font` returns for it; refunded when the last goes. |
| 24 | `DataValue` string values | the string's capacity (`String`) or length (`CoreString`), charged before it replaces the old value. A value cut at its first NUL is shrunk to what is kept, so the dropped tail is neither held nor charged. |

## Largest charge reached

Measured with a temporary high-water hook (written per VM to a scratch
directory, then removed before the commits; not in the tree).

| Run | Largest charge |
|---|---|
| Golden fixtures (`make golden-compare`, `make scripted-golden-compare`) | 46,050 B (`script_path_effects_test.riv`); next `group_effect.riv` 20,844 B |
| `nuxie` tests (`--features scripting,test-support`) | 68,310 B (`path_effects_examples`) |
| Pre-existing `nuxie-scripting` tests | 180,207 B (`path_commands_preserve_raw_quad_and_cubic_point_order`) |
| Lane A's mesh-limit test (349,525 instances) | 50,331,744 B (349,525 × 144 = 48 MiB, under 64 MiB) |
| This lane's limit tests | up to 22,770,732 B (the 65,537-style test) |

So every fixture and existing test runs at under 0.3 % of the default
budget; within-budget scripts observe nothing new.

## Panic lints

Forced-lint sites (`tools/ratchets/panic-lints.sh`, the seven panic lints)
per owned file:

| File | Before (Apple / wasm) | After |
|---|---|---|
| `lua_text.rs` | 41 / 41 | 0 / 0 |
| `lua_path.rs` | 4 / 4 | 0 / 0 |
| `lua_mesh.rs` | 3 / 3 | 0 / 0 |
| `lua_font.rs` | 2 / 2 | 0 / 0 |
| `lua_image_decode.rs` | 1 / 0 | 0 / 0 |
| `renderer.rs` | 1 / 1 | 0 / 0 |
| `lua_rive_base.rs`, `lua_data_value.rs` | 0 / 0 | 0 / 0 |
| `host_memory_budget.rs` (new) | — | 0 / 0 |
| crate total | 212 / 211 | 160 / 160 |

No other file's count changed. `tests/ratchets/*` were not edited; the
script reports "counts went down; run with --update to lock them in".
Load averages: before 6.18 8.41 11.03; after 25.53 29.94 25.53; after the
review fixes 11.31 13.83 18.82 (Apple only; 160).

Shape changes rather than `allow`s: indexing became `get`/iterators/zip,
arithmetic became checked or saturating with an error, `expect` on the
render path became `Option::insert`, `lua_text` table building uses
`raw_push`.

`lua_text.rs`: a `RawText` revision is a `u32` that wraps; a handle compared
revisions, so a line handle from 2^32 edits ago could pass as fresh. Each
layout now keeps a `u64` epoch advanced whenever the revision it last saw
changes; handles keep the epoch, and a stale handle errors with the
existing stale-handle text. Row 9 (`selectionRects`): the rects are
collected, the `RefCell` borrow ends, then the tables are built.

## Rows not covered here, and why

| Row | Why |
|---|---|
| 5–9 | GPU resources and canvases: lane D4's files (`gpu_canvas*`), off-limits. |
| 14–18 | Audio: off-limits. |
| 19 | `artboard:instance()`: `lua_artboards.rs` is D2's. |
| 20 (gradients) | `ScriptedGradient(Rc<…>)` is built in `lua_paint.rs:259` (D3's) and its shader is retained there, so a charge would have to live in `lua_paint.rs`. |
| 21–23 | listeners: `view_model.rs` (D3's) and the runtime. |
| 24 (rest) | the Blob setter, list `push`/`insert` and `Data.X.new` are in `view_model.rs` (D3's). |
| 25 | `vm.rs` `install_native_file_assets` (D3's). |
| 26 | `host_commands.rs` (D3's). |

Also not covered, in owned bindings:

- `addToPath` (`lua_artboards.rs`, through `with_render_raw_path_mut`)
  grows a scripted path through a closure that cannot return an error; the
  growth is charged at the path's next operation, not before.
- `RawText` builds shaped and outline geometry in `RawText::update`
  (nuxie-runtime) that it does not report; the 24 B/input-byte figure is
  the inventory's estimate, not a measurement of each layout. A charge of
  the real size needs `RawText` to report it.
- Measure extracts are admitted on a bound of the whole measured source
  (`2V + 2` pieces of 2 verbs and 4 points), not the requested interval:
  bounding the interval needs the measure's segment table, which
  `ContourMeasure`/`PathMeasure` (nuxie-runtime) do not expose. So a
  non-empty extract from a large source can be refused when the budget has
  less than about 8× the source's charge free (at the default budget, a
  source over about 8 MiB of host bytes; the largest fixture path charge is
  46 KB). Empty intervals need no room.
- `RawPath` has no `capacity`/`try_reserve`, so a path charges the length
  it will reach (×3 for the render copy) and the push itself is still the
  infallible `Vec` growth. Within budget that is at most 64 MiB of pushes,
  so not an abort in practice, but not an `Err` from the allocator either.
- A font the shaper falls back to (not decoded by the script) has no
  charge; `glyph.font` for it is uncharged, as before.
- A host writer of the shared log line (`lua_image.rs`,
  `lua_audio_processors.rs`) that runs inside a `__tostring` empties the
  buffer without resetting print's count, so that outer line may be cut
  early; the line was already broken by that writer's `begin_line` before
  this lane.

Left open from review round 4:

- A decoded font assigned to a view-model font property: `ScriptedPropertyFont`
  (`view_model.rs`, D3's) keeps only the native font, so collecting the
  script's font refunds while the property keeps the face; reading the
  property back gives an uncharged wrapper. Fix: the property keeps
  `ScriptedFont::charge()` (an `Rc`, already shareable) and hands it to the
  wrappers it creates, through `lua_font::push_charged_font`.
- `Path:measure()` and `Path:contours()` build contour point and segment
  tables (and the iterator a copy of the path) with no charge, so a script
  can keep measures after resetting the source. In `lua_path.rs`, but the
  segment count depends on curve subdivision inside `ContourMeasure`
  (nuxie-runtime), which does not report its size; a sound charge needs
  that, or a conservative per-curve bound worked out from the tolerance.
- Outline text geometry (round 1 again, measured by the reviewer at about
  2.78 MB for 10,000 ASCII characters against a 240 KB charge): needs
  `RawText` to report what `update` allocates.
- `DataValue` string assignment copies the VM string (`LuaString::as_bytes`)
  before the charge is checked. The copy is transient and at most one VM
  string (bounded by the VM's own memory limit); checking first needs a
  length accessor on the facade's `LuaString` (not owned here).

## Behaviour that changed (all outside budget or at extremes)

- `decodeImage` of an image over 8,192 px a side or 64 MiB decoded now
  rejects with the existing `failed to decode image data`; before it
  decoded (and was rejected afterwards only if over the VM ceiling).
- A `print` line over 1 MiB is cut with ` [truncated]`.
- New error texts: the budget text above; `decodeImage: at most 16 images
  may be decoding at once`; `Text may hold at most 65536 styles`;
  `Text has an invalid align value`, `Text has an invalid direction value`,
  `glyph has no font` (each replacing a panic).
- When other host memory is already in use, a large mesh can see the
  budget error before the mesh count error.

## Review

`codex review --base levi/luaur-client-stage1c`, four rounds.

| Round | Findings | Outcome |
|---|---|---|
| 1 | font charge lost when the decoded font is collected; `Path:reset` keeps the render path; cancelled decode refunded early; `RawText::update` geometry uncharged | font and decode fixed; the reset fix (drop the render path) changed the golden stream and was replaced by keeping the charge; geometry needs `RawText` to report its size (nuxie-runtime), listed above |
| 2 | nested print bypasses the cap; `glyph.font` uncharged; clear refunds fonts shaped runs still hold; extract admission on the whole source | first three fixed; empty extracts fixed, the whole-source bound kept and explained above |
| 3 | NUL-cut string keeps its copy's capacity; Text font list grows across clears | both fixed |
| 4 | a font assigned to a view-model font property loses its charge; `measure()`/`contours()` storage uncharged; outline text geometry; a data-value string is copied before its charge is checked | not fixed: the review limit was reached, and three need files or types outside this lane (below) |

## Checks

| Check | Result | Load average (start → end) |
|---|---|---|
| `python3 tools/bazel/runtime.py test -p nuxie-scripting` | 39/39 pass | 14.97 → 32.89; after review fixes 8.17 → 12.09 |
| `python3 tools/bazel/runtime.py test -p nuxie --features scripting,test-support` | 41/42; the only red is the known `stateful_component_image_bind` | 32.66 → 13.06 |
| `make luau-vm-diff` | 409 runs, 0 disagreements | — |
| `make scripted-golden-compare` | exact 348, diverges 16 | 13.06 → 22.13 |
| `make golden-compare` | exact 356, diverges 8 | 22.13 → 29.70 |
| `tools/ratchets/panic-lints.sh` | PASS, 160 / 160 | above |

Rerun on the tip after the review fixes:

| Check | Result | Load average (start → end) |
|---|---|---|
| `nuxie-scripting` tests | 39/39 | 11.45 → (with the next two) 12.88 |
| `nuxie` tests | 41/42, only `stateful_component_image_bind` | → 12.88 16.81 14.58 |
| `make luau-vm-diff` | 409 runs, 0 disagreements | 16.54 |
| `make scripted-golden-compare` | exact 348, diverges 16 | 11.45 → 12.88 |
| `make golden-compare` | exact 356, diverges 8 | 15.03 → 11.69 |

The first rerun caught a regression: the round-1 `Path:reset` fix dropped
the render path, so the next draw made a new one through the factory, and
`script_layout_test` failed exact verification on an extra
`makeEmptyRenderPath`. That fix was replaced (reset keeps the render path
and the charge), and the scripted compare and both test suites were run
again green, then `luau-vm-diff` (0 disagreements) and `golden-compare`
(356 / 8) once more on the tip (load 13.03 → 15.29). Disk stayed at 160–162 GB free.

Each fixed row has a test that fails on the old code: the path self-add
(1 MiB budget, 30 doublings), path commands and extracts (64 KiB), vertex
and triangle buffers, mesh instances (100 × 144 B budget refuses the
101st), font decode (226,004 B), Text append and clear, 65,537 styles,
data-value strings, decodes in flight, a decode charged until it settles,
a cancelled decode charged until its task lets go, a font charged while
anything keeps its face, a glyph's font sharing the charge, a Text's font
list charged across clears, an empty extract at a 1,000-byte budget,
NUL-cut data-value strings, a failed nested print counted against the
line, the epoch check, and print truncation.

## Not verified

- `Path:reset` keeping the charge once a render path exists has no
  dedicated test (the existing reset test never draws).
- The wasm lint count after the review fixes (Apple only; the touched code
  has no target-specific parts).
- The real size of `RawText`'s shaped data per layout.

## Where the design was wrong or thin

- "Mesh budget becomes a charge": done as a charge beside the count cap,
  not instead of it; the 349,525 cap and text had to stay, and a byte charge
  alone would let 64 MiB / 144 B = 466,033 instances through.
- Gradients (row 20) cannot be charged from `renderer.rs`; the retaining
  type is in `lua_paint.rs`.
- Charging "before growth" for `RawPath` and `RawText` needs those
  nuxie-runtime types to expose capacity or size; without that the charge
  is a formula, not a count of what was allocated.
- Cancelled decodes: refunding at cancel undercounts, because the pool
  task still holds the copy; the charge now waits for the task.
