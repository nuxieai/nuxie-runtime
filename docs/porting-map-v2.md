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
through the existing C++ Rive Renderer via FFI. Every subsystem the C++
runtime delegates to vendored C/C++ libraries uses a Rust-native equivalent
instead (see #V2-7), chosen by the rule that spec-defined behavior may swap
engines while implementation-defined semantics keep the same engine — where a
faithful, upstream-conformance-verified port (HarfRust, luaur) counts as the
same engine.

**Pure-Rust end state (policy, 2026-07-07):** the shipped runtime contains no
C/C++ under the covers. FFI appears in exactly two sanctioned forms: (1) the
feature-gated renderer bridge, whose removal is the whole point of Phase R
(`docs/renderer-port-map.md`), and (2) untriggered fallbacks (Yoga behind the
layout trait, mlua behind the scripting seam) — documented contingencies with
zero code, zero Cargo.lock presence, and zero build cost, whose activation
requires a logged Decision and creates an obligation to return to the
Rust-native path. The C++ golden runner is test scaffolding, not product, and
stays C++ deliberately: it is the oracle the Rust runtime is verified
against.

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
- **Scripting: `luaur` (`luaur-rt`).** Decided 2026-07-07 (supersedes the
  earlier mlua choice): luaur is a line-for-line translation of the actual
  Luau compiler/VM/type checker from C++17 to Rust
  (https://github.com/pjankiewicz/luaur) — 5,347 ported unit tests pass and
  all 293 upstream conformance scripts run byte-identically against the C++
  VM, with bytecode compatibility. Under the engine-swap rule this counts as
  the same engine (a verified faithful port, the HarfRust pattern), so
  scripted files still target strict `exact`; the golden harness is the
  third oracle — the C++ probe runs real Luau, so any luaur semantic drift
  surfaces as a stream diff, attributable and reportable upstream. Cautions:
  the project is young and research-grade — pin the version, and pin it
  against the Luau version the reference runtime vendors. `luaur-rt`
  deliberately mirrors mlua's API, so `mlua`+`luau` remains the untriggered
  fallback behind the same scripting seam (the Taffy/Yoga-FFI pattern). Port
  the `src/lua/` glue (~16.4k) against that seam, feature-gated,
  corpus-file-by-corpus-file. This must not become the next formula swamp.

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

## #V2-9: Closeout Hardening (M8)

Blocked by: M7 (complete)
Added 2026-07-09 by user decision: close out the runtime port fully before
Phase R — scripting supported, the C ABI complete, and confidence in the port
raised from "corpus-verified" to "hardened."

### Scope

1. **Scripting integration.** Land the seam from the scripting lane report
   (traits `ScriptingVm`/`ScriptInstance`/`ScriptHost` defined in
   `rive-runtime`, implemented by `rive-scripting`, wired in `crates/rive`
   behind a feature). Sandbox parity per the lane findings (globals before
   `luau_load`, `lua_l_sandbox` order), native-vector `Vector` binding, then
   port bindings corpus-file-by-corpus-file (~half the 18.2k C++ glue per the
   census, in the lane report's order). HARNESS PREREQUISITE: the C++ golden
   runner must be built WITH scripting (reference build supports it) so
   scripted files get real reference streams instead of the current
   FileAssetContents-stripped ones; re-generate those corpus entries' status
   once both sides run scripts. Acceptance: the gated/harness scripting files
   become exact (or carry a sharper named diagnostic).
2. **C ABI completion** (from the capi lane follow-ups): pointer events
   (`pointer_down/move/up`), view-model/data-bind context access, an additive
   cache-holding draw that reuses render handles across frames now that draw
   retention landed, and one recorded decision aligning default-state-machine
   selection between capi and the golden runner.
3. **Hardening (the Bun-migration lessons, 2026-07-09):**
   - Semantic-trap audit: fix the findings from the cross-language sweep
     (panic-reachable `unwrap`/`expect`/indexing on weird-but-accepted
     inputs, float `partial_cmp().unwrap()`, `as`-cast truncation, overflow
     semantics) — release is `panic=abort`, so every reachable panic is a
     process kill in an embedder.
   - Adversarial-review findings on the perf-era retention/epoch code and
     the `unsafe` inventory: fix or explicitly accept each with rationale.
   - Fuzzing wired into CI: extend beyond import to advance/draw/pointer
     paths (mutated-but-importable files, long random input scripts).
   - `PORTING.md`: distill the C++→Rust idiom codex (rcp→arena, virtuals→
     enum dispatch, pointer graphs→dense slots, dirt bits, callback
     canonicalization) from the status archives into one document — prep
     artifact for Phase R and any new contributor.
4. **Residuals:** drain or explicitly re-gate the remaining `harness` bucket;
   the two view-model-convention residuals stay gated on component-list
   instancing per the recorded decision.

5. **Release prep — Nuxie runtime (added 2026-07-09).** The port ships as
   open source under the product name Nuxie (nuxie.io — the user's own
   product; the runtime powers Nuxie paywalls/flows embedded in customers'
   apps, with `.riv` compatibility as a verified feature and a future `.nux`
   superset format):
   - Rename sweep BEFORE first publish: workspace crates `rive-*` →
     `nuxie-*`/`nux-*`, the C ABI prefix `rive_*` → `nux_*` (unshipped, so
     free now; breaking later), feature names, header, smoke test. Internal
     doc/history references to Rive stay as factual references.
   - README positioning: "An independent, pure-Rust runtime for Nuxie flows,
     compatible with the Rive (.riv) file format. Not affiliated with or
     endorsed by Rive Inc." Factual compat claims only; no Rive logo/brand.
   - License hygiene: preserve upstream MIT copyright notices (derivative
     work), same for luaur/Lua/Roblox notices. Corpus fixtures are Rive's
     test assets — do NOT vendor in the public repo; fetch from
     rive-app/rive-runtime at CI time (the C++ reference runtime remains a
     dev-time test dependency only, never shipped).
   - SDK footprint: record a binary-size budget for the `nux-capi` cdylib
     (mobile SDK context), measure current size, and apply the cheap wins
     (identical-code-folding/linker flags, `opt-level` check, symbol
     stripping in the release artifact). Size becomes a tracked number like
     the perf ratio, and a Phase R renderer-selection criterion.

### Milestone M8 (exit)

Scripting corpus files verified against a scripting-enabled C++ runner; C ABI
covers the embed loop including pointer events and retained-draw; both audits
resolved; fuzzers in CI with zero known reachable panics from accepted files;
`PORTING.md` committed; the rename/licensing/size release-prep items done so
the repo is publishable as Nuxie runtime as-is. Then Phase R is a clean
start.

## Phase S: Upstream Sync (separate map)

After M8, Nuxie runtime tracks `rive-app/rive-runtime` through the recurring
workflow in `docs/upstream-sync-map.md` (`/sync-upstream` command): triage
new upstream commits with impact/risk/effort ratings, a HARD user-approval
gate on every port decision, then port-and-advance-the-pin with the golden
ratchet as proof of completeness. Nightly automation runs triage-only after
two clean manual cycles. Added 2026-07-09 by user decision.

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
