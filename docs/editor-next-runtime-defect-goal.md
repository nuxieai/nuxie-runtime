# Goal: Unblock Editor Next Production by Closing Runtime Defects

This is the operating protocol for the Editor Next runtime-defect program.
The objective is not merely to reduce a bug count. It is to make Editor Next
production-ready by proving, repairing, consuming, and closing every runtime,
API, renderer, browser, artifact, and integration defect recorded by the
cutover.

The program is complete only when the Editor has consumed the exact landed
runtime repairs and its unchanged linked product gates pass. A green direct
runtime test does not by itself unblock production.

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

If they disagree, the atlas wins for a defect row, the FL ownership ledger
wins for a runtime writer lease, and this goal wins over historical WebGL2
implementation language in the port map. Update the stale lower-precedence
file in the same evidence or planning PR.

The immutable Editor source checkpoint is
`d5bbbb31178b8b29c40747fdd21a829348ede624` on
`origin/levi/editor-next-cutover-assembly`:

- proposal SHA-256:
  `804161a06d88cf6cdabd12d90581e2a71109d6d490a1056bae8bbe02a3468a24`;
- runtime-defects SHA-256:
  `5e2e0306bf9bb2ec3bdf54dc316e48ef0eea16391bf6e72c489960094c96c2de`;
- parity-ledger SHA-256:
  `68e4b28a536473298b42331b1bec2132fc4dadccc46f5902b4f33ab306a35aab`.

The pinned C++ runtime is
`d788e8ec6e8b598526607d6a1e8818e8b637b60c`. The WebGPU-only runtime
decision landed in PR #47 at
`95027109c89f651835c76646ebf4d8734f032f07`.

## Session-start ritual

Every goal turn, without exception:

1. Verify `/Users/levi/dev/oss/rive-runtime` is exactly at the pinned C++ SHA.
   If not, restore it and rebuild the affected C++ probe/reference before
   accepting evidence.
2. Read this file and the complete live status file.
3. Run `tools/editor-next-runtime-defects/run-check.sh` against the immutable
   Editor checkpoint. Never use a hash override or `--test-mode` for
   production evidence.
4. Read the current FL status and reservation. Obtain a fresh handoff after
   every FL landing before assigning any runtime writer.
5. Fetch `origin/main`, inspect active worktrees, and start production work
   from a clean isolated worktree. Do not include quarantined or unrelated
   edits.
6. Restate one sentence: the top unblocked slice, its defect IDs, exact owner
   boundary, reserved files, targeted Editor children, and current gate
   ratchets.
7. Work only that slice through qualification, implementation if authorized,
   independent review, merge, Editor consumption, and status update—or record
   the precise external handoff blocking its next transition.

## Current scoreboard

The atlas contains 25 defect IDs plus the reserved `LOC-010` tombstone.

- Closed: `RT-ED-001`, `RT-ED-002`, `RT-ED-006`, `LOC-003`, `LOC-004`,
  `LOC-009`.
- Open formal blockers: `RT-ED-003`, `RT-ED-004`, `RT-ED-005`,
  `RT-ED-007`.
- Open Scene/API candidates: `LOC-001`, `LOC-005`, `LOC-008`.
- Open runtime/FL candidates: `LOC-002`, `LOC-007`, `LOC-011`, `LOC-013`.
- Open browser/renderer candidates: `LOC-006`, `LOC-012`, `LOC-014`,
  `LOC-019`.
- Open artifact/Editor candidates: `LOC-015`, `LOC-016`, `LOC-017`,
  `LOC-018`.

There are eight children with formal runtime dependencies, 20
candidate-linked children, and 27 unique affected children. The goal burns
all 19 open rows to a terminal, evidence-backed state and gets the complete
27-child product matrix green or explicitly user-excepted.

## Binding decisions

### C++ parity is the implementation authority

For a proven runtime mismatch, port the complete pinned-C++ owner family and
replace the divergent Rust mechanism. Do not imitate the visible outcome,
patch around the Rust design, add a cache, tune a constant, or invent an
optimization. Every production repair cites the exact pinned C++ files,
members, and lifecycle it translates.

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
- `RT-ED-004`, `LOC-006`, and any WebGL2 portion of `LOC-012` or `LOC-014`
  close only after the Editor consumes the WebGPU-only runtime and the
  corresponding supported WebGPU product child is rerun;
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
and review prerequisites pass. Product consumption follows the exact merge SHA
and gates final defect closure. Do not leave a train of stacked drafts. Do not
combine unrelated defects to amortize gates.

