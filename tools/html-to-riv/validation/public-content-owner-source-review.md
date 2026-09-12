Independent bounded review of the proposed public content-owner integration.
The initial notes below reviewed the APIs before implementation. The final-code
review at the end supersedes their pending status and clarifies the descriptor
scope. Only this review document is owned by the reviewer; no production code
was edited.

The proposed `Copy` numeric layout view is a suitable boundary for the existing
exponent guard: sizes, bounds, padding, native legacy-flex behavior, eligible
cross stretch, and the actual parent direction are the inputs the current
`Bounds::child_with_sizing` evaluator consumes. It avoids cloning the authored
custom-property environment merely to analyze a generated component. The
common evaluator should preserve the existing upper/lower/witness arithmetic
and retain unknown intrinsic measurements rather than treating them as zero.

The view's cross-stretch flag must combine
`style.self_alignment.stretches()` **and** absence of cross-axis automatic
margins. The current evaluator checks both. If margins are omitted from the new
view without combining that condition at construction, `auto` width/height
with cross auto margins can incorrectly acquire a parent-size bound. The
synthetic inner view intentionally has no auto margins, is eligible for cross
stretch, has legacy flex behavior, and has zero padding. Whether stretch is
actually used still depends on an automatic dimension and the actual packing
axis; fixed point dimensions must not be replaced by the outer available size.

Two evaluations remain necessary. The outer uses translated dimensions/bounds
and actual padding. Its output bounds describe available space inside that
outer component. The inner uses the authored point/auto dimensions/bounds and
zero padding against that available space. Point inner targets can therefore
remain positive when outer subtraction cancels to zero. Both the emitter and
this second evaluation must consume the same content-owner plan and finalized
packing direction. The current padded alignment-wrapper exclusion should
remain; removing it would invalidate the earlier assumption that authored and
native parent axes coincide.

The existing diagnostic descriptor API has separate `Parent` and `Item` facts.
`Parent::extract_lowered` currently takes outer translated sizes and the
authored container direction/padding/distribution. That combination cannot be
reused unchanged for both new groups:

| Group | Required facts |
| --- | --- |
| Outer authored component → inner helper | Actual outer ID; packing direction and physical-start alignment; translated outer dimensions and provenance; authored padding; no transferred gap/distribution. The generated inner participant must be visible as a helper/composition fact. |
| Inner helper → authored children | Actual inner ID; original container direction/distribution; authored point/auto content dimensions and their original provenance; zero native padding, with the original gap state; actual authored-child native parent links. |

The authored outer ID and source-map path remain the element's identity. The
inner is a generated component, not another authored element. Any group for the
inner can retain the owning element's source path as context, but must not
pretend that its parent ID is the authored outer ID or that the old outer
numeric width describes its own field. Sibling item descriptors continue to
bind the actual outer participant's translated fields.

There is a concrete certificate hazard in the current APIs:
`flex_sizes::propagate` checks the scene certificate, local defaults,
constraints, helpers, wrappers, and native topology. It does **not** inspect
`Group.structural_issues` or `Group.numerical_admission`. Adding only an
"unresolved content-owner composition" message is therefore not an effective
barrier to propagation. Public integration needs an explicit composition state
that every relevant propagation/certification consumer treats as unresolved,
or an intentional typed unresolved result. Incidental rejection from padding
or a missing inner-parent domain should not carry this responsibility.

No native zero field should inherit incompatible authored provenance. The
private r3 guard-only clone cleared padding/flex fields but kept the authored
`NumericStyle`; that was harmless because the exponent guard ignores it.
For public descriptors, synthetic zero padding/factors require generated exact
constants, or no exposed factor/padding carrier when that fact is not part of
the interface. Content width/height and bounds must retain authored decimal,
font, inheritance, and unavailable-original provenance. Do not rebuild their
ideals from emitted binary32 values. Padding inheritance still comes from the
unchanged authored style used for DOM recursion.

Production review will check these concrete cases and invariants:

- The cancelled point/minimum/maximum targets on both axes keep a positive
  inner bound; ten amplified percentage descendants diagnose before output.
  The zero controls stay zero and finite shorter chains retain the private r3
  files. This is the existing 20-control corpus, not a new native stress run.
- Cross-auto-margin views retain unknown intrinsic dimensions, and stretched
  auto dimensions use the actual packing axis. Padded alignment wrappers remain
  excluded until their full native topology is modeled.
- Exact inner point provenance and actual parent IDs appear in descriptors;
  outer translated provenance remains attached to the outer group. Generated
  composition groups explicitly remain unresolved even if other local defaults
  happen to look eligible.
- Normal scopes retain existing descriptor behavior, and DOM/source-map identity
  and authored inheritance do not acquire synthetic helper state.
- The final implementation preserves the nonrecursive preparation boundary;
  public native/WASM depth, element, variable-resource, and transport checks are
  still required. This source review alone cannot establish those outcomes.

The measured private evidence and its remaining six fractional-paint failures
are recorded in `content-owner-source-audit.md`. This API review neither
upgrades that paint status nor expands owner percentage support.

The implementation plan now explicitly addresses the two identified API
hazards: cross stretch includes the cross-auto-margin exclusion, and a
`content_owner` field on Parent/Group will be checked in `flex_sizes`,
`flex_world`, and `flex_proof`. Those are all current Group-consuming proof
paths found by source inspection. The planned inner Parent reads only authored
size/provenance fields and actual child flow, with native padding/gap facts;
it does not clone the authored factor/padding metadata. A focused regression
should mark an otherwise eligible direct group as a content-owner composition
and supply valid parent domains, then require an explicit unresolved result.
That checks the new barrier independently of incidental rejection by the
scene's existing padding or topology guards. Production implementation and
that regression remain to be reviewed.

