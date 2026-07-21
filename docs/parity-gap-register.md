# Parity Gap Register

**Purpose.** The single map of everything standing between Nuxie runtime and the
claim "a verifiable, better-performing replacement for the Rive runtime."
Compiled 2026-07-20 from a six-way evidence sweep of both codebases (typeKey
diff, C++ source provenance, embedder API surface, verification blind spots,
renderer/perf status, known-backlog sweep). Upstream reference pin:
`d788e8ec`; renderer pixel-oracle pin: `7c778d13`.

**How to read this.** The golden ratchet (317 exact files / 647 segments /
zero divergences, plus 1,468/1,468 contract-exact renderer pixels) proves
parity **only** over: behavior visible in the render-call stream × files in
the corpus × sampled times × pointer-only input scripts. Every gap below is
one of five kinds, and the kinds are not interchangeable:

- **V — Verification integrity**: parity is *claimed* but the oracle cannot
  see the channel. Fixing these makes the existing claim trustworthy.
- **F — Feature/subsystem**: upstream code with no Rust counterpart.
- **A — API surface**: runtime behavior exists but embedders can't reach it.
- **C — Coverage**: supported behavior no corpus file exercises.
- **D — Deliberate divergence**: recorded non-parity by choice; must be
  *declared*, not fixed, for an honest replacement claim.

Register discipline (same culture as `corpus.toml`): every row keeps a
status, and a row only closes with a **named, mechanical exit gate** — never
"looked at it." New rows enter via Phase S triage (upstream drift) or via a
discovered divergence.

---

## V — Verification-integrity gaps (close these first)

The "100% behavioral parity" goal is only as strong as the oracle. These are
holes in the oracle itself, ordered by how much claimed territory they leave
unobserved.

| id | gap | evidence | exit gate |
|---|---|---|---|
| V1 | **The two oracles never compose.** `corpus.toml` proves runtime→draw-calls; `corpus-r.toml` replays pre-serialized `.rive-stream` fixtures through the renderer. Nothing runs `.riv` → Rust runtime → Rust renderer → pixels end-to-end vs C++ pixels. A bug canceling between stages is invisible. | verification audit; `tools/golden-compare`, `tools/renderer-replay` | New `make e2e-golden`: corpus subset rendered end-to-end through both full stacks, pixel-compared under the existing per-row contracts. |
| V2 | **237 of 317 corpus entries sample `t=0` only.** Any post-first-frame divergence (transitions, loop wrap, settle, decay) on those files is unobserved. | `corpus.toml` `samples` fields | Densify: every animated entry gets ≥3 samples incl. a loop boundary; ratchet `exact-segments` at the new denominator. |
| V3 | **Differential fuzzing was planned (V2 map, "Long-Tail Strategy" §2) but never built.** `fuzz/` targets are panic-only — no C++ comparison, no randomized times/inputs. The long-tail strategy's main discovery engine is missing. | `fuzz/src/lib.rs`; CI `fuzz-smoke` (20s) | Nightly differential job: corpus files × random sample times × random pointer scripts through both runtimes, stream-diffed; failures minimize into new corpus entries. |
| V4 | **Reported events are never compared.** Neither runner calls `reportedEventCount()/reportedEventAt()`; event files verify by draw side effects only. Same for hit-test results, `advanceAndApply`'s settled/needs-redraw bool, hover cursor, SM current-state, and view-model values. | `tools/golden-runner/main.cpp`, `tools/rust-golden-runner` | Extend both runners to serialize an **event/state side-channel** (events with properties, HitResult per pointer op, settled bool per advance) into the diffed stream. |
| V5 | **Input scripts are pointer-only.** No direct SM input set (bool/number/trigger), no view-model mutation (`--view-model-script` is explicitly rejected in the C++ runner), no resize/DPR, no keyboard/text/gamepad, no backwards time. M5's "externally-driven view-model mutation scripts" milestone claim is not actually covered. | script grammar in both runners | Implement `--view-model-script` + `setInput` verbs in both runners; add resize verb; corpus entries that use them. |
| V6 | **Wide-tolerance escape hatches**: `computed_root_transform` tolerant(0.5), `list_index_script_access` tolerant(0.75) — loose enough to hide real divergence. | `corpus.toml` | Root-cause each; tighten below 0.01 or record a D-row explaining why not. |
| V7 | **Renderer oracle is Dawn-WebGPU-on-one-Apple-adapter only.** Native Metal/D3D/Vulkan/GL upstream backends never pixel-verified against; single OS/GPU; 108 rows rest on the 2/32 subpixel contract; the two clockwise-atomic findings (`rust-wgpu-atomic-color-plane-lifetime-parity`, `native-clockwise-atomic-clip-edge-and-composite-parity`) have **no same-backend oracle** and remain live defect hypotheses. | `renderer-parity-workflow.md`, `renderer-exactness-map.md` | (a) One additional adapter/OS in the pixel matrix; (b) a purpose-built C++ oracle config for the clockwise-atomic hypotheses, or reclassify them as D-rows with area caps. |
| V8 | **Browser WebGL2 path (femtovg) is a separate approximating renderer with no exactness harness** — smoke tests only. | `crates/nuxie-renderer/src/webgl2.rs` | Decide: pixel-corpus it (with its own contracts), or declare it a documented degraded mode (D-row). |
| V9 | **Rust-only diagnostics never differentially compared**: `--layout-bounds` Taffy report has no C++ counterpart. | rust-golden-runner | Optional: C++ layout-bounds flag; else note as tolerant-mode-only coverage. |
| V10 | **Perf gate is non-blocking and thin**: CI `perf-json` is `continue-on-error`; corpus is 6 hardcoded files; null-renderer only. Renderer timing gate exists but is separate. | `tools/perf-compare`, `.github/workflows/ci.yml` | Make the hot-loop gate blocking at ratio ≤ 1.0 on a broadened (≥20-file) perf corpus; publish the ratio per commit like `exact-segments`. |

