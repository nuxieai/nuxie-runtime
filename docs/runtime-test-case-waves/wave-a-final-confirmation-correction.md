# Wave A final-confirmation correction

Rejected receipt: `dfe0e9f06` (`wave-a-final-confirmation.md`)

Pinned upstream: `4ac7b32798da0482e441ef09304dc3b480ed3ee5`

Verdict: **CORRECTED; PENDING ONE FRESH INDEPENDENT CONFIRMATION**

This correction changes test evidence, ledger metadata, and ledger validation
only. It changes no runtime production behavior and does not promote the main
1,404-case ledger.

## Metadata blockers closed

- All 17 stale evidence locators now resolve to the current Rust function
  lines: two Bezier tests, two CDN tests, and 13 color-glyph tests.
- The five expected-red strings now exactly equal their evidence tests'
  `#[ignore]` reasons: audio #2, data-bind-container #10,
  data-binding-artboards #8, data-binding-blobs #5, and
  data-binding-computed-values #2.
- Component-list #16, #20, and #21 each have exactly one top-level typed
  evidence locator. That entry retains a schema-validated
  `supporting_rust_tests` locator for the direct assertion body, so the direct
  assertions are not discarded when the Silver replay is the primary test.
  The validator requires every supporting locator to resolve to a discovered,
  non-ignored Rust test.

The strict Wave A validation census is now:

- status: direct 240, differential 4, adapted 15, pending 0;
- outcome: pass 217, expected-red 42, unverified 0;
- total: 259/259 schema-valid rows.

## Six execution blockers closed

The `cpp_probe` integration target now compares retained Artboard inputs by
matching `ScriptArtboardSource::File(id)` to the C++ integer and
`ScriptArtboardSource::Live(_)` to no generated file id. This removes the
invalid enum cast without changing runtime semantics.

The exact six rejected rows are executable:

- `audio_test.cpp#12` and `audio_test.cpp#13`: the shared audio differential
  passed against the fingerprint-checked pinned C++ probe. It no longer returns
  early when the probe is absent; it builds and then requires the pinned probe.
- `bounds_test.cpp#2`: the coarse/precise RawPath bounds port passed.
- `component_origin_test.cpp#1` and `#2`: ordinary execution discovers both
  exact ignore reasons; explicit ignored execution reaches the documented
  immutable-object-arena insertion boundary and fails there.
- `cubic_value_test.cpp#1`: the cubic and elastic fixture assertions passed.

## Composite evidence execution

- Component-list #16 direct scroll assertions passed; its explicit Silver run
  reached the retained stream comparison and failed at frame 2, operation 384.
- Component-list #20 direct initial-position assertions passed; its explicit
  Silver run reached the retained stream comparison and failed at frame 6,
  operation 413, transform `tx` (expected -90, got 0).
- Component-list #21 direct ItemCount/list mutation assertions passed and its
  Silver replay passed.

## Gates

- strict Wave A shard validator: 259/259, zero pending, zero metadata errors;
- repository checker: 157 files and 1,404 pinned `TEST_CASE`s, green;
- checker unit suite: 24/24 green;
- `cpp_probe` tools-feature target: compiles;
- audio #12/#13 shared differential: 1/1 green;
- bounds #2: 1/1 green;
- cubic-value #1: 1/1 green;
- ComponentOrigin discovery: two exact ignored tests;
- ComponentOrigin explicit boundaries: both fail only at their documented
  insertion seams;
- component-list direct supports: 3/3 green;
- component-list primary Silver evidence: #21 green; #16 and #20 fail only at
  their documented retained-stream divergences.

Wave A remains pending one fresh independent confirmation from the corrected
commit.
