# UNIV-1408 paint-parity diagnosis

Date: 2026-08-03  
Rust revision: `d596a19b` (`origin/main`)  
Pinned C++ revision: `4ac7b32798da0482e441ef09304dc3b480ed3ee5`

## Result

The requested browser-free harness does **not** reproduce the reported border or
badge paint defects on current `origin/main`. It refutes both proposed border
hypotheses:

- H1 is false at the runtime/renderer boundary: the recorded paint contains the
  full authored thickness (`8` and `4`), not half.
- H2 is false for the product's documented object graph: the recorded paths are
  the authored centerline rectangles (`88x56` and `92x60`), not border-box-sized
  rectangles.

The dash and dot paths are also non-empty and have the product-authored fitted
cadence. The absolute badge's fill and label resolve to the same owner origin.
Forcing these tests to fail would encode claims contradicted by the stream, so
the six symptom-labelled tests remain ignored but their parity assertions pass
when run with `--ignored`.

The tight-line-height displacement is real relative to a CSS/DOM expectation,
but it is not a Rust/C++ divergence. Pinned C++ deliberately places the first
line at the font's natural ascent even when `lineHeight` is `1px`; current Rust
does exactly the same. This needs a product-lowering decision if DOM placement
is the contract, not a runtime parity fix.

A separate `LayoutParticipant` port gap is confirmed exactly as described in
the ticket. The product compiler emits no `LayoutParticipant`, so that gap is
unreachable in all six scenes and cannot explain UNIV-1408.

## Harness and method

`crates/nuxie/tests/univ_1408_paint_parity.rs` authors all six scenes through
`Scene`, draws through `RecordingFactory`, and parses `RenderStream`. It uses
`fixtures/command_queue/OpenSans-Italic.ttf` for both text scenes. The dashed
and dotted cases mirror the product compiler's fitted-run float order and final
run correction. In particular, dotted uses the compiler's non-zero `0.25px`
ON run; authoring zero would be an invalid reproduction because the binary
writer omits zero properties and both runtimes reject an all-zero cadence.

`tools/cpp-probe` cannot author or serialize these Scene paint graphs. C++
expectations below are therefore source-derived at the pinned revision, as
allowed by the task. The probe was rebuilt against the pin during bootstrap.

Run the measurements with:

```text
cargo test -p nuxie --test univ_1408_paint_parity -- --ignored --nocapture
```

## Measurements

| Case | Recorded transform | Recorded paint/path | Conclusion |
|---|---|---|---|
| `border_transparent` | `[1,0,0,1,48,32]` | stroke thickness `8`; Butt/Miter; local bounds `[-44,-28]..[44,28]` (`88x56`) | Full authored stroke and centerline path; H1 and H2 refuted. |
| `border_basic` | `[1,0,0,1,48,32]` | stroke thickness `4`; Butt/Miter; local bounds `[-46,-30]..[46,30]` (`92x60`) | Full authored stroke and centerline path; H1 and H2 refuted. |
| `border_dashed` | `[1,0,0,1,48,32]` | thickness `4`; Butt/Miter; 25 ON contours; first centerline run `(-46,-30)..(-40,-30)` | Product's fitted 8px border-box corner dash becomes a 6px centerline reach after the 2px overhang. |
| `border_dotted` | `[1,0,0,1,48,32]` | thickness `4`; Round/Miter; 40 ON contours; first wrapped half-run `(-46,-29.875)..(-46,-30)` | Non-empty product-faithful dots: 12+8+12+8 intervals and an offset of `-0.125`. |
| `text_tight_line_height` | identity | fill glyph bounds `[2.709961,9.272461]..[66.19629,39.27246]` | A single first line has identical Y bounds at line heights `1` and `40`, proving the natural-first-ascent rule. |
| `absolute_badge` | fill `[1,0,0,1,264,0]`; label `[1,0,0,1,204,-8]` | fill local bounds `[-70,-11]..[70,11]`; label local bounds `[0.461914,3.791992]..[59.023636,11.864746]` | Fill origin is `(194,-11)`; subtracting the label's authored `(10,3)` offset gives the same `(194,-11)` owner origin. No space split. |

The tiny `30.000002`/`46.000004` maxima in effected paths are ordinary path
measurement roundoff. No assertion widens a runtime tolerance; the harness
uses a local `1e-4` reporting comparison for decoded stream floats.

## Seam analysis

### Stroke width and alignment

