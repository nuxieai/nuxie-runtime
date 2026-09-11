# Last-baseline ordinary-file investigation

Status: composition candidate, not public support. The compiler continues to diagnose `last baseline`. The immutable runtime contract and all 99 backlog items remain unchanged.

## Semantics and candidate

The [Flexbox baseline rules](https://www.w3.org/TR/css-flexbox-1/#flex-baselines) derive a normal column's last cross-axis baseline from its endmost child, accounting for ordering. In the current zero-margin/padding profile, a candidate metric is the preceding used-height sum plus that child's last baseline. This is a different summary from first baseline; both summaries must be retained when nested boxes can supply either. The [Box Alignment baseline rules](https://www.w3.org/TR/css-align-3/#baseline-align-self) distinguish first and last baseline groups.

For fixed metrics, measure the group's maximum ascent A and maximum descent D. An in-flow zero-width helper of height A+D preserves parent intrinsic extent. A parent-bottom landmark and an ordinary offset node at -D are a candidate shared last-baseline target. Each participant follows that landmark using its own baseline origin. The candidate must preserve negative overflow when the group exceeds a fixed parent. It must not apply safe-end fallback to an actively participating group.

First and last groups must be measured separately. A shared parent uses their maximum extent, alongside ordinary nonparticipants, rather than adding both group extents. Broader responsive expressions and baselines outside participant boxes need explicit proof and diagnostics until qualified.

## Pinned Chrome semantic evidence

`output/last-baseline-browser-r1/manifest.json` binds 14 Chrome-only cases (both row directions). The pinned run uses Chrome 153.0.8010.12. A preliminary Chrome 152 run is retained separately and excluded from target evidence. CLI artifacts were moved from the browser skill's output directory into compiler-owned output after closing the browser.

With a 100px parent and a 60px nested participant containing 10px and 20px children, the participant's last ascent is 30px and descent 30px. The group's shared baseline is 70px; the nested participant starts at 40px. Giving the final child a nested 5px descendant yields ascent 15px/descent 45px and a shared baseline of 55px. Reordering the children changes their positions while retaining the total last ascent in the empty-child case.

Automatic participant height produces zero descent when all children are empty fixed boxes. A 20px participant with 30px of child content supplies a last baseline outside its own box; this is observed Chrome behavior, not current compiler admission. Mixed first/last groups in the automatic-parent fixture produce 70px parent height, and the following footer begins at 70px. Four key geometry values and footer placement were asserted for each of the 14 cases. These are browser semantic observations, not native visual qualification.

## Native experiment result

`output/last-baseline-landmark-r1` contains an isolated exact-fixture adapter using existing ordinary Rive records. It is testing 30 cases across row/reverse, fixed/automatic/min/max/overflow parent sizing, and empty/nested/ordered children. The completed run passes all 240 geometry/pixel frames and 480 clear controls. All 30 frame-1 pairs were directly reviewed; exact decoded crop/white-extension proofs cover the other 210 frames, including bottom clipping at the shortest viewport. All 30 outputs reproduce exactly, with semantic source-map identity. See last-baseline-experiment-receipt.json for the frozen evidence. The adapter's fixture constants are experimental inputs, not a production lowering strategy. No runtime changes or public source edits are part of this investigation.

## Next implementation step

Replace fixture-selected constants with compiler-derived first/last summaries, preserve existing first-baseline bytes where no last-baseline group exists, and measure the two groups independently. Add public grammar/diagnostic tests, CLI/WASM parity, resource bounds and native coverage of mixed groups, nested ordering, automatic heights and offset parents. Keep unproven responsive and out-of-box metrics diagnostic until separately evidenced. This experiment proves a viable ordinary-record mechanism; it does not implement the language feature.
