# Editor Next Runtime Defect Status

This is the resume ledger for
`docs/editor-next-runtime-defect-goal.md`. The detailed ownership plan is
`docs/editor-next-runtime-defect-port-map.md`, and the machine-readable source
of truth is `docs/editor-next-runtime-defect-atlas.toml`.

## Current state

- phase: `F-ED-11A` / `LOC-019` independent verification and product
  consumption after `LOC-009` closure;
- pinned C++ runtime: `d788e8ec6e8b598526607d6a1e8818e8b637b60c`;
- investigation base: `e72323c808b91d706ba3b745396beaca7accd69a`;
- Editor's last consumed runtime:
  `e72323c808b91d706ba3b745396beaca7accd69a`;
- rows: 25 defects plus the reserved `LOC-010` tombstone;
- closed rows: `RT-ED-001`, `RT-ED-002`, `RT-ED-006`, `LOC-003`,
  `LOC-004`, and `LOC-009`;
- open rows: 19;
- formal product children in the landed Editor snapshot: 8;
- candidate-linked product children: 20;
- union: 27, with only `P09-C01` overlapping;
- correction rows: 12.
- fixture rows: 25 total, with `RT-ED-001`, `RT-ED-002`, `LOC-003`, and
  `LOC-009` directly qualified.
- supported browser backend: WebGPU only, landed in runtime PR #47 at
  `95027109c89f651835c76646ebf4d8734f032f07`.

The active FL executor owns FL-A and the complete reservation recorded in the
atlas. F-ED may write only its new evidence/checker/fixture paths until that
lease changes. The intended published FL-A tip is
`c4d81801898563c23f1b4f68e0c9ef0df83b1d41`; its uncommitted owner work and
`LOC-007` remain outside F-ED.

`F-ED-11A` localized `LOC-019` independently of `LOC-009`. The WebGPU
device and valid draw succeed; a clean `GPUDevice.popErrorScope()` fulfills
with JavaScript `null`, which wasm-bindgen 0.2.126's undefined-only
`JsOption::into_option` path sends into vendored wgpu's `Error::from_js`.
The narrow repair at the existing BrowserWebGpu vendor boundary is red/green
in real Chrome, and `make browser-webgpu-only-check` is green with a
deterministic 64-pixel clean-scope row plus a concrete validation-error row.
That repair landed in runtime PR #51 at exact merge
`ef9dcedd82265efc0184f4f59d5f6aaab0b56cd9`; unchanged Editor `P14-C06`
consumption remains.

`F-ED-11` qualified `LOC-009` as a separate structural mistranslation.
Pinned C++ retains one backend-neutral file-owned `ShaderAsset` across its
bare and folder-qualified aliases, then creates a fresh `ScriptedShader` and
backend module for every successful lookup occurrence. Entries within that
occurrence share the module; a second same-name lookup does not. WebGPU
selects authored target-0 whole-module WGSL plus mandatory target-16
`BindingMap` at lookup and preserves arbitrary logical and physical entries
in declaration order. Rust instead selected retired target-1 GLSL, discarded
sidecars, split stages, cross-translated GLSL to WGSL, and initially retained
the wrong lookup ownership.

PR #52 merged the first target-0/16 consumer at `88fdc5a6`, but that partial
landing left eager, name-keyed ShaderAsset ownership. Corrective PR #54 ports
the complete file-owner and lookup-occurrence lifecycle, independent explicit
stage handles, omitted-fragment handle reuse, nil-and-continue failures, and
occurrence-keyed pipeline retention. It merged at exact SHA
`7f1450dc22ca7370eac9dc9f422351c2dfcc07ee`; remote `refs/heads/main` was
verified equal. The target-1 translator and dependency closure remain deleted.

The post-review executor battery is green: 414 runtime tests, 144 public
`nuxie` tests, 721 pinned-C++ probes, ordinary and scripted 317/317 entries
plus 647/647 segments with zero divergences, 1,468/1,468 renderer rows,
required-WebGPU real-Chrome, C API, local Apple, workspace, renderer-consumer,
frame-loop, B-6, lint, format, and diff floors. The release-size SDK closure
is 7,984,520 bytes without scripting and 8,885,736 bytes with scripting,
both below the 9,437,184-byte ceiling. Independent
re-review is clean with no P0/P1/P2 findings. Hosted parity, Phase R pixels,
and every other non-Apple canonical check passed. The optional 1.0x perf
artifact failed at 1.554x aggregate on the separate known FL performance
signal; LOC-009 touches no frame-loop owner, and no gate was loosened. Four
hosted Apple jobs remained queued without assigned
self-hosted runners and were cancelled after merge; they are recorded as
infrastructure-queued, not green. Their exact-head local Apple, C ABI/header,
XCFramework, and release floors passed.

Independent orchestration then verified the exact merged LOC-009 repair and
its unchanged floors. The unchanged `P14-C01` command executed at clean
checkpoint `d5bbbb31178b8b29c40747fdd21a829348ede624` against runtime
`e72323c808b91d706ba3b745396beaca7accd69a` and passed 4/4 in 1.6 minutes.
Superseding checkpoint `4da896beb5ec6815f6b01a2433875274a321d06c`
descends from d5bbbb, retains that runtime, and records the final LOC-009
qualification. LOC-009 is therefore closed. The retained console is
`/private/tmp/nuxie-editor-retained-evidence/p14-c01-e723.log`; its SHA-256 is
`c2324d04cf1baa6ac024ae3b4f0607ca3a4ad64ecace7ac67e612707480527f0`,
and the complete Playwright report/results archive is
`/private/tmp/nuxie-editor-retained-evidence/p14-c01-e723.tar.gz`; its SHA-256
is
`515f53c5710c8069bf60f4c64c64568c219ee3cfb242fbe8e59846b1c0f96bd3`.

## Editor source snapshot

The Editor executor committed and pushed superseding immutable
source/dependency snapshot
`4da896beb5ec6815f6b01a2433875274a321d06c`. It pins runtime `e72323c8`,
includes the intentional correction moving `P04-C12` off `RT-ED-004` and
linking it to `LOC-018`, and records the final browser/LOC-009 qualification.
The three source artifacts are clean at that checkpoint. The retained log and
archive above remain the original unchanged product execution from d5bbbb,
carried forward without claiming a rerun.

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

The clean snapshot consumes runtime
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

1. independently verify `LOC-019` and its unchanged `P14-C06` evidence against
   the consumed runtime `e72323c808b91d706ba3b745396beaca7accd69a`;
2. obtain independent orchestrator verification for merged `F-ED-03` and have
   Editor rerun unchanged `P09-C01`;
3. qualify and close `F-ED-04` against its existing low-level runtime path
   without touching reserved modules;
4. requalify `F-ED-06`, `F-ED-07`, and `F-ED-08` under the
   WebGPU-only support contract—no WebGL2 repair or fallback;
5. keep `LOC-007` and every other reserved runtime-owner finding with the
   active FL executor while file-disjoint evidence/API work proceeds;
6. burn down the remaining queues in
   `docs/editor-next-runtime-defect-goal.md` through Editor consumption and
   the complete 27-child product matrix.

No production defect repair is authorized by this status file alone. The goal,
atlas classification, and live writer lease must all authorize the slice.
