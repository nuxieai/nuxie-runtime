# Flex factor resource and numeric bounds

The limits of 8192 authored items, factors at most1000000, and declared sizes at most1000000 do not establish general numeric safety. A single group whose **resolved** sizes are bounded has ample binary32 exponent range, but its sums can already lose placement-relevant precision. Percentage chains can overflow despite individually valid declarations. Public admission needs resolved-size and aggregate-error bounds in addition to per-token validation. This audit establishes arithmetic risks and guard requirements, not browser pixel equivalence.

## What the unchanged code computes

The vendored native flex loop sums outer bases/targets (`flexbox.rs:1236–1276`), sums grow/shrink factors sequentially (`1278–1282`), applies the below-one factor rule (`1284–1295`), then distributes growth as `basis + free_space / sum_grow * grow` (`1318–1323`). The order matters: it divides first. Shrink uses products `inner_basis*shrink`, a sum of those products, and `free_space*(scaled/sum_scaled)` (`1325–1337`). Min/max violation totals select which items freeze (`1345–1380`). Rounding can therefore affect both placement and later control flow, not just the final pixel conversion.

The factor parser accepts finite binary32 numbers from zero through1000000 (`src/flex.rs:50–58`). The size parser accepts up to1000000 in the declared unit, including percent (`src/compiler.rs:151–163`). Percentage magnitudes are not resolved pixel-size bounds. The current contextual flex guard requires a definite main-size ancestor chain and diagnoses intrinsic/flex-sized ancestors; that semantic guard should remain until independently qualified.

## Single-group exponent bounds

Assume N≤8192, grow/shrink≤K=10^6, every resolved basis/minimum/target and relevant definite container extent bounded by M=10^6, nonnegative finite inputs, and zero margins/gap. Then exact factor sums and size sums are bounded by `N*K` and `N*M` (8.192×10^9). A scaled shrink factor is bounded by `K*M=10^12`, and its sum by `N*K*M=8.192×10^15`. These are far below binary32's largest finite value (about3.4×10^38). Rounding allowances do not change that conclusion.

The growing branch with sum below one scales initial free space by that same sum before dividing; with sum at least one the divisor cannot amplify free space. The shrinking branch checks a positive scaled sum before division, and each scaled factor is at most that sum up to ordinary rounding. Consequently tiny positive factors alone do not imply division overflow in this definite, zero-gap loop. Do not diagnose every tiny factor as an infinity risk. Underflow and the explicit `free_space.is_normal()` gate remain relevant: subnormal/zero free-space products bypass distribution. The documented bounds must apply to actual resolved inputs, not merely specified tokens.

These exponent estimates are not a precision qualification, and they do not cover a different caller supplying unbounded resolved percentages, measured content or generated helper values.

## Concrete arithmetic counterexamples

`output/flex-factor-resource-r1/probe.rs` reproduces source-order binary32 arithmetic directly, without changing runtime sources or claiming native/Chrome rendering evidence. Its log records:

1. Sequentially summing8192 dimensions of1000000 yields8191544832 instead of8192000000: an error of−455168 pixels in the aggregate. All inputs are valid and every operation stays finite.
2. Summing factors `[1000000, 0.01, …]` with8191 small entries stays1000000: each addition is smaller than half an ULP at that magnitude. Using the native divide-then-multiply expression with free space16384 assigns16384 to the large item and about0.00016384 to each small item. The real sum of the stored target values is16385.342013…, exceeding available space by1.342013 pixels. This is not a complete flex-layout execution, but it demonstrates the aggregate precision issue before later layout processing.
3. Nine nested declared1000000% dimensions multiply the parent by10000 at each level. Starting at viewport16384, the ninth scalar result is infinity. Depth9 is below128 and every percentage token is within the parser limit. The parent task also reproduced this with an actual file: nine nested divs, widths1000000%, heights1px, viewport16384×32. The frozen public-auto-margins-r3 CLI accepted it (exit0); the immutable baseline probe exited1 with `nonfinite observed layout at frame 0, object 36`. Inputs, RIV bytes, map and actual commands are preserved under `output/nested-percentage-overflow-r1/` and bound in this audit receipt. This is actual native geometry evidence, with no rendering or Chrome comparison.
4. The smallest positive binary32 factor multiplied by0.5 underflows to zero; `is_normal()` is false. A lower-bound proof that assumes every positive authored factor produces a positive native allocation is invalid.

A scaled-shrink sum probe also confirms approximately8.192582×10^15 remains finite. This checks exponent headroom, not allocation accuracy.

## Intrinsic calculations are a separate domain

