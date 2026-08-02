Implemented UNIV-1291 and left the worktree uncommitted/unmerged.

Key outcomes:

- `TextStylePaint` dirt now routes through FL-E7 and rebuilds retained paints during component update.
- Unchanged glyph paths remain retained; the FL-E7 ratchet is preserved.
- Draw-time failure is now an invariant assertion.
- Public test covers initial binding, cursor write, and repeated VM writes.
- C++ differential verifies actual rendered paint colors, property values, and path counts.

All requested gates pass:

- `cargo test -p nuxie-runtime` — 915 passed, one ignored.
- `cargo test -p nuxie --features scripting` — 475 passed.
- `make cpp-probe` — passed using a sandbox-safe local output directory.
- Ordinary goldens — 320/320 entries, 652/652 segments exact.
- Scripted goldens — 320/320 entries, 652/652 segments exact.
- Formatting and diff checks pass.
- Final standards/spec reviews are clean.

Full details: [U1291-report.md](/Users/levi/dev/worktrees/nuxie-fld1/U1291-report.md)

The branch remains 19 commits behind local `origin/main`. Fetch was attempted but blocked by sandbox access to shared worktree Git metadata; no merge was performed.