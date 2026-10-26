# Live moves runtime core

UNIV-5289. Core PR, sections 1 to 7 of the runtime design. Section 8 occurrence values are described below. The corrected sections 1, 3 and 4 are implemented.

The plan is in the RIV file. There is no JSON move-list API. A player takes units into the exact flight slots named by its file records, follows the two endpoints after its normal advance, and lands the same instances. A cut returns borrowed instances to the incoming Screen immediately. A two-version incoming instance that is still held on a third Screen also comes home before taking off from its incoming slot. All planned returns and flights are checked before changing ownership.

The shared Rust page-player follow-up, record-free page opt-in, pending list edits and content-height query are documented in [page-player.md](page-player.md).

## Owners and integration

| Owner | Work |
| --- | --- |
| `defs/nuxie/live_slot.json`, `live_step.json`, `live_step_unit.json` | Published records. Types 62000 to 62002 and properties 62000 to 62014. The remaining allocation is reserved in `defs/nuxie/README.md`. |
| `crates/nuxie-schema/src/generated/schema.rs` | Generated record metadata. Progress, the two path offsets and fade are keyable doubles. |
| `crates/nuxie-runtime/src/live_move/records.rs` | Record creation, cloning, deserialization, lifecycle validation, and registry access to flight controls. Invalid parents produce a diagnostic and are dropped. |
| `live_move/transport.rs` | Atomic instance and playback transport, binding adoption, focus repair, semantic reparenting, and slot-owned frozen layout leaves. |
| `live_move/plan.rs` | Published-step discovery, two-version steps, coming home, current-holder history, and pruning on membership changes. |
| `live_move/follow.rs` | One draw-pose projection shared by both endpoints and the flight parent, shortest rotation, scale or layout sizing, opacity, path offsets, and update-only settlement of flight rows. |
| `live_move/collapse.rs` | Non-virtual row sleep through upstream component collapse, destination reconciliation, and wake. Item identity carries sleep across list reorders. |
| `crates/nuxie-runtime/src/hosted_scene.rs` | The occurrence-owned hosted-instance index and its shared active-row projection. Membership stamps are scoped to the root instance. |
| `live_move/frame.rs`, `host_viewmodel/transactions.rs` | Active VM graph projection and a lazy rollback checkpoint for live-move frames. |
| `crates/nux-capi/src/live_moves.rs`, `lib.rs::player_operation_body` | Decode the three live-move C calls and run them inside the same guarded mutation boundary as player steps. The generated header and portable v4 exports include them. |
| `crates/nux-capi/src/video.rs`, `player_view_models.rs` | Indexed video enumeration, constant-time occurrence lookup, and indexed bound-owner capture. |
| `crates/nuxie-runtime/src/host_state_machine/nested_events.rs` | Event drains use the active row projection. This was the remaining step cost. |
| `live_move_fixture.rs`, `live_move/device_fixture.rs` | Literal RIV builders, compiled only with `test-support`. Both crate test suites use them. |

The mechanical port contains small hooks for ownership access, frozen layout nodes, transform composition, active list rows, membership invalidation, semantic validation, and the lazy checkpoint. The hooks are commented. Visit counters, their types, and their storage exist only with `test-support`.

## State and lifetime

Live-move state belongs to the scene occurrence. Every C API player created from the same `NuxArtboardInstance` shares the native occurrence's plan, hosted-instance index and frame cache revision. Its media bridge shares the video ID registry. Record-free native occurrences allocate none of this live-move state. The existing occurrence guard serializes their mutations. Flights, rows and coming-home history remain available when a player handle closes. A player's selected playback and explicit global VM context remain player-owned.

Rust `LiveMoves` adapters and C API entry points dispatch into that native occurrence state. Keeping an adapter alive does not keep a plan, cache or retired handle alive. The child artboard and every object keep their handles. The nested wrappers stay in their original arenas. Only their running machine or timeline instances move. The slot's active and explicit VM state, ownership flag, global VMs, quantization remainder, mounted bindable, focus scope, listeners, and host references travel with the occurrence.

At take-off, a one-version step saves the incoming slot's box and detaches its authored props into the native inline model, then discards its fresh instance. Both ordinary slots are empty during flight. Two-version steps move two distinct unit artboards, each with its own unit key. Both fly along the outgoing-to-incoming path, then return to their own Screen slots. Each move still validates the same artboard and wrappers. Forward and back steps name separate flight pairs; their authored object order puts the incoming version above. The destination releases fresh playback before releasing its fresh artboard. The moved occurrence is never mounted again. Its internal binds and script instances are not reinitialized. Its own data context stays the same; its parent changes. Newly attached occurrence props blend numbers and colors during flight; other values switch immediately, with the typed-text exception described in Section 8 below.

Within one semantic manager, only the existing boundary is detached and attached. Descendant registrations and ids remain. A first semantic mount or a different manager uses normal cleanup and construction. Moves repair focus ancestry without blurring a visible focused leaf. Sleep follows upstream hidden-focus policy: a normal advance can blur a hidden field. Wake retains the same focus and semantic nodes; applications can focus that same field again.

The follower multiplies blended endpoint opacity by the flight slot's keyable `fade`, before compensating for the flight parent's effective inherited and hosted opacity. Pose endpoints can hold different unit artboards. Only transport requires compatible artboards.

A live-move frame has four barriers, each across all active flights:

1. **Size:** capture keyed controls and endpoint sizes without sampling transforms, then apply every follower's size and blended values. Values belong here because their bindings can affect layout. Layout mode writes the interpolated box; scale mode keeps its departure box.
2. **Layout:** settle affected layout roots through the ordinary `Artboard::update_pass_handle`, with inner independent trees before containing trees. Shared roots are deduplicated. The ordinary list path refreshes cached row transforms and settles awake siblings. Explicit row passes also cover fixed-size Screens whose hosting list stays clean.
3. **Pose:** sample all awake endpoint and flight-parent transforms from that settled layout, retaining the last settled pose of sleeping endpoints. Compute all projections before applying their transform/opacity overrides. Use the same captured controls as Size; do not write size or blended values here.
4. **Settle:** use the ordinary update pass again to publish the final transforms and hosted caches. Poses do not invalidate layout, so this phase does not solve it again. Neither update phase advances machines or timelines.

The follower sizes the flight slot's own box, through the route its host kind already sizes a unit by: a CSS host's item record width and height, a native layout host's `instanceWidth` and `instanceHeight` in points (upstream's `StyleOverrider` pushes these into the unit), or a plain host's artboard size (as upstream's `NestedArtboardLeaf` sizes its artboard). Plain flight hosts appear only in test fixtures; the compiler emits layout hosts. A unit therefore always uses its host's ordinary sizing: the source's before takeoff, the flight slot's in flight, the destination's or home's after landing or return. Scale mode always writes the departure box, then scales that box in the pose.

A flight never pushes content. When a Screen occurrence is built, before its first layout, the runtime writes `position: absolute` into each CSS flight slot's item record; the Screen definition stays as imported. A native layout flight slot has no record to write, so it must be out of flow, for example under a plain `Node`; import warns when one sits in a parent that lays out. The compiler writes flight records border-box, with no minimum or maximum, `flex: none` and zero margins, so the written size is the box. The pose override supplies placement. The compiler emits no layout animation, so nothing retimes the follower's box. The flight record keeps the last size written: an absolute box does not size its parent, scrolling reads no Taffy content size, and an empty flight slot has nothing to hit.

Transport's saved `destination_size` and empty `leaf` copy seed a handoff and preserve the vacancy. A handoff clears the old native host dimension/unit caches, then runs ordinary host setup (including CSS ownership) before the next layout pass. A unit leaving a CSS carrier rebuilds its item style from its new host, so no field of the old record survives. The incoming or home record therefore applies as a whole.

A single pass cannot replace Layout and the final Settle when sizing dirties layout: pose sampling must happen between them. Clean component passes are skipped, so a frame without size/value work still needs just the final row pass (one row and two child updates for two flights sharing that row). The sizing pass and pose projection are separate operations; there is no layout/pose feedback loop.

An explicit occurrence VM can belong to its original Screen arena. The scene occurrence holds its storage lease by transported unit identity while it is needed. Slots and rows never own another row. Return and replacement release the lease, and dropping the scene releases every remaining lease. There is no arena history retained for the occurrence's lifetime. Empty slots retain only a layout leaf and identity observations. The controller caches weak row and slot identities. Removing an active flight holder or landing row also prunes the active step and settles surviving loans. If the incoming row leaves while another flight row survives, its loans return to surviving empty homes. Mounted rows determine its records and holder history. Removing a holder first returns loans to surviving homes, before the row is released.

## Work that was still reaching collapsed rows

The eager frame checkpoint and graph validation reached retained Screen models through the root `screens` list. Live-move frames now save rollback state before a property's first mutable access. The active-scene projection skips sleeping model graphs at list edges. A deliberate write to a sleeping model admits its ancestor path for that frame. Detached owners and overflow writes keep rollback coverage. Files without records retain the existing eager checkpoint, unless the host explicitly opts into row collapse.

A property can receive its VM owner after its first mutable access during mounting. The live checkpoint resolves those late owners from its touched properties too. This fixes the first-step `LimitExceeded` caused by discarding fresh flight mounts after their initialization writes.

The first device-shaped host probe still measured 3.02 ms with 39 rows collapsed against 0.29 ms for one row. Hosted-instance and VM-graph counters were already equal. Step phase timings put 2.70 ms in the advance block. A separate event-drain counter then found 162 artboard visits against 6. The host nested-event collector walked every retained row twice per step. It now uses the same active-row indices. The red runner is `eventred1`; the passing runner is `eventfixed2`.

Row world-transform updates and list-constraint transform loops also used retained rows. They now use the active row projection, as do layout-node collection, advance, drawing, and hits. Video enumeration uses the flat active index. Per-video operations reuse one identity map for the frame and membership revision. Decoder observations do not cause another scene walk. Semantic geometry validation uses the active hosted edges and checks a row again on wake. Structural discovery includes existing paint-artboard hosts, so ordinary files keep their video occurrences.

A flight caches the last settled pose of an endpoint that becomes collapsed. Following does not reenter that row. Waking refreshes the endpoint from the new advance. The occurrence owns the membership stamp and index, shared by all its players.

## Choices where the design leaves room

- Steps sit on the page root. Screens use artboard names and duplicate mounted Screen names are refused for live plans.
- The three records inherit `ContainerComponent`. This lets `LiveStepUnit` use the ordinary Component lifecycle under `LiveStep`. It avoids changing upstream parent validation.
- The active index has two views: a flat list of active instances owned by the scene occurrence for host queries, and the maintained active-row projection used inside upstream-shaped list loops. Both derive from the same membership edits. A retained identity list serves resource controls only and is rebuilt on structural edits.
- A context-less occurrence that inherited its Screen's context gets a local bridge when moved. Reparenting the Screen's shared context would modify other instances or form a cycle. An occurrence with its own context keeps that context's identity.
- Video occurrence metadata is cached by scene occurrence frame and membership revision. It remains valid across render revisions caused only by decoder observations. Video state is still read live from the retained handle.
- The checkpoint is lazy rather than a smaller eager snapshot. Merely omitting sleeping owners would lose rollback when a script deliberately writes one.

These choices keep the feature logic outside the mechanical port. The record and plan correction required no further mechanical-port edits. The row-sleep redesign below adds only the boundary hooks that component collapse cannot cross on its own.

Record parents, local value rules, and whole-step unit and flight uniqueness are checked during import. Once Screen instances are mounted, plan validation checks compatible unit artboards and wrappers, unique flight keys, and each step's flight references on its two Screens. The single-version unit and flight pairs must match. File sibling draw order follows Rive's reverse object order. The two-version fixture authors each named pair accordingly. The actual draw test caught the initial reversed fixture order in `correctfocus1` (35 passed, 1 failed); `correctfocus2` passed all 36 cases. The final `correctfocus3` run adds the third-Screen return regression and passes all 37.

## Regression assertions

Historical results describe their implementations at the time. Round 4 supersedes semantic-only sleep and the focus-navigation hook. Round 5 replaces the separate live-move mutation boundary and makes all feature collapse reconciliation use one hosted-ancestor visibility rule. Round 6 replaces global bookkeeping and adapter-owned caches with optional native occurrence state.

The numeric assertions below are implemented in `crates/nuxie-runtime/tests/live_moves.rs` unless a different owner is named. The focused runtime run `correctfocus3` passed all 37 cases. The whole-suite results and baseline comparisons are recorded in Verification.

