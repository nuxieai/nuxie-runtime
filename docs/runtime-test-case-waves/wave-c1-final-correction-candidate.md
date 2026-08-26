# Wave C1 final correction candidate

This candidate corrects the 15 rows rejected by the independent receipt at
`23c16de35`. It is test/evidence work only and does not change production
runtime behavior. It does not self-accept the corrections.

## Corrected live-owner evidence

- In-band asset rows 1 and 3 now execute the exact import/fallback flows and
  query the live `RuntimeImageAssetOwners` occurrence. Both are truthful
  expected-red tests: the owner returns decoded RGBA length `Some(4)` where
  pinned `ImageAsset::decodedByteSize` returns the 308-byte source length.
- Layout-participant rows 1, 3, 4, 5, 6, 11, and 15 now assert retained
  post-`Artboard::advance` `ArtboardInstance::layout_bounds` results. The fresh
  diagnostic Taffy solve is gone.
- Solo rows 2 and 7 now resolve the active occurrence from the live
  `activeComponentId` property and the occurrence-owned `cpp_local_ids`
  relation. They no longer infer the active child from collapse state.

## Honest unresolved owner gaps

Four rows are `pending` / `unverified` rather than retaining rejected proxy
evidence:

- layout-grid row 5: no retained Taffy grid-line result owner is callable;
- layout-participant rows 18 and 19: no `Shape::computeIntrinsicBounds` owner
  is callable;
- layout-scroll row 6: no retained two-axis
  `ScrollConstraint::nearestSnapOffsetInDirection` owner is callable.

The complete pinned downstream assertion tables are retained in each row's
note. Rectangle bounds, GridTrack definitions, paint/world-path
reconstruction, snapshot inequality, and the scalar snap helper are not
substituted for those missing owners.

## Candidate census and gates

- 62 total cases: 40 pass, 18 executable expected-red, 4 pending;
- 58 executable rows; 51 direct, 7 adapted, 4 pending;
- corrected retained participant target: 9/9 pass;
- corrected Solo owner target: 2/2 pass;
- in-band rows 1 and 3 were forced individually and fail exactly at
  `Some(4) != Some(308)` after their complete setup/action streams;
- all 40 pass rows execute successfully: 24 unit-owner rows, 15 integration
  rows, and one Silver row;
- all 18 expected-red rows were forced individually with non-incremental Cargo:
  5 unit-owner rows and 13 Silver rows each execute one test and fail at their
  documented concrete boundary;
- all 58 evidence locators resolve exactly after the scoped mechanical refresh;
- strict pinned identity, source line, exact name, classification, pending
  shape, JSON, and locator checks pass for 62/62 rows;
- repository correspondence passes for 157 files / 1,404 pinned cases;
- the correspondence checker unit suite passes 24/24.
