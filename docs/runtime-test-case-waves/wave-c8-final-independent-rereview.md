# Wave C8 final independent rereview

Verdict: **ACCEPTED**

Corrected candidate: `065d3972720faa88ffde90e24e9c1987b7a13f93`

Original rejection: `487e4e144b17d7b2d6274dd2c8f5f9ede1b25af4`

Pinned upstream: `4ac7b32798da0482e441ef09304dc3b480ed3ee5`

The correction removes and demotes exactly the four rejected proxies: Reader
case 2 and serialized-rendering cases 1, 28, and 35. Each is now strict
pending/unverified with empty evidence and no note, adaptation, or expected-red
reason. The corresponding Rust symbols are absent. All other 58 ledger rows
are frozen except for exact locator-line shifts caused by those deletions.
The only other Rust-source delta is formatter-only import ordering; no retained
test body or production behavior changed.

The final 62-row topology is exact: 22 passing, 13 executable expected-red,
and 27 pending; 32 direct, three structured adaptations, and 27 pending.

Validation:

- focused non-incremental Reader suite: 3 passed;
- focused non-incremental Silver suite: 13 passed and 13 ignored;
- strict identity, ordinal, source-line, source-name, outcome, adaptation,
  pending-shape, locator, and ignore-reason audit: 62/62;
- frozen-row comparison: no non-locator change in the other 58 rows;
- all four pinned source hashes and the upstream checkout SHA match;
- deleted-symbol scan, JSON parsing, scoped diff, and whitespace checks pass.

The historical ratchet reports `max_pending` 27 above the candidate's former
23 floor. That is the expected result of retracting four invalid proofs under
the independent rejection, and is reported separately from the green strict
current-ledger audit. Previously recorded global correspondence and release
containment remain reusable because this correction deletes test evidence and
changes only its correspondence documentation.
