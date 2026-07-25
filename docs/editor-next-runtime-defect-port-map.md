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
| `nuxie-editor-next-cutover-proposal.md` | `10fd9abc06b9b75577207711adf463301c6bd4b6e2468ad106846446f9a1150b` |
| `nuxie-editor-next-runtime-defects.md` | `3161a240cd940dec88a387a0b0875df2383705bb9d872f6baaff62b967de8dfa` |
| `nuxie-editor-next-parity-ledger.json` | `f4579d8272bb4f44633afc88f98c0d714f163147ad6ea01765c6ddbcfb8cb180` |

The source copies live under:

`/Users/levi/.codex/worktrees/7189/nuxie-dev/worktrees/editor-next-cutover-assembly/plans/`

The Editor artifacts last consumed runtime commit
`13aedd6d92de0991eed8dc3fda085db2dff18d48`. That is not the same thing as
the runtime investigation HEAD or a future merged/consumed repair. Every atlas
row must retain all of those SHAs separately.

There are 25 unique handoff IDs:

- four active `RT-ED-*` product dependencies;
- two transferred, currently unlinked `RT-ED-*` observations;
- sixteen unresolved `LOC-*` candidates;
- one unlinked `LOC-*` observation;
- one resolved editor-owned `LOC-*`;
- one retracted `RT-ED-*`.

Nine parity children currently name runtime dependencies. Six of those are
blocked by `RT-ED-004`; two name `RT-ED-003`; one names `RT-ED-005`; and one
names `RT-ED-007`. `P04-C01` names both `RT-ED-003` and `RT-ED-004`.

## Boundary

The existing runtime frame-loop port remains separate. It begins at
`StateMachineInstance::advanceAndApply`, follows runtime ownership and update
through live Artboard draw, and stops at the existing
`Renderer` / `RenderFactory` interface.

This Editor Next program spans a wider product boundary for **qualification**
only:

- C++-corresponding runtime and frame-loop behavior;
- the additive high-level `nuxie::Scene` authoring facade;
- browser presentation adapters in `nuxie-renderer`;
- renderer-backend behavior only when an Editor blocker is proved there;
- artifact publication and Editor integration evidence.

Crossing one of those boundaries does not silently widen the C++ runtime port:

- the 1,468-row renderer floor proves the existing primary renderer reference
  path; it does not prove every browser WebGL2 or WebGPU setup path;
- a missing `Scene` authoring operation is an API-surface gap even when the
  low-level Rust runtime already implements the C++ behavior;
- a browser canvas presentation or fallback failure is an adapter/backend
  defect, not a frame-loop ownership defect;
- a failure from a stale ABI or non-identical editor record is not runtime
  source evidence.

## Live frame-loop concurrency lease

This section records the explicit 2026-07-24 handshake with the active
frame-loop executor. It is a writer lease, not a semantic disposition. It must
be refreshed after every FL wave landing before a new production writer is
assigned.

The active frame-loop work is `FL-A`, the atomic Component/update occurrence
graph and exact Component-to-DataBind collapsables contract, on branch
`levi/fl-a` from specification commit `eee8597d`. There is no production
landing SHA yet. The next reserved waves are `FL-B`
(KeyFrame-through-LinearAnimation ownership), `FL-C`
(StateMachineInstance/layer/transition/action/input/listener ownership), then
`FL-D` (Artboard/DataBind/event settlement).

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

The present F-ED checkout also contains quarantined, unstaged experiments in
`animation.rs` and `artboard.rs`. They are not F-ED inputs and must never enter
an F-ED commit. Production F-ED work starts in a clean worktree from the
current merged `origin/main`, while this document may be committed by exact
path.

### Work that is unimpeded now

The following work can proceed without waiting for an FL landing:

