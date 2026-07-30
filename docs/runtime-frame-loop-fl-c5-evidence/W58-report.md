Implemented the W58 round-6 corrective completely. No tests were weakened, no rm-style commands were used, and no commit was created—HEAD remains `50bf85e8c4e8891ce506832004ba268948f91362`.

Production/tests:

- Per-animation event/audio chains are now atomic and instance-owned, including nested simple animations and error ordering: [state_machine_instance.rs](/Users/levi/dev/worktrees/nuxie-fl-c/crates/nuxie-runtime/src/state_machine/state_machine_instance.rs:12574), [artboard.rs](/Users/levi/dev/worktrees/nuxie-fl-c/crates/nuxie-runtime/src/artboard.rs:6189).
- Nested state-machine advancement is a trivial delegation: [nested_state_machine.rs](/Users/levi/dev/worktrees/nuxie-fl-c/crates/nuxie-runtime/src/state_machine/nested_state_machine.rs:326).
- Interrupted-transition teardown/reset order matches pinned C++: [state_machine_layer_instance.rs](/Users/levi/dev/worktrees/nuxie-fl-c/crates/nuxie-runtime/src/state_machine/state_machine_layer_instance.rs:733).
- Required live differentials are present:
  - `fl_c5_same_host_multi_reporter_chains_are_atomic`
  - `fl_c5_nested_simple_timeline_event_and_audio_are_delivered`
  - `fl_c5_interrupted_transition_releases_old_keyframe_binds_before_reset`
  - `blend_states_from_artboard_a_use_a_definitions_through_wrong_artboard_and_clone_remount_like_cpp_probe`
- The C++ probe now records exact nested-simple Event/AudioEvent type, count, and order: [main.cpp](/Users/levi/dev/worktrees/nuxie-fl-c/tools/cpp-probe/main.cpp:360).

Prose/ratchets:

- Re-keyed all four owner-boundary ratchets, including helper-name-independent dispatch-decision detection: [runtime-frame-loop-gaps.toml](/Users/levi/dev/worktrees/nuxie-fl-c/docs/runtime-frame-loop-gaps.toml:1397).
- Clarified the implementation boundary and honest audio evidence: [runtime-frame-loop-fl-c5-impl-spec.md](/Users/levi/dev/worktrees/nuxie-fl-c/docs/runtime-frame-loop-fl-c5-impl-spec.md:51).
- Updated phase and Next sequencing through floors, E4, and reviews: [runtime-frame-loop-status.md](/Users/levi/dev/worktrees/nuxie-fl-c/docs/runtime-frame-loop-status.md:833).

Validation:

- `nuxie-runtime --lib`: 720 passed
- `cpp_probe`: 822 passed
- `nuxie --lib`: 146 passed
- Ordinary and scripted goldens: 317/317 supported, 647/647 cases, zero divergences
- Checker/ratchet tests: 67/67 passed
- Final trace gate reports exactly the two spec-authorized pending E4 items: candidate fingerprint and runner provenance
- `cargo fmt --all -- --check`: clean
- `git diff --check`: clean
- Independent spec and standards re-reviews: no remaining findings

Elapsed goal time: about 61 minutes.