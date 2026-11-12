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
- `overflow_from_border_box` in the same file, called at every contribution
  site (block, flex, grid, absolute): a child's overflow rectangle is
  measured from its padding box and joined at its border-box location, so
  it is first moved by the child's start border. Stock left that out (a
  200-point descendant of a child with 10-point borders ended 10 short),
  which the earlier every-level end padding had hidden.
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
  its last descendants', 6 for a bordered or padded child's descendant, and 5 for a grid track floored by its item's
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

## Sizes go through `aspect-ratio` in one place

Exact's patch 12. Stock applied the ratio to the size, the minimum and the
maximum each on its own (`Size::maybe_apply_aspect_ratio`), before
`box-sizing`. That transferred a limit into an axis the box sizes itself
(a 200-point width with `max-height: 60px` became 120 wide), fixed a
derived height the content should be able to pass, and ignored padding in
a border-box ratio.

- `src/compute/ratio.rs` (new): `Ratio` holds the ratio and the inset
  between the border box and the box the ratio relates (`box-sizing`).
  `Ratio::resolve` takes the given border-box size, minimum and maximum:
  one dimension given is clamped by its own limits and gives the other,
  which is clamped on its own; two given ignore the ratio; a limit
  transfers only into a dimension that is not given; a derived height of a
  box that is not replaced and not a scroll container, with
  `min-height: auto`, is a minimum the content can pass (CSS Sizing 4
  §5.1). `sizes_through_ratio`, `container_sizes_through_ratio`,
  `transfer_into_unsized`, `resolve_through` and `percentage_height` are
  the forms the algorithms call.
- `src/compute/block.rs`: the container's own sizes, the incoming known
  dimensions and each block item's sizes go through it; the ratio height
  from a known width is the percentage basis.
- `src/compute/leaf.rs`: a leaf's sizes go through it; with neither
  dimension given the content width gives the height (a floor or a size as
  above). A replaced leaf keeps the stock floor until replaced sizing is
  ported.
- `src/compute/mod.rs`: the root's sizes.
- `src/compute/flexbox.rs`: the container's sizes; `compute_constants`
  transfers the incoming dimensions and reads the main size from the outer
  size (a ratio height in the inner size is a percentage basis, not a used
  height); `FlexItem::ratio` replaces the bare number; an item's limits
  transfer only into axes it does not size (`transfer_into_unsized`); a
  column item's width is left to follow its flexed height; a stretched
  cross size that is not definite is not transferred through the ratio
  when the item is measured.
- `src/compute/oof.rs`: an absolutely positioned box's sizes go through
  it; a height the ratio gives is not stretched between insets; with
  neither dimension known the shrink-to-fit width gives the height.

Rules Chrome follows that Exact's copy does not, found in review and
guarded by rows in `TAFFY_FORK_ASPECT_RATIO_ROWS`:

- A transferred minimum never passes the maximum of the axis it lands in
  (`Ratio::resolve`).
- `height` or `min-height` as a percentage with nothing to resolve against
  is not `auto`: the derived height is then a size, not a floor
  (`floors_height`). A content-size probe leaves the box's own `height`
  out, so it floors whatever that height is (`floors_height_in`).
- Percentage heights of a ratio box's children resolve against the ratio
  height within `min-height` and `max-height`, also when content made the
  box taller (`percentage_height`; `AlgoConstants::ratio_percentage_height`
  in flex; an absolutely positioned box passes its grown height as not
  definite). In a min-content height query (a column flex item's automatic
  minimum) they stay automatic.
- A flex container's ratio height is also where a wrapping column breaks
  its lines and the cross space a row sizes its items in; a ratio width
  from a set height is the column's cross size.
- An absolutely positioned box stretched between vertical insets: the
  width the ratio gives is clamped by `min-width` and `max-width` and gives
  the height back, unless `align-self: stretch` is set, which keeps the
  stretched height.

- A block container that finds its width from its content generates its
  items again against that width before it measures its height
  (`compute_inner`), so an item's size and its percentage padding are of
  one resolution: a content-box ratio item with `padding-left: 10%` in a
  shrink-to-fit parent was measured 20 points short.

### A ratio box with no width is at least as wide as its content

