# Wave B1 independent semantic review

Reviewed commits: `828e6158f` and `a94907b5e`

Pinned upstream: `4ac7b32798da0482e441ef09304dc3b480ed3ee5`

Reviewer: independent `runtime_test_wave_b2` lane

Verdict: **REJECTED**

## Acceptance rule

All 70 rows were read against the pinned C++ case and the committed executable
Rust evidence. A row was accepted only when it preserved the fixture, action
order, production owner, and assertion semantics, or executed every available
prerequisite and stopped at the first concrete absent or divergent seam. A
render replay that omitted non-render assertions, an indirect facade/proxy
observation, a substituted action, or a different approximation rule did not
count as exact correspondence.

## Exact review census

| upstream file | cases | accepted pass | accepted executable expected-red | incomplete or narrower |
|---|---:|---:|---:|---:|
| `data_binding_converters_test.cpp` | 3 | 1 | 2 | 0 |
| `data_binding_cycle_test.cpp` | 7 | 6 | 1 | 0 |
| `data_binding_fonts_test.cpp` | 2 | 1 | 0 | 1 |
| `data_binding_images_test.cpp` | 10 | 5 | 1 | 4 |
| `data_binding_keyframes.cpp` | 5 | 0 | 1 | 4 |
| `data_binding_test.cpp` | 40 | 25 | 8 | 7 |
| `data_binding_viewmodels_test.cpp` | 3 | 1 | 2 | 0 |
| **total** | **70** | **39** | **15** | **16** |

The accepted set is 54 rows: 39 passing ports and 15 executable expected-red
ports. The rejected set is 12 declared passing rows and four declared
expected-red rows. All 70 rows are mechanically discoverable and executable;
the rejection is semantic, not a missing-entry or compilation failure.

## Blocking semantic findings

### Seven rows use different approximation semantics

The following declared-direct evidence replaces pinned Catch `Approx`
semantics with a fixed absolute `0.0001` comparison:

- `data_binding_test.cpp#1`, **artboard with bound properties**;
- `data_binding_test.cpp#7`, **Range Mapper**;
- `data_binding_images_test.cpp#9`, **Layout image composes user scale on top
  of fit for 7.2 files**; and
- `data_binding_keyframes.cpp#2-5`, all four deterministic keyframe behavior
  cases.

This is not a cosmetic difference. In the keyframe tests, for example, Catch's
margin scales with the distinctive `424242` sentinel, while the Rust helper
uses an absolute ten-thousandth. The standalone negative assertion therefore
accepts values that pinned Catch rejects. The Range Mapper port also changes
pinned exact integer assertions into approximate assertions. The bound-
properties port additionally compares against Rust's `PI` rather than the
pinned `3.14159f`. The image 7.2 red reaches a real scale divergence, but its
three preceding/final comparisons still do not preserve the pinned oracle.

### Font and image rows observe or mutate the wrong owner

`data_binding_fonts_test.cpp#2` asserts decoded `HBFont` identity on the
property's backing `FontAsset` after two assignments and a clear. The Rust
evidence stores `Arc<[u8]>` values and compares byte-allocation pointers on the
view-model property. It can pass without decoding the font or updating the
backing `FontAsset::font` owner.

`data_binding_images_test.cpp#1` asserts that the named root and nested
`Image` owners point to the exact selected file `ImageAsset` objects before and
after mutation. The Rust evidence searches a recording for matching encoded
asset bytes. Duplicate contents or an incorrectly wired asset object can pass
that proxy check.

`data_binding_images_test.cpp#2` calls
`ViewModelInstanceAssetImage::value(nullptr)`, which clears the live
`RenderImage` and applies the sentinel. The SRIV action instead writes the
generic file-asset scalar `u64::MAX`; it never executes the live-image clear
owner. Matching rendered output does not make the substituted action exact.

### Two SRIV replays omit pinned work

