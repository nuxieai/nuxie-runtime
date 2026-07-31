# FL-E5 W65 test disposition

W65 assigns 22 `layout_test.cpp` cases, 3 `bounds_test.cpp` cases, 17
`text_test.cpp` cases, 2 `text_modifier_test.cpp` cases, and the nested-text
cases to this SCC. Stage 2 completes the assigned assertion load: direct Rust
tests preserve every non-silver assertion sequence, while the nine layout and
four text bodies whose pinned contract is a serialized-rendering comparison
remain exercised by the silver corpus. No expectation or comparison tolerance
was weakened.

| W65 class | Upstream source | FL-E5 evidence |
|---|---|---|
| B | `bounds_test.cpp` (3 assigned) | `upstream_background_shape_bounds_follow_text_and_artboard_transform`, `upstream_raw_path_coarse_and_precise_bounds_are_ported`, and `upstream_text_local_bounds_fixture_retains_origin_adjusted_bounds` directly port all three assertion sequences, including mutable Artboard world transform and retained pre-draw Text bounds. |
| B | `layout_test.cpp` (22 assigned) | Thirteen direct bodies cover row/column/gap/wrap/center/intrinsic measurement, padding, margin, corner radii, inherited RTL, forced-size dirt, alignment mutation, and Artboard percent-margin behavior. The remaining nine pinned `[silver]` bodies execute as `collapsing_elements`, `layout_display`, `layout_paint`, `layout_anim_bound`, `layout_anim_component_list`, `layout_anim_nested`, `layout_aspect_ratio`, `layout_fixed_fill`, and `layout_hug_artboard`. |
| B | `text_test.cpp` (17 assigned) | Thirteen direct bodies cover object/run discovery, mutation including empty text, vertical trim and packed passthroughs, ellipsis/glyph lookup, fit-font-size, word mapping, run-selected modifier coverage including whitespace, varying UTF-8 run size, double newlines, and opacity-modifier load. The four serialized bodies remain covered by `zero_width_space_line_break`, `word_joiner_test`, `fit_font_size_test`, and `text_vertical_trim_test`/`vertical_align_ellipsis`. |
| B | `text_modifier_test.cpp` (2 assigned) | `upstream_text_modifier_structure_body_is_ported` and `upstream_text_feather_falloff_repro_structure_body_is_ported` directly preserve both assertion sequences; the latter's full animated draw remains in the `text_feather_falloff` silver/golden case. |
| B | `serialized_rendering_test.cpp` text/layout cases | The silver report records every requested layout/text signature at its current first mismatch; exact-count ratchets remain active. |

Boundary notes:

- TextInput editing/cursor/selection tests belong to the input-text family.
- The 18 former consolidated owners now have direct file/member boundaries.
  Implemented members live with their matching pinned owner; owners that remain
  tracked gaps (TextModifier, TextStyleFeature, TextTargetModifier, and
  TextVariationModifier) directly own the explicit unsupported boundary rather
  than unrelated helpers. RawText directly owns the implemented settled-value
  read and records its standalone builder/update/render API as a precise
  pending member remainder. Solo directly owns occurrence state while
  `artboard.rs` remains its explicit graph-wide mutation/query integration
  client. `text.rs`, `draw.rs`, `components.rs`, and `lib.rs` likewise remain
  shared coordinators/integration clients, and pending feature ceilings are not
  reclassified by the behavior-preserving moves.
- `Text::buildRenderStyles` construction, retained bounds, and the
  Text → Node → LayoutComponent → Artboard publication now occur in the mutable
  update phase. FL-G07 is closed with mechanism citations.
- The three `layout_anim_*` point-value mismatches are retained layout geometry
  interpolation differences, not hidden as callback-chain test passes.
