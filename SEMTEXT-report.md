# Semantic text geometry regression report

Upstream reference: `rive-runtime` at
`4ac7b32798da0482e441ef09304dc3b480ed3ee5`.

## Outcome

`semantic_text_inference_lt1` is exact again. Rust still propagates the
trim-driven `Path`/`WorldTransform` event to semantic data, but semantic bounds
now read the current retained layout-animation frame. At the failing action,
that frame has not changed, so the existing exact bounds-delta gate produces
the pinned C++ result:

```text
semantics updatedGeometry bounds=[]
```

The V37 invalidation fix was not weakened, the semantic side-channel emitters
were not changed, and `text_vertical_trim_test` remains exact.

## Diagnosis

Pinned C++ does not gate this case on a narrower dirt category:

- `src/semantic/semantic_data.cpp:273-293` calls `updateWorldBounds()` for full
  `WorldTransform` or `Path` dirt.
- `src/semantic/semantic_data.cpp:501-531` asks
  `SemanticProvider::semanticBounds(parent)` and returns without marking
  `SemanticDirt::Bounds` when the newly observed bounds equal the semantic
  node's retained bounds.
- `src/semantic/semantic_provider.cpp:13-32,76-94` obtains those bounds from
  `Node::localBounds()` and the live world transform.
- `include/rive/layout_component.hpp:192-211` implements
  `LayoutComponent::localBounds()` with the current `m_layout.width()` and
  `m_layout.height()`.

A focused C++ probe using the pinned runtime and the same scripted action
showed node 8 staying at `(128,753.7,1936,2009.55)`, its layout owner staying
at local `(0,0,1808,1255.85)`, and `updatedGeometry=0` after the action and
advance.

The corresponding pre-fix Rust trace showed that the retained layout animation
was still on its old frame while the solve had published a new target:

```text
current: width=1794, height=1255.8477
target:  width=1794, height=879.89844
old semantic max_y: 2009.5476
pre-fix observed max_y: 1633.5984
```

The owner-pushed semantic dirt was therefore correct. The over-report happened
because both layout semantic-bound paths called `ArtboardInstance::layout_bounds`,
which intentionally exposes the newly solved target, instead of reading
`RuntimeLayoutComponentState::current_bounds`, the Rust counterpart of C++'s
current `m_layout`.

## Fix and regression coverage

- `ArtboardInstance::layout_world_bounds` now maps the current retained layout
  width and height through the current world transform.
- `ArtboardInstance::semantic_layout_world_bounds` uses that same current frame
  for its padded D3 layout adapter.
- `semantic_layout_bounds_use_current_animation_frame_not_solved_target`
  constructs an 80-pixel current frame with a 40-pixel solved animation target
  and requires semantic bounds to remain 80 pixels high. The test failed with
  40 before the fix and passes afterward.

No semantic journal, trim invalidation, corpus classification, or side-channel
emitter was modified.

## Five whys

1. Geometry was reported because the semantic node observed a different box.
2. It observed a different box because the provider read the solved layout
   target.
3. The target was visible because `layout_bounds` is an observation API for the
   latest solve, including a not-yet-advanced animation destination.
4. Pinned C++ did not see that destination because `localBounds()` reads the
   current interpolated `m_layout` frame.
5. Rust had conflated solved-layout observation with current render/semantic
   geometry. The new regression test fixes that boundary explicitly.

## Verification

- Focused `semantic_text_inference_lt1`: exact; 3 draw and 3 side-channel
  segments.
- Focused `text_vertical_trim_test`: exact; 3 draw and 3 side-channel segments.
- `make scripted-golden-compare`: passed; 363 entries, 332 exact, 24 pinned
  divergences, 7 not-yet, zero unexpected regressions.
- `cargo test -p nuxie-runtime`: passed; 961 unit tests plus all integration and
  doc tests.
- `cargo test -p nuxie --features scripting`: passed; all unit, integration,
  and doc-test suites.
- `make rust-attribution-check`: passed.
- `make runtime-frame-loop-port-check`: passed (125 checker tests plus
  correspondence, layout-handler, and ownership checks).
- `git diff --check`: passed.

Three broader checkers have unrelated branch-HEAD failures and were not changed
as part of this regression fix:

- `make port-manifest-check`: its own fixture expects the `P3E` feature id in
  the pre-existing `src/lua/renderer/lua_gpu.cpp` note.
- `make b6-audit-check`: the audit expects upstream
  `d788e8ec6e8b598526607d6a1e8818e8b637b60c`, while this repository is pinned
  to `4ac7b32798da0482e441ef09304dc3b480ed3ee5`.
- `make runtime-drawing-port-check`: the pre-existing ownership ledger points
  `shape.path_composer` at absent anchor `fn update_runtime_path_composer`.
