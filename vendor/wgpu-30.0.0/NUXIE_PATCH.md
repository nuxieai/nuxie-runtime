# Nuxie distribution wiring for wgpu 30

This directory vendors the crates.io `wgpu` 30.0.0 package with Nuxie's
downstream BrowserWebGpu compatibility patches. Its normalized `Cargo.toml`
points to Nuxie's vendored `wgpu-core`, `wgpu-hal`, and platform feature-helper
packages. Those relative dependencies make the parity-critical Metal
command-buffer coalescing transitive for both workspace builds and downstream
git/path consumers; it no longer relies on a root-only Cargo `[patch]` table.
The native `wgpu-core` dependency also leaves WGSL input support to wgpu's
public `wgsl` feature instead of enabling it unconditionally. Default builds
still enable `wgsl`; `default-features = false` Metal consumers can now omit
Naga's `wgsl-in` parser when they use only trusted passthrough MSL and disable
wgpu's internal WGSL helper pipelines.

The four `wgpu-core-deps-*` directories are otherwise exact crates.io package
sources. Their normalized manifests point to the same vendored HAL so backend
features cannot accidentally enable a second registry copy. Every package
retains its upstream MIT and Apache-2.0 license files.

Every vendored wgpu manifest — these six plus `../wgpu-hal-30.0.0` — also ends
in an empty `[workspace]` table. These packages are excluded from the root
workspace and built on their own via `--manifest-path` (see
`renderer-wgpu-backend-check`), so cargo otherwise searches upward for a
workspace root. From a git worktree rooted inside the main checkout
(`.claude/worktrees/<name>`) that search walks past the worktree's own root,
which excludes them, into the parent checkout's `Cargo.toml`, which does not —
and cargo then rejects the mismatch, breaking `cargo fmt --all` and every
`--manifest-path` check run from a worktree. The empty table terminates the
search at the package itself, which is what each of these already was.

Provenance:

- Package: crates.io `wgpu` 30.0.0
- Original package checksum: `6d8f4bd44d92da5270f03409dba9f952dab24f128e05d6a554926101d1bf9114`
- Additional behavioral source change in this slice: `future_pop_error_scope`
  treats the clean JavaScript `null` result as no error before converting
  present `GpuError` values. Rejected promises and real errors retain their
  upstream behavior.
- Native feature-wiring change: the target-specific `wgpu-core` dependency no
  longer force-enables `wgsl`. The existing public `wgsl` feature remains the
  single owner of `wgpu-core/wgsl`, and it remains part of wgpu's defaults.
- Upstream issue: https://github.com/wasm-bindgen/wasm-bindgen/issues/5234
- Distribution-manifest wiring SHA-256: `632166f561bda7ca790e97f5e28ccc8abefcca61d318fb686e56ba6f7faa79a5`
  (was `693f49693094a63d258bf151bb462f1345a37bd1720e828c427c79edc874791a`
  before the empty `[workspace]` tables described above)
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
