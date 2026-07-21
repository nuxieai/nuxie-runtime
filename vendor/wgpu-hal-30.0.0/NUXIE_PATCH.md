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

Upstream identity and review material:

- Package: crates.io `wgpu-hal` 30.0.0
- Package checksum in the original workspace lock: `cf765132d8d5f50e192e7880464890c13f4e7457aafe8e5466e8174586e9f101`
- Canonical source patch SHA-256: `b6d2a27aa6fabe80bf02a0c3744819629894202ae7081a9035b6a49a3d3b0745`
- Companion core source patch SHA-256: `d73919c84bcf241e5ecece989bcd055eae3600d762ffab695bb25cc5ae8e95db`
- Direct-crate test lock SHA-256: `bc27e50dd420d2dd78fdce4000b28fb8492fb07cda4da37c4fd488f0829a4476`

The source-patch hash is the SHA-256 of `git diff --full-index --binary`
against a Git snapshot of the exact unpacked crates.io package. The overlay
excludes Cargo extraction metadata, `NUXIE_PATCH.md`, the direct-crate
`Cargo.lock`, and `target/`; the lockfile is covered by its separate hash above.

Run `make renderer-wgpu-backend-check` after touching this code. A wgpu upgrade
must re-establish the exact Metal transition, encoder-lifecycle, empty-buffer,
and strict-sync invariants before retaining either capability override.
