# B-6 structural audit second pass

Date: 2026-07-24

C++ pin: `d788e8ec6e8b598526607d6a1e8818e8b637b60c`

Rust baseline: `a8f7d5f2`

This is the disposition record for the 36 rows that the initial audit could
not classify and the five mechanism families left for post-RB-1/RD-1 review.
It supersedes the initial verdict for the rows named below. It does not promote
any file-correspondence row: `status` and `verification` remain subject to the
orchestrator-only promotion rule.

## Closure rule

The initial three architectural verdicts could not honestly classify a C++
lifecycle that Rust simply did not implement: `DIVERGENT` requires a live
mutation-gated Rust compensation mechanism, while `N/A` means the C++ row has
no applicable runtime seam at the pin. The second pass therefore adds
`TRACKED-GAP`. It means:

1. the C++ lifecycle and the missing/incomplete Rust surface were both
   identified;
2. no compensation mechanism was invented to force a `DIVERGENT` verdict; and
3. an existing F/A/C/RB register item owns the unfinished implementation.

`TRACKED-GAP` closes the audit decision, not the implementation gap.

## Final verdict census

| Verdict | Rows |
|---|---:|
| ISOMORPHIC | 19 |
| ADAPTED | 192 |
| DIVERGENT | 157 |
| TRACKED-GAP | 30 |
| UNKNOWN | 0 |
| N/A | 49 |
| **Total** | **447** |

The change from the initial census is exactly:

- 30 UNKNOWN → TRACKED-GAP;
- 5 UNKNOWN → ADAPTED;
- 1 UNKNOWN → N/A; and
- 5 DIVERGENT mesh/slice rows → ADAPTED after RD-1.

## Former UNKNOWN rows

### Animation and input

| Row | C++ file | Final verdict | Evidence and owner |
|---|---|---|---|
| B6-0027 | `focus_listener_group.cpp` | TRACKED-GAP | Rust still filters the focus listener family from the imported state-machine listener set. Owned by RB-2 and F5. |
| B6-0046 | `listener_align_target.cpp` | TRACKED-GAP | There is no `RuntimeScheduledListenerAction` equivalent for align-target dispatch. Owned by F13. |
| B6-0047 | `listener_bool_change.cpp` | TRACKED-GAP | Direct bool writes exist, but nested input resolution remains outside the implemented listener-change path. Owned by F13. |
| B6-0049 | `listener_input_change.cpp` | TRACKED-GAP | The shared nested-input listener lifecycle remains incomplete. Owned by F13. |
| B6-0050 | `listener_invocation.cpp` | TRACKED-GAP | Rust listener invocation covers pointer/reported-event/none, not the pinned keyboard, text, focus, gamepad, semantic, and view-model family. Owned by F5/F13. |
| B6-0051 | `listener_number_change.cpp` | TRACKED-GAP | Direct number writes exist, but nested input resolution remains incomplete. Owned by F13. |
| B6-0052 | `listener_trigger_change.cpp` | TRACKED-GAP | Direct trigger writes exist, but nested input resolution remains incomplete. Owned by F13. |
| B6-0067 | `property_recorder.cpp` | ADAPTED | C++ snapshots a mutable source artboard before component-list cloning and reapplies the baseline. Rust constructs every mounted occurrence from the immutable `ArtboardGraph` prototype, so the same baseline is obtained without a recorder stream. AF-6 applies: mutable instance copies are explicit and prototype state is never the live occurrence. |
| B6-0246 | `joystick.cpp` | TRACKED-GAP | Rust retains joystick animation/dependent identity and apply order, but still lacks C++ `Joystick::update` world-space handle-source-to-axis settlement. Owned by F9. |

### Assets and importers

