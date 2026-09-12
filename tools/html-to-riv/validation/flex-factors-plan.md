# Public flex grow, shrink and basis implementation plan

Status: proposed implementation sequence, not qualified public support. Preserve the immutable runtime/schema/dependency boundary. The source findings and arithmetic counterexamples are in [flex-factors-audit.md](flex-factors-audit.md); that audit does not establish ordinary-file import or Chrome parity.

## Deliverable and sequence

Add a compiler-private `flex.rs` that owns computed flex values, contextual admission, and an ordinary-item lowering descriptor. Route public CSS only after each profile passes the independent Chrome/native driver. Keep computed author values separate from native fields and from helper participants. Implement these stages in order:

| Stage | Candidate public profile | Native encoding / remaining proof |
| --- | --- | --- |
| F0 | Existing `0 0 auto`, including explicit declarations | Preserve existing bytes exactly; grammar/cascade integration only. |
| F1 | Zero **point** basis; arbitrary nonnegative grow/shrink; automatic preferred main size; definite main parent | Main Fill, fraction=grow, explicit basis 0/Point. Whole-parent sibling and helper guards below are mandatory. |
| F2 | Equal grow/shrink, point basis or auto basis; automatic preferred main size | Main Fill, fraction=both, configured basis/units. These fields already exist on ordinary LayoutComponent. Qualify fixed basis first; auto basis separately with nested content. |
| F3 | Independent factors whose irrelevant branch is provably never used | A compile-time, viewport-independent positive- or negative-free-space certificate may select a tied factor without changing the reachable branch. Min/max freeze loops must be included in that certificate. Otherwise diagnose. |
| F4 | Percent basis, explicit preferred main size, intrinsic parents and ancestors | Extend descriptors and proofs independently; do not infer them from the definite zero-point profile. |
| F5 | General independent factors | Develop an ordinary-file composition, with bounded resource cost and native/Chrome evidence. No general encoding is currently proven. |
| F6 | Helpers, baseline, wrapping and further layout combinations | Integrate each proven profile with two-stage leftover distribution, flexible baseline metrics and wrapped slot/paint lowering. No change to grid policy. |

F2 includes fraction zero with explicit point basis: a Fill item can have grow=shrink=0 and retain an explicit basis. This is a direct candidate for `0 0 <length>` when preferred main size is auto; do not replace it with a width overwrite or an unnecessary wrapper without testing the native path first. Keep `0 0 auto` on F0 to preserve old bytes.

## Computed CSS model and parser

Use `Flex { grow: f32, shrink: f32, basis: Basis }`, where Basis distinguishes Auto, point lengths, percentages, and any later supported content/intrinsic keywords. Resolve em/rem basis lengths using the element's computed font size. Store the specified percentage descriptor through inheritance. Never turn zero percent into zero points merely because a sampled parent is definite.

The authoring reset is `flex: 0 0 auto`; therefore `Style::default()` retains grow 0, shrink 0, basis Auto. CSS `initial`/`unset` are different: grow 0, shrink **1**, basis Auto. Explicit `flex-shrink:initial` must not silently resolve to reset shrink 0. `inherit` copies each parent's computed field; shorthand inherit copies the triple. Non-inherited properties do not inherit implicitly.

Start routing longhands plus CSS-wide forms and F0-compatible declarations. Validate tokens with cssparser, including finite nonnegative numeric factors, complete token consumption, and strict diagnostics even for unmatched/overridden unsupported declarations as this compiler already does. Factor zero remains legal. Choose factor bounds from representability and measured sum/product safety; do not normalize factors or introduce an unexplained cap. Check aggregate arithmetic for the admitted authored-element bound.

Add `flex` shorthand only with explicit conformance tests for each grammar form and expansion: `none` -> `0 0 auto`, `auto` -> `1 1 auto`, CSS-wide reset -> `0 1 auto`; single/two-number forms supply the browser's omitted basis (normally 0%, distinct from 0px), basis-only supplies 1/1, and full grow/shrink/basis sets all fields. Test ambiguous unitless zero using pinned Chrome computed styles. The compiler may parse a valid shorthand and diagnose an unqualified resulting profile; it must not change its computed meaning to fit F1. Later longhands, earlier important longhands, variables and shorthand resets must all preserve cascade order.

