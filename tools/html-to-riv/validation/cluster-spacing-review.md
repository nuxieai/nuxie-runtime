# Experimental cluster-spacing seam

The normal Rive interpretation remains per-glyph spacing. HbFont now exposes an
explicit per-font LetterSpacingMode::ClusterExperimental opt-in. Decode defaults
to RiveGlyph; with_options preserves the chosen mode while retaining original
bytes, face, axes and features. Choosing RiveGlyph again restores legacy advances.
No global flag, font-name marker or altered OpenType tag changes existing assets.

The shaping loop adds spacing at the final glyph of a contiguous shaping cluster,
leaving earlier glyph advances and mark offsets untouched. This fixes the LTR
multiple-mark specimen. It is an experiment, not a complete Unicode typography
claim: cursive/RTL, optional ligature suppression and fallback-policy propagation
need broader qualification before calling it a general CSS text mode.

The probe's NUXIE_CLUSTER_SPACING=1 opt-in replaces decoded font assets with
policy-preserving clones before drawing. The gallery identifies this as an
experimental host override. The compiler bytes do not yet request this policy.
The new native test checks positive/negative cluster advances, unchanged glyphs
and mark offsets, retained mode through font options, and restoration of legacy
advances. Both existing pinned HarfBuzz compatibility unit tests pass.

All 27 letter-spacing browser comparisons pass with the override, including all
three previous multiple-mark failures. Corrected mark placement was visually
inspected at 240/390/768px. The same font bytes, compiled scene, native layout,
real renderer and unchanged browser gates are used. No browser rectangles are
baked into the compiler output.

Full experimental override gate: **310/311**, with only the preserved A09
fractional-edge failure. All 50 Rust tests, five JS/WASM tests, two gallery checks,
TypeScript, module Clippy and pure-runtime boundary pass. The two existing
HarfBuzz compatibility unit tests pass. The 27 renderer-state controls pass
separately. The vector new-only override also passes all 27 spacing comparisons.

Comparing embedded native PNGs between the prior default gallery and this full
override gallery finds exactly three changed images among 303 scene renders:
the multiple-mark specimen at each viewport. Those three were visually inspected;
the other 300 PNGs are byte-identical. The eight remaining checks are pixel-gate
sensitivity tests, not additional scene images. Chromium 153.0.8010.12, DPR 1.

Reproduce the experimental gate:

```sh
NUXIE_CLUSTER_SPACING=1 bash tools/html-to-riv/validation/run.sh native-glyphs
```

Remaining delivery work: expose a versioned runtime requirement in the compiler
result and a checked native host path that applies it on import and on future
font replacement. Missing capability must be explicit; an ordinary .riv import
continues to use legacy semantics. Evaluate persistence/publish packaging and
native/WASM API parity together, without silently using a probe-only environment
variable as the product contract. A12 remains partial until this is resolved.

## Next implementation slice

Use the existing CompileOutput object as the artifact envelope: riv bytes,
source map and versioned runtime requirements. A requirement must be emitted
whenever a text style has nonzero spacing, not only when its current string
happens to contain combining marks; later text replacement can introduce them.
Expose and compare the same requirements through the CLI sidecar, WASM metadata
and TypeScript result. Plain scenes should require no experimental text mode.

On the runtime side, the existing FontAsset setter/decoder is the retention
point to investigate: installing a mode once must also govern later replacement
of that asset's font. A one-time font clone in the probe is insufficient. Keep
this selection explicit at the file/asset host boundary, and preserve the
zero-configuration Rive path. Validate unsupported/missing capabilities,
replacement, repeated import, independent files and serialization round trips
before treating the envelope as a publish contract.

Artifacts: output/playwright/html-to-riv/cluster-spacing-full/gallery.html,
cluster-spacing/gallery.html (new-only glyph profile), and
cluster-spacing-vector/gallery.html. Default-mode evidence remains in
letter-spacing-full/gallery.html. Qualification of this host override does not
make raw compiled .riv bytes request it; A12 remains partial.
