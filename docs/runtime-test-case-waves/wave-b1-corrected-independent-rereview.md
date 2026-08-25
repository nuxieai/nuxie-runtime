# Wave B1 corrected independent semantic rereview

Reviewed commits: `828e6158f`, `a94907b5e`, `17d95fc7e`, and locator refresh
`8c3d9c963`

Prior rejection receipt: `817a8c8b3`

Pinned upstream: `4ac7b32798da0482e441ef09304dc3b480ed3ee5`

Verdict: **REJECTED**

## Acceptance rule

All 70 rows were reread against the pinned C++ case and the committed Rust
evidence. A row was accepted only when the committed locator selected evidence
that preserved the fixture, action order, live production owner, and assertion
semantics, or stopped at the first concrete absent or divergent seam. A stale
classification, a graph or recording proxy for a live owner, a substituted
action, or a different numeric oracle did not count as exact correspondence.

## Exact census

| upstream file | cases | accepted pass | accepted executable expected-red | rejected |
|---|---:|---:|---:|---:|
| `data_binding_converters_test.cpp` | 3 | 1 | 2 | 0 |
| `data_binding_cycle_test.cpp` | 7 | 6 | 1 | 0 |
| `data_binding_fonts_test.cpp` | 2 | 1 | 0 | 1 |
| `data_binding_images_test.cpp` | 10 | 5 | 1 | 4 |
| `data_binding_keyframes.cpp` | 5 | 0 | 1 | 4 |
| `data_binding_test.cpp` | 40 | 27 | 9 | 4 |
| `data_binding_viewmodels_test.cpp` | 3 | 1 | 2 | 0 |
| **total** | **70** | **41** | **16** | **13** |

The accepted set is 57 rows: 41 passing ports and 16 executable expected-red
ports. The rejected set is 13 rows. The 54 rows accepted by the first review
remain accepted. Of the 16 corrected rows, three are now accepted and 13 still
fail exact evidence review.

## Rejected rows

### Seven helpers still do not implement Catch `Approx`

The correction replaces the fixed absolute tolerance, but the new helpers
still calculate subtraction and tolerance in `f32`:

```rust
(actual - expected).abs() <= f32::EPSILON * 100.0 * expected.abs()
```

Pinned Catch stores the `float` operands as `double` and performs the margin
comparison in `double`. The distinction is observable. For
`expected = f32::from_bits(0x0072abfc)` and an `actual` 90 representable `f32`
values above it, the Rust helper returns true while pinned Catch returns false.
The `f32` difference and threshold both round to `1.26e-43`; Catch compares the
exact widened difference `1.2611686178923354e-43` with threshold
`1.2553862250802864e-43` and rejects it.

The seven affected rows remain rejected:

- `data_binding_test.cpp#1`, **artboard with bound properties**;
- `data_binding_test.cpp#7`, **Range Mapper**;
- `data_binding_images_test.cpp#9`, the 7.2 layout-image scale case; and
- `data_binding_keyframes.cpp#2-5`.

The restored pinned constants and exact-vs-approximate assertion selection are
otherwise correct. The remaining defect is specifically the arithmetic domain
of the Catch oracle.

### Six committed evidence rows are stale or still prove the wrong owner

`wave-b1.json` still declares the pre-correction `51 pass / 19 expected-red`
census rather than the corrected `49 / 21` census.

- `data_binding_fonts_test.cpp#2` points to the corrected ignored backing-owner
  flow but still classifies it as passing and has no expected-red reason.
- `data_binding_images_test.cpp#1` still points to the public encoded-byte
  recording proxy rather than the new root/nested stable `ImageAsset` owner
  evidence.
- `data_binding_images_test.cpp#2` still points to the scalar/SRIV pass instead
  of the corrected executable live-image-null expected-red.
- `data_binding_images_test.cpp#5` still points to the old one-step SRIV proxy
  instead of the corrected 61-frame owner flow. The corrected flow also searches
  for `fit == 0` when asserting the no-scale image. Pinned `Fit::none` is `5`;
  `0` is `Fit::fill`, so both transform assertions still observe the wrong
  image owner.
- `data_binding_test.cpp#14` retains the old counted-list reason. The corrected
  test does execute insert/swap/remove calls, but constructs a typed child view
  model handle for each item. Pinned C++ inserts default
  `ViewModelInstanceListItem` owners whose `viewModelInstance` is null. That is
  a substituted action, not the exact empty-item stream, and the first missing
  nullable-item seam must remain explicit.
- `data_binding_test.cpp#18` still points only to the generic SRIV replay. The
  added supporting test reads the named trigger from the immutable imported
  graph and checks its schema type; upstream finds the live
  `CustomPropertyTrigger` occurrence on the instantiated artboard. Static graph
  membership can pass when the live owner is absent or mis-instantiated.

The decoded-font expected-red, exact stable image-owner test, live-image-null
flow, complete 61-frame flow, concrete list mutation API flow, and trigger
supporting test are useful corrections. They cannot promote their rows until
the committed evidence selects the right bodies and the remaining owner/action
substitutions above are removed or adjudicated at their first real seam.

## Accepted corrected rows

The following prior rejects now preserve the pinned operation and assertion
stream:

- `data_binding_test.cpp#2` mutates the enum through the pinned member name
  `state-blue` and then fires the trigger;
- `data_binding_test.cpp#23` restores `viewModelName`, typed property/cache
  behavior, nested lookup, enum name, and empty non-enum name; and
- `data_binding_test.cpp#31` asserts the retained source X/Y after settlement
  before asserting target X/Y, so the expected-red detects post-settle source
  clobber at the correct owner.

## Rechecked prior accepts

All 54 rows accepted by `817a8c8b3` were reread and re-executed. The correction
did not narrow their fixture, owner, action, or assertion streams. In
particular, the cycle occurrence/event ordering, shared-vs-distinct instance
identity, decoded dynamic/stateful image actions, target-to-source TwoWay
owner mutation, and accepted SRIV provenance remain intact.

## Mechanical, execution, provenance, and IR gates

- pinned upstream HEAD: exact at
  `4ac7b32798da0482e441ef09304dc3b480ed3ee5`;
- all 70 typed locator lines and symbols resolve uniquely after `8c3d9c963`;
- strict committed classification/reason validation: 68/70, failing font #2's
  pass-to-ignored mismatch and Transition Self's stale ignore reason;
- committed shard census: `70 direct`, `51 pass / 19 expected-red`; corrected
  executable census used for execution: `49 pass / 21 expected-red`;
- all 49 corrected passing rows were selected individually and passed; the
  separate Custom Trigger supporting assertion also passed;
- all 21 corrected expected-red rows were forced individually; every command
  selected exactly one ignored test and failed inside its named body;
- all 34 corrected-candidate SRIV row IDs resolve to the exact pinned upstream
  file and exact `TEST_CASE` provenance name;
- repository correspondence checker: 157 files and 1,404 pinned `TEST_CASE`s,
  green; the independent main case ledger remains all pending;
- correspondence checker unit suite: 24/24 green;
- non-test LLVM IR contains neither the B1 image-owner test module nor its test
  symbol;
- JSON parsing and scoped `git diff --check`: green.

Execution success does not promote the 13 rows whose committed oracle, owner,
action, or evidence classification remains inexact. Wave B1 therefore remains
rejected at 57/70.