| Row | C++ file | Final verdict | Evidence and owner |
|---|---|---|---|
| B6-0098 | `audio_asset.cpp` | TRACKED-GAP | Rust imports the descriptor but has no audio decode/playback owner. Owned by F1/A2. |
| B6-0103 | `font_asset.cpp` | TRACKED-GAP | Embedded and host-attached fonts shape correctly, but the C++ asset-owned decode plus referencer notification lifecycle is not present as a `FileAssetLoader` callback. Owned by A1. |
| B6-0104 | `image_asset.cpp` | ADAPTED | RD-1 retains one decoded `RenderImage` owner per file-asset global identity and shares that owner with image occurrences. AF-1 and RF-27/RF-28 apply; renderer allocation may be factory-late, but file-asset identity is retained. |
| B6-0106 | `script_asset.cpp` | ADAPTED | The mapped implementation is the high-level file script catalog plus `nuxie-scripting`, not `objects.rs`: imported `ScriptAsset` bytes produce retained module/protocol VM owners shared by runtime occurrences. AF-1 and AF-5 apply. |

### Constraints

| Row | C++ file | Final verdict | Evidence and owner |
|---|---|---|---|
| B6-0127 | `draggable_constraint.cpp` | TRACKED-GAP | Static constraint math exists, but the component-provided listener group and start/drag/end pointer lifecycle do not. Owned by F4/F5. |
| B6-0134 | `clamped_scroll_physics.cpp` | TRACKED-GAP | Clamping math is present; the full retained physics/drag lifecycle still needs the registered fixture. Owned by F4/F10. |
| B6-0139 | `scroll_constraint_proxy.cpp` | TRACKED-GAP | Interactive proxy ownership and forwarding remain partial. Owned by F4. |
| B6-0140 | `scroll_physics.cpp` | TRACKED-GAP | The retained velocity/time/drag physics lifecycle is incomplete. Owned by F4. |

### Lua and scripted runtime

| Row | C++ file | Final verdict | Evidence and owner |
|---|---|---|---|
| B6-0260 | `logging_scripting_context.cpp` | TRACKED-GAP | The VM has no host-provided line-buffered info/error sink equivalent. Owned by F7. |
| B6-0267 | `lua_listener_invocation.cpp` | TRACKED-GAP | Rust exposes pointer/reported-event/none invocation values, not the complete keyboard/text/focus/gamepad/view-model Lua wrapper family. Owned by F5/F7. |
| B6-0270 | `lua_rive_base.cpp` | TRACKED-GAP | The pinned `_G.print` routing through `ScriptingContext` has no Rust host-sink counterpart. Owned by F7. |
| B6-0321 | `scripted_data_converter.cpp` | ADAPTED | RB-1 left each artboard with retained scripted-converter VM instances and the retained data-bind graph invokes forward/reverse conversion through them. AF-1/AF-5 apply to the VM table and imported converter catalog. |
| B6-0323 | `scripted_interpolator.cpp` | TRACKED-GAP | No per-animation/keyframe scripted interpolator clone and `transform`/`transformValue` lifecycle exists. Owned by C1/F7. |
| B6-0324 | `scripted_layout.cpp` | TRACKED-GAP | RD-1 ported live draw traversal, but `measure`, `resize`, and layout dirt ownership remain absent. Owned by F7. |

### Math and containers

| Row | C++ file | Final verdict | Evidence and owner |
|---|---|---|---|
| B6-0295 | `mat2d_find_max_scale.cpp` | N/A | The helper has no caller in pinned C++ source or headers, so it has no live runtime relationship to audit at this pin. |
| B6-0301 | `rectangles_to_contour.cpp` | TRACKED-GAP | C++ uses it for text-value and selection contours; Rust has no live TextInput selection contour owner. Owned by F2. |
| B6-0339 | `list_path.cpp` | TRACKED-GAP | The concrete type still relies on generic path handling without its required parity fixture. Owned by F10. |
| B6-0375 | `simple_array.cpp` | ADAPTED | The `.cpp` body contains testing-only allocation counters; production ownership is the header `SimpleArray<T>`. Rust's owned `Vec`/slice storage is the AF-7 unique-owner mapping. |

