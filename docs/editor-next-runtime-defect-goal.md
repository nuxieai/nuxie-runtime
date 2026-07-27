# Goal: Unblock Editor Next Production by Closing Runtime Defects

This is the operating protocol for the Editor Next runtime-defect program.
Its formal objective is to own the complete queue of runtime, API, renderer,
browser, and artifact defects reported by Editor Fix; qualify each against
pinned C++; faithfully repair every confirmed Rust defect directly or through
an explicitly delegated sole external owner for genuinely non-port work, while
retaining port-covered findings as formal-port-wave dependencies for unchanged
post-port verification; independently verify and land each repair; maintain
exact status; and return immutable landing evidence for downstream
consumption.

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
`233552c13929b09666a62ddff541eb8620d1882b` on
`origin/levi/editor-next-cutover-assembly`:

- proposal SHA-256:
  `905bf599f2058828e678bff118261a60fdda4a1a09f4557693b7247409b5beb9`;
- runtime-defects SHA-256:
  `24e78816d3bafdd61903e4ea1b36ecb77e946accff847963b2ab886d9530b2ae`;
- parity-ledger SHA-256:
  `07d345c82b8dfd18a06201f08726bafd233f13eabd3cca16c3a8d833f8759226`.

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
Only a terminal Editor-owned row may use the source template's `Editor fix`
bullet for the Editor role and `Current checkpoint` bullet for the runtime
role. Each such special block must contain exactly one full SHA, and the role
word must directly identify that SHA; the labels never complete a closed
runtime/API row, accept a multi-SHA block, or infer runtime provenance from
unrelated prose.

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

- Closed: `RT-ED-001`, `RT-ED-002`, `RT-ED-003`, `RT-ED-006`, `LOC-003`,
  `LOC-004`, `LOC-006`, `LOC-011`, `LOC-014`, `LOC-015`, `LOC-016`,
  `LOC-017`, and `LOC-019`.
- Changed intake needs evidence: `RT-ED-005`. Its historical repair is merged,
  consumed, and executor-green, with no production implementation remaining,
  but the changed committed inbox record does not separately label one full
  Editor SHA and one full Runtime SHA. The current cycle cannot independently
  promote it until that fail-closed evidence requirement is satisfied.
- Regression reopened after historical executor-green evidence: `LOC-009`.
  PR #54's `7f1450dc` landing remains immutable history, but independent
  real-GPU verification found an unresolved physical shader-module
  error-scope regression. The row is not promotable or complete and requires
  a new production landing. Diagnosis is parked pending a different reliable
  execution/model environment; replacement clean-worktree task
  `019f9f59-1ac6-7e32-b973-5deb6b457c05` (“Diagnose browser Lua crash”)
  ended without authoritative output. The current
  evidence is a temporary, uncommitted real-GPU probe at exact `fe0a0a07` /
  tree `4512e0d7`; its
  2,129-byte local log has SHA-256
  `93ecaae76c5bfd6252e5fb919087215a1c60a397dd5cfb9a8bc8bf64929b5611`.
  The sole authoritative browser observation is the canonical Chrome abort at
  `luaG_indexerror` / `luaD_throw`; its cause remains under investigation.
  LOC-009 remains parked and frozen, its reopened cycle has consumed nothing,
  and it may consume only a reviewed landed replacement SHA.
- Confirmed runtime defect retained for deferred post-port verification:
  `RT-ED-007`.
  The completed five-part report proves the recovered Scene producer emits
  target property 158 and complete nested source path `[0,0,0]`; identical
  bytes animate at pinned d788 but fail at fe0 because
  `runtime_transition_duration_bindings` drops an unresolved nested default
  reference. The first divergence is narrowly in `state_machine.rs` /
  `state_machine/bindables.rs` (or a focused extracted module), but this row
  makes no direct Runtime Fix request or schedule. The uncommitted Scene API
  patch has no landing claim. After the relevant state-machine port wave
  lands, Defects Fix reruns only the unchanged set → fire → `advance(0)`
  acceptance.
- Historical WebGL2 evidence only, with no linked product child:
  `RT-ED-004`.
- Mapped Scene/API owner family: `LOC-001`, with `LOC-002` and `LOC-005` as
  duplicate acceptance cases under the same retained-owner repair.
- Deferred post-port verification: `LOC-007` path dirt, `LOC-008` intrinsic
  text measurement, and ordinary layout/TextStyle execution. `LOC-008` is
  additionally `intake-needs-evidence` because its changed source record lacks
  a separately labeled full Editor SHA. These records make no direct
  implementation request or schedule. Escalate only an actual simultaneous
  file-writer collision or a safety/data-loss issue.
