# Editor Next Runtime Defect Status

This is the resume ledger for
`docs/editor-next-runtime-defect-port-map.md`. The machine-readable source of
truth is `docs/editor-next-runtime-defect-atlas.toml`.

## Current state

- phase: `F-ED-00A`;
- pinned C++ runtime: `d788e8ec6e8b598526607d6a1e8818e8b637b60c`;
- investigation base: `efb6ad128d6aac7b81ed57d4a8b76eb9259ec833`;
- Editor's last consumed runtime:
  `13aedd6d92de0991eed8dc3fda085db2dff18d48`;
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

The active FL executor owns FL-A and the complete reservation recorded in the
atlas. F-ED may write only its new evidence/checker/fixture paths until that
lease changes.

## Editor source snapshot

The Editor executor committed and pushed the reviewed source snapshot at
`8fb90154af8d50847f4efea71ad56a1da6d9e8bf`. It includes the intentional
correction moving `P04-C12` off `RT-ED-004` and linking it to `LOC-018`. The
three source artifacts are clean at that checkpoint.

The landed snapshot hashes are:

- proposal:
  `2ff7bf172d5867808078b2ad10d1b0315c502fbb43194395b39c7febb5abb130`;
- runtime defects:
  `6c99b42c12ea3f698f8da66dcb70cfb9a35c9dc0420a04bd1f45b70520d12055`;
- parity ledger:
  `da024a3450d99769c0b2ab847ec9259e23feee3a74d438264cb18429b5cb85b4`.

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

1. run `F-ED-07`'s exact 402×874 radius-57 current-pin rounded-clip
   qualification;
2. qualify `F-ED-03` and `F-ED-04` against their existing low-level runtime
   paths without touching reserved modules;
3. qualify `F-ED-06`'s browser presentation/readback seam;
4. fan out the remaining direct evidence while routing every runtime-owner
   finding to the active FL executor.

No production defect repair is authorized by this status file alone.
