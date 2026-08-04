# Parity Gap Register

**Purpose.** The single map of everything standing between Nuxie runtime and the
claim "a verifiable, better-performing replacement for the Rive runtime."
Compiled 2026-07-20 from a six-way evidence sweep of both codebases (typeKey
diff, C++ source provenance, embedder API surface, verification blind spots,
renderer/perf status, known-backlog sweep). Upstream reference pin:
`4ac7b327`; renderer pixel-oracle pin: `7c778d13`.

**How to read this.** The golden ratchet (317 exact files / 647 segments /
zero divergences, plus 1,468/1,468 contract-exact renderer pixels) proves
parity **only** over: behavior visible in the render-call stream × files in
the corpus × sampled times × the scripted interaction verbs specified in
`docs/side-channel-format.md`. Every gap below is
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
"looked at it." New rows enter via Upstream Sync cycle triage (upstream drift)
or via a discovered divergence.

---

## V — Verification-integrity gaps (close these first)

The "100% behavioral parity" goal is only as strong as the oracle. These are
holes in the oracle itself, ordered by how much claimed territory they leave
unobserved.

| id | gap | evidence | exit gate |
|---|---|---|---|
| V1 | **The two oracles never compose.** `corpus.toml` proves runtime→draw-calls; `corpus-r.toml` replays pre-serialized `.rive-stream` fixtures through the renderer. Nothing runs `.riv` → Rust runtime → Rust renderer → pixels end-to-end vs C++ pixels. A bug canceling between stages is invisible. | verification audit; `tools/golden-compare`, `tools/renderer-replay` | New `make e2e-golden`: corpus subset rendered end-to-end through both full stacks, pixel-compared under the existing per-row contracts. |
| V2 | **PARTIAL — the animated-corpus sampling hole is closed; the ratchet is withheld.** All 226 entries that combined `LinearAnimation` coverage with a sole `t=0` sample now retain `t=0`, a midpoint, and an authored animation boundary. The added samples exposed V11–V40 rather than being removed or hidden by wider tolerances. Because the corpus is not all green, the `exact-segments` floor remains unchanged. | `corpus.toml`; `tools/golden-compare/src/bin/densify-corpus.rs`; `CORPDEN-report.md` | Resolve V11–V40, restore every parked row to `exact`, then ratchet `exact-segments` at the dense denominator. |
| V3 | **Differential fuzzing was planned (V2 map, "Long-Tail Strategy" §2) but never built.** `fuzz/` targets are panic-only — no C++ comparison, no randomized times/inputs. The long-tail strategy's main discovery engine is missing. | `fuzz/src/lib.rs`; CI `fuzz-smoke` (20s) | Nightly differential job: corpus files × random sample times × random pointer scripts through both runtimes, stream-diffed; failures minimize into new corpus entries. |
| V4 | **PARTIAL — side-channel gate built (#OR-1/#OR-2, 2026-08-02).** Both runners serialize `settled` per advance (the `advanceAndApply` return incl. zero-second forcing), tri-state `HitResult` per pointer verb, reported events with typed custom properties (H4 folded in), and `stateChangedCount`, into the diffed stream behind `--side-channel` (spec: `docs/side-channel-format.md`); `make golden-compare` runs it corpus-wide and ratchets `side-channel-segments` (669). First catch: V11. REMAINDER: per-layer changed-state identity (Rust runtime records only the count; C++ `stateChangedByIndex` has no Rust counterpart), view-model value dumps (no pinned enumeration order exists on both sides), hover cursor. | `docs/side-channel-format.md`; `tools/golden-runner/main.cpp`, `tools/rust-golden-runner`, `tools/golden-compare` | Close the remainder: record changed-state identities in the Rust runtime, pin a VM-value enumeration order, then extend the channel. |
| V5 | **CLOSED 2026-08-03 (#OR-3): scripted external mutation is differentially covered.** Both runners execute typed `setInput` bool/number/trigger mutations, bound-main-view-model boolean/number/trigger mutations through `--view-model-script`, and logical resize + DPR events. Five exact corpus entries cover each family plus cross-stream equal-time ordering. Keyboard/text remain reserved for #FT-TEXT; gamepad and backwards-time scripting remain future grammar additions rather than part of #OR-3's declared close gate. | `docs/side-channel-format.md`; both golden runners; `tools/golden-compare`; `script_verbs_*` corpus entries | **Green:** verb parser/unit tests; five exact scripted corpus entries; corpus-wide `make scripted-golden-compare` at 330/330 exact, 683/683 exact segments, and 682/682 side-channel segments. |
| V6 | **Wide-tolerance escape hatches**: `computed_root_transform` tolerant(0.5), `list_index_script_access` tolerant(0.75) — loose enough to hide real divergence. | `corpus.toml` | Root-cause each; tighten below 0.01 or record a D-row explaining why not. |
| V7 | **Renderer oracle remains one-OS and WebGPU-on-Metal only.** The static pixel matrix covers Apple M5 Max and Apple Paravirtual device at 1,468/1,468 exact on both, so the second-adapter subgate is complete. The required same-runner gate now compares Rust with a separately pinned current-d788 C++ Dawn replay rather than the historical 7c oracle; it is 1,468/1,468 contract-exact locally on M5 (1,370 byte-exact), with the Paravirtual rerun pending. Native Metal/D3D/Vulkan/GL upstream backends remain unverified; 108 rows rest on the 2/32 subpixel contract; the two clockwise-atomic findings (`rust-wgpu-atomic-color-plane-lifetime-parity`, `native-clockwise-atomic-clip-edge-and-composite-parity`) still lack the purpose-built same-backend oracle needed to dismiss or classify them. | `renderer-parity-workflow.md`, `renderer-exactness-map.md`; CI runs `29788092231` and `29806487036`, artifact `8480363545`; local `make renderer-golden-same-runner` 2026-07-20 | (a) **Complete:** second adapter in the blocking static pixel matrix; (b) a purpose-built C++ oracle config for the clockwise-atomic hypotheses, or reclassify them as D-rows with area caps. |
| V8 | **CLOSED 2026-07-24: the unverified browser WebGL2/FemtoVG renderer was retired by user decision.** WebGPU is the sole supported browser backend; missing or unusable WebGPU is an explicit unsupported browser/device state. This is a product support-matrix reduction, not a claim that WebGL2 reached C++ renderer parity. | deleted `crates/nuxie-renderer/src/webgl2.rs`; `tools/browser-renderer-smoke` | Keep the WebGPU Core/Compatibility, lifecycle, stream, GPU-canvas, and unavailable-device gates executable; grep prevents reintroducing the retired implementation/API/dependency surface. |
| V9 | **Rust-only diagnostics never differentially compared**: `--layout-bounds` Taffy report has no C++ counterpart. | rust-golden-runner | Optional: C++ layout-bounds flag; else note as tolerant-mode-only coverage. |
| V10 | **Perf gate is non-blocking and thin**: CI `perf-json` is `continue-on-error`; corpus is 6 hardcoded files; null-renderer only. Renderer timing gate exists but is separate. | `tools/perf-compare`, `.github/workflows/ci.yml` | Make the hot-loop gate blocking at ratio ≤ 1.0 on a broadened (≥20-file) perf corpus; publish the ratio per commit like `exact-segments`. |
| V11 | **`global_variables_test`: Rust misses one root-layer transition at t=0 (found by V4's side channel, 2026-08-02).** C++ `stateChangedCount()`=2 (two layers each entering an AnimationState, coreType 61); Rust `changed_state_count()`=1. Draw streams are identical at t=0 but diverge structurally at t>0 (worst numeric delta 32.0 at samples 0.5/1.0) — a real missed transition hidden until V2 densification. Localized: the second C++ transition is gated on a `TransitionPropertyViewModelComparator`/`TransitionValueNumberComparator` condition whose Rust-side evaluation timing differs mid-frame (settle probes run unconditionally on main; instrumentation shows the Rust layer never transitions in main advance, components, or settlement). The corpus row retains all three samples and is explicitly `diverges`, with named `draw-stream-diverges:V11` and `side-channel-diverges:V11` features. | `corpus.toml` `global_variables_test`; repro: both runners `--samples 0,0.5,1 --side-channel` | Port the pinned mid-frame comparator-evaluation ordering (suspect family: `transition_viewmodel_condition.cpp:49-60` timing vs data-bind passes), then delete both V11 tags and restore the row to `exact`. |
| V12 | **CLOSED 2026-08-03: `db_health_tracker` bounded settlement now matches C++.** Rust had the pinned five-pass ceiling and per-pass resets, but treated pending DataBind bookkeeping as a second continuation condition after Component dirt was clean. The port now breaks solely on clean Component dirt after each reset, matching `state_machine_instance.cpp:2649-2707`. The three-sample debug runner completes, and the full draw plus side-channel comparison is exact. | `corpus.toml` `db_health_tracker`; targeted settlement regression; one-row scripted golden comparison: 3/3 exact segments and 3/3 exact side-channel segments | **Green:** `db_health_tracker` is restored to `exact`; `post-zero-runtime-hang:V12` is removed; the corpus-wide scripted golden gate remains regression-free. |
| V13 | **`animated_clipping`: post-zero draw-command structure diverges.** At the authored 1.0s boundary Rust emits an empty render path where C++ draws the animated compound clip path. | `corpus.toml` milestone V13, samples 0/0.5/1 | Reconcile animated clipping path rebuild/order and restore `exact`. |
| V14 | **`artboard_list_overrides`: post-zero list layout height diverges.** Rust clips at height 724 where C++ clips at 1074. | `corpus.toml` milestone V14, samples 0/0.5/1 | Reconcile list override layout propagation and restore `exact`. |
| V15 | **`bad_skin`: post-zero command/side-channel phase diverges.** At t=2 Rust is still emitting gradient setup where C++ has reached the advance record; the existing 0.0004 tolerance cannot and does not hide the structural mismatch. | `corpus.toml` milestone V15, samples 0/2/4 | Localize the extra/missing draw commands and restore `exact`. |
| V16 | **`bankcard`: post-zero compound path geometry diverges.** The first mismatch is in draw path 58 at the authored 2.0s boundary. | `corpus.toml` milestone V16, samples 0/1/2 | Reconcile the animated path state and restore `exact`. |
| V17 | **`bullet_man`: post-zero stroke geometry diverges.** Rust path 19 endpoints are near 6.45/10.43 while C++ is near 15.29/15.31; the existing 0.0005 contract correctly rejects it. | `corpus.toml` milestone V17, samples 0/0.5/1 | Reconcile the animated stroke/constraint state and restore `exact`. |
| V18 | **`clipping_and_draw_order`: post-zero transform diverges.** Rust emits translation (1121,259) where C++ emits identity. | `corpus.toml` milestone V18, samples 0/0.5/1 | Reconcile post-zero draw-order target transform state and restore `exact`. |
| V19 | **`component_list_child_origin`: post-zero command count/phase diverges.** Rust emits paint creation where C++ has reached the t=0.5 advance record. | `corpus.toml` milestone V19, samples 0/0.5/1 | Reconcile component-list child-origin rebuild output and restore `exact`. |
| V20 | **`component_stateful_vm_instance`: post-zero stateful component size diverges.** Rust draws the first ellipse with radius 30 while C++ uses radius 50. | `corpus.toml` milestone V20, samples 0/0.5/1 | Reconcile VM-instance stateful component binding and restore `exact`. |
| V21 | **`component_stateful_vm_instance_2`: post-zero transform sign diverges.** Rust emits scale -1.5 while C++ emits +1.5. | `corpus.toml` milestone V21, samples 0/0.5/1 | Reconcile the second VM-instance stateful transform and restore `exact`. |
| V22 | **`computed_values_test`: post-zero computed layout bounds diverge.** Rust clips at 245×250 where C++ clips at 490×362.5. | `corpus.toml` milestone V22, samples 0/0.5/1 | Reconcile computed-value propagation before layout and restore `exact`. |
| V23 | **`death_knight`: post-zero command/side-channel phase diverges.** Rust emits gradient setup where C++ has reached the t=0.5 advance record. | `corpus.toml` milestone V23, samples 0/0.5/1 | Localize the extra/missing draw commands and restore `exact`. |
| V24 | **`echo_show_demo`: post-zero Rust draw fails.** The Rust runner exits at the densified samples with `missing render paint for global 584`; C++ completes. | `corpus.toml` milestone V24, samples 0/0.5/1 | Restore retained paint 584 across post-zero advance, compare the stream, and restore `exact`. |
| V25 | **`group_effect`: post-zero compound path geometry diverges.** The first mismatch is draw path 2 at the authored 1.0s boundary. | `corpus.toml` milestone V25, samples 0/0.5/1 | Reconcile group-effect path mutation and restore `exact`. |
| V26 | **`hunter_x_demo`: post-zero command/side-channel phase diverges.** Rust emits gradient 189 where C++ has reached the t=0.5 advance record; the 0.0015 contract correctly rejects the structural mismatch. | `corpus.toml` milestone V26, samples 0/0.5/1 | Localize the extra/missing draw commands and restore `exact`. |
| V27 | **`image_fit_alignment_2`: post-zero image buffer commands diverge.** C++ emits vertex buffer data before t=0.5 where Rust has already reached the advance record. | `corpus.toml` milestone V27, samples 0/0.5/1 | Reconcile animated image-fit geometry/buffer emission and restore `exact`. |
| V28 | **`multi_listeners`: post-zero reported events diverge.** C++ reports `main-event-2` at delay 0.183333337 where Rust emits no corresponding event before the semantic record. | `corpus.toml` milestone V28, samples 0/0.5/1 | Reconcile multi-listener event timing and restore `exact`. |
| V29 | **`new_text`: post-zero Rust draw fails.** The Rust runner exits with `missing render paint for global 71`; C++ completes. | `corpus.toml` milestone V29, samples 0/0.5/1 | Restore retained paint 71 across post-zero advance, compare the stream, and restore `exact`. |
| V30 | **`path_effect_with_feathers`: post-zero effected path geometry diverges.** The first mismatch is draw path 5. | `corpus.toml` milestone V30, samples 0/0.5/1 | Reconcile path-effect/feather advancement and restore `exact`. |
| V31 | **`rewards_demo`: post-zero command/side-channel phase diverges.** Rust emits gradient 51 where C++ has reached the t=0.5 advance record; the 0.0005 contract correctly rejects the structural mismatch. | `corpus.toml` milestone V31, samples 0/0.5/1 | Localize the extra/missing draw commands and restore `exact`. |
| V32 | **`scripted_as_path`: post-zero scripted path diverges.** Rust draws an empty path 7 while C++ draws the authored seven-segment closed path. | `corpus.toml` milestone V32, samples 0/0.5/1 | Reconcile scripted `asPath` retention/advance and restore `exact`. |
| V33 | **`stateful_keyed_trigger`: post-zero keyed color diverges.** Rust remains red (`0xffff0000`) where C++ applies green (`0xff07fb5a`). | `corpus.toml` milestone V33, samples 0/0.5/1 | Reconcile keyed-trigger state application and restore `exact`. |
| V34 | **`stateful_nested`: post-zero nested stateful path geometry diverges.** The first mismatch is draw path 3. | `corpus.toml` milestone V34, samples 0/0.5/1 | Reconcile nested stateful geometry and restore `exact`. |
| V35 | **`stateful_source_switch`: post-zero source-switched size diverges.** Rust draws radius/extent 100 where C++ uses 75. | `corpus.toml` milestone V35, samples 0/0.5/1 | Reconcile stateful source-switch propagation and restore `exact`. |
| V36 | **`superbowl`: post-zero draw-command structure diverges.** Rust emits empty render path 101 where C++ draws compound path 61. | `corpus.toml` milestone V36, samples 0/0.5/1 | Reconcile post-zero path retention/order and restore `exact`. |
| V37 | **`text_vertical_trim_test`: post-zero vertical trim position diverges.** Rust translates to y=182.76001 where C++ uses y=177.935791. | `corpus.toml` milestone V37, samples 0/0.5/1 | Reconcile animated vertical-trim text layout and restore `exact`. |
| V38 | **`viewmodel_instance_to_artboard`: post-zero nested artboard path geometry diverges.** The first mismatch is draw path 3 after VM-instance selection. | `corpus.toml` milestone V38, samples 0/0.5/1 | Reconcile VM-instance-to-artboard state propagation and restore `exact`. |
| V39 | **`virtualize_blendmode`: post-zero command/side-channel phase diverges.** Rust emits paint 17 where C++ has reached the t=2 advance record. | `corpus.toml` milestone V39, samples 0/2/4 | Localize the virtualized blend-mode command mismatch and restore `exact`. |
| V40 | **`zombie_skins`: post-zero command/side-channel phase diverges.** Rust emits gradient 30 where C++ has reached the t=1 advance record. | `corpus.toml` milestone V40, samples 0/1/2 | Localize the skin/gradient command mismatch and restore `exact`. |
| V41 | **`paused_nested_artboard_opacity`: nested opacity differs after enrollment.** Rust emits alpha `0xf7` where C++ emits `0xff` for the same `0x6e0000` color payload. | `corpus.toml` milestone V41, samples 0/0.5/1 | Reconcile paused nested-artboard opacity propagation and enroll the row as `exact`. |
| V42 | **`stateful_component_image_test`: image decode command order differs.** Rust emits `decodeImage` before the paint record that appears first in the C++ stream. | `corpus.toml` milestone V42, samples 0/0.5/1 | Reconcile stateful component image decode/paint ordering and enroll the row as `exact`. |
| V43 | **`data_bind_blob_test`: data-bound blob geometry differs.** At the first differing draw, Rust's rectangle height is 2098.35938 while C++ uses 926.574219. | `corpus.toml` milestone V43, samples 0/0.1/0.5/1/2 | Reconcile blob-bound layout/geometry and enroll the row as `exact`. |
| V44 | **`artboard_opacity_and_transform_test`: the Rust runner lacks nested-child data binding.** It exits on data-bind global 29 (`data-binding-nested-child`, target `Artboard`) before a stream can be compared. | `corpus.toml` milestone V44, samples 0/0.5/1 | Implement the nested-child binding surface, compare the complete stream, and enroll the row as `exact`. |

## F — Feature/subsystem gaps (code that does not exist)

Ranked by upstream line count × product relevance. "Historical backlog"
ceilings from `v2-status.md` are merged in.

| id | subsystem | size (≈lines) | status | notes |
|---|---|---|---|---|
| F1 | **Audio** — `src/audio/**` engine/source/sound/reader, `audio_event.cpp` firing, `Artboard::volume` | 1,030+ | PARTIAL (P2F1/P2F2) | Symphonia WAV/MP3/FLAC source/reader decode, file-owned AudioAsset loading, Factory decode, the Rive-owned headless frame-clock/mixer/sound lifecycle and retained default engine, dense-ordinal AudioEvent playback, multiplied Artboard volume, recursive engine/volume propagation, and Artboard-scoped teardown are ported under D17. Lua audio and CPAL device output remain later packages. |
| F2 | **Text input editing** — cursor motion, selection, keyboard routing (`raw_text_input.cpp` 992, `text_input.cpp` 777, `cursor.cpp` 359, selection/selected-text files) | ~2,400 | CLOSED | FL-E6 ports the retained buffer/journal, cursor and selection paths, key/committed-text routing, multiline source/display behavior, pointer multi-click/drag selection, focus request, and scroll-viewport edge advancement. The remaining non-TextInput gamepad/semantic listener work stays in F5. |
| F3 | **Command queue/server** — threaded host command API (`command_server.cpp` 3,821 + `command_queue.cpp` 2,321) | 6,142 | ABSENT | The model rive-ios/flutter bindings drive. FlowSession is the Nuxie analog but single-threaded; its product C boundary is owned by `nuxie-ios` (see A-tier). Decide: port, or declare FlowSession the supported architecture (D-row + docs). |
| F4 | **Scroll physics** — `elastic_scroll_physics.cpp` (303), `scroll_bar_constraint(.proxy)` (237+), momentum/virtualized scroll | ~700 | PARTIAL | Clamped/core scroll constraint ported at sample-0; interactive momentum, elastic overscroll, scrollbars absent. Paywall-relevant (scrolling lists). |
| F5 | **Keyboard/gamepad/semantic/text-input listener groups + input runtime** (`*_listener_group.cpp` 481, `gamepad_batch.cpp` 363, inputs/) | ~930 | ABSENT | Pointer listeners only. Blocks F2 interaction and any keyboard-driven content. |
| F6 | **Semantics/accessibility** — `semantic_manager` 1,109, `semantic_data` 572, provider, inference registry | 1,926 | CLOSED (FTAIL) | The retained runtime and LT-1 full diff/action/focus side channel are implemented against `4ac7b327`. Nested focus, Simpsons, and data_binding_lists are exact. The latter now shapes its four initial mounted Text bounds through the same retained glyph path used for drawing (`text.cpp:534-615,1154-1233`; `semantic_data.cpp:273-293,501-532`). Component settlement journals generic owner WorldTransform/Path dirt once per semantic synchronization, replacing the former snapshot-only refresh; the dedicated journal test and exact three-sample data_binding_lists projection close the named SEMRES remainders. |
| F7 | **Unported Lua bindings** — `lua_gpu` 3,734, `lua_promise` 1,323, `lua_scripted_context` 583, `lua_buffer_ext` 538, `lua_audio` 507, `lua_data_value` 503, `lua_image_decode` 467, + mesh/color/image/blob/state/data_context/gradient/input | ~9,800 | PARTIAL (by design) | FTAIL partially promotes `lua_scripted_context.cpp`: the full Context method-name surface, headless `features`, and sized/deferred `gpuCanvas` descriptors are present. Canvas 2D has the named `scripted-context-canvas` runtime diagnostic, but its unsupported corpus fixture remains open because no pinned/importable fixture invokes it; component-derived owner-specific `markNeedsUpdate` also remains named residue. The GPU-prefixed `lua_gpu.cpp` candidate still retains its own Canvas 2D and `Image:view` residue. Other named Lua families retain their correspondence status and corpus gates. |
| F8 | **ORE scripted GPU host** (GPUBuffer/GPUCanvas contexts) | — | DEFERRED (`deferred-2026-07-19-ore-gpu`) | GPU-prefixed userdata reaches wgpu under the approved D18 adapter ceiling. Native ORE remains deferred; Canvas 2D/`Image:view` are outside the adapter. |
| F9 | **Joystick runtime behavior** | 169 | PARTIAL (verify) | Only property keys found; confirm advance/apply behavior or add fixture proving it. |
| F10 | **Behavioral-verify candidates** — concrete typeKeys with no bespoke handler: `ClampedScrollPhysics`/`ElasticScrollPhysics` (524/525), `ListPath` (619), `ListenerInputTypeEvent/Text` (659/666), `TransitionValueIdComparator` (601) | — | UNKNOWN | Cheapest wins in the register: author one fixture each; either it's generically handled (close row) or it diffs (new F-row). |
| F11 | **Compressed-texture decoders** (astc/bc/ktx2/etc) | 735 | ABSENT | GPU texture path; relevance depends on whether editor exports these. |
| F12 | **Async work pool** (346) + **profiler** (407) | 753 | PARTIAL | P1-m ports the profiler records, lifecycle, wire format, and runtime hooks; its capture backend is the declared D16 adaptation. The async work pool remains absent and matters only if F3 is ported. |
| F13 | Historical backlog ceilings (recorded in `v2-status.md`): full ListenerGroup drag/opaque behavior, nested pointer/listener hit propagation beyond event bubbling, live data-bound nested-host controls beyond generated defaults, richer static-text modifiers (shape/origin, gradient text effects) | — | LATENT | Currently exact for all corpus files; will surface as diffs when fixtures exist (see C-rows). |
| F14 | `binary_writer`/`binary_data_reader`, `static_scene.cpp`, `hittest_command_path.cpp`, `intrinsically_sizeable.cpp` | ~350 | ABSENT (accepted) | Read-only runtime doesn't need writers; note and close. |

## A — Embedder API surface gaps

The runtime behavior often exists; the surface doesn't. Structural finding:
**capability fragmentation** — events-with-properties, text runs, VM lists,
multi-touch batches live only in FlowSession. The product C boundary exposing
them is owned by `nuxie-ios`; portable `nux_capi.h` remains a minimal surface
in this repository.

| id | gap | tier |
|---|---|---|
| A1 | **No `FileAssetLoader` callback** — no lazy/out-of-band/CDN asset resolution; host must pre-resolve all bytes at import; `cdnUuid`/`cdnBaseUrl` never consulted. | 1 |
| A2 | **Native device-output control remains absent** — the Rust Artboard facade now exposes headless engine and volume control, but CPAL start/stop and the portable C boundary remain later work. | 1 |
| A3 | **Text run set/get not in the portable surface** — runtime primitive exists (`set_root_text_value_run`) but is surfaced only via the `nuxie-ios` FlowSession boundary; reading a run's text is exposed nowhere. Most common SDK write after inputs. | 1 |
| A4 | **Event custom properties missing from the low-level surface** — `StateMachineReportedEvent` carries name/url/target/delay only; properties exist only in FlowSession output. Portable embedders lose them. | 2 |
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

## RB — Structural rebuild follow-ups

These are architecture findings with live mutation-gated Rust mechanisms.
They are implementation tickets, not UNKNOWN audit rows. Closing one requires
porting the pinned C++ ownership/update lifecycle and deleting the named
mechanism under the ordinary, scripted, probe, and applicable pixel floors.

| id | subsystem | named mechanism | status |
|---|---|---|---|
| RB-2 | Focus ownership/projection | `RuntimeFocusTree::sync` descriptor projection plus `target_nodes` rebuild instead of retained live `Focusable`/`FocusData` relationships | OPEN |
| RB-3 | Scripted-object advance | `script_advance_queue` stored elapsed steps during component advance and replayed them later at a factory-bearing facade | CLOSED — queue removed; exact-slot ordering and park-on-error lifecycle are ported |
| RB-4 | Scalar ScriptInput binding | `rehydrate_script_listener_actions` rescans and hydrates scalar inputs at scene rebind instead of retaining the C++ `ScriptInput`/`DataBindContext` push relationship | OPEN |
| RB-5 | SolidColor paint mutation | `solid_color_paint_revisions` defers the C++ `SolidColor::colorValueChanged` retained-paint mutation to a later draw handoff | OPEN |

RB-3's exact-slot schedule and deferred-queue removal landed in `d6d36a32`.
Rust retains one `RuntimeAdvancingComponent` schedule in authored object order,
threads the factory capability into that ordinary walk, and calls each
scripted drawable, layout, or path-effect VM from its exact slot before
artboard data binds. The live pinned-C++ schedule differential
`graph_projects_resetting_and_advancing_component_registrations` and the
queue-sensitive runtime regression
`mixed_scripted_component_advances_run_at_their_retained_cpp_slots` bind the
`Artboard::m_advancingComponents` lifecycle at pinned
`src/artboard.cpp:1463-1480`, `src/scripted/scripted_drawable.cpp:376-399`, and
`src/scripted/scripted_path_effect.cpp:111-133`. This reconciliation closes the
remaining error lifecycle: `ScriptedObject::scriptAdvance` converts a protected
call failure to `false`
(`src/scripted/scripted_object.cpp:178-203`), so the owner cleared before the
call remains parked. Rust additionally surfaces the typed `ScriptError` to its
host, but that additive signal neither rearms the owner nor changes scheduling.
`failed_script_advance_parks_each_owner_while_surfacing_the_error` covers all
three Rust owner families, and
`persistent_advance_failure_runs_once_like_pinned_cpp` proves the live Rust/C++
one-attempt behavior.

## D — Deliberate-divergence register (declare, don't fix)

These are recorded choices. A public "verifiable replacement" claim should
ship this list as documentation; each row needs to stay true under the Upstream
Sync cycle.

1. `f32::total_cmp` sort order vs C++ `operator<` on NaN/±0 (reproducibility over degenerate-input parity).
2. Saturating float→int casts vs C++ UB; PingPong `duration==0` is the one constructible divergence.
3. **Taffy, not Yoga** — edge-case layouts verify `tolerant`; fence: never pin Taffy behavior-by-behavior. This is the FLR-20 **layout-engine** ceiling.
4. luaur-rt pinned =0.1.8 as the scripting engine (mlua fallback untriggered); Luau engine-version skew is a standing WATCH (`deferred-2026-07-19-luau-engine`).
5. Rust image decoders vs platform decoders — JPEG color-profile rows resolvable only by CoreGraphics; dimension+tolerant-pixel verification, never payload hashes.
6. Renderer fuzz-accepted findings R3-FZ-03/04/05 (area-capped, neither rasterization canonical).
7. GPU integer semantics (unsigned-cast fixed-point limits; checked-sub vs deliberate wrap in row-wrap rebuild).
8. Jellyfish dither-accumulation precision gate.
9. `solar-system.riv` malformed-blendMode import rejection (`rejects-malformed`).
10. 108 renderer rows contract-exact under the reviewed 2/32 Metal-vs-WebGPU subpixel budget (not byte-exact).
11. **Bounded host decoded-image policy (2026-07-21).** The high-level `nuxie::File` import path caps the aggregate decoded RGBA bytes retained by one artboard-tree render cache at 64 MiB by default (`FileImportLimits::max_retained_decoded_image_bytes`); pinned C++ has no aggregate ceiling. The low-level compatibility/golden paths and `FileImportLimits::unbounded()` retain every image exactly like C++, so the exact-corpus floor is unaffected. No C ABI change.
12. **[SUPERSEDED same day by #RD-1 — see map Phase RD.] Retained-renderer invalidation epochs (briefly user-approved 2026-07-21, #B-6 Family B).** The pure-Rust renderer retains replay caches (prepared paints/paths, draw command lists, text layout) that C++ has no counterpart for — C++ redraws through live objects each frame. The instance-to-cache version counters (cache/prepared/command/path/layout/text/draw-order/tree-paint epochs) are the invalidation bridge that retained design requires, validated by the 1,468/1,468 pixel gate and both golden gates. Guardrail: any epoch later found compensating for a missed PORT (lost C++ information) rather than bridging to the renderer is a defect and gets fixed individually — the distinction is "our design keeps more than C++" (feature cost, accepted) vs "our port lost what C++ had" (defect, rebuild).
16. **Pure-Rust profiler capture backend (user-approved P1-m decomposition question 4, 2026-08-01).** The pinned 16-line `src/profiler/profiler.cpp` MicroProfile wrapper is replaced by a pluggable Rust `ProfileCapture` trait, with no MicroProfile or C++ FFI dependency. Fence: this is the FLR-21 **profiler-capture-backend** ceiling only. `RiveProfile` transition/listener records, stable string table, lifecycle and delayed-frame behavior, version-2 BinaryWriter bytes, and the original state-machine/listener hook semantics remain faithful obligations.
17. **Symphonia audio decoder/resampler (Levi-approved P2-f decomposition question 5, 2026-08-01).** The pinned miniaudio memory decoder/channel converter/resampler is replaced by pure-Rust Symphonia decode plus the Rive-owned headless engine glue. WAV, MP3, and FLAC are wired; Vorbis remains recognized-but-unwired like the pinned build. Fence: this is the **decoded-PCM/resampled-frame** ceiling only. PCM bytes and individual decoded samples are never byte-pinned; offline differentials compare metadata plus energy/envelope presence, and resampled frame counts permit an absolute difference of at most two frames. Absolute-frame scheduling/clipping, engine clock, lifecycle/completion, per-artboard stop, levels, and sound volume remain exact obligations. CPAL/device output is not part of this adaptation or this package.
18. **wgpu Lua GPU execution contract (Levi-approved GPUCEIL D-row, 2026-08-03).** The pinned ORE-backed objects in `src/lua/renderer/lua_gpu.cpp` are represented by Rust userdata, immutable backend-neutral submission snapshots, and retained wgpu resources. Equality obligations cover authored pipeline/resource selection, draw order and dynamic state, attachment load/store/resolve/depth behavior, texture upload history, stable resource identity across submissions, and resulting pixels under the existing renderer contract. Nuxie requires explicit `GPURenderPass:finish()` at script return rather than reproducing ORE's auto-finish/orphan-error lifecycle, and its factory retains at most 16 external GPU texture identities. Fence: this is the **lua-gpu-wgpu-adapter** ceiling only; it does not permit dropping GPU-prefixed names or substituting CPU rendering, and it does not include this mixed file's Canvas 2D or `Image:view` residue.

## H — Drift & housekeeping

| id | item |
|---|---|
| H1 | Cycle-3 approval granted 2026-07-21 for the fixed `d788e8ec..b73bc675` cut: PORT TextInput (`1b4df2ad`) and static-link (`b73bc675`), profiler (`079305d7`) deferred WATCH, both dependency WATCH rows retained. Later `ba2b6434` drift belongs to the next inventory. |
| H2 | Size budget user-approved 2026-07-21: 9 MiB (9,437,184 B) blocking for BOTH variants. History: the initial 8 MiB choice predated `974aab66`; re-measurement with the 43-root harness (7.84 MiB OFF / 8.70 MiB ON) reopened the gate the same day, and the user approved the recommended replacement (~3.4% headroom over ON). Any future breach reopens the gate with fresh measurements rather than raising the constant. |
| H3 | Two `TODO(golden)` markers: `state_machine.rs:797` (port `addToHitLookup`), `draw.rs:3555` (unify layout-bounds path). |
| H4 | CLOSED 2026-08-02: folded into V4's event side-channel — both runners now diff every reported event's type/name/delay/url/target and typed custom properties corpus-wide (`docs/side-channel-format.md`). |

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
   upstream file (Upstream Sync cycle inventory) with no manifest row fails
   triage. This converts the one-off provenance sweep into a standing invariant.
3. **Oracle-first rule.** No F-row implementation starts without a fixture
   or channel that would *fail* today (the existing golden culture, applied
   to gaps).
4. **The Upstream Sync cycle feeds the register.** Triage rows marked
   WATCH/deferred land here automatically with staleness counters (already the
   convention).
5. **Parity scorecard.** Publish the claim in tiers, each backed by a gate:
   *frame parity* (golden ratchet + V1 e2e), *interaction parity* (V4/V5
   channels), *SDK parity* (A-tier table), *platform parity* (V7/V8 matrix),
   *performance* (V10 blocking ratio ≤ 1.0). "Verifiable replacement" =
   every tier green or its exceptions listed in D.
