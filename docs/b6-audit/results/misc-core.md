# B-6 Structural Audit — misc-core

Pinned C++: `d788e8ec6e8b598526607d6a1e8818e8b637b60c`. All 40 assigned C++ files and the mapped Rust facade were read completely. Coverage clause: crate-wide family/keyword grep plus sibling sweeps covered `lib.rs`, `artboard.rs`, `artboard_data_bind.rs`, `components.rs`, `data_bind_graph.rs`, `draw.rs`, `focus.rs`, `objects.rs`, `properties.rs`, `scripting.rs`, `state_machine.rs`, `state_machine/bindables.rs`, `state_machine/instance.rs`, `text.rs`, `view_model.rs`, and the scripting facade in `crates/nuxie/src/lib.rs`. The pinned upstream checkout was read-only. Import/build-only ids, type tags, property keys, parent/dependent vectors, and update-order vectors are AF-1/AF-5/AF-7 idioms and do not pass the mutation-timing gate. Current data-bind/view-model state is recorded against the #RB-1 mini-queue at `docs/parity-closeout-status.md:208-225`; no remediation judgment is made here.

Common Rust sibling sweep for every row below: `["crates/nuxie-runtime/src/lib.rs", "crates/nuxie-runtime/src/artboard.rs", "crates/nuxie-runtime/src/artboard_data_bind.rs", "crates/nuxie-runtime/src/components.rs", "crates/nuxie-runtime/src/data_bind_graph.rs", "crates/nuxie-runtime/src/draw.rs", "crates/nuxie-runtime/src/focus.rs", "crates/nuxie-runtime/src/objects.rs", "crates/nuxie-runtime/src/properties.rs", "crates/nuxie-runtime/src/scripting.rs", "crates/nuxie-runtime/src/state_machine.rs", "crates/nuxie-runtime/src/state_machine/bindables.rs", "crates/nuxie-runtime/src/state_machine/instance.rs", "crates/nuxie-runtime/src/text.rs", "crates/nuxie-runtime/src/view_model.rs", "crates/nuxie/src/lib.rs"]`.

## B6-0001

~~~yaml
row_id: B6-0001
cpp_files: ["src/advancing_component.cpp"]
rust_module: "crates/nuxie-runtime/src/lib.rs"
subsystem_cluster: misc-core
sibling_files_swept: ["crates/nuxie-runtime/src/lib.rs", "crates/nuxie-runtime/src/artboard.rs", "crates/nuxie-runtime/src/components.rs", "crates/nuxie-runtime/src/state_machine.rs", "crates/nuxie-runtime/src/state_machine/instance.rs"]
verdict: ADAPTED
axes:
  retained_identity: {status: adapted, idiom_rule: "AF-1 arena id", evidence: ["src/advancing_component.cpp:17-44", "crates/nuxie-runtime/src/components.rs:477-514"]}
  push_vs_poll: {status: isomorphic, cpp_pushes: false, evidence: ["src/advancing_component.cpp:17-44", "crates/nuxie-runtime/src/artboard.rs:4389-4422"]}
  update_ordering: {status: adapted, phases_cpp: ["type dispatch", "advance"], phases_rust: ["enum/type-tag dispatch", "advance"]}
  ownership: {status: adapted, idiom_rule: "AF-7 own-by-value", evidence: ["crates/nuxie-runtime/src/components.rs:477-514"]}
  compensation: {status: adapted, mechanisms: [], import_time_constants: [{name: "component type tag", idiom_rule: "AF-5 import-time devirtualization", evidence: ["crates/nuxie-runtime/src/components.rs:477-514"]}]}
idiom_rules_invoked: ["AF-1 arena id", "AF-5 import-time devirtualization", "AF-7 own-by-value"]
confidence: high
notes: "Virtual interface dispatch is consolidated into imported type tags and direct Rust calls; the sibling sweep found no cycle-time drift tracker for this row."
~~~

## B6-0095

~~~yaml
row_id: B6-0095
cpp_files: ["src/artboard_component_list.cpp"]
rust_module: "crates/nuxie-runtime/src/lib.rs"
subsystem_cluster: misc-core
sibling_files_swept: ["src/artboard_list_map_rule.cpp", "src/focus_data.cpp", "crates/nuxie-runtime/src/artboard.rs", "crates/nuxie-runtime/src/artboard_data_bind.rs", "crates/nuxie-runtime/src/draw.rs", "crates/nuxie-runtime/src/focus.rs", "crates/nuxie-runtime/src/view_model.rs"]
verdict: DIVERGENT
axes:
  retained_identity: {status: divergent, evidence: ["src/artboard_component_list.cpp:1453-1587", "crates/nuxie-runtime/src/artboard.rs:237-240", "crates/nuxie-runtime/src/artboard.rs:2488-2808"]}
  push_vs_poll: {status: divergent, cpp_pushes: true, evidence: ["src/artboard_component_list.cpp:715-814", "src/artboard_component_list.cpp:1394-1409", "crates/nuxie-runtime/src/artboard.rs:2818-2839", "crates/nuxie-runtime/src/artboard.rs:3218-3223"]}
  update_ordering: {status: divergent, phases_cpp: ["list mutation callback", "updateList", "layout/virtualization"], phases_rust: ["advance-time source refresh", "generation/identity compare", "logical and mounted occurrence reconciliation", "layout/virtualization"]}
  ownership: {status: mixed, evidence: ["src/artboard_component_list.cpp:150-217", "crates/nuxie-runtime/src/artboard.rs:237-240", "crates/nuxie-runtime/src/artboard.rs:2648-2685"]}
  compensation:
    status: divergent
    mechanisms:
      - {name: "advance_time_component_list_reconciliation", kind: "AF-2/AF-8 generation poll plus copied-list refresh", mutation_gated: true, cpp_counterpart: none, evidence: ["crates/nuxie-runtime/src/artboard.rs:2648-2685", "crates/nuxie-runtime/src/artboard.rs:2818-2839", "crates/nuxie-runtime/src/artboard.rs:3218-3223"]}
    import_time_constants:
      - {name: "map rules and component-list descriptors", idiom_rule: "AF-5 import-time devirtualization", evidence: ["crates/nuxie-runtime/src/artboard.rs:2488-2550"]}
idiom_rules_invoked: ["AF-5 import-time devirtualization"]
confidence: high
notes: "C++ retains list-item/artboard identity and enters updateList from direct list change notification; Rust also retains handles, but polls mutation generations and reconciles copied logical/mounted projections during the cycle. Current list binding state is within #RB-1; see docs/parity-closeout-status.md:208-225."
~~~

## B6-0096

~~~yaml
row_id: B6-0096
cpp_files: ["src/artboard_list_map_rule.cpp"]
rust_module: "crates/nuxie-runtime/src/lib.rs"
subsystem_cluster: misc-core
sibling_files_swept: ["src/artboard_component_list.cpp", "crates/nuxie-runtime/src/artboard.rs", "crates/nuxie-runtime/src/draw.rs", "crates/nuxie-runtime/src/text.rs"]
verdict: ADAPTED
axes:
  retained_identity: {status: adapted, idiom_rule: "AF-7 own-by-value", evidence: ["src/artboard_list_map_rule.cpp:7-20", "crates/nuxie-runtime/src/artboard.rs:2518-2545"]}
  push_vs_poll: {status: isomorphic, cpp_pushes: false, evidence: ["src/artboard_list_map_rule.cpp:7-20", "crates/nuxie-runtime/src/artboard.rs:2518-2545"]}
  update_ordering: {status: isomorphic, phases_cpp: ["import rule", "resolve mapping"], phases_rust: ["import rule", "resolve mapping"]}
  ownership: {status: adapted, idiom_rule: "AF-7 own-by-value", evidence: ["crates/nuxie-runtime/src/artboard.rs:2518-2545"]}
  compensation: {status: adapted, mechanisms: [], import_time_constants: [{name: "view-model/artboard map rule", idiom_rule: "AF-5 import-time devirtualization", evidence: ["crates/nuxie-runtime/src/artboard.rs:2518-2545"]}]}
idiom_rules_invoked: ["AF-5 import-time devirtualization", "AF-7 own-by-value"]
confidence: high
notes: "The rule is immutable imported mapping data; component-list cycle-time reconciliation is charged to B6-0095, not duplicated here."
~~~

## B6-0097

~~~yaml
row_id: B6-0097
cpp_files: ["src/artboard_referencer.cpp"]
rust_module: "crates/nuxie-runtime/src/lib.rs"
subsystem_cluster: misc-core
sibling_files_swept: ["src/nested_artboard_layout.cpp", "src/script_input_artboard.cpp", "crates/nuxie-runtime/src/artboard.rs", "crates/nuxie-runtime/src/draw.rs", "crates/nuxie-runtime/src/state_machine/bindables.rs"]
verdict: ADAPTED
axes:
  retained_identity: {status: adapted, idiom_rule: "AF-1 arena id", evidence: ["src/artboard_referencer.cpp:8-57", "crates/nuxie-runtime/src/artboard.rs:339-369"]}
  push_vs_poll: {status: isomorphic, cpp_pushes: false, evidence: ["src/artboard_referencer.cpp:8-57", "crates/nuxie-runtime/src/artboard.rs:339-369"]}
  update_ordering: {status: adapted, phases_cpp: ["resolve artboard", "instance"], phases_rust: ["resolve graph id", "own child instance"]}
  ownership: {status: adapted, idiom_rule: "AF-6 explicit deep clone", evidence: ["crates/nuxie-runtime/src/artboard.rs:339-369", "crates/nuxie-runtime/src/artboard.rs:665-704"]}
  compensation: {status: adapted, mechanisms: [], import_time_constants: [{name: "referenced artboard graph id", idiom_rule: "AF-1 arena id", evidence: ["crates/nuxie-runtime/src/artboard.rs:339-369"]}]}
