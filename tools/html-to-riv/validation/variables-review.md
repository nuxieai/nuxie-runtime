# Current status

Fallback-cycle rejection described in the historical checkpoint below is superseded by lazy resolution. Both formerly rejected forms now pass; see [current cycle evidence](variable-cycles-review.md). Missing/invalid ordinary values without usable fallbacks still diagnose.

# Custom properties and var(): compiler-owned computation

Custom properties are authoring token data. They are resolved before ordinary Rive objects are emitted; no variable environment, runtime setter or host CSS evaluator is required to render the output. Percentage results remain native percentage descriptors and font-relative lengths use the existing computed font context.

The initial checkpoint followed the older published [CSS Variables specification](https://www.w3.org/TR/css-variables-1/) for case-sensitive names, resolution before inheritance, fallback references in dependency cycles, and the distinction between empty values and guaranteed-invalid values. An inherited computed alias is not re-evaluated against child overrides. Token boundaries must survive substitution so a number followed by an authored identifier cannot accidentally become a dimension. Expansion and dependency limits are explicit compiler resource boundaries.

This compiler remains stricter than browser recovery. Unsupported ordinary property names reject even when their values contain var(). A missing/cyclic value with no usable fallback or an unsupported value after substitution produces a diagnostic; it does not silently drop the declaration or roll back to an earlier cascade value. Revert/revert-layer, @property registration, scripting and runtime variable mutation are outside current admission. Custom token data may remain unused; preserving it does not admit rendering that token stream as a new property value.

The admitted 17-case corpus passes136/136 geometry and pixel frames and272 clear controls. It covers aliases, forward references, local overrides, case-sensitive names, nested/missing fallbacks, direct cycles with consuming fallbacks, RGB token insertion, whole values, percentage resize, em/font contexts, important precedence, custom CSS-wide keywords, unused token data and empty fallbacks. All authored nodes have independent expectedPaint assertions, and public Rust output is compared to handwritten literal controls.55 Rust and14 Node tests pass, including CLI/WASM exact bytes and identical no-output rejection of disputed cycles.

Nineteen unique full-frame pairs were directly inspected:17 first frames plus two additional responsive-width viewports. The remaining117 pairs transfer by exact repeat/clone image identity or verified white-canvas extension/crop. `public-variable-receipt.json` binds this review and the exact-byte source/binary checkpoint.

## Preserved browser disagreement

With `--good:navy; --x:var(--good,var(--x))`, the first resolver invalidated --x from the dependency graph and used the consuming fallback teal. Pinned Chrome153.0.8010.12 instead paints navy, ignoring the unused fallback reference for this case. A mutual variant behaved similarly. The saved pre-diagnostic native run has8/8 pixel failures; its first pair was directly inspected. Sources, compiler binary and all artifacts are bound by `variable-cycle-failure-receipt.json`.

The current compiler explicitly rejects cycles containing a fallback-reference edge, whether that fallback is traversed or unused. This is a compiler restriction pending browser-aligned investigation, not a runtime limitation or proof of impossibility. Direct cycles remain invalid values and can be recovered by a consuming var fallback. Separate rejected fixtures preserve both disputed and traversed forms; no previously failing fixture is presented as qualified by deleting it from the record.

S09/S10 remain partial. Browser invalid-at-computed-value recovery, fallback-cycle semantics, broader token/property contexts and custom registrations are not fully implemented. Existing text and fractional paint failures remain open.

Resource bounds:64 levels of token/dependency traversal,256 custom names per environment,64KiB per serialized value and1MiB total environment. Limits are diagnosed before unbounded expansion. A larger design input still obeys the existing module limits.
