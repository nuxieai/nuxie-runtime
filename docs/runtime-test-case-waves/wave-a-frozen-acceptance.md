# Wave A frozen final independent acceptance

Reviewed production commit: `2965fb84b2b210bfdb49d43089cc695c7e895fa9`

Reviewed locator commit: `c0ed02757a1dc6d1edca298283b817d7024a3285`

Pinned upstream: `4ac7b32798da0482e441ef09304dc3b480ed3ee5`

Reviewer: independent `runtime_test_wave_a_frozen_acceptance` lane

Verdict: **ACCEPTED**

## Review boundary and rule

The review ran from a clean detached worktree at the locator commit. A direct
row was accepted only when its exact committed test locator resolved and the
test recreated the pinned fixture, actions, production owner, and assertions.
A differential row had to execute the live C++ and Rust authorities. An
expected-red row had to be discoverable with its exact committed ignore reason
and, when forced, execute exactly one test and fail at a concrete retained
behavior boundary. Frozen source, inert action lists, pending helpers,
metadata-only evidence, and recording proxies for a different owner did not
count.

The tracked production tree at the locator commit is byte-identical to
`2965fb84b`; the intervening locator commit changes only three lines in
`wave-a.json` and adds its correction receipt.

## Exact frozen census

All 259 rows match the first 259 pinned C++ cases by path, ordinal, source
line, and exact case name. Every primary and supporting Rust locator resolves
to a discovered test at its committed line and symbol.

- mechanism: 240 direct, four live differential, 15 adapted;
- outcome: 217 pass, 42 executable expected-red;
- adaptation: 13 native-audio and two Rust-safety;
- incomplete: zero pending, unverified, narrow, metadata-only, frozen-source,
  inert-action, pending-helper, or proxy rows.

The two Rust-safety adaptations preserve the tested Bezier observables while
excluding only simultaneous overlapping const/mutable borrows and a
dereferenceable nullable raw pointer. The 13 audio adaptations exclude the
concrete miniaudio engine/source/sound identity and allocator lifetime, not
the tested scheduling, decoding, duration, event, or playback behavior.

## Mechanical gates

- strict Wave A validation: 259/259, with the exact mechanism and outcome
  census above;
- repository correspondence: 157 files and 1,404 pinned `TEST_CASE`s, green;
- correspondence checker unit suite: 24/24 green;
- exact internal-source evidence execution: 67 unique symbols, 63 pass and
  four exact ignored;
- forced expected-red execution: all 42 unique primary tests selected exactly
  one test and failed; none silently passed, skipped, or selected zero tests;
- production freeze: zero tracked production or test differences from
  `2965fb84b`.

## High-risk semantic witnesses

### C++ probe-backed rows

The clean worktree began without a probe executable. The shared audio test
therefore invoked the repository build, built the pinned provenance-bound C++
probe, verified its fingerprint, and passed. That proves the two audio rows do
not skip when the probe is absent.

The other four `cpp_probe.rs` rows also executed with the tools feature:

- RawPath coarse and precise bounds passed;
- cubic and elastic fixture values passed against the freshly built probe;
- both ComponentOrigin tests were discovered with their exact ignore reason;
- forcing each ComponentOrigin test completed its imported-fixture controls
  and failed only at the documented immutable object-arena insertion seam.

### Retained clip world path

The clip row imports `artboardclipping.riv`, selects and advances `Center`, and
reads the retained `m_worldPath`-equivalent owner. Forced execution passed the
initial four points `(0,0)`, `(500,0)`, `(500,500)`, `(0,500)`, then called
`frameOrigin(false)` and updated components. It failed only because that same
retained owner remained unchanged instead of becoming `(-250,-250)`,
`(250,-250)`, `(250,250)`, `(-250,250)`. This is a genuine executable red,
not transform arithmetic performed by the test.

### Component-list composite evidence

The direct supporting assertions for cases 16, 20, and 21 all passed. Case
21's primary Silver replay passed. Forced primary execution for case 16
reached frame 2, operation 384 before the retained stream diverged; case 20
reached frame 6, operation 413 and observed transform `tx` expected `-90`, got
`0`. The supporting and primary locators therefore jointly execute the exact
assertions and retained-stream behavior claimed by each row.

### CDN, dash, and Bezier

Both CDN descriptor tests executed and passed at their corrected committed
locators. The zero-length dash test executed and passed at its corrected
locator.

All four live Bezier differentials passed, respectively executing 1,850, 15,
74,370, and 16,539 assertions against the pinned cases and repository-owned
shader authority. The Rust Bezier family reported ten pass and three ignored;
the one ledger expected-red was also forced and failed.

## Representative execution across all 25 source families

Every evidence-owning integration target was run, in addition to the exact
internal-symbol and all-expected-red sweeps:

- AABB: 12 pass; animation-state: all 14 exact symbols pass;
- artboard transform: five pass, one expected-red discovered;
- audio core: six pass; upstream audio: eight pass, one expected-red; the
  shared probe differential passed;
- binary reader roundtrip: 12 pass; bound bones: one pass;
- bounds: precise/coarse pass plus both expected-reds forced;
- CDN: two pass; child iterator: its expected-red forced;
- clip/instancing and the retained-world-path witness executed as described;
- color glyph: 18 pass, three expected-red; the private shaping owner passed;
- color: both exact symbols pass; command queue: 90 pass;
- component-list direct assertions: 19 pass, one expected-red discovered;
  Silver Wave A: 13 pass, 17 expected-red discovered;
- ComponentOrigin, component, contour, cubic, dash, data-bind container,
  data-bind lists, data-binding artboards, blobs, and computed values all ran
  through their owning targets; every passing locator passed and every red was
  additionally forced to its live failure boundary.

Temporary untracked fixture symlinks used to compile fixture-dependent tests
in the detached worktree were removed before this receipt was written. They
did not alter reviewed source, tests, evidence, or the commit boundary.

## Non-gating exploratory full-library run

For transparency, an additional unscoped `nuxie-runtime --lib` run was not
used as a Wave A gate. It reported 1,173 pass, six ignored, and nine failures
outside the Wave A evidence set: eight pre-existing runtime behavior tests and
one missing external raster-font fixture. The scoped Wave A evidence targets,
including every locator and every forced expected-red, are green under the
acceptance rule above.

Wave A is accepted at the frozen production and locator commits. This review
changes no production or test code and does not promote the separate main
1,404-case ledger.
