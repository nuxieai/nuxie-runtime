# Golden-Stream Runtime Side-Channel (v2)

Closes register row V4 (with H4 folded in): everything the runtime returns
without drawing — reported events, per-pointer hit results, the
`advanceAndApply` settled/needs-frame bool, and the changed-state count — is
serialized into the diffed golden stream by BOTH runners and compared
line-exactly by `tools/golden-compare` under the ordinary stream rules.

Map tickets: #OR-1 (spec + C++ emit), #OR-2 (Rust emit + corpus-wide gate).
The #LT-1 extension adds semantic/accessibility records without changing the
activation flag or comparator rules.

## Activation

Both runners accept `--side-channel`. Without the flag the stream is
byte-identical to the pre-side-channel format. With the flag, the lines below
are interleaved into the existing `rive-golden-stream-v1` stream at the exact
positions defined here. `golden-compare --side-channel` passes the flag to
BOTH runners; enabling it for only one side must fail every segment (the
stub-baseline check). `--side-channel` is rejected in `--benchmark` mode.

## Oracle contract

Every value records what the **pinned C++ embedder surface** returns
(`RIVE_RUNTIME_DIR` at `LAST_SYNCED_SHA`); the Rust runner mirrors the
corresponding ported facade. Citations:

- `settled` — negation of `Scene::advanceAndApply(elapsed)`'s return.
  C++ state machines: `StateMachineInstance::advanceAndApply`
  (`state_machine_instance.cpp:2601-2665` — note the zero-second force-true
  and the pending report/listener-view-model terms). C++ static scenes:
  `StaticScene::advanceAndApply` returns `true` unconditionally
  (`static_scene.cpp:22-28`). Rust mirrors:
  `StateMachineInstance::advance_and_apply_return`
  (`state_machine/state_machine_instance.rs`) over the runner's composite
  advance result, and `static_scene.rs`'s constant-true facade.
- `statesChanged` — `StateMachineInstance::stateChangedCount()`
  (`state_machine_instance.cpp:2955-2966`); Rust
  `StateMachineInstance::changed_state_count()`.
- `event` lines — `reportedEventCount()/reportedEventAt(i)`
  (`state_machine_instance.hpp:226-229`, `event_report.hpp`); Rust
  `reported_event_count()/reported_event(artboard, i)`
  (`state_machine/event_report.rs`). This is H4's one thin trace.
- `hit` lines — the `HitResult` returned by `Scene::pointerDown/Move/Up/Exit`
  (`scene.hpp:55-60`; tri-state computed in
  `StateMachineInstance::updateListeners`,
  `state_machine_instance.cpp:1494-1545`; base `Scene` returns
  `HitResult::none`, `scene.cpp:18-24`). Rust: the tri-state
  `RuntimeHitResult` pointer variants (projection of the same FL-ported
  internal result that the established `bool` facade collapses).
- `semantics` lines — the complete `SemanticsDiff` returned by
  `StateMachineInstance::semanticManager()->drainDiff()` after semantics are
  enabled (`state_machine_instance.cpp:2413-2424`,
  `semantic_manager.hpp:48-50`, `semantic_snapshot.hpp`). Rust drains the
  selected retained manager through
  `StateMachineInstance::drain_semantics_diff(artboard)`; that call also
  synchronizes live mounted occurrences before producing the diff.
- `semanticAction` — dispatch through
  `StateMachineInstance::fireSemanticAction(id, action)`
  (`state_machine_instance.cpp:2552-2582`). The C++ API is `void`, so the
  observable outcome is whether the manager resolved the id to a non-boundary
  `SemanticData`; callback effects are recorded by the following advance's
  complete semantic diff. Rust reports that same resolution boundary rather
  than exposing its additional internal callback boolean.
- `semanticFocus` — the bool returned by
  `SemanticManager::requestFocus(id)`; Rust mirrors it with
  `StateMachineInstance::request_semantic_focus(id)`.

## Line grammar

All floats use the existing stream float format (C++ `floatToString`, Rust
`write_float`: `max_digits10` defaultfloat). All strings use the existing
`quotedString` escaping. Field order is fixed. The comparator's numeric
epsilon applies to embedded numbers exactly as on every other stream line.

### `advance` — one line per `advanceTo` call

Emitted immediately after every advance the runner performs — both the
pre-advance to an input event's timestamp and the advance to each sample.
This includes zero-elapsed advances (e.g. the advance to t=0 before the first
sample), where C++ forces the return true, i.e. `settled=false`.

State-machine scenes:

    advance seconds=<target> settled=<true|false> statesChanged=<n>

Static scenes (no state machine; C++ uses `StaticScene`):

    advance seconds=<target> settled=false

`seconds` is the target timeline position of the advance (not the elapsed
delta). `settled` is the negation of the `advanceAndApply` return: `true`
means the runtime reports no further frame is needed. A stop-when-settled
embedder loop may stop exactly when `settled=true`.

### `event` — one line per reported event, after its `advance` line

Events reported by the state machine during that advance, in report order:

    event type=<coreType> name=<quoted> delay=<float> props=[...]
    event type=<coreType> name=<quoted> delay=<float> url=<quoted> target=<name> props=[...]

- `type` — the event's core type key (`Event`, `OpenUrlEvent`, `AudioEvent`).
- `name` — the event's name (empty string when unnamed).
- `delay` — `EventReport::secondsDelay`.
- `url`/`target` — present only for `OpenUrlEvent`; `target` is the
  `targetValue` mapping `0..3` → `_blank|_parent|_self|_top`, else empty.
