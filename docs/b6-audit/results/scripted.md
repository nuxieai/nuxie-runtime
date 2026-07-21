# B-6 Structural Audit — scripted

Pinned C++: d788e8ec. Coverage sweep included all of crates/nuxie-runtime plus the scripting seam siblings in artboard.rs, data_bind_graph.rs, draw.rs, and state_machine/instance.rs. RB-1 generation/candidate/observed hits were unrelated. The active/update ID sets match C++ m_isAdvanceActive and ComponentDirt; they are not compensation. The deferred elapsed-step queue below has no C++ lifecycle counterpart and passes the mutation-timing gate.

## B6-0321

~~~yaml
row_id: B6-0321
cpp_files: ["src/scripted/scripted_data_converter.cpp"]
rust_module: "crates/nuxie-runtime/src/scripting.rs"
subsystem_cluster: scripted
sibling_files_swept:
  - "src/scripted/scripted_drawable.cpp"
  - "src/scripted/scripted_interpolator.cpp"
  - "src/scripted/scripted_layout.cpp"
  - "src/scripted/scripted_object.cpp"
  - "src/scripted/scripted_path_effect.cpp"
  - "include/rive/scripted/scripted_data_converter.hpp"
  - "include/rive/scripted/scripted_drawable.hpp"
  - "include/rive/scripted/scripted_interpolator.hpp"
  - "include/rive/scripted/scripted_layout.hpp"
  - "include/rive/scripted/scripted_object.hpp"
  - "include/rive/scripted/scripted_path_effect.hpp"
  - "crates/nuxie-runtime/src/scripting.rs"
  - "crates/nuxie-runtime/src/artboard.rs"
  - "crates/nuxie-runtime/src/data_bind_graph.rs"
  - "crates/nuxie-runtime/src/draw.rs"
  - "crates/nuxie-runtime/src/state_machine/instance.rs"
  - "crates/nuxie/src/lib.rs"
verdict: UNKNOWN
axes:
  retained_identity: {status: unknown, evidence: []}
  push_vs_poll: {status: unknown, cpp_pushes: false, evidence: []}
  update_ordering: {status: unknown, phases_cpp: ["read pinned C++ row"], phases_rust: ["blocker: complete mapped lifecycle absent"]}
  ownership: {status: unknown, evidence: []}
  compensation: {status: unknown, mechanisms: [], import_time_constants: []}
idiom_rules_invoked: []
confidence: low
notes: "BLOCKER: The mapped Rust conversion path exists (data_bind_graph.rs:1767-1791), but no Rust counterpart for C++ ScriptedDataConverter::advance/markConverterDirty (cpp:190-213) was found; the row-wide update lifecycle cannot be honestly classified."
~~~

## B6-0322

~~~yaml
row_id: B6-0322
cpp_files: ["src/scripted/scripted_drawable.cpp"]
rust_module: "crates/nuxie-runtime/src/scripting.rs"
subsystem_cluster: scripted
sibling_files_swept:
  - "src/scripted/scripted_data_converter.cpp"
  - "src/scripted/scripted_interpolator.cpp"
  - "src/scripted/scripted_layout.cpp"
  - "src/scripted/scripted_object.cpp"
  - "src/scripted/scripted_path_effect.cpp"
  - "include/rive/scripted/scripted_data_converter.hpp"
  - "include/rive/scripted/scripted_drawable.hpp"
  - "include/rive/scripted/scripted_interpolator.hpp"
  - "include/rive/scripted/scripted_layout.hpp"
  - "include/rive/scripted/scripted_object.hpp"
  - "include/rive/scripted/scripted_path_effect.hpp"
  - "crates/nuxie-runtime/src/scripting.rs"
  - "crates/nuxie-runtime/src/artboard.rs"
  - "crates/nuxie-runtime/src/data_bind_graph.rs"
  - "crates/nuxie-runtime/src/draw.rs"
  - "crates/nuxie-runtime/src/state_machine/instance.rs"
  - "crates/nuxie/src/lib.rs"
