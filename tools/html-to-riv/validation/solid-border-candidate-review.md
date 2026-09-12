# P01 uniform solid border: ordinary-file candidate

This is a **private composition experiment**, not public CSS admission or P01 qualification. It uses the immutable runtime/renderer/schema baseline `6c7ac16617835b5f581784ff08a9e779bb52faf3`; the runtime identity guard passes. No shared compiler, runtime, renderer, dependencies, root build, SUPPORT or BACKLOG files were changed by this experiment.

A responsive square border can be composed from existing objects. Set the semantic LayoutComponentStyle's four native border widths so ordinary layout reserves border space, and emit four absolute LayoutComponents with Fill/SolidColor paint. A border does not need an opaque inner fill, a clipping owner, a centered stroke, or a renderer adapter. The transparent center, translucent paint, background under the border, content inset and visible content overflow work in the passing fixtures.

The authoring adapter is a finite-fixture executable. It calls the frozen public compiler on the fixture's border-free CSS, decodes and exactly round-trips its ordinary bytes, adds the border fields and objects, and re-encodes before import. Native observation receives only the resulting RIV file. This is a compiler-owned lowering experiment, not a proposal to ship a post-import adapter or CSS sidecar. Its explicit `nativeCss` and border metadata are experiment inputs, not CSS parsing support.

## Exact file composition

For used border width `b`, retain the existing semantic owner and its background, padding and children. Set native `borderLeft/Right/Top/Bottom = b` and each unit to Point. Add these paint-only absolute children with no padding, border, margin, clip, transform or own content:

| Strip | Width / height | Absolute offsets |
|---|---|---|
| Top | Auto / `b` | left `-b`, right `-b`, top `-b` |
| Bottom | Auto / `b` | left `-b`, right `-b`, bottom `-b` |
| Left | `b` / Auto | left `-b`, top `0`, bottom `0` |
| Right | `b` / Auto | right `-b`, top `0`, bottom `0` |

Both size scale types are Fixed; Auto size units allow the two opposed point insets to resolve the long axis. The negative offsets compensate for the owner's native border inset: the tested absolute containing block is its padding box. Top/bottom span the full border-box width, and left/right span the vertical interior between those bands. Their geometric interiors do not overlap. The native border floor keeps an owner's width/height at least `2b`; no negative strip dimension or clamping shim is required in the tested floor cases. The extra padding floor is also supplied by ordinary native layout.

The strips paint after the owner's background and before its authored children. **Emitted subtrees must remain contiguous.** Simply appending every border strip at the end of the file preserves geometry but loses nested border paint. The initial run preserves that failure. The corrected candidate recursively orders the existing parent/style/paint/layout records, places added paint children at the end of their parent's emitted subtree, and remaps `parentId`, `styleId` and the diagnostic source map. It closes its input vocabulary to Backboard, Artboard, LayoutComponent, LayoutComponentStyle, Fill and SolidColor. A general implementation must handle every relevant object reference explicitly; this experiment's two-property remapper cannot be reused for arbitrary record types.

The existing source audit in `immutable-shape-encoding.md` explains the other candidates. A Stroke on the semantic layout path is centered and therefore an outline, not an inside CSS border. Clipping a doubled stroke on a separate decorative sibling/wrapper remains a plausible alternative for further testing; this experiment did not qualify or disprove it. It did not need that alternative for its passing square-border cases.

## Preserved browser/native evidence

The shared driver uses pinned Chrome **153.0.8010.12**, DPR 1, the module's unchanged reset, and the immutable `rust-metal` renderer in effective RasterOrdering mode. The CLI token remains `clockwise-atomic`. Existing geometry and whole-image/local/interior pixel thresholds are unchanged. Every file is compiled once at 390×160, then the same original and a clone run through 240×160 → 390×200 → 768×120 → 240×160. Cyan and transparent canvas clears verify that the white host comes from the ordinary file.

