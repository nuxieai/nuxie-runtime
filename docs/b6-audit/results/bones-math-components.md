# B-6 Structural Audit — bones-math-components

Pinned C++: d788e8ec6e8b598526607d6a1e8818e8b637b60c. All 21 assigned C++ files and the mapped `crates/nuxie-runtime/src/components.rs` regions were read. The coverage-clause sweep followed component dirt, skeletal graph descriptors, bone/skin dependency edges, weighted-path reconstruction, render-path storage, geometry utilities, and every cache/epoch family through the listed sibling files. The `TransformPropertyKeys`/`OnceLock` family and immutable skeletal graph descriptors are build/static data and fail the mutation-timing gate; they are recorded as AF-5 import-time devirtualization. The pinned upstream checkout was read-only.

## B6-0115

~~~yaml
row_id: B6-0115
cpp_files: ["src/bones/bone.cpp"]
rust_module: "crates/nuxie-runtime/src/components.rs"
subsystem_cluster: bones-math-components
sibling_files_swept: ["crates/nuxie-runtime/src/components.rs", "crates/nuxie-runtime/src/artboard.rs", "crates/nuxie-runtime/src/draw.rs", "crates/nuxie-runtime/src/constraints.rs", "crates/nuxie-runtime/src/objects.rs", "crates/nuxie-runtime/src/text.rs", "crates/nuxie-runtime/src/data_bind_graph.rs", "crates/nuxie-graph/src/lib.rs", "crates/nuxie-render-api/src/lib.rs"]
verdict: DIVERGENT
axes:
  retained_identity: {status: adapted, idiom_rule: "AF-1 Retained identity via stable arena/local id", evidence: ["src/bones/bone.cpp:7-43", "crates/nuxie-graph/src/lib.rs:842-850", "crates/nuxie-graph/src/lib.rs:2895-2943", "crates/nuxie-runtime/src/artboard.rs:3635-3679"]}
  push_vs_poll: {status: isomorphic, cpp_pushes: true, evidence: ["src/bones/bone.cpp:7-43", "crates/nuxie-graph/src/lib.rs:3768-3781", "crates/nuxie-runtime/src/artboard.rs:3948-3998"], note: "Stable dependency edges push dirt; no relationship rescan or observed-value diff was found."}
  update_ordering: {status: isomorphic, phases_cpp: ["property mutation", "dirt collection/cascade", "graph-ordered component update", "dependent update"], phases_rust: ["property mutation", "dirt collection/cascade", "graph-ordered component update", "dependent update"]}
  ownership: {status: adapted, evidence: ["src/bones/bone.cpp:7-43", "crates/nuxie-graph/src/lib.rs:842-850", "crates/nuxie-graph/src/lib.rs:2895-2943", "crates/nuxie-runtime/src/artboard.rs:3635-3679"]}
  compensation:
    status: divergent
    mechanisms:
      - {name: "cache_epoch", kind: "cross-frame cache invalidation generation", mutation_gated: true, cpp_counterpart: none, evidence: ["crates/nuxie-runtime/src/artboard.rs:3728-3731", "crates/nuxie-runtime/src/artboard.rs:4424-4434", "crates/nuxie-runtime/src/draw.rs:2221-2286"]}
      - {name: "prepared_epoch", kind: "prepared-frame invalidation generation", mutation_gated: true, cpp_counterpart: none, evidence: ["crates/nuxie-runtime/src/artboard.rs:3747-3755", "crates/nuxie-runtime/src/artboard.rs:3948-3973", "crates/nuxie-runtime/src/draw.rs:2239-2249"]}
      - {name: "tree paint preparation epoch", kind: "shared tree-level invalidation generation", mutation_gated: true, cpp_counterpart: none, evidence: ["crates/nuxie-runtime/src/artboard.rs:1621-1632", "crates/nuxie-runtime/src/artboard.rs:3747-3755", "crates/nuxie-runtime/src/draw.rs:2239-2250"]}
    import_time_constants:
      - {name: "TransformPropertyKeys and OnceLock property-key cache", idiom_rule: "AF-5 Import-time devirtualization is legitimate", evidence: ["crates/nuxie-runtime/src/components.rs:160-243", "crates/nuxie-runtime/src/components.rs:477-513"], note: "computed at build/static lookup time and not written during advance/update/bind"}
      - {name: "skeletal graph descriptors and dependency indexes", idiom_rule: "AF-5 Import-time devirtualization is legitimate", evidence: ["crates/nuxie-graph/src/lib.rs:842-871", "crates/nuxie-graph/src/lib.rs:2895-3016", "crates/nuxie-graph/src/lib.rs:5026-5079"], note: "built once from the imported graph; not a cycle-time drift tracker"}
idiom_rules_invoked: ["AF-1 Retained identity via stable arena/local id", "AF-5 Import-time devirtualization is legitimate"]
confidence: high
notes: "C++ retains child-bone and peer-constraint pointers and pushes transform dirt; Rust retains stable local ids and the same dirt cascade, but also mutates cross-frame renderer epochs when the update-cycle dirt arrives."
~~~

## B6-0116

~~~yaml
row_id: B6-0116
cpp_files: ["src/bones/root_bone.cpp"]
rust_module: "crates/nuxie-runtime/src/components.rs"
subsystem_cluster: bones-math-components
sibling_files_swept: ["crates/nuxie-runtime/src/components.rs", "crates/nuxie-runtime/src/artboard.rs", "crates/nuxie-runtime/src/draw.rs", "crates/nuxie-runtime/src/constraints.rs", "crates/nuxie-runtime/src/objects.rs", "crates/nuxie-runtime/src/text.rs", "crates/nuxie-runtime/src/data_bind_graph.rs", "crates/nuxie-graph/src/lib.rs", "crates/nuxie-render-api/src/lib.rs"]
verdict: DIVERGENT
axes:
  retained_identity: {status: adapted, idiom_rule: "AF-1 Retained identity via stable arena/local id", evidence: ["src/bones/root_bone.cpp:5-15", "crates/nuxie-runtime/src/artboard.rs:3580-3610", "crates/nuxie-runtime/src/artboard.rs:3635-3679"]}
  push_vs_poll: {status: isomorphic, cpp_pushes: true, evidence: ["src/bones/root_bone.cpp:5-15", "crates/nuxie-graph/src/lib.rs:3768-3781", "crates/nuxie-runtime/src/artboard.rs:3948-3998"], note: "Stable dependency edges push dirt; no relationship rescan or observed-value diff was found."}
  update_ordering: {status: isomorphic, phases_cpp: ["property mutation", "dirt collection/cascade", "graph-ordered component update", "dependent update"], phases_rust: ["property mutation", "dirt collection/cascade", "graph-ordered component update", "dependent update"]}
  ownership: {status: adapted, evidence: ["src/bones/root_bone.cpp:5-15", "crates/nuxie-runtime/src/artboard.rs:3580-3610", "crates/nuxie-runtime/src/artboard.rs:3635-3679"]}
  compensation:
    status: divergent
    mechanisms:
      - {name: "cache_epoch", kind: "cross-frame cache invalidation generation", mutation_gated: true, cpp_counterpart: none, evidence: ["crates/nuxie-runtime/src/artboard.rs:3728-3731", "crates/nuxie-runtime/src/artboard.rs:4424-4434", "crates/nuxie-runtime/src/draw.rs:2221-2286"]}
      - {name: "prepared_epoch", kind: "prepared-frame invalidation generation", mutation_gated: true, cpp_counterpart: none, evidence: ["crates/nuxie-runtime/src/artboard.rs:3747-3755", "crates/nuxie-runtime/src/artboard.rs:3948-3973", "crates/nuxie-runtime/src/draw.rs:2239-2249"]}
      - {name: "tree paint preparation epoch", kind: "shared tree-level invalidation generation", mutation_gated: true, cpp_counterpart: none, evidence: ["crates/nuxie-runtime/src/artboard.rs:1621-1632", "crates/nuxie-runtime/src/artboard.rs:3747-3755", "crates/nuxie-runtime/src/draw.rs:2239-2250"]}
    import_time_constants:
      - {name: "TransformPropertyKeys and OnceLock property-key cache", idiom_rule: "AF-5 Import-time devirtualization is legitimate", evidence: ["crates/nuxie-runtime/src/components.rs:160-243", "crates/nuxie-runtime/src/components.rs:477-513"], note: "computed at build/static lookup time and not written during advance/update/bind"}
      - {name: "skeletal graph descriptors and dependency indexes", idiom_rule: "AF-5 Import-time devirtualization is legitimate", evidence: ["crates/nuxie-graph/src/lib.rs:842-871", "crates/nuxie-graph/src/lib.rs:2895-3016", "crates/nuxie-graph/src/lib.rs:5026-5079"], note: "built once from the imported graph; not a cycle-time drift tracker"}
