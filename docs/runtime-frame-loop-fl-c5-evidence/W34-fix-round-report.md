# W34 FL-C5 family-review fix round

Base candidate: `dc96e571`

Pinned C++ runtime: `d788e8ec6e8b598526607d6a1e8818e8b637b60c`

This report records the uncommitted W34 repair delta. No commit was created.
The two family reviews that caused the round remain preserved beside this
report as `W32-oracle-review.md` and `W33-standards-review.md`.

## Oracle findings

| Finding | Repair | Pinned source | Strengthened proof |
| --- | --- | --- | --- |
| O1 — script preparation suppressed ordinary public behavior | Removed the blanket preparation gates from pointer/input/listener and raw/facade advance paths. The public borrowed and owned facade constructors synchronously prepare scripted data whenever the normal runtime-file resolver is available. When it is genuinely unavailable, only the individual unmounted scripted callback is skipped; ordinary actions and bookkeeping continue. | `state_machine_instance.cpp:2072-2127`, especially constructor completion before return and `.cpp:2546-2668` / `.cpp:3173-3187` for ordinary advance and pointer work. | `public_machine_construction_synchronously_prepares_scripted_data_without_blocking_ordinary_input` constructs through the public facade and compares immediate `pointer_down` plus `advance_and_apply` to the live scripted C++ probe. `matched_pointer_listener_marks_advance_even_when_actions_are_noops` retains an unavailable scripted object and proves the ordinary listener return and raw advance still run. The former manual-init masking helper was replaced with public construction. |
| O2 — typed named-input lookup absent | Added type-and-name authored-order lookup and public `get_bool`, `get_number`, and `get_trigger`, while retaining the existing name-only `input_named` convenience semantics. | `state_machine_instance.cpp:2689-2714`. | `typed_named_inputs_match_type_and_name_in_authored_order` and live `fl_c5_typed_named_inputs_match_cpp_with_an_earlier_wrong_type` author Number `"x"` before Bool `"x"` and require typed Bool lookup to select the later Bool. |
| O3 — `fireSemanticAction` was a no-op | Added the exact manager node-ID → SemanticData identity lookup followed by the tap/increase/decrease switch (`0/1/2`; other values no-op), recorded both lookup boundaries at the declared seams, and dispatched the selected action through the retained SemanticData/listener path. Manager node IDs are kept distinct from authored component local IDs. | `state_machine_instance.cpp:2509-2544`; the manager/data internals remain recorded at B6-0329/B6-0327. | `semantic_callbacks_apply_constraints_preserve_duplicates_and_defer_actions` proves manager node ID `1` resolves SemanticData local ID `2`, rejects local ID `2` as a node ID, and observes the selected callback. Focus/semantic tests cover all three valid actions, invalid action, missing manager/node/data, constraints, duplicates, and queue timing. The ratchet now requires the node-ID lookup, data projection, switch, and call; its injected negative conflates the node ID with the target. |
| O4 — audio-event selection absent | Added exact `AudioEvent` type recognition on reported events and invokes the recorded playback seam once per AudioEvent after local dispatch and bubbling. Ordinary Events never reach it. Playback internals remain recorded at B6-0113. | `state_machine_instance.cpp:3155-3169`. | `fl_c5_event_bubbling_precedes_the_recorded_audio_seam_through_two_ancestors` sends one ordinary Event and one AudioEvent through two bubble owners, checks authored bubbling order, and requires only the AudioEvent receipt after each bubble step. The ratchet and negative now distinguish typed filtering from batch-wide invocation. |

## Standards findings

| Finding | Repair | Proof |
| --- | --- | --- |
| S1 — duplicated artboard orchestration | Deleted `ArtboardInstance::advance_state_machine_instances_with_nested_context` and the artboard-owned zero-time follow-up helper. The public artboard wrappers now provide only the borrow-model nested-artboard closure to `StateMachineInstance::advance_artboard_frame_components_with`, which owns root advance, nested event delivery, and follow-up policy. | Existing nested-event differentials plus the strengthened artboard-ownership ratchet. Its injected negative restores the exact displaced orchestration function shape and must fail. |
| S2 — stale mechanical correspondence | `state_machine.cpp` now maps to `state_machine/state_machine.rs`; `state_machine_instance.cpp` includes the genuine `state_machine/state_machine_instance.rs` owner alongside the retained focused-input/private-layer modules. The legacy `instance.rs` owner mapping was removed. The ownership ledger and correspondence manifest agree, and both rows remain `pending` as required. | `docs/runtime-frame-loop-ownership.toml` and `file-correspondence-manifest.toml`; the full checker validates their consistency. |
| S3 — public API inventory did not prove signatures | Replaced reachability-only coverage with exact typed function-pointer bindings for every W4 §C inventory item, including generic hydration methods. The downstream inventory now contains 328 exact coercions covering the complete pointer/context/script-host families, scripted converter families, bindable queries/setters, default/imported/owned ViewModel families, and the newly named typed lookup APIs. | Downstream `fl_c5_public_reexports_are_downstream_visible_after_file_split` compiles only if receiver, arguments, return type, ownership, and visibility remain exact. |
| S4 — vacuous ratchets | Thin-entry rules reject arbitrary functions—including `const`, `unsafe`, and `extern` forms—plus impls, structs, enums, unions, and traits. The public inventory rules require the real coercion macro body, exactly 328 typed invocations, and the SHA-256 digest of the complete exhaustive signature block, not selected names. The artboard rule catches the S1 function shape. Semantic and audio rules require their new behavior. | `make runtime-frame-loop-port-test`: 56/56. Injected negatives cover ordinary and qualified displaced helpers, a disabled coercion macro, a count-preserving still-compiling substitution of one exact signature with a duplicate, restored artboard orchestration, conflated semantic node lookup, and untyped audio dispatch. |
| S5 — five-pass DELETE proof was source-only | Added a live persistent-dirt C++ probe action and matching Rust probe. Each runtime executes `advanceAndApply` against dirt that survives all five update passes. | `fl_c5_five_pass_unconditional_probe` compares 6 total advance calls, 5 update calls, and dirt still present across both runtimes. |
| S6 clarification — history-range finding rejected in substance | The closure lists the interleaved, separately audited FL-C1 commits and their accepted evidence, and states that the FL-C5 package delta itself neither took ownership of nor promoted an FL-C1 row. | `docs/runtime-frame-loop-fl-c5-closure.md`, opening clarification. |
| S7 — evidence packet not tracked/auditable | Copied W29/W30/W31, all seven floor logs, and both review verdicts into `docs/runtime-frame-loop-fl-c5-evidence/`. Removed stale “current HEAD” language and cited tracked paths. This W34 report is part of the same packet. | The final trace fingerprint covers the tracked evidence directory, and `runtime-frame-loop-port-check` rejects stale tracked or untracked candidate sources. |

