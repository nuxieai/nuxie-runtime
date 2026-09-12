# Different-line accumulation and scalar normalizer audit

This source audit refines `wrapped-sizing-certificate-next.md`. It does not qualify public wrapping or replace a final-record certificate. No runtime changes or new rendering campaign were made. The arithmetic example below was evaluated with binary32 packing in Python, not presented as a native scene observation.

## Different-line count

Use the conditional fourteen-operation per-anchor and three-operation pair ledger in `wrapped-anchor-error-audit.md`. Its 31 coefficient stops at the measured signed difference. Let every actual scalar result on rounded operands have magnitude at most R and let `eta = 2^-24 R + 2^-149`, with all bounds rounded outward.

The following additional conditions are required:

- Native wrapping is enabled. Every slot has the already-proven definite cross size invariant, zero margins/insets and no baseline alignment.
- **Line** alignment is positional, not `align-content:stretch` or distributed alignment. Rejecting only stretched items is insufficient. `flexbox.rs:1591–1597` adds available space to line extents even when all items have definite sizes. Otherwise line extents need not be bounded above by the maximum slot extent H.
- Positional first-line offset O is treated as a shared **machine value**, just like each machine line extent s. Subsequent line offsets are exactly zero for zero gap (`compute/common/alignment.rs`, non-first positional arm). A common alignment fraction f applies to all measurement slots, after physical direction conversion.
- Ancestor linear transforms are identity, root translation conventions match generated nodes, and all participants share the same parent.

In physical traversal order, LTR cross placement uses

```
T_0 = 0
T_(j+1) = fl(T_j + fl(O_j + s_j)), O_0 = O, O_(j>0) = 0.
```

The two syntactic operations are the addition inside and the accumulator addition in `flexbox.rs:2097`. RTL columns use the corresponding subtraction **before** placement (`:2059`), initialized from container cross size (`:2107–2111`). Wrap reversal reverses traversal; it does not introduce another accumulator. A full interval implementation may exploit additions of zero as exact, but charging two operations per visited line is conservative.

For a line reached after at most N updates, accumulator error against exact arithmetic on shared machine O and s is at most `2N eta`. Each endpoint has that budget; therefore

```
E_gap = upward((31 + 4N) eta)
```

is sufficient **conditional on the existing anchor ledger and a validated intermediate envelope**. The previous proposed `(31 + 4N + 8) eta` is a conservative overcount under these same conditions. No term for the floating-point sum used to compute O is necessary in this pair error: O is one shared machine input which cancels algebraically. Its size and finiteness must still be bounded. Counting O against an ideal CSS offset instead would require additional sum/free-space accounting and would not justify a constant eight-operation allowance for arbitrary N.

For adjacent LTR physical lines the exact anchor gap on these machine inputs is `(1-f)s_j + f s_(j+1)`; RTL reverses the corresponding physical ordering/sign. For nonadjacent lines, intervening positive extents add to that gap. Thus a source-derived positive lower bound L for every line extent suffices without enumerating partitions.

Do not assert that the proposed B formula bounds machine intermediates merely because it bounds an exact sum. Compute an outward interval for the actual line-size sum, first-line offset and accumulator, or validate explicit summation-error slack. A simple interval recurrence over at most N unknown extents in [L,H] is enough for magnitudes; correlated O must remain shared only in the separate error proof. Deriving R from that validated interval avoids a circular assumption that R already bounds its own rounding error.

The absolute-value composition is special: with finite `2*d`, identity transforms and root-zero helpers, local max(d,-d) computes an exact sign flip or copy. This exactness does not generalize to arbitrary graph maxima.

## DistanceConstraint after the early-return check

Bind zero target world translation and zero component local anchor explicitly. With one nonzero scalar component d>0, `Vec2D::length_squared` (`vec2d.rs:18–23`) gives `q=fl(d*d)` in either orientation: the other component is zero. The code uses an FMA for x but adds exact zero, so it has the same scalar square result as the y case. Then

```
l = fl(sqrt(q))
t = fl(D/l)
p = fl(d*t), D = 65536
z = fl(fma(fl(p-d), 1, d))
```

The last expression denotes one rounded subtraction and one rounded FMA; the surrounding notation does not add a second rounding after FMA. See `distance_constraint.rs:55–78` and `vec2d.rs:41–42`. Target addition and final zero-anchor subtraction are exact under the bound shape.

Check `q`, `l`, `t` and `p` are finite and normal, and that the actual lower bound for **l**, not merely d, exceeds `0.001f32`. One convenient sufficient route uses directed endpoints for q and l. A theoretical `d > 0.001f32` by an arbitrarily small amount alone is not a bound on the rounded square-root result. The current implementation returns early for `l < 0.001f32`; equality would pass, but strict separation is a simpler certificate premise.

