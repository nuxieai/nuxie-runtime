# Script-input source certification

Pinned upstream: `4ac7b32798da0482e441ef09304dc3b480ed3ee5`

Implementing auditor: root campaign lane

Adversarial review: **REJECTED BY THE FIRST FRESH REVIEW AFTER `4c8daea99`**

This receipt deliberately includes executable methods defined inline in the
handwritten `include/rive/script_input_*.hpp` headers. Those methods exposed
the initial `.cpp`-only denominator as incomplete; they are part of the source
contract even though the first generated census did not count them.

## Common scalar ownership

The Boolean, Color, Number, String, and Trigger implementations share the
same three lifecycle translations:

- binary import status and the ordered scripted-object input catalogue replace
  `ScriptedObjectImporter::addInput` plus the conditional component `Super`
  import;
- `RuntimeScriptInputProperties::clone_for_scripted_object` and
  `RuntimeScriptedListenerInputBindingOccurrence::from_definition` retain one
  clone-owned Core value/DataBind/converter occurrence in authored order;
- Rust collection ownership drops the occurrence and its bind atomically,
  replacing the concrete destructor's `removeProperty` pointer unlink.

The ownership representation is adapted to Rust, but the imported/dropped
status, list order, occurrence isolation, and disposal boundary are exact.
`runtime_import_status_tracks_scripted_object_input_contexts`, the binary
`cpp_import` scripted-object comparison, and
`typed_script_inputs_apply_live_cpp_core_values_and_keep_occurrences_isolated`
cover those shared paths.

## `src/script_input_boolean.cpp` and `include/rive/script_input_boolean.hpp`

| C++ symbol | Rust owner | disposition | evidence |
|---|---|---|---|
| `ScriptInputBoolean::~ScriptInputBoolean` | drop of `RuntimeScriptedListenerInputBindingOccurrence` from its owning scripted object | adapted | occurrence replacement/disposal tests |
| `ScriptInputBoolean::import` | binary scripted-object import stack; `runtime_scripted_object_definition`; `runtime_scripted_object_binding_definition` | exact | import-status and C++/Rust import comparison |
| `ScriptInputBoolean::onAddedClean` | ordered `ScriptListenerInputDefinition` and binding-definition construction | exact | imported input order/count comparison |
| `ScriptInputBoolean::initScriptedValue` (inline) | `prepare_script_listener_hydration`; `ScriptListenerActionHydration::apply_inputs` | exact | typed scalar hydration tests |
| `ScriptInputBoolean::validateForScriptInit` (inline) | scalar branch of `prepare_script_listener_hydration` | exact | whole-object validation tests |
| `ScriptInputBoolean::propertyValueChanged` | `RuntimeScriptInputProperties::apply_target`; `apply_scripted_input_update` | exact | `typed_script_inputs_apply_live_cpp_core_values_and_keep_occurrences_isolated`; scalar projection tests |

## `src/script_input_color.cpp` and `include/rive/script_input_color.hpp`

| C++ symbol | Rust owner | disposition | evidence |
|---|---|---|---|
| `ScriptInputColor::~ScriptInputColor` | owning occurrence drop | adapted | occurrence replacement/disposal tests |
| `ScriptInputColor::import` | binary scripted-object import stack; runtime definition builders | exact | import-status and C++/Rust import comparison |
| `ScriptInputColor::onAddedClean` | ordered input/binding-definition construction | exact | imported input order/count comparison |
| `ScriptInputColor::initScriptedValue` (inline) | `prepare_script_listener_hydration`; `ScriptListenerActionHydration::apply_inputs` | exact | typed scalar hydration tests |
| `ScriptInputColor::validateForScriptInit` (inline) | scalar hydration preflight | exact | whole-object validation tests |
| `ScriptInputColor::propertyValueChanged` | `RuntimeScriptInputProperties::apply_target`; `apply_scripted_input_update` | exact | typed live-Core and projection tests |

The signed C++ color value and Rust `u32` retain identical bits at the script
boundary.

## `src/script_input_number.cpp` and `include/rive/script_input_number.hpp`

| C++ symbol | Rust owner | disposition | evidence |
|---|---|---|---|
| `ScriptInputNumber::~ScriptInputNumber` | owning occurrence drop | adapted | occurrence replacement/disposal tests |
| `ScriptInputNumber::import` | binary scripted-object import stack; runtime definition builders | exact | import-status and C++/Rust import comparison |
| `ScriptInputNumber::onAddedClean` | ordered input/binding-definition construction | exact | imported input order/count comparison |
| `ScriptInputNumber::initScriptedValue` (inline) | `prepare_script_listener_hydration`; `ScriptListenerActionHydration::apply_inputs` | exact | typed scalar hydration tests |
| `ScriptInputNumber::validateForScriptInit` (inline) | scalar hydration preflight | exact | whole-object validation tests |
| `ScriptInputNumber::propertyValueChanged` | `RuntimeScriptInputProperties::apply_target`; `apply_scripted_input_update` | exact | live f32 projection and occurrence-isolation tests |

Rust retains the Core `float` before projecting it to the scripting backend's
number type; the SymbolListIndex-to-f32 precision boundary is tested directly.

## `src/script_input_string.cpp` and `include/rive/script_input_string.hpp`

| C++ symbol | Rust owner | disposition | evidence |
|---|---|---|---|
| `ScriptInputString::~ScriptInputString` | owning occurrence drop | adapted | occurrence replacement/disposal tests |
| `ScriptInputString::import` | binary scripted-object import stack; runtime definition builders | exact | import-status and C++/Rust import comparison |
| `ScriptInputString::onAddedClean` | ordered input/binding-definition construction | exact | imported input order/count comparison |
| `ScriptInputString::initScriptedValue` (inline) | `prepare_script_listener_hydration`; `ScriptListenerActionHydration::apply_inputs` | exact | typed scalar hydration tests |
| `ScriptInputString::validateForScriptInit` (inline) | scalar hydration preflight | exact | whole-object validation tests |
| `ScriptInputString::propertyValueChanged` | `RuntimeScriptInputProperties::apply_target`; `apply_scripted_input_update` | exact | byte-preserving Core-string and projection tests |

## `src/script_input_trigger.cpp` and `include/rive/script_input_trigger.hpp`

| C++ symbol | Rust owner | disposition | evidence |
|---|---|---|---|
| `ScriptInputTrigger::~ScriptInputTrigger` | owning occurrence drop | adapted | occurrence replacement/disposal tests |
| `ScriptInputTrigger::import` | binary scripted-object import stack; runtime definition builders | exact | import-status and C++/Rust import comparison |
| `ScriptInputTrigger::onAddedClean` | ordered input/binding-definition construction | exact | imported input order/count comparison |
| `ScriptInputTrigger::validateForScriptInit` (inline) | trigger hydration branch | exact | whole-object validation tests |
| `ScriptInputTrigger::propertyValueChanged` | `runtime_scripted_listener_bound_value`; `apply_scripted_input_update` | exact | `scripted_input_scalar_trigger_and_artboard_projection_failures_match_cpp`; repeated live trigger test |

Trigger has no initial `initScriptedValue` projection. Zero remains a no-op;
every nonzero changed Core value invokes the named table callback.

## `src/script_input_artboard.cpp` and `include/rive/script_input_artboard.hpp`

| C++ symbol | Rust owner | disposition | evidence |
|---|---|---|---|
| `ScriptInputArtboard::~ScriptInputArtboard` | owning occurrence drop; `RuntimeScriptInputArtboardOccurrence` drop | adapted | occurrence replacement/disposal tests |
| `ScriptInputArtboard::import` | binary Backboard + scripted-object context validation; runtime definition builders | exact | `runtime_import_status_tracks_scripted_object_input_contexts`; C++/Rust import comparison |
| `ScriptInputArtboard::initScriptedValue` | artboard branch of `prepare_script_listener_hydration`; prepared Artboard recipe; `apply_inputs` | pending fresh review | consumer-File/concrete-source regression; non-bypassable prepared-recipe ordering tests |
| `ScriptInputArtboard::validateForScriptInit` (inline) | resolved-reference preflight | exact | typed-artboard validation failure tests |
| `ScriptInputArtboard::validateForColdScriptInit` | cold phase accepts the unresolved live context | exact | cold/live hydration lifecycle tests |
| `ScriptInputArtboard::validateHydrationPrerequisites` | snapshot/reference preflight before any writes | exact | `scripted_hydration_validation_failure_applies_no_inputs_or_init` |
| `ScriptInputArtboard::hydrateScriptInput` | prepared-recipe validation plus authored-order construction and projection | pending fresh review | public and doc-hidden hydration boundary tests; successful authored-order trace |
| `ScriptInputArtboard::syncReferencedArtboard` | `set_artboard_input_core`; live `apply_scripted_input_update` | exact | converter production-path regression observes one projection carrying the retained live identity |
| `ScriptInputArtboard::onAddedClean` | ordered input/binding-definition construction | exact | imported input order/count comparison |
| `ScriptInputArtboard::clone` | `RuntimeScriptInputArtboardOccurrence::clone_for_scripted_object` | exact | `fresh_clone_preserves_the_exact_live_bindable_identity` |
| `ScriptInputArtboard::file` (inline) | `RuntimeScriptInputArtboardOccurrence::file_attached` | exact | resolved/unresolved clone authority test |
| `ScriptInputArtboard::artboardIdChanged` | `RuntimeScriptInputProperties::apply_target`; `apply_artboard_id_changed` | exact | missing-id clear and generated-id separation tests |
| `ScriptInputArtboard::updateArtboard` | `RuntimeScriptInputProperties::apply_artboard_source`; `RuntimeScriptInputArtboardOccurrence::apply_artboard_source` | pending fresh review | ordinary and fresh-cloned primary-converter ancestor-authority tests |
| `ScriptInputArtboard::referencedArtboardId` | generated value in `RuntimeScriptInputProperties::value`; binary `cpp_artboard_referencer_index` | exact | import resolver and generated-id tests |

The generated `artboardId` and retained referenced Artboard remain separate.
A generated-field write clears an unresolved retained pointer when File
authority exists; a ViewModel update preserves the old pointer on failed
lookup and does not rewrite the generated id.

The generated-id/pointer distinction also corrects the implementing receipt's
original owner mapping: pinned `referencedArtboardId()` returns `artboardId()`,
not the retained reference's id.

## `src/script_input_viewmodel_property.cpp`

| C++ symbol | Rust owner | disposition | evidence |
|---|---|---|---|
| `ScriptInputViewModelProperty::~ScriptInputViewModelProperty` | owning occurrence/path drop | adapted | occurrence replacement/disposal tests |
| `ScriptInputViewModelProperty::decodeDataBindPathIds` | binary DataBindPath import; `ScriptInputViewModelPropertyPath::from_imported` | exact | path import tests |
| `ScriptInputViewModelProperty::copyDataBindPathIds` | `RuntimeScriptInputProperties::clone_for_scripted_object` | exact | `view_model_property_path_is_deep_cloned_per_scripted_object_occurrence` |
| `ScriptInputViewModelProperty::initScriptedValue` | `ScriptListenerInputHydration::ViewModel`; `apply_inputs` | exact | nullable and non-null ViewModel hydration tests |
| `ScriptInputViewModelProperty::validateForScriptInit` | generator-time acceptance with no resolved child | exact | cold/live retry lifecycle tests |
| `ScriptInputViewModelProperty::validateForColdScriptInit` | cold acceptance with cleared occurrence resolution | exact | cold/live retry lifecycle tests |
| `ScriptInputViewModelProperty::validateHydrationPrerequisites` | `bound_script_view_model_property_from_owned_path`; whole-batch preflight | exact | no-partial-write validation test |
| `ScriptInputViewModelProperty::hydrateScriptInput` | phase-two path resolution and `ScriptListenerInputHydration::ViewModel` | exact | authored-order re-resolution test |
| `ScriptInputViewModelProperty::import` | DataBindPath + scripted-object importer state; runtime definition builders | exact | binary import/path tests |
| `ScriptInputViewModelProperty::onAddedClean` | ordered input/binding-definition construction | exact | imported input order/count comparison |

A valid ViewModel-valued property whose current child pointer is null still
passes preflight, performs no table write, continues to later inputs, and may
run user `init`. The path and its resolved-name buffer are independently
cloned for every scripted-object occurrence.

## Implementing correction claims

This section records the implementing lane's correction rationale and evidence.
The independent review below supersedes the claims that it falsifies.

The header-aware denominator is correct. The seven `.cpp` files contain 42
out-of-line definitions. The handwritten headers add exactly 11 executable
bodies: `validateForScriptInit` and `file` for Artboard; `initScriptedValue`
and `validateForScriptInit` for each of Boolean, Color, Number, and String; and
`validateForScriptInit` for Trigger. No additional handwritten inline body was
omitted.

The rejected ID-only representation has been replaced by an explicit retained
`ScriptArtboardSource::{File, Live}` identity. A context binding separately
retains the ViewModel Artboard state handle, so a dirty source reads its current
live `RuntimeBindableArtboard` rather than trying to reconstruct it from the
generated integer. Each state-machine ScriptInput clone also receives an
owner-Artboard source chain keyed by File identity plus global id. The pinned
order is now literal: a live asset is preferred; an ancestor live asset is
rejected without falling through; only an absent live asset uses the numeric
File lookup. Failed lookup preserves the old reference. Successful projection
carries the retained source through hydration and the facade resolver rather
than collapsing it back to an id.

`clone_for_scripted_object` now clones the exact retained live bindable
identity and copies File authority whenever that pointer exists. The generated
`artboardId` remains unchanged and continues to own
`referencedArtboardId()`.

