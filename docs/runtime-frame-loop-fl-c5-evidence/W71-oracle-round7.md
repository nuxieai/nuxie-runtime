REJECT. Three blocking defects remain.

## Standards

- **BLOCKING — ownership ratchet remains evadable.** W63 requires detection of nested-animation selection under renamed mechanics in every non-owner file ([W63:61](/Users/levi/dev/worktrees/nuxie-e5-review/docs/runtime-frame-loop-fl-c5-evidence/round-specs/W63-round7-spec.md:61)). The detector handles fully qualified variants and `use … as` aliases, but not a plain variant import ([check.py:602](/Users/levi/dev/worktrees/nuxie-e5-review/tools/runtime-frame-loop-port/check.py:602)):

  ```rust
  use RuntimeNestedAnimationInstance::StateMachine;
  if let StateMachine(owner) = animation {
      owner.advance(child, elapsed, Some(&mut reported_events));
  }
  ```

  Direct execution of the detector returned `[]`. Permanent negatives cover `as Machine` and enum aliases, not this valid form ([test_check.py:3757](/Users/levi/dev/worktrees/nuxie-e5-review/tools/runtime-frame-loop-port/test_check.py:3757)). The live checker therefore passes despite a cheap relocation bypass.

- **NON-BLOCKING — publication NEXT remains false.** W63 explicitly requires reviews-then-promotion and “no publish-this instruction” at publication ([W63:72](/Users/levi/dev/worktrees/nuxie-e5-review/docs/runtime-frame-loop-fl-c5-evidence/round-specs/W63-round7-spec.md:72)). Published HEAD still begins with “Publish the staged E5 evidence/docs packet” ([status:829](/Users/levi/dev/worktrees/nuxie-e5-review/docs/runtime-frame-loop-status.md:829)).

## Spec

- **BLOCKING — Blend1D clone/remount introduced non-C++ reset behavior.** Round 7 added `last_applied_artboard_identity` and `last_applied_values`, replays saved values when the target artboard changes, and constructs a fresh `AnimationReset` after every apply ([state_machine.rs:820](/Users/levi/dev/worktrees/nuxie-e5-review/crates/nuxie-runtime/src/state_machine/state_machine.rs:820), [state_machine.rs:1047](/Users/levi/dev/worktrees/nuxie-e5-review/crates/nuxie-runtime/src/state_machine/state_machine.rs:1047)). Pinned C++ owns only the optional constructor-time reset and simply applies it before blending ([C++ constructor](/Users/levi/dev/oss/rive-runtime/src/animation/blend_state_1d_instance.cpp:10), [C++ apply](/Users/levi/dev/oss/rive-runtime/src/animation/blend_state_1d_instance.cpp:67)). FL-B expressly forbids reset ownership or update ordering absent from C++ ([FL-B:189](/Users/levi/dev/worktrees/nuxie-e5-review/docs/runtime-frame-loop-fl-b-spec.md:189)).

  The differential is asymmetric: C++ advances the same owner artboard, while Rust alone replaces the caller with a freshly imported artboard between steps ([cpp_probe.rs:20313](/Users/levi/dev/worktrees/nuxie-e5-review/crates/nuxie-runtime/tests/cpp_probe.rs:20313), [cpp_probe.rs:20325](/Users/levi/dev/worktrees/nuxie-e5-review/crates/nuxie-runtime/tests/cpp_probe.rs:20325)). The added reset state compensates for that non-oracle remount instead of merely proving retained `from`/`to` identities.

