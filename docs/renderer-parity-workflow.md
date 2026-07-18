# Renderer 1.0x Parity Workflow

This is the operating contract for making the pure-Rust renderer at least as
fast as the C++ Dawn renderer without changing its rendered result. It
supersedes R4's historical 2.0x directional threshold. The old R4 reports
remain useful history, but `1.0x` is now the acceptance threshold.

## Definition of parity

The primary metric is submit-to-GPU-complete frame time from
`rive-renderer-perf-v3` at 1024x1024 on the same Metal adapter. A parity result
must satisfy all of the following:

1. collect exactly five independent fixed-matrix reports, each with seven
   counterbalanced samples per runner and immutable provenance;
2. within each report, split samples by launch order, take the per-scene median
   for each runner, sum those medians, form candidate/C++ for each order, and
   equal-weight the two order ratios;
3. the median of the five report ratios is at most `1.0` overall, for
   clockwise atomic, and for MSAA;
4. renderer goldens have zero newly divergent or gated rows;
5. deterministic candidate-excess work remains zero;
6. renderer, shader-reproducibility, backend, browser, and workspace tests
   pass or have an explicitly recorded external-fixture blocker.

`renderer-perf-parity-gate` is the sole timing verdict. It rejects fewer or
more than five reports, duplicate/aliased/semantically copied reports, changed
scene order, non-Metal or differing adapters, non-release runners, changed
structural metrics, relaxed sample counts, and mixed manifest, runner,
generator, or source identities. The older C++-minimum-selected paired ratio
is retained in every report as a deliberately non-gating diagnostic; selecting
the minimum control biases that view and makes it too noisy for a 1.0x claim.
CPU encode time alone, a profiler interval, or a report from another load
window is also diagnostic evidence, not the parity verdict.

## Frozen comparison

The current research baseline is:

- nuxie runtime: `eb0e2527` before the 1.0x worktree changes;
- C++ runtime: `7c778d13` in `/Users/levi/dev/oss/rive-runtime`;
- C++ backend: Dawn WebGPU on Metal, not native Rive Metal;
- wgpu base packages: crates.io `30.0.0`; the current candidate applies the
  provenance-bound core/HAL patches described below;
- adapter: Apple M5 Max;
- C++ runner SHA-256:
  `c0be5dea661f44751490e397759bd0b6fe1f8a7526dccf9c8677b6077fed5487`;
- final Rust runner SHA-256:
  `d876c3c4b35ec8d116af3e1f059f51830058b9baabe0090c9e5476212055d4b9`;
- report-generator SHA-256:
  `6715a33396819b9d82e591f884b39c213a539e64056df052c81b5bfc9677930b`;
- five-report gate SHA-256:
  `49113ffa5db866bf9eac04fb927eb22c86b98530400ceb68644cc69b8bd5e0d1`;
- candidate source identity:
  `eb0e2527dacd68cf55fc181d124cf619f7d11615+renderer-dirty-sha256-45957a307a7c93058bfbbb3dac41f1b7fc409e902d1f75e627f78c02aa4364ae`.

Any decision report must record source revisions, runner hashes, adapter
identity, mode, dimensions, warmups, measured frames, and completion scope.
Never overwrite an accepted runner while collecting its comparison.

The renderer-source suffix above is reconstructable. Starting at the recorded
base, SHA-256 the byte stream formed by the base line, `git diff --binary HEAD
-- Cargo.toml Cargo.lock crates`, and sorted `path:file-SHA-256` lines for every
untracked, non-ignored file below `crates/` and `vendor/`. This intentionally
identifies the renderer/runtime source scope rather than every modified
workspace tool. The candidate executable hash binds all code actually linked
into the runner, while the manifest and generator have their own hashes in
every report. This recipe covered 390 untracked crate/vendor files.

