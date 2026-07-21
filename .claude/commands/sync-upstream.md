---
description: Run one upstream-sync cycle against rive-app/rive-runtime — triage new commits with ratings, stop for user approval, then port approved changes and advance the reference pin with a green ratchet.
---

# Sync Upstream (Phase S cycle)

You are running one cycle of the upstream-sync workflow defined in
`docs/upstream-sync-map.md` — read it first; it is authoritative. This
command may be invoked manually or by a scheduled job.

GATE: if M8 is not checked off in `docs/v2-status.md`, stop and say so —
Phase S activates only after the migration closes.

## Steps

1. **Inventory.** Fetch the reference repo, list
   `LAST_SYNCED_SHA..upstream/main` (or the tag the user named), then run
   `RIVE_RUNTIME_DIR=<clean-candidate-worktree> make port-manifest-check`
   before bucketing every commit per the map's path signatures. A missing or
   stale non-generated `src/**/*.cpp` row (`src/generated/**` is owned by the
   schema/codegen gate) is a required triage finding; do not regenerate or
   reclassify the manifest before the approval gate.
2. **Probe.** On a throwaway branch, bump the reference pin and run the
   default + scripted golden compares. Attribute every diff to an upstream
   commit. Unattributed diffs mean your triage is incomplete — re-triage
   before writing the report. Use a clean temporary C++ worktree, verify its
   HEAD is the candidate SHA, and pass it explicitly as
   `RIVE_RUNTIME_DIR=<candidate-worktree>`. Do not mutate the pinned checkout
   or land the probe branch.
3. **Triage report.** Write `docs/sync/triage-<date>-<shortsha>.md` per the
   map's table format and rating rubric, including version-skew checks
   (.riv header/format FIRST, then Luau bytecode/vendor compatibility, then
   shaping/layout/bidi/image dependencies). Renderer is an active sync bucket,
   not a Phase-R deferral. Resurface prior deferred rows with staleness counters.
4. **STOP FOR APPROVAL.** Present the report summary and top
   recommendations to the user. Port NOTHING and move NO pins without
   explicit row-level approval, a standing category approval, or a
   cycle-scoped authorization recorded in the map's State section. Cite the
   applicable authorization before acting.
5. **Port approved rows** in upstream order — V2 method, one commit per
   upstream change, message format `[sync] Port rive-runtime <sha>: <title>`,
   goldens as oracle. New upstream fixtures enter the corpus as `not-yet`.
6. **Advance the pin** across every active-pin artifact listed in the map's
   State section, plus the map/status state, only when the full ratchet is
   green at the new pin. Never blanket-rewrite historical evidence or fixture
   provenance. Deliberate skips that move goldens require a Decision entry
   naming the upstream sha diverged from.
7. **Close the cycle**: append the summary to the triage file; append
   deferred rows with staleness counters; update LAST_SYNCED_SHA and the clean
   manual-cycle count.

## Rules

- Scheduled triage ends at step 4 with a report and a notification unless a
  standing category approval is recorded. It never infers approval from prior
  cycles.
- All V2/goal ground rules apply to port slices (port code not behaviors,
  ratchet per commit, fences, single writer, threads policy for scouts).
- Keep the pinned reference checkout untouched. Fetch its repository metadata
  if needed, but run candidate probes from a clean temporary C++ worktree and
  remove that worktree when the cycle ends.
