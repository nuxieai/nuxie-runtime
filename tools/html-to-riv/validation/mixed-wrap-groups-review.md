# Geometry-selected line groups reproduce mixed-reversal painting

Both previously failing mixed-reversal profiles now pass their bounded ordinary-file experiment: **16/16 geometry and pixel comparisons**, with **32 passing clear controls**. The cases are `row-reverse + wrap` and `row + wrap-reverse`, each resizing one original and one cloned scene through two, one, three and two lines. This is an experimental file composition, not public compiler support. It targets the pinned Chrome fragment paint ordering documented in `flex-direction-review.md`, not a claim about normative painting in every CSS implementation.

The result closes a specific gap left by the two static paint chains: the emitted file can change which ordered copies are visible when native layout changes line membership. It does not establish general wrapping or unrestricted conditional behavior.

## Construction

Three items have main sizes 60, 60 and 40 px and equal cross size 25 px. Each owns an opaque colored leaf measuring 90×45 px, exposing overlap within and between lines. Parent main size is 50%, cross size 100 px. Viewports 200×200, 320×320, 120×120 and 200×200 alter line membership without recompilation. Exact CSS uses start line and item alignment; the parent navy background remains ordinary native paint.

Native layout receives the true main direction and wrap mode, preserving the order-sorted input stream. The original three leaf paint colors are made transparent, and nine ordinary ForegroundLayoutDrawable copies paint the same responsive leaf paths. Each candidate line group contains one copy of all three leaves. No authored layout parent, size or read-only geometry identity changes.

For potential leader i, a group is active only if i is first or its preceding sibling lies on another line. Within that group, leaf j is visible only if its owner shares i's line. The same-line mask uses the previously demonstrated cross-coordinate difference and exact DistanceConstraint normalization. The leader mask uses the inverted signal with an authored offset and sign appropriate to wrap direction. ClippingShape intersections enforce both conditions on each paint copy.

A fixed DrawRules chain orders potential groups forward for wrap and backward for wrap-reverse. Within groups it orders members backward for row-reverse and forward for row. Unlike selecting one global forward/reverse chain, clipping activates the appropriate groups and membership after each native reflow. The runtime never changes its rule IDs. No browser geometry, media query, host setter, script, binding or animation determines visibility.

## Bounds and accounting

The fixture uses three groups and nine painted copies. Source accounting yields 132 added ordinary records: one origin Node; 22 records for two leader gates; 54 for six distinct member gates; 27 for nine foreground/fill/color triples; 12 clipping records; and 16 records for eight DrawRules/DrawTarget links. This substantial overhead is part of the evidence, not hidden implementation detail or a qualified production resource budget.

The general proposed arrangement grows quadratically in flat item count. These cases have positive 25 px line separation, well above DistanceConstraint's 0.001 cutoff, and common line anchors because all items use start alignment and the same cross size. The earlier tiny/zero gate failures remain applicable. Arbitrary small, zero, transformed or independently aligned line metrics need proof or diagnostics.

Only opaque fills were tested. Their success does not alone prove exactly-once alpha compositing: coincident duplicate opaque copies can be visually indistinguishable. A translucent-paint control must verify mask exclusivity before claiming that property. Nested painted parents, clipping stacks, opacity groups, independent alignment wrappers, column axes and resource ceilings also remain separate work.

## Evidence

Artifacts are in `tools/html-to-riv/output/mixed-wrap-groups-r1/`; tracked hashes are in [mixed-wrap-groups-receipt.json](mixed-wrap-groups-receipt.json).

- The immutable native run passed all 16 geometry/pixel checks and 32 cyan/transparent clear checks. Chrome 153.0.8010.12, rust-metal RasterOrdering. The raw driver's public scope wording is overridden by the experimental manifest.
- All six distinct viewport pairs were directly inspected in `visual-0.png` and `visual-1.png`, including the two-line states that neither static chain could match. Ten return/clone pairs were independently proven identical to reviewed Chrome/native counterparts by full decoded RGBA.
- Both complete adapter executions reproduce the original RIV bytes and parsed source maps exactly. `reproductions.json` records commands, hashes and file sizes.
- Exact source, dependency and augmenter hashes, build command, adapter, fixture sources and native receipt are bound in the manifest. The frozen public compiler base is independently hashed. No runtime or compiler source was changed.

The next tests should challenge alpha exclusivity, unequal line cross sizes and column-axis mapping before combining these gates with general nested subtree paint or public syntax admission. The bounded success is evidence to continue investigating ordinary compositions; it is not a reason to remove existing unsupported-context diagnostics yet.
