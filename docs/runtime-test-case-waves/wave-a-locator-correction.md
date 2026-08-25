# Wave A final locator correction

Rejected receipt: `bb47528a3` (`wave-a-acceptance.md`)

Resolved production commit: `2965fb84b`

Pinned upstream: `4ac7b32798da0482e441ef09304dc3b480ed3ee5`

Verdict: **CORRECTED; PENDING ONE CLEAN FRESH INDEPENDENT ACCEPTANCE**

The rejected review correctly found that evidence lines had drifted after its
reviewed production commit. Production was frozen before this correction, and
all 259 Wave A rows were then re-resolved against the clean committed tree.
All 258 Rust-test locators, including supporting locators, resolve uniquely.

Three primary evidence lines required correction:

- `cdn_asset_test.cpp#1`: hosted-image test `10220` -> `10274`;
- `cdn_asset_test.cpp#2`: hosted-font test `10243` -> `10297`;
- `dash_test.cpp#1`: zero-length-dash test `25511` -> `25526`.

No test body, runtime behavior, proof classification, outcome, or adaptation
changed. The strict clean-commit shard validator reports 259/259 valid rows:
240 direct, four differential, 15 adapted; 217 pass and 42 expected-red.

The repository 1,404-case checker passed, its unit suite passed 24/24, the two
focused CDN tests passed, and the focused dash test passed. This receipt does
not self-certify Wave A or promote the main ledger; one clean fresh independent
acceptance is still required.
