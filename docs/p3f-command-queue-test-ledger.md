# P3F CommandQueue / CommandServer test ledger

Pinned oracle: `rive-runtime@4ac7b32798da0482e441ef09304dc3b480ed3ee5`,
`tests/unit_tests/runtime/command_queue_test.cpp` (83 `TEST_CASE`s).

The lane-focused command is:

```text
cargo test -p nuxie --test command_queue --no-fail-fast
```

## Complete upstream case ports (24)

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

The focused Rust suite has additional supporting assertions for typed monotonic
handles, ordered `runOnce` callbacks, bounded command/message polling,
wait/wake, weak listeners, basic artboard lifetime, typed errors, and dependent
cleanup. Those are supporting assertions or partial upstream-case coverage,
not extra case-count credit.

## Pending non-F6 rows (42)

These rows are not promoted: their fixture- or API-specific assertions have
not yet been executed on this branch. They are baseline follow-up work, not F6
exclusions.

1. `Set Artboard Size / Reset Artboard Size`
2. `Set Artboard Volume / Get Artboard Volume`
3. `View Model Property Subscriptions`
4. `View Model Property Async Subscriptions`
5. `List View Model Property Set/Get`
6. `file Error Messages`
7. `listArtboard`
8. `listEnums`
9. `requestViewModelInstanceViewModelName and requestViewModelInstanceName`
10. `render image / audio source / font error`
11. `state machine error`
12. `artboard errors`
13. `Set Artboard Volume / Get Artboard Volume errors on invalid handles`
14. `Set Artboard Size / Reset Artboard Size errors on invalid handles`
15. `listStateMachine`
16. `requestArtboardSize`
17. `requestDefaultViewModel`
18. `bindViewModelInstance`
19. `advanceStateMachine`
20. `listenerDeleteCallbacks`
21. `fileLoadedCallback`
22. `artboardInstantiatedCallback`
23. `stateMachineInstantiatedCallback`
24. `viewModelInstanceInstantiatedCallback`
25. `decodedCallbacks`
26. `listenerLifeTimes`
27. `empty test for code cove`
28. `pointer input`
29. `pointer down advances before rapid pointer up`
30. `pointer input translation`
31. `global Listener`
32. `sync pointer events`
33. `requestViewModelInstanceListClear`
34. `dependency lifetime management`
35. `file assets listed - image asset`
36. `file assets listed - font asset`
37. `file assets listed - type IDs match runtime`
38. `file assets listed - empty file`
39. `file assets listed - invalid handle`
40. `file assets listed - all assets returned`
41. `Global View Model Names Listed`
42. `Set/Bind/Get Global View Model Instance`

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

Ratchet accounting: **24 complete + 42 pending non-F6 + 13 pending F6 +
4 S4-45 WATCH = 83 expected upstream cases**.
