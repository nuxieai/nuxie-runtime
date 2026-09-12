# Private wrapping size and base-scene binding checkpoint

The compiler now has two executable prerequisites for source-derived wrapping admission: `src/wrapping_sizes.rs` and `src/wrapping_slots.rs`. They are private and not called by the public HTML/CSS compiler. Public wrapping remains **unadmitted and unresolved**, not demonstrated impossible. No runtime, renderer, schema, shared dependency or host behavior changed.

## What is implemented

The size resolver encloses actual binary32 dimensions over a supplied closed parent domain. It binds authored scalar provenance to final ordinary units and float bits, resolves percentages using the immutable `(coefficient * parent) * 0.01f32` order, rejects intermediate overflow before any clamp, and applies min-after-max selection with the original parent basis. Bounds may mix point and percentage units. It requires explicit numeric minima and treats automatic preferred sizes/minima as unresolved. A positive authored percentage can still produce a zero machine lower bound; the line-extent helper rejects that as insufficient for the current positive-gap route. These intervals do not claim ideal CSS equality.

The structural binder borrows the examined completed base-record array. It accepts one top-left root-origin wrapping parent, definite independent direct slots, unique styles and one solid-painted visible child per slot. It checks complete child ordering and ownership, native units/dimensions, identity transforms, zero insets/margins/gaps, positional alignment, and ordinary fixed-size layout. Unknown records, constraints, interpolation, intrinsic/scale overrides and shared styles reject. Native direction decoding was checked against source:0/1 are column/column-reverse,2/3 row/row-reverse, default2. The artboard accepts only nonreverse0/2. This is a base-scene binding, **not yet a certificate for the augmented constraint/landmark/mask graph**.

Nine new tests cover consecutive representable percentage inputs around zero/subnormal/normal/large transitions; mixed/conflicting bounds and original parent bases; masked overflow; stale metadata/native bits; positive-source underflow; all72 native direction/alignment/wrap combinations; multiple slots and reordered/missing/cross-owned roles; style and transform mutations; and actual bound fields passed into the size resolver. The arithmetic tests exercise enclosure across finite samples; the all-domain justification is monotonicity of the pinned nonnegative operations, not extrapolation from those samples. An independent agent reviewed the resolver against immutable source and found no arithmetic soundness defect within that stated contract. Root reviewed the structural binder and the native alignment mapping.

## Validation performed

Frozen build: `output/wrapped-size-binding-build-r1/frozen`. All **370 Rust /56 Node tests**, strict TypeScript, native/WASM builds and the immutable source guard pass. The build binds290 source/artifact entries and verifies source stability throughout compilation and transport tests.

- CLI SHA-256: `fe27c13223299189e7c92edea2d5d9cb1c45e2a892913411b6c4aaf5cdc055b9`.
- WASM SHA-256: `3f8bb54d61de9875941b5e631a721e776750e34d82ea2ac534fc2b4e1a2f6031`.
- All282 prior image request/file/map outputs reproduce exactly in `output/wrapped-size-binding-existing-r1`.
- All794 historical outputs reproduce exactly in `output/public-transport-malformed-size-binding-regression-r1`.
- `python3 validation/wrapped-size-binding-evidence.py` verifies7,846 current/frozen/artifact bindings, including actual request/file/map bytes and build logs. It does not compile or render. Verification output is `output/wrapped-size-binding-verification-r1.json`.

No new native or Chrome captures were needed: the new helpers are not publicly routed, and all checked public files remain exact. Earlier wrapped sizing evidence remains the48-scene384-frame private campaign. This checkpoint claims neither new visual inspection nor public qualification from those prior pixels. Every earlier image/text/threshold failure remains preserved.

## What must happen next

Connect authored parent/slot domains to the base binding, then prove that final helper augmentation preserves the bound layout roles. Implement actual accumulator interval bounds, the conditional same/different-line error ledger, rounded normalizer length and interpolation bounds, carry reconstruction error, and active/inactive mask coverage. Bind every premise to the actual emitted graph and charge its real resource cost before exposing a public profile.

The independent source audit `wrapped-gap-normalizer-audit.md` corrects important assumptions in the earlier plan: line stretch can exceed H; the31+4N different-line count requires a shared machine offset and a validated magnitude envelope; the normalizer needs a rounded length and interpolation check; and local-space max can overshoot its operand by an ulp. Propagate that carry error instead of assuming measured slot bounds automatically bound all carried values. Mask margins remain unresolved. The experimental1/64 epsilon remains a private experiment input.

Overall backlog counts remain13 qualified /23 partial /4 investigating /59 pending. This checkpoint closes implementation prerequisites; it does not close a backlog row or the overall goal. Work remains isolated and unpublished.