- Open runtime/FL candidate: `LOC-013`.
- Closed Editor-owned lowering defect: `LOC-011`; one identical explicit-empty
  source-first file is empty through encode, import, bind, shaping, and draw in
  pinned C++ and Rust, and reviewed Editor fix
  `fc1a7e406494ee970bd93e456d1f5cfae468bfd4`, landed tree-identically as
  `3bc62bf82ac7d8518e89d093b46f92c727c5af7a`, repairs the actual
  absent-versus-empty lowering boundary without a runtime change.
- Closed no-repair stale characterization: `LOC-006`; exact committed
  provenance and an independent exact-checkpoint rerun prove the prior
  renderer symptom was caused by the later hover/clear gesture.
- Open browser/renderer qualification candidate: `LOC-012`.
- Closed Apple artifact family: `LOC-015`, `LOC-016`, and `LOC-017`.
  Exact runtime identity
  `0.2.0@b1f58004332a73564ffdd9f8585838209604c4d1`, Editor correction
  `233552c13929b09666a62ddff541eb8620d1882b`, and qualification-only iOS
  consumer `f9528fe4295de0a55d121fd7e5290374b22f03c5` pass the full native
  corpus. Clients bind runtime version plus source revision; no separately
  negotiated ABI version or public-release prerequisite remains.
- Changed intake needs evidence after its additive Scene repair landed:
  `LOC-018`.

There are 11 unique structured children, ten candidate-linked children, and
21 unique affected children. The goal burns
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
Explicit delegation to a sole external owner is an allowed implementation
dependency for genuinely non-port work, not permission for a duplicate writer.
A port-covered row may instead remain a tracked formal-port-wave dependency
with no implementation request, schedule, assignment, or writer lease. In
either case the row remains tracked until the external landing or relevant
port wave is independently verified with the unchanged acceptance, or the row
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

`LOC-007` retains the exact Editor three-test reproduction and pinned C++
ParametricPath citations as deferred post-port verification. This program
keeps the product test red and adds no workaround, implementation request, or
schedule; it reruns the unchanged acceptance after the relevant path/dirt port
wave lands. The same post-port verification rule applies to any other
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
the immutable Editor checkpoint recorded above. Q1 remains open only for the
surviving browser qualification and parked-diagnosis work below. The original
WebGPU-only handoff was
`95027109c89f651835c76646ebf4d8734f032f07`; the consumed runtime has since
advanced to `e72323c808b91d706ba3b745396beaca7accd69a`.
Editor already removed the backend-preference arguments, consumes
`BrowserFactory::new(canvas, width, height)`, and has green required-WebGPU
normal/proof/product evidence at the immutable checkpoint. Do not repeat that
migration.

Independent promotion is complete for `RT-ED-003` and `LOC-019`; both rows are
closed and authorize no further production or promotion work. Current work is:

- keep `LOC-009` outside the serialized tracking merge line, parked, and
  frozen while diagnosis waits for a different reliable execution/model
  environment; replacement task `019f9f59-1ac6-7e32-b973-5deb6b457c05`
  ended without authoritative output. Preserve PR #54 at `7f1450dc` as
  historical evidence, but do not promote, close, or consume the row without
  a reviewed new production landing;
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

### Q2 — Promote RT-ED-005 and retain RT-ED-007 post-port verification

`F-ED-03` (`RT-ED-005`) is merged and consumed. Its generic number/color
property-key authoring, direction semantics, record round trips, runtime
behavior, and executor floors remain historical landing provenance. The
changed committed inbox record is `intake-needs-evidence` because it omits
separately labeled full Editor and Runtime SHAs, so independent promotion is
blocked on evidence—not production code. Do not reopen that Scene
implementation. `P09-C01` is green
for the generic property-target primitive and remains a nonblocking Known
Runtime Defect only for the separate `LOC-002` retained-owner behavior.
Ordinary layout padding and TextStyle font-size/line-height remain under
`P08-C01` / `LOC-018`; their runtime dirt/reflow acceptance is deferred until
the relevant layout/text port wave lands.

The separate `LOC-018` additive Scene-authoring repair is merged: PR #66 exact
head `2707280cb3507f8d5c2f48cfe58f1cf0990e9ed0` rebase-merged at
`d7cef0a8b80411b8ef16bf8b48452ea42f71fbe3`. It adds the complete typed public
`LayoutComponent` 409 / `LayoutComponentStyle` 420 hierarchy, property domain,
owner IDs, record order, encode/import/export fixpoint, and every concrete
pinned-C++ `KeyFrameInterpolator` descendant accepted by `interpolatorId`,
including semantic ScriptAsset ordinal mapping. Its honest current-product
claim remains only +60 type-409 plus +60 type-420 records (410 -> 530). The
remaining ten records and product traversal/order are Editor-owned; runtime
layout/dirt/text execution and pixels remain post-port verification. Its
changed committed inbox record stays `intake-needs-evidence` because it does
not separately label one full Editor assembly SHA. The
implementation is landed, but the row cannot promote until that
committed-evidence gap is resolved.