idiom_rules_invoked: ["AF-1 arena id", "AF-6 Deep copy is explicit"]
confidence: high
notes: "Stable graph/local ids replace raw referencer pointers, and child occurrence cloning is explicit."
~~~

## B6-0114

~~~yaml
row_id: B6-0114
cpp_files: ["src/bindable_artboard.cpp"]
rust_module: "crates/nuxie-runtime/src/lib.rs"
subsystem_cluster: misc-core
sibling_files_swept: ["crates/nuxie-runtime/src/state_machine.rs", "crates/nuxie-runtime/src/state_machine/bindables.rs", "crates/nuxie-runtime/src/state_machine/instance.rs", "crates/nuxie-runtime/src/data_bind_graph.rs"]
verdict: ADAPTED
axes:
  retained_identity: {status: adapted, idiom_rule: "AF-1 arena id", evidence: ["src/bindable_artboard.cpp:1-7", "crates/nuxie-runtime/src/state_machine/bindables.rs:196-205", "crates/nuxie-runtime/src/state_machine/bindables.rs:537-560"]}
  push_vs_poll: {status: adapted, cpp_pushes: true, evidence: ["crates/nuxie-runtime/src/state_machine/bindables.rs:548-556", "crates/nuxie-runtime/src/state_machine/instance.rs:5321-5399"]}
  update_ordering: {status: adapted, phases_cpp: ["bind", "property write"], phases_rust: ["bind descriptor", "direct instance value write"]}
  ownership: {status: adapted, idiom_rule: "AF-7 own-by-value", evidence: ["crates/nuxie-runtime/src/state_machine/bindables.rs:537-560"]}
  compensation: {status: adapted, mechanisms: [], import_time_constants: [{name: "data-bind index vector", idiom_rule: "AF-5 import-time devirtualization", evidence: ["crates/nuxie-runtime/src/state_machine/bindables.rs:1507-1542"]}]}
idiom_rules_invoked: ["AF-1 arena id", "AF-5 import-time devirtualization", "AF-7 own-by-value"]
confidence: high
notes: "This isolated bindable value uses stable ids and direct mutation. Broader RB-1 candidate/generation machinery is recorded on Core and the data-bind/view-model rows, not duplicated here."
~~~

## B6-0145

~~~yaml
row_id: B6-0145
cpp_files: ["src/container_component.cpp"]
rust_module: "crates/nuxie-runtime/src/lib.rs"
subsystem_cluster: misc-core
sibling_files_swept: ["src/solo.cpp", "src/layout_component.cpp", "crates/nuxie-runtime/src/components.rs", "crates/nuxie-runtime/src/artboard.rs"]
verdict: ADAPTED
axes:
  retained_identity: {status: adapted, idiom_rule: "AF-1 arena id", evidence: ["src/container_component.cpp:8-44", "crates/nuxie-runtime/src/components.rs:477-514", "crates/nuxie-runtime/src/artboard.rs:5434-5505"]}
  push_vs_poll: {status: isomorphic, cpp_pushes: false, evidence: ["src/container_component.cpp:8-44", "crates/nuxie-runtime/src/artboard.rs:5434-5505"]}
  update_ordering: {status: isomorphic, phases_cpp: ["collapse", "recurse children"], phases_rust: ["collapse", "recurse child ids"]}
  ownership: {status: adapted, idiom_rule: "AF-7 own-by-value", evidence: ["crates/nuxie-runtime/src/components.rs:477-514"]}
  compensation: {status: adapted, mechanisms: [], import_time_constants: [{name: "parent_local topology", idiom_rule: "AF-1 arena id", evidence: ["crates/nuxie-runtime/src/components.rs:477-514"]}]}
idiom_rules_invoked: ["AF-1 arena id", "AF-7 own-by-value"]
confidence: high
notes: "The Rust child walk derives immutable imported topology during a collapse call; it does not track mutable topology drift and therefore does not pass the compensation gate."
~~~

## B6-0146

~~~yaml
row_id: B6-0146
cpp_files: ["src/core.cpp"]
rust_module: "crates/nuxie-runtime/src/lib.rs"
subsystem_cluster: misc-core
sibling_files_swept: ["crates/nuxie-runtime/src/artboard.rs", "crates/nuxie-runtime/src/artboard_data_bind.rs", "crates/nuxie-runtime/src/data_bind_graph.rs", "crates/nuxie-runtime/src/retained_data_bind.rs", "crates/nuxie-runtime/src/view_model.rs", "crates/nuxie-runtime/src/view_model_cell.rs", "crates/nuxie-runtime/src/state_machine/instance.rs"]
verdict: DIVERGENT
axes:
  retained_identity: {status: mixed, evidence: ["src/core.cpp:12-76", "crates/nuxie-runtime/src/view_model_cell.rs:149-224", "crates/nuxie-runtime/src/artboard_data_bind.rs:611-698"]}
  push_vs_poll: {status: divergent, cpp_pushes: true, evidence: ["src/core.cpp:12-76", "crates/nuxie-runtime/src/artboard.rs:3733-3734", "crates/nuxie-runtime/src/artboard_data_bind.rs:5800-5925", "crates/nuxie-runtime/src/state_machine/instance.rs:5390-5408"]}
  update_ordering: {status: divergent, phases_cpp: ["property write", "notify intrusive observers", "dependent dirt"], phases_rust: ["property write", "dirty epoch/cell dirt", "advance-time generation rebind", "apply"]}
  ownership: {status: mixed, evidence: ["src/core.cpp:12-76", "crates/nuxie-runtime/src/view_model_cell.rs:149-224", "crates/nuxie-runtime/src/artboard_data_bind.rs:4318-4326"]}
  compensation:
    status: divergent
    mechanisms:
      - {name: "artboard_data_bind_dirty_epoch_gate", kind: "AF-4/AF-8 parallel dirt epoch", mutation_gated: true, cpp_counterpart: none, evidence: ["crates/nuxie-runtime/src/artboard.rs:293-294", "crates/nuxie-runtime/src/artboard.rs:3733-3734", "crates/nuxie-runtime/src/artboard_data_bind.rs:5800-5925"]}
      - {name: "owned_candidate_mutation_generation_rebind", kind: "AF-2 generation poll/rebind", mutation_gated: true, cpp_counterpart: none, evidence: ["crates/nuxie-runtime/src/state_machine/instance.rs:5172-5220", "crates/nuxie-runtime/src/state_machine/instance.rs:5390-5408"]}
    import_time_constants:
      - {name: "data_bind_observed/type descriptors", idiom_rule: "AF-5 import-time devirtualization", evidence: ["crates/nuxie-runtime/src/artboard.rs:935-1041"]}
idiom_rules_invoked: ["AF-5 import-time devirtualization"]
confidence: high
notes: "Current main has retained cells for migrated scalar paths but still carries dirty-epoch and candidate-generation lifecycles beside them. This is the current #RB-1 state; see docs/parity-closeout-status.md:208-225."
~~~

## B6-0156

~~~yaml
row_id: B6-0156
cpp_files: ["src/custom_property_container.cpp"]
rust_module: "crates/nuxie-runtime/src/lib.rs"
subsystem_cluster: misc-core
sibling_files_swept: ["crates/nuxie-runtime/src/artboard.rs", "crates/nuxie-runtime/src/artboard_data_bind.rs", "crates/nuxie-runtime/src/objects.rs", "crates/nuxie-runtime/src/state_machine.rs"]
verdict: ADAPTED
axes:
  retained_identity: {status: adapted, idiom_rule: "AF-1 arena id", evidence: ["src/custom_property_container.cpp:7-36", "crates/nuxie-runtime/src/artboard.rs:1548-1584"]}
  push_vs_poll: {status: adapted, cpp_pushes: true, evidence: ["src/custom_property_container.cpp:18-36", "crates/nuxie-runtime/src/artboard.rs:4574-4583"]}
  update_ordering: {status: adapted, phases_cpp: ["import child properties", "direct property change"], phases_rust: ["import typed slots", "direct setter/dirt"]}
  ownership: {status: adapted, idiom_rule: "AF-7 own-by-value", evidence: ["crates/nuxie-runtime/src/artboard.rs:1548-1584"]}
  compensation: {status: adapted, mechanisms: [], import_time_constants: [{name: "custom-property membership/type", idiom_rule: "AF-5 import-time devirtualization", evidence: ["crates/nuxie-runtime/src/artboard.rs:1548-1584"]}]}
idiom_rules_invoked: ["AF-1 arena id", "AF-5 import-time devirtualization", "AF-7 own-by-value"]
confidence: high
notes: "Container membership is imported once; mutable property writes use the ordinary runtime setter/dirt path."
~~~

## B6-0201

~~~yaml
row_id: B6-0201
cpp_files: ["src/data_bind_path_referencer.cpp"]
rust_module: "crates/nuxie-runtime/src/lib.rs"
subsystem_cluster: misc-core
sibling_files_swept: ["crates/nuxie-runtime/src/objects.rs", "crates/nuxie-runtime/src/scripting.rs", "crates/nuxie-runtime/src/artboard_data_bind.rs", "crates/nuxie-runtime/src/data_bind_graph.rs"]
verdict: ADAPTED
axes:
  retained_identity: {status: adapted, idiom_rule: "AF-1 arena id", evidence: ["src/data_bind_path_referencer.cpp:7-45", "crates/nuxie-runtime/src/scripting.rs:1333-1375"]}
  push_vs_poll: {status: isomorphic, cpp_pushes: false, evidence: ["src/data_bind_path_referencer.cpp:7-45", "crates/nuxie-runtime/src/scripting.rs:1333-1375"]}
  update_ordering: {status: adapted, phases_cpp: ["resolve path", "retain/claim"], phases_rust: ["decode id path", "resolve against retained context"]}
  ownership: {status: adapted, idiom_rule: "AF-7 own-by-value", evidence: ["crates/nuxie-runtime/src/scripting.rs:1333-1375"]}
  compensation: {status: adapted, mechanisms: [], import_time_constants: [{name: "resolved source path ids", idiom_rule: "AF-5 import-time devirtualization", evidence: ["crates/nuxie-runtime/src/scripting.rs:1333-1375"]}]}
