# W53 — Joint round-5 corrective (W50/W51/W52 findings)

Reviews: .flc5/out/W50-oracle-round4.md, W51-standards-round4.md,
W52-flb-round4.md. All findings orchestrator-upheld; the FL-B scope
question is resolved by the dated authorization now in
docs/runtime-frame-loop-fl-b-spec.md. This round is PRODUCTION+TESTS+
RATCHETS+SPEC-PROSE only — do NOT regenerate the trace or touch the
evidence directory (the orchestrator owns E3 with fingerprint-last
sequencing and personally verifies the checker before committing).

## A. O4 production-path chain atomicity (W50-spec-1/2; W51-1; W50-std-1)

Binding design: C++ completes each reporter's ENTIRE chain synchronously
at report time (state_machine_instance.cpp:3155-3169) — locals up the
ancestry, then audio tails on the unwind — before the next component's
advance can report. Restructure so the instance-owned policy performs
per-source dispatch: during the component advance loop, each nested
reporter's collected events are dispatched (ancestor locals + audio
unwind) IMMEDIATELY after that source's advance, not batched after the
whole list. All collection/selection/dispatch/unwind logic MOVES INTO
state_machine_instance.rs policy functions; artboard.rs (the ~10155
region) and nested_state_machine.rs (the ~346 region) keep only thin
borrow closures. Error path: a later component's ScriptError must not
drop earlier deferred audio — flush before propagating.

Proofs: a PRODUCTION-PATH cross-instance total-order differential with
two sibling reporters A/B under one root driven through the real
advance_and_apply loop (C++ order: A's full chain then B's full chain),
plus the error-path audio-flush test. The manual notify/take/flush tests
may remain but no longer stand as the O4 proof. Extend the ownership
ratchet with the four orchestration markers the W51 reviewer enumerated
(event collection, nested animation selection, dispatch, audio unwinding
in artboard.rs/nested_state_machine.rs) with renamed-shape negatives.

## B. Retained-arena completion (W52-2)

linear_animation_instance_definition (artboard.rs:4504) and its users —
the four state_machine_layer_instance.rs call sites (transition duration,
exit time, refresh; :579,:669,:738,:848) and
animation_reset_factory.rs:101 — must resolve definitions through the
instance-retained arena, never the caller artboard. Extend the
wrong-artboard differential through the state-machine path: a machine
built on A advanced via B uses A's definitions for durations/exit
times/resets in both runtimes.

## C. Semantic scanner two-phase (W51-2)

The forbidden-projection scanner must detect the ordinal-fallback shape
in EVERY state_machine/**/*.rs file regardless of where
SemanticNodeResolver is declared: phase 1 checks the resolver seam exists
repo-wide; phase 2 scans every file for the ordinal shape. Negative: the
renamed fallback in a SIBLING file (e.g. semantic_listener_group.rs) must
fail.

## D. Stamp-guard correctness (W51-4; W52-3)

The receipt-stamp assertion must enumerate tracked floor logs via git
(recursive, no hard-coded count, includes superseded/) and verify each
carries a resolvable stamp. Negative: corrupting any tracked receipt —
including a superseded one — fails.

## E. Binding-command reproducibility + prose (W50-std-3; W52-4; W50-nb)

Update the impl-spec acceptance commands to include --features tools
wherever cpp_probe is invoked; fix the closure checker-test count to
reference the live number (or state it is reported by the gate receipt);
remove the stale status instruction to land the already-landed E2 (the
NEXT pointer should describe P4->E3->round-5 reviews).

## Acceptance (run yourself; all green)

- New production-path O4 differential and extended wrong-artboard
  differential live-green (red-first where practical).
- cargo test -p nuxie-runtime --lib; cargo test -p nuxie-runtime
  --features tools --test cpp_probe; -p nuxie --lib; both goldens;
  make runtime-frame-loop-port-check EXCEPT the trace
  fingerprint/provenance steps which will be red until E3 regenerates
  them — report that step's status honestly and separately.
- All new/changed ratchet negatives demonstrated.
- fmt/diff clean. Never weaken a test; never use rm-style commands.

Do NOT commit. Report per-finding fixes with citations, separating
production files from spec-prose files in the staged list.
