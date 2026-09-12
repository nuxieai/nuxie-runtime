# Public physical gaps

The public compiler now accepts gap, row-gap and column-gap with normal, nonnegative px/em/rem/percent/zero, CSS-wide keywords and variables. Font-relative values resolve before computed inheritance. Emission uses existing physical gap fields and changes no runtime, renderer, schema or dependency.

Point gaps work in the tested intrinsic and definite containers. Percentages require a bounded corresponding content dimension when at least two children can use the gap: row uses height, column uses width. Native percent/100 then multiplication and repeated gap addition receive a finite-value guard over the existing viewport bounds. Empty/single-child gaps need no percentage-base calculation. These checks do not prove all aggregate position or raster rounding behavior.

Nonzero gap containers reject padding/distributed spacing, automatic-margin or center/end/baseline child helpers, padded children and placement inside alignment wrappers. Gap-dependent first/last baseline and height summaries remain unknown, so ancestor baseline code cannot reuse summaries that omit gaps. Intrinsic percentage bases remain unqualified and diagnose. These are present compiler restrictions, not claims of runtime impossibility.

Validation:201 Rust tests and36 Node tests pass; WASM and TypeScript checks pass. New transport tests compare point, inherited and percentage gap bytes/maps across three viewports and verify rejected contexts. All482 previous public outputs,79 positive controls and64 private flex/gap files/maps remain exact. Twelve admitted prior native gap scenes reproduce exactly; the four intrinsic-percentage experiment controls now reject publicly, with their original pixel failures retained.

The fresh16-scene public matrix passes128/128 Chrome/native geometry and pixel comparisons and256 clear checks, including original/clone resize. Forty-eight distinct pairs were inspected directly;80 remaining pairs have independent full-RGBA review transfers. The first setup run used unsupported align-items and failed before rendering; the corrected supported per-child alignment fixtures and failed run are preserved. Reverse-overflow siblings outside the viewport have geometry evidence but no visible-paint claim. See public-gap-contexts-review.md and its receipt for complete evidence.

An eight-level amplifying percentage-size control with a1% gap imports/draws/resizes in six original/clone native frames, including16384x16384, with finite geometry. The corresponding1000000% gap rejects before emission. This is exponent-boundary evidence, not huge-coordinate pixel qualification.

Source/artifact bindings are in public-gap-receipt.json. The runtime guard passes. The remaining99-item backlog and unrelated public restrictions remain unchanged; gaps restore a baseline capability outside those incremental item counts.