| Test in `live_moves.rs` | Assertions |
| --- | --- |
| `continuity_identity_and_once_only` | 12 object identities and the running machine stay equal. Both timelines read 30, 50, then 60 through forward and back moves. |
| `follows_current_frame_and_comes_home` | Progress 0, 0.5, and 1 follow x = 215, 225, and 235 from the same advances. A cut returns the same instance. |
| `rejected_moves_do_not_mutate_slots` | Refused moves leave both occupied identities unchanged. |
| `private_values_text_and_destination_prop_survive` | Private count 7, 12-byte typed text, VM identity, and local context survive. Incoming prop becomes 200, follows 321, and ignores source 999. |
| `hits_follow_the_new_host` | A destination hit sets the same input to 9. A hit at the empty source leaves it at 0. |
| `empty_source_draws_no_geometry` | The source draws 1 shape before take-off and 0 after. |
| `collapsed_rows_cost_one_row_and_wake_with_data` | 40 active rows make 81 advances and 40 draws. With 39 collapsed: 3 advances and 1 draw. Sleeping data value 777 applies on wake. |
| `focus_scope_moves_without_blurring_the_field` | The same focused leaf stays primary across take-off, flight, and landing. |
| `empty_layout_slot_preserves_the_siblings_position` | The empty 150 by 80 slot preserves its sibling position. |
| `rotation_scale_opacity_layout_size_and_path_are_blended` | Shortest rotation crosses 180 degrees. Midpoint scale is 3 by 4, opacity 0.6, box 150 by 100, and fill-child width 150. |
| `timeline_controls_progress_and_path_offset` | Published progress and offsets control flight. There are no child-node controls. |
| `moved_vm_survives_removing_its_original_row` | Private count changes to 8 after origin-row removal. Identity survives, history is 0, and the removed row is refused as a move target. |
| `semantic_subtree_has_one_stable_occurrence_at_every_move_boundary` | Focused and unfocused cases make 7 moves. The unit keeps its 3 semantic nodes and one boundary without repeated ids. |
| `collapsed_semantic_geometry_is_validated_again_before_publishing_on_wake` | A 5,000-rectangle clip fails, is skipped while collapsed, and fails again on wake. Repair restores success. |
| `semantics_enabled_after_collapse_publish_only_active_rows_and_wake_with_stable_ids` | Late enablement publishes 1 field. Wake publishes 2. Two collapse/wake cycles preserve both ids. |
| `published_records_have_their_authored_parents` | Both Step records retain their unit child. Slot records retain their nested-artboard parents. |
| `native_keyframes_drive_live_slot_progress` | A real keyframe reaches progress 0.5. The midpoint adds path offsets 7 and 11. |
| `two_artboard_versions_keep_identity_and_reverse_authored_draw_order` | Red and blue units use distinct artboards. Forward names flights 11 and 12, back names 13 and 14. Actual page draws contain exactly 2 shapes, red then blue forward and blue then red back. Both identities return to their own slots and history is 0. Direct transport between versions is refused. |
| `two_version_fade_multiplies_blended_opacity_at_each_progress` | Endpoint opacities are 0.4 and 0.8. At progress 0, 0.5 and 1, forward incoming opacity is 0, 0.6, 0.8 and outgoing is 0.4, 0.6, 0. Back incoming is 0, 0.6, 0.4 and outgoing is 0.8, 0.6, 0. Both the slot and its painted shape match, and both versions follow the same pose path. |
| `malformed_flight_plans_are_rejected_before_transport` | Unknown flight key 999, a unit artboard mismatch and duplicate flight keys fail before either unit is taken. |
| `native_keyframes_drive_live_slot_fade` | A native timeline keys fade to 0.5 at 0.5 seconds. Output opacity equals blended endpoint opacity times 0.5. |
| `a_loaned_incoming_version_returns_before_its_two_version_takeoff` | A third Screen holds the incoming version. Begin returns it, then puts that same identity in named flight 12. Landing restores it to the incoming slot; the third slot stays empty. |
| `removing_the_holder_brings_its_borrowed_instance_home_first` | Removing the current holder returns the original instance first. History is 0. |
| `lua_private_state_survives_flight_landing_and_a_back_step` | A real Luau drawable accumulates 500 ms with init count 1 and the same CoreHandle. |
| `invalid_slot_parent_is_dropped_during_import` | A slot under a Node is dropped with a diagnostic. |
| `a_file_without_records_keeps_the_upstream_playback_and_mounted_slots` | Both ordinary mounted units retain their identities and both timelines reach 30 after 0.3 seconds. |
| `scale_mode_is_read_from_the_published_flight_record` | Mode 0 comes from the file. At progress 0.5 the box remains 100 by 80 and scales by 1.5 and 1.25. |
| `incompatible_wrappers_and_cross_player_moves_are_refused_without_taking_the_source` | Cross-player and mismatched-wrapper refusals preserve source ownership. |
| `navigating_to_a_kept_row_comes_home_from_a_third_screen` | A, B, and R2 share one transported identity. A cut brings it back to the kept row. |
| `another_players_membership_edits_do_not_rebuild_this_players_index` | Changing another player causes 0 index rebuilds in the measured player. |
| `removing_a_third_holder_preserves_the_other_kept_rows_home` | Removing R2 preserves the remaining empty-home route and the original instance. |
| `hosted_index_includes_existing_paint_artboard_occurrences_without_live_records` | An ordinary paint host has 2 indexed artboards, including its painted child. |
| `takeoff_discards_the_fresh_single_version_destination` | Both ordinary slots are empty, fresh playback is released, and the whole page draws 1 unit during flight. |
| `removing_a_flight_holder_returns_home_and_prunes_the_active_step` | The loan returns before removal. Active flights and history both become 0. A later cut succeeds. |
| `following_two_flights_settles_their_shared_row_once_without_playback` | 2 flights cause 3 update entries, one row and two units, and 0 playback advances. |
| `removing_the_incoming_screen_during_a_back_flight_returns_to_a_surviving_home` | The original instance returns to the surviving home. Flights and stale history become 0. A later cut succeeds. |
| `two_flights_read_their_own_published_sizing_modes` | Each flight reads its own record: incoming mode 0 keeps 200 by 120 and scales by 0.75 and 0.8333; outgoing mode 1 reflows to 150 by 100. |

The C API unit run `correct03capiunit` passed 4 cases. It covers guarded begin/end, null-name cuts, video id 1073741824 across source/flight/destination/source, and 20 semantic snapshots across focused and unfocused variants. Each variant performs 7 moves. Its 12-byte field value and field id stay equal. Guard cases cover null, unregistered, wrong-thread, reentrant, poisoned, and exhausted-revision calls before reading their string arguments.

The two `live_checkpoint` tests passed in `graphfixed1`. Beginning captures 0 properties. Detached scalar and list writes roll back. A write past a 0-entry bound still rolls back. The C API startup case also verifies late VM-owner assignment during fresh mounting.

The device-shaped video test passed again in `life04capi`. It keeps actual source id 1073741825 across take-off, landing, collapse and wake. With the current fixture that source is also the minimum unit occurrence id. Earlier fixture probes exposed the harness selecting a different minimum id. The device-shaped probes use the real C API for step and presentation acknowledgement, video enumeration and visibility, media commands, semantics, and row collapse and wake. They compare every frame-path counter for one row and forty rows with 39 collapsed, plus a flight. The counter set includes native advance, layout, drawing, hits, event drains, video walks, semantic validation, bound-owner capture, VM-graph visits, and checkpoint touches. Lookups after enumeration add zero scene visits.

## Host performance and visit counts

`life04capi` measures the updated lifecycle fixtures over 20 frames per case. Each frame advances with a pointer move, draws, enumerates videos, checks visibility, and takes a semantic snapshot. The phase timer and every counter are compiled only with `test-support`.

| Case | Step ms | Video enumeration ms | Visibility batch ms | Semantic snapshot ms |
| --- | ---: | ---: | ---: | ---: |
| One row | 0.787840 | 0.092533 | 0.299590 | 0.555346 |
| Forty rows, 39 collapsed | 0.490254 | 0.054213 | 0.110954 | 0.509458 |
| Same page, flight | 0.472402 | 0.039704 | 0.104144 | 0.296854 |
| Semantics enabled after collapse | 1.203311 | 0.105650 | 0.168300 | 0.804715 |

One row, 39 collapsed, and late semantic enablement have exactly these per-frame counts:

| Path | Visits |
| --- | ---: |
| Index rebuild | 0 |
| Native advance | 3 |
| Layout/update | 3 |
| Draw | 3 |
| Pointer routing | 3 |
| Nested-event drain | 6 |
| Video enumeration | 2 |
| Per-video lookup | 0 |
| Semantic validation | 2 |
| Bound-owner capture | 4 |
| VM-graph owners | 9 |
| Checkpoint properties | 1 |

The three lifecycle values add three VM-graph visits to the earlier fixture. One row and 39 collapsed rows still match on every counter.

Flight has the same counts, plus two layout/update entries for its one row and moving unit. Screen A has the inline comparison content. Screen B does not. A flight in B therefore has one video and one semantic field, against two of each on A. The moving unit still appears once.

These are host timings. They do not establish the device 1 ms bar.

## Device fixtures and device qualification

`test -p nuxie-runtime --test live_moves_device_fixture` exports `page1.riv`, `page2.riv`, `page40.riv`, and video inventories. Each Screen has its own lifecycle values at `screen/phase`, `screen/transition`, and `screen/appearances`. Phase uses the string form accepted by the device lifecycle contract: hidden, entering, active, exiting. This also satisfies the current preflight, which requires string-valued phase. Appearances is numeric and starts at zero. The SDK writes it.

The two-row and forty-row pages have two named arrivals. B owns flight key 1 for `t1`. A owns flight key 2 for `back`. Each Screen's default machine has an arrival layer guarded by both entering phase and its step name. Its one-shot timeline keys that flight's `LiveSlot.progress` from 0 to 1 at 60 fps over one second. An Event callback at frame 60 fires `nx_enter_done:t1` or `nx_enter_done:back`. Holding entering holds progress one without replaying the event. Leaving entering or changing the step name returns the layer to idle, so the next entry starts at zero. No progress, path or fade property is VM-bound. These device pages contain a single-version unit; the separate two-version runtime tests cover authored fade multiplying blended opacity.

The builder retains the real font, native input listeners, videos, clipping, and twenty content rows. It validates the payload before writing. Every page checks 20 content rows, 24 layouts, 1 native input field, keyboard/text listeners, keyed x = 30 after 0.3 seconds, and continuity through take-off to x = 50 after another 0.2 seconds. The files are 26,373, 27,741, and 39,353 bytes after the inline-video audio correction below. There are no step or flight copies in the one-row baseline and no `moves.json`.

`lifecycle_writes_play_and_restart_both_authored_arrivals` passes in `life03` and the final whole-suite run `life06runtime`. It drives arrivals using only Screen phase/transition writes and step begin/end calls. It checks both directions twice, progress 0, 0.5 and 1, exactly one completion event per traversal, and no repeated event while entering is held. It checks 8 traversals, 24 progress samples, 8 completion events, both start guards and 126 lifecycle defaults across 42 Screens in the two-row and forty-row pages.

The six builder outputs are available at `/private/tmp/live-moves-runtime-v2-device-fixtures`. The original lifecycle exports matched all six SHA-256 values from the whole-suite run `life06runtime`. The RIV files now include the inline-video audio correction below. Fingerprints are retained in `lifecycle-device-fixtures.json` in the correction verification directory. The device harness preflight passed for all three pages with `--emit-contract`; its three `lifecycleN.json` files are alongside the pages and inventories. It validates the video inventory, new records, authored one-second progress tracks and completion declarations.

Runtime behavior and ported files were unchanged by this fixture correction. iOS Simulator, Android emulator, and Bat Phone qualification has not run here. The supplied runner accepts host Rust package checks and tests, so physical device playback remains the harness's next check.

## Review round 1

This round fixes all six findings on top of `731bff2e84370435dcb718660c774b42bcc40d5e`. Changes remain uncommitted. Every regression reached its reported failure before its fix. Rust builds and tests used the supplied request-file runner only.

| Finding | Fix and regression |
| --- | --- |
| 1. Checkpoint overflow | `nux-capi/src/lib.rs` poisons the occurrence before reporting live checkpoint overflow. VM rollback cannot undo input, playback, poses or ownership. `live_moves_checkpoint_overflow_poisons_the_occurrence_after_playback` imports two instances sharing a bounded schema and authors 4,097 to-source bindings. A 0.5-second timeline advance reaches x = 50 before overflow. Subsequent draw, step begin and player step must return `RuntimeError`. A two-property control succeeds. Red: `review1r1g`, draw returned `Ok` after overflow. |
| 2. Missing landing row | Preparation cancels the entire active step if a flight holder or landing slot has disappeared. The existing settlement pass returns surviving loans and releases orphans. The rule that a holder returns other Screens' loans before removal remains in place. `removing_a_two_version_landing_screen_settles_survivors_and_allows_the_next_step` puts both flights in B, removes their outgoing landing row A, then checks zero flights, zero history, the incoming unit's identity in B, and a successful later cut. Red: `review1r2`, the active flights remained. |
| 3. Inherited flight opacity | The follower compensates with the parent's effective `world_transform_child_opacity()`, matching rendering. `flight_drawn_opacity_matches_ends_under_translucent_ancestors_and_hosts` checks progress 0 and 1 against normal source and destination paints under ancestor opacity 0.5, hosted opacity 0.5, and both. It also checks the landed paint. The ancestor-only endpoints are 0.4 and 0.3. Red: `review1r3`, source opacity was 0.2 instead of 0.4. |
| 4. Keyboard focus | A small commented `FocusManager` hook checks active hosted ancestry for navigation stops. It leaves held primary focus intact. `collapsed_rows_are_skipped_by_next_and_previous_focus_and_wake_with_the_same_nodes` uses the existing font-backed device fixture. Three collapse/wake cycles exclude B in both traversal directions, retain A as a stop, and restore B with the same focus-node identity. Red: `review1r4b`, forward traversal entered B while collapsed. The existing focused semantic-boundary regression also passes. |
| 5. VM list projection | List notifications and rollback invalidate cached filtered children, including outside a live frame. Weak cache observers retain no player or model arena. The frame falls back to current list children immediately; the next frame rebuilds the projection. `projected_vm_lists_refresh_before_hosted_membership_changes` edits the Screen list both inside and outside a frame while the hosted membership stamp stays unchanged. It checks added-model reachability, delivery of value 321, continued sleeping-model exclusion, removal, and rollback after a provisional projection rebuild. Red: `review1r5`, the added model was unreachable. |
| 6. Layout flight placement | A small commented `NestedArtboardLayout` hook skips layout placement while the full flight pose override is active. Ordinary layout is unchanged. `a_layout_flight_with_nonzero_placement_matches_both_endpoints` starts with a flight placement of x = 170, checks all six matrix entries at progress 0 and 1, and lands the same unit at (260, 25). Red: `review1r6`, take-off x was 170 instead of 0. |

