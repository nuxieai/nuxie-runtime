# Metadata-free Symphonia fork

The shipping runtime uses the Symphonia 0.5.5 FLAC, MP3, PCM, and WAV
demuxers/decoders, but it does not expose container tags or artwork. The
crates.io facade and three component paths otherwise pull
`symphonia-metadata`, whose ID3 charset handling pulls `encoding_rs` into every
Apple slice.

Nuxie therefore depends on the component crates directly and patches the four
components below. Their default feature profiles preserve upstream metadata
behavior for other consumers. `nuxie-audio` selects `default-features = false`
for the vendored components, which makes FLAC comments/pictures and WAV INFO
lists skip their declared byte ranges instead of decoding text. The MP3 bundle's
unused metadata dependency is removed. A small bounds-checked ID3v2 header
walker in `nuxie-audio` advances tagged MP3 streams to the first MPEG frame;
audio bytes remain owned and unchanged.

| Package | Original crates.io checksum | Patch |
| --- | --- | --- |
| `symphonia-bundle-flac` 0.5.5 | `c91565e180aea25d9b80a910c546802526ffd0072d0b8974e3ebe59b686c9976` | optional metadata parsing; skip comment/picture blocks when disabled |
| `symphonia-bundle-mp3` 0.5.5 | `4872dd6bb56bf5eac799e3e957aa1981086c3e613b27e0ac23b176054f7c57ed` | remove unused metadata dependency |
| `symphonia-format-riff` 0.5.5 | `c2d7c3df0e7d94efb68401d81906eae73c02b40d5ec1a141962c592d0f11a96f` | optional WAV INFO parsing; skip LIST bytes when disabled |
| `symphonia-utils-xiph` 0.5.5 | `ee27c85ab799a338446b68eec77abf42e1a6f1bb490656e121c6e27bfbab9f16` | feature-gate metadata parser re-exports |

Acceptance is the pinned audio parity fixture suite plus the Apple and size
report dependency ratchets, which reject both `symphonia-metadata` and
`encoding_rs` as normal target dependencies.
