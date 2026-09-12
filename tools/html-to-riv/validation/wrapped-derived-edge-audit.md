# Derived fractional-edge audit

Scope: read-only inspection of the existing `output/playwright/wrapped-derived-r2` campaign. No new render, product edit, runtime edit, or tolerance change. This audits a specific pixel discrepancy; it does not qualify wrapping or prove an immutable-runtime impossibility.

## Observed pixels

The receipt has 384 rows, 48 with pixel failures. All 48 failures belong to responsive-visible cases at step 2 (including original and clone); no equal-size case fails. Geometry failures are empty. This does not imply that passing frames are pixel-identical: the representative responsive step 0 passes the existing gate despite 180 mismatched pixels.

Direct inspection: `derived-row-reverse0-wrap1-level0-responsive-visible/frame-2.{chrome,native,diff}.png`. The red/orange box has matching measured geometry x=0, y=64.5, width=80, height=45. The teal box is x=0, y=119.5, width=40, height=52.5. Differences are thin horizontal edge strips, visibly present in the diff. At x=10:

| y | Chrome RGB | Native RGB |
|---|---|---|
| 64 | 255,255,255 | 255,215,203 |
| 65 | 255,175,151 | 255,175,151 |
| 109 | 255,175,151 | 255,215,203 |
| 110 | 255,255,255 | 255,255,255 |
| 119 | 255,255,255 | 159,229,219 |
| 120 | 63,204,183 | 63,204,184 |
| 171 | 63,204,183 | 63,204,183 |
| 172 | 255,255,255 | 255,255,255 |

The first rectangle's Chrome result is consistent with rounding painted edges from [64.5,109.5] to [65,110], while native gives half coverage at both fractional edges. This is an observation from pinned Chrome 153.0.8010.12, not a claim about every Chromium paint path or a source-proven universal rounding rule. Interior 1-channel differences also exist. Native alpha is 255 throughout these samples.

## Existing serialized controls and source route

`crates/nuxie-runtime/src/mechanical_port/source/layout_component.rs:1542` constructs background bounds directly as `Aabb::new(0.0, 0.0, self.layout.width(), self.layout.height())`, then adds the rounded rectangle and transforms its path. No device-pixel quantization occurs in that paint construction. `:1478` onward sends registered paints through the ordinary path draw route.

`crates/nuxie-runtime/src/mechanical_port/source/shapes/rectangle.rs:47` constructs rectangle vertices from float width, height and origin. For zero corner radii, changing from layout background to ordinary Rectangle alone does not introduce snapping at this path-generation stage.

Generated controls inspected: LayoutComponent clip, width, height, styleId, fractionalWidth and fractionalHeight; Rectangle corner radii/linking; inherited ParametricPath width/height/origins; Shape length; ShapePaint visibility and blend mode. None is a direct serialized pixel-grid paint-snap flag. In particular, `layout_component.rs:3347` reads fractionalWidth/Height as flex grow/shrink weights for Fill scale, not as a pixel rounding switch. Searching runtime source for pixel-snap and pixel-grid names found no direct control; this search is supporting evidence only, not proof that an arbitrary composition cannot implement quantization.

No renderer setting change is proposed: changing antialiasing, sampling mode, or host transforms would change the fixed target and is outside this compiler's contract. Ordinary clip/constraint compositions remain potential experiments. A finite-domain threshold construction may in principle quantize an edge, but its bounded object cost, native updates, device-coordinate dependence, and Chrome semantic match have not been established here.

## Next discriminating standalone experiment

Create a validation-only ordinary scene with a white artboard and three independent translucent paints, no wrapping machinery, gate objects, paint replication or extra masks. Author a red/orange rectangle at (0,64.5), size (80,45), and teal at (0,119.5), size (40,52.5), retaining the same exact ARGB colors and 96x240 viewport. Compare to independently authored CSS absolute boxes with those same literal dimensions. These literals are authored test inputs, not browser-measured geometry fed to the compiler.

Use three matched variants: (A) ordinary LayoutComponent background Fill, (B) Shape + Rectangle + Fill, (C) integer-edge controls at y=65,h=45 and y=120,h=52. Preserve ordinary file import and clones, metadata discarded; capture streams and pixels with the existing baseline tools/gates. Resizing the same files must not recompile them. Extend the same fixture family with authored quarter/half/three-quarter offsets and one responsive percentage edge so any future remedy must track live geometry rather than hard-code this one viewport.

If A and B reproduce the existing edge pixels while C removes the edge error, wrapping placement and mask composition are unnecessary causes of this discrepancy, and ordinary unsnapped fractional paint is the next issue to solve. If A differs from B, investigate the specific paint route. If C still differs materially, isolate alpha/blending or other paint errors before attributing the entire failure to edge snapping. Only after this experiment should we spend effort on a bounded file-level quantization composition or document a narrowly evidenced limitation. The current evidence does not justify rejecting all fractional geometry or silently rounding public authored layouts.

## Content bindings

- `tools/html-to-riv/output/playwright/wrapped-derived-r2/receipt.json`: `1e3b000d8b34b1ba082b9d7f90aa67db3258dfa7668381a56be01771c8dbcfc7`
- `tools/html-to-riv/output/playwright/wrapped-derived-r2/derived-row-reverse0-wrap1-level0-responsive-visible/frame-2.chrome.png`: `f75869720146713ec71b8406123240f861bea5853f43159df2de245a0582b624`
- `tools/html-to-riv/output/playwright/wrapped-derived-r2/derived-row-reverse0-wrap1-level0-responsive-visible/frame-2.native.png`: `bd055a4cd739555ce7fa87998c1581f3c3d63981a51ae618149635df602cdc5b`
- `tools/html-to-riv/output/playwright/wrapped-derived-r2/derived-row-reverse0-wrap1-level0-responsive-visible/frame-2.diff.png`: `3c49cf4c1a4ae4c4f9caa19895ed6ae8f377d17d6772ef061aa6972a505fe371`
- `crates/nuxie-runtime/src/mechanical_port/source/layout_component.rs`: `5154d46f471e2497886b2dbd420d5a0532baf8521f6cfe655a432c60d05122c3`
- `crates/nuxie-runtime/src/mechanical_port/source/shapes/rectangle.rs`: `a7cc0d6c1698593dd3da3df3980fd1ae439cdb51187a5762c9241332df812bc3`
- `crates/nuxie-runtime/src/mechanical_port/source/generated/layout_component_base.rs`: `10911fbe1d164784a5660c6bede729b038506fbff19ad40bbd35cdffdc3011b4`
- `crates/nuxie-runtime/src/mechanical_port/source/generated/shapes/paint/shape_paint_base.rs`: `b60b87122bbb89cd80e9fde90af0324e2655b9749f997a6cb97e81f5b0a854c0`