The cache invalidation uses the existing Nuxie list-change bridge. This keeps the ported list implementation untouched. The only new ported edits are the navigation-stop guard and the layout-placement guard. No schema, header, export or ABI changed.

### Review verification

All checks below passed with the normal repository build settings. There are no remaining failures. The two ignored runtime unit cases were already ignored.

| Check | Runner | Result |
| --- | --- | --- |
| `test -p nuxie-runtime --test live_moves` | `review1v01` | 42 passed, 0 failed. |
| `test -p nux-capi --lib live_moves` | `review1v02` | 5 passed, 0 failed. |
| `test -p nux-capi` | `review1v03` | 216 passed, 0 failed. |
| `check -p nux-capi --all-targets` | `review1v04` | 26 targets passed. |
| `check -p nuxie-runtime --all-targets` | `review1v05` | 235 targets passed. |
| `test -p nuxie-runtime --lib` | `review1v06` | 691 passed, 0 failed, 2 ignored. |
| `test -p nuxie-runtime --test host_focus_input` | `review1v07` | 4 passed, 0 failed. |
| `test -p nuxie-runtime --test focus_retained_tree` | `review1v08` | 4 passed, 0 failed. |
| `test -p nuxie-runtime --test upstream_focus_wave_b3` | `review1v09` | 109 passed, 0 failed. |
| `test -p nuxie-runtime --test upstream_focus_b36aa3d0` | `review1v10` | 3 passed, 0 failed. |
| `test -p nuxie-runtime --test upstream_focus_lifetime_58c2f5e8` | `review1v11` | 7 passed, 0 failed. |
| `test -p nuxie-runtime --test semantic_focus_runtime` | `review1v12` | 10 passed, 0 failed. |
| `test -p nuxie-runtime --test upstream_semantic_focus_list` | `review1v13` | 2 passed, 0 failed. |
| `test -p nuxie-runtime --test host_view_model_list_projection` | `review1v14` | 10 passed, 0 failed. |
| `test -p nuxie-runtime --test upstream_view_model_change_window` | `review1v15` | 18 passed, 0 failed. |
| `test -p nuxie-runtime --test layout_minimum_contract_tests` | `review1v16` | 16 passed, 0 failed. |
| `test -p nuxie-runtime --test upstream_legacy_layout_transform` | `review1v17` | 3 passed, 0 failed. |
| `test -p nuxie-runtime --test native_layout_proxy_visibility` | `review1v18` | 1 passed, 0 failed. |
| `test -p nuxie-runtime --test upstream_layout_matrix` | `review1v19` | 1 passed, 0 failed. |
| `test -p nuxie-runtime --test live_moves_device_fixture` | `review1v20` | 2 passed, 0 failed. |
| Device preflight | `check_live_fixture.py` | Rows 1, 2 and 40 passed. |
| Generated Bazel files | `python3 tools/bazel/generate.py --check` | Passed. |
| Formatting and whitespace | `rustfmt --check`, `git diff --check` | Passed. |

The device builder is unchanged. Fresh export hashes and all nine existing RIV, inventory and lifecycle artifacts match the start of this round. The unchanged device preflight passes for rows 1, 2 and 40. The wider checkpoint fixture and layout flight fixture are separate `test-support` builders and do not affect device bytes.

Red and green test logs, counts, fixture hashes, preflight output and check results are retained in `/private/tmp/live-moves-review1`. Runner request logs remain in the supplied `runtime-runner-v2` directory. The earlier full-runtime main comparison below is historical evidence from the fixture correction, not a new whole-runtime run for this review.

## Android inline-video audio correction

Only the inline comparison Video now authors `muted=true`. The unit Video remains audible. This avoids the inline player's competing Android audio-focus request. The export validator checks both mute values.

Runner `inlinemute01`, `test -p nuxie-runtime --test live_moves_device_fixture`, passed both tests. Preflight passed for rows 1, 2 and 40. Each RIV differs from the prior export by exactly one four-byte property insertion on `inlineClip`, definition 1, component 22. Removing those four bytes reproduces the original file exactly. Record identities, inventories and lifecycle files are unchanged. Verified exports replace the files in `/private/tmp/live-moves-runtime-v2-device-fixtures`. Logs, before/after bytes and hashes are in `/private/tmp/live-moves-inline-muted`.

## Review round 2

This round fixes both findings on top of `62c1a0269225e942a34605c873761ee31c359880`. Changes remain uncommitted. No ported file, schema, C API surface or device builder changed.

`records.rs` now shares whole-step validation with `plan.rs`. Import rejects the whole file when a step repeats an incoming unit, outgoing unit or flight key across mappings. A single-version mapping may name its one flight twice. Different mappings may still use the same unit key in opposite roles. The existing `LiveStep` clean lifecycle provides import validation after all children are linked, so no ported import hook is needed. Calls validate before preparation can discard fresh flight mounts and again before building the ownership schedule. Preflight also rejects repeated coming-home owners or destinations, and fresh destinations that overlap any scheduled move.

`overlapping_step_units_are_rejected_before_any_slot_changes` first runs a valid four-flight control with two unit definitions and four instances. It then checks three invalid plans at the call: duplicate incoming keys, duplicate outgoing keys, and a shared flight. Every one of the 12 slots keeps its prior instance identity or empty state; flights and retained history remain zero. `import_rejects_overlapping_step_unit_records` checks that the same three malformed files fail import. Before the fix, `review2red1b` passed 42 cases and failed both regressions. Import accepted duplicate incoming key 2. At the call, the `1 -> 2`, `2 -> 2` plan discarded B's unit and moved A's unit before returning an error.

`collapse.rs` now marks the list's `RENDER_OPACITY` dirty on wake. Its existing native update reapplies inherited opacity to active rows, alongside the existing transform refresh. No row identities change and sleeping rows stay out of those updates. This reuses the native opacity calculation without changing ported code.

`waking_a_row_refreshes_inherited_opacity_and_world_transform` checks three host states after a sleeping row wakes: opacity and translation `(1, 70, 30)`, `(0.5, 70, 30)`, and `(0.25, -12, 18)`. The rendered paint opacity must match the literal value. The complete canonical draw recording must match a never-collapsed control with the same host edits. The control must differ from its pre-edit recording, and the woken unit keeps its identity. In `review2red2`, the plan regressions passed and the wake regression failed with expected opacity 0.5, actual 1: 44 passed, 1 failed. The world-transform control already passed before the fix.

### Round 2 verification

| Check | Runner | Result |
| --- | --- | --- |
| `test -p nuxie-runtime --test live_moves` | `review2v01` | 45 passed, 0 failed. |
| `test -p nux-capi --lib live_moves` | `review2v02` | 5 passed, 0 failed. |
| `test -p nux-capi` | `review2v03` | 216 passed, 0 failed. |
| `check -p nux-capi --all-targets` | `review2v04` | 26 targets passed. |
| `check -p nuxie-runtime --all-targets` | `review2v08` | 235 targets passed. |
| `test -p nuxie-runtime --test live_moves_device_fixture` | `review2v06` | 2 passed, 0 failed. |
| `test -p nuxie-runtime --lib` | `review2v07` | 691 passed, 0 failed, 2 ignored. |
| Device preflight | `check_live_fixture.py` | Rows 1, 2 and 40 passed. |
| Generated Bazel files | `python3 tools/bazel/generate.py --check` | Passed. |
| Formatting and whitespace | `rustfmt --check`, `git diff --check` | Passed. |

The first runtime all-target attempt, `review2v05`, compiled all 235 targets successfully, then the runner failed during artifact copying with `FileNotFoundError` for `nuxie-runtime__test_upstream_text_input_native`. The unchanged retry, `review2v08`, passed. Both logs are retained. The two runtime unit ignores are the existing hidden `.frame_probe` report-only sweeps.

All checks used the supplied request-file runner and normal repository build settings. No failure remains. The device builder and all nine existing RIV, inventory and lifecycle artifacts match their hashes from the start of this round. Fresh exports match all six original RIV and inventory hashes. The fixture preflight also passes for rows 1, 2 and 40. Evidence is retained in `/private/tmp/live-moves-review2` and the runner request directory. Earlier whole-runtime main comparisons below are historical; this round runs the suites listed above.

## Review round 3

This round fixes three findings by their two causes on top of `323808ec3e`. Changes remain uncommitted. No ported file, schema, export or device builder changed. The generated C header changes only the video ID lifetime comment.

### Occurrence ownership and mutation publication

`ArtboardOccurrence` now owns `LiveMoves`, `VideoOccurrences` and the scene frame revision. Every player of that occurrence uses the same controller, active index, flights, rows and coming-home history. Closing a player does not discard the occurrence's flight. The existing occurrence mutation guard makes this safe without rejecting a second player. Player selection and explicit global VM contexts retain their existing ownership.

The three calls in `live_moves.rs` capture VM owners before mutation and finish through `commit_legacy_runtime_change_or_poison()`. It resolves exact changed owners across the pre-mutation and current scene, reserves one mutation generation, and calls `mark_observable_mutation()` on those owners. Another occurrence holding the same model therefore requests presentation. Unrelated models are not invalidated. Captures and owner resolution use the existing boundary, including its poison behavior on a post-mutation failure.

The landing test also exposed stale parent prop bindings targeting discarded fresh destination and flight mounts. A subsequent row settlement wrote 321 to two retired models outside the active owner scope. `transport.rs` now detaches those live bindings when discarding a mount. Saved prop templates remain available for adoption. This keeps the boundary's exact-owner checks intact and needs no ported hook.

### Retained video control

`HostedIndex` retains a weak identity list alongside its active list. The video control table discovers every retained video when membership changes. It drops removed mounts and retains IDs across collapse and wake. Frame enumeration still scans only active boards. Visibility first checks the active ID set, so a sleeping retained video reports false while control, observations and decoder reclaim remain available by ID.

The existing playback reclaim operation now runs for sleeping videos. It clears readiness and advances the decoder generation. The replacement's ready observation seeks to retained playback position before restoring play intent. No playback controller behavior changed.

### Round 3 regressions

| Finding | Regression in `nux-capi/src/live_move_tests.rs` | Assertions |
| --- | --- | --- |
| A1. Independent player controllers | `live_moves_players_share_the_occurrence_flight_and_history` | A takes off, B enumerates and advances, and A lands. B starts coming home and A ends it. Video IDs agree. A then starts another flight and closes; B advances and lands it. |
| A2. Unpublished binding writes | `live_moves_publish_exact_binding_owners_on_takeoff_landing_and_wake` | Another occurrence holds the transported unit's existing model. Take-off writes 200, landing writes 321, and wake writes 456. Each refuses the observer's old presentation revision. A sleeping source edit has not yet written the unit. An unrelated model's presentation revision remains valid throughout. |
| B. Sleeping videos lose control lookup | `live_moves_sleeping_video_can_reclaim_and_resume_the_same_id` | An authored 2.5-second seek opens decoder generation 1 and plays. Collapse removes the video from enumeration and visibility. Reclaim while asleep returns generation 2. Suspension commands still work. Wake keeps the ID, ignores old ready, and emits seek to 2.5 before play for generation 2. Duplicate ready emits nothing. |

The final three regressions were rerun against `323808ec3e`'s production code, retaining only the new tests and test-support variant. Runner `r3baseline` passed the five existing cases and failed all three new cases. A1 returned `InvalidArgument` when A tried to land after B's enumeration and advance. A2 accepted the other holder's stale presentation revision after take-off. B returned `NotFound` when reclaiming the sleeping video. Restoring the fixes passed all eight cases in `r3bgreen2`. The temporary baseline source replacement was restored byte for byte from saved fixed-source hashes before final verification.

The full C API run also passed `live_moves_collapsed_device_probe`. Its existing visit-count assertions still exclude sleeping rows from frame work. In this debug host run, 20 iterations reported the following averages. These timings are observations, not test thresholds.

| Scene | Enumerated videos | Semantic nodes | Step average (ms) |
| --- | --- | --- | --- |
| 1 row | 2 | 2 | 0.724510 |
| 40 rows, 39 asleep | 2 | 2 | 0.634100 |
| 40 rows, one flight, 39 asleep | 1 | 1 | 0.188996 |
| 40 rows, late semantics, 39 asleep | 2 | 2 | 0.558012 |

The video regression uses a separate `test-support` builder variant. Existing generic fixture bytes and exported device fixtures retain their authored structure.

### Round 3 verification

