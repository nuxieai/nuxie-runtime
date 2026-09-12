# Public fixed normal/stretch line distribution

The public compiler now admits omitted align-content, normal and stretch for its fixed-root/direct-fixed-leaf wrapping profile when line distribution is exactly representable. Generated slots expand to each line's cross maximum plus its equal share of positive free space; visible children preserve all original dimensions and bounds. A source-owned Plan independently rechecks used-size clamps, partition, exact LayoutUnit arithmetic and generated slot fields. Existing ordinary runtime layout, position, scalar and paint proofs still run. No browser geometry, runtime modification or new runtime helper is used.

The current arithmetic profile requires every fixed field and intermediate sum within65,536px and positive free space divisible by line count in1/64px LayoutUnits. Nondivisible distribution rejects Remainder. This is a compiler proof boundary, not impossibility. The pinned Chrome source review identifies its exact remainder diffuser and reversal order for the next extension. Center positioning on odd raw-unit differences still needs explicit boundary coverage; neither geometry gates nor exact line distribution claim universally exact CSS geometry.

## Validation

- Frozen product build `output/public-wrapping-stretch-product-build-r1`:474 Rust tests,57 Node tests, native/WASM/TypeScript/source guard pass. CLI SHA25657aa949f48a445d12b17b3f72c2c7687f28a3ddbe312f98700d3b71d322bf532; WASM81af2e12cb314a78c54a5f4570fb8e4528acb1f8fe391faf6353986baadfb9a4.
-24 public CLI fixtures pass192 geometry/pixel frames and384 alternate-clear checks in `output/playwright/public-wrapping-stretch-r1`. Both axes/all directions/wrap modes, three/four unequal lines, normal/stretch/omitted twins, order, min-over-max, visible maximum clamps, single-line and overflow are included. Fixed-root resizing tests clipping/persistence, not responsive reflow. The unchanged driver's legacy private labels do not describe the actual public CLI input.
-72 representative image pairs are on18 full-size review sheets;120 repeated frames transfer through exact RIV/image/geometry identity. Direct review notes are separate.
-New CLI and raw WASM reproduce all1,092 prior public scenes exactly (282image +794transport +16wrapping), including source maps. The earlier128 wrapping captures retain identical file input; no historical rerender is claimed.

## Remaining scope

Remainder distribution, percentages, intrinsic/auto sizes, nested content, assets inside wrapping roots and broader layout semantics remain work. Existing graph costs still apply: compact paint is conditional on forward start-aligned integral contained owners, not on integer dimensions alone. Reverse directions, center/end alignment and fractional geometry can require the expensive fallback. No new lifecycle performance improvement is claimed from stretch slot expansion. L04/L06 stay partial; all99 backlog items remain in scope.
