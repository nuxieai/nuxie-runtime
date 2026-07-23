# Goal: Finish the Nuxie Runtime Parity Closeout (Executor Protocol)

You are the continuing engineer on the Rive-to-Rust parity closeout. The
planning is DONE — every remaining gap is identified, sequenced, and
documented. Your job is execution: small, mechanical, floor-gated steps.
When this file and the status file disagree, the status file wins (it is
newer). Resume state lives ONLY in files, never in memory:

- `docs/parity-closeout-status.md` — live state, ticket mini-queues, Next
  queue. Read FIRST every session.
- `docs/parity-closeout-map.md` — the plan (Phases 0-4, RB, RD).
- `docs/PORTING.md` — porting rules incl. §8 architecture-fidelity (AF-1..8).
- `docs/rb1-compensation-inventory.md` — what slice (f) deletes, file:line.
- `docs/b6-audit/` — structural audit results, TRIAGE.md, spec.
- `docs/parity-gap-register.md` — gap register incl. D-rows.

## Session start ritual (every session, no exceptions)

1. `git -C /Users/levi/dev/oss/rive-runtime rev-parse HEAD` MUST print
   `d788e8ec6e8b598526607d6a1e8818e8b637b60c`. If not: checkout that pin
   and rebuild `make cpp-probe` + `librive` before trusting ANY gate.
   Unpinned checkouts have poisoned this project twice — never skip this.
2. Read docs/parity-closeout-status.md end to end.
3. Check `git worktree list` and `git branch --list 'worktree-agent-*'`
   for a completed in-flight lane (e.g. the e5 lane): if a lane branch has
   commits, VERIFY its gates yourself (battery below) and ff-merge (rebase
   onto main first if docs-only commits landed since); if clean/stale,
   remove it and do the work inline.
4. Restate in one sentence: current queue item, gate numbers, what you
   are picking up.

## The gate battery (run yourself; ratchets move only on YOUR run)

```
cargo test -p nuxie-runtime --lib                     # currently 344, 0 failed
cargo test -p nuxie --lib                             # currently 132, 0 failed
RIVE_CPP_PROBE=$PWD/tools/cpp-probe/build/macosx/bin/debug/rive_cpp_probe \
  cargo test -p nuxie-runtime --test cpp_probe        # 707/707 required
make golden-compare                                   # entries=317 exact=317 exact-segments=647 diverges=0
make scripted-golden-compare                          # same summary; failures currently EXACTLY:
                                                      #   echo_show_demo, list_index_script_access
make renderer-golden                                  # exact=1468 diverges=0 (run when renderer/draw code changes)
make capi-smoke                                       # ok
cargo test --workspace                                # all suites (set RIVE_CPP_PROBE)
```

Iron rules: a gate is never loosened, a tolerance never widened, a test
expectation never edited to pass — those are USER decisions. Any corpus
entry changing that your change did not predict = stop, revert or report.
Commit with ticket tags (`[RB-1]`, `[RD-1]`, `[B-5]`...), push to main
after gates; `git pull --rebase` first (a teammate lands editor work on
main — their changes are audited by the same gates, not trusted).

## The queue (work top to bottom; details in the status file)

1. **#RB-1 e5** (three parts, separate commits — full brief is in the
   status file's mini-queue and the dispatched lane prompt):
   (A) close `echo_show_demo`: Rust fires neither event listener at t=0;
   C++ fires "Night" (StateMachineFireEvent on duration-100 transition →
   ListenerViewModelChange applies Weather enum 3 via updateSourceBinding).
   Port the C++ event-fire/notify ordering with citations.
   (B) `list_index_script_access`: script-visible symbol list index
   diverges (VM state identical; drawn digit differs). Fix or fully
   localize — a documented localization is acceptable.
   (C) Scene facade flush becomes cell-dirt-driven (keep signatures);
   SM trigger reads through trigger cells where the owned path has them.
2. **#RB-1 slice (f) — deletion gate.** Delete the compensation family
   per docs/rb1-compensation-inventory.md's checklist: mutation clocks,
   candidate vectors, listener rescan loop + observed copies, alias
   mirrors, Scene dirty bit, per-source copied direction flags,
   RuntimeArtboardOwnedContextKey. Rewrite the ~20 tests the inventory
   marks "rewrite", delete the ones marked "delete". Gate: full battery
   green INCLUDING scripted at 317/317 with ZERO failures, and a grep
   proving every checklist symbol is gone. This closes #RB-1.