The retained source identity is carried through both C++ ownership sites:
direct `ScriptedListenerAction` input bindings and the cloned DataBinds owned
by `ScriptedDataConverter::m_dataBinds`. Each bind/rebind stores the resolved
ViewModel Artboard state handle; source application reads the handle's current
live `RuntimeBindableArtboard` before calling the shared Artboard-referencer
owner. It therefore does not collapse converter-owned inputs to the numeric
`propertyValue` sentinel.

The nullable ViewModel verdict was checked and is sound. C++ preflight tests
the resolved `ViewModelInstanceValue`'s schema property, while
`setViewModelInput` separately observes a null
`referenceViewModelInstance()` and leaves the table field unchanged. The C++
probe and Rust hydration-order tests exercise that distinction.

Destructor accounting in the implementing result was also wrong: all seven
classes have concrete destructors, not five. Rust collection/drop ownership
is an accepted representation adaptation, and the occurrence-drop test proves
source unregistration, but that changes the arithmetic below.

Focused correction evidence with `CARGO_INCREMENTAL=0`:

- `script_input_artboard::tests`: 4 passed, including live asset precedence,
  generated-id separation, ancestor rejection, numeric fallback only when the
  live asset is absent, and live identity preservation through clone;
- `converter_owned_artboard_input_retains_the_live_view_model_source`: passed,
  including live-source replacement with an unchanged numeric sentinel and
  preservation of the authored generated `artboardId`;
- `cargo check -p nuxie --features scripting`: passed;
- `cargo test -p nuxie-runtime --no-run`: passed;

- binary scripted-object import-context test: passed;
- typed live Core values/occurrence isolation, Artboard clone authority,
  ViewModel path deep-clone, hydration-order, scalar/trigger/Artboard callback,
  embedded-NUL Core string, and occurrence-drop tests: passed;
- `script_input_bindings` integration suite: 11 passed;
- Lua C-string boundary test: passed;
- the C++ null-ViewModel probe target compiled and passed its harness, but the
  actual external probe comparison was skipped because
  `RIVE_CPP_PROBE_SCRIPTED` was not configured in this checkout;
- `cargo fmt --all -- --check`: passed.

## Independent adversarial review

The re-review read all seven pinned `.cpp` owners and all seven handwritten
headers completely, then traced both upstream ownership sites through the Rust
bind, rebind, DataContext, clone, hydration, and facade paths. The v2 census is
arithmetically sound: the owners contain 42 out-of-line functions and 11
executable header functions. The other seven v2 units are include-guard macro
definitions; they are non-behavioral `not-applicable` units under the governing
decision in this campaign's README, not part of the 53 behavioral rows.
This review is based on the accepted 1,105-owner/7,818-unit denominator in
`4144a92c5`; comparison with the prior v2 snapshot found no newly recovered or
replaced units in these 14 owners or in the directly coupled ScriptedObject,
ScriptedListenerAction, ScriptedDataConverter, ArtboardReferencer,
DataBindContextValueArtboard, ViewModelInstanceArtboard, BindableArtboard, and
Lua Artboard facade owners.

The scalar, Trigger, and ViewModel-property rows were not falsified. Neither
were Artboard import, validation, generated-id separation, file-id change,
clone identity, or the direct `ScriptedListenerAction` bind/rebind paths. In
particular, both root and DataContext resolution retain a live
`ViewModelInstanceArtboard` state handle and reread it after dirt, so replacing
the live source without changing the generated `propertyValue` sentinel is
represented. The direct listener owner also receives the containing
Artboard's ancestor-source chain.

The correction is nevertheless rejected because three end-to-end gaps remain.

### Converter-owned live changes do not update the hydrated script table

Pinned `DataBindContextValueArtboard::apply` calls
`ScriptInputArtboard::updateArtboard`; every accepted reference then calls
`syncReferencedArtboard`, which calls `setArtboardInput` even when the generated
`artboardId` is unchanged.

Rust's converter path does retain the new live source in
`update_one_input_binding`, but table projection then calls
`RuntimeScriptInputProperties::projection_value`. For an Artboard that helper
asks `referenced_artboard_id`; a retained `ScriptArtboardSource::Live` has no
file id, so it returns `None` and the function exits without invoking the
hydrated owner's `apply` callback. The direct-listener path has a separate
Artboard special case and does not have this defect.

The focused converter test cannot observe the missing side effect: it passes
`owner_instance = None` and expressly expects that no table callback runs. It
proves internal source retention and generated-id separation, but not pinned
`syncReferencedArtboard` behavior after hydration or dynamic replacement.

### Converter-owned occurrences do not reject the owner or its ancestors

Pinned `ArtboardReferencer::findArtboard` always compares a live asset against
the `ScriptInputArtboard`'s parent Artboard, irrespective of whether the cloned
input is owned by a listener action or by `ScriptedDataConverter::m_dataBinds`.

Rust installs `artboard_referencer_ancestor_sources()` only on
`scripted_object_bindings` during `StateMachineInstance` initialization.
`RuntimeScriptedDataConverterState` has no ancestor-source field or setter, so
its cloned Artboard inputs retain `ancestor_sources = None`. An owner Artboard
or ancestor supplied through a converter custom input is consequently accepted
where pinned C++ returns null and preserves the old reference. The existing
ancestor test constructs a standalone occurrence with an explicitly installed
chain and does not exercise a converter owner.

### The live facade is not cross-File faithful

Pinned `BindableArtboard` retains both its source `File` and its concrete
`ArtboardInstance`. `ScriptedObject::setArtboardInput` clones that exact
Artboard, and `ScriptReffedArtboard` obtains the default state machine from the
cloned instance itself.

Rust's `RuntimeBindableArtboard` retains only the `ArtboardInstance`.
`FileScriptArtboard::new_from_live` searches the scripted owner's resolver
`File` for a graph with the live instance's numeric `global_id`, constructs
from that unrelated file-local entry, and only then overwrites the instance.
The command-server API permits assigning any instantiated Artboard handle to
any ViewModel Artboard property without requiring matching File handles, so
this is reachable rather than a theoretical representation difference. A
valid cross-File source is rejected when its id is absent; if the two files
reuse the same global id, Rust can instead select the wrong file's default
state-machine metadata. All current live-Artboard tests use one File.

Because live resolution is deferred to `ScriptListenerActionHydration::apply_inputs`,
the cross-File facade error can also occur after earlier scalar inputs have
already been written. Pinned preflight accepts the non-null reference and the
corresponding C++ projection does not introduce that error, so the existing
whole-batch no-partial-write evidence does not cover this failure.

Focused independent evidence with `CARGO_INCREMENTAL=0` and an isolated target
directory:

- `script_input_artboard::tests`: 4 passed;
- `converter_owned_artboard_input_retains_the_live_view_model_source`: passed;
- `script_input_bindings`: 11 passed;
- ignored expected-red
  `converter_owned_live_artboard_projects_to_the_hydrated_table_expected_red`
  fails with zero table projections where pinned behavior requires one;
- ignored expected-red
  `converter_owned_artboard_input_rejects_its_owner_source_expected_red`
  fails because the owner source is accepted rather than preserving the old
  reference.

These green tests confirm the supported same-File paths but do not contradict
the code-path counterexamples above.

## Result

The seven source/header pairs contribute 42 out-of-line definitions and 11
executable inline methods. All 53 have identified Rust owners. The independent
review accepts 49 rows, including the seven approved destructor adaptations,
and rejects four Artboard rows as `missing`: `initScriptedValue`,
`hydrateScriptInput`, `syncReferencedArtboard`, and `updateArtboard`. The
receipt is **not certified** until converter-owned ancestry and live table
projection are restored and the facade carries valid live cross-File authority
without reconstructing it from a file-local numeric id.

The historical scalar RB4 gap is stale: scalar Core values,
occurrence-local scalar cloning, converter ownership, callback projection, and
file-index source/target reconciliation are live and directly exercised. That
does not close the retained Artboard-pointer branch described above.

## Implementing correction after rejection

The four rejected Artboard rows now have an implementation correction, but
this receipt deliberately remains pending until a different auditor repeats
the complete adversarial review.

- converter projection now passes the retained `ScriptArtboardSource::Live`
  directly to the already-hydrated table, matching the direct-listener path;
- ancestor authority is retained on `RuntimeScriptedDataConverterState` and
  recursively installed through Group, detached, listener-owned,
  state-machine graph, keyframe graph, and Artboard-level converter owners;
  fresh scripted occurrence resets preserve or reinstall that authority;
- `RuntimeBindableArtboard` now retains its source facade `File` opaquely
  alongside the exact `ArtboardInstance`; `FileScriptArtboard::new_from_live`
  uses that File and derives the default state machine from the retained
  instance rather than the consumer File's colliding numeric graph id;
- artboard facades are resolved by `preflight_artboards` before the hydration
  object receives a `ScriptInstance`, so a later artboard failure cannot leave
  an earlier scalar or context write behind.

Focused correction evidence with `CARGO_INCREMENTAL=0` and isolated target
directories:

- `converter_owned_`: 7 passed, including both former expected-red tests with
  their ignores removed;
- `artboard_facade_failure_precedes_every_hydration_write`: passed;
- `live_scripted_artboard_uses_its_source_file_despite_global_id_collision`
  was added with two distinct Files that deliberately reuse the same graph id;
  its `nuxie` unit target is currently blocked before execution because the
  pre-existing `silver-corpus` dev dependency still implements the obsolete
  numeric ScriptArtboard API;
- `cargo check -p nuxie-runtime -p nuxie --features nuxie/scripting`: passed.

These results are implementing-lane claims, not an independent acceptance.

## First independent re-review of correction `01e0c65bf`

Verdict: **REJECTED.** The correction closes the two converter-specific
failures from `ffa9b4bfb`, but its facade File oracle is contrary to pinned
C++, and atomic preflight is not enforced by the hydration boundary itself.

### Prior-failure closure matrix

| Prior failure | Independent result |
| --- | --- |
| Converter-owned live changes retain the new source but do not update the hydrated table | **Closed.** `update_one_input_binding` now forwards the retained `ScriptArtboardSource::Live` whenever the Artboard target reports `ChangedWithTableProjection`. The real state-machine callback passes that value through `apply_scripted_input_update`, whose Artboard branch resolves it and invokes `set_artboard_input_core`. The former expected-red projection test now observes exactly one callback carrying the same bindable identity. |
| Converter-owned occurrences do not reject their owner/ancestors | **Closed.** `RuntimeScriptedDataConverterState` now owns the Artboard ancestor chain; installation reaches direct listener converters, converter-owned DataBinds, nested Groups, detached states, state-machine and keyframe graphs, and all Artboard-level converter state collections. Fresh clones/resets preserve or reinstall it. The former owner-source expected-red is green, and the already-existing direct occurrence evidence covers an actual ancestor rather than only the owner. |
| Live facade reconstructs a cross-File source through the consumer File's colliding numeric graph id | **Mechanism closed, replacement oracle rejected.** The bindable now retains an opaque source-File pin and no longer needs a consumer-file graph-id lookup. However, `FileScriptArtboard::new_from_live` promotes that pin into the scripted facade's operative File. Pinned C++ does not. |
| A fallible facade resolution can occur after earlier hydration writes | **Closed only in the two `nuxie` preparation helpers; not closed structurally.** Those helpers call `preflight_artboards` before returning a hydration batch, and the focused helper test proves that explicit call performs no `ScriptInstance` writes. Public/doc-hidden hydration entry points can still receive an unresolved `Artboard` variant and call `apply_inputs` directly. |

### The source-File correction uses the opposite pinned File

The earlier rejection was right that a valid cross-File live Artboard must not
be rediscovered by numeric graph id in the consumer File. It was wrong to infer
that pinned scripting then uses the bindable's source File as the facade File.
The complete pinned call chain is explicit:

1. `BindableArtboard` retains its source `File` privately to keep the concrete
   Artboard instance and its source objects alive.
2. `ArtboardReferencer::findArtboard` returns the bindable's concrete
   `ArtboardInstance`; it has no access to `BindableArtboard::m_file`.
3. `ScriptInputArtboard::syncReferencedArtboard` passes only that Artboard
   pointer to `ScriptedObject::setArtboardInput`.
4. `ScriptedObject::setArtboardInput` clones the exact Artboard, but passes
   `scriptAsset()->file()` to the `ScriptedArtboard` constructor.
5. `ScriptReffedArtboard` stores that consumer/script File in `m_file`, uses it
   for `createViewModelInstance`, and derives the default state machine
   separately from the cloned source Artboard instance.

Rust now calls `Self::new(source_file, artboard_index, ...)`, so the source File
becomes both the facade File and the ViewModel-creation authority. The new test
asserts `scripted.file` is the source File; pinned C++ requires the resolver's
script/consumer File there. This is not merely lifetime management: the File
is retained on `FileScriptArtboard`, participates in ViewModel creation and
asset lookup, and is observable through subsequent Artboard facade behavior.

The literal correction needs two separate authorities: retain the source File
only as a lifetime pin, instantiate from the already-retained concrete
Artboard without any numeric lookup, and keep the resolver/script File as the
operative `FileScriptArtboard.file`. The default state-machine correction is
sound: both implementations derive it from the retained/cloned Artboard
instance rather than a colliding file-local graph.

### Atomic hydration remains caller-optional

`preflight_artboards` itself is correctly ordered and the ordinary `nuxie`
listener/converter builders both invoke it. The invariant is not owned by
`ScriptListenerActionHydration`, however:

- `ScriptListenerActionHydration::apply` installs Context and then calls
  `apply_inputs` without first calling `preflight_artboards`;
- `apply_inputs` still accepts `ScriptListenerInputHydration::Artboard`,
  resolves it inline in authored order, and therefore can fail after Context
  and earlier scalar writes; and
