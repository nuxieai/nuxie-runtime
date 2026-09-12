Read-only audit of the initial ordinary content-owner composition, against
compiler commit `87aadf32336b9cb582b83c3f13e57824c907ea5f` and immutable runtime
baseline `6c7ac166`. The inspected runtime layout files and vendored Taffy tree
have no diff from that baseline. This document is the only file changed by this
audit. The initial design notes below precede the private experiments; the
empirical update at the end supersedes their pending status for the identified
cases. This audit has read and checked the recorded r1–r3 evidence, but has not
itself imported, rendered, or directly visually reviewed a candidate scene.

The proposed outer component retains the authored source identity, translated
border-box dimensions/bounds, padding, paint, and participation in its parent.
It packs one inner component in normal column flow. The inner component has no
paint or padding, retains authored point/auto dimensions and bounds, and owns
the original direction, distribution, and authored children. Owner percentage
dimensions/bounds are outside this initial candidate.

**The fixed-size case has a concrete arithmetic benefit. The automatic-size
case needs contextual qualification, and the current numeric guard cannot be
reused for the new descendant owner.** These are separate issues: fixing the
guard is necessary before public admission even if visible candidate cases pass.

Source facts established by inspection:

| Source | Relevant behavior |
| --- | --- |
| `src/box_sizing.rs::lower` and `translated` | Authored computed style is retained separately from translated outer sizes/bounds. Point padding is accumulated in binary32, then added to each definite content dimension and bound. |
| `layout/layout_style_applier.rs::YGStyle::default`, `taffy_style`, and `dimension` in the runtime | Ordinary styles retain Taffy's border-box default; point dimensions become point lengths. Dimension percentages preserve the authored coefficient through `rive_yoga_percent`; padding percentages use coefficient/100. |
| `vendor/taffy-0.12.1-rive-yoga-order/src/util/resolve.rs:39` | Tagged dimensions resolve as binary32 `coefficient * owner * 0.01`, with the multiplication order retained. |
| Taffy `compute/flexbox.rs:175`, `:448`, `:474`, `:533` | Padding resolves against parent content width; this includes vertical padding. A container's descendant sizing basis is its known outer size minus its content-box inset. Children resolve dimensions/bounds against that basis. |
| Taffy `compute/flexbox.rs:199`, `:210`, `:1175`, `:1887` | Min/max clamping and padding floors participate in outer sizing. A supplied known dimension can take precedence over the locally styled dimension. |
| `src/compiler.rs::Emitter::layout_box` | Fixed points stay Fixed. Automatic cross dimensions can become Fill/stretch; automatic main dimensions become Hug/nonflexing. Those choices depend on the actual parent direction. |
| Runtime `layout_component.rs:3334–3373`; Taffy `compute/flexbox.rs:1614` | Main Fill sets linked grow/shrink factors; cross Fill selects stretch. Stretch affects automatic cross sizes, not fixed cross sizes. |
| Runtime `layout_component.rs:2174` | Automatic minima on native flex children are adapted to zero. A new helper does not introduce browser min-content minima automatically. |
| `src/numeric.rs::Bounds::child_with_sizing` | Current descendant bounds describe the translated outer box after subtracting its padding. They do not describe an independent fixed inner component. |
| `src/flex_descriptor.rs::Group::extract`, `src/flex_sizes.rs::propagate` | Final native parent IDs, participant order, helpers, padding, and native fields are part of the proof premises. Authored styles alone are insufficient. |

The following arithmetic was checked with explicit binary32 rounding after each
operation using Python's `struct.pack('f')`/`unpack('f')`. It is an arithmetic
check, not a native wrapper observation. Let `f` mean rounding to binary32,
`P = f(left + right)`, `O = f(content + P)`, and `R = f(O - P)`.

| Authored content width | Left/right padding | Outer O | Recovered outer content R | 100% of independent inner content |
| ---: | ---: | ---: | ---: | ---: |
| 999999.875 | 1000000 / 1000000 | 3000000 | 1000000 | 999999.8125 |
| .03125 | 1000000 / .03125 | 1000000 | 0 | .03125 |
| 1000000 | .03125 / .03125 | 1000000.0625 | 1000000 | 1000000 |

