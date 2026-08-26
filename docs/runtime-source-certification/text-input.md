# TextInput source-pair correspondence candidate

Status: **author candidate; pending independent semantic review**

Pinned upstream: `4ac7b32798da0482e441ef09304dc3b480ed3ee5`

This is the first atomic source-pair audit under
`docs/runtime-exact-parity-workflow-correction.md`. It does not inherit the
older file-level `mapped` or `faithful` verdict. The complete pinned files were
read side by side with every current Rust owner:

- `src/text/text_input.cpp` — 777 lines, 22,898 bytes, SHA-256
  `cad4952755d38a6ad8b0f47cdb3bd041af7cd629af1801e19116263b1d8a1265`;
- `include/rive/text/text_input.hpp` — 137 lines, 4,841 bytes, SHA-256
  `a640ca2b11d9a7df6cfbb7a03ac3bc58f66aeaacdda04ca027c7c8f4fd5098fe`.

The generated denominator assigns 38 authority units to the `.cpp`: 37 C++
functions and one Emscripten `EM_JS` statement. The header contributes five
executable inline methods plus one non-behavioral include guard. Every unit is
listed below. `exact` and `adapted` are candidate source-read classifications,
not certification: all rows still need fresh independent review and direct
behavioral evidence. `incorrect` and `missing` are blockers to parity.

## Concrete Rust ownership boundary

The primary owner is `crates/nuxie-runtime/src/text_input.rs`. Exact behavior
is currently distributed across these narrower owners rather than being
claimed transitively from their whole files:

- occurrence fields and cold defaults:
  `components.rs::RuntimeTextInputState::{default}` and
  `retain_runtime_text_input_scroll_constraints`;
- advancing dispatch: `artboard/text/text_input.rs::advance_text_input_entry`;
- ScrollConstraint calls made by TextInput:
  `constraints.rs::{text_input_scroll_viewport,
  reset_text_input_cross_axis_scroll,scroll_text_input_caret_into_view,
  advance_text_input_scroll}`;
- Taffy/text geometry adapter:
  `text.rs::text_input_layout_measure_bounds` and
  `StaticTextSlice::from_text_input_graph`;
- empty draw and concrete hit routing:
  `draw.rs::runtime_draw_live`'s `TextInputContainer` branch and
  `state_machine_instance.rs::hit_expandable`;
- focus/text dispatch:
  `state_machine_instance/text_input_listener_group.rs` and
  `keyboard_listener_group.rs`.

No whole-module path is evidence for a member. The locations below name only
the concrete symbol that owns the corresponding behavior.

## Out-of-line `.cpp` authority: lifecycle, layout, and text synchronization

