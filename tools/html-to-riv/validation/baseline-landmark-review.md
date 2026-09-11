# Ordinary baseline landmark experiments

Historical checkpoint: current first-baseline admission and remaining restrictions are documented in [first-baseline-review.md](first-baseline-review.md). This supersedes earlier blanket baseline-rejection statements below.

Public baseline alignment remains rejected. These fixture-specific experiments establish ordinary-file primitives and preserve counterexamples; they do not add a general compiler lowering. The native importer receives only Rive bytes, while the diagnostic source map is used for read-only geometry joins. Chrome receives the original CSS baseline fixture.

## Landmark and anchor primitive

The experiment first compiles the original topology with baseline participants start-aligned. A schema-based writer round-trips the emitted records exactly, then appends a Node with TransformConstraint targeting the known tallest participant's bottom bounds. Each other participant receives a ComponentOrigin at its bottom and a TranslationConstraint that copies only the landmark's world Y. Original main-axis positions and containing blocks remain intact. The runtime resolves the landmark from live bounds; no host setters or recompilation occur on resize.

Three fixtures cover fixed empty boxes, percentage-height children within a percentage-height parent, and mixed baseline/center alignment. All24 original/clone frames pass geometry and pixels with48 clear controls. All nine distinct viewport pairs were visually reviewed;15 repeat/clone pairs have exact decoded RGBA identity proofs. This succeeds in cases where introducing an intrinsic-height group previously destroyed the percentage containing block or included nonparticipants in the shared baseline.

The experiment uses the frozen public compiler plus an isolated authoring augmentation executable. Its source, schema/library hashes, build command, ordinary emitted files and original/transformed requests are preserved under output/baseline-landmark-r1. All three outputs and maps reproduce exactly. The initial isolated Cargo build failed from insufficient disk space; its new reproducible target cache was removed, logs retained, and direct rustc against existing authoring libraries succeeded. No runtime build or dependency override was introduced.

## Counterexamples and nested control

Three more fixtures pass14/24 geometry/pixel frames with48 clear controls. Every frame1 pair was inspected, including both failing cases; this is not full visual qualification. Source and exact output reproductions are preserved under output/baseline-landmark-boundaries-r1.

- When the parent height grows to160px, its50% child becomes80px and overtakes the selected60px baseline provider. The static provider choice places that child at y-20 instead of0 and leaves the other participants20px too high. Both original and clone fail that viewport. A general composition must select the maximum live baseline distance, not a compile-time winning element.
- For a60px nested flex box whose baseline comes from a10px child, the fixture-specific anchor fraction1/6 positions it correctly against a40px sibling baseline. All eight fixed-parent frames pass. This only proves the tested anchor arithmetic; it does not implement general nested baseline discovery.
- Giving that nested case an automatic-height parent exposes an independent problem: children move to correct positions, but the parent remains60px tall while Chrome requires90px. All eight frames fail parent geometry and background pixels. Post-layout constraints do not automatically repair the parent's layout extent.

## Last-baseline overflow

The independent end-anchored group experiment covers row/row-reverse,40px fixed or automatic parents, and60px tallest children. Unsafe end anchoring passes all four contexts. Safe end anchoring fails the two fixed-parent cases: Chrome places the tallest child at y-20 while safe fallback puts it at y0. Overall48/64 geometry/pixel frames pass;128 clear controls pass. All eight frame1 pairs were visually reviewed. The interrupted ENOSPC run is preserved separately from the completed render-r2 run under output/last-baseline-overflow-r1.

## Next decision

The remaining problem is responsive baseline measurement and its contribution to parent sizing. Investigate an ordinary unpainted layout composition that computes maximum ascent and descent from participating items, retains percentage bases and main-axis order, and supplies both the shared landmark and the parent's intrinsic extent. Constraint-based positioning is a demonstrated primitive, but this measurement composition is unresolved. Do not admit a fixture-selected provider or fixed nested anchor ratio as general baseline support, and do not classify the failed candidates as proof that the immutable runtime cannot express baseline layout.

Evidence bindings and precise visual scopes are in baseline-landmark-receipt.json. Existing pixel tolerances and the baseline runtime/renderer remain unchanged.