idiom_rules_invoked: ["AF-1 Retained identity via stable arena/local id", "AF-5 Import-time devirtualization is legitimate"]
confidence: high
notes: "The special root authored-position path and transform dirt are retained; the Rust dirt/update bridge additionally mutates renderer cache epochs."
~~~

## B6-0117

~~~yaml
row_id: B6-0117
cpp_files: ["src/bones/skin.cpp"]
rust_module: "crates/nuxie-runtime/src/components.rs"
subsystem_cluster: bones-math-components
sibling_files_swept: ["crates/nuxie-runtime/src/components.rs", "crates/nuxie-runtime/src/artboard.rs", "crates/nuxie-runtime/src/draw.rs", "crates/nuxie-runtime/src/constraints.rs", "crates/nuxie-runtime/src/objects.rs", "crates/nuxie-runtime/src/text.rs", "crates/nuxie-runtime/src/data_bind_graph.rs", "crates/nuxie-graph/src/lib.rs", "crates/nuxie-render-api/src/lib.rs"]
verdict: DIVERGENT
axes:
  retained_identity: {status: adapted, idiom_rule: "AF-1 Retained identity via stable arena/local id", evidence: ["src/bones/skin.cpp:11-93", "crates/nuxie-graph/src/lib.rs:853-861", "crates/nuxie-graph/src/lib.rs:2945-3016", "crates/nuxie-runtime/src/artboard.rs:3948-3998", "crates/nuxie-runtime/src/draw.rs:6420-6465"]}
  push_vs_poll: {status: isomorphic, cpp_pushes: true, evidence: ["src/bones/skin.cpp:11-93", "crates/nuxie-graph/src/lib.rs:3768-3781", "crates/nuxie-runtime/src/artboard.rs:3948-3998"], note: "Stable dependency edges push dirt; no relationship rescan or observed-value diff was found."}
  update_ordering: {status: isomorphic, phases_cpp: ["property mutation", "dirt collection/cascade", "graph-ordered component update", "dependent update"], phases_rust: ["property mutation", "dirt collection/cascade", "graph-ordered component update", "dependent update"]}
  ownership: {status: adapted, evidence: ["src/bones/skin.cpp:11-93", "crates/nuxie-graph/src/lib.rs:853-861", "crates/nuxie-graph/src/lib.rs:2945-3016", "crates/nuxie-runtime/src/artboard.rs:3948-3998", "crates/nuxie-runtime/src/draw.rs:6420-6465"]}
  compensation:
    status: divergent
    mechanisms:
      - {name: "cache_epoch", kind: "cross-frame cache invalidation generation", mutation_gated: true, cpp_counterpart: none, evidence: ["crates/nuxie-runtime/src/artboard.rs:3728-3731", "crates/nuxie-runtime/src/artboard.rs:4424-4434", "crates/nuxie-runtime/src/draw.rs:2221-2286"]}
      - {name: "path_epoch", kind: "path-geometry invalidation generation", mutation_gated: true, cpp_counterpart: none, evidence: ["crates/nuxie-runtime/src/artboard.rs:3788-3790", "crates/nuxie-runtime/src/artboard.rs:3965-3969", "crates/nuxie-runtime/src/draw.rs:10958-10978"]}
      - {name: "prepared_epoch", kind: "prepared-frame invalidation generation", mutation_gated: true, cpp_counterpart: none, evidence: ["crates/nuxie-runtime/src/artboard.rs:3747-3755", "crates/nuxie-runtime/src/artboard.rs:3948-3973", "crates/nuxie-runtime/src/draw.rs:2239-2249"]}
      - {name: "command_epoch", kind: "draw-command invalidation generation", mutation_gated: true, cpp_counterpart: none, evidence: ["crates/nuxie-runtime/src/artboard.rs:3747-3750", "crates/nuxie-runtime/src/draw.rs:10772-10790"]}
      - {name: "tree paint preparation epoch", kind: "shared tree-level invalidation generation", mutation_gated: true, cpp_counterpart: none, evidence: ["crates/nuxie-runtime/src/artboard.rs:1621-1632", "crates/nuxie-runtime/src/artboard.rs:3747-3755", "crates/nuxie-runtime/src/draw.rs:2239-2250"]}
    import_time_constants:
      - {name: "TransformPropertyKeys and OnceLock property-key cache", idiom_rule: "AF-5 Import-time devirtualization is legitimate", evidence: ["crates/nuxie-runtime/src/components.rs:160-243", "crates/nuxie-runtime/src/components.rs:477-513"], note: "computed at build/static lookup time and not written during advance/update/bind"}
      - {name: "skeletal graph descriptors and dependency indexes", idiom_rule: "AF-5 Import-time devirtualization is legitimate", evidence: ["crates/nuxie-graph/src/lib.rs:842-871", "crates/nuxie-graph/src/lib.rs:2895-3016", "crates/nuxie-graph/src/lib.rs:5026-5079"], note: "built once from the imported graph; not a cycle-time drift tracker"}
idiom_rules_invoked: ["AF-1 Retained identity via stable arena/local id", "AF-5 Import-time devirtualization is legitimate"]
confidence: high
notes: "C++ retains a Skin-owned transform buffer and pushes dirt to registered skinnables. Rust retains graph identities and pushes equivalent dirt, but reconstructs the current bone-transform vector at draw time and uses the mutation-gated epoch family to invalidate retained path/command caches."
~~~

## B6-0118

~~~yaml
row_id: B6-0118
cpp_files: ["src/bones/skinnable.cpp"]
rust_module: "crates/nuxie-runtime/src/components.rs"
subsystem_cluster: bones-math-components
sibling_files_swept: ["crates/nuxie-runtime/src/components.rs", "crates/nuxie-runtime/src/artboard.rs", "crates/nuxie-runtime/src/draw.rs", "crates/nuxie-runtime/src/constraints.rs", "crates/nuxie-runtime/src/objects.rs", "crates/nuxie-runtime/src/text.rs", "crates/nuxie-runtime/src/data_bind_graph.rs", "crates/nuxie-graph/src/lib.rs", "crates/nuxie-render-api/src/lib.rs"]
verdict: DIVERGENT
axes:
  retained_identity: {status: adapted, idiom_rule: "AF-1 Retained identity via stable arena/local id", evidence: ["src/bones/skinnable.cpp:7-19", "crates/nuxie-graph/src/lib.rs:853-861", "crates/nuxie-graph/src/lib.rs:2945-2976", "crates/nuxie-graph/src/lib.rs:3768-3781", "crates/nuxie-runtime/src/artboard.rs:3948-3998"]}
  push_vs_poll: {status: isomorphic, cpp_pushes: true, evidence: ["src/bones/skinnable.cpp:7-19", "crates/nuxie-graph/src/lib.rs:3768-3781", "crates/nuxie-runtime/src/artboard.rs:3948-3998"], note: "Stable dependency edges push dirt; no relationship rescan or observed-value diff was found."}
  update_ordering: {status: isomorphic, phases_cpp: ["property mutation", "dirt collection/cascade", "graph-ordered component update", "dependent update"], phases_rust: ["property mutation", "dirt collection/cascade", "graph-ordered component update", "dependent update"]}
  ownership: {status: adapted, evidence: ["src/bones/skinnable.cpp:7-19", "crates/nuxie-graph/src/lib.rs:853-861", "crates/nuxie-graph/src/lib.rs:2945-2976", "crates/nuxie-graph/src/lib.rs:3768-3781", "crates/nuxie-runtime/src/artboard.rs:3948-3998"]}
  compensation:
    status: divergent
    mechanisms:
      - {name: "cache_epoch", kind: "cross-frame cache invalidation generation", mutation_gated: true, cpp_counterpart: none, evidence: ["crates/nuxie-runtime/src/artboard.rs:3728-3731", "crates/nuxie-runtime/src/artboard.rs:4424-4434", "crates/nuxie-runtime/src/draw.rs:2221-2286"]}
      - {name: "path_epoch", kind: "path-geometry invalidation generation", mutation_gated: true, cpp_counterpart: none, evidence: ["crates/nuxie-runtime/src/artboard.rs:3788-3790", "crates/nuxie-runtime/src/artboard.rs:3965-3969", "crates/nuxie-runtime/src/draw.rs:10958-10978"]}
      - {name: "prepared_epoch", kind: "prepared-frame invalidation generation", mutation_gated: true, cpp_counterpart: none, evidence: ["crates/nuxie-runtime/src/artboard.rs:3747-3755", "crates/nuxie-runtime/src/artboard.rs:3948-3973", "crates/nuxie-runtime/src/draw.rs:2239-2249"]}
      - {name: "command_epoch", kind: "draw-command invalidation generation", mutation_gated: true, cpp_counterpart: none, evidence: ["crates/nuxie-runtime/src/artboard.rs:3747-3750", "crates/nuxie-runtime/src/draw.rs:10772-10790"]}
      - {name: "tree paint preparation epoch", kind: "shared tree-level invalidation generation", mutation_gated: true, cpp_counterpart: none, evidence: ["crates/nuxie-runtime/src/artboard.rs:1621-1632", "crates/nuxie-runtime/src/artboard.rs:3747-3755", "crates/nuxie-runtime/src/draw.rs:2239-2250"]}
    import_time_constants:
      - {name: "TransformPropertyKeys and OnceLock property-key cache", idiom_rule: "AF-5 Import-time devirtualization is legitimate", evidence: ["crates/nuxie-runtime/src/components.rs:160-243", "crates/nuxie-runtime/src/components.rs:477-513"], note: "computed at build/static lookup time and not written during advance/update/bind"}
      - {name: "skeletal graph descriptors and dependency indexes", idiom_rule: "AF-5 Import-time devirtualization is legitimate", evidence: ["crates/nuxie-graph/src/lib.rs:842-871", "crates/nuxie-graph/src/lib.rs:2895-3016", "crates/nuxie-graph/src/lib.rs:5026-5079"], note: "built once from the imported graph; not a cycle-time drift tracker"}
