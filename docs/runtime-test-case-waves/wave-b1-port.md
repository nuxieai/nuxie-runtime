# Runtime test Wave B1 receipt

Wave B1 covers the first seven pinned upstream runtime test files, from
`data_binding_converters_test.cpp` through `data_binding_viewmodels_test.cpp`,
at upstream commit `4ac7b32798da0482e441ef09304dc3b480ed3ee5`.

## Census

- Expected upstream cases: 70
- Executable Rust owner-flow cases: 70
- Passing: 51
- Expected-red: 19
- Pending or proxy-only: 0
- Production runtime files changed by this wave: 0

The machine-readable case ledger is `wave-b1.json`. Every expected-red case is
an ignored Rust test that executes the pinned fixture setup and action flow,
then fails at a named behavioral mismatch or missing runtime occurrence seam.
No expected-red uses an unconditional panic, raw C++ source string, or proxy
assertion.

## Material parity gaps exposed

- Interpolation/reset, transform, and renderer-stream ordering differ in
  several converter, keyframe, relative-binding, and nested/list flows.
- Three-level nested view-model propagation stops one level short.
- Dynamic listener image binding diverges after exact eager image decoding and
  assignment; layout-image fit does not compose the 7.2 authored user scale
  exactly. The stateful live-image flow is operation-exact and green.
- Pad-string empty output and both two-way precedence directions diverge.
- Counted empty list items do not materialize concrete occurrences, so the
  pinned transition-self swap notification cannot execute.

These are evidence for later production parity work, not fixes made in this
tests-only wave.

## Verification

- Focused ordinary tests: all passing cases green; expected-red cases ignored.
- Forced-red audit: all 19 expected-red cases executed individually and failed
  inside their named test bodies.
- Shard validator: 70 unique identities, exact evidence locators, census
  `51 pass / 19 expected-red / 0 pending`.
- Repository correspondence checker: 157 files and 1,404 pinned `TEST_CASE`s.
- Correspondence checker unit suite: 24 passing tests.
- Rust formatting and `git diff --check`: clean for Wave B1 files.
