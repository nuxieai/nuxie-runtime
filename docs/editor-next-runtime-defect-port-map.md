# Editor Next Runtime Defect Qualification and Port Map

This is the finite investigation and execution map for the runtime handoffs
found by the Editor Next cutover. It deliberately does **not** treat the
handoff ledger as a flat bug queue. Every observation must first be localized
to one of four boundaries:

1. pinned C++ runtime ownership/behavior;
2. Rust runtime ownership/behavior;
3. Nuxie's additive typed authoring or browser presentation surface;
4. Editor Next integration, packaging, or stale artifact evidence.

Only a proven C++/Rust runtime mismatch enters a file-corresponding port.

Pinned C++ source:
`d788e8ec6e8b598526607d6a1e8818e8b637b60c`.

Renderer provenance is separate from the runtime pin. The existing reference
registry includes historical renderer revision `7c778d13`; no backend verdict
may rely on that shorthand alone. `F-ED-00` must bind each renderer probe to
the exact renderer repository/submodule revision, Dawn revision, backend and
mode, feature flags, surface format, build-provenance stamp, and reference
executable.

## Input provenance

This map was prepared from these Editor Next handoff artifacts:

| artifact | SHA-256 |
|---|---|
| `nuxie-editor-next-cutover-proposal.md` | `39b17ac5632156f6b762372c28ac661b0a47974d4f2e56ab7d81e32376415401` |
| `nuxie-editor-next-runtime-defects.md` | `9e81f237ed568b873304a5853a05026d06e360f8475bcb6dec4da9d04bf7390c` |
| `nuxie-editor-next-parity-ledger.json` | `04f205269cb833adad7aa15a0e7c18be149c337f0e97bdffce171723eed69e25` |

The source copies live under:

`/Users/levi/.codex/worktrees/7189/nuxie-dev/worktrees/editor-next-cutover-assembly/plans/`

The immutable source checkpoint for those hashes is
`7ca11e331a57cb3ea574848f8e93eb108878c40b`.

The Editor artifacts at this immutable checkpoint consume runtime commit
`e72323c808b91d706ba3b745396beaca7accd69a`. That is not the same thing as
the runtime investigation HEAD or a future merged/consumed repair. Every atlas
row must retain all of those SHAs separately.

There are 25 unique handoff IDs: seven `RT-ED-*` rows and eighteen `LOC-*`
rows. `LOC-010` is a reserved tombstone, not a defect row.

Nine parity children now carry structured runtime links through either
`runtimeDependencies` or `runtimeDefects`: two name `RT-ED-003`; one names
`RT-ED-005`; one names `RT-ED-007`; three name `LOC-002`; one names
`LOC-005`; and one names `LOC-007`. Fifteen additional candidate links remain,
with 23 unique affected children and only `P09-C01` overlapping.
`RT-ED-004` is retained only as historical WebGL2 evidence and has no current
structured child.

## Boundary

The existing runtime frame-loop port remains separate. It begins at
`StateMachineInstance::advanceAndApply`, follows runtime ownership and update
through live Artboard draw, and stops at the existing
`Renderer` / `RenderFactory` interface.

The supported browser boundary is now WebGPU-only. PR #47 removed WebGL2,
fallback selection, and the FemtoVG dependency at merged runtime
`95027109c89f651835c76646ebf4d8734f032f07`. Historical WebGL2 rows remain
evidence, but no ticket in this map may implement or restore WebGL2. The
current operating queue and retirement/requalification rules live in
`docs/editor-next-runtime-defect-goal.md` and supersede historical WebGL2
implementation language below.

This Editor Next program spans a wider product boundary for **qualification**
only:

- C++-corresponding runtime and frame-loop behavior;
- the additive high-level `nuxie::Scene` authoring facade;
- browser presentation adapters in `nuxie-renderer`;
- renderer-backend behavior only when an Editor blocker is proved there;
- artifact publication and Editor integration evidence.

Crossing one of those boundaries does not silently widen the C++ runtime port:

- the 1,468-row renderer floor proves the existing primary renderer reference
  path; it does not prove every browser WebGPU setup and presentation path;
- a missing `Scene` authoring operation is an API-surface gap even when the
  low-level Rust runtime already implements the C++ behavior;
- a browser canvas presentation or initialization failure is an adapter/backend
  defect, not a frame-loop ownership defect;
- a failure from a stale ABI or non-identical editor record is not runtime
  source evidence.

## Live frame-loop concurrency lease

This section records the explicit 2026-07-24 handshake with the active
frame-loop executor. It is a writer lease, not a semantic disposition. It must
be refreshed after every FL wave landing before a new production writer is
assigned.

`FL-A`, the atomic Component/update occurrence graph and exact
Component-to-DataBind collapsables contract, is independently promoted on
`levi/fl-a` at
`f86d5ba0146697abc996310c62fa45e1f053144b`. `FL-B`
(KeyFrame-through-LinearAnimation ownership) has not started production
because the public pre-advance `LinearAnimationInstance::didLoop` behavior
requires the recorded user safety/API decision. `FL-C`
(StateMachineInstance/layer/transition/action/input/listener ownership) and
`FL-D` (Artboard/DataBind/event settlement) remain later waves.

The coordinator canceled Defects Fix's duplicate stable-Apple branch because
Runtime Fix owns that mechanical repair. This does not release any reservation
below: the FL owner remains the sole writer for the listed runtime/graph
families until the coordinator publishes a new lease after that repair and
the `didLoop` decision.

The FL executor is the sole production writer for this current reservation:

- `crates/nuxie-graph/src/lib.rs`;
- `crates/nuxie-runtime/src/artboard.rs`;
- `crates/nuxie-runtime/src/artboard_data_bind.rs`;
- `crates/nuxie-runtime/src/components.rs`;
- `crates/nuxie-runtime/src/constraints.rs`;
- `crates/nuxie-runtime/src/draw.rs`;
- `crates/nuxie-runtime/src/focus.rs`;
- `crates/nuxie-runtime/src/lib.rs`;
- `crates/nuxie-runtime/src/objects.rs`;
- `crates/nuxie-runtime/src/retained_data_bind.rs`;
- `crates/nuxie-runtime/src/text.rs`;
- `docs/runtime-frame-loop-gaps.toml`.

FL integration also reserves `docs/runtime-frame-loop-ownership.toml`,
`docs/runtime-frame-loop-status.md`, and `file-correspondence-manifest.toml`.
`animation.rs` is already reserved for
`FL-B`; `state_machine.rs`, `state_machine/**`, and their Artboard integration
seams are reserved for `FL-C`. No F-ED writer may change runtime
component/dirt/update/clone/DataBind-queue semantics, animation or
state-machine owner internals, or those shared ledgers while the lease is
active.

The earlier F-ED checkout's quarantined `animation.rs` and `artboard.rs`
experiments were never inputs and remain excluded. Q0 changes only the
defect-control documents and fixtures. Any later production F-ED work starts
in a clean worktree from the current merged `origin/main` after an exact
coordinator lease check.

### Work that is unimpeded now

The following work can proceed without waiting for an FL landing:

| F-ED work | permitted now | hard boundary |
|---|---|---|
| `F-ED-00A` atlas/checker/evidence spine | maintain and independently verify the atlas, status, checker, correction manifest, and direct fixtures; run existing ledgers read-only | do not edit shared harness files, probe translation units, or FL ledgers; queue shared-ledger links for the owning FL executor |
| all direct C++/Rust/Editor qualifications | inspect, build, probe, and record evidence in new F-ED-owned files | qualification grants no production-runtime edit |
| `F-ED-03` / `RT-ED-005` | independently verify and promote the exact PR #49 landing provenance | the production authoring repair is complete; do not reopen it or conflate the separate FL-E layout/TextStyle dependency |
| `F-ED-04` / `RT-ED-007` | preserve diagnosis/tests and independently verify the Runtime Fix landing | Defects Fix has no Scene-authoring writer; Runtime Fix exclusively owns the production repair |
| `F-ED-06` / `RT-ED-003` | independently verify and promote the exact PR #55 landing provenance | the production browser-presentation repair is complete; do not reopen it or mix it with historical `F-ED-07` |
| `F-ED-12` | locally build and qualify the existing ABI 1.6 artifact | no header/C-API source change without a fresh scope review; no external publication before the user checkpoint |
| `F-ED-13` | normalize and compare records and repair an Editor/lowering-only first divergence | identical records are required before any renderer attribution; no runtime edit |

