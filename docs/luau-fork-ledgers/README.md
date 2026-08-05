# luaur fork ladder ledgers (archive)

Working artifacts of the 2026-08-04/05 luaur engine fork ladder
(docs/luau-fork.md). These were produced in the lane worktrees' untracked
`.luau-fork-work/` directories and are archived in the sibling `luau-fork-ledgers.tar.gz` as provenance (tarballed so repo-scanning gates — test-correspondence counts, ownership-ledger and negative-control scans — do not ingest the ledgers' C++ symbol and TEST_CASE vocabulary as phantom entries) for what
each rung ported; a future engine bump generates fresh ones with the same
method.

- `inventory/` — the adjudicated hunk ledgers: `job0.md` is the baseline
  audit (vendored translation vs the `8f33df91` fork tree); `job1..job8`
  are the official release rungs 0.725..0.732; `job9.md` is the
  rive_0_732 patch set. Every C hunk is attributed to a Rust twin with
  FFlag posture and scope class; each ledger ends with completeness
  accounting (hunks total/attributed/unresolved).
- `prompts/` — the reusable lane prompts: the shared translation-convention
  `preamble.md`, per-rung inventory prompts, and per-rung writer prompts
  (including the binding FFlag policy and atomic row-groups).
- `dispositions/` — per-rung writer reports: `rungN-noop.md` (rows verified
  as C++-only no-ops) and `rungN-blocked.md` (rows deferred with rationale,
  e.g. the untranslated Require/ and Inliner/ subsystems).
