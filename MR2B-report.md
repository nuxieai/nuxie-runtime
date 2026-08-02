# MR-2b C02 runtime-components split-wave report

## Moved rows

| Row | Canonical target | Disposition |
|---|---|---|
| B6-0116 · `src/bones/root_bone.cpp` | `crates/nuxie-runtime/src/bones/root_bone.rs` | Moved exact RootBone subtype and property-key logic; the shared bone payload stays in the natural B6-0115 owner. |
| B6-0118 · `src/bones/skinnable.cpp` | `crates/nuxie-runtime/src/bones/skinnable.rs` | Moved Skinnable kind/state, subtype dispatch, and occurrence clone logic. |
| B6-0119 · `src/bones/tendon.cpp` | `crates/nuxie-runtime/src/bones/tendon.rs` | Moved Tendon state, exact subtype construction, and default reset logic. |
| B6-0291 · `src/math/bit_field_loc.cpp` | `crates/nuxie-runtime/src/math/bit_field_loc.rs` | Moved the complete bit-offset/width mask helper and redirected `objects.rs`. |
| B6-0294 · `src/math/mat2d.cpp` | `crates/nuxie-runtime/src/math/mat2d.rs` | Moved the Mat2D value type, core arithmetic, and core regression test. |
| B6-0295 · `src/math/mat2d_find_max_scale.cpp` | `crates/nuxie-runtime/src/math/mat2d_find_max_scale.rs` | Moved the whole inherent method and its direct fixtures. |

All six moves are behavior-neutral. Public `nuxie_runtime::Mat2D` remains unchanged through the required `components` compatibility re-export because `crates/nuxie-runtime/src/lib.rs` is C09-owned and still exports that path.

## Justified exceptions

These rows retain a genuine multi-owner seam after the clean bodies leave `components.rs`, or their plan mapping pointed at an aggregate that did not contain the actual separable implementation. Their manifest and frame-loop rows now name the retained owners, and each manifest note names the entangled sibling that prevents a behavior-neutral per-row split.

| Row | Retained owner | Entangled sibling(s) |
|---|---|---|
| B6-0115 · `src/bones/bone.cpp` | `bones/bone.rs`; `artboard.rs` | B6-0094 Artboard's private synthetic regression harness. The state implementation moved; the row-specific retained-child dirt test and citation remain in that harness. |
| B6-0289 · `src/math/aabb.cpp` | `crates/nuxie-runtime/src/draw.rs` | B6-0302 Vec2D and B6-0294 Mat2D consumers. |
| B6-0290 · `src/math/bezier_utils.cpp` | `crates/nuxie-runtime/src/draw.rs` | B6-0292 ContourMeasure and B6-0297 PathMeasure. |
| B6-0292 · `src/math/contour_measure.cpp` | `crates/nuxie-runtime/src/draw.rs` | B6-0290 BezierUtils and B6-0297 PathMeasure share `TrimContour`. |
| B6-0296 · `src/math/n_slicer_helpers.cpp` | `layout/n_sliced_node.rs`; `shapes/slice_mesh.rs` | B6-0254 NSlicedNode and B6-0370 SliceMesh. |
| B6-0297 · `src/math/path_measure.cpp` | `crates/nuxie-runtime/src/draw.rs` | B6-0292 ContourMeasure and B6-0290 BezierUtils. |
| B6-0300 · `src/math/raw_path_utils.cpp` | `crates/nuxie-runtime/src/draw.rs` | B6-0299 RawPath and B6-0290 BezierUtils command helpers. |
| B6-0302 · `src/math/vec2d.cpp` | `crates/nuxie-runtime/src/draw.rs` | B6-0289 Aabb and B6-0290 BezierUtils. |

No empty target modules or one-line re-export shims were created for exceptions.

## Queued cross-root rows

These rows remain wholly untouched for coordinated landing with their other owners:

| Row | Planned target | Other owned root(s) |
|---|---|---|
| B6-0117 · `src/bones/skin.cpp` | `crates/nuxie-runtime/src/bones/skin.rs` | `artboard.rs` (C04) |
| B6-0120 · `src/bones/weight.cpp` | `crates/nuxie-runtime/src/bones/weight.rs` | `artboard.rs` (C04) |
| B6-0123 · `src/component.cpp` | `crates/nuxie-runtime/src/component.rs` | `artboard.rs` (C04) |
| B6-0299 · `src/math/raw_path.cpp` | `crates/nuxie-runtime/src/math/raw_path.rs` | `draw.rs` (C05) |
| B6-0388 · `src/text/text_input.cpp` | `crates/nuxie-runtime/src/text_input.rs` | `artboard.rs` (C04), `constraints.rs` (C03), `text.rs` (C12) |

The explicitly foreign roots `nuxie-binary/src/lib.rs`, `nuxie-scripting/src/vm.rs`, `nuxie-runtime/src/lib.rs`, and `nuxie/src/lib.rs` were not edited.

## Four-place residue

- `file-correspondence-manifest.toml`: all 14 local rows updated to canonical targets or justified actual-owner exceptions.
- `docs/runtime-frame-loop-ownership.toml`: all 14 per-row `rust_modules` arrays and the `component-update-graph` source-set path list updated.
- Mat2D upstream attribution comments and tests moved with their bodies; no moved upstream-file mention remains in `components.rs`.
- `objects.rs` imports the moved BitFieldLoc helper directly. No orphaned one-line re-export shim was introduced; the retained Mat2D re-export is live public-API plumbing owned for later C09 reconciliation.

## Validation

- `cargo fmt --all -- --check`: passed.
- `cargo check -p nuxie-runtime`: passed.
- `cargo check --workspace --exclude nux-capi`: passed.
- `make runtime-frame-loop-port-check`: passed after the final ledger correction (108 unit tests plus correspondence and ledger checks).
- `make rust-attribution-check`: its 10 unit tests pass, then the repository-level scan reports only three pre-existing unclassified files: `crates/nuxie-audio/src/device.rs`, `crates/nuxie-audio/src/engine.rs`, and `crates/nuxie-runtime/src/state_machine/transition_condition_op.rs`. The C02-caused `nuxie-render-api/src/lib.rs` overlap was removed; no C02-created file remains in the finding list.

## Commit and sandbox state

- Batch commit: this report is included in the commit titled `[MR-2b/C02] Split B6-0115,B6-0116,B6-0118,B6-0119,B6-0291,B6-0294,B6-0295 from components.rs` (the final SHA changes when this report is amended).
- The checkout began on stale `levi/mr2-c02`, not the requested `levi/mr2b-c02`. The sandbox allowed the batch commit but repeatedly denied the worktree/ref locks needed to switch, rebase, merge current `origin/main`, rename the branch, or create the requested branch ref. The implementation files were reconciled against `origin/main`; branch-base correction remains an integrator action.
