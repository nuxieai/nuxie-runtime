# RD-1 measured live-traversal spike

Date: 2026-07-22

Pre-spike commit: `076b4139a574c98b2d606386589bed61e55f314a`

Upstream runtime: `d788e8ec6e8b598526607d6a1e8818e8b637b60c`

This is the mandatory RD-1b checkpoint. No prepared-frame, command-stream,
path-cache, or epoch-bridge deletion is part of this spike. The experimental
runner switch, `NUXIE_RD1_LIVE_TRAVERSAL_SPIKE=1`, keeps sorted drawable
topology and object render resources retained but materializes the scene
command frame from the live artboard traversal on every draw. Normal runtime
and runner behavior keep the switch off.

## Representative slice

The slice covers an animated shape, an image, and a nested artboard:

- `animation_reset_cases` at five sample times
- `image_fit_alignment` at its corpus sample
- `nested_artboard_opacity` at its corpus sample

Both measurements used release runners, 10 measured iterations, 3 warmups,
and a 10,000-repeat runner hot loop. The measurement ceiling was opened to
`PERF_MAX_RATIO=999` only so both observations could be captured; no checked-in
gate or acceptance threshold changed.

| Segment | Prepared baseline min (ms) | Live traversal min (ms) | Live / prepared | Delta |
| --- | ---: | ---: | ---: | ---: |
| `animation_reset_cases@0` | 1.806875 | 11.032125 | 6.106x | +510.6% |
| `animation_reset_cases@0.25` | 1.791125 | 11.478584 | 6.409x | +540.9% |
| `animation_reset_cases@0.5` | 1.786959 | 11.182709 | 6.258x | +525.8% |
| `animation_reset_cases@0.75` | 1.793292 | 11.182041 | 6.235x | +523.5% |
| `animation_reset_cases@1` | 1.793417 | 10.641750 | 5.934x | +493.4% |
| `image_fit_alignment@0` | 1.461250 | 7.874250 | 5.389x | +438.9% |
| `nested_artboard_opacity@0` | 1.100625 | 4.267584 | 3.877x | +287.7% |
| **Aggregate** | **11.533543** | **67.659043** | **5.866x** | **+486.6%** |

The prepared observation was 1.670x C++ in aggregate; the spike observation
was 8.936x C++. The controlled Rust-to-Rust result is the checkpoint number:
**live traversal is 5.866x the prepared feed, a 486.6% increase** on this
slice. The spike deliberately pays temporary `RuntimeDrawCommand`
materialization cost and therefore establishes a conservative warning, not a
forecast for the final object-owned live feed. RD-C1/RD-C2 must remove that
materialization seam before scene-cache cutover.

The user accepted the 5.866x result as a property of this temporary seam and
authorized RD-1 to proceed. A second measured checkpoint is binding: after
RD-C1/RD-C2 remove command materialization and before any scene-cache deletion,
rerun the same performance comparison and report the delta to the user. Scene
cache demolition remains blocked until that checkpoint is reviewed.

The ignored raw reports are reproducible at:

- `target/rd1-spike-baseline-perf.json` — SHA-256
  `dee957e7a633d4c6a2d8c644ef82cc0344e1b449ffe26ce46ac7768b7d96a231`
- `target/rd1-spike-live-perf.json` — SHA-256
  `89dfa192762df0accfedf3c5bb2e9ba8c85d931aa75f85547c9608fa650719ce`

The production diff used to build the immutable B runner has SHA-256
`58474d436ca54b9768b6b18a05880f8dc5aac7fb2612be8281e552a4e6457196`.

## R4 timing gate

`make r4-timing-gate` ran four complete A-B-B-A brackets with the checked-in
12% host-idle-spread fence unchanged. Every bracket completed its four runner
measurements, then failed closed in `validate-host-load`; consequently the
comparator produced no valid B/A decision.

| Attempt | Host idle spread | Result |
| --- | ---: | --- |
| v2 | 25.94% | invalid: greater than 12% |
| v3 | 49.30% | invalid: greater than 12% |
| v4 | 21.12% | invalid: greater than 12% |
| v5 | 20.83% | invalid: greater than 12% |

The pinned runner hashes were C++ baseline
`2286f5af1ca04d7c658920e27ece455d406f4ec1776b26f5c75228134b3f7fa5`, A
`cc469b21af4fd905d40bfb01ff905e3f0d008c99ada0ee224d2fc8263beada0a`, and B
`82b4ee55f6a5a94ae8d37e86a9b2e1d12ac5d193aec29d99bcb73d5f6f07229b`.
This checkpoint does not claim an R4 pass or a performance ratio from the
invalid brackets.

The follow-up R4 rerun is explicitly deferred until a quiet host is available.
The checked-in 12% host-idle-spread fence remains unchanged and no invalid
bracket is promoted to evidence.

### Post-cutover checkpoint attempts

After RD-C1 through RD-C6 removed current-object command materialization from
ordinary renderer feed, the checkpoint was rerun with immutable
source-identified executables:

