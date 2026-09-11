# Percentage min/max dimensions (A11, qualified)

Min/max width and height retain percentage values and unit tags in the native
layout style; no viewport-derived pixels are baked into the file. The existing
Rive style adapter already transfers these units to layout min/max dimensions.
Inheritance retains the percentage; initial max constraints clear the limit.
Coefficients follow the existing [0,10000]% resource range. Auto and intrinsic
keywords remain outside this feature's contract.

Eight fixtures cover width and height caps/floors, flex weights, inherited
percentages, indefinite width/height parents, minimum-over-maximum precedence,
and responsive text. The public contract initially failed on unsupported
percentage lengths. A native import test resizes both axes on one scene and
checks minimum/maximum clamping in three viewports.

The initial full experimental glyph gate passed 281/284 checks. Failures were
the existing A09 fractional-edge specimen and text limits at 240/390px. At 240px,
the text box had the correct 168px width but native height 48px versus browser
96px; its background ended before the last two painted lines. At 390px native
height was 48px versus 72px. An independently authored 168px maximum reproduced
the bug, excluding percentage conversion as its cause.

Taffy's flex-basis measurement passed an authored known cross size before
clamping it to min/max. The fix clamps only that cross size; the main-axis basis
remains unconstrained. The pinned Yoga source also constrains maxima before
measuring a child's flex basis, including exact modes. See the vendor patch
receipt. A public native pixel/percentage capped-text regression fails before
and passes after the fix at 240,390,768px.

Final experimental glyph gate: **283/284**, with only the preserved A09
fractional-edge failure. All 24 new geometry/pixel comparisons pass in both
renderer profiles. The vector run is new-only. All 48 Rust tests, five JS/WASM
tests, two gallery checks, TypeScript, Clippy and pure-runtime boundary pass.
The extended native text test also checks minimum widths; independent Chromium
measurements confirm 96/72/24px heights for a 75% minimum at the three widths.

All 41 selected upstream/proxy layout tests pass, as does the pinned exact render
comparison wave_b_focus_test_078_direct_port_expected_red. The 27 renderer host
controls pass separately. These cover the changed layout seam and existing
fixtures; they are not a claim of exhaustive upstream runtime qualification.

All 24 browser/native/diff triples were visually inspected. The 21 shape PNGs
are byte-identical across renderer profiles; the three vector text images were
also inspected. Background height now follows the wrapped text. Chromium
153.0.8010.12, DPR 1; actual Rust Metal replay. No tolerances changed.

Commands from repository root:

```sh
CARGO_INCREMENTAL=0 bash tools/html-to-riv/validation/run.sh native-glyphs
CARGO_INCREMENTAL=0 cargo test -p nuxie-runtime --locked --test upstream_layout_wave_c2 --test upstream_layout_participant_wave_c1 --test upstream_legacy_layout_transform --test native_layout_proxy_visibility
NUXIE_NATIVE_GLYPHS=1 node tools/html-to-riv/validation/glyph-state-control.mjs
```

Semantic reference: [CSS sizing](https://www.w3.org/TR/css-sizing-3/#min-size-properties).
The profile remains flex-only and border-box. Neither Grid nor the editor is
involved. Remaining unrelated rasterizer differences are not waived by this work.

For new-only vector qualification:

```sh
cd tools/html-to-riv
env -u NUXIE_NATIVE_GLYPHS npm test -- --grep 'percent-limits-' --output=test-results-percent-vector
```

Local artifacts: output/playwright/html-to-riv/percent-limits-fixed/gallery.html,
review-{240,390,768}-{0,1}.png, and percent-limits-vector/gallery.html. The earlier
percent-limits-full/gallery.html preserves the pre-fix browser evidence. The
checked-in corpus and native test reproduce the corrected behavior.

A11 is qualified within the supported flex profile. Next: A12 letter spacing.
