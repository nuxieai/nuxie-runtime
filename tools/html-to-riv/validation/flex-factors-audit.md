# Independent flex factors and basis: immutable-target audit

The unchanged runtime already exposes configurable flex basis on ordinary `LayoutComponent` records. Its native grow and shrink factors are tied to the same fraction. A useful first profile is explicit zero-point basis with authored growth, subject to whole-container interaction guards. This is an implementation proposal supported by source analysis and direct vendor-Taffy arithmetic probes, not Chrome/native file or pixel qualification.

## Source boundary

Audited immutable commit: `6c7ac16617835b5f581784ff08a9e779bb52faf3`. The relevant working-tree runtime/vendor sources match that commit. Evidence and executable arithmetic probes are under `output/flex-factors-audit-r1/`; `receipt.json` hashes sources, harness, lockfile and logs. The standalone harness uses the actual vendored Taffy source and disables rounding. It does not change runtime code, public compiler code, or dependencies of either package.

Two similarly named paths differ:

- `crates/nuxie-runtime/src/mechanical_port/source/layout/layout_participant.rs:215–277`: main Fill maps to grow=fraction, shrink=fraction, basis=0 points. Other scales map to grow=shrink=0 and basis auto.
- `crates/nuxie-runtime/src/mechanical_port/source/layout_component.rs:3227–3371`: the ordinary `LayoutComponent` used by this compiler reads `flexBasis` and `flexBasisUnitsValue` from its style. Main Fill maps to grow=fraction, shrink=fraction, **that configured basis**. Fixed/Hug map to grow=shrink=0, basis auto.
- The component registers itself then its style as appliers (`layout_component.rs:1452–1453`). The style's base pass sets display/min/max via `LayoutSizingStyle`; its item pass sets positioning, margins and other item properties, without replacing the component's factors/basis. Four-pass application is at `layout_component.rs:1717`.
- The generated style defaults are basis=0, **basis units=3 (Auto)**, not Point. For a genuine zero-basis profile explicitly emit `flexBasis=0`, `flexBasisUnitsValue=1`. Simply switching the main scale to Fill is insufficient for a nonempty item.

Fill also changes the preferred main dimension to Auto. Therefore preserving basis does not automatically preserve an authored main width/height's influence on intrinsic sizing or automatic minimum sizing. Separate qualification is needed for those contexts.

## Implementable first admission

A conservative container profile is nowrap, zero padding/border/gap, definite main extent, and either existing fixed/Hug non-growing/non-shrinking children or flexible children with explicit zero-point basis. Flexible children have finite nonnegative grow; first qualification should use automatic preferred main size and existing explicit zero or proven fixed min/max bounds. Encode the grow in the main fractional field and main scale Fill. Cross-axis sizing remains a separate operation.

For this profile, authored shrink can be arbitrary finite nonnegative:

1. Flexible items have zero inner flex base size, so their scaled shrink contribution is zero.
2. A positive minimum makes hypothetical size positive. In a shrinking line, zero basis is smaller than that hypothetical size, so the item freezes at its minimum before distribution.
3. Other siblings are nonshrinking. Thus replacing the flexible item's shrink by its grow cannot make any positive-basis sibling consume negative space.
4. In a growing line, the growth factor, zero basis and min/max clamps are unchanged. Remaining-space redistribution after a maximum clamp remains native.

This proves a restricted arithmetic mapping, not arbitrary CSS `flex-shrink` support. Preserve the authored computed shrink in the style model for inheritance, shorthand behavior and future admission; do not pretend the stored native fraction is that computed value.

The vendor implements partial growth: `flexbox.rs:1281–1291` scales initial free space when unfrozen grow sum is below one. Factors must not be normalized to sum one. Direct probe: parent 200, fixed sibling 40, zero-basis item grow .25 produces item width 40 and leaves 120 free. With two .25 items and a maximum of 20 on the first, widths are 20 and 40, fixed sibling 40. Max-clamping does not automatically allocate all leftover space to the remaining item.

The direct overflow probe with parent 50, zero-basis item min 10 and fixed sibling 100 gives widths 10 and 100 both for authored shrink 7 and native shrink .2. This is expected nonshrinking overflow, not a failed attempt to fit everything.

## Concrete counterexamples and required guards