For the first row, an inner Fixed width of 999999.875 is independently
representable. Its 100% child is predicted to resolve to 999999.8125, matching
the previously recorded Chrome local ResizeObserver width. The existing
single-box output resolves that child to 1000000. Three successive native
100% resolutions from the independent inner width all evaluate to
999999.8125. This makes both one-child and percentage-grandchild controls
useful. It does not establish the Chrome result for a newly added grandchild.
The previous Chrome evidence is preserved in `content-box-rounding-review.md`.

For nonnegative point preferred/minimum/maximum values and fixed point padding,
rounding the translated values is monotone. Consequently a locally clamped
point content value and its translated outer clamp are consistent in the
restricted case where neither component's target is overridden by flex or
stretch: clamp the content, then add padding, or clamp the corresponding
translated outer fields. A larger minimum wins over a smaller maximum. This
algebra does not prove automatic-size behavior or the full native measurement
schedule. Minimum, maximum, and preferred values must all be retained on the
inner component; restoring only the preferred width misses constrained cases.

Concrete candidate checks, with predictions distinguished from observations:

| Case | Prediction or required observation |
| --- | --- |
| Fixed owner width 999999.875, height 20, horizontal padding 1m/1m, child width 100% | Predict outer width 3m, inner width 999999.875, child local width 999999.8125. Repeat with child Fixed and auto width; the expected inner content base remains independent of outer cancellation. |
| Fixed width 300, min-width 100, max-width 200, padding 10 | Predict inner width 200 and outer width 220. With min-width 250/max-width 200, predict inner 250/outer 270. Verify percentage descendants use 200 or 250, respectively. |
| Fixed width with auto height in a column parent, two fixed child heights 20 and 30, padding 10 | Predict intrinsic inner height 50 and outer height 70, provided helper measurements do not supply another known target. This is a bounded candidate prediction, not admission for every auto axis. |
| Auto height stretched in a row parent | The normal-column carrier changes the inner auto height from a cross-axis stretch target to a main-axis Hug target. See the explicit counterexample below. |
| Auto width with start alignment/intrinsic width | Inspect both the outer contribution and the inner width. The inserted normal-column owner makes width its cross axis; Taffy's flex-base measurement can supply available cross space to an auto stretched child (`flexbox.rs:675–711`). Intrinsic sizing must not become available-width sizing accidentally. No numeric outcome is asserted without a native run. |
| Point dimensions with all four directions, distribution, and order | Original direction/distribution and all spacing helpers must belong to the inner box. The outer must physically anchor that box at the padding start. Keeping original reverse/distribution alignment on the outer can place the inner in residual rounded free space. |
| Percentage descendants and descendant percentage padding | Their immediate native containing block must be the inner content component. All four percentage padding sides must use inner content width, including top/bottom. Retain authored padding inheritance separately. |
| Min/max-driven auto dimensions and overflow | Check min greater than preferred, max less than intrinsic content, min greater than max, and minimum smaller than one padding ULP. Inner and outer targets must be inspected separately; merely matching the outer paint rectangle is insufficient. |

The automatic-height counterexample is finite and uses no owner percentage:

```html
<div id="g"><div id="p"><div id="a"></div><div id="b"></div></div></div>
```

```css
#g { width: 300px; height: 120px; flex-direction: row; }
#p { box-sizing: content-box; width: 100px; height: auto;
     padding: 10px; justify-content: space-evenly; }
#a, #b { width: 20px; height: 20px; }
```

Under the compiler's existing reset, `p` stretches across its row parent's
120px height, giving it a 100px content height. The expected browser child
positions relative to `p` are y=30 and y=70: 20px of distribution space before,
between, and after the two children, plus the 10px top padding. The proposed
normal-column inner uses main-axis Hug for `height:auto`, so its predicted
height is only 40px; distribution has no remaining space and child positions
become y=10 and y=30. These are source-derived predictions awaiting the separate
native/Chrome controls, not observations from this audit. The outer border box
can still match exactly while descendants are wrong. Point min-height and
max-height variants should test whether a clamp masks or exposes the mismatch.