idiom_rules_invoked: ["AF-1 arena id", "AF-5 import-time devirtualization", "AF-7 own-by-value"]
confidence: high
notes: "The referencer's fixed object path becomes an owned id vector; RB-1 rebind mechanisms are attributed to the consumers, not this immutable path descriptor."
~~~

## B6-0202

~~~yaml
row_id: B6-0202
cpp_files: ["src/dependency_sorter.cpp"]
rust_module: "crates/nuxie-runtime/src/lib.rs"
subsystem_cluster: misc-core
sibling_files_swept: ["crates/nuxie-runtime/src/components.rs", "crates/nuxie-runtime/src/artboard.rs", "crates/nuxie-runtime/src/draw.rs"]
verdict: ADAPTED
axes:
  retained_identity: {status: adapted, idiom_rule: "AF-1 arena id", evidence: ["src/dependency_sorter.cpp:6-48", "crates/nuxie-runtime/src/components.rs:560-633", "crates/nuxie-runtime/src/artboard.rs:872-886"]}
  push_vs_poll: {status: isomorphic, cpp_pushes: false, evidence: ["src/dependency_sorter.cpp:6-48", "crates/nuxie-runtime/src/artboard.rs:4389-4422"]}
  update_ordering: {status: isomorphic, phases_cpp: ["DFS dependents", "assign order", "iterate order"], phases_rust: ["build dependent ids", "sort graph order", "iterate order"]}
  ownership: {status: adapted, idiom_rule: "AF-7 own-by-value", evidence: ["crates/nuxie-runtime/src/artboard.rs:872-886"]}
  compensation: {status: adapted, mechanisms: [], import_time_constants: [{name: "update_order and dependent ids", idiom_rule: "AF-5 import-time devirtualization", evidence: ["crates/nuxie-runtime/src/components.rs:560-633", "crates/nuxie-runtime/src/artboard.rs:872-886"]}]}
idiom_rules_invoked: ["AF-1 arena id", "AF-5 import-time devirtualization", "AF-7 own-by-value"]
confidence: high
notes: "The visited/cycle guards mirror the C++ sorter and are fixed graph-construction machinery, not compensation."
~~~

## B6-0203

~~~yaml
row_id: B6-0203
cpp_files: ["src/draw_rules.cpp"]
rust_module: "crates/nuxie-runtime/src/lib.rs"
subsystem_cluster: misc-core
sibling_files_swept: ["src/draw_target.cpp", "src/drawable.cpp", "crates/nuxie-runtime/src/draw.rs", "crates/nuxie-runtime/src/artboard.rs"]
verdict: ADAPTED
axes:
  retained_identity: {status: adapted, idiom_rule: "AF-1 arena id", evidence: ["src/draw_rules.cpp:8-40", "crates/nuxie-runtime/src/draw.rs:1790-1810"]}
  push_vs_poll: {status: adapted, cpp_pushes: true, evidence: ["src/draw_rules.cpp:27-40", "crates/nuxie-runtime/src/artboard.rs:4548-4562"]}
  update_ordering: {status: adapted, phases_cpp: ["resolve target", "dirty/reorder"], phases_rust: ["resolve target id", "mark draw-order dirty"]}
  ownership: {status: adapted, idiom_rule: "AF-1 arena id", evidence: ["crates/nuxie-runtime/src/draw.rs:1790-1810"]}
  compensation: {status: adapted, mechanisms: [], import_time_constants: [{name: "draw target id/property key", idiom_rule: "AF-5 import-time devirtualization", evidence: ["crates/nuxie-runtime/src/draw.rs:141-145"]}]}
idiom_rules_invoked: ["AF-1 arena id", "AF-5 import-time devirtualization"]
confidence: high
notes: "Stable target ids and ordinary draw-order dirt replace raw pointers without a refresh lifecycle."
~~~

## B6-0204

~~~yaml
row_id: B6-0204
cpp_files: ["src/draw_target.cpp"]
rust_module: "crates/nuxie-runtime/src/lib.rs"
subsystem_cluster: misc-core
sibling_files_swept: ["src/draw_rules.cpp", "src/drawable.cpp", "crates/nuxie-runtime/src/draw.rs", "crates/nuxie-runtime/src/artboard.rs"]
verdict: ADAPTED
axes:
  retained_identity: {status: adapted, idiom_rule: "AF-1 arena id", evidence: ["src/draw_target.cpp:8-32", "crates/nuxie-runtime/src/draw.rs:1798-1808"]}
  push_vs_poll: {status: adapted, cpp_pushes: true, evidence: ["src/draw_target.cpp:19-32", "crates/nuxie-runtime/src/artboard.rs:4553-4562"]}
  update_ordering: {status: adapted, phases_cpp: ["resolve drawable", "placement dirt"], phases_rust: ["resolve local id", "draw-order dirt"]}
  ownership: {status: adapted, idiom_rule: "AF-1 arena id", evidence: ["crates/nuxie-runtime/src/draw.rs:1798-1808"]}
  compensation: {status: adapted, mechanisms: [], import_time_constants: [{name: "placement/property keys", idiom_rule: "AF-5 import-time devirtualization", evidence: ["crates/nuxie-runtime/src/draw.rs:141-145"]}]}
idiom_rules_invoked: ["AF-1 arena id", "AF-5 import-time devirtualization"]
confidence: high
notes: "Placement changes enter the canonical artboard/draw dirt path directly."
~~~

## B6-0205

~~~yaml
row_id: B6-0205
cpp_files: ["src/drawable.cpp"]
rust_module: "crates/nuxie-runtime/src/lib.rs"
subsystem_cluster: misc-core
sibling_files_swept: ["src/draw_rules.cpp", "src/draw_target.cpp", "crates/nuxie-runtime/src/draw.rs", "crates/nuxie-runtime/src/artboard.rs", "crates/nuxie-runtime/src/components.rs"]
verdict: ADAPTED
axes:
  retained_identity: {status: adapted, idiom_rule: "AF-1 arena id", evidence: ["src/drawable.cpp:10-87", "crates/nuxie-runtime/src/draw.rs:4090-4240"]}
  push_vs_poll: {status: adapted, cpp_pushes: true, evidence: ["src/drawable.cpp:10-45", "crates/nuxie-runtime/src/artboard.rs:3790-3818"]}
  update_ordering: {status: adapted, phases_cpp: ["dirt", "update", "draw/hit-test"], phases_rust: ["dirt", "prepare commands", "draw/hit-test"]}
  ownership: {status: adapted, idiom_rule: "AF-7 own-by-value", evidence: ["crates/nuxie-runtime/src/draw.rs:4090-4240"]}
  compensation: {status: adapted, mechanisms: [], import_time_constants: [{name: "drawable kind/local id", idiom_rule: "AF-5 import-time devirtualization", evidence: ["crates/nuxie-runtime/src/draw.rs:6656-6689"]}]}
idiom_rules_invoked: ["AF-1 arena id", "AF-5 import-time devirtualization", "AF-7 own-by-value"]
confidence: high
notes: "Renderer-resource cache epochs are canonical derived-renderer invalidation and have direct C++ drawable/paint dirt counterparts; no copied authoritative relationship for this row is reconciled."
~~~

## B6-0206

~~~yaml
row_id: B6-0206
cpp_files: ["src/event.cpp"]
rust_module: "crates/nuxie-runtime/src/lib.rs"
subsystem_cluster: misc-core
sibling_files_swept: ["crates/nuxie-runtime/src/artboard.rs", "crates/nuxie-runtime/src/state_machine.rs", "crates/nuxie-runtime/src/state_machine/instance.rs"]
verdict: ADAPTED
axes:
  retained_identity: {status: adapted, idiom_rule: "AF-7 own-by-value", evidence: ["src/event.cpp:8-10", "crates/nuxie-runtime/src/artboard.rs:1548-1584"]}
  push_vs_poll: {status: isomorphic, cpp_pushes: true, evidence: ["src/event.cpp:8-10", "crates/nuxie-runtime/src/state_machine/instance.rs:4958-5014"]}
  update_ordering: {status: isomorphic, phases_cpp: ["report callback"], phases_rust: ["emit/report event"]}
  ownership: {status: adapted, idiom_rule: "AF-7 own-by-value", evidence: ["crates/nuxie-runtime/src/artboard.rs:1548-1584"]}
  compensation: {status: adapted, mechanisms: [], import_time_constants: [{name: "event/property descriptors", idiom_rule: "AF-5 import-time devirtualization", evidence: ["crates/nuxie-runtime/src/artboard.rs:1548-1584"]}]}
idiom_rules_invoked: ["AF-5 import-time devirtualization", "AF-7 own-by-value"]
confidence: high
notes: "Event payloads are owned values and are emitted from the active state-machine cycle; no polling bridge was found."
~~~

## B6-0207