| Run | Cases / frames | Geometry passes | Pixel passes | Clear passes |
|---|---:|---:|---:|---:|
| Initial append-only composition (`render`) | 10 / 80 | 64 | 48 | 160 / 160 |
| Contiguous subtrees and explicit quantized variants (`r2/render`) | 12 / 96 | 80 | 68 | 192 / 192 |
| Minimal fractional-origin controls (`r3/render`) | 3 / 24 | 24 | 12 | 48 / 48 |

The counts intentionally include failures. Driver receipt strings mentioning “public baseline” identify reuse of the existing comparison driver; these files come from the private finite-fixture candidate and do not certify a public API feature.

Eight r2 cases pass geometry and pixels on all eight frames: integer border/background/content; transparent center on a colored ancestor; translucent border over translucent background; visible overflowing content; border floor; border-plus-padding floor; the quantized 2.75px border; and the quantized 0.25px border. These are bounded positive examples, not a proof across arbitrary viewports, depths, color combinations or numeric input domains. The first four use responsive percentage sizes and test the actual original/clone lifecycle. No compiler run occurs between resize steps.

Concrete floor observations: a declared 3×2 box with an 8px border becomes 16×16, with its 2×2 child at (8,8). Adding 3px padding makes the owner 22×22 and the child starts at (11,11). The translucent floor fixture checks that the degenerate zero-height side strips do not duplicate alpha at the two full-width band junctions.

### Fractional CSS border widths

The raw 2.75px and 0.25px native border widths both fail all eight geometry/pixel frames. At DPR 1, independent Chrome computed-style probes report `2.75px → 2px` and `0.25px → 1px`. Their raw native child positions differ by 0.75px; content widths differ by 1.5px. Explicit native widths 2 and 1 restore all eight frames for both fixtures.

`chrome-computed-widths.json` records 13 authored widths from 0 through 9.999. In those observations, zero remains zero; positive widths below one become one; other positive fractional widths truncate to an integer. This is evidence for the pinned DPR-1 profile, not a cross-DPR/device-scale rule or a parser implementation. A production lowering must evaluate the CSS computed length before applying the target profile's used-width quantization, retain the authored/computed distinction for diagnostics, and handle near-integer precision and `em`/`rem`/variables explicitly. Do not pass the authored decimal directly into native layout, silently quantize all layout lengths, or claim these 13 probes prove all syntax and arithmetic cases.

### Fractional paint edges remain unresolved

The fractional-edge scene matches authored geometry on all eight frames but fails every pixel gate. The corrected nested-border scene now displays both borders, passes at the 240px viewport (four frames including clone/repeat), and retains pixel failures at 390/768 where edges become fractional.

A minimal opaque 100×60 border box at (5.25,5.25), with no background or child, matches geometry but fails pixels on all eight frames. At 240×160 it has 624 mismatched pixels (ratio 0.01625). Chrome paints hard pixel edges while the ordinary native paint retains fractional coverage. Its **background-only control also fails** at 240px (320 mismatched pixels), while passing the larger-area gates; the mechanism is therefore not established as a border-only limitation. A translucent-border control passes the configured metrics at all sizes but still shows the expected edge-coverage difference on visual inspection. Do not turn alpha into an admission workaround.

These failures disqualify an unconditional claim of browser-matching fractional border geometry. They do not prove that every ordinary-file composition is impossible. A next experiment should compare the same minimal failures against a clip/doubled-stroke or a single compound-path candidate, inspect the exact coverage change and responsive control of both contours, and retain all current thresholds and controls. Alternatively, a production subset needs an explicit geometry/paint error contract and an admission proof that its accepted scenes remain within it across resizing; three sampled viewports are insufficient.

## Visual review, exact identities and reproducibility

All **48 distinct positive/negative image pairs** were inspected in 16 full-size paired sheets. Each sheet contains the complete unchanged Chrome and native RGBA images at their original resolution, including the three initial nested paint-loss failures. `solid-border-candidate-receipt.py` verifies each displayed crop byte-for-byte against its source image. It binds every one of the 200 frames to a direct review or an exact full-RGBA transfer: **48 direct pairs, 152 transferred frames**. Transfer requires matching fixture name, viewport, exact request hash and both full PNG hashes, followed by actual RGBA equality. Every row retains the request, RIV, source map, native geometry and image hashes; the initial and corrected record identities are not conflated.

