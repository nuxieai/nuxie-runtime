# B-6 Structural Audit — focus-input

Pinned C++: d788e8ec. Coverage sweep included the whole crates/nuxie-runtime source tree plus focus siblings state_machine/instance.rs, upstream artboard.cpp, artboard_component_list.cpp, nested_artboard.cpp, and animation/state_machine_instance.cpp. The only mutation-gated focus family is the Rust descriptor rescan/reconciliation below; unrelated RB-1 generation/candidate/observed hits belong to data binding.

## B6-0238

~~~yaml
row_id: B6-0238
cpp_files: ["src/input/focus_manager.cpp"]
rust_module: "crates/nuxie-runtime/src/focus.rs"
subsystem_cluster: focus-input
sibling_files_swept:
  - "src/input/focus_node.cpp"
  - "src/input/focusable.cpp"
  - "include/rive/input/focus_manager.hpp"
  - "include/rive/input/focus_node.hpp"
  - "include/rive/input/focusable.hpp"
  - "src/artboard.cpp"
  - "src/artboard_component_list.cpp"
  - "src/nested_artboard.cpp"
  - "src/animation/state_machine_instance.cpp"
  - "crates/nuxie-runtime/src/focus.rs"
  - "crates/nuxie-runtime/src/state_machine/instance.rs"
verdict: DIVERGENT
axes:
  retained_identity: {status: isomorphic, idiom_rule: "AF-1 arena id", evidence: ["include/rive/input/focus_manager.hpp:38-40", "include/rive/input/focus_manager.hpp:160-161", "crates/nuxie-runtime/src/focus.rs:222-229"]}
  push_vs_poll: {status: isomorphic, cpp_pushes: false, evidence: ["src/input/focus_manager.cpp:175-223", "crates/nuxie-runtime/src/focus.rs:242-271"]}
  update_ordering: {status: divergent, phases_cpp: ["build retained tree at initialization", "incrementally add/remove/reparent at structural mutation", "traverse retained tree"], phases_rust: ["rescan artboard before pointer/advance", "diff descriptor keys", "overwrite retained arena nodes", "rebuild lookup maps", "traverse"]}
  ownership: {status: isomorphic, idiom_rule: "AF-1 arena id", evidence: ["include/rive/input/focus_manager.hpp:160-161", "crates/nuxie-runtime/src/focus.rs:224-229"]}
  compensation:
    status: divergent
    mechanisms:
      - {name: "full_tree_descriptor_rescan", kind: "AF-8 invented lifecycle", mutation_gated: true, cpp_counterpart: none, evidence: ["crates/nuxie-runtime/src/state_machine/instance.rs:5754-5755", "crates/nuxie-runtime/src/focus.rs:837-868"]}
      - {name: "focus_key_topology_mirrors", kind: "AF-2/AF-8 rescan reconciliation", mutation_gated: true, cpp_counterpart: none, evidence: ["crates/nuxie-runtime/src/focus.rs:813-819", "crates/nuxie-runtime/src/focus.rs:884-921"]}
      - {name: "target_lookup_rebuild", kind: "AF-8 refresh pass", mutation_gated: true, cpp_counterpart: none, evidence: ["crates/nuxie-runtime/src/focus.rs:922-930"]}
    import_time_constants:
      - {name: "RuntimeFocusTree.inert", idiom_rule: "AF-5 import-time devirtualization", evidence: ["crates/nuxie-runtime/src/focus.rs:813-834"]}
idiom_rules_invoked: ["AF-1 arena id"]
confidence: high
notes: "\"The core manager retains one node per stable arena id, which satisfies AF-1. Divergence is the off-file RuntimeFocusTree lifecycle: C++ builds at initialization and directly updates list/nested ownership at mutation sites (src/artboard.cpp:1999-2027; src/artboard_component_list.cpp:807-813), while Rust reconciles the full projection before pointer and advance calls.\""
~~~

## B6-0239

~~~yaml
row_id: B6-0239
cpp_files: ["src/input/focus_node.cpp"]
rust_module: "crates/nuxie-runtime/src/focus.rs"
subsystem_cluster: focus-input
sibling_files_swept:
  - "src/input/focus_manager.cpp"
  - "src/input/focusable.cpp"
  - "include/rive/input/focus_manager.hpp"
  - "include/rive/input/focus_node.hpp"
  - "include/rive/input/focusable.hpp"
  - "src/artboard.cpp"
  - "src/artboard_component_list.cpp"
  - "src/nested_artboard.cpp"
  - "src/animation/state_machine_instance.cpp"
  - "crates/nuxie-runtime/src/focus.rs"
  - "crates/nuxie-runtime/src/state_machine/instance.rs"