- the doc-hidden but public state-machine and converter hydration methods call
  `apply_inputs` on whatever batch their public callback returns, without
  enforcing a prepared-only state.

Thus the same counterexample remains representable: construct a batch with a
scalar followed by an Artboard whose resolver fails, return it from a hydration
callback (or call `apply`), and observe the scalar write before the error. The
focused test proves only an explicit call to the helper; it never supplies a
`ScriptInstance` and cannot falsify this bypass. Exact ownership requires the
hydration boundary itself to preflight, or a type-state/API split that makes an
unprepared batch impossible to apply.

### Silver-corpus diagnosis

The `nuxie` unit target currently fails while compiling its dev dependency
`tools/silver-corpus`, with five direct API-shape errors: the resolver still
takes `u64`, the prepared enum still stores `artboard_id: usize`, and hydration
still initializes the removed `artboard_id` field instead of `source`.
`SilverScriptArtboardResolver` intentionally rejects every Artboard request
and never realizes a facade, so this stale adapter does not hide a separate
production resolution policy. It is test-only drift caused by the retained
source API change. Updating it should preserve its inert policy while matching
`ScriptArtboardSource::{File,Live}`; it does not justify changing production
back to numeric ids.

The drift does mean the cross-File `nuxie` regression has never compiled or
run. More importantly, that test's expected File identity must be corrected
before it becomes evidence; merely updating silver-corpus would make a
source-opposite assertion green.

### Independent focused evidence

With `CARGO_INCREMENTAL=0`:

- `cargo test -p nuxie-runtime converter_owned_ -- --nocapture`: seven passed,
  including both former expected-red converter tests;
- `cargo test -p nuxie-runtime --lib
  artboard_facade_failure_precedes_every_hydration_write -- --nocapture`: one
  passed, proving the explicit helper but not the bypass above;
- `cargo test -p nuxie --lib --features scripting
  live_scripted_artboard_uses_its_source_file_despite_global_id_collision
  --no-run`: failed in `silver-corpus` with the five stale numeric-API errors
  described above, before the `nuxie` test could compile;
- `cargo check -p nuxie-runtime -p nuxie --features nuxie/scripting`: passed;
- `make --no-print-directory runtime-source-symbol-check`: passed at the
  corrected 1,105-owner / 7,818-unit denominator.

## Current result after first re-review

The 42 out-of-line + 11 inline behavioral census remains correct. Converter
live projection and converter ancestry are now accepted. Complete certification
is still **REJECTED** because the cross-File facade promotes a lifetime pin into
the wrong operative File and because unprepared Artboard hydration can still
bypass atomic preflight. After those exact-source corrections, the silver and
golden test adapters must migrate to the retained-source API, the corrected
cross-File oracle must execute, and a fresh independent review must repeat the
four-row audit.

## Second independent re-review of correction `01e0c65bf`

Verdict: **REJECTED.** This review was performed independently against the
exact correction commit in an isolated worktree. It re-read the pinned
Artboard-input, Artboard-referencer, ScriptedObject, BindableArtboard, and Lua
Artboard call chains rather than adopting either the correction rationale or
the first re-review. The four previously rejected source rows now divide as
follows: `syncReferencedArtboard` is accepted; `initScriptedValue`,
`hydrateScriptInput`, and `updateArtboard` remain rejected.

### Four-row adjudication

| Pinned row | Second-review result |
| --- | --- |
| `ScriptInputArtboard::initScriptedValue` | **Rejected.** The corrected ordinary `nuxie` builders resolve every Artboard facade before Context or any authored input write. Pinned `ScriptedObject::hydrateScriptInputs` validates the whole object first, but constructs and installs each input in authored phase-two order. Earlier scalar writes therefore precede a later Artboard construction. The correction changes that order and still permits bypass through public hydration APIs. |
| `ScriptInputArtboard::hydrateScriptInput` | **Rejected.** `preflight_artboards` resolves all Artboards in a separate first pass, while `apply_inputs` retains the old inline-fallible Artboard branch. This is neither the pinned authored pass nor a structural transaction: ordinary helpers reorder work, and public `apply`/`apply_inputs` callers can still fail after Context or scalar mutation. |
| `ScriptInputArtboard::syncReferencedArtboard` | **Accepted.** The converter path now forwards the retained `ScriptArtboardSource::Live` to the hydrated owner's production callback and performs one table projection even when the generated numeric id is unchanged. The former expected-red regression observes the exact retained bindable identity. |
| `ScriptInputArtboard::updateArtboard` | **Rejected.** Live-first selection, failed-lookup preservation, and ordinary owner/ancestor rejection are present. However, `StateMachineInstance::clone` fresh-clones listener bindings without reinstalling the containing Artboard's ancestry. A scripted converter in such a binding is recreated by `RuntimeDataBindGraphConverterState::for_converter`, which starts without `ancestor_sources`; the clone path never calls `initialize_scripted_clones_and_facilities`. The cloned public snapshot can therefore accept its owner or an ancestor where pinned `ArtboardReferencer::findArtboard` must reject it. |

### Pinned File authority remains reversed

The retained bindable source File is valid lifetime authority, but it is not
the operative scripting File in pinned C++. `BindableArtboard::m_file` is
private and is not returned by `ArtboardReferencer::findArtboard`.
`ScriptedObject::setArtboardInput` clones the resolved concrete Artboard and
constructs `ScriptedArtboard` with `scriptAsset()->file()`. That consumer File
is stored by `ScriptReffedArtboard` and used to create ViewModel instances;
the default state machine is obtained separately from the cloned Artboard.

`FileScriptArtboard::new_from_live` instead promotes
`RuntimeBindableArtboard::source_file_authority` to `FileScriptArtboard.file`,
searches that File for a matching graph id, and calls `Self::new` with it. The
new cross-File test explicitly asserts that source-File identity, so its oracle
is opposite to pinned behavior. Exact translation needs to keep the consumer
File operative, instantiate from the already-retained concrete Artboard, and
retain source authority only for the source object's lifetime. This finding
rejects complete `initScriptedValue`/`hydrateScriptInput` ownership even if the
preflight API is later made non-bypassable.

### Ordering evidence is internally contradictory

Both ordering tests pass at the exact correction commit, but they certify two
different contracts:

- `scripted_hydration_typed_artboard_failure_stops_later_inputs_and_init`
  exercises `apply_inputs` and observes the pinned phase-two shape: Context and
  an earlier scalar are written before a later Artboard resolution error;
- `artboard_facade_failure_precedes_every_hydration_write` calls
  `preflight_artboards` explicitly and proves only that this helper can fail
  before it is handed a `ScriptInstance`.

The second test does not prove that resolver work is unobservable, preserve
authored construction order, or prevent a caller from skipping the helper.
Consequently it cannot serve as exact-order evidence for the pinned method.

### Fresh-clone ancestry gap

The normal `StateMachineInstance::new` path calls
`initialize_scripted_clones_and_facilities` and installs one ancestor-source
chain on listener bindings, the state-machine DataBind graph, and keyframe
graphs. Several nested converter reset paths also preserve this metadata.
Those facts make the ordinary correction tests green, but do not cover the
public snapshot path:

1. `StateMachineInstance::clone` maps every scripted listener binding through
   `RuntimeScriptedListenerActionBindingOccurrence::fresh_clone`.
2. That method preserves each input's outer `properties`, but constructs its
   primary `converter_state` with
   `RuntimeDataBindGraphConverterState::for_converter`.
3. A newly constructed scripted converter state has no ancestor-source chain.
   Preservation inside `converter_data_binds.fresh_clone()` applies only to
   those nested/detached states, not to this primary converter state.
4. The `Clone` implementation does not rerun
   `initialize_scripted_clones_and_facilities`.

No correction test takes this route. Therefore the prior failure “converter
occurrences do not reject owner/ancestors” is closed for ordinary construction
but not for all live Rust owners of the pinned row.

### Silver-corpus scope

The stale adapter is excluded from the shipping `nuxie` library: it is a
`[dev-dependencies]` entry and every `nuxie` reference found by this review is
under `crates/nuxie/tests`. That is why `cargo check -p nuxie --features
scripting` succeeds.

It is nevertheless not literally test-only code. `silver-corpus` is a normal
workspace member and a normal library target, with the obsolete API in
unconditional `tools/silver-corpus/src/scripting.rs`. At the exact correction
commit, `cargo check -p silver-corpus` fails with five errors: the resolver
still takes `u64`, File-source filtering and integer conversion still assume a
numeric value, and hydration initializes the removed `artboard_id` field
instead of `source`. The drift does not affect the production library artifact,
but it blocks workspace checks and all `nuxie` test-target compilation. It
should be classified as stale test-support/workspace code, not as a hidden
production policy and not as code gated by `cfg(test)`.

### Second-review evidence

With `CARGO_INCREMENTAL=0` at detached `01e0c65bf`:

- all seven `converter_owned_` focused tests passed;
- `artboard_facade_failure_precedes_every_hydration_write` passed;
- `nested_scripted_converter_reset_preserves_artboard_ancestor_authority`
  passed, but exercises a nested reset rather than the listener primary-state
  clone counterexample above;
- both authored-order tests passed:
  `scripted_hydration_resolves_artboard_then_viewmodel_in_authored_apply_order`
  and
  `scripted_hydration_typed_artboard_failure_stops_later_inputs_and_init`;
- `cargo check -p nuxie --features scripting` passed;
- `cargo check -p silver-corpus` failed with the five stale API errors above.

The 42 out-of-line + 11 inline census remains accepted. The second review
accepts 50 of 53 behavioral rows and rejects three Artboard rows. Certification
requires literal consumer-File semantics, one non-bypassable hydration design
that preserves pinned authored phase-two behavior, and ancestor authority on
the listener-converter `StateMachineInstance::clone` path. Silver-corpus then
needs a separate API-shape migration before the blocked `nuxie` evidence can
compile and run.

## Second correction after the two independent rejections

Verdict: **PENDING A FRESH TWO-REVIEW CYCLE.** This section records the
implementing lane's correction; it does not supersede either rejection until
two different reviewers have independently accepted the complete pinned call
chain and the three rows above.

The cross-File facade now retains the source `File` only as a lifetime pin.
The consumer/script `File` remains the operative `ScriptReffedArtboard` File
for ViewModel lookup, DataContext binding, and scripting authority. The
already-resolved concrete source Artboard carries its exact immutable runtime,
graph catalog, graph index, and source rendering resources into the recipe.
No numeric `graph_global_id` lookup is performed in either File, so a colliding
consumer graph cannot replace the concrete source.

Hydration now has an explicit unprepared/prepared type-state boundary.
`ScriptListenerActionHydration::apply` and `apply_inputs` own that boundary,
and every doc-hidden state-machine, converter, and interpolator callback turns
an unprepared callback result into a prepared batch before any authored
phase-two input write. Preparation consumes every fallible Artboard branch.
For a File source it resolves the graph index, constructs and validates the
runtime occurrence, retains the exact catalog, and prepares the consumer-File
ViewModel/DataContext/state-machine state. For a live source it retains the
concrete occurrence, lifetime pin, exact source catalog/resources, and the
same consumer-File state. The resulting production recipe's `construct`
method can only move those immutable prepared fields into
`FileScriptArtboard` and box it; it performs no lookup, resolver call, runtime
construction, or fallible operation. Allocation failure remains Rust's normal
abort/panic domain rather than a semantic `Result` branch.

This preserves the pinned successful phase-two trace: recipe construction and
table publication still happen at the Artboard input's authored position, so
an earlier scalar write precedes them. A preparation failure occurs before
any authored phase-two input write. The pinned Context-before-validation route
remains Context-first at the state-machine boundary, while direct public batch
application validates before installing Context.

`RuntimeScriptedListenerActionBindingOccurrence::fresh_clone` now copies the
primary converter state's Artboard ancestor-source authority into the fresh
state. This closes the `StateMachineInstance::clone` path that the second
review separated from already-correct nested converter clones.

The unconditional silver-corpus workspace adapter and the golden runner now
use `ScriptArtboardSource` and the prepared resolver contract. The former
retains its deliberately inert Artboard policy; the latter discharges its
fallible harness construction during preparation.

Focused correction evidence with `CARGO_INCREMENTAL=0` and warnings suppressed
only to keep the existing vendor warning volume readable:

- `cargo check -p nuxie-runtime -p nuxie -p silver-corpus --features
  nuxie/scripting`: passed;
- `cargo check -p rust-golden-runner`: passed;
- `scripted_hydration_`: 4 passed, including successful authored construction
  order and preparation failure before phase-two writes;
- `public_hydration_apply_and_apply_inputs_cannot_bypass_artboard_preparation`:
  passed for both public entry points;
- `fresh_clone_preserves_primary_converter_artboard_ancestor_authority`:
  passed;
- `converter_owned_`: 7 passed;
- `live_scripted_artboard_uses_consumer_file_and_retains_concrete_cross_file_source`:
  passed, including the colliding-id concrete source and source-pin lifetime.

The broader shared-worktree run reached 1,109 passing runtime library tests
with eight failures outside this receipt: seven are in the concurrently dirty
HitTester/semantic-geometry lane and one is the AABB/layout solve-count test.
The full `nuxie` library run reached 54 passing tests with the unrelated
malformed-Luau-bytecode host-log expectation failing. None of those failures
intersects the focused ScriptInput owners above; they are recorded rather than
silently treated as a green workspace gate.

These tests justify returning the three rows to review, not self-acceptance.

## First fresh independent review of correction `6b3f7c320`

