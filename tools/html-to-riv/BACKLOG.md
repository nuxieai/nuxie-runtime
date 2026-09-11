# Standalone compiler backlog — immutable runtime target

The target is the unchanged repository tree at `6c7ac16617835b5f581784ff08a9e779bb52faf3`, restored on main by PR #629 (`9738049372ffd45639de39c2217c1f548963340a`). This supersedes the compiler-specific runtime profile from PR #628. Runtime, renderer, format/schema, dependency forks and host behavior are fixed inputs.

Keep all 99 original items and priority order. Existing parsing and browser references can be reused after audit; runtime-dependent qualification cannot. A feature is supported only after ordinary emitted Rive bytes pass the unchanged importer/renderer, including same-scene resizing and full visual review. A file-level shim uses only existing Rive objects; no CSS policy installation, host callbacks, special renderer, new wire fields or runtime scripts. Unsupported semantics produce a diagnostic, with runtime proposals documented separately. Do not flatten responsive scenes to browser-baked rectangles or images as an implicit fallback.

Statuses: `pending` = implementation/proof remains; `qualified` = immutable-target evidence complete; `unsupported` = evidenced current-runtime limitation. Wrapper candidates remain pending until tested. Do not equate pending proof with impossibility. Historical results are preserved in validation/history/BACKLOG-mutated-runtime.md.

Excluded: CSS Grid, editor integration, scripts, interactions, bindings and animation.

