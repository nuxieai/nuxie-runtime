# Independent review: `TextModifier`

Candidate `ae29c645ce056403e74824911e0d9472c6840722` is **rejected** under
`docs/runtime-exact-parity-workflow-correction.md`.

I independently read the complete pinned `src/text/text_modifier.cpp` and
`include/rive/text/text_modifier.hpp` at
`4ac7b32798da0482e441ef09304dc3b480ed3ee5`. Their line/byte/hash authority is
confirmed as 21 / 529 /
`52659ea3db48f7acfc4a31458a620f57c3f29e6783cfe691d5ac9527e103eda5`
and 13 / 283 /
`ba29e20cfcaea03e58138b0ed25956942d87ac215d0fd9978d5df929483ddcbe`.
The executable denominator is exactly one cpp body and zero executable
primary-header bodies.

## Blocking finding

The new occurrence vectors are accurate retained observations, but they do not
yet own production behavior. `RuntimeTextModifierGroupState` is cleared and
populated after the Component Super link in authored order, and clone starts it
fresh. However, repository-wide use search finds its three read methods only in
the new focused tests. Actual Text shaping/rendering continues through
`StaticTextModifierGroup::from_graph`, which independently scans immutable
`ArtboardGraph` child lists and reconstructs separate all/shape/follow vectors.
The candidate therefore adds a shadow copy of pinned `m_modifiers`,
`m_shapeModifiers`, and `m_followPathModifiers`; the graph projection remains
the causal production owner.

This is observably incorrect across the very clone boundary governed by
`onAddedDirty`. With two valid groups, live-writing a modifier's generated
`Component.parentId` from group A to group B leaves the source occurrence's
resolved parent and registration in A. A clone copies that live ID and reruns
Component plus TextModifier construction, so pinned C++ registers the clone in
B. Candidate occurrence vectors likewise rebuild into B, but the clone's real
`StaticTextSlice`/render path still assigns the modifier to authored group A
because it rereads frozen graph children. The current clone test does not
change `parentId`, so both the shadow state and graph projection happen to
agree and the discrepancy is hidden.

Narrow correction: make the production Text modifier traversal consume the
occurrence-owned registration vectors (or an equivalent single retained owner
derived from them), rather than independently rebuilding modifier membership
from graph children. Add one real two-group occurrence that writes `parentId`,
proves the source remains registered/consumed in A, clones it, and proves both
the clone's retained state and actual all/shape/follow production consumers use
B. Remove or demote the duplicate graph-membership path; do not satisfy this
with another debug projection. Preserve the malformed-subclass behavior,
consumer accounting, and adjacent TextModifierGroup discrepancy.

## Non-blocking conclusions and focused gates

Within the retained registration owner, Super-first linking, direct is-a-group
success, MissingObject continuation, exact-once authored appends, subtype
vector order, malformed Variation/Follow omission, and cold/post-materialized
clone reconstruction are correct. The concrete target-derived path performs
no target work after base failure, and late `StaticTextSlice` rejection is
removed. Abstract base definitions remain honestly non-instantiable context.
The adjacent TextModifierGroup wrong-parent hard error is explicitly reopened
and is not accepted by this review. The sole consumer topology remains **1
direct pass / 0 red / 0 adapted / 0 pending**.

- Valid authored-order/clone owner test: 1 passed.
- Malformed concrete-subclass omission test: 1 passed.
- `cargo test -p nuxie-runtime d_st_target --lib -- --nocapture`: 4 passed.
- `cargo test -p nuxie-runtime --lib cxx_text_follow_path -- --nocapture`: 3
  passed.
- Exact Wave B4 sole consumer: 1 passed.
- `git diff --check ae29c645c^ ae29c645c`: passed.

The candidate is contained to its seven declared paths. All 17 pre-existing
user-dirty paths remained unstaged and outside this receipt.

## Narrow causal-correction rereview

Correction `73bd7313b38adefd893f8cf2471ad0a4f90ff5cc` remains **rejected**
for one retained-owner clone defect.

The correction successfully routes live constructor callsites through
`StaticTextSlice::from_instance`; exhaustive callsite search leaves
`from_graph` only in the explicit no-occurrence support query and tests. A cold
clone therefore builds all/shape/follow descriptors from its occurrence
registration. However, `RuntimeTextDrawOwner::clone` at
`crates/nuxie-runtime/src/draw.rs:10508-10520` copies the source owner's
already-materialized `topology` Arc into the clone. `topology_or_build` then
returns that Arc without consulting the clone occurrence.

This preserves the exact rejected A/B divergence on the required
post-materialized path. The candidate test materializes source topology at
`text.rs:8412-8414`, clones it at `:8415`, but validates the clone at `:8418`
through the local `production_topology` closure. That closure directly invokes
`StaticTextSlice::from_instance` (`:8360-8362`) and bypasses the clone's real
retained draw owner. Actual retained draw, measure, onDirty, and topology
callers reuse source-A membership instead of rebuilding clone-B membership.

Narrow correction: `RuntimeTextDrawOwner` clone construction must start with
cold topology (or explicitly rebuild it from the clone occurrence only after
its relations are reconstructed), matching fresh C++ custom Text collections.
Replace the bypassing post-materialized assertion with an inspection of
`materialized_clone.retained_static_text_topology(...)`, and drive at least one
real retained consumer such as draw-frame rebuild or WorldTransform onDirty to
prove it observes group B. Preserve the now-correct cold constructor routing,
malformed behavior, topology accounting, and adjacent group discrepancy.

The focused lifecycle test still passes 1 test because of the bypass above.
Correction-range `git diff --check` passed; the commit is contained to its
seven declared paths. The pre-existing unstaged formatting-only hunk in
`draw.rs` remains outside the commit, and all 17 user-dirty paths are preserved.

## Final retained-owner correction rereview

Correction `47dd86b9dd743a9e7608ff18cf01abea6e2171b7` **closes the residual
finding and is accepted**.

`RuntimeTextDrawOwner::clone` now returns the same wholly fresh default owner
used for every other custom Text clone field; it no longer copies the source
topology Arc. Exhaustive owner search finds no alternate Text topology-copy
site. The earlier occurrence-driven constructor routing remains intact, with
`from_graph` confined to the no-occurrence support query and tests.

The revised A-to-B evidence no longer calls a direct slice constructor for the
post-materialized assertion. It retains source A through the real draw owner,
verifies a cold clone retains B, then clones the materialized source, drives
`WORLD_TRANSFORM` through `add_dirt` and real onDirty reentrancy, observes the
ordered group-A non-follow/group-B follow actions twice at the accumulated
World then World-or-Path masks, and reads all/shape/follow topology B from
`materialized_clone.retained_static_text_topology`. This closes both the stale
Arc and bypassing-evidence defects.

The consumer topology remains **1 direct pass / 0 red / 0 adapted / 0
pending**. The focused lifecycle test passed 1 test and exact Wave B4 passed 1
test. Correction-range `git diff --check` passed; the delta is exactly the
declared three paths. The only committed `draw.rs` hunk is the cold clone owner
change, while the pre-existing formatting-only hunk remains unstaged. All 17
user-dirty paths are preserved.
