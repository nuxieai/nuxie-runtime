# Public wrapping implementation plan

Integrate wrapping as a compiler-owned ordinary-file lowering, with context-specific admission and exact cost accounting. This plan advances the existing 99-item backlog; it does not replace the backlog with a smaller wrapping-only goal. No public wrapping profile is qualified by this document. Runtime, renderer, schema, dependencies and root build configuration remain immutable at the target in `TARGET.md`.

## Evidence and current boundaries

The existing `compiler.rs:441` child emitter combines authored ordering, layout, fill emission, source mapping and baseline summaries. Its sizing transfer at `compiler.rs:487` is useful, but extending its perpendicular wrapper indiscriminately is incorrect.

| Evidence | What it establishes | What it does not establish |
| --- | --- | --- |
| `line-alignment-wrappers-review.md` and `line-alignment-boundaries-review.md` | Fixed positional item wrappers separate line start/center/end from item alignment in tested fixtures. | Default normal/stretch line sizing or general item sizing. |
| `wrapped-sizing-audit.md` and counterexample receipt | Naive cross percentages change containing blocks; all-item wrappers lose empty auto-cross stretch. | A runtime impossibility. |
| `wrapped-auto-stretch-bypass-receipt.json` | Empty auto-cross explicit stretch can bypass the positional wrapper in twelve tested contexts. | Intrinsic children, bounds, fixed-cross stretch, or combined paint behavior. |
| `wrapped-percent-slots-review.md` and control review | Direct sizing slots plus an ordinary scalar graph can preserve percentage/bounded cross sizing and independently move visible objects/descendants. | General zero/tiny line discrimination or unbounded carry reset; eight fractional pixel failures remain preserved. |
| `wrapped-paint-alignment-review.md`, `mixed-wrap-groups-alpha-review.md` | ForegroundLayoutDrawable subtree/group ordering candidates address demonstrated nested alpha paint-order failures. | Arbitrary deep/nested wrap scopes, clipping/opacity, or combined percentage-slot and paint qualification. |
| `mixed-wrap-gate-review.md`, `mixed-wrap-resource-review.md` | Concrete tiny/zero gate failures and quadratic group-composition cost. | Universal resource or device-performance guarantees. |

The combined percentage sizing and paint matrix now passes all384geometry/pixel frames, with96directly reviewed pairs and288full-RGBA transfers; see wrapped-combined-slots-review.md. The optimized paint construction independently passes192frames across N3/N8 and reduces the one-paint-per-item cost to7N²+6N−7; see mixed-wrap-optimized-review.md. Combined optimized output also passes384frames and is RGBA-identical to the reviewed combined corpus, reducing paint overhead195→110records; see wrapped-combined-optimized-review.md. These remain adapter proofs. Public admission still depends on the coordinate/gate, sizing, paint-scope and resource guards below.

Implementation has begun in src/wrapping.rs: a compiler-private variable-count sizing graph reproduces all48three-item experimental sizing outputs byte-for-byte. Exact preflight counts and plan validation are tested; CSS routing is intentionally not yet connected. The private paint lowering now also reproduces48final optimized files, with explicit paint ownership and varying-count preflight; see wrapping-paint-lowering-review.md. See wrapping-module-review.md for sizing.

## Module boundary and scalar types

Add a compiler-private `wrapping` module alongside `baseline` and `spacing`. It owns admission, sizing/positioning plans, native wrapping fields and wrap-scope completion. Share an ordinary scalar-constraint builder only if two proven compositions need identical primitives; avoid a generic expression language or host evaluator.

Suggested types, not committed API:

```rust
enum FlexWrap { NoWrap, Wrap, WrapReverse }
enum LineAlignment { Normal, Stretch, Positional(LinePosition) }
struct WrapContext { axis: Axis, reverse_main: bool, reverse_cross: bool,
                     line_alignment: LineAlignment }
struct ItemLayoutPlan { /* outer/inner sizes, bounds, native directions,
                          alignment strategy; scalar fields only */ }
struct ItemHandles { authored_id: u32, slot_id: u32,
                     dom_index: usize, order: i32,
                     paint_group: PaintGroupId, metric: MetricSummary }
struct PaintEntry { drawable_id: u32, fill_id: u32,
                    geometry_owner: u32, scope: PaintScopeId }
struct EmissionCost { records: usize, paint_replicas: usize,
                      mask_objects: usize, constraint_edges: usize }
```

