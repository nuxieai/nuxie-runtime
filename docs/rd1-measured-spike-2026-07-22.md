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