Exact's patches 50 and 64, with Chrome 152 rows in
`TAFFY_FORK_RATIO_MINIMUM_ROWS`.

- `ratio::minimum_ratio_width`: a non-replaced ratio box with a height,
  `width: auto`, `min-width: auto` and no scrolling overflow is at least
  the larger of its min-content width and the height's transfer, capped by
  `max-width` but never under its padding and border (`height:100px;
  aspect-ratio:2` with a 300 wide child is 300 wide, not 200). Called for a
  block item, also one whose height is `stretch` (`block.rs`, in the final
  layout and in the container's content width), a row flex item (`flexbox.rs`), and an absolutely
  positioned box (`oof.rs`).
- A row item's height from its ratio follows the width it is flexed to
  (`FlexItem::row_height_from_ratio`), unless it is stretched in a single
  line of definite height; the base size and minimum probes still see the
  height for its authored width.
- A `max-height` under the padding and border transfers as that much
  (`Ratio::resolve`, `transfer_into_unsized`); a minimum does not.
- A cross size the flex container's ratio fixes (`min-height` set, so not a
  floor) sizes a single line and stretches lines as a passed-in one does.
- A block whose height comes only from `max-height` not being above
  `min-height` is not a percentage basis for its children (as the flex
  container already was), and a child's `stretch` height is known while the
  block's content width is found (`block.rs`).

Left out: column flex items with a ratio keep their earlier sizing, and a
shrink-to-fit ratio box has no least width from its padding (UNIV-6229).
The minimum reads its box's min-content width, so where that is wrong for
a column flex container with ratio items inside, the minimum is too.
Two layouts are worse than before this change (UNIV-6230): a row item
with `width: min-content`, a ratio and `min-height` equal to `max-height`
under its padding is too wide, and a `height: stretch` ratio box whose
parent's height comes from content gets the content minimum.
Known differences left (UNIV-6228): an absolutely positioned box with
width, height and ratio all set keeps a content minimum in Chrome; a row
item's flex base size from a transferred `min-height`.

### Ratio sizes in grid

Exact's patches 12 and 56 at the grid sites, and 58. Grid kept the stock
`maybe_apply_aspect_ratio` after the block and flex sites moved to
`compute::ratio`, so a grid item with a ratio had a fixed ratio height its
content could not pass, limits transferred wrongly, and a border-box ratio
ignored its padding.

- `grid/mod.rs`: the container's own size, minimum and maximum go through
  `container_sizes_through_ratio`, as a block's and a flex container's do.
- `grid/types/grid_item.rs`: `known_dimensions` and `minimum_contribution`
  resolve through `Ratio` (`aspect_ratio_content_box` and
  `ratio_floors_unsized_height` are kept on the item). A height the ratio
  derives from the column width is a floor of the row, not a known height.
  A width the ratio transfers is not a preferred width for the column's
  minimum. The inline contributions take `minimum_ratio_width`. A ratio
  item's row contributions ask for its height as indefinite, not as
  min-content, so percentages in it resolve against the ratio height as in
  its final layout.
- `grid/alignment.rs`: the final size resolves through `Ratio` after each
  step that can give a width or a height; the padding and border floor is
  applied after the ratio, not before. An explicit `justify-self: stretch`
  gives the width on its own, with no ratio minimum; otherwise
  `minimum_ratio_width` is a floor of the width.

- A ratio grid container's ratio height is the space its rows are sized in
  (flexible rows share it), while content can still make it taller; not in
  a min-content height query, which is a column flex item's minimum.
- A `stretch` keyword size is given before limits transfer, so a
  `min-height` does not widen `width: stretch`. A `min-height` the ratio
  transfers is the item's minimum width for its column, as before the port.
- `Ratio::resolve` (shared with block and flex): a height under the padding
  and border is raised to them before it gives the width.
- An explicit `align-self: stretch` gives the height on its own, and the
  width then comes from it (Chrome: `aspect-ratio: 2` stretched in a 100
  point row is 200 by 100).
- A size the ratio derives is not a preferred size for the track minimum
  in either axis, so `min-height: 0` and a scroll container keep their zero
  minimum. Rows asked for under a min-content height grow to their content
  as under max-content (a grid has one content height), which a grid in a
  column flex container needs once an item's minimum can be zero.