Verdict: **REJECTED.** The correction closes the consumer-File promotion bug,
the listener primary-converter fresh-clone ancestry gap, the public bypass of
fallible Artboard preparation, and the stale silver/golden API adapters. It
does not preserve the pinned successful phase-two construction boundary, and
the production facade still chooses an auto-created ViewModel from immutable
source metadata instead of the concrete live Artboard occurrence.

### Production preparation performs the construction it claims to defer

Pinned `ScriptedObject::hydrateScriptInputs` first calls only
`validateHydrationPrerequisites()` for every input. For
`ScriptInputArtboard`, that validation is exactly
`m_referencedArtboard != nullptr`. During the second authored loop,
`hydrateScriptInput()` calls `syncReferencedArtboard()`, and only then does
`ScriptedObject::setArtboardInput` call `artboard->instance()`, set
`frameOrigin(false)`, construct `ScriptedArtboard`/`ScriptReffedArtboard`,
create or retain its ViewModel, construct its default state machine, and bind
the child DataContext.

The corrected File resolver does those semantic operations inside
`prepare_script_artboard`, before the authored input loop:

- `FileScriptArtboard::prepare_with_view_model` constructs the complete
  `RuntimeArtboardInstance`;
- `prepare_from_live` calls `RuntimeBindableArtboard::artboard_instance`,
  which cold-clones the concrete live occurrence at that same preflight
  boundary;
- `prepare_from_concrete` sets frame origin, creates the ViewModel, binds the
  Artboard DataContext, constructs the default `StateMachineInstance`, calls
  `bind_script_artboard_data_context`, and immediately calls
  `advance_data_context`;
- `PreparedFileScriptArtboard::construct` then only moves that already-built
  state into `FileScriptArtboard` and allocates the box.

Consequently an earlier authored scalar still precedes table *publication*,
but it no longer precedes Artboard cloning, ViewModel creation, DataContext
binding, state-machine construction, or the initial context advance. Those
are precisely the operations performed by the pinned phase-two
`setArtboardInput` call chain. The successful-order test does not exercise
this production behavior: its test resolver builds `ProjectionArtboard`
during `prepare_script_artboard`, while its `construct` trace is labeled
`resolve-artboard`. The trace therefore proves box publication order, not
Artboard construction order.

This is also observable through the public prepared facade. A caller can
preflight a `ScriptArtboardSource::Live`, refresh the retained
`RuntimeBindableArtboard` to a different concrete occurrence, and then apply
the public `PreparedScriptListenerActionHydration`; the prepared recipe
publishes the earlier clone. The pinned method has no externally suspendable
validation/construction split and reads the retained Artboard at its authored
phase-two position.

The type-state fence is accepted as a non-bypassable discharge of Rust-only
semantic `Result` branches. Exact translation still requires the prepared
value to retain immutable constructor authority rather than the constructed
occurrence. The phase-two `construct` boundary must perform the infallible
equivalents of `Artboard::instance`, ViewModel/DataContext setup, and default
state-machine ownership in authored order.

### Live Artboard ViewModel selection reads the wrong source

Pinned `ScriptReffedArtboard` receives the concrete clone first and then calls
the consumer `File::createViewModelInstance(m_artboard.get())`. That overload
reads `m_artboard->viewModelId()` from the cloned live Artboard occurrence.
`ArtboardBase::viewModelId` is a generated mutable property, and
`Artboard::instance()` copies its current value.

`FileScriptArtboard::prepare_from_concrete` instead reads
`source_runtime.artboard(artboard_index).uint_property("viewModelId")`. For a
live bindable whose concrete Artboard occurrence has changed `viewModelId`,
the Rust facade ignores the copied live value and chooses the consumer File's
ViewModel using the immutable authored source catalog. The current cross-File
test changes File identity and collides graph ids, but leaves `viewModelId`
equal to its authored value, so it cannot detect this discrepancy. The
default state-machine selection does correctly read the concrete `instance`;
ViewModel selection must use the same live-occurrence authority.

### Adjudication

| Pinned boundary | First fresh-review result |
| --- | --- |
| `ScriptInputArtboard::initScriptedValue` / `hydrateScriptInput` | **Rejected.** Fallible prerequisites are structurally fenced, but production Artboard/ViewModel/DataContext/default-state-machine construction runs during preflight rather than at this input's authored phase-two position. |
| `ScriptInputArtboard::syncReferencedArtboard` | **Accepted locally.** It forwards the retained File/live source and performs the table projection, but the downstream File facade remains rejected for the construction-phase and live-`viewModelId` discrepancies above. |
| `ScriptInputArtboard::updateArtboard` | **Accepted.** Live-first source selection, failed-lookup preservation, owner/ancestor rejection, and fresh-clone primary-converter ancestry now follow the pinned call chain. |
| Cross-File `ScriptReffedArtboard` ownership | **Accepted in part.** The consumer File remains operative and the source File is only a lifetime/resource pin; exact auto-ViewModel selection still fails when the concrete live occurrence's `viewModelId` differs from immutable source metadata. |

### First fresh-review evidence

At correction commit `6b3f7c320`, with `CARGO_INCREMENTAL=0`:

- `public_hydration_apply_and_apply_inputs_cannot_bypass_artboard_preparation`:
  1 passed;
- `scripted_hydration_`: 4 passed;
- `fresh_clone_preserves_primary_converter_artboard_ancestor_authority`:
  1 passed;
- `live_scripted_artboard_uses_consumer_file_and_retains_concrete_cross_file_source`:
  1 passed;
- `cargo check -p nuxie-runtime -p nuxie -p silver-corpus --features
  nuxie/scripting`: passed.

Those results accept the structural fence, adapter migration, cross-File pin,
and ancestry correction. They do not cover either rejected production
counterexample. Certification remains rejected pending a correction and a
new two-review cycle.

## Phase-two timing and live-ViewModel correction candidate

Status: **REJECTED by the first fresh independent review.** Neither later
review may inherit an acceptance from the rejected `6b3f7c320` cycle or this
rejected correction cycle.

The correction keeps the non-bypassable prepared hydration type, but changes
what it is allowed to retain. `prepare_script_artboard` now validates and
retains only constructor authority:

- a File source validates its immutable Artboard catalog entry and retains the
  consumer File, source id, and parent context;
- a live source retains the stable `RuntimeBindableArtboard` handle itself;
- neither path clones an `ArtboardInstance`, selects or creates a ViewModel,
  binds a DataContext, constructs a default state machine, advances that
  context, or snapshots a public live occurrence during validation.

`PreparedScriptArtboard::construct` is now honestly fallible. The authored
phase-two loop calls it at the corresponding `setArtboardInput` position, so
Rust-only semantic errors surface there instead of being moved into the first
loop or hidden by `expect`. The File facade takes the live bindable snapshot,
clones/constructs the concrete occurrence, sets `frameOrigin(false)`, selects
and binds the ViewModel/DataContext, creates the default state machine, and
advances its DataContext only at that boundary. The golden and silver adapters
now preserve the same validation/construction split.

The auto-created ViewModel now reads the fresh concrete occurrence's live
root `Artboard.viewModelId`. It resolves that id in the consumer File and uses
the generated `File::createViewModelInstance` analogue rather than an authored
default instance. Immutable `source_runtime.artboard(index)` metadata is no
longer consulted for this selection.

### Correction witnesses

- `scripted_hydration_resolves_artboard_then_viewmodel_in_authored_apply_order`
  constructs a real `ProjectionArtboard` recipe only after validation and
  records construction before publication;
- `prepared_artboard_semantic_failure_occurs_at_its_authored_phase_two_position`
  proves an earlier scalar setter runs before a prepared Artboard's semantic
  construction failure, while publication and later setters do not run;
- `prepared_live_script_artboard_snapshots_the_occurrence_during_construct`
  validates one live bindable occurrence, refreshes it, and proves construct
  uses the replacement occurrence's dimensions;
- `live_scripted_artboard_selects_consumer_view_model_from_mutated_occurrence`
  leaves the immutable source Artboard at `viewModelId == 0`, mutates the live
  occurrence to `viewModelId == 1`, and proves the consumer File creates and
  binds the second model: its numeric property drives a concrete Rectangle
  DataBind whose same-index property is a string in the immutable first model.

### Correction evidence

With `CARGO_INCREMENTAL=0`:

- `cargo test -p nuxie-runtime scripted_hydration_ --lib`: 4 passed;
- `cargo test -p nuxie-runtime
  prepared_artboard_semantic_failure_occurs_at_its_authored_phase_two_position
  --lib`: 1 passed;
- `cargo test -p nuxie-runtime
  public_hydration_apply_and_apply_inputs_cannot_bypass_artboard_preparation
  --lib`: 1 passed;
- the two new `nuxie` live-occurrence witnesses and the existing cross-File
  ownership witness: 3 passed;
- `cargo test -p nuxie-runtime scripted_listener_action_tests --lib`: 100
  passed;
- `cargo test -p nuxie --features scripting --lib`: 56 passed, 1 ignored,
  and the unrelated
  `inert_script_import_tests::file_host_log_sink_reaches_its_lazy_scripting_vm`
  fixture failed because its embedded Luau bytecode version `1` is outside the
  current supported range `3..=13`; the failure reproduces in isolation;
- `cargo check -p nuxie-runtime -p nuxie -p silver-corpus --features
  nuxie/scripting`: passed;
- `cargo check -p rust-golden-runner --features scripting`: passed.

These results justify restarting the two-review cycle. They do not certify the
rows themselves.

## First fresh independent review of correction `124bc7598`

Verdict: **REJECTED.** The correction does restore the outer authored input
position and reads a refreshed live occurrence's mutable `viewModelId` at that
position. It also preserves primary-converter ancestor authority across the
audited fresh-clone path. The complete construction call chain is still not
the pinned `ScriptedObject::setArtboardInput` / `ScriptReffedArtboard` chain.

### Construction is evaluated outside the pinned live-table guard

Pinned `ScriptedObject::hydrateScriptInputs` returns before validation when
`state() == nullptr || m_self == 0` (`src/scripted/scripted_object.cpp:399-405`).
Even after entering the authored loop, `setArtboardInput` checks both `state()`
and `scriptAsset()` before it calls `artboard->instance()`
(`src/scripted/scripted_object.cpp:43-58`). An inert occurrence therefore does
not clone an Artboard, select a ViewModel, create a state machine, or bind a
DataContext.

Rust evaluates `recipe.construct()?` as the argument to
`set_artboard_input_core` (`crates/nuxie-runtime/src/scripting.rs:447-459`).
The production backend's missing-table guard is inside
`set_artboard_input_core` (`crates/nuxie-scripting/src/vm.rs:2811-2829`), after
that evaluation. The public prepared API can consequently construct and bind
a child, or surface a construction error, for an instance on which the pinned
setter is inert. The ordinary live-update path has a separate
`script_lifetime_valid` guard, but the shared hydration type does not own this
invariant. Golden and silver use the same eager argument boundary.

Correction must put deferred construction behind the `ScriptInstance`
live-table/script-asset guard, for example by giving the backend a prepared
recipe/closure rather than a preconstructed `Box<dyn ScriptArtboard>`.

### The production `ScriptReffedArtboard` sequence is reversed and duplicated

Pinned order is mechanical:

1. `Artboard::instance()` copies the concrete source occurrence, including its
   current `m_DataContext` (`include/rive/artboard.hpp:548-564`).
2. `frameOrigin(false)` is set.
3. `ScriptReffedArtboard` constructs `m_stateMachine` from
   `m_artboard->defaultStateMachine()` in its member initializer
   (`src/lua/lua_artboards.cpp:21-38`). If the cloned live Artboard already has
   a DataContext, `ArtboardInstance::stateMachineAt` inherits it
   (`src/artboard.cpp:2906-2919`).
4. The consumer File creates the ViewModel from the cloned Artboard's current
   `viewModelId`, then the new consumer/parent DataContext is bound once
   (`src/lua/lua_artboards.cpp:40-60`).
5. The ViewModel is tracked for host frame-tail advancement; the constructor
   does not call `advancedDataContext` (`src/lua/lua_artboards.cpp:61-67`;
   `src/lua/rive_lua_libs.cpp:1234-1269`).

`FileScriptArtboard::from_concrete` instead selects the consumer ViewModel,
binds the Artboard DataContext, constructs the default state machine after
that bind, binds the same context to the state machine again, and immediately
calls `advance_data_context` (`crates/nuxie/src/lib.rs:3795-3844`). The first
bind causes `state_machine_instance` to inherit the already-installed context
(`crates/nuxie-runtime/src/artboard.rs:5792-5810`); the explicit second bind is
therefore not the single pinned replacement bind. Immediate advance consumes
trigger state during construction, while pinned tracking defers that consume
to the host frame tail. A live source already carrying a different context
also loses the pinned source-context inheritance phase before consumer rebind.

The golden runner repeats the same shape: it prebinds the Artboard before
state-machine construction, binds again afterward, and calls
`advance_data_context` (`tools/rust-golden-runner/src/main.rs:3668-3761`). Its
green output cannot certify this production ordering.

Correction must reproduce the constructor sequence directly: clone with its
live source context, create the default state machine, select the consumer
ViewModel, bind the replacement consumer/parent context exactly once, retain
the ViewModel for frame-tail advancement, and perform no construction-time
`advance_data_context`.

### Cross-File DataBind resolution uses the consumer catalog

The consumer File is correctly retained as `ScriptReffedArtboard::m_file` and
correctly owns auto-ViewModel creation. It is not the source Artboard's
DataBindPath resolver. In pinned C++, every child DataBindPath was imported by
the source File; relative paths consult `dataBindPath->file()->dataResolver()`
(`src/data_bind/data_context.cpp:458-475`).

