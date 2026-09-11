# P05 group opacity implementation boundary

Status: investigation; public CSS opacity remains rejected. P04 full native
regression uses an immutable frozen toolchain and is still pending.

## Required behavior

Render the element background, borders and descendants together, then composite
that result once with the element opacity. Nested groups each composite once.
Opacity zero must preserve layout and suppress all group pixels; opacity one
must preserve ordinary output. Unrelated siblings must retain their own alpha.
The initial candidate grammar is finite numeric/percentage opacity with CSS
clamping plus existing CSS-wide cascade semantics. Admission requires the full
runtime path; parsing alone must not imply support.

Opacity groups also participate in stacking. Positioned descendants must not
escape the group's stacking context. Qualification must include negative and
positive z-index descendants and overlap with outside siblings, in addition to
the existing initial shape controls. Six cases now exist in group-opacity-stacking-cases.json, with18 pinned Chrome views and exact overlap samples in group-opacity-stacking-oracle/stacking-samples.json.

## Inspected implementation seams

- `crates/nuxie-render-api/src/lib.rs`: Renderer exposes save/restore, clips and
  per-draw `modulate_opacity`, but no group begin/end operation. Factory exposes
  `make_render_canvas`; RenderCanvas exposes a texture image and begin/finish
  frame. Unsupported factories return an explicit RenderCanvasError.
- `crates/nuxie-renderer/src/native_metal/render_canvas.rs`: canvas retains its
  source image/target and execution guard; frames use a separate renderer.
  This is a candidate reusable allocation mechanism, not proof that nested
  rendering while another frame is active is safe.
- `crates/nuxie-renderer/src/native_metal/mod.rs`: opacity forwards directly to
  the source renderer's modulateOpacity. It does not establish isolation.
- `crates/nuxie-render-stream/src/lib.rs`: typed resources/commands have no
  canvas/group operation. The portable recorder and replay need a matching
  contract; a live-only Metal workaround would leave publish validation broken.
- LayoutComponent currently modulates border colors by inherited render opacity.
  A group policy must prevent that opacity being applied again to child paints.

## Implementation sequence and invariants

1. Prove transparent offscreen rendering and subsequent image compositing using
   existing canvas owners; verify nested frames, resource-domain ownership and
   clip/transform state restoration. Preserve failures if simultaneous frame
   submission needs a renderer change.
2. Define a checked renderer group operation and matching recorded command
   representation. Unsupported backends must reject explicitly. Validate
   balanced groups and finite alpha before execution; do not partially paint
   malformed streams. Specify nesting/resource limits.
3. Composite in device-aligned bounds so an untransformed group does not add
   image resampling. Include descendant overflow in bounds; preserve parent
   clips, internal clips, draw order and nested group alpha independently.
   Verify DPR and transformed groups before claiming those combinations.
4. Install a versioned occurrence policy and group boundaries in runtime draw
   order. Preserve clone/resize/clear semantics and create stacking contexts.
   Separate CSS group opacity from existing Rive inherited paint opacity.
5. Emit the public requirement only after native and portable replay agree;
   add host capability rejection, Rust/CLI/WASM/JS parity and declarations.

## Existing evidence and next tests

`group-opacity-initial-cases.json` has12 prospective shape scenes: opacity
0/0.5/1, overlapping children, nested opacity, rounded clipping and translucent
borders. `output/playwright/html-to-riv/group-opacity-initial-oracle` retains36
pinned Chrome153 views; initial-admission preserves all12 compiler rejections.

`group-opacity-samples.py` validates18 overlap/nested reference frames using
interior red-only, child-overlap, background-only and independent sibling
samples. It rejects the analytic per-draw-alpha overlap result. This gate does
not establish geometry, contours, transformed bounds or native support.

Before qualification add z-index isolation, visible overflow, text/image groups,
nested clips and independent alpha factors. Compile once and replay original
and cloned scenes at240/390/768/240. Inspect browser/native/difference images;
keep targeted overlap samples alongside full metrics. Performance/memory bounds
must be measured for nested groups; do not rasterize every fully opaque box.

