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
| `nuxie-editor-next-cutover-proposal.md` | `905bf599f2058828e678bff118261a60fdda4a1a09f4557693b7247409b5beb9` |
| `nuxie-editor-next-runtime-defects.md` | `24e78816d3bafdd61903e4ea1b36ecb77e946accff847963b2ab886d9530b2ae` |
| `nuxie-editor-next-parity-ledger.json` | `07d345c82b8dfd18a06201f08726bafd233f13eabd3cca16c3a8d833f8759226` |

The source copies live under:

`/Users/levi/.codex/worktrees/7189/nuxie-dev/worktrees/editor-next-cutover-assembly/plans/`

The immutable source checkpoint for those hashes is
`233552c13929b09666a62ddff541eb8620d1882b`.

The Editor artifacts at this immutable checkpoint consume runtime commit
`e72323c808b91d706ba3b745396beaca7accd69a`. That is not the same thing as
the runtime investigation HEAD or a future merged/consumed repair. Every atlas
row must retain all of those SHAs separately.

There are 25 unique handoff IDs: seven `RT-ED-*` rows and eighteen `LOC-*`
rows. `LOC-010` is a reserved tombstone, not a defect row.

Eleven unique parity children now carry thirteen structured runtime links
through either `runtimeDependencies` or `runtimeDefects`: two name
`RT-ED-003`; one
names `RT-ED-005`; one names `RT-ED-007`; one names `LOC-001`; four name
`LOC-002`; one names `LOC-005`; one names `LOC-007`; one names `LOC-008`;
and one names `LOC-018`. Ten additional candidate links remain, with 21
unique affected children and no
formal/candidate overlap. `P09-C01` is linked to both the consumed
`RT-ED-005` dependency and the still-red `LOC-002` behavior, but both links
are formal.
Closed `RT-ED-004` is retained only as historical WebGL2 evidence and has no
current structured child. Closed `LOC-013` retains `P08-C08` only as
historical candidate linkage; its exact current font-to-pixel differential
authorizes no runtime writer.

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
- artifact qualification, distribution, and Editor integration evidence.

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
| `F-ED-04` / `RT-ED-007` | preserve the unchanged red acceptance and independently rerun it after the relevant state-machine port wave | deferred post-port verification; no direct Runtime Fix request, schedule, or active writer lease |
| `F-ED-06` / `RT-ED-003` | preserve the independently closed PR #55 landing provenance | the production browser-presentation repair and promotion are complete; do not reopen it or mix it with historical `F-ED-07` |
| `F-ED-10` / `LOC-012` | preserve the qualified required-WebGPU stale-golden evidence and request the explicit user decision | keep `P19-C08`, the registered executable fixture, and `COR-07` open; no runtime/renderer repair, C++ parity, or closure claim exists |
| `F-ED-12` | complete: exact-runtime-identity `0.2.0` artifact and native corpus are qualified | no source repair survived qualification; public distribution is downstream and requires no defect writer |
| `F-ED-13` | normalize and compare records and repair an Editor/lowering-only first divergence | identical records are required before any renderer attribution; no runtime edit |

`F-ED-04` records only the proven state-machine/bindables seam for deferred
post-port verification; it makes no direct Runtime Fix request or schedule,
and the qualified-correct uncommitted Scene producer has no landing claim.
`F-ED-03` remains a merged landing-provenance row awaiting independent
promotion; `F-ED-06` is independently closed. `F-ED-00A` must still prove any
new closure. If an
open lane needs a reserved file or changes the frame-loop/advance contract, it
immediately moves to the deferred set.

### Work that may be prepared but needs a landing handshake

`F-ED-08` is independently closed as a no-repair stale characterization.
`F-ED-10` has qualified evidence that the supported WebGPU product mismatch
is an Editor-owned stale golden rather than a runtime or renderer divergence,
but changing the expected golden requires an explicit user decision. The row
therefore remains open at `stale-oracle`, retains `P19-C08`, its registered
executable fixture, and `COR-07`, and cannot claim pinned-C++ parity or closure
without the exact historical input and provenance.
`F-ED-11` and `LOC-019` are independently closed. PR #54's `LOC-009` consumer repair at
`7f1450dc` remains historical evidence, but independent real-GPU verification
reopened that row on an unresolved physical shader-module error-scope
regression. `LOC-009` is not promotable or complete and requires a new
production landing. Diagnosis is parked pending a different reliable
execution/model environment; replacement clean-worktree task
`019f9f59-1ac6-7e32-b973-5deb6b457c05` ended without authoritative output.
Before such a fix lands, the F-ED orchestrator must obtain a fresh handshake
from the FL executor and rerun the unchanged 1,468-row pixel referee. The
renderer backend is outside the FL port boundary, but FL uses its pixels as a
merge referee, so an uncoordinated renderer landing would invalidate the
other executor's floor.

`F-ED-07` is historical-evidence work only. It may preserve the old WebGL2
observation; it has no linked product scenario. An explicitly scheduled
identical-input proof may requalify the same typed clip on supported WebGPU,
but it
authorizes no WebGL2 writer, fork, dependency, or fallback. There is no active
browser writer from `F-ED-10` while its stale-golden evidence awaits the
explicit user decision; landed `F-ED-06/11` and no-repair `F-ED-08` have no
new writer. Overlap among
`browser.rs` and WebGPU resource owners serializes any surviving open slices
even though they are disjoint from FL.

### Work deferred from production

The following direct probes and source closures are safe, but production
translation waits for the named FL owner boundary:

