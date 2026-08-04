# Text-topology lane report

Date: 2026-08-04

Branch: `levi/perf-text-topology`

Pinned C++ runtime: `rive-runtime@4ac7b32798da0482e441ef09304dc3b480ed3ee5`

## Outcome

The imported Text topology is now retained per concrete Text occurrence rather
than reconstructed on every dirty draw. The retained slice owns the imported
run, style, modifier, variation, feature, paint-container handle, and font-asset
identity topology. Shaping is rerun only at the existing text-shape
invalidation boundary; its final draw commands, raw paths, clip state, paint
pools, and backend resources remain in the existing retained draw frame.
Runtime file import now also builds a dense index of imported
FileAsset object ids, so font and other file-asset resolution no longer scans
every object in the file.

The work was landed in independent rows:

1. `5fd52314 perf(binary): index imported file assets`
2. `d9dc3be4 perf(text): retain imported text topology`
3. `7e993be7 test(silver): promote animated clipping nodes`
4. `01e3ae4f perf(text): retain style handles across transform dirt`

The third row is a silver-oracle ratchet only: `animated_clipping-nodes` was
already byte-exact and was promoted from an allowed difference to exact.

## C++ correspondence and retained ownership

The correspondence manifest maps `src/assets/file_asset.cpp` to
`crates/nuxie-binary/src/assets/file_asset.rs` and `src/text/text.cpp` to the
decomposed Rust owners in `crates/nuxie-runtime/src/text/text.rs`,
`crates/nuxie-runtime/src/text.rs`, `crates/nuxie-runtime/src/draw.rs`, and
`crates/nuxie-runtime/src/artboard.rs`.

The port mirrors the pinned C++ ownership boundaries as follows:

| Pinned C++ owner | Rust retained owner |
|---|---|
| `Text::m_allRuns`, styles, modifier groups | one lazy `Arc<StaticTextSlice>` on each `RuntimeTextDrawOwner` |
| shaped paragraphs and lines | rebuilt at text-shape invalidation; their materialized command/path output is retained in the draw frame |
| style paints and draw commands | `RuntimeCachedTextShapePaints::commands` |
| font references | stable font asset global/id identities in `StaticTextStyle`; bytes remain live runtime inputs |
| render paths, clip path, paint pools, backend handles | existing `RuntimeCachedTextShapePaints` and `RuntimeTextBackendResources` |
| file-level asset collection | import-time `RuntimeFile::file_asset_object_ids` dense index |

`StaticTextSlice` and its styles are owned values. Styles retain stable graph
indices for their paint containers rather than cloning mutable graph nodes;
embedded/runtime font byte storage is deliberately not copied into the
topology.

## Invalidation fidelity

- Text value and data-bound runs are still resolved from the live
  `ArtboardInstance` whenever shaping is rebuilt; only immutable imported
  run/style topology is cached.
- Dynamic font resolution remains live and keeps the existing precedence:
  instance override, runtime-loaded bytes, embedded bytes, then external asset
  id.
- Text modifiers and variation helpers still participate in the existing dirt
  propagation and shaping path.
- Text-shape, layout, Path, and Paint dirt retain their prior rebuild
  boundaries. Render-opacity-only dirt now follows pinned `Text::update`: it
  updates effective command paint opacity without rebuilding shaped geometry
  or render paths.
- World-transform dirt updates cached world geometry, clip state, paint-space
  transforms, and color transforms in place. It does not reshape local text or
  rebuild local paths. The imported local transform is retained directly so
  initially singular world transforms remain exact.
- `text_render_opacity_propagates_without_rebuilding_retained_paths` asserts
  path `Arc` identity and byte-identical path commands across an opacity-only
  update.

## Performance evidence

Both requested measurements use release scripted runners, the pinned C++
runtime, 100 frames at 60 Hz, C++-first ordering, scripts enabled, and the
median of five iterations. Times below are the Rust `advance + draw` phase per
frame.

| Fixture | Baseline ms/frame | Retained ms/frame | Change |
|---|---:|---:|---:|
| `script_create_text_runs` | 1.527619 | 1.367418 | -10.49% |
| `text_vertical_trim_test` | 0.094717 | 0.064614 | -31.78% |
| `layout_text_match` | 0.171508 | 0.146746 | -14.44% |

All three requested fixtures improve; the dirty-text fixture removes 0.160200
ms/frame and the two mostly static fixtures remove 0.030103 and 0.024763
ms/frame. The dated methodology, digests, and raw JSON links are in
[`docs/perf-size-evidence.md`](docs/perf-size-evidence.md) and
[`docs/evidence/texttop-2026-08-04/`](docs/evidence/texttop-2026-08-04/).

The independent 24-row performance ratchet is green:

- `make perf-gate`: `PASS files=24`
- `script_create_text_runs`: 1.229818 Rust ms/frame, 293.690x, ceiling 378x
- `text_vertical_trim_test`: 0.069744 Rust ms/frame, 11.500x, ceiling 25x

## Exactness and focused verification

- Full `make scripted-golden-compare`: 363 entries, 346 exact rows, 1,126
  exact segments, 1,121 side-channel segments, 12 declared divergences, five
  declared not-yet rows, and zero undeclared failures.
- Requested text rows are byte-exact: `script_create_text_runs` (123,660
  bytes), `text_vertical_trim_test` (266,633 bytes), and `layout_text_match`
  (2,690,852 bytes); eight exact and eight side-channel segments total.
- `cargo test -p nuxie-binary`: green (26 library, 13 authoring, 74 C++
  import, six F14, and 108 fixture tests).
- `cargo test -p nuxie-runtime --lib`: 976/976 green (including all 32 text
  tests).
- The opacity-retention regression test is green.
- `cargo check -p nuxie-runtime`: green.
- `make rust-attribution-check`: green.
- `make runtime-frame-loop-port-check`: green.

## Repository-wide gate blockers

The branch does not claim every requested repository-wide gate is green. The
following failures are outside the two implementation rows and were retained
rather than weakening exact or audit oracles:

- Full silver validation reaches the exact `focusable_element` row and differs
  at frame 1, operation 144 (`expected color, got save`). Reverse-applying both
  lane implementation commits produces the identical failure. Earlier in the
  same run, `animated_clipping-nodes` proved exact and its stale difference was
  promoted in `7e993be7`.
- Ordinary `cargo test --workspace` reaches 909 passing C++-probe tests, then
  fails the unrelated FL-C5 expectations
  `fl_c5_apply_events_100_batches` (100 versus 1) and
  `fl_c5_constructor_order_source_and_runtime_boundaries_match_cpp`.
- `make cpp-oracle-workspace-tests` stops in its prerequisite B-6 checker: the
  current correspondence manifest and task pin are `4ac7b327`, while the
  checker still hard-codes the older `d788e8ec` audit pin and 448-row census.
- `make port-manifest-check` fails two checker unit tests that still expect the
  old `P3E` prefix for the `lua_gpu.cpp` note; the live row now starts with
  `GPUCEIL`.
- `make runtime-drawing-port-check` still points the
  `shape.path_composer` owner at the pre-decomposition `draw.rs`; the function
  now lives in `shapes/path_composer.rs`.

The upstream runtime checkout remains at the required `4ac7b327` pin with no
tracked-file changes (its existing ignored/untracked build products remain).
No evidence or build output was written under `/tmp`.
