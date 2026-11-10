# Nuxie patches for taffy 0.15.0

Every changed site in `src` carries a `// nuxie-css:` comment. Everything
else is the published package unchanged.

## Measure replaced wrappers before placing their children

`src/tree/taffy_tree.rs`, `TaffyView::compute_child_layout`: a visible replaced
node with children and a measurement context calls the measurement callback
before container layout. Its measured border-box size becomes the known size
for placing those children. Hidden nodes and ordinary containers keep their
existing dispatch.

The runtime uses this for a CSS picture box with a filling Image or Video
participant. The ordinary leaf callback cannot size that box because Taffy
dispatches nodes with children straight to block, flex or grid layout. This
hook lets the runtime use the same measurement resolver as a direct picture
participant, then preserve the child's layout, paint and fit. No natural
ratio is stored in Style::aspect_ratio or in a separate style field. The
runtime reads current picture dimensions whenever invalidated layout measures
the node, and authored aspect-ratio stays in its existing style field.

Runtime guards in `layout_fit_content_tests.rs` cover natural block size,
column stretching, definite width in a flex row, padding with border-box
sizing, height constraints and frame changes without manual invalidation.

## In-flow children count their margin boxes in scrollable overflow

Stock 0.15.0 counts a child's border box (and the descendant overflow it
lets escape) in its parent's scrollable overflow, leaves the child's margins
out, and adds a scroll container's end padding after all of that overflow.
Chrome extends the end padding past the margin boxes of the scroll
container's own in-flow children, and counts everything else as it is: a
last item's end margin and the padding extend a scroll range, while a
descendant overflowing its parent, a descendant's margin and an absolutely
positioned child get neither.
[CSS Overflow 3](https://www.w3.org/TR/css-overflow-3/#scrollable) requires
the margin boxes for flex and grid items and leaves other boxes undefined;
Chrome does it for block children as well, including a margin collapsed
through a child. The padding placement is ported from the Taffy copy in
Expo's Exact (its patch 67) and rewritten against this copy.

`src/compute/common/scrollable_overflow.rs` keeps the two parts apart in
`InflowOverflow`: `overflow` is the stock rectangle, `margin_boxes` the union
of the in-flow children's margin boxes. `InflowOverflow::rect` joins them: a
scroll container gets `overflow` joined with `margin_boxes` grown by its end
padding; any other box gets `overflow` alone.
`compute_inflow_overflow_contribution` gives what one child adds to both.
Callers:

- `src/compute/flexbox.rs`, `calculate_flex_item` and `final_layout_pass`:
  every flex item, with its used margins (auto margins included).
- `src/compute/grid/types/grid_item.rs`, `minimum_contribution`: the
  result is floored by the item's padding and border, whatever it comes
  from (a preferred size, `min-height: 0`, a scroll container, a flexible
  track), so a
  border-box `height: 0` item with 20 of padding gets a 40-point track, as
  in Chrome (stock sized the track to the margin alone).
- `src/compute/grid/mod.rs`: a grid's `margin_boxes` is the extent of its
  tracks. Chrome pads the tracks: an item wider than its column, or its
  margin past it, gets no padding, and an empty track still counts. Items
  add their overflow only (`src/compute/grid/alignment.rs`).
- `src/compute/block.rs`, `compute_inner`: after `align-content` has moved
  the flow, in-flow block children add their inline margins (a float
  adds its whole margin box), and the block-axis end comes from the flow:
  `perform_final_layout_on_in_flow_children` returns the in-flow content's
  end after its trailing collapsed margins. A `relative_offset` field on the
  block item keeps each child's relative shift.
- `src/compute/grid/alignment.rs`: `relative_offset`, the relative shift
  `align_item_within_area` applied inline, is a helper of its own (the
  earlier margin patch shared it).

Rules:

- The padded margin boxes are joined with the overflow, so a negative margin
  never hides a border box or descendant overflow.
- The margin box sits at the child's in-flow position: callers pass the
  `position: relative` shift already in the location, and the stock
  contribution keeps the shifted border box, as Chrome counts both.
- In block flow the flow's end after the trailing collapsed margins counts,
  also when those margins collapse through the container to its parent. A
  child with a height also counts to its own bottom margin edge:
  its margin collapsed with its last descendants' that adjoin it, and not
  with what follows it (a 120-point child followed by an empty one with a
  -40 top margin still ends at 120; an auto-height child with a 120 bottom
  margin whose last child has -50 ends 70 past its border box). A self-collapsing child counts only where it
  sits; its margins count through the collapsed end.
- RTL callers pass mirrored locations, so the logical start margin is the
  physical right one.
- The margin box counts even when the border box is empty (an empty spacer
  with a bottom margin), as in Chrome. Each axis counts on its
  own, and a negative end margin pulls the end edge back no further than
  the border box's start edge.
- A box with no width or no height still passes on a descendant that
  overflows it toward the start (`compute_scrollable_overflow_contribution`
  judged emptiness from the end extent alone).
- Only a scroll container's own children count. The runtime puts a content
  box between a CSS scroller and its items, which are the scroller's
  children in the source HTML. `Style::scroll_content` (not a CSS property;
  `CoreStyle::scroll_content`) marks that box: it reports its own children's
  margin boxes in `LayoutOutput::scroll_content_margin_boxes`, and its
  parent counts them as its own children's. A marked box that clips or
  contains its overflow keeps them in. The runtime sets the flag on a
  CSS viewport's content box in `LayoutComponent::apply_scroll_content_minimum`.

### Measurements

Headless Chrome 156, a 100 by 96 scroller with 5 and 11 points of vertical
padding (or the horizontal equivalent), a 120-point item and a 17-point end
margin:

| Case | Chrome | Stock | Fork |
|---|---|---|---|
| flex column, grid, block, flex row, RTL flex row, block row | 57 | 40 | 57 |
| block, the margin collapsed through an auto-height child | 57 | 40 | 57 |
| empty last item with a 17-point bottom margin (flex and block) | 57 | 40 | 57 |

A 100 by 80 scroller without padding:

| Case | Chrome | Stock | Fork |
|---|---|---|---|
| 120-point item, `top: 30px`, 17-point bottom margin (flex, grid, block) | 70 | 70 | 70 |
| the same with `top: -30px` (flex) | 57 | 10 | 57 |
| empty block (margins 10, 17) between a 120-point and a 1-point block | 58 | 58 | 58 |
| the empty block last, margins 10 and 17, or 30 and 17 | 57, 70 | 40 | 57, 70 |
| the empty block, then a 1-point block with a 20-point top margin | 61 | 61 | 61 |
| the empty flex item last, margins 10 and 17 (flex margins do not collapse) | 67 | 40 | 67 |

nuxie-dev's Chrome differential fixtures, ranges at widths 360, 402, 520:

| Case | Chrome |
|---|---|
| c045, flex | (132, 176), (128, 176), (116, 176) |
| c147, grid | (0, 594), (0, 494), (0, 494) |
| c297, flex | (41, 298), (12, 298), (0, 298) |

### End padding, Chrome 152

The same 100 by 96 scroller with 5 and 11 points of vertical padding:

| Case | Chrome | Stock | Before this port | Fork |
|---|---|---|---|---|
| a 200-point descendant overflowing the 120-point item | 109 | 120 | 120 | 109 |
| the same, the descendant with a 17-point bottom margin | 109 | 120 | 137 | 109 |
| a last child with a -10 bottom margin | 30 | 40 | 40 | 30 |

### Known differences from Chrome

A `row-reverse` or `column-reverse` flex scroller scrolls toward its start
in Chrome (a 300-point item with a 15-point margin in a 200-point scroller
with 10 points of padding reads 335); stock and the fork read no range
there. Not part of this patch.

A grid scroller with tracks and no children scrolls to its tracks in Chrome
(a 300 by 200 track in a 200 by 100 scroller with 10 points of padding
reads 320 by 220). `TaffyTree` lays out a childless node as a leaf, so the
grid code does not run and there is no range.

Overflow wholly before the scroller's origin cannot be scrolled to in
Chrome (a 200 by 20 box in a zero-height column, pulled 100 above a 100 by
96 scroller, leaves the scroll width at 100). Inside the runtime's content
box it still counts (200): the content box is not the scroller and does not
know the scroller's start padding, which can reach such overflow (a box
pulled 30 up into 20 of top padding reads 200 in Chrome and here). Stock
has the same difference for a box with a size.

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
  `css_scroll_overflow_block_row_includes_the_end_margin`,
  `css_scroll_overflow_relative_items_count_their_margin_before_the_shift`,
  `css_scroll_overflow_self_collapsing_spacers_count_their_collapsed_margin`:
  the Chrome measurements above, through the runtime's CSS records.
