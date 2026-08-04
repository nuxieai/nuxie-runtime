# VFIX lane 2 report: stateful nested-artboard/VMI propagation

Date: 2026-08-03
Branch: `levi/vfix-nested-vmi`
Pinned upstream: `rive-runtime@4ac7b32798da0482e441ef09304dc3b480ed3ee5`

## Outcome

V20, V21, V34, and V38 are closed. Their three-sample scripted draw and
side-channel streams are exact against the pinned C++ runtime:

| Row | Corpus entry | Result |
|---|---|---|
| V20 | `component_stateful_vm_instance` | exact |
| V21 | `component_stateful_vm_instance_2` | exact |
| V34 | `stateful_nested` | exact |
| V38 | `viewmodel_instance_to_artboard` | exact |

The focused comparison reports 4 exact entries, 12 exact segments, 12
side-channel segments, and no divergences.

## Diagnosis and implementation

The cluster contained two ordering hazards at one ownership seam:

1. Initial nested occurrence construction created a state machine that bound
   and consumed its default DataContext before the mounted child received the
   active stateful VMI.
2. Artboard replacement inserted the new occurrence into the live map before
   replacement VMI selection and child/state-machine context binding finished.

The runtime now follows pinned C++ `NestedArtboard::bindStateful` ordering
(`src/nested_artboard.cpp:156-185`): construct the occurrence without consuming
a default context, bind the active local/global VMI list to the mounted child,
then forward the child's resulting DataContext to its nested state machines.
Host-authored stateful values are synchronized into Rust's detached occurrence
before nested advance.

Artboard swaps now build and bind the detached replacement before publishing it
to the parent occurrence map, matching the single replacement operation in
`src/nested_artboard.cpp:228-350`. Script-projected artboards use the same
child-first ordering, and an explicit local VMI retains its inherited parent
fallback context.

An authored target-to-source bind is also enrolled in its concrete execution
queue when C++ rebind reconciliation marks that direction. A unit regression
covers the pure target-to-source/no-shared-converter case. During corpus
verification, an unconditional root-artboard pre-settle was found to perturb
`data_viz_demo`; removing that root-wide timing change preserved the nested
fix, restored `data_viz_demo`, and kept all four lane rows exact.

## Corpus and register

- Promoted the four corpus entries from `diverges` to `exact` and removed their
  milestone annotations.
- Updated V20, V21, V34, and V38 in `docs/parity-gap-register.md` only after the
  corrected full corpus gate passed.
- Kept the local `docs/v-row-triage.md` copy untracked and out of commits.

## Verification

- Bootstrap: fixture sync, `make fixtures`, and `make cpp-probe` passed.
- `cargo test -p nuxie-runtime` passed.
- `cargo test -p nuxie --features scripting` passed.
- Focused four-row scripted comparison passed: 4 exact entries, 12/12 exact
  segments, 12/12 side-channel segments.
- `make scripted-golden-compare` passed with 362 entries, 330 exact entries,
  1,078 exact segments, 1,078 side-channel segments, 23 registered divergences,
  9 registered not-yet rows, and zero failed entries.
- `make runtime-frame-loop-port-check` passed.
- `make rust-attribution-check` passed.