idiom_rules_invoked: ["AF-1 Retained identity via stable arena/local id", "AF-5 Import-time devirtualization is legitimate"]
confidence: high
notes: "The C++ Skin pointer is represented by stable graph-local identity and a dependency edge; the update-cycle also mutates renderer path/command epochs for this skinnable relationship."
~~~

## B6-0119

~~~yaml
row_id: B6-0119
cpp_files: ["src/bones/tendon.cpp"]
rust_module: "crates/nuxie-runtime/src/components.rs"
subsystem_cluster: bones-math-components
sibling_files_swept: ["crates/nuxie-runtime/src/components.rs", "crates/nuxie-runtime/src/artboard.rs", "crates/nuxie-runtime/src/draw.rs", "crates/nuxie-runtime/src/constraints.rs", "crates/nuxie-runtime/src/objects.rs", "crates/nuxie-runtime/src/text.rs", "crates/nuxie-runtime/src/data_bind_graph.rs", "crates/nuxie-graph/src/lib.rs", "crates/nuxie-render-api/src/lib.rs"]
verdict: DIVERGENT
axes:
  retained_identity: {status: adapted, idiom_rule: "AF-1 Retained identity via stable arena/local id", evidence: ["src/bones/tendon.cpp:8-50", "crates/nuxie-graph/src/lib.rs:863-871", "crates/nuxie-graph/src/lib.rs:2979-3016", "crates/nuxie-graph/src/lib.rs:4589-4654"]}
  push_vs_poll: {status: isomorphic, cpp_pushes: true, evidence: ["src/bones/tendon.cpp:8-50", "crates/nuxie-graph/src/lib.rs:3768-3781", "crates/nuxie-runtime/src/artboard.rs:3948-3998"], note: "Stable dependency edges push dirt; no relationship rescan or observed-value diff was found."}
  update_ordering: {status: isomorphic, phases_cpp: ["property mutation", "dirt collection/cascade", "graph-ordered component update", "dependent update"], phases_rust: ["property mutation", "dirt collection/cascade", "graph-ordered component update", "dependent update"]}
  ownership: {status: adapted, evidence: ["src/bones/tendon.cpp:8-50", "crates/nuxie-graph/src/lib.rs:863-871", "crates/nuxie-graph/src/lib.rs:2979-3016", "crates/nuxie-graph/src/lib.rs:4589-4654"]}
  compensation:
    status: divergent
    mechanisms:
      - {name: "cache_epoch", kind: "cross-frame cache invalidation generation", mutation_gated: true, cpp_counterpart: none, evidence: ["crates/nuxie-runtime/src/artboard.rs:3728-3731", "crates/nuxie-runtime/src/artboard.rs:4424-4434", "crates/nuxie-runtime/src/draw.rs:2221-2286"]}
      - {name: "path_epoch", kind: "path-geometry invalidation generation", mutation_gated: true, cpp_counterpart: none, evidence: ["crates/nuxie-runtime/src/artboard.rs:3788-3790", "crates/nuxie-runtime/src/artboard.rs:3965-3969", "crates/nuxie-runtime/src/draw.rs:10958-10978"]}
      - {name: "prepared_epoch", kind: "prepared-frame invalidation generation", mutation_gated: true, cpp_counterpart: none, evidence: ["crates/nuxie-runtime/src/artboard.rs:3747-3755", "crates/nuxie-runtime/src/artboard.rs:3948-3973", "crates/nuxie-runtime/src/draw.rs:2239-2249"]}
      - {name: "command_epoch", kind: "draw-command invalidation generation", mutation_gated: true, cpp_counterpart: none, evidence: ["crates/nuxie-runtime/src/artboard.rs:3747-3750", "crates/nuxie-runtime/src/draw.rs:10772-10790"]}
      - {name: "tree paint preparation epoch", kind: "shared tree-level invalidation generation", mutation_gated: true, cpp_counterpart: none, evidence: ["crates/nuxie-runtime/src/artboard.rs:1621-1632", "crates/nuxie-runtime/src/artboard.rs:3747-3755", "crates/nuxie-runtime/src/draw.rs:2239-2250"]}
    import_time_constants:
      - {name: "TransformPropertyKeys and OnceLock property-key cache", idiom_rule: "AF-5 Import-time devirtualization is legitimate", evidence: ["crates/nuxie-runtime/src/components.rs:160-243", "crates/nuxie-runtime/src/components.rs:477-513"], note: "computed at build/static lookup time and not written during advance/update/bind"}
      - {name: "skeletal graph descriptors and dependency indexes", idiom_rule: "AF-5 Import-time devirtualization is legitimate", evidence: ["crates/nuxie-graph/src/lib.rs:842-871", "crates/nuxie-graph/src/lib.rs:2895-3016", "crates/nuxie-graph/src/lib.rs:5026-5079"], note: "built once from the imported graph; not a cycle-time drift tracker"}
idiom_rules_invoked: ["AF-1 Retained identity via stable arena/local id", "AF-5 Import-time devirtualization is legitimate"]
confidence: high
notes: "Bone/Skin links and inverse-bind data are precomputed into stable graph ids and values, while cycle-time tendon-driven dirt additionally advances renderer path/command epochs."
~~~

## B6-0120

~~~yaml
row_id: B6-0120
cpp_files: ["src/bones/weight.cpp"]
rust_module: "crates/nuxie-runtime/src/components.rs"
subsystem_cluster: bones-math-components
sibling_files_swept: ["crates/nuxie-runtime/src/components.rs", "crates/nuxie-runtime/src/artboard.rs", "crates/nuxie-runtime/src/draw.rs", "crates/nuxie-runtime/src/constraints.rs", "crates/nuxie-runtime/src/objects.rs", "crates/nuxie-runtime/src/text.rs", "crates/nuxie-runtime/src/data_bind_graph.rs", "crates/nuxie-graph/src/lib.rs", "crates/nuxie-render-api/src/lib.rs"]
verdict: DIVERGENT
axes:
  retained_identity: {status: adapted, idiom_rule: "AF-1 Retained identity via stable arena/local id", evidence: ["src/bones/weight.cpp:7-55", "crates/nuxie-runtime/src/draw.rs:21543-21588", "crates/nuxie-runtime/src/draw.rs:6420-6465"]}
  push_vs_poll: {status: isomorphic, cpp_pushes: true, evidence: ["src/bones/weight.cpp:7-55", "crates/nuxie-graph/src/lib.rs:3768-3781", "crates/nuxie-runtime/src/artboard.rs:3948-3998"], note: "Stable dependency edges push dirt; no relationship rescan or observed-value diff was found."}
  update_ordering: {status: isomorphic, phases_cpp: ["property mutation", "dirt collection/cascade", "graph-ordered component update", "dependent update"], phases_rust: ["property mutation", "dirt collection/cascade", "graph-ordered component update", "dependent update"]}
  ownership: {status: adapted, evidence: ["src/bones/weight.cpp:7-55", "crates/nuxie-runtime/src/draw.rs:21543-21588", "crates/nuxie-runtime/src/draw.rs:6420-6465"]}
  compensation:
    status: divergent
    mechanisms:
      - {name: "cache_epoch", kind: "cross-frame cache invalidation generation", mutation_gated: true, cpp_counterpart: none, evidence: ["crates/nuxie-runtime/src/artboard.rs:3728-3731", "crates/nuxie-runtime/src/artboard.rs:4424-4434", "crates/nuxie-runtime/src/draw.rs:2221-2286"]}
      - {name: "path_epoch", kind: "path-geometry invalidation generation", mutation_gated: true, cpp_counterpart: none, evidence: ["crates/nuxie-runtime/src/artboard.rs:3788-3790", "crates/nuxie-runtime/src/artboard.rs:3965-3969", "crates/nuxie-runtime/src/draw.rs:10958-10978"]}
      - {name: "prepared_epoch", kind: "prepared-frame invalidation generation", mutation_gated: true, cpp_counterpart: none, evidence: ["crates/nuxie-runtime/src/artboard.rs:3747-3755", "crates/nuxie-runtime/src/artboard.rs:3948-3973", "crates/nuxie-runtime/src/draw.rs:2239-2249"]}
      - {name: "command_epoch", kind: "draw-command invalidation generation", mutation_gated: true, cpp_counterpart: none, evidence: ["crates/nuxie-runtime/src/artboard.rs:3747-3750", "crates/nuxie-runtime/src/draw.rs:10772-10790"]}
      - {name: "tree paint preparation epoch", kind: "shared tree-level invalidation generation", mutation_gated: true, cpp_counterpart: none, evidence: ["crates/nuxie-runtime/src/artboard.rs:1621-1632", "crates/nuxie-runtime/src/artboard.rs:3747-3755", "crates/nuxie-runtime/src/draw.rs:2239-2250"]}
    import_time_constants:
      - {name: "TransformPropertyKeys and OnceLock property-key cache", idiom_rule: "AF-5 Import-time devirtualization is legitimate", evidence: ["crates/nuxie-runtime/src/components.rs:160-243", "crates/nuxie-runtime/src/components.rs:477-513"], note: "computed at build/static lookup time and not written during advance/update/bind"}
      - {name: "skeletal graph descriptors and dependency indexes", idiom_rule: "AF-5 Import-time devirtualization is legitimate", evidence: ["crates/nuxie-graph/src/lib.rs:842-871", "crates/nuxie-graph/src/lib.rs:2895-3016", "crates/nuxie-graph/src/lib.rs:5026-5079"], note: "built once from the imported graph; not a cycle-time drift tracker"}