- `css_scroll_overflow_taffy_fork_end_padding_extends_children_not_descendants`:
  116 Chrome 152 scrollWidth by scrollHeight rows on plain Taffy trees, for
  block, flex and grid scrollers.
- `css_scroll_overflow_taffy_fork_block_margins_collapse_through_a_child`:
  8 Chrome 152 rows for a block child whose bottom margin collapses with
  its last descendants', and 5 for a grid track floored by its item's
  padding.
- `css_scroll_overflow_end_padding_passes_through_the_content_box`: the end
  padding table above through the runtime's content box.
- `css_scroll_overflow_taffy_fork_clipping_content_box_keeps_its_items_in`
  and
  `css_scroll_overflow_taffy_fork_content_box_keeps_overflow_in_the_start_padding`:
  a marked content box passes nothing up when it clips, and does not drop
  overflow before its own origin, which the scroller's start padding can
  still reach.
- `css_scroll_overflow_descendants_do_not_gain_the_items_margin`: an item's
  margin is not added after its descendants' overflow (Chrome 120).
- `css_scroll_overflow_c045_flex_item_margin_box`,
  `css_scroll_overflow_c147_grid_final_margin`,
  `css_scroll_overflow_c297_flex_margin_and_viewport_padding`: the
  differential fixtures at all three widths.
