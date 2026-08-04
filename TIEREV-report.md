# TIEREV lane report

Branch: `levi/tier-evidence`

Baseline: `e8726db8ffc689d3b19f1c0a55794aa6daf9956d`

Pinned C++ runtime: `4ac7b32798da0482e441ef09304dc3b480ed3ee5`

## Result

Both requested evidence gaps are filled.

- V1 has a composed, full-stream differential at
  `make e2e-composed-compare`. Each enrolled fixture is loaded once per
  runtime and driven through one ordered session containing advances, pointer
  input, typed view-model mutation, resize, semantics reads, samples, and
  frames. Ten existing multi-feature fixtures are enrolled; all 10 sessions
  and all 40 stream segments compare exactly.
- Tier-5 now has a five-largest-fixture release measurement for 100 sequential
  frames, median of five runs per runtime, plus raw Rust/C++ static-archive
  sizes. The evidence is intentionally adverse rather than normalized away:
  Rust is slower on all five fixtures and its raw static archive is larger.
  Full method, numbers, caveats, and raw reports are in
  `docs/perf-size-evidence.md`.

No parity tolerance changed. The only parity-register edits append evidence
pointers to the existing V1 and V10 evidence cells; their gap and exit-gate
text is unchanged.

## Required bootstrap

Run first, in the requested order, with no `/tmp` use:

```text
rsync -a /Users/levi/dev/nuxie-runtime/fixtures/ fixtures/   PASS
make fixtures                                                PASS
make cpp-probe                                               PASS
```

## Composed oracle

The comparator's `--require-composed-session` mode is layered on the existing
side-channel verb stream. It requires both scripts and rejects any actual
runner stream missing an advance, pointer input, view-model mutation, resize,
semantics record, sample, or frame. It compares the complete emitted stream;
there is no composed-mode projection or relaxed verification.

The 10 fixtures in `e2e-composed.toml` cover listeners, typed data converters,
relative/collapsible binding, and boolean, color, enum, and string view-model
families. The gate result was:

```text
golden-compare summary: entries=10 exact=10 exact-segments=40
  side-channel-segments=40 divergences=0 unsupported-feature=0
  post-zero-runtime-hang=0 post-zero-incomplete-stream=0 not-yet-implemented=0
golden-compare composed sessions: exact=10/10
```

The unchanged full scripted gate also passed under all existing row contracts:

```text
entries=363 exact=342 exact-segments=1114 side-channel-segments=1109
divergences=16 unsupported-feature=0 not-yet-implemented=5
```

## Tier-5 measurements

The tracked metric is median per-run `advance + draw` wall time divided by 100
frames. `perf-compare` derives the combined phase inside each invocation before
choosing the median.

| Fixture | C++ ms/frame | Rust ms/frame | Rust/C++ |
|---|---:|---:|---:|
| `text_vertical_trim_test` | 0.004629 | 0.117580 | 25.402x |
| `jellyfish_test` | 0.000500 | 0.011172 | 22.341x |
| `echo_show_demo` | 0.050823 | 2.085713 | 41.039x |
| `car_widgets_v01` | 0.035157 | 76.895463 | 2,187.196x |
| `zombie_skins` | 0.037295 | 1.494437 | 40.071x |

Raw release archives:

| Archive | Bytes | Relative size |
|---|---:|---:|
| Rust `libnux_capi.a` | 108,554,768 | 4.565x C++ |
| C++ `librive.a` | 23,781,600 | 1.000x |

Raw archive bytes are not final linked footprint; the existing post-link
`make size-report` contract remains authoritative for the SDK budget.

## Verification

Green:

- `make e2e-composed-compare`
- `make scripted-golden-compare` (existing contracts and floors unchanged)
- `cargo test -p golden-compare` (22 passed)
- `cargo test -p perf-compare` (33 library + 22 binary tests passed)
- scoped `cargo fmt --check` for `golden-compare` and `perf-compare`
- `make check`
- `make runtime-frame-loop-port-check` (125 tool tests and all live checks)
- `make rust-sources-fresh`
- `jq empty docs/evidence/tier5-2026-08-04/*.json`
- `git diff --check`

Two broader live checkers expose baseline metadata drift unrelated to this
lane, and were not altered to manufacture green results:

- `make b6-audit-check` expects upstream `d788e8ec…`, while this lane and the
  required bootstrap use pinned upstream `4ac7b327…`.
- `make runtime-drawing-port-check` passes its seven unit tests, then reports
  that ownership row `shape.path_composer` names absent anchor
  `fn update_runtime_path_composer` in `crates/nuxie-runtime/src/draw.rs`.

The global `cargo fmt --all --check` also observes pre-existing formatting
drift in unrelated runtime files; the two touched Rust tools pass their scoped
format checks.

## Commits

- `e982cfed` — `[TIEREV] Add composed session differential`
- `e1f33a42` — `[TIEREV] Report combined advance and draw timing`
- `14de19c1` — `[TIEREV] Record Tier 5 performance and size evidence`
- `a1c42f7f` — `[TIEREV] Correct Tier 5 largest-fixture set`

## Review

The final two-axis review found no documented-standards violations. Its Spec
pass caught that the first performance enrollment had skipped the true
third-largest fixture, `echo_show_demo`, and had consequently included the
sixth-largest fixture. Commit `a1c42f7f` replaces that measurement and raw
report, recomputes the five-fixture aggregate, and restores the exact requested
ranking before handoff.