An outer carrier whose axis follows the actual parent axis is a possible
follow-up experiment: it could keep a stretched automatic dimension cross-axis
for the inner participant. That is not a proof for all auto sizing, wrapper
alignment, or intrinsic measurement contexts. Alternatively, the first public
profile can diagnose the unresolved stretched-auto cases. A failed fixed-axis
wrapper does not establish impossibility for ordinary-file compositions.

**Concrete unsafe guard counterexample.** Use an owner with
`box-sizing:content-box;width:.03125px;padding:0 .03125px 0 1000000px`, then a
chain of descendants each with `width:1000000%` and no finite maximum. The
current outer-based guard computes width 1000000 minus native padding sum
1000000, yielding a zero content bound. A real independent Fixed inner width
.03125 survives even though it overflows that zero remaining outer content.
The guard would incorrectly propagate zero through every descendant if reused
unchanged. The actual native percentage recurrence grows by roughly 10000 at
each level; its tenth percentage multiplication overflows binary32. All source
coefficients are within the unchanged 1000000 source limit, and the DOM depth
is well below 128. Do not render the overflowing case: require a compile-time
diagnostic. Finite shorter chains can test the actual inner ownership.

Before public integration, derive descendant bounds from the exact emitted
inner sizes/bounds, separately from the outer participation bounds. For fixed
point inner targets this must retain the authored scalar and local clamps,
without subtracting the outer padding again. For auto targets it must model the
actual helper direction/stretch context or remain unresolved. Preserve
monotone outward rounding, lower overflow witnesses, min/max ordering, and the
complete viewport domain. The old guard's conservative overestimate in the
999999.875 example is not evidence that it remains conservative in general.

Authored `Style` must remain the inheritance source. Feeding an artificial
padding-zero or translated-size style to DOM descendants would break
`padding:inherit`, `width:inherit`, and relative-font provenance. Emission and
numeric analysis need separate outer/content views, while selector order,
computed inheritance, source IDs, source paths, and the authored-element count
still follow the original DOM. Helpers must not consume authored depth or
appear as extra source-map elements. Their extra native topology still needs
the existing depth-128 native/WASM and import/clone checks.

Descriptor capture must not silently treat the authored outer ID as the direct
parent of children now attached to the inner ID. Nor may it attach the outer
translated numeric width to the inner native fields. The outer group now has
a helper participant; the inner group has different padding and parent facts.
Keep the existing direct-participant certificate unresolved for this
composition until every native ownership and scalar binding is represented.
The reported private candidate's assertion that descriptor capture is disabled
is appropriate for an experiment. Existing padding/baseline and alignment
wrapper exclusions must remain: `baseline.rs` explicitly declines padded
metrics, and adding an inner box does not supply the missing ascent/origin proof.

World projection is another independent check. Runtime
`layout_component.rs::compose_world_transform` (line 989) multiplies
`parent_world * layout_translation * own_transform`. Inserting an inner
component changes how translations are grouped. Local width agreement cannot
prove projected rectangle or pixel agreement at large coordinates. Preserve the
original left-1m case and right-only/left-only/half-split padding controls;
record local widths, world transforms, projected bounds, and painted output
independently. A useful small arithmetic stress case combines an ancestor
translation of 1m with two .03125 offsets: grouping the small offsets before or
after adding 1m can change the result by .0625. This is an additional
falsifiable projection hypothesis, not an observed wrapper failure.

No owner percentage dimension or bound should be enabled by this initial
composition. Copying such a percentage to the inner component changes its base
from the authored parent's content size to the outer's remaining content size.
For example, with a 400px containing width, content width 50% and 10px horizontal
padding require content 200 and outer 220; simply putting 50% on both levels
cannot preserve that dependency. Percentage padding with fixed content is a
different possible composition problem, not ruled out by this observation.
This audit neither establishes a universal impossibility result nor qualifies
any responsive extension beyond the candidate's declared point/auto scope.