Left out (UNIV-6231): a ratio item whose width comes from its content
(`start` or `center` alignment, or a keyword width) does not get its
height from that width; an empty padded border-box ratio item has no least
width from its padding; a measured width beside a set height (wrong in
block too); a few stretch cases. The ratio minimum reads its box's
min-content width, and a grid's min-content width counts an item with
`min-width: 0` or hidden overflow at its content width (Chrome: zero), so a
ratio grid with a height holding such an item is too wide, in a grid parent
now as in a block parent before. Three random layouts that
matched before differ now, all where Chrome sizes a column once and does
not follow the row afterwards: a ratio grid container holding a
`height: stretch` ratio item, an explicitly stretched item with both
`max-width` and `max-height` and wide content, and a `min-height: 0`
ratio item with tall content in flexible rows of a `min-height` grid.
Two more from review, both from a derived height no longer being a row
minimum: a hidden-overflow ratio item spanning two flexible rows makes them
too tall, and a centered `min-height: 0` ratio item with an auto top
margin that overflows its row is centered, not placed at the top.

Chrome 152, a 400-point grid, one `aspect-ratio: 2` item:

| Case | Chrome | Before |
|---|---|---|
| `width: 100px`, a 30 by 150 child | 100 by 150 | 100 by 50 |
| `padding: 10px` | 400 by 210 | 400 by 200 |
| `width: 100px; max-height: 30px` | 100 by 30 | 60 by 30 |
| `height: 100px`, a 300 wide child | 300 by 100 | 200 by 100 |

A later change (UNIV-6231, UNIV-6234) is about a ratio item whose width is
not stretched: a content keyword width (`max-content`, `min-content`,
`fit-content`), or an automatic width with `justify-self` other than
stretch or with automatic side margins. With `normal` block alignment
Chrome gives such an item the width its area's height gives through the
ratio. Stock sized it as for `start` in both axes, so an empty one was 0
by 0.

- `grid/alignment.rs`, `ratio_stretched_width`: the area's height, within
  the item's own `min-height` and `max-height` and not below its padding
  and border, through the ratio. It is the item's preferred width only.
  The height then follows the width within the limits as for any ratio
  box, and content can make the item taller than its area.
- `grid/types/grid_item.rs`, `block_contribution_known_dimensions`: such
  an item gives its rows the height its own content width gives through
  the ratio, in its min-content and max-content contributions and so in
  its automatic minimum. Stock gave the content's height alone, so a
  `max-content` row around `aspect-ratio: 2; width: max-content` and a
  300-wide child was 10 tall where Chrome has 150.
- With the rows sized that way there is no separate rule for content: the
  item is as wide as its area's height gives. A row that takes the item's
  contribution is tall enough for the content width; a row that does not
  (a set length, `minmax(0, 1fr)`, `min-height: 0` under an `auto` row
  that a set grid height fills, a span across flexible rows) leaves the
  item narrower than its content, as in Chrome. An earlier version of
  this change kept the row sizes and floored the width by the content
  instead; four review rounds each found a row form that floor got
  wrong.

| Case (grid 400 wide) | Chrome | Before |
|---|---|---|
| `aspect-ratio: 2; width: max-content`, grid 300 tall | 600 by 300 | 0 by 0 |
| `aspect-ratio: 0.5; max-width: 120px; justify-self: start`, grid 300 tall | 120 by 240 | 0 by 0 |
| `aspect-ratio: 4; justify-self: start`, a 60 by 60 child | 240 by 60 | 60 by 60 |
| `aspect-ratio: 2; width: max-content`, a 40px row, an 80 by 120 child | 80 by 120 | 80 by 40 |

Of 29,172 Chrome 152 layouts (the picture set of "Replaced boxes" below),
1,488 differed before this change and 1,112 differ now: 376 now match and
none match no longer.

Guard test `css_aspect_ratio_grid_taffy_fork_matches_chrome`: 895 Chrome
152 rows; 137 of the first 382 differed before, and of the last 452, 119
differed before the later change, 15 are layouts earlier versions of it
broke, and 318 are review probes (each kind of row with and without a
grid height, spans, each width form, each minimum; 222 of the last 248
differed before). Two random comparisons, 4,890 grid layouts: 1,643 that differed
before now match, 5 that matched before differ (UNIV-6231).


