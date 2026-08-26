# Wave B4 independent adversarial review

Candidate: `023aefab0`

Pinned upstream: `4ac7b32798da0482e441ef09304dc3b480ed3ee5`

Verdict: **REJECT — 24 accepted, 14 rejected**

This receipt is review-only. It changes no candidate test, ledger, fixture,
manifest, or production source.

## Exact census

- Accepted: 24/38 — 23 passing rows and one executable expected-red.
- Rejected: 14/38 — 10 passing rows whose evidence is narrowed or owned by
  the wrong surface, and four expected-red rows that do not execute the exact
  pinned action flow to the claimed seam.
- Candidate execution remains mechanically green: all 33 declared passing
  rows pass, and all five ignored rows can be forced to fail individually.
  Mechanical success does not repair the semantic proof gaps below.

## Rejected rows

### Follow-path cases 1-3

`follow_path_constraint_test.cpp#1`, `#2`, and `#3` are not exact ports.
Each upstream case:

1. resolves both names specifically as `TransformComponent` owners;
2. calls the root `Artboard::advance(0)` settlement boundary;
3. decomposes both world transforms; and
4. compares the decomposed `x` and `y` values.

The shared Rust helper only finds arbitrary named graph components, calls
`update_pass()`, and compares raw matrix slots 4 and 5. It therefore omits both
type assertions and bypasses the exact advance/decompose owner path. Passing
on these fixtures cannot certify the pinned behavior.

### All five font cases

`font_test.cpp#1`, `#2`, `#4`, and `#5` decode through `RawTextFont` but then
open the retained bytes as a fresh `SkrifaFontRef` and interrogate Skrifa
attributes, metrics, axes, GPOS, and GSUB tables directly. Those are source
font-table probes, not tests of the corresponding runtime `Font` owner
surfaces used by pinned C++ (`getWeight`, `isItalic`, `lineMetrics`,
`capHeight`, `xHeight`, `getAxis*`, `makeAtCoords`, and `features`). They can
stay green while the runtime font behavior is missing or wrong.

Case `#2` additionally does not implement pinned Catch `Approx` semantics.
Catch scales its default epsilon by the expected value only. The helper adds
`1.0` and uses `max(abs(actual), abs(expected))`, admitting values that the
pinned assertion rejects.

`font_test.cpp#3` does exercise the live shaping path and proves fallback
glyph byte-owner identity, but its claimed cleanup equivalence is inert. It
drops only the derived line vector while the original glyph vector, run,
primary font, and occurrence-local fallback chain remain live, then asserts a
new empty `Vec` is empty. That does not translate the pinned destruction of
the shaped paragraphs followed by clearing fallback ownership and removing
the fallback procedure.

### Global-binding cases 4, 5, 6, and 12

`global_view_model_binding_test.cpp#4` and `#5` claim expected-red at the
missing Artboard `setViewModelInstance` owner, but never invoke an equivalent
owner. Their helper passes the non-global main instance to the existing
global-slot setter and fails because that different setter correctly rejects
a non-global slot name. This is a proxy failure, not execution to the exact
absent seam.

Case `#6` replaces the exact non-null
`StateMachineInstance::bindViewModelInstance` action with separate
`set_view_model_instance` and `bind` calls even though the literal Rust owner
exists. It proves the constituents, not the pinned convenience owner under
test.

Case `#12` omits the pinned retained-identity assertion. C++ keeps the
pre-bind `DataContext` handle and proves that same context gains its main
instance during `bind()`. Rust discards the pre-bind borrow and inspects a new
post-bind snapshot, so wholesale context replacement would still pass.

### Global-viewmodel Silver cases 1 and 3

`global_viewmodels_test.cpp#1` points to a manifest entry with `actions = []`.
The pinned case sets the main instance, creates and sets every global slot,
binds, advances/draws once, then performs 62 frame/advance/draw iterations.
The ignored comparator therefore does not execute the claimed complete action
stream and cannot establish the stated missing/nonterminating seam.

`global_viewmodels_test.cpp#3` also narrows the owner flow. The pinned case
uses explicit state-machine setters and preserves two different mutation
orders: main then global in the first block, global then main in the second.
The manifest instead creates already-completed contexts and applies each with
`bind-prepared-view-model`; it neither invokes nor preserves those setter
orders. Its later SRIV mismatch is not evidence for the exact pinned action
stream.

## Accepted rows

- Follow-path Silver cases `#4-#8`: their manifest fixtures, state-machine
  advances, frame counts, draw order, and expected SRIV identities match the
  pinned producers.
- Gamepad cases `#1-#7`: the six direct cases submit the exact little-endian
  records to the live `StateMachineInstance` owner in pinned order, and the
  Silver case executes the complete record/advance/draw/focus stream before
  its concrete comparator failure.
- Global-binding cases `#1-#3`, `#7-#11`, and `#13-#15`: exact fixture,
  retained slot identities, set/get/null validation, ordering, bind, and
  untouched-slot assertions are preserved.
- Global-viewmodel Silver case `#2`: exact fixture, default binding, two
  advances, frame boundary, draws, and full SRIV comparison are preserved.

## Mechanical validation

- Passing targets: 33/33 green (`3 + 5 + 6 + 13 + 6`).
- Forced expected-red targets: 5/5 selected individually and failed.
- Strict pinned identities, names, source lines, evidence locators,
  classifications, and ignore reasons: 38/38 mechanically valid.
- Repository correspondence checker: 157 files and 1,404 pinned cases green.
- Correspondence checker unit suite: 24/24 green.
- Non-test `nuxie-runtime` artifact contains no Wave B4 symbols.
- Candidate diff check: green.

Wave B4 must correct these 14 rows and receive a fresh independent semantic
review before acceptance.
