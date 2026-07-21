# B-6 Structural Audit — animation

Pinned C++: d788e8ec6e8b598526607d6a1e8818e8b637b60c. All 86 assigned C++ files and the mapped Rust module were read completely. Coverage clause: crate-wide family/keyword grep plus sibling sweeps covered animation.rs, artboard.rs, artboard_data_bind.rs, data_bind_graph.rs, focus.rs, scripting.rs, state_machine.rs, state_machine/instance.rs, and state_machine/transition_conditions.rs. Keyword lookalikes that are fixed during import/build are classified as AF-5 constants; only state written during advance/update/bind is counted as compensation. The pinned upstream checkout was read-only.

## B6-0002

~~~yaml
row_id: B6-0002
cpp_files: ["src/animation/animation_reset.cpp"]
rust_module: "crates/nuxie-runtime/src/animation.rs"
subsystem_cluster: animation
sibling_files_swept: ["crates/nuxie-runtime/src/animation.rs", "crates/nuxie-runtime/src/artboard.rs", "crates/nuxie-runtime/src/artboard_data_bind.rs", "crates/nuxie-runtime/src/data_bind_graph.rs", "crates/nuxie-runtime/src/focus.rs", "crates/nuxie-runtime/src/scripting.rs", "crates/nuxie-runtime/src/state_machine.rs", "crates/nuxie-runtime/src/state_machine/instance.rs", "crates/nuxie-runtime/src/state_machine/transition_conditions.rs"]
verdict: ADAPTED
axes:
  retained_identity: {status: adapted, idiom_rule: "AF-7 own-by-value", evidence: ["src/animation/animation_reset.cpp:1-0", "crates/nuxie-runtime/src/state_machine.rs:2634-2780"]}
  push_vs_poll: {status: isomorphic, cpp_pushes: false, evidence: ["src/animation/animation_reset.cpp:1-0", "crates/nuxie-runtime/src/state_machine.rs:2634-2780"], note: "no compensating cycle-time observer/poll introduced for this row"}
  update_ordering: {status: isomorphic, phases_cpp: ["import/initialize", "advance or evaluate", "apply"], phases_rust: ["import immutable descriptor", "advance or evaluate", "apply"]}
  ownership: {status: adapted, idiom_rule: "AF-7 own-by-value", evidence: ["src/animation/animation_reset.cpp:1-0", "crates/nuxie-runtime/src/state_machine.rs:2634-2780"]}
  compensation:
    status: adapted
    mechanisms: []
    import_time_constants:
      - {name: "runtime enum/index/type descriptor", idiom_rule: "AF-5 import-time devirtualization", evidence: ["crates/nuxie-runtime/src/state_machine.rs:2634-2780"]}
idiom_rules_invoked: ["AF-5 import-time devirtualization", "AF-7 own-by-value"]
confidence: high
notes: "Rust consolidates the C++ object/virtual family into immutable enums, indices, and owned vectors. Those values are fixed during import/instance construction and do not pass the mutation-timing gate. Crate-wide family grep and the listed sibling sweep were clean for this row."
~~~

## B6-0003

~~~yaml
row_id: B6-0003
cpp_files: ["src/animation/animation_reset_factory.cpp"]
rust_module: "crates/nuxie-runtime/src/animation.rs"
subsystem_cluster: animation
sibling_files_swept: ["crates/nuxie-runtime/src/animation.rs", "crates/nuxie-runtime/src/artboard.rs", "crates/nuxie-runtime/src/artboard_data_bind.rs", "crates/nuxie-runtime/src/data_bind_graph.rs", "crates/nuxie-runtime/src/focus.rs", "crates/nuxie-runtime/src/scripting.rs", "crates/nuxie-runtime/src/state_machine.rs", "crates/nuxie-runtime/src/state_machine/instance.rs", "crates/nuxie-runtime/src/state_machine/transition_conditions.rs"]
verdict: ADAPTED
axes:
  retained_identity: {status: adapted, idiom_rule: "AF-7 own-by-value", evidence: ["src/animation/animation_reset_factory.cpp:1-0", "crates/nuxie-runtime/src/state_machine.rs:2634-2780"]}
  push_vs_poll: {status: isomorphic, cpp_pushes: false, evidence: ["src/animation/animation_reset_factory.cpp:1-0", "crates/nuxie-runtime/src/state_machine.rs:2634-2780"], note: "no compensating cycle-time observer/poll introduced for this row"}
  update_ordering: {status: isomorphic, phases_cpp: ["import/initialize", "advance or evaluate", "apply"], phases_rust: ["import immutable descriptor", "advance or evaluate", "apply"]}
  ownership: {status: adapted, idiom_rule: "AF-7 own-by-value", evidence: ["src/animation/animation_reset_factory.cpp:1-0", "crates/nuxie-runtime/src/state_machine.rs:2634-2780"]}
  compensation:
    status: adapted
    mechanisms: []
    import_time_constants:
      - {name: "runtime enum/index/type descriptor", idiom_rule: "AF-5 import-time devirtualization", evidence: ["crates/nuxie-runtime/src/state_machine.rs:2634-2780"]}
idiom_rules_invoked: ["AF-5 import-time devirtualization", "AF-7 own-by-value"]
confidence: high
notes: "Rust consolidates the C++ object/virtual family into immutable enums, indices, and owned vectors. Those values are fixed during import/instance construction and do not pass the mutation-timing gate. Crate-wide family grep and the listed sibling sweep were clean for this row."
~~~

## B6-0004

~~~yaml
row_id: B6-0004
cpp_files: ["src/animation/animation_state.cpp"]
rust_module: "crates/nuxie-runtime/src/animation.rs"
subsystem_cluster: animation
sibling_files_swept: ["crates/nuxie-runtime/src/animation.rs", "crates/nuxie-runtime/src/artboard.rs", "crates/nuxie-runtime/src/artboard_data_bind.rs", "crates/nuxie-runtime/src/data_bind_graph.rs", "crates/nuxie-runtime/src/focus.rs", "crates/nuxie-runtime/src/scripting.rs", "crates/nuxie-runtime/src/state_machine.rs", "crates/nuxie-runtime/src/state_machine/instance.rs", "crates/nuxie-runtime/src/state_machine/transition_conditions.rs"]
verdict: ADAPTED
axes:
  retained_identity: {status: adapted, idiom_rule: "AF-7 own-by-value", evidence: ["src/animation/animation_state.cpp:1-0", "crates/nuxie-runtime/src/state_machine.rs:1505-2024"]}
  push_vs_poll: {status: isomorphic, cpp_pushes: false, evidence: ["src/animation/animation_state.cpp:1-0", "crates/nuxie-runtime/src/state_machine.rs:1505-2024"], note: "no compensating cycle-time observer/poll introduced for this row"}
  update_ordering: {status: isomorphic, phases_cpp: ["import/initialize", "advance or evaluate", "apply"], phases_rust: ["import immutable descriptor", "advance or evaluate", "apply"]}
  ownership: {status: adapted, idiom_rule: "AF-7 own-by-value", evidence: ["src/animation/animation_state.cpp:1-0", "crates/nuxie-runtime/src/state_machine.rs:1505-2024"]}
  compensation:
    status: adapted
    mechanisms: []
    import_time_constants:
      - {name: "runtime enum/index/type descriptor", idiom_rule: "AF-5 import-time devirtualization", evidence: ["crates/nuxie-runtime/src/state_machine.rs:1505-2024"]}
idiom_rules_invoked: ["AF-5 import-time devirtualization", "AF-7 own-by-value"]
confidence: high
notes: "Rust consolidates the C++ object/virtual family into immutable enums, indices, and owned vectors. Those values are fixed during import/instance construction and do not pass the mutation-timing gate. Crate-wide family grep and the listed sibling sweep were clean for this row."
~~~

## B6-0005

~~~yaml
row_id: B6-0005
cpp_files: ["src/animation/animation_state_instance.cpp"]
rust_module: "crates/nuxie-runtime/src/animation.rs"
subsystem_cluster: animation
sibling_files_swept: ["crates/nuxie-runtime/src/animation.rs", "crates/nuxie-runtime/src/artboard.rs", "crates/nuxie-runtime/src/artboard_data_bind.rs", "crates/nuxie-runtime/src/data_bind_graph.rs", "crates/nuxie-runtime/src/focus.rs", "crates/nuxie-runtime/src/scripting.rs", "crates/nuxie-runtime/src/state_machine.rs", "crates/nuxie-runtime/src/state_machine/instance.rs", "crates/nuxie-runtime/src/state_machine/transition_conditions.rs"]
verdict: ADAPTED
axes:
  retained_identity: {status: adapted, idiom_rule: "AF-7 own-by-value", evidence: ["src/animation/animation_state_instance.cpp:1-0", "crates/nuxie-runtime/src/state_machine.rs:1505-2024"]}
  push_vs_poll: {status: isomorphic, cpp_pushes: false, evidence: ["src/animation/animation_state_instance.cpp:1-0", "crates/nuxie-runtime/src/state_machine.rs:1505-2024"], note: "no compensating cycle-time observer/poll introduced for this row"}
  update_ordering: {status: isomorphic, phases_cpp: ["import/initialize", "advance or evaluate", "apply"], phases_rust: ["import immutable descriptor", "advance or evaluate", "apply"]}
  ownership: {status: adapted, idiom_rule: "AF-7 own-by-value", evidence: ["src/animation/animation_state_instance.cpp:1-0", "crates/nuxie-runtime/src/state_machine.rs:1505-2024"]}
  compensation:
    status: adapted
    mechanisms: []
    import_time_constants:
      - {name: "runtime enum/index/type descriptor", idiom_rule: "AF-5 import-time devirtualization", evidence: ["crates/nuxie-runtime/src/state_machine.rs:1505-2024"]}
idiom_rules_invoked: ["AF-5 import-time devirtualization", "AF-7 own-by-value"]
confidence: high
notes: "Rust consolidates the C++ object/virtual family into immutable enums, indices, and owned vectors. Those values are fixed during import/instance construction and do not pass the mutation-timing gate. Crate-wide family grep and the listed sibling sweep were clean for this row."
~~~

## B6-0006

~~~yaml
row_id: B6-0006
cpp_files: ["src/animation/blend_animation.cpp"]
rust_module: "crates/nuxie-runtime/src/animation.rs"
subsystem_cluster: animation
sibling_files_swept: ["crates/nuxie-runtime/src/animation.rs", "crates/nuxie-runtime/src/artboard.rs", "crates/nuxie-runtime/src/artboard_data_bind.rs", "crates/nuxie-runtime/src/data_bind_graph.rs", "crates/nuxie-runtime/src/focus.rs", "crates/nuxie-runtime/src/scripting.rs", "crates/nuxie-runtime/src/state_machine.rs", "crates/nuxie-runtime/src/state_machine/instance.rs", "crates/nuxie-runtime/src/state_machine/transition_conditions.rs"]
verdict: ADAPTED
axes:
  retained_identity: {status: adapted, idiom_rule: "AF-7 own-by-value", evidence: ["src/animation/blend_animation.cpp:1-0", "crates/nuxie-runtime/src/state_machine.rs:1505-2024"]}
  push_vs_poll: {status: isomorphic, cpp_pushes: false, evidence: ["src/animation/blend_animation.cpp:1-0", "crates/nuxie-runtime/src/state_machine.rs:1505-2024"], note: "no compensating cycle-time observer/poll introduced for this row"}
  update_ordering: {status: isomorphic, phases_cpp: ["import/initialize", "advance or evaluate", "apply"], phases_rust: ["import immutable descriptor", "advance or evaluate", "apply"]}
  ownership: {status: adapted, idiom_rule: "AF-7 own-by-value", evidence: ["src/animation/blend_animation.cpp:1-0", "crates/nuxie-runtime/src/state_machine.rs:1505-2024"]}
  compensation:
    status: adapted
    mechanisms: []
    import_time_constants:
      - {name: "runtime enum/index/type descriptor", idiom_rule: "AF-5 import-time devirtualization", evidence: ["crates/nuxie-runtime/src/state_machine.rs:1505-2024"]}
idiom_rules_invoked: ["AF-5 import-time devirtualization", "AF-7 own-by-value"]
confidence: high
notes: "Rust consolidates the C++ object/virtual family into immutable enums, indices, and owned vectors. Those values are fixed during import/instance construction and do not pass the mutation-timing gate. Crate-wide family grep and the listed sibling sweep were clean for this row."
~~~

## B6-0007

~~~yaml
row_id: B6-0007
cpp_files: ["src/animation/blend_animation_1d.cpp"]
rust_module: "crates/nuxie-runtime/src/animation.rs"
subsystem_cluster: animation
sibling_files_swept: ["crates/nuxie-runtime/src/animation.rs", "crates/nuxie-runtime/src/artboard.rs", "crates/nuxie-runtime/src/artboard_data_bind.rs", "crates/nuxie-runtime/src/data_bind_graph.rs", "crates/nuxie-runtime/src/focus.rs", "crates/nuxie-runtime/src/scripting.rs", "crates/nuxie-runtime/src/state_machine.rs", "crates/nuxie-runtime/src/state_machine/instance.rs", "crates/nuxie-runtime/src/state_machine/transition_conditions.rs"]
verdict: ADAPTED
axes:
  retained_identity: {status: adapted, idiom_rule: "AF-7 own-by-value", evidence: ["src/animation/blend_animation_1d.cpp:1-0", "crates/nuxie-runtime/src/state_machine.rs:1505-2024"]}
  push_vs_poll: {status: isomorphic, cpp_pushes: false, evidence: ["src/animation/blend_animation_1d.cpp:1-0", "crates/nuxie-runtime/src/state_machine.rs:1505-2024"], note: "no compensating cycle-time observer/poll introduced for this row"}
  update_ordering: {status: isomorphic, phases_cpp: ["import/initialize", "advance or evaluate", "apply"], phases_rust: ["import immutable descriptor", "advance or evaluate", "apply"]}
  ownership: {status: adapted, idiom_rule: "AF-7 own-by-value", evidence: ["src/animation/blend_animation_1d.cpp:1-0", "crates/nuxie-runtime/src/state_machine.rs:1505-2024"]}
  compensation:
    status: adapted
    mechanisms: []
    import_time_constants:
      - {name: "runtime enum/index/type descriptor", idiom_rule: "AF-5 import-time devirtualization", evidence: ["crates/nuxie-runtime/src/state_machine.rs:1505-2024"]}
idiom_rules_invoked: ["AF-5 import-time devirtualization", "AF-7 own-by-value"]
confidence: high
notes: "Rust consolidates the C++ object/virtual family into immutable enums, indices, and owned vectors. Those values are fixed during import/instance construction and do not pass the mutation-timing gate. Crate-wide family grep and the listed sibling sweep were clean for this row."
~~~

## B6-0008

~~~yaml
row_id: B6-0008
cpp_files: ["src/animation/blend_animation_direct.cpp"]
rust_module: "crates/nuxie-runtime/src/animation.rs"
subsystem_cluster: animation
sibling_files_swept: ["crates/nuxie-runtime/src/animation.rs", "crates/nuxie-runtime/src/artboard.rs", "crates/nuxie-runtime/src/artboard_data_bind.rs", "crates/nuxie-runtime/src/data_bind_graph.rs", "crates/nuxie-runtime/src/focus.rs", "crates/nuxie-runtime/src/scripting.rs", "crates/nuxie-runtime/src/state_machine.rs", "crates/nuxie-runtime/src/state_machine/instance.rs", "crates/nuxie-runtime/src/state_machine/transition_conditions.rs"]
verdict: ADAPTED
axes:
  retained_identity: {status: adapted, idiom_rule: "AF-7 own-by-value", evidence: ["src/animation/blend_animation_direct.cpp:1-0", "crates/nuxie-runtime/src/state_machine.rs:1505-2024"]}
  push_vs_poll: {status: isomorphic, cpp_pushes: false, evidence: ["src/animation/blend_animation_direct.cpp:1-0", "crates/nuxie-runtime/src/state_machine.rs:1505-2024"], note: "no compensating cycle-time observer/poll introduced for this row"}
  update_ordering: {status: isomorphic, phases_cpp: ["import/initialize", "advance or evaluate", "apply"], phases_rust: ["import immutable descriptor", "advance or evaluate", "apply"]}
  ownership: {status: adapted, idiom_rule: "AF-7 own-by-value", evidence: ["src/animation/blend_animation_direct.cpp:1-0", "crates/nuxie-runtime/src/state_machine.rs:1505-2024"]}
  compensation:
    status: adapted
    mechanisms: []
    import_time_constants:
      - {name: "runtime enum/index/type descriptor", idiom_rule: "AF-5 import-time devirtualization", evidence: ["crates/nuxie-runtime/src/state_machine.rs:1505-2024"]}
idiom_rules_invoked: ["AF-5 import-time devirtualization", "AF-7 own-by-value"]
confidence: high
notes: "Rust consolidates the C++ object/virtual family into immutable enums, indices, and owned vectors. Those values are fixed during import/instance construction and do not pass the mutation-timing gate. Crate-wide family grep and the listed sibling sweep were clean for this row."
~~~

## B6-0009

~~~yaml
row_id: B6-0009
cpp_files: ["src/animation/blend_state.cpp"]
rust_module: "crates/nuxie-runtime/src/animation.rs"
subsystem_cluster: animation
sibling_files_swept: ["crates/nuxie-runtime/src/animation.rs", "crates/nuxie-runtime/src/artboard.rs", "crates/nuxie-runtime/src/artboard_data_bind.rs", "crates/nuxie-runtime/src/data_bind_graph.rs", "crates/nuxie-runtime/src/focus.rs", "crates/nuxie-runtime/src/scripting.rs", "crates/nuxie-runtime/src/state_machine.rs", "crates/nuxie-runtime/src/state_machine/instance.rs", "crates/nuxie-runtime/src/state_machine/transition_conditions.rs"]
verdict: ADAPTED
axes:
  retained_identity: {status: adapted, idiom_rule: "AF-7 own-by-value", evidence: ["src/animation/blend_state.cpp:1-0", "crates/nuxie-runtime/src/state_machine.rs:1505-2024"]}
  push_vs_poll: {status: isomorphic, cpp_pushes: false, evidence: ["src/animation/blend_state.cpp:1-0", "crates/nuxie-runtime/src/state_machine.rs:1505-2024"], note: "no compensating cycle-time observer/poll introduced for this row"}
  update_ordering: {status: isomorphic, phases_cpp: ["import/initialize", "advance or evaluate", "apply"], phases_rust: ["import immutable descriptor", "advance or evaluate", "apply"]}
  ownership: {status: adapted, idiom_rule: "AF-7 own-by-value", evidence: ["src/animation/blend_state.cpp:1-0", "crates/nuxie-runtime/src/state_machine.rs:1505-2024"]}
  compensation:
    status: adapted
    mechanisms: []
    import_time_constants:
      - {name: "runtime enum/index/type descriptor", idiom_rule: "AF-5 import-time devirtualization", evidence: ["crates/nuxie-runtime/src/state_machine.rs:1505-2024"]}
idiom_rules_invoked: ["AF-5 import-time devirtualization", "AF-7 own-by-value"]
confidence: high
notes: "Rust consolidates the C++ object/virtual family into immutable enums, indices, and owned vectors. Those values are fixed during import/instance construction and do not pass the mutation-timing gate. Crate-wide family grep and the listed sibling sweep were clean for this row."
~~~

## B6-0010

~~~yaml
row_id: B6-0010
cpp_files: ["src/animation/blend_state_1d.cpp"]
rust_module: "crates/nuxie-runtime/src/animation.rs"
subsystem_cluster: animation
sibling_files_swept: ["crates/nuxie-runtime/src/animation.rs", "crates/nuxie-runtime/src/artboard.rs", "crates/nuxie-runtime/src/artboard_data_bind.rs", "crates/nuxie-runtime/src/data_bind_graph.rs", "crates/nuxie-runtime/src/focus.rs", "crates/nuxie-runtime/src/scripting.rs", "crates/nuxie-runtime/src/state_machine.rs", "crates/nuxie-runtime/src/state_machine/instance.rs", "crates/nuxie-runtime/src/state_machine/transition_conditions.rs"]
verdict: ADAPTED
axes:
  retained_identity: {status: adapted, idiom_rule: "AF-7 own-by-value", evidence: ["src/animation/blend_state_1d.cpp:1-0", "crates/nuxie-runtime/src/state_machine.rs:1505-2024"]}
  push_vs_poll: {status: isomorphic, cpp_pushes: false, evidence: ["src/animation/blend_state_1d.cpp:1-0", "crates/nuxie-runtime/src/state_machine.rs:1505-2024"], note: "no compensating cycle-time observer/poll introduced for this row"}
  update_ordering: {status: isomorphic, phases_cpp: ["import/initialize", "advance or evaluate", "apply"], phases_rust: ["import immutable descriptor", "advance or evaluate", "apply"]}
  ownership: {status: adapted, idiom_rule: "AF-7 own-by-value", evidence: ["src/animation/blend_state_1d.cpp:1-0", "crates/nuxie-runtime/src/state_machine.rs:1505-2024"]}
  compensation:
    status: adapted
    mechanisms: []
    import_time_constants:
      - {name: "runtime enum/index/type descriptor", idiom_rule: "AF-5 import-time devirtualization", evidence: ["crates/nuxie-runtime/src/state_machine.rs:1505-2024"]}
idiom_rules_invoked: ["AF-5 import-time devirtualization", "AF-7 own-by-value"]
confidence: high
notes: "Rust consolidates the C++ object/virtual family into immutable enums, indices, and owned vectors. Those values are fixed during import/instance construction and do not pass the mutation-timing gate. Crate-wide family grep and the listed sibling sweep were clean for this row."
~~~

## B6-0011

~~~yaml
row_id: B6-0011
cpp_files: ["src/animation/blend_state_1d_input.cpp"]
rust_module: "crates/nuxie-runtime/src/animation.rs"
subsystem_cluster: animation
sibling_files_swept: ["crates/nuxie-runtime/src/animation.rs", "crates/nuxie-runtime/src/artboard.rs", "crates/nuxie-runtime/src/artboard_data_bind.rs", "crates/nuxie-runtime/src/data_bind_graph.rs", "crates/nuxie-runtime/src/focus.rs", "crates/nuxie-runtime/src/scripting.rs", "crates/nuxie-runtime/src/state_machine.rs", "crates/nuxie-runtime/src/state_machine/instance.rs", "crates/nuxie-runtime/src/state_machine/transition_conditions.rs"]
verdict: ADAPTED
axes:
  retained_identity: {status: adapted, idiom_rule: "AF-7 own-by-value", evidence: ["src/animation/blend_state_1d_input.cpp:1-0", "crates/nuxie-runtime/src/state_machine.rs:1505-2024"]}
  push_vs_poll: {status: isomorphic, cpp_pushes: false, evidence: ["src/animation/blend_state_1d_input.cpp:1-0", "crates/nuxie-runtime/src/state_machine.rs:1505-2024"], note: "no compensating cycle-time observer/poll introduced for this row"}
  update_ordering: {status: isomorphic, phases_cpp: ["import/initialize", "advance or evaluate", "apply"], phases_rust: ["import immutable descriptor", "advance or evaluate", "apply"]}
  ownership: {status: adapted, idiom_rule: "AF-7 own-by-value", evidence: ["src/animation/blend_state_1d_input.cpp:1-0", "crates/nuxie-runtime/src/state_machine.rs:1505-2024"]}
  compensation:
    status: adapted
    mechanisms: []
    import_time_constants:
      - {name: "runtime enum/index/type descriptor", idiom_rule: "AF-5 import-time devirtualization", evidence: ["crates/nuxie-runtime/src/state_machine.rs:1505-2024"]}
idiom_rules_invoked: ["AF-5 import-time devirtualization", "AF-7 own-by-value"]
confidence: high
notes: "Rust consolidates the C++ object/virtual family into immutable enums, indices, and owned vectors. Those values are fixed during import/instance construction and do not pass the mutation-timing gate. Crate-wide family grep and the listed sibling sweep were clean for this row."
~~~

## B6-0012

~~~yaml
row_id: B6-0012
cpp_files: ["src/animation/blend_state_1d_instance.cpp"]
rust_module: "crates/nuxie-runtime/src/animation.rs"
subsystem_cluster: animation
sibling_files_swept: ["crates/nuxie-runtime/src/animation.rs", "crates/nuxie-runtime/src/artboard.rs", "crates/nuxie-runtime/src/artboard_data_bind.rs", "crates/nuxie-runtime/src/data_bind_graph.rs", "crates/nuxie-runtime/src/focus.rs", "crates/nuxie-runtime/src/scripting.rs", "crates/nuxie-runtime/src/state_machine.rs", "crates/nuxie-runtime/src/state_machine/instance.rs", "crates/nuxie-runtime/src/state_machine/transition_conditions.rs"]
verdict: ADAPTED
axes:
  retained_identity: {status: adapted, idiom_rule: "AF-7 own-by-value", evidence: ["src/animation/blend_state_1d_instance.cpp:1-0", "crates/nuxie-runtime/src/state_machine.rs:1505-2024"]}
  push_vs_poll: {status: isomorphic, cpp_pushes: false, evidence: ["src/animation/blend_state_1d_instance.cpp:1-0", "crates/nuxie-runtime/src/state_machine.rs:1505-2024"], note: "no compensating cycle-time observer/poll introduced for this row"}
  update_ordering: {status: isomorphic, phases_cpp: ["import/initialize", "advance or evaluate", "apply"], phases_rust: ["import immutable descriptor", "advance or evaluate", "apply"]}
  ownership: {status: adapted, idiom_rule: "AF-7 own-by-value", evidence: ["src/animation/blend_state_1d_instance.cpp:1-0", "crates/nuxie-runtime/src/state_machine.rs:1505-2024"]}
  compensation:
    status: adapted
    mechanisms: []
    import_time_constants:
      - {name: "runtime enum/index/type descriptor", idiom_rule: "AF-5 import-time devirtualization", evidence: ["crates/nuxie-runtime/src/state_machine.rs:1505-2024"]}
idiom_rules_invoked: ["AF-5 import-time devirtualization", "AF-7 own-by-value"]
confidence: high
notes: "Rust consolidates the C++ object/virtual family into immutable enums, indices, and owned vectors. Those values are fixed during import/instance construction and do not pass the mutation-timing gate. Crate-wide family grep and the listed sibling sweep were clean for this row."
~~~

## B6-0013

