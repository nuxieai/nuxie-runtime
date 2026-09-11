# Empty-box baseline composition experiment

An ordinary full-width intrinsic-height row group, aligned to the original parent's cross start, can align its empty children at their synthesized bottom baselines by end-aligning them. The group height becomes the tallest child height even when the original parent is taller. This uses existing layout objects and preserves original/clone resizing.

Six exact fixture transformations pass48 geometry and pixel frames plus96 clear controls: row/row-reverse, fixed/automatic parent height, and responsive widths with order. All18 distinct viewport pairs were inspected visually;30 repeat/clone pairs have exact decoded RGBA identity proofs. No visible divergence was found. See empty-baseline-receipt.json and output/empty-baseline-experiment-r1/render/gallery.html.

This is an experiment, not public compiler support. An exact-fixture CLI adapter transforms authored HTML/CSS before invoking the frozen ordinary compiler. Chrome receives the original baseline source. The native importer receives only ordinary Rive bytes. Source-map filtering restores original identities for read-only geometry comparisons; it cannot control rendering. The unmodified driver's public-compiler label must not be interpreted as public baseline qualification. The supplemental receipt records visual review performed after the experimental manifest was frozen.

The fixture pairs and adapter protocol are preserved in output/baseline-group-plan-r1; the executed adapter, copied pairs, transformed requests, hashes and negative adapter checks are in output/empty-baseline-experiment-r1. Public baseline values still reject. All participants in these six fixtures are empty boxes with fixed physical heights. Nested baseline propagation, percentage heights and mixed/noncontiguous alignment groups need independent experiments before a general lowering is selected. A successful bounded group does not establish that it preserves those semantics, and a failed group would not prove no other ordinary composition exists.

## Boundary experiments

Seven further compiled fixtures pass24/56 geometry and pixel frames;32 failures are preserved, with112 clear controls passing. All seven frame1 pairs were inspected, including each failing case. This is failure characterization, not complete visual qualification. See baseline-boundaries-receipt.json and output/empty-baseline-boundaries-r1/FINDINGS.md.

- A parent shorter than its baseline children and the explicit first-baseline keyword pass the tested group composition.
- Last-baseline with the original start-aligned group fails by40px; an end-aligned group control passes the tested empty-box case.
- Mixed center alignment and separated baseline participants fail: including a taller nonparticipant incorrectly changes the shared baseline from40px to60px.
- Nested descendants fail because a flex item's baseline can come from a descendant rather than its own bottom edge. In the preserved example Chrome places the60px box at y30 while the naive group puts it at y0.
- Percentage height does not reach native rendering: introducing the automatic-height group triggers the compiler's existing indefinite-parent diagnostic. This is a rejected composition path, not evidence that ordinary Rive cannot express the original case.

The next candidate should preserve original containing blocks and main-axis slots while positioning baseline participants against a shared landmark. Existing TransformConstraint can target a fraction of object bounds, and TranslationConstraint can copy only Y with an offset; these are source-level possibilities requiring emitted-file and lifecycle experiments. Nested baseline selection and automatic parent extent still need proof. No runtime changes or public baseline admission follow from these findings.
