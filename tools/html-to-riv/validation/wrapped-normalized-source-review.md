# Fixed-layout source and normalized bridge review

Read-only review of current sources while separate builds/captures run. No product source, build configuration, dependency or existing capture was edited. The review includes three executions of the already frozen validation constructor in a separate output directory; those do not import a runtime scene or render pixels. No new native qualification is claimed.

## Confirmed conversion scope

`fixed_layout::convert` performs one actual f32 multiplication by64, then explicitly saturates to int32 bounds or truncates toward zero in the interior. This matches the pinned LayoutUnit float constructor (`layout_unit.h125–130`) and its saturated raw conversion. Comparing the scaled value in f64 against exact int32 bounds avoids incorrectly converting i32::MAX to the f32 value2147483648 before comparison. Finite multiplication overflow is handled by the saturation branches.

The output check computes exact `raw / 64` in f64 and refuses a nonexact ordinary f32 representation. Positive saturation to2147483647 is therefore intentionally `NotRepresentable`; negative saturation to−2147483648 is exact. The last accepted positive f32 immediately below33554432 maps to2147483520 raw units. These are encoding boundaries, not changes to public scene limits. Nonfinite input rejects rather than emulating Chrome's NaN-to-zero constructor case; callers already promise finite inputs. Integer zero emits+0 while the original−0 computed bits remain in the descriptor.

`FixedLayoutLength` retains the original ScalarProvenance object and stores computed bits, derived raw units and emitted bits separately. `validate()` proves internal consistency with that retained native float; it does not prove that the upstream CSS parser/unit resolver produced the correct float. The typed domain path additionally checks authored nonnegativity and emitted wire bits for dimensions. Negative saturation therefore cannot turn a negative authored dimension into an admitted positive one. Percent coefficients are retained and continue through the previous native percent-resolution path; no Chrome percentage-result quantization is claimed.

The normalized bridge converts parent, slot and visible preferred/min/max fields before constructing records. The domain join requires descriptors for every slot and visible owner, and Candidate retains owned normalized descriptors through `layout_authored()`. Serialization reads the owned Candidate; no records are modified after Derived construction. Auto maximum omission remains explicit. No concrete defect was found in the finite-f32 Length→LayoutUnit conversion itself.

## Concrete bridge source-premise issue

At the reviewed bridge revision, `numeric()` checks `source.parse::<f32>()` directly against the serde-deserialized `value:f32`. The pinned Chrome computed-length path is different:

- `CSSNumericLiteralValue` accepts/stores a double (`css_numeric_literal_value.cc26–27`); ComputeLengthPx returns double and calls ZoomedComputedPixels at131–134.
- The px resolver returns the double value multiplied by zoom (`css_length_resolver.cc153–159`).
- `CSSPrimitiveValue::ComputeLength<Length>` calls ClampToCSSLengthRange on that double before constructing Fixed (`css_primitive_value.cc328–332`). The clamp explicitly returns float (`60–65`).
- Fixed layout then calls `LayoutUnit(length.Pixels())` (`length_functions.h53–63`).

Thus the audited zoom1 computed chain contains f64→f32 before quantization. Direct decimal→f32 can differ through double rounding. The exact decimal

`0.9999999701976775845491118843710864894092082977294921875`

is `1 - 2^-25 - 2^-55`. Exact nearest-f32 selection gives0x3f7fffff, while its nearest f64 lies on the f32 midpoint and conversion gives0x3f800000. The resulting fixed layout lengths are0.984375 and1 respectively. This is a discriminating source-premise case, not merely a provenance formatting difference.

The frozen constructor uses serde_json1.0.151 with default/std features and no float_roundtrip feature (recorded compiler fingerprint). Its ordinary f32 deserializer uses the f64 number path; the feature-gated single-precision parser is not active (`serde_json/src/de.rs346–352,623–648,1509–1513`). Three small executions confirm its behavior for the **same long JSON numeric token**:

| Source string | Frozen bridge result |
| --- | --- |
| Original adversarial decimal above | Rejects computed-f32 mismatch |
| `1` | Constructs successfully |
| Exact lower-f32 decimal `0.999999940395355224609375` | Rejects computed-f32 mismatch |

This demonstrates that the JSON value becomes1.0 but the reviewed bridge's direct source parse chooses the lower neighbor. The issue presently causes a private source-check rejection; it is not evidence that any current admitted public compiler accepts wrong CSS. Arbitrary original CSS decimal equivalence was not established by the bridge check.

## Concrete repair after the current frozen capture

