# Wave C14 scroll-velocity independent adversarial review

Verdict: **ACCEPTED — 4/4 structured adaptations pass**

Reviewed candidate: `91950130b45c5e862d9bb03da80023196b4944b5`

Pinned upstream: `4ac7b32798da0482e441ef09304dc3b480ed3ee5`

## Body-level correspondence

I independently compared all four Catch bodies in pinned
`tests/unit_tests/runtime/scroll_velocity_test.cpp` against their four
distinct executable Rust tests in
`crates/nuxie-runtime/tests/upstream_scroll_velocity.rs`.

- All four import the exact pinned asset, instantiate the default artboard,
  select the first retained `ScrollConstraint`, and perform the initial
  zero-second artboard advance.
- Cases 1-3 select the exact `State Machine 1` and preserve every pointer
  coordinate, pointer event order, pointer release, and state-machine advance.
- Case 1 preserves the initial X/Y zero and inactive assertions, active drag
  with zero Y velocity, nonzero Y velocity after the 200-pixel upward move,
  active state during the pause, and running physics plus active state
  immediately after release.
- Case 2 preserves the swipe stream, immediate running/active assertions, the
  at-most-600 settle loop, `0.016` advance, early break only when the retained
  physics stops, and the terminal stopped/inactive/zero-X/zero-Y assertions.
- Case 3 preserves the horizontal fixture and exact leftward drag, nonzero X,
  zero Y, active state, and terminal release.
- Case 4 writes `scrollPercentY = 0.5` to the same retained occurrence and
  preserves the exact inactive/zero-X/zero-Y idle assertions.

Every asserted velocity, physics-running, and activity value is read from the
live retained `RuntimeScrollConstraintState` and its owned physics through
`RuntimeScrollConstraintSnapshot`. The snapshot does not recompute those
observables, and no test-local physics, expected-value algorithm, aggregate
runner, proxy fixture, or placeholder failure is present.

## Structured adaptations

The first three rows adapt only the pinned C++ ambient
`high_resolution_clock` sample to the Rust host API's explicit deterministic
pointer timestamp. The initial hover remains at timestamp zero and the authored
drag move supplies timestamp `1.0`, creating a positive elapsed tick without
inventing a numeric velocity expectation. Pointer coordinates, event order,
advance durations, asserted signs/zeros, active state, release behavior, and
settle behavior are unchanged.

Case 4 adapts only the typed C++ `setScrollPercentY` call surface. Rust resolves
the exact `ScrollConstraint.scrollPercentY` schema key and sends `0.5` through
`ArtboardInstance::set_double_property` to the same first occurrence. The
generic path reaches the retained ScrollConstraint percent-intent owner; it is
not a detached object-table write or test-only setter.

The old expected-red explanation is stale: the runtime now retains and exposes
the exact velocity and activity owners. The evidence file contains no ignore,
missing-owner panic, or stale expected-red reason.

## Provenance and gates

- Pinned checkout identity is exact. The pinned source and both fixture working
  blobs match their Git objects. Their SHA-256 values are:
  - `scroll_velocity_test.cpp`:
    `b137de2cb47b45400eb973a79ae89672a2ecc77865dc79efc7a8d675ba404c6f`;
  - `layout_scroll_vertical.riv`:
    `8b1d5a1a14576cb32f2d1fc7d8bf05ed6000128d9545cf8a6760559a42b5cd20`;
  - `layout_scroll_horizontal.riv`:
    `cd0746ddc52519c1bd9ef40cb8f489e1f80231c0a2af65dd40e8e79168b3a3f4`.
- Strict pinned identity, ordinal, source-line, exact-name, classification,
  outcome, structured-adaptation metadata, and evidence-locator validation:
  4/4 green.
- Focused non-incremental suite: 4 passed, zero failed, zero ignored.
- Repository correspondence checker: 157 files / 1,404 cases, green.
- Correspondence-checker unit suite: 24/24 green.
- Default non-test release LLVM IR contains none of the Wave C14 velocity test
  symbols or fixture names.
- Candidate JSON parsing, scoped `git diff --check`, production freeze, and
  stale-red/ignore scans are green. Candidate `91950130b` changes only its
  two correspondence documents.

Every relied-on Cargo invocation used `CARGO_INCREMENTAL=0` and disabled
incremental compilation for the invoked test or release profile.
