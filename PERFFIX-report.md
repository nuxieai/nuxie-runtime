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
| Baseline | Source `3f94fe1f` | 73.478530 | 2.144770 |
| Fix 1 | Retain occurrence-local opacity-owner to paint-container indices | 9.438670 | 1.550700 |

Fix 1 replaces per-dirty-owner scans of all static shape-paint containers and
their parent chains with an ordered index retained on each `RuntimeShapeList`.
The index is seeded once from the graph and cloned with the occurrence; the
existing owner-local opacity propagation and dirt behavior are unchanged.

## Validation

- Retained opacity-owner index regression: green.
- Full scripted golden comparison after fix 1: green and byte-identical at
  363 entries, 342 exact, 16 diverges, 5 not-yet, 1,114 exact segments, and
  1,109 side-channel segments.
- `cargo test -p nuxie-runtime`: pending final lane validation.
- `cargo test -p nuxie --features scripting`: pending final lane validation.
- Checkers: pending final lane validation.
- `e2e-composed-compare`: not present on local `main` or this branch.
