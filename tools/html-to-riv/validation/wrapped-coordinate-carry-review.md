# Source-derived wrapping coordinates and carry bounds

The private compiler candidate now derives epsilon from the checked source domain and emits that value through `compose_with_bounds`. This closes the implemented coordinate/dead/carry prerequisites in their stated conditional scope. It does not admit public wrapping or complete the final scalar, positioning and clipping proof. No runtime, renderer, schema, shared dependency or host changes were made.

## Implemented analysis

`wrapping_coordinates.rs` evaluates actual binary32 endpoint recurrences for native main/cross sums, positional first offsets, accumulator updates and slot locations over the full supplied viewport domain. Unknown line partitions are enclosed using up to N nonempty lines and slot extent bounds; no browser geometry or fixture-specific packing is used. Reverse directions change traversal/fractions under the checked LTR shape. Every intermediate is checked finite. Both axes are included because landmark TransformConstraints carry orthogonal coordinates before scalar projection.

The checked native envelope supplies R, rather than assuming an exact-real sum covers its own rounding. The conditional audited31-operation same-line and31+4N different-line budgets produce an upward-rounded epsilon and outward separated-anchor bounds. Dead endpoints then use the actual native f32 subtraction. The existing normalizer helper checks rounded threshold, correlated projection and interpolation/saturation safety. Measured bottom-minus-top intervals carry the same conservative anchor-pair error into the carry proof.

`wrapping_carry.rs` propagates machine upper bounds and absolute measurement/max-reconstruction errors through both directional passes and final maxima. It checks each entering reset value against65536. Local max is not assumed exact: fl(fl(b-a)+a) can overshoot b by an ulp. Final maxima are not incorrectly subjected to a reset they never enter; singleton aliasing retains exact-copy behavior. Gates are still an explicit exact0/65536 and correct-membership premise pending the complete scalar-field/evaluation certificate.

`compose_with_bounds` owns the analysis alongside the exact candidate configured with its epsilon. The older explicit-epsilon constructor remains a private experiment seam, not a public default or admission route.

## Visible-source correction

The audit found a gap in the earlier base/domain scope: finite stored visible dimensions did not prove finite resolved sizes or initial transforms. A finite percentage can overflow its raw native product, and an oversized center/end-aligned child can overflow world translation. Strength-one TranslationConstraint still evaluates old*0; infinity therefore cannot be repaired by the final target.

The structural binding now retains visible native sizes, slot placement fractions and the parent's main fraction. Domains resolve both visible dimensions against their original slot axes using native fields directly. This does not fabricate authored ideal provenance or certify visible CSS equality. The coordinate pass bounds visible local free-space alignment and world translation before any positioning constraint. Tests retain both raw-percentage overflow before a finite maximum and finite-size/overflowing-world cases.

The dimension helper still requires explicit minima. The immutable all-flex path at `layout_component.rs:2173–2184` adapts automatic minima to zero; that is a usable future premise, not evidence that automatic minima are impossible. Current declaration scope is unchanged.

## Validation and independent review

Eleven new tests cover native-only visible sizing and overflow, direction fractions, all partitions of five-item coordinate recurrences, main-axis overflow alignment and summation overflow, max reconstruction overshoot, every partition of six-item carry arrays, reset boundaries, actual emitted source-derived epsilon constants, tiny-gap rejection and initial visible-world overflow. The recurrence and scalar tests supplement source reasoning; they are not native scene observations.

Two independent source reviews found no underbounds in the implemented conditional layer. They checked native operation order (`flexbox.rs:1680–1682,1722–1731,1904–1921,1970–1977,2018,2097`), nonstretch line extents, visible nowrap placement, original percentage bases, outward dead endpoints and measurement-error transfer to carry. They explicitly did not discharge final scalar parameter/evaluation, positioning or raster coverage obligations.

Frozen build: `output/wrapped-coordinate-carry-build-r1/frozen`, with295 bound source/artifact entries. All **390 Rust /56 Node tests**, strict TypeScript, native/WASM builds and immutable source guard pass, with source stability checked across the build.

- CLI SHA-256: `a6f3f640baffa67204ef831fd8e14241041ca667b46a53c3c2da98fd99114914`.
- WASM SHA-256: `d5b1ccf6a3d30a7d6601f6d6a154c5a153129396937d4b64d5ecaf5eddcd933a`.
- All282 earlier image request/file/map outputs reproduce exactly in `output/wrapped-coordinate-carry-existing-r1`.
- All794 historical outputs reproduce exactly in `output/public-transport-malformed-wrapped-coordinate-carry-regression-r1`.
- `python3 validation/wrapped-coordinate-carry-evidence.py` verifies7,857 current/frozen/artifact bindings without compiling or rendering. Output: `output/wrapped-coordinate-carry-verification-r1.json`.

No public compiler path uses the new candidate. No new native/Chrome capture or visual inspection is claimed; all checked public bytes are unchanged. Earlier experimental1/64 pixels are not qualification for the new derived-epsilon candidate. Preserve every earlier text/image/threshold failure.

## Next work

Bind all scalar record fields, dependency structure/evaluation premises, zero anchors and ordinary defaults to this candidate. Propagate error through final max-minus-height, scale, target sum and visible copy. Establish active/inactive mask coverage with rounding/raster margins. Then capture and inspect the derived-epsilon native/Chrome original/clone resize campaign before public admission. Source-derived upper bounds alone do not qualify painting.

All99 backlog items remain in scope; counts remain13 qualified /23 partial /4 investigating /59 pending. This is a local, unpublished prerequisite checkpoint, not completion of wrapping or the compiler goal.
