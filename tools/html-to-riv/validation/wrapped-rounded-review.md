# Rounded paint in the actual wrapping composition

The private `compose_with_bounds` construction now owns rounded box paint, its numeric-domain certificate and independent record binding. The 48 existing Derived recipes produce deterministic ordinary Rive files and pass all 384 pinned Chrome/native geometry and pixel frames through original/clone resize cycles. This is private composition evidence; public wrapping remains unadmitted.

## What changed

`paint_box.rs` emits one shared 2,310-record graph per distinct visible geometry: ordinary live corner reads, four selected-axis saturated inputs, four signed rounding graphs, raw-extent thin predicates and four rectangular masks. Saturation is [-16384,16384], while the raw corner difference supplies the thin predicate. Each paint replica is now a root Shape with an ordinary large Rectangle, cloned Fill/SolidColor and the four shared box clips. Existing line leader/member clips and the physical DrawRules chain remain in place. Layout owners are not rounded or replaced.

`wrapping_paint` preflights exact cost and object capacity before emission, caches by geometry, and restores original colors plus appended records on failure. A three-owner/six-replica paint suffix costs 7,073 records. `paint_box_binding` independently reads every new record/field and checks trace agreement; `wrapping_paint_binding` binds the suffix, cloned colors, masks and draw order. `Candidate` owns this trace and records; `Derived` additionally requires the box-domain proof before construction. Its records remain read-only. Sizing helpers retain their separate slot-only dependency policy.

The domain review in `wrapped-rounded-domain-review.md` explains finite raw corner transforms, correlated extent-subtraction error, the thin-predicate boundary, selected-axis saturation and mask coverage over the declared [0,16384]² viewport domain. This does not establish arbitrary transforms, general thin dimensions or Chrome LayoutUnit equivalence.

## Current evidence

- Frozen `wrapped-rounded-product-build-r1`: 419 Rust tests and 56 Node tests; native/WASM builds, TypeScript and immutable source guard pass. Source hashes stayed unchanged throughout. Public CLI/WASM bytes equal `rounding-product-build-r1`; the identity verifier rechecks historical 282 image and 794 transport artifacts without claiming a rerun.
- Frozen `wrapped-rounded-constructor-r1`: all 48 recipes constructed twice with exactly matching base/scene/map/trace/proof artifacts. Candidate SHA-256: `69ba5be02217cc432133ce2fa6b8bc07f23bab55f0f0a152fe5dd773b94740d7`.
- `output/playwright/wrapped-rounded-r1`: 384/384 geometry frames, 384/384 pixel frames, 768/768 cyan/transparent clear comparisons and 240 identical repeated/original-clone pairs. Pinned Chrome is 153.0.8010.12; immutable native rendering is Rust Metal RasterOrdering. Pixel gates are unchanged. Maximum mismatched pixels is zero, but maximum mean channel error is 0.03318142361111111, so Chrome/native RGBA bytes are not claimed identical.
- The observer rechecks 52,992 numerical native scalar/edge/mask values with zero failures, 2,304 independently derived Chrome edge comparisons with zero differences and 384 strict command streams. Those streams contain 11,904 clip calls and 2,688 draws, including artboard backgrounds and replicas hidden by their clips. Five modified-stream negative controls reject. Stream analysis describes recorded commands, not direct GPU-internal state.
- All 144 distinct full-size Chrome/native/diff pairs in 36 sheets were directly inspected in saved root/agent notes, with 240 exact same-case transfers. The notes report matching edges, overlap, gaps and responsive transitions, with faint filled color residuals and no visible missing paint or red mismatch bands. `wrapped-rounded-evidence.py` refuses a success receipt if those notes or any bound artifact are missing.

The native probe receives only the emitted `.riv` path and resize dimensions. Source maps and proof/trace metadata are used by validation afterward; they do not drive import, layout or rendering. The driver asserts finite actual LayoutComponent geometry before comparison, verifies the probe's imported bytes match the constructor output, and resizes the same original and cloned artboard without recompilation.

## Failure preservation and correspondence

The old `wrapped-derived-r2` result remains 384 geometry passes, 336 pixel passes and 48 pixel failures. All 384 current authored requests, Chrome PNG hashes and DOM boxes are exactly equal to their old counterparts. All 48 formerly failing frames now pass with the new ordinary rounded paint. The old native output and failure records remain intact; `before-after-receipt.json` records exact per-frame correspondence. This does not transfer old native stream/layout-record identity to the new graph.

The observer's first setup run used a wrong recipe path and failed before producing observations; its launch log is retained. The corrected observer run produced the recorded results without changing native captures. Early evidence-script runs also stop on missing observer/review artifacts rather than issuing partial success receipts.

## Remaining qualification work

Public parser/planner provenance, diagnostics and transport behavior must exercise this composition before public support changes. Broader overlapping/translucent scenes, zero/thin dimensions, saturation boundaries and browser quantization differences require their own evidence. Roughly 2,310 shared helpers per visible owner is a material cost: practical import/update/render/clone/release and aggregate-budget boundary measurements remain open. The current private full-domain proof rejects uncertain thin predicates and oversized viewport domains; that is a composition boundary, not proof of runtime impossibility.

No runtime, renderer, schema, shared dependency or root build configuration was changed. No editor integration, bindings, host CSS logic, raster fallback or browser-baked layout was introduced.
