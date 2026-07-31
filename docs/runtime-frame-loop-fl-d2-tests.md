# FL-D2 upstream test disposition

This note records the FL-D2 slice of the classifications in
`W65-unit-test-triage.md`.

- Class A: W65 assigns no fixture scenario specifically to the runtime-wrapper
  owner family. Existing probe and golden scenarios continue to exercise the
  shared owned/default/imported ViewModel entrypoints preserved by the facade.
- Class B: W65 assigns no fixture differential specifically to FL-D2.
- Class C, `viewmodel_instance_list_index_runtime_test.cpp`: both test cases
  are literal public-seam Rust ports:
  `list_index_runtime_reports_type_and_reads_value` and
  `list_index_runtime_reports_value_changes`.
- Class C, `viewmodel_instance_prune_test.cpp`: the `removeValue` assertion was
  ported by FL-D1; the sparse `DataContext` fallback assertion belongs to
  FL-D3.
- Class C, `viewmodel_instance_replace_test.cpp`: both replacement/dependent
  assertions were ported by FL-D1 because the authored instance owns the
  replacement and dirt cascade. FL-D2 additionally covers facade-cache
  replacement identity in
  `replacement_and_list_mutation_keep_concrete_runtime_identity`.
- Class D: W65 assigns no C++-container-mechanics-only test to FL-D2, so this
  family has no skipped Class D case.

Additional FL-D2 regressions cover repeated direct/nested typed-wrapper
identity, wrong-type cache safety, wrapper destruction, list occurrence
identity and mutation, the indexed-insertion parent-registration asymmetry,
live image/artboard state across facade reacquisition, and ViewModel facade
schema/instance creation.