```sh
renderer_base=$(git rev-parse HEAD)
renderer_dirty_sha=$({
  printf 'base:%s\n' "$renderer_base"
  git diff --binary --no-ext-diff HEAD -- Cargo.toml Cargo.lock crates
  git ls-files --others --exclude-standard -z -- crates vendor |
    while IFS= read -r -d '' renderer_file; do
      renderer_file_sha=$(shasum -a 256 "$renderer_file" | awk '{print $1}')
      printf 'untracked:%s:%s\n' "$renderer_file" "$renderer_file_sha"
    done
} | shasum -a 256 | awk '{print $1}')
candidate_source_id="${renderer_base}+renderer-dirty-sha256-${renderer_dirty_sha}"
```

## Final 1.0x result

The first strict five-report attempt missed only MSAA: its medians were
`0.997900` overall, `0.998105` clockwise atomic, and `1.005095` MSAA. That
failure is preserved as `parity-gate.{json,md}` beside the final evidence.
Source comparison then found a concrete lifetime mismatch: C++ retains and
resets its midpoint-fan preparation allocators with `RenderContext`, while
Rust recreated the contour and prepared-stroke vectors for every stroke and
dropped them with the frame. Rust now checks out one bounded preparation
scratch set from a context-owned pool, returns it even when a frame is
abandoned, allows uncached overflow for concurrent frames, and discards a
pathological retained set above 1 MiB.

The next five fresh reports from that final source passed on 2026-07-18:

| scope | median Rust/C++ | passing reports | interpretation |
| --- | ---: | ---: | --- |
| overall | 0.966058 | 5/5 | Rust about 3.4% faster |
| clockwise atomic | 0.941193 | 5/5 | Rust about 5.9% faster |
| MSAA | 0.996544 | 3/5 | parity; Rust about 0.35% faster |

The immutable, source-controlled gate report is
`docs/evidence/renderer-parity-2026-07-18/attempt2-parity-gate.json`
(SHA-256
`e64600c80fab893d38de55aaec0b6441b6e511c21d8607ae8ca475fe5fdde294`).
All five raw report hashes are embedded in that report. The non-gating
minimum-selected medians were 1.127866 overall, 1.126457 clockwise atomic,
and 1.129898 MSAA; they are recorded to expose the selection bias, not to
override the equal-order verdict.

The corrected physical-work oracle also passed all 16 variants with zero Rust
excesses. Clockwise-atomic uses five actual Rust buffer copies where C++ uses
six or seven; MSAA uses one where C++ uses six. Rust uploads fewer bytes in
every row. The final counter runner SHA-256 is
`96eb5c8b3d797caaa79f7c34356f6d5649776bde22ddbb2c28ebf38e22853470`;
the pinned C++ counter runner SHA-256 is
`f291ebded45728b39b47ed0f7585663e2116229dcccc61b176e99b4fc824c385`;
the counter generator SHA-256 is
`cda2b9ad477c241fe3e128db2959e13fac8043a5deb5b3bd35da1326d9e5e22e`;
the schema-v2 report is
`docs/evidence/renderer-parity-2026-07-18/renderer-work-counters.json`
(SHA-256
`92b73d58f6ee5e58baaf4d30a34f4478152a57818bee6b7db0962e19ef25a6bc`).

Final behavior verification completed the serialized pixel corpus at
`exact=1468`, `byte-exact=757`, `diverges=0`, `gated=0`, `total=1468`.
Shader reproducibility, native backend invariants, the external consumer,
browser WebGPU/fallback smoke tests, renderer all-features tests, and the full
workspace suite also pass on this source identity.

## Investigation loop

Work one falsifiable hypothesis at a time.

1. Reproduce with the fixed matrix and preserve the JSON/Markdown report.
2. Compare the corresponding C++ preparation, scheduling, resource lifetime,
   shader specialization, and backend submission code.
3. Prove or reject excess work with deterministic counters before timing.
4. Make the smallest behavior-preserving change that closes the identified
   difference.
5. Run focused structural and byte-exact tests.
6. For an individual timing-defined optimization, build immutable A and B
   runners and use an A-B-B-A bracket before retaining it. For the final parity
   claim, collect the five fresh reports required by the gate above.