Chrome 152, a 400-point parent, `aspect-ratio: 2`:

| Case | Chrome | Before |
|---|---|---|
| block, `width: 200px; height: 50px` | 200 by 50 | 200 by 100 |
| `width: 200px; max-height: 60px` | 200 by 60 | 120 by 60 |
| `height: 100px; max-width: 150px` | 150 by 100 | 150 by 75 |
| `width: 200px`, a 150-point child | 200 by 150 | 200 by 100 |
| flex row, `flex-grow: 1`, no sizes | 400 by 200 | 400 by 0 |
| absolute, no sizes, a 60 by 10 child | 60 by 30 | 60 by 10 |

Guard test `css_aspect_ratio_taffy_fork_matches_chrome` in the runtime's
`layout_css_mode_tests.rs`: 175 Chrome 152 rows (block children, flex
items in rows and columns, containers with children, absolute boxes); 60
of them differed before.

## Replaced boxes in shrink-to-fit parents

Exact's patch 36. Exact's patches 5 and 37 to 39 size a replaced leaf by
the CSS 2.1 section 10.4 table inside Taffy's leaf code; they are not
ported, because the runtime sizes a picture in its own resolver
(`css_layout/replaced.rs`), which already follows that table: 6,829 of
7,860 Chrome 152 picture layouts matched before this port.

