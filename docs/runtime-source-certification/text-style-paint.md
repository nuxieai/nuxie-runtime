# `TextStylePaint` source-pair certification candidate

Status: **author candidate; independent review required**.

This receipt is governed by
`docs/runtime-exact-parity-workflow-correction.md`. The complete pinned cpp,
primary handwritten header, and directly inherited/generated authority were
read before adjudication. This candidate does not self-accept.

## Frozen authority and strict denominator

- Upstream authority: Rive runtime
  `4ac7b32798da0482e441ef09304dc3b480ed3ee5`.
- `src/text/text_style_paint.cpp`: 134 lines, 4,006 bytes, SHA-256
  `1b9fb64d3440012c7640b68dc9a2ba306f32e7b27c3465f02e597e2c324794d9`.
- `include/rive/text/text_style_paint.hpp`: 38 lines, 1,200 bytes, SHA-256
  `c1cb701650a54203d0f0448ad59d96a5c481a43f96b5a02ac3e091aeaed51ed2`.
- Strict handwritten executable denominator: **11 units**: eight cpp bodies
  and three executable header inlines. The include guard and retained
  fields/defaults are mapped separately below.

## Complete handwritten authority map

| # | Pinned authority | Required behavior/order | Concrete Rust ownership | Candidate disposition |
|---:|---|---|---|---|
| 1 | cpp 13 constructor | Construct the aggregate `ShapePaintPath` as clockwise and empty. | `text/text_style_paint.rs:8-20::RuntimeTextStylePaintPaths`; `text.rs:2708-2720` constructs one empty owner per style; command/backend ownership fixes clockwise at `text.rs:733-736` and `draw.rs:21638-21755`. | **mapped under the retained-command renderer adaptation**. |
| 2 | cpp 15-20 `rewindPath` | Rewind aggregate path, clear `m_hasContents`, clear opacity map; retain paint pool. | `text/text_style_paint.rs:81-86::rewind_path`; each render-style rebuild replaces only path state while `draw.rs:10934-10962::RuntimeTextBackendResources` retains pooled paints. | **mapped exact**. |
| 3 | cpp 22-49 `addPath` | Snapshot/raise `m_hasContents` before `opacity > 0`; reject zero, negative, and NaN; append accepted paths clockwise to aggregate and an exact-float opacity bucket; return first-add state. | `text/text_style_paint.rs:20-50::add_path`; insertion is driven from the real retained layout at `text.rs:2708-2751`; exact ordering/NaN/alias evidence at `text/text_style_paint.rs:112`. | **partially mapped; dependent red remains**. State, guard, exact-key buckets, ordering, and return are exact. The Rust path-command seam still lacks the separately owned `ShapePaintPath::addPathClockwise` winding normalization and may differ for contours whose incoming direction requires reversal. This pair does not invent that algorithm. |
| 4 | cpp 51-102 `draw` | Visit current paints in authored order; `shouldDraw`; copy Text blend; opaque bucket first; grow pool to map size; reset pool index for every child; ascending buckets skipping opaque; unconditionally apply each child and reset feather; save/transform/clip/draw order. | Current occurrence paint order: `components.rs:1023-1052`, `artboard.rs:1457-1472`, `draw.rs:10037-10188`; command construction: `text.rs:653-807`; empty-bucket paint loop: `draw.rs:18728-18766`; shared pool and live draw: `draw.rs:18767-18987`; unconditional child configuration: `draw.rs:18559-18579`. | **mapped exact under the renderer backend adaptation**. The prior epoch-only reuse, missing rejected-first paint loop, and wrong empty-style blend source are corrected. |
| 5 | cpp 104-116 `foregroundColor` | Return first current Fill/SolidColor in occurrence order, without `shouldDraw`; otherwise opaque black. | `text.rs:5063-5104::foreground_color` reads `RuntimeShapeList::text_style_paint_locals`, then the live SolidColor property; occurrence/clone membership is rebuilt at `draw.rs:10037-10188`. | **mapped exact**. |
| 6 | cpp 118-121 `shapeWorldTransform` | Return direct parent Text's shape-world transform by reference. | `text.rs:653-681` obtains the current Text world transform and composes the retained shape-local transform; `draw.rs:18834-18871` consumes the same retained transform during draw. | **mapped under the value/retained-command adaptation**; Rust has no borrowed C++ matrix reference. |
| 7 | cpp 123 `pathBuilder` | Return the direct parent Component. | `components.rs:1023-1052` retains the container occurrence; `text.rs:653-807` builds style paths only through the owning Text slice and direct style identity. | **mapped exact in occurrence identity**. |
| 8 | cpp 125-134 `clone` | Generated clone starts custom path/pool state cold, conditionally reattaches the current file asset, and rebuilds inherited child registration from clone lifecycle. | `draw.rs:9419-9451::RuntimeShapeList::clone` cold-clones backend owners; `components.rs:1038-1045::RuntimeTextStylePaintState::clone_for_occurrence` starts membership empty; `artboard.rs:763-803` reruns construction; `draw.rs:10037-10188` rebuilds renderer membership. File-asset behavior is the independently accepted TextStyle owner. | **mapped exact**. Live paint-parent writes freeze the source; the clone registers copied parent ids and uses the rebuilt list in foreground/render owners. |
| 9 | hpp 25 `localPath` | Return `&m_path`. | `text/text_style_paint.rs:56-58::local_path`; renderer keys the single retained style path owner rather than duplicating a local path. | **mapped exact**. |
| 10 | hpp 26 `localClockwisePath` | Return the same `&m_path` pointer as `localPath`. | `text/text_style_paint.rs:60-62::local_clockwise_path`; `draw.rs:19919-19931` aliases clockwise to local backend ownership. Direct pointer/slice identity is asserted at `text/text_style_paint.rs:133-136`. | **mapped exact**. |
| 11 | hpp 30 `getArtboard` | Return the owning Artboard used for factory allocation. | The concrete `ArtboardInstance` owns `RuntimeShapeList` and `RuntimeTextBackendResources`; `draw.rs:18767-18831` allocates the pool through the active occurrence factory. | **mapped under the renderer factory adaptation**. |