| # | Pinned definition | Required behavior, side effects, and order | Concrete Rust owner | Candidate disposition and evidence/blocker |
|---:|---|---|---|---|
| 1 | `TextInput::draw` (17) | Intentional empty draw; concrete children paint instead. | `draw.rs:4606-4613`, `RuntimeDrawableFamily::TextInputContainer` | **exact, pending evidence**. The draw traversal skips the container. Existing render tests are downstream evidence only. |
| 2 | `TextInput::hitTest` (19) | Always returns null. | No callable Rust member; generic geometry does not return the container, while state-machine input uses `hit_expandable`. | **missing direct owner**. Blocker: prove the public/general hit path cannot return TextInput itself, or add the literal owner during source correction. |
| 3 | `TextInput::hitTestPoint` (21) | Invert world transform; reject failed inverse; reject outside local bounds; only then delegate to `Drawable::hitTestPoint` with both flags unchanged. | `state_machine_instance.rs:4501-4531::hit_expandable`, `text_input.rs:525::text_input_local_bounds_retained` | **incomplete**. The live TextInput listener path preserves bounds-before-parent ordering, but hard-codes the caller's `true,true` flags and is not a general override. Blocker: pending `text_input_test.cpp` hit cases and a direct flag-preservation owner. |
| 4 | `textChanged` (43) | Copy generated `m_Text` to source; sync display without preserving cursor; mark layout dirty; mark shape dirty, in that order. | `text_input.rs:253::text_input_property_changed`, `text/text.rs:57::mark_shape_dirty` | **exact candidate, pending evidence** for valid UTF-8. The callback updates source/display before publishing shape and layout dirt. Blocker: direct state/dirt trace. |
| 5 | `selectionRadiusChanged` (54) | Set raw selection radius, then always publish TextShape dirt. | `text_input.rs:301::text_input_selection_radius_changed`, `raw_text_input.rs:568::set_selection_corner_radius` | **incorrect**. Rust returns early for an unchanged radius and publishes `PATH`, not the pinned unconditional `TextShape` dirt family. Blocker: direct repeated-write dirt trace. |
| 6 | `multilineChanged` (62) | Call `updateMultiline(true)`. | `text_input.rs:278::text_input_multiline_changed` | **incomplete**. Live changes sync display with cursor preservation and reset the cross-axis scroll, but raw max-width/sizing is represented indirectly by static geometry and the initial lifecycle does not execute this owner. Blocker: pending multiline/control-size tests. |
| 7 | `markPaintDirty` (64) | Add Paint dirt. | `text_input.rs` call sites using `add_dirt(..., ComponentDirt::PAINT, false)` | **exact candidate, pending evidence**. Blocker: a direct dirt trace rather than a final render. |
| 8 | `markShapeDirty` (66) | Add TextShape dirt. | `text/text.rs:57::mark_shape_dirty` | **adapted Rust ownership, pending evidence**. Rust expands the logical TextShape event into text revision, Path, WorldTransform, and layout-node invalidation. Blocker: prove the expanded dirt graph preserves C++ ordering and does not create extra observable work. |
| 9 | `localBounds` (68) | Return retained RawTextInput bounds. | `text_input.rs:525::text_input_local_bounds_retained` | **exact candidate, pending evidence** through retained geometry. |
| 10 | `onAddedClean` (77) | Run superclass; retain first TextStyle; copy source; install font/font size; sync display; retain first scroll constraint on grandparent transform; run `updateMultiline(false)`; return MissingObject without style. | `text_input.rs:193::initialize_text_input`, `components.rs:2085::retain_runtime_text_input_scroll_constraints`, `artboard.rs:2805` initialization call | **incorrect/incomplete**. Rust retains the first style and scroll constraint and constructs display geometry, but has no MissingObject result, does not reject a missing TextStyle, and does not perform the initial cross-axis scroll reset owned by `updateMultiline`. Blockers: missing-style import and authored nonzero cross-axis-scroll fixtures. |
| 11 | `update` (113) | Super first; for Paint/TextShape set font size, update RawTextInput; on shape change update world bounds and auto-height layout dirt; on selection change invalidate every child stroke effect; then, unless dragging/edge-scrolling, stop physics and scroll caret into the visible axis. | `artboard.rs:9028-9062` update dispatch, `text_input.rs:126::refresh_text_input_geometry`, `541::adjust_text_input_scroll_to_caret`, `draw.rs:18017::update_runtime_text_render_styles` | **incomplete/adapted**. Geometry/world-bounds and caret scrolling exist, but the update trigger mask/order differs, selection invalidation is represented by rebuilding retained child draw owners rather than an explicit stroke-effect invalidation, and auto-height layout publication is not bound to the raw `shapeDirty` result. Blocker: focused update-phase state trace for all flag combinations. |
| 12 | `measureLayout` (199) | Map undefined width/height to float max, call raw measure, return bounds size. | `text.rs:518::text_input_layout_measure_bounds`; Taffy callers in `draw.rs:13961` and `14128` | **adapted (Taffy), pending evidence**. Bounds caching and max constraints exist, but the callback signature is projected through Taffy. Blocker: exact four-mode measure differential. |
| 13 | `controlSize` (218) | Store only `size.x`, ignore the other arguments, then run `updateMultiline(false)`. | `text_input.rs:126::refresh_text_input_geometry` retains constraint width; `artboard.rs:6868::invalidate_runtime_layout_text_host` dirties geometry | **adapted (Taffy), pending evidence**. The resolved width reaches geometry, but there is no one-call owner preserving the exact assignment-then-update order. Blocker: control-size state trace. |
| 14 | `strippedLineBreaks` (227) | Byte-preserve input while coalescing each contiguous CR/LF run to one ASCII space. | `text_input.rs:36::strip_line_breaks` | **adapted (Rust safety), pending evidence**. Exact for valid UTF-8; Rust `str` excludes invalid byte strings. Existing unit coverage checks mixed CR/LF runs. |
| 15 | `displayedText` (251) | Source unchanged when multiline; stripped source otherwise. | `text_input.rs:193::initialize_text_input`, `253::text_input_property_changed`, `278::text_input_multiline_changed` | **exact candidate, pending evidence**. |
| 16 | `syncDisplayedTextFromSource` (256) | Return if display is unchanged; otherwise select preserving/non-preserving raw setter from the flag. | `raw_text_input.rs:244::{set_text,set_text_preserve_cursor}` with `text_input.rs:253,278` callers | **exact candidate, pending evidence**. The two call sites preserve the false/true split. |
| 17 | `syncSourceTextFromRaw` (275) | Read raw text; for single-line strip and write corrected raw text preserving cursor; move to source; invoke generated `text(source)` callback. | `text_input.rs:317::sync_text_input_source_from_raw` and `set_string_property` | **exact candidate, pending evidence** for valid UTF-8. The generated-property notification path remains part of the required evidence. |
| 18 | `cursorBoundary` (294) | Meta wins as line; Alt+Ctrl is subword; Alt alone word; otherwise character. | `text_input.rs:336::text_input_cursor_boundary` | **exact candidate, pending evidence**. |
| 19 | `EM_JS isWindowsBrowser` (312) | On wasm, inspect `navigator.platform` for `Win`. | No Rust owner. | **missing/incorrect**. Rust cannot distinguish a Windows browser on `wasm32`; this directly affects undo/select system shortcuts. Blocker: a browser-platform input seam. |
| 20 | `systemModifier` (317) | Emscripten Windows browser and native Windows use Ctrl; all others use Meta. | `text_input.rs:28::system_modifier` | **incorrect on wasm Windows browsers**; exact on native targets. Evidence blocker is the missing `navigator.platform` owner from row 19. |
| 21 | `updateMultiline` (330) | Set raw maxWidth/sizing for the selected mode; clear only the obsolete nonzero scroll axis after stopping physics; optionally sync display preserving cursor; mark layout dirty; add TextShape dirt. | `text_input.rs:278::text_input_multiline_changed`, `constraints.rs:1100::reset_text_input_cross_axis_scroll`, static `TextInput` sizing in `text.rs:3811::authored_sizing` | **incomplete**. Live property changes reproduce display/scroll/dirt at a higher-level representation, but initialization and `controlSize` do not call one equivalent owner, and no retained raw maxWidth/sizing fields exist. Blocker: initial, live-toggle, and control-size traces. |