## F1 admission: exact initial boundary

Preflight every authored sibling before emitting any main-axis helper or authored item. Admit only when all these conditions hold:

1. The container is in the current nowrap, horizontal-LTR, zero padding/border/gap profile. Its used main extent is proven definite, not merely positive in the initial viewport.
2. Every non-F0 item has zero **point** basis, automatic preferred main dimension, and finite nonnegative factors. Main minima are explicit finite point values (including reset zero); main maxima are absent or finite point values. Defer automatic/percentage min/max for first qualification.
3. Siblings are either these zero-basis items or unchanged F0 children. A positive-basis shrinking sibling invalidates the arbitrary-shrink proof for the whole group: unscaled shrink sums affect the below-one shrink rule.
4. No authored growth shares this parent with around/evenly or any sibling's main-axis auto margin. Existing flexible helper siblings would consume growth at the wrong phase. Test the entire group, not only the growing child's own margin. Initially reject these helpers for the whole F1/F2 profile even at grow zero if positive-basis shrinking is present, until negative-space interaction is independently qualified.
5. Do not admit baseline-dependent topology whose used vertical metrics depend on these flexible main sizes. Initially reject first/last baseline participation in the affected group and invalidate ancestor scalar metrics when a vertical main-size flex child is present. Unrelated fixed-height row flex containers may retain existing metrics only if their descendants' required baseline is independently known.
6. No public wrap routing in this milestone. Private wrapped sizing/paint plans must not consume a flex descriptor until their slot model incorporates its final main size.

The arithmetic proof for arbitrary shrink applies because zero bases have zero scaled shrink; positive minima freeze before shrinking; F0 siblings cannot shrink. It does not prove that changing shrink is harmless in all contexts.

## Definite-size and intrinsic dependency guard

Introduce a small measurement context instead of testing `Size::Auto` locally. Track per axis whether the containing size is definite and whether a descendant is currently contributing to an intrinsic measurement. Host dimensions are definite. Point sizes are definite when their own flex sizing does not change that interpretation. Percent sizes require a definite corresponding containing dimension. Stretch, flex-produced sizes, min/max-clamped Auto, and inferred intrinsic sums require explicit propagation rules; mark unknown until supported.

F1 should initially require a direct definite parent main size and reject any ancestor intrinsic measurement whose result depends on the flexible subtree along that axis. A grandparent with auto main size can request max-content contributions from a descendant even when a nearer box has a local size descriptor. Do not use initial compiled pixels, viewport sampling, or a parent's fixed cross size as proof of main definiteness. When a proof is too conservative, diagnose and add a targeted case before widening it.

For F2 equal factors the native factors/basis match exactly, but Fill still changes preferred main dimension to Auto. Keeping the authored main dimension Auto avoids this discrepancy. Intrinsic native-vs-Chrome behavior remains a separate qualification matrix, not an automatic consequence of equal factors.

## Source integration points

Locations refer to the current source and should be found by symbol when lines move:

