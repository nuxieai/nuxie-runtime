# B-6 Structural Audit — binary-core

Pinned C++: d788e8ec. Crate-wide compensation-family sweep: generation, dirty-container fields, observed/snapshot/candidate vectors, and alias mirrors were searched across crates/nuxie-binary; no hit in the decode cycle tracks drift from a source. Dirty-effect APIs elsewhere in the file model authored runtime behavior and are outside this decode subsystem.

## B6-0148

~~~yaml
row_id: B6-0148
cpp_files: ["src/core/binary_reader.cpp"]
rust_module: "crates/nuxie-binary/src/lib.rs"
subsystem_cluster: binary-core
sibling_files_swept:
  - "src/core/field_types/core_bool_type.cpp"
  - "src/core/field_types/core_bytes_type.cpp"
  - "src/core/field_types/core_color_type.cpp"
  - "src/core/field_types/core_double_type.cpp"
  - "src/core/field_types/core_string_type.cpp"
  - "src/core/field_types/core_uint_type.cpp"
  - "crates/nuxie-schema/src/lib.rs"
  - "crates/nuxie-binary/src/lib.rs"
verdict: ISOMORPHIC
axes:
  retained_identity: {status: isomorphic, evidence: ["src/core/binary_reader.cpp:8-13", "crates/nuxie-binary/src/lib.rs:14013-14020"]}
  push_vs_poll: {status: isomorphic, cpp_pushes: false, evidence: ["src/core/binary_reader.cpp:39-49", "crates/nuxie-binary/src/lib.rs:14071-14084"]}
  update_ordering: {status: isomorphic, phases_cpp: ["bounds-check/decode", "advance position", "return"], phases_rust: ["bounds-check/decode", "advance offset", "return Result"]}
  ownership: {status: isomorphic, evidence: ["src/core/binary_reader.cpp:8-13", "crates/nuxie-binary/src/lib.rs:14013-14020"]}
  compensation: {status: clear, mechanisms: [], import_time_constants: []}
idiom_rules_invoked: []
confidence: high
notes: "C++ stores an immutable Span plus cursor and Rust stores the equivalent borrowed slice plus offset. Rust Result error propagation replaces sticky overflow flags but introduces no update/bind-cycle drift tracking. Family grep cleared the decode path."
~~~

## B6-0150

~~~yaml
row_id: B6-0150
cpp_files: ["src/core/field_types/core_bool_type.cpp"]
rust_module: "crates/nuxie-binary/src/lib.rs"
subsystem_cluster: binary-core
sibling_files_swept:
  - "src/core/binary_reader.cpp"
  - "src/core/field_types/core_bytes_type.cpp"
  - "src/core/field_types/core_color_type.cpp"
  - "src/core/field_types/core_double_type.cpp"
  - "src/core/field_types/core_string_type.cpp"
  - "src/core/field_types/core_uint_type.cpp"
  - "crates/nuxie-schema/src/lib.rs"
  - "crates/nuxie-binary/src/lib.rs"
verdict: ADAPTED
axes:
  retained_identity: {status: isomorphic, evidence: ["src/core/field_types/core_bool_type.cpp:6-8", "crates/nuxie-binary/src/lib.rs:13644-13646"]}
  push_vs_poll: {status: isomorphic, cpp_pushes: false, evidence: ["src/core/field_types/core_bool_type.cpp:6-8", "crates/nuxie-binary/src/lib.rs:13644-13646"]}
  update_ordering: {status: isomorphic, phases_cpp: ["dispatch CoreBoolType", "read BinaryReader value", "return"], phases_rust: ["match static FieldKind::Bool", "read BinaryReader value", "construct FieldValue"]}
  ownership: {status: isomorphic, evidence: ["src/core/field_types/core_bool_type.cpp:6-8", "crates/nuxie-binary/src/lib.rs:13644-13646"], note: "scalar bool by value"}
  compensation:
    status: adapted
    mechanisms: []
    import_time_constants:
      - {name: "Property.runtime_type", idiom_rule: "AF-5 import-time devirtualization", evidence: ["crates/nuxie-schema/src/lib.rs:98-103", "crates/nuxie-binary/src/lib.rs:13644-13656"]}
idiom_rules_invoked: ["AF-5 import-time devirtualization"]
confidence: high
notes: "Rust replaces the C++ Core field-type dispatch family with a generated, immutable schema discriminant read by one FieldKind match. It is not mutated during decode/update/bind. Family grep cleared the decode path."
~~~

## B6-0151