Rust initializes a stroke at
`crates/nuxie-runtime/src/draw.rs:15928-15946`. The decisive operation is line
15937, which passes the authored `thickness` unchanged to `RenderPaint`.
Pinned C++ does the same at `src/shapes/paint/stroke.cpp:12-19`:
`renderPaint->thickness(thickness())`. C++ chooses the local path for
`transformAffectsStroke` at lines 62-68, matching the recorded local rectangle.

The Rust renderer converts full thickness to radius once at
`crates/nuxie-renderer/src/draw.rs:495-497` (`stroke_radius = thickness * 0.5`)
and stores that radius in `PathData` at lines 769-779. That is the correct
diameter-to-radius conversion, not evidence of a second halving.

There is no current Rust/C++ divergence site and therefore no justified runtime
change. The minimum next diagnostic is to preserve one exact failing compiled
artifact and compare (1) its decoded properties, (2) this render stream, and
(3) its final pixels. If its stream matches this harness, the first divergence
is downstream in rasterization or outside the runtime; if it does not, the
artifact's retained mutation/value-rule history is the missing input.

### Dash and dot

Rust applies dash offset and children in authored order at
`crates/nuxie-runtime/src/draw.rs:22077-22101`, then alternates ON/OFF runs in
`dash_path_apply_dash`. Pinned C++ `src/shapes/paint/dash_path.cpp:39-98` has
the same guards and evaluation order: establish that one run is positive,
normalize by the measured path length, clamp a run to the contour, walk the
alternating children, and wrap at the contour end.

No runtime divergence is measured. Dash/dot placement is downstream of path
length because the fitted runs are percentages. A wrong H2-sized path would
move them, but the measured path is `92x60`, so H2 does not explain the current
dash/dot streams. Any remaining DOM pixel mismatch is independent of stroke
thickness transport and must be localized with an end-to-end pixel artifact.

No runtime change is recommended. Changing dash float order or tolerances would
violate the porting prime directive without a first divergent C++ call.

### Tight line height

Current Rust is an exact port at `crates/nuxie-runtime/src/text.rs:2838-2910`.
For explicit line height it preserves the natural baseline ratio at lines
2885-2895, but line zero uses `natural_ascent` at lines 2898-2903.

Pinned C++ `src/text/line_breaker.cpp:26-42` computes the adjusted ascent and
descent. Then `GlyphLine::ComputeLineSpacing` at lines 77-88 makes the relevant
ordering explicit: for the first line it assigns `Y = -realAscent` before
storing `line.baseline`; only later lines subtract the adjusted ascent. Pinned
`src/text/text.cpp:644-669` adds that baseline to the render Y position.

Thus Rust and C++ agree, while CSS's 1px line-box placement does not. The
minimal DOM-contract change belongs in the read-only product compiler: add a
font-metric-aware Y compensation (or choose an explicit text-origin/vertical
alignment lowering) while leaving Rive's first-line rule intact. That change
must be measured with the actual product font; this harness intentionally
asserts the rule rather than pretending Open Sans has Arial's metrics.

This seam is independent of borders and badge layout. In particular, the
roughly similar screenshot magnitudes do **not** mean the tight text shift and
the roughly 14px badge fill shift share a root cause: one is a confirmed
baseline semantic, while the direct badge fill and label have identical owner
coordinates.

### Absolute badge fill versus inspected geometry

The nested layout transform is composed at
`crates/nuxie-runtime/src/draw.rs:6475-6544`; ordinary children inherit it at
lines 6420-6450. Pinned C++ composes the layout slot with its parent at
`src/layout_component.cpp:196-246`, and ordinary transform children compose
`parentWorld * local` at `src/transform_component.cpp:63-89`.

Those sites agree in the authored scene. The filled Shape and Text label both
resolve to `(194,-11)` before their own local offsets. Consequently, there is
no exact current Rust site to identify as divergent and no supported runtime
fix to describe. The regression-window layout rework is still a plausible
source of a retained-update-only failure, but the static Scene exercises its
current transform path without a split. The minimum follow-up is the exact
failing compiled artifact plus its value-rule update sequence; geometry
inspection coordinates alone are insufficient to reproduce a paint-only
history bug.

Layout-owned backgrounds are a different path family:
`crates/nuxie-runtime/src/draw.rs:6176-6204` keeps paint local and translates
the clip, mirroring pinned C++ `src/layout_component.cpp:486-525`, which builds
a local `(0,0)..(width,height)` raw path and separately builds its world path
with `m_WorldTransform`. That family should not be conflated with the requested
Shape-owned badge fill.

