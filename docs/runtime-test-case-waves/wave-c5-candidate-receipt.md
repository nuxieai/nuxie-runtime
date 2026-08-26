# Wave C5 geometry/path candidate receipt

Pinned upstream: `4ac7b32798da0482e441ef09304dc3b480ed3ee5`

Scope: 43 cases from `path_test.cpp`, `raw_path_test.cpp`, `rounded_rect_path_test.cpp`, `rectangles_to_contour_test.cpp`, `trim_test.cpp`, `stroke_test.cpp`, `mat2d_test.cpp`, `wangs_formula_test.cpp`, the four single constraint files, and `render_test.cpp`.

## Candidate census

- Direct pass: 17
- Direct expected-red: 2
- Pending: 24
- Adapted/differential/not-applicable: 0
- Total: 43

The denominator remains deliberately conservative. Existing Rust tests were rejected as evidence whenever they bypassed a retained Artboard owner, reconstructed an absent iterator or geometry owner in test code, observed a projection instead of the pinned retained owner, or terminated at a generic missing-owner panic.

## Executable owners

The 17 passing rows exercise live RawPath, Mat2D, RectanglesToContour, Artboard/render, Stroke, Trim, transform-constraint, Silver render, and the pinned case-local quadratic tolerance reference. The three new passing Silver tests replay their complete manifest action streams and compare exact pinned SRIV.

`wangs_formula_test.cpp#4` is direct rather than adapted: the pinned C++ case itself defines `quadratic_pow4` as a case-local reference and tests that reference's tolerance invariant. It does not invoke the standalone production Wangs API. The Rust row preserves the same case-local authority and assertion stream; no missing production Wangs owner is claimed.

## Honest missing authority

The 24 pending rows retain no evidence locator. They cover:

- retained rectangle/ellipse path setup and clipping contour owners;
- Feather render count and the group-effect mid-frame dirt owner;
- RawPath iterator, including the prune row whose existing post-state observer reconstructs that absent iterator in test code;
- Rectangle/ShapePaintPath comparison owners;
- retained production Mat2D decomposition in three constraint cases;
- retained StrokeEffect effectPath rawPath;
- seven standalone Wangs production owners.

Test-local reconstructions, draw projections, and unconditional missing-owner panics are not accepted as parity evidence.

## Expected-red boundaries

- `rounded_rect_path_test.cpp#2` executes the exact live `Path::addRoundedRect` input and fails because the returned path ignores the non-zero AABB origin.
- `path_test.cpp#13` executes the complete pinned feather action stream and fails at the exact frame-0 SRIV paint-owner mismatch.

Both rows are independently forceable; neither uses an unconditional or generic failure.

## Candidate gates

- pinned upstream HEAD: `4ac7b32798da0482e441ef09304dc3b480ed3ee5`;
- strict C5 identity, line, name, outcome, pending shape, and evidence-locator audit: 43/43 green, 19/19 executable locators current;
- exact Silver sweep: 3 pass / 1 ignored expected-red;
- focused live-owner sweeps: all 17 denominator pass rows green (the RawPath file also retains two pending proxy tests outside the executable ledger);
- both expected-red rows individually forced non-incrementally and failed at their declared concrete boundary;
- repository correspondence checker: 157 files / 1,404 pinned cases green;
- correspondence-checker unit suite: 24/24 green;
- pinned source aggregate SHA-256: `8f79cc3389a10d3f0bfd8545dc51eecb30c115b8600067ec8b74542f43b02814`;
- release Silver library IR contains no `wave_c5_` test symbol;
- scoped JSON parse, rustfmt, and `git diff --check`: green.
