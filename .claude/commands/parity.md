---
description: Drive the parity closeout (docs/parity-closeout-map.md) to a green five-tier scorecard, one high-leverage session at a time, dispatching sliced work to spine/lane/scout/batch workers with mechanical gates.
---

# Goal: Close the Rive Parity Gap Register

You are the continuing engineer (and orchestrator) on this project. V2 (the
runtime port) and Phase R (the renderer port) are COMPLETE and form the
regression floor. Your active mission is `docs/parity-closeout-map.md`,
your evidence base is `docs/parity-gap-register.md`, and your working state
is `docs/parity-closeout-status.md`. Read the status file and the map
before doing anything else. This command may be invoked hundreds of times;
each invocation moves the scorecard measurably and leaves a clean handoff.

**Complete means:** scorecard tiers 1–5 green in CI with every exception
recorded as a register D-row, and tickets #B/#OR/#FT/#HD checked off in the
status file. #LT tickets open only by explicit user activation. When
complete, say so and stop — do not invent new scope.

## The metric

**tiers-green (0–5)** from `make parity-scorecard`, backed by the per-tier
ratchets (exact-segments, e2e-exact, side-channel-segments,
fuzz-clean-nights, A-rows-closed, pixel-exact, perf ratio, size). Every
session must raise a ratchet, unblock a ticket's gate, or fix a floor
regression. There is no fourth category of valid work.

Honesty invariants: stub-baseline re-verification whenever any comparator
changes (side-channel and e2e included); tolerances/gates are never
loosened to pass — that is a user-level Decision. The regression floor
(`make golden-compare`, `scripted-golden-compare`, `renderer-golden`,
`cargo test --workspace`, `capi-smoke`) stays green at completed values;
floor regressions outrank all other work.

## Session loop

1. **Orient.** Read `docs/parity-closeout-status.md`. Run the floor +
   scorecard. Restate in one sentence: current ticket, ratchet values, the
   task you are picking up.
2. **Pick work.** Top of the Next queue. If empty, derive from the map's
   ticket order (critical path: B-1 → OR-1 → OR-2 → OR-3 → fan-out; lanes
   anytime). Before starting, name which gate or ratchet this advances —
   if you cannot, the task is out of scope.
3. **Check gates.** If the item is a USER-GATE (listed in the map), prepare
   the decision brief, record it under "Pending USER-GATEs" in the status
   file, surface it to the user, and take the next unblocked item. Never
   infer approval; Phase S approvals follow `docs/upstream-sync-map.md`.
4. **Execute or dispatch.** Spine work you do yourself in this worktree
   (single writer: runners, golden-compare, corpus manifests, side-channel
   format, status file, CI). Orthogonal work goes to workers using the
   map's brief template and thread shapes:
   - Scouts: read-only evidence fan-out; fold reports into the status file.
   - Lanes: own worktree (`git worktree add <dir> -b lane/<name>`),
     `[lane-<name>]` commits, exact touch/don't-touch scope; at most 2–3
     lanes in flight.
   - Batch fan-out: disjoint corpus shards, fixtures/reports only; YOU
     apply all corpus.toml edits.
   Route by verifiability per the map: executor-tier for mechanical/
   citable/batched work; planner-tier for format & gate design, novel
   divergence root-cause, merges, anything touching tolerances or the
   D-list. Executor fails the same gate twice → take it yourself with the
   failure context.
5. **Verify.** Re-run the slice's gate AND the floor yourself — never trust
   a worker's claim. Ratchets only move on your run. Divergences follow the
   escalation ladder with the half-day budget, then get filed as register
   rows with a failing entry attached.
6. **Record.** Update the status file (scorecard, checklist, Next queue,
   Pending USER-GATEs, one-line log), update register rows you closed or
   opened, commit with the ticket tag (`[OR-2] …`, `[lane-ft-scroll] …`) on
   `main`, push origin. Push failures get a log note, never block.
7. **Continue or hand off.** Loop to 2 while context allows; otherwise end
   with the status file current enough to resume from alone.

## Weeds tripwires — check at every commit

The map's seven tripwires are binding: (1) three commits on one divergence
with no status change; (2) any tolerance/gate loosening outside a user
Decision; (3) a commit that cannot honestly carry a ticket tag; (4) ~10
commits with no ratchet motion while not building ticketed infrastructure;
(5) implementing a feature no fixture can fail — oracle first; (6) merging
on a worker's claimed green; (7) two writers on spine files. If one fires:
stop, one-line confession in the status log, back to the queue.

## Asking the user

Work autonomously. Interrupt only for: the map's named USER-GATEs (Phase S
approvals, size budget, audio engine, threading-model decision, WebGL2,
production-corpus access, any new D-row), destructive/irreversible actions,
genuine scope changes to the map itself, or a blocker that survives two
different tactics. Everything else is a recorded decision in the status
file.
