# Chromium 153 underline skip-ink scalar eligibility

The proposed predicate in `/tmp/html-skip-eligibility.rs` implements the scalar eligibility decision in pinned Chromium **153.0.8010.12**, rather than substituting Unicode scripts or current Chrome main. It contains **139 sorted, merged exclusion intervals**, covering **116,917 Unicode scalars**. This is a source-derived compatibility table; it does not claim pixel qualification of those characters or fonts.

## Exact rule

`Character::CanTextDecorationSkipInk(c)` returns false for the union of:

- U+002F SOLIDUS, U+005C REVERSE SOLIDUS, U+005F LOW LINE.
- `IsCjkIdeographOrSymbol(c)`: the explicit singleton/range lists in the pinned Blink header, plus `Emoji_Presentation`, plus Extended_Pictographic scalars participating in RGI emoji ZWJ or modifier sequences. The property generator has **no generic Han/Hiragana/Katakana script expansion**.
- Entire ICU blocks Hangul Jamo (1100–11FF), Hangul Compatibility Jamo (3130–318F), Hangul Syllables (AC00–D7AF), Hangul Jamo Extended A (A960–A97F), Hangul Jamo Extended B (D7B0–D7FF), and Linear B Ideograms (10080–100FF), including unassigned positions within those blocks.

All other Unicode scalars return true. The Rust API takes `char`, making surrogate and out-of-range inputs impossible. Use this predicate for the browser's automatic eligibility decision; this research does not establish how `skip-ink: all` should bypass or interact with it.

Some surprising but intentional pinned results:

| Scalars | Eligible |
| --- | --- |
| `/`, `\`, `_`, U+02C7 | No |
| U+302E–302F Hangul tone marks | Yes |
| U+3030 WAVY DASH | Yes |
| U+31F0 Katakana Phonetic Extensions start | Yes |
| U+FF0D, U+FF1B–FF1C, U+FF1E | Yes |
| U+FF1D FULLWIDTH EQUALS SIGN | No |
| U+1B130 Kana Extended A position | Yes |
| Entire U+20000–2FFFF Plane 2 | No |
| U+30000 Plane 3 start | Yes |

These are strong reasons not to substitute a broad CJK-script heuristic. Fullwidth glyphs or supplementary ideographs do not imply automatic exclusion outside the actual table.

## Reproduction and evidence

Run from repository root:

```sh
python3 tools/html-to-riv/validation/underline-skip-eligibility-reference.py
rustc --test /tmp/html-skip-eligibility.rs -o /tmp/html-skip-eligibility-test
/tmp/html-skip-eligibility-test
```

The generator downloads immutable tagged/revision-pinned sources, parses the explicit Blink arrays, and extracts `Emoji_Presentation` / `Extended_Pictographic` from pinned ICU `ppucd.txt` (Unicode 17.0). Block properties provide defaults for codepoint records; unassigned records reset binary properties before applying their own overrides. The two pinned sequence files provide RGI sequence membership. The resulting JSON stores source SHA-256 hashes, all merged intervals, and **481 boundary cases** at interval starts/ends and adjacent scalars. The standalone Rust proposal preserves Chromium's BSD license notice and tests every recorded boundary plus table ordering/disjointness.

Validation performed during research:

1. Parsed pinned ICU 78.2 data independently compared with installed ICU 78.3 `u_hasBinaryProperty` for both required binary properties across **all 1,114,112 code points: zero mismatches**.
2. A separate C++ reconstruction uses ICU `UnicodeSet` queries for Emoji_Presentation and the union of the two RGI sequence properties, ICU's Extended_Pictographic property, the original Blink arrays, and `ublock_getCode` for the six blocks. Compared with the generated exclusion table across **all 1,114,112 code points: zero mismatches**. `/tmp/underline-eligibility-icu.cc` and `/tmp/underline-eligibility-icu.bin` are the local evidence.
3. Standalone Rust tests: **2 passed**, covering 481 boundary decisions and interval ordering.

The installed ICU comparison is corroboration, not the source of the generated table: the authoritative inputs remain the ICU **78.2** revision pinned by Chromium's DEPS. No runtime or compiler source was changed by this research task. Native glyph/pixel fixtures are still required when integrating the drawing behavior.

## Pinned primary sources

- [Character::CanTextDecorationSkipInk](https://chromium.googlesource.com/chromium/src/+/153.0.8010.12/third_party/blink/renderer/platform/text/character.cc), lines 143–171.
- [Character fast path](https://chromium.googlesource.com/chromium/src/+/153.0.8010.12/third_party/blink/renderer/platform/text/character.h), `IsCjkIdeographOrSymbol` (below U+02C7 always false).
- [Explicit singleton/range lists](https://chromium.googlesource.com/chromium/src/+/153.0.8010.12/third_party/blink/renderer/platform/text/character_property_data.h).
- [Emoji property construction](https://chromium.googlesource.com/chromium/src/+/153.0.8010.12/third_party/blink/renderer/platform/text/character_property_data_generator.cc), `Initialize`, `SetIsCjkIdeographOrSymbolForEmoji`, `SetExtPictFromEmojiSequences`.
- [Chromium DEPS pin](https://chromium.googlesource.com/chromium/src/+/153.0.8010.12/DEPS): ICU revision `8cc91d9b6ab9991802fd208ee03a69714fd0251c`.
- [Pinned ICU version](https://chromium.googlesource.com/chromium/deps/icu/+/8cc91d9b6ab9991802fd208ee03a69714fd0251c/source/common/unicode/uvernum.h): 78.2.
- [Pinned Unicode property data](https://chromium.googlesource.com/chromium/deps/icu/+/8cc91d9b6ab9991802fd208ee03a69714fd0251c/source/data/unidata/ppucd.txt).
- [Pinned emoji sequences](https://chromium.googlesource.com/chromium/deps/icu/+/8cc91d9b6ab9991802fd208ee03a69714fd0251c/source/data/unidata/emoji-sequences.txt) and [ZWJ sequences](https://chromium.googlesource.com/chromium/deps/icu/+/8cc91d9b6ab9991802fd208ee03a69714fd0251c/source/data/unidata/emoji-zwj-sequences.txt).