Keep authored `Style` separate from native plans. Retain only scalar sizing facts, object IDs and paint descriptors across siblings. The existing custom-property memory regression must not return through storing complete sibling styles/environments.

`plan_item(authored_style, parent_context)` chooses a proven strategy: existing no-wrap lowering; perpendicular positional wrapper; direct auto-cross stretch candidate; or definite sizing slot with a 100%-sized visible child and deferred translation. The plan must not select a strategy merely because its property names are recognized. It returns a contextual diagnostic for unresolved combinations.

`finish_scope(context, items, paint_groups, budget)` emits measurement/positioning and paint-order compositions after authored geometry exists. All constraints measure sizing slots only. Visible constrained children must never feed the same scope's sizing measurements.

## Integration sequence

1. **Syntax and authored state.** Parse `flex-wrap` and `align-content` through literal validation and computed-value resolution, including CSS-wide keywords, variables and inheritance. Preserve `normal` as the initial line alignment. Never silently map omitted/default normal or stretch to positional start. Until a line-stretch composition is qualified, an affected wrapped context diagnoses; explicit positional declarations can follow their own qualified path. Keep intentional syntax rejection consistent with the compiler's existing losing/unmatched-declaration policy.
2. **Authored collection and identities.** Continue selecting/cascading against the original DOM. Preserve stable `(order, DOM index)` ordering from `compiler.rs:470`. Collect handles for each emitted authored child without inserting slots into selector ancestry or sibling indices. Source maps continue to identify the visible authored object and original DOM path. Numeric object IDs may change in new wrapping output; their semantic mapping must not. No-wrap output must retain byte identity.
3. **Sizing slots and wrappers.** Transfer main dimension/min/max once to the actual line-breaking participant. Clear inner main dimension/bounds where the existing transfer requires it; do not double-apply percentages. Direct slots keep cross percentages/bounds under the original flex container, while visible children receive resolved 100% dimensions. Admit this only when slot sizing is independent of child content. Keep stretch auto-cross distinct from fixed-cross stretch. Preserve original authored style when recursing into descendants, as at `compiler.rs:514`.
4. **Measurement and positioning.** Derive axis projections, slot extents and common line anchors from immutable geometry. Emit independent forward/backward maxima and visible offsets after all slots exist. Generalize the three-item prototype's indices and fractions from computed styles; do not retain fixture names/constants or fixed child counts. Use proven world/local spaces and explicit dependency direction. Line sorting and positional alignment must remain separate from paint order.
5. **Paint collection and scope completion.** Record each item's background and descendant paints in ancestor-before-descendant order, retaining the geometry owner. Use ForegroundLayoutDrawables and proven DrawRules/DrawTarget arrangements on existing layout geometry. Reversing group order must not reverse paints inside each group. Attach each paint once unless the chosen gated composition explicitly charges replicas. Keep container background/footer scope boundaries intact. Nested wrapping requires a scope tree and proof that an inner scope cannot be reordered through its ancestor's background or escape its paint group.
6. **Cost preflight and emission.** Calculate exact expansion before building records. Use checked arithmetic for item×paint/group products, record IDs and encoded size estimates; then charge actual emissions and assert agreement with the plan. Include all slots, scalar graph objects, constraints, masks, foreground replicas and draw links. Charge scene-wide sums across nested scopes, not just each container. Existing 8,192 authored-element admission does not bound quadratic expansion acceptably. Establish a documented resource ceiling from the actual chosen composition and validated lifecycle envelope before public admission; do not turn a single-machine timing into an arbitrary production performance promise.
7. **Public qualification.** Compile fixtures directly from original HTML/CSS through Rust, CLI, WASM and JavaScript. Experimental adapters are replaced by the public implementation, not installed as runtime hooks. Freeze exact outputs and source provenance; complete browser/native validation before expanding support documentation.

Refactor this seam incrementally with old-output regression coverage. Do not require a large new frontend IR solely to add wrapping, and do not reserialize or rewrite already qualified no-wrap geometry unless a demonstrated integration need justifies it.