## Closure and proof-gap repairs

- Added the missing private `unbind` member row with
  `state_machine_instance.cpp:2949-2953`, bind/lifecycle proof keys, ordering,
  and teardown adversarial cases.
- Restored the full same-byte C++/Rust definition fixture and all of its
  name, count, authored input/layer order, duplicate-first-match, and
  case-sensitivity comparisons. Added a separate safe C++ definition/importer
  seam for an authored null input slot, duplicate layer names, retained
  listener indices, and duplicate DataBind property keys. That focused seam
  intentionally does not enter C++ `onAddedDirty`, whose null-slot
  dereference is malformed.
- Kept the combined all-collection duplicate/null fixture unchecked: executing
  that malformed shape would enter pinned C++ null dereferences rather than
  create a safe behavioral oracle.
- Kept the full one-table bind null matrix unchecked. Existing distinct live
  branches and the inherited A→B registration hazard are cited, but
  unconstructed combinations are not claimed.
- Expanded clone isolation across reporting/current/bubble queues,
  listener-report queues, pointer state, hit owners, listener groups, nested
  registrations, detached primary-context state, callback dirt sinks, layer
  identities, and cold script tables.
- Added the exact adversarial in-place hit sort case; it fails under a stable
  sort or a scan that stops at the first duplicate.
- Added an end-to-end keyframe initialize → converter → enrollment live
  comparison: the initialized converted value is observed, the live source is
  dirtied, and both runtimes must emit the second converted value.

The closure leaves every remaining proof gap explicitly unchecked with its
reason; no source-cited or unsafe case is relabeled as a live differential.

## Why the changed tests are stronger

No assertion, fixture dimension, or oracle comparison was removed to make a
failure pass.

- The O1 fixture no longer manually marks initialization complete; it uses the
  public constructor and a live scripted C++ oracle.
- Pointer-return comparison now settles both C++ and Rust geometry with the
  same first update, compares that update report, then compares the exact C++
  `HitResult != none` projection.
- The pre-existing same-byte definition differential and every one of its
  comparisons remain intact. The additional null-hole proof uses the actual
  `StateMachineImporter::readNullObject` definition seam instead of an unsafe
  full C++ artboard lifecycle, compares the exact Rust/C++ input and layer
  sequences, and checks both duplicate bind keys.
- O3 changed the old expected no-op into an observable valid-action callback
  and added invalid/missing lookup controls.
- O4 changed a one-event phase-marker check into ordinary-plus-audio typed
  selection through bubbling.
- S3 changed reachability-only compilation into exact signature compilation.
- S4 negative controls now include qualified displaced functions and a
  count-preserving, still-compiling API-signature substitution that only the
  exhaustive block digest catches.
- S5 changed C++ source-string inspection into a live two-runtime comparison.

## Acceptance receipts

| Gate | Receipt |
| --- | --- |
| Runtime library | `cargo test -p nuxie-runtime --lib`: 715 passed. |
| Live C++ differential suite | `RIVE_CPP_PROBE=... cargo test -p nuxie-runtime --test cpp_probe`: 806 passed. |
| Exact downstream FL-C5 signatures | `cargo test -p nuxie-runtime --test public_api_fl_c5`: 1 passed. |
| Public facade library | `cargo test -p nuxie --lib`: 146 passed. |
| C API | `cargo test -p nux-capi`: 3 library + 16 integration tests passed; doc tests green. |
| Ordinary golden | `make golden-compare`: 317/317 entries, 647 exact segments, zero divergences/unsupported/not-yet. |
| Scripted golden | Fresh Rust scripted runner against the existing pinned C++ scripted runner: 317/317 entries, 647 exact segments, zero divergences/unsupported/not-yet. The wrapper’s attempted external rebuild was denied by the workspace sandbox before comparison; the direct compare is the same command with the already-built pinned runner. |
| Structural/injected negatives | `make runtime-frame-loop-port-test`: 56/56. |
| Public integration test | The current sandbox run passed all 14 code/API cases; only `public_api_exposes_the_default_rust_renderer` could not construct a Metal adapter (`metal found no adapters`). The tracked `floor-public-api.log` records the unchanged renderer path green at 15/15. No test was skipped or weakened. |
| Final provenance/checker | Regenerated after this report; `make runtime-frame-loop-port-check` is the authoritative final receipt. |
| Formatting/worktree | `cargo fmt --all -- --check` and `git diff --check` are the authoritative final receipts. |