| F-ED work | qualification now | production release condition |
|---|---|---|
| `F-ED-05` (`LOC-007`) | preserve the exact path-dirt reproducer and d788 expected callback chain | rerun after the corresponding formal path/dirt port wave; no direct implementation request or schedule |
| `LOC-008` in `F-ED-09` | preserve the exact intrinsic-measurement reproducer and pinned-C++ expected bounds behavior | rerun after the corresponding formal text-measurement port wave; no direct implementation request or schedule |
| runtime side of remaining `F-ED-09` rows | text/bind/shaping stage localization | no edits to `text.rs`, `draw.rs`, `artboard.rs`, or another reserved runtime owner |
| runtime side of open `F-ED-11` work | GPU-canvas record and backend localization | a renderer-only result may use the landing-handshake lane; any runtime result waits for its FL owner; open `F-ED-10` authorizes no writer while the stale-golden decision is pending |
| ABI/header/C-API repair | local ABI evidence | separate scope review after a current artifact proves a surviving ABI defect |

If qualification first diverges inside a reserved module, the F-ED
orchestrator records the exact row and gives it to the FL executor; it does not
open a second writer. After each FL landing, the executors exchange the merged
SHA, remaining reservation, displaced mechanisms, and green-floor record
before this table is revised.

### Non-conflicting execution order

The safe next queue is therefore:

1. retain latest intake reconciliation PR #76 at
   `cb4e7748c5b4233375c388b433696ffd34a3c9de`, earlier intake provenance PR
   #67 at `74368a874130a91c9837439b691f0cf44fa4c4a6`, and additive `LOC-018`
   PR #66 at `d7cef0a8b80411b8ef16bf8b48452ea42f71fbe3`; the implementation is
   landed with only the +60/+60 (410 -> 530) authoring claim, while the
   incomplete inbox row remains evidence-blocked;
2. quarantine the Scene-only `LOC-001` candidate and preserve `LOC-001`,
   `LOC-002`, and `LOC-005` as unchanged duplicate-family acceptances. The
   reported-family dependency maps to FL-D `viewmodel.owner`. Pinned C++
   disproves mutable live-`RuntimeFile` catalog growth, and the original
   LOC-001 fixture authors both images before mounting. Preserve the distinct
   dynamic-image source observation in the deferred post-port list and require
   one identical C++/Rust/Editor stimulus, or an evidence-backed
   Editor-not-applicable disposition, after FL-D before classifying it;
3. keep `LOC-009` outside that shared tracking line, parked, and frozen while
   diagnosis waits for a different reliable execution/model environment;
4. retain the completed `RT-ED-004` support-matrix and `LOC-013`
   variable-font no-repair closures; both have executable current-WebGPU
   evidence and no writer;
5. retain `F-ED-04` as deferred post-port verification of the narrow
   state-machine/bindables seam, with no direct Runtime Fix request, schedule,
   Scene writer, or active runtime lease; preserve `F-ED-10` as qualified
   stale-golden evidence awaiting an explicit user decision, with no
   runtime/renderer writer;
