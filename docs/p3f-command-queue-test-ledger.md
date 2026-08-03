# P3F CommandQueue / CommandServer test ledger

Pinned oracle: `rive-runtime@4ac7b32798da0482e441ef09304dc3b480ed3ee5`,
`tests/unit_tests/runtime/command_queue_test.cpp` (83 `TEST_CASE`s).

The lane-focused command is:

```text
cargo test -p nuxie --test command_queue --no-fail-fast
```

## Complete upstream case ports (66)

| Upstream `TEST_CASE` | Rust evidence |
|---|---|
| `POD Stream RCP` | `pod_stream_rcp` — preserves non-null/null optional ownership and shared identity through the queued callback seam. |
| `artboard management` | `artboard_management` — uses pinned `two_artboards.riv`, covers named/missing instances, invalid deletion, and file-owned cleanup. |
| `state machine management` | `state_machine_management` — uses pinned `multiple_state_machines.riv`, covers named/missing instances and dependent cleanup. |
| `default artboard & state machine` | `default_artboard_and_state_machine` — uses pinned `entry.riv` and proves default helpers equal empty names, including authored names. |
| `invalid handles` | `invalid_handles` — ports good/bad file, artboard, and state-machine handle creation, invalid deletion, non-destructive errors, and valid cleanup. |
| `draw loops` | `draw_loops` — independently schedules both keys, each key alone, idle polls, and teardown-safe one-shot draw callbacks. |
| `test support for all asset types` | `test_support_for_all_asset_types` — loads pinned `data_bind_test_cmdq.riv` and verifies the imported typed asset list through the queue callback. |
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
| `External Resources` | `external_resources` — covers external image/audio/font identity and delete cleanup; the S4-45 blob assertions remain on WATCH. |
| `RenderImage` | `render_image` — covers decode, retained dimensions, and delete lifecycle. |
| `AudioSource` | `audio_source` — covers decode identity and delete lifecycle. |
| `Font` | `font` — covers decode face identity and delete lifecycle. |
| `View Model Property Set/Get` | `view_model_property_set_get` — ports typed ordered set/get, nested replacement, authored enum strings, retained image/artboard identity and clearing, invalid values/paths/handles, and deletion callback/error accounting. |
| `CommandServer::getHandleForInstance` | `command_server_get_handle_for_instance` — round-trips retained instance identity to its queue handle. |
| `Set Artboard Size / Reset Artboard Size` | `set_and_reset_artboard_size` — sets an explicit size, reports it through the queue, and restores the authored dimensions. |
| `Set Artboard Volume / Get Artboard Volume` | `set_and_get_artboard_volume` — preserves the pinned default and queued volume updates. |
| `View Model Property Subscriptions` | `view_model_property_subscriptions` — covers initial delivery, changed-value delivery, no duplicate for an unchanged value, trigger delivery, and unsubscribe. |
| `View Model Property Async Subscriptions` | `view_model_property_async_subscriptions` — preserves ordered asynchronous subscription delivery across command/message polls. |
| `List View Model Property Set/Get` | `list_view_model_property_set_get` — gets the authored list, replaces it through queued handles, clears it, and rejects the wrong property type. |
| `file Error Messages` | `file_error_messages` — checks invalid file operations and their request IDs without producing success callbacks. |
| `listArtboard` | `list_artboard` — reports the pinned file's authored artboards in order and errors for an invalid file handle. |
| `listEnums` | `list_enums` — reports authored enum names and ordered keys/values, with invalid-file error coverage. |
| `requestViewModelInstanceViewModelName and requestViewModelInstanceName` | `request_view_model_and_instance_name` — reports both retained view-model and instance names and rejects an invalid handle. |
| `render image / audio source / font error` | `render_image_audio_source_font_error` — exercises invalid decode payload errors for all three resource kinds. |
| `state machine error` | `state_machine_error` — covers missing names and invalid dependent handles with the pinned error callback contract. |
| `artboard errors` | `artboard_errors` — covers missing artboard names and invalid file/artboard operations. |
| `Set Artboard Volume / Get Artboard Volume errors on invalid handles` | `invalid_artboard_volume_errors` — checks both set and request failures for an invalid artboard handle. |
| `Set Artboard Size / Reset Artboard Size errors on invalid handles` | `invalid_artboard_size_errors` — checks set, reset, and request failures for an invalid artboard handle. |
| `listStateMachine` | `list_state_machine` — reports ordered state-machine names and errors for an invalid artboard handle. |
| `requestArtboardSize` | `request_artboard_size` — reports authored and updated dimensions with matching request IDs. |
| `requestDefaultViewModel` | `request_default_view_model` — reports the artboard's authored default view model and the absent-default case. |
| `bindViewModelInstance` | `bind_view_model_instance` — binds a retained instance to a state machine and rejects invalid handles. |
| `advanceStateMachine` | `advance_state_machine` — advances the live state machine through the queued command and covers invalid-handle errors. |
| `listenerDeleteCallbacks` | `listener_delete_callbacks` — verifies delete callbacks for listener-owned file, artboard, state-machine, and view-model handles. |
| `fileLoadedCallback` | `file_loaded_callback` — delivers the file-loaded callback once with the allocated handle. |
| `artboardInstantiatedCallback` | `artboard_instantiated_callback` — delivers the instantiated callback once with the allocated artboard handle. |
| `stateMachineInstantiatedCallback` | `state_machine_instantiated_callback` — delivers the instantiated callback once with the allocated state-machine handle. |
| `viewModelInstanceInstantiatedCallback` | `view_model_instance_instantiated_callback` — delivers the instantiated callback once with the allocated view-model handle. |
| `decodedCallbacks` | `decoded_callbacks` — verifies successful image, audio, and font decode callbacks and their handles. |
| `listenerLifeTimes` | `listener_lifetimes` — proves queued listeners are retained until delivery and released after teardown. |
| `empty test for code cove` | `empty_listener_code_coverage` — exercises default no-op listener methods across the complete event surface. |
| `pointer input` | `pointer_input` — ports pointer move/down/up/exit dispatch and the bound Boolean results from the pinned fixture. |
| `pointer down advances before rapid pointer up` | `pointer_down_advances_before_rapid_pointer_up` — proves the down transition advances before the immediately queued up event. |
| `pointer input translation` | `pointer_input_translation` — applies the pinned contain-fit/alignment transform before pointer dispatch. |
| `global Listener` | `global_listener` — routes file, artboard, state-machine, view-model, image, audio, and font callbacks through global listeners; S4-45 blob messages remain WATCH. |
| `sync pointer events` | `sync_pointer_events` — exercises synchronous move/down/up/exit calls and their queue-visible state on the server owner thread. |
| `requestViewModelInstanceListClear` | `request_view_model_instance_list_clear` — clears a populated list property and reports the empty result. |
| `dependency lifetime management` | `dependency_lifetime_management` — deletes file, artboard, and state-machine owners while proving dependent handle cleanup and callbacks. |
| `file assets listed - image asset` | `file_assets_listed_image_asset` — checks every pinned image-asset field, including the concrete runtime extension and type ID. |
| `file assets listed - font asset` | `file_assets_listed_font_asset` — checks every pinned font-asset field, including the concrete runtime extension and type ID. |
| `file assets listed - type IDs match runtime` | `file_assets_listed_type_ids_match_runtime` — anchors image/font/audio IDs to schema keys 105/141/406. |
| `file assets listed - empty file` | `file_assets_listed_empty_file` — reports an empty catalog for the pinned no-assets file. |
| `file assets listed - invalid handle` | `file_assets_listed_invalid_handle` — suppresses the success callback and emits the matching file error. |
| `file assets listed - all assets returned` | `file_assets_listed_all_assets_returned` — compares the queued catalog count with the imported file's full asset catalog. |
| `Global View Model Names Listed` | `global_view_model_names_listed` — reports the pinned global model names in order and errors for an invalid file. |
| `Set/Bind/Get Global View Model Instance` | `set_bind_get_global_view_model_instance` — sets a global instance, binds the state machine, retrieves the retained instance, and covers invalid handles. |

