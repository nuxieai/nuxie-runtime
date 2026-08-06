# CommandQueue / CommandServer test ledger

Pinned oracle: `rive-runtime@4ac7b32798da0482e441ef09304dc3b480ed3ee5`,
`tests/unit_tests/runtime/command_queue_test.cpp` (83 `TEST_CASE`s).

Register status: F3 is **CLOSED at 83/83** and A6's former “no command server”
premise is closed. The four S4-45 blob cases were direct upstream runtime
behavior, so they are ported here rather than assigned to an editor or Apple
product layer. Flow-session equivalence remains shared-product evidence and is
not a baseline command-queue gap.

The lane-focused command is:

```text
cargo test -p nuxie --test command_queue --no-fail-fast
```

## Complete upstream case ports (83)

| Upstream `TEST_CASE` | Rust evidence |
|---|---|
| `POD Stream RCP` | `pod_stream_rcp` — ports the RCP stream to Rust's owned command-enum transport and moves both non-null and null `Option<Arc<_>>` payloads through the queue while preserving shared identity. |
| `artboard management` | `artboard_management` — uses pinned `two_artboards.riv`, covers named/missing instances, invalid deletion, and file-owned cleanup. |
| `state machine management` | `state_machine_management` — uses pinned `multiple_state_machines.riv`, covers named/missing instances and dependent cleanup. |
| `default artboard & state machine` | `default_artboard_and_state_machine` — uses pinned `entry.riv` and proves default helpers equal empty names, including authored names. |
| `invalid handles` | `invalid_handles` — ports good/bad file, artboard, and state-machine handle creation, invalid deletion, non-destructive errors, and valid cleanup. |
| `draw loops` | `draw_loops` — independently schedules both keys, each key alone, idle polls, and teardown-safe one-shot draw callbacks. |
| `test support for all asset types` | `test_support_for_all_asset_types` — constructs `CommandServer` with its file-asset loader, proves the pinned fixture invokes the loader only with supported image/font/audio kinds, and verifies the queued typed catalog. Script assets use the separate frozen trust/script pipeline in Rust. |
| `wait for server race condition` | `wait_for_server_race_condition` — interleaves 100 queued callbacks and draw keys against the blocking server loop, then drains before disconnect. |
| `stopMesssages command` | `stop_messages_command` — ports the `1`, `7`, `11` command-loop break boundaries exactly. |
| `draw happens once per poll` | `draw_happens_once_per_poll` + `draw_is_coalesced_by_key_within_one_poll` |
| `cancelDraw prevents pending draw from running` | `cancel_draw_only_cancels_matching_pending_key` |
| `cancelDraw only cancels the matching draw key` | `cancel_draw_only_cancels_matching_pending_key` |
| `disconnect` | `disconnect_stops_a_non_waiting_server` |
| `global asset set / remove` | `global_asset_set_and_remove` — preserves image/audio/font name registration, replacement, removal, and deletion cleanup. |
| `View Models` | `view_models` — covers named/artboard blank, default, named, nested, list, invalid, and independent dependent-handle lifetimes. |
| `View Model Listed Listener` | `view_model_listed_listener` — asserts the pinned six-name ordering and suppresses the callback for an invalid file. |
| `View Model Listener` | `view_model_listener` — asserts the pinned instance-name ordering and all ten typed property definitions, including enum and nested-model metadata. |
| `View Model Instance Listener` | `view_model_instance_listener` — preserves delete callbacks for all valid and invalid named/artboard instance handles. |
| `External Resources` | `external_resources` — covers external image/audio/font/blob identity and delete cleanup. |
| `RenderImage` | `render_image` — retains a successfully decoded handle, rejects invalid bytes, and removes both handles on delete. |
| `BlobAsset` | `blob_asset` — preserves raw and empty byte payloads behind typed handles, then removes both handles on delete. |
| `blob asset listener callbacks` | `blob_asset_listener_callbacks` — covers decoded, external, null-external error, and deleted listener messages with handle and request-id fidelity. |
| `AudioSource` | `audio_source` — retains a successful decode, rejects invalid bytes, removes both handles, and delivers the matching delete callback. |
| `Font` | `font` — retains a successful decode, rejects invalid bytes, removes both handles, and delivers the matching delete callback. |
| `View Model Property Set/Get` | `view_model_property_set_get` — compares every typed get callback's request/path/value in order, covers nested replacement and authored enum strings, proves decoded/external image and artboard retained identity, clearing, failed-set retention, invalid values/paths/handles, and deletion/error accounting. |
| `CommandServer::getHandleForInstance` | `command_server_get_handle_for_instance` — round-trips retained instance identity to its queue handle. |
| `Set Artboard Size / Reset Artboard Size` | `set_and_reset_artboard_size` — observes explicit size and scale changes on the server-owned artboard and restores the authored dimensions. |
| `Set Artboard Volume / Get Artboard Volume` | `set_and_get_artboard_volume` — observes queued server-side volume updates and the requested `0.75` callback with its request ID. |
| `View Model Property Subscriptions` | `view_model_property_subscriptions` — covers the nine typed subscriptions, changed-value and trigger delivery at the end-of-poll subscription pass, invalid path/type errors, and unsubscribe. |
| `View Model Blob Property Set` | `view_model_blob_property_set` — sets and clears retained blob bytes by property path, preserves shared identity, and reports invalid handle/path/view-model errors. |
| `View Model Blob Property Subscription` | `view_model_blob_property_subscription` — subscribes to the authored blob property, emits its typed handle on change, rejects a bad path, and stops after unsubscribe. |
| `View Model Property Async Subscriptions` | `view_model_property_async_subscriptions` — delivers a changed number subscription across the background server/message boundary and then unsubscribes. |
| `List View Model Property Set/Get` | `list_view_model_property_set_get` — checks exact appended/inserted/swapped handle identity and order around authored entries, exact sizes, unchanged state after invalid operations, and all invalid-handle/path/index errors. |
| `file Error Messages` | `file_error_messages` — checks the expected error counts for invalid file operations without producing success callbacks. |
| `listArtboard` | `list_artboard` — reports the pinned file's authored artboards in order and suppresses a success callback for an invalid file handle. |
| `listEnums` | `list_enums` — reports the authored enum name and ordered values and suppresses a success callback for an invalid file. |
| `requestViewModelInstanceViewModelName and requestViewModelInstanceName` | `request_view_model_and_instance_name` — reports both retained view-model and instance names and rejects an invalid handle. |
| `render image / audio source / font error` | `render_image_audio_source_font_error` — exercises invalid decode payload errors for all three resource kinds. |
| `state machine error` | `state_machine_error` — covers missing names and invalid dependent handles with the pinned error callback contract. |
| `artboard errors` | `artboard_errors` — covers missing artboard names and invalid file/artboard operations. |
| `Set Artboard Volume / Get Artboard Volume errors on invalid handles` | `invalid_artboard_volume_errors` — checks both set and request failures for an invalid artboard handle. |
| `Set Artboard Size / Reset Artboard Size errors on invalid handles` | `invalid_artboard_size_errors` — checks the ordered set and reset failures for an invalid artboard handle. |
| `listStateMachine` | `list_state_machine` — reports ordered state-machine names and suppresses a success callback for an invalid artboard handle. |
| `requestArtboardSize` | `request_artboard_size` — reports authored and updated dimensions with matching request IDs. |
| `requestDefaultViewModel` | `request_default_view_model` — reports the artboard's authored default view model and the absent-default case. |
| `bindViewModelInstance` | `bind_view_model_instance` — binds a retained instance to a state machine and rejects invalid handles. |
| `advanceStateMachine` | `advance_state_machine` — advances the live state machine to its settled callback and suppresses a settled callback for an invalid handle. |
| `listenerDeleteCallbacks` | `listener_delete_callbacks` — verifies delete callbacks for listener-owned file, artboard, state-machine, and image handles. |
| `fileLoadedCallback` | `file_loaded_callback` — delivers the file-loaded callback once with the allocated handle. |
| `artboardInstantiatedCallback` | `artboard_instantiated_callback` — delivers the instantiated callback once with the allocated artboard handle. |
| `stateMachineInstantiatedCallback` | `state_machine_instantiated_callback` — delivers the instantiated callback once with the allocated state-machine handle. |
| `viewModelInstanceInstantiatedCallback` | `view_model_instance_instantiated_callback` — delivers the instantiated callback once with the allocated view-model handle. |
| `decodedCallbacks` | `decoded_callbacks` — verifies successful image, audio, and font decode callbacks and their handles. |
| `listenerLifeTimes` | `listener_lifetimes` — proves queued listeners are retained until delivery and released after teardown. |
| `empty test for code cove` | `empty_listener_code_coverage` — sends file-load error and file-delete events through a no-op listener. |
| `pointer input` | `pointer_input` — ports pointer move/down/up/exit dispatch and the bound Boolean results from the pinned fixture. |
| `pointer down advances before rapid pointer up` | `pointer_down_advances_before_rapid_pointer_up` — proves the down transition advances before the immediately queued up event. |
| `pointer input translation` | `pointer_input_translation` — applies the pinned contain-fit/alignment transform before pointer dispatch. |
| `global Listener` | `global_listener` — routes file, artboard, state-machine, view-model, image, audio, font, and blob callbacks through global listeners. |
| `sync pointer events` | `sync_pointer_events` — interleaves 20 queued and synchronous move/down/up calls on the server owner thread and safely ignores calls after deletion. |
| `requestViewModelInstanceListClear` | `request_view_model_instance_list_clear` — clears a populated list property and reports the empty result. |
| `dependency lifetime management` | `dependency_lifetime_management` — deletes one artboard and one state machine while proving only their dependent handles are cleaned up and siblings remain live. |
| `file assets listed - image asset` | `file_assets_listed_image_asset` — checks every pinned image-asset field, including the concrete runtime extension and type ID. |
| `file assets listed - font asset` | `file_assets_listed_font_asset` — checks every pinned font-asset field, including the concrete runtime extension and type ID. |
| `file assets listed - type IDs match runtime` | `file_assets_listed_type_ids_match_runtime` — anchors image/font/audio IDs to schema keys 105/141/406. |
| `file assets listed - empty file` | `file_assets_listed_empty_file` — reports an empty catalog for the pinned no-assets file. |
| `file assets listed - invalid handle` | `file_assets_listed_invalid_handle` — suppresses the success callback and emits the matching file error. |
| `file assets listed - all assets returned` | `file_assets_listed_all_assets_returned` — compares the queued catalog count with the imported file's full asset catalog. |
| `Global View Model Names Listed` | `global_view_model_names_listed` — reports non-empty global model names and suppresses a success callback for an invalid file. |
| `Set/Bind/Get Global View Model Instance` | `set_bind_get_global_view_model_instance` — sets a global instance, binds the state machine, retrieves a handle with the expected model name, and errors for an invalid global name. |
| `Semantics advance does not auto-deliver diff` | `semantics_advance_does_not_auto_deliver_diff` — enables and advances the pinned `semantic/simpsons.riv` fixture without draining, then proves neither a diff nor an error is delivered. |
| `Semantics enable + initial diff on drainSemanticsDiff` | `semantics_enable_and_initial_diff_on_drain` — enables, settles, explicitly drains, replays the returned diff, and proves the initial tree is non-empty and contains the authored tab list. |
| `Semantics no diff when not enabled` | `semantics_no_diff_when_not_enabled` — settles the fixture without enabling or draining and proves no semantic diff/error delivery. |
| `Semantics drainSemanticsDiff errors when not enabled` | `semantics_drain_diff_errors_when_not_enabled` — checks the single state-machine error carries request id `0x1234` and that no diff is emitted. |
| `Semantics drainSemanticsDiff only emits for non-empty diff` | `semantics_drain_diff_only_emits_for_non_empty_diff` — checks one callback and request id for the initial drain, then proves a second unchanged drain emits nothing. |
| `Semantics fireSemanticAction tap changes selected tab` | `semantics_fire_tap_changes_selected_tab` — finds the selected and first unselected authored tabs, fires `Tap`, settles/drains, and proves the selected bits swap with no error. |
| `Semantics commands on invalid state machine handle` | `semantics_commands_on_invalid_state_machine_handle` — routes all five semantic commands to a failed named-machine handle and checks the exact ordered request ids `0xE1..=0xE5`, with no diff. |
| `Semantics drainSemanticsDiff maps bounds into view space` | `semantics_drain_diff_maps_bounds_into_view_space` — drains independent 200×200 and 800×800 views, matches a non-empty tab by id, and preserves the pinned 1% relative scale checks on width and height. |
| `Semantics requestSemanticFocus errors when not enabled` | `semantics_request_focus_errors_when_not_enabled` — checks exactly one error with request id `0x5151` and no diff. |
| `Semantics fireSemanticAction errors when not enabled` | `semantics_fire_action_errors_when_not_enabled` — checks exactly one error with request id `0x5252` and no diff. |
| `Semantics requestSemanticFocus on a valid node routes without error` | `semantics_request_focus_on_valid_node_routes_without_error` — drains a real node id, requests focus, settles/drains again, and proves the enabled route never reports an error whether or not that node accepts focus. |
| `Semantics clearSemanticFocus removes Focused bit from focused node` | `semantics_clear_focus_removes_focused_bit` — uses pinned `semantic_list_scroll_focus_fixed.riv`, focuses a `Focusable` node through the queue, observes `Focused`, clears through the queue, and observes the bit removed without error. |
| `Semantics drainSemanticsDiff honors scaleFactor when the view matches the artboard` | `semantics_drain_diff_honors_scale_factor_for_matching_view` — discovers the origin-based authored bounds, uses the matching view with `Fit::Layout`, and preserves the pinned 2% relative checks that scale factor 2 doubles every shared non-empty node's width and height. |

