# P3A2 sandbox handoff map

The provided worktree is `/Users/levi/dev/worktrees/nuxie-mr-c13`. Its Git
metadata is outside the writable sandbox, so this worker could not create or
commit `levi/p3a-focus-tree-v2` there. The complete committed branch is in:

`/tmp/nuxie-p3a-v2.KlWl4s`

It is based on exact current-main commit
`cc7effd0f8043abf0db925cea7ba4e6c19153537`, not on that temporary clone's
synthetic remote-tracking name.

When Git metadata is writable, import the committed branch without replaying
scratch files:

```sh
cd /Users/levi/dev/worktrees/nuxie-mr-c13
git fetch /tmp/nuxie-p3a-v2.KlWl4s \
  refs/heads/levi/p3a-focus-tree-v2:refs/heads/levi/p3a-focus-tree-v2
git switch levi/p3a-focus-tree-v2
git merge-base --is-ancestor \
  cc7effd0f8043abf0db925cea7ba4e6c19153537 HEAD
git log --oneline --no-merges cc7effd0..HEAD
```

Expected committed series before the final report commit:

```text
6bb8d7b4 [P3-a] Pin retained focus coordinator docs
7dd319a8 [P3-a] Attribute retained focus facades
84651c5d [P3-a] Fix retained focus domain sharing
38b74b2b [P3-a] Reconcile retained focus gate ownership
ab373e1f [P3-a] Promote retained focus correspondence
6ae848c1 [P3-a] Preserve focus manager switch API
55bd7181 [P3-a] Reconcile focus dirty cache with retained tree
678d1542 [P3-a] Rebuild retained focus tree on current main
```

The final report commit follows this series. The provided worktree also contains
the exact final P3A-owned files, but not every unrelated file introduced between
the old lane base and current main. If the temporary clone is unavailable,
create the branch at `cc7effd0`, preserve the mirrored P3A files, and commit the
worktree state with `[P3-a]` provenance.