~~~yaml
row_id: B6-0151
cpp_files: ["src/core/field_types/core_bytes_type.cpp"]
rust_module: "crates/nuxie-binary/src/lib.rs"
subsystem_cluster: binary-core
sibling_files_swept:
  - "src/core/binary_reader.cpp"
  - "src/core/field_types/core_bool_type.cpp"
  - "src/core/field_types/core_color_type.cpp"
  - "src/core/field_types/core_double_type.cpp"
  - "src/core/field_types/core_string_type.cpp"
  - "src/core/field_types/core_uint_type.cpp"
  - "crates/nuxie-schema/src/lib.rs"
  - "crates/nuxie-binary/src/lib.rs"
verdict: ADAPTED
axes:
  retained_identity: {status: isomorphic, evidence: ["src/core/field_types/core_bytes_type.cpp:6-8", "crates/nuxie-binary/src/lib.rs:13647-13649"]}
  push_vs_poll: {status: isomorphic, cpp_pushes: false, evidence: ["src/core/field_types/core_bytes_type.cpp:6-8", "crates/nuxie-binary/src/lib.rs:13647-13649"]}
  update_ordering: {status: isomorphic, phases_cpp: ["dispatch CoreBytesType", "read BinaryReader value", "return"], phases_rust: ["match static FieldKind::Bytes", "read BinaryReader value", "construct FieldValue"]}
  ownership: {status: isomorphic, evidence: ["src/core/field_types/core_bytes_type.cpp:6-8", "crates/nuxie-binary/src/lib.rs:13647-13649"], note: "immutable input span is copied into the decoded file-owned BytesValue; no shared mutable identity exists"}
  compensation:
    status: adapted
    mechanisms: []
    import_time_constants:
      - {name: "Property.runtime_type", idiom_rule: "AF-5 import-time devirtualization", evidence: ["crates/nuxie-schema/src/lib.rs:98-103", "crates/nuxie-binary/src/lib.rs:13644-13656"]}
idiom_rules_invoked: ["AF-5 import-time devirtualization"]
confidence: high
notes: "Rust replaces the C++ Core field-type dispatch family with a generated, immutable schema discriminant read by one FieldKind match. It is not mutated during decode/update/bind. Family grep cleared the decode path."
~~~

## B6-0152

~~~yaml
row_id: B6-0152
cpp_files: ["src/core/field_types/core_color_type.cpp"]
rust_module: "crates/nuxie-binary/src/lib.rs"
subsystem_cluster: binary-core
sibling_files_swept:
  - "src/core/binary_reader.cpp"
  - "src/core/field_types/core_bool_type.cpp"
  - "src/core/field_types/core_bytes_type.cpp"
  - "src/core/field_types/core_double_type.cpp"
  - "src/core/field_types/core_string_type.cpp"
  - "src/core/field_types/core_uint_type.cpp"
  - "crates/nuxie-schema/src/lib.rs"
  - "crates/nuxie-binary/src/lib.rs"
verdict: ADAPTED
axes:
  retained_identity: {status: isomorphic, evidence: ["src/core/field_types/core_color_type.cpp:6-8", "crates/nuxie-binary/src/lib.rs:13652"]}
  push_vs_poll: {status: isomorphic, cpp_pushes: false, evidence: ["src/core/field_types/core_color_type.cpp:6-8", "crates/nuxie-binary/src/lib.rs:13652"]}
  update_ordering: {status: isomorphic, phases_cpp: ["dispatch CoreColorType", "read BinaryReader value", "return"], phases_rust: ["match static FieldKind::Color", "read BinaryReader value", "construct FieldValue"]}
  ownership: {status: isomorphic, evidence: ["src/core/field_types/core_color_type.cpp:6-8", "crates/nuxie-binary/src/lib.rs:13652"], note: "scalar u32 by value"}
  compensation:
    status: adapted
    mechanisms: []
    import_time_constants:
      - {name: "Property.runtime_type", idiom_rule: "AF-5 import-time devirtualization", evidence: ["crates/nuxie-schema/src/lib.rs:98-103", "crates/nuxie-binary/src/lib.rs:13644-13656"]}
idiom_rules_invoked: ["AF-5 import-time devirtualization"]
confidence: high
notes: "Rust replaces the C++ Core field-type dispatch family with a generated, immutable schema discriminant read by one FieldKind match. It is not mutated during decode/update/bind. Family grep cleared the decode path."
~~~

## B6-0153

