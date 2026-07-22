# Parity Closeout Status

Live state for `docs/parity-closeout-map.md`. The next session must be able
to resume from this file alone. Keep it small: archive completed-ticket
logs the way `v2-status.md` / `renderer-status.md` did.

## Scorecard (update per session; `make parity-scorecard` once #B-4 lands)

| tier | state | number | notes |
|---|---|---|---|
| 1 Frame parity | PARTIAL | exact-segments 647/647; scripted 647/647; e2e-exact: gate not built | both runtime floors restored green 2026-07-21 (image-policy split); #OR-6 missing |
| 2 Interaction parity | RED | side-channel: gate not built; fuzz-clean-nights: 0 | #OR-1/2/3/7 |
| 3 SDK parity | RED | A-rows closed 0/8 | register A-table |
| 4 Platform parity | PARTIAL | pixel-exact 1468/1468; adapters 2/2; live same-runner 1468/1468 local | static byte-exact 837; live d788 M5 byte-exact 1370; Paravirtual rerun pending; #HD-2's hypothesis oracle and #HD-3 remain |
| 5 Performance & size | RED | ratio 0.897–0.914 (non-blocking, 6 files); size 7.84 MiB OFF / 8.70 MiB ON vs user-approved 9 MiB budget (both variants block; CI recording re-enabled, first green recording pending) | #OR-9 |

Regression floor (must stay green): runtime lib 399/399, nuxie lib 140/140,
C++ probe 721/721, both runtime golden gates 317/317 exact / 647/647 segments;
ordinary and scripted both have zero failures. The workspace push gate is green
as of 2026-07-22 and now builds/exports `RIVE_CPP_PROBE`, so its log contains
the 721/721 probe run rather than silently skipping it. Every remaining RB-1
cut and every RB-1 push must run `make scripted-golden-compare` in addition to
the ordinary gate. Review
of e5(A) restored the distinction between raw `StateMachineInstance::advance`
and the full `advanceAndApply` facade: the facade forces zero-second returns
true and includes pending reports (`state_machine_instance.cpp:2608-2613,
2663-2665`). The two zero-second Scene assertions remain unchanged. The one
event-listener ViewModel assertion now records the exact `applyEvents` timing:
an event created during layer advance is delivered at the next frame start,
where chained notifications drain to completion (`state_machine_instance.cpp:
2320-2343`).
NOTE: the `RIVE_RUNTIME_DIR` checkout governs probe/runner builds — it must
be at pin `d788e8ec`; unpinned checkouts poisoned two earlier floor runs.