## F — Feature/subsystem gaps (code that does not exist)

Ranked by upstream line count × product relevance. "Historical backlog"
ceilings from `v2-status.md` are merged in.

| id | subsystem | size (≈lines) | status | notes |
|---|---|---|---|---|
| F1 | **Audio** — `src/audio/**` engine/source/sound/reader, `audio_event.cpp` firing, `Artboard::volume` | 1,030+ | ABSENT | Only schema/object-model stubs. Any `.riv` with sound is silent. Planned engine: cpal/rodio/kira (paper decision only). |
| F2 | **Text input editing** — cursor motion, selection, keyboard routing (`raw_text_input.cpp` 992, `text_input.cpp` 777, `cursor.cpp` 359, selection/selected-text files) | ~2,400 | PARTIAL | Rendering/layout of TextInput **is** ported; interaction is not. Upstream is actively improving TextInput (commit `1b4df2ad`, past our pin) — this gap is *widening*. |
| F3 | **Command queue/server** — threaded host command API (`command_server.cpp` 3,821 + `command_queue.cpp` 2,321) | 6,142 | ABSENT | The model rive-ios/flutter bindings drive. FlowSession is the Nuxie analog but single-threaded and Apple-ABI-only (see A-tier). Decide: port, or declare FlowSession the supported architecture (D-row + docs). |
| F4 | **Scroll physics** — `elastic_scroll_physics.cpp` (303), `scroll_bar_constraint(.proxy)` (237+), momentum/virtualized scroll | ~700 | PARTIAL | Clamped/core scroll constraint ported at sample-0; interactive momentum, elastic overscroll, scrollbars absent. Paywall-relevant (scrolling lists). |
| F5 | **Keyboard/gamepad/semantic/text-input listener groups + input runtime** (`*_listener_group.cpp` 481, `gamepad_batch.cpp` 363, inputs/) | ~930 | ABSENT | Pointer listeners only. Blocks F2 interaction and any keyboard-driven content. |
| F6 | **Semantics/accessibility** — `semantic_manager` 1,109, `semantic_data` 572, provider, inference registry | 1,926 | ABSENT | Screen-reader tree, semantic actions/focus. Product/App-Store relevance for embedded flows is real even if corpus never touches it. |
| F7 | **Unported Lua bindings** — `lua_gpu` 3,734, `lua_promise` 1,323, `lua_scripted_context` 583, `lua_buffer_ext` 538, `lua_audio` 507, `lua_data_value` 503, `lua_image_decode` 467, + mesh/color/image/blob/state/data_context/gradient/input | ~9,800 | PARTIAL (by design) | Corpus-gated per V2 rules. Any scripted file touching these fails today; ensure the import diagnostic names the missing binding (spot-check). |
| F8 | **ORE scripted GPU host** (GPUBuffer/GPUCanvas contexts) | — | DEFERRED (`deferred-2026-07-19-ore-gpu`) | Keep deferred unless product adopts ORE scripting; exit criteria already recorded in triage. |
| F9 | **Joystick runtime behavior** | 169 | PARTIAL (verify) | Only property keys found; confirm advance/apply behavior or add fixture proving it. |
| F10 | **Behavioral-verify candidates** — concrete typeKeys with no bespoke handler: `ClampedScrollPhysics`/`ElasticScrollPhysics` (524/525), `ListPath` (619), `ListenerInputTypeEvent/Text` (659/666), `TransitionValueIdComparator` (601) | — | UNKNOWN | Cheapest wins in the register: author one fixture each; either it's generically handled (close row) or it diffs (new F-row). |
| F11 | **Compressed-texture decoders** (astc/bc/ktx2/etc) | 735 | ABSENT | GPU texture path; relevance depends on whether editor exports these. |
| F12 | **Async work pool** (346) + **profiler** (407) | 753 | ABSENT | Work pool matters only if F3 is ported; profiler is dev-tooling. |
| F13 | Historical backlog ceilings (recorded in `v2-status.md`): full ListenerGroup drag/opaque behavior, nested pointer/listener hit propagation beyond event bubbling, live data-bound nested-host controls beyond generated defaults, richer static-text modifiers (shape/origin, gradient text effects) | — | LATENT | Currently exact for all corpus files; will surface as diffs when fixtures exist (see C-rows). |
| F14 | `binary_writer`/`binary_data_reader`, `static_scene.cpp`, `hittest_command_path.cpp`, `intrinsically_sizeable.cpp` | ~350 | ABSENT (accepted) | Read-only runtime doesn't need writers; note and close. |

