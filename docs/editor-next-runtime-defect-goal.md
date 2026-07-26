# Goal: Unblock Editor Next Production by Closing Runtime Defects

This is the operating protocol for the Editor Next runtime-defect program.
Its formal objective is to own the complete queue of runtime, API, renderer,
browser, and artifact defects reported by Editor Fix; qualify each against
pinned C++; faithfully repair every confirmed Rust defect either directly or
through its sole active Runtime Fix owner; independently verify and land each
repair; maintain exact status; and return immutable landing evidence for
downstream consumption.

Editor may merge before this queue is empty. Editor consumption is a tracked
downstream state, not a prerequisite for closing a landed repair or completing
this program.

## Source-of-truth order

Read these files at the start of every goal turn:

1. `docs/editor-next-runtime-defect-goal.md` — operating protocol and
   completion contract;
2. `docs/editor-next-runtime-defect-status.md` — live queue and latest
   handoffs;
3. `docs/editor-next-runtime-defect-atlas.toml` — machine-readable defect
   state, ownership, provenance, fixtures, revisions, and product children;
4. `docs/editor-next-runtime-defect-port-map.md` — complete qualification and
   owner-family plan;
5. `docs/PORTING.md` — binding C++-to-Rust translation rules;
6. `docs/runtime-frame-loop-status.md` and
   `docs/runtime-frame-loop-ownership.toml` — active FL writer lease.

If they disagree, the atlas wins for a defect row, the status file wins for
scheduling, this goal defines completion, and the FL ownership ledger plus
coordinator task `019f9c97-edcf-76d3-a786-11f443da22d3` wins for writer
collisions. This goal also wins over historical WebGL2 implementation language
in the port map. Update the stale lower-precedence file in the same evidence
or planning PR.

The immutable Editor source checkpoint is
`7ca11e331a57cb3ea574848f8e93eb108878c40b` on
`origin/levi/editor-next-cutover-assembly`:

- proposal SHA-256:
  `39b17ac5632156f6b762372c28ac661b0a47974d4f2e56ab7d81e32376415401`;
- runtime-defects SHA-256:
  `9e81f237ed568b873304a5853a05026d06e360f8475bcb6dec4da9d04bf7390c`;
- parity-ledger SHA-256:
  `04f205269cb833adad7aa15a0e7c18be149c337f0e97bdffce171723eed69e25`.

The pinned C++ runtime is
`d788e8ec6e8b598526607d6a1e8818e8b637b60c`. The WebGPU-only runtime
decision landed in PR #47 at
`95027109c89f651835c76646ebf4d8734f032f07`; the immutable Editor checkpoint
above consumes current runtime
`e72323c808b91d706ba3b745396beaca7accd69a`.

## Defect inbox protocol

The committed Editor repository is the mailbox. Do not depend on Editor Fix
sending messages or ask it for ongoing status.

- canonical branch:
  `origin/levi/editor-next-cutover-assembly`, switching to the final merged
  Editor branch when cutover lands;
- inbox:
  `plans/nuxie-editor-next-runtime-defects.md`;
- linkage:
  `plans/nuxie-editor-next-parity-ledger.json`;
- checkpoint identity: Editor commit SHA plus the SHA-256 of both files.

Intake runs only at an explicit scheduling boundary:

1. finish or integrate the current control-plane/scheduled batch;
2. fetch the canonical Editor branch;
3. compare its latest committed checkpoint with the last consumed checkpoint
   in the atlas/status;
4. import new or changed canonical `LOC-NNN` and `RT-ED-NNN` records,
   preserving Editor provenance;
5. validate stable IDs, commands, SHAs, evidence, and ledger links;
6. leave incomplete records `intake-needs-evidence` without starting chatty
   clarification;
7. commit the consumed checkpoint and imported rows atomically; and
8. rebuild the dependency/file-ownership DAG before scheduling new work.