`F-ED-04` is assigned to Runtime Fix and authorizes no Defects Fix production
writer. `F-ED-03` and `F-ED-06` are merged landing-provenance rows awaiting
independent promotion only. `F-ED-00A` must still prove any new closure. If an
open lane needs a reserved file or changes the frame-loop/advance contract, it
immediately moves to the deferred set.

### Work that may be prepared but needs a landing handshake

`F-ED-08` is independently closed as a no-repair stale characterization.
`F-ED-10` may perform qualification now, and a resulting fix may be developed only when its
exact production closure is renderer/backend-only. `F-ED-11`
production is complete for `LOC-019`, which needs only independent
landing-provenance promotion. PR #54's `LOC-009` consumer repair at
`7f1450dc` remains historical evidence, but independent real-GPU verification
reopened that row on an unresolved physical shader-module error-scope
regression. `LOC-009` is not promotable or complete and requires a new
production landing after internal Lua WebAssembly repair task
`019f9f34-a75f-7a11-a580-e9f54e610d93` on
`levi/fix-wasm-lua-coroutine-resume`.
Before such a fix lands, the F-ED orchestrator must obtain a fresh handshake
from the FL executor and rerun the unchanged 1,468-row pixel referee. The
renderer backend is outside the FL port boundary, but FL uses its pixels as a
merge referee, so an uncoordinated renderer landing would invalidate the
other executor's floor.

`F-ED-07` is historical-evidence work only. It may preserve the old WebGL2
observation; it has no linked product scenario. An explicitly scheduled
identical-input proof may requalify the same typed clip on supported WebGPU,
but it
authorizes no WebGL2 writer, fork, dependency, or fallback. The active browser
writer order therefore begins only with a qualified open `F-ED-10` closure;
landed `F-ED-06/11` and no-repair `F-ED-08` have no new writer. Overlap among
`browser.rs` and WebGPU resource owners serializes any surviving open slices
even though they are disjoint from FL.

### Work deferred from production

The following direct probes and source closures are safe, but production
translation waits for the named FL owner boundary:

| F-ED work | qualification now | production release condition |
|---|---|---|
| `F-ED-01/02` (`LOC-001/002/005`) | stable ViewModel/relation-owner differential | FL-D assigns the exact Artboard/DataBind/ViewModel closure after preceding waves land |
| `F-ED-05` (`LOC-007`) | ParametricPath dirt differential | the existing FL-E Path/Shape lock and integration rule in this map |
| runtime side of `F-ED-09` | text/bind/shaping/measurement stage localization | no edits to `text.rs`, `draw.rs`, `artboard.rs`, or another reserved runtime owner; a proven Scene-only facade may instead use the safe API lane |
| runtime side of `F-ED-10/11` | feather/GPU-canvas record and backend localization | a renderer-only result may use the landing-handshake lane; any runtime result waits for its FL owner |
| ABI/header/C-API repair | local ABI evidence | separate scope review after a current artifact proves a surviving ABI defect |

If qualification first diverges inside a reserved module, the F-ED
orchestrator records the exact row and gives it to the FL executor; it does not
open a second writer. After each FL landing, the executors exchange the merged
SHA, remaining reservation, displaced mechanisms, and green-floor record
before this table is revised.

### Non-conflicting execution order

The safe next queue is therefore:

1. retain checkpoint-7ca reconciliation PR #63 control-plane integrity at
   exact runtime main `fe0a0a07db302ce2f0282a2d919ea249e83144e5`;
2. serialize the evidence-only closures in binding order: `LOC-006`,
   `LOC-014`, `LOC-011`, `RT-ED-003`, then `LOC-019`;
3. keep `LOC-009` outside that shared tracking line while its new production
   repair waits on internal Lua WebAssembly repair task
   `019f9f34-a75f-7a11-a580-e9f54e610d93` and branch
   `levi/fix-wasm-lua-coroutine-resume`;
4. fan out read-only direct qualifications; treat `RT-ED-004` only as
   historical WebGL2 evidence unless an explicit WebGPU-only requalification
   is scheduled;
5. leave `F-ED-04` production exclusively with Runtime Fix and prepare only
   Defects Fix's independent post-landing verification; activate a qualified
   renderer-only `F-ED-10` closure only if it survives the atlas and
   coordinator lease checks;
6. perform record normalization and local ABI 1.6 qualification;
7. keep every runtime-owner result as evidence until the corresponding FL
   executor releases or absorbs its owner family;
8. re-handshake after the Runtime Fix stable-Apple repair, after the `FL-B`
   safety/API decision, after every later FL landing, and before any
   renderer-only landing.

## The governing decision

Use one identical stimulus at three layers:

1. direct pinned-C++ probe/reference;
2. direct Rust runtime or renderer probe;
3. Editor Next product reproduction.

Then classify mechanically:

| evidence | classification | permitted response |
|---|---|---|
| C++ passes; Rust direct fails; the corresponding Rust lifecycle is absent | `TRACKED-GAP` / missing port | port the complete C++ owner family |
| C++ passes; Rust direct fails; Rust uses copies, polling, replay, or mutation-gated compensation | structural mistranslation | replace the divergent Rust owner family and delete the displaced mechanism |
| C++ passes; Rust direct fails; the owner family is otherwise faithful | local translation defect | repair the exact guard, callback, order, or setter with C++ citations |
| C++ and Rust direct pass; Editor fails | editor integration defect | change the Editor adapter or call sequence; leave runtime untouched |
| low-level behavior works but typed/public callers cannot express it | A-row / API-surface gap | add a thin typed facade that emits the exact underlying record |
| no like-for-like observation channel exists | V-row / verification gap | build the differential before source changes |
| pinned C++ lacks it but newer C++ has it | upstream drift | use the Phase-S inventory and approval process |
| neither pinned nor current C++ has it | additive product feature | stop for user classification; do not call it a parity port |
| failure exists only in an old or malformed artifact | unqualified evidence | publish/correct the artifact and rerun before source work |

This preserves the closeout rule: faithful code gets a source-corresponding
repair; divergent code gets a complete owner-family replacement. A screenshot
symptom is never sufficient authority for an implementation.

### Atlas state machine

Classification and execution state are separate fields. Atlas schema
`nuxie.editor-next.runtime-defect-atlas/v2` and its checker permit only these
transitions:

```text
reported
  → intake-needs-evidence → reproduced
  → reproduced
  → qualified | stale-oracle | retracted

qualified
  → mapped → executor-green → orchestrator-verified → handoff-ready
  → handoff-ready                         # editor/artifact owner only; no runtime edit

executor-green → regression-reopened → qualified
                                          # failed independent current-path verification

handoff-ready → closed
              → editor-consumed → closed
stale-oracle | retracted → closed
```

- A separate `owner_class` is exactly one of `runtime`, `api`, `renderer`,
  `editor`, or `artifact`.
- A qualified runtime row may carry `TRACKED-GAP`, `DIVERGENT`, or
  `local-translation-defect`.
- A qualified API row may carry an A-row.
- A qualified renderer row must cite the renderer provenance record. Structural
  renderer/runtime classifications require a C++ correspondence. A
  `local-translation-defect` may instead cite `AF-10` only when the first
  divergence is confined to an existing foreign platform binding that pinned
  C++ cannot execute; the normative platform contract then governs that narrow
  adapter, not any product or renderer behavior above it.
- A qualified editor row is transferred to the Editor owner and cannot authorize
  runtime edits.
- A qualified artifact row names the repository and release prerequisite; it is
  not closed until the corrected artifact is green, a current failure is
  promoted to another qualified class, or the user approves an exception.
- `stale-oracle` and `retracted` retain their historical evidence and reason.
- A V-row remains `reproduced` until the missing observation channel exists.
- Only the independent orchestrator may move an executor result past
  `executor-green`.
- Independent current-path verification may move `executor-green` to
  `regression-reopened`. The historical executor result is retained separately,
  while current executor and orchestrator gates fail closed. A fresh
  qualification, mapping, production landing, executor result, and independent
  verification are required before promotion. The current merge and optional
  downstream-consumption revisions remain pending while reopened; a fresh
  executor-green event must record a full merge SHA distinct from the
  historical landing and command/evidence distinct from the historical
  executor record.