Upstream pins: runtime `d788e8ec` (cycle-3 cut `b73bc675`, 3 commits ahead,
awaiting #B-1 approval). Upstream advanced after that completed inventory to
`ba2b6434`; it is next-cycle drift, not part of the pending authorization.
Renderer pixel-oracle `7c778d13` (historical, do not advance casually — see
upstream-sync-map registry).

## Ticket checklist

- [x] #RB-1 data-binding foundation rebuild (map Phase RB; user-directed
  2026-07-21, P0) — port C++'s retained-identity view-model/data-bind core
  and delete the compensation family; floors are the harness; exit gate
  includes the deletion list. Mini-queue:
  - [x] (a)+(b) retained cell core — `view_model_cell.rs` landed: shared
    typed cells (`Rc<RefCell>` ≈ `rcp`), weak dirt-sink dependents
    (`DependencyHelper` analog; cascade sets bits only, no callbacks),
    `ValueFlags::valueChanged`/`advanced()` semantics including trigger
    zeroing under `SuppressDelegation`; 7 unit tests mirror the C++
    contracts. Additive — no consumers yet, floors untouched.
  - [x] (c) parent-linked `DataContext` — landed: cell-backed instances
    (`RuntimeViewModelInstanceCells`, both `createViewModelInstance` forms,
    full kind lattice, nested recursion cycle-guarded, per-kind instance
    seeding) + `RuntimeDataContext` with C++ lookup semantics
    (path[0]=viewModelId, tail walks nested slots, parent fallback);
    10 unit tests incl. identity-preserving lookup and the
    instance-0-vs-defaults split. Lists are placeholder slots (items build
    in slice (d/e) with the list lifecycle). Additive; floors untouched.
    ALSO: scout inventory landed as docs/rb1-compensation-inventory.md
    (five public seams, full call-site lists, deletion checklist,
    test-rewrite counts).
  - [x] (d) retained `DataBind` direction engine — landed
    (`retained_data_bind.rs`): `set_source(cell)` registers the sink as
    dependent (bindsOnce never registers), two dirt bits
    (BINDINGS/BINDINGS_TARGET), TargetOrigin latch with favored-direction
    reconcile, suppressed self-notify on both apply paths, and
    `reconcile()` in C++ favor order. Target application is behind the
    `RuntimeDataBindTarget` trait; converters and the arena wiring land in
    (e). 6 tests incl. the two-way target-seeds-source init ordering (the
    instance-0 scroll-scalar bug class) and sibling propagation without
    echo. NOTE for (e): the old view_model module exports a colliding
    `RuntimeDataContext` name; resolve at migration.
  - [ ] (e) migrate consumers — sequenced by
    docs/rb1-compensation-inventory.md; floors green after every step.
    DESIGN DECISIONS (2026-07-21, binding): (1) `RuntimeOwnedViewModelInstance`
    keeps its public API but its scalar VALUE STORAGE re-backs onto
    `RuntimeViewModelInstanceCells` — hybrid first (scalars on cells;
    lists/children/aliases unchanged), then children share cells (which
    dissolves the alias-mirror machinery naturally: shared identity needs
    no synchronization). (2) `Clone` for the instance must remain a DEEP
    copy (C++ `copyViewModelInstance`); sharing stays explicit via the
    existing `RuntimeOwnedViewModelHandle` Rc wrapper — deriving shared
    scalars into `.clone()` would silently flip snapshot semantics across
    hundreds of call sites. (3) The old `view_model::RuntimeDataContext`
    export name collides with the new core's context; the new type stays
    namespaced until the old one deletes. Steps:
    - [x] e1 re-back owned-instance scalar storage with cells — landed via
      lane worktree, orchestrator-verified gates: rt lib 340, nuxie lib
      132, probe 707/707, golden 317/317 (647), scripted main pass
      317/317 with only the four pre-existing red entries. All ten scalar
      kinds on cells (Enum/Symbol/Asset/Artboard as u32 payloads behind
      u64 APIs with C++ -1 sentinel saturation; String keeps a byte
      mirror for borrowed getters — slot setter is the only writer).
      Deep-copy Clone preserved on every slot. EXCEPTION: AssetFont stays
      on old storage (two-part payload: file-asset index + retained live
      Font bytes with Arc::ptr_eq change semantics) — needs a font-cell
      payload decision in e2.
    - [x] e2 landed via lane, orchestrator-verified (rt lib 341, nuxie
      132, probe 707/707, golden 317/317, scripted main 317/317 + only
      the four known reds): AssetFont onto cells via a change-identity
      stamp beside the retained payload; nested children shared by cell
      identity (C++ rcp semantics) with Clone porting copyViewModelInstance's
      instancesMap dedupe so sharing topology survives inside deep copies;
      alias-mirror bodies reduced to forwarding + debug_assert of shared
      identity (signatures/call sites intact for slice f). At the e2 landing,
      the overlay, duplicate mutable authored-instance constructor, and list
      detachment still formed boundary deep-copies; the later canonical
      constructor parity slice removed that compensation. Authored instances
      now always follow C++ clone + `completeViewModelInstance`, including one
      shared source-instance map across ViewModel and list edges. Top-level
      fonts kept their pre-existing no-mirror asymmetry (revisit at e3/f).
    - [x] e3 landed via lane, orchestrator-verified (rt lib 343, nuxie
      132, probe 707/707, golden 317/317, scripted main 317/317 + exactly
      the four known reds — zero corpus movement): owned-candidate graph
      sources hold retained binds (set_source + rebind-reconcile;
      same-cell ptr_eq rebinds become dirt-driven, ending value-copy
      thrash); cell lookup lives on the owned types
      (cell_for_source_path with C++ tryGetViewModelProperty semantics);
      RuntimeGraphSourceValueTarget adapts the direction engine onto the
      graph's value slot so converters apply unchanged;
      collect_source_dirt now reports whether sink dirt was folded (so a
      rebind latch can't flip the favored origin). KEY FINDING: the four
      reds did NOT flip — C++ pin evidence shows the missing seeding
      lives in the artboard-side owned-path target-to-source pull
      (Artboard::updateDataBinds(true) → DataBind::updateSourceBinding),
      NOT the SM copy path. That pull is e4's PRIMARY target; e3's
      read_target adapter + retained cells are the ready plumbing.
    - [x] e4 landed via lane, orchestrator-verified (rt lib 344, nuxie
      132, probe 707/707, golden 317/317, scripted 317/317 main + only
      TWO reds remaining): db_health_tracker and superbowl FLIPPED GREEN.
      Premise falsified by instrumentation: VM end-of-frame state was
      byte-identical to C++ all along — the real bug was PULL ORDERING
      (child data-bind pulls ran before the nested subtree advanced; C++
      advances the whole subtree first, artboard.cpp:1195-1201 pull
      recursion, nested_artboard.cpp:965-1008). One-statement fix with
      pin citations. Listeners now register as cell dependents
      (dirt-driven with last-delivered dedup; rescan loop intact for
      slice f). Remaining two reds localized with evidence:
      echo_show_demo = SM event-fire timing at t=0 (C++ fires the Night
      listener via StateMachineFireEvent on a duration-100 transition;
      Rust fires neither → Weather enum 3 vs 0);
      list_index_script_access = script-visible list index seam
      (getIndexString "-1"; digit glyph differs). Both are e5/f scope.
    - [x] e5 final retained-cell cutover.
      - [x] (A) Ordinary state-machine frames drain queued events before
        advancing data binds/layers, without the compensating zero-time
        layer advance (`state_machine_instance.cpp:2320-2335,2555-2584`).
        `echo_show_demo` is exact; focused C++ probe coverage raises the
        probe floor to 708/708. Closeout review added independent host-report
        and listener-delivery lifetimes, so `FlowSession` may drain a report
        before advance without suppressing next-frame `applyEvents`; the full
        facade retains C++'s zero-second forcing and pending-report return.
      - [x] (B) `list_index_script_access` closes exact at 33,432 bytes.
        Nested scripted drawables now initialize only after component-list
        mounting, retain each row's occurrence-scoped context, and pull that
        child's data binds before first draw. This mirrors pinned C++ index-
        before-create/bind/init ordering (`artboard_component_list.cpp:759-784,
        1453-1477,1528-1543`; `artboard.cpp:2551-2573`) instead of collapsing
        same-graph rows through a graph-id snapshot.
      - [x] (C) Scene facade drops its dirty bit, advance-time flush, and
        pointer-time rebinds. Owned state-machine triggers retain and read
        the resolved trigger cell; fire/acknowledgment mutate that cell, and
        the suppressed C++ 1→0 acknowledgment does not notify ViewModel
        listeners (`viewmodel_instance_trigger.cpp:10-27`;
        `state_machine_instance.cpp:1374-1380,2546-2565`). Runtime lib rises
        to 345/345; nuxie lib 132/132 and C++ probe 708/708 remain green.
- [x] (f) deletion gate follows e5.
  - [x] (f) deletion gate: mutation clocks, candidate vectors, listener
    rescans, alias mirrors, and remaining copied-direction state all removed;
    floors stay at their completed values, including scripted 317/317 with
    zero failures.

    | Slice | Status | Completion result |
    | --- | --- | --- |
    | f1 | Complete | Deleted the reverse linked-child alias registry. |
    | f2 | Complete | Replaced copied graph direction state with one retained DataBind engine. |
    | f3 | Complete | Deleted the unread per-instance mutation-generation shadow. |
    | f4 | Complete | Replaced owned-listener polling with exact-cell next-frame reports. |
    | f5 | Complete | Deleted compatibility listener snapshots and rescans. |
    | f6 | Complete | Replaced list-row clock unions with retained weak-parent relays. |
    | f7A | Complete | Routed linked Font access through the exact retained child cell. |
    | f7B | Complete | Moved String and Font payloads into retained typed cells. |
    | f8 | Complete | Deleted dynamic linked-child structural mirrors. |
    | f9 | Complete | Retained List/ListLength/ViewModel sources and deleted generation polling. |
    | f10 | Complete | Retained live converter operand cells and deleted operand rescans. |
    | f11 | Complete | Reunified each authored artboard DataBind around one retained state. |
    | f12A | Complete | Moved owned transition-trigger state onto its exact retained cell. |
    | f12B | Complete | Canonicalized default/imported triggers onto file-owned cells. |
    | f13 | Complete | Deleted listener-triggered whole-context rebinding. |
    | f14A-D | Complete | Replaced candidates/shadows with a parent-linked production DataContext and canonical trigger sources. |
    | f15 | Complete | Proved the inventory empty and passed every Phase-RB exit gate and both reviews. |

    Remaining f-prefixed work: none. RB-1 is complete when this f15 evidence
    commit is on `main`; Phase RD is the next spine item.
  - [x] (f1) reverse alias registry deleted. Linked child properties retain
    the child directly and already share scalar cells; no reverse owner list
    or mutation-time mirror fan-out remains. The temporary shared mutation
    clock stays only for structural/list dirt until their retained-dependent
    cut lands. Runtime lib remains 345/345.
  - [x] (f2) copied graph-source `target_origin` deleted. Every graph source
    now owns one always-present `RuntimeRetainedDataBind` direction engine;
    migrated owned scalars attach their cell to it, while compatibility kinds
    use the same engine's source/target/reconcile marks until their cells land.
    Floors: runtime lib 345, nuxie lib 132, probe 708/708.
  - [x] (f3) unread per-instance `mutation_generation` shadow counter and the
    zero-call list-item context replacement helper deleted. The still-live
    shared structural clock is unchanged until list/ViewModel slots gain
    retained dirt. Runtime lib 345/345; probe 708/708.
  - [x] (f4) owned-candidate ViewModel listeners no longer poll copied values
    or run a bind-time bounded fixpoint. Retained scalar cells append the
    listener occurrence for every genuine mutation, preserving duplicates and
    registration order; the next ordinary frame swaps that queue with events
    and drains chained batches up to C++'s 100-batch cap. Trigger-zero
    acknowledgment alone is suppressed, batch 101 stays pending, and both raw
    advance state and the `advanceAndApply` facade include pending listener
    reports (`state_machine_instance.cpp:1374-1380,2320-2343,2583-2584,
    2663-2665,3021-3025,3048-3058`). The compatibility snapshot/context-chain
    listener paths remain for their later retained-context cut. Full evidence:
    runtime lib 348/348, nuxie lib 132/132, probe 708/708, both golden modes
    317/317 entries and 647/647 exact segments with zero failures, C API smoke
    green, and `cargo test --workspace` green.
  - [x] (f5) compatibility snapshot, mutable, and context-chain ViewModel
    listeners now retain the same scalar cells as their borrowed context;
    the remaining `observed` value enum/readers/rescan loops and their
    bind-time settlement API are deleted. Bind/rebind only relinks dependents;
    ordinary new-frame `applyEvents` performs actions, mutable context-aware
    frames receive ViewModel writes, immutable/context-chain frames do not,
    and chained reports drain inside the same 100-batch loop
    (`state_machine_instance.cpp:1331-1427,2320-2343,2555-2565`). The
    `newFrame=false` nested-event follow-up now also leaves newly created
    event/listener reports queued instead of inventing a same-frame drain.
    Full evidence: runtime lib 349/349, nuxie lib 132/132, probe 708/708,
    both golden modes 317/317 entries and 647/647 exact segments with zero
    failures, C API smoke green, and `cargo test --workspace` green.
  - [x] (f6) list rows no longer join their containing instance's mutation
    clock. Each retained instance now owns C++-shaped weak parent relays;
    dynamic list add/insert/update attaches them, removal/truncate/clear/drop
    detaches them, and nested ViewModel replacement recursively rebinds every
    live parent. Pointer-unique duplicate parents, authored-row non-registration,
    and `pop()`'s missing detach deliberately match the pinned lifecycle
    (`viewmodel_instance.cpp:118-188,346-415`;
    `viewmodel_instance_list.cpp:11-24,38-225`; `file.cpp:949-977`). Deep
    copies rebuild only registrations present in the source graph through the
    existing identity memo; alias-mirror list storage retains its real owner.
    Mounted component-list occurrences consume row-local generations after
    settlement, so scalar row dirt stays local without losing row refresh.
    The remaining mutation-clock union is confined to direct ViewModel
    properties pending their retained-dependent cut. Full evidence: runtime
    lib 357/357, nuxie lib 132/132, probe 708/708, ordinary and scripted
    goldens 317/317 entries plus 647/647 exact segments with zero failures,
    C API smoke green, and `cargo test --workspace` green. Renderer goldens
    are not applicable because the slice changes no renderer/draw code. Both
    Standards and Spec reviews are clean.
  - [x] (f7A) linked `AssetFont` values now retain the exact child slot cell
    and route every nested host/data-bind payload write through that retained
    child. This closes the cell/delegation omission in
    `mirror_linked_instance` and is a prerequisite for the retained Font
    source cut. The oracle is C++'s single `rcp<ViewModelInstance>` child
    reference
    (`viewmodel_instance_viewmodel.hpp:19-39`) plus the two-part Font setter
    (`viewmodel_instance_asset_font.cpp:29-75`). The Font payload itself is
    still a snapshot refreshed at the `RuntimeOwnedViewModelHandle::borrow`
    seam, like String bytes; a held parent borrow can remain stale after a
    direct child write. The typed payload endpoint must remove that remaining
    Rust boundary before the direct-property clock union can delete. String/
    Font source dependents, artboard sinks, and converter operands also remain.
    Fast and corpus evidence: runtime lib 357/357, nuxie lib 132/132, probe
    708/708, and ordinary plus scripted goldens 317/317 entries / 647/647
    segments with zero failures. C API smoke and `cargo test --workspace` are
    green; Standards and Spec re-reviews are clean. Renderer goldens are not
    applicable because the slice changes no renderer/draw code.
  - [x] (f7B) owned String and Font payloads now live in their retained typed
    cells instead of parallel slot snapshots. String reads return shared
    immutable bytes; Font reads return the complete file-index/live-Font value,
    while aliases and graph candidates retain the exact source cell. This
    matches String's content-checked setter/deep clone
    (`viewmodel_instance_string_base.hpp:33-55`;
    `viewmodel_instance_string.cpp:10-25`), Font's two-part setter and clone
    (`viewmodel_instance_asset_font.cpp:13-86`), and DataBind's retained source
    (`data_bind.cpp:210-216`). Font tests pin the non-sentinel/sentinel and
    same/different live-pointer dirt multiplicity, data-bind null/live paths,
    and clone clearing of the private live Font. Scene no longer keeps a
    nested String payload cache. The scope is owned retained contexts;
    imported overrides, artboard target/cache values, converter operands, and
    the remaining direct-property clock union are later cuts. Fast and corpus
    evidence: runtime lib 361/361, nuxie lib 132/132, probe 708/708, ordinary
    and scripted goldens 317/317 entries / 647/647 segments with zero failures,
    C API smoke green, and `cargo test --workspace` green. Standards review
    found and fixed one unchanged-String allocation before the equality
    early-out; Standards and Spec re-reviews are clean. Renderer goldens are
    not applicable because this slice changes no renderer/draw code.
  - [x] (f8) ViewModel-valued properties now retain one structural endpoint;
    linked reads and writes traverse the retained child directly, and handle
    borrows no longer refresh or recopy a dynamic mirror. Same-child
    assignment still runs the lifecycle, while authored selection detaches an
    explicit link and reveals the untouched compatibility storage. This
    matches C++'s retained child setter and synchronous parent relink walk
    (`viewmodel_instance_viewmodel.hpp:23-35`;
    `viewmodel_instance.cpp:118-188`). The direct-property mutation clock
    remains for artboard target/cache and converter consumers. Full evidence:
    runtime lib 361/361, nuxie lib 132/132, probe 708/708, both golden modes
    317/317 entries and 647/647 exact segments with zero failures, C API smoke
    and workspace green; both closeout reviews clean. Renderer goldens were
    not applicable.
  - [x] (f9) state-machine `List`, `ListLength`, and `ViewModel` graph sources
    now retain both the exact property cell and the structural list/child
    endpoint itself instead of depending on a root mutation-generation sample
    or storing a copied structural payload in the cell. The graph derives its
    list-count read model and linked-child identity from that retained source
    after property dirt. List mutations dirty the list property, including
    same-index swaps and empty-to-empty `updateList`; ViewModel assignment
    dirties its structural endpoint; and the existing weak-parent relay pushes
    a dedicated DataContext-rebind sink through nested replacements. Explicit
    same-source binds still mark both supported directions for reconcile in
    C++ favor order. This follows C++'s value-owned dependent list, retained
    list/child ContextValues, exact list-property notification, retained
    DataBind source, and synchronous parent relink lifecycle
    (`viewmodel_instance_value.hpp:68-97`;
    `viewmodel_instance_list.cpp:26-60,76-143,183-225`;
    `context_value.cpp:133-165`; `context_value_list.cpp:17-29`;
    `context_value_viewmodel.cpp:21-41`; `data_bind_context.cpp:80-85`;
    `data_bind.cpp:210-240,502-546`;
    `viewmodel_instance.cpp:346-415`). State-machine
    `owned_view_model_candidate_generations`, the candidate mutation accessor,
    the steady-frame full-graph poll/rebind, and the trigger refresh scan are
    deleted. Candidate order remains for DataContext lookup and listener
    addressing; the artboard structural key, direct-property mutation clock,
    target/cache payloads, and converter operands remain queued. Retained-child
    pointer projections use a clone-fresh allocation identity rather than the
    semantic instance ID, matching C++ pointer-key behavior. Full evidence:
    runtime lib 363/363, nuxie lib 133/133, probe 708/708, ordinary and
    scripted goldens 317/317 entries and 647/647 exact segments with zero
    failures, C API smoke and workspace green; Standards and Spec re-reviews
    clean. Renderer goldens are not applicable because no renderer/draw code
    changed.
  - [x] (f10) owned converter operands retain their exact Number cells and
    push dirt into the bind that owns the converter. `OperationViewModel`
    reads its retained operand at conversion time; Project converter value
    paths do the same instead of rebuilding `resolved_values` snapshots.
    State-machine graph sources register operand cells on the SAME retained
    bind sink as their primary source. Artboard authored binds are not yet
    reunified, so each split converter occurrence temporarily owns a fresh
    sink; shared/property/converter-property records route directly, while
    formula/list records wake their existing bounded active pass. Clones
    register independent sinks against the shared cells. This follows C++'s
    retained converter source and outer-DataBind
    dependent registration (`data_converter_operation_viewmodel.cpp:8-27,
    48-59`), conversion-time read (`data_converter.cpp:34-55`), and queued
    bind traversal (`data_bind_container.cpp:115-203`). The old owned-context
    operand snapshot refreshers and listener-write path scanners are deleted;
    serialized snapshots remain only for default/imported contexts. The
    artboard structural key/root mutation clock still serves primary
    target/cache refresh and is the next deletion slice, together with
    reunifying all execution records for one authored DataBind. Focused
    evidence: runtime lib 370/370, nuxie lib 134/134, C++ probe 708/708,
    ordinary and scripted goldens 317/317 entries and 647/647 exact segments
    with zero failures, C API smoke green, and `cargo test --workspace` green.
    Review corrections covered source-origin routing, key-frame converter
    subscription rewiring, steady-frame operand rescans/allocations, and exact
    nested compatibility-context resolution; Standards and Spec re-reviews are
    clean. Renderer goldens are not applicable because no renderer/draw code
    changed.
  - [x] (f11) artboard authored binds now have one retained state per authored
    `data_bind_index`. That state owns the exact source, direction/origin dirt,
    shared two-way converter state, outer converter-operand subscriptions, and
    target-notification suppression. Each retained bind reports its exact
    authored occurrence into a reusable container queue, so steady source
    delivery neither scans all binds nor allocates. Source dirt reads that
    retained source directly; direct list sources run the complete occurrence-
    local adapter and component-list reconciliation, including different list
    items at an unchanged count. Exact target dispatch is indexed by
    `data_bind_index`, so a converter operand cannot wake same-path siblings.
    Reverse writes pass through the same bind and suppress their own cell echo;
    default/imported rebinding clears the old source. Only pushed
    structural-rebind dirt re-resolves ordered context candidates. The old
    shared-converter map, reverse-write sibling resolver, artboard structural
    key, component-list structural-generation poll, and complete shared
    mutation-clock family are deleted. Formula-token and converter-property
    records keep separate sinks because they are subordinate authored binds,
    not split records of the outer bind. Formula-token DataBinds own an
    independent exact primary-source queue; clone-local sinks preserve pending
    source and converter-operand dirt. `bindsOnce` registers no DataBind source
    edge unless an attached Formula (including inside a Group) owns C++'s
    independent dependency. Source-change resets walk converter/state groups
    in lockstep and clear only Formula children whose own random mode is
    sourceChange, while explicit deterministic RNG replacement still
    invalidates every Formula cache. Reconcile now preserves C++'s
    already-dirty origin guard, and source dirt is consumed immediately before
    target application so dirt created during apply relatches for the next
    pass (`data_bind.cpp:502-531`; `data_bind_container.cpp:144-147`). The
    zero-second `advanceAndApply` forcing and pending-report return semantics
    remain unchanged (`state_machine_instance.cpp:2612,2663`), as does the
    next-frame-start `applyEvents` chained-batch drain
    (`state_machine_instance.cpp:2320-2343`). Evidence: runtime lib 391/391,
    nuxie lib 140/140, and C++ probe 714/714; Standards, Spec, and converter
    re-reviews are clean. Full corpus, C API, and workspace evidence is
    recorded in the history entry below. The ordinary corpus has zero
    failures; scripted exact parity is complete with only the two permitted
    verification names (`data_viz_demo`, `db_health_tracker`). Candidate vectors remain only for
    ordered context/listener addressing; their public-seam and trigger-state
    cleanup remains before the overall (f) gate can close. Renderer goldens
    are not applicable because no renderer/draw code changed.
  - [x] (f12A) owned ViewModel transition triggers now keep counter,
    `valueChanged`, and per-layer consumption on the exact retained trigger
    cell. `StateMachineViewModelTriggerInstance` is only metadata plus either
    that retained source or an explicit copied compatibility source for the
    not-yet-canonical default/imported modes; owned bindings no longer carry
    parallel `value`/`changed`/`used_layers` state. Each state-machine layer
    occurrence receives a distinct identity token, refreshed on clone, because
    C++ keys `Triggerable::m_usedLayers` by `StateMachineLayerInstance*`; two
    machines sharing one ViewModel may therefore consume the same trigger in
    their respective layers. This follows C++'s single retained value/change
    object (`viewmodel_instance_value.cpp:59-62,131-135,176-179`), inherited
    layer-use state (`state_machine_input_instance.hpp:78-102`), and transition
    evaluation/use of the DataBind's exact source
    (`transition_viewmodel_condition.cpp:49-60`;
    `transition_property_viewmodel_comparator.cpp:50-67`). Default/imported
    canonical cell ownership, `sync_default_view_model_triggers_from_active`,
    and `reset_bound_trigger_sources` remain explicitly queued as f12B rather
    than inventing file-level sharing in this mechanical slice. Evidence:
    runtime lib 393/393, nuxie lib 140/140, C++ probe 714/714, ordinary and
    scripted goldens 317/317 entries plus 647/647 exact segments; ordinary has
    zero failures and scripted retains exactly the two permitted verification
    names (`data_viz_demo`, `db_health_tracker`). C API smoke and the full
    workspace are green. Standards and Spec re-reviews are clean.
    Renderer goldens are not applicable because no renderer/draw code changed.
  - [x] (f12B) default and imported trigger bindings now retain canonical
    file-owned cells instead of copied compatibility state. One serialized
    instance catalog belongs to each loaded `nuxie::File` and is passed into
    every root/nested `RuntimeArtboardBuildContext`; standalone raw constructors
    create an explicit fresh occurrence. Machines in one file occurrence share
    the exact graph/transition trigger cells. The artboard occurrence factory
    creates imported contexts already attached to those cells, so pre-bind
    writes from multiple contexts preserve one C++ mutation order; detached
    compatibility contexts adopt on bind and reject pre-bind trigger writes.
    Context clones copy serialized trigger payload but not dynamic changed,
    used-layer, or dependent-dirt state, while state-machine clones deep-copy
    and rebind their catalog to preserve snapshot semantics. This
    follows the pinned probe's direct `ViewModel::instance(index)` retention
    (`tools/cpp-probe/main.cpp:1267-1300,4683-4721`) and C++ transition use of
    the DataBind's retained source (`transition_viewmodel_condition.cpp:49-60`;
    `transition_property_viewmodel_comparator.cpp:50-67`). The copied/retained
    trigger-source enum, imported trigger overrides, default-trigger mirrors,
    `sync_default_view_model_triggers_from_active`,
    `reset_bound_trigger_sources`, and `reset_active_view_model_triggers` are
    gone; file reset advances each precomputed unique trigger cell once,
    including nested/list topology and cycles, while owned-context reset walks
    live list storage so inserted rows participate and removed rows do not.
    Missing paths are explicitly unbound. The
    zero-second `advanceAndApply` forcing, pending-report return term, and
    next-frame `applyEvents` chained drain remain unchanged. Evidence: runtime
    lib 399/399, nuxie lib 140/140, probe 721/721, both corpus summaries
    317/317 entries and 647/647 exact segments with zero failures, C API smoke
    and workspace green. The workspace target builds the probe and exports
    `RIVE_CPP_PROBE`; every further RB-1 cut must run the scripted gate before
    push.
    Renderer goldens are not applicable.
  - [x] (f13) listener ViewModel changes no longer rebind every owned
    candidate graph source after mutating one retained cell. The exact source
    cell now carries dirt into the next `updateDataBinds(false)` batch, matching
    `ListenerViewModelChange::perform` and `DataBind` propagation
    (`listener_viewmodel_change.cpp:42-80`; `data_bind.cpp:502-546`) without
    spuriously reconciling unrelated two-way binds. Explicit DataContext bind
    and pushed structural replacement still run the legitimate aggregate bind
    operation (`state_machine_instance.cpp:2901-2913`;
    `data_bind_container.cpp:25-35`; `data_bind_context.cpp:56-89`). The dead
    trigger-sync branch and the stale `rebind_owned_view_model_context_candidates`
    / `refresh_owned_view_model_candidates` symbols are deleted. Full evidence:
    runtime lib 401/401, nuxie lib 140/140, C++ probe 721/721, ordinary and
    scripted goldens 317/317 entries plus 647/647 exact segments with zero
    failures, C API smoke and the CI-shaped full workspace green. Candidate
    vectors, the nested-host flattening helper, and the active-trigger shadow
    path remain queued for the atomic parent-linked DataContext/source-path
    migration. Renderer goldens are not applicable.
  - [x] (f14) replaced the last candidate/shadow layer with the production
    parent-linked owned `RuntimeOwnedDataContext` in one atomic landing:
    - [x] (f14A) the carrier retains an ordered local main/global instance list
      plus one parent, resolves local before parent, and owns exact-source
      listener/write/advance/rebind lookup. Global slot keys determine input
      order only: the carrier stores no slot identity, compares the occupant's
      actual ViewModel id, and never rewrites authored paths. Resolution also
      continues to the next local/parent instance when a same-model instance
      lacks the final property, matching `data_context.cpp:265-332,397-442`.
    - [x] (f14B) every state-machine, artboard, graph, converter, listener,
      font, advance, reset, clone, public-bind, and structural-relink consumer
      now uses that context. Nested artboards and component-list rows add local
      instances over their complete parent; hosts without local instances pass
      the inherited context through unchanged (`artboard.cpp:2551-2567,
      2694-2707`; `nested_artboard.cpp:885-939`).
    - [x] (f14C) transition trigger compare/use reads the source retained by
      the bindable property's `DataBind`; `StateMachineFireTrigger` retains its
      authored path and resolves it against the current DataContext at perform
      time (`transition_viewmodel_condition.cpp:49-60`;
      `transition_property_viewmodel_comparator.cpp:50-67`;
      `state_machine_fire_trigger.cpp:7-18`). Immutable metadata alone remains
      for public trigger inspection.
    - [x] (f14D) production contains none of
      `RuntimeOwnedViewModelBindingCandidate`, `owned_view_model_candidates`,
      `artboard_owned_view_model_candidates`,
      `owned_view_model_context_candidates_for_nested_host`,
      `bind_active_owned_view_model_triggers_for_candidates`, or the active
      `StateMachineViewModelTriggerInstance` shadow. Context tests cover
      local/parent order, actual occupant identity, partial-instance fallback,
      nested/component-list inheritance, structural relink, listeners,
      transition consumption, and live fire-trigger resolution.
  - [x] (f15) closed the overall deletion gate. The compensation inventory has
    zero family survivors; runtime/nuxie libs, the 721-test C++ probe, ordinary
    and scripted 317-entry/647-segment corpora, C API smoke, the probe-armed
    CI-shaped workspace, and all 1,468 renderer rows pass. Independent
    Standards and Spec passes are clean, and scripted comparison is rerun
    immediately before the RB-1 push. #RD-1 is now unblocked.