~~~yaml
row_id: B6-0013
cpp_files: ["src/animation/blend_state_1d_viewmodel.cpp"]
rust_module: "crates/nuxie-runtime/src/animation.rs"
subsystem_cluster: animation
sibling_files_swept: ["crates/nuxie-runtime/src/animation.rs", "crates/nuxie-runtime/src/artboard.rs", "crates/nuxie-runtime/src/artboard_data_bind.rs", "crates/nuxie-runtime/src/data_bind_graph.rs", "crates/nuxie-runtime/src/focus.rs", "crates/nuxie-runtime/src/scripting.rs", "crates/nuxie-runtime/src/state_machine.rs", "crates/nuxie-runtime/src/state_machine/instance.rs", "crates/nuxie-runtime/src/state_machine/transition_conditions.rs"]
verdict: DIVERGENT
axes:
  retained_identity: {status: divergent, evidence: ["src/animation/blend_state_1d_viewmodel.cpp:1-0", "crates/nuxie-runtime/src/state_machine.rs:1505-2024", "crates/nuxie-runtime/src/state_machine/instance.rs:79-80"]}
  push_vs_poll: {status: divergent, cpp_pushes: true, evidence: ["src/animation/blend_state_1d_viewmodel.cpp:1-0", "crates/nuxie-runtime/src/state_machine/instance.rs:5174-5220", "crates/nuxie-runtime/src/state_machine/instance.rs:5390-5403"]}
  update_ordering: {status: divergent, phases_cpp: ["retain live view-model/bindable relationship", "source mutation pushes dirt", "evaluate/action"], phases_rust: ["read candidate generations", "rebind copied graph state", "evaluate/action"]}
  ownership: {status: divergent, evidence: ["src/animation/blend_state_1d_viewmodel.cpp:1-0", "crates/nuxie-runtime/src/state_machine/instance.rs:79-80", "crates/nuxie-runtime/src/state_machine/instance.rs:5174-5220"]}
  compensation:
    status: divergent
    mechanisms:
      - {name: "owned_view_model_candidate_generation_rebind", kind: "AF-2 copied-state refresh / AF-4 push-to-poll regression", mutation_gated: true, cpp_counterpart: none, evidence: ["crates/nuxie-runtime/src/state_machine/instance.rs:5174-5220", "crates/nuxie-runtime/src/state_machine/instance.rs:5390-5403"]}
    import_time_constants:
      - {name: "view-model path/type descriptor", idiom_rule: "AF-5 import-time devirtualization", evidence: ["crates/nuxie-runtime/src/state_machine.rs:1505-2024"]}
idiom_rules_invoked: ["AF-5 import-time devirtualization"]
confidence: high
notes: "Current mid-RB-1 state recorded mechanically; candidate/generation work is scheduled by the #RB-1 mini-queue at docs/parity-closeout-status.md:210-225. No remediation judgment is made."
~~~

## B6-0014

~~~yaml
row_id: B6-0014
cpp_files: ["src/animation/blend_state_direct.cpp"]
rust_module: "crates/nuxie-runtime/src/animation.rs"
subsystem_cluster: animation
sibling_files_swept: ["crates/nuxie-runtime/src/animation.rs", "crates/nuxie-runtime/src/artboard.rs", "crates/nuxie-runtime/src/artboard_data_bind.rs", "crates/nuxie-runtime/src/data_bind_graph.rs", "crates/nuxie-runtime/src/focus.rs", "crates/nuxie-runtime/src/scripting.rs", "crates/nuxie-runtime/src/state_machine.rs", "crates/nuxie-runtime/src/state_machine/instance.rs", "crates/nuxie-runtime/src/state_machine/transition_conditions.rs"]
verdict: ADAPTED
axes:
  retained_identity: {status: adapted, idiom_rule: "AF-7 own-by-value", evidence: ["src/animation/blend_state_direct.cpp:1-0", "crates/nuxie-runtime/src/state_machine.rs:1505-2024"]}
  push_vs_poll: {status: isomorphic, cpp_pushes: false, evidence: ["src/animation/blend_state_direct.cpp:1-0", "crates/nuxie-runtime/src/state_machine.rs:1505-2024"], note: "no compensating cycle-time observer/poll introduced for this row"}
  update_ordering: {status: isomorphic, phases_cpp: ["import/initialize", "advance or evaluate", "apply"], phases_rust: ["import immutable descriptor", "advance or evaluate", "apply"]}
  ownership: {status: adapted, idiom_rule: "AF-7 own-by-value", evidence: ["src/animation/blend_state_direct.cpp:1-0", "crates/nuxie-runtime/src/state_machine.rs:1505-2024"]}
  compensation:
    status: adapted
    mechanisms: []
    import_time_constants:
      - {name: "runtime enum/index/type descriptor", idiom_rule: "AF-5 import-time devirtualization", evidence: ["crates/nuxie-runtime/src/state_machine.rs:1505-2024"]}
idiom_rules_invoked: ["AF-5 import-time devirtualization", "AF-7 own-by-value"]
confidence: high
notes: "Rust consolidates the C++ object/virtual family into immutable enums, indices, and owned vectors. Those values are fixed during import/instance construction and do not pass the mutation-timing gate. Crate-wide family grep and the listed sibling sweep were clean for this row."
~~~

## B6-0015

~~~yaml
row_id: B6-0015
cpp_files: ["src/animation/blend_state_direct_instance.cpp"]
rust_module: "crates/nuxie-runtime/src/animation.rs"
subsystem_cluster: animation
sibling_files_swept: ["crates/nuxie-runtime/src/animation.rs", "crates/nuxie-runtime/src/artboard.rs", "crates/nuxie-runtime/src/artboard_data_bind.rs", "crates/nuxie-runtime/src/data_bind_graph.rs", "crates/nuxie-runtime/src/focus.rs", "crates/nuxie-runtime/src/scripting.rs", "crates/nuxie-runtime/src/state_machine.rs", "crates/nuxie-runtime/src/state_machine/instance.rs", "crates/nuxie-runtime/src/state_machine/transition_conditions.rs"]
verdict: ADAPTED
axes:
  retained_identity: {status: adapted, idiom_rule: "AF-7 own-by-value", evidence: ["src/animation/blend_state_direct_instance.cpp:1-0", "crates/nuxie-runtime/src/state_machine.rs:1505-2024"]}
  push_vs_poll: {status: isomorphic, cpp_pushes: false, evidence: ["src/animation/blend_state_direct_instance.cpp:1-0", "crates/nuxie-runtime/src/state_machine.rs:1505-2024"], note: "no compensating cycle-time observer/poll introduced for this row"}
  update_ordering: {status: isomorphic, phases_cpp: ["import/initialize", "advance or evaluate", "apply"], phases_rust: ["import immutable descriptor", "advance or evaluate", "apply"]}
  ownership: {status: adapted, idiom_rule: "AF-7 own-by-value", evidence: ["src/animation/blend_state_direct_instance.cpp:1-0", "crates/nuxie-runtime/src/state_machine.rs:1505-2024"]}
  compensation:
    status: adapted
    mechanisms: []
    import_time_constants:
      - {name: "runtime enum/index/type descriptor", idiom_rule: "AF-5 import-time devirtualization", evidence: ["crates/nuxie-runtime/src/state_machine.rs:1505-2024"]}
idiom_rules_invoked: ["AF-5 import-time devirtualization", "AF-7 own-by-value"]
confidence: high
notes: "Rust consolidates the C++ object/virtual family into immutable enums, indices, and owned vectors. Those values are fixed during import/instance construction and do not pass the mutation-timing gate. Crate-wide family grep and the listed sibling sweep were clean for this row."
~~~

## B6-0016

~~~yaml
row_id: B6-0016
cpp_files: ["src/animation/blend_state_transition.cpp"]
rust_module: "crates/nuxie-runtime/src/animation.rs"
subsystem_cluster: animation
sibling_files_swept: ["crates/nuxie-runtime/src/animation.rs", "crates/nuxie-runtime/src/artboard.rs", "crates/nuxie-runtime/src/artboard_data_bind.rs", "crates/nuxie-runtime/src/data_bind_graph.rs", "crates/nuxie-runtime/src/focus.rs", "crates/nuxie-runtime/src/scripting.rs", "crates/nuxie-runtime/src/state_machine.rs", "crates/nuxie-runtime/src/state_machine/instance.rs", "crates/nuxie-runtime/src/state_machine/transition_conditions.rs"]
verdict: ADAPTED
axes:
  retained_identity: {status: adapted, idiom_rule: "AF-7 own-by-value", evidence: ["src/animation/blend_state_transition.cpp:1-0", "crates/nuxie-runtime/src/state_machine.rs:1505-2024"]}
  push_vs_poll: {status: isomorphic, cpp_pushes: false, evidence: ["src/animation/blend_state_transition.cpp:1-0", "crates/nuxie-runtime/src/state_machine.rs:1505-2024"], note: "no compensating cycle-time observer/poll introduced for this row"}
  update_ordering: {status: isomorphic, phases_cpp: ["import/initialize", "advance or evaluate", "apply"], phases_rust: ["import immutable descriptor", "advance or evaluate", "apply"]}
  ownership: {status: adapted, idiom_rule: "AF-7 own-by-value", evidence: ["src/animation/blend_state_transition.cpp:1-0", "crates/nuxie-runtime/src/state_machine.rs:1505-2024"]}
  compensation:
    status: adapted
    mechanisms: []
    import_time_constants:
      - {name: "runtime enum/index/type descriptor", idiom_rule: "AF-5 import-time devirtualization", evidence: ["crates/nuxie-runtime/src/state_machine.rs:1505-2024"]}
idiom_rules_invoked: ["AF-5 import-time devirtualization", "AF-7 own-by-value"]
confidence: high
notes: "Rust consolidates the C++ object/virtual family into immutable enums, indices, and owned vectors. Those values are fixed during import/instance construction and do not pass the mutation-timing gate. Crate-wide family grep and the listed sibling sweep were clean for this row."
~~~

## B6-0017

~~~yaml
row_id: B6-0017
cpp_files: ["src/animation/cubic_ease_interpolator.cpp"]
rust_module: "crates/nuxie-runtime/src/animation.rs"
subsystem_cluster: animation
sibling_files_swept: ["crates/nuxie-runtime/src/animation.rs", "crates/nuxie-runtime/src/artboard.rs", "crates/nuxie-runtime/src/artboard_data_bind.rs", "crates/nuxie-runtime/src/data_bind_graph.rs", "crates/nuxie-runtime/src/focus.rs", "crates/nuxie-runtime/src/scripting.rs", "crates/nuxie-runtime/src/state_machine.rs", "crates/nuxie-runtime/src/state_machine/instance.rs", "crates/nuxie-runtime/src/state_machine/transition_conditions.rs"]
verdict: ISOMORPHIC
axes:
  retained_identity: {status: isomorphic, evidence: ["src/animation/cubic_ease_interpolator.cpp:1-0", "crates/nuxie-runtime/src/animation.rs:22-277"], note: "pure value computation; no shared mutable identity"}
  push_vs_poll: {status: isomorphic, cpp_pushes: false, evidence: ["src/animation/cubic_ease_interpolator.cpp:1-0", "crates/nuxie-runtime/src/animation.rs:22-277"]}
  update_ordering: {status: isomorphic, phases_cpp: ["evaluate/interpolate", "return value"], phases_rust: ["evaluate/interpolate", "return value"]}
  ownership: {status: isomorphic, evidence: ["src/animation/cubic_ease_interpolator.cpp:1-0", "crates/nuxie-runtime/src/animation.rs:22-277"], note: "solver and interpolation values are owned by value"}
  compensation:
    {status: clear, mechanisms: [], import_time_constants: []}
idiom_rules_invoked: []
confidence: high
notes: "The mathematical hot path and call ordering match; crate-wide keyword and family greps found no state written during advance/update/bind to reconcile drift for this row."
~~~

## B6-0018

~~~yaml
row_id: B6-0018
cpp_files: ["src/animation/cubic_interpolator.cpp"]
rust_module: "crates/nuxie-runtime/src/animation.rs"
subsystem_cluster: animation
sibling_files_swept: ["crates/nuxie-runtime/src/animation.rs", "crates/nuxie-runtime/src/artboard.rs", "crates/nuxie-runtime/src/artboard_data_bind.rs", "crates/nuxie-runtime/src/data_bind_graph.rs", "crates/nuxie-runtime/src/focus.rs", "crates/nuxie-runtime/src/scripting.rs", "crates/nuxie-runtime/src/state_machine.rs", "crates/nuxie-runtime/src/state_machine/instance.rs", "crates/nuxie-runtime/src/state_machine/transition_conditions.rs"]
verdict: ISOMORPHIC
axes:
  retained_identity: {status: isomorphic, evidence: ["src/animation/cubic_interpolator.cpp:1-0", "crates/nuxie-runtime/src/animation.rs:22-277"], note: "pure value computation; no shared mutable identity"}
  push_vs_poll: {status: isomorphic, cpp_pushes: false, evidence: ["src/animation/cubic_interpolator.cpp:1-0", "crates/nuxie-runtime/src/animation.rs:22-277"]}
  update_ordering: {status: isomorphic, phases_cpp: ["evaluate/interpolate", "return value"], phases_rust: ["evaluate/interpolate", "return value"]}
  ownership: {status: isomorphic, evidence: ["src/animation/cubic_interpolator.cpp:1-0", "crates/nuxie-runtime/src/animation.rs:22-277"], note: "solver and interpolation values are owned by value"}
  compensation:
    {status: clear, mechanisms: [], import_time_constants: []}
idiom_rules_invoked: []
confidence: high
notes: "The mathematical hot path and call ordering match; crate-wide keyword and family greps found no state written during advance/update/bind to reconcile drift for this row."
~~~

## B6-0019

~~~yaml
row_id: B6-0019
cpp_files: ["src/animation/cubic_interpolator_component.cpp"]
rust_module: "crates/nuxie-runtime/src/animation.rs"
subsystem_cluster: animation
sibling_files_swept: ["crates/nuxie-runtime/src/animation.rs", "crates/nuxie-runtime/src/artboard.rs", "crates/nuxie-runtime/src/artboard_data_bind.rs", "crates/nuxie-runtime/src/data_bind_graph.rs", "crates/nuxie-runtime/src/focus.rs", "crates/nuxie-runtime/src/scripting.rs", "crates/nuxie-runtime/src/state_machine.rs", "crates/nuxie-runtime/src/state_machine/instance.rs", "crates/nuxie-runtime/src/state_machine/transition_conditions.rs"]
verdict: ISOMORPHIC
axes:
  retained_identity: {status: isomorphic, evidence: ["src/animation/cubic_interpolator_component.cpp:1-0", "crates/nuxie-runtime/src/animation.rs:22-277"], note: "pure value computation; no shared mutable identity"}
  push_vs_poll: {status: isomorphic, cpp_pushes: false, evidence: ["src/animation/cubic_interpolator_component.cpp:1-0", "crates/nuxie-runtime/src/animation.rs:22-277"]}
  update_ordering: {status: isomorphic, phases_cpp: ["evaluate/interpolate", "return value"], phases_rust: ["evaluate/interpolate", "return value"]}
  ownership: {status: isomorphic, evidence: ["src/animation/cubic_interpolator_component.cpp:1-0", "crates/nuxie-runtime/src/animation.rs:22-277"], note: "solver and interpolation values are owned by value"}
  compensation:
    {status: clear, mechanisms: [], import_time_constants: []}
idiom_rules_invoked: []
confidence: high
notes: "The mathematical hot path and call ordering match; crate-wide keyword and family greps found no state written during advance/update/bind to reconcile drift for this row."
~~~

## B6-0020

~~~yaml
row_id: B6-0020
cpp_files: ["src/animation/cubic_interpolator_solver.cpp"]
rust_module: "crates/nuxie-runtime/src/animation.rs"
subsystem_cluster: animation
sibling_files_swept: ["crates/nuxie-runtime/src/animation.rs", "crates/nuxie-runtime/src/artboard.rs", "crates/nuxie-runtime/src/artboard_data_bind.rs", "crates/nuxie-runtime/src/data_bind_graph.rs", "crates/nuxie-runtime/src/focus.rs", "crates/nuxie-runtime/src/scripting.rs", "crates/nuxie-runtime/src/state_machine.rs", "crates/nuxie-runtime/src/state_machine/instance.rs", "crates/nuxie-runtime/src/state_machine/transition_conditions.rs"]
verdict: ISOMORPHIC
axes:
  retained_identity: {status: isomorphic, evidence: ["src/animation/cubic_interpolator_solver.cpp:1-0", "crates/nuxie-runtime/src/animation.rs:22-277"], note: "pure value computation; no shared mutable identity"}
  push_vs_poll: {status: isomorphic, cpp_pushes: false, evidence: ["src/animation/cubic_interpolator_solver.cpp:1-0", "crates/nuxie-runtime/src/animation.rs:22-277"]}
  update_ordering: {status: isomorphic, phases_cpp: ["evaluate/interpolate", "return value"], phases_rust: ["evaluate/interpolate", "return value"]}
  ownership: {status: isomorphic, evidence: ["src/animation/cubic_interpolator_solver.cpp:1-0", "crates/nuxie-runtime/src/animation.rs:22-277"], note: "solver and interpolation values are owned by value"}
  compensation:
    {status: clear, mechanisms: [], import_time_constants: []}
idiom_rules_invoked: []
confidence: high
notes: "The mathematical hot path and call ordering match; crate-wide keyword and family greps found no state written during advance/update/bind to reconcile drift for this row."
~~~

## B6-0021

~~~yaml
row_id: B6-0021
cpp_files: ["src/animation/cubic_value_interpolator.cpp"]
rust_module: "crates/nuxie-runtime/src/animation.rs"
subsystem_cluster: animation
sibling_files_swept: ["crates/nuxie-runtime/src/animation.rs", "crates/nuxie-runtime/src/artboard.rs", "crates/nuxie-runtime/src/artboard_data_bind.rs", "crates/nuxie-runtime/src/data_bind_graph.rs", "crates/nuxie-runtime/src/focus.rs", "crates/nuxie-runtime/src/scripting.rs", "crates/nuxie-runtime/src/state_machine.rs", "crates/nuxie-runtime/src/state_machine/instance.rs", "crates/nuxie-runtime/src/state_machine/transition_conditions.rs"]
verdict: ISOMORPHIC
axes:
  retained_identity: {status: isomorphic, evidence: ["src/animation/cubic_value_interpolator.cpp:1-0", "crates/nuxie-runtime/src/animation.rs:22-277"], note: "pure value computation; no shared mutable identity"}
  push_vs_poll: {status: isomorphic, cpp_pushes: false, evidence: ["src/animation/cubic_value_interpolator.cpp:1-0", "crates/nuxie-runtime/src/animation.rs:22-277"]}
  update_ordering: {status: isomorphic, phases_cpp: ["evaluate/interpolate", "return value"], phases_rust: ["evaluate/interpolate", "return value"]}
  ownership: {status: isomorphic, evidence: ["src/animation/cubic_value_interpolator.cpp:1-0", "crates/nuxie-runtime/src/animation.rs:22-277"], note: "solver and interpolation values are owned by value"}
  compensation:
    {status: clear, mechanisms: [], import_time_constants: []}
idiom_rules_invoked: []
confidence: high
notes: "The mathematical hot path and call ordering match; crate-wide keyword and family greps found no state written during advance/update/bind to reconcile drift for this row."
~~~

## B6-0022

~~~yaml
row_id: B6-0022
cpp_files: ["src/animation/elastic_ease.cpp"]
rust_module: "crates/nuxie-runtime/src/animation.rs"
subsystem_cluster: animation
sibling_files_swept: ["crates/nuxie-runtime/src/animation.rs", "crates/nuxie-runtime/src/artboard.rs", "crates/nuxie-runtime/src/artboard_data_bind.rs", "crates/nuxie-runtime/src/data_bind_graph.rs", "crates/nuxie-runtime/src/focus.rs", "crates/nuxie-runtime/src/scripting.rs", "crates/nuxie-runtime/src/state_machine.rs", "crates/nuxie-runtime/src/state_machine/instance.rs", "crates/nuxie-runtime/src/state_machine/transition_conditions.rs"]
verdict: ISOMORPHIC
axes:
  retained_identity: {status: isomorphic, evidence: ["src/animation/elastic_ease.cpp:1-0", "crates/nuxie-runtime/src/animation.rs:22-277"], note: "pure value computation; no shared mutable identity"}
  push_vs_poll: {status: isomorphic, cpp_pushes: false, evidence: ["src/animation/elastic_ease.cpp:1-0", "crates/nuxie-runtime/src/animation.rs:22-277"]}
  update_ordering: {status: isomorphic, phases_cpp: ["evaluate/interpolate", "return value"], phases_rust: ["evaluate/interpolate", "return value"]}
  ownership: {status: isomorphic, evidence: ["src/animation/elastic_ease.cpp:1-0", "crates/nuxie-runtime/src/animation.rs:22-277"], note: "solver and interpolation values are owned by value"}
  compensation:
    {status: clear, mechanisms: [], import_time_constants: []}
idiom_rules_invoked: []
confidence: high
notes: "The mathematical hot path and call ordering match; crate-wide keyword and family greps found no state written during advance/update/bind to reconcile drift for this row."
~~~

## B6-0023

~~~yaml
row_id: B6-0023
cpp_files: ["src/animation/elastic_interpolator.cpp"]
rust_module: "crates/nuxie-runtime/src/animation.rs"
subsystem_cluster: animation
sibling_files_swept: ["crates/nuxie-runtime/src/animation.rs", "crates/nuxie-runtime/src/artboard.rs", "crates/nuxie-runtime/src/artboard_data_bind.rs", "crates/nuxie-runtime/src/data_bind_graph.rs", "crates/nuxie-runtime/src/focus.rs", "crates/nuxie-runtime/src/scripting.rs", "crates/nuxie-runtime/src/state_machine.rs", "crates/nuxie-runtime/src/state_machine/instance.rs", "crates/nuxie-runtime/src/state_machine/transition_conditions.rs"]
verdict: ISOMORPHIC
axes:
  retained_identity: {status: isomorphic, evidence: ["src/animation/elastic_interpolator.cpp:1-0", "crates/nuxie-runtime/src/animation.rs:22-277"], note: "pure value computation; no shared mutable identity"}
  push_vs_poll: {status: isomorphic, cpp_pushes: false, evidence: ["src/animation/elastic_interpolator.cpp:1-0", "crates/nuxie-runtime/src/animation.rs:22-277"]}
  update_ordering: {status: isomorphic, phases_cpp: ["evaluate/interpolate", "return value"], phases_rust: ["evaluate/interpolate", "return value"]}
  ownership: {status: isomorphic, evidence: ["src/animation/elastic_interpolator.cpp:1-0", "crates/nuxie-runtime/src/animation.rs:22-277"], note: "solver and interpolation values are owned by value"}
  compensation:
    {status: clear, mechanisms: [], import_time_constants: []}
idiom_rules_invoked: []
confidence: high
notes: "The mathematical hot path and call ordering match; crate-wide keyword and family greps found no state written during advance/update/bind to reconcile drift for this row."
~~~

## B6-0024

~~~yaml
row_id: B6-0024
cpp_files: ["src/animation/focus_action_clear.cpp"]
rust_module: "crates/nuxie-runtime/src/animation.rs"
subsystem_cluster: animation
sibling_files_swept: ["crates/nuxie-runtime/src/animation.rs", "crates/nuxie-runtime/src/artboard.rs", "crates/nuxie-runtime/src/artboard_data_bind.rs", "crates/nuxie-runtime/src/data_bind_graph.rs", "crates/nuxie-runtime/src/focus.rs", "crates/nuxie-runtime/src/scripting.rs", "crates/nuxie-runtime/src/state_machine.rs", "crates/nuxie-runtime/src/state_machine/instance.rs", "crates/nuxie-runtime/src/state_machine/transition_conditions.rs"]
verdict: ADAPTED
axes:
  retained_identity: {status: adapted, idiom_rule: "AF-7 own-by-value", evidence: ["src/animation/focus_action_clear.cpp:1-0", "crates/nuxie-runtime/src/state_machine.rs:2055-2207"]}
  push_vs_poll: {status: isomorphic, cpp_pushes: false, evidence: ["src/animation/focus_action_clear.cpp:1-0", "crates/nuxie-runtime/src/state_machine.rs:2055-2207"], note: "no compensating cycle-time observer/poll introduced for this row"}
  update_ordering: {status: isomorphic, phases_cpp: ["import/initialize", "advance or evaluate", "apply"], phases_rust: ["import immutable descriptor", "advance or evaluate", "apply"]}
  ownership: {status: adapted, idiom_rule: "AF-7 own-by-value", evidence: ["src/animation/focus_action_clear.cpp:1-0", "crates/nuxie-runtime/src/state_machine.rs:2055-2207"]}
  compensation:
    status: adapted
    mechanisms: []
    import_time_constants:
      - {name: "runtime enum/index/type descriptor", idiom_rule: "AF-5 import-time devirtualization", evidence: ["crates/nuxie-runtime/src/state_machine.rs:2055-2207"]}
idiom_rules_invoked: ["AF-5 import-time devirtualization", "AF-7 own-by-value"]
confidence: high
notes: "Rust consolidates the C++ object/virtual family into immutable enums, indices, and owned vectors. Those values are fixed during import/instance construction and do not pass the mutation-timing gate. Crate-wide family grep and the listed sibling sweep were clean for this row."
~~~

## B6-0025

~~~yaml
row_id: B6-0025
cpp_files: ["src/animation/focus_action_target.cpp"]
rust_module: "crates/nuxie-runtime/src/animation.rs"
subsystem_cluster: animation
sibling_files_swept: ["crates/nuxie-runtime/src/animation.rs", "crates/nuxie-runtime/src/artboard.rs", "crates/nuxie-runtime/src/artboard_data_bind.rs", "crates/nuxie-runtime/src/data_bind_graph.rs", "crates/nuxie-runtime/src/focus.rs", "crates/nuxie-runtime/src/scripting.rs", "crates/nuxie-runtime/src/state_machine.rs", "crates/nuxie-runtime/src/state_machine/instance.rs", "crates/nuxie-runtime/src/state_machine/transition_conditions.rs"]
verdict: ADAPTED
axes:
  retained_identity: {status: adapted, idiom_rule: "AF-7 own-by-value", evidence: ["src/animation/focus_action_target.cpp:1-0", "crates/nuxie-runtime/src/state_machine.rs:2055-2207"]}
  push_vs_poll: {status: isomorphic, cpp_pushes: false, evidence: ["src/animation/focus_action_target.cpp:1-0", "crates/nuxie-runtime/src/state_machine.rs:2055-2207"], note: "no compensating cycle-time observer/poll introduced for this row"}
  update_ordering: {status: isomorphic, phases_cpp: ["import/initialize", "advance or evaluate", "apply"], phases_rust: ["import immutable descriptor", "advance or evaluate", "apply"]}
  ownership: {status: adapted, idiom_rule: "AF-7 own-by-value", evidence: ["src/animation/focus_action_target.cpp:1-0", "crates/nuxie-runtime/src/state_machine.rs:2055-2207"]}
  compensation:
    status: adapted
    mechanisms: []
    import_time_constants:
      - {name: "runtime enum/index/type descriptor", idiom_rule: "AF-5 import-time devirtualization", evidence: ["crates/nuxie-runtime/src/state_machine.rs:2055-2207"]}
idiom_rules_invoked: ["AF-5 import-time devirtualization", "AF-7 own-by-value"]
confidence: high
notes: "Rust consolidates the C++ object/virtual family into immutable enums, indices, and owned vectors. Those values are fixed during import/instance construction and do not pass the mutation-timing gate. Crate-wide family grep and the listed sibling sweep were clean for this row."
~~~

## B6-0026

