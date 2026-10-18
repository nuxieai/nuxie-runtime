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

Allocation, after Video:

- **Type keys**: 60002 to 60005.
- **Property keys**: 60022 to 60118.

The definition files are added in `defs/nuxie/` beside `video.json` when this contract is accepted, and the README's allocation list gains these lines.

| Type | Key | Extends | Parent | Rive-native or ours |
|---|---|---|---|---|
| `LayoutCssStyle` | 60002 | `component.json` | a box, or a participant's host; at most one per parent | ours |
| `BoxBorder` | 60003 | `component.json` | a `Stroke` painted by a box | ours |
| `BoxGradient` | 60004 | `component.json` | a `LinearGradient` or `RadialGradient` painted by a box | ours |
| `BoxShadow` | 60005 | `component.json` | a `Feather` on a `Fill` painted by a box | ours |

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
| `display` | 60022 `display` | uint8 | 0 block, 1 flex, 2 grid, 3 none, 4 flow-root | `inline` in CSS, which we do not have; 0 block is the `div` default |
| `box-sizing` | 60023 `boxSizing` | uint8 | 0 content-box, 1 border-box | content-box |
| `position` | 60024 `position` | uint8 | 0 static, 1 relative, 2 absolute | static |
| `overflow-x` | 60025 `overflowX` | uint8 | 0 visible, 1 clip, 2 hidden, 3 scroll, 4 auto | visible |
| `overflow-y` | 60026 `overflowY` | uint8 | as overflow-x | visible |
| `direction` | 60027 `direction` | uint8 | 0 inherit, 1 ltr, 2 rtl | inherited (ltr at the root) |
| `flex-direction` | 60028 `flexDirection` | uint8 | 0 row, 1 row-reverse, 2 column, 3 column-reverse | row |
| `flex-wrap` | 60029 `flexWrap` | uint8 | 0 nowrap, 1 wrap, 2 wrap-reverse | nowrap |
| `flex-grow` | 60030 `flexGrow` | double | number | 0 |
| `flex-shrink` | 60031 `flexShrink` | double | number | 1 |
| `flex-basis` | 60032 `flexBasisKind`, 60033 `flexBasis`, 60034 `flexBasisPercent` | uint8, double, double | size kinds, plus 6 content | auto |
| `align-items` | 60035 `alignItems` | uint8 | item alignment | normal |
| `align-self` | 60036 `alignSelf` | uint8 | item alignment, 0 auto | auto |
| `justify-items` | 60037 `justifyItems` | uint8 | item alignment | legacy (behaves as normal) |
| `justify-self` | 60038 `justifySelf` | uint8 | item alignment, 0 auto | auto |
| `align-content` | 60039 `alignContent` | uint8 | content alignment | normal |
| `justify-content` | 60040 `justifyContent` | uint8 | content alignment | normal |
| `grid-auto-flow` | 60041 `gridAutoFlow` | uint8 | 0 row, 1 column, 2 row dense, 3 column dense | row |
| `aspect-ratio` | 60042 `aspectRatio` | double | width over height; 0 auto | auto |
| `width` | 60043 `widthKind`, 60044 `width`, 60045 `widthPercent` | uint8, double, double | size kinds | auto |
| `height` | 60046 `heightKind`, 60047 `height`, 60048 `heightPercent` | uint8, double, double | size kinds | auto |
| `min-width` | 60049 `minWidthKind`, 60050 `minWidth`, 60051 `minWidthPercent` | uint8, double, double | 0 auto, 1 length-percentage, 2 percentage (as size kind 7) | auto (the browser's automatic minimum) |
| `min-height` | 60052 to 60054, as min-width | | | auto |
| `max-width` | 60055 `maxWidthKind`, 60056 `maxWidth`, 60057 `maxWidthPercent` | uint8, double, double | 0 none, 1 length-percentage, 2 percentage (as size kind 7) | none |
| `max-height` | 60058 to 60060, as max-width | | | none |
| `margin-top`, `-right`, `-bottom`, `-left` | 60061 to 60063, 60064 to 60066, 60067 to 60069, 60070 to 60072: `margin<Side>Kind`, `margin<Side>`, `margin<Side>Percent` | uint8, double, double | 0 length-percentage, 1 auto | 0 |
| `top`, `right`, `bottom`, `left` | 60073 to 60075, 60076 to 60078, 60079 to 60081, 60082 to 60084: `inset<Side>Kind`, `inset<Side>`, `inset<Side>Percent` | uint8, double, double | 0 auto, 1 length-percentage | auto |
| `padding-top`, `-right`, `-bottom`, `-left` | 60085 and 60086, 60087 and 60088, 60089 and 60090, 60091 and 60092: `padding<Side>`, `padding<Side>Percent` | double, double | length-percentage | 0 |
| `border-top-width`, `-right-`, `-bottom-`, `-left-` | 60093 `borderTop`, 60094 `borderRight`, 60095 `borderBottom`, 60096 `borderLeft` | double | points | 0 (CSS's `medium` is used only when a border style is set; the compiler writes the used width) |
| `row-gap` | 60097 `rowGap`, 60098 `rowGapPercent` | double, double | length-percentage | normal (0 in flex and grid) |
| `column-gap` | 60099 `columnGap`, 60100 `columnGapPercent` | double, double | length-percentage | normal (0) |
| `border-*-radius`, percent part | 60101 `radiusTopLeftPercent`, 60102 `radiusTopRightPercent`, 60103 `radiusBottomRightPercent`, 60104 `radiusBottomLeftPercent` | double | CSS percentage | 0 |

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

On an `Artboard` the artboard's own width and height define the root box. A record there contributes only container properties: display, padding, border, gap, direction, alignment, wrap and overflow.

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
| `content_size` | a node's scrollable overflow rectangle, `scroll_width` and `scroll_height`, for scroll ranges |

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
- In CSS mode the compiler gives that content box CSS that does not shrink to the viewport: `display: block` for a vertical list, or a row with `width: max-content` (or `flex: none`) and items at `flex-shrink: 0` for a horizontal one.
- The scroll range of a CSS viewport is Taffy's `scroll_width` and `scroll_height` of that viewport, replacing the content size minus viewport size minus end padding used today.
- A translated viewport keeps today's range.
- Virtualised and looping lists keep their own extents (Rive-native).

**Grid.** `display: grid` in CSS mode selects Taffy's grid, with the box's `GridTrack` and `GridItemPlacement` records, `gridAutoFlow`, the gaps and the alignment from the record. `layoutTypeValue` is ignored in CSS mode: a style left as a Rive stack does not make a CSS box a stack.

## 4. Box paint

A box paints itself at the size layout gives it: its border box, a rounded rectangle. Its paint never counts towards any size. An artboard with a record paints its own fills and strokes the same way (CSS radii, `BoxBorder`, `BoxShadow` and `BoxGradient`), in its own drawing space, which starts at its origin.

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
| Percent radii | `LayoutCssStyle` 60101 to 60104 | CSS percentage per corner | The corner's horizontal radius is its point radius plus that percentage of the box width, and its vertical radius is the same with the height, so a non-square box gets elliptical corners, as CSS. Overlapping radii scale down by CSS's rule |
| Per-side borders | `BoxBorder` 60105 `sides`, uint8 bit mask: 1 top, 2 right, 4 bottom, 8 left; default 15 | | Its `Stroke` draws its paint over the CSS border ring of the given sides: between the border box and the padding box, from the record's four border widths, with inner radii reduced by those widths, and adjacent sides meeting on the line from each outer corner through the matching inner corner, continued to the end of the corner's curve, as CSS, so a rounded ring is shared between its two sides. The stroke's own thickness and position are ignored. A different colour per side is one `Stroke` and `BoxBorder` per colour |
| Box-relative gradients | `BoxGradient`, see the table below | | Recomputes its gradient's start and end (and, for an ellipse, the shader's scale) from the box's padding box at each layout, as CSS's background positioning area |
| Shadow spread and CSS shadow clipping | `BoxShadow` 60118 `spread`, double, points; default 0 | | The shadow fill's path is the border box grown by the spread (an inset shadow: the padding box shrunk by it), corner radii grown or shrunk by CSS's rule. An outer shadow draws only outside the border box, and an inset one only inside the padding box, as CSS |
| Padding-box clip | none: implied by CSS mode | | A CSS box whose computed `overflow-x` or `overflow-y` is not visible clips its children to its padding box, with inner radii when both axes clip. As CSS computes, `hidden`, `scroll` or `auto` on one axis turns `visible` on the other into `auto` (and `clip` into `hidden`), so both axes clip; `clip` with `visible` clips only its own axis, without radii. The clip is for the content: the box's own fills, borders and shadows draw before it. A hit on a child, and a child's semantic visibility, pass through the same clip; the box itself is hit anywhere in its border box. On an artboard, the record's overflow clips the artboard's content the same way, inside the artboard's own clip and after its own paints. A translated box keeps Rive's clip at its outer edge from `LayoutComponent.clip`, which also clips its own paints |

`BoxGradient` keys:

| Key | Name | Type | Values | Default |
|---|---|---|---|---|
| 60106 | `angle` | double | linear only: CSS degrees, 0 to top, clockwise | 180 (to bottom) |
| 60107 | `shape` | uint8 | radial only: 0 ellipse, 1 circle | 0 |
| 60108 | `size` | uint8 | radial only: 0 farthest-corner, 1 farthest-side, 2 closest-corner, 3 closest-side, 4 explicit | 0 |
| 60109, 60110 | `centerX`, `centerXPercent` | double, double | radial `at` position, length-percentage | 0 and 50 |
| 60111, 60112 | `centerY`, `centerYPercent` | double, double | as centerX | 0 and 50 |
| 60113, 60114 | `radiusX`, `radiusXPercent` | double, double | explicit size; a circle uses only radiusX | 0 and 0 |
| 60115, 60116 | `radiusY`, `radiusYPercent` | double, double | explicit size, ellipse only | 0 and 0 |
| 60117 | `toCorner` | uint8 | linear only: 0 none (use angle), 1 to top right, 2 to bottom right, 3 to bottom left, 4 to top left (CSS's corner angle, which depends on the box's shape) | 0 |

Stops keep Rive's `GradientStop` positions (0 to 1) and colours.

### 4.3 Checked against Chrome

The runtime's box paint is compared with Chrome 155's pixels for the same CSS
(`crates/nuxie-renderer/src/deferred/gm/css_box_paint.rs`, references in
`fixtures/css_layout/chrome/` from `tools/css-layout/chrome-references.sh`):

- Shadows: a blur alone, offsets with spread, outer-only clipping, an inset shadow
  inside a border, and a rounded box. With `strength` equal to the CSS blur
  radius the mean difference is under half a level (of 255) in every case, and
  half or double that strength is clearly worse, so Rive's feather is two
  standard deviations, as CSS's blur radius is. Straight edges differ by at most
  8 levels; the rounded knock-out's corner arcs are anti-aliased less than
  Chrome's on under a hundred edge pixels.
- Gradients: an angle, a corner, a farthest-corner ellipse and a closest-side
  circle at a position, all within 2 levels.
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
- border styles other than solid;
- `border-image`;
- radii with the slash syntax (separate horizontal and vertical values).

## 5. Leaf measurement

A box with layout children lays them out. Its non-layout children do not take part in its layout; they draw where they are authored. A box without layout children measures its content. In CSS mode the compiler makes each `<img>`, `<svg>` or text run that must sit in flow among siblings its own box or participant.

**Where a CSS leaf puts its content.** A CSS box without layout children, an artboard included (from its drawing origin), places its content (text, images, shapes, drawing groups, nested artboards) at its content box's origin: its left border plus left padding, and its top border plus top padding, as a browser does. The compiler authors that content at (0, 0), and the runtime adds the origin before the content's own transform, at each layout. Padding and border written at run time (a binding, an animation, a breakpoint, a script) move the content at the next layout. The content box is physical, so the origin is the same right to left. A translated box, and a CSS box with layout children, leave their content where it was authored, as Rive does.

| Content | Natural size | Does layout resize it? |
|---|---|---|
| Text | Rive's `Text::measure` and `Text::measure_layout`, unchanged. max-content is the unwrapped width; min-content is the longest word (the fit-content work already on main). Hidden, Clipped, Ellipsis, Fit and FitFontSize text has a zero automatic minimum, as a CSS scroll container would | Yes, in both modes: its width follows its box's content width, as inline text flows. In CSS mode the text is given the box's content box (the border box less padding and border), where it was measured, and is placed at its origin (above); translated boxes give their border box, as Rive does |
| Image | its image's natural size and ratio, transferred through a known cross size and clamped through the cross minimum and maximum, as a CSS replaced element | CSS mode: no, as content it keeps its natural size, past its box if the box is smaller, and its overflow counts towards a scroll container's range; as a participant, its CSS sizes it and Rive's fit draws inside. Translated mode: as today |
| Shape (CSS mode) | the union of its paths' geometry at their authored sizes, through each path's transforms; an empty shape adds nothing. Like an `<svg width="24">`. Its minimum is the same size | No |
| Shape resized by layout (translated mode: a box's direct shape child, which Rive resizes to the box) | the box's paint: it gives a size only to a box that hugs it, through Rive's own measure (the path's authored size), and adds nothing to any other box's automatic minimum | Yes, as Rive does |
| Drawing group | the union of its content, each child placed by its local transform (a path's transform composed through any nodes inside its shape; images and joysticks about their origin, joysticks at their position) | No |
| N-slice, joystick | its authored width and height | As today |
| Mounted artboard (a component-list item) or nested artboard | its artboard's own size, like an image: a box hugs to it, and on an axis where it has a definite size that size is its automatic minimum. That minimum is clamped by a length maximum or by a percentage of a parent's definite length. It gives no minimum on a scroll-container axis | As today |
| Text input, scripted layout, artboard without an instance | zero | |

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
