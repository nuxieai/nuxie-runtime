# MERGEFIX report: comparator effects + VFIX2

## Outcome

Merged `origin/main` into `levi/vfix-comparator-effects` and composed the branch's V11/V33 comparator settlement with main's VFIX2 nested-VMI binding and publish-back behavior.

The corpus found no runtime entry where the two policies genuinely conflict. Main's authored-arena policy is therefore retained at the collision: host-first synchronization applies the authored value even when the retained cell still reports the preceding write as changed. The branch's stale-mirror skip guard was removed because VFIX2 publishes mounted-child mutations back to the authored arena immediately after each occurrence advance, and the full corpus passed without the guard.

## Runtime resolution

- Preserved the branch's persistent comparator resolution path and call sites:
  - `resolve_value_and_cell_for_source_path_with_persistent_resolver`
  - `resolved_value_and_cell_for_property_path`
  - `root_handle_for_view_model_index`
  - the retained-file-aware binding calls in `context_value.rs` and `state_machine_instance.rs`
- Preserved `enqueue_artboard_shared_converter_direction` for the branch's shared-converter call sites, while retaining main's broader `enqueue_artboard_authored_data_bind_direction` for VFIX2's explicit rebind reconciliation.
- Adopted main's child-first `bind_owned_view_model_occurrence_data_context` ordering and immediate `publish_nested_view_model_context_mutations` call after nested occurrence advance.
- Kept the branch's authored-dirty tracking and mounted-layout ownership handling used by V11 settlement.
- Removed only the stale-cell early-`continue` guard that could reject a coherent authored-arena value after VFIX2 publish-back.

This composition follows the pinned C++ reference at `/Users/levi/dev/oss/rive-runtime` commit `4ac7b327`:

- `src/animation/transition_viewmodel_condition.cpp:49-60`
- `src/nested_artboard.cpp:228-350`

## Parity register resolution

`docs/parity-gap-register.md` was resolved per V-row by retaining the more progressed result.

- Branch progress retained for V11, V22, V25, V27, V30, V33, V35, and V36.
- Main progress retained for V12, V13, V20, V21, V23, V24, V26, V28, V29, V31, V32, V34, V37, and V38.
- V38 remains CLOSED/exact.
- Main's W3 semantic-UAF watch row was retained and separated from the preceding prose line.

## Verification

The required gates were run in order against the final resolved tree:

1. `cargo build -p nuxie-runtime` — PASS
2. `cargo test -p nuxie-runtime` — PASS; zero failures
3. `cargo test -p nuxie --features scripting` — PASS; zero failures (one fixture-generator test intentionally ignored)
4. `make scripted-golden-compare` — PASS

Final corpus summary:

```text
entries=363 exact=341 exact-segments=1111 side-channel-segments=1106 diverges=17 unsupported-feature=0 not-yet=5
```

Acid-test results:

| Row / entry | Result |
| --- | --- |
| V11 `global_variables_test` | Registered divergence reproduced; comparator side-channel fix remains, with the documented mounted-layout draw remainder |
| V33 `stateful_keyed_trigger` | exact |
| V25 `group_effect` | Registered divergence reproduced with the progressed invalidation diagnosis |
| V30 `path_effect_with_feathers` | Registered divergence reproduced with the progressed feather diagnosis |
| V38 `viewmodel_instance_to_artboard` | exact |
| `bidirectional_stateful_property` | exact |

There were zero corpus-wide regressions and no entry requiring restoration of the stale-mirror guard.
