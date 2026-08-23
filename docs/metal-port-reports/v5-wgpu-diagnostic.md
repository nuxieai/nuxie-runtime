# V5 — Rust-WGPU diagnostic differential

Status: GREEN as a diagnostic-completeness gate on 2026-08-22.

Command: `make renderer-metal-wgpu-diagnostic`.

Pinned C++ Metal remains the sole correctness oracle. This lane cannot make a source-exact Metal result fail, make a source-divergent Metal result pass, or establish a Metal tolerance.

Results:

- 736/736 rows executed and 736/736 adapter identities matched.
- 678 rows met the exact diagnostic comparison and 60 were byte-identical.
- 0 crashes, timeouts, malformed outputs, process failures, or public success/error mismatches.
- 58 WGPU-only pixel divergences were retained with visual diffs under `target/renderer-metal-wgpu-parity/results/`.
- Because V4 is green on the same Rust Metal output, the 58 differences are WGPU diagnostic debt, not Metal-port failures.

The final run log SHA-256 is `c7b0929fd3514402f79fa0ba5ffd449a81fa2d0a409b335635c61efc9aa294a9`.