7. Run the full pixel and workspace gates before accepting the slice.
8. Record accepted and rejected hypotheses so they are not rediscovered.

Use profiles to choose the next hypothesis, not to declare a win. Time Profiler
separates CPU preparation from wgpu/backend work. Metal System Trace must count
physical command buffers, render/blit encoders, GPU span, and GPU busy time.

## Two controls for atomic mode

The production Rust result is the release gate. A second forced-generic Rust
control is needed when attributing generic atomic changes: the C++ Dawn runner
does not advertise `supportsClockwiseAtomicMode`, so its nominal
`clockwise-atomic` lane executes generic atomics with a clockwise override.
Rust can opportunistically select its specialized clockwise implementation.
Keep those facts separate:

- production Rust versus C++ answers whether the shipped renderer is faster;
- forced-generic Rust versus C++ answers whether the same architecture is
  implemented with comparable cost.

Do not slow the production path merely to make the control architectures look
alike.

The final evidence does not contain a global forced-generic Rust toggle, so it
makes only the first claim: shipped specialized production Rust versus the
fixed C++ Dawn runner. It does not claim that Rust's generic-atomic architecture
has the same cost as C++'s generic lane. Adding that control remains useful
stretch attribution, not a condition on the production 1.0x result.

## Current attribution

The old Rust trace contained 46 physical Metal command buffers for 24 real
encoders. That observation remains useful history, but it is no longer the
current architecture. The accepted vendored wgpu core/HAL fast path now keeps
a render-pass body on the open Metal command buffer only when exhaustive core
classification proves there is no ordered pre-pass work. It separately drops
a transition-only native buffer only when buffer and texture initialization
encoded no clear. Any pass that needs query resets, indirect validation,
render-bundle work, store-discard repair, memory-initialization clears, or a
strict-event prologue retains its safe stock boundary; all-empty submission
semantics are unchanged. All non-Metal backends default to stock behavior.

A current steady-state Metal trace has the same physical topology on both
sides:

| backend | physical Metal command buffers | encoders |
| --- | ---: | --- |
| C++ Dawn | 1 | 3 (blit, tessellation, solid) |
| Rust wgpu | 1 | 3 (blit, tessellation, solid) |

The patched package provenance is:

- `wgpu-core` canonical source patch SHA-256:
  `9751a43416597ec05ba9608f924cd4ada7eeb123643f0b45eec671c3c0245411`;
- `wgpu-hal` canonical source patch SHA-256:
  `9e55f5a57cbe17cfe0d61d22ab5c691e88e2dfba510496bd4a039fbc85893e69`;
- six-manifest distribution wiring SHA-256:
  `693f49693094a63d258bf151bb462f1345a37bd1720e828c427c79edc874791a`;
- direct-crate `wgpu`, core, and HAL lock SHA-256 values:
  `3f2d79fa13fcedee842d5ca987245d8e01025469bf119c193197b6236c8ccd48`,
  `f57c034f1479e0fcc1257c094521091d3ebb99775a988902f8cf42dae083b7e0`,
  and `bc27e50dd420d2dd78fdce4000b28fb8492fb07cda4da37c4fd488f0829a4476`.

The embedded hashes in the two vendor-local `NUXIE_PATCH.md` files predate
the final formatting-only source normalization. Those metadata bytes are
preserved because they are part of the frozen candidate source identity; the
recomputed post-format canonical source-patch hashes above are authoritative.

The seven-package vendored family is path-wired transitively, not merely
selected by a workspace-root `[patch]`. `make renderer-wgpu-consumer-check`
proves that an external git/path consumer resolves and compiles this measured
stack with no duplicate registry copy. `nuxie-renderer` and its umbrella crate
remain `publish = false`: publishing this exact architecture to crates.io is
intentionally blocked until the changes are upstream or the complete
seven-crate wgpu family is published under coordinated renamed packages.

The focused CPU profile after command-buffer coalescing attributes the largest
remaining native backend category to render-pass begin/encoder setup. A
descriptor-caching probe did not show enough benefit to retain. This is a
cost-per-pass lead, not evidence of a remaining physical submission split,
and it is profiling guidance rather than the parity verdict.