~~~yaml
row_id: B6-0153
cpp_files: ["src/core/field_types/core_double_type.cpp"]
rust_module: "crates/nuxie-binary/src/lib.rs"
subsystem_cluster: binary-core
sibling_files_swept:
  - "src/core/binary_reader.cpp"
  - "src/core/field_types/core_bool_type.cpp"
  - "src/core/field_types/core_bytes_type.cpp"
  - "src/core/field_types/core_color_type.cpp"
  - "src/core/field_types/core_string_type.cpp"
  - "src/core/field_types/core_uint_type.cpp"
  - "crates/nuxie-schema/src/lib.rs"
  - "crates/nuxie-binary/src/lib.rs"
verdict: ADAPTED
axes:
  retained_identity: {status: isomorphic, evidence: ["src/core/field_types/core_double_type.cpp:6-8", "crates/nuxie-binary/src/lib.rs:13653"]}
  push_vs_poll: {status: isomorphic, cpp_pushes: false, evidence: ["src/core/field_types/core_double_type.cpp:6-8", "crates/nuxie-binary/src/lib.rs:13653"]}
  update_ordering: {status: isomorphic, phases_cpp: ["dispatch CoreDoubleType", "read BinaryReader value", "return"], phases_rust: ["match static FieldKind::Double", "read BinaryReader value", "construct FieldValue"]}
  ownership: {status: isomorphic, evidence: ["src/core/field_types/core_double_type.cpp:6-8", "crates/nuxie-binary/src/lib.rs:13653"], note: "scalar f32 by value"}
  compensation:
    status: adapted
    mechanisms: []
    import_time_constants:
      - {name: "Property.runtime_type", idiom_rule: "AF-5 import-time devirtualization", evidence: ["crates/nuxie-schema/src/lib.rs:98-103", "crates/nuxie-binary/src/lib.rs:13644-13656"]}
idiom_rules_invoked: ["AF-5 import-time devirtualization"]
confidence: high
notes: "Rust replaces the C++ Core field-type dispatch family with a generated, immutable schema discriminant read by one FieldKind match. It is not mutated during decode/update/bind. Family grep cleared the decode path."
~~~

## B6-0154

~~~yaml
row_id: B6-0154
cpp_files: ["src/core/field_types/core_string_type.cpp"]
rust_module: "crates/nuxie-binary/src/lib.rs"
subsystem_cluster: binary-core
sibling_files_swept:
  - "src/core/binary_reader.cpp"
  - "src/core/field_types/core_bool_type.cpp"
  - "src/core/field_types/core_bytes_type.cpp"
  - "src/core/field_types/core_color_type.cpp"
  - "src/core/field_types/core_double_type.cpp"
  - "src/core/field_types/core_uint_type.cpp"
  - "crates/nuxie-schema/src/lib.rs"
  - "crates/nuxie-binary/src/lib.rs"
verdict: ADAPTED
axes:
  retained_identity: {status: isomorphic, evidence: ["src/core/field_types/core_string_type.cpp:6-8", "crates/nuxie-binary/src/lib.rs:13654; crates/nuxie-binary/src/lib.rs:14054-14058"]}
  push_vs_poll: {status: isomorphic, cpp_pushes: false, evidence: ["src/core/field_types/core_string_type.cpp:6-8", "crates/nuxie-binary/src/lib.rs:13654; crates/nuxie-binary/src/lib.rs:14054-14058"]}
  update_ordering: {status: isomorphic, phases_cpp: ["dispatch CoreStringType", "read BinaryReader value", "return"], phases_rust: ["match static FieldKind::String", "read BinaryReader value", "construct FieldValue"]}
  ownership: {status: isomorphic, evidence: ["src/core/field_types/core_string_type.cpp:6-8", "crates/nuxie-binary/src/lib.rs:13654; crates/nuxie-binary/src/lib.rs:14054-14058"], note: "decoded string/raw bytes are owned by the parsed value; no shared mutable identity exists"}
  compensation:
    status: adapted
    mechanisms: []
    import_time_constants:
      - {name: "Property.runtime_type", idiom_rule: "AF-5 import-time devirtualization", evidence: ["crates/nuxie-schema/src/lib.rs:98-103", "crates/nuxie-binary/src/lib.rs:13644-13656"]}
idiom_rules_invoked: ["AF-5 import-time devirtualization"]
confidence: high
notes: "Rust replaces the C++ Core field-type dispatch family with a generated, immutable schema discriminant read by one FieldKind match. It is not mutated during decode/update/bind. Family grep cleared the decode path."
~~~

## B6-0155

