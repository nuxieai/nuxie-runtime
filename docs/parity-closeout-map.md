# Parity Closeout Map

The execution plan for `docs/parity-gap-register.md`. The register is the
WHAT (every gap, with evidence); this map is the HOW (tickets, slices, exit
gates, dispatch); `docs/parity-closeout-status.md` is the live state.
Session protocol: `.claude/commands/parity.md`.

Mission: close the register to the point where the **parity scorecard**
(below) is green in CI, so the public claim "a verifiable, faster
replacement for the Rive runtime" is backed by named gates, with all
remaining non-parity declared in the register's D-list.

Method inheritance: everything in `docs/PORTING.md` (idioms, fence rules,
float lore) and the V2/R culture applies unchanged — port code not
behaviors, oracle-first, half-day divergence budget, no tolerance widening,
statuses only move via mechanical gates.

---

## The scorecard (the metric)

Five tiers. Each tier is a named command gate; `make parity-scorecard`
(built in #B-4) prints the table. The project health number is
**tiers-green (0–5)** plus the per-tier ratchets. CI publishes it per
commit. "Complete" = tiers 1–5 green with every exception recorded as a
D-row.

| tier | claim | gate(s) | ratchet number |
|---|---|---|---|
| 1 Frame parity | .riv → identical frames, end to end | `make golden-compare` + `scripted-golden-compare` (floor) + `make e2e-golden` (#OR-6) | exact-segments; e2e-exact |
| 2 Interaction parity | events, hits, inputs, settling identical | side-channel compare (#OR-1/2), script verbs (#OR-3), densified sampling (#OR-4/5), differential fuzz nights (#OR-7) | side-channel-segments; fuzz-clean-nights |
| 3 SDK parity | embedders can do what rive-cpp embedders can | A-row checklist in the register (#FT tickets) | A-rows closed / total |
| 4 Platform parity | renderer verified where we ship | pixel ratchet (floor) + adapter matrix (#HD-2) + WebGL2 decision (#HD-3) | pixel-exact entries; adapters ≥ 2 |
| 5 Performance & size | faster than C++, size within budget | blocking `make perf-hot-loop` ratio ≤ 1.0 (#OR-9) + `r4-timing-gate` + `make size-report` within budget (#B-3) | rust/C++ ratio; MiB |

**Honesty invariants** (inherited): stub-baseline re-verification whenever a
comparator changes (a do-nothing implementation must fail every active
entry — this applies to the new side-channel and e2e comparators too), and
no worker may loosen a gate to pass it.

**The regression floor** at all times: `make golden-compare`,
`make scripted-golden-compare`, `make renderer-golden`,
`cargo test --workspace`, `make capi-smoke` — green at their completed
values. Any slice that regresses the floor is reverted before other work.

---

## Dispatch model

Same three thread shapes as Phase R, plus one new one:

- **Spine (single-writer).** Exactly one thread at a time may edit: the two
  golden runners (`tools/golden-runner`, `tools/rust-golden-runner`),
  `tools/golden-compare`, the stream/side-channel format, `corpus.toml`,
  `corpus-r.toml`, the status file, and CI workflows. All format changes are
  spine work. The orchestrator session IS the spine by default.
- **Lane (own worktree, orthogonal modules).** Feature work that touches
  crates the spine isn't editing: asset-loader seam, audio, scroll physics,
  capi surface, e2e harness tooling, fuzzer, port-manifest tool. `git
  worktree add <dir> -b lane/<name>`; `[lane-<name>]` commit prefix; merge
  only after the orchestrator re-runs the gates itself.
- **Scout (read-only fan-out).** Evidence gathering: probe a typeKey's
  handling, classify a divergence, inventory upstream commits, draft sample
  times for corpus entries. Scouts never write; results fold into the
  status file.
- **Batch fan-out (new).** Homogeneous corpus work sharded across workers:
  sampling densification, fixture authoring, input-script authoring. Each
  worker gets a disjoint entry list and writes ONLY fixture/asset files and
  a report; the spine applies all `corpus.toml` edits from the reports.
  ~10–60 entries per brief; never two workers on one entry.

**Model routing** (inherit the goal.md two-tier convention — route by
verifiability, not difficulty). Executor-tier: mechanical translation with
a C++ citation, batch fan-out briefs, compiler burn-down, fixture
authoring, regeneration, fuzz babysitting. Planner-tier only: format/gate
design (#OR-1 spec, e2e comparator, scorecard definitions), novel
divergence root-cause, all merges and ratchet verdicts, anything touching
the D-list or tolerances, USER-GATE preparation. Escalation: executor
fails its gate twice → planner takes it with the failure context.

**Worker brief template** (every dispatched worker gets exactly this):

```
CONTEXT (read first): docs/PORTING.md, docs/parity-gap-register.md (row(s)
  <ids>), docs/parity-closeout-map.md (slice <id>), current
  docs/parity-closeout-status.md entry for <id>.
GOAL: <one sentence — the slice's exit gate restated>.
STEPS: <the slice's steps, with C++ source citations where porting>.
TOUCH: <exact files/dirs>. DON'T TOUCH: corpus.toml, corpus-r.toml,
  docs/parity-closeout-status.md, tools/golden-* (unless slice says),
  frozen: nuxie-schema, nuxie-binary (unless Phase S says), V1 contract
  suite, any tolerance or gate threshold.
GATE: run <command(s)>; paste the numbers. If the gate fails twice for the
  same cause, STOP and report the failure context — do not loosen anything.
REPORT: a status-file entry draft (metric delta, files touched, decisions
  needed) + the gate output. Your final message is the report.
```

**USER-GATE rows** (orchestrator stops and asks; never inferred): Phase S
approvals (#B-1, per `docs/upstream-sync-map.md`), size budget (#B-3),
audio engine confirmation (#FT-AUDIO a), command-server decision (#HD-1),
WebGL2 decision (#HD-3), production-flow corpus access (#FT-PROD), and any
new D-row (deliberate divergence) — declaring accepted non-parity is always
a user decision.

---

## Porting methodology — structure-preserving by default (standing; user-directed 2026-07-22)

Adopted from the Anthropic large-scale-migrations write-up (the Bun
Zig→Rust precedent: structure-preserving, file-corresponding translation
with the old code as the spec — NOT the redesign mode). Our own closeout
history is the evidence: every major incident family traces to a
subsystem that reimplemented C++ behavior under a different design
(compensation machinery → #RB-1, scene-level replay caches → #RD-1, CSS
text spacing), while faithfully ported code has stayed quiet.

1. **Structure-preserving is the default for all divergent subsystems.**
   When a #B-6 divergent family (or any new divergence) is fixed, the fix
   is "port the corresponding C++ file(s), replace ours" — not "patch our
   design until behavior matches." Point-fixing inside a divergent design
   is the failure mode that #RB-1's four-red evidence trail documents.
   Already-faithful, gate-green code is NOT re-ported for its own sake.
2. **The rulebook is codified, not implicit.** docs/PORTING.md carries the
   translation table: C++ idiom → Rust idiom (rcp/RefCounted →
   Rc<RefCell> cell handles + weak dirt sinks, DependencyHelper →
   RuntimeCellDirtSink, Core property setters → arena apply seam, §8
   AF-1..8, etc.). Every new mapping established by a rebuild phase is
   added to the table when the phase lands. What C++ expresses that Rust
   cannot directly (inheritance dispatch, intrusive lists) gets a gap-
   inventory row with the chosen mapping and its C++ citation.
3. **Stress-test before fan-out.** Any multi-file porting phase (starting
   with #RD-1) begins with: two agents independently translate the same
   2–3 representative files — one strictly from the rulebook, one "as a
   senior Rust engineer" — diff the results, fold every disagreement into
   the rulebook as a new rule, then DISCARD both translations. Only then
   fan out. (This catches rulebook holes at 3-file cost instead of
   1,400-file cost.)
4. **The file-correspondence manifest is the exit condition.** Every
   in-scope C++ source file maps to a Rust module with exactly one
   status: `faithful` | `divergent-by-decision` (must cite a D-row) |
   `pending`. Seed from the #B-6 447-row manifest. The closeout is DONE
   when no row is `pending` and every `divergent-by-decision` row has its
   D-row. Ratchet rule: a row moves to `faithful` only on an orchestrator-
   verified gate run.
5. **The judge is already validated both directions** (passes against
   C++ by construction; has caught real regressions within hours). The
   iron rules keep it that way: never weakened, never widened, never
   edited to pass.

---

## Phase 0 — Baseline (#B) — do first, mostly parallel lanes

### #B-1 Phase S sync cycle to current upstream — SPINE, S/M, USER-GATE
The fixed cycle-3 approval cut is `b73bc675`, 3 commits past the pin, including
TextInput improvements (`1b4df2ad`) that land squarely on #FT-TEXT. Later
upstream drift is a separate inventory, not an implicit widening of this cut.
Run the `/sync-upstream` workflow as written (inventory → triage report → STOP
for approval → port approved rows → advance pins). **Blocks #FT-TEXT**;
nothing else waits on it.
**Gate:** ratchet green at the new pin; `LAST_SYNCED_SHA` advanced; triage
file committed.

### #B-2 Port-manifest invariant — LANE, M
Convert the one-off provenance sweep into a standing invariant. Build
`tools/port-manifest/` generating + checking `port-manifest.toml`: one row
per upstream `src/**/*.cpp` (447 files today) →
`{status: ported|partial|absent|not-applicable, rust_module, note}`. Seed
statuses from the register's F-table. `make port-manifest-check` fails on:
an upstream file with no row, or a row whose `rust_module` path no longer
exists. Wire into CI and into the Phase S inventory step (a new upstream
file with no row fails triage).
**Gate:** `make port-manifest-check` green in CI; 447/447 rows; seeded
statuses match the register (absent rows carry their F-row id).

### #B-3 Size re-measure — LANE, S, ends in USER-GATE
`docs/SIZE.md` predates Phase R; the 2.50 MiB figure excludes
nuxie-renderer + vendored wgpu. Re-run `make size-report` on the
post-Phase-R release artifact (scripting on and off, with renderer), update
SIZE.md, then STOP: present the number and let the user set the new budget.
**Gate:** SIZE.md current; budget decision recorded in the status file.

### #B-4 Scorecard plumbing — LANE, S
`make parity-scorecard`: prints the five-tier table from existing gate
outputs (floor gates now; later gates report "not built" until their slice
lands). CI publishes it per commit next to exact-segments.
**Gate:** scorecard runs in CI; tier 1 floor components green; unbuilt
gates listed explicitly, never silently omitted.

---

## Phase 1 — Oracle integrity (#OR) — P0; spine first, then fan-out

Order matters: OR-1 → OR-2 → OR-3 are serial spine (format changes); OR-4
through OR-9 fan out after.

### #OR-1 Side-channel format + C++ emit — SPINE, M (register V4)
Design the side-channel stream spec (`docs/side-channel-format.md`):
per-sample `event name=… props=[k:v…] url=… target=… delay=…` (from
`reportedEventCount/At`, including custom properties), `hit result=…` per
pointer verb (the `HitResult` both runners currently discard), `settled=…`
per advance (the `advanceAndApply` return). Implement in
`tools/golden-runner` behind `--side-channel`; teach `golden-compare` to
parse and diff the channel when present.
**Gate:** spec committed; C++ runner emits on 3 hand-picked event/listener
corpus files; comparator round-trips; stub-baseline check (Rust side absent
→ every side-channel segment fails).

### #OR-2 Rust emit + corpus-wide comparison — SPINE, M (V4)
Emit the identical channel from `tools/rust-golden-runner` (the runtime
already exposes `reported_event*`, hit ids, and the advance bool — this is
plumbing, not porting). Enable `--side-channel` for the full corpus run.
This is the moment of truth: new channels may reveal real divergences that
draw-stream parity hid.
**Gate:** `make golden-compare` with side-channel ON — 317 files exact, OR
each divergence localized (half-day budget) and filed as a register row
with a failing corpus entry. New ratchet number: `side-channel-segments`,
published in CI. Tier 2 partially green.

### #OR-3 Script verbs — SPINE, M (V5)
Extend the input-script grammar in BOTH runners, same parser rules:
`setBool <name> <v>` / `setNumber` / `fireTrigger` (direct SM inputs),
`setVmNumber|String|Bool|Enum|Color <path> <v>` + `fireVmTrigger` (the
`--view-model-script` the C++ runner currently rejects — implement it
there), `resize <w> <h>`. Reserve `key <code> <down|up>` and
`textInput <utf8>` verbs in the spec now (implemented by #FT-TEXT).
**Gate:** verb-parity unit tests both sides; ≥5 new corpus entries
exercising direct-input and VM-mutation scripts, exact. M5's "external
mutation" claim becomes actually covered.

### #OR-4 Sampling densification — BATCH FAN-OUT, M total (V2)
237 entries sample t=0 only. Scout pass first: for each, derive proposed
samples (≥3; include a loop boundary and a post-transition time — animation
duration is readable from the file via `rivinfo`/existing tooling). Then
batch workers regenerate reference streams per shard; the spine applies
`corpus.toml` sample updates and re-ratchets.
**Gate:** every animated entry ≥3 samples; `exact-segments` at the new,
larger denominator with zero unexplained regressions (any diff found is a
real caught bug: half-day protocol, file it, fix it, keep the sample).

### #OR-5 Input-script coverage — BATCH FAN-OUT, M (C2)
Only ~14/317 entries exercise any pointer input. Scout: list corpus files
containing listeners/hit shapes with no `input_script`. Batch-author
scripts (pointer + new OR-3 verbs) targeting each file's listeners.
**Gate:** every listener-bearing corpus file has an input script; exact
(with side-channel on, so hits and events are verified too).

### #OR-6 Composed end-to-end oracle — LANE, L (V1)
`make e2e-golden`: for a designated corpus subset (start ≥50 files spanning
text/layout/images/scripting/nested), run `.riv` → Rust runtime → Rust
renderer → PNG, and `.riv` → C++ runtime → C++ Dawn reference → PNG, on the
same adapter; compare under per-row pixel contracts (reuse the corpus-r
contract/tolerance machinery and provenance sidecars). This closes the
"two oracles never compose" hole.
**Gate:** e2e lane blocking in CI at `e2e-exact = N/N`; stub-baseline
verified; contracts inherited, never widened. Tier 1 fully green.

### #OR-7 Differential fuzzer — LANE, L (V3)
The V2 plan's promised discovery engine. Nightly job: corpus files ×
randomized sample times × randomized input scripts (pointer + OR-3 verbs,
valid by construction) through both runtimes, streams + side-channel
diffed. On a finding: auto-minimize (times, then script, then file if
mutation is in scope later) and emit a ready-to-commit corpus entry +
register row. Start with times+inputs only; file mutation stays out of
scope until the harness is stable.
**Gate:** nightly wired in CI; a seeded synthetic divergence (temporarily
patched runtime) is caught and minimized correctly; then 3 consecutive
clean nights → `fuzz-clean-nights` ratchet counts in the scorecard.

### #OR-8 Tolerance root-cause — SCOUT→SPINE, S (V6)
`computed_root_transform` tolerant(0.5) and `list_index_script_access`
tolerant(0.75) are wide enough to hide real drift. Localize each (existing
escalation ladder).
**Gate:** each ≤ tolerant(0.01), or a D-row with the numeric story and the
user's sign-off.

### #OR-9 Blocking perf gate — LANE, S/M (V10)
Broaden `PERF_CORPUS_IDS` to ≥20 files spanning text/layout/data-bind/
scripting/nested; remove `continue-on-error` from the CI perf job; keep the
existing fence rules (release-vs-release, min-aggregation, pinned repeats).
**Gate:** blocking CI at ratio ≤ 1.0 on the broadened corpus; ratio
published per commit. Tier 5 runtime half green.

---

## Phase 2 — Product features (#FT) — P1; parallel lanes, oracle-first

Every FT slice follows oracle-first: the fixture/channel that would FAIL
today lands with (or before) the implementation — a feature without a
failing-then-passing corpus entry does not close.

### #FT-ASSET — asset loader seam (A1) — LANE, M
(a) Design the loader seam: a Rust trait mirroring C++
`FileAssetLoader::loadContents` (asset descriptor + in-band bytes +
factory; consult `cdnUuid`/`cdnBaseUrl`), wired into `File::import`;
byte-attach API becomes a canned loader. (b) Surface: `nux-capi` callback
vtable + Apple ABI. (c) Fixture: an OOB-asset file whose import fails
without a loader today; corpus entry proving in-band vs loader-supplied
parity.
**Gate:** fixture exact via loader path; capi smoke covers the callback;
A1 closed in the register.

### #FT-TEXT — text input interaction (F2, F5-part, A3) — LANE, L, blocked by #B-1
Serial inside the lane: (a) port keyboard/text-input listener groups +
input runtime (`animation/keyboard_listener_group.cpp`,
`text_input_listener_group.cpp`, `inputs/keyboard_input.cpp` — small,
~500 lines total); (b) SPINE COORDINATION: implement the reserved `key`/
`textInput` verbs in both runners (one spine slice, scheduled by the
orchestrator); (c) port editing behavior — cursor motion, selection,
keyboard routing (`raw_text_input.cpp`, `cursor.cpp`, selection files) —
against fixtures replaying typed-text scripts, verified via draw stream +
side-channel; (d) surface text-run get/set + text-input APIs in the
portable capi (A3).
**Gate:** a text-input fixture with a typing/selection script is exact in
both channels; upstream `1b4df2ad` behaviors included (post #B-1); A3
closed. Largest single feature lane — expect its own mini-queue in the
status file.

### #FT-SCROLL — scroll physics (F4) — LANE, M
Port `elastic_scroll_physics.cpp` (303), `scroll_bar_constraint.cpp` (237)
+ proxies, momentum path in `ScrollConstraint`. Fixtures: drag-fling and
scrollbar-drag pointer scripts sampled at ≥4 times (physics decay is
exactly what t=0 sampling missed); verify `ClampedScrollPhysics`/
`ElasticScrollPhysics` typeKeys go corpus-covered (C1 partial).
**Gate:** scroll fixtures exact at all samples; F4 + the two scroll rows of
F10 closed.

### #FT-AUDIO — audio subsystem (F1, A2) — LANE, L, starts with USER-GATE
(a) USER-GATE: confirm engine choice (map says cpal/rodio/kira — a paper
decision; pick one, record it). (b) SPINE COORDINATION: extend the
side-channel spec with `audio start|stop id=… t=…` emitted where C++ fires
audio events — playback *triggers* become oracle-comparable even though DSP
output never will (record that as a D-row: audio sample-stream parity is
out of scope). (c) Port the seam: `AudioEngine` trait, decode via chosen
crate, `audio_event.cpp` firing semantics, `Artboard::volume` + capi/Apple
surface. (d) Fixtures: `sound.riv` + a volume-scripted entry verified via
side-channel.
**Gate:** audio fixtures exact in the side-channel; volume API in all three
surfaces; F1/A2 closed; D-row recorded.

### #FT-CAPI — portable surface completion (A4, A5, A7) — LANE, M, parallelizable by group
Groups, independently dispatchable: (1) events out with custom properties
in the low-level Rust surface + capi (`StateMachineReportedEvent` gains
properties — closes A4 and the `event_report.hpp` thinness); (2) VM
coverage: color/enum/trigger/image/artboard/list ops + typed getters in
capi; (3) `pointer_exit`, input reads, default-artboard fn; (4) first-class
artboard resize/layout override (`width/height`, `updateLayoutBounds`,
`resetArtboardSize` analogs) in facade + capi (pairs with OR-3's `resize`
verb for verification).
**Gate per group:** capi-smoke extended to the new functions; header
regenerated; register row closed. Fragmentation note: each capability must
land in BOTH the portable capi and (where applicable) FlowSession — no new
Apple-only capabilities.

### #FT-PROD — production-flow corpus lane (C3) — LANE, S/M, USER-GATE
Needs the user to supply/point at real Nuxie flow `.riv` files and decide
where they live (private fixture bucket; never the public repo, same rule
as Rive assets). Then: a CI lane running golden-compare (+ side-channel)
over every shipped flow.
**Gate:** lane green in CI over ≥ the current set of shipped flows; new
flows auto-enter the lane.

### #FT-FIXSWEEP — typeKey fixture sweep (F10 remainder, C1) — BATCH FAN-OUT, S
One fixture per uncovered live typeKey: `ListPath`,
`ListenerInputTypeEvent/Text`, `TransitionValueIdComparator`, `Folder`,
`NSlicerTileMode`, `TextVariationModifier`, `TextStyleFeature`,
`BlobAsset`, `ScriptedInterpolator`, joystick (F9). Author in-editor or
mine community files; each entry either goes exact (close the row — it was
generically handled) or diffs (new F-row with a failing entry attached —
the cheapest possible bug discovery).
**Gate per fixture:** corpus entry exact or a filed row; C1 empty except
the semantic family (which waits for #LT-1).

---

## Phase RB — Data-binding foundation rebuild (#RB) — P0 (user-directed 2026-07-21)

### #RB-1 Retained-identity view-model/data-bind core — SPINE, L (own mini-queue)
User decision: rebuild the view-model, data-binding, and value layers to
match the C++ architecture exactly, replacing the copied-value/polling
design. C++ retains the mutable `ViewModelInstanceValue`, registers each
`DataBind` as a dependent (`data_bind.cpp` `DataBind::source()`,
`viewmodel_instance_number.cpp` `propertyValueChanged()`,
`dependency_helper.hpp` `addDirt()`), and propagates dirt through the
ordinary update cycle. The Rust port flattened bound sources into copied
`RuntimeDataBindGraphValue`s and grew a compensation family to reconstruct
the lost identity: whole-model mutation clocks, candidate context vectors
with path rewriting, generation-diff refresh/rebind, listener observed-copy
rescans (bounded at 100 iterations), detached/mirrored instance trees with
alias registries, per-source direction/reconcile flags, converter polling,
and fabricated unresolved placeholders. Evidence that the compensation
family cannot hold the invariant: all 20 editor-cutover regressions repaired
2026-07-21 plus the one still open (four scripted entries) localize inside
it, each introduced by a reasonable-looking edit.
**Port slices (each floor-gated):** (a) retained typed property cells
(`ViewModelInstanceValue` analogs with shared identity, `rcp` ≈ retained
handles); (b) dependent registration + property-level dirt
(`DependencyHelper`); (c) parent-linked `DataContext` replacing candidate
vectors; (d) retained `DataBind`/listener/converter lifecycle with C++'s
favored-direction init ordering; (e) migrate state-machine/artboard/facade
callers; (f) DELETION gate.
**Gate:** all five floors green at completed values (647 exact segments,
707 probe assertions, scripted lane including the four currently-red
entries, 1,468 pixels, workspace) AND the compensation family is deleted —
`mutation_generation`-based rebinds, `RuntimeOwnedViewModelBindingCandidate`
chains, listener observed-copy rescans, alias mirror registries, and the
Scene-wide rebind bit are all gone. Any survivor means the invariant was
not reached. The four red scripted entries are expected to close as a
byproduct; if any stays red it is a fresh divergence to localize on the new
foundation. Editor-team changes to this layer freeze until #RB-1 lands
(or route through the same floor gates).

## Phase RD — C++ runtime drawing port (#RD, historical internal code) — P0 after #RB-1 (user-directed 2026-07-21)

### #RD-1 Runtime objects own drawing state; renderer backend unchanged — SPINE, XL (own mini-map before execution)
User decision (2026-07-21, superseding register D-12): match C++'s
retention boundary instead of the scene-level retained-replay design. C++
retains GPU resources ON the live objects (each Shape its RenderPath, each
paint its RenderPaint, mutated in place under ComponentDirt) and traverses
the live drawable list every frame (`Artboard::draw`, live `willDraw()`),
with backend-private batching below the Renderer interface. The Rust
scene-level replay layer — prepared frames, retained command streams, path
caches, and their eight invalidation epochs — is to be deleted; per-object
retained resources remain (they ARE the C++ design). The performance claim
is subordinate to design fidelity: the perf ratio stays measured and
published, but a post-RD ratio above 1.0 is a user-reviewed number, not a
blocker on this mandate.
**Sequencing (binding):** (1) #RB-1 completes first — one foundation
rebuild at a time while the editor team lands on main. (2) A measured
spike precedes demolition: implement the live-traversal feed for a
representative corpus slice, run the r4/renderer timing gates and
perf-hot-loop on it, and publish the real delta — the number informs
execution order and batching design, not whether to proceed. (2b)
RULEBOOK + STRESS TEST (per the standing porting methodology above):
codify the renderer-feed translation rules in docs/PORTING.md, then two
independent translations of 2–3 representative C++ draw/traversal files
(rulebook-strict vs senior-engineer), diff, fold disagreements into the
rulebook, discard both translations — only then fan out. (3) Then a
lane-by-lane migration executed as FILE-CORRESPONDING PORTS of the C++
draw/traversal sources (`artboard.cpp` draw path, `shape.cpp`,
`image.cpp`, drawable `willDraw` family, draw-target ordering) rather
than reshaping the existing Rust feed code — same lane pattern as #RB-1:
pixel corpus (1,468) and both golden gates are the referee at every
merge — output is identical BY DEFINITION; only the production strategy
changes. (4) DELETION gate: prepared-frame machinery, command-stream
retention, epoch bridges, and the D-12 register row all removed
together; the file-correspondence manifest rows for the renderer feed
flip to `faithful` on the orchestrator's verified run.

The remaining execution language and member-level proof live in
`docs/runtime-drawing-port-map.md`,
`docs/runtime-drawing-ownership.toml`, and
`docs/runtime-drawing-gaps.toml`. “Runtime drawing” means the Artboard/Shape/
Paint/Text/Image object and update code above the existing Renderer API. The
working renderer backend, shaders, atlases, and GPU algorithms are out of
scope.
**Gate:** all floors green at completed values on the live-traversal
feed; zero scene-level cache/epoch mechanisms remaining (the #B-6 audit
re-run over the renderer clusters returns no mutation-gated mechanisms);
perf ratio measured and reported to the user.

---

## Phase 3 — Hardening & decisions (#HD) — P2

- **#HD-1 Threading-model decision (A6) — USER-GATE, then S or L.** Either
  document FlowSession's single-threaded contract as THE supported embedder
  architecture (S: docs + register D-row) or port
  `command_queue/command_server` (L: own mini-map first). Present the
  trade-off with evidence from the register; do not pre-decide.
- **#HD-2 Renderer oracle hardening (V7) — LANE, M.** One additional
  adapter/OS in the pixel matrix; purpose-built C++ oracle config for the
  two clockwise-atomic hypotheses, or reclassify them as area-capped
  D-rows (USER-GATE if reclassifying).
- **#HD-3 WebGL2 decision (V8) — USER-GATE, then M.** Pixel-corpus the
  femtovg path with its own contracts, or declare it a documented degraded
  mode (D-row).
- **#HD-4 TODO(golden) pair (H3) — SPINE, S.** `state_machine.rs:797`
  (`addToHitLookup` — likely interacts with OR-2 hit-channel findings) and
  `draw.rs:3555` (layout-bounds path unification).
- **#HD-5 Publish the claim — SPINE, S.** A public-facing parity document
  generated from the scorecard + D-list: what "verified replacement" means,
  gate by gate, exception by exception. This is the artifact the register
  exists to make honest.

## Phase 4 — Long tail (#LT) — P3, each gated on product need (USER-GATE to open)

- **#LT-1 Semantics/accessibility (F6)** — port `semantic_manager` + data +
  provider (~1.9k) + semantic listener groups; verification needs a
  semantic side-channel extension. Open when embedded-flow accessibility
  becomes a product requirement (likely App Store-driven — flag early).
- **#LT-2 Remaining Lua bindings (F7)** — corpus-gated as designed;
  spot-check that a scripted file touching an unported binding produces a
  NAMED import/runtime diagnostic (that check itself is an S slice worth
  doing in Phase 2).
- **#LT-3 ORE GPU host (F8)** — stays `deferred-2026-07-19-ore-gpu` with
  its recorded exit criteria.
- **#LT-4 Compressed textures (F11), work pool/profiler (F12)** — open on
  evidence a shipped file/perf need requires them.

---

## Tripwires (check at every commit; confess in the status log and requeue)

1. Three commits on one divergence with no entry changing status —
   sub-oracle or file it and move on.
2. Widening any tolerance, or hand-tuning an entry/gate to pass — that is a
   user-level Decision, never a slice detail.
3. A commit message that cannot honestly carry a ticket tag (`[OR-2]`,
   `[FT-SCROLL]`, `[B-2]`…).
4. Scorecard numbers unmoved in ~10 commits while not building ticketed
   infrastructure — change tactics or record a blocker.
5. Implementing a feature no fixture can fail — land the failing fixture
   first (oracle-first rule).
6. A worker report claims green but you haven't re-run the gate yourself —
   merges only after orchestrator-verified gates.
7. Two threads editing spine files — stop one; the spine is single-writer.

## Sizing & sequencing summary

Rough sizes (S <½ day, M 1–3 sittings, L own mini-queue): #B: S+M+S+S.
#OR: M,M,M spine serial; then M,M,L,L,S,S/M parallel. #FT: M, L, M, L, M,
S/M, S — all parallel lanes after their blockers. #HD/#LT: decision-gated.
Critical path: B-1 → OR-1 → OR-2 → OR-3 → (fan-out) with FT-TEXT chained
after B-1 + the key-verb spine slice. Everything else parallelizes into
lanes/batches immediately.