| F-ED work | permitted now | hard boundary |
|---|---|---|
| `F-ED-00A` atlas/checker/evidence spine | create the new atlas, status, standalone checker, correction manifest, and new-file direct probe fixtures; run existing ledgers read-only | do not edit shared harness files, probe translation units, or FL ledgers; queue shared-ledger links for the owning FL executor |
| all direct C++/Rust/Editor qualifications | inspect, build, probe, and record evidence in new F-ED-owned files | qualification grants no production-runtime edit |
| `F-ED-03` / `F-ED-04` | after atlas qualification, implement the thin `nuxie::Scene` authoring surface and disjoint authoring tests | exact `TOUCH` set must stay in `crates/nuxie/src/scene.rs` and non-runtime schema/export/import helpers; no runtime, graph, Scene-advance, or frame-loop semantics |
| `F-ED-06` | after `F-ED-07` first localizes the clip row, implement ordinary browser presentation versus explicit readback | exact `TOUCH` set must stay in browser/app presentation modules in `nuxie-renderer`; no runtime edit; serialize against other F-ED browser writers |
| `F-ED-12` | locally build and qualify the existing ABI 1.6 artifact | no header/C-API source change without a fresh scope review; no external publication before the user checkpoint |
| `F-ED-13` | normalize and compare records and repair an Editor/lowering-only first divergence | identical records are required before any renderer attribution; no runtime edit |

`F-ED-03`, `F-ED-04`, and `F-ED-06` are conditionally production-safe, not
pre-approved implementations. `F-ED-00A` must still prove their exact closures.
If one needs any reserved file or changes the frame-loop/advance contract, it
immediately moves to the deferred set.

### Work that may be prepared but needs a landing handshake

`F-ED-07/08/10/11` may perform all qualification now. A resulting fix may be
developed only when its exact production closure is renderer/backend-only.
Before such a fix lands, the F-ED orchestrator must obtain a fresh handshake
from the FL executor and rerun the unchanged 1,468-row pixel referee. The
renderer backend is outside the FL port boundary, but FL uses its pixels as a
merge referee, so an uncoordinated renderer landing would invalidate the
other executor's floor.

The browser writer order remains `F-ED-07` localization before `F-ED-06`,
then qualified `F-ED-08/11`. File overlap among `browser.rs`, `webgl2.rs`, and
WebGPU resource owners serializes those F-ED slices even though they are
disjoint from FL.

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

1. land `F-ED-00A` using only the new F-ED-owned paths specified by that
   ticket;
2. fan out read-only direct qualifications, with `RT-ED-004` first in the
   browser lane;
3. activate only exact Scene-only `F-ED-03/04` and browser-only `F-ED-06`
   production closures that survive the atlas checks;
4. perform record normalization and local ABI 1.6 qualification;
5. keep every runtime-owner result as evidence until the corresponding FL
   executor releases or absorbs its owner family;
6. re-handshake after `FL-A`, and again before any renderer-only landing.

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

Classification and execution state are separate fields. The checker permits
only these forward transitions:

```text
reported
  → reproduced
  → qualified | stale-oracle | retracted

qualified
  → mapped → executor-green → orchestrator-verified → handoff-ready
  → handoff-ready                         # editor/artifact work with no runtime edit

handoff-ready → editor-consumed → closed
stale-oracle | retracted → closed
```

- A separate `owner_class` is exactly one of `runtime`, `api`, `renderer`,
  `editor`, or `artifact`.
- A qualified runtime row may carry `TRACKED-GAP`, `DIVERGENT`, or
  `local-translation-defect`.
- A qualified API row may carry an A-row.
- A qualified renderer row must cite the renderer provenance record and may carry
  the same structural classifications only when a C++ correspondence exists.
- A qualified editor row is transferred to the Editor owner and cannot authorize
  runtime edits.
- A qualified artifact row names the repository and release prerequisite; it is
  not closed until the corrected artifact is green, a current failure is
  promoted to another qualified class, or the user approves an exception.
- `stale-oracle` and `retracted` retain their historical evidence and reason.
- A V-row remains `reproduced` until the missing observation channel exists.
- Only the independent orchestrator may move an executor result past
  `executor-green`.
