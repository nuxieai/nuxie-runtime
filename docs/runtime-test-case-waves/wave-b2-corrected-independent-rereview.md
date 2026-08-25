# Wave B2 corrected candidate independent rereview

Original candidate: `bbf1ce429e87deafb6cfb89610d29ddf2b66039f`

Semantic correction: `997e8fa25a78d9f1c4a68daaaf06449f6112272d`

Prior rejection receipt: `943755fa4778267ab22b39b1f26c4b7cd12ad142`

Pinned upstream: `4ac7b32798da0482e441ef09304dc3b480ed3ee5`

Verdict: **REJECTED — 44/45 accepted, one direct passing row remains
semantically narrower than pinned C++**

## Acceptance rule

Each row was reread from the pinned C++ case and its executable Rust evidence.
A passing row was accepted only when the Rust body recreated the fixture,
owner actions and their order, and assertion semantics. An expected-red row
had to execute all prerequisites available before the concrete missing or
divergent boundary. Mechanical identity, discovery, and a passing test did not
substitute for body-level semantic correspondence.

## Exact census

| upstream file | cases | accepted pass | accepted executable expected-red | incomplete or narrower |
|---|---:|---:|---:|---:|
| `decode_ktx2_test.cpp` | 11 | 0 | 11 | 0 |
| `default_state_machine_test.cpp` | 1 | 1 | 0 | 0 |
| `distance_constraint_test.cpp` | 1 | 1 | 0 | 0 |
| `draw_order_test.cpp` | 1 | 1 | 0 | 0 |
| `elastic_easing_test.cpp` | 2 | 2 | 0 | 0 |
| `enums_test.cpp` | 17 | 17 | 0 | 0 |
| `file_test.cpp` | 12 | 8 | 3 | 1 |
| **total** | **45** | **30** | **14** | **1** |

The 25 rows accepted by the prior review remain accepted. Of its 20 rejected
rows, 19 are now accepted and one remains incomplete. The declared ledger
still reports 31 pass and 14 expected-red; semantic review demotes one of the
declared passes. The accepted proof-mechanism census is 27 direct and 17
adapted rows.

## Corrected-target adjudication

### The 17 enum rows are corrected

The Rust fixture reproduces `std::mt19937_64(0xf934929)` rather than the
rejected xorshift substitute. An independently compiled pinned-C++ oracle
matched all five committed prefix values and the committed 1,000-value FNV
fingerprint `0xbd69e87b0d1876de`. It also confirmed the exact underlying
types on the pinned target: signed 32-bit `Flags` and unsigned 64-bit
`Flags64`.

The typed `Flag<U>` route and primitive-integral route are separate executable
implementations, so the comparisons are no longer `value == value`
tautologies. The ports retain the exact five unary basics, eleven binary
basics, 1,000 unary or binary samples per helper invocation, 100 samples per
flag bit, 32/64 bit branches, per-type generator reset, return-type distinction,
and the pinned `Flags64` unmasked-any branch's accidental `decr` call. All 17
tests pass.

### Catch `Approx` semantics are corrected

`elastic_easing_test.cpp#2` keeps exact equality for the two actual-amplitude
checks. Its remaining three checks use Catch's pinned default epsilon of
`100 * float epsilon`, zero scale, zero fixed margin, and expected-value
magnitude scaling. The finite values under test make the Rust absolute-delta
form equivalent to Catch's overflow-safe two-sided margin comparison. The
focused internal owner test passes.

### Live Artboard `graphOrder` is corrected

`file_test.cpp#6` now instantiates the retained Artboard, reads `graph_order`
from live `RuntimeComponent` owners, asserts Artboard local 0 is order 0, and
performs all five pinned relative-order comparisons before the zero-time
update and exact world-translation assertions. It no longer substitutes the
static graph's optional order field.

### `markPathDirty` is still incomplete

`file_test.cpp#9` does enumerate every retained `PointsPath`, performs the
first update, schedules each path in source order, and performs the second
update. It does not, however, execute the `PointsPath::markPathDirty` owner
flow claimed by the ledger. It calls generic
`ArtboardInstance::add_dirt(path, PATH, false)` directly.

Pinned `src/shapes/points_path.cpp:43-49` first calls
`skin()->addDirt(ComponentDirt::Skin)` when a path has a skin, then calls
`Path::markPathDirty`; pinned `src/shapes/path.cpp:412-420` dirties the path
and notifies its containing shape. An independent pinned C++ fixture probe
found 77 `PointsPath`s in `bad_skin.riv`, including seven attached skins and
the one deliberately orphaned skin. The Rust evidence supplies only the path
dirt for all 77. It omits the seven Skin dirt actions and their required
skin-before-path ordering, and its post-action assertion observes only PATH
dirt. A green second update therefore does not prove the exact upstream action
stream.

This row needs to invoke a retained Rust `PointsPath::markPathDirty` owner that
includes the conditional Skin and containing-Shape side effects, or explicitly
reproduce and assert those side effects in the same order. Generic path dirt
alone is not exact correspondence.

## Prior accepted rows and red-boundary rereview

The unchanged 25-row accepted set was reread rather than inherited by count:
the eleven KTX2 cases preserve their per-case header/index/payload setup before
the absent decoder boundary; the default-state-machine, distance-constraint,
draw-order, elastic-load, and seven unaffected File passing cases preserve
their fixtures, owner actions, and assertions; and the strip-assets,
signed-script, and deterministic-mode rows still reach their documented
concrete boundaries.

All 14 expected-red rows were forced individually. Every invocation selected
exactly one test and failed: eleven at the absent KTX2/BC7 decoder owner, one
after importing and validating the Jellyfish/image fixture at the absent
`File::stripAssets` owner, one after importing real ScriptAssets at the absent
retained `verified()` state, and the Silver row at frame 0, operation 25's
signed-zero transform difference.

## Mechanical and execution gates

- strict pinned identity, source line/name, typed evidence locator,
  executable-test, ignore-reason, classification, and declared-census
  validation: 45/45 green (`28 direct / 17 adapted`, `31 pass / 14
  expected-red`);
- all 31 declared passing entry points executed successfully, including all
  17 enum targets, the eight tools-feature `cpp_probe` targets, the five
  `nuxie` targets, and the internal elastic target;
- all 14 expected-red targets forced individually with exactly one selected
  and one concrete failure;
- repository correspondence checker: 157 files and 1,404 pinned `TEST_CASE`s,
  green; the independent main case ledger remains unchanged and pending;
- correspondence checker unit suite: 24/24 green;
- scoped candidate `git diff --check`: green;
- rereview made no production or test correction.

Mechanical success does not promote `file_test.cpp#9`. Correct that owner flow
and submit Wave B2 to another fresh independent semantic review.