For u=2^-24, standard normal binary32 relative error gives conservative correlated projected bounds

```
p_lo = downward(D * (1-u)^2 / (1+u)^2)
p_hi = upward  (D * (1+u)^2 / (1-u)^2).
```

The sqrt factor is bounded conservatively by 1±u as well as the sqrt rounding itself. Do not derive this interval by independently multiplying d and D/l; that discards essential correlation.

For an actual machine dead interval `[d_lo,d_hi]`, define

```
J = max(abs(p_lo-d_hi), abs(p_hi-d_lo))
e_sub = upward(u*J + 2^-149)
e_fma = upward(u*(max(abs(p_lo),abs(p_hi))+e_sub) + 2^-149)
z_lo = downward(p_lo - e_sub - e_fma)
z_hi = upward(p_hi + e_sub + e_fma).
```

Validate all intermediate subtraction and FMA ranges as finite. Requiring `z_lo > D/2` and finite `2*z_hi` proves the factor-two copy followed by the upper clamp produces exactly D. Strength one is **not** an exact copy in this DistanceConstraint because its Vec2D lerp differs from the direct weighted-sum interpolation in TranslationConstraint. Finiteness without the z lower bound is insufficient.

Zero d follows the early-return branch and stays zero under the same zero target/anchor/orthogonal-coordinate bindings. No relative-error lemma applies to that branch.

## Carry maxima need a further bound

`Graph::max(a,b)` is implemented with a node parented to a and a TranslationConstraint targeting b, clamped in local space. `translation_constraint.rs:101–132` first transforms b into the parent's local space, clamps and transforms back. For pure translations the b-selected branch is

```
m = fl(fl(b-a)+a)
```

and can differ from b. For example, exact machine values `a=3*2^-24`, `b=1+3*2^-23` produce `m=1+4*2^-23`, exceeding b by one binary32 ulp. The a-selected branch copies a exactly, assuming finite operands and identity transforms. The same two-rounding error bound used above, with a and b substituted, bounds the reconstruction branch; unlike DistanceConstraint this final add is ordinary addition, but at multiplier one the FMA has the same exact real input and error bound.

Consequently, proving every measured slot extent is <=D does **not by itself** prove every carried value is <=D. Propagate a machine interval through the N−1 carry maxima in each direction, and through their final maximum. On a known same-line gate-zero step, `diff(carry,0)` and nonnegative relu are exact copies under the reviewed shape, so only the max reconstruction needs new error. On a different-line gate-D step, first prove the entering carry upper bound <=D; then subtraction is nonpositive and relu clears it exactly. If using one simple range [0,U] for nonnegative a,b, a sufficient per-max upper increment is `eta(U) + eta(U+eta(U))`, subject to finite intermediate checks. Feed the resulting increment forward instead of assuming U remains constant. This also supplies an explicit accumulated geometry error for the final offset calculation, which exact line gates alone do not establish.

Mask coverage and raster margins remain separate unresolved obligations. This audit does not resolve them.

## Source identity

All six native sources below were compared byte-for-byte with `git show 6c7ac16617835b5f581784ff08a9e779bb52faf3:<path>` and matched. The compiler wrapping helper is a separate current-source binding.

| Source | SHA-256 |
| --- | --- |
| `vendor/taffy-0.12.1-rive-yoga-order/src/compute/flexbox.rs` | `75f7a79993712f3f115762291819bf3aab530e79002746c63ad9ad500cc728c2` |
| `vendor/taffy-0.12.1-rive-yoga-order/src/compute/common/alignment.rs` | `f082c4a5b0607c0083c5b774c66b4006bd7324dc80159674df1141c7a20f0547` |
| `crates/nuxie-runtime/src/mechanical_port/source/constraints/distance_constraint.rs` | `ee2d2df3ec8fe3bf2d23ff974823c7f71260206ff7f50c15c44dfc6e9b9844cf` |
| `crates/nuxie-runtime/src/mechanical_port/source/constraints/translation_constraint.rs` | `b7d9eb0c64ebb1da20fbf37a296e16c41e8ed42b437d59c9f849e39b8d5eac14` |
| `crates/nuxie-runtime/src/mechanical_port/source/math/vec2d.rs` | `63bc1279d9f3a2258b6a22f1d0637cb7492cd7370e42bd15e36dace740a69a5a` |
| `crates/nuxie-runtime/src/mechanical_port/source/math/mat2d.rs` | `f7a43f39caef3529a3ee1a16f8b0dd06d9370f09d8185ecbf01606c1ff475ed2` |
| `tools/html-to-riv/src/wrapping.rs` | `18460f64c664b7c73a86fef81cd30e84deb76da916e9ef2c3674128f1e899d70` |