~~~yaml
row_id: B6-0155
cpp_files: ["src/core/field_types/core_uint_type.cpp"]
rust_module: "crates/nuxie-binary/src/lib.rs"
subsystem_cluster: binary-core
sibling_files_swept:
  - "src/core/binary_reader.cpp"
  - "src/core/field_types/core_bool_type.cpp"
  - "src/core/field_types/core_bytes_type.cpp"
  - "src/core/field_types/core_color_type.cpp"
  - "src/core/field_types/core_double_type.cpp"
  - "src/core/field_types/core_string_type.cpp"
  - "crates/nuxie-schema/src/lib.rs"
  - "crates/nuxie-binary/src/lib.rs"
verdict: ADAPTED
axes:
  retained_identity: {status: isomorphic, evidence: ["src/core/field_types/core_uint_type.cpp:6-8", "crates/nuxie-binary/src/lib.rs:13655; crates/nuxie-binary/src/lib.rs:13998-14009"]}
  push_vs_poll: {status: isomorphic, cpp_pushes: false, evidence: ["src/core/field_types/core_uint_type.cpp:6-8", "crates/nuxie-binary/src/lib.rs:13655; crates/nuxie-binary/src/lib.rs:13998-14009"]}
  update_ordering: {status: isomorphic, phases_cpp: ["dispatch CoreUintType", "read BinaryReader value", "return"], phases_rust: ["match static FieldKind::Uint", "read BinaryReader value", "construct FieldValue"]}
  ownership: {status: isomorphic, evidence: ["src/core/field_types/core_uint_type.cpp:6-8", "crates/nuxie-binary/src/lib.rs:13655; crates/nuxie-binary/src/lib.rs:13998-14009"], note: "scalar integer by value"}
  compensation:
    status: adapted
    mechanisms: []
    import_time_constants:
      - {name: "Property.runtime_type", idiom_rule: "AF-5 import-time devirtualization", evidence: ["crates/nuxie-schema/src/lib.rs:98-103", "crates/nuxie-binary/src/lib.rs:13644-13656"]}
idiom_rules_invoked: ["AF-5 import-time devirtualization"]
confidence: high
notes: "Rust replaces the C++ Core field-type dispatch family with a generated, immutable schema discriminant read by one FieldKind match. It is not mutated during decode/update/bind. Family grep cleared the decode path."
~~~

## B6-0448

Post-audit amendment (2026-07-31): this row repairs the B-2 inventory
omission of `src/core/field_types/core_uint64_type.cpp`. The file exists in
the pinned C++ at `d788e8ec` (it is removed later, at candidate `b73bc675`,
by the static-library-linking commit — see
`docs/sync/triage-2026-07-20-b73bc675.md` S3-3) but was absent from the
seeded 447-row inventory, so the original B-6 sweep never assigned it a row.
Audited now against the same pin with the same axes as its B6-0150..0155
siblings. Line anchors cite the current Rust tree, not the tree the original
sweep cited.

~~~yaml
row_id: B6-0448
cpp_files: ["src/core/field_types/core_uint64_type.cpp"]
rust_module: "crates/nuxie-binary/src/lib.rs"
subsystem_cluster: binary-core
sibling_files_swept:
  - "src/core/binary_reader.cpp"
  - "src/core/field_types/core_bool_type.cpp"
  - "src/core/field_types/core_bytes_type.cpp"
  - "src/core/field_types/core_color_type.cpp"
  - "src/core/field_types/core_double_type.cpp"
  - "src/core/field_types/core_string_type.cpp"
  - "src/core/field_types/core_uint_type.cpp"
  - "crates/nuxie-schema/src/lib.rs"
  - "crates/nuxie-binary/src/lib.rs"
verdict: ADAPTED
axes:
  retained_identity: {status: isomorphic, evidence: ["src/core/field_types/core_uint64_type.cpp:6-9", "crates/nuxie-binary/src/lib.rs:14518-14532"]}
  push_vs_poll: {status: isomorphic, cpp_pushes: false, evidence: ["src/core/field_types/core_uint64_type.cpp:6-9", "crates/nuxie-binary/src/lib.rs:14518-14532"]}
  update_ordering: {status: isomorphic, phases_cpp: ["dispatch CoreUint64Type", "readVarUint64 BinaryReader value", "return"], phases_rust: ["match static UintStorage::Uint64", "read_var_uint BinaryReader value", "construct FieldValue"]}
  ownership: {status: isomorphic, evidence: ["src/core/field_types/core_uint64_type.cpp:6-9", "crates/nuxie-binary/src/lib.rs:14518-14532"], note: "scalar integer by value"}
  compensation:
    status: adapted
    mechanisms: []
    import_time_constants:
      - {name: "Property.uint_storage", idiom_rule: "AF-5 import-time devirtualization", evidence: ["crates/nuxie-schema/src/lib.rs:33-53", "crates/nuxie-schema/src/lib.rs:139", "crates/nuxie-binary/src/lib.rs:14518-14532"]}
