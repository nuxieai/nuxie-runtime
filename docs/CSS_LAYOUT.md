# CSS layout contract

Status: accepted. Step 1 is built in the runtime (branch `claude/browser-css-layout`).
Plan: browser-like layout on Taffy 0.15, step 1 (runtime). Refs [UNIV-5012](https://universe.basis.dev/issue/UNIV-5012).

This is the contract between the runtime (this repository) and the nuxie-dev compiler. The compiler writes what this document describes, and the runtime lays out and paints exactly that. Everything here is our layer over Rive, not Rive-native, unless a line says Rive-native.

Words used below:

- **box**: a `LayoutComponent` (an `Artboard` is the root box).
- **participant**: a `LayoutParticipant`, which makes a non-layout object (a shape, an image, text) an item of its parent box.
- **content**: the non-layout children a box measures (text, images, shapes, groups, n-slices, joysticks, nested artboards).
- **paint**: what a box draws for itself (fills, strokes, shadows, clip). Paint never counts towards a size.
- **CSS mode**: a box (or participant) whose layout comes from a CSS record (section 2). **Translated mode**: one whose layout comes from Rive's Hug, Fixed and Fill.

## 1. Keys

### 1.1 Why a new record type

`nuxie-codegen` accepts Nuxie definitions only as whole types in `defs/nuxie/`. It refuses a Nuxie file that shadows an upstream definition, so we cannot add properties to Rive's `LayoutComponentStyle` or `LayoutSizingStyle`. The CSS properties therefore live on one new record type, `LayoutCssStyle`.

It follows the precedent of Rive's `GridItemPlacement`:

- It is a component whose parent is the box, or the object that hosts a participant.
- It registers itself as one more style applier on that box or participant.

Its presence is also the switch between CSS mode and translated mode (section 2). Upstream may later start reading its unused flex keys (520 to 522, 600 to 603); this record never collides with them.

Allocation, in a block of its own that `defs/nuxie/README.md` reserves for CSS layout, box paint and CSS text (types 61000 to 61099, properties 61000 to 61999), so other Nuxie definitions, which allocate one after another from 60000, never reach it:

- **Type keys**: 61000 to 61005.
- **Property keys**: 61000 to 61105.

The definition files live in `defs/nuxie/` beside `video.json`, and the README's allocation list names them.

| Type | Key | Extends | Parent | Rive-native or ours |
|---|---|---|---|---|
| `LayoutCssStyle` | 61000 | `component.json` | a box, a `NestedArtboardLayout`, or a participant's host; at most one per parent | ours |
| `BoxBorder` | 61001 | `component.json` | a `Stroke` painted by a box | ours |
| `BoxGradient` | 61002 | `component.json` | a `LinearGradient` or `RadialGradient` painted by a box | ours |
| `BoxShadow` | 61003 | `component.json` | a `Feather` on a `Fill` painted by a box | ours |
| `BoxPaint` | 61004 | `component.json` | a box or a CSS artboard; at most one per parent | ours |
| `TextStyleCssText` | 61005 | `component.json` | a `TextStylePaint` | ours |

### 1.2 Value encodings

Three conventions cover every value below.

- **Enums** are `uint8`. Value 0 is always the property's CSS initial value. A record with a field left out therefore means the CSS initial value, never Rive's.
- **Length-percentage**: two `double` fields, `<name>` (points) and `<name>Percent` (a CSS percentage, so 50 means 50%). The used value is `length + percent / 100 * basis`, where the basis is CSS's for that property. This one form covers:
  - `24px`: length 24, percent 0.
  - `50%`: length 0, percent 50.
  - `calc(50% - 12px)`: length -12, percent 50.
  - `calc((100% - 24px) / 3)`: length -8, percent 33.333333.
- **Kind**: a `uint8` beside a length-percentage, for properties that also take keywords. It says which form the value has; the two numbers are read only when the kind is a length-percentage or `fit-content(<length-percentage>)`.

What `calc()` may contain:

- Accepted: any expression the compiler can reduce to `<percent> + <length>`.
- Refused: `min()`, `max()`, `clamp()`, a product or quotient of two lengths, and viewport units (`vw`, `vh`, `vmin`, `vmax`).
- Font-relative units (`em`, `rem`, `ch`, `ex`) are resolved to points by the compiler before encoding, or refused when it cannot know the font size.

**Mixed values are deferred.** Step 1 resolves a length-percentage only when at least one of its two parts is 0, so it is a plain length or a plain percentage. A value with both parts non-zero (a mixed `calc()`) is refused at load as unsupported, like the record's other refused forms: the import fails. The compiler refuses a mixed `calc()` with a clear error. The encoding above stays, so the format does not change when mixed values arrive.

Mixed values come in a separate runtime change after steps 1 and 2 land. Taffy resolves a `calc()` value through its tree's `resolve_calc_value`, and Taffy's own `TaffyTree` answers 0 there, so that change drives Taffy through the runtime's own tree view (section 3).

### 1.3 `LayoutCssStyle` properties

Doubles are bindable and animate. Enums are bindable and do not animate.

| CSS | Keys | Type | Values | CSS initial (value 0 or default) |
|---|---|---|---|---|
| `display` | 61000 `display` | uint8 | 0 block, 1 flex, 2 grid, 3 none, 4 flow-root | `inline` in CSS, which we do not have; 0 block is the `div` default |
| `box-sizing` | 61001 `boxSizing` | uint8 | 0 content-box, 1 border-box | content-box |
| `position` | 61002 `position` | uint8 | 0 static, 1 relative, 2 absolute | static |
| `z-index` | 61099 `zIndexKind`, 61100 `zIndex` | uint8, double | kind 0 auto, 1 an integer (the double, rounded) | auto |
| `overflow-x` | 61003 `overflowX` | uint8 | 0 visible, 1 clip, 2 hidden, 3 scroll, 4 auto | visible |
| `overflow-y` | 61004 `overflowY` | uint8 | as overflow-x | visible |
| `direction` | 61005 `direction` | uint8 | 0 inherit, 1 ltr, 2 rtl | inherited (ltr at the root) |
| `flex-direction` | 61006 `flexDirection` | uint8 | 0 row, 1 row-reverse, 2 column, 3 column-reverse | row |
| `flex-wrap` | 61007 `flexWrap` | uint8 | 0 nowrap, 1 wrap, 2 wrap-reverse | nowrap |
| `flex-grow` | 61008 `flexGrow` | double | number | 0 |
| `flex-shrink` | 61009 `flexShrink` | double | number | 1 |
| `flex-basis` | 61010 `flexBasisKind`, 61011 `flexBasis`, 61012 `flexBasisPercent` | uint8, double, double | size kinds, plus 6 content | auto |
| `align-items` | 61013 `alignItems` | uint8 | item alignment | normal |
| `align-self` | 61014 `alignSelf` | uint8 | item alignment, 0 auto | auto |
| `justify-items` | 61015 `justifyItems` | uint8 | item alignment | legacy (behaves as normal) |
| `justify-self` | 61016 `justifySelf` | uint8 | item alignment, 0 auto | auto |
| `align-content` | 61017 `alignContent` | uint8 | content alignment | normal |
| `justify-content` | 61018 `justifyContent` | uint8 | content alignment | normal |
| `grid-auto-flow` | 61019 `gridAutoFlow` | uint8 | 0 row, 1 column, 2 row dense, 3 column dense | row |
| `aspect-ratio` | 61020 `aspectRatio` | double | width over height; 0 auto | auto |
| `width` | 61021 `widthKind`, 61022 `width`, 61023 `widthPercent` | uint8, double, double | size kinds | auto |
| `height` | 61024 `heightKind`, 61025 `height`, 61026 `heightPercent` | uint8, double, double | size kinds | auto |
| `min-width` | 61027 `minWidthKind`, 61028 `minWidth`, 61029 `minWidthPercent` | uint8, double, double | 0 auto, 1 length-percentage, 2 percentage (as size kind 7) | auto (the browser's automatic minimum) |
| `min-height` | 61030 to 61032, as min-width | | | auto |
| `max-width` | 61033 `maxWidthKind`, 61034 `maxWidth`, 61035 `maxWidthPercent` | uint8, double, double | 0 none, 1 length-percentage, 2 percentage (as size kind 7) | none |
| `max-height` | 61036 to 61038, as max-width | | | none |
| `margin-top`, `-right`, `-bottom`, `-left` | 61039 to 61041, 61042 to 61044, 61045 to 61047, 61048 to 61050: `margin<Side>Kind`, `margin<Side>`, `margin<Side>Percent` | uint8, double, double | 0 length-percentage, 1 auto | 0 |
| `top`, `right`, `bottom`, `left` | 61051 to 61053, 61054 to 61056, 61057 to 61059, 61060 to 61062: `inset<Side>Kind`, `inset<Side>`, `inset<Side>Percent` | uint8, double, double | 0 auto, 1 length-percentage | auto |
| `padding-top`, `-right`, `-bottom`, `-left` | 61063 and 61064, 61065 and 61066, 61067 and 61068, 61069 and 61070: `padding<Side>`, `padding<Side>Percent` | double, double | length-percentage | 0 |
| `border-top-width`, `-right-`, `-bottom-`, `-left-` | 61071 `borderTop`, 61072 `borderRight`, 61073 `borderBottom`, 61074 `borderLeft` | double | points | 0 (CSS's `medium` is used only when a border style is set; the compiler writes the used width) |
| `row-gap` | 61075 `rowGap`, 61076 `rowGapPercent` | double, double | length-percentage | normal (0 in flex and grid) |
| `column-gap` | 61077 `columnGap`, 61078 `columnGapPercent` | double, double | length-percentage | normal (0) |
| `border-*-radius`, percent part | 61079 `radiusTopLeftPercent`, 61080 `radiusTopRightPercent`, 61081 `radiusBottomRightPercent`, 61082 `radiusBottomLeftPercent` | double | CSS percentage | 0 |

Size kinds (`width`, `height`, `flex-basis`):

| Value | Meaning |
|---|---|
| 0 | auto |
| 1 | length-percentage |
| 2 | min-content |
| 3 | max-content |
| 4 | fit-content |
| 5 | `fit-content(<length-percentage>)` |
| 6 | `content` (flex-basis only) |
| 7 | a percentage: the percent field alone, also when it is 0. `0%` and `0px` differ against an indefinite size: `flex: 1` writes `flex-basis: 0%`, which there is the content's size, not 0. Kind 1 with a non-zero percent part still reads as that percentage |

Item alignment (`align-items`, `align-self`, `justify-items`, `justify-self`):

| Value | Meaning |
|---|---|
| 0 | the initial value (normal, auto or legacy, as above) |
| 1 | normal |
| 2 | start |
| 3 | end |
| 4 | flex-start |
| 5 | flex-end |
| 6 | self-start |
| 7 | self-end |
| 8 | center |
| 9 | baseline |
| 10 | stretch |

Content alignment (`align-content`, `justify-content`):

| Value | Meaning |
|---|---|
| 0 | normal |
| 1 | start |
| 2 | end |
| 3 | flex-start |
| 4 | flex-end |
| 5 | center |
| 6 | stretch |
| 7 | space-between |
| 8 | space-evenly |
| 9 | space-around |

Grid placement and tracks keep Rive's own records in both modes (Rive-native):

- `GridItemPlacement` (1068): line, and span per axis.
- `GridTrack` (1058): per entry, its list (template columns, template rows, auto columns, auto rows) and a size of auto, points, percent or fr, with an optional max.

The compiler expands `repeat(<integer>, ...)` into entries.

### 1.4 Refused in step 1

The compiler reports these as unsupported instead of writing them:

- `display` inline, inline-block, inline-flex, inline-grid, contents, any table or list-item value.
- `position` fixed and sticky. `float` and `clear`.
- `width` and `height` `stretch`.
- `min-*` and `max-*` keywords other than auto and none. Taffy's minimum and maximum take only lengths, percentages and `calc()`.
- Alignment with `safe`, `unsafe`, `first baseline`, `last baseline`, `left`, `right`.
- Grid:
  - `subgrid`, `masonry`;
  - named lines and areas;
  - `repeat(auto-fill)`, `repeat(auto-fit)`;
  - tracks sized `min-content`, `max-content` or `fit-content()`.
- `overflow` values beyond the five above; `scrollbar-gutter`; `overflow-clip-margin`.
- The `calc()` forms in 1.2, and in step 1 any length-percentage with both parts non-zero (refused at load).
- Enum values outside the ranges in 1.3 (refused at load).

## 2. CSS mode or translated mode

### 2.1 The switch

A box, or a participant, is in CSS mode exactly when its parent relationship gives it a `LayoutCssStyle`: the record is a child of the box, or of the participant's host. Otherwise it is in translated mode. The choice is per node: a CSS box can sit inside a translated box and the other way round, because Taffy reads each node's style on its own.

In CSS mode the runtime builds the node's whole Taffy style from the record (fields left out take their CSS initial value). It ignores the layout fields of the box's `LayoutComponentStyle`, `LayoutComponent` width, height and fractional sizes, and `LayoutComponent.clip`. These Rive keys still apply in CSS mode:

- the `LayoutComponentStyle` corner radii in points (paint, section 4);
- its layout animation settings (animation style, interpolation, interpolator and time);
- `GridTrack` and `GridItemPlacement`.

On a top-level `Artboard`, its own width and height define the root box, with its existing intrinsic sizing behaviour. Its record contributes container properties: display, padding, border, gap, direction, container alignment, wrap, overflow and grid flow. Its item sizes, limits, margins, insets and aspect ratio do not apply.

A hosted artboard with a record can instead be a CSS box in its host's layout:

- An `ArtboardComponentList` row takes both container and item properties from its own artboard's record.
- A `NestedArtboardLayout` with a child `LayoutCssStyle`, hosting an artboard with its own record, uses the copy's child record in place of the artboard's record for that occurrence's whole box: container and item properties, including `display: none`. The compiler writes the component root's declarations with the copy tag's declarations on top into this one record. The runtime does not merge the two records; omitted fields take their CSS initial values. Bound writes to the copy's record update that copy and relayout its host. Paints and paint records are unchanged.

Item properties are box-sizing, position, insets, size kinds, minimum and maximum sizes, margins, flex-grow, flex-shrink, flex-basis, align-self, justify-self and aspect-ratio. Omitted fields take their CSS initial values, including on a nested copy's whole-box record. These hosted roots size from CSS and their content, ignoring stored artboard width and height and Rive instance widths, heights, units and scale types. Grid placement records still apply. Lists estimate rows not yet built from stored artboard sizes.

A hosted artboard without its own record, or a `NestedArtboardLayout` without a child record, keeps its existing sizing. A CSS artboard hosted by a `NestedArtboardLayout` without that child record still contributes only container properties.

On a participant the record contributes only item properties: display none, position, insets, size, minimum, maximum, aspect ratio, margins, flex-grow, flex-shrink, flex-basis, align-self and justify-self. Participants have no children to lay out.

What the file must hold:

- Every box keeps its `LayoutComponentStyle` (its `styleId`), as today: a `LayoutComponent` without one does not lay out, in either mode.
- A record comes after its parent in the file, as every child does.
- `position` is CSS's: a `static` box ignores its insets and is not the containing block of an absolute descendant, which is placed against its nearest positioned ancestor (or the artboard).

At run time (animation, binding, scripts) values are stored as written. An enum outside its range reads as its initial value, and a length-percentage made mixed uses its percentage alone, until mixed values arrive (1.2).

### 2.2 Translation for files made in Rive

A box without a record gets this CSS. The runtime builds it in memory and runs the same CSS-to-Taffy code as CSS mode. Nothing is written back.

| Rive | CSS it becomes |
|---|---|
| any box | `box-sizing: border-box` (how Rive sizes boxes) |
| `displayValue` none | `display: none` |
| `layoutTypeValue` flex, grid | `display: flex`, `display: grid` |
| `layoutTypeValue` stack | Rive's stack as today: a one-cell grid. Rive-only, no CSS form |
| Fixed, either axis | `width` (or `height`) from `LayoutComponent` width with its units (points or percent) |
| Hug, either axis | `width: fit-content`, or `max-content` when `hugUnbounded` is set |
| Legacy hug: Fixed on both axes, `intrinsicallySized`, and a leaf | Hug on both axes |
| Fill on the main axis | `flex: <fraction> 1 0%`: flex-grow is the fractional size, flex-shrink 1, flex-basis 0%; size auto. Siblings share the space by fraction, whatever their content |
| Fill on the cross axis | `align-self: stretch`, size auto |
| Fixed or Hug on the main axis | flex-grow 0, flex-shrink 1, flex-basis auto |
| `flexBasis` authored as points or percent, on a Fill | that `flex-basis` in place of 0%, as Rive reads it only for Fill; Fixed and Hug keep `flex-basis: auto` (the Rive editor writes 0 points on every style) |
| min and max sizes | `min-*` and `max-*` with their units; an unset minimum is `auto`, the browser's automatic minimum (no longer Yoga's zero) |
| `layoutAlignmentType` | `justify-content` and `align-items` (and `align-content` from the cross value). Main axis: start, center, end or space-between. Cross axis: flex-start, center or flex-end. As today's mapping |
| a grid parent | its items keep flex-grow 0 and flex-shrink 0, `justify-self` from `justifySelfValue`, and stretch when the item Fills, as today |
| absolute position | `position: absolute`; Fixed and Fill keep their stored units, Hug stays content-sized |
| margin, padding, border, gap, insets, aspect ratio, direction, wrap, flex direction | the same values and units |
| `overflowValue` hidden, scroll | `overflow: hidden`, `scroll` |
| visible with `LayoutComponent.clip` | `overflow: clip`; a clip is not a scroll container, so the content minimum stays |
| a `ScrollConstraint` viewport | `overflow: scroll` on its axes (and visible becomes auto on the other axis, as CSS computes) |
| a `ScrollConstraint`'s content box | on each scroll axis where the content box Fills, `min-height: 0` (or `min-width: 0`) in place of the automatic minimum; an authored minimum stays. A Rive author who Fills scroll content means "as big as the viewport, the items scroll", and a zero minimum is how CSS says that there: without it the content's own minimum grows the box past its viewport, as `min-height: auto` does to a flex item in a scroll container, and a virtualized list realizes more items than Rive does. A Hug or Fixed content box keeps its automatic minimum, so it is never shrunk below its content. CSS-mode boxes keep `min-*: auto` |
| `LayoutComponent.clip` | the clip itself stays Rive's: the outer edge of the box |

The box sizes used:

- A Fixed axis uses `LayoutComponent` width and height.
- A Fill axis uses the box's `fractionalWidth` or `fractionalHeight`.
- An artboard override replaces both, as today.
- Participants translate the same way from their `LayoutNodeStyle` width, height and fractional sizes.

## 3. Taffy features

Switched on, beside today's `std`, `taffy_tree`, `flexbox`, `grid` and `detailed_layout_info`:

| Feature | Why |
|---|---|
| `block_layout` | `display: block` and `flow-root`, the browser default for a `div`, with margin collapsing |
| `content_size` | a node's scrollable overflow rectangle, `scroll_width` and `scroll_height`, for scroll ranges; the local Taffy fork counts in-flow children's margin boxes (see [taffy-fork.md](taffy-fork.md)) |

Left off: `float_layout` (floats are refused), `flexbox_balance` and `parse` (the compiler encodes values; the runtime never parses CSS), and, in step 1, `calc`.

`calc` waits for mixed length-percentages (1.2). `TaffyTree::resolve_calc_value` always returns 0, so that later change implements Taffy's layout traits over the runtime's own tree for the solve:

- keeping the node storage and styles it has today;
- keeping a cache and the unrounded layouts in each node's context;
- calling Taffy's compute functions for each display mode.

It changes no part of the file format.

**Block flow.** A CSS box with `display: block` lays its layout children out in Taffy's block flow, and its leaf content is measured as section 5 says. Translated boxes never use block flow.

**Scroll containers.**

- A viewport box with `overflow` hidden, scroll or auto is a Taffy scroll container. It gets no automatic minimum, and its scrollbar width is 0.
- Rive's `ScrollConstraint` keeps moving one content box inside it.
- Scroll mode is fixed when the constraint's viewport is mounted during load. A CSS viewport without virtualization or infinite scrolling uses the CSS scroll axis; every other viewport uses native scrolling. Later `virtualize` and `infinite` writes on a loaded CSS viewport are ignored with a debug-build log, including when it loaded in native mode. Translated viewports keep native flag changes. Size changes, including zero size, never change the decided mode. The CSS scroll rules below apply to the fixed CSS scroll axis.
- In CSS mode the compiler gives that content box CSS that does not shrink to the viewport: `display: block` for a vertical list, or a row with `width: max-content` (or `flex: none`) and items at `flex-shrink: 0` for a horizontal one.
- The scroll range of a CSS viewport is Taffy's `scroll_width` and `scroll_height` of that viewport, replacing the content size minus viewport size minus end padding used today.
- Taffy is a local fork ([taffy-fork.md](taffy-fork.md)) whose scrollable overflow counts in-flow children's margin boxes at their in-flow position (block flow counts its end after the trailing collapsed margins), so a last item's end margin extends the range as Chrome's does, through the content box too. The scroller's end padding is added after all overflow, including descendants that overflow their item, where Chrome adds it after in-flow items only; the fork's notes list the cases that differ.
- A right-to-left CSS viewport starts at its right edge: its horizontal offset is 0 there and negative toward the end, and moves the content right, as a browser's RTL scroll position does. Indexed scrolling counts its items from that edge, and the host's scrolled layout bounds move content as it is drawn.
- Snap and indexed scrolling share one CSS scroll axis: an item's logical start is measured from the viewport's padding-box start edge, including the content box's solved position. This works whether padding lives on the viewport or the content box. Horizontal starts mirror in RTL. Steps follow solved item starts and use the CSS record's axis gap, resolved against the content box, for the last item's end.
- Each item's snap position is its logical start clamped into the scroll range. `scroll-index` keeps original item indices and interpolates between these positions in either direction, including reversed rows and columns. After the final item's index, fractional travel uses its size plus the resolved gap, as in Rive. If that interval reaches the scroll range's end, the end reads as its fraction; if the final item's start is beyond the range, its clamped snap position reads as its integer index. Indices at or beyond the item count target the scroll range's end. When several items share a clamped position, it reads the last index at that position.
- A CSS viewport with mandatory snap (`ScrollConstraint.snap`) settles after its first solved layout to the snap position nearest its initial offset. Leading padding therefore settles to the first item's start with index 0.
- Bound `scroll-x` and `scroll-y` properties use Rive's authored offsets, with negative values toward the end, in CSS and translated viewports alike. Offset bindings use push delivery through the ordinary property setters. Writes apply directly, including while the viewport is zero-sized.
- A `ScrollBarConstraint` thumb on a CSS viewport shows its visible share of the scrollable extent: the viewport's padding box over the padding box plus the scroll range. In RTL it rests at the right, its progress, track clicks, drags and overscroll shrink mirror horizontally, and its movement accounts for the thumb's solved position even when the track itself inherits RTL.
- Focus scroll-into-view converts physical movement into logical offsets in RTL. It reveals items inside the viewport's padding-box clip, using its border inset and its width and height, independent of the content box's solved position.
- The compiler writes `threshold` 10 on every CSS viewport's `ScrollConstraint`, the CSS default: a press moves 10 points before it becomes a scroll, about the slop a browser and iOS give a touch, so a tap that moves a little stays a tap. A scroll that starts cancels the press's clicks (4.5). A translated viewport keeps its authored threshold.
- A translated viewport keeps today's range, snap, index, thumb, focus and bound offset behavior.
- Viewports loaded with virtualization or infinite scrolling keep native viewport metrics, content-origin adjustment, extents and scroll coordinates for their lifetime, including inside a CSS viewport.

**Grid.** `display: grid` in CSS mode selects Taffy's grid, with the box's `GridTrack` and `GridItemPlacement` records, `gridAutoFlow`, the gaps and the alignment from the record. `layoutTypeValue` is ignored in CSS mode: a style left as a Rive stack does not make a CSS box a stack.

## 4. Box paint

A box paints itself at the size layout gives it: its border box, a rounded rectangle, unless a `BoxPaint` gives its paint another size (4.2). Its paint never counts towards any size. An artboard with a record paints its own fills and strokes the same way (CSS radii, `BoxBorder`, `BoxShadow` and `BoxGradient`), in its own drawing space, which starts at its origin. A box or artboard hidden by `display: none` paints none of its own paint, whatever its `BoxPaint`'s size and whether or not it clips, and is no visible geometry.

### 4.1 What the compiler writes on the box (Rive-native)

| CSS | Written as |
|---|---|
| `background-color` | a `Fill` child of the box with a `SolidColor` |
| `background-image: linear-gradient()` or `radial-gradient()` | a `Fill` with a `LinearGradient` or `RadialGradient` and its `GradientStop`s, plus a `BoxGradient` (4.2) so its geometry follows the box |
| uniform solid `border` (one width and one colour on all four sides) | a `Stroke` child, `position` 0 (inside), `thickness` the width. An inside stroke of the border box covers exactly the CSS border ring, inner radii included. The four border widths are also written in the record, because they take part in layout |
| `border-radius` in points, circular | `LayoutComponentStyle` `cornerRadiusTL` (640), `TR` (641), `BL` (642), `BR` (643), with `linkCornerRadius` (639) false when they differ |
| `box-shadow` | a `Fill` with a `SolidColor` (the shadow colour), written before the background `Fill` so it draws below it. Its `Feather` child has `strength` equal to the CSS blur radius (both are two standard deviations), `offsetX` and `offsetY` equal to the CSS offsets, `spaceValue` 1 (local, so the shadow moves with its box as in CSS), and `inner` true for `inset`. Inset shadows go after the background. Every shadow also gets a `BoxShadow` (4.2), spread 0 when it has none: it gives the shadow CSS's clipping |
| several backgrounds or shadows | several fills, in CSS's paint order (the first CSS layer is drawn last) |

### 4.2 Our additions

| Addition | Keys | Values | What the runtime does |
|---|---|---|---|
| Percent radii | `LayoutCssStyle` 61079 to 61082 | CSS percentage per corner | The corner's horizontal radius is its point radius plus that percentage of the box width, and its vertical radius is the same with the height, so a non-square box gets elliptical corners, as CSS. Overlapping radii scale down by CSS's rule |
| Per-side borders | `BoxBorder` 61083 `sides`, uint8 bit mask: 1 top, 2 right, 4 bottom, 8 left; default 15 | | Its `Stroke` draws its paint over the CSS border ring of the given sides: between the border box and the padding box, from the record's four border widths, with inner radii reduced by those widths, and adjacent sides meeting on the line from each outer corner through the matching inner corner, continued to the end of the corner's curve, as CSS, so a rounded ring is shared between its two sides. The stroke's own thickness and position are ignored. A different colour per side is one `Stroke` and `BoxBorder` per colour |
| Border styles | `BoxBorder` 61103 `style`, uint8: 0 solid, 1 dashed, 2 dotted; default 0 | | Applies to the record's sides, drawn as Chrome 155 draws them (Blink's border painter), inside the same ring and corner joins. On a box without radii each side is a line down its middle across its whole outer length: dashes two widths long with one-width gaps (three and two widths below 3 points), dots up to 3 points wide square and wider ones round, fitted to the side's length at every layout so a resize keeps them even: dashed gaps stretch to the dash count closest to the base gap, a dash at each end; thin dots keep whole dots at both ends; round dots are evenly spread end to end. A side too short for two dashes is solid, and so is a side whose marks would number more than 65,536 (a hairline border's), which look solid and stay within memory. On a rounded box a side whose inner corners are rounded follows the whole centre line instead (the border box inset by half the widths, from the top side's left end, clockwise), with dashes or dots fitted to its length as Skia measures it, cut to the ring. A uniform dashed or dotted border is one `BoxBorder` with all four sides, since a plain inside `Stroke` cannot fit dashes per side. Rive's own `DashPath` is not used: its dashes are fixed lengths or fractions of the whole contour, with no per-side fit or round dots |
| Box-relative gradients | `BoxGradient`, see the table below | | Recomputes its gradient's start and end (and, for an ellipse, the shader's scale) from the box's padding box at each layout, as CSS's background positioning area |
| Shadow spread and CSS shadow clipping | `BoxShadow` 61096 `spread`, double, points; default 0 | | The shadow fill's path is the border box grown by the spread (an inset shadow: the padding box shrunk by it), corner radii grown or shrunk by CSS's rule. An outer shadow draws only outside the border box, and an inset one only inside the padding box, as CSS |
| Paint size | `BoxPaint` 61097 `width` and 61101 `widthPercent`, 61098 `height` and 61102 `heightPercent`, doubles; points default 0, percentages default 100; keyable and bindable | each axis a length-percentage of the border box: points plus that percentage of the border box's size on the axis. Mixed values are allowed, since BoxPaint never goes through Taffy | While a box (or CSS artboard) has a `BoxPaint`, its own paint uses a rectangle of that size, centred on its border box and recomputed at each layout, in place of the border box: its fills and strokes, `BoxBorder` rings, `BoxShadow` paths (spread grows from that rectangle), `BoxGradient` positioning area (that rectangle's padding box), and its corner radii, with percentages resolved against it. At rest (0 points, 100%) the paint is the border box and follows it through any resize. An animation interpolates the points and the percentage separately, so (0, 100%) to (120, 0%) moves smoothly from the box's size to 120 points, as CSS interpolates a mixed length-percentage. A negative or non-finite result counts as 0. Layout, the content clip (still the padding box), an artboard's own clip, the box's hit testing and its semantics ignore it, so animating it reflows nothing; the box's painted bounds are that rectangle. In translated mode it has no effect |
| Padding-box clip | none: implied by CSS mode | | A CSS box whose computed `overflow-x` or `overflow-y` is not visible clips its children to its padding box, with inner radii when both axes clip. As CSS computes, `hidden`, `scroll` or `auto` on one axis turns `visible` on the other into `auto` (and `clip` into `hidden`), so both axes clip; `clip` with `visible` clips only its own axis, without radii. The clip is for the content: the box's own fills, borders and shadows draw before it. A hit on a child, and a child's semantic visibility, pass through the same clip; the box itself is hit anywhere in its border box. On an artboard, the record's overflow clips the artboard's content the same way, inside the artboard's own clip and after its own paints. A translated box keeps Rive's clip at its outer edge from `LayoutComponent.clip`, which also clips its own paints |