For schema-v2 intake, each complete new or changed record supplies a
role-labeled Editor SHA, a role-labeled runtime SHA, an exact command or
reproducer, and result/evidence in column-zero top-level Markdown bullets.
The original exact `Editor SHA` / `Runtime SHA` / `Command` / `Result` labels
remain valid. The committed Editor inbox template also permits the narrowly
enumerated `Exact Editor checkpoint/provenance/base`, `Runtime pin`,
`Editor/runtime checkpoint`, and command/reproducer labels. A combined
Editor/runtime checkpoint must contain two distinct full SHAs. An unrelated
continuation SHA, fixture code span, indented bullet, or container-nested text
cannot satisfy a missing role.

New inbox rows do not preempt active repairs. They enter the next intake batch
after active lanes reach a merge or block boundary. Escalate only a safety or
data-loss report, or a concrete shared-file collision, through the
coordinator. Once Editor merges, switch the canonical branch and retain the
final assembly checkpoint as immutable provenance. Route dependency and
landing handoffs through the coordinator; do not task or poll Editor Fix
directly.

`newest_available_*` means the newest commit observed at the last explicit
intake boundary, not a live poll during an active repair. The checker
materializes both recorded commits, verifies their ancestry on the canonical
local branch, hashes both inbox files, binds every atlas ID to its committed
defect section, validates `runtimeDependencies`, `runtimeDefects`, and
candidate assertion links, and derives the exact new-or-changed canonical
record count. It never fetches by itself. Before a program
completion check, the scheduler must fetch once; completion additionally
requires the recorded newest checkpoint to equal the canonical branch tip and
the derived unconsumed count to be zero.

## Session-start ritual

Every goal turn, without exception:

1. Verify `/Users/levi/dev/oss/rive-runtime` is exactly at the pinned C++ SHA.
   If not, restore it and rebuild the affected C++ probe/reference before
   accepting evidence.
2. Read this file and the complete live status file.
3. Run `tools/editor-next-runtime-defects/run-check.sh` against the immutable
   Editor checkpoint. Never use a hash override or `--test-mode` for
   production evidence.
4. Read the current FL status and reservation, then reconcile the atlas with
   the dependency/file-ownership DAG before assigning any writer. Obtain a
   fresh handoff after every FL landing.
5. Fetch `origin/main`, inspect active worktrees, and start production work
   from a clean isolated worktree. Do not include quarantined or unrelated
   edits.
6. Report atlas counts by state, intake count, runnable lanes,
   blocked/overlapping lanes, active PRs/executors, and the exact next
   scheduling decision. Include the last consumed Editor checkpoint, newest
   known checkpoint, unconsumed inbox count, imported atlas count, defects
   closed since the last report, and exact landing SHAs.
7. Fill available lanes with independent read-only qualifications or
   file-disjoint writers. Each lane owns exactly one defect or tightly coupled
   C++ owner family; shared or FL-owned files serialize through the
   coordinator.
8. After every landing, update the atlas/status/checker, notify the
   coordinator, unblock dependents, and refill available lanes before
   starting unrelated work.

## Current scoreboard

The atlas contains 25 defect IDs plus the reserved `LOC-010` tombstone.

- Closed: `RT-ED-001`, `RT-ED-002`, `RT-ED-006`, `LOC-003`, `LOC-004`,
  `LOC-006`.
- Landing-provenance and independent-promotion only: `RT-ED-003`,
  `RT-ED-005`, and `LOC-019`. Their repairs are merged and consumed; no
  production implementation remains in those lanes.
- Regression reopened after historical executor-green evidence: `LOC-009`.
  PR #54's `7f1450dc` landing remains immutable history, but independent
  real-GPU verification found an unresolved physical shader-module
  error-scope regression. The row is not promotable or complete and requires a
  a new production landing after replacement clean-worktree task
  `019f9f59-1ac6-7e32-b973-5deb6b457c05` (“Diagnose browser Lua crash”).
  That task must verify origin/main `fe0a0a07`, start read-only with minimal
  instrumentation, and must not copy the prior dirty harness. The current
  evidence is a temporary, uncommitted real-GPU probe at exact `fe0a0a07` /
  tree `4512e0d7`; its
  2,129-byte local log has SHA-256
  `93ecaae76c5bfd6252e5fb919087215a1c60a397dd5cfb9a8bc8bf64929b5611`.
  The sole authoritative browser observation is the canonical Chrome abort at
  `luaG_indexerror` / `luaD_throw`; its cause remains under investigation.
  LOC-009 remains frozen, its reopened cycle has consumed nothing, and it may
  consume only a reviewed landed replacement SHA.
