# Wave B2 final independent semantic review

Reviewed candidate commits:

- `bbf1ce429e87deafb6cfb89610d29ddf2b66039f`
- `997e8fa25a78d9f1c4a68daaaf06449f6112272d`
- `8c3d9c963`

Prior rejection receipts: `943755fa4` and `4b42896ca`

Pinned upstream: `4ac7b32798da0482e441ef09304dc3b480ed3ee5`

Verdict: **ACCEPTED — all 45 Wave B2 rows preserve the pinned semantic
question.**

The campaign-wide consolidated locator gate remains red because of seven
stale Wave B3 SRIV locators. That external mechanical defect is not a Wave B2
semantic or shard defect and does not demote this 45-row acceptance.

## Acceptance rule

Every row was reread from pinned C++ and its executable Rust body. A passing
row was accepted only when it preserved the fixture, setup, owner actions and
their order, and assertion semantics. An expected-red row had to execute every
available prerequisite and stop at its concrete absent or divergent runtime
seam. Mechanical discovery or a nearby proxy did not substitute for this
body-level review.

## Exact accepted census

| upstream file | cases | accepted pass | accepted executable expected-red |
|---|---:|---:|---:|
| `decode_ktx2_test.cpp` | 11 | 0 | 11 |
| `default_state_machine_test.cpp` | 1 | 1 | 0 |
| `distance_constraint_test.cpp` | 1 | 1 | 0 |
| `draw_order_test.cpp` | 1 | 1 | 0 |
| `elastic_easing_test.cpp` | 2 | 2 | 0 |
| `enums_test.cpp` | 17 | 17 | 0 |
| `file_test.cpp` | 12 | 9 | 3 |
| **total** | **45** | **31** | **14** |

Proof mechanisms remain 28 direct rows and 17 explicitly adjudicated
`cxx-language-only` adaptations. There are no pending, source-anchor-only,
inert, incomplete, or semantically narrower rows.

## Hostile rereview of corrected owners

### MT19937 and typed enum semantics

All 17 enum adaptations use the exact `std::mt19937_64(0xf934929)` stream. The
committed five-value prefix and 1,000-value fingerprint match the independently
compiled pinned-C++ oracle. `Flags` retains signed 32-bit behavior and
`Flags64` retains unsigned 64-bit behavior.

The typed `Flag<U>` operations and primitive-integral oracle are separate
executions, rather than the rejected raw-value tautologies. The ports preserve
the five unary basics, eleven binary basics, 1,000 random helper samples,
per-type generator reset, 100 samples per flag bit, 32/64-bit branches,
enum-versus-scalar return distinction, and the pinned `Flags64` unmasked-any
branch's accidental `decr` operation.

### Catch Approx

The elastic numeric owner keeps exact equality for the two actual-amplitude
assertions. The remaining three comparisons use pinned Catch's default
`100 * float epsilon`, zero scale, zero fixed margin, and expected-value
magnitude scaling. For these finite values the Rust absolute-delta expression
is equivalent to Catch's overflow-safe two-sided margin comparison.

### Live graphOrder

The dependency case instantiates the live Artboard and reads graph order from
retained `RuntimeComponent` owners. It asserts Artboard local zero has order
zero, preserves all five relative comparisons, then performs the zero-time
update and exact world-translation assertions.

### PointsPath, Skin, then Path

The final `bad_skin.riv` evidence executes in the concrete PointsPath owner
module. It proves the pinned fixture census of 77 PointsPaths, eight Skins,
seven bidirectional attachments, and one orphan Skin.

For each PointsPath in source order it dirties the retained Skin first when
present, then invokes the concrete Path dirt owner. Skin dirt dispatches
reentrantly to the attached PointsPath; Path dirt dispatch reaches the
containing Shape/PathComposer invalidation. The test asserts the Skin-before-
Path call order, all seven Skin calls, all 77 Path calls, PATH dirt on every
live path, and a successful second Artboard update. It no longer substitutes
generic path-only dirt for `PointsPath::markPathDirty`.

## Other rows

- All eleven KTX2 ports build their complete pinned per-case header, level
  index, and payload stream before reaching the absent production KTX2/BC7
  decoder owner.
- Default-state-machine, distance-constraint, draw-order, elastic-load, and
  the unaffected File rows preserve their exact fixtures, action order, owner
  observations, and assertions.
- Strip-assets imports and validates the Jellyfish/image owners before the
  absent `File::stripAssets` seam. Signed-script imports real ScriptAssets
  before the absent retained `verified()` state. Deterministic-mode performs
  the complete pinned action stream and reaches its exact frame 0, operation
  25 signed-zero transform difference.

## Execution and shard gates

- All 31 declared passing rows executed successfully: 17 enum targets, seven
  tools-feature `cpp_probe` targets, three `nuxie` File targets, distance and
  draw-order integrations, the internal elastic owner, and the PointsPath
  owner test.
- All 14 expected-red rows were forced individually. Each invocation selected
  exactly one test and failed at its declared concrete seam.
- Strict Wave B2 pinned identity, exact source name/line, typed evidence
  locator, executable-test, ignore-reason, classification, and census
  validation is 45/45 green: `28 direct / 17 adapted`, `31 pass / 14
  expected-red`.
- Repository correspondence is green for 157 files and 1,404 pinned
  `TEST_CASE`s. Its checker unit suite is 24/24 green.
- A fresh non-test `nuxie-runtime` LLVM-IR emission contains neither cfg(test)
  relationship accessor, the PointsPath fixture test, nor its trace type.
- Review and validation made no production or candidate correction.

## External consolidated-locator blocker

The current frozen locator audit is:

- Wave A: 258/258 clean;
- Wave B1: 70/70 clean;
- Wave B2: 45/45 clean;
- Wave B3: 79/86 clean, seven stale; and
- consolidated: 452/459 clean.

The seven stale entries are the SRIV symbols in
`tools/silver-corpus/tests/wave_b3.rs`. The Wave B3 ledger records lines
`43/49/55/60/65/71/76`, while the committed symbols are at
`39/43/47/50/53/57/60`. The ledger was refreshed against unstaged rustfmt
collateral rather than the committed file layout.

This requires a separate Wave B3 locator-only correction before campaign-wide
closeout can claim 459/459. It does not change any Wave B2 identity, evidence
body, locator, outcome, or semantic verdict.
