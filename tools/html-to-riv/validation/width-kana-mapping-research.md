# Width and kana mapping research

Researched 2026-09-08. This is mapping evidence, not a browser/native rendering qualification receipt. Unicode data is pinned to 17.0.0; CSS and Gecko links describe inspected current sources, not a pinned installed browser.

## Defined transform pipeline

The grammar permits one casing operation, `full-width`, and `full-size-kana`, in any written order; `none` stands alone. Execution order is casing, width, then kana size. Width mapping takes Unicode `<narrow>` decompositions forward and `<wide>` decompositions backward. Kana size uses the normative Appendix G table, including supplementary-plane small kana and halfwidth small kana. Consequently `full-size-kana` alone preserves halfwidth form (`ｧ` → `ｱ`); adding `full-width` produces `ア`. Transformations occur between whitespace processing phases I and II. [CSS Text 3, mapping rules and order](https://drafts.csswg.org/css-text-3/#text-transform-property), [Appendix G](https://drafts.csswg.org/css-text-3/#small-kana)

Do not equate the phrase “preserved white space” here with only the `pre`/`pre-wrap` property values. WPT explicitly expects three ordinary collapsing spaces to become one U+3000 after phase I, while three `pre-wrap` spaces become three U+3000. Thus skipping width transformation of the surviving ordinary space would disagree with the tests. [Collapsed-space test](https://raw.githubusercontent.com/web-platform-tests/wpt/master/css/css-text/text-transform/text-transform-fullwidth-006.html), [preserved-space test](https://raw.githubusercontent.com/web-platform-tests/wpt/master/css/css-text/text-transform/text-transform-fullwidth-007.html)

WPT also expects transformed U+3000 to follow natural ideographic-space hanging rules: at soft line ends under `normal`, and soft/forced boundaries under `pre-wrap`. Test narrow widths and right alignment; an early ASCII trim or ordinary-space advance removal can hide a semantic error. [Normal trailing-space test](https://raw.githubusercontent.com/web-platform-tests/wpt/master/css/css-text/text-transform/text-transform-fullwidth-008.html), [pre-wrap trailing-space test](https://raw.githubusercontent.com/web-platform-tests/wpt/master/css/css-text/text-transform/text-transform-fullwidth-009.html)

## Exact Unicode mappings and composition

Inspection of Unicode 17.0.0 finds **104 `<wide>` and 122 `<narrow>` entries**, all with one scalar in the decomposition. Examples below are data-derived, not font substitutions. [Pinned UnicodeData.txt](https://www.unicode.org/Public/17.0.0/ucd/UnicodeData.txt)

| Input | Full-width output |
| --- | --- |
| U+0020 SPACE | U+3000 IDEOGRAPHIC SPACE |
| ASCII `!` through `~` | U+FF01 through U+FF5E |
| U+00AF MACRON | U+FFE3 FULLWIDTH MACRON |
| U+FF76 HALFWIDTH KA | U+30AB KATAKANA KA |
| U+FF9E voiced mark | U+3099 COMBINING VOICED MARK |
| U+FF9F semi-voiced mark | U+309A COMBINING SEMI-VOICED MARK |
| U+FFA0 HALFWIDTH HANGUL FILLER | U+3164 HANGUL FILLER |
| U+FFA1 HALFWIDTH KIYEOK | U+3131 HANGUL KIYEOK |

Derived implication: `ｶﾞ` → `カ\u3099` and `ﾊﾟ` → `ハ\u309A`, retaining two scalars. Do **not** apply NFC/NFKC merely to make them precomposed `ガ`/`パ`; the mapping operation itself does not request that normalization. TAB, LF and NBSP have no width-tag mapping. Other compatibility decompositions, such as ligatures, are outside width conversion.

Gecko independently corroborates scalar mapping: after casing it calls `unicode::GetFullWidth(ch)`, then looks up small kana. `GetFullWidth` is a BMP table lookup returning one scalar; this pass does not compose neighboring voiced marks. Its kana table includes the supplementary small characters, including U+1B132 → U+3053 and U+1B155 → U+30B3. [Gecko transformation loop](https://github.com/mozilla/gecko-dev/blob/master/layout/generic/nsTextRunTransformations.cpp#L782-L838), [width lookup](https://github.com/mozilla/gecko-dev/blob/master/intl/unicharutil/util/nsUnicodeProperties.cpp#L144-L157)

## Compiler implications and difficult cases

These are implementation deductions from the mappings, not additional standards requirements:

- Width and kana stages preserve **scalar** boundary counts, even supplementary-to-BMP kana. UTF-16 offsets can shrink, so never substitute browser UTF-16 positions for public scalar positions.
- Compose their identity scalar maps after the existing potentially expanding/deleting casing map. `ß` plus uppercase/full-width becomes two fullwidth S scalars; casing remains responsible for that expansion.
- Keep combining marks as source-backed scalars even if shaping merges the visual cluster. Test standalone, repeated and non-composable voiced marks as well as valid pairs.
- Cover halfwidth Hangul and symbols, unchanged NBSP/TAB/LF, retained versus collapsed spaces, a forced break after transformed spaces, and width-sensitive wrapping/alignment.
- Compare all Appendix G pairs logically; font-supported rendering cases cannot establish supplementary character coverage by themselves.
- Do not count an unsupported browser declaration as a passing visual oracle. Record `CSS.supports`, computed style and actual logical transformation before choosing that browser as evidence.


## Qualification target clarification

Chrome remains authoritative following the user's question about Firefox.
This investigation is supplemental; it does not change the compiler's target
or qualify width/kana support. Those operations remain deferred and rejected
until their Chrome qualification can be established. A20 underline is the next
independent implementation task.
