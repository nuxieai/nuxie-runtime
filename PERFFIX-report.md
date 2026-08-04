# Performance-parity fix report

Date: 2026-08-04

Branch: `levi/perf-opacity-owners`

Pinned C++ runtime: `/Users/levi/dev/oss/rive-runtime` at `4ac7b327`

## Measurement protocol

All measurements use the `docs/perf-size-evidence.md` method: ordinary release
runners, 100 sequential frames from `0/60` through `99/60`, five fresh
iterations, zero warmups, median aggregation, C++ first, and the runner's
combined `advance + draw` metric divided by 100.

## Results

| Step | Change | `car_widgets_v01` Rust ms/frame | `zombie_skins` Rust ms/frame |
|---|---|---:|---:|
| Baseline | Source `3f94fe1f` | 69.710143 | 1.361694 |
| Fix 1 | Retain occurrence-local opacity-owner to paint-container indices | 8.861663 | 0.912388 |
| Fix 2 | Gate renderer tree attachment/seeding on the mounted structure epoch | 6.895138 | 0.590826 |
| Fix 3 | Reuse the retained preparation key as the clean draw boundary | 6.833802 | 0.582881 |

Fix 1 replaces per-dirty-owner scans of all static shape-paint containers and
their parent chains with an ordered index retained on each `RuntimeShapeList`.
The index is seeded once from the graph and cloned with the occurrence; the
existing owner-local opacity propagation and dirt behavior are unchanged.

Fix 2 records the mounted occurrence-tree structure epoch independently for
image attachment and opacity seeding. Repeated synchronization now performs
neither tree walk until structure changes, and opacity seeding consumes only
the retained pending-owner queue at each occurrence. Compared with fix 1,
`advance + draw` fell another 22.19% on `car_widgets_v01` and 35.24% on
`zombie_skins`.

Fix 3 makes `needs_paint_preparation` consult the exact retained preparation
key written after paint realization and traversal. It still invalidates for a
changed graph/occurrence/world/nested epoch or stale owner backend, but the
draw immediately following an explicit prepare no longer synchronizes again.
Compared with fix 2, `advance + draw` fell another 0.89% on
`car_widgets_v01` and 1.34% on `zombie_skins`; the car draw phase fell from
0.078630 to 0.068612 ms/frame.

The branch initially carried a pre-`advance_draw` `perf-compare`; its total
hot-loop output included `prepare`. The comparison tool was brought forward to
the evidence method before the three source revisions were remeasured. The
authoritative raw reports are the `corrected-*` baseline/fix-1 files and the
`fix2-*`/`fix3-*` files in `docs/evidence/perffix-2026-08-04/`.

## Validation

- Retained opacity-owner index regression: green.
- Full scripted golden comparison after fix 1: green and byte-identical at
  363 entries, 342 exact, 16 diverges, 5 not-yet, 1,114 exact segments, and
  1,109 side-channel segments.
- Full scripted golden comparison after fix 2: same byte-identical summary.
- Full scripted golden comparison after fix 3: same byte-identical summary.
- Repeated renderer initialization regression and `perf-compare` phase tests:
  green.
- Clean prepare-to-draw boundary and dirt-invalidation regression: green.
- `cargo test -p nuxie-runtime`: green.
- `cargo test -p nuxie --features scripting`: green.
- `make check`: green.
- Green checkers: `rust-attribution-check`, `runtime-frame-loop-port-check`,
  `perf-runtime-ref-check`, `silver-corpus-manifest-check`,
  `renderer-wgpu-backend-check`, and `renderer-wgpu-consumer-check`.
- Baseline checker debt, unchanged from source `3f94fe1f`:
  `port-manifest-check` fails because the generated `lua_gpu.cpp` note lacks
  register ID `P3E`; `b6-audit-check` requires upstream `d788e8ec` while this
  task explicitly pins the manifest/runtime at `4ac7b327`;
  `runtime-drawing-port-check` names the already-absent
  `fn update_runtime_path_composer` anchor; and `renderer-shaders-check` has
  patches which do not apply to the required `4ac7b327` checkout. This lane
  changes none of those checker inputs.
- `e2e-composed-compare`: not present on local `main` or this branch.
