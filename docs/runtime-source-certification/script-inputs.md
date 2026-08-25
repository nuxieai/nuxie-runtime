# Script-input source certification

Pinned upstream: `4ac7b32798da0482e441ef09304dc3b480ed3ee5`

Implementing auditor: root campaign lane

Adversarial review: pending

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
| `ScriptInputArtboard::clone` | `RuntimeScriptInputArtboardOccurrence::clone_for_scripted_object` | exact | `fresh_clone_copies_file_authority_only_with_a_resolved_reference` |
| `ScriptInputArtboard::file` (inline) | `RuntimeScriptInputArtboardOccurrence::file_attached` | exact | resolved/unresolved clone authority test |
| `ScriptInputArtboard::artboardIdChanged` | `RuntimeScriptInputProperties::apply_target`; `apply_artboard_id_changed` | exact | missing-id clear and generated-id separation tests |
| `ScriptInputArtboard::updateArtboard` | `RuntimeScriptInputProperties::apply_artboard_source`; `apply_artboard_source` | exact | live source failure-preservation and repeated-id projection tests |
| `ScriptInputArtboard::referencedArtboardId` | `RuntimeScriptInputArtboardOccurrence::referenced_artboard_id` | exact | input snapshot and clone tests |

The generated `artboardId` and retained referenced Artboard remain separate.
A generated-field write clears an unresolved retained pointer when File
authority exists; a ViewModel update preserves the old pointer on failed
lookup and does not rewrite the generated id.

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

## Result

The seven files contribute 42 out-of-line definitions in the initial census
and 11 additional executable inline methods in their handwritten headers. All
53 have concrete Rust owners. Five destructor rows use the approved Rust
ownership adaptation; the remaining 48 preserve the supported observable
contract exactly. No production change was required in this pass.

The historical RB4 gap is stale for these owners: scalar Core values,
occurrence-local cloning, converter ownership, callback projection, and public
source/target reconciliation are live and directly exercised. Independent
adversarial review and the repaired header-aware denominator remain required
before certification.