| Check | Runner | Result |
| --- | --- | --- |
| `test -p nuxie-runtime --test live_moves` | `r3v01` | 45 passed, 0 failed. |
| `test -p nux-capi --lib live_moves` | `r3v02` | 8 passed, 0 failed. |
| `test -p nux-capi` | `r3v03` | 219 passed in 22 targets, 0 failed. |
| `check -p nux-capi --all-targets` | `r3v04` | 26 targets passed. |
| `check -p nuxie-runtime --all-targets` | `r3v05` | 235 targets passed. |
| `test -p nuxie-runtime --test live_moves_device_fixture` | `r3v06` | 2 passed, 0 failed. |
| `test -p nux-capi --test video` | `r3v07` | 9 passed, 0 failed. |
| `test -p nux-capi --test video_properties` | `r3v08` | 10 passed, 0 failed. |
| `test -p nux-capi --lib video` | `r3v09` | 7 passed, 0 failed. |
| Device preflight | `check_live_fixture.py` | Rows 1, 2 and 40 passed. |
| Generated Bazel files | `tools/bazel/generate.py`, then `--check` | Passed. |
| C surface and layout contracts | `check-nux-capi-surface.py`, `check-nux-capi-layout.py` | Passed. |
| Formatting and whitespace | `rustfmt --check`, `git diff --check` | Passed. |

Every final runner request exited 0. No final verification failure remains, so no new main comparison was needed. The three expected regression failures against the pre-fix commit are recorded separately above. Earlier whole-runtime comparisons below remain historical.

The device builder and all nine existing RIV, inventory and lifecycle files match their hashes from the start of this round. Fresh fixture exports match all six original RIV and inventory hashes. Preflight passes without rewriting the contracts. Evidence, logs, baseline results and fixture hashes are retained in `/private/tmp/live-moves-review3` and the request directory. Only newly copied `target/debug` artifacts were removed after verification; the existing text-input clock probe and platform build directories were preserved. No direct cargo or Bazel invocation was made, and nothing was committed.

## Review round 4: collapse and drawing redesign

This redesign is on top of `163d6bd7d0`. Changes remain uncommitted. Records, ABI, device builder, and device fixture bytes are unchanged.

### One native collapse path

Sleeping a row collapses its artboard's top-level component children through the existing virtual component collapse. The artboard root stays awake, as upstream requires. Wake uncollapses those children and settles their retained binding dirt. Native component collapse now supplies drawable and hit exclusion, focus eligibility, component-target binding suspension, semantic membership, and dirty-component catch-up. No semantic nodes are cleaned up or rebuilt for sleep or a move within the same semantic manager.

List rows live outside the component tree, and upstream nested hosts stop component collapse at artboard boundaries. The maintained row index therefore remains necessary for list advance, machine binds, draw and hit entry, layout collection, and the runtime's video, model, semantic-validation, event and checkpoint walks. Decoder control still uses every retained video's stable ID.

The list's existing collapse suffix composes `list collapsed || row asleep`. Its native semantic-boundary operation stays inside that suffix. A small opt-in hook collapses row contents too, and reconciles hosted boundaries recursively. Opt-in persists after wake so later native list uncollapse can reconcile a row that was woken while its list was hidden. Ordinary upstream lists retain their existing behavior. Structural collapse visits all retained rows; frame geometry work still uses active rows. Native opacity inheritance reaches retained row roots when list opacity changes; sleeping children keep dirt for uncollapse. There is no feature wake-time opacity re-dirty.

Every `move_live` attachment reconciles the destination's effective component collapse through `NestedArtboard`'s existing collapse suffix. That suffix restores or removes the transported unit's existing semantic boundary and descendants. Reconciliation also handles nested hosts whose own collapse bit did not change during the attachment. Late semantic enabling uses the same row reconciliation rather than a separate visibility rule.

Removed hooks and feature helpers:

- The ported `FocusManager::is_focus_stop` guard and `live_move::focus_stop_allowed`. Native focus eligibility now sees collapsed components.
- `hosted_scene::collapse_new_row_semantics` and `resume_semantics`, including their direct feature calls to semantic boundary collapse.
- The wake-time list `RENDER_OPACITY` re-dirty. Native inheritance and retained component dirt supply catch-up.
- The follower's manually composed `board_to_root` transform.

The small list collapse and late-semantic-enable hooks remain because upstream collapse stops at artboard boundaries. Existing active-index, ownership and layout override hooks remain. The layout override hook still prevents applying authored placement twice while a full flight transform is active.

### One draw-pose projection

`follow.rs::draw_pose` reads rendered opacity and effective child opacity, and projects a component's world basis through the existing `Artboard::semantic_root_transform`. That native drawing-contract projection already includes artboard self transforms, frame origins, the list host's full `draw_transform_for_index()` matrix, and nested layout placement. Both endpoints and the flight parent use this one function. Following converts the blended root pose through the inverse parent draw pose and compensates for the same effective child opacity used by rendering. No new pose hook was added inside ported code.

### Regressions and verification

The three requested regressions were run against unchanged `163d6bd7d0` production sources, retaining only the new tests and test-support path variant. Runner `r4baseline` failed all three: a cut left the field absent, wake under a hidden list exposed it, and the flight's first rendered bound was 249.5 instead of 67.5. All eight fixed production paths were then restored and verified byte for byte. The intermediate compiler and recording-helper failures are retained in the logs; they were corrected before this baseline run.

| Finding or contract | Regression | Assertions |
| --- | --- | --- |
| 1. Cut out of a sleeping row | `sleeping_row_cut_restores_semantics_round4` | A to B, sleep B, cut to expanded A. The same field id becomes visible again. |
| 2. Lossy list-row poses | `follow_path_rows_draw_the_same_bounds_at_flight_endpoints_round4` | A real `ListFollowPathConstraint` gives the two rows tangents more than one radian apart. Row scales are `(1.5, 0.75)` and `(0.65, 1.3)`. Independently recorded renderer matrices and painted path vertices give matching bounds at progress 0 and 1 and after landing, within 0.002. The existing inherited-opacity and nonzero-layout-placement regressions also pass. |
| 3. Wake under a hidden list | `wake_under_a_collapsed_list_keeps_semantics_hidden_round4` | Individual sleep, native list collapse, individual wake exposes no field. List uncollapse restores the same field id. The inverse order keeps an individually sleeping row hidden through list uncollapse. |
| SDK identity | C API `live_moves_sleep_wake_and_hidden_transfers_keep_sdk_identities_round4` | The same TextInput semantic object and id, and opaque SDK video id, survive sleep/wake, a cut out of a sleeping row, and direct compatible moves into and out of one. Snapshots show exactly one field when visible and none when asleep. Video enumeration excludes asleep instances; control lookup still accepts their original id. The observed ids were semantic 6 and video 1073741824. Steps intentionally require an awake incoming Screen, so the hidden-destination case uses the public Rust transport seam. |
| Native wake work | `one_wake_in_forty_rows_reports_native_dirt` | Wakes B in the unchanged 40-row device fixture after 39 rows sleep. Every root artboard stays uncollapsed. The other 38 sleeping rows and their retained instances have identical dirt before and after wake. |

The first passing runtime run, `r4g01`, passed all 49 cases. Its single wake measured 0.217583 ms for the wake call and 1.543875 ms for the next normal zero-time advance and settle. Dirt changed on 25 components: Page 2, B 22, and the hosted mascot 1. Added bits were `0x00c4`: COMPONENTS, TRANSFORM and WORLD_TRANSFORM. Removed bits were `0x0001`: COLLAPSED. There was no added RENDER_OPACITY bit. These are debug-host observations, not performance thresholds.

The existing visit-count regression still reports 81 versus 3 advances and 40 versus 1 draw for 40 expanded rows versus 39 sleeping rows. Its 100-advance timings in `r4g01` were 101.058458 ms expanded, 1.571625 ms sleeping, and 2.210291 ms for the one-row control. The same-node focus traversal, late semantics, binding-owner publication, decoder replacement, opacity catch-up and transform catch-up regressions also pass.

The nested-list regression `nested_list_descendants_follow_the_same_native_collapse_path` also covers a unit that hosts another list. The first fixture attempt passed a model instead of a native list item and was corrected. With the valid fixture, `r4nestedRed2` reproduced a semantic field still visible after moving the unit to B and sleeping B. Reconciliation now invokes the native list suffix too, preserving a child row's own sleep state. The final test checks sleep/wake, child sleep through ancestor wake, a cut-style transport out of sleeping B, and the same nested field id.

The final runtime run `r4final01` passes all 50 cases. Its wake call measured 0.146417 ms and the following zero-time settle measured 1.053250 ms. Dirt changes are the same 25 components and the same flags as above. Its 100-advance measurements were 36.939458 ms expanded, 1.472000 ms with 39 sleeping rows, and 1.272125 ms for one row. Wall timings vary with host load; visit counts remain exactly 81/3 advances and 40/1 draws.

The final C API collapsed-device probe also keeps every steady-frame counter equal between one row and 40 rows with 39 asleep (including late semantic enabling): index rebuilds 0, advance/layout/draw/hit 3 each, video enumeration 2, lookup walks 0, semantic validation 2, owner capture 4, checkpoint 1, value graph 9, and nested-event drains 6. Its 20-frame step averages were 0.285531 ms for one row and 0.332261 ms for 40 with 39 asleep. One active flight averaged 0.252917 ms. These are host observations; no device timing claim is made.

### Final round 4 verification

| Check | Runner | Result |
| --- | --- | --- |
| `test -p nuxie-runtime --test live_moves` | `r4final01` | 50 passed, 0 failed. |
| `test -p nux-capi --lib live_moves` | `r4final02` | 9 passed, 0 failed. |
| `test -p nux-capi` | `r4final03` | 220 passed in 22 targets, 0 failed. Includes C API focus, semantic and video suites. |
| `check -p nux-capi --all-targets` | `r4final04` | 26 targets passed. |
| `check -p nuxie-runtime --all-targets` | `r4final05` | 235 targets passed. |
| `test -p nuxie-runtime --test live_moves_device_fixture` | `r4final06` | 2 passed, 0 failed. Authored lifecycle playback and all three exports pass. |
| `test -p nuxie-runtime --lib` | `r4final07` | 691 passed, 0 failed, 2 existing report-only cases ignored. |
| `test -p nuxie-runtime --test upstream_focus_wave_b3` | `r4final08` | 109 passed, 0 failed. |
| `test -p nuxie-runtime --test semantic_focus_runtime` | `r4final09` | 10 passed, 0 failed. |
| `test -p nuxie-runtime --test upstream_semantic_focus_list` | `r4final10` | 2 passed, 0 failed. |
| `test -p nuxie-runtime --test nux_video` | `r4final11` | 31 passed, 0 failed. |
| `test -p nuxie-runtime --test video_properties` | `r4final12` | 7 passed, 0 failed. |
| Device harness preflight | `check_live_fixture.py` | Rows 1, 2 and 40 pass. |
| Generated Bazel files | `tools/bazel/generate.py`, then `--check` | Passed. |
| C surface and layout contracts | `check-nux-capi-surface.py`, `check-nux-capi-layout.py` | Passed. |
| Formatting and whitespace | `rustfmt --check`, `git diff --check` | Passed. |

Every final request exited 0. No verification failure remains, so no new main comparison was needed. Expected regression failures against the pre-redesign code are recorded separately above. The first complete C API and all-target runs (`r4v03` to `r4v05`) also passed, then were repeated after the nested-list boundary correction. Remaining focused checks were held until that correction to avoid unnecessary requests.

The device builder and all nine existing RIV, inventory and lifecycle files retain their starting hashes. Fresh exports match all six original RIV and inventory hashes. Preflight passes without changing contracts. All production and test sources match the hashes saved before final verification. Evidence and logs are retained in `/private/tmp/live-moves-redesign4` and the request directory. The 259 newly copied `target/debug` artifacts were removed after verification; the preexisting text-input clock probe and platform build directories were preserved. No direct cargo or Bazel command was run. Nothing was committed.


## Review round 5: shared visibility and mutation boundaries

The starting revision is `4241d8b941`. These changes are uncommitted. Records, ABI signatures, flight poses and device fixture contents are unchanged.

### A. One effective hidden state

`live_move/collapse.rs::effectively_hidden` is the one feature visibility rule. It checks component collapse and row sleep, then crosses each hosted artboard boundary through the actual host and row identity. It keeps walking through enclosing nested artboards and the containing Screen. It accepts a borrowed component so the native list collapse suffix can use it without reborrowing that list.

Sleep, wake, slot attachment, nested host reconciliation, late semantic enabling and the plan's active-row checks all use this rule. The cached row membership indices remain for per-frame loops. The list suffix applies it to each managed row. The old local list-bit test and carried hidden argument are removed. Ordinary lists that have not opted into row sleep retain their native collapse path. A wake only flushes retained binds if the resulting effective state is visible. Upstream component collapse still owns descendant dirt, focus, semantic membership and opacity catch-up.

`nested_row_wake_under_sleeping_screen_defers_semantics_and_binds_round5` builds a list inside a nested unit inside Screen A. It sleeps the inner row and Screen A, changes the inner row's prop source to 777, then wakes the inner row. The list's own collapse bit is false, yet the TextInput stays absent and the bound prop stays at 200. Waking Screen A restores the same semantic id and applies 777. Against the original production code, runner `r5aRed2` fails with actual 777 versus expected 200 during the inner wake. Its earlier setup attempt was corrected to mount the inner data context before testing; that setup failure is not regression evidence.

### B. One player mutation boundary

`lib.rs::player_operation_body` now contains the existing player step boundary. Both `player_step_body` and the three status-only live-move calls enter that body. Live-move requests are decoded after the common handle, thread, occurrence and reentrancy checks. They use the existing global subscriptions and scene roots, bounded checkpoint and change capture, value-policy passes, delegate transaction, exact changed-owner publication, render revisions and post-mutation poison policy. A refused request that changed no ownership, components or captured VM values leaves the occurrence usable. Revision exhaustion is checked before reading request strings.

