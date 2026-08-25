# Focus ownership source certification

> **Independent adversarial review: rejected.** Commit `d4c083f22` does not
> preserve the pinned `FocusData::focused` occurrence sequence, and this
> receipt does not cover the complete v2 focus authority described below.

## Scope and evidence

This is a fresh literal audit against pinned upstream commit
`4ac7b32798da0482e441ef09304dc3b480ed3ee5`. It does not inherit an RB-2 or
earlier audit verdict. The four complete translation units and their complete
mapped Rust owners were read side by side:

| Manifest row | Pinned C++ translation unit | Exact Rust owner |
| --- | --- | --- |
| B6-0209 | `src/focus_data.cpp` | `crates/nuxie-runtime/src/focus_data.rs` |
| B6-0238 | `src/input/focus_manager.cpp` | `crates/nuxie-runtime/src/input/focus_manager.rs` |
| B6-0240 | `src/input/focusable.cpp` | `crates/nuxie-runtime/src/input/focusable.rs` |
| B6-0027 | `src/animation/focus_listener_group.cpp` | `crates/nuxie-runtime/src/state_machine/focus_listener_group.rs` |

The corresponding upstream headers were also read completely:
`include/rive/focus_data.hpp`, `include/rive/input/focus_manager.hpp`,
`include/rive/input/focus_node.hpp`, `include/rive/input/focusable.hpp`, and
`include/rive/animation/focus_listener_group.hpp`. `src/input/focus_node.cpp`
and Rust `input/focus_node.rs` were inspected as ownership dependencies, but
their independent B6-0239 certification is outside this shard.

That four-translation-unit boundary is too narrow for the certification claim.
The v2 denominator contains 20 directly coupled focus owners and 156 executable
units across `FocusData`, `FocusManager`, `FocusNode`, `Focusable`,
`FocusListener`, the focus listener group, the three focus actions, and the
transition focus condition. They are:

- 89 units in nine source owners: the 30/43/6/2 units in
  `focus_data.cpp`, `focus_manager.cpp`, `focus_node.cpp`, and
  `focusable.cpp`; the four listener-group units; and one unit in each of the
  three focus-action sources and `transition_focus_condition.cpp`;
- 67 units in eleven handwritten headers: the FocusData, FocusManager,
  FocusNode, Focusable, FocusListener, listener-group, focus-action, and
  transition-condition headers.

The inventory below dispositions only the 79 out-of-line units in four source
owners. Saying that five headers were read does not disposition their 61
executable units, and the other 16 directly coupled source/header units are not
inventoried. Therefore 77 of the 156 v2 units have no symbol-level disposition
or accepted evidence in this receipt. In particular, excluding FocusNode while
claiming exact focus-chain callbacks removes the inline `focused()`/`blurred()`
delegation that makes the disputed behavior observable.

## Literal symbol inventory

Every out-of-line symbol in the four pinned translation units is accounted for
below. Anonymous-namespace helpers are included because their ordering and
selection rules are observable through focus traversal.

### `src/focus_data.cpp`

