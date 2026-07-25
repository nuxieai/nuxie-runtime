# Editor Next Runtime Defect Status

This is the resume ledger for
`docs/editor-next-runtime-defect-goal.md`. The detailed ownership plan is
`docs/editor-next-runtime-defect-port-map.md`, and the machine-readable source
of truth is `docs/editor-next-runtime-defect-atlas.toml`.

## Current state

- phase: `F-ED-11` / `LOC-009` publication and independent verification;
- pinned C++ runtime: `d788e8ec6e8b598526607d6a1e8818e8b637b60c`;
- investigation base: `08286481b4e7420768f625f901a944f313b84903`;
- Editor's last consumed runtime:
  `ef9dcedd82265efc0184f4f59d5f6aaab0b56cd9`;
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

`F-ED-11` has now qualified `LOC-009` as a separate structural
mistranslation. Pinned C++ WebGPU selects authored target-0 whole-module WGSL
plus mandatory target-16 `BindingMap`, preserves arbitrary logical and
physical entry records in declaration order, and creates one shared shader
module. Rust instead selected retired target-1 GLSL, discarded sidecars, split
the stages, and cross-translated GLSL to WGSL. The repair at exact code
checkpoint `22e4900243ee92a436afc1609f456525e8312352` ports the complete
consumer lifecycle, including C++ last-wins descriptors, bare and named entry
selection, retained Naga usage information, fail-closed target-16 visibility,
and direct authored-WGSL submission. The old translator and dependency closure
are deleted.

The post-review executor battery is green: 414 runtime tests, 144 public
`nuxie` tests, 721 pinned-C++ probes, ordinary and scripted 317/317 entries
plus 647/647 segments with zero divergences, 1,468/1,468 renderer rows,
required-WebGPU real-Chrome, C API, Apple, workspace, renderer-consumer,
frame-loop, B-6, lint, format, and diff floors. The release-size SDK closure
is 7,984,504 bytes without scripting and 8,885,736 bytes with scripting,
both below the 9,437,184-byte ceiling. Independent
re-review is clean with no P0/P1/P2 findings; merge and unchanged Editor
`P14-C01` consumption remain.

## Editor source snapshot

The Editor executor committed and pushed the current reviewed source snapshot
at `27ef7d471c3034aba4a4b839d2c8150d3bcb40c3`. It includes the intentional
correction moving `P04-C12` off `RT-ED-004` and linking it to `LOC-018`, the
WebGPU-only Editor support state, and the current product-child results. The
three source artifacts are clean at that checkpoint.

The landed snapshot hashes are:

- proposal:
  `148d11f206edc41caad1f48cae0810b268456b2e220ba6253ac6d04ef450b9db`;
- runtime defects:
  `a610201cc34c95bd5ff0838d95228af3983f38327ebbef87b253c3e49a357b9c`;
- parity ledger:
  `d89e185411197c5d98c7e1a01cb414022de988a5ba4194670fcdefb9c39b7c97`.

The earlier reviewed hashes remain in this file's Git history, but their formal
dependency map is stale and must not be used for qualification. Any later
artifact change makes the source-root check fail until a newly reviewed Editor
checkpoint is recorded.

Editor later consumed runtime
`ef9dcedd82265efc0184f4f59d5f6aaab0b56cd9`. Producer checkpoint
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

1. publish and merge the independently reviewed executor-green `LOC-009`
   repair, then have Editor consume the exact merge SHA and rerun unchanged
   `P14-C01`;
2. rerun unchanged Editor `P14-C06` against merged `LOC-019` runtime
   `ef9dcedd82265efc0184f4f59d5f6aaab0b56cd9`;
3. obtain independent orchestrator verification for merged `F-ED-03` and have
   Editor rerun unchanged `P09-C01`;
4. qualify and close `F-ED-04` against its existing low-level runtime path
   without touching reserved modules;
5. requalify `F-ED-06`, `F-ED-07`, and `F-ED-08` under the
   WebGPU-only support contract—no WebGL2 repair or fallback;
6. keep `LOC-007` and every other reserved runtime-owner finding with the
   active FL executor while file-disjoint evidence/API work proceeds;
7. burn down the remaining queues in
   `docs/editor-next-runtime-defect-goal.md` through Editor consumption and
   the complete 27-child product matrix.

No production defect repair is authorized by this status file alone. The goal,
atlas classification, and live writer lease must all authorize the slice.
