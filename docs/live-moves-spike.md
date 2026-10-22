# Live moves runtime spike

UNIV-5112. 21 October 2026. Branch `claude/live-moves-spike`.

This spike transports a live nested unit inside one player. It keeps the child graph and moves the playback instances behind the destination's own wrappers. The runtime and C API fixtures exercise a page with an overlapping `screens` list and two Screen rows. The destination sits below a clipping parent with a keyed offset. The flight sits outside that parent.

The main implementation is `crates/nuxie-runtime/src/live_move.rs`. It contains the atomic move, frozen layout leaves, flight following, destination prop binding adoption, row collapse, and the per-player `LiveMoves` controller. The ported files expose ownership fields and small layout, composition, and row-skip hooks. Ordinary hosts have no move state or transform override in use. Their existing mount and animation paths stay in place.

`crates/nux-capi/src/live_moves.rs` exposes:

- `nux_player_set_moves(player, json)`.
- `nux_player_step_begin(player, incoming_screen_id, transition_id)`.
- `nux_player_step_end(player, incoming_screen_id, transition_id)`.
- `nux_player_set_row_collapsed(player, list_name, row_index, collapsed)`.

Strings use `NuxStringView`. A null transition pointer means no flight. The calls use the existing thread, handle, occurrence, reentrancy, and render-revision guards. Malformed moves, unresolved paths, incompatible wrappers, and invalid step boundaries return `InvalidArgument`. JSON is bounded to 1 MiB. The generated header and portable v4 export list include the four calls. The `nuxie` Rust facade re-exports the new module.

The SDK must mount and expand the incoming row before `step_begin`. It must call `step_end` before collapsing the outgoing row. `set_moves` empties known flight slots that are already mounted. The post-advance hook prepares newly mounted rows. A flight uses the `progress` child node's x when present. Otherwise Rust callers set `Flight::progress`. The `path` child adds a point offset. This spike does not add a separate C progress setter.

A move validates player membership, unit definition, wrapper types and animation IDs, and nested machine input structure before taking the child. The destination releases its wrapper dependencies before its fresh instance is dropped. The source loses its instance and playback. No machine initialization, authored input reapplication, stateful bind, context clear, or Lua initialization runs on the transported unit.

The move reparents the existing data context. It rehomes the existing focus scope and repairs ancestor focus flags without blurring the field. It reparents the semantic tree and replaces the old nested event/listener host references. It also carries the accumulated quantization time and the active VM ownership fields. Parent-owned occurrence prop binds are detached from the old Screen. Saved destination templates target the transported VM's corresponding properties. Unbound values stay on the same objects.

The player remembers every departed slot, including departures during immediate coming-home moves. It finds the current holder by instance identity. A cut can therefore bring a unit home from another row without a flight. The controller retains original VM arenas and their owning artboards outside the scene graph. This avoids arena ownership cycles and keeps an explicit occurrence VM alive if its original row is removed.

An empty layout slot uses a tiny artboard with only a root layout node. Its frozen style keeps the last measured box. It is never enumerated as mounted content, advanced, drawn, or hit. This is separate from the transported artboard and from the discarded destination instance.

Flight following runs after the player's existing advance and layout. It reads both live endpoints, interpolates translation, shortest-path rotation, scale, size, and world opacity, then converts through the flight parent's world matrix. Layout mode changes the unit's size. Scale mode keeps its departure box and scales that box. A full local matrix override preserves the skew that can arise when inverting a rotated, nonuniformly scaled parent. The second pass updates the flight row without advancing playback.

An ordinary list can collapse only as a whole. The spike adds collapse for one non-virtual row. Its mounted instances remain addressable. A bitmap skips collapsed rows in advance, settlement, and data binding. Ordered draw/hit indices and layout node collection also exclude them. The existing inactive change sequence is marked while rows are skipped. The wake test changes data while a row is collapsed and observes it after expansion.