- Open formal implementation blocker assigned to Runtime Fix: `RT-ED-007`.
  Defects Fix preserves its diagnosis and tests but has no Scene-authoring
  writer; it records independent verification after the Runtime Fix landing.
- Historical WebGL2 evidence only, with no linked product child:
  `RT-ED-004`.
- Open Scene/API candidates: `LOC-001`, `LOC-005`, `LOC-008`.
- Open runtime/FL candidates: `LOC-002`, `LOC-007`, `LOC-011`, `LOC-013`.
- Closed no-repair stale characterization: `LOC-006`; exact committed
  provenance and an independent exact-checkpoint rerun prove the prior
  renderer symptom was caused by the later hover/clear gesture.
- Open browser/renderer qualification candidates: `LOC-012` and `LOC-014`.
- Open artifact/Editor candidates: `LOC-015`, `LOC-016`, `LOC-017`,
  `LOC-018`.

There are nine children with structured runtime links, 15 candidate-linked
children, and 23 unique affected children. The goal burns
all accepted rows to a terminal, evidence-backed disposition. The linked
product matrix remains valuable downstream consumption evidence, but Editor
consumption is not a completion gate for this program.

## Per-defect completion

A confirmed Rust repair is complete when:

1. the exact Editor reproducer and its Editor/runtime SHAs are preserved;
2. pinned-C++ expected behavior is established;
3. the Rust failure is classified and mapped to its C++ owner and exact Rust
   touch set;
4. the faithful repair is independently verified and merged;
5. the exact landing SHA, tests, gates, reviews, and PR are recorded; and
6. the coordinator receives the immutable handoff for optional
   current-or-later Editor consumption.

Editor consumption may advance a downstream state, but it is not required to
close the repair. A row may also close through proven Editor ownership, a
stale oracle, retraction, artifact correction, or explicit user decision.
Assignment to an active Runtime Fix owner is an allowed implementation
dependency, not permission for a duplicate writer; the row remains tracked
until that owner lands and the repair is independently verified, or the row
otherwise reaches a non-repair evidence-backed disposition.

## Parallel work model

After Q0, the dependency/file-ownership DAG is the scheduler. Multiple lanes
run when touch sets are disjoint. Read-only qualification may run beside any
production lane. Every lane records its defect ID, C++ sources, Rust touch
set, reproducer, gates, executor, and merge boundary. A shared file or
FL-owned file serializes through the coordinator; overlap is routed with the
defect ID, C++ owner, Rust files, proposed executor, and dependency rather
than being dropped.

## Binding decisions

### C++ parity is the implementation authority

For a proven runtime mismatch, port the complete pinned-C++ owner family and
replace the divergent Rust mechanism. Do not imitate the visible outcome,
patch around the Rust design, add a cache, tune a constant, or invent an
optimization. Every production repair cites the exact pinned C++ files,
members, and lifecycle it translates.

When that repair translates behavior owned by a focused C++ source file, put
the new or moved Rust implementation in the corresponding focused Rust module
by default instead of growing giant files such as `draw.rs` or
`state_machine.rs`. Preserve existing public APIs through re-exports, make the
move independently reviewable, and update source correspondence. Apply this
only to the owner touched by the active defect; it does not authorize a broad
cleanup.

If pinned C++ lacks the behavior, classify it as an additive product/API
feature and stop for user direction. Do not disguise it as parity work.

### WebGPU is the only browser backend

WebGL2 production support, fallback, selectors, dependencies, and tests were
removed by PR #47. WebGPU absence or initialization failure is an explicit
unsupported browser/device state.

Consequences for the defect program:

- no new WebGL2 repair, FemtoVG fork, WebGL2 fallback, or WebGL2-specific
  runtime workaround is permitted;
- historical WebGL2 rows and fixtures remain immutable evidence until their
  state transitions are recorded;