| ID | Feature | Status | Evidence / next action |
| --- | --- | --- | --- |
| A01 | Standard CSS named colors | qualified (solid fills) | All 148 named colors now pass public HTML/CSS native pixels and original/clone resizing; CLI/WASM parity and visual review recorded. Qualified for admitted solid fills; all 148 colors rerendered with canvas independence in validation/public-background-receipt.json. Text and other paints require their own feature admission. |
| A02 | HSL/HSLA | qualified (solid fills) | Public HSL/HSLA admission, 24 angle/alpha/clamping boundary swatches, CLI/WASM parity and immutable native visual comparisons pass. Qualified for admitted 8-bit solid fills; full rerun and clear controls in validation/public-background-receipt.json. Other paint/text contexts remain separately scoped. |
| A03 | `inherit` | partial | Corrected inherited currentColor with preserved 8-frame failure and 24 passing/reviewed native regression frames. Width/height descriptor inheritance now passes the public CSS-wide corpus, including percent and auto-height chains. Broader properties remain pending. See validation/public-inheritance-receipt.json and validation/public-css-wide-receipt.json. |
| A04 | `initial` | partial | Width/height reset to auto; foreground/background reset semantics pass public native comparisons and visual review. Inline display remains rejected; row initial direction is now qualified in the bounded L01 profile. See validation/public-css-wide-receipt.json. |
| A05 | `unset` | partial | Inherited color vs initial dimensions/background distinguished, with cascade and native resize proof. Broader property contexts remain pending. See validation/public-css-wide-receipt.json. |
| A06 | Solid-color `background` shorthand | qualified (solid fills) | Single color/currentColor/none/CSS-wide forms, ordering/important/inline precedence and inherited sentinel validated with 14 scenes. Other constituents rejected; see validation/public-background-receipt.json. |
| A07 | `font` shorthand | investigating | Ordinary embedded-font import/clone works. Initial 24 pixel comparisons fail; expanded paint/font experiments pass 48/88, with small-size failures preserved. Duplicate Fill is a bounded candidate; line metrics and general typography remain unresolved. Public font/text admission and shorthand remain pending. See validation/ordinary-text-review.md. |
| A08 | Unitless line-height | pending | Immutable-target audit and revalidation required; historical status: qualified. |
| A09 | `em` lengths | partial | Width/height and compiler-side font-size context pass64 public geometry/pixel frames, original/clone resize, CLI/WASM parity and visual review. Other property contexts remain pending. See validation/public-font-relative-receipt.json. |
| A10 | `rem` lengths | partial | Width/height and font-size context validated against explicit16px root reset. Broader property contexts remain pending; see validation/public-font-relative-receipt.json. |
| A11 | Percentage min/max dimensions | partial | Ordinary numeric/auto bounds implemented;112/112 geometry,110/112 pixel frames pass. Fractional-edge failure retained without tolerance changes. Automatic minima preserve native Auto; indefinite height-parent context rejected pending proof. See validation/minmax-review.md and public-minmax-receipt.json. |
| A12 | Letter spacing | pending | Immutable-target audit and revalidation required; historical status: partial. |
| A13 | Word spacing | pending | Immutable-target audit and revalidation required; historical status: partial. |
| A14 | Explicit `<br>` | pending | Immutable-target audit and revalidation required; historical status: qualified. |
| A15 | `white-space: nowrap` | pending | Immutable-target audit and revalidation required; historical status: qualified. |
| A16 | `white-space: pre` | pending | Immutable-target audit and revalidation required; historical status: qualified. |
| A17 | `white-space: pre-wrap` | pending | Immutable-target audit and revalidation required; historical status: partial. |
| A18 | `white-space: pre-line` | pending | Immutable-target audit and revalidation required; historical status: partial. |
| A19 | Text transforms | pending | Immutable-target audit and revalidation required; historical status: partial. |
| A20 | Underline | pending | Immutable-target audit and revalidation required; historical status: partial. |
| A21 | Strikethrough | pending | Immutable-target audit and revalidation required; historical status: partial. |
| A22 | Text clipping | pending | Immutable-target audit and revalidation required; historical status: partial. |
| A23 | Text ellipsis | pending | Immutable-target audit and revalidation required; historical status: partial. |
| S01 | Attribute selectors | partial | Operators and i/default case semantics pass public color/geometry/native corpus. Authored attrs remain id/class/style only; broader admission pending. See validation/public-selector-receipt.json. |
| S02 | Adjacent sibling selector | qualified (static box DOM) | Public selector corpus passes192 geometry/pixel frames plus384 clear controls, expected-color assertions, CLI/WASM parity and visual review. Existing grammar/admission restrictions remain; see validation/selectors-review.md. |
| S03 | General sibling selector | qualified (static box DOM) | Public selector corpus passes192 geometry/pixel frames plus384 clear controls, expected-color assertions, CLI/WASM parity and visual review. Existing grammar/admission restrictions remain; see validation/selectors-review.md. |
| S04 | `:first-child` / `:last-child` / `:only-child` | qualified (static box DOM) | Public selector corpus passes192 geometry/pixel frames plus384 clear controls, expected-color assertions, CLI/WASM parity and visual review. Existing grammar/admission restrictions remain; see validation/selectors-review.md. |
| S05 | `:nth-child` / `:nth-last-child` | qualified (static box DOM) | Public selector corpus passes192 geometry/pixel frames plus384 clear controls, expected-color assertions, CLI/WASM parity and visual review. Existing grammar/admission restrictions remain; see validation/selectors-review.md. |
| S06 | `:not()` | qualified (static box DOM) | Public selector corpus passes192 geometry/pixel frames plus384 clear controls, expected-color assertions, CLI/WASM parity and visual review. Existing grammar/admission restrictions remain; see validation/selectors-review.md. |
| S07 | `:is()` | qualified (static box DOM) | Public selector corpus passes192 geometry/pixel frames plus384 clear controls, expected-color assertions, CLI/WASM parity and visual review. Existing grammar/admission restrictions remain; see validation/selectors-review.md. |
| S08 | `:where()` | qualified (static box DOM) | Public selector corpus passes192 geometry/pixel frames plus384 clear controls, expected-color assertions, CLI/WASM parity and visual review. Existing grammar/admission restrictions remain; see validation/selectors-review.md. |
| S09 | Custom properties | partial | Compiler-owned environments and inheritance pass136 public/native frames. Bounded token data, aliases and CSS-wide custom defaults admitted; registrations and invalid-computed recovery remain pending. Lazy fallback cycles now pass128 additional frames; see validation/variable-cycles-review.md. See validation/variables-review.md. |
| S10 | `var()` and fallbacks | partial | Nested/empty fallbacks, direct cycles and token boundaries validated. Older eager graph caused an8-frame failure, preserved historically. Lazy resolution now matches pinnedChrome across16 cycle cases/128 native frames. Browser invalid-computed recovery remains pending. See validation/public-variable-receipt.json. |
| L01 | Reverse flex directions | qualified (single-line Chrome profile) | All41feature scenes pass328geometry/pixel frames and visual review with ordinary file ordering, compensated native direction/alignment, intrinsic/stretch sizing and same-file original/clone resize.63Rust/17Node tests pass. PinnedChrome reverse-overlap painting decision is explicit; no normative allCSS claim. Full87scene regression preserves two known fractional pixel failures. See validation/flex-direction-review.md and public-flex-final-receipt.json. |
| L02 | `order` | qualified (single-line Chrome profile) | Compiler-owned stable sorting preserves original DOM selectors/identities and handles all four directions, ties, signed integers, inheritance and variables. 19 scenes pass 152 geometry/pixel frames, 304 clear checks and complete visual review. 70 Rust/18 Node tests pass; 168 existing fixtures retain identical bytes/maps. Sorting retains integer keys instead of full sibling environments; measured memory regression fixed. See validation/order-review.md and public-order-receipt.json. |
| L03 | `align-self` | partial | Core, safe/logical, first and last baseline profiles implemented. Last baseline: 66 public scenes, 528 geometry/526 pixel passes, 1056 clear controls and complete visual review. Two fractional leaf failures preserved; 92 Rust/26 Node tests and 304 prior byte-identical outputs. Responsive participant metrics, out-of-box baselines and broader nested topology remain unresolved. See validation/last-baseline-review.md and public-last-baseline-receipt.json; earlier evidence retained. |
| L04 | `align-content` | pending | Positional cross-stretch wrappers pass the bounded start/center/end matrix, while normal/stretch controls fail. Overflow/auto cross-size follow-up passes192/192geometry/pixels; distributed lines, normal/stretch and public admission remain unresolved. See validation/flex-line-audit.md. |
| L05 | `space-around` / `space-evenly` | qualified (single-line main-axis profile) | Compiler-derived ordinary spacers:112 public scenes,896 geometry/pixel frames,1792 clear controls;98 Rust/28 Node tests;370 prior outputs unchanged. Distributed-column baseline contexts diagnose and wrapping is separate. See validation/spacing-review.md and public-spacing-receipt.json. |
| L06 | `wrap-reverse` | pending | Foreground direction matrix passes128/128geometry but48/128pixels. Ordinary+wrap and reverse+wrap-reverse have passing static-chain candidates; mixed reversal fails as line membership changes. Nested painting and general compositions remain unresolved. See validation/foreground-directions-review.md and validation/flex-line-audit.md. |
| L07 | Auto margins | pending | Immutable-target audit and revalidation required; historical status: native-qualified. |
| L08 | Independent grow/shrink | pending | Immutable-target audit and revalidation required; historical status: native-qualified. |
| L09 | Sub-unit flex factors | pending | Immutable-target audit and revalidation required; historical status: native-qualified. |
| L10 | Content-derived auto basis | pending | Immutable-target audit and revalidation required; historical status: native-qualified. |
| L11 | Percentage basis in indefinite containers | pending | Immutable-target audit and revalidation required; historical status: native-qualified. |
| L12 | Content-box sizing | pending | Immutable-target audit and revalidation required; historical status: native-qualified. |
| L13 | Aspect ratio | pending | Immutable-target audit and revalidation required; historical status: investigating. |
| L13a | Numeric math in aspect ratios | pending | Immutable-target audit and revalidation required; historical status: active. |
| L14 | Intrinsic image sizing | pending | Immutable-target audit and revalidation required; historical status: active. |
| L15 | Percentage padding/margins | pending | Immutable-target audit and revalidation required; historical status: partial. |
| L16 | Negative margins | pending | Immutable-target audit and revalidation required; historical status: native-qualified. |
| L17 | Relative positioning | pending | Immutable-target audit and revalidation required; historical status: partial. |
| L18 | Absolute positioning and insets | pending | Immutable-target audit and revalidation required; historical status: active. |
| L19 | Stacking order / z-index | pending | Immutable-target audit and revalidation required; historical status: qualified. |
| L20 | Overflow clipping | pending | Immutable-target audit and revalidation required; historical status: qualified. |
| P01 | Uniform solid borders | pending | Immutable-target audit and revalidation required; historical status: qualified. |
| P02 | Individual border sides | pending | Immutable-target audit and revalidation required; historical status: native-qualified. |
| P03 | Per-corner circular radii | investigating | Direct modest corners8/8; large top corners2/8pixels despite8/8geometry. Clipped ordinary paint-child composition8/8in bounded min-height case. All reviewed; general normalization and public admission pending. See validation/ordinary-layout-review.md. |
| P04 | Elliptical radii | pending | Immutable-target audit and revalidation required; historical status: native-qualified. |
| P05 | Group opacity | pending | Immutable-target audit and revalidation required; historical status: active. |
| P06 | Linear gradients | pending | Immutable-target audit and revalidation required; historical status: qualified. |
| P07 | Radial gradients | pending | Immutable-target audit and revalidation required; historical status: pending. |
| P08 | Outer box shadows | pending | Immutable-target audit and revalidation required; historical status: pending. |
| P09 | Inset box shadows | pending | Immutable-target audit and revalidation required; historical status: pending. |
| P10 | Background images | pending | Immutable-target audit and revalidation required; historical status: pending. |
| P11 | 2D transforms | pending | Immutable-target audit and revalidation required; historical status: pending. |
| P12 | Transform origin | pending | Immutable-target audit and revalidation required; historical status: pending. |
| I01 | `object-fit: contain` | pending | Immutable-target audit and revalidation required; historical status: pending. |
| I02 | `object-fit: cover` | pending | Immutable-target audit and revalidation required; historical status: pending. |
| I03 | `object-position` | pending | Immutable-target audit and revalidation required; historical status: pending. |
| I04 | JPEG assets | pending | Immutable-target audit and revalidation required; historical status: pending. |
| I05 | WebP assets | pending | Immutable-target audit and revalidation required; historical status: pending. |
| I06 | Image orientation | pending | Immutable-target audit and revalidation required; historical status: pending. |
| I07 | Color-managed images | pending | Immutable-target audit and revalidation required; historical status: pending. |
| I08 | SVG assets | pending | Immutable-target audit and revalidation required; historical status: pending. |
| I09 | Asset deduplication | pending | Immutable-target audit and revalidation required; historical status: pending. |
| T01 | Mixed text and inline spans | pending | Immutable-target audit and revalidation required; historical status: pending. |
| T02 | Per-run typography | pending | Immutable-target audit and revalidation required; historical status: pending. |
| T03 | Bold/italic semantic tags | pending | Immutable-target audit and revalidation required; historical status: pending. |
| T04 | Multiple font weights | pending | Immutable-target audit and revalidation required; historical status: pending. |
| T05 | Italic fonts | pending | Immutable-target audit and revalidation required; historical status: pending. |
| T06 | Font fallback | pending | Immutable-target audit and revalidation required; historical status: pending. |
| T07 | Variable fonts | pending | Immutable-target audit and revalidation required; historical status: pending. |
| T08 | Complex-script qualification | pending | Immutable-target audit and revalidation required; historical status: pending. |
| T09 | Bidirectional text | pending | Immutable-target audit and revalidation required; historical status: pending. |
| R01 | Viewport units | pending | Immutable-target audit and revalidation required; historical status: pending. |
| R02 | `calc()` | pending | Immutable-target audit and revalidation required; historical status: pending. |
| R03 | `min()` / `max()` / `clamp()` | pending | Immutable-target audit and revalidation required; historical status: pending. |
| R04 | Media queries | pending | Immutable-target audit and revalidation required; historical status: pending. |
| R05 | Container queries | pending | Immutable-target audit and revalidation required; historical status: pending. |
| Q01 | Precise source locations | pending | Immutable-target audit and revalidation required; historical status: pending. |
| Q02 | Actionable diagnostics | pending | Immutable-target audit and revalidation required; historical status: pending. |
| Q03 | Machine-readable capability manifest | pending | Immutable-target audit and revalidation required; historical status: pending. |
| Q04 | Determinism expansion | pending | Immutable-target audit and revalidation required; historical status: pending. |
| Q05 | Malformed-input fuzzing | pending | Immutable-target audit and revalidation required; historical status: pending. |
| Q06 | Resource-limit boundary coverage | pending | Immutable-target audit and revalidation required; historical status: pending. |
| Q07 | Performance benchmarks | pending | Immutable-target audit and revalidation required; historical status: pending. |
| Q08 | Package/version compatibility | pending | Immutable-target audit and revalidation required; historical status: pending. |
| Q09 | Realistic composition corpus | pending | Immutable-target audit and revalidation required; historical status: partial. |
| Q10 | Deterministic interaction matrix | partial | Bytes-only immutable baseline probe imports once, clones once, and resizes seven scenes through four views;56geometry and50pixelpasses,6preserved corner failures. No interactions added. See validation/ordinary-layout-receipt.json. |