Sequential canvas tests now pass for overlapping shapes, nested transparent alpha, parent clipping and restored transform/clip state. See group-opacity-offscreen-progress.json. Active-frame suspension and portable group replay remain unimplemented.

## Replay scheduling constraint

RenderStream::replay_frame currently receives an already-open `&mut dyn Renderer`
and creates resources before executing a flat command list. Since canvas frames
share the source render context, a group implementation must prepare nested
textures before opening the parent frame, or implement explicit renderer frame
suspension. Simply inserting canvas begin/finish calls into that loop is not a
validated design. A preparation pass must prevalidate balanced groups/resources,
resolve bounds and inherited state, render deepest groups first, and retain
textures through final frame completion. Non-group streams must keep their
existing execution path and output.

## Structural stream preparation implemented

The typed and textual stream now recognizes BeginOpacity/EndOpacity. A child-first range planner validates normalized finite alpha, depth64, matching boundaries and isolated save/restore scopes. replay_frame invokes this before resource allocation and rejects groups with UnsupportedOperation until offscreen preparation is implemented. Four new tests plus existing stream tests pass (11 total), including unchanged recording output on invalid and unsupported groups. This does not establish group pixels, bounds/state capture, recording emission or compiler support. Next: capture inherited transform/clip state and prepare textures before opening the destination frame.

## Inherited state plan

Plan now contains child-first Group ranges and persistent StateNode chains of command indices. Each group retains its entry chain; transforms, all clip kinds and per-draw modulation append nodes. Save/restore restores the chain head; group end restores its own entry head. Tests verify nested and sibling isolation and linear state-node allocation for10000 sibling groups. Stream suite13/13 passes. No texture preparation yet. At execution, partition inherited state carefully: ancestors' transforms position offscreen contents, but ancestor clips belong on the final composition to avoid twice antialiasing the same edge. Per-draw modulation and CSS group alpha remain distinct operations; test their interaction before admission.

## Sequential execution implemented

render_frame_to_canvas now loads resources once, prepares each child group in a transparent viewport canvas and composites them into a final owned canvas. It must be called before any destination frame opens. Ancestor transforms are replayed for texture contents; external clips/modulation are deferred to composition. Inverse inherited transforms place prepared images in device coordinates. Every begun frame is finished even after an execution error. One real Metal test passes nested half-alpha overlapping shapes under translation and parent clip with an unaffected sibling. Stream15 tests pass.

This candidate deliberately reports unsupported extents/allocation budgets and noninvertible/nonfinite transforms. Viewport textures are capped at256MiB aggregate RGBA pixels. No CSS admission yet. Next execution coverage: inner clips, images/text, fractional/affine transforms, zero/one alpha, same-stream resize, resource errors and lifetime. Then recorder/runtime group emission and versioned host/compiler transport. Tight bounds and singular transforms remain implementation work, not an external blocker.

## Renderer recording interface

Renderer opt-in begin_opacity_group/end_opacity_group now defaults to unsupported with no side effects. RecordingRenderer emits semantic lines and checks finite normalized alpha, maximum64 depth and matched ends. The portable parser/planner validates full save/restore scopes before execution. Stream16 tests pass. Seven real Metal outer/inner alpha configurations cover zero/half/one and exact repeated half/half output. Runtime group boundaries, native immediate support and compiler/host admission remain open.

## Runtime paint-plan boundaries

PaintTree now carries validated CSS alpha (runtime trees still default to1 until occurrence-policy installation is connected). Alpha below1 creates an atomic stacking context, including alpha0; alpha1 does not. PaintGroup includes begin/end boundary events around the context background and all deferred descendants. Nested groups retain nested boundaries and ancestor clip lists. The existing Artboard executor does not consume these events yet, and production trees cannot produce them yet.

Next integration must atomically install validated layout-target alphas, copy/clear policy with occurrences, pass it through runtime_tree and rebuild the paint plan. Add a checked draw entry point or preflight renderer capability before drawing any background; backend refusal must not silently emit opaque content or partially paint a scene. Existing Rive inherited opacity remains separate. Preserve compiler gating until recording/replay/native host paths all execute the same boundaries.

