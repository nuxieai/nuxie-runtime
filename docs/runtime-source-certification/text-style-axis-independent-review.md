# Independent review: `TextStyleAxis`

Candidate `d1ed4103bdfbddffd2ab3f40291d905b3ee4cc58` is **rejected pending one
accounting-only correction** under
`docs/runtime-exact-parity-workflow-correction.md`.

The complete pinned authority at
`4ac7b32798da0482e441ef09304dc3b480ed3ee5` is confirmed: the cpp is 29
lines / 687 bytes / SHA-256
`0462a2fb81a963115c3d973d4b3a794a5d8683927f6566d2173f4a3417bf97bd`,
the hpp is 15 lines / 379 bytes / SHA-256
`57a07fb76437e0db73029891d885b48c9ab00dcb2fc533397e57d0c9730ee06a`,
and the executable denominator is exactly three cpp bodies and zero primary-
header bodies.

The semantic translation is sound. Construction links Component Super first,
continues only for `Ok`, requires a direct is-a `TextStyle` parent, and appends
the Axis exactly once to fresh occurrence-owned state in authored order. Live
parent-id writes leave the source registration frozen; cold clone state is
rebuilt from copied properties and re-registers A-to-B, while invalid copied
parentage fails reconstruction. Text and TextInput production shaping consume
the occurrence list; the graph scan is only the non-occurrence bootstrap. Both
callbacks observe generated equal-value and write-first ordering, add only
non-recursive `TextShape` dirt to the direct Style, and are excluded from the
generic notification tail. The downstream `TextStyle::markShapeDirty` cascade
and `TextVariationHelper` dependency/clone topology remain explicitly red and
are not credited here. No pinned test body names this owner, so literal consumer
topology remains **0 pass / 0 red / 0 adapted / 0 pending**.

## Blocking accounting finding

The incidental asset-impact paragraph counts a comment-only asset basename as
a real source reference. The corrected inventory has **131 referenced assets
plus 2 unreferenced assets** (the 133-asset / 1,039-axis inventory itself is
unchanged), and **279 real literal references**: 273 unique fixture-to-case
edges, 4 helper/global references, and 2 duplicate literals. Those edges cover
**271**, not 272, unique `TEST_CASE`s. Their outcome topology is **133 pass / 61
red / 3 adapted / 52 pending / 22 unmapped**, and their evidence topology is
**207 accepted / 42 provisional / 22 untouched**. The candidate's 296
references, 278 fixture-to-case pairs, 18 helper/global sites, 272 cases, 134
passes, and 208 accepted cases are therefore false positives.

Narrow correction: remove the comment-only basename from the reference join
and replace only the incidental-impact counts and prose with the corrected
figures above. Do not change production, tests, the three-body dispositions,
the zero-consumer topology, or either explicit downstream red.

## Focused gates and containment

- Both exact Axis owner tests passed, one test each.
- The focused Variation owner test passed, one test.
- `cargo check -p nuxie-runtime --no-default-features --features tools`
  passed.
- Candidate-range `git diff --check` passed and its six declared paths are
  contained.
- All 17 pre-existing user-dirty paths remained unstaged and outside this
  receipt.