verdict: DIVERGENT
axes:
  retained_identity: {status: isomorphic, idiom_rule: "AF-1 arena id", evidence: ["include/rive/input/focus_node.hpp:28-31", "include/rive/input/focus_node.hpp:203-211", "crates/nuxie-runtime/src/focus.rs:68-83", "crates/nuxie-runtime/src/focus.rs:224-229"]}
  push_vs_poll: {status: isomorphic, cpp_pushes: false, evidence: ["src/input/focus_node.cpp:23-64", "crates/nuxie-runtime/src/focus.rs:242-271", "crates/nuxie-runtime/src/focus.rs:463-471"]}
  update_ordering: {status: divergent, phases_cpp: ["mutate retained node at owner/list/nested call site", "reparent retained node", "traverse"], phases_rust: ["rescan descriptors", "copy descriptor fields into arena node", "reparent by key", "traverse"]}
  ownership: {status: isomorphic, idiom_rule: "AF-1 arena id", evidence: ["src/input/focus_node.cpp:12-20", "crates/nuxie-runtime/src/focus.rs:224-229", "crates/nuxie-runtime/src/focus.rs:339-353"]}
  compensation:
    status: divergent
    mechanisms:
      - {name: "focus_node_snapshot_overwrite", kind: "AF-2 copied-state refresh", mutation_gated: true, cpp_counterpart: none, evidence: ["crates/nuxie-runtime/src/focus.rs:887-900", "crates/nuxie-runtime/src/state_machine/instance.rs:5754-5755"]}
      - {name: "focus_key_topology_mirrors", kind: "AF-8 rescan reconciliation", mutation_gated: true, cpp_counterpart: none, evidence: ["crates/nuxie-runtime/src/focus.rs:813-819", "crates/nuxie-runtime/src/focus.rs:859-921"]}
    import_time_constants:
      - {name: "RuntimeFocusTree.inert", idiom_rule: "AF-5 import-time devirtualization", evidence: ["crates/nuxie-runtime/src/focus.rs:813-834"]}
idiom_rules_invoked: ["AF-1 arena id"]
confidence: high
notes: "\"Stable FocusNodeId ownership is an accepted AF-1 arena mapping. The mutation-gated descriptor overwrite/topology mirror is not: C++ mutates retained FocusNode relationships directly, while Rust refreshes copied node state during sync.\""
~~~

## B6-0240

~~~yaml
row_id: B6-0240
cpp_files: ["src/input/focusable.cpp"]
rust_module: "crates/nuxie-runtime/src/focus.rs"
subsystem_cluster: focus-input
sibling_files_swept:
  - "src/input/focus_manager.cpp"
  - "src/input/focus_node.cpp"
  - "include/rive/input/focus_manager.hpp"
  - "include/rive/input/focus_node.hpp"
  - "include/rive/input/focusable.hpp"
  - "src/artboard.cpp"
  - "src/artboard_component_list.cpp"
  - "src/nested_artboard.cpp"
  - "src/animation/state_machine_instance.cpp"
  - "crates/nuxie-runtime/src/focus.rs"
  - "crates/nuxie-runtime/src/state_machine/instance.rs"
verdict: DIVERGENT
axes:
  retained_identity: {status: divergent, evidence: ["include/rive/input/focus_node.hpp:31", "include/rive/input/focus_node.hpp:50-52", "include/rive/input/focus_node.hpp:203", "crates/nuxie-runtime/src/focus.rs:70-83"]}
  push_vs_poll: {status: isomorphic, cpp_pushes: false, evidence: ["include/rive/input/focusable.hpp:173-201", "crates/nuxie-runtime/src/focus.rs:474-516"]}
  update_ordering: {status: divergent, phases_cpp: ["retain Focusable pointer", "query virtual eligibility/bounds at focus operation", "dispatch callback directly"], phases_rust: ["rescan artboard", "copy eligibility/position into FocusNode", "rebuild target lookup", "emit queued event"]}
  ownership: {status: divergent, evidence: ["include/rive/input/focusable.hpp:160-208", "include/rive/input/focus_node.hpp:203", "crates/nuxie-runtime/src/focus.rs:70-83", "crates/nuxie-runtime/src/focus.rs:798-819"]}
  compensation:
    status: divergent
    mechanisms:
      - {name: "flattened_focusable_snapshot", kind: "AF-1 copied shared relationship", mutation_gated: true, cpp_counterpart: none, evidence: ["crates/nuxie-runtime/src/focus.rs:70-83", "crates/nuxie-runtime/src/focus.rs:887-900", "crates/nuxie-runtime/src/state_machine/instance.rs:5754-5755"]}
      - {name: "focus_target_lookup_rebuild", kind: "AF-8 refresh pass", mutation_gated: true, cpp_counterpart: none, evidence: ["crates/nuxie-runtime/src/focus.rs:918-930"]}
    import_time_constants:
      - {name: "RuntimeFocusTree.inert", idiom_rule: "AF-5 import-time devirtualization", evidence: ["crates/nuxie-runtime/src/focus.rs:813-834"]}
idiom_rules_invoked: []
confidence: high
notes: "\"C++ FocusNode retains and calls a live Focusable pointer; Rust stores copied booleans/geometry plus occurrence-key lookup maps and refreshes them in the advance/pointer cycle. Possible behavior gap: no non-test Rust consumer of take_events and no mapped key/text/gamepad focus dispatch was found; not investigated.\""
~~~