~~~yaml
row_id: B6-0026
cpp_files: ["src/animation/focus_action_traversal.cpp"]
rust_module: "crates/nuxie-runtime/src/animation.rs"
subsystem_cluster: animation
sibling_files_swept: ["crates/nuxie-runtime/src/animation.rs", "crates/nuxie-runtime/src/artboard.rs", "crates/nuxie-runtime/src/artboard_data_bind.rs", "crates/nuxie-runtime/src/data_bind_graph.rs", "crates/nuxie-runtime/src/focus.rs", "crates/nuxie-runtime/src/scripting.rs", "crates/nuxie-runtime/src/state_machine.rs", "crates/nuxie-runtime/src/state_machine/instance.rs", "crates/nuxie-runtime/src/state_machine/transition_conditions.rs"]
verdict: ADAPTED
axes:
  retained_identity: {status: adapted, idiom_rule: "AF-7 own-by-value", evidence: ["src/animation/focus_action_traversal.cpp:1-0", "crates/nuxie-runtime/src/state_machine.rs:2055-2207"]}
  push_vs_poll: {status: isomorphic, cpp_pushes: false, evidence: ["src/animation/focus_action_traversal.cpp:1-0", "crates/nuxie-runtime/src/state_machine.rs:2055-2207"], note: "no compensating cycle-time observer/poll introduced for this row"}
  update_ordering: {status: isomorphic, phases_cpp: ["import/initialize", "advance or evaluate", "apply"], phases_rust: ["import immutable descriptor", "advance or evaluate", "apply"]}
  ownership: {status: adapted, idiom_rule: "AF-7 own-by-value", evidence: ["src/animation/focus_action_traversal.cpp:1-0", "crates/nuxie-runtime/src/state_machine.rs:2055-2207"]}
  compensation:
    status: adapted
    mechanisms: []
    import_time_constants:
      - {name: "runtime enum/index/type descriptor", idiom_rule: "AF-5 import-time devirtualization", evidence: ["crates/nuxie-runtime/src/state_machine.rs:2055-2207"]}
idiom_rules_invoked: ["AF-5 import-time devirtualization", "AF-7 own-by-value"]
confidence: high
notes: "Rust consolidates the C++ object/virtual family into immutable enums, indices, and owned vectors. Those values are fixed during import/instance construction and do not pass the mutation-timing gate. Crate-wide family grep and the listed sibling sweep were clean for this row."
~~~

## B6-0027

~~~yaml
row_id: B6-0027
cpp_files: ["src/animation/focus_listener_group.cpp"]
rust_module: "crates/nuxie-runtime/src/animation.rs"
subsystem_cluster: animation
sibling_files_swept: ["crates/nuxie-runtime/src/animation.rs", "crates/nuxie-runtime/src/artboard.rs", "crates/nuxie-runtime/src/artboard_data_bind.rs", "crates/nuxie-runtime/src/data_bind_graph.rs", "crates/nuxie-runtime/src/focus.rs", "crates/nuxie-runtime/src/scripting.rs", "crates/nuxie-runtime/src/state_machine.rs", "crates/nuxie-runtime/src/state_machine/instance.rs", "crates/nuxie-runtime/src/state_machine/transition_conditions.rs"]
verdict: UNKNOWN
axes:
  retained_identity: {status: unknown, evidence: ["src/animation/focus_listener_group.cpp:1-0", "crates/nuxie-runtime/src/state_machine.rs:2055-2207"]}
  push_vs_poll: {status: unknown, cpp_pushes: true, evidence: ["src/animation/focus_listener_group.cpp:1-0", "crates/nuxie-runtime/src/state_machine.rs:2055-2207"]}
  update_ordering: {status: unknown, phases_cpp: ["read complete C++ row"], phases_rust: ["mapped/current subset only"]}
  ownership: {status: unknown, evidence: ["src/animation/focus_listener_group.cpp:1-0", "crates/nuxie-runtime/src/state_machine.rs:2055-2207"]}
  compensation:
    {status: clear, mechanisms: [], import_time_constants: []}
idiom_rules_invoked: []
confidence: high
notes: "UNKNOWN blocker: Rust listener import filters Focus listener types out at state_machine.rs:729-735; no current counterpart to C++ FocusListenerGroup add/remove-dependent lifecycle was found. Family grep and sibling sweep found no mutation-gated mechanism that could justify DIVERGENT; no inference was made."
~~~

## B6-0029

~~~yaml
row_id: B6-0029
cpp_files: ["src/animation/interpolating_keyframe.cpp"]
rust_module: "crates/nuxie-runtime/src/animation.rs"
subsystem_cluster: animation
sibling_files_swept: ["crates/nuxie-runtime/src/animation.rs", "crates/nuxie-runtime/src/artboard.rs", "crates/nuxie-runtime/src/artboard_data_bind.rs", "crates/nuxie-runtime/src/data_bind_graph.rs", "crates/nuxie-runtime/src/focus.rs", "crates/nuxie-runtime/src/scripting.rs", "crates/nuxie-runtime/src/state_machine.rs", "crates/nuxie-runtime/src/state_machine/instance.rs", "crates/nuxie-runtime/src/state_machine/transition_conditions.rs"]
verdict: ADAPTED
axes:
  retained_identity: {status: adapted, idiom_rule: "AF-7 own-by-value", evidence: ["src/animation/interpolating_keyframe.cpp:1-0", "crates/nuxie-runtime/src/animation.rs:302-2016"]}
  push_vs_poll: {status: isomorphic, cpp_pushes: false, evidence: ["src/animation/interpolating_keyframe.cpp:1-0", "crates/nuxie-runtime/src/animation.rs:302-2016"], note: "no compensating cycle-time observer/poll introduced for this row"}
  update_ordering: {status: isomorphic, phases_cpp: ["import/initialize", "advance or evaluate", "apply"], phases_rust: ["import immutable descriptor", "advance or evaluate", "apply"]}
  ownership: {status: adapted, idiom_rule: "AF-7 own-by-value", evidence: ["src/animation/interpolating_keyframe.cpp:1-0", "crates/nuxie-runtime/src/animation.rs:302-2016"]}
  compensation:
    status: adapted
    mechanisms: []
    import_time_constants:
      - {name: "runtime enum/index/type descriptor", idiom_rule: "AF-5 import-time devirtualization", evidence: ["crates/nuxie-runtime/src/animation.rs:302-2016"]}
idiom_rules_invoked: ["AF-5 import-time devirtualization", "AF-7 own-by-value"]
confidence: high
notes: "Rust consolidates the C++ object/virtual family into immutable enums, indices, and owned vectors. Those values are fixed during import/instance construction and do not pass the mutation-timing gate. Crate-wide family grep and the listed sibling sweep were clean for this row."
~~~

## B6-0031

~~~yaml
row_id: B6-0031
cpp_files: ["src/animation/keyed_object.cpp"]
rust_module: "crates/nuxie-runtime/src/animation.rs"
subsystem_cluster: animation
sibling_files_swept: ["crates/nuxie-runtime/src/animation.rs", "crates/nuxie-runtime/src/artboard.rs", "crates/nuxie-runtime/src/artboard_data_bind.rs", "crates/nuxie-runtime/src/data_bind_graph.rs", "crates/nuxie-runtime/src/focus.rs", "crates/nuxie-runtime/src/scripting.rs", "crates/nuxie-runtime/src/state_machine.rs", "crates/nuxie-runtime/src/state_machine/instance.rs", "crates/nuxie-runtime/src/state_machine/transition_conditions.rs"]
verdict: ADAPTED
axes:
  retained_identity: {status: adapted, idiom_rule: "AF-7 own-by-value", evidence: ["src/animation/keyed_object.cpp:1-0", "crates/nuxie-runtime/src/animation.rs:302-2016"]}
  push_vs_poll: {status: isomorphic, cpp_pushes: false, evidence: ["src/animation/keyed_object.cpp:1-0", "crates/nuxie-runtime/src/animation.rs:302-2016"], note: "no compensating cycle-time observer/poll introduced for this row"}
  update_ordering: {status: isomorphic, phases_cpp: ["import/initialize", "advance or evaluate", "apply"], phases_rust: ["import immutable descriptor", "advance or evaluate", "apply"]}
  ownership: {status: adapted, idiom_rule: "AF-7 own-by-value", evidence: ["src/animation/keyed_object.cpp:1-0", "crates/nuxie-runtime/src/animation.rs:302-2016"]}
  compensation:
    status: adapted
    mechanisms: []
    import_time_constants:
      - {name: "runtime enum/index/type descriptor", idiom_rule: "AF-5 import-time devirtualization", evidence: ["crates/nuxie-runtime/src/animation.rs:302-2016"]}
idiom_rules_invoked: ["AF-5 import-time devirtualization", "AF-7 own-by-value"]
confidence: high
notes: "Rust consolidates the C++ object/virtual family into immutable enums, indices, and owned vectors. Those values are fixed during import/instance construction and do not pass the mutation-timing gate. Crate-wide family grep and the listed sibling sweep were clean for this row."
~~~

## B6-0032

~~~yaml
row_id: B6-0032
cpp_files: ["src/animation/keyed_property.cpp"]
rust_module: "crates/nuxie-runtime/src/animation.rs"
subsystem_cluster: animation
sibling_files_swept: ["crates/nuxie-runtime/src/animation.rs", "crates/nuxie-runtime/src/artboard.rs", "crates/nuxie-runtime/src/artboard_data_bind.rs", "crates/nuxie-runtime/src/data_bind_graph.rs", "crates/nuxie-runtime/src/focus.rs", "crates/nuxie-runtime/src/scripting.rs", "crates/nuxie-runtime/src/state_machine.rs", "crates/nuxie-runtime/src/state_machine/instance.rs", "crates/nuxie-runtime/src/state_machine/transition_conditions.rs"]
verdict: ADAPTED
axes:
  retained_identity: {status: adapted, idiom_rule: "AF-7 own-by-value", evidence: ["src/animation/keyed_property.cpp:1-0", "crates/nuxie-runtime/src/animation.rs:302-2016"]}
  push_vs_poll: {status: isomorphic, cpp_pushes: false, evidence: ["src/animation/keyed_property.cpp:1-0", "crates/nuxie-runtime/src/animation.rs:302-2016"], note: "no compensating cycle-time observer/poll introduced for this row"}
  update_ordering: {status: isomorphic, phases_cpp: ["import/initialize", "advance or evaluate", "apply"], phases_rust: ["import immutable descriptor", "advance or evaluate", "apply"]}
  ownership: {status: adapted, idiom_rule: "AF-7 own-by-value", evidence: ["src/animation/keyed_property.cpp:1-0", "crates/nuxie-runtime/src/animation.rs:302-2016"]}
  compensation:
    status: adapted
    mechanisms: []
    import_time_constants:
      - {name: "runtime enum/index/type descriptor", idiom_rule: "AF-5 import-time devirtualization", evidence: ["crates/nuxie-runtime/src/animation.rs:302-2016"]}
idiom_rules_invoked: ["AF-5 import-time devirtualization", "AF-7 own-by-value"]
confidence: high
notes: "Rust consolidates the C++ object/virtual family into immutable enums, indices, and owned vectors. Those values are fixed during import/instance construction and do not pass the mutation-timing gate. Crate-wide family grep and the listed sibling sweep were clean for this row."
~~~

## B6-0033

~~~yaml
row_id: B6-0033
cpp_files: ["src/animation/keyframe.cpp"]
rust_module: "crates/nuxie-runtime/src/animation.rs"
subsystem_cluster: animation
sibling_files_swept: ["crates/nuxie-runtime/src/animation.rs", "crates/nuxie-runtime/src/artboard.rs", "crates/nuxie-runtime/src/artboard_data_bind.rs", "crates/nuxie-runtime/src/data_bind_graph.rs", "crates/nuxie-runtime/src/focus.rs", "crates/nuxie-runtime/src/scripting.rs", "crates/nuxie-runtime/src/state_machine.rs", "crates/nuxie-runtime/src/state_machine/instance.rs", "crates/nuxie-runtime/src/state_machine/transition_conditions.rs"]
verdict: ADAPTED
axes:
  retained_identity: {status: adapted, idiom_rule: "AF-7 own-by-value", evidence: ["src/animation/keyframe.cpp:1-0", "crates/nuxie-runtime/src/animation.rs:302-2016"]}
  push_vs_poll: {status: isomorphic, cpp_pushes: false, evidence: ["src/animation/keyframe.cpp:1-0", "crates/nuxie-runtime/src/animation.rs:302-2016"], note: "no compensating cycle-time observer/poll introduced for this row"}
  update_ordering: {status: isomorphic, phases_cpp: ["import/initialize", "advance or evaluate", "apply"], phases_rust: ["import immutable descriptor", "advance or evaluate", "apply"]}
  ownership: {status: adapted, idiom_rule: "AF-7 own-by-value", evidence: ["src/animation/keyframe.cpp:1-0", "crates/nuxie-runtime/src/animation.rs:302-2016"]}
  compensation:
    status: adapted
    mechanisms: []
    import_time_constants:
      - {name: "runtime enum/index/type descriptor", idiom_rule: "AF-5 import-time devirtualization", evidence: ["crates/nuxie-runtime/src/animation.rs:302-2016"]}
idiom_rules_invoked: ["AF-5 import-time devirtualization", "AF-7 own-by-value"]
confidence: high
notes: "Rust consolidates the C++ object/virtual family into immutable enums, indices, and owned vectors. Those values are fixed during import/instance construction and do not pass the mutation-timing gate. Crate-wide family grep and the listed sibling sweep were clean for this row."
~~~

## B6-0034

~~~yaml
row_id: B6-0034
cpp_files: ["src/animation/keyframe_bool.cpp"]
rust_module: "crates/nuxie-runtime/src/animation.rs"
subsystem_cluster: animation
sibling_files_swept: ["crates/nuxie-runtime/src/animation.rs", "crates/nuxie-runtime/src/artboard.rs", "crates/nuxie-runtime/src/artboard_data_bind.rs", "crates/nuxie-runtime/src/data_bind_graph.rs", "crates/nuxie-runtime/src/focus.rs", "crates/nuxie-runtime/src/scripting.rs", "crates/nuxie-runtime/src/state_machine.rs", "crates/nuxie-runtime/src/state_machine/instance.rs", "crates/nuxie-runtime/src/state_machine/transition_conditions.rs"]
verdict: DIVERGENT
axes:
  retained_identity: {status: divergent, evidence: ["src/animation/keyframe_bool.cpp:1-0", "src/animation/linear_animation_instance.cpp:81-100", "crates/nuxie-runtime/src/animation.rs:1540-1557", "crates/nuxie-runtime/src/animation.rs:1727-1758"]}
  push_vs_poll: {status: divergent, cpp_pushes: true, evidence: ["src/animation/state_machine_instance.cpp:3276-3374", "crates/nuxie-runtime/src/animation.rs:1632-1641", "crates/nuxie-runtime/src/animation.rs:1679-1724"]}
  update_ordering: {status: divergent, phases_cpp: ["construct retained BindableProperty targets on state entry", "DataBind writes holders directly", "keyframe apply reads same holders"], phases_rust: ["poll prototype revision", "copy graph source state", "drain updates into value map", "keyframe apply reads copied values"]}
  ownership: {status: divergent, evidence: ["src/animation/linear_animation_instance.cpp:81-100", "crates/nuxie-runtime/src/animation.rs:1555-1557", "crates/nuxie-runtime/src/animation.rs:1606-1629"]}
  compensation:
    status: divergent
    mechanisms:
      - {name: "key_frame_prototype_revision_poll", kind: "AF-2 copied-state refresh / AF-8 invented lifecycle", mutation_gated: true, cpp_counterpart: none, evidence: ["crates/nuxie-runtime/src/animation.rs:1557", "crates/nuxie-runtime/src/animation.rs:1632-1641", "crates/nuxie-runtime/src/data_bind_graph.rs:4400-4435", "crates/nuxie-runtime/src/data_bind_graph.rs:4505-4519"]}
      - {name: "copied_key_frame_value_holder_refresh", kind: "AF-1 retained-identity break / AF-2 copied-state refresh", mutation_gated: true, cpp_counterpart: "none", evidence: ["crates/nuxie-runtime/src/animation.rs:1555", "crates/nuxie-runtime/src/animation.rs:1643-1677", "crates/nuxie-runtime/src/animation.rs:1727-1758"]}
    import_time_constants:
      - {name: "keyed-property type/source snapshots", idiom_rule: "AF-5 import-time devirtualization", evidence: ["crates/nuxie-runtime/src/animation.rs:407-429"]}
idiom_rules_invoked: ["AF-5 import-time devirtualization"]
confidence: high
notes: "The keyword lookalikes at animation.rs:407-429 are import/build constants, not drift tracking. The two listed mechanisms are written during bind/advance and therefore pass the mutation-timing gate."
~~~

## B6-0035

~~~yaml
row_id: B6-0035
cpp_files: ["src/animation/keyframe_callback.cpp"]
rust_module: "crates/nuxie-runtime/src/animation.rs"
subsystem_cluster: animation
sibling_files_swept: ["crates/nuxie-runtime/src/animation.rs", "crates/nuxie-runtime/src/artboard.rs", "crates/nuxie-runtime/src/artboard_data_bind.rs", "crates/nuxie-runtime/src/data_bind_graph.rs", "crates/nuxie-runtime/src/focus.rs", "crates/nuxie-runtime/src/scripting.rs", "crates/nuxie-runtime/src/state_machine.rs", "crates/nuxie-runtime/src/state_machine/instance.rs", "crates/nuxie-runtime/src/state_machine/transition_conditions.rs"]
verdict: ADAPTED
axes:
  retained_identity: {status: adapted, idiom_rule: "AF-7 own-by-value", evidence: ["src/animation/keyframe_callback.cpp:1-0", "crates/nuxie-runtime/src/animation.rs:302-2016"]}
  push_vs_poll: {status: isomorphic, cpp_pushes: false, evidence: ["src/animation/keyframe_callback.cpp:1-0", "crates/nuxie-runtime/src/animation.rs:302-2016"], note: "no compensating cycle-time observer/poll introduced for this row"}
  update_ordering: {status: isomorphic, phases_cpp: ["import/initialize", "advance or evaluate", "apply"], phases_rust: ["import immutable descriptor", "advance or evaluate", "apply"]}
  ownership: {status: adapted, idiom_rule: "AF-7 own-by-value", evidence: ["src/animation/keyframe_callback.cpp:1-0", "crates/nuxie-runtime/src/animation.rs:302-2016"]}
  compensation:
    status: adapted
    mechanisms: []
    import_time_constants:
      - {name: "runtime enum/index/type descriptor", idiom_rule: "AF-5 import-time devirtualization", evidence: ["crates/nuxie-runtime/src/animation.rs:302-2016"]}
idiom_rules_invoked: ["AF-5 import-time devirtualization", "AF-7 own-by-value"]
confidence: high
notes: "Rust consolidates the C++ object/virtual family into immutable enums, indices, and owned vectors. Those values are fixed during import/instance construction and do not pass the mutation-timing gate. Crate-wide family grep and the listed sibling sweep were clean for this row."
~~~

## B6-0036

~~~yaml
row_id: B6-0036
cpp_files: ["src/animation/keyframe_color.cpp"]
rust_module: "crates/nuxie-runtime/src/animation.rs"
subsystem_cluster: animation
sibling_files_swept: ["crates/nuxie-runtime/src/animation.rs", "crates/nuxie-runtime/src/artboard.rs", "crates/nuxie-runtime/src/artboard_data_bind.rs", "crates/nuxie-runtime/src/data_bind_graph.rs", "crates/nuxie-runtime/src/focus.rs", "crates/nuxie-runtime/src/scripting.rs", "crates/nuxie-runtime/src/state_machine.rs", "crates/nuxie-runtime/src/state_machine/instance.rs", "crates/nuxie-runtime/src/state_machine/transition_conditions.rs"]
verdict: DIVERGENT
axes:
  retained_identity: {status: divergent, evidence: ["src/animation/keyframe_color.cpp:1-0", "src/animation/linear_animation_instance.cpp:81-100", "crates/nuxie-runtime/src/animation.rs:1540-1557", "crates/nuxie-runtime/src/animation.rs:1727-1758"]}
  push_vs_poll: {status: divergent, cpp_pushes: true, evidence: ["src/animation/state_machine_instance.cpp:3276-3374", "crates/nuxie-runtime/src/animation.rs:1632-1641", "crates/nuxie-runtime/src/animation.rs:1679-1724"]}
  update_ordering: {status: divergent, phases_cpp: ["construct retained BindableProperty targets on state entry", "DataBind writes holders directly", "keyframe apply reads same holders"], phases_rust: ["poll prototype revision", "copy graph source state", "drain updates into value map", "keyframe apply reads copied values"]}
  ownership: {status: divergent, evidence: ["src/animation/linear_animation_instance.cpp:81-100", "crates/nuxie-runtime/src/animation.rs:1555-1557", "crates/nuxie-runtime/src/animation.rs:1606-1629"]}
  compensation:
    status: divergent
    mechanisms:
      - {name: "key_frame_prototype_revision_poll", kind: "AF-2 copied-state refresh / AF-8 invented lifecycle", mutation_gated: true, cpp_counterpart: none, evidence: ["crates/nuxie-runtime/src/animation.rs:1557", "crates/nuxie-runtime/src/animation.rs:1632-1641", "crates/nuxie-runtime/src/data_bind_graph.rs:4400-4435", "crates/nuxie-runtime/src/data_bind_graph.rs:4505-4519"]}
      - {name: "copied_key_frame_value_holder_refresh", kind: "AF-1 retained-identity break / AF-2 copied-state refresh", mutation_gated: true, cpp_counterpart: "none", evidence: ["crates/nuxie-runtime/src/animation.rs:1555", "crates/nuxie-runtime/src/animation.rs:1643-1677", "crates/nuxie-runtime/src/animation.rs:1727-1758"]}
    import_time_constants:
      - {name: "keyed-property type/source snapshots", idiom_rule: "AF-5 import-time devirtualization", evidence: ["crates/nuxie-runtime/src/animation.rs:407-429"]}
idiom_rules_invoked: ["AF-5 import-time devirtualization"]
confidence: high
notes: "The keyword lookalikes at animation.rs:407-429 are import/build constants, not drift tracking. The two listed mechanisms are written during bind/advance and therefore pass the mutation-timing gate."
~~~

## B6-0037

~~~yaml
row_id: B6-0037
cpp_files: ["src/animation/keyframe_double.cpp"]
rust_module: "crates/nuxie-runtime/src/animation.rs"
subsystem_cluster: animation
sibling_files_swept: ["crates/nuxie-runtime/src/animation.rs", "crates/nuxie-runtime/src/artboard.rs", "crates/nuxie-runtime/src/artboard_data_bind.rs", "crates/nuxie-runtime/src/data_bind_graph.rs", "crates/nuxie-runtime/src/focus.rs", "crates/nuxie-runtime/src/scripting.rs", "crates/nuxie-runtime/src/state_machine.rs", "crates/nuxie-runtime/src/state_machine/instance.rs", "crates/nuxie-runtime/src/state_machine/transition_conditions.rs"]
verdict: DIVERGENT
axes:
  retained_identity: {status: divergent, evidence: ["src/animation/keyframe_double.cpp:1-0", "src/animation/linear_animation_instance.cpp:81-100", "crates/nuxie-runtime/src/animation.rs:1540-1557", "crates/nuxie-runtime/src/animation.rs:1727-1758"]}
  push_vs_poll: {status: divergent, cpp_pushes: true, evidence: ["src/animation/state_machine_instance.cpp:3276-3374", "crates/nuxie-runtime/src/animation.rs:1632-1641", "crates/nuxie-runtime/src/animation.rs:1679-1724"]}
  update_ordering: {status: divergent, phases_cpp: ["construct retained BindableProperty targets on state entry", "DataBind writes holders directly", "keyframe apply reads same holders"], phases_rust: ["poll prototype revision", "copy graph source state", "drain updates into value map", "keyframe apply reads copied values"]}
  ownership: {status: divergent, evidence: ["src/animation/linear_animation_instance.cpp:81-100", "crates/nuxie-runtime/src/animation.rs:1555-1557", "crates/nuxie-runtime/src/animation.rs:1606-1629"]}
  compensation:
    status: divergent
    mechanisms:
      - {name: "key_frame_prototype_revision_poll", kind: "AF-2 copied-state refresh / AF-8 invented lifecycle", mutation_gated: true, cpp_counterpart: none, evidence: ["crates/nuxie-runtime/src/animation.rs:1557", "crates/nuxie-runtime/src/animation.rs:1632-1641", "crates/nuxie-runtime/src/data_bind_graph.rs:4400-4435", "crates/nuxie-runtime/src/data_bind_graph.rs:4505-4519"]}
      - {name: "copied_key_frame_value_holder_refresh", kind: "AF-1 retained-identity break / AF-2 copied-state refresh", mutation_gated: true, cpp_counterpart: "none", evidence: ["crates/nuxie-runtime/src/animation.rs:1555", "crates/nuxie-runtime/src/animation.rs:1643-1677", "crates/nuxie-runtime/src/animation.rs:1727-1758"]}
    import_time_constants:
      - {name: "keyed-property type/source snapshots", idiom_rule: "AF-5 import-time devirtualization", evidence: ["crates/nuxie-runtime/src/animation.rs:407-429"]}
idiom_rules_invoked: ["AF-5 import-time devirtualization"]
confidence: high
notes: "The keyword lookalikes at animation.rs:407-429 are import/build constants, not drift tracking. The two listed mechanisms are written during bind/advance and therefore pass the mutation-timing gate."
~~~

## B6-0038

~~~yaml
row_id: B6-0038
cpp_files: ["src/animation/keyframe_id.cpp"]
rust_module: "crates/nuxie-runtime/src/animation.rs"
subsystem_cluster: animation
sibling_files_swept: ["crates/nuxie-runtime/src/animation.rs", "crates/nuxie-runtime/src/artboard.rs", "crates/nuxie-runtime/src/artboard_data_bind.rs", "crates/nuxie-runtime/src/data_bind_graph.rs", "crates/nuxie-runtime/src/focus.rs", "crates/nuxie-runtime/src/scripting.rs", "crates/nuxie-runtime/src/state_machine.rs", "crates/nuxie-runtime/src/state_machine/instance.rs", "crates/nuxie-runtime/src/state_machine/transition_conditions.rs"]
verdict: ADAPTED
axes:
  retained_identity: {status: adapted, idiom_rule: "AF-7 own-by-value", evidence: ["src/animation/keyframe_id.cpp:1-0", "crates/nuxie-runtime/src/animation.rs:302-2016"]}
  push_vs_poll: {status: isomorphic, cpp_pushes: false, evidence: ["src/animation/keyframe_id.cpp:1-0", "crates/nuxie-runtime/src/animation.rs:302-2016"], note: "no compensating cycle-time observer/poll introduced for this row"}
  update_ordering: {status: isomorphic, phases_cpp: ["import/initialize", "advance or evaluate", "apply"], phases_rust: ["import immutable descriptor", "advance or evaluate", "apply"]}
  ownership: {status: adapted, idiom_rule: "AF-7 own-by-value", evidence: ["src/animation/keyframe_id.cpp:1-0", "crates/nuxie-runtime/src/animation.rs:302-2016"]}
  compensation:
    status: adapted
    mechanisms: []
    import_time_constants:
      - {name: "runtime enum/index/type descriptor", idiom_rule: "AF-5 import-time devirtualization", evidence: ["crates/nuxie-runtime/src/animation.rs:302-2016"]}
