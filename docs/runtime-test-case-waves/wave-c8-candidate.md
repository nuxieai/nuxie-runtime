# Wave C8 binary/serialization candidate

Pinned upstream: `rive-runtime` `4ac7b32798da0482e441ef09304dc3b480ed3ee5`

## Exact inventory and census

The denominator is exactly 62 active `TEST_CASE` occurrences:

- `serialized_rendering_test.cpp`: 38
- `signed_content_header_test.cpp`: 16
- `object_stream_test.cpp`: 4
- `reader_test.cpp`: 4

Candidate for fresh independent review: **23 executable passes, 16 individually
forceable expected reds, and 23 honest pending rows**. The strict
classifications are 35 direct, 4 `rust-safety`/`cxx-language-only` adapted,
zero differential, and 23 pending.

For `serialized_rendering_test.cpp`, the complete per-case manifest action
streams support 13 exact passes (ordinals 2-6, 14, 18, 19, 22, 26, 33, 36,
and 37), 16 byte-matching expected reds (1, 7, 9, 10, 12, 13, 15, 17, 20,
21, 27, 28, 32, 34, 35, and 38), and 9 pending cases (8, 11, 16, 23-25,
and 29-31). Every executable occurrence has its own discoverable Rust test;
the replay helper imports the exact fixture, selects the literal artboard and
state machine, executes every manifest action in order, serializes through the
retained renderer, parses both SRIV streams, and performs the exact comparison.
Ordinals 35-37 also preserve the pinned initial cleared-random-provider count
through the retained runtime FIFO/count owner.

Render ordinals 8, 11, and 16 have no executable manifest action stream.
Ordinals 29-31 need retained ViewModel parent/dependent/list mutation
observables not exposed by the render stream. Ordinals 23-25 remain pending
even though their final SRIV streams diverge: each pinned occurrence also has
intermediate `RandomProvider::totalCalls()` assertions, and the whole-case
Silver executor does not expose those checkpoints. Counting only the final
byte mismatch would omit pinned assertions.

`SignedContent` directly owns signed-envelope parsing, borrowed content,
signature, version, and success/failure for header ordinals 1-4, 6, and 7.
Header ordinal 5 remains pending because the fallible Rust parser correctly
rejects truncation but then exposes no owner-derived `isSigned()==true`
observable; inspecting the raw flag byte is not accepted as owner evidence.
ScriptAsset ordinals 8-16 remain pending. The existing test-local
`ScriptAssetProbe` duplicates verification and bytecode storage and is rejected
as a byte-proxy facade.

All four object-stream occurrences remain pending. Rust has no retained
`ObjectStream`/`PODStream` owner; the existing test-local `VecDeque` and native
byte wrappers hand-code the behavior under test and are not evidence.

All four reader occurrences execute through retained `BinaryDataReader` with
their exact literal bytes and ordered success/offset/failure assertions. Their
ledger rows declare the unavoidable safe-reader adaptations: returned values,
owned strings, positions, and overflow state replace C++ raw output pointers,
end pointers, destination allocation, and byte-count return values. The byte
decoder writes all four reads separately; no parameter loop collapses the
pinned stream.

## Hash evidence

Pinned source SHA-256:

- `serialized_rendering_test.cpp`: `acdb2b1e294effc3856251fa9b6c2d8a0f09e37ffb2c934c8e901640f0e2ffaf`
- `signed_content_header_test.cpp`: `6a61831d25ca1ed8f010116d500fa4a41f8ca84298a20f1c26f8f8b7c9cdf0cd`
- `object_stream_test.cpp`: `824f4f25d9cc0165b0319f51bbff5d309542913d759b41b5d787fb9e74864097`
- `reader_test.cpp`: `4cc27a1dd50fd31c87cd3a6b85ef9f7c350e283c90bdef6bb3f1cd3b6a549c6c`

Evidence artifact SHA-256 before receipt creation:

- `crates/nuxie-binary/tests/wave_c8_reader.rs`: `cf6607d4a6b4687fee5242579e3da19c0c0b93eacec9162c63df538a67c6589e`
- `tools/silver-corpus/tests/wave_c8.rs`: `a2987f988767298a8e3863ae3f286f50e183779d5cd3f770f785580360006d6c`
- `docs/runtime-test-case-waves/wave-c8.json`: `feb482b12f4eec4f4c341fc67eee3a4fe021b053ecd6c7a91e48ed787c14cffa`
- retained signed-header evidence: `274887cb4f925e0c3c544954a11b6b95ffd5854f5a8ff1d165aca916b12bf93f`
- checked-in Silver manifest: `e9120465f38423bbb70386e65f7ab558dbc4f34644847bf0f6501546ecc139e1`

## Validation

- focused non-incremental execution: 23/23 passed (13 Silver, 6 signed
  header, 4 reader); final Silver target also reports exactly 16 ignored reds;
- every final expected red was forced separately and failed at its exact named
  SRIV mismatch, never at fixture import, setup, or owner construction;
- strict Wave C8 helper audit: 62/62 identities and locators accepted;
  direct 35, adapted 4, pending 23; pass 23, expected-red 16, unverified 23;
- global correspondence: 157 files and 1,404 pinned declarations accepted;
- correspondence checker unit suite: 24/24 passed;
- JSON parse, pending-row strict schema, exact Rust locator resolution,
  forbidden proxy/helper-symbol scan, scoped formatting, and `git diff
  --check`: passed;
- generated-vs-checked manifest comparison is exact for all 38 C8 rendering
  rows. The repository-wide generator check separately reports two pre-existing
  non-C8 stale rows (`global_variables_test` and
  `global_viewmodels_test-set_instance`); this candidate does not alter them or
  the shared manifest;
- release LLVM IR for `nuxie-binary` and `silver-corpus` contains no Wave C8
  test symbols.

No production behavior or shared Silver fixture/manifest changed. This is
candidate evidence only and does not self-accept Wave C8.
