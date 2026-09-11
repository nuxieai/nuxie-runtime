# L09 sub-unit flex factors — investigation

Not implemented or qualified. L08 full regression is running with unchanged
publisher/probe/replay/WASM binaries and case corpus; preserve those inputs.

The current compiler accepts only factor0 or [1,10000]. Candidate syntax is
finite nonnegative factors through10000, including fractions below1, in existing
flex shorthand/longhands and cascade. L08's independent-factor occurrence policy
already transports fractional values, while equal factors retain shared Rive
weights. Both paths need geometry and native/WASM coverage. Content-derived auto
basis and indefinite percentage basis remain separate backlog items.

Code diagnosis: `resolve_flexible_lengths` in the vendored layout engine includes
`total_main_axis_gap` in `used_space` before computing `initial_free_space`.
Its grow-sum<1 and shrink-sum<1 branches subtract the same gap again after scaling
initial free space. The original two-child .25/.25 case has220px inner size,
40px total basis and8px gap: initial free space172px. Half is86px, yielding63px
per child; subtracting gap again yields59px. This matches the preserved historical
reproducer, but a fix still needs executed independent browser evidence.

`partial-flex-oracle.mjs` captures144 scenes/432 viewport references: four
directions, six factor pairs, gap0/gap8 and grow/shrink/max-freeze modes. Below-one
factor sums exercise partial allocation; sums above one control the other branch.
Fixed column height makes all three widths useful as cross-axis resize controls;
rows cross available-space thresholds. The frozen-item mode tests redistribution
after max constraints. Additional shrink freezing, mixed item weights, realistic
text compositions and original two-child reproducer should accompany adoption.

Original public rejection cases with gap0 and8 are preserved in
`output/playwright/html-to-riv/partial-flex-initial/`, both exit1 unsupported-value.
Do not remove the deferred-case entry until implementation and validation succeed.

Required gates: isolated engine regression, public compiler/cascade and invalid
factor tests, same-byte original/clone resize geometry, native/WASM byte/map/
requirements parity, checked-host geometry, native/vector pixels, actual visual
review and full regression with unchanged tolerances. No browser-baked layouts.

Browser capture52138 completed:144 scenes/432 viewports, Chromium153.0.8010.12 (`/tmp/partial-flex-oracle.log`). Compact geometry fixture checked in as tests/assets/partial-flex-boxes.json. Isolated engine test20402 failed with638 mismatched coordinates (`/tmp/partial-flex-taffy-initial.log`), preserving the pre-fix evidence. Removing duplicate gap subtraction in both partial grow and shrink branches makes all432 reference viewports pass; all101 isolated engine library tests pass (82215 exit0, `/tmp/partial-flex-taffy-fixed.log`). Compiler admission remains unchanged and sub-unit values are still rejected. No native/WASM publisher rebuild or pixel qualification for L09; all five active L08 full-regression input hashes verified unchanged after isolated testing. Next: expand shrink-freeze/mixed/composition coverage, public compiler admission/cascade/original-clone tests, then builds/parity/pixels only after L08 terminal.

Expanded oracle:240 scenes/720 viewports adds min-constrained shrink freezing and mixed sibling factors/bases. First expansion exposed a fixture specificity error: #root>div overrode #b/#c flex values, producing768 comparison mismatches (`/tmp/partial-flex-taffy-expanded.log`). Preserved incorrect browser capture as partial-flex-oracle-mixed-specificity-error.json. Corrected to #root>#b/#root>#c and added computed-style assertions for basis/grow/shrink before geometry capture. Corrected capture65999 passes; isolated engine54663 passes all101 tests and720 viewports (`/tmp/partial-flex-taffy-expanded2.log`). Three browser text compositions/nine viewports captured68126 and directly inspected, with hashed receipt. Intentional partial-shrink overflow at240 retained for panels/reverse. Prepared244 prospective pixel fixtures (240 boxes,3 compositions,original two-child reproducer) separately from active corpus, and public original/clone oracle test source. Compiler still rejects sub-unit factors; the new public test awaits compiler admission and execution after L08 regression.

Compiler source candidate now accepts finite factors in[0,10000]. Partial values (including equal pairs) emit version10 with layout-css-partial-flex-factors-v1 plus the existing layout factor payload/capability, so older hosts fail rather than applying unfixed distribution. Version10 requires at least one partial factor; lower versions reject partial payloads, and capability/payload inconsistencies fail. Nested ordinary unequal factors use max(version,9), preserving version10. Checked host advertises corrected distribution and supports an environment disable for rejection tests. JS version10/types and public cascade/contract/original-clone tests added. The original partial-fill source is promoted from deferred rejection corpus into the prospective positive corpus; original failing diagnostics remain preserved. Two obsolete fractional-rejection test expectations updated. Source check25462 passes (`/tmp/partial-flex-source-check.log`), types86726 passes (`/tmp/partial-flex-types.log`). Added checked-host test for original two-child63/100.5/195px widths and no-stream rejection for absent capability or invalid contract. Public execution, builds, parity and pixels remain pending L08 full regression; all five active input hashes still match.