## Regression-window evidence

`git diff ae81ae0a..42496d5a -- crates/nuxie-runtime/src/draw.rs
crates/nuxie-runtime/src/text.rs` changes only `draw.rs` (`+1065/-177`); text is
unchanged. The stroke initialization and dash application routines have no
hunks in that window. The material changes are layout transform/background
work and the participant branch. Current main still executes the new layout
composition, yet the badge harness aligns, so the window alone is not enough
to attribute any of the six reported pixel diffs.

The tight-line-height behavior already matched pinned C++ before both pointer
endpoints. It cannot be a text-math regression introduced by this pointer roll.

## Separate confirmed port gap: `LayoutParticipant`

Rust `runtime_layout_control_size_for_path` at
`crates/nuxie-runtime/src/draw.rs:8572-8607` returns the participant's solved
bounds at lines 8591-8595 and consequently resizes a descendant parametric
path. Pinned C++ does the opposite at `src/layout_component.cpp:983-1013`: a
child with a `LayoutNodeProvider` is skipped because the layout engine owns its
size.

For a participating Shape, pinned C++ instead measures combined intrinsic
bounds at `src/shapes/shape.cpp:536-558`, calls `updateLayoutScale` from
`Shape::controlSize` at lines 561-579, stores the derived host scale at lines
581-601, and composes slot anchoring plus that scale at lines 623-647. Rust has
no corresponding `host_scale`, `update_layout_scale`, or combined intrinsic
bounds mechanism.

The minimal parity fix is a separate commit which:

1. makes propagation skip layout providers rather than returning their bounds;
2. adds retained participant host scale computed from the Shape's combined
   intrinsic path bounds; and
3. composes resolved slot translation, intrinsic top-left anchoring, authored
   transform, and host scale in the pinned C++ order.

This gap is not reachable here. Walking a border Rectangle reaches Shape and
then Artboard, so Rust returns `None`; the harness confirms the path is not
resized. The product compiler was also grepped and contains no
`LayoutParticipant` emission. It is independent of all six UNIV-1408 cases.

This is alignable C++ behavior, not a legitimate deliberate divergence. D3 in
`docs/parity-gap-register.md` and the `layout-engine` ceiling in PORTING rule
FLR-20 cover Taffy-versus-Yoga solver edges only; they do not authorize losing
Shape participant host scale. No existing D-row applies. Choosing CSS text
semantics inside the runtime would likewise require a new, explicitly
user-approved D-row/ceiling under FLR-20; product-side compensation avoids that
runtime divergence.

## Fix independence

- Product text compensation can be its own compiler commit.
- Any artifact-proven stroke/raster defect can be fixed independently of text
  and layout after the first divergent stage is captured.
- Dash/dot share source path length with border geometry, but are not downstream
  of a thickness-transport defect; their fitted cadence is otherwise separate.
- A retained badge value-rule/update defect, if reproduced from the real
  artifact, is independent of the tight-line-height rule.
- The participant host-scale port is a separate upstream-parity commit and must
  not be used as a speculative UNIV-1408 fix.

## Addendum (2026-08-04): downstream rasterization is exonerated natively

Follow-up measurements on `origin/main` (`6f6191e4`) after rebasing this
branch:

1. `dump_streams` (new ignored test) writes the six recorded scenes plus a
   filled-border variant as `rive-golden-stream-v1` files under
   `target/univ-1408-streams/`.
2. `renderer-replay --backend rust-wgpu --mode msaa` on `border_basic`
   paints the exact DOM-expected ring: 1216 dark pixels at 1x (96x64 ->
   88x56) and 4864 at a device-pixel-ratio-2 wrapper transform, matching
   the DOM/baseline pixel count from the nightly evidence. The forced
   texture-backed vertex-storage polyfill path and the filled fill+stroke
   variant are also exact.
3. The pointer-roll window `ae81ae0a..42496d5a` touches
   `crates/nuxie-renderer` in exactly one commit (`09440677`), a shader
   regeneration whose only semantic change is alpha-0 dither suppression
   with its own two-mode regression test.