The original pending work belonged to the separately recorded experiments: measure
the fixed-size benefit; test the finite auto/stretch counterexample; test point
clamps and shorter percentage chains; compare local and world geometry; inspect
paint across original/clone resize sequences; and implement actual-inner bounds
before any public compiler admission. No runtime or renderer change is needed
to perform those experiments.

**Empirical update: private r1, r2, and r3.** The normal-column r1 candidate
passes all 224 geometry observations in its 28-scene core matrix, while six
pixel observations fail in `content-point-fractional-paint-control`. Its
separate six-scene stretch matrix has 48 observations: 16 geometry and 16 pixel
failures, all in the two row/row-reverse `space-evenly` automatic-height cases.
This confirms the earlier stretch/distribution concern in bounded native and
Chrome measurements, rather than leaving it as a source-only hypothesis.

The parent-axis r2 candidate passes all **272 geometry observations** across
the combined 34 scenes. It retains exactly the same six fractional-paint
failures, at frames 0, 2, 3, 4, 6, and 7 of
`content-point-fractional-paint-control`. Its recorded status remains
`failed-public-baseline`; geometry agreement is not a full paint pass. These
results support the adaptive packing repair for these cases, including both
row directions, both column directions, fixed clamps, inheritance, nested
percentages, and the exercised overlapping paint. They do not establish the
whole point/auto domain or additional world-coordinate combinations.

The r3 candidate changes the private numeric guard, not emitted layout records.
Direct comparison of its preflight requests and artifacts with r2 confirms
**34 identical request/Rive/map triples**, and the five responsive proposals
retain identical diagnostics. There is no separate r3 native render in this
audit. The 20 compile-only cancellation controls now produce these outcomes:

| Compiler/graph | Accepted | Rejected |
| --- | ---: | ---: |
| Frozen public compiler, original single owner | 18 | 2 |
| Private r2, inner owner with old guard | 18 | 2 |
| Private r3, inner owner with two-stage guard | 12 | 8 |

The six newly rejected requests are width and height versions of cancelled
point, minimum, and maximum content targets, each followed by ten amplified
percentage descendants. Both no-padding overflow controls were already
rejected. The twelve r3 accepted controls retain exact r2 bytes/maps. Rejected
requests leave no Rive or map output. No scene from this control experiment,
including its finite controls, was imported or rendered. In particular, public
acceptance is not evidence of the same unsafe native graph: the original
single-owner graph actually cancels the relevant content size to zero. The
unsafe new graph was r2's nonzero inner owner combined with the old zero bound.

The r3 build's 31 copied source files, compiler, patch, and harness were checked
against its build receipt. The control receipt, cases, compiler identities,
requests, logs, output hashes, and absent rejected outputs were independently
rechecked (236 per-case/compiler/artifact hash comparisons). Relevant bindings:

| Recorded evidence | SHA-256 |
| --- | --- |
| `output/content-owner-candidate-native-r1/receipt.json` | `b81e16e29df99487f716861cb8e54d5b77259148d05b14e590087a892c8ec7b5` |
| `output/content-owner-stretch-native-r1/receipt.json` | `f3b0da55bea658f9074096be64b440ef8f4806799d16eedf338640571c6576ea` |
| `output/content-owner-candidate-native-r2/receipt.json` | `8ab2aa5dd2b67ecf3095e04b399c65d6d801bc1f795579bb589692da409d7c73` |
| `output/content-owner-candidate-build-r3/build-receipt.json` | `c1e737b72c07ef69aaa9aeecc0239b962056f9e08fd03811595d378600740f4e` |
| r3 `compiler.patch` | `7d4b75628ca50f8c380c92f1b0f487e9527d77a229a8fa7f411473e3483ee893` |
| r3 compiler | `b4c46ff26a0a46af3b06bcffb3a337b88358ee116fd0ea922ce81f821af4e7b8` |
| `output/content-owner-candidate-preflight-r3/receipt.json` | `e602873a6a5148c9b802fbd7e2ed6aa2692845dced99e843de2f945eb52096bc` |
| `output/content-owner-bounds-r1/receipt.json` | `0b7bb9dc9cc8321d5ffb7c64972cee64280ecade48de83920d74efa12611cf4b` |