verdict: DIVERGENT
axes:
  retained_identity: {status: adapted, idiom_rule: "rc-refcell-for-rcp", evidence: ["src/scripted/scripted_drawable.cpp:19-73", "crates/nuxie-runtime/src/scripting.rs:1688-1708", "crates/nuxie-runtime/src/draw.rs:13360-13404"]}
  push_vs_poll: {status: isomorphic, cpp_pushes: false, evidence: ["src/scripted/scripted_drawable.cpp:347-360", "crates/nuxie-runtime/src/artboard.rs:1145-1170"]}
  update_ordering: {status: divergent, phases_cpp: ["component advance calls script immediately", "script result rearms object", "dirt schedules update/draw"], phases_rust: ["root/nested advance enqueues elapsed", "factory-bearing facade later replays queue", "active set rearms id", "pending set runs update", "draw"]}
  ownership: {status: adapted, idiom_rule: "rc-refcell-for-rcp", evidence: ["include/rive/scripted/scripted_object.hpp:30-52", "crates/nuxie-runtime/src/scripting.rs:1688-1708"]}
  compensation:
    status: divergent
    mechanisms:
      - {name: "deferred_script_advance_queue", kind: "AF-8 invented lifecycle", mutation_gated: true, cpp_counterpart: none, evidence: ["crates/nuxie/src/lib.rs:1562", "crates/nuxie-runtime/src/artboard.rs:1333-1360"]}
    import_time_constants:
      - {name: "ArtboardInstance.has_scripted_drawables", idiom_rule: "AF-5 import-time devirtualization", evidence: ["crates/nuxie-runtime/src/artboard.rs:246-255; crates/nuxie-runtime/src/artboard.rs:1121-1136"]}
idiom_rules_invoked: ["rc-refcell-for-rcp"]
confidence: high
notes: "\"C++ advances the retained scripted drawable directly in advanceComponent. Rust retains the VM instance correctly, but stores elapsed steps during the advance cycle and replays them only when a renderer factory is available.\""
~~~

## B6-0323

~~~yaml
row_id: B6-0323
cpp_files: ["src/scripted/scripted_interpolator.cpp"]
rust_module: "crates/nuxie-runtime/src/scripting.rs"
subsystem_cluster: scripted
sibling_files_swept:
  - "src/scripted/scripted_data_converter.cpp"
  - "src/scripted/scripted_drawable.cpp"
  - "src/scripted/scripted_layout.cpp"
  - "src/scripted/scripted_object.cpp"
  - "src/scripted/scripted_path_effect.cpp"
  - "include/rive/scripted/scripted_data_converter.hpp"
  - "include/rive/scripted/scripted_drawable.hpp"
  - "include/rive/scripted/scripted_interpolator.hpp"
  - "include/rive/scripted/scripted_layout.hpp"
  - "include/rive/scripted/scripted_object.hpp"
  - "include/rive/scripted/scripted_path_effect.hpp"
  - "crates/nuxie-runtime/src/scripting.rs"
  - "crates/nuxie-runtime/src/artboard.rs"
  - "crates/nuxie-runtime/src/data_bind_graph.rs"
  - "crates/nuxie-runtime/src/draw.rs"
  - "crates/nuxie-runtime/src/state_machine/instance.rs"
  - "crates/nuxie/src/lib.rs"
verdict: UNKNOWN
axes:
  retained_identity: {status: unknown, evidence: []}
  push_vs_poll: {status: unknown, cpp_pushes: false, evidence: []}
  update_ordering: {status: unknown, phases_cpp: ["read pinned C++ row"], phases_rust: ["blocker: complete mapped lifecycle absent"]}
  ownership: {status: unknown, evidence: []}
  compensation: {status: unknown, mechanisms: [], import_time_constants: []}
idiom_rules_invoked: []
confidence: low
notes: "BLOCKER: No Rust ScriptedInterpolator transform/transformValue or per-keyframe scripted-object clone implementation was found in the mapped module or crate-wide sibling sweep."
~~~

## B6-0324

