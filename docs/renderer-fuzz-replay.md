# Renderer Fuzz-Replay Gate

This document is the operating contract for the R3 dual-renderer negative-input
gate. The deterministic harness is `tools/renderer-fuzz-replay`; run it with:

```sh
make renderer-fuzz-replay
```

The Make target builds one feature-complete `renderer-replay` binary and sends
the same typed `rive-golden-stream-v1` stream through Rust/wgpu and the pinned
C++/Metal FFI. Each case runs in its own child process with a 20-second wall
deadline. A pass requires both children to exit successfully, emit a valid
64x64 PNG, and preserve an opaque finite control draw made after the hostile
commands. Each backend first renders a control-only baseline; every pixel in
the full 16x16 control footprint, including antialiased edges and surrounding
clear pixels, must remain byte-exact in that backend's hostile replay. A panic,
abort, timeout, device/map error, invalid PNG, invalid atomic submission, or
damaged control footprint fails the gate. Expected output files are removed
before each child starts, so stale PNGs cannot satisfy the oracle.

The gate uses clockwise-atomic mode because R3-ST-05 specifically asks whether
hostile numeric state can poison a later atomic batch. Non-control pixels are
compared separately, outside the full control-draw footprint.

## Cases And Findings

| ID | Input | Pixel contract | 2026-07-13 arm64 macOS 26.4.1 result |
| --- | --- | --- | --- |
| R3-FZ-01 | NaN, positive/negative infinity, and `f32::MAX` transforms under save/restore | Exact outside control | Exact |
| R3-FZ-02 | Move-only, coincident-line, and zero-area closed paths | Exact outside control | Exact |
| R3-FZ-03 | Zero, negative/positive `f32::MAX`, NaN, and infinity stroke widths | Named delta capped at 1,024 pixels/max delta 255; zero resolves it | 826 pixels, max delta 255 |
| R3-FZ-04 | 64 nested saved clip paths followed by full restoration | At most 32 pixels at max delta 1 | 21 pixels, max delta 1 |
| R3-FZ-05 | Empty, duplicate, reversed, out-of-range, NaN, and infinite gradient stops/radius | Named delta capped at 384 pixels/max delta 255; zero resolves it | 300 pixels, max delta 255 |

R3-FZ-03 and R3-FZ-05 deliberately name C++/Rust behavior outside the valid
numeric contract rather than declaring either rasterization canonical. Their
pixel-area caps prevent an accepted finding from expanding silently. They do
not permit a Rust crash, hang, device loss, bad output, or any change inside
the same-backend control footprint. If the implementations converge, the
harness reports the finding as resolved.

The first R3-FZ-03 run found a Rust debug-overflow panic while calculating
stroke tessellation segments for `f32::MAX`. The fix clamps in floating-point
space before converting to `u32`; a focused renderer unit test pins that path.

A fresh bounded campaign on 2026-07-14 replayed both control baselines and all
five hostile streams in 53.79 seconds. All 12 child processes completed within
their 20-second deadlines and reproduced the table above exactly. A
supplemental MSAA attempt stops on the control-only C++/Metal assertion that
native Metal does not implement MSAA flush; it is not evidence against this
clockwise-atomic gate and cannot serve as a dual-renderer MSAA oracle.

## CI And Provenance

The `renderer-golden` macOS job checks out C++ runtime revision
`7c778d13c5d903b3b74eec1dd6bb68a811dea5f2`, builds the debug runtime and Metal
renderer archives, and runs this gate before the normal pixel corpus. The
subprocess boundary is part of the oracle: in-process panic catching cannot
reliably bound GPU waits or native aborts.
