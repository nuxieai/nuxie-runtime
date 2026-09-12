# Private flex numerical analyzer

The compiler-private `src/flex_numeric.rs` ports the r3 single-pass error model to outward binary64 bookkeeping. It consumes production `ScalarProvenance` values and explicit parent/world envelopes. This is a conditional numerical proof API; public flex admission and emitted scene behavior remain unchanged.

## Contract and result

`analyze(&GroupDescriptor) -> Result<Analysis, Unresolved>` requires all sixteen named structural premises. The fourteen r3 premises cover zero insets/gaps, no wrapping, no auto margins, no intrinsic sizing/aspect/constraints/animation, pure translation, preserved known child targets, definite cross-start sizing, LTR, identity transforms, zero origins, physical start and native reverse flow. The added explicit premises require actual participant order and descendant geometry error accounting. No premise or parent/world error defaults to proved/zero. Callers must obtain these facts from the actual emitted record graph and computed metadata before using the result.

Items arrive in logical CSS order and retain original ideal scalar provenance alongside actual native coefficients. Positive-basis items require provable ideal factor equality plus native factor equality. A zero-basis item's authored shrink can differ from the native fraction only when the whole group contains no positive-basis active shrink. Tiny ideal nonzero basis/grow values cannot be silently treated as native zero. Initial min/max clamping, possible later freezing, incomplete premises, nonfinite envelopes and excessive error return explicit unresolved reasons.

The result reports ideal size/position ranges, absolute native error bounds, logical-to-file order, native traversal and branch-join error. The configured additional geometry error participates in the final budget. `ErrorEnvelope::new` requires explicit finite ordered ideal bounds and finite nonnegative error. Parent and world envelopes are supplied separately; no concrete-owner value is mistaken for a viewport-domain proof.

## Arithmetic coverage

Each nontrivial binary64 arithmetic operation in endpoint/error bookkeeping is directed outward separately. Endpoint arithmetic and propagated-error arithmetic cannot be reassociated. Exact zero shortcuts preserve the native no-rounding cases; native rounding uses unit roundoff 2^-24 plus a 2^-149 underflow term, with finite binary32 intermediate checks. Grow uses `(remaining / denominator) * grow`; shrink uses `remaining * (scaled / denominator)`. The model retains below-one factor handling, uncertain grow/shrink join bounds, normality allowance, sequential native file-order sums and native reverse-traversal placement. Final translation adds the explicit parent-world envelope.

The count-dependent gamma filter is arithmetic uncertainty, not a new arbitrary item cap. Counts at or above 2^24 are unresolved because that gamma bound is unavailable; smaller groups can still be unresolved because their error or freeze uncertainty is too large. Production extraction and its existing resource limits remain separate.

## Verification

`output/flex-numeric-production-r1/check.py` builds a harness against the actual production module and scalar carrier. Its 53 structural cases preserve all eight bounded and sixteen unresolved original r3 profiles, and the additional non-dyadic em/order/point-bounds example. Added cardinalities 1/3/4/5/8/16/32/64/128 in both orders, near-zero parent intervals, and zero-basis cardinalities exercise the structural API without fixture-name admission. There are 27 bounded and 26 unresolved results. All 617 classification and exact Fraction endpoint/error comparisons pass. Error bounds are conservatively above the oracle; bounded Rust ranges enclose the exact ideal oracle ranges.

`arithmetic.py` tests 3,891 addition/subtraction/multiplication/division operations, including signed intervals, tiny/subnormal scales, cancellation and exact zero. All 11,673 exact rational lower/upper/error inequalities pass. The harness copies production arithmetic verbatim, changing only crate documentation comments and appending a private-access test entry point. This checks the outward bookkeeping independently of the flex fixtures; it is not an exhaustive floating-point proof.

Eight analyzer unit tests check every missing premise, explicit additional/world error, ideal/native equality distinction, tiny nonzero conversion, the whole-group zero-basis shrink condition, preserved freeze failures, actual order output and invalid/overflowing arithmetic. These and the nine scalar-carrier tests pass (17 total).

## Limits

No new native render or Chrome image comparison was run for this port. The r3 native-observation evidence remains historical; this checkpoint checks the arithmetic port and production carrier interface. Source extraction, parent viewport envelopes, descendant preservation and actual participant graph verification are not automatically established by this module. Supplying false premises is a caller error. Freeze-dependent profiles remain unresolved. Browser raster equivalence, wrapped layout, fractional raster edge parity and public CSS support are not claimed.

Receipt and executable reproductions: `output/flex-numeric-production-r1/receipt.json`.

## Registered checkpoint

The analyzer is now a private compiler submodule. The sole implementation change from the standalone source is its scalar import path (`crate` to `super`). Fresh registered-module tests pass:185 Rust and35 Node tests, plus the WASM build and strict TypeScript check. The structural oracle again passes53 cases/617 checks and the arithmetic oracle3891 cases/11673 inequalities. The standalone arithmetic harness adapts the import back to its crate root; its arithmetic implementation is unchanged. See flex-numeric-registration-receipt.json for current bindings. Registration does not establish caller premises or admit public flex declarations. No native/browser rerender is claimed.
