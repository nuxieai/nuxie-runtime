# V3 — native structure, lifecycle, and failure

Status: GREEN on 2026-08-22.

Command: `make renderer-native-metal-v3` with `MTL_DEBUG_LAYER=1`, `MTL_SHADER_VALIDATION=1`, and `NUXIE_REQUIRE_LIVE_METAL_TESTS=1`.

Results:

- Renderer library: 713 passed, 0 failed, 40 exact external-oracle/diagnostic ignores.
- Native Metal tracer: 27 passed, 0 failed or ignored; Metal API and GPU validation were enabled.
- ORE default: 118 unit tests plus the live binding witness passed.
- ORE tools: 133 unit tests plus the live binding witness passed.
- Aggregate: 993 passed, 0 failed, 40 declared ignores; no required-live lane silently skipped device acquisition.

The final run log SHA-256 is `c5bb4b05912878367ca088b4fc9f1631a087db29f6d4fa538a83052c7e95c0b6`.
