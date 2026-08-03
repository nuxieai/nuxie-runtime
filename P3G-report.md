# P3G factory/renderer seam report

## Status

- Ported against pinned `rive-runtime` commit `d788e8ec6e8b598526607d6a1e8818e8b637b60c`.
- `src/factory.cpp` / B6-0207 is a faithful candidate with `pending-verification`.
- `src/renderer.cpp` / B6-0311 is a faithful candidate with `pending-verification`.
- The generated parity scorecard moves from 423 faithful / 21 pending to 425 faithful / 19 pending. The remaining misc-core pending rows are `src/core.cpp` and `src/focus_data.cpp`.
- No production file was created, so no new four-place residue entry or frame-loop file row is required. The existing render-api crate-root owner is attributed by both mechanical correspondence ledgers and was removed from `rust-additions.toml`.

## Evidence

### Factory

- `src/factory.cpp:15-20` → `Factory::make_render_path_from_aabb`, including clockwise rectangle construction and `FillRule::NonZero`; focused test: `p3g_factory_aabb_helper_builds_the_pinned_nonzero_rectangle_path`.
- `src/factory.cpp:22-29` → byte-owning HarfRust validation in `Factory::decode_font`; focused tests: `p3g_factory_font_helper_validates_and_owns_the_encoded_font` and `p3g_font_asset_decode_routes_through_the_factory_helper`.
- Loader-driven and pre-renderer RuntimeFile font installation both execute `Factory::decode_font`; the portable path uses `NullFactory` because pinned `decodeFont` is nonvirtual.
- `src/factory.cpp:31-40` → the pre-existing `Factory::decode_audio`; focused evidence remains `factory_decode_audio_owns_and_decodes_the_pinned_wav`.

### Renderer

- `src/renderer.cpp:7-70` → standalone `compute_alignment`, with an origin/size adapter used by nested-artboard alignment to avoid a lossy max-coordinate reconstruction; all eight pinned Fit cases are covered by `p3g_compute_alignment_matches_every_pinned_fit_case`.
- `src/renderer.cpp:72-88` → exact Renderer trait defaults for translate/scale/rotate; focused test: `p3g_renderer_transform_helpers_emit_the_pinned_matrices`.
- `src/renderer.cpp:90-140` → existing resource and RenderBuffer traits; type/flags/size/map/unmap/draw handoff evidence: `records_buffers_gradients_images_and_meshes`. Rust's exclusive mutable slice owns the map-access window and the adapter bytes are authoritative, so the B6 adaptation has no separate dirty-state mirror to reconcile; debug map/unmap assertions remain an adapter contract.
- `src/renderer.cpp:142-229` → exact whitespace and shaped-run break/joiner helpers; every complete HarfRust shaping path materializes the annotations on its glyphs and static wrapping consumes those retained values, including consecutive joiners and joiners adjacent to multi-character glyph clusters. Focused tests: `p3g_glyph_run_annotations_match_pinned_break_and_joiner_rules` and `p3g_renderer_whitespace_contract_drives_runtime_word_units`.
- Exact `isWhiteSpace` call-site ports are limited to pinned `raw_text_input.cpp` and `text_modifier_range.cpp` consumers; unrelated Unicode-whitespace owners were left unchanged.

### Lane gate

- `cargo check -p nuxie-render-api` — pass.
- `cargo check -p nuxie-runtime` — pass with the branch's existing warnings.
- `cargo test -p nuxie-render-api p3g_ --lib` — 5 passed.
- `cargo test -p nuxie-render-api records_buffers_gradients_images_and_meshes --lib` — 1 passed.
- `cargo test -p nuxie-runtime p3g_ --lib` — 2 passed.
- `cargo test -p nuxie-runtime asynchronously_decoded_font_notifies_live_text_style_shape_dirt --lib` — 1 passed.
- `python3 tools/b6-audit/check.py` — 448 rows verified, zero UNKNOWN.
- `python3 tools/b6-audit/rust_attribution.py ...` — every in-scope Rust source classified.
- Parity scorecard snapshot regeneration — 448 rows, 425 faithful, 19 pending; scatter ratchet remains `max_multi_module_rows = 154`.

Per the lane gate, no full-workspace battery, C++/Rust golden compare, corpus mutation, or threshold change was run.

## Pending rows

- P3G has no remaining implementation-pending factory.cpp or renderer.cpp member. Both rows intentionally remain `pending-verification` until the landing orchestrator runs the full battery and performs the verification flip.

## Conflict queue

- Shared landing files: `Cargo.lock`, `file-correspondence-manifest.toml`, `port-manifest.toml`, `rust-additions.toml`, `docs/b6-audit/results/misc-core.md`, `docs/parity-scorecard.md`, and `tools/port-manifest/port_manifest.py`.
- The current `docs/parity-closeout-map.md` has no P3-g/C9 row to update; the orchestrator should reconcile its concurrently landing lane map rather than accepting a speculative shared-row edit here.
- The announced S4 pin advance/rebase may conflict in the shared manifests and lockfile; all source claims and tests here are against `d788e8ec` as instructed.
- Workspace-wide `cargo fmt --all -- --check` is red on unrelated/pre-existing formatting in `crates/nuxie/tests/scene_authoring.rs`, `crates/nuxie-audio/src/audio_engine.rs`, `crates/nuxie-runtime/src/constraints.rs`, `crates/nuxie-runtime/src/state_machine.rs`, the existing import order in `crates/nuxie-runtime/src/text/raw_text_input.rs`, and `crates/nuxie-runtime/src/text/text.rs`. P3G did not absorb those unrelated formatting hunks.
