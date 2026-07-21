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
