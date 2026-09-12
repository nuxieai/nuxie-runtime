# Image percentage padding: bounded composition audit

This is a **source-backed composition plan, not public admission or native qualification**. It reviews compiler checkpoint `017689927234360499efe5a9861211d73419bcc7`. At that checkpoint, nonzero image percentage padding is still diagnosed. This audit changed only this document; it did not build a compiler, invoke the public compiler, produce an ordinary file, or render a scene. The one executed calculation below evaluates explicitly rounded binary32 arithmetic without Rust or the runtime.

The useful initial candidate keeps percentage padding on the authored outer owner, where the immutable layout engine can resolve it against the original parent content width. For content-box point sizes, an automatic outer size can accumulate padding around an exact-size inner owner. That synthetic automatic size must not gain cross-axis stretch merely because the authored alignment is stretch. Border-box sizing can retain the existing outer dimensions and use the inner owner for content ratio and paint.

This extends the distinction established by [the point-padding plan](/Users/levi/.codex/worktrees/html-css-immutable/tools/html-to-riv/validation/image-padding-plan.md). That earlier plan missed percentage dimensions with padding only on the opposite axis; its R2 correction remains mandatory here. An authored percentage already resolved by the outer owner must become inner **100%**, not the same percentage again.

## Source and specification facts

For the admitted horizontal writing profile, all four percentage padding sides depend on the containing block's width, including top and bottom. Content-box specified sizes exclude padding; border-box specified sizes include it and floor content dimensions at zero. These are the intended CSS rules, not conclusions about the proposed file. [CSS Box 3, physical padding](https://www.w3.org/TR/css-box-3/#padding-physical), [CSS Sizing 3, box sizing](https://www.w3.org/TR/css-sizing-3/#box-sizing).