| Pinned symbol | Side effects / observable result | Rust ownership |
| --- | --- | --- |
| `findSiblingSemanticData` | Finds the first `SemanticData` sibling under the same parent. | Semantic sibling lookup is owned by the Artboard semantic tree seam, not `focus_data.rs`. |
| `FocusData::~FocusData` | Detaches its node from its parent and manager, clears retained node ownership. | Retained occurrence removal and domain rebuild detach stable node ids without pointer destruction. |
| `focusNode` | Lazily creates a `FocusNode`, binds the focusable, locates the closest parent node, and registers it with the manager. | `RuntimeFocusTree` creates stable arena nodes and records owner/local occurrence mappings during exact topology construction. |
| `addFocusListener` / `removeFocusListener` | Mutate the focus-listener collection. | Immutable listener occurrences are collected during construction and dispatched from the focus transition event queue. |
| `addKeyboardListener` / `removeKeyboardListener` | Mutate the keyboard-listener collection. | Listener occurrence maps are built and removed with retained owner topology. |
| `addTextInputListener` / `removeTextInputListener` | Mutate the text-input-listener collection. | Listener occurrence maps are built and removed with retained owner topology. |
| `addGamepadListener` / `removeGamepadListener` | Mutate the gamepad-listener collection. | Listener occurrence maps are built and removed with retained owner topology. |
| `focus` | Intentional no-op. | No independent Rust operation. |
| `keyInput` | Dispatches in listener order and returns at the first handled result. | Retained listener dispatch preserves occurrence order and the first-handled short circuit. |
| `textInput` | Dispatches in listener order and returns at the first handled result. | Retained listener dispatch preserves occurrence order and the first-handled short circuit. |
| `gamepadDispatch` | Dispatches in listener order, threads scripted-drawable output, and returns at the first handled result. | Retained listener dispatch preserves occurrence order and the first-handled short circuit; scripted drawable ownership remains at the host boundary. |
| `scrollIntoView` | Walks ancestor scroll constraints and asks each eligible constraint to reveal the focused bounds. | `ArtboardInstance::scroll_focus_target_into_view` owns the same mutation because it has the mutable Artboard occurrence. This audit restored its missing invocation for every ordinary focus transition. |
| `scrollConstraintToShowBounds` | Computes local target bounds, padding, clamped offsets, and mutates the scroll constraint. | `ArtboardInstance::scroll_focus_target_into_view` and the scroll-constraint implementation own the translated geometry/mutation. |
| `focused` | In order: scrolls into view, notifies focus listeners, focuses sibling semantic data, then focuses parent text input. | The focus domain now retains the exact focused occurrence; the first Artboard-bearing advance performs the scroll before processing queued listener events. Semantic and text-input synchronization are cross-owner seams and remain separately called out below. |
| `blurred` | Notifies blur listeners, clears sibling semantic focus, then clears parent text-input focus. | Blur events are queued in focus-transition order. Semantic and text-input synchronization are cross-owner seams and remain separately called out below. |
| `focusFlagsChanged` | Updates its retained node flags and invalidates focus caches. | Runtime topology refresh updates node flags and invalidates retained manager caches. |
| `edgeBehaviorValueChanged` | Updates node edge behavior. | Runtime topology/property refresh updates retained edge behavior. |
| `findParentFocusData` | Walks parents, crossing nested-artboard boundaries, to locate inherited focus ownership. | Construction resolves occurrence ancestry across mounted owner identities. |
| `findClosestFocusNode` | Walks parents; lazily materializes focus data nodes and handles nested artboard inheritance. | Exact topology construction records focus nodes and mounted roots by stable owner/local identity. |
| `componentAllowsFocusTraversal` | Recursively checks component and ancestor visibility/opacity. | Focusable eligibility is retained from effective component visibility/opacity state. |
| `isEligibleForFocusTraversal` | Combines hidden check, flag eligibility, and ancestor traversal eligibility. | Manager eligibility combines retained hidden state and focus flags. |
| `worldPosition` | Resolves a world point for nested artboards, text inputs, or world-transform components. | Retained focusable bounds/positions are produced from the corresponding Artboard occurrence. |
| `nameChanged` | Synchronizes the focus node name. | Topology/property refresh synchronizes retained node names. |
| `buildDependencies` | Adds dependency edges to the parent focus data, layout/text/world-transform owner, and scroll constraints. | Construction and property refresh happen at the Rust lifecycle sites that own topology and geometry; there is no speculative per-frame source rescan. |
| `update` | On world-transform dirt, recomputes world bounds. | Artboard-driven refresh updates retained bounds at the corresponding dirty lifecycle boundary. |
| `updateWorldBounds` | Recomputes bounds and invalidates manager bounds caches. | Bounds refresh replaces retained bounds and invalidates directional traversal caches. |

### `src/input/focus_manager.cpp`

| Pinned symbol group | Side effects / observable result | Rust ownership |
| --- | --- | --- |
| `focusNodeEligibleForFocus`, `focusNodeEligibleForTraversal` | Apply flag and focusability predicates. | `FocusManager` node predicates over stable ids. |
| `getRootPosition`, `getRootBounds` | Normalize positions/bounds across nested Artboard ownership. | `RuntimeFocusable` retains normalized geometry for the owning occurrence. |
| `isLeaf`, `focusNodeTraversable`, `getFirstLeaf`, `getLastLeaf`, `firstEligibleLeafFrom` | Define the exact sequential leaf traversal and edge behavior. | `input/focus_manager.rs` mirrors leaf selection, traversal eligibility, edge behavior, and root-edge focus clearing. |
| `collectAllTraversableNodes`, `subtreeHasFocusableContent`, `hasEligibleTraversableChildInFocusTree`, `getTraversableNodes` | Build and filter directional candidate sets while respecting focus roots. | Rust candidate collection and focus-root filtering operate in retained child order. |
| `calculateOverlap`, `ScoreBreakdown`, `scoreCandidateBoundsDetailed`, `scoreCandidateBounds`, `scoreCandidatePoint` | Score directional candidates, including overlap and axis penalties. | Rust directional scoring preserves the pinned comparisons and tie order. |
| `dropFocusIfFocusTargetHidden` | Clears focus when the target becomes ineligible. | Rust focus refresh drops an ineligible primary target. |
| `primaryFocusArtboard` | Resolves the Artboard containing the primary focus. | Owner identity on each retained focusable resolves the mounted Artboard occurrence. |
| `~FocusManager`, `removeManager` | Clear manager back-pointers throughout roots. | Arena ownership removes pointer back-reference requirements. |
| `setFocus`, `clearFocus`, `notifyFocusChange` | Mutate the primary focus, blur the old chain, focus the new chain, and notify in exact chain order. | Stable node ids replace `rcp` pointers; Rust computes old/new ancestor chains, updates flags, and queues ordered focus/blur occurrences. |
| `hasFocus`, `hasPrimaryFocus` | Query inherited or primary focus. | Stable-id focus queries. |
| `addChild` overloads, `removeChild`, `detachChild`, `eraseRoot` | Reparent nodes, migrate managers, maintain root order, preserve/clear focus as specified, and invalidate caches. | Safe arena mutation mirrors parent/root ordering and cache invalidation; cycle and unknown-id attempts fail closed. |
| `focusNext`, `focusPrevious`, `findNextFocusable` | Traverse sequentially, including clearing focus at non-looping root edges. | Rust mirrors the pinned root-edge behavior rather than retaining the last focus. |
| `hasFocusableContent` | Memoizes whether any eligible focusable content exists. | Rust cache and invalidation mirror the query. |
| `findNodeInDirection`, `focusLeft`, `focusRight`, `focusUp`, `focusDown` | Select and focus the best directional candidate. | Rust candidate collection/scoring and final focus transition mirror the pinned direction path. |
| `keyInput`, `textInput`, `gamepadDispatch` | Bubble input from primary focus through ancestors and aggregate handled state. | Rust bubbles retained occurrences from primary focus through parent ids in order. |

