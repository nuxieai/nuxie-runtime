# V6 — WebGPU MSAA exclusion

Status: GREEN on 2026-08-22.

Command: `make renderer-metal-msaa-contract`.

The pinned native Metal replay rejected `--mode msaa` with the authored `native Metal does not implement \`msaa\`` error. The harness therefore cannot substitute or relabel Dawn/WGPU output as native Metal. All 733 WebGPU-style MSAA rows remain explicit exclusions from the native Metal parity denominator.

The final run log SHA-256 is `5034b67bc2f8228e613d955ea78682b69e8889c5c8d8f86638741aa2b0f62710`.