~~~yaml
row_id: B6-0207
cpp_files: ["src/factory.cpp"]
rust_module: "crates/nuxie-runtime/src/lib.rs"
subsystem_cluster: misc-core
sibling_files_swept: ["src/renderer.cpp", "crates/nuxie-runtime/src/draw.rs", "crates/nuxie-runtime/src/scripting.rs", "crates/nuxie-runtime/src/lib.rs"]
verdict: ADAPTED
axes:
  retained_identity: {status: adapted, idiom_rule: "AF-3 trait object for real polymorphism", evidence: ["src/factory.cpp:7-39", "crates/nuxie-runtime/src/scripting.rs:109-180"]}
  push_vs_poll: {status: isomorphic, cpp_pushes: false, evidence: ["src/factory.cpp:7-39", "crates/nuxie-runtime/src/scripting.rs:109-180"]}
  update_ordering: {status: isomorphic, phases_cpp: ["allocate renderer object"], phases_rust: ["allocate through RenderFactory trait"]}
  ownership: {status: adapted, idiom_rule: "AF-3 trait object for real polymorphism", evidence: ["crates/nuxie-runtime/src/scripting.rs:109-180"]}
  compensation: {status: adapted, mechanisms: [], import_time_constants: []}
idiom_rules_invoked: ["AF-3 trait object for real polymorphism"]
confidence: high
notes: "Factory polymorphism remains runtime polymorphism through a trait; no representation-repair lifecycle is present."
~~~

## B6-0208

~~~yaml
row_id: B6-0208
cpp_files: ["src/file.cpp"]
rust_module: "crates/nuxie-runtime/src/lib.rs"
subsystem_cluster: misc-core
sibling_files_swept: ["crates/nuxie-runtime/src/objects.rs", "crates/nuxie-runtime/src/artboard.rs", "crates/nuxie-runtime/src/view_model.rs", "crates/nuxie-runtime/src/state_machine.rs", "crates/nuxie-runtime/src/lib.rs"]
verdict: ADAPTED
axes:
  retained_identity: {status: adapted, idiom_rule: "AF-1 arena id", evidence: ["src/file.cpp:127-246", "src/file.cpp:804-895", "crates/nuxie-runtime/src/objects.rs:1-220"]}
  push_vs_poll: {status: isomorphic, cpp_pushes: false, evidence: ["src/file.cpp:127-246", "crates/nuxie-runtime/src/objects.rs:1-220"]}
  update_ordering: {status: adapted, phases_cpp: ["read objects", "resolve imports", "instantiate artboard/view model"], phases_rust: ["binary decode", "build object arena/graphs", "instantiate artboard/view model"]}
  ownership: {status: adapted, idiom_rule: "AF-6 explicit deep clone", evidence: ["src/file.cpp:201-246", "crates/nuxie-runtime/src/artboard.rs:665-704"]}
  compensation: {status: adapted, mechanisms: [], import_time_constants: [{name: "object/type/property registries", idiom_rule: "AF-5 import-time devirtualization", evidence: ["crates/nuxie-runtime/src/objects.rs:1-220"]}]}
idiom_rules_invoked: ["AF-1 arena id", "AF-5 import-time devirtualization", "AF-6 Deep copy is explicit"]
confidence: medium
notes: "The monolithic C++ file is split across the binary decoder and runtime graph/object modules; the audited structural seams preserve stable ids and explicit occurrence copies."
~~~

## B6-0209

~~~yaml
row_id: B6-0209
cpp_files: ["src/focus_data.cpp"]
rust_module: "crates/nuxie-runtime/src/lib.rs"
subsystem_cluster: misc-core
sibling_files_swept: ["src/artboard_component_list.cpp", "crates/nuxie-runtime/src/focus.rs", "crates/nuxie-runtime/src/state_machine/instance.rs", "crates/nuxie-runtime/src/artboard.rs"]
verdict: DIVERGENT
axes:
  retained_identity: {status: divergent, evidence: ["src/focus_data.cpp:38-185", "src/focus_data.cpp:414-469", "crates/nuxie-runtime/src/focus.rs:70-83", "crates/nuxie-runtime/src/focus.rs:813-930"]}
  push_vs_poll: {status: divergent, cpp_pushes: true, evidence: ["src/focus_data.cpp:72-185", "src/focus_data.cpp:372-389", "crates/nuxie-runtime/src/state_machine/instance.rs:5754-5755", "crates/nuxie-runtime/src/focus.rs:837-930"]}
  update_ordering: {status: divergent, phases_cpp: ["mutate retained FocusNode/FocusData", "direct listeners/reparent", "traverse"], phases_rust: ["advance-time descriptor rescan", "overwrite node snapshots", "rebuild topology/lookup", "traverse"]}
  ownership: {status: mixed, evidence: ["src/focus_data.cpp:38-185", "crates/nuxie-runtime/src/focus.rs:222-229", "crates/nuxie-runtime/src/focus.rs:798-819"]}
  compensation:
    status: divergent
    mechanisms:
      - {name: "full_focus_descriptor_rescan", kind: "AF-8 invented lifecycle", mutation_gated: true, cpp_counterpart: none, evidence: ["crates/nuxie-runtime/src/state_machine/instance.rs:5754-5755", "crates/nuxie-runtime/src/focus.rs:837-868"]}
      - {name: "focus_node_snapshot_and_topology_mirror", kind: "AF-1/AF-2 copied-state reconciliation", mutation_gated: true, cpp_counterpart: none, evidence: ["crates/nuxie-runtime/src/focus.rs:813-819", "crates/nuxie-runtime/src/focus.rs:884-921"]}
      - {name: "focus_target_lookup_rebuild", kind: "AF-8 refresh pass", mutation_gated: true, cpp_counterpart: none, evidence: ["crates/nuxie-runtime/src/focus.rs:922-930"]}
    import_time_constants:
      - {name: "RuntimeFocusTree.inert", idiom_rule: "AF-5 import-time devirtualization", evidence: ["crates/nuxie-runtime/src/focus.rs:813-834"]}
idiom_rules_invoked: ["AF-5 import-time devirtualization"]
confidence: high
notes: "This row owns the same mutation-gated focus projection family recorded by the dedicated focus cluster, but for FocusData's direct listener/parent relationships. Possible behavior gap: mapped key/text/gamepad FocusData dispatch was not found; noted only."
~~~

## B6-0210

~~~yaml
row_id: B6-0210
cpp_files: ["src/foreground_layout_drawable.cpp"]
rust_module: "crates/nuxie-runtime/src/lib.rs"
subsystem_cluster: misc-core
sibling_files_swept: ["src/layout_component.cpp", "src/drawable.cpp", "crates/nuxie-runtime/src/draw.rs", "crates/nuxie-runtime/src/artboard.rs"]
verdict: ADAPTED
axes:
  retained_identity: {status: adapted, idiom_rule: "AF-1 arena id", evidence: ["src/foreground_layout_drawable.cpp:9-82", "crates/nuxie-runtime/src/draw.rs:4109-4230", "crates/nuxie-runtime/src/draw.rs:12956-12970"]}
  push_vs_poll: {status: adapted, cpp_pushes: true, evidence: ["src/foreground_layout_drawable.cpp:31-61", "crates/nuxie-runtime/src/draw.rs:12956-12970"]}
  update_ordering: {status: isomorphic, phases_cpp: ["parent layout update", "paint parent path"], phases_rust: ["prepare parent layout command", "paint parent path"]}
  ownership: {status: adapted, idiom_rule: "AF-1 arena id", evidence: ["crates/nuxie-runtime/src/draw.rs:4109-4230"]}
  compensation: {status: adapted, mechanisms: [], import_time_constants: [{name: "drawable kind/parent local", idiom_rule: "AF-5 import-time devirtualization", evidence: ["crates/nuxie-runtime/src/draw.rs:6656-6689"]}]}
idiom_rules_invoked: ["AF-1 arena id", "AF-5 import-time devirtualization"]
confidence: high
notes: "The parent pointer becomes a stable local id and the draw path preserves foreground ordering."
~~~

## B6-0247

~~~yaml
row_id: B6-0247
cpp_files: ["src/layout.cpp"]
rust_module: "crates/nuxie-runtime/src/lib.rs"
subsystem_cluster: misc-core
sibling_files_swept: ["src/layout_component.cpp", "crates/nuxie-runtime/src/components.rs", "crates/nuxie-runtime/src/draw.rs"]
verdict: ISOMORPHIC
axes:
  retained_identity: {status: isomorphic, evidence: ["src/layout.cpp:5-21", "crates/nuxie-runtime/src/components.rs:116-180"]}
  push_vs_poll: {status: isomorphic, cpp_pushes: false, evidence: ["src/layout.cpp:5-21", "crates/nuxie-runtime/src/components.rs:116-180"]}
  update_ordering: {status: isomorphic, phases_cpp: ["read alignment constants"], phases_rust: ["read alignment values"]}
  ownership: {status: isomorphic, evidence: ["src/layout.cpp:5-21", "crates/nuxie-runtime/src/components.rs:116-180"]}
  compensation: {status: isomorphic, mechanisms: [], import_time_constants: [{name: "alignment constants", idiom_rule: "AF-7 own-by-value", evidence: ["src/layout.cpp:5-21", "crates/nuxie-runtime/src/components.rs:116-180"]}]}
idiom_rules_invoked: ["AF-7 own-by-value"]
confidence: high
notes: "This file is immutable alignment data; no mutable relationship exists to compensate."
~~~

## B6-0258

