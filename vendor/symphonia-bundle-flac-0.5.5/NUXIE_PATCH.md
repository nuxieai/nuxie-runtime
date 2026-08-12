# Nuxie patch

Crates.io `symphonia-bundle-flac` 0.5.5, checksum
`c91565e180aea25d9b80a910c546802526ffd0072d0b8974e3ebe59b686c9976`.

Adds a default-on `metadata` feature. With it disabled, Vorbis-comment and
picture blocks are skipped while stream info, seek tables, cues, packets, and
sample decoding retain upstream behavior. See `docs/symphonia-fork.md`.
