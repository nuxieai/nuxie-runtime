# FL-C5 synthesis directives (orchestrator-binding decisions)

These decisions bind the closure checklist and implementation spec. They were
made against the W1–W4 inventories and the ownership/manifest ledgers.

## 1. Scope

FL-C5 = exactly four pinned files: `state_machine.hpp/.cpp`,
`state_machine_instance.hpp/.cpp`. Rust destinations:

- `crates/nuxie-runtime/src/state_machine/state_machine.rs` (NEW — definition
  owner ported from `state_machine.cpp`).
- `crates/nuxie-runtime/src/state_machine/state_machine_instance.rs` (NEW —
  instance owner ported from `state_machine_instance.cpp`).
- `state_machine.rs` and `state_machine/instance.rs` become thin entry
  points/re-exports preserving every public API in W4 §C.
- `state_machine_layer_instance.rs` remains the owner of the
  `StateMachineLayerInstance` class (already corresponds; FL-C3-accepted
  behavior; FL-C5 touches it only where W4 flags divergence).

Out-of-scope, each with a RECORDED seam + owning row:
- `src/listener_group.cpp` pointer-group internals → FL-D row.
- `src/animation/text_input_listener_group.cpp` → FL-E row.
- `src/input/gamepad_batch.cpp` (`submitGamepadsFromBuffer` definition) → its
  own pending manifest row.
- `src/input/focus_manager.cpp` internals → its row (`focus.rs`, DIVERGENT).
- `src/semantic/semantic_manager.cpp`, `semantic_data.cpp` → their rows
  (absent). Manager-dependent members (`enableSemantics`,
  `setExternalSemanticManager`, `semanticManager`, `fireSemanticAction` node
  lookup) are recorded dependency gaps owned by those rows; FL-C5 ports the
  queue/process/dispatch orchestration that lives in its own files.
- Audio playback pass in `notifyEventListeners` → `audio_event.cpp` row
  (absent). FL-C5 ports ordering up to the seam; `playsAudio` constant is
  ported; the play call is a recorded dependency gap.
- Importers keep their own rows; `state_machine.cpp` import/onAdded semantics
  are represented within the Rust import architecture as a documented
  adaptation consistent with the accepted sibling definition files.

## 2. Binding architecture decisions

1. **Listener retention divergence (must fix in FL-C5).** `state_machine.rs`
   `filter_map` drops unbuildable listeners and compacts indices
   (`state_machine.rs:377-391`). Port C++ retention: every authored listener
   occupies its authored index (inert if unbuildable), so
   `listenerCount`/`listener(index)` match C++.

2. **Hit-component hierarchy (largest chunk).** Port
   `HitComponent`/`HitDrawable`/`HitExpandable`/`HitTextRun`/`HitLayout`/
   `HitNestedArtboard`/`HitComponentList` as trait + concrete types inside
   `state_machine_instance.rs`, with: three-pass `updateListeners`
   (reset → prepare → process), tri-state `HitResult` incl. `hitOpaque`
   propagation via `canHit`, exit pointer-state release, `sortHitComponents`
   + `m_drawOrderChangeCounter` re-sort on draw-order change, constructor hit
   lookup (`addToHitLookup` dedup/recursion/opacity rules per W2), nested
   artboard and component-list pointer routing (reverse order, occlusion→exit
   conversion), and drag start/end enable/disable walks.
   Group-internal semantics (hover/click-phase/consumed/per-pointer state)
   delegate to the existing Rust pointer machinery behind a
   ListenerGroup-shaped internal seam; faithful internals of that seam are
   FL-D's `listener_group.cpp` row. Delete the per-listener dispatch
   orchestration that FL-C5's files displace; the pointer capture tables that
   implement FL-D-owned semantics stay, documented, until FL-D.

3. **Advance consolidation.** `advance(seconds,newFrame)` and both
   `advanceAndApply` overloads become members of `state_machine_instance.rs`
   taking the artboard explicitly (documented borrow-model adaptation);
   `artboard.rs` keeps thin delegating wrappers. DELETE the clean zero-delta
   fast path (`instance.rs:8960-8967`) — C++ always runs bind/layer/input
   bookkeeping. DELETE `requires_post_update_state_probe` capability caching —
   C++ probes unconditionally each settlement pass. Restore per-layer
   `stateChangedOnAdvance` flags + `stateChangedByIndex` + `currentState`/
   `layerState` accessors; the cached aggregate count may remain as a
   derived value only.

4. **Bind family.** Implement the C++ member surface as the primary
   structure: `setViewModelInstance` (stage without bind; null no-op),
   `setGlobalViewModelInstance` (validation + slot semantics),
   `completeViewModelInstances`, `bind` (complete → artboard → machine),
   `bindViewModelInstance` (null clears machine context + artboard unbind
   only), `bindDataContext`, `inheritDataContext` (no prior-clear hazard
   preserved), `dataContext` setter/getter, `rebind`, `clearDataContext`,
   `relinkDataContext`, `rebuildDataBind`, `unbind`, `internalDataContext`
   (exact ordering incl. listener-cell rebind and scripted-context passes).
   Existing typed Rust context APIs (W4 §C) become delegating adaptations and
   remain public. Preserve C++ null-behavior distinctions exactly.

5. **Events.** Keep `applyEvents` batch semantics (already faithful) but
   verify the 100-iteration boundary and mid-callback queue visibility rows
   from W3 with differentials. Port the trigger zero-suppression guard in
   `reportToStateMachine` (missing in Rust — W4 A3). Port
   `notifyEventListeners` bubbling order to the audio seam.

6. **FP policy.** C++-corresponding members forward NaN/infinity/signed-zero
   exactly as C++ does (advance seconds, pointer coords, delays). Rust-only
   convenience seams may keep finite-validation ONLY if they are distinct
   entry points; any C++-corresponding path must not reject. Each such row
   needs a differential. Exact `seconds == 0.0f` (both signs) forced
   keep-going must be preserved.

7. **Lifecycle.** Port `dispose`/`removeEventListeners` semantics into the
   Rust ownership model as a documented adaptation (explicit detach of nested
   event registrations); destructor teardown ordering represented via Drop
   ordering where observable. Deleted C++ copy constructor vs Rust snapshot
   `Clone`: Clone stays (public API), with the closure documenting its
   non-aliasing rules as an approved adaptation.

8. **Compensation verdicts** (from W4 §B): DELETE zero-delta fast path,
   `requires_post_update_state_probe` scan+flags, cached-only
   changed-state count. KEEP AS DOCUMENTED ADAPTATIONS: index/arena identity
   (no raw pointers), snapshot Clone, event dual cursors, notification queue,
   script lifecycle maps and generation counters (facade mount timing),
   terminal `script_error` channel, per-layer random-weight scratch and
   trigger-layer IDs, typed bindable arrays, host report projection,
   formula-random and probe seams. Every KEEP gets a closure-row citation;
   every DELETE gets a differential proving the C++ behavior it was masking.

## 3. Proof requirements

Every checklist row names its behavioral proof (Rust/C++ differential, live
probe, focused unit test, or source citation where a differential is
impractical) and structural proof (checker rule) per the family procedure.
The 12 required adversarial rows in the existing closure doc stub map onto
these directives 1:1 and must each enumerate their concrete test cases from
the W1–W3 adversarial bullets.
