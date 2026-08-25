# Wave B2 PointsPath owner correction

Rejected candidate: `997e8fa25a78d9f1c4a68daaaf06449f6112272d`

Fresh rejection receipt: `4b42896ca`

Pinned upstream: `4ac7b32798da0482e441ef09304dc3b480ed3ee5`

Status: **corrected candidate pending fresh independent review**

## Scope

The correction changes only the evidence for
`tests/unit_tests/runtime/file_test.cpp#9`. The other 44 accepted Wave B2 rows
are unchanged except for the four `cpp_probe.rs` line locators shifted when the
rejected proxy test was removed. The same removal shifted two frozen Wave A
locators, which are refreshed without changing their tests. No production
behavior is added or changed.

## Exact owner evidence

The rejected integration test called generic `ArtboardInstance::add_dirt`
with `ComponentDirt::PATH`. Its replacement is a `cfg(test)` test in the
concrete `shapes/points_path.rs` owner. It imports pinned `bad_skin.riv`,
instantiates the live Artboard, performs the first update, and proves the
fixture contains exactly:

- 77 retained `PointsPath` owners;
- eight retained `Skin` owners;
- seven bidirectionally retained PointsPath/Skin relationships; and
- one malformed Skin whose retained Skinnable relationship is absent.

For every PointsPath in source order, the test-only concrete owner records and
executes the pinned action stream: conditional retained Skin dirt first, then
the inherited Path dirt call. It asserts the per-path call order, all 77 Path
calls, all seven Skin calls, Path dirt on every live owner, and a successful
second Artboard update. The old generic integration proxy is deleted.

The only Artboard additions are `cfg(test)` read-only relationship accessors;
they do not exist in production builds. The owner invokes the already-existing
retained Skin and Path dirt operations and does not add runtime behavior.

## Frozen locator refresh

Removing the 30-line rejected integration proxy shifted four later Wave B2
symbols and two Wave A symbols. The Wave B2 locator changes are File #10 and
#11, Elastic #1, and DefaultStateMachine #1. The Wave A locator changes are
Bounds #2 and CubicValue #1. All six resolve uniquely to the same unchanged
test functions; no classification, outcome, action, or assertion changed.

## Validation

- focused exact owner test: green;
- all 31 declared passing Wave B2 rows: green;
- all 14 expected-red rows forced individually: each selected exactly one
  test and failed at its declared concrete seam;
- strict Wave B2 pinned identity/locator/classification validator: 45/45,
  `28 direct / 17 adapted`, `31 pass / 14 expected-red`;
- strict frozen Wave A locator validator: 259/259 after the two explicit
  locator-only refreshes;
- repository correspondence checker: 157 files and 1,404 pinned
  `TEST_CASE`s, green;
- checker unit suite: 24/24 green;
- non-test LLVM IR audit: neither cfg(test) relationship accessor, the fixture
  test, nor its trace type is present in the emitted production library IR;
- scoped `git diff --check` and JSON parsing: green.

This receipt does not declare the wave accepted. The corrected candidate
requires another independent semantic review.