The shared literal fixture builder is `crates/nuxie-runtime/src/live_move_fixture.rs`. It is public only with the `test-support` feature. The `nuxie` facade re-exports it under the same feature. Both test suites use this module directly. The runtime has no `runtime-test-support` dev dependency. The C API retains its main dev dependency on `runtime-test-support` for the existing Apple authored shader fixture and Android GPU recorder. It has no local copies of those helpers. The shared test-support package has no live-move module. It emits a Rive file directly. The unit has a state machine playing a one-second looping x timeline, a separate live y timeline wrapper, private `count`, a bound `prop`, a TextInput, FocusData, a pointer listener, visible geometry, and a Video. The C API imports the same bytes with explicit video capability admission.

Round 1 found a test dependency problem. The initial fixture helper pulled the renderer graph into runtime test targets. That selected a second runtime build without `tools`, so the layout callback test could not compile. Moving the schema-only fixture into the runtime removes this edge. The next full run exposed the same missing test feature in the clone chronology callback test. The runtime now declares a self dev-dependency with `tools` and `test-support`, following the facade pattern. This enables test-only callbacks and the new fixture for package-specific test runs. Shipping defaults stay unchanged. Generated Bazel manifests and targets were regenerated with `python3 tools/bazel/generate.py`. The generated-file check passes.

The numeric checks are in `crates/nuxie-runtime/tests/live_moves.rs` and `crates/nux-capi/src/live_move_tests.rs`.

| Requested proof | Numeric result |
| --- | --- |
| Continuity | At 0.3 s, keyed x and y are 30. Ten further 0.02 s frames through the flight produce 50. A reverse flight for 0.1 s produces 60. |
| State | Private `count` stays 7. Text stays `hello mascot`. VM and data-context identity stay equal. The destination bound prop becomes 200, then follows a later destination write to 321. |
| Advanced once | Ten frames add exactly 0.2 s, observed as 20 points on both independent timelines. The empty source issues 0 geometry draws, compared with 1 before takeoff. |
| Size | A sibling stays after the retained layout box while the instance is away and after it returns. The final fixture uses a 150 by 80 slot around a unit authored at 100 by 80. |
| Follow | Endpoint progress is checked at 0, 0.5, and 1. B's destination x is 215, 225, and 235 in those same advances. Mid-flight rotation takes 170 degrees to 190 degrees through 180 degrees. Scale is 3 by 4. Opacity is 0.6. Layout size is 150 by 100 between 100 by 80 and 200 by 120. A fill child reflows to width 150. Reverse Scale mode keeps 200 by 120 and scales by 2.25 and 3.333333. |
| Timeline controls | A child progress value of 0.25 overrides API progress 0.9. Path offset is 7 by 11 points. Changing the child progress to 1 follows the destination from that same advance. |
| Hits | A destination pointer changes the moved machine's input from 0 to 9. After resetting it, a pointer at the empty source leaves it at 0. |
| Identity | All 12 unit objects retain their CoreHandle identities. The machine handle also remains equal. Video ID is 1073741824 before takeoff, in flight, after landing, and after coming home. |
| Coming home | A no-flight step returns the exact original instance to A. Another no-flight step returns it to B. No replacement is created. |
| Focus | The same focused leaf survives takeoff, landing, and collapse of the outgoing row. |
| Collapsed rows | Forty active rows issue 81 artboard advances and 40 geometry draws. With 39 collapsed, the counts are 3 and 1. The root, one Screen, and one unit account for the three advances. A collapsed row's keyed x does not move. A data write of 777 is observed on waking. |
| C API | `set_moves`, `step_begin`, a 0.2 s player step, and `step_end` land the same unit at x = 50. Null-transition begin/end brings it home. Row collapse and expansion both return `Ok`. |
| Origin-row lifetime | The original row is removed after landing. The transported VM remains alive and accepts a private count write from 7 to 8. |

The round 1 live-move test measured 100 advances at 46.67 ms with all 40 rows active, 1.83 ms with 39 collapsed, and 1.85 ms for the one-row baseline. These are host timings from this fixture, not device budgets. The exact work counts are the stronger result. Timing is printed on every test run.

Validation used the supplied request-file runner. No Cargo or Bazel command was invoked directly. No commit or push was attempted. All 232 runtime test targets built. The completed full run includes 122 executed targets and 110 cached results. It has 223 passing targets and 9 failing targets. All 60 failing cases reproduce on the pinned main source with identical failure names and counts.

