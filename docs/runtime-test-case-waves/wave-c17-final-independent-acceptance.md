# Wave C17 final independent acceptance

Author commit: `801cd0e23`

Rejected receipt: `0dbc09b16`

Correction commit: `056e52dec`

Pinned upstream: `4ac7b32798da0482e441ef09304dc3b480ed3ee5`

Verdict: **ACCEPTED — 36/36 cases**

The correction changes no production code, test code, evidence locator,
classification, outcome, or ledger row other than case 15. The other 35 case
rows remain structurally identical to the rejected candidate.

Case 15 now uses the required structured `adaptation` record:

- `kind` is `cxx-language-only`;
- `rationale` accurately explains that pinned C++ exposes enum and `uint32_t`
  overloads while Rust intentionally exposes one live `u32` owner;
- `inapplicable_observable` precisely identifies C++ compile-time overload
  resolution, not any runtime classification behavior.

This is consistent with the previously accepted executable stream: explicit
enum-to-`u32` conversions exercise the live Rust owner for every pinned enum
assertion, and the two raw-`u32` assertions call that same owner directly.

## Gates

- Official strict Wave C17 shard validator: 36/36 green — 35 direct, one
  adapted, 36 pass, zero pending, zero ignored.
- Pinned identity, ordinal, source-line, exact-name, symbol, and evidence
  locators: 36/36 green through the same validator.
- JSON parsing and correction-commit `git diff --check`: green.
- Frozen-row audit: all 35 non-case-15 rows unchanged.
- Correction scope: only `wave-c17.json` and its correction note; no Rust file
  changed.

The prior receipt already accepted the executable semantics 36/36 and ran the
focused non-incremental suite 36/36 green. Because correction commit
`056e52dec` changes no executable source, that semantic review remains valid
and was not reopened.