~~~yaml
row_id: B6-0324
cpp_files: ["src/scripted/scripted_layout.cpp"]
rust_module: "crates/nuxie-runtime/src/scripting.rs"
subsystem_cluster: scripted
sibling_files_swept:
  - "src/scripted/scripted_data_converter.cpp"
  - "src/scripted/scripted_drawable.cpp"
  - "src/scripted/scripted_interpolator.cpp"
  - "src/scripted/scripted_object.cpp"
  - "src/scripted/scripted_path_effect.cpp"
  - "include/rive/scripted/scripted_data_converter.hpp"
  - "include/rive/scripted/scripted_drawable.hpp"
  - "include/rive/scripted/scripted_interpolator.hpp"
  - "include/rive/scripted/scripted_layout.hpp"
  - "include/rive/scripted/scripted_object.hpp"
  - "include/rive/scripted/scripted_path_effect.hpp"
  - "crates/nuxie-runtime/src/scripting.rs"
  - "crates/nuxie-runtime/src/artboard.rs"
  - "crates/nuxie-runtime/src/data_bind_graph.rs"
  - "crates/nuxie-runtime/src/draw.rs"
  - "crates/nuxie-runtime/src/state_machine/instance.rs"
  - "crates/nuxie/src/lib.rs"
verdict: UNKNOWN
axes:
  retained_identity: {status: unknown, evidence: []}
  push_vs_poll: {status: unknown, cpp_pushes: false, evidence: []}
  update_ordering: {status: unknown, phases_cpp: ["read pinned C++ row"], phases_rust: ["blocker: complete mapped lifecycle absent"]}
  ownership: {status: unknown, evidence: []}
  compensation: {status: unknown, mechanisms: [], import_time_constants: []}
idiom_rules_invoked: []
confidence: low
notes: "BLOCKER: Rust recognizes ScriptedLayout as a drawable, but no mapped measureLayout/controlSize/resize script lifecycle was found; the C++ row cannot be honestly audited end to end."
~~~

## B6-0325

~~~yaml
row_id: B6-0325
cpp_files: ["src/scripted/scripted_object.cpp"]
rust_module: "crates/nuxie-runtime/src/scripting.rs"
subsystem_cluster: scripted
sibling_files_swept:
  - "src/scripted/scripted_data_converter.cpp"
  - "src/scripted/scripted_drawable.cpp"
  - "src/scripted/scripted_interpolator.cpp"
  - "src/scripted/scripted_layout.cpp"
  - "src/scripted/scripted_path_effect.cpp"
  - "include/rive/scripted/scripted_data_converter.hpp"
  - "include/rive/scripted/scripted_drawable.hpp"
  - "include/rive/scripted/scripted_interpolator.hpp"
  - "include/rive/scripted/scripted_layout.hpp"
  - "include/rive/scripted/scripted_object.hpp"
  - "include/rive/scripted/scripted_path_effect.hpp"
  - "crates/nuxie-runtime/src/scripting.rs"
  - "crates/nuxie-runtime/src/artboard.rs"
  - "crates/nuxie-runtime/src/data_bind_graph.rs"
  - "crates/nuxie-runtime/src/draw.rs"
  - "crates/nuxie-runtime/src/state_machine/instance.rs"
  - "crates/nuxie/src/lib.rs"
verdict: DIVERGENT
axes:
  retained_identity: {status: adapted, idiom_rule: "rc-refcell-for-rcp", evidence: ["include/rive/scripted/scripted_object.hpp:30-52", "crates/nuxie-runtime/src/scripting.rs:1508-1670", "crates/nuxie-runtime/src/scripting.rs:1688-1708"]}
  push_vs_poll: {status: isomorphic, cpp_pushes: false, evidence: ["src/scripted/scripted_object.cpp:43-175", "crates/nuxie-runtime/src/artboard.rs:1452-1518"]}
  update_ordering: {status: divergent, phases_cpp: ["hydrate/init retained object", "advance callback directly", "dirt update", "draw/evaluate"], phases_rust: ["hydrate retained trait object", "queue elapsed step", "later flush advance with factory", "pending update", "draw/evaluate"]}
  ownership: {status: adapted, idiom_rule: "rc-refcell-for-rcp", evidence: ["src/scripted/scripted_object.cpp:313-397", "crates/nuxie-runtime/src/scripting.rs:1688-1708"]}
  compensation:
    status: divergent
    mechanisms:
      - {name: "deferred_script_advance_queue", kind: "AF-8 invented lifecycle", mutation_gated: true, cpp_counterpart: none, evidence: ["crates/nuxie/src/lib.rs:1562", "crates/nuxie-runtime/src/artboard.rs:1333-1360"]}
    import_time_constants:
      - {name: "ArtboardInstance.has_scripted_drawables", idiom_rule: "AF-5 import-time devirtualization", evidence: ["crates/nuxie-runtime/src/artboard.rs:246-255; crates/nuxie-runtime/src/artboard.rs:1121-1136"]}
