# Wave A animation-state correction

Pinned upstream: `4ac7b32798da0482e441ef09304dc3b480ed3ee5`

Scope: only the 14 `TEST_CASE`s in
`tests/unit_tests/runtime/animation_state_instance_test.cpp`.

The independent Wave A review correctly rejected the original evidence. Its
single matrix constructed `LinearAnimationInstance` directly and multiplied
elapsed time outside the state owner. That bypassed the exact
`AnimationStateInstance` construction and advance path exercised upstream.

The correction reads every complete pinned case and gives each ordinal a
distinct Rust test. Each test now:

1. constructs the upstream dummy `Artboard`/instance equivalent;
2. creates the authored `LinearAnimation` and `AnimationState` definitions;
3. constructs the real Rust animation-state occurrence through
   `RuntimeStateInstance::make`;
4. invokes `RuntimeStateInstance::advance` when the pinned case advances; and
5. evaluates the same `time`, `totalTime`, and `spilledTime` assertions on the
   occurrence-owned `LinearAnimationInstance`.

The four construction-only cases inspect the owned animation immediately, in
the same action order as upstream. The six spill cases preserve their exact
duration, FPS, animation speed, loop mode, elapsed time, and all three
assertions. No production behavior changed and none of the 245 other Wave A
rows is re-adjudicated here.

Focused verification:

```text
CARGO_INCREMENTAL=0 cargo test -p nuxie-runtime upstream_animation_state_instance_ --lib
running 14 tests
test result: ok. 14 passed; 0 failed; 0 ignored
```

This correction resolves only the animation-state rejection in
`wave-a-review.md`; the review's other rejected rows and its broad Wave A
verdict remain unchanged pending their own corrections and independent review.
