# Nuxie patches for taffy 0.15.0

Every changed site in `src` carries a `// nuxie-css:` comment. Everything
else is the published package unchanged.

## In-flow children count their margin boxes in scrollable overflow

Stock 0.15.0 counts a child's border box (and the descendant overflow it
lets escape) in its parent's scrollable overflow, and leaves the child's
margins out. A scroll range therefore loses the last item's end margin.
Chrome counts an in-flow child's margin box, so the end margin extends the
range. [CSS Overflow 3](https://www.w3.org/TR/css-overflow-3/#scrollable)
requires this for flex and grid items and leaves other boxes undefined;
Chrome does it for block children as well, including a margin collapsed
through a child.

The patch adds `compute_inflow_overflow_contribution` in
`src/compute/common/scrollable_overflow.rs`, a wrapper that unions the
child's margin box with the unchanged stock contribution. Callers:

- `src/compute/flexbox.rs`, `calculate_flex_item`: every flex item, with
  its used margins (auto margins included).
- `src/compute/grid/alignment.rs`, `align_and_position_item`: in-flow grid
  items; absolutely positioned ones take the stock path.
- `src/compute/block.rs`, the per-child contribution in the in-flow pass and
  the recomputation after `align-content`: in-flow block children; floats
  take the stock path.

Rules the wrapper keeps:

- The margin box is a union, so a negative margin never shrinks the border
  box or descendant overflow, and descendants are neither moved nor
  enlarged by their ancestor's margins.
- RTL callers pass mirrored locations, so the logical start margin is the
  physical right one.
- The margin box counts even when the border box is empty (an empty spacer
  with a bottom margin), as in Chrome. A margin box wholly before a scroll
  container's origin is excluded, as stock excludes such border boxes.
- It applies at every level, not only to a scroll container's own children.
  The runtime puts a content box between a CSS scroller and its items and
  sizes it to the viewport, so items overflow it; counting margins at every
  level passes their margin boxes through that box, as if the items were the
  scroller's children, which they are in the source HTML.

### Measurements

Headless Chrome 156, a 100 by 96 scroller with 5 and 11 points of vertical
padding (or the horizontal equivalent), a 120-point item and a 17-point end
margin:

| Case | Chrome | Stock | Fork |
|---|---|---|---|
| flex column, grid, block, flex row, RTL flex row, block row | 57 | 40 | 57 |
| block, the margin collapsed through an auto-height child | 57 | 40 | 57 |
| empty last item with a 17-point bottom margin (flex and block) | 57 | 40 | 57 |

nuxie-dev's Chrome differential fixtures, ranges at widths 360, 402, 520:

| Case | Chrome |
|---|---|
| c045, flex | (132, 176), (128, 176), (116, 176) |
| c147, grid | (0, 594), (0, 494), (0, 494) |
| c297, flex | (41, 298), (12, 298), (0, 298) |

### Known differences from Chrome

These come from where Taffy adds a scroller's end padding (after all
overflow, where Chrome adds it after in-flow children's margin boxes only)
and from counting margins at every level. Same geometry as above; the stock
and fork columns follow from the rules above rather than from a test:

| Case | Chrome | Stock | Fork |
|---|---|---|---|
| a 200-point descendant overflowing the 120-point item | 109 | 120 | 120 |
| the same, the descendant with a 17-point bottom margin | 109 | 120 | 137 |
| a last block child with a -10 bottom margin | 30 | 40 | 40 |

The second row is the cost of counting margins at every level. A fix for the
padding placement must keep the runtime's content box transparent, or the
scroller loses its end padding.

### Guard tests

In the runtime's
`crates/nuxie-runtime/src/mechanical_port/source/layout_css_mode_tests.rs`:

- `css_scroll_overflow_taffy_fork_counts_the_last_items_margin`: a plain
  Taffy tree for flex, grid and block scrollers; 57 with the patch, 40
  without it. A Taffy update that drops the patch fails here first.
- `css_scroll_overflow_flex_row_includes_final_margin_and_padding`,
  `css_scroll_overflow_flex_column_and_grid_include_final_margin_and_padding`,
  `css_scroll_overflow_rtl_includes_end_margin_and_padding`,
  `css_scroll_overflow_block_child_includes_its_end_margin`,
  `css_scroll_overflow_block_includes_a_margin_collapsed_through_a_child`,
  `css_scroll_overflow_empty_last_item_keeps_its_margin`,
  `css_scroll_overflow_block_row_includes_the_end_margin`: the Chrome
  measurements above, through the runtime's CSS records.
- `css_scroll_overflow_descendants_do_not_gain_the_items_margin`: an item's
  margin is not added after its descendants' overflow (Chrome 120).
- `css_scroll_overflow_c045_flex_item_margin_box`,
  `css_scroll_overflow_c147_grid_final_margin`,
  `css_scroll_overflow_c297_flex_margin_and_viewport_padding`: the
  differential fixtures at all three widths.
- `translated_scroll_overflow_keeps_content_bounds_range`: translated
  (Rive) scrollers do not use Taffy's overflow and keep their range.
