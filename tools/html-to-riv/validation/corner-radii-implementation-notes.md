# P03 implementation and qualification notes

Status: implemented; focused initial and composition evidence complete. Full
native regression and broad gallery audit remain pending. P02 is native-qualified.

Accepted syntax: one-to-four nonnegative circular lengths in `border-radius`,
and one circular length in each physical corner longhand. Supported length
resolution includes px/em/rem and unitless zero. Shorthand expansion uses CSS
TL/TR/BR/BL order. CSS-wide keywords and custom-property substitution preserve
selected-corner semantics. Elliptical pairs/slashes, percentages and general
math remain excluded pending P04 and responsive-expression work.

The compiler stores four radii and unlinks native corner fields only when unequal.
Uniform output retains the established linked form. Runtime CSS paths now apply
proportional overlap reduction for unequal corners; the original oversized-corner
failure and corrected192-frame replay remain preserved. No tolerance was widened.

Completed evidence:

- Initial24 scenes:72 directly inspected views,192 original/clone resize frames,
  and fresh public artifact reproduction complete.
- Text/image/clip-margin compositions36 scenes:108 reviewed views (54 direct,
  54 exact full-image transfers),288 resize/clone frames and36 reproduced artifacts.
- Public Rust corner tests, full module404 tests before the latest additions,
  and expanded native/WASM parity13 tests pass. Additional all-corner cascade
  regressions are recorded separately; consult their log for terminal status.
- Source gallery audits verify7362 prior pairs and72 new initial-view references.
  Full7444-check target regression is still running; no full-target pass claimed.

Remaining qualification work:

- Complete full regression and exact input/artifact/full-image gallery audits.
- Expand isolated nonzero-corner and fractional/large-radius browser coverage,
  including nested flex compositions and selected-corner inheritance/reset cases.
- Preserve documented curve antialias and image sampling differences. Broader
  vector-renderer qualification remains separate from the native profile.
- Runtime `paint_geometry` swaps physical corners in RTL. CSS direction controls
  are currently outside the compiler profile; physical RTL placement must be
  explicitly qualified before admitting them (T09), rather than inferred from
  the current LTR tests.

Implementation: `src/style.rs`, `src/substitution_validity.rs`,
`tests/corner_radii.rs`, `tests/border_runtime.rs`, and runtime
`layout_component.rs` / `css_clip_path.rs`. Durable receipts live under
`output/playwright/html-to-riv/corner-radii-*`.
