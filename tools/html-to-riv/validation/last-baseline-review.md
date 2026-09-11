# Compiler-derived last-baseline alignment

The public lowering derives last-baseline measurements from authored box styles and emits ordinary Rive objects. This extends L03 in horizontal LTR single-line row and row-reverse containers; L03 remains partial.

Empty boxes supply a baseline at their bounded used-height bottom. Normal columns supply the last order-modified descendant's last baseline plus preceding siblings' bounded used heights. Fixed and provable intrinsic heights with fixed minimum/maximum bounds are admitted. A preceding sibling only needs a proven used height; its own baseline can be unresolved. The final descendant must supply a proven last metric. Nested intrinsic columns recurse. Last baselines outside their box, symbolic participant heights, percentage bounds, explicit auto minima and unresolved nested row/reversed-column final baselines still diagnose. Responsive parent dimensions remain ordinary native layout inputs.

The compiler retains separate first and last scalar metrics. Each group gets its own zero-width in-flow extent measurement; the native row takes the maximum alongside normal children. Last groups use a parent-bottom TransformConstraint landmark and an offset Node/TranslationConstraint at minus the maximum descent. Participants follow it at their own baseline origin. This preserves overflow above undersized parents. First-only record emission remains unchanged. Added records are bounded by eight per authored baseline participant (six per last group plus two per participant); no sibling Style environments are retained.

## Validation and preserved limitation

The main 62-scene public corpus passes all 496 geometry frames and 494 pixel frames; all 992 clear controls pass. Four additional nested-constraint scenes pass all 32 geometry/pixel frames and 64 clear controls. The combined result is 528 geometry passes and 526 pixel passes, not a fully passing native corpus. Same original and cloned files resize without recompilation on immutable rust-metal RasterOrdering against pinned Chrome 153.0.8010.12.

The two failures are frames 1 and 5 of `last-public-row-reverse-responsive-parent`, at 390px viewport width and a 292.5px parent. The small `leaf` fails the unchanged local RGB gate. Direct inspection of the enlarged crop shows edge-coverage differences; geometry passes. This is a preserved pixel limitation, not an unsupported layout declaration or proof that all fractional rendering is impossible. Prior fractional failures remain historical evidence; no exact identity or shared root cause is claimed for this new case.

All 70 distinct main-corpus viewport pairs were directly inspected (36 by the review agent and 34 by the root), including the failing case; exact decoded crop/white-extension proofs cover 426 remaining frames. The root inspected all four nested pairs, with 28 exact transfers. Apart from the fractional detail, no visible divergence was found. These reviews do not override metric failures.

The standard freeze tool rejected the failed main receipt, as required. A second checkpoint freezes the same compiler against the passing nested receipt. Its compiler hash matches the main run, and separate exact reproduction proves all 62 main files and source maps unchanged. All 304 prior corpus requests retain identical Rive bytes and maps. This is byte regression evidence, not a fresh rendering claim for those prior scenes.

All 92 Rust and 26 Node tests pass. Native/WASM builds, strict TypeScript and the immutable source guard pass. A stale grammar-rejection test initially failed because `last baseline` is now recognized; the failure log is preserved and the invalid-input case now uses `last baseline extra`. Read-only review identified nested constraint coverage, which was added and passed. No runtime or renderer source changed.
 The broader 99-item objective, unchanged runtime target, preserved failures and fixed tolerances remain in force.