## A — Embedder API surface gaps

The runtime behavior often exists; the surface doesn't. Structural finding:
**capability fragmentation** — events-with-properties, text runs, VM lists,
multi-touch batches live only in FlowSession, whose C ABI ships only in the
Apple/Metal `nux_runtime.h`; portable `nux_capi.h` is a minimal surface.

| id | gap | tier |
|---|---|---|
| A1 | **No `FileAssetLoader` callback** — no lazy/out-of-band/CDN asset resolution; host must pre-resolve all bytes at import; `cdnUuid`/`cdnBaseUrl` never consulted. | 1 |
| A2 | **Audio control absent everywhere** (volume, engine start/stop) — pairs with F1. | 1 |
| A3 | **Text run set/get not in the portable surface** — runtime primitive exists (`set_root_text_value_run`) but is surfaced only via Apple FlowSession; reading a run's text exposed nowhere. Most common SDK write after inputs. | 1 |
| A4 | **Event custom properties missing from the low-level surface** — `StateMachineReportedEvent` carries name/url/target/delay only; properties exist only in FlowSession output. Non-Apple embedders lose them. | 2 |
| A5 | **`nux-capi` cannot read events at all**; VM coverage is bool/number/string set-only (no color/enum/trigger/image/artboard/list, no getters/observers); no `pointer_exit`; no input reads. | 2 |
| A6 | **No thread-safe command-server model** — FlowSession is explicitly single-threaded. Either port F3 or document the threading contract embedders must own (decision row). | 2 |
| A7 | **Artboard resize/layout override not first-class** (`width(x)`, `layoutWidth/Height`, `updateLayoutBounds`, `resetArtboardSize`) — only `raw_mut().set_artboard_dimensions`. Responsive hosts need this. | 2 |
| A8 | Async decode callbacks; RTTI-style typed queries; semantic-tree protocol (pairs with F6). | 3 |

Nuxie-only *additive* surfaces (not gaps, keep them): `scene::Scene` authoring
API, text caret/hit/selection geometry richer than upstream public headers.

## C — Coverage holes (supported but never exercised)

