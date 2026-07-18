# Nuxie distribution wiring for wgpu 30

This directory vendors the crates.io `wgpu` 30.0.0 package without Rust source
changes. Its normalized `Cargo.toml` points to Nuxie's vendored `wgpu-core`,
`wgpu-hal`, and platform feature-helper packages. Those relative dependencies
make the parity-critical Metal command-buffer coalescing transitive for both
workspace builds and downstream git/path consumers; it no longer relies on a
root-only Cargo `[patch]` table.

The four `wgpu-core-deps-*` directories are otherwise exact crates.io package
sources. Their normalized manifests point to the same vendored HAL so backend
features cannot accidentally enable a second registry copy. Every package
retains its upstream MIT and Apache-2.0 license files.

Provenance:

- Package: crates.io `wgpu` 30.0.0
- Original package checksum: `6d8f4bd44d92da5270f03409dba9f952dab24f128e05d6a554926101d1bf9114`
- Behavioral Rust source changes: none
- Distribution-manifest wiring SHA-256: `693f49693094a63d258bf151bb462f1345a37bd1720e828c427c79edc874791a`
- Direct-crate test lock SHA-256: `3f2d79fa13fcedee842d5ca987245d8e01025469bf119c193197b6236c8ccd48`

The wiring hash is the SHA-256 of the ordinary `shasum -a 256` output, in
this exact order, for `wgpu`, `wgpu-core`, `wgpu-core-deps-apple`,
`wgpu-core-deps-emscripten`, `wgpu-core-deps-wasm`, and
`wgpu-core-deps-windows-linux-android` `Cargo.toml` files at their repository
paths.

Run both `make renderer-wgpu-backend-check` and
`make renderer-wgpu-consumer-check` after changing this graph. The consumer
check resolves and compiles `nuxie-renderer` from outside the workspace and
rejects registry or duplicate copies of any vendored wgpu package.
