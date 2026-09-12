> Anchored checkpoint supersedes the missing-term task: ComponentOrigin now lands the actual visible extent without feedback, with updated record/error bounds and a384frame native positive/old-emitter negative control. Next bind paint/mask premises and validate the actual source-derived candidate against Chrome. Read wrapped-anchored-review.md.

> Latest scalar/position checkpoint: sizing records/defaults/operands and final emitted-expression bounds are now connected to the same owned candidate. A new unequal-visible-size counterexample reveals the missing a*(h-v) semantic term; repair before public admission. Existing mask rectangles may preserve AA through the pinned renderer's clip intersection, so bind/test that route before changing their dimensions. See wrapped-scalar-position-review.md and its linked audits. Older proposed obligations below remain historical where superseded.

# Source-derived certificate for wrapped sizing and paint gates

This is a read-only implementation plan, not a completed proof or public admission. It builds on `wrapped-anchor-error-audit.md`, `wrapped-slot-size-invariant.md` and `wrapped-coordinate-admission.md`. No runtime changes or new native campaign are proposed here.

## Implemented prerequisites and audit corrections

`src/wrapping_sizes.rs` now resolves closed machine size intervals from authored numeric metadata and bit-matching ordinary fields. It follows native binary32 percentage multiplication order, checks intermediate overflow before clamps, retains original parent bases and implements minimum-wins endpoint clamping. These are machine enclosures, not ideal CSS error envelopes. Explicit minima are required; automatic minima remain unresolved.

`src/wrapping_slots.rs` binds a closed, completed base scene before augmentation: one root-origin parent, independent definite slots, one painted visible child per slot, unique styles and no unrecognized records or native overrides. The token borrows the inspected records. This is **not yet a binding of the final helper graph**; generated landmarks/constraints/masks and preservation of the base scene still need inspection after composition.

The newer `wrapped-gap-normalizer-audit.md` refines the proposed equations below. Exclude line stretch as well as item stretch; validate actual accumulator magnitude intervals before choosing R; check the rounded normalizer length; and propagate reconstruction error through carry maxima. In particular a local-space max constraint can exceed its selected operand by one ulp, so the old measured-extent bound alone does not establish reset safety. The conditional different-line operation count31+4N is sufficient only with the shared-machine-offset and validated-envelope premises in that audit. None of these prerequisites admits public wrapping yet.

## Domain and composition integration

`wrapping_domains.rs` now joins authored NumericStyle dimensions to the checked base: parent axes resolve against the explicit viewport domain, slot axes against those parent axes. It rejects missing/duplicate metadata, stale units/values, and an absent positive cross bound without introducing a viewport floor. Its fields are private and exposed read-only.

`wrapping_composition.rs` consumes those domains, charges exact existing sizing/paint expansion before emission, and owns the generated candidate records. An independent prefix/role check preserves base layout inputs, allows only the identified original colors to be hidden, excludes layout participants, and validates typed helper parents/references. `wrapped-composition-layout-isolation.md` records the native source chain for layout isolation. This closes base-role preservation for this closed composition path; it does not validate every scalar constraint parameter or prove numerical or pixel correctness.

`wrapping_normalizer.rs` implements the conditional positive-branch normalizer proof, including normal intermediate ranges, rounded length threshold, correlated projection, subtraction/FMA error and exact saturation conditions. It is not yet connected to a source-derived dead interval or final scalar graph binding. The caller must prove those remaining inputs. No public route is enabled.

Next implement the actual coordinate accumulator envelope and carry reconstruction bounds, derive epsilon and the positive dead interval, bind the normalizer's zero-target/identity/axis/distance/mode/strength/clamp premises to the candidate, and establish mask margins. Do not repeat the completed domain/base-composition implementation merely because older plan paragraphs below describe it as future work.

## Coordinate/carry implementation and visible-source correction

`wrapping_coordinates.rs` now encloses actual native main/cross sums, positional first offsets, accumulator updates and slot locations using binary32 endpoint recurrences. It checks both coordinates of slot landmarks and visible initial world transforms, derives R and31/31+4N conditional error budgets, rounds epsilon upward, derives positive dead endpoints with native subtraction, calls the normalizer proof, and passes measured-height enclosures/errors into `wrapping_carry.rs`.

`wrapping_carry.rs` propagates measurement and max-reconstruction error through both passes and final maxima, checking every entering reset against65536. `compose_with_bounds` uses the resulting source-derived epsilon for the exact owned candidate; the old explicit-epsilon constructor remains a private experiment seam.

The coordinate audit found that finite stored visible dimensions were insufficient: percentage resolution and initial visible world translations could overflow. The binder now retains visible sizes and placement fractions, and Domains resolves both visible axes from final ordinary records against each slot's axes. Native-only bounds do not invent authored provenance or certify visible CSS semantics. Explicit minima are still required by this helper. The immutable all-flex native path does adapt automatic minima to zero (`layout_component.rs:2173–2184`); that is a usable future premise, not an intrinsic runtime impossibility.