- `handoff-ready` records an exact merged commit or locally qualified artifact;
  immutable external publication remains a separately approved action/evidence
  field.

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
| `RT-ED-001` | likely stale/resolved transferred observation | rerun focused scripted `data_viz_demo` at current head; current ordinary and scripted floors are already zero-failure |
| `RT-ED-002` | likely stale/resolved transferred observation | rerun focused scripted `db_health_tracker` at current head |
| `RT-ED-003` | proven browser presentation/API defect, not runtime traversal | `crates/nuxie-renderer/src/browser.rs`, `webgl2.rs`, and the WebGPU present/readback boundary |
| `RT-ED-004` | proven WebGL2 capability failure; C++ parity classification still unproven | direct current-pin typed rounded-clip oracle before choosing a renderer translation or additive feature |
| `RT-ED-005` | proven high-level authoring-surface omission | generic number/color `propertyKey` targeting in `Scene`; low-level runtime execution already exists |
| `RT-ED-006` | retracted | retain tombstone only; no source work |
| `RT-ED-007` | proven high-level authoring-surface generalization gap | nested `ViewModelNumberSource` path for transition duration; low-level path resolution already exists |
| `LOC-001` | high-confidence structural Scene ownership mistranslation | type-specific remount carry versus one stable retained ViewModel instance |
| `LOC-002` | high-confidence derived-relation rebind gap | rerun after `LOC-001/005`; then compare C++ immediate current-source evaluation |
| `LOC-003` | unlinked additive-listener/product candidate | first prove whether old editor/C++ owns a long-press primitive or compiles down/up plus timing |
| `LOC-004` | resolved editor-owned | no runtime work |
| `LOC-005` | high-confidence same ownership mistranslation as `LOC-001` | one retained ViewModel instance shared by two source artboards |
| `LOC-006` | WebGL2 clip/composite candidate | logical draw/hit state is correct; qualify independently now |
| `LOC-007` | high-confidence missing C++ dirt callback | ParametricPath width/height/origin setters through Path, Shape, and retained paint-path invalidation |
| `LOC-008` | candidate public API-surface gap | establish the C++ ownership contract, then expose the exact runtime path only if runtime owns it |
| `LOC-009` | unproven scripted GPU-canvas execution defect | identical persisted bytes/resources through C++ scripted drawable and Rust typed WebGPU |
| `LOC-011` | real product symptom, owner unproven | inspect authored text, live VM value, post-bind target, shaped runs, then pixels |
| `LOC-012` | renderer-backend differential | compare WebGPU, WebGL2, and the applicable C++ reference only after text/feather rows are separated |
| `LOC-013` | text/font-pipeline candidate, exact stage unproven | same font bytes, axes, glyph IDs, advances, and outlines through C++ and Rust |
| `LOC-014` | high-confidence mechanism divergence; symptom attribution still needs the oracle | compare Rust feather image plan with C++ Feather/ShapePaint/renderer-LUT ownership; do not tune constants |
| `LOC-015` | stale ABI 1.5 artifact evidence | publish ABI 1.6, pin URL/checksum, rerun |
| `LOC-016` | source implementation already present; publication gap | publish ABI 1.6 typed animation selection and integrate it |
| `LOC-017` | invalid old native capture | rerun ABI 1.6 with typed player/time and production host composition |
| `LOC-018` | evidence/localization gap | prove normalized record and object-order identity before renderer attribution |
| `LOC-019` | browser WebGPU setup/fallback defect | preserve the underlying JS error; prove Auto reaches WebGL2 when WebGPU setup fails |

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

The typed Rust authoring facade hardcodes a world-opacity target. Pinned C++
applies number/color context values through a generic property key:

- `src/data_bind/context/context_value_number.cpp:11-37`;
- `src/data_bind/context/context_value_color.cpp:11-20`.

The low-level Rust importer/runtime already understands generic property-key
records. The missing work is the typed authoring representation, validation,
and exact export/import round trip.

### Nested transition duration — `RT-ED-007`

