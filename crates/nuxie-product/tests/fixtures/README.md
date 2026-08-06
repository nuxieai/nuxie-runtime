# Text authoring font fixture

`roboto-a.ttf.base64` is a test-only subset of Roboto Regular 2.137 containing
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
openssl base64 -in roboto-a.ttf > roboto-a.ttf.base64
```

The decoded subset SHA-256 is
`b481b059ee94961c7b18585a596935aaa7cc44b68879c096d2cd06922e0431b1`.
Roboto is distributed under Apache-2.0; the license copied from the source
artifact is in `LICENSE-ROBOTO.txt`.