### Text

| Row | C++ file | Final verdict | Evidence and owner |
|---|---|---|---|
| B6-0378 | `cursor.cpp` | TRACKED-GAP | Rust exposes query-local caret geometry, not a retained editable cursor. Owned by F2. |
| B6-0384 | `raw_text_input.cpp` | TRACKED-GAP | The mutable edit buffer, journal, cursor, selection, and undo/redo lifecycle are absent. Owned by F2. |
| B6-0388 | `text_input.cpp` | TRACKED-GAP | Rendering is live, but focus/key/drag/scroll/edit state is absent. Owned by F2/F5. |
| B6-0389 | `text_input_cursor.cpp` | TRACKED-GAP | Rust still draws placeholder cursor geometry rather than the retained live cursor path. Owned by F2. |
| B6-0391 | `text_input_selected_text.cpp` | TRACKED-GAP | Selected-text path ownership and range-only draw are not implemented. Owned by F2/C1. |
| B6-0392 | `text_input_selection.cpp` | TRACKED-GAP | The live selection path is absent. Owned by F2. |
| B6-0398 | `text_selection_path.cpp` | TRACKED-GAP | Rectangle-to-contour construction and retained rounded selection path are absent. Owned by F2. |
| B6-0401 | `text_style_feature.cpp` | TRACKED-GAP | OpenType feature children are recognized by schema metadata but not retained by the runtime text style or passed to shaping. Owned by C1/F13. |
| B6-0406 | `text_variation_modifier.cpp` | TRACKED-GAP | The type is recognized but rejected by modifier-group reconstruction; no live axis-value propagation exists. Owned by C1/F13. |

## Five pending mechanism families

| Family | Rows | Final disposition |
|---|---|---|
| Mesh/slice vertex snapshots | B6-0249, B6-0255, B6-0340, B6-0341, B6-0370 | **Resolved by RD-1; ADAPTED.** The old compare-to-`last_vertex_bytes` / `last_update` / `input_words` / `world_bits` lifecycle is gone. Clone-owned `RuntimeMeshOwner` and `RuntimeSliceMeshOwner` receive C++ dirt directly. Their owner-local settled CPU bytes are RF-28 state used only to materialize/rematerialize factory-late backend buffers, not a source-drift comparison. |
| Deferred script advance queue | B6-0322, B6-0325, B6-0326 | **Confirmed DIVERGENT; RB-3.** `ArtboardInstance::queue_script_advance` still stores elapsed steps and the high-level facade later flushes them with a factory; C++ calls the retained object's script advance directly in component advance. |
| Script-input scalar rehydration | B6-0315–B6-0318 | **Confirmed DIVERGENT; RB-4.** Import-time kind/name/default metadata is valid AF-5, but `rehydrate_script_listener_actions` still scans and hydrates bound scalar values at the scene rebind boundary instead of retaining the C++ `ScriptInput`/`DataBindContext` push relationship. |
| Solid-color revision handoff | B6-0355 | **Confirmed DIVERGENT; RB-5.** `solid_color_paint_revisions` is still written by runtime property changes and consumed later by drawing. C++ mutates the attached `RenderPaint` in `SolidColor::colorValueChanged`; RF-28 allows delayed backend allocation, not delayed paint-state ownership. |
| Focus target lookup rebuild | B6-0209, B6-0238, B6-0240 | **Confirmed DIVERGENT; already owned by RB-2.** `RuntimeFocusTree::sync` still projects copied descriptors and rebuilds `target_nodes`; C++ retains live `Focusable`/`FocusData` relationships. |

## Mechanical closure

`make b6-audit-check` enforces:

- exactly 447 unique B-6 rows;
- the pinned C++ ref;
- zero `UNKNOWN` verdicts;
- the final verdict census above;
- every exact second-pass row disposition recorded above;
- an owner token on every `TRACKED-GAP` row; and
- every second-pass row continuing to cite this record.