idiom_rules_invoked: ["AF-1 Retained identity via stable arena/local id", "AF-5 Import-time devirtualization is legitimate"]
confidence: high
notes: "The four-weight deformation arithmetic is direct, but its live skeletal inputs are reconstructed for path preparation and their dirt mutates the renderer epoch family used to track cached weighted geometry."
~~~

## B6-0123

~~~yaml
row_id: B6-0123
cpp_files: ["src/component.cpp"]
rust_module: "crates/nuxie-runtime/src/components.rs"
subsystem_cluster: bones-math-components
sibling_files_swept: ["crates/nuxie-runtime/src/components.rs", "crates/nuxie-runtime/src/artboard.rs", "crates/nuxie-runtime/src/draw.rs", "crates/nuxie-runtime/src/constraints.rs", "crates/nuxie-runtime/src/objects.rs", "crates/nuxie-runtime/src/text.rs", "crates/nuxie-runtime/src/data_bind_graph.rs", "crates/nuxie-graph/src/lib.rs", "crates/nuxie-render-api/src/lib.rs"]
verdict: DIVERGENT
axes:
  retained_identity: {status: isomorphic, evidence: ["src/component.cpp:13-127", "crates/nuxie-runtime/src/components.rs:14-105", "crates/nuxie-runtime/src/components.rs:477-557", "crates/nuxie-runtime/src/artboard.rs:3948-3998", "crates/nuxie-runtime/src/artboard.rs:4365-4466"]}
  push_vs_poll: {status: isomorphic, cpp_pushes: true, evidence: ["src/component.cpp:13-127", "crates/nuxie-graph/src/lib.rs:3768-3781", "crates/nuxie-runtime/src/artboard.rs:3948-3998"], note: "Stable dependency edges push dirt; no relationship rescan or observed-value diff was found."}
  update_ordering: {status: isomorphic, phases_cpp: ["property mutation", "dirt collection/cascade", "graph-ordered component update", "dependent update"], phases_rust: ["property mutation", "dirt collection/cascade", "graph-ordered component update", "dependent update"]}
  ownership: {status: isomorphic, evidence: ["src/component.cpp:13-127", "crates/nuxie-runtime/src/components.rs:14-105", "crates/nuxie-runtime/src/components.rs:477-557", "crates/nuxie-runtime/src/artboard.rs:3948-3998", "crates/nuxie-runtime/src/artboard.rs:4365-4466"]}
  compensation:
    status: divergent
    mechanisms:
      - {name: "cache_epoch", kind: "cross-frame cache invalidation generation", mutation_gated: true, cpp_counterpart: none, evidence: ["crates/nuxie-runtime/src/artboard.rs:3728-3731", "crates/nuxie-runtime/src/artboard.rs:4424-4434", "crates/nuxie-runtime/src/draw.rs:2221-2286"]}
      - {name: "prepared_epoch", kind: "prepared-frame invalidation generation", mutation_gated: true, cpp_counterpart: none, evidence: ["crates/nuxie-runtime/src/artboard.rs:3747-3755", "crates/nuxie-runtime/src/artboard.rs:3948-3973", "crates/nuxie-runtime/src/draw.rs:2239-2249"]}
      - {name: "command_epoch", kind: "draw-command invalidation generation", mutation_gated: true, cpp_counterpart: none, evidence: ["crates/nuxie-runtime/src/artboard.rs:3747-3750", "crates/nuxie-runtime/src/draw.rs:10772-10790"]}
      - {name: "path_epoch", kind: "path-geometry invalidation generation", mutation_gated: true, cpp_counterpart: none, evidence: ["crates/nuxie-runtime/src/artboard.rs:3788-3790", "crates/nuxie-runtime/src/artboard.rs:3965-3969", "crates/nuxie-runtime/src/draw.rs:10958-10978"]}
      - {name: "layout_epoch", kind: "layout-cache invalidation generation", mutation_gated: true, cpp_counterpart: none, evidence: ["crates/nuxie-runtime/src/artboard.rs:3783-3785", "crates/nuxie-runtime/src/artboard.rs:3961-3964", "crates/nuxie-runtime/src/draw.rs:10885-10917"]}
      - {name: "text_epoch", kind: "text-cache invalidation generation", mutation_gated: true, cpp_counterpart: none, evidence: ["crates/nuxie-runtime/src/artboard.rs:3793-3805", "crates/nuxie-runtime/src/draw.rs:11008-11020"]}
      - {name: "draw_order_epoch", kind: "draw-order invalidation generation", mutation_gated: true, cpp_counterpart: none, evidence: ["crates/nuxie-runtime/src/artboard.rs:3817-3819", "crates/nuxie-runtime/src/artboard.rs:3970-3972", "crates/nuxie-runtime/src/draw.rs:10858-10882"]}
      - {name: "tree paint preparation epoch", kind: "shared tree-level invalidation generation", mutation_gated: true, cpp_counterpart: none, evidence: ["crates/nuxie-runtime/src/artboard.rs:1621-1632", "crates/nuxie-runtime/src/artboard.rs:3747-3755", "crates/nuxie-runtime/src/draw.rs:2239-2250"]}
    import_time_constants:
      - {name: "TransformPropertyKeys and OnceLock property-key cache", idiom_rule: "AF-5 Import-time devirtualization is legitimate", evidence: ["crates/nuxie-runtime/src/components.rs:160-243", "crates/nuxie-runtime/src/components.rs:477-513"], note: "computed at build/static lookup time and not written during advance/update/bind"}
idiom_rules_invoked: ["AF-4 One dirt model"]
confidence: high
notes: "Rust preserves the per-component dirt bitset, dependent cascade, graph order, collapse handling, and update loop. The same cycle-time dirt bridge also mutates eight cross-frame cache/renderer epoch mechanisms absent as fields in Component."
~~~

## B6-0289

~~~yaml
row_id: B6-0289
cpp_files: ["src/math/aabb.cpp"]
rust_module: "crates/nuxie-runtime/src/components.rs"
subsystem_cluster: bones-math-components
sibling_files_swept: ["crates/nuxie-runtime/src/components.rs", "crates/nuxie-runtime/src/artboard.rs", "crates/nuxie-runtime/src/draw.rs", "crates/nuxie-runtime/src/constraints.rs", "crates/nuxie-runtime/src/objects.rs", "crates/nuxie-runtime/src/text.rs", "crates/nuxie-runtime/src/data_bind_graph.rs", "crates/nuxie-graph/src/lib.rs", "crates/nuxie-render-api/src/lib.rs"]
verdict: ISOMORPHIC
axes:
  retained_identity: {status: isomorphic, evidence: ["src/math/aabb.cpp:9-91", "crates/nuxie-render-api/src/lib.rs:26-60", "crates/nuxie-runtime/src/draw.rs:18078-18154"], note: "AABB operations is represented with owned values; no shared mutable identity is required by this relationship."}
  push_vs_poll: {status: isomorphic, cpp_pushes: false, evidence: ["src/math/aabb.cpp:9-91", "crates/nuxie-render-api/src/lib.rs:26-60", "crates/nuxie-runtime/src/draw.rs:18078-18154"], note: "This is a direct utility/query relationship; neither side registers an observer for it."}
  update_ordering: {status: isomorphic, phases_cpp: ["call utility/query", "compute in local values", "return/apply result"], phases_rust: ["call utility/query", "compute in local values", "return/apply result"]}
  ownership: {status: isomorphic, evidence: ["src/math/aabb.cpp:9-91", "crates/nuxie-render-api/src/lib.rs:26-60", "crates/nuxie-runtime/src/draw.rs:18078-18154"]}
  compensation:
    status: isomorphic
    mechanisms: []
    import_time_constants:
      - {name: "TransformPropertyKeys and OnceLock property-key cache", idiom_rule: "AF-5 Import-time devirtualization is legitimate", evidence: ["crates/nuxie-runtime/src/components.rs:160-243", "crates/nuxie-runtime/src/components.rs:477-513"], note: "computed at build/static lookup time and not written during advance/update/bind"}
