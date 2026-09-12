# Flex literal metadata and structural admission prerequisites

The r3 experiment adds executable CSS-literal error accounting and a structural descriptor checker, without changing the compiler or runtime. It corrects the parent-percentage premise in r2. Eight original single-pass cases now have modeled whole-domain bounds below **0.028109 px**; a separate non-dyadic group using `em`, ordering, and point bounds is below **0.073652 px**. The sixteen freeze-dependent cases remain unresolved. These are bounds against exact CSS arithmetic, not browser or pixel qualification, and public flex admission remains unchanged.

Run `python3 tools/html-to-riv/output/flex-literal-descriptor-r3/run.py`. The output includes descriptors, exact bounds, scalar parser/arithmetic traces, negative cases, and the source/build receipt. The runner has 140 passing assertions and rechecks 512 previously captured native comparison errors against the corrected bounds. It makes no new native or browser run.

## Why the original CSS value must survive normalization

A single decimal-to-binary32 ulp allowance is insufficient for this compiler's current path. Stylesheet declarations initially retain authored token text (`css.rs:190–238`), but `ordinary_value` tokenizes and serializes values before computation (`css.rs:75–128,157–160`; `compiler.rs:319–337`). The serializer can change the value beyond half an ulp. The scalar regression `100.71428680419922px` normalizes to `100.714px`; the tracer records the actual resulting bits. Root's separate `flex-freeze-cascade-r1` evidence found the corresponding native cap mismatch and a corrected serialized-input comparison. This experiment records only its own scalar trace, not a new rendering claim.

Percentage tokens have an additional path: cssparser parses the numeric value in binary64, divides by 100, stores a binary32 unit value, then serialization multiplies the unit value by 100 again. The compiler's size parser subsequently parses the printed percentage amount as binary32. This need not preserve the original decimal or first token's binary32 value. Rather than assuming a tokenizer rounding theorem, r3 pairs the original exact decimal with the actual final bits.

Custom-property parsing preserves Number spelling but serializes Dimension and Percentage tokens (`variables.rs:109–117`). Expansion preserves token boundaries with comments (`145–157`) and selects only the resolved value/fallback (`167–195`). Consequently numeric provenance must travel with the selected tokens through `Variables`, including inheritance and fallback selection. Recovering it later from a computed float or searching for a matching number in the stylesheet is unsound. Losers still undergo the compiler's existing strict validation; metadata must follow the winning computed field without weakening that behavior.

## Metadata carrier and computation rules

The prototype uses `Num { ideal: Fraction, native: exact-binary32-value, origins }`, with exact conversion error `abs(native - ideal)`. A computed dimension additionally retains its unit kind. The production equivalent should store native bits, an outward enclosure of exact CSS semantics, an exact-zero flag, and provenance/expression identity. It must not replace a missing ideal with the native value.

Use the existing computation order and update the carrier in the same operation that updates the float:

- Point lengths and factors: obtain the ideal from original numeric token provenance and pair it with the actual parser result after every normalization step. Numeric exponent spelling and comments are handled by the existing tokenizer; do not invent another CSS grammar.
- `em`: multiply the ideal coefficient by the ideal computed font; record the actual existing binary32 multiplication. `rem` uses the profile's exact root font size 16.
- Font percentages: ideal `coefficient / 100 * parent_font`; actual `fl(fl(coefficient / 100f) * parent_font)`, matching `computed_font_size` (`compiler.rs:207–218`). This is different from runtime preferred-size percentage resolution.
- Font-size is resolved first, against the parent font, before other relative lengths, regardless of declaration order (`compiler.rs:333–338`). A later `font-size` declaration must therefore affect an earlier width/basis `em` declaration in the same element.
- Inheritance copies the computed pair. An inherited `em` length is already an absolute point value; it must not be recomputed with the receiving element's font. Inherited percentages retain their coefficient and resolve against the receiving containing block later.
- Flex shorthand expands to three independently tracked computed fields. Longhands replace only their field. CSS-wide keywords copy the parent field or install the existing exact reset constant; omitted shorthand basis remains percentage zero, distinct from explicit point zero.
- Preserve an exact-zero classification independently of enclosure width. A tiny nonzero factor that normalizes to zero changes the active/frozen classification; r3 diagnoses this instead of treating it as an authored zero. A nonzero unitless basis that underflows to zero is likewise not an exact CSS zero.