3. **#RD-1 — renderer feed to the C++ retention boundary** (map Phase RD;
   user mandate, supersedes D-12). Sequence is binding: (a) write the
   mini-map (slice list) as your first commit; (b) MEASURED SPIKE — live
   per-frame traversal for a corpus slice, run r4-timing-gate +
   perf-hot-loop, commit the numbers, REPORT THE PERF DELTA TO THE USER
   before demolition begins (this is a USER checkpoint, not a gate);
   (b2) RULEBOOK + STRESS TEST (map "Porting methodology" section, user-
   directed 2026-07-22): codify the renderer-feed translation rules in
   docs/PORTING.md, then have two agents independently translate the
   SAME 2–3 representative C++ draw/traversal files (one strictly from
   the rulebook, one "as a senior Rust engineer"), diff the two, fold
   every disagreement into the rulebook as a new rule, then DISCARD both
   translations before fanning out;
   (c) lane-by-lane migration as FILE-CORRESPONDING PORTS of the C++
   draw/traversal sources (port the C++ file, replace ours — do not
   reshape the existing Rust feed), pixel corpus (1,468) referees every
   merge;
   (d) deletion gate: prepared frames, command streams, path caches,
   epoch bridges gone; re-run the B-6 audit brief over renderer clusters
   expecting zero mutation-gated mechanisms; remove register D-12.
   R4 host-idle samples are telemetry only and never admit or reject a run
   (explicit user decision 2026-07-23); immutable provenance, A-B-B-A order,
   paired control drift, repeat drift, and performance ratios remain gating.
4. **#B-5 fixtures** (4 named in the status file): two-way TrimPath.start
   seeding, nested parent-relative bind default at t=0, self-referential
   layout recursion, deep trigger source paths. Each becomes a repair or
   a user-decided D-row.
5. **#B-1 port** — S3-1 TextInput (`1b4df2ad`) + S3-3 static linking
   (`b73bc675`) per docs/upstream-sync-map.md; advance LAST_SYNCED_SHA on
   green ratchet. Watch the 9 MiB size budget (make size-report) — a
   breach REOPENS the budget USER-GATE; never raise the constant.
6. **#B-6 leftovers**: verify the 5 pending small families (TRIAGE.md
   Family C list) one by one → ticket or clear; second pass over the 36
   UNKNOWN rows (after RB-1, so data-adjacent rows audit stable code).
7. **Then the map's #OR queue** (OR-1 side-channel first) and remaining
   tickets per the map's ordering.
8. **RB-2 (focus system rebuild)** — slot after RD-1 or alongside B-5,
   whichever the status file's Next queue says by then.

## Stop-and-ask-the-user list (never decide these yourself)

- Any new D-row (accepting a divergence) or reclassification of one.
- Any budget/tolerance/gate-threshold change (incl. the 9 MiB size gate).
- The RD-1 post-spike perf number (report it; user reviews).
- Phase S port approvals beyond the already-approved S3-1/S3-3.
- Anything where two honest tactics both failed.

## Working style (calibrated for you)

- Work inline on the spine in SMALL slices; run the fast gates (lib +
  probe) after every substantive change, the full battery before every
  push. Dispatch lanes only for the well-templated shapes (a fixture
  batch, an audit batch) using the briefs in docs/ as templates.
- use as many sub agents as you need when the work is parallelizable
- Port code, not behaviors: every fix cites the C++ file:line it mirrors.
  If you cannot cite it, stop — that is a design question for the user.
- STRUCTURE-PRESERVING BY DEFAULT (map "Porting methodology" section,
  user-directed 2026-07-22): when fixing any divergent subsystem (a B-6
  divergent family or a newly found divergence), port the corresponding
  C++ file(s) and replace our design — do not patch our design until
  behavior matches. New C++→Rust idiom mappings you establish go into
  docs/PORTING.md's translation table when the slice lands. Keep the
  file-correspondence manifest (seeded from the b6-audit manifest)
  current: each in-scope C++ file is `faithful`, `divergent-by-decision`
  (cites a D-row), or `pending`; rows flip to `faithful` only on an
  orchestrator-verified run. Already-faithful gate-green code is never
  re-ported for its own sake.
- Half-day budget per divergence: localize with instrumentation (env-var
  gated eprintln, removed before commit), then fix or file. Never guess.
- Keep the status file current enough that the NEXT session resumes from
  it alone; one-line log entry per landed slice.
- If a teammate's concurrent commit breaks a gate: localize, attribute
  (pristine-main check), fix forward with citation or record at top of
  the Next queue — exactly like the 2026-07-21 log entries model.
