# W48 — Joint round-4 corrective (W45/W46/W47 findings)

Reviews: .flc5/out/W45-oracle-round3.md, W46-standards-round3.md,
W47-flb-round3.md. All findings orchestrator-upheld. Never weaken a test.

## FL-C5 oracle

O4-order: restore C++ cross-ancestor ordering
(state_machine_instance.cpp:3155-3169): bubbling is synchronous
depth-first — every ancestor's local dispatch completes before ANY audio
tail runs, and audio tails execute on the unwind (root first, leaf last).
Rework the owner-seam delivery so the instance-owned policy performs
recursive ancestor dispatch before audio tails, exactly:
leaf-local, parent-local, root-local, root-audio, parent-audio,
leaf-audio for a leaf->parent->root chain. Replace the masking
per-instance-trace test with a CROSS-INSTANCE total-order assertion for
that chain (and a two-level variant).

## FL-C5 standards

S-a Exact inventory: field assertions must use coercion-free forms
   (by-value field moves in return position: fn(m: RuntimeStateMachine)
   -> ExactType { m.field } or equivalent destructuring) so a
   Deref-wrapper substitute fails; generic-bound assertions must prove
   the EXACT bound set — pass an FnOnce-only closure (consuming a
   captured non-Clone value) so narrowing to Fn fails to compile, and
   add compile-fail-style negatives demonstrating both. Fold into the
   digest.
S-b Gate the FL-B differential's probe surface:
   RuntimeNestedRemapAnimationReport /
   runtime_nested_remap_animation_reports get the repo's S-TOOLS
   probe gating and leave the unauthorized lib.rs export (relocate or
   gate the re-export; FL-B's closure excludes that file).
S-c Re-key the semantic forbidden-projection ratchet on SHAPE (ordinal
   assignment/scan over SemanticData components coexisting with the
   resolver) with a RENAMED-fallback injected negative.
S-d Receipts provenance: (i) every floor receipt must carry the tree
   SHA inside the file — add a stamp step to the floor targets or a
   documented stamping wrapper used for evidence copies; (ii) the
   evidence copy of floor2-apple.log must disclose the attempt-1
   dirty-tree failure with the rerun story (or split into
   attempt-1/attempt-2 files); (iii) W41-report's 'both reviews no
   findings' prose must be corrected to name its INTERNAL closeout
   reviews, and the W39/W40/W45/W46/W47 independent verdicts must be
   tracked under docs/runtime-frame-loop-fl-c5-evidence/; (iv) fix the
   status-doc NEXT instruction that still tells the coordinator to land
   the already-landed publication commit.

## FL-B

B-a Trace checker must validate rust_ref (== HEAD or the recorded
   production candidate per the packet convention) and verify artifact
   hashes exist/match schema, with injected negatives for a mutated
   rust_ref and a mutated artifact hash.
B-b Retained-owner resolution everywhere: advance, keep-going, and apply
   must resolve the DEFINITION through the instance's retained arena
   (never the caller artboard's), matching C++ retained m_animation
   (linear_animation_instance.cpp:187; linear_animation_instance.hpp:78).
   The caller artboard remains only the application target (documented
   borrow-model adaptation). Add the auditor's exposing differential:
   instance from artboard A index-0 Loop passed to artboard B with a
   different OneShot at index 0 — advance/apply/query all use A's
   definition in both runtimes.

## Acceptance

- New/changed differentials live-green (red-first where practical);
  the O4 cross-instance order test and B-b wrong-artboard test named
  explicitly in the receipt.
- Full: runtime lib, cpp_probe, nuxie lib, public_api (honest adapter
  note), capi, both goldens, port check with ALL new negatives
  demonstrated.
- fmt/diff clean.

Do NOT commit. Report per-finding fixes with citations. Production and
evidence staged separately: after your handoff the orchestrator commits
production P3, reruns all floors WITH SHA stamps, then a final evidence
pass names P3 and lands as E2.
