# Nuxie Rive HarfBuzz outline parity patches

This directory starts from the crates.io `skrifa` 0.45.1 package. Nuxie keeps
Skrifa as its Rust-native outline backend and changes two HarfBuzz outline
seams to preserve the observable behavior of Rive's pinned HarfBuzz.

## Source authority

- Rive runtime commit:
  `4ac7b32798da0482e441ef09304dc3b480ed3ee5`.
- Rive text owner: `src/text/font_hb.cpp`, `HBFont::getPath`, which delegates
  to `hb_font_draw_glyph` after setting the font scale to 2048.
- Rive HarfBuzz dependency: `rive-app/harfbuzz` tag `rive_13.1.1`.
- HarfBuzz owner: `src/OT/glyf/Glyph.hh`, which assembles composite outlines
  in font units, and `src/OT/glyf/path-builder.hh`, which applies the font's
  requested em scale to every resulting point.
- HarfBuzz contour-order owner: `src/OT/glyf/glyf.hh`,
  `glyf_accelerator_t::get_points`, which rotates a contour whose serialized
  first point is off-curve so its final serialized point is consumed first.

## Behavioral delta

Skrifa 0.45.1's HarfBuzz scaler applies the requested em scale while loading
each child outline, but adds the composite's placement offset afterward in
unscaled font units. The pinned HarfBuzz owner assembles both child points and
their placement offsets in font units, then applies the requested em scale to
the complete path. This fork applies Skrifa's existing scale to that one
offset before adding it to the already-scaled child. All other Skrifa behavior
is byte-identical to the crates.io package.

Skrifa 0.45.1's `PathStyle::HarfBuzz` applies the forward-scan rules from
`path-builder.hh`, but it originally fed those rules the serialized contour
order directly. This fork first applies the off-curve contour rotation from
the pinned `glyf.hh`, then applies Skrifa's HarfBuzz-style forward scan.

## Differential evidence

The pinned `bankcard.riv` C8 replay first diverges in variable-font composite
glyph 610 at frame 19, operation 6274 when Skrifa's original unscaled offset is
used. The displacement is constant across the affected component contour,
matching an offset added in the wrong coordinate space.

The pinned `new_text.riv` scripted replay previously first diverged at the
outline of the standalone glyph `x`. Rust emitted identical closed contour
geometry with the first quadratic segment cyclically moved to the end. After
applying the pinned HarfBuzz off-curve contour rotation, all three requested
samples compare exactly with the pinned C++ runner.
