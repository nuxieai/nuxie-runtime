# Semantic residuals report

## Scope and oracle

- Lane: `levi/sem-residuals`, based on `origin/main`.
- Oracle: `/Users/levi/dev/oss/rive-runtime` at
  `4ac7b32798da0482e441ef09304dc3b480ed3ee5`.
- Required fixture refresh was run first:
  `rsync -a /Users/levi/dev/nuxie-runtime/fixtures/ fixtures/ && make fixtures`.
- Baseline scripted corpus: `entries=356 exact=325 exact-segments=673
  side-channel-segments=672 diverges=0 unsupported-feature=0 not-yet=31`.

## Port

- Collapsed mounted-artboard/list boundaries now carry inherited collapse state
  into semantic membership. This mirrors upstream
  `Artboard::collapseSemanticBoundary`, `SemanticData::syncSemanticTreeVisibility`,
  and the `SemanticManager::removeChild` structural path. The data-binding
  action now reports the exact removed ids and hierarchy patches.
- Semantic bounds remain occurrence-owned and use retained list-host/item
  transforms while a structural diff is built. The structural snapshot does
  not repoll every node after layout has advanced. Focus scrolling is detected
  by comparing retained scroll-transform snapshots and then running the
  incremental bounds pass. That preserves the observed ordering of upstream
  `SemanticData::updateWorldBounds` and
  `SemanticManager::refresh`/`buildDiffFromFlats`, but it is a compatibility
  adapter rather than the upstream dependency-dirt push path.
- Layout semantic boxes begin with the solved `LayoutComponent` border box and
  use a D3 Taffy adapter to restore the horizontal fill padding that Yoga
  includes in the upstream box before applying the mounted root transform.
  The resulting provider calculation mirrors
  `SemanticProvider::tryNodeWorldBounds`, `rootTransformAABB`, and
  `semanticBounds` for the provider-owned node box.
- `SemanticBounds::is_empty_or_nan` now mirrors pinned
  `AABB::isEmptyOrNaN`: only positive-area boxes are non-empty. A direct unit
  test covers point, line, NaN, expansion-sentinel, and positive-area cases.
- Existing authored `stateFlags` synchronization preserves the retained
  Focused bit and now lands in the same structural/update window as upstream
  `SemanticData::stateFlagsChanged`. Both semantic fixtures produce the exact
  Selected transitions.

## Differential results

| Corpus row | Result | Evidence |
| --- | --- | --- |
| `semantic_text_inference_lt1` (Simpsons) | **exact; promoted** | 3/3 exact side-channel segments. Initial root/pre-layout bounds, inferred hierarchy/labels, Selected updates, and post-action update lists all match. |
| `semantic_provider_focus_lt1` | **exact; unchanged** | Existing 3/3 exact focus/scroll side-channel evidence remains green. |
| `semantic_data_binding_action_lt1` | **not-yet; improved** | The only semantic-line difference is the initial glyph bounds for inferred Text ids 5, 8, 11, and 14. Selected propagation, removals, children patches, and every frame after the tap are exact. |

The remaining data-binding values are:

| id | C++ initial bounds | Rust initial bounds |
| --- | --- | --- |
| 5 | `(169,206.398438,288.53125,225.601562)` | expansion sentinel |
| 8 | `(169,246.398438,324.789062,265.601562)` | expansion sentinel |
| 11 | `(169,286.398438,257.523438,305.601562)` | expansion sentinel |
| 14 | `(169,326.398438,255.3125,345.601562)` | expansion sentinel |

These Text nodes live inside freshly mounted component-list children. Rust has
not yet published their retained shaped-glyph bounds when the initial semantic
snapshot is taken. The row remains honestly `not-yet`; no tolerance or
fixture-specific expected output was added.

## Correspondence promotion

- `src/semantic/semantic_inference_registry.cpp`: promoted to `faithful` /
  `orchestrator-verified` in `file-correspondence-manifest.toml` and `ported` in
  the legacy port manifest, based on the exact Simpsons differential.
- `semantic_data.cpp`, `semantic_manager.cpp`, and `semantic_provider.cpp`:
  remain pending/partial because the complete data-binding differential is not
  exact. The sampled corpus remainder is the four initial mounted Text bounds;
  the structural review remainder is replacing scroll snapshot comparison
  with owner-pushed WorldTransform/Path semantic dirt (and retaining recursive
  scratch collections rather than allocating them per synchronization).

## Review closeout

### Standards

- Two hard findings remain: scroll-transform snapshot comparison is not the
  upstream owner-pushed dependency-dirt architecture, and recursive semantic
  synchronization creates fresh maps/sets/vectors rather than clearing and
  refilling retained scratch storage.
- The horizontal fill-padding correction is explicitly classified as a D3
  Taffy/Yoga boundary adapter. It reconstructs the upstream provider input and
  is covered by the exact Simpsons differential.

An owner-pushed scroll-dirt experiment was attempted during review, but it
changed the pre-layout/list-host update window and regressed the exact semantic
fixtures. It was reverted instead of weakening the oracle. These findings are
why the data, manager, and provider correspondence rows remain partial.

### Spec

- The requested Selected propagation, collapsed-boundary removals, and
  root/pre-layout bounds residuals are exact in Simpsons and exact after the
  data-binding action.
- Generic non-scroll WorldTransform/Path semantic dirt is not yet routed as a
  pushed update. The only observed data-binding differential remains the four
  initial shaped-glyph bounds listed above.

## Gates

- `cargo test -p nuxie-runtime` — pass.
- `cargo test -p nuxie --features scripting` — pass.
- `make scripted-golden-compare` — pass:
  `entries=356 exact=326 exact-segments=676 side-channel-segments=675
  diverges=0 unsupported-feature=0 not-yet=30`.
- `make runtime-frame-loop-port-check` — pass (112 checker tests; final port
  and test-correspondence checks pass).
- `make rust-attribution-check` — pass (10 tests; all in-scope Rust sources
  classified).
- `git diff --check` — pass.

## Commit status

- `8a9b5474` (`[SEMRES] Capture lane finalization`) records the semantic runtime,
  corpus promotion, manifests, documentation, and this report.
- `bb422201` merges the updated `origin/main` into the lane after that commit.

Git metadata was read-only during implementation, so the runtime changes could
not be split into the originally requested coherent-step commits before lane
finalization. The history records one semantic finalization commit plus the
upstream merge rather than pretending that split occurred.