**Assessment of r3's two-stage bounds.** Within its current guarded point/auto
scope, the two stages correspond to the two emitted layout owners. Stage one
uses translated outer dimensions/bounds and authored padding, producing a
bound for the space available inside the outer box. Stage two uses authored
point/auto dimensions/bounds, no padding, zero flex factors, no margins, and
automatic self alignment. Those properties match the emitted inner component:
points are Fixed, auto main size is Hug, and auto cross size is Fill/stretch.
Point targets therefore recover their own clamped value independently of
outer cancellation; auto cross targets use the outer available content bound,
while intrinsic auto main sizes remain unknown. The second stage does not
subtract the padding again. Descendant gap checking receives the inner bounds,
matching the gap fields moved to the inner component.

One equality is currently conditional on an existing exclusion: the guard
chooses its packing parent from `parent_style.direction`, whereas the emitter
uses the later `native_parent_direction`. They coincide for admitted content
owners because content-owner selection requires nonzero native padding and the
existing padded alignment-wrapper guard rejects every case that would change
`native_parent_direction`. No new unsafe bound counterexample was found within
that guarded scope. If padded alignment wrappers are later admitted, compute
the bounds from the final emitted packing direction; the earlier authored-axis
choice must not silently survive that change.

Remaining public-integration requirements are specific:

- Keep one concrete outer/content emission description shared with numeric
  analysis. The experiment separately constructs `inner_sizing` and later
  repeats the arrays in `layout_box`; they match now, but public edits must not
  change only one copy. Preserve the cancelled point/minimum/maximum controls
  on both axes, plus auto cross bounds and automatic minima with overflow
  witnesses. The exponent guard still does not prove intrinsic measurements,
  aggregate sibling/position arithmetic, or world-coordinate error.
- Preserve authored inheritance while separating generated provenance. The r3
  temporary clone clears `inner.padding`, `inner.flex`, and margins but keeps
  the cloned `NumericStyle`, including authored padding/factor provenance.
  That is harmless to this guard because `Bounds` does not read numeric
  provenance. It is not an accurate descriptor for the generated zero-padding,
  zero-factor inner box. A future descriptor must bind its synthetic constants
  explicitly, retain authored point/auto sizing provenance, and keep the outer
  translated provenance separate. Never feed the temporary clone to DOM
  inheritance or treat unavailable/underflowed ideals as exact zero.
- Keep descriptor capture explicitly unavailable for this graph until native
  parent IDs and both owners are represented. The private assertion prevents a
  false certificate. Removing it and trusting incidental padding/helper
  rejections is not a substitute for a defined unresolved composition result.
- Requalify native/WASM depth 128, authored-element limit 8192, and large
  inherited variable environments after integration. `style.clone()` adds a
  temporary copy of the native custom-property strings (their original-source
  strings use `Arc`); it drops before recursion, but adds allocation/copy work
  for each selected owner. Helpers also enlarge the native layout topology.
  Do not count them as authored elements or reduce existing input limits to
  avoid testing the new graph. The private rustc harness does not establish
  public WASM stack behavior or CLI/WASM/Node parity.
- Record public changed bytes/maps explicitly. Adding a helper changes object
  IDs and ordinary parent links, while authored source identities and DOM paths
  must remain stable. The 34 r2/r3 matches establish guard-only stability; they
  do not establish a complete comparison with the public reference corpus.
- Retain the six fractional-paint failures and earlier failing observations.
  Any eventual support statement must distinguish the established geometry
  result from paint qualification, using the unchanged runtime, renderer,
  browser oracle, and gates. No source coefficient limit or expected image was
  changed by this audit.

The private r3 repair resolves the demonstrated cancelled-bound defect for its
new graph and preserves the measured r2 files. It remains experimental until
the public emission/guard interface, resource checks, transport checks,
descriptor behavior, and retained paint limitations are handled explicitly.