- Only an `editor` or `artifact` owner may move directly from `qualified` to
  `handoff-ready`; runtime, API, and renderer rows must traverse the complete
  mapped/executor/orchestrator path.
- `intake-needs-evidence` preserves an incomplete committed Editor record
  without initiating chatty clarification or production work.
- `handoff-ready` records an exact merged commit or corrected artifact plus
  downstream notification. Editor consumption may advance the optional
  `editor-consumed` state, but is not required before `closed`; immutable
  external publication remains a separately approved action/evidence field.

Every adaptation cites an existing AF/RF/FLR rule. A new adaptation rule is
adjudicated and landed before implementation; it is never inferred from a
green test.

## Methodology provenance

The execution shape combines the project's binding `PORTING.md`/FL rules with
the two migration methods previously reviewed:

- [Bun's Zig-to-Rust rewrite](https://bun.com/blog/bun-in-rust): write the
  porting guide first, run a representative file trial, translate
  mechanically with minimal behavior change, preserve the same test suite,
  and use independent reviewers plus a fixer;
- [Anthropic's dynamic workflows](https://claude.com/blog/introducing-dynamic-workflows-in-claude-code):
  split implementer, reviewer, and orchestrator roles and use parallel
  subagents only where their outputs can be independently verified.

For this already-green runtime, that means broad parallel read-only
qualification followed by disjoint complete owner-family writers. It does not
mean deliberately breaking the shared runtime tree or using compiler errors as
the only queue when exact C++ and pixel oracles already exist.

## Preliminary defect atlas

These are source-backed preliminary dispositions. `proven` here means the
layer has been localized from source; it does not mean the implementation is
authorized or closed. Every open row still receives a current-pin direct
fixture in `F-ED-00`.

| ID | preliminary disposition | owner or next proof |
|---|---|---|
| `RT-ED-001` | closed stale-oracle observation | focused current-pin `data_viz_demo` is exact; no further source work |
| `RT-ED-002` | closed stale-oracle observation | focused current-pin `db_health_tracker` is exact; no further source work |
| `RT-ED-003` | landed browser presentation/API repair, not runtime traversal; independent promotion pending | PR #55 splits ordinary WebGPU presentation from explicit readback at merge `e72323c8`, consumed by Editor checkpoint `7ca11e33` |
| `RT-ED-004` | historical WebGL2 capability evidence; current WebGPU parity remains unqualified | preserve the old evidence; any current claim requires a direct current-pin typed rounded-clip WebGPU oracle, with no WebGL2 implementation or fork |
| `RT-ED-005` | landed high-level generic number/color authoring repair; independent promotion pending | PR #49 merge `08286481` is consumed by Editor; P09-C01 remains separately Partial on FL-E layout/TextStyle ownership |
| `RT-ED-006` | retracted | retain tombstone only; no source work |
| `RT-ED-007` | proven high-level authoring-surface generalization gap assigned to Runtime Fix | Defects Fix preserves the nested `ViewModelNumberSource` diagnosis/tests and independently verifies after the Runtime Fix landing; no duplicate Scene writer |
| `LOC-001` | high-confidence structural Scene ownership mistranslation | type-specific remount carry versus one stable retained ViewModel instance |
| `LOC-002` | committed Editor evidence isolates rematerialization losing nested ViewModel state | requalify the historical f4bb C++ citations at d788, then route the exact Scene/FL owner collision |
| `LOC-003` | closed unlinked additive product feature | pinned C++ has no timed-hold primitive; the user decision authorizes no runtime port |
| `LOC-004` | resolved editor-owned | no runtime work |
| `LOC-005` | committed Editor evidence isolates independent live cells for one authored shared instance; runtime ownership remains a candidate | run the identical d788 application-owned instance differential before assigning a writer |
| `LOC-006` | closed no-repair stale characterization | exact committed provenance plus the independent no-hover rerun prove the alleged retained-pixel defect was gesture contamination; the legal reproduced/stale-oracle/closed path is complete |
| `LOC-007` | committed Editor evidence plus historical C++ source identifies a missing dirt chain | requalify d788 ParametricPath width/height through Path, Shape, and PathComposer, then track the sole FL-E owner landing |
| `LOC-008` | candidate public API-surface gap | establish the C++ ownership contract, then expose the exact runtime path only if runtime owns it |
| `LOC-009` | historical structural WebGPU consumer repair with a confirmed real-GPU shader-module validation regression | preserve PR #54 / `7f1450dc` as history; new production repair is active but waits on Lua WebAssembly task `019f9f34-a75f-7a11-a580-e9f54e610d93` / `levi/fix-wasm-lua-coroutine-resume` |
| `LOC-011` | real product symptom, owner unproven | inspect authored text, live VM value, post-bind target, shaped runs, then pixels |
| `LOC-012` | current renderer differential | compare the supported Rust WebGPU result with pinned C++ only after text/feather rows are separated |
| `LOC-013` | text/font-pipeline candidate, exact stage unproven | same font bytes, axes, glyph IDs, advances, and outlines through C++ and Rust |
| `LOC-014` | high-confidence mechanism divergence; symptom attribution still needs the oracle | compare Rust feather image plan with C++ Feather/ShapePaint/renderer-LUT ownership; do not tune constants |
| `LOC-015` | stale ABI 1.5 artifact evidence | publish ABI 1.6, pin URL/checksum, rerun |
| `LOC-016` | source implementation already present; publication gap | publish ABI 1.6 typed animation selection and integrate it |
| `LOC-017` | invalid old native capture | rerun ABI 1.6 with typed player/time and production host composition |
| `LOC-018` | evidence/localization gap | prove normalized record and object-order identity before renderer attribution |
| `LOC-019` | landed and consumed BrowserWebGpu nullable-error translation repair; independent promotion pending | PR #51 merge `ef9dcedd` is in consumed runtime `e72323c8`; unchanged P14-C06 passes 17/17 on required WebGPU |

There is no `LOC-010` in the source artifacts. `F-ED-00` records an explicit
tombstone in a separate `reserved_ids` table so future automation does not
interpret the gap as a dropped row. It is not a twenty-sixth defect row.

## Strong source correspondences

The first audit already found several cases where the correct shape is clear:

### Stable ViewModel ownership — `LOC-001`, `LOC-005`, then `LOC-002`

Rust `Scene` currently carries only number/string/boolean values across a
reconstruction and reapplies those copied values to a new mount. Other typed
setters mutate that disposable mount. Pinned C++ instead retains an
application-owned `rcp<ViewModelInstance>` and repoints attached containers
when it is bound:

- `src/artboard.cpp:2655-2692`;
- `src/viewmodel/viewmodel_instance.cpp:385-416`;
- `src/data_bind/data_bind_container.cpp:86-154`.

The Rust low-level runtime already has a shareable
`RuntimeOwnedViewModelHandle`. The candidate structural slice is therefore a
stable Scene-level handle per authored instance/default identity plus
C++-ordered rebind, not one global instance and not an extension of the carry
enum with five more value kinds.

### Parametric path dirt — `LOC-007`

The Rust Artboard callback dispatch handles many path/transform/paint changes
but lacks the ParametricPath width/height/origin path-dirt callbacks. Pinned
C++ directly propagates that mutation through:

- `src/shapes/parametric_path.cpp:35-66`;
- `src/shapes/path.cpp:327-372`;
- `src/shapes/shape.cpp:99-108`;
- `src/shapes/path_composer.cpp:19-116`;
- `src/shapes/paint/shape_paint_path.cpp:13-75`.

This is a source-corresponding missing callback/dirt edge, not a reason to
reset renderer resources or add a scene cache.

### Generic property binding — `RT-ED-005`

The historical typed Rust authoring facade hardcoded a world-opacity target.
Pinned C++ applies number/color context values through a generic property key:

- `src/data_bind/context/context_value_number.cpp:11-37`;
- `src/data_bind/context/context_value_color.cpp:11-20`.

The low-level Rust importer/runtime already understood generic property-key
records. PR #49 final head
`f0bd914fbac1fd4cf82814216f2ddc88c3e32083` ports the typed authoring
representation, validation, and exact export/import round trip; merge
`08286481b4e7420768f625f901a944f313b84903` has the same tree. Clean Editor
checkpoint `4da896beb5ec6815f6b01a2433875274a321d06c` consumes it through runtime
`e72323c808b91d706ba3b745396beaca7accd69a`. The generic number/color paint
consumer is therefore landed and consumed; broader ordinary layout and
TextStyle dirt/reflow remain a separate FL-E dependency.

### Nested transition duration — `RT-ED-007`

The typed Rust facade accepts a root `ViewModelNumberId` and exports a
two-element path. The low-level Rust bindable path resolver already walks an
arbitrary nested number path. Pinned C++ has the canonical nested-duration
fixture in:

- `tests/unit_tests/runtime/state_machine_test.cpp:719-748`;
- `transition_duration_bind_nested.riv`.

The exported authoring seam remains `crates/nuxie/src/scene.rs`, but the
recorded owner path crosses Runtime Fix's active
`crates/nuxie-runtime/src/state_machine.rs` and
`crates/nuxie-runtime/src/state_machine/bindables.rs` work. Runtime Fix owns
that complete production path. Defects Fix keeps its Scene candidate
quarantined and owns only unchanged set → fire → `advance(0)` verification
after the exact Runtime Fix landing.

### Browser presentation — `RT-ED-003`

Historically, the browser wrapper made `finish` mean both presentation and an
RGBA result. The retired backend took a screenshot on finish, while the old
WebGPU path mapped a texture and copied the returned bytes through
`ImageData`. The source baseline is
`bc139955c7e2d30d9cf611dd14c24606fd13520a`. PR #55 final head
`a1c56b5a80c88db4f6cee6550795b6e242394c46` separates those operations; merge
`e72323c808b91d706ba3b745396beaca7accd69a` has the same tree. On clean
Editor checkpoint `7ca11e331a57cb3ea574848f8e93eb108878c40b`, ordinary WebGPU presentation
records `getCurrentTexture=1`, `mapAsyncRead=0`, and `putImageData=0`;
explicit readback records `getCurrentTexture=0` and `mapAsyncRead=1`.

This mirrors C++'s ordinary-present versus explicit-snapshot lifecycle
conceptually, but the browser API is additive Nuxie infrastructure rather than
a C++ runtime file port. The product-host proof and unchanged normal-timeout
device-frame drag gate pass; the atlas row remains reported only until
independent orchestrator promotion.

### Historical backend evidence and current WebGPU oracles — `RT-ED-004`, `LOC-014`

The retired backend's nonrectangular clip allocated full-frame content and
mask images, then composited with `DestinationIn`; its feather path used a
locally designed Gaussian image plan. Those dated observations remain
renderer-backend provenance, never a writer, fork, fallback, or repeatable
qualification target. `LOC-014` is current only insofar as the same typed
input remains a candidate on required WebGPU. `LOC-006` is separately closed
in the committed inbox as gesture-contaminated stale characterization and has
no renderer writer. Any current `LOC-014` claim must
be qualified as follows:

1. prove the exact current-pin C++ reference behavior on the same typed input;
2. compare the supported Rust WebGPU path with the same normalized stimulus
   and stamp the exact backend/reference provenance;
3. if there is no corresponding C++ capability, stop for an additive-backend
   decision instead of claiming a parity port; never use historical WebGL2 as
   a production repair target.

## Artifact corrections required before implementation

`F-ED-00` corrects or explicitly versions these contradictions:

1. The handoff mixes C++ `f4bb3025` and `d788e8ec`; every source citation and
   fixture must be revalidated at `d788e8ec`.
2. The defect document introduction calls the ledger confirmed, although most
   rows are candidates, one is editor-owned, and one is retracted.
3. `RT-ED-004` lacks the direct current-pin C++ clip proof required by its
   claimed classification.
4. `RT-ED-005` cites `outline-visual-consistency.spec.ts`, but linked
   `P09-C01` does not clearly run that focused reproduction.
5. P16 contains a stale current-runtime SHA.
6. P18 alternates between an eight- and nine-entry native corpus.
7. `LOC-012` reports both 5,790 and 5,616 exact pixel differences without a
   capture/mode label.
8. `LOC-013` reports both 10,199 and 11,019 baseline differences without a
   capture/mode label.
9. `linkedRerun` is used inconsistently, especially for `P15-C01`.
10. The “42-case” GPU matrix reports 45 results because three auxiliary tests
    are included; the label must say so.
11. `LOC-015/016/017` must remain outside source-port fan-out until ABI 1.6 is
    published and consumed.
12. The absent `LOC-010` needs a tombstone.

No executor may “pick the likely count” or silently normalize one of these.

## F-ED execution map

Ticket numbers are stable owner-family identifiers, not permission to bypass
the live status/DAG scheduler. The original blocker order below is retained as
historical planning provenance and does not authorize or schedule production
work after Q0:

1. `RT-ED-004`;
2. `RT-ED-003`;
3. `RT-ED-005`;
4. `RT-ED-007`.

Disjoint file lanes may advance concurrently, but a candidate row may not move
ahead of one of those blockers when they contend for the same writer or module.

The named `LOC-*` branches below are conditional landing destinations, not
pre-authorized fixes. A file-disjoint lane receives production code only
after `F-ED-00A` promotes its state to `qualified` with the matching owner
class. A runtime-owner lane also waits for `F-ED-00B`. A source review may
raise confidence; it cannot substitute for that promotion.

### `F-ED-00` — Oracle normalization and executable atlas

`F-ED-00A` is the first commit and precedes every production change.

While the live FL writer lease is active, execute this ticket in two
machine-checked parts:

- `F-ED-00A` creates only the new F-ED-owned paths below and may authorize a
  proved file-disjoint API, browser-presentation, renderer-only, Editor, or
  artifact lane under the additional locks in this map;
- the owning FL executor applies every queued runtime cross-link and
  verification-state update to the shared FL ledgers while it holds their
  writer lease;
- `F-ED-00B` verifies those links at the FL executor's merged/released SHA,
  wires the standalone checker into the shared workspace evidence target in a
  separately coordinated harness commit, and records the integration result
  in the F-ED atlas/status.

No runtime-owner production translation may begin between `00A` and `00B`.
The split is writer serialization only; both parts are required to close
`F-ED-00`.

`F-ED-00A` may create only:

- `docs/editor-next-runtime-defect-atlas.toml`;
- `docs/editor-next-runtime-defect-status.md`;
- `docs/editor-next-runtime-defect-corrections.toml`;
- `tools/editor-next-runtime-defects/**`, containing a standalone fail-closed
  checker, its checker tests, and standalone pinned-C++ probe sources/build
  scripts;
- new F-ED-specific Rust integration-test and fixture files whose exact paths
  are first declared in the atlas.

`00A` invokes its checker directly. It does not edit the root Makefile,
workspace/Cargo manifests, existing test files, `tools/cpp-probe/**`, any
reserved runtime/graph source, or any shared FL ledger. `00B` performs the
workspace-target wiring only after the FL executor has landed the queued
ledger links. Every open row still receives a current-pin direct fixture/probe
ID before it can be promoted.

Every atlas row must name:

- the exact Editor reproduction and artifact hashes;
- the original localization Rust SHA, Editor's last consumed runtime SHA,
  current investigation HEAD, merged repair SHA, consumed runtime SHA, and
  consumed superproject SHA as distinct fields;
- minimized direct Rust stimulus;
- minimized pinned-C++ stimulus or an explicit “no C++ equivalent” result,
  while preserving every historical pin/result rather than rewriting it;
- C++ direct result, Rust direct result, and Editor result;
- each direct-result field accepts only an evidence record or
  `not-applicable` with a mandatory reason; closed/retracted/artifact-only
  rows never receive synthetic probes;
- for renderer work, the complete renderer/Dawn/backend/mode/features/surface
  provenance and stamped reference executable;
- source files, state-bearing members, and lifecycle phases;
- Rust owner/module and any displaced mechanism;
- classification;
- implementation family and dependencies;
- exact `TOUCH` and `DON'T TOUCH` sets checked against all active worktrees and
  the live FL file/member ownership;
- target tests, product ledger children, and required floors;
- executor and independent-orchestrator verification state.

The atlas is an index, not a parallel source of truth. Every qualified C++
runtime file/member must link to and update:

- `file-correspondence-manifest.toml`;
- overlapping `docs/runtime-frame-loop-ownership.toml` rows;
- `docs/runtime-frame-loop-gaps.toml`;
- or, for C++ renderer source outside those ledgers, a dedicated
  machine-checked renderer file/member correspondence ledger.

Closed mode requires zero pending/compensation rows, adaptation-rule citations,
decision-row citations, and orchestrator-only verification promotion in the
owning ledger.

`RT-ED-001/002` are exact at the current pin and closed as stale observations.
`LOC-003` is closed by the explicit user decision that an unlinked long-press
feature with no pinned-C++ primitive is not runtime parity work. `LOC-004` is
recorded closed, `RT-ED-006` retracted, and `LOC-010` tombstoned. No closed
row is reopened from prose alone.

This runtime commit records the discrepancies in its own atlas. Corrections to
the three source Editor artifacts are a separate Editor-owned commit: the
runtime side supplies a versioned correction manifest, and the Editor cutover
applies it without rewriting historical observations.

### `F-ED-01` — Stable Scene ViewModel owner

Qualification targets: `LOC-001`, `LOC-005`; `LOC-002` is an adversarial
rebind case, not yet a proven semantic dependency.

If the direct current-pin differential confirms the preliminary structural
classification, replace type-specific cross-remount carry with one retained
handle per authored ViewModelInstance/default identity, shared by every
artboard that references that identity. A Scene may own multiple such
identities. Translate the complete bind, attached-container rebind, structural
edit, clone, source removal, and drop lifecycle. Delete the displaced
carry/replay mechanism in the same landing.

Acceptance must cover:

- number, string, boolean, color, image, enum, trigger, and list-index values;
- equal-value return semantics;
- an unrelated Scene edit preserving the exact live instance;
- one instance bound to two artboards with writes visible through both;
- two distinct authored/default identities remaining pointer- and
  value-independent;
- detach/rebind/drop and deep-copy cases matching pinned C++.

This is one structural slice. Do not land per-type carry extensions.
`F-ED-01` may close with `LOC-002` still red only if the atlas proves that its
first divergence belongs to a separate C++ owner family and maps that complete
family to `F-ED-02`.

### `F-ED-02` — Conditional separate current-source relation owner

Qualification target: `LOC-002`.

Run the exact relation fixture independently, and rerun it after `F-ED-01` if
that slice is qualified. This is a suspected collapse, not a hard dependency.
If the residual is proved to belong to a separate DataBind-container owner
family, map and port that family's complete registration, bind/rebind,
current-source evaluation, clone, detach, and drop lifecycle. If it belongs
inside the Scene/ViewModel bind lifecycle, it closes in `F-ED-01` instead.
Do not add a post-remount boolean replay.

Acceptance proves retained and fresh mounts produce the same relation and
pixels from one current source value.

### `F-ED-03` — Generic visual-property binding

Target: `RT-ED-005`.

PR #49 generalizes typed number/color binding to carry an owner/property
target with type validation and exact `propertyKey` export. Import/export
round-trips the same record, and the runtime uses the existing generic
DataBind execution path.

Before extending downstream P09 consumption beyond that landed primitive, the
Editor owner must normalize the exact remaining style leaves into a reviewed
target map, and the runtime atlas must ratchet it:

| semantic leaf | required exact mapping proof |
|---|---|
| `borderWidth` | every numeric target record, including per-edge or stroke-thickness expansion, its concrete C++/Rust setter, and path/paint/layout dirt closure |
| `borderColor` | every color target record for the border/stroke paint, its concrete setter, and paint invalidation closure |
| `backgroundColor` | every fill/background color target record, its concrete setter, and fill/paint invalidation closure |
| `padding` | the complete one-to-many edge/unit record expansion and each layout/bounds/hit-geometry invalidation callback |

Generic property-key dispatch proves only that a value can reach a generated
setter. It does not prove that each concrete target has the C++-matching
paint/layout/dirt callback; `LOC-007` is the counterexample. Editor Next keeps
ownership of semantic style normalization. The runtime slice owns only the
typed exact target representation and execution.

Acceptance includes:

- number and color targets beyond opacity;
- converter chains;
- two-way direction semantics where the property supports them;
- exact cold/retained record identity;
- the focused occurrence-local generic number/color paint consumer.

Those authoring and execution checks are green on PR #49, whose final head
`f0bd914fbac1fd4cf82814216f2ddc88c3e32083` and merge
`08286481b4e7420768f625f901a944f313b84903` have the same tree. Editor
checkpoint `4da896beb5ec6815f6b01a2433875274a321d06c` consumes the repair through
runtime `e72323c808b91d706ba3b745396beaca7accd69a`. A downstream rerun of the
full linked `P09-C01` child is not required to close this verified landed
repair: that child remains separately Partial on ordinary LayoutComponent
padding and TextStyle font-size/line-height authoring plus FL-E dirt/reflow.
The atlas row remains reported solely pending independent promotion.

### `F-ED-04` — Nested transition-duration source

Target: `RT-ED-007`.

Runtime Fix owns the complete production change across the exported
`crates/nuxie/src/scene.rs` seam and its active
`crates/nuxie-runtime/src/state_machine.rs` /
`crates/nuxie-runtime/src/state_machine/bindables.rs` owner path. Defects Fix
must not author or duplicate that Scene/runtime change.

Defects Fix's only post-landing action is to run the unchanged set → fire →
`advance(0)` runtime-duration verification and record that independent result.
All authoring, invalid-path, round-trip, and linked-product executor coverage
belongs to Runtime Fix.

### `F-ED-05` — ParametricPath dirt callbacks

Qualification target: `LOC-007`.

If the direct differential confirms the missing-callback classification, port
width/height/origin setter callbacks and their complete
Path → PathComposer/Shape → retained paint-path invalidation owner closure.
Delete the renderer-resource-reset control from the expected production path;
retain it only as a negative-control test if useful.

This slice touches the active frame-loop owner substrate. Its direct probe may
land during FL work, but production implementation waits until the FL-E file
rows for `src/shapes/parametric_path.cpp`, `src/shapes/path.cpp`,
`src/shapes/path_composer.cpp`, and `src/shapes/shape.cpp`, plus their imported
runtime-drawing owner rows, have an executor-green integration and the active
FL executor assigns this defect to that same owner closure. Final promotion
still waits for independent orchestrator verification. It must not be
developed concurrently against unmerged edits in `artboard.rs`, `draw.rs`, or
an adjacent Path/Shape ownership module.

Acceptance records property value, dirt propagation, geometry, draw
operations, and pixels at multiple samples in both C++ and Rust.

### `F-ED-06` — Browser presentation without mandatory readback

Target: `RT-ED-003`.

The historical browser wrapper conflated presentation with full CPU RGBA
capture. PR #55 final head
`a1c56b5a80c88db4f6cee6550795b6e242394c46` now splits ordinary WebGPU
presentation/flush from explicit snapshot/readback; rebase merge
`e72323c808b91d706ba3b745396beaca7accd69a` has the same tree. Ordinary
Editor frames do not map a GPU buffer, allocate a full RGBA result, or copy
through `ImageData`; explicit captures retain exact pixels and deterministic
errors.

Recorded acceptance includes:

- initialization, resize, asynchronous completion, repeated capture, setup
  failure, device/context loss, and teardown of any presentation/readback
  resources;
- an instrumented zero-readback ordinary WebGPU frame test;
- an exact explicit-snapshot test for the supported WebGPU backend;
- `scripts/audit-editor-readback-boundary.mjs`;
- `device-frame-bezel-drag.spec.ts`;
- an ordinary ProductHost proof with `getCurrentTexture=1`,
  `mapAsyncRead=0`, and `putImageData=0`, plus explicit readback with
  `getCurrentTexture=0` and `mapAsyncRead=1`;
- the unchanged normal-timeout device-frame drag result 1/1;
- the 1,468-row renderer floor.

Clean Editor checkpoint
`4da896beb5ec6815f6b01a2433875274a321d06c` consumes the merge. `P19-C03` is
complete; `P04-C01` records the same runtime repair but remains Partial only on
separate Editor-owned work. Neither downstream state gates RT-ED-003 repair
closure or handoff. The atlas row remains reported pending independent
orchestrator promotion.

### `F-ED-07` — Historical rounded WebGL2 clip evidence

Target: `RT-ED-004`.

This ticket owns no production WebGL2 implementation. Preserve the original
402×874, radius-57 observation as historical evidence. WebGL2 is retired, so
this row authorizes no backend restoration, dependency fork, fallback, or
special-case repair.

If a current disposition is scheduled, run the exact typed clip through
pinned C++ and the supported Rust WebGPU path with the same transform, fill
rule, surface size, and pixel oracle. Stamp the exact renderer revision, Dawn
revision, feature flags, surface, mode, and reference executable.

- If current WebGPU matches pinned C++, retain the WebGL2 observation as
  historical backend evidence; no runtime repair exists.
- If current WebGPU differs, map that new/current WebGPU parity defect to its
  complete C++ owner lifecycle before assigning a writer.
- If pinned C++ has no corresponding capability, stop for a user decision;
  that is additive functionality, not an authorized parity port.

Any current lifecycle proof covers construction, resize, nested clip
allocation, composite order, repeated-frame reuse, allocation failure,
device/surface loss and recovery, and teardown on WebGPU. Acceptance requires
the focused current-pin clip differential and unchanged renderer floors, with
no resource-budget or error-path weakening.

The landed Editor checkpoint has no formal or candidate child for
`RT-ED-004`. Historical product consumption therefore does not gate this
evidence row or any independently landed browser repair.

### `F-ED-08` — Closed conditional-visibility stale report

Closed row: `LOC-006`.

Independent verification completed the legal
`reported -> reproduced -> stale-oracle -> closed` path. At exact Editor
checkpoint `7ca11e331a57cb3ea574848f8e93eb108878c40b` and runtime
`e72323c808b91d706ba3b745396beaca7accd69a`, the focused Chromium diagnostic
passed 1/1. Its exact five-frame sequence is `post-write-no-hover-1` through
`post-write-no-hover-4`, followed by `post-capture-no-hover-5`; all remain at
draw count 30 with identical timing-stripped frame hashes, no red compositor
residue, zero probe errors, and no rejected maps, device loss, or uncaptured
errors. The old 34-draw state returns only after the deliberately later
`hoverAt` plus `clearHover` gesture. The fresh 70,947-byte run log has SHA-256
`1a9b91fcb8a64296a4c464ad1848be839e9bc91da8d9dfdef337707a0a09f328`;
the committed 171,784-byte machine report has SHA-256
`f78b93d7575c3543e57de49bd73dce5783648b4c5a258328cfdd1f5eeb2652b5`.
This creates no runtime, renderer, browser, or Editor writer. The historical
retired-backend observation remains provenance only and creates no fork,
fallback, test target, or serialization edge.

### `F-ED-09` — Text semantics, variable fonts, and measurement

Qualification targets: `LOC-011`, `LOC-013`, `LOC-008`.

This is a shared evidence cluster, not a predeclared single writer. The rows
may localize to different owner families: Scene/import/binding initialization,
font instance/shaping/outline/raster, and a public measurement facade.
`F-ED-00` must produce exact file/member closures before writer assignment.

1. Empty text: compare authored record, instantiated ViewModel string,
   post-bind target, shaped runs, draw paths, and pixels. Source inspection
   does not support the claim that the renderer itself synthesizes `"Text"`.
2. Variable font: use identical font bytes, axis tags/values, glyph IDs,
   advances, and outlines. The first differing stage localizes the row; then
   the governing classification decides between an exact-site repair, a
   complete missing lifecycle port, or structural owner-family replacement.
3. Measurement: expose a typed read-only facade over the existing exact
   shaping/measurement path only if the direct disposition confirms that the
   runtime owns the contract and the missing public surface is the first gap.
   Do not add a DOM/editor approximation.

Every promoted owner family becomes a separate committed and floor-gated
slice. Rows may share a writer only after exact closure proves a real module
or lifecycle overlap.

### `F-ED-10` — Feather translation: pinned C++ versus supported WebGPU

Qualification targets: `LOC-014`, `LOC-012`.

Run the exact feather scenes through pinned C++ and the supported Rust WebGPU
path only. Compare normalized records, Feather/ShapePaint ownership, LUT and
composition resources, and final pixels. If the current WebGPU path is the
first divergence, classify the complete surrounding owner/resource lifecycle.
A faithful owner permits an exact-site repair; a missing/divergent lifecycle
requires the complete C++-corresponding Feather/ShapePaint/renderer ownership
and LUT/composition translation. Do not tune sigma, offsets, or tolerances
against screenshots.

The proof covers creation, resize, nested feather/composite ordering, repeated
frames, allocation/setup failure, context loss/reset, and teardown.

`LOC-012` may be qualified in parallel. Its final attribution must account for
the `F-ED-09` text result and this feather result; neither is assumed to be a
semantic prerequisite. Historical retired-backend counts are dated provenance
only and do not participate in the current oracle.

### `F-ED-11` — WebGPU setup and GPU-canvas qualification

Qualification targets: `LOC-019`, `LOC-009`. `LOC-019` is independently
localized and repaired. `LOC-009` preserves its independently localized
consumer repair as history, but a later independent real-GPU regression
reopened the row; neither row is an Editor workaround or a WebGL2 fallback.

`LOC-019` is no longer an adapter-selection or fallback hypothesis. At runtime
`95027109`, real Chrome creates the WebGPU device and executes a valid draw.
The first divergence is the clean error-scope result: WebGPU fulfills
`GPUDevice.popErrorScope()` with JavaScript `null`, while wasm-bindgen 0.2.126
uses undefined-only `JsOption` conversion and vendored wgpu passes that null
to `Error::from_js`. The faithful platform translation is local: recognize
fulfilled `null` as no error before converting a present `GpuError`, while
leaving the pre-existing rejected-promise path unchanged and preserving real
error conversion. This is the `AF-10` foreign-platform-binding case and stays
at Nuxie's existing vendored BrowserWebGpu compatibility boundary; it does
not add a dependency fork, downgrade wasm-bindgen, or restore WebGL2 fallback.

PR #51 final head
`22454fb58bc80d95174ca78d0c0d4d611b0d5a08` implements that translation;
rebase merge `ef9dcedd82265efc0184f4f59d5f6aaab0b56cd9` has the same tree. Clean
Editor checkpoint `4da896beb5ec6815f6b01a2433875274a321d06c` consumes it through runtime
`e72323c808b91d706ba3b745396beaca7accd69a`. The unchanged required-WebGPU
P14-C06 matrix passes 17/17. The atlas row remains executor-green pending
independent orchestrator verification; the recorded executor evidence is
local and does not relabel queued hosted Apple work as green.

`LOC-009` was a separate structural consumer mistranslation. The exact
compiler-produced one-UBO RSTB reached Rust before any device call with only
retired WebGL2 variants at the old producer checkpoint; its minimized target-1
GLSL path then failed in Rust's invented GLSL-to-WGSL translator. Pinned C++
WebGPU instead selects target 0, requires target 16, parses one authored WGSL
module with arbitrary logical/physical entry records in declaration order,
resolves omitted entries to the first stage declaration and named entries
from logical to physical names, and creates one shared module. The binding-map
sidecar remains authoritative for backend identity and visibility.

The repaired Rust lifecycle now matches that contract end to end:

- `ShaderAsset` coalesces target descriptors with C++ last-wins semantics,
  validates the final descriptor for each target, and preserves target 0 plus
  target 16 without interpreting retired target 1;
- scripting accepts a bare `Shader` or
  `{ module = Shader, entryPoint = string? }`, retains the exact selected
  logical/physical pair, and uses declaration order for omitted or empty entry
  names;
- the renderer parses authored WGSL once, retains Naga `ModuleInfo`, rejects a
  target-16 visibility mask that underdeclares actual entry-point usage, allows
  the broader visibility C++/WebGPU permits, creates one shared module, and
  submits the selected physical vertex and fragment entries directly;
- the target-1 GLSL cross-translator, split-stage representation, discarded
  sidecars, `wgpu` GLSL feature, and `pp-rs`/`unicode-xid` closure are deleted.

The row first reached executor-green at
`22e4900243ee92a436afc1609f456525e8312352`. The producer is pinned at Editor
checkpoint `f9d798dd3b1f9b2dfdbeb74dcdf4485aae4519f6`, whose exact inner RSTB is
SHA-256
`546517d0dc9fbdaf9585f3daa6e440628e62292d7cb8aa7253fd3019aa35713d`.
Independent re-review was clean with no P0/P1/P2 findings. The corrective
consumer/lookup ownership PR #54 head
`1fc9ac4f5926b7a0799211cd66cbf9231a6843fd` rebase-merged at exact runtime
SHA `7f1450dc22ca7370eac9dc9f422351c2dfcc07ee`.

Clean Editor checkpoint
`4da896beb5ec6815f6b01a2433875274a321d06c` consumes descendant runtime
`e72323c808b91d706ba3b745396beaca7accd69a`; the unchanged required-WebGPU
`P14-C01` command passed 4/4 in 1.6 minutes. Its retained console SHA-256 is
`c2324d04cf1baa6ac024ae3b4f0607ca3a4ad64ecace7ac67e612707480527f0`,
and its retained complete report/results archive SHA-256 is
`515f53c5710c8069bf60f4c64c64568c219ee3cfb242fbe8e59846b1c0f96bd3`.
Those immutable merge and consumption records replace the stale
awaiting-merge/awaiting-rerun language and remain historical evidence.
Independent later real-GPU verification found an unresolved physical
shader-module error-scope regression, so the row is now
`regression-reopened`, not promotable or complete. A new production landing is
required after internal Lua WebAssembly repair task
`019f9f34-a75f-7a11-a580-e9f54e610d93` on
`levi/fix-wasm-lua-coroutine-resume`.

The current negative proof is local/non-hosted at unfixed main
`fe0a0a07db302ce2f0282a2d919ea249e83144e5` (tree `4512e0d7`) on Apple M5
Max/Metal. The temporary, uncommitted `loc009_real_gpu_fail_closed_probe`
in source checkout `/private/tmp/loc009-base-repro.AiNb0d` records
`max_bind_groups=4`, authors group 4 binding 0, expects `Err`/no shader, but
receives `Ok` plus an uncaught `Device::create_shader_module` validation
error. Its 2,129-byte raw log has SHA-256
`93ecaae76c5bfd6252e5fb919087215a1c60a397dd5cfb9a8bc8bf64929b5611`.
PR #54's positive-path 7/7 and browser results remain valid historical evidence
but never exercised a device-rejected physical module.

The canonical browser path separately remains a valid crash at
`luaG_indexerror` / `luaD_throw`. Its cause is under investigation in the
focused Lua WebAssembly dependency task, which now uses short durable
diagnostic checkpoints after repeated system errors. LOC-009 is frozen outside
the shared tracking line and its reopened cycle has consumed no runtime or
superproject landing.

The two rows share a real-Chrome smoke harness but not a defect mechanism:
`LOC-019` owns nullable WebIDL error-scope decoding, while `LOC-009` owns RSTB
and authored-WGSL consumption.

### `F-ED-12` — Apple ABI 1.6 publication and requalification

Targets: `LOC-015`, `LOC-016`, `LOC-017`.

This is a three-owner relay, not one runtime slice:

1. the runtime repository builds and locally qualifies ABI 1.6 from an exact
   source SHA, producing the proposed version/channel/checksum;
2. **user release checkpoint:** confirm the exact immutable artifact,
   version, and publication channel before the external publish;
3. `nuxie-ios` updates the URL, checksum, ABI floor, and typed selector;
4. `nuxie-dev` consumes that SDK result and reruns the corrected native corpus
   with typed player selection, timestamp, and production host composition.

Only a failure that survives this rerun may return to a runtime owner lane.
No runtime source edit is authorized by ABI 1.5 evidence.

### `F-ED-13` — Legacy/Rust record normalization

Target: `LOC-018`.

Normalize the complete typed records, resource hashes, object order, time,
surface, and composition for old-editor and Rust output. If records differ,
the editor/lowering layer owns the first repair. Only identical records may
promote the residual to renderer parity.

### `F-ED-14` — Product-ledger closeout

This is a two-owner handoff:

1. the runtime owner supplies the merged/published runtime SHA, focused direct
   evidence, complete floor record, defect disposition, and expected remaining
   signatures;
2. the Editor cutover owner updates its runtime pin, consumes that exact SHA,
   reruns the product commands, and updates the Editor parity/defect artifacts
   with exact commands, hashes, and results.

The landed artifact snapshot has nine children with structured
`runtimeDependencies` or `runtimeDefects`, but that is not the complete product
rerun set. The remaining candidate matrix currently maps as follows:

| candidate | directly affected children in the artifact snapshot |
|---|---|
| `LOC-001` | `P13-C07` |
| `LOC-002` | `P09-C01` (structured links now own `P04-C11`, `P09-C03`, and `P09-C06`) |
| `LOC-003` | none; closed user decision authorizes no runtime implementation |
| `LOC-005` | none (`P09-C05` is now a structured runtime-defect link) |
| `LOC-006` | `P09-C04` (historical source linkage retained after the stale characterization closed; it authorizes no writer) |
| `LOC-007` | none (`P11-C12` is now a structured runtime-defect link) |
| `LOC-008` | `P08-C06` |
| `LOC-009` | `P14-C01` |
| `LOC-011` | `P08-C06` |
| `LOC-012` | `P19-C08` |
| `LOC-013` | `P08-C08` |
| `LOC-014` | `P08-C09` |
| `LOC-015` | `P18-C01`, `P18-C04`, `P18-C05`, `P18-C07` |
| `LOC-016` | `P18-C01`, `P18-C04` |
| `LOC-017` | `P18-C07` |
| `LOC-018` | `P04-C12`, `P07-C04` |
| `LOC-019` | `P14-C06` |

The candidate set contains 15 unique children. Unioning it with the nine
formal children yields 23 unique directly affected child IDs because
`P09-C01` overlaps. The current Editor checkpoint retains `RT-ED-004` only as
historical WebGL2 evidence and removes all five of its former formal child
links. It also keeps `P04-C12` linked only to candidate `LOC-018`. Broad
aggregate gates such as `P08-C01` and `P11-C01` rerun after their focused
children. The executable atlas owns the complete
defect → child → aggregate-command matrix.

Closure requires every one of the 25 IDs to be:

- fixed and independently verified;
- editor-owned and linked to its editor repair;
- artifact-only and requalified to one of: corrected artifact green, current
  failure promoted to a qualified owner class, or user-approved exception;
- explicitly retracted/tombstoned;
- or backed by a user-approved decision row.

No `LOC-*` candidate remains unqualified.

## Dependency types

Evidence, semantic ownership, and writer serialization are separate fields in
the atlas. A shared file is not proof of a semantic dependency.

### Evidence prerequisites

- `F-ED-00A` precedes every production change. Its qualification may authorize
  only the file-disjoint API, browser-presentation, renderer-only, Editor, and
  artifact lanes named by the live concurrency lease.
- completed `F-ED-00B`, and therefore full `F-ED-00`, precedes every
  runtime-owner production translation.
- `LOC-012` can be qualified immediately, but final attribution accounts for
  both the text and feather differential results.
- ABI 1.6 local qualification precedes the user publication checkpoint; SDK
  consumption precedes the corrected native product rerun.
- `F-ED-14` waits for every directly affected branch, not just the four
  current `RT-ED-*` blockers.

### Semantic prerequisites

- None are assumed between `LOC-001/002/005` until the direct owner closure is
  complete. `LOC-002` may collapse into `F-ED-01` or become a separate family.
- `LOC-006` is a no-repair stale characterization, semantically distinct from
  historical `RT-ED-004`; neither has a production writer.
- `LOC-009` is not blocked by `LOC-019`.
- `RT-ED-005` and `RT-ED-007` are independent API gaps: the former is landed
  and consumed pending promotion, while the latter is assigned to Runtime Fix;
  Defects Fix retains no production writer.

### Writer/file-lock order

- confirmed blocker priority governs when two rows contend for one module;
- `F-ED-05` shares the FL-E Path/Shape owner lock;
- browser WebGPU, text, and Scene rows serialize only after `F-ED-00` proves
  overlapping `TOUCH` sets or adjacent lifecycles; historical WebGL2 evidence
  has no production writer;
- no candidate lane is called safe before that proof.

## Parallel execution

Qualification fan-out is broader than implementation fan-out.

### Qualification wave

One orchestrator owns the atlas/status. Landing-provenance reviewers handle
`RT-ED-003`, `RT-ED-005`, and `LOC-019`; no implementation scout or writer is
assigned to those completed repairs. `LOC-009` stays outside the shared
tracking line and receives a new production writer only after the dedicated
Lua WebAssembly dependency lands. The orchestrator may dispatch three
read-only scouts for the remaining open rows:

1. Scene/ViewModel/DataBind/StateMachine:
   `LOC-001/002/005`, `RT-ED-007`, and `LOC-007`;
2. browser/renderer:
   historical-only `RT-ED-004` when explicitly requested, plus
   `LOC-012/014`; closed `LOC-006` is historical no-repair evidence only;
3. text/records:
   `LOC-008/011/013/018`.

The orchestrator handles `RT-ED-001/002`, closed/retracted/tombstone rows, and
the ABI 1.6 artifact lane.
Scouts return direct fixtures, current-pin source closures, and classifications
only. They do not edit production code.

### Writer wave

These are candidate writer lanes, not pre-approved worktree assignments.
`F-ED-00` must replace every illustrative lock below with exact checked
`TOUCH`/`DON'T TOUCH` sets and prove no collision with active worktrees or FL
ownership before activation.

| candidate lane | order if closure overlaps | expected area to prove |
|---|---|---|
| Scene/API | Runtime Fix exclusively owns open blocker `F-ED-04`; Defects Fix may independently verify its landing before qualified candidate work `F-ED-01/02`; landed `F-ED-03` has no writer | `scene.rs`, schema/export/import helpers, and any runtime handle seam |
| browser | qualified open `F-ED-10`; landed `F-ED-06` and the `LOC-019` half of `F-ED-11` have no writer; reopened `LOC-009` receives a new writer only after its Lua WebAssembly dependency lands; no-repair `F-ED-08` and historical-only `F-ED-07` have no writer | `browser.rs`, WebGPU GPU-canvas/scripting seams, and resource owners; no `webgl2.rs` writer |
| runtime dirt | qualified `F-ED-05` | waits for the exact FL-E Path/Shape owner lock |
| text-derived | qualified `F-ED-09` rows split by discovered owner | Scene/import/binding, `text.rs`, font parser/shaper, or renderer |
| renderer feather | qualified `F-ED-10` | exact renderer/backend owner closure |
| evidence/artifact | `F-ED-12/13` | no runtime source until qualification promotes a survivor |

Parallel read-only translations may inspect the same family. Parallel
production writers may not share a Rust module or adjacent lifecycle boundary.

For every multi-file structural family, either cite existing stress-tested
AF/RF/FLR rules that fully cover its idioms or run the established stress test
before production work. Renderer-backend families may not assume runtime RF
rules cover them:

1. one disposable rulebook-strict translation;
2. one disposable senior-engineer translation;
3. adjudicate every difference from pinned C++;
4. add genuinely new idiom mappings to `docs/PORTING.md`;
5. discard both translations;
6. start the real owner-family port.

## Per-slice landing contract

Every production slice:

1. begins with a deterministic direct failure;
2. enumerates and completely reads every pinned C++ header/source in the exact
   member/dependency closure; preliminary `:line+` citations become exact
   pin-bound ranges before implementation;
3. records construct, retain, dirty, update/advance, draw, clone/rebind, and
   drop, plus resource setup/resize/failure/loss/teardown where applicable;
4. names the exact Rust owner and displaced mechanism;
5. translates the complete owner family or repairs an already-faithful exact
   site;
6. deletes displaced compensation in the same landing;
7. adds lifecycle/order/identity tests;
8. receives independent ownership/architecture and spec/behavior reviews;
9. runs its targeted Editor reproduction and all applicable floors;
10. lands as one small commit tagged with its `F-ED-NN` identifier;
11. remains `pending-verification` until an independent orchestrator reruns
    the evidence.

An implementer may not use their own measurement to promote a row to closed.

## Gates

The last accepted clean FL baseline is commit `69e89b3c`: runtime 414/414,
`nuxie` 140/140, C++ probe 721/721, both command corpora 317/317 entries and
647/647 segments with zero failures, renderer 1,468/1,468, and the structural,
Apple, C API, lint, format, and size gates green. `F-ED-00` must rerun and
record the exact clean investigation-base commit/counts before production
work. The atlas checker stores that commit plus exact monotonic minima; prose
does not permit 414 to become 411 or 140 to become 139.

The named battery is:

```text
cargo test -p nuxie-runtime --lib
cargo test -p nuxie --lib
make cpp-probe
env -u CPP_CONFIG -u RUST_PROFILE make golden-compare
env -u CPP_CONFIG -u RUST_PROFILE make scripted-golden-compare
env -u CPP_CONFIG -u RUST_PROFILE make cpp-oracle-workspace-tests
make renderer-golden                     # every draw/renderer-facing slice
make capi-smoke
make apple-runtime-check
make runtime-frame-loop-port-check
make b6-audit-check
make lint-gate
cargo fmt --all -- --check
git diff --check
make size-report
```

The explicit probe-only result inside the armed workspace must remain 721/721.
Both SDK size variants remain below 9,437,184 bytes.

Exact baseline counts may rise when a slice adds tests; they may never fall.
No test, expected value, corpus entry, tolerance, resource ceiling, error
contract, provenance guard, or gate is weakened to admit a repair.

The nine children carrying structured JSON `runtimeDependencies` or
`runtimeDefects` in the landed Editor checkpoint are:

| child | structured runtime links |
|---|---|
| `P04-C01` | `RT-ED-003` |
| `P04-C11` | `LOC-002` |
| `P09-C01` | `RT-ED-005` |
| `P09-C03` | `LOC-002` |
| `P09-C05` | `LOC-005` |
| `P09-C06` | `LOC-002` |
| `P11-C12` | `LOC-007` |
| `P19-C03` | `RT-ED-003` |
| `P19-C09` | `RT-ED-007` |

Their canonical commands remain in the executable Editor parity ledger.
Consumption and product reruns remain downstream evidence; they do not hold a
faithfully repaired, independently verified, merged runtime row open.

## Stop conditions

Stop and ask the user before:

- creating or accepting a new deliberate divergence;
- treating a browser/backend feature with no C++ equivalent as a parity port;
- changing the renderer/runtime boundary;
- widening a budget, tolerance, resource ceiling, or gate;
- changing the pinned C++ revision;
- using a new adaptation without an existing AF/RF/FLR rule; adjudicate and
  land the rule first;
- implementing from stale ABI evidence;
- publishing an immutable ABI/SDK artifact before the explicit version,
  checksum, and channel checkpoint;
- choosing between two honest tactics after both fail;
- making a source change without a current-pin C++ citation or an explicit
  additive-feature classification.

## Completion

This program is complete only when:

- the executable atlas contains every accepted Editor-reported ID and every
  row has a terminal evidence-backed disposition;
- every active source claim has a current `d788e8ec` result beside, rather
  than in place of, its preserved historical pin/evidence;
- every proven C++/Rust mismatch has a complete owner-family repair merged, or
  an explicit sole active Runtime Fix owner whose tracked landing is
  independently verified before the row closes;
- every API-surface gap has exact underlying record/runtime tests;
- every stale artifact has been replaced or retired;
- every repair records immutable C++ evidence, tests, applicable gates,
  independent reviews, PR, and exact landing SHA;
- the applicable runtime, renderer, browser, Apple, size, and structural
  floors are green;
- all correspondence rows are independently verified;
- no duplicate writer, unowned overlap, displaced compensation, or temporary
  diagnostic remains;
- the atlas, checker, status, and ownership DAG agree; and
- the current intake queue is empty. Later Editor reports begin a new intake
  cycle.

Editor merge and consumption are not completion blockers. When available,
they remain immutable downstream evidence.

The implementation queue is this ownership/dependency map, never the most
visually dramatic screenshot or the latest failing product test.
