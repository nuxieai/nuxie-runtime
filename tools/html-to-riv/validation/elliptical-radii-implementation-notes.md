# P04 elliptical and percentage radii

Status: public compiler emission and checked native occurrence installation implemented through runtime requirements version 24 (`layout-css-corner-radii-v1`). Initial public geometry/pixel lifecycle passes128 frames; all48 initial public views are directly reviewed and the128-frame public lifecycle audit is complete. Module409, JavaScript/native-WASM13, targeted host2 and TypeScript pass. Expanded fractional/composition and full regression qualification are pending. P03 is separately qualified against its frozen toolchain.

## CSS contract

The [CSS Backgrounds specification](https://www.w3.org/TR/css-backgrounds-3/#border-radius)
defines horizontal/vertical pairs per corner. Shorthand lists expand independently
on either side of a slash; a missing vertical list copies the horizontal values.
Percentages resolve against the corresponding border-box dimension. A zero axis
makes a square corner. Adjacent-radius overlap reduces every axis by one shared
factor. Inner radii subtract the corresponding border/padding thickness, clamped
at zero. Negative values are invalid.

## Runtime findings and proposed implementation

`css_clip_path.rs` already has internal elliptical path construction and overlap
reduction, but public helper inputs expand circular scalars into equal axis pairs.
`layout_component.rs::paint_geometry` returns four scalars used by background,
border and clipping paths. Existing Rive corner fields are scalar inputs.

Proposed contract: retain unresolved per-axis length/percentage values in checked
runtime requirements. Resolve after layout against the live border box, then use
one shared resolved geometry for background, border ring/side partitions, image
and overflow clipping. Store this as occurrence state with clone/clear/dirty
handling. Require an explicit host capability; old hosts must reject missing
support. Preserve circular P03 artifact behavior when the new contract is unused.
Do not bake browser measurements into paths or resolve percentages at publish time.

## Validation sequence

Initial16 cases/48 Chrome views cover slash lists, longhand pairs, percentages,
mixed units, zero axes and overlapping radii in both box models. Preserve current
compiler rejection diagnostics before implementation. Then test grammar/cascade,
checked host rejection and lifecycle, native/WASM transport, all128 original/clone
resize frames and real pixels. Expand unequal borders, images/text, clipping and
nested flex compositions before qualification. Math expressions remain tracked
under R02/R03; this work must not accidentally admit unevaluated expressions.

## Fractional clip geometry

The pinned [Chrome153 paint property builder](https://chromium.googlesource.com/chromium/src/+/refs/tags/153.0.8010.12/third_party/blink/renderer/core/paint/paint_property_tree_builder.cc) installs a rounded border clip and a distinct snapped overflow clip for ordinary rounded boxes. Their intersection explains why square controls alone did not establish correct rounded-edge behavior. Captured source and hashes are retained in `output/playwright/html-to-riv/elliptical-clip-chromium-source`. The runtime now represents this intersection; focused evidence and remaining qualification are recorded in `elliptical-radii-dual-clip-receipt.json`.
