# Wave A component-list correction receipt

Pinned upstream: `4ac7b32798da0482e441ef09304dc3b480ed3ee5`

Scope: only `component_list_test.cpp` cases 1-16, 20, 21, 28, and 30 that
the independent Wave A review rejected as metadata-only or narrower than the
pinned case. The ten previously accepted component-list rows were not used as
correction targets.

## Executable evidence

`crates/nuxie-runtime/src/artboard/component_list_wave_a.rs` imports each
pinned `.riv` fixture, binds its default view model, executes the upstream
advance, pointer, mutation, scroll, ordering, or draw flow, and asserts the
concrete Rust owner corresponding to the upstream owner. The frozen raw C++
anchors remain provenance only; no corrected manifest row cites them as test
evidence.

The Rust ownership adaptations are explicit:

- a test-only observer verifies that a row state-machine instance retains the
  mounted row Artboard's exact definition arena, corresponding to C++'s
  `sm->artboard() == artboard` pointer assertion;
- hosted root layout position is read from the mounted row Artboard's retained
  layout owner;
- the clip probe interprets the recording renderer's save/restore, affine
  transform, path, and clip stream and asserts the pinned world-space bounds.

Cases 16, 20, and 21 also restore the direct assertions omitted by their
existing Silver mappings:

- case 16: initial `scrollPercentY`, `offsetY`, scroll index, stopped physics,
  dragged X offset/index, and running physics after pointer-up;
- case 20: initial `(x, y)` values for all three list-item view models;
- case 21: initial `ItemCount == 10` and the live 10-to-5 list mutation.

## Outcomes

The 20 direct Rust paths produce 19 passes and one concrete expected-red:

- case 7 reaches the pinned first-row hover action, then observes the third
  row's `Hover` input already `true` where upstream requires `false`;
- the other 19 direct paths pass, including cases 4, 15, and 30 after correcting
  false failures in the first translation (the hosted layout owner, viewport
  owner, and one scrolled draw respectively).

The manifest retains expected-red outcomes for cases 16 and 20 because their
full pinned Silver streams still diverge even though the newly restored direct
assertions pass. Case 21's direct assertions and Silver replay both pass.
Thus the selected manifest rows finish as 17 pass and 3 expected-red.

## Verification

```text
CARGO_INCREMENTAL=0 RIVE_RUNTIME_DIR=/Users/levi/dev/oss/rive-runtime \
  cargo test -p nuxie-runtime --lib wave_a_component_list_case_ -- --nocapture

19 passed; 0 failed; 1 ignored
```

Running the ignored case directly reaches its exact divergent assertion:

```text
CARGO_INCREMENTAL=0 RIVE_RUNTIME_DIR=/Users/levi/dev/oss/rive-runtime \
  cargo test -p nuxie-runtime --lib \
  wave_a_component_list_case_07_state_machine_listener -- --ignored --nocapture

left: Some(true)
right: Some(false)
```

`jq empty docs/runtime-test-case-waves/wave-a.json` and scoped
`git diff --check` also pass. This receipt does not certify the rest of Wave A
and contains no production behavior fix.