`BoxGradient` keys:

| Key | Name | Type | Values | Default |
|---|---|---|---|---|
| 61084 | `angle` | double | linear only: CSS degrees, 0 to top, clockwise | 180 (to bottom) |
| 61085 | `shape` | uint8 | radial only: 0 ellipse, 1 circle | 0 |
| 61086 | `size` | uint8 | radial only: 0 farthest-corner, 1 farthest-side, 2 closest-corner, 3 closest-side, 4 explicit | 0 |
| 61087, 61088 | `centerX`, `centerXPercent` | double, double | radial `at` position, length-percentage | 0 and 50 |
| 61089, 61090 | `centerY`, `centerYPercent` | double, double | as centerX | 0 and 50 |
| 61091, 61092 | `radiusX`, `radiusXPercent` | double, double | explicit size; a circle uses only radiusX | 0 and 0 |
| 61093, 61094 | `radiusY`, `radiusYPercent` | double, double | explicit size, ellipse only | 0 and 0 |
| 61095 | `toCorner` | uint8 | linear only: 0 none (use angle), 1 to top right, 2 to bottom right, 3 to bottom left, 4 to top left (CSS's corner angle, which depends on the box's shape) | 0 |

Stops keep Rive's `GradientStop` positions (0 to 1) and colours.

