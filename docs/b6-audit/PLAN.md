# #B-6 Execution Plan — Structural Fidelity Audit (447 rows)

Status: **COMPLETE.** Phases 0–3 completed on 2026-07-21. The
post-RB-1/RD-1 Phase 4 second pass completed on 2026-07-24; see
`SECOND_PASS.md`. `make b6-audit-check` is the permanent closure ratchet.

Roles: a PLANNER session (the parity orchestrator) owns rules, triage, and
remediation decisions. An EXECUTOR session (cheaper model, e.g.
Sonnet-class) performs the mechanical audit sweep described here. The
executor produces evidence records; it never decides remediation.

Authority documents, in order: docs/b6-structural-audit-spec.md (the
validated five-axis brief, schema, validity rules), docs/PORTING.md §8
(the architecture-fidelity rules findings must cite), port-manifest.toml
(the 447-row queue), docs/rb1-compensation-inventory.md (worked example of
a DIVERGENT family — style reference only, never evidence).

## Phase 0 — Cluster partition (executor, first commit)

Build docs/b6-audit/clusters.toml from port-manifest.toml: every manifest
row assigned to exactly one subsystem cluster (data-bind, view-model,
state-machine, animation, layout, text, shapes/paths, constraints,
assets/importers, audio-stubs, misc-core, ...). Rows sharing a
rust_module MUST share a cluster. Multi-file compensation families (see
the five seam files in rb1-compensation-inventory.md) must be co-clustered.
Emit per cluster: name, manifest rows, rust modules touched, planned batch
splits (8-12 pairs; up to 15 for isolated single-module clusters).
Commit before auditing anything.

## Phase 1 — Audit loop (executor)

Work cluster by cluster. For each cluster batch:
1. Read every C++ file in the batch (pin d788e8ec at
   /Users/levi/dev/oss/rive-runtime — READ ONLY, never checkout/build).
2. Read the mapped Rust modules (region-scoped; use search, do not slurp
   10k-line files end to end).
3. Apply the five-axis brief from the spec, with BOTH calibration
   amendments: the mutation-timing gate on axis (e), and the crate-wide
   family grep + sibling sweep before any verdict.
4. Write one record per manifest row into
   docs/b6-audit/results/<cluster>.md using the spec schema, including
   the hard validity rules (DIVERGENT requires a mutation-gated
   mechanism; clean multi-file verdicts require sibling_files_swept;
   keyword matches that are not mechanisms go under
   import_time_constants with an idiom rule from PORTING.md §8).
5. Commit per cluster: `[B-6] Audit <cluster> (<n> rows)`. Touch ONLY
   docs/b6-audit/**. `git pull --rebase` before each push; concurrent
   work on main is expected and does not conflict with this directory.

Resumability is by construction: a row is DONE iff its record exists in
the cluster results file; on restart, rebuild the queue from disk and
continue. Rows with status "absent" or "not-applicable" in the manifest
get a one-line record (verdict N/A, note pointing at the register row) —
they still count as done.

Special rows: the data-bind/view-model clusters must be audited against
CURRENT main (the RB-1 rebuild in progress) and should expect a mixed
state — record what exists today, note "RB-1 migration in flight" where
the compensation family is present-but-scheduled-for-deletion, and cite
the #RB-1 mini-queue entry instead of re-litigating it.

## Phase 2 — Summary and handback (executor, final commit)

Write docs/b6-audit/SUMMARY.md: totals per verdict, per-cluster table,
every DIVERGENT row ranked by count of mutation-gated mechanisms, every
low-confidence row, and any rows where the C++/Rust mapping in the
manifest looked wrong. Then STOP. Do not open tickets, do not propose
rebuilds, do not modify code, corpus files, or any doc outside
docs/b6-audit/**.

## Phase 3 — Triage (planner + user; NOT the executor)

The planner reviews SUMMARY.md, spot-verifies a sample of DIVERGENT and
ISOMORPHIC rows (adversarial re-check), then converts findings into:
rebuild tickets (RB-n, like RB-1), explicit D-rows (user decision), or
rulebook amendments when a "violation" recurs because the rule was wrong.

## Phase 4 — Post-rebuild second pass (planner, completed 2026-07-24)

Re-audit the 36 UNKNOWN rows and the five Family C mechanism families after
RB-1/RD-1. Missing/incomplete Rust lifecycles use TRACKED-GAP with an existing
register owner; they do not fabricate a DIVERGENT mechanism. Update the
correspondence manifest, summary, triage, register, and closeout status, then
install a mechanical zero-UNKNOWN/count/ownership ratchet.

## Escalation rules (executor)

- A row that cannot be honestly audited (missing mapping, unreadable
  region, C++ file gone at the pin) → record verdict UNKNOWN with the
  blocker; never guess.
- Discovery of an apparent BEHAVIOR bug while auditing structure → one
  line in the record's notes; do not chase it.
- Anything tempting you to edit code → stop; that is planner territory.
