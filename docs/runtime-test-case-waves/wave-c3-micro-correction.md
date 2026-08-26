# Wave C3 micro correction candidate

Original candidate: `fae9e1843`

Independent rejection: `82a8d8b39`

Pinned upstream: `rive-runtime` `4ac7b32798da0482e441ef09304dc3b480ed3ee5`

## Corrected verdict

Candidate for fresh independent rereview: **10 executable passes, 13 honest pending, 0 expected-red** across the exact 23-case denominator.

The 10 accepted cases are unchanged semantically: lite RTTI #1, malformed import #1-#2, and math #1, #2, #9-#12, and #14.

The rejected test-local mixed-integer implementation and its six tests were removed. The test-local round-up closure and test were removed. The duplicated `count_ones` fallback test was removed because it could not prove the separately named upstream fallback owner. The `InstanceObjectArena` Node proxy module and both tests were removed. Their ten ledger rows now join math #16-#18 as evidence-free pending rows with precise missing-owner reasons. No proxy was replaced with a synthetic red or a production implementation.

## Validation

- Focused non-incremental executable sweep: 7 math, 1 lite-RTTI, and 2 malformed-import tests passed; zero failed or ignored.
- Strict Wave C3 census: 23/23 identities; 10 executable locators, 13 pending, zero expected-red.
- Rejected MixedInteger, round-up, fallback, and Node-proxy test symbols/owners are absent from discoverable test code.
- Pinned source SHA-256 values: lite RTTI `9202813700e3f2680038f009a267d27be51e43057352785f6aa0c71a669d63a7`; malformed import `ce7469a2c48748e6e3052607a896df5d0cf8f28764f3f52a79d244b184e68767`; math `732a98761728f2ff34f63e2395cbf0bab014caea67f1d3db508e03e2660558e6`; Node `ca37dd0710b4bc26895b4453c0cf02fb9dce04f6de6f4655e49ee5ef1509d0a1`.
- Strict JSON parsing and identity/locator validation: passed for all 23 rows.
- Repository correspondence checker: passed for 157 files and 1,404 pinned `TEST_CASE` declarations.
- Correspondence checker unit suite: 24 passed.
- Non-test release LLVM IR: no Wave C3 test or rejected proxy symbols retained.
- Scoped formatting and `git diff --check`: passed.

This correction receipt does not self-accept Wave C3.