### 4.3 Checked against Chrome

The runtime's box paint is compared with Chrome 155's pixels for the same CSS
(`crates/nuxie-renderer/src/deferred/gm/css_box_paint.rs`, references in
`fixtures/css_layout/chrome/` from `tools/css-layout/chrome-references.sh`).
Every check draws its scene in both native Metal modes, raster-ordering (which
every iPhone uses) and clockwise-atomic, and each bound below holds in both:

- Shadows: a blur alone, offsets with spread, outer-only clipping, an inset shadow
  inside a border, and a rounded box. With `strength` equal to the CSS blur
  radius the mean difference is under half a level (of 255) in every case, and
  half or double that strength is clearly worse, so Rive's feather is two
  standard deviations, as CSS's blur radius is. Straight edges differ by at most
  8 levels; the rounded knock-out's corner arcs are anti-aliased less than
  Chrome's on under a hundred edge pixels.
- Gradients: an angle, a corner, a farthest-corner ellipse and a closest-side
  circle at a position, all within 2 levels.
- Dashed and dotted borders (`fixtures/css_layout/chrome/border_dashed_*` and
  `border_dotted_*`): thin and thick dashes, square and round dots, a rounded
  box and two colours, each within a mean of half a level; straight dashes
  and square dots are pixel-exact or within a few edge pixels, and round dots
  and curves differ only in their anti-aliasing.
