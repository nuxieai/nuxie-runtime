# Independent numeric and inheritance audit for L12

This review started from `c169469f31` in the immutable worktree. It reviews the
integration boundary, rather than qualifying the new public feature. The listed
binary32 counterexamples were evaluated with Python `struct.pack/unpack('f')`;
they are arithmetic witnesses, **not native/Chrome render receipts**. Native
measurements and public tests must supply the remaining evidence.

The runtime target remains
`6c7ac16617835b5f581784ff08a9e779bb52faf3`. This review changed no production,
runtime, renderer, schema or dependency files.

## Required data separation

At the reviewed starting commit, `compiler.rs:417–422` computes width/height and
min/max inheritance directly from the parent's `Style`. The recursive call at
line712 passes that same style to descendants. A lowering must therefore leave
this object authored: outer dimensions are a separate view consumed by emission
and numeric propagation.

Use the following independent controls for a parent with authored width100px,
horizontal padding10px and content-box sizing:

* Parent's native outer width is120px.
* Child with `width:inherit;box-sizing:content-box;padding:5px` has authored
  width100px and native outer width110px.
* Child with `width:inherit;box-sizing:border-box;padding:5px` has outer width100px
  and inner width90px.
* A further inheriting content-box descendant still inherits100px, not110px or
  120px. Changing its font size must not resolve an inherited `em` dimension or
  inherited `em` padding again.
* Explicitly inherited min/max values need the same checks. Crossing min/max
  should still let the minimum win after adding the receiving box's own inset.

The independently written border-box control must replace each element's own
width/min/max from these authored rules. Reusing output from a second invocation
of the same lowerer would not detect repeated inheritance inflation.

`box-sizing` is not implicitly inherited. The module reset sets border-box;
authored `initial`/`unset` select content-box; explicit `inherit` uses the computed
parent mode. Test these transitions alongside `width:inherit`, not only on empty
boxes with zero insets.

## Descendant bounds must use the emitted outer dimensions

At the starting commit, `numeric.rs:42–44` reads Style sizes and min/max as outer
dimensions. Lines117–121 then subtract padding to propagate content bounds. The
lowered arrays must replace all three inputs, including lower witnesses and
min/max paths. Keeping authored arrays understates the descendant base.

A concrete overflow regression is a content-box parent with width1px and
padding1px, followed by ten nested boxes with width1000000% and height1px. The
parent's correct content width is1px. The native Yoga order is
`percent * parent * 0.01f`; the tenth descendant's first multiplication overflows.
If the guard sees authored outer1px and subtracts2px padding, it propagates zero
instead and can miss the overflow. The public compile must diagnose before
returning bytes. Pair this with a finite chain and a bounded/clamped variant so
the test checks propagation, not merely rejects large percentages globally.

Do not replace lowered-minus-padding with authored content values, either.
Binary32 cancellation changes the base. For content width999999.9375px and
left/right padding1000000px, native outer rounds to3000000px, then its content
width becomes1000000px. That is0.0625px greater than the authored width. A unit
test of `Bounds::child` must enclose the native content value.

Percentage padding must retain the existing bounded containing-width requirement;
top and bottom percentages also use width. Auto intrinsic dimensions cannot gain
a definite bound solely because their translated maximum is finite.

## Native operation order and conversion error

Baseline `vendor/taffy-0.12.1-rive-yoga-order/src/compute/flexbox.rs:176–197`
resolves each inset, sums the two sides per axis, and then adds that sum to a
quantitative size when internal content-box mode is selected. The ordinary file
path keeps border-box mode, so the compiler performs that adjustment itself.
Lines448–475 compute the content inset and subtract its axis sum from the native
outer dimensions. Lines203–211 establish minimum/maximum ordering and the padding
floor.

The following fixed-point arithmetic probes distinguish common wrong lowerings:

| Authored content | Start padding | End padding | Binary32 padding sum | Binary32 outer | Exact authored outer |
|---:|---:|---:|---:|---:|---:|
|1000000|0.03125|0.03125|0.0625|1000000.0625|1000000.0625|
|0.03125|1000000|0.03125|1000000|1000000|1000000.0625|
|999999.9375|1000000|1000000|2000000|3000000|2999999.9375|
|999999.875|1000000|1000000|2000000|3000000|2999999.875|

In the first row, `(content + start) + end` yields1000000 and loses both small
insets. In the second, converting the full real sum once yields1000000.0625,
which is a different operation sequence from native padding-sum then dimension
addition. Keep actual encoded bits and original mathematical intent separately.