## Out-of-line `.cpp` authority: input, focus, coordinates, and dragging

| # | Pinned definition | Required behavior, side effects, and order | Concrete Rust owner | Candidate disposition and evidence/blocker |
|---:|---|---|---|---|
| 22 | `keyInput` (370) | Ignore releases. For each recognized pressed key, run the exact raw operation, then publish the specified dirt even if the raw value did not change. Undo/redo/backspace/delete sync source then shape dirt; select/navigation use paint dirt; Enter rejects single-line but multiline inserts/syncs and uses **Paint** dirt. | `text_input.rs:382::text_input_key_input` | **incorrect**. Rust gates dirt/source sync on the raw method's `changed` boolean, so recognized no-op keys omit pinned dirt. It also routes multiline Enter through shape dirt rather than Paint. Blockers: repeated/no-op key dirt trace and Enter dirt assertion; do not fix from the tests without reusing this source order. |
| 23 | `textInput` (467) | Strip line breaks only for single-line; empty insertion succeeds without work; otherwise insert, sync source, shape dirty; always return true. | `text_input.rs:488::text_input_text_input` | **exact candidate, pending evidence** for a valid TextInput occurrence and valid UTF-8. |
| 24 | `gamepadDispatch` (482) | Always false and does not write the optional output pointer. | `input/focusable.rs:55::gamepad_dispatch_default`; no TextInput gamepad group is registered | **exact candidate, pending evidence**. Blocker: direct TextInput-focused gamepad dispatch assertion. |
| 25 | `focused` (487) | Always assign true, then Paint dirt. | `text_input.rs:75::sync_text_input_focus` | **incorrect side-effect gating**. Rust publishes Paint only when the bool changes. Blocker: repeated focus callback dirt trace. |
| 26 | `blurred` (493) | Always assign false, clear selection, then Paint dirt. | `text_input.rs:75::sync_text_input_focus` | **incorrect**. When already unfocused Rust skips both selection clearing and Paint dirt; a retained selection can therefore survive an explicit repeated blur. Blocker: unfocused-with-selection callback trace. |
| 27 | `worldPosition` (502) | Use `worldTranslation`, then apply the owning Artboard's root transform for nested artboards; always return true. | Only tools-only `text_input.rs:849::debug_text_input_world_point`; no production Focusable owner | **missing production owner**. Blocker: nested-artboard TextInput focus/world-position case. |
| 28 | `worldBounds` (521) | Reject empty/NaN retained world bounds; otherwise root-transform min and max and return true. | `RuntimeTextInputState::world_bounds` is written at `text_input.rs:142-171` but has no production reader | **missing production owner**. Blocker: direct nested root-transform bounds query; downstream geometry bounds are not a substitute. |
| 29 | `edgeScrollSpeedForDistance` (545) | `clamp(45 + distance*4, 45, 400)`. | `text_input.rs:59::edge_scroll_speed_for_distance` | **exact candidate, pending evidence**. |
| 30 | `edgeActivationDistance` (555) | Zero at/inside edge; otherwise `edgeStart-position`. | `text_input.rs:66::edge_activation_distance` | **exact candidate, pending evidence**. |
| 31 | `worldToLocalWithViewport` (560) | Zero both velocities before inversion; invert or fail; transform point; for active single/multiline axis compute 20px edge distances, set signed speed only when enabled, and clamp coordinates only beyond viewport. | `text_input.rs:611::text_input_move_cursor_to_world_with_auto_scroll`, `constraints.rs:1074::text_input_scroll_viewport` | **exact algorithm candidate, packed with cursor movement**. Blocker: expose/trace the intermediate local point and both velocities independently; current final cursor result is insufficient. |
| 32 | `startDrag` (653) | Set dragging and last world position before conversion; on successful conversion move cursor without selection and always Paint dirt. | `text_input.rs:710::text_input_start_drag` plus `602::text_input_move_cursor_to_world` | **incorrect dirt gating**. State-before-conversion order is exact, but Paint dirt occurs only when the cursor changes. Blocker: same-position start-drag trace. |
| 33 | `drag` (672) | Update last world position before conversion; auto-scroll conversion; on success extend selection and always Paint dirt. | `text_input.rs:721::text_input_drag` plus `611::text_input_move_cursor_to_world_with_auto_scroll` | **incorrect dirt gating**. Blocker: successful conversion with unchanged cursor/selection. |
| 34 | `endDrag` (688) | Ignore the argument; clear dragging, set last position to NaN/NaN, zero both velocities. | `text_input.rs:731::text_input_end_drag` | **adapted (Rust signature), exact state candidate**. Caller drops the unused position. Pending direct field trace. |
| 35 | `selectWord` (696) | Invoke raw selectWord and always Paint dirt. | `text_input.rs:743::text_input_select_word` | **incorrect dirt gating**. Rust paints only when selection changes. Blocker: repeated selectWord trace. |
| 36 | `selectLine` (704) | Invoke raw selectLine and always Paint dirt. | `text_input.rs:755::text_input_select_line`, `350::text_input_line_range_for_cursor` | **incomplete/incorrect dirt gating**. Rust reconstructs the visual-line range outside RawTextInput and paints only on change. Blocker: wrapped/bidi visual-line differential plus repeated-call dirt trace. |
| 37 | `advanceDrag` (712) | If not dragging, zero velocities and stop. Require nonzero velocity and constraint. Stop physics, advance and clamp each active axis, then re-hit the finite last position with auto-scroll, move selection, and always Paint on successful conversion; return current dragging state. | `artboard/text/text_input.rs:54::advance_text_input_entry`, `constraints.rs:1187::advance_text_input_scroll`, `text_input.rs:721::text_input_drag` | **incorrect dirt gating; otherwise exact candidate**. The state/physics/clamp/re-hit order exists, but the delegated drag omits Paint when the cursor does not change. Blocker: stationary-cursor edge-scroll trace. |
| 38 | `advanceComponent` (774) | Ignore flags and return `advanceDrag(elapsedSeconds)`. | `artboard.rs:6737` TextInput schedule to `advance_text_input_entry` | **exact candidate, pending evidence**. The scheduled kind ignores flags and returns the drag result. |

