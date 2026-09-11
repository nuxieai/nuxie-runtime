# Same-line anchor error budget on the immutable runtime

A source-derived budget is possible for the narrow, translation-only sibling-slot graph. Viewport ≤16384 and depth ≤128 alone do not establish it: admission must also bound intermediate coordinates and ensure that the size used for native cross alignment is the size subsequently read by the anchor. Do not replace those conditions with an empirical epsilon cap.

## Conditional implementable bound

Let `u = 2^-24`, the binary32 unit roundoff. Let `R` bound the absolute value of every exact scalar result in the operations listed below, including operations on already rounded operands, and require finite normal-range arithmetic without overflow. Allow gradual underflow by using `eta(R) = u*R + 2^-149` per potentially rounding operation. Then a conservative bound for the measured difference between two mathematically common line anchors is:

```
epsilon_same = upward_f32(31 * (2^-24 * R + 2^-149))
```

Compute the right side in binary64 and round the emitted binary32 value upward (next representable value if the ordinary cast rounds down). This coefficient comes from the operation accounting below, not a fitted tolerance. It is intentionally conservative; operations involving zero, ±1 or exact halving often incur no rounding at all.

Conditions for this bound:

- Slots are direct siblings in one native flex line, with the same native physical alignment fraction `f` in {0, 0.5, 1}; same-line membership is defined by native partitioning.
- Slot margins and relative insets are zero; no auto margins, baselines, transforms, nonzero component origins, or user constraints modify slots or their ancestor transforms. Ancestors have identity linear matrices; artboard-local/world coordinate conventions are consistent for generated nodes.
- A slot's final published cross size equals the `outer_target_size` used by cross alignment (with zero margins). The bound treats the machine value of that common size as the input; it is not a bound on CSS-vs-native size error.
- Every constraint involved has full strength and the fixed spaces/factors of the reviewed graph. Generated measurement nodes have zero local anchors. The graph is acyclic and evaluated after current native layout; stale intermediate values are not a rounding error.
- `R` covers all relevant intermediate values, not just final pixels or viewport dimensions. A bound `B` on only final world positions does not automatically cover local offsets that cancel, nor unconstrained intermediate helper nodes.

Under these conditions the ideal per-item expression, using shared machine values, is `P + T + O + f*L`: parent world translation P, accumulated line offset T, line alignment offset O and native line extent L. Native item placement contributes `f*(L-h)` and its anchor contributes `f*h`; h cancels. Errors in earlier computation of P/T/O/L are common inputs, so they need not be charged once per ancestor or preceding line. Depth 128 does not produce a factor of 128 in this same-line comparison. If anchors follow different parent chains, this argument no longer applies.

## Source operation accounting

The following overcount is per cross coordinate. Errors are added with coefficients of absolute value at most one; f is at most one. There is no unbounded matrix condition number because the linear transforms are identity.

| Stage | Budgeted rounding operations per anchor | Source |
| --- | ---: | --- |
| Native free space `L-h` and positional factor f | 2 | vendored flexbox.rs:1752, 1789–1840 |
| Final native location: four additions including zero margin/inset terms | 4 | vendored flexbox.rs:1973–1977 |
| Slot world composition, two translation matrix products | 2 | layout_component.rs:989–1013; mat2d.rs:255–265 |
| Local bound anchor multiplication and addition | 2 | transform_constraint.rs:28–30; layout_component.rs:1135–1140 |
| Target world translation composition | 1 | transform_constraint.rs:32 |
| Full-strength translation interpolation | 3 | transform_constraint.rs:115–116 |
| Total per anchor | 14 | |
| Pair subtraction graph allowance | 3 per pair | translation_constraint.rs:63–104, 136–143; generated diff/sum graph |

Thus `2*14 + 3 = 31`. The full-strength interpolation's multiplications by 0/1 and addition of zero are exact for finite values; charging them is conservative. Transform decomposition copies translation values, and composition writes them back unchanged (`mat2d.rs:203–246`). Identity linear matrices stay identity; `from_rotation(0)` bypasses trigonometric calls (`mat2d.rs:61–67`). `Constraint::land_anchor` returns immediately for generated zero anchors (`constraint.rs:201–205`). Translation-only inversion has determinant one and exact sign inversion; graph subtraction adds at most the three budgeted scalar roundings, including parent composition and destination-local addition. Copies with factor ±1 and strength one do not amplify errors.

This accounting stops before snapping, DistanceConstraint normalization, and min/max gates. A prototype that changes the subtraction graph, uses non-unit copy factors, adds nonzero origins, or anchors from different ancestry must recount its operations. In particular, operations on the 65536 sentinel belong to a separate gate correctness proof; they are not covered merely by a viewport bound.

## Turning B and H into R

A practical guard can use `R = 4*(B + H + 1)` if B explicitly bounds the absolute machine values of parent translations, accumulated native line offsets, line alignment offsets and final local slot locations, and H bounds both line and slot cross extents. The factor four covers the three coordinate terms plus one extent term and slack in the listed scalar expressions. Guard B/H as finite nonnegative values and verify the derived R remains far from binary32 overflow. This is a sufficient intermediate-value domain, not a claim that the current compiler already proves that domain from arbitrary CSS.