The two `addChild` overloads are represented by one Rust API because Rust does
not need C++ overloads to distinguish a direct child from a nested manager root.
The resulting topology and ordering, not the overload surface, are the parity
contract.

### `src/input/focusable.cpp`

| Pinned symbol | Side effects / observable result | Rust ownership |
| --- | --- | --- |
| `Focusable::gamepadDispatch` | Default implementation returns `false`. | The default Rust focusable path is unhandled. |
| `Focusable::from` | Returns `TextInput` or `NestedArtboard` focusables; null/other objects return null. | Construction creates retained runtime focusables only for translated focus-bearing component kinds; non-focusable objects do not receive a target mapping. |

### `src/animation/focus_listener_group.cpp`

| Pinned symbol | Side effects / observable result | Rust ownership |
| --- | --- | --- |
| `FocusListenerGroup::FocusListenerGroup` | Retains listener/focus-data pointers, snapshots focus/blur flags, registers with `FocusData`. | `RuntimeFocusListenerGroup::new` snapshots the same two eligibility flags; the owning state-machine construction records the occurrence. |
| `~FocusListenerGroup` | Unregisters from `FocusData`. | Removing the owning state-machine occurrence removes the immutable retained registration. |
| `onFocused` | Queues the listener only when the snapshotted focus flag is set. | `queue_focus` gates on the snapshotted focus flag. |
| `onBlurred` | Queues the listener only when the snapshotted blur flag is set. | `queue_blur` gates on the snapshotted blur flag. |

## Demonstrated mistranslation and attempted correction

Pinned `FocusData::focused` calls `scrollIntoView()` before any focus-listener
notification. Rust already contained the translated scroll algorithm in
`ArtboardInstance::scroll_focus_target_into_view`, but called it only from the
semantic-focus request path. Ordinary direct, sequential, and directional
focus transitions therefore omitted a pinned side effect.

The attempted correction is an ownership adaptation rather than a new scroll
algorithm:

1. A successful `RuntimeFocusTree` focus transition retains one
   `(owner_identity, target_local)` pair.
2. The first `advance_on_artboard_with_script_host` consumes that occurrence
   and calls the existing translated scroll implementation.
3. When that handoff survives until the next advance, it occurs before the
   queued focus-listener batch in that advance.
4. The semantic-focus path no longer separately schedules the same scroll, so
   one successful focus transition produces one scroll request.

The focused regression evidence verifies only a flat, single-node transition:
one requested pair is stored and `take_pending_focus_scroll` consumes it once.
It does not invoke `ArtboardInstance::scroll_focus_target_into_view`, observe a
scroll mutation, observe callback ordering, or cover nested scopes, multiple
transitions, semantic focus, cloning, manager replacement, or cleanup.

## Independent adversarial findings

### The retained pair is not the pinned focused-occurrence sequence

Pinned `FocusManager::setFocus` first descends an eligible requested scope to
its first eligible leaf. `notifyFocusChange` then walks from that selected leaf
through every newly focused ancestor, calling `FocusNode::focused()` for each
node. Each backed node delegates to `FocusData::focused()`, which performs its
own `scrollIntoView()` before its listener, semantic, and text-input effects.