The focused Rust suite has additional supporting assertions for typed monotonic
handles, ordered `runOnce` callbacks, bounded command/message polling,
wait/wake, weak listeners, basic artboard lifetime, typed errors, and dependent
cleanup. Those are supporting assertions or partial upstream-case coverage,
not extra case-count credit.

## Pending non-F6 rows (0)

No current-pin non-F6 upstream case remains pending. This floor may only
tighten; newly discovered baseline gaps must be recorded before implementation.

## S4-45 blob disposition

All four cases are implemented as baseline command-runtime behavior. Upstream
commit `3c77a64d01c2afd2a50e47a324ee77972be5b370` added blob handles, queue/server
transport, listener messages, and view-model blob set/subscription behavior to
the C++ runtime itself. The Rust port retains each C++ `rcp<BlobAsset>` as an
`Arc<RuntimeBlobAsset>` and uses `CommandValue::Blob` for its typed handle;
neither adaptation introduces editor, Flow, or Apple lifecycle policy.

## Pending F6 dependency rows (0)

PR #216's F6 semantic runtime and PR #218's mounted-focus correction unblock
all 13 former dependency rows. The focused semantic queue run is green at
13/13; the pending floor tightens from 13 to 0 and may not be raised.

Ratchet accounting: **83 complete + 0 pending non-F6 + 0 pending F6 +
0 WATCH = 83 expected upstream cases**.
