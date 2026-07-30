# W48 joint round-four corrective handoff

Base publication commit:
`ff94a5f264f461a54ee81328e687f3d5ae0bed21`.

This is an uncommitted handoff. It is not the final P3/E2 publication receipt.
The orchestrator must split production from evidence, commit production as P3,
rerun every floor with P3 stamped inside each receipt, regenerate the trace
with `rust_ref=P3`, and land the resulting evidence-only packet as E2.

## Per-finding corrections

- **O4:** event delivery now uses instance-owned deferred owner-audio
  occurrences. Each nested report batch runs local listeners, recursively
  bubbles through its owners, and then flushes audio while unwinding from root
  to leaf before the next report batch. The shared chronology tests are
  `fl_c5_event_bubbling_precedes_the_recorded_audio_seam_through_two_ancestors`
  (`leaf-local, parent-local, root-local, root-audio, parent-audio,
  leaf-audio`) and
  `fl_c5_event_bubbling_cross_instance_total_order_through_one_ancestor`.
  The three-level test was red first with the old leaf-audio-first order.
- **S-a:** the four public definition fields are moved by value into exact
  return types. All hydration callbacks consume a borrowed, non-Clone,
  non-Send/non-Sync token, excluding `Fn`, `Clone`, `Send`, `Sync`, and
  `'static` narrowing. Compile-fail negatives prove a deref wrapper and an
  `Fn` bound are rejected, and the digested inventory was refreshed.
- **S-b:** the nested-remap report type, query, re-export, and `cpp_probe`
  integration test are all gated by the `tools` feature. Live ratchets and
  injected missing-gate negatives cover all four surfaces.
- **S-c:** the forbidden semantic fallback is detected by function shape:
  ordinal iteration/counter projection over `SemanticData` while the
  `SemanticNodeResolver` seam exists. A renamed `relabeled_data_slot` /
  `matching_order` fallback is rejected.
- **S-d:** all 13 tracked historical floor logs now carry their candidate SHA
  inside the file. `floor2-apple.log` discloses the attempt-1 dirty-tree
  refusal and points to the successful clean-tree XCFramework attempt.
  `stamp_floor_receipt.py` atomically stamps future P3 copies, with positive,
  replacement, malformed-SHA, and tracked-receipt tests. W41 now identifies
  its reviews as INTERNAL closeouts; independent W39/W40/W45/W46/W47
  rejection verdicts are archived here; the status NEXT section now directs
  P3, stamped reruns, then E2.
- **B-a:** the checker accepts only a full existing `rust_ref` equal to HEAD or
  an ancestor separated by publication-only documentation. It requires the
  exact eight-hash v2 artifact schema and equality with the packet manifest.
  Mutated-ref and mutated-artifact negatives fail.
- **B-b:** linear-animation advance, keep-going, and apply resolve through the
  instance's retained definition arena. The caller artboard is documented as
  only the mutable apply target. The live differential
  `linear_animation_instance_from_artboard_a_uses_a_definition_when_called_through_artboard_b_like_cpp_probe`
  constructs A/index-0 Loop and B/index-0 OneShot with different speed and
  keyframes; advance, query, and apply all match C++ A-definition behavior.
  This differential was red first against caller-artboard resolution.

## Current in-sandbox acceptance

- Runtime library: 716/716.
- Live C++ differential suite (`--features tools`): 816/816.
- `nuxie` library: 146/146.
- Exact FL-C5 public inventory: 1/1.
- C API: 3 library tests and 16 integration tests.
- Public API: 14/14 code/API cases pass. The unchanged default-renderer
  construction case cannot run because this sandbox exposes no graphics
  adapter (`metal found no adapters`); it was not skipped or weakened.
- Ordinary golden: 317/317 entries and 647/647 exact segments; zero
  divergences, unsupported features, or not-yet cases.
- Scripted golden with all diagnostic verifiers: 317/317 entries and 647/647
  exact segments; zero divergences, unsupported features, or not-yet cases.
- Frame-loop checker tests: 66/66, including every new injected negative and
  the 13-receipt stamp assertion.
- Live structural/provenance checker: green after regenerating the current
  uncommitted-candidate trace and matching all eight artifact hashes.
- `cargo fmt --all -- --check`, unstaged `git diff --check`, and staged
  `git diff --cached --check`: green at handoff; nothing is staged or
  committed.

## Required production/evidence split

P3 contains runtime code, feature declarations, differentials, checker and
stamp tooling, Makefile wiring, and the live gaps/ownership ratchets. E2
contains this evidence directory, the regenerated P3 trace, closure/status
prose, P3-stamped floor reruns, and the final P3 acceptance counts. The
historical floor stamps remain historical evidence and must not substitute for
P3 reruns.