~~~yaml
row_id: B6-0258
cpp_files: ["src/layout_component.cpp"]
rust_module: "crates/nuxie-runtime/src/lib.rs"
subsystem_cluster: misc-core
sibling_files_swept: ["src/foreground_layout_drawable.cpp", "src/nested_artboard_layout.cpp", "crates/nuxie-runtime/src/artboard.rs", "crates/nuxie-runtime/src/components.rs", "crates/nuxie-runtime/src/draw.rs"]
verdict: DIVERGENT
axes:
  retained_identity: {status: divergent, evidence: ["src/layout_component.cpp:255-930", "crates/nuxie-runtime/src/artboard.rs:319-320", "crates/nuxie-runtime/src/draw.rs:4543-4635"]}
  push_vs_poll: {status: mixed, cpp_pushes: true, evidence: ["src/layout_component.cpp:31-44", "src/layout_component.cpp:1412-1430", "crates/nuxie-runtime/src/artboard.rs:3758-3785"]}
  update_ordering: {status: divergent, phases_cpp: ["mutate retained Yoga node", "dirt dependents", "calculate/update layout"], phases_rust: ["property/layout dirt", "rebuild Taffy bounds snapshot", "dirty layout dependents", "prepare draw"]}
  ownership: {status: divergent, evidence: ["src/layout_component.cpp:255-930", "crates/nuxie-runtime/src/artboard.rs:319-320", "crates/nuxie-runtime/src/draw.rs:4543-4635"]}
  compensation:
    status: divergent
    mechanisms:
      - {name: "layout_constraint_bounds_snapshot_refresh", kind: "AF-1/AF-8 copied layout-tree snapshot", mutation_gated: true, cpp_counterpart: none, evidence: ["crates/nuxie-runtime/src/artboard.rs:319-320", "crates/nuxie-runtime/src/artboard.rs:3758-3785", "crates/nuxie-runtime/src/draw.rs:4543-4635"]}
    import_time_constants:
      - {name: "layout style/property descriptors", idiom_rule: "AF-5 import-time devirtualization", evidence: ["crates/nuxie-runtime/src/draw.rs:4543-4589"]}
idiom_rules_invoked: ["AF-5 import-time devirtualization"]
confidence: high
notes: "C++ owns and mutates a retained Yoga node/tree. Rust retains authored component state but materializes an Arc bounds snapshot and refreshes it during layout mutation/update to keep the separate representation coherent."
~~~

## B6-0304

~~~yaml
row_id: B6-0304
cpp_files: ["src/nested_artboard_layout.cpp"]
rust_module: "crates/nuxie-runtime/src/lib.rs"
subsystem_cluster: misc-core
sibling_files_swept: ["src/layout_component.cpp", "src/nested_artboard_leaf.cpp", "src/nested_artboard_origin.cpp", "crates/nuxie-runtime/src/artboard.rs", "crates/nuxie-runtime/src/draw.rs"]
verdict: DIVERGENT
axes:
  retained_identity: {status: mixed, evidence: ["src/nested_artboard_layout.cpp:21-217", "crates/nuxie-runtime/src/artboard.rs:339-369", "crates/nuxie-runtime/src/artboard.rs:3491-3537"]}
  push_vs_poll: {status: divergent, cpp_pushes: true, evidence: ["src/nested_artboard_layout.cpp:102-217", "crates/nuxie-runtime/src/artboard.rs:3491-3537"]}
  update_ordering: {status: divergent, phases_cpp: ["transfer retained Yoga node", "direct child update", "host dirt"], phases_rust: ["compare parent/child layout generations", "refresh child constraint snapshot", "write child dimensions", "record transfer key"]}
  ownership: {status: mixed, evidence: ["src/nested_artboard_layout.cpp:21-101", "crates/nuxie-runtime/src/artboard.rs:339-369"]}
  compensation:
    status: divergent
    mechanisms:
      - {name: "nested_layout_data_transfer_key", kind: "AF-8 cross-representation generation key", mutation_gated: true, cpp_counterpart: none, evidence: ["crates/nuxie-runtime/src/artboard.rs:3491-3498", "crates/nuxie-runtime/src/artboard.rs:3510-3537"]}
      - {name: "nested_child_constraint_snapshot_refresh", kind: "AF-1 copied layout snapshot refresh", mutation_gated: true, cpp_counterpart: none, evidence: ["crates/nuxie-runtime/src/artboard.rs:3510-3517", "crates/nuxie-runtime/src/artboard.rs:3758-3780"]}
    import_time_constants: []
idiom_rules_invoked: []
confidence: high
notes: "Rust retains the child occurrence, but the parent/child layout split is reconciled by a cycle-written key and refreshed copied bounds rather than C++'s transferred live layout node."
~~~

## B6-0305

~~~yaml
row_id: B6-0305
cpp_files: ["src/nested_artboard_leaf.cpp"]
rust_module: "crates/nuxie-runtime/src/lib.rs"
subsystem_cluster: misc-core
sibling_files_swept: ["src/nested_artboard_layout.cpp", "src/nested_artboard_origin.cpp", "crates/nuxie-runtime/src/artboard.rs", "crates/nuxie-runtime/src/draw.rs"]
verdict: ADAPTED
axes:
  retained_identity: {status: adapted, idiom_rule: "AF-1 arena id", evidence: ["src/nested_artboard_leaf.cpp:8-42", "crates/nuxie-runtime/src/draw.rs:15385-15430"]}
  push_vs_poll: {status: isomorphic, cpp_pushes: false, evidence: ["src/nested_artboard_leaf.cpp:8-42", "crates/nuxie-runtime/src/draw.rs:15345-15430"]}
  update_ordering: {status: isomorphic, phases_cpp: ["read fit/alignment", "compute transform"], phases_rust: ["read fit/alignment", "compute transform"]}
  ownership: {status: adapted, idiom_rule: "AF-7 own-by-value", evidence: ["crates/nuxie-runtime/src/draw.rs:15385-15430"]}
  compensation: {status: adapted, mechanisms: [], import_time_constants: [{name: "fit/alignment property keys", idiom_rule: "AF-5 import-time devirtualization", evidence: ["crates/nuxie-runtime/src/draw.rs:285-292"]}]}
idiom_rules_invoked: ["AF-1 arena id", "AF-5 import-time devirtualization", "AF-7 own-by-value"]
confidence: high
notes: "Leaf alignment is computed directly from current values; nested layout transfer compensation is charged to B6-0304."
~~~

## B6-0306

~~~yaml
row_id: B6-0306
cpp_files: ["src/nested_artboard_origin.cpp"]
rust_module: "crates/nuxie-runtime/src/lib.rs"
subsystem_cluster: misc-core
sibling_files_swept: ["src/nested_artboard_layout.cpp", "src/nested_artboard_leaf.cpp", "crates/nuxie-runtime/src/artboard.rs", "crates/nuxie-runtime/src/draw.rs"]
verdict: ADAPTED
axes:
  retained_identity: {status: adapted, idiom_rule: "AF-1 arena id", evidence: ["src/nested_artboard_origin.cpp:6-20", "crates/nuxie-runtime/src/artboard.rs:4866-4895", "crates/nuxie-runtime/src/artboard.rs:6077-6092"]}
  push_vs_poll: {status: isomorphic, cpp_pushes: true, evidence: ["src/nested_artboard_origin.cpp:6-20", "crates/nuxie-runtime/src/artboard.rs:4866-4895"]}
  update_ordering: {status: isomorphic, phases_cpp: ["origin property change", "write nested instance origin"], phases_rust: ["origin property change", "write nested occurrence origin"]}
  ownership: {status: adapted, idiom_rule: "AF-1 arena id", evidence: ["crates/nuxie-runtime/src/artboard.rs:4866-4895"]}
  compensation: {status: adapted, mechanisms: [], import_time_constants: [{name: "origin property keys", idiom_rule: "AF-5 import-time devirtualization", evidence: ["crates/nuxie-runtime/src/artboard.rs:4866-4895"]}]}
idiom_rules_invoked: ["AF-1 arena id", "AF-5 import-time devirtualization"]
confidence: high
notes: "Origin mutations write the retained nested occurrence directly."
~~~

## B6-0307

~~~yaml
row_id: B6-0307
cpp_files: ["src/node.cpp"]
rust_module: "crates/nuxie-runtime/src/lib.rs"
subsystem_cluster: misc-core
sibling_files_swept: ["src/transform_component.cpp", "src/world_transform_component.cpp", "crates/nuxie-runtime/src/components.rs", "crates/nuxie-runtime/src/artboard.rs"]
verdict: ADAPTED
axes:
  retained_identity: {status: adapted, idiom_rule: "AF-1 arena id", evidence: ["src/node.cpp:9-57", "crates/nuxie-runtime/src/components.rs:477-557"]}
  push_vs_poll: {status: isomorphic, cpp_pushes: true, evidence: ["src/node.cpp:18-57", "crates/nuxie-runtime/src/artboard.rs:3948-4026"]}
  update_ordering: {status: isomorphic, phases_cpp: ["local-transform dirt", "recompute", "world update"], phases_rust: ["transform dirt", "recompute", "world update"]}
  ownership: {status: adapted, idiom_rule: "AF-7 own-by-value", evidence: ["crates/nuxie-runtime/src/components.rs:477-557"]}
  compensation: {status: adapted, mechanisms: [], import_time_constants: [{name: "transform property keys", idiom_rule: "AF-5 import-time devirtualization", evidence: ["crates/nuxie-runtime/src/components.rs:222-237"]}]}
idiom_rules_invoked: ["AF-1 arena id", "AF-5 import-time devirtualization", "AF-7 own-by-value"]
confidence: high
notes: "The local-transform cache/dirt pattern exists in C++ itself; Rust's analogous derived transform state is not compensation."
~~~

## B6-0308

~~~yaml
row_id: B6-0308
cpp_files: ["src/parent_traversal.cpp"]
rust_module: "crates/nuxie-runtime/src/lib.rs"
subsystem_cluster: misc-core
sibling_files_swept: ["src/node.cpp", "crates/nuxie-runtime/src/components.rs", "crates/nuxie-runtime/src/artboard.rs", "crates/nuxie-runtime/src/draw.rs"]
verdict: ADAPTED
axes:
  retained_identity: {status: adapted, idiom_rule: "AF-1 arena id", evidence: ["src/parent_traversal.cpp:9-61", "crates/nuxie-runtime/src/components.rs:477-514"]}
  push_vs_poll: {status: isomorphic, cpp_pushes: false, evidence: ["src/parent_traversal.cpp:9-61", "crates/nuxie-runtime/src/draw.rs:4320-4360"]}
  update_ordering: {status: isomorphic, phases_cpp: ["walk retained parent pointers"], phases_rust: ["walk stable parent-local ids"]}
  ownership: {status: adapted, idiom_rule: "AF-1 arena id", evidence: ["crates/nuxie-runtime/src/components.rs:477-514"]}
  compensation: {status: adapted, mechanisms: [], import_time_constants: [{name: "parent_local topology", idiom_rule: "AF-1 arena id", evidence: ["crates/nuxie-runtime/src/components.rs:477-514"]}]}