## Mandatory contextual guards and unresolved work

- **Line default normal/stretch:** positional line proofs do not implement stretched line cross extent. Recognition and CSS initial values must not imply admission.
- **Intrinsic sizes:** auto/intrinsic slots can create feedback through their 100% visible children. Main intrinsic contributions, cross bounds, percentage definiteness, nested content and descendants need separate measurements/proofs. Preserve conservative diagnostics rather than browser-baking dimensions.
- **Floating-point and coordinate admission:** `wrapped-coordinate-admission.md` records nonzero normalizer residuals and a concrete dead-zone/snapped-gate candidate. Positive geometric minima alone do not prove same-line floating equality or mask coverage. The snapped-gate probe now demonstrates exact binary signals for ten bounded constant-anchor cases and preserves a threshold-transition failure. wrapped-anchor-error-audit.md derives a conditional31-operation error budget but still requires native target/final-size equality and intermediate-coordinate bounds. Implement and validate an arithmetic/coordinate certificate before public routing.
- **Line membership gates:** DistanceConstraint's `<0.001` early return and coincident zero-extent line anchors are real failures. Derive a conservative positive-separation proof from authored constraints for any admitted profile, or develop another discriminator. A sampled resize suite is insufficient to establish all allowed sizes. Do not impose an unrequested minimum size on the authored design to make a gate work.
- **Carry and coordinate bounds:** the current scalar penalty 65,536 only resets smaller line maxima. The mask experiments also use finite covering rectangles. Prove extent and coordinate envelopes, including negative overflow, ancestor offsets, repeated row/column placement and viewport changes. Diagnose inputs outside a proven envelope; do not silently crop paint or clamp design dimensions.
- **Baseline interaction:** first/last summaries assume packed single-column topology, and row helper participants can affect line assignment. Do not reuse `baseline::summarize`, `used_height` or `summarize_last` for multiline geometry without a proof. Initially reject affected wrapped baseline sharing and ancestor summaries that depend on unresolved multiline metrics; fixed independent heights can remain separately provable.
- **Main-axis distribution:** current `spacing::emit` inserts flexible synthetic participants. In a wrapping parent those participants may form their own lines or change where authored items wrap. Around/evenly integration requires a line-aware ordinary composition; reject the unproved combination, without disabling distribution in unrelated descendant scopes.
- **Nested scopes and painting:** no assumption that a global chain, duplicated subtree or finite mask preserves arbitrary nested wrap/clip/group behavior. Shared paint ownership, constraint targets and group dependencies require explicit tests.
- **Combined percentage plus paint:** the48-scene combined artifact covers displaced descendants, alpha overflow and mixed reversal. Broader nested scopes and coordinate domains remain unresolved. Retain the known fractional failures and controls separately.

These are integration gates and open engineering tasks, not declarations that features are impossible or finished backlog rows.

## Validation before each admission expansion

Use pinned Chrome 153.0.8010.12 and the frozen immutable probe/renderer, with source metadata discarded before import. Resize the same original and clone through one/two/three lines, exact-fit and overflow transitions, differing aspect ratios and return states. Include parent translations, both axes, both main reversals and both wrap modes admitted by that increment.

Test authored ordering ties, negative order, structural selectors, variables, inheritance and source-map descendants through the actual public interfaces. Sizing cases must activate main/cross min and max independently, show a percentage item moving while shorter than its line, and show nested descendants following. Include auto-cross stretch alongside fixed-cross stretch, zero/tiny gate boundaries, masks near coordinate limits, multiple containers and nested scopes. Paint cases need overlapping translucent backgrounds and descendants; keep regional pixel gates that caught failures invisible to a global mismatch ratio.

Reproduce deterministic RIV/maps across transports; inspect distinct native/Chrome image pairs and bind exact independent RGBA transfers for repeat states. Preserve failing candidate outputs and controls. Run lifecycle/resource tests against actual emitted cost, including import, clone, repeated resize and multiple scopes; report observed measurements with their machine/build context.

Update `SUPPORT.md`, `VALIDATION.md`, per-item evidence and progress only for the exact qualified contexts. Keep unresolved and evidenced limitation states distinct, and continue remaining backlog work after this integration milestone.
