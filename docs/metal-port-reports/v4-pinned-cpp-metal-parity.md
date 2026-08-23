# V4 — pinned C++ Metal parity

Status: GREEN on 2026-08-22.

Authority: freshly rebuilt C++ native Metal replay from `4ac7b32798da0482e441ef09304dc3b480ed3ee5`, run on the same Apple M5 Max as the Rust candidate.

Command: `make renderer-metal-cpp-parity`.

Results:

- 736/736 native-Metal-compatible rows accepted.
- 698 rows byte-identical.
- 38 rows satisfied unchanged, predeclared source-manifest comparison budgets.
- 0 divergences, 0 gated rows, and 736/736 adapter provenance matches.
- The downscaled transparent image regression that exposed the mipmap dispatch defect is now byte-exact with zero differing pixels and maximum channel delta 0.
- WGPU was not run or consulted for acceptance.

The final run log SHA-256 is `e450b11854eb06bb9fadb2ab9d19c7186a2d1578d6957f1b13e30f9098ffef86`.