- `RT-ED-004` and any WebGL2 portion of `LOC-012` or `LOC-014` require an
  evidence-backed WebGPU disposition; `LOC-006` has completed that disposition
  as a no-repair stale characterization. Downstream Editor reruns are tracked
  when available but do not gate a landed runtime repair;
- a surviving WebGPU failure is requalified against its real WebGPU/runtime
  owner. Retirement of WebGL2 is not evidence that the product scenario
  works;
- `LOC-019` now qualifies required-WebGPU setup and typed unsupported-state
  behavior. It must not test fallback to WebGL2.

### Runtime frame-loop ownership has one writer

The FL executor owns the reserved runtime/graph files and the component,
dirt, update, clone, DataBind queue, animation, and state-machine owner
internals listed in the live FL ledgers.

`LOC-007` has already been handed to that owner with the exact Editor
three-test reproduction and pinned C++ ParametricPath citations. This program
keeps the product test red and adds no workaround while FL implements the
faithful callback/dirt closure. The same handoff rule applies to any other
qualification whose first divergence enters a reserved owner.

### One independently reviewable PR per thing

Each qualified defect or inseparable C++ owner family lands as its own PR.
PRs are non-draft and merge as soon as their required direct, repository-floor,
and review prerequisites pass. The coordinator receives the exact merge SHA
for optional current-or-later Editor consumption. Do not leave a train of
stacked drafts. Do not combine unrelated defects to amortize gates.

## Execution queue

Reconcile the atlas and ownership DAG, then fill available lanes in dependency
order. A blocked owner does not stop file-disjoint qualification or
implementation.

### Q0 — Keep the control plane truthful

1. Keep the atlas, status, fixture registry, correction manifest, and Editor
   source hashes synchronized.
2. Replace historical WebGL2 execution language with WebGPU-only
   requalification without rewriting historical evidence.
3. Complete `F-ED-00B` after the FL owner lands and releases the shared
   ledger/harness boundary.
4. Run the checker before and after every atlas transition.

Exit: every row has a valid current fixture registration and the live queue
contains no stale snapshot, backend, owner, or dependency claim.

### Q1 — Promote landed WebGPU repairs and qualify surviving browser rows

The WebGPU runtime consumption and API-migration prerequisite is complete in
the immutable Editor checkpoint recorded above. Q1 remains open for the
explicit requalification and independent-promotion work below. The original
WebGPU-only handoff was
`95027109c89f651835c76646ebf4d8734f032f07`; the consumed runtime has since
advanced to `e72323c808b91d706ba3b745396beaca7accd69a`.
Editor already removed the backend-preference arguments, consumes
`BrowserFactory::new(canvas, width, height)`, and has green required-WebGPU
normal/proof/product evidence at the immutable checkpoint. Do not repeat that
migration.

Current work is:

- independently promote the landing-provenance records for `RT-ED-003` and
  `LOC-019`; their production repairs and Editor consumption are complete;
- keep `LOC-009` outside the serialized tracking merge line and frozen while
  replacement task `019f9f59-1ac6-7e32-b973-5deb6b457c05` diagnoses the
  canonical Chrome abort from verified origin/main `fe0a0a07` in a clean
  worktree; preserve PR #54 at `7f1450dc` as historical evidence, but do not
  promote, close, or consume the row without a reviewed new production
  landing;
- retain `RT-ED-004` / `F-ED-07` only as historical WebGL2 evidence. It has no
  linked product scenario and authorizes no implementation. A current
  rounded-clip claim exists only if an explicitly scheduled identical-input
  pinned-C++ versus WebGPU proof creates one;
- retain the completed `LOC-006` / `F-ED-08`
  `reported -> reproduced -> stale-oracle -> closed` path as immutable
  no-repair evidence; and
- qualify the remaining open renderer rows in Q5 without reopening the landed
  `F-ED-06` implementation or discarding the historical `F-ED-11` repair
  evidence.

Runtime-side WebGPU work must keep `make browser-webgpu-only-check` green and
must not reintroduce any prohibited WebGL2 surface.

### Q2 — Promote RT-ED-005 and verify Runtime Fix's RT-ED-007 landing

