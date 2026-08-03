# P3F CommandQueue / CommandServer test ledger

Advanced test oracle: `rive-runtime@4ac7b327`,
`tests/unit_tests/runtime/command_queue_test.cpp` (83 `TEST_CASE`s).

The lane-focused command is:

```text
cargo test -p nuxie --test command_queue --no-fail-fast
```

## Complete upstream case ports (4)

| Upstream `TEST_CASE` | Rust evidence |
|---|---|
| `draw happens once per poll` | `draw_happens_once_per_poll` + `draw_is_coalesced_by_key_within_one_poll` |
| `cancelDraw prevents pending draw from running` | `cancel_draw_only_cancels_matching_pending_key` |
| `cancelDraw only cancels the matching draw key` | `cancel_draw_only_cancels_matching_pending_key` |
| `disconnect` | `disconnect_stops_a_non_waiting_server` |

The 14-test Rust suite has additional focused assertions for typed monotonic
handles, ordered `runOnce` callbacks, bounded command/message polling,
wait/wake, weak listeners, basic artboard lifetime, typed errors, and dependent
cleanup. Those are supporting assertions or partial upstream-case coverage,
not extra case-count credit.

## Pending non-F6 rows (62)

These rows are not promoted: their fixture- or API-specific assertions have
not yet been executed on this branch. They are baseline follow-up work, not F6
exclusions.

1. `POD Stream RCP`
2. `artboard management`
3. `state machine management`
4. `default artboard & state machine`
5. `invalid handles`
6. `draw loops`
7. `test support for all asset types`
8. `wait for server race condition`
9. `stopMesssages command`
10. `global asset set / remove`
11. `View Models`
12. `View Model Listed Listener`
13. `View Model Listener`
14. `View Model Instance Listener`
15. `External Resources`
16. `RenderImage`
17. `AudioSource`
18. `Font`
19. `View Model Property Set/Get`
20. `CommandServer::getHandleForInstance`
21. `Set Artboard Size / Reset Artboard Size`
22. `Set Artboard Volume / Get Artboard Volume`
23. `View Model Property Subscriptions`
24. `View Model Property Async Subscriptions`
25. `List View Model Property Set/Get`
26. `file Error Messages`
27. `listArtboard`
28. `listEnums`
29. `requestViewModelInstanceViewModelName and requestViewModelInstanceName`
30. `render image / audio source / font error`
31. `state machine error`
32. `artboard errors`
33. `Set Artboard Volume / Get Artboard Volume errors on invalid handles`
34. `Set Artboard Size / Reset Artboard Size errors on invalid handles`
35. `listStateMachine`
36. `requestArtboardSize`
37. `requestDefaultViewModel`
38. `bindViewModelInstance`
39. `advanceStateMachine`
40. `listenerDeleteCallbacks`
41. `fileLoadedCallback`
42. `artboardInstantiatedCallback`
43. `stateMachineInstantiatedCallback`
44. `viewModelInstanceInstantiatedCallback`
45. `decodedCallbacks`
46. `listenerLifeTimes`
47. `empty test for code cove`
48. `pointer input`
49. `pointer down advances before rapid pointer up`
50. `pointer input translation`
51. `global Listener`
52. `sync pointer events`
53. `requestViewModelInstanceListClear`
54. `dependency lifetime management`
55. `file assets listed - image asset`
56. `file assets listed - font asset`
57. `file assets listed - type IDs match runtime`
58. `file assets listed - empty file`
59. `file assets listed - invalid handle`
60. `file assets listed - all assets returned`
61. `Global View Model Names Listed`
62. `Set/Bind/Get Global View Model Instance`

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

## Pending S4-45 blob WATCH rows (4)

The advanced pin adds four blob command-transport cases. P3F's structural
owners do not yet implement blob handles, callbacks, or view-model blob
properties, so these cases receive no completion credit.

1. `BlobAsset`
2. `blob asset listener callbacks`
3. `View Model Blob Property Set`
4. `View Model Blob Property Subscription`

Ratchet accounting: **4 complete + 62 pending non-F6 + 13 pending F6 +
4 pending S4-45 blob WATCH = 83 expected upstream cases**.
