# F3CASE sandbox audit map

The sandbox intermittently denied creation of the external worktree Git index
lock. Each implementation/review-fix commit was subsequently recorded, so no
source hunk remains stranded. The final evidence commit contains:

## `[F3CASE] Record lane completion evidence`

- `F3CASE-report.md`: add the full lane status, gate evidence, commit list,
  residue/correspondence disposition, and queued F6/S4/orchestrator work.
- `F3CASE-map.md`: retain this audit of the transient commit blocker and
  gitignored fixture disposition.

Both files match repository ignore patterns and therefore require forced
staging:

```text
git add -f F3CASE-report.md F3CASE-map.md
git commit -m '[F3CASE] Record lane completion evidence'
```

Fixture note: the required `rsync` refresh populated gitignored fixture
directories. The durable fixture additions are represented by the committed
fetch-script URL/hash entries (`1e1b93ef`). If the landing orchestrator chooses
to vendor the downloaded binaries, they must use `git add -f fixtures/...`;
otherwise the ignored copies should remain workspace-only.