Rust retains `source_runtime` and `source_artboards`, but passes
`file.runtime` (the consumer) into
`instance.bind_script_artboard_data_context` (`crates/nuxie/src/lib.rs:
3827-3835`). That File parameter drives relative/name-based path resolution
inside the source Artboard graph. The existing cross-File witnesses use
matching File schemas and cannot distinguish the two catalogs. A source File
and consumer File with incompatible manifests or same-index/different-name
models can therefore bind a different property than pinned, or bind one where
pinned remains unresolved.

Correction must keep the authorities split: consumer File for facade and
ViewModel creation; source RuntimeFile for the cloned Artboard/default state
machine's imported DataBind and relative-path resolution.

### Live preflight accepts an absent concrete occurrence

Pinned `ScriptInputArtboard::validateHydrationPrerequisites` is exactly
`m_referencedArtboard != nullptr` (`src/script_input_artboard.cpp:56-64`). The
public Rust source type can represent
`ScriptArtboardSource::Live(RuntimeBindableArtboard::new(...))`, whose retained
concrete occurrence is `None`. `FileScriptArtboardResolver::prepare_script_artboard`
accepts every `Live` value without checking that prerequisite
(`crates/nuxie/src/lib.rs:3646-3678`), then `construct` fails after any earlier
authored scalar. That is the opposite of pinned no-partial-write validation.
Production DataBind projection rejects this shape when it is created, but the
public hydration boundary claims the invariant and does not enforce it.

Correction must validate live occurrence presence without constructing the
facade, retain the stable bindable authority, and still take the fresh clone at
the authored phase-two position.

### Accepted portions and evidence

- A refreshed `RuntimeBindableArtboard` is retained through preparation and
  snapshotted by `construct`, not during validation.
- The current concrete root's mutable `viewModelId` is read and the consumer
  File creates that generated model; immutable source Artboard metadata is no
  longer used for this selection.
- `RuntimeScriptedListenerActionBindingOccurrence::fresh_clone` carries the
  primary converter's ancestor sources into its fresh converter state.
- File-source prerequisite validation remains write-free and the authored
  scalar/artboard/ViewModel iteration order is retained.

At `124bc7598`, with `CARGO_INCREMENTAL=0`, the focused correction tests all
passed: `scripted_hydration_`, the prepared semantic-failure witness, the live
replacement snapshot witness, the mutated-live-`viewModelId` witness, and
`fresh_clone_preserves_primary_converter_artboard_ancestor_authority`. Those
tests accept the listed portions but do not exercise the rejected live-table
guard, source-context/default-state-machine order, construction-time trigger
consume, incompatible cross-File resolver, or empty-live preflight cases.

## Phase-two correction after fresh review `498ca5ba1`

Status: **PENDING two fresh independent reviews.** This correction addresses
the four blockers in `498ca5ba1`; it does not certify any row before both new
reviews accept the combined behavior.

### Recovered pinned sequence

- Prepared Artboard construction now crosses the backend-owned
  state/live-table/ScriptAsset guard before `PreparedScriptArtboard::construct`
  can run. The Luau backend checks both its concrete table (`m_self`) and
  retained generator (`scriptAsset`) before cloning, binding, or publishing a
  child, matching `src/scripted/scripted_object.cpp:43-59,399-405`.
- Production and golden `ScriptReffedArtboard` paths retain the cloned source
  occurrence, create its default state machine before consumer ViewModel
  selection, then bind the selected consumer/parent context once across the
  split Rust Artboard/state-machine owners. Neither path performs a
  construction-time DataContext advance. Existing Luau userdata and golden
  registered-File frame owners retain the selected VM for host frame-tail
  advance, matching `src/lua/lua_artboards.cpp:21-67` and
  `src/lua/rive_lua_libs.cpp:1234-1269`.
- The consumer File still owns facade creation and generated ViewModel
  selection from the fresh concrete occurrence's mutable `viewModelId`, but
  the source RuntimeFile now remains the imported child DataBindPath resolver.
  This preserves the authority split in
  `src/data_bind/data_context.cpp:458-475`.
- The first prerequisite loop now rejects `Live` sources with no concrete
  occurrence through an allocation-free presence query. A valid bindable is
  still retained through preflight and freshly snapshotted only at its
  authored phase-two position.

The previously accepted refreshed-live snapshot, mutable `viewModelId`,
consumer-created ViewModel, authored outer ordering, and primary-converter
ancestor authority are unchanged.

### New falsification witnesses

- `empty_live_occurrence_is_rejected_in_the_first_validation_loop` proves the
  missing occurrence fails before resolver preparation or any phase-two write.
- `inert_script_lifetime_returns_before_deferred_artboard_construction` and
  `artboard_setter_checks_script_asset_before_running_prepared_constructor`
  use constructors that count and fail if evaluated; both remain uncalled
  behind the shared occurrence-lifetime and concrete Luau ScriptAsset guards.
- `cross_file_script_artboard_resolves_child_data_binds_with_source_manifest`
  uses incompatible real name/path manifests. Its negative control proves the
  consumer resolver drives `consumerOnly=22`; the production path instead
  retains and inherits the live source's bound `secondOnly=7` context, then
  uses the source resolver for the consumer replacement and preserves width
  `7`, so matching numeric indices cannot produce a false green.
- `scripted_child_advance_does_not_consume_the_supplied_root_trigger` now has
  an authored default state machine, proves its constructor did not see the
  replacement consumer context, and proves construction plus child advance do
  not consume the supplied trigger; the host frame tail does.

### Correction evidence

With `CARGO_INCREMENTAL=0`:

- `cargo test -p nuxie-runtime hydration_atomicity_tests --lib`: 3 passed;
- `cargo test -p nuxie-runtime scripted_hydration_ --lib`: 4 passed;
- `cargo test -p nuxie-runtime scripted_listener_action_tests --lib`: 100
  passed;
- `cargo test -p nuxie-scripting --features compiler
  artboard_setter_checks_script_asset_before_running_prepared_constructor
  --lib`: 1 passed;
- the new incompatible-manifest witness, the default-state-machine/frame-tail
  witness, and both existing `live_scripted_artboard_` witnesses: 4 passed;
- `cargo test -p nuxie-scripting --features compiler scripted_artboard_ --lib`:
  1 passed;
- `cargo check -p rust-golden-runner --features scripting`: passed;
- `cargo check -p nuxie-runtime -p nuxie-scripting -p nuxie -p silver-corpus
  --features nuxie/scripting`: passed.

## First fresh independent review after `dd0f5ca7f`

Status: **REJECTED.** The four new focused witnesses are green and the main
prepared-hydration path now has the intended empty-Live preflight, deferred
backend guard, source RuntimeFile resolver, default-state-machine-before-bind
order, and host-frame-tail trigger consumption. The combined behavior is not
yet certifiable because two operative ScriptInputArtboard paths bypass those
recovered invariants.

### Live Artboard updates still construct before the ScriptAsset guard

Pinned `ScriptInputArtboard::syncReferencedArtboard` calls
`ScriptedObject::setArtboardInput`, whose `state()` and `scriptAsset()` checks
both precede `Artboard::instance()` and `ScriptReffedArtboard` construction
(`src/script_input_artboard.cpp:68-79`;
`src/scripted/scripted_object.cpp:43-59`).

The ordinary bound-input update owner still checks only
`script_lifetime_valid`, calls the eager convenience
`resolve_script_artboard`, and only then passes the already constructed child
to `set_artboard_input_core`
(`crates/nuxie-runtime/src/state_machine/state_machine_instance/
state_machine_instance.rs:2005-2041`). That bypasses both the new
`script_artboard_input_context_live`/ScriptAsset guard and the prepared recipe
boundary. A live table whose ScriptAsset authority is absent can therefore
clone, bind, or fail a child where pinned C++ returns before construction.
The new counter witness exercises only prepared hydration, so it cannot catch
this update path.

Correction must retain the prepared recipe through the live update and hand
it to the backend-owned guarded setter, rather than calling
`resolve_script_artboard` before that guard.

### Golden DataContext rehydration publishes an unbound Artboard

Pinned `hydrateScriptInputs` replays `ScriptInputArtboard::hydrateScriptInput`
on every hydration, and every resulting `ScriptReffedArtboard` constructs its
default state machine and then binds its selected consumer/parent ViewModel
when both exist (`src/scripted/scripted_object.cpp:399-426`;
`src/lua/lua_artboards.cpp:21-60`).

The golden cold path calls `bind_view_model`, and the prepared listener recipe
does too. The golden DataContext rehydration path does not: it creates
`RunnerScriptArtboard` and immediately publishes it through
`set_script_artboard_input_for_global`
(`tools/rust-golden-runner/src/main.rs:6028-6045`). The constructor has already
created the default state machine, but `_data_context` remains absent and
neither the Artboard nor machine receives the selected replacement context.
This makes the receipt's statement that the golden paths bind exactly once
false, and none of the new production-focused witnesses enters this direct
golden rehydration owner.

Correction must run the same post-state-machine `bind_view_model` step before
publishing the rehydrated golden Artboard and add a witness that distinguishes
an unbound child from the pinned consumer/parent binding.

### Review evidence

At `dd0f5ca7f`, with `CARGO_INCREMENTAL=0`, the focused empty-Live and inert
constructor tests, the concrete Luau ScriptAsset constructor guard, the
incompatible cross-File manifest witness, and the default-state-machine/
frame-tail trigger witness all passed. Those results accept the corrected
prepared listener path but are non-probative for the two bypasses above.

## Operative-path correction after fresh review `933abebaa`

Status: **PENDING TWO FRESH INDEPENDENT REVIEWS.** This correction addresses
the two operative bypasses found by `933abebaa`; it does not self-certify the
ScriptInputArtboard rows or inherit an acceptance from an earlier review.

The live bound-input owner no longer calls the eager
`resolve_script_artboard` convenience path. It performs only resolver
preparation, retains the deferred recipe, and hands that recipe to
`ScriptInstance::set_prepared_artboard_input_core`. The concrete backend's
state/live-table/ScriptAsset guard therefore runs before the recipe can clone,
bind, or fail an Artboard, matching the same guarded setter used by cold and
prepared hydration. `live_artboard_update_checks_script_asset_before_deferred_construction`
keeps the outer scripted lifetime live while making the Artboard setter
context inert; preparation succeeds, the update is accepted as inert, and the
constructor trace remains empty.

The golden DataContext rehydration owner now repeats the cold/prepared
`ScriptReffedArtboard` sequence: `RunnerScriptArtboard` creates its authored
default state machine first, then `bind_view_model` installs the selected
consumer/parent context once across the Artboard and machine, and publication
follows. The frame owner is neither duplicated nor advanced during this
construction. The adjacent bound-Artboard refresh owner had the same unbound
publication shape and now performs the same post-construction bind. The cold
golden path and `PreparedRunnerScriptArtboard::construct` already used this
order and remain unchanged.

Focused correction evidence with `CARGO_INCREMENTAL=0`:

- `live_artboard_update_checks_script_asset_before_deferred_construction`:
  passed;
- `scripted_input_scalar_trigger_and_artboard_projection_failures_match_cpp`:
  passed, retaining ordinary and typed-resource failure policy;
- `artboard_setter_checks_script_asset_before_running_prepared_constructor` in
  the concrete Luau backend: passed;
- `rehydrated_script_artboard_binds_after_default_machine_and_keeps_host_frame_tail`:
  passed with a synthetic imported Artboard/ViewModel/default-state-machine
  graph, a concrete bound DataContext, and one unchanged registered File frame
  owner;
- complete `scripted_listener_action_tests`: 101 passed;
- complete golden-runner scripting unit suite: 15 passed;
- `cargo check -p rust-golden-runner --features scripting`: passed;
- combined `nuxie-runtime`, `nuxie-scripting`, `nuxie`, and `silver-corpus`
  scripting check: passed.

The source/live authority split, refreshed live snapshot, mutable
`viewModelId`, first-loop empty-Live validation, primary-converter ancestry,
and no-construction-time-advance corrections from `dd0f5ca7f` were not changed.
Certification now requires two new independent audits of the combined call
chain.

## First fresh independent review of correction `a32a53a58`

Status: **REJECTED.** The correction fixes the two literal bypasses named by
`933abebaa`: construction is now deferred behind the backend setter, and the
golden rehydration owner binds after creating the default state machine. Those
focused witnesses are green. The combined ScriptInputArtboard authority is
still not certifiable because the live-update guard is one operation too late,
the golden rehydration owner still bypasses the pinned all-input validation
transaction, and the corrected sibling refresh discards the owning parent
DataContext.

### Live updates prepare before the pinned ScriptAsset/table guard

Pinned `ScriptedObject::setArtboardInput` obtains `state()` and returns when
either the state or `scriptAsset()` is absent before it performs any Artboard
lookup, cloning, or other setter work (`src/scripted/scripted_object.cpp:
43-59`). The setter also adds `ScriptUpdate` dirt only after a live table has
accepted the new userdata.

`apply_scripted_input_update` checks only the broader
`script_lifetime_valid`, then invokes
`ScriptArtboardResolver::prepare_script_artboard` before borrowing the backend
for `set_prepared_artboard_input_core`
(`crates/nuxie-runtime/src/state_machine/state_machine_instance/
state_machine_instance.rs:2003-2044`). The new witness proves only that
`construct` remains behind the guard; its resolver preparation is deliberately
unobserved and successful. A resolver preparation with a typed resource error
still escapes an occurrence whose Luau table or ScriptAsset is absent, and a
successful inert setter still returns `Ok(true)` even though pinned C++ adds no
script dirt.

The backend-owned Artboard context guard must precede resolver preparation on
this operative update path, and the inert result must not report a successful
table projection. A counter/failing resolver witness needs to observe both
preparation and construction, not construction alone.

### Golden rehydration is not an atomic two-loop transaction

