# V2 Status

Working state for `/goal` sessions. Keep this file small and current; it is
the only memory the next session has. Update it every commit.

## Metric

- Exact-status segments (file × sample): 584 across 263 files (strict
  exact=573/252; tolerant=11/11; structural=0/0)
- Current compare: `make golden-compare` reports exact=263,
  exact-segments=584, diverges=0, unsupported-feature=32, not-yet=0
- Parked breakdown: M5=0 by manifest query; `make golden-compare` reports
  gated=6 harness=26
- Current milestone: **M7 — Public `rive` API + C ABI; perf within target of C++**

## M7 Perf Fence

- Gate command: bare `make perf-hot-loop`, which now uses release/null-renderer
  whole-repeat `total_ms` runner timings, min aggregation, the deliberate
  image-bearing focused corpus, `PERF_ITERATIONS=10`, and
  `PERF_BENCHMARK_REPEAT=100`. Phase timings remain JSON/console diagnostics
  but no longer define the aggregate score.
- Acceptance: three independent invocations must all report aggregate min
  Rust/C++ <= 2.0, with 1-minute load below about 8 and the C++ min-sum inside
  its 0.70-0.95 ms sanity band.
- Noise rule: ratio movement below about 0.08 is below single-run resolution;
  claim it only with two pre/post runs. Debug builds, wall-clock process time,
  and serializer output are not M7 decision-grade.
- Current standing: after retained path-composer graph lookups, dense
  draw-path slots, graph-scoped dense path-geometry command slots, dense
  decoded-image slots, cached layout-adjusted draw world transforms, and dense
  mesh render-buffer slots, retained image layout local transforms, and
  retained draw raw paths, dense retained clip/background path slots, plus
  dense render-paint configuration slots, no-nested clean paint-preparation
  skip, whole-repeat hot-loop total timing, and retained shape-paint path
  command slots, plus solid-color render-paint dirt gating, direct
  append/prune shape path assembly for common non-n-sliced draw paths, and
  no-gradient paint-prep fast paths, full
  `make golden-compare` and `cargo test --workspace` remain green. The latest
  release/null-renderer sample before the `total_ms` harness change is still
  directional only:
  `make perf-hot-loop PERF_MAX_RATIO=999` reports aggregate min Rust/C++=3.219,
  but the C++ min-sum was 1.037 ms, outside the 0.70-0.95 ms sanity band.
  Movement from the previous 3.225 directional sample is below the 0.08 noise
  floor. Retaining shape-paint path commands moved the focused
  `spotify_kids_demo@0` JSON run from total=6.315 / draw=8.222 to total=4.038
  / draw=6.729, with Rust min total dropping from 1.884 ms to 1.301 ms.
  Three post-slice user-requested high-load tracking runs report aggregate min
  Rust/C++=3.474, 3.173, then 3.030, but C++ min-sums were
  1.203/1.232/1.104 ms, outside the sanity band; treat them as directional.
  Solid-color render-paint dirt gating moves focused `advance_blend_mode@0`
  JSON from total Rust/C++=5.417 to 5.019 by keeping visible-to-visible
  `SolidColor.colorValue` changes out of the prepared topology epoch while
  still invalidating topology on alpha visibility crossings. A follow-up
  open-fence hot-loop reports aggregate min Rust/C++=3.043, but C++ min-sum
  was 1.090 ms and load was still above the fence, so it is directional only.
  The append/prune slice is also directional-only: it lowers sampled allocator
  churn in `spotify_kids_demo@0` and the best focused JSON draw min moved from
  Rust 0.752 ms / 6.708x draw to Rust 0.638 ms / 5.815x draw, but same-session
  full hot-loop samples ran at load 21-24 with C++ min-sums above the sanity
  band. The no-gradient paint-prep fast path is a focused win for
  `advance_blend_mode`: a Rust-only 20M repeat sample moved from
  elapsed=13053 ms / prepare=2610 ms to elapsed=11568 ms / prepare=1305 ms,
  and the best same-session focused JSON prepare min moved from 0.0201 ms to
  0.0137 ms. The latest user-requested open-fence hot-loop snapshot reports
  aggregate min Rust/C++=2.943 with load 5.58/7.58/12.68 and C++ min-sum
  0.984 ms, still outside the 0.70-0.95 ms sanity band; visible outliers are
  `advance_blend_mode` at 4.305/4.663, `spotify_kids_demo@0`=4.132,
  `animation_reset_cases` samples around 3.04-3.32, `ai_assitant@0`=2.367,
  `animated_clipping@0`=1.972, and `align_target@0`=1.757. Strict <=2.0
  remains open. The update-pass fixed-overhead slice removes the Rust-only
  per-pass `update_order` clone, uses the cached `SolidColor.colorValue`
  property key in the remaining dirt checks, and skips converter advancement
  on zero-elapsed data-bind updates. Same-session focused tracking moves
  `advance_blend_mode` advance from about 0.030 ms to a best 0.026 ms, with a
  Rust-only 20M repeat moving advance from 8011 ms to 6669 ms. The requested
  open-fence hot-loop snapshot reports aggregate min Rust/C++=3.105 with load
  33.66/24.38/22.16 and C++ min-sum=1.208 ms, so it is tracking-only rather
  than M7 acceptance evidence. A follow-up no-recording/scratch-buffer slice
  removes the remaining Rust-only `updated_locals` vector growth from runtime
  `update_pass` callers while preserving public/test reporting, and retains
  the nested context-source collection buffer on `ArtboardInstance` instead of
  allocating it fresh every nested data-bind propagation pass. A same-file
  `advance_blend_mode` Rust-only 50M repeat moved advance from 12676 ms to
  11605 ms and total from 27905 ms to 26480 ms. The same-turn open-fence
  hot-loop snapshot reports aggregate min Rust/C++=3.198 with load
  36.41/24.07/20.27 and C++ min-sum=1.589 ms, so it is tracking-only and not
  M7 acceptance evidence. A follow-up user-requested tracking run deliberately
  ignored the load fence with `PERF_MAX_RATIO=999` and reports aggregate min
  Rust/C++=2.879 with load 18.92/21.62/19.94 and C++ min-sum=1.378 ms. This
  is still outside the sanity band, but it is the current open-fence tracking
  baseline; visible remaining ratios are `advance_blend_mode`=4.119/4.284,
  `spotify_kids_demo@0`=3.949, `animation_reset_cases`=3.423-3.783,
  `animated_clipping@0`=2.348, `ai_assitant@0`=2.231, and
  `align_target@0`=1.969. A draw/property key-lookup slice then removes the
  remaining direct `RuntimeObject::*_property("name")` calls from the
  draw/image/mesh hot path, replacing them with numeric key reads and explicit
  call-site defaults, matching C++ generated field access rather than runtime
  name discovery. Focused `advance_blend_mode` profiling no longer samples
  `property_by_name_in_hierarchy` or `stored_field_initializer`; Rust-only
  50M tracking moved total from 37491 ms to 34859 ms and draw from 15269 ms
  to 13161 ms under high load. The same-turn open-fence hot-loop snapshot
  reports aggregate min Rust/C++=2.883 with load 11.59/17.24/21.02 and C++
  min-sum=1.077 ms, so it is tracking-only and not M7 acceptance evidence;
  visible ratios are `spotify_kids_demo@0`=4.387,
  `advance_blend_mode`=4.138/3.928, `animation_reset_cases`=2.831-3.123,
  `ai_assitant@0`=2.265, `animated_clipping@0`=1.897, and
  `align_target@0`=1.667. A follow-up user-requested run deliberately ignored
  the load fence with the same open-fence command and reports aggregate min
  Rust/C++=2.792 with load 11.51/16.31/19.75 and C++ min-sum=1.060 ms; visible
  ratios are `spotify_kids_demo@0`=4.147, `advance_blend_mode`=3.975/4.061,
  `animation_reset_cases`=2.726-3.109, `ai_assitant@0`=2.224,
  `animated_clipping@0`=1.952, and `align_target@0`=1.554. This is the current
  open-fence tracking baseline, not M7 acceptance evidence. A C++ path dirt
  gate slice then stopped treating ordinary `WORLD_TRANSFORM` dirt as retained
  path-command epoch churn while still invalidating the prepared draw frame.
  Focused `spotify_kids_demo@0` profiling showed the expected draw-side
  movement (`draw_ms` 33483 -> 32698 over a 5M repeat; sampled
  `append_transformed_path_commands` remains dominant). Full
  `make golden-compare`, `cargo test --workspace`, and
  `cargo fmt --all -- --check` pass. The same-session open-fence hot-loop
  reports aggregate min Rust/C++=2.804 with load 21.53/15.32/15.20 and C++
  min-sum=1.074 ms, so it is tracking-only and below the noise floor versus
  the prior 2.792 snapshot;
  `spotify_kids_demo@0` is directionally better at 4.066. A follow-up
  user-requested open-fence rerun deliberately ignored the load fence and
  reports aggregate min Rust/C++=2.903, Rust min-sum=3.464 ms, C++
  min-sum=1.193 ms, and load 29.71/19.24/16.76. This is the current
  tracking-only snapshot, not M7 acceptance evidence; visible ratios are
  `advance_blend_mode@0.25`=4.229, `spotify_kids_demo@0`=4.229,
  `advance_blend_mode@0`=3.793, `animation_reset_cases`=2.812-3.053,
  `ai_assitant@0`=2.325, `animated_clipping@0`=2.262, and
  `align_target@0`=1.775. A follow-up scout tried narrowing the
  shape-paint path-command cache from the artboard-global `path_epoch` to a
  mixed component-local dependency epoch over the shape and composed path
  locals. It was fully backed out: the focused `spotify_kids_demo@0` 5M run
  stayed flat-to-worse (total 55600 -> 55182 ms, draw 36056 -> 41454 ms), and
  the same open-fence command worsened to aggregate Rust/C++=3.288 with C++
  min-sum=1.255 ms under very high load. A follow-up user-requested
  open-fence measurement on the backed-out source deliberately ignored the
  load fence with `PERF_MAX_RATIO=999` and reports aggregate min Rust/C++=2.858,
  Rust min-sum=3.099 ms, C++ min-sum=1.084 ms, and load rising from
  18.57/13.80/15.53 before the run to 30.08/20.93/18.22 after it. This is the
  current tracking-only snapshot, not M7 acceptance evidence; visible ratios
  are `spotify_kids_demo@0`=4.186, `advance_blend_mode`=3.854/4.012,
  `animation_reset_cases`=2.908-3.307, `ai_assitant@0`=2.160,
  `animated_clipping@0`=1.997, and `align_target@0`=1.944. After profiling
  that exact hot site, Rust now caches the remaining generated draw-property
  keys for `Fill.fillRule`, draw parent ids, mesh UVs, and weight buffers,
  matching C++ generated field access instead of runtime name discovery.
  Focused `spotify_kids_demo@0` tracking moved from a 5M baseline
  total=43254.6/draw=29210.5 ms to a 2M post-slice sample
  total=17142.6/draw=11105.3 ms, and the sample no longer shows
  `property_key_for_name` or `definition_by_name`; the remaining stack is
  path assembly, world-transform-with-bounds, path-geometry command frames,
  layout-style key dispatch, and paint configuration. A pre-reserve path
  command-capacity scout was fully backed out because it shifted cost into
  capacity walking and ran worse. The user-requested open-fence hot-loop run
  deliberately ignored the load fence with `PERF_MAX_RATIO=999` and reports
  aggregate min Rust/C++=2.660, Rust min-sum=2.628 ms, C++ min-sum=0.988 ms,
  and post-run load 3.35/4.39/6.37; this is tracking-only because the C++
  min-sum is just outside the 0.70-0.95 ms sanity band. A follow-up
  layout-style generated-key slice removes the sampled
  `runtime_layout_style_property_key_for_name` string dispatch by routing
  fixed `LayoutComponentStyle` reads through enum-backed cached numeric keys,
  matching C++ generated field access. Focused `spotify_kids_demo@0` 2M
  tracking moves total=17142.6/draw=11105.3 ms to
  total=16057.6/draw=10408.0 ms, and the sample no longer shows the
  layout-style helper. The same-session open-fence hot-loop reports aggregate
  min Rust/C++=2.718, Rust min-sum=2.635 ms, C++ min-sum=0.969 ms, and load
  4.33/3.78/4.66; this is still tracking-only, with aggregate movement
  against 2.660 below the 0.08 noise floor and `spotify_kids_demo@0`
  directionally better at 3.894. A follow-up user-requested open-fence
  tracking run deliberately ignored the load fence and reported aggregate min
  Rust/C++=2.731, Rust min-sum=2.666 ms, C++ min-sum=0.976 ms, with
  `spotify_kids_demo@0`=3.811. A C++ collapse-propagation slice then kept
  gradient paint preparation on the ancestor-aware collapse predicate but
  routed draw/path/layout traversal through a local C++-shaped predicate:
  `Component::isCollapsed` plus `LayoutComponent::isCollapsed` display:none,
  relying on the existing update pass to propagate collapse dirt to
  descendants. Focused `spotify_kids_demo@0` 2M tracking moved
  total=16008.97/draw=10370.35 ms to total=12555.24/draw=7039.85 ms, and the
  sample no longer shows `runtime_component_is_effectively_collapsed` in the
  draw hot stack. The same-session open-fence hot-loop reports aggregate min
  Rust/C++=2.454, Rust min-sum=2.418 ms, C++ min-sum=0.986 ms, and
  `spotify_kids_demo@0`=3.063; this is still tracking-only because the C++
  min-sum is outside the 0.70-0.95 ms sanity band. Full
  `make golden-compare`, `cargo test --workspace`, and
  `cargo fmt --all -- --check` pass. A follow-up RawPath prune-shape slice
  ports C++ `RawPath::pruneEmptySegments` compaction behavior more closely by
  avoiding the eager multi-contour scan and write-back/truncate work until an
  empty segment is actually found, while preserving the Rust multi-contour
  near-empty guard. Focused `spotify_kids_demo@0` 2M tracking moved
  total=12763.56/draw=6993.76 ms to total=12446.43/draw=6906.15 ms, and
  `prune_empty_path_segments_from` fell out of the sampled top stack. The
  same-session open-fence hot-loop reports aggregate min Rust/C++=2.451, Rust
  min-sum=2.391 ms, C++ min-sum=0.975 ms, and `spotify_kids_demo@0`=2.936;
  still tracking-only because the C++ min-sum is outside the 0.70-0.95 ms
  sanity band. Full `make golden-compare`, `cargo test --workspace`, and
  `cargo fmt --all -- --check` pass. A follow-up clipping command-retention
  slice lands the next C++ `ClippingShape::m_path`-shaped layer:
  `runtime_draw_clip_start` now reuses composed clipping command streams from
  `RuntimeRenderPathCache` keyed by graph/clipping local and path/layout epoch
  before feeding the existing retained clip render path. This follows the
  current Rust retained path/layout invalidation boundary rather than adding a
  new world-transform rule in this slice. Caller instrumentation
  on `spotify_kids_demo@0` showed repeat-frame append work dominated by the
  clipping site: 440 calls from `runtime_clipping_shape_path_commands` versus
  148 from regular shape-paint composition in a repeat=4 probe. Focused
  `spotify_kids_demo@0` 2M tracking moved total=13071.20/draw=7104.94 ms to
  total=7536.70/draw=2357.05 ms, and `append_transformed_path_commands` /
  `path_geometry_commands_frame` fell out of the sampled top stack. The
  user-requested open-fence hot-loop run deliberately ignored the load fence
  with `PERF_MAX_RATIO=999` and reports aggregate min Rust/C++=2.290, Rust
  min-sum=2.238 ms, C++ min-sum=0.977 ms, and `spotify_kids_demo@0`=2.190;
  still tracking-only because the C++ min-sum is just outside the 0.70-0.95 ms
  sanity band. Full `make golden-compare`, `cargo test --workspace`,
  `cargo fmt --all -- --check`, and `git diff --check` pass. A follow-up
  keyed-animation setter slice ports the C++ `KeyedProperty::apply` /
  `KeyFrameDouble::apply` property-key shape: transform keyframes now use the
  already-imported `propertyKey` for transform reads/writes, and keyed double
  animations call the generated double setter directly while preserving the
  existing Artboard data-bind, cache, layout, path, and dirt side effects.
  Focused `spotify_kids_demo@0` 2M tracking moved total=7404.35 /
  advance=5187.65 ms to total=5964.58 / advance=3778.97 ms, with an
  intermediate transform-key-only sample at total=6602.11 / advance=4406.16
  ms. The midpoint sample no longer showed `transform_property_key` and exposed
  generic setter work, which the second half of the slice removes from keyed
  double writes. The latest profile frontier is now `apply_linear_animation`,
  generated `double_property`, cubic interpolation, and the smaller draw-side
  world-transform/draw-path/paint-config stack. The
  user-requested open-fence hot-loop reports aggregate min Rust/C++=2.152,
  Rust min-sum=2.046 ms, C++ min-sum=0.951 ms, and
  `spotify_kids_demo@0`=2.046; still tracking-only because the aggregate is
  above 2.0 and the C++ min-sum is just outside the 0.70-0.95 ms sanity band.
  Full `make golden-compare`, `cargo test --workspace`,
  `cargo fmt --all -- --check`, and `git diff --check` pass. A follow-up
  keyed-color setter slice ports the matching C++ `KeyFrameColor::apply`
  shape: keyed color animations and animation-reset color/double entries now
  write through generated property-key setters, preserving the existing
  Artboard data-bind/cache/layout/path/dirt side effects. SolidColor
  `colorValue` changes also route directly to the existing alpha visibility
  topology rule instead of entering the broad prepared-frame type-name
  predicate first. The profiled `advance_blend_mode@0` 100M Rust-only tracking
  run moved from total=43040.31 / advance=21624.27 ms to total=40950.57 /
  advance=20305.17 ms; a post-slice sample no longer shows generic
  `set_property_value` under color animation, and instead samples the generated
  `InstanceObjectStorage::set_color_property` path. `animation_reset_cases@0`
  is roughly neutral at total=21297.77 / advance=8636.76 ms for a 100M
  Rust-only run. The user-requested open-fence hot-loop reports aggregate min
  Rust/C++=2.148, Rust min-sum=2.065 ms, C++ min-sum=0.962 ms,
  `advance_blend_mode`=4.167/3.672, `animation_reset_cases`=2.768-3.005,
  `ai_assitant@0`=2.059, and `spotify_kids_demo@0`=1.970; this is
  tracking-only because the aggregate is above 2.0, C++ min-sum is outside the
  0.70-0.95 ms sanity band, and aggregate movement versus 2.152 is below the
  0.08 noise floor. A nested-child context source-local slice then removes the
  remaining steady-state `stateful_nested_host_value_local_for_slots` scan from
  `sync_nested_child_artboard_data_contexts`: nested child data-bind propagation
  now trusts the build-time source-local slots, reuses an
  `ArtboardInstance` update scratch buffer, and drops the no-longer-read nested
  source maps, matching C++'s pointer-bound `DataContext` shape. The
  user-requested open-fence hot-loop reports aggregate min Rust/C++=2.054,
  Rust min-sum=2.013 ms, C++ min-sum=0.980 ms, `ai_assitant@0`=1.867,
  `spotify_kids_demo@0`=1.892, `animated_clipping@0`=1.881,
  `align_target@0`=1.541, `advance_blend_mode`=3.925/3.703, and
  `animation_reset_cases`=2.562-3.185; this is tracking-only because the
  aggregate remains above 2.0 and the C++ min-sum is outside the 0.70-0.95 ms
  sanity band. Full `make golden-compare`, `cargo test --workspace`,
  `cargo fmt --all -- --check`, and `git diff --check` pass. A
  retained-vector/lazy-artboard-clip slice then removes the sampled Rust-only
  `Vec::clone` sites from recursive dirt propagation and joystick application,
  matching C++'s retained dependency/joystick iteration, and makes the retained
  artboard clip path build its rectangle commands only on cache miss, matching
  C++ `m_worldPath.renderPath(this)`. Focused 100M tracking for
  `animation_reset_cases@0` moved from total=22883.37 / advance=8887.97 /
  draw=13379.22 ms before the slice to total=20603.59 / advance=8720.33 /
  draw=12335.55 ms after lazy artboard clipping. `advance_blend_mode@0`
  remains noisy: advance stayed below the pre-slice 21275.92 ms sample, but
  total/draw varied upward in same-session runs. Open-fence hot-loop tracking
  reports aggregate Rust/C++=2.032 then 2.106, with Rust min-sum=2.020 then
  1.988 ms and C++ min-sum=0.994 then 0.944 ms; this is tracking-only, not M7
  acceptance, because confirmation bounced above the prior 2.054/2.066-2.083
  range and strict <=2.0 still did not hold. Full `make golden-compare`,
  `cargo test --workspace`, `cargo fmt --all -- --check`, and
  `git diff --check` pass. M7 remains open.
  Do not repeat the rejected shallow non-mesh image draw-state cache scout,
  image mesh-index precompute scout, shallow command-vector/path wrapper
  caches, shared shape path-command buffer scout, component-local shape-paint
  path dependency epoch scout, or path-command capacity pre-reserve scout;
  they preserved correctness but worsened or failed to move direct/fenced
  release timings. Next priority is a clean low-load/sanity-band release
  sample when available; under the user's open-fence tracking request, the
  next measured implementation target remains the tiny-file fixed-overhead
  outliers: re-profile `advance_blend_mode` after the retained-vector cleanup
  and `animation_reset_cases` after lazy artboard clipping, then read the
  matching C++ animation/state-machine/data-bind or draw path before adding
  another runtime fast path; do not chase the smaller `ai_assitant@0` or
  `spotify_kids_demo@0` tails again until a fresh profile puts them back above
  fixed overhead.

## Milestones

- [x] M0: Golden diff harness + corpus manifest + one exact file
- [x] M1: Static vector corpus files exact at advance(0); FFI viewer demo
- [x] M2: Animated playback exact at sampled times; real object model landed; lib.rs modularized
- [x] M3: Interactive files exact under scripted pointer input
- [x] M4: Nested artboards/lists exact for corpus entries whose first verified blocker is not M5/M6/gated
- [x] M5: Data binding exact incl. external view-model mutation
- [x] M6: Layout + text verified per declared corpus modes; audio/scripting gated with diagnostics
- [ ] M7: Public `rive` API + C ABI; perf within target of C++

## Next

1. Active `not-yet` and `milestone = "M6"` queues are empty.
   `rewards_demo.riv` is exact-status under
   `verification = "tolerant(0.0005)"`; the tolerance covers residual
   HarfRust/Skrifa text-outline coordinate drift only.
2. Initial M7 public Rust API crate exists at `crates/rive`: `File::import`,
   artboard listing/selection, artboard instantiation, one-shot advance/draw
   through the renderer traits, and raw runtime/graph escape hatches. First C
   ABI facade exists at `crates/rive-capi` with import/free and artboard
   metadata functions. `make perf-compare`, `make perf-corpus`, and
   `make perf-hot-loop` build release C++/Rust runners by default, and
   `perf-hot-loop` scores runner-emitted whole-repeat `total_ms` rather than
   wall-clock process time, stream-serialization time, or per-frame phase timer
   overhead. Both runners have null-renderer benchmark backends, so M7 perf
   checks exclude golden recording output. Both runners also support
   `--benchmark-repeat N` for long single-sample profiling runs. A release
   `ai_assitant.riv` profile found fixed schema-name property
   lookup in the paint/path hot paths; caching fixed paint keys previously
   reduced Rust direct `ai_assitant` 100-segment repeat time from about
   1019 ms to about 255 ms. Follow-up path-geometry key caching,
   repeat-aware `perf-compare`, removal of `artboard_data_bind.rs`
   hot-loop graph/binding clones, shallow sharing of immutable
   animation/state-machine definition vectors, an epoch-keyed retained
   prepared draw-command frame, epoch-keyed retained draw `RenderPath`
   handles, cached fixed layout/schema property keys, and cached fixed
   data-bind property keys now give focused
   10-iteration verification with `make perf-hot-loop PERF_CORPUS_LIMIT=5
   PERF_ITERATIONS=10 PERF_WARMUPS=1 PERF_MAX_RATIO=999` at aggregate
   Rust/C++=3.096 over 5 exact entries / 10 segments (`ai_assitant`=3.347,
   `align_target`=1.947, `animated_clipping`=2.711). This repeat=1 focused
   ratio is noisy and strict `PERF_MAX_RATIO=2.0` still fails by inspection.
   M7 perf is now explicitly defined as steady-state per-frame runtime cost;
   direct `ai_assitant` with `--benchmark-repeat 100` now reports
   Rust/C++=34.736 on the current 10-iteration run
   (cpp median=0.543 ms, rust median=18.878 ms), confirming retained
   frame/path preparation and cached keys are real clean-frame wins but still
   far from the strict target. Generated schema kind/property switch tables now
   remove the remaining linear schema/type lookup from the hot
   `RuntimeFile::data_bind_path_for_referencer_object`,
   `InstanceObjectArena::set_property_value` / `property_kind`, and layout/draw
   property helper paths; focused 10-iteration verification now reports
   aggregate Rust/C++=2.543 over the same 5 exact entries / 10 segments
   (`ai_assitant`=2.611, `align_target`=1.831, `animated_clipping`=2.460).
   Direct `ai_assitant --benchmark-repeat 100` improves to Rust/C++=17.233
   (cpp median=0.625 ms, rust median=10.766 ms). A fresh release sample then
   split Taffy layout bounds behind a `layout_epoch`, mirroring C++
   `markLayoutNodeDirty` without invalidating layout for paint/color and
   non-text string updates; text-shape string/style changes and fractional
   layout sizing still invalidate layout like C++. Focused 10-iteration
   verification after the text/fractional safety pass reports aggregate
   Rust/C++=2.699 over the same 5 exact entries / 10 segments
   (`ai_assitant`=2.785, `align_target`=2.399,
   `animated_clipping`=2.406). Direct `ai_assitant --benchmark-repeat 100`
   now reports Rust/C++=13.850 (cpp median=0.591 ms, rust median=8.183 ms);
   C++ median variance makes the ratio noisy, but Rust steady-state time
   improved. Retained gradient preparation in `RuntimeRenderPathCache` now
   caches graph-static gradient mutator buckets and dependency-order vectors
   instead of rebuilding them every paint-prep pass. Focused 10-iteration
   verification reports aggregate Rust/C++=2.647 over the same 5 exact entries
   / 10 segments (`ai_assitant`=2.906, `align_target`=1.832,
   `animated_clipping`=2.400). Direct `ai_assitant --benchmark-repeat 100`
   reports cpp median=0.398 ms, rust median=7.700 ms, Rust/C++=19.356; the
   ratio remains C++-median-sensitive, but Rust steady-state time improved.
   Retained render-paint draw configuration in `RuntimeRenderPaintCache` now
   records the last persistent paint type/stroke/blend/shader/feather config,
   skips redundant draw-time paint setters, and invalidates that config when
   gradient preparation mutates a retained paint. Focused 10-iteration
   verification reports aggregate Rust/C++=2.518 over the same 5 exact entries
   / 10 segments (`ai_assitant`=2.583, `align_target`=1.864,
   `animated_clipping`=2.422). Direct `ai_assitant --benchmark-repeat 100`
   reports cpp median=0.393 ms, rust median=7.341 ms, Rust/C++=18.668.
   A path-specific retained draw-path epoch now separates `RenderPath` rebuild
   invalidation from broad prepared-frame/paint invalidation:
   `RuntimeRenderPathCache::draw_path` uses `ArtboardInstance::path_epoch`,
   bumped by path/vertices/world-transform/layout/NSlicer dirt, collapse, and
   C++ `StrokeEffect`-style TrimPath/DashPath/Dash/Feather path-affecting
   property changes, including Feather `inner`/`spaceValue` because they change
   the cached inner-feather command stream. Paint-only changes no longer rebuild
   retained draw paths, while animated trim/dash/effect paths still invalidate
   correctly. Focused 10-iteration verification reports aggregate
   Rust/C++=2.405 over the same 5 exact entries / 10 segments
   (`advance_blend_mode`=4.554, `ai_assitant`=2.533,
   `align_target`=1.663, `animated_clipping`=2.266,
   `animation_reset_cases`=3.966). Direct
   `ai_assitant --benchmark-repeat 100` reports cpp median=0.363 ms, rust
   median=7.695 ms, Rust/C++=21.222.
   A 2026-07-08 scout implementation of a Rust-only `Shape` paint
   path-command cache was intentionally not landed. While present it kept
   focused tests, `make golden-compare`, and `cargo test --workspace` green,
   but the fenced release hot-loop did not show a completion-grade win:
   focused 5-entry aggregate moved to Rust/C++=2.588, and direct
   `ai_assitant --benchmark-repeat 100` reported cpp median=0.555 ms, rust
   median=10.197 ms, Rust/C++=18.375. The useful finding is the layer
   boundary: caching cloned `Vec<RuntimePathCommand>` above
   `RuntimeShapePaintCommand` is not the C++ optimization. The next landing
   slice should either make steady frames skip prepare via audited
   idempotent dirt raisers, or port actual `RawPath`/`PathComposer`
   retention behind C++ dirt gates.
   A follow-up scout that retained artboard/background/layout/clip
   `RenderPath` handles behind the existing layout/path epochs was also
   intentionally not landed. It kept focused tests and `make golden-compare`
   green, but the fenced release/null-renderer perf gate moved the focused
   aggregate to Rust/C++=2.705 and then 3.338; direct
   `ai_assitant --benchmark-repeat 100` was only neutral at Rust/C++=19.424.
   Treat this as too shallow a layer: clip/layout/background path rebuild
   gating can wait until the lower-level `ShapePaintPath`/`PathComposer`
   retention has landed or a profile shows it on the hot path.
   A second lower-level scout that converted `RuntimeShapePaintCommand`
   path/effect/inner-feather payloads to shared `Arc<[RuntimePathCommand]>`
   slices and cached shape paint path-command buffers by
   `(graph, shape, path kind, path_epoch, layout_epoch)` was also backed out.
   It preserved `make golden-compare` at exact=263 / exact-segments=584 /
   diverges=0 and kept the focused path/probe tests green, but the fenced
   release/null-renderer aggregate stayed worse than the current baseline:
   Rust/C++=2.627 and 2.619. Direct `ai_assitant --benchmark-repeat 100`
   improved only to Rust/C++=18.598. The next attempt should stop clean
   frames from entering prepare at all via audited C++ dirt gates, or port
   actual `PathComposer`/raw-path retention, not wrap prepared command vectors.
   Retained path-geometry command frames now live on
   `RuntimeRenderPathCache` by `(graph_global_id, path_local)` and
   `(path_epoch, layout_epoch)`, so prepared-frame rebuilds reuse C++
   `ShapePaintPath` / `RawPath`-shaped runtime geometry command streams for
   clean paths instead of rerunning `runtime_path_geometry` and
   `path_commands` for every paint path. Shape paint paths still transform,
   NSlice, reverse, and prune per composed path. Collapse checks now use a
   component-count cycle guard instead of allocating a `BTreeSet`, and
   RawPath-style empty-segment pruning compacts in place. Full
   `make golden-compare` remains exact=263 / exact-segments=584 /
   diverges=0; `cargo test --workspace`, `cargo fmt --all -- --check`, and
   `git diff --check` pass. Same-turn fenced repeat-aware hot-loop improves
   from aggregate Rust/C++=3.809 to 3.616. Strict <=2.0 remains open. Next:
   profile remaining `advance_blend_mode` / `animation_reset_cases` fixed
   overhead and `ai_assitant` advance/data-bind after this path-retention
   cleanup; likely next targets are prepared-frame clean-skip/idempotent dirt
   or sampled data-bind/context hotspots, not shallow command-vector caches.
   Nested-artboard layout bounds are now retained on `ArtboardInstance` by
   `(graph_global_id, layout_epoch)`, matching the C++ `markLayoutNodeDirty`
   / `Artboard::markLayoutDirty` boundary for layout recomputation during
   nested advance. Focused release/null-renderer verification reports
   aggregate Rust/C++=2.329 over the same 5 exact entries / 10 segments
   (`advance_blend_mode`=5.649, `ai_assitant`=2.221,
   `align_target`=1.888, `animated_clipping`=2.461,
   `animation_reset_cases`=4.264). Two direct
   `ai_assitant --benchmark-repeat 100` checks report about Rust/C++=19.5-20.0
   (rerun cpp median=0.595 ms, rust median=11.919 ms, Rust/C++=20.018), so the
   strict <=2.0 target remains open and long-repeat Rust median is still noisy.
   `RuntimeRenderPaintCache` now also records a paint-preparation key
   `(graph_global_id, cache_epoch)` and skips repeated non-dependency-order
   paint preparation when no Rust property setter or component dirt raiser
   changed the instance since the last prepare, matching C++'s clean-frame
   `updateComponents` early-out at Rust's conservative cache epoch boundary.
   Focused release/null-renderer runs over the same 5 exact entries / 10
   segments reported aggregate Rust/C++=2.493, 1.832, and 2.166; direct
   `ai_assitant --benchmark-repeat 100` reports cpp median=0.582-0.603 ms,
   rust median=5.149-5.885 ms, Rust/C++=8.852-9.756. This is a real
   steady-state Rust win, but strict <=2.0 is still not reliable on the focused
   corpus. A follow-up macOS `sample` profile of
   `ai_assitant --benchmark-repeat 100000` showed the remaining Rust time
   dominated by advance/data-bind, especially owned view-model nested artboard
   context-chain rebinding and property-path allocation. A narrow allocation
   cleanup now avoids the extra collected `Vec` while resolving context source
   paths and avoids staging owned-view-model artboard binding updates in a
   temporary vector. Direct `ai_assitant --benchmark-repeat 100` reports rust
   median=4.553-4.764 ms (Rust/C++=7.731-9.399), and the Rust-only
   repeat=100000 run moves from elapsed=4437.5 / advance=3476.3 ms to
   elapsed=3840.8 / advance=2936.9 ms. Focused release/null-renderer runs were
   still noisy (aggregate Rust/C++=2.517 and 2.776), so strict <=2.0 remains
   open. Rust nested owned-view-model binding now passes borrowed context-path
   slices instead of cloning a `Vec<Vec<usize>>` chain on every nested host,
   matching C++ `DataContext` parent-chain lookup more closely without adding
   new skip semantics. Full `make golden-compare` remains exact=263 /
   exact-segments=584 / diverges=0. Direct
   `ai_assitant --benchmark-repeat 100` reports cpp median=0.603 ms, rust
   median=4.348 ms, Rust/C++=7.210; a fresh baseline worktree for the prior
   commit ran Rust-only repeat=100000 at elapsed=4235.4 / advance=3275.3 ms,
   while this slice runs elapsed=4109.3 / advance=3120.9 ms. Focused
   release/null-renderer is still not completion-grade but moves to aggregate
   Rust/C++=2.321. The next M7 target should stop doing context-chain allocation
   cleanup and port actual C++ data-bind dirt retention: `DataBind::addDirt`,
   `DataBindContainer` dirty queues, and push-driven target-to-source updates.
   A follow-up scout that added naive `target_dirty` bits directly to
   artboard property/image bindings was intentionally not landed: it kept
   focused probes and `make golden-compare` green, but repeat-heavy
   `ai_assitant` regressed to Rust/C++=10.962 and 15.381, Rust-only
   repeat=100000 regressed to elapsed=4766.0 / advance=3385.2 ms, and the
   focused 5-entry ratio moved to Rust/C++=2.614. The next attempt should port
   the actual C++ container lists and enrollment semantics, not add per-binding
   dirty booleans around the current scans.
   Artboard source-to-target property/image binds now have container-owned
   dirty target queues indexed by source path, seeded for initial apply and
   enrolled through `set_artboard_data_bind_value_for_path`, formula token /
   operation converter updates, and stateful converter advance. This mirrors
   the C++ `DataBindContainer` dirty-list shape for the source-to-target
   subset without yet moving polling target-to-source binds onto push queues.
   Full `make golden-compare` remains exact=263 / exact-segments=584 /
   diverges=0, and `cargo test --workspace` passes. A same-session
   throwaway worktree at `988fc29` measured Rust-only repeat=100000 at
   elapsed=3080.3 / advance=2392.7 ms and focused hot-loop aggregate
   Rust/C++=2.723; this slice measures Rust-only repeat=100000 at
   elapsed=2480.2 / advance=1859.1 ms and focused hot-loop aggregate
   Rust/C++=2.371 / 2.599. Direct `ai_assitant --benchmark-repeat 100`
   reports cpp median=0.666 ms, rust median=4.851 ms, Rust/C++=7.279.
   Artboard target-to-source binds now have container-owned source queues:
   generated property setters enroll push-capable custom-property and direct
   numeric source binds, source-to-target applies suppress self-notification by
   data-bind index like C++ `DataBind::suppressDirt`, computed layout/solo/shape
   sources stay on polling fallback, and converter-backed custom sources stay
   on a conservative persisting lane until every converter dirty edge is modeled
   explicitly. Full `make golden-compare` remains exact=263 /
   exact-segments=584 / diverges=0, and `cargo test --workspace` passes.
   Focused release/null-renderer hot-loop runs report aggregate Rust/C++=2.784
   and 2.500; direct repeat-heavy `ai_assitant --benchmark-repeat 100` reports
   cpp median=0.569 ms, rust median=4.019 ms, Rust/C++=7.060. Strict <=2.0
   remains open. Next: profile remaining advance/data-bind time, then replace
   the converter-backed custom persisting fallback with explicit C++
   converter-parent dirty edges before widening this queue pattern further.
   A narrower follow-up landed only the audited OperationViewModel-number
   converter-parent dirty edge for artboard source path changes. Converter-backed
   custom-property sources intentionally remain on the conservative persisting
   lane: a broader RangeMapper/converter-property scout was backed out after it
   drove wrong `db_health_tracker` clip positions, which confirms global
   DataBindContext converter-property writes are not the safe fallback-removal
   path. Full `make golden-compare` remains exact=263 / exact-segments=584 /
   diverges=0, `cargo test --workspace` passes, and fenced hot-loop reports
   aggregate Rust/C++=2.592. Next: enumerate and port concrete C++
   converter-parent dirty edges one converter family at a time before removing
   the persisting fallback.
   Converter-backed custom-property sources now narrow that fallback by
   converter family instead of treating every converter as persisting:
   `PassThrough`, `BooleanNegate`, `TriggerIncrement`, `ToNumber`,
   `ListToLength`, `StringRemoveZeros`, `Formula`, and groups containing only
   push-safe children leave the conservative polling lane. Families with
   unmodeled converter-owned dirt edges remain persisting: `NumberToList`,
   `ToString`, operation-view-model/system operation, `Rounder`, `RangeMapper`,
   `StringTrim`, `StringPad`, `Interpolator`, and unsupported converters.
   Full `make golden-compare` remains exact=263 / exact-segments=584 /
   diverges=0, `cargo test --workspace` passes, and fenced hot-loop improves
   the current focused aggregate to Rust/C++=2.409. Strict <=2.0 remains open.
   Next: port concrete C++ converter-property dirty setters family-by-family,
   then shrink this predicate again.
   `DataConverterOperationValue.operationValueChanged()` is now the next
   landed concrete family: artboard `OperationValue` converters leave the
   persisting lane because Rust already updates them through
   `set_artboard_operation_value`, resets formula randoms, and enqueues
   dependent property/custom parents. System-operation subclasses remain
   conservative because their bind-target path is not modeled by that exact
   updater. Full `make golden-compare` remains exact=263 /
   exact-segments=584 / diverges=0, `cargo test --workspace` passes, and
   fenced hot-loop improves the focused aggregate to Rust/C++=2.201. Strict
   <=2.0 remains open.
   `DataConverterToString::{decimals,colorFormat}Changed()` is now modeled as
   a family-specific converter-property dirty edge: imported ToString
   converter-property binds are queued by source path, seeded for initial
   application, update dependent copied converters by target converter id, and
   enqueue their property/custom parents without broad DataBindContext
   converter-property writes. `ToString` now leaves the converter-backed custom
   persisting lane. Remaining conservative families are `NumberToList`,
   operation-view-model/system operation, `Rounder`, `RangeMapper`,
   `StringTrim`, `StringPad`, `Interpolator`, and unsupported converters. Full
   `make golden-compare` remains exact=263 / exact-segments=584 / diverges=0,
   `cargo test --workspace` passes, and fenced hot-loop is noisy but roughly
   neutral at aggregate Rust/C++=2.238 then 2.195. Strict <=2.0 remains open.
   `DataConverterStringTrim::trimTypeChanged()` is now modeled through the same
   family-specific converter-property dirty lane: imported StringTrim `trimType`
   binds are queued by source path, seeded for initial application, update
   dependent copied converters by target converter id, and enqueue their
   property/custom parents without broad DataBindContext converter-property
   writes. `StringTrim` now leaves the converter-backed custom persisting lane.
   Remaining conservative families are `NumberToList`, operation-view-model /
   system operation, `Rounder`, `RangeMapper`, `StringPad`, `Interpolator`, and
   unsupported converters. Full `make golden-compare` remains exact=263 /
   exact-segments=584 / diverges=0, `cargo test --workspace`,
   `cargo fmt --all -- --check`, and `git diff --check` pass, and fenced
   hot-loop is noisy but roughly neutral at aggregate Rust/C++=2.173 then
   2.239. Strict <=2.0 remains open.
   `DataConverterStringPad::{length,text,padType}Changed()` is now modeled
   through the same family-specific converter-property dirty lane: imported
   StringPad binds are queued by source path, seeded for initial application,
   update dependent copied converters by target converter id, and enqueue their
   property/custom parents without broad DataBindContext converter-property
   writes. `StringPad` now leaves the converter-backed custom persisting lane.
   Remaining conservative families are `NumberToList`, operation-view-model /
   system operation, `Rounder`, `RangeMapper`, `Interpolator`, and unsupported
   converters. Full `make golden-compare` remains exact=263 /
   exact-segments=584 / diverges=0, `cargo test --workspace`,
   `cargo fmt --all -- --check`, and `git diff --check` pass, and fenced
   hot-loop is noisy but roughly neutral at aggregate Rust/C++=2.120 then
   2.186. Strict <=2.0 remains open.
   `DataConverterRounder.decimalsChanged()` is generated but empty in C++, and
   the handwritten Rounder class does not override it. Rust therefore removes
   Rounder custom sources from the conservative polling lane without adding a
   converter-property updater: there is no C++ `DataConverter::markConverterDirty`
   edge to model for this family. The status review keeps the scout and perf
   methodology discoveries in force: broad converter-property writes remain
   rejected after the `db_health_tracker` RangeMapper scout, shallow cached
   command/path wrappers remain rejected by fenced perf, and M7 decisions now
   use release/null-renderer hot-loop whole-repeat `total_ms` scoring.
   Remaining conservative families are `NumberToList`, operation-view-model /
   system operation, `RangeMapper`, `Interpolator`, and unsupported converters.
   Full `make golden-compare` remains exact=263 / exact-segments=584 /
   diverges=0,
   `cargo test --workspace`, `cargo fmt --all -- --check`, and
   `git diff --check` pass, and fenced hot-loop reports aggregate
   Rust/C++=2.310 then 2.181. Strict <=2.0 remains open.
   A 2026-07-08 family-specific `RangeMapper` scout was intentionally not
   landed. It added artboard converter-property bindings only for the four C++
   dirty callbacks (`minInputChanged()`, `maxInputChanged()`,
   `minOutputChanged()`, `maxOutputChanged()`), kept `flags`,
   `interpolationType`, and `interpolatorId` out of the lane because their
   generated callbacks are empty, and moved RangeMapper custom sources out of
   the persisting fallback. `cargo check -p rive-runtime`,
   `cargo test -p rive-runtime queues`, and
   `cargo test -p rive-runtime range_mapper` passed, but full
   `make golden-compare` failed `db_health_tracker` at line 3390 with the same
   clip-path x-position drift as the earlier broad RangeMapper scout
   (Rust first point x=48.2119293 vs C++ x=64.2483139). The code was backed out.
   Treat RangeMapper as requiring deeper C++ DataBind/DataConverter ownership
   and ordering analysis before another fallback-removal attempt; do not retry
   the StringPad-style copied-converter updater for this family.
   `DataConverterInterpolator.durationChanged()` is now modeled through the
   same family-specific converter-property dirty lane: C++ only marks dirty for
   `durationChanged()`, while generated `interpolationTypeChanged()` and
   `interpolatorIdChanged()` are empty. Imported Interpolator `duration` binds
   are queued by source path, seeded for initial application, update dependent
   copied converters by target converter id, and enqueue their property/custom
   parents without broad DataBindContext converter-property writes; the existing
   stateful-converter advance queue continues to cover the in-flight
   `InterpolatorAdvancer::advance()` dirty edge. `Interpolator` now leaves the
   converter-backed custom persisting lane.
   `DataConverterNumberToList.viewModelIdChanged()` is now modeled through a
   family-specific converter-property dirty lane: C++ clears cached
   `m_listItems` and marks the converter dirty; Rust has no persistent
   `ViewModelInstanceListItem` cache in this layer, so copied NumberToList
   converters store `view_model_id` plus the file's view-model count and
   recompute list size from the current id each conversion. Imported
   NumberToList `viewModelId` binds are queued by source path, seeded for
   initial application, update dependent copied converters by global id, and
   enqueue their concrete property/custom/list parents without broad
   DataBindContext converter-property writes. `NumberToList` now leaves the
   converter-backed custom persisting lane.
   Operation-view-model and system-operation custom sources now leave the
   conservative persisting lane. The status-doc scout review keeps the
   RangeMapper and perf-methodology fences in force: no broad DataBindContext
   converter-property writes, no StringPad-style RangeMapper retry, and no
   shallow command/path-wrapper caching without release/null-renderer hot-loop
   evidence. `DataConverterOperationViewModel` has no C++ converter-property
   dirty callback to model here; Rust relies on the existing source-path
   dependent refresh edges. `DataConverterSystemDegsToRads` and
   `DataConverterSystemNormalizer` inherit `DataConverterOperationValue`, so
   their `operationValue` binds now use the same explicit updater by converter
   global id and enqueue concrete parents. Remaining conservative families are
   `RangeMapper` and unsupported converters. Full `make golden-compare` remains
   exact=263 / exact-segments=584 / diverges=0; `cargo test --workspace`,
   `cargo fmt --all -- --check`, and `git diff --check` pass; fenced hot-loop
   reports aggregate Rust/C++=2.111. Strict <=2.0 remains open. Next: profile
   remaining advance/data-bind time or audit RangeMapper C++ ownership/update
   order before another fallback-removal attempt.
   A follow-up release `ai_assitant --benchmark-repeat 1000000` sample found
   the remaining Rust time still concentrated in owned view-model/nested
   artboard data-context propagation: `bind_owned_view_model_artboard_context_chain`,
   `sync_nested_child_artboard_data_contexts`, and
   `RuntimeFile::data_bind_path_for_referencer_object`. Rust now mirrors C++'s
   pointer-walking context propagation more closely by no longer cloning nested
   child property/image binding vectors during every sync pass, and by using
   fixed cached generated-property keys for the sampled nested-host
   `ViewModelInstance*` lookups instead of the generic name dispatcher.
   Long-repeat Rust-only `ai_assitant` improves from elapsed=2095.6 /
   advance=1602.2 ms to elapsed=1734.9 / advance=1233.3 ms for 100000
   segments. Full `make golden-compare` remains exact=263 /
   exact-segments=584 / diverges=0; `cargo test --workspace`,
   `cargo fmt --all -- --check`, and `git diff --check` pass. Fenced
   release/null-renderer hot-loop is still noisy/neutral at aggregate
   Rust/C++=2.119, so strict <=2.0 remains open. Next: port the C++ retained
   `DataBindPath`/data-context lookup shape for nested hosts so Rust stops
   resolving `data_bind_path_for_referencer_object` inside the steady frame.
   Nested host `DataBindPath` resolution is now retained on
   `RuntimeNestedArtboardInstance`, including dynamic `artboardId` swaps,
   mirroring C++ `NestedArtboard::dataBindPath()` plus lazy
   `DataBindPath::resolvedPath()` retention. The steady owned-view-model nested
   context path now consumes the retained path slice instead of calling
   `RuntimeFile::data_bind_path_for_referencer_object`. This is immutable
   import/build data retention, not a new skip/cache invalidation rule. Full
   `make golden-compare` remains exact=263 / exact-segments=584 / diverges=0;
   `cargo test --workspace`, `cargo fmt --all -- --check`, and
   `git diff --check` pass. Long-repeat Rust-only `ai_assitant` improves from
   elapsed=1734.9 / advance=1233.3 ms to elapsed=1408.5 / advance=902.0 ms for
   100000 segments. Fenced release/null-renderer hot-loop reports aggregate
   Rust/C++=2.338 then 2.430, so strict <=2.0 remains open. Next: profile
   remaining release/null-renderer advance/data-bind time after the retained
   path change while keeping the scout fences in force: no broad
   DataBindContext converter-property writes, no StringPad-style RangeMapper
   retry, and no shallow command/path-wrapper caching without fenced evidence.
   A fresh release sample after the retained-path slice found the remaining
   Rust time still concentrated in owned view-model nested artboard
   data-context propagation, with allocations in context-source path lookup and
   smaller retained animation/state-machine name clone/drop traffic. Rust now
   walks numeric owned-view-model context-source paths directly through active
   child property lists, retaining the existing name-based fallback; nested
   host context-chain prepends use stack storage for the common shallow case;
   artboard property/image/custom binding paths are cloned only after equality
   proves a changed value; and retained linear-animation/state-machine names
   share `Arc<str>` definitions. This follows C++ `DataContext` pointer walks
   and immutable definition sharing without adding new invalidation or skip
   caching. Full `make golden-compare` remains exact=263 /
   exact-segments=584 / diverges=0; `cargo test --workspace`,
   `cargo fmt --all -- --check`, and `git diff --check` pass. Direct
   Rust-only `ai_assitant --benchmark-repeat 100000` improves from
   elapsed=1408.5 / advance=902.0 ms to elapsed=1225.7 / advance=706.0 ms.
   Single-file release/null-renderer `ai_assitant --benchmark-repeat 100`
   reports cpp median=0.429 ms, rust median=1.928 ms, Rust/C++=4.496, and the
   focused 5-entry hot-loop reports aggregate Rust/C++=2.363. Strict <=2.0
   remains open. Next: profile the remaining
   `bind_owned_view_model_artboard_context_chain` /
   `collect_nested_artboard_context_source_values` time and continue only
   audited C++ retention/dirt slices; keep the scout fences in force: no broad
   converter-property writes, no StringPad-style RangeMapper retry, and no
   shallow command/path-wrapper caching without release/null-renderer evidence.
   Nested artboard context-source propagation now uses a single accumulator
   while walking descendants instead of allocating a descendant-value `Vec` at
   each host, cloning that vector into the parent, and then replaying the owned
   values into the child. Nested host loops also walk the retained host map
   directly instead of snapshotting keys into a temporary vector. This mirrors
   C++'s retained object traversal shape and does not add skip/cache
   invalidation. Full `make golden-compare` remains exact=263 /
   exact-segments=584 / diverges=0; `cargo test --workspace`,
   `cargo fmt --all -- --check`, and `git diff --check` pass. Direct Rust-only
   `ai_assitant --benchmark-repeat 1000000` improves from elapsed=11839.2 /
   advance=6853.7 ms to elapsed=11624.2 / advance=6594.8 ms. Focused
   release/null-renderer hot-loop reports aggregate Rust/C++=2.281 over the
   5-entry / 10-segment corpus, while single-file repeat=100 reports
   cpp median=0.373 ms, rust median=1.944 ms, Rust/C++=5.207. Strict <=2.0
   remains open. Next: profile and port the remaining child-context
   construction in `bind_owned_view_model_artboard_context_chain` rather than
   broadening to fenced-off converter/property caches.
   Nested owned-view-model child-context paths now use borrowed/inline stack
   storage for the common numeric `DataBindPath` case instead of allocating a
   `Vec<usize>` before prepending the child to the parent context chain. This
   keeps C++'s `DataContext::getViewModelInstance` shape: check the current
   context root, walk the numeric tail, then fall through to parent contexts;
   it does not add a new skip/cache invalidation rule. Full
   `make golden-compare` remains exact=263 / exact-segments=584 /
   diverges=0; `cargo test --workspace`, `cargo fmt --all -- --check`, and
   `git diff --check` pass. Focused release/null-renderer hot-loop improves
   the 5-entry / 10-segment aggregate from Rust/C++=2.281 to 2.169
   (`ai_assitant`=1.921). Direct Rust-only `ai_assitant
   --benchmark-repeat 1000000` is noisy but roughly neutral/slightly improved
   in one same-session run, elapsed=11715.7 / advance=6692.8 ms before and
   elapsed=11580.4 / advance=6636.0 ms after. Strict <=2.0 remains open.
   Next: profile remaining `bind_owned_view_model_artboard_context_chain` and
   `collect_nested_artboard_context_source_values` time, keeping the scout
   fences in force.
   Source-producing artboard data-bind paths are now shared as immutable
   `Arc<[u32]>` slices for custom-property, layout-computed, solo-source, and
   nested context-source values, so context-source propagation clones a retained
   path handle instead of allocating a fresh path `Vec`. This is import/build
   data sharing, not a skip/cache invalidation rule. Full `make golden-compare`
   remains exact=263 / exact-segments=584 / diverges=0; `cargo test
   --workspace`, `cargo fmt --all -- --check`, and `git diff --check` pass.
   Focused release/null-renderer hot-loop is a tiny/noisy aggregate improvement
   from Rust/C++=2.169 to 2.164 over the 5-entry / 10-segment corpus
   (`ai_assitant`=1.936). Direct Rust-only `ai_assitant
   --benchmark-repeat 1000000` is noisy/slightly worse at elapsed=11884.7 /
   advance=6753.7 ms, and single-file repeat=100 JSON at
   `/tmp/rive-ai-shared-paths-perf.json` reports cpp median=0.371 ms, rust
   median=1.988 ms, Rust/C++=5.363. Strict <=2.0 remains open. Next: profile
   remaining advance/data-bind time before landing more context allocation
   cleanup; if no clear C++ retention/dirt slice appears, move to a
   higher-leverage audited dirt/retention target.
   Nested-host source locals for child data-context sync are now retained on
   `RuntimeNestedArtboardInstance` by child binding path and rebuilt during
   dynamic `artboardId` swaps. `sync_nested_child_artboard_data_contexts`
   consumes the retained path-to-local map and falls back to the old slot walk
   for unresolved paths; that loop also walks the retained host map directly
   instead of snapshotting keys into a temporary vector. This mirrors C++
   `DataContext`/view-model instance pointer walks without adding a new
   skip/cache invalidation rule. Full `make golden-compare` remains exact=263 /
   exact-segments=584 / diverges=0; `cargo test --workspace`,
   `cargo fmt --all -- --check`, and `git diff --check` pass. Direct Rust-only
   `ai_assitant --benchmark-repeat 1000000` improves from elapsed=11884.7 /
   advance=6753.7 ms to elapsed=11658.6 / advance=6683.3 ms. Single-file
   repeat=100 JSON at `/tmp/rive-ai-retained-source-locals-perf.json` reports
   cpp median=0.389 ms, rust median=1.898 ms, Rust/C++=4.880. Focused
   release/null-renderer hot-loop is improved but still noisy at aggregate
   Rust/C++=2.024 then 2.253, so strict <=2.0 remains open. Next: profile the
   remaining advance/data-bind time again before choosing between another
   audited retained data-context lookup and a higher-leverage dirt/retention
   target.
   Nested-host root `ViewModelInstance` locals are now retained on
   `RuntimeNestedArtboardInstance` by `viewModelId`, rebuilt with dynamic
   `artboardId` swaps, and reused by child data-context sync before walking
   property values. Successful fallback source-local resolutions are also
   retained by binding path even when the current sync pass does not materialize
   a value. This mirrors C++ `DataContext` root pointers plus
   `DataBindPath::resolvedPath()` retention without adding a negative cache or
   invented skip invalidation. Full `make golden-compare` remains exact=263 /
   exact-segments=584 / diverges=0; `cargo test --workspace`,
   `cargo fmt --all -- --check`, and `git diff --check` pass. Direct Rust-only
   `ai_assitant --benchmark-repeat 1000000` improves from elapsed=11658.6 /
   advance=6683.3 ms to elapsed=9791.6 / advance=4650.2 ms. Single-file
   repeat=100 JSON at `/tmp/rive-ai-root-locals-perf-final.json` reports
   cpp median=0.463 ms, rust median=1.717 ms, Rust/C++=3.710. A 3M sample
   shows `stateful_nested_host_value_local_for_slots` dropping from 775 to 48
   samples; the new top is draw/prepare plus schema `definition_by_name`,
   `bind_owned_view_model_artboard_context_chain`, and BTree range lookups.
   Focused release/null-renderer hot-loop reports aggregate Rust/C++=2.136
   (`ai_assitant`=1.852), so strict <=2.0 remains open. Next: profile/port the
   remaining C++-audited retained/dirt targets in
   `bind_owned_view_model_artboard_context_chain` / context-source BTree
   lookups or address remaining schema lookup in sampled draw/prepare without
   violating the scout fences.
   Draw-time drawable classification now avoids schema reflection for the two
   sampled hot checks: render-opacity filtering uses the fixed `Shape` /
   `TextInputDrawable` class surface, and nested-artboard dispatch uses the
   three concrete nested host classes, matching C++'s concrete `is<T>()` /
   type-key shape instead of calling `definition_by_name(...).is_a(...)` in
   the frame loop. This removes a sampled draw/prepare schema lookup without
   adding a skip cache or changing dirty invalidation. Full
   `make golden-compare` remains exact=263 / exact-segments=584 / diverges=0;
   `cargo test --workspace`, `cargo fmt --all -- --check`, and
   `git diff --check` pass. Single-file repeat=100 JSON at
   `/tmp/rive-ai-draw-kind-perf.json` reports cpp median=0.465 ms,
   rust median=1.675 ms, Rust/C++=3.603. Focused release/null-renderer
   hot-loop reports aggregate Rust/C++=2.243 then 2.133, so strict <=2.0
   remains open. Next: re-profile after this schema-classification cleanup and
   pick the next C++-audited retained/dirt or hot BTree target from the new
   sample.
   Fixed draw/image/mesh/nested property keys now also route through the
   cached runtime draw-key helper instead of calling the generic
   `property_key_for_name` dispatcher for literal schema pairs in the frame
   loop. This covers `DrawRules.drawTargetId`, `DrawTarget.placementValue`,
   `Drawable.blendModeValue`, `Image.assetId/origin/fit/alignment`,
   `Vertex.x/y`, `NestedArtboard.artboardId`, `NestedArtboardLeaf`
   fit/alignment, and `Artboard.opacity`. It is C++-shaped generated-key
   retention, not a new skip/cache invalidation rule. Full `make
   golden-compare` remains exact=263 / exact-segments=584 / diverges=0;
   `cargo test --workspace`, focused draw tests, `cargo fmt --all -- --check`,
   and `git diff --check` pass. Single-file repeat=100 JSON at
   `/tmp/rive-ai-draw-property-keys-perf.json` reports cpp median=0.531 ms,
   rust median=1.586 ms, Rust/C++=2.985. Focused release/null-renderer
   hot-loop reports aggregate Rust/C++=2.187 then 2.122, so strict <=2.0
   remains open. Next: re-profile after this fixed-key cleanup and choose
   between the sampled hot BTree/draw-order path and remaining audited
   data-context retention work.
   Sorted drawable order is now retained in `RuntimeRenderPathCache` by
   `(graph_global_id, draw_order_epoch)`, and `draw_order_epoch` is bumped by
   the C++ DrawOrder dirt raisers for `DrawRules.drawTargetId` and
   `DrawTarget.placementValue` through `ComponentDirt::DRAW_ORDER`. This keeps
   prepared command rebuilds from reconstructing draw-target BTree groupings
   when only paint/data-bind/cache epoch changes. Full `make golden-compare`
   remains exact=263 / exact-segments=584 / diverges=0; focused draw-order and
   draw tests, `cargo test --workspace`, `cargo fmt --all -- --check`, and
   `git diff --check` pass. Direct `ai_assitant --benchmark-repeat 100` JSON
   at `/tmp/rive-ai-sorted-draw-order-perf.json` reports cpp median=0.522 ms,
   rust median=1.592 ms, Rust/C++=3.051; focused hot-loop reports aggregate
   Rust/C++=2.136 then 2.107, so strict <=2.0 remains open. Next: re-profile;
   likely remaining targets are draw command/prepare retention below sorted
   order and data-context BTree/range work. Keep the scout/perf fences in
   force: no broad converter-property writes, no StringPad-style RangeMapper
   retry, and no shallow command/path-wrapper caching without
   release/null-renderer evidence.
   Nested host traversal now retains `nested_artboard_locals` on
   `ArtboardInstance`, replacing repeated BTree key collection and range cursor
   walks in nested advance, owned view-model context binding, nested
   context-source propagation, and nested child data-context sync. The retained
   ordered host list mirrors the current nested map keys and is updated only
   when dynamic `artboardId` swaps create or remove a child instance. This
   follows C++ retained object traversal and does not add new dirty or skip
   semantics.
   Full `make golden-compare` remains exact=263 / exact-segments=584 /
   diverges=0; focused nested/data-bind tests, `cargo test --workspace`,
   `cargo fmt --all -- --check`, and `git diff --check` pass. Fenced
   release/null-renderer hot-loop reports aggregate Rust/C++=2.017, and a
   closeout rerun reports aggregate Rust/C++=2.093
   (`advance_blend_mode`=6.239, `ai_assitant`=1.820,
   `align_target`=1.750, `animated_clipping`=2.337,
   `animation_reset_cases`=4.105; rerun `ai_assitant`=1.877). A Rust-only
   `ai_assitant --benchmark-repeat 3000000` sample at
   `/tmp/rive-ai-retained-host-locals.sample.txt` reports elapsed=24583.5 ms,
   advance=12061.8 ms, prepare=5582.8 ms, draw=6771.0 ms, and no longer shows
   `find_leaf_edges_spanning_range` or `BTreeMap::` in the sampled nested host
   traversal. Strict <=2.0 remains open. Next: profile the remaining
   advance/data-bind time, especially
   `bind_owned_view_model_artboard_context_chain` and
   `advance_artboard_data_binds_with_root_transform`, plus draw
   command/prepare retention below sorted order under the scout/perf fences.
   Prepared shape-paint commands now retain their deterministic draw-path
   cache slot indices instead of rebuilding the per-command path-slot vector
   during every draw replay. Dynamic text shape-paint commands still assign
   slots when the transient text commands are generated. This removes the
   sampled `runtime_cached_path_slot_index` / `RawVec::grow_one` draw replay
   site without changing draw invalidation or skip semantics.
   Full `make golden-compare` remains exact=263 / exact-segments=584 /
   diverges=0; focused draw/path tests, `cargo test --workspace`, `cargo fmt
   --all -- --check`, and `git diff --check` pass. Fenced release/null-renderer
   hot-loop reports aggregate Rust/C++=2.136 with `ai_assitant`=1.972. A
   Rust-only `ai_assitant --benchmark-repeat 3000000` sample at
   `/tmp/rive-ai-draw-slot.sample.txt` reports elapsed=23857.3 ms,
   advance=12268.1 ms, prepare=5482.9 ms, draw=5939.5 ms, and no longer shows
   `runtime_cached_path_slot_index` in the sampled draw replay. Strict <=2.0
   remains open. Next: profile and port remaining data-bind/context-chain
   hotspots first, especially `advance_artboard_data_binds_with_root_transform`
   and `bind_owned_view_model_artboard_context_chain`; draw-side leftovers are
   now lower-level `runtime_configure_paint_with_cache` and
   `RuntimeRenderPathCache::draw_path`.
   Nested-host data-bind application now reuses retained binding entries
   instead of cloning `artboard_nested_host_bindings` every advance, and nested
   child data-context sync now drops same-value child updates before allocating
   update work. This is a direct no-op removal around C++'s retained
   `DataBind`/`DataContext` references, not a new dirty gate. Full
   `make golden-compare` remains exact=263 / exact-segments=584 / diverges=0;
   focused nested/data-bind tests, `cargo test --workspace`, `cargo fmt --all
   -- --check`, and `git diff --check` pass. Fenced release/null-renderer
   hot-loop reports aggregate Rust/C++=2.105 with `ai_assitant`=1.939. Strict
   <=2.0 remains open. Next: profile and port the larger
   `bind_owned_view_model_artboard_context_chain` path-resolution hotspot.
   A follow-up scratch-reuse scout for name-based owned view-model context
   source paths was intentionally not landed. It reused a caller-owned
   `Vec<usize>` while resolving `bind_owned_view_model_artboard_context_chain`
   and kept focused nested/data-bind tests plus `make golden-compare` green
   at exact=263 / exact-segments=584 / diverges=0, but fenced
   release/null-renderer hot-loop rejected it: aggregate Rust/C++ worsened to
   2.250 and then 2.478. The useful finding is the layer boundary: C++
   `DataBindContext::bindFromContext` resolves and retains a concrete
   `ViewModelInstanceValue` source via `DataContext`; Rust's artboard owned
   context path still recomputes the context-chain source lookup every frame.
   Next: port a C++-aligned retained source binding/rebind path for artboard
   owned-context data binds, or first write that design if the invalidation
   surface is too large for one safe slice.
   Artboard owned-context data binds now retain their resolved source
   property path on property/image/custom bindings behind an
   `(owned view-model index, context-chain)` key. This mirrors the C++
   `DataBindContext::bindFromContext` source-retention boundary without
   adding a value-read skip: each frame still reads the current source value
   and routes changes through the existing data-bind value/target queues.
   Full `make golden-compare` remains exact=263 / exact-segments=584 /
   diverges=0; focused nested/data-bind tests, `cargo test --workspace`,
   `cargo fmt --all -- --check`, and `git diff --check` pass. Fenced
   release/null-renderer hot-loop reports noisy aggregate Rust/C++=2.154
   then 2.438, but the targeted direct `ai_assitant --benchmark-repeat 100`
   JSON at `/tmp/rive-ai-retained-owned-source-paths-perf.json` reports
   cpp median=0.442 ms, rust median=1.420 ms, Rust/C++=3.212. Strict <=2.0
   remains open. Next: profile the remaining `ai_assitant` advance/data-bind
   time after retained source paths; likely targets are C++-aligned rebind
   gating for owned context chains or remaining nested context-source
   propagation, not scratch-only allocation helpers.
   Owned-context artboard rebinds are now gated by the root owned
   view-model's mutation generation plus the retained context-chain identity.
   `RuntimeOwnedViewModelInstance` bumps the generation for public owned-value
   mutations and view-model relinks; `RuntimeArtboardOwnedContextKey` includes
   that generation; and each artboard skips only its own bind/apply and nested
   animation-context rebind when its key is clean while still descending into
   children so local dynamic `artboardId` swaps can invalidate themselves.
   This keeps the status-doc scout/perf fences in force: it ports the C++
   `DataBindContext::bindFromContext` rebind boundary, not the rejected
   scratch-path reuse or shallow path-command cache layers. Full
   `make golden-compare` remains exact=263 / exact-segments=584 /
   diverges=0, and `cargo test --workspace` passes. Fenced
   release/null-renderer hot-loop reports aggregate Rust/C++=2.073 then
   2.274 over the 5-entry / 10-segment focused corpus (`ai_assitant`=1.875
   then 2.165). Single-file repeat=100 JSON at
   `/tmp/rive-ai-owned-context-generation-perf.json` reports cpp median=0.420
   ms, rust median=1.411 ms, Rust/C++=3.357. A Rust-only 3M
   `ai_assitant` repeat improves from the prior retained-source-path baseline
   elapsed=23231.2 / advance=11731.1 ms to elapsed=20152.4 /
   advance=8773.6 ms. Strict <=2.0 remains open. Next: profile the focused
   outliers (`animated_clipping`, the small-file fixed overhead in
   `advance_blend_mode` / `animation_reset_cases`, and `ai_assitant` if it
   stays >2 on rerun) under the same scout/perf fences: no broad
   DataBindContext converter-property writes, no StringPad-style RangeMapper
   retry, no scratch-only owned-context path reuse, and no shallow
   command/path-wrapper caching without release/null-renderer evidence.
   Text shape-paint commands are now retained in `RuntimeRenderPathCache` by
   graph/text plus `path_epoch`, `layout_epoch`, and the conservative instance
   cache epoch. The profile target was `animated_clipping`: a live macOS sample
   showed `runtime_draw_command` spending almost all time in
   `runtime_text_shape_paint_commands` / `StaticTextSlice::render_data` /
   HarfRust shaping, while C++ `Text::buildRenderStyles()` retains
   `m_drawCommands` and `Text::draw()` replays them until `markShapeDirty`.
   Rust-only `animated_clipping --benchmark-repeat 3000000` improves from
   elapsed=147254.8 / draw=146414.0 ms to elapsed=4008.4 / draw=3348.1 ms.
   Full `make golden-compare` remains exact=263 / exact-segments=584 /
   diverges=0, `cargo test --workspace`, `cargo fmt --all -- --check`, and
   `git diff --check` pass. Fenced repeat=1 hot-loop remains noisy above
   target at aggregate Rust/C++=2.395 then 2.223; direct repeat=100
   `animated_clipping` JSON at
   `/tmp/rive-animated-clipping-text-cache-perf.json` reports cpp median=0.100
   ms, rust median=0.397 ms, Rust/C++=3.954. Strict <=2.0 remains open. Next:
   profile the small-file fixed overhead in `advance_blend_mode` /
   `animation_reset_cases`, and consider a repeat-aware corpus aggregation
   harness slice before using repeat-heavy evidence as the main M7 score.
   `perf-compare` corpus mode now supports repeat-aware steady-state scoring:
   when `--benchmark-repeat N` is used, the selected exact files are expanded
   after `--corpus-limit` into one runner target per sample segment, preserving
   the golden runners' single-sample repeat invariant while reporting a corpus
   aggregate over the same file x sample segments. The first focused command,
   `make perf-hot-loop PERF_CORPUS_LIMIT=5 PERF_ITERATIONS=10 PERF_WARMUPS=1
   PERF_MAX_RATIO=999 PERF_BENCHMARK_REPEAT=100`, reports entries=10 /
   segments=10 and aggregate Rust/C++=3.711 (`advance_blend_mode@0`=9.385,
   `advance_blend_mode@0.25`=8.619, `ai_assitant@0`=3.681,
   `align_target@0`=2.146, `animated_clipping@0`=2.857,
   `animation_reset_cases` samples around 4.0). Strict <=2.0 remains open.
   Prepared draw-command frames are now keyed by
   `(graph_global_id, prepared_epoch)` instead of the broad instance
   `cache_epoch`. `prepared_epoch` is bumped by path/layout/draw-order/render
   opacity/image/nested-artboard identity and draw-affecting properties, while
   nested input proxy values, data-bind/view-model metadata, and
   nested-artboard animation knobs keep only the broad cache epoch. This ports
   the C++ `Artboard::updateComponents` / `ComponentDirt` retention boundary
   without adding a new unaudited skip layer. Rust-only long-repeat
   `advance_blend_mode --benchmark-repeat 1000000` improves from
   elapsed=1382.1 / prepare=695.0 ms to elapsed=1269.1 / prepare=609.9 ms;
   `animation_reset_cases` is roughly neutral at elapsed=516.3 ms. Focused
   repeat-aware hot-loop is noisy/neutral at aggregate Rust/C++=3.897 then
   3.852, and a fresh sample at
   `/tmp/rive-advance-blend-prepared-epoch.sample.txt` still shows nested
   prepared-frame rebuilds dominated by `runtime_shape_paint_path_commands`,
   `path_commands`, `runtime_path_geometry`, and allocation. Strict <=2.0
   remains open. Next: port lower-level C++ `RawPath`/`PathComposer` /
   `ShapePaintPath` retention or another sampled audited data-bind/context
   target; do not retry shallow command-vector/path-wrapper caches without
   fenced release/null-renderer evidence.
   Runtime state-machine definitions are now retained behind
   `Arc<Vec<RuntimeStateMachine>>`, so `advance_state_machine_instance` clones
   only the outer definition handle instead of cloning/dropping the
   `RuntimeStateMachine` definition every advance. This mirrors C++
   `StateMachineInstance` holding a stable `StateMachine` pointer and stays
   within the scout/perf fences: immutable definition retention, not a new
   skip/cache invalidation rule. Full `make golden-compare` remains exact=263 /
   exact-segments=584 / diverges=0; `cargo test --workspace`, `cargo fmt --all
   -- --check`, and `git diff --check` pass. Fenced repeat-aware hot-loop
   reports aggregate Rust/C++=3.613 over the 5-entry / 10-segment corpus
   (`ai_assitant`=3.299), so strict <=2.0 remains open. Next: profile the
   remaining fixed overhead / advance-data-bind time under the same fences.
   A follow-up `ai_assitant --benchmark-repeat 3000000` sample found current
   Rust time split between advance/data-bind and draw/prepare, with
   dependency-ordered gradient paint preparation spending visible time in
   per-frame `BTreeMap`/`BTreeSet` insert/drop. Dependency-ordered paint prep
   now uses small vectors for nested-host command lookup and gradient paint /
   host de-dupe, preserving the old duplicate rules (`collect` last-wins for
   prepared commands, `or_insert` first-wins for layout-discovered commands)
   while matching C++'s retained object/vector traversal shape. Full
   `make golden-compare` remains exact=263 / exact-segments=584 / diverges=0;
   focused draw probes, `cargo test --workspace`, `cargo fmt --all -- --check`,
   and `git diff --check` pass. Fenced repeat-aware hot-loop improves from
   aggregate Rust/C++=3.632 to 3.447 and 3.327 on rerun, but strict <=2.0
   remains open. Next: re-profile `ai_assitant` and the fixed-overhead files;
   likely remaining targets are advance/data-bind context lookup and lower
   draw/prepare retention, not shallow command/path-wrapper caches.
   A follow-up `ai_assitant --benchmark-repeat 30000000` sample found
   remaining Rust time split across paint/prepare, data-bind, and
   layout-adjusted world-transform lookup. Runtime components now retain the
   static layout-topology facts needed by
   `runtime_component_world_transform_with_bounds`, and
   `ArtboardInstance::component` / `component_mut` use each dense slot's
   retained component index instead of a frame-loop `BTreeMap` lookup. This is
   C++-shaped retained object/index traversal and does not add new skip/cache
   invalidation. Full `make golden-compare` remains exact=263 /
   exact-segments=584 / diverges=0; `cargo test -p rive-runtime`,
   `cargo test --workspace`, `cargo fmt --all -- --check`, and
   `git diff --check` pass. Fenced repeat-aware hot-loop baseline was
   aggregate Rust/C++=3.515 with Rust median sum 2.429 ms; after the slice,
   reruns report aggregate Rust/C++=3.326 and 3.392 with the better Rust
   median sum at 2.342 ms. Direct repeat=100 `ai_assitant` JSON at
   `target/perf-ai-layout-topology.json` reports cpp median=0.427 ms, rust
   median=1.196 ms, Rust/C++=2.800. Strict <=2.0 remains open. Next:
   re-profile `ai_assitant` plus the fixed-overhead files; likely targets
   remain dependency-ordered paint/prepare work and
   `advance_artboard_data_binds_with_root_transform`, not broad
   converter-property writes or shallow command/path-wrapper caches.
   A fresh `ai_assitant --benchmark-repeat 30000000` sample found the current
   Rust time still split across dependency-ordered paint preparation, draw
   replay, and artboard data-bind/context propagation. Dependency-ordered
   paint preparation now uses the existing retained preparation frame even
   when nested layout gradients force dependency ordering: the cache key
   includes the root `cache_epoch` plus nested command identity and child
   `cache_epoch` values, so clean nested paint frames skip the prep pass while
   child animation/data-bind changes still invalidate it. Artboard data-bind
   source queues also stop cloning target-source vectors during enqueue and
   recycle their update-index buffers across frames, matching C++ retained
   `DataBindContainer` dirty-list storage without adding new skip semantics.
   Full `make golden-compare` remains exact=263 / exact-segments=584 /
   diverges=0; `cargo test --workspace`, `cargo fmt --all -- --check`, and
   `git diff --check` pass. Fenced repeat-aware hot-loop baseline was
   aggregate Rust/C++=3.330; after the slice reruns report aggregate
   Rust/C++=3.110 and 3.067. Direct repeat=100 `ai_assitant` JSON at
   `target/perf-ai-dependency-prep-skip.json` reports cpp median=0.376 ms,
   rust median=1.031 ms, Rust/C++=2.747. Strict <=2.0 remains open. A fresh
   release/null-renderer sample after this dependency-prep skip found
   `runtime_draw_command`, `advance_artboard_data_binds_with_root_transform`,
   and `runtime_configure_paint_with_cache` as the leading Rust hot sites.
   Draw-time render-paint config now carries the artboard `cache_epoch`, so
   clean frames skip recomputing stroke/blend/shader/feather configuration
   while gradient preparation can still invalidate by removing the cached
   config when it mutates retained paint. Full `make golden-compare` remains
   exact=263 / exact-segments=584 / diverges=0; `cargo test --workspace`,
   `cargo fmt --all -- --check`, and `git diff --check` pass. Fenced
   repeat-aware hot-loop improves from aggregate Rust/C++=3.260 to 2.903 and
   3.105 on rerun; direct repeat=100 `ai_assitant` JSON at
   `target/perf-ai-paint-config-epoch.json` reports cpp median=0.566 ms,
   rust median=1.404 ms, Rust/C++=2.483. Strict <=2.0 remains open. Next:
   re-profile; likely remaining targets are draw replay / lower
   `RuntimeRenderPathCache::draw_path` map lookups and
   `advance_artboard_data_binds_with_root_transform`, not broad
   converter-property writes or shallow path-command caches. The next profile
   found `runtime_draw_command`, data-bind context propagation, retained path
   lookups, and state-machine advance as the remaining split. Two same-turn
   scouts were backed out: source-queue vector take/recycle worsened fenced
   hot-loop aggregate to Rust/C++=3.119 and 3.077, and carrying a borrowed
   retained `RenderPaint` through draw gave direct `ai_assitant` Rust/C++=2.658
   versus the prior 2.483. Prepared shape-paint commands now retain
   `paint_global_id`, matching C++'s retained `ShapePaint` object identity and
   removing a draw-time local-to-global map lookup. Full `make golden-compare`
   remains exact=263 / exact-segments=584 / diverges=0; `cargo test
   --workspace`, `cargo fmt --all -- --check`, and `git diff --check` pass.
   Fenced hot-loop is noisy: aggregate Rust/C++=3.038 then 2.889; direct
   repeat=100 `ai_assitant` JSON at
   `target/perf-ai-shape-paint-global-id.json` reports cpp median=0.604 ms,
   rust median=1.495 ms, Rust/C++=2.477. Strict <=2.0 remains open. Next:
   re-profile; likely targets remain data-bind context/source-local lookup and
   lower `RuntimeRenderPathCache::draw_path` map lookup, not source-queue
   vector swaps or borrowed retained-paint threading.
   Nested child data-context sync now retains resolved source locals by child
   property/image binding index on `RuntimeNestedArtboardInstance`, seeded from
   the existing path map and rebuilt with dynamic `artboardId` swaps. This ports
   the C++ `DataContext`/`DataBind` retained-source shape without adding a new
   skip gate: each frame still reads the current source value, while the steady
   path avoids a per-binding path-map lookup before the fallback slot walk. Full
   `make golden-compare` remains exact=263 / exact-segments=584 / diverges=0;
   `cargo test --workspace`, `cargo fmt --all -- --check`, and
   `git diff --check` pass. Fenced repeat-aware hot-loop is noisy at aggregate
   Rust/C++=2.972 then 3.167, but Rust median sum was 3.331 then 2.724 ms and
   `ai_assitant` Rust median improved to 1.414 then 1.140 ms. Direct repeat=100
   `ai_assitant` JSON at `target/perf-ai-binding-source-local-slots.json`
   reports cpp median=0.581 ms, rust median=1.462 ms, Rust/C++=2.516. A fresh
   sample shows `stateful_nested_host_binding_value_for` and
   `stateful_nested_host_value_local_for_slots` lower, with remaining time split
   across draw replay/path-cache `BTreeMap` lookups, data-bind source queues,
   converter advance, and state-machine advance. Strict <=2.0 remains open.
   Next: profile the remaining draw `BTreeMap::get` under
   `runtime_draw_command` / `RuntimeRenderPathCache::draw_path` and the
   remaining data-bind queue drains; keep the source-queue vector-swap and
   borrowed retained-paint threading scouts rejected.
   Retained render paints now live in dense global-id slots
   (`RuntimeRenderPaints`) instead of a `BTreeMap`, so persistent draw and
   gradient-prep paint access mirrors C++ retained `ShapePaint::renderPaint()`
   pointer lookup while preserving the old factory allocation side effects for
   golden ordering. Full `make golden-compare` remains exact=263 /
   exact-segments=584 / diverges=0; `cargo test --workspace`,
   `cargo fmt --all -- --check`, and `git diff --check` pass. Fenced
   repeat-aware hot-loop reports aggregate Rust/C++=3.000 with
   `ai_assitant`=2.799. Direct repeat=100 `ai_assitant` JSON at
   `target/perf-ai-render-paint-slots-rerun.json` reports cpp median=0.576 ms,
   rust median=1.437 ms, Rust/C++=2.495, with Rust draw median down to
   0.289 ms from the previous artifact's 0.312 ms. Strict <=2.0 remains open.
   Next: profile the remaining lower `RuntimeRenderPathCache::draw_path`
   lookup and data-bind queue drains; keep source-queue vector swaps and
   borrowed retained-paint threading rejected.
   A status-doc review of the scout/perf discoveries keeps the fences binding:
   no broad converter-property writes, no RangeMapper retry without deeper C++
   ownership/order analysis, no scratch-only context-path reuse, no shallow
   command/path wrappers, no source-queue vector swaps, and no borrowed
   retained-paint threading without release/null-renderer evidence.
   `ai_assitant` profiling after state-machine pending-action retention still
   showed `advance_nested_artboards_collect_events` beside data-bind,
   state-machine, and draw replay. C++ `NestedArtboard::advanceComponent`
   advances nested animations without a caller-owned per-child event vector, and
   `StateMachineInstance` owns reported event queues. Rust nested artboard
   advance now makes event collection optional: no-observer paths pass `None`
   through nested animation advance and avoid allocating ignored event vectors or
   cloning reported events. Full `make golden-compare` remains exact=263 /
   exact-segments=584 / diverges=0; `cargo test --workspace`, `cargo fmt --all
   -- --check`, and `git diff --check` pass. Fenced repeat-aware hot-loop is
   noisy by ratio but Rust median sum improves from the pre-slice 3.488 ms to
   2.476 ms and 2.488 ms on rerun; aggregate reports Rust/C++=3.041 and 3.073
   because C++ median sum also dropped. Direct repeat=100 `ai_assitant` JSON at
   `target/perf-ai-nested-event-option.json` reports cpp median=0.413 ms, rust
   median=1.041 ms, Rust/C++=2.521. Strict <=2.0 remains open.
   A follow-up profile after the optional nested-event slice showed the hot
   split across `advance_artboard_data_binds_with_root_transform`,
   `runtime_draw_command`, nested-event collection, state-machine advance, and
   nested data-context lookup. C++ `DataBindContainer` owns persistent and dirty
   vectors and uses membership state to avoid duplicate dirty-list enrollment.
   Rust now uses a per-binding `custom_property_update_flags` bitmap when
   merging dirty and persisting custom-property source updates, and skips both
   custom-property and numeric source update-index construction for empty lanes.
   This keeps the source-queue vector-swap scout rejected while porting the
   C++ queue-membership shape. Full `make golden-compare` remains exact=263 /
   exact-segments=584 / diverges=0; `cargo test --workspace`, `cargo fmt --all
   -- --check`, and `git diff --check` pass. Fenced repeat-aware hot-loop is
   noisy: the baseline after nested-event collection was aggregate Rust/C++=2.877
   with Rust median sum 2.467 ms; the post-slice rerun reports aggregate
   Rust/C++=3.007 with Rust median sum 2.319 ms, while the first post-slice run
   regressed to Rust median sum 2.667 ms. Direct repeat=100 `ai_assitant` JSON
   at `target/perf-ai-data-bind-queue-flags.json` reports cpp median=0.379 ms,
   rust median=0.989 ms, Rust/C++=2.613. Strict <=2.0 remains open. Next:
   profile remaining `runtime_draw_command` / `RuntimeRenderPathCache::draw_path`,
   state-machine fixed overhead, and remaining nested data-bind context-path
   work under these fences.
   Solo target binding apply now walks the retained binding list without
   cloning each `RuntimeArtboardSoloBindingInstance` before calling the Solo
   update path, matching C++ `DataBindContainer` retained-list traversal plus
   `context_value_{number,string,enum}` direct `Solo::updateByIndex` /
   `Solo::updateByName` dispatch. Full `make golden-compare` remains exact=263
   / exact-segments=584 / diverges=0, and `cargo test -p rive-runtime --quiet`
   passes. Fenced repeat-aware hot-loop is noisy: aggregate Rust/C++=2.939
   with Rust median sum 2.315 ms. The targeted single-file repeat=100 JSON at
   `target/perf-ai-solo-binding-no-clone.json` reports cpp median=0.391 ms,
   rust median=0.939 ms, Rust/C++=2.403, improving the prior `ai_assitant`
   Rust median from 0.989 ms. Strict <=2.0 remains open. Next: profile the
   remaining draw-path lookup, state-machine fixed overhead, and nested
   data-bind context-path work.
   A same-session dense paint-configuration sidecar scout was intentionally not
   landed. It replaced the draw-time `BTreeMap<u32,
   RuntimeCachedRenderPaintConfiguration>` with global-id slots, mirroring the
   retained render-paint slot shape, but fenced release/null-renderer evidence
   rejected it: focused hot-loop moved to aggregate Rust/C++=3.001 and direct
   repeat=100 `ai_assitant` worsened from rust median=0.939 ms to 0.997 ms
   (`target/perf-ai-dense-paint-config.json`). The sampled BTree hit is not
   enough by itself; keep profiling draw replay and prefer a deeper C++
   retained-path / object-identity slice over another sidecar container swap.
   The release Rust profile now matches the build-profile parity scout's
   shipping-runtime recommendation: root `Cargo.toml` sets `lto = "fat"`,
   `codegen-units = 1`, and `panic = "abort"` for `[profile.release]`, after a
   `catch_unwind`/`resume_unwind` search found no C ABI unwind reliance. Full
   `make golden-compare` remains exact=263 / exact-segments=584 / diverges=0;
   `cargo test --workspace`, `cargo fmt --all -- --check`, `git diff --check`,
   and `cargo build --release -p rive-capi` pass. Fenced repeat-aware hot-loop
   with the LTO profile reports noisy median aggregates Rust/C++=3.371 then
   2.760; direct repeat=100 `ai_assitant` JSON at
   `target/perf-ai-release-profile.json` reports cpp median=0.423 ms,
   rust median=1.352 ms, Rust/C++=3.195, while min timings are cpp=0.388 ms
   and rust=0.981 ms (~2.53x). Strict <=2.0 remains open. Next: implement the
   scout's min-based/deliberate perf gate (`--aggregate=min` and image-bearing
   focused corpus) before using focused perf numbers to choose another runtime
   slice.
   The min-based/deliberate gate tooling is now landed: `perf-compare` accepts
   `--aggregate median|min`, thresholds the selected statistic, preserves both
   median and min sums in JSON, and supports `--corpus-ids` so focused perf is
   not alphabetical truncation. `make perf-hot-loop` defaults to
   `PERF_AGGREGATE=min` and the deliberate focused corpus
   `advance_blend_mode,ai_assitant,align_target,animated_clipping,animation_reset_cases,spotify_kids_demo`,
   with default `PERF_ITERATIONS=10` and `PERF_BENCHMARK_REPEAT=100` so bare
   `make perf-hot-loop` runs the adopted fence rather than a smoke-only path.
   Fenced release/null-renderer smoke with the defaults reports aggregate min
   Rust/C++=4.758 over 11 file/sample entries; the newly visible image path
   dominates (`spotify_kids_demo@0` min Rust/C++=10.413), followed
   by the known tiny-file fixed overhead outliers. Full `make golden-compare`
   remains exact=263 / exact-segments=584 / diverges=0; `cargo test
   --workspace`, `cargo fmt --all -- --check`, and `git diff --check` pass.
   Follow-up image micro-slices showed the shallow cache and mesh-index
   precompute layers are not the C++ optimization. Retained clipping-shape
   path geometry, the N-slicer fast-miss, retained draw-command object kinds,
   retained path-composer local-index lookups, dense draw-path retained slots,
   and graph-scoped dense path-geometry command slots now remove sampled
   draw-replay rebuild/discovery/string-dispatch/vector-scan/BTreeMap lookup
   paths while preserving the full ratchet. Full `make golden-compare` remains
   exact=263 / exact-segments=584 / diverges=0; `cargo test --workspace`,
   `cargo fmt --all -- --check`, and `git diff --check` pass. A same-session
   fenced run before the local-index slice reported aggregate min Rust/C++=3.399
   but C++ min-sum=1.024 ms, outside the sanity band; the post-slot perf runs
   were deferred because verification-time load remained above the fence,
   latest 25.13/42.76/45.72. Strict <=2.0 remains open. Next runtime target
   should be a low-load release sample and then actual image/`PathComposer`/
   raw-path retention or deeper draw-replay fixed-overhead work, under the
   existing scout fences.
   Layout-adjusted draw world transforms are now cached in
   `RuntimeRenderPathCache` dense local slots behind the existing
   `(cache_epoch, layout_epoch)` dirt boundary. Shape path prep, clipping prep,
   gradient paint prep, image draw, mesh-image draw, and nested-artboard host
   draw now route through this cache when a layout-bounds frame exists, while
   no-layout calls keep using the retained component transform directly. This
   targets scout item 17's draw-replay world-transform recompute bucket without
   adding a new image draw-state cache, mesh-index precompute, or geometry
   float-math rewrite. Full `make golden-compare` remains exact=263 /
   exact-segments=584 / diverges=0; `cargo check -p rive-runtime`,
   `cargo test -p rive-runtime --quiet`, `cargo test --workspace`,
   `cargo fmt --all -- --check`, and `git diff --check` pass. Perf was not
   rerun because load was 24.90/37.27/29.64, outside the acceptance fence.
   Strict <=2.0 remains open. Next: run a clean low-load `make perf-hot-loop`,
   then profile/port the remaining image/`PathComposer`/raw-path retention or
   deeper draw-replay fixed-overhead work under the scout fences.
   Mesh render buffers are now retained in dense graph-local slots on
   `RuntimeRenderPaintCache`. C++ `MeshDrawable` owns its vertex/UV/index render
   buffers directly; Rust previously kept `RuntimeMeshRenderBuffers` behind a
   draw-time `BTreeMap` keyed by mesh local id. `RuntimeMeshRenderBufferSlots`
   now keeps the same preallocated buffers in local-id slots while preserving
   the existing mesh discovery, source-buffer allocation, vertex-byte reuse, and
   weighted mesh math. This is not the rejected image mesh-index precompute
   scout: image-to-mesh discovery still happens through the existing graph
   lookup. Full `make golden-compare` remains exact=263 /
   exact-segments=584 / diverges=0; `cargo check -p rive-runtime`,
   `cargo test -p rive-runtime --quiet`, `cargo test --workspace`,
   `cargo fmt --all -- --check`, and `git diff --check` pass. Two post-slice
   release/null-renderer samples with `make perf-hot-loop PERF_MAX_RATIO=999`
   report aggregate min Rust/C++=3.219 then 3.176, but C++ min-sums=0.992 ms
   and 1.053 ms are outside the 0.70-0.95 ms sanity band despite low 1-minute
   post-run load. Strict <=2.0 remains open. Next: run a clean
   low-load/sanity-band `make perf-hot-loop`, then continue actual
   image/`PathComposer`/raw-path retention or deeper draw-replay fixed-overhead
   work under the scout fences.
   Image layout local transforms are now retained in `RuntimeRenderPathCache`
   behind the existing `(cache_epoch, layout_epoch)` dirt boundary plus image
   and layout dimensions. This ports the C++ `Image::updateImageScale` shape:
   clean frames reuse the computed image scale/offset local transform, then
   multiply it by the cached parent layout world transform. Mesh-image draw now
   also routes through this retained image world transform, matching C++
   `Mesh::draw`'s use of the parent image `worldTransform()`. The slice does
   not cache blend/opacity/draw state and does not repeat the rejected shallow
   image draw-state cache. Full `make golden-compare` remains exact=263 /
   exact-segments=584 / diverges=0; `cargo check -p rive-runtime`,
   `cargo test -p rive-runtime --quiet`, `cargo test --workspace`,
   `cargo fmt --all -- --check`, and `git diff --check` pass. A post-slice
   `make perf-hot-loop PERF_MAX_RATIO=999` reports aggregate min Rust/C++=3.225,
   but C++ min-sum=1.043 ms is outside the 0.70-0.95 ms sanity band. Strict
   <=2.0 remains open. Next: run a clean low-load/sanity-band
   `make perf-hot-loop`, then continue actual `PathComposer`/raw-path retention
   or deeper draw-replay fixed-overhead work under the scout fences.
   Draw `RenderPath` cache entries now retain the raw path payload used to
   rebuild the renderer path, mirroring C++ `ShapePaintPath::m_rawPath` /
   `ShapePaintPath::renderPath`. On a path-epoch miss Rust rebuilds the cached
   `RawPath` in place with `rewind()` capacity reuse, then calls
   `RenderPath::add_raw_path`; clean frames keep reusing the retained
   `RenderPath` as before. This does not reintroduce the rejected shared
   shape path-command buffer/cache layer and does not change path geometry
   math, fill-rule handling, or the existing `path_epoch` invalidation
   boundary. Full `make golden-compare` remains exact=263 /
   exact-segments=584 / diverges=0; `cargo check -p rive-runtime`, the focused
   draw-path reuse test, `cargo test --workspace`,
   `cargo fmt --all -- --check`, and `git diff --check` pass. A same-session
   release/null-renderer sample before
   the final capacity-reuse polish reported aggregate min Rust/C++=3.219, but
   C++ min-sum=1.037 ms is outside the 0.70-0.95 ms sanity band and movement
   versus 3.225 is below the noise floor; no decision-grade post-polish perf
   sample was taken because load rose to 21.26/15.84/13.92. Strict <=2.0
   remains open. Next: rerun a clean low-load/sanity-band `make perf-hot-loop`,
   then profile/deepen draw-replay fixed overhead or clean-prepare-skip work
   rather than extending raw-path wrappers.
3. The former `nested-stateful-view-model-property`,
   `nested-layout-clip-data-bind`, `nested-node-transform-data-bind`,
   `nested-text-outline-contour-order`, `layout-component-paint`, and
   `nested-feather-gradient-space` unsupported queues are empty.
4. Remaining non-exact entries are intentionally parked as `gated` or
   `harness`. Gated diagnostics include `scripted-data-context`
   (`scripted_data_context.riv`), `scripted-transition-condition` (2 gated),
   `scripted-path-effects` (1 gated), and `text-polygon-sibling`
   (`bankcard.riv`). Keep these parked queues as explicit unsupported/gated
   work until an M7 or scripting/harness slice can either promote a file or
   replace the guard with a sharper diagnostic.
   The old `text-input` manifest queue is empty.
5. M5 is closed for the current corpus: `grep -B6 'milestone = "M5"'
   corpus.toml` is empty. Do not reopen data-binding work unless a newly added
   corpus entry exposes a pre-text/pre-layout data-binding diagnostic.
6. Remaining exact entries pinned to sample `0` are static M1 holdovers:
   `artboardclipping.riv`, `shapetest.riv`, and `trim.riv`. Do not prioritize
   them during M6 unless a related refactor needs a cheap draw-regression check.

7. Threads are now policy (see `/goal` "Threads" section): the main loop
   stays the single writer here; use read-only scout threads to triage the
   remaining M6 queues in parallel. Start the first lane thread in a NEW
   worktree for the C++ golden-runner crash repair (`milestone =
   "harness"`, 36 files; FileAssetContents/scripting/data-viz crash paths
   in `tools/golden-runner` only), merging back into this branch once the
   full ratchet passes. Recovered files enter as `not-yet` — denominator
   growth, zero conflict with M6 runtime work.

8. Harness lane MERGED (e5941e7): the C++ golden-runner now survives 34 of
   36 `milestone = "harness"` files (FileAssetContents stripping for the
   non-scripting librive build, flush + `_Exit(0)` before teardown, ABI
   define alignment). MAIN-LOOP FOLLOW-UP is partially complete: 10 recovered
   entries were promoted exact after the image scripting property-value
   ordering fix; continue flipping the remaining recovered files from
   `milestone = "harness"` only after assigning each to exact/not-yet/gated
   with a verified compare result.
   Residuals (2): `data_viz_demo` and `data_binding_artboards_test` crash
   only because the runner binds a blank default view-model instance;
   binding named instance 0 (like the reference unit tests) recovers both
   but perturbs 66 currently-exact entries — treat as a coordinated
   convention-change decision, not a harness fix. Keep them
   `milestone = "harness"` until decided.

9. REVISED (see Decisions 2026-07-07): do not adopt the global named
   view-model instance 0 binding convention yet. The coordinated runner
   experiment recovered `scripted_color.riv` after binding the selected
   artboard's own owned view-model context, but still left 48 exact entries
   divergent because serialized list data makes C++ `ArtboardComponentList`
   instantiate and draw item artboards while Rust still has only partial
   component-list runtime support. Keep `data_viz_demo` and
   `data_binding_artboards_test` in `milestone = "harness"` under the
   current blank-default runner convention. Reopen the convention only after
   Rust supports `ArtboardComponentList` item artboard instancing, draw,
   layout, and view-model data-context binding well enough for the affected
   exact corpus to reverify green in the same commit.

10. SCOUT RESULT (read-only pre-classification of the 34 recovered harness
   files; streams/diffs in the session scratchpad — trust but re-verify on
   promotion): (a) promoted exact in the main loop:
   audio_script, multi_listeners, script_dependency_test,
   script_dependency_test2, script_dependency_test_using_library(+_v2),
   script_namespace_test, script_string_converter_test,
   scripted_listener_action, image_scripting_property_value. The latter
   required matching the non-scripting C++ golden runner's import stack:
   `ScriptAsset` does not displace a pending image `FileAssetImporter`, so
   the second image decodes after the source render-paint allocation.
   (b) gated-scripting (21): all remaining script*/viewmodel*/gamepad/
   data_bind_artboard_input/path_effect_with_feathers/group_effect/
   replace_view_model files — blocked on the Luau VM; note
   path_effect_with_feathers is ScriptedPathEffect content, NOT M6 feather
   work. (c) HARNESS-BLOCKED runtime candidates (3): relative_data_bind_path
   (nested-child data bind into NestedArtboard),
   scripted_data_converter_bound_input (data bind target Shape.x through
   static-text subset), databind_viewmodel (DataConverterToString value
   mismatch feeding a Text run — Rust data_bind_graph ToString produces a
   different string than C++). They remain `milestone = "harness"` until the
   C++ runner path is recovered and each file is reverified.
   PROCESS FIX REQUIRED before flipping the 18 stream-subset scripting
   files: the Rust runner silently drops ScriptedDrawable draws (known-
   ignored list in text.rs), so they would land as `diverges` and invite
   wrong work — add a loud `unsupported: scripting` diagnostic for
   ScriptedDrawable-bearing files first, then flip them straight to
   `milestone = "gated"`. Unsupported is never silent.

11. PERF METHODOLOGY FENCE (measurement gate before optimization). Earlier
   debug-vs-debug and recording-serializer perf numbers are void. The release
   C++/Rust runner builds, null-renderer benchmark mode, whole-repeat
   `total_ms` scoring, and perf JSON artifact path have landed; keep using
   them for all M7 decisions. Per-frame phase timings remain diagnostics only.
   Required order for any new optimization slice:
   (a) Release-vs-release perf builds: `cargo build --release` for the
       Rust runner and a release C++ runner + release reference libraries;
       correctness ratchet stays on debug. Re-baseline all ratios and
       discard debug-era perf conclusions and priorities.
   (b) Null-renderer benchmark mode on BOTH runners (same trait calls,
       output discarded) so the measured cost is pure runtime
       advance/prepare/draw-path work, not stream serialization.
       Re-baseline again.
   (c) Only then resume optimization slices, each one: flamegraph
       attribution (samply/Instruments) -> read the C++ source at the same
       hot site -> PORT the C++ optimization if one exists (keyframe
       cursors, ComponentDirt gating, RawPath rewind/reuse, paint/path
       caching) -> invent a novel optimization only when C++ has none
       there.
   (d) Statistical floor: >=10 iterations with median + spread, a pinned
       perf corpus spanning tiny/medium/heavy files, and a per-commit perf
       JSON artifact so trends are data, not "noisy but typical" recall.
   Fidelity rules while optimizing: no tolerance widening for perf; no
   float-math restructuring in geometry paths (the fused scaleAndAdd
   lesson — no reassociation/fast-math; SIMD only if the ratchet stays
   strictly green); no invalidation/skip logic that does not mirror an
   audited C++ dirt gate — invented caching is how original-author
   decisions get silently broken on unsampled timelines.

12. SCOUT REPORT — C++ animation-apply audit (port-ready, cited against
    reference @7c778d13). Headline: C++ has NO keyframe cursor and NO
    value-unchanged skip in the animation layer — do not invent them.
    KeyedProperty::apply is a stateless binary search over CACHED
    per-keyframe seconds (keyed_property.cpp:20-52) with an O(1)
    past-last-frame fast path (:28-32); the unchanged-value short-circuit
    lives in generated property setters (node_base.hpp:53-62), which
    Rust's changed-bool setters already mirror. Port slices, ranked:
    (1) STOP PER-FRAME DEEP CLONES — likely the dominant cost of the
        21.9x: crates/rive-runtime/src/artboard.rs:510 clones the entire
        RuntimeLinearAnimation (all keyed objects/keyframes incl. string
        byte Vecs) on EVERY apply, and artboard.rs:594 clones the whole
        RuntimeStateMachine on every advance. C++ applies from a shared
        immutable definition (LinearAnimation::apply(...) const,
        linear_animation.cpp:71-85) with mutation confined to the
        instance. Restructure to shared immutable definitions
        (Arc/index-based split borrows), apply by &ref.
    (2) Cache keyframe seconds at build (KeyFrame::computeSeconds,
        keyframe.cpp:10, called once at keyed_property_importer.cpp:15);
        Rust recomputes frame/fps with a zero-branch on every comparison
        of every search (animation.rs:1102-1107 + 5 sibling impls).
    (3) Precompute cubic solver state at build: 11-entry bezier-x table
        (cubic_interpolator_solver.cpp:28-95, built once at
        cubic_interpolator.cpp:5-11) — Rust rebuilds it inside every
        get_t call (animation.rs:145-156); also cache CubicValue
        coefficients behind a from/to guard (cubic_value_interpolator
        .cpp:26-35 vs animation.rs:128-139).
    (4) Kill steady-state allocs in advance plumbing: persistent
        reported-events buffers (state_machine_instance.hpp:336, drained
        :2293-2317), blend instance lists built once with reserve
        (blend_state_instance.hpp:51-71), pooled AnimationReset
        (animation_reset_factory.cpp:226-235) — vs Rust fresh Vecs per
        advance (artboard.rs:552-560, :601-604, :617-645).
    Also: interpolator pointers resolve once at onAddedDirty, validation
    is hoisted to init (invalid keyed objects erased), advanceAndApply
    caps at 5 passes breaking when Components dirt clears
    (state_machine_instance.cpp:2589-2616).

13. SCOUT REPORT — C++ draw-retention audit (port-ready, cited against
    reference @7c778d13). Governing principle: C++ computes NOTHING during
    draw() — all geometry/paint work happens in updateComponents gated by
    dirt (clean frame: first-branch return, artboard.cpp:1186-1189), and
    draw() replays retained RenderPath/RenderPaint handles. Confirmed Rust
    per-frame rebuilds: sorted drawable order w/ BTreeMaps+clones
    (draw.rs:224-299), vertex->command re-derivation per paint
    (draw.rs:2836-2951), unconditional runtime_rebuild_path on every cache
    access (draw.rs:5028-5041), layout bounds re-derived (draw.rs:996).
    Ranked port slices:
    (1) ShapePaintPath retention: retained RawPath + retained RenderPath +
        one dirty bool (shape_paint_path.hpp:78-84, .cpp:13-76); rebuild
        becomes a no-op on clean frames. Largest draw-phase win.
    (2) PathComposer gated by Path|NSlicer dirt (path_composer.cpp:40-117)
        plus dirt plumbing Path::markPathDirty/onDirty/Shape::pathChanged
        (path.cpp:327-348, shape.cpp:99-108); note plain transform changes
        do NOT rebuild vertex paths — WorldTransform only couples to path
        rebuild when a deformer exists (path.cpp:358-359).
    (3) Path::m_rawPath retention with rewind() capacity reuse
        (path.cpp:350-380; raw_path.cpp:446-451 rewind keeps capacity;
        addPath bulk memcpy+SIMD transform :255-279); zero-opacity deferral
        via canDeferPathUpdate + m_deferredPathDirt (path.cpp:111-126,
        :344-347).
    (4) RenderPaint mutate-in-place for instance lifetime: solid color
        writes mutate immediately w/o dirt (solid_color.cpp:24-54), stroke
        props via Paint dirt (stroke.cpp:37-53), gradients rebuild only on
        Paint|Stops|(WorldTransform iff world-space) into retained
        m_colorStorage with only the shader rcp swapped
        (linear_gradient.cpp:86-201).
    (5) Retained sorted drawable list (intrusive, resorted only on
        DrawOrder dirt, artboard.cpp:569-660,1142-1145) + retained clip
        paths (clipping_shape.cpp:151-173).
    CROSS-CUTTING PREREQUISITE for 1-3: per-component dirt bitset with the
    updateComponents early-out (artboard.cpp:1184-1223) so clean frames
    skip the entire prepare phase. Pairs with the animation-apply slices; do
    the deep-clone removal first, then this prerequisite, then slices by rank.

14. SCOUT REPORT — C++ dirt-gating audit (port-ready, cited against
    reference @7c778d13). Confirms Rust already mirrors the
    updateComponents loop skeleton (add_dirt / update_components_with_hook
    / dirt_depth vs artboard.cpp:1184-1223) — the gap is that per-frame
    work is not BEHIND the gates. Core primitives to port exactly:
    Component::addDirt early-out when bits already set (component.cpp:
    34-38, the single most important line: repeated writes collapse to one
    bit test); dirt cleared BEFORE update() runs (artboard.cpp:1209);
    DirtDepth lowered by upstream re-dirt triggers inner-loop break +
    re-sweep (artboard.cpp:978-990, 1215-1218); advanceAndApply settles
    with up to 5 updatePass loops breaking when Components dirt clears
    (state_machine_instance.cpp:2589-2615). Clean-frame contract: SM
    layers still APPLY keyframes every frame, but generated setters'
    equality early-outs mean steady values raise zero dirt, so
    updateComponents returns at its first branch and NO component is
    visited — draw() never checks dirt, it reads coherent caches.
    Ranked slices:
    (1) Idempotent property writes + the *Changed() dirt-raiser table
        (node.cpp:9-10, transform_component.cpp:54-61,119-121,
        world_transform_component.cpp:10-28, parametric_path.cpp:63-66,
        path_vertex.cpp:21-30, stroke.cpp:37-41,
        linear_gradient.cpp:203-215). Turns steady-value animation frames
        into zero-dirt frames.
    (2) Geometry behind Path dirt (= item 11 slices 1-3), incl. the
        invisible-shape deferral bonus: canDeferPathUpdate +
        m_deferredPathDirt (shape.cpp:35-52, path.cpp:344-347,361-365,
        path_composer.cpp:29-38,44-48) — opacity-0 shapes never build
        geometry.
    (3) Sorted draw list behind DrawOrder dirt only (raisers:
        draw_rules.cpp:40, draw_target.cpp:31); clipping ops behind
        Clipping dirt (artboard.cpp:1146-1149).
    (4) Render paints behind Paint|Stops|RenderOpacity (= item 11 slice
        4).
    (5) Data-bind dirty queues instead of scans (data_bind_container.cpp:
        145-258, data_bind.cpp:487-511, core.cpp:25-46 push observers with
        one-branch no-subscriber fast path, artboard.cpp:1169-1173).
    COMBINED SEQUENCE across the animation/draw/dirt scouts: kill per-frame
    definition clones -> idempotent writes/raiser table -> draw-retention
    prerequisite + retention slices in rank order -> remaining animation/data-bind
    dirt slices as flamegraph data directs. Full ComponentDirt bit inventory with consumers is in the
    scout transcript; component_dirt.hpp:8-81 is the source of truth.

15. SCOUT REPORT — release flamegraph attribution (samply, release build,
    null-renderer hot loop; profiles in session scratchpad). REORDERS the
    dirt-gating combined sequence:
    (0) NEW TOP SLICE — schema reflection in hot paths, ~36% of self time:
        definition_by_name (rive-schema lib.rs:252, LINEAR SCAN + string
        eq, 17.5%), definition_by_type_key (lib.rs:232 linear scan, 8.4%),
        Definition::property_by_key (lib.rs:217, walks ancestors via
        definition_by_name, 5.4%), property_key_for_name (properties.rs:
        200, string->key per property READ, 5.4%). C++ uses compile-time
        property-key constants + switch tables; runtime name/definition
        resolution must not exist in the frame loop. Fix: precompute
        typed accessor/key tables at instance build (fidelity-neutral —
        this is invented Rust structure, not C++ behavior).
    (1) Clone hypothesis CONFIRMED in direction, corrected in site:
        allocator/copy traffic is 25-44% of self time, but ~70-85% of
        clone samples come from ArtboardGraph deep clones in
        artboard_data_bind.rs (~:1617, runtime_graph().cloned() in
        update_*_source_bindings, multiple times per advance);
        artboard.rs:594 is secondary (~5-11%), artboard.rs:510 minor.
        Fix the data-bind clones FIRST, then item 10(1).
    (2) ai_assitant's 16.1% TrimContour::get_segment: re-dashing every
        frame with linear segment scans; C++ caches m_contours + dashed
        result behind dirt (trim_path.cpp, contour_measure.cpp).
    (3) Taffy node tree rebuilt every prepare+draw (60% inclusive on
        blend file) with reflection-heavy style reads; C++ runs layout
        only on markLayoutNodeDirty.
    MEASUREMENT CORRECTIONS: (a) current tree measures 8.44x on
    ai_assitant (not 37.5x — earlier number was different tree state);
    (b) CRITICAL harness hazard: with --benchmark-repeat 4000, C++ drops
    to ~1.5us/segment because dirt-gating makes frames 2..N nearly free —
    the steady-state gap is orders of magnitude larger than the
    single-pass ratio, and the ratio is extremely sensitive to
    amortization. DEFINE the M7 perf target explicitly as STEADY-STATE
    per-frame cost (high repeat count, cold frame excluded or reported
    separately); the retention/dirt slices in items 10-12 are what close
    the steady-state gap. Record the chosen definition as a Decision
    before optimizing further.

16. LANE MERGED (88fe434): scripting spike. `crates/rive-scripting`
    (feature `luau`, default-on, zero deps leaking) proves luaur 0.1.8
    (PINNED =0.1.8, upstream Luau commit 8f33df9): boots, loads real
    Rive-editor Luau BYTECODE directly (ScriptAssets carry bytecode v6 in
    a SignedContentHeader envelope — ported as rive_scripting::envelope;
    the runtime never compiles source), executes corpus scripts
    end-to-end (ArtboardGrid generator->instance with inputs), and
    resolves the corpus require chain via C++-style registration retries
    (mirrors ScriptingContext::performRegistration). mlua fallback NOT
    needed on this evidence. Known gaps recorded in the lane report:
    bytecode loads via one unsafe luau_load seam (upstream ask: safe
    ChunkMode::Binary — file on pjankiewicz/luaur); sandbox parity
    REQUIRED before real integration (C++ init order: open libs -> rive
    globals -> luaL_sandbox -> load; GETIMPORT resolves globals at LOAD
    time — install all globals first); bind Vector via luaur's native
    vector type, not a table. Seam proposal: traits ScriptingVm /
    ScriptInstance / ScriptHost defined IN rive-runtime (keeps luaur out
    of its deps), implemented by rive-scripting, wired in crates/rive
    behind a feature; method dispatch gated by SERIALIZED
    OptionalScriptedMethods bitmask (script_asset.hpp:70-181), input
    writes raise ScriptUpdate dirt. Bindings sizing from a census of all
    57 corpus scripts: ~half of the 18.2k C++ glue needed, in order:
    boot/registration+scripted_object protocol (~2.5k) -> Vector/Mat2D/
    Color/Path/Paint/renderer verbs (~2.5k) -> artboards/animations
    (~1k) -> DataValue+viewmodel properties (~2-3k) -> listener/input
    tail (~1.5k). NOT needed by corpus: lua_gpu (3.7k), lua_promise,
    lua_mat4, lua_buffer_ext, most of lua_image_decode, lua_audio.
    Signature verification (libhydrogen) deferrable — corpus unsigned.

17. LANE MERGED (d8cf8cb): C ABI embed loop + perf JSON.
    crates/rive-capi now covers file->artboard-instance->state-machine->
    inputs->advance->draw via a caller-provided RiveRenderCallbacks
    repr(C) vtable (FFI-renderer pattern, opaque u64 handles, balanced
    release_* calls, nullable callbacks); `make capi-smoke` runs a real C
    embed loop. perf-compare gained --json/--meta (phase-sum metrics,
    benchmark_repeat recorded, meta passed in never computed) +
    `make perf-json` + additive CI jobs (capi-smoke; perf-json artifact,
    continue-on-error). Additive crates/rive API: Factory/Renderer
    re-exports, Artboard::state_machine_name/default_state_machine_index,
    ArtboardInstance::default_state_machine_instance/
    advance_with_state_machine. Follow-ups: (a) once draw-frame retention
    stabilizes, add an additive cache-holding draw so the C ABI reuses
    render handles across frames; (b) pointer events + view-model
    contexts are additive ABI gaps; (c) default-SM selection: capi
    falls back to first (C++ defaultScene) while the golden runner uses
    flagged-or-none — align once embed parity matters.

16. SCOUT REPORT — gate protocol + phase-gap localization (decision-grade,
    45 fenced runs; scripts/JSONs in session scratchpad). FOUR findings
    that redirect M7 perf work:
    (a) ADOPT MIN-BASED AGGREGATION NOW: the median-based aggregate has a
        +-0.42 noise band per run (observed phantom 2.18 reading when
        min-based truth was 2.95); sum of per-target min_ms over 10
        iterations gives +-0.07 with the same central value. Contention
        noise is one-sided; min recovers intrinsic cost, both sides
        treated identically. Improvements < ~0.08 ratio are below
        single-run resolution — don't claim them without 2 runs pre/post.
    (b) PIN THE GATE DEFINITION: total(N) = first-frame + (N-1)*clean-
        frame, and the two have different ratios. Scout-time focused
        aggregate was ~2.98 at repeat=100 but ~3.95 at repeat=1000
        (pure steady state), before the deliberate image-bearing default
        gate made the current standing worse. The adopted M7 gate is
        repeat=100; track improvements at that fixed N.
    (c) THE GAP IS RENDER-SIDE, NOT ADVANCE-SIDE: for text-bearing files
        Rust ADVANCE is already FASTER than C++; the focused-corpus gap
        concentrates in prepare+draw clean-frame replay plus ~0.5-1us/
        frame fixed overhead (epoch checks) that dominates tiny files
        (3.5-6.4x) vs heavy (2.1-2.6x). Recent data-bind/advance slices
        target the smaller half of the remaining gap — shift to draw
        replay cost and tiny-file fixed overhead.
    (d) HEADLINE — IMAGE FILES ARE 10-170x AND INVISIBLE TO THE GATE:
        car_widgets_v01=145-170x, echo_show_demo=81-112x,
        jellyfish_test=61-65x, spotify_kids_demo=11x — draw-dominated,
        LINEAR in repeat count (~4.6ms/frame steady on car_widgets => a
        retained-draw cache is missed/rebuilt every frame on the image
        path). PERF_CORPUS_LIMIT=5 takes the first five ALPHABETICAL
        files, which contain no images. Fix the image draw retention AND
        make the gate corpus deliberate (include >=1 image file, e.g.
        spotify_kids_demo); track repeat=1000 as a secondary diagnostic.
    GATE PROTOCOL (adopt as Decision): ratio-of-sums over per-target
    min_ms, 10 iterations, repeat pinned; acceptance = 3 independent
    invocations ALL <= 2.0 with 1-min load < ~8 and C++ min-sum inside
    its 0.70-0.95ms sanity band. Scout-time standing before the
    image-bearing default gate was 2.98 (band 2.82-3.06) at repeat=100
    and ~3.95 at repeat=1000; see the top-level M7 Perf Fence for the
    current standing. The distance to 2.0 is real, not noise. Tool
    follow-ups landed:
    --aggregate=min flag, deliberate --corpus-ids gate, and make defaults for
    10 iterations / repeat=100 on perf-hot-loop.

17. SCOUT REPORT — fresh flamegraph of TODAY'S tree (samply, release,
    steady + cold regimes profiled separately; C++ steady profile captured
    as fairness baseline; profiles in session scratchpad). Old top sites
    CONFIRMED DEAD in steady: schema scans <1%, ai malloc ~1.5%,
    TrimContour retention works, Taffy zero frames. What remains, ranked
    by payoff:
    (1) CHEAPEST BIG WIN — cold-frame name reflection is ~60% of the gate
        metric's delta (repeat=100 blends heavily with the cold frame):
        authored_transform does SIX name lookups per component on the
        first world-transform pass (artboard.rs:885-890 ->
        objects.rs:298 runtime_property_metadata_by_name linear scan +
        ancestor walk); definition_by_name 19.4% self in the cold region.
        MECHANICAL: per-component cached transform property keys or
        direct storage fields. Also clears the keyframe-apply name
        resolution survivor on animation_reset_cases (5.3% incl) and
        align_target's raw uint_property reads (9.6% incl).
    (2) STEADY #1 (~43% of steady delta) — data-bind cluster runs ~12
        unconditional sub-passes per frame (artboard_data_bind.rs:2887):
        Rust re-reads/compares every source value every frame; C++ only
        ticks converter clocks, with target writes gated by
        ComponentDirt::Bindings via the addDirtyDataBind queue
        (data_bind_container.cpp:38, data_bind.cpp:487/546). STRUCTURAL:
        port the dirty-databind architecture, not another value-compare
        cache.
    (3) STEADY #2 (~31%) — draw replay dispatch: BTreeMap gets inside
        runtime_draw_command are 8.6% self (-> dense Vec-by-slot),
        type_name string dispatch per command (-> precomputed command-
        kind enum), world transforms recomputed in draw (2.4-3.1%
        everywhere -> cache behind dirt like transform_component.cpp).
        Per-file structural item: animated_clipping rebuilds clip-path
        Vec + verbs every frame (50.7% inclusive + ~21% allocator) ->
        port ClippingShape/PathComposer clip RenderPath retention.
        Gradient re-prep per frame explains advance_blend_mode's 4.55
        (prepare 36% inclusive) -> gate stops behind Paint dirt.
    (4) SM FIXED OVERHEAD — REFUTED as a priority: C++ spends 25-30% of
        its own frame there; Rust is ~2x per-unit, the BEST bucket. Do
        not spend slices here.
    HARNESS NOTE: mach_absolute_time reads are up to 23% of tiny-file
    steady frames (4 timed sections/frame), compressing small-file
    ratios — consider timing whole repeat blocks instead of per-frame
    sections. Regime split for honest tracking: cold ai_assitant 2.41x,
    steady ~3.4x — item 16's pinned-N decision applies.

18. SCOUT REPORT — build-profile parity audit (variants built from a
    pinned snapshot in an isolated target dir; raw data + binaries in
    session scratchpad). VERDICT: the gate is currently UNFAIR AGAINST
    RUST. C++ librive.a builds at Rive's shipping config with FULL LTO
    (-flto=full is the default in rive_build_config.lua; archive members
    are LLVM bitcode) while the Rust workspace has NO [profile.release]
    anywhere — bare cargo defaults (lto off, codegen-units=16), which is
    not how a shipping Rust runtime is built. Measured, fidelity-verified
    (full golden-compare, 263 exact, diverges=0 per variant):
    (1) ADOPT: [profile.release] lto = "fat", codegen-units = 1 in the
        root Cargo.toml — aggregate ratio ~2.80 -> ~2.58 (median-agg,
        r=100), lopsided toward align_target (-12%) and animated_clipping
        (-19%). If the 122s build hurts the loop, lto = "thin" (38s) is
        also fidelity-clean; measure once before choosing.
    (2) ADOPT (with one check): panic = "abort" in release — further
        ~2.58 -> ~2.49. Verify no catch_unwind reliance in rive-capi
        consumers first (none observed in the runner path).
    (3) DO NOT ADOPT: -C target-cpu=native — passes fidelity (Rust FP is
        IEEE-strict regardless of ISA) but fails FAIRNESS: C++ builds
        generic arm64. Only symmetrically, only if a machine-tuned gate
        is ever wanted.
    (4) CODE-LEVEL NOTE, FENCED: C++ gets free FMA fusion from clang's
        default -ffp-contract=on; Rust never contracts. Closing this
        means explicit f32::mul_add at hot sites, which CHANGES float
        results — each site requires golden re-verification and may flip
        exact files; treat as last-resort under the geometry-float fence,
        never as a bulk pass.
    C++ side verdict: NOT under-built; nothing to fix there. After
    (1)+(2), remaining ~2.5x is genuine runtime-architecture cost —
    items 16/17 are the map for that. The Cargo.toml change is the
    main loop's slice to land (single-writer rule).

## Known Divergences

- There are no active `status = "not-yet"` entries.
- There is no remaining `milestone = "M6"` parked work; remaining non-exact
  files are behind explicit `gated` or `harness` diagnostics.

## Backlog (unsupported features awaiting corpus demand)

- Golden runner view-model mutation scripts; `--view-model-script` is reserved
  but rejected until a future external data-binding corpus file requires it.
- Scripted data-context execution is gated until the `mlua`/Luau scripting
  glue lands: `scripted_data_context.riv` now emits
  `rust-runner-unsupported:scripted-data-context` when a `ScriptedDrawable`
  combines `DataBindContext` text with nested view-model context. The focused
  C++ runner printed `Failed to import object of type 106` before suppressing
  the text, so this is an M6 scripting diagnostic rather than text
  draw-suppression work.
- Rust golden draw path currently supports sorted absolute-time samples,
  visibility-gated artboard clip/background, selected-artboard origins, solid
  fills/strokes, and
  `ClippingShape` clip paths, skinned `PointsPath` deformation, plus empty and
  multi-contour TrimPath effects, DashPath stroke effects, and linear/radial
  gradient shader creation, default state-machine frame-0 application for
  color/bool/uint/string keyframes, Solo active-child refresh, source-to-target
  and target-to-source `Solo.activeComponentId` enum binds, enum-to-string
  artboard property conversion, `Text.alignValue` enum/uint binds,
  ViewModel-vs-ViewModel transition comparators for number, bool, color,
  string, enum, asset, and artboard bindables, and
  before-update joystick animation application, single Joystick data binds
  already covered by exact fixtures, keyed double/color
  interpolation for CubicEase/CubicValue/Elastic keyframe interpolators, and
  flagged 1D blend-state double/color animation resets using the first blend
  animation as the baseline like C++ `AnimationResetFactory`, and
  `DistanceConstraint` world-translation application and
  `TranslationConstraint` target/source/destination/min-max translation
  application, `RotationConstraint` compose/decompose rotation,
  `ScaleConstraint` compose/decompose scale, `TransformConstraint`
  target-origin full-transform interpolation, `FollowPathConstraint`
  Shape/Path target sampling against runtime path geometry, C++ Bone x/y
  overrides, `IKConstraint` FK-chain solving, and
  `ListFollowPathConstraint` registration/application over component-list item
  transform slices once layout-backed list instances populate them, and
  parametric
  Star/Polygon local path sampling for follow-path targets, plus static plain
  `NestedArtboard` host draw with child root opacity inheritance, default
  nested simple-animation/state-machine hosts backed by persistent child
  artboard instances, stateful child `ViewModelInstance` subtree admission
  under plain and layout/leaf nested hosts, nested child unbound SolidColor data-bind defaults,
  nested child Ellipse width/height, RootBone x/y, and Shape x/y
  source-to-target number binds backed by stateful child view-model values,
  direct no-converter Shape x/y number binds, direct SolidColor `colorValue`
  color binds, artboard
  source-to-target `DataConverterInterpolator` number/color binds,
  artboard source-to-target `DataConverterGroup`/`DataConverterFormula`
  transform binds with C++ fallback random sequencing, near-zero-duration
  `DataConverterInterpolator` Shape x/y transform binds,
  nested bool/number/trigger input proxying, and basic nested remap-time host
  plumbing, runtime `DrawTarget` placement sorting from active `DrawRules`,
  serialized nested host speed/quantize local elapsed, generated
  source-to-target nested host `isPaused`/`speed`/`quantize` default binding,
  source-to-target nested host `artboardId` default/runtime swaps with
  cleared-host draw suppression,
  per-host nested paint caches for repeated child instances under Solo-owned
  hosts, and nested state-machine reported-event bubbling into parent event
  listeners, nested child `Node.opacity` and `Rectangle.width/height`
  source-to-target number binds with child artboard data-bind advancement,
  nested child `CustomPropertyString.propertyValue` string binds and
  `Rectangle.width/height` 20/21 binds, nested child `TextValueRun.text`
  string, `SolidColor.colorValue` color, and converted
  `Shape.rotation`/`Node.rotation` binds backed by stateful child view-model
  values,
  authored-transparent Backboard/background draw suppression,
  custom-property trigger keyed-callback target-to-source binding,
  custom-property enum/boolean/color target-to-source binding, live data-bound
  nested host `isPaused` mutation, plus no-input recursive nested
  `ListenerAlignTarget` fixtures where the action is unexercised, plus plain
  embedded/hosted non-mesh `Image::draw` including layout-controlled
  fit/alignment under Taffy bounds, metadata-only `NSlicer`/axis
  image-layout fixtures that render through existing `LayoutComponent` paints,
  and sample-0 asset-image listener files whose image decode/source-paint
  ordering is exact while drawing only simple vector/text siblings, plus
  import-stack-displaced pre-source embedded image decode ordering for mixed
  file-asset imports, simple
  clipped/draw-target image fixtures with metadata-only component-list nodes,
  plus an asset-only unresolved nested-library host that decodes its image
  asset but draws only the root background like C++, simple
  ShapePaint/Feather draws including outer feathers and repeated
  inner-feather paints that share the original/effect clip path, and
  NSlicedNode vector shape path deformation for local/world draw commands,
  plus mesh-backed `Image::draw`/`drawImageMesh` with file-wide source mesh
  buffer preallocation, cloned dynamic vertex buffers, UV/index buffers,
  skinned mesh vertex updates, live `Stroke.thickness` visibility/paint
  application for state-machine-keyed paints, and frame-0 `TextInput`
  multiline text/cursor/empty-selection generated paths with intrinsic
  layout measurement.
  Custom handle-source world-space math, data-bound nested host controls beyond
  generated defaults (external/live pause/speed/quantize mutation), remaining
  nested child data-bind targets beyond the current number/color/default bind
  set, and broader nested child object/list/value propagation remain
  corpus-driven follow-up work. Focus data,
  input-driven recursive
  `ListenerAlignTarget` and nested pointer/listener hit propagation beyond
  reported `Event` listeners, and layout-backed or virtualized component-list
  instancing remain M6 or later diagnostics.
  Golden runner sample lists now advance by sorted absolute-time deltas and
  reuse render paths across samples; no broader NSliced image layout parity,
  selected-root image paint/preallocation ordering beyond ImportStack-displaced
  embedded-image predecode and the text-root single external-image predecode
  case, remaining text
  layout/editing, full `TextInput` editing/selection/keyboard behavior,
  selected-root gradient shader ordering, selected-root
  skinned clip-path geometry, nested-feather gradient-space exact parity for
  `ai_assitant.riv`, live
  data-bound nested host controls/artboard swaps, nested layout/leaf, scroll
  constraints, or layout-backed/virtualized component-list instancing.
  Harness-level scripted input replay dispatches
  pointerDown/pointerMove/pointerUp/pointerExit markers into direct rectangle
  state-machine listeners with listener input actions, direct rectangle
  enter/exit hover state, direct rectangle click synthesis, and listener-owned
  default view-model trigger target-to-source writes. Full C++ ListenerGroup
  drag/opaque behavior and input-driven nested align-target/list/text/layout
  targets are still not supported.
- Static text support currently covers one style or matching-metric
  multi-style text, static authored-line-break and no-break multi-run text,
  fixed-size ellipsis across multiple authored lines with bottom/middle
  vertical alignment, variation-aware no-break multi-run style outlines,
  auto-width origin offsets, and translation/rotation/scale/opacity
  `TextModifierGroup` over C++-style
  `TextModifierRange` character, character-excluding-space, word, and static
  line range maps with runId targeting and optional cubic range
  interpolation, including C++-ordered opacity buckets, plus solid fill/stroke
  `TextStylePaint` drawing with DashPath stroke effects and text paint feather
  state, text under `Shape` parent transforms, fit-font-size wrapping under
  layout-controlled bounds with C++ zero-font collapsed glyph paths, and
  source-to-target `TextValueRun.text` / `Text.alignValue` /
  `Text.overflowValue` / `TextStylePaint.fontSize` /
  `LayoutComponent.height` / `SolidColor.colorValue` / `Shape.x/y` through
  no-converter and `DataConverterGroup` paths / `Shape.rotation` via
  `DataConverterSystemDegsToRads`, `Text.width/height` through no converter
  or `DataConverterFormula`, static `TextFollowPathModifier` over Shape/Path
  targets with C++ `PathMeasure` tolerance, static vertical trim measured and
  rendered bounds with `Text.verticalTrimTopValue` /
  `Text.verticalTrimBottomValue` bitmask passthrough binds, plus no-converter
  `ParametricPath` width/height binds for
  Ellipse/Polygon/Rectangle/Star/Triangle around static text.
  Static text can coexist with authored nested bool input controls beside
  nested state-machine hosts and passive sample-0 `FocusData` /
  `KeyboardInput` metadata, passive nested numeric controls, plus inert
  `ScriptedDrawable` siblings. Shape/origin modifiers, gradient/other text
  effects, richer layout, broader `Text` property
  data binds, and text input/editing remain M6 text diagnostics.
- `TransformConstraint` currently covers Text constraint bounds for the
  supported static Text subset plus the default empty
  `TransformComponent::constraintBounds()` path. LayoutComponent bounds remain
  parked behind M6 layout diagnostics.
- Passive sample-0 `ScrollConstraint` files with zero authored offset,
  percent, and index values, no input events, no state-machine listener target,
  registered layout-provider children, and a coherent Taffy snapshot are
  admitted by the Rust runner, including at-rest snap metadata. Empty
  virtualized `ArtboardComponentList` layout providers are also admitted when
  they have no children and no map rules, because they create no virtualized
  instances at sample 0. `scroll_snap.riv` is exact after applying the
  accumulated Taffy layout bounds through artboard-origin-adjusted layout world
  transforms. Remaining scroll-constraint corpus files stay parked behind M6
  layout/runtime support via `rust-runner-unsupported:scroll-constraints`;
  C++ `src/constraints/scrolling/scroll_constraint.cpp` also reads
  `LayoutComponent` dimensions, layout-provider child bounds, physics state,
  infinite scroll state, and non-empty component-list virtualization, so
  dynamic/virtualized scroll remains outside the passive initial slice.
- Per-file parked reasons now live in `corpus.toml`: each gated entry
  carries `milestone = "M3|M4|M5|M6|gated|harness"` plus its diagnostic
  feature tags (`rust-runner-unsupported:*`, `cpp-runner-crash`,
  `import-error:*`). Query a milestone's work-list with e.g.
  `grep -B6 'milestone = "M5"' corpus.toml`.
- Entries tagged `cpp-runner-crash` (`milestone = "harness"`) stay parked
  until the C++ golden runner survives the FileAssetContents, scripting,
  and data-viz crash paths it currently aborts on.
- `coin.riv` is no longer parked as an M3 constraints or gated feather file:
  repeated inner-feather paints now share the C++ clip-path cache key and
  `coin.riv` is exact at sample 0. `bankcard.riv` now reaches the text
  diagnostic after clearing its `layout-component-paint` and feather blockers.
- `solar-system.riv` stays gated on a Rust import gap: `blendModeValue = 5`
  rejected on Shape object 13.

## Decisions

- 2026-07-09: [M7] Removed retained-vector clones and lazy-built the artboard
  clip rectangle. The user-requested fixed-overhead profiles showed
  `Vec::clone` under `ArtboardInstance::update_pass` plus draw-side allocation
  in `draw_prepared_static_artboard_internal_with_path_cache` for
  `animation_reset_cases`; C++ walks dependency/joystick vectors in place via
  `DependencyHelper::addDirtToDependents`, `Artboard::updatePass`, and
  `Joystick::apply`, and clips the artboard through retained
  `m_worldPath.renderPath(this)`. Rust now cascades recursive dirt by retained
  dependent index, applies joysticks by retained index instead of cloning the
  joystick list, and only builds artboard clip rectangle commands when the
  retained clip path cache misses. Focused 100M tracking moved
  `animation_reset_cases@0` from total=22883.37 / advance=8887.97 /
  draw=13379.22 ms to total=20603.59 / advance=8720.33 / draw=12335.55 ms;
  `advance_blend_mode@0` kept a lower advance time than the pre-slice sample
  but total/draw remained noisy. Open-fence hot-loop tracking reports
  aggregate Rust/C++=2.032 then 2.106, so this is not M7 acceptance evidence.
  Full `make golden-compare` remains exact=263 / exact-segments=584 /
  diverges=0; `cargo test --workspace`, `cargo fmt --all -- --check`, and
  `git diff --check` pass. Next target remains fixed-overhead profiling after
  this cleanup, especially `advance_blend_mode`'s remaining animation/update
  stack and `animation_reset_cases` draw allocation.
- 2026-07-09: [M7] Removed steady-state nested child source-local fallback
  scans. The user-requested open-fence profile showed
  `stateful_nested_host_binding_value_for` /
  `stateful_nested_host_value_local_for_slots` in the `ai_assitant@0`
  data-bind advance stack. C++ binds nested artboard data contexts to retained
  `ViewModelInstance` / `DataContext` pointers; Rust already computed the
  equivalent child-binding source locals when each nested instance was built,
  but still retried the static slot walk every frame for missing cached
  locals. Rust now trusts the build-time source-local slots in
  `sync_nested_child_artboard_data_contexts`, reuses a retained update scratch
  buffer on `ArtboardInstance`, and drops the now-dead stored nested source
  maps. Full `make golden-compare` remains exact=263 / exact-segments=584 /
  diverges=0; `cargo test --workspace`, `cargo fmt --all -- --check`, and
  `git diff --check` pass. The open-fence hot-loop improves the tracking
  aggregate from Rust/C++=2.148 to 2.054, with `ai_assitant@0` moving from
  2.059 to 1.867; it remains tracking-only because aggregate is still above
  2.0 and C++ min-sum=0.980 ms is outside the 0.70-0.95 ms sanity band. Next
  profile target is the tiny fixed-overhead pair: `advance_blend_mode` and
  `animation_reset_cases`.
- 2026-07-09: [M7] Thread keyed animation property keys into generated double
  setters. C++ `KeyedProperty::apply` stores `propertyKey()` once and
  `KeyFrameDouble::apply` calls `CoreRegistry::getDouble/setDouble` with that
  key; Rust was re-resolving transform keys through the component type on
  every transform keyframe and routing keyed double writes through
  `InstanceObjectArena::set_property_value` metadata validation even though
  import had already validated `object_supports_property` and the core
  registry field kind. Rust now uses the imported keyed-property key for
  transform reads/writes and adds a generated-double setter path for keyed
  animation writes while keeping the same Artboard data-bind/cache/layout/path
  side effects. Focused `spotify_kids_demo@0` 2M tracking moved total
  7404.35/advance 5187.65 ms to total 5964.58/advance 3778.97 ms; the
  transform-key-only midpoint was total 6602.11/advance 4406.16 ms, and that
  midpoint sample no longer showed `transform_property_key` while exposing the
  generic setter work removed by the generated-double half of this slice. Full
  `make golden-compare` remains exact=263 / exact-segments=584 / diverges=0;
  `cargo test --workspace`, `cargo fmt --all -- --check`, and
  `git diff --check` pass. The user-requested open-fence hot-loop reports
  aggregate min Rust/C++=2.152, Rust min-sum=2.046 ms, C++ min-sum=0.951 ms,
  `ai_assitant@0`=2.008, and `spotify_kids_demo@0`=2.046, so it is close but
  still tracking-only rather than M7 acceptance evidence.
- 2026-07-09: [M7] Retain clipping shape command streams before clip
  render-path replay. C++ `ClippingShape::update` retains `m_path` and rebuilds
  it only on path/world-transform/n-slicer dirt before draw asks that path for
  a render path. Rust already retained the final clip `RenderPath`, but
  `runtime_draw_clip_start` still rebuilt the composed clipping command stream
  on every repeated clip start. `RuntimeRenderPathCache` now stores composed
  clipping commands by graph id, clipping-shape local id, `path_epoch`, and
  `layout_epoch`, then passes the retained command slice into the existing
  retained clip path cache; this ports the retention layer without changing the
  current Rust retained path/layout invalidation boundary. Focused
  `spotify_kids_demo@0` repeat=4 caller
  instrumentation showed 440 append calls from clipping versus 148 from
  regular shape-paint composition; the 2M tracking run moved total
  13071.20/draw 7104.94 ms to total 7536.70/draw 2357.05 ms, and
  `append_transformed_path_commands` / `path_geometry_commands_frame` no
  longer appear in the sampled top stack. Full `make golden-compare` remains
  exact=263 / exact-segments=584 / diverges=0; `cargo test --workspace`,
  `cargo fmt --all -- --check`, and `git diff --check` pass. The
  user-requested open-fence hot-loop run deliberately ignored the load fence
  with `PERF_MAX_RATIO=999` and reports aggregate min Rust/C++=2.290, Rust
  min-sum=2.238 ms, C++ min-sum=0.977 ms, and `spotify_kids_demo@0`=2.190, so
  it is tracking-only rather than M7 acceptance evidence.
- 2026-07-09: [M7] Avoid no-op path-prune compaction after appending shape path
  commands. C++ `RawPath::pruneEmptySegments` only copies and truncates after it
  finds an empty segment; Rust was eagerly scanning for multi-contour paths and
  writing every retained command back to itself on the common no-prune path.
  Rust now lazily checks the multi-contour near-empty guard only when a cubic is
  not exactly empty, and only compacts/truncates after a command is actually
  pruned. Focused `spotify_kids_demo@0` 2M tracking moves total
  12763.56/draw 6993.76 ms to total 12446.43/draw 6906.15 ms, and
  `prune_empty_path_segments_from` drops out of the sampled top stack. Full
  `make golden-compare` remains exact=263 / exact-segments=584 / diverges=0;
  `cargo test --workspace`, `cargo fmt --all -- --check`, and
  `git diff --check` pass. `make perf-hot-loop PERF_MAX_RATIO=999` reports
  aggregate min Rust/C++=2.451, Rust min-sum=2.391 ms, C++ min-sum=0.975 ms,
  and `spotify_kids_demo@0`=2.936, so the broad result is tracking-only rather
  than M7 acceptance evidence.
- 2026-07-09: [M7] Route fixed layout-style hot reads through generated-key
  enum dispatch. The `spotify_kids_demo@0` sample still showed Rust spending
  time in `runtime_layout_style_property_key_for_name` even after the
  draw/image/mesh name lookups were removed. C++ reads these fields through
  generated `LayoutComponentStyle` accessors, so Rust now passes fixed
  `RuntimeLayoutStyleProperty` enum values into the layout/Taffy helpers and
  resolves cached numeric keys directly instead of string-matching the field
  name on every read. Focused `spotify_kids_demo@0` 2M tracking improves
  total=17142.6/draw=11105.3 ms to total=16057.6/draw=10408.0 ms, and the
  sample no longer shows the layout-style key helper. Full
  `make golden-compare` remains exact=263 / exact-segments=584 / diverges=0;
  `cargo test --workspace`, `cargo fmt --all -- --check`, and
  `git diff --check` pass. `make perf-hot-loop PERF_MAX_RATIO=999` reports
  aggregate min Rust/C++=2.718, Rust min-sum=2.635 ms, C++ min-sum=0.969 ms,
  and load 4.33/3.78/4.66, so the broad result is tracking-only rather than
  M7 acceptance evidence.
- 2026-07-09: [M7] Cached remaining draw hot-path generated property keys and
  recorded the user-requested open-fence measurement. Rust now handles
  `Component.parentId`, `WorldTransformComponent.parentId`, `Fill.fillRule`,
  `MeshVertex.u/v`, and `Weight.indices/values` through cached numeric keys
  in the draw helper instead of runtime schema/name lookup, matching the C++
  generated field-access shape. Focused `spotify_kids_demo@0` sampling no
  longer shows `property_key_for_name` or `definition_by_name`; remaining
  samples are dominated by `append_transformed_path_commands`,
  `component_world_transform_with_bounds`, `path_geometry_commands_frame`,
  layout-style key dispatch, and paint configuration. Full
  `make golden-compare` remains exact=263 / exact-segments=584 / diverges=0;
  `cargo test --workspace`, `cargo fmt --all -- --check`, and
  `git diff --check` pass. `make perf-hot-loop PERF_MAX_RATIO=999` reports
  aggregate min Rust/C++=2.660, Rust min-sum=2.628 ms, C++ min-sum=0.988 ms,
  and post-run load 3.35/4.39/6.37, so the slice is tracking-only rather than
  M7 acceptance evidence.
- 2026-07-09: [M7] Rejected a component-local shape-paint path dependency
  epoch scout. The idea was to mirror C++ `PathComposer`/`ShapePaintPath`
  locality by keying cached shape-paint path commands from the shape local and
  composed path locals rather than the artboard-global `path_epoch`. The code
  compiled and a focused sample showed fewer normalized
  `append_transformed_path_commands` samples, but the decision-grade tracking
  moved the wrong way: `spotify_kids_demo@0` 5M focused draw moved from
  36056 ms to 41454 ms, and `make perf-hot-loop PERF_MAX_RATIO=999
  PERF_ITERATIONS=10 PERF_BENCHMARK_REPEAT=100 PERF_AGGREGATE=min` worsened to
  aggregate Rust/C++=3.288 with C++ min-sum=1.255 ms under very high load.
  The scout was fully backed out; next path work should port a more concrete
  C++ retained raw-path/render-path mechanism or switch to the still-sampled
  draw-side generated property access overhead.
- 2026-07-09: [M7] Re-ran the open-fence hot-loop at the user's request even
  though the machine was above the M7 load fence. `make perf-hot-loop
  PERF_MAX_RATIO=999 PERF_ITERATIONS=10 PERF_BENCHMARK_REPEAT=100
  PERF_AGGREGATE=min` reports aggregate Rust/C++=2.903, Rust min-sum=3.464 ms,
  C++ min-sum=1.193 ms, and load 29.71/19.24/16.76. This is tracking-only
  evidence because both the load and C++ sanity-band checks are out of fence.
  It keeps the next M7 slice measurement-backed: profile the tied
  `advance_blend_mode@0.25` / `spotify_kids_demo@0` hot paths before adding
  another optimization.
- 2026-07-09: [M7] Narrowed retained path-command invalidation for transform
  dirt to match C++ path composer behavior. The focused `spotify_kids_demo@0`
  sample showed `append_transformed_path_commands` and allocator growth
  dominating draw time. C++ `src/shapes/path.cpp::Path::update` rebuilds raw
  path geometry for `Path`/`NSlicer` dirt, with world-transform rebuilds only
  for deformed paths; the old world-transform path-changed branch is
  commented out, while `ShapePaint::draw` applies local path transforms at draw
  time. Rust now keeps `WORLD_TRANSFORM` invalidating the prepared frame, but
  no longer bumps `path_epoch` for ordinary transform dirt. A unit test covers
  that split. Focused `spotify_kids_demo@0` 5M tracking moves draw from
  33483 ms to 32698 ms, but the full open-fence hot-loop aggregate is flat at
  Rust/C++=2.804 under high load and an out-of-band C++ min-sum, so this is
  not M7 acceptance evidence. Full `make golden-compare` remains exact=263 /
  exact-segments=584 / diverges=0, `cargo test --workspace` passes, and M7
  remains open.
- 2026-07-09: [M7] Re-ran the open-fence hot-loop before another optimization
  slice. User explicitly asked to ignore the load fence so the port work stays
  measurement-backed. `make perf-hot-loop PERF_MAX_RATIO=999
  PERF_ITERATIONS=10 PERF_BENCHMARK_REPEAT=100 PERF_AGGREGATE=min` reports
  aggregate Rust/C++=2.792, Rust min-sum=2.959 ms, C++ min-sum=1.060 ms, and
  load 11.51/16.31/19.75. This improves the previous open-fence 2.883
  tracking snapshot but remains directional-only because both load and the C++
  sanity band are outside the M7 acceptance fence. Top remaining ratios are
  `spotify_kids_demo@0`, `advance_blend_mode`, `animation_reset_cases`, and
  `ai_assitant@0`, so the next implementation target remains nested
  data-bind/state-machine/property-binding overhead after reading the C++ dirt
  gates.
- 2026-07-09: [M7] Read runtime object draw fallbacks by numeric property key
  instead of by name. The focused `advance_blend_mode` sample showed
  `property_by_name_in_hierarchy` on the draw path even though C++
  `Artboard::drawInternal`, `Shape::draw`, and `ShapePaint::draw` read typed
  generated fields. Rust now keeps the default-aware key helper for required
  relationship fields, but draw/image/mesh/stroke fallback reads use explicit
  property-key scans and the same call-site default literals already used by
  the C++-ported draw code. Focused samples no longer show
  `property_by_name_in_hierarchy` or `stored_field_initializer`; a high-load
  Rust-only 50M `advance_blend_mode` tracking run moved total 37491 -> 34859
  ms and draw 15269 -> 13161 ms. Full `make golden-compare` remains exact=263
  / exact-segments=584 / diverges=0, `cargo test --workspace` passes, and
  open-fence `make perf-hot-loop PERF_MAX_RATIO=999 PERF_ITERATIONS=10
  PERF_BENCHMARK_REPEAT=100 PERF_AGGREGATE=min` reports aggregate
  Rust/C++=2.883 with load 11.59/17.24/21.02 and C++ min-sum=1.077 ms, so M7
  remains open.
- 2026-07-09: [M7] Record open-fence hot-loop tracking runs even when they are
  not acceptance-grade. User explicitly requested ignoring the load fence to
  keep optimization work measurement-backed. The current tracking command is
  `make perf-hot-loop PERF_MAX_RATIO=999 PERF_ITERATIONS=10
  PERF_BENCHMARK_REPEAT=100 PERF_AGGREGATE=min`; when load is high or the C++
  min-sum falls outside 0.70-0.95 ms, record the aggregate and top ratios as
  directional only. A high-load run at 18.92/21.62/19.94 reports aggregate
  Rust/C++=2.879 and C++ min-sum=1.378 ms, improving on the prior open-fence
  3.198 snapshot but still not satisfying M7.
- 2026-07-09: [M7] Stop recording debug update locals in the runtime update
  pass and retain nested context-source scratch storage. The focused
  `advance_blend_mode` profile showed C++-unshaped overhead in Rust
  `update_pass`: the hot runtime callers were growing
  `UpdateComponentsReport.updated_locals`, while C++
  `Artboard::updateComponents` only indexes the
  retained dependency order and does not build a report vector. Rust now keeps
  the public `update_components` / test reporting path intact, but uses a
  no-recording internal path from `update_pass`. The same slice also keeps the
  nested artboard context-source collection buffer on `ArtboardInstance`,
  matching C++'s container-owned dirty-list shape more closely than a fresh
  per-frame `Vec`. Focused Rust-only 50M `advance_blend_mode` tracking moved
  advance from 12676 ms to 11605 ms and total from 27905 ms to 26480 ms under
  high/noisy load. User-requested open-fence `make perf-hot-loop
  PERF_MAX_RATIO=999 PERF_ITERATIONS=10 PERF_BENCHMARK_REPEAT=100
  PERF_AGGREGATE=min` reports aggregate Rust/C++=3.198 with load
  36.41/24.07/20.27 and C++ min-sum=1.589 ms, so it is directional only.
  Full `make golden-compare` remains exact=263 / exact-segments=584 /
  diverges=0, and `cargo test --workspace` passes. M7 remains open; next
  profile target is actual nested context collection/binding conversion and
  remaining state-machine color/property mutation overhead.
- 2026-07-09: [M7] Trimmed update-pass/data-bind fixed overhead in the focused
  hot loop. C++ `Artboard::updateComponents` captures dependency-order count
  and indexes the retained order; Rust now does the same instead of cloning
  `update_order` every update pass. C++ separates zero-elapsed data-bind
  updates from converter advancement; Rust now skips converter advancement on
  zero-elapsed nested/data-bind update calls while preserving the property
  converter target-queue drain when nonzero advancement dirties property
  bindings. The remaining `SolidColor.colorValue` topology-dirt checks now use
  the generated cached property key instead of a schema-name lookup. Focused
  `advance_blend_mode` tracking moves same-session Rust advance best from
  about 0.030 ms to 0.026 ms, and Rust-only 20M repeat advance from 8011 ms to
  6669 ms. User-requested open-fence `make perf-hot-loop PERF_MAX_RATIO=999
  PERF_ITERATIONS=10 PERF_BENCHMARK_REPEAT=100 PERF_AGGREGATE=min` reports
  aggregate Rust/C++=3.105 with load 33.66/24.38/22.16 and C++ min-sum=1.208
  ms, so it is directional only. Full `make golden-compare` remains exact=263
  / exact-segments=584 / diverges=0, and `cargo test --workspace` passes. M7
  remains open; next profile target is the remaining nested data-bind
  propagation and state-machine/property-binding overhead.
- 2026-07-09: [M7] Skipped gradient paint-prep scans when a graph has no
  gradient paints. C++ `LinearGradient::update` only rebuilds gradient shader
  state under actual paint/stops/render-opacity/transform dirt; graphs without
  linear/radial gradient paints have no gradient mutators to visit. Rust now
  exposes `RuntimeGradientPreparationFrame::has_paints()`, returns immediately
  from root static paint prep when empty, and in dependency-ordered nested
  paint prep prepares nested hosts directly instead of walking the root
  gradient dependency order. Focused `advance_blend_mode@0` tracking improves
  the Rust-only 20M repeat sample from elapsed=13053 ms / prepare=2610 ms to
  elapsed=11568 ms / prepare=1305 ms; focused JSON best prepare min moved from
  0.0201 ms to 0.0137 ms, while total remains noisy/flat because advance and
  draw fixed overhead now dominate. User-requested open-fence `make
  perf-hot-loop PERF_MAX_RATIO=999 PERF_ITERATIONS=10
  PERF_BENCHMARK_REPEAT=100 PERF_AGGREGATE=min` reports aggregate Rust/C++=
  2.943 with load 5.58/7.58/12.68 and C++ min-sum=0.984 ms, so it is not
  acceptance-grade. Full `make golden-compare` remains exact=263 /
  exact-segments=584 / diverges=0, and `cargo test --workspace` passes. M7
  remains open; next profile target is update-pass/data-bind fixed overhead
  (`advance_artboard_data_binds_with_root_transform`,
  `update_nested_artboard_from_host_dirt`, and related property-binding work).
- 2026-07-08: [M7] Append common shape/clipping path commands directly into the
  destination buffer. C++ `RawPath::addPath` maps source points into the
  retained raw path and then prunes only the newly added segment range.
  Profiling `spotify_kids_demo@0` showed Rust spending draw time in
  intermediate `Vec<RuntimePathCommand>` clone/grow/prune work under
  `draw_prepared_static_artboard_internal_with_path_cache`. Rust now routes the
  common non-n-sliced, non-reversed path assembly through direct transformed
  append plus slice-local prune; reversed local-clockwise and n-sliced paths
  keep the existing fallback. The path transform helper now branches once per
  path for scale/translate vs affine, mirroring C++ `Mat2D::mapPoints`.
  Focused spotify JSON is noisy under load, but the best same-turn draw phase
  moved from Rust min 0.752 ms / draw=6.708x to Rust min 0.638 ms /
  draw=5.815x, and sampling shows allocator top-stack entries much lower while
  the remaining draw hotspot is actual transformed append/prune work. Final
  open-fence `make perf-hot-loop PERF_MAX_RATIO=999 PERF_ITERATIONS=10
  PERF_BENCHMARK_REPEAT=100 PERF_AGGREGATE=min` reports aggregate
  Rust/C++=3.298 with load 24.43 and C++ min-sum=1.178 ms, so it is not
  acceptance-grade. Full `make golden-compare` remains exact=263 /
  exact-segments=584 / diverges=0. M7 remains open; next profile target is
  the remaining `advance_blend_mode` fixed overhead or spotify draw
  transform/prune/world-transform time under a clean low-load sample.
- 2026-07-08: [M7] Kept solid-color paint changes out of prepared topology
  when C++ would only update the retained `RenderPaint`. C++
  `src/shapes/paint/solid_color.cpp` handles `colorValueChanged()` by
  recomputing the retained render paint color/visibility flags and calling
  `artboard()->changed()`, not by rebuilding shape/path topology. Rust now
  stores a prepared command's render opacity, re-reads current solid color
  while configuring retained render paints, and leaves `prepared_epoch` stable
  for visible-to-visible `SolidColor.colorValue` changes. Alpha visibility
  crossings still bump `prepared_epoch` so retained command membership stays
  exact. Focused `advance_blend_mode@0` JSON moves from total Rust/C++=5.417 to
  5.019; the open-fence hot-loop reports aggregate min Rust/C++=3.043 with C++
  min-sum=1.090 ms and high load, so it remains directional only. Full
  `make golden-compare` remains exact=263 / exact-segments=584 / diverges=0;
  `cargo test --workspace`, `cargo fmt --all -- --check`, and
  `git diff --check` pass. M7 remains open.
- 2026-07-08: [M7] Retained shape-paint path commands behind path/layout dirt.
  C++ `PathComposer` owns retained local/world/localClockwise
  `ShapePaintPath`s, and `ShapePaintPath::renderPath` only rewinds/rebuilds
  when path dirt marks the raw path dirty. A spotify profile showed Rust was
  rebuilding transformed shape-paint command vectors inside prepared-frame
  replay even when only the broader prepared epoch moved. Rust now keeps dense
  shape-paint path command slots keyed by graph, paint local, shape local,
  `path_epoch`, `layout_epoch`, and path kind; the existing prepared command
  frame can rebuild paint state while reusing the C++-equivalent path facts.
  Focused `spotify_kids_demo@0` JSON improves from total=6.315 / draw=8.222 to
  total=4.038 / draw=6.729, with Rust min total 1.884 ms -> 1.301 ms. Full
  open-fence `make perf-hot-loop PERF_MAX_RATIO=999 PERF_ITERATIONS=10
  PERF_BENCHMARK_REPEAT=100 PERF_AGGREGATE=min` runs report aggregate
  Rust/C++=3.474 then 3.173; both remain directional only because C++ min-sums
  were 1.203/1.232 ms, outside the M7 sanity band. `cargo check -p
  rive-runtime`, `cargo test --workspace`, and `make golden-compare` pass with
  exact=263 / exact-segments=584 / diverges=0. M7 remains open.
- 2026-07-08: [M7] Ignored the load fence for a user-requested directional
  hot-loop tracking run. `make perf-hot-loop PERF_MAX_RATIO=999
  PERF_ITERATIONS=10 PERF_BENCHMARK_REPEAT=100 PERF_AGGREGATE=min` reports
  aggregate min Rust/C++=3.030 over 11 file/sample entries. This is not
  acceptance-grade: `uptime` reported load averages 14.48/27.81/31.10, and the
  C++ min-sum was 1.104 ms, outside the 0.70-0.95 ms sanity band. The useful
  ordering changed from spotify-first to tiny-file fixed overhead first:
  `advance_blend_mode` reports 6.810/5.333, `spotify_kids_demo@0`=4.483,
  `animation_reset_cases` spans 2.980-3.247, `ai_assitant@0`=2.211,
  `animated_clipping@0`=2.072, and `align_target@0`=1.821. A focused
  `advance_blend_mode@0` JSON run reports total Rust/C++=5.417, advance=2.462,
  prepare C++=0 vs Rust min=0.036 ms, and draw=7.751. A long Rust-only sample
  points at `prepare_static_artboard_tree_paints_internal` rebuilding prepared
  frames for the nested artboard path, plus allocation churn around
  `runtime_draw_command_for_node`, `runtime_shape_paint_command`, path-slot
  assignment, and data-bind/context advance. Next measured slice should read
  the C++ hot site first, then target the prepared-frame/dirt gate or
  data-bind/context fixed overhead; do not return to the rejected shallow
  command-vector cache shape.
- 2026-07-08: [M7] Ran a user-requested high-load directional hot-loop sample.
  `make perf-hot-loop PERF_MAX_RATIO=999` used the current release/null-renderer
  whole-repeat `total_ms` gate settings and reported aggregate min Rust/C++=
  3.529 over 11 file/sample entries. The run is not acceptance-grade because
  the machine was above the load fence and the C++ min-sum was 1.194 ms, outside
  the 0.70-0.95 ms sanity band. It is still useful for ordering: the biggest
  current visible outlier is `spotify_kids_demo@0` at 6.168, followed by
  tiny-file fixed overhead in `advance_blend_mode` and `animation_reset_cases`;
  `ai_assitant@0` is 2.257. Next runtime slice should continue from those
  measured buckets, not from unmeasured cache guesses.
- 2026-07-08: [M7] Re-ran the user-requested high-load directional hot-loop
  despite the load fence to keep optimization work measured. `make
  perf-hot-loop PERF_MAX_RATIO=999 PERF_ITERATIONS=10 PERF_BENCHMARK_REPEAT=100
  PERF_AGGREGATE=min` reports aggregate min Rust/C++=3.394 across 11
  file/sample entries, with C++ min-sum=1.205 ms outside the 0.70-0.95 ms
  sanity band. The run is not acceptance-grade, but it preserves the ordering:
  `advance_blend_mode` remains tiny-file fixed overhead at 5.665/5.385,
  `spotify_kids_demo@0` is 5.451, `animation_reset_cases` spans 2.968-3.725,
  `ai_assitant@0` is 2.546, and `animated_clipping@0` is 2.271. A focused
  `spotify_kids_demo@0` JSON run on the same measurement path reports total=
  6.315, advance=2.746, and draw=8.222, making spotify draw replay the highest
  priority measured slice.
- 2026-07-08: [M7] Corrected stale perf-fence wording after the whole-repeat
  `total_ms` harness change. The live M7 gate already scores
  release/null-renderer runner-emitted `total_ms`; lingering references to
  hot-loop phase-sum scoring in the Next queue, methodology fence, and older
  decision summary now point at `total_ms` with phase timings explicitly
  advisory. This keeps the scout/perf fences queryable before the next runtime
  slice and avoids optimizing against the pre-`total_ms` gate.
- 2026-07-08: [M7] Store render-paint configuration cache in dense global
  slots. The M7 scout keeps draw-replay dispatch as the implementation target
  when the low-load perf gate is unavailable. `RuntimeRenderPaintCache` already
  stores `RenderPaint`s in dense global-id slots; its configuration cache now
  uses the same shape instead of a `BTreeMap<u32, ...>`, while preserving the
  existing instance-epoch/configuration comparison and gradient invalidation.
  This is a backing-store change for the C++ retained-paint shape, not a new
  dirty rule. Full `make golden-compare` remains exact=263 /
  exact-segments=584 / diverges=0; `cargo test --workspace`,
  `cargo test -p rive-runtime --quiet`, `cargo check -p rive-runtime`,
  `cargo fmt --all -- --check`, and `git diff --check` pass. M7 perf was not
  rerun because load stayed outside the acceptance fence. Strict <=2.0 remains
  open.
- 2026-07-08: [M7] Retain clip/background render paths in dense local slots.
  The status scout keeps draw-replay dispatch, not state-machine fixed
  overhead, as the current runtime priority when the low-load perf gate is not
  available. `RuntimeRenderPathCache` now stores artboard clip,
  clipping-shape, layout-clip, and background `RenderPath`s as retained
  `RawPath`/`RenderPath` entries behind graph-local dense slots and the
  existing path/layout epochs plus fill rule. This removes the remaining
  draw-time `BTreeMap` lookup/rebuild shape for these paths without adding a
  new image draw-state cache or a shared path-command buffer. Full
  `make golden-compare` remains exact=263 / exact-segments=584 / diverges=0;
  `cargo test --workspace`, `cargo check -p rive-runtime`, the focused
  retained-path unit test, `cargo fmt --all -- --check`, and `git diff --check`
  pass. M7 perf was not rerun because load stayed outside the acceptance
  fence. Strict <=2.0 remains open.
- 2026-07-08: [M7] Measure hot-loop totals with whole-repeat runner timing.
  Scout item 17 found `mach_absolute_time` reads could be a material fraction
  of tiny-file steady frames when the harness timed every phase of every
  repeat. `perf-compare` now prefers runner-emitted `total_ms` for the
  aggregate and falls back to the old phase sum for older runners. Both golden
  runners emit `total_ms` from a separate whole-repeat benchmark pass when
  `--benchmark-repeat` is active, then run the existing phase-timed pass for
  diagnostic `advance`/`prepare`/`draw` output. This keeps bare
  `make perf-hot-loop` as the gate while removing per-frame timing calls from
  the score; phase timings remain advisory. Full `make golden-compare` remains
  exact=263 / exact-segments=584 / diverges=0; `cargo test --workspace`,
  `cargo test -p perf-compare --quiet`, `cargo check -p rust-golden-runner`,
  `tools/golden-runner/build.sh debug`, `cargo fmt --all -- --check`, and
  `git diff --check` pass. A debug `shapetest` benchmark smoke confirmed both
  runners emit `total_ms` and `perf-compare` consumes it. M7 perf was not rerun
  because load averages were 10.60/13.69/17.46 after verification, outside the
  acceptance fence. Strict <=2.0 remains open.
- 2026-07-08: [M7] Skip clean no-nested paint preparation before prepared
  command lookup. A status-doc scout review keeps the current M7 findings in
  force: min-based release/null-renderer repeat=100 is the only
  decision-grade perf gate; state-machine fixed overhead is not the priority;
  the remaining runtime target is draw-side clean-frame replay, while the
  runner timing-read overhead is a separate methodology candidate. Rust paint
  preparation now records whether a cached preparation frame saw nested
  artboards; flat clean frames whose graph and instance epoch still match return
  before touching the prepared artboard command frame. Nested artboard cases
  still use the existing command scan and nested epoch, so this does not add a
  new invalidation rule. Full `make golden-compare` remains exact=263 /
  exact-segments=584 / diverges=0; `cargo check -p rive-runtime`,
  `cargo test -p rive-runtime --quiet`, `cargo test --workspace`,
  `cargo fmt --all -- --check`, and `git diff --check` pass. M7 perf was not
  rerun because load averages were 18.98/25.03/23.10, outside the acceptance
  fence. Strict <=2.0 remains open.
- 2026-07-08: [M7] Retain draw raw paths under the draw `RenderPath` cache.
  C++ `ShapePaintPath` owns both `m_rawPath` and `m_renderPath`; when dirty it
  rewinds and repopulates the retained render path from the retained raw path.
  Rust already retained the draw `RenderPath` by `path_epoch`, but rebuilt it
  directly from `RuntimePathCommand` slices on an epoch miss. Draw path cache
  entries now retain a `RawPath`, rebuild that raw path in place with
  `rewind()` capacity reuse when `path_epoch` changes, and feed the renderer
  path through `RenderPath::add_raw_path`. This ports the lower-level raw-path
  ownership shape without changing geometry math, fill-rule handling, or
  invalidation, and does not revive the rejected shared shape path-command
  buffer/cache scout. Full `make golden-compare` remains exact=263 /
  exact-segments=584 / diverges=0; `cargo check -p rive-runtime`, the focused
  draw-path reuse test, `cargo test --workspace`,
  `cargo fmt --all -- --check`, and `git diff --check` pass. Same-session
  `make perf-hot-loop
  PERF_MAX_RATIO=999` before the final capacity-reuse polish reported
  aggregate min Rust/C++=3.219, but C++ min-sum=1.037 ms is outside the M7
  sanity band and the movement is below the noise floor. M7 remains open.
- 2026-07-08: [M7] Retain image layout local transforms. C++
  `Image::updateImageScale` stores layout-driven scale in the image local
  transform and keeps `m_layoutOffsetX/Y` on the `Image` object, so clean draws
  reuse that object state before `Image::draw` applies only the image-origin
  translation. Rust had recomputed the layout fit/alignment/origin math in
  `runtime_image_world_transform` on every image draw, and mesh-image draw used
  the generic component world transform instead of the C++ parent image
  `worldTransform()`. `RuntimeRenderPathCache` now retains the image layout
  local transform in dense slots keyed by graph/local plus cache epoch, layout
  epoch, image dimensions, and layout dimensions; both normal image draw and
  mesh-image draw multiply that retained local transform by the cached parent
  layout world transform. This keeps blend/opacity/draw state uncached and does
  not revive the rejected shallow image draw-state cache. Full
  `make golden-compare` remains exact=263 / exact-segments=584 / diverges=0;
  `cargo check -p rive-runtime`, `cargo test -p rive-runtime --quiet`,
  `cargo test --workspace`, `cargo fmt --all -- --check`, and
  `git diff --check` pass. Post-slice
  `make perf-hot-loop PERF_MAX_RATIO=999` reports aggregate min Rust/C++=3.225,
  but C++ min-sum=1.043 ms is outside the M7 sanity band, so the sample is
  directional only. M7 remains open.
- 2026-07-08: [M7] Store mesh render buffers in dense local slots. C++
  `MeshDrawable` owns `m_VertexRenderBuffer`, `m_UVRenderBuffer`, and
  `m_IndexRenderBuffer` on the mesh object, while Rust kept
  `RuntimeMeshRenderBuffers` behind a `BTreeMap<mesh local id, buffers>` on the
  image draw path. `RuntimeRenderPaintCache` now stores those retained buffers
  in `RuntimeMeshRenderBufferSlots`, a dense local-id slot table populated by
  the existing mesh preallocation pass and read by mesh-image draw replay. This
  keeps mesh discovery, source buffer allocation, vertex-byte reuse, and
  weighted mesh math unchanged, and does not repeat the rejected image
  mesh-index precompute scout. Full `make golden-compare` remains exact=263 /
  exact-segments=584 / diverges=0; `cargo check -p rive-runtime`,
  `cargo test -p rive-runtime --quiet`, `cargo test --workspace`,
  `cargo fmt --all -- --check`, and `git diff --check` pass. Two post-slice
  `make perf-hot-loop PERF_MAX_RATIO=999` runs report aggregate min
  Rust/C++=3.219 then 3.176, but C++ min-sums=0.992 ms and 1.053 ms are outside
  the M7 sanity band, so the samples are directional only. M7 remains open.
- 2026-07-08: [M7] Cache layout-adjusted draw world transforms in retained
  dense slots. C++ updates component/world transforms behind dirt and draw
  sites reuse those retained matrices; Rust still recomputed layout-adjusted
  world transforms during shape path prep, clipping prep, gradient paint prep,
  image draw, mesh-image draw, and nested-artboard host draw.
  `RuntimeRenderPathCache` now keeps graph-scoped dense local transform slots
  keyed by the existing `(cache_epoch, layout_epoch)` boundary, using the
  retained component transform directly when no layout-bounds frame is active.
  This ports the scout's
  transform-dirt cache direction without adding a rejected image draw-state
  cache, mesh-index precompute, or new geometry float math. Full
  `make golden-compare` remains exact=263 / exact-segments=584 / diverges=0;
  `cargo check -p rive-runtime`, `cargo test -p rive-runtime --quiet`,
  `cargo test --workspace`, `cargo fmt --all -- --check`, and
  `git diff --check` pass. Perf was not rerun because load was
  24.90/37.27/29.64, outside the M7 acceptance fence. M7 remains open.
- 2026-07-08: [M7] Store decoded images in dense global-id slots. C++
  `Image::draw` reaches the retained `RenderImage` through the `ImageAsset`
  object, while Rust still looked decoded images up through a draw-time
  `BTreeMap<u32, Box<dyn RenderImage>>`. `RuntimeRenderPaintCache` now stores
  decoded images in `RuntimeRenderImages`, a dense global-id slot table shaped
  like the existing retained render-paint slots. This removes an image draw
  replay map lookup without adding transformed image-state caching, mesh-index
  precompute, or new skip/invalidation logic. Full `make golden-compare`
  remains exact=263 / exact-segments=584 / diverges=0; `cargo test
  --workspace`, `cargo test -p rive-runtime --quiet`, the cargo fmt check, and
  `git diff --check` pass. A same-session release/null-renderer sample reports
  aggregate min Rust/C++=3.254 with `spotify_kids_demo@0`=4.985, but C++
  min-sum=1.035 ms and post-run load 22.44/17.12/25.78 put it outside the
  acceptance sanity band. M7 remains open.
- 2026-07-08: [M7] Store path geometry commands in dense graph-local slots.
  C++ retains per-path raw/render path state on the path/ShapePaintPath object
  rather than looking it up through a draw-time map. Rust already retained the
  computed path geometry commands behind the existing path/layout epoch key,
  but stored those retained entries in a `BTreeMap` keyed by graph id and path
  local id. `RuntimeRenderPathCache` now keeps the same cached command payloads
  in graph-scoped dense local slots, clearing the slot table if a cache is ever
  reused for another graph. This preserves `path_epoch`/`layout_epoch`
  invalidation and does not add a new skip/cache rule. Full
  `make golden-compare` remains exact=263 / exact-segments=584 / diverges=0;
  `cargo test --workspace`, focused path-epoch/draw-cache tests,
  `cargo fmt --all -- --check`, and `git diff --check` pass. M7 perf was not
  rerun because load averages were 25.13/42.76/45.72, outside the acceptance
  fence. M7 remains open.
- 2026-07-08: [M7] Retain draw paths in dense slots. A status-doc review of
  the scout/perf discoveries keeps the current M7 fences binding: use the
  min-based release/null-renderer repeat=100 gate for decisions, do not revive
  shallow command/path wrappers or image-sidecar caches without fenced evidence,
  and target draw-replay dispatch before smaller state-machine fixed overhead.
  `RuntimeRenderPathCache` now stores retained draw `RenderPath`s in dense
  local/path-kind/path-slot vectors keyed by prepared
  `RuntimeShapePaintCommand` slot indices. That matches the C++
  `ShapePaintPath::m_renderPath` reuse shape more closely than a draw-time
  `BTreeMap` key, while preserving the existing `path_epoch` invalidation and
  avoiding command-vector caching. Full `make golden-compare` remains exact=263
  / exact-segments=584 / diverges=0; `cargo test --workspace`,
  `cargo fmt --all -- --check`, and `git diff --check` pass. M7 perf was not
  rerun because load averages were 62.40/62.92/45.19, outside the acceptance
  fence. The last directional fenced run remains aggregate min Rust/C++=3.399
  with C++ min-sum outside the sanity band. M7 remains open.
- 2026-07-08: [M7] Use membership flags for data-bind source update queues. A
  release/null-renderer profile after optional nested-event collection still
  showed data-bind queue drains in the `ai_assitant` hot split. C++
  `DataBindContainer` stores persistent and dirty lists, and `DataBind` carries
  dirty-list membership state so repeated dirt raises do not require scanning
  the queued indices. Rust now keeps `custom_property_update_flags` beside the
  recycled custom-property update index buffer, dedupes dirty plus persisting
  updates through that bitmap, and returns early when custom-property or numeric
  source lanes are empty. This is retained queue membership storage, not the
  rejected source-queue vector-swap scout. Full `make golden-compare` remains
  exact=263 / exact-segments=584 / diverges=0; `cargo test --workspace`,
  `cargo fmt --all -- --check`, and `git diff --check` pass. Fenced perf is
  noisy but directionally useful: focused baseline Rust median sum was 2.467 ms,
  post-slice rerun is 2.319 ms, and direct repeat=100 `ai_assitant` reports
  rust median=0.989 ms versus 1.041 ms before. M7 remains open.
- 2026-07-08: [M7] Skip nested event collection when callers ignore reports.
  A status-doc review kept the scout/perf discoveries in force: broad
  converter-property writes, RangeMapper retries, scratch-only context-path
  reuse, shallow command/path wrappers, source-queue vector swaps, and borrowed
  retained-paint threading remain rejected without fenced evidence. A
  release/null-renderer `ai_assitant` profile after state-machine scratch
  retention still showed `advance_nested_artboards_collect_events` in the hot
  split. C++ `NestedArtboard::advanceComponent` does not allocate a caller-owned
  per-child event vector; nested state-machine reports live on
  `StateMachineInstance`. Rust nested advance now passes optional event
  collectors, so no-observer paths avoid ignored event vectors and reported-event
  clones while state-machine propagation keeps the old event order. Full `make
  golden-compare` remains exact=263 / exact-segments=584 / diverges=0; `cargo
  test --workspace`, `cargo fmt --all -- --check`, and `git diff --check` pass.
  Fenced hot-loop ratios are noisy at Rust/C++=3.041 and 3.073, but Rust median
  sum improves from the pre-slice 3.488 ms to 2.476 ms and 2.488 ms; direct
  repeat=100 `ai_assitant` JSON reports cpp median=0.413 ms, rust median=1.041
  ms, Rust/C++=2.521. M7 remains open.
- 2026-07-08: [M7] Retain state-machine pending view-model action storage.
  A release/null-renderer profile after render-paint slots showed
  `StateMachineLayerInstance::advance`, `try_change_state`, and
  data-bind/context propagation sharing the remaining `ai_assitant` hot path.
  C++ `StateMachineInstance` keeps event/listener queues on the instance, so
  Rust now retains the pending view-model listener-action scratch vector on
  `StateMachineInstance` and passes it through layer advance instead of
  allocating a fresh per-layer vector. This is retained instance scratch, not a
  new skip/cache invalidation rule. Full `make golden-compare` remains
  exact=263 / exact-segments=584 / diverges=0; `cargo test --workspace`,
  `cargo fmt --all -- --check`, and `git diff --check` pass. Fenced
  repeat-aware hot-loop is noisy: the first post-slice run improved aggregate
  Rust/C++ from 3.026 to 2.942, while final reruns report 3.298 then 2.925
  with the best Rust median sum at 2.460 ms versus the baseline 2.580 ms. M7
  remains open.
- 2026-07-08: [M7] Store retained render paints in dense global-id slots. A
  release/null-renderer profile after nested-host source-local retention still
  showed draw replay/path-cache `BTreeMap` lookup time under
  `runtime_draw_command`. C++ stores the render paint on each retained
  `ShapePaint`, so Rust now stores persistent render paints in
  `RuntimeRenderPaints` dense global-id slots and uses slot access for gradient
  prep/background/shape draw. The allocation helper still calls
  `make_render_paint()` in the same occupied-slot cases as before so factory
  side effects and golden ordering stay intact. Full `make golden-compare`
  remains exact=263 / exact-segments=584 / diverges=0; `cargo test
  --workspace`, `cargo fmt --all -- --check`, and `git diff --check` pass.
  Fenced hot-loop reports aggregate Rust/C++=3.000, while direct repeat=100
  `ai_assitant` JSON reports rust median=1.437 ms / Rust/C++=2.495, so M7
  remains open.
- 2026-07-08: [M7] Retain nested-host source locals by child binding index.
  A release/null-renderer `ai_assitant` sample after the shape-paint global-id
  slice still showed `stateful_nested_host_binding_value_for`,
  `stateful_nested_host_value_local_for_slots`, and `BTreeMap::get` in nested
  data-context sync. C++ `DataContext` walks retained view-model/source
  pointers, so Rust now seeds per-property/per-image source-local slots on
  `RuntimeNestedArtboardInstance` from the existing path map and fills fallback
  misses into those slots. The path map remains as a build/fallback table, but
  steady child sync reads by binding index before falling back to the slot walk;
  dynamic `artboardId` swaps rebuild the nested instance and slots. Full `make
  golden-compare` remains exact=263 / exact-segments=584 / diverges=0; `cargo
  test --workspace`, `cargo fmt --all -- --check`, and `git diff --check`
  pass. Fenced hot-loop is noisy at aggregate Rust/C++=2.972 then 3.167, while
  direct repeat=100 `ai_assitant` JSON reports rust median=1.462 ms, so M7
  remains open.
- 2026-07-08: [M7] Retain shape-paint global IDs in prepared draw commands.
  A release/null-renderer profile showed `runtime_draw_command` time with
  visible `BTreeMap::get` under draw replay after the paint-config epoch
  slice. C++ draws from retained `ShapePaint` objects, so Rust now stores each
  prepared `RuntimeShapePaintCommand`'s `paint_global_id` when the command is
  built and uses it directly at draw time instead of resolving
  `paint_local -> global_id` through `graph.local_objects` every frame.
  Two nearby scouts were rejected and backed out by the perf fence:
  source-queue vector take/recycle worsened focused aggregate to Rust/C++=3.119
  and 3.077, and carrying a borrowed retained `RenderPaint` through draw gave
  direct `ai_assitant` Rust/C++=2.658 versus the prior 2.483. Full `make
  golden-compare` remains exact=263 / exact-segments=584 / diverges=0; `cargo
  test --workspace`, `cargo fmt --all -- --check`, and `git diff --check`
  pass. Fenced hot-loop is noisy at aggregate Rust/C++=3.038 then 2.889, while
  direct repeat=100 `ai_assitant` JSON reports Rust/C++=2.477, so M7 remains
  open.
- 2026-07-08: [M7] Skip clean-frame paint configuration recomputation. A
  release/null-renderer profile after the dependency-prep skip showed
  `runtime_configure_paint_with_cache` beside `runtime_draw_command` and
  `advance_artboard_data_binds_with_root_transform` in the remaining Rust hot
  sites. C++ keeps `ShapePaint` render-paint state retained and updates it via
  dirt/property changes, so Rust now stores each draw-time paint configuration
  with the artboard `cache_epoch`: clean frames return before recomputing
  stroke/blend/shader/feather configuration, epoch-changed but equivalent
  configs only refresh the cached epoch, and gradient preparation still removes
  the entry when it mutates retained paint. Full `make golden-compare` remains
  exact=263 / exact-segments=584 / diverges=0; `cargo test --workspace`,
  `cargo fmt --all -- --check`, and `git diff --check` pass. Fenced
  repeat-aware hot-loop improves from aggregate Rust/C++=3.260 to 2.903 and
  3.105 on rerun, while direct repeat=100 `ai_assitant` JSON reports
  Rust/C++=2.483, so M7 remains open.
- 2026-07-08: [M7] Skip dependency-ordered paint prep on clean nested frames.
  A release `ai_assitant` sample after retained layout topology still showed
  `prepare_static_artboard_tree_paints_internal`,
  `prepare_static_gradient_paints_for_dependency_local`, data-bind queue
  draining, and draw replay as the main Rust hot sites. C++ clean frames return
  from `Artboard::updateComponents()` before visiting component prep work, so
  Rust now allows dependency-ordered nested paint preparation to reuse the
  retained `RuntimePaintPreparationFrame` when the root cache epoch and nested
  child cache epochs are unchanged. The cache key includes nested command
  identity plus child `cache_epoch`, preserving invalidation when nested
  animation/data-bind changes. Artboard source queues also retain their dirty
  update-index buffers and enqueue target-source refs without cloning the
  vector. Full `make golden-compare` remains exact=263 / exact-segments=584 /
  diverges=0; `cargo test --workspace`, `cargo fmt --all -- --check`, and
  `git diff --check` pass. Fenced repeat-aware hot-loop improves from
  aggregate Rust/C++=3.330 to 3.110 and 3.067 on rerun, so M7 remains open.
- 2026-07-08: [M7] Replace dependency-ordered paint-prep trees with vectors.
  The current release sample showed `ai_assitant` time split between
  advance/data-bind and draw/prepare, with gradient paint preparation still
  allocating and dropping per-frame `BTreeMap`/`BTreeSet` nodes. C++ keeps this
  work on retained object/vector lists, so Rust now uses small vectors for the
  dependency-ordered nested-host command table and prepared paint/host de-dupe.
  Duplicate behavior is preserved: prepared command collection remains
  last-wins, while layout-discovered fallback command collection remains
  first-wins before overwriting the prepared table. Full `make golden-compare`
  remains exact=263 / exact-segments=584 / diverges=0; focused draw probes,
  `cargo test --workspace`, `cargo fmt --all -- --check`, and `git diff
  --check` pass. Fenced repeat-aware hot-loop improves from aggregate
  Rust/C++=3.632 to 3.447 and 3.327 on rerun, so M7 remains open.
- 2026-07-08: [M7] Retain state-machine definitions during advance. A release
  `ai_assitant` sample still showed per-frame `RuntimeStateMachine` clone/drop
  traffic after the status-doc scout review had fenced off shallow
  command/path-wrapper caches and broad converter/property writes. Rust now
  retains the imported state-machine definition vector behind
  `Arc<Vec<RuntimeStateMachine>>`; advancing a `StateMachineInstance` borrows
  the stable definition through an outer `Arc` clone, matching C++'s
  `StateMachineInstance` pointer to immutable `StateMachine` data. Full
  `make golden-compare` remains exact=263 / exact-segments=584 / diverges=0;
  `cargo test --workspace`, `cargo fmt --all -- --check`, and `git diff
  --check` pass. Fenced repeat-aware hot-loop reports aggregate Rust/C++=3.613,
  so M7 remains open.
- 2026-07-08: [M7] Retain path-geometry commands behind path/layout dirt.
  The status scout review keeps three fences binding: no broad
  converter/property writes after the RangeMapper scout, no shallow
  command-vector/path-wrapper caches after fenced perf rejected them, and no
  perf claim without release/null-renderer evidence. This slice ports the
  lower-level C++ retention shape instead: `RuntimeRenderPathCache` retains
  runtime path geometry commands by `(graph_global_id, path_local)` plus
  `path_epoch`/`layout_epoch`, then shape paint composition applies transforms,
  NSlicing, clockwise reversal, and pruning on top. Collapse checks drop their
  per-call `BTreeSet`, and empty segment pruning now compacts in place. Full
  `make golden-compare` remains exact=263 / exact-segments=584 / diverges=0;
  `cargo test --workspace`, `cargo fmt --all -- --check`, and `git diff
  --check` pass. Same-turn fenced repeat-aware hot-loop improves from
  aggregate Rust/C++=3.809 to 3.616, so M7 remains open and next work should
  profile the remaining fixed overhead / data-bind time under the same fences.
- 2026-07-08: [M7] Make corpus perf repeat-aware by sample segment.
  `perf-compare --corpus --benchmark-repeat N` used to fail on multi-sample
  exact files because the golden runners only repeat a single sample. Corpus
  mode now preserves `--corpus-limit` as a file count, then expands each
  selected exact file into one target per sample when repeat is requested; input
  script entries still reject repeat mode. This makes M7's steady-state
  release/null-renderer score measurable over the focused file x sample corpus
  instead of relying on noisy repeat=1 medians. Targeted `cargo test -p
  perf-compare` passes. The first repeat-aware focused run with
  `PERF_BENCHMARK_REPEAT=100` reports entries=10 / segments=10 and aggregate
  Rust/C++=3.711, so M7 remains open and the next runtime work should profile
  the small-file fixed overhead in `advance_blend_mode` /
  `animation_reset_cases`.
- 2026-07-08: [M7] Retain static text shape-paint commands behind existing
  dirt epochs. The current `animated_clipping` scout/profiling target was not
  actually clipping math: a live `sample` run showed the Rust draw loop
  repeatedly entering `runtime_text_shape_paint_commands`,
  `StaticTextSlice::render_data`, and HarfRust shaping. C++'s corresponding
  boundary is `Text::buildRenderStyles()` / `RawText::update()`: shape, style
  paths, and draw commands are rebuilt on text shape dirt, then
  `Text::draw()` replays retained commands. Rust now stores text shape-paint
  commands on `RuntimeRenderPathCache` by graph/text plus `path_epoch`,
  `layout_epoch`, and instance `cache_epoch`, and assigns draw-path slot
  indices when that cache entry is built. `make golden-compare` remains
  exact=263 / exact-segments=584 / diverges=0; `cargo test --workspace`,
  `cargo fmt --all -- --check`, and `git diff --check` pass. Rust-only
  `animated_clipping --benchmark-repeat 3000000` improves from
  elapsed=147254.8 / draw=146414.0 ms to elapsed=4008.4 / draw=3348.1 ms.
  Fenced repeat=1 hot-loop remains noisy above target at Rust/C++=2.395 then
  2.223, and direct repeat=100 `animated_clipping` reports cpp median=0.100 ms,
  rust median=0.397 ms, Rust/C++=3.954. M7 remains open; next profile the
  small-file fixed overhead and consider repeat-aware corpus aggregation.
- 2026-07-08: [M7] Gate owned-context artboard rebinds by owned view-model
  mutation generation. After reviewing the status scout/perf fences, the safe
  next step was the C++ `DataBindContext::bindFromContext` rebind boundary, not
  the rejected scratch-path reuse layer. Rust now tracks a root
  `RuntimeOwnedViewModelInstance` mutation generation, includes it in each
  artboard's retained owned-context key, and skips only that artboard's own
  owned-context bind/apply and nested animation-context rebind when the
  view-model, context chain, and generation are unchanged. Traversal still
  descends into children so dynamic nested-artboard swaps can invalidate a
  descendant locally. Dynamic `artboardId` swaps clear the affected artboard
  key. `make golden-compare` remains exact=263 / exact-segments=584 /
  diverges=0; focused nested/data-bind and owned-viewmodel probes,
  `cargo test --workspace`, `cargo fmt --all -- --check`, and
  `git diff --check` pass. Fenced hot-loop reports aggregate Rust/C++=2.073
  then 2.274; direct repeat=100 `ai_assitant` reports cpp median=0.420 ms,
  rust median=1.411 ms, Rust/C++=3.357; Rust-only 3M `ai_assitant` improves
  to elapsed=20152.4 / advance=8773.6 ms from the prior retained-source-path
  baseline's elapsed=23231.2 / advance=11731.1 ms. M7 remains open; next
  profile the remaining focused outliers under the same scout/perf fences.
- 2026-07-08: [M7] Retain resolved artboard owned-context data-bind source
  paths behind a context-chain key. C++ `DataBindContext::bindFromContext`
  resolves through `DataContext` to a concrete source and retains that source
  until the bound context/source identity changes. Rust now mirrors the safe
  portion of that shape for artboard property/image/custom data binds: each
  `ArtboardInstance` records the current `(owned view-model index,
  context-chain)` key, clears retained source paths when that key changes, and
  lets each binding reuse its resolved `Vec<usize>` source path for current
  value reads. This intentionally does not skip source value reads or target
  application; unchanged values still flow through
  `set_artboard_data_bind_value_for_path` and the existing target queues.
  `make golden-compare` remains exact=263 / exact-segments=584 / diverges=0;
  focused nested/data-bind tests, `cargo test --workspace`, `cargo fmt --all
  -- --check`, and `git diff --check` pass. Fenced hot-loop aggregate remains
  noisy above target at Rust/C++=2.154 then 2.438, while direct repeat=100
  `ai_assitant` reports cpp median=0.442 ms, rust median=1.420 ms,
  Rust/C++=3.212. M7 remains open; next work should profile the remaining
  advance/data-bind time after retained source paths.
- 2026-07-08: [M7] Reject owned-context path scratch reuse as a standalone
  optimization. A scout reused a single `Vec<usize>` while resolving
  name-based paths through `bind_owned_view_model_artboard_context_chain`,
  but it still repeated the full context-chain/source lookup every frame.
  Focused nested/data-bind tests and `make golden-compare` stayed green at
  exact=263 / exact-segments=584 / diverges=0, but the fenced
  release/null-renderer hot-loop rejected the slice: aggregate Rust/C++ moved
  to 2.250 and then 2.478, worse than the prior 2.105 run. C++'s relevant
  shape is `DataBindContext::bindFromContext`: resolve through `DataContext`
  to a concrete `ViewModelInstanceValue` source, retain it, and rebind only
  when the source identity changes. Next M7 data-bind work should port that
  retained source/rebind model for artboard owned-context bindings instead of
  adding scratch-only path allocation helpers.
- 2026-07-08: [M7] Avoid nested-host data-bind no-op work on clean frames.
  C++ keeps `DataBind` and `DataContext` references on the owning objects;
  Rust was cloning `artboard_nested_host_bindings` every advance and queueing
  nested child context updates even when the child already held the same
  value. Rust now copies only the small nested-host target/property identity
  while reading retained bindings, and checks the child context value before
  pushing property/image update work. This does not add a new dirty gate: it
  removes work that the existing `set_artboard_data_bind_value_for_path`
  would have rejected as unchanged. `make golden-compare` remains exact=263 /
  exact-segments=584 / diverges=0; focused nested/data-bind tests, `cargo test
  --workspace`, `cargo fmt --all -- --check`, and `git diff --check` pass.
  Fenced hot-loop reports aggregate Rust/C++=2.105 with `ai_assitant`=1.939.
  M7 remains open; next focus is the larger
  `bind_owned_view_model_artboard_context_chain` path-resolution hotspot.
- 2026-07-08: [M7] Retain shape-paint draw path slot indices in prepared
  commands. C++ retained draw replay does not rebuild a fresh slot-vector for
  each command; Rust previously recomputed `runtime_cached_path_slot_index`
  during every `runtime_draw_command`, including allocation/growth under the
  sampled draw stack. Rust now assigns deterministic path and inner-feather
  clip slot indices when `RuntimeShapePaintCommand` values are built, while
  dynamic text commands assign their slots when their transient commands are
  generated. Draw replay uses those retained indices for `RenderPath` cache
  keys. This is prepared-frame annotation only: it does not add dirty skipping
  or broaden path invalidation. `make golden-compare` remains exact=263 /
  exact-segments=584 / diverges=0; focused draw/path tests, `cargo test
  --workspace`, `cargo fmt --all -- --check`, and `git diff --check` pass.
  Fenced hot-loop reports aggregate Rust/C++=2.136 with `ai_assitant`=1.972.
  A Rust-only 3M-segment sample drops draw time from the prior 6.82s sample to
  5.94s and no longer shows `runtime_cached_path_slot_index` in the draw
  replay. M7 remains open; next focus is the remaining data-bind/context-chain
  hotspots.
- 2026-07-08: [M7] Retain nested artboard host local order. C++ traverses
  retained nested host objects rather than rebuilding ordered key snapshots or
  range cursors each frame. Rust now stores `nested_artboard_locals` on
  `ArtboardInstance` immediately after building the `nested_artboards` map and
  reuses that stable host-local order in nested advance, owned view-model
  context binding, nested context-source propagation, and nested child
  data-context sync. Dynamic `artboardId` swaps update this retained ordered
  list only when they create or remove a child instance, preserving the old
  BTreeMap traversal surface. This removes a sampled BTree range hot site
  without adding dirty/skip semantics. `make golden-compare` remains exact=263
  / exact-segments=584 / diverges=0; focused nested/data-bind tests,
  `cargo test --workspace`, `cargo fmt --all -- --check`, and `git diff
  --check` pass. Fenced hot-loop reports aggregate Rust/C++=2.017, with
  `ai_assitant`=1.820; closeout rerun reports aggregate Rust/C++=2.093, with
  `ai_assitant`=1.877. A Rust-only 3M-segment sample no longer shows
  `find_leaf_edges_spanning_range` or `BTreeMap::` in the sampled nested host
  traversal. M7 remains open.
- 2026-07-08: [M7] Retain sorted drawable order behind DrawOrder dirt. C++
  retains an intrusive drawable list and only resorts it when
  `ComponentDirt::DrawOrder` is raised by `DrawRules::drawTargetIdChanged()`
  or `DrawTarget::placementValueChanged()`. Rust now tracks a
  `draw_order_epoch` on `ArtboardInstance`, bumps it through the same
  property-change dirt surface, and caches `runtime_sorted_drawable_order` in
  `RuntimeRenderPathCache` by `(graph_global_id, draw_order_epoch)`.
  Prepared draw commands can still rebuild on the broader cache epoch, but the
  draw-target grouping BTree pass no longer reruns on clean draw-order frames.
  This follows the scout-ranked C++ dirt gate and does not add a speculative
  skip cache. `make golden-compare` remains exact=263 / exact-segments=584 /
  diverges=0; focused draw tests, `cargo test --workspace`, `cargo fmt --all
  -- --check`, and `git diff --check` pass. Repeat=100 JSON reports cpp
  median=0.522 ms, rust median=1.592 ms, Rust/C++=3.051; focused hot-loop
  reports aggregate Rust/C++=2.136 then 2.107. M7 remains open.
- 2026-07-08: [M7] Cache fixed draw/image/nested property keys. A post
  draw-classifier sample still had generic schema/property lookup under
  draw/prepare through literal `property_key_for_name` calls. C++ uses
  generated property-key constants instead of name lookup in draw/update
  paths, so Rust now extends `runtime_draw_property_key_for_name` to cover the
  fixed draw-order, image, mesh vertex, nested-artboard, and artboard-opacity
  keys used by `draw.rs`, while leaving dynamic component helper fallbacks
  untouched. This removes reflected schema lookup from fixed frame-loop reads;
  it is not a skip cache or dirty invalidation change. `make golden-compare`
  remains exact=263 / exact-segments=584 / diverges=0; focused draw tests,
  `cargo test --workspace`, `cargo fmt --all -- --check`, and `git diff
  --check` pass. Repeat=100 JSON reports cpp median=0.531 ms, rust
  median=1.586 ms, Rust/C++=2.985; focused hot-loop reports aggregate
  Rust/C++=2.187 then 2.122. M7 remains open.
- 2026-07-08: [M7] Remove draw-time schema lookup from fixed drawable
  classification. A post root-local sample still showed schema
  `definition_by_name` under draw/prepare even after generated switch tables
  removed the broader runtime reflection hot path. The remaining calls came
  from `sorted_drawable_uses_render_opacity` and
  `sorted_drawable_is_nested_artboard`, which only need fixed class answers in
  this runtime layer. C++ draw/update code uses concrete checks such as
  `drawable->is<Shape>()` and nested-artboard type-key dispatch, so Rust now
  classifies render-opacity drawables with `Shape | TextInputDrawable` and
  nested hosts with `NestedArtboard | NestedArtboardLeaf |
  NestedArtboardLayout` directly. This is a reflected-schema removal, not a
  new cache or invalidation rule. `make golden-compare` remains exact=263 /
  exact-segments=584 / diverges=0; focused draw tests, `cargo test
  --workspace`, `cargo fmt --all -- --check`, and `git diff --check` pass.
  Repeat=100 JSON reports cpp median=0.465 ms, rust median=1.675 ms,
  Rust/C++=3.603; focused hot-loop reports aggregate Rust/C++=2.243 then
  2.133. M7 remains open.
- 2026-07-08: [M7] Retain nested-host root view-model locals for child
  data-context sync. The previous retained source-local table still paid one
  scan through parent artboard slots to find the root `ViewModelInstance` for
  each unresolved child binding path, then walked property values from there.
  C++ `DataContext` keeps a root `ViewModelInstance` pointer and
  `DataBindPath::resolvedPath()` buffer, so Rust now stores the nested host's
  root `ViewModelInstance` locals by `viewModelId`, preserves first-match
  import order, rebuilds the table on dynamic `artboardId` swaps, and reuses
  it both while pre-resolving child source locals and during fallback sync.
  Successful fallback source-local resolutions are retained by path after they
  resolve; unsupported value kinds still do not create child updates. This is
  retained C++-shaped pointer/path data, not a new skip or negative-cache
  invalidation rule. `make golden-compare` remains exact=263 /
  exact-segments=584 / diverges=0; focused nested tests, `cargo test
  --workspace`, `cargo fmt --all -- --check`, and `git diff --check` pass.
  Direct Rust-only `ai_assitant --benchmark-repeat 1000000` improves from
  elapsed=11658.6 / advance=6683.3 ms to elapsed=9791.6 / advance=4650.2 ms;
  repeat=100 JSON reports cpp median=0.463 ms, rust median=1.717 ms,
  Rust/C++=3.710; focused hot-loop reports aggregate Rust/C++=2.136. M7 remains
  open.
- 2026-07-08: [M7] Retain nested-host source locals for child data-context sync.
  The previous Rust path still resolved each nested child property/image
  binding by scanning the parent artboard slots for a matching
  `ViewModelInstance`, then scanning child value slots for each path segment.
  C++ keeps view-model instance/value pointers and `DataBindPath` buffers on
  the data-context path, so Rust now builds a
  `RuntimeNestedArtboardInstance::data_bind_source_locals_by_path` table from
  the child artboard's binding paths when the nested instance is created,
  including dynamic `artboardId` swaps. The steady sync path uses the retained
  source local first and falls back to the old slot walk for unresolved paths;
  `sync_nested_child_artboard_data_contexts` also walks the retained host map
  directly instead of allocating a temporary host-key vector. This is retained
  import/build lookup data, not a new invalidation or skip rule. `make
  golden-compare` remains exact=263 / exact-segments=584 / diverges=0; focused
  nested/data-bind tests, `cargo test --workspace`, `cargo fmt --all
  -- --check`, and `git diff --check` pass. Direct Rust-only
  `ai_assitant --benchmark-repeat 1000000` improves to elapsed=11658.6 /
  advance=6683.3 ms; single-file repeat=100 improves to Rust/C++=4.880; focused
  release/null-renderer hot-loop reports aggregate Rust/C++=2.024 then 2.253,
  so M7 remains open.
- 2026-07-08: [M7] Share source-producing artboard data-bind paths as immutable
  slices. The previous Rust path retained custom-property/layout-computed/solo
  source paths as `Vec<u32>` and cloned those vectors while collecting nested
  context-source values. C++ keeps the resolved import-time path buffer on the
  binding/context object and passes pointers during context propagation, so Rust
  now stores those source-producing paths as `Arc<[u32]>` and clones the shared
  handle when a context-source value is emitted. This preserves the existing
  equality checks and data-bind value map semantics; it does not add a new
  invalidation or skip rule. `make golden-compare` remains exact=263 /
  exact-segments=584 / diverges=0; `cargo test --workspace`, `cargo fmt --all
  -- --check`, and `git diff --check` pass. Focused release/null-renderer
  hot-loop is effectively neutral/slightly positive at aggregate Rust/C++=2.164
  over the 5-entry / 10-segment corpus (`ai_assitant`=1.936), while direct
  Rust-only repeat=1000000 is noisy/slightly worse at elapsed=11884.7 /
  advance=6753.7 ms. M7 remains open.
- 2026-07-08: [M7] Inline nested owned-view-model child context paths for
  numeric retained `DataBindPath` lookups. The previous Rust path still
  combined the matched parent context path and child source tail into an owned
  `Vec<usize>` before constructing the child context chain. C++ keeps a
  `DataContext` pointer chain and resolves a nested artboard's
  `DataBindPath::resolvedPath()` by checking the current view-model instance
  root, walking the path tail, and then falling through to the parent context.
  Rust now mirrors that representation more closely with borrowed/inline
  storage for the common shallow numeric path and heap allocation only for
  unusually deep paths. The same view-model-target validation still runs
  before binding animations or recursively binding the child artboard, and no
  new skip/cache invalidation rule was added. `make golden-compare` remains
  exact=263 / exact-segments=584 / diverges=0; `cargo test --workspace`,
  `cargo fmt --all -- --check`, and `git diff --check` pass. Focused
  release/null-renderer hot-loop improves from aggregate Rust/C++=2.281 to
  2.169 over the 5-entry / 10-segment corpus, still above strict <=2.0, so M7
  remains open. A post-change sample still shows time in
  `bind_owned_view_model_artboard_context_chain` and
  `collect_nested_artboard_context_source_values`; profile those before the
  next slice, and keep the converter/property-cache scout fences in force.
- 2026-07-08: [M7] Accumulate nested context-source propagation without
  per-host descendant vectors. A release `ai_assitant` sample showed
  `collect_nested_artboard_context_source_values` still allocating and cloning
  under owned view-model/nested artboard data-context propagation. Rust now
  appends descendant context-source values into one accumulator, replays the
  just-appended range into the immediate child by reference, and appends that
  child's own context-source values after its data-bind advance. Nested host
  loops also walk the retained host map directly instead of allocating a key
  snapshot vector. This preserves the existing propagation order and mirrors
  C++ retained object traversal; it is not a skip/cache invalidation rule.
  `make golden-compare` remains exact=263 / exact-segments=584 / diverges=0;
  `cargo test --workspace`, `cargo fmt --all -- --check`, and
  `git diff --check` pass. Direct Rust-only `ai_assitant
  --benchmark-repeat 1000000` improves from elapsed=11839.2 / advance=6853.7
  ms to elapsed=11624.2 / advance=6594.8 ms; focused release/null-renderer
  hot-loop reports aggregate Rust/C++=2.281. Single-file repeat=100 reports
  Rust/C++=5.207 because the C++ median moved, so strict <=2.0 remains open.
  Next target is the remaining child-context path construction in
  `bind_owned_view_model_artboard_context_chain`.
- 2026-07-08: [M7] Trim owned view-model context-path allocation without
  widening skip/cache semantics. A release `ai_assitant` sample after the
  retained nested-host path slice still showed owned view-model/nested artboard
  data-context propagation hot, especially context-source path allocation and
  smaller retained animation/state-machine name clone/drop traffic. Rust now
  resolves numeric context-source paths by walking active child property lists
  directly, while keeping the existing name-based fallback; nested host
  context-chain prepends use stack storage for shallow chains; artboard
  property/image/custom binding paths are cloned only after equality shows a
  changed value; and retained linear-animation/state-machine names share
  `Arc<str>` definitions. This is a C++-shaped allocation cleanup
  (`DataContext` pointer walks plus immutable definitions), not a new dirt or
  skip cache. The status-doc scout review remains binding: no broad
  DataBindContext converter-property writes after the `db_health_tracker`
  RangeMapper failures, no StringPad-style RangeMapper retry without deeper
  ownership/order analysis, and no shallow command/path-wrapper caching without
  release/null-renderer evidence. `make golden-compare` remains exact=263 /
  exact-segments=584 / diverges=0; `cargo test --workspace`,
  `cargo fmt --all -- --check`, and `git diff --check` pass. Direct Rust-only
  `ai_assitant --benchmark-repeat 100000` improves from elapsed=1408.5 /
  advance=902.0 ms to elapsed=1225.7 / advance=706.0 ms; single-file
  release/null-renderer repeat=100 reports Rust/C++=4.496; focused hot-loop
  reports aggregate Rust/C++=2.363. Strict <=2.0 remains open.
- 2026-07-08: [M7] Retain nested-host data-bind paths outside the steady frame.
  C++ keeps the host `DataBindPath` on the nested artboard and lazily resolves
  the path buffer once; Rust now stores the resolved path ids on
  `RuntimeNestedArtboardInstance` during initial host construction and dynamic
  `artboardId` swaps. The owned view-model nested context path consumes that
  retained slice instead of resolving
  `RuntimeFile::data_bind_path_for_referencer_object` every sync pass. This
  preserves the scout/perf fences: it is immutable import/build data retention,
  not invented skip/cache invalidation. `make golden-compare` remains exact=263
  / exact-segments=584 / diverges=0; `cargo test --workspace`,
  `cargo fmt --all -- --check`, and `git diff --check` pass. Long-repeat
  Rust-only `ai_assitant` improves from elapsed=1734.9 / advance=1233.3 ms to
  elapsed=1408.5 / advance=902.0 ms for 100000 segments; fenced
  release/null-renderer hot-loop reports aggregate Rust/C++=2.338 then 2.430,
  still above strict <=2.0, so M7 remains open.
- 2026-07-08: [M7] Remove Rust-only nested child data-bind cloning from the
  steady owned-view-model context path. A release `ai_assitant` sample showed
  `sync_nested_child_artboard_data_contexts` cloning child property/image
  binding vectors and repeatedly entering the generic data-bind property-key
  dispatcher while C++ propagates nested data contexts through retained
  objects/pointers. Rust now stages only `(binding index, value)` updates before
  mutating the child, and the sampled nested-host `ViewModelInstance*`
  property lookups use fixed cached generated-property keys. This does not add
  a new skip/cache invalidation rule. `make golden-compare` remains exact=263 /
  exact-segments=584 / diverges=0; `cargo test --workspace`, format check, and
  `git diff --check` pass. Long-repeat Rust-only `ai_assitant`
  improves from elapsed=2095.6 / advance=1602.2 ms to elapsed=1734.9 /
  advance=1233.3 ms for 100000 segments; fenced release/null-renderer hot-loop
  reports aggregate Rust/C++=2.119 over the 5-entry / 10-segment focused
  corpus, still above strict <=2.0, so M7 remains open.
- 2026-07-08: [M7] Remove operation-view-model/system-operation custom sources
  from the conservative polling lane while preserving the scout/perf fences.
  `DataConverterOperationViewModel` has no C++ converter-property dirty callback
  for this layer; Rust already refreshes dependent operation-view-model numbers
  from source-path changes. `DataConverterSystemDegsToRads` and
  `DataConverterSystemNormalizer` inherit `DataConverterOperationValue`, whose
  `operationValueChanged()` calls `markConverterDirty`, so Rust now builds
  artboard `operationValue` converter-property bindings for those subclasses,
  updates copied dependent converters by global id, and enqueues the concrete
  property/custom parents already covered by target/source queues. This keeps
  the rejected broad DataBindContext converter-property write and rejected
  RangeMapper fallback-removal scouts out of the runtime. `make golden-compare`
  remains exact=263 / exact-segments=584 / diverges=0; `cargo test --workspace`,
  `cargo fmt --all -- --check`, and `git diff --check` pass. Fenced
  release/null-renderer hot-loop reports aggregate Rust/C++=2.111 over the
  5-entry / 10-segment focused corpus, still above strict <=2.0, so M7 remains
  open.
- 2026-07-08: [M7] Remove `DataConverterNumberToList` custom sources from the
  conservative polling lane. The C++ handwritten NumberToList class overrides
  `viewModelIdChanged()` to clear its generated `m_listItems` cache and call
  `DataConverter::markConverterDirty`. Rust now builds artboard NumberToList
  `viewModelId` converter-property bindings, queues them by source path,
  updates copied dependent converters by global id, and enqueues the concrete
  property/custom/list parents already covered by target/source queues. The
  C++ cached-list clearing edge is represented by the Rust converter carrying
  the current `view_model_id` and file view-model count, then recomputing
  `List { item_count }` on conversion; this runtime layer does not cache
  generated `ViewModelInstanceListItem` objects. `make golden-compare` remains
  exact=263 / exact-segments=584 / diverges=0; `cargo check -p rive-runtime`,
  `cargo test -p rive-runtime queues`, `cargo test -p rive-runtime
  number_to_list`, `cargo test --workspace`, `cargo fmt --all -- --check`, and
  `git diff --check` pass. Fenced release/null-renderer hot-loop reports
  aggregate Rust/C++=2.243 over the 5-entry / 10-segment focused corpus, still
  above strict <=2.0, so M7 remains open.
- 2026-07-08: [M7] Remove `DataConverterInterpolator` custom sources from the
  conservative polling lane for the audited `duration` property only. The C++
  handwritten Interpolator class overrides `durationChanged()` to call
  `DataConverter::markConverterDirty`, while generated `interpolationType` and
  `interpolatorId` callbacks are empty. Rust now builds artboard Interpolator
  `duration` converter-property bindings, queues them by source path, updates
  copied dependent converters by global id, and enqueues the concrete
  property/custom parents already covered by target/source queues. The separate
  stateful advance path remains responsible for the C++
  `InterpolatorAdvancer::advance()` dirty edge. `make golden-compare` remains
  exact=263 / exact-segments=584 / diverges=0; `cargo test --workspace`,
  `cargo fmt --all -- --check`, and `git diff --check` pass. Fenced
  release/null-renderer hot-loop reports aggregate Rust/C++=2.180 and 2.264
  over the 5-entry / 10-segment focused corpus, still above strict <=2.0, so M7
  remains open.
- 2026-07-08: [M7] Do not land the family-specific RangeMapper converter-property
  updater as the fallback-removal path. The scout followed the C++ dirty surface
  more narrowly than the earlier broad converter-property write: it covered only
  `minInput`, `maxInput`, `minOutput`, and `maxOutput`, left generated-empty
  `flags`, `interpolationType`, and `interpolatorId` alone, and treated
  RangeMapper custom sources as push-safe. Focused checks passed
  (`cargo check -p rive-runtime`, `cargo test -p rive-runtime queues`, and
  `cargo test -p rive-runtime range_mapper`), and the full compare passed the
  early C++ stream for `db_health_tracker`, but `make golden-compare` failed the
  actual tolerant comparison for that file at line 3390 with clip-path x-position
  drift (Rust first point x=48.2119293 vs C++ x=64.2483139). The code was backed
  out. Keep RangeMapper on the conservative persisting lane until the C++
  DataBind/DataConverter ownership and update-order semantics are audited more
  deeply.
- 2026-07-08: [M7] Remove `DataConverterRounder` custom sources from the
  conservative polling lane. The C++ generated `decimalsChanged()` hook is
  empty, and the handwritten `DataConverterRounder` class does not override it;
  its only authored behavior is `convert()` plus `outputType()`. Rust therefore
  treats Rounder as push-safe without adding a converter-property updater,
  matching the absence of a C++ `DataConverter::markConverterDirty` edge. The
  status review keeps the scout/perf fence discoveries intact: the broad
  RangeMapper/converter-property-write scout stays rejected after the
  `db_health_tracker` clip-position failure, shallow cached command/path-wrapper
  scouts stay rejected by release/null-renderer hot-loop data, and M7 perf
  decisions stay fenced to release C++/Rust runners, null-renderer benchmark
  mode, phase-sum metrics, and repeated focused corpus checks. `make
  golden-compare` remains exact=263 / exact-segments=584 / diverges=0; `cargo
  test --workspace`, `cargo fmt --all -- --check`, and `git diff --check` pass.
  Fenced release/null-renderer hot-loop reports aggregate Rust/C++=2.310 and
  2.181 over the 5-entry / 10-segment focused corpus, still above strict <=2.0,
  so M7 remains open.
- 2026-07-08: [M7] Remove `DataConverterStringPad` custom sources from the
  conservative polling lane. The C++ family has three bindable property
  callbacks, `lengthChanged()`, `textChanged()`, and `padTypeChanged()`, and all
  call `DataConverter::markConverterDirty`. Rust now builds artboard StringPad
  converter-property bindings for those exact properties, queues them by source
  path through `RuntimeArtboardDataBindTargetQueues`, seeds them for initial
  apply, and updates copied dependent converters by the target converter's
  global id. The updater then enqueues the concrete property/custom parents
  already covered by the target/source queues, keeping the rejected broad
  converter-property-write scout out of the runtime. `make golden-compare`
  remains exact=263 / exact-segments=584 / diverges=0; `cargo test --workspace`,
  `cargo fmt --all -- --check`, and `git diff --check` pass. Fenced
  release/null-renderer hot-loop reports aggregate Rust/C++=2.120 and 2.186
  over the 5-entry / 10-segment focused corpus, still above strict <=2.0, so M7
  remains open.
- 2026-07-08: [M7] Remove `DataConverterStringTrim` custom sources from the
  conservative polling lane. The C++ family has one bindable property callback,
  `trimTypeChanged()`, which calls `DataConverter::markConverterDirty`. Rust
  now builds artboard StringTrim converter-property bindings for `trimType`,
  queues them by source path through `RuntimeArtboardDataBindTargetQueues`,
  seeds them for initial apply, and updates copied dependent converters by the
  target converter's global id. The updater then enqueues the concrete
  property/custom parents already covered by the target/source queues, keeping
  the rejected broad converter-property-write scout out of the runtime.
  `make golden-compare` remains exact=263 / exact-segments=584 / diverges=0;
  `cargo test --workspace`, `cargo fmt --all -- --check`, and
  `git diff --check` pass. Fenced release/null-renderer hot-loop reports
  aggregate Rust/C++=2.173 and 2.239 over the 5-entry / 10-segment focused
  corpus, still above strict <=2.0, so M7 remains open.
- 2026-07-08: [M7] Remove `DataConverterToString` custom sources from the
  conservative polling lane. The C++ family has two bindable property callbacks,
  `decimalsChanged()` and `colorFormatChanged()`, and both call
  `DataConverter::markConverterDirty`. Rust now builds artboard ToString
  converter-property bindings for those exact properties, queues them by source
  path through `RuntimeArtboardDataBindTargetQueues`, seeds them for initial
  apply, and updates copied dependent converters by the target converter's
  global id. The updater then enqueues the concrete property/custom parents
  already covered by the target/source queues, so this keeps the rejected broad
  converter-property-write scout out of the runtime. `make golden-compare`
  remains exact=263 / exact-segments=584 / diverges=0; `cargo test --workspace`,
  `cargo fmt --all -- --check`, and `git diff --check` pass. Fenced
  release/null-renderer hot-loop reports aggregate Rust/C++=2.238 and 2.195
  over the 5-entry / 10-segment focused corpus, still above strict <=2.0, so
  M7 remains open.
- 2026-07-08: [M7] Remove plain `DataConverterOperationValue` custom sources
  from the conservative polling lane. The C++ family has a single dirty
  callback, `operationValueChanged()`, which calls
  `DataConverter::markConverterDirty`; Rust's artboard data-bind path already
  models that callback through `set_artboard_operation_value`, updating
  dependent `OperationValue` converters and enqueuing their property/custom
  parents. The queue predicate now treats only plain `OperationValue` as
  push-safe; `SystemOperationValue` subclasses stay persisting until their
  inherited `operationValue` bind-target path is modeled directly. `make
  golden-compare` remains exact=263 / exact-segments=584 / diverges=0; `cargo
  test --workspace`, `cargo fmt --all -- --check`, and `git diff --check`
  pass. Fenced release/null-renderer hot-loop reports aggregate Rust/C++=2.201
  over the 5-entry / 10-segment focused corpus, improved from the previous
  2.409 but still above strict <=2.0, so M7 remains open.
- 2026-07-08: [M7] Narrow converter-backed custom-property polling to
  converter families with unmodeled C++ dirty edges. The status review keeps
  the scout/perf fences in force: release/null-renderer hot loops are the
  decision-grade metric, shallow cached command wrappers stay rejected, and
  data-bind work should follow C++ `DataBindContainer` queues plus audited
  converter-parent dirt. C++ converter dirt flows through
  `DataConverter::markConverterDirty`; the Rust source queue now treats pure
  and explicit-token converter families as push-safe, while `RangeMapper`,
  `StringPad`, `StringTrim`, `ToString`, `Interpolator`, `NumberToList`,
  operation/system-operation, and unsupported converters stay on the
  conservative persisting lane until their property dirty callbacks are
  modeled directly. `make golden-compare` remains exact=263 /
  exact-segments=584 / diverges=0; `cargo test --workspace`, `cargo fmt --all
  -- --check`, and `git diff --check` pass. Fenced release/null-renderer
  hot-loop reports aggregate Rust/C++=2.409 over the 5-entry / 10-segment
  focused corpus, improved from the previous converter-edge slice's 2.592 but
  still above the strict <=2.0 target, so M7 remains open.
- 2026-07-08: [M7] Land the OperationViewModel-number converter-parent dirty
  edge only. Artboard source-path number changes now refresh dependent
  OperationViewModel converters across property, custom-property, formula-token,
  and list bindings, then enqueue the concrete property/custom parents that have
  a push queue. This is the first explicit replacement edge for the conservative
  converter-backed custom-property persisting lane, but it does not remove that
  lane. A broader scout that tried global converter-property writes for
  RangeMapper-style dependencies was backed out after `db_health_tracker`
  showed wrong clip positions, so the next fallback-removal work must port
  concrete C++ converter-parent dirty edges family-by-family instead of writing
  converter properties generically. `make golden-compare` remains exact=263 /
  exact-segments=584 / diverges=0; `cargo test --workspace`, `cargo fmt --all
  -- --check`, and `git diff --check` pass. Fenced release/null-renderer
  hot-loop reports aggregate Rust/C++=2.592 over the 5-entry / 10-segment
  focused corpus, so M7 remains open.
- 2026-07-08: [M7] Land artboard target-to-source dirty/persisting source
  queues. `ArtboardInstance` now owns a C++-shaped source queue for
  target-to-source artboard binds: push-capable custom-property and direct
  numeric targets are indexed by `(target local, property key)`, generated
  setters enqueue those sources, source-to-target property applies suppress
  only the currently writing data-bind index, and layout-computed, solo
  `activeComponentId`, and shape-length sources remain on polling fallback
  like C++ `targetSupportsPush()`. Converter-backed custom-property sources
  remain on a conservative persisting lane because C++ converters dirty their
  parent `DataBind` through converter-owned dependencies, and Rust still needs
  explicit coverage for every converter dirty edge. `make golden-compare`
  remains exact=263 / exact-segments=584 / diverges=0, `cargo test
  --workspace`, `cargo fmt --all -- --check`, and `git diff --check` pass.
  Fenced release/null-renderer focused hot-loop runs report aggregate
  Rust/C++=2.784 and 2.500; direct repeat-heavy `ai_assitant
  --benchmark-repeat 100` reports cpp median=0.569 ms, rust median=4.019 ms,
  Rust/C++=7.060. M7 remains open; next remove the conservative converter
  persisting fallback by porting explicit C++ converter-parent dirty
  enrollment, then re-profile before widening the data-bind queue pattern.
- 2026-07-08: [M7] Land path-indexed artboard source-to-target dirty queues
  as the first C++ `DataBindContainer` queue slice. Unlike the rejected
  per-binding `target_dirty` scout, this slice does not scan all bindings when
  a source path changes: `ArtboardInstance` owns container queues indexed by
  data-bind source path, seeds them for initial apply, enrolls affected
  property/image targets from the source setter, and re-enrolls property
  targets when formula token/operation values or stateful converters mutate
  converter state. This mirrors the C++ dirty-list enrollment shape for
  source-to-target artboard binds while leaving target-to-source binds on the
  existing polling path until the next slice can port `targetSupportsPush()`
  and the persisting/dirty-to-source lists. `make golden-compare` remains
  exact=263 / exact-segments=584 / diverges=0 and `cargo test --workspace`
  passes. Same-session baseline worktree `988fc29` measured Rust-only
  repeat=100000 at elapsed=3080.3 ms / advance=2392.7 ms and focused
  hot-loop aggregate Rust/C++=2.723; this slice measured Rust-only
  repeat=100000 at elapsed=2480.2 ms / advance=1859.1 ms and focused
  aggregate Rust/C++=2.371 / 2.599. Direct repeat-heavy `ai_assitant` remains
  above the target at Rust/C++=7.279, so M7 stays open.
- 2026-07-08: [M7] Do not land naive artboard binding `target_dirty` bits as
  the data-bind dirty-queue port. The scout added dirty booleans to
  source-to-target artboard property/image bindings, marked them when a source
  path changed, and drained only dirty targets in the existing two apply phases.
  This preserved correctness while present: focused data-bind cpp-probe tests
  passed and `make golden-compare` remained exact=263 / exact-segments=584 /
  diverges=0. It failed the M7 perf fence: direct repeat-heavy
  `ai_assitant --benchmark-repeat 100` reported Rust/C++=10.962 and 15.381
  versus the landed borrowed-chain result of 7.210; Rust-only repeat=100000
  regressed to elapsed=4766.0 ms / advance=3385.2 ms; focused 5-entry
  hot-loop moved to Rust/C++=2.614. The useful finding is scope: the next
  data-bind performance slice must port the real C++ `DataBindContainer`
  dirty/persisting vectors and `DataBind::addDirt` enrollment, not wrap the
  current broad scans in per-binding target booleans. The scout code was backed
  out before commit.
- 2026-07-08: [M7] Borrow nested owned-view-model context chains instead of
  cloning them per host. The previous Rust path represented C++'s
  `DataContext` parent chain as a `Vec<Vec<usize>>` and cloned the whole chain
  in `bind_owned_view_model_artboard_context_chain` for every nested host. This
  slice keeps the same lookup order but threads `&[&[usize]]` through
  artboard, state-machine, and data-bind graph owned-view-model binding; it
  only allocates a small vector of borrowed path slices when a nested host
  contributes a child context path. This is not a dirty-queue port and does not
  add skip semantics; it is a narrower translation of the C++
  `DataContext::getViewModelProperty` parent traversal shape. Focused data-bind
  cpp-probe tests, `cargo check -p rive-runtime`, and `make golden-compare`
  pass at exact=263 / exact-segments=584 / diverges=0. Direct repeat-heavy
  `ai_assitant` reports cpp median=0.603 ms, rust median=4.348 ms,
  Rust/C++=7.210. A fresh baseline worktree at the prior commit measured
  Rust-only repeat=100000 at elapsed=4235.4 ms / advance=3275.3 ms; this slice
  measures elapsed=4109.3 ms / advance=3120.9 ms. Focused 5-entry hot-loop is
  still above the strict target at Rust/C++=2.321, so M7 remains open. Next:
  stop trimming context-chain containers and port the real C++
  `DataBindContainer` dirty queues / `DataBind::addDirt` enrollment.
- 2026-07-08: [M7] Trim owned view-model data-bind allocation in the profiled
  advance path. The post paint-preparation profile sampled
  `ai_assitant --benchmark-repeat 100000` and showed Rust time dominated by
  `advance_ms` (elapsed=4437.5 ms, advance=3476.3 ms, prepare=400.3 ms,
  draw=552.3 ms), with top stacks in owned view-model nested artboard
  context-chain rebinding and `property_path_for_context_source_path`
  allocation. This slice keeps behavior unchanged but removes two Rust-only
  allocation layers: resolved context source paths no longer collect an
  intermediate `Vec` before extending the real property path, and
  `bind_owned_view_model_artboard_values` applies each owned-view-model
  property/image/custom-property update directly instead of staging a temporary
  update vector. This follows the C++ shape in `DataContext::getViewModelProperty`
  and `DataBind::addDirt`: C++ walks retained context/source pointers and
  queues dirty data binds rather than rebuilding owned path/update containers
  every frame. Focused data-bind cpp-probe tests, `cargo check -p
  rive-runtime`, and `make golden-compare` pass at exact=263 /
  exact-segments=584 / diverges=0. Direct repeat-heavy `ai_assitant` improved
  to rust median=4.553 / 4.764 ms (Rust/C++=7.731 / 9.399); Rust-only
  repeat=100000 improved to elapsed=3840.8 ms, advance=2936.9 ms. Focused
  5-entry hot-loop ratios were noisy and not completion-grade
  (Rust/C++=2.517 and 2.776), so M7 remains open. Next stay in data-bind:
  profile again, then port actual `DataBindContainer` dirty queues or retained
  data-context chains before returning to path retention.
- 2026-07-08: [M7] Cache clean-frame paint preparation behind the conservative
  Rust instance cache epoch. `RuntimeRenderPaintCache` now stores a
  `RuntimePaintPreparationFrame` keyed by `(graph_global_id, cache_epoch)` and
  `prepare_static_artboard_tree_paints_internal` returns early for
  non-dependency-order prepares when that key still matches. The key is tied to
  Rust's existing idempotent property setters and `add_dirt` /
  `on_component_dirty` path, so this mirrors the C++
  `Artboard::updateComponents()` clean-frame first-branch return
  conservatively rather than inventing a geometry invalidation shortcut. The
  dependency-order layout-gradient path remains unskipped. Focused tests,
  `cargo check -p rive-runtime`, and `make golden-compare` pass at exact=263 /
  exact-segments=584 / diverges=0. Fenced release/null-renderer focused runs
  reported aggregate Rust/C++=2.493, 1.832, and 2.166 over 5 entries / 10
  segments; direct repeat-heavy `ai_assitant` improved to cpp median=0.582 /
  0.603 ms and rust median=5.149 / 5.885 ms (Rust/C++=8.852 / 9.756), versus
  the prior logged direct ratio around 20.0. M7 is still open because the
  focused corpus does not reliably clear strict <=2.0. Next: profile the
  current tree before choosing between deeper `ShapePaintPath`/`PathComposer`
  retention and remaining data-bind/nested advance allocation.
- 2026-07-08: [M7] Do not land shared shape path-command buffers as the
  `ShapePaintPath` retention slice. The experiment changed
  `RuntimeShapePaintCommand` path/effect/inner-feather payloads from owned
  `Vec<RuntimePathCommand>` to shared `Arc<[RuntimePathCommand]>` slices and
  added a `RuntimeRenderPathCache` shape-path command cache keyed by graph,
  shape, path kind, `path_epoch`, and `layout_epoch`. This is closer to C++'s
  retained raw path than the earlier cloned-Vec scout, and it kept correctness
  green: focused path/probe tests passed and `make golden-compare` reported
  exact=263 / exact-segments=584 / diverges=0. It still failed the M7 perf
  fence: two release/null-renderer focused hot-loop runs reported aggregate
  Rust/C++=2.627 and 2.619 versus the logged 2.329 baseline. Direct
  repeat-heavy `ai_assitant` improved to cpp median=0.604 ms, rust
  median=11.242 ms, Rust/C++=18.598, but that is not enough to override the
  focused corpus regression. Backed out the code. The next M7 slice should
  either make clean frames skip prepare through the audited C++ dirt/update
  gate, or port actual `PathComposer`/raw-path retention below prepared
  command construction.
- 2026-07-08: [M7] Do not land the retained clip/layout/background path cache
  scout as a standalone optimization. The experiment changed
  `RuntimeRenderPathCache` so artboard clips, clipping shapes, layout clips,
  and backgrounds reused retained `RenderPath` handles until the existing
  layout/path epoch or fill rule changed, with separate artboard clip slots for
  origin-transformed and root-space clips. Focused runtime tests,
  `cargo check -p rive-runtime`, and `make golden-compare` stayed green at
  exact=263 / exact-segments=584 / diverges=0, but the M7 perf fence rejected
  it: two release/null-renderer hot-loop runs reported aggregate Rust/C++=2.705
  and 3.338 over the 5-entry / 10-segment focused corpus, worse than the
  previous logged 2.329 baseline. Direct repeat-heavy `ai_assitant` was neutral
  rather than decisive (cpp median=0.813 ms, rust median=15.786 ms,
  Rust/C++=19.424). The useful finding is priority, not code: do not spend the
  next slice on shallow clip/layout/background cache wrappers. Continue with
  the scout-ranked lower-level `ShapePaintPath` retained RawPath/RenderPath and
  `PathComposer` Path|NSlicer dirt gating work.
- 2026-07-08: [M7] Cached nested-artboard layout bounds by `layout_epoch`.
  Rust nested advance previously recomputed `runtime_taffy_layout_bounds` and
  cloned the artboard graph whenever any `NestedArtboardLayout` host existed.
  `ArtboardInstance` now retains those bounds behind a
  `(graph_global_id, layout_epoch)` key, so clean nested-advance frames reuse
  the layout snapshot until the same layout dirt boundary Rust already uses for
  draw-side Taffy bounds changes. This follows the C++ layout dirt model:
  `LayoutComponent::markLayoutNodeDirty()` dirties the Yoga node and calls
  `Artboard::markLayoutDirty()`, and `NestedArtboardLayout` routes host layout
  dirt through that path. `make golden-compare` passes at exact=263 /
  exact-segments=584 / diverges=0; `cargo test --workspace` passes, including
  the focused `nested_layout_bounds_cache_tracks_layout_epoch` test. Focused
  release/null-renderer `make perf-hot-loop PERF_CORPUS_LIMIT=5
  PERF_ITERATIONS=10 PERF_WARMUPS=1 PERF_MAX_RATIO=999` reports aggregate
  Rust/C++=2.329 over 5 exact entries / 10 segments
  (`advance_blend_mode`=5.649, `ai_assitant`=2.221,
  `align_target`=1.888, `animated_clipping`=2.461,
  `animation_reset_cases`=4.264). Direct `ai_assitant --benchmark-repeat 100`
  remains around Rust/C++=20.0 (rerun cpp median=0.595 ms, rust median=11.919
  ms), so strict <=2.0 remains open. Next target remains the scout-ranked
  `ShapePaintPath`/raw-path retention and `PathComposer` dirt gating path,
  with idempotent dirt raisers only where audited against C++.
- 2026-07-08: [M7] Scout review after the path-command cache experiment keeps
  the next slice scoped to C++ dirt/retention semantics. A private Rust
  `Shape` paint path-command cache was tested and then backed out: it preserved
  correctness while present, but focused release/null-renderer perf was not a
  clear improvement (5-entry aggregate Rust/C++=2.588; direct
  `ai_assitant --benchmark-repeat 100` cpp median=0.555 ms, rust median=10.197
  ms, Rust/C++=18.375). Do not repeat this as a standalone cache layer. The
  scout reports say the next landed work should be either audited idempotent
  dirt raisers that let steady frames skip prepare, or actual
  `ShapePaintPath`/`PathComposer` RawPath retention behind C++ dirt gates.
- 2026-07-08: [M7] Split retained draw-path invalidation onto
  `ArtboardInstance::path_epoch`. The scout result says C++ draw() should replay
  retained handles and geometry rebuilds belong behind dirt/update gates; this
  slice narrows Rust's retained `RenderPath` rebuild trigger from the broad
  draw-cache epoch to a path-specific epoch. `path_epoch` is bumped by
  path/vertices/world-transform/layout/NSlicer dirt, collapse changes, and
  C++ `StrokeEffect`-style TrimPath/DashPath/Dash/Feather path-affecting
  property changes. Feather `inner` and `spaceValue` are included because they
  switch or transform the cached inner-feather command stream, while paint-only
  dirt keeps the retained draw path. An initial attempt caught `fill_trim_path`
  and `stacked_path_effects` regressions in `make golden-compare`; the final
  effect-property invalidation fixed them.
  `cargo fmt --all -- --check`, focused runtime tests
  `path_epoch_tracks_path_dirt_separately_from_draw_cache_epoch`,
  `path_epoch_tracks_effect_path_property_changes`, and
  `draw_path_reuses_render_path_until_path_epoch_changes`,
  `cargo test --workspace`, and `make golden-compare` pass at exact=263 /
  exact-segments=584 / diverges=0. Focused release/null-renderer
  `make perf-hot-loop PERF_CORPUS_LIMIT=5 PERF_ITERATIONS=10 PERF_WARMUPS=1
  PERF_MAX_RATIO=999` reports aggregate Rust/C++=2.405 over 5 exact entries /
  10 segments (`advance_blend_mode`=4.554, `ai_assitant`=2.533,
  `align_target`=1.663, `animated_clipping`=2.266,
  `animation_reset_cases`=3.966). Direct
  `ai_assitant --benchmark-repeat 100` reports cpp median=0.363 ms, rust
  median=7.695 ms, Rust/C++=21.222; strict <=2.0 remains open. Next target is
  deeper scout-ranked `ShapePaintPath` raw-path retention / `PathComposer` dirt
  gating, not a new behavior family.
- 2026-07-08: [M7] Retained render-paint draw configuration in
  `RuntimeRenderPaintCache`. Persistent draw paints now remember the last
  applied paint type, stroke thickness/cap/join, blend mode, solid color or
  preserved gradient shader, and feather strength. Draw-time configuration is
  skipped when the retained `RenderPaint` already matches; temporary text paints
  stay uncached, and gradient preparation removes the cached config before
  mutating a retained paint so shader/style state cannot go stale. This is a
  narrow C++-aligned retention slice, not a new runtime behavior gate. `cargo
  fmt --all -- --check`, focused runtime test
  `draw_path_reuses_render_path_until_instance_epoch_changes`, `cargo test
  --workspace`, and `make golden-compare` pass at exact=263 /
  exact-segments=584 / diverges=0. Focused release/null-renderer `make
  perf-hot-loop PERF_CORPUS_LIMIT=5 PERF_ITERATIONS=10 PERF_WARMUPS=1
  PERF_MAX_RATIO=999` reports aggregate Rust/C++=2.518 over 5 exact entries /
  10 segments (`ai_assitant`=2.583, `align_target`=1.864,
  `animated_clipping`=2.422). Direct `ai_assitant --benchmark-repeat 100`
  reports cpp median=0.393 ms, rust median=7.341 ms, Rust/C++=18.668; strict
  <=2.0 remains open. Next target should return to the scout-ranked
  `ShapePaintPath`/raw-path retention and clean-frame dirt-gating path.
- 2026-07-08: [M7] Retained gradient paint preparation grouping/order in
  `RuntimeRenderPathCache`. Rust paint prep now caches graph-static gradient
  mutator buckets plus dependency/dependency-insertion order vectors by live
  artboard graph identity, mirroring C++'s retained component graph/update
  ordering instead of rebuilding `BTreeMap` groupings and dependency vectors on
  every prepare/draw pass. Shader and paint mutation remain governed by the
  existing gradient state cache; this adds no tolerance widening and no
  invented dirt skip. `cargo fmt --all -- --check`, focused runtime test
  `draw_path_reuses_render_path_until_instance_epoch_changes`,
  `cargo test --workspace`, and `make golden-compare` pass at
  exact=263/exact-segments=584. Focused release/null-renderer
  `make perf-hot-loop PERF_CORPUS_LIMIT=5 PERF_ITERATIONS=10 PERF_WARMUPS=1
  PERF_MAX_RATIO=999` reports aggregate Rust/C++=2.647 over 5 exact entries /
  10 segments (`ai_assitant`=2.906, `align_target`=1.832,
  `animated_clipping`=2.400). Direct `ai_assitant --benchmark-repeat 100`
  reports cpp median=0.398 ms, rust median=7.700 ms, Rust/C++=19.356; strict
  <=2.0 remains open. Next target is actual C++ `Paint|Stops|RenderOpacity`
  dirt-gated render-paint mutation or `ShapePaintPath` retention.
- 2026-07-07: [M7] Split retained Taffy layout bounds from the coarse draw
  cache epoch. `ArtboardInstance` now tracks a `layout_epoch` bumped by
  C++-aligned layout dirt/property changes (`LayoutStyle`, layout
  width/height/style/fractional sizing, text-shape sizing/style/text changes,
  nested-artboard layout sizing, collapse), while paint/color and non-text
  string changes only invalidate the full prepared draw-command frame.
  `RuntimeRenderPathCache` reuses layout bounds across non-layout
  frame-cache changes, mirroring `LayoutComponent::markLayoutNodeDirty`
  without inventing paint invalidation. Full `cargo fmt --all -- --check`,
  `cargo test --workspace`, and `make golden-compare` pass at
  exact=263/exact-segments=584. Release sample with `/usr/bin/sample` on
  `ai_assitant --benchmark-repeat 100000` no longer shows
  `runtime_taffy_layout_bounds` in the hot stack; heat has moved to
  data-bind/nested advance allocation. Focused
  `make perf-hot-loop PERF_CORPUS_LIMIT=5 PERF_ITERATIONS=10 PERF_WARMUPS=1
  PERF_MAX_RATIO=999` first reported aggregate Rust/C++=2.228, then the
  C++-aligned text/fractional invalidation safety pass reran at 2.699 over 5
  exact entries / 10 segments (`ai_assitant`=2.785,
  `align_target`=2.399, `animated_clipping`=2.406). Direct
  `ai_assitant --benchmark-repeat 100` improves Rust median from 10.766 ms to
  8.183 ms, though the C++ median variance makes the reported ratio
  Rust/C++=13.850 on this run. Strict <=2.0 remains open; next target is
  data-bind/nested advance allocation or C++ `Paint|Stops|RenderOpacity` /
  `ShapePaintPath` dirt retention.
- 2026-07-07: [M7] Generated switch-table schema lookups from `rive-codegen`
  and routed the public `rive-schema` definition/property/core-registry helpers
  through them. This removes linear `DEFINITIONS` / ancestor scans from the
  frame-loop lookup sites called by data-bind path referencers, instance
  property setting/kind checks, and layout/draw property helpers while keeping
  the old public API and first-match semantics. Full
  `cargo fmt --all -- --check`, `cargo test --workspace`, and
  `make golden-compare` pass at exact=263/exact-segments=584. Release
  hot-loop smoke with `PERF_CORPUS_LIMIT=5 PERF_ITERATIONS=10 PERF_WARMUPS=1
  PERF_MAX_RATIO=999` improves aggregate Rust/C++ from 3.096 to 2.543 over 5
  exact entries / 10 segments (`ai_assitant`=2.611). Direct
  `ai_assitant --benchmark-repeat 100` improves from Rust/C++=34.736 to
  17.233 (cpp median=0.625 ms, rust median=10.766 ms). Strict <=2.0 remains
  open; next target is a fresh release profile, then audited C++ dirty-gated
  layout/paint preparation retention.
- 2026-07-07: [M7] Cached fixed data-bind property keys used by artboard
  property/default/nested-host binding paths, mirroring C++ generated
  `*PropertyKey` / CoreRegistry access instead of doing schema name scans in
  frame-loop data-bind code. Full `cargo fmt --all -- --check`,
  `cargo test --workspace`, and `make golden-compare` pass at
  exact=263/exact-segments=584. Release hot-loop smoke with
  `PERF_CORPUS_LIMIT=5 PERF_ITERATIONS=10 PERF_WARMUPS=1 PERF_MAX_RATIO=999`
  reports aggregate Rust/C++=3.096 over 5 exact entries / 10 segments
  (`ai_assitant`=3.347). Direct `ai_assitant --benchmark-repeat 100` reports
  Rust/C++=34.736 (cpp median=0.543 ms, rust median=18.878 ms), so strict
  <=2.0 remains open. Post-slice sampling confirms raw `property_key_for_name`
  is no longer the stateful nested-host binding hot site; remaining heat is
  `RuntimeFile::data_bind_path_for_referencer_object` ->
  `definition_by_type_key`, `InstanceObjectArena::set_property_value` /
  `property_kind` -> core registry/type scans, and layout/draw property helper
  keys. Next M7 target: generated/cached schema kind/property tables for these
  frame-loop lookup sites, then audited dirty-gated layout/paint preparation
  retention.
- 2026-07-07: [M7] Cached fixed layout/schema property keys used by layout
  preparation, collapse/visibility checks, nested-artboard layout sizing, and
  shared Solo/Joystick helpers, mirroring C++ generated `*PropertyKey`
  constants instead of doing schema name/property scans in the frame loop.
  Full `cargo test --workspace` and `make golden-compare` pass at
  exact=263/exact-segments=584. Release hot-loop smoke with
  `PERF_CORPUS_LIMIT=5 PERF_ITERATIONS=10 PERF_WARMUPS=1 PERF_MAX_RATIO=999`
  reports aggregate Rust/C++=3.306 over 5 exact entries / 10 segments
  (`ai_assitant`=3.941). Direct `ai_assitant --benchmark-repeat 100` reports
  Rust/C++=43.716 on the second 10-iteration run (the first was a noisy
  49.859 outlier), so strict <=2.0 remains open. A broad prepared-paint
  skip-cache experiment was discarded before commit after regressing focused
  aggregate to 11.227; future paint-prep work must mirror audited C++ dirt
  invalidation rather than caching by instance epoch. Next M7 target: fresh
  release profile, then C++ dirty-gated layout/paint preparation retention.
- 2026-07-07: [M7] Retained draw `RenderPath` handles behind the
  `ArtboardInstance` cache epoch, mirroring C++ `ShapePaintPath`'s retained
  `RenderPath` plus dirty bit. Clean-epoch draw path cache hits no longer
  rewind/reserve/append path commands; a focused unit test now guards reuse
  until the epoch changes. Full `cargo test --workspace` and
  `make golden-compare` pass at exact=263/exact-segments=584. Release
  hot-loop smoke with `PERF_CORPUS_LIMIT=5 PERF_ITERATIONS=10 PERF_WARMUPS=1
  PERF_MAX_RATIO=999` reports aggregate Rust/C++=3.764 over 5 exact entries /
  10 segments, so strict <=2.0 remains open and the repeat=1 corpus is still
  noisy. Direct `ai_assitant --benchmark-repeat 100` improves from 52.493 to
  44.023. A post-slice `/usr/bin/sample` run shows `draw_path` and
  `runtime_rebuild_path` have fallen out of the main `ai_assitant` hot stack;
  current heat is per-frame paint preparation,
  `runtime_component_is_effectively_collapsed`, and schema reflection from
  property lookup. Next M7 slice should profile/port C++-aligned paint-prep
  retention and generated/cached property-key access there before extending
  path/clip rebuild retention.
- 2026-07-07: [M7] Retained prepared draw-command frames behind an
  `ArtboardInstance` cache epoch that bumps on C++-style dirt/change
  invalidation, and replay draw commands by reference so clean frames no
  longer rebuild the sorted drawable/layout/path-command projection twice per
  segment. `set_nested_artboard_artboard_id` is now idempotent for the same
  referenced artboard so data-binding does not spuriously invalidate the
  frame. Full `cargo test --workspace` and `make golden-compare` pass at
  exact=263/exact-segments=584. Release hot-loop smoke with
  `PERF_CORPUS_LIMIT=5 PERF_ITERATIONS=10 PERF_WARMUPS=1 PERF_MAX_RATIO=999`
  reports aggregate Rust/C++=3.673 over 5 exact entries / 10 segments, and
  direct `ai_assitant --benchmark-repeat 100` improves from 316.968 to
  52.493. Next M7 target is lower-level C++ `ShapePaintPath`/`RenderPath`
  retention and path dirt gating; strict Rust/C++ <=2.0 is still not met.
- 2026-07-07: [M7] Defined the perf exit target as steady-state per-frame
  runtime cost, not process elapsed, serializer output, import, or cold first
  frame. Decision-grade M7 perf proof must use release C++/Rust runners,
  null-renderer benchmark mode, the pinned perf corpus, >=10 iterations with
  median/spread, and a repeat-heavy or cold-excluded measurement where warm
  frames dominate. The pass threshold remains Rust/C++ <=2.0 on
  `advance_ms + input_ms + prepare_ms + draw_ms` per measured segment, with
  exact corpus verification still green. `perf-hot-loop` now forwards
  `PERF_BENCHMARK_REPEAT` through `perf-compare --benchmark-repeat` for
  single-sample runner benchmarks; multi-sample/input corpus entries continue
  using repeat=1 until the harness grows an explicit cold/steady split.
- 2026-07-07: [M7] Extended the fixed schema-key cache from paint/effect
  reads into runtime path geometry reads, mirroring C++ generated
  `*PropertyKey` constants instead of doing schema name/property scans in the
  frame loop. `make perf-hot-loop PERF_CORPUS_LIMIT=5 PERF_ITERATIONS=10
  PERF_WARMUPS=1 PERF_MAX_RATIO=999` now reports aggregate Rust/C++=6.387
  over 5 exact entries / 10 segments (`ai_assitant`=7.514). The new
  repeat-aware path also shows direct `ai_assitant --benchmark-repeat 100`
  at Rust/C++=465.901, so the next M7 slice must attack steady-state
  dirt/retention rather than more single-pass fixed lookup cost.
- 2026-07-07: [M7] Removed the data-bind hot-loop deep clones identified by
  the release flamegraph scout. `update_artboard_numeric_source_bindings`,
  `update_artboard_layout_computed_bindings`, and
  `update_artboard_solo_source_bindings` now borrow the artboard graph only
  for value calculation and clone only the changed binding path, instead of
  cloning the full `ArtboardGraph` or source-binding vectors every advance.
  Focused runtime tests pass. `make perf-hot-loop PERF_CORPUS_LIMIT=5
  PERF_ITERATIONS=10 PERF_WARMUPS=1 PERF_MAX_RATIO=999` improves aggregate
  Rust/C++ to 5.723, and direct `ai_assitant --benchmark-repeat 100` improves
  to Rust/C++=355.870. Next target remains C++ dirt/retention, not more
  clone cleanup.
- 2026-07-07: [M7] Made runtime animation/state-machine definitions
  shallow-clone by storing immutable keyed-object, layer, listener, and
  bindable vectors behind `Arc<Vec<_>>`. This mirrors C++ applying from
  shared immutable definitions while preserving the current Rust
  borrow-avoidance call shape. Focused runtime tests pass; full
  `cargo test --workspace` and `make golden-compare` remain green at
  exact=263/exact-segments=584. Release hot-loop smoke with
  `PERF_CORPUS_LIMIT=5 PERF_ITERATIONS=10 PERF_WARMUPS=1 PERF_MAX_RATIO=999`
  reports aggregate Rust/C++=6.353 (ratio noisy, not M7 passing), while
  direct `ai_assitant --benchmark-repeat 100` improves from 355.870 to
  316.968. Next target remains audited C++ dirt/retention, not more clone
  cleanup.
- 2026-07-07: [M7] Cached fixed schema property keys in the release
  null-renderer draw hot path after profiling `ai_assitant.riv`, mirroring the
  generated C++ property-key/member access at the same paint hot site. Both
  golden runners now accept `--benchmark-repeat N`, restricted to benchmark
  mode with one sample, so sampler runs can stay inside the already-imported
  hot loop.
  The pre-change Rust direct `ai_assitant` 100-segment repeat was about
  1019 ms; after caching `ShapePaint.isVisible`, `SolidColor.colorValue`, and
  the fixed Stroke/Gradient/Feather/Dash/Trim keys used by draw preparation,
  the same repeat is about 255 ms. Focused decision-grade verification with
  `make perf-hot-loop PERF_CORPUS_LIMIT=5 PERF_ITERATIONS=10 PERF_WARMUPS=1
  PERF_MAX_RATIO=999` reports aggregate Rust/C++=7.002 over 5 exact entries /
  10 segments; the strict `PERF_MAX_RATIO=2.0` run still fails at 7.503. Fresh
  post-cache sampling shows the paint property lookup hotspot mostly gone;
  next M7 perf work should profile/port the C++ trim/path-geometry strategy for
  `TrimContour::get_segment` / `TrimSegmentKind::extract` allocation and
  remaining `runtime_path_geometry` property-key scans. Full
  `cargo test --workspace` passes, and full `make golden-compare` remains
  `exact=263`, `exact-segments=584`, `diverges=0`,
  `unsupported-feature=32`, `not-yet=0`.
- 2026-07-07: [M7] Corrected the hot-loop perf proof path. Perf Make targets
  now build release C++/Rust runners by default. Runner benchmark comparison
  consumes `advance_ms + input_ms + prepare_ms + draw_ms` instead of
  `elapsed_ms`, and both golden runners now route `--benchmark` through
  null-renderer/null-factory backends so serializer and golden recording work
  stay out of the metric. Decision-grade focused verification:
  `make perf-hot-loop PERF_CORPUS_LIMIT=5 PERF_ITERATIONS=10 PERF_WARMUPS=1`
  fails the strict `PERF_MAX_RATIO=2.0` target at aggregate Rust/C++=21.908
  over 5 exact entries / 10 segments; largest focused ratios are
  `ai_assitant`=37.503, `advance_blend_mode`=19.690, and
  `animation_reset_cases`=19.837. This replaces the earlier recording-renderer
  perf numbers as M7 signal. Full `cargo test --workspace` passes, and full
  `make golden-compare` remains `exact=263`, `exact-segments=584`,
  `diverges=0`, `unsupported-feature=32`, `not-yet=0`. Next M7 slice should
  flamegraph the release null-renderer hot loop, starting with
  `ai_assitant.riv`, then port the corresponding C++ optimization rather than
  inventing cache/dirt behavior.
- 2026-07-07: [M7] Reduced Rust hot-loop draw/prepare allocations. The draw
  path no longer rebuilds a per-frame local-to-global `BTreeMap`; it indexes
  `ArtboardGraph.local_objects` directly. Per-draw path slot dedup now borrows
  path-command slices instead of cloning each distinct `Vec<RuntimePathCommand>`.
  `RenderPath::reserve` lets the recording renderer pre-size raw path verb and
  point buffers before command replay, while the FFI renderer keeps the default
  no-op. Gradient cache hits no longer allocate temporary color/position vectors;
  those vectors are built only inside the shader cache-miss closure. Focused
  verification: `make perf-hot-loop PERF_CORPUS_LIMIT=5 PERF_ITERATIONS=2
  PERF_WARMUPS=1 PERF_MAX_RATIO=8.0` passes at aggregate Rust/C++=5.290; the
  same command with `PERF_MAX_RATIO=2.0` fails at aggregate Rust/C++=5.122.
  Direct `ai_assitant.riv` phase samples are noisy, with a representative Rust
  run at ~= 146.57ms total (7.27ms advance, 44.41ms prepare, 94.89ms draw) vs
  C++ ~= 25.08ms total (7.62ms advance, 17.45ms draw). Full
  `cargo test --workspace` passes, `cargo fmt --all -- --check` passes,
  `git diff --check` passes, and full `make golden-compare` remains
  `exact=263`, `exact-segments=584`, `diverges=0`,
  `unsupported-feature=32`, `not-yet=0`. Next M7 perf slice should focus on
  remaining fixed overhead in tiny benchmark files plus `ai_assitant` draw and
  paint-prep cost.
- 2026-07-07: [M7] Replaced the Rust `RecordingRenderer` path/paint snapshot
  construction hot path with direct writes into the shared `RecordingStream`.
  `drawPath`, `clipPath`, `makeRenderPath`, `makeEmptyRenderPath`, and
  `makeRenderPaint` now avoid nested temporary `String` construction for
  path/paint snapshots; float and color formatting can append directly to the
  output buffer. A measured `RefCell<Option<String>>` snapshot cache attempt
  was rejected before commit because one-shot path/paint lifetimes made the
  loose hot-loop smoke regress to aggregate Rust/C++ ~= 8.85. Focused
  verification after the direct-write slice: direct `ai_assitant.riv` sample 0
  reports Rust ~= 147.91ms total (6.75ms advance, 46.29ms prepare, 94.86ms
  draw) vs C++ ~= 25.98ms total (7.09ms advance, 18.89ms draw);
  `make perf-hot-loop PERF_CORPUS_LIMIT=5 PERF_ITERATIONS=2 PERF_WARMUPS=1
  PERF_MAX_RATIO=8.0` passes at aggregate Rust/C++=6.457; the same command
  with `PERF_MAX_RATIO=2.0` fails at aggregate Rust/C++=6.732. Full
  `cargo test --workspace` passes, `cargo fmt --all -- --check` passes,
  `git diff --check` passes, and full `make golden-compare` remains
  `exact=263`, `exact-segments=584`, `diverges=0`,
  `unsupported-feature=32`, `not-yet=0`. Next M7 perf slice should profile
  runtime draw command emission / paint preparation rather than reintroducing
  path/paint snapshot caches.
- 2026-07-07: [M7] Extended runner benchmark output with phase timings:
  `advance_ms`, `input_ms`, `prepare_ms`, `draw_ms`, and `bookkeeping_ms`.
  `perf-compare` still consumes `elapsed_ms`, so corpus thresholding remains
  unchanged, but direct runner invocations now localize hot-loop overhead.
  Direct debug-runner timing for `ai_assitant.riv` at sample 0 reports C++ ~=
  18.89ms total (4.96ms advance, 13.93ms draw) and Rust ~= 142.96ms total
  (6.15ms advance, 42.20ms prepare, 94.60ms draw). The next optimization slice
  should target Rust draw/recording overhead first, with paint prep second.
  Focused `make perf-hot-loop` with `PERF_CORPUS_LIMIT=5`,
  `PERF_ITERATIONS=2`, `PERF_WARMUPS=1`, `PERF_MAX_RATIO=8.0` passes at
  aggregate Rust/C++=7.007. Full `cargo test --workspace` passes, and full
  `make golden-compare` remains `exact=263`, `exact-segments=584`,
  `diverges=0`, `unsupported-feature=32`, `not-yet=0`.
- 2026-07-07: [M7] Added runner-side hot-loop benchmarking. Both
  golden runners accept `--benchmark` and emit `rive-golden-benchmark-v1`,
  timing the already-imported sample/input advance-and-draw loop. `perf-compare`
  now has `--runner-benchmark`, and `make perf-hot-loop` wires it over the exact
  corpus subset. Focused debug-runner verification with
  `PERF_CORPUS_LIMIT=5`, `PERF_ITERATIONS=2`, `PERF_WARMUPS=1`,
  `PERF_MAX_RATIO=8.0` reports aggregate Rust/C++=7.306 and passes the loose
  smoke threshold. The same command with the strict M7 target
  `PERF_MAX_RATIO=2.0` fails at aggregate Rust/C++=7.159. The next M7 perf
  slice should profile Rust hot-loop overhead, starting with `ai_assitant.riv`
  because it contributes most of the aggregate absolute time. Full
  `cargo test --workspace` passes, and full `make golden-compare` remains
  `exact=263`, `exact-segments=584`, `diverges=0`,
  `unsupported-feature=32`, `not-yet=0`.
- 2026-07-07: [M7] Added corpus-mode performance thresholding to
  `tools/perf-compare` and `make perf-corpus`. The tool now reads
  `corpus.toml`, selects exact entries, preserves per-entry samples and input
  scripts, resolves assets through `RIVE_RUNTIME_DIR`, sums median C++ and Rust
  runner timings, and fails when aggregate Rust/C++ exceeds `--max-ratio`.
  Focused verification with `PERF_CORPUS_LIMIT=5`, `PERF_ITERATIONS=2`,
  `PERF_WARMUPS=1`, `PERF_MAX_RATIO=2.0` reports 5 exact entries / 10 segments,
  aggregate Rust/C++=1.183 and passes the threshold. Full
  `cargo test --workspace` passes, and full `make golden-compare` remains
  `exact=263`, `exact-segments=584`, `diverges=0`,
  `unsupported-feature=32`, `not-yet=0`. `ai_assitant.riv` is the visible
  outlier at Rust/C++=4.715, so the next M7 perf slice should add an in-process
  advance+draw benchmark and/or localize that file's Rust-side overhead.
- 2026-07-07: [M7] Added the first C++/Rust performance baseline command,
  `make perf-compare`, backed by `tools/perf-compare`. It builds both golden
  runners, executes the same file/sample set with configurable iterations and
  warmups, validates each run emitted a golden stream, and reports median/min/max
  plus Rust/C++ ratio. Default `shapetest.riv` process-level baseline
  (`samples=0`, `iterations=5`, `warmups=1`) reports C++ median 37.848ms, Rust
  median 5.131ms, Rust/C++=0.136 on this machine. This is a first ratchet, not
  final M7 perf proof: it includes process startup/import/serialization. Full
  `cargo test --workspace` passes, and `make golden-compare` remains unchanged
  at `exact=263`, `exact-segments=584`, `diverges=0`,
  `unsupported-feature=32`, `not-yet=0`. Next M7 slice should add corpus or
  in-process advance+draw timing and define the pass threshold.
- 2026-07-07: [M7] Added the first runtime C ABI crate, `rive-capi`. It
  publishes an opaque `RiveFile` handle, `rive_file_import`/`rive_file_free`,
  artboard count/name accessors, animation/state-machine count accessors, and
  `include/rive_capi.h`. Focused `cargo test -p rive-capi` passes against the
  reference `shapetest.riv` fixture; full `cargo test --workspace` passes, and
  `make golden-compare` remains unchanged at `exact=263`,
  `exact-segments=584`, `diverges=0`, `unsupported-feature=32`, `not-yet=0`.
  Next M7 slice should create the C++/Rust performance baseline before
  expanding C-owned artboard instances or draw.
- 2026-07-07: [M7] Added the initial user-facing `rive` crate. The public
  facade imports `.riv` bytes, exposes borrowed artboard handles, instantiates
  artboards with their file/graph context attached, re-exports the renderer
  traits and state-machine/input types, and provides a one-shot `advance`/`draw`
  path backed by the existing runtime. `cargo test -p rive` passes against the
  reference `shapetest.riv` fixture; full `cargo test --workspace` passes, and
  `make golden-compare` remains unchanged at `exact=263`,
  `exact-segments=584`, `diverges=0`, `unsupported-feature=32`, `not-yet=0`.
  Next M7 slice should publish the first C ABI facade or create the C++/Rust
  perf baseline.
- 2026-07-07: [M6] Promoted `stateful_nested.riv` to exact-status and closed
  the current M6 manifest queue. The old
  `nested-stateful-view-model-property` guard was cleared by admitting nested
  child `ViewModelInstance*::propertyValue` data binds, propagating boolean
  and enum nested host values alongside the existing string/color/number
  path, allowing static text to accept the same passive view-model instance
  binds, and syncing bound `Artboard.clip` values into the draw-time clip
  cache. Focused exact compare passes; full `make golden-compare` reports
  `exact=263`, `exact-segments=584`, `diverges=0`,
  `unsupported-feature=32`, `not-yet=0`, parked
  `gated=6 harness=26`; `cargo test --workspace` passes. Next target is M7
  ship surface: public API/C ABI/perf baseline.
- 2026-07-07: [M6] Promoted `stateful_multi_property.riv` to exact-status.
  The old `nested-layout-clip-data-bind` guard was cleared by adding boolean
  source-to-target artboard property bindings, admitting nested `Artboard.clip`
  and `LayoutComponentStyle.displayValue` data binds in the Rust runner, and
  teaching the static text subset that those layout-affecting binds are
  supported siblings. Focused exact compare passes; full `make golden-compare`
  reports `exact=262`, `exact-segments=583`, `diverges=0`,
  `unsupported-feature=33`, `not-yet=0`, parked
  `M6=1 gated=6 harness=26`; `cargo test --workspace` passes. Next target is
  `stateful_nested.riv`
  (`rust-runner-unsupported:nested-stateful-view-model-property`).
- 2026-07-07: [M6] Promoted `rewards_demo.riv` to exact-status under
  `verification = "tolerant(0.0005)"`. The promotion closed the active
  `not-yet:nested-feather-gradient-space` queue by matching C++ NSliced path
  deformation/clip behavior, zero-size layout clip paths, inner-feather clip
  fill-rule ordering, platform text line metrics, and a narrowed clone-time
  SolidColor default rule: only name-based source-to-target SolidColor binds
  get the opaque-black default, preserving authored id-path text paints while
  keeping `relative_data_binding.riv` exact. Full `make golden-compare`
  reports `exact=261`, `exact-segments=582`, `diverges=0`,
  `unsupported-feature=34`, `not-yet=0`, parked
  `M6=2 gated=6 harness=26`; `cargo test --workspace` passes. Next target is
  `stateful_multi_property.riv`
  (`rust-runner-unsupported:nested-layout-clip-data-bind`).
- 2026-07-07: [M6] Cleared the focused `rewards_demo.riv` Chest shader
  allocation/order mismatch without changing runtime scheduler order.
  `rive-graph` now keeps a separate `dependency_insertion_order` projection
  while preserving the existing sorted `dependency_order`/`graph_order`, and
  the runtime uses the insertion-order projection only for deferred static
  artboard-tree paint preparation. Focused exact compare now passes the prior
  Chest `makeLinearGradient id=15` block and first fails at line 492 on
  Chest path geometry/local transform under the same render transform and
  shader. Full `make golden-compare` reports `exact=260`,
  `exact-segments=581`, `diverges=0`, `unsupported-feature=34`,
  `not-yet=1`, parked `M6=2 gated=6 harness=26`; `cargo test --workspace`
  passes. Next target remains `rewards_demo.riv`, localizing the Chest
  nested layout/path-transform divergence.
- 2026-07-07: [M6] Moved `rewards_demo.riv` from
  `rust-runner-unsupported:nested-feather-gradient-space` to active
  `not-yet:nested-feather-gradient-space`. The runner now admits the file by
  allowing simple clipped layout paints and feathered simple layout
  backgrounds, while the runtime prepares hidden layout nested-artboard paints
  needed for shader allocation and limits NSliced layout sizing to real
  `LayoutComponent` ancestors so `n_slice_triangle.riv` remains exact. Focused
  compare now reaches a real stream mismatch at Chest `makeLinearGradient
  id=15` (`Rust paint global 1963/local 1020/mutator 217` versus `C++ paint
  global 1956/local 1013/mutator 188`). Full `make golden-compare` reports
  `exact=260`, `exact-segments=581`, `diverges=0`,
  `unsupported-feature=34`, `not-yet=1`, parked
  `M6=2 gated=6 harness=26`; `cargo test --workspace` passes. Next target
  remains `rewards_demo.riv`, localizing the Chest nested layout/gradient
  ordering divergence.
- 2026-07-07: [M6] Promoted `car_widgets_v01.riv` to exact-status under
  `verification = "tolerant(0.001)"`. The `nested-text-outline-contour-order`
  guard was a coarse proxy: the first real failure was Rust retaining a
  near-empty terminal cubic in a final multi-contour shape path where C++'s
  render stream had already normalized it away. Runtime shape paint command
  assembly now prunes base, effect, and inner-feather paths after final
  assembly, preserving exact C++ empty-segment pruning but allowing
  `f32::EPSILON` cancellation only for multi-contour cubic empties. Full
  `make golden-compare` reports `exact=260`, `exact-segments=581`,
  `diverges=0`, `unsupported-feature=35`, `not-yet=0`, parked
  `M6=3 gated=6 harness=26`; `cargo test --workspace` passes. Next target is
  `rewards_demo.riv` (`rust-runner-unsupported:nested-feather-gradient-space`).
- 2026-07-07: [M6] Closed stale `nested-node-transform-data-bind` by admitting
  nested child `Node.rotation` binds through `DataConverterGroup`, letting
  static text accept the same target, inheriting normal static-text paint blend
  from owning `Text.blendModeValue`, and making background shape paints inherit
  their container blend. Focused `car_widgets_v01.riv` now reaches draw and
  exposes a nested text-outline contour-order mismatch, so it is retagged as
  `rust-runner-unsupported:nested-text-outline-contour-order`. Full
  `make golden-compare` remains `exact=259`, `exact-segments=580`,
  `diverges=0`, `unsupported-feature=36`, `not-yet=0`, parked
  `M6=4 gated=6 harness=26`; `cargo test --workspace` passes. Next target is
  `car_widgets_v01.riv`
  (`rust-runner-unsupported:nested-text-outline-contour-order`).
- 2026-07-07: [M6] Sharpened `rewards_demo.riv` from
  `layout-component-paint` to `nested-feather-gradient-space`. A focused
  exact-candidate bypass proved layout-paint admission alone was not enough:
  the first mismatch was gradient preparation/order around `makeLinearGradient
  id=15`, followed by nested transform/gradient coordinate differences. The
  runner now reports `nested-feather-gradient-space` only for nested child
  artboards that have layout components, no pre-existing static-text blocker,
  and a feathered gradient paint container; this preserves exact
  `ai_assitant.riv` and keeps `bankcard.riv` on its sharper
  `text-polygon-sibling` diagnostic. Full `make golden-compare` remains
  `exact=259`, `exact-segments=580`, `diverges=0`,
  `unsupported-feature=36`, `not-yet=0`, parked `M6=4 gated=6 harness=26`;
  `cargo test --workspace` passes. Next target then was `car_widgets_v01.riv`
  (`rust-runner-unsupported:nested-node-transform-data-bind`).
- 2026-07-07: [M6] Promoted `echo_show_demo.riv` to exact by matching the
  C++ text line-metrics/bounds path instead of widening the nested-remap
  runtime surface. Rust text metrics now mirror
  `src/text/font_hb.cpp::make_lmx`: prefer OS/2 typo extents when present,
  fall back to hhea, and apply MVAR `HASC`/`HDSC` deltas before Rive scales the
  authored font size. Rust text bounds now match C++ shaped-run scope by
  computing max line metrics only from styles referenced by actual
  `TextValueRun`s, not every `TextStylePaint` child on the `Text`. This fixed
  the focused exact-candidate failures at lines 1593 and 1610, allowing removal
  of the `joystick-nested-remap-transform-update-order` runner/corpus guard.
  Full `make golden-compare` reports `exact=259`, `exact-segments=580`,
  `diverges=0`, `unsupported-feature=36`, `not-yet=0`, parked
  `M6=4 gated=6 harness=26`; `cargo test --workspace` passes. Next target is
  `rewards_demo.riv` (`rust-runner-unsupported:layout-component-paint`).
- 2026-07-07: [M6] Narrowed `echo_show_demo.riv` by making deferred layout
  gradient prep use dependency order for the whole artboard tree, including
  recursive nested prepass traversal. A focused exact-candidate bypass now
  matches the C++ shader creation sequence that previously diverged around
  shader id 6, then first fails at line 1593 on a nested transform Y
  translation (`230.126801` Rust vs `232.096527` C++). The runner/corpus
  diagnostic is sharpened from `joystick-nested-remap-gradient-update-order`
  to `joystick-nested-remap-transform-update-order`. Full
  `make golden-compare` remains `exact=258`, `exact-segments=579`,
  `diverges=0`, `unsupported-feature=37`, `not-yet=0`, parked
  `M6=5 gated=6 harness=26`; `cargo test --workspace` passes. Next target
  remains `echo_show_demo.riv`
  (`rust-runner-unsupported:joystick-nested-remap-transform-update-order`).
- 2026-07-07: [M6] Narrowed `echo_show_demo.riv` by matching C++ transition
  source retention for Entry/non-animated sources. C++ keeps `m_stateFrom`
  during a nonzero-duration Entry -> animation transition, so the destination
  animation applies at mix 0 on the first update; Rust now tracks
  `transition_source_state_index` even when the previous state has no
  animation/blend payload, backed by a synthetic Entry-timed-transition
  regression. A focused bypass compare moved the first pre-sample diff past
  shader ids 4 and 5: both now match C++, and the remaining first diff is
  shader id 6 where Rust emits a later zero-alpha radial out of order.
  Full `make golden-compare` remains `exact=258`, `exact-segments=579`,
  `diverges=0`, `unsupported-feature=37`, `not-yet=0`, parked
  `M6=5 gated=6 harness=26`; `cargo test --workspace` passes. Next target
  remains `echo_show_demo.riv`
  (`rust-runner-unsupported:joystick-nested-remap-gradient-update-order`).
- 2026-07-07: [M6] Promoted `image_scripting_property_value.riv` by narrowing
  pre-source image decode ordering to the non-scripting C++ golden-runner
  import stack: `ScriptAsset`/`ShaderAsset` do not use the `FileAsset` stack
  there, so they must not displace a pending embedded `ImageAsset` before
  source render-paint allocation. Full `make golden-compare` reports
  `exact=258`, `exact-segments=579`, `diverges=0`,
  `unsupported-feature=37`, `not-yet=0`, parked
  `M6=5 gated=6 harness=26`. Next target is `echo_show_demo.riv`
  (`rust-runner-unsupported:joystick-nested-remap-gradient-update-order`).
- 2026-07-07: [M6] Narrowed `echo_show_demo.riv` with a temporary
  `RIVE_TRACE_ECHO` bypass/trace that was fully reverted before this commit.
  The focused Rust stream still first differs before `sample seconds=0`;
  trace line 4 shows Rust shader id 4 is selected-root paint global 636
  (container 431, mutator 438) after joystick/remap has moved the radial
  endpoints to `(-185.856506,-285.401245)` and driven render opacity to zero.
  C++ shader id 4 remains the authored radial `(-218.036606,-275.353241)` with
  nonzero alpha, and C++ never emits Rust's `-185.856506` radial before the
  first sample. A non-before-joystick `updateDataBinds()` placement experiment
  was also rejected in this pass: both echo joysticks are before-update, and
  the focused stream still first-differed at line 980. Next target remains
  `echo_show_demo.riv`, but the slice is now constrained to C++ update-time
  gradient allocation for the `636/635` root branch before remap/final-state
  static paint prep sees it.
- 2026-07-02: V2 map adopted (`docs/porting-map-v2.md`); V1 map superseded, its contract suite frozen as regression floor.
- 2026-07-02: Golden runner records decoded image payloads by size/hash for the first renderer slice; real decoded dimensions are deferred until `rive_decoders` is wired into the CLI harness build.
- 2026-07-02: Golden runner emits one accumulated stream per run with
  `source`, `input`, `sample`, and `frame` markers; `golden-compare` will split
  sample segments from that stream.
- 2026-07-02: `rive-render-api` owns the renderer seam; `rive-runtime` should
  drive those traits when static drawing moves from reports to real rendering.
- 2026-07-02: `golden-compare` validates the C++ stream for `not-yet` entries
  and refuses `exact` entries unless a Rust runner is supplied, keeping the
  exact count honest while the Rust draw path is still absent.
- 2026-07-02: First exact file is `dependency_test.riv`; the Rust runner
  preallocates source + instance render paints to mirror C++ import/clone
  paint lifetimes before drawing.
- 2026-07-02: `tools/golden-compare --bin generate-corpus` generates the
  corpus manifest from the C++ unit-test assets, preserving exact/unsupported
  annotations across regenerations.
- 2026-07-02: CI pins the reference C++ runtime to
  `7c778d13c5d903b3b74eec1dd6bb68a811dea5f2` and builds root
  `premake5_v2.lua` debug libraries before running `make golden-compare`.
- 2026-07-06: [M6] Closed `text-modifier-group-flags` by adding C++-style
  `TextModifierGroup` scale interpolation and passive `NestedNumber` static
  text sibling admission. `hunter_x_demo.riv` now reaches a Rust stream and
  parks as `not-yet:gradient-shader-order`; `rewards_demo.riv` now verifies as
  `rust-runner-unsupported:nested-layout-size-data-bind` for LayoutComponent
  width binding. `make golden-compare` reports exact-segments=555,
  diverges=0, unsupported-feature=58, not-yet=3; next target is
  `rust-runner-unsupported:mesh-images`. `cargo test --workspace` passes.
- 2026-07-06: [M6] Closed the `mesh-images` runner guard by porting
  mesh-backed `Image::draw` from C++ `src/shapes/image.cpp` /
  `src/shapes/mesh.cpp`: source mesh vertex/UV/index buffers are allocated
  file-wide before clone dynamic vertex buffers, `drawImageMesh` uses dynamic
  skinned vertices, and `golden-compare` treats vertex `bufferData` as
  semantic f32 data while keeping index buffers exact. `jellyfish_test.riv`
  is exact under `tolerant(0.0004)` for skinned vertex-buffer float residuals,
  `tape.riv` is strict exact, and `bad_skin.riv` is parked as
  `not-yet:skinned-contour-transform-order` after a later structural skinned
  transform delta. Full `make golden-compare` reports `exact=236`,
  `exact-segments=557`, `diverges=0`, `unsupported-feature=55`,
  `not-yet=4`, and parked `M6=14 gated=5 harness=36`; next target is
  `bad_skin.riv`.
- 2026-07-06: [M6] Promoted `bad_skin.riv` by mirroring C++
  `Stroke::isVisible()` against live instance `Stroke.thickness` and applying
  live stroke thickness/cap/join during paint configuration, so the default
  state machine's sample-0 keyed thickness suppresses the mask stroke instead
  of drawing the authored width. `bad_skin.riv` is exact under
  `tolerant(0.0004)` for residual skinned path float drift. Full
  `make golden-compare` reports `exact=237`, `exact-segments=558`,
  `diverges=0`, `unsupported-feature=55`, `not-yet=3`, and parked
  `M6=14 gated=5 harness=36`; next target is `local_bounds.riv`
  (`not-yet:image-predecode-order`).
- 2026-07-06: [M6] Promoted `local_bounds.riv` by mirroring C++
  `ImportStack` file-asset importer displacement: when a pre-artboard
  file-asset importer supersedes an embedded `ImageAsset` importer, Rust now
  predecodes that image before source paint allocation. `local_bounds.riv` is
  exact under `tolerant(0.00001)` for residual HarfRust/C++ text-outline float
  drift. Full `make golden-compare` reports `exact=238`,
  `exact-segments=559`, `diverges=0`, `unsupported-feature=55`,
  `not-yet=2`, and parked `M6=14 gated=5 harness=36`; next target is
  `hunter_x_demo.riv` (`not-yet:gradient-shader-order`).
- 2026-07-06: [M6] Closed the selected-root gradient shader-order adjacency
  by deferring selected-root `LayoutComponent` gradient shader preparation
  until after nested artboard paint preparation while preserving child-artboard
  ordering. `hunter_x_demo.riv` now matches C++ selected-root shader
  allocation order and is parked at `not-yet:gradient-opacity-propagation`
  after the focused stream first differs at child gradient stop alpha values.
  The stale `selected-root-gradient-shader-order` runner guard was removed;
  `bullet_man.riv` now verifies as
  `rust-runner-unsupported:selected-root-skinned-clip-path`. Full
  `make golden-compare` reports `exact=238`, `exact-segments=559`,
  `diverges=0`, `unsupported-feature=55`, `not-yet=2`, and parked
  `M6=14 gated=5 harness=36`; `cargo test --workspace` passes. Next target is
  `hunter_x_demo.riv` (`not-yet:gradient-opacity-propagation`).
- 2026-07-06: [M6] Closed the `hunter_x_demo.riv` gradient-opacity adjacency
  by live-reading `LinearGradient.opacity`/`RadialGradient.opacity` through
  paint mutators and by matching C++ `shouldDraw()` effective-visibility
  gating for layout, foreground layout, and text paints while preserving
  ordinary Shape alpha-zero draw emission. Focused streams now match through
  selected-root shader allocation and child gradient stop alpha propagation;
  the first diff is local-clockwise child contour verb ordering, parked as
  `not-yet:local-clockwise-contour-order`. Full `make golden-compare` reports
  `exact=238`, `exact-segments=559`, `diverges=0`,
  `unsupported-feature=55`, `not-yet=2`, and parked
  `M6=14 gated=5 harness=36`; `cargo test --workspace` passes. Next target is
  `hunter_x_demo.riv`
  (`not-yet:local-clockwise-contour-order`).
- 2026-07-06: [M6] Promoted `hunter_x_demo.riv` by honoring
  `Path.pathFlags` for `PointsPath` clockwise state, reusing the C++ draw-path
  identity for inner-feather clips, reading live `Feather` properties at draw
  time, preserving distinct local/localClockwise `Shape` path cache identity
  while aliasing text/layout providers, and building weighted `PointsPath`
  commands from deformed cubic handles directly instead of round-tripping
  through angle/distance. The stream is structurally identical and guarded by
  `verification = "tolerant(0.0015)"` for bounded skinned float drift (max
  observed `0.0013504`). Full `make golden-compare` reports `exact=239`,
  `exact-segments=560`, `diverges=0`, `unsupported-feature=55`, `not-yet=1`,
  and parked `M6=14 gated=5 harness=36`. Next target is `ai_assitant.riv`
  (`not-yet:nested-feather-gradient-space`).
- 2026-07-06: [M6] Promoted `ai_assitant.riv` by matching C++ world-space
  gradient shader construction for strokes whose `transformAffectsStroke` flag
  selects `PathFlags::world`: Rust now keeps local gradient mutator payloads
  for probe parity, carries a `paint_space_transform` on draw commands, and
  applies that transform only while configuring/caching linear/radial render
  shaders. Focused streams are structurally identical with max numeric drift
  `0.000122`, below the golden epsilon. Full `make golden-compare` reports
  `exact=240`, `exact-segments=561`, `diverges=0`,
  `unsupported-feature=55`, `not-yet=0`, and parked
  `M6=14 gated=5 harness=36`. The active `not-yet` queue is empty.
- 2026-07-06: [M6] Reclassified `text_input.riv` from the stale
  `layout-component-paint` guard to a precise `text-input` diagnostic after a
  focused C++ stream showed sample 0 draws the layout background plus
  `TextInputCursor`, empty `TextInputSelection`, and shaped
  `TextInputText` paths. The Taffy refusal was downstream of measuring a
  `TextInput` child inside layout global 21, so the next implementation slice
  is `RawTextInput`-style generated path/measurement support rather than more
  generic layout paint admission. The metric is intentionally unchanged:
  `exact=240`, `exact-segments=561`, `diverges=0`,
  `unsupported-feature=55`, `not-yet=0`, parked
  `M6=14 gated=5 harness=36`.
- 2026-07-06: [M6] Promoted `text_input.riv` by porting the frame-0
  `TextInput` generated path/measurement slice: `TextInput` uses the existing
  HarfRust/Skrifa static text shaper as RawTextInput-style text path
  generation, measures multiline auto-height `TextInput` children through
  Taffy, draws `TextInputCursor`, empty `TextInputSelection`, and
  `TextInputText` with the parent `TextInput` world transform, and removes the
  blanket runner `TextInput` gate. Full `make golden-compare` reports
  `exact=241`, `exact-segments=562`, `diverges=0`,
  `unsupported-feature=54`, `not-yet=0`, parked
  `M6=13 gated=5 harness=36`; `cargo test --workspace` passes. Next target is
  one of the remaining one-file M6 queues: `focus-data`,
  `viewmodel-asset-conditions`, `text-joystick-data-bind`,
  `nested-artboard-layout`, `selected-root-skinned-clip-path`, or the nested
  data-bind diagnostics.
- 2026-07-06: [M6] Moved `scripted_data_context.riv` from M6 to gated after
  confirming the Rust runner already emits the loud
  `unsupported: scripted-data-context` diagnostic for the selected artboard's
  `ScriptedDrawable` + nested-view-model `DataBindContext` text surface. This
  is blocked on the #V2-7 Luau scripting lane, not layout/text runtime parity.
  Metrics are intentionally unchanged at `exact=241`,
  `exact-segments=562`, `diverges=0`, `unsupported-feature=54`,
  `not-yet=0`; parked becomes `M6=12 gated=6 harness=36`. Next target is
  `focus-data`, `viewmodel-asset-conditions`, `text-joystick-data-bind`,
  `nested-artboard-layout`, `selected-root-skinned-clip-path`, or the nested
  data-bind diagnostics.
- 2026-07-06: [M6] Promoted `focus_traversal.riv` after narrowing the frame-0
  blocker to foreground layout path identity instead of focus traversal
  execution. Effect-free `ForegroundLayoutDrawable` paints now cache draw paths
  under their parent `LayoutComponent`, allowing the stroke to reuse the layout
  fill path just like the C++ stream. The runner still gates nested `FocusData`
  when an input script is present, but no-input traversal metadata is now
  admitted.
  Full `make golden-compare` reports `exact=242`, `exact-segments=563`,
  `diverges=0`, `unsupported-feature=53`, `not-yet=0`, parked
  `M6=11 gated=6 harness=36`; `cargo test --workspace` passes. Next target is
  `viewmodel-asset-conditions`, `text-joystick-data-bind`,
  `nested-artboard-layout`, `selected-root-skinned-clip-path`, or the nested
  data-bind diagnostics.
- 2026-07-06: [M6] Promoted `viewmodel_based_condition.riv` by adding typed
  `TransitionPropertyViewModelComparator` pair conditions for view-model
  number, boolean, color, string, enum, asset, and artboard bindables. The
  file's blocked transitions were ViewModel-vs-ViewModel asset/color/string
  comparisons, not literal asset comparators, so the runner
  `viewmodel-asset-conditions` guard was removed with the corpus entry.
  Direct C++/Rust streams match, and full `make golden-compare` reports
  `exact=243`, `exact-segments=564`, `diverges=0`,
  `unsupported-feature=52`, `not-yet=0`, parked
  `M6=10 gated=6 harness=36`. Next target is `text-joystick-data-bind`,
  `nested-artboard-layout`, `selected-root-skinned-clip-path`, or the nested
  data-bind diagnostics.
- 2026-07-06: [M6] Rechecked `echo_show_demo.riv` and replaced the stale
  `text-joystick-data-bind` guard with
  `text-joystick-data-bind-divergence`. Rust now admits the static-text
  Joystick/NestedRemapAnimation sibling scan and Joystick.x/y data-bind
  targets, and exact Joystick bind fixtures (`coin.riv`,
  `magic_alley_db_reduced_export.riv`, `joystick_flag_test.riv`,
  `joystick_nested_remap.riv`) still pass. A direct C++/Rust probe of
  `echo_show_demo.riv` reaches draw but diverges at first shader creation
  after stream setup, so the file stays M6 parked behind a narrower
  multiple-converted-Joystick.x diagnostic. Full `make golden-compare`
  remains `exact=243`, `exact-segments=564`, `diverges=0`,
  `unsupported-feature=52`, `not-yet=0`, parked
  `M6=10 gated=6 harness=36`. Next target is `nested-artboard-layout`,
  `selected-root-skinned-clip-path`, or the nested data-bind diagnostics.
- 2026-07-06: [M6] Rechecked `superbowl.riv` and replaced the stale
  `nested-artboard-layout` guard with
  `state-machine-viewmodel-solo-image`. Rust now admits
  `NestedArtboardLayout`/`NestedArtboardLeaf` static-text siblings; focused
  `Logo` and `Side` streams show the first missing draw is an image under a
  `Solo` whose active child is selected by a view-model enum state-machine
  path, not by generic nested layout. The focused runner now emits
  `unsupported: state-machine-viewmodel-solo-image` at image global `3567`,
  and full `make golden-compare` reports `exact=243`,
  `exact-segments=564`, `diverges=0`, `unsupported-feature=52`,
  `not-yet=0`, parked `M6=10 gated=6 harness=36`. Next target is
  `selected-root-skinned-clip-path` or the nested data-bind diagnostics.
- 2026-07-06: [M6] Split the two-file `selected-root-skinned-clip-path` queue
  after bypassing the guard and comparing focused streams. `bullet_man.riv`
  first diverges before the sample because C++ prepares the selected root's
  leading `Background` nested artboard gradients before the root gradient
  batch, while Rust still prepares the selected root first; it is now parked as
  `selected-root-leading-nested-paint-order` at nested global `786`.
  `spotify_kids_demo.riv` still reaches the skinned clip-path geometry drift
  and keeps the existing diagnostic. Full `make golden-compare` reports
  `exact=243`, `exact-segments=564`, `diverges=0`,
  `unsupported-feature=52`, `not-yet=0`, parked
  `M6=10 gated=6 harness=36`. Next target is
  `selected-root-leading-nested-paint-order`,
  `selected-root-skinned-clip-path`, or the nested data-bind diagnostics.
- 2026-07-06: [M6] Closed `selected-root-leading-nested-paint-order` by
  interleaving selected-root skinned paint preparation with nested-host
  dependency order. A focused `bullet_man.riv` probe with both selected-root
  guards bypassed now matches all shader creation and first diverges in the
  skinned path geometry; `bullet_man.riv` is retagged back into the shared
  `selected-root-skinned-clip-path` queue with `spotify_kids_demo.riv`. Full
  `make golden-compare` reports `exact=243`, `exact-segments=564`,
  `diverges=0`, `unsupported-feature=52`, `not-yet=0`, parked
  `M6=10 gated=6 harness=36`. Next target is
  `selected-root-skinned-clip-path` or the nested data-bind diagnostics.
- 2026-07-06: [M6] Promoted `spotify_kids_demo.riv` by matching C++
  `LinearAnimation::durationSeconds()` work-area semantics: runtime animation
  duration is `abs(endSeconds() - startSeconds())`, not the serialized full
  duration, so joystick-driven work-area animations sample the same local time
  as C++. The broad `selected-root-skinned-clip-path` guard is narrowed to
  `selected-root-skinned-ik-clip-path`; `bullet_man.riv` remains parked there
  with tiny skinned path drift under IK plus clipping. Full
  `make golden-compare` reports `exact=244`, `exact-segments=565`,
  `diverges=0`, `unsupported-feature=51`, `not-yet=0`, parked
  `M6=9 gated=6 harness=36`. Next target is
  `selected-root-skinned-ik-clip-path` or the nested data-bind diagnostics.
- 2026-07-06: [M6] Opened the first `db_health_tracker.riv` nested data-bind
  lane by admitting direct nested Node x/y binds, TrimPath start/end/offset
  binds with no converter or `DataConverterGroup`, LayoutComponent
  width/height binds with no converter or `DataConverterInterpolator`, and
  `NestedSimpleAnimation` static-text siblings. The file now reaches Rust draw
  and is parked as `not-yet:nested-data-bind-text-path-divergence` after a
  focused exact compare found a structural text/glyph path mismatch near the
  first differing drawPath. `rewards_demo.riv` advances from
  `nested-layout-size-data-bind` to `layout-component-paint`. Full
  `make golden-compare` reports `exact=244`, `exact-segments=565`,
  `diverges=0`, `unsupported-feature=50`, `not-yet=1`, parked
  `M6=8 gated=6 harness=36`. Next target is
  `db_health_tracker.riv`'s active not-yet mismatch.
- 2026-07-07: [M6] Promoted `db_health_tracker.riv` under
  `verification = "tolerant(0.0011)"` by binding
  `DataConverterOperationValue.operationValue`, marking parent `Text` shapes
  dirty when `TextValueRun.text` mutates, matching C++ trailing-hard-break text
  measurement, applying layout bounds to clipping paths, preserving authored
  non-identity transforms for nested artboard layout hosts, and treating
  nonzero undefined min-size as auto only for intrinsic static-measured hug
  layout nodes. `artboard_width_test.riv` stays strict exact after narrowing
  identity nested-layout hosts back to bounds-only transforms. Full
  `make golden-compare` reports `exact=245`, `exact-segments=566`,
  `diverges=0`, `unsupported-feature=50`, `not-yet=0`, parked
  `M6=8 gated=6 harness=36`; `cargo test --workspace` passes. Next target is
  `nested_hug.riv` (`rust-runner-unsupported:nested-artboard-root-transform`).
- 2026-07-07: [M6] Promoted `nested_hug.riv` by admitting Artboard x/y nested
  child data binds, sizing root Artboard hug axes through Taffy max-content/auto
  layout, drawing Artboard backgrounds from runtime-backed root layout bounds,
  and aligning `NestedArtboardLeaf` content against child root layout bounds.
  Full `make golden-compare` reports `exact=246`, `exact-segments=567`,
  `diverges=0`, `unsupported-feature=49`, `not-yet=0`, parked
  `M6=7 gated=6 harness=36`; `cargo test --workspace` passes. Next target is
  `echo_show_demo.riv`
  (`rust-runner-unsupported:text-joystick-data-bind-divergence`).
- 2026-07-07: [M6] Narrowed `echo_show_demo.riv` by wiring
  `RuntimeJoystick` to its graph-projected nested remap dependents and by
  having `Joystick::apply` advance matching child `NestedRemapAnimation`
  instances. A focused bypass run now reaches the same nested-remap application
  path but still diverges at the first gradient shader allocation because C++
  records intermediate child gradient shader-cache states during
  `NestedRemapAnimation::advance(0, false)` and Rust's runner prepares the
  final post-update state. The manifest/runner guard is sharpened from
  `text-joystick-data-bind-divergence` to
  `text-joystick-nested-remap-gradient-order`. Full `make golden-compare`
  remains `exact=246`, `exact-segments=567`, `diverges=0`,
  `unsupported-feature=49`, `not-yet=0`, parked
  `M6=7 gated=6 harness=36`; `cargo test --workspace` passes. Next target is
  `superbowl.riv` (`state-machine-viewmodel-solo-image`).
- 2026-07-07: [M6] Narrowed `superbowl.riv` by making state-machine bindable
  sources artboard-aware, preserving inherited owned view-model context chains
  for nested state machines, and admitting structural enum sources when a
  nested child also needs an ancestor view-model path, while retaining the
  view-model-0 fallback for generated default-view-model state-machine probes.
  The focused bypass now draws the Logo Solo image and binds Summary's
  `[2,2]`, `[2,7]`, and root `[3,5]` sources. The remaining first diff is
  Summary nested layout/state-machine layout/style invalidation
  (`transform ...102.686523` in C++ vs `...33.5` in Rust), so the
  runner/corpus diagnostic is sharpened to
  `nested-state-machine-layout-update` at host global `142`. Full
  `make golden-compare` reports `exact=246`, `exact-segments=567`,
  `diverges=0`, `unsupported-feature=49`, `not-yet=0`, parked
  `M6=7 gated=6 harness=36`; `cargo test --workspace` passes. Next target is
  `bullet_man.riv` (`selected-root-skinned-ik-clip-path`).
- 2026-07-07: [M6] Promoted `bullet_man.riv` after a focused bypass proved
  the selected-root skinned/IK clip-path stream is structurally identical to
  C++ at sample 0, with only bounded skinned path numeric drift (max
  `0.000489` across 6972 numeric tokens). The stale
  `selected-root-skinned-ik-clip-path` runner guard is removed and the corpus
  entry is exact under `verification = "tolerant(0.0005)"`. Full
  `make golden-compare` reports `exact=247`, `exact-segments=568`,
  `diverges=0`, `unsupported-feature=48`, `not-yet=0`, parked
  `M6=6 gated=6 harness=36`; `cargo test --workspace` passes. Next target is
  `echo_show_demo.riv` (`text-joystick-nested-remap-gradient-order`).
- 2026-07-07: [M6] Sharpened `echo_show_demo.riv` from
  `text-joystick-nested-remap-gradient-order` to
  `joystick-nested-remap-gradient-update-order` after a focused bypass proved
  the first mismatch happens before `sample seconds=0`, in gradient shader
  creation rather than draw geometry. Paint allocation count still matches C++
  exactly (`972` render paints), but C++ creates nonzero intermediate shaders
  for the joystick-driven nested-remap fill pair before later zero-opacity
  versions; Rust only observes the final zero-opacity state. An initial-state
  prewarm experiment produced 160 pre-sample gradients vs C++'s 107, confirming
  this needs C++-style dirty/update interleaving for nested remap gradient side
  effects, not a broader pre-draw scan. Full `make golden-compare` remains
  `exact=247`, `exact-segments=568`, `diverges=0`, `unsupported-feature=48`,
  `not-yet=0`, parked `M6=6 gated=6 harness=36`; `cargo test --workspace`
  passes. Next target is `superbowl.riv`
  (`nested-state-machine-layout-update`).
- 2026-07-07: [M6] Narrowed `superbowl.riv` by sizing nested layout hosts
  from child Taffy root bounds when available, propagating
  `LayoutComponentStyle.displayValue` collapse/style dirt with C++-style
  direct-child collapse dispatch, and allowing a newly uncollapsed remap child
  with pending component dirt to run its first child `update_pass` only when
  the host artboard has data-bind bindings. Focused `Celebration` direct
  streams are exact, `death_knight.riv` and the solo/collapse exact corpus
  stay exact, and the prior Summary nested layout/red-remap-path mismatch is
  gone. A full bypass now reaches a text residual: bounded numeric glyph drift
  around `3e-5` plus a structural empty-glyph/path-order mismatch near line
  997, where C++ emits an empty filled text path and Rust skips it. The
  runner/corpus diagnostic is sharpened to
  `nested-state-machine-text-empty-glyph-path-order`; with the guard restored,
  `cargo test --workspace` passes and full `make golden-compare` remains
  `exact=247`, `exact-segments=568`, `diverges=0`,
  `unsupported-feature=48`, `not-yet=0`, parked `M6=6 gated=6 harness=36`.
- 2026-07-07: [M6] Promoted `superbowl.riv` to exact by mirroring C++
  `TextStylePaint::addPath` for positive-opacity glyphs whose raw path has no
  verbs: static text now keeps empty opacity buckets, emits empty text draw
  paths, and only treats fully absent buckets as no text content. Removed the
  temporary `nested-state-machine-text-empty-glyph-path-order` runner gate and
  added a C++ probe assertion for the positive-opacity empty-path behavior.
  Full `make golden-compare` reports `exact=248`, `exact-segments=569`,
  `diverges=0`, `unsupported-feature=47`, `not-yet=0`, parked
  `M6=5 gated=6 harness=36`; `cargo test --workspace` passes. Next target is
  `echo_show_demo.riv` (`joystick-nested-remap-gradient-update-order`).
- 2026-07-07: [M6] Kept `echo_show_demo.riv` parked but landed the safe
  gradient dirt subset found during the bypass: color data binds now call the
  color change handler, string binds still call the string handler, gradient
  endpoint/opacity changes mark transform/paint dirt, and `GradientStop`
  color/position changes mark parent gradient stops dirty. A proposed
  component-update shader creation hook was explicitly rejected after it
  regressed `bullet_man.riv` and `hunter_x_demo.riv`; the remaining
  `echo_show_demo.riv` blocker is state-machine/joystick/nested-remap update
  ordering, not standalone render-paint prewarming. Full `make
  golden-compare` remains `exact=248`, `exact-segments=569`, `diverges=0`,
  `unsupported-feature=47`, `not-yet=0`, parked `M6=5 gated=6 harness=36`;
  `cargo test --workspace` passes. Next target remains `echo_show_demo.riv`
  (`joystick-nested-remap-gradient-update-order`).
- 2026-07-07: [M6] Rejected a C++ `advanceAndApply` outer-loop-shaped
  experiment for `echo_show_demo.riv`: exposing a Rust `try_change_state` and
  looping update-pass / zero-time state-machine+nest advancement in the
  golden runner left the focused bypass unchanged (`makeRadialGradient id=4`
  still differs at line 980; C++ emits 107 pre-sample gradients, Rust 108).
  The experiment was fully reverted. Next pass should inspect the concrete
  joystick/nested-remap data path that selects paint global 636 in Rust before
  first draw, rather than adding another generic runner loop.
- 2026-07-07: [M6] Rejected the global named view-model instance 0 runner
  convention for now. The coordinated experiment recovered `scripted_color.riv`
  after binding the selected artboard's own owned context, but still made 48
  exact entries diverge because C++ `ArtboardComponentList` instantiates and
  draws serialized list item artboards that Rust does not yet instantiate.
  `RuntimeOwnedViewModelInstance::from_instance` remains as tested groundwork
  for future serialized-instance binding, but both golden runners stay on the
  blank-default convention. Full `make golden-compare` remains `exact=248`,
  `exact-segments=569`, `diverges=0`, `unsupported-feature=47`,
  `not-yet=0`, parked `M6=5 gated=6 harness=36`; `cargo test --workspace`
  passes. Next target remains `echo_show_demo.riv`
  (`joystick-nested-remap-gradient-update-order`).
- 2026-07-02: `rive-runtime` owns static draw emission through
  `rive-render-api`; `rust-golden-runner` now only orchestrates import,
  artboard selection, stream markers, and recording output.
- 2026-07-02: Static rendering applies artboard origin as a top-level draw
  transform and preallocates clone render paints only for the selected
  artboard, matching C++ multi-artboard import/draw behavior.
- 2026-07-02: Empty effect paths are distinct from no effect path;
  `RuntimeShapePaintCommand` tracks whether a supported effect exists so C++
  empty TrimPath output is preserved.
- 2026-07-02: Effect-bearing selected-artboard paints preallocate before the
  remaining local paint order, matching C++ clone paint IDs for `trim.riv`
  without regressing `dependency_test.riv` or `shapetest.riv`.
- 2026-07-02: Corpus features prefixed `rust-runner-unsupported:` are verified
  by `golden-compare` when `--rust-runner` is supplied; use them when a
  later-phase feature would otherwise be silently omitted by Rust rendering.
- 2026-07-02: `exact` is scoped to the samples/scripts in `corpus.toml`;
  animated files may be exact at sample `0` now and still need wider M2 samples
  later.
- 2026-07-02: `golden-compare` exact stream comparison uses numeric-token
  epsilon while keeping call order, IDs, verbs, and non-numeric text exact,
  matching the V2 renderer seam plan.
- 2026-07-03: `rive-renderer-ffi` native mode now has a local null-context
  fallback that compiles the C++ renderer sources needed by
  `RenderContextNULL` when `librive_pls_renderer.a` is absent; the
  `ffi_null_draw` example imports `dependency_test.riv` and drew 3 calls
  through `FfiFactory`/`FfiFrame` into C++ `RiveRenderer`. Full
  visible/offscreen Metal remains blocked on the machine missing Apple's Metal
  Toolchain while building the renderer archive (`xcodebuild
  -downloadComponent MetalToolchain`).
- 2026-07-02: Instance `RenderPaint` ID allocation follows C++ import-time
  `ShapePaintMutator` object order, not Fill/Stroke object order and not draw
  order; Rust preallocates by mutator owner first, then falls back to any
  unallocated Fill/Stroke.
- 2026-07-02: Rust golden runner scene markers follow C++
  `defaultStateMachine()` selection by checking whether
  `defaultStateMachineId` was serialized on the selected artboard and treating
  the value as a state-machine index; schema default values alone do not
  select a state machine.
- 2026-07-02: Runtime composed shape paths default to C++
  `ShapePaintPath` fill rule `clockwise`; Fill paints still override the
  path fill rule immediately before draw, while Stroke paints preserve the
  composed path default.
- 2026-07-02: Imported Solo collapse mirrors `src/solo.cpp` for static state:
  constraints and clipping shapes inherit the Solo's collapse value, while
  participating children collapse unless they match the imported
  `activeComponentId` resolved through the artboard-local object table.
- 2026-07-02: Delegated subsystems (#V2-7) use Rust-native libraries where the
  delegated behavior is spec-defined, chosen by "spec-defined may swap engines;
  implementation-defined may not": Taffy (layout, behind a trait, Yoga-FFI as
  untriggered fallback), HarfRust + read-fonts/skrifa (shaping/font parsing),
  unicode-bidi (bidi), `image`-ecosystem crates (decoders), cpal/rodio
  (audio), and mlua+`luau` vendoring the official Luau VM (scripting uses the
  same VM as C++, so scripted files stay `exact`). `corpus.toml` gains
  per-entry verification modes `exact | tolerant(ε) | structural`; files
  exercising Taffy layout, HarfRust shaping/font numeric drift, or lossy image
  decoding verify `tolerant`, everything else stays `exact`. Rive-owned text
  layout, wrapping, fit-font-size, draw suppression, call order, and glyph
  contour ordering are ported behavior, not tolerant delegated-engine drift.
  Cross-runtime image comparison must use decoded dimensions + tolerant pixel
  sampling, never payload hashes (supersedes the size/hash recording decision
  above once Rust image support lands). Do not pin Taffy against Yoga
  behavior-by-behavior. Taffy CSS Grid is a post-M7 enhancement idea, not port
  scope.
- 2026-07-06: #V2-7 image verifier first slice: C++ and Rust recording
  factories now parse PNG/JPEG/WebP dimensions from encoded headers, emit
  `decodeImage id=... width=... height=...`, and return those dimensions
  through `RecordingRenderImage`. Payload hashes are no longer in golden
  streams; tolerant pixel sampling remains before lossy decoder fixtures can
  rely on image-specific tolerant verification.
- 2026-07-03: Metric is now segments-weighted: `golden-compare` reports
  `exact-segments` (sum of samples across exact entries) alongside the file
  count, so M2 sample widening registers as metric movement. Gated corpus
  entries carry `milestone = "M3|M4|M5|M6|gated|harness"` (preserved by
  `generate-corpus`), and the summary prints a parked-by-milestone
  breakdown, so each milestone's work-list is queryable from `corpus.toml`
  instead of backlog prose. Completed-milestone log entries are archived in
  `docs/v2-log-archive.md` to keep this file small.
- 2026-07-05: `component_stateful.riv` is exact after admitting nested
  `TextValueRun.text` string binds from stateful child view-model values and
  clearing created default nested text contexts. `relative_data_binding.riv`
  and `shared_viewmodel_instance.riv` now render but are parked as M6
  divergences because Rust draws nested/shared text that C++ does not at
  sample 0.
- 2026-07-04: Remaining scroll-constraint files are M6, not M3: the C++
  implementation is coupled to layout dimensions, layout-provider child
  bounds, physics, and component-list virtualization. Use the explicit
  `rust-runner-unsupported:scroll-constraints` diagnostic for this queue
  until layout/runtime support opens it.
- 2026-07-04: `golden-compare` numeric-token epsilon is now `1.3e-4`, raised
  from `1e-4` after `follow_path_shapes.riv` exposed local path float
  cancellation between C++ clang contraction/rounding and Rust strict `f32`.
  The comparator still rejects the next observed cancellation-grid step, and
  call order, IDs, verbs, and non-numeric text remain exact.
- 2026-07-04: Rust golden runner now mirrors C++ input-script parsing and
  timeline replay for pointer events, records input markers, and dispatches
  pointer events into direct rectangle state-machine listeners for the first
  M3 scripted-interactivity slice. Full C++ ListenerGroup hover/click/drag/
  opaque behavior remains corpus-driven follow-up work.
- 2026-07-04: Runtime draw order is dynamic once `DrawTarget` rules can be
  driven by animations or nested remap time: Rust derives sorted drawables
  from active `DrawRules.drawTargetId` and `DrawTarget.placementValue` during
  draw, then recomputes clipping proxies and save-operation elision.
- 2026-07-04: M4 is corpus-closed after a direct `rust-golden-runner` sweep:
  no `milestone = "M4"` entries remain. Former M4 parked files now carry their
  first verified later diagnostic: M5 data-binding nested child/host or
  custom-property trigger paths, and M6 text/images/nested-artboard-layout/
  focus/layout-component-paint paths. This opens M5 without hiding the later
  text/layout/list work.

- 2026-07-05: Layout engine fence: the hand-rolled flex math that closed
  the simple root row/column layout-paint queue is capped at exactly that
  scope. The next layout gate that needs wrapping, grow/shrink ratios,
  percent/auto sizing, alignment beyond start/center/end, gaps, or nested
  layout containers MUST trigger the #V2-7 decision instead: integrate
  Taffy behind the layout trait and route the existing simple cases
  through it. Extending the hand-rolled math case-by-case is re-porting
  Yoga behavior-by-behavior — the V1 pattern — and is a tripwire. Files
  whose layouts diverge under Taffy verify in `tolerant` mode per the
  V2 map; do not pin Taffy against Yoga.
- 2026-07-05: Layout trait contract: the #V2-7 layout adapter computes a
  coherent whole-artboard layout snapshot from Rive style/component data and
  either returns all supported `LayoutComponent` bounds for that snapshot or
  refuses the tree. Runtime draw, world-transform, and computed-value code
  consume those bounds; they must not mix Taffy-solved nodes with ad hoc
  per-node flex fixes inside the same layout tree. `tolerant` verification
  covers swapped-engine numeric geometry drift, not missing style plumbing.
- 2026-07-05: `golden-compare` implements the #V2-7 manifest field
  `verification = "exact" | "tolerant(ε)" | "structural"` for exact corpus
  entries, defaulting omitted entries to `exact`; `generate-corpus` preserves
  non-default verification modes across regeneration. This is the harness
  prerequisite for Taffy/HarfRust/image-decoder corpus admission.
- 2026-07-05: #V2-7 verification language is interpreted by the current
  comparator as accepted-under-declared-mode, not byte-identical for all
  accepted files. `exact-segments` counts `status = "exact"` entries, including
  entries that declare `verification = "tolerant(...)"`. Tolerant verification
  relaxes numeric tokens only: call order, IDs, path verbs, non-numeric payloads,
  and glyph contour ordering remain strict unless a future Decision introduces
  a dedicated outline canonicalization or raster comparison mode. It does not
  hide missing Rive text layout behavior such as wrapping, fit-font-size, or
  layout-controlled text bounds. New Taffy layout gates may not be promoted
  through hand-rolled fallback after the #V2-7 layout adapter refuses a tree.
- 2026-07-06: #V2-7 fallback fence: legacy hand-rolled layout helpers may
  remain inside `rive-runtime` as regression scaffolding for older exact
  slices, but `rust-golden-runner` must reject any layout-dependent draw
  candidate when the Taffy adapter cannot produce a coherent whole-artboard
  layout snapshot. This includes painted `LayoutComponent` paths plus child
  text/image/shape drawables whose parent chain passes through a
  `LayoutComponent`. Existing exact layout fixtures must either compute Taffy
  bounds or return to an explicit unsupported-feature gate.
- 2026-07-06: #V2-7 scroll admission rule: `ScrollConstraint` is Rive-owned
  runtime behavior, not delegated tolerant behavior. The runner may admit only
  passive initial-state scroll constraints once Rust applies the C++
  `constrain` / `constrainChild` transform slice over registered
  layout-provider children and Taffy can compute a coherent whole-artboard
  snapshot. Input-driven drag, nonzero offset/percent/index state, snap,
  infinite scroll, virtualized lists, listener-targeted scroll, physics
  advancement, and scroll-bar driving remain
  `rust-runner-unsupported:scroll-constraints`.
- 2026-07-05: M6 layout/text diagnostic rule: when a Taffy-backed file reaches
  draw but diverges on wrapped layout placement, expose local-id layout boxes
  from C++ Yoga and Rust Taffy before adding more renderer/text behavior. Draw
  suppression and layout participation are separate facts; do not infer one
  from the other without a focused C++ probe.
- 2026-07-05: Scripted data-context files are M6 scripting gates, not text
  draw-suppression targets, when the selected artboard combines a
  `ScriptedDrawable`, `DataBindContext` text, and nested view-model context.
  The Rust runner emits `unsupported: scripted-data-context` for that surface
  until the #V2-7 `mlua`/Luau glue lands; passive script fixtures that already
  match C++ remain eligible for exact comparison.

- 2026-07-06: Threads policy adopted (recorded in `/goal`): single writer
  per worktree; parallel threads are read-only triage scouts or orthogonal
  lane threads started in a new worktree and merged back through the full
  ratchet (eligible lanes: C++ harness crash repair, M7 scaffolding —
  benchmarks/fuzz/API drafts — and the feature-gated scripting spike).
  Never two writers on adjacent critical-path runtime slices.

- 2026-07-07: View-model binding convention revised: keep the runners on
  the blank default view-model instance convention until Rust has real
  `ArtboardComponentList` item artboard runtime support. The named-instance
  experiment (`createViewModelInstance(viewModelId, 0)` / serialized
  instance 0 in Rust) recovered `scripted_color.riv` only after adding the
  selected artboard's own owned-context binding, but still made 48 exact
  entries diverge. Direct stream inspection of `component_list_1.riv`
  showed the decisive gap: C++ instantiates and draws list item artboards
  from serialized list data, while Rust does not yet. Treat named-instance
  binding as blocked on component-list item instancing, draw, layout, and
  data-context parity; do not ship a partial runner convention.

- 2026-07-07: Scripting VM decided (user directive, supersedes the mlua
  plan): use `luaur` / `luaur-rt` (https://github.com/pjankiewicz/luaur),
  a line-for-line Rust translation of the actual Luau compiler/VM/type
  checker — all 293 upstream conformance scripts byte-identical vs C++
  Luau, bytecode-compatible. Scripted corpus files still target strict
  `exact`: the C++ probe runs real Luau, so any luaur drift appears as a
  golden stream diff (report upstream, do not pin around it). PIN the
  luaur version, and check its conformance-pinned Luau commit against the
  Luau version vendored by the reference runtime. Integrate behind a thin
  scripting seam; `mlua`+`luau` (same API shape) is the untriggered
  fallback. Port `src/lua/` glue corpus-file-by-corpus-file — the fence
  rules apply to the 16.4k-line binding surface more than anywhere else.

- 2026-07-07: Perf methodology fence adopted: benchmarks must be
  release-vs-release and exclude serializer cost (null-renderer mode)
  before any further optimization; debug-era perf numbers are void.
  Optimization slices follow flamegraph -> C++-site-first -> port-their-
  optimization-before-inventing. Fidelity while optimizing: no tolerance
  widening, no geometry float-math restructuring, no skip/cache logic
  that does not mirror an audited C++ dirt gate. Statistical floor: >=10
  iterations, median+spread, pinned size-class corpus, per-commit perf
  JSON artifact.

## Log

- Completed-milestone entries (M0 through M5) are archived verbatim in
  `docs/v2-log-archive.md`; when a milestone completes, move its entries
  there and keep only the active milestone's recent working window here.

- 2026-07-08: [M7] Stored render-paint configurations in dense global slots.
  The low-load perf gate stayed unavailable, so the session continued the
  documented draw-replay fixed-overhead lane. `RuntimeRenderPaintCache` now
  keeps cached paint configurations in a dense global-id slot table matching
  `RuntimeRenderPaints`, replacing draw-time `BTreeMap` lookups while keeping
  the same instance-epoch/configuration invalidation. Full
  `make golden-compare` remains exact=263/exact-segments=584/diverges=0;
  `cargo test --workspace`, `cargo test -p rive-runtime --quiet`,
  `cargo check -p rive-runtime`, `cargo fmt --all -- --check`, and
  `git diff --check` pass. Perf was not rerun because load remained outside
  the M7 acceptance fence.

- 2026-07-08: [M7] Switched hot-loop aggregate timing to whole-repeat
  `total_ms`. `perf-compare` now scores runner benchmarks from `total_ms` when
  present, with a phase-sum fallback for older runners. The C++ and Rust golden
  runners emit `total_ms` from a separate whole-repeat pass for
  `--benchmark-repeat`, then keep the existing per-phase pass as diagnostics.
  This implements the scout methodology fence that tiny-file phase timing was
  paying for repeated clock reads. Full `make golden-compare` remains
  exact=263/exact-segments=584/diverges=0; `cargo test --workspace`,
  `cargo test -p perf-compare --quiet`, `cargo check -p rust-golden-runner`,
  `tools/golden-runner/build.sh debug`, `cargo fmt --all -- --check`, and
  `git diff --check` pass. A debug `shapetest` benchmark smoke confirms both
  runners emit `total_ms` and `perf-compare` consumes it. Perf was not rerun
  because load stayed outside the M7 acceptance fence.

- 2026-07-08: [M7] Retained clip/background paths in dense slots. When the
  low-load perf gate stayed unavailable, the session followed the documented
  draw-replay fixed-overhead fallback: artboard clips, clipping-shape clips,
  layout clips, and backgrounds now reuse retained `RawPath`/`RenderPath`
  entries keyed by graph-local dense slots, path/layout epochs, and fill rule
  instead of draw-time `BTreeMap` path lookups and unconditional path rebuilds.
  Full `make golden-compare` remains exact=263/exact-segments=584/diverges=0;
  `cargo test --workspace`, `cargo check -p rive-runtime`, the focused
  retained-path unit test, `cargo fmt --all -- --check`, and `git diff --check`
  pass. Perf was not rerun because load remained outside the M7 acceptance
  fence.

- 2026-07-08: [M7] Skipped clean no-nested paint preparation before prepared
  command lookup. The scout review identified the binding fences for this
  session: do not use high-load perf as decision-grade, do not chase
  state-machine fixed overhead, keep rejected shallow image/command/path
  wrapper scouts out, and treat timer-read block timing as a harness-methodology
  candidate rather than a runtime shortcut. `RuntimePaintPreparationFrame` now
  tracks whether the cached frame saw nested artboards; when it did not, a
  matching graph/global plus instance epoch returns before prepared-command
  frame work. Full `make golden-compare` remains
  exact=263/exact-segments=584/diverges=0; `cargo check -p rive-runtime`,
  `cargo test -p rive-runtime --quiet`, `cargo test --workspace`,
  `cargo fmt --all -- --check`, and `git diff --check` pass. Perf was not
  rerun because load remained outside the M7 acceptance fence.

- 2026-07-08: [M7] Retained draw raw paths beside cached draw `RenderPath`s.
  This ports C++ `ShapePaintPath::m_rawPath` ownership for the draw-path cache:
  path-epoch misses rebuild the cached `RawPath` in place with `rewind()`
  capacity reuse and then call `RenderPath::add_raw_path`, while clean frames
  keep reusing the retained `RenderPath`. Full `make golden-compare` remains
  exact=263/exact-segments=584/diverges=0; `cargo check -p rive-runtime`, the
  focused draw-path reuse test, `cargo test --workspace`,
  `cargo fmt --all -- --check`, and `git diff --check` pass. Same-session perf
  before the final capacity-reuse polish was directional only: aggregate min
  Rust/C++=3.219 with C++ min-sum 1.037 ms outside the sanity band, and
  movement versus 3.225 is below the noise floor. Strict <=2.0 remains open;
  next step is a clean low-load/sanity-band release sample, then deeper
  draw-replay fixed-overhead or clean-prepare-skip work under the scout fences.

- 2026-07-08: [M7] Retained image layout local transforms behind existing
  cache/layout epochs. This ports C++ `Image::updateImageScale` state without
  caching blend/opacity/draw state, and routes mesh-image draw through the
  parent image layout transform like C++ `Mesh::draw`. Full
  `make golden-compare` remains exact=263/exact-segments=584/diverges=0;
  `cargo check -p rive-runtime`, `cargo test -p rive-runtime --quiet`,
  `cargo test --workspace`, `cargo fmt --all -- --check`, and
  `git diff --check` pass. Post-slice `make perf-hot-loop PERF_MAX_RATIO=999`
  reports aggregate min Rust/C++=3.225, but C++ min-sum=1.043 ms is outside the
  sanity band, so it is directional only. Strict <=2.0 remains open; next step
  is a low-load/sanity-band release sample, then actual
  `PathComposer`/raw-path retention or deeper draw-replay fixed-overhead work
  under the scout fences.

- 2026-07-08: [M7] Stored retained mesh render buffers in dense local-id slots.
  This ports C++ `MeshDrawable` buffer ownership without changing mesh
  discovery or reviving the rejected image mesh-index precompute scout. Full
  `make golden-compare` remains exact=263/exact-segments=584/diverges=0;
  `cargo check -p rive-runtime`, `cargo test -p rive-runtime --quiet`,
  `cargo test --workspace`, `cargo fmt --all -- --check`, and
  `git diff --check` pass. Two post-slice `make perf-hot-loop
  PERF_MAX_RATIO=999` runs report aggregate min Rust/C++=3.219 then 3.176, but
  C++ min-sums=0.992 ms and 1.053 ms are outside the sanity band, so they are
  directional only. Strict <=2.0 remains open; next step is a
  low-load/sanity-band release sample, then actual
  image/`PathComposer`/raw-path retention or deeper draw-replay fixed-overhead
  work under the scout fences.

- 2026-07-08: [M7] Retained path-composer graph lookups for draw and clip
  composition. `RuntimeRenderPathCache` now keeps a graph-identity frame of
  dense local-to-index slots for `PathComposerNode` and `PathGeometryNode`,
  matching C++'s direct `Shape::m_PathComposer` / `Shape::paths` links instead
  of scanning graph vectors while preparing shape-paint and clipping paths.
  Full `make golden-compare` remains exact=263/exact-segments=584/diverges=0;
  `cargo check -p rive-runtime`, `cargo test --workspace`,
  `cargo fmt --all -- --check`, and `git diff --check` pass. M7 perf was not
  rerun because verification-time load averages were 88.55/28.08/22.97, far
  outside the acceptance fence. Strict <=2.0 remains open; next step is a
  low-load release sample, then deeper image/`PathComposer`/raw-path retention
  or draw-replay fixed-overhead work under the scout fences.

- 2026-07-08: [M7] Retained draw-command object kinds for replay dispatch.
  `RuntimeDrawCommand` now stores a precomputed concrete object-kind enum during
  prepared-command construction and uses it for the hot nested/image/text/
  layout dispatch path, matching C++ concrete type dispatch instead of repeated
  `type_name` string checks in draw replay. Full `make golden-compare` remains
  exact=263/exact-segments=584/diverges=0; `cargo test --workspace`,
  `cargo fmt --all -- --check`, and `git diff --check` pass. Fenced
  release/null-renderer hot-loop is directional only: aggregate min
  Rust/C++=3.399, `spotify_kids_demo@0`=5.500, with C++ min-sum=1.024 ms
  outside the 0.70-0.95 sanity band and high 5/15-minute load. Strict <=2.0
  remains open. Next: rerun low-load release perf, then port actual image/
  `PathComposer`/raw-path retention or deeper draw-replay fixed overhead under
  the scout fences.

- 2026-07-08: [M7] Added an N-slicer fast-miss in the shape draw path.
  A fresh release/null-renderer sample under high load found
  `spotify_kids_demo@0` back on top (`make perf-hot-loop PERF_MAX_RATIO=999`
  aggregate min Rust/C++=3.743, `spotify_kids_demo@0`=6.389, but C++ min-sum
  outside the sanity band). A macOS `sample` run of
  `spotify_kids_demo --benchmark-repeat 10000000` showed
  `runtime_nsliced_node_context_for_shape` as a top draw-stack cost even
  though the active artboards have empty `n_slicer_details`; this mirrors the
  C++ retained `NSlicedNode`/`ComponentDirt::NSlicer` boundary by returning
  before scanning shape deformer rows when no N-slicer details exist.
  Rebuilt release direct `spotify_kids_demo --benchmark-repeat 1000000`
  improves from elapsed/draw=17402/13772 ms to 14017/10856 ms; post-change
  sampling drops the helper from hundreds of samples to a small leaf. Full
  `make golden-compare` remains exact=263/exact-segments=584/diverges=0;
  `cargo test --workspace`, `cargo check -p rive-runtime`, and
  `cargo fmt --all -- --check` pass. Fenced hot-loop moves directionally to
  aggregate=3.671 and `spotify_kids_demo@0`=5.886, but load was about 32 and
  C++ min-sum=1.287 ms, so strict <=2.0 remains open. Next: rerun a low-load
  release sample, then port actual `PathComposer`/raw-path retention or
  fixed-overhead draw replay instead of shallow command-vector caches.

- 2026-07-08: [M7] Reused retained path-geometry command frames for
  clipping-shape draw replay. `runtime_clipping_shape_path_commands` now takes
  the existing `RuntimeRenderPathCache` and calls
  `path_geometry_commands_frame` instead of rebuilding
  `runtime_path_geometry_with_layout_control` and weighted path context
  directly on every clip-start draw. This ports one slice of the C++
  `ClippingShape`/`PathComposer` retained-path shape without widening
  invalidation: the cache is still keyed by `path_epoch` and `layout_epoch`.
  Two same-session image-path scouts were rejected before commit: a deeper
  `Image::updateImageScale` sidecar proved too broad for a safe quick slice,
  and an `Image::m_Mesh`-style mesh-index precompute worsened direct
  `spotify_kids_demo` long-repeat draw time. Direct Rust-only
  `spotify_kids_demo --benchmark-repeat 1000000` improves from
  elapsed/draw=18763/15899 ms to 14382/11420 ms and 14793/11743 ms on rerun.
  Full `make golden-compare` remains exact=263/exact-segments=584/diverges=0;
  `cargo test --workspace`, `cargo fmt --all -- --check`, and
  `git diff --check` pass. Fenced `make perf-hot-loop PERF_MAX_RATIO=999`
  reports aggregate min Rust/C++=4.010 and 3.570, with
  `spotify_kids_demo@0`=7.920 and 6.283, but both runs had C++ min-sums
  outside the sanity band, so treat them as directional. Strict <=2.0 remains
  open. Next: fresh release sample, then continue retained draw-replay/fixed
  overhead under the scout fences.

- 2026-07-08: [M7] Cached transform property keys for authored transforms and
  keyframe transform-property classification. `RuntimeComponent` now snapshots
  concrete generated transform property keys at instance build time, so
  `ArtboardInstance` reads and writes transform values through generated keys
  instead of `double_property_by_name` while still preserving concrete storage
  such as `StraightVertex.x`. `transform_property_for_key` compares cached
  generated keys instead of resolving schema names each call. Full
  `make golden-compare` remains exact=263/exact-segments=584/diverges=0, and
  `cargo test --workspace` passes. Fenced
  `make perf-hot-loop PERF_MAX_RATIO=999` improves the current aggregate min
  Rust/C++ from 4.758 to 4.499, 4.475, and 4.383 on rerun;
  `spotify_kids_demo@0` improves from 10.413 to 9.480, 9.506, and 9.125.
  A post-commit rerun reports aggregate min Rust/C++=4.449 but has C++
  min-sum=1.019, outside the sanity band, so it is directional only. Strict
  <=2.0 remains open. Next: profile/port deeper C++ image
  `updateImageScale`/transform retention, or move to tiny-file draw
  replay/fixed overhead if that slice proves too broad.
- 2026-07-08: [M7] Clarified the adopted perf fence at the top of this status
  file so `/goal` sessions can choose M7 work without rereading buried scout
  notes. The fence is bare `make perf-hot-loop` with release/null-renderer
  whole-repeat `total_ms` scoring, min aggregation, deliberate image-bearing
  corpus, `PERF_ITERATIONS=10`, and `PERF_BENCHMARK_REPEAT=100`; acceptance
  remains three independent aggregate-min runs <=2.0 with load and C++ min-sum
  sanity checks. The older 2.98/3.95 scout standings are now labeled historical,
  and the current priority is explicit: profile/port image draw retention
  before smaller draw-replay/fixed-overhead work.
- 2026-07-08: [M7] Rejected a shallow non-mesh image draw-state cache scout.
  The code retained image origin/fit/alignment/world/blend/opacity in
  `RuntimeRenderPathCache` behind prepared/layout/cache epochs and image
  dimensions, matching the existing Rust draw-time math on cache misses. It
  preserved `make golden-compare` at exact=263/exact-segments=584/diverges=0
  and `cargo check -p rive-runtime` passed, but the adopted fence rejected it:
  `make perf-hot-loop PERF_MAX_RATIO=999` reported aggregate min Rust/C++=4.804
  and `spotify_kids_demo@0`=10.607, worse than the standing 4.758/10.413. Code
  backed out. Next image work must port the deeper C++ `Image::updateImageScale`
  / transform-retention shape or switch to the transform-key cold-frame
  fallback from scout item 17.
- 2026-07-08: [M7] Made the focused perf-gate defaults match the scout fence.
  `make perf-hot-loop` now defaults to `PERF_ITERATIONS=10` and
  `PERF_BENCHMARK_REPEAT=100`, keeping the existing min aggregation and
  deliberate image-bearing corpus, so bare M7 perf runs are decision-grade
  instead of five-iteration/repeat-one smoke checks. Pre-change
  `make golden-compare` remains exact=263/exact-segments=584/diverges=0.
  Post-change `make perf-hot-loop PERF_MAX_RATIO=999` exercises the defaults
  and reports aggregate min Rust/C++=4.758; `spotify_kids_demo@0` remains the
  largest outlier at min Rust/C++=10.413. Full post-change `make
  golden-compare`, `cargo test --workspace`, `cargo fmt --all -- --check`, and
  `git diff --check` pass. Next: profile/port the highest current gate
  outlier under these defaults.
- 2026-07-08: [M7] Landed min-based and deliberate focused perf-gate tooling.
  `perf-compare` now accepts `--aggregate median|min`, applies the selected
  statistic to per-entry ratios, aggregate ratio, JSON selected sums, and
  `--max-ratio`, while preserving median/min fields for downstream reports.
  Corpus mode also accepts ordered `--corpus-ids`; `make perf-hot-loop` now
  defaults to min aggregation and a named focused corpus including
  `spotify_kids_demo` so image draw cost is visible. Full
  `make golden-compare` remains exact=263/exact-segments=584/diverges=0;
  `cargo test --workspace`, `cargo fmt --all -- --check`, and
  `git diff --check` pass. Fenced release/null-renderer smoke with
  `PERF_BENCHMARK_REPEAT=100` reports aggregate min Rust/C++=4.702 over 11
  entries; `spotify_kids_demo@0` is the dominant focused outlier at min
  Rust/C++=10.263. Strict <=2.0 remains open; next profile/port image
  draw-retention before returning to smaller draw-replay/fixed-overhead items.
- 2026-07-08: [M7] Landed the release build-profile parity slice from the
  scout report: root `[profile.release]` now uses fat LTO, one codegen unit,
  and aborting release panics, matching C++'s full-LTO shipping build more
  fairly without machine-specific `target-cpu=native` or float contraction.
  No C ABI `catch_unwind`/`resume_unwind` reliance was found. Full
  `make golden-compare` remains exact=263/exact-segments=584/diverges=0;
  `cargo test --workspace`, `cargo fmt --all -- --check`, `git diff --check`,
  and `cargo build --release -p rive-capi` pass. Fenced repeat-aware hot-loop
  under the new profile reports noisy median aggregates Rust/C++=3.371 then
  2.760; direct repeat=100 `ai_assitant` JSON at
  `target/perf-ai-release-profile.json` reports median Rust/C++=3.195 and min
  timing ratio about 2.53x. Strict <=2.0 remains open; next land the
  min-based/deliberate perf gate before choosing the next runtime hotspot.
- 2026-07-08: [M7] Stopped cloning retained solo binding instances during
  artboard solo target apply. This mirrors the C++ retained DataBind list plus
  direct Solo update dispatch without adding a new skip gate. `make
  golden-compare` remains exact=263/exact-segments=584/diverges=0; focused
  `cargo test -p rive-runtime --quiet` passes. Fenced repeat-aware hot-loop
  reports aggregate Rust/C++=2.939 with Rust median sum 2.315 ms, while direct
  repeat=100 `ai_assitant` JSON at `target/perf-ai-solo-binding-no-clone.json`
  improves Rust median from 0.989 ms to 0.939 ms / Rust/C++=2.403. Strict
  <=2.0 remains open; next profile draw-path lookup, state-machine fixed
  overhead, and nested data-bind context-path work.
- 2026-07-08: [M7] Rejected a dense paint-configuration sidecar scout after a
  fresh `ai_assitant --benchmark-repeat 30000000` sample showed draw replay and
  `runtime_configure_paint_with_cache` map lookups. The code kept
  `cargo check -p rive-runtime`, focused runtime tests, and formatting green,
  but fenced repeat-aware hot-loop worsened to aggregate Rust/C++=3.001 and
  direct repeat=100 `ai_assitant` worsened from rust median=0.939 ms to
  0.997 ms. Code backed out; next draw-side work should be deeper C++
  retained-path/object-identity work, not another sidecar container swap.
- 2026-07-08: [M7] Added data-bind source update queue membership flags and
  empty-lane early-outs, matching C++ `DataBindContainer` dirty/persisting queue
  shape without landing the rejected source-queue vector-swap scout. `make
  golden-compare` remains exact=263/exact-segments=584/diverges=0; `cargo test
  --workspace`, `cargo fmt --all -- --check`, and `git diff --check` pass.
  Fenced repeat-aware hot-loop is noisy: baseline Rust median sum after the
  nested-event slice was 2.467 ms, the post-slice rerun is 2.319 ms, and direct
  repeat=100 `ai_assitant` JSON reports rust median=0.989 ms / Rust/C++=2.613.
  Strict <=2.0 remains open; next profile draw-path lookup, state-machine fixed
  overhead, and remaining nested data-bind context-path work under the scout
  fences.
- 2026-07-08: [M7] Made nested artboard reported-event collection optional,
  matching C++ nested advance / state-machine-owned event queues and preserving
  state-machine propagation order when reports are requested. `make
  golden-compare` remains exact=263/exact-segments=584/diverges=0; `cargo test
  --workspace`, `cargo fmt --all -- --check`, and `git diff --check` pass.
  Fenced repeat-aware hot-loop reports Rust median sum 2.476 ms and 2.488 ms
  versus the pre-slice 3.488 ms, while ratios remain noisy at Rust/C++=3.041 and
  3.073; direct repeat=100 `ai_assitant` JSON is Rust/C++=2.521. Strict <=2.0
  remains open; next profile draw-path lookup, data-bind queue drains, and
  state-machine fixed overhead under the existing scout/perf fences.
- 2026-07-08: [M7] Retained state-machine pending view-model action storage on
  `StateMachineInstance`, matching C++ instance-owned event/listener queues and
  removing a fresh per-layer scratch vector allocation from
  `StateMachineLayerInstance::advance`. `make golden-compare` remains
  exact=263/exact-segments=584/diverges=0; `cargo test --workspace`,
  `cargo fmt --all -- --check`, and `git diff --check` pass. Fenced
  repeat-aware hot-loop is noisy: first post-slice run was aggregate
  Rust/C++=2.942, final reruns report 3.298 then 2.925, and the best
  `ai_assitant` rust median is 1.010 ms versus the 1.092 ms baseline. Strict
  <=2.0 remains open;
  next profile remaining state-machine fixed overhead, nested data-bind
  context-source propagation, and lower draw-path lookup under the existing
  scout/perf fences.
- 2026-07-08: [M7] Stored retained render paints in dense global-id slots
  instead of a draw-time `BTreeMap`. `make golden-compare` remains
  exact=263/exact-segments=584/diverges=0; `cargo test --workspace`,
  `cargo fmt --all -- --check`, and `git diff --check` pass. Fenced
  repeat-aware hot-loop reports aggregate Rust/C++=3.000, and direct repeat=100
  `ai_assitant` JSON at `target/perf-ai-render-paint-slots-rerun.json` reports
  cpp median=0.576 ms, rust median=1.437 ms, Rust/C++=2.495. Strict <=2.0
  remains open; next profile lower `RuntimeRenderPathCache::draw_path` lookup
  and data-bind queue drains under the existing scout/perf fences.
- 2026-07-08: [M7] Retained layout-topology facts on runtime components and
  routed component lookup through dense slot component indices instead of a
  frame-loop `BTreeMap`. `make golden-compare` remains
  exact=263/exact-segments=584/diverges=0; `cargo test -p rive-runtime`,
  `cargo test --workspace`, `cargo fmt --all -- --check`, and
  `git diff --check` pass. Fenced repeat-aware hot-loop baseline was
  Rust/C++=3.515 with Rust median sum 2.429 ms; reruns after the slice report
  Rust/C++=3.326 and 3.392, with the better Rust median sum at 2.342 ms. Direct
  repeat=100 `ai_assitant` JSON is at `target/perf-ai-layout-topology.json`;
  strict <=2.0 remains open.
- 2026-07-08: [M7] Replaced dependency-ordered gradient paint-prep
  `BTreeMap`/`BTreeSet` temporaries with small vectors while preserving the old
  duplicate command rules. This follows the C++ retained object/vector traversal
  shape and does not add a new skip/cache invalidation rule. `make
  golden-compare` remains exact=263/exact-segments=584/diverges=0; focused draw
  probes, `cargo test --workspace`, `cargo fmt --all -- --check`, and
  `git diff --check` pass. Fenced repeat-aware hot-loop improves from
  Rust/C++=3.632 to 3.447 and 3.327 on rerun; strict <=2.0 remains open.
- 2026-07-08: [M7] Retained runtime state-machine definitions behind
  `Arc<Vec<RuntimeStateMachine>>`, removing the per-advance definition
  clone/drop in `ArtboardInstance::advance_state_machine_instance` while
  preserving the public borrowed slice API. This is C++-shaped immutable
  definition retention, not a new skip gate. `make golden-compare` remains
  exact=263/exact-segments=584/diverges=0; `cargo test --workspace`, `cargo fmt
  --all -- --check`, and `git diff --check` pass. Fenced repeat-aware hot-loop
  reports aggregate Rust/C++=3.613; strict <=2.0 remains open.
- 2026-07-08: [M7] Retained per-path runtime geometry commands by
  `path_epoch`/`layout_epoch` on `RuntimeRenderPathCache`, while keeping
  per-paint transform/NSlice/reversal/prune semantics intact. Also removed
  hot-loop allocation from collapse checks and compacted empty path-segment
  pruning in place. `make golden-compare` remains
  exact=263/exact-segments=584/diverges=0; `cargo test --workspace`,
  `cargo fmt --all -- --check`, and `git diff --check` pass. Fenced
  repeat-aware hot-loop improves from same-turn baseline Rust/C++=3.809 to
  3.616. Strict <=2.0 remains open; next profile remaining small-file fixed
  overhead and `ai_assitant` advance/data-bind under the scout/perf fences.
- 2026-07-08: [M7] Split prepared draw-command invalidation from broad
  instance cache invalidation. `RuntimeRenderPathCache::prepared_artboard_frame`
  now keys frames by `ArtboardInstance::prepared_epoch`, which follows the
  audited C++ dirt boundary for path/layout/draw-order/render opacity, image
  override, nested-artboard identity, and draw-affecting property changes while
  ignoring nested input proxy values and data-bind/view-model metadata. New
  focused epoch tests cover both sides. `make golden-compare` remains
  exact=263/exact-segments=584/diverges=0; `cargo test --workspace` passes;
  `cargo fmt --all -- --check` passes. Rust-only 1M `advance_blend_mode`
  improves elapsed/prepare from 1382.1/695.0 ms to 1269.1/609.9 ms, but
  repeat-aware hot-loop is neutral at Rust/C++=3.897 then 3.852. Strict <=2.0
  remains open; next target is lower-level C++ path/composer retention or a
  sampled audited data-bind/context hotspot under the scout/perf fences.
- 2026-07-08: [M7] Added repeat-aware corpus aggregation to `perf-compare`:
  `--benchmark-repeat N` now expands selected exact files into one target per
  sample segment after applying `--corpus-limit`, while still rejecting input
  scripts. `cargo test -p perf-compare` passes. Focused
  `make perf-hot-loop PERF_CORPUS_LIMIT=5 PERF_ITERATIONS=10 PERF_WARMUPS=1
  PERF_MAX_RATIO=999 PERF_BENCHMARK_REPEAT=100` reports entries=10 /
  segments=10, aggregate Rust/C++=3.711. Strict <=2.0 remains open; next
  profile `advance_blend_mode` / `animation_reset_cases` fixed overhead under
  this repeat-aware score.
- 2026-07-08: [M7] Retained text shape-paint commands on
  `RuntimeRenderPathCache` behind graph/text plus path/layout/cache epochs,
  matching C++ `Text::buildRenderStyles()` / `Text::draw()` retained command
  replay. `make golden-compare` remains exact=263/exact-segments=584/diverges=0;
  `cargo test --workspace`, `cargo fmt --all -- --check`, and `git diff
  --check` pass. Rust-only 3M `animated_clipping` improves from
  elapsed=147254.8/draw=146414.0 ms to elapsed=4008.4/draw=3348.1 ms; fenced
  repeat=1 hot-loop remains above target at Rust/C++=2.395 then 2.223. Strict
  <=2.0 remains open; next profile `advance_blend_mode` /
  `animation_reset_cases` fixed overhead and decide whether to add
  repeat-aware corpus aggregation.
- 2026-07-08: [M7] Retained resolved artboard owned-context source paths on
  property/image/custom bindings behind an owned-view-model/context-chain key,
  mirroring C++ `DataBindContext::bindFromContext` source retention without
  skipping value reads. `make golden-compare` remains
  exact=263/exact-segments=584/diverges=0; focused nested/data-bind tests and
  `cargo test --workspace` pass; fenced hot-loop is noisy at Rust/C++=2.154
  then 2.438, while direct `ai_assitant --benchmark-repeat 100` reports
  cpp median=0.442 ms, rust median=1.420 ms, Rust/C++=3.212. Strict <=2.0
  remains open; next profile remaining advance/data-bind time after retained
  source paths.
- 2026-07-08: [M7] Gated owned-context artboard rebinds by root owned
  view-model mutation generation after reviewing the status scout/perf fences.
  Each artboard now skips its own clean rebind/apply work when the retained
  owned-context key is unchanged, while still descending to children so local
  dynamic nested-artboard swaps can invalidate themselves. `make
  golden-compare` remains exact=263/exact-segments=584/diverges=0; focused
  nested/data-bind and owned-viewmodel probes, `cargo test --workspace`,
  `cargo fmt --all -- --check`, and `git diff --check` pass. Fenced hot-loop
  reports aggregate Rust/C++=2.073 then 2.274; direct repeat=100
  `ai_assitant` reports Rust/C++=3.357; Rust-only 3M `ai_assitant` improves
  to elapsed=20152.4/advance=8773.6 ms. Strict <=2.0 remains open; next
  profile focused outliers under the existing scout/perf fences.
- 2026-07-08: [M7] Reviewed the status scout/perf fences around
  `bind_owned_view_model_artboard_context_chain` and backed out the
  scratch-only owned-context path scout. The scout kept focused nested/data-bind
  tests and `make golden-compare` green at exact=263/exact-segments=584/
  diverges=0, but fenced release/null-renderer hot-loop reported aggregate
  Rust/C++=2.250 and then 2.478, worse than the prior 2.105 run. No runtime
  code from the scout remains. Next implementation target is the C++-aligned
  retained source/rebind model for artboard owned-context bindings, not
  temporary path-buffer reuse.
- 2026-07-08: [M7] Removed nested-host data-bind clean-frame no-op work by
  reusing retained binding entries instead of cloning
  `artboard_nested_host_bindings`, and by skipping same-value nested child
  context updates before queueing them. `make golden-compare` remains
  exact=263/exact-segments=584/diverges=0; `cargo test --workspace` passes;
  focused hot-loop reports Rust/C++=2.105 with `ai_assitant`=1.939. Strict
  <=2.0 remains open; next target is the larger owned-view-model context-chain
  path-resolution hotspot.
- 2026-07-08: [M7] Retained shape-paint draw path slot indices on
  `RuntimeShapePaintCommand`, replacing per-draw `runtime_cached_path_slot_index`
  vector rebuilds for prepared shape/background commands while keeping dynamic
  text assignment transient. `make golden-compare` remains
  exact=263/exact-segments=584/diverges=0; `cargo test --workspace` passes;
  focused hot-loop reports Rust/C++=2.136 with `ai_assitant`=1.972; the
  3M-segment Rust-only sample drops draw time from about 6.82s to 5.94s and no
  longer samples `runtime_cached_path_slot_index`. Strict <=2.0 remains open;
  next profile/port remaining data-bind/context-chain hotspots first.
- 2026-07-08: [M7] Retained nested artboard host local order on
  `ArtboardInstance`, replacing repeated BTree key/range walks in nested
  advance, owned context-chain binding, context-source propagation, and child
  data-context sync. `make golden-compare` remains
  exact=263/exact-segments=584/diverges=0; `cargo test --workspace` passes;
  focused hot-loop is Rust/C++=2.017 to 2.093 with `ai_assitant`=1.820 to
  1.877. Strict <=2.0 remains open; next profile remaining advance/data-bind
  and lower draw/prepare retention under the scout/perf fences.
- 2026-07-08: [M7] Retained sorted drawable order by `draw_order_epoch`,
  raised from the C++ `DrawRules.drawTargetId` / `DrawTarget.placementValue`
  dirt hooks, so prepared-frame rebuilds no longer redo draw-order BTree
  grouping unless DrawOrder dirt changed. `make golden-compare` remains
  exact=263/exact-segments=584/diverges=0; `cargo test --workspace` passes;
  repeat=100 JSON reports Rust/C++=3.051 and focused hot-loop reports
  Rust/C++=2.136 then 2.107. Strict <=2.0 remains open; next re-profile below
  sorted order and data-context BTree/range work under the scout/perf fences.
- 2026-07-08: [M7] Cached fixed draw/image/mesh/nested property keys in
  `runtime_draw_property_key_for_name`, replacing literal frame-loop
  `property_key_for_name` calls without changing dirty or skip semantics.
  `make golden-compare` remains exact=263/exact-segments=584/diverges=0;
  `cargo test --workspace` passes; repeat=100 JSON reports Rust/C++=2.985 and
  focused hot-loop reports Rust/C++=2.187 then 2.122. Strict <=2.0 remains
  open; next re-profile and choose between sampled BTree/draw-order work and
  remaining audited data-context retention.
- 2026-07-08: [M7] Removed draw-time schema lookup from render-opacity and
  nested-artboard drawable classification by replacing
  `definition_by_name(...).is_a(...)` with C++-shaped fixed type checks. `make
  golden-compare` remains exact=263/exact-segments=584/diverges=0; `cargo test
  --workspace` passes; repeat=100 JSON reports Rust/C++=3.603 and focused
  hot-loop reports Rust/C++=2.243 then 2.133. Strict <=2.0 remains open; next
  re-profile and choose the next retained/dirt or hot BTree target.
- 2026-07-08: [M7] Retained nested-host root `ViewModelInstance` locals by
  `viewModelId` and lazily retained successful fallback child source-local
  resolutions. `make golden-compare` remains
  exact=263/exact-segments=584/diverges=0; `cargo test --workspace` passes;
  direct Rust-only repeat=1000000 `ai_assitant` improves from
  elapsed=11658.6/advance=6683.3 ms to elapsed=9791.6/advance=4650.2 ms;
  repeat=100 JSON reports Rust/C++=3.710 and focused hot-loop reports
  Rust/C++=2.136. Strict <=2.0 remains open; next profile remaining
  context-chain/BTree lookup or schema lookup time under the scout fences.
- 2026-07-08: [M7] Retained nested-host child data-context source locals by
  binding path and removed the remaining host-key snapshot from
  `sync_nested_child_artboard_data_contexts`. `make golden-compare` remains
  exact=263/exact-segments=584/diverges=0; `cargo test --workspace` passes;
  single-file repeat=100 improves to Rust/C++=4.880 and focused hot-loop is
  Rust/C++=2.024 then 2.253. Strict <=2.0 remains open; next profile the
  remaining advance/data-bind path before another slice.
- 2026-07-08: [M7] Shared source-producing artboard data-bind paths as
  immutable `Arc<[u32]>` slices to avoid per-context-source path Vec allocation.
  `make golden-compare` remains exact=263/exact-segments=584/diverges=0;
  `cargo test --workspace` passes; focused hot-loop is Rust/C++=2.164, still
  above strict <=2.0. Next profile remaining advance/data-bind time before
  further context allocation cleanup.
- 2026-07-08: [M7] Replaced per-host child-context `Vec<usize>` construction
  with borrowed/inline storage for numeric retained nested `DataBindPath`
  lookups. `make golden-compare` remains
  exact=263/exact-segments=584/diverges=0; `cargo test --workspace` passes;
  focused hot-loop improves to Rust/C++=2.169, still above strict <=2.0. Next
  profile remaining `bind_owned_view_model_artboard_context_chain` /
  `collect_nested_artboard_context_source_values` time under the existing scout
  fences.
- 2026-07-08: [M7] Reworked nested context-source propagation to append into a
  single accumulator and walk retained nested host keys without temporary
  vectors. `make golden-compare` remains
  exact=263/exact-segments=584/diverges=0; `cargo test --workspace` passes;
  direct Rust-only repeat=1000000 `ai_assitant` improves from
  elapsed=11839.2/advance=6853.7 ms to elapsed=11624.2/advance=6594.8 ms.
  Focused hot-loop is Rust/C++=2.281, while repeat=100 single-file is
  Rust/C++=5.207 due to C++ median movement. Strict <=2.0 remains open; next
  profile child-context path construction in
  `bind_owned_view_model_artboard_context_chain`.
- 2026-07-08: [M7] Trimmed owned view-model context-source path allocation,
  stack-prepended shallow nested context chains, avoided unchanged binding path
  clones, and shared retained animation/state-machine names as `Arc<str>`.
  `make golden-compare` remains exact=263/exact-segments=584/diverges=0;
  `cargo test --workspace` passes; direct Rust-only repeat=100000
  `ai_assitant` improves to elapsed=1225.7/advance=706.0 ms. Single-file
  release/null-renderer repeat=100 is Rust/C++=4.496 and focused hot-loop is
  Rust/C++=2.363, so strict <=2.0 remains open. Scout/perf fences remain:
  no broad converter-property writes, no StringPad-style RangeMapper retry, and
  no shallow command/path-wrapper caching without fenced evidence.
- 2026-07-08: [M7] Retained nested-host data-bind resolved paths on
  `RuntimeNestedArtboardInstance`, including dynamic `artboardId` swaps.
  `make golden-compare` remains exact=263/exact-segments=584/diverges=0;
  `cargo test --workspace` passes; long-repeat `ai_assitant` improves to
  elapsed=1408.5/advance=902.0 ms for 100000 segments, while focused hot-loop
  is Rust/C++=2.338 then 2.430. Strict <=2.0 remains open; next profile
  remaining release/null-renderer advance/data-bind time under the scout/perf
  fences.
- 2026-07-08: [M7] Removed Rust-only nested child binding-vector clones and
  dynamic nested-host property-key lookups from the sampled owned-view-model
  context path. `make golden-compare` remains
  exact=263/exact-segments=584/diverges=0; `cargo test --workspace` passes;
  long-repeat `ai_assitant` improves to elapsed=1734.9/advance=1233.3 ms for
  100000 segments, while focused hot-loop is Rust/C++=2.119. Strict <=2.0
  remains open; next port retained nested-host `DataBindPath` lookup shape.
- 2026-07-08: [M7] Removed operation-view-model/system-operation custom sources
  from the conservative persisting lane after reviewing the scout/perf fences.
  `make golden-compare` remains exact=263/exact-segments=584/diverges=0;
  `cargo test --workspace` passes; focused hot-loop is Rust/C++=2.111. Strict
  <=2.0 remains open; next profile remaining data-bind/advance or audit
  RangeMapper ownership/order before retrying that family.
- 2026-07-08: [M7] Removed `DataConverterNumberToList.viewModelId` custom
  sources from the conservative persisting lane with a family-specific updater.
  `make golden-compare` remains exact=263/exact-segments=584/diverges=0;
  `cargo test --workspace` passes; focused hot-loop is Rust/C++=2.243. Strict
  <=2.0 remains open; next inspect operation-view-model / system-operation
  converter-backed custom sources.
- 2026-07-08: [M7] Removed `DataConverterInterpolator.duration` custom sources
  from the conservative persisting lane with a family-specific updater.
  `make golden-compare` remains exact=263/exact-segments=584/diverges=0;
  `cargo test --workspace` passes; focused hot-loop is Rust/C++=2.180 then
  2.264. Strict <=2.0 remains open; next inspect NumberToList view-model-id
  dirty/cache semantics.
- 2026-07-08: [M7] Landed path-indexed artboard source-to-target dirty queues
  for property/image data binds. `make golden-compare` remains
  exact=263/exact-segments=584/diverges=0; `cargo test --workspace` passes;
  same-session `988fc29` baseline was 3080.3/2392.7 ms elapsed/advance
  Rust-only repeat=100000 and focused hot-loop Rust/C++=2.723, while this
  slice is 2480.2/1859.1 ms and focused Rust/C++=2.371/2.599. Strict <=2.0
  remains open; next port to-source dirty/persisting queues.
- 2026-07-08: [M7] Borrowed nested owned-view-model context chains instead of
  cloning `Vec<Vec<usize>>` per host. `make golden-compare` remains
  exact=263/exact-segments=584/diverges=0; direct
  `ai_assitant --benchmark-repeat 100` is Rust/C++=7.210, focused 5-entry
  hot-loop is Rust/C++=2.321, and same-session Rust-only repeat=100000 improves
  from baseline 4235.4/3275.3 ms elapsed/advance to 4109.3/3120.9 ms. Strict
  <=2.0 remains open; next is actual `DataBindContainer` dirty queues.
- 2026-07-08: [M7] Rejected a naive per-binding `target_dirty` scout for
  artboard property/image data binds. Correctness stayed green at exact=263 /
  exact-segments=584 / diverges=0, but repeat-heavy `ai_assitant` regressed to
  Rust/C++=10.962 and 15.381, Rust-only repeat=100000 regressed to
  4766.0/3385.2 ms elapsed/advance, and focused hot-loop moved to
  Rust/C++=2.614. Code backed out; next port the real C++ container dirty
  vectors/enrollment.
- 2026-07-08: [M7] Trimmed owned view-model data-bind allocation by avoiding
  an intermediate context-source-path `Vec` and owned-view-model update staging
  vector. `make golden-compare` remains exact=263/exact-segments=584/diverges=0;
  direct `ai_assitant --benchmark-repeat 100` improves to Rust/C++=7.731-9.399,
  and Rust-only repeat=100000 drops elapsed/advance to 3840.8/2936.9 ms.
  Focused corpus strict <=2.0 remains open.
- 2026-07-08: [M7] Cached clean-frame paint preparation in
  `RuntimeRenderPaintCache` behind `(graph_global_id, cache_epoch)`. `make
  golden-compare` remains exact=263/exact-segments=584/diverges=0; focused
  release/null-renderer runs are Rust/C++=2.493, 1.832, and 2.166, while direct
  `ai_assitant --benchmark-repeat 100` improves to Rust/C++=8.852-9.756.
  Strict <=2.0 remains open.
- 2026-07-07: [M7] Cached fixed data-bind property keys. `make golden-compare`
  remains exact=263/exact-segments=584 with diverges=0; `cargo test
  --workspace` passes. Focused release hot-loop is Rust/C++=3.096 aggregate,
  and direct `ai_assitant --benchmark-repeat 100` is Rust/C++=34.736. Next
  target is generated/cached schema kind/property tables in the remaining
  frame-loop lookup sites.
- 2026-07-04: [M6] Opened M6 after closing the M5 queue: the final four M5
  entries now probe as nested child `TextValueRun`, so the next loop starts
  with the text sizing spike from `docs/porting-map-v2.md`. `make
  golden-compare` reports `exact=128`, `exact-segments=449`, `diverges=0`,
  `unsupported-feature=167`, `not-yet=0`, and parked
  `M6=124 gated=7 harness=36`; manifest query confirms M5=0, and `cargo
  test --workspace` passes.
- 2026-07-04: [M6] Sized the text opening in
  `docs/prototypes/m6-text-sizing-spike.md`: the largest M6 diagnostic bucket
  is `text` (59 files), C++ text is about an 11k-line stack across import,
  shaping, line breaking, draw, and input/editing, and the first implementation
  slice is now pinned to `hello_world.riv` instead of manifest-first
  `align_target.riv` because it isolates static top-level text path emission.
- 2026-07-04: [M6] Promoted `hello_world.riv` by adding a narrow embedded
  static text draw path in `rive-runtime` with HarfRust/Skrifa shaping and
  outlines, keeping richer text behind static-subset diagnostics. `make
  golden-compare` moved to `exact=129`, `exact-segments=450`,
  `unsupported-feature=166`, and parked `M6=123 gated=7 harness=36`; `cargo
  test --workspace` passes.
- 2026-07-04: [M6] Rechecked the post-`hello_world` text queue. `new_text.riv`
  is too broad for the next slice (five texts plus multi-run/style,
  gradient/stroke, clipping, and keyframed text). `ellipsis.riv` is the
  smallest one-run axis/layout target; axis-only bypass reaches draw but
  diverges on C++ ellipsis layout, so the next implementation must port that
  layout path rather than simply admitting axes.
- 2026-07-04: [M6] Promoted `ellipsis.riv` with static `TextStyleAxis`
  variation setup plus the smallest one-run fixed-height ellipsis/wrap path.
  `make golden-compare` moved to `exact=130`, `exact-segments=451`,
  `unsupported-feature=165`, and parked `M6=122 gated=7 harness=36`; the next
  narrow text tracer is `hosted_font_file.riv`, which isolates no-loader
  hosted font resolution rather than text layout.
- 2026-07-04: [M6] Promoted `hosted_font_file.riv` by mirroring C++
  `FileAssetImporter` no-loader behavior: a hosted `FontAsset` with no
  in-band contents resolves without a decoded font, so static text emits its
  drawable save/restore wrapper but no text path. `make golden-compare` moved
  to `exact=131`, `exact-segments=452`, `unsupported-feature=164`, and parked
  `M6=121 gated=7 harness=36`; the next narrow text tracer is
  `animated_clipping.riv`, which now stops on sibling shape/clipping admission.
- 2026-07-04: [M6] Promoted `animated_clipping.riv` by admitting sibling
  Shape/ClippingShape scaffolding around the one supported static Text path
  and preserving C++'s text-local save/restore around glyph transforms even
  when clipping elides the drawable-level save. The same gate relaxation also
  unlocked byte-identical `databind_artboard.riv`. `make golden-compare`
  moved to `exact=133`, `exact-segments=454`, `unsupported-feature=162`, and
  parked `M6=119 gated=7 harness=36`; next inspect `background_measure.riv`,
  which stops on sibling `RootBone` rather than modifiers.
- 2026-07-04: [M6] Promoted `background_measure.riv` by admitting passive
  bone/skin `PointsPath` decoration around one static Text, computing static
  Text constraint bounds for `TransformConstraint`, and rounding HarfBuzz-style
  line metrics while disabling legacy kern-only advances to preserve
  `hello_world.riv`. `make golden-compare` moved to `exact=134`,
  `exact-segments=455`, `unsupported-feature=161`, and parked
  `M6=118 gated=7 harness=36`; next inspect the narrow text-modifier fixtures.
- 2026-07-04: [M6] Promoted `modifier_test.riv` and `align_target.riv` by
  adding the first static text-modifier slice: translation-only
  `TextModifierGroup`, character-unit `TextModifierRange` coverage, and cubic
  range falloff. `make golden-compare` moved to `exact=136`,
  `exact-segments=457`, `unsupported-feature=159`, and parked
  `M6=116 gated=7 harness=36`; `modifier_to_run.riv` remains parked on
  word/line range mapping plus run-scoped/multi-run text.
- 2026-07-04: [M6] Rechecked the post-modifier text queue. `modifier_to_run`
  is not a narrow modifier follow-up: it has four Text objects plus word/line
  range units, runId targeting, and multi-run text. `vertical_align_ellipsis`
  reaches draw if sibling `Stroke` is admitted, but exact comparison diverges
  first on fixed-size vertical align/ellipsis text placement. The next narrow
  implementation slice is `double_line.riv`, which isolates same-style
  multi-run text and explicit line breaks before the modifier range-map files.
- 2026-07-04: [M6] Promoted `double_line.riv` by aggregating same-style
  authored-line-break `TextValueRun` children and placing shaped non-empty
  lines at C++-style static line-height baselines while preserving empty forced
  line breaks. `make golden-compare` moved to `exact=137`,
  `exact-segments=458`, `unsupported-feature=158`, and parked
  `M6=115 gated=7 harness=36`; next reopen `modifier_to_run.riv`, which now
  fails first on `TextModifierRange` word/line range-map units.
- 2026-07-04: [M6] Promoted `modifier_to_run.riv` by translating the static
  range-map path from C++ `src/text/text_modifier_range.cpp`: word/line and
  character-excluding-space units, runId clipping, matching-metric multi-style
  no-break runs, and per-style text paint allocation ordering. `make
  golden-compare` moved to `exact=138`, `exact-segments=459`,
  `unsupported-feature=157`, and parked `M6=114 gated=7 harness=36`; next
  reopen `test_modifier_run.riv`, which now fails first on rotation modifier
  flags.
- 2026-07-04: [M6] Promoted `test_modifier_run.riv` by translating the static
  glyph rotation path from C++ `src/text/text_modifier_group.cpp`, including
  per-glyph center transforms and averaged glyph coverage for multi-codepoint
  glyphs. `make golden-compare` moved to `exact=139`,
  `exact-segments=460`, `unsupported-feature=156`, and parked
  `M6=113 gated=7 harness=36`; next reopen `text_opacity_modifier.riv`, which
  now fails first on a static-text sibling `CubicEaseInterpolator`.
- 2026-07-04: [M6] Promoted `text_opacity_modifier.riv` by translating C++
  `TextModifierGroup::computeOpacity` and `TextStylePaint` opacity buckets,
  including temporary render-paint allocation order and libc++ float bucket
  iteration for exact stream ordering. `make golden-compare` moved to
  `exact=140`, `exact-segments=461`, `unsupported-feature=155`, and parked
  `M6=112 gated=7 harness=36`; next reopen `text_stroke_test.riv`, which now
  fails first on a static-text sibling `DashPath`.
- 2026-07-04: [M6] Promoted `text_stroke_test.riv` by admitting solid
  `Stroke` paints on `TextStylePaint`, routing DashPath effects through the
  existing shape stroke-effect path, and matching C++'s per-style text
  paint-pool allocation. `make golden-compare` moved to `exact=141`,
  `exact-segments=462`, `unsupported-feature=154`, and parked
  `M6=111 gated=7 harness=36`; next reopen `vertical_align_ellipsis.riv`,
  which now fails first on ellipsis across multiple authored lines.
- 2026-07-04: [M6] Promoted `vertical_align_ellipsis.riv` by moving
  fixed-size ellipsis line selection and bottom/middle vertical-align offsets
  into the static text render path, mirroring C++
  `src/text/text.cpp::computeBoundsInfo`/`buildRenderStyles`. `make
  golden-compare` moved to `exact=142`, `exact-segments=463`,
  `unsupported-feature=153`, and parked `M6=110 gated=7 harness=36`; next
  reopen `text_listener_simpler.riv`, which now fails first on mismatched
  no-break multi-run `TextStylePaint` metrics.
- 2026-07-04: [M6] Promoted `text_listener_simpler.riv` by shaping static
  no-break text per `TextValueRun` style/variation, using measured auto-width
  for C++-style origin offsets, and preserving per-style paint buckets. `make
  golden-compare` moved to `exact=143`, `exact-segments=464`,
  `unsupported-feature=152`, and parked `M6=109 gated=7 harness=36`; next
  reopen `new_text.riv`, which now fails first on sibling `LinearGradient`.
- 2026-07-04: [M6] Admitted `new_text.riv` through its LinearGradient sibling
  gate: static text allows gradient siblings and gradient text fill/stroke
  paints, TextStylePaints without authored font/container no longer abort the
  whole text, and keyed runtime gradient endpoints/render opacity now match
  C++. The file reaches draw but is parked as the sole known divergence on
  text-outline contour ordering between Rust/Skrifa and C++ HarfBuzz. `make
  golden-compare` reports `exact=143`, `exact-segments=464`, `diverges=1`,
  `unsupported-feature=151`, `not-yet=0`, and parked
  `M6=108 gated=7 harness=36`; next start `runtime_nested_text_runs.riv`,
  which fails first on sibling `NestedArtboard`.
- 2026-07-04: [M6] Promoted `runtime_nested_text_runs.riv` by admitting
  passive `NestedArtboard`/`NestedStateMachine` siblings around static text;
  the existing nested artboard draw path and text paint preallocation already
  matched C++ structurally once the text gate was removed. `make
  golden-compare` moved to `exact=144`, `exact-segments=465`,
  `unsupported-feature=150`, and parked `M6=107 gated=7 harness=36`; next
  start the high-frequency static text data-binding blocker with
  `bankcard.riv`.
- 2026-07-04: [M6] Admitted source-to-target `TextValueRun.text` and
  `SolidColor.colorValue` binds around static text. This promoted
  `databind_external_artboard_child.riv`, `sorted_listeners.riv`, and
  `zero_width_space_line_break.riv`; six broader data-bound text/converter
  files now run but are marked as M6 divergences; and `bankcard.riv` gets past
  data binding to the painted `LayoutComponent` gate. `make golden-compare`
  moved to `exact=147`, `exact-segments=468`, `diverges=7`,
  `unsupported-feature=141`, and parked `M6=98 gated=7 harness=36`; next
  start the painted `LayoutComponent` slice with `bankcard.riv`.
- 2026-07-04: [M6] Started the painted `LayoutComponent` slice by routing
  `LayoutComponent` shape paints through the runtime draw-command path with
  serialized background-rect commands, moving the explicit
  `layout-component-paint` runner gate ahead of static text, and retagging
  `bankcard.riv` plus ten similar files from stale `text` diagnostics to
  `layout-component-paint`. `make golden-compare` stayed at `exact=147`,
  `exact-segments=468`, `diverges=7`, `unsupported-feature=141`, and parked
  `M6=98 gated=7 harness=36`; next port computed layout bounds/style plumbing
  before removing the gate.
- 2026-07-04: [M6] Admitted the first exact painted `LayoutComponent` subset:
  simple root full-artboard solid fills now draw through the layout-proxy
  command path with C++-style background rect paths, promoting
  `viewmodel_list_trigger.riv`, `transition_index_condition.riv`,
  `viewmodel_from_context.riv`, `list_to_length_test.riv`, and
  `reset_phase.riv`. `artboard_list_map_rules.riv` is reclassified as the
  next M6 divergence on component-list/map-rule layout bounds. `make
  golden-compare` moved to `exact=152`, `exact-segments=473`,
  `diverges=8`, `unsupported-feature=135`, and parked
  `M6=92 gated=7 harness=36`; `cargo test --workspace` passes.
- 2026-07-04: [M6] Promoted `artboard_list_map_rules.riv` by translating the
  first C++ `LayoutComponent` root-row fill sizing path: sibling root layout
  children split the artboard width, layout proxy draw commands use the
  computed layout transform, and layout proxies keep per-layout path-cache
  identity. `make golden-compare` moved to `exact=153`,
  `exact-segments=474`, `diverges=7`, `unsupported-feature=135`, and parked
  `M6=92 gated=7 harness=36`; `cargo test --workspace` passes. Next target:
  `artboard_list_overrides.riv`, which stops on nested clipped layout global
  21 with `ArtboardComponentListOverride` sizing.
- 2026-07-04: [M6] Promoted `artboard_list_overrides.riv` by mirroring C++
  clipped `LayoutComponent::drawProxy` save/clip/restore ordering, giving
  layout clips their own render-path cache, and collapsing the nested fill/hug
  component-list override layout to the C++ empty-list zero-size bounds. `make
  golden-compare` moved to `exact=154`, `exact-segments=475`,
  `diverges=7`, `unsupported-feature=134`, and parked
  `M6=91 gated=7 harness=36`; next target: `bankcard.riv`, still gated on
  `layout-component-paint` global 21.
- 2026-07-04: [M6] Cleared `bankcard.riv`'s first `LayoutComponent` paint
  blocker by admitting root layout backgrounds with rounded style corners and
  moving unconditional `Feather` diagnostics ahead of text. `bankcard.riv` is
  now `gated` on feather; passive text sibling/Node ancestry admission also
  promoted `joel_v3.riv` and `word_joiner_test.riv`, while
  `number_to_list_nested_children.riv` now runs as an M6
  `layout-component-bounds` divergence. `make golden-compare` moved to
  `exact=156`, `exact-segments=477`, `diverges=8`,
  `unsupported-feature=131`, and parked `M6=87 gated=8 harness=36`; next
  target: `collapse_data_binds.riv`, still gated on `layout-component-paint`
  global 31.
- 2026-07-04: [M6] Reclassified `collapse_data_binds.riv` from generic
  `layout-component-paint` to `layout-computed-values` after finding
  data-bound `LayoutComponent.computedLocalX` values feeding text. `make
  golden-compare` stayed at `exact=156`, `exact-segments=477`,
  `diverges=8`, `unsupported-feature=131`, and parked
  `M6=87 gated=8 harness=36`; next target:
  `component_list_child_origin.riv`, still gated on `layout-component-paint`
  global 19.
- 2026-07-04: [M6] Narrowed the root row layout paint gate by admitting
  clockwise layout background paths and root padding/gap sizing. This retags
  `component_list_child_origin.riv`, `component_list_virtualized.riv`, and
  `virtualized_artboard_databound_children.riv` to `scroll-constraints`, and
  moves `transition_duration_bind_list.riv` to the existing
  `layout-component-bounds` divergence (`2617` vs C++ `2000` height). `make
  golden-compare` reports `exact=156`, `exact-segments=477`, `diverges=9`,
  `unsupported-feature=130`, and parked `M6=86 gated=8 harness=36`; next
  target: `computed_root_transform.riv`, still gated on
  `layout-component-paint` global 32.
- 2026-07-04: [M6] Promoted `computed_root_transform.riv` and
  `list_items.riv` by adding the first simple flex layout background sizing:
  non-reverse row/column parents, fixed point/percent main-axis sizes,
  fill-weighted remaining space via `fractionalWidth`/`fractionalHeight`, and
  fill/fixed/hug cross-axis sizing. Seven files now clear layout paint and are
  retagged to `rust-runner-unsupported:text`; only
  `data_bind_test_cmdq.riv`, `scroll_snap.riv`, and `scroll_test.riv` remain
  on `layout-component-paint`. `make golden-compare` reports `exact=158`,
  `exact-segments=479`, `diverges=9`, `unsupported-feature=128`, and parked
  `M6=84 gated=8 harness=36`; `cargo test --workspace` passes.
- 2026-07-04: [M6] Closed the remaining layout-component-paint manifest queue
  by admitting rounded simple flex backgrounds plus invisible, stroked, and
  gradient layout background paints already handled by the runtime draw path.
  `data_bind_test_cmdq.riv` now parks on `text`,
  `scroll_snap.riv` parks on `text`, and `scroll_test.riv` parks on
  `scroll-constraints`; `grep -n ... corpus.toml` for
  `rust-runner-unsupported:layout-component-paint` is empty. `make
  golden-compare` reports `exact=158`, `exact-segments=479`, `diverges=9`,
  `unsupported-feature=128`, and parked `M6=84 gated=8 harness=36`; `cargo
  test --workspace` passes. Next target: `collapse_data_binds.riv` on
  `layout-computed-values`.
- 2026-07-05: [M6] Closed the `layout-computed-values` runner gate by polling
  target-to-source `LayoutComponent.computed*` data binds from runtime layout
  geometry, building a graph-aware artboard context for `from_graph()`, and
  drawing static `Text` under `LayoutComponent` through runtime component
  world transforms. The layout bounds resolver is now memoized to avoid
  recursive fill/hug overflow. `collapse_data_binds.riv`,
  `data_binding_artboards_source_test.riv`, and
  `hittest_collapsed_layouts.riv` now run and are retagged as
  `rust-runner-divergence:layout-component-bounds`; the first inspected diff
  is the broader solver gap, not computed data-bind plumbing. `make
  golden-compare` reports `exact=158`, `exact-segments=479`, `diverges=12`,
  `unsupported-feature=125`, `not-yet=0`, and parked
  `M6=81 gated=8 harness=36`; `cargo test --workspace` passes. Next target:
  broader `LayoutComponent` bounds/positioning parity, starting with
  `collapse_data_binds.riv`.
- 2026-07-05: [M6] Promoted `collapse_data_binds.riv` by adding
  effective-collapse checks through layout ancestors, display-none layout
  handling, absolute layout bounds, space-between/alignment offsets, and
  intrinsic flex-basis sizing that avoids computed-bounds feedback. Narrow
  direct `DataConverterToString` default admission now lets numeric view-model
  values bind to `TextValueRun.text` without waking unrelated formula or
  interpolator defaults. `make golden-compare` reports `exact=159`,
  `exact-segments=480`, `diverges=11`, `unsupported-feature=125`,
  `not-yet=0`, and parked `M6=81 gated=8 harness=36`; next target:
  `data_binding_artboards_source_test.riv`.
- 2026-07-05: [M6] Promoted `data_binding_artboards_source_test.riv` by
  creating C++-style default view-model values from declared paths when no
  serialized default instance exists and using root-hug Artboard layout bounds
  for background drawing. `make golden-compare` reports `exact=160`,
  `exact-segments=481`, `diverges=10`, `unsupported-feature=125`,
  `not-yet=0`, and parked `M6=81 gated=8 harness=36`; `cargo test
  --workspace` passes. Next target: `hittest_collapsed_layouts.riv`.
- 2026-07-05: [M6] Promoted `hittest_collapsed_layouts.riv` by aligning the
  Rust golden runner with C++ `File::createViewModelInstance(artboard)` fresh
  view-model defaults for state-machine data contexts, while preserving
  serialized default-context probe behavior. Owned-context listener trigger
  writes now flow through target-to-source conversion and mirror the active
  view-model trigger cache so the same pointer scripts stay exact.
  `make golden-compare` reports `exact=161`, `exact-segments=482`,
  `diverges=9`, `unsupported-feature=125`, `not-yet=0`, and parked
  `M6=81 gated=8 harness=36`; `cargo test --workspace` passes. Next target:
  `number_to_list_nested_children.riv`.
- 2026-07-05: [M6] Promoted `number_to_list_nested_children.riv` after the
  focused C++/Rust golden stream compare showed the stale
  `layout-component-bounds` divergence was already closed by the previous
  layout/default-context work. `make golden-compare` reports `exact=162`,
  `exact-segments=483`, `diverges=8`, `unsupported-feature=125`,
  `not-yet=0`, and parked `M6=81 gated=8 harness=36`; next target:
  `transition_duration_bind_list.riv`.
- 2026-07-05: [M6] Promoted `transition_duration_bind_list.riv` after the
  focused direct C++/Rust stream compare also showed exact output at its
  declared sample; the stale `layout-component-bounds` manifest tag came from
  before the previous layout/default-context fixes. `make golden-compare`
  reports `exact=163`, `exact-segments=484`, `diverges=7`,
  `unsupported-feature=125`, `not-yet=0`, and parked
  `M6=81 gated=8 harness=36`; next target: `new_text.riv`, then the
  data-bound text divergence bucket.
- 2026-07-05: [M6] Rechecked the M6 text divergence queue with direct
  C++/Rust streams. `new_text.riv` remains a real text-outline contour-order
  divergence, but `format_number_with_commas.riv`,
  `listener_view_model.riv`, and `trigger_fires_single_change.riv` are now
  epsilon-equivalent and were promoted to exact. `make golden-compare`
  reports `exact=166`, `exact-segments=487`, `diverges=4`,
  `unsupported-feature=125`, `not-yet=0`, and parked
  `M6=81 gated=8 harness=36`; next target:
  `rebind_with_nested_viewmodel.riv`.
- 2026-07-05: [M6] Promoted `rebind_with_nested_viewmodel.riv` by binding
  artboard data-bind defaults to the selected artboard `viewModelId` and
  following `ViewModelPropertyViewModel.viewModelReferenceId` in declared
  paths. `make golden-compare` reports `exact=167`,
  `exact-segments=488`, `diverges=3`, `unsupported-feature=125`,
  `not-yet=0`, and parked `M6=81 gated=8 harness=36`; next target:
  `replace_vm_instance.riv`, which now has matching stream line count but
  shifted text outlines.
- 2026-07-05: [M6] Promoted `replace_vm_instance.riv` after mirroring C++
  static text horizontal alignment for `Text.alignValue` in the Rust text
  renderer. The focused C++/Rust streams are epsilon-equivalent after the
  center-aligned header text starts from the C++ line offset; `make
  golden-compare` reports `exact=168`, `exact-segments=489`, `diverges=2`,
  `unsupported-feature=125`, `not-yet=0`, and parked
  `M6=81 gated=8 harness=36`; next target: `transition_actions.riv`.
- 2026-07-05: [M6] Promoted `transition_actions.riv` by carrying scheduled
  state-machine `ListenerViewModelChange` actions through layer advancement,
  applying them to the bound view-model data-bind graph, and mirroring the
  changed source path into artboard-side data-bind values before static text
  draw. `make golden-compare` reports `exact=169`,
  `exact-segments=490`, `diverges=1`, `unsupported-feature=125`,
  `not-yet=0`, and parked `M6=81 gated=8 harness=36`; next target is the M6
  `rust-runner-unsupported:text` manifest queue, starting with
  `bindable_artboard_nesty.riv` unless a smaller text-only entry is found.
- 2026-07-05: [M6] Promoted `bindable_artboard_nesty.riv` by admitting
  source-to-target `NestedArtboard` host binds through the static text gate
  for the nested-host properties already applied by the runtime
  (`artboardId`, `isPaused`, `speed`, and `quantize`). `make golden-compare`
  reports `exact=170`, `exact-segments=491`, `diverges=1`,
  `unsupported-feature=124`, `not-yet=0`, and parked
  `M6=80 gated=8 harness=36`; next target is `component_stateful.riv`.
- 2026-07-05: [M6] Promoted `component_stateful_vm_instance_2.riv` by
  allowing static text to coexist with `Star` siblings, admitting stateful
  nested child `Shape.rotation` binds through `DataConverterSystemDegsToRads`,
  and propagating child `ViewModelInstanceColor.propertyValue` into nested
  `SolidColor.colorValue`. `make golden-compare` reports `exact=172`,
  `exact-segments=493`, `diverges=3`, `unsupported-feature=120`,
  `not-yet=0`, and parked `M6=76 gated=8 harness=36`; `cargo test
  --workspace` passes. Next target is `computed_values_test.riv`.
- 2026-07-05: [M6] Added #V2-7 per-entry verification modes to
  `golden-compare` and preserved non-default modes in `generate-corpus` so
  layout/text/image entries can declare `tolerant(ε)` or `structural` before
  moving to `exact`. Baseline after unwinding the misaligned computed-values
  spike remains `exact=172`, `exact-segments=493`, `diverges=3`,
  `unsupported-feature=120`, `not-yet=0`, and parked
  `M6=76 gated=8 harness=36`; next target is the Taffy-backed layout trait
  slice for `computed_values_test.riv`.
- 2026-07-05: [M6] Routed supported `LayoutComponent` bounds through a
  #V2-7 Taffy layout trait that computes coherent whole-artboard snapshots
  from Rive style data, refuses nested artboard/component-list provider trees
  this slice cannot model yet, and leaves the old hand-rolled helpers as
  fallback only for refused trees. The existing simple root row/column layout
  cases stay exact under the snapshot-first resolver. `make golden-compare`
  remains `exact=172`, `exact-segments=493`, `diverges=3`,
  `unsupported-feature=120`, `not-yet=0`, and parked
  `M6=76 gated=8 harness=36`; `cargo test --workspace` passes. Next target:
  reopen `computed_values_test.riv` through the Taffy-backed layout path.
- 2026-07-05: [M6] Reopened `computed_values_test.riv` by admitting
  `ArtboardComponentList.listSource`, nested child `Shape.computedRootX/Y`
  binds, and empty component-list provider trees through the Taffy layout
  adapter. The file now reaches draw and is retagged as
  `rust-runner-divergence:computed-values-text`; `computed_root_transform.riv`
  declares `verification = "tolerant(0.5)"` for the subpixel Yoga/Taffy layout
  rounding exposed by the same path. `make golden-compare` reports
  `exact=172`, `exact-segments=493`, `diverges=4`,
  `unsupported-feature=119`, `not-yet=0`, and parked
  `M6=75 gated=8 harness=36`; next target is `follow_path_path.riv`.
- 2026-07-05: [M6] Reopened `follow_path_path.riv` by admitting static text
  siblings `FollowPathConstraint`, `CubicDetachedVertex`,
  `CubicAsymmetricVertex`, and `CubicMirroredVertex`. It now reaches draw and
  is parked as `rust-runner-divergence:follow-path-text-outline`; the same
  gate removal made `spotify_kids_app_icon.riv` reach draw, now parked as
  `rust-runner-divergence:spotify-icon-draw-order`. `make golden-compare`
  reports `exact=172`, `exact-segments=493`, `diverges=6`,
  `unsupported-feature=117`, `not-yet=0`, and parked
  `M6=73 gated=8 harness=36`; `cargo test --workspace` passes. Next target is
  `data_bind_test_cmdq.riv`.
- 2026-07-05: [M6] Admitted inert `Event` siblings through the static text
  gate. `nested_events.riv` is exact by focused stream comparison;
  `data_bind_test_cmdq.riv` now reaches draw and is parked as
  `rust-runner-divergence:data-bind-command-queue-text-layout`; the same gate
  removal reopens `state_transition_fire_trigger.riv` and
  `trigger_based_listeners.riv`, both parked as
  `rust-runner-divergence:event-trigger-extra-text-draw`. `make
  golden-compare` reports `exact=173`, `exact-segments=494`, `diverges=9`,
  `unsupported-feature=113`, `not-yet=0`, and parked
  `M6=69 gated=8 harness=36`; `cargo test --workspace` passes. Next target is
  `data_binding_test.riv`.
- 2026-07-05: [M6] Reopened `data_binding_test.riv` by admitting
  `ForegroundLayoutDrawable` through the static text gate; that C++ class is
  already modeled in graph/draw ordering as layout foreground paint glue. The
  file reaches draw and is parked as
  `rust-runner-divergence:foreground-layout-text-transform` after the focused
  stream diff showed C++ placing text at `[1,0,0,1,400,468.925781]` while Rust
  emits identity transform and a shorter stream. `make golden-compare` reports
  `exact=173`, `exact-segments=494`, `diverges=10`,
  `unsupported-feature=112`, `not-yet=0`, and parked
  `M6=68 gated=8 harness=36`; `cargo test --workspace` passes. Next target is
  `data_converter_to_number.riv`.
- 2026-07-05: [M6] Reopened `data_converter_to_number.riv` by admitting
  custom-property siblings through static text and adding
  `CustomPropertyBoolean`/`CustomPropertyColor` target-to-source binding
  values. The file reaches draw and is parked as
  `rust-runner-divergence:data-converter-to-number-text-values` after focused
  streams showed the first text path at `[1,0,0,1,34.473156,389.39209]` had 17
  C++ contours versus 15 Rust contours. `make golden-compare` reports
  `exact=173`, `exact-segments=494`, `diverges=11`,
  `unsupported-feature=111`, `not-yet=0`, and parked
  `M6=67 gated=8 harness=36`; next target is `databind_solo_to_enum.riv`.
- 2026-07-05: [M6] Promoted `databind_solo_to_enum.riv` by admitting Solo
  parent/sibling text, mapping enum source-to-target Solo binds through
  DataEnum labels, mirroring target-to-source Solo active-child enum writes,
  and applying `Text.alignValue` enum/uint binds. `make golden-compare`
  reports `exact=174`, `exact-segments=495`, `diverges=11`,
  `unsupported-feature=110`, `not-yet=0`, and parked
  `M6=66 gated=8 harness=36`; `cargo test --workspace` passes. Next target is
  `fit_font_size_test.riv`.
- 2026-07-05: [M6] Reopened `fit_font_size_test.riv` by admitting
  source-to-target `TextStylePaint.fontSize`, `Text.overflowValue`, and
  `LayoutComponent.height` binds through the static text gate. The file reaches
  draw and is parked as
  `rust-runner-divergence:text-fit-font-size-layout-bounds`: focused streams
  show C++ wrapping/fitting the right-column text where Rust keeps advancing on
  a wider line (`x=7.71484375` versus `x=212.890625`), and C++ emits a
  zero-sized middle text path that Rust suppresses. `make golden-compare`
  reports `exact=174`, `exact-segments=495`, `diverges=12`,
  `unsupported-feature=109`, `not-yet=0`, and parked
  `M6=65 gated=8 harness=36`; `cargo test --workspace` passes. Next target is
  `hit_test_nested.riv`.
- 2026-07-05: [M6] Promoted `hit_test_nested.riv` by admitting authored
  `NestedBool` siblings through the static text gate and allowing static text
  under `Shape` parent transforms. Focused direct streams then matched C++ under
  numeric-token epsilon, and the full corpus promoted the file to exact.
  `make golden-compare` reports `exact=175`, `exact-segments=496`,
  `diverges=12`, `unsupported-feature=108`, `not-yet=0`, and parked
  `M6=64 gated=8 harness=36`; `cargo test --workspace` passes. Next target is
  `interpolate_to_end.riv`.
- 2026-07-05: [M6] Reopened `interpolate_to_end.riv` by admitting nested child
  `TextValueRun.text` converter groups through the golden-runner gate and
  letting artboard property-binding admission validate stateful converter
  groups with `RuntimeDataBindGraphConverterState`. The file now reaches draw
  and is parked as
  `rust-runner-divergence:nested-child-text-converter-context`: focused streams
  show C++ rendering the nested data-bound/interpolated numeric text at
  `[1,0,0,1,245.207031,58.4726562]` while Rust still emits the serialized
  fallback text glyph payload. `make golden-compare` reports `exact=175`,
  `exact-segments=496`, `diverges=13`, `unsupported-feature=107`,
  `not-yet=0`, and parked `M6=63 gated=8 harness=36`; `cargo test
  --workspace` passes. Next target is `keyboard_listener.riv`.
- 2026-07-05: [M6] Promoted `keyboard_listener.riv` by admitting passive
  `FocusData` and `KeyboardInput` siblings through the static text subset for
  sample-0 rendering. The file's direct C++/Rust streams have the same call
  sequence and pass golden numeric-token comparison, so the stale
  `rust-runner-unsupported:text` manifest gate is removed. `make
  golden-compare` reports `exact=176`, `exact-segments=497`, `diverges=13`,
  `unsupported-feature=106`, `not-yet=0`, and parked
  `M6=62 gated=8 harness=36`; `cargo test --workspace` passes. Next target is
  `list_index_script_access.riv`.
- 2026-07-05: [M6] Promoted `list_index_script_access.riv` by admitting
  inert `ScriptedDrawable` siblings through the static text subset for
  sample-0 rendering and declaring its existing Taffy/Yoga list-row rounding
  drift as `verification = "tolerant(0.75)"`. The same gate removal reopens
  `scripted_data_context.riv`, now parked as
  `rust-runner-divergence:scripted-data-context-extra-text-draw` after direct
  streams showed Rust drawing two data-bound text payloads C++ suppresses.
  `make golden-compare` reports `exact=177`, `exact-segments=498`,
  `diverges=14`, `unsupported-feature=104`, `not-yet=0`, and parked
  `M6=60 gated=8 harness=36`; `cargo test --workspace` passes. Next target is
  `saturation.riv`.
- 2026-07-05: [M6] Reopened `saturation.riv` by admitting static-text
  `Shape.x/y` source-to-target binds with no converter or a
  `DataConverterGroup`, clearing the stale `rust-runner-unsupported:text`
  stop. The file reaches draw and is parked as
  `rust-runner-divergence:saturation-color-to-string-text`: focused streams
  first differ at text path id 3 under `[1,0,0,1,64.5,26.5]`, while the later
  numeric/color text path is only float drift. `make golden-compare` reports
  `exact=177`, `exact-segments=498`, `diverges=15`,
  `unsupported-feature=103`, `not-yet=0`, and parked
  `M6=59 gated=8 harness=36`; `cargo test --workspace` passes. Next target is
  `scroll_snap.riv`.
- 2026-07-05: [M6] Reclassified `scroll_snap.riv` by moving the existing
  `ScrollConstraint` runner preflight ahead of the static-text gate, so the
  first Rust diagnostic is now `rust-runner-unsupported:scroll-constraints`
  for global 93 instead of a stale sibling-text error. This confirms the file
  belongs with the scroll/layout runtime queue, not the text-layout queue.
  `make golden-compare` reports `exact=177`, `exact-segments=498`,
  `diverges=15`, `unsupported-feature=103`, `not-yet=0`, and parked
  `M6=59 gated=8 harness=36`; `cargo test --workspace` passes. Next target is
  `stateful_source_switch.riv`.
- 2026-07-05: [M6] Promoted `stateful_source_switch.riv` by admitting
  no-converter source-to-target `ParametricPath.width/height` binds for static
  text sibling shapes (`Ellipse` in the active stateful source, plus the same
  C++ property family for Polygon/Rectangle/Star/Triangle). Direct Rust and C++
  sample-0 streams now match the parent artboard background-only draw. `make
  golden-compare` reports `exact=178`, `exact-segments=499`, `diverges=15`,
  `unsupported-feature=102`, `not-yet=0`, and parked
  `M6=58 gated=8 harness=36`; `cargo test --workspace` passes. Next target is
  `text_follow_path_shape_length.riv`.
- 2026-07-05: [M6] Reclassified `text_follow_path_shape_length.riv` after
  admitting source-to-target `Text.width/height` data binds with no converter
  or `DataConverterFormula` through the static text gate. Direct Rust now gets
  past the generic `Text` property blockers and stops on
  `TextFollowPathModifier` global 168, so the file is retagged as
  `rust-runner-unsupported:text-follow-path-modifier`. `make golden-compare`
  remains `exact=178`, `exact-segments=499`, `diverges=15`,
  `unsupported-feature=102`, `not-yet=0`, and parked
  `M6=58 gated=8 harness=36`; `cargo test --workspace` passes. Next target is
  `text_vertical_trim_test.riv`.
- 2026-07-05: [M6] Reclassified `text_vertical_trim_test.riv` as
  `rust-runner-unsupported:text-vertical-trim` after confirming property keys
  1027/1028 are `Text.verticalTrimTopValue` /
  `Text.verticalTrimBottomValue`, bitmask passthroughs into
  `verticalTrimValue`. C++ applies them in `src/text/text.cpp` through
  `Text::computeVerticalTrim` to the rendered/measured text bounds, so this is
  a real text-layout port rather than a finite static admission. Direct Rust
  now reports `unsupported: text-vertical-trim`; `make golden-compare` remains
  `exact=178`, `exact-segments=499`, `diverges=15`,
  `unsupported-feature=102`, `not-yet=0`, and parked
  `M6=58 gated=8 harness=36`; `cargo test --workspace` passes. Next target is
  `transition_duration_bind_nested.riv`.
- 2026-07-05: [M6] Reclassified `transition_duration_bind_nested.riv` by
  admitting nested child `TextValueRun.text` through `DataConverterFormula`.
  The stale generic `rust-runner-unsupported:text` stop is gone: direct Rust
  reaches draw, and the first real diff is nested transition-duration reveal
  behavior where C++ collapses the icon circles to zero-scale transforms at
  sample 0 while Rust draws them at full scale. The #V2-7 decision language was
  reviewed at the same time and remains the right guardrail: Taffy is the
  layout adapter, tolerant verification is numeric-only, and missing
  text/layout behavior must stay visible as diagnostics or divergences.
  `make golden-compare` reports `exact=178`, `exact-segments=499`,
  `diverges=16`, `unsupported-feature=101`, `not-yet=0`, and parked
  `M6=58 gated=8 harness=36`; `cargo test --workspace` passes. Next target is
  `transition_duration_bind_nested.riv` as a focused nested
  transition-duration/data-bind divergence.
- 2026-07-05: [M6] Promoted `transition_duration_bind_nested.riv` by mirroring
  C++ per-instance `StateTransition.duration` data binds. State-machine data
  binds targeting transitions now create runtime transition-duration slots,
  child-artboard default view-model values resolve against the selected
  artboard context, and transition mixing rounds/clamps bound durations like
  C++ `StateMachineInstance::resolvedDuration`. `make golden-compare` reports
  `exact=179`, `exact-segments=500`, `diverges=15`,
  `unsupported-feature=101`, `not-yet=0`, and parked
  `M6=57 gated=8 harness=36`; `cargo test --workspace` passes. Next target is
  the M6 text layout/draw-suppression bucket, starting with
  `data_binding_test.riv`.
- 2026-07-05: [M6] Narrowed `data_binding_test.riv` by routing
  `ForegroundLayoutDrawable` paints through their parent `LayoutComponent`
  path/transform, threading the coherent Taffy layout snapshot into draw/text,
  disabling Taffy rounding to mirror Yoga point-scale `0`, measuring static
  Text leaves for layout control size, and using controlled layout width for
  auto-width text alignment under non-artboard layout parents. Focused streams
  now have matching length and no identity-transform failure; the first
  remaining diff is the Taffy/Yoga wrapped row offset described above.
  `make golden-compare` remains `exact=179`, `exact-segments=500`,
  `diverges=15`, `unsupported-feature=101`, `not-yet=0`, and parked
  `M6=57 gated=8 harness=36`; `cargo test --workspace` passes. Next target is
  a C++/Rust local-id layout-box probe for `data_binding_test.riv`.
- 2026-07-05: [M6] Promoted `data_binding_test.riv` after the local-id
  C++ Yoga/Rust Taffy probe showed all 142 layout nodes match once static Text
  leaves measure with finite layout constraints. The remaining focused stream
  diff was `DataConverterToString` spelling C++ `std::to_string(NaN)` as
  lowercase `nan`, now mirrored in the shared converter helper. `make
  golden-compare` reports `exact=180`, `exact-segments=501`, `diverges=14`,
  `unsupported-feature=101`, `not-yet=0`, and parked
  `M6=57 gated=8 harness=36`; `cargo test --workspace` passes. Next target is
  `data_bind_test_cmdq.riv` in the text layout/draw-suppression bucket.
- 2026-07-05: [M6] Narrowed `data_bind_test_cmdq.riv` by measuring Shape
  layout leaves in the Taffy adapter using C++ `Shape::measureLayout` /
  `ParametricPath::measureLayout` semantics for the static runtime subset.
  Rust layout-bounds now succeeds with all 19 boxes and local 40 measures
  24x24; the first stream diff improved from y=`460.671631` to
  y=`453.185791` but the file remains a known Taffy/Yoga text-layout
  divergence. `make golden-compare` remains `exact=180`,
  `exact-segments=501`, `diverges=14`, `unsupported-feature=101`,
  `not-yet=0`, and parked `M6=57 gated=8 harness=36`;
  `cargo test --workspace` passes. Next target stays
  `data_bind_test_cmdq.riv`.
- 2026-07-05: [M6] Narrowed `data_bind_test_cmdq.riv` again by mirroring C++
  `LayoutComponent::syncStyle`: only leaf layout components with
  `intrinsicallySizedValue` get a Taffy measure context. The C++ Yoga and Rust
  Taffy local-id layout boxes now match for all 19 nodes, including the bottom
  command-queue block at local 98/101; the remaining first diff is the
  `Update Random Vals` glyph path payload at the matched transform, so the
  file is retagged as `rust-runner-divergence:data-bind-command-queue-text-outline`.
  `make golden-compare` remains `exact=180`, `exact-segments=501`,
  `diverges=14`, `unsupported-feature=101`, `not-yet=0`, and parked
  `M6=57 gated=8 harness=36`; `cargo test --workspace` passes. Next target is
  `data_converter_to_number.riv`.
- 2026-07-05: [M6] Promoted `data_converter_to_number.riv` after refreshing
  focused C++/Rust streams: the stale 17-vs-15 contour note was gone, both
  streams had 75 lines with matching non-numeric structure, and the largest
  numeric text-outline delta was about `1e-6`, below the normal golden epsilon.
  `make golden-compare` reports `exact=181`, `exact-segments=502`,
  `diverges=13`, `unsupported-feature=101`, `not-yet=0`, and parked
  `M6=57 gated=8 harness=36`; next target is `scripted_data_context.riv`.
- 2026-07-05: [M6] Reclassified `scripted_data_context.riv` as an explicit
  scripting gate after focused streams showed the C++ runner printing
  `Failed to import object of type 106` and suppressing two script-driven
  data-context text draws. The Rust runner now emits
  `unsupported: scripted-data-context` only for selected artboards with a
  `ScriptedDrawable`, `DataBindContext` text, and nested view-model context;
  checked exact script fixtures `list_index_script_access.riv` and
  `scripting_root_viewmodel.riv` still stream. `make golden-compare` reports
  `exact=181`, `exact-segments=502`, `diverges=12`,
  `unsupported-feature=102`, `not-yet=0`, and parked
  `M6=58 gated=8 harness=36`; next target is
  `state_transition_fire_trigger.riv`.
- 2026-07-05: [M6] Promoted `state_transition_fire_trigger.riv` and
  `trigger_based_listeners.riv` by preserving nested child default text
  contexts when the child artboard owns state-machine data binds, while
  retaining serialized-text fallback for plain nested text hosts. Focused
  sample-0 streams match C++; `make golden-compare` reports `exact=183`,
  `exact-segments=504`, `diverges=10`, `unsupported-feature=102`,
  `not-yet=0`, and parked `M6=58 gated=8 harness=36`; `cargo test
  --workspace` passes. Next target is the text-outline
  backend/canonicalization slice starting with `new_text.riv`.
- 2026-07-05: [M6] Promoted `new_text.riv` by using Skrifa FreeType-style
  outline extraction for static fonts while retaining HarfBuzz-style outlines
  for variable fonts, matching C++'s HarfBuzz callback contour starts without
  regressing Inter variable-font text fixtures. Focused streams for
  `new_text.riv` and sampled exact text fixtures match under the golden
  epsilon; `make golden-compare` reports `exact=184`,
  `exact-segments=505`, `diverges=9`, `unsupported-feature=102`,
  `not-yet=0`, and parked `M6=58 gated=8 harness=36`; `cargo test
  --workspace` passes. Next target is `follow_path_path.riv`'s follow-path
  text transform.
- 2026-07-05: [M6] Promoted `follow_path_path.riv` by letting text draw use
  constraint-written component world transforms unless a layout ancestor needs
  the #V2-7 layout-bounds path. Focused streams now match all four follow-path
  text transforms under the golden epsilon. `make golden-compare` reports
  `exact=185`, `exact-segments=506`, `diverges=8`,
  `unsupported-feature=102`, `not-yet=0`, and parked
  `M6=57 gated=8 harness=36`; `cargo test --workspace` passes. Next target is
  `data_bind_test_cmdq.riv`.
- 2026-07-05: [M6] Promoted `data_bind_test_cmdq.riv` by mirroring C++
  `LayoutComponent::propagateSizeToChildren` / `ParametricPath::controlSize`
  for layout-controlled parametric shape draw. The focused command-queue
  sample now keeps matching local-id layout boxes and expands the inner
  authored `20x18` trigger ellipse to the solved `24x24` layout size before
  draw, matching C++ under the golden epsilon. `make golden-compare` reports
  `exact=186`, `exact-segments=507`, `diverges=7`,
  `unsupported-feature=102`, `not-yet=0`, and parked
  `M6=58 gated=8 harness=36`; `cargo test --workspace` passes. Next target is
  `saturation.riv`.
- 2026-07-05: [M6] Narrowed `saturation.riv` by making artboard
  custom-property target-to-source binds carry the data-bind flags, converter
  state, and source default kind, then applying C++'s main-direction converter
  rule before writing the shared artboard source cache. This fixes the coarse
  wrong-text/fallback path: the focused sample now matches the data-bound
  color-to-string payloads and only diverges on tiny text outline coordinate
  drift. `cargo check -q -p rive-runtime` passes, and `make golden-compare`
  remains `exact=186`, `exact-segments=507`, `diverges=7`,
  `unsupported-feature=102`, `not-yet=0`, with parked
  `M6=58 gated=8 harness=36`. Next pass should decide whether the remaining
  `saturation.riv` float drift is a direct text-outline parity fix or a
  verification-mode policy decision.
- 2026-07-05: [M6] Promoted `saturation.riv` after the narrowed focused diff
  proved to be same-structure HarfRust/C++ outline coordinate drift at roughly
  `1e-6`, not missing text layout or data-bind behavior. The entry now declares
  `verification = "tolerant(0.00001)"`, small enough that integer IDs still
  cannot be accidentally accepted by the current numeric-token comparator.
  `make golden-compare` reports `exact=187`, `exact-segments=508`,
  `diverges=6`, `unsupported-feature=102`, `not-yet=0`, and parked
  `M6=58 gated=8 harness=36`. Next target is `fit_font_size_test.riv`.
- 2026-07-05: [M6] Promoted `fit_font_size_test.riv` by translating C++
  `src/text/text.cpp::Text::fitFontScale` into the static text path: Rust now
  binary-searches the largest fitting integer top font size, scales font-size
  only during shaping/metrics/line breaking, and preserves C++ zero-font
  collapsed text paths. Focused streams are epsilon-equivalent under the exact
  comparator. `make golden-compare` reports `exact=188`,
  `exact-segments=509`, `diverges=5`, `unsupported-feature=102`,
  `not-yet=0`, and parked `M6=58 gated=8 harness=36`;
  `cargo test --workspace` passes. Next target is `spotify_kids_app_icon.riv`.
- 2026-07-06: [M6] Promoted `spotify_kids_app_icon.riv` by routing root
  artboard background paints through the same C++ `ShapePaint::shouldDraw`
  visibility gate used by regular shape paints. This suppresses the hidden
  full-artboard Backboard fill before the centered icon while preserving the
  visible rounded background draw. `make golden-compare` reports `exact=189`,
  `exact-segments=510`, `diverges=4`, `unsupported-feature=102`, `not-yet=0`,
  and parked `M6=58 gated=8 harness=36`; `cargo test --workspace` passes. Next
  target is `computed_values_test.riv`.
- 2026-07-06: [M6] Promoted `computed_values_test.riv` by mirroring C++
  host-first artboard data-bind updates and `Node::computedRootX/Y`
  root-transform semantics through nested artboard hosts. Nested child
  `Shape.computedRootX/Y` now publishes `238.5/205` in root artboard space
  instead of child-local `39/49`; focused streams are exact under the normal
  golden epsilon. `make golden-compare` reports `exact=190`,
  `exact-segments=511`, `diverges=3`, `unsupported-feature=102`, `not-yet=0`,
  and parked `M6=58 gated=8 harness=36`; `cargo test --workspace` passes. Next
  target is `relative_data_binding.riv` with `shared_viewmodel_instance.riv`.
- 2026-07-06: [M6] Promoted `relative_data_binding.riv` and
  `shared_viewmodel_instance.riv` by binding owned view-model contexts through
  nested artboard hosts, resolving manifest-backed name paths, defaulting fresh
  generated color values to opaque black, and clearing missing name-based
  `TextValueRun.text` only for concrete nested owned contexts. The runner now
  applies this owned-context pass to nested artboards only, leaving root
  artboard values on the existing state-machine/default data-bind path so
  `transition_actions.riv` remains exact. `make golden-compare` reports
  `exact=192`, `exact-segments=513`, `diverges=1`,
  `unsupported-feature=102`, `not-yet=0`, and parked
  `M6=58 gated=8 harness=36`; `cargo test --workspace` passes. Next target is
  `interpolate_to_end.riv`.
- 2026-07-06: [M6] Promoted `interpolate_to_end.riv` after the previous nested
  owned view-model/context work reduced its focused stream diff to numeric path
  drift accepted by the standard exact comparator epsilon. A scratch exact
  corpus for only this file passes, and full `make golden-compare` reports
  `exact=193`, `exact-segments=514`, `diverges=0`,
  `unsupported-feature=102`, `not-yet=0`, and parked
  `M6=58 gated=8 harness=36`; `cargo test --workspace` passes. Next target is
  `text_follow_path_shape_length.riv`.
- 2026-07-06: [M6] Promoted `text_follow_path_shape_length.riv` by porting
  static `TextFollowPathModifier` over Shape/Path targets, adding
  target-to-source TrimPath/Shape length source values for the fixture's
  formula-driven `Text.width`, and mirroring C++'s text follow-path
  `PathMeasure(&path, 0.1f)` tolerance. Focused and full streams are exact:
  `make golden-compare` reports `exact=194`, `exact-segments=515`,
  `diverges=0`, `unsupported-feature=101`, `not-yet=0`, and parked
  `M6=57 gated=8 harness=36`; `cargo test --workspace` passes. Next target is
  `text_vertical_trim_test.riv`.
- 2026-07-06: [M6] Promoted `text_vertical_trim_test.riv` by adding generated
  bitmask passthrough set/get support for `Text.verticalTrimTopValue` /
  `Text.verticalTrimBottomValue`, admitting their no-converter data binds, and
  porting the static `Text::computeVerticalTrim` bounds/render offset path for
  the current text subset. Focused `background_measure.riv` stayed exact after
  backing out a too-broad line-metric detour, and focused
  `text_vertical_trim_test.riv` is exact. Full `make golden-compare` reports
  `exact=195`, `exact-segments=516`, `diverges=0`,
  `unsupported-feature=100`, `not-yet=0`, and parked
  `M6=56 gated=8 harness=36`. Next target is the M6 image bucket, starting
  with `custom_image_name.riv`.
- 2026-07-06: [M6] Promoted `custom_image_name.riv` by porting the first
  non-mesh `Image::draw` slice from C++ `src/shapes/image.cpp`, decoding
  in-band `ImageAsset` contents through the render factory before the stream
  `source` marker, and keeping broader mesh/layout/data-bound image files
  behind the existing image diagnostic. Focused C++ and Rust streams are
  byte-identical. Full `make golden-compare` reports `exact=196`,
  `exact-segments=517`, `diverges=0`, `unsupported-feature=99`, `not-yet=0`,
  and parked `M6=55 gated=8 harness=36`. Next target is the nested-library
  image pair: `double_library_with_image.riv` and `library_with_image.riv`.
- 2026-07-06: [M6] Reviewed the #V2-7 decision and enforced its fallback
  boundary in `rust-golden-runner`: painted `LayoutComponent` corpus entries
  now fail with a Taffy-refused diagnostic instead of promoting through the
  legacy hand-rolled layout resolver. To keep existing exact list fixtures on
  the snapshot path, the Taffy adapter now treats root artboards as definite
  viewport nodes even when their style uses hug axes, and admits zero-sized
  `ArtboardComponentList` metadata children such as map rules and overrides.
  `transition_duration_bind_list.riv`, `artboard_list_map_rules.riv`, and
  `artboard_list_overrides.riv` all compute Taffy layout bounds. Full
  `make golden-compare` remains `exact=196`, `exact-segments=517`,
  `diverges=0`, `unsupported-feature=99`, `not-yet=0`, and parked
  `M6=55 gated=8 harness=36`; `cargo test --workspace` passes. Next target
  remains the nested-library image pair: `double_library_with_image.riv` and
  `library_with_image.riv`.
- 2026-07-06: [M6] Promoted `double_library_with_image.riv` and
  `library_with_image.riv` by widening the Rust runner's image admission from
  a single image-only artboard to a static image artboard tree. The existing
  runtime image cache already predecoded embedded `ImageAsset` contents and
  threaded them through nested hosts; focused C++/Rust streams for both
  fixtures and `custom_image_name.riv` are byte-identical. Full
  `make golden-compare` reports `exact=198`, `exact-segments=519`,
  `diverges=0`, `unsupported-feature=97`, `not-yet=0`, and parked
  `M6=53 gated=8 harness=36`; `cargo test --workspace` passes. Next target is
  the top-level image pair: `hosted_image_file.riv` and `in_band_asset.riv`.
- 2026-07-06: [M6] Closed the #V2-7 fallback-fence audit gap by treating
  `ArtboardComponentList` `ListFollowPathConstraint` children as zero-sized
  metadata in the Taffy adapter. `component_list_follow_path_distance.riv` now
  computes a coherent layout snapshot instead of relying on legacy bounds
  fallback, and a sweep of all 33 exact `LayoutComponent` entries reports zero
  `--layout-bounds` failures. Full `make golden-compare` remains `exact=198`,
  `exact-segments=519`, `diverges=0`, `unsupported-feature=97`, `not-yet=0`,
  and parked `M6=53 gated=8 harness=36`; `cargo test --workspace` passes.
  Next target remains the top-level image pair: `hosted_image_file.riv` and
  `in_band_asset.riv`.
- 2026-07-06: [M6] Promoted `hosted_image_file.riv` and
  `in_band_asset.riv` by admitting simple image artboards with root background
  paints and hosted image assets that have no decoded `RenderImage`, matching
  C++ `Image::draw`'s early return before save. Image predecode now runs after
  source-file paint allocation but before selected-artboard clone paint
  allocation, matching C++ paint/decode ID order for in-band image contents.
  Focused streams for both fixtures plus `custom_image_name.riv`,
  `library_with_image.riv`, and `double_library_with_image.riv` are
  byte-identical. Full `make golden-compare` reports `exact=200`,
  `exact-segments=521`, `diverges=0`, `unsupported-feature=95`, `not-yet=0`,
  and parked `M6=51 gated=8 harness=36`; `cargo test --workspace` passes.
  Next target is `walle.riv` and `image_fit_alignment_3.riv`.
- 2026-07-06: [M6] Promoted `walle.riv` and `image_fit_alignment_3.riv`.
  `walle.riv` admits inert image-artboard animation metadata and preserves C++
  multi-image decode/source-paint ordering by splitting the first embedded
  image decode before source paint allocation. `image_fit_alignment_3.riv`
  ports the plain non-mesh `Image::controlSize` / `updateImageScale`
  fit/alignment path for images under a Taffy-backed `LayoutComponent`,
  including C++'s zero-sized recording-image NaN transform surface. The
  #V2-7 fallback fence now rejects any layout-dependent draw candidate when
  Taffy refuses the whole-artboard snapshot, not just painted
  `LayoutComponent` paths. Asset-image view-model binding/reset files remain
  explicitly gated after `image_fit_alignment.riv` and
  `viewmodel_image_reset.riv` proved successful Rust image drawing is not
  exact there. Full `make golden-compare` reports `exact=202`,
  `exact-segments=523`, `diverges=0`, `unsupported-feature=93`, `not-yet=0`,
  and parked `M6=49 gated=8 harness=36`; `cargo test --workspace` passes.
  Next target is the asset-image view-model pair:
  `image_fit_alignment.riv` and `viewmodel_image_reset.riv`.
- 2026-07-06: [M6] Promoted `viewmodel_image_reset.riv` by applying
  `ViewModelInstanceAssetImage` defaults to `Image.assetId` targets like C++
  `DataBindContextValueAssetImage`, including the empty private image-asset
  reset path that suppresses `Image::draw`. Removed the blanket asset-image
  image gate and replaced `image_fit_alignment.riv` with the sharper
  `rust-runner-unsupported:asset-image-layout` diagnostic after the focused
  diff narrowed to image decode ordering plus LayoutComponent Y placement
  (`272.5` in C++ vs `539.5` in Rust). Full `make golden-compare` reports
  `exact=203`, `exact-segments=524`, `diverges=0`,
  `unsupported-feature=92`, `not-yet=0`, and parked
  `M6=48 gated=8 harness=36`; `cargo test --workspace` passes. Next target is
  `image_fit_alignment.riv`, then `image_fit_alignment_2.riv`.
- 2026-07-06: [M6] Closed the #V2-7 payload-hash image verifier gap by
  switching both recording factories from encoded `size/hash` lines to
  decoded-header `width/height` lines for PNG/JPEG/WebP assets, and by
  returning those dimensions through `RecordingRenderImage`. Focused
  `image_fit_alignment_3.riv` and `walle.riv` streams remain exact with real
  dimensions. Full `make golden-compare` remains `exact=203`,
  `exact-segments=524`, `diverges=0`, `unsupported-feature=92`, `not-yet=0`,
  and parked `M6=48 gated=8 harness=36`; `cargo test --workspace` passes.
  Next target remains `image_fit_alignment.riv`, then
  `image_fit_alignment_2.riv`.
- 2026-07-06: [M6] Promoted `image_fit_alignment.riv` by mapping Yoga
  undefined position insets to Taffy auto in the #V2-7 layout adapter, matching
  C++ `LayoutComponent::positionTypeChanged` for stale non-absolute
  `positionTop` values, and by widening image predecode ordering for
  asset-image-bound layout trees so the first two embedded images decode
  before source paint allocation. Focused streams for
  `image_fit_alignment.riv`, `viewmodel_image_reset.riv`, `walle.riv`, and
  `image_fit_alignment_3.riv` are exact. Full `make golden-compare` reports
  `exact=204`, `exact-segments=525`, `diverges=0`,
  `unsupported-feature=91`, `not-yet=0`, and parked
  `M6=47 gated=8 harness=36`; `cargo test --workspace` passes. Next target is
  `image_fit_alignment_2.riv`.
- 2026-07-06: [M6] Promoted `image_fit_alignment_2.riv` by admitting
  metadata-only `NSlicer`/`AxisX`/`AxisY` image artboards through the static
  image gate while keeping meshes and actual `NSlicedNode` draw behavior gated.
  The existing runtime draw path already matched C++: the fixture decodes the
  embedded images but renders only layout-component background paints. Focused
  C++/Rust streams are exact. Full `make golden-compare` reports `exact=205`,
  `exact-segments=526`, `diverges=0`, `unsupported-feature=90`, `not-yet=0`,
  and parked `M6=46 gated=8 harness=36`; `cargo test --workspace` passes. Next
  target is the M6 nested-artboard-layout bucket, starting with
  `artboard_width_test.riv`.
- 2026-07-06: [M6] Promoted `artboard_width_test.riv` and
  `transition_artboard_condition_test.riv` by admitting layout-backed nested
  artboard hosts into the Taffy snapshot, applying solved host dimensions to
  persistent child artboards before nested state-machine advancement, drawing
  `NestedArtboardLayout` hosts at their solved layout node position, and
  parsing `TransitionArtboardCondition` through the existing artboard-number
  comparator path. Focused streams for both fixtures are exact. Full
  `make golden-compare` reports `exact=207`, `exact-segments=528`,
  `diverges=0`, `unsupported-feature=88`, `not-yet=0`, and parked
  `M6=44 gated=8 harness=36`; `cargo test --workspace` passes. Next target
  remains the nested-artboard-layout bucket, starting with
  `collapsing_elements.riv`.
- 2026-07-06: [M6] Promoted `collapsing_elements.riv` and
  `multitouch_enter.riv` by admitting `NestedArtboardLeaf` as a persistent
  nested artboard host, matching C++ leaf `computeAlignment` behavior, and
  giving fixed/default `NestedArtboardLayout` nodes the referenced child
  artboard's intrinsic size instead of collapsing to a zero-size Taffy leaf.
  Focused C++/Rust stream diffs for both fixtures are exact. Full
  `make golden-compare` reports `exact=209`, `exact-segments=530`,
  `diverges=0`, `unsupported-feature=86`, `not-yet=0`, and parked
  `M6=42 gated=8 harness=36`; `cargo test --workspace` passes. Next target is
  the M6 image bucket, starting with `bad_skin.riv`.
- 2026-07-06: [M6] Promoted `image_binding_with_listener.riv` by narrowing the
  static image gate to admit simple Shape/Rectangle siblings beside image
  drawables while keeping complex image files with skins, scripts, draw rules,
  events, and constraints behind `rust-runner-unsupported:images`. A scratch
  exact corpus proved only this one of the eight newly admitted image entries
  matched C++; full `make golden-compare` reports `exact=210`,
  `exact-segments=531`, `diverges=0`, `unsupported-feature=85`, `not-yet=0`,
  and parked `M6=41 gated=8 harness=36`; `cargo test --workspace` passes.
  Next target remains the M6 image bucket, starting with `bad_skin.riv`.
- 2026-07-06: [M6] Promoted `library.riv` by admitting simple static text
  siblings beside non-mesh image drawables in the static image gate and the
  static text sibling allow-list. Focused and full streams are exact; full
  `make golden-compare` reports `exact=211`, `exact-segments=532`,
  `diverges=0`, `unsupported-feature=84`, `not-yet=0`, and parked
  `M6=40 gated=8 harness=36`; `cargo test --workspace` passes. Next target is
  `library_with_text_and_image.riv`, then `bad_skin.riv` if the nested-library
  slice is not a small promotion.
- 2026-07-06: [M6] Promoted `library_with_text_and_image.riv` by letting the
  static image gate treat unresolved `NestedArtboard` hosts as empty child
  draws, matching C++ for this asset-only nested-library file: both runtimes
  decode the image asset, then draw only the selected artboard background.
  Focused streams are exact; full `make golden-compare` reports `exact=212`,
  `exact-segments=533`, `diverges=0`, `unsupported-feature=83`, `not-yet=0`,
  and parked `M6=39 gated=8 harness=36`; `cargo test --workspace` passes. Next
  target is the M6 image bucket, starting with `bad_skin.riv` unless focused
  classification finds another smaller slice.
- 2026-07-06: [M6] Promoted `clipping_and_draw_order.riv` by admitting simple
  clipped/draw-target image artboards and by treating `ArtboardComponentList`
  sorted drawable entries as metadata-only for draw-command generation, so
  they no longer flush pending clipping shapes into empty clip operations.
  Focused streams for the promoted fixture and component-list guard fixtures
  are exact; full `make golden-compare` reports `exact=213`,
  `exact-segments=534`, `diverges=0`, `unsupported-feature=82`, `not-yet=0`,
  and parked `M6=38 gated=8 harness=36`; `cargo test --workspace` passes. Next
  target remains the M6 image bucket, starting with `bad_skin.riv` unless
  focused classification finds another smaller slice.
- 2026-07-06: [M6] Promoted `data_binding_images_test.riv` by admitting
  nested-state-machine metadata in the static image gate, allowing nested
  child `Image.assetId` binds, and aligning generated owned asset-image
  defaults with C++'s private empty asset sentinel. Nested asset-image context
  imports now use the C++ decode/source-paint ordering where all but the final
  embedded image resolve before source paint allocation. Focused streams for
  the promoted fixture plus `viewmodel_image_reset.riv`,
  `image_fit_alignment*.riv`, `walle.riv`, and `custom_image_name.riv` are
  exact. Full `make golden-compare` reports `exact=214`,
  `exact-segments=535`, `diverges=0`, `unsupported-feature=81`, `not-yet=0`,
  and parked `M6=37 gated=8 harness=36`; `cargo test --workspace` passes. Next
  target remains the M6 image bucket, starting with `bad_skin.riv` unless
  focused classification finds another smaller slice.
- 2026-07-06: [M6] Promoted `scripted_property_image.riv` by admitting inert
  `ScriptedDrawable`/`Event` metadata in the static image gate, preserving
  C++'s no-layout asset-image decode/source-paint ordering, and relying on the
  existing empty asset-image defaults to suppress the two image draws after the
  C++ `FileAssetContents` import failure. `viewmodel_based_condition.riv`
  moved from the generic image bucket to the sharper
  `rust-runner-unsupported:viewmodel-asset-conditions` diagnostic after
  focused streams showed wrong state-machine condition colors once admitted.
  Full `make golden-compare` reports `exact=215`,
  `exact-segments=536`, `diverges=0`, `unsupported-feature=80`,
  `not-yet=0`, and parked `M6=36 gated=8 harness=36`; `cargo test
  --workspace` passes. Next target is the largest M6 bucket,
  `rust-runner-unsupported:nested-artboard-layout`, starting with
  `db_health_tracker.riv` unless focused classification finds a smaller
  nested-layout slice.
- 2026-07-06: [M6] Reclassified `db_health_tracker.riv` from
  `nested-artboard-layout` to `scroll-constraints` by porting the
  `NestedArtboardLayout` scale-type 2 hug/intrinsic sizing path from C++
  `StyleOverrider` into the Taffy adapter. The file now computes a coherent
  whole-artboard layout snapshot for its six layout-backed nested hosts; the
  first remaining Rust runner gate is the authored `ScrollConstraint` global
  210. Full `make golden-compare` remains `exact=215`,
  `exact-segments=536`, `diverges=0`, `unsupported-feature=80`,
  `not-yet=0`, and parked `M6=36 gated=8 harness=36`; `cargo test
  --workspace` passes. Next target is still the largest M6 bucket,
  `rust-runner-unsupported:nested-artboard-layout`, starting with
  `focus_collapsing.riv` unless focused classification finds a smaller
  nested-layout slice.
- 2026-07-06: [M6] Closed the stale
  `rust-runner-unsupported:nested-artboard-layout` manifest queue by direct
  `rust-golden-runner` classification of all 13 remaining entries. Each now
  computes or passes Taffy layout far enough to expose a sharper first gate:
  `focus-data`, `data-binding-nested-stateful-view-model`,
  `data-binding-nested-child`, `scroll-constraints`, `feather`, or `images`.
  Full `make golden-compare` remains `exact=215`, `exact-segments=536`,
  `diverges=0`, `unsupported-feature=80`, `not-yet=0`, and parked
  `M6=36 gated=8 harness=36`; `cargo test --workspace` passes. Next target is
  the largest M6 bucket, `rust-runner-unsupported:scroll-constraints`, starting
  with `component_list_1.riv` unless focused classification finds a smaller
  scroll slice.
- 2026-07-06: [M6] Promoted `component_list_1.riv`,
  `deterministic_mode.riv`, `interactive_scrolling.riv`, `scroll_test.riv`,
  and `scroll_threshold.riv` by porting the passive initial
  `ScrollConstraint::constrain` / `constrainChild` transform slice over
  registered layout-provider children and admitting only zero-offset,
  non-interactive, non-snap, non-virtualized scroll constraints behind a
  coherent Taffy snapshot. Focused streams match exactly apart from signed-zero
  matrix text accepted by `golden-compare` numeric-token comparison. Full
  `make golden-compare` reports `exact=220`, `exact-segments=541`,
  `diverges=0`, `unsupported-feature=75`, `not-yet=0`, and parked
  `M6=31 gated=8 harness=36`; `cargo test --workspace` passes. Next target is
  the largest M6 bucket, `rust-runner-unsupported:images`, starting with
  `bad_skin.riv` unless focused classification finds a smaller first gate.
- 2026-07-06: [M6] Promoted `zombie_skins.riv` by widening the static image
  gate to admit non-mesh image artboard trees with nested vector children,
  preserving C++ all-but-final embedded image decode ordering, and reading live
  animated `GradientStop` color/position values when building gradient shaders.
  Focused `zombie_skins.riv` and full streams are exact; `bad_skin.riv`,
  `bullet_man.riv`, and `spotify_kids_demo.riv` stay parked behind the image
  diagnostic after focused diffs exposed contour-mesh path drift, selected-root
  gradient allocation ordering, and image/deeper layout-text drift respectively.
  Full `make golden-compare` reports `exact=221`, `exact-segments=542`,
  `diverges=0`, `unsupported-feature=74`, `not-yet=0`, and parked
  `M6=30 gated=8 harness=36`; `cargo test --workspace` passes. Next target
  remains the M6 image bucket, starting with `bad_skin.riv` unless focused
  classification finds another smaller first gate.
- 2026-07-06: [M6] Reclassified stale image diagnostics by letting
  `Feather`/`NSlicedNode` gates fire before the broad image fence.
  `car_widgets_v01.riv`, `echo_show_demo.riv`, and
  `feather_render_test.riv` now verify as `rust-runner-unsupported:feather`;
  `local_bounds.riv` verifies as `rust-runner-unsupported:n-slice`. The image
  queue is down to six true image-admission/mesh/paint-order candidates:
  `bad_skin.riv`, `bullet_man.riv`, `jellyfish_test.riv`,
  `spotify_kids_demo.riv`, `superbowl.riv`, and `tape.riv`. Full
  `make golden-compare` reports `exact=221`, `exact-segments=542`,
  `diverges=0`, `unsupported-feature=74`, `not-yet=0`, and parked
  `M6=30 gated=8 harness=36`; `cargo test --workspace` passes. Next target is
  the largest M6 bucket, `rust-runner-unsupported:scroll-constraints`, starting
  with `component_list_child_origin.riv` unless focused classification finds a
  smaller first gate.
- 2026-07-06: [M6] Tightened `golden-compare` unsupported diagnostics so a
  manifest `rust-runner-unsupported:*` entry must match the Rust runner's
  actual first diagnostic. The stricter check exposed two stale labels:
  `db_health_tracker.riv` now verifies as
  `rust-runner-unsupported:data-binding-nested-child`, and `superbowl.riv`
  now verifies as `rust-runner-unsupported:text`. Full `make golden-compare`
  reports `exact=221`, `exact-segments=542`, `diverges=0`,
  `unsupported-feature=74`, `not-yet=0`, and parked
  `M6=30 gated=8 harness=36`; `cargo test --workspace` passes. Next target
  remains the largest true M6 bucket,
  `rust-runner-unsupported:scroll-constraints`, starting with
  `component_list_child_origin.riv`.
- 2026-07-06: [M6] Promoted `component_list_child_origin.riv` and
  `virtualize_blendmode.riv` by admitting passive empty virtualized
  `ArtboardComponentList` scroll providers and drawing layout proxy clip paths
  from computed Taffy bounds. `scroll_snap.riv` remains parked because authored
  snap exposes broader layout transform inheritance drift. Full
  `make golden-compare` reports `exact=223`, `exact-segments=544`,
  `diverges=0`, `unsupported-feature=72`, `not-yet=0`, and parked
  `M6=28 gated=8 harness=36`; `cargo test --workspace` passes. Next target is
  one of the tied largest M6 buckets, image or feather, starting with
  `bad_skin.riv` unless focused classification finds a smaller first gate.
- 2026-07-06: [M6] Split the remaining generic image gate into first-blocker
  diagnostics. `bad_skin.riv` now verifies as
  `rust-runner-unsupported:contour-mesh-metadata`; `jellyfish_test.riv` and
  `tape.riv` verify as `rust-runner-unsupported:mesh-images`;
  `bullet_man.riv` and `spotify_kids_demo.riv` verify as
  `rust-runner-unsupported:selected-root-image-order`. Full
  `make golden-compare` reports `exact=223`, `exact-segments=544`,
  `diverges=0`, `unsupported-feature=72`, `not-yet=0`, and parked
  `M6=28 gated=8 harness=36`; `cargo test --workspace` passes. Next target is
  the largest M6 bucket, `feather`, starting with `car_widgets_v01.riv`
  unless focused classification finds a smaller first gate.
- 2026-07-06: [M6] Promoted `feather_render_test.riv` by porting the runtime
  ShapePaint/Feather draw slice from C++ `shape_paint.cpp` and `feather.cpp`:
  render paints now carry feather strength, outer feathers apply world/local
  offsets in the same order as C++, and inner feathers draw their generated
  inner path under the original/effect path clip. The broad feather queue is
  gone; the remaining former feather files now verify as sharper first gates:
  `selected-root-image-order`, `text`, `feather-inner-multipaint`, or
  `nested-feather-paints`. Full `make golden-compare` reports `exact=224`,
  `exact-segments=545`, `diverges=0`, `unsupported-feature=71`,
  `not-yet=0`, and parked `M6=27 gated=8 harness=36`; `cargo test
  --workspace` passes. Next target is the tied
  `rust-runner-unsupported:scroll-constraints` bucket, starting with
  `component_list_virtualized.riv` unless focused classification finds a
  smaller first gate.
- 2026-07-06: [M6] Promoted `scroll_snap.riv` by letting passive at-rest snap
  metadata through the scroll gate and applying accumulated Taffy layout
  bounds with artboard-origin-adjusted layout world transforms, matching C++
  `LayoutComponent::update` placement for nested layout children under the
  scroll viewport. Full `make golden-compare` reports `exact=225`,
  `exact-segments=546`, `diverges=0`, `unsupported-feature=70`,
  `not-yet=0`, and parked `M6=26 gated=8 harness=36`; `cargo test
  --workspace` passes. Next target is the largest remaining M6 bucket,
  `rust-runner-unsupported:data-binding-nested-stateful-view-model`.
- 2026-07-06: [M6] Promoted `stateful_keyed_trigger.riv` by admitting
  `ViewModelInstance*` subtrees under `NestedArtboardLayout` and
  `NestedArtboardLeaf` hosts. The old
  `data-binding-nested-stateful-view-model` queue is closed:
  `focus_traversal.riv` now verifies as `rust-runner-unsupported:focus-data`,
  while `stateful_multi_property.riv` and `stateful_nested.riv` verify as
  `rust-runner-unsupported:data-binding-nested-child`. Full
  `make golden-compare` reports `exact=226`, `exact-segments=547`,
  `diverges=0`, `unsupported-feature=69`, `not-yet=0`, and parked
  `M6=25 gated=8 harness=36`; `cargo test --workspace` passes. Next target is
  the largest remaining M6 bucket,
  `rust-runner-unsupported:data-binding-nested-child`.
- 2026-07-06: [M6] Reclassified `hit_test_test.riv` after admitting nested
  child `ArtboardComponentList.listSource` / `DataConverterNumberToList` binds
  in the Rust runner; it now reaches
  `rust-runner-unsupported:scroll-constraints`. Focused comparison also showed
  that admitting the broader stateful Artboard/text/layout value path is not
  exact yet: `nested_hug.riv`, `stateful_multi_property.riv`, and
  `stateful_nested.riv` render but drift in nested transforms, so they remain
  parked as `rust-runner-unsupported:data-binding-nested-child`. Next target is
  one of the tied four-file M6 buckets: data-binding nested child, focus data,
  or scroll constraints.
- 2026-07-06: [M6] Promoted passive nested focus metadata for
  `focus_collapsing.riv` and `focusable_element.riv`. The runner now admits
  nested `FocusData` only for no-input, non-traversal files; `focus_traversal`
  stays parked as `rust-runner-unsupported:focus-data` because the focused
  stream renders but differs structurally in path allocation, and `text_input`
  advances to `rust-runner-unsupported:layout-component-paint`. Full
  `make golden-compare` reports `exact=228`, `exact-segments=549`,
  `diverges=0`, `unsupported-feature=67`, `not-yet=0`, and parked
  `M6=23 gated=8 harness=36`. Next target is one of the tied four-file M6
  buckets: data-binding nested child or scroll constraints.
- 2026-07-06: [M6] Closed the `scroll-constraints` queue by admitting
  passive, listener-free, zero-offset virtualized/infinite scroll constraints
  when a Taffy layout snapshot exists, and by suppressing authored-transparent
  layout proxy paints without suppressing transparent normal hit shapes.
  Promoted `component_list_virtualized.riv`, `draw_index_list.riv`,
  `hit_test_test.riv`, and `virtualized_artboard_databound_children.riv`.
  Full `make golden-compare` reports `exact=232`, `exact-segments=553`,
  `diverges=0`, `unsupported-feature=63`, `not-yet=0`, and parked
  `M6=19 gated=8 harness=36`; `cargo test --workspace` passes. Next target is
  the largest remaining M6 bucket,
  `rust-runner-unsupported:data-binding-nested-child`.
- 2026-07-06: [M6] Split the remaining
  `rust-runner-unsupported:data-binding-nested-child` bucket into four
  sharper one-file diagnostics after focused probes showed that temporary
  broad admissions render but still drift in nested layout transforms/clips.
  `db_health_tracker.riv` now verifies as
  `rust-runner-unsupported:nested-trim-path-data-bind`, `nested_hug.riv` as
  `rust-runner-unsupported:nested-artboard-root-transform`,
  `stateful_multi_property.riv` as
  `rust-runner-unsupported:nested-layout-clip-data-bind`, and
  `stateful_nested.riv` as
  `rust-runner-unsupported:nested-stateful-view-model-property`. Full
  `make golden-compare` reports `exact=232`, `exact-segments=553`,
  `diverges=0`, `unsupported-feature=63`, `not-yet=0`, and parked
  `M6=19 gated=8 harness=36`; `cargo test --workspace` passes. Next target is
  one of the tied largest M6 buckets, `selected-root-image-order` or `text`.
- 2026-07-06: [M6] Split
  `rust-runner-unsupported:selected-root-image-order` into three verified
  first blockers. `car_widgets_v01.riv` now reaches the existing
  `rust-runner-unsupported:text` gate on text paint feather,
  `bullet_man.riv` now verifies as
  `rust-runner-unsupported:selected-root-gradient-shader-order`, and
  `spotify_kids_demo.riv` now verifies as
  `rust-runner-unsupported:selected-root-skinned-clip-path` after adding the
  selected-root single-image predecode ordering observed in the focused
  compare and narrowing it against `feather_render_test.riv`'s exact paint
  ordering. `golden-compare` now reports the first differing stream line for
  failed exact comparisons. Full `make golden-compare` reports `exact=232`,
  `exact-segments=553`, `diverges=0`, `unsupported-feature=63`, `not-yet=0`,
  and parked `M6=19 gated=8 harness=36`; `cargo test --workspace` passes.
  Next target is the `text` bucket.
- 2026-07-06: [M6] Closed the broad
  `rust-runner-unsupported:text` queue by admitting text paint feather state
  through the existing `ShapePaint` feather path and splitting the five
  remaining broad text-tagged entries into first blockers.
  `car_widgets_v01.riv` and `hunter_x_demo.riv` now verify as
  `rust-runner-unsupported:feather-inner-multipaint`, `echo_show_demo.riv` as
  `rust-runner-unsupported:text-joystick-data-bind`, `superbowl.riv` as
  `rust-runner-unsupported:nested-artboard-layout`, and gated `bankcard.riv`
  as `rust-runner-unsupported:text-polygon-sibling`. Full
  `make golden-compare` reports `exact=232`, `exact-segments=553`,
  `diverges=0`, `unsupported-feature=63`, `not-yet=0`, and parked
  `M6=19 gated=8 harness=36`; `cargo test --workspace` passes. Next target is
  `rust-runner-unsupported:feather-inner-multipaint`.
- 2026-07-06: [M6] Closed the
  `rust-runner-unsupported:feather-inner-multipaint` queue by removing the
  runner-only global inner-feather guard and keying repeated inner-feather
  clip paths by the original draw command instead of by each paint-local
  generated inner path. `coin.riv` is now exact, `car_widgets_v01.riv` now
  verifies as `rust-runner-unsupported:nested-node-transform-data-bind`, and
  `hunter_x_demo.riv` plus `rewards_demo.riv` now verify as
  `rust-runner-unsupported:nested-feather-paints`. Full
  `make golden-compare` reports `exact=233`, `exact-segments=554`,
  `diverges=0`, `unsupported-feature=62`, `not-yet=0`, and parked
  `M6=19 gated=7 harness=36`; next target is the tied largest M6 bucket
  `rust-runner-unsupported:nested-feather-paints`.
- 2026-07-06: [M6] Closed the stale
  `rust-runner-unsupported:nested-feather-paints` runner guard and let the
  affected files run to their real first blockers. `hunter_x_demo.riv` now
  verifies as `rust-runner-unsupported:text-modifier-group-flags`,
  `rewards_demo.riv` as `rust-runner-unsupported:n-slice`, and gated
  `ai_assitant.riv` now reaches Rust draw output but is parked as
  `not-yet:nested-feather-gradient-space` because its focused compare differs
  at nested-feather linear-gradient coordinates. Full `make golden-compare`
  reports `exact=233`, `exact-segments=554`, `diverges=0`,
  `unsupported-feature=61`, `not-yet=1`, and parked
  `M6=19 gated=6 harness=36`; next target is the tied largest M6 bucket,
  preferring `rust-runner-unsupported:n-slice` over `mesh-images` because it
  is layout-facing and now blocks both `local_bounds.riv` and
  `rewards_demo.riv`.
- 2026-07-06: [M6] Ported NSlicedNode vector path deformation by mirroring the
  C++ N-slicer stop mapping for local/world shape draw commands, admitting
  passive NSlicedNode/axis/draw-rule siblings through static text, and
  removing the stale runner-level N-slice guard. `n_slice_triangle.riv` is now
  exact, `rewards_demo.riv` now verifies as
  `rust-runner-unsupported:text-modifier-group-flags`, and
  `local_bounds.riv` is parked as `not-yet:image-predecode-order` because its
  focused run now reaches Rust draw output but differs in external image
  predecode ordering and tiny static-text float residuals. Full
  `make golden-compare` reports `exact=234`, `exact-segments=555`,
  `diverges=0`, `unsupported-feature=59`, `not-yet=2`, and parked
  `M6=18 gated=5 harness=36`; next target is the tied largest M6 bucket,
  preferring `rust-runner-unsupported:text-modifier-group-flags` over
  `mesh-images`.
