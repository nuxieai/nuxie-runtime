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
| V1 | **The two oracles never compose.** `corpus.toml` proves runtime→draw-calls; `corpus-r.toml` replays pre-serialized `.rive-stream` fixtures through the renderer. Nothing runs `.riv` → Rust runtime → Rust renderer → pixels end-to-end vs C++ pixels. A bug canceling between stages is invisible. | verification audit; `tools/golden-compare`, `tools/renderer-replay`; composed-session evidence: `docs/e2e-composed-evidence.md` | New `make e2e-golden`: corpus subset rendered end-to-end through both full stacks, pixel-compared under the existing per-row contracts. |
| V2 | **PARTIAL — the animated-corpus sampling hole is closed; the ratchet is withheld.** All 226 entries that combined `LinearAnimation` coverage with a sole `t=0` sample now retain `t=0`, a midpoint, and an authored animation boundary. The added samples exposed V11–V40 rather than being removed or hidden by wider tolerances. Because the corpus is not all green, the `exact-segments` floor remains unchanged. | `corpus.toml`; `tools/golden-compare/src/bin/densify-corpus.rs` | Resolve V11–V40, restore every parked row to `exact`, then ratchet `exact-segments` at the dense denominator. |
| V3 | **Differential fuzzing was planned (V2 map, "Long-Tail Strategy" §2) but never built.** `fuzz/` targets are panic-only — no C++ comparison, no randomized times/inputs. The long-tail strategy's main discovery engine is missing. | `fuzz/src/lib.rs`; CI `fuzz-smoke` (20s) | Nightly differential job: corpus files × random sample times × random pointer scripts through both runtimes, stream-diffed; failures minimize into new corpus entries. |
| V4 | **PARTIAL — side-channel gate built (#OR-1/#OR-2, 2026-08-02).** Both runners serialize `settled` per advance (the `advanceAndApply` return incl. zero-second forcing), tri-state `HitResult` per pointer verb, reported events with typed custom properties (H4 folded in), and `stateChangedCount`, into the diffed stream behind `--side-channel` (spec: `docs/side-channel-format.md`); `make golden-compare` runs it corpus-wide and ratchets `side-channel-segments` (669). First catch: V11. REMAINDER: per-layer changed-state identity (Rust runtime records only the count; C++ `stateChangedByIndex` has no Rust counterpart), view-model value dumps (no pinned enumeration order exists on both sides), hover cursor. | `docs/side-channel-format.md`; `tools/golden-runner/main.cpp`, `tools/rust-golden-runner`, `tools/golden-compare` | Close the remainder: record changed-state identities in the Rust runtime, pin a VM-value enumeration order, then extend the channel. |
| V5 | **CLOSED 2026-08-03 (#OR-3): scripted external mutation is differentially covered.** Both runners execute typed `setInput` bool/number/trigger mutations, bound-main-view-model boolean/number/trigger mutations through `--view-model-script`, and logical resize + DPR events. Five exact corpus entries cover each family plus cross-stream equal-time ordering. Keyboard/text remain reserved for #FT-TEXT; gamepad and backwards-time scripting remain future grammar additions rather than part of #OR-3's declared close gate. | `docs/side-channel-format.md`; both golden runners; `tools/golden-compare`; `script_verbs_*` corpus entries | **Green:** verb parser/unit tests; five exact scripted corpus entries; corpus-wide `make scripted-golden-compare` at 330/330 exact, 683/683 exact segments, and 682/682 side-channel segments. |
| V6 | **Wide-tolerance escape hatches**: `computed_root_transform` tolerant(0.5), `list_index_script_access` tolerant(0.75) — loose enough to hide real divergence. | `corpus.toml` | Root-cause each; tighten below 0.01 or record a D-row explaining why not. |
| V7 | **Renderer oracle remains one-OS and WebGPU-on-Metal only.** The static pixel matrix covers Apple M5 Max and Apple Paravirtual device at 1,468/1,468 exact on both, so the second-adapter subgate is complete. The required same-runner gate now compares Rust with a separately pinned current-d788 C++ Dawn replay rather than the historical 7c oracle; it is 1,468/1,468 contract-exact locally on M5 (1,370 byte-exact), with the Paravirtual rerun pending. Native Metal/D3D/Vulkan/GL upstream backends remain unverified; 108 rows rest on the 2/32 subpixel contract; the two clockwise-atomic findings (`rust-wgpu-atomic-color-plane-lifetime-parity`, `native-clockwise-atomic-clip-edge-and-composite-parity`) still lack the purpose-built same-backend oracle needed to dismiss or classify them. | `renderer-parity-workflow.md`, `renderer-exactness-map.md`; CI runs `29788092231` and `29806487036`, artifact `8480363545`; local `make renderer-golden-same-runner` 2026-07-20 | (a) **Complete:** second adapter in the blocking static pixel matrix; (b) a purpose-built C++ oracle config for the clockwise-atomic hypotheses, or reclassify them as D-rows with area caps. |
| V8 | **CLOSED 2026-07-24: the unverified browser WebGL2/FemtoVG renderer was retired by user decision.** WebGPU is the sole supported browser backend; missing or unusable WebGPU is an explicit unsupported browser/device state. This is a product support-matrix reduction, not a claim that WebGL2 reached C++ renderer parity. | deleted `crates/nuxie-renderer/src/webgl2.rs`; `tools/browser-renderer-smoke` | Keep the WebGPU Core/Compatibility, lifecycle, stream, GPU-canvas, and unavailable-device gates executable; grep prevents reintroducing the retired implementation/API/dependency surface. |
| V9 | **Rust-only diagnostics never differentially compared**: `--layout-bounds` Taffy report has no C++ counterpart. | rust-golden-runner | Optional: C++ layout-bounds flag; else note as tolerant-mode-only coverage. |
| V10 | **BLOCKING RATCHET INSTALLED 2026-08-04; direct ratio parity remains open.** `make perf-gate` measures 24 manifest-owned files with scripting-enabled release C++/Rust runners, 100 sequential frames at 60 Hz, and the median of five `advance + draw` sessions. CI and the serial timing phase in `tools/land.sh` block when any file exceeds its checked-in `ceil(worst current ratio × 1.15)` ceiling, and every landing prints the complete ratio table. | `perf-corpus.toml`; `tools/perf-gate/perf_gate.py`; `tools/perf-compare`; `.github/workflows/ci.yml`; `tools/land.sh`; baseline and variance evidence in `docs/perf-size-evidence.md` | Use `make perf-gate-tighten` after measured improvements; it takes the per-file maximum of three fresh sessions and can only lower baselines/ceilings. Keep ratcheting every per-file ceiling toward **Rust/C++ ≤ 1.0**; V10 remains open until all files meet that parity target. |
| V11 | **CLOSED 2026-08-04 — comparator timing and mounted-layout draw are exact.** A same-layer consumed global now advances the shared mounted-frame fence without scheduling an independent child layout solve. `NestedArtboardLayout` draw/hit/bounds read the mounted Artboard's retained root position, and a transferred child reuses its parent-owned layout snapshot; `global_variables_test` matches C++ at t=0/0.5/1 including the side channel. | exact scripted golden capture; `global_variables_mid_frame_view_model_comparator_matches_cpp_layer_count`; `nested_layout_constraint_space_refreshes_for_parent_or_child_layout_generation`; `nested_artboard_layout.cpp:24-78`; `artboard.cpp:1245-1253,1332-1341` | **Green:** corpus row restored to `exact`; obsolete `draw-stream-diverges:V11` removed. |
| V12 | **CLOSED 2026-08-03: `db_health_tracker` bounded settlement now matches C++.** Rust had the pinned five-pass ceiling and per-pass resets, but treated pending DataBind bookkeeping as a second continuation condition after Component dirt was clean. The port now breaks solely on clean Component dirt after each reset, matching `state_machine_instance.cpp:2649-2707`. The three-sample debug runner completes, and the full draw plus side-channel comparison is exact. | `corpus.toml` `db_health_tracker`; targeted settlement regression; one-row scripted golden comparison: 3/3 exact segments and 3/3 exact side-channel segments | **Green:** `db_health_tracker` is restored to `exact`; `post-zero-runtime-hang:V12` is removed; the corpus-wide scripted golden gate remains regression-free. |
| V13 | **CLOSED 2026-08-03 — `animated_clipping` is exact.** Animated `ClippingShape` dirt now stays with the clipping owner instead of replacing the Text occurrence's retained glyph `RenderPath`; ids remain stable at samples 0/0.5/1. | `corpus.toml` milestone V13; exact three-sample C++/Rust side-channel comparison | Keep the row exact. |
| V14 | **CLOSED 2026-08-04 — `artboard_list_overrides` is exact in the scripted lane.** LayoutComponent size retention now publishes C++'s Path-before-World dirt, layout clip backends key their concrete owner revision, hosted row resizes refresh the mounted root constraint frame before child update, and `TextVariationHelper::update` no longer synthesizes a second TextShape publication. | `corpus.toml` milestone V14, samples 0/0.5/1; exact scripted side-channel comparison; `retained_layout_size_change_publishes_path_before_world`; `layout_clip_backend_key_tracks_the_layout_path_owner`; `component_list_row_resize_refreshes_the_mounted_root_constraint_frame`; pinned `layout_component.cpp:1153-1178`, `text_variation_helper.cpp:7-17` | Closed; keep ordinary and scripted status `exact`. |
| V15 | **CLOSED 2026-08-03 — `bad_skin` is exact under its 0.0004 contract.** An unchanged settled gradient no longer rematerializes a shader on the retained ShapePaint occurrence at t=2/t=4. | `corpus.toml` milestone V15; exact three-sample C++/Rust side-channel comparison | Keep the row exact. |
| V16 | **`bankcard`: authored gradient/inner-feather dependency topology is ported; one compound inner-path contour remains displaced.** Rust now interleaves `LinearGradient`/`RadialGradient` and `Feather` edges at their authored component slots, matches the nested `Artboard` C++ graph orders (`1@5`, `367@6`, `365@7`, `360@8`, `332@9`, `334@18`, `335@19`, `337@20`), retains the layout path at the Feather dependency node, and makes layout drawing consume that settled owner. The exact t=2 gate still finds one 39-point contour in draw path 54 translated by about `+0.014061` on Y; all other material operands are within the existing exact comparator. | `corpus.toml` milestone V16, samples 0/1/2; `bankcard_inner_feather_dependency_order_matches_cpp`; `linear_animation.cpp:71-84`; `feather.cpp:36-89` | Keep `diverges`; the dependency-order residual is closed without tolerance. Isolate the remaining single-contour `RawPath::addPathBackwards`/compound-glyph transform ownership difference before restoring `exact`. |
| V17 | **CLOSED 2026-08-04 — `bullet_man` preserves `StaticScene` time semantics for the mounted `Sparks` TrimPaths.** Path 19 is the first retained TrimPath output of the `Sparks` nested artboard (host 649, simple animation 650), not skinned geometry. The artboard has a state machine but no authored default-state-machine ID, so pinned C++ selects `StaticScene`, ignores each sample delta, and advances the artboard at zero elapsed time (`static_scene.cpp:22-28`). The Rust golden runner now does the same instead of advancing mounted frame components by the sample clock. | `corpus.toml` `bullet_man`, samples 0/0.5/1; exact focused draw and side-channel comparison (3/3 each); pinned `static_scene.cpp:22-28`, `nested_simple_animation.cpp`, `trim_path.cpp` | **Green:** row restored to `exact` under its existing `tolerant(0.0005)` numeric contract. Explicit state-machine scenes retain their positive-time frame-component advance. |
| V18 | **`clipping_and_draw_order`: post-zero transform diverges.** Rust emits translation (1121,259) where C++ emits identity. | `corpus.toml` milestone V18, samples 0/0.5/1 | Reconcile post-zero draw-order target transform state and restore `exact`. |
| V19 | **CLOSED 2026-08-04 — `component_list_child_origin` is exact in the scripted lane.** Pooled rows retain their hosted root layout result, nested row changes no longer discard that parent-owned size, adopted Text owners rewind their retained opacity paths, and non-virtual list drawing composes the fresh Yoga base with the retained parent `ScrollConstraint` before drawing. | `corpus.toml` milestone V19, samples 0/0.5/1; exact scripted side-channel comparison; `component_list_mount_settles_context_without_advancing_the_row_state_machine`; `upstream_clipped_component_list_fixture_imports_virtualized_scroll_owners`; pinned `artboard_component_list.cpp:1153-1191,1302-1358` | Closed; keep ordinary and scripted status `exact`. |
| V20 | **CLOSED 2026-08-03: `component_stateful_vm_instance` is exact after child-first stateful VMI binding.** Nested state-machine construction no longer consumes a default context before the mounted child receives its active VMI. | `corpus.toml` `component_stateful_vm_instance`, samples 0/0.5/1; pinned `nested_artboard.cpp:156-185` | **Green:** targeted scripted comparison exact at all three samples; corpus-wide `make scripted-golden-compare` has zero failed entries. |
| V21 | **CLOSED 2026-08-03: `component_stateful_vm_instance_2` is exact after child-first stateful VMI binding.** The nested machine consumes the mounted child's completed DataContext, preserving the authored transform sign. | `corpus.toml` `component_stateful_vm_instance_2`, samples 0/0.5/1; pinned `nested_artboard.cpp:156-185` | **Green:** targeted scripted comparison exact at all three samples; corpus-wide `make scripted-golden-compare` has zero failed entries. |
| V22 | **Artifact reclassified — runner-feature split.** Fresh ordinary runners are exact at t=0/0.5/1, while fresh scripting-enabled runners reproduce the 245×250-vs-490×362.5 clip mismatch. The corpus cannot encode ordinary `exact` with scripted `diverges`, so the conservative shared annotation remains `diverges`. | fresh ordinary and scripted side-channel captures; `corpus.toml` milestone V22, samples 0/0.5/1 | Track the scripting-feature initialization artifact separately; do not attribute it to computed-value propagation in the ordinary runtime. |
| V23 | **CLOSED 2026-08-03 — `death_knight` is exact.** The retained ShapePaint shader is reused when the queued settled gradient state is unchanged. | `corpus.toml` milestone V23; exact three-sample C++/Rust side-channel comparison | Keep the row exact. |
| V24 | **PARTIAL 2026-08-03 — retained-paint loss is closed; a later gradient-order gap remains.** Rust now completes all samples and paint global 584 remains addressable through its concrete Text occurrence. The next mismatch is line 1726 at t=0.5: Rust creates the `(1.121,216.208)→(2.858,-198.985)` gradient where C++ creates the `(101.369,-1.169)→(103.401,210.817)` gradient. | `corpus.toml` milestone V24, samples 0/0.5/1; exact-mode side-channel repro | Reconcile live TextStylePaint gradient dependency order, then restore `exact`. |
| V25 | **Script-update invalidation ported; chained GroupEffect output remains divergent.** A true scripted path-effect advance now schedules `ScriptUpdate` at the effect dependency slot, but fresh t=0/0.5/1 capture shows Rust rebuilding only the short retained TargetEffect chain while C++ rebuilds the full compound group path. | fresh scripted side-channel capture; `true_scripted_path_effect_advance_schedules_effect_invalidation`; `scripted_path_effect.cpp:111-132,199-207`; `shape_paint.cpp:115-152` | Keep `diverges`; retain occurrence-specific GroupEffect/TargetEffect proxy output through invalidation before promoting. |
| V26 | **PARTIAL 2026-08-03 — retained-gradient rematerialization is closed; a later path gap remains.** Gradient 189 is no longer recreated at t=0.5. The next mismatch is draw path 397 at stream line 6677 under the 0.0015 contract. | `corpus.toml` milestone V26, samples 0/0.5/1; tolerant side-channel repro | Reconcile the later animated path geometry and restore `exact`. |
| V27 | **Artifact reclassified — runner-feature split.** Fresh ordinary runners produce byte-identical 417,801-byte streams, while fresh scripting-enabled runners reproduce the image-buffer command-phase mismatch. The corpus cannot encode ordinary `exact` with scripted `diverges`, so the conservative shared annotation remains `diverges`. | fresh ordinary and scripted side-channel captures; `corpus.toml` milestone V27, samples 0/0.5/1 | Track the scripting-feature initialization artifact separately; ordinary image-fit geometry is exact. |
| V28 | **CLOSED 2026-08-03: `multi_listeners` retains keyed callbacks across nested-notification settlement.** The callback walk already covered the pinned `(lastTime, newTime]` interval in authored order; the runner's nested-notification follow-up now uses the non-NewFrame zero-time probe, so it no longer consumes `main-event-2` before host reporting. | `corpus.toml` `multi_listeners`, samples 0/0.5/1; targeted C++/Rust stream comparison including side channel | **Green:** event `main-event-2` reports at t=1 with delay `0.183333337`; all three draw and side-channel segments exact. |
| V29 | **CLOSED 2026-08-04 — retained paint and post-keyframe mixed-style shaping are exact.** Rust retains HarfBuzz offsets, wraps from contextual per-style advances like C++ `BreakLines`, and outlines in authored font units before normalization. `new_text` now matches all 0/0.5/1 draw and side-channel segments. | `corpus.toml` milestone V29, samples 0/0.5/1; exact-mode side-channel repro | Closed. |
| V30 | **Feather invalidation is wired, but its upstream effect path remains divergent.** V25's `ScriptUpdate` reaches `ShapePaint` before feather preparation; fresh dense capture still differs because the effected path supplied to the inner-feather rebuild is the shortened Rust chain rather than C++'s full retained path. | fresh scripted side-channel capture; `true_scripted_path_effect_advance_schedules_effect_invalidation`; `scripted_path_effect.cpp:111-132,199-207`; `shape_paint.cpp:115-152` | Keep `diverges`; close the V25 retained-chain residual, then reverify inner-feather geometry. |
| V31 | **PARTIAL 2026-08-04 — retained-gradient rematerialization and root-gradient construction order are closed; a weighted rounded-path gap remains.** Gradient 51 is not recreated at t=0.5, and root gradient 49 now observes the numeric ContextValue default before its sibling target-to-source write. The next mismatch is draw path 140 at stream line 1930: C++ retains a one-ulp midpoint line between the radius-129 and radius-54 weighted vertices while Rust collapses it. V17 is now independently closed and does not own this residue. | `corpus.toml` milestone V31, samples 0/0.5/1; tolerant side-channel repro; pinned `artboard.cpp:1195-1203`, `data_bind_container.cpp:156-203`, `path.cpp:330-371` | Keep `diverges`; isolate the weighted rounded-vertex midpoint/verb preservation arithmetic. |
| V32 | **CLOSED 2026-08-03: `scripted_as_path` exposes the current retained authored `Path::rawPath()` after dependency update.** The script-facing snapshot now follows the pinned lookup semantics instead of allocating an empty path on every lookup; the existing scripted drawable advance/dirt-before-draw lifecycle then emits the authored seven-segment closed path at post-zero samples. | `corpus.toml` `scripted_as_path`, samples 0/0.5/1; byte-identical scripted C++/Rust streams (8,086 bytes) | **Green:** focused retained-path snapshot test and targeted scripted golden comparison; row restored to `exact`. |
| V33 | **CLOSED 2026-08-03 — `stateful_keyed_trigger` is exact at t=0/0.5/1.** Keyed callbacks now fire the authored `ViewModelInstanceTrigger`; nested context forwarding no longer acknowledges that change before `ConditionComparisonSelf` probes it, so the t=0.5 nested paint is the C++ green `0xff07fb5a`. | fresh scripted golden capture; `stateful_keyed_trigger_reaches_nested_comparator_in_the_same_frame`; `transition_viewmodel_condition.cpp:49-60,1098-1108`; `state_machine_instance.cpp:2665-2697` | **Green:** corpus row restored to `exact`. |
| V34 | **CLOSED 2026-08-03: `stateful_nested` is exact after host-first retained-VMI synchronization.** Authored keyed values reach the detached mounted occurrence before its nested machine advances, matching C++ shared-pointer visibility. | `corpus.toml` `stateful_nested`, samples 0/0.5/1; pinned `nested_artboard.cpp:156-185` | **Green:** targeted scripted comparison exact at all three samples; corpus-wide `make scripted-golden-compare` has zero failed entries. |
| V35 | **Artifact reclassified — ordinary oracle crash versus scripted geometry capture.** The fresh ordinary debug C++ runner terminates with SIGSEGV before comparison; fresh scripting-enabled runners complete and reproduce the filed 100-vs-75 radius mismatch. The earlier “stale capture” conclusion came from treating an unfinished runner session as success. | fresh ordinary and scripted side-channel captures; `corpus.toml` milestone V35, samples 0/0.5/1 | Keep `diverges`; diagnose the ordinary C++ diagnostic-path crash separately from the scripted source-switch geometry difference. |
| V36 | **Artifact reclassified — runner-feature split.** Fresh ordinary runners are exact at t=0/0.5/1, while fresh scripting-enabled runners reproduce the empty-path-vs-compound-path mismatch. The corpus cannot encode ordinary `exact` with scripted `diverges`, so the conservative shared annotation remains `diverges`. | fresh ordinary and scripted side-channel captures; `corpus.toml` milestone V36, samples 0/0.5/1 | Track the scripting-feature path-retention artifact separately; ordinary path retention is exact. |
| V37 | **CLOSED 2026-08-03: `text_vertical_trim_test` reshapes and reflows generated top/bottom trim changes before render placement.** The generated bitmask-passthrough fields now take the same shape/layout invalidation path as `verticalTrimValue`, and a same-size solved layout move dirties the owner's world transform instead of retaining the previous y=187.584229 placement. | `corpus.toml` `text_vertical_trim_test`, samples 0/0.5/1; targeted C++/Rust stream comparison including side channel | **Green:** C++ y=182.76001 at t=0.5; all three draw and side-channel segments exact (266,528-byte stream); row restored to `exact`. |
| V38 | **CLOSED 2026-08-03: `viewmodel_instance_to_artboard` is exact after atomic occurrence replacement.** The selected local VMI and inherited parent fallback are bound to the replacement child before its state machine is created and consumes the context. | `corpus.toml` `viewmodel_instance_to_artboard`, samples 0/0.5/1; pinned `nested_artboard.cpp:228-350` | **Green:** targeted scripted comparison exact at all three samples; corpus-wide `make scripted-golden-compare` has zero failed entries. |
| V39 | **CLOSED 2026-08-04 — virtualized rows retain renderer owners across pool reuse.** Rust previously reconstructed each pooled child from a fresh clone and lost the ShapePaint, ShapePaintPath, and LayoutComponent backend sidecars even though it kept the outer row cache. Authored property restoration now moves those renderer owners onto the refreshed CPU state, matching C++ recorder replay on the same Artboard occurrence. | `corpus.toml` `virtualize_blendmode`, samples 0/2/4; `pooled_component_list_renderer_backends_survive_authored_state_restore`; exact scripted draw and side-channel capture | **Green:** all three scripted segments are exact; the scripted divergence annotation is removed. |
| V40 | **CLOSED 2026-08-03 — `zombie_skins` is exact.** The three retained skin-gradient occurrences keep their installed shaders across t=1/t=2 instead of recreating ids 30–32 and 54–56. | `corpus.toml` milestone V40; exact three-sample C++/Rust side-channel comparison | Keep the row exact. |
| V41 | **`paused_nested_artboard_opacity`: nested opacity differs after enrollment.** Rust emits alpha `0xf7` where C++ emits `0xff` for the same `0x6e0000` color payload. | `corpus.toml` milestone V41, samples 0/0.5/1 | Reconcile paused nested-artboard opacity propagation and enroll the row as `exact`. |
| V42 | **`stateful_component_image_test`: image decode command order differs.** Rust emits `decodeImage` before the paint record that appears first in the C++ stream. | `corpus.toml` milestone V42, samples 0/0.5/1 | Reconcile stateful component image decode/paint ordering and enroll the row as `exact`. |
| V43 | **`data_bind_blob_test`: data-bound blob geometry differs.** At the first differing draw, Rust's rectangle height is 2098.35938 while C++ uses 926.574219. | `corpus.toml` milestone V43, samples 0/0.1/0.5/1/2 | Reconcile blob-bound layout/geometry and enroll the row as `exact`. |
| V44 | **`artboard_opacity_and_transform_test`: the Rust runner lacks nested-child data binding.** It exits on data-bind global 29 (`data-binding-nested-child`, target `Artboard`) before a stream can be compared. | `corpus.toml` milestone V44, samples 0/0.5/1 | Implement the nested-child binding surface, compare the complete stream, and enroll the row as `exact`. |

## F — Feature/subsystem gaps (code that does not exist)

Ranked by upstream line count × product relevance. "Historical backlog"
ceilings from the original port's status log (git history: `docs/v2-status.md`)
are merged in.

| id | subsystem | size (≈lines) | status | notes |
|---|---|---|---|---|
| F1 | **Audio** — `src/audio/**` engine/source/sound/reader, `audio_event.cpp` firing, `Artboard::volume` | 1,030+ | PARTIAL (P2F1/P2F2) | Symphonia WAV/MP3/FLAC source/reader decode, file-owned AudioAsset loading, Factory decode, the Rive-owned headless frame-clock/mixer/sound lifecycle and retained default engine, dense-ordinal AudioEvent playback, multiplied Artboard volume, recursive engine/volume propagation, and Artboard-scoped teardown are ported under D17. Lua audio and CPAL device output remain later packages. |
| F2 | **Text input editing** — cursor motion, selection, keyboard routing (`raw_text_input.cpp` 992, `text_input.cpp` 777, `cursor.cpp` 359, selection/selected-text files) | ~2,400 | CLOSED | FL-E6 ports the retained buffer/journal, cursor and selection paths, key/committed-text routing, multiline source/display behavior, pointer multi-click/drag selection, focus request, and scroll-viewport edge advancement. The remaining non-TextInput gamepad/semantic listener work stays in F5. |
| F3 | **Command queue/server** — threaded host command API (`command_server.cpp` 3,821 + `command_queue.cpp` 2,321) | 6,142 | CLOSED (83/83) | The direct `CommandQueue`/`CommandServer` port covers all 83 pinned cases. S4-45's four blob handle/message cases are baseline runtime behavior, with their upstream provenance and Rust adaptations recorded in `docs/command-queue-test-ledger.md`. FlowSession remains a separate renderer-neutral product transaction protocol, not the command-port substitute or an iOS-owned baseline API. |
| F4 | **Scroll physics** — `elastic_scroll_physics.cpp` (303), `scroll_bar_constraint(.proxy)` (237+), momentum/virtualized scroll | ~700 | PARTIAL | Clamped/core scroll constraint ported at sample-0; interactive momentum, elastic overscroll, scrollbars absent. Paywall-relevant (scrolling lists). |
| F5 | **Keyboard/gamepad/semantic/text-input listener groups + input runtime** (`*_listener_group.cpp` 481, `gamepad_batch.cpp` 363, inputs/) | ~930 | ABSENT | Pointer listeners only. Blocks F2 interaction and any keyboard-driven content. |
| F6 | **Semantics/accessibility** — `semantic_manager` 1,109, `semantic_data` 572, provider, inference registry | 1,926 | CLOSED (FTAIL) | The retained runtime and LT-1 full diff/action/focus side channel are implemented against `4ac7b327`. Nested focus, Simpsons, and data_binding_lists are exact. The latter now shapes its four initial mounted Text bounds through the same retained glyph path used for drawing (`text.cpp:534-615,1154-1233`; `semantic_data.cpp:273-293,501-532`). Component settlement journals generic owner WorldTransform/Path dirt once per semantic synchronization, replacing the former snapshot-only refresh; the dedicated journal test and exact three-sample data_binding_lists projection close the named SEMRES remainders. |
| F7 | **Unported Lua bindings** — `lua_gpu` 3,734, `lua_promise` 1,323, `lua_scripted_context` 583, `lua_buffer_ext` 538, `lua_audio` 507, `lua_data_value` 503, `lua_image_decode` 467, + mesh/color/image/blob/state/data_context/gradient/input | ~9,800 | PARTIAL (by design) | FTAIL partially promotes `lua_scripted_context.cpp`: the full Context method-name surface, headless `features`, and sized/deferred `gpuCanvas` descriptors are present. Canvas 2D has the named `scripted-context-canvas` runtime diagnostic, but its unsupported corpus fixture remains open because no pinned/importable fixture invokes it; component-derived owner-specific `markNeedsUpdate` also remains named residue. The GPU-prefixed `lua_gpu.cpp` candidate still retains its own Canvas 2D and `Image:view` residue. Other named Lua families retain their correspondence status and corpus gates. |
| F8 | **ORE scripted GPU host** (GPUBuffer/GPUCanvas contexts) | — | DEFERRED (`deferred-2026-07-19-ore-gpu`) | GPU-prefixed userdata reaches wgpu under the approved D18 adapter ceiling. Native ORE remains deferred; Canvas 2D/`Image:view` are outside the adapter. |
| F9 | **Joystick runtime behavior** | 169 | PARTIAL (verify) | Only property keys found; confirm advance/apply behavior or add fixture proving it. |
| F10 | **Behavioral-verify candidates** — concrete typeKeys with no bespoke handler: `ClampedScrollPhysics`/`ElasticScrollPhysics` (524/525), `ListPath` (619), `ListenerInputTypeEvent/Text` (659/666), `TransitionValueIdComparator` (601) | — | UNKNOWN | Cheapest wins in the register: author one fixture each; either it's generically handled (close row) or it diffs (new F-row). |
| F11 | **Compressed-texture decoders** (astc/bc/ktx2/etc) | 735 | ABSENT | GPU texture path; relevance depends on whether editor exports these. |
| F12 | **Async work pool** (346) + **profiler** (407) | 753 | PARTIAL | P1-m ports the profiler records, lifecycle, wire format, and runtime hooks; its capture backend is the declared D16 adaptation. The async work pool remains absent; the current command-server port provides server-thread confinement without claiming that separate work-pool correspondence. |
| F13 | Historical backlog ceilings (from the original port's status log): full ListenerGroup drag/opaque behavior, nested pointer/listener hit propagation beyond event bubbling, live data-bound nested-host controls beyond generated defaults, richer static-text modifiers (shape/origin, gradient text effects) | — | LATENT | Currently exact for all corpus files; will surface as diffs when fixtures exist (see C-rows). |
| F14 | `binary_writer`/`binary_data_reader`, `static_scene.cpp`, `hittest_command_path.cpp`, `intrinsically_sizeable.cpp` | ~350 | ABSENT (accepted) | Read-only runtime doesn't need writers; note and close. |
| F15 | **Participant layout animation** — the C++ `ParticipantAnimation` lifecycle (`layout_participant.cpp:29-43,398-455,508-644`: `cascadeLayoutStyle` allocation, `advanceComponent`, `applyInterpolation` incl. smoothing/retarget). | ~300 | PARTIAL | UNIV-1603; found by B-6 post-audit row B6-0455 (2026-08-04), ported 2026-08-05: `concrete.participant_layout` reuses the LayoutComponent animation state in inherit-only mode; cascade reaches participants through transparent containers; the participant advances as its own AdvancingComponent; solve settles retarget through `retain_bounds`; parametric-path control size reads the animated slot. Upstream `layout_participant_test.cpp` "animates its slot" (:203), "re-targets in flight" (:256), and "disabling interpolation frees animation" are ported and bind to the implementation (neutralizing the advance entry or the cascade arming fails them). Animated **position** is now interpolated too (2026-08-05): `is_interpolating()`-gated overrides in both world-transform reads serve the retained animated x/y for layouts and participants on the parent-mapped path, differentially proven by silver `layout_grid_stack_grid_with_layouts_size_changing` flipping byte-exact (was `transform ty: expected 336.03, got 310`); the mid-flight animation family is covered by the silver corpus (layout_anim_* rows now diverge only in the redundant-rewind cadence form, the D12 retained-cache design). REMAINDER: (a) the artboard-origin fallback convention in the world-transform read stays un-overridden (retain/read conventions differ there; commented in code); (b) Text-host layout constraints read the solve, not the animated slot, during interpolation. |

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
| A6 | **Command-server product adoption incomplete** — the 83/83 baseline port closes the old “no model” premise. The current iOS pin predates the completed port, and Flow's synchronous rollback/ordering/wake/error contract is not yet proven equivalent. | 2 |
| A7 | **Artboard resize/layout override not first-class** (`width(x)`, `layoutWidth/Height`, `updateLayoutBounds`, `resetArtboardSize`) — only `raw_mut().set_artboard_dimensions`. Responsive hosts need this. | 2 |
| A8 | Async decode callbacks; RTTI-style typed queries; semantic-tree protocol (pairs with F6). | 3 |

Nuxie-only *additive* surfaces are not parity gaps. `scene::Scene` authoring and
authored observation policy move to the editor owner; independently justified
low-level text caret/hit/selection observations remain baseline APIs.

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
4. luaur-rt pinned =0.1.8 as the scripting engine (mlua fallback untriggered). Luau engine-version skew is **CLOSED 2026-08-05**: the engine is now an in-house fork ported rung-by-rung to the pinned C++ engine's `rive_0_732` tip (docs/luau-fork.md; `deferred-2026-07-19-luau-engine` exit criterion was fork parity, not an upstream luaur release). The vendored crate version stays `0.1.8` because the `[patch]` must satisfy the `=0.1.8` pins; fork state is identified by docs/luau-fork.md plus per-crate `NUXIE_PATCH.md`.
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

**P2 — SDK completeness.** A6 integration/equivalence follow-through, A7
(resize), remaining capi/VM coverage, F10/C1 cheap fixture sweeps, V6/V7
renderer-oracle hardening, V10 blocking perf gate, H2 size re-measure.

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

## W — Upstream watch items (blocked on upstream fixes)

| id | gap | status | notes |
|---|---|---|---|
| W3 | **Upstream semantic UAF on nested-artboard swap** — `NestedArtboard::nest()` destroys the outgoing `ArtboardInstance` without evicting its `SemanticManager` nodes; next `drainDiff()` dereferences a freed `LayoutComponent` (heap-use-after-free, ASan-proven, deterministic in debug) | WATCH (upstream-blocked at `4ac7b327`) | Side-channel comparison quarantined for `replace_view_model` and `data_binding_artboards_test` via `side_channel_divergence`; draw streams stay compared. Rust port must be audited for the mirrored eviction gap (nested-VMI lane adjudication). Details and a candidate upstream patch: `docs/watch-cpp-nest-semantic-uaf.md`, `docs/watch-cpp-nest-semantic-uaf-candidate-fix.patch`. |