Pinned `hydrateScriptInputs` first checks every custom property's
`validateHydrationPrerequisites`; a single unresolved Artboard or ViewModel
returns before any earlier scalar, Artboard, or ViewModel is published. Only a
fully successful first loop enters the authored hydration loop
(`src/scripted/scripted_object.cpp:399-426`).

Golden `rehydrate_script_inputs` has only the authored application loop
(`tools/rust-golden-runner/src/main.rs:6130-6202`). It can publish earlier
inputs and construct an Artboard before discovering a later invalid Artboard.
More directly, an unresolved `ScriptInputViewModelProperty` takes `continue`
at lines 6171-6176, after which later inputs and caller-owned `init` handling
continue. Pinned `ScriptInputViewModelProperty::validateHydrationPrerequisites`
returns false for that same missing context/path/property
(`src/script_input_viewmodel_property.cpp:62-84`). This is the exact
partial-hydration failure covered by upstream
`hydration preflight fails atomically when VM input cannot resolve`.

The golden owner needs a separate allocation-free validation pass for every
input before construction or table writes, with unresolved ViewModel inputs
failing the whole occurrence rather than being treated as absent optional
values. The new rehydrated-Artboard test contains only one Artboard input and
cannot falsify cross-input atomicity.

### Golden bound-Artboard refresh drops the parent DataContext

Pinned `setArtboardInput` always passes the owning ScriptedObject's current
`dataContext()` to `ScriptReffedArtboard`; when a child ViewModel and default
state machine exist, that context becomes the child's parent
(`src/scripted/scripted_object.cpp:51-58`; `src/lua/lua_artboards.cpp:40-59`).

The adjacent golden refresh owner now calls `bind_view_model`, but constructs
with `RunnerScriptArtboard::new`, whose parent context is unconditionally
`None`, even though `refresh_bound_script_artboard_inputs` already holds the
live owner context (`tools/rust-golden-runner/src/main.rs:1191-1242`). The
child therefore receives a root local context rather than the pinned
local-plus-parent chain. This breaks parent/global DataBind paths during a
bound Artboard replacement and makes the receipt's claim that the sibling
refresh uses the same recovered sequence incomplete.

The refresh must create a `ScriptArtboardParentContext` from its retained owner
handle and use the same `new_with_parent_context`/default-state-machine/bind
sequence as rehydration. Its witness must resolve a property that exists only
through the parent chain; checking only `_data_context.is_some()` cannot
distinguish the incorrect root context.

### Accepted portions and review evidence

Independent source inspection accepts the production `FileScriptArtboard`
authority split: the consumer File creates the ViewModel, while the source
RuntimeFile resolves the cloned child's imported DataBinds. It also accepts
the refreshed live snapshot, mutable concrete `viewModelId`, default-state-
machine-before-replacement-bind order, no construction-time
`advance_data_context`, and the public empty-Live preflight on the prepared
hydration path.

At the reviewed tree, with `CARGO_INCREMENTAL=0`, the focused live-update
constructor guard, scalar/trigger/Artboard projection policy, concrete Luau
ScriptAsset guard, and golden rehydration/default-state-machine/frame-tail
witnesses all passed. They establish the accepted local properties but do not
enter any of the three rejected paths above.

## Correction after review `8449eee45`

Status: **PENDING — TWO FRESH INDEPENDENT REVIEWS REQUIRED.** This correction
addresses only the three rejected operative paths. It does not self-certify
the ScriptInput authority.

### The live backend guard now precedes resolver preparation

`apply_scripted_input_update` now asks the concrete ScriptInstance for
`script_artboard_input_context_live()` at the start of the Artboard branch,
before resolver lookup or `prepare_script_artboard`. A missing state/table/
ScriptAsset therefore returns `Ok(false)`: no typed preparation error escapes,
no deferred constructor authority is acquired, no setter/dirt path runs, and
the caller receives no table-projection success. The setter repeats the guard
before construction so a backend lifetime change between those two authored
boundaries remains inert.

`live_artboard_update_checks_script_asset_before_deferred_construction` now
uses a resolver whose `prepare_script_artboard` both increments a counter and
returns a typed resource error. Against an instance with a live general
lifetime but absent Artboard table/ScriptAsset context, the actual update path
returns false, the preparation count remains zero, and the setter/projection
trace remains empty.

### Golden rehydration now preserves the pinned two-loop transaction

`rehydrate_script_inputs_in_range` is the operative golden owner for both the
production rehydration call and its witnesses. Its first, allocation-free loop
validates every matching input without invoking the supplied application
owner. An Artboard must retain a representable, present referenced graph. A
ViewModel input must retain a DataContext, claimed path, and concrete
ViewModel-property cell. Only after every input passes does the second loop
invoke setters in authored order.

The validation API now preserves the pinned distinction between an unresolved
ViewModel property (`None`, whole hydration returns false) and a valid
ViewModel property whose selected child occurrence is currently null
(`Some(None)`, hydration succeeds without replacing the table field). The
caller skips `didHydrateScriptInputs`, layout/path-effect hydration completion,
and user init when the first loop returns false; a cold pending init stays
deferred. The authored application loop re-resolves the ViewModel property and
also returns false, rather than panicking or continuing, if an earlier authored
setter made that prerequisite unavailable after preflight.

`rehydration_preflight_is_atomic_for_late_artboard_and_unresolved_view_model`
enters this operative owner twice. A valid scalar followed by a missing
Artboard produces zero application callbacks, and an actual retained owner
context plus an unclaimed ViewModel path also produces zero application
callbacks. Neither former partial-publish/continue route remains.

The cold `hydrate_script_inputs` sibling was inspected separately. Pinned
`validateForColdScriptInit` returns true for Artboard and ViewModel inputs, and
the generated scalar/trigger inputs have no failing cold prerequisite, so
moving its authored construction/application work behind a new live-hydration
preflight would not reproduce an upstream gate.

### Bound-Artboard refresh now retains the owner as parent

`refresh_bound_script_artboard_inputs` now calls
`new_refreshed_runner_script_artboard`. That owner creates a
`ScriptArtboardParentContext` from the retained owning handle, constructs the
child through `new_with_parent_context`, creates its authored default state
machine, and only then binds the selected child ViewModel across the Artboard
and state machine. The unparented `RunnerScriptArtboard::new` bypass was
removed because no remaining caller had the authority to discard the parent.

`refreshed_script_artboard_preserves_parent_only_value_resolution` uses two
partial occurrences of the same ViewModel schema: the child local occurrence
contains only property zero, while the retained owner contains only property
one with value `35`. After the actual refresh constructor, the child's bound
DataContext resolves source path `[0, 1]` to `35` through its parent. A root-
only context returns no value for that path, so this witness distinguishes the
corrected chain from the rejected implementation.

Focused correction evidence, all with `CARGO_INCREMENTAL=0`:

- `live_artboard_update_checks_script_asset_before_deferred_construction`:
  passed;
- `rehydration_preflight_is_atomic_for_late_artboard_and_unresolved_view_model`:
  passed;
- `refreshed_script_artboard_preserves_parent_only_value_resolution`: passed;
- `rehydrated_script_artboard_binds_after_default_machine_and_keeps_host_frame_tail`:
  passed;
- complete `scripted_listener_action_tests`: 101 passed;
- complete golden-runner scripting unit suite: 17 passed;
- concrete Luau
  `artboard_setter_checks_script_asset_before_running_prepared_constructor`:
  passed.

Receipt status remains pending until two fresh reviewers independently inspect
the corrected production/golden call chains and the operative witnesses.

## First fresh independent review after `0c1e2b675`

Status: **REJECTED.** The correction closes the specific live
`apply_scripted_input_update` guard found by `8449eee45`, and the new golden
rehydration and refresh witnesses enter their advertised helpers. The combined
ScriptInput authority is still not certifiable because ViewModel prerequisite
validation remains bypassable at the public hydration boundary, while the
golden cold and bound-refresh owners still bypass pinned hydration/setter
guards.

### Public prepared hydration validates Artboards, not every input

Pinned `ScriptedObject::hydrateScriptInputs` validates every custom property
before Context-independent input publication. In particular,
`ScriptInputViewModelProperty::validateHydrationPrerequisites` distinguishes an
unresolved property (whole hydration fails) from a valid ViewModel-valued
property whose selected child is null (hydration succeeds without a table
write).

`ScriptListenerActionHydration::preflight_artboards` validates only Artboard
variants. It moves a public `ViewModel` variant and its resolver unchanged into
`PreparedScriptListenerActionHydration`; `apply` may then install Context and
publish earlier scalar/Artboard inputs before the resolver is called. The
ordinary facade builders perform a separate caller-owned ViewModel check, but
the public `new` plus `apply`/`apply_inputs` entry points do not make that check
structural. This is the same kind of bypass the Artboard type-state correction
was intended to remove.

The correction must preserve two distinct ViewModel states at preflight: an
invalid/unresolved property must reject the whole batch, while `Some(None)`
must remain a valid nullable child. Phase two must still re-resolve the valid
property at its authored position, because pinned C++ repeats that lookup and
may fail there if an earlier setter changed the prerequisite.

### Golden cold hydration still has no normal two-loop preflight

The correction adds a two-loop transaction only to
`rehydrate_script_inputs_in_range`. The operative cold
`hydrate_script_inputs` remains a single authored loop. A missing
`ScriptInputArtboard::artboardId` is silently skipped; an unavailable later
Artboard can fail construction only after earlier scalar setters; and the
caller proceeds to user `init` whenever the one-pass helper returns. Pinned
`validateForColdScriptInit == true` permits generator construction, but it does
not replace the normal `hydrateScriptInputs` first loop that immediately
follows and rejects a null `m_referencedArtboard` before any setter or user
`init`.

Thus a cold script with an earlier number input and a later absent/invalid
Artboard can publish the number and run `init` in the golden owner where pinned
C++ returns false with the script table otherwise untouched. The receipt's
claim that the cold sibling needs no live-hydration preflight conflates
`validateForColdScriptInit` with `validateHydrationPrerequisites`.

### Golden rehydrate and bound refresh still construct before the live setter guard

The corrected runtime update path now checks
`script_artboard_input_context_live` before resolver preparation, and the
concrete setter repeats the check before construction. The golden paths do not
share that invariant:

- `rehydrate_script_inputs` calls
  `new_rehydrated_runner_script_artboard` before
  `set_script_artboard_input_for_global`; and
- `refresh_bound_script_artboard_inputs` calls
  `new_refreshed_runner_script_artboard` before the same setter.

Neither helper first proves that the target scripted occurrence still has the
state/table/ScriptAsset authority required by pinned
`ScriptedObject::setArtboardInput`. An inert or disposed occurrence can
therefore allocate a child, construct its default state machine, create/bind a
ViewModel/DataContext, register render/frame ownership, or surface a resource
error before the eventual setter discovers the missing table. The new refresh
parent-chain witness proves the constructed child's context, but cannot
falsify this earlier guard bypass.

The golden owners need the same deferred prepared-recipe handoff used by the
runtime path, with the occurrence guard ahead of preparation/construction and
an inert result that skips dirt, completion hooks, and init.

### Accepted portions and evidence

Independent inspection accepts the corrected runtime live-update guard: an
absent Artboard table/ScriptAsset returns false before resolver preparation,
and the backend setter rechecks before construction. The golden live
rehydration first pass correctly distinguishes an unresolved ViewModel
property from a valid nullable child. The refreshed child now retains its
owning parent context, and the previously recovered source RuntimeFile,
consumer ViewModel, default-state-machine-before-bind, exactly-once bind, and
host-frame-tail/no-construction-advance sequence were not falsified on the
guarded production path.

At `0c1e2b675`, with `CARGO_INCREMENTAL=0`:

- complete `scripted_listener_action_tests`: 101 passed;
- concrete Luau
  `artboard_setter_checks_script_asset_before_running_prepared_constructor`:
  passed;
- complete golden-runner scripting unit suite: 17 passed;
- combined `nuxie-runtime`, `nuxie-scripting`, `nuxie`, `silver-corpus`, and
  golden-runner scripting checks: passed;
- source correspondence remained 456 applicable rows with zero pending;
- symbol correspondence remained 1,105 owners / 7,818 units, and its 33
  checker tests passed.

Those green tests accept the listed local corrections but have no witness for
an initially unresolved public ViewModel batch, cold invalid-Artboard
atomicity/init suppression, or an inert golden rehydrate/refresh target. The
receipt remains rejected and requires a new correction plus a fresh two-review
cycle.

## Correction after first post-`0c1e2b675` review

Status: **PENDING — TWO FRESH INDEPENDENT REVIEWS REQUIRED.** This correction
addresses only the three blockers identified by review `6823d7881`; it does
not self-certify ScriptInput authority.

### Public phase-one now validates every typed prerequisite

`ScriptListenerActionHydration::preflight_artboards` now resolves each
`ScriptInputViewModelProperty` once in phase one as well as validating every
Artboard. An error rejects the complete public batch before Context or any
authored input setter. `Ok(None)` remains the distinct valid-null-child state.
The resolved value is deliberately discarded and the resolver is retained in
the prepared type state so phase two repeats the lookup at the input's authored
position, after any earlier setter, exactly as pinned
`validateHydrationPrerequisites` plus `hydrateScriptInput` do.

The witnesses cover all three edges:

- `unresolved_view_model_preflight_precedes_public_apply_setters` enters the
  public `new`/`apply` boundary and proves a later unresolved ViewModel runs no
  Context or earlier scalar setter;
- `scripted_hydration_initially_unresolved_viewmodel_is_atomic` proves the
  state-machine facade likewise reaches no earlier input or user init;
