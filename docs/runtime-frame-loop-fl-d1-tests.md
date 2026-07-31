# FL-D1 upstream test disposition

This note records the FL-D1 slice of the classifications in
`W65-unit-test-triage.md`.

- Class A: the existing C++ probe fixtures and both golden corpora exercise
  authored ViewModel construction, values, retained children, lists, and
  frame advancement. No D1-only fixture replacement was needed.
- Class C, `viewmodel_instance_prune_test.cpp`: the D1
  `removeValue` contract is ported literally as
  `remove_value_removes_only_the_matching_property`. The sparse
  `DataContext` parent-fallback assertion belongs to FL-D3.
- Class C, `viewmodel_instance_replace_test.cpp`: both dependent-dirty
  assertions are ported literally as
  `replacing_view_model_notifies_every_value_dependent`. The separate
  data-bind apply route is ported as
  `applying_view_model_update_notifies_value_dependent` through the actual
  authored-instance-index `set_view_model_by_property_path` entrypoint used by
  Rust data-bind apply.
- Class C, `viewmodel_instance_list_index_runtime_test.cpp`: assigned to the
  FL-D2 runtime-wrapper family, not FL-D1.
- Class C, `scripting_detached_viewmodel_advance_test.cpp`: the scripting
  wrapper and track lifecycle are outside FL-D1. The D1-owned trigger
  advancement and retained-parent behavior remain covered by the existing
  owned-context tests and scripted golden corpus.
- Class D: W65 assigns no C++-mechanics-only test to FL-D1, so this family has
  no skipped Class D case.

Additional D1 parity regressions cover authored polymorphic value order,
duplicate-value first-match removal, and the rule that removing the winning
`itemIndex` symbol must not restore an earlier symbol registration.
