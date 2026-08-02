Implemented the scoped P2-d scripted-interpolator port on `levi/p2d-scripted-interpolator`.

Key results:

- Lua `transform` / `transformValue` callbacks with pinned numeric coercion.
- Per-animation/per-keyframe stateful Lua instances.
- Shared definition-level apply path.
- Artboard context, input hydration-before-init, fallback, and bounded diagnostics.
- Exact golden fixture covering both callbacks, mutable state, init, and inputs.
- Golden corpus: 321 entries, 657 segments, zero divergences.
- Ledger unchanged at 85 pending; ratchet not loosened.
- Manifest honestly remains `pending` / `TRACKED-GAP` for the broader generic cloned-DataBind lifecycle.
- All requested gates passed.
- `Cargo.lock` unchanged; no commit created.

Full handoff: [P2D-report.md](/Users/levi/dev/worktrees/nuxie-p2f-audio/P2D-report.md).