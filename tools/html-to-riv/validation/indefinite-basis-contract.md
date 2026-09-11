# L11 compiler/runtime contract

Status: native-qualified against the immutable indefinite-v12-toolchain. Public233 tests, checked host13, native/WASM parity across1847 scenes, original/clone960 resize comparisons and full native5551 tests pass. All5541 full native scene pairs are visually accounted for. Vector text has88 preserved pixel failures; all vector evidence is reviewed. See indefinite-basis-qualification.md.

## Syntax and semantics

Extend the existing finite, nonnegative percentage `flex-basis` grammar and flex
shorthands to parents whose main size is indefinite. Keep the existing factor
range, cascade, variables, inheritance, and diagnostic rules. This includes `0%`,
which remains distinct from a definite `0px` basis.

An unresolved percentage basis contributes content sizing rather than falling
back to an authored main size as `auto` does. Preserve authored dimensions as
separate sizing inputs. Resolve percentages again when the containing main size
becomes definite; retain responsive behavior when resizing the same Rive bytes.
Auto-width text contributes its minimum and maximum intrinsic advances. Auto rows
fit the available width between intrinsic limits; wrapping changes the minimum
contribution because items can occupy separate lines. Layout measurements remain
fractional; CSS paint bounds use the separately validated snapping policy.

## Portable requirements

Use requirements version 12 with capability
`layout-css-indefinite-basis-v1` whenever a percentage basis depends on an
indefinite parent main size. The capability attests to the corrected runtime
layout and measurement semantics; it must not imply support for other CSS.

Keep the existing intrinsic-sizing and independent-factor occurrence payloads
where needed. Version 12 may contain no intrinsic targets for a box-only scene;
when intrinsic targets exist, their uniqueness and matching capability remain
mandatory. Version 11 retains its current intrinsic-target requirement. A child
requiring version 11 must never downgrade a scene already requiring version 12.
The version-12 capability and version must agree. Older hosts must reject the
manifest before producing any render stream. Unknown, missing, and inconsistent
capabilities remain errors.

No new per-node runtime toggle is proposed for the global layout corrections.
If validation shows a policy must vary by occurrence, revise this contract and
test that transport explicitly before admission.

## Qualification requirements (native evidence complete)

- Public acceptance and cascade equivalence, including equal/zero factors,
  explicit main dimensions, direct/nested text, min/max and reverse/wrapped axes.
- Host rejection of old/stripped capabilities and invalid versions or payloads.
- Native/WASM equality of bytes, source maps, and portable requirements.
- Original and cloned instances resized repeatedly using unchanged compiled bytes.
- Chromium geometry and real native/vector pixels at 240, 390, and 768px;
  complete direct review or documented image-equivalence transfers.
- Full regression against the final immutable compiler/runtime toolchain.

CSS Grid, content-box sizing, percentage padding/margins, negative margins,
positioning, new overflow rules, editor integration, scripts, interactions,
bindings, and animation receive no support from L11. Existing syntax rejections
in the exploratory corpora remain explicit negative tests.

Foundation evidence: focused percentage/content-auto Rust tests pass; checked host13/13 passes including missing-capability/version rejection before render stream. TypeScript declarations pass. Admission, compiler emission, native/WASM parity and full qualification are still pending.

Admission evidence supersedes the earlier foundation-only note: emission is implemented, focused public49/49, actual compiled-scene host13/13 and native/WASM parity9/9 pass. Original/clone120-scene lifecycle passes960 instance/viewport comparisons. Full5551 native and full public regression are running; complete visual review remains pending.