A live operation replaces only the playback-advance action. It does not dispatch frame input, advance playback, increment the scene step or run the follower a second time. The common result firewall publishes and commits its internal result, then the status-only adapter frees it. The C signatures stay the same.

Deleted from `live_moves.rs`: `change`, `boundary`, their separate handle and occurrence guards, bound-only `begin_bound_view_model_change_capture`, direct `commit_legacy_runtime_change_or_poison` call, independent render-revision precheck and separate panic wrappers. The module now only decodes requests and applies their runtime operation. The existing legacy helper remains for other API boundaries.

`live_moves_global_only_boundary_publishes_and_notifies_round5` uses two players that share a global Unit model. The writer has no explicit main-model bind. Take-off writes 200, landing writes 321 and wake writes 456. Each operation invalidates the observer's acknowledged render revision and delivers the property's notification exactly once after commit. `live_moves_use_step_publication_poisoning_round5` injects the existing step-publication panic separately after take-off, landing and wake. Each returns RuntimeError, poisons the occurrence and refuses its next mutation. Against the original production code, `r5bRed2` fails both tests: the observer accepts its stale revision, and the old live-move path ignores the publication failure injection. The existing guard, exact-owner publication and checkpoint-overflow poison regressions also pass in the shared boundary.

### C. No row scan without transported loans

`transport.rs` records lists with actual transported holders as weak identity observations. Ownership transfer and discard update this registration at the ownership change. It retains no artboard arenas. `return_row_loans` returns before borrowing or scanning a list when no transported holder is registered, including files without records. For lists that do hold loans, the hook builds the next membership set once instead of doing linear `next.contains` for every row.

`ordinary_ten_thousand_row_append_does_no_live_move_work_round5` imports a file with no live-move records, realizes 10,000 ordinary rows and appends one through its public view-model list. The test-support `loan_row` counter measures the unmount hook's realized-row visits. Against the original hook, `r5acRed` records 10,000 visits and fails. The fixed append realizes 10,001 rows with zero loan-row visits. There is no wall-time assertion. Existing loan-return and disappearing-holder regressions still pass.

### Formatting and verification

Rustfmt uses edition 2024. All 47 Rust files changed by this branch from `origin/main` were formatted. A full workspace check also found 52 renderer and pixel-compare files whose original bytes matched `origin/main`; those changes are formatting only. For each, the saved original bytes were checked against main, and running rustfmt on those bytes reproduces the current file exactly. The equivalent full `cargo fmt --all -- --check` target set was checked by calling rustfmt directly over 458 roots from 31 workspace manifests. It passes. Cargo and Bazel were not invoked directly.

The focused runtime run `r5green01` passes all 52 tests. This includes the earlier hidden-transfer, field/video identity, list-path pose, layout placement, inherited opacity and wake catch-up cases. The 40-row visit counts remain 81 versus 3 advances and 40 versus 1 draw. The retained single-wake measurement is 0.126458 ms for wake and 1.011417 ms for zero-time settlement. It changes the same 25 components, adding dirt `0x00c4` and removing `0x0001`, with no RENDER_OPACITY re-dirty. These are host observations, not timing thresholds.

The full C API device-shaped probe also keeps one-row and 40-row steady-frame counts equal with 39 asleep: advance/layout/draw/hit 3 each, video enumeration 2, lookup walks 0, semantic validation 2, owner capture 4, checkpoint 1, value graph 9, nested-event drains 6 and loan-row visits 0. Its 20-frame averages are 0.379265 ms for one row and 0.353635 ms for 40 with 39 asleep. These are host measurements.

| Check | Runner or command | Result |
| --- | --- | --- |
| `test -p nuxie-runtime --test live_moves` | `r5green01` | 52 passed, 0 failed. |
| `test -p nux-capi --lib live_moves` | `r5bGreen2` | 11 passed, 0 failed. |
| `test -p nux-capi` | `r5v03` | 222 passed in 22 targets, 0 failed. Includes global models, focus, semantic and video coverage. |
| `check -p nux-capi --all-targets` | `r5v04` | 26 targets passed. |
| `check -p nuxie-runtime --all-targets` | `r5v05` | 235 targets passed. |
| `test -p nuxie-runtime --test live_moves_device_fixture` | `r5v06` | 2 passed, 0 failed. |
| Full workspace formatting | Direct `rustfmt --edition 2024 --check` | 458 target roots passed. |
| Generated Bazel files | `tools/bazel/generate.py --check` | Passed. |
| C surface and layout | `check-nux-capi-surface.py`, `check-nux-capi-layout.py` | Passed. |
| Device harness preflight | `check_live_fixture.py` | Rows 1, 2 and 40 pass. |
| Whitespace | `git diff --check` | Passed. |

Every final runner request exited 0. No flaky-host retry or new main comparison was needed. The runtime all-target check waited for a shared build-pool action, then completed normally. The first green C API run exposed a test setup error at landing: the unbound Page handle did not own the runtime's implicitly mounted rows. The test now exposes the actual mounted Screen B owner through the same native-owner seam used for the shared Unit. All three writes, revisions and notifications pass together.

The device builder and all nine saved RIV, inventory and lifecycle files have unchanged hashes. All six fresh RIV and inventory exports match their original hashes too. The device harness preflight passes for 1, 2 and 40 rows. Verification evidence, the failing baseline logs, formatting proofs and source hashes are saved in `/private/tmp/live-moves-review5`. Request logs remain in the supplied runner directory. All tested Rust source hashes remain unchanged. The 257 new regenerable `target/debug` outputs were removed after verification. The four preexisting debug outputs and platform directories were preserved. No commit was made.

## Review round 6: opt-in occurrence bookkeeping

The starting revision is `66f9023eff`. These changes are uncommitted. Files without live-move records keep the native path. UNIV-5435 formatting in renderer and pixel-compare is outside this change.

### Ownership and gates

`File` records one capability bit while accepting LiveSlot, LiveStep or LiveStepUnit records. After import, only opted-in files mark their source artboards and the nested templates already made during importer resolution. Ordinary imports skip that traversal. Cloning carries this metadata. Only a root runtime occurrence from an opted-in file allocates `hosted_scene::SceneState`; nested instances resolve the state through their existing host ancestry.

The occurrence owns its membership revision, plan and history, flights, hosted-index cache, VM projection and loan registrations. The global ROUTES, REVISIONS, LOAN_LISTS and PROJECTIONS stores are deleted. There is no cleanup sweep waiting for a later index build. The C API's separate controller field is deleted. `LiveMoves` and `HostedIndex` are stateless dispatchers, so retaining an adapter after releasing its occurrence retains no identity storage.

Every gate is explicit:

- File import creates only the capability bit until accepted live-move records opt in.
- Root instantiation allocates scene state only for that file capability. Source definitions and nested templates do not own scene state.
- Attachment changes, nested bound swaps, list clear, structural list edits and row removal resolve the containing scene before touching its revision. Without state they return without allocating.
- Row sleep requires an opted-in occurrence. The optional row-state box holds the collapsed identities, bitmap and active indices. A list without that box returns before building the temporary membership set. Native frame loops use their existing realized indices. Host traversal uses an iterator without a temporary vector. The transform hook immediately delegates to native realized-row traversal when no row is sleeping.
- Slot transport and emptying require an opted-in occurrence before saving props, frozen layout or ownership history. Slot state is an optional box; ordinary nested-artboard transform and layout hooks read None and use native behavior.
- The row-unmount hook returns before membership-set construction or row discovery unless the owning scene has registered loans. Loan identities live in that scene, rather than a thread-global map.
- Plans, following and frame projection return immediately for ordinary roots. Dispatch requires the owning root's state; a hosted child cannot retarget its parent's cached plan. The active and retained hosted-index caches exist only in scene state. Ordinary host queries use transient native traversal.
- VM projection invalidation compares the list properties' existing native change sequences. The global weak observer list and its hooks in VM capture and rollback are removed. No new view-model port hook or subscription is needed. Projection state is owned by the occurrence.
- Scene destruction also clears its active transient frame projection. Frame guards carry scalar tokens, so a stale guard cannot clear a later scene's frame and retains no CoreHandle storage.
- C API media queries use the unchanged main collector and scalar video identities for ordinary files. The retained lookup, sources table, visibility projection and frame cache live in an optional box allocated only for record files. That box belongs to the shared C API occurrence. Ordinary media controls rescan just as main does, so legacy attachments need no membership stamp.

The existing player mutation boundary, effective hidden rule and follower pose rules remain in use. Visit and bookkeeping counters exist only with `test-support`.

### Regressions and verification

The pre-fix run `r6Red2` fails all three count-based regressions. The first ordinary pair leaves 2 revisions. The dropped live scene leaves 10 routes and 9 revisions. Dropping during flight leaves 9 routes, 8 revisions, 1 loan list and 1 projection observer entry. The first request, `r6Red`, failed before compilation because Bazel could not create its installation symlink in the external cache. It was resubmitted unchanged.

| Test | Contract |
| --- | --- |
| `record_free_create_and_free_has_no_bookkeeping_round6` | Runs 128 create/free cycles of two artboards, alternating File instantiation and the legacy definition-clone entry point, mounting two rows too. It also requests active and retained native traversal, a no-op live dispatcher/frame scope, and a rejected sleep call. All bookkeeping counts stay at zero before and after each pair, and after freeing the file. |
| `live_occurrence_builds_and_reclaims_bookkeeping_round6` | Runs 8 live occurrence lifetimes. Each builds a real index and revision, then returns all bookkeeping counts to zero after drop. |
| `dropping_an_in_flight_occurrence_reclaims_all_bookkeeping_round6` | Builds flights, a loan, a sleeping row and a VM projection. Dropping the occurrence releases all bookkeeping, including the active transient frame, while its VM and an old frame guard remain alive. |

| `child_dispatch_cannot_retarget_its_owning_scene_plan_round6` | A child-directed frame query creates no frame or cache and leaves the root plan usable for take-off and landing. |
| `live_moves_record_free_media_queries_keep_no_live_cache_round6` (C API) | Ordinary videos remain enumerable and accept visibility queries. The live media cache stays absent, and native bookkeeping counts stay at zero through free. |

The first rewritten runtime run `r6Green1` passed 37 cases and failed 18. Sixteen failures exposed a hook trying to reborrow the list handle while already mutating it; that hook now uses the borrowed receiver. The other two were test setup assumptions: a record-free Scene helper explicitly emptied overlay slots, and a layout-transport fixture had no opt-in record. The ordinary helper now leaves every mounted instance alone. The separate layout test fixture now authors a LiveSlot. Device fixtures did not change.

The corrected runtime run `r6Green2` passes all 55 cases. `r6Capi1` passes all 12 focused C API cases. The native visit counts remain 81 versus 3 advances and 40 versus 1 draw. In this loaded host run, a wake measured 0.265500 ms and the following settle 133.208334 ms. The same 25 components changed, with added dirt `0x00c4` and removed dirt `0x0001`. Timings are observations, not thresholds; the deterministic visit and dirt assertions pass.

The first complete verification set also passed: `r6Green2`, `r6Capi1`, and `r6v03` through `r6v06`. A final audit then removed the temporary vector from the semantic/host traversal helper, restored native realized-row traversal in the ordinary transform hook, and made dispatcher access root-only. The ordinary loop now alternates File and legacy clone APIs. A new regression verifies that a child-directed query cannot retarget its parent plan. Active transient frame entries are counted too. The complete requested verification set is repeated against these final frozen sources below. `r6final01` passes all 56 runtime cases. Its single wake measures 0.162208 ms and the next zero-time settle 1.529917 ms, with the same 25 components and dirt bits. Visit counts remain 81/3 advances and 40/1 draws. `r6final02` passes all 12 focused C API cases.

| Final check | Request | Result |
| --- | --- | --- |
| Runtime live moves | `r6final01` | 56 passed, 0 failed, exit 0. |
| Focused C API live moves | `r6final02` | 12 passed, 0 failed, exit 0. |
| Whole C API | `r6final03`, retry `r6retry03` | All 223 cases and 22 targets passed in Bazel. The runner then exited 143. Retry is queued. |
| C API all targets | `r6final04`, retry `r6retry04` | Bazel reports 26 targets passed, but the runner exited 143. Retry is queued. |
| Runtime all targets | `r6final05`, retry `r6retry05` | Interrupted during the build, runner exit 143. Retry is queued. |
| Device fixture suite | `r6final06` | Queued, not yet taken. The preceding `r6v06` passed both cases. |
| Branch-only formatting | `rustfmt --edition 2024 --config skip_children=true --check` | All 52 branch Rust files passed. Renderer and pixel-compare were untouched. |
| Bazel generated files | `tools/bazel/generate.py --check` | Passed. |
| C surface and layout | Surface and layout check scripts | Passed. |
| Device preflight | `check_live_fixture.py` | Rows 1, 2 and 40 pass. |
| Whitespace | `git diff --check` | Passed. |

The final sources are frozen. The last four clean runner completions are still pending after the signal-143 interruptions. They are not reported as completed verification. The request files remain ready for the runner. Source hashes, failing baseline logs, passing results, the exact-main ordinary-video collector comparison and formatting evidence are retained in `/private/tmp/live-moves-review6`. The device builder and all nine saved fixture files keep their original hashes. The earlier complete fixture export also matched all six original RIV and inventory hashes. No direct cargo or Bazel command was run. Nothing was committed. New debug outputs are left in place while the runner may still be using them.

