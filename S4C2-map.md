# S4C2 commit map

The requested commit could not be created because the managed workspace cannot
write the external worktree index:

```text
fatal: Unable to create '/Users/levi/dev/nuxie-runtime/.git/worktrees/nuxie-mr-c11/index.lock': Operation not permitted
```

Nothing is staged. The commit boundary is exactly these two files:

- `crates/nuxie-runtime/src/artboard.rs`
  - strengthen the paused nested-artboard opacity regression with a faithful
    Artboard-root to Shape-content dependency,
  - preserve the S4-23 own-opacity/host-opacity split, and
  - verify propagated content opacity plus complete dirt consumption.
- `S4C2-report.md`
  - diagnosis, upstream/candidate comparison, scope, and gate evidence.

`S4C2-report.md` matches the repository's ignored `*-report.md` pattern and
must be force-added. From a Git-writable shell in this worktree:

```sh
git add crates/nuxie-runtime/src/artboard.rs S4C2-map.md
git add -f S4C2-report.md
git diff --cached --check
git commit -m '[sync] Reconcile e0d4913f with 0a2e478a paused-artboard opacity interaction'
```

Do not stage corpus, golden, fixture, attribution, transform/clip, or generated
files for this interaction resolution.