idiom_rules_invoked: ["AF-5 import-time devirtualization", "AF-7 own-by-value"]
confidence: high
notes: "Rust consolidates the C++ object/virtual family into immutable enums, indices, and owned vectors. Those values are fixed during import/instance construction and do not pass the mutation-timing gate. Crate-wide family grep and the listed sibling sweep were clean for this row."
~~~

## B6-0039

~~~yaml
row_id: B6-0039
cpp_files: ["src/animation/keyframe_interpolator.cpp"]
rust_module: "crates/nuxie-runtime/src/animation.rs"
subsystem_cluster: animation
sibling_files_swept: ["crates/nuxie-runtime/src/animation.rs", "crates/nuxie-runtime/src/artboard.rs", "crates/nuxie-runtime/src/artboard_data_bind.rs", "crates/nuxie-runtime/src/data_bind_graph.rs", "crates/nuxie-runtime/src/focus.rs", "crates/nuxie-runtime/src/scripting.rs", "crates/nuxie-runtime/src/state_machine.rs", "crates/nuxie-runtime/src/state_machine/instance.rs", "crates/nuxie-runtime/src/state_machine/transition_conditions.rs"]
verdict: ADAPTED
axes:
  retained_identity: {status: adapted, idiom_rule: "AF-7 own-by-value", evidence: ["src/animation/keyframe_interpolator.cpp:1-0", "crates/nuxie-runtime/src/animation.rs:302-2016"]}
  push_vs_poll: {status: isomorphic, cpp_pushes: false, evidence: ["src/animation/keyframe_interpolator.cpp:1-0", "crates/nuxie-runtime/src/animation.rs:302-2016"], note: "no compensating cycle-time observer/poll introduced for this row"}
  update_ordering: {status: isomorphic, phases_cpp: ["import/initialize", "advance or evaluate", "apply"], phases_rust: ["import immutable descriptor", "advance or evaluate", "apply"]}
  ownership: {status: adapted, idiom_rule: "AF-7 own-by-value", evidence: ["src/animation/keyframe_interpolator.cpp:1-0", "crates/nuxie-runtime/src/animation.rs:302-2016"]}
  compensation:
    status: adapted
    mechanisms: []
    import_time_constants:
      - {name: "runtime enum/index/type descriptor", idiom_rule: "AF-5 import-time devirtualization", evidence: ["crates/nuxie-runtime/src/animation.rs:302-2016"]}
idiom_rules_invoked: ["AF-5 import-time devirtualization", "AF-7 own-by-value"]
confidence: high
notes: "Rust consolidates the C++ object/virtual family into immutable enums, indices, and owned vectors. Those values are fixed during import/instance construction and do not pass the mutation-timing gate. Crate-wide family grep and the listed sibling sweep were clean for this row."
~~~

## B6-0040

~~~yaml
row_id: B6-0040
cpp_files: ["src/animation/keyframe_string.cpp"]
rust_module: "crates/nuxie-runtime/src/animation.rs"
subsystem_cluster: animation
sibling_files_swept: ["crates/nuxie-runtime/src/animation.rs", "crates/nuxie-runtime/src/artboard.rs", "crates/nuxie-runtime/src/artboard_data_bind.rs", "crates/nuxie-runtime/src/data_bind_graph.rs", "crates/nuxie-runtime/src/focus.rs", "crates/nuxie-runtime/src/scripting.rs", "crates/nuxie-runtime/src/state_machine.rs", "crates/nuxie-runtime/src/state_machine/instance.rs", "crates/nuxie-runtime/src/state_machine/transition_conditions.rs"]
verdict: DIVERGENT
axes:
  retained_identity: {status: divergent, evidence: ["src/animation/keyframe_string.cpp:1-0", "src/animation/linear_animation_instance.cpp:81-100", "crates/nuxie-runtime/src/animation.rs:1540-1557", "crates/nuxie-runtime/src/animation.rs:1727-1758"]}
  push_vs_poll: {status: divergent, cpp_pushes: true, evidence: ["src/animation/state_machine_instance.cpp:3276-3374", "crates/nuxie-runtime/src/animation.rs:1632-1641", "crates/nuxie-runtime/src/animation.rs:1679-1724"]}
  update_ordering: {status: divergent, phases_cpp: ["construct retained BindableProperty targets on state entry", "DataBind writes holders directly", "keyframe apply reads same holders"], phases_rust: ["poll prototype revision", "copy graph source state", "drain updates into value map", "keyframe apply reads copied values"]}
  ownership: {status: divergent, evidence: ["src/animation/linear_animation_instance.cpp:81-100", "crates/nuxie-runtime/src/animation.rs:1555-1557", "crates/nuxie-runtime/src/animation.rs:1606-1629"]}
  compensation:
    status: divergent
    mechanisms:
      - {name: "key_frame_prototype_revision_poll", kind: "AF-2 copied-state refresh / AF-8 invented lifecycle", mutation_gated: true, cpp_counterpart: none, evidence: ["crates/nuxie-runtime/src/animation.rs:1557", "crates/nuxie-runtime/src/animation.rs:1632-1641", "crates/nuxie-runtime/src/data_bind_graph.rs:4400-4435", "crates/nuxie-runtime/src/data_bind_graph.rs:4505-4519"]}
      - {name: "copied_key_frame_value_holder_refresh", kind: "AF-1 retained-identity break / AF-2 copied-state refresh", mutation_gated: true, cpp_counterpart: "none", evidence: ["crates/nuxie-runtime/src/animation.rs:1555", "crates/nuxie-runtime/src/animation.rs:1643-1677", "crates/nuxie-runtime/src/animation.rs:1727-1758"]}
    import_time_constants:
      - {name: "keyed-property type/source snapshots", idiom_rule: "AF-5 import-time devirtualization", evidence: ["crates/nuxie-runtime/src/animation.rs:407-429"]}
idiom_rules_invoked: ["AF-5 import-time devirtualization"]
confidence: high
notes: "The keyword lookalikes at animation.rs:407-429 are import/build constants, not drift tracking. The two listed mechanisms are written during bind/advance and therefore pass the mutation-timing gate."
~~~

## B6-0041

~~~yaml
row_id: B6-0041
cpp_files: ["src/animation/keyframe_uint.cpp"]
rust_module: "crates/nuxie-runtime/src/animation.rs"
subsystem_cluster: animation
sibling_files_swept: ["crates/nuxie-runtime/src/animation.rs", "crates/nuxie-runtime/src/artboard.rs", "crates/nuxie-runtime/src/artboard_data_bind.rs", "crates/nuxie-runtime/src/data_bind_graph.rs", "crates/nuxie-runtime/src/focus.rs", "crates/nuxie-runtime/src/scripting.rs", "crates/nuxie-runtime/src/state_machine.rs", "crates/nuxie-runtime/src/state_machine/instance.rs", "crates/nuxie-runtime/src/state_machine/transition_conditions.rs"]
verdict: ADAPTED
axes:
  retained_identity: {status: adapted, idiom_rule: "AF-7 own-by-value", evidence: ["src/animation/keyframe_uint.cpp:1-0", "crates/nuxie-runtime/src/animation.rs:302-2016"]}
  push_vs_poll: {status: isomorphic, cpp_pushes: false, evidence: ["src/animation/keyframe_uint.cpp:1-0", "crates/nuxie-runtime/src/animation.rs:302-2016"], note: "no compensating cycle-time observer/poll introduced for this row"}
  update_ordering: {status: isomorphic, phases_cpp: ["import/initialize", "advance or evaluate", "apply"], phases_rust: ["import immutable descriptor", "advance or evaluate", "apply"]}
  ownership: {status: adapted, idiom_rule: "AF-7 own-by-value", evidence: ["src/animation/keyframe_uint.cpp:1-0", "crates/nuxie-runtime/src/animation.rs:302-2016"]}
  compensation:
    status: adapted
    mechanisms: []
    import_time_constants:
      - {name: "runtime enum/index/type descriptor", idiom_rule: "AF-5 import-time devirtualization", evidence: ["crates/nuxie-runtime/src/animation.rs:302-2016"]}
idiom_rules_invoked: ["AF-5 import-time devirtualization", "AF-7 own-by-value"]
confidence: high
notes: "Rust consolidates the C++ object/virtual family into immutable enums, indices, and owned vectors. Those values are fixed during import/instance construction and do not pass the mutation-timing gate. Crate-wide family grep and the listed sibling sweep were clean for this row."
~~~

## B6-0042

~~~yaml
row_id: B6-0042
cpp_files: ["src/animation/layer_state.cpp"]
rust_module: "crates/nuxie-runtime/src/animation.rs"
subsystem_cluster: animation
sibling_files_swept: ["crates/nuxie-runtime/src/animation.rs", "crates/nuxie-runtime/src/artboard.rs", "crates/nuxie-runtime/src/artboard_data_bind.rs", "crates/nuxie-runtime/src/data_bind_graph.rs", "crates/nuxie-runtime/src/focus.rs", "crates/nuxie-runtime/src/scripting.rs", "crates/nuxie-runtime/src/state_machine.rs", "crates/nuxie-runtime/src/state_machine/instance.rs", "crates/nuxie-runtime/src/state_machine/transition_conditions.rs"]
verdict: ADAPTED
axes:
  retained_identity: {status: adapted, idiom_rule: "AF-7 own-by-value", evidence: ["src/animation/layer_state.cpp:1-0", "crates/nuxie-runtime/src/animation.rs:302-2016"]}
  push_vs_poll: {status: isomorphic, cpp_pushes: false, evidence: ["src/animation/layer_state.cpp:1-0", "crates/nuxie-runtime/src/animation.rs:302-2016"], note: "no compensating cycle-time observer/poll introduced for this row"}
  update_ordering: {status: isomorphic, phases_cpp: ["import/initialize", "advance or evaluate", "apply"], phases_rust: ["import immutable descriptor", "advance or evaluate", "apply"]}
  ownership: {status: adapted, idiom_rule: "AF-7 own-by-value", evidence: ["src/animation/layer_state.cpp:1-0", "crates/nuxie-runtime/src/animation.rs:302-2016"]}
  compensation:
    status: adapted
    mechanisms: []
    import_time_constants:
      - {name: "runtime enum/index/type descriptor", idiom_rule: "AF-5 import-time devirtualization", evidence: ["crates/nuxie-runtime/src/animation.rs:302-2016"]}
idiom_rules_invoked: ["AF-5 import-time devirtualization", "AF-7 own-by-value"]
confidence: high
notes: "Rust consolidates the C++ object/virtual family into immutable enums, indices, and owned vectors. Those values are fixed during import/instance construction and do not pass the mutation-timing gate. Crate-wide family grep and the listed sibling sweep were clean for this row."
~~~

## B6-0043

~~~yaml
row_id: B6-0043
cpp_files: ["src/animation/linear_animation.cpp"]
rust_module: "crates/nuxie-runtime/src/animation.rs"
subsystem_cluster: animation
sibling_files_swept: ["crates/nuxie-runtime/src/animation.rs", "crates/nuxie-runtime/src/artboard.rs", "crates/nuxie-runtime/src/artboard_data_bind.rs", "crates/nuxie-runtime/src/data_bind_graph.rs", "crates/nuxie-runtime/src/focus.rs", "crates/nuxie-runtime/src/scripting.rs", "crates/nuxie-runtime/src/state_machine.rs", "crates/nuxie-runtime/src/state_machine/instance.rs", "crates/nuxie-runtime/src/state_machine/transition_conditions.rs"]
verdict: ADAPTED
axes:
  retained_identity: {status: adapted, idiom_rule: "AF-7 own-by-value", evidence: ["src/animation/linear_animation.cpp:1-0", "crates/nuxie-runtime/src/animation.rs:302-2016"]}
  push_vs_poll: {status: isomorphic, cpp_pushes: false, evidence: ["src/animation/linear_animation.cpp:1-0", "crates/nuxie-runtime/src/animation.rs:302-2016"], note: "no compensating cycle-time observer/poll introduced for this row"}
  update_ordering: {status: isomorphic, phases_cpp: ["import/initialize", "advance or evaluate", "apply"], phases_rust: ["import immutable descriptor", "advance or evaluate", "apply"]}
  ownership: {status: adapted, idiom_rule: "AF-7 own-by-value", evidence: ["src/animation/linear_animation.cpp:1-0", "crates/nuxie-runtime/src/animation.rs:302-2016"]}
  compensation:
    status: adapted
    mechanisms: []
    import_time_constants:
      - {name: "runtime enum/index/type descriptor", idiom_rule: "AF-5 import-time devirtualization", evidence: ["crates/nuxie-runtime/src/animation.rs:302-2016"]}
idiom_rules_invoked: ["AF-5 import-time devirtualization", "AF-7 own-by-value"]
confidence: high
notes: "Rust consolidates the C++ object/virtual family into immutable enums, indices, and owned vectors. Those values are fixed during import/instance construction and do not pass the mutation-timing gate. Crate-wide family grep and the listed sibling sweep were clean for this row."
~~~

## B6-0044

~~~yaml
row_id: B6-0044
cpp_files: ["src/animation/linear_animation_instance.cpp"]
rust_module: "crates/nuxie-runtime/src/animation.rs"
subsystem_cluster: animation
sibling_files_swept: ["crates/nuxie-runtime/src/animation.rs", "crates/nuxie-runtime/src/artboard.rs", "crates/nuxie-runtime/src/artboard_data_bind.rs", "crates/nuxie-runtime/src/data_bind_graph.rs", "crates/nuxie-runtime/src/focus.rs", "crates/nuxie-runtime/src/scripting.rs", "crates/nuxie-runtime/src/state_machine.rs", "crates/nuxie-runtime/src/state_machine/instance.rs", "crates/nuxie-runtime/src/state_machine/transition_conditions.rs"]
verdict: DIVERGENT
axes:
  retained_identity: {status: divergent, evidence: ["src/animation/linear_animation_instance.cpp:1-0", "src/animation/linear_animation_instance.cpp:81-100", "crates/nuxie-runtime/src/animation.rs:1540-1557", "crates/nuxie-runtime/src/animation.rs:1727-1758"]}
  push_vs_poll: {status: divergent, cpp_pushes: true, evidence: ["src/animation/state_machine_instance.cpp:3276-3374", "crates/nuxie-runtime/src/animation.rs:1632-1641", "crates/nuxie-runtime/src/animation.rs:1679-1724"]}
  update_ordering: {status: divergent, phases_cpp: ["construct retained BindableProperty targets on state entry", "DataBind writes holders directly", "keyframe apply reads same holders"], phases_rust: ["poll prototype revision", "copy graph source state", "drain updates into value map", "keyframe apply reads copied values"]}
  ownership: {status: divergent, evidence: ["src/animation/linear_animation_instance.cpp:81-100", "crates/nuxie-runtime/src/animation.rs:1555-1557", "crates/nuxie-runtime/src/animation.rs:1606-1629"]}
  compensation:
    status: divergent
    mechanisms:
      - {name: "key_frame_prototype_revision_poll", kind: "AF-2 copied-state refresh / AF-8 invented lifecycle", mutation_gated: true, cpp_counterpart: none, evidence: ["crates/nuxie-runtime/src/animation.rs:1557", "crates/nuxie-runtime/src/animation.rs:1632-1641", "crates/nuxie-runtime/src/data_bind_graph.rs:4400-4435", "crates/nuxie-runtime/src/data_bind_graph.rs:4505-4519"]}
      - {name: "copied_key_frame_value_holder_refresh", kind: "AF-1 retained-identity break / AF-2 copied-state refresh", mutation_gated: true, cpp_counterpart: "none", evidence: ["crates/nuxie-runtime/src/animation.rs:1555", "crates/nuxie-runtime/src/animation.rs:1643-1677", "crates/nuxie-runtime/src/animation.rs:1727-1758"]}
    import_time_constants:
      - {name: "keyed-property type/source snapshots", idiom_rule: "AF-5 import-time devirtualization", evidence: ["crates/nuxie-runtime/src/animation.rs:407-429"]}
idiom_rules_invoked: ["AF-5 import-time devirtualization"]
confidence: high
notes: "The keyword lookalikes at animation.rs:407-429 are import/build constants, not drift tracking. The two listed mechanisms are written during bind/advance and therefore pass the mutation-timing gate."
~~~

## B6-0045

~~~yaml
row_id: B6-0045
cpp_files: ["src/animation/listener_action.cpp"]
rust_module: "crates/nuxie-runtime/src/animation.rs"
subsystem_cluster: animation
sibling_files_swept: ["crates/nuxie-runtime/src/animation.rs", "crates/nuxie-runtime/src/artboard.rs", "crates/nuxie-runtime/src/artboard_data_bind.rs", "crates/nuxie-runtime/src/data_bind_graph.rs", "crates/nuxie-runtime/src/focus.rs", "crates/nuxie-runtime/src/scripting.rs", "crates/nuxie-runtime/src/state_machine.rs", "crates/nuxie-runtime/src/state_machine/instance.rs", "crates/nuxie-runtime/src/state_machine/transition_conditions.rs"]
verdict: ADAPTED
axes:
  retained_identity: {status: adapted, idiom_rule: "AF-7 own-by-value", evidence: ["src/animation/listener_action.cpp:1-0", "crates/nuxie-runtime/src/state_machine.rs:571-778; crates/nuxie-runtime/src/state_machine.rs:2055-2385"]}
  push_vs_poll: {status: isomorphic, cpp_pushes: false, evidence: ["src/animation/listener_action.cpp:1-0", "crates/nuxie-runtime/src/state_machine.rs:571-778; crates/nuxie-runtime/src/state_machine.rs:2055-2385"], note: "no compensating cycle-time observer/poll introduced for this row"}
  update_ordering: {status: isomorphic, phases_cpp: ["import/initialize", "advance or evaluate", "apply"], phases_rust: ["import immutable descriptor", "advance or evaluate", "apply"]}
  ownership: {status: adapted, idiom_rule: "AF-7 own-by-value", evidence: ["src/animation/listener_action.cpp:1-0", "crates/nuxie-runtime/src/state_machine.rs:571-778; crates/nuxie-runtime/src/state_machine.rs:2055-2385"]}
  compensation:
    status: adapted
    mechanisms: []
    import_time_constants:
      - {name: "runtime enum/index/type descriptor", idiom_rule: "AF-5 import-time devirtualization", evidence: ["crates/nuxie-runtime/src/state_machine.rs:571-778; crates/nuxie-runtime/src/state_machine.rs:2055-2385"]}
idiom_rules_invoked: ["AF-5 import-time devirtualization", "AF-7 own-by-value"]
confidence: high
notes: "Rust consolidates the C++ object/virtual family into immutable enums, indices, and owned vectors. Those values are fixed during import/instance construction and do not pass the mutation-timing gate. Crate-wide family grep and the listed sibling sweep were clean for this row."
~~~

## B6-0046

~~~yaml
row_id: B6-0046
cpp_files: ["src/animation/listener_align_target.cpp"]
rust_module: "crates/nuxie-runtime/src/animation.rs"
subsystem_cluster: animation
sibling_files_swept: ["crates/nuxie-runtime/src/animation.rs", "crates/nuxie-runtime/src/artboard.rs", "crates/nuxie-runtime/src/artboard_data_bind.rs", "crates/nuxie-runtime/src/data_bind_graph.rs", "crates/nuxie-runtime/src/focus.rs", "crates/nuxie-runtime/src/scripting.rs", "crates/nuxie-runtime/src/state_machine.rs", "crates/nuxie-runtime/src/state_machine/instance.rs", "crates/nuxie-runtime/src/state_machine/transition_conditions.rs"]
verdict: UNKNOWN
axes:
  retained_identity: {status: unknown, evidence: ["src/animation/listener_align_target.cpp:1-0", "crates/nuxie-runtime/src/state_machine.rs:571-778; crates/nuxie-runtime/src/state_machine.rs:2055-2385"]}
  push_vs_poll: {status: unknown, cpp_pushes: true, evidence: ["src/animation/listener_align_target.cpp:1-0", "crates/nuxie-runtime/src/state_machine.rs:571-778; crates/nuxie-runtime/src/state_machine.rs:2055-2385"]}
  update_ordering: {status: unknown, phases_cpp: ["read complete C++ row"], phases_rust: ["mapped/current subset only"]}
  ownership: {status: unknown, evidence: ["src/animation/listener_align_target.cpp:1-0", "crates/nuxie-runtime/src/state_machine.rs:571-778; crates/nuxie-runtime/src/state_machine.rs:2055-2385"]}
  compensation:
    {status: clear, mechanisms: [], import_time_constants: []}
idiom_rules_invoked: []
confidence: high
notes: "UNKNOWN blocker: No RuntimeScheduledListenerAction variant or importer branch for ListenerAlignTarget exists at state_machine.rs:2055-2207. Family grep and sibling sweep found no mutation-gated mechanism that could justify DIVERGENT; no inference was made."
~~~

## B6-0047

~~~yaml
row_id: B6-0047
cpp_files: ["src/animation/listener_bool_change.cpp"]
rust_module: "crates/nuxie-runtime/src/animation.rs"
subsystem_cluster: animation
sibling_files_swept: ["crates/nuxie-runtime/src/animation.rs", "crates/nuxie-runtime/src/artboard.rs", "crates/nuxie-runtime/src/artboard_data_bind.rs", "crates/nuxie-runtime/src/data_bind_graph.rs", "crates/nuxie-runtime/src/focus.rs", "crates/nuxie-runtime/src/scripting.rs", "crates/nuxie-runtime/src/state_machine.rs", "crates/nuxie-runtime/src/state_machine/instance.rs", "crates/nuxie-runtime/src/state_machine/transition_conditions.rs"]
verdict: UNKNOWN
axes:
  retained_identity: {status: unknown, evidence: ["src/animation/listener_bool_change.cpp:1-0", "crates/nuxie-runtime/src/state_machine.rs:571-778; crates/nuxie-runtime/src/state_machine.rs:2055-2385"]}
  push_vs_poll: {status: unknown, cpp_pushes: true, evidence: ["src/animation/listener_bool_change.cpp:1-0", "crates/nuxie-runtime/src/state_machine.rs:571-778; crates/nuxie-runtime/src/state_machine.rs:2055-2385"]}
  update_ordering: {status: unknown, phases_cpp: ["read complete C++ row"], phases_rust: ["mapped/current subset only"]}
  ownership: {status: unknown, evidence: ["src/animation/listener_bool_change.cpp:1-0", "crates/nuxie-runtime/src/state_machine.rs:571-778; crates/nuxie-runtime/src/state_machine.rs:2055-2385"]}
  compensation:
    {status: clear, mechanisms: [], import_time_constants: []}
idiom_rules_invoked: []
confidence: high
notes: "UNKNOWN blocker: Direct bool changes exist, but nestedInputId is explicitly rejected at state_machine.rs:2316-2324, so the complete C++ row cannot be audited as implemented. Family grep and sibling sweep found no mutation-gated mechanism that could justify DIVERGENT; no inference was made."
~~~

## B6-0048

~~~yaml
row_id: B6-0048
cpp_files: ["src/animation/listener_fire_event.cpp"]
rust_module: "crates/nuxie-runtime/src/animation.rs"
subsystem_cluster: animation
sibling_files_swept: ["crates/nuxie-runtime/src/animation.rs", "crates/nuxie-runtime/src/artboard.rs", "crates/nuxie-runtime/src/artboard_data_bind.rs", "crates/nuxie-runtime/src/data_bind_graph.rs", "crates/nuxie-runtime/src/focus.rs", "crates/nuxie-runtime/src/scripting.rs", "crates/nuxie-runtime/src/state_machine.rs", "crates/nuxie-runtime/src/state_machine/instance.rs", "crates/nuxie-runtime/src/state_machine/transition_conditions.rs"]
verdict: ADAPTED
axes:
  retained_identity: {status: adapted, idiom_rule: "AF-7 own-by-value", evidence: ["src/animation/listener_fire_event.cpp:1-0", "crates/nuxie-runtime/src/state_machine.rs:571-778; crates/nuxie-runtime/src/state_machine.rs:2055-2385"]}
  push_vs_poll: {status: isomorphic, cpp_pushes: false, evidence: ["src/animation/listener_fire_event.cpp:1-0", "crates/nuxie-runtime/src/state_machine.rs:571-778; crates/nuxie-runtime/src/state_machine.rs:2055-2385"], note: "no compensating cycle-time observer/poll introduced for this row"}
  update_ordering: {status: isomorphic, phases_cpp: ["import/initialize", "advance or evaluate", "apply"], phases_rust: ["import immutable descriptor", "advance or evaluate", "apply"]}
  ownership: {status: adapted, idiom_rule: "AF-7 own-by-value", evidence: ["src/animation/listener_fire_event.cpp:1-0", "crates/nuxie-runtime/src/state_machine.rs:571-778; crates/nuxie-runtime/src/state_machine.rs:2055-2385"]}
  compensation:
    status: adapted
    mechanisms: []
    import_time_constants:
      - {name: "runtime enum/index/type descriptor", idiom_rule: "AF-5 import-time devirtualization", evidence: ["crates/nuxie-runtime/src/state_machine.rs:571-778; crates/nuxie-runtime/src/state_machine.rs:2055-2385"]}
idiom_rules_invoked: ["AF-5 import-time devirtualization", "AF-7 own-by-value"]
confidence: high
notes: "Rust consolidates the C++ object/virtual family into immutable enums, indices, and owned vectors. Those values are fixed during import/instance construction and do not pass the mutation-timing gate. Crate-wide family grep and the listed sibling sweep were clean for this row."
~~~

## B6-0049

~~~yaml
row_id: B6-0049
cpp_files: ["src/animation/listener_input_change.cpp"]
rust_module: "crates/nuxie-runtime/src/animation.rs"
subsystem_cluster: animation
sibling_files_swept: ["crates/nuxie-runtime/src/animation.rs", "crates/nuxie-runtime/src/artboard.rs", "crates/nuxie-runtime/src/artboard_data_bind.rs", "crates/nuxie-runtime/src/data_bind_graph.rs", "crates/nuxie-runtime/src/focus.rs", "crates/nuxie-runtime/src/scripting.rs", "crates/nuxie-runtime/src/state_machine.rs", "crates/nuxie-runtime/src/state_machine/instance.rs", "crates/nuxie-runtime/src/state_machine/transition_conditions.rs"]
verdict: UNKNOWN
axes:
  retained_identity: {status: unknown, evidence: ["src/animation/listener_input_change.cpp:1-0", "crates/nuxie-runtime/src/state_machine.rs:571-778; crates/nuxie-runtime/src/state_machine.rs:2055-2385"]}
  push_vs_poll: {status: unknown, cpp_pushes: true, evidence: ["src/animation/listener_input_change.cpp:1-0", "crates/nuxie-runtime/src/state_machine.rs:571-778; crates/nuxie-runtime/src/state_machine.rs:2055-2385"]}
  update_ordering: {status: unknown, phases_cpp: ["read complete C++ row"], phases_rust: ["mapped/current subset only"]}
  ownership: {status: unknown, evidence: ["src/animation/listener_input_change.cpp:1-0", "crates/nuxie-runtime/src/state_machine.rs:571-778; crates/nuxie-runtime/src/state_machine.rs:2055-2385"]}
  compensation:
    {status: clear, mechanisms: [], import_time_constants: []}
