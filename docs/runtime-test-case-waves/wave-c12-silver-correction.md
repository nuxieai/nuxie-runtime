# Wave C12 scripted-property Silver correction

Status: **CORRECTED CANDIDATE; PENDING INDEPENDENT REREVIEW**

Rejected candidate: `d3c1305892976cb914c0cbf7870152afb9daa7aa`

Independent rejection receipt: `cf7ae7e72`

Pinned upstream: `4ac7b32798da0482e441ef09304dc3b480ed3ee5`

## Correction

The eight exact tests no longer use `silver_corpus::Execution`, whose replay
registered File and StateMachine scripts but left every `ScriptedDrawable`
inert. The isolated test target now runs through `nuxie::ArtboardInstance`,
the repository facade that owns the genuine ScriptedDrawable mount lifecycle.

For each case the shared runner:

1. imports the exact pinned fixture with authenticated scripting;
2. initializes renderer resources and the fixture's File script VM;
3. selects state machine 0 and constructs the exact pinned ViewModel instance;
4. binds that same live ViewModel owner to the artboard;
5. calls `mount_scripted_drawables`, whose production path instantiates each
   script with the retained ViewModel context, attaches its table, re-collects
   concrete targets, and fails closed if any remains unattached;
6. checks the retained context's identity against the exact bound ViewModel
   and checks every selected-artboard ScriptedDrawable global directly with
   `has_script_instance_for_global` before the remaining behavioral actions;
7. executes the unmodified pinned advance, draw, frame, pointer, and trigger
   sequence and compares the fresh stream with the exact `.sriv` baseline.

This is direct live-owner evidence, not a drawable count, graph-metadata proxy,
test-local script, or inert replay.

## Corrected outcomes

| ordinal | exact locator | outcome | observed realized result |
|---:|---|---|---|
| 9 | `wave_c12_silver_009_access_view_model_properties_and_enum_properties` | pass | exact `viewmodel_access.sriv` match |
| 10 | `wave_c12_silver_010_creates_view_models_from_specified_named_instances` | expected red | frame 0 op 8: expected `makeRenderPaint`, got `frameSize` |
| 11 | `wave_c12_silver_011_replace_a_view_model_property_value_from_a_script` | expected red | frame 1 op 93: expected `color`, got `save` |
| 12 | `wave_c12_silver_012_scripts_can_remove_items_from_lists` | expected red | frame 1 op 195: expected `rewind`, got `drawPath` |
| 13 | `wave_c12_silver_013_expose_list_index_to_scripts_and_ensure_type_is_correct` | pending | genuine mount reaches child occurrence graph 84 without retained source File authority |
| 14 | `wave_c12_silver_014_scripted_image_properties` | expected red | frame 0 op 21: expected `save`, got `restore` |
| 15 | `wave_c12_silver_015_image_read_from_property_value` | expected red | frame 0 op 1: expected `decodeImage`, got `makeRenderPaint` |
| 16 | `wave_c12_silver_016_reset_detached_view_model_instances_at_end_of_frame` | expected red | frame 0 op 10: expected `makeRenderPaint`, got `frameSize` |

Case 16 still contains exactly six draws, five frame boundaries, timings
`0, 0.016, 0.016, 0.016, 0.016, 0.016`, two `tri1` trigger writes, and both
pointer down/up pairs at `(45,165)`.

## Census

- 8/8 pinned case identities and exact fixtures retained;
- 1 realized pass;
- 6 individually forceable realized-owner expected reds;
- 1 honest pending case at the genuine nested-owner mount seam;
- 0 inert-replay results retained;
- no production behavior changes.

## Gates

- focused non-incremental target: 1 pass / 0 failures / 7 ignored;
- six realized reds forced individually: all six fail at their recorded fresh
  realized-owner SRIV difference;
- pending #13 forced individually: fails at the recorded genuine nested-owner
  File-authority mount seam before comparison;
- explicit per-global realization/context assertions: present and exercised by
  every pass/red case;
- correspondence checker: 157 files / 1,404 pinned cases, green;
- correspondence checker unit suite: 24/24 green;
- scoped rustfmt and `git diff --check`: green;
- JSON census and seven executable evidence locators: green;
- all eight exact pinned fixtures and `.sriv` baselines: present at the pinned
  upstream SHA;
- non-test `silver-corpus` LLVM IR contains no Wave C12 Silver test/helper
  symbol.