idiom_rules_invoked: ["rc-refcell-for-rcp"]
confidence: high
notes: "\"The retained VM table maps cleanly to Rc<RefCell<Box<dyn ScriptInstance>>>. The architectural divergence is the extra deferred-advance lifecycle shared by scripted object subclasses.\""
~~~

## B6-0326

~~~yaml
row_id: B6-0326
cpp_files: ["src/scripted/scripted_path_effect.cpp"]
rust_module: "crates/nuxie-runtime/src/scripting.rs"
subsystem_cluster: scripted
sibling_files_swept:
  - "src/scripted/scripted_data_converter.cpp"
  - "src/scripted/scripted_drawable.cpp"
  - "src/scripted/scripted_interpolator.cpp"
  - "src/scripted/scripted_layout.cpp"
  - "src/scripted/scripted_object.cpp"
  - "include/rive/scripted/scripted_data_converter.hpp"
  - "include/rive/scripted/scripted_drawable.hpp"
  - "include/rive/scripted/scripted_interpolator.hpp"
  - "include/rive/scripted/scripted_layout.hpp"
  - "include/rive/scripted/scripted_object.hpp"
  - "include/rive/scripted/scripted_path_effect.hpp"
  - "crates/nuxie-runtime/src/scripting.rs"
  - "crates/nuxie-runtime/src/artboard.rs"
  - "crates/nuxie-runtime/src/data_bind_graph.rs"
  - "crates/nuxie-runtime/src/draw.rs"
  - "crates/nuxie-runtime/src/state_machine/instance.rs"
  - "crates/nuxie/src/lib.rs"
verdict: DIVERGENT
axes:
  retained_identity: {status: adapted, idiom_rule: "rc-refcell-for-rcp", evidence: ["src/scripted/scripted_path_effect.cpp:21-79", "crates/nuxie-runtime/src/scripting.rs:1672-1685", "crates/nuxie-runtime/src/scripting.rs:1688-1708"]}
  push_vs_poll: {status: divergent, cpp_pushes: true, evidence: ["src/scripted/scripted_path_effect.cpp:190-207", "crates/nuxie-runtime/src/draw.rs:19104-19131"]}
  update_ordering: {status: divergent, phases_cpp: ["dependent dirt invalidates cached EffectPath", "component advance calls script", "updateEffect lazily rebuilds invalid path"], phases_rust: ["advance queues elapsed", "factory flush replays script advance", "draw path calls script effect directly"]}
  ownership: {status: adapted, idiom_rule: "rc-refcell-for-rcp", evidence: ["include/rive/scripted/scripted_path_effect.hpp:12-19", "crates/nuxie-runtime/src/scripting.rs:1688-1708"]}
  compensation:
    status: divergent
    mechanisms:
      - {name: "deferred_script_advance_queue", kind: "AF-8 invented lifecycle", mutation_gated: true, cpp_counterpart: none, evidence: ["crates/nuxie/src/lib.rs:1562", "crates/nuxie-runtime/src/artboard.rs:1333-1360"]}
    import_time_constants:
      - {name: "ArtboardInstance.has_scripted_drawables", idiom_rule: "AF-5 import-time devirtualization", evidence: ["crates/nuxie-runtime/src/artboard.rs:246-255; crates/nuxie-runtime/src/artboard.rs:1121-1136"]}
      - {name: "ArtboardInstance.script_path_effect_globals", idiom_rule: "AF-5 import-time devirtualization", evidence: ["crates/nuxie-runtime/src/artboard.rs:246-255; crates/nuxie-runtime/src/artboard.rs:1177-1184"]}
idiom_rules_invoked: ["rc-refcell-for-rcp"]
confidence: high
notes: "\"C++ registers a dependent and invalidates a retained effect path. Rust calls the script from the draw-path sibling and also defers advance through the elapsed-step queue. The queue is the mutation-gated mechanism; the draw-time recomputation is structural context, not counted separately because it stores no drift tracker.\""
~~~