idiom_rules_invoked: []
confidence: high
notes: "UNKNOWN blocker: The shared listener-input-change path is incomplete because nestedInputId is explicitly rejected at state_machine.rs:2316-2324. Family grep and sibling sweep found no mutation-gated mechanism that could justify DIVERGENT; no inference was made."
~~~

## B6-0050

~~~yaml
row_id: B6-0050
cpp_files: ["src/animation/listener_invocation.cpp"]
rust_module: "crates/nuxie-runtime/src/animation.rs"
subsystem_cluster: animation
sibling_files_swept: ["crates/nuxie-runtime/src/animation.rs", "crates/nuxie-runtime/src/artboard.rs", "crates/nuxie-runtime/src/artboard_data_bind.rs", "crates/nuxie-runtime/src/data_bind_graph.rs", "crates/nuxie-runtime/src/focus.rs", "crates/nuxie-runtime/src/scripting.rs", "crates/nuxie-runtime/src/state_machine.rs", "crates/nuxie-runtime/src/state_machine/instance.rs", "crates/nuxie-runtime/src/state_machine/transition_conditions.rs"]
verdict: UNKNOWN
axes:
  retained_identity: {status: unknown, evidence: ["src/animation/listener_invocation.cpp:1-0", "crates/nuxie-runtime/src/state_machine.rs:571-778; crates/nuxie-runtime/src/state_machine.rs:2055-2385"]}
  push_vs_poll: {status: unknown, cpp_pushes: true, evidence: ["src/animation/listener_invocation.cpp:1-0", "crates/nuxie-runtime/src/state_machine.rs:571-778; crates/nuxie-runtime/src/state_machine.rs:2055-2385"]}
  update_ordering: {status: unknown, phases_cpp: ["read complete C++ row"], phases_rust: ["mapped/current subset only"]}
  ownership: {status: unknown, evidence: ["src/animation/listener_invocation.cpp:1-0", "crates/nuxie-runtime/src/state_machine.rs:571-778; crates/nuxie-runtime/src/state_machine.rs:2055-2385"]}
  compensation:
    {status: clear, mechanisms: [], import_time_constants: []}
idiom_rules_invoked: []
confidence: high
notes: "UNKNOWN blocker: Rust imports only pointer, event, and view-model listener families at state_machine.rs:718-778; the C++ invocation variants for keyboard, text, focus, gamepad, and semantic inputs are not present. Family grep and sibling sweep found no mutation-gated mechanism that could justify DIVERGENT; no inference was made."
~~~

## B6-0051

~~~yaml
row_id: B6-0051
cpp_files: ["src/animation/listener_number_change.cpp"]
rust_module: "crates/nuxie-runtime/src/animation.rs"
subsystem_cluster: animation
sibling_files_swept: ["crates/nuxie-runtime/src/animation.rs", "crates/nuxie-runtime/src/artboard.rs", "crates/nuxie-runtime/src/artboard_data_bind.rs", "crates/nuxie-runtime/src/data_bind_graph.rs", "crates/nuxie-runtime/src/focus.rs", "crates/nuxie-runtime/src/scripting.rs", "crates/nuxie-runtime/src/state_machine.rs", "crates/nuxie-runtime/src/state_machine/instance.rs", "crates/nuxie-runtime/src/state_machine/transition_conditions.rs"]
verdict: UNKNOWN
axes:
  retained_identity: {status: unknown, evidence: ["src/animation/listener_number_change.cpp:1-0", "crates/nuxie-runtime/src/state_machine.rs:571-778; crates/nuxie-runtime/src/state_machine.rs:2055-2385"]}
  push_vs_poll: {status: unknown, cpp_pushes: true, evidence: ["src/animation/listener_number_change.cpp:1-0", "crates/nuxie-runtime/src/state_machine.rs:571-778; crates/nuxie-runtime/src/state_machine.rs:2055-2385"]}
  update_ordering: {status: unknown, phases_cpp: ["read complete C++ row"], phases_rust: ["mapped/current subset only"]}
  ownership: {status: unknown, evidence: ["src/animation/listener_number_change.cpp:1-0", "crates/nuxie-runtime/src/state_machine.rs:571-778; crates/nuxie-runtime/src/state_machine.rs:2055-2385"]}
  compensation:
    {status: clear, mechanisms: [], import_time_constants: []}
idiom_rules_invoked: []
confidence: high
notes: "UNKNOWN blocker: Direct number changes exist, but nestedInputId is explicitly rejected at state_machine.rs:2316-2324. Family grep and sibling sweep found no mutation-gated mechanism that could justify DIVERGENT; no inference was made."
~~~

## B6-0052

~~~yaml
row_id: B6-0052
cpp_files: ["src/animation/listener_trigger_change.cpp"]
rust_module: "crates/nuxie-runtime/src/animation.rs"
subsystem_cluster: animation
sibling_files_swept: ["crates/nuxie-runtime/src/animation.rs", "crates/nuxie-runtime/src/artboard.rs", "crates/nuxie-runtime/src/artboard_data_bind.rs", "crates/nuxie-runtime/src/data_bind_graph.rs", "crates/nuxie-runtime/src/focus.rs", "crates/nuxie-runtime/src/scripting.rs", "crates/nuxie-runtime/src/state_machine.rs", "crates/nuxie-runtime/src/state_machine/instance.rs", "crates/nuxie-runtime/src/state_machine/transition_conditions.rs"]
verdict: UNKNOWN
axes:
  retained_identity: {status: unknown, evidence: ["src/animation/listener_trigger_change.cpp:1-0", "crates/nuxie-runtime/src/state_machine.rs:571-778; crates/nuxie-runtime/src/state_machine.rs:2055-2385"]}
  push_vs_poll: {status: unknown, cpp_pushes: true, evidence: ["src/animation/listener_trigger_change.cpp:1-0", "crates/nuxie-runtime/src/state_machine.rs:571-778; crates/nuxie-runtime/src/state_machine.rs:2055-2385"]}
  update_ordering: {status: unknown, phases_cpp: ["read complete C++ row"], phases_rust: ["mapped/current subset only"]}
  ownership: {status: unknown, evidence: ["src/animation/listener_trigger_change.cpp:1-0", "crates/nuxie-runtime/src/state_machine.rs:571-778; crates/nuxie-runtime/src/state_machine.rs:2055-2385"]}
  compensation:
    {status: clear, mechanisms: [], import_time_constants: []}
idiom_rules_invoked: []
confidence: high
notes: "UNKNOWN blocker: Direct trigger changes exist, but nestedInputId is explicitly rejected at state_machine.rs:2316-2324. Family grep and sibling sweep found no mutation-gated mechanism that could justify DIVERGENT; no inference was made."
~~~

## B6-0053

~~~yaml
row_id: B6-0053
cpp_files: ["src/animation/listener_types/listener_input_type.cpp"]
rust_module: "crates/nuxie-runtime/src/animation.rs"
subsystem_cluster: animation
sibling_files_swept: ["crates/nuxie-runtime/src/animation.rs", "crates/nuxie-runtime/src/artboard.rs", "crates/nuxie-runtime/src/artboard_data_bind.rs", "crates/nuxie-runtime/src/data_bind_graph.rs", "crates/nuxie-runtime/src/focus.rs", "crates/nuxie-runtime/src/scripting.rs", "crates/nuxie-runtime/src/state_machine.rs", "crates/nuxie-runtime/src/state_machine/instance.rs", "crates/nuxie-runtime/src/state_machine/transition_conditions.rs"]
verdict: ADAPTED
axes:
  retained_identity: {status: adapted, idiom_rule: "AF-7 own-by-value", evidence: ["src/animation/listener_types/listener_input_type.cpp:1-0", "crates/nuxie-runtime/src/state_machine.rs:571-778; crates/nuxie-runtime/src/state_machine.rs:2055-2385"]}
  push_vs_poll: {status: isomorphic, cpp_pushes: false, evidence: ["src/animation/listener_types/listener_input_type.cpp:1-0", "crates/nuxie-runtime/src/state_machine.rs:571-778; crates/nuxie-runtime/src/state_machine.rs:2055-2385"], note: "no compensating cycle-time observer/poll introduced for this row"}
  update_ordering: {status: isomorphic, phases_cpp: ["import/initialize", "advance or evaluate", "apply"], phases_rust: ["import immutable descriptor", "advance or evaluate", "apply"]}
  ownership: {status: adapted, idiom_rule: "AF-7 own-by-value", evidence: ["src/animation/listener_types/listener_input_type.cpp:1-0", "crates/nuxie-runtime/src/state_machine.rs:571-778; crates/nuxie-runtime/src/state_machine.rs:2055-2385"]}
  compensation:
    status: adapted
    mechanisms: []
    import_time_constants:
      - {name: "runtime enum/index/type descriptor", idiom_rule: "AF-5 import-time devirtualization", evidence: ["crates/nuxie-runtime/src/state_machine.rs:571-778; crates/nuxie-runtime/src/state_machine.rs:2055-2385"]}
idiom_rules_invoked: ["AF-5 import-time devirtualization", "AF-7 own-by-value"]
confidence: high
notes: "Rust consolidates the C++ object/virtual family into immutable enums, indices, and owned vectors. Those values are fixed during import/instance construction and do not pass the mutation-timing gate. Crate-wide family grep and the listed sibling sweep were clean for this row."
~~~

## B6-0057

~~~yaml
row_id: B6-0057
cpp_files: ["src/animation/listener_types/listener_input_type_viewmodel.cpp"]
rust_module: "crates/nuxie-runtime/src/animation.rs"
subsystem_cluster: animation
sibling_files_swept: ["crates/nuxie-runtime/src/animation.rs", "crates/nuxie-runtime/src/artboard.rs", "crates/nuxie-runtime/src/artboard_data_bind.rs", "crates/nuxie-runtime/src/data_bind_graph.rs", "crates/nuxie-runtime/src/focus.rs", "crates/nuxie-runtime/src/scripting.rs", "crates/nuxie-runtime/src/state_machine.rs", "crates/nuxie-runtime/src/state_machine/instance.rs", "crates/nuxie-runtime/src/state_machine/transition_conditions.rs"]
verdict: DIVERGENT
axes:
  retained_identity: {status: divergent, evidence: ["src/animation/listener_types/listener_input_type_viewmodel.cpp:1-0", "src/animation/state_machine_instance.cpp:1278-1423", "crates/nuxie-runtime/src/state_machine/instance.rs:649", "crates/nuxie-runtime/src/state_machine/instance.rs:4902-4951"]}
  push_vs_poll: {status: divergent, cpp_pushes: true, evidence: ["src/animation/state_machine_instance.cpp:1401-1423", "src/animation/state_machine_instance.cpp:1481-1488", "crates/nuxie-runtime/src/state_machine/instance.rs:4902-4951"]}
  update_ordering: {status: divergent, phases_cpp: ["register dependent", "property dirt callback enqueues listener", "dispatch"], phases_rust: ["bind/refresh context", "rescan current value", "compare observed copy", "dispatch"]}
  ownership: {status: divergent, evidence: ["src/animation/state_machine_instance.cpp:1278-1423", "crates/nuxie-runtime/src/state_machine/instance.rs:649", "crates/nuxie-runtime/src/state_machine/instance.rs:851-860"]}
  compensation:
    status: divergent
    mechanisms:
      - {name: "listener_observed_copy_rescan", kind: "AF-2 copied-state refresh / AF-4 push-to-poll regression", mutation_gated: true, cpp_counterpart: "none", evidence: ["crates/nuxie-runtime/src/state_machine/instance.rs:649", "crates/nuxie-runtime/src/state_machine/instance.rs:4902-4951"]}
    import_time_constants:
      - {name: "RuntimeListenerType/property path", idiom_rule: "AF-5 import-time devirtualization", evidence: ["crates/nuxie-runtime/src/state_machine.rs:600-645", "crates/nuxie-runtime/src/state_machine.rs:718-790"]}
idiom_rules_invoked: ["AF-5 import-time devirtualization"]
confidence: high
notes: "Current mid-RB-1 state recorded mechanically; the observed-copy work is scheduled by the #RB-1 mini-queue at docs/parity-closeout-status.md:210-225 rather than re-triaged."
~~~

## B6-0058

~~~yaml
row_id: B6-0058
cpp_files: ["src/animation/listener_viewmodel_change.cpp"]
rust_module: "crates/nuxie-runtime/src/animation.rs"
subsystem_cluster: animation
sibling_files_swept: ["crates/nuxie-runtime/src/animation.rs", "crates/nuxie-runtime/src/artboard.rs", "crates/nuxie-runtime/src/artboard_data_bind.rs", "crates/nuxie-runtime/src/data_bind_graph.rs", "crates/nuxie-runtime/src/focus.rs", "crates/nuxie-runtime/src/scripting.rs", "crates/nuxie-runtime/src/state_machine.rs", "crates/nuxie-runtime/src/state_machine/instance.rs", "crates/nuxie-runtime/src/state_machine/transition_conditions.rs"]
verdict: DIVERGENT
axes:
  retained_identity: {status: divergent, evidence: ["src/animation/listener_viewmodel_change.cpp:1-0", "crates/nuxie-runtime/src/state_machine.rs:571-778; crates/nuxie-runtime/src/state_machine.rs:2055-2385", "crates/nuxie-runtime/src/state_machine/instance.rs:79-80"]}
  push_vs_poll: {status: divergent, cpp_pushes: true, evidence: ["src/animation/listener_viewmodel_change.cpp:1-0", "crates/nuxie-runtime/src/state_machine/instance.rs:5174-5220", "crates/nuxie-runtime/src/state_machine/instance.rs:5390-5403"]}
  update_ordering: {status: divergent, phases_cpp: ["retain live view-model/bindable relationship", "source mutation pushes dirt", "evaluate/action"], phases_rust: ["read candidate generations", "rebind copied graph state", "evaluate/action"]}
  ownership: {status: divergent, evidence: ["src/animation/listener_viewmodel_change.cpp:1-0", "crates/nuxie-runtime/src/state_machine/instance.rs:79-80", "crates/nuxie-runtime/src/state_machine/instance.rs:5174-5220"]}
  compensation:
    status: divergent
    mechanisms:
      - {name: "owned_view_model_candidate_generation_rebind", kind: "AF-2 copied-state refresh / AF-4 push-to-poll regression", mutation_gated: true, cpp_counterpart: none, evidence: ["crates/nuxie-runtime/src/state_machine/instance.rs:5174-5220", "crates/nuxie-runtime/src/state_machine/instance.rs:5390-5403"]}
      - {name: "listener_action_candidate_rebind", kind: "AF-8 invented lifecycle", mutation_gated: true, cpp_counterpart: none, evidence: ["crates/nuxie-runtime/src/state_machine/instance.rs:2248-2258"]}
    import_time_constants:
      - {name: "view-model path/type descriptor", idiom_rule: "AF-5 import-time devirtualization", evidence: ["crates/nuxie-runtime/src/state_machine.rs:571-778; crates/nuxie-runtime/src/state_machine.rs:2055-2385"]}
idiom_rules_invoked: ["AF-5 import-time devirtualization"]
confidence: high
notes: "Current mid-RB-1 state recorded mechanically; candidate/generation work is scheduled by the #RB-1 mini-queue at docs/parity-closeout-status.md:210-225. No remediation judgment is made."
~~~

## B6-0059

~~~yaml
row_id: B6-0059
cpp_files: ["src/animation/nested_animation.cpp"]
rust_module: "crates/nuxie-runtime/src/animation.rs"
subsystem_cluster: animation
sibling_files_swept: ["crates/nuxie-runtime/src/animation.rs", "crates/nuxie-runtime/src/artboard.rs", "crates/nuxie-runtime/src/artboard_data_bind.rs", "crates/nuxie-runtime/src/data_bind_graph.rs", "crates/nuxie-runtime/src/focus.rs", "crates/nuxie-runtime/src/scripting.rs", "crates/nuxie-runtime/src/state_machine.rs", "crates/nuxie-runtime/src/state_machine/instance.rs", "crates/nuxie-runtime/src/state_machine/transition_conditions.rs"]
verdict: ADAPTED
axes:
  retained_identity: {status: adapted, idiom_rule: "AF-7 own-by-value", evidence: ["src/animation/nested_animation.cpp:1-0", "crates/nuxie-runtime/src/artboard.rs:641-658; crates/nuxie-runtime/src/artboard.rs:5525-5845; crates/nuxie-runtime/src/artboard.rs:6117-6324"]}
  push_vs_poll: {status: isomorphic, cpp_pushes: false, evidence: ["src/animation/nested_animation.cpp:1-0", "crates/nuxie-runtime/src/artboard.rs:641-658; crates/nuxie-runtime/src/artboard.rs:5525-5845; crates/nuxie-runtime/src/artboard.rs:6117-6324"], note: "no compensating cycle-time observer/poll introduced for this row"}
  update_ordering: {status: isomorphic, phases_cpp: ["import/initialize", "advance or evaluate", "apply"], phases_rust: ["import immutable descriptor", "advance or evaluate", "apply"]}
  ownership: {status: adapted, idiom_rule: "AF-7 own-by-value", evidence: ["src/animation/nested_animation.cpp:1-0", "crates/nuxie-runtime/src/artboard.rs:641-658; crates/nuxie-runtime/src/artboard.rs:5525-5845; crates/nuxie-runtime/src/artboard.rs:6117-6324"]}
  compensation:
    status: adapted
    mechanisms: []
    import_time_constants:
      - {name: "runtime enum/index/type descriptor", idiom_rule: "AF-5 import-time devirtualization", evidence: ["crates/nuxie-runtime/src/artboard.rs:641-658; crates/nuxie-runtime/src/artboard.rs:5525-5845; crates/nuxie-runtime/src/artboard.rs:6117-6324"]}
idiom_rules_invoked: ["AF-5 import-time devirtualization", "AF-7 own-by-value"]
confidence: high
notes: "Rust consolidates the C++ object/virtual family into immutable enums, indices, and owned vectors. Those values are fixed during import/instance construction and do not pass the mutation-timing gate. Crate-wide family grep and the listed sibling sweep were clean for this row."
~~~

## B6-0060

~~~yaml
row_id: B6-0060
cpp_files: ["src/animation/nested_bool.cpp"]
rust_module: "crates/nuxie-runtime/src/animation.rs"
subsystem_cluster: animation
sibling_files_swept: ["crates/nuxie-runtime/src/animation.rs", "crates/nuxie-runtime/src/artboard.rs", "crates/nuxie-runtime/src/artboard_data_bind.rs", "crates/nuxie-runtime/src/data_bind_graph.rs", "crates/nuxie-runtime/src/focus.rs", "crates/nuxie-runtime/src/scripting.rs", "crates/nuxie-runtime/src/state_machine.rs", "crates/nuxie-runtime/src/state_machine/instance.rs", "crates/nuxie-runtime/src/state_machine/transition_conditions.rs"]
verdict: ADAPTED
axes:
  retained_identity: {status: adapted, idiom_rule: "AF-7 own-by-value", evidence: ["src/animation/nested_bool.cpp:1-0", "crates/nuxie-runtime/src/artboard.rs:641-658; crates/nuxie-runtime/src/artboard.rs:5525-5845; crates/nuxie-runtime/src/artboard.rs:6117-6324"]}
  push_vs_poll: {status: isomorphic, cpp_pushes: false, evidence: ["src/animation/nested_bool.cpp:1-0", "crates/nuxie-runtime/src/artboard.rs:641-658; crates/nuxie-runtime/src/artboard.rs:5525-5845; crates/nuxie-runtime/src/artboard.rs:6117-6324"], note: "no compensating cycle-time observer/poll introduced for this row"}
  update_ordering: {status: isomorphic, phases_cpp: ["import/initialize", "advance or evaluate", "apply"], phases_rust: ["import immutable descriptor", "advance or evaluate", "apply"]}
  ownership: {status: adapted, idiom_rule: "AF-7 own-by-value", evidence: ["src/animation/nested_bool.cpp:1-0", "crates/nuxie-runtime/src/artboard.rs:641-658; crates/nuxie-runtime/src/artboard.rs:5525-5845; crates/nuxie-runtime/src/artboard.rs:6117-6324"]}
  compensation:
    status: adapted
    mechanisms: []
    import_time_constants:
      - {name: "runtime enum/index/type descriptor", idiom_rule: "AF-5 import-time devirtualization", evidence: ["crates/nuxie-runtime/src/artboard.rs:641-658; crates/nuxie-runtime/src/artboard.rs:5525-5845; crates/nuxie-runtime/src/artboard.rs:6117-6324"]}
idiom_rules_invoked: ["AF-5 import-time devirtualization", "AF-7 own-by-value"]
confidence: high
notes: "Rust consolidates the C++ object/virtual family into immutable enums, indices, and owned vectors. Those values are fixed during import/instance construction and do not pass the mutation-timing gate. Crate-wide family grep and the listed sibling sweep were clean for this row."
~~~

## B6-0061

~~~yaml
row_id: B6-0061
cpp_files: ["src/animation/nested_linear_animation.cpp"]
rust_module: "crates/nuxie-runtime/src/animation.rs"
subsystem_cluster: animation
sibling_files_swept: ["crates/nuxie-runtime/src/animation.rs", "crates/nuxie-runtime/src/artboard.rs", "crates/nuxie-runtime/src/artboard_data_bind.rs", "crates/nuxie-runtime/src/data_bind_graph.rs", "crates/nuxie-runtime/src/focus.rs", "crates/nuxie-runtime/src/scripting.rs", "crates/nuxie-runtime/src/state_machine.rs", "crates/nuxie-runtime/src/state_machine/instance.rs", "crates/nuxie-runtime/src/state_machine/transition_conditions.rs"]
verdict: ADAPTED
axes:
  retained_identity: {status: adapted, idiom_rule: "AF-7 own-by-value", evidence: ["src/animation/nested_linear_animation.cpp:1-0", "crates/nuxie-runtime/src/artboard.rs:641-658; crates/nuxie-runtime/src/artboard.rs:5525-5845; crates/nuxie-runtime/src/artboard.rs:6117-6324"]}
  push_vs_poll: {status: isomorphic, cpp_pushes: false, evidence: ["src/animation/nested_linear_animation.cpp:1-0", "crates/nuxie-runtime/src/artboard.rs:641-658; crates/nuxie-runtime/src/artboard.rs:5525-5845; crates/nuxie-runtime/src/artboard.rs:6117-6324"], note: "no compensating cycle-time observer/poll introduced for this row"}
  update_ordering: {status: isomorphic, phases_cpp: ["import/initialize", "advance or evaluate", "apply"], phases_rust: ["import immutable descriptor", "advance or evaluate", "apply"]}
  ownership: {status: adapted, idiom_rule: "AF-7 own-by-value", evidence: ["src/animation/nested_linear_animation.cpp:1-0", "crates/nuxie-runtime/src/artboard.rs:641-658; crates/nuxie-runtime/src/artboard.rs:5525-5845; crates/nuxie-runtime/src/artboard.rs:6117-6324"]}
  compensation:
    status: adapted
    mechanisms: []
    import_time_constants:
      - {name: "runtime enum/index/type descriptor", idiom_rule: "AF-5 import-time devirtualization", evidence: ["crates/nuxie-runtime/src/artboard.rs:641-658; crates/nuxie-runtime/src/artboard.rs:5525-5845; crates/nuxie-runtime/src/artboard.rs:6117-6324"]}
idiom_rules_invoked: ["AF-5 import-time devirtualization", "AF-7 own-by-value"]
confidence: high
notes: "Rust consolidates the C++ object/virtual family into immutable enums, indices, and owned vectors. Those values are fixed during import/instance construction and do not pass the mutation-timing gate. Crate-wide family grep and the listed sibling sweep were clean for this row."
~~~

## B6-0062

~~~yaml
row_id: B6-0062
cpp_files: ["src/animation/nested_number.cpp"]
rust_module: "crates/nuxie-runtime/src/animation.rs"
subsystem_cluster: animation
sibling_files_swept: ["crates/nuxie-runtime/src/animation.rs", "crates/nuxie-runtime/src/artboard.rs", "crates/nuxie-runtime/src/artboard_data_bind.rs", "crates/nuxie-runtime/src/data_bind_graph.rs", "crates/nuxie-runtime/src/focus.rs", "crates/nuxie-runtime/src/scripting.rs", "crates/nuxie-runtime/src/state_machine.rs", "crates/nuxie-runtime/src/state_machine/instance.rs", "crates/nuxie-runtime/src/state_machine/transition_conditions.rs"]
verdict: ADAPTED
axes:
  retained_identity: {status: adapted, idiom_rule: "AF-7 own-by-value", evidence: ["src/animation/nested_number.cpp:1-0", "crates/nuxie-runtime/src/artboard.rs:641-658; crates/nuxie-runtime/src/artboard.rs:5525-5845; crates/nuxie-runtime/src/artboard.rs:6117-6324"]}
  push_vs_poll: {status: isomorphic, cpp_pushes: false, evidence: ["src/animation/nested_number.cpp:1-0", "crates/nuxie-runtime/src/artboard.rs:641-658; crates/nuxie-runtime/src/artboard.rs:5525-5845; crates/nuxie-runtime/src/artboard.rs:6117-6324"], note: "no compensating cycle-time observer/poll introduced for this row"}
  update_ordering: {status: isomorphic, phases_cpp: ["import/initialize", "advance or evaluate", "apply"], phases_rust: ["import immutable descriptor", "advance or evaluate", "apply"]}
  ownership: {status: adapted, idiom_rule: "AF-7 own-by-value", evidence: ["src/animation/nested_number.cpp:1-0", "crates/nuxie-runtime/src/artboard.rs:641-658; crates/nuxie-runtime/src/artboard.rs:5525-5845; crates/nuxie-runtime/src/artboard.rs:6117-6324"]}
  compensation:
    status: adapted
    mechanisms: []
    import_time_constants:
      - {name: "runtime enum/index/type descriptor", idiom_rule: "AF-5 import-time devirtualization", evidence: ["crates/nuxie-runtime/src/artboard.rs:641-658; crates/nuxie-runtime/src/artboard.rs:5525-5845; crates/nuxie-runtime/src/artboard.rs:6117-6324"]}
idiom_rules_invoked: ["AF-5 import-time devirtualization", "AF-7 own-by-value"]
confidence: high
notes: "Rust consolidates the C++ object/virtual family into immutable enums, indices, and owned vectors. Those values are fixed during import/instance construction and do not pass the mutation-timing gate. Crate-wide family grep and the listed sibling sweep were clean for this row."
~~~

## B6-0063