## Review round 7: one checkpoint contract and scene-owned VM storage

Eager and lazy frame checkpoints now enter through `begin_checkpoint`. Both capture their initial `ChangeScope` with `reachable_change_owner_snapshot`, retain its native storage through `CheckpointStorage`, and resolve changes with the existing `resolve_captured_changes` rule: an owner must belong to the initial scope or be reachable from the final roots. Capturing rollback state does not admit an owner into that scope. The active graph projection still excludes sleeping rows.

Every `Snapshot`, including provisional properties captured during cloning, retains the same native storage. `CoreHandle::runtime_arena_owner` resolves the root already registered in a runtime arena before children are cloned. This is a small ownership query in ported code. It creates no index, allocation or additional bookkeeping. It works before a new property's component parent or view model owner has been assigned.

The common C API scene-root collector also includes inline VM components owned by active artboards. These are real scene models, including a flight slot's authored model after its fresh child is retired. They are cached in the occurrence's existing index on membership edits. An unused slot's inline model remains an eligible scene root, but the existing projection suppresses its graph walk until it is touched or used by an active context. The native property-access hook activates projected edges for both checkpoint timings. Ordinary files return before creating this cache. This fixes an omission exposed when the stricter transaction validation first rejected provisional flight-prop writes. Neither checkpoint infers writable scope from the write journal.

The lazy journal now only deduplicates first writes, captures snapshots and records overflow. It no longer reconstructs accepted owners from captured properties or performs a second rollback loop. At completion it unregisters first and transfers its snapshots to `Transaction`. The existing eager commit, reverse rollback and notification disposal code completes both modes. Storage remains alive until that completion, including when provisional rows have been replaced.

Moved units' explicit VM storage leases moved from `SlotState::vm_owner` to `SceneState::unit_vm_storage`. They are keyed by unit identity, released on return or replacement, and all dropped with the scene. The scene never leases its own root. A discarded unit is disposed before releasing its VM storage lease. This prevents crossed flight slots from making Screen A and Screen B retain each other. Existing removed-source-row behavior still keeps the moved unit's VM alive while needed.

### Regressions

| Finding | Regression and failing baseline |
| --- | --- |
| 1. Rebinding Page retires captured properties | `live_moves_rebind_default_page_then_step_does_not_poison_round7` uses the real two-row device fixture. It steps, creates and binds a fresh default Page, frees the previous public model handle, then steps twice. It repeats the replacement twice. Before the fix, `r7redrebind` returned `RuntimeError` after `transaction retains native property owner`. |
| 1. Rollback after replacement | `checkpoint_rollback_retains_and_restores_replaced_rows_round7` runs eager and lazy modes, changes the explicit unit model, replaces the Page model and its rows, and frees the old row handles. It checks that checkpoint storage retains the row, rollback restores the scalar and the two-item Screen list, and releasing the checkpoint and final test observation frees the row. Before the fix, `r7rednative` lost the row and panicked restoring the captured property, including in eager mode. The shared storage fix covers both. |
| 2. Scope widened by rollback capture | `frame_still_refuses_writes_outside_its_scope_and_overflowed_captures` now runs the same four scenarios against both modes: unrelated owner refused, newly reachable owner accepted, transient join-and-detach refused, and overflow refused with rollback. `r7redscope` failed the unrelated-owner assertion under `begin_live`. |
| 3. Crossed flights retain rows | `crossed_two_version_flights_release_rows_and_units_round7` sends A's explicit-VM unit to B's flight and B's unit to A's flight. It frees the scene mid-flight and asserts both weak row handles, both weak unit handles and all bookkeeping counts are released. Before the fix, `r7rednative` reported `crossed flight retained a row`. |
| 3. Root-owned storage | `scene_owned_vm_storage_does_not_retain_the_scene_root_round7` imports an extra root-level Unit slot, moves its unit into a row's flight slot, verifies that its VM identity is unchanged, and frees the scene. `r7redrootretry` fails with `scene retained its own root through a VM lease` before the guard. The separate native fixture builder emits this case; device fixtures are untouched. |

The first corrected runtime request `r7runtime1` passed 58 tests. `r7capi1` exposed the scene-root omission above, with 10 scope-validation failures. The corrected shared collector passes all 13 focused C API tests in `r7capi2`. The device-shaped cost probe `r7cost1` then caught 17 versus 15 graph visits from a dormant flight-template model. The projection now suppresses that unused edge without removing the model from valid scene membership. `r7cost3` passes the complete device-shaped probe: one row, 39 of 40 rows asleep, and late semantics all have exactly 15 graph visits, 1 checkpoint, 3 advances, 3 draws and zero index rebuilds per frame. In flight, graph visits remain 15 and checkpoints remain 1. The Linux host measured 1.062107 ms per one-row step, 1.166852 ms with 39 sleeping rows and 1.056260 ms in flight over 20 frames. Timings are observations, not thresholds. `r7cost2` was an infrastructure failure before compilation because the pool could not execute rustup or produce its result archive. The additional root-owned-storage baseline was delayed by pool capacity: `r7redroot` timed out after 30 minutes without compiling, and `r7redrootretry` was admitted after roughly 18 minutes, then reproduced the leak. The final source adds the root lease guard and preserves the storage lease through unit disposal. Verification ran sequentially through the supplied runner. The full C API suite ran once, last. Device fixture bytes and the device builder remain unchanged. The runner uses its configured `-Cdebuginfo=0`; no build configuration was edited. Round 6's two all-target checks and device fixture suite were confirmed passing by the user before this round.

### Final verification

| Check | Request | Result |
| --- | --- | --- |
| `test -p nuxie-runtime --test live_moves` | `r7verifyruntime` | 59 passed, 0 failed, exit 0. Includes both checkpoint timings, crossed flights and scene-root VM storage. |
| `test -p nux-capi --lib live_moves` | `r7verifycapi` | 13 passed, 0 failed, exit 0. Includes rebind, shared-player state, overflow poisoning, exact publication and global-only subscriptions. |
| `check -p nux-capi --all-targets` | `r7checkcapi` | All 26 targets built successfully, exit 0. |
| `check -p nuxie-runtime --all-targets` | `r7checkruntime` | All 235 targets built successfully. The runner then exited 2 because artifact collection exceeded its 2 GiB limit. This is not reported as a clean runner exit. |
| `test -p nuxie-runtime --lib graph_transaction_tests` | `r7transactions` | 5 passed, 0 failed, exit 0. The scope test runs all four scenarios with eager and lazy checkpoints. |
| `test -p nux-capi` | `r7wholecapi` | All 22 targets passed: 224 tests, 0 failures, 0 ignored, exit 0. The whole suite ran exactly once. |
| Branch formatting | `rustfmt --edition 2024 --config skip_children=true --check` | All 41 branch Rust files passed. Renderer and pixel-compare files were untouched. |
| Generated files and whitespace | `tools/bazel/generate.py --check`, `git diff --check` | Passed. |
| Device fixtures | Saved SHA-256 hashes and `check_live_fixture.py` | Device builder and all nine fixture files unchanged. Preflight passes rows 1, 2 and 40. |

The runtime all-target log contains `INFO: Found 235 targets` and `INFO: Build completed successfully, 1933 total actions`, followed by `ValueError: native artifacts exceed 2 GiB`. Compilation completed without a source error. The limit belongs to the runner's result collection. There are no failing test cases in the final test runs, so no main comparison was needed for a test failure. The collection limit remains an infrastructure limitation.

The final whole C API run repeats the device-shaped cost probe on the final sources. One row, 39 of 40 rows asleep, and late semantics have identical complete visit counts: 3 advances, 3 layout visits, 3 draws, 3 hits, 2 video enumerations, 0 video lookups, 2 semantic validations, 4 owner captures, 1 checkpoint, 15 graph visits, 6 event drains and 0 index rebuilds per frame. Flights keep 15 graph visits and 1 checkpoint, with 5 layout visits. Over 20 frames, the Mac host measured step times of 0.269944 ms for one row, 0.308085 ms with 39 sleeping rows and 0.261408 ms in flight. The final runtime wake measured 0.122958 ms and its settle 1.287875 ms, with 25 changed components and dirt `+0x00c4`, `-0x0001`. Timings are observations, not thresholds.

Source hashes, all request logs, failing baselines, passing test summaries, formatting and preflight evidence are retained in `/private/tmp/live-moves-review7`. Runner-created `.nuxie` output was moved there as `runner-output`. The preexisting `target/debug` files were preserved; the one new generated target file was moved to the same evidence directory. Source changes remain uncommitted. No direct cargo or Bazel command was run.

## Earlier implementation and fixture verification

Rust builds and tests used only the supplied request-file runner. No cargo or Bazel command was run directly. Nothing was committed. This fixture correction changes only `live_move/device_fixture.rs`, its integration test, and this report. Runtime behavior, ported files, records, generated schema and the C API are unchanged by this correction.

| Check | Runner or command | Result |
| --- | --- | --- |
| Fixture export and lifecycle playback | `life03`, `life06runtime` | 2 tests passed. Both two-row and forty-row pages play forward and back twice with the lifecycle guards, progress samples and completion events described above. All three pages export successfully. |
| Focused runtime suite | `life06runtime`, target `live_moves` | All 37 cases passed, including Lua continuity, distinct-artboard two-version draw order and fade. |
| Whole `test -p nux-capi` | `life04capi` | All 22 targets passed: 215 cases, 0 failures. Includes both device-shaped probes and the 4 guarded live-move unit cases. |
| `check -p nux-capi --all-targets` | `life05check` | Passed, 26 targets. |
| Whole `test -p nuxie-runtime` | `life06runtime` | All 233 targets finished: 2,461 cases passed, 197 failed, 11 ignored. All failures match current main as shown below. |
| Device harness preflight | `check_live_fixture.py --emit-contract` | Passed for rows 1, 2 and 40. Video inventories, authored one-second flight tracks and lifecycle completion declarations validate. A separate structural check finds zero progress, path or fade VM bindings. |
| Bazel generated files | `python3 tools/bazel/generate.py --check` | Passed. |
| C surface and layout | `tools/check-nux-capi-surface.py`, `tools/check-nux-capi-layout.py` | Passed. |
| Formatting and whitespace | `rustfmt --check`, `git diff --check` | Passed. |

The earlier unchanged schema reproducibility check `correct06schema` passed with 373 definitions and 815 properties. Namespace checks `correct11namespace` passed 3 cases. The shipping check `correct10shipping` and both static and dynamic export inspections matched all 152 portable exports. No schema or ABI source changed in this fixture correction.

The C API suite, all-target check and focused fixture runs used the normal repository setting. For the completed whole runtime run and the fresh main comparisons, `.bazelrc` temporarily used `-Cdebuginfo=0` instead of `-Cdebuginfo=1`, following the disk exhaustion documented by the earlier `correct09runtime` and `correct13runtime` attempts. Optimization and debug assertions were unchanged. The original build configuration was restored and verified byte for byte.

### Fresh baseline comparison

The baseline is current `origin/main` at `be395daa6c67175bd85e2e2f84f934102aa090c5`. This includes the CSS fix added since the previous baseline. All 62 relevant source and configuration paths were saved before loading main. The loaded tree matched that revision, with only the shared debug-symbol setting different. Every saved path was restored and verified byte for byte after the ten baseline runs, then the original build setting was restored.

Every failed case requires the missing pinned C++ oracle at `tools/cpp-probe/build/macosx/bin/debug/rive_cpp_probe`. The comparison matches each suite's passing and failing counts, every failed test name, and every panic message.

| Suite | Failed cases in both runs | Fresh main runner |
| --- | ---: | --- |
| `cpp_probe_audio_native` | 2 | `lifebase01` |
| `cpp_probe_data_bind_enum_native` | 32 | `lifebase02` |
| `cpp_probe_data_bind_native` | 51 | `lifebase03` |
| `cpp_probe_data_bind_string_color_native` | 38 | `lifebase04` |
| `cpp_probe_draw_native` | 17 | `lifebase05` |
| `cpp_probe_host_contracts` | 1 | `lifebase06` |
| `cpp_probe_import_animation` | 7 | `lifebase07` |
| `cpp_probe_native` | 14 | `lifebase08` |
| `cpp_probe_transitions_native` | 34 | `lifebase09` |
| `profiler_cpp_probe` | 1 | `lifebase10` |

All 197 failures match fresh main. Current and main logs, names, messages, totals, source restoration evidence, build-setting hashes, preflight results and six fixture hashes are retained in `/private/tmp/live-moves-runtime-v2-correction-verification`. Request logs remain in the supplied `runtime-runner-v2` directory.

The assets and lifecycle contracts are ready for the device harness. Physical device qualification remains unrun here. This verification preceded the section 8 change below.

## Review round 8: reconcile the mounted occurrence

The mounted native scene is authoritative. `SceneState::reconcile` runs at `Plan::prepare`, before its cached-plan check. Direct move validation and empty-slot operations share the same reconciliation. Row sleep and wake enter the occurrence's preparation, and `step_end` prepares before validating its saved flights. A matching occurrence revision returns immediately. A changed revision reuses the hosted index, including sleeping rows, to refresh slot state, row masks, loan membership and reachable unit storage leases. It does not walk retained rows on unchanged frames.