The typed Rust facade accepts a root `ViewModelNumberId` and exports a
two-element path. The low-level Rust bindable path resolver already walks an
arbitrary nested number path. Pinned C++ has the canonical nested-duration
fixture in:

- `tests/unit_tests/runtime/state_machine_test.cpp:719-748`;
- `transition_duration_bind_nested.riv`.

The repair belongs in the typed authoring facade, not the state-machine
runtime.

### Browser presentation — `RT-ED-003`

The browser wrapper currently makes `finish` mean both presentation and an
RGBA result. WebGL2 screenshots on finish; WebGPU reads back and then copies
pixels through `ImageData`. Ordinary presentation and explicit capture must
be separate operations. This mirrors C++'s lifecycle conceptually, but the
browser API is additive Nuxie infrastructure rather than a C++ runtime file
port.

### WebGL2 clip/feather — `RT-ED-004`, `LOC-006`, `LOC-014`

The current WebGL2 nonrectangular clip allocates full-frame content and mask
images, then composites with `DestinationIn`. Feather uses a locally designed
Gaussian image plan. These are renderer-backend mechanisms, outside the
frame-loop port. Before changing them:

1. prove the exact current-pin C++ reference behavior on the same typed input;
2. identify the corresponding C++ renderer ownership/mechanism;
3. if there is no corresponding C++ capability, stop for an additive-backend
   decision instead of claiming a parity port.

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
the handoff's blocker priority. Qualification and any resulting implementation
are dispatched in this blocker order:

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

`RT-ED-001/002` are rerun here. `LOC-004` is recorded closed,
`RT-ED-006` retracted, and `LOC-010` tombstoned. No old failure is reopened
from prose alone.

`LOC-003` receives its own direct old-editor/C++ authored-listener
qualification here; it must end as `qualified` with a runtime/API/editor owner,
`stale-oracle`, or a user decision. If it is promoted to implementation, the
Editor ledger must first add a dedicated live press/event product child; direct
runtime tests alone cannot close it.

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

Generalize typed number/color binding to carry an owner/property target with
type validation and exact `propertyKey` export. Import/export must round-trip
the same record; the runtime must use the existing generic DataBind execution
path.

Before authoring API code, the Editor owner must normalize the exact P09
fixture into a reviewed target map, and the runtime atlas must ratchet it:

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
- the focused export-limit paywall reproduction;
- a rerun of linked `P09-C01` proving the `RT-ED-005` signature is gone.
  That child closes only after every separately recorded dependency/candidate
  affecting it is consumed.

The focused test must be added to the linked child or represented by a new
ledger child before closure.

### `F-ED-04` — Nested transition-duration source

Target: `RT-ED-007`.

Change the typed authoring operation to accept a `ViewModelNumberSource`
path, validate every segment, and emit the same nested path shape as pinned
C++. Reuse the existing low-level resolver.

Acceptance includes direct root and nested cases, invalid-path diagnostics,
round-trip record identity, runtime duration updates, and linked `P19-C09`.

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

Split ordinary presentation/flush from explicit snapshot/readback. Ordinary
Editor frames must not map a GPU buffer, call WebGL readback, allocate a full
RGBA result, or copy through `ImageData`. Explicit captures must retain exact
pixels and deterministic errors.

Acceptance includes:

- initialization, resize, asynchronous completion, repeated capture, setup
  failure, device/context loss, and teardown of any presentation/readback
  resources;
- an instrumented zero-readback ordinary-frame test;
- an exact explicit-snapshot test for each supported browser backend;
- `scripts/audit-editor-readback-boundary.mjs`;
- `device-frame-bezel-drag.spec.ts`;
- a rerun of linked `P04-C01` and `P19-C03` proving the `RT-ED-003`
  signature is gone; `P04-C01` closes only after `RT-ED-004` is also consumed;
- the 1,468-row renderer floor.

If forced WebGPU setup still traps in the canonical environment, the
direct-present implementation may become executor-green, but WebGPU snapshot
verification and final `F-ED-06` closure wait for `LOC-019` to receive a
disposition. This is a conditional evidence edge, not a semantic dependency
for `LOC-009`.