## Runtime installation connected

Artboard::set_css_group_opacity validates layout targets and normalized finite alpha before replacement. Omitted/alpha1 targets restore normal stacking, clones copy independent policy, and runtime_tree resolves policy by owner. Drawing consumes boundary events. Renderer::opacity_group_capacity preflights maximum depth; Artboard::try_draw_internal_handle returns false before background painting if capacity is insufficient. Existing infallible drawing asserts on this unsupported configuration. RecordingRenderer advertises its remaining64-level capacity; immediate backends remain unsupported.

The integration test passes atomic rejection, nested boundaries, independent clone/clear, repeated resize and unchanged output on capacity refusal. Next: feed runtime-produced recordings into sequential canvas execution against existing Chrome opacity fixtures, then introduce versioned compiler/host transport only after visual and lifecycle evidence.

## Native replay tool connected

renderer-replay standard/atomic Metal modes now detect groups and call render_frame_to_canvas before opening the final frame. The final frame clears transparent because the prepared root canvas already contains the requested clear color. Flat streams use the original path. CLI tests pass nested alpha, exact repeats, white override, translucent-clear alpha and malformed truncated-group rejection without output. Evidence: group-opacity-replay-cli-r2/receipt.json. Current target/debug renderer supports this path; frozen P04 renderer is intentionally unchanged. Next: runtime-generated fixture recordings versus pinned Chrome.

## Text/image composition preparation

Twelve prospective fixtures now cover text, image and mixed content with visible/rounded clipping at alpha0.5/1, including independently half-opaque text. Pinned Chrome captured36 reference views in group-opacity-composition-oracle-r2. Removing an unused #content opacity rule from image-only cases leaves all36 browser PNGs identical to the first capture. No runtime composition recording yet.

GlyphRenderer previously declined group capability by default. It now forwards capacity/begin/end and snapshots its own transform/modulation state across group boundaries; failed begin leaves state untouched and unbalanced explicit saves prevent group end. Its group alpha is not multiplied into per-glyph alpha. Tests require both renderer-metal and native-glyphs-experimental; the initial command omitted the latter and selected zero tests (preserved log).

## Intermediate dithering correction

A single flat-colored rectangle reproduces spatial variation growing with group depth. Nearest filtering made no difference. Disabling intermediate dithering fixed both the minimal loop and original composition corpus. Explicit begin_compositing_frame now disables dithering only on intermediate native canvases; begin_frame and final rendering retain existing defaults. Unsupported canvas implementations reject the new operation. Temporary diagnostic environment code was removed. Fixed flat-color8, CLI controls, API36/stream16 and original96-frame replay pass. Corrected visual review/artifact audit remain in progress.

### P05 corrected composition visual and artifact audit complete — 2026-09-10

All36 corrected text/image/mixed Chrome/native views now have audited visual coverage (18 directly inspected and18 exact full-image transfers). Text wrapping, nested alpha, image quadrants, responsive borders and outside siblings agree visually; small quantization and sparse curved-edge antialias residuals remain documented without tolerance changes. Fresh asset-aware artifact audit reproduces12 opacity-stripped compiler artifacts and verifies injected alpha, recorded group balance, all96 original/clone resize frames and reviewed image hashes. Public opacity CSS remains unqualified. Next revalidate initial/stacking corpora with the corrected renderer, then complete compiler/host transport and parity. Evidence: output/playwright/html-to-riv/group-opacity-composition-compositing-frame-lifecycle/diagnostic-opacity-audit.json and group-opacity-composition-runtime-progress.json.

### P05 corrected initial and stacking replay passes — 2026-09-10

The frozen undithered-compositing renderer passes all96 initial and48 stacking frames against pinned Chrome, including original/clone repeat stability. Initial corrected review has6/36 directly inspected views (nested half-opacity and translucent border at three widths); stacking18 views await review. Only9 initial views are byte-identical to the old output; no wholesale review transfer claimed. Fresh artifact audits remain pending until visual coverage completes. Evidence: output/playwright/html-to-riv/group-opacity-corrected-regression-progress.json. Public opacity admission remains pending.