These implementations close the stated source-envelope and carry prerequisites within the checked base shape. They still require final scalar-field/dependency evaluation bindings, final offset/target error and mask/raster coverage before public routing. Native evidence with the derived epsilon has not yet been captured. Do not rerun the earlier experimental epsilon campaign as though it qualifies this candidate.

## Smallest general route

Start with a root-origin, translation-only wrapping scope whose parent has definite bounded dimensions, zero insets/gap, and independent slots with two definite dimensions. Require every slot's native cross size to have a strictly positive source-derived lower bound. Permit variable item count subject to computed bounds and actual graph cost; do not specialize the certificate to the three historical fixture identities. Explicit positional line fractions0,1/2,1 and both wrap directions are possible once each passes the same inequalities. Initially reject intrinsic/stretch/baseline slots, relative insets, slot aspect ratios, native size-modifying constraints, nonzero origins and unknown ancestry.

The easiest first implementation admits authored finite point cross sizes and homogeneous point min/max, together with responsive main dimensions. Cross percentages can follow using the same interval interface when their minima establish a positive machine lower bound. Existing percentage-only historical fixtures cannot all enter this route: their cross-size lower bound is zero over the full open viewport domain. Preserve that limitation rather than increasing the minimum viewport implicitly.

The slot-size invariant establishes that the published final size equals the target size used in cross alignment for this zero-inset definite-slot shape. Without that fact, an anchor reconstructed from final size need not identify the same line anchor. Bind it to final ordinary records and their native size/units, not only computed Style.

## Available implementation pieces and their actual limits

- `ScalarProvenance` retains authored ideal intervals, native binary32 value and error, including source coefficient selection. Its nonnegative add/multiply/min/max operations suit source size arithmetic; they do not prove runtime anchor equality.
- `flex_numeric::ErrorEnvelope` exposes ideal lower/upper and error, checked add/subtract and `preferred_percent`. The percentage helper correctly models `fl(fl(coefficient * owner) * 0.01f32)` and rejects intermediate overflow. Enclose native values with outward-rounded `[lower-error, upper+error]`. `mul`, `div` and clamp remain private implementation helpers; there is no public sqrt/normalizer correlation proof.
- `numeric::Bounds` tracks finite upper/lower/witness dimensions but not origin arithmetic errors or record identities. Its exponent checks alone cannot certify gates.
- `flex_sizes::propagate` currently rejects nonzero min/max, flexible targets, unknown parents, helpers/wrappers and content owners. It is not already a wrapped-slot bound pass. `flex_world::propagate` proves direct single-line start-aligned groups, not wrapping line offsets. `flex_proof` is explicitly a direct-leaf conditional analyzer.
- `flex_scene` and descriptors provide a useful final-record/ancestry binding pattern. A wrapping certificate needs explicit roles for independent slots, visible moved owners, generated landmarks and masks; accepting arbitrary user constraints into the existing direct-box certificate would be unsound.

Add small internal helpers for native size-interval resolution and bound clamping, finalized slot-shape binding, wrapping local-coordinate bounds, and gate/mask inequalities. Reuse arithmetic primitives rather than treating the current direct-leaf certificate as a wrapping proof.

## Concrete quantities and inequalities

For each slot resolve the preferred dimension and min/max against the original fixed parent content basis. With nonnegative intervals, native used size is monotone:

`h = max(minimum, min(preferred, maximum))`, with absent maximum unbounded.

Clamp lower endpoints together and upper endpoints together, with native-operation outward rounding. Let `l_i` and `h_i` bound that machine size; `L = min(l_i)` and `H = max(h_i)`. Reject unknown, nonfinite or nonpositive L for this first route. A maximum smaller than a positive minimum does not erase the lower bound because minimum wins.

Any nonempty contiguous line has extent at least L and at most H. For adjacent visual lines with common fraction f, the ideal anchor gap is `(1-f)*linePrevious + f*lineNext`, so at least L. Nonadjacent paint-anchor pairs cannot have a smaller ideal gap. No enumeration of browser line partitions is needed for this sufficient route. A later partition-aware lower bound could admit center alignment when not every item has positive l_i; it is a separate optimization.

Let N be slot count, C the parent cross upper bound, and P bound its absolute machine world translation. A deliberately loose candidate envelope is:

`B = P + 2*C + (2*N + 4)*H + 1`

`R = 4*(B + H + 1)`.

Evaluate every sum/product outward. The local envelope must be justified against the chosen native direction/positional-alignment path: at most N line extents, first-line positional offset bounded by max(C,NH), local item offsets bounded by H, and reconstructed anchors bounded by another H. For initial root-origin scopes P=0. Do not substitute a lexical depth limit for P. If allowing ancestors later, derive each actual world interval independently.

The existing same-line operation audit supplies `E_same = upward_f32(31*(2^-24*R + 2^-149))`, conditional on its exact landmark/subtraction graph and final-size invariant. Use this as emitted epsilon after validating the graph's operations still match the audit. Shared parent/line values cancel symbolically; independently interval-subtracting two full world ranges loses that relation and cannot prove a small same-line difference.