- `src/compiler.rs` private module declarations, `Style` and `Style::default`: add computed Flex and host/reset default.
- `validate` (currently around line 238): extend property/var allowlists and delegate grammar validation. Keep substitution followed by validation.
- `computed` (around 297): apply shorthand/longhands after final font-size resolution, using parent computed descriptors and existing sorted cascade declarations.
- `Emitter::children` (around 453): perform a compact sibling preflight for flex/helper/measurement compatibility before spacing emission. Keep DOM source index and authored order; do not retain full per-sibling variable environments. A two-pass compute with compact flex summaries is preferable to duplicating megabyte variable maps per sibling.
- Per-item wrapper emission (around 532): identify the **outer participant in the authored parent's main axis**. Transfer flex basis/fraction and main bounds there when cross-alignment wrappers are present. The inner authored object remains the source-map/paint identity and stretches to the slot's main extent.
- `layout_box` (around 409): accept a private flex lowering descriptor or apply a dedicated checked mutation to returned record handles. Explicitly write main Fill, fractionalWidth/Height, flexBasis, and flexBasisUnitsValue. Do not assume all emitted boxes are authored flex participants: baseline, spacing and alignment helpers retain their own sizing rules. A no-op descriptor must preserve prior bytes.
- `src/baseline.rs`: pass actual used-height knowledge or unknown into Child; invalidate first/last propagated metrics for unresolved vertical flex sizing. Existing `used_height(style, children)` cannot blindly use the authored main height after flex distribution.
- `src/wrapping.rs` / `src/wrapping_paint.rs`: no initial changes; document F1/F2 as unavailable to this private composition until final-size slots and paint handles are qualified together.
- `tests/flex.rs`, Node/WASM parity tests, supported-feature documentation and a new validation fixture family: mirror actual admitted contexts and explicit rejection boundaries.

Do not modify `src/reset.css` to make unsupported declarations disappear. No runtime/schema change is needed for F0–F2's candidate fields.

## Chrome/native qualification matrix

Use a fresh output directory and frozen public binary/module snapshot for each candidate. Independently capture Chrome geometry/pixels from authored HTML/CSS and native geometry/pixels from compiled bytes; run original and cloned scenes through roomy, constrained, overflow, and return viewports. Maintain the existing direct visual review plus verified full-RGBA-transfer workflow. No unreviewed frame becomes visually qualified from geometry alone.

Initial F1 fixtures:

- All four directions; empty and nested painted items; siblings with order changes and alpha overlap paint.
- Grow factors 0, .25, .5, 1, 2; sums below/equal/above one; fixed siblings before/between/after flex items. Include parent 200/fixed40/grow.25 -> flex40, leaving120 unused.
- Mixed grow weights; authored shrink 0/.1/1/7 with zero point basis; overflow and positive minimum. Include parent50/min10 flexible item/fixed100 ->10+100 overflow.
- Min/max redistribution, simultaneous clamp violations and max below min. Include two .25 items with first max20 plus fixed40 in parent200 ->20/40/40.
- Cross stretch, start, center/end wrappers, safe overflow alignment and cross-auto margins; transfer flex to the outer slot and verify descendants still resolve percentages against the authored box.
- Nested fixed-size containers, top-level items, definite percent ancestry; separate rejection tests for intrinsic ancestors and unknown definiteness.
- CSS-wide values, inheritance, shorthand/longhand order, important precedence, variables, invalid/unmatched/overridden declarations, factor overflow, object bounds, stable source paths and no-flex byte identity.

F2 adds positive point bases, unequal bases, auto basis with content, shrink sums below one, shrink-weighted-by-base distribution, zero factors, and repeated min/max freezes. Keep the audit counterexamples as rejection regressions until a later encoding solves them: mixed zero-basis arbitrary shrink changes B50 to85, and independent grow1/shrink0/basis100 in parent50 must remain100 rather than50. Parent200/grow1/basis0/main-auto must remain child200; a100/100 result exposes helper competition.

For F4, explicitly compare 0px vs0%, percentage bases in row/column, definite versus intrinsic parents, automatic minima, explicit main size and descendants with percentage sizes. For F5/F6, each proposed composition needs emitted-record cost/preflight, native import/clone/resize bounds, source-identity and paint-order verification before adding language admission.

## Completion criteria per stage

A stage is complete only with grammar/cascade tests, meaningful native geometry and pixel cases, actual visual evidence, deterministic native/WASM outputs, no-flex byte/map regression checks, source-guard verification, documented support/rejection boundaries, and an immutable-runtime receipt. A failed case stays recorded and narrows the gate or drives a compiler-only fix. Broader independent shrink, intrinsic sizing or helper compositions remain “not yet qualified,” not “impossible.”
