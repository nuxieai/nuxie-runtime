# A20 host-state qualification: retained failures

Run `node tools/html-to-riv/validation/glyph-state-control.mjs underline` from
repo root after the native-glyph validation build. The native-glyph gate also
runs this profile after the ordinary glyph controls (earlier failures can stop
the gate before it reaches these controls).

The Japanese fixture is compiled once at 390px and the same scene is imported
and resized to 240, 390 and 768px. Two paragraphs exercise Auto and All skip-ink,
with 24px text, 40px lines, red 2px underline and -6px offset. These are host
renderer transforms, not added CSS transform syntax. Chrome is the reference.

Nine states per width: identity, fractional translation, clip through glyphs,
opacity, uniform scale, nonuniform scale, rotation, shear, and combined
rotation/clip/opacity. Every frame draws a second copy after save/restore, at
y=160, to expose state leakage. Browser and native share the embedded font.
DPR remains 1; this run does not qualify DPR 2 or 3.

## Results

27/27 pass geometry within 0.1px, ordinary image limits, and ink coverage.
0/27 pass the additional whole-frame red-decoration support check: 2–6 missing
and 6–16 extra red pixels outside its existing one-pixel neighborhood. No
thresholds changed. All 27 browser/native pairs and diffs were inspected across
nine contact sheets; the layout, clipping and restored copies align visually,
with sparse decoration/glyph edge differences. This is not a qualification pass.

The whole-frame red check includes both copies. Therefore the 27 failures do
not establish 27 distinct transform defects: the identity/restored text already
fails. Fix the identity discrepancy first, then distinguish transformed residuals.
The added Latin paragraph broadens the original CJK reproducer. Do not assume
all additional differences have the hard-clip root cause until tested.

Artifacts: `output/playwright/html-to-riv/underline-state-controls/`, including
source request, compiled scene and requirements, per-case host state, draw
stream, bounds, browser/native/diff PNGs, results.json and review-{width}.html.
Reviewed sheets: review-{240,390,768}-{1,2,3}.png.
Log: `/tmp/html-underline-state.log` (exit 1, expected retained failures).

The unchanged glyph profile was rerun after parameterization: 27/27 pass,
`/tmp/html-glyph-state-regression.log` (exit 0). No production compiler or
runtime change in this step, so native/WASM parity was not rerun.

## Renderer seam inspection

At initial inspection, `nuxie-render-api::Renderer` only exposed `clip_path`; it had no difference
operation or antialiasing selection. The glyph adapter forwards that call.
Recording/serialization and `nuxie-render-stream::Command::ClipPath` also retain
only a path, and replay invokes that same API. NativeMetalFrame delegates to
the mechanically ported RiveRenderer clipPath. Its rectangular optimization
represents intersection clipping, not a hard rectangular difference operation.

A backend-only patch would be lost through recording/replay. The eventual fix
must preserve clip operation and coverage semantics through the public renderer
seam, recording, stream replay, adapters and native backend, including save/restore
and device transforms. Compiler-local interval rounding is insufficient. See
underline-clip-research.md for the pinned Chrome evidence.

## Isolated transformed copies

Use `node tools/html-to-riv/validation/glyph-state-control.mjs underline --no-restore`
to omit the second copy from both native and Chrome output. Results go to the
separate `underline-state-controls-isolated` artifact directory. The default
profile retains save/restore coverage unchanged.

6/27 isolated comparisons pass all checks: clip-through-glyphs and combined at
all three widths. The other 21 pass geometry, ordinary image and ink limits but
fail red support. Identity has 2 missing / 8 extra pixels at 240 and 2 missing /
6 extra at 390 and 768. Thus restored copies were masking the six clipped passes
in the original whole-frame result. All 27 isolated pairs were visually inspected
in the three full contact sheets. No error budget was changed.

Runtime preparation now exposes `ResolvedUnderline::build_stripes`, retaining
full line bounds and unsnapped per-glyph exclusion rectangles. Exclusions expand
the inset measurement stripe vertically by 1px, matching the pinned Chrome
source. Existing `build_path` consumes these stripes as a geometric fallback;
this does not yet implement hard clipping or claim a pixel improvement.
The pinned Inter geometry test also verifies the vertical expansion, fractional
edges, None mode, and translation of line and exclusions. Six runtime integration
tests pass. Renderer capability, stream serialization/replay and backend support
remain to be implemented.

Refactor verification: 19/19 public compiler tests pass (including native import,
underline drawing, and resizing); the rebuilt probe produces all 27 isolated
native PNGs byte-identical to the pre-refactor run. The original glyph profile
still passes 27/27 after the harness change. The runtime dependency-boundary check
passes. Logs: `/tmp/html-underline-stripes-expanded.log`,
`/tmp/html-underline-stripes-compiler.log`, `/tmp/html-underline-stripes-build.log`,
`/tmp/html-underline-stripes-render.log`, `/tmp/html-glyph-state-stripes-regression.log`,
`/tmp/html-underline-stripes-boundary.log`. PNG hashes before the refactor are in
`/tmp/html-underline-stripes-before.json`. Native/WASM compiler parity was not
rerun: compiler/transport code is unchanged in this step.

The subsequent transport step adds optional `clip_out_rect`, recording and text
stream replay, including explicit unsupported-operation failure. Native coverage
and runtime switching are still pending; see underline-clip-research.md.
