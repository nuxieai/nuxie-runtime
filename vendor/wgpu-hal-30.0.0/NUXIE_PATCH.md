# Nuxie patch: explicit Metal command-buffer capabilities

This directory vendors crates.io `wgpu-hal` 30.0.0. It adds a default-false
render-pass continuation capability and a default-false transition-only
command-buffer discard capability for the companion patched `wgpu-core` crate.
Only the pinned Metal encoder opts in, because its buffer and texture
transitions are native no-ops. Dynamic dispatch forwards both concrete backend
answers; every other backend retains stock behavior.

The Metal shader compiler also mirrors Dawn/Tint's invariance contract:
`preserveInvariance` is enabled only when Naga emitted an invariant position.
Enabling it for ordinary positions changes 4x-MSAA edge coverage on Apple
Paravirtual devices.

The `apple-msl-capture` feature is tooling-only. When it is explicitly enabled
and `NUXIE_APPLE_MSL_CAPTURE_DIR` is set, the Metal pipeline path records the
exact layout-derived Naga-to-MSL inputs and outputs used by the repository's
offline shader catalog. Normal runtime builds do not enable the feature; a
feature-enabled build is still inert when the environment variable is absent.

The `apple-msl-replay` feature is likewise test-only. When both replay
environment paths are set, it resolves the exact layout-derived compiler key
against the committed schema-2 catalog, verifies the content-keyed MSL path,
source digest, language version, and every reflection field consumed by the
HAL, then substitutes the committed MSL. It fails closed and records a hit
only after all checks pass. Production builds do not enable this feature.

Metal opts out of both capabilities after strict event sync is enabled, because
continuing an older native command buffer or discarding a new one would bypass
the relay wait prologue. Nuxie never calls `enable_strict_event_sync`; callers
that add such a call must configure it before command recording and must not
race it with encoding.

The canonical source patch changes only:

- `src/dynamic/command.rs`
- `src/lib.rs`
- `src/metal/command.rs`
- `src/metal/device.rs`
- `src/metal/mod.rs`
- `src/metal/shader_capture.rs`
- `src/metal/shader_replay.rs`
- `src/metal/shader_translation.rs`

The normalized `Cargo.toml` and preserved `Cargo.toml.orig` add the tooling-only
`apple-msl-capture` and `apple-msl-replay` features plus their optional
`serde`, `serde_json`, and `sha2` dependencies. The normalized manifest also
retains the empty `[workspace]` table that stops Cargo from walking into an
enclosing checkout; see
`../wgpu-30.0.0/NUXIE_PATCH.md`. These manifest-only changes and Cargo
extraction metadata are outside the source-patch hash below and are covered by
the committed manifest and direct-lock review instead.

Upstream identity and review material:

- Package: crates.io `wgpu-hal` 30.0.0
- Package checksum in the original workspace lock: `cf765132d8d5f50e192e7880464890c13f4e7457aafe8e5466e8174586e9f101`
- Canonical source patch SHA-256: `13595289b3b70bc3eaa440fdb4afd4aefa4e4ffcde0be290446d2df6871559bb`
- Companion core source patch SHA-256: `d73919c84bcf241e5ecece989bcd055eae3600d762ffab695bb25cc5ae8e95db`
- Direct-crate test lock SHA-256: `e1ee3eb0e8c7fbe3121021e867bc7ac5f9291a98cc4bda7b19af8ccdf20e4d15`

The source-patch hash is the SHA-256 of `git diff --full-index --binary`
against a Git snapshot of the exact unpacked crates.io package. The overlay
excludes Cargo extraction metadata, `NUXIE_PATCH.md`, the direct-crate
`Cargo.lock`, and `target/`; the lockfile is covered by its separate hash above.

Run `make renderer-wgpu-backend-check` after touching this code. A wgpu upgrade
must re-establish the exact Metal transition, encoder-lifecycle, empty-buffer,
and strict-sync invariants before retaining either capability override.
