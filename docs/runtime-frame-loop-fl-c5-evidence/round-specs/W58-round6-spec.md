# W58 — Joint round-6 corrective (W55/W56/W57 findings)

Reviews: .flc5/out/W55-oracle-round5.md, W56-standards-round5.md,
W57-flb-round5.md. All orchestrator-upheld. Small, named fixes only —
no broad refactoring. Never weaken a test; never use rm-style commands.

## 1. Per-ANIMATION chain atomicity (W55-1)

C++ loops nested animations individually (nested_artboard.cpp:989); each
NestedStateMachine::advance completes local apply, ancestor bubbling, and
audio unwind synchronously before the next ANIMATION advances — even on
the same host. Move the advance_nested_event_source_with policy boundary
from per-NestedArtboard-component to per-nested-animation-owner: each
reporting animation's chain (dispatch + audio) completes before the next
animation on the same host advances, and before the host's child subtree
work continues. The error path refines identically: a later failure must
not leave an earlier animation's collected reports undispatched — each
animation's chain completes at its own boundary. Extend the production
test and live differential with the multi-reporter single-host shape
(the existing three-machines-on-one-host fixture at artboard.rs:11872 is
the template): C++ order A-local, ancestor-A, ancestor-A-audio, A-audio,
B-local, ... must hold.

## 2. Nested linear-animation event delivery (W55-2)

C++ registers nested linear animations as event notifiers
(state_machine_instance.cpp:2025); NestedSimpleAnimation advances with
the reporting instance (nested_simple_animation.cpp:13) and
LinearAnimationInstance::reportEvent synchronously notifies parent
listeners (linear_animation_instance.cpp:442). Route
RuntimeNestedAnimationInstance::Simple advancement through the reporting
facade so timeline Event/AudioEvent reports flow into the same
per-animation chain policy as state-machine sources (local -> ancestor ->
audio ordering identical). Add a live differential: a nested simple
animation with a timeline Event and an AudioEvent — parent listener
fires and the audio seam receives exactly the AudioEvent, at C++ order.

## 3. Event-collection/audio mechanics behind owner policy (W56-1)

The queue-drain/copy inside RuntimeNestedStateMachineInstance::advance
and the audio-unwind initiation inside RuntimeNestedArtboardInstance::
advance are the notify-seam mechanics owned by state_machine_instance.cpp
(C++ NestedStateMachine::advance is a trivial delegation —
nested_state_machine.cpp:16). Hoist BOTH into instance-owner policy
functions that the nested/artboard files call with their borrows; the
composition (animation ordering, data-bind ordering) stays put with a
precise impl-spec boundary sentence replacing the over-broad one. Re-key
the four ratchets to the REAL boundary: forbid report-queue iteration,
dispatch decisions, and audio-seam initiation outside the instance owner
(renamed-shape negatives that would catch the current forms), while
permitting plain policy calls.

## 4. Reset-order restoration + interrupted-transition proof (W57-1)

Restore the pinned order in state_machine_layer_instance.rs:747: remove
the superseded state_from's key-frame binds, release/replace state_from,
and only THEN construct transition_animation_reset
(state_machine_instance.cpp:573-585). Add the interrupted-transition
differential (a second transition selected while state_from is alive
with key-frame binds) that fails against the current inverted order.

## 5. Blend re-verification completion (W57-2)

Extend the state-construction wrong-artboard differential to construct
Blend1D and BlendDirect states (machine on A advanced via B uses A's
definitions in both runtimes), satisfying the FL-B authorization's
designated-proof requirement including a clone/remount blend case.

## 6. Prose (W55-nb, W56-nb)

Fix the stale status NEXT pointer (E3 is published; next = round-6
production commit -> floors -> E4 -> reviews). Reword the atomicity
differential's claim honestly: the C++ audio chronology is established
by source citation while report names are live-compared — or strengthen
the probe to record C++ audio phases if the probe tool already carries
the plumbing.

## Acceptance (run yourself; all green)

- New/extended differentials live-green, red-first where practical
  (name the multi-reporter, nested-simple-event, interrupted-transition,
  and blend wrong-artboard tests explicitly in the receipt).
- cargo test -p nuxie-runtime --lib; cargo test -p nuxie-runtime
  --features tools --test cpp_probe; -p nuxie --lib; both goldens;
  make runtime-frame-loop-port-check with the re-keyed ratchet negatives
  demonstrated (the two trace steps report honestly as pending E4).
- fmt/diff clean.

Do NOT commit. Report per-finding fixes with citations, separating
production from prose in the staged list.