**Zero scaled shrink does not imply global shrink independence.** Vendor `flexbox.rs:1281–1291` sums *unscaled* shrink factors for the below-one rule; only the later distribution uses scaled factors (`1325–1335`). With parent 50, item A basis 0/grow .2/shrink 1 and item B basis 100/grow .1/shrink .1, direct Taffy gives B width 50. Tying A shrink to .2 gives B width 85. Both executions are in `run-r2.log`. Do not combine arbitrary-shrink zero-basis items with general positive-basis shrinking siblings under this proof.

**Positive basis cannot generally tie independent factors.** Parent 50 and a single basis-100 item with grow 1/shrink 0 should retain width 100; tying shrink to 1 yields 50. Both arithmetic executions are recorded. Equal grow/shrink with configurable basis is a separate promising direct mapping, not a solution for unequal factors.

**Existing flexible spacing helpers compete with real growth.** A parent 200 with one basis-zero grow-1 child and `margin-left:auto` should grow the child to 200 before assigning leftover margin space. Encoding that margin as a sibling Fill weight 1 yields widths 100/100 (recorded arithmetic probe). Around/evenly helpers have the same phase-order issue. Guard the *whole parent* when any growing authored child shares it with any main auto margin or distribution helper, even if the margin is on a different child. Cross auto margins are conceptually independent but wrappers need their own qualification.

**Alignment wrappers move the flex item boundary.** Existing perpendicular cross-alignment wrappers transfer main size/bounds from the authored element to an outer participant. Growth/basis must move to that participant too; placing the fraction only on the inner authored component would grow along a different parent axis. Baseline height summaries likewise cannot continue treating an authored fixed height as the used height when that height is on a flexible main axis. Conservatively diagnose unresolved baseline metrics or qualify a revised summary.

**Indefinite main sizing needs a different proof.** Vendor intrinsic sizing (`flexbox.rs:1023–1025`, `1111–1155`) explicitly inspects shrink zero/nonzero and preferred size, beyond the final scaled-shrink formula. Do not extend the definite-main argument to auto/intrinsic ancestors by analogy. Percentage zero basis also differs from zero points under indefinite sizing. Automatic minimum sizes and specified preferred main dimensions need targeted tests because Fill changes that preferred dimension to Auto.

## Next ordinary-file candidates

1. **Equal factors with basis:** use existing component fraction and style basis/units fields, first with fixed-point basis, automatic preferred main dimension and definite parent extent. Verify import, clones, resize, min/max redistribution, partial factors and paint against Chrome. Auto/percentage basis can follow with their own definite/indefinite matrix. This requires no new runtime field.
2. **Non-growing, non-shrinking explicit basis:** use a fixed outer main size equal to a fixed basis and preserve the authored box/source identity inside it. Distinguish basis from preferred size, min/max, intrinsic contributions and percentage containing blocks; a naive width overwrite is not a general proof.
3. **Independent grow/shrink:** a fixed basis participant plus separate flexible remainder can model some positive-space cases, but paints and descendants need to span both pieces, and shrink/min/max require shared capacity. This is a composition candidate, not yet an implementation recipe.
4. **Two-phase leftover distribution:** a wrapper for the authored flex group plus external margin/distribution pieces may separate growth from leftover space only when the group can compute its natural final extent. Partial factors and max clamps make a fixed Fill wrapper insufficient. Geometry expressions or constraints may provide the missing extent; test before admitting.

No item above establishes that broader CSS behavior is impossible on the immutable target. Unsupported combinations should be described as unqualified encodings, retaining these bounded direct mappings and counterexamples.

## Qualification matrix before routing public CSS

Test all four main directions with original/clone grow→shrink→return resizing; grow 0/.25/.5/1/2 and mixed fixed siblings; sum below/equal/above one; unequal factors with zero basis; simultaneous min/max freezes; empty and nested painted children; cross auto and explicit cross alignment wrappers; ordered siblings; bounded percentage descendants; inherited/reset/shorthand values; definite main ancestors versus explicitly rejected intrinsic cases. Maintain source-map identity and no-flex byte equality. Arithmetic probes here establish neither native importer behavior nor Chrome geometry/pixel parity; those claims require the pinned independent driver and visual workflow.
