# Renderer Exactness Follow-up

This document is the operating contract for closing the renderer corpus after
the renderer port. It separates release correctness from a useful, stricter health
signal so that the work has a finite end condition.

## Metrics

`make renderer-golden-same-runner` reports two independent numbers against a
separately pinned current-runtime (`d788e8ec`) C++ Dawn live replay, run
immediately before each Rust frame on the same adapter:

- **Contract exact**: decoded Rust pixels satisfy the row's reviewed
  `max_channel_delta` and `max_different_pixels` contract. This is the release
  gate and the meaning of `status = "exact"`.
- **Byte exact**: decoded Rust RGBA equals decoded C++ reference RGBA byte for
  byte. This is a non-gating health metric. It does not compare PNG container
  bytes.

The current d788 live-oracle result on Apple M5 Max is:

- Contract: `exact=1,468`, `diverges=0`, `gated=0`, `total=1,468`.
- Byte identity: `1,370/1,468` active rows.

The immutable renderer-port closure oracle remains `exact=1,468`,
`byte-exact=1,360`, `diverges=0`, `gated=0`, `total=1,468`. These results name
different C++ runtime oracles and must not be collapsed or relabeled. The
current-runtime Apple Paravirtual rerun is pending.

The byte-exact metric does not replace the contract metric. Applying `0/0` to
the whole corpus would redefine hundreds of already reviewed edge-coverage
contracts and would make output depend on GPU implementation details. It is
still valuable because stable rows can tighten independently and unexpected
movement is visible.

## Oracle Rules

1. WebGPU parity uses Rust wgpu versus C++ Dawn WebGPU in the same render mode
   on the same Metal adapter. Native C++ Metal is diagnostic evidence, not the
   primary WebGPU oracle.
2. Native Metal parity uses Rust Metal versus pinned C++ native Metal in the
   same render mode on the same reported Metal adapter. C++ Metal is the
   primary platform oracle; Rust wgpu is a separately recorded secondary
   regression oracle. Follow `docs/METAL_PORTING.md` and run the reference as
   `--reference-backend ffi-metal`.
3. Every reference change is produced by the manifest-bound C++ replay and has
   a provenance sidecar containing the adapter, stream and PNG hashes, runtime
   revision, backend-specific dependency revisions, replay-binary hash, frame,
   mode, and dimensions.
4. A row promotes only under its existing contract. Tolerance widening cannot
   close a gate.
5. A stable same-tier row may tighten to `0/0` independently. Deterministic
   CPU or intermediate-buffer oracles use `0/0` from the start.
6. Byte identity is always reported, but universal byte identity is not a
   release condition.

The Dawn capture path is reproducible with a `perf-dawn` renderer-replay build
and `capture-corpus-r-references --backend ffi-dawn`. The capture tool records
provenance automatically.

For CI hosts whose Metal adapter is not the adapter that produced the committed
PNG, `corpus-r` creates the current-runtime C++ Dawn oracle on that runner and
compares the Rust frame immediately afterward. On a new machine, point
`RIVE_RUNTIME_DIR` at a clean exact-d788 checkout, put the pinned depot tools,
Naga 30, and Premake on `PATH`, then run:

```sh
export RIVE_RUNTIME_DIR=/path/to/clean-d788-rive-runtime
make renderer-dawn-live-reference-bootstrap
make renderer-dawn-live-reference-replay
make renderer-golden-same-runner
```

The live bootstrap verifies the runtime, Dawn, and dependency revisions before
building. CI caches only the resulting FFI-only live replay under its
exact-input cache key; it always rebuilds the Rust candidate from HEAD. The
historical `renderer-dawn-reference-*` targets remain reserved for the
immutable renderer-port closure oracle and are not consumed by
`renderer-golden-same-runner`.

This mode still applies each manifest row's existing channel and pixel budgets;
it neither edits nor widens them. Each row leaves the C++ reference PNG, Rust
candidate PNG, any three-panel diff, and a TOML provenance record under the
output directory. The record binds the comparison to the stream, replay
executables, generated PNG hashes, frame, mode, tolerance, backends, and
reported adapters. Replays report adapters with one `adapter=<name>` stdout
line. A mismatch or a missing identity fails the corpus before pixel comparison
and retains the generated frames plus provenance naming the mismatch or the
side that did not report. Dynamic same-runner comparisons never claim a
verified match without two equal adapter identities.

The native Metal harness uses the same invariant. Build pin-matched upstream
archives with `make renderer-metal-reference-bootstrap`, build the C++
Metal-only replay with `make renderer-metal-reference-replay`, verify it with
`make renderer-metal-reference-check`, and exercise the green atomic tracer
with `make renderer-metal-oracle-tracers`. Before UNIV-2086 supplies a Rust
Metal candidate, that tracer target compares current Rust-wgpu to C++ Metal
only to validate the oracle plumbing; it is not a Metal-port parity claim.
After a Rust Metal candidate is selected, the target emits both primary C++
Metal and secondary Rust-wgpu comparisons. C++ Metal provenance names and
hashes the input manifest containing the Rive SHA and every linked archive.

`make renderer-metal-msaa-contract` is a green negative contract. Both the
pinned source and current upstream native Metal explicitly exclude
`InterlockMode::msaa`; native Metal selects raster-order execution or an atomic
fallback instead. The contract rejects MSAA before C++ pipeline construction,
where the old probe crashed. UNIV-2088 covers both native execution branches.
The existing Dawn MSAA PNG remains WebGPU regression evidence and must not be
relabeled as C++ Metal.

## Same-tier Migration

The former 59-row retained queue was recaptured through C++ Dawn WebGPU. All
58 new atomic captures reproduced the scout campaign byte for byte; the one
existing strict Dawn MSAA capture was already identical to the generic replay.

Fifty-five rows pass their unchanged contracts and are now exact. This closes
all 53 former native-Metal/WebGPU subpixel gates, the fixed-function Spotify
row, and atomic Interleaved Feather. None of those 55 rows is byte-identical,
which is direct evidence that contract exactness and byte identity measure
different things.

## Closed Queue

| Row | Final same-tier result | Resolution |
| --- | --- | --- |
| `riv-echo_show_demo-frame-0-clockwise-atomic` | 0 pixels beyond delta 2, max delta 1 | Keep one generic packed color plane alive across the advanced atomic segment. |
| `riv-car_widgets_v01-frame-0-clockwise-atomic` | 13 pixels beyond delta 2, max delta 3 | Same atomic color-plane lifetime fix as Echo Show. |
| `gm-interleavedfeather-msaa` | 20 pixels beyond delta 2, max delta 5 | Pack feather masks into C++-ordered logical-flush atlases and retain C++ WebGPU's 2,048 texture-allocation ceiling. |
| `gm-dstreadshuffle-clockwise-atomic` | Byte-identical, max delta 0 | Same atomic color-plane lifetime fix; no repeat variation remains. |

All four rows retain their existing `max_channel_delta = 2` and
`max_different_pixels = 32` contracts. No reference or tolerance changed.

## End Condition

This follow-up completed when:

- `make renderer-golden-same-runner` reports `exact=1,468`, `diverges=0`,
  `gated=0` under unchanged row contracts;
- every primary reference is same-tier or has an explicit reason why the
  capability is unavailable;
- the decoded-RGBA byte-exact count is reported as a secondary metric; and
- any rows tightened to `0/0` remain protected by the corpus ratchet.

There is no requirement for all 1,468 final GPU frames to become byte-identical.
