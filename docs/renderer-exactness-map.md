# Renderer Exactness Follow-up

This document is the operating contract for closing the renderer corpus after
Phase R. It separates release correctness from a useful, stricter health
signal so that the work has a finite end condition.

## Metrics

`make renderer-golden` reports two independent numbers:

- **Contract exact**: decoded Rust pixels satisfy the row's reviewed
  `max_channel_delta` and `max_different_pixels` contract. This is the release
  gate and the meaning of `status = "exact"`.
- **Byte exact**: decoded Rust RGBA equals decoded C++ reference RGBA byte for
  byte. This is a non-gating health metric. It does not compare PNG container
  bytes.

The completed corpus is:

- Contract: `exact=1,468`, `diverges=0`, `gated=0`, `total=1,468`.
- Byte identity: `757/1,468` active rows.

The byte-exact metric does not replace the contract metric. Applying `0/0` to
the whole corpus would redefine hundreds of already reviewed edge-coverage
contracts and would make output depend on GPU implementation details. It is
still valuable because stable rows can tighten independently and unexpected
movement is visible.

## Oracle Rules

1. WebGPU parity uses Rust wgpu versus C++ Dawn WebGPU in the same render mode
   on the same Metal adapter. Native C++ Metal is diagnostic evidence, not the
   primary WebGPU oracle.
2. Every reference change is produced by the manifest-bound C++ replay and has
   a provenance sidecar containing the adapter, stream and PNG hashes, runtime
   and Dawn revisions, replay-binary hash, frame, mode, and dimensions.
3. A row promotes only under its existing contract. Tolerance widening cannot
   close a gate.
4. A stable same-tier row may tighten to `0/0` independently. Deterministic
   CPU or intermediate-buffer oracles use `0/0` from the start.
5. Byte identity is always reported, but universal byte identity is not a
   release condition.

The Dawn capture path is reproducible with a `perf-dawn` renderer-replay build
and `capture-corpus-r-references --backend ffi-dawn`. The capture tool records
provenance automatically.

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

- `make renderer-golden` reports `exact=1,468`, `diverges=0`, `gated=0` under
  unchanged row contracts;
- every primary reference is same-tier or has an explicit reason why the
  capability is unavailable;
- the decoded-RGBA byte-exact count is reported as a secondary metric; and
- any rows tightened to `0/0` remain protected by the corpus ratchet.

There is no requirement for all 1,468 final GPU frames to become byte-identical.