idiom_rules_invoked: ["AF-1 arena id"]
confidence: high
notes: "Parent traversal uses stable imported ids; visited guards mirror C++ dependency-cycle protection and do not track mutable drift."
~~~

## B6-0311

~~~yaml
row_id: B6-0311
cpp_files: ["src/renderer.cpp"]
rust_module: "crates/nuxie-runtime/src/lib.rs"
subsystem_cluster: misc-core
sibling_files_swept: ["src/factory.cpp", "src/drawable.cpp", "crates/nuxie-runtime/src/draw.rs", "crates/nuxie-runtime/src/text.rs", "crates/nuxie-runtime/src/scripting.rs"]
verdict: ADAPTED
axes:
  retained_identity: {status: adapted, idiom_rule: "AF-3 trait object for real polymorphism", evidence: ["src/renderer.cpp:7-82", "src/renderer.cpp:102-234", "crates/nuxie-runtime/src/draw.rs:1-180"]}
  push_vs_poll: {status: isomorphic, cpp_pushes: false, evidence: ["src/renderer.cpp:7-82", "crates/nuxie-runtime/src/draw.rs:1-180"]}
  update_ordering: {status: adapted, phases_cpp: ["prepare render data", "draw"], phases_rust: ["prepare command data", "renderer trait draw"]}
  ownership: {status: adapted, idiom_rule: "AF-3 trait object for real polymorphism", evidence: ["crates/nuxie-runtime/src/scripting.rs:109-180"]}
  compensation: {status: adapted, mechanisms: [], import_time_constants: []}
idiom_rules_invoked: ["AF-3 trait object for real polymorphism"]
confidence: medium
notes: "The C++ renderer file spans alignment, buffers, and text helpers that Rust splits across draw/text/backend traits; no extra authoritative-state reconciliation was found for the facade row."
~~~

## B6-0312

~~~yaml
row_id: B6-0312
cpp_files: ["src/resetting_component.cpp"]
rust_module: "crates/nuxie-runtime/src/lib.rs"
subsystem_cluster: misc-core
sibling_files_swept: ["src/advancing_component.cpp", "crates/nuxie-runtime/src/components.rs", "crates/nuxie-runtime/src/artboard.rs", "crates/nuxie-runtime/src/state_machine.rs"]
verdict: ADAPTED
axes:
  retained_identity: {status: adapted, idiom_rule: "AF-1 arena id", evidence: ["src/resetting_component.cpp:12-25", "crates/nuxie-runtime/src/components.rs:477-514"]}
  push_vs_poll: {status: isomorphic, cpp_pushes: false, evidence: ["src/resetting_component.cpp:12-25", "crates/nuxie-runtime/src/artboard.rs:4389-4422"]}
  update_ordering: {status: adapted, phases_cpp: ["type dispatch", "reset"], phases_rust: ["type-tag dispatch", "reset"]}
  ownership: {status: adapted, idiom_rule: "AF-7 own-by-value", evidence: ["crates/nuxie-runtime/src/components.rs:477-514"]}
  compensation: {status: adapted, mechanisms: [], import_time_constants: [{name: "component type tag", idiom_rule: "AF-5 import-time devirtualization", evidence: ["crates/nuxie-runtime/src/components.rs:477-514"]}]}
idiom_rules_invoked: ["AF-1 arena id", "AF-5 import-time devirtualization", "AF-7 own-by-value"]
confidence: high
notes: "Virtual reset dispatch is devirtualized at import; no cycle-time repair state was found."
~~~

## B6-0313

~~~yaml
row_id: B6-0313
cpp_files: ["src/scene.cpp"]
rust_module: "crates/nuxie-runtime/src/lib.rs"
subsystem_cluster: misc-core
sibling_files_swept: ["src/event.cpp", "src/renderer.cpp", "crates/nuxie-runtime/src/artboard.rs", "crates/nuxie-runtime/src/draw.rs", "crates/nuxie-runtime/src/state_machine/instance.rs"]
verdict: ADAPTED
axes:
  retained_identity: {status: adapted, idiom_rule: "AF-6 explicit deep clone", evidence: ["src/scene.cpp:7-40", "crates/nuxie-runtime/src/artboard.rs:220-321"]}
  push_vs_poll: {status: isomorphic, cpp_pushes: true, evidence: ["src/scene.cpp:20-40", "crates/nuxie-runtime/src/state_machine/instance.rs:4958-5014"]}
  update_ordering: {status: isomorphic, phases_cpp: ["advance/update", "draw/report"], phases_rust: ["advance/update", "draw/report"]}
  ownership: {status: adapted, idiom_rule: "AF-6 explicit deep clone", evidence: ["crates/nuxie-runtime/src/artboard.rs:665-704"]}
  compensation: {status: adapted, mechanisms: [], import_time_constants: []}
idiom_rules_invoked: ["AF-6 Deep copy is explicit"]
confidence: high
notes: "Scene is a thin owned occurrence facade. The separate scripting rehydration lifecycle is attributed to ScriptInput rows below."
~~~

## B6-0314

~~~yaml
row_id: B6-0314
cpp_files: ["src/script_input_artboard.cpp"]
rust_module: "crates/nuxie-runtime/src/lib.rs"
subsystem_cluster: misc-core
sibling_files_swept: ["src/script_input_boolean.cpp", "src/script_input_trigger.cpp", "src/script_input_viewmodel_property.cpp", "crates/nuxie-runtime/src/scripting.rs", "crates/nuxie-runtime/src/state_machine/instance.rs", "crates/nuxie/src/lib.rs"]
verdict: DIVERGENT
axes:
  retained_identity: {status: divergent, evidence: ["src/script_input_artboard.cpp:10-134", "crates/nuxie-runtime/src/scripting.rs:1130-1148", "crates/nuxie/src/lib.rs:807-845"]}
  push_vs_poll: {status: divergent, cpp_pushes: true, evidence: ["src/script_input_artboard.cpp:54-134", "crates/nuxie/src/lib.rs:654-676", "crates/nuxie/src/lib.rs:807-845"]}
  update_ordering: {status: divergent, phases_cpp: ["data-bind property change", "direct retained script input write"], phases_rust: ["scene rebind", "rescan input binding", "construct artboard userdata", "hydrate script table"]}
  ownership: {status: divergent, evidence: ["src/script_input_artboard.cpp:10-134", "crates/nuxie/src/lib.rs:816-842"]}
  compensation:
    status: divergent
    mechanisms:
      - {name: "scene_rebind_artboard_input_rehydration", kind: "AF-2/AF-8 rescan and table rehydrate", mutation_gated: true, cpp_counterpart: none, evidence: ["crates/nuxie/src/lib.rs:654-676", "crates/nuxie/src/lib.rs:807-845", "crates/nuxie/src/flow_session.rs:1239-1245"]}
    import_time_constants:
      - {name: "script input kind/name/global id", idiom_rule: "AF-5 import-time devirtualization", evidence: ["crates/nuxie/src/lib.rs:710-744"]}
idiom_rules_invoked: ["AF-5 import-time devirtualization"]
confidence: high
notes: "Current state is part of #RB-1's retained data-context rebuild and Scene-wide rebind deletion gate; see docs/parity-closeout-status.md:208-225."
~~~

## B6-0315

~~~yaml
row_id: B6-0315
cpp_files: ["src/script_input_boolean.cpp"]
rust_module: "crates/nuxie-runtime/src/lib.rs"
subsystem_cluster: misc-core
sibling_files_swept: ["src/script_input_color.cpp", "src/script_input_number.cpp", "src/script_input_string.cpp", "crates/nuxie-runtime/src/scripting.rs", "crates/nuxie-runtime/src/state_machine/instance.rs", "crates/nuxie/src/lib.rs"]
verdict: DIVERGENT
axes:
  retained_identity: {status: divergent, evidence: ["src/script_input_boolean.cpp:10-66", "crates/nuxie-runtime/src/scripting.rs:1094-1122", "crates/nuxie/src/lib.rs:736-785"]}
  push_vs_poll: {status: divergent, cpp_pushes: true, evidence: ["src/script_input_boolean.cpp:42-66", "crates/nuxie/src/lib.rs:654-676", "crates/nuxie/src/lib.rs:736-785"]}
  update_ordering: {status: divergent, phases_cpp: ["propertyValueChanged", "direct script input write"], phases_rust: ["scene rebind", "scan DataBindContext/path", "hydrate script input"]}
  ownership: {status: mixed, evidence: ["src/script_input_boolean.cpp:10-66", "crates/nuxie-runtime/src/scripting.rs:1094-1122"]}
  compensation: {status: divergent, mechanisms: [{name: "scene_rebind_scalar_input_rehydration", kind: "AF-2/AF-8 binding rescan and copied-value hydrate", mutation_gated: true, cpp_counterpart: none, evidence: ["crates/nuxie-runtime/src/scripting.rs:1094-1122", "crates/nuxie/src/lib.rs:654-676", "crates/nuxie/src/lib.rs:736-785"]}], import_time_constants: [{name: "script input kind/name/default", idiom_rule: "AF-5 import-time devirtualization", evidence: ["crates/nuxie/src/lib.rs:710-785"]}]}