For a stricter implementation, an interval pass can bound each expression directly and substitute the maximum result into R. The fixed three-item, definite-percentage, zero-margin fixture can supply such bounds from admitted viewport and authored percentage/min/max ranges. General min/max, overflow, arbitrary ancestors or transforms need more analysis. Depth and viewport alone permit large overflowing local coordinates and cancellation and therefore cannot justify a small epsilon.

Even a valid same-line bound does not establish cross-line separation. To snap safely with threshold epsilon, prove the smallest ideal distinct-line anchor separation is greater than `2*epsilon + 0.001` (with conservative rounding margin). One epsilon accounts for the candidate measurement error and the other for the snap threshold; 0.001 is the existing DistanceConstraint early-return boundary. If that separation proof fails, reject the composition or use a separately proven construction. An upper line extent H supplies no positive lower separation bound. For common start/end/center anchors with nonnegative gap and positive extents, derive separation from the neighboring line extents and gap; do not infer it from H alone.

## Native rounding and remaining proof obligations

The runtime explicitly calls `tree.disable_rounding()` when constructing Taffy (`layout_component.rs:2072–2073`). It obtains `tree.layout(...)` and publishes location/size unchanged (`layout_component.rs:2289–2300`). The vendored tree returns unrounded results when that flag is disabled (`tree/taffy_tree.rs:254–257`). There is no whole-pixel rounding term to add. Binary32 arithmetic remains and can produce unequal common-anchor reconstructions. Chrome's own rounding or rasterization cannot supply a bound for these native signals.

Taffy computes item alignment from `outer_target_size` (`flexbox.rs:1752`), but final placement calls `perform_child_layout(...)` and publishes its returned `size` (`flexbox.rs:1936–1954, 2001+`). Definite inputs are promising, but that alone is not a source proof for all percentage/min/max, aspect ratio, border/padding and intrinsic-content combinations. The admission guard must establish equality for its selected slot profile or add a separately bounded size discrepancy term: for per-item discrepancy ≤Esize, add `2*Esize` to the pair budget. No empirical probe can replace this invariant.

This audit therefore provides a conditional operation budget and explicit obstacles to general admission. It does not certify every current profile or choose a production threshold. The concurrent prototype is `output/wrapped-snapped-gate-r1/src/main.rs`: absolute anchor difference, subtraction of experimental epsilon 0.015625, clamp at zero, normalization to 65536, then doubled/clamped separation and inverted leader masks. Its announced cases include gap ±41, translated gap41, translated 0.001953125, zero, subepsilon, epsilon and threshold transitions. Those runs were still in progress when this source audit was written; no numerical outcome is claimed here. Existing failed gap/rounding probes should be retained as counterexamples; matching their observed maximum residual is a consistency check only. No runtime/public source edits or new native experiments were made for this audit.

## Source bindings

All following sources match immutable baseline `6c7ac16617835b5f581784ff08a9e779bb52faf3` byte-for-byte. Paths above are relative to `crates/nuxie-runtime/src/mechanical_port/source/` or `vendor/taffy-0.12.1-rive-yoga-order/src/` as appropriate.

| Source | SHA-256 |
| --- | --- |
| `Cargo.toml` | `6eae874f0f25fdffe224dad17032f269c40a2f414f8a175cf88008088f49fe5e` |
| `Cargo.lock` | `2654da7218ed71d6c84d21157cddea237b278d401003ac77db5bd8376815467e` |
| `crates/nuxie-runtime/src/mechanical_port/source/layout_component.rs` | `5154d46f471e2497886b2dbd420d5a0532baf8521f6cfe655a432c60d05122c3` |
| `crates/nuxie-runtime/src/mechanical_port/source/constraints/transform_constraint.rs` | `450f0443e9499db7c5b7a4261d77fe5ede678e40f975ec0b77278c6b5b240f8d` |
| `crates/nuxie-runtime/src/mechanical_port/source/constraints/translation_constraint.rs` | `b7d9eb0c64ebb1da20fbf37a296e16c41e8ed42b437d59c9f849e39b8d5eac14` |
| `crates/nuxie-runtime/src/mechanical_port/source/constraints/constraint.rs` | `d89f7c20f18c2f3ca437129d3098a6f98b11326bd0ad121aa932570bd64f364e` |
| `crates/nuxie-runtime/src/mechanical_port/source/math/mat2d.rs` | `f7a43f39caef3529a3ee1a16f8b0dd06d9370f09d8185ecbf01606c1ff475ed2` |
| `vendor/taffy-0.12.1-rive-yoga-order/src/compute/flexbox.rs` | `75f7a79993712f3f115762291819bf3aab530e79002746c63ad9ad500cc728c2` |
| `vendor/taffy-0.12.1-rive-yoga-order/src/tree/taffy_tree.rs` | `34a5271e28364beebf77e9f55ca92cd9937cc115d7d15261902563244df6341b` |