- `translated_scroll_overflow_keeps_content_bounds_range`: translated
  (Rive) scrollers do not use Taffy's overflow and keep their range.

## Intrinsic single-line columns use outer hypothetical main sizes

In `src/compute/flexbox.rs`, `determine_container_main_size`, a column
with `flex-wrap: nowrap` sized under a min-content or max-content
constraint sums its items' outer hypothetical main sizes, then adds the
main-axis gaps and the container's padding and border insets. This applies
to every flex basis and does not depend on the grow or shrink factor.

`determine_flex_base_size` runs first and already computes
`hypothetical_inner_size` by clamping the flex base size by the resolved
minimum and maximum. The automatic minimum participates in this clamp:
for the non-replaced block items in the guard tests it is the smaller of
content size and a definite specified height, or zero for a scroll
container. `hypothetical_outer_size` adds the used margins. The patch
reuses that field rather than measuring content or resolving the limits
again. Flexible-length resolution then initializes each item's target
size from its hypothetical size as usual.

Upstream's preferred-size shortcut avoids an expensive measurement by
assuming an inherent-size measurement would return the preferred main
size. That is insufficient for intrinsic columns: an unresolved percentage
basis uses content and can size either below or above the specified
height. For example, `flex: 2; height: 80px` with 120 points of content
uses 120, while empty content uses zero.

Bypassing that shortcut alone also differs from Chrome. The general
column path measures content and floors it by the flex basis, allowing
content past a definite preferred height to enlarge the column. Chrome
156 instead uses the hypothetical size: `height: 80px` with 120 points of
content stays 80, as do `flex-basis: 0; height: 80px` and
`flex: 1 1 30px; height: 80px` with that content. The latter two reach 80
through their automatic minimum. A 100-point basis with the same content
stays 100.

Rows and wrapping columns (`wrap` and `wrap-reverse`) retain the upstream
intrinsic-sizing path, including both preferred-size shortcuts unchanged.
Only nowrap columns take the new branch, which requires
`!constants.is_wrap`, even when a wrapping container happens to collect
only one line. Wrapping columns have not been measured against Chrome.

A runtime layout root with definite available height can bypass the
intrinsic path. A block parent measures its children with max-content
available height and exposes the difference. Reverse direction, margins,
paint, sibling padding and borders, and scrolling are not required.

Guard tests in the runtime's `layout_css_mode_tests.rs`:

- `css_intrinsic_column_uses_hypothetical_item_heights`: all 15 supplied
  Chrome 156 measurements, each with a block parent, normal and reverse
  columns, and auto, min-content and max-content column heights. Covers
  auto, percentage and length bases, content below and above the specified
  height, explicit minimum and maximum heights, and hidden overflow.
- `css_intrinsic_column_sums_outer_hypothetical_sizes_and_gaps`: multiple
  items, positive and negative margins, a gap, padding and borders; the
  expected sizes are 80 and 50 for the items and 174 for the column.
- `css_flex_basis_percentage_in_a_column_under_a_block`: the minimal record
  tree, with normal and reverse columns and with and without an explicit
  column width.
- `css_flex_basis_taffy_intrinsic_columns_use_content`: a fresh plain Taffy
  tree under min-content and max-content constraints, both column
  directions, percentage, length and content bases, empty and 50-point
  content, and explicit minimum and maximum heights.
