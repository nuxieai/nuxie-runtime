# Editor Next Runtime Defect Status

This is the resume ledger for
`docs/editor-next-runtime-defect-goal.md`. The detailed ownership plan is
`docs/editor-next-runtime-defect-port-map.md`, and the machine-readable source
of truth is `docs/editor-next-runtime-defect-atlas.toml`.

## Current state

- phase: post-Q0 intake reconciliation, immediately followed by the
  malformed-font crash repair;
- pinned C++ runtime: `d788e8ec6e8b598526607d6a1e8818e8b637b60c`;
- investigation base: `e72323c808b91d706ba3b745396beaca7accd69a`;
- Editor's last consumed runtime:
  `e72323c808b91d706ba3b745396beaca7accd69a`;
- rows: 25 defects plus the reserved `LOC-010` tombstone;
- closed rows: `RT-ED-001`, `RT-ED-002`, `RT-ED-006`, `LOC-003`,
  and `LOC-004`;
- open rows: 20;
- formal/structured product children in the landed Editor snapshot: 9;
- candidate-linked product children: 15;
- union: 23, with only `P09-C01` overlapping;
- correction rows: 12.
- fixture rows: 25 total, with `RT-ED-001`, `RT-ED-002`, and `LOC-003`
  directly qualified.
- supported browser backend: WebGPU only, landed in runtime PR #47 at
  `95027109c89f651835c76646ebf4d8734f032f07`.
- latest control-plane landing: Q0 PR #61 rebase-merged at exact runtime main
  `2e24cd7f23a35fc96a71c5edb5da77d1a8634e08`;
- active control-plane lane: consume Editor checkpoint `7ca11e33`, accept
  canonical `LOC-*` and `RT-ED-*` changes under one fail-closed contract, and
  bind both structured ledger link forms;
- active production lane: malformed embedded-font outline crash, sole existing
  writer PR #60 at pre-rebase head
  `61d5d018aa036882d17cea1065a78d7f2e057547`;
- defects closed since the preceding Q0 report: 0;
- runnable repair lanes: finish and rebase PR #60 with the exact 833,949-byte
  reproducer, focused font-validation module, d788 evidence, reviews, and
  canonical floors;
- blocked/overlapping lanes: `LOC-002`, `LOC-007`, and `RT-ED-007` need d788
  requalification and their FL collision boundary; `LOC-005` needs the direct
  d788 shared-instance differential. `LOC-006` needs independent no-repair
  promotion, not code.

Defects Fix owns intake, triage, pinned-C++ qualification, faithful repair
orchestration, independent verification, PR/landing tracking, and immutable
downstream handoff evidence for the complete Editor-reported queue. Editor may
merge before the queue is empty, and Editor consumption is not required to
close a verified landed repair. A Runtime Fix assignment prevents duplicate
writers but does not close its row until that owner's landing is independently
verified.

The FL reservation in the atlas remains deliberately conservative after Q0.
No runtime, renderer, Scene, state-machine, Editor product, or compiler file is
authorized by this provenance-only slice.

The current textual FL handoff supersedes the older lease snapshot without
mutating that atlas table in this control-plane follow-up: FL-A is
independently promoted on `levi/fl-a` at
`f86d5ba0146697abc996310c62fa45e1f053144b`; FL-B production is blocked on the
recorded pre-advance `LinearAnimationInstance::didLoop` user safety/API
decision. Defects Fix's duplicate stable-Apple branch was canceled; Runtime
Fix owns that mechanical repair. Therefore all listed
runtime/graph/ledger reservations remain binding until the coordinator
publishes a new lease after that repair and decision.

The immutable Editor checkpoint records the completed WebGPU-only consumption
through runtime `e72323c8`: `P14-C01` is 4/4 green, `P14-C06` is 17/17 green,
and RT-ED-003 direct presentation is consumed. This intake does not use those product
results to self-promote an atlas row; independent state promotion remains a
separate step.

The same committed ledger has nine structured runtime links across
`runtimeDependencies` and `runtimeDefects`, plus 15 candidate links. It
retains `RT-ED-004` only as historical WebGL2 evidence. This intake mirrors
that source linkage without independently promoting any row state.

## Defect inbox

The committed Editor repository is the mailbox:

- canonical branch: `origin/levi/editor-next-cutover-assembly`;
- inbox: `plans/nuxie-editor-next-runtime-defects.md`;
- linkage: `plans/nuxie-editor-next-parity-ledger.json`;
- last consumed checkpoint:
  `7ca11e331a57cb3ea574848f8e93eb108878c40b`;
- newest known checkpoint:
  `7ca11e331a57cb3ea574848f8e93eb108878c40b`;