| Full command | Final result | Runner ID |
| --- | --- | --- |
| `test -p nuxie-runtime` | 2554 passed, 60 failed, 11 ignored. All 232 targets completed. | `round1runtime4` |
| `test -p nux-capi` | 211 passed, 0 failed, 0 ignored. All 20 targets passed. | `round1capi1` |
| `check -p nux-capi --all-targets` | All 24 targets built. | `round1check1` |

The full runs include all 12 runtime live-move cases and both C API live-move cases. The layout callback and clone chronology suites each pass 6 cases. The constrained nested-artboard suite passes 6 cases with the fetched sync assets. The change-window suite now runs and passes 18 cases. The focus suite passes 109 cases. The nested opacity suite passes 3 cases.

The generated Bazel-file check, C API surface guard, and diff whitespace check pass after restoring the spike. Symbol inspection confirms all four new exports in the built dynamic library. The Android runtime contract and C API distribution suites pass 46 tests. The Bazel build-tool suites pass 48 tests.

The full C API run passes 211 cases across 20 test targets. It has zero failed or ignored cases. Five platform or feature-gated targets contain zero cases under this command. The unit-test target includes both live-move cases and the previously environment-blocked player globals and value-policy cases. The focus-input target passes all 6 cases. The runner now exports `RIVE_RUNTIME_DIR=/Users/levi/dev/oss/rive-runtime`. The C API all-target check passes 24 targets.

The completed runtime result above includes all targets. The earlier incomplete attempts are not added to its totals. Two early attempts stopped for lack of disk space. The first linker error was `errno=28`. The second attempt could not create a test log. Only regenerable artifacts in this worktree's `target/debug` were removed. The completed retry used one build action at a time to reduce peak disk use. The original build configuration has been restored byte for byte.

The main comparisons pin commit `7daf0bd71dc27f1a18d29179c318b67fd4af38a6` from the start of this review run. Its only commit after this branch's base changes CI. The tracked spike files were temporarily replaced with that main source. The same `tools` and `test-support` self dev-dependency was kept as a test harness setting. All 9 failing suites were run through the supplied runner. All 60 failure names and their counts match. After each comparison batch, all saved spike files were restored and checked byte for byte.

| Existing suite | Spike passed / failed | Main passed / failed | Main runner evidence |
| --- | --- | --- | --- |
| `upstream_text_measure_key_5551696a` | 3 / 2 | 3 / 2 | `round1main1` |
| `cpp_probe_audio_native` | 1 / 2 | 1 / 2 | `round1main2` |
| `cpp_probe_host_contracts` | 7 / 1 | 7 / 1 | `round1main3` |
| `cpp_probe_data_bind_native` | 42 / 10 | 42 / 10 | `round1main4` |
| `upstream_wave_b_expected_red` | 251 / 1 | 251 / 1 | `round1main5` |
| `upstream_data_binding_keyframes` | 6 / 1 | 6 / 1 | `round1main6` |
| `cpp_probe_transitions_native` | 7 / 34 | 7 / 34 | `round1main7` |
| `upstream_frozen_pose` | 0 / 7 | 0 / 7 | `round1main8` |
| `upstream_legacy_script_hit_test` | 0 / 2 | 0 / 2 | `round1main9` |

The text cases are `a_variation_modified_text_re_solves_its_hug_layout_once` and `a_variation_modifier_change_reshapes_its_text`. Both fail at line 25 while reading `tests/unit_tests/assets/text_variation_modifier_hug.riv` from the upstream checkout. That file is absent. They do not reach the runtime assertions.

The other fixture failures are the draw-target case, one keyframe lookup case, all 7 frozen-pose cases, and both legacy script hit-test cases. The absent files are `draw_target_missing_drawable.riv`, `frozen_pose.riv`, `frozen_pose_rows.riv`, `reverse_state_freeze.riv`, and `legacy_script_hit_test.riv`. Together with the text variation fixture, these account for 13 failing cases. Each failure occurs while reading a file.

The two audio cases fail after `make` reports `No rule to make target cpp-probe` inside Bazel runfiles. The host-contract case is `built_probe_reports_current_source_fingerprint`. It requires the absent C++ oracle. The ten random-function data-bind cases and 34 transition cases also require that oracle. These account for the other 47 failing cases. All reproduce on main. Several other C++ comparison cases return early without the oracle. Their passing Rust test status does not prove a C++ comparison.

