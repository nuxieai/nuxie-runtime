Completed P3-g with a clean worktree.

- Factory and renderer rows promoted to faithful candidates, still `pending-verification`.
- Scorecard: 425 faithful / 19 pending; misc-core now only `core.cpp` and `focus_data.cpp`.
- Both mechanical ledgers, B6 evidence, attribution, and scorecard are consistent.
- Focused tests and both touched-crate `cargo check`s pass.
- No full workspace or golden batteries were run.
- Review found no remaining spec issues; the crate-root C17 ownership exception remains an explicit judgment call.

Commits:

- `48eef3cd` Close factory and renderer seam helpers
- `acca108f` Promote factory and renderer correspondence
- `1f7887d0` Preserve joined shaping clusters

Full evidence, pending rows, and conflict queue: [P3G-report.md](/Users/levi/dev/worktrees/nuxie-mr-c17/P3G-report.md).