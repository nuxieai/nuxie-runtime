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

Semantic fixture differentials may additionally pass
`--semantic-default-view-model`. This selector is valid only with
`--side-channel` and makes both runners create and bind the artboard's authored
default view-model instance before constructing/draining the semantic tree.
It mirrors the setup used by the pinned semantic runtime tests; without it,
the runners retain the ordinary golden-runner behavior of binding a fresh
view-model instance. The selector adds no stream line of its own: its effects
must appear in the complete semantic diff and subsequent action/focus diffs.

Fixtures whose drawing belongs to a separately tracked parity family may also
pass `--semantic-side-channel-only` (again valid only with `--side-channel`).
Both runners still execute the complete scene and emit the ordinary stream;
the corpus comparator projects that stream to `advance`, `semantics`,
`semanticAction`, and `semanticFocus` records before applying the entry's
ordinary exact/numeric comparison. This is an evidence-scope selector, not a
tolerance or a filed divergence: every semantic payload field and every
semantic transition remains oracle-compared, while unrelated draw records
cannot block promotion of the semantic source rows.

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

### Scripted state, view-model, and resize verbs

Input scripts also accept direct state-machine input mutation and host-resize
verbs:

    <seconds> setInput <name> bool <true|false>
    <seconds> setInput <name> number <float>
    <seconds> setInput <name> trigger
    <seconds> resize <width> <height> <dpr>

`name` is one whitespace-delimited state-machine input name. The selected
scene must be a state machine, and the name must resolve to the declared input
type; a missing scene, missing name, or type mismatch is a script error.
`trigger` calls the runtime's trigger/fire surface once. Boolean and number
writes use the runtime's ordinary input setters, including their normal
needs-advance and listener behavior.

Both runners additionally accept `--view-model-script <path>`. Its
timestamped grammar is:

    <seconds> setVmBool <path> <true|false>
    <seconds> setVmNumber <path> <float>
    <seconds> setVmString <path> <utf8-token>
    <seconds> setVmEnum <path> <u32-index>
    <seconds> setVmColor <path> <0xAARRGGBB>
    <seconds> fireVmTrigger <path>

`path` is one whitespace-delimited slash-separated property path rooted at
the main view-model instance bound to the selected artboard. Empty path
segments are invalid. The terminal property must exist and have the declared
type; otherwise the runner reports a script error. String values follow the
existing whitespace-token grammar and therefore cannot contain spaces. Enum
values are zero-based unsigned indices. Colors are exactly eight hexadecimal
digits prefixed by `0x`. `fireVmTrigger` increments the view-model trigger
through the runtime's public trigger surface rather than assigning an authored
counter value.

All new script floats must be finite. Resize width, height, and DPR must also
be greater than zero. A resize sets the selected root artboard's logical
width and height. DPR does not alter logical runtime coordinates; it derives
the host pixel extent as `ceil(width * dpr)` by `ceil(height * dpr)`.

Input-script and view-model-script commands are merged into one timestamp
order. Commands are applied after the pre-advance to their timestamp. File
order is stable within each script; when commands from both files have the
same timestamp, all input-script commands at that timestamp run before all
view-model-script commands. As with pointer input, commands later than the
last requested sample are not applied.

The commands emit these ordinary golden-stream records whether or not
`--side-channel` is enabled:

    setInput seconds=<float> name=<quoted> type=bool value=<true|false>
    setInput seconds=<float> name=<quoted> type=number value=<float>
    setInput seconds=<float> name=<quoted> type=trigger
    viewModel seconds=<float> path=<quoted> type=bool value=<true|false>
    viewModel seconds=<float> path=<quoted> type=number value=<float>
    viewModel seconds=<float> path=<quoted> type=string value=<quoted>
    viewModel seconds=<float> path=<quoted> type=enum value=<u32-index>
    viewModel seconds=<float> path=<quoted> type=color value=<0xAARRGGBB>
    viewModel seconds=<float> path=<quoted> type=trigger
    resize seconds=<float> logical=(<float>,<float>) dpr=<float> pixels=(<u32>,<u32>)

These records identify the successfully resolved mutation. Script errors do
not emit an outcome line. With `--side-channel`, the following advance and
event/semantic records expose the mutation's runtime effects under the same
rules as pointer and semantic input.

Corpus entries select view-model scripts with
`view_model_script = "<path>"`; `golden-compare` resolves the path relative
to the corpus and forwards `--view-model-script` to both runners. The
existing `input_script` field carries `setInput` and `resize` commands.

## Stream position

Within one sample-loop iteration, with the side channel ON:

    advance seconds=E settled=S statesChanged=N     # pre-advance to input time
    event ...                                       # 0..k reported events
    semantics ...                                   # seven complete-diff lines
    input kind=... seconds=E position=(x,y) pointerId=p   # pointer command
    hit result=R                                          # pointer command
    setInput ... | viewModel ... | resize ...             # mutation command
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
run with the flag ON; every exact entry and segment must match with the
channel enabled, while every non-exact entry remains explicitly parked or
has its divergence localized and filed as a register row.
Corpus entries set `semantic_default_view_model = true` when they need the
pinned semantic-test fixture setup; the comparator forwards
`--semantic-default-view-model` to both runners only for those entries.
They set `semantic_side_channel_only = true` to request the semantic record
projection and forward `--semantic-side-channel-only` to both runners. Such
entries remain `status = "exact"`; they cannot carry a relaxed verification
mode or a side-channel-divergence feature.

## Sampling caveat (register V2)

Most corpus entries sample `t=0` only, where `settled` is forced
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
