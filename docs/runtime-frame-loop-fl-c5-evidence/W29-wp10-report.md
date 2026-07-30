WP10 execution is complete. No commit was created, no performance command ran, and the pre-existing `.flc5/` directory was untouched.

The aggregate publication checker is fully green, but genuine correctness failures remain. Accordingly, FL-C5 stays in `planning`; closure rows were not papered over.

### Reconciliation outcome

- W4 §C API inventory:
  - Exhaustive in-crate inventory retained.
  - Added a downstream-visible inventory covering every public API group: `1 passed / 0 failed`.
  - No production re-export repair was necessary.
- Compensation: `29/29` reconciled — `25 KEEP`, `4 DELETE`.
  - All deleted mechanisms are absent.
  - Masking differentials are recorded.
- Out-of-scope seams: `8/8 RECORDED`, each with its owning row.
- No deferred ownership row was promoted.
- Closure:
  - Members: `242/249 checked`, `7 open`.
  - Adversarial rows: `6/12 checked`, `6 open`.
  - Publication packet: `5/9 checked`, `4 open`.
- Structural checker: `56/56 passed`.
- Aggregate inventory: `341 files`, `75 mechanical member entries`, `10 gaps`.
- Trace regenerated: `103/341 C++ files`, `34 Rust modules`, `18 landmarks`.
- Candidate fingerprint: `7248` files, SHA-256 `4df412eb78915c7dd6f33f9664fdbffe4a251a94a972d19ff9efe9acbedc75fb`.

Primary artifacts: [closure](</Users/levi/dev/worktrees/nuxie-fl-c/docs/runtime-frame-loop-fl-c5-closure.md>), [trace](</Users/levi/dev/worktrees/nuxie-fl-c/docs/runtime-frame-loop-trace.json>), [external API inventory](</Users/levi/dev/worktrees/nuxie-fl-c/crates/nuxie-runtime/tests/public_api_fl_c5.rs>).

### Focused acceptance receipts

| Floor | Receipt |
|---|---:|
| `nuxie-runtime --lib` | 713 passed, 0 failed |
| `cpp_probe` | 802 passed, 2 failed |
| `nuxie --lib` | 144 passed, 2 failed |
| `nuxie public_api` | 14 passed, 1 failed |
| `nux-capi` | 19 passed, 0 failed |
| External FL-C5 API inventory | 1 passed, 0 failed |
| `runtime-frame-loop-port-test` | 56 passed, 0 failed |
| `runtime-frame-loop-port-check` | 56 passed, checker green |
| Format / `git diff --check` | green |

The public API failure is environmental: no graphics adapter was available.

### Whole-family non-performance floor

- Probe-armed workspace: scripted C++ runner clean failed on the read-only upstream checkout before the complete workspace suite started. A partial direct run passed package groups of `0, 1, 13, 24, 3, 16`, then `nuxie --lib` finished `200 passed / 5 failed`.
- Ordinary goldens: summary reported `317/317` exact entries and `647` exact segments, but the floor correctly exited red because `sound` differed at line 37 (`1` failed corpus entry).
- Scripted goldens: `0` entries executed; blocked by read-only upstream `liblibpng.a`.
- Static pixel referee: `0` rows completed; first row failed because no graphics adapter was available.
- Same-runner pixel referee: `0` comparisons; current-runtime C++ Dawn reference executable was missing.
- C API smoke: green — `draw_paths=2`, `objects=4`.
- Apple native:
  - Header checks passed.
  - Panic firewall: `1 passed`, `65 filtered`; integration `0 passed`, `15 filtered`.
  - Product tests: `48 passed / 18 failed`, consistently returning status `7` instead of `0` in the adapter-dependent paths.
- Apple XCFramework/ABI/header: fully green after redirecting module caches; `3` target builds, both XCFramework slices verified. Checksum: `cddec074a775030b3063b284b790c9dbfcf616592ea1eca39cc2ad988bd7d972`.
- Browser build/WebGPU-only: `0` browser flows completed; dependency installation failed because `index.crates.io` and Playwright dependencies were unreachable.
- Lint: green; recorded warning baselines — runtime `1156`, binary `278`, graph `78`, C API `33`.
- Size gate: `0` variants measured. It refuses a dirty tracked tree, while WP10 requires edits and the user prohibited commits.
- Trace/provenance: fully green with the receipt above.
- Performance: `0` commands run.

### Rows intentionally left unchecked

Seven member rows:

- `currentState` — generic layer reports Rust core type `0` versus pinned C++ `60`.
- `stateChangedByIndex` — same `0` versus `60` mismatch.
- `advance(seconds,newFrame)` — bound VM trigger returns `false` in Rust versus `true` in C++.
- `applyEvents` — synchronous pointer-event prefixes are lost.
- `pointerDown` — scripted pointer callbacks/deferred callback table are lost.
- `internalDataContext` — fixed binding source is not observed and deferred listener state is lost.
- `initScriptedObjects` — missing-context listener loses its retained callback table.

Six adversarial rows:

- Occurrence construction order.
- Transition search and state change.
- Hit listener and focus ownership.
- DataContext bind/rebind/clear.
- Event application and chained reports.
- Advance return and pending work.

### Breakage found

Eight distinct semantic failures were exposed:

- Two C++ differentials: generic current state and VM-trigger advance return.
- Two synchronous event/facade tests.
- Three scripting pointer/hydration lifecycle tests.
- The `sound` ordinary golden.

I also repaired evidence-only breakage: parallel C++ probe fixture collisions, a stale constructor anchor, post-file-split checker paths, trace symbol anchors, and stale trace provenance.

Final review found no documented coding-standard violations. Goal usage: `854,098` tokens over approximately 56 minutes.