6. perform record normalization for the remaining evidence/localization rows;
   the exact-runtime-identity `0.2.0` artifact qualification is complete;
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
| failure exists only in an old or malformed artifact | unqualified evidence | build or correct an exact-identity artifact and rerun before source work |

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
- A qualified artifact row names the repository and exact artifact identity;
  it is not closed until that artifact is green, a current failure is
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
| `RT-ED-003` | closed independently verified browser presentation repair | ordinary frames acquire one surface with no MAP_READ/ImageData, explicit capture alone reads back, Lost recovery is bounded, and merge `e72323c8` is consumed |
| `RT-ED-004` | closed historical WebGL2 support-matrix observation | preserve the exact old 402×874/radius-57 failure as true evidence; supported WebGPU clips match pinned C++ across the same-runner corpus and browser smoke, P04-C01 is 21/21, and no WebGL2 repair/fork/writer exists |
| `RT-ED-005` | changed intake needs evidence; historical landed generic number/color authoring repair remains preserved | PR #49 merge `08286481` is consumed by Editor and its P09-C01 primitive is green, but the changed inbox record omits separately labeled full Editor and Runtime SHAs; ordinary layout/TextStyle work is separate under P08-C01 / LOC-018 |
| `RT-ED-006` | retracted | retain tombstone only; no source work |
| `RT-ED-007` | confirmed runtime transition-duration binding defect | preserve the qualified-correct Scene bytes and unchanged red acceptance for deferred verification after the relevant state-machine port wave; no direct Runtime Fix request or schedule |
| `LOC-001` | mapped retained-owner gap | quarantined Scene diagnostics prove retained identity is necessary; in-place `RuntimeOwnedViewModelInstance` schema/cell migration maps to FL-D `viewmodel.owner`. Exact C++ disproves mutable file-catalog growth, and the original fixture already authors both images, so no second LOC-001 dependency or writer exists |
| `LOC-002` | confirmed duplicate acceptance for LOC-001 | unchanged direct Scene/ProductHost/browser selected-product reproducers rerun after FL-D `viewmodel.owner`; no separate writer |
| `LOC-003` | closed unlinked additive product feature | pinned C++ has no timed-hold primitive; the user decision authorizes no runtime port |
| `LOC-004` | resolved editor-owned | no runtime work |
| `LOC-005` | confirmed duplicate acceptance for LOC-001 | unchanged cross-artboard shared-boolean reproducers rerun after FL-D `viewmodel.owner`; no separate writer |
| `LOC-006` | closed no-repair stale characterization | exact committed provenance plus the independent no-hover rerun prove the alleged retained-pixel defect was gesture contamination; the legal reproduced/stale-oracle/closed path is complete |
| `LOC-007` | committed Editor evidence plus d788 source identifies a missing dirt chain | deferred post-port verification after the formal path/dirt wave; rerun the unchanged four-test product command and classify resolved/still open |
| `LOC-008` | changed intake needs evidence; exact intrinsic-width/multiline-height product failure | checkpoint `233552c1` records runtime `e72323c8` and 166,969 differing pixels after the empty-value fix, but lacks a separately labeled full Editor SHA; preserve deferred post-port verification and the unchanged P08-C06 command without authorizing a writer |
| `LOC-009` | historical structural WebGPU consumer repair with a confirmed real-GPU shader-module validation regression | preserve PR #54 / `7f1450dc` as history; keep the row parked and frozen until diagnosis resumes in a different reliable execution/model environment |
| `LOC-011` | closed Editor-owned lowering defect | identical explicit-empty source-first bytes stay empty through import, bind, shaping, and draw in pinned C++ and Rust; Editor fix `fc1a7e40` repairs absent-versus-empty lowering and the browser reports both prices empty |
| `LOC-012` | open Editor-owned stale-golden evidence | the required-WebGPU visual/spacing gate is 2/2 after replacing only the obsolete expected image, but accepting that expected-image change requires an explicit user decision. Exact historical input is unavailable, so no C++/Rust parity, renderer repair, or closure is claimed; `P19-C08`, the registered fixture, and `COR-07` remain open |
| `LOC-013` | closed Editor-owned variable-font stale oracle | exact Inter bytes and four `wght` values match through 64 glyph IDs, 1,507 outline commands, typed import, 38-line streams/resources, and every 240×112 C++ Dawn/Rust wgpu MSAA pixel; retain P08-C08 as historical linkage only |
| `LOC-014` | closed stale-oracle observation | exact 180x124 clockwise Feather replay matches pinned C++ and Rust in bounds, atlas plan, and every pixel; do not tune constants |
| `LOC-015` | closed stale-binary artifact qualification | exact `0.2.0@b1f58004` framework draws the production corpus; no runtime repair remains |
| `LOC-016` | closed artifact verification gap | exact framework plus typed selection passes all 28 named operation/easing animations at start, quarter, and end |
| `LOC-017` | closed invalid historical capture / Editor integration | corrected Editor producer plus typed player/time and production host composition pass the full Metal corpus |
| `LOC-018` | additive Scene authoring repair landed, changed intake still needs evidence | PR #66 / `d7cef0a8` supplies exact typed 409/420 hierarchy/fixpoint; checkpoint `233552c1` records producer `38f5170f`, runtime `e72323c8`, and Journey acceptance `a2dbcd2c`, but lacks a separately labeled full Editor assembly SHA. P08-C06 is its only formal child; runtime execution/pixels stay post-port |
| `LOC-019` | closed independently verified BrowserWebGpu nullable-error repair | real Chrome clean-null and concrete-error paths, the full WebGPU matrix, and corpus 1468 exact/837 byte-exact/0 divergent verify merge `ef9dcedd` in consumed runtime `e72323c8` |

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

The quarantined Scene candidate established that a stable handle and
generation-matched materialization are necessary.
`RuntimeOwnedViewModelInstance` privately owns its property-name schema,
scalar cells, parent relay, list/child edges, and aliases at
`crates/nuxie-runtime/src/view_model.rs:1597-1618`; the in-place migration
needed to preserve those identities maps to FL-D `viewmodel.owner`.

The candidate fallback is not admissible: it remounted owner-sharing
artboards/state machines and bumped the Scene epoch for a scalar image write,
resetting live state; rejected retained schema edits that previously
succeeded; and invented append-only private catalogs and tombstones absent
from C++. Candidate `dcccdf4fb09275783f6910e5a4a01c028f2c817e` (parent
`bd40c60a07bacfc991f3f070ba77de2041c5d978`) plus uncommitted correction diff
SHA-256
`5477057e14eab86a2d0b2b7c5e8e95e2c837bfa33624fa43c6dee9f24aeef981`
are diagnostic only. FL-D `viewmodel.owner` is the sole dependency of the
reported family. The corrected mapping landed as control-plane-only PR #80 at
exact runtime main `22ba401a9f734eafe0fa3a5852e960e47a4c6121`. No repair
landed and no writer is active.

### Dynamic image property source acceptance — unreported FL-D risk

The same audit initially inferred a second mutable-file-catalog dependency,
but exact pinned C++ disproves it:

- `src/file.cpp:310-355,1423,1492-1498` populates assets only during import
  and exposes the resulting catalog for lookup;
- `src/importers/backboard_importer.cpp:31-59,76-100` collects and resolves
  asset referencers during import;
- `src/viewmodel/viewmodel_instance_asset_image.cpp:13-62` gives each
  image-valued ViewModel property a private retained `ImageAsset`, swaps only
  its `RenderImage`, uses sentinel `-1`, and dirties bindings;
- `src/data_bind/context/context_value_asset_image.cpp:13-48` first tries the
  immutable file ordinal and otherwise binds the target to the source
  property's private image;
- `tests/unit_tests/runtime/data_binding_images_test.cpp:179-233` writes a
  newly decoded image, draws, and clears it through the same retained file,
  ViewModel instance, state machine, and artboard.

