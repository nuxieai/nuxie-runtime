# Intrinsic image sizing investigation (L14)

Current status: compiler support and the row-stretch runtime fix are implemented;
qualification remains active. The frozen `intrinsic-image-stretch-toolchain`
passes all 1,152 focused Chrome/native geometry and pixel comparisons across
384 initial, card, flex and edge scenes, with complete audited visual coverage.
Public original/clone resizing covers 3,072 instance-viewports. The full public
suite passes 291 tests in 53 groups; expanded native/WASM parity passes 11 tests,
and Taffy passes 110 tests. No tolerances were widened.

The preserved pre-fix flex reproducer has 24 failures. The fix maps CSS intrinsic
wrappers to replaced items and transfers the known stretched cross dimension to
flex basis and automatic minimum width for single-line rows. Temporary tracing
and the ineffective leaf-sizing experiment were removed.

The pre-fix full regression passed 5,983 tests, including 5,973 exact reviewed
baseline image pairs. It does not qualify the fix. Fixed-runtime full regression
session 44428 remains active. Independent prior aspect-ratio image/image-stress
replays (80 scenes / 240 comparisons) passed with complete audited exact-input/image visual transfers; JavaScript host
and TypeScript checks completed successfully under session 56307. Their output directories are
`intrinsic-image-stretch-prior-images` and `intrinsic-image-stretch-host` beneath
`output/playwright/html-to-riv`. Host and TypeScript commands both exited 0, with the frozen WASM hash verified. Broader vector fallback limitations remain separate.

The sections below retain the investigation history and intermediate evidence;
statements about prior rejection or pending runs describe their recorded stage.

The existing PNG profile remains unchanged: embedded static RGB/RGBA8 PNGs,
no external fetching or new asset formats. Intended layout support includes both
automatic dimensions from decoded size, one authored dimension with the other
automatic, min/max constraints, flex sizing and responsive percentages. Bare
authored aspect ratios and combined auto+ratio must follow replaced-element
semantics; a decoded natural ratio takes priority for auto+ratio. Definite width
and height remain independent. HTML sizing attributes and object-fit extensions
are separate syntax work, not implicitly admitted by this investigation.

Current compiler rejects every32 initial case with unsupported-image-sizing.
The immutable pre-feature inputs and diagnostics are preserved under
`output/playwright/html-to-riv/intrinsic-image-initial-admission/receipt.json`.
Chrome32 scenes/96 views are captured in `intrinsic-image-initial-oracle`.
The cases cross row/column and content/border box with natural size, percentage
width, fixed height, max-width, max-height, bare authored ratio with both axes
auto, auto+ratio, and a height/max-width conflict. The first asset is32x32; add
nonsquare asset coverage before qualification to discriminate dimension and
ratio propagation more strongly.

Local seam findings:

- `src/lib.rs` explicitly rejects automatic image sizing except a bare authored
  ratio and exactly one automatic axis. The image is emitted under a layout
  component with Image fit7 and LayoutParticipant scale controls.
- `src/assets.rs` already validates decoded PNG metadata. Natural dimensions
  must come from the validated asset, not browser-rendered rectangles.
- Runtime `shapes/image.rs::measure_layout` exposes decoded natural width and
  height for non-exact axes, but chooses each independently. That alone does
  not establish browser replaced-element ratio/constraint semantics.
- Any new natural-size/ratio policy must survive occurrence cloning and live
  resize, preserve ordinary Rive behavior and have a capability-gated contract
  if compiler output needs new runtime behavior.

Next: use Chrome geometry to distinguish content dimensions from border boxes,
probe existing runtime intrinsic measurement through public compiled records,
then add the minimum correct runtime/compiler mapping. Keep pre-feature failures.
Do not turn auto dimensions into fixed viewport-specific authoring coordinates.
All public, native/WASM, native-pixel and visual qualification remains pending.


Existing-runtime intrinsic image probe:32 initial scenes now match Chrome for all
256 original/clone instance-viewports after restoring automatic units AND Hug
scale type on automatic axes, setting intrinsic measurement, and selecting the
natural/content-box or authored ratio with version15 exact-pair policy. Initial
units-only experiment left Fixed scale type and failed240/256; it is preserved
but does not establish a valid automatic-size limitation. Probe output and source
snapshots: `intrinsic-image-runtime-investigation/receipt.json`. Public compiler
admission remains unchanged. Next add non-square asset coverage, then implement
the compiler mapping and complete parity/native-pixel/visual gates. No engine
change was needed for this initial32-scene seam test.


L14 non-square/runtime and public mapping:64 wide/tall cases pass512 runtime
clone/resize comparisons. Compiler natural-image admission now passes all96
square/wide/tall scenes (768 instance-viewports), preserving auto dimensions and
using embedded PNG metadata. Intrinsic measurement uses existing Rive fields;
version15 exact-ratio capability carries natural or authored preferred ratio.
Public pre-implementation rejection and green logs are preserved. Full suite
exposed a stack overflow in the depth/resource-limit test because a PNG decoder
local enlarged the recursive compiler frame. Metadata reading moved to the asset
module; the failing resource-limit test now passes without stack-limit changes.
Full module rerun is live (`/tmp/intrinsic-image-full-public-v2.log`); fresh binary
parity/native-pixel/visual gates remain pending. Evidence:
`intrinsic-image-implementation/receipt.json`. The still-running frozen full
L13 regression predates L14 and cannot qualify these compiler changes.


L14 initial native qualification advances: rebuilt/frozen intrinsic-image
publisher and WASM pass all288 native geometry/pixel comparisons (96 square,
wide and tall scenes). All768 public clone/resize instance-viewports pass; full
module288 tests/53 groups and type check pass. Combined native/WASM11 + host15
suite passes26 tests. The native probe/renderer are unchanged from the exact-pair
snapshot. Receipts: intrinsic-image-native/receipt.json and
intrinsic-image-implementation/receipt.json. Visual review remains pending.
32 image/text card compositions were added and Chrome capture started; those
compositions are not included in the passing counts. The full pre-L14 frozen
regression remains independently live in session17229.


L14 card composition stage:32 image/text card scenes pass96 native geometry/pixel
comparisons and256 original/clone instance-viewports. Initial image suite plus
cards now totals128 scenes/384 native comparisons and1024 instance-viewports.
No new compiler/runtime change was needed for cards. Their32 inputs were added
to native/WASM parity, which is running in session44031 (log
`/tmp/intrinsic-image-card-parity.log`). Initial image visual inspection covers
108/288 views after inspecting18 square-asset three-width sheets and auditing
54 direct+54 exact-image transfers;180 initial image views and96 card views still
need review. Thin sampling-edge differences remain within unchanged thresholds.
The full pre-image frozen regression remains live in session17229. No full L14
qualification claim. Receipts: intrinsic-image-native/receipt.json and
intrinsic-image-card-native/receipt.json.
