# Wave C9 state-machine independent adversarial review

Verdict: **REJECTED — five semantic rows use diff projections, one numeric
assertion is weakened, and ten callable Silver owners are misclassified as
pending**

Reviewed candidate: `326a1a10c982fdad57f10c640c32efa0ce7e0a80`

Pinned upstream: `4ac7b32798da0482e441ef09304dc3b480ed3ee5`

Frozen denominator: 46 cases: 19 `state_machine_test.cpp`, 11
`state_machine_event_test.cpp`, one `state_machine_input_test.cpp`, and 15
`semantic_state_machine_test.cpp`.

## Blocking findings

### Five semantic passes observe a projection instead of `nodeById`

Semantic cases 2–5 and 7 assert current state through the retained live
`SemanticManager::nodeById` owner in the pin. Their mapped Rust tests instead
drain a `SemanticsDiff` and reconstruct a test-local `BTreeMap` snapshot. The
diff is real runtime output, but it is not the causal owner asserted by these
five cases. This is the same missing owner that correctly leaves semantic
cases 13–15 pending; accessibility does not make the live state inapplicable.

Demote these five rows to strict pending/unverified with empty evidence, or
move each complete case into distinct owner-local evidence that reads the
retained manager nodes directly. Do not relabel diff reconstruction as a Rust
ownership adaptation.

### Event case 8 weakens Catch `Approx`

The pinned timeline-event case compares `secondsDelay()` with
`Approx(0.1f)`. Catch uses `100 * f32::EPSILON * abs(expected)` here, roughly
`1.19e-6`. The Rust test accepts an absolute error of `1e-5`, over eight times
wider. Replace it with the exact Catch comparison, including widening the
`f32` operands before subtraction as the repository's existing oracle does.

### Ten pending rows already have literal callable owners

The following rows cannot remain missing-owner pending. Their corresponding
`silver-corpus.toml` entries are `runtime-literal`, name the same pinned source
case, preserve the complete fixture/action/draw stream, and compare against
the pinned SRIV baseline:

- event case 10: `target_event`;
- state cases 9–11: `transition_index_condition`, `sorted_listeners`, and
  `multi_listeners`;
- state cases 13–14: `transition_duration_bind_nested` and
  `transition_duration_bind_list`;
- state cases 16–18: `component_based_conditions`,
  `component_based_conditions-Artboard2`, and `transition_actions`; and
- state case 19: `paused_nested_artboard_opacity`.

Add one distinct test locator per upstream row. The manifest records the first
nine as exact, but the independent current replay found `sorted_listeners`
red at `frame 5, op 134 (addRawPath): expected 180 fields, got 337`; classify
the forced outcome rather than trusting stale manifest status. State case 19
is also a genuine executable expected-red whose exact known boundary is
`frame 1, op 103 (rewind): expected rewind, got drawPath`. Keep each ignore
reason byte-identical to its independently forced failure. An aggregate loop
is not a locator, but that does not turn these callable literal owners into
missing-owner cases.

State cases 12 and 15 remain honest pending because their manifests require
the absent runtime-owned nested ViewModel replacement-by-handle surface.

## Audited remainder

The other 19 declared executable rows preserve their pinned fixture, action,
and assertion streams through retained imported definitions or live runtime
owners. In particular, event case 11 selects authored ViewModel index 0,
instance 0—the same occurrence selected by pinned
`createViewModelInstance(viewModelId, 0)`—rather than the unrelated default
context used by the legacy false-red test.

The remaining pending rows are honest: semantic manager pointer identity;
semantic live-node focus/current-bounds checks; nested child event/report
authority; the live Shape→Stroke→SolidColor chain; animation-reset resource
pool state; live layer-state identity; and the two nested ViewModel replacement
streams. Static graphs, diff reconstruction, downstream render results, and
unconditional missing-owner panics do not satisfy those observables.

If the five semantic rows are demoted rather than moved to exact owner-local
tests and the other manifest outcomes remain as recorded, the corrected
topology is **28 pass, two executable expected-red, and 16 pending**: 30
executable total, consisting of 20 direct and ten structured Rust-safety
adaptations.

## Gates

- Strict identity, ordinal, source-line, source-name, outcome, pending shape,
  and locator audit: mechanically green for the candidate's declared 46/46
  rows; 25 executable and 21 pending.
- All four pinned source SHA-256 values and the pinned checkout SHA match the
  candidate receipt.
- Focused non-incremental `upstream_wave_c9` execution: 14 passed, zero failed
  or ignored.
- Focused non-incremental `semantic_focus_runtime` execution: 15 passed, zero
  failed or ignored. Passing projections do not cure the owner mismatch.
- Non-incremental literal Silver checkpoint: three aggregate tests passed and
  the resolved group failed only at the concrete `sorted_listeners` boundary
  above. This independently confirms that the Silver rows are callable rather
  than missing-owner pending.
- The candidate changes only tests and correspondence documentation; no new
  production or test-only seam requires a containment rerun.

Every relied-on Cargo invocation disabled incremental compilation. This
receipt changes no candidate test, production source, ledger row, or fixture.
