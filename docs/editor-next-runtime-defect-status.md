# Editor Next Runtime Defect Status

This is the resume ledger for
`docs/editor-next-runtime-defect-goal.md`. The detailed ownership plan is
`docs/editor-next-runtime-defect-port-map.md`, and the machine-readable source
of truth is `docs/editor-next-runtime-defect-atlas.toml`.

## Current state

- phase: `Q0` control-plane provenance reconciliation;
- pinned C++ runtime: `d788e8ec6e8b598526607d6a1e8818e8b637b60c`;
- investigation base: `e72323c808b91d706ba3b745396beaca7accd69a`;
- Editor's last consumed runtime:
  `e72323c808b91d706ba3b745396beaca7accd69a`;
- rows: 25 defects plus the reserved `LOC-010` tombstone;
- closed rows: `RT-ED-001`, `RT-ED-002`, `RT-ED-006`, `LOC-003`,
  and `LOC-004`;
- open rows: 20;
- formal product children in the landed Editor snapshot: 8;
- candidate-linked product children: 20;
- union: 27, with only `P09-C01` overlapping;
- correction rows: 12.
- fixture rows: 25 total, with `RT-ED-001`, `RT-ED-002`, and `LOC-003`
  directly qualified.
- supported browser backend: WebGPU only, landed in runtime PR #47 at
  `95027109c89f651835c76646ebf4d8734f032f07`.

The FL reservation in the atlas remains deliberately conservative during Q0.
No runtime, renderer, Scene, state-machine, Editor product, or compiler file is
authorized by this provenance-only slice.

The immutable Editor checkpoint records the completed WebGPU-only consumption
through runtime `e72323c8`: `P14-C01` is 4/4 green, `P14-C06` is 17/17 green,
and RT-ED-003 direct presentation is consumed. Q0 does not use those product
results to self-promote an atlas row; independent state promotion remains a
separate step.

## Editor source snapshot

The Editor executor committed and pushed the current reviewed source snapshot
at `4da896beb5ec6815f6b01a2433875274a321d06c`. Its worktree and remote branch
both resolve to that exact SHA, its runtime gitlink is
`e72323c808b91d706ba3b745396beaca7accd69a`, and the three source artifacts
are clean.

The landed snapshot hashes are:

- proposal:
  `0d19ae37038b145e2f67c08bfcaad49122be963f3cdc146fbad625f1600a0983`;
- runtime defects:
  `01fe2cadfeddf7d42338d026c012d47ce88bedc28146608b0fa33cbf97f96d67`;
- parity ledger:
  `a0664bf40813b2ba332d63c3deddfeeb49e15f0b7ec10fdd45e0f2cc78b37b04`.

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

`F-ED-03` / `RT-ED-005` is classified as an API-surface gap, not a
low-level runtime defect. Production commits
`4eec745b704e9920f67098138963dc973e7b2d87` and
`e2d274d8d3b8de3af705d18506a6d48eadebfc0c` port the pinned C++ generic
`DataBind.propertyKey` authoring contract into Scene while leaving the
FL-owned runtime mechanism unchanged. They add typed number/color binds,
converter-free direction selection, stable `LayoutComponentStyle` targets
for all four padding keys, exact target/property collision identity,
converter output validation matching C++ `Input`/`None`/`Any` semantics, and
encoded `File::import` behavior tests. Independent review found the missing
converter-free direction surface; the follow-up now proves numeric
`ToSource`, numeric source-first `TwoWay`, and color source-first `TwoWay`
through exact re-import and reverse propagation. The runtime repair landed in
PR #49 at exact merge `08286481b4e7420768f625f901a944f313b84903`.
The row remains `reported` until orchestrator verification and unchanged
`P09-C01` Editor consumption evidence are complete. Its executor battery was
green:
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

## Next queue

1. finish Q0 by validating the immutable Editor checkpoint and the three
   refreshed browser stimulus hashes with `run-check.sh`;
2. independently review and merge the provenance-only Q0 diff;
3. report the exact Q0 merge evidence and propose the next atlas defect;
4. do not begin that defect until its scope is confirmed.

No production defect repair is authorized by this status file alone. The goal,
atlas classification, and live writer lease must all authorize the slice.