The index now retains the host identities it already discovers during a structural rebuild. Reconciliation compares each remembered slot's mounted child identity with its actual native child. Ordinary bound replacement invalidates the old empty-slot definition, prop templates, listeners, placeholder and flight override together. Occupied-slot compatibility reads the current child's definition directly. Live transport records its own child changes, so take-off and landing preserve authored empty-slot templates. No replacement-specific cache invalidation hook was added.

A unit VM storage lease survives only while that unit remains mounted in the occurrence. Reconciliation releases leases whose units disappeared during native removal or Page rebinding. Removed storage is released outside the map borrow, so native cleanup can query the scene safely. A mounted unit still keeps its original VM storage when its source row alone is removed.

Occurrence construction derives live-move capability from concrete record metadata in the actual source definition arena. The arena is the file's existing definition storage, including tools replacements. This query creates no inventory or live-move allocation and runs only when constructing a root occurrence. It reads existing slot type metadata without borrowing `File`, so it also works during native import and tools callbacks. The copied `Artboard::live_move_enabled`, imported `File::has_live_move_records`, and `enable_definitions` traversal are deleted. Ordinary runtime hooks resolve existing scene ancestry and return without bookkeeping when there is no occurrence state.

### Regressions and verification

| Finding | Regression | Baseline evidence |
| --- | --- | --- |
| 1. Retired storage after Page rebinding | `fresh_page_rebind_releases_retired_flight_rows_round8` repeats take-off and fresh default Page binding four times. It checks both retired rows and both unit instances are released after each preparation, and that the unit-storage map is empty. | `r8red`: `retired row retained by a scene lease`. |
| 2. Current slot definition | `bound_slot_replacement_rejects_current_incompatible_definition_round8` remembers both slots, replaces them through the native bound-replacement API with matching wrappers but different artboards, then checks refusal and unchanged ownership. | `r8red`: incompatible replacement passed validation. |
| 2. Current prop templates | `bound_slot_replacement_adopts_fresh_prop_templates_round8` replaces compatible children and supplies a new destination prop binding to `count`. The transported unit must receive 200 from Screen B. | `r8redprops`: `expected 200, got 0`. The initial test setup tried to edit an already retired binding; this corrected case supplies the fresh binding explicitly and reproduces the stale template. |
| 3. Tools replacement capability | `tools_replaced_live_page_uses_its_definition_records_round8` replaces Page with its own authored byte run, creates a new occurrence and runs take-off and landing. | `r8red`: `file has no live-move records`. |

All three findings have confirmed failing baselines on `956d56861c`. The first run completed four cases, including the prop-test setup error described above. The corrected prop baseline completed one case. Baseline logs and frozen source snapshots are saved in `/private/tmp/live-moves-review8`.

Patched Rust verification is pending. The runner completed `r8redprops`, then stopped claiming requests. `request-r8runtime` contains `test -p nuxie-runtime --test live_moves` and remains the only pending request. No patched Rust result is claimed. The remaining C API and runtime checks will run sequentially when the watcher resumes, with the whole C API suite last.

Local checks pass: 28 Android contract cases and both Android plan commands, 41 distribution contract cases, the shipped surface and layout checks, generated-file checking, and rustfmt checking for all 40 branch Rust paths. The device builder and all nine saved fixture artifacts match their saved SHA-256 hashes. Fixture preflight passes rows 1, 2 and 40. The required fixture-export runner test remains pending with the Rust verification.


## Review round 9: restrict the compiler shape and remove templates

Live moves support an unhosted Page root whose `screens` ArtboardComponentList directly holds Screen artboards, with marked NestedArtboard slots inside those Screens, including their flight layers. Paint-hosted artboards, other nested content and other lists keep ordinary runtime behavior. They contribute to the enclosing Page's hosted index, but never own another live-move scene or row index. Import diagnoses and ignores records outside that definition shape; runtime entry points also check the actual mounted path. A valid Screen definition used as ordinary nested or paint content is still ordinary content.

The generic artboard-instantiation scene allocation is removed. An explicit live operation creates the eligible Page's occurrence state lazily; native structure hooks only resolve existing enclosing scene ancestry. Thus a paint list edit or nested child replacement invalidates the Page's index instead of a private paint index. Source-definition capability remains derived from native record metadata. No copied import flag or replacement-specific invalidation hook is added. Generalized ancestor-list discovery and row sleep or transport on arbitrary nested lists are removed. The earlier root-owned Unit slot regression becomes a refusal and import-diagnostic test; the old shape is unsupported.

`SlotState` no longer stores cloned prop templates. On departure, original authored parent bindings leave their active parent container and enter their native inline view model's existing value-binding collection. They retain their native targets and owned converters. At arrival, transport derives the slot's current bindings from that inline model and the mounted parent container, giving fresh native bindings priority. Only the transported unit receives retargeted instance bindings, whose existing native deletion path retires the prior bind and converter on each adoption. The per-slot cloned-template allocation and its invalidation lifecycle are deleted.

Mutable list collapse records completion while borrowed and reconciles after `CoreHandle::with_mut` releases the payload. It uses the same effective hidden-state rule as handle-based collapse. No borrowed list is followed through its own row's host ancestry.

Status-only live operations still use the shared player mutation boundary. Successful operations retain raw script host commands on their player. The next successful external step prepends them to that step's commands and publishes the combined result once. Pending commands participate in the existing aggregate result-size check, retain production order, and clear only after successful result and mutation publication. No C API is added.

### Regressions and verification

| Finding | Regression | Evidence |
| --- | --- | --- |
| A. Paint content owns a private scene | `live_moves_paint_content_edits_invalidate_the_enclosing_index_round9` appends a Screen occurrence inside a paint-hosted unit, checks two new videos, replaces its nested child, checks its retired weak handle and the new video id, then removes the row. The paint instance never acquires scene state. | `r9redadproof` reproduces two scene bookkeeping entries instead of one. `r9paintreal` passes all 15 focused C API cases. The corrected test replaces with a distinct unit definition: replacing with the paint ancestor itself is refused by the native ancestor guard. It refreshes video inventory and the next frame before checking release of the replaced child. |
| A. Unsupported nested and root shapes | `misplaced_page_slot_is_ignored_and_cannot_transport_round9` checks the import filter and refusal without ownership changes. `ordinary_nested_row_cannot_wake_a_sleeping_screen_round9` rejects row sleep on an inner list while preserving the enclosing Screen's hidden semantics, deferred binds and semantic id. | `r9redbfullfix` fails these new shape restrictions under the old generalized implementation. |
| A. Endpoint-only Screen classification | `only_step_endpoints_keep_slot_records_regardless_of_default_rows_round9` imports empty and populated default lists. A, B and R2 are normal step endpoints; R3 is a zero-unit cut endpoint; R4 carries slots but is never named. Only R4 loses its records, in both files. | `r9redbfullfix` retains R4 slots with empty defaults, failing the expected refusal. Patched focused runtime verification passes this case. |
| B. Prop-template leaks | `retained_row_binding_count_stays_flat_after_replacement_and_transport_round9` supplies fresh native bindings and converters on each of nine compatible replacement and round-trip cycles. Native property removal retires each previous authored binding. Both retained row arenas must keep exactly their warm-up binding and converter counts; the adopted prop must still equal the authored Screen A value, 10. | `r9redbfullfix` reproduces growth in each row from `(4 bindings, 2 converters)` to `(6 bindings, 4 converters)` on the next replacement cycle. Patched runs stay flat at `(2 bindings, 1 converter)` per row for all nine cycles. |
| C. Mutable list collapse borrows itself | `mutable_list_collapse_reconciles_after_its_borrow_round9` sleeps one row, collapses and uncollapses its list through a mutable handle, then checks the awake and sleeping slots. | `r9redbfullfix` reproduces `RefCell already mutably borrowed` and the regression's explicit panic assertion. Patched focused runtime runs pass. |
| D. Script commands discarded by a status result | `live_moves_script_commands_are_delivered_by_the_next_step_once_round9` imports a real compiled Lua drawable with a number input. Take-off emits 200 and landing emits 321. The next external step must return those commands and payloads in order; the following step must return none. | `r9redadproof` reproduces zero returned commands instead of two, with the Lua source gate corrected to Screen A's authored value of 10. `r9capigreen` passes with ordered payloads 200 and 321 and an empty second result. |

Verification resumed after the machine-wide disk cleanup. The interrupted `r9capilive` request was resubmitted with a new id. Early macOS pool attempts failed in dependency procedural macros before compiling the tests; subsequent Linux requests ran successfully. All requests are sequential and use the runner with debug information disabled.

The provisional binding loop held a container read borrow during removal. It now copies native handles before mutation. The corrected binding regression supplies fresh bindings before each transport and confirms the authored value is 10. The command fixture's source gate also uses 10. Two layout regressions formerly placed slots directly on the root; their fixtures now mount the same layout through Page/screens/Screen without changing the coordinate assertions.

Final verification is in progress. Completed runner checks:

| Request | Command | Result |
| --- | --- | --- |
| `r9runtimefinal` | `test -p nuxie-runtime --test live_moves` | 66 passed, 0 failed. |
| `r9paintreal` | `test -p nux-capi --lib live_moves` | 15 passed, 0 failed. |
| `r9transactions` | `test -p nuxie-runtime --lib transactions` | 5 passed, 0 failed. |

The collapsed-row test retains advance visits 81 versus 3 and draw visits 40 versus 1. For 100 advances it measured 170.31 ms expanded, 8.30 ms collapsed, and 8.30 ms for the one-row baseline. One wake measured 0.28 ms, followed by 1.78 ms settlement; 25 components changed dirt. These are test-host observations, not device qualification.

The Android contracts and distribution contract gate pass. All 116 branch Rust paths pass rustfmt with their crate editions, generated files are current, and `git diff --check` is clean. Renderer files were untouched. The device builder and all nine saved artifacts still match their starting SHA-256 hashes, and preflight passes rows 1, 2 and 40. The fixture export request remains pending. The all-target request `r9capicheckfinal` initially hit the known macOS `zerofrom_derive` pool fault. After the coordinator fixed the pool, that same held request resumed automatically. It compiled all 26 targets, then the runner refused artifacts over its 2 GiB limit and returned exit code 2. No duplicate request was submitted. The full C API suite has not yet run.

The compiler identifies Screen definitions exclusively through the Page root's `LiveStep.fromArtboard` and `LiveStep.toArtboard` endpoints. The Page's default `screens` list is empty by contract; the SDK populates it at runtime. Every artboard carrying a LiveSlot is an endpoint of at least one Page step. Fade steps and cuts with overrides also have zero-unit LiveStep records. Import diagnoses and ignores LiveSlots on unnamed artboards. No view model defaults or list item types participate in Screen classification. Runtime entry points independently enforce the mounted Page/screens/Screen path.

Baseline logs, available frozen source snapshots and local checks are retained in `/private/tmp/live-moves-review9`. The user confirmed all round 8 checks passed on `51ee009249`; the preceding round 8 pending-runner note records that earlier session's state.

### Android Back followed by outgoing-row removal

`live_moves_remove_outgoing_row_after_back_then_step_round9` drives the unmodified two-row device fixture through the C API. It writes lifecycle phases and names, runs `t1` to its authored arrival event, lands and sleeps A, performs the no-op Back list move and zero-time step, wakes A, and runs `back` to its authored arrival event. With row order still `[A, B]`, it releases the acquired row VM handles, removes `screens[1]` through `nux_view_model_mutate`, and requires two ordinary 1/60-second steps to succeed. There is no SDK removal hook in this test.

The fixture authors `nx_enter_done:t1` and `nx_enter_done:back`, with no outgoing completion event; the regression waits for each authored arrival before landing. Request `r9backremovebaseline` runs this case against the current round 9 runtime, before any runtime fix for this report. It remained queued and untaken after 90 minutes, behind the existing UNIV-5510 pool hold. Disk space was sufficient (195 GiB at the last check). No held verification request was resubmitted. No pass, failure, cause or earlier-fix attribution is claimed until that request completes. The device fixture bytes and builder are unchanged.

### Round 9d resumed verification

The coordinator confirmed the pool repair and resumed `r9capicheckfinal`. `r9backremovebaseline` remains queued behind it. The requested final sequence is runtime live moves, focused C API live moves, transaction unit tests, C API all-target checking, device fixture export, and the full C API suite. A new pool failure will be reported without a retry or workaround. Results below will be added only after the corresponding `done-` files exist.

### Round 9d runner artifact-limit failure

`r9capicheckfinal` completed all 1,017 build actions and reported all 26 targets built successfully. The runner then reported `Native request refused: ValueError: native artifacts exceed 2 GiB; owned evidence retained`, with pool result 1 and request exit code 2. This is not a passing end-to-end check. In accordance with the instruction to stop on pool failures, no retry, target splitting or artifact-limit workaround was attempted.

| Request | Command | Round 9d result at stop |
| --- | --- | --- |
| `r9capicheckfinal` | `check -p nux-capi --all-targets` | Compilation succeeded for 26 targets; runner failed its 2 GiB artifact limit, exit 2. |
| `r9backremovebaseline` | `test -p nux-capi --lib live_moves_remove_outgoing_row_after_back_then_step_round9` | Runner started it automatically after the check; no verdict yet. |
| `v9suite1` | `test -p nuxie-runtime --test live_moves` | Queued by the user, not resubmitted. |
| `v9suite2` | `test -p nux-capi --lib live_moves` | Queued by the user, not resubmitted. |
| `v9suite3` | `test -p nuxie-runtime --lib transactions` | Queued by the user, not resubmitted. |
| `v9suite4` | `test -p nuxie-runtime --test live_moves_device_fixture` | Queued by the user, not resubmitted. |
| `v9suite5` | `test -p nux-capi` | Queued by the user, not resubmitted. |