`F-ED-04` (`RT-ED-007`) is a confirmed runtime defect retained as deferred
post-port verification. The recovered e723 producer plus dirty
`scene.rs` patch emits a 323-byte artifact whose normalized record 26 carries
property 158 and source path `[0,0,0]`. The same bytes animate at pinned d788;
fe0 recognizes the target but resolves no nested default source because
`runtime_transition_duration_bindings` uses the default-instance-only helper
and drops the unresolved reference.

The first divergence is narrowly in
`crates/nuxie-runtime/src/state_machine.rs` and
`crates/nuxie-runtime/src/state_machine/bindables.rs`, or a focused
`transition_duration_binding` module extracted from that seam. This row makes
no direct Runtime Fix request or schedule and grants no Scene writer or broad
Scene/runtime lease. The uncommitted Scene API patch has no landing claim,
although its emitted bytes are qualified correct. Defects Fix retains the
unchanged set → fire → `advance(0)` acceptance and records its independent
result after the relevant state-machine port wave lands.

### Q3 — Close the mapped Scene ownership family

Qualification has localized the current repair to the Scene facade. The
active `levi/loc-001-retained-viewmodel-instance` branch must preserve one
Scene-lifecycle-owned ViewModel-instance handle per authored
`ViewModelInstanceId` across artboard rematerialization. `LOC-002` and
`LOC-005` remain unchanged acceptance cases for the same owner lifecycle.
There is no commit, PR, or landing claim yet; any need to edit lower-level
runtime owners stops this lane.

Retain `LOC-008` intrinsic text measurement as deferred post-port
verification. After the relevant text-measurement wave lands, rerun the
unchanged product acceptance and classify any surviving first divergence; do
not request or schedule an implementation from this program.

### Q4 — Absorb runtime-owner defects through FL

For `LOC-013`:

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

- `F-ED-09`: `LOC-011` is closed by Editor fix `fc1a7e40` after one identical
  explicit-empty file remained empty through the complete pinned-C++/Rust
  runtime chain. After the relevant text-measurement port wave, rerun
  `LOC-008` as deferred post-port verification; split remaining
  `LOC-013` at the first differing authored-value, bind, shaping, outline, or
  pixel stage.
- `F-ED-10`: qualify `LOC-012` on WebGPU. `LOC-014` is independently closed as
  a stale oracle after the exact same 180-by-124 typed Feather scene produced
  zero differing C++/Rust pixels; do not reopen it by tuning feather constants
  or tolerances.
- `F-ED-13`: normalize complete records for `LOC-018`; an Editor/lowering
  difference is repaired there before renderer attribution.

Each discovered owner family becomes its own PR and product handoff.

### Q6 — Apple artifact qualification (complete)

`LOC-015`, `LOC-016`, and `LOC-017` are closed:

1. the hash-addressed XCFramework was built from exact runtime source
   `b1f58004332a73564ffdd9f8585838209604c4d1` with identity `0.2.0@b1f58004`;
2. Editor correction `233552c13929b09666a62ddff541eb8620d1882b`
   and qualification-only iOS consumer
   `f9528fe4295de0a55d121fd7e5290374b22f03c5` exercised the exact artifact;
3. run `5ef5769f-d521-4471-8b91-b9f83acdd065` passed all six sentinels,
   nine native screens, signed GPU canvas, 28 named animations at
   start/quarter/end, behavior checks, archive purity, and framework
   validators; and
4. clients bind the runtime version plus exact source revision. There is no
   separately client-versioned ABI.

The legacy `apple-runtime-v0.1.0` evidence is superseded for qualification.
Public URL/default SwiftPM distribution remains optional downstream work and
does not hold these rows or the defect program open.

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
- publishing an immutable Apple runtime artifact;
- changing a test expectation to fit Rust instead of pinned C++;
- choosing a design change after two honest C++-faithful tactics fail.

## Completion contract

Mark this formal goal complete only when all of the following are true:

- every accepted Editor-reported row has a terminal, evidence-backed
  disposition;
- every confirmed Rust parity defect is faithfully repaired and merged,
  explicitly delegated as genuinely non-port work to a sole external owner
  whose tracked landing is independently verified, or retained as a
  formal-port-wave dependency whose unchanged post-port acceptance is
  independently rerun and classified resolved before the row closes, with no
  duplicate writer;
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