- `css_flex_basis_percentage_in_a_reverse_column_under_a_block_scroller`:
  the compiler's viewport/content-box lowering and the differential
  fixture's full geometry. Chrome 156 at width 360 gives the flexible
  item `[2, 453, 110, 0]`, its column `[-4, 431, 368, 144]`, and scroll
  range `(4, 433)`. Before either patch the item was 80 points tall and the
  vertical range was 513.
- The existing
  `css_flex_basis_percentage_ignores_the_height_against_an_indefinite_column`
  still covers painted and unpainted items, including growth to 140 in a
  definite 200-point column.

## Flex alignment follows Chrome on overflow

Three changes ported from the Taffy copy in Expo's Exact
(github.com/expo/exact, `vendor/taffy`, MIT; its patches 52, 53 and 55),
with Chrome 151 numbers from that project's fixtures.

- `src/compute/flexbox.rs`, `resolve_cross_axis_auto_margins`: auto
  cross-axis margins take positive free space only. An item larger than its
  line stays at the start edge: the top in a row, the left in a column, and
  the right in a right-to-left column, where the negative space becomes the
  item's cross offset. Stock gives the auto margin the negative space, so an
  80-point item with `margin-left: auto` in a 50-point column sits at -30
  where Chrome has 0.
- `src/compute/flexbox.rs`, `determine_used_cross_size`: a stretched item's
  cross size is floored at its padding and border, also when its maximum is
  smaller. Stock applies that floor only in child layout, so alignment
  places a narrower box than the one laid out: a stretched item with 40
  points of side padding in a 10-point right-to-left column sits at 0 where
  Chrome has -30.
- `src/compute/common/alignment.rs`, `apply_alignment_fallback`: when lines
  or items overflow, or there is one of them, `stretch` and `space-between`
  fall back to unsafe `flex-start`. Stock makes the fallback safe, which
  turns it into `start` on overflow and loses the reversal under
  `wrap-reverse`, a reversed main axis and a right-to-left column. CSS Align
  asks for safe `flex-start` for `space-between`; Chrome uses the unsafe
  one, and this follows Chrome. `space-around` and `space-evenly` keep their
  safe center. Grid and block callers share the function and are unchanged
  in effect: without flex reversal `start` and `flex-start` are the same
  edge.

Guard tests in the runtime's `layout_css_mode_tests.rs`, each on a plain
Taffy tree and each failing on stock 0.15.0:

- `css_flex_taffy_fork_cross_auto_margins_take_positive_space_only`: 13
  cases across rows, columns, a reversed column and right-to-left columns,
  with overflowing and fitting items.
- `css_flex_taffy_fork_stretched_item_is_never_narrower_than_its_padding`:
  the right-to-left and left-to-right 10-point columns and a fitting
  control.
- `css_flex_taffy_fork_overflowing_lines_keep_their_flex_start_edge`: 16
  cases of two wrapped lines, overflowing and fitting, under `wrap`,
  `wrap-reverse`, right-to-left columns, and the default, `stretch` and
  `space-between`.

## Flex container main size follows Chrome

Four changes in `determine_container_main_size` (`src/compute/flexbox.rs`),
ported from the Taffy copy in Expo's Exact (its patches 14, 33, 45 and 46),
with Chrome 151 numbers from that project's fixtures. The intrinsic nowrap
column arm above is kept as it is.

- In a column, an item's length on a line (definite space, and the wrapping
  min-content arm) is its flex base size clamped by the minimum that flexing enforces,
  the automatic minimum included, and by its max main size, floored by its
  padding and border, with the margin added last. Stock floors the base size
  by `min-*` alone and applies the padding floor after the margin. A
  wrapping column of three items with `flex-basis: 60px` around 100-point
  children was 60 tall where Chrome has 100; with `max-height: 70px` on the
  items Chrome has 70; with `margin-top: -10px` Chrome has 90. Rows are
  sized from intrinsic contributions (next section).
- A multi-line container is at least as long as its offered space only when
  it is a row. A wrapping column's automatic height is its longest line:
  three 60-point items in a 300-wide column with `max-height: 100px` make a
  column 60 tall in Chrome, where stock gave 100.
- An item's intrinsic contribution is clamped by its flex base size first
  and its min and max main size last (CSS Flexbox 9.9.1), and a scroll
  container's contribution is its base size clamped the same way, floored
  by its padding and border.
- The contribution's margin is added after the inner size is clamped. Added
  first, a negative margin was erased by `max(flex_basis)` and by the min
  and max clamp.
