# Wave C7 final independent rereview

Verdict: **ACCEPTED — 6/58 executable passes and 52 honest pending owner blockers**

Original candidate: `fa9fc4841c9d9c202bc890adfd07d61325dc5a6d`

Independent rejection: `d6c1813f520e608ea02b1027c04c553eff18bcd7`

Correction candidate: `29a7cefe6`

Pinned upstream: `4ac7b32798da0482e441ef09304dc3b480ed3ee5`

## Correction verdict

The correction follows the rejection exactly. Raw-text-input cases 9, 11, 14,
and 16 no longer claim Unicode-category movement or deletion as evidence for
the pinned font-dependent shaped-glyph owner. Their four Rust proxy tests were
deleted and their rows are strict `pending` / `unverified` entries with empty
evidence and no adaptation or note.

The other 54 rows are frozen: 51 are byte-identical to the original candidate,
and cases 10, 15, and 17 differ only in evidence line numbers shifted by the
four deletions. No production behavior changed.

The accepted topology is six passing executable cases: three direct and three
adapted. There are zero expected-red cases and 52 audited missing-owner cases.
Pending rows are not claimed as executable or as completed parity.

## Fresh gates

- focused non-incremental execution: the four owner-local `wave_c7_` tests and
  the two distinct live-Artboard text-query tests passed, 6/6;
- isolated strict shard: 58/58 identities and locators valid; direct 3,
  adapted 3, pending 52; pass 6, unverified 52;
- historical floor, reported separately as required: expected non-green
  `case max_pending 52 regressed from historical 48`, because 48 was the
  rejected candidate's invalid semantic floor;
- all five pinned source SHA-256 values match the candidate receipt;
- rejected proxy-symbol, JSON, correction/frozen-row, and diff checks passed;
- repository correspondence census: 157 files / 1,404 pinned cases, green;
- correspondence-checker unit suite: 24/24 green;
- fresh default non-test release LLVM IR contains no Wave C7 test or rejected
  proxy symbol.

Wave C7 is accepted only under the corrected workflow's split accounting: six
executable exact ports and 52 pending source-owner blockers.