- `props` — the event's custom properties in child order:
  `props=[{name=<quoted>,value=<typed>},...]`, empty as `props=[]`. Typed
  values: Number → float format; Boolean → `true|false`; String → quoted;
  Color → `0x%08x`; Enum/Trigger → unsigned decimal.

### `hit` — one line per pointer input, after the existing `input` line

    hit result=<none|hit|hitOpaque>

The tri-state returned by the pointer verb the runner invoked. Scenes
without a state machine emit `result=none` (the C++ `Scene` base default).

### `semantics` — seven lines after every state-machine `advance`

When `--side-channel` selects a state-machine scene, both runners call
`enableSemantics()` after scene construction/binding and before the first
advance. Immediately after the `advance` and its zero or more `event` lines,
the runner drains and serializes the complete incremental diff in this fixed
order:

    semantics frame=<u64> treeVersion=<u64> rootId=<u32>
    semantics removed ids=[<u32>,...]
    semantics added nodes=[<node>,...]
    semantics moved nodes=[<node>,...]
    semantics childrenUpdated entries=[<children>,...]
    semantics updatedSemantic nodes=[<node>,...]
    semantics updatedGeometry bounds=[<bounds>,...]

An empty diff still emits all seven lines, including its returned
`frameNumber`, `treeVersion`, and `rootId`. Static scenes emit no semantic
lines because they have no semantic-manager embedder surface.

The nested grammars are:

    <node> = {id=<u32>,role=<u32>,label=<quoted>,value=<quoted>,hint=<quoted>,stateFlags=<u32>,traitFlags=<u32>,headingLevel=<u32>,bounds=(<float>,<float>,<float>,<float>),parentId=<i32>,siblingIndex=<u32>}
    <children> = {parentId=<i32>,childIds=[<u32>,...]}
    <bounds> = {id=<u32>,bounds=(<float>,<float>,<float>,<float>)}

Vector order is the oracle's returned order; runners must not sort it.
Strings and bounds use the ordinary quoted-string and float rules. This is a
full-diff channel: no field may be omitted even when it has its default value.
Consequently it directly covers Text-inferred role/label payloads, explicit
SemanticData precedence, provider root-space bounds, manager-local ids,
parent/sibling order, nested boundaries, removals, and incremental content or
geometry patches.

### Semantic input verbs and outcomes

Input scripts additionally accept these timestamped verbs:

    <seconds> semanticAction <nodeId> <tap|increase|decrease>
    <seconds> semanticFocus <nodeId>

They are applied after the pre-advance to `seconds`, just like pointer input.
With the side channel enabled they emit, respectively:

    semanticAction seconds=<float> nodeId=<u32> action=<tap|increase|decrease> outcome=<dispatched|missing>
    semanticFocus seconds=<float> nodeId=<u32> outcome=<focused|rejected>

`dispatched` means the selected semantic manager resolved `nodeId` to a
non-boundary node with owning `SemanticData` and the runtime invoked the
requested method. It does not claim that an authored listener accepted that
action; any listener/state/data effect is observable in the next advance and
semantic diff. `missing` includes no selected manager, an unknown id, and a
boundary node. `focused`/`rejected` is the exact requestFocus return.

## Stream position

Within one sample-loop iteration, with the side channel ON:

    advance seconds=E settled=S statesChanged=N     # pre-advance to input time
    event ...                                       # 0..k reported events
    semantics ...                                   # seven complete-diff lines
    input kind=... seconds=E position=(x,y) pointerId=p   # existing line
    hit result=R
    ...                                             # further inputs due
    advance seconds=T settled=S statesChanged=N     # advance to the sample
    event ...
    semantics ...
    sample seconds=T                                # existing line
    ...draw lines...                                # existing
    frame                                           # existing

## Comparator and ratchet

`golden-compare --side-channel` forwards the flag to both runners; stream
equality rules are unchanged (the channel is ordinary lines). The summary
gains `side-channel-segments=<n>`: equal to `exact-segments` when the flag is
on, `0` when off. `make golden-compare` and `make scripted-golden-compare`
run with the flag ON; the corpus-wide gate is 317/317 entries with the
channel enabled, or each divergence localized and filed as a register row.

## Sampling caveat (register V2)

237 of 317 corpus entries sample `t=0` only, where `settled` is forced
`false` by the zero-second rule — settling is unobservable there. Settling
behavior is exercised by the dedicated `settle_*` corpus entries (same
fixtures, post-first-frame samples: one-shot animations sampled past their
duration, loops sampled across a wrap, state machines that go idle). Full
corpus densification remains #OR-4.

## Reserved (documented non-goals)

- `key <code> <down|up>` / `textInput <utf8>` verbs (#FT-TEXT, per #OR-3).
- `audio start|stop id=… t=…` (#FT-AUDIO).
- Per-layer changed-state identity: C++ exposes `stateChangedByIndex`
  publicly but the Rust runtime records only the count; per-state identity
  needs a runtime-side recording seam (register V4 follow-up, noted in the
  settling assessment).
- View-model value dumps: no C++-matching public enumeration order exists on
  both sides today; comparing them without a pinned order would only produce
  enumeration noise. Tracked in the V4 register row as an explicit remainder.
- Platform accessibility adapter output. The channel stops at the pinned
  runtime's `SemanticsDiff` and action/focus embedder surfaces; OS-specific
  accessibility-tree behavior is outside the golden runner.