- [ ] #B-6 structural fidelity audit (user-directed 2026-07-21, adopted
  from Anthropic's migration methodology) — sweep all 447 port-manifest
  rows comparing each C++ file's ARCHITECTURE against its Rust module:
  (a) retained identity vs copies, (b) push/dependents vs polling,
  (c) update-cycle ordering, (d) ownership/lifecycle, (e) mechanisms with
  no C++ counterpart (compensation smell: generations, rescan loops,
  mirrors, epochs). Classify ISOMORPHIC / ADAPTED (rule cited) /
  DIVERGENT (rebuild ticket like RB-1, or explicit D-row). Prereqs:
  (i) extend docs/PORTING.md with architecture-fidelity rules from RB-1 so
  reviewers cite rules; (ii) VALIDATE THE JUDGE — the audit brief must
  flag the known-bad pre-RB-1 data-binding design (calibrate at commit
  bf051718's ancestors) and pass a known-good subsystem before the
  447-row fan-out is trusted. Batch fan-out over the #B-2 manifest in
  dependency order; findings feed the ticket queue mechanically.
  SWEEP COMPLETE + TRIAGED 2026-07-21 (executor session, e528fe2b):
  447/447 rows recorded — 19 ISOMORPHIC / 182 ADAPTED / 162 DIVERGENT /
  36 UNKNOWN / 48 N/A. Planner triage (docs/b6-audit/TRIAGE.md) collapses
  DIVERGENT into: ~65 rows = RB-1 scope (keyframe data-bind graphs now
  explicitly included); ~60-70 rows = retained-renderer invalidation
  epochs → APPROVED as register D-12 (accepted architecture); RB-2
  opened (focus system, spot-verified, ties into #FT-TEXT keyboard gap);
  5 small families pending planner verification; 36 UNKNOWNs re-pass
  after RB-1. JUDGE VALIDATED 2026-07-21: caught the known-bad pre-RB1 data binds
  (independent rediscovery of the in-file compensation family), cleared
  keyed animation, and produced two binding amendments — the
  mutation-timing gate on axis (e) and the cross-file coverage clause
  with subsystem-clustered batching. Spec landed as
  docs/b6-structural-audit-spec.md; remaining: PORTING.md
  architecture-fidelity rules section, then the ~40-55 batch fan-out.
- [ ] #RD-1 renderer-feed restoration to the C++ retention boundary
  (user-directed 2026-07-21, P0 AFTER #RB-1; supersedes D-12) — see map
  Phase RD: measured spike, then lane-by-lane live-traversal migration
  with the pixel corpus as referee, ending in the scene-cache deletion
  gate. The ~60-70 #B-6 Family B rows re-open and close for good at that
  gate. RD-1a's file-corresponding lane map and exact retention boundary are
  fixed in `docs/rd1-renderer-feed-map.md`; no production code preceded it.
  RD-1b's non-default measured spike is complete and stopped at the mandatory
  user checkpoint: live per-frame command materialization measured 5.866x
  prepared Rust (+486.6%) on the seven-segment shape/image/nested slice.
  The user accepted that temporary-seam cost and authorized RD-1b2 plus
  RD-C1/RD-C2. A second measured user checkpoint is binding after C1/C2 remove
  command materialization and before any scene-cache deletion; demolition
  cannot self-clear that checkpoint.
- [ ] #B-5 editor-cutover parity audit (user-directed 2026-07-21) — scout
  report complete, 12 findings. VERDICT: broadly parity-aligned with
  isolated slips, not structurally off-course — most bytes are additive
  editor surface (project-data-converter format, event-context host APIs,
  converter build cache) plus tests. Risk concentrates in ONE refactor:
  the owned-view-model "candidate" unification (c7d48ca0), which added a
  strict value-kind match guard in `resolve_value_for_source_path`
  (kind mismatch → `bound=false` → serialized-default fallback) and new
  unresolved-source fallbacks synthesizing bindable defaults from
  serialized `propertyValue` (enum default changed 0 → u32::MAX). That
  guard/fallback pair is the prime suspect for the open scalar shift.
  Remaining (c)-class rows needing fixtures before close: (i) 974aab66's
  two-way custom-property seeding no longer target→source-seeds two-way
  binds (C++ seeds both, favored direction wins) — fixture: two-way
  TrimPath.start bind, serialized target ≠ source initial, t=0; (ii) a
  parent-relative nested number/enum bind with serialized target
  propertyValue ≠ 0 at t=0; (iii) self-referential layout recursion guard
  (mark-on-entry → deferred insert); (iv) owned trigger source paths
  deeper than 2 segments. Each becomes a repair or an explicit D-row.
- [ ] #B-1 Phase S sync to b73bc675 — triage submitted; USER-GATE blocks port/pin movement
- [ ] #B-2 port-manifest invariant — implementation/local gate complete at
  exact b73bc675 (447/447: 378 ported / 21 partial / 43 absent / 5 N/A);
  first main CI green pending
- [ ] #B-3 size re-measure — user-approved budget 9 MiB (9,437,184 B) both
  variants, fully wired (`make size-report` blocks; scorecard validates
  `size-report.json`; CI records); close on the first green main CI
  recording
- [ ] #B-4 `make parity-scorecard` — implementation/local gate complete;
  canonical five-floor evidence and CI publication wired; first main CI green
  blocked by the decoded-image policy gate and seven freshly exposed d788
  C++-probe parity assertions
- [ ] #OR-1 side-channel spec + C++ emit
- [ ] #OR-2 Rust emit + corpus-wide side-channel exact
- [ ] #OR-3 script verbs (setInput/VM-mutation/resize; key/textInput reserved)
- [ ] #OR-4 sampling densification (237 t=0-only entries)
- [ ] #OR-5 input-script coverage (all listener-bearing files)
- [ ] #OR-6 `make e2e-golden` blocking (≥50 files)
- [ ] #OR-7 differential fuzzer nightly (seeded-divergence proof, then 3 clean nights)
- [ ] #OR-8 tolerance root-cause (0.5 / 0.75 entries)
- [ ] #OR-9 blocking perf gate (≥20 files)
- [ ] #FT-ASSET asset loader seam (A1)
- [ ] #FT-TEXT text-input interaction (F2/F5/A3; blocked by #B-1)
- [ ] #FT-SCROLL scroll physics (F4)
- [ ] #FT-AUDIO audio (F1/A2; USER-GATE engine choice)
- [ ] #FT-CAPI portable surface (A4/A5/A7; 4 groups)
- [ ] #FT-PROD production-flow corpus lane (C3; USER-GATE access)
- [ ] #FT-FIXSWEEP typeKey fixtures (F10/C1/F9)
- [ ] #FT diagnostic spot-check: unported Lua binding → named diagnostic (from #LT-2)
- [ ] #HD-1 threading-model decision (USER-GATE)
- [ ] #HD-2 renderer oracle hardening (V7; adapter matrix 2/2 complete,
  current-runtime same-runner 1,468/1,468 local, Paravirtual CI rerun and
  clockwise-atomic hypothesis oracle pending)
- [ ] #HD-3 WebGL2 decision (USER-GATE)
- [ ] #HD-4 TODO(golden) pair
- [ ] #HD-5 publish the parity claim doc
- [ ] #LT-* long tail (each opens by USER-GATE)

## Next queue (top = next; orchestrator maintains)

1. #RD-1b2 renderer-feed rulebook and dual-translation stress test. The user
   accepted RD-1b's 5.866x temporary command-materialization result and
   authorized work to proceed. `r4-timing-gate` ran four unchanged-threshold
   brackets, all invalidated by its host-idle-spread fence, so no R4 ratio is
   claimed; a follow-up run is explicitly deferred to a quiet host with the
   12% fence intact. A second USER CHECKPOINT follows RD-C1/RD-C2 seam removal and
   blocks every scene-cache deletion until the new delta is reported and
   reviewed.

ARCHIVED EVIDENCE for the four scripted entries (was queue item 1;
   subsumed by #RB-1) — FOUR scripted-golden-compare
   entries broken by concurrent main `c7d48ca0` (`db_health_tracker`,
   `echo_show_demo`, `list_index_script_access`, `superbowl`: wild
   transform/gradient/path divergences under forced scripting; the default
   gate is green). Attribution proven on a pristine `5927654b` worktree.
   `c7d48ca0` touched no nuxie-scripting code, so the cause is runtime code
   behaving differently under forced scripting — start from its artboard.rs
   (+1,350 — NOTE: almost all tests; production changes are the
   owned-candidate bind refactor) / data_bind_graph.rs / instance.rs
   changes. LOCALIZED LEAD (2026-07-21, sharpened): in `db_health_tracker`,
   EVERY row-container transform in the scripted lane is shifted by exactly
   -2945.44531 versus both C++ and the pre-rebase-green Rust baseline
   (271.49→-2673.95, 582.49→-2362.95, -20.51→-2965.95, 1195.49→-1749.95;
   y identical, glyph outlines byte-identical). One scalar diverges — the
   list container's scrolled/bound x offset, most plausibly a data-bound
   number through the DataConverterInterpolator/BlendState1DViewModel
   chain evaluating differently under scripting-enabled import. Reproduce:
   run both scripted runners on the file (`--samples 0`, Rust adds
   `--execute-scripts`) and diff; everything else is LSB float noise. A
   green Rust baseline stream builds from pre-rebase commit `a159897f`
   (reflog) with `cargo build -p rust-golden-runner --features scripting`.
   BISECTION COMPLETE THROUGH FOUR STRIKES (2026-07-21): (1) the
   scripting-FEATURE runner build diverges even with `--execute-scripts`
   OFF while the featureless binary from the same tree is exact; every
   runner-side cfg difference was patched out experimentally (scripted
   preallocation, scripted-drawable init, the rebind call) with no effect —
   EXCEPT `selected_artboard_owned_view_model_context`, where the feature
   build constructs the owned main context with
   `RuntimeOwnedViewModelInstance::from_instance(runtime, vm_index, 0)`
   (serialized instance 0) and the featureless build uses `::new`
   (definition defaults); forcing `::new` in the feature build makes the
   stream exact. (2) The runner is CORRECT: C++'s scripted runner does the
   same (`File::createViewModelInstance(viewModelId, 0)` copies serialized
   instance 0) and C++ still renders 271.49 — so the runtime's handling of
   a from_instance-built owned context is at fault. (3) The #B-5 scout's
   kind-guard suspect is EXONERATED: instrumentation shows zero kind
   mismatches; `property_path_for_source_path` fails identically in both
   builds for source paths [3,0] and [2,4] (common, not the delta). (4)
   Property ordering is exonerated: both constructors share
   `from_view_model`'s definition walk; only seeded VALUES differ. (5) The
   value-table diff is DONE (dump
   `runtime_owned_view_model_binding_value_for_property_path` behind an env
   var in both builds, `sort -u`, diff): the instance-0 context resolves
   real serialized data — notably property_path [4,1] = 4022.0 where the
   default context resolves 0.0, plus [0,0..6] = 67/55/70/80/67/75/64 and
   [3,3] ≈ 410–482 vs ≈ 16 — and those values flow into the artboard,
   shifting the rows. ROOT-CAUSE HYPOTHESIS (one step from a fix): C++
   ALSO copies instance 0 yet renders 271.49, so C++'s init must seed
   target→source FIRST (the artboard's serialized state writes into the VM
   before any source→target read — `DataBind::updateSourceBinding` +
   TargetOrigin/`sourceToTargetRunsFirst` favored-direction semantics),
   while c7d48ca0's owned-candidate path in
   `state_machine/instance.rs::bind_owned_view_model_context_candidates`
   applies source→target from the live VM values. Note the artboard side
   (`bind_owned_view_model_artboard_context_candidates`) still calls
   `bind_owned_view_model_target_to_source_bindings` before value
   application — compare the state-machine candidate path against it and
   against pre-rebase `a159897f`, then port the favored-direction ordering
   from C++ `data_bind.cpp`. This also intersects the #B-5 (c)-row (i):
   974aab66 stopped target→source seeding for two-way binds.
   The commit's
   OTHER regression — ten trigger probe assertions — is FIXED 2026-07-21:
   `reset_advanced_data_context` had swapped `trigger.reset()` for
   value-retaining `advanced()`, but C++
   `ViewModelInstanceTrigger::advanced()` zeroes `propertyValue` itself;
   the revert restores 707/707 with every c7d48ca0-added test still green.
   Context: the commit was validated while `RIVE_RUNTIME_DIR` sat at
   drifted `ba2b6434`, and CI's `cargo test --workspace` skips the probe
   suite when `RIVE_CPP_PROBE` is unset, so main went red silently — the
   same class as the five `974aab66` component-list regressions.
2. #B-1 port — execute the approved S3-1 (TextInput) + S3-3 (static linking)
   port per `docs/upstream-sync-map.md`; advance `LAST_SYNCED_SHA` to
   `b73bc675` on a green ratchet. (Text-input code is outside the #RB-1
   layer; may proceed in a lane while #RB-1 holds the spine.)
3. #B-5 editor-cutover parity audit — classify every runtime-behavior hunk
   of `974aab66`/`c7d48ca0` (see Ticket checklist); most (b)/(c) rows are
   expected to dissolve into #RB-1's deletion gate.
4. #OR-1 — side-channel spec + C++ emit once the floor is restored.
5. #FT-TEXT — unblocked by the #B-1 approval; starts after the port lands.

## Pending USER-GATEs

(none — the reopened #B-3 budget was decided 2026-07-21; see the
Decisions log.)

## Decisions log

- 2026-07-20: Project created from `docs/parity-gap-register.md` (six-way
  evidence sweep). Method/threading/routing inherited from
  `.claude/commands/goal.md` culture; session protocol at
  `.claude/commands/parity.md`.
- 2026-07-20: Cycle-3 approval cut remains fixed at `b73bc675`; post-inventory
  upstream `ba2b6434` is explicitly deferred to the next inventory rather than
  silently widening the pending authorization.
- 2026-07-20: The historical 2.75 MiB size budget is not reused: it measured a
  pre-renderer artifact. #B-3 remains open until the user chooses a new metric
  and budget.
- 2026-07-21: **Decoded-image policy split approved (register D-11).**
  Low-level compatibility/golden paths retain every decoded image, exactly
  like pinned C++; the high-level `nuxie::File` path is bounded at 64 MiB
  aggregate by default via `FileImportLimits::max_retained_decoded_image_bytes`;
  `FileImportLimits::unbounded()` is truly unbounded. Only the bounded host
  mode is a deliberate divergence. No C ABI change.
- 2026-07-21: **#B-1 cycle-3 approval granted.** PORT S3-1 (`1b4df2ad`,
  TextInput) and S3-3 (`b73bc675`, static library linking); S3-2 (`079305d7`,
  profiler) deferred WATCH; both dependency WATCH rows retained at staleness 2.
  Authorization covers exactly the `b73bc675` cut; `ba2b6434` drift stays in
  the next inventory.
- 2026-07-21: **#B-3 budget decided: 8 MiB (8,388,608 B), blocking for BOTH
  scripting OFF and ON.** ON has ~52 KiB headroom today; if the approved
  TextInput port pushes ON past the budget, the gate reopens with fresh
  measurements — the constant is never silently raised.
- 2026-07-21: **#RB-1 opened (user decision): rebuild the view-model,
  data-binding, and value layers to match the C++ architecture exactly.**
  Rationale: the Rust port flattened bound sources into copied values,
  losing C++'s retained `ViewModelInstanceValue` identity and dependent
  registration; the resulting compensation family (mutation clocks,
  candidate vectors, listener rescans, alias mirrors, facade-wide rebinds)
  has produced every one of the 20+ editor-cutover parity regressions found
  2026-07-21, including the four still-red scripted entries. Point-fixing
  inside the compensations is stopped; the map gains Phase RB with a
  deletion exit gate. Editor changes to this layer freeze until it lands.
- 2026-07-21: **#RD-1 user-directed (supersedes D-12 the same day):
  restore C++'s retention boundary.** Delete the scene-level retained
  replay layer (prepared frames, command streams, path caches, epoch
  bridges); keep per-object retained resources (the C++ design).
  Performance is subordinate to design fidelity — the ratio stays
  measured; >1.0 post-RD is a user-reviewed number, not a blocker.
  Sequencing binding: after #RB-1; measured spike before demolition;
  pixel corpus is the referee. See map Phase RD.
- 2026-07-21: **#B-6 Family B user-approved as register D-12.** The
  retained-renderer invalidation epochs are accepted architecture (the
  deliberate Phase R retained-replay design's required bridge), closing
  ~60-70 DIVERGENT audit rows as documented-and-intentional, with the
  ported-information-loss guardrail recorded in the D-row.
- 2026-07-21: **#B-3 replacement budget user-approved: 9 MiB (9,437,184 B),
  both variants blocking.** The 8 MiB decision predated `974aab66`; honest
  re-measurement with the 43-root harness (OFF 7.84 MiB / ON 8.70 MiB)
  reopened the gate the same day, and the user approved the recommended
  9 MiB replacement (~3.4% headroom over ON). CI evidence recording is
  re-enabled.

## Log

- 2026-07-22 — #RD-1b measured a non-default live per-frame traversal without
  deleting any retained scene layer. Across five animated-shape samples, one
  image sample, and one nested-artboard sample, aggregate Rust hot-loop minimum
  rose from 11.533543 ms prepared to 67.659043 ms live: 5.866x, or +486.6%.
  The slice remained exact at 3/3 entries and 7/7 segments; runtime lib is
  399/399 and scripted goldens are 317/317 plus 647/647 with zero failures.
  Four `r4-timing-gate` brackets failed closed at the unchanged host-load
  fence. The rebuilt pinned Dawn live reference then passed the same-runner
  renderer corpus at 1,468/1,468 with zero divergences and zero gated cases.
  Full evidence is in `docs/rd1-measured-spike-2026-07-22.md`. Work stopped
  here for the binding user checkpoint; RD-1b2 and all demolition remained
  untouched until the user's proceed decision.

- 2026-07-22 — The user accepted RD-1b's 5.866x delta as temporary command-
  materialization overhead and authorized RD-1b2 plus RD-C1/RD-C2. A second
  measured performance checkpoint is now binding after that seam is removed
  and before scene-cache deletion. The prior seven-entry ordinary-golden
  record was traced to a `CPP_CONFIG=release` C++ runner linked against a
  differently featured `tests/out/release/librive.a`; checked-in default debug
  is 317/317 entries and 647/647 segments with zero divergences. The golden
  runner now requires pinned-SHA, exact-feature provenance for its archive.
  R4 remains explicitly deferred to a quiet host; its 12% idle-spread fence is
  unchanged. Local closeout is green: runtime 399/399, nuxie 140/140, pinned
  probe 721/721, ordinary and scripted goldens 317/317 plus 647/647 with zero
  divergences, renderer 1,468/1,468 with zero divergences/gated cases, C API
  smoke, and the probe-armed full workspace. Size is 8,267,064 bytes without
  scripting and 9,184,664 bytes with scripting, both within 9,437,184 bytes.

- 2026-07-22 — #RD-1a fixed the renderer-feed restoration sequence before
  production changes. `docs/rd1-renderer-feed-map.md` maps the pinned C++
  draw/traversal files to RD-C1..C7, records the ownership move from prepared
  scene replay to an Artboard live list with object-owned render resources,
  separates legitimate per-object caches from the deletion inventory, and
  binds every lane to renderer pixels plus both zero-failure golden gates.
  Next is the no-demolition measured spike and its mandatory user checkpoint.

- 2026-07-20 — #B-1 triage submitted for all 3 commits at `b73bc675`; pinned
  and candidate runtime ratchets localized exactly, and the session stopped
  before porting or pin movement.
- 2026-07-20 — Floor repair restored 317/647 default and scripted runtime
  exactness, migrated three stale atomic/tape renderer oracles without changing
  their contracts, and bound the adapter-dependent clippedcubic2 strict oracle
  to exact M5 Max / Apple Paravirtual references from the same historical `7c`
  revision, with provenance and a fail-closed revision-consistency check. The
  tape row's d788 static reference has a distinct path from the immutable 7c
  strict-atlas reference, so either capture workflow cannot overwrite the
  other's evidence.
- 2026-07-20 — Closeout hardening made the renderer negative control fail
  every active row: exact 0 / diverges 1,468. Ten 6×5 enum rows whose former
  32-pixel budget accepted every possible image tightened from 2/32 to 0/0;
  the real Rust/reference pairs remain byte-exact.
- 2026-07-20 — #B-2 landed the 447-row fail-closed port manifest; exact-b73
  verification reports 378 ported, 21 partial, 43 absent, and 5 N/A.
- 2026-07-20 — #B-3 remeasured the complete 42-root Darwin renderer surface
  twice with byte-identical artifacts: 7.19 MiB OFF, 7.95 MiB ON; independent
  source/root/symbol/hash audit clean; budget USER-GATE pending.
- 2026-07-20 — #B-4 landed the five-tier scorecard and bound each evidence file
  to its canonical command; unbuilt gates remain explicit rather than green.
- 2026-07-20 — #B-1 pin/candidate probing exposed stale golden-runner objects
  across `RIVE_RUNTIME_DIR` changes. The runner now rebuilds both translation
  units for every invocation, preventing upstream-header/library ABI mixing.
- 2026-07-20 — Adapter-selected static renderer references now participate in
  the same physical-alias and stream/frame/mode identity checks as primary
  references; adapter-bound stub oracles must be members of the approved set.
- 2026-07-20 — #HD-2's adapter ratchet advanced from 1/2 to 2/2 without a
  status or tolerance change. Eighty-two clockwise-atomic rows now select
  strict C++ Dawn references for Apple M5 Max and Apple Paravirtual device;
  the 28 legacy native-Metal rows were recaptured on M5 at renderer pin
  `7c778d13` / Dawn `211333b2`. Both static floors report 1,468/1,468 exact;
  the first main CI rerun is pending.
- 2026-07-20 — Main CI run `29806487036` kept the non-renderer required gates
  green but exposed nine same-runner rows: `tape` was a historical-7c versus
  product-d788 oracle skew, while the other eight localized to unconditional
  Metal `preserveInvariance` changing one-of-four edge samples on Apple
  Paravirtual. The gate now builds a separately pinned d788 C++ Dawn replay,
  including immutable dependency revisions and an exact-input cache, without
  relabeling the historical 7c oracle. Vendored wgpu now requests preserved
  invariance only for Naga-emitted invariant positions. Local M5 same-runner is
  1,468/1,468 contract-exact (1,370 byte-exact); the static floor is also
  1,468/1,468 with byte-exactness improved from 831 to 837. All five regression
  floors were green before the concurrent main change; the Paravirtual rerun
  remains pending.
- 2026-07-20 — #HD-2 was rebased intact over concurrent main `974aab66` as
  local commit `bcb6e165`. The historical 7c oracle remains immutable; the
  separately pinned d788 live replay is 1,468/1,468 on M5 with 1,370 byte-exact,
  and the static floor is 1,468/1,468 with 837 byte-exact. The commit is not
  pushed because the required aggregate floor is red.
- 2026-07-20 — Concurrent main `974aab66` introduced a global 64 MiB retained
  decoded-image ceiling. Both runtime golden gates now fail only
  `jellyfish_test`; localization proves image import and codecs are sound and
  the aggregate reservation alone suppresses images 22–23. This is a candidate
  deliberate-divergence row and is stopped at the USER-GATE above.
- 2026-07-20 — Rebuilding `tools/cpp-probe` at exact d788 fixed the apparent
  unknown-uint overflow mismatch: pinned upstream commit `296742c13` and Rust
  both consume the full uint64 fallback. The fresh oracle then exposed seven
  runtime assertions that the prior probe/test ordering had masked: five
  component-list bind cases, opacity-zero shape filtering, and an unresolved
  name-based number bind. These are floor work, not accepted divergences.
- 2026-07-21 — Three USER-GATEs decided (image-policy split, #B-1 approval,
  #B-3 8 MiB budget); see Decisions log. Register gained D-11; H1/H2 updated.
- 2026-07-21 — Image-policy split implemented: `RuntimeRenderImages` carries
  an optional aggregate budget (Default = unbounded, like C++); all low-level
  preallocation entry points stay unbounded; `nuxie::File` threads
  `FileImportLimits::max_retained_decoded_image_bytes` (default 64 MiB,
  `unbounded()` = none) into cache allocation. Both runtime golden floors are
  back to 317/317 exact / 647/647 segments with `jellyfish_test` green.
- 2026-07-21 — Seven d788 probe assertions repaired to 706/706. A pre-974
  control run (same fresh probe, parent commit e21a0ca0) proved five were
  `974aab66` regressions: bind-time component-list sizing eagerly reported
  the populated item count where C++ defers population to the advance pass;
  reverted to bind-time zero. The opacity failure was the oracle-facing
  `draw_commands()` stream retaining opacity-zero drawables C++ filters via
  live `willDraw`; the prepared-frame path now passes `include_invisible`
  (topology retained, replay filters live) while the oracle path applies
  `Shape/TextInputDrawable/Image` opacity checks at build. The name-based
  failure was Rust fabricating an unresolved-source fallback with the
  serialized default where C++ `bindFromContext` unbinds; the fallback is now
  skipped for name-based binds, mirroring the artboard-flow contract.
- 2026-07-21 — #B-3 wired: `make size-report` enforces 8,388,608 B on both
  variants and prints a parseable summary; the scorecard validates recorded
  `size-report.json` evidence against `size.budget_bytes`
  (parity-scorecard.toml) with drift and over-budget as errors; CI records
  the gate in the runtime evidence job; scorecard tests 19/19.
- 2026-07-21 — #B-3 gate fired on its first full run and REOPENED the budget:
  the fail-closed root inventory caught `974aab66`'s new public
  `Factory::make_gpu_canvas_image` (43rd root added to the audited inventory
  and consumer harness), and honest re-measurement reports 7.84 MiB OFF /
  8.70 MiB ON — ON exceeds the 8 MiB decision by 729,496 B because the
  decision was made against pre-974aab66 evidence. The constant was NOT
  raised; the CI recording step is held; the replacement budget is a pending
  USER-GATE (recommendation 9 MiB both variants).
- 2026-07-21 — Rebase over concurrent main `c7d48ca0`/`5927654b` (editor
  cutover gaps + apple runtime 0.1.2). The remote change had independently
  "fixed" jellyfish by doubling the global decoded-image cap to 128 MiB —
  the rejected arbitrary-constant option; the conflict resolved to the
  approved D-11 policy split and the three admission tests were restored to
  the 64 MiB bounded-host form. Post-rebase floors: both runtime golden
  gates 317/317 exact / 647/647, capi-smoke ok, but `cargo test --workspace`
  is 697/707 — ten NEW trigger-flavored probe regressions traced to
  `c7d48ca0` (top of the Next queue), and the scripted gate's exit status
  needs re-examination.
- 2026-07-21 — The c7d48ca0 trigger regression is repaired: one-line revert
  of `advanced()` back to `reset()` in `reset_advanced_data_context`, cited
  against C++ `ViewModelInstanceTrigger::advanced()` which zeroes
  `propertyValue` under SuppressDelegation. cpp-probe 707/707; nuxie-runtime
  lib 324/324 including every test c7d48ca0 added (its retained-value
  semantics was not load-bearing). The four scripted corpus entries remain
  red and stay at the top of the queue.
- 2026-07-21 — Upstream checkout hygiene: `/Users/levi/dev/oss/rive-runtime`
  was at post-inventory `ba2b6434` with no built libraries, so the first
  probe/runner rebuild of the day silently targeted the wrong revision. The
  checkout was restored to pin `d788e8ec` and `librive` (+text/layout
  companions) rebuilt before any floor evidence was accepted.
- 2026-07-21 — #RB-1 e5(A) ported pinned C++'s event-drain boundary:
  queued events notify before the next ordinary frame's data-bind/layer
  advance, and the old listener-only zero-time layer advance is gone.
  `echo_show_demo` flipped exact; nuxie-runtime lib 344/344 and the raised
  C++ probe floor 708/708 are green.
- 2026-07-21 — #RB-1 e5(B) closed `list_index_script_access`: nested scripts
  no longer consume one-shot init on provisional component-list rows or
  collapse occurrence identity by graph id. Retained rows initialize with
  indices 0/1/2 and pull their script mutations before first draw; the focused
  33,432-byte scripted C++ stream is exact, and the full scripted gate is now
  317/317 entries with 647/647 exact segments and zero failures.
- 2026-07-21 — #RB-1 e5(C) completed the retained-cell cutover: Scene no
  longer polls a dirty bit or rebinds contexts on writes/pointer dispatch;
  owned triggers retain their source cells through fireability and C++-style
  acknowledgment. Floors are rt lib 345, nuxie lib 132, probe 708, and both
  golden corpora 317/317 entries with 647/647 exact segments. The workspace
  push gate is held on the three stale Scene integration expectations recorded
  at the top of this file.
- 2026-07-21 — #RB-1 f1 deleted the reverse linked-child alias registry and
  mutation-time mirror fan-out. Linked properties keep their direct retained
  child and shared scalar cells; the temporary clock union remains only for
  structural/list dirt. Runtime lib stays 345/345.
- 2026-07-21 — #RB-1 f2 deleted the graph source's copied `target_origin`;
  all source kinds now use the retained DataBind direction engine as the sole
  origin latch. Runtime lib 345/345, nuxie lib 132/132, probe 708/708.
- 2026-07-21 — #RB-1 f3 deleted the unread instance-local mutation counter
  and a zero-call list-context replacement helper; the shared structural
  clock remains until retained List/ViewModel dirt lands. Runtime lib 345/345
  and probe 708/708.
- 2026-07-21 — e5(A) review correction landed in code, not expectations: raw
  state-machine advance keeps its C++ return, while all borrowed/owned/factory
  `advanceAndApply` facades force zero-second `true` and retain the pending-
  report term. Public reports and next-frame listener delivery now have
  independent lifetimes across `FlowSession::take_reported_events`. The two
  zero-return assertions are untouched; the third test cites `applyEvents`
  lines 2320-2343 and expects the listener write on the next frame. Full
  evidence: runtime lib 346/346, nuxie lib 132/132, probe 708/708, both golden
  gates 317/317 entries and 647/647 segments with zero failures, capi smoke
  green, and `cargo test --workspace` green; scripted failure list is empty.
- 2026-07-21 — #RB-1 f4 deleted the owned-candidate listener observed-copy
  scan and its bind-time fixpoint. Retained scalar cells now report every
  mutation in registration order into the state machine's next-frame queue;
  `applyEvents` swaps events and listener reports together, runs event actions
  first, and drains chained listener batches through the C++ 100-batch cap.
  Trigger-zero acknowledgment is suppressed and batch 101 remains pending.
  Review found and removed one per-report action-vector clone; both Standards
  and Spec re-reviews are clean. Full evidence: runtime lib 348/348, nuxie lib
  132/132, probe 708/708, ordinary and scripted goldens 317/317 entries plus
  647/647 exact segments with zero failures, C API smoke green, and the full
  workspace green. The scripted failure list is empty.
- 2026-07-21 — #RB-1 f6 deleted the list-edge mutation-clock union after
  landing the pinned C++ weak-parent relay lifecycle. Scalar list-row writes
  stay on the row; nested ViewModel replacement propagates through dynamic
  multi-parent and nested-list chains; removal, clone isolation, duplicate
  occurrence semantics, authored-row completion, and the pop/shift asymmetry
  have explicit tests. Component-list mounts retain row-local generations and
  consume them only after settlement/reverse writes. Review found and fixed
  one premature generation sample; Standards and Spec re-reviews are clean.
  Full evidence is runtime lib 357/357, nuxie lib 132/132, C++ probe 708/708,
  ordinary and scripted goldens 317/317 entries plus 647/647 exact segments
  with zero failures, C API smoke green, and `cargo test --workspace` green.
  Renderer goldens are not applicable because the slice changes no
  renderer/draw code.
- 2026-07-21 — #RB-1 f7A closed the linked-`AssetFont` retained-cell and
  write-delegation omission. Parent paths now share the child Font cell,
  refresh their payload snapshot on handle borrow, and delegate host, sync,
  and full data-bind writes to the retained child. The regression proves cell
  pointer identity, writes in both directions, preservation of the private
  live Font across a file-index write, and target-to-source payload
  application. It does not claim that a payload reference held across a
  separate child mutation updates live; removing that snapshot boundary is
  still part of the typed Font endpoint. Fast and corpus evidence is runtime
  lib 357/357, nuxie lib 132/132, C++ probe 708/708, and both golden modes
  317/317 entries plus 647/647 exact segments with zero failures. The scripted
  failure list is empty, C API smoke and the workspace suite are green, and
  both re-reviews are clean. Renderer goldens are not applicable.
- 2026-07-21 — #RB-1 f7B moved the complete owned String and Font payloads
  into their retained typed cells. Parent aliases and graph candidates now
  observe direct child writes without a borrow-time mirror refresh; Scene's
  nested String cache was deleted. Font mutation tests pin the C++ setter's
  one- versus two-report cases and its clone rule that preserves the file
  index while clearing the private live Font. Runtime lib is 361/361, nuxie
  lib 132/132, the C++ probe 708/708, ordinary and scripted goldens 317/317
  entries plus 647/647 exact segments with zero failures, and C API smoke is
  green. Renderer goldens are not applicable. Imported overrides, artboard
  target/cache payloads, converter operands, and the direct-property clock
  union remain explicit future cuts. The full workspace is green. Standards
  review found one steady-frame allocation before String's equality early-out;
  the allocation now occurs only after the C++-matching content comparison,
  and both Standards and Spec re-reviews are clean.
- 2026-07-21 — #RB-1 f8 deleted the dynamic linked-child structural mirror
  refresh. ViewModel-valued properties now retain one structural endpoint for
  imported-instance selection and linked-child identity; handle borrows and
  linked writes no longer retry or recopy topology, and link setup performs no
  one-time scalar/list copy. Same-child
  assignment now re-runs the lifecycle instead of taking a Rust-only equality
  early-out, and authored selection detaches an explicit link and reveals the
  untouched authored compatibility storage instead of layering over it.
  Active reads/writes, script advance, and structural graph walks now follow
  only the retained link. A held-parent regression proves a
  direct grandchild replacement is visible immediately, matching C++'s
  retained setter (`viewmodel_instance_viewmodel.hpp:23-35`) and synchronous
  replacement/relink walk (`viewmodel_instance.cpp:118-188`). The independent
  mutation clock remains because artboard
  target/cache payloads and converter operands still use it as their only
  wakeup; deleting those bumps in this slice would create stale consumers.
  Full evidence is runtime lib 361/361, nuxie lib 132/132, C++ probe 708/708,
  ordinary and scripted goldens 317/317 entries plus 647/647 exact segments
  with zero failures, C API smoke green, and the full workspace green. Both
  Standards and Spec re-reviews are clean. Renderer goldens are not applicable
  because this slice changes no renderer/draw code.
- 2026-07-22 — #RB-1 f9 replaced the state-machine candidate-generation poll
  with retained structural property dirt. `List`, `ListLength`, and
  `ViewModel` sources now retain their exact property cell plus the actual
  list/child endpoint; the cell is dirt-only and the graph derives its read
  model from the retained object. List mutations notify that cell even for a
  same-index swap or empty-to-empty update, ViewModel assignment notifies its
  endpoint, and nested replacement pushes a DataContext-rebind sink through
  the existing weak parent relays. Explicit same-source binds preserve C++'s
  bidirectional reconcile marking. The state machine no longer stores or compares
  `owned_view_model_candidate_generations`, performs a steady-frame full-graph
  rebind, or separately refreshes trigger sources. This matches C++'s
  per-value dependents and retained bind source (`viewmodel_instance_value.hpp:
  68-97`; `viewmodel_instance_list.cpp:26-60,76-143,183-225`;
  `context_value.cpp:133-165`; `context_value_list.cpp:17-29`;
  `context_value_viewmodel.cpp:21-41`; `data_bind_context.cpp:80-85`;
  `data_bind.cpp:210-240,502-546`) plus its synchronous structural parent walk
  (`viewmodel_instance.cpp:346-415`). Candidate order is retained for context
  lookup; artboard target/cache and converter consumers still require the
  direct mutation clock and remain queued. Retained-child pointer projections
  use a clone-fresh allocation identity while the semantic instance ID remains
  stable across detached copies, matching C++ pointer-key behavior. Full
  evidence is runtime lib 363/363, nuxie lib 133/133, C++ probe 708/708,
  ordinary and scripted goldens 317/317 entries plus 647/647 exact segments
  with zero failures, C API smoke and workspace green; Standards and Spec
  re-reviews clean. Renderer goldens are not applicable because the slice
  changes no renderer/draw code.
- 2026-07-22 — #RB-1 f10 replaced owned converter-operand snapshots and
  write-path rescans with retained exact-cell dependencies. Operation-ViewModel
  and Project value-path conversion read live Number cells; state-machine
  operands share their owning bind's dirt sink, while artboard's transitional
  split records use occurrence-owned sinks with clone-independent
  subscriptions (formula/list wake their existing active pass). The artboard
  target/cache clock and structural key remain for the next authored-bind
  reunification/deletion slice. Review corrections fixed source-origin
  routing, key-frame converter subscription rewiring, steady-frame operand
  rescans/allocations, and exact nested compatibility-context resolution;
  Standards and Spec re-reviews are clean. Full evidence is runtime lib
  370/370, nuxie lib 134/134, C++ probe 708/708, ordinary and scripted goldens
  317/317 entries plus 647/647 exact segments with zero failures, C API smoke
  green, and the full workspace green. Renderer goldens are not applicable.
- 2026-07-22 — #RB-1 f11 reunified each artboard authored DataBind around one
  retained state: exact source identity, direction/origin dirt, shared
  converter state, outer converter operands, and self-notification
  suppression. Exact source dirt now arrives through a reusable occurrence-
  indexed queue and updates only that `data_bind_index`'s target adapters,
  without a candidate rescan or same-path sibling fan-out. Direct list dirt
  runs the full retained adapter/component-list update, so a different same-
  count list replaces the old row occurrences. Target-to-source writes use
  that same state, and pushed
  structural rebind is the only path-resolution wakeup. The artboard
  structural key, component-list generation poll, shared-converter map,
  reverse-write sibling scan, and full mutation-clock family are deleted.
  C++ ordering corrections preserve already-pending dirt origin, consume
  source dirt at the apply boundary, retain zero-second facade forcing and
  pending-report returns, and leave next-frame chained listener delivery
  unchanged. Subordinate Formula-token binds retain exact primary-source and
  converter-operand dirt across clones, skip the DataBind source edge for
  non-Formula `bindsOnce`, and reset only each Formula child whose own random
  mode is sourceChange; explicit deterministic RNG replacement remains a broad
  cache reset. Standards, Spec, and converter re-reviews are clean. Evidence is
  runtime lib 391/391, nuxie lib 140/140, C++ probe 714/714, and both golden modes at
  317/317 entries plus 647/647 exact segments. Ordinary has zero failures;
  scripted retains exactly the two permitted verification names
  (`data_viz_demo`, `db_health_tracker`) and no new failure. C API smoke and
  the full workspace are green. Renderer goldens are not applicable.
- 2026-07-22 — #RB-1 f12A removed the owned-trigger parallel state copy.
  Retained trigger cells now own the fire counter, changed flag, and C++
  `Triggerable` layer-use set; every Rust layer occurrence has a clone-fresh
  identity token so two machines sharing one source do not collide at the same
  numeric layer index. Default/imported contexts intentionally remain copied
  compatibility sources pending f12B's file-level canonical-cell ownership
  decision, so their mirror/reset helpers remain queued. Full evidence is
  runtime lib 393/393, nuxie lib 140/140, C++ probe 714/714, both golden modes
  at 317/317 entries plus 647/647 exact segments, scripted limited to
  `data_viz_demo` and `db_health_tracker`, C API smoke green, and the full
  workspace green. Renderer goldens are not applicable.
- 2026-07-22 — #RB-1 f12B canonicalized default/imported trigger ownership.
  A loaded-file catalog now supplies the exact serialized-instance cells to
  every root/nested artboard's graph sources and transition metadata; explicit
  imported contexts created by the artboard occurrence retain those cells
  immediately, while state-machine clones detach and rebind their catalog for
  snapshot isolation. Multiple pre-bind contexts therefore mutate one
  canonical occurrence in call order. Detached compatibility contexts adopt
  only on bind and reject pre-bind trigger writes. Adoption validates and
  preserves nested/list aliases and cycles transactionally; context clones
  copy payload but not C++ dynamic changed/use/dirt state. Missing paths remain
  unbound. The copied trigger variant, imported trigger
  overrides, default mirrors, and sync/reset compensation helpers are deleted.
  The user-corrected zero-second facade forcing, pending-report return, and
  next-frame `applyEvents` semantics remain unchanged. Full evidence is runtime
  lib 399/399, nuxie lib 140/140, C++ probe 721/721, ordinary and scripted
  corpus summaries 317/317 entries plus 647/647 exact segments, scripted
  limited to `data_viz_demo` and `db_health_tracker`, C API smoke and workspace
  green. Renderer goldens are not applicable.
- 2026-07-22 — #RB-1 f12B closeout repair restored the scripted floor before
  any further deletion cut. `f8422eec` had filtered imported
  converter-property DataBinds through the outer artboard authored-occurrence
  table, so converter-owned RangeMapper/Interpolator/OperationValue chains
  never received their subordinate source updates. The repair keeps those
  bindings in the converter's own path-driven queue, matching
  `data_bind.cpp:94-100` and `data_bind_container.cpp:86-112`; an outer
  `data_bind_index` can no longer enqueue them. Focused `data_viz_demo` and
  `db_health_tracker` comparison is exact. Full evidence is runtime lib
  399/399, nuxie lib 140/140, C++ probe 721/721, ordinary and scripted goldens
  317/317 entries plus 647/647 exact segments with zero failures, C API smoke,
  and the full workspace green. `cpp-oracle-workspace-tests` now depends on
  `cpp-probe`, exports `RIVE_CPP_PROBE`, and its own log proves the 721-test
  suite ran. The zero-second facade return and next-frame chained
  `applyEvents` assertions remain unchanged. Renderer goldens are not
  applicable because no renderer/draw code changed.
- 2026-07-22 — #RB-1 f13 removed listener-triggered whole-context rebinding.
  Retained listener writes now dirty only their exact source/paired target;
  explicit DataContext binds and pushed structural relinks keep the aggregate
  bind path. The dead trigger-sync branch and both stale polling/rebind helper
  names are gone. Full evidence is runtime lib 401/401, nuxie lib 140/140,
  C++ probe 721/721, ordinary and scripted goldens 317/317 entries plus
  647/647 exact segments with zero failures, C API smoke and the CI-shaped
  workspace green. The remaining RB-1 cut is the parent-linked owned
  DataContext plus canonical transition/fire-trigger source path.
- 2026-07-22 — #RB-1 remainder audit mapped the ambiguous final cut into f14
  (one atomic production DataContext/source-path migration with A-D internal
  checkpoints) and f15 (the actual deletion/evidence closure gate). Code
  inventory confirms every prior compensation family is absent; the remaining
  production surface is candidate/context routing in `artboard.rs`,
  `artboard_data_bind.rs`, `data_bind_graph.rs`, and
  `state_machine/instance.rs`, plus the active ViewModel-trigger cell route in
  `state_machine.rs`, `state_machine/transition_conditions.rs`, and
  `state_machine/instance.rs`. The converter-chain repair and probe-armed
  workspace target had landed after the reported red range in `74b6e1fb`.
  Current-head verification re-ran the two f8422eec regression entries
  individually exact, then completed
  `make scripted-golden-compare` at 317/317 entries and 647/647 segments with
  zero failures. `make cpp-oracle-workspace-tests` also passed and its log
  explicitly ran all 721 C++ probe tests. The complete battery also passed at
  runtime 401/401, nuxie 140/140, ordinary 317/647, renderer 1,468/1,468,
  C API smoke, and the full workspace. RB-1 remains open until f14 and f15 are
  complete.
- 2026-07-22 — #RB-1 f14/f15 completed the production DataContext migration
  and closed the deletion track. One parent-linked `RuntimeOwnedDataContext`
  now replaces the candidate-routing and active-trigger-shadow layers across
  artboards, state machines, graphs, converters, listeners, fonts, structural
  relinks, nested hosts, and component lists. Resolution uses the actual
  occupant ViewModel identity (not the authored slot key), searches local
  before parent, and continues past a partial same-model instance when the
  final property is absent, matching the pinned C++ DataContext walk.
  Transitions retain the last non-ToSource DataBind source; fire actions retain
  the authored path and resolve the live DataContext when performed, including
  C++'s relative-resolver rule. The zero-second facade forcing and pending
  report return term remain intact, and next-frame `applyEvents` still drains
  chained listener notifications to completion within that advance. Standards
  and Spec reviews are clean, and the deleted-name inventory has zero
  production hits. Final evidence is runtime lib 399/399, nuxie lib 140/140,
  C++ probe 721/721, ordinary and scripted goldens at 317/317 entries plus
  647/647 exact segments with zero failures (including exact `data_viz_demo`
  and `db_health_tracker`), C API smoke, the probe-armed full workspace, and
  renderer goldens 1,468/1,468 with zero divergences or gated cases. All
  f1-f15 slices are complete; #RB-1 is closed and #RD-1 is next.