This needed more than swapping the nested artboard handle. Playback belongs to wrappers in the host's arena. Explicit occurrence VMs and their bindings can also belong to the Screen arena. Moving just the child either restarts playback, leaves old bindings writing to it, or loses its VM when a row is released. Frozen layout nodes and focus ancestor repair were also necessary. Retaining only an arena failed the origin-row removal test because `Artboard::drop` explicitly retires its objects. The controller now retains the owning artboard too. The stronger reflow test also showed that updating only the flight host leaves the unit's layout and paints stale. The follower explicitly settles the transported subtree. Existing unit data binds are not rebound, since a scripted converter context bind can reinitialize Lua.

This is a runtime spike, not the whole published Magic Move value model. Destination number binding adoption is proved. Public number/color tweening and writing unbound typed text into a newly introduced destination binding are not implemented. The unbound text preservation case is proved. Stateful Lua continuity is supported by retaining the graph and avoiding initialization, but there is no dedicated Lua fixture in this spike. Individual collapse rejects virtualized lists. The page host uses a non-virtual list.

The raw Rust `move_instance` operation requires its owning row artboards to remain alive. The per-player controller supplies that lifetime retention. The current controller keeps those VM arenas and owning artboards until player destruction. Pruning retained arenas as history and units disappear is a follow-up. The move assumes the compiler supplies immutable, matching unit definitions and wrappers. This is not a general subtree extraction API. Slot paths traverse nodes in one Screen arena. Cross-artboard path traversal is not implemented.

The next step is the iOS and Android device harness. Verify the same decoder object, playback clock, audio, and video texture survive all three moves. Verify the same native text editor and IME composition session survive focus rehoming and outgoing-row collapse. A stable runtime video ID alone does not prove either platform's decoder reconciliation. Also exercise accessibility node IDs and actions, nested lists and scrolling inside a unit, bound size changes, nonzero origins, multiple simultaneous units, and removal of old Screens under real navigation. Decide the destination binding conflict policy and typed-text writeback before implementing value interpolation. Measure retained arena memory as the back stack grows and shrinks.

The round 1 full-suite commands are `test -p nuxie-runtime`, `test -p nux-capi`, and `check -p nux-capi --all-targets`. Their runner IDs are `round1runtime4`, `round1capi1`, and `round1check1`. Read `result-<id>.log` and `done-<id>` in the supplied `scratchpad/runtime-runner` directory. The main comparison IDs are in the table above. The earlier incomplete runtime IDs are `round1runtime1`, `round1runtime2`, and `round1runtime3`. The first exposed the missing test feature. The next two stopped for disk space.


Round 2 restores nux-capi's existing `runtime-test-support` dev dependency. Its Android import helper and Apple authored GPU canvas helper match main. The two local helper copies and the local recorder module hook are removed. The nux-capi Cargo manifest, BUILD file, and both flattened Bazel Cargo manifests match main byte for byte. The live-move fixture remains in the runtime behind `test-support`, reached through the `nuxie` facade. The shared test-support package has no live-move module.

| Round 2 command | Result | Runner ID |
| --- | --- | --- |
| `test -p nux-capi` | 211 passed, 0 failed, 0 ignored. All 20 targets passed. | `round2capi1` |
| `check -p nux-capi --all-targets` | All 24 targets built. | `round2check1` |
| `test -p nuxie-runtime --test live_moves` | 12 passed, 0 failed, 0 ignored. | `round2moves1` |
| `test -p nuxie-runtime --test source_registry_layout_callbacks` | 6 passed, 0 failed, 0 ignored. | `round2layout1` |

All four requests return exit code 0. The generated-file check, surface guard, and diff whitespace check also pass. No Cargo or Bazel command was invoked directly. No commit was made.


Round 3 fixes the duplicate semantic identities seen on the Bat Phone after the first takeoff. The device evidence is `live-moves-device-check/data/iphone-20261022T014507Z-97384/checks.log` in the handoff repository. It reports `duplicateIdentity(4)` immediately after `step_begin`. The new C API regression reproduced the same failure before the fix. Its first flight snapshot contained 27 entries but only 10 unique IDs. The moved group appeared under both the source and flight slots. Rebuilding also appended its existing children again. The failing runner IDs are `round3capired1` and `round3red2`.

