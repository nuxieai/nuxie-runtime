# Wave C3 micro final assertion correction

Status: **CORRECTION CANDIDATE; PENDING FRESH INDEPENDENT REREVIEW**

Original candidate: `fae9e184300a8b0fd49ea75787c35de3f81fa296`

Owner/proxy rejection: `82a8d8b39`

Evidence-census correction: `6886bc0ecfe497e988a87a122bd97be11306423b`

Ledger-schema correction: `b40f87e119ca1198ee998b9c7e80ac043f1c410a`

Final assertion-stream rejection: `d5fef6910bbb7876dd78e3fd04dcf0de01be9f78`

Pinned upstream: `4ac7b32798da0482e441ef09304dc3b480ed3ee5`

## Narrow correction

This correction deletes only the two unpinned assertions identified by the final independent rereview:

- lite RTTI case 1 no longer independently asserts `TypeId::<RttiF> != TypeId::<RttiG>`; the `F`/`G`/`H` declarations, shared `Rc<dyn Any>` owner, both downcasts, and pinned success/failure assertions remain in source order;
- malformed-import case 1 no longer requires the full final prefix to succeed; its exact inclusive prefix range, import calls, result match, and drop lifecycle remain unchanged, while case 2 still independently imports the complete fixture and asserts the retained default artboard.

The malformed-import deletion naturally moves the case-2 Rust test declaration from line 36 to line 34. The only ledger change is that evidence locator refresh. No identity, classification, outcome, adaptation, note, ratchet, topology, or other locator changed.

The zero-context test diff contains exactly the removed RTTI assertion and the removed malformed-import assertion (plus the now-unneeded separating blank line). It adds no test statement. The ledger diff is exactly `line: 36` to `line: 34` for malformed-import case 2.

## Frozen topology

The denominator remains 23 cases: lite RTTI 1, malformed import 2, math 18, and Node 2. The corrected shard remains nine adapted passes, one direct pass, and 13 pending/unverified rows. No expected-red, replacement proxy, helper implementation, production behavior change, or test topology change is introduced.

The rejected Wave C3 `MixedInteger` tests, round-up recreation, duplicated count-set-bits fallback test, and Node proxy tests remain absent.

## Validation

- Focused non-incremental sweep: seven math, one lite-RTTI, and two malformed-import tests passed; zero failed or ignored.
- Established isolated corrected-shard content validation: all 23 pinned identities, lines, names, statuses, outcomes, structured adaptations, and ten live evidence locators resolve; nine adapted, one direct, 13 pending; ten pass and 13 unverified.
- The repository-wide correspondence checker passed for 157 files and 1,404 pinned `TEST_CASE` declarations.
- Correspondence-checker unit suite: 24/24 passed.
- Pinned source SHA-256 values remain: lite RTTI `9202813700e3f2680038f009a267d27be51e43057352785f6aa0c71a669d63a7`; malformed import `ce7469a2c48748e6e3052607a896df5d0cf8f28764f3f52a79d244b184e68767`; math `732a98761728f2ff34f63e2395cbf0bab014caea67f1d3db508e03e2660558e6`; Node `ca37dd0710b4bc26895b4453c0cf02fb9dce04f6de6f4655e49ee5ef1509d0a1`.
- JSON parsing, forbidden Wave C3 helper/test-symbol scan, locator resolution, and correction-scoped `git diff --check` passed.
- Default release `nuxie-runtime` LLVM IR contains no Wave C3 test or rejected helper/proxy-test symbol.

The standalone historical-floor invocation still reports the pre-existing mismatch `case max_pending 13 regressed from historical 3`; that mismatch predates and is unaffected by this two-assertion correction. It is reported here explicitly and is not represented as a green ratchet gate. The corrected-shard content validator and repository-wide manifest checker results above are separate.

This receipt records a correction candidate only and does not self-accept Wave C3.
