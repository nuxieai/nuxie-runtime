# Nuxie patch

Crates.io `symphonia-format-riff` 0.5.5, checksum
`c2d7c3df0e7d94efb68401d81906eae73c02b40d5ec1a141962c592d0f11a96f`.

Adds a default-on `metadata` feature. With it disabled, WAV INFO lists are
skipped while format/data/fact parsing and PCM packetization retain upstream
behavior. See `docs/symphonia-fork.md`.