idiom_rules_invoked: ["AF-3 Poll only where C++ polls"]
confidence: high
notes: "Both sides use owned value bounds and stateless arithmetic; no observer relationship or cycle-persistent drift tracker belongs to this row."
~~~

## B6-0290

~~~yaml
row_id: B6-0290
cpp_files: ["src/math/bezier_utils.cpp"]
rust_module: "crates/nuxie-runtime/src/components.rs"
subsystem_cluster: bones-math-components
sibling_files_swept: ["crates/nuxie-runtime/src/components.rs", "crates/nuxie-runtime/src/artboard.rs", "crates/nuxie-runtime/src/draw.rs", "crates/nuxie-runtime/src/constraints.rs", "crates/nuxie-runtime/src/objects.rs", "crates/nuxie-runtime/src/text.rs", "crates/nuxie-runtime/src/data_bind_graph.rs", "crates/nuxie-graph/src/lib.rs", "crates/nuxie-render-api/src/lib.rs"]
verdict: ISOMORPHIC
axes:
  retained_identity: {status: isomorphic, evidence: ["src/math/bezier_utils.cpp:21-332", "crates/nuxie-runtime/src/draw.rs:20055-20212", "crates/nuxie-runtime/src/draw.rs:19381-19824"], note: "Bezier evaluation and subdivision is represented with owned values; no shared mutable identity is required by this relationship."}
  push_vs_poll: {status: isomorphic, cpp_pushes: false, evidence: ["src/math/bezier_utils.cpp:21-332", "crates/nuxie-runtime/src/draw.rs:20055-20212", "crates/nuxie-runtime/src/draw.rs:19381-19824"], note: "This is a direct utility/query relationship; neither side registers an observer for it."}
  update_ordering: {status: isomorphic, phases_cpp: ["call utility/query", "compute in local values", "return/apply result"], phases_rust: ["call utility/query", "compute in local values", "return/apply result"]}
  ownership: {status: isomorphic, evidence: ["src/math/bezier_utils.cpp:21-332", "crates/nuxie-runtime/src/draw.rs:20055-20212", "crates/nuxie-runtime/src/draw.rs:19381-19824"]}
  compensation:
    status: isomorphic
    mechanisms: []
    import_time_constants:
      - {name: "TransformPropertyKeys and OnceLock property-key cache", idiom_rule: "AF-5 Import-time devirtualization is legitimate", evidence: ["crates/nuxie-runtime/src/components.rs:160-243", "crates/nuxie-runtime/src/components.rs:477-513"], note: "computed at build/static lookup time and not written during advance/update/bind"}
idiom_rules_invoked: ["AF-3 Poll only where C++ polls"]
confidence: high
notes: "The Rust sibling keeps the Bezier work in local values and follows the same stateless geometry phases. Crate-wide cache/epoch hits belong to enclosing retained draw caches, not this utility relationship."
~~~

## B6-0291

~~~yaml
row_id: B6-0291
cpp_files: ["src/math/bit_field_loc.cpp"]
rust_module: "crates/nuxie-runtime/src/components.rs"
subsystem_cluster: bones-math-components
sibling_files_swept: ["crates/nuxie-runtime/src/components.rs", "crates/nuxie-runtime/src/artboard.rs", "crates/nuxie-runtime/src/draw.rs", "crates/nuxie-runtime/src/constraints.rs", "crates/nuxie-runtime/src/objects.rs", "crates/nuxie-runtime/src/text.rs", "crates/nuxie-runtime/src/data_bind_graph.rs", "crates/nuxie-graph/src/lib.rs", "crates/nuxie-render-api/src/lib.rs"]
verdict: ADAPTED
axes:
  retained_identity: {status: adapted, idiom_rule: "AF-5 import-time devirtualization; AF-7 own-by-value", evidence: ["src/math/bit_field_loc.cpp:6-19", "crates/nuxie-runtime/src/objects.rs:278-321"], note: "bit-field location masking is represented with owned values; no shared mutable identity is required by this relationship."}
  push_vs_poll: {status: isomorphic, cpp_pushes: false, evidence: ["src/math/bit_field_loc.cpp:6-19", "crates/nuxie-runtime/src/objects.rs:278-321"], note: "This is a direct utility/query relationship; neither side registers an observer for it."}
  update_ordering: {status: isomorphic, phases_cpp: ["call utility/query", "compute in local values", "return/apply result"], phases_rust: ["call utility/query", "compute in local values", "return/apply result"]}
  ownership: {status: adapted, idiom_rule: "AF-5 import-time devirtualization; AF-7 own-by-value", evidence: ["src/math/bit_field_loc.cpp:6-19", "crates/nuxie-runtime/src/objects.rs:278-321"]}
  compensation:
    status: adapted
    mechanisms: []
    import_time_constants:
      - {name: "TransformPropertyKeys and OnceLock property-key cache", idiom_rule: "AF-5 Import-time devirtualization is legitimate", evidence: ["crates/nuxie-runtime/src/components.rs:160-243", "crates/nuxie-runtime/src/components.rs:477-513"], note: "computed at build/static lookup time and not written during advance/update/bind"}
idiom_rules_invoked: ["AF-5 import-time devirtualization", "AF-7 own-by-value"]
confidence: high
notes: "C++ packages bit offset/width in a helper object; Rust uses imported schema metadata and performs the mask operation inline over owned values."
~~~

## B6-0292

~~~yaml
row_id: B6-0292
cpp_files: ["src/math/contour_measure.cpp"]
rust_module: "crates/nuxie-runtime/src/components.rs"
subsystem_cluster: bones-math-components
sibling_files_swept: ["crates/nuxie-runtime/src/components.rs", "crates/nuxie-runtime/src/artboard.rs", "crates/nuxie-runtime/src/draw.rs", "crates/nuxie-runtime/src/constraints.rs", "crates/nuxie-runtime/src/objects.rs", "crates/nuxie-runtime/src/text.rs", "crates/nuxie-runtime/src/data_bind_graph.rs", "crates/nuxie-graph/src/lib.rs", "crates/nuxie-render-api/src/lib.rs"]
verdict: ADAPTED
axes:
  retained_identity: {status: adapted, idiom_rule: "AF-7 own-by-value", evidence: ["src/math/contour_measure.cpp:28-608", "crates/nuxie-runtime/src/draw.rs:19381-19824", "crates/nuxie-runtime/src/draw.rs:20055-20212"], note: "contour measurement is represented with owned values; no shared mutable identity is required by this relationship."}
  push_vs_poll: {status: isomorphic, cpp_pushes: false, evidence: ["src/math/contour_measure.cpp:28-608", "crates/nuxie-runtime/src/draw.rs:19381-19824", "crates/nuxie-runtime/src/draw.rs:20055-20212"], note: "This is a direct utility/query relationship; neither side registers an observer for it."}
  update_ordering: {status: isomorphic, phases_cpp: ["call utility/query", "compute in local values", "return/apply result"], phases_rust: ["call utility/query", "compute in local values", "return/apply result"]}
  ownership: {status: adapted, idiom_rule: "AF-7 own-by-value", evidence: ["src/math/contour_measure.cpp:28-608", "crates/nuxie-runtime/src/draw.rs:19381-19824", "crates/nuxie-runtime/src/draw.rs:20055-20212"]}
  compensation:
    status: adapted
    mechanisms: []
    import_time_constants:
      - {name: "TransformPropertyKeys and OnceLock property-key cache", idiom_rule: "AF-5 Import-time devirtualization is legitimate", evidence: ["crates/nuxie-runtime/src/components.rs:160-243", "crates/nuxie-runtime/src/components.rs:477-513"], note: "computed at build/static lookup time and not written during advance/update/bind"}
idiom_rules_invoked: ["AF-7 own-by-value"]
confidence: high
notes: "C++ retains heap-owned contour segments; Rust owns the equivalent segment/value data in local Vecs at the path-measure seam. Phase sequence and stateless evaluation are preserved."
~~~