### P05 corrected runtime corpora fully audited — 2026-09-10

Corrected initial36 and stacking18 views now have complete audited coverage (initial30 direct/6 exact-image transfers; stacking12 direct/6 transfers). Fresh artifact audits verify all18 stripped compiler artifacts and144 original/clone frames against the recorded inputs and reviewed images. Alongside corrected composition12 scenes/96 frames/36 views, the current renderer has30 diagnostic scenes,240 lifecycle frames and90 reviewed views. Stacking discriminators preserve group isolation below opacity1 and allow child z-index participation at opacity1. Sparse curved-edge antialias and quantization differences remain explicit; public compiler admission, host transport, native/WASM parity and broader public regression are still pending. Receipt: output/playwright/html-to-riv/group-opacity-corrected-regression-progress.json.

### P05 computed opacity cascade integrated — 2026-09-10

Style now stores group opacity independently of paint colors, defaults to1 without inheritance, supports explicit inherit/initial/unset, and uses the existing normalized number/percentage parser. Added cascade/substitution tests. Initial focused run5/6 exposed math fallback incorrectly invalidating to unset; corrected compatibility classification retains unsupported math diagnostics. Full library48/48 passes. Public admission remains explicitly gated until isolated-compositing transport exists; one regression test covers8 authored values and passes. No new public opacity support claimed. Receipt: output/playwright/html-to-riv/group-opacity-computed-style-progress.json.

### P05 version25 requirement contract tested — 2026-09-10

Added LayoutGroupOpacityRequirement with unique non-root LayoutComponent IDs and finite normalized opacity in [0,1); opaque entries are omitted because opacity1 does not establish a stacking context. Version25 and layout-css-group-opacity-v1 must match nonempty entries. The capability explicitly requires isolated subtree compositing, undithered intermediate surfaces and checked renderer capacity. Structural validation rejects missing/extra fields, wrong types, duplicates, root targets, invalid alpha and mismatched versions/capabilities. Corner coexistence and old-version controls pass. Three opacity plus four corner requirement tests pass, and TypeScript version25 declarations/typecheck pass. Checked host installation, public emission and native/WASM parity remain pending; CSS admission stays gated. Receipt: output/playwright/html-to-riv/group-opacity-requirements-progress.json.

### P05 checked host installation verified — 2026-09-10

Probe now advertises and installs version25 group-opacity requirements, supports explicit disabled-capability controls and uses checked instance drawing. Added public try_draw/try_draw_handle entry points preserving artboard frame identity; insufficient group capacity refuses before paint. Existing atomic/clone/clear/resize integration test now exercises the checked instance API and passes. New host transport test passes nested alpha at four widths, missing capability and malformed manifest rejection before stream output. Initial full host run had two feature-configuration failures (probe lacked native-glyph-controls); preserved. Rebuilt with native-glyph-controls and full host regression passes. Public CSS emission remains gated; these are manually supplied manifest controls. Receipt: output/playwright/html-to-riv/group-opacity-host-progress.json.

### P05 public opacity emission candidate — 2026-09-10

Public CSS now emits version25 normalized group requirements for computed opacity below1, leaving Rive layout and paint bytes unchanged. Explicit inheritance, CSS-wide resets, percentages, clamping, important cascade and custom-property fallback are covered; opaque values omit requirements and preserve prior artifacts. Five requirement/public emission tests pass. General opacity math remains rejected. Full Rust regression initially stopped on a stale opacity-rejection expectation; that test now checks excluded calc syntax, with original failure retained. Full rerun and native/WASM parity are pending; this is an implemented candidate, not a native-qualified feature. Added 30-scene JS/native/WASM parity test using existing Chrome-reference corpora; execution awaits matching builds. Current scope remains static isolated compositing through the checked recording/replay host, not immediate backends without group support.