- A contribution is never smaller than the item's padding and border, also
  when its `max-*` is smaller than they are.

Guard tests in the runtime's `layout_css_mode_tests.rs`, on plain Taffy
trees built from Chrome rows:

- `css_flex_taffy_fork_wrapping_column_is_as_tall_as_its_longest_line`: 14
  rows of wrapping and nowrap columns with `max-height`, `min-height`,
  inverted bounds, padding, a gap, and reversed directions.
- `css_flex_taffy_fork_item_length_uses_the_enforced_minimum_and_maximum`:
  13 rows of columns whose items have a content-based minimum, a maximum, an
  explicit zero minimum, hidden overflow, padding and border, and negative
  and positive margins.
- `css_flex_taffy_fork_intrinsic_contribution_keeps_padding_under_a_small_maximum`:
  a max-content wrapping column of three padded items whose `max-height` is
  below their padding stays on one line, 90 tall.
- `css_flex_taffy_fork_auto_width_rows_keep_their_width`: two auto-width
  rows (a negative margin on a zero-basis item, and an item with a maximum
  under `max-width`) that the column change must not touch.

## Flex rows size from their items' intrinsic contributions

Ported from the Taffy copy in Expo's Exact (its patches 41, 47, 48 and 51),
with Chrome 151 rows from that project's fixtures, plus one change of our
own that the port made necessary (the first item). All in
`src/compute/flexbox.rs`.

- `determine_container_main_size`, definite arm: a row with an automatic
  width in definite space is fit-content sized,
  `min(max-content, max(min-content, available))`, from its items'
  intrinsic contributions. Stock summed flex bases, so a shrink-to-fit row
  of `flex: 1` items had no width, a row never grew to its min-content
  width, and a flex container that is a start-aligned item of a 100-point
  column was as wide as its unwrapped text (Chrome: 100, text wrapped).
  This one is ours: Exact gets fit-content from its own callers. A
  wrapping row that comes out wider than the space its lines were broken
  at is laid out again at its width (`compute_preliminary`).
- `intrinsic_line_item_sum` (the body of the stock intrinsic arm, now a
  function): in a row, an item with a definite flex basis that cannot flex
  to its contribution contributes its hypothetical size; a row has no
  clamping basis; a scroll container in a row is measured like any item;
  outer contributions are summed as they are. Stock went through a flex
  fraction and back, which lost a negative margin on an item with a zero
  inner basis (`width: 100px; flex-basis: 0; margin-right: -10px` beside a
  30-point item: Chrome 20, stock 30). `FlexItem::content_flex_fraction`
  is gone and `used_flex_basis_is_definite` is new.
- The same function: a stretched cross size is used for a contribution only
  from a definite cross size on a single line, without auto cross margins.
- `wrapping_row_min_content`: a wrapping row's min-content width is its
  widest item's own min-content contribution, and its max-content width is
  never narrower than that. Three items with `flex-basis: 60px` around
  100-point children in an 80-point box: Chrome 100 wide on three lines,
  stock 180 on one.
- `collect_flex_lines` and `compute_preliminary`: a row's max-content width
  is measured on one line also when it has a `max-width`; lines are then
  broken at the clamped width. Under min-content the row is laid out again
  at its resolved width. Line breaking sums items and gaps apart with
  1/1024 of a pixel of slack.

The fit-content arm made rows shrink in places where stock gave them the
whole offered width, so these follow from it (all ours, each a Chrome 152
row in the shrink-to-fit guard test):

- `measure_item_content_main_size` is the one measurement of an item's
  content: the base size step, the intrinsic sums and the wrapping row's
  min-content all measure in `SizingMode::ContentSize` with the cross size
  clamped by the item's cross minimum and maximum. Measuring in two sizing
  modes threw a node's cached sizes away at every level of nesting.
- Keyword sizes. A `min-content` or `max-content` width on a row's item is
  the constraint the item is measured under for its contribution
  (`main_size_keyword_constraint`), also when it has a flex basis; the same
  keyword as a flex basis does not replace the contribution. An item's
  keyword cross size is the space every measurement of it uses
  (`cross_size_keyword_available_space`), and a column's item with a keyword
  width is first measured for that width and then for its height at it
  (`column_item_keyword_width`).
