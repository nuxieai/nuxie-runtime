# Script-input source certification

Pinned upstream: `4ac7b32798da0482e441ef09304dc3b480ed3ee5`

Implementing auditor: root campaign lane

Adversarial review: **correction implemented; independent re-review pending**

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
| `ScriptInputArtboard::initScriptedValue` | artboard branch of `prepare_script_listener_hydration`; `ScriptListenerActionHydration::apply_inputs` | exact | authored artboard hydration tests |
| `ScriptInputArtboard::validateForScriptInit` (inline) | resolved-reference preflight | exact | typed-artboard validation failure tests |
| `ScriptInputArtboard::validateForColdScriptInit` | cold phase accepts the unresolved live context | exact | cold/live hydration lifecycle tests |
| `ScriptInputArtboard::validateHydrationPrerequisites` | snapshot/reference preflight before any writes | exact | `scripted_hydration_validation_failure_applies_no_inputs_or_init` |
| `ScriptInputArtboard::hydrateScriptInput` | authored phase-two artboard resolution and projection | exact | authored artboard hydration tests |
| `ScriptInputArtboard::syncReferencedArtboard` | `set_artboard_input_core`; live `apply_scripted_input_update` | exact | artboard projection tests |
| `ScriptInputArtboard::onAddedClean` | ordered input/binding-definition construction | exact | imported input order/count comparison |
| `ScriptInputArtboard::clone` | `RuntimeScriptInputArtboardOccurrence::clone_for_scripted_object` | corrected; pending review | `fresh_clone_preserves_the_exact_live_bindable_identity` |
| `ScriptInputArtboard::file` (inline) | `RuntimeScriptInputArtboardOccurrence::file_attached` | exact | resolved/unresolved clone authority test |
| `ScriptInputArtboard::artboardIdChanged` | `RuntimeScriptInputProperties::apply_target`; `apply_artboard_id_changed` | exact | missing-id clear and generated-id separation tests |
| `ScriptInputArtboard::updateArtboard` | `RuntimeScriptInputProperties::apply_artboard_source`; `RuntimeScriptInputArtboardOccurrence::apply_artboard_source` | corrected; pending review | `live_asset_is_preferred_and_preserves_generated_artboard_id`; `ancestor_live_asset_is_rejected_then_absent_asset_uses_file_index` |
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

## Adversarial correction

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

## Result

The seven files contribute 42 out-of-line definitions and 11 executable inline
methods. All 53 have identified Rust owners. Seven destructor rows use the
approved Rust ownership adaptation and 44 rows retain their prior accepted
verdict. The two formerly rejected Artboard rows are corrected in this change
with focused behavioral evidence, but remain explicitly pending an independent
adversarial acceptance pass across every ScriptedObject owner path; this
implementing lane does not self-certify them.

The historical scalar RB4 gap is stale: scalar Core values,
occurrence-local scalar cloning, converter ownership, callback projection, and
file-index source/target reconciliation are live and directly exercised. That
does not close the retained Artboard-pointer branch described above.