~~~yaml
row_id: B6-0063
cpp_files: ["src/animation/nested_remap_animation.cpp"]
rust_module: "crates/nuxie-runtime/src/animation.rs"
subsystem_cluster: animation
sibling_files_swept: ["crates/nuxie-runtime/src/animation.rs", "crates/nuxie-runtime/src/artboard.rs", "crates/nuxie-runtime/src/artboard_data_bind.rs", "crates/nuxie-runtime/src/data_bind_graph.rs", "crates/nuxie-runtime/src/focus.rs", "crates/nuxie-runtime/src/scripting.rs", "crates/nuxie-runtime/src/state_machine.rs", "crates/nuxie-runtime/src/state_machine/instance.rs", "crates/nuxie-runtime/src/state_machine/transition_conditions.rs"]
verdict: ADAPTED
axes:
  retained_identity: {status: adapted, idiom_rule: "AF-7 own-by-value", evidence: ["src/animation/nested_remap_animation.cpp:1-0", "crates/nuxie-runtime/src/artboard.rs:641-658; crates/nuxie-runtime/src/artboard.rs:5525-5845; crates/nuxie-runtime/src/artboard.rs:6117-6324"]}
  push_vs_poll: {status: isomorphic, cpp_pushes: false, evidence: ["src/animation/nested_remap_animation.cpp:1-0", "crates/nuxie-runtime/src/artboard.rs:641-658; crates/nuxie-runtime/src/artboard.rs:5525-5845; crates/nuxie-runtime/src/artboard.rs:6117-6324"], note: "no compensating cycle-time observer/poll introduced for this row"}
  update_ordering: {status: isomorphic, phases_cpp: ["import/initialize", "advance or evaluate", "apply"], phases_rust: ["import immutable descriptor", "advance or evaluate", "apply"]}
  ownership: {status: adapted, idiom_rule: "AF-7 own-by-value", evidence: ["src/animation/nested_remap_animation.cpp:1-0", "crates/nuxie-runtime/src/artboard.rs:641-658; crates/nuxie-runtime/src/artboard.rs:5525-5845; crates/nuxie-runtime/src/artboard.rs:6117-6324"]}
  compensation:
    status: adapted
    mechanisms: []
    import_time_constants:
      - {name: "runtime enum/index/type descriptor", idiom_rule: "AF-5 import-time devirtualization", evidence: ["crates/nuxie-runtime/src/artboard.rs:641-658; crates/nuxie-runtime/src/artboard.rs:5525-5845; crates/nuxie-runtime/src/artboard.rs:6117-6324"]}
idiom_rules_invoked: ["AF-5 import-time devirtualization", "AF-7 own-by-value"]
confidence: high
notes: "Rust consolidates the C++ object/virtual family into immutable enums, indices, and owned vectors. Those values are fixed during import/instance construction and do not pass the mutation-timing gate. Crate-wide family grep and the listed sibling sweep were clean for this row."
~~~

## B6-0064

~~~yaml
row_id: B6-0064
cpp_files: ["src/animation/nested_simple_animation.cpp"]
rust_module: "crates/nuxie-runtime/src/animation.rs"
subsystem_cluster: animation
sibling_files_swept: ["crates/nuxie-runtime/src/animation.rs", "crates/nuxie-runtime/src/artboard.rs", "crates/nuxie-runtime/src/artboard_data_bind.rs", "crates/nuxie-runtime/src/data_bind_graph.rs", "crates/nuxie-runtime/src/focus.rs", "crates/nuxie-runtime/src/scripting.rs", "crates/nuxie-runtime/src/state_machine.rs", "crates/nuxie-runtime/src/state_machine/instance.rs", "crates/nuxie-runtime/src/state_machine/transition_conditions.rs"]
verdict: ADAPTED
axes:
  retained_identity: {status: adapted, idiom_rule: "AF-7 own-by-value", evidence: ["src/animation/nested_simple_animation.cpp:1-0", "crates/nuxie-runtime/src/artboard.rs:641-658; crates/nuxie-runtime/src/artboard.rs:5525-5845; crates/nuxie-runtime/src/artboard.rs:6117-6324"]}
  push_vs_poll: {status: isomorphic, cpp_pushes: false, evidence: ["src/animation/nested_simple_animation.cpp:1-0", "crates/nuxie-runtime/src/artboard.rs:641-658; crates/nuxie-runtime/src/artboard.rs:5525-5845; crates/nuxie-runtime/src/artboard.rs:6117-6324"], note: "no compensating cycle-time observer/poll introduced for this row"}
  update_ordering: {status: isomorphic, phases_cpp: ["import/initialize", "advance or evaluate", "apply"], phases_rust: ["import immutable descriptor", "advance or evaluate", "apply"]}
  ownership: {status: adapted, idiom_rule: "AF-7 own-by-value", evidence: ["src/animation/nested_simple_animation.cpp:1-0", "crates/nuxie-runtime/src/artboard.rs:641-658; crates/nuxie-runtime/src/artboard.rs:5525-5845; crates/nuxie-runtime/src/artboard.rs:6117-6324"]}
  compensation:
    status: adapted
    mechanisms: []
    import_time_constants:
      - {name: "runtime enum/index/type descriptor", idiom_rule: "AF-5 import-time devirtualization", evidence: ["crates/nuxie-runtime/src/artboard.rs:641-658; crates/nuxie-runtime/src/artboard.rs:5525-5845; crates/nuxie-runtime/src/artboard.rs:6117-6324"]}
idiom_rules_invoked: ["AF-5 import-time devirtualization", "AF-7 own-by-value"]
confidence: high
notes: "Rust consolidates the C++ object/virtual family into immutable enums, indices, and owned vectors. Those values are fixed during import/instance construction and do not pass the mutation-timing gate. Crate-wide family grep and the listed sibling sweep were clean for this row."
~~~

## B6-0065

~~~yaml
row_id: B6-0065
cpp_files: ["src/animation/nested_state_machine.cpp"]
rust_module: "crates/nuxie-runtime/src/animation.rs"
subsystem_cluster: animation
sibling_files_swept: ["crates/nuxie-runtime/src/animation.rs", "crates/nuxie-runtime/src/artboard.rs", "crates/nuxie-runtime/src/artboard_data_bind.rs", "crates/nuxie-runtime/src/data_bind_graph.rs", "crates/nuxie-runtime/src/focus.rs", "crates/nuxie-runtime/src/scripting.rs", "crates/nuxie-runtime/src/state_machine.rs", "crates/nuxie-runtime/src/state_machine/instance.rs", "crates/nuxie-runtime/src/state_machine/transition_conditions.rs"]
verdict: ADAPTED
axes:
  retained_identity: {status: adapted, idiom_rule: "AF-7 own-by-value", evidence: ["src/animation/nested_state_machine.cpp:1-0", "crates/nuxie-runtime/src/artboard.rs:641-658; crates/nuxie-runtime/src/artboard.rs:5525-5845; crates/nuxie-runtime/src/artboard.rs:6117-6324"]}
  push_vs_poll: {status: isomorphic, cpp_pushes: false, evidence: ["src/animation/nested_state_machine.cpp:1-0", "crates/nuxie-runtime/src/artboard.rs:641-658; crates/nuxie-runtime/src/artboard.rs:5525-5845; crates/nuxie-runtime/src/artboard.rs:6117-6324"], note: "no compensating cycle-time observer/poll introduced for this row"}
  update_ordering: {status: isomorphic, phases_cpp: ["import/initialize", "advance or evaluate", "apply"], phases_rust: ["import immutable descriptor", "advance or evaluate", "apply"]}
  ownership: {status: adapted, idiom_rule: "AF-7 own-by-value", evidence: ["src/animation/nested_state_machine.cpp:1-0", "crates/nuxie-runtime/src/artboard.rs:641-658; crates/nuxie-runtime/src/artboard.rs:5525-5845; crates/nuxie-runtime/src/artboard.rs:6117-6324"]}
  compensation:
    status: adapted
    mechanisms: []
    import_time_constants:
      - {name: "runtime enum/index/type descriptor", idiom_rule: "AF-5 import-time devirtualization", evidence: ["crates/nuxie-runtime/src/artboard.rs:641-658; crates/nuxie-runtime/src/artboard.rs:5525-5845; crates/nuxie-runtime/src/artboard.rs:6117-6324"]}
idiom_rules_invoked: ["AF-5 import-time devirtualization", "AF-7 own-by-value"]
confidence: high
notes: "Rust consolidates the C++ object/virtual family into immutable enums, indices, and owned vectors. Those values are fixed during import/instance construction and do not pass the mutation-timing gate. Crate-wide family grep and the listed sibling sweep were clean for this row."
~~~

## B6-0066

~~~yaml
row_id: B6-0066
cpp_files: ["src/animation/nested_trigger.cpp"]
rust_module: "crates/nuxie-runtime/src/animation.rs"
subsystem_cluster: animation
sibling_files_swept: ["crates/nuxie-runtime/src/animation.rs", "crates/nuxie-runtime/src/artboard.rs", "crates/nuxie-runtime/src/artboard_data_bind.rs", "crates/nuxie-runtime/src/data_bind_graph.rs", "crates/nuxie-runtime/src/focus.rs", "crates/nuxie-runtime/src/scripting.rs", "crates/nuxie-runtime/src/state_machine.rs", "crates/nuxie-runtime/src/state_machine/instance.rs", "crates/nuxie-runtime/src/state_machine/transition_conditions.rs"]
verdict: ADAPTED
axes:
  retained_identity: {status: adapted, idiom_rule: "AF-7 own-by-value", evidence: ["src/animation/nested_trigger.cpp:1-0", "crates/nuxie-runtime/src/artboard.rs:641-658; crates/nuxie-runtime/src/artboard.rs:5525-5845; crates/nuxie-runtime/src/artboard.rs:6117-6324"]}
  push_vs_poll: {status: isomorphic, cpp_pushes: false, evidence: ["src/animation/nested_trigger.cpp:1-0", "crates/nuxie-runtime/src/artboard.rs:641-658; crates/nuxie-runtime/src/artboard.rs:5525-5845; crates/nuxie-runtime/src/artboard.rs:6117-6324"], note: "no compensating cycle-time observer/poll introduced for this row"}
  update_ordering: {status: isomorphic, phases_cpp: ["import/initialize", "advance or evaluate", "apply"], phases_rust: ["import immutable descriptor", "advance or evaluate", "apply"]}
  ownership: {status: adapted, idiom_rule: "AF-7 own-by-value", evidence: ["src/animation/nested_trigger.cpp:1-0", "crates/nuxie-runtime/src/artboard.rs:641-658; crates/nuxie-runtime/src/artboard.rs:5525-5845; crates/nuxie-runtime/src/artboard.rs:6117-6324"]}
  compensation:
    status: adapted
    mechanisms: []
    import_time_constants:
      - {name: "runtime enum/index/type descriptor", idiom_rule: "AF-5 import-time devirtualization", evidence: ["crates/nuxie-runtime/src/artboard.rs:641-658; crates/nuxie-runtime/src/artboard.rs:5525-5845; crates/nuxie-runtime/src/artboard.rs:6117-6324"]}
idiom_rules_invoked: ["AF-5 import-time devirtualization", "AF-7 own-by-value"]
confidence: high
notes: "Rust consolidates the C++ object/virtual family into immutable enums, indices, and owned vectors. Those values are fixed during import/instance construction and do not pass the mutation-timing gate. Crate-wide family grep and the listed sibling sweep were clean for this row."
~~~

## B6-0067

~~~yaml
row_id: B6-0067
cpp_files: ["src/animation/property_recorder.cpp"]
rust_module: "crates/nuxie-runtime/src/animation.rs"
subsystem_cluster: animation
sibling_files_swept: ["crates/nuxie-runtime/src/animation.rs", "crates/nuxie-runtime/src/artboard.rs", "crates/nuxie-runtime/src/artboard_data_bind.rs", "crates/nuxie-runtime/src/data_bind_graph.rs", "crates/nuxie-runtime/src/focus.rs", "crates/nuxie-runtime/src/scripting.rs", "crates/nuxie-runtime/src/state_machine.rs", "crates/nuxie-runtime/src/state_machine/instance.rs", "crates/nuxie-runtime/src/state_machine/transition_conditions.rs"]
verdict: UNKNOWN
axes:
  retained_identity: {status: unknown, evidence: ["src/animation/property_recorder.cpp:1-0", "crates/nuxie-runtime/src/animation.rs:302-2016"]}
  push_vs_poll: {status: unknown, cpp_pushes: true, evidence: ["src/animation/property_recorder.cpp:1-0", "crates/nuxie-runtime/src/animation.rs:302-2016"]}
  update_ordering: {status: unknown, phases_cpp: ["read complete C++ row"], phases_rust: ["mapped/current subset only"]}
  ownership: {status: unknown, evidence: ["src/animation/property_recorder.cpp:1-0", "crates/nuxie-runtime/src/animation.rs:302-2016"]}
  compensation:
    {status: clear, mechanisms: [], import_time_constants: []}
idiom_rules_invoked: []
confidence: high
notes: "UNKNOWN blocker: No PropertyRecorder/PropertyRecorderGroup runtime counterpart or mapped Rust region was found in the crate-wide and sibling sweep. Family grep and sibling sweep found no mutation-gated mechanism that could justify DIVERGENT; no inference was made."
~~~

## B6-0068

~~~yaml
row_id: B6-0068
cpp_files: ["src/animation/scripted_listener_action.cpp"]
rust_module: "crates/nuxie-runtime/src/animation.rs"
subsystem_cluster: animation
sibling_files_swept: ["crates/nuxie-runtime/src/animation.rs", "crates/nuxie-runtime/src/artboard.rs", "crates/nuxie-runtime/src/artboard_data_bind.rs", "crates/nuxie-runtime/src/data_bind_graph.rs", "crates/nuxie-runtime/src/focus.rs", "crates/nuxie-runtime/src/scripting.rs", "crates/nuxie-runtime/src/state_machine.rs", "crates/nuxie-runtime/src/state_machine/instance.rs", "crates/nuxie-runtime/src/state_machine/transition_conditions.rs"]
verdict: ADAPTED
axes:
  retained_identity: {status: adapted, idiom_rule: "AF-7 own-by-value", evidence: ["src/animation/scripted_listener_action.cpp:1-0", "crates/nuxie-runtime/src/scripting.rs:97-259; crates/nuxie-runtime/src/state_machine/instance.rs:2292-2377"]}
  push_vs_poll: {status: isomorphic, cpp_pushes: false, evidence: ["src/animation/scripted_listener_action.cpp:1-0", "crates/nuxie-runtime/src/scripting.rs:97-259; crates/nuxie-runtime/src/state_machine/instance.rs:2292-2377"], note: "no compensating cycle-time observer/poll introduced for this row"}
  update_ordering: {status: isomorphic, phases_cpp: ["import/initialize", "advance or evaluate", "apply"], phases_rust: ["import immutable descriptor", "advance or evaluate", "apply"]}
  ownership: {status: adapted, idiom_rule: "AF-7 own-by-value", evidence: ["src/animation/scripted_listener_action.cpp:1-0", "crates/nuxie-runtime/src/scripting.rs:97-259; crates/nuxie-runtime/src/state_machine/instance.rs:2292-2377"]}
  compensation:
    status: adapted
    mechanisms: []
    import_time_constants:
      - {name: "runtime enum/index/type descriptor", idiom_rule: "AF-5 import-time devirtualization", evidence: ["crates/nuxie-runtime/src/scripting.rs:97-259; crates/nuxie-runtime/src/state_machine/instance.rs:2292-2377"]}
idiom_rules_invoked: ["AF-5 import-time devirtualization", "AF-7 own-by-value"]
confidence: high
notes: "Rust consolidates the C++ object/virtual family into immutable enums, indices, and owned vectors. Those values are fixed during import/instance construction and do not pass the mutation-timing gate. Crate-wide family grep and the listed sibling sweep were clean for this row."
~~~

## B6-0069

~~~yaml
row_id: B6-0069
cpp_files: ["src/animation/scripted_transition_condition.cpp"]
rust_module: "crates/nuxie-runtime/src/animation.rs"
subsystem_cluster: animation
sibling_files_swept: ["crates/nuxie-runtime/src/animation.rs", "crates/nuxie-runtime/src/artboard.rs", "crates/nuxie-runtime/src/artboard_data_bind.rs", "crates/nuxie-runtime/src/data_bind_graph.rs", "crates/nuxie-runtime/src/focus.rs", "crates/nuxie-runtime/src/scripting.rs", "crates/nuxie-runtime/src/state_machine.rs", "crates/nuxie-runtime/src/state_machine/instance.rs", "crates/nuxie-runtime/src/state_machine/transition_conditions.rs"]
verdict: ADAPTED
axes:
  retained_identity: {status: adapted, idiom_rule: "AF-7 own-by-value", evidence: ["src/animation/scripted_transition_condition.cpp:1-0", "crates/nuxie-runtime/src/state_machine/transition_conditions.rs:26-360"]}
  push_vs_poll: {status: isomorphic, cpp_pushes: false, evidence: ["src/animation/scripted_transition_condition.cpp:1-0", "crates/nuxie-runtime/src/state_machine/transition_conditions.rs:26-360"], note: "no compensating cycle-time observer/poll introduced for this row"}
  update_ordering: {status: isomorphic, phases_cpp: ["import/initialize", "advance or evaluate", "apply"], phases_rust: ["import immutable descriptor", "advance or evaluate", "apply"]}
  ownership: {status: adapted, idiom_rule: "AF-7 own-by-value", evidence: ["src/animation/scripted_transition_condition.cpp:1-0", "crates/nuxie-runtime/src/state_machine/transition_conditions.rs:26-360"]}
  compensation:
    status: adapted
    mechanisms: []
    import_time_constants:
      - {name: "runtime enum/index/type descriptor", idiom_rule: "AF-5 import-time devirtualization", evidence: ["crates/nuxie-runtime/src/state_machine/transition_conditions.rs:26-360"]}
idiom_rules_invoked: ["AF-5 import-time devirtualization", "AF-7 own-by-value"]
confidence: high
notes: "Rust consolidates the C++ object/virtual family into immutable enums, indices, and owned vectors. Those values are fixed during import/instance construction and do not pass the mutation-timing gate. Crate-wide family grep and the listed sibling sweep were clean for this row."
~~~

## B6-0071

~~~yaml
row_id: B6-0071
cpp_files: ["src/animation/state_instance.cpp"]
rust_module: "crates/nuxie-runtime/src/animation.rs"
subsystem_cluster: animation
sibling_files_swept: ["crates/nuxie-runtime/src/animation.rs", "crates/nuxie-runtime/src/artboard.rs", "crates/nuxie-runtime/src/artboard_data_bind.rs", "crates/nuxie-runtime/src/data_bind_graph.rs", "crates/nuxie-runtime/src/focus.rs", "crates/nuxie-runtime/src/scripting.rs", "crates/nuxie-runtime/src/state_machine.rs", "crates/nuxie-runtime/src/state_machine/instance.rs", "crates/nuxie-runtime/src/state_machine/transition_conditions.rs"]
verdict: ADAPTED
axes:
  retained_identity: {status: adapted, idiom_rule: "AF-7 own-by-value", evidence: ["src/animation/state_instance.cpp:1-0", "crates/nuxie-runtime/src/state_machine.rs:47-460; crates/nuxie-runtime/src/state_machine/instance.rs:822-1055; crates/nuxie-runtime/src/state_machine/instance.rs:5533-5910"]}
  push_vs_poll: {status: isomorphic, cpp_pushes: false, evidence: ["src/animation/state_instance.cpp:1-0", "crates/nuxie-runtime/src/state_machine.rs:47-460; crates/nuxie-runtime/src/state_machine/instance.rs:822-1055; crates/nuxie-runtime/src/state_machine/instance.rs:5533-5910"], note: "no compensating cycle-time observer/poll introduced for this row"}
  update_ordering: {status: isomorphic, phases_cpp: ["import/initialize", "advance or evaluate", "apply"], phases_rust: ["import immutable descriptor", "advance or evaluate", "apply"]}
  ownership: {status: adapted, idiom_rule: "AF-7 own-by-value", evidence: ["src/animation/state_instance.cpp:1-0", "crates/nuxie-runtime/src/state_machine.rs:47-460; crates/nuxie-runtime/src/state_machine/instance.rs:822-1055; crates/nuxie-runtime/src/state_machine/instance.rs:5533-5910"]}
  compensation:
    status: adapted
    mechanisms: []
    import_time_constants:
      - {name: "runtime enum/index/type descriptor", idiom_rule: "AF-5 import-time devirtualization", evidence: ["crates/nuxie-runtime/src/state_machine.rs:47-460; crates/nuxie-runtime/src/state_machine/instance.rs:822-1055; crates/nuxie-runtime/src/state_machine/instance.rs:5533-5910"]}
idiom_rules_invoked: ["AF-5 import-time devirtualization", "AF-7 own-by-value"]
confidence: high
notes: "Rust consolidates the C++ object/virtual family into immutable enums, indices, and owned vectors. Those values are fixed during import/instance construction and do not pass the mutation-timing gate. Crate-wide family grep and the listed sibling sweep were clean for this row."
~~~

## B6-0072

~~~yaml
row_id: B6-0072
cpp_files: ["src/animation/state_machine.cpp"]
rust_module: "crates/nuxie-runtime/src/animation.rs"
subsystem_cluster: animation
sibling_files_swept: ["crates/nuxie-runtime/src/animation.rs", "crates/nuxie-runtime/src/artboard.rs", "crates/nuxie-runtime/src/artboard_data_bind.rs", "crates/nuxie-runtime/src/data_bind_graph.rs", "crates/nuxie-runtime/src/focus.rs", "crates/nuxie-runtime/src/scripting.rs", "crates/nuxie-runtime/src/state_machine.rs", "crates/nuxie-runtime/src/state_machine/instance.rs", "crates/nuxie-runtime/src/state_machine/transition_conditions.rs"]
verdict: ADAPTED
axes:
  retained_identity: {status: adapted, idiom_rule: "AF-7 own-by-value", evidence: ["src/animation/state_machine.cpp:1-0", "crates/nuxie-runtime/src/state_machine.rs:47-460; crates/nuxie-runtime/src/state_machine/instance.rs:822-1055; crates/nuxie-runtime/src/state_machine/instance.rs:5533-5910"]}
  push_vs_poll: {status: isomorphic, cpp_pushes: false, evidence: ["src/animation/state_machine.cpp:1-0", "crates/nuxie-runtime/src/state_machine.rs:47-460; crates/nuxie-runtime/src/state_machine/instance.rs:822-1055; crates/nuxie-runtime/src/state_machine/instance.rs:5533-5910"], note: "no compensating cycle-time observer/poll introduced for this row"}
  update_ordering: {status: isomorphic, phases_cpp: ["import/initialize", "advance or evaluate", "apply"], phases_rust: ["import immutable descriptor", "advance or evaluate", "apply"]}
  ownership: {status: adapted, idiom_rule: "AF-7 own-by-value", evidence: ["src/animation/state_machine.cpp:1-0", "crates/nuxie-runtime/src/state_machine.rs:47-460; crates/nuxie-runtime/src/state_machine/instance.rs:822-1055; crates/nuxie-runtime/src/state_machine/instance.rs:5533-5910"]}
  compensation:
    status: adapted
    mechanisms: []
    import_time_constants:
      - {name: "runtime enum/index/type descriptor", idiom_rule: "AF-5 import-time devirtualization", evidence: ["crates/nuxie-runtime/src/state_machine.rs:47-460; crates/nuxie-runtime/src/state_machine/instance.rs:822-1055; crates/nuxie-runtime/src/state_machine/instance.rs:5533-5910"]}
idiom_rules_invoked: ["AF-5 import-time devirtualization", "AF-7 own-by-value"]
confidence: high
notes: "Rust consolidates the C++ object/virtual family into immutable enums, indices, and owned vectors. Those values are fixed during import/instance construction and do not pass the mutation-timing gate. Crate-wide family grep and the listed sibling sweep were clean for this row."
~~~

## B6-0073

~~~yaml
row_id: B6-0073
cpp_files: ["src/animation/state_machine_fire_action.cpp"]
rust_module: "crates/nuxie-runtime/src/animation.rs"
subsystem_cluster: animation
sibling_files_swept: ["crates/nuxie-runtime/src/animation.rs", "crates/nuxie-runtime/src/artboard.rs", "crates/nuxie-runtime/src/artboard_data_bind.rs", "crates/nuxie-runtime/src/data_bind_graph.rs", "crates/nuxie-runtime/src/focus.rs", "crates/nuxie-runtime/src/scripting.rs", "crates/nuxie-runtime/src/state_machine.rs", "crates/nuxie-runtime/src/state_machine/instance.rs", "crates/nuxie-runtime/src/state_machine/transition_conditions.rs"]
verdict: ADAPTED
axes:
  retained_identity: {status: adapted, idiom_rule: "AF-7 own-by-value", evidence: ["src/animation/state_machine_fire_action.cpp:1-0", "crates/nuxie-runtime/src/state_machine.rs:47-460; crates/nuxie-runtime/src/state_machine/instance.rs:822-1055; crates/nuxie-runtime/src/state_machine/instance.rs:5533-5910"]}
  push_vs_poll: {status: isomorphic, cpp_pushes: false, evidence: ["src/animation/state_machine_fire_action.cpp:1-0", "crates/nuxie-runtime/src/state_machine.rs:47-460; crates/nuxie-runtime/src/state_machine/instance.rs:822-1055; crates/nuxie-runtime/src/state_machine/instance.rs:5533-5910"], note: "no compensating cycle-time observer/poll introduced for this row"}
  update_ordering: {status: isomorphic, phases_cpp: ["import/initialize", "advance or evaluate", "apply"], phases_rust: ["import immutable descriptor", "advance or evaluate", "apply"]}
  ownership: {status: adapted, idiom_rule: "AF-7 own-by-value", evidence: ["src/animation/state_machine_fire_action.cpp:1-0", "crates/nuxie-runtime/src/state_machine.rs:47-460; crates/nuxie-runtime/src/state_machine/instance.rs:822-1055; crates/nuxie-runtime/src/state_machine/instance.rs:5533-5910"]}
  compensation:
    status: adapted
    mechanisms: []
    import_time_constants:
      - {name: "runtime enum/index/type descriptor", idiom_rule: "AF-5 import-time devirtualization", evidence: ["crates/nuxie-runtime/src/state_machine.rs:47-460; crates/nuxie-runtime/src/state_machine/instance.rs:822-1055; crates/nuxie-runtime/src/state_machine/instance.rs:5533-5910"]}
idiom_rules_invoked: ["AF-5 import-time devirtualization", "AF-7 own-by-value"]
confidence: high
notes: "Rust consolidates the C++ object/virtual family into immutable enums, indices, and owned vectors. Those values are fixed during import/instance construction and do not pass the mutation-timing gate. Crate-wide family grep and the listed sibling sweep were clean for this row."
~~~

## B6-0074

