# FL-D3 / FL-D4 upstream-test evidence

This closeout ports the pinned C++ owner families from
`/Users/levi/dev/oss/rive-runtime` at the repository revision recorded by the
frame-loop manifest. It does not weaken or replace the existing FL-D
differentials.

## FL-D3

FL-D3 has no standalone fixture class in the W65 triage. Its C++ contracts are
covered directly in Rust:

- mutable shared `DataContext` main/global slots, parent fallback, dependent
  registration, replacement, and clone isolation;
- deferred `DataBindPath` decoding/resolution, authored one-ID expansion, and
  file identity;
- `DataBindPathReferencer` copy, claim, and inline-decode ownership.

## FL-D4

| W65 class | Upstream tests | Rust evidence |
|---|---:|---|
| `data_bind_container_test.cpp` (C) | 12 | 12 direct queue/membership tests in `data_bind/data_bind_container.rs`, including mutation during update, additions-before-removals, persistence, recursive rejection, and next-tick dirt |
| `data_bind_lists_test.cpp` (A) | 4 | all four assigned fixtures execute through the silver action runner: `clear_viewmodel_list`, `list_items`, `number_to_list_nested_children`, and `viewmodel_list_trigger` |
| `data_binding_converters_test.cpp` (A) | 3 | all three assigned fixtures execute through the silver action runner: `data_converter_interpolator_reset`, `interpolation_zero_duration`, and `list_to_length_test` |
| `data_binding_cycle_test.cpp` (A) | 7 | seven literal `data_binding_test_3.riv` ports in `tests/cpp_probe.rs`, including child↔parent next-frame propagation, event propagation, same-frame target-to-source precedence, and three-level shared-context writes |

The cycle ports instantiate each authored `main-1` through `main-7` artboard,
bind the same mutable default `DataContext` to the artboard tree and outer state
machine, replay the exact upstream advances/pointer coordinates, and assert the
same rectangle widths or text values at each frame boundary.

The broader W65 FL-D fixture families remain covered by the full `cpp_probe`,
ordinary/scripted golden compares, and silver corpus. The silver generator
reclassifies a case only after the action interpreter can execute its authored
scenario; execution mismatches are retained as signed divergences.
