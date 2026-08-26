# Wave C8 proxy-evidence correction candidate

Corrected candidate: `0399161813ff2b9ba2481def92eef54475da4311`

Independent rejection: `487e4e144b17d7b2d6274dd2c8f5f9ede1b25af4`

Pinned upstream: `rive-runtime` `4ac7b32798da0482e441ef09304dc3b480ed3ee5`

## Correction

This correction applies the rejection literally and changes no production
behavior:

- Reader case 2 is demoted from adapted/pass to strict pending/unverified.
  Its downstream length-prefixed `read_string` proxy test is deleted.
- Serialized-rendering cases 1, 28, and 35 are demoted from direct
  expected-red to strict pending/unverified. Their hard-coded duration,
  discarded advance-result, and binary64 loop-count proxy tests are deleted.
- All four pending rows have empty evidence and no note, adaptation, locator,
  or expected-red reason. No replacement owner is invented.
- `max_pending` is corrected from 23 to 27.
- Only evidence line numbers mechanically shifted by those deletions are
  refreshed. The other 58 rows and their semantic evidence are frozen.

## Corrected topology

The corrected Wave C8 candidate contains **22 passes, 13 genuine executable
expected reds, and 27 pending rows across 62 exact identities**:

- 32 direct rows;
- three structured Rust-safety/C++-language adaptation rows; and
- 27 strict pending rows with no claimed evidence.

## Validation

- Focused non-incremental Reader execution must discover and pass exactly the
  three surviving tests.
- Focused non-incremental Silver execution must report 13 passes and 13
  ignored expected reds. Every retained red stream and byte-identical ignore
  reason is unchanged.
- Strict identity, ordinal, source-line, source-name, status, outcome,
  adaptation, pending shape, and locator validation must accept all 62 rows
  with the corrected topology above.
- All four pinned source hashes, JSON parsing, exact locator resolution,
  deleted-symbol scan, scoped formatting, and `git diff --check` must pass.
- Previously recorded global correspondence and release containment are reused
  because the correction only removes test evidence and edits its ledger.

This is a correction candidate only and does not self-accept Wave C8.
