# Pinned upstream microbenchmark inventory

The repository tracks the 20 benchmarks registered by pinned Rive runtime
commit `5551696a826ff546775fffe46ca78824b1f1dc51`, the runtime's pin. Their
names, upstream source hashes, fixture conversions, and provenance live in
`microbenchmarks.toml`. Upstream's per-frame artboard benchmarks
(`tests/bench/artboard_frame_bench.cpp`, added before this pin) are not part
of the mirror.

This is an inventory and provenance gate, not an executable Rust/C++ ratio
harness. The former Criterion target depended on the packed runtime facade and
was removed with that implementation. It is intentionally not restored or
replaced with a synthetic benchmark.

## Active commands

```sh
make microbench-gate RIVE_RUNTIME_DIR=/path/to/rive-runtime
make microbench-extract RIVE_RUNTIME_DIR=/path/to/rive-runtime
```

`microbench-gate` runs the microbenchmark tool tests, validates the exact
20-case inventory and converted fixture hashes, and verifies the pinned
upstream registrations, source hashes, `RenderContextNULL` capability source,
and generated fixture provenance.

`microbench-extract` deterministically refreshes the converted datasets from
the pinned upstream checkout. Review any resulting bytes and manifest hashes;
it is not a benchmark run.

The tool retains `run-cpp` for focused inspection of an already-built pinned
C++ benchmark executable. The retired Rust `run` and `compare` commands are
not exposed because their Criterion target no longer exists.

## Performance coverage that remains

Runtime performance remains covered by the source-shaped hot-loop and corpus
performance gates in `tools/perf-gate/`, `tools/perf-compare/`, and the
corresponding Makefile targets. Renderer performance and renderer
microbenchmark infrastructure remain owned by their renderer-specific
harnesses. This cleanup does not change those implementations, their
manifests, or their CI gates.

Historical reports under `docs/evidence/` describe measurements made while
the Criterion mirror existed. They remain valid historical evidence, but their
old `microbench-run` and `microbench-compare` commands are no longer current
instructions.
