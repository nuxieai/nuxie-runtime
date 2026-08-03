---
description: Run one upstream-sync cycle against rive-app/rive-runtime — triage new commits with ratings, stop for user approval, then port approved changes and advance the reference pin with a green ratchet.
---

# Upstream Sync cycle

You are running one cycle of the upstream-sync workflow defined in
`docs/upstream-sync-map.md` — read it first; it is authoritative. This
command may be invoked manually or by a scheduled job.

GATE: if M8 is not checked off in `docs/v2-status.md`, stop and say so — the
Upstream Sync cycle activates only after the migration closes.

## Steps

1. **Prepare the pass.** The orchestrator fetches upstream, creates or refreshes
   a clean candidate worktree under `~/dev/worktrees/`, verifies its HEAD, and
   clones any candidate dependency forks needed by the runners. Keep the pinned
   checkout untouched.
2. **Scout and probe.** Sandboxed scouts inventory the span, run
   `port-manifest-check`, bucket every commit, and run every locally available
   ordinary/scripted probe. The orchestrator completes network-blocked fetches,
   dependency setup, and probes. Every final-cut diff must be attributed to an
   upstream row or the pass stops for re-triage.
3. **Extend one triage report.** Write or update
   `docs/sync/triage-<date>-<shortsha>.md` per the map. A later candidate is a
   top-up pass on that same report: preserve existing `S<cycle>-<n>` IDs, append
   rows, and refresh final-cut evidence in place. Include version-skew checks
   (.riv header/format first, then Luau, then shaping/layout/bidi/image and build
   interface changes) plus deferred rows and staleness counters.
4. **STOP FOR APPROVAL.** Present the report summary and top
   recommendations to the user. Port NOTHING and move NO pins without
   explicit row-level approval, a standing category approval, or a
   cycle-scoped authorization recorded in the map's State section. Cite the
   applicable authorization before acting.
5. **Port approved rows by owner set.** Safety fixes go first; foundational
   chains are serial; then disjoint subsystem-owner sets may run in parallel,
   each in its own worktree. Preserve upstream order within each set and keep
   one commit per upstream change. Stage unverifiable new fixtures in the
   cycle-local `.s<cycle>-deferred-corpus.toml`, not the pinned corpus.
6. **Verify landings.** A sandbox-blocked worker supplies a commit map for
   orchestrator reconstruction. Treat every reported SHA as a claim: verify
   that the commit object exists and inspect its diff before scheduling it.
   Route overlap sets through named semantic merge resolvers and rerun their
   focused oracles.
7. **Close atomically.** In one landing, advance every active pin and
   `LAST_SYNCED_SHA`, update candidate-dependent runner build configs, rebuild
   the required oracles, enroll deferred corpus entries, run the full ratchet,
   remove staging, and append the cycle summary/deferred counters. Current pins
   move together; historical evidence, audit pins, source citations, and prior
   fixture provenance stay frozen.

## Rules

- Scheduled triage ends at step 4 with a report and a notification unless a
  standing category approval is recorded. It never infers approval from prior
  cycles.
- All V2/goal ground rules apply to port slices (port code not behaviors,
  ratchet per commit, fences, single writer, threads policy for scouts).
- Keep the pinned reference checkout untouched. Run candidate probes from the
  verified candidate worktree and remove cycle-only worktrees when the cycle
  ends.