The fix is in `crates/nuxie-runtime/src/live_move.rs`. When the source and destination share a semantic manager, the move removes the existing boundary from its old parent and attaches it once under the destination. It preserves the boundary, all descendants, their registrations, and their IDs. It marks the boundary transform dirty so semantic bounds follow the current host. It does not rebuild the live subtree. The ordinary cleanup and build path handles first semantic mounts or a different nested manager. No ported upstream files changed.

`live_move_fixture::semantic_fixture` adds explicit semantic nodes for each slot and for the unit's group, TextInput, and button. The runtime regression checks the manager snapshot and internal boundary parent. The C API regression calls `nux_player_semantic_snapshot`, `nux_semantic_snapshot_info`, and `nux_semantic_snapshot_node`, which are the SDK snapshot path. It acknowledges the current render revision at each boundary without advancing between the move and snapshot. It also reads the live field text through `nux_player_field_string_copy` using the original semantic ID.

Each regression checks 10 snapshots in each of two cases, focused and unfocused. Each case performs seven moves. It covers first takeoff, a frame in flight, landing, a direct return home, another outward flight, and a return flight. Every snapshot has unique IDs. The unit has exactly three exported semantic nodes, each present once under its current slot. The internal boundary is omitted from exported snapshots and is checked separately by the runtime test. The C API snapshot has 10 nodes before and during the first flight, then 7 after discarding the destination's fresh instance. The original TextInput ID remains stable across all seven moves. Its text stays `hello mascot`, or 12 UTF-8 bytes. Its focused bit stays 128 in the focused case and 0 in the unfocused case. The same focus leaf and focused TextInput survive every boundary.

| Round 3 command | Result | Runner ID |
| --- | --- | --- |
| `test -p nuxie-runtime --test live_moves` | 13 passed, 0 failed, 0 ignored. | `round3moves2` |
| `test -p nux-capi --lib live_moves` | 3 passed, 0 failed, 0 ignored. | `round3capi3` |
| `test -p nux-capi` | 212 passed, 0 failed, 0 ignored. All 20 targets passed. | `round3capifull1` |
| `check -p nux-capi --all-targets` | All 24 targets built. | `round3check1` |
| `test -p nuxie-runtime --test upstream_semantic_artboard_swap` | 8 passed, 0 failed, 0 ignored. | `round3semantics1` |
| `test -p nuxie-runtime --test upstream_semantic_data_lifecycle` | 13 passed, 0 failed, 0 ignored. | `round3semantics2` |
| `test -p nuxie-runtime --test semantic_focus_runtime` | 10 passed, 0 failed, 0 ignored. | `round3semantics3` |

All seven final runner requests return exit code 0. The runtime row timing check measured 47.85 ms for 100 advances with 40 active rows, 2.07 ms with 39 collapsed, and 1.84 ms for the one-row baseline. Advance counts remain 81 versus 3. Geometry draw counts remain 40 versus 1.

The surface guard, Rust formatting check, and diff whitespace check pass. The generated-file check reports one stale file, `crates/nuxie-runtime/BUILD.bazel`. Comparing its generated content shows only the missing target for the pre-existing untracked `tests/live_moves_device_fixture.rs`. That throwaway device fixture and the tracked BUILD file are untouched. This fix adds no manifests, dependencies, exports, headers, or target declarations.

The first attempt used the wrong fixture property name, `roleValue`, and failed before reaching a move. It was corrected to `role`. The first fix build also caught an ArtboardInstance closure signature mismatch. Both are resolved in the final runs above. The two red regression runs listed above show the actual duplicate snapshot failure before the semantic rehome fix. No final test failure remains.

The iOS device harness has not been rerun here. Repeat the Bat Phone check and the Android harness with the fixed library. Verify that the native text editor, IME composition, and accessibility elements stay attached to the stable semantic IDs through every boundary. The snapshot tests prove the runtime and C API contract. They do not prove platform reconciliation. Typed-text writeback from the device finding remains a separate open question. No Cargo or Bazel command was invoked directly. No commit was made.
