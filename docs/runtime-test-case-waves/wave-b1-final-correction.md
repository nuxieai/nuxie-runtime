# Wave B1 final correction candidate

Pinned upstream: `4ac7b32798da0482e441ef09304dc3b480ed3ee5`

Prior independent rereview: `543ba313d`

Status: **CORRECTED CANDIDATE — PENDING FRESH INDEPENDENT REVIEW**

This correction addresses all 13 rows rejected by the prior rereview. It does
not self-accept Wave B1.

## Corrected census

| upstream file | cases | pass | executable expected-red |
|---|---:|---:|---:|
| `data_binding_converters_test.cpp` | 3 | 1 | 2 |
| `data_binding_cycle_test.cpp` | 7 | 6 | 1 |
| `data_binding_fonts_test.cpp` | 2 | 1 | 1 |
| `data_binding_images_test.cpp` | 10 | 6 | 4 |
| `data_binding_keyframes.cpp` | 5 | 4 | 1 |
| `data_binding_test.cpp` | 40 | 30 | 10 |
| `data_binding_viewmodels_test.cpp` | 3 | 1 | 2 |
| **total** | **70** | **49** | **21** |

All 70 rows remain direct executable correspondence evidence. The 57 rows
accepted by the prior rereview were preserved.

## Corrections

### Exact Catch `Approx` arithmetic

The three shared Rust helpers now widen both `float` operands to `f64` before
calculating the difference and scaled `float` epsilon margin, matching Catch's
`double` arithmetic. This repairs the oracle used by the seven rejected rows:

- `data_binding_test.cpp#1` and `#7`;
- `data_binding_images_test.cpp#9`; and
- `data_binding_keyframes.cpp#2-5`.

Each helper also contains the rereview's discriminating counterexample:
`expected = f32::from_bits(0x0072abfc)` and `actual` 90 representable `f32`
values above it. Every corrected helper rejects that pair.

### Exact evidence and owner flows

- Font case #2 is classified expected-red and observes the decoded font retained
  by the backing `FontAsset`, rather than the input byte `Arc`.
- Image case #1 points to the executable root/nested live `Image` owner and
  stable selected `ImageAsset` identity evidence.
- Image case #2 points to the live image-property null action and stops at that
  action's concrete rejection before the pinned two-frame draw flow.
- Image case #5 selects `Fit::none` by its pinned value `5`, preserves the
  fit/alignment setters and assertions, and executes all 20 frames in each of
  the three phases before reaching the concrete post-swap transform divergence.
- Data-binding case #14 no longer substitutes typed child handles for pinned
  nullable empty list items. It reproduces the complete preceding scalar flow,
  verifies the retained list starts empty, and stops at the first absent null
  `ViewModelInstanceListItem` construction seam.
- Data-binding case #18 now observes the live instantiated `MainCircle` and
  `CustomPropertyTrigger` owners, performs all three pinned owner assertions,
  and replays the initial draw plus six `0.16` advances against the frozen C++
  SRIV stream.

## Gates

- pinned upstream HEAD: exact;
- B1 identity, upstream, evidence-line, symbol, outcome, ignore, and reason
  validation: 70/70;
- declared execution census: 49 pass / 21 expected-red;
- all 49 passing rows executed successfully;
- all 21 expected-red rows were forced individually, each selecting one ignored
  test and failing inside its named executable body at its real runtime seam;
- repository correspondence checker: 157 files and 1,404 pinned `TEST_CASE`s,
  green;
- correspondence checker unit suite: 24/24 green;
- non-test LLVM IR excludes the B1 private image-owner test module and symbol;
- JSON parsing and scoped `git diff --check`: green.

No production runtime behavior was changed. The next action is a fresh
independent semantic review of all 70 rows, with particular attention to these
13 corrected cases.