## Handwritten-header authority

The include guard `_RIVE_TEXT_INPUT_HPP_` is **not applicable**: it is build
structure with no runtime effect.

| # | Pinned inline definition | Concrete Rust owner | Candidate disposition and evidence/blocker |
|---:|---|---|---|
| H1 | `rawTextInput()` (29) returns the occurrence-owned raw editor. | `components.rs:963::RuntimeTextInputState::raw` | **adapted (Rust ownership), pending evidence**. `RefCell<RawTextInput>` replaces a mutable raw pointer while retaining one clone-owned occurrence. |
| H2 | `focusableArtboard()` (60) returns the owning Artboard. | Focus nodes carry `ArtboardInstance::instance_identity`; nested dispatch uses the owner identity in `state_machine_instance/text_input_listener_group.rs:15-43`. | **adapted (Rust ownership), pending evidence**. No borrowed Artboard pointer crosses the arena boundary. |
| H3 | `acceptsKeyboardInput()` (61) always returns true. | `state_machine_instance.rs:2197::publish_focusable_keyboard_capabilities` | **incorrect/incomplete**. Rust derives the capability only from registered keyboard listener groups; the TextInput's own listener-group registration does not independently set it true. Blocker: focused TextInput with no authored keyboard listener must report keyboard acceptance. |
| H4 | `isDragging()` (87) returns the retained bool, default false. | `RuntimeTextInputState::is_dragging`; readers in `advance_text_input_entry` | **exact candidate, pending evidence**. |
| H5 | `isFocused()` (90) returns the retained bool, default false. | `RuntimeTextInputState::is_focused`; `text_input.rs:106::text_input_is_focused` | **exact candidate, pending evidence**. |