CSS cross-axis stretch applies to an authored automatic cross size, subject to its constraints. The synthetic `Auto` used to encode an authored point size is a compiler implementation detail and must preserve the authored definite-size behavior. [CSS Flexbox 1, stretch](https://www.w3.org/TR/css-flexbox-1/#valdef-align-items-stretch).

The source supports these narrower facts:

| Source | Relevant behavior |
| --- | --- |
| [images.rs:124](/Users/levi/.codex/worktrees/html-css-immutable/tools/html-to-riv/src/images.rs:124) | `padded_plan` presently rejects percentage padding. It retains point content sizes, uses inner 100% for resolved percentages/border-box axes, and places the ratio on an inner automatic axis. Both-auto/start uses source intrinsic point dimensions; both-auto/stretch has a separate synthetic outer 100% cross axis. |
| [box_sizing.rs:76](/Users/levi/.codex/worktrees/html-css-immutable/tools/html-to-riv/src/box_sizing.rs:76) | General content-box lowering rejects nonzero percentage padding before it can produce `Lowered`. It translates point-padding sizes and bounds with numerical provenance. |
| [compiler.rs:685](/Users/levi/.codex/worktrees/html-css-immutable/tools/html-to-riv/src/compiler.rs:685) | General lowering currently runs before `image.adjust_outer`, then the outer numerical guard runs. An adjustment after this call cannot enable content-box percentage padding. |
| [compiler.rs:549](/Users/levi/.codex/worktrees/html-css-immutable/tools/html-to-riv/src/compiler.rs:549) | `layout_box` encodes automatic cross dimensions as Fill when `stretch` is true, otherwise Hug. Point and percentage dimensions remain Fixed. |
| [LayoutParticipant::apply_base_style](/Users/levi/.codex/worktrees/html-css-immutable/crates/nuxie-runtime/src/mechanical_port/source/layout/layout_participant.rs:215) | Fill selects automatic dimensions and cross align-self stretch. Hug selects automatic dimensions without that override and retains zero main growth/shrink. |
| [LayoutComponentStyle](/Users/levi/.codex/worktrees/html-css-immutable/crates/nuxie-runtime/src/mechanical_port/source/layout/layout_component_style.rs:166) | The ordinary start-aligned owner uses start alignment. Cross Hug therefore does not acquire stretch through the native parent's default start alignment. |
| [Taffy flexbox.rs:176](/Users/levi/.codex/worktrees/html-css-immutable/vendor/taffy-0.12.1-rive-yoga-order/src/compute/flexbox.rs:176), [flexbox.rs:531](/Users/levi/.codex/worktrees/html-css-immutable/vendor/taffy-0.12.1-rive-yoga-order/src/compute/flexbox.rs:531) | Container and child padding resolve against parent content width. The ordinary runtime adapter uses the baseline border-box style, so the content-box-specific adjustment at flexbox.rs:714 is not the proposed encoding. |
| [Taffy flexbox.rs:953](/Users/levi/.codex/worktrees/html-css-immutable/vendor/taffy-0.12.1-rive-yoga-order/src/compute/flexbox.rs:953), [flexbox.rs:1881](/Users/levi/.codex/worktrees/html-css-immutable/vendor/taffy-0.12.1-rive-yoga-order/src/compute/flexbox.rs:1881) | Automatic container main/cross dimensions add measured content and padding. This is the mechanism being considered; it also introduces an addition that must be bounded. |
| [numeric.rs:69](/Users/levi/.codex/worktrees/html-css-immutable/tools/html-to-riv/src/numeric.rs:69) | `child_with_sizing` consumes lowered sizes/bounds but derives cross stretch again from computed `Style`. It must receive the actual emitted stretch decision for the new composition. |
| [numeric.rs:74](/Users/levi/.codex/worktrees/html-css-immutable/tools/html-to-riv/src/numeric.rs:74), [numeric.rs:28](/Users/levi/.codex/worktrees/html-css-immutable/tools/html-to-riv/src/numeric.rs:28) | The guard evaluates an inner owner separately, then checks image ratio/fit. It does not currently return from those inner bounds to check a content-derived outer automatic size. |

The three runtime/vendor files in the identity table below were compared against immutable baseline `6c7ac16617835b5f581784ff08a9e779bb52faf3` with `git diff --name-only`; the comparison was empty. No runtime or renderer change is part of this plan.

## Ownership and finite candidate profile

Keep the existing authored outer source identity, parent contribution, padding and background. Keep one unpainted inner owner with zero padding, the image clip, and the image ratio when required. The existing Image child still fills that inner owner and uses the existing fit/position/sampling fields. Do not move percentage padding inward: doing so changes its percentage basis from the original containing width to the authored box's content width.

The following table describes **candidates to qualify**, not a new support declaration. Existing restrictions on image min/max, auto margins, baseline, alignment wrappers, public flex factors, format admission and bounded parent dimensions remain in force.

| Authored sizing context | Proposed outer | Proposed inner | Initial disposition |
| --- | --- | --- | --- |
| Border-box, both axes definite points or resolvable percentages | Retain authored dimensions and ordinary padding floor | 100% of the resolved content in both axes | Candidate; include zero-content floors |
| Border-box, one definite axis and one ratio-derived auto axis | Retain the definite axis; retain the appropriate automatic axis | Definite-content axis 100%; ratio axis Auto | Candidate; parent axis and stretch decide which auto axis is ratio-derived |
| Content-box point dimension with nonzero percentage padding on that axis | Replace only that emitted outer dimension with Auto/Hug; retain responsive padding | Exact authored point dimension | Candidate; disable outer cross stretch if this is the cross dimension |
| Content-box point dimension with only point padding on that axis, while the other axis has percentage padding | Preserve existing point translation when practical | Exact authored point dimension | Candidate; do not needlessly change the proven point arithmetic |
| Content-box percentage dimension with exactly zero padding on the same axis | Retain the authored outer percentage | 100%, since the outer already resolves the percentage | Candidate, including percentage padding on the opposite axis |
| Content-box percentage dimension with nonzero same-axis padding | Existing diagnostic | No qualified plan | Unresolved; see narrower alternatives below |
| Both axes Auto, start alignment | Outer Hug in both axes | Intrinsic asset width/height as point sizes, without a ratio constraint | Separate candidate; intrinsic dimensions exclude padding in either box-sizing mode |
| Both axes Auto, stretch alignment | Existing synthetic outer 100% on the cross axis; automatic main axis | 100% cross content, ratio-derived automatic main | Separate candidate; preserve the existing definite-parent requirements |
| One point main axis, genuinely Auto/stretch cross axis | Main may become Hug to accumulate percentage padding; preserve cross Fill | Exact point main content, 100% stretched cross content; no ratio replacing stretch | Candidate; a blanket stretch disable would be wrong |
| Percentage padding with an unknown original parent content-width bound | Do not substitute viewport width or inner point width | No invented percentage basis | Retain diagnostic |
| Required inner 100% axis depends on an unknown automatic outer content axis | Do not certify a self-dependent measurement | No invented finite bound | Retain diagnostic unless a separate composition establishes that axis |

Default image minima are zero and maxima are absent in the current admitted profile. Responsive padding must not become a fabricated point minimum. Native padding floors already prevent a border box smaller than its padding. On an axis changed to Auto for responsive padding, retain the unshifted zero minimum/absent maximum and their honest source metadata; arbitrary nonzero minima or finite maxima remain outside this candidate. Do not open general non-image content-box percentage padding through this image-specific path.

### Cross stretch and layout phases

For a column parent with content size 200×120 and a 96×64 asset, content-box `width:96px;height:64px;padding:10% 5%` has intended point content 96×64 and intended outer 116×104 before browser layout quantization. Both percentage padding pairs depend on **200**, not the image width or parent height. Width is an authored definite cross dimension. If changing it to Auto accidentally emits Fill, the outer becomes parent-wide instead of 116. In a row parent, the corresponding authored definite height is the cross dimension and must likewise remain Hug after the synthetic conversion.

Conversely, in a row parent, content-box `width:96px;height:auto;padding:10% 5%;align-self:stretch` still has a genuinely automatic cross dimension. The candidate should preserve outer cross stretch to 120, leaving content height 80, while the main contribution is 96+20=116. Disabling stretch just because **width** became Auto would instead invite intrinsic cross measurement and change that behavior.

Select the ratio axis from the authored automatic axes and authored effective alignment, before converting point outer axes to Auto. Otherwise an implementation detail can incorrectly create another ratio-derived axis. Preserve the actual `ContentOwner.packing` choice in both emission and the inner numerical guard. This matters for the existing one-axis ratio cases where the synthetic owner's packing axis differs from the authored parent axis.

These are falsifiable predictions. Native flex measurement can visit indefinite sizing phases before final layout; a finite final algebraic size does not prove identical measurement, stretch or sibling placement. Root's ordinary-file experiment must observe both inner content and outer/tail geometry across resizing.

## A concrete compiler-owned interface

The present pair of general lowering followed by `image.adjust_outer` splits a single layout decision across admission, emission and the numerical guard. Keep the seam small, but return the complete padded image layout before general content-box lowering rejects it. One concrete interface shape is:

```rust
// Illustrative private interface, not implemented by this audit.
struct PaddedLayout {
    outer: box_sizing::Lowered,
    outer_cross_stretch: bool,
    content: box_sizing::ContentOwner,
}

fn lower_padded_image(
    metadata: ImageMetadata,
    authored: &Style,
    parent: &Style,
    source: &str,
) -> Result<PaddedLayout, Diagnostic>;
```

The existing `Plan` can own this result along with metadata and the authored-derived `aspect_axis`. A query can derive the genuinely content-sized outer axes from the actual emitted `Auto` dimensions and cross-stretch flag; an independent mutable mask is unnecessary. Keep the existing zero-padding path unchanged to preserve its qualified byte output. Non-images continue through existing general lowering.

`layout_box` and `Bounds::child_with_sizing` must consume the same outer cross-stretch decision. A narrow explicit argument or a borrowed shared emitted-layout view suffices; a second computed-style clone does not. The guard sequence is:

1. Evaluate actual outer dimensions, constraints and percentage padding against the **original parent Bounds**, with actual emitted stretch.
2. Evaluate the zero-padding inner `ContentOwner` using its actual sizes, bounds and packing.
3. Apply `Bounds::image` to the inner content, establishing finite ratio and fit dimensions.
4. For each genuinely content-derived outer Hug axis, validate an outward-rounded sum of inner content upper bound plus that axis's padding sum resolved against the **original parent's width**. Retain unknown as a diagnostic if the required bound is missing.

The last check is different from fixed or Fill outer axes, where native clamp/floor and subtraction establish content. Do not impose a content-plus-padding equality on a fixed border-box axis. Do not overwrite the inner content bounds with outer sizes after checking the sum: paint/ratio admission still needs the inner dimensions.

Keep computed `Style` intact for inheritance, effective alignment, authored diagnostics and source mapping. Synthetic Auto requires `NumericSize::Auto` in the emitted outer view; synthetic 100% needs the existing exact-constant coefficient carrier. Preserve the authored point carrier on the inner point size and original per-side padding provenance. A decoded token or source decimal that rounds to native zero is not proof of an exact zero source expression. Where the existing path carries unresolved unit provenance for tiny percentage padding, preserve that unresolved state rather than relabeling it as point padding.

The descriptor remains diagnostic metadata for authored child lists, not a complete enumeration or certificate of every native owner. Retain padding/content-owner structural barriers and the hard proof-path guards. Do not describe synthetic Auto or 100% as authored coefficients, and do not infer exact composed geometry from an exact synthetic 100 coefficient. Images have no authored children, so this work does not require inventing a synthetic source identity or a new generic proof framework.

## Required numerical addition check: a concrete counterexample

The current sequential outer/inner/image checks are insufficient if their percentage-padding admission barrier is simply removed. The following hypothetical source uses only bounded source coefficients and legal image dimensions:

```html
<!-- Seven nested .grow containers, then one .basis container. -->
<div class="grow"><div class="grow"><div class="grow">
<div class="grow"><div class="grow"><div class="grow"><div class="grow">
  <div class="basis"><img src="tall"></div>
</div></div></div></div></div></div></div>
```

```css
.grow, .basis { display:flex; flex-direction:column; }
.grow { width:1000000%; }
.basis { width:10000%; }
img {
  box-sizing:content-box;
  width:auto; height:auto; align-self:stretch;
  padding:1000000% 0;
}
```

`tall` denotes a valid admitted 1×8192 PNG, not a generated fixture in this audit. At the upper supported viewport width 16384, the parent width after seven ×10000 factors and one ×100 factor is about 1.6384e34. Native percentage dimensions use multiply-then-scale; padding uses divide-then-multiply. Both-auto/stretch supplies inner width from the outer 100% cross dimension. The asset ratio is exactly 1/8192.

The following standalone Python calculation was executed once, using `struct` to round each specified operation to binary32. It is an arithmetic control, not a CLI admission or native-layout result:

```python
import math, struct
f = lambda x: struct.unpack('f', struct.pack('f', x))[0]
width = f(16384.)
for percent in [1000000.] * 7 + [10000.]:
    width = f(f(f(percent) * width) * f(.01))
padding = f(f(1000000. / 100.) * width)
padding_sum = f(padding + padding)
inner_width = f(f(100. * width) * f(.01))
ratio = f(1. / 8192.)
inner_height = f(inner_width * f(1. / ratio))
try:
    outer_height = f(inner_height + padding_sum)
except OverflowError:
    outer_height = math.inf
```

| Quantity | Executed binary32 result |
| --- | ---: |
| Parent width | 1.638399529477753e34 |
| Each vertical padding | 1.6383995599805955e38 |
| Vertical padding sum | 3.276799119961191e38 |
| Inner 100% width | 1.638399529477753e34 |
| Ratio-derived content height | 1.3421768945481752e38 |
| Content height plus padding sum | infinity |

Every earlier dimension multiplication, each padding side, their sum, and the ratio/fit dimensions are finite. The existing guard leaves automatic outer height unknown; the inner percentage width and subsequent ratio recover finite inner bounds without ever checking the final outer addition. The expected failure is supported by Taffy's automatic main-size addition at flexbox.rs:975/1167. Actual public reproduction remains pending and must be a compile-only diagnostic control, not a giant scene render. Current public source rejects the percentage-padding profile before this issue is reachable.

The proposed final content-derived-outer check should reject this control, with outward rounding at both the per-side padding operations and the final addition. It need not solve arbitrary intrinsic container aggregation or world-coordinate arithmetic; those remain separately documented limits. Fixed content point sizes can survive enormous outer rounding because the actual inner point object remains separate, but that does not prove the outer sum finite.

## Percentage sizes with same-axis padding: keep the question open

Same-axis **zero** padding is straightforward: leave the percentage on the original outer axis, then fill it with inner 100%. This remains true when the opposite axis has responsive padding. Width 50% with only vertical percentage padding and height:50% with only horizontal percentage padding are required controls, not exclusions. Height still needs the existing definite containing-height admission.

For width percentages plus horizontal padding made entirely of percentages, there is a narrower algebraic candidate: with original parent content width C, width p%, left a% and right b% suggest an outer coefficient p+a+b and inner 100% after ordinary padding. This uses the same width basis for all three terms. It is **not yet a numerical equivalence proof**: dimension `p*C*.01`, padding `(a/100)*C`, scalar addition, padding subtraction and browser source precision have different rounding paths. Large cancellation can erase the intended content, and a derived coefficient may exceed the authored coefficient limit without being intrinsically invalid. This alternative needs explicit provenance, finite/intermediate bounds, and content/paint tests before it can replace the diagnostic.

That scalar shortcut does not directly cover height percentages plus vertical percentage padding: their independent bases are parent height and parent width. Mixed point padding similarly introduces an independent additive length. A possible further composition would preserve the original containing-size carrier while another ordinary owner accumulates content plus padding, but a naïve extra 100% carrier contributes its own full extent to parent layout or makes the child's percentage resolve against the wrong owner. A real proposal must preserve outer parent contribution, original percentage basis and paint ownership simultaneously. No source fact here proves such a composition impossible; none is qualified by this audit.

## Finite validation matrix before public admission

Use existing source asset bytes and ordinary-file native/Chrome workflow. The following is a bounded proposed set of discriminators; combine redundant transport inputs after their byte identity is established. Expected numbers above describe CSS intent before layout quantization. Browser measurements, existing tolerances, local content pixels, original outer/tail gates, and same-file original/clone resizes decide qualification.

| Control | Distinction it must expose |
| --- | --- |
| Border-box fixed width/height; asymmetric percentage sides | Parent-width basis and outer/background versus inner paint ownership |
| Border-box width percentage with vertical and horizontal percentage padding | Outer percentage applies once; percentage padding remains based on the original parent width |
| Border-box fixed width/Auto height and fixed height/Auto width | Ratio uses content, not padded outer extent; both packing axes |
| Border-box padding equal to and greater than a fixed outer dimension | Correct zero-content floor; explicit blank-content presence expectation |
| Content-box 96×64, padding:10% 5%, column parent 200×120, default stretch | Definite cross width converted to Hug, not Fill; expected outer 116×104 |
| Same fixed content/padding in a row parent | Definite cross height converted to Hug |
| Content-box point main dimension plus genuine Auto/stretch cross, row and column | Main synthetic Auto must not disable legitimate cross stretch |
| Content-box one point cross dimension plus ratio-derived Auto main, row and column | Synthetic Auto must not change ratio-axis selection |
| Content-box one point axis with point padding; percentage padding only on the other axis | Existing point translation and exact inner size remain consistent |
| Both-auto/start intrinsic, both box-sizing modes | Natural content dimensions remain 96×64; padding adds outside them |
| Both-auto/stretch, row and column with definite parent axes | Separate stretch/ratio sequence, including parent-height requirement in row |
| Content-box width:50%, padding:10% 0 | R2 lesson: percentage applied once; padding uses parent width vertically |
| Content-box height:50%, padding:0 5% with definite parent height | Opposite-axis percentage padding does not justify rejecting or reapplying height percentage |
| The prior two controls with same-axis nonzero padding added | Existing unresolved mixed-size diagnostic retained |
| Nested padded parent whose content width differs from viewport and border width | Padding uses the original parent's content width, not viewport or parent outer width |
| Same width, changing parent height; then same height, changing parent width | Vertical padding responds to width; legitimate cross stretch responds to the cross dimension |
| Zero percentage, tiny source percentage rounding to zero, inherited percentage padding, inherited em padding | No source-unit or computed-style laundering; keep correct per-side provenance |
| Odd dimensions and one fractional percentage/asymmetric control, including translucent image/background | Rounding, content sampling, clipping and outer/tail pixels; no tolerance widening |
| Unknown containing-width and self-dependent inner 100% contexts | Precise diagnostic, no fabricated finite bound |
| The large chain/ratio/padding control above, plus a comfortably smaller finite counterpart | Final automatic-outer addition guard; compile-only, no extreme native render |

The existing point-padding and zero-padding byte/map regressions remain required. New public outcomes require deterministic Rust/CLI/WASM/JS agreement and recovery, baseline import, retained native geometry/pixels, exact-file resize/clone evidence, and visual review. This source audit supplies no replacement for those gates and does not silently convert existing fractional paint failures into passes.

## Reviewed identities

Paths are relative to `tools/html-to-riv` except the three repository paths labeled below. These hashes bind the inspected source, not a final compiler freeze.

| File | SHA-256 |
| --- | --- |
| `src/images.rs` | `d90efbc8d011562218bc60b5282dbbfd1b937fa0979510d2d793445c6bcb221c` |
| `src/compiler.rs` | `2c53f93ce31f34043f782780c17fcbb1477e9f2851124716555b78a6a55d9146` |
| `src/box_sizing.rs` | `3ebdbbee9f1821558ccad324cea593f2387857039f7a6ed692ff97cebf82754b` |
| `src/numeric.rs` | `d093c82e0f59e9978b0d073ef335ef8346893b9ddf7d4a2a257a3052cb510a8e` |
| `src/padding.rs` | `4429150da94bfdd59c412f2a983f47ace556a5434e57b00860cf6174db2c30ec` |
| `src/flex_descriptor.rs` | `65b06dd62b620f4e900949dda76d378cf4632d69c1d5dde6faded0bc3c7ab919` |
| repository `vendor/taffy-0.12.1-rive-yoga-order/src/compute/flexbox.rs` | `75f7a79993712f3f115762291819bf3aab530e79002746c63ad9ad500cc728c2` |
| repository `crates/nuxie-runtime/src/mechanical_port/source/layout/layout_participant.rs` | `fceb9920fa6442d4ff9fc9c31086f9fcdd2def7ae0d445db69d5c899b2925c46` |
| repository `crates/nuxie-runtime/src/mechanical_port/source/layout/layout_component_style.rs` | `46b8b93aa0ee7cd97063716a64730e8174410de8d1fb44f1f8ad96310f5f1335` |
