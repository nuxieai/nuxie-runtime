# W63 — Joint round-7 corrective (W60/W61/W62 findings)

Reviews: .flc5/out/W60-oracle-round6.md, W61-standards-round6.md,
W62-flb-round6.md. All orchestrator-upheld. The three delivery findings
are ONE coherent redesign — implement them together, not as patches.

## 1. Per-callback singleton delivery (W60-1, W62-1) — BINDING DESIGN

C++ granularity: each crossed keyframe callback constructs a singleton
report and completes the ENTIRE chain immediately — listener
notification, data-bind update, recursive bubbling, audio — before the
next callback fires and before the animation mix continues
(keyed_property.cpp:90, scene.cpp:33, linear_animation_instance.cpp:442,
state_machine_instance.cpp:3041,3155). Thread a per-callback dispatch
sink through the reporting advance path (animation.rs:1443 accumulation
replaced): the sink is an instance-owner policy closure that runs the
full chain per callback. Multi-callback frames are callback-major.

## 2. Discard nested-simple secondsDelay (W60-2)

C++ discards the computed delay: EventReport(event, 0)
(linear_animation_instance.cpp:442, nested_animation.hpp:50). Zero the
delay at that seam. Differential must use an OVERSHOOTING advance (past
the keyframe) so the divergence is observable, plus the batch-boundary
assertion below.

## 3. Recursive full-height bubbling (W60-3)

At every nesting depth, each owner's report chain completes to the TOP
of the ancestry (all ancestor locals, then audio unwind) at report time
— before the next owner, before subtree continuation, and before source
audio, exactly as C++'s recursive listener calls
(state_machine_instance.cpp:3155). The error path at any depth must not
discard already-reported chains: each completed chain stands; the
failing owner's chain completes through audio before the error
propagates. Deep-topology differential: three-level nesting with an
intermediate-host owner reporting, asserting full-height completion
before sibling/subtree work, plus the deep error-path case.

## 4. C++ probe batch boundaries (W62-1 evidence)

The probe recorder must record NOTIFY-BATCH BOUNDARIES (not a flattened
vector) so callback-major C++ vs batched Rust is distinguishable; the
differentials assert per-callback singleton batches.

## 5. Restore the weakened differentials (W60-nb-2)

Restore C++/Rust elapsed-time parity (0.25/0.25) in cpp_probe.rs:21378
and :83875. If restoring them fails, the failure is a REAL divergence
uncovered by items 1-3 — fix production, never the inputs. Explain in
your report what the desynchronization was hiding.

## 6. Blend1D clone/remount (W62-2)

Enable clone_remount for the Blend1D case in the wrong-artboard
differential's table so its from/to occurrence identities are proved
across clone/remount.

## 7. Structural ratchet rewrite (W61-1)

Replace the four regex ratchets with structural detection in check.py
(pattern: the semantic scanner's approach): in non-owner files, detect
report-queue access (reported_event_count/reported_event/take_* in any
call form incl. UFCS), notify_events-family calls, audio-seam touches,
and nested-animation selection matches — allowlisting only the blessed
policy entry calls. Negatives MUST include the reviewer's exact evasion
forms: UFCS spellings, impl-ArtboardInstance relocation, and consistent
helper renames of the current mechanics.

## 8. Packet prose (W60-nb-1, W61-nb, W62-nb)

Reword the status NEXT so it is true AT the publication commit
(reviews-then-promotion, no publish-this instruction). Make the
impl-spec self-contained: copy the operative round specs (W48/W53/W58/
W63) into docs/runtime-frame-loop-fl-c5-evidence/round-specs/ and cite
those tracked paths instead of .flc5.

## Acceptance (run yourself; all green)

- New/changed differentials live-green red-first where practical:
  per-callback singleton order (multi-callback frame), overshoot delay,
  deep-topology atomicity, deep error path, restored 0.25 parity pair,
  Blend1D clone/remount. Name each in the receipt.
- cargo test -p nuxie-runtime --lib; cargo test -p nuxie-runtime
  --features tools --test cpp_probe; -p nuxie --lib; both goldens;
  make runtime-frame-loop-port-check with the structural ratchet
  negatives demonstrated (trace steps pending E5 reported honestly).
- fmt/diff clean. Never weaken a test; never use rm-style commands.

Do NOT commit. Report per-finding fixes with citations; separate
production/tests/checker/prose in the staged list.