## Retained fields, defaults, and order dependencies

`RuntimeTextInputState::default` preserves the source defaults for null style
and constraint, false focus/drag, NaN last drag position and layout width, and
zero scroll velocities. `source_text: None` is Rust's pre-`onAddedClean`
sentinel; C++'s string is value-initialized empty. `world_bounds: None` is the
Rust projection of an empty AABB. The raw editor and source string clone with
each occurrence.

The following order constraints are not transitive implementation details and
must be retained during correction:

1. `onAddedClean` resolves style/source/font and display before multiline
   configuration and its MissingObject result.
2. property callbacks update raw/source state before dirt and generated
   property-listener notification;
3. recognized key handling performs the raw operation, source synchronization
   when applicable, then the key-specific dirt unconditionally;
4. drag state/last position changes precede coordinate conversion;
5. update rebuilds raw geometry and selection effects before automatic caret
   scrolling;
6. edge-scroll advance stops physics, applies X then Y, then re-hits the last
   finite pointer and publishes Paint.

## Packed ownership and proposed split

No source split is implemented in this candidate. The primary 977-line Rust
file already has the correct file-level identity and is not the original
multi-upstream "uni-file" failure. Moving code now would mix a mechanical
refactor with unresolved translation errors.

After the incorrect/missing rows are corrected, the smallest behavior-neutral
cleanup is:

1. keep source-owned constants, helpers, and `ArtboardInstance` TextInput
   methods in `text_input.rs`;
2. move only tools/test observers and the two local tests to
   `text_input/debug.rs` behind the existing cfg;
3. keep arena state in `components.rs` and ScrollConstraint algorithms in
   `constraints.rs`, but document them as narrow dependency owners rather than
   attributing either whole file to TextInput;
4. keep `advance_text_input_entry` in the existing artboard schedule module,
   because moving it would not reduce ambiguity and would increase borrow and
   visibility churn.

This proposal makes the primary source comparison shorter without pretending
that shared arena, layout, scroll, and focus dependencies belong to the C++
translation unit.

## Candidate verdict

This pair is **not at exact source parity**. The first complete read found at
least these actionable source discrepancies:

- missing direct/general `hitTest` evidence and incomplete `hitTestPoint` flag
  preservation;
- selection-radius dirt mismatch;
- incomplete initial `onAddedClean`/`updateMultiline` behavior and missing
  TextStyle failure result;
- missing wasm Windows platform modifier detection;
- recognized key, focus, drag, selection, and edge-scroll callbacks incorrectly
  gate dirt on a state change;
- multiline Enter publishes the wrong dirt family;
- missing production `worldPosition` and `worldBounds` owners; and
- `acceptsKeyboardInput` is not unconditionally true for TextInput.

The 20 pending `text_input_test.cpp` cases remain blockers/consumers, not proof
that these source rows are complete. Production behavior was not changed and
no test was added in this audit. A fresh reviewer must read the complete pair
and try to falsify every mapping before any correction begins.
