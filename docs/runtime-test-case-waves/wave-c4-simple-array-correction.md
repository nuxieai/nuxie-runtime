# Wave C4 SimpleArray evidence correction

Status: **CORRECTION CANDIDATE; PENDING FRESH INDEPENDENT REREVIEW**

Original candidate: `adecccd73a9534f5a99669bbd67322c7d79ea386`

Independent rejection: `53064ea2f7358c79b5a3b6edcef769a87e907d79`

Pinned upstream: `4ac7b32798da0482e441ef09304dc3b480ed3ee5`

## Narrow correction

This correction removes only the rejected executable evidence for
`tests/unit_tests/runtime/simple_array_test.cpp#1`. The owner-local Rust test
that read the private backing `Vec` to proxy `size_bytes()` and
`begin() == end()` is deleted in full and is not replaced with a partial test,
capacity calculation, pointer comparison, helper algorithm, or other proxy.

The row is now strict `pending` / `unverified` with empty evidence and no note,
adaptation, or locator. The shard's `max_pending` bound is corrected from 21
to 22 so it states the honest topology instead of preserving the rejected
proof. Removing the test moves only the two retained Span test declarations
in the same module; their evidence line locators are
refreshed from 19 to 7 and from 33 to 21. Their bodies and all other Wave C4
tests and ledger semantics are byte-for-byte unchanged.

## Frozen corrected topology

The denominator remains exactly 52: SIMD 23, SimpleArray 13, Span three,
RefCnt two, and type conversions 11. The corrected classification is three
direct passes, 27 structured `cxx-language-only` adapted passes, and 22 strict
pending/unverified rows: 30 executable passes and 22 pending. No expected-red
is introduced.

SimpleArray cases 1-13 are now all honest pending because the retained owner
does not expose the full pinned observable streams without backing-container,
capacity, allocation, byte-size, or pointer proxies.

## Validation

- Focused non-incremental sweep: the remaining 30 Wave C4 executable tests
  pass; zero fail or are ignored.
- Established isolated corrected-shard validation: all 52 identities validate
  as three direct, 27 adapted, and 22 pending; 30 pass and 22 are unverified.
- Repository correspondence checker: 157 files / 1,404 pinned cases, green.
- Correspondence-checker unit suite: 24/24 green.
- Pinned checkout SHA and all five Wave C4 source SHA-256 identities remain
  exact.
- JSON parsing, strict pending/adaptation shape, evidence resolution,
  forbidden-symbol scan, correction-scoped diff, and diff whitespace checks
  are green.
- Default release `nuxie-binary`, `nuxie-runtime`, and `nuxie-renderer` builds
  are green; release LLVM IR contains no Wave C4 test, expected-red, rejected
  SimpleArray evidence, or pending-helper symbol.

Every relied-on Cargo invocation disabled incremental compilation. This
receipt records a correction candidate only and does not self-accept Wave C4.