For **different-line** separation, account for line-offset accumulation separately. `flexbox.rs:2059,2097` updates the cross accumulator using a sum and add/subtract; it is not safe to assume every earlier common-line cancellation applies unchanged. A conservative candidate budget is `E_gap = upward((31 + 4*N + 8)*eta(R))`, where `eta(R)=2^-24*R+2^-149`: two scalar operations per line, two endpoint accumulation budgets, and slack for first-line placement. The helper must audit and record that path-specific operation count; this formula is a proposed sufficient overcount, not an implemented theorem.

With epsilon>=E_same, same-line subtraction is nonpositive before the zero floor and therefore produces exact zero. For separated lines require the downward-rounded bound

`deadLower = down(L - E_gap - epsilon - E_subtract) > 0.001f32`.

Here E_subtract bounds the actual abs-minus-epsilon operation; use its checked interval, not a guessed tolerance. Also bound `deadUpper`, its square and sqrt inputs as finite. The square must not overflow merely because deadUpper is finite.

As a concrete **candidate certificate input**, N=3, parent cross upper8192, slot cross sizes1px,30px,50px and P=0 give L=1,H=50,B=16885,R=67744. Before outward endpoint refinements, E_same is0.1251735687 and the proposed E_gap is0.2059307098; L minus both leaves about0.6688957214, far above0.001. This shows a source-derived epsilon can differ substantially from historical experimental1/64. It does not impose a1px minimum on arbitrary designs: the actual inequalities determine admission. Historical percentage-only start/end cases still fail the positive-gap premise.

## Normalizer, carry and masks need their own checks

Once dead is positive normal-range and above the early return, the projected vector has exactly one nonzero component. The actual immutable operation is `dead * (D / sqrt(fl(dead*dead)))`, followed by target addition and **Vec2D::lerp at strength1** (`distance_constraint.rs:75–77`, `vec2d.rs:41–42`). Retain correlation between dead and its length. Before interpolation, a conservative lower factor `(1-u)^2/(1+u)^2` relative to D is comfortably above1/2 when the square/sqrt/divide/multiply remain finite and normal.

Do not stop that proof before interpolation. Native lerp computes `(b-a).mul_add(1,a)` after a rounded subtraction; strength1 does not make it an exact copy. With a very large dead value, cancellation can destroy b even if normalization itself was finite. Bound subtraction error by the actual `|b-a|` interval, then the fused add result; require the resulting normalized lower bound, after interpolation and zero-anchor subtraction, to remain strictly above D/2. With the concrete moderate-coordinate R above the bound has ample margin, but mere finiteness is insufficient. Doubling and clamping then gives exact65536; zero returns early and stays zero. The corresponding upper bound must keep doubling finite. This missing interpolation obligation is directly visible in current immutable source, not a new native failure capture.

A generic independent interval calculation of `dead * (D/length)` loses correlation and may be too weak. A dedicated scalar-normalizer lemma plus the actual lerp error bound is the smaller helper than broadly exposing unchecked arithmetic or assuming DistanceConstraint is an exact sentinel primitive.

Carry reset additionally requires every measured line/slot extent upper, including landmark-difference error, to be at most65536. Exact unsigned sentinel subtraction followed by max(0,...) then clears the previous-line carry. Do not use only CSS h_i if the measured bottom-minus-top path has error. Same-line carried maxima must retain their own size/alignment error envelope; correct binary membership alone does not prove final geometry.

Current paint masks are centered at16384 with size32768, giving bounds[0,32768]. Their zero active edge has no positive margin around the observed viewport[0,16384]. A concrete ordinary-file candidate expands them to[-m,32768+m] by retaining center16384 and using size32768+2m, where representable m is derived from active-mask transform/raster uncertainty. Require `m >= E_active + rasterMargin` and `65536-m-E_inactive > 16384+rasterMargin`; audit the orthogonal axis too. Do not declare m=1 universally sufficient without deriving those errors. Exact0/65536 signals remove their normalization residual but not every mask transform or coverage obligation. Runtime viewing outside the declared artboard-local viewport or with arbitrary transformations requires a different observation certificate.

## Certificate output and next implementation seam

Return an internal `WrappingCertificate` containing finalized role/record bindings, domain, native per-slot intervals, L/H/N/P/B/R, counted operation graph, epsilon, positive dead interval, normalizer proof premises, measured carry bounds, mask coverage inequalities and exact charged record cost. Return contextual unresolved reasons for any missing premise. This certificate is compiler-only data, never a runtime-policy sidecar.

Implement the first machine-interval resolver and slot-shape binder before public routing; then derive the equations above from real source inputs and emitted records. Connect the private snapped sizing experiment to that certificate only after the different-line operation count and mask expansion have evidence. No current scalar or flex helper proves those missing steps automatically.
