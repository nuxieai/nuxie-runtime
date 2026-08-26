# Wave C12 scripted-property Silver independent review

Reviewed candidate: `d3c1305892976cb914c0cbf7870152afb9daa7aa`

Pinned upstream: `4ac7b32798da0482e441ef09304dc3b480ed3ee5`

Verdict: **REJECTED — 0/8 semantically accepted**

## Acceptance rule

Each port must execute the pinned fixture and complete per-case bind, advance,
pointer, trigger, frame, and draw sequence through the corresponding live Rust
owners before comparing the fresh serialized render stream with the exact
pinned `.sriv`. A comparator failure is not sufficient when the replay
harness silently omits the scripted owner that performs the behavior under
test.

## Mechanical findings

The candidate correctly preserves the eight upstream identities, fixtures,
baseline names, frame times, action order, and evidence locators for
`scripting_properties_test.cpp` ordinals 9-16. In particular, case 16 contains
all six draws, five `addFrame` boundaries, the initial zero advance followed by
five `0.016` advances, both `tri1` trigger writes, and both pointer down/up
pairs at `(45, 165)`.

The eight ignored tests also reach `compare_sriv` individually and fail with
the recorded first differences:

| ordinal | baseline | forced first difference |
|---:|---|---|
| 9 | `viewmodel_access` | frame 0, op 32: expected `transform`, got `save` |
| 10 | `viewmodel_from_instance` | frame 0, op 8: expected `makeRenderPaint`, got `frameSize` |
| 11 | `replace_view_model` | frame 0, op 42 `transform.tx`: expected `0`, got `250` |
| 12 | `remove_from_list` | frame 0, op 165: expected `save`, got `restore` |
| 13 | `list_index_script_access` | frame 0, op 80 `addRawPath`: expected 33 fields, got 808 |
| 14 | `scripted_property_image` | frame 0, op 18: expected `save`, got `restore` |
| 15 | `image_scripting_property_value` | frame 0, op 23 `transform.tx`: expected `-702`, got `-139` |
| 16 | `reset_shared_viewmodel_instance_test` | frame 0, op 10: expected `makeRenderPaint`, got `frameSize` |

These facts validate the candidate's metadata and comparator reporting, but
not its semantic owner fidelity.

## Blocking semantic defect

All eight pinned `.riv` fixtures contain a `ScriptedDrawable` on the selected
artboard; `list_index_script_access.riv` contains another one on its child
artboard. Those live scripted drawables are the owners that read or mutate the
view-model properties exercised by these tests.

The shared replay path used by every candidate test does not mount or realize
them. `tools/silver-corpus/src/scripting.rs` explicitly documents that
"scripted drawables are not realized." `Execution::run` registers the File VM
and initializes state-machine scripted objects, but never invokes the
ScriptedDrawable mount/realize flow. By contrast, the repository's full
`rust-golden-runner` calls `initialize_scripted_drawables_and_realize` (or the
prepare/realize variant) and binds their live data context before advancing
and drawing.

The candidate therefore executes eight literal action lists against artboards
whose central scripted behavior has been left inert. The resulting SRIV
differences are real comparisons, but they are caused by a replay-harness
omission before the translated behavior executes. They cannot certify the
runtime divergences named by the rows, and the common helper invalidates all
eight cases rather than only one.

## Required correction

Route these cases through a replay owner that mounts and realizes the fixture's
actual ScriptedDrawable instances, binds their live view-model context, and
then executes each existing literal action sequence. If the full owner cannot
yet execute a fixture, classify that case as pending/unverified or make it an
expected red at the concrete mount/realize failure; do not silently continue
with an inert ScriptedDrawable and treat the later SRIV mismatch as parity
evidence.

## Gates

- pinned upstream checkout: exact;
- candidate diff: test/docs only; no production behavior changes;
- focused non-incremental target: 0 passed / 0 failed / 8 ignored;
- forced expected-red sweep: 8/8 selected individually, all eight failed at
  their recorded live SRIV comparison;
- repository correspondence checker: 157 files / 1,404 cases, green;
- correspondence-checker unit suite: 24/24 green;
- ledger JSON parse, candidate diff check, fixture identity, and baseline
  existence checks: green.

The mechanical gates do not override the shared owner-bypass defect.
