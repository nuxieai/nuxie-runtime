Stopped without modifying `tools/golden-runner`: ASan proves the defect is in pinned upstream runtime `4ac7b327`.

Root cause: `NestedArtboard::nest()` destroys the outgoing `ArtboardInstance` without removing its nodes from `SemanticManager`. The next harness `drainDiff()` dereferences a freed layout component:

```text
SemanticManager::drainDiff()
SemanticManager::refresh()
SemanticManager::reconcileBoundsForSubtree()
LayoutComponentBase::width()  ← heap-use-after-free
```

The instance was freed through:

```text
NestedArtboard::nest()
NestedArtboard::updateArtboard()
DataBindContextValueArtboard::apply()
StateMachineInstance::advanceAndApply()
```

Controls:

- Release: 22 crashes across 80 replays.
- Debug: 40/40 deterministic crashes.
- ASan without `--side-channel`: both fixtures pass.
- ASan with `--side-channel`: both fail.
- Neither fixture uses `--view-model-script`; `applyViewModelEvent` is absent from both traces.

Full evidence and rationale are in [CPPCRASH-report.md](/Users/levi/dev/worktrees/nuxie-p2f-audio/CPPCRASH-report.md).

No 30-replay/full-gate proof was run because no valid harness fix exists without masking or corrupting the semantic oracle. I also could not commit the report: the sandbox denies writes to the worktree’s Git index (`.git/worktrees/nuxie-p2f-audio/index.lock`). The pre-existing `docs/v-row-triage.md` remains untouched.