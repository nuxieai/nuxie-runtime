# Wave C5 correction independent adversarial rereview

Status: **ACCEPTED**

Original candidate: `8bedccb1ec23f29b71dff9b503166689a6b0669d`

Independent rejection: `54cbd88806cabfbe6fc849e0ae912f1bd23e187c`

Correction reviewed: `ae32ddce9c476161b3e6819bcb6776ab210925f5`

Pinned upstream: `4ac7b32798da0482e441ef09304dc3b480ed3ee5`

## Verdict

The corrected Wave C5 ledger has the exact 43-case denominator and is
accepted. Its final topology is:

- 12 direct passes;
- one `rust-safety` adapted pass;
- two direct genuine expected-red rows;
- 28 honest pending/unverified rows.

All 15 executable rows have distinct discoverable evidence locators. No
pending row has executable evidence, a note, adaptation, or locator. No hidden
red, placeholder failure, test-local owner recreation, or aggregate facade is
counted as proof.

## Seven-row correction audit

- `mat2d_test.cpp#2` now calls the retained `Mat2D::invert()` owner for every
  base matrix, asserts the returned `Option` is present at the pinned
  `REQUIRE(invertible)` position, and stores that exact returned inverse. The
  fallback-to-identity call and unpinned determinant assertion are absent. All
  deterministic matrices, random choices, vector generation, tolerances, and
  ordered scale assertions remain unchanged.
- `mat2d_test.cpp#3` retains its complete matrix, point, AABB, and ordered
  numeric assertion stream. Its structured `rust-safety` adaptation declares
  only independent C++ raw-pointer/count versus `Span` overload dispatch
  inapplicable; both spellings use the same safe retained slice owner in Rust.
- `raw_path_test.cpp#4` and `#7` are strict pending with empty evidence. The
  test-local transform multiplication, iterator fold, and three extra helper
  assertions are not accepted as denominator evidence.
- `rectangles_to_contour_test.cpp#1` restores
  `contour_count() == 1` immediately after the first contour computation and
  before the first contour size/point stream. The reset, second computation,
  second count, sizes, and exact point order remain pinned.
- `render_test.cpp#1` remains strict pending because the existing wrapper does
  not expose the retained state-machine view-model bind seam; Artboard-only
  binding is not substituted.
- `stroke_test.cpp#1` remains strict pending because immutable graph
  name/type/child projections are not live Artboard lookup and retained paint
  type authority.

The correction changes no production behavior, fixture, Silver stream,
baseline, expected-red boundary, or expectation. The other ten accepted pass
rows, 24 previously pending rows, and both expected-red rows remain
semantically unchanged from the independently audited candidate.

## Reaudited unchanged rows

The accepted direct owners still preserve the exact RawPath basic/helper/
bounds streams, live transform-constraint and trim actions, Mat2D map-points
stream, three Silver programs, and case-local Wang quadratic tolerance
reference. In particular, `wangs_formula_test.cpp#4` retains the pinned
exponent range, segment loop, interval chopping, midpoint evaluation,
normal-distance calculation, and tolerance assertion; it does not claim an
absent standalone production Wang owner.

The three passing Silver rows still select the exact pinned fixture, artboard,
default state machine, action stream, and SRIV baseline. Stacked-path and
fill-trim each execute one bind, 64 advances, 64 draws, and 63 frame
boundaries. Missing-targets executes one bind, one zero-time advance, and one
draw. The feather row retains its complete 182-action stream.

All 28 pending rows were reread against their pinned occurrences. Their
missing retained path, iterator, clipping, render, constraint-decomposition,
stroke-effect, or standalone Wang owners remain unavailable; their empty
evidence is honest.

## Genuine expected-red verification

- `rounded_rect_path_test.cpp#2` has an ignore reason byte-identical to its
  ledger reason. Forced individually, its live `Path::addRoundedRect` result
  fails the first pinned bounds check with `left: 0.0`, `right: 10.0`.
- `path_test.cpp#13` has an ignore reason byte-identical to its ledger reason.
  Forced individually through the real SRIV comparator, it fails exactly at
  `frame 0, op 21 (feather), field paint_id: expected 8, got 5`.

Neither red uses an unconditional or generic failure.

## Gates

- Focused non-incremental sweep: all 13 pass rows green.
- Exact Silver sweep: three passed, one declared red ignored.
- Both declared reds forced individually and verified at the exact boundaries
  above.
- Established isolated strict Wave C5 validation: 43/43 identities and 15/15
  executable locators resolve; 14 direct, one adapted, 28 pending; 13 pass,
  two expected-red, and 28 unverified.
- Repository correspondence: 157 files / 1,404 pinned cases, green.
- Correspondence-checker unit suite: 24/24 green.
- Pinned checkout and all 13 source identities are exact and clean. Raw
  concatenation of the sorted pinned blobs is SHA-256
  `d7bfa25cfcf0453e90336ffcc855b9c746367abb3fe2f0727dead14de6ea16b9`.
- The four relied-on SRIV baselines were rehashed from pinned upstream; their
  SHA-256 values are `354b78d75d3363550d1172608d0443b5d6c424cbd6332890f2374b352ff8f7e6`,
  `6db30f32deb4794e78985feffaf2fbdbaabf4c5fc02a04a1b6babf103ee49d25`,
  `9aecbb9b542a19d2d29ee6afc572495a493c990fa8bb0206730c4b1d087029a3`,
  and `5f493ec7ecd79567932cfeea27c88bb96476d6bb785087e3e64561aec9ac97d2`.
- Ledger JSON parsing, topology/schema checks, evidence resolution, correction
  diff whitespace check, proxy-evidence scan, and unchanged Wang/Silver source
  comparison are green.
- Non-incremental release Silver library LLVM IR contains no `wave_c5_` test
  symbol, expected-red reason, or declared red failure string.

Every relied-on Cargo invocation disabled incremental compilation. This
receipt changes no candidate code, ledger row, fixture, baseline, or runtime
behavior.