idiom_rules_invoked: ["AF-5 import-time devirtualization"]
confidence: high
notes: "Current state is within #RB-1; see docs/parity-closeout-status.md:208-225."
~~~

## B6-0316

~~~yaml
row_id: B6-0316
cpp_files: ["src/script_input_color.cpp"]
rust_module: "crates/nuxie-runtime/src/lib.rs"
subsystem_cluster: misc-core
sibling_files_swept: ["src/script_input_boolean.cpp", "src/script_input_number.cpp", "src/script_input_string.cpp", "crates/nuxie-runtime/src/scripting.rs", "crates/nuxie-runtime/src/state_machine/instance.rs", "crates/nuxie/src/lib.rs"]
verdict: DIVERGENT
axes:
  retained_identity: {status: divergent, evidence: ["src/script_input_color.cpp:10-66", "crates/nuxie-runtime/src/scripting.rs:1094-1122", "crates/nuxie/src/lib.rs:736-785"]}
  push_vs_poll: {status: divergent, cpp_pushes: true, evidence: ["src/script_input_color.cpp:42-66", "crates/nuxie/src/lib.rs:654-676", "crates/nuxie/src/lib.rs:736-785"]}
  update_ordering: {status: divergent, phases_cpp: ["propertyValueChanged", "direct script input write"], phases_rust: ["scene rebind", "scan DataBindContext/path", "hydrate script input"]}
  ownership: {status: mixed, evidence: ["src/script_input_color.cpp:10-66", "crates/nuxie-runtime/src/scripting.rs:1094-1122"]}
  compensation: {status: divergent, mechanisms: [{name: "scene_rebind_scalar_input_rehydration", kind: "AF-2/AF-8 binding rescan and copied-value hydrate", mutation_gated: true, cpp_counterpart: none, evidence: ["crates/nuxie-runtime/src/scripting.rs:1094-1122", "crates/nuxie/src/lib.rs:654-676", "crates/nuxie/src/lib.rs:736-785"]}], import_time_constants: [{name: "script input kind/name/default", idiom_rule: "AF-5 import-time devirtualization", evidence: ["crates/nuxie/src/lib.rs:710-785"]}]}
idiom_rules_invoked: ["AF-5 import-time devirtualization"]
confidence: high
notes: "Current state is within #RB-1; see docs/parity-closeout-status.md:208-225."
~~~

## B6-0317

~~~yaml
row_id: B6-0317
cpp_files: ["src/script_input_number.cpp"]
rust_module: "crates/nuxie-runtime/src/lib.rs"
subsystem_cluster: misc-core
sibling_files_swept: ["src/script_input_boolean.cpp", "src/script_input_color.cpp", "src/script_input_string.cpp", "crates/nuxie-runtime/src/scripting.rs", "crates/nuxie-runtime/src/state_machine/instance.rs", "crates/nuxie/src/lib.rs"]
verdict: DIVERGENT
axes:
  retained_identity: {status: divergent, evidence: ["src/script_input_number.cpp:10-66", "crates/nuxie-runtime/src/scripting.rs:1094-1122", "crates/nuxie/src/lib.rs:736-785"]}
  push_vs_poll: {status: divergent, cpp_pushes: true, evidence: ["src/script_input_number.cpp:42-66", "crates/nuxie/src/lib.rs:654-676", "crates/nuxie/src/lib.rs:736-785"]}
  update_ordering: {status: divergent, phases_cpp: ["propertyValueChanged", "direct script input write"], phases_rust: ["scene rebind", "scan DataBindContext/path", "hydrate script input"]}
  ownership: {status: mixed, evidence: ["src/script_input_number.cpp:10-66", "crates/nuxie-runtime/src/scripting.rs:1094-1122"]}
  compensation: {status: divergent, mechanisms: [{name: "scene_rebind_scalar_input_rehydration", kind: "AF-2/AF-8 binding rescan and copied-value hydrate", mutation_gated: true, cpp_counterpart: none, evidence: ["crates/nuxie-runtime/src/scripting.rs:1094-1122", "crates/nuxie/src/lib.rs:654-676", "crates/nuxie/src/lib.rs:736-785"]}], import_time_constants: [{name: "script input kind/name/default", idiom_rule: "AF-5 import-time devirtualization", evidence: ["crates/nuxie/src/lib.rs:710-785"]}]}
idiom_rules_invoked: ["AF-5 import-time devirtualization"]
confidence: high
notes: "Current state is within #RB-1; see docs/parity-closeout-status.md:208-225."
~~~

## B6-0318

~~~yaml
row_id: B6-0318
cpp_files: ["src/script_input_string.cpp"]
rust_module: "crates/nuxie-runtime/src/lib.rs"
subsystem_cluster: misc-core
sibling_files_swept: ["src/script_input_boolean.cpp", "src/script_input_color.cpp", "src/script_input_number.cpp", "crates/nuxie-runtime/src/scripting.rs", "crates/nuxie-runtime/src/state_machine/instance.rs", "crates/nuxie/src/lib.rs"]
verdict: DIVERGENT
axes:
  retained_identity: {status: divergent, evidence: ["src/script_input_string.cpp:10-66", "crates/nuxie-runtime/src/scripting.rs:1094-1122", "crates/nuxie/src/lib.rs:736-785"]}
  push_vs_poll: {status: divergent, cpp_pushes: true, evidence: ["src/script_input_string.cpp:42-66", "crates/nuxie/src/lib.rs:654-676", "crates/nuxie/src/lib.rs:736-785"]}
  update_ordering: {status: divergent, phases_cpp: ["propertyValueChanged", "direct script input write"], phases_rust: ["scene rebind", "scan DataBindContext/path", "hydrate script input"]}
  ownership: {status: mixed, evidence: ["src/script_input_string.cpp:10-66", "crates/nuxie-runtime/src/scripting.rs:1094-1122"]}
  compensation: {status: divergent, mechanisms: [{name: "scene_rebind_scalar_input_rehydration", kind: "AF-2/AF-8 binding rescan and copied-value hydrate", mutation_gated: true, cpp_counterpart: none, evidence: ["crates/nuxie-runtime/src/scripting.rs:1094-1122", "crates/nuxie/src/lib.rs:654-676", "crates/nuxie/src/lib.rs:736-785"]}], import_time_constants: [{name: "script input kind/name/default", idiom_rule: "AF-5 import-time devirtualization", evidence: ["crates/nuxie/src/lib.rs:710-785"]}]}
idiom_rules_invoked: ["AF-5 import-time devirtualization"]
confidence: high
notes: "Current state is within #RB-1; see docs/parity-closeout-status.md:208-225."
~~~

## B6-0319

~~~yaml
row_id: B6-0319
cpp_files: ["src/script_input_trigger.cpp"]
rust_module: "crates/nuxie-runtime/src/lib.rs"
subsystem_cluster: misc-core
sibling_files_swept: ["src/script_input_boolean.cpp", "src/script_input_viewmodel_property.cpp", "crates/nuxie-runtime/src/scripting.rs", "crates/nuxie-runtime/src/state_machine/instance.rs", "crates/nuxie/src/lib.rs"]
verdict: DIVERGENT
axes:
  retained_identity: {status: divergent, evidence: ["src/script_input_trigger.cpp:10-65", "crates/nuxie-runtime/src/scripting.rs:1153-1169", "crates/nuxie/src/lib.rs:786-806"]}
  push_vs_poll: {status: divergent, cpp_pushes: true, evidence: ["src/script_input_trigger.cpp:60-65", "crates/nuxie/src/lib.rs:654-676", "crates/nuxie/src/lib.rs:786-806"]}
  update_ordering: {status: divergent, phases_cpp: ["trigger property callback", "direct trigger function call"], phases_rust: ["scene rebind", "resolve current and previous counts", "diff", "hydrate trigger"]}
  ownership: {status: divergent, evidence: ["src/script_input_trigger.cpp:10-65", "crates/nuxie/src/lib.rs:786-806"]}
  compensation:
    status: divergent
    mechanisms:
      - {name: "scene_rebind_trigger_rehydration", kind: "AF-2/AF-8 binding rescan", mutation_gated: true, cpp_counterpart: none, evidence: ["crates/nuxie-runtime/src/scripting.rs:1153-1169", "crates/nuxie/src/lib.rs:654-676", "crates/nuxie/src/lib.rs:786-806"]}
      - {name: "previous_root_trigger_count_diff", kind: "AF-2 copied previous/current state comparison", mutation_gated: true, cpp_counterpart: none, evidence: ["crates/nuxie/src/lib.rs:790-806"]}
    import_time_constants:
      - {name: "script trigger name/global id", idiom_rule: "AF-5 import-time devirtualization", evidence: ["crates/nuxie/src/lib.rs:710-744"]}
idiom_rules_invoked: ["AF-5 import-time devirtualization"]
confidence: high
notes: "Current state is within #RB-1; see docs/parity-closeout-status.md:208-225."
~~~

## B6-0320

