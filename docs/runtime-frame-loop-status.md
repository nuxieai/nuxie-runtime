# Runtime Frame-Loop Port Status

Sole resume state for the C++-corresponding frame-loop performance closeout.

## Current

- Phase: FL-0 execution atlas.
- Pinned C++: `d788e8ec6e8b598526607d6a1e8818e8b637b60c`.
- File closure: 0 / 337 in-scope C++ files.
- Member closure: 41 / 74 owner/member rows (the imported, already-closed
  runtime-drawing ledger); 33 frame-loop rows pending.
- Open mechanism gaps: 7 / 8.
- Current dependency wave: none; production translation is blocked on FL-0.
- Current experimental changes: uncommitted KeyFrame retained-seconds and
  Component-handle candidates remain quarantined. They are not standalone
  slices and must be re-derived in FL-B/FL-A or discarded.

## FL-0 evidence

- Static closure: seeded and reviewed. Six non-overlapping source sets expand
  to 337 explicit file rows across component/update, animation, state machine,
  DataBind/Artboard, and live draw. The 103 dynamically reached rows and 234
  cold rows are machine-checked against trace evidence; each cold family stays
  in scope under its virtual-dispatch/dependency rationale.
- Dynamic reachability: captured from LLVM function-entry counters with
  construction counters reset immediately before the sample loop. C++ reached
  461 functions in 103 / 337 scoped files; Rust reached 1,087 functions in 18
  runtime modules. Full names and counts are in
  `docs/runtime-frame-loop-trace.json`.
- Deterministic structural counters: captured on the same six entries and 11
  samples against clean Rust `13aedd6d` and pinned C++. Exact pairs:
  Artboard/SMI/LinearAnimation construction 24/24, 24/24, 27/27;
  SMI advance 30/30; layer advance 31/31; animation advance 38/38; update pass
  29/29; component update 29/29; event batch 30/30; keyframe-double apply
  steps 124/124; layout compute 24/24; public/internal draw 11/11 and 30/30.
- Structural mismatches are now finite owner-family work:
  - FL-A: Component dirt additions C++ 201 vs Rust 287.
  - FL-C: transition searches 176 vs 154.
  - FL-D: Artboard DataBind batches 90 vs 113.
  - FL-A/FL-E integration: draw-order sorts 24 vs 607, clipping redundant-list
    clears 48 vs 1,214, and drawable owner lookup 0 vs 448.
  - Cross-wave allocation oracle: C++ 2,732 vs Rust 6,118 frame-loop
    allocations (debug coverage runners, identical corpus/samples, counter
    reset after construction).
  Each mismatch has a machine-checked gap row. None is a benchmark-scene
  slice.
- Deterministic renderer-feed operations are exact: 11 frames, 148 drawPath,
  134 makeEmptyRenderPath, 283 makeRenderPaint, 32 makeLinearGradient, 17
  clipPath, 146 transform, 152 save/restore, and one image decode on both.
- Cold lifecycle oracle: clean `13aedd6d` targeted tests
  `public_artboard_clone_is_cold_but_transient_layout_clone_keeps_scripts` and
  `mounted_child_backend_resources_clone_and_remount_cold` both pass (1/1
  each), preserving public clone identity separation and cold backend
  remounts. Their C++ lifecycle citations remain in the imported drawing
  ledger.
- Fail-closed checker: included in the FL-0 map commit with nine checker
  negative controls plus three summarizer unit tests. It rejects scope growth,
  overlaps, missing per-file rows, stale dynamic markers, premature close,
  unverified file promotion, missing adaptation rules, untracked counter
  mismatches, and renderer-stream work mismatches.
- Trace harness: opt-in and isolated. Instrumented C++ uses a dedicated runtime
  archive and runner name with a trace-flags stamp next to `librive.a`; Rust
  uses a dedicated Cargo target and feature. Both runners reject unavailable
  instrumentation and repeated benchmark mode rather than emitting misleading
  evidence. Ordinary runner paths remain untouched.
- Map/checker commit: this FL-0 slice. No production behavior change may land
  until its clean committed-tree floor is recorded.

The prior sampled seven-divergence run used a release-linked C++ ordinary
runner and is invalid ordinary-golden evidence. Ordinary parity uses only
`env -u CPP_CONFIG -u RUST_PROFILE make golden-compare` with the checked-in
debug C++ configuration and its provenance stamp.

## Baseline performance

- Last committed-tree canonical hot-loop artifact:
  `target/perf-hot-loop-13aedd6d.json`.
- Aggregate at `13aedd6d`: approximately 1.479× C++.
- This is context, not a work queue. The next checkpoint occurs only after a
  complete dependency wave.

## Gate ledger

No FL-0 commit gate has run yet. Before push, record exact results here for:

- runtime and nuxie library tests;
- ordinary/scripted/probe oracle gates;
- renderer pixels when applicable;
- capi, Apple, lint, formatting, diff check;
- runtime-frame-loop structural checker;
- committed-tree size report.

## Next

1. Run all applicable FL-0 floors from a clean `13aedd6d` worktree carrying
   only the atlas/harness patch.
2. Record the clean floor in a follow-up FL-0 evidence commit and push both.
3. Begin FL-1 dual-translation rulebook validation; do not start FL-A
   production changes before FL-1 closes.