Keep the current constructor and captures intact. In the next bridge revision, parse the source as f64, require finiteness, perform the explicit f32 conversion, require the resulting float finite, then compare its bits with the supplied `value`. Pass the **unchanged original decimal string** and the checked computed float into ScalarProvenance. Add this long decimal and its opposite-side neighbor as constructor controls, alongside source/value mismatch controls. Do not fix the discrepancy by rewriting the source string to `1` or pretending a computed value is the authored exact value.

The JSON number parser itself is not a CSS parser and may reject or round other extreme lexical forms differently. For this private authored-recipe constructor, a disagreement should fail explicitly; a future public path must bind the actual CSS parser/resolver output and audit its lexical conversion, unit arithmetic, clamping and zoom. No dependency-feature override is proposed. The fixed_layout module's caller precondition already leaves that upstream binding distinct from its checked conversion.

Primary links: [numeric literal double computation](https://chromium.googlesource.com/chromium/src/+/971a7443b0c9b0a9b2860529b33331b76077ec62/third_party/blink/renderer/core/css/css_numeric_literal_value.cc#131), [px resolver](https://chromium.googlesource.com/chromium/src/+/971a7443b0c9b0a9b2860529b33331b76077ec62/third_party/blink/renderer/core/css/css_length_resolver.cc#153), [computed Length conversion](https://chromium.googlesource.com/chromium/src/+/971a7443b0c9b0a9b2860529b33331b76077ec62/third_party/blink/renderer/core/css/css_primitive_value.cc#328). Exact downloaded-file hashes are in the existing quantization-source manifest and wrapped-boundary-quantization-review.md.

## Bound reviewed state

Constructor execution receipt: [receipt.json](/Users/levi/.codex/worktrees/html-css-immutable/tools/html-to-riv/output/wrapped-normalized-source-review-r1/receipt.json), SHA-256 `afc4ed21def1b2af84278b6cfd4cd380789c27ad637a47b2498db82323b62daf`. It binds exact input recipes, logs and frozen binary identity.

| File | SHA-256 |
| --- | --- |
| `tools/html-to-riv/src/fixed_layout.rs` | `d28393ed229530187ce7508003350ce7348fef6a0d25f1b2fc3cb8c409a4cca5` |
| `tools/html-to-riv/src/wrapping_sizes.rs` | `7945ca9c4cb327b3fb812175e7a97656caf1bfe5dc1ad390c5ac1e13d758155b` |
| `tools/html-to-riv/src/wrapping_domains.rs` | `cc537a4abf008f84ea15a1edfbf9ad43dff84a63ec0acead874912ec1949cbbe` |
| `tools/html-to-riv/src/wrapping_composition.rs` | `928cbe9c3d0dd18b3b6037eeb37e7a6adc69f622dcb706fc5f0ebf0073ad0009` |
| `tools/html-to-riv/validation/wrapped-normalized-bridge.rs` | `147cdf1850e087fdd178d7f0d37806dccc01df9792a0059c154ee905017134b5` |

## Corrected r2 constructor confirmation

After the r1 capture was preserved, root changed the validation bridge to parse the original source through f64, check finiteness, convert to f32 and compare computed bits. The corrected bridge SHA-256 is `147cdf1850e087fdd178d7f0d37806dccc01df9792a0059c154ee905017134b5`; frozen r2 constructor SHA-256 is `f52f2f46e59a1331c95c43be6f7389979bab3838ceac7bd7fe599dcac247aea4`.

Four source controls were executed against that already-built constructor, without runtime import or rendering:

- Original long source plus the same long JSON value now constructs successfully.
- Source `1` plus that JSON value still constructs successfully.
- Exact lower-f32 source plus that JSON value rejects the mismatched computed float.
- Original long source plus the exact lower-f32 JSON value also rejects.

The accepted original-source proof retains the original decimal digits and an ideal enclosure below1, while recording computed bits0x3f800000, raw units64 and emitted bits0x3f800000. Thus the repair fixes the conversion premise without rewriting the authored value as exact1. The original r1 rejection receipt remains unchanged. [r2 control receipt](/Users/levi/.codex/worktrees/html-css-immutable/tools/html-to-riv/output/wrapped-normalized-source-review-r2/receipt.json) SHA-256: `6d9598ba27d792b9761769baf1d185baea0a5f96da01d7502af8bff6a6365f26`. Its artifact list includes accepted proof/scene files and all exact recipe/log bytes. This closes the demonstrated validation-bridge mismatch; it does not qualify arbitrary CSS parser input or new native pixels.
