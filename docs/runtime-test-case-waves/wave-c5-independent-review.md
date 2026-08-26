# Wave C5 geometry/path independent adversarial review

Verdict: **REJECTED — 12 accepted executable rows, 7 rejected claimed-pass
rows, and 24 honest pending rows across the exact denominator of 43**

Reviewed candidate: `8bedccb1e`

Pinned upstream: `4ac7b32798da0482e441ef09304dc3b480ed3ee5`

## Accepted evidence

Ten claimed-pass rows preserve their pinned owner, action stream, and
assertions: `mat2d_test.cpp#1`, `path_test.cpp#9`, `#10`, and `#12`,
`raw_path_test.cpp#1`, `#2`, and `#5`, `transform_constraint_test.cpp#1`,
`trim_test.cpp#1`, and `wangs_formula_test.cpp#4`.

The three Silver passes select the exact source fixture, artboard, default
state machine, and baseline. The stacked-path and fill-trim cases each execute
one bind, 64 advances, 64 draws, and 63 frame boundaries. The missing-targets
case executes one bind, one zero-time advance, and one draw. No action comes
from a test-local aggregate or synthesized expectation.

`wangs_formula_test.cpp#4` is correctly owner-local. The pinned case itself
uses the file-local quadratic reference, random-Bezier, chop, and evaluation
helpers; it does not call the standalone production Wangs API. The Rust test
preserves the exponent range, segment loop, interval chopping, midpoint,
normal-distance calculation, and tolerance assertion.

Both claimed expected-red rows are accepted:

- `rounded_rect_path_test.cpp#2` calls the live rounded-rectangle owner and,
  when individually forced, fails its first pinned bounds assertion with
  `left: 0.0`, `right: 10.0`. Its ignore reason byte-matches the ledger.
- `path_test.cpp#13` executes the exact 182-action stream (61 advances, 61
  draws, and 60 frame boundaries). When individually forced, the real SRIV
  comparator fails at frame 0, operation 21, feather `paint_id` (expected 8,
  got 5). Its ignore reason byte-matches the ledger.

All 24 pending rows are honest: each has `outcome: unverified`, empty evidence,
no locator, no note, and no adaptation. Their missing retained owner or
observable was confirmed; none is backed by a placeholder panic, proxy, or
aggregate facade in the ledger.

## Rejected rows and correction recipe

1. `mat2d_test.cpp#2` replaces pinned `invert(&out)` plus
   `REQUIRE(invertible)` with `invert_or_identity()` and a separate determinant
   assertion. This can conceal an incorrect `None` result and changes the
   assertion owner. Correct it as **direct** by calling the existing
   `Mat2D::invert()` owner, asserting `Some` in the pinned position, and using
   that returned inverse without the extra determinant assertion.
2. `mat2d_test.cpp#3` executes the safe slice owner twice where pinned C++
   deliberately tests both `(pointer, count)` and `Span` overloads. The numeric
   stream is valid, but the candidate's `direct` classification silently
   counts an unrepresentable overload distinction. Keep the executable test
   and classify it **adapted / rust-safety**, with the literal inapplicable
   observable being independent dispatch through C++'s raw-pointer/count
   overload versus its Span overload.
3. `raw_path_test.cpp#4` replaces production `RawPath::transformInPlace` on
   the direct reference path with test-local point multiplication, then checks
   `add_path` against that synthesized expectation. This is a prohibited local
   algorithm. Keep the row **pending** until the real transform-in-place owner
   is callable; only then can the exact direct-vs-addPath comparison be
   **direct**.
4. `raw_path_test.cpp#7` replaces every pinned `RawPath::Iter` visit with a
   test-local verb/point-count fold. It also turns three ignored helper return
   values into three extra assertions. Keep the row **pending** until the real
   iterator owner is callable, then preserve the iterator visits while leaving
   those three return values unasserted.
5. `rectangles_to_contour_test.cpp#1` omits the first pinned
   `contourCount() == 1` assertion. The following contour helper proves the
   contents of contour zero but would not reject an extra contour. Correct it
   as **direct** by restoring that assertion before the first contour-size and
   point checks.
6. `render_test.cpp#1` creates the state machine before the view model but
   binds only the Artboard. The public API documentation states that an
   already-created machine must be bound separately, matching pinned
   `stateMachine->bindViewModelInstance(vmi)`. The principal bind action is
   therefore absent. Keep the row **pending** unless the wrapper exposes the
   real state-machine bind seam; with that seam, bind the same retained handle
   to the machine in pinned order and classify it **direct**.
7. `stroke_test.cpp#1` uses the immutable graph's named component, type-name
   string, and child list as proxies for pinned live
   `artboard->find<Stroke>()` and `stroke->paint()->is<SolidColor>()`, then
   mutates by local id. Keep the row **pending** until live runtime lookup and
   retained paint-type authority are callable; do not promote the static graph
   projection as an adaptation.

These are evidence/classification failures, not renderer or runtime behavior
bugs. This review makes no production or candidate correction.

## Validation

- exact ledger census: 43 rows; candidate topology 17 direct passes, 2 direct
  expected-red, and 24 pending; 19 distinct current evidence locators (the
  candidate receipt's `20/20` statement is off by one);
- focused non-incremental suites: all candidate pass tests green; Silver sweep
  3 passed / 1 ignored; both expected-red rows independently forced to their
  documented live boundary;
- repository correspondence checker: 157 files / 1,404 pinned cases green;
- correspondence-checker unit suite: 24/24 green;
- pinned checkout at the exact SHA; all 13 source files and case identities
  read from that checkout; raw concatenation of the sorted pinned blobs is
  SHA-256 `d7bfa25cfcf0453e90336ffcc855b9c746367abb3fe2f0727dead14de6ea16b9`
  (the candidate receipt does not document how its different aggregate was
  derived);
- all four exact Silver baseline SHA-256 values were re-read from the pinned
  checkout; JSON parsing, current locator resolution, candidate-scoped
  `git diff --check`, and the frozen three-path candidate diff are green;
- non-incremental release Silver library LLVM IR builds and contains no
  `wave_c5_` test symbol or expected-red string;
- candidate `8bedccb1e` changes only the Wave C5 ledger, candidate receipt, and
  Silver test file; this independent receipt changes no code or ledger.

Wave C5 must not be accepted until all seven rejected rows are corrected or
honestly reclassified and receive a fresh independent rereview.
