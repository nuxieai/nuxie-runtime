# Test assets

`Inter-Regular.ttf` is the static Inter 18pt regular font copied from
`tests/unit_tests/assets/fonts/Inter_18pt-Regular.ttf` in the local upstream Rive
runtime checkout at `9ed5b516`. It is distributed under the SIL Open Font
License; see `Inter-LICENSE.txt`, retrieved from
https://github.com/rsms/inter/blob/master/LICENSE.txt.

SHA-256: `3e5f90a0138b38de4cf4d779ad78391974ea1df776b9164842bdcbb60ce383c5`.
The exact same bytes are embedded in Rive and served to Chromium via @font-face.

`quadrants.png` is an original 32×32 RGBA test pattern generated for this module.
It contains four solid quadrants and has no color-management metadata or
external attribution requirements. It is covered by the repository license.

`OpenSans-Regular.ttf` is the static regular fixture copied without modification
from the upstream checkout's
`dependencies/rive-app_harfbuzz_rive_13.1.1/test/api/fonts/OpenSans-Regular.ttf`.
Its embedded name table credits Google Corporation, 2010–2011, and declares
Apache License 2.0; see `OpenSans-LICENSE.txt` from
https://www.apache.org/licenses/LICENSE-2.0.txt. This historical font's embedded
license differs from newer Open Sans releases.

SHA-256: `e64e508b2aa2880f907e470c4550980ec4c0694d103a43f36150ac3f93189bee`.
Identical bytes are embedded in Rive and served to Chromium as Open Sans Fixture.
Unlike the Inter fixture, it forms optional fi/ffi/fl ligatures, making the
letter-spacing qualification sensitive to optional-ligature suppression.

`NotoSansOgham-Regular.ttf` is an unmodified static Google Fonts fixture from
https://raw.githubusercontent.com/google/fonts/baa2e5561af8a4873b058859dcfe158bdd033942/ofl/notosansogham/NotoSansOgham-Regular.ttf.
It uses SIL Open Font License 1.1; see `NotoSansOgham-LICENSE.txt` from the same
revision. SHA-256: `6a70572d381f3a54fdaa3b373b865f4d97377c65434493b294b1dfacc7c4f58c`.
Identical bytes are embedded in Rive and served to Chromium as Ogham Fixture.
This supplies the visible U+1680 mark that Inter lacks, without browser fallback.


`NuxieJapaneseFixture-Regular.otf` is a renamed subset of static
NotoSansCJKjp-Regular.otf from Noto Sans CJK 2.004, revision
523d033d6cb47f4a80c58a35753646f5c3608a78. It retains glyphs in U+0000–052F,
U+2000–30FF, U+31F0–31FF, U+FB00–FB06, U+FF00–FFEF and U+1B000–1B16F where present in
that font, with fontTools layout closure. This is a test fixture, not a claim
that all those codepoints exist. Font family/PostScript names are changed to
Nuxie Japanese Fixture; original copyright/license records are retained.

Original: https://raw.githubusercontent.com/notofonts/noto-cjk/523d033d6cb47f4a80c58a35753646f5c3608a78/Sans/OTF/Japanese/NotoSansCJKjp-Regular.otf
License: https://raw.githubusercontent.com/notofonts/noto-cjk/523d033d6cb47f4a80c58a35753646f5c3608a78/LICENSE
See NuxieJapaneseFixture-LICENSE.txt (SIL OFL 1.1).
Original SHA-256: `68a3fc98800b2a27b371f2fb79991daf3633bd89309d4ffaa6946fd587f375b5`.
Subset SHA-256: `c1b4f9e2fbf8cfb1a26cf051bf4a356a1bbb3027bedef91c1ebea4bb7b4d5132`.
Reproduce with fonttools==4.61.1 and
`python validation/build-japanese-fixture.py /path/to/NotoSansCJKjp-Regular.otf`.
