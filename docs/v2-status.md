# V2 Status

Working state for `/goal` sessions. Keep this file small and current; it is
the only memory the next session has. Update it every commit.

## Metric

- Corpus files `exact`: n/a (golden harness not yet built)
- Current milestone: **M0 — Golden Harness And Renderer Seam (#V2-1)**

## Milestones

- [ ] M0: Golden diff harness + corpus manifest + one exact file
- [ ] M1: Static vector corpus files exact at advance(0); FFI viewer demo
- [ ] M2: Animated playback exact at sampled times; real object model landed; lib.rs modularized
- [ ] M3: Interactive files exact under scripted pointer input
- [ ] M4: Nested artboards/lists exact
- [ ] M5: Data binding exact incl. external view-model mutation
- [ ] M6: Layout + text exact; audio/scripting gated with diagnostics
- [ ] M7: Public `rive` API + C ABI; perf within target of C++

## Next

1. `crates/rive-render-api`: `Renderer`/`Factory`/`RenderPath`/`RenderPaint`/`RenderImage` traits + matching Rust serializer.
2. `make golden-compare` + `corpus.toml` seeded from `tests/unit_tests/assets` with type-key tags from `riv-inspect`.
3. First exact file: a trivial static rectangle fixture, end to end.

## Backlog (unsupported features awaiting corpus demand)

- Golden runner view-model mutation scripts; `--view-model-script` is reserved
  but rejected until M5 external data-binding corpus files require it.

## Decisions

- 2026-07-02: V2 map adopted (`docs/porting-map-v2.md`); V1 map superseded, its contract suite frozen as regression floor.
- 2026-07-02: Golden runner records decoded image payloads by size/hash for the first renderer slice; real decoded dimensions are deferred until `rive_decoders` is wired into the CLI harness build.
- 2026-07-02: Golden runner emits one accumulated stream per run with
  `source`, `input`, `sample`, and `frame` markers; `golden-compare` will split
  sample segments from that stream.

## Log

- 2026-07-02: V2 plan, `/goal` command, and this status file created. No V2 code yet.
- 2026-07-02: [M0] Added `tools/golden-runner` RecordingRenderer/Factory scaffold, smoke binary, and `make golden-runner`; `make golden-compare` still not present.
- 2026-07-02: [M0] Golden runner CLI now imports real `.riv` files, selects
  artboards/state machines, advances sampled timelines, replays pointer input
  scripts, and emits recording streams; `make golden-compare` still not
  present.
