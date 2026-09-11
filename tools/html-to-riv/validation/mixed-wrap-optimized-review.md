# Optimizing geometry-selected paint groups

The optimized ordinary-file composition preserves the tested Chrome behavior while roughly halving generated objects and substantially reducing import/clone cost. It remains experimental and quadratic. No runtime, renderer, schema, dependency or public compiler changes were made.

The previous axes-aware construction emitted every item in every potential line group. In logical packing order, a group led by item i cannot contain an earlier item j<i. The optimized experiment skips those candidates, reducing paint replicas from N² to T=N(N+1)/2. An adjacent pair's difference signal already used by membership is reused to detect the next line leader. All signals sharing a source anchor reuse its two-record copy Node. Membership masks themselves are not merged across different anchor pairs. The optimization preserves ordinary constraints and masks, rather than adding runtime CSS or a host callback.

## Exact object count

Let M=N(N−1)/2 and N≥1. Source anchor copies are shared across ordered pairs, and the N−1 adjacent leader signals are among the M membership signals.

| Added objects | Records |
|---|---:|
| Common origin | 1 |
| Transform anchors (Node + TransformConstraint) | 2N |
| Shared source-anchor copies | 2(N−1) |
| Remaining difference-signal objects | 5M |
| Membership rectangle masks | 2M |
| Leader inversion + rectangle masks | 4(N−1) |
| Two families of clipping records | 2M |
| Foreground + Fill + color replicas | 3T |
| DrawRules + DrawTargets | 2(T−1) |
| **Total** | **7N²+6N−7** |

N=3/8/16/32 now adds 74/489/1,881/7,353 records. The previous axes-aware formula is 16N²+2N−12 (138/1,028/4,116/16,436). The earlier resource audit used equal-height top-anchor scenes without the extra 2N Transform-anchor records; its historical formula 16N²−12 is still correct for that version. The optimization comparisons below use those same historical scenes and unchanged authored geometry. Empty containers are outside the formula; no composition is needed. One painted descendant per item is assumed, and logical packing order is supplied directly by these fixtures. Additional paint descendants and reordered authored DOM require separate construction/evidence.

## Correctness and visual evidence

`output/mixed-wrap-optimized-r1` reuses all 16 N3 axis/wrap/unequal-height/positional-wrapper alpha cases. All 128 geometry and pixel frames and 256 clear controls pass. Every Chrome and native decoded image is identical to the corresponding prior axes-run image, preserving its earlier visual evidence. Sixteen exact RIV/map reproductions pass.

`output/mixed-wrap-optimized-n8-r1` adds eight scenes spanning row, row-reverse, column and column-reverse × wrap/wrap-reverse, with eight items and overlapping translucent descendants. Three viewport sizes cover one line, several lines and one item per line; the same scene is returned to the first size and cloned. All 64 geometry and pixel frames and 128 clear controls pass. All 24 distinct pairs were directly inspected (twelve sheets), with no visible differences in placement or alpha layering; 40 exact full-RGBA repeat/clone transfers cover the rest. Eight exact RIV/map reproductions pass. N8 cross metrics are equal; N3 covers the unequal/wrapper contexts. No tolerance changed.

These are adapter experiments. The generic driver's public status label does not constitute public syntax admission; the experiment manifests explicitly correct it. The N3 adapter uses authored/transformed fixture data; N8 generates the same one-painted-descendant family without browser coordinates. The resulting scene performs wrapping and paint selection when resized without recompilation.

## Paired lifecycle benchmark

`output/mixed-wrap-optimized-resource-r1` uses the exact previous N8/16/32 authored raw files in both mixed-flow directions. A stock immutable probe verifies eight frames per optimized scene. Every observed layout record matches the previous construction exactly. Three old/new timing trials per scene use the previously bound observer and runtime libraries; each trial's geometry and command stream exactly matches its corresponding stock observation. Streams across old/new constructions are not claimed identical because paint objects and clipping commands changed.

Times are medians in milliseconds; peak RSS is the maximum process resident set across trials, including original/clone and recorded streams. Changed-size steps determine resize timing; record draw is CPU command recording, not Metal rasterization. Hardware is the same M5 Max and immutable debug-baseline build as the previous resource audit. Local paired observations are not release/mobile guarantees or isolated workload measurements.

| N | Flow | Version | Import ms | Clone ms | Resize ms | Record draw ms | Peak RSS MiB |
|---:|---|---|---:|---:|---:|---:|---:|
|8|wrap|old|8.91|6.78|1.39|1.32|25.03|
|8|wrap|optimized|3.91|2.87|0.76|0.73|21.84|
|8|wrap-reverse|old|8.57|6.79|1.38|1.35|25.09|
|8|wrap-reverse|optimized|3.87|2.81|0.75|0.73|22.02|
|16|wrap|old|66.35|60.76|5.21|5.48|44.88|
|16|wrap|optimized|21.64|18.97|2.52|2.86|33.12|
|16|wrap-reverse|old|67.22|61.09|5.20|5.50|46.03|
|16|wrap-reverse|optimized|21.46|18.61|2.48|2.90|32.89|
|32|wrap|old|811.71|789.60|20.68|22.30|118.41|
|32|wrap|optimized|208.38|198.46|9.40|11.46|68.05|
|32|wrap-reverse|old|813.37|795.91|20.59|22.28|118.11|
|32|wrap-reverse|optimized|207.52|200.73|9.58|11.44|67.25|

The optimized N8 files are about 6.0 KB, N16 21.4 KB and N32 81.2 KB. At N32, import and clone fall from about 812/790 ms to 208/199 ms; changed-size update plus recording falls from roughly 43 ms to 21 ms and RSS from about 118 MiB to 68 MiB. This remains over a hypothetical 16.7 ms frame budget before GPU work on this high-end debug target. N16's roughly 5.4 ms update-plus-recording cost and 21.5/18.8 ms import/clone are better bounded candidate measurements, but not a product latency commitment.

The improvement supersedes cost estimates only for this specific optimized construction. It does not remove the need to budget expansion across a whole scene, account for arbitrary paint trees, test target devices, and report exceeded limits honestly. No public resource guard was installed and no broader pixel guarantee is inferred from the larger lifecycle-only scenes. Previous experiments and failed candidates remain intact.
