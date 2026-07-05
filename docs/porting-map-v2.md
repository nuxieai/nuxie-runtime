# Rive Rust Porting Map V2

Working directory: `/Users/levi/dev/rive-rust`

Reference runtime: `/Users/levi/dev/oss/rive-runtime`

Supersedes: `docs/porting-map.md` (V1). V1 remains the historical record and its
artifacts remain in force: `rive-schema`, `rive-binary`, `rive-graph`, the
dirt/transform runtime, the keyframe/state-machine semantics already in
`rive-runtime`, and all existing contract tests. V2 changes the goal, the
verification model, and the porting method for everything that remains.

## Goal

A Rust runtime that loads real `.riv` files, instantiates artboards, advances
animations/state machines with pointer input, resolves data binding, and draws
through the existing C++ Rive Renderer via FFI. Everything but the renderer
and the Luau VM is Rust: the subsystems the C++ runtime delegates to vendored
C/C++ libraries use Rust-native equivalents instead (see #V2-7), chosen by the
rule that spec-defined behavior may swap engines while implementation-defined
semantics keep the same engine.

Definition of done for every milestone: **N corpus files produce output
identical to the C++ runtime**, never "behavior X is pinned."

## Ground Rules

These replace the V1 method. They are process, not aspiration; a change that
violates them should be rejected in review.

1. **Port code, not behaviors.** The unit of work is one C++ class/file
   translated coarsely in one sitting, with a comment naming its C++ source
   file. No contract doc, no probe fixture, no audit doc precedes
   implementation.
2. **Verification is a posteriori.** Divergence is found by diffing end-to-end
   render-call streams against C++, then localized. cpp-probe pinning is a
   debugging tool with a budget — if a divergence resists localization for
   about half a day, write one targeted probe — never the development loop.
3. **The existing contract suite is frozen, not extended.** All V1 contract
   tests keep running in CI as the regression floor. New contract docs and new
   probe-first coverage require an active divergence as justification.
4. **The corpus decides priority.** Features are scheduled by corpus
   frequency. Anything not exercised by a corpus file gets a loud
   `unsupported: <feature>` import-time diagnostic and a backlog entry — not a
   speculative implementation.
5. **Every phase ends with a demo.** Something visibly draws or responds; the
   milestone metric is percentage of corpus files frame-exact.

## Progress Metric

`corpus.toml` tracks every fixture with the object type-keys it contains and a
status: `exact | diverges | unsupported-feature | not-yet`. The single project
health number is `exact-segments` from `make golden-compare`: the sum of
verified file x sample segments across `exact` corpus entries. CI publishes it
per commit.

---

## #V2-1: Golden Harness And Renderer Seam (Phase 0)

Blocked by: none
Target: days 1–3

### Deliverables

1. **`RecordingRenderer` (C++).** A `rive::Renderer` implementation in
   `tools/golden-runner` that serializes every call — `save`, `restore`,
   `transform`, `clipPath`, `drawPath`, `drawImage`, `drawImageMesh` — with
   full path verbs/points, paint state (color, gradient stops, thickness,
   cap/join, blend mode, feather), and image references into a canonical
   stream. Headless, deterministic, no GPU.
2. **Golden runner CLI (C++).** Takes `(file, artboard, state machine,
   timeline sample times, scripted input events, scripted view-model
   mutations)` and emits one stream per sample using the full C++ runtime.
3. **`rive-render-api` (Rust).** `Renderer`, `Factory`, `RenderPath`,
   `RenderPaint`, `RenderImage` traits mirroring the C++ abstract interface.
   First implementation: a serializer emitting the identical stream format.
   The existing renderer-independent draw command structures in `rive-runtime`
   are reshaped to drive this trait.
4. **`make golden-compare`.** For each corpus entry × samples × input script:
   run both runtimes, diff streams with float epsilon, report the first
   divergent call with context.
5. **Corpus.** All of `tests/unit_tests/assets` and golden assets from the C++
   repo, plus real/production files, plus complex community files.
   `corpus.toml` manifest generated with `riv-inspect` type-key scans.
6. **`rive-renderer-ffi` (parallel, low risk).** A second `Renderer` impl
   forwarding the trait through a C ABI into the real Rive Renderer, plus a
   minimal demo viewer window or offscreen pixel target. This is the permanent
   production seam; building it now validates the ABI early. M1 landed this as
   a macOS Metal offscreen readback demo before moving on to M2.

### Exit Criteria

Golden harness runs in CI; `corpus.toml` exists with feature tags; at least
one trivial static file is frame-exact end to end.

## #V2-2: Static Vector Rendering Exact (Phase 1)

Blocked by: #V2-1
Target: week 1

### Port

Runtime halves of `src/shapes/` (~5.1k lines) and the drawing parts of
top-level `src/*.cpp`: `Path::buildPath`, live `PathComposer`, parametric path
flattening, `ShapePaint` runtime, `LinearGradient`/`RadialGradient` update,
clipping stack execution, `Artboard::draw`. Fill gaps in `src/math/` (~3.4k:
`RawPath`, `Mat2D`, contour measure). Much of this exists as static projection
in `rive-graph` and payload structs in `rive-runtime`; the work is making it
live code emitting real render calls instead of test facts.

No pinning: parametric-path and paint edge cases are caught by golden frames
on files that contain those shapes.

### Milestone M1

Every static (no-animation) corpus file renders frame-exact at `advance(0)`.
Demo: the viewer draws real files through the Rive Renderer from Rust.

## #V2-3: Animated Playback Exact + Real Object Model (Phase 2)

Blocked by: #V2-2
Target: weeks 1–2

### Port

Remainder of `src/animation/` runtime application (keyframe types and
transition semantics largely exist; the gap is applying to the full property
set through generated setters), `Artboard::advance` ordering,
`AdvancingComponent`/`ResettingComponent` execution, joystick application,
`Scene::advanceAndApply` semantics.

### Structural Refactor (do it now, in this phase)

Generate concrete object structs plus setters/getters from `rive-schema` (the
work deferred since V1 #4) and give `ArtboardInstance` a real cloned object
arena, replacing the `BTreeMap<(component, property_key), value>` overlays.
This closes V1 #9 properly. It happens here because every later phase (nested
artboards, data binding, text) multiplies the cost of delaying it. Split the
monolithic `crates/rive-runtime/src/lib.rs` into modules as part of landing
this. The frozen contract suite plus goldens guard the refactor.

### Milestone M2

Animated corpus files are frame-exact at sampled times (t = 0, 0.25s, 0.5s, …,
plus loop boundaries). Object model landed; `lib.rs` modularized.

## #V2-4: Interactivity Exact (Phase 3)

Blocked by: #V2-3
Target: weeks 2–3

### Port

`src/input/` hit testing (~1.4k), listener dispatch and pointer routing,
`src/constraints/` (~3.3k), `src/bones/` plus skin/IK solver math (graph edges
and tendon/skin facts are already projected — this is the math), reported
events out.

Golden harness gains scripted input: `(pointerDown x y, advance dt,
pointerMove …)` sequences in the corpus manifest, replayed against both
runtimes.

### Milestone M3

Interactive corpus files respond identically to scripted pointer sequences.
Demo: a button file clickable in the Rust viewer.

## #V2-5: Nested Artboards And Lists Exact (Phase 4)

Blocked by: #V2-3
Target: week 3

### Port

`ArtboardHost`/`NestedArtboard` runtime, nested state-machine/animation
advancement and remapping, `ArtboardComponentList` instancing and map-rule
selection (statics already projected), event propagation across hosts. Closes
V1 #13.

### Milestone M4

Composite corpus files exact. Nested artboards appear in essentially all real
production content, which is why this outranks finishing data binding.

## #V2-6: Data Binding The Fast Way (Phase 5)

Blocked by: #V2-3, #V2-5
Target: weeks 3–4

### Method

The V1 audits are an asset; the implementation flips to translation:

- Build `RuntimeDataBindGraph` exactly as scoped in the V1 #12 contract by
  translating `src/data_bind/` plus `src/viewmodel/` (~7k lines)
  class-by-class, not behavior-by-behavior.
- The 478 existing data-binding contract tests are consumed as the regression
  suite proving the translation — no new contracts.
- Converters, including `DataConverterFormula`, are straight translations of
  each converter class; formula edge cases fall out of translating
  `data_converter_formula.cpp` once and letting frozen tests plus golden
  frames judge it.
- Public view-model mutation API (set number/string/color/enum/trigger/list
  from outside) lands here — it is what users of data binding actually call.

### Milestone M5

Data-bound corpus files exact, including externally-driven view-model mutation
scripts in the golden harness.

## #V2-7: Delegated Subsystems (Phase 6)

Blocked by: #V2-2 (layout/text draw through the same seam)
Target: weeks 4–6, corpus-gated order

The C++ runtime delegates these subsystems to vendored C/C++ libraries. The
Rust port uses Rust-native equivalents, selected by one rule: **spec-defined
behavior may swap engines; implementation-defined behavior may not.** Each
subsystem sits behind a trait so an engine can be swapped later without
touching runtime code.

Decided libraries (2026-07-02):

- **Layout: Taffy** (`taffy` crate, DioxusLabs). Same lineage as Yoga — Yoga
  core author Emil Sjölander wrote Stretch, which became Taffy. Implements
  Block/Flexbox/CSS Grid per the CSS specs, while Yoga preserves
  React-Native-era quirks that Rive editor files are authored against, so
  edge-case layouts may diverge: layout-bearing corpus files verify in
  `tolerant` mode. Port `src/layout/` glue (~0.9k) plus `LayoutComponent`
  style plumbing against a layout trait. Fallback if corpus divergence proves
  painful: Yoga via FFI behind the same trait — do not pin Taffy against Yoga
  behavior-by-behavior; that is V1 with someone else's library. Taffy's CSS
  Grid support is a post-M7 Rust-only enhancement opportunity (the editor
  cannot author it).
- **Text shaping: HarfRust** (`harfrust`, the HarfBuzz organization's official
  Rust port) over **`read-fonts`/`skrifa`** (Google fontations) for font
  parsing — the stack HarfRust itself builds on. Closest-to-exact swap in
  this list: it tracks upstream HarfBuzz and passes nearly all of its test
  suite. Known gaps (no Arabic fallback shaper for malformed fonts, a couple
  dozen esoteric shaping features) go to the backlog only if a corpus file
  hits them.
- **Bidi: `unicode-bidi`.** Both it and C++'s SheenBidi implement UAX #9,
  which is spec-defined; a mismatch is a bug in one of them, not a porting
  concern.
- **Line breaking / text layout:** Rive's own code in `src/text/` (~8.5k) —
  ported, not replaced. Largest remaining chunk; run a one-day sizing spike
  before scheduling. Avoid `cosmic-text`: it imposes its own line-layout
  model and Rive has its own.
- **Image decoding:** the `image`-ecosystem crates (`png`, `zune-jpeg`,
  `image-webp`) replace libpng/libjpeg/libwebp. PNG is lossless and stays
  exact; JPEG decoders are not bit-identical across implementations, so
  image-bearing files verify decoded dimensions plus tolerant pixel sampling
  — never payload hashes across runtimes.
- **Audio: `cpal`/`rodio`** (or `kira`) instead of miniaudio, ~1k of glue.
  Only when corpus files contain audio events.
- **Scripting: `mlua` with the `luau` feature.** No mature pure-Rust Luau
  exists; mlua vendors and compiles the official Roblox Luau (C++) sources
  inside cargo and exposes safe bindings. Technically FFI, strategically
  correct: scripting semantics are implementation-defined, and running the
  same VM as the C++ runtime gives parity by construction, so scripted files
  can still verify `exact`. Port the `src/lua/` glue (~16.4k) against mlua's
  API, feature-gated, scheduled only when a corpus file demands it. This must
  not become the next formula swamp.

### Verification Modes

`corpus.toml` gains a per-entry `verification = "..."` mode consumed by
`golden-compare`:

- `exact` — default; byte/epsilon-identical streams. All fully-ported
  subsystems, plus scripting (same VM).
- `tolerant(ε)` — positions/pixels compared within epsilon. Files exercising
  Taffy layout, HarfRust-shaped text, or lossy image decoding.
- `structural` — same call sequence and counts, values within tolerance; last
  resort, requires a Decision-log entry justifying it.

The mode is per-file and as strict as the file's content allows; a file with
no layout/text/images never gets to hide behind `tolerant`.

### Milestone M6

Layout and text corpus files verified in their declared modes (`exact` where
no swapped engine is involved, `tolerant` otherwise); audio/scripting gated
with explicit diagnostics.

## #V2-8: Ship Surface (Phase 7)

Blocked by: #V2-4, #V2-6
Target: ongoing from week 4

- Public Rust API crate (`rive`): `File::import`, `ArtboardInstance`,
  `StateMachineInstance`, inputs, events, view models, `Renderer` trait —
  designed for users, not probes. Closes V1 #15.
- C ABI on top for embedding; the renderer FFI from #V2-1 already proved the
  pattern. Closes V1 #16 with the decision: Rust runtime, C++ Rive Renderer
  behind FFI, renderer-agnostic trait so a Rust renderer can come later.
- Performance pass: arena layout, remove `BTreeMap` from hot paths, benchmark
  `advance` against C++ on corpus files.
- Wire the already-hardened importer into `cargo fuzz`.

### Milestone M7

Public API and C ABI published in-repo; advance/draw performance within target
of C++ on the corpus.

## Phase R: Renderer Port (separate map)

The eventual Rust renderer — a faithful port of the C++ Rive Renderer's
algorithm layer onto `wgpu`, replacing the ORE/impl HAL entirely — is planned
in `docs/renderer-port-map.md`. It is blocked by M7 **and explicit user
activation**; it is not part of V2 scope and must not be started by a `/goal`
session on its own initiative.

---

## Long-Tail Strategy

The long tail is discovered, triaged, and localized — never enumerated up
front.

1. **Golden frames are the oracle.** An unconsidered behavior either shows up
   as a diff on a real file (fix it) or never manifests (ignore it). The C++
   source's behavior surface stops being the work queue.
2. **Differential fuzzing replaces hand-enumeration.** A nightly job renders
   corpus files at randomized times with randomized input sequences and
   randomized view-model mutations through both runtimes and diffs the
   streams. This finds the weird interleavings — loop-edge callbacks,
   transition interruptions, trigger-reset ordering — automatically.
3. **Escalation ladder for a divergence.** Frame diff → first divergent render
   call → binary-search the timeline → disable subtrees/objects to isolate the
   component → read the two implementations side by side. Only if still stuck
   after about half a day: one targeted cpp-probe pin. Expected rate: a few
   pins per week, not per commit.
4. **Unsupported is never silent.** Import-time feature detection reports
   exactly which corpus files touch unimplemented type-keys/behaviors. The
   backlog reads "these 4 files need TextFollowPath," never "C++ has a method
   we haven't audited."
5. **The frozen contract suite guards the floor.** Refactors — especially the
   #V2-3 object model — run against all V1 contracts plus goldens, so the
   rigor already purchased keeps paying without new premiums.

## Milestone Summary

| # | When | Provable claim |
|---|------|----------------|
| M0 | day 3 | Golden diff harness + corpus manifest + one exact file |
| M1 | week 1 | Static vector files pixel-exact via real Rive Renderer |
| M2 | week 2 | Animated playback exact at sampled times; real object model landed |
| M3 | weeks 2–3 | Pointer-interactive files exact under scripted input |
| M4 | week 3 | Nested artboards/lists exact |
| M5 | week 4 | Data binding exact incl. external mutation |
| M6 | weeks 5–6 | Layout + text verified per declared modes; audio/scripting gated |
| M7 | week 6 | Public API + C ABI; perf within target of C++ |

Timeline calibration: the V1 process shipped binary import, the static graph,
and most animation semantics in five days — cadence is not the constraint.
Redirected at coarse translation with end-to-end verification, the remaining
~50k productive lines over ~6 weeks is aggressive but consistent with
demonstrated velocity. Risk concentrates in text and the #V2-3 refactor, which
is why text gets a sizing spike and the refactor happens early.