Final-code review, 2026-09-12: no actionable numerical, inheritance, or false
certificate defect was found in the seven-file implementation diff against
`87aadf32336b9cb582b83c3f13e57824c907ea5f`. This is a bounded source review,
not a replacement for the frozen public-output, WASM, native-layout, and paint
receipts. The exact reviewed source contents were:

| File under `src/` | SHA-256 |
| --- | --- |
| `compiler.rs` | `e5a55f2bd426ca8c51f8b75f35ef0bd053ce3c0b0066dcf8f68c760fdd5f5b4a` |
| `box_sizing.rs` | `93293ea8888775066b338498f7917dc756dcdf257013140fab6412466fbb05cd` |
| `numeric.rs` | `67fefd66435534bc2e1d448c5fe688cf8bff2c948410ae484f0c0eb0380f82fa` |
| `flex_descriptor.rs` | `a282700a7d024830d11df30e667fcb20e12eab08fd1c44aa9dca0cc56a5c4281` |
| `flex_sizes.rs` | `c32fc61b7d868e9fec5a3589cfeadc88b6cbfe2834841df80ea3a81a66a8e9f6` |
| `flex_world.rs` | `cb18878ec8be1767f6018b03f03556f7e2f73b3e5584c010b692cb85b1bc0de3` |
| `flex_proof.rs` | `bb2c5f56770d35c607a09337517af6bbe38e31284c1827c425f96b4ea0e8f02a` |

`compiler.rs:674` evaluates the ordinary outer sizing first. Lines 744–747
construct the content-owner plan with the finalized native parent axis and
evaluate the inner against the outer's content bounds. Lines 748–769 emit
those same outer packing and inner size/bound fields. The inner receives zero
padding, legacy sizing, cross stretch, and no automatic margins. Gap emission
moves to the inner; the public padding-plus-gap restriction remains in
`prepare_children`. The padded alignment-wrapper rejection remains at line
691, so the outer numeric view cannot silently use the pre-wrapper direction
for an admitted content owner whose actual parent direction has changed.

`numeric.rs:53–63` centralizes the existing evaluator inputs without changing
its upper/lower/witness arithmetic. The ordinary view retains the
cross-auto-margin exclusion and the nonlegacy main-axis unknown bound. The
inner view takes its sizes, bounds, and parent axis directly from the emitted
plan. Thus a positive authored point target or bound is re-evaluated after
outer cancellation rather than inheriting a zero content bound. Automatic
main dimensions remain unknown; automatic cross dimensions use the available
outer content dimension only under the same stretch condition emitted by
`layout_box`. This is a definite-chain exponent guard, not a new intrinsic,
aggregate, or world-position proof.

No new clone of `Style`, inherited variables, or the full authored
`NumericStyle` was introduced. `PreparedChild` remains heap allocated, stores
the small `ContentOwner` plan, and lends that plan to DOM recursion. Recursion
continues with `&child.style`, while the native parent changes to
`child.content_id` (`compiler.rs:590`). The source map still records only the
authored outer ID. The existing outer `Lowered` numerical copy remains
separate from authored inheritance. The synthetic descriptor reads only
authored width/height `NumericSize` carriers, preserving decimal, font,
inheritance, and unresolved provenance without presenting authored padding or
flex factors as generated zero fields (`flex_descriptor.rs:16–19`).

The earlier two-row group table described facts needed to model both native
edges; it does **not** describe the final diagnostic API's enumeration scope.
Descriptors represent authored child lists. An owner with authored children
produces an explicitly marked inner-to-authored-children group, whose
`parent_id` is the actual synthetic inner ID and whose child links bind final
records. Its authored outer remains an Item of the preceding authored list,
with translated size/bound metadata. The outer-to-inner edge is indexed by
the final `SceneIndex`, but is not a separate arithmetic Group. An owner with
no authored children likewise has no inner Group. No synthetic source
identity is invented. Full native-group enumeration and a composition
certificate remain future work; this omission must not be described as a
complete topology proof.

That narrower descriptor scope does not yield a certificate in the current
consumers. The marked inner group is rejected independently by
`flex_sizes.rs:46`, `flex_world.rs:24`, and both the direct and scene entry
points in `flex_proof.rs:50,265`. The padded outer's final native local facts
also fail the existing direct-box checks. The inner Parent uses actual
authored content dimensions and child flow, reports native zero padding
subject to gap state, and records `content_owner_composition` as unresolved.
Public `compile` does not use descriptor capture or these diagnostic proofs
for admission (`compiler.rs:456–469`).

The independent focused command below passed **one test**. Its control first
proves an ordinary eligible group, then changes only the composition marker
and reuses previously valid size/world domains. It requires explicit failure
from direct group proof, size propagation, world propagation, and whole-scene
analysis. This verifies the barrier independently of incidental padding,
missing-parent, or other topology rejection:

```sh
cargo test --manifest-path tools/html-to-riv/Cargo.toml --lib content_owner_marker_blocks_each_proof_even_with_otherwise_valid_domains
```

The public tests were also inspected: 34 exact private-r3 output references,
three authored-inheritance controls at three viewports, all 20 finite/overflow
controls, five mixed-unit rejections, depth 128/129, authored objects
8192/8193, and a 256 KiB inherited custom-value context at depth 128. Their
full execution and corresponding CLI/WASM checks belong to the parent run;
inspection here does not assert those outcomes. The existing six fractional
paint mismatches remain unqualified. No new native render, public regression,
runtime mutation, or production edit was performed for this review.
