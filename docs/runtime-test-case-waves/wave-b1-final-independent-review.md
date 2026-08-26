# Wave B1 final independent semantic review

Reviewed correction: `f36ac6facbc37f8191144a771293a80be9a0567f`

Prior rejection: `543ba313d`

Pinned upstream: `4ac7b32798da0482e441ef09304dc3b480ed3ee5`

Verdict: **REJECTED — 69/70 semantically accepted**

## Review rule

All 70 rows were checked against their pinned C++ fixture, action order, live
owner, and assertion semantics. An expected-red must execute the equivalent
flow to the first concrete missing or divergent owner seam. A named
unconditional panic does not prove a seam: it cannot distinguish the current
missing behavior from a future implementation and therefore can never turn
green when parity is restored.

## Exact census

| upstream file | cases | accepted pass | accepted expected-red | rejected |
|---|---:|---:|---:|---:|
| `data_binding_converters_test.cpp` | 3 | 1 | 2 | 0 |
| `data_binding_cycle_test.cpp` | 7 | 6 | 1 | 0 |
| `data_binding_fonts_test.cpp` | 2 | 1 | 1 | 0 |
| `data_binding_images_test.cpp` | 10 | 6 | 4 | 0 |
| `data_binding_keyframes.cpp` | 5 | 4 | 1 | 0 |
| `data_binding_test.cpp` | 40 | 30 | 9 | 1 |
| `data_binding_viewmodels_test.cpp` | 3 | 1 | 2 | 0 |
| **total** | **70** | **49** | **20** | **1** |

The committed shard mechanically declares 49 pass and 21 expected-red. One of
those 21 red rows is rejected as evidence, leaving 69 semantically accepted
rows.

## Remaining defect

`data_binding_test.cpp#14`, **Transition self conditions**, still does not
execute the first pinned list action at an actual owner seam.

The corrected test faithfully imports
`transition_self_comparator_test.riv`, binds the live owners, reproduces the
initial draw and full scalar mutation/draw prefix, and verifies that the
retained `lis` owner starts empty. At that point pinned C++ constructs a
`ViewModelInstanceListItem` with a null `viewModelInstance` and calls
`addItem`. Rust has no corresponding nullable-list-item construction API.

Instead of invoking an executable construction/add seam and failing from its
result, the evidence ends with:

```rust
panic!(
    "Rust list owner cannot construct the pinned null ViewModelInstanceListItem required by the first addItem action"
);
```

That is an unconditional failure after the setup. It proves neither that the
nullable owner is absent nor that the first `addItem` action rejects the
nullable item. More importantly, it remains red unchanged if production later
adds exact nullable-item support. The row therefore stops before, rather than
at, an executable concrete seam.

This row needs evidence at the narrowest actual list-owner boundary that
attempts the pinned nullable construction/add action and fails because that
owner behavior is absent. It must not substitute a typed child handle, but it
also cannot use an unconditional panic as the missing action.

## Accepted corrections from the prior rejection

The other 12 previously rejected rows are corrected.

### Catch `Approx`

All three shared helpers widen both `f32` operands to `f64` before subtraction
and compare against the default Catch float epsilon scaled in the widened
domain. This matches Catch's double arithmetic for the pinned `Approx(float)`
expressions in:

- `data_binding_test.cpp#1` and `#7`;
- `data_binding_images_test.cpp#9`; and
- `data_binding_keyframes.cpp#2-5`.

Each helper contains and passes the discriminating counterexample from the
prior rejection: `expected = f32::from_bits(0x0072abfc)` and `actual` 90
representable `f32` values above it. The corrected helper rejects that pair.

### Exact retained owners and action streams

- Font case #2 now observes the decoded bytes retained by the property's
  backing `RuntimeFontAssetOwners` entry. Its forced red fails when the owner
  still contains the prior decoded font, at the exact first replacement
  assertion, rather than comparing only the input byte `Arc`.
- Image case #1 resolves the live root `Image` and mounted nested `Image`,
  asserts their selected `ImageAsset` global identities, performs both pinned
  mutations, and proves both live owners switch to the new stable identities.
- Image case #2 performs the actual live image-property null assignment and
  fails because that assignment returns false, before the pinned two-frame
  draw continuation.
- Image case #5 uses the pinned `Fit::none` value `5`, preserves the generated
  fit/alignment setter/getter assertions, performs the image2 and image3 swaps,
  and contains all 20 frames in each of the three phases. Its forced red fails
  on the first pinned post-image2 negative local-translation assertion.
- Data-binding case #18 resolves live instantiated `MainCircle` and `Trig`
  occurrences, asserts Shape identity plus both scale values and
  `CustomPropertyTrigger` identity, then compares the complete initial draw and
  six `0.16` frame stream against the pinned C++ SRIV.

## Rechecked prior accepted rows

The other 57 rows retain the exact fixtures, action ordering, live owners,
assertion streams, and previously adjudicated concrete red seams. The
correction changes no production behavior. Outside the 13 prior rejects, its
only executable effect is the stricter shared Catch oracle, which preserves
the pinned arithmetic rather than narrowing an accepted owner flow.

## Validation

- pinned upstream HEAD is exactly
  `4ac7b32798da0482e441ef09304dc3b480ed3ee5`;
- all 70 rows resolve exact pinned identities, ordinals, source lines, names,
  evidence symbols, classifications, and ignore reasons;
- all 49 declared passing rows were selected individually and passed;
- all 21 declared expected-red rows were forced individually; each selected
  exactly one test and failed inside its named body;
- repository correspondence checker: 157 files and 1,404 pinned cases, green;
- correspondence checker unit suite: 24/24 green;
- non-test LLVM IR contains neither the private B1 image-owner module nor its
  test symbol;
- scoped JSON and diff checks are green.

Mechanical success cannot promote the unconditional Transition Self red.
Wave B1 remains rejected until that single evidence row reaches an executable
actual-owner seam.