Rust currently stores only `AssetImage(u32)` in
`crates/nuxie-runtime/src/view_model_cell.rs` and resolves only that ordinal in
`crates/nuxie-runtime/src/artboard_data_bind.rs:7218-7224`. This source shape
identifies an FL-D `viewmodel.owner` plus `databind.context` risk—not a reason
to grow the file catalog—but it is not yet a confirmed defect because one
identical C++/Rust/Editor dynamic-image stimulus, or an evidence-backed
Editor-not-applicable disposition, has not run. Preserve that stimulus in the
canonical deferred post-port list and classify it only after the two FL-D
members land. The original LOC-001 fixture already authors both hero and
alternate images before its first materialization, so this source-level risk
is not part of the LOC-001/002/005 family and creates no current dependency or
writer.

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
checkpoint `233552c13929b09666a62ddff541eb8620d1882b` consumes it through runtime
`e72323c808b91d706ba3b745396beaca7accd69a`. The generic number/color paint
consumer is therefore landed and consumed, and the linked `P09-C01` primitive
is green. That child remains a nonblocking Known Runtime Defect only for the
separate `LOC-002` retained-owner behavior. Broader ordinary layout and
TextStyle dirt/reflow remain under `P08-C01` / `LOC-018`.

### Nested transition duration — `RT-ED-007`

The recovered producer is runtime
`e72323c808b91d706ba3b745396beaca7accd69a` plus an uncommitted `scene.rs`
patch with SHA-256
`16492cda16a2f91da7d612c9348c6cca572b294d0d25b782c42ab686904ef57a`;
no committed SHA contains `bind_transition_duration_source`. Its exact
323-byte artifact `/private/tmp/rted007-scene-e723-dirty.riv` has SHA-256
`b8e1696a3166959ab7afbca6d7e8ba4abaf99c9e04a15f144327699ce54ebe70`,
and its normalized 31-record dump has SHA-256
`aa199f8e58050272016865f24fd0792375ddddc0c48da83b236db282ef30fcf4`.
Record 26 is `DataBindContext(propertyKey=158, flags=0,
sourcePath=[0,0,0])` under transition 25. The producer semantics are correct.
Pinned C++ has the canonical nested-duration fixture in:

- `tests/unit_tests/runtime/state_machine_test.cpp:719-748`;
- `transition_duration_bind_nested.riv`.

Exact fe0 import recognizes target 25 but never resolves a default nested
source; `default_view_model_number_source_value_for_data_bind(0)` remains
`None` before and after bind, set, and advance. The unchanged set → fire →
`advance(0)` path therefore produces opacity 0.8 immediately, not 0.2. Pinned
d788 with the same bytes and equivalent owned occurrence produces opacity
0.200000003 at `advance(0)`, then 0.5 after another 0.5 seconds.

The first divergence is
`crates/nuxie-runtime/src/state_machine.rs::runtime_transition_duration_bindings`
calling the default-instance-only helper and dropping the unresolved nested
reference, narrowly across that file and
`crates/nuxie-runtime/src/state_machine/bindables.rs`, or a focused
`transition_duration_binding` module extracted from them. `instance.rs`,
`data_bind_graph.rs`, and live path resolution are downstream and not
implicated. Non-main commit `dd3be99c` appears to implement the exact seam but
is not an ancestor of fe0/main. This record makes no direct Runtime Fix request
or schedule. The uncommitted Scene API patch has no landing claim, and Defects
Fix keeps only the unchanged acceptance for rerun after the relevant
state-machine port wave.

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
device-frame drag gate pass. Independent direct-presentation, readback,
loss-recovery, renderer, and corpus evidence has promoted and closed the atlas
row.

### Closed historical backend evidence and current WebGPU oracles — `RT-ED-004`, `LOC-014`

The retired backend's nonrectangular clip allocated full-frame content and
mask images, then composited with `DestinationIn`; its feather path used a
locally designed Gaussian image plan. Those dated observations remain true
renderer-backend provenance, never a writer, fork, fallback, or live
implementation target.

`RT-ED-004` is closed through the user-selected support matrix. The original
402×874/radius-57 four-cubic WebGL2 fixture, frame image, command, and exact
`frame.surface.finish_failed` text remain preserved; runtime `95027109`
removed the backend instead of repairing it. Current supported-WebGPU evidence
at runtime `e494995c`, pinned C++ `d788e8ec`, and Dawn `211333b2` proves
1,468/1,468 same-runner entries exact with 1,375 byte-exact and zero
divergences. `gm-clippedcubic2`, `riv-circle_clips` frame 0, and
`riv-clip_tests` frame 0 are zero-delta in both final modes, browser
`gm-cliprects` is zero-delta, and unchanged P04-C01 passes 21/21. That is a
support-matrix resolution, not a WebGL2 repair or retraction.

The historical clip and variable-font evidence package landed together in PR
#78 at exact runtime main
`98bf5de1f9dfc5d280d29d295dcdc4e418f74c9b`. That landing is
control-plane/evidence-only and changes no production renderer or text path.

`LOC-014` is independently closed as a stale oracle:
the same typed 180-by-124 Feather scene, dimensions, pixel density, background,
resources, and bounds produced zero differing pixels in pinned C++ and Rust
WebGPU. Its qualifying comparison:

1. proved the exact current-pin C++ reference behavior on the same typed input;
2. compared the supported Rust WebGPU path with the same normalized stimulus
   and stamped the exact backend/reference provenance;
3. found no divergence authorizing a parity port, production repair, tuned
   feather constant, or relaxed tolerance.

`LOC-006` is separately closed in the committed inbox as gesture-contaminated
stale characterization and has no renderer writer.

## Artifact corrections recorded by F-ED-00

`F-ED-00` corrected or explicitly versioned these contradictions; later
closures below supersede their original scheduling implications:

1. The handoff mixes C++ `f4bb3025` and `d788e8ec`; every source citation and
   fixture must be revalidated at `d788e8ec`.
2. The defect document introduction calls the ledger confirmed, although most
   rows are candidates, one is editor-owned, and one is retracted.
3. `RT-ED-004` originally lacked a direct current-pin C++ clip proof. Hosted
   run `30217608092` now supplies the exact same-runner and browser WebGPU
   evidence; the historical WebGL2 failure remains preserved under the
   user-selected support-matrix closure.
