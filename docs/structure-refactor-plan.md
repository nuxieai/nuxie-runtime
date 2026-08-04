# Structure scout: large-file splits and nuxie-only seam

## Scope and baseline

This is a read-only structure analysis. It proposes no behavior, source, parity-ledger, or ownership-ledger changes.

The inspected worktree was already detached at `e8726db8ffc689d3b19f1c0a55794aa6daf9956d` (`Merge pull request #246 from TheOneLevin/feat/b6-0058-listener-viewmodel-change`, 2026-08-04). The local `origin/main` ref resolves to the same commit. The requested `git fetch origin` and redundant `git checkout --detach origin/main` could not update the shared worktree metadata because the sandbox cannot write the parent repository's `.git/worktrees/nuxie-mr-c12` directory. The analysis is therefore pinned to the locally available `origin/main` commit above, not a newly fetched remote ref. The pre-existing untracked `vtriage-capture/` directory was left untouched.

The two target files have grown beyond the sizes in the request:

| File | Current lines | Production/support | Tests |
| --- | ---: | ---: | ---: |
| `crates/nuxie-runtime/src/artboard.rs` | 23,374 | 12,039 | 11,335 (`205` tests) |
| `crates/nuxie-runtime/src/state_machine/state_machine_instance.rs` | 21,674 | 14,079 | 7,595 (`94` tests) |

The safest split mechanism is literal `include!` fragments, not Rust child modules. An included fragment remains in the original semantic module (`crate::artboard` or `crate::state_machine::state_machine_instance`), so private-name resolution, inherent method paths, sibling access, and demangled symbols remain unchanged. A `mod build;`/`#[path = ...] mod build;` split would introduce new privacy boundaries and make a supposedly mechanical PR into an API refactor.

---

## 1. Large-file split plan: `artboard.rs`

### Current structural map

| Lines | Region |
| ---: | --- |
| 1-106 | Imports and supporting aliases |
| 107-135 | Draw-frame counter and generated `Mat2D` helper |
| 136-160 | `ExternalFontAssetError` |
| 161-181 | `RuntimeArtboardInstanceIdentity` |
| 182-231 | `RuntimeScriptState` and implementations |
| 232-250 | Advancing-component and persistent-dirt fixtures |
| 251-298 | Script advance/update mode enums and implementations |
| 299-327 | Resetting/runtime-component helper types |
| 328-334 | `RuntimeTextStyleFeatureOption` |
| 335-523 | `ArtboardInstance` storage |
| 524-714 | `Clone for ArtboardInstance` |
| 715-739 | `Drop for ArtboardInstance` followed by the existing leaf `include!` declarations |
| 740-773 | Occurrence and frame-advance report types |
| 774-844 | Build context/index/text-flag helpers |
| 845-860 | `RuntimeNestedAnimationInstance` |
| 861-11,133 | Main `impl ArtboardInstance` |
| 11,134-11,147 | Focus-scroll path and focus-bounds transform helper |
| 11,148-11,187 | Small follow-up `impl ArtboardInstance` |
| 11,188-11,226 | Component-list reset helper |
| 11,227-11,595 | `impl RuntimeNestedArtboardInstance` |
| 11,596-11,611 | `impl RuntimeNestedAnimationInstance` |
| 11,612-12,039 | Free construction/nested-artboard helpers |
| 12,040-23,374 | Single `#[cfg(test)] mod tests` (`205` tests) |

The main implementation itself divides along stable responsibility boundaries:

| Lines | Logical responsibility |
| ---: | --- |
| 861-1,104 | Lifecycle, data context, and transient-clone restoration |
| 1,105-2,440 | Build occurrence relationships and constructors |
| 2,441-2,712 | Path/hit initialization and external assets |
| 2,713-3,399 | Scripting lifecycle |
| 3,400-3,989 | Object/component/audio/hit/graph/definition access |
| 3,990-4,152 | Scripted interpolators and state-machine definition selection |
| 4,153-4,480 | Dimensions, world/layout bounds, scrolling, and focus reveal |
| 4,481-5,229 | Generated property façade and text values |
| 5,230-5,568 | Animation/state-machine/view-model occurrences |
| 5,569-6,312 | Component-list lifecycle and layout |
| 6,313-7,408 | Root/nested frame advance and retained-component dispatch |
| 7,409-7,585 | Nested-layout frame propagation |
| 7,586-8,496 | Transforms, epochs, dirt, and invalidation |
| 8,497-9,191 | Update pass and five-pass state-machine settlement |
| 9,192-9,825 | Component update dispatch and script scheduling |
| 9,826-11,133 | Property-change callbacks, nested controls, and collapse |

### Proposed physical layout

Keep `artboard.rs` as the owner. It retains imports, supporting types, `ArtboardInstance`, `Clone`, `Drop`, the existing leaf includes, occurrence/report/build helper types, and an ordered series of literal includes. This preserves declaration order where it matters and makes all fragments part of `crate::artboard`.

```text
src/
  artboard.rs                         owner/types, existing leaf includes, ordered include! list
  artboard/
    build.rs                          current 861-2712
    script_runtime.rs                 current 2713-3399
    access.rs                         current 3400-3989
    definitions_and_layout.rs         current 3990-4480
    properties.rs                     current 4481-5568
    component_lists.rs                current 5569-6312
    advance.rs                        current 6313-7585
    dirt.rs                           current 7586-8496
    update.rs                         current 8497-9825
    callbacks.rs                      current 9826-11133
    nested.rs                         current 11134-12039
    tests.rs                          body of current tests module
```

The production fragments are intentionally in execution/dependency order rather than alphabetical order. `tests.rs` should be included inside the original test module:

```rust
#[cfg(test)]
mod tests {
    include!("artboard/tests.rs");
}
```

This retains the test module's name and access. It also gives the attribution checker a deliberately exempt `tests.rs` leaf.

### Lockstep correspondence and checker manifest

The following table is the required per-file edit manifest. “B6 rows” are `[[files]]` entries in `file-correspondence-manifest.toml`; add the fragment's path to those rows' semicolon-separated `rust_module` values and update their C17 retained-module notes. Keep status, upstream pins, C++ paths, verification text, and correspondence claims unchanged. The row list is conservative: a later provenance cleanup can narrow it, but narrowing must not be mixed into this mechanical split.

| New file | B6 rows that must name it | Frame-loop/checker bindings that must move with it |
| --- | --- | --- |
| `artboard/build.rs` | `B6-0001`, `B6-0094`, `B6-0115`, `B6-0117`, `B6-0123`, `B6-0125`, `B6-0129`, `B6-0131`, `B6-0142`, `B6-0146`, `B6-0202`, `B6-0258`, `B6-0303`, `B6-0304`, `B6-0305`, `B6-0331`, `B6-0361`, `B6-0362`, `B6-0365`, `B6-0368`, `B6-0385`, `B6-0388`, `B6-0399`, `B6-0405`, `B6-0408` | Repoint `summarize_trace.py` dependency-build and IK-chain-build source scans when their anchors land here. Repoint matching `runtime-frame-loop-gaps.toml` owner-boundary entries. |
| `artboard/script_runtime.rs` | `B6-0077`, `B6-0194`, `B6-0200`, `B6-0322`, `B6-0326` | Repoint the script-lifecycle citations in `runtime-frame-loop-ownership.toml` and any corresponding synthetic test fixture paths. |
| `artboard/access.rs` | `B6-0094`, `B6-0095`, `B6-0123`, `B6-0146`, `B6-0203`, `B6-0204`, `B6-0205` | Repoint any live-draw/renderer-interface lifecycle citations whose named methods move here. |
| `artboard/definitions_and_layout.rs` | `B6-0077`, `B6-0094`, `B6-0258`, `B6-0303`, `B6-0304`, `B6-0305`, `B6-0385`, `B6-0388` | Repoint focus-retained-tree and layout lifecycle citations, plus matching owner-boundary allow entries. |
| `artboard/properties.rs` | `B6-0067`, `B6-0077`, `B6-0094`, `B6-0194`, `B6-0200`, `B6-0352`, `B6-0355`, `B6-0385`, `B6-0388`, `B6-0399`, `B6-0404`, `B6-0405`, `B6-0408` | `B6-0067` currently has only one Rust module: **replace** `artboard.rs` with this fragment rather than adding a second path. Repoint property/text lifecycle citations and corresponding owner-boundary entries. |
| `artboard/component_lists.rs` | `B6-0095`, `B6-0258`, `B6-0303`, `B6-0304`, `B6-0305` | Repoint component-list and nested-layout lifecycle citations and matching test fixtures. |
| `artboard/advance.rs` | `B6-0001`, `B6-0059`, `B6-0061`, `B6-0063`, `B6-0064`, `B6-0077`, `B6-0094`, `B6-0095`, `B6-0194`, `B6-0200`, `B6-0258`, `B6-0303`, `B6-0304`, `B6-0305`, `B6-0322`, `B6-0326`, `B6-0388` | In `runtime-frame-loop-ownership.toml`, move `artboard.advance`'s `rust_file` and the lifecycle citations for artboard/nested advance. Repoint `summarize_trace.py` advancing/resetting dispatch scans as applicable. Repoint relevant gap allow rows and `test_check.py` advance fixtures. |
| `artboard/dirt.rs` | `B6-0094`, `B6-0123`, `B6-0258`, `B6-0331`, `B6-0352`, `B6-0355`, `B6-0357`, `B6-0361`, `B6-0362`, `B6-0365`, `B6-0368`, `B6-0385`, `B6-0388`, `B6-0399`, `B6-0405`, `B6-0408` | Move `component.dirt`, `component.dependents` (`pub fn add_dirt(`), and transform/dirt lifecycle citations as appropriate. Repoint `summarize_trace.py`'s two add-dirt anchors, dirt consumptions, owner resolutions, and skin-buffer scans according to their final physical locations. Repoint matching gap rows. |
| `artboard/update.rs` | `B6-0001`, `B6-0094`, `B6-0115`, `B6-0117`, `B6-0123`, `B6-0125`, `B6-0129`, `B6-0131`, `B6-0142`, `B6-0194`, `B6-0200`, `B6-0203`, `B6-0204`, `B6-0205`, `B6-0258`, `B6-0303`, `B6-0304`, `B6-0305`, `B6-0322`, `B6-0326`, `B6-0331`, `B6-0352`, `B6-0355`, `B6-0357`, `B6-0361`, `B6-0362`, `B6-0365`, `B6-0368`, `B6-0385`, `B6-0388`, `B6-0399`, `B6-0404`, `B6-0405`, `B6-0408` | Move `component.update_order` (`fn update_components_with_hook_recording`) and `artboard.update_pass` (`pub fn update_pass(`) `rust_file` values and lifecycle citations. Repoint retained update/settlement gap rows, trace scans, and test fixtures. |
| `artboard/callbacks.rs` | `B6-0059`, `B6-0061`, `B6-0063`, `B6-0064`, `B6-0077`, `B6-0094`, `B6-0123`, `B6-0125`, `B6-0129`, `B6-0131`, `B6-0142`, `B6-0194`, `B6-0200`, `B6-0258`, `B6-0303`, `B6-0304`, `B6-0305`, `B6-0331`, `B6-0352`, `B6-0355`, `B6-0357`, `B6-0361`, `B6-0362`, `B6-0365`, `B6-0368`, `B6-0385`, `B6-0388`, `B6-0399`, `B6-0404`, `B6-0405`, `B6-0408` | Change `check_layout_style_handlers.py`'s `RUST_ARTBOARD_MODULE` to this fragment if both display-change markers remain here; otherwise make the checker accept the two explicit physical files. Repoint callback lifecycle citations, gap rows, and fixtures. |
| `artboard/nested.rs` | `B6-0059`, `B6-0061`, `B6-0063`, `B6-0064`, `B6-0077`, `B6-0094`, `B6-0095`, `B6-0258`, `B6-0303`, `B6-0304`, `B6-0305` | Repoint nested-artboard advance/layout citations, gap allow entries, and nested fixtures. |
| `artboard/tests.rs` | None. `rust_attribution.py` excludes a source whose stem is `tests`. | Move the artboard test body unchanged. Update the seven relative `include_bytes!`/fixture paths currently near old lines 19,951, 20,001, 20,168, 20,542, 20,597, 20,612, and 23,311 because the file becomes one directory deeper. Prefer `concat!(env!("CARGO_MANIFEST_DIR"), "/...")` for fixture stability. The `env!` use near old line 14,290 remains valid. |

#### Global artboard bindings

These bindings span more than one fragment and therefore belong in the same PR:

- `rust_attribution.py` discovers all production `.rs` files below `crates/nuxie-runtime/src`. Every new production fragment must be listed in at least one `file-correspondence-manifest.toml` row and must **not** be added to `rust-additions.toml`. The checker exempts `tests.rs`, names ending in `_tests`, and files below a `tests/` path.
- The manifest's scatter ratchet is already at its maximum: 155 rows currently have multiple Rust modules. Except for `B6-0067`, all affected artboard rows are already multi-module. Replacing the `B6-0067` root path prevents the split from increasing that count.
- Update every affected C17 note that enumerates retained Rust modules. Do not change the underlying parity classification merely because the code's physical address changed.
- `docs/runtime-frame-loop-ownership.toml` has artboard citations in `focus.retained_tree`, `component.identity`, `component.dirt`, `component.dependents`, `component.update_order`, `component.transforms`, `component.clone_drop`, `artboard.advance`, `artboard.update_pass`, `nested_artboard.advance`, `artboard.live_draw`, and `renderer.interface_boundary`. Keep `component.clone_drop` and its `pub struct ArtboardInstance` anchor on the root; change each other `rust_file` or `rust:path:line-range` to its physical fragment.
- `docs/runtime-frame-loop-gaps.toml` has 25 owner-boundary allow entries pointing at `artboard.rs` (entries 0-7, 11-22, and 29-34 in current order). Repoint each to the fragment that contains its anchor. A literal move should preserve its token-based `site_hash`; a changed hash is a signal that the split was not mechanical.
- `tools/runtime-frame-loop-port/summarize_trace.py` hard-codes `artboard.rs` nine times for add-dirt, dirt consumption, owner resolution, dependency build, skin rebuild, advancing dispatch, resetting dispatch, and IK-chain discovery. Give each query its actual fragment path. The demangled owner should remain `artboard::ArtboardInstance` because `include!` does not introduce a child module.
- `tools/runtime-frame-loop-port/check_layout_style_handlers.py` hard-codes the artboard source while looking for `propagate_layout_component_display_changed` and the display property key. Point it to the fragment(s) containing those markers.
- `tools/runtime-frame-loop-port/test_check.py` contains at least 14 exact artboard paths covering probe gating, layer occurrence, live-advance ratchets, registry site hashes/substitution, and trait-anchor behavior. Update each synthetic fixture to the fragment that owns the tested anchor; do not perform a blind string replacement.
- Leave the existing root-level includes (`nested_artboard.rs`, `artboard_component_list.rs`, `artboard_list_map_rule.rs`, `artboard_referencer.rs`, `bindable_artboard.rs`, `nested_artboard_layout.rs`, `nested_artboard_leaf.rs`, `component_origin.rs`, bone/weight, profiler/profile) where they are. Moving them under `artboard/` changes relative include resolution and correspondence ownership.
- Keep `crates/nuxie-runtime/src/lib.rs`'s `mod artboard;` unchanged. The public and crate-private paths must not acquire an extra `artboard::<fragment>` segment.

### Mechanical PR sequence and proof

One PR should contain only this file's split and the path/anchor bookkeeping above. Recommended commit sequence within that PR:

1. Extract `tests.rs`, fix only its fixture paths, and prove the same 205 tests are collected.
2. Extract production regions in original order using `include!`; make no renames, visibility edits, formatting sweeps, or logic edits.
3. Update correspondence C17/path fields and all frame-loop physical anchors.
4. Update checker source locations and synthetic fixtures, including an assertion that demangled ownership remains `artboard::ArtboardInstance`.

Proof battery:

```text
make rust-sources-fresh
make cpp-probe
make runtime-frame-loop-port-test
make runtime-frame-loop-port-check
make rust-attribution-check
cargo check --workspace
cargo test -p nuxie-runtime
cargo test -p nuxie --features scripting
make scripted-golden-compare
make silver-corpus-test
```

The PR is structure-preserving only if parity row counts/statuses, trace landmarks, owner-boundary hashes, test counts, and golden/silver outputs are unchanged.

---

## 2. Large-file split plan: `state_machine_instance.rs`

### Current structural map

| Lines | Region |
| ---: | --- |
| 1-93 | Imports and ownership commentary |
| 94-255 | Nested event-chain/notify phases, trace structures, and trace recording |
| 256-312 | Semantic-node resolver and audio/event seam types |
| 313-1,271 | `RuntimeHitResult`, `HitComponent`, and hierarchy implementations |
| 1,272-1,467 | Nested notifier, focus, semantic, and state helper types |
| 1,468-1,474 | `RuntimeDataContextBindError` |
| 1,475-1,711 | `StateMachineInstance` storage |
| 1,712-1,744 | Data-bind occurrence/pointer/executor structures |
| 1,745-2,147 | Listener-action executor implementations |
| 2,148-2,335 | `Clone for StateMachineInstance` |
| 2,336-2,356 | `Drop for StateMachineInstance` |
| 2,357-2,638 | Free helpers and view-model listener types |
| 2,639-13,938 | Main `impl StateMachineInstance` |
| 13,939-13,953 | `runtime_owned_font` helper |
| 13,954-14,079 | `view_model_listener_tests` (`1` test) |
| 14,080-21,674 | `scripted_listener_action_tests` (`93` tests) |

The main implementation divides as follows:

| Lines | Logical responsibility |
| ---: | --- |
| 2,639-2,932 | Teardown, enrollment, focus manager, clone rebind |
| 2,933-3,741 | Construction and listener/layer initialization |
| 3,742-5,102 | Scripting, hydration, and adoption |
| 5,103-6,007 | State/input/focus/semantic/keyboard/gamepad operations |
| 6,008-7,487 | Hit/listener/pointer pipeline |
| 7,488-8,525 | Event notification, deferred callbacks, listener actions |
| 8,526-9,100 | Direct bind targets and value readers |
| 9,101-10,748 | Default/imported/owned source setters and relinking |
| 10,749-12,599 | Data-context bind/rebind, listener cells, bind updates |
| 12,600-13,938 | Reports, advance/apply, nested event dispatch, settlement |

### Proposed physical layout

```text
src/state_machine/
  state_machine_instance.rs               owner/types, hit hierarchy, lifecycle, ordered include! list
  state_machine_instance/
    construction.rs                       current 2639-3741
    scripted_objects.rs                   current 3742-5102
    input_focus.rs                        current 5103-6007
    pointer.rs                            current 6008-7487
    events_actions.rs                     current 7488-8525
    bind_targets.rs                       current 8526-9100
    bind_sources.rs                       current 9101-10748
    data_context.rs                       current 10749-12599
    advance.rs                            current 12600-13953
    tests/
      view_model_listener.rs              body of current first test module
      scripted_listener_actions.rs         body of current second test module
```

Retain the two existing module names in the root and include only their bodies. This preserves test paths and any name-sensitive filtering:

```rust
#[cfg(test)]
mod view_model_listener_tests {
    include!("state_machine_instance/tests/view_model_listener.rs");
}
```

Use the same pattern for `scripted_listener_action_tests`.

### Lockstep correspondence and checker manifest

| New file | B6 rows that must name it | Frame-loop/checker bindings that must move with it |
| --- | --- | --- |
| `state_machine_instance/construction.rs` | `B6-0077`, `B6-0194`, `B6-0195`, `B6-0196`, `B6-0200`, `B6-0310` | Repoint construction/enrollment lifecycle citations and matching synthetic fixtures. Keep `state_machine.collections`' `pub struct StateMachineInstance` anchor on the root. |
| `state_machine_instance/scripted_objects.rs` | `B6-0077`, `B6-0175`, `B6-0194`, `B6-0195`, `B6-0196`, `B6-0200` | Repoint scripting/hydration lifecycle citations and scripted-action fixture paths. |
| `state_machine_instance/input_focus.rs` | `B6-0077`, `B6-0083` | Repoint focus/semantic/keyboard/gamepad citations and tests. |
| `state_machine_instance/pointer.rs` | `B6-0077`, `B6-0083` | Repoint hit/pointer/listener lifecycle citations and FL-C4/layer/hit fixtures. |
| `state_machine_instance/events_actions.rs` | `B6-0058`, `B6-0077`, `B6-0440` | Repoint event reporting, firing/bubbling, and action-dispatch citations/fixtures. `state_machine.actions`' primary anchor remains on the root if the listener-action executor stays there. |
| `state_machine_instance/bind_targets.rs` | `B6-0058`, `B6-0194`, `B6-0196`, `B6-0200`, `B6-0440` | Repoint bind-target and view-model listener citations/fixtures. |
| `state_machine_instance/bind_sources.rs` | `B6-0058`, `B6-0175`, `B6-0194`, `B6-0195`, `B6-0196`, `B6-0200`, `B6-0440` | Repoint source/relink/data-converter citations and bind fixtures. |
| `state_machine_instance/data_context.rs` | `B6-0058`, `B6-0077`, `B6-0194`, `B6-0195`, `B6-0196`, `B6-0200`, `B6-0440` | Repoint data-context lifecycle citations and listener-cell/bind fixtures. |
| `state_machine_instance/advance.rs` | `B6-0077`, `B6-0194`, `B6-0195`, `B6-0196`, `B6-0200`, `B6-0310`, `B6-0440` | Move `state_machine.advance`'s `rust_file` and `advance_with_report_mode` anchor here. Repoint state-machine event/action/reporting lifecycle citations and advance/keyframe fixtures. Include `runtime_owned_font` in this fragment to avoid an artificial one-function file. |
| `state_machine_instance/tests/view_model_listener.rs` | None; it resides below a `tests/` path. | Move the body unchanged and preserve the outer module name. |
| `state_machine_instance/tests/scripted_listener_actions.rs` | None; it resides below a `tests/` path. | Move the body unchanged. Replace or correct the relative fixture path near old line 19,011 (`../../../../fixtures/sync/vm_listener_fire_event.riv`) because the source becomes two directories deeper; prefer a `CARGO_MANIFEST_DIR`-anchored path. |

#### Global state-machine bindings

- All production fragments must be named by `file-correspondence-manifest.toml`, never `rust-additions.toml`. Every affected SMI row is already multi-module, so this split does not increase the manifest's 155-row scatter count.
- Update C17 retained-module notes for `B6-0058`, `B6-0077`, `B6-0083`, `B6-0175`, `B6-0194`, `B6-0195`, `B6-0196`, `B6-0200`, `B6-0310`, and `B6-0440`; leave the substantive parity assertions untouched.
- `docs/runtime-frame-loop-ownership.toml` cites this file from `focus.retained_tree`, `state_machine.collections`, `state_machine.advance`, `state_machine.events`, `state_machine.actions`, and `event.reporting`. Keep root anchors for the struct, stored event collection, and action executor. Repoint method/lifecycle citations to physical fragments.
- `tools/runtime-frame-loop-port/check.py` currently treats one hard-coded SMI file as the nested-event owner and excludes it from outside-owner scans. Once methods are in included fragments, a path-only exclusion would falsely flag the fragments. Make owner discovery include-aware: recursively expand literal local `include!("...")` files in source order for export analysis, and treat every expanded file as the same logical owner. Do not broaden the exclusion to a directory glob.
- Add checker unit coverage for the include-aware logical owner: an exported nested-event method in an included fragment must be accepted, while the same method in an unrelated file must still fail. Preserve syntax parsing and duplicate-anchor detection.
- `tools/runtime-frame-loop-port/test_check.py` has at least 14 exact SMI references, plus constructed/split path strings, covering FL-C4, lifecycle, layer, hit, bind, events, firing/bubbling, advance, owner exports, keyframes, focus, and semantic behavior. Point each fixture to the actual fragment; do not simply substitute the root path everywhere.
- `docs/runtime-frame-loop-gaps.toml` has no current SMI owner-boundary allow row to migrate.
- Keep `crates/nuxie-runtime/src/state_machine.rs`'s `pub(crate) mod state_machine_instance;` and its re-exports unchanged. Literal includes keep all symbols at the original module path.

### Mechanical PR sequence and proof

This should be a second, independent PR after the artboard split has landed. It uses the same proof battery. The checker must first learn the logical-owner/include relationship in the same PR, because moving the methods without that update creates false ownership failures. Apart from that narrowly required path-awareness, no checker policy should change.

Success means the same 94 tests are collected, no exported API path changes, the correspondence scatter count stays at 155, no new owner-boundary allowance appears, and the full battery shown in the artboard plan remains green.

---

## 3. Nuxie-only seam inventory

### Classification method

The authoritative starting point is correspondence metadata:

- Production Rust paths mapped to upstream C++ in `file-correspondence-manifest.toml` are port-side unless a smaller product-only region can be proven.
- Production Rust paths listed in `rust-additions.toml` are intentional Rust-only additions. The relevant ownership labels are `flowsession-abi`, `scene-api`, and generated/codegen infrastructure.
- `build.rs` is outside the attribution scanner's `src/` roots, so it must be inspected manually.
- A convenience façade is not automatically product-side. Thin Rust-only namespace/generated-storage coordinators that serve the port stay with the port even though no one C++ file corresponds to them.

This prevents two opposite mistakes: burying product authoring features inside the port forever, and extracting Rust implementation infrastructure merely because its file has no one-to-one C++ source.

### Product features and mixed glue

| Region | Current size and location | Runtime internals reached / extraction surface | Placement assessment |
| --- | --- | --- | --- |
| FlowSession wire/domain model | `crates/nuxie/src/flow_session.rs:21-621` (601 lines) | Public `File`, `Factory`, renderer, artboard, view-model, and state-machine types | Clean product vocabulary; move to `nuxie-flow` after dependency direction is fixed. |
| FlowSession orchestration | `flow_session.rs:622-2163` (1,542) | Calls crate-private script preparation, listener preparation, rehydration, apply/rollback, host-cycle, and command-drain methods on the `nuxie` façade | Product-side, but requires a narrow public/sealed flow-runtime bridge before crate extraction. Moderate coupling. |
| FlowSession value snapshots | `flow_session.rs:2164-3105` (942) | Traverses public runtime-owned view-model/value objects and builds its own graph/arena | Product-side and cohesive. Mostly mechanical after bridge creation. |
| FlowSession validation/accounting | `flow_session.rs:3106-3545` (440) | Product protocol limits and payload validation | Clean extraction candidate. |
| FlowSession mutation/apply/diff | `flow_session.rs:3546-4515` (970) | Applies mutations through runtime view-model/state-machine handles; shares rollback/command sequencing with façade-private methods | Product-side; transactional sequencing makes it medium risk. |
| FlowSession selection/catalog | `flow_session.rs:4516-5020` (505) | File/artboard/state-machine discovery | Product-side; mostly public runtime surface. |
| FlowSession tests | `flow_session.rs:5021-7227` (2,207; 40 tests), plus `crates/nuxie/tests/flow_session_contract.rs` (335) | Exercises the public ABI, VM lifecycle, and transactional behavior | Move with FlowSession. Preserve public names through compatibility re-exports during migration. |
| Scene contract and IDs | `crates/nuxie/src/scene.rs:28-1187` (1,160) | Public runtime/binary identities and errors | Product-side, cleanly demarcatable. |
| Scene durable definitions/index/specs | `scene.rs:1188-3401` (2,214) | `nuxie_binary::AuthoringRecord`, property/value types, runtime file identities | Product-side authoring model. |
| Scene hierarchy/materialization | `scene.rs:3402-5622` (2,221) | Constructs/reads `RuntimeFile`; directly touches `materialized.file.runtime` in a handful of places even though `File::runtime()` already exists | Product-side but high coupling. Replace direct private field access with the existing public accessor before moving. |
| Silver/scene export schema | `scene.rs:5623-6819` (1,197) | Public exported-record/document DTOs; lowering later feeds `RuntimeFile::from_authoring_records` | Product-side. The schema and its generated counterpart must move together. |
| Scene store/live mount API | `scene.rs:6820-9099` (2,280) | Owns mounted `File`/artboard/view-model state and coordinates transactions | Product-side; medium-to-high coupling. |
| SceneTx and subordinate transactions | `scene.rs:9100-15589` (6,490) | `SceneTx`, `DataConverterTx`, `VmTx`, `AnimTx`, and `MachineTx` mutate the authoring graph, then materialize into runtime/binary types | Core product authoring seam. Cohesive, but atomic commit/materialization behavior makes the move high risk. |
| Scene frame queries/live mutation | `scene.rs:15590-17592` (2,003) | Hit/geometry/semantic queries and intrinsic image sizing reach crate-private `OwnedArtboardInstance` helpers | Product-side. Needs stable snapshot/command DTOs instead of exposing mutable runtime internals. |
| Scene lowering/validation/export | `scene.rs:17593-25563` (7,971) | Converts authoring records into `RuntimeFile`, validates cross-object invariants, and produces exported documents | Product-side. Highest-risk region because it couples the authoring model to port construction invariants. |
| Scene tests | `scene.rs:25564-38224` (12,661; 120 tests), plus `crates/nuxie/tests/scene_authoring.rs` (15,674) and related authoring tests | Broad transaction, materialization, export, and live-runtime contract | Move with Scene; use this suite as the seam migration oracle. |
| Scene schema generator | `crates/nuxie/build.rs` (4,531) and `scene.rs`'s `include!(concat!(env!("OUT_DIR"), "/scene_schema.rs"))` near line 860 | Generates the Scene/silver record schema at build time | Product-side and inseparable from Scene. Extraction must move the build script/generator inputs and preserve the generated include contract. |
| Script import authorization | `crates/nuxie/src/script_import.rs:1-157` production, `158-211` tests | Ed25519/container verification; calls crate-private `File::execution_authorization_for`/authorization state | Product policy. Move to `nuxie-flow`, but expose a capability-based import seam rather than making raw authorization state public. |
| Mixed façade bridge | `crates/nuxie/src/lib.rs`, principally project-envelope classification and private bridge methods around lines 55, 154, 223-258, 307, 344, 376, 1,492, 2,983, 3,328, 4,304, and 5,600-5,733 | `File::from_runtime`; flow script/listener lifecycle; apply/rollback/command drain; artboard geometry/semantic/intrinsic-image helpers | Not an independent feature. Convert these call sites into explicit narrow bridge traits/DTOs, then move their product consumers. |

The concrete crate-private methods the extraction must account for are:

- Flow lifecycle around `nuxie/src/lib.rs:5600-5675`: `prepare_flow_scripts`, `prepare_flow_listener_actions`, rehydrate/apply, begin host cycle, rollback, and drain commands.
- Scene live/runtime queries around `lib.rs:5705-5733`: hit-test path segments with bounds, geometry path segments, retained/semantic text, and intrinsic image-dimension registration.
- `File::from_runtime` around `lib.rs:4304` and the script execution authorization capability.

These are already cross-crate calls from `nuxie` into public `nuxie-runtime` types; the missing visibility is chiefly inside the `nuxie` façade. Extraction should expose a sealed product bridge from the future base façade, not expand dozens of runtime fields to `pub`.

### Project-data converter: product model inside the port

`crates/nuxie-runtime/src/project_data_converter.rs` is 2,686 lines and is explicitly classified `scene-api` in `rust-additions.toml`:

| Lines | Region | Coupling and disposition |
| ---: | --- | --- |
| 22-507 | Public values, specs, errors, and state | Pure product/protocol model; eventual neutral-crate candidate. |
| 508-1,289 | Envelope/program/catalog compilation and evaluation | Mostly `std`/`serde`; eventual neutral-crate candidate. |
| 1,290-2,324 | Validation, coercion, formatting, interpolation | Pure-looking core, but behavior is part of runtime data-bind semantics. Extract only with parity tests. |
| 2,325-2,686 | Formula parser | Mechanically separable after the outer seam is established. |

The file is locally self-contained, but its runtime integration is not. `data_bind/context/context_value.rs` has a `RuntimeDataBindGraphConverter::Project` variant and owns conversion/evaluator state; `data_bind/data_bind_context.rs` constructs it from a script-asset envelope; both runtime and `nuxie` re-export or classify its types; Scene authors and lowers them. A direct move to `nuxie-flow` would invert the desired dependency and make the port depend on the product crate.

Two crate-private numeric-policy helpers in this file (the bounded-list constant near line 467 and helper near line 2,320) are also used by port-side Number-to-List/context conversion. If the model is eventually extracted, those policies should remain in the port or move to a dependency-neutral shared crate; they should not become a product API solely to satisfy the move.

Recommendation: first demarcate this as `project_data_converter/{model, program, evaluator, bridge}` inside `nuxie-runtime`; later extract only the pure model/program/evaluator to a neutral `nuxie-project-data` crate that both runtime and `nuxie-flow` may depend on. The runtime adapter stays in `nuxie-runtime`.

### Rust-only infrastructure that should remain port-side

The following `rust-additions.toml` entries have no one-file C++ correspondence but are not product features:

| Current path/group | Lines | Why it stays in `nuxie-runtime` |
| --- | ---: | --- |
| `data_bind_container.rs`, `data_converter.rs`, `retained_data_bind.rs` | 12 total | Tiny generated/type façade modules over port functionality. |
| `input/mod.rs`, `shapes/mod.rs`, `shapes/paint/mod.rs` | 168 total | Rust namespace/generated-storage coordinators. |
| `objects.rs` | 1,018 | Runtime object registry plus `include!(concat!(env!("OUT_DIR"), "/runtime_objects.rs"))`; generated by `crates/nuxie-runtime/build.rs` (707 lines). This is port construction infrastructure. |
| `properties.rs` | 826 | Generated property interpretation/storage used throughout the port. |
| `state_machine.rs`, `state_machine/instance.rs`, `state_machine/listener_types/mod.rs` | 215 total | Rust module/re-export façade for the ported state-machine implementation. |
| `viewmodel/mod.rs`, `viewmodel/runtime/mod.rs` | 58 total | Rust namespace/re-export façade for ported view-model behavior. |
| `focus.rs` | 7 | Thin public façade over retained focus behavior, not a standalone product feature. |

These codegen/facade additions total about 2,303 source lines excluding the project converter and focus façade. They should be clearly marked as Rust port infrastructure, but moving them to `nuxie-flow` would blur rather than improve the seam.

`crates/nuxie/src/command_queue.rs`, server code, and raw-text support are not included in the Nuxie-only list: their current files are explicitly mapped to upstream C++ correspondence. A richer Rust API around a ported facility does not by itself make that facility product-side.

---

## 4. Recommended seam design

### Decision

Use a **module boundary now and a crate boundary later**.

The immediate structure should make product ownership visible without changing dependencies:

```text
crates/nuxie/src/product/
  flow_session/
  scene/
    model/
    transaction/
    live/
    export/
  script_import/

crates/nuxie-runtime/src/project_data_converter/
  model.rs
  program.rs
  evaluator.rs
  bridge.rs
```

Preserve current public paths with root re-exports. This first step is a namespace/demarcation change, not an extraction. It gives reviewers an auditable boundary while the current test and parity baseline still observes exactly the same crate graph.

The long-term crate graph should avoid a cycle:

```text
nuxie-runtime  <---  nuxie-core  <---  nuxie-flow
       ^                 ^                 ^
       |                 |                 |
       +------ nuxie-project-data --------+

nuxie (umbrella compatibility crate) re-exports nuxie-core + nuxie-flow
```

`nuxie-flow` contains FlowSession, Scene/SceneTx, silver/export DTOs and lowering, and script-import policy. `nuxie-core` contains `File`, `Factory`, owned artboard/view-model handles, renderer integration, and narrow bridge traits. `nuxie-runtime` remains the C++-correspondence port plus explicitly labeled Rust port infrastructure. A small dependency-neutral `nuxie-project-data` is optional and should be introduced only after the runtime adapter has been separated from the pure evaluator.

### Public surface of the port/base layer

The extraction should not make arbitrary runtime fields public. The required surface is narrow:

1. A sealed/trusted constructor that wraps a validated `RuntimeFile` as a façade `File`, with an explicit authoring/import authorization policy.
2. A flow-runtime bridge for prepare, rehydrate, apply, host-cycle, rollback, and command-drain operations. Its inputs/outputs should be stable value DTOs, not references to internal queues.
3. Read-only artboard snapshot operations for hit geometry, retained geometry, and semantic text, plus a command for intrinsic image dimensions. Return owned snapshots so Scene cannot retain internal graph borrows.
4. A project-converter adapter owned by `nuxie-runtime`; pure program/model types may come from the neutral crate. Do not expose the runtime bind graph or evaluator cells as product API.
5. Existing upstream-corresponding runtime types only where they are already public and semantically stable. Product re-exports should live in `nuxie`/`nuxie-flow`, not expand the port's promise.

### Migration order

1. Establish the exact green parity/golden baseline and keep it pinned in every PR.
2. Land the `artboard.rs` and `state_machine_instance.rs` mechanical splits as separate PRs. This makes later ownership changes reviewable without mixing 20,000-line source moves into semantic work.
3. Demarcate `nuxie::product::{flow_session, scene, script_import}` and the project-converter submodules in place, preserving public re-exports.
4. Define the sealed flow, authoring-file, geometry-snapshot, and script-authorization bridges; migrate existing internal callers to them while everything remains in the same crate.
5. Extract the non-product façade/runtime handles into `nuxie-core`, with `nuxie` temporarily acting as an umbrella compatibility crate.
6. Move FlowSession and script import to `nuxie-flow`; move their tests and retain compatibility re-exports.
7. Move Scene model, SceneTx, export schema/generator, lowering, live adapter, and tests together. Do not split the generated schema from `build.rs` or split transaction commit from materialization in the extraction PR.
8. Isolate the project converter's pure model/program/evaluator. Extract it to a neutral crate only if doing so leaves the runtime adapter dependency pointing inward, never from `nuxie-runtime` to `nuxie-flow`.
9. Remove compatibility re-exports and tighten visibility in a later breaking-change PR, after downstream migration.

### Risk map

| Risk | Areas | Reason |
| --- | --- | --- |
| Mechanical/low | Thin namespace façades, codegen coordinators staying in place, test-module extraction, pure FlowSession validation/types | Mostly physical ownership with no runtime mutation semantics. |
| Moderate | FlowSession after bridge creation, script import after capability API, Scene DTO/export types | Public API and authorization compatibility, but limited graph mutation. |
| High | SceneTx commit/materialization, live Scene mounting, geometry/semantic access, Scene lowering and generated schema | Atomicity, identity, generated code, and live runtime invariants cross the seam. |
| High | Project-data converter extraction | The source looks pure, but evaluator state is embedded in runtime data-bind context and Number-to-List policy. |
| High | Script/listener ownership during FlowSession moves | Rehydration, rollback, command queues, and VM object lifetime must remain one coherent transaction. |

### Why baseline-first matters

`docs/parity-gap-register.md` explicitly recommends making the parity claim trustworthy before feature work and landing each subsequent change against the oracle (the “Recommended execution order” around lines 204-211). It also describes Scene as a Nuxie-only additive API (around lines 126-127) while recording FlowSession/capability-boundary gaps elsewhere in the register. That is the governing principle here: first make source moves mechanically observable by the existing correspondence, ownership, trace, golden, and silver batteries; then introduce the product seam behind those same oracles. A green build alone is insufficient because it can preserve compilation while silently weakening ownership scans or changing what “zero divergences” measures.

The two large-file splits are therefore prerequisites for, not part of, the product extraction. Each split keeps the logical Rust module unchanged and updates only physical-address consumers. The seam work then proceeds through explicit bridges and compatibility re-exports, with risky transaction/materialization changes isolated from mechanical moves.

# nuxie-runtime structure report

## Scope and baseline

This is a read-only structure analysis. It proposes no behavior, source, parity-ledger, or ownership-ledger changes.

The inspected worktree was already detached at `e8726db8ffc689d3b19f1c0a55794aa6daf9956d` (`Merge pull request #246 from TheOneLevin/feat/b6-0058-listener-viewmodel-change`, 2026-08-04). The local `origin/main` ref resolves to the same commit. The requested `git fetch origin` and redundant `git checkout --detach origin/main` could not update the shared worktree metadata because the sandbox cannot write the parent repository's `.git/worktrees/nuxie-mr-c12` directory. The analysis is therefore pinned to the locally available `origin/main` commit above, not a newly fetched remote ref. The pre-existing untracked `vtriage-capture/` directory was left untouched.

The two target files have grown beyond the sizes in the request:

| File | Current lines | Production/support | Tests |
| --- | ---: | ---: | ---: |
| `crates/nuxie-runtime/src/artboard.rs` | 23,374 | 12,039 | 11,335 (`205` tests) |
| `crates/nuxie-runtime/src/state_machine/state_machine_instance.rs` | 21,674 | 14,079 | 7,595 (`94` tests) |

The safest split mechanism is literal `include!` fragments, not Rust child modules. An included fragment remains in the original semantic module (`crate::artboard` or `crate::state_machine::state_machine_instance`), so private-name resolution, inherent method paths, sibling access, and demangled symbols remain unchanged. A `mod build;`/`#[path = ...] mod build;` split would introduce new privacy boundaries and make a supposedly mechanical PR into an API refactor.

---

## 1. Large-file split plan: `artboard.rs`

### Current structural map

| Lines | Region |
| ---: | --- |
| 1-106 | Imports and supporting aliases |
| 107-135 | Draw-frame counter and generated `Mat2D` helper |
| 136-160 | `ExternalFontAssetError` |
| 161-181 | `RuntimeArtboardInstanceIdentity` |
| 182-231 | `RuntimeScriptState` and implementations |
| 232-250 | Advancing-component and persistent-dirt fixtures |
| 251-298 | Script advance/update mode enums and implementations |
| 299-327 | Resetting/runtime-component helper types |
| 328-334 | `RuntimeTextStyleFeatureOption` |
| 335-523 | `ArtboardInstance` storage |
| 524-714 | `Clone for ArtboardInstance` |
| 715-739 | `Drop for ArtboardInstance` followed by the existing leaf `include!` declarations |
| 740-773 | Occurrence and frame-advance report types |
| 774-844 | Build context/index/text-flag helpers |
| 845-860 | `RuntimeNestedAnimationInstance` |
| 861-11,133 | Main `impl ArtboardInstance` |
| 11,134-11,147 | Focus-scroll path and focus-bounds transform helper |
| 11,148-11,187 | Small follow-up `impl ArtboardInstance` |
| 11,188-11,226 | Component-list reset helper |
| 11,227-11,595 | `impl RuntimeNestedArtboardInstance` |
| 11,596-11,611 | `impl RuntimeNestedAnimationInstance` |
| 11,612-12,039 | Free construction/nested-artboard helpers |
| 12,040-23,374 | Single `#[cfg(test)] mod tests` (`205` tests) |

The main implementation itself divides along stable responsibility boundaries:

| Lines | Logical responsibility |
| ---: | --- |
| 861-1,104 | Lifecycle, data context, and transient-clone restoration |
| 1,105-2,440 | Build occurrence relationships and constructors |
| 2,441-2,712 | Path/hit initialization and external assets |
| 2,713-3,399 | Scripting lifecycle |
| 3,400-3,989 | Object/component/audio/hit/graph/definition access |
| 3,990-4,152 | Scripted interpolators and state-machine definition selection |
| 4,153-4,480 | Dimensions, world/layout bounds, scrolling, and focus reveal |
| 4,481-5,229 | Generated property façade and text values |
| 5,230-5,568 | Animation/state-machine/view-model occurrences |
| 5,569-6,312 | Component-list lifecycle and layout |
| 6,313-7,408 | Root/nested frame advance and retained-component dispatch |
| 7,409-7,585 | Nested-layout frame propagation |
| 7,586-8,496 | Transforms, epochs, dirt, and invalidation |
| 8,497-9,191 | Update pass and five-pass state-machine settlement |
| 9,192-9,825 | Component update dispatch and script scheduling |
| 9,826-11,133 | Property-change callbacks, nested controls, and collapse |

### Proposed physical layout

Keep `artboard.rs` as the owner. It retains imports, supporting types, `ArtboardInstance`, `Clone`, `Drop`, the existing leaf includes, occurrence/report/build helper types, and an ordered series of literal includes. This preserves declaration order where it matters and makes all fragments part of `crate::artboard`.

```text
src/
  artboard.rs                         owner/types, existing leaf includes, ordered include! list
  artboard/
    build.rs                          current 861-2712
    script_runtime.rs                 current 2713-3399
    access.rs                         current 3400-3989
    definitions_and_layout.rs         current 3990-4480
    properties.rs                     current 4481-5568
    component_lists.rs                current 5569-6312
    advance.rs                        current 6313-7585
    dirt.rs                           current 7586-8496
    update.rs                         current 8497-9825
    callbacks.rs                      current 9826-11133
    nested.rs                         current 11134-12039
    tests.rs                          body of current tests module
```

The production fragments are intentionally in execution/dependency order rather than alphabetical order. `tests.rs` should be included inside the original test module:

```rust
#[cfg(test)]
mod tests {
    include!("artboard/tests.rs");
}
```

This retains the test module's name and access. It also gives the attribution checker a deliberately exempt `tests.rs` leaf.

### Lockstep correspondence and checker manifest

The following table is the required per-file edit manifest. “B6 rows” are `[[files]]` entries in `file-correspondence-manifest.toml`; add the fragment's path to those rows' semicolon-separated `rust_module` values and update their C17 retained-module notes. Keep status, upstream pins, C++ paths, verification text, and correspondence claims unchanged. The row list is conservative: a later provenance cleanup can narrow it, but narrowing must not be mixed into this mechanical split.

| New file | B6 rows that must name it | Frame-loop/checker bindings that must move with it |
| --- | --- | --- |
| `artboard/build.rs` | `B6-0001`, `B6-0094`, `B6-0115`, `B6-0117`, `B6-0123`, `B6-0125`, `B6-0129`, `B6-0131`, `B6-0142`, `B6-0146`, `B6-0202`, `B6-0258`, `B6-0303`, `B6-0304`, `B6-0305`, `B6-0331`, `B6-0361`, `B6-0362`, `B6-0365`, `B6-0368`, `B6-0385`, `B6-0388`, `B6-0399`, `B6-0405`, `B6-0408` | Repoint `summarize_trace.py` dependency-build and IK-chain-build source scans when their anchors land here. Repoint matching `runtime-frame-loop-gaps.toml` owner-boundary entries. |
| `artboard/script_runtime.rs` | `B6-0077`, `B6-0194`, `B6-0200`, `B6-0322`, `B6-0326` | Repoint the script-lifecycle citations in `runtime-frame-loop-ownership.toml` and any corresponding synthetic test fixture paths. |
| `artboard/access.rs` | `B6-0094`, `B6-0095`, `B6-0123`, `B6-0146`, `B6-0203`, `B6-0204`, `B6-0205` | Repoint any live-draw/renderer-interface lifecycle citations whose named methods move here. |
| `artboard/definitions_and_layout.rs` | `B6-0077`, `B6-0094`, `B6-0258`, `B6-0303`, `B6-0304`, `B6-0305`, `B6-0385`, `B6-0388` | Repoint focus-retained-tree and layout lifecycle citations, plus matching owner-boundary allow entries. |
| `artboard/properties.rs` | `B6-0067`, `B6-0077`, `B6-0094`, `B6-0194`, `B6-0200`, `B6-0352`, `B6-0355`, `B6-0385`, `B6-0388`, `B6-0399`, `B6-0404`, `B6-0405`, `B6-0408` | `B6-0067` currently has only one Rust module: **replace** `artboard.rs` with this fragment rather than adding a second path. Repoint property/text lifecycle citations and corresponding owner-boundary entries. |
| `artboard/component_lists.rs` | `B6-0095`, `B6-0258`, `B6-0303`, `B6-0304`, `B6-0305` | Repoint component-list and nested-layout lifecycle citations and matching test fixtures. |
| `artboard/advance.rs` | `B6-0001`, `B6-0059`, `B6-0061`, `B6-0063`, `B6-0064`, `B6-0077`, `B6-0094`, `B6-0095`, `B6-0194`, `B6-0200`, `B6-0258`, `B6-0303`, `B6-0304`, `B6-0305`, `B6-0322`, `B6-0326`, `B6-0388` | In `runtime-frame-loop-ownership.toml`, move `artboard.advance`'s `rust_file` and the lifecycle citations for artboard/nested advance. Repoint `summarize_trace.py` advancing/resetting dispatch scans as applicable. Repoint relevant gap allow rows and `test_check.py` advance fixtures. |
| `artboard/dirt.rs` | `B6-0094`, `B6-0123`, `B6-0258`, `B6-0331`, `B6-0352`, `B6-0355`, `B6-0357`, `B6-0361`, `B6-0362`, `B6-0365`, `B6-0368`, `B6-0385`, `B6-0388`, `B6-0399`, `B6-0405`, `B6-0408` | Move `component.dirt`, `component.dependents` (`pub fn add_dirt(`), and transform/dirt lifecycle citations as appropriate. Repoint `summarize_trace.py`'s two add-dirt anchors, dirt consumptions, owner resolutions, and skin-buffer scans according to their final physical locations. Repoint matching gap rows. |
| `artboard/update.rs` | `B6-0001`, `B6-0094`, `B6-0115`, `B6-0117`, `B6-0123`, `B6-0125`, `B6-0129`, `B6-0131`, `B6-0142`, `B6-0194`, `B6-0200`, `B6-0203`, `B6-0204`, `B6-0205`, `B6-0258`, `B6-0303`, `B6-0304`, `B6-0305`, `B6-0322`, `B6-0326`, `B6-0331`, `B6-0352`, `B6-0355`, `B6-0357`, `B6-0361`, `B6-0362`, `B6-0365`, `B6-0368`, `B6-0385`, `B6-0388`, `B6-0399`, `B6-0404`, `B6-0405`, `B6-0408` | Move `component.update_order` (`fn update_components_with_hook_recording`) and `artboard.update_pass` (`pub fn update_pass(`) `rust_file` values and lifecycle citations. Repoint retained update/settlement gap rows, trace scans, and test fixtures. |
| `artboard/callbacks.rs` | `B6-0059`, `B6-0061`, `B6-0063`, `B6-0064`, `B6-0077`, `B6-0094`, `B6-0123`, `B6-0125`, `B6-0129`, `B6-0131`, `B6-0142`, `B6-0194`, `B6-0200`, `B6-0258`, `B6-0303`, `B6-0304`, `B6-0305`, `B6-0331`, `B6-0352`, `B6-0355`, `B6-0357`, `B6-0361`, `B6-0362`, `B6-0365`, `B6-0368`, `B6-0385`, `B6-0388`, `B6-0399`, `B6-0404`, `B6-0405`, `B6-0408` | Change `check_layout_style_handlers.py`'s `RUST_ARTBOARD_MODULE` to this fragment if both display-change markers remain here; otherwise make the checker accept the two explicit physical files. Repoint callback lifecycle citations, gap rows, and fixtures. |
| `artboard/nested.rs` | `B6-0059`, `B6-0061`, `B6-0063`, `B6-0064`, `B6-0077`, `B6-0094`, `B6-0095`, `B6-0258`, `B6-0303`, `B6-0304`, `B6-0305` | Repoint nested-artboard advance/layout citations, gap allow entries, and nested fixtures. |
| `artboard/tests.rs` | None. `rust_attribution.py` excludes a source whose stem is `tests`. | Move the artboard test body unchanged. Update the seven relative `include_bytes!`/fixture paths currently near old lines 19,951, 20,001, 20,168, 20,542, 20,597, 20,612, and 23,311 because the file becomes one directory deeper. Prefer `concat!(env!("CARGO_MANIFEST_DIR"), "/...")` for fixture stability. The `env!` use near old line 14,290 remains valid. |

#### Global artboard bindings

These bindings span more than one fragment and therefore belong in the same PR:

- `rust_attribution.py` discovers all production `.rs` files below `crates/nuxie-runtime/src`. Every new production fragment must be listed in at least one `file-correspondence-manifest.toml` row and must **not** be added to `rust-additions.toml`. The checker exempts `tests.rs`, names ending in `_tests`, and files below a `tests/` path.
- The manifest's scatter ratchet is already at its maximum: 155 rows currently have multiple Rust modules. Except for `B6-0067`, all affected artboard rows are already multi-module. Replacing the `B6-0067` root path prevents the split from increasing that count.
- Update every affected C17 note that enumerates retained Rust modules. Do not change the underlying parity classification merely because the code's physical address changed.
- `docs/runtime-frame-loop-ownership.toml` has artboard citations in `focus.retained_tree`, `component.identity`, `component.dirt`, `component.dependents`, `component.update_order`, `component.transforms`, `component.clone_drop`, `artboard.advance`, `artboard.update_pass`, `nested_artboard.advance`, `artboard.live_draw`, and `renderer.interface_boundary`. Keep `component.clone_drop` and its `pub struct ArtboardInstance` anchor on the root; change each other `rust_file` or `rust:path:line-range` to its physical fragment.
- `docs/runtime-frame-loop-gaps.toml` has 25 owner-boundary allow entries pointing at `artboard.rs` (entries 0-7, 11-22, and 29-34 in current order). Repoint each to the fragment that contains its anchor. A literal move should preserve its token-based `site_hash`; a changed hash is a signal that the split was not mechanical.
- `tools/runtime-frame-loop-port/summarize_trace.py` hard-codes `artboard.rs` nine times for add-dirt, dirt consumption, owner resolution, dependency build, skin rebuild, advancing dispatch, resetting dispatch, and IK-chain discovery. Give each query its actual fragment path. The demangled owner should remain `artboard::ArtboardInstance` because `include!` does not introduce a child module.
- `tools/runtime-frame-loop-port/check_layout_style_handlers.py` hard-codes the artboard source while looking for `propagate_layout_component_display_changed` and the display property key. Point it to the fragment(s) containing those markers.
- `tools/runtime-frame-loop-port/test_check.py` contains at least 14 exact artboard paths covering probe gating, layer occurrence, live-advance ratchets, registry site hashes/substitution, and trait-anchor behavior. Update each synthetic fixture to the fragment that owns the tested anchor; do not perform a blind string replacement.
- Leave the existing root-level includes (`nested_artboard.rs`, `artboard_component_list.rs`, `artboard_list_map_rule.rs`, `artboard_referencer.rs`, `bindable_artboard.rs`, `nested_artboard_layout.rs`, `nested_artboard_leaf.rs`, `component_origin.rs`, bone/weight, profiler/profile) where they are. Moving them under `artboard/` changes relative include resolution and correspondence ownership.
- Keep `crates/nuxie-runtime/src/lib.rs`'s `mod artboard;` unchanged. The public and crate-private paths must not acquire an extra `artboard::<fragment>` segment.

### Mechanical PR sequence and proof

One PR should contain only this file's split and the path/anchor bookkeeping above. Recommended commit sequence within that PR:

1. Extract `tests.rs`, fix only its fixture paths, and prove the same 205 tests are collected.
2. Extract production regions in original order using `include!`; make no renames, visibility edits, formatting sweeps, or logic edits.
3. Update correspondence C17/path fields and all frame-loop physical anchors.
4. Update checker source locations and synthetic fixtures, including an assertion that demangled ownership remains `artboard::ArtboardInstance`.

Proof battery:

```text
make rust-sources-fresh
make cpp-probe
make runtime-frame-loop-port-test
make runtime-frame-loop-port-check
make rust-attribution-check
cargo check --workspace
cargo test -p nuxie-runtime
cargo test -p nuxie --features scripting
make scripted-golden-compare
make silver-corpus-test
```

The PR is structure-preserving only if parity row counts/statuses, trace landmarks, owner-boundary hashes, test counts, and golden/silver outputs are unchanged.

---

## 2. Large-file split plan: `state_machine_instance.rs`

### Current structural map

| Lines | Region |
| ---: | --- |
| 1-93 | Imports and ownership commentary |
| 94-255 | Nested event-chain/notify phases, trace structures, and trace recording |
| 256-312 | Semantic-node resolver and audio/event seam types |
| 313-1,271 | `RuntimeHitResult`, `HitComponent`, and hierarchy implementations |
| 1,272-1,467 | Nested notifier, focus, semantic, and state helper types |
| 1,468-1,474 | `RuntimeDataContextBindError` |
| 1,475-1,711 | `StateMachineInstance` storage |
| 1,712-1,744 | Data-bind occurrence/pointer/executor structures |
| 1,745-2,147 | Listener-action executor implementations |
| 2,148-2,335 | `Clone for StateMachineInstance` |
| 2,336-2,356 | `Drop for StateMachineInstance` |
| 2,357-2,638 | Free helpers and view-model listener types |
| 2,639-13,938 | Main `impl StateMachineInstance` |
| 13,939-13,953 | `runtime_owned_font` helper |
| 13,954-14,079 | `view_model_listener_tests` (`1` test) |
| 14,080-21,674 | `scripted_listener_action_tests` (`93` tests) |

The main implementation divides as follows:

| Lines | Logical responsibility |
| ---: | --- |
| 2,639-2,932 | Teardown, enrollment, focus manager, clone rebind |
| 2,933-3,741 | Construction and listener/layer initialization |
| 3,742-5,102 | Scripting, hydration, and adoption |
| 5,103-6,007 | State/input/focus/semantic/keyboard/gamepad operations |
| 6,008-7,487 | Hit/listener/pointer pipeline |
| 7,488-8,525 | Event notification, deferred callbacks, listener actions |
| 8,526-9,100 | Direct bind targets and value readers |
| 9,101-10,748 | Default/imported/owned source setters and relinking |
| 10,749-12,599 | Data-context bind/rebind, listener cells, bind updates |
| 12,600-13,938 | Reports, advance/apply, nested event dispatch, settlement |

### Proposed physical layout

```text
src/state_machine/
  state_machine_instance.rs               owner/types, hit hierarchy, lifecycle, ordered include! list
  state_machine_instance/
    construction.rs                       current 2639-3741
    scripted_objects.rs                   current 3742-5102
    input_focus.rs                        current 5103-6007
    pointer.rs                            current 6008-7487
    events_actions.rs                     current 7488-8525
    bind_targets.rs                       current 8526-9100
    bind_sources.rs                       current 9101-10748
    data_context.rs                       current 10749-12599
    advance.rs                            current 12600-13953
    tests/
      view_model_listener.rs              body of current first test module
      scripted_listener_actions.rs         body of current second test module
```

Retain the two existing module names in the root and include only their bodies. This preserves test paths and any name-sensitive filtering:

```rust
#[cfg(test)]
mod view_model_listener_tests {
    include!("state_machine_instance/tests/view_model_listener.rs");
}
```

Use the same pattern for `scripted_listener_action_tests`.

### Lockstep correspondence and checker manifest

| New file | B6 rows that must name it | Frame-loop/checker bindings that must move with it |
| --- | --- | --- |
| `state_machine_instance/construction.rs` | `B6-0077`, `B6-0194`, `B6-0195`, `B6-0196`, `B6-0200`, `B6-0310` | Repoint construction/enrollment lifecycle citations and matching synthetic fixtures. Keep `state_machine.collections`' `pub struct StateMachineInstance` anchor on the root. |
| `state_machine_instance/scripted_objects.rs` | `B6-0077`, `B6-0175`, `B6-0194`, `B6-0195`, `B6-0196`, `B6-0200` | Repoint scripting/hydration lifecycle citations and scripted-action fixture paths. |
| `state_machine_instance/input_focus.rs` | `B6-0077`, `B6-0083` | Repoint focus/semantic/keyboard/gamepad citations and tests. |
| `state_machine_instance/pointer.rs` | `B6-0077`, `B6-0083` | Repoint hit/pointer/listener lifecycle citations and FL-C4/layer/hit fixtures. |
| `state_machine_instance/events_actions.rs` | `B6-0058`, `B6-0077`, `B6-0440` | Repoint event reporting, firing/bubbling, and action-dispatch citations/fixtures. `state_machine.actions`' primary anchor remains on the root if the listener-action executor stays there. |
| `state_machine_instance/bind_targets.rs` | `B6-0058`, `B6-0194`, `B6-0196`, `B6-0200`, `B6-0440` | Repoint bind-target and view-model listener citations/fixtures. |
| `state_machine_instance/bind_sources.rs` | `B6-0058`, `B6-0175`, `B6-0194`, `B6-0195`, `B6-0196`, `B6-0200`, `B6-0440` | Repoint source/relink/data-converter citations and bind fixtures. |
| `state_machine_instance/data_context.rs` | `B6-0058`, `B6-0077`, `B6-0194`, `B6-0195`, `B6-0196`, `B6-0200`, `B6-0440` | Repoint data-context lifecycle citations and listener-cell/bind fixtures. |
| `state_machine_instance/advance.rs` | `B6-0077`, `B6-0194`, `B6-0195`, `B6-0196`, `B6-0200`, `B6-0310`, `B6-0440` | Move `state_machine.advance`'s `rust_file` and `advance_with_report_mode` anchor here. Repoint state-machine event/action/reporting lifecycle citations and advance/keyframe fixtures. Include `runtime_owned_font` in this fragment to avoid an artificial one-function file. |
| `state_machine_instance/tests/view_model_listener.rs` | None; it resides below a `tests/` path. | Move the body unchanged and preserve the outer module name. |
| `state_machine_instance/tests/scripted_listener_actions.rs` | None; it resides below a `tests/` path. | Move the body unchanged. Replace or correct the relative fixture path near old line 19,011 (`../../../../fixtures/sync/vm_listener_fire_event.riv`) because the source becomes two directories deeper; prefer a `CARGO_MANIFEST_DIR`-anchored path. |

#### Global state-machine bindings

- All production fragments must be named by `file-correspondence-manifest.toml`, never `rust-additions.toml`. Every affected SMI row is already multi-module, so this split does not increase the manifest's 155-row scatter count.
- Update C17 retained-module notes for `B6-0058`, `B6-0077`, `B6-0083`, `B6-0175`, `B6-0194`, `B6-0195`, `B6-0196`, `B6-0200`, `B6-0310`, and `B6-0440`; leave the substantive parity assertions untouched.
- `docs/runtime-frame-loop-ownership.toml` cites this file from `focus.retained_tree`, `state_machine.collections`, `state_machine.advance`, `state_machine.events`, `state_machine.actions`, and `event.reporting`. Keep root anchors for the struct, stored event collection, and action executor. Repoint method/lifecycle citations to physical fragments.
- `tools/runtime-frame-loop-port/check.py` currently treats one hard-coded SMI file as the nested-event owner and excludes it from outside-owner scans. Once methods are in included fragments, a path-only exclusion would falsely flag the fragments. Make owner discovery include-aware: recursively expand literal local `include!("...")` files in source order for export analysis, and treat every expanded file as the same logical owner. Do not broaden the exclusion to a directory glob.
- Add checker unit coverage for the include-aware logical owner: an exported nested-event method in an included fragment must be accepted, while the same method in an unrelated file must still fail. Preserve syntax parsing and duplicate-anchor detection.
- `tools/runtime-frame-loop-port/test_check.py` has at least 14 exact SMI references, plus constructed/split path strings, covering FL-C4, lifecycle, layer, hit, bind, events, firing/bubbling, advance, owner exports, keyframes, focus, and semantic behavior. Point each fixture to the actual fragment; do not simply substitute the root path everywhere.
- `docs/runtime-frame-loop-gaps.toml` has no current SMI owner-boundary allow row to migrate.
- Keep `crates/nuxie-runtime/src/state_machine.rs`'s `pub(crate) mod state_machine_instance;` and its re-exports unchanged. Literal includes keep all symbols at the original module path.

### Mechanical PR sequence and proof

This should be a second, independent PR after the artboard split has landed. It uses the same proof battery. The checker must first learn the logical-owner/include relationship in the same PR, because moving the methods without that update creates false ownership failures. Apart from that narrowly required path-awareness, no checker policy should change.

Success means the same 94 tests are collected, no exported API path changes, the correspondence scatter count stays at 155, no new owner-boundary allowance appears, and the full battery shown in the artboard plan remains green.

---

## 3. Nuxie-only seam inventory

### Classification method

The authoritative starting point is correspondence metadata:

- Production Rust paths mapped to upstream C++ in `file-correspondence-manifest.toml` are port-side unless a smaller product-only region can be proven.
- Production Rust paths listed in `rust-additions.toml` are intentional Rust-only additions. The relevant ownership labels are `flowsession-abi`, `scene-api`, and generated/codegen infrastructure.
- `build.rs` is outside the attribution scanner's `src/` roots, so it must be inspected manually.
- A convenience façade is not automatically product-side. Thin Rust-only namespace/generated-storage coordinators that serve the port stay with the port even though no one C++ file corresponds to them.

This prevents two opposite mistakes: burying product authoring features inside the port forever, and extracting Rust implementation infrastructure merely because its file has no one-to-one C++ source.

### Product features and mixed glue

| Region | Current size and location | Runtime internals reached / extraction surface | Placement assessment |
| --- | --- | --- | --- |
| FlowSession wire/domain model | `crates/nuxie/src/flow_session.rs:21-621` (601 lines) | Public `File`, `Factory`, renderer, artboard, view-model, and state-machine types | Clean product vocabulary; move to `nuxie-flow` after dependency direction is fixed. |
| FlowSession orchestration | `flow_session.rs:622-2163` (1,542) | Calls crate-private script preparation, listener preparation, rehydration, apply/rollback, host-cycle, and command-drain methods on the `nuxie` façade | Product-side, but requires a narrow public/sealed flow-runtime bridge before crate extraction. Moderate coupling. |
| FlowSession value snapshots | `flow_session.rs:2164-3105` (942) | Traverses public runtime-owned view-model/value objects and builds its own graph/arena | Product-side and cohesive. Mostly mechanical after bridge creation. |
| FlowSession validation/accounting | `flow_session.rs:3106-3545` (440) | Product protocol limits and payload validation | Clean extraction candidate. |
| FlowSession mutation/apply/diff | `flow_session.rs:3546-4515` (970) | Applies mutations through runtime view-model/state-machine handles; shares rollback/command sequencing with façade-private methods | Product-side; transactional sequencing makes it medium risk. |
| FlowSession selection/catalog | `flow_session.rs:4516-5020` (505) | File/artboard/state-machine discovery | Product-side; mostly public runtime surface. |
| FlowSession tests | `flow_session.rs:5021-7227` (2,207; 40 tests), plus `crates/nuxie/tests/flow_session_contract.rs` (335) | Exercises the public ABI, VM lifecycle, and transactional behavior | Move with FlowSession. Preserve public names through compatibility re-exports during migration. |
| Scene contract and IDs | `crates/nuxie/src/scene.rs:28-1187` (1,160) | Public runtime/binary identities and errors | Product-side, cleanly demarcatable. |
| Scene durable definitions/index/specs | `scene.rs:1188-3401` (2,214) | `nuxie_binary::AuthoringRecord`, property/value types, runtime file identities | Product-side authoring model. |
| Scene hierarchy/materialization | `scene.rs:3402-5622` (2,221) | Constructs/reads `RuntimeFile`; directly touches `materialized.file.runtime` in a handful of places even though `File::runtime()` already exists | Product-side but high coupling. Replace direct private field access with the existing public accessor before moving. |
| Silver/scene export schema | `scene.rs:5623-6819` (1,197) | Public exported-record/document DTOs; lowering later feeds `RuntimeFile::from_authoring_records` | Product-side. The schema and its generated counterpart must move together. |
| Scene store/live mount API | `scene.rs:6820-9099` (2,280) | Owns mounted `File`/artboard/view-model state and coordinates transactions | Product-side; medium-to-high coupling. |
| SceneTx and subordinate transactions | `scene.rs:9100-15589` (6,490) | `SceneTx`, `DataConverterTx`, `VmTx`, `AnimTx`, and `MachineTx` mutate the authoring graph, then materialize into runtime/binary types | Core product authoring seam. Cohesive, but atomic commit/materialization behavior makes the move high risk. |
| Scene frame queries/live mutation | `scene.rs:15590-17592` (2,003) | Hit/geometry/semantic queries and intrinsic image sizing reach crate-private `OwnedArtboardInstance` helpers | Product-side. Needs stable snapshot/command DTOs instead of exposing mutable runtime internals. |
| Scene lowering/validation/export | `scene.rs:17593-25563` (7,971) | Converts authoring records into `RuntimeFile`, validates cross-object invariants, and produces exported documents | Product-side. Highest-risk region because it couples the authoring model to port construction invariants. |
| Scene tests | `scene.rs:25564-38224` (12,661; 120 tests), plus `crates/nuxie/tests/scene_authoring.rs` (15,674) and related authoring tests | Broad transaction, materialization, export, and live-runtime contract | Move with Scene; use this suite as the seam migration oracle. |
| Scene schema generator | `crates/nuxie/build.rs` (4,531) and `scene.rs`'s `include!(concat!(env!("OUT_DIR"), "/scene_schema.rs"))` near line 860 | Generates the Scene/silver record schema at build time | Product-side and inseparable from Scene. Extraction must move the build script/generator inputs and preserve the generated include contract. |
| Script import authorization | `crates/nuxie/src/script_import.rs:1-157` production, `158-211` tests | Ed25519/container verification; calls crate-private `File::execution_authorization_for`/authorization state | Product policy. Move to `nuxie-flow`, but expose a capability-based import seam rather than making raw authorization state public. |
| Mixed façade bridge | `crates/nuxie/src/lib.rs`, principally project-envelope classification and private bridge methods around lines 55, 154, 223-258, 307, 344, 376, 1,492, 2,983, 3,328, 4,304, and 5,600-5,733 | `File::from_runtime`; flow script/listener lifecycle; apply/rollback/command drain; artboard geometry/semantic/intrinsic-image helpers | Not an independent feature. Convert these call sites into explicit narrow bridge traits/DTOs, then move their product consumers. |

The concrete crate-private methods the extraction must account for are:

- Flow lifecycle around `nuxie/src/lib.rs:5600-5675`: `prepare_flow_scripts`, `prepare_flow_listener_actions`, rehydrate/apply, begin host cycle, rollback, and drain commands.
- Scene live/runtime queries around `lib.rs:5705-5733`: hit-test path segments with bounds, geometry path segments, retained/semantic text, and intrinsic image-dimension registration.
- `File::from_runtime` around `lib.rs:4304` and the script execution authorization capability.

These are already cross-crate calls from `nuxie` into public `nuxie-runtime` types; the missing visibility is chiefly inside the `nuxie` façade. Extraction should expose a sealed product bridge from the future base façade, not expand dozens of runtime fields to `pub`.

### Project-data converter: product model inside the port

`crates/nuxie-runtime/src/project_data_converter.rs` is 2,686 lines and is explicitly classified `scene-api` in `rust-additions.toml`:

| Lines | Region | Coupling and disposition |
| ---: | --- | --- |
| 22-507 | Public values, specs, errors, and state | Pure product/protocol model; eventual neutral-crate candidate. |
| 508-1,289 | Envelope/program/catalog compilation and evaluation | Mostly `std`/`serde`; eventual neutral-crate candidate. |
| 1,290-2,324 | Validation, coercion, formatting, interpolation | Pure-looking core, but behavior is part of runtime data-bind semantics. Extract only with parity tests. |
| 2,325-2,686 | Formula parser | Mechanically separable after the outer seam is established. |

The file is locally self-contained, but its runtime integration is not. `data_bind/context/context_value.rs` has a `RuntimeDataBindGraphConverter::Project` variant and owns conversion/evaluator state; `data_bind/data_bind_context.rs` constructs it from a script-asset envelope; both runtime and `nuxie` re-export or classify its types; Scene authors and lowers them. A direct move to `nuxie-flow` would invert the desired dependency and make the port depend on the product crate.

Two crate-private numeric-policy helpers in this file (the bounded-list constant near line 467 and helper near line 2,320) are also used by port-side Number-to-List/context conversion. If the model is eventually extracted, those policies should remain in the port or move to a dependency-neutral shared crate; they should not become a product API solely to satisfy the move.

Recommendation: first demarcate this as `project_data_converter/{model, program, evaluator, bridge}` inside `nuxie-runtime`; later extract only the pure model/program/evaluator to a neutral `nuxie-project-data` crate that both runtime and `nuxie-flow` may depend on. The runtime adapter stays in `nuxie-runtime`.

### Rust-only infrastructure that should remain port-side

The following `rust-additions.toml` entries have no one-file C++ correspondence but are not product features:

| Current path/group | Lines | Why it stays in `nuxie-runtime` |
| --- | ---: | --- |
| `data_bind_container.rs`, `data_converter.rs`, `retained_data_bind.rs` | 12 total | Tiny generated/type façade modules over port functionality. |
| `input/mod.rs`, `shapes/mod.rs`, `shapes/paint/mod.rs` | 168 total | Rust namespace/generated-storage coordinators. |
| `objects.rs` | 1,018 | Runtime object registry plus `include!(concat!(env!("OUT_DIR"), "/runtime_objects.rs"))`; generated by `crates/nuxie-runtime/build.rs` (707 lines). This is port construction infrastructure. |
| `properties.rs` | 826 | Generated property interpretation/storage used throughout the port. |
| `state_machine.rs`, `state_machine/instance.rs`, `state_machine/listener_types/mod.rs` | 215 total | Rust module/re-export façade for the ported state-machine implementation. |
| `viewmodel/mod.rs`, `viewmodel/runtime/mod.rs` | 58 total | Rust namespace/re-export façade for ported view-model behavior. |
| `focus.rs` | 7 | Thin public façade over retained focus behavior, not a standalone product feature. |

These codegen/facade additions total about 2,303 source lines excluding the project converter and focus façade. They should be clearly marked as Rust port infrastructure, but moving them to `nuxie-flow` would blur rather than improve the seam.

`crates/nuxie/src/command_queue.rs`, server code, and raw-text support are not included in the Nuxie-only list: their current files are explicitly mapped to upstream C++ correspondence. A richer Rust API around a ported facility does not by itself make that facility product-side.

---

## 4. Recommended seam design

### Decision

Use a **module boundary now and a crate boundary later**.

The immediate structure should make product ownership visible without changing dependencies:

```text
crates/nuxie/src/product/
  flow_session/
  scene/
    model/
    transaction/
    live/
    export/
  script_import/

crates/nuxie-runtime/src/project_data_converter/
  model.rs
  program.rs
  evaluator.rs
  bridge.rs
```

Preserve current public paths with root re-exports. This first step is a namespace/demarcation change, not an extraction. It gives reviewers an auditable boundary while the current test and parity baseline still observes exactly the same crate graph.

The long-term crate graph should avoid a cycle:

```text
nuxie-runtime  <---  nuxie-core  <---  nuxie-flow
       ^                 ^                 ^
       |                 |                 |
       +------ nuxie-project-data --------+

nuxie (umbrella compatibility crate) re-exports nuxie-core + nuxie-flow
```

`nuxie-flow` contains FlowSession, Scene/SceneTx, silver/export DTOs and lowering, and script-import policy. `nuxie-core` contains `File`, `Factory`, owned artboard/view-model handles, renderer integration, and narrow bridge traits. `nuxie-runtime` remains the C++-correspondence port plus explicitly labeled Rust port infrastructure. A small dependency-neutral `nuxie-project-data` is optional and should be introduced only after the runtime adapter has been separated from the pure evaluator.

### Public surface of the port/base layer

The extraction should not make arbitrary runtime fields public. The required surface is narrow:

1. A sealed/trusted constructor that wraps a validated `RuntimeFile` as a façade `File`, with an explicit authoring/import authorization policy.
2. A flow-runtime bridge for prepare, rehydrate, apply, host-cycle, rollback, and command-drain operations. Its inputs/outputs should be stable value DTOs, not references to internal queues.
3. Read-only artboard snapshot operations for hit geometry, retained geometry, and semantic text, plus a command for intrinsic image dimensions. Return owned snapshots so Scene cannot retain internal graph borrows.
4. A project-converter adapter owned by `nuxie-runtime`; pure program/model types may come from the neutral crate. Do not expose the runtime bind graph or evaluator cells as product API.
5. Existing upstream-corresponding runtime types only where they are already public and semantically stable. Product re-exports should live in `nuxie`/`nuxie-flow`, not expand the port's promise.

### Migration order

1. Establish the exact green parity/golden baseline and keep it pinned in every PR.
2. Land the `artboard.rs` and `state_machine_instance.rs` mechanical splits as separate PRs. This makes later ownership changes reviewable without mixing 20,000-line source moves into semantic work.
3. Demarcate `nuxie::product::{flow_session, scene, script_import}` and the project-converter submodules in place, preserving public re-exports.
4. Define the sealed flow, authoring-file, geometry-snapshot, and script-authorization bridges; migrate existing internal callers to them while everything remains in the same crate.
5. Extract the non-product façade/runtime handles into `nuxie-core`, with `nuxie` temporarily acting as an umbrella compatibility crate.
6. Move FlowSession and script import to `nuxie-flow`; move their tests and retain compatibility re-exports.
7. Move Scene model, SceneTx, export schema/generator, lowering, live adapter, and tests together. Do not split the generated schema from `build.rs` or split transaction commit from materialization in the extraction PR.
8. Isolate the project converter's pure model/program/evaluator. Extract it to a neutral crate only if doing so leaves the runtime adapter dependency pointing inward, never from `nuxie-runtime` to `nuxie-flow`.
9. Remove compatibility re-exports and tighten visibility in a later breaking-change PR, after downstream migration.

### Risk map

| Risk | Areas | Reason |
| --- | --- | --- |
| Mechanical/low | Thin namespace façades, codegen coordinators staying in place, test-module extraction, pure FlowSession validation/types | Mostly physical ownership with no runtime mutation semantics. |
| Moderate | FlowSession after bridge creation, script import after capability API, Scene DTO/export types | Public API and authorization compatibility, but limited graph mutation. |
| High | SceneTx commit/materialization, live Scene mounting, geometry/semantic access, Scene lowering and generated schema | Atomicity, identity, generated code, and live runtime invariants cross the seam. |
| High | Project-data converter extraction | The source looks pure, but evaluator state is embedded in runtime data-bind context and Number-to-List policy. |
| High | Script/listener ownership during FlowSession moves | Rehydration, rollback, command queues, and VM object lifetime must remain one coherent transaction. |

### Why baseline-first matters

`docs/parity-gap-register.md` explicitly recommends making the parity claim trustworthy before feature work and landing each subsequent change against the oracle (the “Recommended execution order” around lines 204-211). It also describes Scene as a Nuxie-only additive API (around lines 126-127) while recording FlowSession/capability-boundary gaps elsewhere in the register. That is the governing principle here: first make source moves mechanically observable by the existing correspondence, ownership, trace, golden, and silver batteries; then introduce the product seam behind those same oracles. A green build alone is insufficient because it can preserve compilation while silently weakening ownership scans or changing what “zero divergences” measures.

The two large-file splits are therefore prerequisites for, not part of, the product extraction. Each split keeps the logical Rust module unchanged and updates only physical-address consumers. The seam work then proceeds through explicit bridges and compatibility re-exports, with risky transaction/materialization changes isolated from mechanical moves.

# nuxie-runtime structure report

## Scope and baseline

This is a read-only structure analysis. It proposes no behavior, source, parity-ledger, or ownership-ledger changes.

The inspected worktree was already detached at `e8726db8ffc689d3b19f1c0a55794aa6daf9956d` (`Merge pull request #246 from TheOneLevin/feat/b6-0058-listener-viewmodel-change`, 2026-08-04). The local `origin/main` ref resolves to the same commit. The requested `git fetch origin` and redundant `git checkout --detach origin/main` could not update the shared worktree metadata because the sandbox cannot write the parent repository's `.git/worktrees/nuxie-mr-c12` directory. The analysis is therefore pinned to the locally available `origin/main` commit above, not a newly fetched remote ref. The pre-existing untracked `vtriage-capture/` directory was left untouched.

The two target files have grown beyond the sizes in the request:

| File | Current lines | Production/support | Tests |
| --- | ---: | ---: | ---: |
| `crates/nuxie-runtime/src/artboard.rs` | 23,374 | 12,039 | 11,335 (`205` tests) |
| `crates/nuxie-runtime/src/state_machine/state_machine_instance.rs` | 21,674 | 14,079 | 7,595 (`94` tests) |

The safest split mechanism is literal `include!` fragments, not Rust child modules. An included fragment remains in the original semantic module (`crate::artboard` or `crate::state_machine::state_machine_instance`), so private-name resolution, inherent method paths, sibling access, and demangled symbols remain unchanged. A `mod build;`/`#[path = ...] mod build;` split would introduce new privacy boundaries and make a supposedly mechanical PR into an API refactor.

---

## 1. Large-file split plan: `artboard.rs`

### Current structural map

| Lines | Region |
| ---: | --- |
| 1-106 | Imports and supporting aliases |
| 107-135 | Draw-frame counter and generated `Mat2D` helper |
| 136-160 | `ExternalFontAssetError` |
| 161-181 | `RuntimeArtboardInstanceIdentity` |
| 182-231 | `RuntimeScriptState` and implementations |
| 232-250 | Advancing-component and persistent-dirt fixtures |
| 251-298 | Script advance/update mode enums and implementations |
| 299-327 | Resetting/runtime-component helper types |
| 328-334 | `RuntimeTextStyleFeatureOption` |
| 335-523 | `ArtboardInstance` storage |
| 524-714 | `Clone for ArtboardInstance` |
| 715-739 | `Drop for ArtboardInstance` followed by the existing leaf `include!` declarations |
| 740-773 | Occurrence and frame-advance report types |
| 774-844 | Build context/index/text-flag helpers |
| 845-860 | `RuntimeNestedAnimationInstance` |
| 861-11,133 | Main `impl ArtboardInstance` |
| 11,134-11,147 | Focus-scroll path and focus-bounds transform helper |
| 11,148-11,187 | Small follow-up `impl ArtboardInstance` |
| 11,188-11,226 | Component-list reset helper |
| 11,227-11,595 | `impl RuntimeNestedArtboardInstance` |
| 11,596-11,611 | `impl RuntimeNestedAnimationInstance` |
| 11,612-12,039 | Free construction/nested-artboard helpers |
| 12,040-23,374 | Single `#[cfg(test)] mod tests` (`205` tests) |

The main implementation itself divides along stable responsibility boundaries:

| Lines | Logical responsibility |
| ---: | --- |
| 861-1,104 | Lifecycle, data context, and transient-clone restoration |
| 1,105-2,440 | Build occurrence relationships and constructors |
| 2,441-2,712 | Path/hit initialization and external assets |
| 2,713-3,399 | Scripting lifecycle |
| 3,400-3,989 | Object/component/audio/hit/graph/definition access |
| 3,990-4,152 | Scripted interpolators and state-machine definition selection |
| 4,153-4,480 | Dimensions, world/layout bounds, scrolling, and focus reveal |
| 4,481-5,229 | Generated property façade and text values |
| 5,230-5,568 | Animation/state-machine/view-model occurrences |
| 5,569-6,312 | Component-list lifecycle and layout |
| 6,313-7,408 | Root/nested frame advance and retained-component dispatch |
| 7,409-7,585 | Nested-layout frame propagation |
| 7,586-8,496 | Transforms, epochs, dirt, and invalidation |
| 8,497-9,191 | Update pass and five-pass state-machine settlement |
| 9,192-9,825 | Component update dispatch and script scheduling |
| 9,826-11,133 | Property-change callbacks, nested controls, and collapse |

### Proposed physical layout

Keep `artboard.rs` as the owner. It retains imports, supporting types, `ArtboardInstance`, `Clone`, `Drop`, the existing leaf includes, occurrence/report/build helper types, and an ordered series of literal includes. This preserves declaration order where it matters and makes all fragments part of `crate::artboard`.

```text
src/
  artboard.rs                         owner/types, existing leaf includes, ordered include! list
  artboard/
    build.rs                          current 861-2712
    script_runtime.rs                 current 2713-3399
    access.rs                         current 3400-3989
    definitions_and_layout.rs         current 3990-4480
    properties.rs                     current 4481-5568
    component_lists.rs                current 5569-6312
    advance.rs                        current 6313-7585
    dirt.rs                           current 7586-8496
    update.rs                         current 8497-9825
    callbacks.rs                      current 9826-11133
    nested.rs                         current 11134-12039
    tests.rs                          body of current tests module
```

The production fragments are intentionally in execution/dependency order rather than alphabetical order. `tests.rs` should be included inside the original test module:

```rust
#[cfg(test)]
mod tests {
    include!("artboard/tests.rs");
}
```

This retains the test module's name and access. It also gives the attribution checker a deliberately exempt `tests.rs` leaf.

### Lockstep correspondence and checker manifest

The following table is the required per-file edit manifest. “B6 rows” are `[[files]]` entries in `file-correspondence-manifest.toml`; add the fragment's path to those rows' semicolon-separated `rust_module` values and update their C17 retained-module notes. Keep status, upstream pins, C++ paths, verification text, and correspondence claims unchanged. The row list is conservative: a later provenance cleanup can narrow it, but narrowing must not be mixed into this mechanical split.

| New file | B6 rows that must name it | Frame-loop/checker bindings that must move with it |
| --- | --- | --- |
| `artboard/build.rs` | `B6-0001`, `B6-0094`, `B6-0115`, `B6-0117`, `B6-0123`, `B6-0125`, `B6-0129`, `B6-0131`, `B6-0142`, `B6-0146`, `B6-0202`, `B6-0258`, `B6-0303`, `B6-0304`, `B6-0305`, `B6-0331`, `B6-0361`, `B6-0362`, `B6-0365`, `B6-0368`, `B6-0385`, `B6-0388`, `B6-0399`, `B6-0405`, `B6-0408` | Repoint `summarize_trace.py` dependency-build and IK-chain-build source scans when their anchors land here. Repoint matching `runtime-frame-loop-gaps.toml` owner-boundary entries. |
| `artboard/script_runtime.rs` | `B6-0077`, `B6-0194`, `B6-0200`, `B6-0322`, `B6-0326` | Repoint the script-lifecycle citations in `runtime-frame-loop-ownership.toml` and any corresponding synthetic test fixture paths. |
| `artboard/access.rs` | `B6-0094`, `B6-0095`, `B6-0123`, `B6-0146`, `B6-0203`, `B6-0204`, `B6-0205` | Repoint any live-draw/renderer-interface lifecycle citations whose named methods move here. |
| `artboard/definitions_and_layout.rs` | `B6-0077`, `B6-0094`, `B6-0258`, `B6-0303`, `B6-0304`, `B6-0305`, `B6-0385`, `B6-0388` | Repoint focus-retained-tree and layout lifecycle citations, plus matching owner-boundary allow entries. |
| `artboard/properties.rs` | `B6-0067`, `B6-0077`, `B6-0094`, `B6-0194`, `B6-0200`, `B6-0352`, `B6-0355`, `B6-0385`, `B6-0388`, `B6-0399`, `B6-0404`, `B6-0405`, `B6-0408` | `B6-0067` currently has only one Rust module: **replace** `artboard.rs` with this fragment rather than adding a second path. Repoint property/text lifecycle citations and corresponding owner-boundary entries. |
| `artboard/component_lists.rs` | `B6-0095`, `B6-0258`, `B6-0303`, `B6-0304`, `B6-0305` | Repoint component-list and nested-layout lifecycle citations and matching test fixtures. |
| `artboard/advance.rs` | `B6-0001`, `B6-0059`, `B6-0061`, `B6-0063`, `B6-0064`, `B6-0077`, `B6-0094`, `B6-0095`, `B6-0194`, `B6-0200`, `B6-0258`, `B6-0303`, `B6-0304`, `B6-0305`, `B6-0322`, `B6-0326`, `B6-0388` | In `runtime-frame-loop-ownership.toml`, move `artboard.advance`'s `rust_file` and the lifecycle citations for artboard/nested advance. Repoint `summarize_trace.py` advancing/resetting dispatch scans as applicable. Repoint relevant gap allow rows and `test_check.py` advance fixtures. |
| `artboard/dirt.rs` | `B6-0094`, `B6-0123`, `B6-0258`, `B6-0331`, `B6-0352`, `B6-0355`, `B6-0357`, `B6-0361`, `B6-0362`, `B6-0365`, `B6-0368`, `B6-0385`, `B6-0388`, `B6-0399`, `B6-0405`, `B6-0408` | Move `component.dirt`, `component.dependents` (`pub fn add_dirt(`), and transform/dirt lifecycle citations as appropriate. Repoint `summarize_trace.py`'s two add-dirt anchors, dirt consumptions, owner resolutions, and skin-buffer scans according to their final physical locations. Repoint matching gap rows. |
| `artboard/update.rs` | `B6-0001`, `B6-0094`, `B6-0115`, `B6-0117`, `B6-0123`, `B6-0125`, `B6-0129`, `B6-0131`, `B6-0142`, `B6-0194`, `B6-0200`, `B6-0203`, `B6-0204`, `B6-0205`, `B6-0258`, `B6-0303`, `B6-0304`, `B6-0305`, `B6-0322`, `B6-0326`, `B6-0331`, `B6-0352`, `B6-0355`, `B6-0357`, `B6-0361`, `B6-0362`, `B6-0365`, `B6-0368`, `B6-0385`, `B6-0388`, `B6-0399`, `B6-0404`, `B6-0405`, `B6-0408` | Move `component.update_order` (`fn update_components_with_hook_recording`) and `artboard.update_pass` (`pub fn update_pass(`) `rust_file` values and lifecycle citations. Repoint retained update/settlement gap rows, trace scans, and test fixtures. |
| `artboard/callbacks.rs` | `B6-0059`, `B6-0061`, `B6-0063`, `B6-0064`, `B6-0077`, `B6-0094`, `B6-0123`, `B6-0125`, `B6-0129`, `B6-0131`, `B6-0142`, `B6-0194`, `B6-0200`, `B6-0258`, `B6-0303`, `B6-0304`, `B6-0305`, `B6-0331`, `B6-0352`, `B6-0355`, `B6-0357`, `B6-0361`, `B6-0362`, `B6-0365`, `B6-0368`, `B6-0385`, `B6-0388`, `B6-0399`, `B6-0404`, `B6-0405`, `B6-0408` | Change `check_layout_style_handlers.py`'s `RUST_ARTBOARD_MODULE` to this fragment if both display-change markers remain here; otherwise make the checker accept the two explicit physical files. Repoint callback lifecycle citations, gap rows, and fixtures. |
| `artboard/nested.rs` | `B6-0059`, `B6-0061`, `B6-0063`, `B6-0064`, `B6-0077`, `B6-0094`, `B6-0095`, `B6-0258`, `B6-0303`, `B6-0304`, `B6-0305` | Repoint nested-artboard advance/layout citations, gap allow entries, and nested fixtures. |
| `artboard/tests.rs` | None. `rust_attribution.py` excludes a source whose stem is `tests`. | Move the artboard test body unchanged. Update the seven relative `include_bytes!`/fixture paths currently near old lines 19,951, 20,001, 20,168, 20,542, 20,597, 20,612, and 23,311 because the file becomes one directory deeper. Prefer `concat!(env!("CARGO_MANIFEST_DIR"), "/...")` for fixture stability. The `env!` use near old line 14,290 remains valid. |

#### Global artboard bindings

These bindings span more than one fragment and therefore belong in the same PR:

- `rust_attribution.py` discovers all production `.rs` files below `crates/nuxie-runtime/src`. Every new production fragment must be listed in at least one `file-correspondence-manifest.toml` row and must **not** be added to `rust-additions.toml`. The checker exempts `tests.rs`, names ending in `_tests`, and files below a `tests/` path.
- The manifest's scatter ratchet is already at its maximum: 155 rows currently have multiple Rust modules. Except for `B6-0067`, all affected artboard rows are already multi-module. Replacing the `B6-0067` root path prevents the split from increasing that count.
- Update every affected C17 note that enumerates retained Rust modules. Do not change the underlying parity classification merely because the code's physical address changed.
- `docs/runtime-frame-loop-ownership.toml` has artboard citations in `focus.retained_tree`, `component.identity`, `component.dirt`, `component.dependents`, `component.update_order`, `component.transforms`, `component.clone_drop`, `artboard.advance`, `artboard.update_pass`, `nested_artboard.advance`, `artboard.live_draw`, and `renderer.interface_boundary`. Keep `component.clone_drop` and its `pub struct ArtboardInstance` anchor on the root; change each other `rust_file` or `rust:path:line-range` to its physical fragment.
- `docs/runtime-frame-loop-gaps.toml` has 25 owner-boundary allow entries pointing at `artboard.rs` (entries 0-7, 11-22, and 29-34 in current order). Repoint each to the fragment that contains its anchor. A literal move should preserve its token-based `site_hash`; a changed hash is a signal that the split was not mechanical.
- `tools/runtime-frame-loop-port/summarize_trace.py` hard-codes `artboard.rs` nine times for add-dirt, dirt consumption, owner resolution, dependency build, skin rebuild, advancing dispatch, resetting dispatch, and IK-chain discovery. Give each query its actual fragment path. The demangled owner should remain `artboard::ArtboardInstance` because `include!` does not introduce a child module.
- `tools/runtime-frame-loop-port/check_layout_style_handlers.py` hard-codes the artboard source while looking for `propagate_layout_component_display_changed` and the display property key. Point it to the fragment(s) containing those markers.
- `tools/runtime-frame-loop-port/test_check.py` contains at least 14 exact artboard paths covering probe gating, layer occurrence, live-advance ratchets, registry site hashes/substitution, and trait-anchor behavior. Update each synthetic fixture to the fragment that owns the tested anchor; do not perform a blind string replacement.
- Leave the existing root-level includes (`nested_artboard.rs`, `artboard_component_list.rs`, `artboard_list_map_rule.rs`, `artboard_referencer.rs`, `bindable_artboard.rs`, `nested_artboard_layout.rs`, `nested_artboard_leaf.rs`, `component_origin.rs`, bone/weight, profiler/profile) where they are. Moving them under `artboard/` changes relative include resolution and correspondence ownership.
- Keep `crates/nuxie-runtime/src/lib.rs`'s `mod artboard;` unchanged. The public and crate-private paths must not acquire an extra `artboard::<fragment>` segment.

### Mechanical PR sequence and proof

One PR should contain only this file's split and the path/anchor bookkeeping above. Recommended commit sequence within that PR:

1. Extract `tests.rs`, fix only its fixture paths, and prove the same 205 tests are collected.
2. Extract production regions in original order using `include!`; make no renames, visibility edits, formatting sweeps, or logic edits.
3. Update correspondence C17/path fields and all frame-loop physical anchors.
4. Update checker source locations and synthetic fixtures, including an assertion that demangled ownership remains `artboard::ArtboardInstance`.

Proof battery:

```text
make rust-sources-fresh
make cpp-probe
make runtime-frame-loop-port-test
make runtime-frame-loop-port-check
make rust-attribution-check
cargo check --workspace
cargo test -p nuxie-runtime
cargo test -p nuxie --features scripting
make scripted-golden-compare
make silver-corpus-test
```

The PR is structure-preserving only if parity row counts/statuses, trace landmarks, owner-boundary hashes, test counts, and golden/silver outputs are unchanged.

---

## 2. Large-file split plan: `state_machine_instance.rs`

### Current structural map

| Lines | Region |
| ---: | --- |
| 1-93 | Imports and ownership commentary |
| 94-255 | Nested event-chain/notify phases, trace structures, and trace recording |
| 256-312 | Semantic-node resolver and audio/event seam types |
| 313-1,271 | `RuntimeHitResult`, `HitComponent`, and hierarchy implementations |
| 1,272-1,467 | Nested notifier, focus, semantic, and state helper types |
| 1,468-1,474 | `RuntimeDataContextBindError` |
| 1,475-1,711 | `StateMachineInstance` storage |
| 1,712-1,744 | Data-bind occurrence/pointer/executor structures |
| 1,745-2,147 | Listener-action executor implementations |
| 2,148-2,335 | `Clone for StateMachineInstance` |
| 2,336-2,356 | `Drop for StateMachineInstance` |
| 2,357-2,638 | Free helpers and view-model listener types |
| 2,639-13,938 | Main `impl StateMachineInstance` |
| 13,939-13,953 | `runtime_owned_font` helper |
| 13,954-14,079 | `view_model_listener_tests` (`1` test) |
| 14,080-21,674 | `scripted_listener_action_tests` (`93` tests) |

The main implementation divides as follows:

| Lines | Logical responsibility |
| ---: | --- |
| 2,639-2,932 | Teardown, enrollment, focus manager, clone rebind |
| 2,933-3,741 | Construction and listener/layer initialization |
| 3,742-5,102 | Scripting, hydration, and adoption |
| 5,103-6,007 | State/input/focus/semantic/keyboard/gamepad operations |
| 6,008-7,487 | Hit/listener/pointer pipeline |
| 7,488-8,525 | Event notification, deferred callbacks, listener actions |
| 8,526-9,100 | Direct bind targets and value readers |
| 9,101-10,748 | Default/imported/owned source setters and relinking |
| 10,749-12,599 | Data-context bind/rebind, listener cells, bind updates |
| 12,600-13,938 | Reports, advance/apply, nested event dispatch, settlement |

### Proposed physical layout

```text
src/state_machine/
  state_machine_instance.rs               owner/types, hit hierarchy, lifecycle, ordered include! list
  state_machine_instance/
    construction.rs                       current 2639-3741
    scripted_objects.rs                   current 3742-5102
    input_focus.rs                        current 5103-6007
    pointer.rs                            current 6008-7487
    events_actions.rs                     current 7488-8525
    bind_targets.rs                       current 8526-9100
    bind_sources.rs                       current 9101-10748
    data_context.rs                       current 10749-12599
    advance.rs                            current 12600-13953
    tests/
      view_model_listener.rs              body of current first test module
      scripted_listener_actions.rs         body of current second test module
```

Retain the two existing module names in the root and include only their bodies. This preserves test paths and any name-sensitive filtering:

```rust
#[cfg(test)]
mod view_model_listener_tests {
    include!("state_machine_instance/tests/view_model_listener.rs");
}
```

Use the same pattern for `scripted_listener_action_tests`.

### Lockstep correspondence and checker manifest

| New file | B6 rows that must name it | Frame-loop/checker bindings that must move with it |
| --- | --- | --- |
| `state_machine_instance/construction.rs` | `B6-0077`, `B6-0194`, `B6-0195`, `B6-0196`, `B6-0200`, `B6-0310` | Repoint construction/enrollment lifecycle citations and matching synthetic fixtures. Keep `state_machine.collections`' `pub struct StateMachineInstance` anchor on the root. |
| `state_machine_instance/scripted_objects.rs` | `B6-0077`, `B6-0175`, `B6-0194`, `B6-0195`, `B6-0196`, `B6-0200` | Repoint scripting/hydration lifecycle citations and scripted-action fixture paths. |
| `state_machine_instance/input_focus.rs` | `B6-0077`, `B6-0083` | Repoint focus/semantic/keyboard/gamepad citations and tests. |
| `state_machine_instance/pointer.rs` | `B6-0077`, `B6-0083` | Repoint hit/pointer/listener lifecycle citations and FL-C4/layer/hit fixtures. |
| `state_machine_instance/events_actions.rs` | `B6-0058`, `B6-0077`, `B6-0440` | Repoint event reporting, firing/bubbling, and action-dispatch citations/fixtures. `state_machine.actions`' primary anchor remains on the root if the listener-action executor stays there. |
| `state_machine_instance/bind_targets.rs` | `B6-0058`, `B6-0194`, `B6-0196`, `B6-0200`, `B6-0440` | Repoint bind-target and view-model listener citations/fixtures. |
| `state_machine_instance/bind_sources.rs` | `B6-0058`, `B6-0175`, `B6-0194`, `B6-0195`, `B6-0196`, `B6-0200`, `B6-0440` | Repoint source/relink/data-converter citations and bind fixtures. |
| `state_machine_instance/data_context.rs` | `B6-0058`, `B6-0077`, `B6-0194`, `B6-0195`, `B6-0196`, `B6-0200`, `B6-0440` | Repoint data-context lifecycle citations and listener-cell/bind fixtures. |
| `state_machine_instance/advance.rs` | `B6-0077`, `B6-0194`, `B6-0195`, `B6-0196`, `B6-0200`, `B6-0310`, `B6-0440` | Move `state_machine.advance`'s `rust_file` and `advance_with_report_mode` anchor here. Repoint state-machine event/action/reporting lifecycle citations and advance/keyframe fixtures. Include `runtime_owned_font` in this fragment to avoid an artificial one-function file. |
| `state_machine_instance/tests/view_model_listener.rs` | None; it resides below a `tests/` path. | Move the body unchanged and preserve the outer module name. |
| `state_machine_instance/tests/scripted_listener_actions.rs` | None; it resides below a `tests/` path. | Move the body unchanged. Replace or correct the relative fixture path near old line 19,011 (`../../../../fixtures/sync/vm_listener_fire_event.riv`) because the source becomes two directories deeper; prefer a `CARGO_MANIFEST_DIR`-anchored path. |

#### Global state-machine bindings

- All production fragments must be named by `file-correspondence-manifest.toml`, never `rust-additions.toml`. Every affected SMI row is already multi-module, so this split does not increase the manifest's 155-row scatter count.
- Update C17 retained-module notes for `B6-0058`, `B6-0077`, `B6-0083`, `B6-0175`, `B6-0194`, `B6-0195`, `B6-0196`, `B6-0200`, `B6-0310`, and `B6-0440`; leave the substantive parity assertions untouched.
- `docs/runtime-frame-loop-ownership.toml` cites this file from `focus.retained_tree`, `state_machine.collections`, `state_machine.advance`, `state_machine.events`, `state_machine.actions`, and `event.reporting`. Keep root anchors for the struct, stored event collection, and action executor. Repoint method/lifecycle citations to physical fragments.
- `tools/runtime-frame-loop-port/check.py` currently treats one hard-coded SMI file as the nested-event owner and excludes it from outside-owner scans. Once methods are in included fragments, a path-only exclusion would falsely flag the fragments. Make owner discovery include-aware: recursively expand literal local `include!("...")` files in source order for export analysis, and treat every expanded file as the same logical owner. Do not broaden the exclusion to a directory glob.
- Add checker unit coverage for the include-aware logical owner: an exported nested-event method in an included fragment must be accepted, while the same method in an unrelated file must still fail. Preserve syntax parsing and duplicate-anchor detection.
- `tools/runtime-frame-loop-port/test_check.py` has at least 14 exact SMI references, plus constructed/split path strings, covering FL-C4, lifecycle, layer, hit, bind, events, firing/bubbling, advance, owner exports, keyframes, focus, and semantic behavior. Point each fixture to the actual fragment; do not simply substitute the root path everywhere.
- `docs/runtime-frame-loop-gaps.toml` has no current SMI owner-boundary allow row to migrate.
- Keep `crates/nuxie-runtime/src/state_machine.rs`'s `pub(crate) mod state_machine_instance;` and its re-exports unchanged. Literal includes keep all symbols at the original module path.

### Mechanical PR sequence and proof

This should be a second, independent PR after the artboard split has landed. It uses the same proof battery. The checker must first learn the logical-owner/include relationship in the same PR, because moving the methods without that update creates false ownership failures. Apart from that narrowly required path-awareness, no checker policy should change.

Success means the same 94 tests are collected, no exported API path changes, the correspondence scatter count stays at 155, no new owner-boundary allowance appears, and the full battery shown in the artboard plan remains green.

---

## 3. Nuxie-only seam inventory

### Classification method

The authoritative starting point is correspondence metadata:

- Production Rust paths mapped to upstream C++ in `file-correspondence-manifest.toml` are port-side unless a smaller product-only region can be proven.
- Production Rust paths listed in `rust-additions.toml` are intentional Rust-only additions. The relevant ownership labels are `flowsession-abi`, `scene-api`, and generated/codegen infrastructure.
- `build.rs` is outside the attribution scanner's `src/` roots, so it must be inspected manually.
- A convenience façade is not automatically product-side. Thin Rust-only namespace/generated-storage coordinators that serve the port stay with the port even though no one C++ file corresponds to them.

This prevents two opposite mistakes: burying product authoring features inside the port forever, and extracting Rust implementation infrastructure merely because its file has no one-to-one C++ source.

### Product features and mixed glue

| Region | Current size and location | Runtime internals reached / extraction surface | Placement assessment |
| --- | --- | --- | --- |
| FlowSession wire/domain model | `crates/nuxie/src/flow_session.rs:21-621` (601 lines) | Public `File`, `Factory`, renderer, artboard, view-model, and state-machine types | Clean product vocabulary; move to `nuxie-flow` after dependency direction is fixed. |
| FlowSession orchestration | `flow_session.rs:622-2163` (1,542) | Calls crate-private script preparation, listener preparation, rehydration, apply/rollback, host-cycle, and command-drain methods on the `nuxie` façade | Product-side, but requires a narrow public/sealed flow-runtime bridge before crate extraction. Moderate coupling. |
| FlowSession value snapshots | `flow_session.rs:2164-3105` (942) | Traverses public runtime-owned view-model/value objects and builds its own graph/arena | Product-side and cohesive. Mostly mechanical after bridge creation. |
| FlowSession validation/accounting | `flow_session.rs:3106-3545` (440) | Product protocol limits and payload validation | Clean extraction candidate. |
| FlowSession mutation/apply/diff | `flow_session.rs:3546-4515` (970) | Applies mutations through runtime view-model/state-machine handles; shares rollback/command sequencing with façade-private methods | Product-side; transactional sequencing makes it medium risk. |
| FlowSession selection/catalog | `flow_session.rs:4516-5020` (505) | File/artboard/state-machine discovery | Product-side; mostly public runtime surface. |
| FlowSession tests | `flow_session.rs:5021-7227` (2,207; 40 tests), plus `crates/nuxie/tests/flow_session_contract.rs` (335) | Exercises the public ABI, VM lifecycle, and transactional behavior | Move with FlowSession. Preserve public names through compatibility re-exports during migration. |
| Scene contract and IDs | `crates/nuxie/src/scene.rs:28-1187` (1,160) | Public runtime/binary identities and errors | Product-side, cleanly demarcatable. |
| Scene durable definitions/index/specs | `scene.rs:1188-3401` (2,214) | `nuxie_binary::AuthoringRecord`, property/value types, runtime file identities | Product-side authoring model. |
| Scene hierarchy/materialization | `scene.rs:3402-5622` (2,221) | Constructs/reads `RuntimeFile`; directly touches `materialized.file.runtime` in a handful of places even though `File::runtime()` already exists | Product-side but high coupling. Replace direct private field access with the existing public accessor before moving. |
| Silver/scene export schema | `scene.rs:5623-6819` (1,197) | Public exported-record/document DTOs; lowering later feeds `RuntimeFile::from_authoring_records` | Product-side. The schema and its generated counterpart must move together. |
| Scene store/live mount API | `scene.rs:6820-9099` (2,280) | Owns mounted `File`/artboard/view-model state and coordinates transactions | Product-side; medium-to-high coupling. |
| SceneTx and subordinate transactions | `scene.rs:9100-15589` (6,490) | `SceneTx`, `DataConverterTx`, `VmTx`, `AnimTx`, and `MachineTx` mutate the authoring graph, then materialize into runtime/binary types | Core product authoring seam. Cohesive, but atomic commit/materialization behavior makes the move high risk. |
| Scene frame queries/live mutation | `scene.rs:15590-17592` (2,003) | Hit/geometry/semantic queries and intrinsic image sizing reach crate-private `OwnedArtboardInstance` helpers | Product-side. Needs stable snapshot/command DTOs instead of exposing mutable runtime internals. |
| Scene lowering/validation/export | `scene.rs:17593-25563` (7,971) | Converts authoring records into `RuntimeFile`, validates cross-object invariants, and produces exported documents | Product-side. Highest-risk region because it couples the authoring model to port construction invariants. |
| Scene tests | `scene.rs:25564-38224` (12,661; 120 tests), plus `crates/nuxie/tests/scene_authoring.rs` (15,674) and related authoring tests | Broad transaction, materialization, export, and live-runtime contract | Move with Scene; use this suite as the seam migration oracle. |
| Scene schema generator | `crates/nuxie/build.rs` (4,531) and `scene.rs`'s `include!(concat!(env!("OUT_DIR"), "/scene_schema.rs"))` near line 860 | Generates the Scene/silver record schema at build time | Product-side and inseparable from Scene. Extraction must move the build script/generator inputs and preserve the generated include contract. |
| Script import authorization | `crates/nuxie/src/script_import.rs:1-157` production, `158-211` tests | Ed25519/container verification; calls crate-private `File::execution_authorization_for`/authorization state | Product policy. Move to `nuxie-flow`, but expose a capability-based import seam rather than making raw authorization state public. |
| Mixed façade bridge | `crates/nuxie/src/lib.rs`, principally project-envelope classification and private bridge methods around lines 55, 154, 223-258, 307, 344, 376, 1,492, 2,983, 3,328, 4,304, and 5,600-5,733 | `File::from_runtime`; flow script/listener lifecycle; apply/rollback/command drain; artboard geometry/semantic/intrinsic-image helpers | Not an independent feature. Convert these call sites into explicit narrow bridge traits/DTOs, then move their product consumers. |

The concrete crate-private methods the extraction must account for are:

- Flow lifecycle around `nuxie/src/lib.rs:5600-5675`: `prepare_flow_scripts`, `prepare_flow_listener_actions`, rehydrate/apply, begin host cycle, rollback, and drain commands.
- Scene live/runtime queries around `lib.rs:5705-5733`: hit-test path segments with bounds, geometry path segments, retained/semantic text, and intrinsic image-dimension registration.
- `File::from_runtime` around `lib.rs:4304` and the script execution authorization capability.

These are already cross-crate calls from `nuxie` into public `nuxie-runtime` types; the missing visibility is chiefly inside the `nuxie` façade. Extraction should expose a sealed product bridge from the future base façade, not expand dozens of runtime fields to `pub`.

### Project-data converter: product model inside the port

`crates/nuxie-runtime/src/project_data_converter.rs` is 2,686 lines and is explicitly classified `scene-api` in `rust-additions.toml`:

| Lines | Region | Coupling and disposition |
| ---: | --- | --- |
| 22-507 | Public values, specs, errors, and state | Pure product/protocol model; eventual neutral-crate candidate. |
| 508-1,289 | Envelope/program/catalog compilation and evaluation | Mostly `std`/`serde`; eventual neutral-crate candidate. |
| 1,290-2,324 | Validation, coercion, formatting, interpolation | Pure-looking core, but behavior is part of runtime data-bind semantics. Extract only with parity tests. |
| 2,325-2,686 | Formula parser | Mechanically separable after the outer seam is established. |

The file is locally self-contained, but its runtime integration is not. `data_bind/context/context_value.rs` has a `RuntimeDataBindGraphConverter::Project` variant and owns conversion/evaluator state; `data_bind/data_bind_context.rs` constructs it from a script-asset envelope; both runtime and `nuxie` re-export or classify its types; Scene authors and lowers them. A direct move to `nuxie-flow` would invert the desired dependency and make the port depend on the product crate.

Two crate-private numeric-policy helpers in this file (the bounded-list constant near line 467 and helper near line 2,320) are also used by port-side Number-to-List/context conversion. If the model is eventually extracted, those policies should remain in the port or move to a dependency-neutral shared crate; they should not become a product API solely to satisfy the move.

Recommendation: first demarcate this as `project_data_converter/{model, program, evaluator, bridge}` inside `nuxie-runtime`; later extract only the pure model/program/evaluator to a neutral `nuxie-project-data` crate that both runtime and `nuxie-flow` may depend on. The runtime adapter stays in `nuxie-runtime`.

### Rust-only infrastructure that should remain port-side

The following `rust-additions.toml` entries have no one-file C++ correspondence but are not product features:

| Current path/group | Lines | Why it stays in `nuxie-runtime` |
| --- | ---: | --- |
| `data_bind_container.rs`, `data_converter.rs`, `retained_data_bind.rs` | 12 total | Tiny generated/type façade modules over port functionality. |
| `input/mod.rs`, `shapes/mod.rs`, `shapes/paint/mod.rs` | 168 total | Rust namespace/generated-storage coordinators. |
| `objects.rs` | 1,018 | Runtime object registry plus `include!(concat!(env!("OUT_DIR"), "/runtime_objects.rs"))`; generated by `crates/nuxie-runtime/build.rs` (707 lines). This is port construction infrastructure. |
| `properties.rs` | 826 | Generated property interpretation/storage used throughout the port. |
| `state_machine.rs`, `state_machine/instance.rs`, `state_machine/listener_types/mod.rs` | 215 total | Rust module/re-export façade for the ported state-machine implementation. |
| `viewmodel/mod.rs`, `viewmodel/runtime/mod.rs` | 58 total | Rust namespace/re-export façade for ported view-model behavior. |
| `focus.rs` | 7 | Thin public façade over retained focus behavior, not a standalone product feature. |

These codegen/facade additions total about 2,303 source lines excluding the project converter and focus façade. They should be clearly marked as Rust port infrastructure, but moving them to `nuxie-flow` would blur rather than improve the seam.

`crates/nuxie/src/command_queue.rs`, server code, and raw-text support are not included in the Nuxie-only list: their current files are explicitly mapped to upstream C++ correspondence. A richer Rust API around a ported facility does not by itself make that facility product-side.

---

## 4. Recommended seam design

### Decision

Use a **module boundary now and a crate boundary later**.

The immediate structure should make product ownership visible without changing dependencies:

```text
crates/nuxie/src/product/
  flow_session/
  scene/
    model/
    transaction/
    live/
    export/
  script_import/

crates/nuxie-runtime/src/project_data_converter/
  model.rs
  program.rs
  evaluator.rs
  bridge.rs
```

Preserve current public paths with root re-exports. This first step is a namespace/demarcation change, not an extraction. It gives reviewers an auditable boundary while the current test and parity baseline still observes exactly the same crate graph.

The long-term crate graph should avoid a cycle:

```text
nuxie-runtime  <---  nuxie-core  <---  nuxie-flow
       ^                 ^                 ^
       |                 |                 |
       +------ nuxie-project-data --------+

nuxie (umbrella compatibility crate) re-exports nuxie-core + nuxie-flow
```

`nuxie-flow` contains FlowSession, Scene/SceneTx, silver/export DTOs and lowering, and script-import policy. `nuxie-core` contains `File`, `Factory`, owned artboard/view-model handles, renderer integration, and narrow bridge traits. `nuxie-runtime` remains the C++-correspondence port plus explicitly labeled Rust port infrastructure. A small dependency-neutral `nuxie-project-data` is optional and should be introduced only after the runtime adapter has been separated from the pure evaluator.

### Public surface of the port/base layer

The extraction should not make arbitrary runtime fields public. The required surface is narrow:

1. A sealed/trusted constructor that wraps a validated `RuntimeFile` as a façade `File`, with an explicit authoring/import authorization policy.
2. A flow-runtime bridge for prepare, rehydrate, apply, host-cycle, rollback, and command-drain operations. Its inputs/outputs should be stable value DTOs, not references to internal queues.
3. Read-only artboard snapshot operations for hit geometry, retained geometry, and semantic text, plus a command for intrinsic image dimensions. Return owned snapshots so Scene cannot retain internal graph borrows.
4. A project-converter adapter owned by `nuxie-runtime`; pure program/model types may come from the neutral crate. Do not expose the runtime bind graph or evaluator cells as product API.
5. Existing upstream-corresponding runtime types only where they are already public and semantically stable. Product re-exports should live in `nuxie`/`nuxie-flow`, not expand the port's promise.

### Migration order

1. Establish the exact green parity/golden baseline and keep it pinned in every PR.
2. Land the `artboard.rs` and `state_machine_instance.rs` mechanical splits as separate PRs. This makes later ownership changes reviewable without mixing 20,000-line source moves into semantic work.
3. Demarcate `nuxie::product::{flow_session, scene, script_import}` and the project-converter submodules in place, preserving public re-exports.
4. Define the sealed flow, authoring-file, geometry-snapshot, and script-authorization bridges; migrate existing internal callers to them while everything remains in the same crate.
5. Extract the non-product façade/runtime handles into `nuxie-core`, with `nuxie` temporarily acting as an umbrella compatibility crate.
6. Move FlowSession and script import to `nuxie-flow`; move their tests and retain compatibility re-exports.
7. Move Scene model, SceneTx, export schema/generator, lowering, live adapter, and tests together. Do not split the generated schema from `build.rs` or split transaction commit from materialization in the extraction PR.
8. Isolate the project converter's pure model/program/evaluator. Extract it to a neutral crate only if doing so leaves the runtime adapter dependency pointing inward, never from `nuxie-runtime` to `nuxie-flow`.
9. Remove compatibility re-exports and tighten visibility in a later breaking-change PR, after downstream migration.

### Risk map

| Risk | Areas | Reason |
| --- | --- | --- |
| Mechanical/low | Thin namespace façades, codegen coordinators staying in place, test-module extraction, pure FlowSession validation/types | Mostly physical ownership with no runtime mutation semantics. |
| Moderate | FlowSession after bridge creation, script import after capability API, Scene DTO/export types | Public API and authorization compatibility, but limited graph mutation. |
| High | SceneTx commit/materialization, live Scene mounting, geometry/semantic access, Scene lowering and generated schema | Atomicity, identity, generated code, and live runtime invariants cross the seam. |
| High | Project-data converter extraction | The source looks pure, but evaluator state is embedded in runtime data-bind context and Number-to-List policy. |
| High | Script/listener ownership during FlowSession moves | Rehydration, rollback, command queues, and VM object lifetime must remain one coherent transaction. |

### Why baseline-first matters

`docs/parity-gap-register.md` explicitly recommends making the parity claim trustworthy before feature work and landing each subsequent change against the oracle (the “Recommended execution order” around lines 204-211). It also describes Scene as a Nuxie-only additive API (around lines 126-127) while recording FlowSession/capability-boundary gaps elsewhere in the register. That is the governing principle here: first make source moves mechanically observable by the existing correspondence, ownership, trace, golden, and silver batteries; then introduce the product seam behind those same oracles. A green build alone is insufficient because it can preserve compilation while silently weakening ownership scans or changing what “zero divergences” measures.

The two large-file splits are therefore prerequisites for, not part of, the product extraction. Each split keeps the logical Rust module unchanged and updates only physical-address consumers. The seam work then proceeds through explicit bridges and compatibility re-exports, with risky transaction/materialization changes isolated from mechanical moves.

# nuxie-runtime structure report

## Scope and baseline

This is a read-only structure analysis. It proposes no behavior, source, parity-ledger, or ownership-ledger changes.

The inspected worktree was already detached at `e8726db8ffc689d3b19f1c0a55794aa6daf9956d` (`Merge pull request #246 from TheOneLevin/feat/b6-0058-listener-viewmodel-change`, 2026-08-04). The local `origin/main` ref resolves to the same commit. The requested `git fetch origin` and redundant `git checkout --detach origin/main` could not update the shared worktree metadata because the sandbox cannot write the parent repository's `.git/worktrees/nuxie-mr-c12` directory. The analysis is therefore pinned to the locally available `origin/main` commit above, not a newly fetched remote ref. The pre-existing untracked `vtriage-capture/` directory was left untouched.

The two target files have grown beyond the sizes in the request:

| File | Current lines | Production/support | Tests |
| --- | ---: | ---: | ---: |
| `crates/nuxie-runtime/src/artboard.rs` | 23,374 | 12,039 | 11,335 (`205` tests) |
| `crates/nuxie-runtime/src/state_machine/state_machine_instance.rs` | 21,674 | 14,079 | 7,595 (`94` tests) |

The safest split mechanism is literal `include!` fragments, not Rust child modules. An included fragment remains in the original semantic module (`crate::artboard` or `crate::state_machine::state_machine_instance`), so private-name resolution, inherent method paths, sibling access, and demangled symbols remain unchanged. A `mod build;`/`#[path = ...] mod build;` split would introduce new privacy boundaries and make a supposedly mechanical PR into an API refactor.

---

## 1. Large-file split plan: `artboard.rs`

### Current structural map

| Lines | Region |
| ---: | --- |
| 1-106 | Imports and supporting aliases |
| 107-135 | Draw-frame counter and generated `Mat2D` helper |
| 136-160 | `ExternalFontAssetError` |
| 161-181 | `RuntimeArtboardInstanceIdentity` |
| 182-231 | `RuntimeScriptState` and implementations |
| 232-250 | Advancing-component and persistent-dirt fixtures |
| 251-298 | Script advance/update mode enums and implementations |
| 299-327 | Resetting/runtime-component helper types |
| 328-334 | `RuntimeTextStyleFeatureOption` |
| 335-523 | `ArtboardInstance` storage |
| 524-714 | `Clone for ArtboardInstance` |
| 715-739 | `Drop for ArtboardInstance` followed by the existing leaf `include!` declarations |
| 740-773 | Occurrence and frame-advance report types |
| 774-844 | Build context/index/text-flag helpers |
| 845-860 | `RuntimeNestedAnimationInstance` |
| 861-11,133 | Main `impl ArtboardInstance` |
| 11,134-11,147 | Focus-scroll path and focus-bounds transform helper |
| 11,148-11,187 | Small follow-up `impl ArtboardInstance` |
| 11,188-11,226 | Component-list reset helper |
| 11,227-11,595 | `impl RuntimeNestedArtboardInstance` |
| 11,596-11,611 | `impl RuntimeNestedAnimationInstance` |
| 11,612-12,039 | Free construction/nested-artboard helpers |
| 12,040-23,374 | Single `#[cfg(test)] mod tests` (`205` tests) |

The main implementation itself divides along stable responsibility boundaries:

| Lines | Logical responsibility |
| ---: | --- |
| 861-1,104 | Lifecycle, data context, and transient-clone restoration |
| 1,105-2,440 | Build occurrence relationships and constructors |
| 2,441-2,712 | Path/hit initialization and external assets |
| 2,713-3,399 | Scripting lifecycle |
| 3,400-3,989 | Object/component/audio/hit/graph/definition access |
| 3,990-4,152 | Scripted interpolators and state-machine definition selection |
| 4,153-4,480 | Dimensions, world/layout bounds, scrolling, and focus reveal |
| 4,481-5,229 | Generated property façade and text values |
| 5,230-5,568 | Animation/state-machine/view-model occurrences |
| 5,569-6,312 | Component-list lifecycle and layout |
| 6,313-7,408 | Root/nested frame advance and retained-component dispatch |
| 7,409-7,585 | Nested-layout frame propagation |
| 7,586-8,496 | Transforms, epochs, dirt, and invalidation |
| 8,497-9,191 | Update pass and five-pass state-machine settlement |
| 9,192-9,825 | Component update dispatch and script scheduling |
| 9,826-11,133 | Property-change callbacks, nested controls, and collapse |

### Proposed physical layout

Keep `artboard.rs` as the owner. It retains imports, supporting types, `ArtboardInstance`, `Clone`, `Drop`, the existing leaf includes, occurrence/report/build helper types, and an ordered series of literal includes. This preserves declaration order where it matters and makes all fragments part of `crate::artboard`.

```text
src/
  artboard.rs                         owner/types, existing leaf includes, ordered include! list
  artboard/
    build.rs                          current 861-2712
    script_runtime.rs                 current 2713-3399
    access.rs                         current 3400-3989
    definitions_and_layout.rs         current 3990-4480
    properties.rs                     current 4481-5568
    component_lists.rs                current 5569-6312
    advance.rs                        current 6313-7585
    dirt.rs                           current 7586-8496
    update.rs                         current 8497-9825
    callbacks.rs                      current 9826-11133
    nested.rs                         current 11134-12039
    tests.rs                          body of current tests module
```

The production fragments are intentionally in execution/dependency order rather than alphabetical order. `tests.rs` should be included inside the original test module:

```rust
#[cfg(test)]
mod tests {
    include!("artboard/tests.rs");
}
```

This retains the test module's name and access. It also gives the attribution checker a deliberately exempt `tests.rs` leaf.

### Lockstep correspondence and checker manifest

The following table is the required per-file edit manifest. “B6 rows” are `[[files]]` entries in `file-correspondence-manifest.toml`; add the fragment's path to those rows' semicolon-separated `rust_module` values and update their C17 retained-module notes. Keep status, upstream pins, C++ paths, verification text, and correspondence claims unchanged. The row list is conservative: a later provenance cleanup can narrow it, but narrowing must not be mixed into this mechanical split.

| New file | B6 rows that must name it | Frame-loop/checker bindings that must move with it |
| --- | --- | --- |
| `artboard/build.rs` | `B6-0001`, `B6-0094`, `B6-0115`, `B6-0117`, `B6-0123`, `B6-0125`, `B6-0129`, `B6-0131`, `B6-0142`, `B6-0146`, `B6-0202`, `B6-0258`, `B6-0303`, `B6-0304`, `B6-0305`, `B6-0331`, `B6-0361`, `B6-0362`, `B6-0365`, `B6-0368`, `B6-0385`, `B6-0388`, `B6-0399`, `B6-0405`, `B6-0408` | Repoint `summarize_trace.py` dependency-build and IK-chain-build source scans when their anchors land here. Repoint matching `runtime-frame-loop-gaps.toml` owner-boundary entries. |
| `artboard/script_runtime.rs` | `B6-0077`, `B6-0194`, `B6-0200`, `B6-0322`, `B6-0326` | Repoint the script-lifecycle citations in `runtime-frame-loop-ownership.toml` and any corresponding synthetic test fixture paths. |
| `artboard/access.rs` | `B6-0094`, `B6-0095`, `B6-0123`, `B6-0146`, `B6-0203`, `B6-0204`, `B6-0205` | Repoint any live-draw/renderer-interface lifecycle citations whose named methods move here. |
| `artboard/definitions_and_layout.rs` | `B6-0077`, `B6-0094`, `B6-0258`, `B6-0303`, `B6-0304`, `B6-0305`, `B6-0385`, `B6-0388` | Repoint focus-retained-tree and layout lifecycle citations, plus matching owner-boundary allow entries. |
| `artboard/properties.rs` | `B6-0067`, `B6-0077`, `B6-0094`, `B6-0194`, `B6-0200`, `B6-0352`, `B6-0355`, `B6-0385`, `B6-0388`, `B6-0399`, `B6-0404`, `B6-0405`, `B6-0408` | `B6-0067` currently has only one Rust module: **replace** `artboard.rs` with this fragment rather than adding a second path. Repoint property/text lifecycle citations and corresponding owner-boundary entries. |
| `artboard/component_lists.rs` | `B6-0095`, `B6-0258`, `B6-0303`, `B6-0304`, `B6-0305` | Repoint component-list and nested-layout lifecycle citations and matching test fixtures. |
| `artboard/advance.rs` | `B6-0001`, `B6-0059`, `B6-0061`, `B6-0063`, `B6-0064`, `B6-0077`, `B6-0094`, `B6-0095`, `B6-0194`, `B6-0200`, `B6-0258`, `B6-0303`, `B6-0304`, `B6-0305`, `B6-0322`, `B6-0326`, `B6-0388` | In `runtime-frame-loop-ownership.toml`, move `artboard.advance`'s `rust_file` and the lifecycle citations for artboard/nested advance. Repoint `summarize_trace.py` advancing/resetting dispatch scans as applicable. Repoint relevant gap allow rows and `test_check.py` advance fixtures. |
| `artboard/dirt.rs` | `B6-0094`, `B6-0123`, `B6-0258`, `B6-0331`, `B6-0352`, `B6-0355`, `B6-0357`, `B6-0361`, `B6-0362`, `B6-0365`, `B6-0368`, `B6-0385`, `B6-0388`, `B6-0399`, `B6-0405`, `B6-0408` | Move `component.dirt`, `component.dependents` (`pub fn add_dirt(`), and transform/dirt lifecycle citations as appropriate. Repoint `summarize_trace.py`'s two add-dirt anchors, dirt consumptions, owner resolutions, and skin-buffer scans according to their final physical locations. Repoint matching gap rows. |
| `artboard/update.rs` | `B6-0001`, `B6-0094`, `B6-0115`, `B6-0117`, `B6-0123`, `B6-0125`, `B6-0129`, `B6-0131`, `B6-0142`, `B6-0194`, `B6-0200`, `B6-0203`, `B6-0204`, `B6-0205`, `B6-0258`, `B6-0303`, `B6-0304`, `B6-0305`, `B6-0322`, `B6-0326`, `B6-0331`, `B6-0352`, `B6-0355`, `B6-0357`, `B6-0361`, `B6-0362`, `B6-0365`, `B6-0368`, `B6-0385`, `B6-0388`, `B6-0399`, `B6-0404`, `B6-0405`, `B6-0408` | Move `component.update_order` (`fn update_components_with_hook_recording`) and `artboard.update_pass` (`pub fn update_pass(`) `rust_file` values and lifecycle citations. Repoint retained update/settlement gap rows, trace scans, and test fixtures. |
| `artboard/callbacks.rs` | `B6-0059`, `B6-0061`, `B6-0063`, `B6-0064`, `B6-0077`, `B6-0094`, `B6-0123`, `B6-0125`, `B6-0129`, `B6-0131`, `B6-0142`, `B6-0194`, `B6-0200`, `B6-0258`, `B6-0303`, `B6-0304`, `B6-0305`, `B6-0331`, `B6-0352`, `B6-0355`, `B6-0357`, `B6-0361`, `B6-0362`, `B6-0365`, `B6-0368`, `B6-0385`, `B6-0388`, `B6-0399`, `B6-0404`, `B6-0405`, `B6-0408` | Change `check_layout_style_handlers.py`'s `RUST_ARTBOARD_MODULE` to this fragment if both display-change markers remain here; otherwise make the checker accept the two explicit physical files. Repoint callback lifecycle citations, gap rows, and fixtures. |
| `artboard/nested.rs` | `B6-0059`, `B6-0061`, `B6-0063`, `B6-0064`, `B6-0077`, `B6-0094`, `B6-0095`, `B6-0258`, `B6-0303`, `B6-0304`, `B6-0305` | Repoint nested-artboard advance/layout citations, gap allow entries, and nested fixtures. |
| `artboard/tests.rs` | None. `rust_attribution.py` excludes a source whose stem is `tests`. | Move the artboard test body unchanged. Update the seven relative `include_bytes!`/fixture paths currently near old lines 19,951, 20,001, 20,168, 20,542, 20,597, 20,612, and 23,311 because the file becomes one directory deeper. Prefer `concat!(env!("CARGO_MANIFEST_DIR"), "/...")` for fixture stability. The `env!` use near old line 14,290 remains valid. |

#### Global artboard bindings

These bindings span more than one fragment and therefore belong in the same PR:

- `rust_attribution.py` discovers all production `.rs` files below `crates/nuxie-runtime/src`. Every new production fragment must be listed in at least one `file-correspondence-manifest.toml` row and must **not** be added to `rust-additions.toml`. The checker exempts `tests.rs`, names ending in `_tests`, and files below a `tests/` path.
- The manifest's scatter ratchet is already at its maximum: 155 rows currently have multiple Rust modules. Except for `B6-0067`, all affected artboard rows are already multi-module. Replacing the `B6-0067` root path prevents the split from increasing that count.
- Update every affected C17 note that enumerates retained Rust modules. Do not change the underlying parity classification merely because the code's physical address changed.
- `docs/runtime-frame-loop-ownership.toml` has artboard citations in `focus.retained_tree`, `component.identity`, `component.dirt`, `component.dependents`, `component.update_order`, `component.transforms`, `component.clone_drop`, `artboard.advance`, `artboard.update_pass`, `nested_artboard.advance`, `artboard.live_draw`, and `renderer.interface_boundary`. Keep `component.clone_drop` and its `pub struct ArtboardInstance` anchor on the root; change each other `rust_file` or `rust:path:line-range` to its physical fragment.
- `docs/runtime-frame-loop-gaps.toml` has 25 owner-boundary allow entries pointing at `artboard.rs` (entries 0-7, 11-22, and 29-34 in current order). Repoint each to the fragment that contains its anchor. A literal move should preserve its token-based `site_hash`; a changed hash is a signal that the split was not mechanical.
- `tools/runtime-frame-loop-port/summarize_trace.py` hard-codes `artboard.rs` nine times for add-dirt, dirt consumption, owner resolution, dependency build, skin rebuild, advancing dispatch, resetting dispatch, and IK-chain discovery. Give each query its actual fragment path. The demangled owner should remain `artboard::ArtboardInstance` because `include!` does not introduce a child module.
- `tools/runtime-frame-loop-port/check_layout_style_handlers.py` hard-codes the artboard source while looking for `propagate_layout_component_display_changed` and the display property key. Point it to the fragment(s) containing those markers.
- `tools/runtime-frame-loop-port/test_check.py` contains at least 14 exact artboard paths covering probe gating, layer occurrence, live-advance ratchets, registry site hashes/substitution, and trait-anchor behavior. Update each synthetic fixture to the fragment that owns the tested anchor; do not perform a blind string replacement.
- Leave the existing root-level includes (`nested_artboard.rs`, `artboard_component_list.rs`, `artboard_list_map_rule.rs`, `artboard_referencer.rs`, `bindable_artboard.rs`, `nested_artboard_layout.rs`, `nested_artboard_leaf.rs`, `component_origin.rs`, bone/weight, profiler/profile) where they are. Moving them under `artboard/` changes relative include resolution and correspondence ownership.
- Keep `crates/nuxie-runtime/src/lib.rs`'s `mod artboard;` unchanged. The public and crate-private paths must not acquire an extra `artboard::<fragment>` segment.

### Mechanical PR sequence and proof

One PR should contain only this file's split and the path/anchor bookkeeping above. Recommended commit sequence within that PR:

1. Extract `tests.rs`, fix only its fixture paths, and prove the same 205 tests are collected.
2. Extract production regions in original order using `include!`; make no renames, visibility edits, formatting sweeps, or logic edits.
3. Update correspondence C17/path fields and all frame-loop physical anchors.
4. Update checker source locations and synthetic fixtures, including an assertion that demangled ownership remains `artboard::ArtboardInstance`.

Proof battery:

```text
make rust-sources-fresh
make cpp-probe
make runtime-frame-loop-port-test
make runtime-frame-loop-port-check
make rust-attribution-check
cargo check --workspace
cargo test -p nuxie-runtime
cargo test -p nuxie --features scripting
make scripted-golden-compare
make silver-corpus-test
```

The PR is structure-preserving only if parity row counts/statuses, trace landmarks, owner-boundary hashes, test counts, and golden/silver outputs are unchanged.

---

## 2. Large-file split plan: `state_machine_instance.rs`

### Current structural map

| Lines | Region |
| ---: | --- |
| 1-93 | Imports and ownership commentary |
| 94-255 | Nested event-chain/notify phases, trace structures, and trace recording |
| 256-312 | Semantic-node resolver and audio/event seam types |
| 313-1,271 | `RuntimeHitResult`, `HitComponent`, and hierarchy implementations |
| 1,272-1,467 | Nested notifier, focus, semantic, and state helper types |
| 1,468-1,474 | `RuntimeDataContextBindError` |
| 1,475-1,711 | `StateMachineInstance` storage |
| 1,712-1,744 | Data-bind occurrence/pointer/executor structures |
| 1,745-2,147 | Listener-action executor implementations |
| 2,148-2,335 | `Clone for StateMachineInstance` |
| 2,336-2,356 | `Drop for StateMachineInstance` |
| 2,357-2,638 | Free helpers and view-model listener types |
| 2,639-13,938 | Main `impl StateMachineInstance` |
| 13,939-13,953 | `runtime_owned_font` helper |
| 13,954-14,079 | `view_model_listener_tests` (`1` test) |
| 14,080-21,674 | `scripted_listener_action_tests` (`93` tests) |

The main implementation divides as follows:

| Lines | Logical responsibility |
| ---: | --- |
| 2,639-2,932 | Teardown, enrollment, focus manager, clone rebind |
| 2,933-3,741 | Construction and listener/layer initialization |
| 3,742-5,102 | Scripting, hydration, and adoption |
| 5,103-6,007 | State/input/focus/semantic/keyboard/gamepad operations |
| 6,008-7,487 | Hit/listener/pointer pipeline |
| 7,488-8,525 | Event notification, deferred callbacks, listener actions |
| 8,526-9,100 | Direct bind targets and value readers |
| 9,101-10,748 | Default/imported/owned source setters and relinking |
| 10,749-12,599 | Data-context bind/rebind, listener cells, bind updates |
| 12,600-13,938 | Reports, advance/apply, nested event dispatch, settlement |

### Proposed physical layout

```text
src/state_machine/
  state_machine_instance.rs               owner/types, hit hierarchy, lifecycle, ordered include! list
  state_machine_instance/
    construction.rs                       current 2639-3741
    scripted_objects.rs                   current 3742-5102
    input_focus.rs                        current 5103-6007
    pointer.rs                            current 6008-7487
    events_actions.rs                     current 7488-8525
    bind_targets.rs                       current 8526-9100
    bind_sources.rs                       current 9101-10748
    data_context.rs                       current 10749-12599
    advance.rs                            current 12600-13953
    tests/
      view_model_listener.rs              body of current first test module
      scripted_listener_actions.rs         body of current second test module
```

Retain the two existing module names in the root and include only their bodies. This preserves test paths and any name-sensitive filtering:

```rust
#[cfg(test)]
mod view_model_listener_tests {
    include!("state_machine_instance/tests/view_model_listener.rs");
}
```

Use the same pattern for `scripted_listener_action_tests`.

### Lockstep correspondence and checker manifest

| New file | B6 rows that must name it | Frame-loop/checker bindings that must move with it |
| --- | --- | --- |
| `state_machine_instance/construction.rs` | `B6-0077`, `B6-0194`, `B6-0195`, `B6-0196`, `B6-0200`, `B6-0310` | Repoint construction/enrollment lifecycle citations and matching synthetic fixtures. Keep `state_machine.collections`' `pub struct StateMachineInstance` anchor on the root. |
| `state_machine_instance/scripted_objects.rs` | `B6-0077`, `B6-0175`, `B6-0194`, `B6-0195`, `B6-0196`, `B6-0200` | Repoint scripting/hydration lifecycle citations and scripted-action fixture paths. |
| `state_machine_instance/input_focus.rs` | `B6-0077`, `B6-0083` | Repoint focus/semantic/keyboard/gamepad citations and tests. |
| `state_machine_instance/pointer.rs` | `B6-0077`, `B6-0083` | Repoint hit/pointer/listener lifecycle citations and FL-C4/layer/hit fixtures. |
| `state_machine_instance/events_actions.rs` | `B6-0058`, `B6-0077`, `B6-0440` | Repoint event reporting, firing/bubbling, and action-dispatch citations/fixtures. `state_machine.actions`' primary anchor remains on the root if the listener-action executor stays there. |
| `state_machine_instance/bind_targets.rs` | `B6-0058`, `B6-0194`, `B6-0196`, `B6-0200`, `B6-0440` | Repoint bind-target and view-model listener citations/fixtures. |
| `state_machine_instance/bind_sources.rs` | `B6-0058`, `B6-0175`, `B6-0194`, `B6-0195`, `B6-0196`, `B6-0200`, `B6-0440` | Repoint source/relink/data-converter citations and bind fixtures. |
| `state_machine_instance/data_context.rs` | `B6-0058`, `B6-0077`, `B6-0194`, `B6-0195`, `B6-0196`, `B6-0200`, `B6-0440` | Repoint data-context lifecycle citations and listener-cell/bind fixtures. |
| `state_machine_instance/advance.rs` | `B6-0077`, `B6-0194`, `B6-0195`, `B6-0196`, `B6-0200`, `B6-0310`, `B6-0440` | Move `state_machine.advance`'s `rust_file` and `advance_with_report_mode` anchor here. Repoint state-machine event/action/reporting lifecycle citations and advance/keyframe fixtures. Include `runtime_owned_font` in this fragment to avoid an artificial one-function file. |
| `state_machine_instance/tests/view_model_listener.rs` | None; it resides below a `tests/` path. | Move the body unchanged and preserve the outer module name. |
| `state_machine_instance/tests/scripted_listener_actions.rs` | None; it resides below a `tests/` path. | Move the body unchanged. Replace or correct the relative fixture path near old line 19,011 (`../../../../fixtures/sync/vm_listener_fire_event.riv`) because the source becomes two directories deeper; prefer a `CARGO_MANIFEST_DIR`-anchored path. |

#### Global state-machine bindings

- All production fragments must be named by `file-correspondence-manifest.toml`, never `rust-additions.toml`. Every affected SMI row is already multi-module, so this split does not increase the manifest's 155-row scatter count.
- Update C17 retained-module notes for `B6-0058`, `B6-0077`, `B6-0083`, `B6-0175`, `B6-0194`, `B6-0195`, `B6-0196`, `B6-0200`, `B6-0310`, and `B6-0440`; leave the substantive parity assertions untouched.
- `docs/runtime-frame-loop-ownership.toml` cites this file from `focus.retained_tree`, `state_machine.collections`, `state_machine.advance`, `state_machine.events`, `state_machine.actions`, and `event.reporting`. Keep root anchors for the struct, stored event collection, and action executor. Repoint method/lifecycle citations to physical fragments.
- `tools/runtime-frame-loop-port/check.py` currently treats one hard-coded SMI file as the nested-event owner and excludes it from outside-owner scans. Once methods are in included fragments, a path-only exclusion would falsely flag the fragments. Make owner discovery include-aware: recursively expand literal local `include!("...")` files in source order for export analysis, and treat every expanded file as the same logical owner. Do not broaden the exclusion to a directory glob.
- Add checker unit coverage for the include-aware logical owner: an exported nested-event method in an included fragment must be accepted, while the same method in an unrelated file must still fail. Preserve syntax parsing and duplicate-anchor detection.
- `tools/runtime-frame-loop-port/test_check.py` has at least 14 exact SMI references, plus constructed/split path strings, covering FL-C4, lifecycle, layer, hit, bind, events, firing/bubbling, advance, owner exports, keyframes, focus, and semantic behavior. Point each fixture to the actual fragment; do not simply substitute the root path everywhere.
- `docs/runtime-frame-loop-gaps.toml` has no current SMI owner-boundary allow row to migrate.
- Keep `crates/nuxie-runtime/src/state_machine.rs`'s `pub(crate) mod state_machine_instance;` and its re-exports unchanged. Literal includes keep all symbols at the original module path.

### Mechanical PR sequence and proof

This should be a second, independent PR after the artboard split has landed. It uses the same proof battery. The checker must first learn the logical-owner/include relationship in the same PR, because moving the methods without that update creates false ownership failures. Apart from that narrowly required path-awareness, no checker policy should change.

Success means the same 94 tests are collected, no exported API path changes, the correspondence scatter count stays at 155, no new owner-boundary allowance appears, and the full battery shown in the artboard plan remains green.

---

## 3. Nuxie-only seam inventory

### Classification method

The authoritative starting point is correspondence metadata:

- Production Rust paths mapped to upstream C++ in `file-correspondence-manifest.toml` are port-side unless a smaller product-only region can be proven.
- Production Rust paths listed in `rust-additions.toml` are intentional Rust-only additions. The relevant ownership labels are `flowsession-abi`, `scene-api`, and generated/codegen infrastructure.
- `build.rs` is outside the attribution scanner's `src/` roots, so it must be inspected manually.
- A convenience façade is not automatically product-side. Thin Rust-only namespace/generated-storage coordinators that serve the port stay with the port even though no one C++ file corresponds to them.

This prevents two opposite mistakes: burying product authoring features inside the port forever, and extracting Rust implementation infrastructure merely because its file has no one-to-one C++ source.

### Product features and mixed glue

| Region | Current size and location | Runtime internals reached / extraction surface | Placement assessment |
| --- | --- | --- | --- |
| FlowSession wire/domain model | `crates/nuxie/src/flow_session.rs:21-621` (601 lines) | Public `File`, `Factory`, renderer, artboard, view-model, and state-machine types | Clean product vocabulary; move to `nuxie-flow` after dependency direction is fixed. |
| FlowSession orchestration | `flow_session.rs:622-2163` (1,542) | Calls crate-private script preparation, listener preparation, rehydration, apply/rollback, host-cycle, and command-drain methods on the `nuxie` façade | Product-side, but requires a narrow public/sealed flow-runtime bridge before crate extraction. Moderate coupling. |
| FlowSession value snapshots | `flow_session.rs:2164-3105` (942) | Traverses public runtime-owned view-model/value objects and builds its own graph/arena | Product-side and cohesive. Mostly mechanical after bridge creation. |
| FlowSession validation/accounting | `flow_session.rs:3106-3545` (440) | Product protocol limits and payload validation | Clean extraction candidate. |
| FlowSession mutation/apply/diff | `flow_session.rs:3546-4515` (970) | Applies mutations through runtime view-model/state-machine handles; shares rollback/command sequencing with façade-private methods | Product-side; transactional sequencing makes it medium risk. |
| FlowSession selection/catalog | `flow_session.rs:4516-5020` (505) | File/artboard/state-machine discovery | Product-side; mostly public runtime surface. |
| FlowSession tests | `flow_session.rs:5021-7227` (2,207; 40 tests), plus `crates/nuxie/tests/flow_session_contract.rs` (335) | Exercises the public ABI, VM lifecycle, and transactional behavior | Move with FlowSession. Preserve public names through compatibility re-exports during migration. |
| Scene contract and IDs | `crates/nuxie/src/scene.rs:28-1187` (1,160) | Public runtime/binary identities and errors | Product-side, cleanly demarcatable. |
| Scene durable definitions/index/specs | `scene.rs:1188-3401` (2,214) | `nuxie_binary::AuthoringRecord`, property/value types, runtime file identities | Product-side authoring model. |
| Scene hierarchy/materialization | `scene.rs:3402-5622` (2,221) | Constructs/reads `RuntimeFile`; directly touches `materialized.file.runtime` in a handful of places even though `File::runtime()` already exists | Product-side but high coupling. Replace direct private field access with the existing public accessor before moving. |
| Silver/scene export schema | `scene.rs:5623-6819` (1,197) | Public exported-record/document DTOs; lowering later feeds `RuntimeFile::from_authoring_records` | Product-side. The schema and its generated counterpart must move together. |
| Scene store/live mount API | `scene.rs:6820-9099` (2,280) | Owns mounted `File`/artboard/view-model state and coordinates transactions | Product-side; medium-to-high coupling. |
| SceneTx and subordinate transactions | `scene.rs:9100-15589` (6,490) | `SceneTx`, `DataConverterTx`, `VmTx`, `AnimTx`, and `MachineTx` mutate the authoring graph, then materialize into runtime/binary types | Core product authoring seam. Cohesive, but atomic commit/materialization behavior makes the move high risk. |
| Scene frame queries/live mutation | `scene.rs:15590-17592` (2,003) | Hit/geometry/semantic queries and intrinsic image sizing reach crate-private `OwnedArtboardInstance` helpers | Product-side. Needs stable snapshot/command DTOs instead of exposing mutable runtime internals. |
| Scene lowering/validation/export | `scene.rs:17593-25563` (7,971) | Converts authoring records into `RuntimeFile`, validates cross-object invariants, and produces exported documents | Product-side. Highest-risk region because it couples the authoring model to port construction invariants. |
| Scene tests | `scene.rs:25564-38224` (12,661; 120 tests), plus `crates/nuxie/tests/scene_authoring.rs` (15,674) and related authoring tests | Broad transaction, materialization, export, and live-runtime contract | Move with Scene; use this suite as the seam migration oracle. |
| Scene schema generator | `crates/nuxie/build.rs` (4,531) and `scene.rs`'s `include!(concat!(env!("OUT_DIR"), "/scene_schema.rs"))` near line 860 | Generates the Scene/silver record schema at build time | Product-side and inseparable from Scene. Extraction must move the build script/generator inputs and preserve the generated include contract. |
| Script import authorization | `crates/nuxie/src/script_import.rs:1-157` production, `158-211` tests | Ed25519/container verification; calls crate-private `File::execution_authorization_for`/authorization state | Product policy. Move to `nuxie-flow`, but expose a capability-based import seam rather than making raw authorization state public. |
| Mixed façade bridge | `crates/nuxie/src/lib.rs`, principally project-envelope classification and private bridge methods around lines 55, 154, 223-258, 307, 344, 376, 1,492, 2,983, 3,328, 4,304, and 5,600-5,733 | `File::from_runtime`; flow script/listener lifecycle; apply/rollback/command drain; artboard geometry/semantic/intrinsic-image helpers | Not an independent feature. Convert these call sites into explicit narrow bridge traits/DTOs, then move their product consumers. |

The concrete crate-private methods the extraction must account for are:

- Flow lifecycle around `nuxie/src/lib.rs:5600-5675`: `prepare_flow_scripts`, `prepare_flow_listener_actions`, rehydrate/apply, begin host cycle, rollback, and drain commands.
- Scene live/runtime queries around `lib.rs:5705-5733`: hit-test path segments with bounds, geometry path segments, retained/semantic text, and intrinsic image-dimension registration.
- `File::from_runtime` around `lib.rs:4304` and the script execution authorization capability.

These are already cross-crate calls from `nuxie` into public `nuxie-runtime` types; the missing visibility is chiefly inside the `nuxie` façade. Extraction should expose a sealed product bridge from the future base façade, not expand dozens of runtime fields to `pub`.

### Project-data converter: product model inside the port

`crates/nuxie-runtime/src/project_data_converter.rs` is 2,686 lines and is explicitly classified `scene-api` in `rust-additions.toml`:

| Lines | Region | Coupling and disposition |
| ---: | --- | --- |
| 22-507 | Public values, specs, errors, and state | Pure product/protocol model; eventual neutral-crate candidate. |
| 508-1,289 | Envelope/program/catalog compilation and evaluation | Mostly `std`/`serde`; eventual neutral-crate candidate. |
| 1,290-2,324 | Validation, coercion, formatting, interpolation | Pure-looking core, but behavior is part of runtime data-bind semantics. Extract only with parity tests. |
| 2,325-2,686 | Formula parser | Mechanically separable after the outer seam is established. |

The file is locally self-contained, but its runtime integration is not. `data_bind/context/context_value.rs` has a `RuntimeDataBindGraphConverter::Project` variant and owns conversion/evaluator state; `data_bind/data_bind_context.rs` constructs it from a script-asset envelope; both runtime and `nuxie` re-export or classify its types; Scene authors and lowers them. A direct move to `nuxie-flow` would invert the desired dependency and make the port depend on the product crate.

Two crate-private numeric-policy helpers in this file (the bounded-list constant near line 467 and helper near line 2,320) are also used by port-side Number-to-List/context conversion. If the model is eventually extracted, those policies should remain in the port or move to a dependency-neutral shared crate; they should not become a product API solely to satisfy the move.

Recommendation: first demarcate this as `project_data_converter/{model, program, evaluator, bridge}` inside `nuxie-runtime`; later extract only the pure model/program/evaluator to a neutral `nuxie-project-data` crate that both runtime and `nuxie-flow` may depend on. The runtime adapter stays in `nuxie-runtime`.

### Rust-only infrastructure that should remain port-side

The following `rust-additions.toml` entries have no one-file C++ correspondence but are not product features:

| Current path/group | Lines | Why it stays in `nuxie-runtime` |
| --- | ---: | --- |
| `data_bind_container.rs`, `data_converter.rs`, `retained_data_bind.rs` | 12 total | Tiny generated/type façade modules over port functionality. |
| `input/mod.rs`, `shapes/mod.rs`, `shapes/paint/mod.rs` | 168 total | Rust namespace/generated-storage coordinators. |
| `objects.rs` | 1,018 | Runtime object registry plus `include!(concat!(env!("OUT_DIR"), "/runtime_objects.rs"))`; generated by `crates/nuxie-runtime/build.rs` (707 lines). This is port construction infrastructure. |
| `properties.rs` | 826 | Generated property interpretation/storage used throughout the port. |
| `state_machine.rs`, `state_machine/instance.rs`, `state_machine/listener_types/mod.rs` | 215 total | Rust module/re-export façade for the ported state-machine implementation. |
| `viewmodel/mod.rs`, `viewmodel/runtime/mod.rs` | 58 total | Rust namespace/re-export façade for ported view-model behavior. |
| `focus.rs` | 7 | Thin public façade over retained focus behavior, not a standalone product feature. |

These codegen/facade additions total about 2,303 source lines excluding the project converter and focus façade. They should be clearly marked as Rust port infrastructure, but moving them to `nuxie-flow` would blur rather than improve the seam.

`crates/nuxie/src/command_queue.rs`, server code, and raw-text support are not included in the Nuxie-only list: their current files are explicitly mapped to upstream C++ correspondence. A richer Rust API around a ported facility does not by itself make that facility product-side.

---

## 4. Recommended seam design

### Decision

Use a **module boundary now and a crate boundary later**.

The immediate structure should make product ownership visible without changing dependencies:

```text
crates/nuxie/src/product/
  flow_session/
  scene/
    model/
    transaction/
    live/
    export/
  script_import/

crates/nuxie-runtime/src/project_data_converter/
  model.rs
  program.rs
  evaluator.rs
  bridge.rs
```

Preserve current public paths with root re-exports. This first step is a namespace/demarcation change, not an extraction. It gives reviewers an auditable boundary while the current test and parity baseline still observes exactly the same crate graph.

The long-term crate graph should avoid a cycle:

```text
nuxie-runtime  <---  nuxie-core  <---  nuxie-flow
       ^                 ^                 ^
       |                 |                 |
       +------ nuxie-project-data --------+

nuxie (umbrella compatibility crate) re-exports nuxie-core + nuxie-flow
```

`nuxie-flow` contains FlowSession, Scene/SceneTx, silver/export DTOs and lowering, and script-import policy. `nuxie-core` contains `File`, `Factory`, owned artboard/view-model handles, renderer integration, and narrow bridge traits. `nuxie-runtime` remains the C++-correspondence port plus explicitly labeled Rust port infrastructure. A small dependency-neutral `nuxie-project-data` is optional and should be introduced only after the runtime adapter has been separated from the pure evaluator.

### Public surface of the port/base layer

The extraction should not make arbitrary runtime fields public. The required surface is narrow:

1. A sealed/trusted constructor that wraps a validated `RuntimeFile` as a façade `File`, with an explicit authoring/import authorization policy.
2. A flow-runtime bridge for prepare, rehydrate, apply, host-cycle, rollback, and command-drain operations. Its inputs/outputs should be stable value DTOs, not references to internal queues.
3. Read-only artboard snapshot operations for hit geometry, retained geometry, and semantic text, plus a command for intrinsic image dimensions. Return owned snapshots so Scene cannot retain internal graph borrows.
4. A project-converter adapter owned by `nuxie-runtime`; pure program/model types may come from the neutral crate. Do not expose the runtime bind graph or evaluator cells as product API.
5. Existing upstream-corresponding runtime types only where they are already public and semantically stable. Product re-exports should live in `nuxie`/`nuxie-flow`, not expand the port's promise.

### Migration order

1. Establish the exact green parity/golden baseline and keep it pinned in every PR.
2. Land the `artboard.rs` and `state_machine_instance.rs` mechanical splits as separate PRs. This makes later ownership changes reviewable without mixing 20,000-line source moves into semantic work.
3. Demarcate `nuxie::product::{flow_session, scene, script_import}` and the project-converter submodules in place, preserving public re-exports.
4. Define the sealed flow, authoring-file, geometry-snapshot, and script-authorization bridges; migrate existing internal callers to them while everything remains in the same crate.
5. Extract the non-product façade/runtime handles into `nuxie-core`, with `nuxie` temporarily acting as an umbrella compatibility crate.
6. Move FlowSession and script import to `nuxie-flow`; move their tests and retain compatibility re-exports.
7. Move Scene model, SceneTx, export schema/generator, lowering, live adapter, and tests together. Do not split the generated schema from `build.rs` or split transaction commit from materialization in the extraction PR.
8. Isolate the project converter's pure model/program/evaluator. Extract it to a neutral crate only if doing so leaves the runtime adapter dependency pointing inward, never from `nuxie-runtime` to `nuxie-flow`.
9. Remove compatibility re-exports and tighten visibility in a later breaking-change PR, after downstream migration.

### Risk map

| Risk | Areas | Reason |
| --- | --- | --- |
| Mechanical/low | Thin namespace façades, codegen coordinators staying in place, test-module extraction, pure FlowSession validation/types | Mostly physical ownership with no runtime mutation semantics. |
| Moderate | FlowSession after bridge creation, script import after capability API, Scene DTO/export types | Public API and authorization compatibility, but limited graph mutation. |
| High | SceneTx commit/materialization, live Scene mounting, geometry/semantic access, Scene lowering and generated schema | Atomicity, identity, generated code, and live runtime invariants cross the seam. |
| High | Project-data converter extraction | The source looks pure, but evaluator state is embedded in runtime data-bind context and Number-to-List policy. |
| High | Script/listener ownership during FlowSession moves | Rehydration, rollback, command queues, and VM object lifetime must remain one coherent transaction. |

### Why baseline-first matters

`docs/parity-gap-register.md` explicitly recommends making the parity claim trustworthy before feature work and landing each subsequent change against the oracle (the “Recommended execution order” around lines 204-211). It also describes Scene as a Nuxie-only additive API (around lines 126-127) while recording FlowSession/capability-boundary gaps elsewhere in the register. That is the governing principle here: first make source moves mechanically observable by the existing correspondence, ownership, trace, golden, and silver batteries; then introduce the product seam behind those same oracles. A green build alone is insufficient because it can preserve compilation while silently weakening ownership scans or changing what “zero divergences” measures.

The two large-file splits are therefore prerequisites for, not part of, the product extraction. Each split keeps the logical Rust module unchanged and updates only physical-address consumers. The seam work then proceeds through explicit bridges and compatibility re-exports, with risky transaction/materialization changes isolated from mechanical moves.

# nuxie-runtime structure report

## Scope and baseline

This is a read-only structure analysis. It proposes no behavior, source, parity-ledger, or ownership-ledger changes.

The inspected worktree was already detached at `e8726db8ffc689d3b19f1c0a55794aa6daf9956d` (`Merge pull request #246 from TheOneLevin/feat/b6-0058-listener-viewmodel-change`, 2026-08-04). The local `origin/main` ref resolves to the same commit. The requested `git fetch origin` and redundant `git checkout --detach origin/main` could not update the shared worktree metadata because the sandbox cannot write the parent repository's `.git/worktrees/nuxie-mr-c12` directory. The analysis is therefore pinned to the locally available `origin/main` commit above, not a newly fetched remote ref. The pre-existing untracked `vtriage-capture/` directory was left untouched.

The two target files have grown beyond the sizes in the request:

| File | Current lines | Production/support | Tests |
| --- | ---: | ---: | ---: |
| `crates/nuxie-runtime/src/artboard.rs` | 23,374 | 12,039 | 11,335 (`205` tests) |
| `crates/nuxie-runtime/src/state_machine/state_machine_instance.rs` | 21,674 | 14,079 | 7,595 (`94` tests) |

The safest split mechanism is literal `include!` fragments, not Rust child modules. An included fragment remains in the original semantic module (`crate::artboard` or `crate::state_machine::state_machine_instance`), so private-name resolution, inherent method paths, sibling access, and demangled symbols remain unchanged. A `mod build;`/`#[path = ...] mod build;` split would introduce new privacy boundaries and make a supposedly mechanical PR into an API refactor.

---

## 1. Large-file split plan: `artboard.rs`

### Current structural map

| Lines | Region |
| ---: | --- |
| 1-106 | Imports and supporting aliases |
| 107-135 | Draw-frame counter and generated `Mat2D` helper |
| 136-160 | `ExternalFontAssetError` |
| 161-181 | `RuntimeArtboardInstanceIdentity` |
| 182-231 | `RuntimeScriptState` and implementations |
| 232-250 | Advancing-component and persistent-dirt fixtures |
| 251-298 | Script advance/update mode enums and implementations |
| 299-327 | Resetting/runtime-component helper types |
| 328-334 | `RuntimeTextStyleFeatureOption` |
| 335-523 | `ArtboardInstance` storage |
| 524-714 | `Clone for ArtboardInstance` |
| 715-739 | `Drop for ArtboardInstance` followed by the existing leaf `include!` declarations |
| 740-773 | Occurrence and frame-advance report types |
| 774-844 | Build context/index/text-flag helpers |
| 845-860 | `RuntimeNestedAnimationInstance` |
| 861-11,133 | Main `impl ArtboardInstance` |
| 11,134-11,147 | Focus-scroll path and focus-bounds transform helper |
| 11,148-11,187 | Small follow-up `impl ArtboardInstance` |
| 11,188-11,226 | Component-list reset helper |
| 11,227-11,595 | `impl RuntimeNestedArtboardInstance` |
| 11,596-11,611 | `impl RuntimeNestedAnimationInstance` |
| 11,612-12,039 | Free construction/nested-artboard helpers |
| 12,040-23,374 | Single `#[cfg(test)] mod tests` (`205` tests) |

The main implementation itself divides along stable responsibility boundaries:

| Lines | Logical responsibility |
| ---: | --- |
| 861-1,104 | Lifecycle, data context, and transient-clone restoration |
| 1,105-2,440 | Build occurrence relationships and constructors |
| 2,441-2,712 | Path/hit initialization and external assets |
| 2,713-3,399 | Scripting lifecycle |
| 3,400-3,989 | Object/component/audio/hit/graph/definition access |
| 3,990-4,152 | Scripted interpolators and state-machine definition selection |
| 4,153-4,480 | Dimensions, world/layout bounds, scrolling, and focus reveal |
| 4,481-5,229 | Generated property façade and text values |
| 5,230-5,568 | Animation/state-machine/view-model occurrences |
| 5,569-6,312 | Component-list lifecycle and layout |
| 6,313-7,408 | Root/nested frame advance and retained-component dispatch |
| 7,409-7,585 | Nested-layout frame propagation |
| 7,586-8,496 | Transforms, epochs, dirt, and invalidation |
| 8,497-9,191 | Update pass and five-pass state-machine settlement |
| 9,192-9,825 | Component update dispatch and script scheduling |
| 9,826-11,133 | Property-change callbacks, nested controls, and collapse |

### Proposed physical layout

Keep `artboard.rs` as the owner. It retains imports, supporting types, `ArtboardInstance`, `Clone`, `Drop`, the existing leaf includes, occurrence/report/build helper types, and an ordered series of literal includes. This preserves declaration order where it matters and makes all fragments part of `crate::artboard`.

```text
src/
  artboard.rs                         owner/types, existing leaf includes, ordered include! list
  artboard/
    build.rs                          current 861-2712
    script_runtime.rs                 current 2713-3399
    access.rs                         current 3400-3989
    definitions_and_layout.rs         current 3990-4480
    properties.rs                     current 4481-5568
    component_lists.rs                current 5569-6312
    advance.rs                        current 6313-7585
    dirt.rs                           current 7586-8496
    update.rs                         current 8497-9825
    callbacks.rs                      current 9826-11133
    nested.rs                         current 11134-12039
    tests.rs                          body of current tests module
```

The production fragments are intentionally in execution/dependency order rather than alphabetical order. `tests.rs` should be included inside the original test module:

```rust
#[cfg(test)]
mod tests {
    include!("artboard/tests.rs");
}
```

This retains the test module's name and access. It also gives the attribution checker a deliberately exempt `tests.rs` leaf.

### Lockstep correspondence and checker manifest

The following table is the required per-file edit manifest. “B6 rows” are `[[files]]` entries in `file-correspondence-manifest.toml`; add the fragment's path to those rows' semicolon-separated `rust_module` values and update their C17 retained-module notes. Keep status, upstream pins, C++ paths, verification text, and correspondence claims unchanged. The row list is conservative: a later provenance cleanup can narrow it, but narrowing must not be mixed into this mechanical split.

| New file | B6 rows that must name it | Frame-loop/checker bindings that must move with it |
| --- | --- | --- |
| `artboard/build.rs` | `B6-0001`, `B6-0094`, `B6-0115`, `B6-0117`, `B6-0123`, `B6-0125`, `B6-0129`, `B6-0131`, `B6-0142`, `B6-0146`, `B6-0202`, `B6-0258`, `B6-0303`, `B6-0304`, `B6-0305`, `B6-0331`, `B6-0361`, `B6-0362`, `B6-0365`, `B6-0368`, `B6-0385`, `B6-0388`, `B6-0399`, `B6-0405`, `B6-0408` | Repoint `summarize_trace.py` dependency-build and IK-chain-build source scans when their anchors land here. Repoint matching `runtime-frame-loop-gaps.toml` owner-boundary entries. |
| `artboard/script_runtime.rs` | `B6-0077`, `B6-0194`, `B6-0200`, `B6-0322`, `B6-0326` | Repoint the script-lifecycle citations in `runtime-frame-loop-ownership.toml` and any corresponding synthetic test fixture paths. |
| `artboard/access.rs` | `B6-0094`, `B6-0095`, `B6-0123`, `B6-0146`, `B6-0203`, `B6-0204`, `B6-0205` | Repoint any live-draw/renderer-interface lifecycle citations whose named methods move here. |
| `artboard/definitions_and_layout.rs` | `B6-0077`, `B6-0094`, `B6-0258`, `B6-0303`, `B6-0304`, `B6-0305`, `B6-0385`, `B6-0388` | Repoint focus-retained-tree and layout lifecycle citations, plus matching owner-boundary allow entries. |
| `artboard/properties.rs` | `B6-0067`, `B6-0077`, `B6-0094`, `B6-0194`, `B6-0200`, `B6-0352`, `B6-0355`, `B6-0385`, `B6-0388`, `B6-0399`, `B6-0404`, `B6-0405`, `B6-0408` | `B6-0067` currently has only one Rust module: **replace** `artboard.rs` with this fragment rather than adding a second path. Repoint property/text lifecycle citations and corresponding owner-boundary entries. |
| `artboard/component_lists.rs` | `B6-0095`, `B6-0258`, `B6-0303`, `B6-0304`, `B6-0305` | Repoint component-list and nested-layout lifecycle citations and matching test fixtures. |
| `artboard/advance.rs` | `B6-0001`, `B6-0059`, `B6-0061`, `B6-0063`, `B6-0064`, `B6-0077`, `B6-0094`, `B6-0095`, `B6-0194`, `B6-0200`, `B6-0258`, `B6-0303`, `B6-0304`, `B6-0305`, `B6-0322`, `B6-0326`, `B6-0388` | In `runtime-frame-loop-ownership.toml`, move `artboard.advance`'s `rust_file` and the lifecycle citations for artboard/nested advance. Repoint `summarize_trace.py` advancing/resetting dispatch scans as applicable. Repoint relevant gap allow rows and `test_check.py` advance fixtures. |
| `artboard/dirt.rs` | `B6-0094`, `B6-0123`, `B6-0258`, `B6-0331`, `B6-0352`, `B6-0355`, `B6-0357`, `B6-0361`, `B6-0362`, `B6-0365`, `B6-0368`, `B6-0385`, `B6-0388`, `B6-0399`, `B6-0405`, `B6-0408` | Move `component.dirt`, `component.dependents` (`pub fn add_dirt(`), and transform/dirt lifecycle citations as appropriate. Repoint `summarize_trace.py`'s two add-dirt anchors, dirt consumptions, owner resolutions, and skin-buffer scans according to their final physical locations. Repoint matching gap rows. |
| `artboard/update.rs` | `B6-0001`, `B6-0094`, `B6-0115`, `B6-0117`, `B6-0123`, `B6-0125`, `B6-0129`, `B6-0131`, `B6-0142`, `B6-0194`, `B6-0200`, `B6-0203`, `B6-0204`, `B6-0205`, `B6-0258`, `B6-0303`, `B6-0304`, `B6-0305`, `B6-0322`, `B6-0326`, `B6-0331`, `B6-0352`, `B6-0355`, `B6-0357`, `B6-0361`, `B6-0362`, `B6-0365`, `B6-0368`, `B6-0385`, `B6-0388`, `B6-0399`, `B6-0404`, `B6-0405`, `B6-0408` | Move `component.update_order` (`fn update_components_with_hook_recording`) and `artboard.update_pass` (`pub fn update_pass(`) `rust_file` values and lifecycle citations. Repoint retained update/settlement gap rows, trace scans, and test fixtures. |
| `artboard/callbacks.rs` | `B6-0059`, `B6-0061`, `B6-0063`, `B6-0064`, `B6-0077`, `B6-0094`, `B6-0123`, `B6-0125`, `B6-0129`, `B6-0131`, `B6-0142`, `B6-0194`, `B6-0200`, `B6-0258`, `B6-0303`, `B6-0304`, `B6-0305`, `B6-0331`, `B6-0352`, `B6-0355`, `B6-0357`, `B6-0361`, `B6-0362`, `B6-0365`, `B6-0368`, `B6-0385`, `B6-0388`, `B6-0399`, `B6-0404`, `B6-0405`, `B6-0408` | Change `check_layout_style_handlers.py`'s `RUST_ARTBOARD_MODULE` to this fragment if both display-change markers remain here; otherwise make the checker accept the two explicit physical files. Repoint callback lifecycle citations, gap rows, and fixtures. |
| `artboard/nested.rs` | `B6-0059`, `B6-0061`, `B6-0063`, `B6-0064`, `B6-0077`, `B6-0094`, `B6-0095`, `B6-0258`, `B6-0303`, `B6-0304`, `B6-0305` | Repoint nested-artboard advance/layout citations, gap allow entries, and nested fixtures. |
| `artboard/tests.rs` | None. `rust_attribution.py` excludes a source whose stem is `tests`. | Move the artboard test body unchanged. Update the seven relative `include_bytes!`/fixture paths currently near old lines 19,951, 20,001, 20,168, 20,542, 20,597, 20,612, and 23,311 because the file becomes one directory deeper. Prefer `concat!(env!("CARGO_MANIFEST_DIR"), "/...")` for fixture stability. The `env!` use near old line 14,290 remains valid. |

#### Global artboard bindings

These bindings span more than one fragment and therefore belong in the same PR:

- `rust_attribution.py` discovers all production `.rs` files below `crates/nuxie-runtime/src`. Every new production fragment must be listed in at least one `file-correspondence-manifest.toml` row and must **not** be added to `rust-additions.toml`. The checker exempts `tests.rs`, names ending in `_tests`, and files below a `tests/` path.
- The manifest's scatter ratchet is already at its maximum: 155 rows currently have multiple Rust modules. Except for `B6-0067`, all affected artboard rows are already multi-module. Replacing the `B6-0067` root path prevents the split from increasing that count.
- Update every affected C17 note that enumerates retained Rust modules. Do not change the underlying parity classification merely because the code's physical address changed.
- `docs/runtime-frame-loop-ownership.toml` has artboard citations in `focus.retained_tree`, `component.identity`, `component.dirt`, `component.dependents`, `component.update_order`, `component.transforms`, `component.clone_drop`, `artboard.advance`, `artboard.update_pass`, `nested_artboard.advance`, `artboard.live_draw`, and `renderer.interface_boundary`. Keep `component.clone_drop` and its `pub struct ArtboardInstance` anchor on the root; change each other `rust_file` or `rust:path:line-range` to its physical fragment.
- `docs/runtime-frame-loop-gaps.toml` has 25 owner-boundary allow entries pointing at `artboard.rs` (entries 0-7, 11-22, and 29-34 in current order). Repoint each to the fragment that contains its anchor. A literal move should preserve its token-based `site_hash`; a changed hash is a signal that the split was not mechanical.
- `tools/runtime-frame-loop-port/summarize_trace.py` hard-codes `artboard.rs` nine times for add-dirt, dirt consumption, owner resolution, dependency build, skin rebuild, advancing dispatch, resetting dispatch, and IK-chain discovery. Give each query its actual fragment path. The demangled owner should remain `artboard::ArtboardInstance` because `include!` does not introduce a child module.
- `tools/runtime-frame-loop-port/check_layout_style_handlers.py` hard-codes the artboard source while looking for `propagate_layout_component_display_changed` and the display property key. Point it to the fragment(s) containing those markers.
- `tools/runtime-frame-loop-port/test_check.py` contains at least 14 exact artboard paths covering probe gating, layer occurrence, live-advance ratchets, registry site hashes/substitution, and trait-anchor behavior. Update each synthetic fixture to the fragment that owns the tested anchor; do not perform a blind string replacement.
- Leave the existing root-level includes (`nested_artboard.rs`, `artboard_component_list.rs`, `artboard_list_map_rule.rs`, `artboard_referencer.rs`, `bindable_artboard.rs`, `nested_artboard_layout.rs`, `nested_artboard_leaf.rs`, `component_origin.rs`, bone/weight, profiler/profile) where they are. Moving them under `artboard/` changes relative include resolution and correspondence ownership.
- Keep `crates/nuxie-runtime/src/lib.rs`'s `mod artboard;` unchanged. The public and crate-private paths must not acquire an extra `artboard::<fragment>` segment.

### Mechanical PR sequence and proof

One PR should contain only this file's split and the path/anchor bookkeeping above. Recommended commit sequence within that PR:

1. Extract `tests.rs`, fix only its fixture paths, and prove the same 205 tests are collected.
2. Extract production regions in original order using `include!`; make no renames, visibility edits, formatting sweeps, or logic edits.
3. Update correspondence C17/path fields and all frame-loop physical anchors.
4. Update checker source locations and synthetic fixtures, including an assertion that demangled ownership remains `artboard::ArtboardInstance`.

Proof battery:

```text
make rust-sources-fresh
make cpp-probe
make runtime-frame-loop-port-test
make runtime-frame-loop-port-check
make rust-attribution-check
cargo check --workspace
cargo test -p nuxie-runtime
cargo test -p nuxie --features scripting
make scripted-golden-compare
make silver-corpus-test
```

The PR is structure-preserving only if parity row counts/statuses, trace landmarks, owner-boundary hashes, test counts, and golden/silver outputs are unchanged.

---

## 2. Large-file split plan: `state_machine_instance.rs`

### Current structural map

| Lines | Region |
| ---: | --- |
| 1-93 | Imports and ownership commentary |
| 94-255 | Nested event-chain/notify phases, trace structures, and trace recording |
| 256-312 | Semantic-node resolver and audio/event seam types |
| 313-1,271 | `RuntimeHitResult`, `HitComponent`, and hierarchy implementations |
| 1,272-1,467 | Nested notifier, focus, semantic, and state helper types |
| 1,468-1,474 | `RuntimeDataContextBindError` |
| 1,475-1,711 | `StateMachineInstance` storage |
| 1,712-1,744 | Data-bind occurrence/pointer/executor structures |
| 1,745-2,147 | Listener-action executor implementations |
| 2,148-2,335 | `Clone for StateMachineInstance` |
| 2,336-2,356 | `Drop for StateMachineInstance` |
| 2,357-2,638 | Free helpers and view-model listener types |
| 2,639-13,938 | Main `impl StateMachineInstance` |
| 13,939-13,953 | `runtime_owned_font` helper |
| 13,954-14,079 | `view_model_listener_tests` (`1` test) |
| 14,080-21,674 | `scripted_listener_action_tests` (`93` tests) |

The main implementation divides as follows:

| Lines | Logical responsibility |
| ---: | --- |
| 2,639-2,932 | Teardown, enrollment, focus manager, clone rebind |
| 2,933-3,741 | Construction and listener/layer initialization |
| 3,742-5,102 | Scripting, hydration, and adoption |
| 5,103-6,007 | State/input/focus/semantic/keyboard/gamepad operations |
| 6,008-7,487 | Hit/listener/pointer pipeline |
| 7,488-8,525 | Event notification, deferred callbacks, listener actions |
| 8,526-9,100 | Direct bind targets and value readers |
| 9,101-10,748 | Default/imported/owned source setters and relinking |
| 10,749-12,599 | Data-context bind/rebind, listener cells, bind updates |
| 12,600-13,938 | Reports, advance/apply, nested event dispatch, settlement |

### Proposed physical layout

```text
src/state_machine/
  state_machine_instance.rs               owner/types, hit hierarchy, lifecycle, ordered include! list
  state_machine_instance/
    construction.rs                       current 2639-3741
    scripted_objects.rs                   current 3742-5102
    input_focus.rs                        current 5103-6007
    pointer.rs                            current 6008-7487
    events_actions.rs                     current 7488-8525
    bind_targets.rs                       current 8526-9100
    bind_sources.rs                       current 9101-10748
    data_context.rs                       current 10749-12599
    advance.rs                            current 12600-13953
    tests/
      view_model_listener.rs              body of current first test module
      scripted_listener_actions.rs         body of current second test module
```

Retain the two existing module names in the root and include only their bodies. This preserves test paths and any name-sensitive filtering:

```rust
#[cfg(test)]
mod view_model_listener_tests {
    include!("state_machine_instance/tests/view_model_listener.rs");
}
```

Use the same pattern for `scripted_listener_action_tests`.

### Lockstep correspondence and checker manifest

| New file | B6 rows that must name it | Frame-loop/checker bindings that must move with it |
| --- | --- | --- |
| `state_machine_instance/construction.rs` | `B6-0077`, `B6-0194`, `B6-0195`, `B6-0196`, `B6-0200`, `B6-0310` | Repoint construction/enrollment lifecycle citations and matching synthetic fixtures. Keep `state_machine.collections`' `pub struct StateMachineInstance` anchor on the root. |
| `state_machine_instance/scripted_objects.rs` | `B6-0077`, `B6-0175`, `B6-0194`, `B6-0195`, `B6-0196`, `B6-0200` | Repoint scripting/hydration lifecycle citations and scripted-action fixture paths. |
| `state_machine_instance/input_focus.rs` | `B6-0077`, `B6-0083` | Repoint focus/semantic/keyboard/gamepad citations and tests. |
| `state_machine_instance/pointer.rs` | `B6-0077`, `B6-0083` | Repoint hit/pointer/listener lifecycle citations and FL-C4/layer/hit fixtures. |
| `state_machine_instance/events_actions.rs` | `B6-0058`, `B6-0077`, `B6-0440` | Repoint event reporting, firing/bubbling, and action-dispatch citations/fixtures. `state_machine.actions`' primary anchor remains on the root if the listener-action executor stays there. |
| `state_machine_instance/bind_targets.rs` | `B6-0058`, `B6-0194`, `B6-0196`, `B6-0200`, `B6-0440` | Repoint bind-target and view-model listener citations/fixtures. |
| `state_machine_instance/bind_sources.rs` | `B6-0058`, `B6-0175`, `B6-0194`, `B6-0195`, `B6-0196`, `B6-0200`, `B6-0440` | Repoint source/relink/data-converter citations and bind fixtures. |
| `state_machine_instance/data_context.rs` | `B6-0058`, `B6-0077`, `B6-0194`, `B6-0195`, `B6-0196`, `B6-0200`, `B6-0440` | Repoint data-context lifecycle citations and listener-cell/bind fixtures. |
| `state_machine_instance/advance.rs` | `B6-0077`, `B6-0194`, `B6-0195`, `B6-0196`, `B6-0200`, `B6-0310`, `B6-0440` | Move `state_machine.advance`'s `rust_file` and `advance_with_report_mode` anchor here. Repoint state-machine event/action/reporting lifecycle citations and advance/keyframe fixtures. Include `runtime_owned_font` in this fragment to avoid an artificial one-function file. |
| `state_machine_instance/tests/view_model_listener.rs` | None; it resides below a `tests/` path. | Move the body unchanged and preserve the outer module name. |
| `state_machine_instance/tests/scripted_listener_actions.rs` | None; it resides below a `tests/` path. | Move the body unchanged. Replace or correct the relative fixture path near old line 19,011 (`../../../../fixtures/sync/vm_listener_fire_event.riv`) because the source becomes two directories deeper; prefer a `CARGO_MANIFEST_DIR`-anchored path. |

#### Global state-machine bindings

- All production fragments must be named by `file-correspondence-manifest.toml`, never `rust-additions.toml`. Every affected SMI row is already multi-module, so this split does not increase the manifest's 155-row scatter count.
- Update C17 retained-module notes for `B6-0058`, `B6-0077`, `B6-0083`, `B6-0175`, `B6-0194`, `B6-0195`, `B6-0196`, `B6-0200`, `B6-0310`, and `B6-0440`; leave the substantive parity assertions untouched.
- `docs/runtime-frame-loop-ownership.toml` cites this file from `focus.retained_tree`, `state_machine.collections`, `state_machine.advance`, `state_machine.events`, `state_machine.actions`, and `event.reporting`. Keep root anchors for the struct, stored event collection, and action executor. Repoint method/lifecycle citations to physical fragments.
- `tools/runtime-frame-loop-port/check.py` currently treats one hard-coded SMI file as the nested-event owner and excludes it from outside-owner scans. Once methods are in included fragments, a path-only exclusion would falsely flag the fragments. Make owner discovery include-aware: recursively expand literal local `include!("...")` files in source order for export analysis, and treat every expanded file as the same logical owner. Do not broaden the exclusion to a directory glob.
- Add checker unit coverage for the include-aware logical owner: an exported nested-event method in an included fragment must be accepted, while the same method in an unrelated file must still fail. Preserve syntax parsing and duplicate-anchor detection.
- `tools/runtime-frame-loop-port/test_check.py` has at least 14 exact SMI references, plus constructed/split path strings, covering FL-C4, lifecycle, layer, hit, bind, events, firing/bubbling, advance, owner exports, keyframes, focus, and semantic behavior. Point each fixture to the actual fragment; do not simply substitute the root path everywhere.
- `docs/runtime-frame-loop-gaps.toml` has no current SMI owner-boundary allow row to migrate.
- Keep `crates/nuxie-runtime/src/state_machine.rs`'s `pub(crate) mod state_machine_instance;` and its re-exports unchanged. Literal includes keep all symbols at the original module path.

### Mechanical PR sequence and proof

This should be a second, independent PR after the artboard split has landed. It uses the same proof battery. The checker must first learn the logical-owner/include relationship in the same PR, because moving the methods without that update creates false ownership failures. Apart from that narrowly required path-awareness, no checker policy should change.

Success means the same 94 tests are collected, no exported API path changes, the correspondence scatter count stays at 155, no new owner-boundary allowance appears, and the full battery shown in the artboard plan remains green.

---

## 3. Nuxie-only seam inventory

### Classification method

The authoritative starting point is correspondence metadata:

- Production Rust paths mapped to upstream C++ in `file-correspondence-manifest.toml` are port-side unless a smaller product-only region can be proven.
- Production Rust paths listed in `rust-additions.toml` are intentional Rust-only additions. The relevant ownership labels are `flowsession-abi`, `scene-api`, and generated/codegen infrastructure.
- `build.rs` is outside the attribution scanner's `src/` roots, so it must be inspected manually.
- A convenience façade is not automatically product-side. Thin Rust-only namespace/generated-storage coordinators that serve the port stay with the port even though no one C++ file corresponds to them.

This prevents two opposite mistakes: burying product authoring features inside the port forever, and extracting Rust implementation infrastructure merely because its file has no one-to-one C++ source.

### Product features and mixed glue

| Region | Current size and location | Runtime internals reached / extraction surface | Placement assessment |
| --- | --- | --- | --- |
| FlowSession wire/domain model | `crates/nuxie/src/flow_session.rs:21-621` (601 lines) | Public `File`, `Factory`, renderer, artboard, view-model, and state-machine types | Clean product vocabulary; move to `nuxie-flow` after dependency direction is fixed. |
| FlowSession orchestration | `flow_session.rs:622-2163` (1,542) | Calls crate-private script preparation, listener preparation, rehydration, apply/rollback, host-cycle, and command-drain methods on the `nuxie` façade | Product-side, but requires a narrow public/sealed flow-runtime bridge before crate extraction. Moderate coupling. |
| FlowSession value snapshots | `flow_session.rs:2164-3105` (942) | Traverses public runtime-owned view-model/value objects and builds its own graph/arena | Product-side and cohesive. Mostly mechanical after bridge creation. |
| FlowSession validation/accounting | `flow_session.rs:3106-3545` (440) | Product protocol limits and payload validation | Clean extraction candidate. |
| FlowSession mutation/apply/diff | `flow_session.rs:3546-4515` (970) | Applies mutations through runtime view-model/state-machine handles; shares rollback/command sequencing with façade-private methods | Product-side; transactional sequencing makes it medium risk. |
| FlowSession selection/catalog | `flow_session.rs:4516-5020` (505) | File/artboard/state-machine discovery | Product-side; mostly public runtime surface. |
| FlowSession tests | `flow_session.rs:5021-7227` (2,207; 40 tests), plus `crates/nuxie/tests/flow_session_contract.rs` (335) | Exercises the public ABI, VM lifecycle, and transactional behavior | Move with FlowSession. Preserve public names through compatibility re-exports during migration. |
| Scene contract and IDs | `crates/nuxie/src/scene.rs:28-1187` (1,160) | Public runtime/binary identities and errors | Product-side, cleanly demarcatable. |
| Scene durable definitions/index/specs | `scene.rs:1188-3401` (2,214) | `nuxie_binary::AuthoringRecord`, property/value types, runtime file identities | Product-side authoring model. |
| Scene hierarchy/materialization | `scene.rs:3402-5622` (2,221) | Constructs/reads `RuntimeFile`; directly touches `materialized.file.runtime` in a handful of places even though `File::runtime()` already exists | Product-side but high coupling. Replace direct private field access with the existing public accessor before moving. |
| Silver/scene export schema | `scene.rs:5623-6819` (1,197) | Public exported-record/document DTOs; lowering later feeds `RuntimeFile::from_authoring_records` | Product-side. The schema and its generated counterpart must move together. |
| Scene store/live mount API | `scene.rs:6820-9099` (2,280) | Owns mounted `File`/artboard/view-model state and coordinates transactions | Product-side; medium-to-high coupling. |
| SceneTx and subordinate transactions | `scene.rs:9100-15589` (6,490) | `SceneTx`, `DataConverterTx`, `VmTx`, `AnimTx`, and `MachineTx` mutate the authoring graph, then materialize into runtime/binary types | Core product authoring seam. Cohesive, but atomic commit/materialization behavior makes the move high risk. |
| Scene frame queries/live mutation | `scene.rs:15590-17592` (2,003) | Hit/geometry/semantic queries and intrinsic image sizing reach crate-private `OwnedArtboardInstance` helpers | Product-side. Needs stable snapshot/command DTOs instead of exposing mutable runtime internals. |
| Scene lowering/validation/export | `scene.rs:17593-25563` (7,971) | Converts authoring records into `RuntimeFile`, validates cross-object invariants, and produces exported documents | Product-side. Highest-risk region because it couples the authoring model to port construction invariants. |
| Scene tests | `scene.rs:25564-38224` (12,661; 120 tests), plus `crates/nuxie/tests/scene_authoring.rs` (15,674) and related authoring tests | Broad transaction, materialization, export, and live-runtime contract | Move with Scene; use this suite as the seam migration oracle. |
| Scene schema generator | `crates/nuxie/build.rs` (4,531) and `scene.rs`'s `include!(concat!(env!("OUT_DIR"), "/scene_schema.rs"))` near line 860 | Generates the Scene/silver record schema at build time | Product-side and inseparable from Scene. Extraction must move the build script/generator inputs and preserve the generated include contract. |
| Script import authorization | `crates/nuxie/src/script_import.rs:1-157` production, `158-211` tests | Ed25519/container verification; calls crate-private `File::execution_authorization_for`/authorization state | Product policy. Move to `nuxie-flow`, but expose a capability-based import seam rather than making raw authorization state public. |
| Mixed façade bridge | `crates/nuxie/src/lib.rs`, principally project-envelope classification and private bridge methods around lines 55, 154, 223-258, 307, 344, 376, 1,492, 2,983, 3,328, 4,304, and 5,600-5,733 | `File::from_runtime`; flow script/listener lifecycle; apply/rollback/command drain; artboard geometry/semantic/intrinsic-image helpers | Not an independent feature. Convert these call sites into explicit narrow bridge traits/DTOs, then move their product consumers. |

The concrete crate-private methods the extraction must account for are:

- Flow lifecycle around `nuxie/src/lib.rs:5600-5675`: `prepare_flow_scripts`, `prepare_flow_listener_actions`, rehydrate/apply, begin host cycle, rollback, and drain commands.
- Scene live/runtime queries around `lib.rs:5705-5733`: hit-test path segments with bounds, geometry path segments, retained/semantic text, and intrinsic image-dimension registration.
- `File::from_runtime` around `lib.rs:4304` and the script execution authorization capability.

These are already cross-crate calls from `nuxie` into public `nuxie-runtime` types; the missing visibility is chiefly inside the `nuxie` façade. Extraction should expose a sealed product bridge from the future base façade, not expand dozens of runtime fields to `pub`.

### Project-data converter: product model inside the port

`crates/nuxie-runtime/src/project_data_converter.rs` is 2,686 lines and is explicitly classified `scene-api` in `rust-additions.toml`:

| Lines | Region | Coupling and disposition |
| ---: | --- | --- |
| 22-507 | Public values, specs, errors, and state | Pure product/protocol model; eventual neutral-crate candidate. |
| 508-1,289 | Envelope/program/catalog compilation and evaluation | Mostly `std`/`serde`; eventual neutral-crate candidate. |
| 1,290-2,324 | Validation, coercion, formatting, interpolation | Pure-looking core, but behavior is part of runtime data-bind semantics. Extract only with parity tests. |
| 2,325-2,686 | Formula parser | Mechanically separable after the outer seam is established. |

The file is locally self-contained, but its runtime integration is not. `data_bind/context/context_value.rs` has a `RuntimeDataBindGraphConverter::Project` variant and owns conversion/evaluator state; `data_bind/data_bind_context.rs` constructs it from a script-asset envelope; both runtime and `nuxie` re-export or classify its types; Scene authors and lowers them. A direct move to `nuxie-flow` would invert the desired dependency and make the port depend on the product crate.

Two crate-private numeric-policy helpers in this file (the bounded-list constant near line 467 and helper near line 2,320) are also used by port-side Number-to-List/context conversion. If the model is eventually extracted, those policies should remain in the port or move to a dependency-neutral shared crate; they should not become a product API solely to satisfy the move.

Recommendation: first demarcate this as `project_data_converter/{model, program, evaluator, bridge}` inside `nuxie-runtime`; later extract only the pure model/program/evaluator to a neutral `nuxie-project-data` crate that both runtime and `nuxie-flow` may depend on. The runtime adapter stays in `nuxie-runtime`.

### Rust-only infrastructure that should remain port-side

The following `rust-additions.toml` entries have no one-file C++ correspondence but are not product features:

| Current path/group | Lines | Why it stays in `nuxie-runtime` |
| --- | ---: | --- |
| `data_bind_container.rs`, `data_converter.rs`, `retained_data_bind.rs` | 12 total | Tiny generated/type façade modules over port functionality. |
| `input/mod.rs`, `shapes/mod.rs`, `shapes/paint/mod.rs` | 168 total | Rust namespace/generated-storage coordinators. |
| `objects.rs` | 1,018 | Runtime object registry plus `include!(concat!(env!("OUT_DIR"), "/runtime_objects.rs"))`; generated by `crates/nuxie-runtime/build.rs` (707 lines). This is port construction infrastructure. |
| `properties.rs` | 826 | Generated property interpretation/storage used throughout the port. |
| `state_machine.rs`, `state_machine/instance.rs`, `state_machine/listener_types/mod.rs` | 215 total | Rust module/re-export façade for the ported state-machine implementation. |
| `viewmodel/mod.rs`, `viewmodel/runtime/mod.rs` | 58 total | Rust namespace/re-export façade for ported view-model behavior. |
| `focus.rs` | 7 | Thin public façade over retained focus behavior, not a standalone product feature. |

These codegen/facade additions total about 2,303 source lines excluding the project converter and focus façade. They should be clearly marked as Rust port infrastructure, but moving them to `nuxie-flow` would blur rather than improve the seam.

`crates/nuxie/src/command_queue.rs`, server code, and raw-text support are not included in the Nuxie-only list: their current files are explicitly mapped to upstream C++ correspondence. A richer Rust API around a ported facility does not by itself make that facility product-side.

---

## 4. Recommended seam design

### Decision

Use a **module boundary now and a crate boundary later**.

The immediate structure should make product ownership visible without changing dependencies:

```text
crates/nuxie/src/product/
  flow_session/
  scene/
    model/
    transaction/
    live/
    export/
  script_import/

crates/nuxie-runtime/src/project_data_converter/
  model.rs
  program.rs
  evaluator.rs
  bridge.rs
```

Preserve current public paths with root re-exports. This first step is a namespace/demarcation change, not an extraction. It gives reviewers an auditable boundary while the current test and parity baseline still observes exactly the same crate graph.

The long-term crate graph should avoid a cycle:

```text
nuxie-runtime  <---  nuxie-core  <---  nuxie-flow
       ^                 ^                 ^
       |                 |                 |
       +------ nuxie-project-data --------+

nuxie (umbrella compatibility crate) re-exports nuxie-core + nuxie-flow
```

`nuxie-flow` contains FlowSession, Scene/SceneTx, silver/export DTOs and lowering, and script-import policy. `nuxie-core` contains `File`, `Factory`, owned artboard/view-model handles, renderer integration, and narrow bridge traits. `nuxie-runtime` remains the C++-correspondence port plus explicitly labeled Rust port infrastructure. A small dependency-neutral `nuxie-project-data` is optional and should be introduced only after the runtime adapter has been separated from the pure evaluator.

### Public surface of the port/base layer

The extraction should not make arbitrary runtime fields public. The required surface is narrow:

1. A sealed/trusted constructor that wraps a validated `RuntimeFile` as a façade `File`, with an explicit authoring/import authorization policy.
2. A flow-runtime bridge for prepare, rehydrate, apply, host-cycle, rollback, and command-drain operations. Its inputs/outputs should be stable value DTOs, not references to internal queues.
3. Read-only artboard snapshot operations for hit geometry, retained geometry, and semantic text, plus a command for intrinsic image dimensions. Return owned snapshots so Scene cannot retain internal graph borrows.
4. A project-converter adapter owned by `nuxie-runtime`; pure program/model types may come from the neutral crate. Do not expose the runtime bind graph or evaluator cells as product API.
5. Existing upstream-corresponding runtime types only where they are already public and semantically stable. Product re-exports should live in `nuxie`/`nuxie-flow`, not expand the port's promise.

### Migration order

1. Establish the exact green parity/golden baseline and keep it pinned in every PR.
2. Land the `artboard.rs` and `state_machine_instance.rs` mechanical splits as separate PRs. This makes later ownership changes reviewable without mixing 20,000-line source moves into semantic work.
3. Demarcate `nuxie::product::{flow_session, scene, script_import}` and the project-converter submodules in place, preserving public re-exports.
4. Define the sealed flow, authoring-file, geometry-snapshot, and script-authorization bridges; migrate existing internal callers to them while everything remains in the same crate.
5. Extract the non-product façade/runtime handles into `nuxie-core`, with `nuxie` temporarily acting as an umbrella compatibility crate.
6. Move FlowSession and script import to `nuxie-flow`; move their tests and retain compatibility re-exports.
7. Move Scene model, SceneTx, export schema/generator, lowering, live adapter, and tests together. Do not split the generated schema from `build.rs` or split transaction commit from materialization in the extraction PR.
8. Isolate the project converter's pure model/program/evaluator. Extract it to a neutral crate only if doing so leaves the runtime adapter dependency pointing inward, never from `nuxie-runtime` to `nuxie-flow`.
9. Remove compatibility re-exports and tighten visibility in a later breaking-change PR, after downstream migration.

### Risk map

| Risk | Areas | Reason |
| --- | --- | --- |
| Mechanical/low | Thin namespace façades, codegen coordinators staying in place, test-module extraction, pure FlowSession validation/types | Mostly physical ownership with no runtime mutation semantics. |
| Moderate | FlowSession after bridge creation, script import after capability API, Scene DTO/export types | Public API and authorization compatibility, but limited graph mutation. |
| High | SceneTx commit/materialization, live Scene mounting, geometry/semantic access, Scene lowering and generated schema | Atomicity, identity, generated code, and live runtime invariants cross the seam. |
| High | Project-data converter extraction | The source looks pure, but evaluator state is embedded in runtime data-bind context and Number-to-List policy. |
| High | Script/listener ownership during FlowSession moves | Rehydration, rollback, command queues, and VM object lifetime must remain one coherent transaction. |

### Why baseline-first matters

`docs/parity-gap-register.md` explicitly recommends making the parity claim trustworthy before feature work and landing each subsequent change against the oracle (the “Recommended execution order” around lines 204-211). It also describes Scene as a Nuxie-only additive API (around lines 126-127) while recording FlowSession/capability-boundary gaps elsewhere in the register. That is the governing principle here: first make source moves mechanically observable by the existing correspondence, ownership, trace, golden, and silver batteries; then introduce the product seam behind those same oracles. A green build alone is insufficient because it can preserve compilation while silently weakening ownership scans or changing what “zero divergences” measures.

The two large-file splits are therefore prerequisites for, not part of, the product extraction. Each split keeps the logical Rust module unchanged and updates only physical-address consumers. The seam work then proceeds through explicit bridges and compatibility re-exports, with risky transaction/materialization changes isolated from mechanical moves.

# nuxie-runtime structure report

## Scope and baseline

This is a read-only structure analysis. It proposes no behavior, source, parity-ledger, or ownership-ledger changes.

The inspected worktree was already detached at `e8726db8ffc689d3b19f1c0a55794aa6daf9956d` (`Merge pull request #246 from TheOneLevin/feat/b6-0058-listener-viewmodel-change`, 2026-08-04). The local `origin/main` ref resolves to the same commit. The requested `git fetch origin` and redundant `git checkout --detach origin/main` could not update the shared worktree metadata because the sandbox cannot write the parent repository's `.git/worktrees/nuxie-mr-c12` directory. The analysis is therefore pinned to the locally available `origin/main` commit above, not a newly fetched remote ref. The pre-existing untracked `vtriage-capture/` directory was left untouched.

The two target files have grown beyond the sizes in the request:

| File | Current lines | Production/support | Tests |
| --- | ---: | ---: | ---: |
| `crates/nuxie-runtime/src/artboard.rs` | 23,374 | 12,039 | 11,335 (`205` tests) |
| `crates/nuxie-runtime/src/state_machine/state_machine_instance.rs` | 21,674 | 14,079 | 7,595 (`94` tests) |

The safest split mechanism is literal `include!` fragments, not Rust child modules. An included fragment remains in the original semantic module (`crate::artboard` or `crate::state_machine::state_machine_instance`), so private-name resolution, inherent method paths, sibling access, and demangled symbols remain unchanged. A `mod build;`/`#[path = ...] mod build;` split would introduce new privacy boundaries and make a supposedly mechanical PR into an API refactor.

---

## 1. Large-file split plan: `artboard.rs`

### Current structural map

| Lines | Region |
| ---: | --- |
| 1-106 | Imports and supporting aliases |
| 107-135 | Draw-frame counter and generated `Mat2D` helper |
| 136-160 | `ExternalFontAssetError` |
| 161-181 | `RuntimeArtboardInstanceIdentity` |
| 182-231 | `RuntimeScriptState` and implementations |
| 232-250 | Advancing-component and persistent-dirt fixtures |
| 251-298 | Script advance/update mode enums and implementations |
| 299-327 | Resetting/runtime-component helper types |
| 328-334 | `RuntimeTextStyleFeatureOption` |
| 335-523 | `ArtboardInstance` storage |
| 524-714 | `Clone for ArtboardInstance` |
| 715-739 | `Drop for ArtboardInstance` followed by the existing leaf `include!` declarations |
| 740-773 | Occurrence and frame-advance report types |
| 774-844 | Build context/index/text-flag helpers |
| 845-860 | `RuntimeNestedAnimationInstance` |
| 861-11,133 | Main `impl ArtboardInstance` |
| 11,134-11,147 | Focus-scroll path and focus-bounds transform helper |
| 11,148-11,187 | Small follow-up `impl ArtboardInstance` |
| 11,188-11,226 | Component-list reset helper |
| 11,227-11,595 | `impl RuntimeNestedArtboardInstance` |
| 11,596-11,611 | `impl RuntimeNestedAnimationInstance` |
| 11,612-12,039 | Free construction/nested-artboard helpers |
| 12,040-23,374 | Single `#[cfg(test)] mod tests` (`205` tests) |

The main implementation itself divides along stable responsibility boundaries:

| Lines | Logical responsibility |
| ---: | --- |
| 861-1,104 | Lifecycle, data context, and transient-clone restoration |
| 1,105-2,440 | Build occurrence relationships and constructors |
| 2,441-2,712 | Path/hit initialization and external assets |
| 2,713-3,399 | Scripting lifecycle |
| 3,400-3,989 | Object/component/audio/hit/graph/definition access |
| 3,990-4,152 | Scripted interpolators and state-machine definition selection |
| 4,153-4,480 | Dimensions, world/layout bounds, scrolling, and focus reveal |
| 4,481-5,229 | Generated property façade and text values |
| 5,230-5,568 | Animation/state-machine/view-model occurrences |
| 5,569-6,312 | Component-list lifecycle and layout |
| 6,313-7,408 | Root/nested frame advance and retained-component dispatch |
| 7,409-7,585 | Nested-layout frame propagation |
| 7,586-8,496 | Transforms, epochs, dirt, and invalidation |
| 8,497-9,191 | Update pass and five-pass state-machine settlement |
| 9,192-9,825 | Component update dispatch and script scheduling |
| 9,826-11,133 | Property-change callbacks, nested controls, and collapse |

### Proposed physical layout

Keep `artboard.rs` as the owner. It retains imports, supporting types, `ArtboardInstance`, `Clone`, `Drop`, the existing leaf includes, occurrence/report/build helper types, and an ordered series of literal includes. This preserves declaration order where it matters and makes all fragments part of `crate::artboard`.

```text
src/
  artboard.rs                         owner/types, existing leaf includes, ordered include! list
  artboard/
    build.rs                          current 861-2712
    script_runtime.rs                 current 2713-3399
    access.rs                         current 3400-3989
    definitions_and_layout.rs         current 3990-4480
    properties.rs                     current 4481-5568
    component_lists.rs                current 5569-6312
    advance.rs                        current 6313-7585
    dirt.rs                           current 7586-8496
    update.rs                         current 8497-9825
    callbacks.rs                      current 9826-11133
    nested.rs                         current 11134-12039
    tests.rs                          body of current tests module
```

The production fragments are intentionally in execution/dependency order rather than alphabetical order. `tests.rs` should be included inside the original test module:

```rust
#[cfg(test)]
mod tests {
    include!("artboard/tests.rs");
}
```

This retains the test module's name and access. It also gives the attribution checker a deliberately exempt `tests.rs` leaf.

### Lockstep correspondence and checker manifest

The following table is the required per-file edit manifest. “B6 rows” are `[[files]]` entries in `file-correspondence-manifest.toml`; add the fragment's path to those rows' semicolon-separated `rust_module` values and update their C17 retained-module notes. Keep status, upstream pins, C++ paths, verification text, and correspondence claims unchanged. The row list is conservative: a later provenance cleanup can narrow it, but narrowing must not be mixed into this mechanical split.

| New file | B6 rows that must name it | Frame-loop/checker bindings that must move with it |
| --- | --- | --- |
| `artboard/build.rs` | `B6-0001`, `B6-0094`, `B6-0115`, `B6-0117`, `B6-0123`, `B6-0125`, `B6-0129`, `B6-0131`, `B6-0142`, `B6-0146`, `B6-0202`, `B6-0258`, `B6-0303`, `B6-0304`, `B6-0305`, `B6-0331`, `B6-0361`, `B6-0362`, `B6-0365`, `B6-0368`, `B6-0385`, `B6-0388`, `B6-0399`, `B6-0405`, `B6-0408` | Repoint `summarize_trace.py` dependency-build and IK-chain-build source scans when their anchors land here. Repoint matching `runtime-frame-loop-gaps.toml` owner-boundary entries. |
| `artboard/script_runtime.rs` | `B6-0077`, `B6-0194`, `B6-0200`, `B6-0322`, `B6-0326` | Repoint the script-lifecycle citations in `runtime-frame-loop-ownership.toml` and any corresponding synthetic test fixture paths. |
| `artboard/access.rs` | `B6-0094`, `B6-0095`, `B6-0123`, `B6-0146`, `B6-0203`, `B6-0204`, `B6-0205` | Repoint any live-draw/renderer-interface lifecycle citations whose named methods move here. |
| `artboard/definitions_and_layout.rs` | `B6-0077`, `B6-0094`, `B6-0258`, `B6-0303`, `B6-0304`, `B6-0305`, `B6-0385`, `B6-0388` | Repoint focus-retained-tree and layout lifecycle citations, plus matching owner-boundary allow entries. |
| `artboard/properties.rs` | `B6-0067`, `B6-0077`, `B6-0094`, `B6-0194`, `B6-0200`, `B6-0352`, `B6-0355`, `B6-0385`, `B6-0388`, `B6-0399`, `B6-0404`, `B6-0405`, `B6-0408` | `B6-0067` currently has only one Rust module: **replace** `artboard.rs` with this fragment rather than adding a second path. Repoint property/text lifecycle citations and corresponding owner-boundary entries. |
| `artboard/component_lists.rs` | `B6-0095`, `B6-0258`, `B6-0303`, `B6-0304`, `B6-0305` | Repoint component-list and nested-layout lifecycle citations and matching test fixtures. |
| `artboard/advance.rs` | `B6-0001`, `B6-0059`, `B6-0061`, `B6-0063`, `B6-0064`, `B6-0077`, `B6-0094`, `B6-0095`, `B6-0194`, `B6-0200`, `B6-0258`, `B6-0303`, `B6-0304`, `B6-0305`, `B6-0322`, `B6-0326`, `B6-0388` | In `runtime-frame-loop-ownership.toml`, move `artboard.advance`'s `rust_file` and the lifecycle citations for artboard/nested advance. Repoint `summarize_trace.py` advancing/resetting dispatch scans as applicable. Repoint relevant gap allow rows and `test_check.py` advance fixtures. |
| `artboard/dirt.rs` | `B6-0094`, `B6-0123`, `B6-0258`, `B6-0331`, `B6-0352`, `B6-0355`, `B6-0357`, `B6-0361`, `B6-0362`, `B6-0365`, `B6-0368`, `B6-0385`, `B6-0388`, `B6-0399`, `B6-0405`, `B6-0408` | Move `component.dirt`, `component.dependents` (`pub fn add_dirt(`), and transform/dirt lifecycle citations as appropriate. Repoint `summarize_trace.py`'s two add-dirt anchors, dirt consumptions, owner resolutions, and skin-buffer scans according to their final physical locations. Repoint matching gap rows. |
| `artboard/update.rs` | `B6-0001`, `B6-0094`, `B6-0115`, `B6-0117`, `B6-0123`, `B6-0125`, `B6-0129`, `B6-0131`, `B6-0142`, `B6-0194`, `B6-0200`, `B6-0203`, `B6-0204`, `B6-0205`, `B6-0258`, `B6-0303`, `B6-0304`, `B6-0305`, `B6-0322`, `B6-0326`, `B6-0331`, `B6-0352`, `B6-0355`, `B6-0357`, `B6-0361`, `B6-0362`, `B6-0365`, `B6-0368`, `B6-0385`, `B6-0388`, `B6-0399`, `B6-0404`, `B6-0405`, `B6-0408` | Move `component.update_order` (`fn update_components_with_hook_recording`) and `artboard.update_pass` (`pub fn update_pass(`) `rust_file` values and lifecycle citations. Repoint retained update/settlement gap rows, trace scans, and test fixtures. |
| `artboard/callbacks.rs` | `B6-0059`, `B6-0061`, `B6-0063`, `B6-0064`, `B6-0077`, `B6-0094`, `B6-0123`, `B6-0125`, `B6-0129`, `B6-0131`, `B6-0142`, `B6-0194`, `B6-0200`, `B6-0258`, `B6-0303`, `B6-0304`, `B6-0305`, `B6-0331`, `B6-0352`, `B6-0355`, `B6-0357`, `B6-0361`, `B6-0362`, `B6-0365`, `B6-0368`, `B6-0385`, `B6-0388`, `B6-0399`, `B6-0404`, `B6-0405`, `B6-0408` | Change `check_layout_style_handlers.py`'s `RUST_ARTBOARD_MODULE` to this fragment if both display-change markers remain here; otherwise make the checker accept the two explicit physical files. Repoint callback lifecycle citations, gap rows, and fixtures. |
| `artboard/nested.rs` | `B6-0059`, `B6-0061`, `B6-0063`, `B6-0064`, `B6-0077`, `B6-0094`, `B6-0095`, `B6-0258`, `B6-0303`, `B6-0304`, `B6-0305` | Repoint nested-artboard advance/layout citations, gap allow entries, and nested fixtures. |
| `artboard/tests.rs` | None. `rust_attribution.py` excludes a source whose stem is `tests`. | Move the artboard test body unchanged. Update the seven relative `include_bytes!`/fixture paths currently near old lines 19,951, 20,001, 20,168, 20,542, 20,597, 20,612, and 23,311 because the file becomes one directory deeper. Prefer `concat!(env!("CARGO_MANIFEST_DIR"), "/...")` for fixture stability. The `env!` use near old line 14,290 remains valid. |

#### Global artboard bindings

These bindings span more than one fragment and therefore belong in the same PR:

- `rust_attribution.py` discovers all production `.rs` files below `crates/nuxie-runtime/src`. Every new production fragment must be listed in at least one `file-correspondence-manifest.toml` row and must **not** be added to `rust-additions.toml`. The checker exempts `tests.rs`, names ending in `_tests`, and files below a `tests/` path.
- The manifest's scatter ratchet is already at its maximum: 155 rows currently have multiple Rust modules. Except for `B6-0067`, all affected artboard rows are already multi-module. Replacing the `B6-0067` root path prevents the split from increasing that count.
- Update every affected C17 note that enumerates retained Rust modules. Do not change the underlying parity classification merely because the code's physical address changed.
- `docs/runtime-frame-loop-ownership.toml` has artboard citations in `focus.retained_tree`, `component.identity`, `component.dirt`, `component.dependents`, `component.update_order`, `component.transforms`, `component.clone_drop`, `artboard.advance`, `artboard.update_pass`, `nested_artboard.advance`, `artboard.live_draw`, and `renderer.interface_boundary`. Keep `component.clone_drop` and its `pub struct ArtboardInstance` anchor on the root; change each other `rust_file` or `rust:path:line-range` to its physical fragment.
- `docs/runtime-frame-loop-gaps.toml` has 25 owner-boundary allow entries pointing at `artboard.rs` (entries 0-7, 11-22, and 29-34 in current order). Repoint each to the fragment that contains its anchor. A literal move should preserve its token-based `site_hash`; a changed hash is a signal that the split was not mechanical.
- `tools/runtime-frame-loop-port/summarize_trace.py` hard-codes `artboard.rs` nine times for add-dirt, dirt consumption, owner resolution, dependency build, skin rebuild, advancing dispatch, resetting dispatch, and IK-chain discovery. Give each query its actual fragment path. The demangled owner should remain `artboard::ArtboardInstance` because `include!` does not introduce a child module.
- `tools/runtime-frame-loop-port/check_layout_style_handlers.py` hard-codes the artboard source while looking for `propagate_layout_component_display_changed` and the display property key. Point it to the fragment(s) containing those markers.
- `tools/runtime-frame-loop-port/test_check.py` contains at least 14 exact artboard paths covering probe gating, layer occurrence, live-advance ratchets, registry site hashes/substitution, and trait-anchor behavior. Update each synthetic fixture to the fragment that owns the tested anchor; do not perform a blind string replacement.
- Leave the existing root-level includes (`nested_artboard.rs`, `artboard_component_list.rs`, `artboard_list_map_rule.rs`, `artboard_referencer.rs`, `bindable_artboard.rs`, `nested_artboard_layout.rs`, `nested_artboard_leaf.rs`, `component_origin.rs`, bone/weight, profiler/profile) where they are. Moving them under `artboard/` changes relative include resolution and correspondence ownership.
- Keep `crates/nuxie-runtime/src/lib.rs`'s `mod artboard;` unchanged. The public and crate-private paths must not acquire an extra `artboard::<fragment>` segment.

### Mechanical PR sequence and proof

One PR should contain only this file's split and the path/anchor bookkeeping above. Recommended commit sequence within that PR:

1. Extract `tests.rs`, fix only its fixture paths, and prove the same 205 tests are collected.
2. Extract production regions in original order using `include!`; make no renames, visibility edits, formatting sweeps, or logic edits.
3. Update correspondence C17/path fields and all frame-loop physical anchors.
4. Update checker source locations and synthetic fixtures, including an assertion that demangled ownership remains `artboard::ArtboardInstance`.

Proof battery:

```text
make rust-sources-fresh
make cpp-probe
make runtime-frame-loop-port-test
make runtime-frame-loop-port-check
make rust-attribution-check
cargo check --workspace
cargo test -p nuxie-runtime
cargo test -p nuxie --features scripting
make scripted-golden-compare
make silver-corpus-test
```

The PR is structure-preserving only if parity row counts/statuses, trace landmarks, owner-boundary hashes, test counts, and golden/silver outputs are unchanged.

---

## 2. Large-file split plan: `state_machine_instance.rs`

### Current structural map

| Lines | Region |
| ---: | --- |
| 1-93 | Imports and ownership commentary |
| 94-255 | Nested event-chain/notify phases, trace structures, and trace recording |
| 256-312 | Semantic-node resolver and audio/event seam types |
| 313-1,271 | `RuntimeHitResult`, `HitComponent`, and hierarchy implementations |
| 1,272-1,467 | Nested notifier, focus, semantic, and state helper types |
| 1,468-1,474 | `RuntimeDataContextBindError` |
| 1,475-1,711 | `StateMachineInstance` storage |
| 1,712-1,744 | Data-bind occurrence/pointer/executor structures |
| 1,745-2,147 | Listener-action executor implementations |
| 2,148-2,335 | `Clone for StateMachineInstance` |
| 2,336-2,356 | `Drop for StateMachineInstance` |
| 2,357-2,638 | Free helpers and view-model listener types |
| 2,639-13,938 | Main `impl StateMachineInstance` |
| 13,939-13,953 | `runtime_owned_font` helper |
| 13,954-14,079 | `view_model_listener_tests` (`1` test) |
| 14,080-21,674 | `scripted_listener_action_tests` (`93` tests) |

The main implementation divides as follows:

| Lines | Logical responsibility |
| ---: | --- |
| 2,639-2,932 | Teardown, enrollment, focus manager, clone rebind |
| 2,933-3,741 | Construction and listener/layer initialization |
| 3,742-5,102 | Scripting, hydration, and adoption |
| 5,103-6,007 | State/input/focus/semantic/keyboard/gamepad operations |
| 6,008-7,487 | Hit/listener/pointer pipeline |
| 7,488-8,525 | Event notification, deferred callbacks, listener actions |
| 8,526-9,100 | Direct bind targets and value readers |
| 9,101-10,748 | Default/imported/owned source setters and relinking |
| 10,749-12,599 | Data-context bind/rebind, listener cells, bind updates |
| 12,600-13,938 | Reports, advance/apply, nested event dispatch, settlement |

### Proposed physical layout

```text
src/state_machine/
  state_machine_instance.rs               owner/types, hit hierarchy, lifecycle, ordered include! list
  state_machine_instance/
    construction.rs                       current 2639-3741
    scripted_objects.rs                   current 3742-5102
    input_focus.rs                        current 5103-6007
    pointer.rs                            current 6008-7487
    events_actions.rs                     current 7488-8525
    bind_targets.rs                       current 8526-9100
    bind_sources.rs                       current 9101-10748
    data_context.rs                       current 10749-12599
    advance.rs                            current 12600-13953
    tests/
      view_model_listener.rs              body of current first test module
      scripted_listener_actions.rs         body of current second test module
```

Retain the two existing module names in the root and include only their bodies. This preserves test paths and any name-sensitive filtering:

```rust
#[cfg(test)]
mod view_model_listener_tests {
    include!("state_machine_instance/tests/view_model_listener.rs");
}
```

Use the same pattern for `scripted_listener_action_tests`.

### Lockstep correspondence and checker manifest

| New file | B6 rows that must name it | Frame-loop/checker bindings that must move with it |
| --- | --- | --- |
| `state_machine_instance/construction.rs` | `B6-0077`, `B6-0194`, `B6-0195`, `B6-0196`, `B6-0200`, `B6-0310` | Repoint construction/enrollment lifecycle citations and matching synthetic fixtures. Keep `state_machine.collections`' `pub struct StateMachineInstance` anchor on the root. |
| `state_machine_instance/scripted_objects.rs` | `B6-0077`, `B6-0175`, `B6-0194`, `B6-0195`, `B6-0196`, `B6-0200` | Repoint scripting/hydration lifecycle citations and scripted-action fixture paths. |
| `state_machine_instance/input_focus.rs` | `B6-0077`, `B6-0083` | Repoint focus/semantic/keyboard/gamepad citations and tests. |
| `state_machine_instance/pointer.rs` | `B6-0077`, `B6-0083` | Repoint hit/pointer/listener lifecycle citations and FL-C4/layer/hit fixtures. |
| `state_machine_instance/events_actions.rs` | `B6-0058`, `B6-0077`, `B6-0440` | Repoint event reporting, firing/bubbling, and action-dispatch citations/fixtures. `state_machine.actions`' primary anchor remains on the root if the listener-action executor stays there. |
| `state_machine_instance/bind_targets.rs` | `B6-0058`, `B6-0194`, `B6-0196`, `B6-0200`, `B6-0440` | Repoint bind-target and view-model listener citations/fixtures. |
| `state_machine_instance/bind_sources.rs` | `B6-0058`, `B6-0175`, `B6-0194`, `B6-0195`, `B6-0196`, `B6-0200`, `B6-0440` | Repoint source/relink/data-converter citations and bind fixtures. |
| `state_machine_instance/data_context.rs` | `B6-0058`, `B6-0077`, `B6-0194`, `B6-0195`, `B6-0196`, `B6-0200`, `B6-0440` | Repoint data-context lifecycle citations and listener-cell/bind fixtures. |
| `state_machine_instance/advance.rs` | `B6-0077`, `B6-0194`, `B6-0195`, `B6-0196`, `B6-0200`, `B6-0310`, `B6-0440` | Move `state_machine.advance`'s `rust_file` and `advance_with_report_mode` anchor here. Repoint state-machine event/action/reporting lifecycle citations and advance/keyframe fixtures. Include `runtime_owned_font` in this fragment to avoid an artificial one-function file. |
| `state_machine_instance/tests/view_model_listener.rs` | None; it resides below a `tests/` path. | Move the body unchanged and preserve the outer module name. |
| `state_machine_instance/tests/scripted_listener_actions.rs` | None; it resides below a `tests/` path. | Move the body unchanged. Replace or correct the relative fixture path near old line 19,011 (`../../../../fixtures/sync/vm_listener_fire_event.riv`) because the source becomes two directories deeper; prefer a `CARGO_MANIFEST_DIR`-anchored path. |

#### Global state-machine bindings

- All production fragments must be named by `file-correspondence-manifest.toml`, never `rust-additions.toml`. Every affected SMI row is already multi-module, so this split does not increase the manifest's 155-row scatter count.
- Update C17 retained-module notes for `B6-0058`, `B6-0077`, `B6-0083`, `B6-0175`, `B6-0194`, `B6-0195`, `B6-0196`, `B6-0200`, `B6-0310`, and `B6-0440`; leave the substantive parity assertions untouched.
- `docs/runtime-frame-loop-ownership.toml` cites this file from `focus.retained_tree`, `state_machine.collections`, `state_machine.advance`, `state_machine.events`, `state_machine.actions`, and `event.reporting`. Keep root anchors for the struct, stored event collection, and action executor. Repoint method/lifecycle citations to physical fragments.
- `tools/runtime-frame-loop-port/check.py` currently treats one hard-coded SMI file as the nested-event owner and excludes it from outside-owner scans. Once methods are in included fragments, a path-only exclusion would falsely flag the fragments. Make owner discovery include-aware: recursively expand literal local `include!("...")` files in source order for export analysis, and treat every expanded file as the same logical owner. Do not broaden the exclusion to a directory glob.
- Add checker unit coverage for the include-aware logical owner: an exported nested-event method in an included fragment must be accepted, while the same method in an unrelated file must still fail. Preserve syntax parsing and duplicate-anchor detection.
- `tools/runtime-frame-loop-port/test_check.py` has at least 14 exact SMI references, plus constructed/split path strings, covering FL-C4, lifecycle, layer, hit, bind, events, firing/bubbling, advance, owner exports, keyframes, focus, and semantic behavior. Point each fixture to the actual fragment; do not simply substitute the root path everywhere.
- `docs/runtime-frame-loop-gaps.toml` has no current SMI owner-boundary allow row to migrate.
- Keep `crates/nuxie-runtime/src/state_machine.rs`'s `pub(crate) mod state_machine_instance;` and its re-exports unchanged. Literal includes keep all symbols at the original module path.

### Mechanical PR sequence and proof

This should be a second, independent PR after the artboard split has landed. It uses the same proof battery. The checker must first learn the logical-owner/include relationship in the same PR, because moving the methods without that update creates false ownership failures. Apart from that narrowly required path-awareness, no checker policy should change.

Success means the same 94 tests are collected, no exported API path changes, the correspondence scatter count stays at 155, no new owner-boundary allowance appears, and the full battery shown in the artboard plan remains green.

---

## 3. Nuxie-only seam inventory

### Classification method

The authoritative starting point is correspondence metadata:

- Production Rust paths mapped to upstream C++ in `file-correspondence-manifest.toml` are port-side unless a smaller product-only region can be proven.
- Production Rust paths listed in `rust-additions.toml` are intentional Rust-only additions. The relevant ownership labels are `flowsession-abi`, `scene-api`, and generated/codegen infrastructure.
- `build.rs` is outside the attribution scanner's `src/` roots, so it must be inspected manually.
- A convenience façade is not automatically product-side. Thin Rust-only namespace/generated-storage coordinators that serve the port stay with the port even though no one C++ file corresponds to them.

This prevents two opposite mistakes: burying product authoring features inside the port forever, and extracting Rust implementation infrastructure merely because its file has no one-to-one C++ source.

### Product features and mixed glue

| Region | Current size and location | Runtime internals reached / extraction surface | Placement assessment |
| --- | --- | --- | --- |
| FlowSession wire/domain model | `crates/nuxie/src/flow_session.rs:21-621` (601 lines) | Public `File`, `Factory`, renderer, artboard, view-model, and state-machine types | Clean product vocabulary; move to `nuxie-flow` after dependency direction is fixed. |
| FlowSession orchestration | `flow_session.rs:622-2163` (1,542) | Calls crate-private script preparation, listener preparation, rehydration, apply/rollback, host-cycle, and command-drain methods on the `nuxie` façade | Product-side, but requires a narrow public/sealed flow-runtime bridge before crate extraction. Moderate coupling. |
| FlowSession value snapshots | `flow_session.rs:2164-3105` (942) | Traverses public runtime-owned view-model/value objects and builds its own graph/arena | Product-side and cohesive. Mostly mechanical after bridge creation. |
| FlowSession validation/accounting | `flow_session.rs:3106-3545` (440) | Product protocol limits and payload validation | Clean extraction candidate. |
| FlowSession mutation/apply/diff | `flow_session.rs:3546-4515` (970) | Applies mutations through runtime view-model/state-machine handles; shares rollback/command sequencing with façade-private methods | Product-side; transactional sequencing makes it medium risk. |
| FlowSession selection/catalog | `flow_session.rs:4516-5020` (505) | File/artboard/state-machine discovery | Product-side; mostly public runtime surface. |
| FlowSession tests | `flow_session.rs:5021-7227` (2,207; 40 tests), plus `crates/nuxie/tests/flow_session_contract.rs` (335) | Exercises the public ABI, VM lifecycle, and transactional behavior | Move with FlowSession. Preserve public names through compatibility re-exports during migration. |
| Scene contract and IDs | `crates/nuxie/src/scene.rs:28-1187` (1,160) | Public runtime/binary identities and errors | Product-side, cleanly demarcatable. |
| Scene durable definitions/index/specs | `scene.rs:1188-3401` (2,214) | `nuxie_binary::AuthoringRecord`, property/value types, runtime file identities | Product-side authoring model. |
| Scene hierarchy/materialization | `scene.rs:3402-5622` (2,221) | Constructs/reads `RuntimeFile`; directly touches `materialized.file.runtime` in a handful of places even though `File::runtime()` already exists | Product-side but high coupling. Replace direct private field access with the existing public accessor before moving. |
| Silver/scene export schema | `scene.rs:5623-6819` (1,197) | Public exported-record/document DTOs; lowering later feeds `RuntimeFile::from_authoring_records` | Product-side. The schema and its generated counterpart must move together. |
| Scene store/live mount API | `scene.rs:6820-9099` (2,280) | Owns mounted `File`/artboard/view-model state and coordinates transactions | Product-side; medium-to-high coupling. |
| SceneTx and subordinate transactions | `scene.rs:9100-15589` (6,490) | `SceneTx`, `DataConverterTx`, `VmTx`, `AnimTx`, and `MachineTx` mutate the authoring graph, then materialize into runtime/binary types | Core product authoring seam. Cohesive, but atomic commit/materialization behavior makes the move high risk. |
| Scene frame queries/live mutation | `scene.rs:15590-17592` (2,003) | Hit/geometry/semantic queries and intrinsic image sizing reach crate-private `OwnedArtboardInstance` helpers | Product-side. Needs stable snapshot/command DTOs instead of exposing mutable runtime internals. |
| Scene lowering/validation/export | `scene.rs:17593-25563` (7,971) | Converts authoring records into `RuntimeFile`, validates cross-object invariants, and produces exported documents | Product-side. Highest-risk region because it couples the authoring model to port construction invariants. |
| Scene tests | `scene.rs:25564-38224` (12,661; 120 tests), plus `crates/nuxie/tests/scene_authoring.rs` (15,674) and related authoring tests | Broad transaction, materialization, export, and live-runtime contract | Move with Scene; use this suite as the seam migration oracle. |
| Scene schema generator | `crates/nuxie/build.rs` (4,531) and `scene.rs`'s `include!(concat!(env!("OUT_DIR"), "/scene_schema.rs"))` near line 860 | Generates the Scene/silver record schema at build time | Product-side and inseparable from Scene. Extraction must move the build script/generator inputs and preserve the generated include contract. |
| Script import authorization | `crates/nuxie/src/script_import.rs:1-157` production, `158-211` tests | Ed25519/container verification; calls crate-private `File::execution_authorization_for`/authorization state | Product policy. Move to `nuxie-flow`, but expose a capability-based import seam rather than making raw authorization state public. |
| Mixed façade bridge | `crates/nuxie/src/lib.rs`, principally project-envelope classification and private bridge methods around lines 55, 154, 223-258, 307, 344, 376, 1,492, 2,983, 3,328, 4,304, and 5,600-5,733 | `File::from_runtime`; flow script/listener lifecycle; apply/rollback/command drain; artboard geometry/semantic/intrinsic-image helpers | Not an independent feature. Convert these call sites into explicit narrow bridge traits/DTOs, then move their product consumers. |

The concrete crate-private methods the extraction must account for are:

- Flow lifecycle around `nuxie/src/lib.rs:5600-5675`: `prepare_flow_scripts`, `prepare_flow_listener_actions`, rehydrate/apply, begin host cycle, rollback, and drain commands.
- Scene live/runtime queries around `lib.rs:5705-5733`: hit-test path segments with bounds, geometry path segments, retained/semantic text, and intrinsic image-dimension registration.
- `File::from_runtime` around `lib.rs:4304` and the script execution authorization capability.

These are already cross-crate calls from `nuxie` into public `nuxie-runtime` types; the missing visibility is chiefly inside the `nuxie` façade. Extraction should expose a sealed product bridge from the future base façade, not expand dozens of runtime fields to `pub`.

### Project-data converter: product model inside the port

`crates/nuxie-runtime/src/project_data_converter.rs` is 2,686 lines and is explicitly classified `scene-api` in `rust-additions.toml`:

| Lines | Region | Coupling and disposition |
| ---: | --- | --- |
| 22-507 | Public values, specs, errors, and state | Pure product/protocol model; eventual neutral-crate candidate. |
| 508-1,289 | Envelope/program/catalog compilation and evaluation | Mostly `std`/`serde`; eventual neutral-crate candidate. |
| 1,290-2,324 | Validation, coercion, formatting, interpolation | Pure-looking core, but behavior is part of runtime data-bind semantics. Extract only with parity tests. |
| 2,325-2,686 | Formula parser | Mechanically separable after the outer seam is established. |

The file is locally self-contained, but its runtime integration is not. `data_bind/context/context_value.rs` has a `RuntimeDataBindGraphConverter::Project` variant and owns conversion/evaluator state; `data_bind/data_bind_context.rs` constructs it from a script-asset envelope; both runtime and `nuxie` re-export or classify its types; Scene authors and lowers them. A direct move to `nuxie-flow` would invert the desired dependency and make the port depend on the product crate.

Two crate-private numeric-policy helpers in this file (the bounded-list constant near line 467 and helper near line 2,320) are also used by port-side Number-to-List/context conversion. If the model is eventually extracted, those policies should remain in the port or move to a dependency-neutral shared crate; they should not become a product API solely to satisfy the move.

Recommendation: first demarcate this as `project_data_converter/{model, program, evaluator, bridge}` inside `nuxie-runtime`; later extract only the pure model/program/evaluator to a neutral `nuxie-project-data` crate that both runtime and `nuxie-flow` may depend on. The runtime adapter stays in `nuxie-runtime`.

### Rust-only infrastructure that should remain port-side

The following `rust-additions.toml` entries have no one-file C++ correspondence but are not product features:

| Current path/group | Lines | Why it stays in `nuxie-runtime` |
| --- | ---: | --- |
| `data_bind_container.rs`, `data_converter.rs`, `retained_data_bind.rs` | 12 total | Tiny generated/type façade modules over port functionality. |
| `input/mod.rs`, `shapes/mod.rs`, `shapes/paint/mod.rs` | 168 total | Rust namespace/generated-storage coordinators. |
| `objects.rs` | 1,018 | Runtime object registry plus `include!(concat!(env!("OUT_DIR"), "/runtime_objects.rs"))`; generated by `crates/nuxie-runtime/build.rs` (707 lines). This is port construction infrastructure. |
| `properties.rs` | 826 | Generated property interpretation/storage used throughout the port. |
| `state_machine.rs`, `state_machine/instance.rs`, `state_machine/listener_types/mod.rs` | 215 total | Rust module/re-export façade for the ported state-machine implementation. |
| `viewmodel/mod.rs`, `viewmodel/runtime/mod.rs` | 58 total | Rust namespace/re-export façade for ported view-model behavior. |
| `focus.rs` | 7 | Thin public façade over retained focus behavior, not a standalone product feature. |

These codegen/facade additions total about 2,303 source lines excluding the project converter and focus façade. They should be clearly marked as Rust port infrastructure, but moving them to `nuxie-flow` would blur rather than improve the seam.

`crates/nuxie/src/command_queue.rs`, server code, and raw-text support are not included in the Nuxie-only list: their current files are explicitly mapped to upstream C++ correspondence. A richer Rust API around a ported facility does not by itself make that facility product-side.

---

## 4. Recommended seam design

### Decision

Use a **module boundary now and a crate boundary later**.

The immediate structure should make product ownership visible without changing dependencies:

```text
crates/nuxie/src/product/
  flow_session/
  scene/
    model/
    transaction/
    live/
    export/
  script_import/

crates/nuxie-runtime/src/project_data_converter/
  model.rs
  program.rs
  evaluator.rs
  bridge.rs
```

Preserve current public paths with root re-exports. This first step is a namespace/demarcation change, not an extraction. It gives reviewers an auditable boundary while the current test and parity baseline still observes exactly the same crate graph.

The long-term crate graph should avoid a cycle:

```text
nuxie-runtime  <---  nuxie-core  <---  nuxie-flow
       ^                 ^                 ^
       |                 |                 |
       +------ nuxie-project-data --------+

nuxie (umbrella compatibility crate) re-exports nuxie-core + nuxie-flow
```

`nuxie-flow` contains FlowSession, Scene/SceneTx, silver/export DTOs and lowering, and script-import policy. `nuxie-core` contains `File`, `Factory`, owned artboard/view-model handles, renderer integration, and narrow bridge traits. `nuxie-runtime` remains the C++-correspondence port plus explicitly labeled Rust port infrastructure. A small dependency-neutral `nuxie-project-data` is optional and should be introduced only after the runtime adapter has been separated from the pure evaluator.

### Public surface of the port/base layer

The extraction should not make arbitrary runtime fields public. The required surface is narrow:

1. A sealed/trusted constructor that wraps a validated `RuntimeFile` as a façade `File`, with an explicit authoring/import authorization policy.
2. A flow-runtime bridge for prepare, rehydrate, apply, host-cycle, rollback, and command-drain operations. Its inputs/outputs should be stable value DTOs, not references to internal queues.
3. Read-only artboard snapshot operations for hit geometry, retained geometry, and semantic text, plus a command for intrinsic image dimensions. Return owned snapshots so Scene cannot retain internal graph borrows.
4. A project-converter adapter owned by `nuxie-runtime`; pure program/model types may come from the neutral crate. Do not expose the runtime bind graph or evaluator cells as product API.
5. Existing upstream-corresponding runtime types only where they are already public and semantically stable. Product re-exports should live in `nuxie`/`nuxie-flow`, not expand the port's promise.

### Migration order

1. Establish the exact green parity/golden baseline and keep it pinned in every PR.
2. Land the `artboard.rs` and `state_machine_instance.rs` mechanical splits as separate PRs. This makes later ownership changes reviewable without mixing 20,000-line source moves into semantic work.
3. Demarcate `nuxie::product::{flow_session, scene, script_import}` and the project-converter submodules in place, preserving public re-exports.
4. Define the sealed flow, authoring-file, geometry-snapshot, and script-authorization bridges; migrate existing internal callers to them while everything remains in the same crate.
5. Extract the non-product façade/runtime handles into `nuxie-core`, with `nuxie` temporarily acting as an umbrella compatibility crate.
6. Move FlowSession and script import to `nuxie-flow`; move their tests and retain compatibility re-exports.
7. Move Scene model, SceneTx, export schema/generator, lowering, live adapter, and tests together. Do not split the generated schema from `build.rs` or split transaction commit from materialization in the extraction PR.
8. Isolate the project converter's pure model/program/evaluator. Extract it to a neutral crate only if doing so leaves the runtime adapter dependency pointing inward, never from `nuxie-runtime` to `nuxie-flow`.
9. Remove compatibility re-exports and tighten visibility in a later breaking-change PR, after downstream migration.

### Risk map

| Risk | Areas | Reason |
| --- | --- | --- |
| Mechanical/low | Thin namespace façades, codegen coordinators staying in place, test-module extraction, pure FlowSession validation/types | Mostly physical ownership with no runtime mutation semantics. |
| Moderate | FlowSession after bridge creation, script import after capability API, Scene DTO/export types | Public API and authorization compatibility, but limited graph mutation. |
| High | SceneTx commit/materialization, live Scene mounting, geometry/semantic access, Scene lowering and generated schema | Atomicity, identity, generated code, and live runtime invariants cross the seam. |
| High | Project-data converter extraction | The source looks pure, but evaluator state is embedded in runtime data-bind context and Number-to-List policy. |
| High | Script/listener ownership during FlowSession moves | Rehydration, rollback, command queues, and VM object lifetime must remain one coherent transaction. |

### Why baseline-first matters

`docs/parity-gap-register.md` explicitly recommends making the parity claim trustworthy before feature work and landing each subsequent change against the oracle (the “Recommended execution order” around lines 204-211). It also describes Scene as a Nuxie-only additive API (around lines 126-127) while recording FlowSession/capability-boundary gaps elsewhere in the register. That is the governing principle here: first make source moves mechanically observable by the existing correspondence, ownership, trace, golden, and silver batteries; then introduce the product seam behind those same oracles. A green build alone is insufficient because it can preserve compilation while silently weakening ownership scans or changing what “zero divergences” measures.

The two large-file splits are therefore prerequisites for, not part of, the product extraction. Each split keeps the logical Rust module unchanged and updates only physical-address consumers. The seam work then proceeds through explicit bridges and compatibility re-exports, with risky transaction/materialization changes isolated from mechanical moves.

# nuxie-runtime structure report

## Scope and baseline

This is a read-only structure analysis. It proposes no behavior, source, parity-ledger, or ownership-ledger changes.

The inspected worktree was already detached at `e8726db8ffc689d3b19f1c0a55794aa6daf9956d` (`Merge pull request #246 from TheOneLevin/feat/b6-0058-listener-viewmodel-change`, 2026-08-04). The local `origin/main` ref resolves to the same commit. The requested `git fetch origin` and redundant `git checkout --detach origin/main` could not update the shared worktree metadata because the sandbox cannot write the parent repository's `.git/worktrees/nuxie-mr-c12` directory. The analysis is therefore pinned to the locally available `origin/main` commit above, not a newly fetched remote ref. The pre-existing untracked `vtriage-capture/` directory was left untouched.

The two target files have grown beyond the sizes in the request:

| File | Current lines | Production/support | Tests |
| --- | ---: | ---: | ---: |
| `crates/nuxie-runtime/src/artboard.rs` | 23,374 | 12,039 | 11,335 (`205` tests) |
| `crates/nuxie-runtime/src/state_machine/state_machine_instance.rs` | 21,674 | 14,079 | 7,595 (`94` tests) |

The safest split mechanism is literal `include!` fragments, not Rust child modules. An included fragment remains in the original semantic module (`crate::artboard` or `crate::state_machine::state_machine_instance`), so private-name resolution, inherent method paths, sibling access, and demangled symbols remain unchanged. A `mod build;`/`#[path = ...] mod build;` split would introduce new privacy boundaries and make a supposedly mechanical PR into an API refactor.

---

## 1. Large-file split plan: `artboard.rs`

### Current structural map

| Lines | Region |
| ---: | --- |
| 1-106 | Imports and supporting aliases |
| 107-135 | Draw-frame counter and generated `Mat2D` helper |
| 136-160 | `ExternalFontAssetError` |
| 161-181 | `RuntimeArtboardInstanceIdentity` |
| 182-231 | `RuntimeScriptState` and implementations |
| 232-250 | Advancing-component and persistent-dirt fixtures |
| 251-298 | Script advance/update mode enums and implementations |
| 299-327 | Resetting/runtime-component helper types |
| 328-334 | `RuntimeTextStyleFeatureOption` |
| 335-523 | `ArtboardInstance` storage |
| 524-714 | `Clone for ArtboardInstance` |
| 715-739 | `Drop for ArtboardInstance` followed by the existing leaf `include!` declarations |
| 740-773 | Occurrence and frame-advance report types |
| 774-844 | Build context/index/text-flag helpers |
| 845-860 | `RuntimeNestedAnimationInstance` |
| 861-11,133 | Main `impl ArtboardInstance` |
| 11,134-11,147 | Focus-scroll path and focus-bounds transform helper |
| 11,148-11,187 | Small follow-up `impl ArtboardInstance` |
| 11,188-11,226 | Component-list reset helper |
| 11,227-11,595 | `impl RuntimeNestedArtboardInstance` |
| 11,596-11,611 | `impl RuntimeNestedAnimationInstance` |
| 11,612-12,039 | Free construction/nested-artboard helpers |
| 12,040-23,374 | Single `#[cfg(test)] mod tests` (`205` tests) |

The main implementation itself divides along stable responsibility boundaries:

| Lines | Logical responsibility |
| ---: | --- |
| 861-1,104 | Lifecycle, data context, and transient-clone restoration |
| 1,105-2,440 | Build occurrence relationships and constructors |
| 2,441-2,712 | Path/hit initialization and external assets |
| 2,713-3,399 | Scripting lifecycle |
| 3,400-3,989 | Object/component/audio/hit/graph/definition access |
| 3,990-4,152 | Scripted interpolators and state-machine definition selection |
| 4,153-4,480 | Dimensions, world/layout bounds, scrolling, and focus reveal |
| 4,481-5,229 | Generated property façade and text values |
| 5,230-5,568 | Animation/state-machine/view-model occurrences |
| 5,569-6,312 | Component-list lifecycle and layout |
| 6,313-7,408 | Root/nested frame advance and retained-component dispatch |
| 7,409-7,585 | Nested-layout frame propagation |
| 7,586-8,496 | Transforms, epochs, dirt, and invalidation |
| 8,497-9,191 | Update pass and five-pass state-machine settlement |
| 9,192-9,825 | Component update dispatch and script scheduling |
| 9,826-11,133 | Property-change callbacks, nested controls, and collapse |

### Proposed physical layout

Keep `artboard.rs` as the owner. It retains imports, supporting types, `ArtboardInstance`, `Clone`, `Drop`, the existing leaf includes, occurrence/report/build helper types, and an ordered series of literal includes. This preserves declaration order where it matters and makes all fragments part of `crate::artboard`.

```text
src/
  artboard.rs                         owner/types, existing leaf includes, ordered include! list
  artboard/
    build.rs                          current 861-2712
    script_runtime.rs                 current 2713-3399
    access.rs                         current 3400-3989
    definitions_and_layout.rs         current 3990-4480
    properties.rs                     current 4481-5568
    component_lists.rs                current 5569-6312
    advance.rs                        current 6313-7585
    dirt.rs                           current 7586-8496
    update.rs                         current 8497-9825
    callbacks.rs                      current 9826-11133
    nested.rs                         current 11134-12039
    tests.rs                          body of current tests module
```

The production fragments are intentionally in execution/dependency order rather than alphabetical order. `tests.rs` should be included inside the original test module:

```rust
#[cfg(test)]
mod tests {
    include!("artboard/tests.rs");
}
```

This retains the test module's name and access. It also gives the attribution checker a deliberately exempt `tests.rs` leaf.

### Lockstep correspondence and checker manifest

The following table is the required per-file edit manifest. “B6 rows” are `[[files]]` entries in `file-correspondence-manifest.toml`; add the fragment's path to those rows' semicolon-separated `rust_module` values and update their C17 retained-module notes. Keep status, upstream pins, C++ paths, verification text, and correspondence claims unchanged. The row list is conservative: a later provenance cleanup can narrow it, but narrowing must not be mixed into this mechanical split.

| New file | B6 rows that must name it | Frame-loop/checker bindings that must move with it |
| --- | --- | --- |
| `artboard/build.rs` | `B6-0001`, `B6-0094`, `B6-0115`, `B6-0117`, `B6-0123`, `B6-0125`, `B6-0129`, `B6-0131`, `B6-0142`, `B6-0146`, `B6-0202`, `B6-0258`, `B6-0303`, `B6-0304`, `B6-0305`, `B6-0331`, `B6-0361`, `B6-0362`, `B6-0365`, `B6-0368`, `B6-0385`, `B6-0388`, `B6-0399`, `B6-0405`, `B6-0408` | Repoint `summarize_trace.py` dependency-build and IK-chain-build source scans when their anchors land here. Repoint matching `runtime-frame-loop-gaps.toml` owner-boundary entries. |
| `artboard/script_runtime.rs` | `B6-0077`, `B6-0194`, `B6-0200`, `B6-0322`, `B6-0326` | Repoint the script-lifecycle citations in `runtime-frame-loop-ownership.toml` and any corresponding synthetic test fixture paths. |
| `artboard/access.rs` | `B6-0094`, `B6-0095`, `B6-0123`, `B6-0146`, `B6-0203`, `B6-0204`, `B6-0205` | Repoint any live-draw/renderer-interface lifecycle citations whose named methods move here. |
| `artboard/definitions_and_layout.rs` | `B6-0077`, `B6-0094`, `B6-0258`, `B6-0303`, `B6-0304`, `B6-0305`, `B6-0385`, `B6-0388` | Repoint focus-retained-tree and layout lifecycle citations, plus matching owner-boundary allow entries. |
| `artboard/properties.rs` | `B6-0067`, `B6-0077`, `B6-0094`, `B6-0194`, `B6-0200`, `B6-0352`, `B6-0355`, `B6-0385`, `B6-0388`, `B6-0399`, `B6-0404`, `B6-0405`, `B6-0408` | `B6-0067` currently has only one Rust module: **replace** `artboard.rs` with this fragment rather than adding a second path. Repoint property/text lifecycle citations and corresponding owner-boundary entries. |
| `artboard/component_lists.rs` | `B6-0095`, `B6-0258`, `B6-0303`, `B6-0304`, `B6-0305` | Repoint component-list and nested-layout lifecycle citations and matching test fixtures. |
| `artboard/advance.rs` | `B6-0001`, `B6-0059`, `B6-0061`, `B6-0063`, `B6-0064`, `B6-0077`, `B6-0094`, `B6-0095`, `B6-0194`, `B6-0200`, `B6-0258`, `B6-0303`, `B6-0304`, `B6-0305`, `B6-0322`, `B6-0326`, `B6-0388` | In `runtime-frame-loop-ownership.toml`, move `artboard.advance`'s `rust_file` and the lifecycle citations for artboard/nested advance. Repoint `summarize_trace.py` advancing/resetting dispatch scans as applicable. Repoint relevant gap allow rows and `test_check.py` advance fixtures. |
| `artboard/dirt.rs` | `B6-0094`, `B6-0123`, `B6-0258`, `B6-0331`, `B6-0352`, `B6-0355`, `B6-0357`, `B6-0361`, `B6-0362`, `B6-0365`, `B6-0368`, `B6-0385`, `B6-0388`, `B6-0399`, `B6-0405`, `B6-0408` | Move `component.dirt`, `component.dependents` (`pub fn add_dirt(`), and transform/dirt lifecycle citations as appropriate. Repoint `summarize_trace.py`'s two add-dirt anchors, dirt consumptions, owner resolutions, and skin-buffer scans according to their final physical locations. Repoint matching gap rows. |
| `artboard/update.rs` | `B6-0001`, `B6-0094`, `B6-0115`, `B6-0117`, `B6-0123`, `B6-0125`, `B6-0129`, `B6-0131`, `B6-0142`, `B6-0194`, `B6-0200`, `B6-0203`, `B6-0204`, `B6-0205`, `B6-0258`, `B6-0303`, `B6-0304`, `B6-0305`, `B6-0322`, `B6-0326`, `B6-0331`, `B6-0352`, `B6-0355`, `B6-0357`, `B6-0361`, `B6-0362`, `B6-0365`, `B6-0368`, `B6-0385`, `B6-0388`, `B6-0399`, `B6-0404`, `B6-0405`, `B6-0408` | Move `component.update_order` (`fn update_components_with_hook_recording`) and `artboard.update_pass` (`pub fn update_pass(`) `rust_file` values and lifecycle citations. Repoint retained update/settlement gap rows, trace scans, and test fixtures. |
| `artboard/callbacks.rs` | `B6-0059`, `B6-0061`, `B6-0063`, `B6-0064`, `B6-0077`, `B6-0094`, `B6-0123`, `B6-0125`, `B6-0129`, `B6-0131`, `B6-0142`, `B6-0194`, `B6-0200`, `B6-0258`, `B6-0303`, `B6-0304`, `B6-0305`, `B6-0331`, `B6-0352`, `B6-0355`, `B6-0357`, `B6-0361`, `B6-0362`, `B6-0365`, `B6-0368`, `B6-0385`, `B6-0388`, `B6-0399`, `B6-0404`, `B6-0405`, `B6-0408` | Change `check_layout_style_handlers.py`'s `RUST_ARTBOARD_MODULE` to this fragment if both display-change markers remain here; otherwise make the checker accept the two explicit physical files. Repoint callback lifecycle citations, gap rows, and fixtures. |
| `artboard/nested.rs` | `B6-0059`, `B6-0061`, `B6-0063`, `B6-0064`, `B6-0077`, `B6-0094`, `B6-0095`, `B6-0258`, `B6-0303`, `B6-0304`, `B6-0305` | Repoint nested-artboard advance/layout citations, gap allow entries, and nested fixtures. |
| `artboard/tests.rs` | None. `rust_attribution.py` excludes a source whose stem is `tests`. | Move the artboard test body unchanged. Update the seven relative `include_bytes!`/fixture paths currently near old lines 19,951, 20,001, 20,168, 20,542, 20,597, 20,612, and 23,311 because the file becomes one directory deeper. Prefer `concat!(env!("CARGO_MANIFEST_DIR"), "/...")` for fixture stability. The `env!` use near old line 14,290 remains valid. |

#### Global artboard bindings

These bindings span more than one fragment and therefore belong in the same PR:

- `rust_attribution.py` discovers all production `.rs` files below `crates/nuxie-runtime/src`. Every new production fragment must be listed in at least one `file-correspondence-manifest.toml` row and must **not** be added to `rust-additions.toml`. The checker exempts `tests.rs`, names ending in `_tests`, and files below a `tests/` path.
- The manifest's scatter ratchet is already at its maximum: 155 rows currently have multiple Rust modules. Except for `B6-0067`, all affected artboard rows are already multi-module. Replacing the `B6-0067` root path prevents the split from increasing that count.
- Update every affected C17 note that enumerates retained Rust modules. Do not change the underlying parity classification merely because the code's physical address changed.
- `docs/runtime-frame-loop-ownership.toml` has artboard citations in `focus.retained_tree`, `component.identity`, `component.dirt`, `component.dependents`, `component.update_order`, `component.transforms`, `component.clone_drop`, `artboard.advance`, `artboard.update_pass`, `nested_artboard.advance`, `artboard.live_draw`, and `renderer.interface_boundary`. Keep `component.clone_drop` and its `pub struct ArtboardInstance` anchor on the root; change each other `rust_file` or `rust:path:line-range` to its physical fragment.
- `docs/runtime-frame-loop-gaps.toml` has 25 owner-boundary allow entries pointing at `artboard.rs` (entries 0-7, 11-22, and 29-34 in current order). Repoint each to the fragment that contains its anchor. A literal move should preserve its token-based `site_hash`; a changed hash is a signal that the split was not mechanical.
- `tools/runtime-frame-loop-port/summarize_trace.py` hard-codes `artboard.rs` nine times for add-dirt, dirt consumption, owner resolution, dependency build, skin rebuild, advancing dispatch, resetting dispatch, and IK-chain discovery. Give each query its actual fragment path. The demangled owner should remain `artboard::ArtboardInstance` because `include!` does not introduce a child module.
- `tools/runtime-frame-loop-port/check_layout_style_handlers.py` hard-codes the artboard source while looking for `propagate_layout_component_display_changed` and the display property key. Point it to the fragment(s) containing those markers.
- `tools/runtime-frame-loop-port/test_check.py` contains at least 14 exact artboard paths covering probe gating, layer occurrence, live-advance ratchets, registry site hashes/substitution, and trait-anchor behavior. Update each synthetic fixture to the fragment that owns the tested anchor; do not perform a blind string replacement.
- Leave the existing root-level includes (`nested_artboard.rs`, `artboard_component_list.rs`, `artboard_list_map_rule.rs`, `artboard_referencer.rs`, `bindable_artboard.rs`, `nested_artboard_layout.rs`, `nested_artboard_leaf.rs`, `component_origin.rs`, bone/weight, profiler/profile) where they are. Moving them under `artboard/` changes relative include resolution and correspondence ownership.
- Keep `crates/nuxie-runtime/src/lib.rs`'s `mod artboard;` unchanged. The public and crate-private paths must not acquire an extra `artboard::<fragment>` segment.

### Mechanical PR sequence and proof

One PR should contain only this file's split and the path/anchor bookkeeping above. Recommended commit sequence within that PR:

1. Extract `tests.rs`, fix only its fixture paths, and prove the same 205 tests are collected.
2. Extract production regions in original order using `include!`; make no renames, visibility edits, formatting sweeps, or logic edits.
3. Update correspondence C17/path fields and all frame-loop physical anchors.
4. Update checker source locations and synthetic fixtures, including an assertion that demangled ownership remains `artboard::ArtboardInstance`.

Proof battery:

```text
make rust-sources-fresh
make cpp-probe
make runtime-frame-loop-port-test
make runtime-frame-loop-port-check
make rust-attribution-check
cargo check --workspace
cargo test -p nuxie-runtime
cargo test -p nuxie --features scripting
make scripted-golden-compare
make silver-corpus-test
```

The PR is structure-preserving only if parity row counts/statuses, trace landmarks, owner-boundary hashes, test counts, and golden/silver outputs are unchanged.

---

## 2. Large-file split plan: `state_machine_instance.rs`

### Current structural map

| Lines | Region |
| ---: | --- |
| 1-93 | Imports and ownership commentary |
| 94-255 | Nested event-chain/notify phases, trace structures, and trace recording |
| 256-312 | Semantic-node resolver and audio/event seam types |
| 313-1,271 | `RuntimeHitResult`, `HitComponent`, and hierarchy implementations |
| 1,272-1,467 | Nested notifier, focus, semantic, and state helper types |
| 1,468-1,474 | `RuntimeDataContextBindError` |
| 1,475-1,711 | `StateMachineInstance` storage |
| 1,712-1,744 | Data-bind occurrence/pointer/executor structures |
| 1,745-2,147 | Listener-action executor implementations |
| 2,148-2,335 | `Clone for StateMachineInstance` |
| 2,336-2,356 | `Drop for StateMachineInstance` |
| 2,357-2,638 | Free helpers and view-model listener types |
| 2,639-13,938 | Main `impl StateMachineInstance` |
| 13,939-13,953 | `runtime_owned_font` helper |
| 13,954-14,079 | `view_model_listener_tests` (`1` test) |
| 14,080-21,674 | `scripted_listener_action_tests` (`93` tests) |

The main implementation divides as follows:

| Lines | Logical responsibility |
| ---: | --- |
| 2,639-2,932 | Teardown, enrollment, focus manager, clone rebind |
| 2,933-3,741 | Construction and listener/layer initialization |
| 3,742-5,102 | Scripting, hydration, and adoption |
| 5,103-6,007 | State/input/focus/semantic/keyboard/gamepad operations |
| 6,008-7,487 | Hit/listener/pointer pipeline |
| 7,488-8,525 | Event notification, deferred callbacks, listener actions |
| 8,526-9,100 | Direct bind targets and value readers |
| 9,101-10,748 | Default/imported/owned source setters and relinking |
| 10,749-12,599 | Data-context bind/rebind, listener cells, bind updates |
| 12,600-13,938 | Reports, advance/apply, nested event dispatch, settlement |

### Proposed physical layout

```text
src/state_machine/
  state_machine_instance.rs               owner/types, hit hierarchy, lifecycle, ordered include! list
  state_machine_instance/
    construction.rs                       current 2639-3741
    scripted_objects.rs                   current 3742-5102
    input_focus.rs                        current 5103-6007
    pointer.rs                            current 6008-7487
    events_actions.rs                     current 7488-8525
    bind_targets.rs                       current 8526-9100
    bind_sources.rs                       current 9101-10748
    data_context.rs                       current 10749-12599
    advance.rs                            current 12600-13953
    tests/
      view_model_listener.rs              body of current first test module
      scripted_listener_actions.rs         body of current second test module
```

Retain the two existing module names in the root and include only their bodies. This preserves test paths and any name-sensitive filtering:

```rust
#[cfg(test)]
mod view_model_listener_tests {
    include!("state_machine_instance/tests/view_model_listener.rs");
}
```

Use the same pattern for `scripted_listener_action_tests`.

### Lockstep correspondence and checker manifest

| New file | B6 rows that must name it | Frame-loop/checker bindings that must move with it |
| --- | --- | --- |
| `state_machine_instance/construction.rs` | `B6-0077`, `B6-0194`, `B6-0195`, `B6-0196`, `B6-0200`, `B6-0310` | Repoint construction/enrollment lifecycle citations and matching synthetic fixtures. Keep `state_machine.collections`' `pub struct StateMachineInstance` anchor on the root. |
| `state_machine_instance/scripted_objects.rs` | `B6-0077`, `B6-0175`, `B6-0194`, `B6-0195`, `B6-0196`, `B6-0200` | Repoint scripting/hydration lifecycle citations and scripted-action fixture paths. |
| `state_machine_instance/input_focus.rs` | `B6-0077`, `B6-0083` | Repoint focus/semantic/keyboard/gamepad citations and tests. |
| `state_machine_instance/pointer.rs` | `B6-0077`, `B6-0083` | Repoint hit/pointer/listener lifecycle citations and FL-C4/layer/hit fixtures. |
| `state_machine_instance/events_actions.rs` | `B6-0058`, `B6-0077`, `B6-0440` | Repoint event reporting, firing/bubbling, and action-dispatch citations/fixtures. `state_machine.actions`' primary anchor remains on the root if the listener-action executor stays there. |
| `state_machine_instance/bind_targets.rs` | `B6-0058`, `B6-0194`, `B6-0196`, `B6-0200`, `B6-0440` | Repoint bind-target and view-model listener citations/fixtures. |
| `state_machine_instance/bind_sources.rs` | `B6-0058`, `B6-0175`, `B6-0194`, `B6-0195`, `B6-0196`, `B6-0200`, `B6-0440` | Repoint source/relink/data-converter citations and bind fixtures. |
| `state_machine_instance/data_context.rs` | `B6-0058`, `B6-0077`, `B6-0194`, `B6-0195`, `B6-0196`, `B6-0200`, `B6-0440` | Repoint data-context lifecycle citations and listener-cell/bind fixtures. |
| `state_machine_instance/advance.rs` | `B6-0077`, `B6-0194`, `B6-0195`, `B6-0196`, `B6-0200`, `B6-0310`, `B6-0440` | Move `state_machine.advance`'s `rust_file` and `advance_with_report_mode` anchor here. Repoint state-machine event/action/reporting lifecycle citations and advance/keyframe fixtures. Include `runtime_owned_font` in this fragment to avoid an artificial one-function file. |
| `state_machine_instance/tests/view_model_listener.rs` | None; it resides below a `tests/` path. | Move the body unchanged and preserve the outer module name. |
| `state_machine_instance/tests/scripted_listener_actions.rs` | None; it resides below a `tests/` path. | Move the body unchanged. Replace or correct the relative fixture path near old line 19,011 (`../../../../fixtures/sync/vm_listener_fire_event.riv`) because the source becomes two directories deeper; prefer a `CARGO_MANIFEST_DIR`-anchored path. |

#### Global state-machine bindings

- All production fragments must be named by `file-correspondence-manifest.toml`, never `rust-additions.toml`. Every affected SMI row is already multi-module, so this split does not increase the manifest's 155-row scatter count.
- Update C17 retained-module notes for `B6-0058`, `B6-0077`, `B6-0083`, `B6-0175`, `B6-0194`, `B6-0195`, `B6-0196`, `B6-0200`, `B6-0310`, and `B6-0440`; leave the substantive parity assertions untouched.
- `docs/runtime-frame-loop-ownership.toml` cites this file from `focus.retained_tree`, `state_machine.collections`, `state_machine.advance`, `state_machine.events`, `state_machine.actions`, and `event.reporting`. Keep root anchors for the struct, stored event collection, and action executor. Repoint method/lifecycle citations to physical fragments.
- `tools/runtime-frame-loop-port/check.py` currently treats one hard-coded SMI file as the nested-event owner and excludes it from outside-owner scans. Once methods are in included fragments, a path-only exclusion would falsely flag the fragments. Make owner discovery include-aware: recursively expand literal local `include!("...")` files in source order for export analysis, and treat every expanded file as the same logical owner. Do not broaden the exclusion to a directory glob.
- Add checker unit coverage for the include-aware logical owner: an exported nested-event method in an included fragment must be accepted, while the same method in an unrelated file must still fail. Preserve syntax parsing and duplicate-anchor detection.
- `tools/runtime-frame-loop-port/test_check.py` has at least 14 exact SMI references, plus constructed/split path strings, covering FL-C4, lifecycle, layer, hit, bind, events, firing/bubbling, advance, owner exports, keyframes, focus, and semantic behavior. Point each fixture to the actual fragment; do not simply substitute the root path everywhere.
- `docs/runtime-frame-loop-gaps.toml` has no current SMI owner-boundary allow row to migrate.
- Keep `crates/nuxie-runtime/src/state_machine.rs`'s `pub(crate) mod state_machine_instance;` and its re-exports unchanged. Literal includes keep all symbols at the original module path.

### Mechanical PR sequence and proof

This should be a second, independent PR after the artboard split has landed. It uses the same proof battery. The checker must first learn the logical-owner/include relationship in the same PR, because moving the methods without that update creates false ownership failures. Apart from that narrowly required path-awareness, no checker policy should change.

Success means the same 94 tests are collected, no exported API path changes, the correspondence scatter count stays at 155, no new owner-boundary allowance appears, and the full battery shown in the artboard plan remains green.

---

## 3. Nuxie-only seam inventory

### Classification method

The authoritative starting point is correspondence metadata:

- Production Rust paths mapped to upstream C++ in `file-correspondence-manifest.toml` are port-side unless a smaller product-only region can be proven.
- Production Rust paths listed in `rust-additions.toml` are intentional Rust-only additions. The relevant ownership labels are `flowsession-abi`, `scene-api`, and generated/codegen infrastructure.
- `build.rs` is outside the attribution scanner's `src/` roots, so it must be inspected manually.
- A convenience façade is not automatically product-side. Thin Rust-only namespace/generated-storage coordinators that serve the port stay with the port even though no one C++ file corresponds to them.

This prevents two opposite mistakes: burying product authoring features inside the port forever, and extracting Rust implementation infrastructure merely because its file has no one-to-one C++ source.

### Product features and mixed glue

| Region | Current size and location | Runtime internals reached / extraction surface | Placement assessment |
| --- | --- | --- | --- |
| FlowSession wire/domain model | `crates/nuxie/src/flow_session.rs:21-621` (601 lines) | Public `File`, `Factory`, renderer, artboard, view-model, and state-machine types | Clean product vocabulary; move to `nuxie-flow` after dependency direction is fixed. |
| FlowSession orchestration | `flow_session.rs:622-2163` (1,542) | Calls crate-private script preparation, listener preparation, rehydration, apply/rollback, host-cycle, and command-drain methods on the `nuxie` façade | Product-side, but requires a narrow public/sealed flow-runtime bridge before crate extraction. Moderate coupling. |
| FlowSession value snapshots | `flow_session.rs:2164-3105` (942) | Traverses public runtime-owned view-model/value objects and builds its own graph/arena | Product-side and cohesive. Mostly mechanical after bridge creation. |
| FlowSession validation/accounting | `flow_session.rs:3106-3545` (440) | Product protocol limits and payload validation | Clean extraction candidate. |
| FlowSession mutation/apply/diff | `flow_session.rs:3546-4515` (970) | Applies mutations through runtime view-model/state-machine handles; shares rollback/command sequencing with façade-private methods | Product-side; transactional sequencing makes it medium risk. |
| FlowSession selection/catalog | `flow_session.rs:4516-5020` (505) | File/artboard/state-machine discovery | Product-side; mostly public runtime surface. |
| FlowSession tests | `flow_session.rs:5021-7227` (2,207; 40 tests), plus `crates/nuxie/tests/flow_session_contract.rs` (335) | Exercises the public ABI, VM lifecycle, and transactional behavior | Move with FlowSession. Preserve public names through compatibility re-exports during migration. |
| Scene contract and IDs | `crates/nuxie/src/scene.rs:28-1187` (1,160) | Public runtime/binary identities and errors | Product-side, cleanly demarcatable. |
| Scene durable definitions/index/specs | `scene.rs:1188-3401` (2,214) | `nuxie_binary::AuthoringRecord`, property/value types, runtime file identities | Product-side authoring model. |
| Scene hierarchy/materialization | `scene.rs:3402-5622` (2,221) | Constructs/reads `RuntimeFile`; directly touches `materialized.file.runtime` in a handful of places even though `File::runtime()` already exists | Product-side but high coupling. Replace direct private field access with the existing public accessor before moving. |
| Silver/scene export schema | `scene.rs:5623-6819` (1,197) | Public exported-record/document DTOs; lowering later feeds `RuntimeFile::from_authoring_records` | Product-side. The schema and its generated counterpart must move together. |
| Scene store/live mount API | `scene.rs:6820-9099` (2,280) | Owns mounted `File`/artboard/view-model state and coordinates transactions | Product-side; medium-to-high coupling. |
| SceneTx and subordinate transactions | `scene.rs:9100-15589` (6,490) | `SceneTx`, `DataConverterTx`, `VmTx`, `AnimTx`, and `MachineTx` mutate the authoring graph, then materialize into runtime/binary types | Core product authoring seam. Cohesive, but atomic commit/materialization behavior makes the move high risk. |
| Scene frame queries/live mutation | `scene.rs:15590-17592` (2,003) | Hit/geometry/semantic queries and intrinsic image sizing reach crate-private `OwnedArtboardInstance` helpers | Product-side. Needs stable snapshot/command DTOs instead of exposing mutable runtime internals. |
| Scene lowering/validation/export | `scene.rs:17593-25563` (7,971) | Converts authoring records into `RuntimeFile`, validates cross-object invariants, and produces exported documents | Product-side. Highest-risk region because it couples the authoring model to port construction invariants. |
| Scene tests | `scene.rs:25564-38224` (12,661; 120 tests), plus `crates/nuxie/tests/scene_authoring.rs` (15,674) and related authoring tests | Broad transaction, materialization, export, and live-runtime contract | Move with Scene; use this suite as the seam migration oracle. |
| Scene schema generator | `crates/nuxie/build.rs` (4,531) and `scene.rs`'s `include!(concat!(env!("OUT_DIR"), "/scene_schema.rs"))` near line 860 | Generates the Scene/silver record schema at build time | Product-side and inseparable from Scene. Extraction must move the build script/generator inputs and preserve the generated include contract. |
| Script import authorization | `crates/nuxie/src/script_import.rs:1-157` production, `158-211` tests | Ed25519/container verification; calls crate-private `File::execution_authorization_for`/authorization state | Product policy. Move to `nuxie-flow`, but expose a capability-based import seam rather than making raw authorization state public. |
| Mixed façade bridge | `crates/nuxie/src/lib.rs`, principally project-envelope classification and private bridge methods around lines 55, 154, 223-258, 307, 344, 376, 1,492, 2,983, 3,328, 4,304, and 5,600-5,733 | `File::from_runtime`; flow script/listener lifecycle; apply/rollback/command drain; artboard geometry/semantic/intrinsic-image helpers | Not an independent feature. Convert these call sites into explicit narrow bridge traits/DTOs, then move their product consumers. |

The concrete crate-private methods the extraction must account for are:

- Flow lifecycle around `nuxie/src/lib.rs:5600-5675`: `prepare_flow_scripts`, `prepare_flow_listener_actions`, rehydrate/apply, begin host cycle, rollback, and drain commands.
- Scene live/runtime queries around `lib.rs:5705-5733`: hit-test path segments with bounds, geometry path segments, retained/semantic text, and intrinsic image-dimension registration.
- `File::from_runtime` around `lib.rs:4304` and the script execution authorization capability.

These are already cross-crate calls from `nuxie` into public `nuxie-runtime` types; the missing visibility is chiefly inside the `nuxie` façade. Extraction should expose a sealed product bridge from the future base façade, not expand dozens of runtime fields to `pub`.

### Project-data converter: product model inside the port

`crates/nuxie-runtime/src/project_data_converter.rs` is 2,686 lines and is explicitly classified `scene-api` in `rust-additions.toml`:

| Lines | Region | Coupling and disposition |
| ---: | --- | --- |
| 22-507 | Public values, specs, errors, and state | Pure product/protocol model; eventual neutral-crate candidate. |
| 508-1,289 | Envelope/program/catalog compilation and evaluation | Mostly `std`/`serde`; eventual neutral-crate candidate. |
| 1,290-2,324 | Validation, coercion, formatting, interpolation | Pure-looking core, but behavior is part of runtime data-bind semantics. Extract only with parity tests. |
| 2,325-2,686 | Formula parser | Mechanically separable after the outer seam is established. |

The file is locally self-contained, but its runtime integration is not. `data_bind/context/context_value.rs` has a `RuntimeDataBindGraphConverter::Project` variant and owns conversion/evaluator state; `data_bind/data_bind_context.rs` constructs it from a script-asset envelope; both runtime and `nuxie` re-export or classify its types; Scene authors and lowers them. A direct move to `nuxie-flow` would invert the desired dependency and make the port depend on the product crate.

Two crate-private numeric-policy helpers in this file (the bounded-list constant near line 467 and helper near line 2,320) are also used by port-side Number-to-List/context conversion. If the model is eventually extracted, those policies should remain in the port or move to a dependency-neutral shared crate; they should not become a product API solely to satisfy the move.

Recommendation: first demarcate this as `project_data_converter/{model, program, evaluator, bridge}` inside `nuxie-runtime`; later extract only the pure model/program/evaluator to a neutral `nuxie-project-data` crate that both runtime and `nuxie-flow` may depend on. The runtime adapter stays in `nuxie-runtime`.

### Rust-only infrastructure that should remain port-side

The following `rust-additions.toml` entries have no one-file C++ correspondence but are not product features:

| Current path/group | Lines | Why it stays in `nuxie-runtime` |
| --- | ---: | --- |
| `data_bind_container.rs`, `data_converter.rs`, `retained_data_bind.rs` | 12 total | Tiny generated/type façade modules over port functionality. |
| `input/mod.rs`, `shapes/mod.rs`, `shapes/paint/mod.rs` | 168 total | Rust namespace/generated-storage coordinators. |
| `objects.rs` | 1,018 | Runtime object registry plus `include!(concat!(env!("OUT_DIR"), "/runtime_objects.rs"))`; generated by `crates/nuxie-runtime/build.rs` (707 lines). This is port construction infrastructure. |
| `properties.rs` | 826 | Generated property interpretation/storage used throughout the port. |
| `state_machine.rs`, `state_machine/instance.rs`, `state_machine/listener_types/mod.rs` | 215 total | Rust module/re-export façade for the ported state-machine implementation. |
| `viewmodel/mod.rs`, `viewmodel/runtime/mod.rs` | 58 total | Rust namespace/re-export façade for ported view-model behavior. |
| `focus.rs` | 7 | Thin public façade over retained focus behavior, not a standalone product feature. |

These codegen/facade additions total about 2,303 source lines excluding the project converter and focus façade. They should be clearly marked as Rust port infrastructure, but moving them to `nuxie-flow` would blur rather than improve the seam.

`crates/nuxie/src/command_queue.rs`, server code, and raw-text support are not included in the Nuxie-only list: their current files are explicitly mapped to upstream C++ correspondence. A richer Rust API around a ported facility does not by itself make that facility product-side.

---

## 4. Recommended seam design

### Decision

Use a **module boundary now and a crate boundary later**.

The immediate structure should make product ownership visible without changing dependencies:

```text
crates/nuxie/src/product/
  flow_session/
  scene/
    model/
    transaction/
    live/
    export/
  script_import/

crates/nuxie-runtime/src/project_data_converter/
  model.rs
  program.rs
  evaluator.rs
  bridge.rs
```

Preserve current public paths with root re-exports. This first step is a namespace/demarcation change, not an extraction. It gives reviewers an auditable boundary while the current test and parity baseline still observes exactly the same crate graph.

The long-term crate graph should avoid a cycle:

```text
nuxie-runtime  <---  nuxie-core  <---  nuxie-flow
       ^                 ^                 ^
       |                 |                 |
       +------ nuxie-project-data --------+

nuxie (umbrella compatibility crate) re-exports nuxie-core + nuxie-flow
```

`nuxie-flow` contains FlowSession, Scene/SceneTx, silver/export DTOs and lowering, and script-import policy. `nuxie-core` contains `File`, `Factory`, owned artboard/view-model handles, renderer integration, and narrow bridge traits. `nuxie-runtime` remains the C++-correspondence port plus explicitly labeled Rust port infrastructure. A small dependency-neutral `nuxie-project-data` is optional and should be introduced only after the runtime adapter has been separated from the pure evaluator.

### Public surface of the port/base layer

The extraction should not make arbitrary runtime fields public. The required surface is narrow:

1. A sealed/trusted constructor that wraps a validated `RuntimeFile` as a façade `File`, with an explicit authoring/import authorization policy.
2. A flow-runtime bridge for prepare, rehydrate, apply, host-cycle, rollback, and command-drain operations. Its inputs/outputs should be stable value DTOs, not references to internal queues.
3. Read-only artboard snapshot operations for hit geometry, retained geometry, and semantic text, plus a command for intrinsic image dimensions. Return owned snapshots so Scene cannot retain internal graph borrows.
4. A project-converter adapter owned by `nuxie-runtime`; pure program/model types may come from the neutral crate. Do not expose the runtime bind graph or evaluator cells as product API.
5. Existing upstream-corresponding runtime types only where they are already public and semantically stable. Product re-exports should live in `nuxie`/`nuxie-flow`, not expand the port's promise.

### Migration order

1. Establish the exact green parity/golden baseline and keep it pinned in every PR.
2. Land the `artboard.rs` and `state_machine_instance.rs` mechanical splits as separate PRs. This makes later ownership changes reviewable without mixing 20,000-line source moves into semantic work.
3. Demarcate `nuxie::product::{flow_session, scene, script_import}` and the project-converter submodules in place, preserving public re-exports.
4. Define the sealed flow, authoring-file, geometry-snapshot, and script-authorization bridges; migrate existing internal callers to them while everything remains in the same crate.
5. Extract the non-product façade/runtime handles into `nuxie-core`, with `nuxie` temporarily acting as an umbrella compatibility crate.
6. Move FlowSession and script import to `nuxie-flow`; move their tests and retain compatibility re-exports.
7. Move Scene model, SceneTx, export schema/generator, lowering, live adapter, and tests together. Do not split the generated schema from `build.rs` or split transaction commit from materialization in the extraction PR.
8. Isolate the project converter's pure model/program/evaluator. Extract it to a neutral crate only if doing so leaves the runtime adapter dependency pointing inward, never from `nuxie-runtime` to `nuxie-flow`.
9. Remove compatibility re-exports and tighten visibility in a later breaking-change PR, after downstream migration.

### Risk map

| Risk | Areas | Reason |
| --- | --- | --- |
| Mechanical/low | Thin namespace façades, codegen coordinators staying in place, test-module extraction, pure FlowSession validation/types | Mostly physical ownership with no runtime mutation semantics. |
| Moderate | FlowSession after bridge creation, script import after capability API, Scene DTO/export types | Public API and authorization compatibility, but limited graph mutation. |
| High | SceneTx commit/materialization, live Scene mounting, geometry/semantic access, Scene lowering and generated schema | Atomicity, identity, generated code, and live runtime invariants cross the seam. |
| High | Project-data converter extraction | The source looks pure, but evaluator state is embedded in runtime data-bind context and Number-to-List policy. |
| High | Script/listener ownership during FlowSession moves | Rehydration, rollback, command queues, and VM object lifetime must remain one coherent transaction. |

### Why baseline-first matters

`docs/parity-gap-register.md` explicitly recommends making the parity claim trustworthy before feature work and landing each subsequent change against the oracle (the “Recommended execution order” around lines 204-211). It also describes Scene as a Nuxie-only additive API (around lines 126-127) while recording FlowSession/capability-boundary gaps elsewhere in the register. That is the governing principle here: first make source moves mechanically observable by the existing correspondence, ownership, trace, golden, and silver batteries; then introduce the product seam behind those same oracles. A green build alone is insufficient because it can preserve compilation while silently weakening ownership scans or changing what “zero divergences” measures.

The two large-file splits are therefore prerequisites for, not part of, the product extraction. Each split keeps the logical Rust module unchanged and updates only physical-address consumers. The seam work then proceeds through explicit bridges and compatibility re-exports, with risky transaction/materialization changes isolated from mechanical moves.

# nuxie-runtime structure report

## Scope and baseline

This is a read-only structure analysis. It proposes no behavior, source, parity-ledger, or ownership-ledger changes.

The inspected worktree was already detached at `e8726db8ffc689d3b19f1c0a55794aa6daf9956d` (`Merge pull request #246 from TheOneLevin/feat/b6-0058-listener-viewmodel-change`, 2026-08-04). The local `origin/main` ref resolves to the same commit. The requested `git fetch origin` and redundant `git checkout --detach origin/main` could not update the shared worktree metadata because the sandbox cannot write the parent repository's `.git/worktrees/nuxie-mr-c12` directory. The analysis is therefore pinned to the locally available `origin/main` commit above, not a newly fetched remote ref. The pre-existing untracked `vtriage-capture/` directory was left untouched.

The two target files have grown beyond the sizes in the request:

| File | Current lines | Production/support | Tests |
| --- | ---: | ---: | ---: |
| `crates/nuxie-runtime/src/artboard.rs` | 23,374 | 12,039 | 11,335 (`205` tests) |
| `crates/nuxie-runtime/src/state_machine/state_machine_instance.rs` | 21,674 | 14,079 | 7,595 (`94` tests) |

The safest split mechanism is literal `include!` fragments, not Rust child modules. An included fragment remains in the original semantic module (`crate::artboard` or `crate::state_machine::state_machine_instance`), so private-name resolution, inherent method paths, sibling access, and demangled symbols remain unchanged. A `mod build;`/`#[path = ...] mod build;` split would introduce new privacy boundaries and make a supposedly mechanical PR into an API refactor.

---

## 1. Large-file split plan: `artboard.rs`

### Current structural map

| Lines | Region |
| ---: | --- |
| 1-106 | Imports and supporting aliases |
| 107-135 | Draw-frame counter and generated `Mat2D` helper |
| 136-160 | `ExternalFontAssetError` |
| 161-181 | `RuntimeArtboardInstanceIdentity` |
| 182-231 | `RuntimeScriptState` and implementations |
| 232-250 | Advancing-component and persistent-dirt fixtures |
| 251-298 | Script advance/update mode enums and implementations |
| 299-327 | Resetting/runtime-component helper types |
| 328-334 | `RuntimeTextStyleFeatureOption` |
| 335-523 | `ArtboardInstance` storage |
| 524-714 | `Clone for ArtboardInstance` |
| 715-739 | `Drop for ArtboardInstance` followed by the existing leaf `include!` declarations |
| 740-773 | Occurrence and frame-advance report types |
| 774-844 | Build context/index/text-flag helpers |
| 845-860 | `RuntimeNestedAnimationInstance` |
| 861-11,133 | Main `impl ArtboardInstance` |
| 11,134-11,147 | Focus-scroll path and focus-bounds transform helper |
| 11,148-11,187 | Small follow-up `impl ArtboardInstance` |
| 11,188-11,226 | Component-list reset helper |
| 11,227-11,595 | `impl RuntimeNestedArtboardInstance` |
| 11,596-11,611 | `impl RuntimeNestedAnimationInstance` |
| 11,612-12,039 | Free construction/nested-artboard helpers |
| 12,040-23,374 | Single `#[cfg(test)] mod tests` (`205` tests) |

The main implementation itself divides along stable responsibility boundaries:

| Lines | Logical responsibility |
| ---: | --- |
| 861-1,104 | Lifecycle, data context, and transient-clone restoration |
| 1,105-2,440 | Build occurrence relationships and constructors |
| 2,441-2,712 | Path/hit initialization and external assets |
| 2,713-3,399 | Scripting lifecycle |
| 3,400-3,989 | Object/component/audio/hit/graph/definition access |
| 3,990-4,152 | Scripted interpolators and state-machine definition selection |
| 4,153-4,480 | Dimensions, world/layout bounds, scrolling, and focus reveal |
| 4,481-5,229 | Generated property façade and text values |
| 5,230-5,568 | Animation/state-machine/view-model occurrences |
| 5,569-6,312 | Component-list lifecycle and layout |
| 6,313-7,408 | Root/nested frame advance and retained-component dispatch |
| 7,409-7,585 | Nested-layout frame propagation |
| 7,586-8,496 | Transforms, epochs, dirt, and invalidation |
| 8,497-9,191 | Update pass and five-pass state-machine settlement |
| 9,192-9,825 | Component update dispatch and script scheduling |
| 9,826-11,133 | Property-change callbacks, nested controls, and collapse |

### Proposed physical layout

Keep `artboard.rs` as the owner. It retains imports, supporting types, `ArtboardInstance`, `Clone`, `Drop`, the existing leaf includes, occurrence/report/build helper types, and an ordered series of literal includes. This preserves declaration order where it matters and makes all fragments part of `crate::artboard`.

```text
src/
  artboard.rs                         owner/types, existing leaf includes, ordered include! list
  artboard/
    build.rs                          current 861-2712
    script_runtime.rs                 current 2713-3399
    access.rs                         current 3400-3989
    definitions_and_layout.rs         current 3990-4480
    properties.rs                     current 4481-5568
    component_lists.rs                current 5569-6312
    advance.rs                        current 6313-7585
    dirt.rs                           current 7586-8496
    update.rs                         current 8497-9825
    callbacks.rs                      current 9826-11133
    nested.rs                         current 11134-12039
    tests.rs                          body of current tests module
```

The production fragments are intentionally in execution/dependency order rather than alphabetical order. `tests.rs` should be included inside the original test module:

```rust
#[cfg(test)]
mod tests {
    include!("artboard/tests.rs");
}
```

This retains the test module's name and access. It also gives the attribution checker a deliberately exempt `tests.rs` leaf.

### Lockstep correspondence and checker manifest

The following table is the required per-file edit manifest. “B6 rows” are `[[files]]` entries in `file-correspondence-manifest.toml`; add the fragment's path to those rows' semicolon-separated `rust_module` values and update their C17 retained-module notes. Keep status, upstream pins, C++ paths, verification text, and correspondence claims unchanged. The row list is conservative: a later provenance cleanup can narrow it, but narrowing must not be mixed into this mechanical split.

| New file | B6 rows that must name it | Frame-loop/checker bindings that must move with it |
| --- | --- | --- |
| `artboard/build.rs` | `B6-0001`, `B6-0094`, `B6-0115`, `B6-0117`, `B6-0123`, `B6-0125`, `B6-0129`, `B6-0131`, `B6-0142`, `B6-0146`, `B6-0202`, `B6-0258`, `B6-0303`, `B6-0304`, `B6-0305`, `B6-0331`, `B6-0361`, `B6-0362`, `B6-0365`, `B6-0368`, `B6-0385`, `B6-0388`, `B6-0399`, `B6-0405`, `B6-0408` | Repoint `summarize_trace.py` dependency-build and IK-chain-build source scans when their anchors land here. Repoint matching `runtime-frame-loop-gaps.toml` owner-boundary entries. |
| `artboard/script_runtime.rs` | `B6-0077`, `B6-0194`, `B6-0200`, `B6-0322`, `B6-0326` | Repoint the script-lifecycle citations in `runtime-frame-loop-ownership.toml` and any corresponding synthetic test fixture paths. |
| `artboard/access.rs` | `B6-0094`, `B6-0095`, `B6-0123`, `B6-0146`, `B6-0203`, `B6-0204`, `B6-0205` | Repoint any live-draw/renderer-interface lifecycle citations whose named methods move here. |
| `artboard/definitions_and_layout.rs` | `B6-0077`, `B6-0094`, `B6-0258`, `B6-0303`, `B6-0304`, `B6-0305`, `B6-0385`, `B6-0388` | Repoint focus-retained-tree and layout lifecycle citations, plus matching owner-boundary allow entries. |
| `artboard/properties.rs` | `B6-0067`, `B6-0077`, `B6-0094`, `B6-0194`, `B6-0200`, `B6-0352`, `B6-0355`, `B6-0385`, `B6-0388`, `B6-0399`, `B6-0404`, `B6-0405`, `B6-0408` | `B6-0067` currently has only one Rust module: **replace** `artboard.rs` with this fragment rather than adding a second path. Repoint property/text lifecycle citations and corresponding owner-boundary entries. |
| `artboard/component_lists.rs` | `B6-0095`, `B6-0258`, `B6-0303`, `B6-0304`, `B6-0305` | Repoint component-list and nested-layout lifecycle citations and matching test fixtures. |
| `artboard/advance.rs` | `B6-0001`, `B6-0059`, `B6-0061`, `B6-0063`, `B6-0064`, `B6-0077`, `B6-0094`, `B6-0095`, `B6-0194`, `B6-0200`, `B6-0258`, `B6-0303`, `B6-0304`, `B6-0305`, `B6-0322`, `B6-0326`, `B6-0388` | In `runtime-frame-loop-ownership.toml`, move `artboard.advance`'s `rust_file` and the lifecycle citations for artboard/nested advance. Repoint `summarize_trace.py` advancing/resetting dispatch scans as applicable. Repoint relevant gap allow rows and `test_check.py` advance fixtures. |
| `artboard/dirt.rs` | `B6-0094`, `B6-0123`, `B6-0258`, `B6-0331`, `B6-0352`, `B6-0355`, `B6-0357`, `B6-0361`, `B6-0362`, `B6-0365`, `B6-0368`, `B6-0385`, `B6-0388`, `B6-0399`, `B6-0405`, `B6-0408` | Move `component.dirt`, `component.dependents` (`pub fn add_dirt(`), and transform/dirt lifecycle citations as appropriate. Repoint `summarize_trace.py`'s two add-dirt anchors, dirt consumptions, owner resolutions, and skin-buffer scans according to their final physical locations. Repoint matching gap rows. |
| `artboard/update.rs` | `B6-0001`, `B6-0094`, `B6-0115`, `B6-0117`, `B6-0123`, `B6-0125`, `B6-0129`, `B6-0131`, `B6-0142`, `B6-0194`, `B6-0200`, `B6-0203`, `B6-0204`, `B6-0205`, `B6-0258`, `B6-0303`, `B6-0304`, `B6-0305`, `B6-0322`, `B6-0326`, `B6-0331`, `B6-0352`, `B6-0355`, `B6-0357`, `B6-0361`, `B6-0362`, `B6-0365`, `B6-0368`, `B6-0385`, `B6-0388`, `B6-0399`, `B6-0404`, `B6-0405`, `B6-0408` | Move `component.update_order` (`fn update_components_with_hook_recording`) and `artboard.update_pass` (`pub fn update_pass(`) `rust_file` values and lifecycle citations. Repoint retained update/settlement gap rows, trace scans, and test fixtures. |
| `artboard/callbacks.rs` | `B6-0059`, `B6-0061`, `B6-0063`, `B6-0064`, `B6-0077`, `B6-0094`, `B6-0123`, `B6-0125`, `B6-0129`, `B6-0131`, `B6-0142`, `B6-0194`, `B6-0200`, `B6-0258`, `B6-0303`, `B6-0304`, `B6-0305`, `B6-0331`, `B6-0352`, `B6-0355`, `B6-0357`, `B6-0361`, `B6-0362`, `B6-0365`, `B6-0368`, `B6-0385`, `B6-0388`, `B6-0399`, `B6-0404`, `B6-0405`, `B6-0408` | Change `check_layout_style_handlers.py`'s `RUST_ARTBOARD_MODULE` to this fragment if both display-change markers remain here; otherwise make the checker accept the two explicit physical files. Repoint callback lifecycle citations, gap rows, and fixtures. |
| `artboard/nested.rs` | `B6-0059`, `B6-0061`, `B6-0063`, `B6-0064`, `B6-0077`, `B6-0094`, `B6-0095`, `B6-0258`, `B6-0303`, `B6-0304`, `B6-0305` | Repoint nested-artboard advance/layout citations, gap allow entries, and nested fixtures. |
| `artboard/tests.rs` | None. `rust_attribution.py` excludes a source whose stem is `tests`. | Move the artboard test body unchanged. Update the seven relative `include_bytes!`/fixture paths currently near old lines 19,951, 20,001, 20,168, 20,542, 20,597, 20,612, and 23,311 because the file becomes one directory deeper. Prefer `concat!(env!("CARGO_MANIFEST_DIR"), "/...")` for fixture stability. The `env!` use near old line 14,290 remains valid. |

#### Global artboard bindings

These bindings span more than one fragment and therefore belong in the same PR:

- `rust_attribution.py` discovers all production `.rs` files below `crates/nuxie-runtime/src`. Every new production fragment must be listed in at least one `file-correspondence-manifest.toml` row and must **not** be added to `rust-additions.toml`. The checker exempts `tests.rs`, names ending in `_tests`, and files below a `tests/` path.
- The manifest's scatter ratchet is already at its maximum: 155 rows currently have multiple Rust modules. Except for `B6-0067`, all affected artboard rows are already multi-module. Replacing the `B6-0067` root path prevents the split from increasing that count.
- Update every affected C17 note that enumerates retained Rust modules. Do not change the underlying parity classification merely because the code's physical address changed.
- `docs/runtime-frame-loop-ownership.toml` has artboard citations in `focus.retained_tree`, `component.identity`, `component.dirt`, `component.dependents`, `component.update_order`, `component.transforms`, `component.clone_drop`, `artboard.advance`, `artboard.update_pass`, `nested_artboard.advance`, `artboard.live_draw`, and `renderer.interface_boundary`. Keep `component.clone_drop` and its `pub struct ArtboardInstance` anchor on the root; change each other `rust_file` or `rust:path:line-range` to its physical fragment.
- `docs/runtime-frame-loop-gaps.toml` has 25 owner-boundary allow entries pointing at `artboard.rs` (entries 0-7, 11-22, and 29-34 in current order). Repoint each to the fragment that contains its anchor. A literal move should preserve its token-based `site_hash`; a changed hash is a signal that the split was not mechanical.
- `tools/runtime-frame-loop-port/summarize_trace.py` hard-codes `artboard.rs` nine times for add-dirt, dirt consumption, owner resolution, dependency build, skin rebuild, advancing dispatch, resetting dispatch, and IK-chain discovery. Give each query its actual fragment path. The demangled owner should remain `artboard::ArtboardInstance` because `include!` does not introduce a child module.
- `tools/runtime-frame-loop-port/check_layout_style_handlers.py` hard-codes the artboard source while looking for `propagate_layout_component_display_changed` and the display property key. Point it to the fragment(s) containing those markers.
- `tools/runtime-frame-loop-port/test_check.py` contains at least 14 exact artboard paths covering probe gating, layer occurrence, live-advance ratchets, registry site hashes/substitution, and trait-anchor behavior. Update each synthetic fixture to the fragment that owns the tested anchor; do not perform a blind string replacement.
- Leave the existing root-level includes (`nested_artboard.rs`, `artboard_component_list.rs`, `artboard_list_map_rule.rs`, `artboard_referencer.rs`, `bindable_artboard.rs`, `nested_artboard_layout.rs`, `nested_artboard_leaf.rs`, `component_origin.rs`, bone/weight, profiler/profile) where they are. Moving them under `artboard/` changes relative include resolution and correspondence ownership.
- Keep `crates/nuxie-runtime/src/lib.rs`'s `mod artboard;` unchanged. The public and crate-private paths must not acquire an extra `artboard::<fragment>` segment.

### Mechanical PR sequence and proof

One PR should contain only this file's split and the path/anchor bookkeeping above. Recommended commit sequence within that PR:

1. Extract `tests.rs`, fix only its fixture paths, and prove the same 205 tests are collected.
2. Extract production regions in original order using `include!`; make no renames, visibility edits, formatting sweeps, or logic edits.
3. Update correspondence C17/path fields and all frame-loop physical anchors.
4. Update checker source locations and synthetic fixtures, including an assertion that demangled ownership remains `artboard::ArtboardInstance`.

Proof battery:

```text
make rust-sources-fresh
make cpp-probe
make runtime-frame-loop-port-test
make runtime-frame-loop-port-check
make rust-attribution-check
cargo check --workspace
cargo test -p nuxie-runtime
cargo test -p nuxie --features scripting
make scripted-golden-compare
make silver-corpus-test
```

The PR is structure-preserving only if parity row counts/statuses, trace landmarks, owner-boundary hashes, test counts, and golden/silver outputs are unchanged.

---

## 2. Large-file split plan: `state_machine_instance.rs`

### Current structural map

| Lines | Region |
| ---: | --- |
| 1-93 | Imports and ownership commentary |
| 94-255 | Nested event-chain/notify phases, trace structures, and trace recording |
| 256-312 | Semantic-node resolver and audio/event seam types |
| 313-1,271 | `RuntimeHitResult`, `HitComponent`, and hierarchy implementations |
| 1,272-1,467 | Nested notifier, focus, semantic, and state helper types |
| 1,468-1,474 | `RuntimeDataContextBindError` |
| 1,475-1,711 | `StateMachineInstance` storage |
| 1,712-1,744 | Data-bind occurrence/pointer/executor structures |
| 1,745-2,147 | Listener-action executor implementations |
| 2,148-2,335 | `Clone for StateMachineInstance` |
| 2,336-2,356 | `Drop for StateMachineInstance` |
| 2,357-2,638 | Free helpers and view-model listener types |
| 2,639-13,938 | Main `impl StateMachineInstance` |
| 13,939-13,953 | `runtime_owned_font` helper |
| 13,954-14,079 | `view_model_listener_tests` (`1` test) |
| 14,080-21,674 | `scripted_listener_action_tests` (`93` tests) |

The main implementation divides as follows:

| Lines | Logical responsibility |
| ---: | --- |
| 2,639-2,932 | Teardown, enrollment, focus manager, clone rebind |
| 2,933-3,741 | Construction and listener/layer initialization |
| 3,742-5,102 | Scripting, hydration, and adoption |
| 5,103-6,007 | State/input/focus/semantic/keyboard/gamepad operations |
| 6,008-7,487 | Hit/listener/pointer pipeline |
| 7,488-8,525 | Event notification, deferred callbacks, listener actions |
| 8,526-9,100 | Direct bind targets and value readers |
| 9,101-10,748 | Default/imported/owned source setters and relinking |
| 10,749-12,599 | Data-context bind/rebind, listener cells, bind updates |
| 12,600-13,938 | Reports, advance/apply, nested event dispatch, settlement |

### Proposed physical layout

```text
src/state_machine/
  state_machine_instance.rs               owner/types, hit hierarchy, lifecycle, ordered include! list
  state_machine_instance/
    construction.rs                       current 2639-3741
    scripted_objects.rs                   current 3742-5102
    input_focus.rs                        current 5103-6007
    pointer.rs                            current 6008-7487
    events_actions.rs                     current 7488-8525
    bind_targets.rs                       current 8526-9100
    bind_sources.rs                       current 9101-10748
    data_context.rs                       current 10749-12599
    advance.rs                            current 12600-13953
    tests/
      view_model_listener.rs              body of current first test module
      scripted_listener_actions.rs         body of current second test module
```

Retain the two existing module names in the root and include only their bodies. This preserves test paths and any name-sensitive filtering:

```rust
#[cfg(test)]
mod view_model_listener_tests {
    include!("state_machine_instance/tests/view_model_listener.rs");
}
```

Use the same pattern for `scripted_listener_action_tests`.

### Lockstep correspondence and checker manifest

| New file | B6 rows that must name it | Frame-loop/checker bindings that must move with it |
| --- | --- | --- |
| `state_machine_instance/construction.rs` | `B6-0077`, `B6-0194`, `B6-0195`, `B6-0196`, `B6-0200`, `B6-0310` | Repoint construction/enrollment lifecycle citations and matching synthetic fixtures. Keep `state_machine.collections`' `pub struct StateMachineInstance` anchor on the root. |
| `state_machine_instance/scripted_objects.rs` | `B6-0077`, `B6-0175`, `B6-0194`, `B6-0195`, `B6-0196`, `B6-0200` | Repoint scripting/hydration lifecycle citations and scripted-action fixture paths. |
| `state_machine_instance/input_focus.rs` | `B6-0077`, `B6-0083` | Repoint focus/semantic/keyboard/gamepad citations and tests. |
| `state_machine_instance/pointer.rs` | `B6-0077`, `B6-0083` | Repoint hit/pointer/listener lifecycle citations and FL-C4/layer/hit fixtures. |
| `state_machine_instance/events_actions.rs` | `B6-0058`, `B6-0077`, `B6-0440` | Repoint event reporting, firing/bubbling, and action-dispatch citations/fixtures. `state_machine.actions`' primary anchor remains on the root if the listener-action executor stays there. |
| `state_machine_instance/bind_targets.rs` | `B6-0058`, `B6-0194`, `B6-0196`, `B6-0200`, `B6-0440` | Repoint bind-target and view-model listener citations/fixtures. |
| `state_machine_instance/bind_sources.rs` | `B6-0058`, `B6-0175`, `B6-0194`, `B6-0195`, `B6-0196`, `B6-0200`, `B6-0440` | Repoint source/relink/data-converter citations and bind fixtures. |
| `state_machine_instance/data_context.rs` | `B6-0058`, `B6-0077`, `B6-0194`, `B6-0195`, `B6-0196`, `B6-0200`, `B6-0440` | Repoint data-context lifecycle citations and listener-cell/bind fixtures. |
| `state_machine_instance/advance.rs` | `B6-0077`, `B6-0194`, `B6-0195`, `B6-0196`, `B6-0200`, `B6-0310`, `B6-0440` | Move `state_machine.advance`'s `rust_file` and `advance_with_report_mode` anchor here. Repoint state-machine event/action/reporting lifecycle citations and advance/keyframe fixtures. Include `runtime_owned_font` in this fragment to avoid an artificial one-function file. |
| `state_machine_instance/tests/view_model_listener.rs` | None; it resides below a `tests/` path. | Move the body unchanged and preserve the outer module name. |
| `state_machine_instance/tests/scripted_listener_actions.rs` | None; it resides below a `tests/` path. | Move the body unchanged. Replace or correct the relative fixture path near old line 19,011 (`../../../../fixtures/sync/vm_listener_fire_event.riv`) because the source becomes two directories deeper; prefer a `CARGO_MANIFEST_DIR`-anchored path. |

#### Global state-machine bindings

- All production fragments must be named by `file-correspondence-manifest.toml`, never `rust-additions.toml`. Every affected SMI row is already multi-module, so this split does not increase the manifest's 155-row scatter count.
- Update C17 retained-module notes for `B6-0058`, `B6-0077`, `B6-0083`, `B6-0175`, `B6-0194`, `B6-0195`, `B6-0196`, `B6-0200`, `B6-0310`, and `B6-0440`; leave the substantive parity assertions untouched.
- `docs/runtime-frame-loop-ownership.toml` cites this file from `focus.retained_tree`, `state_machine.collections`, `state_machine.advance`, `state_machine.events`, `state_machine.actions`, and `event.reporting`. Keep root anchors for the struct, stored event collection, and action executor. Repoint method/lifecycle citations to physical fragments.
- `tools/runtime-frame-loop-port/check.py` currently treats one hard-coded SMI file as the nested-event owner and excludes it from outside-owner scans. Once methods are in included fragments, a path-only exclusion would falsely flag the fragments. Make owner discovery include-aware: recursively expand literal local `include!("...")` files in source order for export analysis, and treat every expanded file as the same logical owner. Do not broaden the exclusion to a directory glob.
- Add checker unit coverage for the include-aware logical owner: an exported nested-event method in an included fragment must be accepted, while the same method in an unrelated file must still fail. Preserve syntax parsing and duplicate-anchor detection.
- `tools/runtime-frame-loop-port/test_check.py` has at least 14 exact SMI references, plus constructed/split path strings, covering FL-C4, lifecycle, layer, hit, bind, events, firing/bubbling, advance, owner exports, keyframes, focus, and semantic behavior. Point each fixture to the actual fragment; do not simply substitute the root path everywhere.
- `docs/runtime-frame-loop-gaps.toml` has no current SMI owner-boundary allow row to migrate.
- Keep `crates/nuxie-runtime/src/state_machine.rs`'s `pub(crate) mod state_machine_instance;` and its re-exports unchanged. Literal includes keep all symbols at the original module path.

### Mechanical PR sequence and proof

This should be a second, independent PR after the artboard split has landed. It uses the same proof battery. The checker must first learn the logical-owner/include relationship in the same PR, because moving the methods without that update creates false ownership failures. Apart from that narrowly required path-awareness, no checker policy should change.

Success means the same 94 tests are collected, no exported API path changes, the correspondence scatter count stays at 155, no new owner-boundary allowance appears, and the full battery shown in the artboard plan remains green.

---

## 3. Nuxie-only seam inventory

### Classification method

The authoritative starting point is correspondence metadata:

- Production Rust paths mapped to upstream C++ in `file-correspondence-manifest.toml` are port-side unless a smaller product-only region can be proven.
- Production Rust paths listed in `rust-additions.toml` are intentional Rust-only additions. The relevant ownership labels are `flowsession-abi`, `scene-api`, and generated/codegen infrastructure.
- `build.rs` is outside the attribution scanner's `src/` roots, so it must be inspected manually.
- A convenience façade is not automatically product-side. Thin Rust-only namespace/generated-storage coordinators that serve the port stay with the port even though no one C++ file corresponds to them.

This prevents two opposite mistakes: burying product authoring features inside the port forever, and extracting Rust implementation infrastructure merely because its file has no one-to-one C++ source.

### Product features and mixed glue

| Region | Current size and location | Runtime internals reached / extraction surface | Placement assessment |
| --- | --- | --- | --- |
| FlowSession wire/domain model | `crates/nuxie/src/flow_session.rs:21-621` (601 lines) | Public `File`, `Factory`, renderer, artboard, view-model, and state-machine types | Clean product vocabulary; move to `nuxie-flow` after dependency direction is fixed. |
| FlowSession orchestration | `flow_session.rs:622-2163` (1,542) | Calls crate-private script preparation, listener preparation, rehydration, apply/rollback, host-cycle, and command-drain methods on the `nuxie` façade | Product-side, but requires a narrow public/sealed flow-runtime bridge before crate extraction. Moderate coupling. |
| FlowSession value snapshots | `flow_session.rs:2164-3105` (942) | Traverses public runtime-owned view-model/value objects and builds its own graph/arena | Product-side and cohesive. Mostly mechanical after bridge creation. |
| FlowSession validation/accounting | `flow_session.rs:3106-3545` (440) | Product protocol limits and payload validation | Clean extraction candidate. |
| FlowSession mutation/apply/diff | `flow_session.rs:3546-4515` (970) | Applies mutations through runtime view-model/state-machine handles; shares rollback/command sequencing with façade-private methods | Product-side; transactional sequencing makes it medium risk. |
| FlowSession selection/catalog | `flow_session.rs:4516-5020` (505) | File/artboard/state-machine discovery | Product-side; mostly public runtime surface. |
| FlowSession tests | `flow_session.rs:5021-7227` (2,207; 40 tests), plus `crates/nuxie/tests/flow_session_contract.rs` (335) | Exercises the public ABI, VM lifecycle, and transactional behavior | Move with FlowSession. Preserve public names through compatibility re-exports during migration. |
| Scene contract and IDs | `crates/nuxie/src/scene.rs:28-1187` (1,160) | Public runtime/binary identities and errors | Product-side, cleanly demarcatable. |
| Scene durable definitions/index/specs | `scene.rs:1188-3401` (2,214) | `nuxie_binary::AuthoringRecord`, property/value types, runtime file identities | Product-side authoring model. |
| Scene hierarchy/materialization | `scene.rs:3402-5622` (2,221) | Constructs/reads `RuntimeFile`; directly touches `materialized.file.runtime` in a handful of places even though `File::runtime()` already exists | Product-side but high coupling. Replace direct private field access with the existing public accessor before moving. |
| Silver/scene export schema | `scene.rs:5623-6819` (1,197) | Public exported-record/document DTOs; lowering later feeds `RuntimeFile::from_authoring_records` | Product-side. The schema and its generated counterpart must move together. |
| Scene store/live mount API | `scene.rs:6820-9099` (2,280) | Owns mounted `File`/artboard/view-model state and coordinates transactions | Product-side; medium-to-high coupling. |
| SceneTx and subordinate transactions | `scene.rs:9100-15589` (6,490) | `SceneTx`, `DataConverterTx`, `VmTx`, `AnimTx`, and `MachineTx` mutate the authoring graph, then materialize into runtime/binary types | Core product authoring seam. Cohesive, but atomic commit/materialization behavior makes the move high risk. |
| Scene frame queries/live mutation | `scene.rs:15590-17592` (2,003) | Hit/geometry/semantic queries and intrinsic image sizing reach crate-private `OwnedArtboardInstance` helpers | Product-side. Needs stable snapshot/command DTOs instead of exposing mutable runtime internals. |
| Scene lowering/validation/export | `scene.rs:17593-25563` (7,971) | Converts authoring records into `RuntimeFile`, validates cross-object invariants, and produces exported documents | Product-side. Highest-risk region because it couples the authoring model to port construction invariants. |
| Scene tests | `scene.rs:25564-38224` (12,661; 120 tests), plus `crates/nuxie/tests/scene_authoring.rs` (15,674) and related authoring tests | Broad transaction, materialization, export, and live-runtime contract | Move with Scene; use this suite as the seam migration oracle. |
| Scene schema generator | `crates/nuxie/build.rs` (4,531) and `scene.rs`'s `include!(concat!(env!("OUT_DIR"), "/scene_schema.rs"))` near line 860 | Generates the Scene/silver record schema at build time | Product-side and inseparable from Scene. Extraction must move the build script/generator inputs and preserve the generated include contract. |
| Script import authorization | `crates/nuxie/src/script_import.rs:1-157` production, `158-211` tests | Ed25519/container verification; calls crate-private `File::execution_authorization_for`/authorization state | Product policy. Move to `nuxie-flow`, but expose a capability-based import seam rather than making raw authorization state public. |
| Mixed façade bridge | `crates/nuxie/src/lib.rs`, principally project-envelope classification and private bridge methods around lines 55, 154, 223-258, 307, 344, 376, 1,492, 2,983, 3,328, 4,304, and 5,600-5,733 | `File::from_runtime`; flow script/listener lifecycle; apply/rollback/command drain; artboard geometry/semantic/intrinsic-image helpers | Not an independent feature. Convert these call sites into explicit narrow bridge traits/DTOs, then move their product consumers. |

The concrete crate-private methods the extraction must account for are:

- Flow lifecycle around `nuxie/src/lib.rs:5600-5675`: `prepare_flow_scripts`, `prepare_flow_listener_actions`, rehydrate/apply, begin host cycle, rollback, and drain commands.
- Scene live/runtime queries around `lib.rs:5705-5733`: hit-test path segments with bounds, geometry path segments, retained/semantic text, and intrinsic image-dimension registration.
- `File::from_runtime` around `lib.rs:4304` and the script execution authorization capability.

These are already cross-crate calls from `nuxie` into public `nuxie-runtime` types; the missing visibility is chiefly inside the `nuxie` façade. Extraction should expose a sealed product bridge from the future base façade, not expand dozens of runtime fields to `pub`.

### Project-data converter: product model inside the port

`crates/nuxie-runtime/src/project_data_converter.rs` is 2,686 lines and is explicitly classified `scene-api` in `rust-additions.toml`:

| Lines | Region | Coupling and disposition |
| ---: | --- | --- |
| 22-507 | Public values, specs, errors, and state | Pure product/protocol model; eventual neutral-crate candidate. |
| 508-1,289 | Envelope/program/catalog compilation and evaluation | Mostly `std`/`serde`; eventual neutral-crate candidate. |
| 1,290-2,324 | Validation, coercion, formatting, interpolation | Pure-looking core, but behavior is part of runtime data-bind semantics. Extract only with parity tests. |
| 2,325-2,686 | Formula parser | Mechanically separable after the outer seam is established. |

The file is locally self-contained, but its runtime integration is not. `data_bind/context/context_value.rs` has a `RuntimeDataBindGraphConverter::Project` variant and owns conversion/evaluator state; `data_bind/data_bind_context.rs` constructs it from a script-asset envelope; both runtime and `nuxie` re-export or classify its types; Scene authors and lowers them. A direct move to `nuxie-flow` would invert the desired dependency and make the port depend on the product crate.

Two crate-private numeric-policy helpers in this file (the bounded-list constant near line 467 and helper near line 2,320) are also used by port-side Number-to-List/context conversion. If the model is eventually extracted, those policies should remain in the port or move to a dependency-neutral shared crate; they should not become a product API solely to satisfy the move.

Recommendation: first demarcate this as `project_data_converter/{model, program, evaluator, bridge}` inside `nuxie-runtime`; later extract only the pure model/program/evaluator to a neutral `nuxie-project-data` crate that both runtime and `nuxie-flow` may depend on. The runtime adapter stays in `nuxie-runtime`.

### Rust-only infrastructure that should remain port-side

The following `rust-additions.toml` entries have no one-file C++ correspondence but are not product features:

| Current path/group | Lines | Why it stays in `nuxie-runtime` |
| --- | ---: | --- |
| `data_bind_container.rs`, `data_converter.rs`, `retained_data_bind.rs` | 12 total | Tiny generated/type façade modules over port functionality. |
| `input/mod.rs`, `shapes/mod.rs`, `shapes/paint/mod.rs` | 168 total | Rust namespace/generated-storage coordinators. |
| `objects.rs` | 1,018 | Runtime object registry plus `include!(concat!(env!("OUT_DIR"), "/runtime_objects.rs"))`; generated by `crates/nuxie-runtime/build.rs` (707 lines). This is port construction infrastructure. |
| `properties.rs` | 826 | Generated property interpretation/storage used throughout the port. |
| `state_machine.rs`, `state_machine/instance.rs`, `state_machine/listener_types/mod.rs` | 215 total | Rust module/re-export façade for the ported state-machine implementation. |
| `viewmodel/mod.rs`, `viewmodel/runtime/mod.rs` | 58 total | Rust namespace/re-export façade for ported view-model behavior. |
| `focus.rs` | 7 | Thin public façade over retained focus behavior, not a standalone product feature. |

These codegen/facade additions total about 2,303 source lines excluding the project converter and focus façade. They should be clearly marked as Rust port infrastructure, but moving them to `nuxie-flow` would blur rather than improve the seam.

`crates/nuxie/src/command_queue.rs`, server code, and raw-text support are not included in the Nuxie-only list: their current files are explicitly mapped to upstream C++ correspondence. A richer Rust API around a ported facility does not by itself make that facility product-side.

---

## 4. Recommended seam design

### Decision

Use a **module boundary now and a crate boundary later**.

The immediate structure should make product ownership visible without changing dependencies:

```text
crates/nuxie/src/product/
  flow_session/
  scene/
    model/
    transaction/
    live/
    export/
  script_import/

crates/nuxie-runtime/src/project_data_converter/
  model.rs
  program.rs
  evaluator.rs
  bridge.rs
```

Preserve current public paths with root re-exports. This first step is a namespace/demarcation change, not an extraction. It gives reviewers an auditable boundary while the current test and parity baseline still observes exactly the same crate graph.

The long-term crate graph should avoid a cycle:

```text
nuxie-runtime  <---  nuxie-core  <---  nuxie-flow
       ^                 ^                 ^
       |                 |                 |
       +------ nuxie-project-data --------+

nuxie (umbrella compatibility crate) re-exports nuxie-core + nuxie-flow
```

`nuxie-flow` contains FlowSession, Scene/SceneTx, silver/export DTOs and lowering, and script-import policy. `nuxie-core` contains `File`, `Factory`, owned artboard/view-model handles, renderer integration, and narrow bridge traits. `nuxie-runtime` remains the C++-correspondence port plus explicitly labeled Rust port infrastructure. A small dependency-neutral `nuxie-project-data` is optional and should be introduced only after the runtime adapter has been separated from the pure evaluator.

### Public surface of the port/base layer

The extraction should not make arbitrary runtime fields public. The required surface is narrow:

1. A sealed/trusted constructor that wraps a validated `RuntimeFile` as a façade `File`, with an explicit authoring/import authorization policy.
2. A flow-runtime bridge for prepare, rehydrate, apply, host-cycle, rollback, and command-drain operations. Its inputs/outputs should be stable value DTOs, not references to internal queues.
3. Read-only artboard snapshot operations for hit geometry, retained geometry, and semantic text, plus a command for intrinsic image dimensions. Return owned snapshots so Scene cannot retain internal graph borrows.
4. A project-converter adapter owned by `nuxie-runtime`; pure program/model types may come from the neutral crate. Do not expose the runtime bind graph or evaluator cells as product API.
5. Existing upstream-corresponding runtime types only where they are already public and semantically stable. Product re-exports should live in `nuxie`/`nuxie-flow`, not expand the port's promise.

### Migration order

1. Establish the exact green parity/golden baseline and keep it pinned in every PR.
2. Land the `artboard.rs` and `state_machine_instance.rs` mechanical splits as separate PRs. This makes later ownership changes reviewable without mixing 20,000-line source moves into semantic work.
3. Demarcate `nuxie::product::{flow_session, scene, script_import}` and the project-converter submodules in place, preserving public re-exports.
4. Define the sealed flow, authoring-file, geometry-snapshot, and script-authorization bridges; migrate existing internal callers to them while everything remains in the same crate.
5. Extract the non-product façade/runtime handles into `nuxie-core`, with `nuxie` temporarily acting as an umbrella compatibility crate.
6. Move FlowSession and script import to `nuxie-flow`; move their tests and retain compatibility re-exports.
7. Move Scene model, SceneTx, export schema/generator, lowering, live adapter, and tests together. Do not split the generated schema from `build.rs` or split transaction commit from materialization in the extraction PR.
8. Isolate the project converter's pure model/program/evaluator. Extract it to a neutral crate only if doing so leaves the runtime adapter dependency pointing inward, never from `nuxie-runtime` to `nuxie-flow`.
9. Remove compatibility re-exports and tighten visibility in a later breaking-change PR, after downstream migration.

### Risk map

| Risk | Areas | Reason |
| --- | --- | --- |
| Mechanical/low | Thin namespace façades, codegen coordinators staying in place, test-module extraction, pure FlowSession validation/types | Mostly physical ownership with no runtime mutation semantics. |
| Moderate | FlowSession after bridge creation, script import after capability API, Scene DTO/export types | Public API and authorization compatibility, but limited graph mutation. |
| High | SceneTx commit/materialization, live Scene mounting, geometry/semantic access, Scene lowering and generated schema | Atomicity, identity, generated code, and live runtime invariants cross the seam. |
| High | Project-data converter extraction | The source looks pure, but evaluator state is embedded in runtime data-bind context and Number-to-List policy. |
| High | Script/listener ownership during FlowSession moves | Rehydration, rollback, command queues, and VM object lifetime must remain one coherent transaction. |

### Why baseline-first matters

`docs/parity-gap-register.md` explicitly recommends making the parity claim trustworthy before feature work and landing each subsequent change against the oracle (the “Recommended execution order” around lines 204-211). It also describes Scene as a Nuxie-only additive API (around lines 126-127) while recording FlowSession/capability-boundary gaps elsewhere in the register. That is the governing principle here: first make source moves mechanically observable by the existing correspondence, ownership, trace, golden, and silver batteries; then introduce the product seam behind those same oracles. A green build alone is insufficient because it can preserve compilation while silently weakening ownership scans or changing what “zero divergences” measures.

The two large-file splits are therefore prerequisites for, not part of, the product extraction. Each split keeps the logical Rust module unchanged and updates only physical-address consumers. The seam work then proceeds through explicit bridges and compatibility re-exports, with risky transaction/materialization changes isolated from mechanical moves.

# nuxie-runtime structure report

## Scope and baseline

This is a read-only structure analysis. It proposes no behavior, source, parity-ledger, or ownership-ledger changes.

The inspected worktree was already detached at `e8726db8ffc689d3b19f1c0a55794aa6daf9956d` (`Merge pull request #246 from TheOneLevin/feat/b6-0058-listener-viewmodel-change`, 2026-08-04). The local `origin/main` ref resolves to the same commit. The requested `git fetch origin` and redundant `git checkout --detach origin/main` could not update the shared worktree metadata because the sandbox cannot write the parent repository's `.git/worktrees/nuxie-mr-c12` directory. The analysis is therefore pinned to the locally available `origin/main` commit above, not a newly fetched remote ref. The pre-existing untracked `vtriage-capture/` directory was left untouched.

The two target files have grown beyond the sizes in the request:

| File | Current lines | Production/support | Tests |
| --- | ---: | ---: | ---: |
| `crates/nuxie-runtime/src/artboard.rs` | 23,374 | 12,039 | 11,335 (`205` tests) |
| `crates/nuxie-runtime/src/state_machine/state_machine_instance.rs` | 21,674 | 14,079 | 7,595 (`94` tests) |

The safest split mechanism is literal `include!` fragments, not Rust child modules. An included fragment remains in the original semantic module (`crate::artboard` or `crate::state_machine::state_machine_instance`), so private-name resolution, inherent method paths, sibling access, and demangled symbols remain unchanged. A `mod build;`/`#[path = ...] mod build;` split would introduce new privacy boundaries and make a supposedly mechanical PR into an API refactor.

---

## 1. Large-file split plan: `artboard.rs`

### Current structural map

| Lines | Region |
| ---: | --- |
| 1-106 | Imports and supporting aliases |
| 107-135 | Draw-frame counter and generated `Mat2D` helper |
| 136-160 | `ExternalFontAssetError` |
| 161-181 | `RuntimeArtboardInstanceIdentity` |
| 182-231 | `RuntimeScriptState` and implementations |
| 232-250 | Advancing-component and persistent-dirt fixtures |
| 251-298 | Script advance/update mode enums and implementations |
| 299-327 | Resetting/runtime-component helper types |
| 328-334 | `RuntimeTextStyleFeatureOption` |
| 335-523 | `ArtboardInstance` storage |
| 524-714 | `Clone for ArtboardInstance` |
| 715-739 | `Drop for ArtboardInstance` followed by the existing leaf `include!` declarations |
| 740-773 | Occurrence and frame-advance report types |
| 774-844 | Build context/index/text-flag helpers |
| 845-860 | `RuntimeNestedAnimationInstance` |
| 861-11,133 | Main `impl ArtboardInstance` |
| 11,134-11,147 | Focus-scroll path and focus-bounds transform helper |
| 11,148-11,187 | Small follow-up `impl ArtboardInstance` |
| 11,188-11,226 | Component-list reset helper |
| 11,227-11,595 | `impl RuntimeNestedArtboardInstance` |
| 11,596-11,611 | `impl RuntimeNestedAnimationInstance` |
| 11,612-12,039 | Free construction/nested-artboard helpers |
| 12,040-23,374 | Single `#[cfg(test)] mod tests` (`205` tests) |

The main implementation itself divides along stable responsibility boundaries:

| Lines | Logical responsibility |
| ---: | --- |
| 861-1,104 | Lifecycle, data context, and transient-clone restoration |
| 1,105-2,440 | Build occurrence relationships and constructors |
| 2,441-2,712 | Path/hit initialization and external assets |
| 2,713-3,399 | Scripting lifecycle |
| 3,400-3,989 | Object/component/audio/hit/graph/definition access |
| 3,990-4,152 | Scripted interpolators and state-machine definition selection |
| 4,153-4,480 | Dimensions, world/layout bounds, scrolling, and focus reveal |
| 4,481-5,229 | Generated property façade and text values |
| 5,230-5,568 | Animation/state-machine/view-model occurrences |
| 5,569-6,312 | Component-list lifecycle and layout |
| 6,313-7,408 | Root/nested frame advance and retained-component dispatch |
| 7,409-7,585 | Nested-layout frame propagation |
| 7,586-8,496 | Transforms, epochs, dirt, and invalidation |
| 8,497-9,191 | Update pass and five-pass state-machine settlement |
| 9,192-9,825 | Component update dispatch and script scheduling |
| 9,826-11,133 | Property-change callbacks, nested controls, and collapse |

### Proposed physical layout

Keep `artboard.rs` as the owner. It retains imports, supporting types, `ArtboardInstance`, `Clone`, `Drop`, the existing leaf includes, occurrence/report/build helper types, and an ordered series of literal includes. This preserves declaration order where it matters and makes all fragments part of `crate::artboard`.

```text
src/
  artboard.rs                         owner/types, existing leaf includes, ordered include! list
  artboard/
    build.rs                          current 861-2712
    script_runtime.rs                 current 2713-3399
    access.rs                         current 3400-3989
    definitions_and_layout.rs         current 3990-4480
    properties.rs                     current 4481-5568
    component_lists.rs                current 5569-6312
    advance.rs                        current 6313-7585
    dirt.rs                           current 7586-8496
    update.rs                         current 8497-9825
    callbacks.rs                      current 9826-11133
    nested.rs                         current 11134-12039
    tests.rs                          body of current tests module
```

The production fragments are intentionally in execution/dependency order rather than alphabetical order. `tests.rs` should be included inside the original test module:

```rust
#[cfg(test)]
mod tests {
    include!("artboard/tests.rs");
}
```

This retains the test module's name and access. It also gives the attribution checker a deliberately exempt `tests.rs` leaf.

### Lockstep correspondence and checker manifest

The following table is the required per-file edit manifest. “B6 rows” are `[[files]]` entries in `file-correspondence-manifest.toml`; add the fragment's path to those rows' semicolon-separated `rust_module` values and update their C17 retained-module notes. Keep status, upstream pins, C++ paths, verification text, and correspondence claims unchanged. The row list is conservative: a later provenance cleanup can narrow it, but narrowing must not be mixed into this mechanical split.

| New file | B6 rows that must name it | Frame-loop/checker bindings that must move with it |
| --- | --- | --- |
| `artboard/build.rs` | `B6-0001`, `B6-0094`, `B6-0115`, `B6-0117`, `B6-0123`, `B6-0125`, `B6-0129`, `B6-0131`, `B6-0142`, `B6-0146`, `B6-0202`, `B6-0258`, `B6-0303`, `B6-0304`, `B6-0305`, `B6-0331`, `B6-0361`, `B6-0362`, `B6-0365`, `B6-0368`, `B6-0385`, `B6-0388`, `B6-0399`, `B6-0405`, `B6-0408` | Repoint `summarize_trace.py` dependency-build and IK-chain-build source scans when their anchors land here. Repoint matching `runtime-frame-loop-gaps.toml` owner-boundary entries. |
| `artboard/script_runtime.rs` | `B6-0077`, `B6-0194`, `B6-0200`, `B6-0322`, `B6-0326` | Repoint the script-lifecycle citations in `runtime-frame-loop-ownership.toml` and any corresponding synthetic test fixture paths. |
| `artboard/access.rs` | `B6-0094`, `B6-0095`, `B6-0123`, `B6-0146`, `B6-0203`, `B6-0204`, `B6-0205` | Repoint any live-draw/renderer-interface lifecycle citations whose named methods move here. |
| `artboard/definitions_and_layout.rs` | `B6-0077`, `B6-0094`, `B6-0258`, `B6-0303`, `B6-0304`, `B6-0305`, `B6-0385`, `B6-0388` | Repoint focus-retained-tree and layout lifecycle citations, plus matching owner-boundary allow entries. |
| `artboard/properties.rs` | `B6-0067`, `B6-0077`, `B6-0094`, `B6-0194`, `B6-0200`, `B6-0352`, `B6-0355`, `B6-0385`, `B6-0388`, `B6-0399`, `B6-0404`, `B6-0405`, `B6-0408` | `B6-0067` currently has only one Rust module: **replace** `artboard.rs` with this fragment rather than adding a second path. Repoint property/text lifecycle citations and corresponding owner-boundary entries. |
| `artboard/component_lists.rs` | `B6-0095`, `B6-0258`, `B6-0303`, `B6-0304`, `B6-0305` | Repoint component-list and nested-layout lifecycle citations and matching test fixtures. |
| `artboard/advance.rs` | `B6-0001`, `B6-0059`, `B6-0061`, `B6-0063`, `B6-0064`, `B6-0077`, `B6-0094`, `B6-0095`, `B6-0194`, `B6-0200`, `B6-0258`, `B6-0303`, `B6-0304`, `B6-0305`, `B6-0322`, `B6-0326`, `B6-0388` | In `runtime-frame-loop-ownership.toml`, move `artboard.advance`'s `rust_file` and the lifecycle citations for artboard/nested advance. Repoint `summarize_trace.py` advancing/resetting dispatch scans as applicable. Repoint relevant gap allow rows and `test_check.py` advance fixtures. |
| `artboard/dirt.rs` | `B6-0094`, `B6-0123`, `B6-0258`, `B6-0331`, `B6-0352`, `B6-0355`, `B6-0357`, `B6-0361`, `B6-0362`, `B6-0365`, `B6-0368`, `B6-0385`, `B6-0388`, `B6-0399`, `B6-0405`, `B6-0408` | Move `component.dirt`, `component.dependents` (`pub fn add_dirt(`), and transform/dirt lifecycle citations as appropriate. Repoint `summarize_trace.py`'s two add-dirt anchors, dirt consumptions, owner resolutions, and skin-buffer scans according to their final physical locations. Repoint matching gap rows. |
| `artboard/update.rs` | `B6-0001`, `B6-0094`, `B6-0115`, `B6-0117`, `B6-0123`, `B6-0125`, `B6-0129`, `B6-0131`, `B6-0142`, `B6-0194`, `B6-0200`, `B6-0203`, `B6-0204`, `B6-0205`, `B6-0258`, `B6-0303`, `B6-0304`, `B6-0305`, `B6-0322`, `B6-0326`, `B6-0331`, `B6-0352`, `B6-0355`, `B6-0357`, `B6-0361`, `B6-0362`, `B6-0365`, `B6-0368`, `B6-0385`, `B6-0388`, `B6-0399`, `B6-0404`, `B6-0405`, `B6-0408` | Move `component.update_order` (`fn update_components_with_hook_recording`) and `artboard.update_pass` (`pub fn update_pass(`) `rust_file` values and lifecycle citations. Repoint retained update/settlement gap rows, trace scans, and test fixtures. |
| `artboard/callbacks.rs` | `B6-0059`, `B6-0061`, `B6-0063`, `B6-0064`, `B6-0077`, `B6-0094`, `B6-0123`, `B6-0125`, `B6-0129`, `B6-0131`, `B6-0142`, `B6-0194`, `B6-0200`, `B6-0258`, `B6-0303`, `B6-0304`, `B6-0305`, `B6-0331`, `B6-0352`, `B6-0355`, `B6-0357`, `B6-0361`, `B6-0362`, `B6-0365`, `B6-0368`, `B6-0385`, `B6-0388`, `B6-0399`, `B6-0404`, `B6-0405`, `B6-0408` | Change `check_layout_style_handlers.py`'s `RUST_ARTBOARD_MODULE` to this fragment if both display-change markers remain here; otherwise make the checker accept the two explicit physical files. Repoint callback lifecycle citations, gap rows, and fixtures. |
| `artboard/nested.rs` | `B6-0059`, `B6-0061`, `B6-0063`, `B6-0064`, `B6-0077`, `B6-0094`, `B6-0095`, `B6-0258`, `B6-0303`, `B6-0304`, `B6-0305` | Repoint nested-artboard advance/layout citations, gap allow entries, and nested fixtures. |
| `artboard/tests.rs` | None. `rust_attribution.py` excludes a source whose stem is `tests`. | Move the artboard test body unchanged. Update the seven relative `include_bytes!`/fixture paths currently near old lines 19,951, 20,001, 20,168, 20,542, 20,597, 20,612, and 23,311 because the file becomes one directory deeper. Prefer `concat!(env!("CARGO_MANIFEST_DIR"), "/...")` for fixture stability. The `env!` use near old line 14,290 remains valid. |

#### Global artboard bindings

These bindings span more than one fragment and therefore belong in the same PR:

- `rust_attribution.py` discovers all production `.rs` files below `crates/nuxie-runtime/src`. Every new production fragment must be listed in at least one `file-correspondence-manifest.toml` row and must **not** be added to `rust-additions.toml`. The checker exempts `tests.rs`, names ending in `_tests`, and files below a `tests/` path.
- The manifest's scatter ratchet is already at its maximum: 155 rows currently have multiple Rust modules. Except for `B6-0067`, all affected artboard rows are already multi-module. Replacing the `B6-0067` root path prevents the split from increasing that count.
- Update every affected C17 note that enumerates retained Rust modules. Do not change the underlying parity classification merely because the code's physical address changed.
- `docs/runtime-frame-loop-ownership.toml` has artboard citations in `focus.retained_tree`, `component.identity`, `component.dirt`, `component.dependents`, `component.update_order`, `component.transforms`, `component.clone_drop`, `artboard.advance`, `artboard.update_pass`, `nested_artboard.advance`, `artboard.live_draw`, and `renderer.interface_boundary`. Keep `component.clone_drop` and its `pub struct ArtboardInstance` anchor on the root; change each other `rust_file` or `rust:path:line-range` to its physical fragment.
- `docs/runtime-frame-loop-gaps.toml` has 25 owner-boundary allow entries pointing at `artboard.rs` (entries 0-7, 11-22, and 29-34 in current order). Repoint each to the fragment that contains its anchor. A literal move should preserve its token-based `site_hash`; a changed hash is a signal that the split was not mechanical.
- `tools/runtime-frame-loop-port/summarize_trace.py` hard-codes `artboard.rs` nine times for add-dirt, dirt consumption, owner resolution, dependency build, skin rebuild, advancing dispatch, resetting dispatch, and IK-chain discovery. Give each query its actual fragment path. The demangled owner should remain `artboard::ArtboardInstance` because `include!` does not introduce a child module.
- `tools/runtime-frame-loop-port/check_layout_style_handlers.py` hard-codes the artboard source while looking for `propagate_layout_component_display_changed` and the display property key. Point it to the fragment(s) containing those markers.
- `tools/runtime-frame-loop-port/test_check.py` contains at least 14 exact artboard paths covering probe gating, layer occurrence, live-advance ratchets, registry site hashes/substitution, and trait-anchor behavior. Update each synthetic fixture to the fragment that owns the tested anchor; do not perform a blind string replacement.
- Leave the existing root-level includes (`nested_artboard.rs`, `artboard_component_list.rs`, `artboard_list_map_rule.rs`, `artboard_referencer.rs`, `bindable_artboard.rs`, `nested_artboard_layout.rs`, `nested_artboard_leaf.rs`, `component_origin.rs`, bone/weight, profiler/profile) where they are. Moving them under `artboard/` changes relative include resolution and correspondence ownership.
- Keep `crates/nuxie-runtime/src/lib.rs`'s `mod artboard;` unchanged. The public and crate-private paths must not acquire an extra `artboard::<fragment>` segment.

### Mechanical PR sequence and proof

One PR should contain only this file's split and the path/anchor bookkeeping above. Recommended commit sequence within that PR:

1. Extract `tests.rs`, fix only its fixture paths, and prove the same 205 tests are collected.
2. Extract production regions in original order using `include!`; make no renames, visibility edits, formatting sweeps, or logic edits.
3. Update correspondence C17/path fields and all frame-loop physical anchors.
4. Update checker source locations and synthetic fixtures, including an assertion that demangled ownership remains `artboard::ArtboardInstance`.

Proof battery:

```text
make rust-sources-fresh
make cpp-probe
make runtime-frame-loop-port-test
make runtime-frame-loop-port-check
make rust-attribution-check
cargo check --workspace
cargo test -p nuxie-runtime
cargo test -p nuxie --features scripting
make scripted-golden-compare
make silver-corpus-test
```

The PR is structure-preserving only if parity row counts/statuses, trace landmarks, owner-boundary hashes, test counts, and golden/silver outputs are unchanged.

---

## 2. Large-file split plan: `state_machine_instance.rs`

### Current structural map

| Lines | Region |
| ---: | --- |
| 1-93 | Imports and ownership commentary |
| 94-255 | Nested event-chain/notify phases, trace structures, and trace recording |
| 256-312 | Semantic-node resolver and audio/event seam types |
| 313-1,271 | `RuntimeHitResult`, `HitComponent`, and hierarchy implementations |
| 1,272-1,467 | Nested notifier, focus, semantic, and state helper types |
| 1,468-1,474 | `RuntimeDataContextBindError` |
| 1,475-1,711 | `StateMachineInstance` storage |
| 1,712-1,744 | Data-bind occurrence/pointer/executor structures |
| 1,745-2,147 | Listener-action executor implementations |
| 2,148-2,335 | `Clone for StateMachineInstance` |
| 2,336-2,356 | `Drop for StateMachineInstance` |
| 2,357-2,638 | Free helpers and view-model listener types |
| 2,639-13,938 | Main `impl StateMachineInstance` |
| 13,939-13,953 | `runtime_owned_font` helper |
| 13,954-14,079 | `view_model_listener_tests` (`1` test) |
| 14,080-21,674 | `scripted_listener_action_tests` (`93` tests) |

The main implementation divides as follows:

| Lines | Logical responsibility |
| ---: | --- |
| 2,639-2,932 | Teardown, enrollment, focus manager, clone rebind |
| 2,933-3,741 | Construction and listener/layer initialization |
| 3,742-5,102 | Scripting, hydration, and adoption |
| 5,103-6,007 | State/input/focus/semantic/keyboard/gamepad operations |
| 6,008-7,487 | Hit/listener/pointer pipeline |
| 7,488-8,525 | Event notification, deferred callbacks, listener actions |
| 8,526-9,100 | Direct bind targets and value readers |
| 9,101-10,748 | Default/imported/owned source setters and relinking |
| 10,749-12,599 | Data-context bind/rebind, listener cells, bind updates |
| 12,600-13,938 | Reports, advance/apply, nested event dispatch, settlement |

### Proposed physical layout

```text
src/state_machine/
  state_machine_instance.rs               owner/types, hit hierarchy, lifecycle, ordered include! list
  state_machine_instance/
    construction.rs                       current 2639-3741
    scripted_objects.rs                   current 3742-5102
    input_focus.rs                        current 5103-6007
    pointer.rs                            current 6008-7487
    events_actions.rs                     current 7488-8525
    bind_targets.rs                       current 8526-9100
    bind_sources.rs                       current 9101-10748
    data_context.rs                       current 10749-12599
    advance.rs                            current 12600-13953
    tests/
      view_model_listener.rs              body of current first test module
      scripted_listener_actions.rs         body of current second test module
```

Retain the two existing module names in the root and include only their bodies. This preserves test paths and any name-sensitive filtering:

```rust
#[cfg(test)]
mod view_model_listener_tests {
    include!("state_machine_instance/tests/view_model_listener.rs");
}
```

Use the same pattern for `scripted_listener_action_tests`.

### Lockstep correspondence and checker manifest

| New file | B6 rows that must name it | Frame-loop/checker bindings that must move with it |
| --- | --- | --- |
| `state_machine_instance/construction.rs` | `B6-0077`, `B6-0194`, `B6-0195`, `B6-0196`, `B6-0200`, `B6-0310` | Repoint construction/enrollment lifecycle citations and matching synthetic fixtures. Keep `state_machine.collections`' `pub struct StateMachineInstance` anchor on the root. |
| `state_machine_instance/scripted_objects.rs` | `B6-0077`, `B6-0175`, `B6-0194`, `B6-0195`, `B6-0196`, `B6-0200` | Repoint scripting/hydration lifecycle citations and scripted-action fixture paths. |
| `state_machine_instance/input_focus.rs` | `B6-0077`, `B6-0083` | Repoint focus/semantic/keyboard/gamepad citations and tests. |
| `state_machine_instance/pointer.rs` | `B6-0077`, `B6-0083` | Repoint hit/pointer/listener lifecycle citations and FL-C4/layer/hit fixtures. |
| `state_machine_instance/events_actions.rs` | `B6-0058`, `B6-0077`, `B6-0440` | Repoint event reporting, firing/bubbling, and action-dispatch citations/fixtures. `state_machine.actions`' primary anchor remains on the root if the listener-action executor stays there. |
| `state_machine_instance/bind_targets.rs` | `B6-0058`, `B6-0194`, `B6-0196`, `B6-0200`, `B6-0440` | Repoint bind-target and view-model listener citations/fixtures. |
| `state_machine_instance/bind_sources.rs` | `B6-0058`, `B6-0175`, `B6-0194`, `B6-0195`, `B6-0196`, `B6-0200`, `B6-0440` | Repoint source/relink/data-converter citations and bind fixtures. |
| `state_machine_instance/data_context.rs` | `B6-0058`, `B6-0077`, `B6-0194`, `B6-0195`, `B6-0196`, `B6-0200`, `B6-0440` | Repoint data-context lifecycle citations and listener-cell/bind fixtures. |
| `state_machine_instance/advance.rs` | `B6-0077`, `B6-0194`, `B6-0195`, `B6-0196`, `B6-0200`, `B6-0310`, `B6-0440` | Move `state_machine.advance`'s `rust_file` and `advance_with_report_mode` anchor here. Repoint state-machine event/action/reporting lifecycle citations and advance/keyframe fixtures. Include `runtime_owned_font` in this fragment to avoid an artificial one-function file. |
| `state_machine_instance/tests/view_model_listener.rs` | None; it resides below a `tests/` path. | Move the body unchanged and preserve the outer module name. |
| `state_machine_instance/tests/scripted_listener_actions.rs` | None; it resides below a `tests/` path. | Move the body unchanged. Replace or correct the relative fixture path near old line 19,011 (`../../../../fixtures/sync/vm_listener_fire_event.riv`) because the source becomes two directories deeper; prefer a `CARGO_MANIFEST_DIR`-anchored path. |

#### Global state-machine bindings

- All production fragments must be named by `file-correspondence-manifest.toml`, never `rust-additions.toml`. Every affected SMI row is already multi-module, so this split does not increase the manifest's 155-row scatter count.
- Update C17 retained-module notes for `B6-0058`, `B6-0077`, `B6-0083`, `B6-0175`, `B6-0194`, `B6-0195`, `B6-0196`, `B6-0200`, `B6-0310`, and `B6-0440`; leave the substantive parity assertions untouched.
- `docs/runtime-frame-loop-ownership.toml` cites this file from `focus.retained_tree`, `state_machine.collections`, `state_machine.advance`, `state_machine.events`, `state_machine.actions`, and `event.reporting`. Keep root anchors for the struct, stored event collection, and action executor. Repoint method/lifecycle citations to physical fragments.
- `tools/runtime-frame-loop-port/check.py` currently treats one hard-coded SMI file as the nested-event owner and excludes it from outside-owner scans. Once methods are in included fragments, a path-only exclusion would falsely flag the fragments. Make owner discovery include-aware: recursively expand literal local `include!("...")` files in source order for export analysis, and treat every expanded file as the same logical owner. Do not broaden the exclusion to a directory glob.
- Add checker unit coverage for the include-aware logical owner: an exported nested-event method in an included fragment must be accepted, while the same method in an unrelated file must still fail. Preserve syntax parsing and duplicate-anchor detection.
- `tools/runtime-frame-loop-port/test_check.py` has at least 14 exact SMI references, plus constructed/split path strings, covering FL-C4, lifecycle, layer, hit, bind, events, firing/bubbling, advance, owner exports, keyframes, focus, and semantic behavior. Point each fixture to the actual fragment; do not simply substitute the root path everywhere.
- `docs/runtime-frame-loop-gaps.toml` has no current SMI owner-boundary allow row to migrate.
- Keep `crates/nuxie-runtime/src/state_machine.rs`'s `pub(crate) mod state_machine_instance;` and its re-exports unchanged. Literal includes keep all symbols at the original module path.

### Mechanical PR sequence and proof

This should be a second, independent PR after the artboard split has landed. It uses the same proof battery. The checker must first learn the logical-owner/include relationship in the same PR, because moving the methods without that update creates false ownership failures. Apart from that narrowly required path-awareness, no checker policy should change.

Success means the same 94 tests are collected, no exported API path changes, the correspondence scatter count stays at 155, no new owner-boundary allowance appears, and the full battery shown in the artboard plan remains green.

---

## 3. Nuxie-only seam inventory

### Classification method

The authoritative starting point is correspondence metadata:

- Production Rust paths mapped to upstream C++ in `file-correspondence-manifest.toml` are port-side unless a smaller product-only region can be proven.
- Production Rust paths listed in `rust-additions.toml` are intentional Rust-only additions. The relevant ownership labels are `flowsession-abi`, `scene-api`, and generated/codegen infrastructure.
- `build.rs` is outside the attribution scanner's `src/` roots, so it must be inspected manually.
- A convenience façade is not automatically product-side. Thin Rust-only namespace/generated-storage coordinators that serve the port stay with the port even though no one C++ file corresponds to them.

This prevents two opposite mistakes: burying product authoring features inside the port forever, and extracting Rust implementation infrastructure merely because its file has no one-to-one C++ source.

### Product features and mixed glue

| Region | Current size and location | Runtime internals reached / extraction surface | Placement assessment |
| --- | --- | --- | --- |
| FlowSession wire/domain model | `crates/nuxie/src/flow_session.rs:21-621` (601 lines) | Public `File`, `Factory`, renderer, artboard, view-model, and state-machine types | Clean product vocabulary; move to `nuxie-flow` after dependency direction is fixed. |
| FlowSession orchestration | `flow_session.rs:622-2163` (1,542) | Calls crate-private script preparation, listener preparation, rehydration, apply/rollback, host-cycle, and command-drain methods on the `nuxie` façade | Product-side, but requires a narrow public/sealed flow-runtime bridge before crate extraction. Moderate coupling. |
| FlowSession value snapshots | `flow_session.rs:2164-3105` (942) | Traverses public runtime-owned view-model/value objects and builds its own graph/arena | Product-side and cohesive. Mostly mechanical after bridge creation. |
| FlowSession validation/accounting | `flow_session.rs:3106-3545` (440) | Product protocol limits and payload validation | Clean extraction candidate. |
| FlowSession mutation/apply/diff | `flow_session.rs:3546-4515` (970) | Applies mutations through runtime view-model/state-machine handles; shares rollback/command sequencing with façade-private methods | Product-side; transactional sequencing makes it medium risk. |
| FlowSession selection/catalog | `flow_session.rs:4516-5020` (505) | File/artboard/state-machine discovery | Product-side; mostly public runtime surface. |
| FlowSession tests | `flow_session.rs:5021-7227` (2,207; 40 tests), plus `crates/nuxie/tests/flow_session_contract.rs` (335) | Exercises the public ABI, VM lifecycle, and transactional behavior | Move with FlowSession. Preserve public names through compatibility re-exports during migration. |
| Scene contract and IDs | `crates/nuxie/src/scene.rs:28-1187` (1,160) | Public runtime/binary identities and errors | Product-side, cleanly demarcatable. |
| Scene durable definitions/index/specs | `scene.rs:1188-3401` (2,214) | `nuxie_binary::AuthoringRecord`, property/value types, runtime file identities | Product-side authoring model. |
| Scene hierarchy/materialization | `scene.rs:3402-5622` (2,221) | Constructs/reads `RuntimeFile`; directly touches `materialized.file.runtime` in a handful of places even though `File::runtime()` already exists | Product-side but high coupling. Replace direct private field access with the existing public accessor before moving. |
| Silver/scene export schema | `scene.rs:5623-6819` (1,197) | Public exported-record/document DTOs; lowering later feeds `RuntimeFile::from_authoring_records` | Product-side. The schema and its generated counterpart must move together. |
| Scene store/live mount API | `scene.rs:6820-9099` (2,280) | Owns mounted `File`/artboard/view-model state and coordinates transactions | Product-side; medium-to-high coupling. |
| SceneTx and subordinate transactions | `scene.rs:9100-15589` (6,490) | `SceneTx`, `DataConverterTx`, `VmTx`, `AnimTx`, and `MachineTx` mutate the authoring graph, then materialize into runtime/binary types | Core product authoring seam. Cohesive, but atomic commit/materialization behavior makes the move high risk. |
| Scene frame queries/live mutation | `scene.rs:15590-17592` (2,003) | Hit/geometry/semantic queries and intrinsic image sizing reach crate-private `OwnedArtboardInstance` helpers | Product-side. Needs stable snapshot/command DTOs instead of exposing mutable runtime internals. |
| Scene lowering/validation/export | `scene.rs:17593-25563` (7,971) | Converts authoring records into `RuntimeFile`, validates cross-object invariants, and produces exported documents | Product-side. Highest-risk region because it couples the authoring model to port construction invariants. |
| Scene tests | `scene.rs:25564-38224` (12,661; 120 tests), plus `crates/nuxie/tests/scene_authoring.rs` (15,674) and related authoring tests | Broad transaction, materialization, export, and live-runtime contract | Move with Scene; use this suite as the seam migration oracle. |
| Scene schema generator | `crates/nuxie/build.rs` (4,531) and `scene.rs`'s `include!(concat!(env!("OUT_DIR"), "/scene_schema.rs"))` near line 860 | Generates the Scene/silver record schema at build time | Product-side and inseparable from Scene. Extraction must move the build script/generator inputs and preserve the generated include contract. |
| Script import authorization | `crates/nuxie/src/script_import.rs:1-157` production, `158-211` tests | Ed25519/container verification; calls crate-private `File::execution_authorization_for`/authorization state | Product policy. Move to `nuxie-flow`, but expose a capability-based import seam rather than making raw authorization state public. |
| Mixed façade bridge | `crates/nuxie/src/lib.rs`, principally project-envelope classification and private bridge methods around lines 55, 154, 223-258, 307, 344, 376, 1,492, 2,983, 3,328, 4,304, and 5,600-5,733 | `File::from_runtime`; flow script/listener lifecycle; apply/rollback/command drain; artboard geometry/semantic/intrinsic-image helpers | Not an independent feature. Convert these call sites into explicit narrow bridge traits/DTOs, then move their product consumers. |

The concrete crate-private methods the extraction must account for are:

- Flow lifecycle around `nuxie/src/lib.rs:5600-5675`: `prepare_flow_scripts`, `prepare_flow_listener_actions`, rehydrate/apply, begin host cycle, rollback, and drain commands.
- Scene live/runtime queries around `lib.rs:5705-5733`: hit-test path segments with bounds, geometry path segments, retained/semantic text, and intrinsic image-dimension registration.
- `File::from_runtime` around `lib.rs:4304` and the script execution authorization capability.

These are already cross-crate calls from `nuxie` into public `nuxie-runtime` types; the missing visibility is chiefly inside the `nuxie` façade. Extraction should expose a sealed product bridge from the future base façade, not expand dozens of runtime fields to `pub`.

### Project-data converter: product model inside the port

`crates/nuxie-runtime/src/project_data_converter.rs` is 2,686 lines and is explicitly classified `scene-api` in `rust-additions.toml`:

| Lines | Region | Coupling and disposition |
| ---: | --- | --- |
| 22-507 | Public values, specs, errors, and state | Pure product/protocol model; eventual neutral-crate candidate. |
| 508-1,289 | Envelope/program/catalog compilation and evaluation | Mostly `std`/`serde`; eventual neutral-crate candidate. |
| 1,290-2,324 | Validation, coercion, formatting, interpolation | Pure-looking core, but behavior is part of runtime data-bind semantics. Extract only with parity tests. |
| 2,325-2,686 | Formula parser | Mechanically separable after the outer seam is established. |

The file is locally self-contained, but its runtime integration is not. `data_bind/context/context_value.rs` has a `RuntimeDataBindGraphConverter::Project` variant and owns conversion/evaluator state; `data_bind/data_bind_context.rs` constructs it from a script-asset envelope; both runtime and `nuxie` re-export or classify its types; Scene authors and lowers them. A direct move to `nuxie-flow` would invert the desired dependency and make the port depend on the product crate.

Two crate-private numeric-policy helpers in this file (the bounded-list constant near line 467 and helper near line 2,320) are also used by port-side Number-to-List/context conversion. If the model is eventually extracted, those policies should remain in the port or move to a dependency-neutral shared crate; they should not become a product API solely to satisfy the move.

Recommendation: first demarcate this as `project_data_converter/{model, program, evaluator, bridge}` inside `nuxie-runtime`; later extract only the pure model/program/evaluator to a neutral `nuxie-project-data` crate that both runtime and `nuxie-flow` may depend on. The runtime adapter stays in `nuxie-runtime`.

### Rust-only infrastructure that should remain port-side

The following `rust-additions.toml` entries have no one-file C++ correspondence but are not product features:

| Current path/group | Lines | Why it stays in `nuxie-runtime` |
| --- | ---: | --- |
| `data_bind_container.rs`, `data_converter.rs`, `retained_data_bind.rs` | 12 total | Tiny generated/type façade modules over port functionality. |
| `input/mod.rs`, `shapes/mod.rs`, `shapes/paint/mod.rs` | 168 total | Rust namespace/generated-storage coordinators. |
| `objects.rs` | 1,018 | Runtime object registry plus `include!(concat!(env!("OUT_DIR"), "/runtime_objects.rs"))`; generated by `crates/nuxie-runtime/build.rs` (707 lines). This is port construction infrastructure. |
| `properties.rs` | 826 | Generated property interpretation/storage used throughout the port. |
| `state_machine.rs`, `state_machine/instance.rs`, `state_machine/listener_types/mod.rs` | 215 total | Rust module/re-export façade for the ported state-machine implementation. |
| `viewmodel/mod.rs`, `viewmodel/runtime/mod.rs` | 58 total | Rust namespace/re-export façade for ported view-model behavior. |
| `focus.rs` | 7 | Thin public façade over retained focus behavior, not a standalone product feature. |

These codegen/facade additions total about 2,303 source lines excluding the project converter and focus façade. They should be clearly marked as Rust port infrastructure, but moving them to `nuxie-flow` would blur rather than improve the seam.

`crates/nuxie/src/command_queue.rs`, server code, and raw-text support are not included in the Nuxie-only list: their current files are explicitly mapped to upstream C++ correspondence. A richer Rust API around a ported facility does not by itself make that facility product-side.

---

## 4. Recommended seam design

### Decision

Use a **module boundary now and a crate boundary later**.

The immediate structure should make product ownership visible without changing dependencies:

```text
crates/nuxie/src/product/
  flow_session/
  scene/
    model/
    transaction/
    live/
    export/
  script_import/

crates/nuxie-runtime/src/project_data_converter/
  model.rs
  program.rs
  evaluator.rs
  bridge.rs
```

Preserve current public paths with root re-exports. This first step is a namespace/demarcation change, not an extraction. It gives reviewers an auditable boundary while the current test and parity baseline still observes exactly the same crate graph.

The long-term crate graph should avoid a cycle:

```text
nuxie-runtime  <---  nuxie-core  <---  nuxie-flow
       ^                 ^                 ^
       |                 |                 |
       +------ nuxie-project-data --------+

nuxie (umbrella compatibility crate) re-exports nuxie-core + nuxie-flow
```

`nuxie-flow` contains FlowSession, Scene/SceneTx, silver/export DTOs and lowering, and script-import policy. `nuxie-core` contains `File`, `Factory`, owned artboard/view-model handles, renderer integration, and narrow bridge traits. `nuxie-runtime` remains the C++-correspondence port plus explicitly labeled Rust port infrastructure. A small dependency-neutral `nuxie-project-data` is optional and should be introduced only after the runtime adapter has been separated from the pure evaluator.

### Public surface of the port/base layer

The extraction should not make arbitrary runtime fields public. The required surface is narrow:

1. A sealed/trusted constructor that wraps a validated `RuntimeFile` as a façade `File`, with an explicit authoring/import authorization policy.
2. A flow-runtime bridge for prepare, rehydrate, apply, host-cycle, rollback, and command-drain operations. Its inputs/outputs should be stable value DTOs, not references to internal queues.
3. Read-only artboard snapshot operations for hit geometry, retained geometry, and semantic text, plus a command for intrinsic image dimensions. Return owned snapshots so Scene cannot retain internal graph borrows.
4. A project-converter adapter owned by `nuxie-runtime`; pure program/model types may come from the neutral crate. Do not expose the runtime bind graph or evaluator cells as product API.
5. Existing upstream-corresponding runtime types only where they are already public and semantically stable. Product re-exports should live in `nuxie`/`nuxie-flow`, not expand the port's promise.

### Migration order

1. Establish the exact green parity/golden baseline and keep it pinned in every PR.
2. Land the `artboard.rs` and `state_machine_instance.rs` mechanical splits as separate PRs. This makes later ownership changes reviewable without mixing 20,000-line source moves into semantic work.
3. Demarcate `nuxie::product::{flow_session, scene, script_import}` and the project-converter submodules in place, preserving public re-exports.
4. Define the sealed flow, authoring-file, geometry-snapshot, and script-authorization bridges; migrate existing internal callers to them while everything remains in the same crate.
5. Extract the non-product façade/runtime handles into `nuxie-core`, with `nuxie` temporarily acting as an umbrella compatibility crate.
6. Move FlowSession and script import to `nuxie-flow`; move their tests and retain compatibility re-exports.
7. Move Scene model, SceneTx, export schema/generator, lowering, live adapter, and tests together. Do not split the generated schema from `build.rs` or split transaction commit from materialization in the extraction PR.
8. Isolate the project converter's pure model/program/evaluator. Extract it to a neutral crate only if doing so leaves the runtime adapter dependency pointing inward, never from `nuxie-runtime` to `nuxie-flow`.
9. Remove compatibility re-exports and tighten visibility in a later breaking-change PR, after downstream migration.

### Risk map

| Risk | Areas | Reason |
| --- | --- | --- |
| Mechanical/low | Thin namespace façades, codegen coordinators staying in place, test-module extraction, pure FlowSession validation/types | Mostly physical ownership with no runtime mutation semantics. |
| Moderate | FlowSession after bridge creation, script import after capability API, Scene DTO/export types | Public API and authorization compatibility, but limited graph mutation. |
| High | SceneTx commit/materialization, live Scene mounting, geometry/semantic access, Scene lowering and generated schema | Atomicity, identity, generated code, and live runtime invariants cross the seam. |
| High | Project-data converter extraction | The source looks pure, but evaluator state is embedded in runtime data-bind context and Number-to-List policy. |
| High | Script/listener ownership during FlowSession moves | Rehydration, rollback, command queues, and VM object lifetime must remain one coherent transaction. |

### Why baseline-first matters

`docs/parity-gap-register.md` explicitly recommends making the parity claim trustworthy before feature work and landing each subsequent change against the oracle (the “Recommended execution order” around lines 204-211). It also describes Scene as a Nuxie-only additive API (around lines 126-127) while recording FlowSession/capability-boundary gaps elsewhere in the register. That is the governing principle here: first make source moves mechanically observable by the existing correspondence, ownership, trace, golden, and silver batteries; then introduce the product seam behind those same oracles. A green build alone is insufficient because it can preserve compilation while silently weakening ownership scans or changing what “zero divergences” measures.

The two large-file splits are therefore prerequisites for, not part of, the product extraction. Each split keeps the logical Rust module unchanged and updates only physical-address consumers. The seam work then proceeds through explicit bridges and compatibility re-exports, with risky transaction/materialization changes isolated from mechanical moves.

# nuxie-runtime structure report

## Scope and baseline

This is a read-only structure analysis. It proposes no behavior, source, parity-ledger, or ownership-ledger changes.

The inspected worktree was already detached at `e8726db8ffc689d3b19f1c0a55794aa6daf9956d` (`Merge pull request #246 from TheOneLevin/feat/b6-0058-listener-viewmodel-change`, 2026-08-04). The local `origin/main` ref resolves to the same commit. The requested `git fetch origin` and redundant `git checkout --detach origin/main` could not update the shared worktree metadata because the sandbox cannot write the parent repository's `.git/worktrees/nuxie-mr-c12` directory. The analysis is therefore pinned to the locally available `origin/main` commit above, not a newly fetched remote ref. The pre-existing untracked `vtriage-capture/` directory was left untouched.

The two target files have grown beyond the sizes in the request:

| File | Current lines | Production/support | Tests |
| --- | ---: | ---: | ---: |
| `crates/nuxie-runtime/src/artboard.rs` | 23,374 | 12,039 | 11,335 (`205` tests) |
| `crates/nuxie-runtime/src/state_machine/state_machine_instance.rs` | 21,674 | 14,079 | 7,595 (`94` tests) |

The safest split mechanism is literal `include!` fragments, not Rust child modules. An included fragment remains in the original semantic module (`crate::artboard` or `crate::state_machine::state_machine_instance`), so private-name resolution, inherent method paths, sibling access, and demangled symbols remain unchanged. A `mod build;`/`#[path = ...] mod build;` split would introduce new privacy boundaries and make a supposedly mechanical PR into an API refactor.

---

## 1. Large-file split plan: `artboard.rs`

### Current structural map

| Lines | Region |
| ---: | --- |
| 1-106 | Imports and supporting aliases |
| 107-135 | Draw-frame counter and generated `Mat2D` helper |
| 136-160 | `ExternalFontAssetError` |
| 161-181 | `RuntimeArtboardInstanceIdentity` |
| 182-231 | `RuntimeScriptState` and implementations |
| 232-250 | Advancing-component and persistent-dirt fixtures |
| 251-298 | Script advance/update mode enums and implementations |
| 299-327 | Resetting/runtime-component helper types |
| 328-334 | `RuntimeTextStyleFeatureOption` |
| 335-523 | `ArtboardInstance` storage |
| 524-714 | `Clone for ArtboardInstance` |
| 715-739 | `Drop for ArtboardInstance` followed by the existing leaf `include!` declarations |
| 740-773 | Occurrence and frame-advance report types |
| 774-844 | Build context/index/text-flag helpers |
| 845-860 | `RuntimeNestedAnimationInstance` |
| 861-11,133 | Main `impl ArtboardInstance` |
| 11,134-11,147 | Focus-scroll path and focus-bounds transform helper |
| 11,148-11,187 | Small follow-up `impl ArtboardInstance` |
| 11,188-11,226 | Component-list reset helper |
| 11,227-11,595 | `impl RuntimeNestedArtboardInstance` |
| 11,596-11,611 | `impl RuntimeNestedAnimationInstance` |
| 11,612-12,039 | Free construction/nested-artboard helpers |
| 12,040-23,374 | Single `#[cfg(test)] mod tests` (`205` tests) |

The main implementation itself divides along stable responsibility boundaries:

| Lines | Logical responsibility |
| ---: | --- |
| 861-1,104 | Lifecycle, data context, and transient-clone restoration |
| 1,105-2,440 | Build occurrence relationships and constructors |
| 2,441-2,712 | Path/hit initialization and external assets |
| 2,713-3,399 | Scripting lifecycle |
| 3,400-3,989 | Object/component/audio/hit/graph/definition access |
| 3,990-4,152 | Scripted interpolators and state-machine definition selection |
| 4,153-4,480 | Dimensions, world/layout bounds, scrolling, and focus reveal |
| 4,481-5,229 | Generated property façade and text values |
| 5,230-5,568 | Animation/state-machine/view-model occurrences |
| 5,569-6,312 | Component-list lifecycle and layout |
| 6,313-7,408 | Root/nested frame advance and retained-component dispatch |
| 7,409-7,585 | Nested-layout frame propagation |
| 7,586-8,496 | Transforms, epochs, dirt, and invalidation |
| 8,497-9,191 | Update pass and five-pass state-machine settlement |
| 9,192-9,825 | Component update dispatch and script scheduling |
| 9,826-11,133 | Property-change callbacks, nested controls, and collapse |

### Proposed physical layout

Keep `artboard.rs` as the owner. It retains imports, supporting types, `ArtboardInstance`, `Clone`, `Drop`, the existing leaf includes, occurrence/report/build helper types, and an ordered series of literal includes. This preserves declaration order where it matters and makes all fragments part of `crate::artboard`.

```text
src/
  artboard.rs                         owner/types, existing leaf includes, ordered include! list
  artboard/
    build.rs                          current 861-2712
    script_runtime.rs                 current 2713-3399
    access.rs                         current 3400-3989
    definitions_and_layout.rs         current 3990-4480
    properties.rs                     current 4481-5568
    component_lists.rs                current 5569-6312
    advance.rs                        current 6313-7585
    dirt.rs                           current 7586-8496
    update.rs                         current 8497-9825
    callbacks.rs                      current 9826-11133
    nested.rs                         current 11134-12039
    tests.rs                          body of current tests module
```

The production fragments are intentionally in execution/dependency order rather than alphabetical order. `tests.rs` should be included inside the original test module:

```rust
#[cfg(test)]
mod tests {
    include!("artboard/tests.rs");
}
```

This retains the test module's name and access. It also gives the attribution checker a deliberately exempt `tests.rs` leaf.

### Lockstep correspondence and checker manifest

The following table is the required per-file edit manifest. “B6 rows” are `[[files]]` entries in `file-correspondence-manifest.toml`; add the fragment's path to those rows' semicolon-separated `rust_module` values and update their C17 retained-module notes. Keep status, upstream pins, C++ paths, verification text, and correspondence claims unchanged. The row list is conservative: a later provenance cleanup can narrow it, but narrowing must not be mixed into this mechanical split.

| New file | B6 rows that must name it | Frame-loop/checker bindings that must move with it |
| --- | --- | --- |
| `artboard/build.rs` | `B6-0001`, `B6-0094`, `B6-0115`, `B6-0117`, `B6-0123`, `B6-0125`, `B6-0129`, `B6-0131`, `B6-0142`, `B6-0146`, `B6-0202`, `B6-0258`, `B6-0303`, `B6-0304`, `B6-0305`, `B6-0331`, `B6-0361`, `B6-0362`, `B6-0365`, `B6-0368`, `B6-0385`, `B6-0388`, `B6-0399`, `B6-0405`, `B6-0408` | Repoint `summarize_trace.py` dependency-build and IK-chain-build source scans when their anchors land here. Repoint matching `runtime-frame-loop-gaps.toml` owner-boundary entries. |
| `artboard/script_runtime.rs` | `B6-0077`, `B6-0194`, `B6-0200`, `B6-0322`, `B6-0326` | Repoint the script-lifecycle citations in `runtime-frame-loop-ownership.toml` and any corresponding synthetic test fixture paths. |
| `artboard/access.rs` | `B6-0094`, `B6-0095`, `B6-0123`, `B6-0146`, `B6-0203`, `B6-0204`, `B6-0205` | Repoint any live-draw/renderer-interface lifecycle citations whose named methods move here. |
| `artboard/definitions_and_layout.rs` | `B6-0077`, `B6-0094`, `B6-0258`, `B6-0303`, `B6-0304`, `B6-0305`, `B6-0385`, `B6-0388` | Repoint focus-retained-tree and layout lifecycle citations, plus matching owner-boundary allow entries. |
| `artboard/properties.rs` | `B6-0067`, `B6-0077`, `B6-0094`, `B6-0194`, `B6-0200`, `B6-0352`, `B6-0355`, `B6-0385`, `B6-0388`, `B6-0399`, `B6-0404`, `B6-0405`, `B6-0408` | `B6-0067` currently has only one Rust module: **replace** `artboard.rs` with this fragment rather than adding a second path. Repoint property/text lifecycle citations and corresponding owner-boundary entries. |
| `artboard/component_lists.rs` | `B6-0095`, `B6-0258`, `B6-0303`, `B6-0304`, `B6-0305` | Repoint component-list and nested-layout lifecycle citations and matching test fixtures. |
| `artboard/advance.rs` | `B6-0001`, `B6-0059`, `B6-0061`, `B6-0063`, `B6-0064`, `B6-0077`, `B6-0094`, `B6-0095`, `B6-0194`, `B6-0200`, `B6-0258`, `B6-0303`, `B6-0304`, `B6-0305`, `B6-0322`, `B6-0326`, `B6-0388` | In `runtime-frame-loop-ownership.toml`, move `artboard.advance`'s `rust_file` and the lifecycle citations for artboard/nested advance. Repoint `summarize_trace.py` advancing/resetting dispatch scans as applicable. Repoint relevant gap allow rows and `test_check.py` advance fixtures. |
| `artboard/dirt.rs` | `B6-0094`, `B6-0123`, `B6-0258`, `B6-0331`, `B6-0352`, `B6-0355`, `B6-0357`, `B6-0361`, `B6-0362`, `B6-0365`, `B6-0368`, `B6-0385`, `B6-0388`, `B6-0399`, `B6-0405`, `B6-0408` | Move `component.dirt`, `component.dependents` (`pub fn add_dirt(`), and transform/dirt lifecycle citations as appropriate. Repoint `summarize_trace.py`'s two add-dirt anchors, dirt consumptions, owner resolutions, and skin-buffer scans according to their final physical locations. Repoint matching gap rows. |
| `artboard/update.rs` | `B6-0001`, `B6-0094`, `B6-0115`, `B6-0117`, `B6-0123`, `B6-0125`, `B6-0129`, `B6-0131`, `B6-0142`, `B6-0194`, `B6-0200`, `B6-0203`, `B6-0204`, `B6-0205`, `B6-0258`, `B6-0303`, `B6-0304`, `B6-0305`, `B6-0322`, `B6-0326`, `B6-0331`, `B6-0352`, `B6-0355`, `B6-0357`, `B6-0361`, `B6-0362`, `B6-0365`, `B6-0368`, `B6-0385`, `B6-0388`, `B6-0399`, `B6-0404`, `B6-0405`, `B6-0408` | Move `component.update_order` (`fn update_components_with_hook_recording`) and `artboard.update_pass` (`pub fn update_pass(`) `rust_file` values and lifecycle citations. Repoint retained update/settlement gap rows, trace scans, and test fixtures. |
| `artboard/callbacks.rs` | `B6-0059`, `B6-0061`, `B6-0063`, `B6-0064`, `B6-0077`, `B6-0094`, `B6-0123`, `B6-0125`, `B6-0129`, `B6-0131`, `B6-0142`, `B6-0194`, `B6-0200`, `B6-0258`, `B6-0303`, `B6-0304`, `B6-0305`, `B6-0331`, `B6-0352`, `B6-0355`, `B6-0357`, `B6-0361`, `B6-0362`, `B6-0365`, `B6-0368`, `B6-0385`, `B6-0388`, `B6-0399`, `B6-0404`, `B6-0405`, `B6-0408` | Change `check_layout_style_handlers.py`'s `RUST_ARTBOARD_MODULE` to this fragment if both display-change markers remain here; otherwise make the checker accept the two explicit physical files. Repoint callback lifecycle citations, gap rows, and fixtures. |
| `artboard/nested.rs` | `B6-0059`, `B6-0061`, `B6-0063`, `B6-0064`, `B6-0077`, `B6-0094`, `B6-0095`, `B6-0258`, `B6-0303`, `B6-0304`, `B6-0305` | Repoint nested-artboard advance/layout citations, gap allow entries, and nested fixtures. |
| `artboard/tests.rs` | None. `rust_attribution.py` excludes a source whose stem is `tests`. | Move the artboard test body unchanged. Update the seven relative `include_bytes!`/fixture paths currently near old lines 19,951, 20,001, 20,168, 20,542, 20,597, 20,612, and 23,311 because the file becomes one directory deeper. Prefer `concat!(env!("CARGO_MANIFEST_DIR"), "/...")` for fixture stability. The `env!` use near old line 14,290 remains valid. |

#### Global artboard bindings

These bindings span more than one fragment and therefore belong in the same PR:

- `rust_attribution.py` discovers all production `.rs` files below `crates/nuxie-runtime/src`. Every new production fragment must be listed in at least one `file-correspondence-manifest.toml` row and must **not** be added to `rust-additions.toml`. The checker exempts `tests.rs`, names ending in `_tests`, and files below a `tests/` path.
- The manifest's scatter ratchet is already at its maximum: 155 rows currently have multiple Rust modules. Except for `B6-0067`, all affected artboard rows are already multi-module. Replacing the `B6-0067` root path prevents the split from increasing that count.
- Update every affected C17 note that enumerates retained Rust modules. Do not change the underlying parity classification merely because the code's physical address changed.
- `docs/runtime-frame-loop-ownership.toml` has artboard citations in `focus.retained_tree`, `component.identity`, `component.dirt`, `component.dependents`, `component.update_order`, `component.transforms`, `component.clone_drop`, `artboard.advance`, `artboard.update_pass`, `nested_artboard.advance`, `artboard.live_draw`, and `renderer.interface_boundary`. Keep `component.clone_drop` and its `pub struct ArtboardInstance` anchor on the root; change each other `rust_file` or `rust:path:line-range` to its physical fragment.
- `docs/runtime-frame-loop-gaps.toml` has 25 owner-boundary allow entries pointing at `artboard.rs` (entries 0-7, 11-22, and 29-34 in current order). Repoint each to the fragment that contains its anchor. A literal move should preserve its token-based `site_hash`; a changed hash is a signal that the split was not mechanical.
- `tools/runtime-frame-loop-port/summarize_trace.py` hard-codes `artboard.rs` nine times for add-dirt, dirt consumption, owner resolution, dependency build, skin rebuild, advancing dispatch, resetting dispatch, and IK-chain discovery. Give each query its actual fragment path. The demangled owner should remain `artboard::ArtboardInstance` because `include!` does not introduce a child module.
- `tools/runtime-frame-loop-port/check_layout_style_handlers.py` hard-codes the artboard source while looking for `propagate_layout_component_display_changed` and the display property key. Point it to the fragment(s) containing those markers.
- `tools/runtime-frame-loop-port/test_check.py` contains at least 14 exact artboard paths covering probe gating, layer occurrence, live-advance ratchets, registry site hashes/substitution, and trait-anchor behavior. Update each synthetic fixture to the fragment that owns the tested anchor; do not perform a blind string replacement.
- Leave the existing root-level includes (`nested_artboard.rs`, `artboard_component_list.rs`, `artboard_list_map_rule.rs`, `artboard_referencer.rs`, `bindable_artboard.rs`, `nested_artboard_layout.rs`, `nested_artboard_leaf.rs`, `component_origin.rs`, bone/weight, profiler/profile) where they are. Moving them under `artboard/` changes relative include resolution and correspondence ownership.
- Keep `crates/nuxie-runtime/src/lib.rs`'s `mod artboard;` unchanged. The public and crate-private paths must not acquire an extra `artboard::<fragment>` segment.

### Mechanical PR sequence and proof

One PR should contain only this file's split and the path/anchor bookkeeping above. Recommended commit sequence within that PR:

1. Extract `tests.rs`, fix only its fixture paths, and prove the same 205 tests are collected.
2. Extract production regions in original order using `include!`; make no renames, visibility edits, formatting sweeps, or logic edits.
3. Update correspondence C17/path fields and all frame-loop physical anchors.
4. Update checker source locations and synthetic fixtures, including an assertion that demangled ownership remains `artboard::ArtboardInstance`.

Proof battery:

```text
make rust-sources-fresh
make cpp-probe
make runtime-frame-loop-port-test
make runtime-frame-loop-port-check
make rust-attribution-check
cargo check --workspace
cargo test -p nuxie-runtime
cargo test -p nuxie --features scripting
make scripted-golden-compare
make silver-corpus-test
```

The PR is structure-preserving only if parity row counts/statuses, trace landmarks, owner-boundary hashes, test counts, and golden/silver outputs are unchanged.

---

## 2. Large-file split plan: `state_machine_instance.rs`

### Current structural map

| Lines | Region |
| ---: | --- |
| 1-93 | Imports and ownership commentary |
| 94-255 | Nested event-chain/notify phases, trace structures, and trace recording |
| 256-312 | Semantic-node resolver and audio/event seam types |
| 313-1,271 | `RuntimeHitResult`, `HitComponent`, and hierarchy implementations |
| 1,272-1,467 | Nested notifier, focus, semantic, and state helper types |
| 1,468-1,474 | `RuntimeDataContextBindError` |
| 1,475-1,711 | `StateMachineInstance` storage |
| 1,712-1,744 | Data-bind occurrence/pointer/executor structures |
| 1,745-2,147 | Listener-action executor implementations |
| 2,148-2,335 | `Clone for StateMachineInstance` |
| 2,336-2,356 | `Drop for StateMachineInstance` |
| 2,357-2,638 | Free helpers and view-model listener types |
| 2,639-13,938 | Main `impl StateMachineInstance` |
| 13,939-13,953 | `runtime_owned_font` helper |
| 13,954-14,079 | `view_model_listener_tests` (`1` test) |
| 14,080-21,674 | `scripted_listener_action_tests` (`93` tests) |

The main implementation divides as follows:

| Lines | Logical responsibility |
| ---: | --- |
| 2,639-2,932 | Teardown, enrollment, focus manager, clone rebind |
| 2,933-3,741 | Construction and listener/layer initialization |
| 3,742-5,102 | Scripting, hydration, and adoption |
| 5,103-6,007 | State/input/focus/semantic/keyboard/gamepad operations |
| 6,008-7,487 | Hit/listener/pointer pipeline |
| 7,488-8,525 | Event notification, deferred callbacks, listener actions |
| 8,526-9,100 | Direct bind targets and value readers |
| 9,101-10,748 | Default/imported/owned source setters and relinking |
| 10,749-12,599 | Data-context bind/rebind, listener cells, bind updates |
| 12,600-13,938 | Reports, advance/apply, nested event dispatch, settlement |

### Proposed physical layout

```text
src/state_machine/
  state_machine_instance.rs               owner/types, hit hierarchy, lifecycle, ordered include! list
  state_machine_instance/
    construction.rs                       current 2639-3741
    scripted_objects.rs                   current 3742-5102
    input_focus.rs                        current 5103-6007
    pointer.rs                            current 6008-7487
    events_actions.rs                     current 7488-8525
    bind_targets.rs                       current 8526-9100
    bind_sources.rs                       current 9101-10748
    data_context.rs                       current 10749-12599
    advance.rs                            current 12600-13953
    tests/
      view_model_listener.rs              body of current first test module
      scripted_listener_actions.rs         body of current second test module
```

Retain the two existing module names in the root and include only their bodies. This preserves test paths and any name-sensitive filtering:

```rust
#[cfg(test)]
mod view_model_listener_tests {
    include!("state_machine_instance/tests/view_model_listener.rs");
}
```

Use the same pattern for `scripted_listener_action_tests`.

### Lockstep correspondence and checker manifest

| New file | B6 rows that must name it | Frame-loop/checker bindings that must move with it |
| --- | --- | --- |
| `state_machine_instance/construction.rs` | `B6-0077`, `B6-0194`, `B6-0195`, `B6-0196`, `B6-0200`, `B6-0310` | Repoint construction/enrollment lifecycle citations and matching synthetic fixtures. Keep `state_machine.collections`' `pub struct StateMachineInstance` anchor on the root. |
| `state_machine_instance/scripted_objects.rs` | `B6-0077`, `B6-0175`, `B6-0194`, `B6-0195`, `B6-0196`, `B6-0200` | Repoint scripting/hydration lifecycle citations and scripted-action fixture paths. |
| `state_machine_instance/input_focus.rs` | `B6-0077`, `B6-0083` | Repoint focus/semantic/keyboard/gamepad citations and tests. |
| `state_machine_instance/pointer.rs` | `B6-0077`, `B6-0083` | Repoint hit/pointer/listener lifecycle citations and FL-C4/layer/hit fixtures. |
| `state_machine_instance/events_actions.rs` | `B6-0058`, `B6-0077`, `B6-0440` | Repoint event reporting, firing/bubbling, and action-dispatch citations/fixtures. `state_machine.actions`' primary anchor remains on the root if the listener-action executor stays there. |
| `state_machine_instance/bind_targets.rs` | `B6-0058`, `B6-0194`, `B6-0196`, `B6-0200`, `B6-0440` | Repoint bind-target and view-model listener citations/fixtures. |
| `state_machine_instance/bind_sources.rs` | `B6-0058`, `B6-0175`, `B6-0194`, `B6-0195`, `B6-0196`, `B6-0200`, `B6-0440` | Repoint source/relink/data-converter citations and bind fixtures. |
| `state_machine_instance/data_context.rs` | `B6-0058`, `B6-0077`, `B6-0194`, `B6-0195`, `B6-0196`, `B6-0200`, `B6-0440` | Repoint data-context lifecycle citations and listener-cell/bind fixtures. |
| `state_machine_instance/advance.rs` | `B6-0077`, `B6-0194`, `B6-0195`, `B6-0196`, `B6-0200`, `B6-0310`, `B6-0440` | Move `state_machine.advance`'s `rust_file` and `advance_with_report_mode` anchor here. Repoint state-machine event/action/reporting lifecycle citations and advance/keyframe fixtures. Include `runtime_owned_font` in this fragment to avoid an artificial one-function file. |
| `state_machine_instance/tests/view_model_listener.rs` | None; it resides below a `tests/` path. | Move the body unchanged and preserve the outer module name. |
| `state_machine_instance/tests/scripted_listener_actions.rs` | None; it resides below a `tests/` path. | Move the body unchanged. Replace or correct the relative fixture path near old line 19,011 (`../../../../fixtures/sync/vm_listener_fire_event.riv`) because the source becomes two directories deeper; prefer a `CARGO_MANIFEST_DIR`-anchored path. |

#### Global state-machine bindings

- All production fragments must be named by `file-correspondence-manifest.toml`, never `rust-additions.toml`. Every affected SMI row is already multi-module, so this split does not increase the manifest's 155-row scatter count.
- Update C17 retained-module notes for `B6-0058`, `B6-0077`, `B6-0083`, `B6-0175`, `B6-0194`, `B6-0195`, `B6-0196`, `B6-0200`, `B6-0310`, and `B6-0440`; leave the substantive parity assertions untouched.
- `docs/runtime-frame-loop-ownership.toml` cites this file from `focus.retained_tree`, `state_machine.collections`, `state_machine.advance`, `state_machine.events`, `state_machine.actions`, and `event.reporting`. Keep root anchors for the struct, stored event collection, and action executor. Repoint method/lifecycle citations to physical fragments.
- `tools/runtime-frame-loop-port/check.py` currently treats one hard-coded SMI file as the nested-event owner and excludes it from outside-owner scans. Once methods are in included fragments, a path-only exclusion would falsely flag the fragments. Make owner discovery include-aware: recursively expand literal local `include!("...")` files in source order for export analysis, and treat every expanded file as the same logical owner. Do not broaden the exclusion to a directory glob.
- Add checker unit coverage for the include-aware logical owner: an exported nested-event method in an included fragment must be accepted, while the same method in an unrelated file must still fail. Preserve syntax parsing and duplicate-anchor detection.
- `tools/runtime-frame-loop-port/test_check.py` has at least 14 exact SMI references, plus constructed/split path strings, covering FL-C4, lifecycle, layer, hit, bind, events, firing/bubbling, advance, owner exports, keyframes, focus, and semantic behavior. Point each fixture to the actual fragment; do not simply substitute the root path everywhere.
- `docs/runtime-frame-loop-gaps.toml` has no current SMI owner-boundary allow row to migrate.
- Keep `crates/nuxie-runtime/src/state_machine.rs`'s `pub(crate) mod state_machine_instance;` and its re-exports unchanged. Literal includes keep all symbols at the original module path.

### Mechanical PR sequence and proof

This should be a second, independent PR after the artboard split has landed. It uses the same proof battery. The checker must first learn the logical-owner/include relationship in the same PR, because moving the methods without that update creates false ownership failures. Apart from that narrowly required path-awareness, no checker policy should change.

Success means the same 94 tests are collected, no exported API path changes, the correspondence scatter count stays at 155, no new owner-boundary allowance appears, and the full battery shown in the artboard plan remains green.

---

## 3. Nuxie-only seam inventory

### Classification method

The authoritative starting point is correspondence metadata:

- Production Rust paths mapped to upstream C++ in `file-correspondence-manifest.toml` are port-side unless a smaller product-only region can be proven.
- Production Rust paths listed in `rust-additions.toml` are intentional Rust-only additions. The relevant ownership labels are `flowsession-abi`, `scene-api`, and generated/codegen infrastructure.
- `build.rs` is outside the attribution scanner's `src/` roots, so it must be inspected manually.
- A convenience façade is not automatically product-side. Thin Rust-only namespace/generated-storage coordinators that serve the port stay with the port even though no one C++ file corresponds to them.

This prevents two opposite mistakes: burying product authoring features inside the port forever, and extracting Rust implementation infrastructure merely because its file has no one-to-one C++ source.

### Product features and mixed glue

| Region | Current size and location | Runtime internals reached / extraction surface | Placement assessment |
| --- | --- | --- | --- |
| FlowSession wire/domain model | `crates/nuxie/src/flow_session.rs:21-621` (601 lines) | Public `File`, `Factory`, renderer, artboard, view-model, and state-machine types | Clean product vocabulary; move to `nuxie-flow` after dependency direction is fixed. |
| FlowSession orchestration | `flow_session.rs:622-2163` (1,542) | Calls crate-private script preparation, listener preparation, rehydration, apply/rollback, host-cycle, and command-drain methods on the `nuxie` façade | Product-side, but requires a narrow public/sealed flow-runtime bridge before crate extraction. Moderate coupling. |
| FlowSession value snapshots | `flow_session.rs:2164-3105` (942) | Traverses public runtime-owned view-model/value objects and builds its own graph/arena | Product-side and cohesive. Mostly mechanical after bridge creation. |
| FlowSession validation/accounting | `flow_session.rs:3106-3545` (440) | Product protocol limits and payload validation | Clean extraction candidate. |
| FlowSession mutation/apply/diff | `flow_session.rs:3546-4515` (970) | Applies mutations through runtime view-model/state-machine handles; shares rollback/command sequencing with façade-private methods | Product-side; transactional sequencing makes it medium risk. |
| FlowSession selection/catalog | `flow_session.rs:4516-5020` (505) | File/artboard/state-machine discovery | Product-side; mostly public runtime surface. |
| FlowSession tests | `flow_session.rs:5021-7227` (2,207; 40 tests), plus `crates/nuxie/tests/flow_session_contract.rs` (335) | Exercises the public ABI, VM lifecycle, and transactional behavior | Move with FlowSession. Preserve public names through compatibility re-exports during migration. |
| Scene contract and IDs | `crates/nuxie/src/scene.rs:28-1187` (1,160) | Public runtime/binary identities and errors | Product-side, cleanly demarcatable. |
| Scene durable definitions/index/specs | `scene.rs:1188-3401` (2,214) | `nuxie_binary::AuthoringRecord`, property/value types, runtime file identities | Product-side authoring model. |
| Scene hierarchy/materialization | `scene.rs:3402-5622` (2,221) | Constructs/reads `RuntimeFile`; directly touches `materialized.file.runtime` in a handful of places even though `File::runtime()` already exists | Product-side but high coupling. Replace direct private field access with the existing public accessor before moving. |
| Silver/scene export schema | `scene.rs:5623-6819` (1,197) | Public exported-record/document DTOs; lowering later feeds `RuntimeFile::from_authoring_records` | Product-side. The schema and its generated counterpart must move together. |
| Scene store/live mount API | `scene.rs:6820-9099` (2,280) | Owns mounted `File`/artboard/view-model state and coordinates transactions | Product-side; medium-to-high coupling. |
| SceneTx and subordinate transactions | `scene.rs:9100-15589` (6,490) | `SceneTx`, `DataConverterTx`, `VmTx`, `AnimTx`, and `MachineTx` mutate the authoring graph, then materialize into runtime/binary types | Core product authoring seam. Cohesive, but atomic commit/materialization behavior makes the move high risk. |
| Scene frame queries/live mutation | `scene.rs:15590-17592` (2,003) | Hit/geometry/semantic queries and intrinsic image sizing reach crate-private `OwnedArtboardInstance` helpers | Product-side. Needs stable snapshot/command DTOs instead of exposing mutable runtime internals. |
| Scene lowering/validation/export | `scene.rs:17593-25563` (7,971) | Converts authoring records into `RuntimeFile`, validates cross-object invariants, and produces exported documents | Product-side. Highest-risk region because it couples the authoring model to port construction invariants. |
| Scene tests | `scene.rs:25564-38224` (12,661; 120 tests), plus `crates/nuxie/tests/scene_authoring.rs` (15,674) and related authoring tests | Broad transaction, materialization, export, and live-runtime contract | Move with Scene; use this suite as the seam migration oracle. |
| Scene schema generator | `crates/nuxie/build.rs` (4,531) and `scene.rs`'s `include!(concat!(env!("OUT_DIR"), "/scene_schema.rs"))` near line 860 | Generates the Scene/silver record schema at build time | Product-side and inseparable from Scene. Extraction must move the build script/generator inputs and preserve the generated include contract. |
| Script import authorization | `crates/nuxie/src/script_import.rs:1-157` production, `158-211` tests | Ed25519/container verification; calls crate-private `File::execution_authorization_for`/authorization state | Product policy. Move to `nuxie-flow`, but expose a capability-based import seam rather than making raw authorization state public. |
| Mixed façade bridge | `crates/nuxie/src/lib.rs`, principally project-envelope classification and private bridge methods around lines 55, 154, 223-258, 307, 344, 376, 1,492, 2,983, 3,328, 4,304, and 5,600-5,733 | `File::from_runtime`; flow script/listener lifecycle; apply/rollback/command drain; artboard geometry/semantic/intrinsic-image helpers | Not an independent feature. Convert these call sites into explicit narrow bridge traits/DTOs, then move their product consumers. |

The concrete crate-private methods the extraction must account for are:

- Flow lifecycle around `nuxie/src/lib.rs:5600-5675`: `prepare_flow_scripts`, `prepare_flow_listener_actions`, rehydrate/apply, begin host cycle, rollback, and drain commands.
- Scene live/runtime queries around `lib.rs:5705-5733`: hit-test path segments with bounds, geometry path segments, retained/semantic text, and intrinsic image-dimension registration.
- `File::from_runtime` around `lib.rs:4304` and the script execution authorization capability.

These are already cross-crate calls from `nuxie` into public `nuxie-runtime` types; the missing visibility is chiefly inside the `nuxie` façade. Extraction should expose a sealed product bridge from the future base façade, not expand dozens of runtime fields to `pub`.

### Project-data converter: product model inside the port

`crates/nuxie-runtime/src/project_data_converter.rs` is 2,686 lines and is explicitly classified `scene-api` in `rust-additions.toml`:

| Lines | Region | Coupling and disposition |
| ---: | --- | --- |
| 22-507 | Public values, specs, errors, and state | Pure product/protocol model; eventual neutral-crate candidate. |
| 508-1,289 | Envelope/program/catalog compilation and evaluation | Mostly `std`/`serde`; eventual neutral-crate candidate. |
| 1,290-2,324 | Validation, coercion, formatting, interpolation | Pure-looking core, but behavior is part of runtime data-bind semantics. Extract only with parity tests. |
| 2,325-2,686 | Formula parser | Mechanically separable after the outer seam is established. |

The file is locally self-contained, but its runtime integration is not. `data_bind/context/context_value.rs` has a `RuntimeDataBindGraphConverter::Project` variant and owns conversion/evaluator state; `data_bind/data_bind_context.rs` constructs it from a script-asset envelope; both runtime and `nuxie` re-export or classify its types; Scene authors and lowers them. A direct move to `nuxie-flow` would invert the desired dependency and make the port depend on the product crate.

Two crate-private numeric-policy helpers in this file (the bounded-list constant near line 467 and helper near line 2,320) are also used by port-side Number-to-List/context conversion. If the model is eventually extracted, those policies should remain in the port or move to a dependency-neutral shared crate; they should not become a product API solely to satisfy the move.

Recommendation: first demarcate this as `project_data_converter/{model, program, evaluator, bridge}` inside `nuxie-runtime`; later extract only the pure model/program/evaluator to a neutral `nuxie-project-data` crate that both runtime and `nuxie-flow` may depend on. The runtime adapter stays in `nuxie-runtime`.

### Rust-only infrastructure that should remain port-side

The following `rust-additions.toml` entries have no one-file C++ correspondence but are not product features:

| Current path/group | Lines | Why it stays in `nuxie-runtime` |
| --- | ---: | --- |
| `data_bind_container.rs`, `data_converter.rs`, `retained_data_bind.rs` | 12 total | Tiny generated/type façade modules over port functionality. |
| `input/mod.rs`, `shapes/mod.rs`, `shapes/paint/mod.rs` | 168 total | Rust namespace/generated-storage coordinators. |
| `objects.rs` | 1,018 | Runtime object registry plus `include!(concat!(env!("OUT_DIR"), "/runtime_objects.rs"))`; generated by `crates/nuxie-runtime/build.rs` (707 lines). This is port construction infrastructure. |
| `properties.rs` | 826 | Generated property interpretation/storage used throughout the port. |
| `state_machine.rs`, `state_machine/instance.rs`, `state_machine/listener_types/mod.rs` | 215 total | Rust module/re-export façade for the ported state-machine implementation. |
| `viewmodel/mod.rs`, `viewmodel/runtime/mod.rs` | 58 total | Rust namespace/re-export façade for ported view-model behavior. |
| `focus.rs` | 7 | Thin public façade over retained focus behavior, not a standalone product feature. |

These codegen/facade additions total about 2,303 source lines excluding the project converter and focus façade. They should be clearly marked as Rust port infrastructure, but moving them to `nuxie-flow` would blur rather than improve the seam.

`crates/nuxie/src/command_queue.rs`, server code, and raw-text support are not included in the Nuxie-only list: their current files are explicitly mapped to upstream C++ correspondence. A richer Rust API around a ported facility does not by itself make that facility product-side.

---

## 4. Recommended seam design

### Decision

Use a **module boundary now and a crate boundary later**.

The immediate structure should make product ownership visible without changing dependencies:

```text
crates/nuxie/src/product/
  flow_session/
  scene/
    model/
    transaction/
    live/
    export/
  script_import/

crates/nuxie-runtime/src/project_data_converter/
  model.rs
  program.rs
  evaluator.rs
  bridge.rs
```

Preserve current public paths with root re-exports. This first step is a namespace/demarcation change, not an extraction. It gives reviewers an auditable boundary while the current test and parity baseline still observes exactly the same crate graph.

The long-term crate graph should avoid a cycle:

```text
nuxie-runtime  <---  nuxie-core  <---  nuxie-flow
       ^                 ^                 ^
       |                 |                 |
       +------ nuxie-project-data --------+

nuxie (umbrella compatibility crate) re-exports nuxie-core + nuxie-flow
```

`nuxie-flow` contains FlowSession, Scene/SceneTx, silver/export DTOs and lowering, and script-import policy. `nuxie-core` contains `File`, `Factory`, owned artboard/view-model handles, renderer integration, and narrow bridge traits. `nuxie-runtime` remains the C++-correspondence port plus explicitly labeled Rust port infrastructure. A small dependency-neutral `nuxie-project-data` is optional and should be introduced only after the runtime adapter has been separated from the pure evaluator.

### Public surface of the port/base layer

The extraction should not make arbitrary runtime fields public. The required surface is narrow:

1. A sealed/trusted constructor that wraps a validated `RuntimeFile` as a façade `File`, with an explicit authoring/import authorization policy.
2. A flow-runtime bridge for prepare, rehydrate, apply, host-cycle, rollback, and command-drain operations. Its inputs/outputs should be stable value DTOs, not references to internal queues.
3. Read-only artboard snapshot operations for hit geometry, retained geometry, and semantic text, plus a command for intrinsic image dimensions. Return owned snapshots so Scene cannot retain internal graph borrows.
4. A project-converter adapter owned by `nuxie-runtime`; pure program/model types may come from the neutral crate. Do not expose the runtime bind graph or evaluator cells as product API.
5. Existing upstream-corresponding runtime types only where they are already public and semantically stable. Product re-exports should live in `nuxie`/`nuxie-flow`, not expand the port's promise.

### Migration order

1. Establish the exact green parity/golden baseline and keep it pinned in every PR.
2. Land the `artboard.rs` and `state_machine_instance.rs` mechanical splits as separate PRs. This makes later ownership changes reviewable without mixing 20,000-line source moves into semantic work.
3. Demarcate `nuxie::product::{flow_session, scene, script_import}` and the project-converter submodules in place, preserving public re-exports.
4. Define the sealed flow, authoring-file, geometry-snapshot, and script-authorization bridges; migrate existing internal callers to them while everything remains in the same crate.
5. Extract the non-product façade/runtime handles into `nuxie-core`, with `nuxie` temporarily acting as an umbrella compatibility crate.
6. Move FlowSession and script import to `nuxie-flow`; move their tests and retain compatibility re-exports.
7. Move Scene model, SceneTx, export schema/generator, lowering, live adapter, and tests together. Do not split the generated schema from `build.rs` or split transaction commit from materialization in the extraction PR.
8. Isolate the project converter's pure model/program/evaluator. Extract it to a neutral crate only if doing so leaves the runtime adapter dependency pointing inward, never from `nuxie-runtime` to `nuxie-flow`.
9. Remove compatibility re-exports and tighten visibility in a later breaking-change PR, after downstream migration.

### Risk map

| Risk | Areas | Reason |
| --- | --- | --- |
| Mechanical/low | Thin namespace façades, codegen coordinators staying in place, test-module extraction, pure FlowSession validation/types | Mostly physical ownership with no runtime mutation semantics. |
| Moderate | FlowSession after bridge creation, script import after capability API, Scene DTO/export types | Public API and authorization compatibility, but limited graph mutation. |
| High | SceneTx commit/materialization, live Scene mounting, geometry/semantic access, Scene lowering and generated schema | Atomicity, identity, generated code, and live runtime invariants cross the seam. |
| High | Project-data converter extraction | The source looks pure, but evaluator state is embedded in runtime data-bind context and Number-to-List policy. |
| High | Script/listener ownership during FlowSession moves | Rehydration, rollback, command queues, and VM object lifetime must remain one coherent transaction. |

### Why baseline-first matters

`docs/parity-gap-register.md` explicitly recommends making the parity claim trustworthy before feature work and landing each subsequent change against the oracle (the “Recommended execution order” around lines 204-211). It also describes Scene as a Nuxie-only additive API (around lines 126-127) while recording FlowSession/capability-boundary gaps elsewhere in the register. That is the governing principle here: first make source moves mechanically observable by the existing correspondence, ownership, trace, golden, and silver batteries; then introduce the product seam behind those same oracles. A green build alone is insufficient because it can preserve compilation while silently weakening ownership scans or changing what “zero divergences” measures.

The two large-file splits are therefore prerequisites for, not part of, the product extraction. Each split keeps the logical Rust module unchanged and updates only physical-address consumers. The seam work then proceeds through explicit bridges and compatibility re-exports, with risky transaction/materialization changes isolated from mechanical moves.

# nuxie-runtime structure report

## Scope and baseline

This is a read-only structure analysis. It proposes no behavior, source, parity-ledger, or ownership-ledger changes.

The inspected worktree was already detached at `e8726db8ffc689d3b19f1c0a55794aa6daf9956d` (`Merge pull request #246 from TheOneLevin/feat/b6-0058-listener-viewmodel-change`, 2026-08-04). The local `origin/main` ref resolves to the same commit. The requested `git fetch origin` and redundant `git checkout --detach origin/main` could not update the shared worktree metadata because the sandbox cannot write the parent repository's `.git/worktrees/nuxie-mr-c12` directory. The analysis is therefore pinned to the locally available `origin/main` commit above, not a newly fetched remote ref. The pre-existing untracked `vtriage-capture/` directory was left untouched.

The two target files have grown beyond the sizes in the request:

| File | Current lines | Production/support | Tests |
| --- | ---: | ---: | ---: |
| `crates/nuxie-runtime/src/artboard.rs` | 23,374 | 12,039 | 11,335 (`205` tests) |
| `crates/nuxie-runtime/src/state_machine/state_machine_instance.rs` | 21,674 | 14,079 | 7,595 (`94` tests) |

The safest split mechanism is literal `include!` fragments, not Rust child modules. An included fragment remains in the original semantic module (`crate::artboard` or `crate::state_machine::state_machine_instance`), so private-name resolution, inherent method paths, sibling access, and demangled symbols remain unchanged. A `mod build;`/`#[path = ...] mod build;` split would introduce new privacy boundaries and make a supposedly mechanical PR into an API refactor.

---

## 1. Large-file split plan: `artboard.rs`

### Current structural map

| Lines | Region |
| ---: | --- |
| 1-106 | Imports and supporting aliases |
| 107-135 | Draw-frame counter and generated `Mat2D` helper |
| 136-160 | `ExternalFontAssetError` |
| 161-181 | `RuntimeArtboardInstanceIdentity` |
| 182-231 | `RuntimeScriptState` and implementations |
| 232-250 | Advancing-component and persistent-dirt fixtures |
| 251-298 | Script advance/update mode enums and implementations |
| 299-327 | Resetting/runtime-component helper types |
| 328-334 | `RuntimeTextStyleFeatureOption` |
| 335-523 | `ArtboardInstance` storage |
| 524-714 | `Clone for ArtboardInstance` |
| 715-739 | `Drop for ArtboardInstance` followed by the existing leaf `include!` declarations |
| 740-773 | Occurrence and frame-advance report types |
| 774-844 | Build context/index/text-flag helpers |
| 845-860 | `RuntimeNestedAnimationInstance` |
| 861-11,133 | Main `impl ArtboardInstance` |
| 11,134-11,147 | Focus-scroll path and focus-bounds transform helper |
| 11,148-11,187 | Small follow-up `impl ArtboardInstance` |
| 11,188-11,226 | Component-list reset helper |
| 11,227-11,595 | `impl RuntimeNestedArtboardInstance` |
| 11,596-11,611 | `impl RuntimeNestedAnimationInstance` |
| 11,612-12,039 | Free construction/nested-artboard helpers |
| 12,040-23,374 | Single `#[cfg(test)] mod tests` (`205` tests) |

The main implementation itself divides along stable responsibility boundaries:

| Lines | Logical responsibility |
| ---: | --- |
| 861-1,104 | Lifecycle, data context, and transient-clone restoration |
| 1,105-2,440 | Build occurrence relationships and constructors |
| 2,441-2,712 | Path/hit initialization and external assets |
| 2,713-3,399 | Scripting lifecycle |
| 3,400-3,989 | Object/component/audio/hit/graph/definition access |
| 3,990-4,152 | Scripted interpolators and state-machine definition selection |
| 4,153-4,480 | Dimensions, world/layout bounds, scrolling, and focus reveal |
| 4,481-5,229 | Generated property façade and text values |
| 5,230-5,568 | Animation/state-machine/view-model occurrences |
| 5,569-6,312 | Component-list lifecycle and layout |
| 6,313-7,408 | Root/nested frame advance and retained-component dispatch |
| 7,409-7,585 | Nested-layout frame propagation |
| 7,586-8,496 | Transforms, epochs, dirt, and invalidation |
| 8,497-9,191 | Update pass and five-pass state-machine settlement |
| 9,192-9,825 | Component update dispatch and script scheduling |
| 9,826-11,133 | Property-change callbacks, nested controls, and collapse |

### Proposed physical layout

Keep `artboard.rs` as the owner. It retains imports, supporting types, `ArtboardInstance`, `Clone`, `Drop`, the existing leaf includes, occurrence/report/build helper types, and an ordered series of literal includes. This preserves declaration order where it matters and makes all fragments part of `crate::artboard`.

```text
src/
  artboard.rs                         owner/types, existing leaf includes, ordered include! list
  artboard/
    build.rs                          current 861-2712
    script_runtime.rs                 current 2713-3399
    access.rs                         current 3400-3989
    definitions_and_layout.rs         current 3990-4480
    properties.rs                     current 4481-5568
    component_lists.rs                current 5569-6312
    advance.rs                        current 6313-7585
    dirt.rs                           current 7586-8496
    update.rs                         current 8497-9825
    callbacks.rs                      current 9826-11133
    nested.rs                         current 11134-12039
    tests.rs                          body of current tests module
```

The production fragments are intentionally in execution/dependency order rather than alphabetical order. `tests.rs` should be included inside the original test module:

```rust
#[cfg(test)]
mod tests {
    include!("artboard/tests.rs");
}
```

This retains the test module's name and access. It also gives the attribution checker a deliberately exempt `tests.rs` leaf.

For every production file in the layout above, the owner must contain a literal `include!("artboard/<name>.rs");` at the point where that region previously appeared. Do not use `#[path]`, `mod`, or `pub(crate) mod`: each would create a child-module/privacy boundary. Every such production fragment is attribution-scanned and must be named by the B6 rows below; only `tests.rs` receives the test-source exemption.

### Lockstep correspondence and checker manifest

The following table is the required per-file edit manifest. “B6 rows” are `[[files]]` entries in `file-correspondence-manifest.toml`; add the fragment's path to those rows' semicolon-separated `rust_module` values and update their C17 retained-module notes. Keep status, upstream pins, C++ paths, verification text, and correspondence claims unchanged. The row list is conservative: a later provenance cleanup can narrow it, but narrowing must not be mixed into this mechanical split.

| New file | B6 rows that must name it | Frame-loop/checker bindings that must move with it |
| --- | --- | --- |
| `artboard/build.rs` | `B6-0001`, `B6-0094`, `B6-0115`, `B6-0117`, `B6-0123`, `B6-0125`, `B6-0129`, `B6-0131`, `B6-0142`, `B6-0146`, `B6-0202`, `B6-0258`, `B6-0303`, `B6-0304`, `B6-0305`, `B6-0331`, `B6-0361`, `B6-0362`, `B6-0365`, `B6-0368`, `B6-0385`, `B6-0388`, `B6-0399`, `B6-0405`, `B6-0408` | Repoint `summarize_trace.py` dependency-build and IK-chain-build source scans when their anchors land here. Repoint matching `runtime-frame-loop-gaps.toml` owner-boundary entries. |
| `artboard/script_runtime.rs` | `B6-0077`, `B6-0194`, `B6-0200`, `B6-0322`, `B6-0326` | Repoint the script-lifecycle citations in `runtime-frame-loop-ownership.toml` and any corresponding synthetic test fixture paths. |
| `artboard/access.rs` | `B6-0094`, `B6-0095`, `B6-0123`, `B6-0146`, `B6-0203`, `B6-0204`, `B6-0205` | Repoint any live-draw/renderer-interface lifecycle citations whose named methods move here. |
| `artboard/definitions_and_layout.rs` | `B6-0077`, `B6-0094`, `B6-0258`, `B6-0303`, `B6-0304`, `B6-0305`, `B6-0385`, `B6-0388` | Repoint focus-retained-tree and layout lifecycle citations, plus matching owner-boundary allow entries. |
| `artboard/properties.rs` | `B6-0067`, `B6-0077`, `B6-0094`, `B6-0194`, `B6-0200`, `B6-0352`, `B6-0355`, `B6-0385`, `B6-0388`, `B6-0399`, `B6-0404`, `B6-0405`, `B6-0408` | `B6-0067` currently has only one Rust module: **replace** `artboard.rs` with this fragment rather than adding a second path. Repoint property/text lifecycle citations and corresponding owner-boundary entries. |
| `artboard/component_lists.rs` | `B6-0095`, `B6-0258`, `B6-0303`, `B6-0304`, `B6-0305` | Repoint component-list and nested-layout lifecycle citations and matching test fixtures. |
| `artboard/advance.rs` | `B6-0001`, `B6-0059`, `B6-0061`, `B6-0063`, `B6-0064`, `B6-0077`, `B6-0094`, `B6-0095`, `B6-0194`, `B6-0200`, `B6-0258`, `B6-0303`, `B6-0304`, `B6-0305`, `B6-0322`, `B6-0326`, `B6-0388` | In `runtime-frame-loop-ownership.toml`, move `artboard.advance`'s `rust_file` and the lifecycle citations for artboard/nested advance. Repoint `summarize_trace.py` advancing/resetting dispatch scans as applicable. Repoint relevant gap allow rows and `test_check.py` advance fixtures. |
| `artboard/dirt.rs` | `B6-0094`, `B6-0123`, `B6-0258`, `B6-0331`, `B6-0352`, `B6-0355`, `B6-0357`, `B6-0361`, `B6-0362`, `B6-0365`, `B6-0368`, `B6-0385`, `B6-0388`, `B6-0399`, `B6-0405`, `B6-0408` | Move `component.dirt`, `component.dependents` (`pub fn add_dirt(`), and transform/dirt lifecycle citations as appropriate. Repoint `summarize_trace.py`'s two add-dirt anchors, dirt consumptions, owner resolutions, and skin-buffer scans according to their final physical locations. Repoint matching gap rows. |
| `artboard/update.rs` | `B6-0001`, `B6-0094`, `B6-0115`, `B6-0117`, `B6-0123`, `B6-0125`, `B6-0129`, `B6-0131`, `B6-0142`, `B6-0194`, `B6-0200`, `B6-0203`, `B6-0204`, `B6-0205`, `B6-0258`, `B6-0303`, `B6-0304`, `B6-0305`, `B6-0322`, `B6-0326`, `B6-0331`, `B6-0352`, `B6-0355`, `B6-0357`, `B6-0361`, `B6-0362`, `B6-0365`, `B6-0368`, `B6-0385`, `B6-0388`, `B6-0399`, `B6-0404`, `B6-0405`, `B6-0408` | Move `component.update_order` (`fn update_components_with_hook_recording`) and `artboard.update_pass` (`pub fn update_pass(`) `rust_file` values and lifecycle citations. Repoint retained update/settlement gap rows, trace scans, and test fixtures. |
| `artboard/callbacks.rs` | `B6-0059`, `B6-0061`, `B6-0063`, `B6-0064`, `B6-0077`, `B6-0094`, `B6-0123`, `B6-0125`, `B6-0129`, `B6-0131`, `B6-0142`, `B6-0194`, `B6-0200`, `B6-0258`, `B6-0303`, `B6-0304`, `B6-0305`, `B6-0331`, `B6-0352`, `B6-0355`, `B6-0357`, `B6-0361`, `B6-0362`, `B6-0365`, `B6-0368`, `B6-0385`, `B6-0388`, `B6-0399`, `B6-0404`, `B6-0405`, `B6-0408` | Change `check_layout_style_handlers.py`'s `RUST_ARTBOARD_MODULE` to this fragment if both display-change markers remain here; otherwise make the checker accept the two explicit physical files. Repoint callback lifecycle citations, gap rows, and fixtures. |
| `artboard/nested.rs` | `B6-0059`, `B6-0061`, `B6-0063`, `B6-0064`, `B6-0077`, `B6-0094`, `B6-0095`, `B6-0258`, `B6-0303`, `B6-0304`, `B6-0305` | Repoint nested-artboard advance/layout citations, gap allow entries, and nested fixtures. |
| `artboard/tests.rs` | None. `rust_attribution.py` excludes a source whose stem is `tests`. | Move the artboard test body unchanged. Update the seven relative `include_bytes!`/fixture paths currently near old lines 19,951, 20,001, 20,168, 20,542, 20,597, 20,612, and 23,311 because the file becomes one directory deeper. Prefer `concat!(env!("CARGO_MANIFEST_DIR"), "/...")` for fixture stability. The `env!` use near old line 14,290 remains valid. |

#### Global artboard bindings

These bindings span more than one fragment and therefore belong in the same PR:

- `rust_attribution.py` discovers all production `.rs` files below `crates/nuxie-runtime/src`. Every new production fragment must be listed in at least one `file-correspondence-manifest.toml` row and must **not** be added to `rust-additions.toml`. The checker exempts `tests.rs`, names ending in `_tests`, and files below a `tests/` path.
- The manifest's scatter ratchet is already at its maximum: 155 rows currently have multiple Rust modules. Except for `B6-0067`, all affected artboard rows are already multi-module. Replacing the `B6-0067` root path prevents the split from increasing that count.
- Update every affected C17 note that enumerates retained Rust modules. Do not change the underlying parity classification merely because the code's physical address changed.
- `docs/runtime-frame-loop-ownership.toml` has artboard citations in `focus.retained_tree`, `component.identity`, `component.dirt`, `component.dependents`, `component.update_order`, `component.transforms`, `component.clone_drop`, `artboard.advance`, `artboard.update_pass`, `nested_artboard.advance`, `artboard.live_draw`, and `renderer.interface_boundary`. Keep `component.clone_drop` and its `pub struct ArtboardInstance` anchor on the root; change each other `rust_file` or `rust:path:line-range` to its physical fragment.
- `docs/runtime-frame-loop-gaps.toml` has 25 owner-boundary allow entries pointing at `artboard.rs` (entries 0-7, 11-22, and 29-34 in current order). Repoint each to the fragment that contains its anchor. A literal move should preserve its token-based `site_hash`; a changed hash is a signal that the split was not mechanical.
- `tools/runtime-frame-loop-port/summarize_trace.py` hard-codes `artboard.rs` nine times for add-dirt, dirt consumption, owner resolution, dependency build, skin rebuild, advancing dispatch, resetting dispatch, and IK-chain discovery. Give each query its actual fragment path. The demangled owner should remain `artboard::ArtboardInstance` because `include!` does not introduce a child module.
- `tools/runtime-frame-loop-port/check_layout_style_handlers.py` hard-codes the artboard source while looking for `propagate_layout_component_display_changed` and the display property key. Point it to the fragment(s) containing those markers.
- `tools/runtime-frame-loop-port/test_check.py` contains at least 14 exact artboard paths covering probe gating, layer occurrence, live-advance ratchets, registry site hashes/substitution, and trait-anchor behavior. Update each synthetic fixture to the fragment that owns the tested anchor; do not perform a blind string replacement.
- Leave the existing root-level includes (`nested_artboard.rs`, `artboard_component_list.rs`, `artboard_list_map_rule.rs`, `artboard_referencer.rs`, `bindable_artboard.rs`, `nested_artboard_layout.rs`, `nested_artboard_leaf.rs`, `component_origin.rs`, bone/weight, profiler/profile) where they are. Moving them under `artboard/` changes relative include resolution and correspondence ownership.
- Keep `crates/nuxie-runtime/src/lib.rs`'s `mod artboard;` unchanged. The public and crate-private paths must not acquire an extra `artboard::<fragment>` segment.

### Mechanical PR sequence and proof

One PR should contain only this file's split and the path/anchor bookkeeping above. Recommended commit sequence within that PR:

1. Extract `tests.rs`, fix only its fixture paths, and prove the same 205 tests are collected.
2. Extract production regions in original order using `include!`; make no renames, visibility edits, formatting sweeps, or logic edits.
3. Update correspondence C17/path fields and all frame-loop physical anchors.
4. Update checker source locations and synthetic fixtures, including an assertion that demangled ownership remains `artboard::ArtboardInstance`.

Proof battery:

```text
make rust-sources-fresh
make cpp-probe
make runtime-frame-loop-port-test
make runtime-frame-loop-port-check
make rust-attribution-check
cargo check --workspace
cargo test -p nuxie-runtime
cargo test -p nuxie --features scripting
make scripted-golden-compare
make silver-corpus-test
```

The PR is structure-preserving only if parity row counts/statuses, trace landmarks, owner-boundary hashes, test counts, and golden/silver outputs are unchanged.

---

## 2. Large-file split plan: `state_machine_instance.rs`

### Current structural map

| Lines | Region |
| ---: | --- |
| 1-93 | Imports and ownership commentary |
| 94-255 | Nested event-chain/notify phases, trace structures, and trace recording |
| 256-312 | Semantic-node resolver and audio/event seam types |
| 313-1,271 | `RuntimeHitResult`, `HitComponent`, and hierarchy implementations |
| 1,272-1,467 | Nested notifier, focus, semantic, and state helper types |
| 1,468-1,474 | `RuntimeDataContextBindError` |
| 1,475-1,711 | `StateMachineInstance` storage |
| 1,712-1,744 | Data-bind occurrence/pointer/executor structures |
| 1,745-2,147 | Listener-action executor implementations |
| 2,148-2,335 | `Clone for StateMachineInstance` |
| 2,336-2,356 | `Drop for StateMachineInstance` |
| 2,357-2,638 | Free helpers and view-model listener types |
| 2,639-13,938 | Main `impl StateMachineInstance` |
| 13,939-13,953 | `runtime_owned_font` helper |
| 13,954-14,079 | `view_model_listener_tests` (`1` test) |
| 14,080-21,674 | `scripted_listener_action_tests` (`93` tests) |

The main implementation divides as follows:

| Lines | Logical responsibility |
| ---: | --- |
| 2,639-2,932 | Teardown, enrollment, focus manager, clone rebind |
| 2,933-3,741 | Construction and listener/layer initialization |
| 3,742-5,102 | Scripting, hydration, and adoption |
| 5,103-6,007 | State/input/focus/semantic/keyboard/gamepad operations |
| 6,008-7,487 | Hit/listener/pointer pipeline |
| 7,488-8,525 | Event notification, deferred callbacks, listener actions |
| 8,526-9,100 | Direct bind targets and value readers |
| 9,101-10,748 | Default/imported/owned source setters and relinking |
| 10,749-12,599 | Data-context bind/rebind, listener cells, bind updates |
| 12,600-13,938 | Reports, advance/apply, nested event dispatch, settlement |

### Proposed physical layout

```text
src/state_machine/
  state_machine_instance.rs               owner/types, hit hierarchy, lifecycle, ordered include! list
  state_machine_instance/
    construction.rs                       current 2639-3741
    scripted_objects.rs                   current 3742-5102
    input_focus.rs                        current 5103-6007
    pointer.rs                            current 6008-7487
    events_actions.rs                     current 7488-8525
    bind_targets.rs                       current 8526-9100
    bind_sources.rs                       current 9101-10748
    data_context.rs                       current 10749-12599
    advance.rs                            current 12600-13953
    tests/
      view_model_listener.rs              body of current first test module
      scripted_listener_actions.rs         body of current second test module
```

Retain the two existing module names in the root and include only their bodies. This preserves test paths and any name-sensitive filtering:

```rust
#[cfg(test)]
mod view_model_listener_tests {
    include!("state_machine_instance/tests/view_model_listener.rs");
}
```

Use the same pattern for `scripted_listener_action_tests`.

For every production file in this layout, place `include!("state_machine_instance/<name>.rs");` in the root at the original region boundary. Do not use `#[path]` or a `mod` declaration. Both test bodies remain inside their original outer modules and use literal includes into the `tests/` directory, which is what preserves their module names while exempting the physical leaves from attribution.

### Lockstep correspondence and checker manifest

| New file | B6 rows that must name it | Frame-loop/checker bindings that must move with it |
| --- | --- | --- |
| `state_machine_instance/construction.rs` | `B6-0077`, `B6-0194`, `B6-0195`, `B6-0196`, `B6-0200`, `B6-0310` | Repoint construction/enrollment lifecycle citations and matching synthetic fixtures. Keep `state_machine.collections`' `pub struct StateMachineInstance` anchor on the root. |
| `state_machine_instance/scripted_objects.rs` | `B6-0077`, `B6-0175`, `B6-0194`, `B6-0195`, `B6-0196`, `B6-0200` | Repoint scripting/hydration lifecycle citations and scripted-action fixture paths. |
| `state_machine_instance/input_focus.rs` | `B6-0077`, `B6-0083` | Repoint focus/semantic/keyboard/gamepad citations and tests. |
| `state_machine_instance/pointer.rs` | `B6-0077`, `B6-0083` | Repoint hit/pointer/listener lifecycle citations and FL-C4/layer/hit fixtures. |
| `state_machine_instance/events_actions.rs` | `B6-0058`, `B6-0077`, `B6-0440` | Repoint event reporting, firing/bubbling, and action-dispatch citations/fixtures. `state_machine.actions`' primary anchor remains on the root if the listener-action executor stays there. |
| `state_machine_instance/bind_targets.rs` | `B6-0058`, `B6-0194`, `B6-0196`, `B6-0200`, `B6-0440` | Repoint bind-target and view-model listener citations/fixtures. |
| `state_machine_instance/bind_sources.rs` | `B6-0058`, `B6-0175`, `B6-0194`, `B6-0195`, `B6-0196`, `B6-0200`, `B6-0440` | Repoint source/relink/data-converter citations and bind fixtures. |
| `state_machine_instance/data_context.rs` | `B6-0058`, `B6-0077`, `B6-0194`, `B6-0195`, `B6-0196`, `B6-0200`, `B6-0440` | Repoint data-context lifecycle citations and listener-cell/bind fixtures. |
| `state_machine_instance/advance.rs` | `B6-0077`, `B6-0194`, `B6-0195`, `B6-0196`, `B6-0200`, `B6-0310`, `B6-0440` | Move `state_machine.advance`'s `rust_file` and `advance_with_report_mode` anchor here. Repoint state-machine event/action/reporting lifecycle citations and advance/keyframe fixtures. Include `runtime_owned_font` in this fragment to avoid an artificial one-function file. |
| `state_machine_instance/tests/view_model_listener.rs` | None; it resides below a `tests/` path. | Move the body unchanged and preserve the outer module name. |
| `state_machine_instance/tests/scripted_listener_actions.rs` | None; it resides below a `tests/` path. | Move the body unchanged. Replace or correct the relative fixture path near old line 19,011 (`../../../../fixtures/sync/vm_listener_fire_event.riv`) because the source becomes two directories deeper; prefer a `CARGO_MANIFEST_DIR`-anchored path. |

#### Global state-machine bindings

- All production fragments must be named by `file-correspondence-manifest.toml`, never `rust-additions.toml`. Every affected SMI row is already multi-module, so this split does not increase the manifest's 155-row scatter count.
- Update C17 retained-module notes for `B6-0058`, `B6-0077`, `B6-0083`, `B6-0175`, `B6-0194`, `B6-0195`, `B6-0196`, `B6-0200`, `B6-0310`, and `B6-0440`; leave the substantive parity assertions untouched.
- `docs/runtime-frame-loop-ownership.toml` cites this file from `focus.retained_tree`, `state_machine.collections`, `state_machine.advance`, `state_machine.events`, `state_machine.actions`, and `event.reporting`. Keep root anchors for the struct, stored event collection, and action executor. Repoint method/lifecycle citations to physical fragments.
- `tools/runtime-frame-loop-port/check.py` currently treats one hard-coded SMI file as the nested-event owner and excludes it from outside-owner scans. Once methods are in included fragments, a path-only exclusion would falsely flag the fragments. Make owner discovery include-aware: recursively expand literal local `include!("...")` files in source order for export analysis, and treat every expanded file as the same logical owner. Do not broaden the exclusion to a directory glob.
- Add checker unit coverage for the include-aware logical owner: an exported nested-event method in an included fragment must be accepted, while the same method in an unrelated file must still fail. Preserve syntax parsing and duplicate-anchor detection.
- `tools/runtime-frame-loop-port/test_check.py` has at least 14 exact SMI references, plus constructed/split path strings, covering FL-C4, lifecycle, layer, hit, bind, events, firing/bubbling, advance, owner exports, keyframes, focus, and semantic behavior. Point each fixture to the actual fragment; do not simply substitute the root path everywhere.
- `docs/runtime-frame-loop-gaps.toml` has no current SMI owner-boundary allow row to migrate.
- Keep `crates/nuxie-runtime/src/state_machine.rs`'s `pub(crate) mod state_machine_instance;` and its re-exports unchanged. Literal includes keep all symbols at the original module path.

### Mechanical PR sequence and proof

This should be a second, independent PR after the artboard split has landed. It uses the same proof battery. The checker must first learn the logical-owner/include relationship in the same PR, because moving the methods without that update creates false ownership failures. Apart from that narrowly required path-awareness, no checker policy should change.

Success means the same 94 tests are collected, no exported API path changes, the correspondence scatter count stays at 155, no new owner-boundary allowance appears, and the full battery shown in the artboard plan remains green.

---

## 3. Nuxie-only seam inventory

### Classification method

The authoritative starting point is correspondence metadata:

- Production Rust paths mapped to upstream C++ in `file-correspondence-manifest.toml` are port-side unless a smaller product-only region can be proven.
- Production Rust paths listed in `rust-additions.toml` are intentional Rust-only additions. The relevant ownership labels are `flowsession-abi`, `scene-api`, and generated/codegen infrastructure.
- `build.rs` is outside the attribution scanner's `src/` roots, so it must be inspected manually.
- A convenience façade is not automatically product-side. Thin Rust-only namespace/generated-storage coordinators that serve the port stay with the port even though no one C++ file corresponds to them.

This prevents two opposite mistakes: burying product authoring features inside the port forever, and extracting Rust implementation infrastructure merely because its file has no one-to-one C++ source.

### Product features and mixed glue

| Region | Current size and location | Runtime internals reached / extraction surface | Placement assessment |
| --- | --- | --- | --- |
| FlowSession wire/domain model | `crates/nuxie/src/flow_session.rs:21-621` (601 lines) | Public `File`, `Factory`, renderer, artboard, view-model, and state-machine types | Clean product vocabulary; move to `nuxie-flow` after dependency direction is fixed. |
| FlowSession orchestration | `flow_session.rs:622-2163` (1,542) | Calls crate-private script preparation, listener preparation, rehydration, apply/rollback, host-cycle, and command-drain methods on the `nuxie` façade | Product-side, but requires a narrow public/sealed flow-runtime bridge before crate extraction. Moderate coupling. |
| FlowSession value snapshots | `flow_session.rs:2164-3105` (942) | Traverses public runtime-owned view-model/value objects and builds its own graph/arena | Product-side and cohesive. Mostly mechanical after bridge creation. |
| FlowSession validation/accounting | `flow_session.rs:3106-3545` (440) | Product protocol limits and payload validation | Clean extraction candidate. |
| FlowSession mutation/apply/diff | `flow_session.rs:3546-4515` (970) | Applies mutations through runtime view-model/state-machine handles; shares rollback/command sequencing with façade-private methods | Product-side; transactional sequencing makes it medium risk. |
| FlowSession selection/catalog | `flow_session.rs:4516-5020` (505) | File/artboard/state-machine discovery | Product-side; mostly public runtime surface. |
| FlowSession tests | `flow_session.rs:5021-7227` (2,207; 40 tests), plus `crates/nuxie/tests/flow_session_contract.rs` (335) | Exercises the public ABI, VM lifecycle, and transactional behavior | Move with FlowSession. Preserve public names through compatibility re-exports during migration. |
| Scene contract and IDs | `crates/nuxie/src/scene.rs:28-1187` (1,160) | Public runtime/binary identities and errors | Product-side, cleanly demarcatable. |
| Scene durable definitions/index/specs | `scene.rs:1188-3401` (2,214) | `nuxie_binary::AuthoringRecord`, property/value types, runtime file identities | Product-side authoring model. |
| Scene hierarchy/materialization | `scene.rs:3402-5622` (2,221) | Constructs/reads `RuntimeFile`; directly touches `materialized.file.runtime` in a handful of places even though `File::runtime()` already exists | Product-side but high coupling. Replace direct private field access with the existing public accessor before moving. |
| Silver/scene export schema | `scene.rs:5623-6819` (1,197) | Public exported-record/document DTOs; lowering later feeds `RuntimeFile::from_authoring_records` | Product-side. The schema and its generated counterpart must move together. |
| Scene store/live mount API | `scene.rs:6820-9099` (2,280) | Owns mounted `File`/artboard/view-model state and coordinates transactions | Product-side; medium-to-high coupling. |
| SceneTx and subordinate transactions | `scene.rs:9100-15589` (6,490) | `SceneTx`, `DataConverterTx`, `VmTx`, `AnimTx`, and `MachineTx` mutate the authoring graph, then materialize into runtime/binary types | Core product authoring seam. Cohesive, but atomic commit/materialization behavior makes the move high risk. |
| Scene frame queries/live mutation | `scene.rs:15590-17592` (2,003) | Hit/geometry/semantic queries and intrinsic image sizing reach crate-private `OwnedArtboardInstance` helpers | Product-side. Needs stable snapshot/command DTOs instead of exposing mutable runtime internals. |
| Scene lowering/validation/export | `scene.rs:17593-25563` (7,971) | Converts authoring records into `RuntimeFile`, validates cross-object invariants, and produces exported documents | Product-side. Highest-risk region because it couples the authoring model to port construction invariants. |
| Scene tests | `scene.rs:25564-38224` (12,661; 120 tests), plus `crates/nuxie/tests/scene_authoring.rs` (15,674) and related authoring tests | Broad transaction, materialization, export, and live-runtime contract | Move with Scene; use this suite as the seam migration oracle. |
| Scene schema generator | `crates/nuxie/build.rs` (4,531) and `scene.rs`'s `include!(concat!(env!("OUT_DIR"), "/scene_schema.rs"))` near line 860 | Generates the Scene/silver record schema at build time | Product-side and inseparable from Scene. Extraction must move the build script/generator inputs and preserve the generated include contract. |
| Script import authorization | `crates/nuxie/src/script_import.rs:1-157` production, `158-211` tests | Ed25519/container verification; calls crate-private `File::execution_authorization_for`/authorization state | Product policy. Move to `nuxie-flow`, but expose a capability-based import seam rather than making raw authorization state public. |
| Mixed façade bridge | `crates/nuxie/src/lib.rs`, principally project-envelope classification and private bridge methods around lines 55, 154, 223-258, 307, 344, 376, 1,492, 2,983, 3,328, 4,304, and 5,600-5,733 | `File::from_runtime`; flow script/listener lifecycle; apply/rollback/command drain; artboard geometry/semantic/intrinsic-image helpers | Not an independent feature. Convert these call sites into explicit narrow bridge traits/DTOs, then move their product consumers. |

The concrete crate-private methods the extraction must account for are:

- Flow lifecycle around `nuxie/src/lib.rs:5600-5675`: `prepare_flow_scripts`, `prepare_flow_listener_actions`, rehydrate/apply, begin host cycle, rollback, and drain commands.
- Scene live/runtime queries around `lib.rs:5705-5733`: hit-test path segments with bounds, geometry path segments, retained/semantic text, and intrinsic image-dimension registration.
- `File::from_runtime` around `lib.rs:4304` and the script execution authorization capability.

These are already cross-crate calls from `nuxie` into public `nuxie-runtime` types; the missing visibility is chiefly inside the `nuxie` façade. Extraction should expose a sealed product bridge from the future base façade, not expand dozens of runtime fields to `pub`.

### Project-data converter: product model inside the port

`crates/nuxie-runtime/src/project_data_converter.rs` is 2,686 lines and is explicitly classified `scene-api` in `rust-additions.toml`:

| Lines | Region | Coupling and disposition |
| ---: | --- | --- |
| 22-507 | Public values, specs, errors, and state | Pure product/protocol model; eventual neutral-crate candidate. |
| 508-1,289 | Envelope/program/catalog compilation and evaluation | Mostly `std`/`serde`; eventual neutral-crate candidate. |
| 1,290-2,324 | Validation, coercion, formatting, interpolation | Pure-looking core, but behavior is part of runtime data-bind semantics. Extract only with parity tests. |
| 2,325-2,686 | Formula parser | Mechanically separable after the outer seam is established. |

The file is locally self-contained, but its runtime integration is not. `data_bind/context/context_value.rs` has a `RuntimeDataBindGraphConverter::Project` variant and owns conversion/evaluator state; `data_bind/data_bind_context.rs` constructs it from a script-asset envelope; both runtime and `nuxie` re-export or classify its types; Scene authors and lowers them. A direct move to `nuxie-flow` would invert the desired dependency and make the port depend on the product crate.

Two crate-private numeric-policy helpers in this file (the bounded-list constant near line 467 and helper near line 2,320) are also used by port-side Number-to-List/context conversion. If the model is eventually extracted, those policies should remain in the port or move to a dependency-neutral shared crate; they should not become a product API solely to satisfy the move.

Recommendation: first demarcate this as `project_data_converter/{model, program, evaluator, bridge}` inside `nuxie-runtime`; later extract only the pure model/program/evaluator to a neutral `nuxie-project-data` crate that both runtime and `nuxie-flow` may depend on. The runtime adapter stays in `nuxie-runtime`.

### Rust-only infrastructure that should remain port-side

The following `rust-additions.toml` entries have no one-file C++ correspondence but are not product features:

| Current path/group | Lines | Why it stays in `nuxie-runtime` |
| --- | ---: | --- |
| `data_bind_container.rs`, `data_converter.rs`, `retained_data_bind.rs` | 12 total | Tiny generated/type façade modules over port functionality. |
| `input/mod.rs`, `math/mod.rs`, `shapes/mod.rs`, `shapes/paint/mod.rs` | 174 total | Rust namespace/generated-storage coordinators. |
| `objects.rs` | 1,018 | Runtime object registry plus `include!(concat!(env!("OUT_DIR"), "/runtime_objects.rs"))`; generated by `crates/nuxie-runtime/build.rs` (707 lines). This is port construction infrastructure. |
| `properties.rs` | 826 | Generated property interpretation/storage used throughout the port. |
| `state_machine.rs`, `state_machine/instance.rs`, `state_machine/listener_types/mod.rs` | 215 total | Rust module/re-export façade for the ported state-machine implementation. |
| `viewmodel/mod.rs`, `viewmodel/runtime/mod.rs` | 58 total | Rust namespace/re-export façade for ported view-model behavior. |
| `focus.rs` | 7 | Thin public façade over retained focus behavior, not a standalone product feature. |

These codegen/facade additions total about 2,303 source lines excluding the project converter and focus façade. They should be clearly marked as Rust port infrastructure, but moving them to `nuxie-flow` would blur rather than improve the seam.

`crates/nuxie/src/command_queue.rs`, server code, and raw-text support are not included in the Nuxie-only list: their current files are explicitly mapped to upstream C++ correspondence. A richer Rust API around a ported facility does not by itself make that facility product-side.

---

## 4. Recommended seam design

### Decision

Use a **module boundary now and a crate boundary later**.

The immediate structure should make product ownership visible without changing dependencies:

```text
crates/nuxie/src/product/
  flow_session/
  scene/
    model/
    transaction/
    live/
    export/
  script_import/

crates/nuxie-runtime/src/project_data_converter/
  model.rs
  program.rs
  evaluator.rs
  bridge.rs
```

Preserve current public paths with root re-exports. This first step is a namespace/demarcation change, not an extraction. It gives reviewers an auditable boundary while the current test and parity baseline still observes exactly the same crate graph.

The long-term crate graph should avoid a cycle:

```text
nuxie-runtime  <---  nuxie-core  <---  nuxie-flow
       ^                 ^                 ^
       |                 |                 |
       +------ nuxie-project-data --------+

nuxie (umbrella compatibility crate) re-exports nuxie-core + nuxie-flow
```

`nuxie-flow` contains FlowSession, Scene/SceneTx, silver/export DTOs and lowering, and script-import policy. `nuxie-core` contains `File`, `Factory`, owned artboard/view-model handles, renderer integration, and narrow bridge traits. `nuxie-runtime` remains the C++-correspondence port plus explicitly labeled Rust port infrastructure. A small dependency-neutral `nuxie-project-data` is optional and should be introduced only after the runtime adapter has been separated from the pure evaluator.

### Public surface of the port/base layer

The extraction should not make arbitrary runtime fields public. The required surface is narrow:

1. A sealed/trusted constructor that wraps a validated `RuntimeFile` as a façade `File`, with an explicit authoring/import authorization policy.
2. A flow-runtime bridge for prepare, rehydrate, apply, host-cycle, rollback, and command-drain operations. Its inputs/outputs should be stable value DTOs, not references to internal queues.
3. Read-only artboard snapshot operations for hit geometry, retained geometry, and semantic text, plus a command for intrinsic image dimensions. Return owned snapshots so Scene cannot retain internal graph borrows.
4. A project-converter adapter owned by `nuxie-runtime`; pure program/model types may come from the neutral crate. Do not expose the runtime bind graph or evaluator cells as product API.
5. Existing upstream-corresponding runtime types only where they are already public and semantically stable. Product re-exports should live in `nuxie`/`nuxie-flow`, not expand the port's promise.

### Migration order

1. Establish the exact green parity/golden baseline and keep it pinned in every PR.
2. Land the `artboard.rs` and `state_machine_instance.rs` mechanical splits as separate PRs. This makes later ownership changes reviewable without mixing 20,000-line source moves into semantic work.
3. Demarcate `nuxie::product::{flow_session, scene, script_import}` and the project-converter submodules in place, preserving public re-exports.
4. Define the sealed flow, authoring-file, geometry-snapshot, and script-authorization bridges; migrate existing internal callers to them while everything remains in the same crate.
5. Extract the non-product façade/runtime handles into `nuxie-core`, with `nuxie` temporarily acting as an umbrella compatibility crate.
6. Move FlowSession and script import to `nuxie-flow`; move their tests and retain compatibility re-exports.
7. Move Scene model, SceneTx, export schema/generator, lowering, live adapter, and tests together. Do not split the generated schema from `build.rs` or split transaction commit from materialization in the extraction PR.
8. Isolate the project converter's pure model/program/evaluator. Extract it to a neutral crate only if doing so leaves the runtime adapter dependency pointing inward, never from `nuxie-runtime` to `nuxie-flow`.
9. Remove compatibility re-exports and tighten visibility in a later breaking-change PR, after downstream migration.

### Risk map

| Risk | Areas | Reason |
| --- | --- | --- |
| Mechanical/low | Thin namespace façades, codegen coordinators staying in place, test-module extraction, pure FlowSession validation/types | Mostly physical ownership with no runtime mutation semantics. |
| Moderate | FlowSession after bridge creation, script import after capability API, Scene DTO/export types | Public API and authorization compatibility, but limited graph mutation. |
| High | SceneTx commit/materialization, live Scene mounting, geometry/semantic access, Scene lowering and generated schema | Atomicity, identity, generated code, and live runtime invariants cross the seam. |
| High | Project-data converter extraction | The source looks pure, but evaluator state is embedded in runtime data-bind context and Number-to-List policy. |
| High | Script/listener ownership during FlowSession moves | Rehydration, rollback, command queues, and VM object lifetime must remain one coherent transaction. |

### Why baseline-first matters

`docs/parity-gap-register.md` explicitly recommends making the parity claim trustworthy before feature work and landing each subsequent change against the oracle (the “Recommended execution order” around lines 204-211). It also describes Scene as a Nuxie-only additive API (around lines 126-127) while recording FlowSession/capability-boundary gaps elsewhere in the register. That is the governing principle here: first make source moves mechanically observable by the existing correspondence, ownership, trace, golden, and silver batteries; then introduce the product seam behind those same oracles. A green build alone is insufficient because it can preserve compilation while silently weakening ownership scans or changing what “zero divergences” measures.

The two large-file splits are therefore prerequisites for, not part of, the product extraction. Each split keeps the logical Rust module unchanged and updates only physical-address consumers. The seam work then proceeds through explicit bridges and compatibility re-exports, with risky transaction/materialization changes isolated from mechanical moves.
# nuxie-runtime structure report

## Scope and baseline

This is a read-only structure analysis. It proposes no behavior, source, parity-ledger, or ownership-ledger changes.

The inspected worktree was already detached at `e8726db8ffc689d3b19f1c0a55794aa6daf9956d` (`Merge pull request #246 from TheOneLevin/feat/b6-0058-listener-viewmodel-change`, 2026-08-04). The local `origin/main` ref resolves to the same commit. The requested `git fetch origin` and redundant `git checkout --detach origin/main` could not update the shared worktree metadata because the sandbox cannot write the parent repository's `.git/worktrees/nuxie-mr-c12` directory. The analysis is therefore pinned to the locally available `origin/main` commit above, not a newly fetched remote ref. The pre-existing untracked `vtriage-capture/` directory was left untouched.

The two target files have grown beyond the sizes in the request:

| File | Current lines | Production/support | Tests |
| --- | ---: | ---: | ---: |
| `crates/nuxie-runtime/src/artboard.rs` | 23,374 | 12,039 | 11,335 (`205` tests) |
| `crates/nuxie-runtime/src/state_machine/state_machine_instance.rs` | 21,674 | 14,079 | 7,595 (`94` tests) |

The safest split mechanism is literal `include!` fragments, not Rust child modules. An included fragment remains in the original semantic module (`crate::artboard` or `crate::state_machine::state_machine_instance`), so private-name resolution, inherent method paths, sibling access, and demangled symbols remain unchanged. A `mod build;`/`#[path = ...] mod build;` split would introduce new privacy boundaries and make a supposedly mechanical PR into an API refactor.

---

## 1. Large-file split plan: `artboard.rs`

### Current structural map

| Lines | Region |
| ---: | --- |
| 1-106 | Imports and supporting aliases |
| 107-135 | Draw-frame counter and generated `Mat2D` helper |
| 136-160 | `ExternalFontAssetError` |
| 161-181 | `RuntimeArtboardInstanceIdentity` |
| 182-231 | `RuntimeScriptState` and implementations |
| 232-250 | Advancing-component and persistent-dirt fixtures |
| 251-298 | Script advance/update mode enums and implementations |
| 299-327 | Resetting/runtime-component helper types |
| 328-334 | `RuntimeTextStyleFeatureOption` |
| 335-523 | `ArtboardInstance` storage |
| 524-714 | `Clone for ArtboardInstance` |
| 715-739 | `Drop for ArtboardInstance` followed by the existing leaf `include!` declarations |
| 740-773 | Occurrence and frame-advance report types |
| 774-844 | Build context/index/text-flag helpers |
| 845-860 | `RuntimeNestedAnimationInstance` |
| 861-11,133 | Main `impl ArtboardInstance` |
| 11,134-11,147 | Focus-scroll path and focus-bounds transform helper |
| 11,148-11,187 | Small follow-up `impl ArtboardInstance` |
| 11,188-11,226 | Component-list reset helper |
| 11,227-11,595 | `impl RuntimeNestedArtboardInstance` |
| 11,596-11,611 | `impl RuntimeNestedAnimationInstance` |
| 11,612-12,039 | Free construction/nested-artboard helpers |
| 12,040-23,374 | Single `#[cfg(test)] mod tests` (`205` tests) |

The main implementation itself divides along stable responsibility boundaries:

| Lines | Logical responsibility |
| ---: | --- |
| 861-1,104 | Lifecycle, data context, and transient-clone restoration |
| 1,105-2,440 | Build occurrence relationships and constructors |
| 2,441-2,712 | Path/hit initialization and external assets |
| 2,713-3,399 | Scripting lifecycle |
| 3,400-3,989 | Object/component/audio/hit/graph/definition access |
| 3,990-4,152 | Scripted interpolators and state-machine definition selection |
| 4,153-4,480 | Dimensions, world/layout bounds, scrolling, and focus reveal |
| 4,481-5,229 | Generated property façade and text values |
| 5,230-5,568 | Animation/state-machine/view-model occurrences |
| 5,569-6,312 | Component-list lifecycle and layout |
| 6,313-7,408 | Root/nested frame advance and retained-component dispatch |
| 7,409-7,585 | Nested-layout frame propagation |
| 7,586-8,496 | Transforms, epochs, dirt, and invalidation |
| 8,497-9,191 | Update pass and five-pass state-machine settlement |
| 9,192-9,825 | Component update dispatch and script scheduling |
| 9,826-11,133 | Property-change callbacks, nested controls, and collapse |

### Proposed physical layout

Keep `artboard.rs` as the owner. It retains imports, supporting types, `ArtboardInstance`, `Clone`, `Drop`, the existing leaf includes, occurrence/report/build helper types, and an ordered series of literal includes. This preserves declaration order where it matters and makes all fragments part of `crate::artboard`.

```text
src/
  artboard.rs                         owner/types, existing leaf includes, ordered include! list
  artboard/
    build.rs                          current 861-2712
    script_runtime.rs                 current 2713-3399
    access.rs                         current 3400-3989
    definitions_and_layout.rs         current 3990-4480
    properties.rs                     current 4481-5568
    component_lists.rs                current 5569-6312
    advance.rs                        current 6313-7585
    dirt.rs                           current 7586-8496
    update.rs                         current 8497-9825
    callbacks.rs                      current 9826-11133
    nested.rs                         current 11134-12039
    tests.rs                          body of current tests module
```

The production fragments are intentionally in execution/dependency order rather than alphabetical order. `tests.rs` should be included inside the original test module:

```rust
#[cfg(test)]
mod tests {
    include!("artboard/tests.rs");
}
```

This retains the test module's name and access. It also gives the attribution checker a deliberately exempt `tests.rs` leaf.

For every production file in the layout above, the owner must contain a literal `include!("artboard/<name>.rs");` at the point where that region previously appeared. Do not use `#[path]`, `mod`, or `pub(crate) mod`: each would create a child-module/privacy boundary. Every such production fragment is attribution-scanned and must be named by the B6 rows below; only `tests.rs` receives the test-source exemption.

### Lockstep correspondence and checker manifest

The following table is the required per-file edit manifest. “B6 rows” are `[[files]]` entries in `file-correspondence-manifest.toml`; add the fragment's path to those rows' semicolon-separated `rust_module` values and update their C17 retained-module notes. Keep status, upstream pins, C++ paths, verification text, and correspondence claims unchanged. The row list is conservative: a later provenance cleanup can narrow it, but narrowing must not be mixed into this mechanical split.

| New file | B6 rows that must name it | Frame-loop/checker bindings that must move with it |
| --- | --- | --- |
| `artboard/build.rs` | `B6-0001`, `B6-0094`, `B6-0115`, `B6-0117`, `B6-0123`, `B6-0125`, `B6-0129`, `B6-0131`, `B6-0142`, `B6-0146`, `B6-0202`, `B6-0258`, `B6-0303`, `B6-0304`, `B6-0305`, `B6-0331`, `B6-0361`, `B6-0362`, `B6-0365`, `B6-0368`, `B6-0385`, `B6-0388`, `B6-0399`, `B6-0405`, `B6-0408` | Repoint `summarize_trace.py` dependency-build and IK-chain-build source scans when their anchors land here. Repoint matching `runtime-frame-loop-gaps.toml` owner-boundary entries. |
| `artboard/script_runtime.rs` | `B6-0077`, `B6-0194`, `B6-0200`, `B6-0322`, `B6-0326` | Repoint the script-lifecycle citations in `runtime-frame-loop-ownership.toml` and any corresponding synthetic test fixture paths. |
| `artboard/access.rs` | `B6-0094`, `B6-0095`, `B6-0123`, `B6-0146`, `B6-0203`, `B6-0204`, `B6-0205` | Repoint any live-draw/renderer-interface lifecycle citations whose named methods move here. |
| `artboard/definitions_and_layout.rs` | `B6-0077`, `B6-0094`, `B6-0258`, `B6-0303`, `B6-0304`, `B6-0305`, `B6-0385`, `B6-0388` | Repoint focus-retained-tree and layout lifecycle citations, plus matching owner-boundary allow entries. |
| `artboard/properties.rs` | `B6-0067`, `B6-0077`, `B6-0094`, `B6-0194`, `B6-0200`, `B6-0352`, `B6-0355`, `B6-0385`, `B6-0388`, `B6-0399`, `B6-0404`, `B6-0405`, `B6-0408` | `B6-0067` currently has only one Rust module: **replace** `artboard.rs` with this fragment rather than adding a second path. Repoint property/text lifecycle citations and corresponding owner-boundary entries. |
| `artboard/component_lists.rs` | `B6-0095`, `B6-0258`, `B6-0303`, `B6-0304`, `B6-0305` | Repoint component-list and nested-layout lifecycle citations and matching test fixtures. |
| `artboard/advance.rs` | `B6-0001`, `B6-0059`, `B6-0061`, `B6-0063`, `B6-0064`, `B6-0077`, `B6-0094`, `B6-0095`, `B6-0194`, `B6-0200`, `B6-0258`, `B6-0303`, `B6-0304`, `B6-0305`, `B6-0322`, `B6-0326`, `B6-0388` | In `runtime-frame-loop-ownership.toml`, move `artboard.advance`'s `rust_file` and the lifecycle citations for artboard/nested advance. Repoint `summarize_trace.py` advancing/resetting dispatch scans as applicable. Repoint relevant gap allow rows and `test_check.py` advance fixtures. |
| `artboard/dirt.rs` | `B6-0094`, `B6-0123`, `B6-0258`, `B6-0331`, `B6-0352`, `B6-0355`, `B6-0357`, `B6-0361`, `B6-0362`, `B6-0365`, `B6-0368`, `B6-0385`, `B6-0388`, `B6-0399`, `B6-0405`, `B6-0408` | Move `component.dirt`, `component.dependents` (`pub fn add_dirt(`), and transform/dirt lifecycle citations as appropriate. Repoint `summarize_trace.py`'s two add-dirt anchors, dirt consumptions, owner resolutions, and skin-buffer scans according to their final physical locations. Repoint matching gap rows. |
| `artboard/update.rs` | `B6-0001`, `B6-0094`, `B6-0115`, `B6-0117`, `B6-0123`, `B6-0125`, `B6-0129`, `B6-0131`, `B6-0142`, `B6-0194`, `B6-0200`, `B6-0203`, `B6-0204`, `B6-0205`, `B6-0258`, `B6-0303`, `B6-0304`, `B6-0305`, `B6-0322`, `B6-0326`, `B6-0331`, `B6-0352`, `B6-0355`, `B6-0357`, `B6-0361`, `B6-0362`, `B6-0365`, `B6-0368`, `B6-0385`, `B6-0388`, `B6-0399`, `B6-0404`, `B6-0405`, `B6-0408` | Move `component.update_order` (`fn update_components_with_hook_recording`) and `artboard.update_pass` (`pub fn update_pass(`) `rust_file` values and lifecycle citations. Repoint retained update/settlement gap rows, trace scans, and test fixtures. |
| `artboard/callbacks.rs` | `B6-0059`, `B6-0061`, `B6-0063`, `B6-0064`, `B6-0077`, `B6-0094`, `B6-0123`, `B6-0125`, `B6-0129`, `B6-0131`, `B6-0142`, `B6-0194`, `B6-0200`, `B6-0258`, `B6-0303`, `B6-0304`, `B6-0305`, `B6-0331`, `B6-0352`, `B6-0355`, `B6-0357`, `B6-0361`, `B6-0362`, `B6-0365`, `B6-0368`, `B6-0385`, `B6-0388`, `B6-0399`, `B6-0404`, `B6-0405`, `B6-0408` | Change `check_layout_style_handlers.py`'s `RUST_ARTBOARD_MODULE` to this fragment if both display-change markers remain here; otherwise make the checker accept the two explicit physical files. Repoint callback lifecycle citations, gap rows, and fixtures. |
| `artboard/nested.rs` | `B6-0059`, `B6-0061`, `B6-0063`, `B6-0064`, `B6-0077`, `B6-0094`, `B6-0095`, `B6-0258`, `B6-0303`, `B6-0304`, `B6-0305` | Repoint nested-artboard advance/layout citations, gap allow entries, and nested fixtures. |
| `artboard/tests.rs` | None. `rust_attribution.py` excludes a source whose stem is `tests`. | Move the artboard test body unchanged. Update the seven relative `include_bytes!`/fixture paths currently near old lines 19,951, 20,001, 20,168, 20,542, 20,597, 20,612, and 23,311 because the file becomes one directory deeper. Prefer `concat!(env!("CARGO_MANIFEST_DIR"), "/...")` for fixture stability. The `env!` use near old line 14,290 remains valid. |

#### Global artboard bindings

These bindings span more than one fragment and therefore belong in the same PR:

- `rust_attribution.py` discovers all production `.rs` files below `crates/nuxie-runtime/src`. Every new production fragment must be listed in at least one `file-correspondence-manifest.toml` row and must **not** be added to `rust-additions.toml`. The checker exempts `tests.rs`, names ending in `_tests`, and files below a `tests/` path.
- The manifest's scatter ratchet is already at its maximum: 155 rows currently have multiple Rust modules. Except for `B6-0067`, all affected artboard rows are already multi-module. Replacing the `B6-0067` root path prevents the split from increasing that count.
- Update every affected C17 note that enumerates retained Rust modules. Do not change the underlying parity classification merely because the code's physical address changed.
- `docs/runtime-frame-loop-ownership.toml` has artboard citations in `focus.retained_tree`, `component.identity`, `component.dirt`, `component.dependents`, `component.update_order`, `component.transforms`, `component.clone_drop`, `artboard.advance`, `artboard.update_pass`, `nested_artboard.advance`, `artboard.live_draw`, and `renderer.interface_boundary`. Keep `component.clone_drop` and its `pub struct ArtboardInstance` anchor on the root; change each other `rust_file` or `rust:path:line-range` to its physical fragment.
- `docs/runtime-frame-loop-gaps.toml` has 25 owner-boundary allow entries pointing at `artboard.rs` (entries 0-7, 11-22, and 29-34 in current order). Repoint each to the fragment that contains its anchor. A literal move should preserve its token-based `site_hash`; a changed hash is a signal that the split was not mechanical.
- `tools/runtime-frame-loop-port/summarize_trace.py` hard-codes `artboard.rs` nine times for add-dirt, dirt consumption, owner resolution, dependency build, skin rebuild, advancing dispatch, resetting dispatch, and IK-chain discovery. Give each query its actual fragment path. The demangled owner should remain `artboard::ArtboardInstance` because `include!` does not introduce a child module.
- `tools/runtime-frame-loop-port/check_layout_style_handlers.py` hard-codes the artboard source while looking for `propagate_layout_component_display_changed` and the display property key. Point it to the fragment(s) containing those markers.
- `tools/runtime-frame-loop-port/test_check.py` contains at least 14 exact artboard paths covering probe gating, layer occurrence, live-advance ratchets, registry site hashes/substitution, and trait-anchor behavior. Update each synthetic fixture to the fragment that owns the tested anchor; do not perform a blind string replacement.
- Leave the existing root-level includes (`nested_artboard.rs`, `artboard_component_list.rs`, `artboard_list_map_rule.rs`, `artboard_referencer.rs`, `bindable_artboard.rs`, `nested_artboard_layout.rs`, `nested_artboard_leaf.rs`, `component_origin.rs`, bone/weight, profiler/profile) where they are. Moving them under `artboard/` changes relative include resolution and correspondence ownership.
- Keep `crates/nuxie-runtime/src/lib.rs`'s `mod artboard;` unchanged. The public and crate-private paths must not acquire an extra `artboard::<fragment>` segment.

### Mechanical PR sequence and proof

One PR should contain only this file's split and the path/anchor bookkeeping above. Recommended commit sequence within that PR:

1. Extract `tests.rs`, fix only its fixture paths, and prove the same 205 tests are collected.
2. Extract production regions in original order using `include!`; make no renames, visibility edits, formatting sweeps, or logic edits.
3. Update correspondence C17/path fields and all frame-loop physical anchors.
4. Update checker source locations and synthetic fixtures, including an assertion that demangled ownership remains `artboard::ArtboardInstance`.

Proof battery:

```text
make rust-sources-fresh
make cpp-probe
make runtime-frame-loop-port-test
make runtime-frame-loop-port-check
make rust-attribution-check
cargo check --workspace
cargo test -p nuxie-runtime
cargo test -p nuxie --features scripting
make scripted-golden-compare
make silver-corpus-test
```

The PR is structure-preserving only if parity row counts/statuses, trace landmarks, owner-boundary hashes, test counts, and golden/silver outputs are unchanged.

---

## 2. Large-file split plan: `state_machine_instance.rs`

### Current structural map

| Lines | Region |
| ---: | --- |
| 1-93 | Imports and ownership commentary |
| 94-255 | Nested event-chain/notify phases, trace structures, and trace recording |
| 256-312 | Semantic-node resolver and audio/event seam types |
| 313-1,271 | `RuntimeHitResult`, `HitComponent`, and hierarchy implementations |
| 1,272-1,467 | Nested notifier, focus, semantic, and state helper types |
| 1,468-1,474 | `RuntimeDataContextBindError` |
| 1,475-1,711 | `StateMachineInstance` storage |
| 1,712-1,744 | Data-bind occurrence/pointer/executor structures |
| 1,745-2,147 | Listener-action executor implementations |
| 2,148-2,335 | `Clone for StateMachineInstance` |
| 2,336-2,356 | `Drop for StateMachineInstance` |
| 2,357-2,638 | Free helpers and view-model listener types |
| 2,639-13,938 | Main `impl StateMachineInstance` |
| 13,939-13,953 | `runtime_owned_font` helper |
| 13,954-14,079 | `view_model_listener_tests` (`1` test) |
| 14,080-21,674 | `scripted_listener_action_tests` (`93` tests) |

The main implementation divides as follows:

| Lines | Logical responsibility |
| ---: | --- |
| 2,639-2,932 | Teardown, enrollment, focus manager, clone rebind |
| 2,933-3,741 | Construction and listener/layer initialization |
| 3,742-5,102 | Scripting, hydration, and adoption |
| 5,103-6,007 | State/input/focus/semantic/keyboard/gamepad operations |
| 6,008-7,487 | Hit/listener/pointer pipeline |
| 7,488-8,525 | Event notification, deferred callbacks, listener actions |
| 8,526-9,100 | Direct bind targets and value readers |
| 9,101-10,748 | Default/imported/owned source setters and relinking |
| 10,749-12,599 | Data-context bind/rebind, listener cells, bind updates |
| 12,600-13,938 | Reports, advance/apply, nested event dispatch, settlement |

### Proposed physical layout

```text
src/state_machine/
  state_machine_instance.rs               owner/types, hit hierarchy, lifecycle, ordered include! list
  state_machine_instance/
    construction.rs                       current 2639-3741
    scripted_objects.rs                   current 3742-5102
    input_focus.rs                        current 5103-6007
    pointer.rs                            current 6008-7487
    events_actions.rs                     current 7488-8525
    bind_targets.rs                       current 8526-9100
    bind_sources.rs                       current 9101-10748
    data_context.rs                       current 10749-12599
    advance.rs                            current 12600-13953
    tests/
      view_model_listener.rs              body of current first test module
      scripted_listener_actions.rs         body of current second test module
```

Retain the two existing module names in the root and include only their bodies. This preserves test paths and any name-sensitive filtering:

```rust
#[cfg(test)]
mod view_model_listener_tests {
    include!("state_machine_instance/tests/view_model_listener.rs");
}
```

Use the same pattern for `scripted_listener_action_tests`.

For every production file in this layout, place `include!("state_machine_instance/<name>.rs");` in the root at the original region boundary. Do not use `#[path]` or a `mod` declaration. Both test bodies remain inside their original outer modules and use literal includes into the `tests/` directory, which is what preserves their module names while exempting the physical leaves from attribution.

### Lockstep correspondence and checker manifest

| New file | B6 rows that must name it | Frame-loop/checker bindings that must move with it |
| --- | --- | --- |
| `state_machine_instance/construction.rs` | `B6-0077`, `B6-0194`, `B6-0195`, `B6-0196`, `B6-0200`, `B6-0310` | Repoint construction/enrollment lifecycle citations and matching synthetic fixtures. Keep `state_machine.collections`' `pub struct StateMachineInstance` anchor on the root. |
| `state_machine_instance/scripted_objects.rs` | `B6-0077`, `B6-0175`, `B6-0194`, `B6-0195`, `B6-0196`, `B6-0200` | Repoint scripting/hydration lifecycle citations and scripted-action fixture paths. |
| `state_machine_instance/input_focus.rs` | `B6-0077`, `B6-0083` | Repoint focus/semantic/keyboard/gamepad citations and tests. |
| `state_machine_instance/pointer.rs` | `B6-0077`, `B6-0083` | Repoint hit/pointer/listener lifecycle citations and FL-C4/layer/hit fixtures. |
| `state_machine_instance/events_actions.rs` | `B6-0058`, `B6-0077`, `B6-0440` | Repoint event reporting, firing/bubbling, and action-dispatch citations/fixtures. `state_machine.actions`' primary anchor remains on the root if the listener-action executor stays there. |
| `state_machine_instance/bind_targets.rs` | `B6-0058`, `B6-0194`, `B6-0196`, `B6-0200`, `B6-0440` | Repoint bind-target and view-model listener citations/fixtures. |
| `state_machine_instance/bind_sources.rs` | `B6-0058`, `B6-0175`, `B6-0194`, `B6-0195`, `B6-0196`, `B6-0200`, `B6-0440` | Repoint source/relink/data-converter citations and bind fixtures. |
| `state_machine_instance/data_context.rs` | `B6-0058`, `B6-0077`, `B6-0194`, `B6-0195`, `B6-0196`, `B6-0200`, `B6-0440` | Repoint data-context lifecycle citations and listener-cell/bind fixtures. |
| `state_machine_instance/advance.rs` | `B6-0077`, `B6-0194`, `B6-0195`, `B6-0196`, `B6-0200`, `B6-0310`, `B6-0440` | Move `state_machine.advance`'s `rust_file` and `advance_with_report_mode` anchor here. Repoint state-machine event/action/reporting lifecycle citations and advance/keyframe fixtures. Include `runtime_owned_font` in this fragment to avoid an artificial one-function file. |
| `state_machine_instance/tests/view_model_listener.rs` | None; it resides below a `tests/` path. | Move the body unchanged and preserve the outer module name. |
| `state_machine_instance/tests/scripted_listener_actions.rs` | None; it resides below a `tests/` path. | Move the body unchanged. Replace or correct the relative fixture path near old line 19,011 (`../../../../fixtures/sync/vm_listener_fire_event.riv`) because the source becomes two directories deeper; prefer a `CARGO_MANIFEST_DIR`-anchored path. |

#### Global state-machine bindings

- All production fragments must be named by `file-correspondence-manifest.toml`, never `rust-additions.toml`. Every affected SMI row is already multi-module, so this split does not increase the manifest's 155-row scatter count.
- Update C17 retained-module notes for `B6-0058`, `B6-0077`, `B6-0083`, `B6-0175`, `B6-0194`, `B6-0195`, `B6-0196`, `B6-0200`, `B6-0310`, and `B6-0440`; leave the substantive parity assertions untouched.
- `docs/runtime-frame-loop-ownership.toml` cites this file from `focus.retained_tree`, `state_machine.collections`, `state_machine.advance`, `state_machine.events`, `state_machine.actions`, and `event.reporting`. Keep root anchors for the struct, stored event collection, and action executor. Repoint method/lifecycle citations to physical fragments.
- `tools/runtime-frame-loop-port/check.py` currently treats one hard-coded SMI file as the nested-event owner and excludes it from outside-owner scans. Once methods are in included fragments, a path-only exclusion would falsely flag the fragments. Make owner discovery include-aware: recursively expand literal local `include!("...")` files in source order for export analysis, and treat every expanded file as the same logical owner. Do not broaden the exclusion to a directory glob.
- Add checker unit coverage for the include-aware logical owner: an exported nested-event method in an included fragment must be accepted, while the same method in an unrelated file must still fail. Preserve syntax parsing and duplicate-anchor detection.
- `tools/runtime-frame-loop-port/test_check.py` has at least 14 exact SMI references, plus constructed/split path strings, covering FL-C4, lifecycle, layer, hit, bind, events, firing/bubbling, advance, owner exports, keyframes, focus, and semantic behavior. Point each fixture to the actual fragment; do not simply substitute the root path everywhere.
- `docs/runtime-frame-loop-gaps.toml` has no current SMI owner-boundary allow row to migrate.
- Keep `crates/nuxie-runtime/src/state_machine.rs`'s `pub(crate) mod state_machine_instance;` and its re-exports unchanged. Literal includes keep all symbols at the original module path.

### Mechanical PR sequence and proof

This should be a second, independent PR after the artboard split has landed. It uses the same proof battery. The checker must first learn the logical-owner/include relationship in the same PR, because moving the methods without that update creates false ownership failures. Apart from that narrowly required path-awareness, no checker policy should change.

Success means the same 94 tests are collected, no exported API path changes, the correspondence scatter count stays at 155, no new owner-boundary allowance appears, and the full battery shown in the artboard plan remains green.

---

## 3. Nuxie-only seam inventory

### Classification method

The authoritative starting point is correspondence metadata:

- Production Rust paths mapped to upstream C++ in `file-correspondence-manifest.toml` are port-side unless a smaller product-only region can be proven.
- Production Rust paths listed in `rust-additions.toml` are intentional Rust-only additions. The relevant ownership labels are `flowsession-abi`, `scene-api`, and generated/codegen infrastructure.
- `build.rs` is outside the attribution scanner's `src/` roots, so it must be inspected manually.
- A convenience façade is not automatically product-side. Thin Rust-only namespace/generated-storage coordinators that serve the port stay with the port even though no one C++ file corresponds to them.

This prevents two opposite mistakes: burying product authoring features inside the port forever, and extracting Rust implementation infrastructure merely because its file has no one-to-one C++ source.

### Product features and mixed glue

| Region | Current size and location | Runtime internals reached / extraction surface | Placement assessment |
| --- | --- | --- | --- |
| FlowSession wire/domain model | `crates/nuxie/src/flow_session.rs:21-621` (601 lines) | Public `File`, `Factory`, renderer, artboard, view-model, and state-machine types | Clean product vocabulary; move to `nuxie-flow` after dependency direction is fixed. |
| FlowSession orchestration | `flow_session.rs:622-2163` (1,542) | Calls crate-private script preparation, listener preparation, rehydration, apply/rollback, host-cycle, and command-drain methods on the `nuxie` façade | Product-side, but requires a narrow public/sealed flow-runtime bridge before crate extraction. Moderate coupling. |
| FlowSession value snapshots | `flow_session.rs:2164-3105` (942) | Traverses public runtime-owned view-model/value objects and builds its own graph/arena | Product-side and cohesive. Mostly mechanical after bridge creation. |
| FlowSession validation/accounting | `flow_session.rs:3106-3545` (440) | Product protocol limits and payload validation | Clean extraction candidate. |
| FlowSession mutation/apply/diff | `flow_session.rs:3546-4515` (970) | Applies mutations through runtime view-model/state-machine handles; shares rollback/command sequencing with façade-private methods | Product-side; transactional sequencing makes it medium risk. |
| FlowSession selection/catalog | `flow_session.rs:4516-5020` (505) | File/artboard/state-machine discovery | Product-side; mostly public runtime surface. |
| FlowSession tests | `flow_session.rs:5021-7227` (2,207; 40 tests), plus `crates/nuxie/tests/flow_session_contract.rs` (335) | Exercises the public ABI, VM lifecycle, and transactional behavior | Move with FlowSession. Preserve public names through compatibility re-exports during migration. |
| Scene contract and IDs | `crates/nuxie/src/scene.rs:28-1187` (1,160) | Public runtime/binary identities and errors | Product-side, cleanly demarcatable. |
| Scene durable definitions/index/specs | `scene.rs:1188-3401` (2,214) | `nuxie_binary::AuthoringRecord`, property/value types, runtime file identities | Product-side authoring model. |
| Scene hierarchy/materialization | `scene.rs:3402-5622` (2,221) | Constructs/reads `RuntimeFile`; directly touches `materialized.file.runtime` in a handful of places even though `File::runtime()` already exists | Product-side but high coupling. Replace direct private field access with the existing public accessor before moving. |
| Silver/scene export schema | `scene.rs:5623-6819` (1,197) | Public exported-record/document DTOs; lowering later feeds `RuntimeFile::from_authoring_records` | Product-side. The schema and its generated counterpart must move together. |
| Scene store/live mount API | `scene.rs:6820-9099` (2,280) | Owns mounted `File`/artboard/view-model state and coordinates transactions | Product-side; medium-to-high coupling. |
| SceneTx and subordinate transactions | `scene.rs:9100-15589` (6,490) | `SceneTx`, `DataConverterTx`, `VmTx`, `AnimTx`, and `MachineTx` mutate the authoring graph, then materialize into runtime/binary types | Core product authoring seam. Cohesive, but atomic commit/materialization behavior makes the move high risk. |
| Scene frame queries/live mutation | `scene.rs:15590-17592` (2,003) | Hit/geometry/semantic queries and intrinsic image sizing reach crate-private `OwnedArtboardInstance` helpers | Product-side. Needs stable snapshot/command DTOs instead of exposing mutable runtime internals. |
| Scene lowering/validation/export | `scene.rs:17593-25563` (7,971) | Converts authoring records into `RuntimeFile`, validates cross-object invariants, and produces exported documents | Product-side. Highest-risk region because it couples the authoring model to port construction invariants. |
| Scene tests | `scene.rs:25564-38224` (12,661; 120 tests), plus `crates/nuxie/tests/scene_authoring.rs` (15,674) and related authoring tests | Broad transaction, materialization, export, and live-runtime contract | Move with Scene; use this suite as the seam migration oracle. |
| Scene schema generator | `crates/nuxie/build.rs` (4,531) and `scene.rs`'s `include!(concat!(env!("OUT_DIR"), "/scene_schema.rs"))` near line 860 | Generates the Scene/silver record schema at build time | Product-side and inseparable from Scene. Extraction must move the build script/generator inputs and preserve the generated include contract. |
| Script import authorization | `crates/nuxie/src/script_import.rs:1-157` production, `158-211` tests | Ed25519/container verification; calls crate-private `File::execution_authorization_for`/authorization state | Product policy. Move to `nuxie-flow`, but expose a capability-based import seam rather than making raw authorization state public. |
| Mixed façade bridge | `crates/nuxie/src/lib.rs`, principally project-envelope classification and private bridge methods around lines 55, 154, 223-258, 307, 344, 376, 1,492, 2,983, 3,328, 4,304, and 5,600-5,733 | `File::from_runtime`; flow script/listener lifecycle; apply/rollback/command drain; artboard geometry/semantic/intrinsic-image helpers | Not an independent feature. Convert these call sites into explicit narrow bridge traits/DTOs, then move their product consumers. |

The concrete crate-private methods the extraction must account for are:

- Flow lifecycle around `nuxie/src/lib.rs:5600-5675`: `prepare_flow_scripts`, `prepare_flow_listener_actions`, rehydrate/apply, begin host cycle, rollback, and drain commands.
- Scene live/runtime queries around `lib.rs:5705-5733`: hit-test path segments with bounds, geometry path segments, retained/semantic text, and intrinsic image-dimension registration.
- `File::from_runtime` around `lib.rs:4304` and the script execution authorization capability.

These are already cross-crate calls from `nuxie` into public `nuxie-runtime` types; the missing visibility is chiefly inside the `nuxie` façade. Extraction should expose a sealed product bridge from the future base façade, not expand dozens of runtime fields to `pub`.

### Project-data converter: product model inside the port

`crates/nuxie-runtime/src/project_data_converter.rs` is 2,686 lines and is explicitly classified `scene-api` in `rust-additions.toml`:

| Lines | Region | Coupling and disposition |
| ---: | --- | --- |
| 22-507 | Public values, specs, errors, and state | Pure product/protocol model; eventual neutral-crate candidate. |
| 508-1,289 | Envelope/program/catalog compilation and evaluation | Mostly `std`/`serde`; eventual neutral-crate candidate. |
| 1,290-2,324 | Validation, coercion, formatting, interpolation | Pure-looking core, but behavior is part of runtime data-bind semantics. Extract only with parity tests. |
| 2,325-2,686 | Formula parser | Mechanically separable after the outer seam is established. |

The file is locally self-contained, but its runtime integration is not. `data_bind/context/context_value.rs` has a `RuntimeDataBindGraphConverter::Project` variant and owns conversion/evaluator state; `data_bind/data_bind_context.rs` constructs it from a script-asset envelope; both runtime and `nuxie` re-export or classify its types; Scene authors and lowers them. A direct move to `nuxie-flow` would invert the desired dependency and make the port depend on the product crate.

Two crate-private numeric-policy helpers in this file (the bounded-list constant near line 467 and helper near line 2,320) are also used by port-side Number-to-List/context conversion. If the model is eventually extracted, those policies should remain in the port or move to a dependency-neutral shared crate; they should not become a product API solely to satisfy the move.

Recommendation: first demarcate this as `project_data_converter/{model, program, evaluator, bridge}` inside `nuxie-runtime`; later extract only the pure model/program/evaluator to a neutral `nuxie-project-data` crate that both runtime and `nuxie-flow` may depend on. The runtime adapter stays in `nuxie-runtime`.

### Rust-only infrastructure that should remain port-side

The following `rust-additions.toml` entries have no one-file C++ correspondence but are not product features:

| Current path/group | Lines | Why it stays in `nuxie-runtime` |
| --- | ---: | --- |
| `data_bind_container.rs`, `data_converter.rs`, `retained_data_bind.rs` | 12 total | Tiny generated/type façade modules over port functionality. |
| `input/mod.rs`, `math/mod.rs`, `shapes/mod.rs`, `shapes/paint/mod.rs` | 174 total | Rust namespace/generated-storage coordinators. |
| `objects.rs` | 1,018 | Runtime object registry plus `include!(concat!(env!("OUT_DIR"), "/runtime_objects.rs"))`; generated by `crates/nuxie-runtime/build.rs` (707 lines). This is port construction infrastructure. |
| `properties.rs` | 826 | Generated property interpretation/storage used throughout the port. |
| `state_machine.rs`, `state_machine/instance.rs`, `state_machine/listener_types/mod.rs` | 215 total | Rust module/re-export façade for the ported state-machine implementation. |
| `viewmodel/mod.rs`, `viewmodel/runtime/mod.rs` | 58 total | Rust namespace/re-export façade for ported view-model behavior. |
| `focus.rs` | 7 | Thin public façade over retained focus behavior, not a standalone product feature. |

These codegen/facade additions total about 2,303 source lines excluding the project converter and focus façade. They should be clearly marked as Rust port infrastructure, but moving them to `nuxie-flow` would blur rather than improve the seam.

`crates/nuxie/src/command_queue.rs`, server code, and raw-text support are not included in the Nuxie-only list: their current files are explicitly mapped to upstream C++ correspondence. A richer Rust API around a ported facility does not by itself make that facility product-side.

---

## 4. Recommended seam design

### Decision

Use a **module boundary now and a crate boundary later**.

The immediate structure should make product ownership visible without changing dependencies:

```text
crates/nuxie/src/product/
  flow_session/
  scene/
    model/
    transaction/
    live/
    export/
  script_import/

crates/nuxie-runtime/src/project_data_converter/
  model.rs
  program.rs
  evaluator.rs
  bridge.rs
```

Preserve current public paths with root re-exports. This first step is a namespace/demarcation change, not an extraction. It gives reviewers an auditable boundary while the current test and parity baseline still observes exactly the same crate graph.

The long-term crate graph should avoid a cycle:

```text
nuxie-runtime  <---  nuxie-core  <---  nuxie-flow
       ^                 ^                 ^
       |                 |                 |
       +------ nuxie-project-data --------+

nuxie (umbrella compatibility crate) re-exports nuxie-core + nuxie-flow
```

`nuxie-flow` contains FlowSession, Scene/SceneTx, silver/export DTOs and lowering, and script-import policy. `nuxie-core` contains `File`, `Factory`, owned artboard/view-model handles, renderer integration, and narrow bridge traits. `nuxie-runtime` remains the C++-correspondence port plus explicitly labeled Rust port infrastructure. A small dependency-neutral `nuxie-project-data` is optional and should be introduced only after the runtime adapter has been separated from the pure evaluator.

### Public surface of the port/base layer

The extraction should not make arbitrary runtime fields public. The required surface is narrow:

1. A sealed/trusted constructor that wraps a validated `RuntimeFile` as a façade `File`, with an explicit authoring/import authorization policy.
2. A flow-runtime bridge for prepare, rehydrate, apply, host-cycle, rollback, and command-drain operations. Its inputs/outputs should be stable value DTOs, not references to internal queues.
3. Read-only artboard snapshot operations for hit geometry, retained geometry, and semantic text, plus a command for intrinsic image dimensions. Return owned snapshots so Scene cannot retain internal graph borrows.
4. A project-converter adapter owned by `nuxie-runtime`; pure program/model types may come from the neutral crate. Do not expose the runtime bind graph or evaluator cells as product API.
5. Existing upstream-corresponding runtime types only where they are already public and semantically stable. Product re-exports should live in `nuxie`/`nuxie-flow`, not expand the port's promise.

### Migration order

1. Establish the exact green parity/golden baseline and keep it pinned in every PR.
2. Land the `artboard.rs` and `state_machine_instance.rs` mechanical splits as separate PRs. This makes later ownership changes reviewable without mixing 20,000-line source moves into semantic work.
3. Demarcate `nuxie::product::{flow_session, scene, script_import}` and the project-converter submodules in place, preserving public re-exports.
4. Define the sealed flow, authoring-file, geometry-snapshot, and script-authorization bridges; migrate existing internal callers to them while everything remains in the same crate.
5. Extract the non-product façade/runtime handles into `nuxie-core`, with `nuxie` temporarily acting as an umbrella compatibility crate.
6. Move FlowSession and script import to `nuxie-flow`; move their tests and retain compatibility re-exports.
7. Move Scene model, SceneTx, export schema/generator, lowering, live adapter, and tests together. Do not split the generated schema from `build.rs` or split transaction commit from materialization in the extraction PR.
8. Isolate the project converter's pure model/program/evaluator. Extract it to a neutral crate only if doing so leaves the runtime adapter dependency pointing inward, never from `nuxie-runtime` to `nuxie-flow`.
9. Remove compatibility re-exports and tighten visibility in a later breaking-change PR, after downstream migration.

### Risk map

| Risk | Areas | Reason |
| --- | --- | --- |
| Mechanical/low | Thin namespace façades, codegen coordinators staying in place, test-module extraction, pure FlowSession validation/types | Mostly physical ownership with no runtime mutation semantics. |
| Moderate | FlowSession after bridge creation, script import after capability API, Scene DTO/export types | Public API and authorization compatibility, but limited graph mutation. |
| High | SceneTx commit/materialization, live Scene mounting, geometry/semantic access, Scene lowering and generated schema | Atomicity, identity, generated code, and live runtime invariants cross the seam. |
| High | Project-data converter extraction | The source looks pure, but evaluator state is embedded in runtime data-bind context and Number-to-List policy. |
| High | Script/listener ownership during FlowSession moves | Rehydration, rollback, command queues, and VM object lifetime must remain one coherent transaction. |

### Why baseline-first matters

`docs/parity-gap-register.md` explicitly recommends making the parity claim trustworthy before feature work and landing each subsequent change against the oracle (the “Recommended execution order” around lines 204-211). It also describes Scene as a Nuxie-only additive API (around lines 126-127) while recording FlowSession/capability-boundary gaps elsewhere in the register. That is the governing principle here: first make source moves mechanically observable by the existing correspondence, ownership, trace, golden, and silver batteries; then introduce the product seam behind those same oracles. A green build alone is insufficient because it can preserve compilation while silently weakening ownership scans or changing what “zero divergences” measures.

The two large-file splits are therefore prerequisites for, not part of, the product extraction. Each split keeps the logical Rust module unchanged and updates only physical-address consumers. The seam work then proceeds through explicit bridges and compatibility re-exports, with risky transaction/materialization changes isolated from mechanical moves.
# nuxie-runtime structure report

## Scope and baseline

This is a read-only structure analysis. It proposes no behavior, source, parity-ledger, or ownership-ledger changes.

The inspected worktree was already detached at `e8726db8ffc689d3b19f1c0a55794aa6daf9956d` (`Merge pull request #246 from TheOneLevin/feat/b6-0058-listener-viewmodel-change`, 2026-08-04). The local `origin/main` ref resolves to the same commit. The requested `git fetch origin` and redundant `git checkout --detach origin/main` could not update the shared worktree metadata because the sandbox cannot write the parent repository's `.git/worktrees/nuxie-mr-c12` directory. The analysis is therefore pinned to the locally available `origin/main` commit above, not a newly fetched remote ref. The pre-existing untracked `vtriage-capture/` directory was left untouched.

The two target files have grown beyond the sizes in the request:

| File | Current lines | Production/support | Tests |
| --- | ---: | ---: | ---: |
| `crates/nuxie-runtime/src/artboard.rs` | 23,374 | 12,039 | 11,335 (`205` tests) |
| `crates/nuxie-runtime/src/state_machine/state_machine_instance.rs` | 21,674 | 14,079 | 7,595 (`94` tests) |

The safest split mechanism is literal `include!` fragments, not Rust child modules. An included fragment remains in the original semantic module (`crate::artboard` or `crate::state_machine::state_machine_instance`), so private-name resolution, inherent method paths, sibling access, and demangled symbols remain unchanged. A `mod build;`/`#[path = ...] mod build;` split would introduce new privacy boundaries and make a supposedly mechanical PR into an API refactor.

---

## 1. Large-file split plan: `artboard.rs`

### Current structural map

| Lines | Region |
| ---: | --- |
| 1-106 | Imports and supporting aliases |
| 107-135 | Draw-frame counter and generated `Mat2D` helper |
| 136-160 | `ExternalFontAssetError` |
| 161-181 | `RuntimeArtboardInstanceIdentity` |
| 182-231 | `RuntimeScriptState` and implementations |
| 232-250 | Advancing-component and persistent-dirt fixtures |
| 251-298 | Script advance/update mode enums and implementations |
| 299-327 | Resetting/runtime-component helper types |
| 328-334 | `RuntimeTextStyleFeatureOption` |
| 335-523 | `ArtboardInstance` storage |
| 524-714 | `Clone for ArtboardInstance` |
| 715-739 | `Drop for ArtboardInstance` followed by the existing leaf `include!` declarations |
| 740-773 | Occurrence and frame-advance report types |
| 774-844 | Build context/index/text-flag helpers |
| 845-860 | `RuntimeNestedAnimationInstance` |
| 861-11,133 | Main `impl ArtboardInstance` |
| 11,134-11,147 | Focus-scroll path and focus-bounds transform helper |
| 11,148-11,187 | Small follow-up `impl ArtboardInstance` |
| 11,188-11,226 | Component-list reset helper |
| 11,227-11,595 | `impl RuntimeNestedArtboardInstance` |
| 11,596-11,611 | `impl RuntimeNestedAnimationInstance` |
| 11,612-12,039 | Free construction/nested-artboard helpers |
| 12,040-23,374 | Single `#[cfg(test)] mod tests` (`205` tests) |

The main implementation itself divides along stable responsibility boundaries:

| Lines | Logical responsibility |
| ---: | --- |
| 861-1,104 | Lifecycle, data context, and transient-clone restoration |
| 1,105-2,440 | Build occurrence relationships and constructors |
| 2,441-2,712 | Path/hit initialization and external assets |
| 2,713-3,399 | Scripting lifecycle |
| 3,400-3,989 | Object/component/audio/hit/graph/definition access |
| 3,990-4,152 | Scripted interpolators and state-machine definition selection |
| 4,153-4,480 | Dimensions, world/layout bounds, scrolling, and focus reveal |
| 4,481-5,229 | Generated property façade and text values |
| 5,230-5,568 | Animation/state-machine/view-model occurrences |
| 5,569-6,312 | Component-list lifecycle and layout |
| 6,313-7,408 | Root/nested frame advance and retained-component dispatch |
| 7,409-7,585 | Nested-layout frame propagation |
| 7,586-8,496 | Transforms, epochs, dirt, and invalidation |
| 8,497-9,191 | Update pass and five-pass state-machine settlement |
| 9,192-9,825 | Component update dispatch and script scheduling |
| 9,826-11,133 | Property-change callbacks, nested controls, and collapse |

### Proposed physical layout

Keep `artboard.rs` as the owner. It retains imports, supporting types, `ArtboardInstance`, `Clone`, `Drop`, the existing leaf includes, occurrence/report/build helper types, and an ordered series of literal includes. This preserves declaration order where it matters and makes all fragments part of `crate::artboard`.

```text
src/
  artboard.rs                         owner/types, existing leaf includes, ordered include! list
  artboard/
    build.rs                          current 861-2712
    script_runtime.rs                 current 2713-3399
    access.rs                         current 3400-3989
    definitions_and_layout.rs         current 3990-4480
    properties.rs                     current 4481-5568
    component_lists.rs                current 5569-6312
    advance.rs                        current 6313-7585
    dirt.rs                           current 7586-8496
    update.rs                         current 8497-9825
    callbacks.rs                      current 9826-11133
    nested.rs                         current 11134-12039
    tests.rs                          body of current tests module
```

The production fragments are intentionally in execution/dependency order rather than alphabetical order. `tests.rs` should be included inside the original test module:

```rust
#[cfg(test)]
mod tests {
    include!("artboard/tests.rs");
}
```

This retains the test module's name and access. It also gives the attribution checker a deliberately exempt `tests.rs` leaf.

For every production file in the layout above, the owner must contain a literal `include!("artboard/<name>.rs");` at the point where that region previously appeared. Do not use `#[path]`, `mod`, or `pub(crate) mod`: each would create a child-module/privacy boundary. Every such production fragment is attribution-scanned and must be named by the B6 rows below; only `tests.rs` receives the test-source exemption.

### Lockstep correspondence and checker manifest

The following table is the required per-file edit manifest. “B6 rows” are `[[files]]` entries in `file-correspondence-manifest.toml`; add the fragment's path to those rows' semicolon-separated `rust_module` values and update their C17 retained-module notes. Keep status, upstream pins, C++ paths, verification text, and correspondence claims unchanged. The row list is conservative: a later provenance cleanup can narrow it, but narrowing must not be mixed into this mechanical split.

| New file | B6 rows that must name it | Frame-loop/checker bindings that must move with it |
| --- | --- | --- |
| `artboard/build.rs` | `B6-0001`, `B6-0094`, `B6-0115`, `B6-0117`, `B6-0123`, `B6-0125`, `B6-0129`, `B6-0131`, `B6-0142`, `B6-0146`, `B6-0202`, `B6-0258`, `B6-0303`, `B6-0304`, `B6-0305`, `B6-0331`, `B6-0361`, `B6-0362`, `B6-0365`, `B6-0368`, `B6-0385`, `B6-0388`, `B6-0399`, `B6-0405`, `B6-0408` | Repoint `summarize_trace.py` dependency-build and IK-chain-build source scans when their anchors land here. Repoint matching `runtime-frame-loop-gaps.toml` owner-boundary entries. |
| `artboard/script_runtime.rs` | `B6-0077`, `B6-0194`, `B6-0200`, `B6-0322`, `B6-0326` | Repoint the script-lifecycle citations in `runtime-frame-loop-ownership.toml` and any corresponding synthetic test fixture paths. |
| `artboard/access.rs` | `B6-0094`, `B6-0095`, `B6-0123`, `B6-0146`, `B6-0203`, `B6-0204`, `B6-0205` | Repoint any live-draw/renderer-interface lifecycle citations whose named methods move here. |
| `artboard/definitions_and_layout.rs` | `B6-0077`, `B6-0094`, `B6-0258`, `B6-0303`, `B6-0304`, `B6-0305`, `B6-0385`, `B6-0388` | Repoint focus-retained-tree and layout lifecycle citations, plus matching owner-boundary allow entries. |
| `artboard/properties.rs` | `B6-0067`, `B6-0077`, `B6-0094`, `B6-0194`, `B6-0200`, `B6-0352`, `B6-0355`, `B6-0385`, `B6-0388`, `B6-0399`, `B6-0404`, `B6-0405`, `B6-0408` | `B6-0067` currently has only one Rust module: **replace** `artboard.rs` with this fragment rather than adding a second path. Repoint property/text lifecycle citations and corresponding owner-boundary entries. |
| `artboard/component_lists.rs` | `B6-0095`, `B6-0258`, `B6-0303`, `B6-0304`, `B6-0305` | Repoint component-list and nested-layout lifecycle citations and matching test fixtures. |
| `artboard/advance.rs` | `B6-0001`, `B6-0059`, `B6-0061`, `B6-0063`, `B6-0064`, `B6-0077`, `B6-0094`, `B6-0095`, `B6-0194`, `B6-0200`, `B6-0258`, `B6-0303`, `B6-0304`, `B6-0305`, `B6-0322`, `B6-0326`, `B6-0388` | In `runtime-frame-loop-ownership.toml`, move `artboard.advance`'s `rust_file` and the lifecycle citations for artboard/nested advance. Repoint `summarize_trace.py` advancing/resetting dispatch scans as applicable. Repoint relevant gap allow rows and `test_check.py` advance fixtures. |
| `artboard/dirt.rs` | `B6-0094`, `B6-0123`, `B6-0258`, `B6-0331`, `B6-0352`, `B6-0355`, `B6-0357`, `B6-0361`, `B6-0362`, `B6-0365`, `B6-0368`, `B6-0385`, `B6-0388`, `B6-0399`, `B6-0405`, `B6-0408` | Move `component.dirt`, `component.dependents` (`pub fn add_dirt(`), and transform/dirt lifecycle citations as appropriate. Repoint `summarize_trace.py`'s two add-dirt anchors, dirt consumptions, owner resolutions, and skin-buffer scans according to their final physical locations. Repoint matching gap rows. |
| `artboard/update.rs` | `B6-0001`, `B6-0094`, `B6-0115`, `B6-0117`, `B6-0123`, `B6-0125`, `B6-0129`, `B6-0131`, `B6-0142`, `B6-0194`, `B6-0200`, `B6-0203`, `B6-0204`, `B6-0205`, `B6-0258`, `B6-0303`, `B6-0304`, `B6-0305`, `B6-0322`, `B6-0326`, `B6-0331`, `B6-0352`, `B6-0355`, `B6-0357`, `B6-0361`, `B6-0362`, `B6-0365`, `B6-0368`, `B6-0385`, `B6-0388`, `B6-0399`, `B6-0404`, `B6-0405`, `B6-0408` | Move `component.update_order` (`fn update_components_with_hook_recording`) and `artboard.update_pass` (`pub fn update_pass(`) `rust_file` values and lifecycle citations. Repoint retained update/settlement gap rows, trace scans, and test fixtures. |
| `artboard/callbacks.rs` | `B6-0059`, `B6-0061`, `B6-0063`, `B6-0064`, `B6-0077`, `B6-0094`, `B6-0123`, `B6-0125`, `B6-0129`, `B6-0131`, `B6-0142`, `B6-0194`, `B6-0200`, `B6-0258`, `B6-0303`, `B6-0304`, `B6-0305`, `B6-0331`, `B6-0352`, `B6-0355`, `B6-0357`, `B6-0361`, `B6-0362`, `B6-0365`, `B6-0368`, `B6-0385`, `B6-0388`, `B6-0399`, `B6-0404`, `B6-0405`, `B6-0408` | Change `check_layout_style_handlers.py`'s `RUST_ARTBOARD_MODULE` to this fragment if both display-change markers remain here; otherwise make the checker accept the two explicit physical files. Repoint callback lifecycle citations, gap rows, and fixtures. |
| `artboard/nested.rs` | `B6-0059`, `B6-0061`, `B6-0063`, `B6-0064`, `B6-0077`, `B6-0094`, `B6-0095`, `B6-0258`, `B6-0303`, `B6-0304`, `B6-0305` | Repoint nested-artboard advance/layout citations, gap allow entries, and nested fixtures. |
| `artboard/tests.rs` | None. `rust_attribution.py` excludes a source whose stem is `tests`. | Move the artboard test body unchanged. Update the seven relative `include_bytes!`/fixture paths currently near old lines 19,951, 20,001, 20,168, 20,542, 20,597, 20,612, and 23,311 because the file becomes one directory deeper. Prefer `concat!(env!("CARGO_MANIFEST_DIR"), "/...")` for fixture stability. The `env!` use near old line 14,290 remains valid. |

#### Global artboard bindings

These bindings span more than one fragment and therefore belong in the same PR:

- `rust_attribution.py` discovers all production `.rs` files below `crates/nuxie-runtime/src`. Every new production fragment must be listed in at least one `file-correspondence-manifest.toml` row and must **not** be added to `rust-additions.toml`. The checker exempts `tests.rs`, names ending in `_tests`, and files below a `tests/` path.
- The manifest's scatter ratchet is already at its maximum: 155 rows currently have multiple Rust modules. Except for `B6-0067`, all affected artboard rows are already multi-module. Replacing the `B6-0067` root path prevents the split from increasing that count.
- Update every affected C17 note that enumerates retained Rust modules. Do not change the underlying parity classification merely because the code's physical address changed.
- `docs/runtime-frame-loop-ownership.toml` has artboard citations in `focus.retained_tree`, `component.identity`, `component.dirt`, `component.dependents`, `component.update_order`, `component.transforms`, `component.clone_drop`, `artboard.advance`, `artboard.update_pass`, `nested_artboard.advance`, `artboard.live_draw`, and `renderer.interface_boundary`. Keep `component.clone_drop` and its `pub struct ArtboardInstance` anchor on the root; change each other `rust_file` or `rust:path:line-range` to its physical fragment.
- `docs/runtime-frame-loop-gaps.toml` has 25 owner-boundary allow entries pointing at `artboard.rs` (entries 0-7, 11-22, and 29-34 in current order). Repoint each to the fragment that contains its anchor. A literal move should preserve its token-based `site_hash`; a changed hash is a signal that the split was not mechanical.
- `tools/runtime-frame-loop-port/summarize_trace.py` hard-codes `artboard.rs` nine times for add-dirt, dirt consumption, owner resolution, dependency build, skin rebuild, advancing dispatch, resetting dispatch, and IK-chain discovery. Give each query its actual fragment path. The demangled owner should remain `artboard::ArtboardInstance` because `include!` does not introduce a child module.
- `tools/runtime-frame-loop-port/check_layout_style_handlers.py` hard-codes the artboard source while looking for `propagate_layout_component_display_changed` and the display property key. Point it to the fragment(s) containing those markers.
- `tools/runtime-frame-loop-port/test_check.py` contains at least 14 exact artboard paths covering probe gating, layer occurrence, live-advance ratchets, registry site hashes/substitution, and trait-anchor behavior. Update each synthetic fixture to the fragment that owns the tested anchor; do not perform a blind string replacement.
- Leave the existing root-level includes (`nested_artboard.rs`, `artboard_component_list.rs`, `artboard_list_map_rule.rs`, `artboard_referencer.rs`, `bindable_artboard.rs`, `nested_artboard_layout.rs`, `nested_artboard_leaf.rs`, `component_origin.rs`, bone/weight, profiler/profile) where they are. Moving them under `artboard/` changes relative include resolution and correspondence ownership.
- Keep `crates/nuxie-runtime/src/lib.rs`'s `mod artboard;` unchanged. The public and crate-private paths must not acquire an extra `artboard::<fragment>` segment.

### Mechanical PR sequence and proof

One PR should contain only this file's split and the path/anchor bookkeeping above. Recommended commit sequence within that PR:

1. Extract `tests.rs`, fix only its fixture paths, and prove the same 205 tests are collected.
2. Extract production regions in original order using `include!`; make no renames, visibility edits, formatting sweeps, or logic edits.
3. Update correspondence C17/path fields and all frame-loop physical anchors.
4. Update checker source locations and synthetic fixtures, including an assertion that demangled ownership remains `artboard::ArtboardInstance`.

Proof battery:

```text
make rust-sources-fresh
make cpp-probe
make runtime-frame-loop-port-test
make runtime-frame-loop-port-check
make rust-attribution-check
cargo check --workspace
cargo test -p nuxie-runtime
cargo test -p nuxie --features scripting
make scripted-golden-compare
make silver-corpus-test
```

The PR is structure-preserving only if parity row counts/statuses, trace landmarks, owner-boundary hashes, test counts, and golden/silver outputs are unchanged.

---

## 2. Large-file split plan: `state_machine_instance.rs`

### Current structural map

| Lines | Region |
| ---: | --- |
| 1-93 | Imports and ownership commentary |
| 94-255 | Nested event-chain/notify phases, trace structures, and trace recording |
| 256-312 | Semantic-node resolver and audio/event seam types |
| 313-1,271 | `RuntimeHitResult`, `HitComponent`, and hierarchy implementations |
| 1,272-1,467 | Nested notifier, focus, semantic, and state helper types |
| 1,468-1,474 | `RuntimeDataContextBindError` |
| 1,475-1,711 | `StateMachineInstance` storage |
| 1,712-1,744 | Data-bind occurrence/pointer/executor structures |
| 1,745-2,147 | Listener-action executor implementations |
| 2,148-2,335 | `Clone for StateMachineInstance` |
| 2,336-2,356 | `Drop for StateMachineInstance` |
| 2,357-2,638 | Free helpers and view-model listener types |
| 2,639-13,938 | Main `impl StateMachineInstance` |
| 13,939-13,953 | `runtime_owned_font` helper |
| 13,954-14,079 | `view_model_listener_tests` (`1` test) |
| 14,080-21,674 | `scripted_listener_action_tests` (`93` tests) |

The main implementation divides as follows:

| Lines | Logical responsibility |
| ---: | --- |
| 2,639-2,932 | Teardown, enrollment, focus manager, clone rebind |
| 2,933-3,741 | Construction and listener/layer initialization |
| 3,742-5,102 | Scripting, hydration, and adoption |
| 5,103-6,007 | State/input/focus/semantic/keyboard/gamepad operations |
| 6,008-7,487 | Hit/listener/pointer pipeline |
| 7,488-8,525 | Event notification, deferred callbacks, listener actions |
| 8,526-9,100 | Direct bind targets and value readers |
| 9,101-10,748 | Default/imported/owned source setters and relinking |
| 10,749-12,599 | Data-context bind/rebind, listener cells, bind updates |
| 12,600-13,938 | Reports, advance/apply, nested event dispatch, settlement |

### Proposed physical layout

```text
src/state_machine/
  state_machine_instance.rs               owner/types, hit hierarchy, lifecycle, ordered include! list
  state_machine_instance/
    construction.rs                       current 2639-3741
    scripted_objects.rs                   current 3742-5102
    input_focus.rs                        current 5103-6007
    pointer.rs                            current 6008-7487
    events_actions.rs                     current 7488-8525
    bind_targets.rs                       current 8526-9100
    bind_sources.rs                       current 9101-10748
    data_context.rs                       current 10749-12599
    advance.rs                            current 12600-13953
    tests/
      view_model_listener.rs              body of current first test module
      scripted_listener_actions.rs         body of current second test module
```

Retain the two existing module names in the root and include only their bodies. This preserves test paths and any name-sensitive filtering:

```rust
#[cfg(test)]
mod view_model_listener_tests {
    include!("state_machine_instance/tests/view_model_listener.rs");
}
```

Use the same pattern for `scripted_listener_action_tests`.

For every production file in this layout, place `include!("state_machine_instance/<name>.rs");` in the root at the original region boundary. Do not use `#[path]` or a `mod` declaration. Both test bodies remain inside their original outer modules and use literal includes into the `tests/` directory, which is what preserves their module names while exempting the physical leaves from attribution.

### Lockstep correspondence and checker manifest

| New file | B6 rows that must name it | Frame-loop/checker bindings that must move with it |
| --- | --- | --- |
| `state_machine_instance/construction.rs` | `B6-0077`, `B6-0194`, `B6-0195`, `B6-0196`, `B6-0200`, `B6-0310` | Repoint construction/enrollment lifecycle citations and matching synthetic fixtures. Keep `state_machine.collections`' `pub struct StateMachineInstance` anchor on the root. |
| `state_machine_instance/scripted_objects.rs` | `B6-0077`, `B6-0175`, `B6-0194`, `B6-0195`, `B6-0196`, `B6-0200` | Repoint scripting/hydration lifecycle citations and scripted-action fixture paths. |
| `state_machine_instance/input_focus.rs` | `B6-0077`, `B6-0083` | Repoint focus/semantic/keyboard/gamepad citations and tests. |
| `state_machine_instance/pointer.rs` | `B6-0077`, `B6-0083` | Repoint hit/pointer/listener lifecycle citations and FL-C4/layer/hit fixtures. |
| `state_machine_instance/events_actions.rs` | `B6-0058`, `B6-0077`, `B6-0440` | Repoint event reporting, firing/bubbling, and action-dispatch citations/fixtures. `state_machine.actions`' primary anchor remains on the root if the listener-action executor stays there. |
| `state_machine_instance/bind_targets.rs` | `B6-0058`, `B6-0194`, `B6-0196`, `B6-0200`, `B6-0440` | Repoint bind-target and view-model listener citations/fixtures. |
| `state_machine_instance/bind_sources.rs` | `B6-0058`, `B6-0175`, `B6-0194`, `B6-0195`, `B6-0196`, `B6-0200`, `B6-0440` | Repoint source/relink/data-converter citations and bind fixtures. |
| `state_machine_instance/data_context.rs` | `B6-0058`, `B6-0077`, `B6-0194`, `B6-0195`, `B6-0196`, `B6-0200`, `B6-0440` | Repoint data-context lifecycle citations and listener-cell/bind fixtures. |
| `state_machine_instance/advance.rs` | `B6-0077`, `B6-0194`, `B6-0195`, `B6-0196`, `B6-0200`, `B6-0310`, `B6-0440` | Move `state_machine.advance`'s `rust_file` and `advance_with_report_mode` anchor here. Repoint state-machine event/action/reporting lifecycle citations and advance/keyframe fixtures. Include `runtime_owned_font` in this fragment to avoid an artificial one-function file. |
| `state_machine_instance/tests/view_model_listener.rs` | None; it resides below a `tests/` path. | Move the body unchanged and preserve the outer module name. |
| `state_machine_instance/tests/scripted_listener_actions.rs` | None; it resides below a `tests/` path. | Move the body unchanged. Replace or correct the relative fixture path near old line 19,011 (`../../../../fixtures/sync/vm_listener_fire_event.riv`) because the source becomes two directories deeper; prefer a `CARGO_MANIFEST_DIR`-anchored path. |

#### Global state-machine bindings

- All production fragments must be named by `file-correspondence-manifest.toml`, never `rust-additions.toml`. Every affected SMI row is already multi-module, so this split does not increase the manifest's 155-row scatter count.
- Update C17 retained-module notes for `B6-0058`, `B6-0077`, `B6-0083`, `B6-0175`, `B6-0194`, `B6-0195`, `B6-0196`, `B6-0200`, `B6-0310`, and `B6-0440`; leave the substantive parity assertions untouched.
- `docs/runtime-frame-loop-ownership.toml` cites this file from `focus.retained_tree`, `state_machine.collections`, `state_machine.advance`, `state_machine.events`, `state_machine.actions`, and `event.reporting`. Keep root anchors for the struct, stored event collection, and action executor. Repoint method/lifecycle citations to physical fragments.
- `tools/runtime-frame-loop-port/check.py` currently treats one hard-coded SMI file as the nested-event owner and excludes it from outside-owner scans. Once methods are in included fragments, a path-only exclusion would falsely flag the fragments. Make owner discovery include-aware: recursively expand literal local `include!("...")` files in source order for export analysis, and treat every expanded file as the same logical owner. Do not broaden the exclusion to a directory glob.
- Add checker unit coverage for the include-aware logical owner: an exported nested-event method in an included fragment must be accepted, while the same method in an unrelated file must still fail. Preserve syntax parsing and duplicate-anchor detection.
- `tools/runtime-frame-loop-port/test_check.py` has at least 14 exact SMI references, plus constructed/split path strings, covering FL-C4, lifecycle, layer, hit, bind, events, firing/bubbling, advance, owner exports, keyframes, focus, and semantic behavior. Point each fixture to the actual fragment; do not simply substitute the root path everywhere.
- `docs/runtime-frame-loop-gaps.toml` has no current SMI owner-boundary allow row to migrate.
- Keep `crates/nuxie-runtime/src/state_machine.rs`'s `pub(crate) mod state_machine_instance;` and its re-exports unchanged. Literal includes keep all symbols at the original module path.

### Mechanical PR sequence and proof

This should be a second, independent PR after the artboard split has landed. It uses the same proof battery. The checker must first learn the logical-owner/include relationship in the same PR, because moving the methods without that update creates false ownership failures. Apart from that narrowly required path-awareness, no checker policy should change.

Success means the same 94 tests are collected, no exported API path changes, the correspondence scatter count stays at 155, no new owner-boundary allowance appears, and the full battery shown in the artboard plan remains green.

---

## 3. Nuxie-only seam inventory

### Classification method

The authoritative starting point is correspondence metadata:

- Production Rust paths mapped to upstream C++ in `file-correspondence-manifest.toml` are port-side unless a smaller product-only region can be proven.
- Production Rust paths listed in `rust-additions.toml` are intentional Rust-only additions. The relevant ownership labels are `flowsession-abi`, `scene-api`, and generated/codegen infrastructure.
- `build.rs` is outside the attribution scanner's `src/` roots, so it must be inspected manually.
- A convenience façade is not automatically product-side. Thin Rust-only namespace/generated-storage coordinators that serve the port stay with the port even though no one C++ file corresponds to them.

This prevents two opposite mistakes: burying product authoring features inside the port forever, and extracting Rust implementation infrastructure merely because its file has no one-to-one C++ source.

### Product features and mixed glue

| Region | Current size and location | Runtime internals reached / extraction surface | Placement assessment |
| --- | --- | --- | --- |
| FlowSession wire/domain model | `crates/nuxie/src/flow_session.rs:21-621` (601 lines) | Public `File`, `Factory`, renderer, artboard, view-model, and state-machine types | Clean product vocabulary; move to `nuxie-flow` after dependency direction is fixed. |
| FlowSession orchestration | `flow_session.rs:622-2163` (1,542) | Calls crate-private script preparation, listener preparation, rehydration, apply/rollback, host-cycle, and command-drain methods on the `nuxie` façade | Product-side, but requires a narrow public/sealed flow-runtime bridge before crate extraction. Moderate coupling. |
| FlowSession value snapshots | `flow_session.rs:2164-3105` (942) | Traverses public runtime-owned view-model/value objects and builds its own graph/arena | Product-side and cohesive. Mostly mechanical after bridge creation. |
| FlowSession validation/accounting | `flow_session.rs:3106-3545` (440) | Product protocol limits and payload validation | Clean extraction candidate. |
| FlowSession mutation/apply/diff | `flow_session.rs:3546-4515` (970) | Applies mutations through runtime view-model/state-machine handles; shares rollback/command sequencing with façade-private methods | Product-side; transactional sequencing makes it medium risk. |
| FlowSession selection/catalog | `flow_session.rs:4516-5020` (505) | File/artboard/state-machine discovery | Product-side; mostly public runtime surface. |
| FlowSession tests | `flow_session.rs:5021-7227` (2,207; 40 tests), plus `crates/nuxie/tests/flow_session_contract.rs` (335) | Exercises the public ABI, VM lifecycle, and transactional behavior | Move with FlowSession. Preserve public names through compatibility re-exports during migration. |
| Scene contract and IDs | `crates/nuxie/src/scene.rs:28-1187` (1,160) | Public runtime/binary identities and errors | Product-side, cleanly demarcatable. |
| Scene durable definitions/index/specs | `scene.rs:1188-3401` (2,214) | `nuxie_binary::AuthoringRecord`, property/value types, runtime file identities | Product-side authoring model. |
| Scene hierarchy/materialization | `scene.rs:3402-5622` (2,221) | Constructs/reads `RuntimeFile`; directly touches `materialized.file.runtime` in a handful of places even though `File::runtime()` already exists | Product-side but high coupling. Replace direct private field access with the existing public accessor before moving. |
| Silver/scene export schema | `scene.rs:5623-6819` (1,197) | Public exported-record/document DTOs; lowering later feeds `RuntimeFile::from_authoring_records` | Product-side. The schema and its generated counterpart must move together. |
| Scene store/live mount API | `scene.rs:6820-9099` (2,280) | Owns mounted `File`/artboard/view-model state and coordinates transactions | Product-side; medium-to-high coupling. |
| SceneTx and subordinate transactions | `scene.rs:9100-15589` (6,490) | `SceneTx`, `DataConverterTx`, `VmTx`, `AnimTx`, and `MachineTx` mutate the authoring graph, then materialize into runtime/binary types | Core product authoring seam. Cohesive, but atomic commit/materialization behavior makes the move high risk. |
| Scene frame queries/live mutation | `scene.rs:15590-17592` (2,003) | Hit/geometry/semantic queries and intrinsic image sizing reach crate-private `OwnedArtboardInstance` helpers | Product-side. Needs stable snapshot/command DTOs instead of exposing mutable runtime internals. |
| Scene lowering/validation/export | `scene.rs:17593-25563` (7,971) | Converts authoring records into `RuntimeFile`, validates cross-object invariants, and produces exported documents | Product-side. Highest-risk region because it couples the authoring model to port construction invariants. |
| Scene tests | `scene.rs:25564-38224` (12,661; 120 tests), plus `crates/nuxie/tests/scene_authoring.rs` (15,674) and related authoring tests | Broad transaction, materialization, export, and live-runtime contract | Move with Scene; use this suite as the seam migration oracle. |
| Scene schema generator | `crates/nuxie/build.rs` (4,531) and `scene.rs`'s `include!(concat!(env!("OUT_DIR"), "/scene_schema.rs"))` near line 860 | Generates the Scene/silver record schema at build time | Product-side and inseparable from Scene. Extraction must move the build script/generator inputs and preserve the generated include contract. |
| Script import authorization | `crates/nuxie/src/script_import.rs:1-157` production, `158-211` tests | Ed25519/container verification; calls crate-private `File::execution_authorization_for`/authorization state | Product policy. Move to `nuxie-flow`, but expose a capability-based import seam rather than making raw authorization state public. |
| Mixed façade bridge | `crates/nuxie/src/lib.rs`, principally project-envelope classification and private bridge methods around lines 55, 154, 223-258, 307, 344, 376, 1,492, 2,983, 3,328, 4,304, and 5,600-5,733 | `File::from_runtime`; flow script/listener lifecycle; apply/rollback/command drain; artboard geometry/semantic/intrinsic-image helpers | Not an independent feature. Convert these call sites into explicit narrow bridge traits/DTOs, then move their product consumers. |

The concrete crate-private methods the extraction must account for are:

- Flow lifecycle around `nuxie/src/lib.rs:5600-5675`: `prepare_flow_scripts`, `prepare_flow_listener_actions`, rehydrate/apply, begin host cycle, rollback, and drain commands.
- Scene live/runtime queries around `lib.rs:5705-5733`: hit-test path segments with bounds, geometry path segments, retained/semantic text, and intrinsic image-dimension registration.
- `File::from_runtime` around `lib.rs:4304` and the script execution authorization capability.

These are already cross-crate calls from `nuxie` into public `nuxie-runtime` types; the missing visibility is chiefly inside the `nuxie` façade. Extraction should expose a sealed product bridge from the future base façade, not expand dozens of runtime fields to `pub`.

### Project-data converter: product model inside the port

`crates/nuxie-runtime/src/project_data_converter.rs` is 2,686 lines and is explicitly classified `scene-api` in `rust-additions.toml`:

| Lines | Region | Coupling and disposition |
| ---: | --- | --- |
| 22-507 | Public values, specs, errors, and state | Pure product/protocol model; eventual neutral-crate candidate. |
| 508-1,289 | Envelope/program/catalog compilation and evaluation | Mostly `std`/`serde`; eventual neutral-crate candidate. |
| 1,290-2,324 | Validation, coercion, formatting, interpolation | Pure-looking core, but behavior is part of runtime data-bind semantics. Extract only with parity tests. |
| 2,325-2,686 | Formula parser | Mechanically separable after the outer seam is established. |

The file is locally self-contained, but its runtime integration is not. `data_bind/context/context_value.rs` has a `RuntimeDataBindGraphConverter::Project` variant and owns conversion/evaluator state; `data_bind/data_bind_context.rs` constructs it from a script-asset envelope; both runtime and `nuxie` re-export or classify its types; Scene authors and lowers them. A direct move to `nuxie-flow` would invert the desired dependency and make the port depend on the product crate.

Two crate-private numeric-policy helpers in this file (the bounded-list constant near line 467 and helper near line 2,320) are also used by port-side Number-to-List/context conversion. If the model is eventually extracted, those policies should remain in the port or move to a dependency-neutral shared crate; they should not become a product API solely to satisfy the move.

Recommendation: first demarcate this as `project_data_converter/{model, program, evaluator, bridge}` inside `nuxie-runtime`; later extract only the pure model/program/evaluator to a neutral `nuxie-project-data` crate that both runtime and `nuxie-flow` may depend on. The runtime adapter stays in `nuxie-runtime`.

### Rust-only infrastructure that should remain port-side

The following `rust-additions.toml` entries have no one-file C++ correspondence but are not product features:

| Current path/group | Lines | Why it stays in `nuxie-runtime` |
| --- | ---: | --- |
| `data_bind_container.rs`, `data_converter.rs`, `retained_data_bind.rs` | 12 total | Tiny generated/type façade modules over port functionality. |
| `input/mod.rs`, `math/mod.rs`, `shapes/mod.rs`, `shapes/paint/mod.rs` | 174 total | Rust namespace/generated-storage coordinators. |
| `objects.rs` | 1,018 | Runtime object registry plus `include!(concat!(env!("OUT_DIR"), "/runtime_objects.rs"))`; generated by `crates/nuxie-runtime/build.rs` (707 lines). This is port construction infrastructure. |
| `properties.rs` | 826 | Generated property interpretation/storage used throughout the port. |
| `state_machine.rs`, `state_machine/instance.rs`, `state_machine/listener_types/mod.rs` | 215 total | Rust module/re-export façade for the ported state-machine implementation. |
| `viewmodel/mod.rs`, `viewmodel/runtime/mod.rs` | 58 total | Rust namespace/re-export façade for ported view-model behavior. |
| `focus.rs` | 7 | Thin public façade over retained focus behavior, not a standalone product feature. |

These codegen/facade additions total about 2,303 source lines excluding the project converter and focus façade. They should be clearly marked as Rust port infrastructure, but moving them to `nuxie-flow` would blur rather than improve the seam.

`crates/nuxie/src/command_queue.rs`, server code, and raw-text support are not included in the Nuxie-only list: their current files are explicitly mapped to upstream C++ correspondence. A richer Rust API around a ported facility does not by itself make that facility product-side.

---

## 4. Recommended seam design

### Decision

Use a **module boundary now and a crate boundary later**.

The immediate structure should make product ownership visible without changing dependencies:

```text
crates/nuxie/src/product/
  flow_session/
  scene/
    model/
    transaction/
    live/
    export/
  script_import/

crates/nuxie-runtime/src/project_data_converter/
  model.rs
  program.rs
  evaluator.rs
  bridge.rs
```

Preserve current public paths with root re-exports. This first step is a namespace/demarcation change, not an extraction. It gives reviewers an auditable boundary while the current test and parity baseline still observes exactly the same crate graph.

The long-term crate graph should avoid a cycle:

```text
nuxie-runtime  <---  nuxie-core  <---  nuxie-flow
       ^                 ^                 ^
       |                 |                 |
       +------ nuxie-project-data --------+

nuxie (umbrella compatibility crate) re-exports nuxie-core + nuxie-flow
```

`nuxie-flow` contains FlowSession, Scene/SceneTx, silver/export DTOs and lowering, and script-import policy. `nuxie-core` contains `File`, `Factory`, owned artboard/view-model handles, renderer integration, and narrow bridge traits. `nuxie-runtime` remains the C++-correspondence port plus explicitly labeled Rust port infrastructure. A small dependency-neutral `nuxie-project-data` is optional and should be introduced only after the runtime adapter has been separated from the pure evaluator.

### Public surface of the port/base layer

The extraction should not make arbitrary runtime fields public. The required surface is narrow:

1. A sealed/trusted constructor that wraps a validated `RuntimeFile` as a façade `File`, with an explicit authoring/import authorization policy.
2. A flow-runtime bridge for prepare, rehydrate, apply, host-cycle, rollback, and command-drain operations. Its inputs/outputs should be stable value DTOs, not references to internal queues.
3. Read-only artboard snapshot operations for hit geometry, retained geometry, and semantic text, plus a command for intrinsic image dimensions. Return owned snapshots so Scene cannot retain internal graph borrows.
4. A project-converter adapter owned by `nuxie-runtime`; pure program/model types may come from the neutral crate. Do not expose the runtime bind graph or evaluator cells as product API.
5. Existing upstream-corresponding runtime types only where they are already public and semantically stable. Product re-exports should live in `nuxie`/`nuxie-flow`, not expand the port's promise.

### Migration order

1. Establish the exact green parity/golden baseline and keep it pinned in every PR.
2. Land the `artboard.rs` and `state_machine_instance.rs` mechanical splits as separate PRs. This makes later ownership changes reviewable without mixing 20,000-line source moves into semantic work.
3. Demarcate `nuxie::product::{flow_session, scene, script_import}` and the project-converter submodules in place, preserving public re-exports.
4. Define the sealed flow, authoring-file, geometry-snapshot, and script-authorization bridges; migrate existing internal callers to them while everything remains in the same crate.
5. Extract the non-product façade/runtime handles into `nuxie-core`, with `nuxie` temporarily acting as an umbrella compatibility crate.
6. Move FlowSession and script import to `nuxie-flow`; move their tests and retain compatibility re-exports.
7. Move Scene model, SceneTx, export schema/generator, lowering, live adapter, and tests together. Do not split the generated schema from `build.rs` or split transaction commit from materialization in the extraction PR.
8. Isolate the project converter's pure model/program/evaluator. Extract it to a neutral crate only if doing so leaves the runtime adapter dependency pointing inward, never from `nuxie-runtime` to `nuxie-flow`.
9. Remove compatibility re-exports and tighten visibility in a later breaking-change PR, after downstream migration.

### Risk map

| Risk | Areas | Reason |
| --- | --- | --- |
| Mechanical/low | Thin namespace façades, codegen coordinators staying in place, test-module extraction, pure FlowSession validation/types | Mostly physical ownership with no runtime mutation semantics. |
| Moderate | FlowSession after bridge creation, script import after capability API, Scene DTO/export types | Public API and authorization compatibility, but limited graph mutation. |
| High | SceneTx commit/materialization, live Scene mounting, geometry/semantic access, Scene lowering and generated schema | Atomicity, identity, generated code, and live runtime invariants cross the seam. |
| High | Project-data converter extraction | The source looks pure, but evaluator state is embedded in runtime data-bind context and Number-to-List policy. |
| High | Script/listener ownership during FlowSession moves | Rehydration, rollback, command queues, and VM object lifetime must remain one coherent transaction. |

### Why baseline-first matters

`docs/parity-gap-register.md` explicitly recommends making the parity claim trustworthy before feature work and landing each subsequent change against the oracle (the “Recommended execution order” around lines 204-211). It also describes Scene as a Nuxie-only additive API (around lines 126-127) while recording FlowSession/capability-boundary gaps elsewhere in the register. That is the governing principle here: first make source moves mechanically observable by the existing correspondence, ownership, trace, golden, and silver batteries; then introduce the product seam behind those same oracles. A green build alone is insufficient because it can preserve compilation while silently weakening ownership scans or changing what “zero divergences” measures.

The two large-file splits are therefore prerequisites for, not part of, the product extraction. Each split keeps the logical Rust module unchanged and updates only physical-address consumers. The seam work then proceeds through explicit bridges and compatibility re-exports, with risky transaction/materialization changes isolated from mechanical moves.
# nuxie-runtime structure report

## Scope and baseline

This is a read-only structure analysis. It proposes no behavior, source, parity-ledger, or ownership-ledger changes.

The inspected worktree was already detached at `e8726db8ffc689d3b19f1c0a55794aa6daf9956d` (`Merge pull request #246 from TheOneLevin/feat/b6-0058-listener-viewmodel-change`, 2026-08-04). The local `origin/main` ref resolves to the same commit. The requested `git fetch origin` and redundant `git checkout --detach origin/main` could not update the shared worktree metadata because the sandbox cannot write the parent repository's `.git/worktrees/nuxie-mr-c12` directory. The analysis is therefore pinned to the locally available `origin/main` commit above, not a newly fetched remote ref. The pre-existing untracked `vtriage-capture/` directory was left untouched.

The two target files have grown beyond the sizes in the request:

| File | Current lines | Production/support | Tests |
| --- | ---: | ---: | ---: |
| `crates/nuxie-runtime/src/artboard.rs` | 23,374 | 12,039 | 11,335 (`205` tests) |
| `crates/nuxie-runtime/src/state_machine/state_machine_instance.rs` | 21,674 | 14,079 | 7,595 (`94` tests) |

The safest split mechanism is literal `include!` fragments, not Rust child modules. An included fragment remains in the original semantic module (`crate::artboard` or `crate::state_machine::state_machine_instance`), so private-name resolution, inherent method paths, sibling access, and demangled symbols remain unchanged. A `mod build;`/`#[path = ...] mod build;` split would introduce new privacy boundaries and make a supposedly mechanical PR into an API refactor.

---

## 1. Large-file split plan: `artboard.rs`

### Current structural map

| Lines | Region |
| ---: | --- |
| 1-106 | Imports and supporting aliases |
| 107-135 | Draw-frame counter and generated `Mat2D` helper |
| 136-160 | `ExternalFontAssetError` |
| 161-181 | `RuntimeArtboardInstanceIdentity` |
| 182-231 | `RuntimeScriptState` and implementations |
| 232-250 | Advancing-component and persistent-dirt fixtures |
| 251-298 | Script advance/update mode enums and implementations |
| 299-327 | Resetting/runtime-component helper types |
| 328-334 | `RuntimeTextStyleFeatureOption` |
| 335-523 | `ArtboardInstance` storage |
| 524-714 | `Clone for ArtboardInstance` |
| 715-739 | `Drop for ArtboardInstance` followed by the existing leaf `include!` declarations |
| 740-773 | Occurrence and frame-advance report types |
| 774-844 | Build context/index/text-flag helpers |
| 845-860 | `RuntimeNestedAnimationInstance` |
| 861-11,133 | Main `impl ArtboardInstance` |
| 11,134-11,147 | Focus-scroll path and focus-bounds transform helper |
| 11,148-11,187 | Small follow-up `impl ArtboardInstance` |
| 11,188-11,226 | Component-list reset helper |
| 11,227-11,595 | `impl RuntimeNestedArtboardInstance` |
| 11,596-11,611 | `impl RuntimeNestedAnimationInstance` |
| 11,612-12,039 | Free construction/nested-artboard helpers |
| 12,040-23,374 | Single `#[cfg(test)] mod tests` (`205` tests) |

The main implementation itself divides along stable responsibility boundaries:

| Lines | Logical responsibility |
| ---: | --- |
| 861-1,104 | Lifecycle, data context, and transient-clone restoration |
| 1,105-2,440 | Build occurrence relationships and constructors |
| 2,441-2,712 | Path/hit initialization and external assets |
| 2,713-3,399 | Scripting lifecycle |
| 3,400-3,989 | Object/component/audio/hit/graph/definition access |
| 3,990-4,152 | Scripted interpolators and state-machine definition selection |
| 4,153-4,480 | Dimensions, world/layout bounds, scrolling, and focus reveal |
| 4,481-5,229 | Generated property façade and text values |
| 5,230-5,568 | Animation/state-machine/view-model occurrences |
| 5,569-6,312 | Component-list lifecycle and layout |
| 6,313-7,408 | Root/nested frame advance and retained-component dispatch |
| 7,409-7,585 | Nested-layout frame propagation |
| 7,586-8,496 | Transforms, epochs, dirt, and invalidation |
| 8,497-9,191 | Update pass and five-pass state-machine settlement |
| 9,192-9,825 | Component update dispatch and script scheduling |
| 9,826-11,133 | Property-change callbacks, nested controls, and collapse |

### Proposed physical layout

Keep `artboard.rs` as the owner. It retains imports, supporting types, `ArtboardInstance`, `Clone`, `Drop`, the existing leaf includes, occurrence/report/build helper types, and an ordered series of literal includes. This preserves declaration order where it matters and makes all fragments part of `crate::artboard`.

```text
src/
  artboard.rs                         owner/types, existing leaf includes, ordered include! list
  artboard/
    build.rs                          current 861-2712
    script_runtime.rs                 current 2713-3399
    access.rs                         current 3400-3989
    definitions_and_layout.rs         current 3990-4480
    properties.rs                     current 4481-5568
    component_lists.rs                current 5569-6312
    advance.rs                        current 6313-7585
    dirt.rs                           current 7586-8496
    update.rs                         current 8497-9825
    callbacks.rs                      current 9826-11133
    nested.rs                         current 11134-12039
    tests.rs                          body of current tests module
```

The production fragments are intentionally in execution/dependency order rather than alphabetical order. `tests.rs` should be included inside the original test module:

```rust
#[cfg(test)]
mod tests {
    include!("artboard/tests.rs");
}
```

This retains the test module's name and access. It also gives the attribution checker a deliberately exempt `tests.rs` leaf.

For every production file in the layout above, the owner must contain a literal `include!("artboard/<name>.rs");` at the point where that region previously appeared. Do not use `#[path]`, `mod`, or `pub(crate) mod`: each would create a child-module/privacy boundary. Every such production fragment is attribution-scanned and must be named by the B6 rows below; only `tests.rs` receives the test-source exemption.

### Lockstep correspondence and checker manifest

The following table is the required per-file edit manifest. “B6 rows” are `[[files]]` entries in `file-correspondence-manifest.toml`; add the fragment's path to those rows' semicolon-separated `rust_module` values and update their C17 retained-module notes. Keep status, upstream pins, C++ paths, verification text, and correspondence claims unchanged. The row list is conservative: a later provenance cleanup can narrow it, but narrowing must not be mixed into this mechanical split.

| New file | B6 rows that must name it | Frame-loop/checker bindings that must move with it |
| --- | --- | --- |
| `artboard/build.rs` | `B6-0001`, `B6-0094`, `B6-0115`, `B6-0117`, `B6-0123`, `B6-0125`, `B6-0129`, `B6-0131`, `B6-0142`, `B6-0146`, `B6-0202`, `B6-0258`, `B6-0303`, `B6-0304`, `B6-0305`, `B6-0331`, `B6-0361`, `B6-0362`, `B6-0365`, `B6-0368`, `B6-0385`, `B6-0388`, `B6-0399`, `B6-0405`, `B6-0408` | Repoint `summarize_trace.py` dependency-build and IK-chain-build source scans when their anchors land here. Repoint matching `runtime-frame-loop-gaps.toml` owner-boundary entries. |
| `artboard/script_runtime.rs` | `B6-0077`, `B6-0194`, `B6-0200`, `B6-0322`, `B6-0326` | Repoint the script-lifecycle citations in `runtime-frame-loop-ownership.toml` and any corresponding synthetic test fixture paths. |
| `artboard/access.rs` | `B6-0094`, `B6-0095`, `B6-0123`, `B6-0146`, `B6-0203`, `B6-0204`, `B6-0205` | Repoint any live-draw/renderer-interface lifecycle citations whose named methods move here. |
| `artboard/definitions_and_layout.rs` | `B6-0077`, `B6-0094`, `B6-0258`, `B6-0303`, `B6-0304`, `B6-0305`, `B6-0385`, `B6-0388` | Repoint focus-retained-tree and layout lifecycle citations, plus matching owner-boundary allow entries. |
| `artboard/properties.rs` | `B6-0067`, `B6-0077`, `B6-0094`, `B6-0194`, `B6-0200`, `B6-0352`, `B6-0355`, `B6-0385`, `B6-0388`, `B6-0399`, `B6-0404`, `B6-0405`, `B6-0408` | `B6-0067` currently has only one Rust module: **replace** `artboard.rs` with this fragment rather than adding a second path. Repoint property/text lifecycle citations and corresponding owner-boundary entries. |
| `artboard/component_lists.rs` | `B6-0095`, `B6-0258`, `B6-0303`, `B6-0304`, `B6-0305` | Repoint component-list and nested-layout lifecycle citations and matching test fixtures. |
| `artboard/advance.rs` | `B6-0001`, `B6-0059`, `B6-0061`, `B6-0063`, `B6-0064`, `B6-0077`, `B6-0094`, `B6-0095`, `B6-0194`, `B6-0200`, `B6-0258`, `B6-0303`, `B6-0304`, `B6-0305`, `B6-0322`, `B6-0326`, `B6-0388` | In `runtime-frame-loop-ownership.toml`, move `artboard.advance`'s `rust_file` and the lifecycle citations for artboard/nested advance. Repoint `summarize_trace.py` advancing/resetting dispatch scans as applicable. Repoint relevant gap allow rows and `test_check.py` advance fixtures. |
| `artboard/dirt.rs` | `B6-0094`, `B6-0123`, `B6-0258`, `B6-0331`, `B6-0352`, `B6-0355`, `B6-0357`, `B6-0361`, `B6-0362`, `B6-0365`, `B6-0368`, `B6-0385`, `B6-0388`, `B6-0399`, `B6-0405`, `B6-0408` | Move `component.dirt`, `component.dependents` (`pub fn add_dirt(`), and transform/dirt lifecycle citations as appropriate. Repoint `summarize_trace.py`'s two add-dirt anchors, dirt consumptions, owner resolutions, and skin-buffer scans according to their final physical locations. Repoint matching gap rows. |
| `artboard/update.rs` | `B6-0001`, `B6-0094`, `B6-0115`, `B6-0117`, `B6-0123`, `B6-0125`, `B6-0129`, `B6-0131`, `B6-0142`, `B6-0194`, `B6-0200`, `B6-0203`, `B6-0204`, `B6-0205`, `B6-0258`, `B6-0303`, `B6-0304`, `B6-0305`, `B6-0322`, `B6-0326`, `B6-0331`, `B6-0352`, `B6-0355`, `B6-0357`, `B6-0361`, `B6-0362`, `B6-0365`, `B6-0368`, `B6-0385`, `B6-0388`, `B6-0399`, `B6-0404`, `B6-0405`, `B6-0408` | Move `component.update_order` (`fn update_components_with_hook_recording`) and `artboard.update_pass` (`pub fn update_pass(`) `rust_file` values and lifecycle citations. Repoint retained update/settlement gap rows, trace scans, and test fixtures. |
| `artboard/callbacks.rs` | `B6-0059`, `B6-0061`, `B6-0063`, `B6-0064`, `B6-0077`, `B6-0094`, `B6-0123`, `B6-0125`, `B6-0129`, `B6-0131`, `B6-0142`, `B6-0194`, `B6-0200`, `B6-0258`, `B6-0303`, `B6-0304`, `B6-0305`, `B6-0331`, `B6-0352`, `B6-0355`, `B6-0357`, `B6-0361`, `B6-0362`, `B6-0365`, `B6-0368`, `B6-0385`, `B6-0388`, `B6-0399`, `B6-0404`, `B6-0405`, `B6-0408` | Change `check_layout_style_handlers.py`'s `RUST_ARTBOARD_MODULE` to this fragment if both display-change markers remain here; otherwise make the checker accept the two explicit physical files. Repoint callback lifecycle citations, gap rows, and fixtures. |
| `artboard/nested.rs` | `B6-0059`, `B6-0061`, `B6-0063`, `B6-0064`, `B6-0077`, `B6-0094`, `B6-0095`, `B6-0258`, `B6-0303`, `B6-0304`, `B6-0305` | Repoint nested-artboard advance/layout citations, gap allow entries, and nested fixtures. |
| `artboard/tests.rs` | None. `rust_attribution.py` excludes a source whose stem is `tests`. | Move the artboard test body unchanged. Update the seven relative `include_bytes!`/fixture paths currently near old lines 19,951, 20,001, 20,168, 20,542, 20,597, 20,612, and 23,311 because the file becomes one directory deeper. Prefer `concat!(env!("CARGO_MANIFEST_DIR"), "/...")` for fixture stability. The `env!` use near old line 14,290 remains valid. |

#### Global artboard bindings

These bindings span more than one fragment and therefore belong in the same PR:

- `rust_attribution.py` discovers all production `.rs` files below `crates/nuxie-runtime/src`. Every new production fragment must be listed in at least one `file-correspondence-manifest.toml` row and must **not** be added to `rust-additions.toml`. The checker exempts `tests.rs`, names ending in `_tests`, and files below a `tests/` path.
- The manifest's scatter ratchet is already at its maximum: 155 rows currently have multiple Rust modules. Except for `B6-0067`, all affected artboard rows are already multi-module. Replacing the `B6-0067` root path prevents the split from increasing that count.
- Update every affected C17 note that enumerates retained Rust modules. Do not change the underlying parity classification merely because the code's physical address changed.
- `docs/runtime-frame-loop-ownership.toml` has artboard citations in `focus.retained_tree`, `component.identity`, `component.dirt`, `component.dependents`, `component.update_order`, `component.transforms`, `component.clone_drop`, `artboard.advance`, `artboard.update_pass`, `nested_artboard.advance`, `artboard.live_draw`, and `renderer.interface_boundary`. Keep `component.clone_drop` and its `pub struct ArtboardInstance` anchor on the root; change each other `rust_file` or `rust:path:line-range` to its physical fragment.
- `docs/runtime-frame-loop-gaps.toml` has 25 owner-boundary allow entries pointing at `artboard.rs` (entries 0-7, 11-22, and 29-34 in current order). Repoint each to the fragment that contains its anchor. A literal move should preserve its token-based `site_hash`; a changed hash is a signal that the split was not mechanical.
- `tools/runtime-frame-loop-port/summarize_trace.py` hard-codes `artboard.rs` nine times for add-dirt, dirt consumption, owner resolution, dependency build, skin rebuild, advancing dispatch, resetting dispatch, and IK-chain discovery. Give each query its actual fragment path. The demangled owner should remain `artboard::ArtboardInstance` because `include!` does not introduce a child module.
- `tools/runtime-frame-loop-port/check_layout_style_handlers.py` hard-codes the artboard source while looking for `propagate_layout_component_display_changed` and the display property key. Point it to the fragment(s) containing those markers.
- `tools/runtime-frame-loop-port/test_check.py` contains at least 14 exact artboard paths covering probe gating, layer occurrence, live-advance ratchets, registry site hashes/substitution, and trait-anchor behavior. Update each synthetic fixture to the fragment that owns the tested anchor; do not perform a blind string replacement.
- Leave the existing root-level includes (`nested_artboard.rs`, `artboard_component_list.rs`, `artboard_list_map_rule.rs`, `artboard_referencer.rs`, `bindable_artboard.rs`, `nested_artboard_layout.rs`, `nested_artboard_leaf.rs`, `component_origin.rs`, bone/weight, profiler/profile) where they are. Moving them under `artboard/` changes relative include resolution and correspondence ownership.
- Keep `crates/nuxie-runtime/src/lib.rs`'s `mod artboard;` unchanged. The public and crate-private paths must not acquire an extra `artboard::<fragment>` segment.

### Mechanical PR sequence and proof

One PR should contain only this file's split and the path/anchor bookkeeping above. Recommended commit sequence within that PR:

1. Extract `tests.rs`, fix only its fixture paths, and prove the same 205 tests are collected.
2. Extract production regions in original order using `include!`; make no renames, visibility edits, formatting sweeps, or logic edits.
3. Update correspondence C17/path fields and all frame-loop physical anchors.
4. Update checker source locations and synthetic fixtures, including an assertion that demangled ownership remains `artboard::ArtboardInstance`.

Proof battery:

```text
make rust-sources-fresh
make cpp-probe
make runtime-frame-loop-port-test
make runtime-frame-loop-port-check
make rust-attribution-check
cargo check --workspace
cargo test -p nuxie-runtime
cargo test -p nuxie --features scripting
make scripted-golden-compare
make silver-corpus-test
```

The PR is structure-preserving only if parity row counts/statuses, trace landmarks, owner-boundary hashes, test counts, and golden/silver outputs are unchanged.

---

## 2. Large-file split plan: `state_machine_instance.rs`

### Current structural map

| Lines | Region |
| ---: | --- |
| 1-93 | Imports and ownership commentary |
| 94-255 | Nested event-chain/notify phases, trace structures, and trace recording |
| 256-312 | Semantic-node resolver and audio/event seam types |
| 313-1,271 | `RuntimeHitResult`, `HitComponent`, and hierarchy implementations |
| 1,272-1,467 | Nested notifier, focus, semantic, and state helper types |
| 1,468-1,474 | `RuntimeDataContextBindError` |
| 1,475-1,711 | `StateMachineInstance` storage |
| 1,712-1,744 | Data-bind occurrence/pointer/executor structures |
| 1,745-2,147 | Listener-action executor implementations |
| 2,148-2,335 | `Clone for StateMachineInstance` |
| 2,336-2,356 | `Drop for StateMachineInstance` |
| 2,357-2,638 | Free helpers and view-model listener types |
| 2,639-13,938 | Main `impl StateMachineInstance` |
| 13,939-13,953 | `runtime_owned_font` helper |
| 13,954-14,079 | `view_model_listener_tests` (`1` test) |
| 14,080-21,674 | `scripted_listener_action_tests` (`93` tests) |

The main implementation divides as follows:

| Lines | Logical responsibility |
| ---: | --- |
| 2,639-2,932 | Teardown, enrollment, focus manager, clone rebind |
| 2,933-3,741 | Construction and listener/layer initialization |
| 3,742-5,102 | Scripting, hydration, and adoption |
| 5,103-6,007 | State/input/focus/semantic/keyboard/gamepad operations |
| 6,008-7,487 | Hit/listener/pointer pipeline |
| 7,488-8,525 | Event notification, deferred callbacks, listener actions |
| 8,526-9,100 | Direct bind targets and value readers |
| 9,101-10,748 | Default/imported/owned source setters and relinking |
| 10,749-12,599 | Data-context bind/rebind, listener cells, bind updates |
| 12,600-13,938 | Reports, advance/apply, nested event dispatch, settlement |

### Proposed physical layout

```text
src/state_machine/
  state_machine_instance.rs               owner/types, hit hierarchy, lifecycle, ordered include! list
  state_machine_instance/
    construction.rs                       current 2639-3741
    scripted_objects.rs                   current 3742-5102
    input_focus.rs                        current 5103-6007
    pointer.rs                            current 6008-7487
    events_actions.rs                     current 7488-8525
    bind_targets.rs                       current 8526-9100
    bind_sources.rs                       current 9101-10748
    data_context.rs                       current 10749-12599
    advance.rs                            current 12600-13953
    tests/
      view_model_listener.rs              body of current first test module
      scripted_listener_actions.rs         body of current second test module
```

Retain the two existing module names in the root and include only their bodies. This preserves test paths and any name-sensitive filtering:

```rust
#[cfg(test)]
mod view_model_listener_tests {
    include!("state_machine_instance/tests/view_model_listener.rs");
}
```

Use the same pattern for `scripted_listener_action_tests`.

For every production file in this layout, place `include!("state_machine_instance/<name>.rs");` in the root at the original region boundary. Do not use `#[path]` or a `mod` declaration. Both test bodies remain inside their original outer modules and use literal includes into the `tests/` directory, which is what preserves their module names while exempting the physical leaves from attribution.

### Lockstep correspondence and checker manifest

| New file | B6 rows that must name it | Frame-loop/checker bindings that must move with it |
| --- | --- | --- |
| `state_machine_instance/construction.rs` | `B6-0077`, `B6-0194`, `B6-0195`, `B6-0196`, `B6-0200`, `B6-0310` | Repoint construction/enrollment lifecycle citations and matching synthetic fixtures. Keep `state_machine.collections`' `pub struct StateMachineInstance` anchor on the root. |
| `state_machine_instance/scripted_objects.rs` | `B6-0077`, `B6-0175`, `B6-0194`, `B6-0195`, `B6-0196`, `B6-0200` | Repoint scripting/hydration lifecycle citations and scripted-action fixture paths. |
| `state_machine_instance/input_focus.rs` | `B6-0077`, `B6-0083` | Repoint focus/semantic/keyboard/gamepad citations and tests. |
| `state_machine_instance/pointer.rs` | `B6-0077`, `B6-0083` | Repoint hit/pointer/listener lifecycle citations and FL-C4/layer/hit fixtures. |
| `state_machine_instance/events_actions.rs` | `B6-0058`, `B6-0077`, `B6-0440` | Repoint event reporting, firing/bubbling, and action-dispatch citations/fixtures. `state_machine.actions`' primary anchor remains on the root if the listener-action executor stays there. |
| `state_machine_instance/bind_targets.rs` | `B6-0058`, `B6-0194`, `B6-0196`, `B6-0200`, `B6-0440` | Repoint bind-target and view-model listener citations/fixtures. |
| `state_machine_instance/bind_sources.rs` | `B6-0058`, `B6-0175`, `B6-0194`, `B6-0195`, `B6-0196`, `B6-0200`, `B6-0440` | Repoint source/relink/data-converter citations and bind fixtures. |
| `state_machine_instance/data_context.rs` | `B6-0058`, `B6-0077`, `B6-0194`, `B6-0195`, `B6-0196`, `B6-0200`, `B6-0440` | Repoint data-context lifecycle citations and listener-cell/bind fixtures. |
| `state_machine_instance/advance.rs` | `B6-0077`, `B6-0194`, `B6-0195`, `B6-0196`, `B6-0200`, `B6-0310`, `B6-0440` | Move `state_machine.advance`'s `rust_file` and `advance_with_report_mode` anchor here. Repoint state-machine event/action/reporting lifecycle citations and advance/keyframe fixtures. Include `runtime_owned_font` in this fragment to avoid an artificial one-function file. |
| `state_machine_instance/tests/view_model_listener.rs` | None; it resides below a `tests/` path. | Move the body unchanged and preserve the outer module name. |
| `state_machine_instance/tests/scripted_listener_actions.rs` | None; it resides below a `tests/` path. | Move the body unchanged. Replace or correct the relative fixture path near old line 19,011 (`../../../../fixtures/sync/vm_listener_fire_event.riv`) because the source becomes two directories deeper; prefer a `CARGO_MANIFEST_DIR`-anchored path. |

#### Global state-machine bindings

- All production fragments must be named by `file-correspondence-manifest.toml`, never `rust-additions.toml`. Every affected SMI row is already multi-module, so this split does not increase the manifest's 155-row scatter count.
- Update C17 retained-module notes for `B6-0058`, `B6-0077`, `B6-0083`, `B6-0175`, `B6-0194`, `B6-0195`, `B6-0196`, `B6-0200`, `B6-0310`, and `B6-0440`; leave the substantive parity assertions untouched.
- `docs/runtime-frame-loop-ownership.toml` cites this file from `focus.retained_tree`, `state_machine.collections`, `state_machine.advance`, `state_machine.events`, `state_machine.actions`, and `event.reporting`. Keep root anchors for the struct, stored event collection, and action executor. Repoint method/lifecycle citations to physical fragments.
- `tools/runtime-frame-loop-port/check.py` currently treats one hard-coded SMI file as the nested-event owner and excludes it from outside-owner scans. Once methods are in included fragments, a path-only exclusion would falsely flag the fragments. Make owner discovery include-aware: recursively expand literal local `include!("...")` files in source order for export analysis, and treat every expanded file as the same logical owner. Do not broaden the exclusion to a directory glob.
- Add checker unit coverage for the include-aware logical owner: an exported nested-event method in an included fragment must be accepted, while the same method in an unrelated file must still fail. Preserve syntax parsing and duplicate-anchor detection.
- `tools/runtime-frame-loop-port/test_check.py` has at least 14 exact SMI references, plus constructed/split path strings, covering FL-C4, lifecycle, layer, hit, bind, events, firing/bubbling, advance, owner exports, keyframes, focus, and semantic behavior. Point each fixture to the actual fragment; do not simply substitute the root path everywhere.
- `docs/runtime-frame-loop-gaps.toml` has no current SMI owner-boundary allow row to migrate.
- Keep `crates/nuxie-runtime/src/state_machine.rs`'s `pub(crate) mod state_machine_instance;` and its re-exports unchanged. Literal includes keep all symbols at the original module path.

### Mechanical PR sequence and proof

This should be a second, independent PR after the artboard split has landed. It uses the same proof battery. The checker must first learn the logical-owner/include relationship in the same PR, because moving the methods without that update creates false ownership failures. Apart from that narrowly required path-awareness, no checker policy should change.

Success means the same 94 tests are collected, no exported API path changes, the correspondence scatter count stays at 155, no new owner-boundary allowance appears, and the full battery shown in the artboard plan remains green.

---

## 3. Nuxie-only seam inventory

### Classification method

The authoritative starting point is correspondence metadata:

- Production Rust paths mapped to upstream C++ in `file-correspondence-manifest.toml` are port-side unless a smaller product-only region can be proven.
- Production Rust paths listed in `rust-additions.toml` are intentional Rust-only additions. The relevant ownership labels are `flowsession-abi`, `scene-api`, and generated/codegen infrastructure.
- `build.rs` is outside the attribution scanner's `src/` roots, so it must be inspected manually.
- A convenience façade is not automatically product-side. Thin Rust-only namespace/generated-storage coordinators that serve the port stay with the port even though no one C++ file corresponds to them.

This prevents two opposite mistakes: burying product authoring features inside the port forever, and extracting Rust implementation infrastructure merely because its file has no one-to-one C++ source.

### Product features and mixed glue

| Region | Current size and location | Runtime internals reached / extraction surface | Placement assessment |
| --- | --- | --- | --- |
| FlowSession wire/domain model | `crates/nuxie/src/flow_session.rs:21-621` (601 lines) | Public `File`, `Factory`, renderer, artboard, view-model, and state-machine types | Clean product vocabulary; move to `nuxie-flow` after dependency direction is fixed. |
| FlowSession orchestration | `flow_session.rs:622-2163` (1,542) | Calls crate-private script preparation, listener preparation, rehydration, apply/rollback, host-cycle, and command-drain methods on the `nuxie` façade | Product-side, but requires a narrow public/sealed flow-runtime bridge before crate extraction. Moderate coupling. |
| FlowSession value snapshots | `flow_session.rs:2164-3105` (942) | Traverses public runtime-owned view-model/value objects and builds its own graph/arena | Product-side and cohesive. Mostly mechanical after bridge creation. |
| FlowSession validation/accounting | `flow_session.rs:3106-3545` (440) | Product protocol limits and payload validation | Clean extraction candidate. |
| FlowSession mutation/apply/diff | `flow_session.rs:3546-4515` (970) | Applies mutations through runtime view-model/state-machine handles; shares rollback/command sequencing with façade-private methods | Product-side; transactional sequencing makes it medium risk. |
| FlowSession selection/catalog | `flow_session.rs:4516-5020` (505) | File/artboard/state-machine discovery | Product-side; mostly public runtime surface. |
| FlowSession tests | `flow_session.rs:5021-7227` (2,207; 40 tests), plus `crates/nuxie/tests/flow_session_contract.rs` (335) | Exercises the public ABI, VM lifecycle, and transactional behavior | Move with FlowSession. Preserve public names through compatibility re-exports during migration. |
| Scene contract and IDs | `crates/nuxie/src/scene.rs:28-1187` (1,160) | Public runtime/binary identities and errors | Product-side, cleanly demarcatable. |
| Scene durable definitions/index/specs | `scene.rs:1188-3401` (2,214) | `nuxie_binary::AuthoringRecord`, property/value types, runtime file identities | Product-side authoring model. |
| Scene hierarchy/materialization | `scene.rs:3402-5622` (2,221) | Constructs/reads `RuntimeFile`; directly touches `materialized.file.runtime` in a handful of places even though `File::runtime()` already exists | Product-side but high coupling. Replace direct private field access with the existing public accessor before moving. |
| Silver/scene export schema | `scene.rs:5623-6819` (1,197) | Public exported-record/document DTOs; lowering later feeds `RuntimeFile::from_authoring_records` | Product-side. The schema and its generated counterpart must move together. |
| Scene store/live mount API | `scene.rs:6820-9099` (2,280) | Owns mounted `File`/artboard/view-model state and coordinates transactions | Product-side; medium-to-high coupling. |
| SceneTx and subordinate transactions | `scene.rs:9100-15589` (6,490) | `SceneTx`, `DataConverterTx`, `VmTx`, `AnimTx`, and `MachineTx` mutate the authoring graph, then materialize into runtime/binary types | Core product authoring seam. Cohesive, but atomic commit/materialization behavior makes the move high risk. |
| Scene frame queries/live mutation | `scene.rs:15590-17592` (2,003) | Hit/geometry/semantic queries and intrinsic image sizing reach crate-private `OwnedArtboardInstance` helpers | Product-side. Needs stable snapshot/command DTOs instead of exposing mutable runtime internals. |
| Scene lowering/validation/export | `scene.rs:17593-25563` (7,971) | Converts authoring records into `RuntimeFile`, validates cross-object invariants, and produces exported documents | Product-side. Highest-risk region because it couples the authoring model to port construction invariants. |
| Scene tests | `scene.rs:25564-38224` (12,661; 120 tests), plus `crates/nuxie/tests/scene_authoring.rs` (15,674) and related authoring tests | Broad transaction, materialization, export, and live-runtime contract | Move with Scene; use this suite as the seam migration oracle. |
| Scene schema generator | `crates/nuxie/build.rs` (4,531) and `scene.rs`'s `include!(concat!(env!("OUT_DIR"), "/scene_schema.rs"))` near line 860 | Generates the Scene/silver record schema at build time | Product-side and inseparable from Scene. Extraction must move the build script/generator inputs and preserve the generated include contract. |
| Script import authorization | `crates/nuxie/src/script_import.rs:1-157` production, `158-211` tests | Ed25519/container verification; calls crate-private `File::execution_authorization_for`/authorization state | Product policy. Move to `nuxie-flow`, but expose a capability-based import seam rather than making raw authorization state public. |
| Mixed façade bridge | `crates/nuxie/src/lib.rs`, principally project-envelope classification and private bridge methods around lines 55, 154, 223-258, 307, 344, 376, 1,492, 2,983, 3,328, 4,304, and 5,600-5,733 | `File::from_runtime`; flow script/listener lifecycle; apply/rollback/command drain; artboard geometry/semantic/intrinsic-image helpers | Not an independent feature. Convert these call sites into explicit narrow bridge traits/DTOs, then move their product consumers. |

The concrete crate-private methods the extraction must account for are:

- Flow lifecycle around `nuxie/src/lib.rs:5600-5675`: `prepare_flow_scripts`, `prepare_flow_listener_actions`, rehydrate/apply, begin host cycle, rollback, and drain commands.
- Scene live/runtime queries around `lib.rs:5705-5733`: hit-test path segments with bounds, geometry path segments, retained/semantic text, and intrinsic image-dimension registration.
- `File::from_runtime` around `lib.rs:4304` and the script execution authorization capability.

These are already cross-crate calls from `nuxie` into public `nuxie-runtime` types; the missing visibility is chiefly inside the `nuxie` façade. Extraction should expose a sealed product bridge from the future base façade, not expand dozens of runtime fields to `pub`.

### Project-data converter: product model inside the port

`crates/nuxie-runtime/src/project_data_converter.rs` is 2,686 lines and is explicitly classified `scene-api` in `rust-additions.toml`:

| Lines | Region | Coupling and disposition |
| ---: | --- | --- |
| 22-507 | Public values, specs, errors, and state | Pure product/protocol model; eventual neutral-crate candidate. |
| 508-1,289 | Envelope/program/catalog compilation and evaluation | Mostly `std`/`serde`; eventual neutral-crate candidate. |
| 1,290-2,324 | Validation, coercion, formatting, interpolation | Pure-looking core, but behavior is part of runtime data-bind semantics. Extract only with parity tests. |
| 2,325-2,686 | Formula parser | Mechanically separable after the outer seam is established. |

The file is locally self-contained, but its runtime integration is not. `data_bind/context/context_value.rs` has a `RuntimeDataBindGraphConverter::Project` variant and owns conversion/evaluator state; `data_bind/data_bind_context.rs` constructs it from a script-asset envelope; both runtime and `nuxie` re-export or classify its types; Scene authors and lowers them. A direct move to `nuxie-flow` would invert the desired dependency and make the port depend on the product crate.

Two crate-private numeric-policy helpers in this file (the bounded-list constant near line 467 and helper near line 2,320) are also used by port-side Number-to-List/context conversion. If the model is eventually extracted, those policies should remain in the port or move to a dependency-neutral shared crate; they should not become a product API solely to satisfy the move.

Recommendation: first demarcate this as `project_data_converter/{model, program, evaluator, bridge}` inside `nuxie-runtime`; later extract only the pure model/program/evaluator to a neutral `nuxie-project-data` crate that both runtime and `nuxie-flow` may depend on. The runtime adapter stays in `nuxie-runtime`.

### Rust-only infrastructure that should remain port-side

The following `rust-additions.toml` entries have no one-file C++ correspondence but are not product features:

| Current path/group | Lines | Why it stays in `nuxie-runtime` |
| --- | ---: | --- |
| `data_bind_container.rs`, `data_converter.rs`, `retained_data_bind.rs` | 12 total | Tiny generated/type façade modules over port functionality. |
| `input/mod.rs`, `math/mod.rs`, `shapes/mod.rs`, `shapes/paint/mod.rs` | 174 total | Rust namespace/generated-storage coordinators. |
| `objects.rs` | 1,018 | Runtime object registry plus `include!(concat!(env!("OUT_DIR"), "/runtime_objects.rs"))`; generated by `crates/nuxie-runtime/build.rs` (707 lines). This is port construction infrastructure. |
| `properties.rs` | 826 | Generated property interpretation/storage used throughout the port. |
| `state_machine.rs`, `state_machine/instance.rs`, `state_machine/listener_types/mod.rs` | 215 total | Rust module/re-export façade for the ported state-machine implementation. |
| `viewmodel/mod.rs`, `viewmodel/runtime/mod.rs` | 58 total | Rust namespace/re-export façade for ported view-model behavior. |
| `focus.rs` | 7 | Thin public façade over retained focus behavior, not a standalone product feature. |

These codegen/facade additions total about 2,303 source lines excluding the project converter and focus façade. They should be clearly marked as Rust port infrastructure, but moving them to `nuxie-flow` would blur rather than improve the seam.

`crates/nuxie/src/command_queue.rs`, server code, and raw-text support are not included in the Nuxie-only list: their current files are explicitly mapped to upstream C++ correspondence. A richer Rust API around a ported facility does not by itself make that facility product-side.

---

## 4. Recommended seam design

### Decision

Use a **module boundary now and a crate boundary later**.

The immediate structure should make product ownership visible without changing dependencies:

```text
crates/nuxie/src/product/
  flow_session/
  scene/
    model/
    transaction/
    live/
    export/
  script_import/

crates/nuxie-runtime/src/project_data_converter/
  model.rs
  program.rs
  evaluator.rs
  bridge.rs
```

Preserve current public paths with root re-exports. This first step is a namespace/demarcation change, not an extraction. It gives reviewers an auditable boundary while the current test and parity baseline still observes exactly the same crate graph.

The long-term crate graph should avoid a cycle:

```text
nuxie-runtime  <---  nuxie-core  <---  nuxie-flow
       ^                 ^                 ^
       |                 |                 |
       +------ nuxie-project-data --------+

nuxie (umbrella compatibility crate) re-exports nuxie-core + nuxie-flow
```

`nuxie-flow` contains FlowSession, Scene/SceneTx, silver/export DTOs and lowering, and script-import policy. `nuxie-core` contains `File`, `Factory`, owned artboard/view-model handles, renderer integration, and narrow bridge traits. `nuxie-runtime` remains the C++-correspondence port plus explicitly labeled Rust port infrastructure. A small dependency-neutral `nuxie-project-data` is optional and should be introduced only after the runtime adapter has been separated from the pure evaluator.

### Public surface of the port/base layer

The extraction should not make arbitrary runtime fields public. The required surface is narrow:

1. A sealed/trusted constructor that wraps a validated `RuntimeFile` as a façade `File`, with an explicit authoring/import authorization policy.
2. A flow-runtime bridge for prepare, rehydrate, apply, host-cycle, rollback, and command-drain operations. Its inputs/outputs should be stable value DTOs, not references to internal queues.
3. Read-only artboard snapshot operations for hit geometry, retained geometry, and semantic text, plus a command for intrinsic image dimensions. Return owned snapshots so Scene cannot retain internal graph borrows.
4. A project-converter adapter owned by `nuxie-runtime`; pure program/model types may come from the neutral crate. Do not expose the runtime bind graph or evaluator cells as product API.
5. Existing upstream-corresponding runtime types only where they are already public and semantically stable. Product re-exports should live in `nuxie`/`nuxie-flow`, not expand the port's promise.

### Migration order

1. Establish the exact green parity/golden baseline and keep it pinned in every PR.
2. Land the `artboard.rs` and `state_machine_instance.rs` mechanical splits as separate PRs. This makes later ownership changes reviewable without mixing 20,000-line source moves into semantic work.
3. Demarcate `nuxie::product::{flow_session, scene, script_import}` and the project-converter submodules in place, preserving public re-exports.
4. Define the sealed flow, authoring-file, geometry-snapshot, and script-authorization bridges; migrate existing internal callers to them while everything remains in the same crate.
5. Extract the non-product façade/runtime handles into `nuxie-core`, with `nuxie` temporarily acting as an umbrella compatibility crate.
6. Move FlowSession and script import to `nuxie-flow`; move their tests and retain compatibility re-exports.
7. Move Scene model, SceneTx, export schema/generator, lowering, live adapter, and tests together. Do not split the generated schema from `build.rs` or split transaction commit from materialization in the extraction PR.
8. Isolate the project converter's pure model/program/evaluator. Extract it to a neutral crate only if doing so leaves the runtime adapter dependency pointing inward, never from `nuxie-runtime` to `nuxie-flow`.
9. Remove compatibility re-exports and tighten visibility in a later breaking-change PR, after downstream migration.

### Risk map

| Risk | Areas | Reason |
| --- | --- | --- |
| Mechanical/low | Thin namespace façades, codegen coordinators staying in place, test-module extraction, pure FlowSession validation/types | Mostly physical ownership with no runtime mutation semantics. |
| Moderate | FlowSession after bridge creation, script import after capability API, Scene DTO/export types | Public API and authorization compatibility, but limited graph mutation. |
| High | SceneTx commit/materialization, live Scene mounting, geometry/semantic access, Scene lowering and generated schema | Atomicity, identity, generated code, and live runtime invariants cross the seam. |
| High | Project-data converter extraction | The source looks pure, but evaluator state is embedded in runtime data-bind context and Number-to-List policy. |
| High | Script/listener ownership during FlowSession moves | Rehydration, rollback, command queues, and VM object lifetime must remain one coherent transaction. |

### Why baseline-first matters

`docs/parity-gap-register.md` explicitly recommends making the parity claim trustworthy before feature work and landing each subsequent change against the oracle (the “Recommended execution order” around lines 204-211). It also describes Scene as a Nuxie-only additive API (around lines 126-127) while recording FlowSession/capability-boundary gaps elsewhere in the register. That is the governing principle here: first make source moves mechanically observable by the existing correspondence, ownership, trace, golden, and silver batteries; then introduce the product seam behind those same oracles. A green build alone is insufficient because it can preserve compilation while silently weakening ownership scans or changing what “zero divergences” measures.

The two large-file splits are therefore prerequisites for, not part of, the product extraction. Each split keeps the logical Rust module unchanged and updates only physical-address consumers. The seam work then proceeds through explicit bridges and compatibility re-exports, with risky transaction/materialization changes isolated from mechanical moves.