`F-ED-03` (`RT-ED-005`) is merged and consumed. Its generic number/color
property-key authoring, direction semantics, record round trips, runtime
behavior, and executor floors are landing provenance awaiting independent
promotion only. Do not reopen that Scene implementation. `P09-C01` remains
separately dependent on the FL-E layout/TextStyle owner family.

`F-ED-04` (`RT-ED-007`) remains the one open formal Scene authoring
implementation, assigned exclusively to Runtime Fix. Defects Fix retains the
diagnosis as read-only evidence and must keep its Scene candidate quarantined.
Runtime Fix owns the complete path
through the exported `scene.rs` seam and its active `state_machine.rs` /
`state_machine/bindables.rs` work. After Runtime Fix lands the nested
`ViewModelNumberSource` repair with its own merged SHA, Defects Fix runs only
the unchanged set → fire → `advance(0)` verification and records the
independent result.

### Q3 — Qualify and close Scene ownership candidates

Qualify `LOC-001`, `LOC-005`, then adversarial `LOC-002` using one identical
C++/Rust/Editor stimulus.

If the first divergence is confined to the public Scene facade, port the
stable retained ViewModel handle/rebind lifecycle in a Scene-owned PR. If it
enters Artboard/DataBind ownership, hand the exact closure to FL-D. Never add
more type-specific remount carry.

Qualify `LOC-008` separately. Add a typed measurement facade only if the
exact low-level runtime measurement path already owns the complete contract.

### Q4 — Absorb runtime-owner defects through FL

For `LOC-002` when runtime-owned, `LOC-007`, `LOC-011`, and `LOC-013`:

1. minimize the current failure;
2. build the pinned C++ direct probe;
3. identify the exact first differing owner/member/lifecycle;
4. update the atlas and route the concrete overlap to the coordinator for
   assignment to the active FL owner;
5. let that sole writer port the complete family;
6. independently rerun the direct fixture and full floors on the merged SHA;
7. send the exact SHA and linked product command to the coordinator.

No F-ED worktree patches reserved runtime files.

### Q5 — Close remaining renderer, text, and record candidates

- `F-ED-09`: split `LOC-008`, `LOC-011`, and `LOC-013` at the first
  differing authored-value, bind, shaping, measurement, outline, or pixel
  stage; do not assume one owner.
- `F-ED-10`: requalify `LOC-012` and `LOC-014` on WebGPU. Port only a proven
  pinned-C++ owner/resource lifecycle; never tune feather constants or
  tolerances.
- `F-ED-13`: normalize complete records for `LOC-018`; an Editor/lowering
  difference is repaired there before renderer attribution.

Each discovered owner family becomes its own PR and product handoff.

### Q6 — Requalify Apple artifacts

For `LOC-015`, `LOC-016`, and `LOC-017`:

1. build and locally qualify ABI 1.6 from an exact runtime SHA;
2. stop at the user release checkpoint with version, channel, checksum, and
   full local evidence;
3. after approval, publish and update `nuxie-ios`;
4. send the immutable SDK identity to the coordinator for optional
   current-or-later Editor consumption and native-corpus rerun;
5. promote only a surviving current failure to a source owner.

Old ABI 1.5 evidence never authorizes runtime source changes.

### Q7 — Downstream handoff

For every merged repair, send the coordinator an Editor-ready handoff with:

- exact merged runtime SHA;
- public API signature changes;
- focused direct evidence;
- linked child commands and expected removed signature;
- remaining known failures, if any.

Editor may consume that exact SHA now or later. When it does, record its
runtime and superproject SHAs and linked product results as downstream
evidence. Do not hold a verified merged runtime repair open solely because
Editor has not yet consumed it.

## Per-slice method

Every slice follows this sequence:

1. Register or validate a deterministic direct failure.
2. Run one identical stimulus through pinned C++, direct Rust, and Editor.
3. Classify the first difference before changing production code.
4. Read every pinned C++ header/source in the complete owner/dependency
   closure. Record construct, retain, dirt, update/advance, draw, clone/rebind,
   and drop; add setup/resize/failure/loss/teardown for resources.
