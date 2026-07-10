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
   `LAST_SYNCED_SHA..upstream/main` (or the tag the user named), bucket
   every commit per the map's path signatures.
2. **Probe.** On a throwaway branch, bump the reference pin and run the
   default + scripted golden compares. Attribute every diff to an upstream
   commit. Unattributed diffs mean your triage is incomplete — re-triage
   before writing the report. Do not land the probe branch.
3. **Triage report.** Write `docs/sync/triage-<date>-<shortsha>.md` per the
   map's table format and rating rubric, including version-skew checks
   (Luau bytecode version FIRST, then shaping/layout deps, then .riv header
   version). Resurface prior deferred rows with staleness counters.
4. **STOP FOR APPROVAL.** Present the report summary and top
   recommendations to the user. Port NOTHING and move NO pins without
   explicit row-level approval. Standing approvals recorded in the map's
   State section may be acted on, citing them.
5. **Port approved rows** in upstream order — V2 method, one commit per
   upstream change, message format `[sync] Port rive-runtime <sha>: <title>`,
   goldens as oracle. New upstream fixtures enter the corpus as `not-yet`.
6. **Advance the pin** (CI SHA, map State section, status file) only when
   the full ratchet is green at the new pin. Deliberate skips that move
   goldens require a Decision entry naming the upstream sha diverged from.
7. **Close the cycle**: append the summary to the triage file; append
   renderer-bucket rows to `docs/sync/phase-r-backlog.md`; update
   LAST_SYNCED_SHA.

## Rules

- The approval gate is absolute — a nightly invocation ends at step 4 with
  a report and a notification, never a port.
- All V2/goal ground rules apply to port slices (port code not behaviors,
  ratchet per commit, fences, single writer, threads policy for scouts).
- Never modify the reference checkout beyond `git fetch`/branch switches,
  and always restore it to the pinned SHA when the cycle ends — the golden
  oracle for the CURRENT tree depends on it.