4. `RT-ED-005` cites `outline-visual-consistency.spec.ts`, but linked
   `P09-C01` does not clearly run that focused reproduction.
5. P16 contains a stale current-runtime SHA.
6. P18's exact inventory is now resolved as nine native screens, one signed
   GPU-canvas case, and 28 named animations at start, quarter, and end.
7. `LOC-012` historically reported both 5,790 and 5,616 exact pixel
   differences. `COR-07` remains open: each historical count still needs its
   backend, mode, surface, capture, and artifact provenance before it may be
   used even as optional characterization. Current supported-WebGPU evidence
   qualifies a stale golden but does not close the row or authorize changing
   the expected image without an explicit user decision.
8. `LOC-013` historically reports both 10,199 and 11,019 baseline differences
   without a capture/mode label. The durable current-input driver supersedes
   those counts for runtime disposition: pinned C++ Dawn and Rust wgpu are
   pixel-identical, while the conflicting retired-WebGL2 counts remain
   historical Editor provenance.
9. `linkedRerun` is used inconsistently, especially for `P15-C01`.
10. The “42-case” GPU matrix reports 45 results because three auxiliary tests
    are included; the label must say so.
11. `LOC-015/016/017` are closed by the locally qualified, hash-addressed
    `0.2.0@b1f58004` artifact. Public distribution and iOS-main consumption
    remain downstream and are not defect closure gates.
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

### `F-ED-01` — Stable ViewModel owner after FL-D

Mapped runtime owner family: `LOC-001`, with `LOC-002` and `LOC-005` as
duplicate acceptance cases and no separate writers.

The direct current-pin differential and pinned d788 owner audit confirm the
structural classification. The quarantined Scene-only candidate proved one
retained handle per authored ViewModelInstance/default identity is necessary.
The atomic in-place schema/cell migration maps to FL-D
`viewmodel.owner` and must preserve the same owner, compatible cells,
child/list edges, aliases, dependents, state machines, and animation state.
Exact pinned C++ rejects the inferred second mutable-catalog requirement:
imported file assets stay fixed, while the image-valued property privately
owns the live image used as the binding fallback. The original LOC-001 fixture
already authors both images before mounting, so the reported family waits only
on `viewmodel.owner`. Under AF-1, AF-2, and AF-8, a remount, scalar replay,
newly rejected schema edit, append-only catalog, or tombstone is not a
translation of the C++ retained-owner lifecycle.

Acceptance must cover:

- number, string, boolean, color, image, enum, trigger, list-index, list,
  nested ViewModel, and artboard values;
- equal-value return semantics;
- an unrelated Scene edit preserving the exact live instance;
- one instance bound to two artboards with writes visible through both;
- two distinct authored/default identities remaining pointer- and
  value-independent;
- detach/rebind/drop and deep-copy cases matching pinned C++.

This is one defect/acceptance family with one lower-runtime dependency. Do not
land per-type carry extensions or the quarantined Scene fallback.
`LOC-001`, `LOC-002`, and `LOC-005` rerun after FL-D `viewmodel.owner`.
No repair or active writer is claimed here.

### `F-ED-02` — No separate writer

`LOC-002` is proven to share the `F-ED-01` retained-owner replacement.
Preserve its exact retained/fresh relation and product reproducers as
post-port acceptance cases; do not create a second DataBind-container writer
or add post-remount value replay.

### FL-D source acceptance — dynamic image property

This is not an Editor-reported defect row and is not part of `F-ED-01`.
Pinned d788 source and `data_binding_images_test.cpp:179-233` expect a newly
decoded image to update the property-private asset while the same file,
ViewModel instance, state machine, artboard, and immutable file catalog remain
retained. Rust's ordinal-only source shape creates a concrete risk across
FL-D `viewmodel.owner` plus `databind.context`, but no identical Rust stimulus
has established a differential yet. After both members land, run one
same-input C++/Rust/Editor differential, or record an evidence-backed
Editor-not-applicable disposition, and only then classify it as resolved or
import a new defect row. This observation authorizes no current implementation
or writer.

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
checkpoint `233552c13929b09666a62ddff541eb8620d1882b` consumes the repair through
runtime `e72323c808b91d706ba3b745396beaca7accd69a`. The generic
property-target portion of `P09-C01` is green; its only remaining nonblocking
runtime defect is the separate `LOC-002` retained-owner behavior. Ordinary
LayoutComponent padding and TextStyle font-size/line-height authoring plus
runtime dirt/reflow remain under `P08-C01` / `LOC-018`, not this row. Historical
executor evidence remains preserved, but the changed atlas row is
intake-needs-evidence until its committed source labels full Editor and Runtime
SHAs.

### `F-ED-04` — Nested transition-duration source

Target: `RT-ED-007`.

The completed five-part report proves the Scene-emitted record is correct and
the first divergence is
`state_machine.rs::runtime_transition_duration_bindings` dropping an
unresolved nested default reference, narrowly across
`crates/nuxie-runtime/src/state_machine.rs` and
`crates/nuxie-runtime/src/state_machine/bindables.rs`, or a focused
`transition_duration_binding` module extracted from that seam.

Record this as deferred post-port verification, not a direct Runtime Fix
request or schedule. Do not assign `scene.rs`, `instance.rs`,
`data_bind_graph.rs`, or any active writer lease. The uncommitted Scene API
patch has no landing claim. Defects Fix preserves and runs only the unchanged
set → fire → `advance(0)` acceptance after the relevant state-machine port
wave lands, then classifies the row resolved or still open.

### `F-ED-05` — ParametricPath dirt callbacks

Deferred post-port verification target: `LOC-007`.