Direct artifact execution passes2 cascade/contract tests and the720-viewports original/clone public runtime oracle (`/tmp/partial-flex-contract-tests.log`, `/tmp/partial-flex-oracle-tests.log`). Added numeric-bound/basis-exclusion test. Full module suite9542 runs `/tmp/partial-flex-module.log`. Native and WASM publisher/probe builds started. L08 full regression was invalidated by test-only Cargo unexpectedly rebuilding the publisher; see L08 investigation. New snapshot-toolchain.mjs copies all four executables to a new directory; NUXIE_HTML_TOOLCHAIN in browser.spec verifies hashes and uses only those copies, including WASM. Full restart pending coherent builds/parity and focused validation.

Native66495 and WASM64427 builds pass. Frozen copies created in partial-flex-toolchain with manifest hashes. Host47549 passes11/11, including partial two-child widths and fail-before-stream rejection. Initial snapshot smoke22846 exited1 with no tests because the anchored grep did not account for Playwright project/file prefixes; corrected selector running in a separate output directory, original log retained. Full public9542 still active.

L09 public suite9542 passes236/236 (`/tmp/partial-flex-module.log`). After adding244 pixel fixtures to the main corpus, the two tests that embed the corpus were rerun (55847 exit0, 96 tests; `/tmp/partial-flex-expanded-contract.log`). Native/WASM parity44630 passes9/9 across the expanded corpus. Frozen snapshot smoke84609 passes21/21; all21 source/browser/native image pairs match reviewed baselines exactly and receipt remaining[]. Negative checks confirm hash mismatch aborts before test execution and snapshot overwrite is rejected. Focused native732 run52606 active (`/tmp/partial-flex-native.log`), explicitly using partial-flex-toolchain. Full and vector validation remain pending.

L09 interim native visual review: first90 passed box comparisons captured from completed per-case artifacts while52606 continues. All61 distinct pairs inspected across11 contact sheets;29 exact duplicates covered by PNG hashes. Batch receipt remaining[] applies only to these90 cases, not the complete732 run. See partial-flex-native-review-batch1/visual-inspection.json and partial-flex-box-review-batch1/manifest.json. Remaining images, text compositions, vector and full regression still pending.

L09 interim visual coverage now360/732. Batch2 covers cases91..180:16 new unique pairs inspected,10 within-batch duplicates and64 exact pair matches to reviewed batch1. Batch3 covers all180 reverse-row cases:77 unique pairs inspected across13 sheets,103 exact duplicates. Receipts for each batch have no remaining inspections within their stated scope. Column cases, compositions and final run status remain pending;52606 still active.

L09 focused native52606 passes732/732 (7.5m, `/tmp/partial-flex-native.log`). Column batch4 completed:90 unique pairs inspected across15 sheets,90 exact duplicates, covering180 source cases. All nine text composition comparisons directly reviewed: unused free space, panel sizing and intentional narrow viewport overflow agree with Chromium. Native overall receipt tracks549 reviewed and183 remaining (reverse-column boxes plus original two-child reproducer). Batch5 sheets generated (81 unique pairs/14 sheets), none inspected yet. Vector732 run20863 active (`/tmp/partial-flex-vector.log`), using the same frozen toolchain. Full regression restart remains pending.

L09 focused visual review complete: native732/732 passes and all732 images accounted for. Final batch5 covers183 cases through81 directly inspected pairs and102 exact duplicates; audited all five batches cover723 boxes exactly once, plus9 reviewed compositions. Vector20863 exited1:731/732 pixel passes; all732 geometries pass (maximum0.01251220703125px),723 source/browser/native PNG pairs exactly match reviewed native images, and all9 composition pairs directly inspected. The240px panels text interior RGB error7.90357023690357 exceeds unchanged6 limit; preserved as a vector text raster limitation. Full native4114 regression restarted on frozen partial-flex-toolchain (session94020, /tmp/partial-flex-full.log); qualification remains pending.

Joint L08/L09 full native94020 passes4114/4114 (38.7m). All4104 scene pairs have exact HTML/CSS and browser/native PNG identity with reviewed auto-margin, independent-flex and partial-flex baselines; no images remain unreviewed. Frozen toolchain hashes revalidated. Receipt: partial-flex-full/visual-inspection.json and baseline-comparison.json. L08 and L09 are now native-qualified; their documented vector text raster failures remain. The invalidated earlier L08 run is retained and not used as qualification.
