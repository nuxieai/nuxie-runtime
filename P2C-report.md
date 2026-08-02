Implemented the P2-c ScriptedLayout surface without touching layout-engine internals.

Highlights:

- Added layout-node access, hydration dirt propagation, Lua `measure`/`resize` callbacks, and layout-driven resize propagation.
- Ported five direct upstream callback tests.
- Added the 20+20-frame Rows/Columns differential: all 205 semantic draws match.
- `script_layout_test` golden is exact at 5,541 bytes.
- Updated all manifest statuses/counts: `5 / 36 / 18 / 87 / 2`.
- Cargo generated the one-line `Cargo.lock` update; it was not hand-edited.
- No commit created.

All required gates pass except full `make scripted-golden-compare`, which encounters the existing unrelated `audio_script` transform mismatch. I reproduced the identical failure from a clean `origin/main` archive; the scripted-layout entry itself passes exactly.

Full details: [P2C-report.md](/Users/levi/dev/worktrees/nuxie-p2f-audio/P2C-report.md)