The focused Rust suite has additional supporting assertions for typed monotonic
handles, ordered `runOnce` callbacks, bounded command/message polling,
wait/wake, weak listeners, basic artboard lifetime, typed errors, and dependent
cleanup. Those are supporting assertions or partial upstream-case coverage,
not extra case-count credit.

## Pending non-F6 rows (0)

No current-pin non-F6 upstream case remains pending. This floor may only
tighten; newly discovered baseline gaps must be recorded before implementation.

## S4-45 WATCH rows (4)

These current-pin cases remain WATCH rather than pending baseline or F6 work;
they require an explicit command-protocol/version decision for S4-45 blob
handles and messages.

1. `BlobAsset`
2. `blob asset listener callbacks`
3. `View Model Blob Property Set`
4. `View Model Blob Property Subscription`

## Pending F6 dependency rows (13)

Each row is blocked specifically on **F6 semantic-manager/action/focus/diff
semantics** and remains outside this baseline lane.

1. `Semantics advance does not auto-deliver diff` — depends on F6 diff ownership.
2. `Semantics enable + initial diff on drainSemanticsDiff` — depends on F6 enable/drain protocol.
3. `Semantics no diff when not enabled` — depends on F6 enable state.
4. `Semantics drainSemanticsDiff errors when not enabled` — depends on F6 error contract.
5. `Semantics drainSemanticsDiff only emits for non-empty diff` — depends on F6 diff generation.
6. `Semantics fireSemanticAction tap changes selected tab` — depends on F6 action routing.
7. `Semantics commands on invalid state machine handle` — depends on F6 command surface.
8. `Semantics drainSemanticsDiff maps bounds into view space` — depends on F6 geometry diff mapping.
9. `Semantics requestSemanticFocus errors when not enabled` — depends on F6 focus state.
10. `Semantics fireSemanticAction errors when not enabled` — depends on F6 action state.
11. `Semantics requestSemanticFocus on a valid node routes without error` — depends on F6 focus routing.
12. `Semantics clearSemanticFocus removes Focused bit from focused node` — depends on F6 focus mutation.
13. `Semantics drainSemanticsDiff honors scaleFactor when the view is scaled` — depends on F6 view-space diff mapping.

Ratchet accounting: **66 complete + 0 pending non-F6 + 13 pending F6 +
4 S4-45 WATCH = 83 expected upstream cases**.