The final r3 executable reproduces all 15 final candidate/control RIV files and source maps exactly. Its source, wire writer, frozen library artifacts and build command are hashed in `r3/build-receipt.json`; its seed is the previously frozen source-backed `flex-world-domains-r1` build receipt. All three run receipts retain tool identities and reset/pixel-gate snapshots. Runtime identity was checked separately. No Rust/CLI/WASM/JS public-border parity claim is made; public admission is unchanged and still diagnoses borders.

Durable entry points, relative to `tools/html-to-riv`:

- `validation/solid-border-candidate-emitter.rs`: executable finite-fixture lowering, ordering and remapping logic.
- `validation/solid-border-candidate-build.py`: rebuild into a fresh child of `output/solid-border-candidate-r1` using the frozen compiler/wire/dependencies.
- `validation/solid-border-candidate-cases.json`: all 15 exact browser/native candidate fixtures, including negative controls.
- `validation/solid-border-candidate-computed.mjs`: pinned Chrome computed-width observations.
- `validation/solid-border-candidate-receipt.json`: compact receipt with exact paths/hashes, matrix results, reproduction and visual bindings.
- `output/solid-border-candidate-r1/render/gallery.html`: initial failures.
- `output/solid-border-candidate-r1/r2/render/gallery.html`: corrected main matrix.
- `output/solid-border-candidate-r1/r3/render/gallery.html`: minimal fractional-origin controls.
- `output/solid-border-candidate-r1/visual-coverage.json`: complete visual coverage of all runs.

Rebuild with `python3 tools/html-to-riv/validation/solid-border-candidate-build.py FRESH_NAME`, then pass that candidate executable and the case JSON to the unchanged `validation/check-public-baseline.mjs` driver with a fresh output directory. The build script deliberately refuses an existing source snapshot directory. The receipt script verifies the historical run layout and uses a fresh `reproduction` directory; preserve its existing evidence before a separate rerun.

## Implementation handoff and remaining work

The useful lowering boundary is `emit_uniform_border(owner_id, computed_used_border_width, argb, records) -> paint_helper_ids`, with validated finite nonnegative used width and an explicit paint-order insertion point. It must leave semantic DOM identity on the existing owner, add the four native style edges exactly once, and create absolute decorative children without changing content clipping or overflow. Existing background paint stays on the owner. A direct integration should emit helpers before closing each owner's subtree, avoiding a whole-file remap. If rewriting an already built record graph, reference remapping is a first-class responsibility, including all supported object-reference fields and diagnostic maps.

Each visible nonzero border costs **16 additional ordinary records**: four LayoutComponents, four styles, four fills and four colors. Geometric/layout work and memory scale linearly with border count. The prototype sort also allocates linear maps/vectors and uses recursive traversal; it has not been qualified at the document's object/depth/numeric boundaries. Public integration must account for helper records in resource limits, descriptor extraction and layout analysis, and avoid treating absolute decorative children as semantic flex participants. A zero-alpha border still reserves space; omitting its invisible paint would require a separately tested optimization.

This initial experiment does not qualify per-side borders, radii, transforms, RTL, wrap, grow/shrink, auto margins, alignment wrappers, gaps, intrinsic/auto dimensions, min/max constraints, inherited border declarations, shorthand/cascade parsing, high DPR, nested opacity, clipping or large resource boundaries. Square fixed/percentage F0 boxes with padding/background and the listed overflow/floor examples have concrete positive evidence. Before public admission, implement parsing/diagnostics/provenance, test independent layout interactions and resource boundaries, establish the accepted numeric/pixel contract across the viewport domain, then run Rust/CLI/WASM/JS parity and the full immutable validation workflow.
