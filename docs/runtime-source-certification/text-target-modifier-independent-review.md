# Independent review: `TextTargetModifier`

Candidate `afd986c996fc563a6d6765f5298863f7bffbc7e6` is **rejected** under
`docs/runtime-exact-parity-workflow-correction.md`.

I independently read the complete pinned
`src/text/text_target_modifier.cpp` and
`include/rive/text/text_target_modifier.hpp` at
`4ac7b32798da0482e441ef09304dc3b480ed3ee5`. Their line/byte/hash authority is
confirmed as 30 / 765 /
`e0c6bd9f73031fb5b68a903350c808b57e49f7f88fc31fee9ef54c989a663bd4`
and 19 / 461 /
`4be7851d8c1cd3ecf0b879bcb5154898e1ef4ad7ef7fd1ac64a7ace76c701ef1`.
The executable denominator is exactly two cpp bodies, zero executable primary
header bodies, and the retained `m_Target` plus generated default/copy/setter
context.

## Blocking finding

`TextTargetModifier::textComponent()` has not been translated literally.
Pinned C++ first requires the modifier's **direct parent** to be a
`TextModifierGroup`; only then does it ask that group for its Text. Candidate
`text_target_modifier_text_component` at
`crates/nuxie-runtime/src/text/text_target_modifier.rs:44-50` obtains the
direct parent but never checks its type. It passes any parent into
`modifier_group_text`, whose only predicate is that the supplied component's
parent is-a Text.

Consequently a malformed but constructible hierarchy such as
`Text -> Shape -> TextFollowPathModifier` returns the Text in Rust, while the
pinned body returns null because Shape is not a TextModifierGroup. This is not
only a projection difference: the derived FollowPath callback owner calls this
function, so a live `radial`/`orient`/`start`/`end`/`offset`/`strength` write can
incorrectly dirty that Text after `TextModifier::onAddedDirty` has already
returned `MissingObject`. The existing wrong-parent evidence uses Text itself
as the direct parent, so its grandparent is not Text and it does not exercise
this false-positive route.

Narrow correction: require the direct parent definition to be-a
`TextModifierGroup` inside the production `text_target_modifier_text_component`
owner before calling `modifier_group_text`. Add one real occurrence with a
non-group ContainerComponent directly under Text and the target-derived
modifier beneath it; prove construction continues under the existing
`MissingObject` convention, retained target and `textComponent` remain null,
and a live FollowPath property write does not dirty the Text. Do not change the
consumer topology or broaden the correction to general `TextModifier`
lifecycle work.

## Non-blocking conclusions and focused gates

The other audited body and retained state are sound: Super failure prevents
target resolution, missing/default target remains Ok plus null, live targetId
writes leave the current occurrence frozen, clone copies the live ID into
fresh state and resolves it, and derived FollowPath reads the retained target.
Inheritance-aware Text resolution correctly excludes TextInput. Filtering a
resolved wrong-type object to null is an explicit Rust memory-safety adaptation
for the pinned unchecked cast and does not invent normal valid-file behavior.
The sole pinned consumer remains **1 direct pass / 0 red / 0 adapted / 0
pending**; supporting lifecycle cases are not counted.

- `cargo test -p nuxie-runtime d_st_target --lib -- --nocapture`: 3 passed.
- `cargo test -p nuxie-runtime --lib cxx_text_follow_path -- --nocapture`: 3
  passed.
- `cargo test -p nuxie-runtime --features tools --test cpp_probe
  d_st_target_live_cpp_missing_target_is_ok_like_rust -- --exact --nocapture`:
  1 passed.
- `cargo test -p silver-corpus --test wave_b4
  wave_b4_text_follow_path_shape_length -- --exact --nocapture`: 1 passed.
- `git diff --check afd986c99^ afd986c99`: passed.

The candidate is contained to its nine declared paths. All 17 pre-existing
user-dirty paths remained unstaged and outside this receipt.

## Narrow correction rereview

Correction `a2a4ce1a9ef84d65fd0bb40402a49c62db632476` **closes the sole
finding and is accepted**.

`text_target_modifier_text_component` now resolves the direct parent, proves
its schema definition is-a `TextModifierGroup`, returns null otherwise, and
only then delegates to `modifier_group_text`. The new real
`Text -> Shape -> TextFollowPathModifier` occurrence constructs under the
existing target-derived `MissingObject` continuation, retains no target,
returns no Text component, and drives the live production `start` setter while
the grandparent Text remains at `ComponentDirt::NONE`. It therefore observes
the exact effect that falsified the original candidate without implementing an
alternate algorithm.

The correction changes only the guard, its focused real-owner evidence, and
the author receipt. Body 1's lifecycle/state owners are byte-for-byte
unchanged. The consumer topology remains **1 direct pass / 0 red / 0 adapted /
0 pending**. `cargo test -p nuxie-runtime d_st_target --lib -- --nocapture`
passed 4 tests with no failures, correction-range `git diff --check` passed,
the delta is contained to the declared three paths, and all 17 pre-existing
user-dirty paths remain unstaged.
