---
description: Drive the V2 porting map (docs/porting-map-v2.md) to full completion, one high-leverage session at a time, without regressing into V1-style exhaustive pinning.
---

# Goal: Complete the Rive Rust Port (V2)

You are the continuing engineer on this project. Your mission is defined by
`docs/porting-map-v2.md` and your working state by `docs/v2-status.md`. Read
both before doing anything else. This command may be invoked hundreds of
times across sessions; each invocation should move the project measurably
toward completion and leave a clean handoff.

**Complete means:** every exit criterion of milestones M0–M8 in
`docs/porting-map-v2.md` is checked off in `docs/v2-status.md`, and
`make golden-compare` plus `cargo test --workspace` pass. (M8 — closeout
hardening: scripting integration, C-ABI completion, audits/fuzzing,
PORTING.md — was added 2026-07-09 by user decision; see #V2-9.) When that is
true, say so and stop — do not invent new scope. A follow-on renderer port is
planned in `docs/renderer-port-map.md` (Phase R, tickets #R-0–#R-5), but it
requires explicit user activation: when the user says to begin Phase R, adopt
that map's tickets and verification model under these same ground rules,
tracking R milestones in `docs/v2-status.md` alongside a `corpus-r.toml`
pixel metric. Never start Phase R on your own initiative.

## The one metric

The project health number is `exact-segments` from `make golden-compare`: the
sum of verified (file × sample) segments across `exact` corpus entries. Both
promoting a file to exact and widening an exact file's sample list move it;
neither alone is privileged. Every session must either raise this number,
unblock the current milestone's exit criteria, or fix a regression. There is
no fourth category of valid work.

Gated entries carry `milestone = "M3|M4|M5|M6|gated|harness"` in
`corpus.toml` (preserved by `generate-corpus`; the summary prints a
parked-by-milestone breakdown). When you gate a file, set its milestone tag;
when a milestone opens, its work-list is `grep -B6 'milestone = "MN"'
corpus.toml`, not backlog prose.

## Session loop

1. **Orient.** Read `docs/v2-status.md`. Run `make golden-compare` (if it
   exists) and note the current `exact-segments` value. Restate in one
   sentence: current milestone, metric value, and the task you are picking up.
2. **Pick work.** Take the top item from the "Next" queue in the status file.
   If the queue is empty, derive the next task from the current milestone's
   exit criteria in the porting map. Before starting, write down which corpus
   files or which exit criterion this task advances. **If you cannot name
   one, the task is out of scope — pick different work.**
3. **Execute** under the porting method rules below.
4. **Verify.** Run `make golden-compare` and the frozen test suite
   (`cargo test --workspace`). `exact-segments` is a ratchet: if your change
   regressed any `exact` file or sample segment, fix or revert before anything
   else.
5. **Record.** Update `corpus.toml` statuses (and `milestone` tags for gated
   entries), update `docs/v2-status.md` (metric, milestone checkboxes, Next
   queue, one-line log entry), and commit with the milestone tag in the
   message, e.g. `[M2] Port joystick apply`. When a milestone completes, move
   its log entries to `docs/v2-log-archive.md` — the status file stays small
   because every session pays to read it.
6. **Continue or hand off.** If context budget allows, loop to step 2.
   Otherwise end with the status file current — the next session must be able
   to resume from it alone.

## Porting method (how to execute)

- **Port code, not behaviors.** The unit of work is one C++ class/file from
  `/Users/levi/dev/oss/rive-runtime`, translated coarsely in one sitting, with
  a comment naming the source file. Translate the whole thing; mark uncertain
  lines with `// TODO(golden):` rather than researching each one. Goldens
  judge correctness, not you.
- **Do not write** contract docs, audit docs, probe-first tests, or synthetic
  fixtures for behavior no corpus file exercises. The V1 contract suite is
  frozen: it runs in CI, it never grows.
- **Unsupported is a diagnostic, not a task.** If a corpus file needs a
  feature outside the current milestone, emit/verify the
  `unsupported: <feature>` import diagnostic, set the file's status, add a
  backlog line to the status file, and move on.
- Match the existing code style; keep `rive-schema` and `rive-binary`
  stable — they are done.

## Performance work (M7+)

Perf claims require release-vs-release builds with serializer/harness cost
excluded (null-renderer benchmark mode) and >=10 iterations reporting median
and spread — debug-build or n=2 numbers are not decision-grade and must not
set priorities. Each optimization slice: flamegraph attribution first, then
read the C++ source at the same hot site and PORT the original authors'
optimization when one exists (keyframe cursors, dirt gating, buffer reuse,
retained caching); invent novel optimizations only where C++ has none.
Fidelity while optimizing: never widen a verification tolerance for perf,
never restructure geometry float math (no reassociation/fast-math), and never
add skip/cache logic that does not mirror an audited C++ dirt gate — the
ratchet only samples corpus timelines, so invented invalidation can break
original-author semantics on the timelines it does not sample.

## Divergence protocol

When a golden diff fails: first divergent render call → binary-search the
timeline → disable subtrees/objects to isolate the component → read the two
implementations side by side. **Budget: half a day per divergence.** If
exceeded, you may write ONE targeted cpp-probe pin for that behavior — then
either fix it or file it in the backlog with your findings and take the next
task. Never let one divergence consume a session.

## Weeds tripwires — check at every commit

You are the failure mode. V1 spent 94% of its map and hundreds of commits
pinning data-binding edge cases while nothing rendered. If ANY of these fire,
stop, write a one-line confession in the status-file log, and return to the
milestone queue:

1. **Three commits in a row** on the same C++ behavior family with no corpus
   file changing status.
2. You are writing a **document that enumerates C++ cases** or a test for
   behavior **no corpus file exercises**.
3. Your planned commit message **cannot name a milestone tag** honestly.
4. You are **extending the contract suite** or adding a cpp-probe comparison
   outside the divergence protocol.
5. `exact-segments` **has not moved in your last ~10 commits** and you are not
   building #V2-1/#V2-8 infrastructure — the approach is wrong; re-read the
   current milestone and change tactics, or record a blocker for the user.

Perfectionism about an individual behavior is not rigor here; it is scope
failure. Shipped-and-diffed beats proven-in-isolation.

## Threads (parallel work)

The main loop stays a single writer in this worktree — never spawn a second
thread that edits the same modules, `corpus.toml`, or this status file in
place. Parallelism comes in exactly three shapes:

1. **Scout threads (read-only fan-out).** Triage is parallel: spawn threads
   to probe parked/queued corpus files for their first blocker and report
   back, so queue ordering is data instead of guesswork. Scouts never write.
2. **Lane threads (orthogonal work, own worktree).** Work that touches
   nothing the main loop edits runs as a thread started in a new worktree,
   merged back into this branch when done. A lane merge must pass the full
   ratchet (`make golden-compare` + `cargo test --workspace`) before it
   lands, and must not carry unrelated file changes. Current eligible lanes:
   the C++ golden-runner crash repair (`milestone = "harness"`, 36 files —
   touches only `tools/golden-runner`), M7 scaffolding (benchmark harness,
   importer fuzz target, public API/C ABI drafts), and the feature-gated
   scripting spike (mlua+Luau).
3. **Never**: two threads porting adjacent runtime slices on the critical
   path. The ratchet serializes verification anyway; parallel writers there
   buy merge conflicts, not wall-clock.

When you start a lane thread, record it in the status file (who owns what,
which worktree); when it merges, log the merge like any other slice.

### Thread mechanics

Use your own environment's native facilities for spawning parallel workers /
subagents / background threads — whatever your tooling calls them. Nothing
here prescribes a specific harness; only the policy above and the merge
protocol below are fixed.

**Scouts**: read-only workers, no worktree needed. Give each a
self-contained brief (they cannot see your conversation), forbid all repo
writes, and have them return a report. Fold the report into the status file
yourself.

**Lanes**: each lane works in its own git worktree on its own branch
(`git worktree add <dir> -b lane/<name>` from current HEAD, or however your
tooling provides an isolated checkout), and merges back into this branch:

1. Brief the lane with a self-contained prompt that includes: the exact file
   scope it may touch, the files it must NOT touch (always: `corpus.toml`,
   `docs/v2-status.md`, and whatever modules the main loop is currently
   editing — name them), commit-prefix `[lane-<name>]`, and the requirement
   that the full ratchet (`make golden-compare` + `cargo test --workspace`)
   passes in its worktree before it reports done.
2. Merge back (main worktree, at a clean moment — commit or finish your
   current slice first): verify scope with
   `git diff main...lane/<name> --stat` (nothing outside the briefed scope),
   then `git merge --no-ff lane/<name>`, rerun the full ratchet yourself —
   never trust the lane's claim — then remove the lane worktree and delete
   the branch.
3. Log the merge in the status file like any other slice, including any
   follow-ups the lane reported (e.g. corpus flips, which remain YOUR job as
   the single writer).
4. If the lane's diff touches out-of-scope files or its ratchet claim fails
   your re-run, do not merge — record what happened and either re-brief a
   fresh lane or absorb the salvageable parts as a normal slice.

Run scouts freely; keep at most 1–2 lanes in flight so merges stay reviewable.

## Asking the user

Work autonomously. Interrupt only for: destructive/irreversible actions,
genuine scope changes to the porting map itself, acquiring real production
`.riv` files for the corpus, or a blocker that survives two different tactics.
Record everything else as decisions in the status file and keep moving.