| id | hole | action |
|---|---|---|
| C1 | 28 schema-known typeKeys appear in **zero** corpus files; after removing abstract bases, the live list: `Folder`, `TextVariationModifier`, `TextStyleFeature`, `NSlicerTileMode`, `ScrollBarConstraint`, `TransitionValueIdComparator`, `BlobAsset`, `ScriptedInterpolator`, `TextInputSelectedText`, and the Semantic family (`SemanticData`, `SemanticInput`, `ListenerInputTypeSemantic`). | One fixture per typeKey (author in editor or mine community files). Each either goes exact (close) or spawns an F-row. |
| C2 | Only ~14/317 entries exercise any pointer input; `structural` verification mode used by zero entries (fine — but means it's untested machinery). | Grow input-script corpus alongside V5. |
| C3 | Corpus is upstream test assets + a few demos; no systematic ingestion of *production Nuxie flow files*. | Add a private product-corpus lane: every shipped Nuxie flow runs the golden compare in CI. This is the strongest "verifiable replacement *for our product*" gate available. |

## D — Deliberate-divergence register (declare, don't fix)

These are recorded choices. A public "verifiable replacement" claim should
ship this list as documentation; each row needs to stay true under Phase S.

1. `f32::total_cmp` sort order vs C++ `operator<` on NaN/±0 (reproducibility over degenerate-input parity).
2. Saturating float→int casts vs C++ UB; PingPong `duration==0` is the one constructible divergence.
3. **Taffy, not Yoga** — edge-case layouts verify `tolerant`; fence: never pin Taffy behavior-by-behavior.
4. luaur-rt pinned =0.1.8 as the scripting engine (mlua fallback untriggered); Luau engine-version skew is a standing WATCH (`deferred-2026-07-19-luau-engine`).
5. Rust image decoders vs platform decoders — JPEG color-profile rows resolvable only by CoreGraphics; dimension+tolerant-pixel verification, never payload hashes.
6. Renderer fuzz-accepted findings R3-FZ-03/04/05 (area-capped, neither rasterization canonical).
7. GPU integer semantics (unsigned-cast fixed-point limits; checked-sub vs deliberate wrap in row-wrap rebuild).
8. Jellyfish dither-accumulation precision gate.
9. `solar-system.riv` malformed-blendMode import rejection (`rejects-malformed`).
10. 108 renderer rows contract-exact under the reviewed 2/32 Metal-vs-WebGPU subpixel budget (not byte-exact).

## H — Drift & housekeeping

| id | item |
|---|---|
| H1 | Cycle-3 triage is submitted for the fixed `d788e8ec..b73bc675` cut: TextInput (`1b4df2ad`) and static-link (`b73bc675`) are recommended PORT, profiler (`079305d7`) WATCH; approval is pending. Later `ba2b6434` drift belongs to the next inventory. |
| H2 | Post-Phase-R size is measured reproducibly at 7.19 MiB scripting OFF / 7.95 MiB ON for the renderer link closure. The historical ≤2.75 MiB budget is retired; the replacement budget and whether one or both variants block remain a USER-GATE. |
| H3 | Two `TODO(golden)` markers: `state_machine.rs:797` (port `addToHitLookup`), `draw.rs:3555` (unify layout-bounds path). |
| H4 | `event_report.hpp` equivalence is thin (one trace) — fold into V4's event side-channel work. |

---

## Recommended order

**P0 — make the parity claim trustworthy (V-rows).** V1 (composed e2e
oracle), V3 (differential fuzzing), V4 (event/state side-channel), V2
(sampling density), V5 (input-script verbs). Rationale: every F/C fix
lands on top of this oracle; until then "zero divergences" quietly means
"zero *visible* divergences at t=0 under pointer input."

**P1 — product-blocking features and surfaces.** A1 (asset loader), A3 +
A4/A5 (text runs + events in the portable surface), F2+F5 (text-input
interaction — upstream is moving here, H1), F4 (scroll physics), F1/A2
(audio, if any flow ships sound), C3 (production-flow corpus lane).

**P2 — SDK completeness.** A6 decision (command-server port vs documented
FlowSession threading contract), A7 (resize), remaining capi/VM coverage,
F10/C1 cheap fixture sweeps, V6/V7 renderer-oracle hardening, V10 blocking
perf gate, H2 size re-measure.

**P3 — long tail.** F6 semantics, F7 remaining Lua bindings (corpus-gated,
as designed), F8 ORE, F11 compressed textures, F12, F13 ceilings as fixtures
appear.

## Keeping the register complete (process)

1. **This file is the queue.** One row per gap; a row closes only via its
   named exit gate; closures cite the commit/ratchet number.
2. **Provenance manifest.** Add a machine-checkable `port-manifest.toml`
   mapping every upstream `src/**.cpp` (447 files) to
   `ported|partial|absent|not-applicable` + the Rust module. CI check: a new
   upstream file (Phase S inventory) with no manifest row fails triage. This
   converts the one-off provenance sweep into a standing invariant.
3. **Oracle-first rule.** No F-row implementation starts without a fixture
   or channel that would *fail* today (the existing golden culture, applied
   to gaps).
4. **Phase S feeds the register.** Triage rows marked WATCH/deferred land
   here automatically with staleness counters (already the convention).
5. **Parity scorecard.** Publish the claim in tiers, each backed by a gate:
   *frame parity* (golden ratchet + V1 e2e), *interaction parity* (V4/V5
   channels), *SDK parity* (A-tier table), *platform parity* (V7/V8 matrix),
   *performance* (V10 blocking ratio ≤ 1.0). "Verifiable replacement" =
   every tier green or its exceptions listed in D.