## B6-0293

~~~yaml
row_id: B6-0293
cpp_files: ["src/math/hit_test.cpp"]
rust_module: "crates/nuxie-runtime/src/components.rs"
subsystem_cluster: bones-math-components
sibling_files_swept: ["crates/nuxie-runtime/src/components.rs", "crates/nuxie-runtime/src/artboard.rs", "crates/nuxie-runtime/src/draw.rs", "crates/nuxie-runtime/src/constraints.rs", "crates/nuxie-runtime/src/objects.rs", "crates/nuxie-runtime/src/text.rs", "crates/nuxie-runtime/src/data_bind_graph.rs", "crates/nuxie-graph/src/lib.rs", "crates/nuxie-render-api/src/lib.rs"]
verdict: ADAPTED
axes:
  retained_identity: {status: adapted, idiom_rule: "AF-5 import-time devirtualization; AF-7 own-by-value", evidence: ["src/math/hit_test.cpp:17-469", "crates/nuxie-runtime/src/draw.rs:18195-18760", "crates/nuxie-render-api/src/lib.rs:26-70"], note: "path hit testing is represented with owned values; no shared mutable identity is required by this relationship."}
  push_vs_poll: {status: isomorphic, cpp_pushes: false, evidence: ["src/math/hit_test.cpp:17-469", "crates/nuxie-runtime/src/draw.rs:18195-18760", "crates/nuxie-render-api/src/lib.rs:26-70"], note: "This is a direct utility/query relationship; neither side registers an observer for it."}
  update_ordering: {status: isomorphic, phases_cpp: ["call utility/query", "compute in local values", "return/apply result"], phases_rust: ["call utility/query", "compute in local values", "return/apply result"]}
  ownership: {status: adapted, idiom_rule: "AF-5 import-time devirtualization; AF-7 own-by-value", evidence: ["src/math/hit_test.cpp:17-469", "crates/nuxie-runtime/src/draw.rs:18195-18760", "crates/nuxie-render-api/src/lib.rs:26-70"]}
  compensation:
    status: adapted
    mechanisms: []
    import_time_constants:
      - {name: "TransformPropertyKeys and OnceLock property-key cache", idiom_rule: "AF-5 Import-time devirtualization is legitimate", evidence: ["crates/nuxie-runtime/src/components.rs:160-243", "crates/nuxie-runtime/src/components.rs:477-513"], note: "computed at build/static lookup time and not written during advance/update/bind"}
idiom_rules_invoked: ["AF-5 import-time devirtualization", "AF-7 own-by-value"]
confidence: high
notes: "Rust consolidates command decoding and winding/crossing work into owned local geometry rather than retaining the C++ helper objects; the hit-test call remains a direct query with no invented lifecycle."
~~~

## B6-0294

~~~yaml
row_id: B6-0294
cpp_files: ["src/math/mat2d.cpp"]
rust_module: "crates/nuxie-runtime/src/components.rs"
subsystem_cluster: bones-math-components
sibling_files_swept: ["crates/nuxie-runtime/src/components.rs", "crates/nuxie-runtime/src/artboard.rs", "crates/nuxie-runtime/src/draw.rs", "crates/nuxie-runtime/src/constraints.rs", "crates/nuxie-runtime/src/objects.rs", "crates/nuxie-runtime/src/text.rs", "crates/nuxie-runtime/src/data_bind_graph.rs", "crates/nuxie-graph/src/lib.rs", "crates/nuxie-render-api/src/lib.rs"]
verdict: ISOMORPHIC
axes:
  retained_identity: {status: isomorphic, evidence: ["src/math/mat2d.cpp:10-245", "crates/nuxie-runtime/src/components.rs:278-392", "crates/nuxie-render-api/src/lib.rs:62-86"], note: "Mat2D operations is represented with owned values; no shared mutable identity is required by this relationship."}
  push_vs_poll: {status: isomorphic, cpp_pushes: false, evidence: ["src/math/mat2d.cpp:10-245", "crates/nuxie-runtime/src/components.rs:278-392", "crates/nuxie-render-api/src/lib.rs:62-86"], note: "This is a direct utility/query relationship; neither side registers an observer for it."}
  update_ordering: {status: isomorphic, phases_cpp: ["call utility/query", "compute in local values", "return/apply result"], phases_rust: ["call utility/query", "compute in local values", "return/apply result"]}
  ownership: {status: isomorphic, evidence: ["src/math/mat2d.cpp:10-245", "crates/nuxie-runtime/src/components.rs:278-392", "crates/nuxie-render-api/src/lib.rs:62-86"]}
  compensation:
    status: isomorphic
    mechanisms: []
    import_time_constants:
      - {name: "TransformPropertyKeys and OnceLock property-key cache", idiom_rule: "AF-5 Import-time devirtualization is legitimate", evidence: ["crates/nuxie-runtime/src/components.rs:160-243", "crates/nuxie-runtime/src/components.rs:477-513"], note: "computed at build/static lookup time and not written during advance/update/bind"}
idiom_rules_invoked: ["AF-3 Poll only where C++ polls"]
confidence: high
notes: "Both sides use six-float value matrices with direct multiply, invert, compose/decompose, and point/direction transforms; no retained observer or compensation relationship exists."
~~~

## B6-0295

~~~yaml
row_id: B6-0295
cpp_files: ["src/math/mat2d_find_max_scale.cpp"]
rust_module: "crates/nuxie-runtime/src/components.rs"
subsystem_cluster: bones-math-components
sibling_files_swept: ["crates/nuxie-runtime/src/components.rs", "crates/nuxie-runtime/src/artboard.rs", "crates/nuxie-runtime/src/draw.rs", "crates/nuxie-runtime/src/constraints.rs", "crates/nuxie-runtime/src/objects.rs", "crates/nuxie-runtime/src/text.rs", "crates/nuxie-runtime/src/data_bind_graph.rs", "crates/nuxie-graph/src/lib.rs", "crates/nuxie-render-api/src/lib.rs"]
verdict: UNKNOWN
axes:
  retained_identity: {status: unknown, evidence: ["src/math/mat2d_find_max_scale.cpp:17-67", "crates/nuxie-runtime/src/components.rs:278-392", "crates/nuxie-render-api/src/lib.rs:62-86"], blocker: "No implementation of the singular-value/max-scale helper was found in the mapped module or crate-wide sibling sweep, so the same relationship cannot be compared honestly."}
  push_vs_poll: {status: unknown, cpp_pushes: false, evidence: ["src/math/mat2d_find_max_scale.cpp:17-67", "crates/nuxie-runtime/src/components.rs:278-392", "crates/nuxie-render-api/src/lib.rs:62-86"], blocker: "No implementation of the singular-value/max-scale helper was found in the mapped module or crate-wide sibling sweep, so the same relationship cannot be compared honestly."}
  update_ordering: {status: unknown, phases_cpp: ["utility invocation"], phases_rust: [], blocker: "No implementation of the singular-value/max-scale helper was found in the mapped module or crate-wide sibling sweep, so the same relationship cannot be compared honestly."}
  ownership: {status: unknown, evidence: ["src/math/mat2d_find_max_scale.cpp:17-67", "crates/nuxie-runtime/src/components.rs:278-392", "crates/nuxie-render-api/src/lib.rs:62-86"], blocker: "No implementation of the singular-value/max-scale helper was found in the mapped module or crate-wide sibling sweep, so the same relationship cannot be compared honestly."}
  compensation:
    status: unknown
    mechanisms: []
    import_time_constants:
      - {name: "TransformPropertyKeys and OnceLock property-key cache", idiom_rule: "AF-5 Import-time devirtualization is legitimate", evidence: ["crates/nuxie-runtime/src/components.rs:160-243", "crates/nuxie-runtime/src/components.rs:477-513"], note: "computed at build/static lookup time and not written during advance/update/bind"}
idiom_rules_invoked: ["AF-5 Import-time devirtualization is legitimate"]
confidence: high
notes: "No implementation of the singular-value/max-scale helper was found in the mapped module or crate-wide sibling sweep, so the same relationship cannot be compared honestly. UNKNOWN names the blocker and makes no remediation decision."
~~~

## B6-0296