~~~yaml
row_id: B6-0074
cpp_files: ["src/animation/state_machine_fire_trigger.cpp"]
rust_module: "crates/nuxie-runtime/src/animation.rs"
subsystem_cluster: animation
sibling_files_swept: ["crates/nuxie-runtime/src/animation.rs", "crates/nuxie-runtime/src/artboard.rs", "crates/nuxie-runtime/src/artboard_data_bind.rs", "crates/nuxie-runtime/src/data_bind_graph.rs", "crates/nuxie-runtime/src/focus.rs", "crates/nuxie-runtime/src/scripting.rs", "crates/nuxie-runtime/src/state_machine.rs", "crates/nuxie-runtime/src/state_machine/instance.rs", "crates/nuxie-runtime/src/state_machine/transition_conditions.rs"]
verdict: ADAPTED
axes:
  retained_identity: {status: adapted, idiom_rule: "AF-7 own-by-value", evidence: ["src/animation/state_machine_fire_trigger.cpp:1-0", "crates/nuxie-runtime/src/state_machine.rs:47-460; crates/nuxie-runtime/src/state_machine/instance.rs:822-1055; crates/nuxie-runtime/src/state_machine/instance.rs:5533-5910"]}
  push_vs_poll: {status: isomorphic, cpp_pushes: false, evidence: ["src/animation/state_machine_fire_trigger.cpp:1-0", "crates/nuxie-runtime/src/state_machine.rs:47-460; crates/nuxie-runtime/src/state_machine/instance.rs:822-1055; crates/nuxie-runtime/src/state_machine/instance.rs:5533-5910"], note: "no compensating cycle-time observer/poll introduced for this row"}
  update_ordering: {status: isomorphic, phases_cpp: ["import/initialize", "advance or evaluate", "apply"], phases_rust: ["import immutable descriptor", "advance or evaluate", "apply"]}
  ownership: {status: adapted, idiom_rule: "AF-7 own-by-value", evidence: ["src/animation/state_machine_fire_trigger.cpp:1-0", "crates/nuxie-runtime/src/state_machine.rs:47-460; crates/nuxie-runtime/src/state_machine/instance.rs:822-1055; crates/nuxie-runtime/src/state_machine/instance.rs:5533-5910"]}
  compensation:
    status: adapted
    mechanisms: []
    import_time_constants:
      - {name: "runtime enum/index/type descriptor", idiom_rule: "AF-5 import-time devirtualization", evidence: ["crates/nuxie-runtime/src/state_machine.rs:47-460; crates/nuxie-runtime/src/state_machine/instance.rs:822-1055; crates/nuxie-runtime/src/state_machine/instance.rs:5533-5910"]}
idiom_rules_invoked: ["AF-5 import-time devirtualization", "AF-7 own-by-value"]
confidence: high
notes: "Rust consolidates the C++ object/virtual family into immutable enums, indices, and owned vectors. Those values are fixed during import/instance construction and do not pass the mutation-timing gate. Crate-wide family grep and the listed sibling sweep were clean for this row."
~~~

## B6-0075

~~~yaml
row_id: B6-0075
cpp_files: ["src/animation/state_machine_input.cpp"]
rust_module: "crates/nuxie-runtime/src/animation.rs"
subsystem_cluster: animation
sibling_files_swept: ["crates/nuxie-runtime/src/animation.rs", "crates/nuxie-runtime/src/artboard.rs", "crates/nuxie-runtime/src/artboard_data_bind.rs", "crates/nuxie-runtime/src/data_bind_graph.rs", "crates/nuxie-runtime/src/focus.rs", "crates/nuxie-runtime/src/scripting.rs", "crates/nuxie-runtime/src/state_machine.rs", "crates/nuxie-runtime/src/state_machine/instance.rs", "crates/nuxie-runtime/src/state_machine/transition_conditions.rs"]
verdict: ADAPTED
axes:
  retained_identity: {status: adapted, idiom_rule: "AF-7 own-by-value", evidence: ["src/animation/state_machine_input.cpp:1-0", "crates/nuxie-runtime/src/state_machine.rs:47-460; crates/nuxie-runtime/src/state_machine/instance.rs:822-1055; crates/nuxie-runtime/src/state_machine/instance.rs:5533-5910"]}
  push_vs_poll: {status: isomorphic, cpp_pushes: false, evidence: ["src/animation/state_machine_input.cpp:1-0", "crates/nuxie-runtime/src/state_machine.rs:47-460; crates/nuxie-runtime/src/state_machine/instance.rs:822-1055; crates/nuxie-runtime/src/state_machine/instance.rs:5533-5910"], note: "no compensating cycle-time observer/poll introduced for this row"}
  update_ordering: {status: isomorphic, phases_cpp: ["import/initialize", "advance or evaluate", "apply"], phases_rust: ["import immutable descriptor", "advance or evaluate", "apply"]}
  ownership: {status: adapted, idiom_rule: "AF-7 own-by-value", evidence: ["src/animation/state_machine_input.cpp:1-0", "crates/nuxie-runtime/src/state_machine.rs:47-460; crates/nuxie-runtime/src/state_machine/instance.rs:822-1055; crates/nuxie-runtime/src/state_machine/instance.rs:5533-5910"]}
  compensation:
    status: adapted
    mechanisms: []
    import_time_constants:
      - {name: "runtime enum/index/type descriptor", idiom_rule: "AF-5 import-time devirtualization", evidence: ["crates/nuxie-runtime/src/state_machine.rs:47-460; crates/nuxie-runtime/src/state_machine/instance.rs:822-1055; crates/nuxie-runtime/src/state_machine/instance.rs:5533-5910"]}
idiom_rules_invoked: ["AF-5 import-time devirtualization", "AF-7 own-by-value"]
confidence: high
notes: "Rust consolidates the C++ object/virtual family into immutable enums, indices, and owned vectors. Those values are fixed during import/instance construction and do not pass the mutation-timing gate. Crate-wide family grep and the listed sibling sweep were clean for this row."
~~~

## B6-0076

~~~yaml
row_id: B6-0076
cpp_files: ["src/animation/state_machine_input_instance.cpp"]
rust_module: "crates/nuxie-runtime/src/animation.rs"
subsystem_cluster: animation
sibling_files_swept: ["crates/nuxie-runtime/src/animation.rs", "crates/nuxie-runtime/src/artboard.rs", "crates/nuxie-runtime/src/artboard_data_bind.rs", "crates/nuxie-runtime/src/data_bind_graph.rs", "crates/nuxie-runtime/src/focus.rs", "crates/nuxie-runtime/src/scripting.rs", "crates/nuxie-runtime/src/state_machine.rs", "crates/nuxie-runtime/src/state_machine/instance.rs", "crates/nuxie-runtime/src/state_machine/transition_conditions.rs"]
verdict: ADAPTED
axes:
  retained_identity: {status: adapted, idiom_rule: "AF-7 own-by-value", evidence: ["src/animation/state_machine_input_instance.cpp:1-0", "crates/nuxie-runtime/src/state_machine.rs:47-460; crates/nuxie-runtime/src/state_machine/instance.rs:822-1055; crates/nuxie-runtime/src/state_machine/instance.rs:5533-5910"]}
  push_vs_poll: {status: isomorphic, cpp_pushes: false, evidence: ["src/animation/state_machine_input_instance.cpp:1-0", "crates/nuxie-runtime/src/state_machine.rs:47-460; crates/nuxie-runtime/src/state_machine/instance.rs:822-1055; crates/nuxie-runtime/src/state_machine/instance.rs:5533-5910"], note: "no compensating cycle-time observer/poll introduced for this row"}
  update_ordering: {status: isomorphic, phases_cpp: ["import/initialize", "advance or evaluate", "apply"], phases_rust: ["import immutable descriptor", "advance or evaluate", "apply"]}
  ownership: {status: adapted, idiom_rule: "AF-7 own-by-value", evidence: ["src/animation/state_machine_input_instance.cpp:1-0", "crates/nuxie-runtime/src/state_machine.rs:47-460; crates/nuxie-runtime/src/state_machine/instance.rs:822-1055; crates/nuxie-runtime/src/state_machine/instance.rs:5533-5910"]}
  compensation:
    status: adapted
    mechanisms: []
    import_time_constants:
      - {name: "runtime enum/index/type descriptor", idiom_rule: "AF-5 import-time devirtualization", evidence: ["crates/nuxie-runtime/src/state_machine.rs:47-460; crates/nuxie-runtime/src/state_machine/instance.rs:822-1055; crates/nuxie-runtime/src/state_machine/instance.rs:5533-5910"]}
idiom_rules_invoked: ["AF-5 import-time devirtualization", "AF-7 own-by-value"]
confidence: high
notes: "Rust consolidates the C++ object/virtual family into immutable enums, indices, and owned vectors. Those values are fixed during import/instance construction and do not pass the mutation-timing gate. C++ valueChanged and Rust changed-result/needs_advance preserve the push boundary. Crate-wide family grep and the listed sibling sweep were clean for this row."
~~~

## B6-0077

~~~yaml
row_id: B6-0077
cpp_files: ["src/animation/state_machine_instance.cpp"]
rust_module: "crates/nuxie-runtime/src/animation.rs"
subsystem_cluster: animation
sibling_files_swept: ["crates/nuxie-runtime/src/animation.rs", "crates/nuxie-runtime/src/artboard.rs", "crates/nuxie-runtime/src/artboard_data_bind.rs", "crates/nuxie-runtime/src/data_bind_graph.rs", "crates/nuxie-runtime/src/focus.rs", "crates/nuxie-runtime/src/scripting.rs", "crates/nuxie-runtime/src/state_machine.rs", "crates/nuxie-runtime/src/state_machine/instance.rs", "crates/nuxie-runtime/src/state_machine/transition_conditions.rs"]
verdict: DIVERGENT
axes:
  retained_identity: {status: divergent, evidence: ["src/animation/state_machine_instance.cpp:1-0", "crates/nuxie-runtime/src/state_machine/instance.rs:79-89", "crates/nuxie-runtime/src/animation.rs:1555-1557"]}
  push_vs_poll: {status: divergent, cpp_pushes: true, evidence: ["src/animation/state_machine_instance.cpp:1278-1488", "src/animation/state_machine_instance.cpp:3276-3374", "crates/nuxie-runtime/src/state_machine/instance.rs:4902-4951", "crates/nuxie-runtime/src/state_machine/instance.rs:5174-5220"]}
  update_ordering: {status: divergent, phases_cpp: ["register retained dependents/holders at state entry", "push dirt/write live targets", "advance/apply"], phases_rust: ["refresh focus and owned-context generations", "rescan listener/trigger snapshots", "sync keyframe prototype", "advance/apply"]}
  ownership: {status: divergent, evidence: ["src/animation/state_machine_instance.cpp:1-0", "crates/nuxie-runtime/src/state_machine/instance.rs:79-89", "crates/nuxie-runtime/src/animation.rs:1540-1557"]}
  compensation:
    status: divergent
    mechanisms:
      - {name: "key_frame_prototype_revision_poll", kind: "AF-2/AF-8", mutation_gated: true, cpp_counterpart: none, evidence: ["crates/nuxie-runtime/src/animation.rs:1632-1641", "crates/nuxie-runtime/src/data_bind_graph.rs:4400-4435"]}
      - {name: "copied_key_frame_value_holder_refresh", kind: "AF-1/AF-2", mutation_gated: true, cpp_counterpart: "none", evidence: ["crates/nuxie-runtime/src/animation.rs:1643-1677", "crates/nuxie-runtime/src/animation.rs:1727-1758"]}
      - {name: "owned_view_model_candidate_generation_rebind", kind: "AF-2/AF-4", mutation_gated: true, cpp_counterpart: none, evidence: ["crates/nuxie-runtime/src/state_machine/instance.rs:5174-5220", "crates/nuxie-runtime/src/state_machine/instance.rs:5390-5403"]}
      - {name: "listener_observed_copy_rescan", kind: "AF-2/AF-4", mutation_gated: true, cpp_counterpart: "none", evidence: ["crates/nuxie-runtime/src/state_machine/instance.rs:4902-4951"]}
      - {name: "view_model_trigger_observed_reset", kind: "AF-2 copied-state rescan", mutation_gated: true, cpp_counterpart: none, evidence: ["crates/nuxie-runtime/src/state_machine.rs:2527-2591", "crates/nuxie-runtime/src/state_machine/instance.rs:5984-6082"]}
      - {name: "focus_tree_descriptor_rescan", kind: "AF-8 invented lifecycle", mutation_gated: true, cpp_counterpart: none, evidence: ["crates/nuxie-runtime/src/state_machine/instance.rs:5754-5755", "crates/nuxie-runtime/src/focus.rs:837-930"]}
    import_time_constants:
      - {name: "listener/keyed-property discriminants", idiom_rule: "AF-5 import-time devirtualization", evidence: ["crates/nuxie-runtime/src/animation.rs:407-429", "crates/nuxie-runtime/src/state_machine.rs:600-645"]}
idiom_rules_invoked: ["AF-5 import-time devirtualization"]
confidence: high
notes: "This umbrella C++ implementation row crosses keyframe, listener, focus, and current RB-1 view-model paths. Current RB-1 work is cited from the mini-queue at docs/parity-closeout-status.md:210-225; the record makes no remediation decision."
~~~

## B6-0078

~~~yaml
row_id: B6-0078
cpp_files: ["src/animation/state_machine_layer.cpp"]
rust_module: "crates/nuxie-runtime/src/animation.rs"
subsystem_cluster: animation
sibling_files_swept: ["crates/nuxie-runtime/src/animation.rs", "crates/nuxie-runtime/src/artboard.rs", "crates/nuxie-runtime/src/artboard_data_bind.rs", "crates/nuxie-runtime/src/data_bind_graph.rs", "crates/nuxie-runtime/src/focus.rs", "crates/nuxie-runtime/src/scripting.rs", "crates/nuxie-runtime/src/state_machine.rs", "crates/nuxie-runtime/src/state_machine/instance.rs", "crates/nuxie-runtime/src/state_machine/transition_conditions.rs"]
verdict: ADAPTED
axes:
  retained_identity: {status: adapted, idiom_rule: "AF-7 own-by-value", evidence: ["src/animation/state_machine_layer.cpp:1-0", "crates/nuxie-runtime/src/state_machine.rs:47-460; crates/nuxie-runtime/src/state_machine/instance.rs:822-1055; crates/nuxie-runtime/src/state_machine/instance.rs:5533-5910"]}
  push_vs_poll: {status: isomorphic, cpp_pushes: false, evidence: ["src/animation/state_machine_layer.cpp:1-0", "crates/nuxie-runtime/src/state_machine.rs:47-460; crates/nuxie-runtime/src/state_machine/instance.rs:822-1055; crates/nuxie-runtime/src/state_machine/instance.rs:5533-5910"], note: "no compensating cycle-time observer/poll introduced for this row"}
  update_ordering: {status: isomorphic, phases_cpp: ["import/initialize", "advance or evaluate", "apply"], phases_rust: ["import immutable descriptor", "advance or evaluate", "apply"]}
  ownership: {status: adapted, idiom_rule: "AF-7 own-by-value", evidence: ["src/animation/state_machine_layer.cpp:1-0", "crates/nuxie-runtime/src/state_machine.rs:47-460; crates/nuxie-runtime/src/state_machine/instance.rs:822-1055; crates/nuxie-runtime/src/state_machine/instance.rs:5533-5910"]}
  compensation:
    status: adapted
    mechanisms: []
    import_time_constants:
      - {name: "runtime enum/index/type descriptor", idiom_rule: "AF-5 import-time devirtualization", evidence: ["crates/nuxie-runtime/src/state_machine.rs:47-460; crates/nuxie-runtime/src/state_machine/instance.rs:822-1055; crates/nuxie-runtime/src/state_machine/instance.rs:5533-5910"]}
idiom_rules_invoked: ["AF-5 import-time devirtualization", "AF-7 own-by-value"]
confidence: high
notes: "Rust consolidates the C++ object/virtual family into immutable enums, indices, and owned vectors. Those values are fixed during import/instance construction and do not pass the mutation-timing gate. Crate-wide family grep and the listed sibling sweep were clean for this row."
~~~

## B6-0079

~~~yaml
row_id: B6-0079
cpp_files: ["src/animation/state_machine_listener.cpp"]
rust_module: "crates/nuxie-runtime/src/animation.rs"
subsystem_cluster: animation
sibling_files_swept: ["crates/nuxie-runtime/src/animation.rs", "crates/nuxie-runtime/src/artboard.rs", "crates/nuxie-runtime/src/artboard_data_bind.rs", "crates/nuxie-runtime/src/data_bind_graph.rs", "crates/nuxie-runtime/src/focus.rs", "crates/nuxie-runtime/src/scripting.rs", "crates/nuxie-runtime/src/state_machine.rs", "crates/nuxie-runtime/src/state_machine/instance.rs", "crates/nuxie-runtime/src/state_machine/transition_conditions.rs"]
verdict: DIVERGENT
axes:
  retained_identity: {status: divergent, evidence: ["src/animation/state_machine_listener.cpp:1-0", "src/animation/state_machine_instance.cpp:1278-1423", "crates/nuxie-runtime/src/state_machine/instance.rs:649", "crates/nuxie-runtime/src/state_machine/instance.rs:4902-4951"]}
  push_vs_poll: {status: divergent, cpp_pushes: true, evidence: ["src/animation/state_machine_instance.cpp:1401-1423", "src/animation/state_machine_instance.cpp:1481-1488", "crates/nuxie-runtime/src/state_machine/instance.rs:4902-4951"]}
  update_ordering: {status: divergent, phases_cpp: ["register dependent", "property dirt callback enqueues listener", "dispatch"], phases_rust: ["bind/refresh context", "rescan current value", "compare observed copy", "dispatch"]}
  ownership: {status: divergent, evidence: ["src/animation/state_machine_instance.cpp:1278-1423", "crates/nuxie-runtime/src/state_machine/instance.rs:649", "crates/nuxie-runtime/src/state_machine/instance.rs:851-860"]}
  compensation:
    status: divergent
    mechanisms:
      - {name: "listener_observed_copy_rescan", kind: "AF-2 copied-state refresh / AF-4 push-to-poll regression", mutation_gated: true, cpp_counterpart: "none", evidence: ["crates/nuxie-runtime/src/state_machine/instance.rs:649", "crates/nuxie-runtime/src/state_machine/instance.rs:4902-4951"]}
    import_time_constants:
      - {name: "RuntimeListenerType/property path", idiom_rule: "AF-5 import-time devirtualization", evidence: ["crates/nuxie-runtime/src/state_machine.rs:600-645", "crates/nuxie-runtime/src/state_machine.rs:718-790"]}
idiom_rules_invoked: ["AF-5 import-time devirtualization"]
confidence: high
notes: "Current mid-RB-1 state recorded mechanically; the observed-copy work is scheduled by the #RB-1 mini-queue at docs/parity-closeout-status.md:210-225 rather than re-triaged."
~~~

## B6-0080

~~~yaml
row_id: B6-0080
cpp_files: ["src/animation/state_machine_listener_single.cpp"]
rust_module: "crates/nuxie-runtime/src/animation.rs"
subsystem_cluster: animation
sibling_files_swept: ["crates/nuxie-runtime/src/animation.rs", "crates/nuxie-runtime/src/artboard.rs", "crates/nuxie-runtime/src/artboard_data_bind.rs", "crates/nuxie-runtime/src/data_bind_graph.rs", "crates/nuxie-runtime/src/focus.rs", "crates/nuxie-runtime/src/scripting.rs", "crates/nuxie-runtime/src/state_machine.rs", "crates/nuxie-runtime/src/state_machine/instance.rs", "crates/nuxie-runtime/src/state_machine/transition_conditions.rs"]
verdict: DIVERGENT
axes:
  retained_identity: {status: divergent, evidence: ["src/animation/state_machine_listener_single.cpp:1-0", "src/animation/state_machine_instance.cpp:1278-1423", "crates/nuxie-runtime/src/state_machine/instance.rs:649", "crates/nuxie-runtime/src/state_machine/instance.rs:4902-4951"]}
  push_vs_poll: {status: divergent, cpp_pushes: true, evidence: ["src/animation/state_machine_instance.cpp:1401-1423", "src/animation/state_machine_instance.cpp:1481-1488", "crates/nuxie-runtime/src/state_machine/instance.rs:4902-4951"]}
  update_ordering: {status: divergent, phases_cpp: ["register dependent", "property dirt callback enqueues listener", "dispatch"], phases_rust: ["bind/refresh context", "rescan current value", "compare observed copy", "dispatch"]}
  ownership: {status: divergent, evidence: ["src/animation/state_machine_instance.cpp:1278-1423", "crates/nuxie-runtime/src/state_machine/instance.rs:649", "crates/nuxie-runtime/src/state_machine/instance.rs:851-860"]}
  compensation:
    status: divergent
    mechanisms:
      - {name: "listener_observed_copy_rescan", kind: "AF-2 copied-state refresh / AF-4 push-to-poll regression", mutation_gated: true, cpp_counterpart: "none", evidence: ["crates/nuxie-runtime/src/state_machine/instance.rs:649", "crates/nuxie-runtime/src/state_machine/instance.rs:4902-4951"]}
    import_time_constants:
      - {name: "RuntimeListenerType/property path", idiom_rule: "AF-5 import-time devirtualization", evidence: ["crates/nuxie-runtime/src/state_machine.rs:600-645", "crates/nuxie-runtime/src/state_machine.rs:718-790"]}
idiom_rules_invoked: ["AF-5 import-time devirtualization"]
confidence: high
notes: "Current mid-RB-1 state recorded mechanically; the observed-copy work is scheduled by the #RB-1 mini-queue at docs/parity-closeout-status.md:210-225 rather than re-triaged."
~~~

## B6-0081

~~~yaml
row_id: B6-0081
cpp_files: ["src/animation/state_transition.cpp"]
rust_module: "crates/nuxie-runtime/src/animation.rs"
subsystem_cluster: animation
sibling_files_swept: ["crates/nuxie-runtime/src/animation.rs", "crates/nuxie-runtime/src/artboard.rs", "crates/nuxie-runtime/src/artboard_data_bind.rs", "crates/nuxie-runtime/src/data_bind_graph.rs", "crates/nuxie-runtime/src/focus.rs", "crates/nuxie-runtime/src/scripting.rs", "crates/nuxie-runtime/src/state_machine.rs", "crates/nuxie-runtime/src/state_machine/instance.rs", "crates/nuxie-runtime/src/state_machine/transition_conditions.rs"]
verdict: ADAPTED
axes:
  retained_identity: {status: adapted, idiom_rule: "AF-7 own-by-value", evidence: ["src/animation/state_transition.cpp:1-0", "crates/nuxie-runtime/src/state_machine.rs:47-460; crates/nuxie-runtime/src/state_machine/instance.rs:822-1055; crates/nuxie-runtime/src/state_machine/instance.rs:5533-5910"]}
  push_vs_poll: {status: isomorphic, cpp_pushes: false, evidence: ["src/animation/state_transition.cpp:1-0", "crates/nuxie-runtime/src/state_machine.rs:47-460; crates/nuxie-runtime/src/state_machine/instance.rs:822-1055; crates/nuxie-runtime/src/state_machine/instance.rs:5533-5910"], note: "no compensating cycle-time observer/poll introduced for this row"}
  update_ordering: {status: isomorphic, phases_cpp: ["import/initialize", "advance or evaluate", "apply"], phases_rust: ["import immutable descriptor", "advance or evaluate", "apply"]}
  ownership: {status: adapted, idiom_rule: "AF-7 own-by-value", evidence: ["src/animation/state_transition.cpp:1-0", "crates/nuxie-runtime/src/state_machine.rs:47-460; crates/nuxie-runtime/src/state_machine/instance.rs:822-1055; crates/nuxie-runtime/src/state_machine/instance.rs:5533-5910"]}
  compensation:
    status: adapted
    mechanisms: []
    import_time_constants:
      - {name: "runtime enum/index/type descriptor", idiom_rule: "AF-5 import-time devirtualization", evidence: ["crates/nuxie-runtime/src/state_machine.rs:47-460; crates/nuxie-runtime/src/state_machine/instance.rs:822-1055; crates/nuxie-runtime/src/state_machine/instance.rs:5533-5910"]}
idiom_rules_invoked: ["AF-5 import-time devirtualization", "AF-7 own-by-value"]
confidence: high
notes: "Rust consolidates the C++ object/virtual family into immutable enums, indices, and owned vectors. Those values are fixed during import/instance construction and do not pass the mutation-timing gate. Crate-wide family grep and the listed sibling sweep were clean for this row."
~~~

## B6-0082

~~~yaml
row_id: B6-0082
cpp_files: ["src/animation/system_state_instance.cpp"]
rust_module: "crates/nuxie-runtime/src/animation.rs"
subsystem_cluster: animation
sibling_files_swept: ["crates/nuxie-runtime/src/animation.rs", "crates/nuxie-runtime/src/artboard.rs", "crates/nuxie-runtime/src/artboard_data_bind.rs", "crates/nuxie-runtime/src/data_bind_graph.rs", "crates/nuxie-runtime/src/focus.rs", "crates/nuxie-runtime/src/scripting.rs", "crates/nuxie-runtime/src/state_machine.rs", "crates/nuxie-runtime/src/state_machine/instance.rs", "crates/nuxie-runtime/src/state_machine/transition_conditions.rs"]
verdict: ADAPTED
axes:
  retained_identity: {status: adapted, idiom_rule: "AF-7 own-by-value", evidence: ["src/animation/system_state_instance.cpp:1-0", "crates/nuxie-runtime/src/state_machine.rs:47-460; crates/nuxie-runtime/src/state_machine/instance.rs:822-1055; crates/nuxie-runtime/src/state_machine/instance.rs:5533-5910"]}
  push_vs_poll: {status: isomorphic, cpp_pushes: false, evidence: ["src/animation/system_state_instance.cpp:1-0", "crates/nuxie-runtime/src/state_machine.rs:47-460; crates/nuxie-runtime/src/state_machine/instance.rs:822-1055; crates/nuxie-runtime/src/state_machine/instance.rs:5533-5910"], note: "no compensating cycle-time observer/poll introduced for this row"}
  update_ordering: {status: isomorphic, phases_cpp: ["import/initialize", "advance or evaluate", "apply"], phases_rust: ["import immutable descriptor", "advance or evaluate", "apply"]}
  ownership: {status: adapted, idiom_rule: "AF-7 own-by-value", evidence: ["src/animation/system_state_instance.cpp:1-0", "crates/nuxie-runtime/src/state_machine.rs:47-460; crates/nuxie-runtime/src/state_machine/instance.rs:822-1055; crates/nuxie-runtime/src/state_machine/instance.rs:5533-5910"]}
  compensation:
    status: adapted
    mechanisms: []
    import_time_constants:
      - {name: "runtime enum/index/type descriptor", idiom_rule: "AF-5 import-time devirtualization", evidence: ["crates/nuxie-runtime/src/state_machine.rs:47-460; crates/nuxie-runtime/src/state_machine/instance.rs:822-1055; crates/nuxie-runtime/src/state_machine/instance.rs:5533-5910"]}