Together with the recording measurements, this exonerates the recorded
stream and its native rasterization for the border family. The remaining
suspects are wasm/browser-only renderer behavior and, more likely, the
retained mutation history of the real product session: every nightly
measurement to date was taken at nuxie-dev pointer `42496d5a`, which
predates the mounted-layout/retained-paint repair family on runtime main
(`6e1eec2a`, `d65b2783`, V29 text-paint mounting, mounted component-list
child transforms). A remeasure of the product visual suite against
`6f6191e4` is the decisive next measurement.

## Final localization (2026-08-04): the border family root cause

The refuted H2 was refuted only for a Shape parented directly to the
Artboard. The product mounts every view under a `LayoutComponent`, and with
that parent the runtime content-sizes the authored centerline rectangle to
the layout's solved bounds:

- `layout_border_rectangle_control_size` (new) authors the product's border
  lowering (`Rectangle 92x60`, `Stroke 4`) under a `LayoutComponent 96x64`
  and records a **96x64** path — `Shape::controlSize` ->
  `ParametricPath::controlSize` overwrote the inset rectangle.
- Replaying that recorded stream through `renderer-replay --mode msaa`
  yields exactly 624 dark CSS px (2,496 at DPR 2) in a 2px ring flush with
  the box edge: byte-for-byte the nightly `border-basic` failure signature
  (DOM/baseline have 4,864; the missing 592 CSS px^2 is the artboard-clipped
  outer half plus hidden inner half of the mis-centered stroke).
- The REAL ProductHost stream (captured natively from the failing
  `border-basic` project snapshot via `ProductHost<RecordingSurface>` at
  runtime `6f6191e4`) contains the same 96x64 stroke rectangle.
- Pinned C++ does the same unconditionally:
  `src/layout_component.cpp:983-1013` (`propagateSizeToChildren` skips only
  nested layouts, layout-transparent containers, and participants),
  `src/shapes/shape.cpp:561-579` (non-participant `Shape::controlSize`
  forwards to the first parametric path), and
  `src/shapes/parametric_path.cpp:24-33` (`controlSize` sets width/height
  to the layout size with no scale-type guard).
- The same probe run at the pre-roll pointer `ae81ae0a` shows the identical
  resize, so the runtime's content-size behavior did not change across the
  roll; what changed is the mounted structure/bounds coverage on the
  product side.

The parity claim is machine-checked, not only source-cited:
`layout_border_rectangle_control_size_matches_cpp_after_exact_riv_round_trip`
(in `crates/nuxie/src/scene.rs`) exports this exact scene to a `.riv`,
draws it through the pinned C++ golden runner, and asserts C++ records the
same content-sized `96x64 x 4` stroke geometry as Rust.

Verdict (border family only): the runtime is C++-parity-correct — proven
at both the pre-roll (`ae81ae0a`) and current (`6f6191e4`) refs; the
border fix belongs in the product mount/lowering (the border Shape must
not be content-sized — e.g. a layout-transparent wrapper — or must author
geometry that survives `controlSize`). A runtime-side skip would be a
deliberate divergence from the pin and is not justified.

## Final localization (2026-08-04): the geometry/paint-split family

Native `ProductHost<RecordingSurface>` captures of the real failing
snapshots (no browser, no GPU) show the displaced backgrounds are already
in the recorded stream while sibling text draws are correct:

- `paywall-limited-time-pill`: label transform x = 37.596 (= pill x 23.596
  + paddingLeft 14, correct); background center x = 127.596 instead of
  150.0 — the fill's left edge lands at 1.19, matching the reported "fill
  pixels start around CSS x=1.5".
- `paywall-pro-tip-card`: card background (362x139.9) world top y = -7.47
  instead of 20; its text children sit at the correct 43.0/62.4.
- `paywall-product-radio-badge`: badge fill top y = 24.98 versus inspected
  y = 11 — the reported ~14 px fill/label split.

So this family is upstream of rasterization: the structural translate
value rules driving background Shapes in the mounted scene resolve
different positions than the (correct) solved layout used by text and by
geometry inspection.

**Status: BISECT PENDING.** "Upstream of rasterization" localizes the
defect to the recorded stream; it does NOT attribute it. The remaining
bisect — product-authored structural-translate rules (nuxie-dev scene
projection) versus runtime transform composition/binding evaluation for
Shape children of mounted layouts — has not been performed. No parity
claim is made for the runtime on this family. The
`univ_1408_border_basic_real_stream_probe` test on nuxie-dev branch
`levi/univ-1408-stream-probe` reproduces the wrong transforms natively
from the captured failing snapshots and is the instrument for that
bisect.
