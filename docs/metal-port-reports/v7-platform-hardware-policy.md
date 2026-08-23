# V7 — platform and hardware policy

Status: GREEN on 2026-08-22.

The nine-lane compile/configuration matrix is recorded in V2. Both source-policy regressions also passed on the final bytes:

- `selects_upstream_metal_capabilities_by_platform_and_device_family`: 1/1.
- `four_atomic_draw_groups_apply_exact_upstream_barrier_policy`: 1/1.

The capability table covers Apple Silicon, Intel Common2, AMD Mac2, old-macOS pass-break policy, iOS device, and simulator host-architecture selection. The barrier test covers raster-order, five explicit memory barriers, and five render-pass breaks. Live Intel/discrete execution is unavailable on this Apple Silicon host; its prescribed x86_64 compile row executed successfully rather than disappearing from the matrix.