idiom_rules_invoked: ["AF-5 import-time devirtualization"]
confidence: high
notes: "C++ CoreUint64Type::deserialize is one readVarUint64 call. Rust replaces the field-type dispatch subclass with the generated UintStorage::Uint64 schema discriminant: read_known_uint_field matches Uint64 and calls BinaryReader::read_var_uint, the same full-range varuint64 wire read, unclamped. The WITH_RIVE_TOOLS deserializeRev delegates to deserialize and is outside the read-only runtime contract. Family grep cleared the decode path."
~~~

## B6-0451

Post-audit addition (2026-08-04): `src/core/field_types/core_int_type.cpp`
first appears upstream after the frozen audit ref — it is absent at `d788e8ec`
and present at the live pin `4ac7b327` (sync cycle S4). It replaces the
pre-S4 `src/core/field_types/core_uint64_type.cpp` owner (B6-0448) as this
cluster's newest field type and is audited against `4ac7b327` with the same
axes as its `B6-0150..0155` siblings. C++ anchors cite that pin; Rust anchors
cite the current tree.

~~~yaml
row_id: B6-0451
cpp_files: ["src/core/field_types/core_int_type.cpp"]
rust_module: "crates/nuxie-binary/src/core/field_types/core_int_type.rs"
subsystem_cluster: binary-core
sibling_files_swept:
  - "src/core/binary_reader.cpp"
  - "src/core/field_types/core_bool_type.cpp"
  - "src/core/field_types/core_bytes_type.cpp"
  - "src/core/field_types/core_color_type.cpp"
  - "src/core/field_types/core_double_type.cpp"
  - "src/core/field_types/core_string_type.cpp"
  - "src/core/field_types/core_uint_type.cpp"
  - "crates/nuxie-schema/src/lib.rs"
  - "crates/nuxie-binary/src/core/field_types/mod.rs"
  - "crates/nuxie-binary/src/core/binary_reader.rs"
verdict: ADAPTED
axes:
  retained_identity: {status: isomorphic, evidence: ["cpp@4ac7b327:src/core/field_types/core_int_type.cpp:6-9", "crates/nuxie-binary/src/core/field_types/core_int_type.rs:5-20"], note: "scalar integer by value on both sides"}
  push_vs_poll: {status: isomorphic, cpp_pushes: false, evidence: ["cpp@4ac7b327:src/core/field_types/core_int_type.cpp:6-9", "crates/nuxie-binary/src/core/field_types/core_int_type.rs:10-12"]}
  update_ordering: {status: isomorphic, phases_cpp: ["dispatch CoreIntType", "readVarUintAs<uint32_t>", "zigzagDecode", "return"], phases_rust: ["match static IntStorage", "read_var_uint", "zigzag decode", "return Result"]}
  ownership: {status: isomorphic, evidence: ["cpp@4ac7b327:src/core/field_types/core_int_type.cpp:6-9", "crates/nuxie-binary/src/core/field_types/core_int_type.rs:5-20"]}
  compensation:
    status: adapted
    mechanisms: []
    import_time_constants:
      - {name: "IntStorage schema discriminant", idiom_rule: "AF-5 import-time devirtualization", evidence: ["crates/nuxie-binary/src/core/field_types/core_int_type.rs:5-19"]}
idiom_rules_invoked: ["AF-5 import-time devirtualization"]
confidence: high
notes: "C++ `CoreIntType::deserialize` is one `zigzagDecode(readVarUintAs<uint32_t>())`; the Rust owner performs the same u32 narrowing and the same `(encoded >> 1) ^ -(encoded & 1)` decode. Rust replaces the field-type dispatch subclass with the generated `IntStorage` schema discriminant, which is fixed at build and only read during decode, so it does not pass the mutation-timing gate. The additional `IntStorage::Int16` range check has no `.cpp` counterpart at this seam because C++ narrows in the generated `int16_t` setter instead; it is a decode-side placement of the same storage contract, not drift tracking. The `WITH_RIVE_TOOLS` `deserializeRev` delegates to `deserialize` and is outside the read-only runtime contract. Family grep cleared the decode path."
~~~
