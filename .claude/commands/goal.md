---
description: Drive Phase R (the renderer port, docs/renderer-port-map.md) to full completion, one high-leverage session at a time, with pixel goldens as the oracle and V2's runtime ratchets as the regression floor.
---

# Goal: Complete the Nuxie Renderer Port (Phase R)

You are the continuing engineer on this project. V2 (the runtime port,
M0–M8) is COMPLETE and is now the regression floor — its history and method
live in `docs/porting-map-v2.md` / `docs/v2-status.md` / archives, and its
idiom codex is `docs/PORTING.md`. Your active mission is defined by
`docs/renderer-port-map.md` and your working state by
`docs/renderer-status.md`. Read both before doing anything else. This
command may be invoked hundreds of times across sessions; each invocation
should move the project measurably toward completion and leave a clean
handoff.

**Complete means:** every exit criterion of tickets #R-0 through #R-5 in
`docs/renderer-port-map.md` (including the 2026-07-11 additions: the mid-R2
adversarial review of the wgpu plumbing, and the R3 entry criteria — GPU
semantic-trap audit and renderer fuzz-replay harness) is checked off in
`docs/renderer-status.md`, with `make renderer-golden` at its target and the
full V2 floor green. When that is true, say so and stop — do not invent new
scope. Phase S (upstream sync, `docs/upstream-sync-map.md`) requires
explicit user activation; never start it on your own initiative.

## The one metric

The project health number is the **pixel-exact entry count** from
`make renderer-golden` over `corpus-r.toml` (currently 1,467 entries:
GM streams + `.riv` streams × modes). Every session must either raise this
number, unblock the current R-ticket's exit criteria, or fix a regression.
There is no fourth category of valid work.

Two honesty invariants protect the metric: the **stub baseline** (a
do-nothing renderer must fail every active entry — re-verify it whenever the
comparator changes) and the tolerance rule (per-mode tolerances are declared
in the manifest and never widened to paper over missing algorithm work —
"do not tune broad tolerances around missing algorithm work" is a standing
decision).

**The V2 regression floor:** `make golden-compare`,
`make scripted-golden-compare`, and `cargo test --workspace` must stay green
at their completed values. Renderer work that regresses the runtime floor is
reverted or fixed before anything else.

## Session loop

1. **Orient.** Read `docs/renderer-status.md`. Run `make renderer-golden`
   and note the exact count. Restate in one sentence: current R ticket,
   metric value, and the task you are picking up.
2. **Pick work.** Take the top item from the "Next" queue in the renderer
   status file. If the queue is empty, derive the next task from the current
   ticket's exit criteria in the map. Before starting, name which corpus-r
   entries or which exit criterion this task advances. **If you cannot name
   one, the task is out of scope — pick different work.**
3. **Execute** under the porting method rules below.
4. **Verify.** Run `make renderer-golden` plus the V2 floor. The exact count
   is a ratchet: if your change regressed any exact entry, fix or revert
   before anything else.
5. **Record.** Update `corpus-r.toml` statuses, update
   `docs/renderer-status.md` (metric, ticket checkboxes, Next queue,
   one-line log entry; archive completed-ticket logs to keep the file
   small), and commit with the ticket tag, e.g. `[R2] Port intersection
   board`. After committing, `git push` (origin is
   github.com/nuxieai/nuxie-runtime) and `git push origin <branch>:main` —
   main mirrors this branch by standing user decision. Push failures get a
   log note, never block slices.
6. **Continue or hand off.** If context budget allows, loop to step 2.
   Otherwise end with the status file current — the next session must be
   able to resume from it alone.

## Porting method (renderer edition)

- **Port code, not behaviors.** The unit of work is one C++ file/class from
  `/Users/levi/dev/oss/rive-runtime/renderer/src`, translated coarsely with
  a comment naming the source, judged by pixel goldens. `docs/PORTING.md`
  is the idiom brief. Shaders come from the upstream-generated WGSL
  (naga-validated), never hand-rewritten.
- **wgpu replaces ORE.** When algorithm code touches GPU resources,
  translate ORE/impl concepts directly to wgpu types — never recreate an
  abstraction layer between them. The wgpu resource/binding plumbing is the
  project's one INVENTED seam: it gets the adversarial review (mid-R2, per
  the map) and extra skepticism, because V2 proved bugs live in invented
  seams, not translated code.
- **Sub-oracles are the escalation for internal state.** When final pixels
  can't localize a divergence, build ground truth for the intermediate GPU
  artifact itself (the atlas-mask oracle is the template): capture the
  C++ buffer, capture the Rust buffer, compare directly. Prefer one
  sub-oracle over guessing from downstream pixels.
- **Unsupported is a named gate, not a task.** Entries needing unported
  features stay gated with a named diagnostic in `corpus-r.toml`.
- The runtime crates are DONE — do not modify them for renderer
  convenience; `nuxie-render-stream` is the frozen isolation boundary.

## Divergence protocol (pixel edition)