### `F-ED-07` — Rounded WebGL2 clip qualification and ownership

Target: `RT-ED-004`.

First run the exact 402×874, radius-57 typed clip through pinned C++ and Rust
with the same transform, fill rule, surface size, and pixels. Record whether
the relevant C++ renderer actually owns an equivalent mechanism.

- If C++ passes through a corresponding renderer path, map the complete clip
  owner/allocation/composite lifecycle and translate it.
- If C++ has no corresponding browser/WebGL2 path, stop for a user decision:
  this is an additive backend capability, not an authorized parity port.

The lifecycle proof covers construction, resize, nested clip allocation,
composite order, repeated-frame reuse, allocation failure, context loss/reset,
and teardown.

Acceptance requires the focused clip test plus linked `P04-C01`, `P04-C12`,
`P05-C01`, `P10-C01`, `P12-C01`, and `P15-C01`, with no resource-budget or
error-path weakening. Each rerun must prove the `RT-ED-004` signature is gone;
a child with another recorded dependency does not close until all dependencies
are consumed.

The source artifacts conflict on `P04-C12`: the JSON still names
`RT-ED-004`, while the prose says the case now reaches complete pixels and is
not linked. `F-ED-00` must adjudicate that exact child before this acceptance
set or the formal dependency count becomes binding.

### `F-ED-08` — Conditional stale WebGL2 pixels

Qualification target: `LOC-006`.

Qualify the conditional-visibility fixture immediately; its completed frame
and stale pixels are distinct from `RT-ED-004`'s frame-construction failure.
Compare logical membership, layer clear, mask composition, destination
composition, and final pixels step by step. The first difference is a
localization point only. Re-enter the governing classification:

- repair an exact local site only when the surrounding owner family is
  already faithful;
- port a complete missing lifecycle;
- or replace a structurally divergent owner family and delete its displaced
  mechanism.

Production work is serialized after `F-ED-07` only if `F-ED-00` proves the two
rows share a Rust module or adjacent clip/composite lifecycle.

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

### `F-ED-10` — Feather translation and backend decomposition

Qualification targets: `LOC-014`, `LOC-012`.

Run the exact feather scenes through C++ and both Rust backends. If the Rust
WebGL2 Gaussian plan is the first divergence, classify the complete surrounding
owner/resource lifecycle. A faithful owner permits an exact-site repair; a
missing/divergent lifecycle requires the complete C++-corresponding
Feather/ShapePaint/renderer ownership and LUT/composition translation. Do not
tune sigma, offsets, or tolerances against screenshots.

The proof covers creation, resize, nested feather/composite ordering, repeated
frames, allocation/setup failure, context loss/reset, and teardown.

`LOC-012` may be qualified in parallel. Its final attribution must account for
the `F-ED-09` text result and this feather result; neither is assumed to be a
semantic prerequisite. Backend-to-backend difference alone does not say which
backend is wrong.

### `F-ED-11` — WebGPU setup and GPU-canvas qualification

Qualification targets: `LOC-019`, `LOC-009`.

Qualify both rows independently. `LOC-009` already reaches a selected WebGPU
backend and typed draw, so its direct C++/Rust record oracle is not blocked by
`LOC-019`.

For `LOC-019`, preserve the underlying JavaScript setup error and first prove
the documented Auto/forced-mode contract. If the browser adapter owns fallback
and typed diagnostics, map its complete adapter/resource lifecycle:
initialization, requested limits/features, asynchronous failure, fallback,
resize, device loss, retry policy, and teardown.

For `LOC-009`, feed identical persisted GPU-canvas records, scripts, WGSL,
resources, and time through C++ and Rust. `RuntimeRejected` alone is not a
diagnosis.

Production serialization between the two rows is imposed only if the atlas
proves a shared module or adjacent WebGPU setup/draw lifecycle.

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