- `scripted_hydration_accepts_valid_null_viewmodel_and_continues_to_init` and
  `scripted_hydration_resolves_artboard_then_viewmodel_in_authored_apply_order`
  prove nullable acceptance and two distinct phase-one/phase-two resolutions.

### Golden cold hydration now uses the same two-loop transaction

The operative `hydrate_script_inputs` now delegates to
`rehydrate_script_inputs_in_range`: the first loop validates every referenced
Artboard before the second loop publishes any scalar or constructs any child.
It returns the pinned hydration boolean to
`initialize_scripted_drawables_for_artboard`. A false result suppresses user
init and leaves `hydration_succeeded` false for no-init scripts too, so neither
ScriptedPathEffect `didHydrateScriptInputs` nor ScriptedLayout completion can
be published after a prerequisite miss.

`cold_hydration_preflight_blocks_earlier_setters_and_user_init` uses an earlier
number and later invalid Artboard through the operative cold helper and proves
zero setters and zero init calls.

### Golden rehydrate and refresh now guard deferred construction

`ArtboardInstance::set_prepared_script_artboard_input_for_global` owns a
non-bypassable handoff: it locates the occurrence, checks the concrete
state/table/ScriptAsset guard, and only then invokes the retained constructor,
sets the field, and publishes ScriptUpdate dirt. Its boolean result prevents a
bound refresh from publishing its resolved-id projection when the setter was
inert.

Both operative golden callers use this handoff. In addition,
`rehydrate_script_inputs` checks the guard before its entire validation/apply
transaction, matching pinned `hydrateScriptInputs`' initial state/m_self
guard, while `refresh_bound_script_artboard_inputs` checks it before bound
resolution and construction.

The sibling audit found the same pre-guard construction in the public
`nuxie::rehydrate_bound_script_inputs` facade path. It now checks the concrete
occurrence before bound resolution and hands the child constructor to
`set_prepared_resolved_script_artboard_input_for_global`, which publishes the
resolved-id projection only after the guarded setter succeeds. The remaining
direct constructor call sites are cold construction or test-only witnesses;
no live rehydrate/refresh caller constructs before this guard.

`golden_rehydrate_and_refresh_guard_precedes_construction_dirt_and_projection`
drives both operative golden functions against an inert occurrence. A
test-only counter at the two child constructors remains zero; rehydrate returns
false; and refresh publishes neither ScriptUpdate dirt nor a resolved-id
projection.

`bound_artboard_rehydrate_guards_before_construction_dirt_and_projection`
drives both mounted-live and pre-advance phases of the public facade sibling.
Its `FileScriptArtboard` counter remains zero; both phases return false; and
the occurrence publishes neither ScriptUpdate dirt nor a resolved-id
projection.

### Correction evidence

With `CARGO_INCREMENTAL=0`:

- complete `scripted_listener_action_tests`: 102 passed;
- public hydration atomicity tests: 4 passed;
- complete golden-runner scripting unit suite: 19 passed;
- focused public-facade bound-Artboard operative witness: passed;
- combined `nuxie-runtime`, `nuxie-scripting`, scripting-enabled `nuxie`,
  `silver-corpus`, and golden-runner checks: passed;
- source correspondence: 456 applicable rows, zero pending;
- symbol correspondence: 1,105 owners / 7,818 authority units;
- symbol-checker unit suite: 33 passed.

This evidence only establishes that the rejected blockers have a scoped
correction and falsifying witnesses. The ScriptInput receipt remains pending
until two fresh independent reviewers inspect the corrected production and
golden paths from the correction commit.

## First fresh independent review after `edb32c9ec`

Status: **REJECTED.** The correction closes the three blockers recorded by
review `6823d7881`: public ViewModel prerequisites now cross the phase-one
type-state boundary, golden cold hydration is a two-loop transaction, and the
three live rehydrate/refresh owners added in the correction defer Artboard
construction until after their concrete occurrence guard. A complete sibling
entry-point census found two remaining operative parity failures outside those
focused witnesses.

### Accepted correction edges

The public `ScriptListenerActionHydration::apply` and `apply_inputs` paths now
preflight both Artboard and ViewModel prerequisites. An unresolved ViewModel
rejects before Context or an earlier scalar setter, a valid nullable child
continues without a table write, and phase two resolves the property again in
authored order. The state-machine witness enters the real hydration/init owner,
not only the helper.

The golden cold, live rehydrate, and bound-refresh helpers now share a real
first validation loop. Failed cold Artboard/ViewModel prerequisites suppress
all setters, user init, `didHydrateScriptInputs`, ScriptedPathEffect completion,
and ScriptedLayout completion. Both golden live constructors and the public
`nuxie` bound-Artboard constructor are handed to a setter-owned closure after
the table/ScriptAsset guard; their inert witnesses observe zero construction,
zero ScriptUpdate dirt, and zero resolved-id projection.

The previously accepted ownership remains intact on the reviewed paths:
Artboard construction uses the source runtime for the cloned source objects,
the consumer File/ViewModel for the scripting facade, the default state
machine is constructed before the selected ViewModel bind, binding occurs
exactly once, and construction does not call `advance_data_context`; the
registered host frame tail still owns that advance. Direct live updates also
retain the refreshed bindable snapshot and guard before resolver preparation.

### Retry owners prepare hydration before recreating the script table

Pinned `Artboard::initScriptedObjects` and
`StateMachineInstance::initScriptedObjects` call
`ScriptAsset::initScriptedObject` when user init is not complete. That calls
`ensureScriptInitialized`, which recreates `m_self`/Context (or exits inert on
generator failure). Only after that generator/table boundary returns do the
owners call `hydrateScriptInputs`; that method immediately checks `state()` and
`m_self` before validating any input prerequisite.

Every Rust reinit sibling reverses the generator and hydration-preparation
parts of this order. For the main state-machine owner,
`hydrate_and_initialize_scripted_object_instance` and its
`after_context_install` sibling call `prepare_hydration` and
`preflight_artboards` before `prepare_init_retry`. The state-machine DataBind
converter, listener-owned converter, DataConverterGroup, and
ScriptedInterpolator owners repeat the same shape. `preflight_artboards` is
not allocation-free validation: it calls each Artboard resolver's
`prepare_script_artboard` and calls each ViewModel resolver once. Those calls
can acquire a prepared recipe or return a typed resource/prerequisite error
before the failed/deferred occurrence's generator has recreated its table.

This is observable in both directions. A generator retry that would fail or
remain inert can currently surface a later input resolver error first. A
successful retry whose generator changes context-dependent state validates
against the old pre-recreation state rather than the new table lifetime.
Neither behavior matches the pinned generator-then-state-guard-then-preflight
sequence. The new public atomicity tests use an already-live probe; the
init-retry tests do not combine an invalid lifetime with a fallible Artboard or
ViewModel resolver, so none enters this ordering edge.

The correction needs one shared lifecycle boundary, used by all five sibling
owners, that installs the already-resolved DataContext barrier, recreates the
generator/table when required, checks the concrete state/table/ScriptAsset
lifetime, and only then performs the two-loop prerequisite/apply transaction.
An inert or failed recreation must not prepare a recipe, resolve a ViewModel,
publish an error from those later stages, set input/dirt/result state, call
user init, or publish a hydration-completion hook.

### Public nested-tree refresh loses both source File and owner DataContext

`nuxie::rehydrate_bound_script_input_tree` visits every nested Artboard
occurrence, but identifies each child graph by looking up its numeric
`graph_global_id` in the root/consumer `File`. It then passes the same root
`RuntimeOwnedViewModelHandle` to every child. The callee resolves primitive and
Artboard input DataBinds against that root handle and constructs the child
Artboard input with a parent context made from that root.

Pinned nested Artboards do not inherit those authorities by numeric identity.
The concrete nested occurrence retains its source File, and its scripted
object observes the nested Artboard's local DataContext plus its parent chain.
Consequently the current public path has two reachable failures:

- a cross-File nested source whose graph id is absent from the root File
  errors before reaching the source occurrence; a colliding id selects the
  wrong root-File graph and wrong ScriptInput catalogue;
- even for a same-File nested graph, a relative/local ScriptInput binding is
  resolved from the root ViewModel, and the newly projected Artboard facade
  receives a root-only parent context instead of the nested owner's
  local-plus-parent chain.

The golden refresh correction demonstrates the right principle for one
sibling by accepting the retained owner handle and proving a parent-only path.
The public `nuxie` guard witness added by `edb32c9ec` contains only a root graph
and an inert table, so it proves deferred construction but cannot distinguish
the wrong nested File, wrong local source, or wrong parent chain. Exact parity
requires tree traversal to carry each concrete child's source File and
occurrence-owned DataContext into primitive resolution, Artboard resolution,
and deferred facade construction; numeric graph lookup in the root File is not
a valid substitute.

### Independent evidence

All commands below used `CARGO_INCREMENTAL=0` where Cargo was involved:

- the four focused public/state-machine ViewModel atomicity and authored-order
  witnesses passed;
- golden cold atomicity, golden inert rehydrate/refresh, golden parent-only
  refresh, and the public `nuxie` inert bound-Artboard witness passed;
- combined checks for `nuxie-runtime`, `nuxie-scripting`, scripting-enabled
  `nuxie`, `silver-corpus`, and scripting-enabled `rust-golden-runner` passed;
- source correspondence remained 456 applicable rows with zero pending;
- symbol correspondence remained 1,105 owners / 7,818 authority units with
  generated authority replayed, and all 33 checker tests passed.

Those green results accept the scoped correction but contain no operative
witness for either rejected edge. The 42 out-of-line plus 11 executable inline
ScriptInput census remains sound; complete certification is rejected until the
shared retry ordering and nested source/context ownership are corrected, then
reviewed by two fresh independent auditors.

## Correction after review `a62bc405a`

Status: **PENDING — TWO FRESH INDEPENDENT REVIEWS REQUIRED.** This correction
addresses only the two blockers identified by review `a62bc405a`; it does not
self-certify ScriptInput authority.

### All five reinit owners now share the pinned lifetime boundary

The StateMachine, StateMachine after-context-install, DataConverterGroup,
listener-owned converter, and ScriptedInterpolator owners now enter one shared
boundary. It installs only the already-selected DataContext, calls
`prepare_init_retry` to recreate the generator/table, and checks the concrete
script lifetime before returning to its caller. Only a live caller then
acquires or validates Artboard/ViewModel recipes and performs the two-loop
apply/hydrate transaction. An inert recreation cannot consult a typed input
resolver or surface an error owned by the old table lifetime.

`every_retry_owner_uses_the_shared_recreate_then_guard_boundary` is a complete
five-owner census backed by an actual ScriptInstance trace. It proves the
shared boundary observes Context, generator recreation, and the lifetime guard
in that order and proves every production owner calls that boundary before its
fallible hydration preparation.

### Nested rehydrate now carries occurrence authority

Every concrete nested occurrence retains its source `File` authority across
instantiation, cloning, and replacement. `rehydrate_bound_script_input_tree`
recovers that source, selects the graph and ScriptInput catalogue from it,
selects the consumer ViewModel from the occurrence, and resolves bindings from
the occurrence's complete local-plus-parent `RuntimeOwnedDataContext`. Deferred
Artboard construction likewise receives the nested occurrence's parent chain
instead of a root-only substitute.

The sibling scripted mount-tree walker was audited and had the same root-File
assumption. Its groups now carry the concrete source `File` through target
lookup, hydration, and lazy interpolator attachment; a missing source runtime
is reported instead of silently consulting a colliding root catalogue.

The operative witnesses cover both rejected dimensions:

- `nested_tree_rehydrate_uses_cross_file_source_catalog_and_occurrence_context`
  mounts a nested occurrence whose source and consumer Files have colliding
  graph ids but distinct ScriptInput manifests, then proves the source manifest
  and nested occurrence context drive the live table;
- `nested_script_input_resolution_uses_local_then_parent_data_context` proves
  a local property wins and a missing local property falls back through the
  retained parent chain;
- `live_scripted_artboard_uses_consumer_file_and_retains_concrete_cross_file_source`
  proves the concrete source File remains retained independently of the
  consumer File.

### Correction evidence

All Cargo commands below used `CARGO_INCREMENTAL=0`:

- the five-owner retry ordering census passed;
- the cross-File distinct-manifest, local-plus-parent, and concrete source
  retention witnesses passed;
- debug and release `nuxie-runtime` lib-test compilation passed;
- scripting-enabled `nuxie` lib-test compilation passed;
- combined checks for `nuxie-runtime`, `nuxie-scripting`, `silver-corpus`, and
  scripting-enabled `rust-golden-runner` passed;
- source correspondence remained 456 applicable rows with zero pending;
- symbol correspondence remained 1,105 owners / 7,818 authority units with
  generated authority replayed, and all 33 checker tests passed.

This evidence establishes only a scoped correction and falsifying witnesses.
The ScriptInput receipt remains pending until two fresh independent reviewers
inspect the correction commit and independently accept both previously
rejected edges.

## First fresh independent review after `4c8daea99`

Status: **REJECTED.** The complete five-owner retry census and the ordinary
nested-Artboard rehydrate correction are sound, but the public mount-tree
sibling still has two reachable source-authority failures. Neither current
cross-File witness enters that mount path, and neither covers a component-list
child created after the facade File was attached.

### Accepted correction edges

All five retry owners were traced independently from their public facade call
sites through the retained `ScriptInstance`: the StateMachine owner, its
after-context-install sibling, state-machine DataBind converter,
listener-owned converter, and ScriptedInterpolator converter all call
`install_context_recreate_and_guard_script_lifetime` before invoking their
fallible hydration factory. The shared boundary installs only the selected
Context chain, recreates the generator/table, checks the concrete table
lifetime, and returns inert before Artboard or ViewModel preparation. The
focused trace observes `context -> recreate -> guard`, and the source census
finds exactly five shared-boundary calls and no direct retry calls in the
three owner files.