Heatmap → identify the draw batch → replay a truncated stream up to that
batch → single-patch/single-entry reproduction → read the two
implementations side by side → sub-oracle for the intermediate buffer if
still ambiguous. GPU captures (Metal frame capture / wgpu trace) are the
stream-bisection equivalent. **Budget: half a day per divergence**, then
record findings in the status file and take the next entry. Never chase
bit-exactness across GPUs/backends/modes — per-mode tolerance plus a
Decision entry is the correct fix for a genuine vendor difference; a
per-entry hack is not.

## Performance work (R4)

All V2 perf-fence rules apply unchanged to renderer benchmarks:
release-vs-release, ≥10 iterations with median+spread, min-based
aggregation where contention is one-sided, pinned repeat counts,
flamegraph/GPU-capture attribution before optimization, port C++'s own
optimization at the same site before inventing one, and never widen a
tolerance or restructure float math for speed. The M7 runtime perf gate
remains part of the regression floor. SDK binary size (`make size-report`)
is a tracked release criterion — the renderer's size impact is measured,
not guessed.

## Weeds tripwires — check at every commit

If ANY of these fire, stop, write a one-line confession in the status-file
log, and return to the ticket queue:

1. **Three commits in a row** on the same divergence with no corpus-r entry
   changing status — sub-oracle it or record it and move on.
2. You are **widening a tolerance** or hand-tuning an entry to pass — that
   is a Sol-level Decision with rationale, never a slice detail.
3. Your planned commit message **cannot name an R-ticket tag** honestly.
4. The exact count **has not moved in your last ~10 commits** and you are
   not building ticket infrastructure (oracles, harnesses, audits) — the
   approach is wrong; re-read the ticket and change tactics, or record a
   blocker for the user.
5. You are porting renderer code **the corpus cannot yet exercise** —
   land the replayable surface first; speculative breadth is the V1
   pattern.

## Threads (parallel work)

The main loop stays a single writer in this worktree — never spawn a second
thread that edits the same modules, `corpus-r.toml`, or the status file in
place. Parallelism comes in exactly three shapes:

1. **Scout threads (read-only fan-out).** Attribution is parallel: probe
   gated entries for their first divergence class, classify heatmaps,
   inventory upstream code — report back; scouts never write.
2. **Lane threads (orthogonal work, own worktree).** Work touching nothing
   the main loop edits: oracle/harness tooling, the R3 entry-criteria
   builds (trap audit, fuzz-replay harness), reference regeneration,
   CI/size tooling. Lane merges must pass `make renderer-golden` plus the
   V2 floor, verified by YOU, and carry no out-of-scope changes.
3. **Never**: two threads porting adjacent algorithm slices on the critical
   path.

### Thread mechanics

Use your environment's native facilities for spawning workers. Scouts:
self-contained brief, no repo writes, report back; you fold results into
the status file. Lanes: own worktree/branch (`git worktree add <dir> -b
lane/<name>`), self-contained brief with exact touch/don't-touch scope
(always excluded: `corpus-r.toml`, the status file, whatever you are
editing), `[lane-<name>]` commit prefix, full-gate verification before
reporting. Merge protocol: scope-check the diff stat, `git merge --no-ff`,
re-run the gates yourself — never trust the lane's claim — then remove the
worktree and delete the branch; log the merge and its follow-ups. Refuse
merges that fail scope or gates. Run scouts freely; keep at most 1–2 lanes
in flight.

## Model routing (plan big, execute small)

Two tiers (2026-07-11 user decision): **GPT 5.6 Sol High** (planner) and
**GPT 5.6 Terra High** (executor). Route by VERIFIABILITY, not difficulty —
faithfulness is enforced by the harness (compiler, pixel gates, ratchets),
so executor output either lands green or doesn't land; tier affects
attempts-until-green, never what merges.

**Terra executes anything harness-verifiable:** attribution sweeps and
heatmap classification, mechanical translation slices (PORTING.md + a
precise brief with the C++ citation), compiler-error burn-down,
known-divergence-class fixes applied across batched entries, reference
regeneration, fuzz babysitting/minimization.

**Sol only, never delegated:** decomposition and queue construction,
root-cause analysis of NOVEL divergences, sub-oracle design, adversarial
review (implement small, review big — never the reverse),
fence/tolerance/gate decisions, merges and ratchet verdicts, anything in
the invented wgpu seam or float semantics.

Rules:
1. **Briefs derive from mechanical inventories only** (corpus-r queries,
   heatmap classifications, compiler-error lists) — never from model
   memory; Sol reviews the queue once before any fan-out.
2. **Batch briefs** — ~10 related entries per Terra worker, not one; spawns
   cost a worktree + build.
3. **Escalation ladder**: Terra fails its gate twice on the same item → Sol
   takes it directly with the failure context. Infrastructure failures
   re-assign to a fresh worker, no escalation.
4. **A worker may NEVER loosen a gate to pass it** — no tolerance changes,
   no corpus flips, no test weakening from Terra; those are Sol decisions
   recorded in the status file.

## Asking the user

Work autonomously. Interrupt only for: destructive/irreversible actions,
genuine scope changes to the renderer map itself, tolerance-model changes
that affect the fidelity story, or a blocker that survives two different
tactics. Record everything else as decisions in the status file and keep
moving.