Preserve the unchanged four-test command
`CARGO_INCREMENTAL=0 CARGO_HOME=/private/tmp/nuxie-editor-cargo-home bash
tools/nuxie-editor-next/scripts/cargo.sh test -p browser-host --test
product_host command_authored_resize_ --offline -- --nocapture`. Pinned d788
expects width/height changes to call `ParametricPath::markPathDirty`, then
`Path::markPathDirty`, `Shape::pathChanged`, and `PathComposer::update`, so
sampled geometry changes from 96×44 to 160×68 rather than remaining static.

This row makes no implementation request, schedule, or active writer lease.
After the corresponding formal path/dirt port wave lands, Defects Fix reruns
the unchanged command, records property values, dirt propagation, geometry,
draw operations, and pixels at frames 0/15/30, then classifies the item
resolved or still open. No renderer reset, remount, replay, or transform
substitution is permitted.

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
closure or handoff. Independent promotion confirms ordinary surface
acquisition=1/MAP_READ=0/putImageData=0, explicit readback surface
acquisition=0/MAP_READ=1, Lost recovery acquisitions=2/surfaces=2, persistent
Lost typed and bounded, renderer 418 pass/40 ignored, and the native corpus
1468 exact/837 byte-exact/0 divergent. The atlas row is closed.

### `F-ED-07` — Closed historical rounded WebGL2 support-matrix row

Closed row: `RT-ED-004`.

This ticket owns no production implementation. The original 402×874,
radius-57 four-cubic WebGL2 observation remains true historical evidence,
including the exact frame asset, command, and failure text. The user decided
on 2026-07-24 to remove WebGL2/FemtoVG/fallback support and require WebGPU, so
runtime `95027109` is the support-matrix resolution rather than a WebGL2
repair.

Current qualification is complete at runtime `e494995c`, pinned C++
`d788e8ec`, Dawn `211333b2`, and Editor checkpoint `233552c1`:

- `make renderer-golden-same-runner` reports 1,468/1,468 exact, 1,375
  byte-exact, and zero divergences;
- `gm-clippedcubic2`, `riv-circle_clips` frame 0, and `riv-clip_tests` frame 0
  have zero delta in both final modes;
- `make browser-webgpu-only-check` reports zero differing `gm-cliprects`
  pixels; and
- unchanged Editor P04-C01 passes 21/21.

The row has no formal/candidate child, displaced live mechanism, or writer.
Executor verification passes; independent orchestrator promotion is not
applicable because no parity repair lands. No later work may restore WebGL2,
fork FemtoVG, add a fallback or fixture special case, or describe the
historical failure as false, retracted, or stale.

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

`LOC-013` variable-font qualification and `LOC-011` empty-text qualification
are complete. `LOC-008` remains intake-needs-evidence and deferred post-port
verification.

This is a shared evidence cluster, not a predeclared single writer. The rows
may localize to different owner families: Scene/import/binding initialization,
font instance/shaping/outline/raster, and a public measurement facade.
`F-ED-00` must produce exact file/member closures before writer assignment.

1. Empty text: completed for `LOC-011`. The exact authored record,
   instantiated ViewModel string, post-bind target, shaped runs, and draw
   commands remain empty in pinned C++ and Rust. Editor fix `fc1a7e40` repairs
   the first unqualified absent-versus-empty lowering boundary, and the
   unchanged browser page paints no placeholder glyphs; no runtime repair is
   authorized.
2. Variable font: completed for `LOC-013`. The exact 879,708-byte Inter font,
   face 0, size 17, line height 22, and `wght` 400/500/600/700 match through
   all 64 glyph IDs, 1,507 outline commands, four distinct weight outline
   hashes, typed `.riv` import, 38-line renderer programs/resources, and every
   240×112 MSAA pixel. The durable driver pins source/compiler/probe/archive/
   generated-result/replay-binary provenance and closes the retired
   WebGL2/old-baseline report as an Editor-owned stale oracle. No runtime
   writer or repair exists; P08-C08 remains historical linkage.
3. Measurement: preserve the unchanged P08-C06 command
   `CARGO_HOME=/private/tmp/nuxie-editor-cargo-home rustup run stable cargo
   test --manifest-path tools/rive-compiler/scene-shared/Cargo.toml -p
   nuxie-scene-compiler --lib
   document_lowering::tests::lowers_list_alias_projection_value_to_a_name_resolved_text_run_binding
   --offline -- --exact && PAGE_PARITY_ASSERT=1 pnpm --dir
   apps/nuxie-dashboard run test:visual:page --grep 'Real-Data Paywall /
   Paywall'`. The pinned-C++ expectation is exact shaper-owned intrinsic width
   and multiline height: the 354-wide subtitle occupies 47.59375 over two
   lines and intrinsic labels do not retain the 180-pixel fallback. Make no
   implementation request or schedule; after the formal text-measurement port
   wave lands, Defects Fix reruns the unchanged command and classifies the
   item resolved or still open. Do not add a DOM/editor approximation.

Every promoted owner family becomes a separate committed and floor-gated
slice. Closed no-repair rows create no writer.

### `F-ED-10` — Qualified supported-WebGPU stale-golden evidence

`LOC-012` remains open at `stale-oracle` pending the explicit user decision
required to change the expected golden. At Editor checkpoint
`3a16e76c6f8461c573afff278176302bff5b08b1` on runtime
`ef9dcedd82265efc0184f4f59d5f6aaab0b56cd9`, the unchanged required-WebGPU
visual/spacing gate passes 2/2 in 2.7 seconds after replacing only the obsolete
golden. Of the original 175,032 differing pixels, 173,764 were white in the
old golden versus the authored `#13253d` background; the rest were authored
border and clip-radius details omitted from the old expected image.