~~~yaml
row_id: B6-0320
cpp_files: ["src/script_input_viewmodel_property.cpp"]
rust_module: "crates/nuxie-runtime/src/lib.rs"
subsystem_cluster: misc-core
sibling_files_swept: ["src/data_bind_path_referencer.cpp", "src/script_input_trigger.cpp", "crates/nuxie-runtime/src/scripting.rs", "crates/nuxie-runtime/src/view_model.rs", "crates/nuxie-runtime/src/state_machine/instance.rs", "crates/nuxie/src/lib.rs"]
verdict: DIVERGENT
axes:
  retained_identity: {status: mixed, evidence: ["src/script_input_viewmodel_property.cpp:20-155", "crates/nuxie-runtime/src/scripting.rs:1333-1375", "crates/nuxie/src/lib.rs:846-872"]}
  push_vs_poll: {status: divergent, cpp_pushes: true, evidence: ["src/script_input_viewmodel_property.cpp:69-155", "crates/nuxie/src/lib.rs:654-676", "crates/nuxie/src/lib.rs:846-872"]}
  update_ordering: {status: divergent, phases_cpp: ["resolve retained ViewModelInstanceValue", "direct script table write"], phases_rust: ["scene rebind", "resolve retained scoped handle", "rehydrate script table"]}
  ownership: {status: adapted, idiom_rule: "AF-1 Rc/RefCell retained handle", evidence: ["crates/nuxie-runtime/src/scripting.rs:1333-1355"]}
  compensation:
    status: divergent
    mechanisms:
      - {name: "scene_rebind_view_model_input_rehydration", kind: "AF-8 facade rehydrate lifecycle", mutation_gated: true, cpp_counterpart: none, evidence: ["crates/nuxie-runtime/src/scripting.rs:1333-1355", "crates/nuxie/src/lib.rs:654-676", "crates/nuxie/src/lib.rs:846-872"]}
    import_time_constants:
      - {name: "resolved source path ids", idiom_rule: "AF-5 import-time devirtualization", evidence: ["crates/nuxie-runtime/src/scripting.rs:1338-1348"]}
idiom_rules_invoked: ["AF-1 Rc/RefCell retained handle", "AF-5 import-time devirtualization"]
confidence: high
notes: "The nested value now uses retained handles, but the surrounding Scene-wide rehydrate lifecycle remains. Current state is within #RB-1; see docs/parity-closeout-status.md:208-225."
~~~

## B6-0375

~~~yaml
row_id: B6-0375
cpp_files: ["src/simple_array.cpp"]
rust_module: "crates/nuxie-runtime/src/lib.rs"
subsystem_cluster: misc-core
sibling_files_swept: ["crates/nuxie-runtime/src/lib.rs", "crates/nuxie-runtime/src/artboard.rs", "crates/nuxie-runtime/src/components.rs", "crates/nuxie-runtime/src/draw.rs", "crates/nuxie-runtime/src/objects.rs", "crates/nuxie-runtime/src/text.rs"]
verdict: UNKNOWN
axes:
  retained_identity: {status: unknown, evidence: ["src/simple_array.cpp:1-18"]}
  push_vs_poll: {status: unknown, cpp_pushes: false, evidence: ["src/simple_array.cpp:1-18"]}
  update_ordering: {status: unknown, phases_cpp: ["TESTING-only static counter access"], phases_rust: ["no mapped production or test-only SimpleArray counterpart found"]}
  ownership: {status: unknown, evidence: ["src/simple_array.cpp:1-18"]}
  compensation: {status: unknown, mechanisms: [], import_time_constants: []}
idiom_rules_invoked: []
confidence: high
notes: "Blocker: the assigned C++ translation unit contains only TESTING-only SimpleArray allocation counters, while the mapped Rust facade and crate-wide family/sibling sweep contain no identifiable counterpart. No structural verdict is guessed."
~~~

## B6-0376

~~~yaml
row_id: B6-0376
cpp_files: ["src/solo.cpp"]
rust_module: "crates/nuxie-runtime/src/lib.rs"
subsystem_cluster: misc-core
sibling_files_swept: ["src/container_component.cpp", "crates/nuxie-runtime/src/components.rs", "crates/nuxie-runtime/src/artboard.rs", "crates/nuxie-runtime/src/artboard_data_bind.rs"]
verdict: ADAPTED
axes:
  retained_identity: {status: adapted, idiom_rule: "AF-1 arena id", evidence: ["src/solo.cpp:8-106", "crates/nuxie-runtime/src/components.rs:637-734"]}
  push_vs_poll: {status: isomorphic, cpp_pushes: true, evidence: ["src/solo.cpp:53-105", "crates/nuxie-runtime/src/artboard.rs:5382-5430"]}
  update_ordering: {status: isomorphic, phases_cpp: ["activeComponent change", "collapse children"], phases_rust: ["activeComponent change", "collapse child ids"]}
  ownership: {status: adapted, idiom_rule: "AF-7 own-by-value", evidence: ["crates/nuxie-runtime/src/components.rs:637-734"]}
  compensation: {status: adapted, mechanisms: [], import_time_constants: [{name: "solo child ids/collapse descriptors", idiom_rule: "AF-5 import-time devirtualization", evidence: ["crates/nuxie-runtime/src/components.rs:698-734"]}]}
idiom_rules_invoked: ["AF-1 arena id", "AF-5 import-time devirtualization", "AF-7 own-by-value"]
confidence: high
notes: "Solo uses immutable child ids and direct collapse propagation; data-bind source handling is attributed to the data-bind family."
~~~

## B6-0408

~~~yaml
row_id: B6-0408
cpp_files: ["src/transform_component.cpp"]
rust_module: "crates/nuxie-runtime/src/lib.rs"
subsystem_cluster: misc-core
sibling_files_swept: ["src/node.cpp", "src/world_transform_component.cpp", "crates/nuxie-runtime/src/components.rs", "crates/nuxie-runtime/src/artboard.rs", "crates/nuxie-runtime/src/properties.rs"]
verdict: ADAPTED
axes:
  retained_identity: {status: adapted, idiom_rule: "AF-1 arena id", evidence: ["src/transform_component.cpp:9-126", "crates/nuxie-runtime/src/components.rs:477-557"]}
  push_vs_poll: {status: isomorphic, cpp_pushes: true, evidence: ["src/transform_component.cpp:42-126", "crates/nuxie-runtime/src/artboard.rs:3948-4026"]}
  update_ordering: {status: isomorphic, phases_cpp: ["property dirt", "local transform", "world transform", "dependents"], phases_rust: ["property dirt", "local transform", "world transform", "dependent ids"]}
  ownership: {status: adapted, idiom_rule: "AF-7 own-by-value", evidence: ["crates/nuxie-runtime/src/components.rs:477-557"]}
  compensation: {status: adapted, mechanisms: [], import_time_constants: [{name: "transform property keys/dependent ids", idiom_rule: "AF-5 import-time devirtualization", evidence: ["crates/nuxie-runtime/src/components.rs:222-237", "crates/nuxie-runtime/src/components.rs:560-633"]}]}
idiom_rules_invoked: ["AF-1 arena id", "AF-5 import-time devirtualization", "AF-7 own-by-value"]
confidence: high
notes: "The transform dirt/update pipeline is direct; by-value matrices and stable dependent ids are accepted Rust representations."
~~~

## B6-0446

~~~yaml
row_id: B6-0446
cpp_files: ["src/virtualizing_component.cpp"]
rust_module: "crates/nuxie-runtime/src/lib.rs"
subsystem_cluster: misc-core
sibling_files_swept: ["src/artboard_component_list.cpp", "crates/nuxie-runtime/src/artboard.rs", "crates/nuxie-runtime/src/components.rs"]
verdict: ADAPTED
axes:
  retained_identity: {status: adapted, idiom_rule: "AF-1 arena id", evidence: ["src/virtualizing_component.cpp:8-15", "crates/nuxie-runtime/src/artboard.rs:2843-2920"]}
  push_vs_poll: {status: adapted, cpp_pushes: false, evidence: ["src/virtualizing_component.cpp:8-15", "crates/nuxie-runtime/src/artboard.rs:2843-2920"]}
  update_ordering: {status: adapted, phases_cpp: ["virtualize dispatch"], phases_rust: ["type-tag virtual-window settlement"]}
  ownership: {status: adapted, idiom_rule: "AF-7 own-by-value", evidence: ["crates/nuxie-runtime/src/artboard.rs:2843-2920"]}
  compensation: {status: adapted, mechanisms: [], import_time_constants: [{name: "virtualizing component type/configuration", idiom_rule: "AF-5 import-time devirtualization", evidence: ["crates/nuxie-runtime/src/artboard.rs:2843-2920"]}]}
idiom_rules_invoked: ["AF-1 arena id", "AF-5 import-time devirtualization", "AF-7 own-by-value"]
confidence: high
notes: "The interface dispatch is devirtualized; component-list source reconciliation is charged to B6-0095 rather than this marker interface row."
~~~

## B6-0447

~~~yaml
row_id: B6-0447
cpp_files: ["src/world_transform_component.cpp"]
rust_module: "crates/nuxie-runtime/src/lib.rs"
subsystem_cluster: misc-core
sibling_files_swept: ["src/node.cpp", "src/transform_component.cpp", "crates/nuxie-runtime/src/components.rs", "crates/nuxie-runtime/src/artboard.rs", "crates/nuxie-runtime/src/draw.rs"]
verdict: ADAPTED
axes:
  retained_identity: {status: adapted, idiom_rule: "AF-1 arena id", evidence: ["src/world_transform_component.cpp:8-28", "crates/nuxie-runtime/src/components.rs:477-557"]}
  push_vs_poll: {status: isomorphic, cpp_pushes: true, evidence: ["src/world_transform_component.cpp:8-28", "crates/nuxie-runtime/src/artboard.rs:3948-4026"]}
  update_ordering: {status: isomorphic, phases_cpp: ["world-transform dirt", "update dependents"], phases_rust: ["world-transform dirt", "update dependent ids"]}
  ownership: {status: adapted, idiom_rule: "AF-1 arena id", evidence: ["crates/nuxie-runtime/src/components.rs:477-557"]}
  compensation: {status: adapted, mechanisms: [], import_time_constants: [{name: "parent/dependent ids", idiom_rule: "AF-1 arena id", evidence: ["crates/nuxie-runtime/src/components.rs:477-514"]}]}
idiom_rules_invoked: ["AF-1 arena id"]
confidence: high
notes: "World-transform dependency propagation remains within the canonical component dirt model."
~~~