- Per-side borders with two colours and a radius: a mean difference of 0.27
  levels; only pixels on the diagonal joins and curved edges differ.

Known differences in step 1:

- A gradient fills the border box with its end colours past the padding box,
  where CSS repeats the background image (its initial `background-repeat`).
  They differ only under a transparent border.
- Shadows and per-side borders drawn by a `ForegroundLayoutDrawable` keep
  Rive's own geometry.

Refused in step 1:

- stop positions in lengths;
- repeating gradients;
- `background-size`, `background-position`, `background-repeat` and `background-clip` values other than their initial ones;
- border styles other than solid, dashed and dotted (double, groove, ridge, inset and outset);
- `border-image`;
- radii with the slash syntax (separate horizontal and vertical values).

### 4.4 Painting order

Rive draws a box's children in reverse file order, so its first child draws on top. A CSS stacking context paints its descendants in CSS's order instead (CSS 2.1 Appendix E), bottom first:

1. the context's own paint (fills, strokes, shadows, borders), then its content clip;
2. descendant stacking contexts with a negative `z-index`, ascending, ties in tree order;
3. in-flow descendants and drawing content in tree order, each non-context box's paint before its in-flow content; flex and grid items paint as units while their positioned descendants join the real context;
4. positioned descendants with `z-index` auto and stacking contexts with `z-index` auto or 0, in tree order across the context; a positioned non-context box paints its own paint and in-flow content as a unit;
5. descendant stacking contexts with a positive `z-index`, ascending, ties in tree order.

