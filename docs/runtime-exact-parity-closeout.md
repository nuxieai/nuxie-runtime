# Runtime exact-parity closeout

This closes the runtime parity campaign against pinned Rive runtime commit
`4ac7b32798da0482e441ef09304dc3b480ed3ee5`. The campaign ported the complete
upstream unit-test denominator, established source-owner correspondence, audited
each pair, and corrected demonstrated hand-porting errors without using failing
tests as permission to invent new behavior.

## Frozen denominator

- Upstream tests: 157 files and 1,404 active Catch2 cases. Rust evidence covers
  149 files directly, 6 differentially, and records 2 as not applicable. No row
  is pending or partial.
- Behavioral source owners: 456 applicable pinned C++ owners. The correspondence
  ledger maps 404 directly, 43 to documented shared owners, and 9 to explicit
  exceptions. No owner is pending.
- Golden corpus: 364 assets. 359 are exact and 5 retain reviewed divergence
  dispositions. No asset is unsupported or not-yet-reviewed.
- Structural audit: all 456 owners have B6 classifications: 22 isomorphic, 216
  adapted, 154 divergent, 27 tracked-gap, and 37 not applicable. `UNKNOWN` is
  empty.

The scripting-enabled execution verifies 358 exact outputs and 6 reviewed
signatures: the five declared divergence rows plus
`editor_scripted_vector_v7`, whose scripted-only signature records the pinned
C++ runner rejecting unsigned editor bytecode that the Rust scripting backend
accepts. That conditional signature is frozen in `corpus.toml`; it is not an
unreviewed failure.

The final ordinary and scripted golden gates use the pinned C++ runners and the
final closeout source. Their machine-readable reports are generated at
`target/runtime-differentials/golden-ordinary.json` and
`target/runtime-differentials/golden-scripted.json`.

## Approved adaptation ceiling

Parity does not mean reproducing implementation choices that this project has
already replaced deliberately. Taffy remains the layout engine; the Rust-native
audio and scripting backends remain; Rust slices and checked arithmetic replace
C++ container helpers; and safe Rust ownership is not bent around undefined
behavior or allocator bookkeeping. These adaptations must preserve the
observable behavior under test and remain named as adaptations rather than being
silently relabeled as literal source equivalence.

## Reviewed remaining golden differences

The five retained differences are explicit corpus dispositions, not unknown or
unexamined failures:

- `echo_show_demo` (V24): gradient construction differs from the pinned runner.
- `group_effect` (V25): retained path-effect command chains differ.
- `path_effect_with_feathers` (V30): the affected retained path differs.
- `rewards_demo` (V31): the gradient x coordinate differs while the accepted
  rendering comparison remains within its recorded tolerance.
- `superbowl` (V36): scripted retained path construction differs.

Their exact diagnostic signatures remain frozen in `corpus.toml`, which is the
authoritative disposition ledger.

## Final correction found by the gates

The full ordinary gate exposed `artboard_width_test` and `script_verbs_resize`.
A direct C++ probe ruled out the first hypothesis: `LayoutComponent` itself
already matched. The hand port had instead omitted the final mounted-artboard
origin compensation in `NestedArtboardLayout::update` for the identity-host
path, while embedding it only in part of the affine path. The correction now
applies the pinned `-artboard->origin()` compensation exactly once after cached
or uncached transform construction. `artboard_width_test`,
`script_verbs_resize`, and `db_health_tracker` are exact in both lanes.

The final scripted gate then exposed retained-path churn in
`artboard_list_overrides` and `component_list_child_origin`. The port had treated
the Artboard root as an ordinary LayoutComponent during hosted size settlement,
manufacturing Text shape dirt whenever a component-list row changed size.
Pinned `Artboard::propagateSize` is an override: it dirties the Artboard path and
host transform but deliberately does not call `propagateSizeToChildren`. Rust
now preserves that owner boundary, while ordinary LayoutComponents continue to
control their direct sizeable children. Both fixtures are exact, and a focused
unit regression freezes the override distinction.

## Closeout gates

The following local gates pass:

- complete test correspondence and frame-loop ledger;
- complete source correspondence;
- Rust source attribution;
- pure-runtime boundary enforcement;
- B6 source-owner audit with no unknown classifications;
- ordinary and scripted golden comparison across the complete corpus.

The scorecard also exposes legacy proof metadata honestly: most owner rows have
stale per-row commit freshness and remain marked behaviorally unverified even
though the denominator-level source, structural, and golden gates above are
current. Mechanically rewriting hundreds of evidence commits would not add new
runtime evidence, so that paperwork is published as debt rather than treated as
a reason to delay the completed behavior campaign.

## Deferred work that is not a runtime-parity blocker

- The behavior-inventory checker now sees renderer and host-C-API sources added
  by the completed Vulkan, WebGPU, WebGL2, and Metal work. Those owners need a
  renderer/inventory integration pass; they are outside this pinned pure-runtime
  denominator and are not hidden by regenerating the approval snapshot here.
- Broad Windows/Linux browser and Linux GPU-vendor matrices, hosted CI plumbing,
  packaging, and editor integration remain separately tracked work. The closeout
  was validated on the available macOS and Android-focused product path as
  previously scoped.
- `mid_animation_data_bound_gap_resolves_transferred_nested_layout_same_frame`
  currently performs 35 Taffy solves against a historical budget of 9. The same
  failure reproduces on untouched `origin/main`; it is a pre-existing performance
  budget issue, not a behavior regression from this campaign.

The next runtime improvements may now be evaluated as intentional changes from
this pinned baseline rather than being mixed with incomplete translation work.