~~~yaml
row_id: B6-0296
cpp_files: ["src/math/n_slicer_helpers.cpp"]
rust_module: "crates/nuxie-runtime/src/components.rs"
subsystem_cluster: bones-math-components
sibling_files_swept: ["crates/nuxie-runtime/src/components.rs", "crates/nuxie-runtime/src/artboard.rs", "crates/nuxie-runtime/src/draw.rs", "crates/nuxie-runtime/src/constraints.rs", "crates/nuxie-runtime/src/objects.rs", "crates/nuxie-runtime/src/text.rs", "crates/nuxie-runtime/src/data_bind_graph.rs", "crates/nuxie-graph/src/lib.rs", "crates/nuxie-render-api/src/lib.rs"]
verdict: ISOMORPHIC
axes:
  retained_identity: {status: isomorphic, evidence: ["src/math/n_slicer_helpers.cpp:10-158", "crates/nuxie-runtime/src/draw.rs:21589-21955"], note: "N-slicer axis scaling helpers is represented with owned values; no shared mutable identity is required by this relationship."}
  push_vs_poll: {status: isomorphic, cpp_pushes: false, evidence: ["src/math/n_slicer_helpers.cpp:10-158", "crates/nuxie-runtime/src/draw.rs:21589-21955"], note: "This is a direct utility/query relationship; neither side registers an observer for it."}
  update_ordering: {status: isomorphic, phases_cpp: ["call utility/query", "compute in local values", "return/apply result"], phases_rust: ["call utility/query", "compute in local values", "return/apply result"]}
  ownership: {status: isomorphic, evidence: ["src/math/n_slicer_helpers.cpp:10-158", "crates/nuxie-runtime/src/draw.rs:21589-21955"]}
  compensation:
    status: isomorphic
    mechanisms: []
    import_time_constants:
      - {name: "TransformPropertyKeys and OnceLock property-key cache", idiom_rule: "AF-5 Import-time devirtualization is legitimate", evidence: ["crates/nuxie-runtime/src/components.rs:160-243", "crates/nuxie-runtime/src/components.rs:477-513"], note: "computed at build/static lookup time and not written during advance/update/bind"}
idiom_rules_invoked: ["AF-3 Poll only where C++ polls"]
confidence: high
notes: "Both sides compute stops, fixed/scalable spans, scale factors, and mapped offsets from local values in the same query phase; no cycle-persistent drift tracker belongs to the helper."
~~~

## B6-0297

~~~yaml
row_id: B6-0297
cpp_files: ["src/math/path_measure.cpp"]
rust_module: "crates/nuxie-runtime/src/components.rs"
subsystem_cluster: bones-math-components
sibling_files_swept: ["crates/nuxie-runtime/src/components.rs", "crates/nuxie-runtime/src/artboard.rs", "crates/nuxie-runtime/src/draw.rs", "crates/nuxie-runtime/src/constraints.rs", "crates/nuxie-runtime/src/objects.rs", "crates/nuxie-runtime/src/text.rs", "crates/nuxie-runtime/src/data_bind_graph.rs", "crates/nuxie-graph/src/lib.rs", "crates/nuxie-render-api/src/lib.rs"]
verdict: ADAPTED
axes:
  retained_identity: {status: adapted, idiom_rule: "AF-7 own-by-value", evidence: ["src/math/path_measure.cpp:6-113", "crates/nuxie-runtime/src/draw.rs:19381-19824"], note: "path measurement is represented with owned values; no shared mutable identity is required by this relationship."}
  push_vs_poll: {status: isomorphic, cpp_pushes: false, evidence: ["src/math/path_measure.cpp:6-113", "crates/nuxie-runtime/src/draw.rs:19381-19824"], note: "This is a direct utility/query relationship; neither side registers an observer for it."}
  update_ordering: {status: isomorphic, phases_cpp: ["call utility/query", "compute in local values", "return/apply result"], phases_rust: ["call utility/query", "compute in local values", "return/apply result"]}
  ownership: {status: adapted, idiom_rule: "AF-7 own-by-value", evidence: ["src/math/path_measure.cpp:6-113", "crates/nuxie-runtime/src/draw.rs:19381-19824"]}
  compensation:
    status: adapted
    mechanisms: []
    import_time_constants:
      - {name: "TransformPropertyKeys and OnceLock property-key cache", idiom_rule: "AF-5 Import-time devirtualization is legitimate", evidence: ["crates/nuxie-runtime/src/components.rs:160-243", "crates/nuxie-runtime/src/components.rs:477-513"], note: "computed at build/static lookup time and not written during advance/update/bind"}
idiom_rules_invoked: ["AF-7 own-by-value"]
confidence: high
notes: "C++ owns contour measures through unique ownership; Rust returns/owns the equivalent contour values in Vecs. Iteration and measurement remain direct, with no polling relationship."
~~~

## B6-0298

~~~yaml
row_id: B6-0298
cpp_files: ["src/math/random.cpp"]
rust_module: "crates/nuxie-runtime/src/components.rs"
subsystem_cluster: bones-math-components
sibling_files_swept: ["crates/nuxie-runtime/src/components.rs", "crates/nuxie-runtime/src/artboard.rs", "crates/nuxie-runtime/src/draw.rs", "crates/nuxie-runtime/src/constraints.rs", "crates/nuxie-runtime/src/objects.rs", "crates/nuxie-runtime/src/text.rs", "crates/nuxie-runtime/src/data_bind_graph.rs", "crates/nuxie-graph/src/lib.rs", "crates/nuxie-render-api/src/lib.rs"]
verdict: ADAPTED
axes:
  retained_identity: {status: adapted, idiom_rule: "AF-6 Deep copy is explicit; AF-8 No invented lifecycles", evidence: ["src/math/random.cpp:11-15", "crates/nuxie-runtime/src/data_bind_graph.rs:130-168"], note: "random-number source is represented with owned values; no shared mutable identity is required by this relationship."}
  push_vs_poll: {status: isomorphic, cpp_pushes: false, evidence: ["src/math/random.cpp:11-15", "crates/nuxie-runtime/src/data_bind_graph.rs:130-168"], note: "This is a direct utility/query relationship; neither side registers an observer for it."}
  update_ordering: {status: isomorphic, phases_cpp: ["call utility/query", "compute in local values", "return/apply result"], phases_rust: ["call utility/query", "compute in local values", "return/apply result"]}
  ownership: {status: adapted, idiom_rule: "AF-6 Deep copy is explicit; AF-8 No invented lifecycles", evidence: ["src/math/random.cpp:11-15", "crates/nuxie-runtime/src/data_bind_graph.rs:130-168"]}
  compensation:
    status: adapted
    mechanisms: []
    import_time_constants:
      - {name: "TransformPropertyKeys and OnceLock property-key cache", idiom_rule: "AF-5 Import-time devirtualization is legitimate", evidence: ["crates/nuxie-runtime/src/components.rs:160-243", "crates/nuxie-runtime/src/components.rs:477-513"], note: "computed at build/static lookup time and not written during advance/update/bind"}
idiom_rules_invoked: ["AF-6 Deep copy is explicit", "AF-8 No invented lifecycles"]
confidence: high
notes: "The pinned C++ .cpp only supplies the TESTING counter while the header/provider owns the sequence contract. Rust retains values, cursor, call count, and fallback seed per formula source and resets them only at the explicit seed/clone lifecycle."
~~~

## B6-0299

~~~yaml
row_id: B6-0299
cpp_files: ["src/math/raw_path.cpp"]
rust_module: "crates/nuxie-runtime/src/components.rs"
subsystem_cluster: bones-math-components
sibling_files_swept: ["crates/nuxie-runtime/src/components.rs", "crates/nuxie-runtime/src/artboard.rs", "crates/nuxie-runtime/src/draw.rs", "crates/nuxie-runtime/src/constraints.rs", "crates/nuxie-runtime/src/objects.rs", "crates/nuxie-runtime/src/text.rs", "crates/nuxie-runtime/src/data_bind_graph.rs", "crates/nuxie-graph/src/lib.rs", "crates/nuxie-render-api/src/lib.rs"]
verdict: ADAPTED
axes:
  retained_identity: {status: adapted, idiom_rule: "AF-6 Deep copy is explicit; AF-7 own-by-value", evidence: ["src/math/raw_path.cpp:17-770", "crates/nuxie-render-api/src/lib.rs:95-410"], note: "raw-path storage and mutation is represented with owned values; no shared mutable identity is required by this relationship."}
  push_vs_poll: {status: isomorphic, cpp_pushes: false, evidence: ["src/math/raw_path.cpp:17-770", "crates/nuxie-render-api/src/lib.rs:95-410"], note: "This is a direct utility/query relationship; neither side registers an observer for it."}
  update_ordering: {status: isomorphic, phases_cpp: ["call utility/query", "compute in local values", "return/apply result"], phases_rust: ["call utility/query", "compute in local values", "return/apply result"]}
  ownership: {status: adapted, idiom_rule: "AF-6 Deep copy is explicit; AF-7 own-by-value", evidence: ["src/math/raw_path.cpp:17-770", "crates/nuxie-render-api/src/lib.rs:95-410"]}
  compensation:
    status: adapted
    mechanisms: []
    import_time_constants:
      - {name: "TransformPropertyKeys and OnceLock property-key cache", idiom_rule: "AF-5 Import-time devirtualization is legitimate", evidence: ["crates/nuxie-runtime/src/components.rs:160-243", "crates/nuxie-runtime/src/components.rs:477-513"], note: "computed at build/static lookup time and not written during advance/update/bind"}
      - {name: "RawPath mutation_id", idiom_rule: "AF-6 Deep copy is explicit", evidence: ["crates/nuxie-render-api/src/lib.rs:95-105", "crates/nuxie-render-api/src/lib.rs:170-177", "crates/nuxie-render-api/src/lib.rs:188-208", "crates/nuxie-render-api/src/lib.rs:398-400"], note: "runtime-written keyword match, but it has the documented C++ RiveRenderPath mutation-ID counterpart and therefore is not compensation with cpp_counterpart:none"}
