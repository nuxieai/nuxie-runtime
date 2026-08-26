# Wave C12 scripted-property Silver corrected independent rereview

Reviewed correction: `257a7b17f2bba812a30de2b625ccf4b119ba3535`

Prior rejection: `cf7ae7e72`

Pinned upstream: `4ac7b32798da0482e441ef09304dc3b480ed3ee5`

Verdict: **REJECTED — owner correction accepted; six expected-red reason
mismatches remain**

## Exact census

| classification | cases |
|---|---:|
| formally accepted pass | 1 |
| semantically accepted realized-owner reds, metadata-blocked | 6 |
| honest pending at the genuine owner seam | 1 |
| total | 8 |

The correction fully resolves the prior inert-ScriptedDrawable defect. Formal
fragment acceptance remains blocked only by the six ledger/ignore reason
mismatches described below.

## Realized-owner correction

Every executable pass/red case now uses `nuxie::ArtboardInstance`, imports the
exact authenticated fixture, initializes the renderer and File VM, binds the
exact pinned ViewModel instance, mounts the concrete ScriptedDrawable tree,
and directly checks each selected-artboard ScriptedDrawable global with
`has_script_instance_for_global` before executing any behavioral action.

The production mount owner collects concrete root and nested targets, prepares
their VM tables with the retained root ViewModel, validates topology, attaches
the tables, recollects the tree, and errors when any target remains unattached.
The test also compares the retained root context's instance identity with the
exact bound `ViewModelInstance`. No graph count or metadata assertion stands
in for ScriptInstance attachment.

The correction is isolated to the integration test, its ledger and receipt,
and a `nuxie` dev-dependency plus lockfile edge. It changes no production
source or behavior. The prior `silver_corpus::Execution` bypass is no longer
used by this target.

## Per-case semantic result

- Case 9 executes the exact fixture, index-zero ViewModel bind, `0.016`
  state-machine advance, draw, and `viewmodel_access.sriv` comparison through
  the realized ScriptedDrawable owner. It passes exactly.
- Case 10 realizes its selected ScriptedDrawable with the authored default
  ViewModel, then fails at frame 0 operation 8: pinned expects
  `makeRenderPaint`, Rust emits `frameSize`.
- Case 11 preserves the two `0.016` advances, first draw, frame boundary,
  pointer down/up at `(artboard width / 2, 480)`, and second draw. After genuine
  realization it fails at frame 1 operation 93: expected `color`, got `save`.
- Case 12 preserves the initial `0.1` advance/draw and exactly ten
  frame/`0.016`/draw iterations. It fails at frame 1 operation 195: expected
  `rewind`, got `drawPath`.
- Case 13 enters the genuine mount owner and fails closed before any behavior
  at `occurrence 1 depth 1 graph 84 has no retained source File`. The ledger
  correctly removes executable evidence and classifies this case
  pending/unverified. No inert-replay comparison remains.
- Case 14 realizes its selected ScriptedDrawable, advances `0.016`, draws, and
  fails at frame 0 operation 21: expected `save`, got `restore`.
- Case 15 preserves the zero advance/draw, frame boundary, `0.25` advance, and
  second draw. It fails at frame 0 operation 1: expected `decodeImage`, got
  `makeRenderPaint`.
- Case 16 preserves all six draws, five frame boundaries, timings
  `0, 0.016, 0.016, 0.016, 0.016, 0.016`, both `tri1` trigger writes, and both
  pointer down/up pairs at `(45, 165)`. It fails at frame 0 operation 10:
  expected `makeRenderPaint`, got `frameSize`.

All fixtures and `.sriv` names match the pinned upstream cases.

## Blocking metadata defect

The six expected-red ledger reasons do not equal their discovered Rust
`#[ignore]` reasons after removing the `expected-red: ` prefix. This exact
reason contract is already enforced by the campaign's Wave C1 review and is
not satisfied by semantically equivalent prose.

| case | ignore reason form | ledger reason form |
|---:|---|---|
| 10 | `viewmodel_from_instance frame 0 op 8 ...` | `viewmodel_from_instance reaches the exact SRIV comparator; at frame 0 operation 8 ...` |
| 11 | `realized replace_view_model frame 1 op 93 ...` | `After the selected ScriptedDrawable is genuinely realized ..., replace_view_model reaches ... frame 1 operation 93 ...` |
| 12 | `realized remove_from_list frame 1 op 195 ...` | `After the selected ScriptedDrawable is genuinely realized ..., remove_from_list reaches ... frame 1 operation 195 ...` |
| 14 | `realized scripted_property_image frame 0 op 21 ...` | `After the selected ScriptedDrawable is genuinely realized ..., scripted_property_image reaches ... frame 0 operation 21 ...` |
| 15 | `realized image_scripting_property_value frame 0 op 1 ...` | `After the selected ScriptedDrawable is genuinely realized ..., image_scripting_property_value reaches ... frame 0 operation 1 ...` |
| 16 | `reset_shared_viewmodel_instance_test frame 0 op 10 ...` | `After the selected ScriptedDrawable is genuinely realized ..., reset_shared_viewmodel_instance_test reaches ... frame 0 operation 10 ...` |

The operation names, frame/operation indices, expected values, and actual
values are correct in both places. Correction only requires choosing one exact
wording per case and using it in both the ledger and `#[ignore]` attribute.

## Gates

- focused non-incremental target: 1 passed / 0 failed / 7 ignored;
- six expected reds forced individually: all six failed at their documented
  realized-owner SRIV difference;
- pending case 13 forced individually: failed at the documented nested source
  File-authority mount seam before actions or comparison;
- all seven executable evidence path/line/symbol locators resolve to distinct
  discovered Rust tests; case 13 correctly has no evidence locator;
- repository correspondence checker: 157 files / 1,404 cases, green;
- correspondence-checker unit suite: 24/24 green;
- pinned fixture/baseline identities, JSON parsing, correction diff check, and
  production-source freeze: green.

Once the six exact reason pairs match, no semantic rework or further reopening
of the realized-owner evidence is required.