The last row has0.125px outer error, exceeding the current0.1px geometry gate.
This is a concrete boundary to run through pinned Chrome and the immutable
importer; finite scalar addition alone cannot establish general geometric
qualification. Do not reduce the source-value limits to hide it. Derived outer
dimensions can legitimately exceed the existing1000000 source-coefficient bound,
up to3000000 for this fixed-padding translation. A failed large-coordinate case
is evidence to preserve, not a reason to widen the geometry tolerance.

Padding and size metadata should enclose original decimals, variable originals,
font-relative products, per-side accumulation, and the final outer addition.
`ScalarProvenance::with_native` updates actual bits while preserving an ideal; it
is not a substitute for adding the padding ideal. Native percentage dimensions
and percentage padding have different operation order in
`layout_style_applier.rs:717–731`: the former retains Yoga percent multiplication,
the latter divides its coefficient by100 first.

## Descriptor certificate regression probes

At the starting commit, `flex_descriptor.rs:12`,69–94 captures authored sizes and
their metadata. Item scalar bindings at186–216 compare these with actual native
fields. Accurate lowered metadata must replace the corresponding dimension
metadata, or the adjustment must remain explicitly unresolved. Avoid describing
authored100px as native120px with the original100px error interval.

That local scalar binding is not a group qualification. Existing nonzero-padding
structure guards (`flex_descriptor.rs:250,255`, `flex_structure.rs:41,59–60`)
and size-propagation checks (`flex_sizes.rs:26–51`) must remain effective. Test a
padded content-box owner, a padded child, and padding-free descendants under a
padded owner. Correctly adjusted scalar metadata must not turn them into a
padding-free direct-box certificate.

An important edge case appeared in the incoming lowerer's initial version:
skipping an axis solely because its **native** padding sum is zero discards
nonzero original semantics. For `width:0px;box-sizing:content-box;padding:1e-50px`,
the ordinary fields are all zero, but the original ideal outer width is2e-50px.
If side metadata is ignored, native-only zero-inset guards and an exact-zero
dimension carrier can falsely prove a zero-size scene. Add a direct regression
that requires adjusted nonzero ideal bounds or an explicit unresolved result.

Underflowed `padding:1e-50%` additionally has a width-dependent ideal. It cannot
be added as a point scalar merely because the native coefficient became zero.
Literal `0%` may prove zero; nonzero or missing original percentage provenance
must remain unresolved unless the containing-width dependency is represented.
The same rule applies when a side has no original metadata. Lack of evidence
cannot become an exact compiler zero.

## Checks to attach before qualification

1. Rust tests for the inheritance tree, all lowered min/max arrays, addition order,
   native cancellation bounds, overflow chain, and underflow descriptor probes.
2. Public CLI/WASM byte parity for admitted forms, with matching diagnostic and
   no-output behavior for mixed dimensions or unsupported percentage padding.
3. Native/Chrome geometry and pixel measurements for ordinary fixed-padding
   compositions, same original/clone resizing, and the explicit large-coordinate
   counterexample above. Keep metric failures distinct from visual conclusions.
4. Source identity guard and confirmation that private descriptor tests still
   return unresolved for inset-bearing scenes.

This audit identifies integration requirements and counterexamples. It does not
claim that every content-box expression is expressible by a single scalar, that
all compositions are impossible, or that public L12 qualification is complete.

## Review of the integrated implementation

The incoming implementation was read after its first source snapshot in
`output/public-content-box-r1/source-bindings.json` and before the subsequent
WASM recursive-stack repair. The source snapshot is a locator for this review,
not a claim that the final commit is frozen to that snapshot. This read did not
rebuild or modify production code.

The identified integration issues are addressed in the reviewed code:

* `compiler.rs` computes an independent `box_sizing::Lowered` for each semantic
  box. Both `layout_box` sizes/min/max and `Bounds::child_with_sizing` receive
  this view. Descendant recursion still receives the original `Style`; lowering
  a parent descriptor again is pure and does not feed back into inheritance.
* `box_sizing.rs` adds the two actual side floats before adding a quantitative
  dimension. Its cloned metadata represents these additions using
  `ScalarProvenance::add_nonnegative`. Auto remains Auto, percentage-plus-point
  expressions with nonzero native padding diagnose, and the zero-axis path
  skips work only when the side provenance also proves exact zero.
* Underflowed point padding retains its nonzero ideal in point dimensions and
  bounds. Percentage padding with a zero native coefficient but nonzero or
  missing ideal is marked unresolved instead of being reinterpreted as pixels.
  An unaffected literal-zero percentage axis retains its coefficient metadata.
