# Direct leaf premises for the conditional flex bridge

Read-only review of current descriptors and immutable runtime. This is a proposed proof contract for compiler-produced records, not a generic RIV validator. It does not enable public flex or certify browser pixels.

## What follows for the restricted family

A direct LayoutComponent with no layout children, no measurement context, finite nonnegative target dimensions, zero padding/border, no aspect ratio or layout animation can preserve both target dimensions exactly at the leaf-layout handoff. Pinned Taffy `vendor/taffy-0.12.1-rive-yoga-order/src/compute/flexbox.rs:1934` passes item targets into child layout with `SizingMode::ContentSize`. In `src/compute/leaf.rs:37`, ContentSize uses known dimensions and ignores preferred/min/max/aspect styles; `:143` prefers known dimensions and `:151` only applies the padding/border floor. With zero floor and nonnegative known dimensions, these selections preserve the target f32 bits. This does not bypass parent flex min/max handling: the analyzer must already have proved the target and freeze behavior.

The native measurement closure in `crates/nuxie-runtime/src/mechanical_port/source/layout_component.rs:2243` returns zero for absent context (`:2273`) and preserves provided known dimensions (`:2276`). `is_intrinsic_leaf` at `:1747` requires both a leaf and intrinsically-sized style. The initial checker should require that flag false and no alternative participant/measurement host. Fill/SolidColor paint records are not layout children; check their precise ownership and allowed kinds so a disguised extra participant cannot enter the family.

An omitted native minimum on a Fixed axis initially maps to Auto, not an encoded point zero. The active LayoutComponent solve subsequently rewrites every flex child's Auto minimum on both axes to point zero (`layout_component.rs:2183-2205`), including Fixed children and explicit Auto. This later stage supersedes the earlier assumption that Taffy's content-based automatic minimum necessarily survives to the solve. Fill also has an earlier Undefined-to-zero conversion. Keep raw omission, explicit Auto and point zero distinct for CSS admission and diagnostics; in this closed flex family the final native effect of omission is zero. The CSS-side empty unmeasured leaf has zero min-content, so the compiler's zero-reset target remains compatible here. This does not establish browser automatic-minimum semantics for nonempty descendants.

A point cross size with omitted max and zero/leaf-auto min can preserve its cross target, but first check actual cross scale Fixed, point units, no aspect ratio, start alignment and absent automatic margins. A computed `start_aligned_cross` flag alone is insufficient. The final cross target supplied to leaf layout must correspond to the authored point value; verifying only the leaf's ability to preserve an arbitrary supplied target would be circular.

## Identity and direction requirements

The compiler header is 7.3 (`tools/html-to-riv/src/wire.rs:88`). Runtime `layout_component.rs:294-304` enables ComposeTransform for 7.3; therefore stored x/y/rotation/scale really matter. Require Node x/y=0, rotation=0, scaleX/Y=1 using pinned defaults plus present overrides, no ComponentOrigin, and no transform-affecting constraints/animation. Do not justify identity by legacy pre-7.3 behavior. Check final ancestors for layout interpolation inheritance as well as local style settings.

Own identity does not prove zero world error. Native composition still adds parent and local translations; see `flex-world-arithmetic-review.md`. The Artboard's origin, actual transform and host sizing error are independent required inputs.

LTR must cover both the Taffy direction field and native `LayoutSyncContext.is_ltr`. `layout_component.rs:2340` resolves actual direction from explicit style or inherited direction; `:2947` propagates inherited direction, and `:1700` uses actual direction to map logical edges. The all-Inherit chain rooted in ordinary default host behaves non-RTL, but a direct child's missing direction property alone does not establish that chain. An RTL ancestor is a counterexample even when a descendant style's direction field is omitted. Bind this to final parent IDs and allowed artboard ancestry, without trusting a caller-supplied boolean.

## Counterexamples and exclusions

- A nested layout box, intrinsic text/image host, or different measurement-capable type can make automatic min-content nonzero and change the final target.
- Positive padding or border floors can exceed the target even with max-size set; a leaf is not enough.
- Aspect ratio, cross automatic margins, center/end alignment, automatic cross size or wrapper transfer break the simple cross-target premise.
- Present max0 differs from omitted maximum. Present bound value with omitted units must not be interpreted by matching only the numeric zero.
- A ComponentOrigin or baseline constraint added after descriptor capture invalidates the own/ancestor transform premise.
- Style interpolation or animation can replace solved layout with interpolated values; a one-time successful import is insufficient.
- Missing style binding, a shared/mutated style, extra direct participant, unexpected record type, or RTL ancestry invalidates this compiler-specific certificate.

## Suggested executable checks

Construct the certificate only after full emission, from actual record IDs and a closed list of compiler record types. Verify participant/style links, complete direct child membership, exact paint ownership, computed-to-native scalar binding, both axes' scale/units/bounds, intrinsic flag, edges/gaps/aspect, alignment/direction, transform/default fields, and relevant ancestor constraints/interpolation. Reject unknown record types/properties in this restricted bridge rather than assuming today's emitter never writes them. Preserve a separate unresolved reason for each missing premise.

Regression controls should mutate one final-record fact at a time: add a layout descendant; turn on intrinsicallySized; add positive padding; change cross scale; add a nonzero origin/rotation/Node offset; introduce RTL on an ancestor; set interpolation; alter a style link or bound unit. Valid controls cover all four compiler directions, ordinary paint-only leaves, Fixed leaf automatic minima, Fill zero minima, and original/clone/resize. Analyzer success stays conditional on independently supplied parent-size/world envelopes.