- inbox SHA-256:
  `9e81f237ed568b873304a5853a05026d06e360f8475bcb6dec4da9d04bf7390c`;
- linkage SHA-256:
  `04f205269cb833adad7aa15a0e7c18be149c337f0e97bdffce171723eed69e25`;
- unconsumed inbox records: 0;
- imported atlas rows: 25.

Intake runs only after the current control-plane or scheduled batch reaches a
merge/block boundary. Missing record evidence becomes
`intake-needs-evidence`; it does not trigger chatty Editor coordination or
preempt active repairs. After reconciliation, the dependency/file-ownership
DAG is rebuilt and disjoint lanes refill available capacity. Dependency and
landing handoffs route through the coordinator; Defects Fix does not task or
poll Editor Fix directly.

Complete schema-v2 records use role-labeled column-zero, top-level bullets for
the Editor SHA, runtime SHA, exact command/reproducer, and result/evidence.
The checker accepts only the enumerated original and current committed inbox
labels; a combined Editor/runtime checkpoint needs two distinct SHAs, and an
unrelated continuation SHA or fixture code span cannot fill a missing role.

The recorded newest checkpoint is the last boundary observation, not a live
poll. The v2 checker proves both recorded commits belong to the canonical
local branch, hashes both committed inbox files, binds atlas IDs and ledger
links to their source records, and derives the unconsumed count. The scheduler
fetches the canonical branch only at a boundary; program completion requires a
fresh fetch, exact tip equality, and zero unconsumed records.

## Editor source snapshot

The last consumed Editor snapshot at intake cycle 2 is
`7ca11e331a57cb3ea574848f8e93eb108878c40b`. The pinned source checkout and
committed blobs used by the checker resolve to that exact SHA, its runtime
gitlink is `e72323c808b91d706ba3b745396beaca7accd69a`, and the three recorded
source artifacts match the commit byte-for-byte. This statement does not claim
that the canonical remote branch is still at the intake-boundary SHA; a later
tip is fetched and reconciled only at the next explicit intake boundary.

The landed snapshot hashes are:

- proposal:
  `39b17ac5632156f6b762372c28ac661b0a47974d4f2e56ab7d81e32376415401`;
- runtime defects:
  `9e81f237ed568b873304a5853a05026d06e360f8475bcb6dec4da9d04bf7390c`;
- parity ledger:
  `04f205269cb833adad7aa15a0e7c18be149c337f0e97bdffce171723eed69e25`.

The earlier reviewed hashes remain in this file's Git history, but their formal
dependency map is stale and must not be used for qualification. Any later
artifact change makes the source-root check fail until a newly reviewed Editor
checkpoint is recorded.

The current checkpoint consumes runtime
`e72323c808b91d706ba3b745396beaca7accd69a`. Producer checkpoint
`f9d798dd3b1f9b2dfdbeb74dcdf4485aae4519f6` emits target-0 WGSL plus
target-16 `BindingMap`; its exact one-UBO inner RSTB is SHA-256
`546517d0dc9fbdaf9585f3daa6e440628e62292d7cb8aa7253fd3019aa35713d`.
That producer checkpoint does not replace the immutable three-artifact source
snapshot above.

## Executable checks

Run the standalone checker tests:

```sh
python3 tools/editor-next-runtime-defects/test_check.py
```

Run the landed-snapshot atlas check:

```sh
tools/editor-next-runtime-defects/run-check.sh
```

The check is provenance-valid only while the source files retain those exact
hashes and the Editor checkout resolves the recorded checkpoint. Never use a
hash override.

`RT-ED-001` (`data_viz_demo`) and `RT-ED-002` (`db_health_tracker`) are closed
as stale observations after a focused current-pin scripted comparison produced
two exact entries and two exact segment streams with zero divergences. The
pinned C++ runner SHA-256 is
`b20b815c9f3fe30223b0c93ed9b162c0ec1f9031fc0001490d094bb006516a0b`.

`LOC-003` is also closed, but for a different reason. A pinned-source audit of
`include/rive/listener_type.hpp`, `src/listener_group.cpp`, and the state
machine pointer entry points found no held-duration or timed long-press
primitive. Rust already mirrors that listener vocabulary. Per the user's exact
C++ parity decision, adding a Rust-only timer would be a new product feature,
not a port repair; the Editor compiler therefore continues to fail closed for
the unsupported duration and its fully qualified regression test passes 1/1.