- **BLOCKING — a failing reporting owner still loses its own bubble/audio tail.** W63 requires the failing owner’s chain to complete through audio before propagating its error ([W63:29](/Users/levi/dev/worktrees/nuxie-e5-review/docs/runtime-frame-loop-fl-c5-evidence/round-specs/W63-round7-spec.md:29)). A listener `ScriptError` returns immediately ([state_machine_instance.rs:7470](/Users/levi/dev/worktrees/nuxie-e5-review/crates/nuxie-runtime/src/state_machine/state_machine_instance.rs:7470)), before the bubble/audio operations at [state_machine_instance.rs:7487](/Users/levi/dev/worktrees/nuxie-e5-review/crates/nuxie-runtime/src/state_machine/state_machine_instance.rs:7487). The deep-error test injects failure only in a later sibling `ScriptedDrawable` ([artboard.rs:12018](/Users/levi/dev/worktrees/nuxie-e5-review/crates/nuxie-runtime/src/artboard.rs:12018)); it proves preservation of an earlier successful chain, not completion of the failing owner’s chain.

- **NON-BLOCKING — the required desynchronization explanation was never published.** Both pairs are genuinely restored to `0.25/0.25` ([transition-duration pair](/Users/levi/dev/worktrees/nuxie-e5-review/crates/nuxie-runtime/tests/cpp_probe.rs:21471), [keyframe-context pair](/Users/levi/dev/worktrees/nuxie-e5-review/crates/nuxie-runtime/tests/cpp_probe.rs:83968)). They now expose active-animation time/return progression and nonzero-time progression of the newly bound keyframe occurrence; the prior Rust zero advances tested hydration/order while concealing those time-dependent paths. W67 does not supply W63’s requested explanation.

## Verified held and boundary sweep

- Successful callback paths are callback-major; nested-simple overshoot is zeroed before dispatch, and C++ and actual Rust notify entries both assert `[1,1]` ([production seam](/Users/levi/dev/worktrees/nuxie-e5-review/crates/nuxie-runtime/src/state_machine/state_machine_instance.rs:12969), [differential](/Users/levi/dev/worktrees/nuxie-e5-review/crates/nuxie-runtime/tests/cpp_probe.rs:84937)).
- Successful recursive bubbling occurs during `applyEvents`, before authored layers, with pre-mix visibility and `updateDataBinds(false)` settlement. The blocker is specifically the listener-error exit.
- O1 construction-time preparation, O2 typed authored-order lookup, and O3 semantic node resolution remain intact ([O1](/Users/levi/dev/worktrees/nuxie-e5-review/crates/nuxie/src/lib.rs:3993), [O2](/Users/levi/dev/worktrees/nuxie-e5-review/crates/nuxie-runtime/src/state_machine/state_machine_instance.rs:5326), [O3](/Users/levi/dev/worktrees/nuxie-e5-review/crates/nuxie-runtime/src/state_machine/state_machine_instance.rs:5584)).
- Typed audio selection, retained definition arenas, and interrupted-transition teardown-before-reset remain intact ([audio](/Users/levi/dev/worktrees/nuxie-e5-review/crates/nuxie-runtime/src/state_machine/state_machine_instance.rs:7529), [arena hookup](/Users/levi/dev/worktrees/nuxie-e5-review/crates/nuxie-runtime/src/state_machine/state_instance.rs:27), [reset order](/Users/levi/dev/worktrees/nuxie-e5-review/crates/nuxie-runtime/src/state_machine/state_machine_layer_instance.rs:733)).
- The seven historical FL-B corrections remain present: signed loop override, invalid-interpolator erasure, importer-cursor survival, doomed-object sink, negative-speed remap, NaN direct blend, and empty-baseline reset. Full FL-B nevertheless fails because of the new Blend1D reset behavior.
- Main-side sweep `e72323c8..afe71e30` and merge `192cbbbe..171b5703` found no edits to `nuxie-runtime`, the checker, or ownership ledger. Scene changes are authoring-only; `nux-container` is an optional scripting dependency ([nuxie Cargo](/Users/levi/dev/worktrees/nuxie-e5-review/crates/nuxie/Cargo.toml:9)); Apple removal does not interact with FL-B/FL-C5. The Makefile merge retained the FL trace/probe targets while removing Apple targets.
- The pinned C++ checkout resolves to `d788e8ec…`; the live checker and `git diff --check` pass. Targeted Cargo replay was prevented only by the frozen checkout refusing target-directory creation.

REJECT