The C++ WebGPU "mapped" buffer path is not GPU zero-copy: it writes a retained
CPU shadow allocation and calls Dawn `Queue::WriteBuffer` on unmap. Rust's
retained upload pages plus `StagingBelt` therefore have the same architectural
shape. A direct staging writer could still remove a CPU copy in a future
profile-led slice, but there is no missing C++ zero-copy mechanism to port.

A whole-process malloc probe is not used as steady-state evidence here. It
counted factory construction and showed Rust performing substantially more
allocations, but Rust eagerly constructs its pipeline family while C++ fills a
lazy pipeline cache. Factory creation is outside the timed frame interval, so
that probe identifies startup work rather than a demonstrated render-loop
gap. The frame-lifetime mismatch it helped expose was independently tied to
the C++ source and fixed with the retained stroke-preparation pool above.

## Accepted source-parity slices

The current worktree ports or tightens these C++ mechanisms:

- path snapshots use `Arc` copy-on-write instead of cloning every path;
- plain stroke tessellation is prepared once and shared across scheduled uses;
- generic paint and auxiliary data use the retained frame upload arena;
- failed logical-batch admission moves the draw instead of cloning it;
- terminal transient MSAA attachments discard after resolve;
- generic atomic initialization disables clipping for unclipped flushes;
- fixed atomic pipelines specialize path clipping and clip rectangles
  independently;
- bind-group visibility matches the exact C++ shader stages;
- specialized clockwise path patches and interiors share compatible passes;
- generic disjoint grouping gives plain fills exactly C++'s one board outset;
- MSAA scheduling uses one flat schedule, one retained intersection-board
  walk, an explicit authored-order tie break, and an in-place move
  permutation instead of nested group vectors and draw clones;
- solid-only frames keep the gradient definition and per-draw tables empty;
- clockwise-atomic paints, paint auxiliaries, and triangle vertices share one
  retained grouped upload, while read-write coverage remains a separate
  retained buffer;
- clockwise-atomic coverage uses C++'s nonzero generation prefix, rare
  full-clear-on-growth/wrap behavior, and three completion-guarded slots. Each
  slot also owns its clip texture, preventing concurrent frame reuse;
- fill midpoint preparation reads contour endpoints directly instead of
  allocating a projected cubic-point vector;
- direct MSAA stroke batching derives compatible runs on demand and uses the
  terminal segment as its sentinel, removing the full-frame stroke-end and
  continuation side arrays;
- immutable MSAA writers fill final GPU records directly instead of building
  mutating intermediate vectors, with byte-for-byte reference tests;
- the exact dense multi-contour stroke layout takes a construction-order fast
  path that eliminates the generic contour remap walk and sort. `OverStroke`
  moves from 520 contour visits and 12 sorts per frame to zero of each;
- each of the three upload-ring slots retains one six-vector MSAA packing
  scratch set (spans, contours, local contour IDs, paths, paints, and paint
  auxiliaries). It is cleared for each logical flush, retains normal capacity,
  and releases excess capacity above 1 MiB per slot, for an aggregate bound of
  roughly 3 MiB;
- context-owned stroke preparation scratch mirrors C++'s resettable
  `RenderContext` preparation allocator. A one-slot lease pool survives frame
  boundaries and abandoned frames, concurrent frames use uncached overflow,
  and retained bytes above 1 MiB are released;
- dropping a frame before staging writes are finished replaces only that
  slot's active belt, bounding recovery after repeated errors. A completed
  submission retains its belt; tests cover abandon/recover and submit-then-
  abandon lifecycles;
- work counters record every actual aligned staging copy rather than one
  aggregate populated destination page, keeping structural evidence honest;
- wgpu render-pass attachment-overlap validation stays inline for ordinary
  attachment counts and promotes only when the inline capacity is exceeded.

The three clockwise-atomic slots can retain up to roughly three times one
frame's peak coverage-buffer and clip-texture footprint. That bounded memory
tradeoff is intentional: a slot is held until GPU completion so no in-flight
frame can alias the retained resources.