This qualifies the stale-golden diagnosis but is not a C++/Rust renderer
parity result and does not authorize closure. The historical input is
recorded as 882,146 bytes at SHA-256
`563da6e08c413f76eb1b728ce2d998098ae7ec1fada9e383daa5f44bb6973d16`,
but exact-checkpoint publisher Wasm SHA-256
`a7877b243a7f82e5c562700d14b4d3374d470dcfd325c0c314469b286e299773`
regenerated 882,277 different bytes at SHA-256
`bf84207ebe120d9790ffb7871ef83c58abfa3fcbc60d5647ca23af8884bb870a`.
The former WebGPU/WebGL2 comparison therefore remains optional historical
characterization until that exact artifact and build provenance exist.
`COR-07` remains open for its conflicting historical counts. `P19-C08` and
the current executable registered fixture remain attached to the open row.
No runtime/renderer writer is authorized while the user decision is pending.

`LOC-014` remains separately closed after exact pinned-C++/Rust Feather pixel
parity. Do not tune sigma, offsets, thresholds, renderer tolerances, or the
`LOC-012` expected golden without the recorded decision.

### `F-ED-11` — WebGPU setup and GPU-canvas qualification

`LOC-019` is independently localized, repaired, verified, consumed, and
closed. `LOC-009` preserves its independently localized consumer repair as
history, but a later independent real-GPU regression reopened that separate
row; neither row is an Editor workaround or a WebGL2 fallback.

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
P14-C06 matrix passes 17/17. Independent promotion verifies real Chrome
clean-null output at 64 pixels/32 red without a device error, invalid WGSL
preserving a concrete validation error, the full WebGPU matrix green, and the
native corpus 1468 exact/837 byte-exact/0 divergent. The atlas row is closed;
no queued hosted Apple work is relabeled green.

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
required after diagnosis resumes and completes in a different reliable
execution/model environment. Replacement task
`019f9f59-1ac6-7e32-b973-5deb6b457c05` ended without authoritative output.

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

The sole authoritative browser observation is the canonical Chrome abort at
`luaG_indexerror` / `luaD_throw`; its cause remains under investigation.
Replacement task `019f9f59-1ac6-7e32-b973-5deb6b457c05` ended without
authoritative output. LOC-009 is parked and frozen outside the shared tracking
line until a different reliable execution/model environment is available, its
reopened cycle has consumed no runtime or superproject landing, and it may
consume only a reviewed landed replacement SHA.

The two rows share a real-Chrome smoke harness but not a defect mechanism:
`LOC-019` owns nullable WebIDL error-scope decoding, while `LOC-009` owns RSTB
and authored-WGSL consumption.

### `F-ED-12` — Apple runtime 0.2.0 exact-identity qualification (closed)

Targets: `LOC-015`, `LOC-016`, `LOC-017`.

The qualification is complete:

1. the runtime repository built exact identity
   `0.2.0@b1f58004332a73564ffdd9f8585838209604c4d1`;
2. Editor PR #5080 landed the corrected producer at
   `233552c13929b09666a62ddff541eb8620d1882b`;
3. qualification-only iOS commit
   `f9528fe4295de0a55d121fd7e5290374b22f03c5` staged that exact framework,
   preserved typed player/time selection, and ran the production host;
4. run `5ef5769f-d521-4471-8b91-b9f83acdd065` passed all six sentinels,
   nine native screens, signed GPU canvas, 28 named animations, behavior,
   archive purity, and framework validation.

Clients bind runtime version plus exact source revision; there is no second
client ABI version. No current failure survived to a runtime owner. Public
URL/default SwiftPM distribution remains optional downstream work.

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

The landed artifact snapshot has 11 unique structured children, but that is
not the complete
product rerun set. `LOC-001` now formally owns `P13-C07`; `LOC-002` formally
owns `P04-C11`, `P09-C01`, `P09-C03`, and `P09-C06`; `P09-C01` also records
the consumed `RT-ED-005` dependency. The remaining candidate matrix maps as
follows:

| candidate | directly affected children in the artifact snapshot |
|---|---|
| `LOC-006` | `P09-C04` (historical source linkage retained after the stale characterization closed; it authorizes no writer) |
| `LOC-009` | `P14-C01` |
| `LOC-012` | `P19-C08` (open stale-golden evidence awaiting the explicit expected-image decision) |
| `LOC-013` | `P08-C08` |
| `LOC-014` | `P08-C09` |
| `LOC-015` | `P18-C01`, `P18-C04`, `P18-C05`, `P18-C07` |
| `LOC-016` | `P18-C01`, `P18-C04` |
| `LOC-017` | `P18-C07` |
| `LOC-019` | `P14-C06` |

The candidate set contains ten unique children. Unioning it with the 11 formal
children yields 21 unique directly affected child IDs with no
formal/candidate overlap. The current Editor checkpoint retains `RT-ED-004`
only as closed historical WebGL2 evidence and removes all five of its former
formal child links. Closed `LOC-013` retains `P08-C08` only as historical
candidate linkage. Structured linkage now makes `P08-C06` a formal child of
both `LOC-008` and `LOC-018`; closed `LOC-011` has no active child, and open
`LOC-012` retains `P19-C08` until the explicit stale-golden decision. Broad
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

Several `LOC-*` rows remain reported/unqualified; each must reach an
evidence-backed terminal disposition or a tracked post-port dependency before
the program checkpoint closes.

## Dependency types

Evidence, semantic ownership, and writer serialization are separate fields in
the atlas. A shared file is not proof of a semantic dependency.

### Evidence prerequisites

