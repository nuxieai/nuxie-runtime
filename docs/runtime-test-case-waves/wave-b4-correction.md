# Wave B4 rejection correction

Status: **CORRECTED CANDIDATE; PENDING FRESH INDEPENDENT REVIEW**

This correction responds to independent rejection receipt `f55de7706` for the
38 pinned cases in upstream runtime files 41 through 45 at
`4ac7b32798da0482e441ef09304dc3b480ed3ee5`. It changes test/evidence
semantics only. It does not declare Wave B4 accepted.

## Corrected rows

- `follow_path_constraint_test.cpp#1-3` now assert that `target` and `rect`
  are transform-capable components, advance the root `ArtboardInstance` at
  zero seconds, decompose the runtime's internal `Mat2D` world transforms, and
  compare the pinned x/y translations. The rejected arbitrary-component
  `update_pass`/raw-matrix integration evidence was removed.
- `font_test.cpp#1-5` now make every observation through cfg(test)
  `RawTextFont` owner methods. Weight/style, line metrics, scaled cap/x height,
  fallback installation and teardown, axis enumeration/defaults/cumulative
  clone state, and feature count/tags are all live. Catch `Approx` uses double
  values, float epsilon, and expected-magnitude scaling; a discriminating
  counterexample rejects the former absolute-only oracle.
- `global_view_model_binding_test.cpp#4/#5` execute the exact Artboard main
  setter prefix and fail at that concrete missing owner. `#6` calls the
  literal state-machine `bind_view_model_instance` owner. `#12` retains the
  pre-bind `RuntimeStateMachineDataContext` and proves pointer identity after
  bind completion.
- `global_viewmodels_test.cpp#1` now creates and sets main, then globals
  `Sizes`, `Colors`, and `Labels` in file order, binds, advances/draws at 0.1,
  and executes exactly 62 `frame / advance(0.016) / draw` iterations.
- `global_viewmodels_test.cpp#3` now stages live main/global handles, preserves
  first-bind main-before-global ordering, applies yellow `0xFFFFFF00`, binds,
  and advances/draws. Its second bind creates a fresh main with
  `"label updated"` and a fresh cyan `GlobalColors.c1` (`0xFF00FFFF`), then
  preserves global-before-main setter ordering before binding and drawing.

The Silver harness additions are explicit command-queue actions over retained
runtime handles. The only runtime exposure is a `feature = "tools"`,
doc-hidden forwarding seam to the existing private main setter; the default
non-test library does not compile that seam. `RawTextFont` variation state is
cfg(test) only.

## Exact census and execution

- 38/38 cases mapped, zero pending;
- classifications: 36 direct, two Rust-safety adaptations;
- outcomes: 33 pass, five expected-red;
- all 33 pass rows executed successfully;
- all five expected-red rows were forced individually and failed at their
  declared owner/SRIV seams:
  - Artboard main setter absent for binding cases 4 and 5;
  - gamepad frame 0 operation 38 (`makeRenderPaint` vs `frameSize`);
  - global variables frame 0 operation 49 (18,540 expected operations vs 49);
  - explicit global instance frame 1 operation 163 (`frame` vs `color`).

## Gates

- strict pinned identity, ordinal, source line, exact name, evidence locator,
  classification, adaptation, and ignore-reason validator: 38/38 green;
- repository correspondence checker: 157 files / 1,404 pinned cases, green;
- correspondence checker unit suite: 24/24 green;
- focused runtime owner, gamepad, and Silver pass suites: green;
- scoped JSON parsing and `git diff --check`: green;
- default, no-tools, non-test LLVM IR contains no `wave_b4`,
  `variation_coords`, or `set_view_model_instance_for_command_queue` symbol.

The 24 rows accepted by the prior review retain their classifications and
semantics; locator-only updates account for line shifts in the corrected owner
files. A fresh independent semantic review is still required.
