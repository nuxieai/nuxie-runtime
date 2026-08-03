# INTEV scripted-interpolator evidence report

## Status

The lane adds valid evidence and support for the ordinary scalar/root/stateless
clone path, but `src/scripted/scripted_interpolator.cpp` remains `pending`.
Closeout review found that the full generic DataBind/converter/DataContext
surface is not green, so the row was deliberately not promoted.

## Startup and pin evidence

- Branch: `levi/pend9-interp-evidence`.
- Lane base: `121230e121ddbac58b299a8183cdd0c295d61253`, identical to
  startup `origin/main` (`121230e1 Merge pull request #218 ...`).
- Pinned C++ checkout: `4ac7b32798da0482e441ef09304dc3b480ed3ee5`.
- `rsync -a /Users/levi/dev/nuxie-runtime/fixtures/ fixtures/ && make fixtures`
  completed successfully before implementation.
- No file was written under `/tmp`.

## Oracle localization

The pinned checkout contains no direct C++ unit-test row with a literal
`ScriptedInterpolator` reference: `rg -n ScriptedInterpolator
/Users/levi/dev/oss/rive-runtime/tests -g '*.cpp'` returns no matches. The
commented serialized-rendering silver row names only the fixture. No direct
test-correspondence case was invented or promoted.

The applicable pinned source oracle is:

- `ScriptedObject::cloneProperties` clones each CustomProperty, its DataBind,
  and its converter, retargets the bind, and adds it to the supplied container
  (`src/scripted/scripted_object.cpp:558-588`).
- `LinearAnimationInstance::statefulInterpolator` creates one clone per
  animation-instance/keyframe, assigns the live DataContext, tracks cloned
  artboard binds, and caches the clone (`src/animation/linear_animation_instance.cpp:109-172`).
- `LinearAnimationInstance::~LinearAnimationInstance` removes and deletes
  those binds before destroying their cloned targets
  (`src/animation/linear_animation_instance.cpp:38-70`).
- `DataBindContainer::addDataBind` binds and synchronously updates a newly
  added bind when the Artboard already has a DataContext
  (`src/data_bind/data_bind_container.cpp:73-101`).

The unchanged Rust lifecycle regression was first RED with `Some(20.0)` rather
than the source-and-RangeMapper result `Some(30.0)`. A synthetic C++
golden-runner file could import the graph, but the repository runner reported
that its generated ScriptAsset had no C++ generator (the existing generated
interpolator fixture has the same limitation), so that run is not cited as
behavioral evidence.

## Landed narrow lifecycle

- `RuntimeScriptedInterpolatorBindingDefinition` imports the authored scalar
  ScriptInput/DataBind/converter recipe through the established ScriptedObject
  binding owner.
- Each lazy keyframe factory instantiates fresh ordinary converter state,
  hydrates the bound value before `init`, and retains the occurrence for later
  same-root source refreshes.
- `DataBoundScriptedInterpolatorInstance` declares bindings before its Lua
  target so Rust drop order runs the shared unbind path first.
- VM-specific interpolator calls remain delegated directly, preserving the
  existing numeric coercion and fallback behavior.

The new regression proves the narrow path across initial converted hydration,
a live same-handle ViewModel change, animation teardown, and a replacement
animation clone. Existing tests retain coverage for `transform`,
`transformValue`, fallback/diagnostics, definition-level apply, and independent
per-keyframe tables.

## Review blockers to faithful promotion

The two-axis closeout review found these actionable gaps:

1. The interpolator wrapper does not execute the existing scripted-converter
   target/bind/rehydrate plan, so a cloned `ScriptedDataConverter` table is not
   instantiated or attached.
2. It does not call `advance_stateful_converters(elapsed_seconds)`, so
   time-based `DataConverterInterpolator` clone state remains frozen.
3. It binds and retains one root `RuntimeOwnedViewModelHandle`, not the full
   parent-scoped `RuntimeOwnedDataContext`; nested/relative paths and later
   Artboard DataContext replacement therefore do not match C++ container
   membership/rebinding.
4. The new regression uses one scalar Number input and a RangeMapper. It does
   not establish cloned Artboard, Trigger, or ViewModel ScriptInput projection,
   and its replacement-clone assertion does not directly count removed source
   dependents.

These gaps belong to the same generic `cloneProperties` surface named by the
pending row. The RangeMapper evidence is useful and green, but it is not a
basis for whole-file promotion.

## Green lane evidence

- `cargo check -p nuxie --features scripting` — PASS.
- `cargo test -p nuxie --features scripting --lib
  scripted_interpolator_tests::` — PASS: 5 passed, 1 fixture generator ignored.
- `cargo test -p nuxie-runtime --lib scripted_interpolator::tests::` — PASS:
  3 passed.
- `cargo test -p nuxie-runtime --lib
  occurrence_drop_unbinds_outer_and_scripted_converter_custom_sources` — PASS.
- `cargo test -p nuxie-runtime --lib
  fresh_clone_does_not_retain_the_live_converter_table` — PASS.
- `git diff --check` — PASS.

Only the requested preparation gates were run. The orchestrator retains the
full runtime, golden, correspondence, attribution, and scorecard battery.

## Correspondence, residue, and ratchets

- `file-correspondence-manifest.toml`: B6-0323 stays `pending` with the new
  green evidence and the four explicit blockers above.
- `port-manifest.toml`: the stale consolidated mapping now names the actual
  runtime owner, VM adapter, and facade seam, while its note records the
  scripted/stateful converter and DataContext residue.
- `docs/parity-scorecard.md`: remains faithful 438, pending 14.
- `test-correspondence-manifest.toml`: unchanged because the pin has no direct
  ScriptedInterpolator unit-test row.
- `docs/runtime-frame-loop-ownership.toml`: unchanged because the frozen
  frame-loop ledger has no `scripted_interpolator.cpp` source/member row.
- Attribution comments live on the new definition/occurrence wrapper and
  facade drop-order owner.
- Correspondence scatter remains 154/155. No scatter exception or ratchet
  value changed.
- No tolerance, corpus threshold, or pending floor was relaxed.

## Commits

1. `0e863013` — `Port cloned scripted interpolator bindings`.
2. The evidence/correspondence commit records the honest pending disposition
   and this report.