`F-ED-06` / `RT-ED-003` began at source baseline
`bc139955c7e2d30d9cf611dd14c24606fd13520a`. PR #55's final head
`a1c56b5a80c88db4f6cee6550795b6e242394c46` rebase-merged at
`e72323c808b91d706ba3b745396beaca7accd69a`; those commits have the same
tree. Clean Editor checkpoint
`4da896beb5ec6815f6b01a2433875274a321d06c` consumes that merge. The committed
browser proof records `getCurrentTexture=1`, `mapAsyncRead=0`, and
`putImageData=0` for every measured ordinary ProductHost presentation, while
explicit capture records `getCurrentTexture=0` and `mapAsyncRead=1`. The
product-host proof, static readback audit, and unchanged normal-timeout
device-frame drag gate are green, including the focused drag result 1/1. The
atlas row remains `reported` until independent orchestrator verification and
promotion; `P19-C03` consumption is downstream evidence, not a prerequisite
for closing the verified landed repair.

`F-ED-03` / `RT-ED-005` is classified as an API-surface gap, not a
low-level runtime defect. PR #49's final head
`f0bd914fbac1fd4cf82814216f2ddc88c3e32083` rebase-merged at
`08286481b4e7420768f625f901a944f313b84903`; those commits have the same tree.
That landing includes production commits
`4eec745b704e9920f67098138963dc973e7b2d87` and
`e2d274d8d3b8de3af705d18506a6d48eadebfc0c`, which port the pinned C++ generic
`DataBind.propertyKey` authoring contract into Scene while leaving the
FL-owned runtime mechanism unchanged. They add typed number/color binds,
converter-free direction selection, stable `LayoutComponentStyle` targets
for all four padding keys, exact target/property collision identity,
converter output validation matching C++ `Input`/`None`/`Any` semantics, and
encoded `File::import` behavior tests. Independent review found the missing
converter-free direction surface; the follow-up now proves numeric
`ToSource`, numeric source-first `TwoWay`, and color source-first `TwoWay`
through exact re-import and reverse propagation. Clean Editor checkpoint
`4da896beb5ec6815f6b01a2433875274a321d06c` consumes descendant runtime
`e72323c808b91d706ba3b745396beaca7accd69a`, including the generic
number/color paint primitive for existing `Stroke` and `SolidColor` targets.
The row remains `reported` until independent orchestrator verification and
promotion are recorded. `P09-C01` remains separately Partial on FL-E's
ordinary layout-container/style and TextStyle dirt/reflow surface; that
downstream result is not required to close the verified landed RT-ED-005
repair.
Its executor battery was green:
the 721-test probe-armed workspace suite, 317/317 ordinary and
scripted golden entries with 647/647 segments each, 1468/1468 renderer rows,
CAPI, Apple, frame-loop, B-6, lint/format/diff, WebGPU-only browser, and the
8.74 MiB maximum SDK floor all passed.

The transferred Editor report cited historical C++ pin
`f4bb3025e263ad1a646ef6971358577a0aa6bfa2`. It is retained as provenance,
not silently treated as the current oracle. The relevant source set changed
before `d788e8ec6e8b598526607d6a1e8818e8b637b60c`: the current pin adds
generated property notifications, target observation, and explicit
reconcile-origin handling. `COR-01` therefore requires the F-ED source hashes,
fixture, executable probe, and behavioral assertions to use `d788e8ec`.

`F-ED-11` / `LOC-019` is landed and consumed but remains `executor-green`
pending independent orchestrator verification. PR #51's final head
`22454fb58bc80d95174ca78d0c0d4d611b0d5a08` rebase-merged at
`ef9dcedd82265efc0184f4f59d5f6aaab0b56cd9`; those commits have the same
tree. Clean Editor checkpoint
`4da896beb5ec6815f6b01a2433875274a321d06c` consumes descendant runtime
`e72323c808b91d706ba3b745396beaca7accd69a`. Its unchanged required-WebGPU
`P14-C06` command passes 17/17: ProductRuntimeTools source validation, both
retained-session fixtures, single and batch snapshots, and all 12 WebGPU
pixel fixtures reach readiness. The executor record remains limited to its
exact local canonical floors; no queued hosted Apple lane is relabeled green.

## Next queue

1. independently review and merge this focused checkpoint-7ca intake/checker
   reconciliation;
2. record the inherited malformed-font crash as `RT-FUZZ-001`, rebase the sole
   existing PR #60 writer onto current main, and preserve the exact 833,949-byte
   reproducer;
3. split the touched font-validation seam into the focused Rust module that
   corresponds to the C++ font owner, update its source mapping, and prove the
   empty-outline behavior against pinned d788;
4. run the focused/full floors and two-axis review, merge PR #60, and update
   the atlas/status with the exact landing SHA;
5. refill disjoint qualification lanes from the reconciled ownership DAG.

No production defect repair is authorized by this status file alone. The goal,
atlas classification, and live writer lease must all authorize the slice.
