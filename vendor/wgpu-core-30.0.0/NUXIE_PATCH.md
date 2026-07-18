# Nuxie patch: Metal command-buffer and validation hot paths

This directory vendors crates.io `wgpu-core` 30.0.0. It is patched together
with `../wgpu-hal-30.0.0` to avoid two empty Metal command-buffer boundaries:
the body/pre-pass split for a render pass with no native pre-pass work, and a
transition-only command buffer when resource initialization emitted no clears.

Both command-buffer paths are deliberately conservative and require explicit
HAL opt-ins. Render-pass continuation rejects pending store-discard repair,
queries, indirect validation, and render bundles that could hide indirect
work. Its command classifier matches every `ArcRenderCommand` variant
exhaustively, and release assertions prevent new pre-pass work from being
silently recorded after a pass body. Transition-buffer discard additionally
requires that neither buffer nor texture initialization encoded a clear.

The patch also keeps ordinary attachment-overlap validation in an inline
`ArrayVec`, promoting without changing membership semantics when a view spans
more than `2 * MAX_COLOR_ATTACHMENTS` subresources. Color-attachment
bytes-per-sample validation is performed once at the original position of the
first active attachment. Duplicate detection, validation errors, and error
order remain unchanged.

The canonical source patch changes only:

- `src/command/memory_init.rs`
- `src/command/mod.rs`
- `src/command/query.rs`
- `src/command/render.rs`
- `src/device/queue.rs`
- `src/indirect_validation/draw.rs`

Upstream identity and review material:

- Package: crates.io `wgpu-core` 30.0.0
- Package checksum in the original workspace lock: `08763620e76fc980bca7bf84de82568614487a53172dd968d89187282eb87fa2`
- Canonical source patch SHA-256: `d73919c84bcf241e5ecece989bcd055eae3600d762ffab695bb25cc5ae8e95db`
- Companion HAL source patch SHA-256: `d4789859640f75213fd50c0b70526a3f2f3723a644e3da1ed2def52f91280c08`
- Direct-crate test lock SHA-256: `f57c034f1479e0fcc1257c094521091d3ebb99775a988902f8cf42dae083b7e0`

The behavioral source-patch hash is the SHA-256 of `git diff --full-index
--binary` against a Git snapshot of the exact unpacked crates.io package,
limited to the six source files listed above. `Cargo.toml` additionally points
to the vendored HAL and four platform feature-helper packages so downstream
git/path consumers cannot fall back to stock wgpu. That distribution-only
wiring is covered separately in `../wgpu-30.0.0/NUXIE_PATCH.md`. Cargo
extraction metadata, `NUXIE_PATCH.md`, the direct-crate `Cargo.lock`, and
`target/` are excluded; the lockfile is covered by its separate hash above.

Run `make renderer-wgpu-backend-check` after touching this code. Before a wgpu
upgrade, regenerate both patches against the exact new crates.io sources,
re-audit every render-command variant and ordered pre-pass producer, then run
the renderer golden corpus and exact R4 timing gate. Do not carry the fast path
forward from version number alone.