The ordinary nested-Artboard live rehydrate path is also corrected. It
recovers the concrete child's source File, selects the graph and ScriptInput
catalogue from that File, selects the child occurrence's ViewModel/DataContext,
and preserves local-then-parent resolution. The distinct-manifest cross-File
witness, the local-plus-parent witness, and the separate source-File retention
witness all pass. The consumer/source split used by `FileScriptArtboard`
remains correct, as do deferred live snapshotting, mutable `viewModelId`,
default-state-machine-before-bind, exactly-once binding, and File-owned frame
tail advancement. The reviewed public, runtime, and golden cold/live/refresh
guards retain a validation loop before authored setters and construct no
Artboard behind an inert table.

### A cold cross-File child cannot bootstrap its own script runtime

The corrected mount walker carries `ScriptMountGroup.file`, but entry and VM
preparation are still rooted in the outer File:

- `mount_scripted_artboard_tree` and its async sibling return before tree
  collection when the root File has no authenticated executable ScriptAsset;
- `collect_script_mount_groups` independently returns an empty collection when
  the root File is unauthenticated, without inspecting a retained child source
  File;
- if the root happens to have scripts and collection reaches the child,
  `instantiate_script_mounts` requires every non-root `group.file` to already
  have `scripts.ready`; `prepare_mounts` prepares only the root File candidate.

A public cold clone intentionally has no mounted script tables. Mounting that
clone as a cross-File nested child of a nonscripted consumer therefore reports
no script target at all; putting an unrelated script in the consumer only
changes the failure to "source File has no prepared scripting runtime" unless
the source File happened to have been initialized elsewhere. Pinned ownership
is per concrete source File and does not depend on unrelated root assets or on
process history. Exact correction requires the mount transaction to discover
and prepare every distinct retained source File before attaching any group.

The existing
`nested_tree_rehydrate_uses_cross_file_source_catalog_and_occurrence_context`
witness attaches a probe table directly and invokes rehydration. It proves the
post-mount catalogue/context path, but cannot falsify this cold mount failure.

### Dynamically created component-list children lose File authority

`OwnedArtboardInstance::instantiate_default` attaches source-File authority to
the occurrence tree that exists at construction time, and ordinary nested
replacement preserves a non-null child authority. A component-list row is a
different creation owner. `create_component_list_item_instance` constructs its
child later with `ArtboardInstance::from_graph_inner`; that constructor sets
`script_source_file_authority` to `None`, and the row owner never inherits the
parent's authority before publication or pool restoration.

Both corrected public walkers visit component-list children. Their visitor
unconditionally calls `script_source_file_authority::<Arc<File>>()` before it
can determine whether the row has a scripted target. Consequently the first
mount or bound-input refresh after a row is materialized fails with "no
retained source File", even for a same-File row with no scripts. A cross-File
row additionally loses the exact catalogue the correction intended to retain.
The current nested witness exercises only `NestedArtboard` replacement, not
the `ArtboardComponentList` creation/pool owner.

### Independent evidence

All Cargo commands used `CARGO_INCREMENTAL=0` and ran from a clean detached
worktree at `4c8daea99`:

- the complete five-owner retry trace/census passed;
- the operative ordinary nested distinct-manifest, local-plus-parent, retained
  source-File, and cross-File child-DataBind witnesses passed;
- golden default-state-machine/frame-tail, two-loop Artboard/ViewModel
  preflight, cold atomicity, inert rehydrate/refresh, and parent-context refresh
  witnesses passed;
- combined checks for `nuxie-runtime`, `nuxie-scripting`, scripting-enabled
  `nuxie`, `silver-corpus`, and scripting-enabled `rust-golden-runner` passed;
- source correspondence remained 456 applicable rows with zero pending;
- symbol correspondence remained 1,105 owners / 7,818 authority units with
  generated authority replayed, and all 33 checker tests passed.

Those green witnesses accept the corrected retry and ordinary nested
rehydration edges. Complete ScriptInput certification remains rejected until
mount preparation is source-File-complete and every late-created component-list
child inherits its exact File authority, with operative cold-mount and dynamic
row witnesses, followed by two fresh independent reviews.

## Correction after review `5b8f4276a`

Status: **PENDING — TWO FRESH INDEPENDENT REVIEWS REQUIRED.** This correction
addresses only the two mount-tree blockers found by the first post-`4c8daea99`
review and does not self-certify the ScriptInput owners.

### Cold mount preparation is complete for every source File

Mount collection no longer gates the complete occurrence tree on the root
File's script authentication or asset catalogue. Each group is adjudicated by
its own retained source File. Before the first generator runs, the mount
transaction discovers every distinct target-bearing File in occurrence order,
validates its renderer-factory domain, and either pins its existing runtime or
builds a cold candidate from that File's own modules, protocols, assets, and
runtime catalogue. Only after every participating runtime is ready does table
generation, ScriptInput hydration, and authored init begin.

Cold candidates remain owned by the transaction while all groups instantiate.
They are published to their respective Files only after topology validation;
then the already-prepared tables are attached in original tree order. A later
File's registration/generator/hydration failure therefore cannot attach an
earlier group or leave only part of the tree mounted. The detached async owner
uses the same multi-File preparation set and commit boundary.

`cold_cross_file_child_mount_prepares_its_source_runtime` is an operative
public-facade witness. It places a cold authenticated
`script_artboard_test.riv` Artboard beneath a scriptless consumer File, calls
the real mount transaction, and proves the source runtime moves cold-to-ready,
the unrelated consumer remains cold, and every nested scripted target receives
its generated and hydrated table. This failed at the root-File early return
before the correction.

### Late component-list rows retain occurrence authority

`ArtboardComponentList::create_component_list_item_instance` now copies the
parent occurrence's exact type-erased File authority into the child immediately
after `from_graph_inner`, before DataContext binding or publication. The
identity-reuse path repairs a missing authority before rebinding; pool reuse
otherwise retains the child-owned pin. The existing recursive facade attach
continues to cover rows that already exist when a root File is installed.

The mount and rehydrate walkers now distinguish an authority-requiring child
from a genuinely un-scripted child before reporting a missing File. A child
with ScriptedDrawable, scripted converter, scripted interpolator, or mounted
script state still fails closed without authority. A genuinely un-scripted
child contributes an empty topology group (mount) or is skipped (rehydrate),
so traversal does not invent a File lookup merely to prove that no scripted
work exists.

The dynamic-row witnesses exercise both sides of that boundary:

- `component_list_mount_settles_context_without_advancing_the_row_state_machine`
  now proves a late-created un-scripted row inherits authority, retains it
  through context refresh, and keeps it through removal/pool remount;
- `scripted_component_list_row_retains_source_authority_through_refresh_and_pool_reuse`
  creates an authored ScriptedDrawable row and proves the authority-required
  classification plus cold creation, refresh reconstruction, and pool reuse.

The ordinary nested replacement and clone owners were re-audited: replacement
still preserves a concrete child's non-null source authority and only inherits
the parent authority for a source-less same-File child; clone and recursive
facade attachment continue to preserve already-materialized rows.

### Correction evidence

All Cargo commands used `CARGO_INCREMENTAL=0`:

- cold cross-File source-runtime mount: 1 passed;
- un-scripted and scripted component-list creation/refresh/pool witnesses:
  2 passed;
- same-File authored Artboard-input mount, ordinary cross-File nested
  rehydrate, existing File runtime/domain preparation, and two detached async
  mount-preparation witnesses: 5 passed;
- combined checks for `nuxie-runtime`, `nuxie-scripting`, scripting-enabled
  `nuxie`, `silver-corpus`, and scripting-enabled `rust-golden-runner`: passed;
- the complete scripting-enabled `nuxie` lib suite reached 61 passed / 1
  ignored; its sole failure is the pre-existing host-log wording assertion
  expecting `bytecode version mismatch` while the VM now reports the more
  precise supported-range diagnostic, outside this correction;
- source correspondence remains 456 applicable rows with zero pending;
- symbol correspondence remains 1,105 owners / 7,818 authority units with
  generated authority replayed, and all 33 checker tests passed.

The receipt remains pending until two fresh independent auditors inspect this
correction commit and independently accept both rejected mount-tree edges.

## First fresh independent review after `b6fdd88ea`

Status: **REJECTED.** The multi-File runtime preparation correction and the
late component-list File-authority correction close the two blockers recorded
by review `5b8f4276a`. A complete mount/hydration census found one remaining
operative ownership failure: mount planning preserves each occurrence's source
File, but erases that occurrence's DataContext before ScriptInput hydration.

### Accepted correction edges

The synchronous, asynchronous, and detached mount owners now collect the
complete occurrence tree even when the root File is scriptless. Each distinct
target-bearing source File is authenticated and prepared from its own runtime,
ScriptAsset catalogue, modules, protocols, assets, and renderer-factory domain
before the first generator is run. Cold candidates remain transaction-owned
through generator, two-pass ScriptInput hydration, authored init, and topology
validation; they are published only immediately before the infallible table
attachment pass. The operative cold cross-File witness enters the real public
mount transaction and proves that a scriptless consumer remains cold while the
retained child source File becomes ready and its concrete targets are mounted.

Late-created component-list rows now inherit their parent's exact type-erased
File authority before DataContext binding or publication. Identity refresh
repairs a missing authority, and refresh reconstruction plus pool removal/reuse
retain it. Un-scripted children without authority are still skipped, while a
ScriptedDrawable, scripted converter/interpolator, or existing table fails
closed without its File. Ordinary nested replacement continues to preserve a
concrete cross-File child's non-null source authority.

The wider ScriptInput ownership remains intact on the audited siblings. All
five retry owners still recreate the generator/table and pass the concrete
lifetime guard before fallible prerequisite preparation. Cold, live,
rehydrate, and refresh paths retain their whole-object validation pass,
authored apply order, consumer/source split, default-state-machine-before-bind,
exactly-once binding, host-owned frame-tail advance, dirt, completion, and
error behavior. In particular, the already-corrected live rehydrate walker
uses each nested occurrence's local-then-parent context.

### Mount groups discard occurrence-owned DataContext

Pinned `Artboard::internalDataContext` installs the concrete Artboard's
DataContext on every scripted object and immediately calls
`initScriptedObjects`; `ScriptedObject::hydrateScriptInputs` therefore resolves
both prerequisite and authored hydration against that exact occurrence. A
component-list row first binds its row ViewModel with the containing Artboard's
DataContext as parent, then the child's scripted objects hydrate from that
local-plus-parent chain (`artboard.cpp:2581-2603`,
`artboard_component_list.cpp:1528-1543`, and
`scripted_object.cpp:399-435`).

The Rust mount walker does retain the concrete `nested` while collecting each
group, but `ScriptMountGroup` stores only `file`, path, graph id, and targets.
Both `instantiate_script_mounts` and its async twin consequently pass the one
outer `root_view_model` to every group's
`hydrate_prepared_scripted_object_inputs`. That helper builds a root-only
parent context and resolves primitive, Artboard, and ViewModel inputs directly
from the outer handle. It never consults the nested occurrence's
`owned_data_context` or its selected local ViewModel.

This is reachable in both corrected creation owners:

- a cold same-File or cross-File nested ScriptedDrawable whose relative input
  should read its own local ViewModel instead reads the root; an absent local
  value cannot fall back through the retained parent chain, while a colliding
  root value silently hydrates the wrong value;
- a dynamically created component-list ScriptedDrawable row correctly retains
  its File after this correction, but its first real mount still hydrates from
  the outer Artboard's root instead of the row-local ViewModel plus parent;
- a nested `ScriptInputArtboard` receives a root-only parent context, so the
  constructed facade's relative parent traversal also differs from the pinned
  concrete occurrence.

The cold cross-File witness has no bound ScriptInputs, so it proves source VM
preparation and table attachment but cannot observe this loss. The existing
local-then-parent and distinct-manifest witnesses enter the separate live
rehydrate path, not the cold mount path. The two dynamic-row witnesses prove
authority identity and pool retention but never mount a real scripted row.

Exact correction requires each collected group to retain the concrete
occurrence's selected ViewModel and complete local-plus-parent DataContext
snapshot and to use that authority for both validation and authored hydration
in the synchronous and asynchronous mount transactions. The source runtime and
ScriptInput catalogue must remain `group.file`; the occurrence context must
not be reconstructed from the outer root. Operative witnesses must enter real
sync and detached-async mounts for nested and component-list scripted children,
with local/root collisions and parent fallback, rather than attaching a probe
table or calling the hydration helper directly.

### Independent evidence

All Cargo commands used `CARGO_INCREMENTAL=0`:

- the complete 102-test scripted-listener/ScriptInput suite passed;
- the complete 19-test scripting-enabled golden-runner unit suite passed;
- the five-owner retry trace, ordinary nested distinct-manifest rehydrate,
  local-then-parent rehydrate, consumer/source split, cold cross-File mount,
  and both component-list authority/pool witnesses passed;
- combined checks for `nuxie-runtime`, `nuxie-scripting`, scripting-enabled
  `nuxie`, `silver-corpus`, and scripting-enabled `rust-golden-runner` passed;
- source correspondence remained 456 applicable rows with zero pending;
- symbol correspondence remained 1,105 owners / 7,818 authority units with
  generated authority replayed, and all 33 checker tests passed.

Those green results accept the scoped File/runtime/authority correction but
contain no real mount witness with occurrence-local ScriptInputs. Complete
ScriptInput authority remains rejected until cold nested and dynamic-row mount
hydration carries the exact occurrence DataContext, followed by two fresh
independent reviews.