`data_binding_images_test.cpp#5` performs and restores generated fit,
alignment-X, and alignment-Y setters/getters, asserts the no-scale image's
negative transform after each asset swap, and runs 20 advance/draw iterations
for each of three phases. The manifest replay performs none of the generated
setter/getter or transform assertions and only one `0.016` advance/draw per
phase. Its expected-red renderer difference therefore does not certify the
pinned action stream.

`data_binding_test.cpp#18`, **Custom Property Trigger Binding**, explicitly
asserts the named circle's initial `scaleX` and `scaleY` and the presence of the
named `CustomPropertyTrigger` owner. The generic SRIV replay omits those three
owner assertions. The later drawing stream is not a substitute for the
missing checks.

### Transition Self substitutes list counts for composite-list mutations

`data_binding_test.cpp#14` creates concrete empty
`ViewModelInstanceListItem` objects, performs two appends, an indexed insert,
an invalid indexed insert, a swap, and two removals, with an advance/draw after
each mutation. The Rust expected-red replaces the first three item mutations
with authored list-count writes and treats the invalid insert as a count
observation. Count changes do not exercise the composite-list item owners or
their mutation notifications. It then fails at swap without ever comparing
the accumulated prefix to the pinned SRIV. The first absent concrete empty-item
mutation is the missing seam; substituted counts cannot move that seam to the
later swap.

### Enum, runtime-property, and TwoWay assertions are narrower

`data_binding_test.cpp#2` changes the enum by the pinned member name
`state-blue`; the Rust test writes numeric index `2`, bypassing the name lookup
that the case asserts.

`data_binding_test.cpp#23` omits the instance's `viewModelName`, the typed-cache
regression asserting that `propertyNumber("str")` returns null, the enum
property's `Horizontal Align` enum name, and the non-enum property's empty enum
name. Schema type-name enumeration and an `enumId == 0` check are not
equivalent owners or assertions.

`data_binding_test.cpp#31` is specifically a TwoWay source-clobber regression.
After settling and changing the source, upstream asserts the retained source X
and Y first, then target X and Y. The Rust expected-red only checks the target
after the advances; its `set_number` helper observes the source before the
advances and cannot detect the clobber the case was added to catch. It also
reorders the first failure from the source owner to the target owner.

## Accepted high-risk areas

- The six passing cycle cases and the three-level expected-red use live parent,
  child, and grandchild occurrences and preserve event/next-frame ordering.
- The three shared-instance cases preserve every pinned downstream identity
  observation: both shared targets update together, distinct targets remain
  isolated, and a newly created parent receives independent empty children.
- Dynamic listener image binding and stateful component image binding decode
  `open_source.jpg` before assignment, preserve click/advance/draw ordering,
  and use the live runtime-image owner. The dynamic case reaches its documented
  renderer-stream divergence only after the complete action sequence.
- The target-to-source TwoWay red (`data_binding_test.cpp#40`) mutates the live
  Node owner and fails on the pinned retained source assertion. Event-driven
  parent/child propagation and the listener-view-model SRIV stream are also
  complete.
- Of the 36 SRIV-backed rows, 33 preserve the pinned render-only action and
  assertion stream. The three exceptions are embedded-image reset, image
  fit/alignment #5, and Custom Property Trigger Binding described above.

## Mechanical and execution gates

- strict pinned identity, ordinal, source line, exact name, evidence locator,
  executable-test discovery, exact ignore-reason, and declared-census
  validation: 70/70 green;
- all 51 declared passing evidence rows executed successfully;
- all 19 declared expected-red rows were forced individually, each selected
  exactly one test and failed inside its named body at the documented runtime
  assertion or SRIV difference;
- all 36 SRIV evidence IDs resolve to exact manifest provenance and pinned
  expected files;
- repository correspondence checker: 157 files and 1,404 pinned
  `TEST_CASE`s, green (the independent main case ledger remains pending);
- correspondence checker unit suite: 24/24 green;
- scoped formatting and diff checks for the reviewed commits: green.

Mechanical success does not promote the 16 semantically incomplete rows.
Wave B1 needs correction and fresh independent review before it can be
accepted at 70/70.