The intrinsic path (`flexbox.rs:1011–1164`) measures or bounds content contributions, computes `diff = contribution-basis`, and divides positive differences by `max(1,grow)` or negative differences by `max(1,shrink*inner_basis)` (`1111–1123`). Thus it does not divide by arbitrarily tiny factors. Later it multiplies the stored fraction by `max(1,grow)` or `max(1,shrink)*inner_basis` (`1144–1157`). Note the different placement of `max(1,...)` in the negative case: it cannot simply be canceled symbolically with the earlier divisor for arbitrary factors/bases.

For one intrinsic call with a separately established content bound C, coarse intermediate bounds can use `abs(diff)≤C+M` and `abs(contribution_term)≤max(K,K*M)*(C+M)`. Summing N such terms still has finite headroom for modest bounded C, but the token limits supply no general C across an intrinsic subtree. Recursive percentage resolution, overflowing descendants and repeated intrinsic sums require their own interval propagation. Treating C as viewport size is unsound.

The code uses infinities internally for absent maxima and available-space sentinels; those intentional option/sentinel values should not be confused with an overflowed actual size. Conversely, the loop does not report a compiler-style diagnostic when a computed size becomes nonfinite. `is_normal()` can skip distribution and comparisons on NaN can select fallback branches. Prevent unsafe inputs before serialization rather than relying on those branches to reject them.

## Required guard strategy

1. **Propagate used-size intervals.** Resolve admitted percentage chains over the declared viewport envelope with outward-rounded arithmetic. Bound relevant parent bases, preferred dimensions, minima/maxima, bases, intrinsic sums if ever admitted, and helper nodes. Reject or defer plans whose upper bound is nonfinite or outside the selected numeric domain. A finite upper viewport alone is insufficient; arbitrary percentages greater than100% amplify it. Open lower viewport bounds also prevent general positive-allocation claims.
2. **Bound sums per group, including generated participants.** Retain authored count and factor/size caps, but compute upper bounds on sum of bases, sum of frozen minima, sum of factors and scaled shrink products. Do not normalize factors: the below-one behavior is meaningful and normalization changes layout. The 8192 authored-element guard does not by itself count added alignment/spacing objects.
3. **Enforce an error budget, not just finiteness.** For nonnegative sequential sums with n additions, use `gamma_n = n*u/(1-n*u)`, `u=2^-24`, and an underflow allowance. A conservative sum-error bound is `gamma_n*sum_exact`. At8192 the relative bound is roughly0.000489; this is not automatically acceptable for layout. Add input quantization and subsequent division/multiplication error, then compare with an explicitly chosen geometry budget. Where sums/violations straddle branch thresholds (one, zero, min/max freeze), either prove branch stability or retain an interval across both branches. Near-canceling violation sums cannot use a relative-error-only test.
4. **Account for cumulative position error.** Even individually tiny target errors can accumulate across thousands of children. Bound prefix sums as well as total allocation and final widths. The factor counterexample shows why checking each small item against a width tolerance does not establish the group's final edge.
5. **Keep intrinsic F1/F2 admission blocked without its separate proof.** The definite-chain guard is necessary for the intended semantics and avoids the unbounded-content arithmetic domain. It does not replace the resolved-percentage bound on that definite chain.
6. **Bound work as well as records.** Each freeze iteration scans items and allocates an unfrozen vector. At least one item freezes per finite nonzero-violation iteration, so one group can require O(N²) work and O(N) temporary storage. N=8192 permits approximately67million item-visits before constants and nested measurement work. Count generated participants and sum per-group quadratic costs when setting a runtime resource budget. No measured performance threshold is established by this audit; choose a budget through separate profiling, not an invented latency claim.

The current independent geometry driver rejects differences greater than0.1px (`validation/check-public-baseline.mjs:65`); this is an existing validation budget, not a proof that every error smaller than0.1px is visually harmless. Any analytical guard must reserve budget for input quantization and other stages rather than assigning the full tolerance independently to every sum. A small first profile with a conservative participant limit and computed interval/error checks is implementable. The exact count/geometry budget is a product policy and validation decision; this audit does not invent a new empirical cap. Broader factor syntax may remain parsed while unsafe or unqualified group plans receive a precise diagnostic.

## Reproduction and immutable source bindings

Probe commands, source and binary/log hashes are in `output/flex-factor-resource-r1/receipt.json`. The probe is scalar arithmetic only, not an importer/native replay or browser test. Runtime/vendor sources below were checked against immutable baseline `6c7ac16617835b5f581784ff08a9e779bb52faf3`. Compiler parser/guard references describe the current in-progress public integration and are not claimed immutable. No runtime/public source files were edited for this audit.

| Immutable source | SHA-256 |
| --- | --- |
| `vendor/taffy-0.12.1-rive-yoga-order/src/compute/flexbox.rs` | `75f7a79993712f3f115762291819bf3aab530e79002746c63ad9ad500cc728c2` |
| `crates/nuxie-runtime/src/mechanical_port/source/layout_component.rs` | `5154d46f471e2497886b2dbd420d5a0532baf8521f6cfe655a432c60d05122c3` |