idiom_rules_invoked: ["AF-5 import-time devirtualization", "AF-7 own-by-value"]
confidence: high
notes: "Rust consolidates the C++ object/virtual family into immutable enums, indices, and owned vectors. Those values are fixed during import/instance construction and do not pass the mutation-timing gate. Crate-wide family grep and the listed sibling sweep were clean for this row."
~~~

## B6-0084

~~~yaml
row_id: B6-0084
cpp_files: ["src/animation/transition_bool_condition.cpp"]
rust_module: "crates/nuxie-runtime/src/animation.rs"
subsystem_cluster: animation
sibling_files_swept: ["crates/nuxie-runtime/src/animation.rs", "crates/nuxie-runtime/src/artboard.rs", "crates/nuxie-runtime/src/artboard_data_bind.rs", "crates/nuxie-runtime/src/data_bind_graph.rs", "crates/nuxie-runtime/src/focus.rs", "crates/nuxie-runtime/src/scripting.rs", "crates/nuxie-runtime/src/state_machine.rs", "crates/nuxie-runtime/src/state_machine/instance.rs", "crates/nuxie-runtime/src/state_machine/transition_conditions.rs"]
verdict: ADAPTED
axes:
  retained_identity: {status: adapted, idiom_rule: "AF-7 own-by-value", evidence: ["src/animation/transition_bool_condition.cpp:1-0", "crates/nuxie-runtime/src/state_machine/transition_conditions.rs:26-360"]}
  push_vs_poll: {status: isomorphic, cpp_pushes: false, evidence: ["src/animation/transition_bool_condition.cpp:1-0", "crates/nuxie-runtime/src/state_machine/transition_conditions.rs:26-360"], note: "no compensating cycle-time observer/poll introduced for this row"}
  update_ordering: {status: isomorphic, phases_cpp: ["import/initialize", "advance or evaluate", "apply"], phases_rust: ["import immutable descriptor", "advance or evaluate", "apply"]}
  ownership: {status: adapted, idiom_rule: "AF-7 own-by-value", evidence: ["src/animation/transition_bool_condition.cpp:1-0", "crates/nuxie-runtime/src/state_machine/transition_conditions.rs:26-360"]}
  compensation:
    status: adapted
    mechanisms: []
    import_time_constants:
      - {name: "runtime enum/index/type descriptor", idiom_rule: "AF-5 import-time devirtualization", evidence: ["crates/nuxie-runtime/src/state_machine/transition_conditions.rs:26-360"]}
idiom_rules_invoked: ["AF-5 import-time devirtualization", "AF-7 own-by-value"]
confidence: high
notes: "Rust consolidates the C++ object/virtual family into immutable enums, indices, and owned vectors. Those values are fixed during import/instance construction and do not pass the mutation-timing gate. Crate-wide family grep and the listed sibling sweep were clean for this row."
~~~

## B6-0085

~~~yaml
row_id: B6-0085
cpp_files: ["src/animation/transition_comparator.cpp"]
rust_module: "crates/nuxie-runtime/src/animation.rs"
subsystem_cluster: animation
sibling_files_swept: ["crates/nuxie-runtime/src/animation.rs", "crates/nuxie-runtime/src/artboard.rs", "crates/nuxie-runtime/src/artboard_data_bind.rs", "crates/nuxie-runtime/src/data_bind_graph.rs", "crates/nuxie-runtime/src/focus.rs", "crates/nuxie-runtime/src/scripting.rs", "crates/nuxie-runtime/src/state_machine.rs", "crates/nuxie-runtime/src/state_machine/instance.rs", "crates/nuxie-runtime/src/state_machine/transition_conditions.rs"]
verdict: ADAPTED
axes:
  retained_identity: {status: adapted, idiom_rule: "AF-7 own-by-value", evidence: ["src/animation/transition_comparator.cpp:1-0", "crates/nuxie-runtime/src/state_machine/transition_conditions.rs:26-360"]}
  push_vs_poll: {status: isomorphic, cpp_pushes: false, evidence: ["src/animation/transition_comparator.cpp:1-0", "crates/nuxie-runtime/src/state_machine/transition_conditions.rs:26-360"], note: "no compensating cycle-time observer/poll introduced for this row"}
  update_ordering: {status: isomorphic, phases_cpp: ["import/initialize", "advance or evaluate", "apply"], phases_rust: ["import immutable descriptor", "advance or evaluate", "apply"]}
  ownership: {status: adapted, idiom_rule: "AF-7 own-by-value", evidence: ["src/animation/transition_comparator.cpp:1-0", "crates/nuxie-runtime/src/state_machine/transition_conditions.rs:26-360"]}
  compensation:
    status: adapted
    mechanisms: []
    import_time_constants:
      - {name: "runtime enum/index/type descriptor", idiom_rule: "AF-5 import-time devirtualization", evidence: ["crates/nuxie-runtime/src/state_machine/transition_conditions.rs:26-360"]}
idiom_rules_invoked: ["AF-5 import-time devirtualization", "AF-7 own-by-value"]
confidence: high
notes: "Rust consolidates the C++ object/virtual family into immutable enums, indices, and owned vectors. Those values are fixed during import/instance construction and do not pass the mutation-timing gate. Crate-wide family grep and the listed sibling sweep were clean for this row."
~~~

## B6-0086

~~~yaml
row_id: B6-0086
cpp_files: ["src/animation/transition_condition.cpp"]
rust_module: "crates/nuxie-runtime/src/animation.rs"
subsystem_cluster: animation
sibling_files_swept: ["crates/nuxie-runtime/src/animation.rs", "crates/nuxie-runtime/src/artboard.rs", "crates/nuxie-runtime/src/artboard_data_bind.rs", "crates/nuxie-runtime/src/data_bind_graph.rs", "crates/nuxie-runtime/src/focus.rs", "crates/nuxie-runtime/src/scripting.rs", "crates/nuxie-runtime/src/state_machine.rs", "crates/nuxie-runtime/src/state_machine/instance.rs", "crates/nuxie-runtime/src/state_machine/transition_conditions.rs"]
verdict: ADAPTED
axes:
  retained_identity: {status: adapted, idiom_rule: "AF-7 own-by-value", evidence: ["src/animation/transition_condition.cpp:1-0", "crates/nuxie-runtime/src/state_machine/transition_conditions.rs:26-360"]}
  push_vs_poll: {status: isomorphic, cpp_pushes: false, evidence: ["src/animation/transition_condition.cpp:1-0", "crates/nuxie-runtime/src/state_machine/transition_conditions.rs:26-360"], note: "no compensating cycle-time observer/poll introduced for this row"}
  update_ordering: {status: isomorphic, phases_cpp: ["import/initialize", "advance or evaluate", "apply"], phases_rust: ["import immutable descriptor", "advance or evaluate", "apply"]}
  ownership: {status: adapted, idiom_rule: "AF-7 own-by-value", evidence: ["src/animation/transition_condition.cpp:1-0", "crates/nuxie-runtime/src/state_machine/transition_conditions.rs:26-360"]}
  compensation:
    status: adapted
    mechanisms: []
    import_time_constants:
      - {name: "runtime enum/index/type descriptor", idiom_rule: "AF-5 import-time devirtualization", evidence: ["crates/nuxie-runtime/src/state_machine/transition_conditions.rs:26-360"]}
idiom_rules_invoked: ["AF-5 import-time devirtualization", "AF-7 own-by-value"]
confidence: high
notes: "Rust consolidates the C++ object/virtual family into immutable enums, indices, and owned vectors. Those values are fixed during import/instance construction and do not pass the mutation-timing gate. Crate-wide family grep and the listed sibling sweep were clean for this row."
~~~

## B6-0087

~~~yaml
row_id: B6-0087
cpp_files: ["src/animation/transition_focus_condition.cpp"]
rust_module: "crates/nuxie-runtime/src/animation.rs"
subsystem_cluster: animation
sibling_files_swept: ["crates/nuxie-runtime/src/animation.rs", "crates/nuxie-runtime/src/artboard.rs", "crates/nuxie-runtime/src/artboard_data_bind.rs", "crates/nuxie-runtime/src/data_bind_graph.rs", "crates/nuxie-runtime/src/focus.rs", "crates/nuxie-runtime/src/scripting.rs", "crates/nuxie-runtime/src/state_machine.rs", "crates/nuxie-runtime/src/state_machine/instance.rs", "crates/nuxie-runtime/src/state_machine/transition_conditions.rs"]
verdict: ADAPTED
axes:
  retained_identity: {status: adapted, idiom_rule: "AF-7 own-by-value", evidence: ["src/animation/transition_focus_condition.cpp:1-0", "crates/nuxie-runtime/src/state_machine/transition_conditions.rs:26-360"]}
  push_vs_poll: {status: isomorphic, cpp_pushes: false, evidence: ["src/animation/transition_focus_condition.cpp:1-0", "crates/nuxie-runtime/src/state_machine/transition_conditions.rs:26-360"], note: "no compensating cycle-time observer/poll introduced for this row"}
  update_ordering: {status: isomorphic, phases_cpp: ["import/initialize", "advance or evaluate", "apply"], phases_rust: ["import immutable descriptor", "advance or evaluate", "apply"]}
  ownership: {status: adapted, idiom_rule: "AF-7 own-by-value", evidence: ["src/animation/transition_focus_condition.cpp:1-0", "crates/nuxie-runtime/src/state_machine/transition_conditions.rs:26-360"]}
  compensation:
    status: adapted
    mechanisms: []
    import_time_constants:
      - {name: "runtime enum/index/type descriptor", idiom_rule: "AF-5 import-time devirtualization", evidence: ["crates/nuxie-runtime/src/state_machine/transition_conditions.rs:26-360"]}
idiom_rules_invoked: ["AF-5 import-time devirtualization", "AF-7 own-by-value"]
confidence: high
notes: "Rust consolidates the C++ object/virtual family into immutable enums, indices, and owned vectors. Those values are fixed during import/instance construction and do not pass the mutation-timing gate. Crate-wide family grep and the listed sibling sweep were clean for this row."
~~~

## B6-0088

~~~yaml
row_id: B6-0088
cpp_files: ["src/animation/transition_input_condition.cpp"]
rust_module: "crates/nuxie-runtime/src/animation.rs"
subsystem_cluster: animation
sibling_files_swept: ["crates/nuxie-runtime/src/animation.rs", "crates/nuxie-runtime/src/artboard.rs", "crates/nuxie-runtime/src/artboard_data_bind.rs", "crates/nuxie-runtime/src/data_bind_graph.rs", "crates/nuxie-runtime/src/focus.rs", "crates/nuxie-runtime/src/scripting.rs", "crates/nuxie-runtime/src/state_machine.rs", "crates/nuxie-runtime/src/state_machine/instance.rs", "crates/nuxie-runtime/src/state_machine/transition_conditions.rs"]
verdict: ADAPTED
axes:
  retained_identity: {status: adapted, idiom_rule: "AF-7 own-by-value", evidence: ["src/animation/transition_input_condition.cpp:1-0", "crates/nuxie-runtime/src/state_machine/transition_conditions.rs:26-360"]}
  push_vs_poll: {status: isomorphic, cpp_pushes: false, evidence: ["src/animation/transition_input_condition.cpp:1-0", "crates/nuxie-runtime/src/state_machine/transition_conditions.rs:26-360"], note: "no compensating cycle-time observer/poll introduced for this row"}
  update_ordering: {status: isomorphic, phases_cpp: ["import/initialize", "advance or evaluate", "apply"], phases_rust: ["import immutable descriptor", "advance or evaluate", "apply"]}
  ownership: {status: adapted, idiom_rule: "AF-7 own-by-value", evidence: ["src/animation/transition_input_condition.cpp:1-0", "crates/nuxie-runtime/src/state_machine/transition_conditions.rs:26-360"]}
  compensation:
    status: adapted
    mechanisms: []
    import_time_constants:
      - {name: "runtime enum/index/type descriptor", idiom_rule: "AF-5 import-time devirtualization", evidence: ["crates/nuxie-runtime/src/state_machine/transition_conditions.rs:26-360"]}
idiom_rules_invoked: ["AF-5 import-time devirtualization", "AF-7 own-by-value"]
confidence: high
notes: "Rust consolidates the C++ object/virtual family into immutable enums, indices, and owned vectors. Those values are fixed during import/instance construction and do not pass the mutation-timing gate. Crate-wide family grep and the listed sibling sweep were clean for this row."
~~~

## B6-0089

~~~yaml
row_id: B6-0089
cpp_files: ["src/animation/transition_number_condition.cpp"]
rust_module: "crates/nuxie-runtime/src/animation.rs"
subsystem_cluster: animation
sibling_files_swept: ["crates/nuxie-runtime/src/animation.rs", "crates/nuxie-runtime/src/artboard.rs", "crates/nuxie-runtime/src/artboard_data_bind.rs", "crates/nuxie-runtime/src/data_bind_graph.rs", "crates/nuxie-runtime/src/focus.rs", "crates/nuxie-runtime/src/scripting.rs", "crates/nuxie-runtime/src/state_machine.rs", "crates/nuxie-runtime/src/state_machine/instance.rs", "crates/nuxie-runtime/src/state_machine/transition_conditions.rs"]
verdict: ADAPTED
axes:
  retained_identity: {status: adapted, idiom_rule: "AF-7 own-by-value", evidence: ["src/animation/transition_number_condition.cpp:1-0", "crates/nuxie-runtime/src/state_machine/transition_conditions.rs:26-360"]}
  push_vs_poll: {status: isomorphic, cpp_pushes: false, evidence: ["src/animation/transition_number_condition.cpp:1-0", "crates/nuxie-runtime/src/state_machine/transition_conditions.rs:26-360"], note: "no compensating cycle-time observer/poll introduced for this row"}
  update_ordering: {status: isomorphic, phases_cpp: ["import/initialize", "advance or evaluate", "apply"], phases_rust: ["import immutable descriptor", "advance or evaluate", "apply"]}
  ownership: {status: adapted, idiom_rule: "AF-7 own-by-value", evidence: ["src/animation/transition_number_condition.cpp:1-0", "crates/nuxie-runtime/src/state_machine/transition_conditions.rs:26-360"]}
  compensation:
    status: adapted
    mechanisms: []
    import_time_constants:
      - {name: "runtime enum/index/type descriptor", idiom_rule: "AF-5 import-time devirtualization", evidence: ["crates/nuxie-runtime/src/state_machine/transition_conditions.rs:26-360"]}
idiom_rules_invoked: ["AF-5 import-time devirtualization", "AF-7 own-by-value"]
confidence: high
notes: "Rust consolidates the C++ object/virtual family into immutable enums, indices, and owned vectors. Those values are fixed during import/instance construction and do not pass the mutation-timing gate. Crate-wide family grep and the listed sibling sweep were clean for this row."
~~~

## B6-0090

~~~yaml
row_id: B6-0090
cpp_files: ["src/animation/transition_property_comparator.cpp"]
rust_module: "crates/nuxie-runtime/src/animation.rs"
subsystem_cluster: animation
sibling_files_swept: ["crates/nuxie-runtime/src/animation.rs", "crates/nuxie-runtime/src/artboard.rs", "crates/nuxie-runtime/src/artboard_data_bind.rs", "crates/nuxie-runtime/src/data_bind_graph.rs", "crates/nuxie-runtime/src/focus.rs", "crates/nuxie-runtime/src/scripting.rs", "crates/nuxie-runtime/src/state_machine.rs", "crates/nuxie-runtime/src/state_machine/instance.rs", "crates/nuxie-runtime/src/state_machine/transition_conditions.rs"]
verdict: ADAPTED
axes:
  retained_identity: {status: adapted, idiom_rule: "AF-7 own-by-value", evidence: ["src/animation/transition_property_comparator.cpp:1-0", "crates/nuxie-runtime/src/state_machine/transition_conditions.rs:26-360"]}
  push_vs_poll: {status: isomorphic, cpp_pushes: false, evidence: ["src/animation/transition_property_comparator.cpp:1-0", "crates/nuxie-runtime/src/state_machine/transition_conditions.rs:26-360"], note: "no compensating cycle-time observer/poll introduced for this row"}
  update_ordering: {status: isomorphic, phases_cpp: ["import/initialize", "advance or evaluate", "apply"], phases_rust: ["import immutable descriptor", "advance or evaluate", "apply"]}
  ownership: {status: adapted, idiom_rule: "AF-7 own-by-value", evidence: ["src/animation/transition_property_comparator.cpp:1-0", "crates/nuxie-runtime/src/state_machine/transition_conditions.rs:26-360"]}
  compensation:
    status: adapted
    mechanisms: []
    import_time_constants:
      - {name: "runtime enum/index/type descriptor", idiom_rule: "AF-5 import-time devirtualization", evidence: ["crates/nuxie-runtime/src/state_machine/transition_conditions.rs:26-360"]}
idiom_rules_invoked: ["AF-5 import-time devirtualization", "AF-7 own-by-value"]
confidence: high
notes: "Rust consolidates the C++ object/virtual family into immutable enums, indices, and owned vectors. Those values are fixed during import/instance construction and do not pass the mutation-timing gate. Crate-wide family grep and the listed sibling sweep were clean for this row."
~~~

## B6-0091

~~~yaml
row_id: B6-0091
cpp_files: ["src/animation/transition_property_viewmodel_comparator.cpp"]
rust_module: "crates/nuxie-runtime/src/animation.rs"
subsystem_cluster: animation
sibling_files_swept: ["crates/nuxie-runtime/src/animation.rs", "crates/nuxie-runtime/src/artboard.rs", "crates/nuxie-runtime/src/artboard_data_bind.rs", "crates/nuxie-runtime/src/data_bind_graph.rs", "crates/nuxie-runtime/src/focus.rs", "crates/nuxie-runtime/src/scripting.rs", "crates/nuxie-runtime/src/state_machine.rs", "crates/nuxie-runtime/src/state_machine/instance.rs", "crates/nuxie-runtime/src/state_machine/transition_conditions.rs"]
verdict: DIVERGENT
axes:
  retained_identity: {status: divergent, evidence: ["src/animation/transition_property_viewmodel_comparator.cpp:1-0", "crates/nuxie-runtime/src/state_machine/transition_conditions.rs:26-360", "crates/nuxie-runtime/src/state_machine/instance.rs:79-80"]}
  push_vs_poll: {status: divergent, cpp_pushes: true, evidence: ["src/animation/transition_property_viewmodel_comparator.cpp:1-0", "crates/nuxie-runtime/src/state_machine/instance.rs:5174-5220", "crates/nuxie-runtime/src/state_machine/instance.rs:5390-5403"]}
  update_ordering: {status: divergent, phases_cpp: ["retain live view-model/bindable relationship", "source mutation pushes dirt", "evaluate/action"], phases_rust: ["read candidate generations", "rebind copied graph state", "evaluate/action"]}
  ownership: {status: divergent, evidence: ["src/animation/transition_property_viewmodel_comparator.cpp:1-0", "crates/nuxie-runtime/src/state_machine/instance.rs:79-80", "crates/nuxie-runtime/src/state_machine/instance.rs:5174-5220"]}
  compensation:
    status: divergent
    mechanisms:
      - {name: "owned_view_model_candidate_generation_rebind", kind: "AF-2 copied-state refresh / AF-4 push-to-poll regression", mutation_gated: true, cpp_counterpart: none, evidence: ["crates/nuxie-runtime/src/state_machine/instance.rs:5174-5220", "crates/nuxie-runtime/src/state_machine/instance.rs:5390-5403"]}
    import_time_constants:
      - {name: "view-model path/type descriptor", idiom_rule: "AF-5 import-time devirtualization", evidence: ["crates/nuxie-runtime/src/state_machine/transition_conditions.rs:26-360"]}
idiom_rules_invoked: ["AF-5 import-time devirtualization"]
confidence: high
notes: "Current mid-RB-1 state recorded mechanically; candidate/generation work is scheduled by the #RB-1 mini-queue at docs/parity-closeout-status.md:210-225. No remediation judgment is made."
~~~

## B6-0092

~~~yaml
row_id: B6-0092
cpp_files: ["src/animation/transition_trigger_condition.cpp"]
rust_module: "crates/nuxie-runtime/src/animation.rs"
subsystem_cluster: animation
sibling_files_swept: ["crates/nuxie-runtime/src/animation.rs", "crates/nuxie-runtime/src/artboard.rs", "crates/nuxie-runtime/src/artboard_data_bind.rs", "crates/nuxie-runtime/src/data_bind_graph.rs", "crates/nuxie-runtime/src/focus.rs", "crates/nuxie-runtime/src/scripting.rs", "crates/nuxie-runtime/src/state_machine.rs", "crates/nuxie-runtime/src/state_machine/instance.rs", "crates/nuxie-runtime/src/state_machine/transition_conditions.rs"]
verdict: ADAPTED
axes:
  retained_identity: {status: adapted, idiom_rule: "AF-7 own-by-value", evidence: ["src/animation/transition_trigger_condition.cpp:1-0", "crates/nuxie-runtime/src/state_machine/transition_conditions.rs:26-360"]}
  push_vs_poll: {status: isomorphic, cpp_pushes: false, evidence: ["src/animation/transition_trigger_condition.cpp:1-0", "crates/nuxie-runtime/src/state_machine/transition_conditions.rs:26-360"], note: "no compensating cycle-time observer/poll introduced for this row"}
  update_ordering: {status: isomorphic, phases_cpp: ["import/initialize", "advance or evaluate", "apply"], phases_rust: ["import immutable descriptor", "advance or evaluate", "apply"]}
  ownership: {status: adapted, idiom_rule: "AF-7 own-by-value", evidence: ["src/animation/transition_trigger_condition.cpp:1-0", "crates/nuxie-runtime/src/state_machine/transition_conditions.rs:26-360"]}
  compensation:
    status: adapted
    mechanisms: []
    import_time_constants:
      - {name: "runtime enum/index/type descriptor", idiom_rule: "AF-5 import-time devirtualization", evidence: ["crates/nuxie-runtime/src/state_machine/transition_conditions.rs:26-360"]}
idiom_rules_invoked: ["AF-5 import-time devirtualization", "AF-7 own-by-value"]
confidence: high
notes: "Rust consolidates the C++ object/virtual family into immutable enums, indices, and owned vectors. Those values are fixed during import/instance construction and do not pass the mutation-timing gate. Crate-wide family grep and the listed sibling sweep were clean for this row."
~~~

## B6-0093

~~~yaml
row_id: B6-0093
cpp_files: ["src/animation/transition_viewmodel_condition.cpp"]
rust_module: "crates/nuxie-runtime/src/animation.rs"
subsystem_cluster: animation
sibling_files_swept: ["crates/nuxie-runtime/src/animation.rs", "crates/nuxie-runtime/src/artboard.rs", "crates/nuxie-runtime/src/artboard_data_bind.rs", "crates/nuxie-runtime/src/data_bind_graph.rs", "crates/nuxie-runtime/src/focus.rs", "crates/nuxie-runtime/src/scripting.rs", "crates/nuxie-runtime/src/state_machine.rs", "crates/nuxie-runtime/src/state_machine/instance.rs", "crates/nuxie-runtime/src/state_machine/transition_conditions.rs"]
verdict: DIVERGENT
axes:
  retained_identity: {status: divergent, evidence: ["src/animation/transition_viewmodel_condition.cpp:1-0", "crates/nuxie-runtime/src/state_machine/transition_conditions.rs:26-360", "crates/nuxie-runtime/src/state_machine/instance.rs:79-80"]}
  push_vs_poll: {status: divergent, cpp_pushes: true, evidence: ["src/animation/transition_viewmodel_condition.cpp:1-0", "crates/nuxie-runtime/src/state_machine/instance.rs:5174-5220", "crates/nuxie-runtime/src/state_machine/instance.rs:5390-5403"]}
  update_ordering: {status: divergent, phases_cpp: ["retain live view-model/bindable relationship", "source mutation pushes dirt", "evaluate/action"], phases_rust: ["read candidate generations", "rebind copied graph state", "evaluate/action"]}
  ownership: {status: divergent, evidence: ["src/animation/transition_viewmodel_condition.cpp:1-0", "crates/nuxie-runtime/src/state_machine/instance.rs:79-80", "crates/nuxie-runtime/src/state_machine/instance.rs:5174-5220"]}
  compensation:
    status: divergent
    mechanisms:
      - {name: "owned_view_model_candidate_generation_rebind", kind: "AF-2 copied-state refresh / AF-4 push-to-poll regression", mutation_gated: true, cpp_counterpart: none, evidence: ["crates/nuxie-runtime/src/state_machine/instance.rs:5174-5220", "crates/nuxie-runtime/src/state_machine/instance.rs:5390-5403"]}
    import_time_constants:
      - {name: "view-model path/type descriptor", idiom_rule: "AF-5 import-time devirtualization", evidence: ["crates/nuxie-runtime/src/state_machine/transition_conditions.rs:26-360"]}
idiom_rules_invoked: ["AF-5 import-time devirtualization"]
confidence: high
notes: "Current mid-RB-1 state recorded mechanically; candidate/generation work is scheduled by the #RB-1 mini-queue at docs/parity-closeout-status.md:210-225. No remediation judgment is made."
~~~

## B6-0246

~~~yaml
row_id: B6-0246
cpp_files: ["src/joystick.cpp"]
rust_module: "crates/nuxie-runtime/src/animation.rs"
subsystem_cluster: animation
sibling_files_swept: ["crates/nuxie-runtime/src/animation.rs", "crates/nuxie-runtime/src/artboard.rs", "crates/nuxie-runtime/src/artboard_data_bind.rs", "crates/nuxie-runtime/src/data_bind_graph.rs", "crates/nuxie-runtime/src/focus.rs", "crates/nuxie-runtime/src/scripting.rs", "crates/nuxie-runtime/src/state_machine.rs", "crates/nuxie-runtime/src/state_machine/instance.rs", "crates/nuxie-runtime/src/state_machine/transition_conditions.rs"]
verdict: UNKNOWN
axes:
  retained_identity: {status: unknown, evidence: ["src/joystick.cpp:1-0", "crates/nuxie-runtime/src/animation.rs:43-79; crates/nuxie-runtime/src/artboard.rs:4470-4505"]}
  push_vs_poll: {status: unknown, cpp_pushes: true, evidence: ["src/joystick.cpp:1-0", "crates/nuxie-runtime/src/animation.rs:43-79; crates/nuxie-runtime/src/artboard.rs:4470-4505"]}
  update_ordering: {status: unknown, phases_cpp: ["read complete C++ row"], phases_rust: ["mapped/current subset only"]}
  ownership: {status: unknown, evidence: ["src/joystick.cpp:1-0", "crates/nuxie-runtime/src/animation.rs:43-79; crates/nuxie-runtime/src/artboard.rs:4470-4505"]}
  compensation:
    {status: clear, mechanisms: [], import_time_constants: []}
idiom_rules_invoked: []
confidence: high
notes: "UNKNOWN blocker: Manifest status is partial: Rust retains animation indices and applies them, but no counterpart to C++ Joystick::handleSource/addDependent/removeDependent push ownership was found. Family grep and sibling sweep found no mutation-gated mechanism that could justify DIVERGENT; no inference was made."
~~~