* Bounds use all lowered preferred/min/max fields, then subtract native padding
  with the existing outward bounds and lower witnesses. The new cancellation
  regression specifically encloses1000000px content after a3000000px native
  outer result; it does not substitute999999.9375px authored content.
* Descriptor extraction uses lowered native dimensions and matching numeric
  metadata. Nonzero native insets retain the existing structural exclusions.
  The padded parent/child regression confirms size propagation remains
  unresolved even though adjusted local scalars bind to the native records.

The five `box_sizing.rs` tests exercise repeated inheritance without mutating the
parent, inherited min/max, native addition order and derived sizes through3m,
per-side font/variable/inheritance provenance, underflowed point/percentage
metadata, and padded descriptor propagation. `numeric.rs` adds the cancellation
and correctly retained1px content-base test. The public rejection corpus contains
the ten-level overflow chain and requires a finite-binary32 diagnostic. The
public control corpus includes independent three-generation inherited widths,
mixed sizing modes, CSS-wide values, fixed min/max and automatic dimensions.

No further actionable arithmetic or inheritance defect was found in this bounded
read. This does not qualify the large-coordinate geometry, visual results or
WASM depth behavior: those still require the separately recorded execution
results. In particular the0.125px rounding witness above remains a retained
validation question, not a passing result inferred from these tests.

## Final r2 emission and depth review

The stack repair was reviewed separately against the original recursive
emission sequence. All32 source/manifest/binary entries in
`output/public-content-box-r2/source-bindings.json` matched the files read for
this review. The frozen compiler source SHA256 is
`d38f8578531b1ac51a44bc876ff3161b439663410b43b4b8ef5f5d2672881881`.
This was a source/receipt review; it did not duplicate the parent task's old-output
regression run or the separate native/visual work.

`prepare_children`, `start_child` and `finish_child` preserve the previous
observable ordering:

* Sibling text/tag/attribute/identity checks, DOM path assignment, CSS order
  sorting and native-order reversal still run before emitting that group.
  Compact sibling flex/baseline/padding checks precede the leading spacing
  helper. The descriptor record range still starts before that helper.
* Each child emits its leading auto-margin helper, optional alignment wrapper,
  semantic box, padding/gap fields, flex fields, paint, descriptor and source-map
  entry before descendant recursion. Lowered outer sizes still feed emission and
  Bounds, while `PreparedChild.style` retains authored computed values for
  inheritance and child-chain calculation.
* Descendants finish before the parent's first/last baseline summaries are
  calculated. The stored vertical-flex flag is the same expression previously
  evaluated after recursion; neither its flex plan nor parent direction changes
  during that recursion. Padded and gapped summaries still return unknown in the
  unchanged baseline functions. The refactor does not feed outer dimensions into
  those authored, zero-inset metric functions.
* Trailing margin/spacing helpers remain after descendant emission and before the
  next sibling. Pushing the completed scalar summary now happens just after
  these helpers; its last-edge flag uses `children.len()+1 == count`, equivalent
  to the former flag after the push. `spacing::emit` reads no summary vector, so
  that local push timing cannot change emitted records. Group baseline
  constraints and final descriptor capture still follow all child summaries and
  spacing helpers, including for empty child lists.

Depth checking still occurs before child scanning in each recursive call and
uses the same `depth > 128` boundary. Extraction moves large preparation and
emission temporaries out of retained recursive frames; it does not alter the
source limit, use a special runtime stack setting, or return partial output.
`compile_with_descriptor_capture` reaches encoding only after the entire tree
succeeds. The unchanged WASM bridge clears its RIV buffer before compile and
returns diagnostic metadata on an error.

The stored
`output/public-content-box-r1/depth-comparison/receipt.json` was inspected with
its driver, and its repaired WASM hash exactly matches the r2 frozen WASM
`559f70d7fabab80bb1b5b296d50030b44e8a1f52a23b7e74b793adf0618293d0`.
For both border-box and content-box inputs, depths64 and128 succeed with the
matching node counts through JS and raw WASM; depth130 produces `depth-limit`,
raw status1 and zero RIV bytes, with no trap. The retained prior compiler also
traps for the border-box depth128/130 fixtures, distinguishing the previously
present default-stack problem from the feature's new semantics.

No new actionable ordering, baseline-summary or error/depth defect was found in
this bounded final refactor review. The receipt demonstrates its exact tested
deep fixtures; it does not replace the broader resource/lifecycle tests or the
independent native visual qualification.
