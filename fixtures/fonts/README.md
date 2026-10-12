# Neutral baseline font fixture

`roboto-a.ttf` is a test-only subset of Roboto Regular 2.137 containing
U+0061 (`a`). The source font came from the published
`com.formdev:flatlaf-fonts-roboto:2.137` Maven artifact:

`https://repo1.maven.org/maven2/com/formdev/flatlaf-fonts-roboto/2.137/flatlaf-fonts-roboto-2.137.jar`

Source checksums:

- artifact: `f46e549e8b553ca670c1ad2b26631ba76d20fd39efcd66dc79ed883fbcf5843e`
- `Roboto-Regular.ttf`: `4e147ab64b9fdf6d89d01f6b8c3ca0b3cddc59d608a8e2218f9a2504b5c98e14`

Reproduce the fixture with HarfBuzz and OpenSSL:

```sh
unzip -j flatlaf-fonts-roboto-2.137.jar \
  com/formdev/flatlaf/fonts/roboto/Roboto-Regular.ttf
hb-subset Roboto-Regular.ttf \
  --unicodes=U+0061 \
  --output-file=roboto-a.ttf
```

The decoded subset SHA-256 is
`b481b059ee94961c7b18585a596935aaa7cc44b68879c096d2cd06922e0431b1`.
Roboto is distributed under Apache-2.0; the license copied from the source
artifact is in `LICENSE-ROBOTO.txt`.

## Native input geometry fixture

`roboto-input.ttf` uses the same verified source font and license. It retains
Basic Latin, space, the zero-width-space codepoint when available, and the bullet
used by upstream secure TextInput. Unlike the deliberately single-glyph fixture,
it can shape an empty input's sentinel and masked text.

Generated with HarfBuzz 14.2.1:

```sh
hb-subset Roboto-Regular.ttf \
  --unicodes=U+0020-007E,U+200B,U+2022 \
  --output-file=roboto-input.ttf
```

SHA-256: `ea879cc16b566f2b07ddc73e2576080bf0ebe2c10198c561caac5cf3c7f3fdea`.
The source and subset retain `unitsPerEm=2048`, hhea ascent `1900`, and hhea
descent `-500`. The native geometry test uses these font-table values as an
independent baseline oracle (24 px font, top origin), rather than asking the
runtime to calculate its own expected result. This is not a replacement for the
open single-glyph-font empty-input regression.

## Native fallback fixture

`ibm-plex-sans-arabic.ttf` is the unmodified IBM Plex Sans Arabic Regular face
used by upstream Rive's fallback font tests. It is copied from
`tests/unit_tests/assets/fonts/IBMPlexSansArabic-Regular.ttf` at Rive pin
`3ccf0d9d7c2cf8bee0553245acaffe21ae3314b6`. Byte length: 226004;
SHA-256: `1118a402b5eabda347c416e49391bdc156836abfc0aca93ae9f1fbddfab2af28`. No subsetting or font-table changes.

It supplies the Arabic letters in `مرحبا` while the Latin-only Roboto input
fixture deliberately does not. The native regression uses this local file,
without an external upstream checkout or network dependency.

The font metadata identifies Copyright2019 IBM Corp. and SIL OFL1.1.
The [IBM license](https://raw.githubusercontent.com/IBM/plex/master/LICENSE.txt) is retained in
`LICENSE-IBM-PLEX.txt` (SHA-256 `7e6b2818edbd8f6a01ae80641cc8f16a51080d08fb4e532be3a0b6f74adb07da`).
This public test font is separate from the private platform reference fonts,
which are never committed or bundled.