## Retained fields, defaults, and generated context

- `m_opacityPaths` is represented by exact-float buckets in
  `RuntimeTextStylePaintPaths`; NaN cannot enter and `ordered_buckets` performs
  the same ascending iteration as `std::map<float, ...>`.
- `m_paintPool` is `RuntimeTextBackendResources::pooled_paints`, keyed by style
  occurrence and pool slot. It survives path rebuilds and is cold on clone.
- `m_path` is the single aggregate path-command owner. Effects and inner
  feather consume this aggregate before per-opacity clip/draw commands at
  `draw.rs:21638-21755`; local and clockwise views alias it.
- `m_hasContents` defaults false, becomes true before the opacity guard, and is
  cleared only by rewind.
- `TextStylePaintBase` generated clone/copy/property behavior and inherited
  TextStyle asset/metric behavior are owned by the separately accepted
  TextStyle pair. `ShapePaintContainer::m_ShapePaints` and ShapePaint child
  lifecycle are directly necessary inherited authority and are represented by
  `RuntimeTextStylePaintState`; the ShapePaint methods themselves are not
  certified here.

The nominal Rust file now owns the pair's custom aggregate/bucket/has-contents
state and inherited TextStyle callback. Renderer/factory resources remain in
`draw.rs` because they are shared deep backend owners, not copied algorithms.

## Source-proven corrections in this candidate

1. Replaced the old `opacity <= 0` gate with exact `opacity > 0`, excluding
   NaN while retaining first-add state before the gate.
2. Preserved an empty-style replay so a rejected-only first path still retains
   `m_renderStyles` and executes the live ShapePaint paint/blend loop.
3. Separated aggregate-path effect/inner-feather preparation from per-opacity
   bucket clip/draw geometry.
4. Removed the epoch-only pooled-paint early return. Every ShapePaint now
   reapplies color/shader/stroke/feather to the shared slot in child order.
