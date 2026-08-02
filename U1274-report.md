Implemented UNIV-1274’s read-only observation surface and silver replay support. No commit was created.

- Runtime, borrowed/owned artboards, and `Scene` expose occurrence-scoped ScrollConstraint snapshots.
- Snapshots include offsets, bounds, clamped offsets, physics presence, and exact running state.
- All six cases now execute and are honestly classified as divergences.
- Exact ratchet remains 76; runtime summary is 93 divergences and 26 unsupported.
- `layout_scroll_snap_carousel` remains separately unsupported.

All requested gates passed, including `make golden-compare` at 319/319 exact. Standards and spec reviews found no remaining issues.

Full per-case results and pinned C++ citations: [U1274-report.md](/Users/levi/dev/worktrees/nuxie-p1a-binary/U1274-report.md).