A flex or grid item's `z-index` applies when it is not positioned too, as CSS says. A participant (a shape, image or text hosting a `LayoutParticipant`) orders by its host's record, as a box does. A translated child of a CSS box is positioned, with `z-index` auto, when layout places it absolutely. The order is computed by the runtime when the draw order is sorted, and again when `position`, `z-index`, `display` or `overflow` change at run time (a binding, an animation, a script), so the compiler writes no order of its own. Hit testing follows the same order: the topmost painted box is hit first, and the event then bubbles to the CSS boxes it is in (4.5).

A context's own paint stays below its descendants, and a non-context box's paint stays below its in-flow content, however its drawables arrive: children materialized after other objects (a text input's hint and value) or re-created by a retained remount reach the draw list after the box's own drawables.

Positioned descendants and descendant stacking contexts escape non-clipping CSS boxes to join their nearest stacking context. The CSS artboard is a context. A CSS box creates a context when it is positioned with an integer `z-index`, is a flex or grid item with an integer `z-index`, has its own opacity below 1, or has translation, rotation or scale authored by a timeline or a binding that writes to the target, including a held identity transform. An unkeyed, unbound transform also creates a context while its current value is not identity. Authored transform targets are recorded once at load; opacity crossing 1 and unkeyed transforms reaching or leaving identity request a re-sort, without sorting on every animation frame. Descendant collection stops at nested contexts and atomic hosts. Translated (Rive) boxes, nested-artboard hosts, component lists and participant hosts stay atomic, and an artboard with draw rules keeps Rive's order throughout. A CSS box whose computed overflow clips its content also keeps its subtree together, including positioned descendants, until per-drawable clip state replaces the content clip's save and restore bracket. A CSS box covered by a layer mask, or used as a layer mask source, also keeps its subtree together until masking supports noncontiguous members.

### 4.5 Host geometry

A CSS box or CSS artboard whose own paints include a visible one is host geometry, as a painted shape is:

- visible geometry: its painted area (the border box, or its `BoxPaint`'s rectangle) with its paints' outsets (strokes, feathers, shadow spread), clipped as it draws by its clipping shapes and its ancestors' clips within its artboard: their paths, rounded corners and a CSS artboard's padding-box clip included, but not its own content clip. A box clipped away is left out. A `BoxBorder` counts as visible whatever its `Stroke`'s thickness, as it draws, and a paint that does not draw (hidden, say) adds nothing. A CSS artboard's visible geometry is its painted area inside its own clip;
- visible-geometry selection (the host's geometry hit query, which an editor selects with): a point selects the box only where one of its paints draws, inside its ancestors' clips, in the painting order of 4.4. A fill draws in its rounded border box (or its `BoxPaint` rectangle), a `BoxBorder` or a `Stroke` on its painted ring (a `BoxBorder` on its own sides, between the rounded border box and the padding box with its inner radii, and a dashed or dotted one only on its marks, not in the gaps between them; a `Stroke` by its thickness and position), and a shadow is not selectable. So a box painted only by its top border is selected on that side and not in its unpainted middle, which matches the bounds it reports;
- interaction hit testing (taps, listeners, scrolling, the state machine): the box itself is hit in its rounded border box (CSS radii, percentages included, elliptical where they are), whatever it paints and whatever its `BoxPaint`, as a browser hits it, when its render opacity is above 0. A child is hit only inside each clipping ancestor's padding box with its inner radii, read as the radii are at that frame, so a tap in a viewport's rounded-off corner reaches what is behind it, and an animated radius clips as it changes;
- pointer events bubble, as a browser's do: an event reaches the topmost hit and then the CSS boxes that hit is in, whose pointer listeners hear it deepest first, since a box's own paint is the bottom of its stack (4.4), and whatever the boxes' own geometry, so a field placed outside its box still clicks the box, and a box that listens for the pointer entering stays entered while the pointer is over a child outside it. An opaque hit, a text input say, blocks everything else behind it: a sibling box under it, and translated (Rive) layouts, including those between it and the CSS boxes. The button query hosts route a press by (whether a listener answers a button at a point) follows the same rule. Wheel scrolling still stops at an opaque hit, as Rive's does; scrolling the CSS box around it, as a browser would, is a follow-up;
- a pointer's drag goes to the innermost drag handler under it, as on a phone. The drag handlers are a `ScrollConstraint`'s scroll, a scroll bar's thumb and track, an authored drag listener, and a focused text input, which selects. An unfocused field, a button or a card with only click listeners is no drag handler: a drag that starts on it scrolls the CSS scroll box around it. A scroll bar's thumb and track, and a focused field, take the drag as the press goes down; a scroll takes it when it starts to scroll, past its `threshold` (3), so a drag across an inner scroll box goes to the one around it, and one along it stays in the inner one. Once a drag handler takes the drag, the others let the press go, and once a scroll starts, the press's clicks and a field's tap are cancelled. A handler inside a nested artboard that takes the press's gesture, a field say, holds the drag for the host's CSS drag handlers too, and when a host handler takes the drag, the nested artboard's drag handlers let it go. A field's tap belongs to its own pointer, so another finger's scroll does not cancel it. An unfocused field focuses on a tap's release, with its caret there, so a scroll that starts on it focuses nothing;
- translated content keeps Rive's hit testing and gestures, as upstream has them: an opaque child blocks its parents' listeners, a field takes a drag whether it is focused or not and focuses on the press, and every scroll under a press tracks it, clicking what is under the release;
- retained geometry: every CSS box with paints.

Translated boxes keep today's catalogue, which counts painted shapes, text and text fields.

## 5. Leaf measurement

A box with layout children lays them out. Its non-layout children do not take part in its layout; they draw where they are authored. A box without layout children measures its content. In CSS mode the compiler makes each `<img>`, `<svg>` or text run that must sit in flow among siblings its own box or participant.

**Where a CSS leaf puts its content.** A CSS box without layout children lays its measured content out as one anonymous item of its own display. Its Taffy node has one synthetic child with CSS's initial item properties: auto size, flex `0 1 auto`, and auto self alignment. That child measures the content, so the box keeps its intrinsic sizes and automatic minimum through it. An empty box, or content without a measurable natural size, gets no synthetic child. Text is measured content even though it reflows rather than having a fixed natural size.

Flex, grid and block alignment apply to that item, including block `align-content`. The runtime places content at the item's solved position, which already includes the box's border and padding, and gives text the item's size. A row flex leaf therefore keeps a short anonymous item at its intrinsic width: `text-align: center` has no extra width to centre within unless container alignment moves the item. Block, column flex and grid normally stretch its inline size. The compiler authors content at (0, 0); the runtime adds the item's position before each content object's own transform. An artboard also adds its drawing origin. Padding, border and alignment written at run time move the content at the next layout. During layout interpolation, the anonymous item interpolates its position and size with the box at the same factor. Retargeting carries the item endpoints through the same smoothing as the box. A leaf without an anonymous item falls back to the content-box origin. A translated box, and a CSS box with layout children, leave their content where it was authored, as Rive does.

| Content | Natural size | Does layout resize it? |
|---|---|---|
| Text | Translated text uses Rive's `Text::measure` and `Text::measure_layout`, unchanged. Text a CSS box lays out uses CSS line boxes: hhea ascent, descent and line gap are each scaled to the font size and rounded to whole points, regardless of USE_TYPO_METRICS. `normal` is their sum; an authored line height is used as given. Each run's box is that line height tall, with its baseline at ascent plus `floor((line-height - ascent - descent) / 2)` from the top, including negative leading. Each line is the union of its runs' boxes and the paragraph strut (the element's own font and line height), aligned on their shared baseline; the first baseline includes this leading too. Measurement, drawing, selection, carets and bounds use these same boxes. max-content is the unwrapped width; min-content is the longest word (the fit-content work already on main). In CSS mode these intrinsic sizes stay the same for Hidden, Clipped, Ellipsis, Fit and FitFontSize text; nowrap min-content is the whole unwrapped line. Only the box's own CSS overflow controls its zero automatic minimum and grid minimum contribution through Taffy's scroll-container rules. Translated text keeps its zero minimum for those text overflow modes | Yes, in both modes: layout gives text its inline size so it can reflow. In CSS mode the text is given the anonymous item's size and is placed at its solved position (above); translated boxes give their border box, as Rive does |
| Image | its image's natural size and ratio, transferred through a known cross size and clamped through the cross minimum and maximum, as a CSS replaced element | CSS mode: no, as content it keeps its natural size, past its box if the box is smaller, and its overflow counts towards a scroll container's range; as a participant, its CSS sizes it and Rive's fit draws inside. Translated mode: as today |
| Shape (CSS mode) | the union of its paths' geometry at their authored sizes, through each path's transforms; an empty shape adds nothing. Like an `<svg width="24">`. Its minimum is the same size | No |
| Shape resized by layout (translated mode: a box's direct shape child, which Rive resizes to the box) | the box's paint: it gives a size only to a box that hugs it, through Rive's own measure (the path's authored size), and adds nothing to any other box's automatic minimum | Yes, as Rive does |
| Drawing group | the union of its content, each child placed by its local transform (a path's transform composed through any nodes inside its shape; images and joysticks about their origin, joysticks at their position). A child without a natural size of its own (a text inside the group) adds nothing | No |
| N-slice, joystick | its authored width and height | As today |
| Mounted artboard (a component-list item) or nested artboard | A list row with an artboard record, or a `NestedArtboardLayout` with both an artboard record and a host child record, is a CSS layout box: its content supplies its natural size. A nested copy's record replaces the artboard's record for its whole box in CSS mode, supplying container and item properties, including `display: none` (2.1). A list row uses its own artboard's record. Other mounted or nested artboards use their artboard's own size, like an image: a box hugs to it, and on an axis where it has a definite size that size is its automatic minimum, clamped by a length maximum or by a percentage of a parent's definite length, with no minimum on a scroll-container axis | CSS hosted roots reflow to their CSS item size. A `NestedArtboardLeaf` keeps its fit and alignment; in a CSS sizing box it fits to the physical content-box dimensions, independent of the anonymous item's size. In a leaf box the content placement is already applied, so the fit starts at local (0, 0). With layout children, content is not moved and the fit starts at the content box's border-and-padding origin. Changes to these fitting bounds recompute the leaf's transform, even when the outer box and content placement stay fixed. A translated sizing box fits the leaf to its whole local bounds. Other cases keep existing sizing |
| Text input, scripted layout, artboard without an instance | zero | |

A CSS box's automatic minimum is the largest of its contents' own: content with a zero minimum (the rows above that say so) adds nothing, and the rest still count. A translated box keeps a zero automatic minimum when any of its measured content has none, as today.

**The update-pass check stays.** It keeps these natural sizes current. The component graph's update loop queues the measured host of each member that recomputes:

- a local transform (Transform dirt);
- a path or its vertices (Path and Vertices dirt);
- an n-slice (NSlicer dirt).

Joysticks (size, origin, position) and images (origin, new decoded image) hand themselves to the same queue.

At the end of each update step, each queued host's natural size is compared with the size its box last measured (within 16 ULPs). A change marks the box, which lays out one frame later.

Only a box that measures content is checked:

- in CSS mode, a leaf box;
- in translated mode, a box that hugs, or any leaf box but an artboard.

A shape that layout resizes as paint is never measured, so it needs no check. Layout's own write-back never registers, because the natural size is the authored one, kept on the generated bases of parametric paths, n-slices and joysticks.

### 5.1 CSS text processing

`TextStyleCssText` is a child of each participating `TextStylePaint`. The compiler writes the inherited values on each style. Both properties are bindable uint8 values, default to 0 and do not animate:

| CSS | Key | Values |
|---|---|---|
| `text-transform` | 61104 `textTransform` | 0 none, 1 uppercase, 2 lowercase, 3 capitalize |
| `white-space` | 61105 `whiteSpace` | 0 normal, 1 pre, 2 pre-wrap (unsupported, behaves as normal), 3 pre-line, 4 nowrap |

Only text owned by a CSS box uses these records. Runs without a record keep their source unchanged. `TextInput` keeps its editable value unchanged. The runtime derives one display string after reading authored and bound values, before shaping or modifiers. Full Unicode default case mapping can expand characters, including `ß` to `SS`; language tailoring is unavailable until the element carries a language. `capitalize` puts a word's first lowercase letter into full default Unicode titlecase (`ǳ` to `ǲ`, `ß` to `Ss`), preserving uppercase and titlecase initials. Capitalization and whitespace state cross run boundaries. Normal and nowrap collapse spaces, tabs and segment breaks; pre-line collapses spaces and tabs but keeps segment breaks; pre preserves spaces and segment breaks. The pre-wrap value is not supported yet and behaves as normal, collapsing spaces, tabs and segment breaks while allowing lines to wrap. CRLF and CR normalize to LF. Preserved tabs are shaped as spaces and given one fixed advance of eight space widths in the tab's font and size. They do not align to tab stops yet. They draw no glyph and keep their source boundary. Wrapping remains on Rive's `Text.wrap` (683), set by the compiler. Paragraph-edge collapsible spaces are removed while building the display, and soft-line edge spaces are excluded from each broken line before measuring, aligning and drawing it. Preserved spaces are never trimmed by this pass.

One source-to-display code-point boundary table covers expansions and removed whitespace. Run offsets, lengths and modifier ranges use display indices. Regular-text selection ranges, hit offsets, copied source text, and host caret, hit and selection rectangle queries use source indices (the host methods retain their UTF-8 byte offset API). Hits within an expanded character compare the pointer with source-boundary caret geometry, so rounding to an intermediate display offset does not move the hit threshold. Boundaries removed by collapsing resolve to the end of the collapsed source span. Record writes, style changes and bound value changes dirty the shape and layout. A text without a CSS owner or without any participating record follows the translated path unchanged.

## 6. What the compiler stops writing

As far as the runtime is concerned, a CSS file carries no compile-time layout result. The runtime ignores, or no longer needs:

- `LayoutComponent` width, height and fractional sizes on a CSS box, and any size the compiler worked out for a box;
- separate paint shapes for backgrounds, borders and shadows, their parametric path sizes, and the extra boxes that made them follow (border centreline, padding clip, one per shadow);
- the Responsive Border and Responsive Gradient scripts, and gradient ends in points for box backgrounds (`BoxGradient` replaces them);
- dash runs worked out from a box's perimeter (dashed borders are refused in step 1);
- wrapper insets worked out at compile time;
- scroll extents and content sizes for a `ScrollConstraint`, which come from Taffy's scrollable area;
- text box sizes estimated from a box (layout sizes the text).

What it does write: the record as authored (section 1), the Rive-native paint in 4.1 with our additions in 4.2, and Rive's objects for everything that is not layout or box paint (text, images, shapes as content, animations, state machines, data binding).
