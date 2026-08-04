# Performance Gate Lane Report

Date: 2026-08-04

Branch: `levi/perf-gate`

Register row: V10

## Outcome

The hot-loop performance check is now a blocking, per-file ratchet over 24 corpus files. Every landing and CI run executes the same release-vs-release, advance-plus-draw comparison, prints the complete per-file ratio table, and fails if any measured ratio exceeds its checked-in ceiling. V10 remains open for the remaining direct-parity work: all per-file ratios must eventually reach `<= 1.0`.

## Row commits

1. `9346fe56` — broaden the hot-loop corpus and add manifest validation.
2. `fcf56780` — add repeatable 100-frame measurement controls, CPU-pinning wrapper, and baseline evidence.
3. `71b780d7` — enforce per-file ceilings and add the tighten-only helper.
4. `a09930bb` — wire the blocking gate into local landing and CI.
5. `beed1895` — add this report, V10 register status, and blocking scorecard consumption.
6. The review-closeout commit enables real scripted coverage and hardens measurement isolation, evidence enforcement, and three-session tightening.

## Corpus

[`perf-corpus.toml`](perf-corpus.toml) contains 24 files selected from `corpus.toml`. It includes the largest practical input-free fixtures and explicit coverage for text-heavy scenes, lists and virtualization, nested artboards, scripted scenes, and layout-heavy scenes. Every entry carries a selection note and category tags.

The very large `data_viz_demo` fixture was measured during corpus selection but excluded because a single current-runtime 100-frame sample took minutes. Keeping it in every landing would make the gate operationally fragile rather than thin. The scripted text-run fixture replaces it while preserving scripted/text coverage.

## Measurement and stability

The comparator follows [`docs/perf-size-evidence.md`](docs/perf-size-evidence.md): both runtimes are scripting-enabled release builds, Rust executes embedded scripts, each file advances and draws 100 sequential frames at 60 Hz, and the reported time is the median of five samples. Ratios are computed per file as current Rust divided by the pinned reference runtime.

The checked-in baseline is the worst ratio from four independent median-of-five sessions. The fourth was retained because the first out-of-sample gate exposed a boundary flake. Each ceiling is exactly `ceil(baseline_ratio * 1.15)`, providing the requested 15% stability margin without hiding regressions. The four raw reports and their SHA-256 digests are recorded in the evidence document; generated reports live under `target/`, never `/tmp`.

On Linux, [`tools/perf-gate/run-pinned.sh`](tools/perf-gate/run-pinned.sh) uses `taskset` to select a highest-frequency performance CPU when the platform exposes one. macOS does not provide a supported process CPU-affinity API, so the wrapper records that it uses the default scheduler. The four-session evidence documents the observed variance and is deliberately reflected in the worst-session baseline.

## Ratchet behavior

`make perf-gate` validates the manifest and evidence metadata, runs the measurements, prints all 24 ratios and ceilings, and fails on the first aggregate result containing any ceiling breach.

`make perf-gate-tighten` collects three independent median-of-five sessions and applies their per-file maximum only when it improves a file's checked-in baseline. It can lower a baseline and ceiling, but it rejects or preserves any value that would loosen the gate. Manifest validation independently rejects ceilings that do not match the 15% formula, preventing silent manual loosening.

## Wiring

- `make perf-gate` is the canonical blocking command.
- `tools/land.sh` includes `perf-gate` in a serial timing-gate list after the CPU-heavy parallel checks, then publishes the full ratio table from either a fresh or cached pass.
- `.github/workflows/ci.yml` runs the same target in a blocking `perf-gate` job and uploads `target/perf-gate.json`; the earlier optional `continue-on-error` measurement was removed.
- The parity scorecard consumes the blocking artifact. Missing evidence or a ceiling breach is red, the result is partial while any ratio remains above 1.0, and it is green only at direct parity.

## Validation

- `make perf-gate` — PASS, 24 files; full ratio table printed.
- `make scripted-golden-compare` — PASS; 363 entries, 346 exact, 1,126 exact segments, 1,121 side-channel segments, 12 diverging, 0 unsupported-feature, 5 not-yet.
- `make rust-sources-fresh runtime-frame-loop-port-check rust-attribution-check perf-corpus-check` — PASS.
- `make check` — PASS (existing compiler warnings only).
- `cargo test -p perf-compare --bin perf-compare` — PASS, 25 tests.
- `python3 -m unittest tools/perf-gate/test_perf_gate.py` — PASS, 9 tests.
- `python3 -m unittest tools/parity-scorecard/test_parity_scorecard.py` — PASS, 24 tests.
- Shell syntax checks for the landing and CPU-pinning scripts — PASS.
- CI workflow YAML parse — PASS; `actionlint` was not installed in this worktree.
