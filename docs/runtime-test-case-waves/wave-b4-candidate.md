# Wave B4 exact test-port candidate

Wave B4 ports every pinned `TEST_CASE` in upstream runtime files 41 through
45 at `4ac7b32798da0482e441ef09304dc3b480ed3ee5`:

- `follow_path_constraint_test.cpp`: 8/8;
- `font_test.cpp`: 5/5;
- `gamepad_test.cpp`: 7/7;
- `global_view_model_binding_test.cpp`: 15/15;
- `global_viewmodels_test.cpp`: 3/3.

The exact census is 38 executable rows, zero pending: 36 direct and two
Rust-safety adaptations; 33 pass and five expected-red.

## Owner evidence

The three non-Silver follow-path cases import each pinned `.riv`, advance the
real `ArtboardInstance`, and compare the target and rectangle world
translations. The five Silver follow-path cases execute their complete pinned
action streams and compare the entire SRIV.

The font cases do not reuse the rejected standalone `Font` façade. They are
`cfg(test)` evidence inside the real `RawTextFont` module and decode the pinned
font bytes through the runtime's Skrifa/Harf owners. They assert attributes,
size-scaled cap/x metrics using Catch-style magnitude-scaled approximation,
fallback glyph ownership and paragraph lifetime, cumulative variation axes,
and the exact OpenType feature set. The two declared adaptations are limited
to occurrence-local fallback state and immutable variation settings in place
of C++ global callback state and mutable cloned `Font` identities.

The six non-Silver gamepad cases import the pinned fixture, initialize its real
state machine, submit the exact wire records in order, and preserve each
boolean assertion. The Silver case executes the full pinned stream.

All 15 global-binding cases use the actual Artboard, StateMachineInstance,
DataContext, and retained view-model owners. Cases 4 and 5 are expected-red at
the concrete Artboard seam: after their exact fixture/action prefix, the only
retained Artboard setter rejects the non-global main instance because the
separate upstream `setViewModelInstance` owner does not exist. The tests do not
use an unconditional panic or a fabricated model. The three global-viewmodel
Silver cases execute their complete pinned streams.

## Expected-red census

- gamepad Silver: frame 0, operation 38, expected `makeRenderPaint`, got
  `frameSize`;
- global-binding cases 4 and 5: missing Artboard main-instance setter distinct
  from binding/global assignment;
- global variables Silver: frame 0 stops after operation 49 instead of the
  expected 18,540 operations;
- explicit global instance Silver: frame 1, operation 163, expected `frame`,
  got `color`.

All five ignored tests were forced individually. Each selected exactly one
test and failed at the declared concrete seam.

## Validation

- all passing owner tests: 33/33 green;
- all expected-red tests forced individually: 5/5 concrete failures;
- strict pinned identity, ordinal, source-line, exact-name, evidence-locator,
  classification, and ignore-reason validation: 38/38;
- repository correspondence checker: 157 files and 1,404 pinned cases, green;
- checker unit suite: 24/24 green;
- non-test LLVM IR contains no Wave B4 test symbols;
- scoped `git diff --check` and JSON parsing: green.

This is a candidate pending fresh independent semantic review. It does not
declare Wave B4 accepted.