| Attempt | A source | B source | Host idle spread | Result |
| --- | --- | --- | ---: | --- |
| 20260723T072026Z-90912 | `076b4139` | `d335d4a1` | 65.15% | invalid: greater than 12% |
| 20260723T081932Z-60125 | `076b4139` | `6830602c` | 36.54% | invalid: greater than 12% |
| 20260723T154251Z-36478 | `076b4139` | `307b0db7` | 80.60% | invalid: greater than 12% |
| 20260723T162631Z-30981 | `076b4139` | `307b0db7` | 17.18% | invalid: greater than 12% |

All four used pinned C++ `d788e8ec` and the canonical A-B-B-A sequence. Each
completed its timed legs but failed closed in `validate-host-load`; none
produced an accepted comparison. The third attempt rebuilt and first-launched
all measurement executables, then began the requested idle interval; the user
directed the run to start after approximately four minutes instead of ten. The
unchanged fence rejected the resulting 0.00%–80.60% idle range. The user then
authorized one new immediate attempt with the same immutable runner hashes;
its 65.92%–83.10% idle range produced 17.18% spread, still above the fence.
Neither attempt ran `perf-hot-loop` or produced a comparison. These rows are
provenance records, not performance evidence, and RD-C7 remains blocked.

On 2026-07-23 the user removed host-idle spread as an acceptance condition and
accepted testing in the current environment. Boundary samples and aggregate
spread remain recorded telemetry, but there is no idle-spread threshold or
host-load rejection phase. Future runs proceed through comparison regardless
of sampled load and are judged by immutable runner provenance, fixed A-B-B-A
ordering, paired C++ control drift, candidate-repeat drift, and performance
ratios. The historical rows above remain invalid under the policy active when
they ran.

### Current-environment checkpoint

The first run under the telemetry-only host-load policy completed the full
A-B-B-A bracket at `target/r4-timing-gate/20260723T163358Z-76070`. Host idle
ranged from 60.52% to 85.23% (24.71% spread), which is recorded but non-gating.
The comparison was produced and failed the remaining gates:

- normalized B/A was 1.068645x (+6.86%);
- normalized B/C++ was 1.149762x in aggregate;
- the worst B row was `gm-OverStroke-clockwise-atomic` at 2.156308x C++;
- C++ control drift was 1.114087x against 1.05x;
- B repeat drift was 1.060374x against 1.05x.

The canonical `perf-hot-loop` report at
`target/rd1-current-env-perf-hot-loop-20260723T1636Z.json` also failed its
unchanged 1.0x threshold. Across 11 entries, min-based aggregate Rust/C++ was
61.278871x (2,505.782499 ms / 40.891460 ms). The dominant row was
`ai_assitant@0` at 229.451367x total and 577.854015x draw; the remaining rows
ranged from 4.623x to 8.955x. These are checkpoint results for user review,
not authorization for RD-C7. Scene-cache deletion remains blocked.

## Correctness and floors

- Spike slice: 3/3 entries and 7/7 exact segments, zero divergence.
- Runtime library: 399/399 tests pass.
- Nuxie library: 140/140 tests pass.
- Pinned C++ probe: 721/721 tests pass, including next-frame listener/event
  delivery and zero-second facade return semantics.
- Scripted golden compare: 317/317 entries and 647/647 exact segments, zero
  failures.
- Ordinary golden compare: 317/317 entries and 647/647 exact segments, zero
  divergences, via `make golden-compare` with checked-in `CPP_CONFIG=debug` and
  the spike switch off.
- The seven-entry evidence cited by the earlier version of this record used
  `CPP_CONFIG=release`: the ordinary-floor observation had the spike switch
  off, and the follow-up experimental observation had it on. Both linked the
  C++ runner against `tests/out/release/librive.a`, whose extra Tools, Canvas,
  desktop GL, and ORE feature flags do not match the ordinary oracle. The same
  signature occurs only with that poisoned release oracle, so it is neither a
  parity finding nor evidence about the spike. The runner build now binds a
  dedicated archive to the pinned SHA and exact expected feature flags and
  rejects a missing or mismatched provenance stamp.
- Renderer pixels: the pinned Dawn live-reference executable was rebuilt and
  `make renderer-golden-same-runner` passes 1,468/1,468 rows with zero
  divergences and zero gated cases on the matched Apple M5 Max adapter.
- C API smoke and the probe-armed full workspace suite pass.
- Size report: renderer+scripting OFF is 8,267,064 bytes; ON is 9,184,664
  bytes. Both remain below the fixed 9 MiB (9,437,184-byte) budget.

The experimental mode remains non-default. The invalid release-oracle run is
not used to characterize its out-of-slice behavior.

## Checkpoint decision

The user accepted RD-1b and authorized RD-1b2 and RD-C1/RD-C2. The second
performance checkpoint above blocks scene-cache demolition after the
materialization seam is gone.