- Cyclic percentages. An item's percentage minimum main size against an
  unknown container size is zero while the container is sized
  (`min_main_size_is_cyclic_percentage`). A row whose items have a
  percentage basis, width, minimum or maximum is laid out again once its
  width is known (`main_size_is_cyclic_percentage`), as is a wrapping row
  that came out at another width than its lines were broken at. A column
  whose width comes from its content measures items with a percentage
  width, minimum or maximum again once it has its width
  (`cross_size_is_cyclic_percentage`).
- `determine_container_cross_size`: the min-content width of a wrapping
  column is its widest line, not the sum of its lines.

Guard tests in the runtime's `layout_css_mode_tests.rs`, Chrome rows on
plain Taffy trees:

- `css_flex_taffy_fork_row_intrinsic_width_follows_chrome`: 39 rows.
- `css_flex_taffy_fork_wrapping_row_min_content_is_its_widest_item`: 38
  rows.
- `css_flex_taffy_fork_capped_row_wraps_at_its_clamped_width`: 16 rows.
- `css_fit_content_container_preserves_intrinsic_queries_and_size_limits`
  (`layout_fit_content_tests.rs`): its automatic-width case now expects
  Chrome's 100 instead of stock's 400.
- `css_flex_taffy_fork_shrink_to_fit_row_breaks_lines_at_its_own_width`:
  30 Chrome 152 rows for the items above.
- `css_flex_taffy_fork_nested_shrink_to_fit_rows_measure_each_level_once`:
  24 nested wrapping rows lay out in linear time.

## A height from bounds or from content is not a percentage basis

The aim of Exact's patches 43 and 44 (height bounds do not give a definite
height), written for this copy from Chrome 152 rows. The rows from Exact's
own tests for those two patches that do not need `aspect-ratio` already
matched here; the percentage cases below did not. The non-ratio half of
their patch 49 is already here (`item_known_dimension_definiteness`).

- `compute_flexbox_layout` (`src/compute/flexbox.rs`): a height the
  container has only because `max-height` is not above `min-height`, or one
  its parent measured from its content and passed as not definite, is a used
  height. `AlgoConstants::height_is_percentage_basis` is false for it, and
  `generate_anonymous_flex_items` and `determine_flex_base_size` then leave
  percentage heights, minimums, maximums and (in a column) flex bases of the
  items unresolved, so they behave as automatic. A stretched item's height
  is still definite for its own children. `min-height: 100px;
  max-height: 100px` row with a `height: 50%` item around 30 points of
  content: Chrome 30, stock 50. An absolutely positioned column still
  breaks its lines at its measured height.
- `src/compute/oof.rs`: an absolutely positioned box's height is passed as
  definite only when it was known before the box was measured (its own
  height, both insets, a ratio or a keyword). Stock passed a measured
  height as definite, so `position: absolute; max-height: 100px` around a
  `height: 50%` child gave the child half the content height in block and
  flex boxes alike.
- `item_known_dimension_definiteness`: in a column whose height is not a
  percentage basis, an item's flexed height is not definite for its own
  children either.
- `determine_container_cross_size`: a wrapping column's automatic width in
  definite space is fit-content, `min(lines, max(widest line, available))`.
  Stock gave the sum of the lines, wider than the space when one line fit.
- `collect_flex_lines`: a column bounded by `max-height` breaks its lines
  at that bound also when its known height is not definite, as it did when
  it was measured.

Guard test `css_flex_taffy_fork_height_bounds_do_not_size_an_intrinsic_width`
in the runtime's `layout_css_mode_tests.rs`: 118 Chrome rows (10 from
Exact's fixtures, Chrome 151; 108 measured in Chrome 152), 31 of the
first 99 differed before.

## A fit-content row's width is definite

The aim of Exact's patches 25 (its fit-content half) and 28: a column's
item that is not stretched is fit-content wide and as tall as its content
at that width. Their `common::fit_content_width` helper is not ported: with
the fit-content arm of `determine_container_main_size` (above) 180 of 192
Chrome 152 rows already matched. The 12 that did not had one cause, ours:

- `compute_preliminary` (`src/compute/flexbox.rs`): a row whose automatic
  width was resolved in definite space has a definite main size from then
  on (`AlgoConstants::has_definite_main_size`), so the widths its items flex
  to are definite. Stock kept it indefinite, and a wrapping row among the
  items, given a narrower width than its content, stayed on one line. A
  start-aligned row in a 200-point column around a wrapping row of two
  120-point boxes: Chrome 200 by 20, before 200 by 10.

Guard test `css_flex_taffy_fork_column_items_are_fit_content_wide` in the
runtime's `layout_css_mode_tests.rs`: 199 Chrome 152 rows.
