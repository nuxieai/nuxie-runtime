# Runtime frame-loop structural trace

This tooling produces deterministic evidence for
`docs/runtime-frame-loop-ownership.toml`. It is diagnostic-only: the ordinary
golden runners and runtime have no counter overhead.

## Isolated instrumented runners

Build both runners with:

```sh
make runtime-frame-loop-trace-runners
```

The target:

- refuses any C++ checkout except
  `d788e8ec6e8b598526607d6a1e8818e8b637b60c`;
- builds the C++ runtime into the dedicated
  `out/rive-frame-loop-coverage-debug` directory;
- records the LLVM flags in
  `librive.a.frame-loop-trace-provenance` next to that archive;
- builds a separately named `rive_golden_runner_coverage`;
- builds Rust under `target/frame-loop-coverage` with
  `-Cinstrument-coverage` and the `coverage-trace` feature;
- verifies that the resulting binaries contain their profile/counter symbols.

It never replaces the ordinary debug archive or ordinary golden-runner binary.

## Capture contract

Use the six `PERF_CORPUS_IDS` entries and their checked-in `corpus.toml`
samples. Every runner invocation must use `--benchmark-repeat 1`.

For frame-only coverage, set:

```sh
LLVM_PROFILE_FILE=/absolute/output.profraw
RIVE_GOLDEN_COVERAGE_FRAME_ONLY=1
RIVE_GOLDEN_COVERAGE_FLUSH=1
```

`RIVE_GOLDEN_COVERAGE_FLUSH` is required only for the unscripted C++ runner,
which deliberately exits without running destructors after its output is
flushed.

For frame-loop allocation evidence, additionally set:

```sh
RIVE_GOLDEN_ALLOCATION_COUNTER=1
```

The runners reset both kinds of evidence after construction and immediately
before the sample loop. They fail instead of reporting a plausible zero when
the requested instrumentation was not compiled in. Frame-only coverage and
allocation counting also reject repeated benchmark mode because that mode runs
two separately constructed timing passes.

Merge the `.profraw` files with `llvm-profdata merge`, export both binaries
with `llvm-cov export`, retain the uninstrumented full-run exports for
construction landmarks, and feed those artifacts plus the paired recording
streams and per-entry allocation JSON to `summarize_trace.py`. The generated
JSON is accepted only when `make runtime-frame-loop-port-check` verifies the
pin, six-entry corpus, static scope, per-file dynamic markers, renderer-feed
work equality, and gap coverage for every mismatched landmark.
