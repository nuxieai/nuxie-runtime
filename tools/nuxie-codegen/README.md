# nuxie-codegen

Prototype question: can Rive's C++ `dev/defs` JSON files generate Rust metadata that preserves type keys, property keys, runtime inheritance, and deserialization-relevant property flags without copying the C++ class hierarchy?

Run from the workspace root:

```sh
make schema
```

The output is written to:

```text
crates/nuxie-schema/src/generated/schema.rs
```

This is a prototype, but the parser/generator module is intentionally written so it can be lifted into the real codegen tool if the shape holds up.