5. Added occurrence-owned TextStylePaint child registration and made clone
   construction rebuild it from copied parent ids. Foreground color, command
   construction, backend ownership, and clone output now consume that list.
6. Corrected rejected-only drawing to copy the direct parent Text blend mode,
   rather than the ShapePaint/TextStylePaint blend composition.

## Consumer and fixture accounting

The strict literal upstream owner topology is **0 direct pass / 0 executable
expected-red / 0 adapted / 1 pending**. The sole material case is
`tests/unit_tests/runtime/text_modifier_test.cpp` ordinal 2. Its current Wave
C7 port is pending/unverified and does not yet preserve the complete upstream
animation/draw/Silver assertion stream, so this candidate does not promote it.

Recursive fixture inventory finds **137** readable Paint-bearing files and
**696** TextStylePaint objects. Exactly **135** files are referenced by the
upstream unit-test source; `library.riv` and root `scroll_snap.riv` are
unreferenced. This is incidental impact surface, not direct owner evidence.
The existing `silver:text_feather_falloff` divergence remains a projection;
its first paint-id difference is not causally attributed to this pair without
a complete differential.

## Supporting evidence and focused gates

- `text/text_style_paint.rs:112::add_path_retains_first_call_and_exact_positive_float_buckets`
  covers NaN, signed zero, negative, infinity, exact duplicate keys, ascending
  order, rewind, and local/local-clockwise alias identity.
- `text.rs:7623::text_style_paint_membership_is_frozen_on_source_and_rebuilt_on_clone`
  uses two real Text occurrences: source A retains a paint after live
  `parentId` A->B; clone B rebuilds `[moved, authored]` membership; actual
  `StaticTextSlice::foreground_color` and backend ownership consume it.
- `draw.rs:27319::text_style_paint_reapplies_each_child_to_the_shared_nonopaque_pool_slot`
  advances the pinned `text_feather_falloff.riv` animation until two real child
  paints share one nonopaque pool slot, then observes distinct child state in
  the real retained draw stream.
- `draw.rs:27438::text_style_paint_rejected_first_opacity_retains_style_and_runs_paint_loop`
  drives a real shaped Text through a NaN modifier opacity and proves zero
  commands, retained `EmptyStyle`, no draw-path call, and a live blend-mode
  publication from the ShapePaint loop.
- `draw.rs:27582::text_style_paint_effect_and_inner_feather_use_the_aggregate_path`
  drives the real TextStylePaint/Feather command owner with separated bucket
  and aggregate geometry and proves preparation sees the far aggregate glyph.

Focused commands and results:

- `cargo test -p nuxie-runtime --lib text_style_paint_owner::tests::add_path_retains_first_call_and_exact_positive_float_buckets -- --exact`: 1 passed.
- `cargo test -p nuxie-runtime --lib text::tests::text_style_paint_membership_is_frozen_on_source_and_rebuilt_on_clone -- --exact`: 1 passed.
- `cargo test -p nuxie-runtime --lib draw::tests::text_style_paint_reapplies_each_child_to_the_shared_nonopaque_pool_slot -- --exact`: 1 passed.
- `cargo test -p nuxie-runtime --lib draw::tests::text_style_paint_rejected_first_opacity_retains_style_and_runs_paint_loop -- --exact`: 1 passed.
- `cargo test -p nuxie-runtime --lib draw::tests::text_style_paint_effect_and_inner_feather_use_the_aggregate_path -- --exact`: 1 passed.
- `cargo check -p nuxie-runtime --lib`: passed after the final focused rerun.

## Author conclusion

All 11 handwritten units and retained fields/defaults are enumerated. The
candidate corrects four hidden renderer-state failures and the clone-owned
ShapePaint membership failure without promoting the incomplete upstream
consumer. Independent review must verify the strict denominator, occurrence
registration, shared-pool ordering, rejected-opacity temporal state, aggregate
effect path, and the explicit dependent `addPathClockwise` red.
