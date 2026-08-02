# MR-2 C02 runtime-components report

## Moved rows

- None. The C02 primary hotspot contains no row that can land as an independent
  pure move: fourteen rows are `split-needed`, and the other five touch roots
  owned by other clusters.

## Skipped rows

- None beyond the `split-needed` and cross-root queues below. No implementation
  body, module declaration/re-export, or manifest row was changed.

## Split-needed queue

The following rows live only in C02's owned root, but the planned extraction is
not mechanical. Their implementations are interleaved with shared
`RuntimeConcreteComponentState` construction/cloning or with multi-row utility
types/`impl` blocks. Extracting them would require partitioning existing items,
changing visibility/signatures, or assigning shared bodies to one upstream row.
They remain unchanged as required by the move plan.

| Row | Planned target | Preserved audit record | Preserved B6 verdict |
|---|---|---|---|
| B6-0115 · `src/bones/bone.cpp` | `crates/nuxie-runtime/src/bones/bone.rs` | `docs/b6-audit/results/bones-math-components.md` | `DIVERGENT` |
| B6-0116 · `src/bones/root_bone.cpp` | `crates/nuxie-runtime/src/bones/root_bone.rs` | `docs/b6-audit/results/bones-math-components.md` | `DIVERGENT` |
| B6-0118 · `src/bones/skinnable.cpp` | `crates/nuxie-runtime/src/bones/skinnable.rs` | `docs/b6-audit/results/bones-math-components.md` | `DIVERGENT` |
| B6-0119 · `src/bones/tendon.cpp` | `crates/nuxie-runtime/src/bones/tendon.rs` | `docs/b6-audit/results/bones-math-components.md` | `DIVERGENT` |
| B6-0289 · `src/math/aabb.cpp` | `crates/nuxie-runtime/src/math/aabb.rs` | `docs/b6-audit/results/bones-math-components.md` | `ISOMORPHIC` |
| B6-0290 · `src/math/bezier_utils.cpp` | `crates/nuxie-runtime/src/math/bezier_utils.rs` | `docs/b6-audit/results/bones-math-components.md` | `ISOMORPHIC` |
| B6-0291 · `src/math/bit_field_loc.cpp` | `crates/nuxie-runtime/src/math/bit_field_loc.rs` | `docs/b6-audit/results/bones-math-components.md` | `ADAPTED` |
| B6-0292 · `src/math/contour_measure.cpp` | `crates/nuxie-runtime/src/math/contour_measure.rs` | `docs/b6-audit/results/bones-math-components.md` | `ADAPTED` |
| B6-0294 · `src/math/mat2d.cpp` | `crates/nuxie-runtime/src/math/mat2d.rs` | `docs/b6-audit/results/bones-math-components.md` | `ISOMORPHIC` |
| B6-0295 · `src/math/mat2d_find_max_scale.cpp` | `crates/nuxie-runtime/src/math/mat2d_find_max_scale.rs` | `docs/b6-audit/SECOND_PASS.md` | `N/A` |
| B6-0296 · `src/math/n_slicer_helpers.cpp` | `crates/nuxie-runtime/src/math/n_slicer_helpers.rs` | `docs/b6-audit/results/bones-math-components.md` | `ISOMORPHIC` |
| B6-0297 · `src/math/path_measure.cpp` | `crates/nuxie-runtime/src/math/path_measure.rs` | `docs/b6-audit/results/bones-math-components.md` | `ADAPTED` |
| B6-0300 · `src/math/raw_path_utils.cpp` | `crates/nuxie-runtime/src/math/raw_path_utils.rs` | `docs/b6-audit/results/bones-math-components.md` | `ISOMORPHIC` |
| B6-0302 · `src/math/vec2d.cpp` | `crates/nuxie-runtime/src/math/vec2d.rs` | `docs/b6-audit/results/bones-math-components.md` | `ISOMORPHIC` |

## Cross-root queue

These rows were left wholly untouched. The ownership ledger and dependency
sequence require C17 to assemble every owner-authored fragment and the manifest
update atomically; a partial C02 move is forbidden.

| Row | Move-kind | Foreign-owned roots | Planned target | Preserved B6 verdict |
|---|---|---|---|---|
| B6-0117 · `src/bones/skin.cpp` | `split-needed` | `crates/nuxie-runtime/src/artboard.rs` (C04) | `crates/nuxie-runtime/src/bones/skin.rs` | `DIVERGENT` |
| B6-0120 · `src/bones/weight.cpp` | `split-needed` | `crates/nuxie-runtime/src/artboard.rs` (C04) | `crates/nuxie-runtime/src/bones/weight.rs` | `DIVERGENT` |
| B6-0123 · `src/component.cpp` | `split-needed` | `crates/nuxie-runtime/src/artboard.rs` (C04) | `crates/nuxie-runtime/src/component.rs` | `DIVERGENT` |
| B6-0299 · `src/math/raw_path.cpp` | `split-needed` | `crates/nuxie-runtime/src/draw.rs` (C05) | `crates/nuxie-runtime/src/math/raw_path.rs` | `ADAPTED` |
| B6-0388 · `src/text/text_input.cpp` | `pure-move`, coordinated | `crates/nuxie-runtime/src/artboard.rs` (C04), `constraints.rs` (C03), and `text.rs` (C12) | `crates/nuxie-runtime/src/text_input.rs` | `TRACKED-GAP` |

All five rows retain their existing manifest `audit_record` values:
B6-0117/B6-0120/B6-0123/B6-0299 retain
`docs/b6-audit/results/bones-math-components.md`; B6-0388 retains
`docs/b6-audit/SECOND_PASS.md`.

## Manifest and audit preservation

- `file-correspondence-manifest.toml` is unchanged because no complete code move
  landed.
- Every row retains its `rust_module`, `audit_record`, `b6_row_id`, and
  `b6_verdict` exactly.

## Gate

- `cargo check --workspace --exclude nux-capi` passed. The workspace emitted
  existing warnings but finished successfully.