### P05 public emission module and parity pass — 2026-09-10

Full corrected Rust module suite426/426 passes. Matching native and WASM compilers build successfully; new30-scene public opacity parity test passes across initial, stacking and text/image composition Chrome-reference inputs, comparing complete Rive bytes, source maps and runtime requirements. Public Chrome/native pixels and compile-once clone/resize qualification remain pending; runtime-only diagnostic receipts are not promoted to public evidence. Receipt: output/playwright/html-to-riv/group-opacity-public-emission-progress.json.

### P05 public lifecycle pixels and artifact audit pass — 2026-09-10

Thirty authored opacity scenes now compile directly, install only emitted group requirements, clone once and resize both instances across240 frames. Geometry and native/Chrome pixel gates pass with repeated-state byte stability. All90 public views exactly match authored HTML/CSS and complete browser/native PNGs from the corrected, independently audited diagnostic reviews; specialized visual transfer records this without claiming new inspection or transferring compiler qualification. A separate fresh public artifact audit reproduces all30 Rive artifacts/requirements, checks source-map alpha targets, all240 stream/frame identities and group balance, and verifies reviewed images. Broader edges, audit negative controls and full regression remain before P05 qualification. Receipt: output/playwright/html-to-riv/group-opacity-public-lifecycle-progress.json.

### P05 audit negative controls and public precision correction — 2026-09-10

Eighteen negative controls reject changed HTML/CSS, hash claims, actual PNG/Rive bytes, qualification, missing/duplicate views, requirements, injected policy, frame identity and missing frames; two positive controls pass. Controls operate in temporary copies and leave originals read-only. Public boundary tests exposed generic CSS f32 token normalization rejecting finite1e100 opacity before clamping. Numeric opacity text is now preserved until computed-value invalidation and the f64 opacity parser; this also retains near100% precision. First fix prematurely validated multi-token var fallbacks; retained failure and corrected ordering. Library48 plus opacity requirement/public6 tests pass. Existing public visual receipts describe the preceding frozen outputs; refreshed parity, Chrome boundary evidence and full regression remain. Receipt: output/playwright/html-to-riv/group-opacity-audit-precision-progress.json.

### P05 Chrome opacity boundary semantics and parity pass — 2026-09-10

Pinned Chrome28 controls (14 values authored directly and through var fallback) agree with public requirement presence using an observable z-index/elementFromPoint discriminator. Chrome serializes computed opacity as1 for some values that still isolate stacking, so computed-style text alone is insufficient:99.999997% retains a group while99.999999% does not. Large finite values clamp correctly. Refreshed WASM and native builds pass opacity parity across30 composition scenes plus28 boundary inputs, comparing Rive bytes, source maps and requirements. Browser screenshots are captured but not claimed as new native visual qualification. Native boundary lifecycle pixels, broader combinations and full regression remain. Receipt: output/playwright/html-to-riv/group-opacity-precision-progress.json.

### P05 native boundary pixels visually audited — 2026-09-10

The28 direct/var boundary inputs now compile publicly and replay through the native renderer at240x320. All geometry/pixel gates and exact browser/native100,100 overlap-color samples pass. Four distinct full-resolution image pairs directly inspected: invisible, half-alpha, fully opaque (red over green), near-opaque isolated (green over red). Remaining24 views have verified exact full-PNG transfers; review audit complete. Small quantization differences remain explicit, tolerances unchanged. These static controls do not establish clone/resize or broader composition coverage. Receipt: output/playwright/html-to-riv/group-opacity-native-boundary-progress.json.

### P05 boundary clone/resize pixels reviewed; metadata correction — 2026-09-10

Twenty-eight boundary scenes now compile once and resize original/clone across224 frames. Geometry/pixels/repeat stability pass, with84 views reviewed (12 direct and72 full-image transfers). Fresh artifact audit exposed recorder serde_json::Value widening f32 alpha versus compiler shortest-roundtrip serialization. Recorder now serializes requirements through the public JSON serializer; corrected recording test passes and all224 stream files are byte-identical to the prior recording. Alpha comparison in the audit uses exact f32 representation for known f32 fields, with no tolerance. Corrected metadata/replay binding and final artifact audit remain pending, so no completed boundary qualification claimed. Receipt: output/playwright/html-to-riv/group-opacity-boundary-lifecycle-progress.json.