The current artifact snapshot has nine children with formal
`runtimeDependencies`, but that is not the complete product rerun set. The
candidate matrix currently maps as follows:

| candidate | directly affected children in the artifact snapshot |
|---|---|
| `LOC-001` | `P13-C07` |
| `LOC-002` | `P04-C11`, `P09-C01`, `P09-C03`, `P09-C06` |
| `LOC-003` | none today; promotion requires a dedicated live press/event child |
| `LOC-005` | `P09-C05` |
| `LOC-006` | `P09-C04` |
| `LOC-007` | `P11-C12` |
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

The candidate set contains 20 unique children. Unioning it with the nine
formal children yields 27 unique directly affected child IDs because
`P04-C12` and `P09-C01` overlap. `F-ED-00` must adjudicate the
`P04-C12` source-artifact conflict before ratcheting the final count. Broad
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
- `LOC-006` is semantically distinct from `RT-ED-004`.
- `LOC-009` is not blocked by `LOC-019`.
- `RT-ED-005` and `RT-ED-007` are independent API gaps.

### Writer/file-lock order

- confirmed blocker priority governs when two rows contend for one module;
- `F-ED-05` shares the FL-E Path/Shape owner lock;
- browser, WebGL2 clip, WebGPU, text, and Scene rows serialize only after
  `F-ED-00` proves overlapping `TOUCH` sets or adjacent lifecycles;
- no candidate lane is called safe before that proof.

## Parallel execution

Qualification fan-out is broader than implementation fan-out.

### Qualification wave

One orchestrator owns the atlas/status and dispatches three read-only scouts:

1. Scene/ViewModel/DataBind/StateMachine:
   `LOC-001/002/003/005`, `RT-ED-005/007`, and `LOC-007`;
2. browser/renderer:
   `RT-ED-003/004`, `LOC-006/009/012/014/019`;
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
| Scene/API | blocker work `F-ED-03 → 04`, then qualified candidate work `F-ED-01/02` | `scene.rs`, schema/export/import helpers, and any runtime handle seam |
| browser | blocker priority `F-ED-07 → 06`, then qualified `F-ED-08/11` | `browser.rs`, `webgl2.rs`, GPU-canvas/scripting seams, and resource owners |
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

The nine children currently carrying formal JSON `runtimeDependencies` are
shown below; `F-ED-00` must resolve the `P04-C12` prose/JSON conflict before
ratcheting this snapshot:

| child | dependencies |
|---|---|
| `P04-C01` | `RT-ED-003`, `RT-ED-004` |
| `P04-C12` | `RT-ED-004` |
| `P05-C01` | `RT-ED-004` |
| `P09-C01` | `RT-ED-005` |
| `P10-C01` | `RT-ED-004` |
| `P12-C01` | `RT-ED-004` |
| `P15-C01` | `RT-ED-004` |
| `P19-C03` | `RT-ED-003` |
| `P19-C09` | `RT-ED-007` |

Their canonical commands remain in the executable Editor parity ledger. A
runtime slice is not product-closed until the Editor worktree consumes its
exact SHA and reruns every formal and candidate-linked command in the
27-child defect matrix plus the affected aggregate gates.

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

- the executable atlas contains all 25 IDs and zero unqualified candidates;
- every active source claim has a current `d788e8ec` result beside, rather
  than in place of, its preserved historical pin/evidence;
- every proven C++/Rust mismatch has a complete owner-family disposition;
- every API-surface gap has exact underlying record/runtime tests;
- every stale artifact has been replaced or retired;
- every formal and candidate-linked child in the adjudicated 27-child matrix,
  plus its affected aggregate gate, passes on the consumed runtime SHA or has
  a user-approved exception;
- the complete runtime, renderer, browser, Apple, size, and product floors are
  green;
- all correspondence rows are independently verified;
- no displaced compensation or temporary diagnostic remains;
- the Editor handoff artifacts contain consistent counts, pins, commands, and
  statuses.

The implementation queue is this ownership/dependency map, never the most
visually dramatic screenshot or the latest failing product test.
