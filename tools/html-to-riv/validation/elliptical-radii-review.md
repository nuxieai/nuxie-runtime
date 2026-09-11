# Elliptical and percentage corner radii (P04)

The standalone compiler accepts one-to-four nonnegative px/em/rem/percentage
values in `border-radius`, with an optional slash separating horizontal and
vertical lists. Physical corner longhands accept one or two values. Existing
CSS-wide keywords and custom-property substitution apply. Percentages remain
responsive: horizontal radii use live border-box width and vertical radii use
live height. Overlapping corners share a proportional reduction factor; a zero
axis produces a square corner.

The checked v24 `layout-css-corner-radii-v1` occurrence policy transports four
axis pairs. Native installation validates all targets before changing any;
clones retain independent policy, and clearing restores imported behavior.
Circular pixel-only cases retain the legacy representation where possible.

## Validation

Pinned Chrome153.0.8010.12 is the reference. Initial16, fractional/clip-edge16
and nested/text/image/unequal-border52 scenes cover84 scenes and672 compiled-once
original/clone lifecycle frames. All252 distinct views have audited visual
coverage. The current native full regression passes7696 checks; baseline7434
and ellipse252 pairs match reviewed inputs and complete browser/native PNGs.
The completed combined audit and native qualification are recorded in
`output/playwright/html-to-riv/elliptical-radii-p04-receipt.json`.

Default compiler suite410 tests passes. The expanded accepted corpus passes
native/WASM parity. Three host checks initially failed from an omitted frozen
probe path, then passed with the explicit path; failure logs are preserved.

## Limits

Qualification is for the frozen native glyph/LTR profile, not every renderer.
The composition vector replay retains36 text pixel failures across6 scenes;
all416 geometry frames pass. These pixel failures remain unwaived. Sparse curve
antialias and image sampling differences remain under unchanged gates. Large
finite radius overflow handling has analytic/runtime tests, not a broad Chrome
pixel qualification. Direction controls, group opacity, transforms and future
paint features require their own composition evidence. Grid/editor integration
remain excluded. P05 source work does not alter the frozen P04 qualification.