### P05 corrected boundary artifact audit complete — 2026-09-10

Corrected metadata recording replays224/224 frames successfully;84 views carry forward prior inspection only after authored source, complete PNG, stream and freshly generated sheet identity checks. Fresh public artifact audit now passes with exact f32 semantic alpha comparison and exact serialized requirement reproduction. Boundary controls are complete within this frozen native-renderer scope. Independently removed the per-group linear plan scan during canvas composition by retaining the group node alongside its indexed prepared canvas; stream tests pass, native verification pending. Receipts: output/playwright/html-to-riv/group-opacity-boundary-completion-progress.json and group-opacity-boundary-public-lifecycle-r2/public-opacity-audit.json. Broader composition/resource controls and full regression remain before P05 qualification.

### P05 indexed canvas lookup preserves all focused native frames — 2026-09-10

The indexed prepared-canvas lookup passes16 stream tests, native CLI alpha/clear/rejection controls and8 flat-color controls. All464 audited public opacity frames rerender with complete native PNG identity to their reviewed baselines; no new visual inspection claimed or needed for unchanged full images. Updated the current P05 backlog row and leading support contract to describe implemented version25 behavior, remaining qualification and explicit replay constraints. Broader opacity compositions, backend/resource limits and full visual regression remain. Receipt: output/playwright/html-to-riv/group-opacity-indexed-native-progress.json.

### P05 resource boundary and overflowing composition coverage — 2026-09-10

Added exact256MiB surface-budget boundary controls: root plus63 1024² groups reaches the allocation interface; an extra group or column rejects before paint, for nested and sibling groups. Stream17 tests pass. Added18 public scenes combining opacity0/.5/1, border/content-box sizing, visible/rounded/axis clipping and genuinely overflowing text/images. All144 original/clone geometry/pixel frames and repeat checks pass; expanded native/WASM opacity parity covers76 scenes. First three half-opacity rounded-clip views directly reviewed; remaining51 views and artifact audit pending. Recorder now installs content-box and axis policies for this corpus. Receipt: output/playwright/html-to-riv/group-opacity-overflow-composition-progress.json.

### P05 decorated opacity visual and artifact audit complete — 2026-09-10

All36 decorated-text views now have complete coverage (27 direct,9 exact full-image transfers for invisible controls). A fresh audit reproduces12 public Rive artifacts/requirements and verifies96 original/clone resize frames. Underline clipping and visible overflow, strikethrough positioning, nested text alpha and outside-sibling placement agree with Chrome; existing sparse antialias/quantization residuals retained without tolerance changes. Focused public totals now88 scenes/704 frames/264 views. The decoration corpus remains separate from the running7924-check full regression and will be registered afterward. Host transform/backend constraints and full regression review remain. Receipt: output/playwright/html-to-riv/group-opacity-decoration-progress.json.

## Frozen opacity full regression and visual audit

The corrected P05 frozen toolchain passes all 7,960 native checks against pinned
Chrome 153.0.8010.12. All 7,950 scene pairs have completed visual coverage:
7,686 exact compiler-input and full-image transfers from the reviewed P04
baseline, plus 264 exact transfers from the reviewed public opacity lifecycle
corpora. Both source audits completed, the union covers every gallery key, and
the combined proof was verified against its component hashes and completed
report. No new direct inspection or tolerance waiver is claimed.

Evidence: `output/playwright/html-to-riv/group-opacity-full-r2-validation-receipt.json`
and `group-opacity-full-native-r2/combined-visual-comparison.json` in the same
output root. The earlier disk-exhaustion run remains preserved. This evidence
applies to the frozen P05 toolchain, not subsequent experimental P06 changes.
P05 remains active for broader host/image modulation and backend qualification.
