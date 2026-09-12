# Failed variable substitution computes as unset

The public compiler now recovers missing or cyclic ordinary-property substitutions
when the resolver returns its guaranteed-invalid value. The declaration keeps its
source location, name, importance and cascade position; its computed value becomes
`unset`. Existing typed property computation then inherits or initializes the value
as CSS requires. Earlier declarations do not reappear. The lightweight sibling-order
prepass uses the same helper, so file ordering agrees with the final computed style.

This is compiler-owned computation before ordinary Rive emission. The custom-property
resolver, runtime, renderer and schema are unchanged. No host variable environment,
post-import setter, policy sidecar or resize recompilation supplies recovery.
See [the initial audit](variable-recovery-audit.md) for the explicit Chrome
counterexamples and [the receipt](public-variable-recovery-receipt.json) for bound
source, binary, checks and rendered evidence.

Only `Ok(None)` from variable substitution recovers. Successful token streams,
including empty values, still pass through the existing strict property admission.
Syntax and resource errors propagate. An unsupported initial value still diagnoses:
`display:var(--missing)` requires unsupported initial display, while failed `flex`
or `flex-shrink` requires nonlegacy shrink behavior. Unmatched and overridden
unsupported declarations remain errors. Whole invalid expressions reset the whole
property; unused fallbacks remain skipped by the existing lazy resolver.

The profile therefore expands S09/S10 without completing them. Successfully
substituted wrong-typed tokens (`--x:20px;background:var(--x)`), valid empty values,
token-boundary invalidity and broader registrations/contexts remain pending. These
are compiler gaps, not established immutable-runtime limitations. Unsupported valid
CSS and numeric/context resource limits must not be mistaken for typed invalidity.

## Validation and direct visual review

The final source passes **224 Rust tests and 38 Node tests**, native/WASM builds and
strict TypeScript checks. Twenty-four positive sources match independently authored
explicit-unset controls in Rust at three viewports, with exact complete output and
determinism checks. CLI/WASM output parity adds 72 pairs; 16 rejected inputs have
identical diagnostics and create neither RIV nor source-map files. Additional tests
cover inline styles, specificity, importance, DOM-based selectors after order
changes, original diagnostic source locations, and resource failures after an
already-failed substitution. A valid 24 KiB variable environment with one later
reference recovers; three later references exceed the 64 KiB expanded value bound
and still diagnose. The prior public corpus remains exact at **605/605 outputs**.

The immutable native run compiles each source once, imports ordinary bytes with
metadata excluded, and resizes its original and clone through
240×160 → 390×200 → 768×120 → 240×160. All **192 geometry and pixel comparisons**
and **384 alternate-clear comparisons** pass against Chrome 153.0.8010.12 using
the existing gates and RustMetal RasterOrdering. Expected background-color
assertions cover every authored box in every frame. Every measured box fits the
full captured viewport.

All **26 distinct full-frame pairs** were inspected at original resolution in
eight sheets. This includes every scene's first pair and both larger viewports
of the responsive auto-width case. The inspected frames show inherited navy paint,
48px font-relative width, removal of coral after background invalidation, auto
widths/intrinsic height, reset order while DOM selectors still identify the original
first child, row direction, zeroed shorthand components with later longhands retained,
and initial min/max behavior. No visible geometry or painting mismatch was found.

The remaining **166 pairs** transfer from those inspected pairs by exact complete
RGBA equality after explicitly extending both canvases with white to a common size.
The responsive case transfers only between identical viewport sizes. This verifies
all cropped-away regions are white; it is not a same-dimension image-hash claim.
Every source image is hash-checked against the render receipt, and each sheet's
source placement is verified byte for byte without scaling or cropping.

The first full Rust run preserved one failure from an old test that intentionally
rejected missing variables. Existing gap/order/general-variable negative checks
were updated to valid-empty or remaining invalid examples, while missing/cyclic
recovery is now positively covered. The original failure log and frozen pre-change
30-case rejection audit remain available. No pixel tolerances changed and no
failed native render was removed from this qualification.

Gallery: `output/public-variable-recovery-r1/render/gallery.html`.
