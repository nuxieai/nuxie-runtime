# Script-input source certification

Pinned upstream: `4ac7b32798da0482e441ef09304dc3b480ed3ee5`

Implementing auditor: root campaign lane

Adversarial review: **PENDING A FRESH TWO-REVIEW CYCLE AFTER THE SECOND CORRECTION**

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