The tracer is built against the installed cssparser 0.37.0 artifact. It records successive token serialization, size/factor parser bits, Rust binary32 multiplication/division, and a repeated multiplication chain. Python's `round32` uses rational nearest-even rounding, avoiding double-rounding assumptions when simulating compiler arithmetic. Its scalar outputs are compared to the Rust trace.

A concrete font counterexample demonstrates why final size caps are not error budgets: starting at 16 and applying `font-size:1.1em` one hundred times produces **220490.171875**, below one million, but differs from exact decimal arithmetic by **0.374437844 px**. The Rust scalar chain matches the metadata result exactly. Using that value as an item cross size is diagnosed. This is scalar/compiler-arithmetic evidence, not a new native renderer observation.

## Correct percentage-owner bounds

Immutable preferred/min/max dimensions preserve Yoga's operation order:

```
actual = fl(fl(percent_amount * owner_size) * 0.01f)
ideal  = exact_percent_amount * ideal_owner / 100
```

This is explicit in `layout_style_applier.rs:719–723` and Taffy's `util/resolve.rs:39,59,80`. The envelope must include the literal coefficient error, owner error, both multiplication roundings, and the difference between binary32 `0.01f` and exact 1/100. R3 evaluates this expression directly using the forward-error primitives.

The viewport input is conservatively enclosed over `(0,16384]`, including a possible host-to-binary32 conversion. For a literal 50% owner, the resulting ideal parent range is `[0,8192]` and its absolute error bound is **0.001647950 px**. R2 supplied only about 0.000488282 px; its original whole-domain claim was therefore conditional on an unproved premise. The earlier review and receipt are marked accordingly, and the historical numerical artifacts remain preserved.

Padding uses the distinct `LengthPercentage` path: first `fl(percent_amount / 100f)`, then multiply by the owner (`layout_style_applier.rs:728–736`). R3 keeps a separate `padding_resolved` function and tests that the two envelopes differ. Padding is still excluded from this flex proof; the function prevents mistakenly reusing the preferred-size formula.

Corrected group bounds for the original cases are:

| Profile | Ordinary CSS direction, either axis | Reverse CSS direction, either axis |
| --- | ---: | ---: |
| equal-partial | 0.009406450 px | 0.024118908 px |
| zero-unequal | 0.012612823 px | 0.028108607 px |

The model now accepts `{ideal, native}` for each basis/factor/point bound. All sums, products, divisions, branch joins, and final native placement propagate these input errors. It proves both ideal and native clamps inactive. Native Fill still links shrink to grow; F1 requires exact ideal and native equality of positive-basis factors. F2 requires ideal and native basis exactly zero and excludes interaction between unequal zero-basis factors and positive-basis shrinking participants. Its irrelevant zero-scaled shrink factors are normalized only after that group-level semantic check.

## Mechanically checkable structural descriptor

The checker consumes computed-style facts plus emitted/lowering-plan facts; it does not accept externally asserted proof booleans. It builds the model contract only after checking the following:

1. The chain to the flex container has definite point/percentage preferred dimensions, exact zero insets and minimums, no maximum, and no flex-sized ancestor. This prototype handles sole-child ordinary physical-start ancestry, proving world origin zero. Multiple siblings or reversed ancestor placement requires a previous group/world bound instead of inventing zero.
2. The group is ordinary LTR horizontal writing, nowrap, physical flex-start, with zero margins/padding/border/gaps/scrollbar gutters/relative insets. Baseline, distributed alignment, and unsupported cross alignment diagnose another proof requirement.
3. Flexible items have automatic preferred main size, explicit point basis, and point min/max bounds. Fixed siblings have a definite point preferred main size. Cross sizes follow the definite point path. Fixed ordinary descendants are restricted to a single physical-start chain; more complex descendant prefix placement needs recursive proof. Ancestor, cross-size, and descendant point-conversion errors are included in the budget instead of being inferred from a magnitude cap.
4. Actual native participants match the complete stable `(order, DOM-index)` sequence after compiler reversal. Actual native direction/alignment match the mapped flow. Actual main/cross scale, preferred cross size, fixed preferred main size, Fill fraction, explicit point basis and units agree with metadata.
5. Actual own transforms/origins, constraint/animation state, and aspect state satisfy the identity path. The complete emitted-record scan must report no unmodeled/transformed records; the negative case for a late ancestor-added constraint checks this requirement. Public extraction must perform this scan after all emission, as the separate production seam does.
6. Initial and final min/max/zero clamps are proved inactive by the numerical model, and all relevant intermediates/divisors remain finite and safe. Otherwise return an unresolved reason. No fixture identifier is involved.

The prototype tests synthetic computed-style/plan inputs and descriptors reconstructed from the original corpus specification. It does not invoke the current compiler extraction seam or claim that such metadata has already been attached to real public Styles. The integration must derive these facts from actual records/plans, including all descendants and ancestor-added helpers. A missing proof remains explicit; it cannot be converted to true because a test fixture passed.

## Production integration plan

Keep the current staged extraction seam read-only with respect to admission until provenance exists. Its `None` errors and explicit missing-premise diagnostics are correct.

First, augment numeric tokens/custom-property token nodes with stable provenance IDs before serialization. Carry those IDs through selected substitutions and cascade evaluation; keep token boundary handling unchanged. Pair final computed float fields with ideal-expression metadata at `computed_size`, `computed_font_size`, bound handling, and flex application. Use one `ComputedScalar` carrier rather than parallel ad hoc lookups by property name.

Second, resolve the parent/ancestor size envelopes from those carriers using the exact native operation path for each unit family. Feed the current emitted-topology descriptor with ideal/native pairs, actual ordered participant facts, and the proved parent/world envelope. Check all native record topology only after emission finishes. Preserve unknown ancestor, intrinsic, helper, transform, or literal premises as diagnostics.

Third, run the no-freeze numerical analysis. Exact rational arithmetic is a useful oracle, but production can use bounded outward intervals. The included `decimal_enclosure` algorithm retains at most 34 significant decimal digits and encloses the discarded tail by one last-place decimal unit; it normalizes exponent and digit counts before scaling. Extreme positive values diagnose range overflow; extremely small positive values get an enclosing interval plus exactZero=false. This avoids constructing enormous integers from million-digit tokens without excluding ordinary non-dyadic numbers. Outward conversion of its bounded rational endpoints to binary64, followed by directed interval operations, can implement the carrier. F1 exact factor equality still needs canonical decimal/provenance equality, not equality of two overlapping intervals. The bounded-decimal helper is tested separately; the r3 group oracle still uses exact rational ideals.

Finally, make the caller's numerical admission decision only when structural premises, all relevant axes/descendants, conversion error, and the final position budget are established. Keep the freeze-loop profiles staged. A computation-budget exhaustion can return an analysis-unresolved diagnostic; it is not a CSS grammar restriction or an empirical factor cap. Browser geometry and visual validation remain separate gates even after this arithmetic proof succeeds.

## Evidence and limits

The 140 assertions cover normalization through one/two/three passes, actual Rust size/factor bits, multiply and divide/multiply agreement, native preferred-percent arithmetic, bounded decimal enclosure including a 20,000-zero tiny literal, shorthand/longhand replacement, computed font order, inheritance, custom-property provenance, the corrected original eight cases, a non-dyadic case, and twelve structural/conversion rejection cases. The existing 512 comparison errors are rechecked against the new bounds; no fresh geometry, clone, raster, or Chrome observation is created.

`receipt.json` binds scripts, exact outputs, the Rust tracer and libraries/build command, compiler source snapshots, immutable percentage-resolution sources, and the retained prior observation evidence. The review cites compiler snapshots under `output/flex-literal-descriptor-r3/source`, because public extraction work was proceeding independently. No compiler/runtime files were changed by this task.