The build request took 4,114 seconds. Its retained evidence is `.nuxie/homelab/0e9c305a-ede5-4290-8883-a4ee7847153f/evidence`, and its complete log remains `result-r9capicheckfinal.log` in the supplied runner directory. The Back/removal regression has no result yet, so no cause or earlier-fix attribution is claimed. No runtime source changed during this resumed verification. All queued requests remain untouched.


## Review on the rebased change

Review of `1eca21db4e` found that record-free frames repeatedly scanned the shared file arena because only positive capability decisions were retained. Each root occurrence now remembers an unchecked, negative, or positive live-move Page decision. Negative decisions allocate no scene. Explicit-operation opt-in and Screen classification are unchanged; definition replacement creates a new occurrence with an unchecked decision.

`record_free_frames_scan_capability_once_per_occurrence` imports a record-free file with ten artboards and plays only the small mascot for 128 frames, leaving the other nine unused. It exercises frame scope, post-advance handling and the capability query used by media, with fresh adapters each frame. The existing test-support walk counters count actual arena capability scans. Before the fix, `reviewred` failed with 384 scans instead of 1; after the fix, the same regression passes with exactly 1 and no scene bookkeeping. The tools-replacement regression also passes.

All verification used the rebase file-request runner, one request at a time:

| Request | Command | Result |
| --- | --- | --- |
| `reviewruntime` | `test -p nuxie-runtime --test live_moves` | 67 passed, exit 0. |
| `reviewcapi` | `test -p nux-capi --lib live_moves` | 16 passed, exit 0. |
| `reviewprobe` | `test -p nux-capi --test live_moves_collapsed_device_probe` | 1 passed; zero steady-state capability scans, exit 0. |
| `reviewcheck` | `check -p nux-capi --all-targets` | 26 targets built, exit 0. |
| `reviewfull` | `test -p nux-capi` | 271 tests across 22 targets passed, including one cached target, exit 0. |

Only the three touched Rust files were formatted with `rustfmt --edition 2024 --config skip_children=true`. No schema definitions or generated files changed. Changes remain uncommitted.


### Detached Screen writes and retired parents

The iOS Page host exposed an existing parent-walk assumption in `mark_observable_mutation`: a retained Screen view model can name a parent whose native arena has retired. Mutation propagation now skips handles that no longer resolve to a `ViewModelInstance`, before recording a generation for them. Live instances keep the same generation propagation and identity-based seen set. The neighbouring value-policy retention walk skips the same retired parent edges. `reachable_change_owner_snapshot` was inspected: it walks children with fallible access, not parents, so its handling of invalid child graphs remains unchanged.

Three regressions use the A/B device fixture, retain SDK row handles, and explicitly model the reported weak backlink by retiring a native parent arena. They cover removal followed by a write and step; a write followed by removal in one mutation batch and step; and a completed removal step followed by late effects captured, resolved and published through the runtime frame APIs. All verify detached and live values survive, and actual live parents still receive the changed child's generation. The separate detached-write case also verifies the former Page receives no generation update. Plain removal ordering alone passed before the fix (`detachedred`); the explicit retired-parent cases all reproduced the original `native instance` panic (`detachedredparents`). The strengthened late-effect version also failed all three cases before the fix (`detachedeffectsred`).

After the fix, `detachedcapi` exposed an incorrect new test assumption that Page must remain a direct parent after removal. The assertion now checks the actual live parent handles; no runtime behavior was changed to satisfy it. Final verification used the rebase runner sequentially:

| Request | Command | Result |
| --- | --- | --- |
| `detachedruntime` | `test -p nuxie-runtime --test live_moves` | 67 passed, exit 0. |
| `detachedcapifinal` | `test -p nux-capi --lib live_moves` | 19 passed, including all three regressions, exit 0. |
| `detachedfull` | `test -p nux-capi` | 274 tests across 22 targets passed, exit 0. |
| `detachedcheck` | `check -p nux-capi --all-targets` | 26 targets built, exit 0. |

Only the three touched Rust files were formatted with `rustfmt --edition 2024`. Formatting and whitespace checks pass. No schema or generated files changed, and nothing was committed.


### Android stage-10 snapshot read on 5ef20c0727

Verdict: (c), a stale presented snapshot read. In `android-20261024T125558Z-49097/checks.log`, native typing succeeds before `readNativeText` returns status 9. The activity's stage-10 block types and sets stage 11 without returning. If both elapsed-time thresholds have passed, the following stage-11 block reads through the old `semantic` capture in the same `actions()` call, before `presented()` refreshes it. The `native-write` line records successful select-all plus text focus inputs through zero-time player steps, not a fresh semantic capture.

The C API contract requires field reads to use the current acknowledged presentation (`semantic_snapshot.rs`, `nux_player_field_string_copy`, `nux_player_validate_semantic_snapshot`, and `nux_player_semantic_snapshot`). A changed typing step advances the runtime revision. Old snapshots remain invalid even after presentation, and new captures are refused until that revision is acknowledged as presented. The complete semantic snapshot implementation is identical at round 6 (`bf44dcf75a`) and this head. Its revision authorization check originated in `658d2a3106` (owned presentation-bound semantic captures), before round 6. The earlier emulator pass does not make the same-frame read valid; the harness's elapsed-time fall-through depends on when stage 10 is reached.

The harness should save the capture serial at the edit, return after changing to stage 11, and defer its runtime text assertion until a successful renderer presentation has acknowledged the new revision and `presented()` has installed a newer snapshot that validates. Merely capturing again immediately after typing is insufficient. Presentation must not be acknowledged early just to authorize the read. Keep the native editor and field identity; replace the stale capture and refresh field metadata through the existing presentation path.

`live_moves_native_typing_requires_a_fresh_presented_snapshot` uses the two-screen device fixture, writes `hello` then `hello there` using the harness's key-down/key-up/text order, and confirms the held snapshot returns HANDLE_MISMATCH. A premature fresh capture also fails. After the presentation receipt and a fresh capture, the correct string is readable with the same field-owner identity. No runtime or SDK implementation changed.

| Request | Result |
| --- | --- |
| `snapshotorder` | Test compile error from comparing a bool to an integer; corrected in the test. |
| `snapshotorderfixed` | New focused test passed, exit 0. |
| `snapshotcapi` | `test -p nux-capi --lib live_moves`: 20 passed, exit 0. |
| `snapshotfull` | `test -p nux-capi`: 275 tests across 22 targets passed, exit 0. |

The test file was formatted with `rustfmt --edition 2024`. Whitespace checks pass. No commits were made. This diagnoses the Android failure; it does not claim a new emulator run or completion of that run's collapsed-budget stage.

### Read-only import validation after review of 5f4bb83701

Replacement validation previously retired misplaced records from every definition in its candidate view of the file. A replacement Page naming different endpoints could therefore strip the installed Screens before a duplicate mapping made `validate_step_units` reject it. Disposing the rejected Page did not restore those shared definitions.

Import shape validation now only diagnoses misplaced records. It never releases their object slots or retires them. The mounted Page occurrence remembers its step endpoint names, and plan discovery and explicit slot operations ignore records outside that classification and the existing Page/screens/Screen path. A root without a `screens` list cannot opt in merely because it retains misplaced steps. Definition replacement gives new occurrences their own classification; retained old occurrences keep their original Page classification. No post-commit cleanup or schema changes are needed.

The regressions cover rejection after naming different Screens, successful replacement that stops naming A, original-file misplaced slots, unnamed rows with both empty and populated defaults, and misplaced steps without `screens`. They check record preservation, transport refusal, unchanged ignored flight contents, and a successful live step from a fresh occurrence after rejection. The diagnostics remain visible in runner output. Existing round 9 tests now assert preservation plus use-time refusal instead of record deletion.

`importreadonlyred` reproduced deletion in the original-file and successful-replacement cases. Its initial rejection fixture was dropped by individual record validation, so it was corrected to duplicate an otherwise valid mapping. `importreadonlyred2` then failed all four selected tests, including `Malformed` followed by missing original A records. Verification results follow below.

The first focused C API run (`importreadonlycapi`) passed 19 cases and failed one obsolete round 9 assertion that paint-content steps had been deleted. That assertion now requires the records to remain, while explicitly refusing scene opt-in and step execution on the paint occurrence. Production code did not change in response to that failure.

| Request | Command | Result |
| --- | --- | --- |
| `importreadonlygreen` | `test -p nuxie-runtime --test live_moves import_validation` | 5 passed, exit 0. |
| `importreadonlyfinal` | Same focused command, with retained-old-Page coverage | 5 passed, exit 0. |
| `importreadonlyruntime` | `test -p nuxie-runtime --test live_moves` | 70 passed, exit 0. |
| `importreadonlycapifinal` | `test -p nux-capi --lib live_moves` | 20 passed, exit 0. |
| `importreadonlyfull` | `test -p nux-capi` | 275 tests across 22 targets passed, exit 0. |
| `importreadonlycheck` | `check -p nux-capi --all-targets` | 26 targets built, exit 0. |

Only the five touched Rust files were formatted with `rustfmt --edition 2024`. Formatting and whitespace checks pass. No generated files changed, no pool failure occurred, and nothing was committed.

### Preserve authored prop binding order during live moves (UNIV-5289)

Departure transferred slot prop bindings through import initialization, which changed two-way, target-first bindings to source-first. Ownership transfer now uses `retain_existing_value_data_bind` to preserve the authored flags; import initialization still normalizes them. The `nuxie-runtime` regression `a_target_first_two_way_prop_keeps_its_order_through_a_move_and_home` in `tests/live_moves.rs` checks the core's derived binding flags after landing and returning home, plus a target write reaching the destination Screen's source after landing. The test has been added but not run in this sandbox.

## Section 8: the next Screen's values during a flight

Ported onto the merged core at `41d71302d3`. Arrival still uses `transport::adopt_props`: `slot_props` derives the current authored bindings from the native inline model and mounted parent, preserving fresh-binding priority and native ownership. No slot templates or separate attachment mechanism are restored. Take-off attaches the landing props directly, avoiding intermediate adoption of the flight placeholder's values. Outgoing bound-text detection uses that same native prop derivation plus the unit's mounted occurrence bindings.

Number and color starts belong to `FlightValues` on the flight. Bindings hold weak projections; ordinary updates hold their targets until the follower samples the live converted source with the exact clamped progress used for its pose. Numbers interpolate linearly; colors use the existing `shapes::paint::color::color_lerp`, including alpha and channel rounding. Nonnumeric props switch immediately. Each version uses its own landing slot. Private values remain untouched. Landing, cuts and same-advance flights adopt the latest source without requiring a frame at progress 1.

Blended values never write back. Attachment requests source-only adoption before normal two-way behavior resumes, retaining authored direction flags. Dropping a projection queues ordinary source settlement if its binding survives cancellation or reconciliation; it performs no value write itself. Flight snapshots do not retain projections. Following visits only active flights, with no new retained-row scan. Existing player mutation capture, checkpoint and exact-owner publication cover all writes; the C API frame loop is unchanged.

Occurrence props target unit VM properties. Edited TextInputs are traced through their internal text bindings to local string properties without outgoing occurrence bindings. An incoming prop targeting that property receives the typed text before adoption: its to-source direction and reverse converter are used when available, otherwise a string VM source receives a direct native setter write. One-way non-string sources are left alone. Authored text does not count as a user edit, and internally bound text inputs retain their bindings and identity.

All 13 runtime regressions are ported in `nuxie-runtime`, `tests/live_moves.rs`, using the core’s current Page / `screens` / Screen fixture builder:

- `incoming_number_and_color_blend_then_bindings_drive_after_landing`
- `incoming_values_follow_live_destination_changes_during_flight`
- `incoming_string_switches_at_takeoff_and_private_values_stay`
- `unbound_typed_text_writes_the_incoming_prop_source_in_both_directions`
- `authored_unbound_text_does_not_override_the_incoming_prop`
- `takeoff_and_landing_without_an_advance_adopt_destination_values_at_once`
- `two_versions_blend_against_their_own_landing_slot_values`
- `values_follow_the_same_native_keyed_progress_as_the_pose`
- `converted_two_way_numbers_blend_without_writing_back_presentation_values`
- `outgoing_bound_typed_text_does_not_write_the_incoming_source`
- `target_first_two_way_midpoint_landing_adopts_source_before_writeback`
- `target_first_two_way_return_after_midflight_row_removal_adopts_home_source`
- `target_first_two_way_expired_projection_adopts_live_source_before_writeback`

The C API regression in `nux-capi`, `src/live_move_tests.rs`, is `live_moves_blended_midpoint_step_publishes_to_the_shared_owner_observer`: begin, keyed half-second step and end yield 10/105/200 and invalidate the shared-model observer's presentation revision, with change notification. Existing immediate-publication and command-ordering tests set progress to 1 to preserve their original assertions. The new midpoint case covers blended publication. Existing record-free and collapsed-row visit regressions are retained.

Compared with the earlier version, only transport's native prop lookup/ownership integration and the current fixture/test context changed. The small upstream binding and text hooks remain compatible with main; main's newer text hit-testing hooks are preserved. No dependencies, schema changes, public APIs or new test files were added. `live_move/values.rs` is a new runtime module. Rustfmt and whitespace checks are local only; compilation, test execution and Android size measurement await the caller's build pool. Earlier branch results are not claimed for this port.