These slices are accepted together by the final counter, pixel, test, and
five-report timing evidence above; no single bullet is treated as the cause of
the aggregate win in isolation.

Measured rejections are also part of the workflow:

- discarding production HAL labels regressed the representative workload;
- disabling wgpu indirect-call validation had an inconsistent sub-1% effect;
- switching bevel output to MSAA changes pixels;
- naively merging generic path and interior draws risks a real storage hazard;
- the original HAL-only lazy-empty-buffer prototype was too broad and lacked
  the required core lifecycle proof; the accepted vendored path above is the
  narrower, explicitly classified replacement.

## Priority queue

1. Keep the provenance-bound five-report gate and physical-copy counter oracle
   green as the renderer evolves; do not replace them with one lucky run.
2. Add a forced-generic Rust control if generic-atomic cost attribution becomes
   a goal. It must remain separate from the shipped specialized result.
3. Profile before pursuing stretch work. The most source-grounded remaining
   candidate is to write prepared stroke plans directly into flush-wide
   storage, as C++ does, instead of retaining a per-stroke prepared copy.
   Metal memoryless transient MSAA attachments and a direct staging writer are
   secondary profile-led candidates; none is an architectural parity blocker.
4. Preserve the current modest beyond-parity margin instead of trading it for
   behavior, tolerance, or unbounded retained memory.

## Commands

Build the same-capability runner once, copy the candidate and report generator
to content-addressed read-only paths, and compute the candidate source identity
with the frozen recipe above. Capture exactly five reports with the legacy
threshold relaxed; those reports are inputs, not individual verdicts:

```sh
make renderer-perf-runners
cargo build --release -p perf-compare \
  --bin renderer-perf --bin renderer-perf-parity-gate

for run_number in 1 2 3 4 5; do
  /absolute/path/to/pinned-renderer-perf \
    --manifest /absolute/path/to/tools/perf-compare/renderer-scenes.toml \
    --baseline-runner /absolute/path/to/pinned-cpp-runner \
    --candidate-runner /absolute/path/to/pinned-rust-runner \
    --baseline-source-id 7c778d13c5d903b3b74eec1dd6bb68a811dea5f2 \
    --candidate-source-id "$candidate_source_id" \
    --max-ratio 1000 \
    --json "/absolute/path/to/run${run_number}.json" \
    --markdown "/absolute/path/to/run${run_number}.md"
done
```

Do not discard or replace a noisy capture. Run the source-controlled verdict
once over those five reports:

```sh
RENDERER_PERF_PARITY_REPORT_1=/absolute/path/to/run1.json \
RENDERER_PERF_PARITY_REPORT_2=/absolute/path/to/run2.json \
RENDERER_PERF_PARITY_REPORT_3=/absolute/path/to/run3.json \
RENDERER_PERF_PARITY_REPORT_4=/absolute/path/to/run4.json \
RENDERER_PERF_PARITY_REPORT_5=/absolute/path/to/run5.json \
RENDERER_PERF_PARITY_MAX_RATIO=1.0 \
make renderer-perf-parity-gate
```

Regenerate the deterministic physical-work oracle:

```sh
make perf-counter-compare
```

The older immutable A-B-B-A gate remains useful for accepting or rejecting one
optimization slice. It requires separate reconstructable
`R4_TIMING_GATE_A_SOURCE_ID` and `R4_TIMING_GATE_B_SOURCE_ID` values and stamps
the matching identity on each leg. It is not the final renderer-versus-C++
parity estimator.

Final behavior verification:

```sh
make renderer-shaders-check
make renderer-wgpu-backend-check renderer-wgpu-consumer-check
make browser-renderer-smoke
make renderer-golden
cargo test -p perf-compare
cargo test -p nuxie-renderer --lib
cargo test -p nuxie-renderer --all-features --lib
cargo test --workspace
```

Run the normal and scripted V2 golden targets serially if they are included;
both prepare the same debug runner path.