- `block.rs`, `BlockItem::compressed_min_content_width` and
  `determine_content_based_container_width`: a compressible replaced child
  whose `width` or `max-width` is a percentage gives its authored
  `min-width` (percentages against zero, as a border box) as its
  min-content contribution, not its natural width and not a minimum
  transferred from its height
  (<https://drafts.csswg.org/css-sizing-3/#replaced-percentage-min-contribution>).
  In a definite space the same child is no wider than the space unless
  its compressed minimum is, which is what shrink-to-fit gives a parent
  whose only wide content is that picture. Exact's patch covers the
  min-content query only.
- `grid/mod.rs`, the column rerun check: an item whose max-content
  contribution was used and has changed after the rows were sized reruns
  column sizing, as a changed min-content contribution does. A block
  holding a `width: 100%; height: 50%` picture keeps a zero min-content
  contribution, while its max-content one follows the height the rows
  resolve. Two random sets of 4,890 grid layouts without pictures are
  unchanged by this.
- `grid/types/grid_item.rs`, `minimum_contribution`: the preferred and
  maximum sizes that cap a replaced item's content-based minimum are
  border-box sizes (stock Taffy compared a content-box `max-width` with a
  border-box contribution), and a minimum the ratio transfers does not
  replace a replaced item's content-based minimum.

- `flexbox.rs`, `determine_flex_base_size` and the second pass in
  `compute_preliminary` (ours, not from Exact): a stretched replaced item
  takes the cross space offered as its size, and its other dimension
  through its natural ratio, only when that stretch is real. In a row that
  is a definite container height on a single line (CSS Flexbox 1 section
  9.2 step 3B). Stock Taffy used the offered space, clamped by the item's
  own limits, whenever the item was stretched: a 40 by 40 picture with
  `padding: 10px; max-height: 200px` in a flex row without a height was
  220 by 220 (Chrome: 60 by 60). A column's width is its inline size,
  which a browser knows before it lays out the items, so there a width
  passed in is real on a single line, and a width that will come from the
  content is found first: the column is laid out a second time at that
  width, the way it already is for items with percentage widths, and a
  request for its height alone waits for that pass. In the first pass a
  replaced item gives the width of its own height (not of the height
  flexing gave it), and its percentage limits do not resolve against a
  height the column has from its content. A `stretch` keyword on the
  cross size is real whenever the container's cross size is definite.
  Kept as before: a replaced item whose `width` or `max-width` is a
  percentage, in a column that shrinks to fit under a min-content width
  or in a space narrower than the item (its contribution there is its
  authored minimum, as in a block parent); and a container with an
  `aspect-ratio` of its own.

Of 19,688 Chrome 152 picture layouts (7,743 saved from the port, 11,663
new random flex layouts in three sets, and the review cases), 2,267
differed before this change and 1,014 differ now: 1,263 now match and 10
match no longer. Each of the ten matched before only because the offered
space happened to be the size Chrome treats as definite (UNIV-6234): a
flex container with a keyword width (`fit-content`, `max-content`) that
is a grid item, where grid measures the width before the item is
stretched to its area's height (7); a row inside a column sized by the
column's own `aspect-ratio` (2); and a percentage `max-height` on a
picture in a content-sized column inside a column with a set height (1).

A later change takes up the first group (UNIV-6234):

- `grid/alignment.rs`: a grid item whose width is a keyword that is
  measured (`fit-content`, `max-content`, `min-content`) and that will be
  stretched to its area's height is measured at that height, as the track
  contributions already are. A flex row holding a stretched picture is as
  wide as the picture is at the area's height (Chrome).
- `flexbox.rs`, `FlexItem::flexed_height_awaits_width`: a column whose
  width comes from its content is as wide as its items are at their own
  heights, not at the heights flexing gave them (Chrome): a
  `height: 80px; aspect-ratio: 2` item shrunk to 40px still makes the
  column 160px wide. The item is measured at its own height while the
  column finds its width, and the existing second pass then gives it the
  width of its flexed height. A height that is a percentage or a keyword,
  and an item whose width is a percentage of the column's, keep the
  flexed height as before. Without this, measuring a grid item at its
  stretched height narrowed or widened columns of ordinary boxes.

Of 29,157 Chrome 152 layouts (the 19,688 above, 4,890 grid layouts
without pictures, and 4,579 new random layouts of flex and grid boxes
inside grid items and set-height parents), 2,623 differed before this
change and 2,086 differ now: 542 now match and 5 match no longer. Six of
the seven grid layouts above match. The seventh and the five lost came
from one cause, which the next change takes up.

A third change is about heights that are known but not set (UNIV-6234).
Chrome measures a box's width before its parent has stretched it, and
keeps that width unless something inside depends on the height by a
percentage. We pass the stretched height into the measurement, so we say
which kind of height it is:

- `LayoutInput::known_height_is_stretch_only` (new, part of the measure
  cache key): the known height is one the node is stretched to by a
  parent that has not fixed it by a set size. Grid sets it when it
  measures an item's width and a row the item spans has no set length
  for its minimum and maximum (`GridItem::rows_are_set`); a column sets
  it when it measures the width of an item with an automatic height at
  the item's flexed height.
- `flexbox.rs`: under that flag a picture's `height: stretch` in a row
  does not resolve (`stretch_height_is_withheld`: the picture is
  measured as if the row had no height), and a column does not resolve
  `height: stretch` items while it finds its width. Items still stretch
  to the height, and percentage heights still resolve. When the measured
  width of an item comes from the height, the flag is dropped: Chrome
  measures again in that case, and the keyword resolves. That is an item
  whose height follows the container (a percentage height, minimum or
  maximum, or an automatic height stretched across a row) and whose
  width follows its height (a picture, or a ratio item with an automatic
  width). A ratio item's `height: stretch` counts; a picture's does not.
  A height the node's own style sets as a length is not stretch-only; a
  percentage height, a `min-height` or a `max-height` leaves it so.
- `flexbox.rs`: where the column's height is set (a set grid row, the
  column's own `height`), a `height: stretch` item is measured at that
  height while the column finds its width, not at its flexed height.
- `flexbox.rs`, `probe_definiteness`: the stand-in height a wrapping row
  offers an item before its lines are sized is passed as known but not
  definite, so nothing inside the item resolves against it.

Of 29,172 Chrome 152 layouts (the 29,157 above and 15 probes of set and
unset rows), 2,090 differed before this change and 1,943 differ now: 148
now match and 1 matches no longer (a picture with a percentage
`max-height` in a row that is an item of a wrapping row with a set
height: Chrome resolves the percentage against the natural line). All
seven grid layouts and the five lost above match.

Left out: an automatic-width grid item with `justify-self: start` holding
a picture with `height: stretch` is still measured at the stretched
height.

A fourth change is about percentage limits against a height that comes
from content (UNIV-6234). A flex container with no set height took its
height from its items and then resolved an item's percentage `max-height`
or `min-height` against that height, so a picture with `max-height: 50%`
in a plain column came out half as tall as in Chrome. A percentage against
a height that depends on content is automatic (CSS 2.1 10.7).

- `flexbox.rs`, `laid_out_percent_resolution_size`: the size an item's own
  percentages resolve against once the container is sized. Its height is
  kept only when the container's height was definite before its items were
  measured. Used for the final layout of each item, for the cross size
  measure, and for the maximum the stretch step clamps to.
- `flexbox.rs`, `row_height_is_item_percentage_basis`: a row's height that
  is known but not definite is not passed as the parent height while an
  item's flex base size and content size are measured.

Of the 29,172 Chrome 152 layouts above, 1,943 differed before this change
and 1,722 differ now: 220 now match and 2 match no longer (a column that
is a grid item in an `auto` column, stretched to its row, holding a
picture with a percentage `max-height`: Chrome measures the column's
width against the stretched height, and before this change the percentage
resolved by accident).

A fifth change is about pictures with a percentage width inside a flex
container sized by its content (UNIV-6234). A picture whose `width` or
`max-width` is a percentage adds nothing to its container's min-content
width beyond its own minimum (CSS Sizing 3 5.2.2, compressible replaced
elements). Stock measured it at its natural width, so a column inside a
100 wide row stayed 120 wide around a 120 wide picture with
`width: 100%`, where Chrome gives 100.

- `flexbox.rs`, `compressed_item_parent_size`: while a flex container with
  no known width is measured under a min-content width, such an item is
  given a parent width of zero. Used for the flex base size (not for a row
  item that cannot shrink: it contributes its base size), the content main
  size, and a column's cross size measure.
- The runtime's picture resolver (`css_layout/replaced.rs`) applies a
  percentage width or max-width of a zero parent width also under
  `SizingMode::ContentSize`, which otherwise ignores the authored width.

Of the 29,172 Chrome 152 layouts above, 1,722 differed before this change
and 1,488 differ now: 234 now match and none match no longer.

Left out (UNIV-6234): the same pictures where the container is sized by
its own `aspect-ratio` or the picture's width follows a stretched height.

A sixth change is about a `stretch` keyword height on a row item whose
alignment is not stretch (UNIV-6234). Stock stretched an item's height
before measuring its width only when its alignment was `stretch`, so
under `align-items: center` or `flex-start` an `aspect-ratio: 2; height:
stretch` item in a 100-tall row had the right height and a width of 0
where Chrome has 200, and a 60 by 120 picture stayed 60 wide where Chrome
has 50.

- `flexbox.rs`, `determine_flex_base_size` and
  `measure_item_content_main_size`, `keyword_stretches`: the keyword
  stretches the item whatever its alignment is, in its flex base size and
  in the row's own content width.
- A picture's does so only in its flex base size and only once the row
  has its width. Chrome measures a row that is an item of a column with
  the picture at its natural width, and a row measured anywhere else with
  the picture at the width its height gives. This copy cannot tell the
  two rows apart, so a measured row keeps the picture's natural width in
  both, as before.

Of 29,742 Chrome 152 layouts, 1,246 differed before this change and 1,115
differ now: 131 now match and none match no longer.

Left out (UNIV-6234): a row whose width is measured around a picture
with a `stretch` keyword height, outside a column (the row keeps the
picture's natural width); the same items in columns, block parents and
grid.

Left out (UNIV-6234): the same percentage limits on pictures inside grid
and block parents, and inside containers sized by their own
`aspect-ratio`; pictures with percentage
sizes inside nested flex or grid boxes that shrink to fit; a picture with
`justify-self: stretch` in a block parent; opposing minimums and maximums
with an authored `aspect-ratio` on a picture in flex; pictures in
wrapping flex containers.

Guard test `css_replaced_taffy_fork_matches_chrome`: 1,372 Chrome 152 rows
(285 from the port, 163 of which differed before it; 235 for the
stretched picture change, all of which differed before it or were lost
and regained in review; 195 for the grid item change, all of which
differed before it or are review cases; 177 for heights that are known
but not set, 138 of which differed before it, the rest probes and
review cases; 100 for percentage limits against a content height, all of
which differed before it; 128 for percentage widths in a box sized by its
content, 104 of which differed before it, the rest layouts an earlier
version of the change broke; 252 for the `stretch` keyword height in a
row, 131 of which differed before it, the rest probes).