- `F-ED-00A` precedes every production change. Its qualification may authorize
  only the file-disjoint API, browser-presentation, renderer-only, Editor, and
  artifact lanes named by the live concurrency lease.
- completed `F-ED-00B`, and therefore full `F-ED-00`, precedes every
  runtime-owner production translation.
- `LOC-012` remains open at `stale-oracle`: its current registered
  required-WebGPU fixture and `P19-C08` stay live until the user explicitly
  decides whether to accept the expected-image change. `COR-07` remains open;
  optional historical characterization cannot substitute for that decision.
- exact-runtime-identity `0.2.0` local qualification and the corrected native
  product rerun are complete; public distribution is not a defect-program
  dependency.
- `F-ED-14` waits for every directly affected branch, not just the four
  current `RT-ED-*` blockers.

### Semantic prerequisites

- `LOC-001/002/005` are one retained-owner acceptance family. Schema/cell
  identity maps to FL-D `viewmodel.owner`; exact pinned C++ disproves mutable
  live-`RuntimeFile` catalog growth, and the original fixture already authors
  both images, so this family has no second dependency. `LOC-002` and
  `LOC-005` have no separate writer; the quarantined Scene candidate is
  diagnostic only and no repair landed.
- the unreported dynamic-image source acceptance waits for FL-D
  `viewmodel.owner` plus `databind.context`, then one identical
  C++/Rust/Editor stimulus or an evidence-backed Editor-not-applicable
  disposition. It is not a LOC-001 dependency or confirmed defect before that
  differential and authorizes no writer.
- `LOC-006` is a no-repair stale characterization, semantically distinct from
  closed historical support-matrix row `RT-ED-004`; neither has a production
  writer.
- `LOC-013` is a closed Editor-owned variable-font stale oracle; its historical
  P08-C08 link does not create a text/runtime writer.
- `LOC-009` is not blocked by `LOC-019`.
- `RT-ED-005` is a landed API gap; `RT-ED-007` is a distinct confirmed runtime
  seam defect retained for deferred post-port verification. Neither record
  creates a direct Runtime Fix request, schedule, or active writer lease.

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

One orchestrator owns the atlas/status. Landing-provenance reviewers may resume
`RT-ED-005` promotion only after its changed intake evidence is complete;
`RT-ED-003` and `LOC-019` are already
independently closed, with no implementation scout or writer assigned.
`LOC-009` stays outside the shared
tracking line, parked and frozen until diagnosis resumes in a different
reliable execution/model environment and a fresh coordinator assignment
follows. The orchestrator may dispatch three read-only
scouts for the remaining open rows:

1. Scene/ViewModel/DataBind/StateMachine:
   `LOC-001/002/005`, `RT-ED-007`, and `LOC-007`;
2. browser/renderer:
   closed `RT-ED-004`, `LOC-006`, and `LOC-014` are historical/no-repair
   evidence, while `LOC-012` retains qualified stale-golden evidence and
   awaits the explicit user decision;
3. text/records:
   `LOC-008/018`; `LOC-011` and `LOC-013` are closed Editor-owned
   dispositions, not remaining qualification scouts.

The orchestrator handles `RT-ED-001/002`, closed/retracted/tombstone rows, and
the exact-runtime-identity `0.2.0` artifact lane.
Scouts return direct fixtures, current-pin source closures, and classifications
only. They do not edit production code.

### Writer wave

These are candidate writer lanes, not pre-approved worktree assignments.
`F-ED-00` must replace every illustrative lock below with exact checked
`TOUCH`/`DON'T TOUCH` sets and prove no collision with active worktrees or FL
ownership before activation.

| candidate lane | order if closure overlaps | expected area to prove |
|---|---|---|
| Scene/API | landed `F-ED-03` has no writer; `F-ED-04` grants no Scene lease because its recovered producer bytes are qualified correct | `scene.rs`, schema/export/import helpers, and public API work remain outside RT-ED-007's post-port verification |
| post-port runtime verification | after the corresponding formal waves land, Defects Fix reruns RT-ED-007, LOC-007, LOC-008, and LOC-018's remaining runtime layout/TextStyle acceptances | evidence-only; no direct Runtime Fix request, schedule, or active writer lease |
| browser | landed `F-ED-06` and the `LOC-019` half of `F-ED-11` have no writer; reopened `LOC-009` is parked until diagnosis resumes in a different reliable execution/model environment and a fresh coordinator assignment follows; no-repair `F-ED-08/10` and historical-only `F-ED-07` have no writer | `browser.rs`, WebGPU GPU-canvas/scripting seams, and resource owners; no `webgl2.rs` writer |
| runtime dirt | qualified `F-ED-05` | waits for the exact FL-E Path/Shape owner lock |
| text-derived | deferred `LOC-008`; closed `LOC-011` and `LOC-013` create no writer | after the formal text-measurement wave, rerun LOC-008 only; preserve the empty-text and variable-font no-repair evidence |
| renderer feather | open-decision `F-ED-10` | no writer; `LOC-012` is qualified Editor stale-golden evidence awaiting the explicit decision, and `LOC-014` is exact no-repair parity evidence |
| evidence/artifact | closed `F-ED-12`; open `F-ED-13` | no F-ED-12 writer remains; F-ED-13 grants no runtime source edit until qualification promotes a survivor |

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
- implementing from stale artifact-identity evidence;
- publishing an immutable Apple runtime artifact before the explicit version,
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
- every proven C++/Rust mismatch has a complete owner-family repair merged, is
  genuinely non-port work explicitly delegated to a sole external owner whose
  tracked landing is independently verified, or remains a formal-port-wave
  dependency whose unchanged post-port acceptance is independently rerun and
  classified resolved before the row closes;
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