`set_focus_target_for_owner` instead stores the *requested* owner/target after
`FocusManager::set_focus` returns. Thus a direct request for a scope stores the
scope even when the manager selected a descendant leaf. More generally, direct,
sequential, directional, and semantic routes all store at most one pair while
the pinned transition can produce a leaf-to-ancestor sequence of multiple
backed focus occurrences. The pair is therefore neither necessarily the
selected occurrence nor a complete representation of the pinned callbacks.

### The one-slot handoff drops valid transitions

`pending_focus_scroll` is an `Option`, so every successful transition before
the next Artboard-bearing advance overwrites the previous transition. Pinned
C++ scrolls synchronously for every transition. For example, focusing A and
then B before the next advance scrolls A and then B upstream, but Rust scrolls
only B. Clearing focus does not clear the retained pair, so focus A followed by
clear can later scroll a stale, no-longer-focused A after intervening geometry
changes. Cleanup of a mounted owner likewise removes its topology without
invalidating a pair that names that owner.

### Deferred execution does not preserve the complete pinned order

For one surviving pair, the next advance consumes the pair before processing
that advance's queued focus-listener batch. That limited order is real, but it
does not establish pinned synchronous ordering. Code between the focus change
and the next advance observes the pre-scroll state. A focus action executed
during an advance creates the pair only after the advance-start consumption,
so later actions in the same listener occurrence run before the scroll even
though pinned `FocusData::focused()` scrolls during the focus action.

The semantic route is a concrete reversal: Rust `request_semantic_focus`
updates semantic focused state immediately after storing the pair, while the
scroll waits for a later advance. Pinned `FocusData::focused()` scrolls first,
then notifies focus listeners, then synchronizes the sibling SemanticData.
Removing the semantic tree's prior scroll request does not make this new order
equivalent.

### Clone and manager-lifecycle evidence does not close the adaptation

`RuntimeFocusTree::clone` deep-clones the whole domain, including the deferred
pair. A snapshot made after focus but before advance can therefore replay the
same delayed scroll in both independent occurrences; pinned C++ had already
completed the synchronous scroll before any later snapshot boundary. Conversely,
`replace_with_owner_occurrence_from` builds a default domain and resets every
node's `has_focus`, pending callbacks, and pending scroll. Those choices may be
valid Rust lifecycle adaptations, but this receipt supplies no occurrence test
against the complete external-manager/nested-artboard call chain.

The retained listener-group representation has useful focused tests for flag
gating, duplicate order, and category construction. The receipt still provides
no direct registration/unregistration evidence for focus-data destruction,
nested swaps, manager cleanup, or replacement. Those claims cannot be promoted
from plausible ownership design to accepted source parity by the flat pending
pair test.

## Adaptations and remaining certification boundaries

- Raw and reference-counted C++ pointers are deliberately represented by
  stable `FocusNodeId`s plus retained owner/local occurrence keys. This is an
  ownership adaptation; it does not authorize changed traversal behavior.
- Rust rejects cycles and unknown node ids instead of permitting invalid pointer
  topology. Valid pinned topology is unchanged.
- Listener registration is retained as immutable constructed occurrences rather
  than mutable callback-pointer vectors. Eligibility is still snapshotted at
  construction and dispatch order remains observable.
- The `WITH_RIVE_TOOLS` editor callbacks and external-parent focus-node hooks are
  outside the pure runtime ceiling. Their absence is not counted as runtime
  behavior parity.
- `FocusData::focused`/`blurred` also synchronize sibling `SemanticData` and a
  parent `TextInput`. Rust has semantic and text-input synchronization at
  Artboard-bearing ownership boundaries, but ordinary traversal through those
  cross-owner seams is **not certified by this four-file shard**. It requires a
  literal joint audit of SemanticData, TextInput, and their Artboard dispatch
  owners; this document does not silently declare it equivalent.
- The source-correspondence manifest cites `P3A2-report.md` in these rows, but no
  such evidence file is present in this checkout. That citation is stale and
  was not used as proof.

## Verdict

**Rejected.** The four out-of-line source inventories are useful partial audit
notes, and the retained manager has substantial focused evidence for traversal,
topology, input bubbling, and listener flag gating. They do not establish a
complete v2 certification, and commit `d4c083f22` does not correct ordinary
focus scrolling with exact occurrence parity. Acceptance requires a lossless
leaf-to-ancestor occurrence queue (including the manager's actual selected
leaf), Artboard-level evidence for every direct/sequential/directional/semantic
route and synchronous ordering seam, lifecycle/clone/nested-manager evidence,
and symbol-level dispositions for all 156 directly coupled v2 units. Semantic
and TextInput coupling remains an additional joint-audit boundary, not the only
remaining boundary.
