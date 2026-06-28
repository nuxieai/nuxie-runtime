# Schema Generation Prototype

Ticket: `#4`

Question: can Rive's `dev/defs` files generate Rust metadata for type keys, property keys, runtime inheritance, and property deserialization without mirroring the C++ class hierarchy?

Prototype command:

```sh
make schema
```

Expected output:

```text
crates/rive-schema/src/generated/schema.rs
```

`make schema` also runs `cargo fmt --all`, so the checked-in generated file matches the formatted codegen output.

Verdict: the shape works as the first durable slice. `rive-codegen` reads the C++ runtime definitions and generates:

- `ObjectKind` variants for every runtime definition.
- `Definition` metadata with type keys, runtime parent names, raw parent files, mixins, generic/generic-passthrough targets, export-with-context flags, abstract/cloneable flags, and ancestor names for `is_a` checks.
- Local `Property` metadata with primary and alternate property keys, declared/runtime field kind, descriptions, explicit and runtime initial values, C++ generator flags such as group, nullable, override-get/set, virtual, pure-virtual, editor-only, coop, tools-only, bindable and animates flags, raw annotations such as computed, journal, parentable, records, conditional runtime export, passthrough flags, bitmask passthrough metadata, whether the property should deserialize/store a field, and helper predicates for the generated C++ value-accessor and `Changed()` hook surface.
- C++-style validation before generation for duplicate definition keys, duplicate property keys, reserved property keys below 3, uint16 runtime key fit, and bitmask passthrough metadata. Bitmask passthrough fields must target a same-definition uint mask, bool bit fields must be width 1, uint bit fields must declare a width, bit ranges must fit inside 32 bits, and ranges for the same mask must not overlap.
- `Property::stored_field_initializer`, exposing the typed effective member initializer C++ would generate for stored fields, including implicit runtime defaults and runtime overrides such as missing IDs.
- `Property::cpp_generates_value_setter_body`, `cpp_setter_uses_stored_field`, `cpp_setter_uses_passthrough`, `cpp_generates_stored_field_getter_body`, `cpp_generates_passthrough_getter_declaration`, `cpp_generates_pure_virtual_value_setter_declaration`, `cpp_generates_encoded_decode_hook`, `cpp_generates_encoded_copy_hook`, `cpp_generates_changed_hook`, and `cpp_bitmask_passthrough_*_constant`, separating binary storage from the generated accessor/change-hook/declaration/bitmask-constant surface needed by a future Rust object layer.
- `is_callback_property_key`, matching C++ `CoreRegistry::isCallback` for callback-only property keys that are not field-fallback entries.
- `object_supports_property`, matching C++ `CoreRegistry::objectSupportsProperty`: inherited and alternate keys are supported, while encoded bytes payload fields remain deserializable but are excluded from the object-support/settable surface.
- `core_registry_setter_field_kind_by_property_key` and `core_registry_getter_field_kind_by_property_key`, matching the generated C++ `CoreRegistry::set*` and `get*` switch families. Getters intentionally exclude callbacks, encoded bytes payloads, and the semantic boolean bitmask passthrough fields that C++ can set but does not expose through `getBool`.

Generated count from the current C++ runtime: 336 runtime definitions and 588 runtime properties.

Test coverage added:

- `crates/rive-schema/tests/generated_schema.rs` checks generated metadata counts, object-kind/type-key lookup, runtime inheritance lookup, alternate property keys, C++ generator metadata flags, stored-field initializers, callback keys, object-support checks, setter/getter family checks, and passthrough property metadata.
- `crates/rive-schema/tests/cpp_generated_headers.rs` compares every Rust runtime definition's `typeKey`, property keys, alternate property keys, stored-field member presence, effective stored-field initializers, generated value-setter body presence and stored-field/passthrough body shape, pure-virtual value setter declarations, generated stored-field getter bodies and passthrough getter declarations, encoded `decode*`/`copy*` declarations, generated bitmask passthrough `Bitmask`/`BitOffset`/`FieldMask` constants, generated `Changed()` hook presence, generated `copy(...)` stored-member assignments, encoded-property copy hooks, and parent delegation, C++ `isTypeOf` ancestry switch, C++ `deserialize` switch entries, `CoreRegistry::makeCoreInstance` constructibility, generated `clone()` declaration presence and `src/generated` clone body shape, `CoreRegistry::propertyFieldId` fallback family, `CoreRegistry::set*`/`get*` switch families, `CoreRegistry::isCallback` callback-key table, and `CoreRegistry::objectSupportsProperty` support table against the generated C++ headers. Set `RIVE_RUNTIME_DIR` to override the default reference runtime path.
- `tools/rive-codegen/tests/generated_schema.rs` regenerates the schema from the current C++ `dev/defs`, formats it with the workspace Rust edition, byte-compares it with the checked-in `crates/rive-schema/src/generated/schema.rs`, separately parses the raw JSON to lock the runtime definition/property counts, declared/runtime field-type surface, definition mixin/generic/export-context metadata, encoded payloads, alternate keys, descriptions, bindable and animates counts, C++ generator property flags, raw annotation counts, passthrough/bitmask metadata, and runtime initializer overrides, and feeds synthetic invalid key and bitmask passthrough definitions through `rive-codegen` to prove they are rejected.

Known limit: this does not yet generate concrete runtime object structs or setter bodies. That belongs in the `rive-core` slice after binary import proves the metadata is sufficient.

Ticket `#5` should consume `rive-schema::generated::DEFINITIONS` to create objects by type key and dispatch property keys while decoding `.riv` files.