idiom_rules_invoked: ["AF-6 Deep copy is explicit", "AF-7 own-by-value"]
confidence: high
notes: "Rust moves RawPath to the render-api ownership seam and owns verbs/points by value. Its mutation_id is written during mutation, but the code explicitly matches C++ RiveRenderPath mutation identity, so it is not a mechanism with cpp_counterpart:none."
~~~

## B6-0300

~~~yaml
row_id: B6-0300
cpp_files: ["src/math/raw_path_utils.cpp"]
rust_module: "crates/nuxie-runtime/src/components.rs"
subsystem_cluster: bones-math-components
sibling_files_swept: ["crates/nuxie-runtime/src/components.rs", "crates/nuxie-runtime/src/artboard.rs", "crates/nuxie-runtime/src/draw.rs", "crates/nuxie-runtime/src/constraints.rs", "crates/nuxie-runtime/src/objects.rs", "crates/nuxie-runtime/src/text.rs", "crates/nuxie-runtime/src/data_bind_graph.rs", "crates/nuxie-graph/src/lib.rs", "crates/nuxie-render-api/src/lib.rs"]
verdict: ISOMORPHIC
axes:
  retained_identity: {status: isomorphic, evidence: ["src/math/raw_path_utils.cpp:11-118", "crates/nuxie-render-api/src/lib.rs:259-410", "crates/nuxie-runtime/src/draw.rs:20055-20212"], note: "raw-path utility operations is represented with owned values; no shared mutable identity is required by this relationship."}
  push_vs_poll: {status: isomorphic, cpp_pushes: false, evidence: ["src/math/raw_path_utils.cpp:11-118", "crates/nuxie-render-api/src/lib.rs:259-410", "crates/nuxie-runtime/src/draw.rs:20055-20212"], note: "This is a direct utility/query relationship; neither side registers an observer for it."}
  update_ordering: {status: isomorphic, phases_cpp: ["call utility/query", "compute in local values", "return/apply result"], phases_rust: ["call utility/query", "compute in local values", "return/apply result"]}
  ownership: {status: isomorphic, evidence: ["src/math/raw_path_utils.cpp:11-118", "crates/nuxie-render-api/src/lib.rs:259-410", "crates/nuxie-runtime/src/draw.rs:20055-20212"]}
  compensation:
    status: isomorphic
    mechanisms: []
    import_time_constants:
      - {name: "TransformPropertyKeys and OnceLock property-key cache", idiom_rule: "AF-5 Import-time devirtualization is legitimate", evidence: ["crates/nuxie-runtime/src/components.rs:160-243", "crates/nuxie-runtime/src/components.rs:477-513"], note: "computed at build/static lookup time and not written during advance/update/bind"}
idiom_rules_invoked: ["AF-3 Poll only where C++ polls"]
confidence: high
notes: "Reverse/path mapping and curve helpers are direct value operations with equivalent query-time phase structure; no observer or drift-tracking relationship belongs to this row."
~~~

## B6-0301

~~~yaml
row_id: B6-0301
cpp_files: ["src/math/rectangles_to_contour.cpp"]
rust_module: "crates/nuxie-runtime/src/components.rs"
subsystem_cluster: bones-math-components
sibling_files_swept: ["crates/nuxie-runtime/src/components.rs", "crates/nuxie-runtime/src/artboard.rs", "crates/nuxie-runtime/src/draw.rs", "crates/nuxie-runtime/src/constraints.rs", "crates/nuxie-runtime/src/objects.rs", "crates/nuxie-runtime/src/text.rs", "crates/nuxie-runtime/src/data_bind_graph.rs", "crates/nuxie-graph/src/lib.rs", "crates/nuxie-render-api/src/lib.rs"]
verdict: UNKNOWN
axes:
  retained_identity: {status: unknown, evidence: ["src/math/rectangles_to_contour.cpp:22-368", "crates/nuxie-runtime/src/draw.rs:18078-18154", "crates/nuxie-render-api/src/lib.rs:26-60"], blocker: "No rectangles-to-contour construction algorithm was found in the mapped module or crate-wide sibling sweep; bounds helpers alone are not the same relationship."}
  push_vs_poll: {status: unknown, cpp_pushes: false, evidence: ["src/math/rectangles_to_contour.cpp:22-368", "crates/nuxie-runtime/src/draw.rs:18078-18154", "crates/nuxie-render-api/src/lib.rs:26-60"], blocker: "No rectangles-to-contour construction algorithm was found in the mapped module or crate-wide sibling sweep; bounds helpers alone are not the same relationship."}
  update_ordering: {status: unknown, phases_cpp: ["utility invocation"], phases_rust: [], blocker: "No rectangles-to-contour construction algorithm was found in the mapped module or crate-wide sibling sweep; bounds helpers alone are not the same relationship."}
  ownership: {status: unknown, evidence: ["src/math/rectangles_to_contour.cpp:22-368", "crates/nuxie-runtime/src/draw.rs:18078-18154", "crates/nuxie-render-api/src/lib.rs:26-60"], blocker: "No rectangles-to-contour construction algorithm was found in the mapped module or crate-wide sibling sweep; bounds helpers alone are not the same relationship."}
  compensation:
    status: unknown
    mechanisms: []
    import_time_constants:
      - {name: "TransformPropertyKeys and OnceLock property-key cache", idiom_rule: "AF-5 Import-time devirtualization is legitimate", evidence: ["crates/nuxie-runtime/src/components.rs:160-243", "crates/nuxie-runtime/src/components.rs:477-513"], note: "computed at build/static lookup time and not written during advance/update/bind"}
idiom_rules_invoked: ["AF-5 Import-time devirtualization is legitimate"]
confidence: high
notes: "No rectangles-to-contour construction algorithm was found in the mapped module or crate-wide sibling sweep; bounds helpers alone are not the same relationship. UNKNOWN names the blocker and makes no remediation decision."
~~~

## B6-0302

~~~yaml
row_id: B6-0302
cpp_files: ["src/math/vec2d.cpp"]
rust_module: "crates/nuxie-runtime/src/components.rs"
subsystem_cluster: bones-math-components
sibling_files_swept: ["crates/nuxie-runtime/src/components.rs", "crates/nuxie-runtime/src/artboard.rs", "crates/nuxie-runtime/src/draw.rs", "crates/nuxie-runtime/src/constraints.rs", "crates/nuxie-runtime/src/objects.rs", "crates/nuxie-runtime/src/text.rs", "crates/nuxie-runtime/src/data_bind_graph.rs", "crates/nuxie-graph/src/lib.rs", "crates/nuxie-render-api/src/lib.rs"]
verdict: ISOMORPHIC
axes:
  retained_identity: {status: isomorphic, evidence: ["src/math/vec2d.cpp:7-25", "crates/nuxie-render-api/src/lib.rs:20-24", "crates/nuxie-runtime/src/draw.rs:20055-20212"], note: "Vec2D arithmetic is represented with owned values; no shared mutable identity is required by this relationship."}
  push_vs_poll: {status: isomorphic, cpp_pushes: false, evidence: ["src/math/vec2d.cpp:7-25", "crates/nuxie-render-api/src/lib.rs:20-24", "crates/nuxie-runtime/src/draw.rs:20055-20212"], note: "This is a direct utility/query relationship; neither side registers an observer for it."}
  update_ordering: {status: isomorphic, phases_cpp: ["call utility/query", "compute in local values", "return/apply result"], phases_rust: ["call utility/query", "compute in local values", "return/apply result"]}
  ownership: {status: isomorphic, evidence: ["src/math/vec2d.cpp:7-25", "crates/nuxie-render-api/src/lib.rs:20-24", "crates/nuxie-runtime/src/draw.rs:20055-20212"]}
  compensation:
    status: isomorphic
    mechanisms: []
    import_time_constants:
      - {name: "TransformPropertyKeys and OnceLock property-key cache", idiom_rule: "AF-5 Import-time devirtualization is legitimate", evidence: ["crates/nuxie-runtime/src/components.rs:160-243", "crates/nuxie-runtime/src/components.rs:477-513"], note: "computed at build/static lookup time and not written during advance/update/bind"}
idiom_rules_invoked: ["AF-3 Poll only where C++ polls"]
confidence: high
notes: "Both sides use two-float owned values and direct arithmetic; there is no retained identity, observer, lifecycle, or compensation mechanism."
~~~

