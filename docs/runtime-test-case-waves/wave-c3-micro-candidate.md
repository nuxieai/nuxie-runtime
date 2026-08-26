# Wave C3 micro/import/language fragment

Status: **CANDIDATE FRAGMENT; PENDING MERGE AND INDEPENDENT REVIEW**

This fragment covers the exact 23 Catch cases in pinned
`lite_rtti_test.cpp`, `malformed_file_import_test.cpp`, `math_test.cpp`, and
`node_test.cpp` at upstream SHA
`4ac7b32798da0482e441ef09304dc3b480ed3ee5`. It is intentionally separate so
the Wave C3 owner can assemble the final `wave-c3.json` with the other disjoint
lanes.

## Census

- 23/23 pinned identities mapped;
- 20 executable passing cases: three direct and 17 explicitly adapted;
- zero expected-red cases;
- three blocked/pending cases: `compact_bitmask_value`,
  `expand_compacted_bitmask_value`, and `iterate_bit_combinations_in_mask`.

The three pending rows have no production runtime owner. An executable red
could only be manufactured by reimplementing the missing algorithm in the
test, calling an unrelated bitmask facility, or failing synthetically. The
fragment records the capability gap instead.

## Owner evidence

- `positive_mod` calls the production converter operation directly and
  preserves all six pinned assertions.
- The remaining executable math utilities use the approved
  `cxx-language-only` adaptation to Rust primitive operations. Mixed signed and
  unsigned comparisons use lossless signed/unsigned representations so Rust
  does not recreate C++'s unsafe default promotions.
- Lite RTTI uses the approved Rust ownership surface (`TypeId`, `Any`, and
  `Rc`) while preserving the complete type distinction, exact-cast,
  nullability, constructor-field, pointer-identity, and reference-counted cast
  sequence.
- Malformed import executes every prefix of the pinned
  `data_binding_test_2.riv` through `nuxie::File::import`. Rust's `Result<File>`
  makes the upstream result/null-file mismatch unrepresentable, while dropping
  every failed or intermediate import exercises the concrete ownership path.
- Node tests use generated `InstanceObjectArena` storage: the exact Node.x
  schema default is read, then the production named setter mutates the same
  occurrence and the getter observes `2.0`.

Only late `cfg(test)` module declarations were added to clean owner files. No
Mat2D, nested-artboard, Artboard runtime, draw, or pre-existing dirty file was
modified. This fragment does not declare itself accepted.

## Gates

- all 20 executable cases pass with `CARGO_INCREMENTAL=0`;
- no expected-red cases exist to force; the three missing owners remain
  honestly pending;
- strict pinned identity, ordinal, source-line, name, classification, outcome,
  adaptation, and evidence-locator validation: 23/23;
- repository correspondence checker: 157 files / 1,404 cases;
- correspondence checker unit suite: 24/24;
- non-test LLVM IR contains no Wave C3 micro test symbols.
