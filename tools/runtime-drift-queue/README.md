# Runtime drift queue

This tool turns the checked-in parity ledgers into one deterministic investigation queue. It does not fix or reclassify runtime behavior. Every source row is either emitted once as a candidate or counted as proven in the JSON `accounting` partition.

Inputs include owner proofs, upstream test correspondence, ordinary/scripted golden status, silver status, tracked gap rows, deliberate decisions, and additive extensions. Candidate IDs are source-stable (`owner:…`, `test:…`, `golden:…`, `silver:…`, `gap:…`, `decision:…`, `extension:…`). Clusters group candidates by upstream owner family and first semantic boundary without merging those IDs.

Regenerate and verify the checked-in queue:

```sh
make runtime-drift-queue-snapshot
make runtime-drift-queue
```

The JSON exposes deterministic filters for `owner_family`, `subsystem`, `evidence_state`, and `disposition`; candidates are ordered by descending `discovery_value` and then stable ID. Churn is the aggregate touch count for mapped Rust and checked-in evidence paths over the last 100 commits (`low`/`medium`/`high`). Freshness and churn both affect discovery value.

Fresh `nuxie-runtime-differentials/v1` artifacts can enrich manifest candidates or surface a new exact-case regression:

```sh
python3 tools/runtime-drift-queue/drift_queue.py build \
  --repo-root . \
  --differential-dir target/runtime-differentials \
  --output target/runtime-drift-queue.json \
  --markdown-output target/runtime-drift-queue.md
```

An artifact is accepted only when its C++ ref matches the owner-proof pin, its Rust commit matches `HEAD`, its lane is known, it accounts for every manifest row, and its manifest, runner, fixture, dependency, script, and baseline provenance records are well formed. Stale artifacts are reported but do not change candidates. Unexecuted rows never replace checked-in evidence or claim a current runtime observation.