5. Name the Rust owner, displaced mechanism, exact `TOUCH` set, exact
   `DON'T TOUCH` set, and active writer lease.
6. Cite the existing AF/RF/FLR rules. If a new idiom is required, run the
   rulebook-strict versus senior-engineer stress test, adjudicate from C++,
   update `PORTING.md`, and discard both translations before implementation.
7. Port the complete owner family or repair an exact site already proven
   structurally faithful. Delete displaced compensation in the same PR.
8. Add identity, ordering, lifecycle, error, and product-regression tests.
9. Run two independent reviews: ownership/architecture and spec/behavior.
10. Run all targeted and applicable global gates.
11. Rebase on current `origin/main`, rerun affected gates, push with an
    explicit refspec, open a non-draft PR, and merge when green.
12. Obtain independent orchestrator verification, record the exact merge SHA
    and gates, and send the immutable handoff to the coordinator. The
    implementer never self-promotes a row to closed; optional Editor
    consumption is recorded separately.

## Gate battery

The atlas owns exact monotonic minima. Counts may rise but never fall without
an explicit audited harness change.

```sh
python3 tools/editor-next-runtime-defects/test_check.py
tools/editor-next-runtime-defects/run-check.sh
cargo test -p nuxie-runtime --lib
cargo test -p nuxie --lib
make cpp-probe
env -u CPP_CONFIG -u RUST_PROFILE make golden-compare
env -u CPP_CONFIG -u RUST_PROFILE make scripted-golden-compare
env -u CPP_CONFIG -u RUST_PROFILE make cpp-oracle-workspace-tests
make renderer-golden
make browser-webgpu-only-check
make capi-smoke
make apple-runtime-check
make runtime-frame-loop-port-check
make b6-audit-check
make lint-gate
cargo fmt --all -- --check
git diff --check
make size-report
```

Use the renderer and browser gates for every draw, renderer, browser, text
pixel, GPU-canvas, or presentation slice. The probe-armed workspace must run
all 721 pinned C++ probes. Both ordinary and scripted corpora remain 317/317
entries and 647/647 segments with zero failures. Renderer pixels remain
1,468/1,468. Both SDK variants remain below 9,437,184 bytes.

No test expectation, corpus row, tolerance, resource ceiling, provenance
guard, error contract, feature requirement, or gate may be weakened to admit
a repair.

## Stop and ask

Stop for user direction before:

- accepting any deliberate C++ divergence;
- adding behavior absent from pinned C++ as though it were a port;
- changing the runtime/renderer boundary;
- reintroducing WebGL2 or a browser fallback;
- changing the pinned C++ revision;
- changing a budget, tolerance, resource ceiling, or gate;
- touching an FL-reserved production file without a fresh writer handoff;
- using a new translation adaptation before rulebook adjudication;
- publishing an immutable ABI/SDK artifact;
- changing a test expectation to fit Rust instead of pinned C++;
- choosing a design change after two honest C++-faithful tactics fail.

## Completion contract

Mark this formal goal complete only when all of the following are true:

- every accepted Editor-reported row has a terminal, evidence-backed
  disposition;
- every confirmed Rust parity defect is faithfully repaired and merged, or is
  explicitly assigned to a sole active Runtime Fix owner whose tracked landing
  is independently verified before the row closes, with no duplicate writer;
- every landed repair records immutable C++ evidence, tests, applicable
  repository floors, independent reviews, PR, and exact landing SHA;
- every API gap emits and round-trips the exact underlying record;
- every historical WebGL2 row is retired or requalified on WebGPU without a
  hidden fallback;
- every artifact row is corrected, retired, transferred to its proven owner,
  or covered by an explicit user decision;
- the atlas, checker, status, and dependency/file-ownership DAG are mutually
  consistent and green;
- no duplicate writer, unowned overlap, displaced compensation, temporary
  diagnostic, stale source hash, or stale owner lease remains; and
- the intake queue is empty at the program checkpoint. Later Editor reports
  begin a new intake cycle.

Editor merge and Editor consumption are not blockers for this completion
contract. Until the contract is satisfied, each turn reconciles the atlas and
ownership DAG, integrates completed lanes, and refills the next disjoint work.
