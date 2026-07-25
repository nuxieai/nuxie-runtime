# Editor Next Runtime Defect Status

This is the resume ledger for
`docs/editor-next-runtime-defect-goal.md`. The detailed ownership plan is
`docs/editor-next-runtime-defect-port-map.md`, and the machine-readable source
of truth is `docs/editor-next-runtime-defect-atlas.toml`.

## Current state

- phase: `F-ED-03` merge closeout plus file-disjoint `F-ED-11A` /
  `LOC-019`;
- pinned C++ runtime: `d788e8ec6e8b598526607d6a1e8818e8b637b60c`;
- investigation base: `05d45f07d87b33665167de0869b7db7b009bf8fe`;
- Editor's last consumed runtime:
  `95027109c89f651835c76646ebf4d8734f032f07`;
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
lease changes.

`F-ED-03` is independently reviewable in PR #49 with its complete local floor
green. Its required CI parity job is blocked by the repository runner lacking
a matching `llvm-nm`, the same harness defect present on `main`; that harness
repair remains a separate approval-gated change.

`F-ED-11A` has localized `LOC-019` independently of `LOC-009`. The WebGPU
device and valid draw succeed; a clean `GPUDevice.popErrorScope()` fulfills
with JavaScript `null`, which wasm-bindgen 0.2.126's undefined-only
`JsOption::into_option` path sends into vendored wgpu's `Error::from_js`.
The narrow repair at the existing BrowserWebGpu vendor boundary is red/green
in real Chrome, and `make browser-webgpu-only-check` is green with a
deterministic 64-pixel clean-scope row. The complete executor floor is green:
414 runtime tests, 141 public `nuxie` tests, 721 pinned-C++ probes, ordinary
and scripted 317/317 entries plus 647/647 segments with zero divergences,
1,468/1,468 renderer rows, C API and workspace suites, and 8.74 MiB under the
9 MiB ceiling. A non-draft PR, independent CI, merge, and unchanged Editor
P14-C06 consumption remain.

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

## Next queue

1. clear the separate `llvm-tools` CI harness prerequisite, merge `F-ED-03`,
   and hand its exact SHA to Editor for P09-C01;
2. run the complete required floors for `F-ED-11A`, open it as a non-draft
   one-defect PR, merge it when green, and hand the exact SHA to Editor for the
   unchanged P14-C06 matrix;
3. qualify `LOC-009` independently from `LOC-019`; `RuntimeRejected` is not a
   diagnosis;
4. qualify and close `F-ED-04` and requalify `F-ED-06`, `F-ED-07`,
   `F-ED-08`, and the remaining `F-ED-11` draw row under the WebGPU-only
   support contract—no WebGL2 repair or fallback;
5. keep `LOC-007` and every other reserved runtime-owner finding with the
   active FL executor while file-disjoint evidence/API work proceeds;
6. burn down the remaining queues in
   `docs/editor-next-runtime-defect-goal.md` through Editor consumption and
   the complete 27-child product matrix.

No production defect repair is authorized by this status file alone. The goal,
atlas classification, and live writer lease must all authorize the slice.