## Execution queue

Always pick the first unblocked item. A blocked owner does not stop
file-disjoint qualification or implementation.

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

### Q1 — Consume the WebGPU-only runtime and requalify browser blockers

Current investigation base:
`e72323c808b91d706ba3b745396beaca7accd69a`. Clean Editor checkpoint
`d5bbbb31178b8b29c40747fdd21a829348ede624` consumes that exact runtime.
`LOC-009` is independently verified and closed after its unchanged `P14-C01`
product command passed 4/4; the other browser rows retain their independent
states and evidence requirements.

Requalify:

- `RT-ED-003` / `F-ED-06`: ordinary presentation must not require CPU RGBA
  readback; explicit capture remains deterministic;
- `RT-ED-004` / `F-ED-07`: rerun its linked product scenarios on supported
  WebGPU, then retire the WebGL2-specific defect or map a surviving WebGPU
  failure;
- `LOC-006` / `F-ED-08`: same rule for stale conditional pixels;
- `LOC-019` / `F-ED-11A`: required-WebGPU setup and typed unsupported-state
  behavior remain independent of shader-record execution;
- `LOC-009` / `F-ED-11`: the exact target-0/16 and lookup-occurrence repair
  merged at `7f1450dc`, was consumed through runtime `e72323c8`, and is closed
  after independent verification and unchanged `P14-C01` 4/4.

Runtime-side WebGPU work must keep `make browser-webgpu-only-check` green and
must not reintroduce any prohibited WebGL2 surface.

### Q2 — Close the two formal Scene authoring gaps

`F-ED-03` (`RT-ED-005`) and `F-ED-04` (`RT-ED-007`) are independent,
Scene-owned slices when their exact `TOUCH` sets remain outside FL
reservations.

- Generic number/color property binding must emit the exact property-key
  records and retain C++ direction semantics. It must prove every concrete
  Editor style target and its real setter/dirt closure, not merely generic
  dispatch.
- Nested transition duration must accept and validate the complete nested
  `ViewModelNumberSource` path and reuse the existing low-level resolver.

Each receives direct record round-trip tests, runtime behavior tests, its
linked Editor child (`P09-C01` or `P19-C09`), independent review, and a
separate merged SHA.

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
4. update the atlas and send the closure to the active FL owner;
5. let that sole writer port the complete family;
6. independently rerun the direct fixture and full floors on the merged SHA;
7. send the exact SHA to Editor for the linked product rerun.

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
4. have Editor consume the SDK and rerun the corrected native corpus;
5. promote only a surviving current failure to a source owner.

Old ABI 1.5 evidence never authorizes runtime source changes.

### Q7 — Product closeout

For every merged repair, send Editor:

- exact merged runtime SHA;
- public API signature changes;
- focused direct evidence;
- linked child commands and expected removed signature;
- remaining known failures, if any.

Editor consumes that exact SHA, reruns every linked focused child and affected
aggregate gate, and publishes a clean immutable assembly checkpoint. Update
the atlas with consumed runtime and superproject SHAs. Finish with the complete
27-child matrix and all production aggregate gates.

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
12. Obtain independent orchestrator verification, then Editor consumption.
    The implementer never self-promotes a row to closed.

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

- all 25 defect IDs have a terminal, evidence-backed disposition and no
  candidate remains `reported`, `reproduced`, `unqualified`, `mapped`,
  `executor-green`, or merely `handoff-ready`;
- every proven C++/Rust mismatch is repaired through its complete owner
  family with independent verification;
- every API gap emits and round-trips the exact underlying record;
- every historical WebGL2 row is retired or requalified on WebGPU without a
  hidden fallback;
- every artifact row is replaced, retired, promoted to a current owner, or
  covered by an explicit user decision;
- the Editor consumes the exact merged runtime/SDK SHAs;
- all 27 linked product children and affected aggregate production gates pass
  or have explicit user-approved exceptions;
- the runtime, renderer, browser, Apple, C API, probe, golden, lint, format,
  structural, and size floors are green;
- all correspondence rows are independently verified;
- no displaced compensation, temporary diagnostic, stale source hash, stale
  owner lease, or unpublished required artifact remains;
- the Editor assembly records a clean immutable production-ready checkpoint.

Until then, every turn resumes from the first unblocked queue item above.
