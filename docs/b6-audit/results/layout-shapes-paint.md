# B-6 Structural Audit — layout-shapes-paint

Pinned C++: /Users/levi/dev/oss/rive-runtime @ d788e8ec6e8b598526607d6a1e8818e8b637b60c.
Coverage sweep: crate-wide family grep for generation/epoch/revision/dirty/observed/snapshot/candidate/alias and subsystem names, followed by sibling inspection of artboard.rs, components.rs, and nuxie-graph graph construction. Renderer-only graph-identity lookup keys are classified under AF-5; mutation-gated epoch/revision/snapshot writes are findings.

## B6-0248

row_id: B6-0248
cpp_files: ["src/layout/artboard_component_list_override.cpp"]
rust_module: "crates/nuxie-runtime/src/draw.rs"
subsystem_cluster: "layout-shapes-paint"
sibling_files_swept: ["crates/nuxie-runtime/src/artboard.rs", "crates/nuxie-runtime/src/components.rs", "crates/nuxie-graph/src/lib.rs"]
verdict: DIVERGENT
axes:
  retained_identity: {status: "adapted-arena-identity", evidence: ["cpp@d788e8ec:src/layout/artboard_component_list_override.cpp:8-19,29-76", "crates/nuxie-graph/src/lib.rs:607-817", "crates/nuxie-runtime/src/artboard.rs:228-320"]}
  push_vs_poll: {status: "isomorphic push; extra drift tracking counted under compensation", cpp_pushes: true, evidence: ["cpp@d788e8ec:src/layout/artboard_component_list_override.cpp:8-19,29-76", "crates/nuxie-runtime/src/artboard.rs:5111-5117; crates/nuxie-runtime/src/draw.rs:6777-6828"]}
  update_ordering: {status: "phase-sequence-equivalent; representation divergent", phases_cpp: "property change -> ComponentDirt/dependent cascade -> component update -> draw", phases_rust: "property write -> dirt cascade -> epoch/snapshot write -> cache preparation/rebuild -> draw"}
  ownership: {status: "isomorphic-or-arena-adapted", evidence: ["cpp@d788e8ec:src/layout/artboard_component_list_override.cpp:8-19,29-76", "crates/nuxie-graph/src/lib.rs:607-817"]}
  compensation:
    status: "present"
    mechanisms: [{name: "mounted child revision/epoch hash family", kind: "cross-file drift tracker", mutation_gated: true, cpp_counterpart: "none", evidence: ["crates/nuxie-runtime/src/artboard.rs:5111-5117; crates/nuxie-runtime/src/draw.rs:6777-6828"]}]
    import_time_constants: []
idiom_rules_invoked: ["AF-1 retained identity", "AF-2 push never reconstruct", "AF-4 one dirt model", "AF-8 no invented lifecycles"]
confidence: high
notes: "C++ retains ArtboardInstance pointers and pushes width/height/host dirt; Rust hashes retained-child epochs/revisions and transient clones to detect stale prepared/layout state. Coverage grep: generation|epoch|revision|dirty|observed|snapshot|candidate|alias plus mesh/effect/path family names across crates/nuxie-runtime/src and crates/nuxie-graph/src; sibling sweep found the off-file members named above."

## B6-0249

row_id: B6-0249
cpp_files: ["src/layout/axis.cpp"]
rust_module: "crates/nuxie-runtime/src/draw.rs"
subsystem_cluster: "layout-shapes-paint"
sibling_files_swept: ["crates/nuxie-runtime/src/artboard.rs", "crates/nuxie-runtime/src/components.rs", "crates/nuxie-graph/src/lib.rs"]
verdict: DIVERGENT
axes:
  retained_identity: {status: "adapted-arena-identity", evidence: ["cpp@d788e8ec:src/layout/axis.cpp:23-29", "crates/nuxie-graph/src/lib.rs:607-817", "crates/nuxie-runtime/src/artboard.rs:228-320"]}
  push_vs_poll: {status: "isomorphic push; extra drift tracking counted under compensation", cpp_pushes: true, evidence: ["cpp@d788e8ec:src/layout/axis.cpp:23-29", "crates/nuxie-runtime/src/draw.rs:9418-9447,13524-13558"]}
  update_ordering: {status: "phase-sequence-equivalent; representation divergent", phases_cpp: "property change -> ComponentDirt/dependent cascade -> component update -> draw", phases_rust: "property write -> dirt cascade -> epoch/snapshot write -> cache preparation/rebuild -> draw"}
  ownership: {status: "isomorphic-or-arena-adapted", evidence: ["cpp@d788e8ec:src/layout/axis.cpp:23-29", "crates/nuxie-graph/src/lib.rs:607-817"]}
  compensation:
    status: "present"
    mechanisms: [{name: "slice-mesh input snapshot", kind: "cross-file drift tracker", mutation_gated: true, cpp_counterpart: "none", evidence: ["crates/nuxie-runtime/src/draw.rs:9418-9447,13524-13558"]}]
    import_time_constants: []
idiom_rules_invoked: ["AF-1 retained identity", "AF-2 push never reconstruct", "AF-4 one dirt model", "AF-8 no invented lifecycles"]
confidence: high
notes: "C++ pushes Axis changes into NSlicer dirt; Rust rebuild gating compares a copied input_words/world_bits snapshot. Coverage grep: generation|epoch|revision|dirty|observed|snapshot|candidate|alias plus mesh/effect/path family names across crates/nuxie-runtime/src and crates/nuxie-graph/src; sibling sweep found the off-file members named above."

## B6-0250

row_id: B6-0250
cpp_files: ["src/layout/axis_x.cpp"]
rust_module: "crates/nuxie-runtime/src/draw.rs"
subsystem_cluster: "layout-shapes-paint"
sibling_files_swept: ["crates/nuxie-runtime/src/artboard.rs", "crates/nuxie-runtime/src/components.rs", "crates/nuxie-graph/src/lib.rs"]
verdict: ADAPTED
axes:
  retained_identity: {status: "adapted-arena-identity", evidence: ["cpp@d788e8ec:src/layout/axis_x.cpp:7-15", "crates/nuxie-graph/src/lib.rs:607-817", "crates/nuxie-runtime/src/artboard.rs:228-320"]}
  push_vs_poll: {status: "isomorphic", cpp_pushes: false, evidence: ["cpp@d788e8ec:src/layout/axis_x.cpp:7-15", "crates/nuxie-runtime/src/draw.rs:20201-21533"]}
  update_ordering: {status: "isomorphic", phases_cpp: "import/build or direct value operation", phases_rust: "graph build or direct value operation"}
  ownership: {status: "adapted", evidence: ["cpp@d788e8ec:src/layout/axis_x.cpp:7-15", "crates/nuxie-graph/src/lib.rs:607-817"]}
  compensation:
    status: "clear"
    mechanisms: []
    import_time_constants: [{name: "precomputed graph membership/type descriptor", idiom_rule: "AF-5 import-time devirtualization", evidence: ["crates/nuxie-graph/src/lib.rs:607-817"]}]
idiom_rules_invoked: ["AF-5 import-time devirtualization", "AF-1 arena-id retained identity"]
confidence: high
notes: "Axis-X membership is precomputed into NSlicerDetailsNode.x_axes at graph build. Coverage grep: generation|epoch|revision|dirty|observed|snapshot|candidate|alias plus mesh/effect/path family names across crates/nuxie-runtime/src and crates/nuxie-graph/src; sibling sweep found the off-file members named above."

## B6-0251

row_id: B6-0251
cpp_files: ["src/layout/axis_y.cpp"]
rust_module: "crates/nuxie-runtime/src/draw.rs"
subsystem_cluster: "layout-shapes-paint"
sibling_files_swept: ["crates/nuxie-runtime/src/artboard.rs", "crates/nuxie-runtime/src/components.rs", "crates/nuxie-graph/src/lib.rs"]
verdict: ADAPTED
axes:
  retained_identity: {status: "adapted-arena-identity", evidence: ["cpp@d788e8ec:src/layout/axis_y.cpp:7-16", "crates/nuxie-graph/src/lib.rs:607-817", "crates/nuxie-runtime/src/artboard.rs:228-320"]}
  push_vs_poll: {status: "isomorphic", cpp_pushes: false, evidence: ["cpp@d788e8ec:src/layout/axis_y.cpp:7-16", "crates/nuxie-runtime/src/draw.rs:20201-21533"]}
  update_ordering: {status: "isomorphic", phases_cpp: "import/build or direct value operation", phases_rust: "graph build or direct value operation"}
  ownership: {status: "adapted", evidence: ["cpp@d788e8ec:src/layout/axis_y.cpp:7-16", "crates/nuxie-graph/src/lib.rs:607-817"]}
  compensation:
    status: "clear"
    mechanisms: []
    import_time_constants: [{name: "precomputed graph membership/type descriptor", idiom_rule: "AF-5 import-time devirtualization", evidence: ["crates/nuxie-graph/src/lib.rs:607-817"]}]
idiom_rules_invoked: ["AF-5 import-time devirtualization", "AF-1 arena-id retained identity"]
confidence: high
notes: "Axis-Y membership is precomputed into NSlicerDetailsNode.y_axes at graph build. Coverage grep: generation|epoch|revision|dirty|observed|snapshot|candidate|alias plus mesh/effect/path family names across crates/nuxie-runtime/src and crates/nuxie-graph/src; sibling sweep found the off-file members named above."

## B6-0252

row_id: B6-0252
cpp_files: ["src/layout/layout_component_style.cpp"]
rust_module: "crates/nuxie-runtime/src/draw.rs"
subsystem_cluster: "layout-shapes-paint"
sibling_files_swept: ["crates/nuxie-runtime/src/artboard.rs", "crates/nuxie-runtime/src/components.rs", "crates/nuxie-graph/src/lib.rs"]
verdict: DIVERGENT
axes:
  retained_identity: {status: "adapted-arena-identity", evidence: ["cpp@d788e8ec:src/layout/layout_component_style.cpp:208-221,289-452", "crates/nuxie-graph/src/lib.rs:607-817", "crates/nuxie-runtime/src/artboard.rs:228-320"]}
  push_vs_poll: {status: "isomorphic push; extra drift tracking counted under compensation", cpp_pushes: true, evidence: ["cpp@d788e8ec:src/layout/layout_component_style.cpp:208-221,289-452", "crates/nuxie-runtime/src/artboard.rs:3783-3785,3948-3966; crates/nuxie-runtime/src/draw.rs:10892-10910"]}
  update_ordering: {status: "phase-sequence-equivalent; representation divergent", phases_cpp: "property change -> ComponentDirt/dependent cascade -> component update -> draw", phases_rust: "property write -> dirt cascade -> epoch/snapshot write -> cache preparation/rebuild -> draw"}
  ownership: {status: "isomorphic-or-arena-adapted", evidence: ["cpp@d788e8ec:src/layout/layout_component_style.cpp:208-221,289-452", "crates/nuxie-graph/src/lib.rs:607-817"]}
  compensation:
    status: "present"
    mechanisms: [{name: "layout epoch fanout", kind: "cross-file drift tracker", mutation_gated: true, cpp_counterpart: "none", evidence: ["crates/nuxie-runtime/src/artboard.rs:3783-3785,3948-3966; crates/nuxie-runtime/src/draw.rs:10892-10910"]}]
    import_time_constants: []
idiom_rules_invoked: ["AF-1 retained identity", "AF-2 push never reconstruct", "AF-4 one dirt model", "AF-8 no invented lifecycles"]
confidence: high
notes: "C++ marks the owning layout node/style dirty; Rust additionally mutates layout/prepared/command epochs consumed by layout-cache keys. Coverage grep: generation|epoch|revision|dirty|observed|snapshot|candidate|alias plus mesh/effect/path family names across crates/nuxie-runtime/src and crates/nuxie-graph/src; sibling sweep found the off-file members named above."

## B6-0253

row_id: B6-0253
cpp_files: ["src/layout/layout_node_provider.cpp"]
rust_module: "crates/nuxie-runtime/src/draw.rs"
subsystem_cluster: "layout-shapes-paint"
sibling_files_swept: ["crates/nuxie-runtime/src/artboard.rs", "crates/nuxie-runtime/src/components.rs", "crates/nuxie-graph/src/lib.rs"]
verdict: ADAPTED
axes:
  retained_identity: {status: "adapted-arena-identity", evidence: ["cpp@d788e8ec:src/layout/layout_node_provider.cpp:10-30", "crates/nuxie-graph/src/lib.rs:607-817", "crates/nuxie-runtime/src/artboard.rs:228-320"]}
  push_vs_poll: {status: "isomorphic", cpp_pushes: false, evidence: ["cpp@d788e8ec:src/layout/layout_node_provider.cpp:10-30", "crates/nuxie-runtime/src/draw.rs:20201-21533"]}
  update_ordering: {status: "isomorphic", phases_cpp: "import/build or direct value operation", phases_rust: "graph build or direct value operation"}
  ownership: {status: "adapted", evidence: ["cpp@d788e8ec:src/layout/layout_node_provider.cpp:10-30", "crates/nuxie-graph/src/lib.rs:607-817"]}
  compensation:
    status: "clear"
    mechanisms: []
    import_time_constants: [{name: "precomputed graph membership/type descriptor", idiom_rule: "AF-5 import-time devirtualization", evidence: ["crates/nuxie-graph/src/lib.rs:607-817"]}]
idiom_rules_invoked: ["AF-5 import-time devirtualization", "AF-1 arena-id retained identity"]
confidence: high
notes: "The closed type switch and constraint membership are precomputed into graph/layout indexes. Coverage grep: generation|epoch|revision|dirty|observed|snapshot|candidate|alias plus mesh/effect/path family names across crates/nuxie-runtime/src and crates/nuxie-graph/src; sibling sweep found the off-file members named above."

## B6-0254

row_id: B6-0254
cpp_files: ["src/layout/n_sliced_node.cpp"]
rust_module: "crates/nuxie-runtime/src/draw.rs"
subsystem_cluster: "layout-shapes-paint"
sibling_files_swept: ["crates/nuxie-runtime/src/artboard.rs", "crates/nuxie-runtime/src/components.rs", "crates/nuxie-graph/src/lib.rs"]
verdict: DIVERGENT
axes:
  retained_identity: {status: "adapted-arena-identity", evidence: ["cpp@d788e8ec:src/layout/n_sliced_node.cpp:8-25,38-48,160-168", "crates/nuxie-graph/src/lib.rs:607-817", "crates/nuxie-runtime/src/artboard.rs:228-320"]}
  push_vs_poll: {status: "isomorphic push; extra drift tracking counted under compensation", cpp_pushes: true, evidence: ["cpp@d788e8ec:src/layout/n_sliced_node.cpp:8-25,38-48,160-168", "crates/nuxie-runtime/src/artboard.rs:3783-3790,3948-3966; crates/nuxie-runtime/src/draw.rs:10965-10975"]}
  update_ordering: {status: "phase-sequence-equivalent; representation divergent", phases_cpp: "property change -> ComponentDirt/dependent cascade -> component update -> draw", phases_rust: "property write -> dirt cascade -> epoch/snapshot write -> cache preparation/rebuild -> draw"}
  ownership: {status: "isomorphic-or-arena-adapted", evidence: ["cpp@d788e8ec:src/layout/n_sliced_node.cpp:8-25,38-48,160-168", "crates/nuxie-graph/src/lib.rs:607-817"]}
  compensation:
    status: "present"
    mechanisms: [{name: "path/layout epoch fanout", kind: "cross-file drift tracker", mutation_gated: true, cpp_counterpart: "none", evidence: ["crates/nuxie-runtime/src/artboard.rs:3783-3790,3948-3966; crates/nuxie-runtime/src/draw.rs:10965-10975"]}]
    import_time_constants: [{name: "RuntimePathComposerLookupCacheKey.graph_identity", idiom_rule: "AF-5 import-time devirtualization", evidence: ["crates/nuxie-runtime/src/draw.rs:10920-10949"]}]
idiom_rules_invoked: ["AF-1 retained identity", "AF-2 push never reconstruct", "AF-4 one dirt model", "AF-8 no invented lifecycles"]
confidence: high
notes: "C++ pushes NSlicer dirt recursively and updates its retained mapper; Rust also records path/layout epochs for later cache-key comparison. Coverage grep: generation|epoch|revision|dirty|observed|snapshot|candidate|alias plus mesh/effect/path family names across crates/nuxie-runtime/src and crates/nuxie-graph/src; sibling sweep found the off-file members named above."

## B6-0255

row_id: B6-0255
cpp_files: ["src/layout/n_slicer.cpp"]
rust_module: "crates/nuxie-runtime/src/draw.rs"
subsystem_cluster: "layout-shapes-paint"
sibling_files_swept: ["crates/nuxie-runtime/src/artboard.rs", "crates/nuxie-runtime/src/components.rs", "crates/nuxie-graph/src/lib.rs"]
verdict: DIVERGENT
axes:
  retained_identity: {status: "adapted-arena-identity", evidence: ["cpp@d788e8ec:src/layout/n_slicer.cpp:9-10,37-55", "crates/nuxie-graph/src/lib.rs:607-817", "crates/nuxie-runtime/src/artboard.rs:228-320"]}
  push_vs_poll: {status: "isomorphic push; extra drift tracking counted under compensation", cpp_pushes: true, evidence: ["cpp@d788e8ec:src/layout/n_slicer.cpp:9-10,37-55", "crates/nuxie-runtime/src/draw.rs:9418-9447,13524-13558"]}
  update_ordering: {status: "phase-sequence-equivalent; representation divergent", phases_cpp: "property change -> ComponentDirt/dependent cascade -> component update -> draw", phases_rust: "property write -> dirt cascade -> epoch/snapshot write -> cache preparation/rebuild -> draw"}
  ownership: {status: "isomorphic-or-arena-adapted", evidence: ["cpp@d788e8ec:src/layout/n_slicer.cpp:9-10,37-55", "crates/nuxie-graph/src/lib.rs:607-817"]}
  compensation:
    status: "present"
    mechanisms: [{name: "slice-mesh input snapshot", kind: "cross-file drift tracker", mutation_gated: true, cpp_counterpart: "none", evidence: ["crates/nuxie-runtime/src/draw.rs:9418-9447,13524-13558"]}]
    import_time_constants: []
idiom_rules_invoked: ["AF-1 retained identity", "AF-2 push never reconstruct", "AF-4 one dirt model", "AF-8 no invented lifecycles"]
confidence: high
notes: "C++ owns SliceMesh by unique_ptr and updates it from pushed NSlicer/world dirt; Rust compares and stores complete mesh inputs before rebuilding. Coverage grep: generation|epoch|revision|dirty|observed|snapshot|candidate|alias plus mesh/effect/path family names across crates/nuxie-runtime/src and crates/nuxie-graph/src; sibling sweep found the off-file members named above."

## B6-0256

row_id: B6-0256
cpp_files: ["src/layout/n_slicer_details.cpp"]
rust_module: "crates/nuxie-runtime/src/draw.rs"
subsystem_cluster: "layout-shapes-paint"
sibling_files_swept: ["crates/nuxie-runtime/src/artboard.rs", "crates/nuxie-runtime/src/components.rs", "crates/nuxie-graph/src/lib.rs"]
verdict: ADAPTED
axes:
  retained_identity: {status: "adapted-arena-identity", evidence: ["cpp@d788e8ec:src/layout/n_slicer_details.cpp:9-33", "crates/nuxie-graph/src/lib.rs:607-817", "crates/nuxie-runtime/src/artboard.rs:228-320"]}
  push_vs_poll: {status: "isomorphic", cpp_pushes: false, evidence: ["cpp@d788e8ec:src/layout/n_slicer_details.cpp:9-33", "crates/nuxie-runtime/src/draw.rs:20201-21533"]}
  update_ordering: {status: "isomorphic", phases_cpp: "import/build or direct value operation", phases_rust: "graph build or direct value operation"}
  ownership: {status: "adapted", evidence: ["cpp@d788e8ec:src/layout/n_slicer_details.cpp:9-33", "crates/nuxie-graph/src/lib.rs:607-817"]}
  compensation:
    status: "clear"
    mechanisms: []
    import_time_constants: [{name: "precomputed graph membership/type descriptor", idiom_rule: "AF-5 import-time devirtualization", evidence: ["crates/nuxie-graph/src/lib.rs:607-817"]}]
idiom_rules_invoked: ["AF-5 import-time devirtualization", "AF-1 arena-id retained identity"]
confidence: high
notes: "Axis and tile membership is assembled once into by-value graph vectors; live values remain addressed by arena local_id. Coverage grep: generation|epoch|revision|dirty|observed|snapshot|candidate|alias plus mesh/effect/path family names across crates/nuxie-runtime/src and crates/nuxie-graph/src; sibling sweep found the off-file members named above."

## B6-0257

row_id: B6-0257
cpp_files: ["src/layout/n_slicer_tile_mode.cpp"]
rust_module: "crates/nuxie-runtime/src/draw.rs"
subsystem_cluster: "layout-shapes-paint"
sibling_files_swept: ["crates/nuxie-runtime/src/artboard.rs", "crates/nuxie-runtime/src/components.rs", "crates/nuxie-graph/src/lib.rs"]
verdict: ADAPTED
axes:
  retained_identity: {status: "adapted-arena-identity", evidence: ["cpp@d788e8ec:src/layout/n_slicer_tile_mode.cpp:7-22", "crates/nuxie-graph/src/lib.rs:607-817", "crates/nuxie-runtime/src/artboard.rs:228-320"]}
  push_vs_poll: {status: "isomorphic", cpp_pushes: false, evidence: ["cpp@d788e8ec:src/layout/n_slicer_tile_mode.cpp:7-22", "crates/nuxie-runtime/src/draw.rs:20201-21533"]}
  update_ordering: {status: "isomorphic", phases_cpp: "import/build or direct value operation", phases_rust: "graph build or direct value operation"}
  ownership: {status: "adapted", evidence: ["cpp@d788e8ec:src/layout/n_slicer_tile_mode.cpp:7-22", "crates/nuxie-graph/src/lib.rs:607-817"]}
  compensation:
    status: "clear"
    mechanisms: []
    import_time_constants: [{name: "precomputed graph membership/type descriptor", idiom_rule: "AF-5 import-time devirtualization", evidence: ["crates/nuxie-graph/src/lib.rs:607-817"]}]
idiom_rules_invoked: ["AF-5 import-time devirtualization", "AF-1 arena-id retained identity"]
confidence: high
notes: "The validated parent/type and patch-style tuple are materialized at graph build. Coverage grep: generation|epoch|revision|dirty|observed|snapshot|candidate|alias plus mesh/effect/path family names across crates/nuxie-runtime/src and crates/nuxie-graph/src; sibling sweep found the off-file members named above."

## B6-0331

row_id: B6-0331
cpp_files: ["src/shapes/clipping_shape.cpp"]
rust_module: "crates/nuxie-runtime/src/draw.rs"
subsystem_cluster: "layout-shapes-paint"
sibling_files_swept: ["crates/nuxie-runtime/src/artboard.rs", "crates/nuxie-runtime/src/components.rs", "crates/nuxie-graph/src/lib.rs"]
verdict: DIVERGENT
axes:
  retained_identity: {status: "adapted-arena-identity", evidence: ["cpp@d788e8ec:src/shapes/clipping_shape.cpp:91-117,140-177", "crates/nuxie-graph/src/lib.rs:607-817", "crates/nuxie-runtime/src/artboard.rs:228-320"]}
  push_vs_poll: {status: "isomorphic push; extra drift tracking counted under compensation", cpp_pushes: true, evidence: ["cpp@d788e8ec:src/shapes/clipping_shape.cpp:91-117,140-177", "crates/nuxie-runtime/src/artboard.rs:3788-3790,3948-3966; crates/nuxie-runtime/src/draw.rs:11139-11147"]}
  update_ordering: {status: "phase-sequence-equivalent; representation divergent", phases_cpp: "property change -> ComponentDirt/dependent cascade -> component update -> draw", phases_rust: "property write -> dirt cascade -> epoch/snapshot write -> cache preparation/rebuild -> draw"}
  ownership: {status: "isomorphic-or-arena-adapted", evidence: ["cpp@d788e8ec:src/shapes/clipping_shape.cpp:91-117,140-177", "crates/nuxie-graph/src/lib.rs:607-817"]}
  compensation:
    status: "present"
    mechanisms: [{name: "clipping path epoch cache", kind: "cross-file drift tracker", mutation_gated: true, cpp_counterpart: "none", evidence: ["crates/nuxie-runtime/src/artboard.rs:3788-3790,3948-3966; crates/nuxie-runtime/src/draw.rs:11139-11147"]}]
    import_time_constants: [{name: "RuntimePathComposerLookupCacheKey.graph_identity", idiom_rule: "AF-5 import-time devirtualization", evidence: ["crates/nuxie-runtime/src/draw.rs:10920-10949"]}]
idiom_rules_invoked: ["AF-1 retained identity", "AF-2 push never reconstruct", "AF-4 one dirt model", "AF-8 no invented lifecycles"]
confidence: high
notes: "C++ registers path-composer dependents and rebuilds a retained clip path on dirt; Rust gates copied clip commands on global path/layout/world epochs. Coverage grep: generation|epoch|revision|dirty|observed|snapshot|candidate|alias plus mesh/effect/path family names across crates/nuxie-runtime/src and crates/nuxie-graph/src; sibling sweep found the off-file members named above."

## B6-0332

row_id: B6-0332
cpp_files: ["src/shapes/cubic_asymmetric_vertex.cpp"]
rust_module: "crates/nuxie-runtime/src/draw.rs"
subsystem_cluster: "layout-shapes-paint"
sibling_files_swept: ["crates/nuxie-runtime/src/artboard.rs", "crates/nuxie-runtime/src/components.rs", "crates/nuxie-graph/src/lib.rs"]
verdict: DIVERGENT
axes:
  retained_identity: {status: "adapted-arena-identity", evidence: ["cpp@d788e8ec:src/shapes/cubic_asymmetric_vertex.cpp:24-48", "crates/nuxie-graph/src/lib.rs:607-817", "crates/nuxie-runtime/src/artboard.rs:228-320"]}
  push_vs_poll: {status: "isomorphic push; extra drift tracking counted under compensation", cpp_pushes: true, evidence: ["cpp@d788e8ec:src/shapes/cubic_asymmetric_vertex.cpp:24-48", "crates/nuxie-runtime/src/artboard.rs:3788-3790,3948-3966; crates/nuxie-runtime/src/draw.rs:10965-10975"]}
  update_ordering: {status: "phase-sequence-equivalent; representation divergent", phases_cpp: "property change -> ComponentDirt/dependent cascade -> component update -> draw", phases_rust: "property write -> dirt cascade -> epoch/snapshot write -> cache preparation/rebuild -> draw"}
  ownership: {status: "isomorphic-or-arena-adapted", evidence: ["cpp@d788e8ec:src/shapes/cubic_asymmetric_vertex.cpp:24-48", "crates/nuxie-graph/src/lib.rs:607-817"]}
  compensation:
    status: "present"
    mechanisms: [{name: "path epoch fanout", kind: "cross-file drift tracker", mutation_gated: true, cpp_counterpart: "none", evidence: ["crates/nuxie-runtime/src/artboard.rs:3788-3790,3948-3966; crates/nuxie-runtime/src/draw.rs:10965-10975"]}]
    import_time_constants: [{name: "RuntimePathComposerLookupCacheKey.graph_identity", idiom_rule: "AF-5 import-time devirtualization", evidence: ["crates/nuxie-runtime/src/draw.rs:10920-10949"]}]
idiom_rules_invoked: ["AF-1 retained identity", "AF-2 push never reconstruct", "AF-4 one dirt model", "AF-8 no invented lifecycles"]
confidence: high
notes: "C++ invalidates per-vertex in/out caches and pushes geometry dirt; Rust relies on instance path epochs to rebuild copied path commands. Coverage grep: generation|epoch|revision|dirty|observed|snapshot|candidate|alias plus mesh/effect/path family names across crates/nuxie-runtime/src and crates/nuxie-graph/src; sibling sweep found the off-file members named above."

## B6-0333

row_id: B6-0333
cpp_files: ["src/shapes/cubic_detached_vertex.cpp"]
rust_module: "crates/nuxie-runtime/src/draw.rs"
subsystem_cluster: "layout-shapes-paint"
sibling_files_swept: ["crates/nuxie-runtime/src/artboard.rs", "crates/nuxie-runtime/src/components.rs", "crates/nuxie-graph/src/lib.rs"]
verdict: DIVERGENT
axes:
  retained_identity: {status: "adapted-arena-identity", evidence: ["cpp@d788e8ec:src/shapes/cubic_detached_vertex.cpp:24-52", "crates/nuxie-graph/src/lib.rs:607-817", "crates/nuxie-runtime/src/artboard.rs:228-320"]}
  push_vs_poll: {status: "isomorphic push; extra drift tracking counted under compensation", cpp_pushes: true, evidence: ["cpp@d788e8ec:src/shapes/cubic_detached_vertex.cpp:24-52", "crates/nuxie-runtime/src/artboard.rs:3788-3790,3948-3966; crates/nuxie-runtime/src/draw.rs:10965-10975"]}
  update_ordering: {status: "phase-sequence-equivalent; representation divergent", phases_cpp: "property change -> ComponentDirt/dependent cascade -> component update -> draw", phases_rust: "property write -> dirt cascade -> epoch/snapshot write -> cache preparation/rebuild -> draw"}
  ownership: {status: "isomorphic-or-arena-adapted", evidence: ["cpp@d788e8ec:src/shapes/cubic_detached_vertex.cpp:24-52", "crates/nuxie-graph/src/lib.rs:607-817"]}
  compensation:
    status: "present"
    mechanisms: [{name: "path epoch fanout", kind: "cross-file drift tracker", mutation_gated: true, cpp_counterpart: "none", evidence: ["crates/nuxie-runtime/src/artboard.rs:3788-3790,3948-3966; crates/nuxie-runtime/src/draw.rs:10965-10975"]}]
    import_time_constants: [{name: "RuntimePathComposerLookupCacheKey.graph_identity", idiom_rule: "AF-5 import-time devirtualization", evidence: ["crates/nuxie-runtime/src/draw.rs:10920-10949"]}]
idiom_rules_invoked: ["AF-1 retained identity", "AF-2 push never reconstruct", "AF-4 one dirt model", "AF-8 no invented lifecycles"]
confidence: high
notes: "C++ invalidates retained vertex caches and pushes geometry dirt; Rust rebuilds the copied command representation behind path epochs. Coverage grep: generation|epoch|revision|dirty|observed|snapshot|candidate|alias plus mesh/effect/path family names across crates/nuxie-runtime/src and crates/nuxie-graph/src; sibling sweep found the off-file members named above."

## B6-0334

row_id: B6-0334
cpp_files: ["src/shapes/cubic_mirrored_vertex.cpp"]
rust_module: "crates/nuxie-runtime/src/draw.rs"
subsystem_cluster: "layout-shapes-paint"
sibling_files_swept: ["crates/nuxie-runtime/src/artboard.rs", "crates/nuxie-runtime/src/components.rs", "crates/nuxie-graph/src/lib.rs"]
verdict: DIVERGENT
axes:
  retained_identity: {status: "adapted-arena-identity", evidence: ["cpp@d788e8ec:src/shapes/cubic_mirrored_vertex.cpp:18-36", "crates/nuxie-graph/src/lib.rs:607-817", "crates/nuxie-runtime/src/artboard.rs:228-320"]}
  push_vs_poll: {status: "isomorphic push; extra drift tracking counted under compensation", cpp_pushes: true, evidence: ["cpp@d788e8ec:src/shapes/cubic_mirrored_vertex.cpp:18-36", "crates/nuxie-runtime/src/artboard.rs:3788-3790,3948-3966; crates/nuxie-runtime/src/draw.rs:10965-10975"]}
  update_ordering: {status: "phase-sequence-equivalent; representation divergent", phases_cpp: "property change -> ComponentDirt/dependent cascade -> component update -> draw", phases_rust: "property write -> dirt cascade -> epoch/snapshot write -> cache preparation/rebuild -> draw"}
  ownership: {status: "isomorphic-or-arena-adapted", evidence: ["cpp@d788e8ec:src/shapes/cubic_mirrored_vertex.cpp:18-36", "crates/nuxie-graph/src/lib.rs:607-817"]}
  compensation:
    status: "present"
    mechanisms: [{name: "path epoch fanout", kind: "cross-file drift tracker", mutation_gated: true, cpp_counterpart: "none", evidence: ["crates/nuxie-runtime/src/artboard.rs:3788-3790,3948-3966; crates/nuxie-runtime/src/draw.rs:10965-10975"]}]
    import_time_constants: [{name: "RuntimePathComposerLookupCacheKey.graph_identity", idiom_rule: "AF-5 import-time devirtualization", evidence: ["crates/nuxie-runtime/src/draw.rs:10920-10949"]}]
idiom_rules_invoked: ["AF-1 retained identity", "AF-2 push never reconstruct", "AF-4 one dirt model", "AF-8 no invented lifecycles"]
confidence: high
notes: "C++ invalidates retained in/out points locally; Rust uses the subsystem path epoch to refresh command copies. Coverage grep: generation|epoch|revision|dirty|observed|snapshot|candidate|alias plus mesh/effect/path family names across crates/nuxie-runtime/src and crates/nuxie-graph/src; sibling sweep found the off-file members named above."

## B6-0335

row_id: B6-0335
cpp_files: ["src/shapes/cubic_vertex.cpp"]
rust_module: "crates/nuxie-runtime/src/draw.rs"
subsystem_cluster: "layout-shapes-paint"
sibling_files_swept: ["crates/nuxie-runtime/src/artboard.rs", "crates/nuxie-runtime/src/components.rs", "crates/nuxie-graph/src/lib.rs"]
verdict: DIVERGENT
axes:
  retained_identity: {status: "adapted-arena-identity", evidence: ["cpp@d788e8ec:src/shapes/cubic_vertex.cpp:29-69,72-90", "crates/nuxie-graph/src/lib.rs:607-817", "crates/nuxie-runtime/src/artboard.rs:228-320"]}
  push_vs_poll: {status: "isomorphic push; extra drift tracking counted under compensation", cpp_pushes: true, evidence: ["cpp@d788e8ec:src/shapes/cubic_vertex.cpp:29-69,72-90", "crates/nuxie-runtime/src/artboard.rs:3788-3790,3948-3966; crates/nuxie-runtime/src/draw.rs:10965-10975"]}
  update_ordering: {status: "phase-sequence-equivalent; representation divergent", phases_cpp: "property change -> ComponentDirt/dependent cascade -> component update -> draw", phases_rust: "property write -> dirt cascade -> epoch/snapshot write -> cache preparation/rebuild -> draw"}
  ownership: {status: "isomorphic-or-arena-adapted", evidence: ["cpp@d788e8ec:src/shapes/cubic_vertex.cpp:29-69,72-90", "crates/nuxie-graph/src/lib.rs:607-817"]}
  compensation:
    status: "present"
    mechanisms: [{name: "path epoch fanout", kind: "cross-file drift tracker", mutation_gated: true, cpp_counterpart: "none", evidence: ["crates/nuxie-runtime/src/artboard.rs:3788-3790,3948-3966; crates/nuxie-runtime/src/draw.rs:10965-10975"]}]
    import_time_constants: [{name: "RuntimePathComposerLookupCacheKey.graph_identity", idiom_rule: "AF-5 import-time devirtualization", evidence: ["crates/nuxie-runtime/src/draw.rs:10920-10949"]}]
idiom_rules_invoked: ["AF-1 retained identity", "AF-2 push never reconstruct", "AF-4 one dirt model", "AF-8 no invented lifecycles"]
confidence: high
notes: "C++ lazily retains computed control points and explicitly invalidates them; Rust reconstructs them under a path-epoch cache boundary. Coverage grep: generation|epoch|revision|dirty|observed|snapshot|candidate|alias plus mesh/effect/path family names across crates/nuxie-runtime/src and crates/nuxie-graph/src; sibling sweep found the off-file members named above."

## B6-0336

row_id: B6-0336
cpp_files: ["src/shapes/deformer.cpp"]
rust_module: "crates/nuxie-runtime/src/draw.rs"
subsystem_cluster: "layout-shapes-paint"
sibling_files_swept: ["crates/nuxie-runtime/src/artboard.rs", "crates/nuxie-runtime/src/components.rs", "crates/nuxie-graph/src/lib.rs"]
verdict: ISOMORPHIC
axes:
  retained_identity: {status: "isomorphic", evidence: ["cpp@d788e8ec:src/shapes/deformer.cpp:7-24", "crates/nuxie-graph/src/lib.rs:607-817", "crates/nuxie-runtime/src/artboard.rs:228-320"]}
  push_vs_poll: {status: "isomorphic", cpp_pushes: false, evidence: ["cpp@d788e8ec:src/shapes/deformer.cpp:7-24", "crates/nuxie-runtime/src/draw.rs:20201-21533"]}
  update_ordering: {status: "isomorphic", phases_cpp: "import/build or direct value operation", phases_rust: "graph build or direct value operation"}
  ownership: {status: "isomorphic-or-arena-adapted", evidence: ["cpp@d788e8ec:src/shapes/deformer.cpp:7-24", "crates/nuxie-graph/src/lib.rs:607-817"]}
  compensation:
    status: "clear"
    mechanisms: []
    import_time_constants: []
idiom_rules_invoked: []
confidence: high
notes: "Both sides perform closed-type selection of NSlicedNode deformation with no observer or runtime compensation state. Coverage grep: generation|epoch|revision|dirty|observed|snapshot|candidate|alias plus mesh/effect/path family names across crates/nuxie-runtime/src and crates/nuxie-graph/src; sibling sweep found the off-file members named above."

## B6-0337

row_id: B6-0337
cpp_files: ["src/shapes/ellipse.cpp"]
rust_module: "crates/nuxie-runtime/src/draw.rs"
subsystem_cluster: "layout-shapes-paint"
sibling_files_swept: ["crates/nuxie-runtime/src/artboard.rs", "crates/nuxie-runtime/src/components.rs", "crates/nuxie-graph/src/lib.rs"]
verdict: DIVERGENT
axes:
  retained_identity: {status: "adapted-arena-identity", evidence: ["cpp@d788e8ec:src/shapes/ellipse.cpp:7-46", "crates/nuxie-graph/src/lib.rs:607-817", "crates/nuxie-runtime/src/artboard.rs:228-320"]}
  push_vs_poll: {status: "isomorphic push; extra drift tracking counted under compensation", cpp_pushes: true, evidence: ["cpp@d788e8ec:src/shapes/ellipse.cpp:7-46", "crates/nuxie-runtime/src/artboard.rs:3788-3790,3948-3966; crates/nuxie-runtime/src/draw.rs:10965-10975"]}
  update_ordering: {status: "phase-sequence-equivalent; representation divergent", phases_cpp: "property change -> ComponentDirt/dependent cascade -> component update -> draw", phases_rust: "property write -> dirt cascade -> epoch/snapshot write -> cache preparation/rebuild -> draw"}
  ownership: {status: "isomorphic-or-arena-adapted", evidence: ["cpp@d788e8ec:src/shapes/ellipse.cpp:7-46", "crates/nuxie-graph/src/lib.rs:607-817"]}
  compensation:
    status: "present"
    mechanisms: [{name: "path epoch fanout", kind: "cross-file drift tracker", mutation_gated: true, cpp_counterpart: "none", evidence: ["crates/nuxie-runtime/src/artboard.rs:3788-3790,3948-3966; crates/nuxie-runtime/src/draw.rs:10965-10975"]}]
    import_time_constants: [{name: "RuntimePathComposerLookupCacheKey.graph_identity", idiom_rule: "AF-5 import-time devirtualization", evidence: ["crates/nuxie-runtime/src/draw.rs:10920-10949"]}]
idiom_rules_invoked: ["AF-1 retained identity", "AF-2 push never reconstruct", "AF-4 one dirt model", "AF-8 no invented lifecycles"]
confidence: high
notes: "C++ updates four retained vertices on Path dirt; Rust regenerates command values behind a global path epoch. Coverage grep: generation|epoch|revision|dirty|observed|snapshot|candidate|alias plus mesh/effect/path family names across crates/nuxie-runtime/src and crates/nuxie-graph/src; sibling sweep found the off-file members named above."

## B6-0338

row_id: B6-0338
cpp_files: ["src/shapes/image.cpp"]
rust_module: "crates/nuxie-runtime/src/draw.rs"
subsystem_cluster: "layout-shapes-paint"
sibling_files_swept: ["crates/nuxie-runtime/src/artboard.rs", "crates/nuxie-runtime/src/components.rs", "crates/nuxie-graph/src/lib.rs"]
verdict: DIVERGENT
axes:
  retained_identity: {status: "adapted-arena-identity", evidence: ["cpp@d788e8ec:src/shapes/image.cpp:114-155,222-247,249-367", "crates/nuxie-graph/src/lib.rs:607-817", "crates/nuxie-runtime/src/artboard.rs:228-320"]}
  push_vs_poll: {status: "isomorphic push; extra drift tracking counted under compensation", cpp_pushes: true, evidence: ["cpp@d788e8ec:src/shapes/image.cpp:114-155,222-247,249-367", "crates/nuxie-runtime/src/artboard.rs:3728-3755; crates/nuxie-runtime/src/draw.rs:10059-10124"]}
  update_ordering: {status: "phase-sequence-equivalent; representation divergent", phases_cpp: "property change -> ComponentDirt/dependent cascade -> component update -> draw", phases_rust: "property write -> dirt cascade -> epoch/snapshot write -> cache preparation/rebuild -> draw"}
  ownership: {status: "isomorphic-or-arena-adapted", evidence: ["cpp@d788e8ec:src/shapes/image.cpp:114-155,222-247,249-367", "crates/nuxie-graph/src/lib.rs:607-817"]}
  compensation:
    status: "present"
    mechanisms: [{name: "image cache/prepared epoch fanout", kind: "cross-file drift tracker", mutation_gated: true, cpp_counterpart: "none", evidence: ["crates/nuxie-runtime/src/artboard.rs:3728-3755; crates/nuxie-runtime/src/draw.rs:10059-10124"]}]
    import_time_constants: []
idiom_rules_invoked: ["AF-1 retained identity", "AF-2 push never reconstruct", "AF-4 one dirt model", "AF-8 no invented lifecycles"]
confidence: high
notes: "C++ retains asset/mesh pointers and layout-fit state and pushes transform dirt; Rust mutates cache/prepared epochs consumed by image-layout transform slots. Coverage grep: generation|epoch|revision|dirty|observed|snapshot|candidate|alias plus mesh/effect/path family names across crates/nuxie-runtime/src and crates/nuxie-graph/src; sibling sweep found the off-file members named above."

## B6-0339

row_id: B6-0339
cpp_files: ["src/shapes/list_path.cpp"]
rust_module: "crates/nuxie-runtime/src/draw.rs"
subsystem_cluster: "layout-shapes-paint"
sibling_files_swept: ["crates/nuxie-runtime/src/artboard.rs", "crates/nuxie-runtime/src/components.rs", "crates/nuxie-graph/src/lib.rs"]
verdict: UNKNOWN
axes:
  retained_identity: {status: "unknown", evidence: ["cpp@d788e8ec:src/shapes/list_path.cpp:15-323", "crates/nuxie-graph/src/lib.rs:607-817", "crates/nuxie-runtime/src/artboard.rs:228-320"]}
  push_vs_poll: {status: "unknown", cpp_pushes: false, evidence: ["cpp@d788e8ec:src/shapes/list_path.cpp:15-323", "crates/nuxie-runtime/src/draw.rs:20201-21533"]}
  update_ordering: {status: "unknown", phases_cpp: "unmapped", phases_rust: "unmapped"}
  ownership: {status: "unknown", evidence: ["cpp@d788e8ec:src/shapes/list_path.cpp:15-323", "crates/nuxie-graph/src/lib.rs:607-817"]}
  compensation:
    status: "unknown"
    mechanisms: []
    import_time_constants: []
idiom_rules_invoked: []
confidence: low
notes: "Blocker: manifest says partial, but no ListPath/VertexListener implementation or mapped region exists in crates/nuxie-runtime/src/draw.rs; current Rust matches are unrelated view-model list-path APIs. Coverage grep: generation|epoch|revision|dirty|observed|snapshot|candidate|alias plus mesh/effect/path family names across crates/nuxie-runtime/src and crates/nuxie-graph/src; sibling sweep found the off-file members named above."

## B6-0340

row_id: B6-0340
cpp_files: ["src/shapes/mesh.cpp"]
rust_module: "crates/nuxie-runtime/src/draw.rs"
subsystem_cluster: "layout-shapes-paint"
sibling_files_swept: ["crates/nuxie-runtime/src/artboard.rs", "crates/nuxie-runtime/src/components.rs", "crates/nuxie-graph/src/lib.rs"]
verdict: DIVERGENT
axes:
  retained_identity: {status: "adapted-arena-identity", evidence: ["cpp@d788e8ec:src/shapes/mesh.cpp:14-25,152-194", "crates/nuxie-graph/src/lib.rs:607-817", "crates/nuxie-runtime/src/artboard.rs:228-320"]}
  push_vs_poll: {status: "isomorphic push; extra drift tracking counted under compensation", cpp_pushes: true, evidence: ["cpp@d788e8ec:src/shapes/mesh.cpp:14-25,152-194", "crates/nuxie-runtime/src/draw.rs:9408-9416,14282-14295"]}
  update_ordering: {status: "phase-sequence-equivalent; representation divergent", phases_cpp: "property change -> ComponentDirt/dependent cascade -> component update -> draw", phases_rust: "property write -> dirt cascade -> epoch/snapshot write -> cache preparation/rebuild -> draw"}
  ownership: {status: "isomorphic-or-arena-adapted", evidence: ["cpp@d788e8ec:src/shapes/mesh.cpp:14-25,152-194", "crates/nuxie-graph/src/lib.rs:607-817"]}
  compensation:
    status: "present"
    mechanisms: [{name: "mesh vertex byte snapshot", kind: "cross-file drift tracker", mutation_gated: true, cpp_counterpart: "none", evidence: ["crates/nuxie-runtime/src/draw.rs:9408-9416,14282-14295"]}]
    import_time_constants: []
idiom_rules_invoked: ["AF-1 retained identity", "AF-2 push never reconstruct", "AF-4 one dirt model", "AF-8 no invented lifecycles"]
confidence: high
notes: "C++ registers skin/parent dependents and flips one vertex-buffer dirty bit; Rust recomputes bytes and stores last_vertex_bytes during mesh update preparation. Coverage grep: generation|epoch|revision|dirty|observed|snapshot|candidate|alias plus mesh/effect/path family names across crates/nuxie-runtime/src and crates/nuxie-graph/src; sibling sweep found the off-file members named above."

## B6-0341

row_id: B6-0341
cpp_files: ["src/shapes/mesh_vertex.cpp"]
rust_module: "crates/nuxie-runtime/src/draw.rs"
subsystem_cluster: "layout-shapes-paint"
sibling_files_swept: ["crates/nuxie-runtime/src/artboard.rs", "crates/nuxie-runtime/src/components.rs", "crates/nuxie-graph/src/lib.rs"]
verdict: DIVERGENT
axes:
  retained_identity: {status: "adapted-arena-identity", evidence: ["cpp@d788e8ec:src/shapes/mesh_vertex.cpp:5-22", "crates/nuxie-graph/src/lib.rs:607-817", "crates/nuxie-runtime/src/artboard.rs:228-320"]}
  push_vs_poll: {status: "isomorphic push; extra drift tracking counted under compensation", cpp_pushes: true, evidence: ["cpp@d788e8ec:src/shapes/mesh_vertex.cpp:5-22", "crates/nuxie-runtime/src/draw.rs:9408-9416,14282-14310"]}
  update_ordering: {status: "phase-sequence-equivalent; representation divergent", phases_cpp: "property change -> ComponentDirt/dependent cascade -> component update -> draw", phases_rust: "property write -> dirt cascade -> epoch/snapshot write -> cache preparation/rebuild -> draw"}
  ownership: {status: "isomorphic-or-arena-adapted", evidence: ["cpp@d788e8ec:src/shapes/mesh_vertex.cpp:5-22", "crates/nuxie-graph/src/lib.rs:607-817"]}
  compensation:
    status: "present"
    mechanisms: [{name: "mesh vertex byte snapshot", kind: "cross-file drift tracker", mutation_gated: true, cpp_counterpart: "none", evidence: ["crates/nuxie-runtime/src/draw.rs:9408-9416,14282-14310"]}]
    import_time_constants: []
idiom_rules_invoked: ["AF-1 retained identity", "AF-2 push never reconstruct", "AF-4 one dirt model", "AF-8 no invented lifecycles"]
confidence: high
notes: "C++ pushes vertex dirt directly to its retained Mesh; Rust discovers buffer drift by comparing the rebuilt vertex-byte snapshot. Coverage grep: generation|epoch|revision|dirty|observed|snapshot|candidate|alias plus mesh/effect/path family names across crates/nuxie-runtime/src and crates/nuxie-graph/src; sibling sweep found the off-file members named above."

## B6-0342

row_id: B6-0342
cpp_files: ["src/shapes/paint/color.cpp"]
rust_module: "crates/nuxie-runtime/src/draw.rs"
subsystem_cluster: "layout-shapes-paint"
sibling_files_swept: ["crates/nuxie-runtime/src/artboard.rs", "crates/nuxie-runtime/src/components.rs", "crates/nuxie-graph/src/lib.rs"]
verdict: ISOMORPHIC
axes:
  retained_identity: {status: "isomorphic", evidence: ["cpp@d788e8ec:src/shapes/paint/color.cpp:9-81", "crates/nuxie-graph/src/lib.rs:607-817", "crates/nuxie-runtime/src/artboard.rs:228-320"]}
  push_vs_poll: {status: "isomorphic", cpp_pushes: false, evidence: ["cpp@d788e8ec:src/shapes/paint/color.cpp:9-81", "crates/nuxie-runtime/src/draw.rs:20201-21533"]}
  update_ordering: {status: "isomorphic", phases_cpp: "import/build or direct value operation", phases_rust: "graph build or direct value operation"}
  ownership: {status: "isomorphic-or-arena-adapted", evidence: ["cpp@d788e8ec:src/shapes/paint/color.cpp:9-81", "crates/nuxie-graph/src/lib.rs:607-817"]}
  compensation:
    status: "clear"
    mechanisms: []
    import_time_constants: []
idiom_rules_invoked: []
confidence: high
notes: "The Rust color packing, opacity modulation, and interpolation helpers are stateless value functions; no ownership, observer, ordering, or lifecycle seam exists. Coverage grep: generation|epoch|revision|dirty|observed|snapshot|candidate|alias plus mesh/effect/path family names across crates/nuxie-runtime/src and crates/nuxie-graph/src; sibling sweep found the off-file members named above."

## B6-0343

row_id: B6-0343
cpp_files: ["src/shapes/paint/dash.cpp"]
rust_module: "crates/nuxie-runtime/src/draw.rs"
subsystem_cluster: "layout-shapes-paint"
sibling_files_swept: ["crates/nuxie-runtime/src/artboard.rs", "crates/nuxie-runtime/src/components.rs", "crates/nuxie-graph/src/lib.rs"]
verdict: DIVERGENT
axes:
  retained_identity: {status: "adapted-arena-identity", evidence: ["cpp@d788e8ec:src/shapes/paint/dash.cpp:14-48", "crates/nuxie-graph/src/lib.rs:607-817", "crates/nuxie-runtime/src/artboard.rs:228-320"]}
  push_vs_poll: {status: "isomorphic push; extra drift tracking counted under compensation", cpp_pushes: true, evidence: ["cpp@d788e8ec:src/shapes/paint/dash.cpp:14-48", "crates/nuxie-runtime/src/artboard.rs:2120-2131,3788-3790,6175-6185; crates/nuxie-runtime/src/draw.rs:19153-19299"]}
  update_ordering: {status: "phase-sequence-equivalent; representation divergent", phases_cpp: "property change -> ComponentDirt/dependent cascade -> component update -> draw", phases_rust: "property write -> dirt cascade -> epoch/snapshot write -> cache preparation/rebuild -> draw"}
  ownership: {status: "isomorphic-or-arena-adapted", evidence: ["cpp@d788e8ec:src/shapes/paint/dash.cpp:14-48", "crates/nuxie-graph/src/lib.rs:607-817"]}
  compensation:
    status: "present"
    mechanisms: [{name: "effect path epoch fanout", kind: "cross-file drift tracker", mutation_gated: true, cpp_counterpart: "none", evidence: ["crates/nuxie-runtime/src/artboard.rs:2120-2131,3788-3790,6175-6185; crates/nuxie-runtime/src/draw.rs:19153-19299"]}]
    import_time_constants: [{name: "RuntimePathComposerLookupCacheKey.graph_identity", idiom_rule: "AF-5 import-time devirtualization", evidence: ["crates/nuxie-runtime/src/draw.rs:10920-10949"]}]
idiom_rules_invoked: ["AF-1 retained identity", "AF-2 push never reconstruct", "AF-4 one dirt model", "AF-8 no invented lifecycles"]
confidence: high
notes: "C++ directly invalidates its parent DashPath; Rust marks a global path epoch and later rebuilds effect commands from live properties. Coverage grep: generation|epoch|revision|dirty|observed|snapshot|candidate|alias plus mesh/effect/path family names across crates/nuxie-runtime/src and crates/nuxie-graph/src; sibling sweep found the off-file members named above."

## B6-0344

row_id: B6-0344
cpp_files: ["src/shapes/paint/dash_path.cpp"]
rust_module: "crates/nuxie-runtime/src/draw.rs"
subsystem_cluster: "layout-shapes-paint"
sibling_files_swept: ["crates/nuxie-runtime/src/artboard.rs", "crates/nuxie-runtime/src/components.rs", "crates/nuxie-graph/src/lib.rs"]
verdict: DIVERGENT
axes:
  retained_identity: {status: "adapted-arena-identity", evidence: ["cpp@d788e8ec:src/shapes/paint/dash_path.cpp:9-31,103-166", "crates/nuxie-graph/src/lib.rs:607-817", "crates/nuxie-runtime/src/artboard.rs:228-320"]}
  push_vs_poll: {status: "isomorphic push; extra drift tracking counted under compensation", cpp_pushes: true, evidence: ["cpp@d788e8ec:src/shapes/paint/dash_path.cpp:9-31,103-166", "crates/nuxie-runtime/src/artboard.rs:2120-2131,3788-3790,6175-6185; crates/nuxie-runtime/src/draw.rs:19153-19299"]}
  update_ordering: {status: "phase-sequence-equivalent; representation divergent", phases_cpp: "property change -> ComponentDirt/dependent cascade -> component update -> draw", phases_rust: "property write -> dirt cascade -> epoch/snapshot write -> cache preparation/rebuild -> draw"}
  ownership: {status: "isomorphic-or-arena-adapted", evidence: ["cpp@d788e8ec:src/shapes/paint/dash_path.cpp:9-31,103-166", "crates/nuxie-graph/src/lib.rs:607-817"]}
  compensation:
    status: "present"
    mechanisms: [{name: "effect path epoch fanout", kind: "cross-file drift tracker", mutation_gated: true, cpp_counterpart: "none", evidence: ["crates/nuxie-runtime/src/artboard.rs:2120-2131,3788-3790,6175-6185; crates/nuxie-runtime/src/draw.rs:19153-19299"]}]
    import_time_constants: [{name: "RuntimePathComposerLookupCacheKey.graph_identity", idiom_rule: "AF-5 import-time devirtualization", evidence: ["crates/nuxie-runtime/src/draw.rs:10920-10949"]}]
idiom_rules_invoked: ["AF-1 retained identity", "AF-2 push never reconstruct", "AF-4 one dirt model", "AF-8 no invented lifecycles"]
confidence: high
notes: "C++ retains per-provider effect paths and invalidates them in place; Rust rebuilds copied command vectors behind the subsystem path epoch. Coverage grep: generation|epoch|revision|dirty|observed|snapshot|candidate|alias plus mesh/effect/path family names across crates/nuxie-runtime/src and crates/nuxie-graph/src; sibling sweep found the off-file members named above."

## B6-0345

row_id: B6-0345
cpp_files: ["src/shapes/paint/effects_container.cpp"]
rust_module: "crates/nuxie-runtime/src/draw.rs"
subsystem_cluster: "layout-shapes-paint"
sibling_files_swept: ["crates/nuxie-runtime/src/artboard.rs", "crates/nuxie-runtime/src/components.rs", "crates/nuxie-graph/src/lib.rs"]
verdict: DIVERGENT
axes:
  retained_identity: {status: "adapted-arena-identity", evidence: ["cpp@d788e8ec:src/shapes/paint/effects_container.cpp:10-70", "crates/nuxie-graph/src/lib.rs:607-817", "crates/nuxie-runtime/src/artboard.rs:228-320"]}
  push_vs_poll: {status: "isomorphic push; extra drift tracking counted under compensation", cpp_pushes: true, evidence: ["cpp@d788e8ec:src/shapes/paint/effects_container.cpp:10-70", "crates/nuxie-runtime/src/artboard.rs:3788-3790,6175-6189; crates/nuxie-runtime/src/draw.rs:19080-19149"]}
  update_ordering: {status: "phase-sequence-equivalent; representation divergent", phases_cpp: "property change -> ComponentDirt/dependent cascade -> component update -> draw", phases_rust: "property write -> dirt cascade -> epoch/snapshot write -> cache preparation/rebuild -> draw"}
  ownership: {status: "isomorphic-or-arena-adapted", evidence: ["cpp@d788e8ec:src/shapes/paint/effects_container.cpp:10-70", "crates/nuxie-graph/src/lib.rs:607-817"]}
  compensation:
    status: "present"
    mechanisms: [{name: "effect path epoch fanout", kind: "cross-file drift tracker", mutation_gated: true, cpp_counterpart: "none", evidence: ["crates/nuxie-runtime/src/artboard.rs:3788-3790,6175-6189; crates/nuxie-runtime/src/draw.rs:19080-19149"]}]
    import_time_constants: [{name: "RuntimePathComposerLookupCacheKey.graph_identity", idiom_rule: "AF-5 import-time devirtualization", evidence: ["crates/nuxie-runtime/src/draw.rs:10920-10949"]}]
idiom_rules_invoked: ["AF-1 retained identity", "AF-2 push never reconstruct", "AF-4 one dirt model", "AF-8 no invented lifecycles"]
confidence: high
notes: "C++ retains effect pointers and cascades invalidation from the changed effect onward; Rust walks copied effect descriptors after path-epoch invalidation. Coverage grep: generation|epoch|revision|dirty|observed|snapshot|candidate|alias plus mesh/effect/path family names across crates/nuxie-runtime/src and crates/nuxie-graph/src; sibling sweep found the off-file members named above."

## B6-0346

row_id: B6-0346
cpp_files: ["src/shapes/paint/feather.cpp"]
rust_module: "crates/nuxie-runtime/src/draw.rs"
subsystem_cluster: "layout-shapes-paint"
sibling_files_swept: ["crates/nuxie-runtime/src/artboard.rs", "crates/nuxie-runtime/src/components.rs", "crates/nuxie-graph/src/lib.rs"]
verdict: DIVERGENT
axes:
  retained_identity: {status: "adapted-arena-identity", evidence: ["cpp@d788e8ec:src/shapes/paint/feather.cpp:23-65,86-110", "crates/nuxie-graph/src/lib.rs:607-817", "crates/nuxie-runtime/src/artboard.rs:228-320"]}
  push_vs_poll: {status: "isomorphic push; extra drift tracking counted under compensation", cpp_pushes: true, evidence: ["cpp@d788e8ec:src/shapes/paint/feather.cpp:23-65,86-110", "crates/nuxie-runtime/src/artboard.rs:3728-3755,3788-3790,6175-6188; crates/nuxie-runtime/src/draw.rs:15731-15770"]}
  update_ordering: {status: "phase-sequence-equivalent; representation divergent", phases_cpp: "property change -> ComponentDirt/dependent cascade -> component update -> draw", phases_rust: "property write -> dirt cascade -> epoch/snapshot write -> cache preparation/rebuild -> draw"}
  ownership: {status: "isomorphic-or-arena-adapted", evidence: ["cpp@d788e8ec:src/shapes/paint/feather.cpp:23-65,86-110", "crates/nuxie-graph/src/lib.rs:607-817"]}
  compensation:
    status: "present"
    mechanisms: [{name: "paint/path epoch fanout", kind: "cross-file drift tracker", mutation_gated: true, cpp_counterpart: "none", evidence: ["crates/nuxie-runtime/src/artboard.rs:3728-3755,3788-3790,6175-6188; crates/nuxie-runtime/src/draw.rs:15731-15770"]}]
    import_time_constants: [{name: "RuntimePathComposerLookupCacheKey.graph_identity", idiom_rule: "AF-5 import-time devirtualization", evidence: ["crates/nuxie-runtime/src/draw.rs:10920-10949"]}]
idiom_rules_invoked: ["AF-1 retained identity", "AF-2 push never reconstruct", "AF-4 one dirt model", "AF-8 no invented lifecycles"]
confidence: high
notes: "C++ registers path dependents and mutates retained inner/effect paths; Rust fans feather writes into paint/path epochs used by later configuration/path rebuilds. Coverage grep: generation|epoch|revision|dirty|observed|snapshot|candidate|alias plus mesh/effect/path family names across crates/nuxie-runtime/src and crates/nuxie-graph/src; sibling sweep found the off-file members named above."

## B6-0347

row_id: B6-0347
cpp_files: ["src/shapes/paint/fill.cpp"]
rust_module: "crates/nuxie-runtime/src/draw.rs"
subsystem_cluster: "layout-shapes-paint"
sibling_files_swept: ["crates/nuxie-runtime/src/artboard.rs", "crates/nuxie-runtime/src/components.rs", "crates/nuxie-graph/src/lib.rs"]
verdict: DIVERGENT
axes:
  retained_identity: {status: "adapted-arena-identity", evidence: ["cpp@d788e8ec:src/shapes/paint/fill.cpp:6-45", "crates/nuxie-graph/src/lib.rs:607-817", "crates/nuxie-runtime/src/artboard.rs:228-320"]}
  push_vs_poll: {status: "isomorphic push; extra drift tracking counted under compensation", cpp_pushes: true, evidence: ["cpp@d788e8ec:src/shapes/paint/fill.cpp:6-45", "crates/nuxie-runtime/src/artboard.rs:3788-3790,3948-3966; crates/nuxie-runtime/src/draw.rs:9871-9918"]}
  update_ordering: {status: "phase-sequence-equivalent; representation divergent", phases_cpp: "property change -> ComponentDirt/dependent cascade -> component update -> draw", phases_rust: "property write -> dirt cascade -> epoch/snapshot write -> cache preparation/rebuild -> draw"}
  ownership: {status: "isomorphic-or-arena-adapted", evidence: ["cpp@d788e8ec:src/shapes/paint/fill.cpp:6-45", "crates/nuxie-graph/src/lib.rs:607-817"]}
  compensation:
    status: "present"
    mechanisms: [{name: "shape-paint path epoch cache", kind: "cross-file drift tracker", mutation_gated: true, cpp_counterpart: "none", evidence: ["crates/nuxie-runtime/src/artboard.rs:3788-3790,3948-3966; crates/nuxie-runtime/src/draw.rs:9871-9918"]}]
    import_time_constants: [{name: "RuntimePathComposerLookupCacheKey.graph_identity", idiom_rule: "AF-5 import-time devirtualization", evidence: ["crates/nuxie-runtime/src/draw.rs:10920-10949"]}]
idiom_rules_invoked: ["AF-1 retained identity", "AF-2 push never reconstruct", "AF-4 one dirt model", "AF-8 no invented lifecycles"]
confidence: high
notes: "C++ registers the path builder as a dependent when effects exist; Rust invalidates copied fill/effect path commands via the shared path epoch. Coverage grep: generation|epoch|revision|dirty|observed|snapshot|candidate|alias plus mesh/effect/path family names across crates/nuxie-runtime/src and crates/nuxie-graph/src; sibling sweep found the off-file members named above."

## B6-0348

row_id: B6-0348
cpp_files: ["src/shapes/paint/gradient_stop.cpp"]
rust_module: "crates/nuxie-runtime/src/draw.rs"
subsystem_cluster: "layout-shapes-paint"
sibling_files_swept: ["crates/nuxie-runtime/src/artboard.rs", "crates/nuxie-runtime/src/components.rs", "crates/nuxie-graph/src/lib.rs"]
verdict: DIVERGENT
axes:
  retained_identity: {status: "adapted-arena-identity", evidence: ["cpp@d788e8ec:src/shapes/paint/gradient_stop.cpp:6-28", "crates/nuxie-graph/src/lib.rs:607-817", "crates/nuxie-runtime/src/artboard.rs:228-320"]}
  push_vs_poll: {status: "isomorphic push; extra drift tracking counted under compensation", cpp_pushes: true, evidence: ["cpp@d788e8ec:src/shapes/paint/gradient_stop.cpp:6-28", "crates/nuxie-runtime/src/artboard.rs:3728-3755,4652-4664,5004-5025; crates/nuxie-runtime/src/draw.rs:15731-15770"]}
  update_ordering: {status: "phase-sequence-equivalent; representation divergent", phases_cpp: "property change -> ComponentDirt/dependent cascade -> component update -> draw", phases_rust: "property write -> dirt cascade -> epoch/snapshot write -> cache preparation/rebuild -> draw"}
  ownership: {status: "isomorphic-or-arena-adapted", evidence: ["cpp@d788e8ec:src/shapes/paint/gradient_stop.cpp:6-28", "crates/nuxie-graph/src/lib.rs:607-817"]}
  compensation:
    status: "present"
    mechanisms: [{name: "paint configuration epoch fanout", kind: "cross-file drift tracker", mutation_gated: true, cpp_counterpart: "none", evidence: ["crates/nuxie-runtime/src/artboard.rs:3728-3755,4652-4664,5004-5025; crates/nuxie-runtime/src/draw.rs:15731-15770"]}]
    import_time_constants: [{name: "RuntimePathComposerLookupCacheKey.graph_identity", idiom_rule: "AF-5 import-time devirtualization", evidence: ["crates/nuxie-runtime/src/draw.rs:10920-10949"]}]
idiom_rules_invoked: ["AF-1 retained identity", "AF-2 push never reconstruct", "AF-4 one dirt model", "AF-8 no invented lifecycles"]
confidence: high
notes: "C++ pushes stop changes to the retained parent gradient; Rust additionally bumps instance/cache epochs consumed by paint configuration. Coverage grep: generation|epoch|revision|dirty|observed|snapshot|candidate|alias plus mesh/effect/path family names across crates/nuxie-runtime/src and crates/nuxie-graph/src; sibling sweep found the off-file members named above."

## B6-0349

row_id: B6-0349
cpp_files: ["src/shapes/paint/group_effect.cpp"]
rust_module: "crates/nuxie-runtime/src/draw.rs"
subsystem_cluster: "layout-shapes-paint"
sibling_files_swept: ["crates/nuxie-runtime/src/artboard.rs", "crates/nuxie-runtime/src/components.rs", "crates/nuxie-graph/src/lib.rs"]
verdict: DIVERGENT
axes:
  retained_identity: {status: "adapted-arena-identity", evidence: ["cpp@d788e8ec:src/shapes/paint/group_effect.cpp:6-72", "crates/nuxie-graph/src/lib.rs:607-817", "crates/nuxie-runtime/src/artboard.rs:228-320"]}
  push_vs_poll: {status: "isomorphic push; extra drift tracking counted under compensation", cpp_pushes: true, evidence: ["cpp@d788e8ec:src/shapes/paint/group_effect.cpp:6-72", "crates/nuxie-runtime/src/artboard.rs:3788-3790,6175-6189; crates/nuxie-runtime/src/draw.rs:19080-19149"]}
  update_ordering: {status: "phase-sequence-equivalent; representation divergent", phases_cpp: "property change -> ComponentDirt/dependent cascade -> component update -> draw", phases_rust: "property write -> dirt cascade -> epoch/snapshot write -> cache preparation/rebuild -> draw"}
  ownership: {status: "isomorphic-or-arena-adapted", evidence: ["cpp@d788e8ec:src/shapes/paint/group_effect.cpp:6-72", "crates/nuxie-graph/src/lib.rs:607-817"]}
  compensation:
    status: "present"
    mechanisms: [{name: "effect path epoch fanout", kind: "cross-file drift tracker", mutation_gated: true, cpp_counterpart: "none", evidence: ["crates/nuxie-runtime/src/artboard.rs:3788-3790,6175-6189; crates/nuxie-runtime/src/draw.rs:19080-19149"]}]
    import_time_constants: [{name: "RuntimePathComposerLookupCacheKey.graph_identity", idiom_rule: "AF-5 import-time devirtualization", evidence: ["crates/nuxie-runtime/src/draw.rs:10920-10949"]}]
idiom_rules_invoked: ["AF-1 retained identity", "AF-2 push never reconstruct", "AF-4 one dirt model", "AF-8 no invented lifecycles"]
confidence: high
notes: "C++ retains target/effect pointers and invalidates their path objects; Rust traverses copied group-effect descriptors after path-epoch invalidation. Coverage grep: generation|epoch|revision|dirty|observed|snapshot|candidate|alias plus mesh/effect/path family names across crates/nuxie-runtime/src and crates/nuxie-graph/src; sibling sweep found the off-file members named above."

## B6-0350

row_id: B6-0350
cpp_files: ["src/shapes/paint/linear_gradient.cpp"]
rust_module: "crates/nuxie-runtime/src/draw.rs"
subsystem_cluster: "layout-shapes-paint"
sibling_files_swept: ["crates/nuxie-runtime/src/artboard.rs", "crates/nuxie-runtime/src/components.rs", "crates/nuxie-graph/src/lib.rs"]
verdict: DIVERGENT
axes:
  retained_identity: {status: "adapted-arena-identity", evidence: ["cpp@d788e8ec:src/shapes/paint/linear_gradient.cpp:28-79,86-127,203-215", "crates/nuxie-graph/src/lib.rs:607-817", "crates/nuxie-runtime/src/artboard.rs:228-320"]}
  push_vs_poll: {status: "isomorphic push; extra drift tracking counted under compensation", cpp_pushes: true, evidence: ["cpp@d788e8ec:src/shapes/paint/linear_gradient.cpp:28-79,86-127,203-215", "crates/nuxie-runtime/src/artboard.rs:3728-3755,4820-4925; crates/nuxie-runtime/src/draw.rs:9482-9513,15731-15770"]}
  update_ordering: {status: "phase-sequence-equivalent; representation divergent", phases_cpp: "property change -> ComponentDirt/dependent cascade -> component update -> draw", phases_rust: "property write -> dirt cascade -> epoch/snapshot write -> cache preparation/rebuild -> draw"}
  ownership: {status: "isomorphic-or-arena-adapted", evidence: ["cpp@d788e8ec:src/shapes/paint/linear_gradient.cpp:28-79,86-127,203-215", "crates/nuxie-graph/src/lib.rs:607-817"]}
  compensation:
    status: "present"
    mechanisms: [{name: "paint configuration epoch fanout", kind: "cross-file drift tracker", mutation_gated: true, cpp_counterpart: "none", evidence: ["crates/nuxie-runtime/src/artboard.rs:3728-3755,4820-4925; crates/nuxie-runtime/src/draw.rs:9482-9513,15731-15770"]}]
    import_time_constants: [{name: "RuntimePathComposerLookupCacheKey.graph_identity", idiom_rule: "AF-5 import-time devirtualization", evidence: ["crates/nuxie-runtime/src/draw.rs:10920-10949"]}]
idiom_rules_invoked: ["AF-1 retained identity", "AF-2 push never reconstruct", "AF-4 one dirt model", "AF-8 no invented lifecycles"]
confidence: high
notes: "C++ registers transform dependents and rebuilds its retained paint on precise dirt bits; Rust also tracks cache/prepared epochs and compares paint-preparation keys. Coverage grep: generation|epoch|revision|dirty|observed|snapshot|candidate|alias plus mesh/effect/path family names across crates/nuxie-runtime/src and crates/nuxie-graph/src; sibling sweep found the off-file members named above."

## B6-0351

row_id: B6-0351
cpp_files: ["src/shapes/paint/radial_gradient.cpp"]
rust_module: "crates/nuxie-runtime/src/draw.rs"
subsystem_cluster: "layout-shapes-paint"
sibling_files_swept: ["crates/nuxie-runtime/src/artboard.rs", "crates/nuxie-runtime/src/components.rs", "crates/nuxie-graph/src/lib.rs"]
verdict: ISOMORPHIC
axes:
  retained_identity: {status: "isomorphic", evidence: ["cpp@d788e8ec:src/shapes/paint/radial_gradient.cpp:7-20", "crates/nuxie-graph/src/lib.rs:607-817", "crates/nuxie-runtime/src/artboard.rs:228-320"]}
  push_vs_poll: {status: "isomorphic", cpp_pushes: false, evidence: ["cpp@d788e8ec:src/shapes/paint/radial_gradient.cpp:7-20", "crates/nuxie-runtime/src/draw.rs:20201-21533"]}
  update_ordering: {status: "isomorphic", phases_cpp: "import/build or direct value operation", phases_rust: "graph build or direct value operation"}
  ownership: {status: "isomorphic-or-arena-adapted", evidence: ["cpp@d788e8ec:src/shapes/paint/radial_gradient.cpp:7-20", "crates/nuxie-graph/src/lib.rs:607-817"]}
  compensation:
    status: "clear"
    mechanisms: []
    import_time_constants: []
idiom_rules_invoked: []
confidence: high
notes: "Both sides override only shader construction, deriving radius from endpoint distance; lifecycle and invalidation are inherited from the linear-gradient row. Coverage grep: generation|epoch|revision|dirty|observed|snapshot|candidate|alias plus mesh/effect/path family names across crates/nuxie-runtime/src and crates/nuxie-graph/src; sibling sweep found the off-file members named above."

## B6-0352

row_id: B6-0352
cpp_files: ["src/shapes/paint/shape_paint.cpp"]
rust_module: "crates/nuxie-runtime/src/draw.rs"
subsystem_cluster: "layout-shapes-paint"
sibling_files_swept: ["crates/nuxie-runtime/src/artboard.rs", "crates/nuxie-runtime/src/components.rs", "crates/nuxie-graph/src/lib.rs"]
verdict: DIVERGENT
axes:
  retained_identity: {status: "adapted-arena-identity", evidence: ["cpp@d788e8ec:src/shapes/paint/shape_paint.cpp:12-57,74-205", "crates/nuxie-graph/src/lib.rs:607-817", "crates/nuxie-runtime/src/artboard.rs:228-320"]}
  push_vs_poll: {status: "isomorphic push; extra drift tracking counted under compensation", cpp_pushes: true, evidence: ["cpp@d788e8ec:src/shapes/paint/shape_paint.cpp:12-57,74-205", "crates/nuxie-runtime/src/artboard.rs:3728-3755,3788-3790; crates/nuxie-runtime/src/draw.rs:9871-9918,15731-15770"]}
  update_ordering: {status: "phase-sequence-equivalent; representation divergent", phases_cpp: "property change -> ComponentDirt/dependent cascade -> component update -> draw", phases_rust: "property write -> dirt cascade -> epoch/snapshot write -> cache preparation/rebuild -> draw"}
  ownership: {status: "isomorphic-or-arena-adapted", evidence: ["cpp@d788e8ec:src/shapes/paint/shape_paint.cpp:12-57,74-205", "crates/nuxie-graph/src/lib.rs:607-817"]}
  compensation:
    status: "present"
    mechanisms: [{name: "paint/path epoch fanout", kind: "cross-file drift tracker", mutation_gated: true, cpp_counterpart: "none", evidence: ["crates/nuxie-runtime/src/artboard.rs:3728-3755,3788-3790; crates/nuxie-runtime/src/draw.rs:9871-9918,15731-15770"]}]
    import_time_constants: [{name: "RuntimePathComposerLookupCacheKey.graph_identity", idiom_rule: "AF-5 import-time devirtualization", evidence: ["crates/nuxie-runtime/src/draw.rs:10920-10949"]}]
idiom_rules_invoked: ["AF-1 retained identity", "AF-2 push never reconstruct", "AF-4 one dirt model", "AF-8 no invented lifecycles"]
confidence: high
notes: "C++ owns one retained RenderPaint and retained effect paths; Rust stores renderer resources separately and synchronizes them through paint/path epochs. Coverage grep: generation|epoch|revision|dirty|observed|snapshot|candidate|alias plus mesh/effect/path family names across crates/nuxie-runtime/src and crates/nuxie-graph/src; sibling sweep found the off-file members named above."

## B6-0353

row_id: B6-0353
cpp_files: ["src/shapes/paint/shape_paint_mutator.cpp"]
rust_module: "crates/nuxie-runtime/src/draw.rs"
subsystem_cluster: "layout-shapes-paint"
sibling_files_swept: ["crates/nuxie-runtime/src/artboard.rs", "crates/nuxie-runtime/src/components.rs", "crates/nuxie-graph/src/lib.rs"]
verdict: DIVERGENT
axes:
  retained_identity: {status: "adapted-arena-identity", evidence: ["cpp@d788e8ec:src/shapes/paint/shape_paint_mutator.cpp:7-47", "crates/nuxie-graph/src/lib.rs:607-817", "crates/nuxie-runtime/src/artboard.rs:228-320"]}
  push_vs_poll: {status: "isomorphic push; extra drift tracking counted under compensation", cpp_pushes: true, evidence: ["cpp@d788e8ec:src/shapes/paint/shape_paint_mutator.cpp:7-47", "crates/nuxie-runtime/src/artboard.rs:3728-3755; crates/nuxie-runtime/src/draw.rs:15731-15770"]}
  update_ordering: {status: "phase-sequence-equivalent; representation divergent", phases_cpp: "property change -> ComponentDirt/dependent cascade -> component update -> draw", phases_rust: "property write -> dirt cascade -> epoch/snapshot write -> cache preparation/rebuild -> draw"}
  ownership: {status: "isomorphic-or-arena-adapted", evidence: ["cpp@d788e8ec:src/shapes/paint/shape_paint_mutator.cpp:7-47", "crates/nuxie-graph/src/lib.rs:607-817"]}
  compensation:
    status: "present"
    mechanisms: [{name: "paint configuration epoch fanout", kind: "cross-file drift tracker", mutation_gated: true, cpp_counterpart: "none", evidence: ["crates/nuxie-runtime/src/artboard.rs:3728-3755; crates/nuxie-runtime/src/draw.rs:15731-15770"]}]
    import_time_constants: [{name: "RuntimePathComposerLookupCacheKey.graph_identity", idiom_rule: "AF-5 import-time devirtualization", evidence: ["crates/nuxie-runtime/src/draw.rs:10920-10949"]}]
idiom_rules_invoked: ["AF-1 retained identity", "AF-2 push never reconstruct", "AF-4 one dirt model", "AF-8 no invented lifecycles"]
confidence: high
notes: "C++ stores a direct RenderPaint pointer in the mutator; Rust separates resources and tracks instance epochs to know when to reapply configuration. Coverage grep: generation|epoch|revision|dirty|observed|snapshot|candidate|alias plus mesh/effect/path family names across crates/nuxie-runtime/src and crates/nuxie-graph/src; sibling sweep found the off-file members named above."

## B6-0354

row_id: B6-0354
cpp_files: ["src/shapes/paint/shape_paint_path.cpp"]
rust_module: "crates/nuxie-runtime/src/draw.rs"
subsystem_cluster: "layout-shapes-paint"
sibling_files_swept: ["crates/nuxie-runtime/src/artboard.rs", "crates/nuxie-runtime/src/components.rs", "crates/nuxie-graph/src/lib.rs"]
verdict: DIVERGENT
axes:
  retained_identity: {status: "adapted-arena-identity", evidence: ["cpp@d788e8ec:src/shapes/paint/shape_paint_path.cpp:8-75", "crates/nuxie-graph/src/lib.rs:607-817", "crates/nuxie-runtime/src/artboard.rs:228-320"]}
  push_vs_poll: {status: "isomorphic push; extra drift tracking counted under compensation", cpp_pushes: true, evidence: ["cpp@d788e8ec:src/shapes/paint/shape_paint_path.cpp:8-75", "crates/nuxie-runtime/src/artboard.rs:3788-3790; crates/nuxie-runtime/src/draw.rs:10227-10270,11105-11129"]}
  update_ordering: {status: "phase-sequence-equivalent; representation divergent", phases_cpp: "property change -> ComponentDirt/dependent cascade -> component update -> draw", phases_rust: "property write -> dirt cascade -> epoch/snapshot write -> cache preparation/rebuild -> draw"}
  ownership: {status: "isomorphic-or-arena-adapted", evidence: ["cpp@d788e8ec:src/shapes/paint/shape_paint_path.cpp:8-75", "crates/nuxie-graph/src/lib.rs:607-817"]}
  compensation:
    status: "present"
    mechanisms: [{name: "retained render-path epoch cache", kind: "cross-file drift tracker", mutation_gated: true, cpp_counterpart: "none", evidence: ["crates/nuxie-runtime/src/artboard.rs:3788-3790; crates/nuxie-runtime/src/draw.rs:10227-10270,11105-11129"]}]
    import_time_constants: [{name: "RuntimePathComposerLookupCacheKey.graph_identity", idiom_rule: "AF-5 import-time devirtualization", evidence: ["crates/nuxie-runtime/src/draw.rs:10920-10949"]}]
idiom_rules_invoked: ["AF-1 retained identity", "AF-2 push never reconstruct", "AF-4 one dirt model", "AF-8 no invented lifecycles"]
confidence: high
notes: "C++ retains RawPath/RenderPath together with one local dirty flag; Rust retains renderer paths in external slots keyed by path/layout/world epochs. Coverage grep: generation|epoch|revision|dirty|observed|snapshot|candidate|alias plus mesh/effect/path family names across crates/nuxie-runtime/src and crates/nuxie-graph/src; sibling sweep found the off-file members named above."

## B6-0355

row_id: B6-0355
cpp_files: ["src/shapes/paint/solid_color.cpp"]
rust_module: "crates/nuxie-runtime/src/draw.rs"
subsystem_cluster: "layout-shapes-paint"
sibling_files_swept: ["crates/nuxie-runtime/src/artboard.rs", "crates/nuxie-runtime/src/components.rs", "crates/nuxie-graph/src/lib.rs"]
verdict: DIVERGENT
axes:
  retained_identity: {status: "adapted-arena-identity", evidence: ["cpp@d788e8ec:src/shapes/paint/solid_color.cpp:9-54", "crates/nuxie-graph/src/lib.rs:607-817", "crates/nuxie-runtime/src/artboard.rs:228-320"]}
  push_vs_poll: {status: "isomorphic push; extra drift tracking counted under compensation", cpp_pushes: true, evidence: ["cpp@d788e8ec:src/shapes/paint/solid_color.cpp:9-54", "crates/nuxie-runtime/src/artboard.rs:313-316,1788-1799,1815-1824; crates/nuxie-runtime/src/draw.rs:15747-15758"]}
  update_ordering: {status: "phase-sequence-equivalent; representation divergent", phases_cpp: "property change -> ComponentDirt/dependent cascade -> component update -> draw", phases_rust: "property write -> dirt cascade -> epoch/snapshot write -> cache preparation/rebuild -> draw"}
  ownership: {status: "isomorphic-or-arena-adapted", evidence: ["cpp@d788e8ec:src/shapes/paint/solid_color.cpp:9-54", "crates/nuxie-graph/src/lib.rs:607-817"]}
  compensation:
    status: "present"
    mechanisms: [{name: "solid-color revision handoff", kind: "cross-file drift tracker", mutation_gated: true, cpp_counterpart: "none", evidence: ["crates/nuxie-runtime/src/artboard.rs:313-316,1788-1799,1815-1824; crates/nuxie-runtime/src/draw.rs:15747-15758"]}]
    import_time_constants: [{name: "RuntimePathComposerLookupCacheKey.graph_identity", idiom_rule: "AF-5 import-time devirtualization", evidence: ["crates/nuxie-runtime/src/draw.rs:10920-10949"]}]
idiom_rules_invoked: ["AF-1 retained identity", "AF-2 push never reconstruct", "AF-4 one dirt model", "AF-8 no invented lifecycles"]
confidence: high
notes: "C++ mutates its attached RenderPaint immediately; Rust increments a separate per-mutator revision consumed later by the renderer cache. Coverage grep: generation|epoch|revision|dirty|observed|snapshot|candidate|alias plus mesh/effect/path family names across crates/nuxie-runtime/src and crates/nuxie-graph/src; sibling sweep found the off-file members named above."

## B6-0356

row_id: B6-0356
cpp_files: ["src/shapes/paint/stroke.cpp"]
rust_module: "crates/nuxie-runtime/src/draw.rs"
subsystem_cluster: "layout-shapes-paint"
sibling_files_swept: ["crates/nuxie-runtime/src/artboard.rs", "crates/nuxie-runtime/src/components.rs", "crates/nuxie-graph/src/lib.rs"]
verdict: DIVERGENT
axes:
  retained_identity: {status: "adapted-arena-identity", evidence: ["cpp@d788e8ec:src/shapes/paint/stroke.cpp:8-29,37-77", "crates/nuxie-graph/src/lib.rs:607-817", "crates/nuxie-runtime/src/artboard.rs:228-320"]}
  push_vs_poll: {status: "isomorphic push; extra drift tracking counted under compensation", cpp_pushes: true, evidence: ["cpp@d788e8ec:src/shapes/paint/stroke.cpp:8-29,37-77", "crates/nuxie-runtime/src/artboard.rs:3728-3755,3788-3790; crates/nuxie-runtime/src/draw.rs:15671-15728,15731-15770"]}
  update_ordering: {status: "phase-sequence-equivalent; representation divergent", phases_cpp: "property change -> ComponentDirt/dependent cascade -> component update -> draw", phases_rust: "property write -> dirt cascade -> epoch/snapshot write -> cache preparation/rebuild -> draw"}
  ownership: {status: "isomorphic-or-arena-adapted", evidence: ["cpp@d788e8ec:src/shapes/paint/stroke.cpp:8-29,37-77", "crates/nuxie-graph/src/lib.rs:607-817"]}
  compensation:
    status: "present"
    mechanisms: [{name: "paint/path epoch fanout", kind: "cross-file drift tracker", mutation_gated: true, cpp_counterpart: "none", evidence: ["crates/nuxie-runtime/src/artboard.rs:3728-3755,3788-3790; crates/nuxie-runtime/src/draw.rs:15671-15728,15731-15770"]}]
    import_time_constants: [{name: "RuntimePathComposerLookupCacheKey.graph_identity", idiom_rule: "AF-5 import-time devirtualization", evidence: ["crates/nuxie-runtime/src/draw.rs:10920-10949"]}]
idiom_rules_invoked: ["AF-1 retained identity", "AF-2 push never reconstruct", "AF-4 one dirt model", "AF-8 no invented lifecycles"]
confidence: high
notes: "C++ pushes Paint dirt and mutates the retained RenderPaint; Rust re-reads stroke properties behind paint/path epoch cache boundaries. Coverage grep: generation|epoch|revision|dirty|observed|snapshot|candidate|alias plus mesh/effect/path family names across crates/nuxie-runtime/src and crates/nuxie-graph/src; sibling sweep found the off-file members named above."

## B6-0357

row_id: B6-0357
cpp_files: ["src/shapes/paint/stroke_effect.cpp"]
rust_module: "crates/nuxie-runtime/src/draw.rs"
subsystem_cluster: "layout-shapes-paint"
sibling_files_swept: ["crates/nuxie-runtime/src/artboard.rs", "crates/nuxie-runtime/src/components.rs", "crates/nuxie-graph/src/lib.rs"]
verdict: DIVERGENT
axes:
  retained_identity: {status: "adapted-arena-identity", evidence: ["cpp@d788e8ec:src/shapes/paint/stroke_effect.cpp:13-65", "crates/nuxie-graph/src/lib.rs:607-817", "crates/nuxie-runtime/src/artboard.rs:228-320"]}
  push_vs_poll: {status: "isomorphic push; extra drift tracking counted under compensation", cpp_pushes: true, evidence: ["cpp@d788e8ec:src/shapes/paint/stroke_effect.cpp:13-65", "crates/nuxie-runtime/src/artboard.rs:3788-3790,6175-6189; crates/nuxie-runtime/src/draw.rs:19080-19149"]}
  update_ordering: {status: "phase-sequence-equivalent; representation divergent", phases_cpp: "property change -> ComponentDirt/dependent cascade -> component update -> draw", phases_rust: "property write -> dirt cascade -> epoch/snapshot write -> cache preparation/rebuild -> draw"}
  ownership: {status: "isomorphic-or-arena-adapted", evidence: ["cpp@d788e8ec:src/shapes/paint/stroke_effect.cpp:13-65", "crates/nuxie-graph/src/lib.rs:607-817"]}
  compensation:
    status: "present"
    mechanisms: [{name: "effect path epoch fanout", kind: "cross-file drift tracker", mutation_gated: true, cpp_counterpart: "none", evidence: ["crates/nuxie-runtime/src/artboard.rs:3788-3790,6175-6189; crates/nuxie-runtime/src/draw.rs:19080-19149"]}]
    import_time_constants: [{name: "RuntimePathComposerLookupCacheKey.graph_identity", idiom_rule: "AF-5 import-time devirtualization", evidence: ["crates/nuxie-runtime/src/draw.rs:10920-10949"]}]
idiom_rules_invoked: ["AF-1 retained identity", "AF-2 push never reconstruct", "AF-4 one dirt model", "AF-8 no invented lifecycles"]
confidence: high
notes: "C++ owns per-provider EffectPath objects and invalidates them in place; Rust rebuilds copied effect commands under path epochs. Coverage grep: generation|epoch|revision|dirty|observed|snapshot|candidate|alias plus mesh/effect/path family names across crates/nuxie-runtime/src and crates/nuxie-graph/src; sibling sweep found the off-file members named above."

## B6-0358

row_id: B6-0358
cpp_files: ["src/shapes/paint/target_effect.cpp"]
rust_module: "crates/nuxie-runtime/src/draw.rs"
subsystem_cluster: "layout-shapes-paint"
sibling_files_swept: ["crates/nuxie-runtime/src/artboard.rs", "crates/nuxie-runtime/src/components.rs", "crates/nuxie-graph/src/lib.rs"]
verdict: DIVERGENT
axes:
  retained_identity: {status: "adapted-arena-identity", evidence: ["cpp@d788e8ec:src/shapes/paint/target_effect.cpp:8-31,34-120", "crates/nuxie-graph/src/lib.rs:607-817", "crates/nuxie-runtime/src/artboard.rs:228-320"]}
  push_vs_poll: {status: "isomorphic push; extra drift tracking counted under compensation", cpp_pushes: true, evidence: ["cpp@d788e8ec:src/shapes/paint/target_effect.cpp:8-31,34-120", "crates/nuxie-runtime/src/artboard.rs:3788-3790,6175-6189; crates/nuxie-runtime/src/draw.rs:19106-19149"]}
  update_ordering: {status: "phase-sequence-equivalent; representation divergent", phases_cpp: "property change -> ComponentDirt/dependent cascade -> component update -> draw", phases_rust: "property write -> dirt cascade -> epoch/snapshot write -> cache preparation/rebuild -> draw"}
  ownership: {status: "isomorphic-or-arena-adapted", evidence: ["cpp@d788e8ec:src/shapes/paint/target_effect.cpp:8-31,34-120", "crates/nuxie-graph/src/lib.rs:607-817"]}
  compensation:
    status: "present"
    mechanisms: [{name: "effect path epoch fanout", kind: "cross-file drift tracker", mutation_gated: true, cpp_counterpart: "none", evidence: ["crates/nuxie-runtime/src/artboard.rs:3788-3790,6175-6189; crates/nuxie-runtime/src/draw.rs:19106-19149"]}]
    import_time_constants: [{name: "RuntimePathComposerLookupCacheKey.graph_identity", idiom_rule: "AF-5 import-time devirtualization", evidence: ["crates/nuxie-runtime/src/draw.rs:10920-10949"]}]
idiom_rules_invoked: ["AF-1 retained identity", "AF-2 push never reconstruct", "AF-4 one dirt model", "AF-8 no invented lifecycles"]
confidence: high
notes: "C++ retains a GroupEffect pointer and proxy path providers; Rust embeds copied group-effect descriptors and refreshes results under a global path epoch. Coverage grep: generation|epoch|revision|dirty|observed|snapshot|candidate|alias plus mesh/effect/path family names across crates/nuxie-runtime/src and crates/nuxie-graph/src; sibling sweep found the off-file members named above."

## B6-0359

row_id: B6-0359
cpp_files: ["src/shapes/paint/trim_path.cpp"]
rust_module: "crates/nuxie-runtime/src/draw.rs"
subsystem_cluster: "layout-shapes-paint"
sibling_files_swept: ["crates/nuxie-runtime/src/artboard.rs", "crates/nuxie-runtime/src/components.rs", "crates/nuxie-graph/src/lib.rs"]
verdict: DIVERGENT
axes:
  retained_identity: {status: "adapted-arena-identity", evidence: ["cpp@d788e8ec:src/shapes/paint/trim_path.cpp:6-21,182-225", "crates/nuxie-graph/src/lib.rs:607-817", "crates/nuxie-runtime/src/artboard.rs:228-320"]}
  push_vs_poll: {status: "isomorphic push; extra drift tracking counted under compensation", cpp_pushes: true, evidence: ["cpp@d788e8ec:src/shapes/paint/trim_path.cpp:6-21,182-225", "crates/nuxie-runtime/src/artboard.rs:2120-2131,3788-3790,6175-6179; crates/nuxie-runtime/src/draw.rs:19306-19375"]}
  update_ordering: {status: "phase-sequence-equivalent; representation divergent", phases_cpp: "property change -> ComponentDirt/dependent cascade -> component update -> draw", phases_rust: "property write -> dirt cascade -> epoch/snapshot write -> cache preparation/rebuild -> draw"}
  ownership: {status: "isomorphic-or-arena-adapted", evidence: ["cpp@d788e8ec:src/shapes/paint/trim_path.cpp:6-21,182-225", "crates/nuxie-graph/src/lib.rs:607-817"]}
  compensation:
    status: "present"
    mechanisms: [{name: "effect path epoch fanout", kind: "cross-file drift tracker", mutation_gated: true, cpp_counterpart: "none", evidence: ["crates/nuxie-runtime/src/artboard.rs:2120-2131,3788-3790,6175-6179; crates/nuxie-runtime/src/draw.rs:19306-19375"]}]
    import_time_constants: [{name: "RuntimePathComposerLookupCacheKey.graph_identity", idiom_rule: "AF-5 import-time devirtualization", evidence: ["crates/nuxie-runtime/src/draw.rs:10920-10949"]}]
idiom_rules_invoked: ["AF-1 retained identity", "AF-2 push never reconstruct", "AF-4 one dirt model", "AF-8 no invented lifecycles"]
confidence: high
notes: "C++ retains contour measures/effect paths until explicit invalidation; Rust rebuilds contours and commands when path-epoch keyed state changes. Coverage grep: generation|epoch|revision|dirty|observed|snapshot|candidate|alias plus mesh/effect/path family names across crates/nuxie-runtime/src and crates/nuxie-graph/src; sibling sweep found the off-file members named above."

## B6-0360

row_id: B6-0360
cpp_files: ["src/shapes/parametric_path.cpp"]
rust_module: "crates/nuxie-runtime/src/draw.rs"
subsystem_cluster: "layout-shapes-paint"
sibling_files_swept: ["crates/nuxie-runtime/src/artboard.rs", "crates/nuxie-runtime/src/components.rs", "crates/nuxie-graph/src/lib.rs"]
verdict: DIVERGENT
axes:
  retained_identity: {status: "adapted-arena-identity", evidence: ["cpp@d788e8ec:src/shapes/parametric_path.cpp:9-66", "crates/nuxie-graph/src/lib.rs:607-817", "crates/nuxie-runtime/src/artboard.rs:228-320"]}
  push_vs_poll: {status: "isomorphic push; extra drift tracking counted under compensation", cpp_pushes: true, evidence: ["cpp@d788e8ec:src/shapes/parametric_path.cpp:9-66", "crates/nuxie-runtime/src/artboard.rs:3783-3790,3948-3966; crates/nuxie-runtime/src/draw.rs:10965-10975"]}
  update_ordering: {status: "phase-sequence-equivalent; representation divergent", phases_cpp: "property change -> ComponentDirt/dependent cascade -> component update -> draw", phases_rust: "property write -> dirt cascade -> epoch/snapshot write -> cache preparation/rebuild -> draw"}
  ownership: {status: "isomorphic-or-arena-adapted", evidence: ["cpp@d788e8ec:src/shapes/parametric_path.cpp:9-66", "crates/nuxie-graph/src/lib.rs:607-817"]}
  compensation:
    status: "present"
    mechanisms: [{name: "path/layout epoch fanout", kind: "cross-file drift tracker", mutation_gated: true, cpp_counterpart: "none", evidence: ["crates/nuxie-runtime/src/artboard.rs:3783-3790,3948-3966; crates/nuxie-runtime/src/draw.rs:10965-10975"]}]
    import_time_constants: [{name: "RuntimePathComposerLookupCacheKey.graph_identity", idiom_rule: "AF-5 import-time devirtualization", evidence: ["crates/nuxie-runtime/src/draw.rs:10920-10949"]}]
idiom_rules_invoked: ["AF-1 retained identity", "AF-2 push never reconstruct", "AF-4 one dirt model", "AF-8 no invented lifecycles"]
confidence: high
notes: "C++ pushes path and hosting-layout dirt from size/origin writes; Rust adds path/layout epochs for cache reconstruction. Coverage grep: generation|epoch|revision|dirty|observed|snapshot|candidate|alias plus mesh/effect/path family names across crates/nuxie-runtime/src and crates/nuxie-graph/src; sibling sweep found the off-file members named above."

## B6-0361

row_id: B6-0361
cpp_files: ["src/shapes/path.cpp"]
rust_module: "crates/nuxie-runtime/src/draw.rs"
subsystem_cluster: "layout-shapes-paint"
sibling_files_swept: ["crates/nuxie-runtime/src/artboard.rs", "crates/nuxie-runtime/src/components.rs", "crates/nuxie-graph/src/lib.rs"]
verdict: DIVERGENT
axes:
  retained_identity: {status: "adapted-arena-identity", evidence: ["cpp@d788e8ec:src/shapes/path.cpp:76-105,327-392", "crates/nuxie-graph/src/lib.rs:607-817", "crates/nuxie-runtime/src/artboard.rs:228-320"]}
  push_vs_poll: {status: "isomorphic push; extra drift tracking counted under compensation", cpp_pushes: true, evidence: ["cpp@d788e8ec:src/shapes/path.cpp:76-105,327-392", "crates/nuxie-runtime/src/artboard.rs:3788-3790,3948-3966; crates/nuxie-runtime/src/draw.rs:9835-9868,10965-10975"]}
  update_ordering: {status: "phase-sequence-equivalent; representation divergent", phases_cpp: "property change -> ComponentDirt/dependent cascade -> component update -> draw", phases_rust: "property write -> dirt cascade -> epoch/snapshot write -> cache preparation/rebuild -> draw"}
  ownership: {status: "isomorphic-or-arena-adapted", evidence: ["cpp@d788e8ec:src/shapes/path.cpp:76-105,327-392", "crates/nuxie-graph/src/lib.rs:607-817"]}
  compensation:
    status: "present"
    mechanisms: [{name: "path epoch fanout", kind: "cross-file drift tracker", mutation_gated: true, cpp_counterpart: "none", evidence: ["crates/nuxie-runtime/src/artboard.rs:3788-3790,3948-3966; crates/nuxie-runtime/src/draw.rs:9835-9868,10965-10975"]}]
    import_time_constants: [{name: "RuntimePathComposerLookupCacheKey.graph_identity", idiom_rule: "AF-5 import-time devirtualization", evidence: ["crates/nuxie-runtime/src/draw.rs:10920-10949"]}]
idiom_rules_invoked: ["AF-1 retained identity", "AF-2 push never reconstruct", "AF-4 one dirt model", "AF-8 no invented lifecycles"]
confidence: high
notes: "C++ retains vertices/raw path and pushes dirt into Shape/PathComposer; Rust stores copied graph geometry and refreshes cached commands by path epoch. Coverage grep: generation|epoch|revision|dirty|observed|snapshot|candidate|alias plus mesh/effect/path family names across crates/nuxie-runtime/src and crates/nuxie-graph/src; sibling sweep found the off-file members named above."

## B6-0362

row_id: B6-0362
cpp_files: ["src/shapes/path_composer.cpp"]
rust_module: "crates/nuxie-runtime/src/draw.rs"
subsystem_cluster: "layout-shapes-paint"
sibling_files_swept: ["crates/nuxie-runtime/src/artboard.rs", "crates/nuxie-runtime/src/components.rs", "crates/nuxie-graph/src/lib.rs"]
verdict: DIVERGENT
axes:
  retained_identity: {status: "adapted-arena-identity", evidence: ["cpp@d788e8ec:src/shapes/path_composer.cpp:11-49,51-132", "crates/nuxie-graph/src/lib.rs:607-817", "crates/nuxie-runtime/src/artboard.rs:228-320"]}
  push_vs_poll: {status: "isomorphic push; extra drift tracking counted under compensation", cpp_pushes: true, evidence: ["cpp@d788e8ec:src/shapes/path_composer.cpp:11-49,51-132", "crates/nuxie-runtime/src/artboard.rs:3788-3790,3948-3994; crates/nuxie-runtime/src/draw.rs:9794-9832,10920-10955"]}
  update_ordering: {status: "phase-sequence-equivalent; representation divergent", phases_cpp: "property change -> ComponentDirt/dependent cascade -> component update -> draw", phases_rust: "property write -> dirt cascade -> epoch/snapshot write -> cache preparation/rebuild -> draw"}
  ownership: {status: "isomorphic-or-arena-adapted", evidence: ["cpp@d788e8ec:src/shapes/path_composer.cpp:11-49,51-132", "crates/nuxie-graph/src/lib.rs:607-817"]}
  compensation:
    status: "present"
    mechanisms: [{name: "path-composer epoch reconstruction", kind: "cross-file drift tracker", mutation_gated: true, cpp_counterpart: "none", evidence: ["crates/nuxie-runtime/src/artboard.rs:3788-3790,3948-3994; crates/nuxie-runtime/src/draw.rs:9794-9832,10920-10955"]}]
    import_time_constants: [{name: "RuntimePathComposerLookupCacheKey.graph_identity", idiom_rule: "AF-5 import-time devirtualization", evidence: ["crates/nuxie-runtime/src/draw.rs:10920-10949"]}]
idiom_rules_invoked: ["AF-1 retained identity", "AF-2 push never reconstruct", "AF-4 one dirt model", "AF-8 no invented lifecycles"]
confidence: high
notes: "C++ registers Shape/Path dependents and owns three mutable composed paths; Rust preindexes composers and reconstructs command copies under the instance path epoch. Coverage grep: generation|epoch|revision|dirty|observed|snapshot|candidate|alias plus mesh/effect/path family names across crates/nuxie-runtime/src and crates/nuxie-graph/src; sibling sweep found the off-file members named above."

## B6-0363

row_id: B6-0363
cpp_files: ["src/shapes/path_vertex.cpp"]
rust_module: "crates/nuxie-runtime/src/draw.rs"
subsystem_cluster: "layout-shapes-paint"
sibling_files_swept: ["crates/nuxie-runtime/src/artboard.rs", "crates/nuxie-runtime/src/components.rs", "crates/nuxie-graph/src/lib.rs"]
verdict: DIVERGENT
axes:
  retained_identity: {status: "adapted-arena-identity", evidence: ["cpp@d788e8ec:src/shapes/path_vertex.cpp:6-29", "crates/nuxie-graph/src/lib.rs:607-817", "crates/nuxie-runtime/src/artboard.rs:228-320"]}
  push_vs_poll: {status: "isomorphic push; extra drift tracking counted under compensation", cpp_pushes: true, evidence: ["cpp@d788e8ec:src/shapes/path_vertex.cpp:6-29", "crates/nuxie-runtime/src/artboard.rs:3788-3790,3948-3966; crates/nuxie-runtime/src/draw.rs:10965-10975"]}
  update_ordering: {status: "phase-sequence-equivalent; representation divergent", phases_cpp: "property change -> ComponentDirt/dependent cascade -> component update -> draw", phases_rust: "property write -> dirt cascade -> epoch/snapshot write -> cache preparation/rebuild -> draw"}
  ownership: {status: "isomorphic-or-arena-adapted", evidence: ["cpp@d788e8ec:src/shapes/path_vertex.cpp:6-29", "crates/nuxie-graph/src/lib.rs:607-817"]}
  compensation:
    status: "present"
    mechanisms: [{name: "path epoch fanout", kind: "cross-file drift tracker", mutation_gated: true, cpp_counterpart: "none", evidence: ["crates/nuxie-runtime/src/artboard.rs:3788-3790,3948-3966; crates/nuxie-runtime/src/draw.rs:10965-10975"]}]
    import_time_constants: [{name: "RuntimePathComposerLookupCacheKey.graph_identity", idiom_rule: "AF-5 import-time devirtualization", evidence: ["crates/nuxie-runtime/src/draw.rs:10920-10949"]}]
idiom_rules_invoked: ["AF-1 retained identity", "AF-2 push never reconstruct", "AF-4 one dirt model", "AF-8 no invented lifecycles"]
confidence: high
notes: "C++ retains the parent Path pointer and pushes geometry dirt; Rust addresses the vertex by local_id and refreshes path commands via the shared epoch. Coverage grep: generation|epoch|revision|dirty|observed|snapshot|candidate|alias plus mesh/effect/path family names across crates/nuxie-runtime/src and crates/nuxie-graph/src; sibling sweep found the off-file members named above."

## B6-0364

row_id: B6-0364
cpp_files: ["src/shapes/points_common_path.cpp"]
rust_module: "crates/nuxie-runtime/src/draw.rs"
subsystem_cluster: "layout-shapes-paint"
sibling_files_swept: ["crates/nuxie-runtime/src/artboard.rs", "crates/nuxie-runtime/src/components.rs", "crates/nuxie-graph/src/lib.rs"]
verdict: ISOMORPHIC
axes:
  retained_identity: {status: "isomorphic", evidence: ["cpp@d788e8ec:src/shapes/points_common_path.cpp:12-15", "crates/nuxie-graph/src/lib.rs:607-817", "crates/nuxie-runtime/src/artboard.rs:228-320"]}
  push_vs_poll: {status: "isomorphic", cpp_pushes: false, evidence: ["cpp@d788e8ec:src/shapes/points_common_path.cpp:12-15", "crates/nuxie-runtime/src/draw.rs:20201-21533"]}
  update_ordering: {status: "isomorphic", phases_cpp: "import/build or direct value operation", phases_rust: "graph build or direct value operation"}
  ownership: {status: "isomorphic-or-arena-adapted", evidence: ["cpp@d788e8ec:src/shapes/points_common_path.cpp:12-15", "crates/nuxie-graph/src/lib.rs:607-817"]}
  compensation:
    status: "clear"
    mechanisms: []
    import_time_constants: []
idiom_rules_invoked: []
confidence: high
notes: "Both sides derive clockwise orientation directly from the authored bit flag; there is no retained or cyclic mechanism. Coverage grep: generation|epoch|revision|dirty|observed|snapshot|candidate|alias plus mesh/effect/path family names across crates/nuxie-runtime/src and crates/nuxie-graph/src; sibling sweep found the off-file members named above."

## B6-0365

row_id: B6-0365
cpp_files: ["src/shapes/points_path.cpp"]
rust_module: "crates/nuxie-runtime/src/draw.rs"
subsystem_cluster: "layout-shapes-paint"
sibling_files_swept: ["crates/nuxie-runtime/src/artboard.rs", "crates/nuxie-runtime/src/components.rs", "crates/nuxie-graph/src/lib.rs"]
verdict: DIVERGENT
axes:
  retained_identity: {status: "adapted-arena-identity", evidence: ["cpp@d788e8ec:src/shapes/points_path.cpp:12-52", "crates/nuxie-graph/src/lib.rs:607-817", "crates/nuxie-runtime/src/artboard.rs:228-320"]}
  push_vs_poll: {status: "isomorphic push; extra drift tracking counted under compensation", cpp_pushes: true, evidence: ["cpp@d788e8ec:src/shapes/points_path.cpp:12-52", "crates/nuxie-runtime/src/artboard.rs:3788-3790,3948-3985; crates/nuxie-runtime/src/draw.rs:10965-10975"]}
  update_ordering: {status: "phase-sequence-equivalent; representation divergent", phases_cpp: "property change -> ComponentDirt/dependent cascade -> component update -> draw", phases_rust: "property write -> dirt cascade -> epoch/snapshot write -> cache preparation/rebuild -> draw"}
  ownership: {status: "isomorphic-or-arena-adapted", evidence: ["cpp@d788e8ec:src/shapes/points_path.cpp:12-52", "crates/nuxie-graph/src/lib.rs:607-817"]}
  compensation:
    status: "present"
    mechanisms: [{name: "path epoch fanout", kind: "cross-file drift tracker", mutation_gated: true, cpp_counterpart: "none", evidence: ["crates/nuxie-runtime/src/artboard.rs:3788-3790,3948-3985; crates/nuxie-runtime/src/draw.rs:10965-10975"]}]
    import_time_constants: [{name: "RuntimePathComposerLookupCacheKey.graph_identity", idiom_rule: "AF-5 import-time devirtualization", evidence: ["crates/nuxie-runtime/src/draw.rs:10920-10949"]}]
idiom_rules_invoked: ["AF-1 retained identity", "AF-2 push never reconstruct", "AF-4 one dirt model", "AF-8 no invented lifecycles"]
confidence: high
notes: "C++ registers Skin as a dependent and deforms retained vertices on Path dirt; Rust adds a path epoch around regenerated weighted commands. Coverage grep: generation|epoch|revision|dirty|observed|snapshot|candidate|alias plus mesh/effect/path family names across crates/nuxie-runtime/src and crates/nuxie-graph/src; sibling sweep found the off-file members named above."

## B6-0366

row_id: B6-0366
cpp_files: ["src/shapes/polygon.cpp"]
rust_module: "crates/nuxie-runtime/src/draw.rs"
subsystem_cluster: "layout-shapes-paint"
sibling_files_swept: ["crates/nuxie-runtime/src/artboard.rs", "crates/nuxie-runtime/src/components.rs", "crates/nuxie-graph/src/lib.rs"]
verdict: DIVERGENT
axes:
  retained_identity: {status: "adapted-arena-identity", evidence: ["cpp@d788e8ec:src/shapes/polygon.cpp:13-54", "crates/nuxie-graph/src/lib.rs:607-817", "crates/nuxie-runtime/src/artboard.rs:228-320"]}
  push_vs_poll: {status: "isomorphic push; extra drift tracking counted under compensation", cpp_pushes: true, evidence: ["cpp@d788e8ec:src/shapes/polygon.cpp:13-54", "crates/nuxie-runtime/src/artboard.rs:3788-3790,3948-3966; crates/nuxie-runtime/src/draw.rs:10965-10975"]}
  update_ordering: {status: "phase-sequence-equivalent; representation divergent", phases_cpp: "property change -> ComponentDirt/dependent cascade -> component update -> draw", phases_rust: "property write -> dirt cascade -> epoch/snapshot write -> cache preparation/rebuild -> draw"}
  ownership: {status: "isomorphic-or-arena-adapted", evidence: ["cpp@d788e8ec:src/shapes/polygon.cpp:13-54", "crates/nuxie-graph/src/lib.rs:607-817"]}
  compensation:
    status: "present"
    mechanisms: [{name: "path epoch fanout", kind: "cross-file drift tracker", mutation_gated: true, cpp_counterpart: "none", evidence: ["crates/nuxie-runtime/src/artboard.rs:3788-3790,3948-3966; crates/nuxie-runtime/src/draw.rs:10965-10975"]}]
    import_time_constants: [{name: "RuntimePathComposerLookupCacheKey.graph_identity", idiom_rule: "AF-5 import-time devirtualization", evidence: ["crates/nuxie-runtime/src/draw.rs:10920-10949"]}]
idiom_rules_invoked: ["AF-1 retained identity", "AF-2 push never reconstruct", "AF-4 one dirt model", "AF-8 no invented lifecycles"]
confidence: high
notes: "C++ resizes and updates a retained vertex vector on Path dirt; Rust regenerates polygon commands behind the path epoch. Coverage grep: generation|epoch|revision|dirty|observed|snapshot|candidate|alias plus mesh/effect/path family names across crates/nuxie-runtime/src and crates/nuxie-graph/src; sibling sweep found the off-file members named above."

## B6-0367

row_id: B6-0367
cpp_files: ["src/shapes/rectangle.cpp"]
rust_module: "crates/nuxie-runtime/src/draw.rs"
subsystem_cluster: "layout-shapes-paint"
sibling_files_swept: ["crates/nuxie-runtime/src/artboard.rs", "crates/nuxie-runtime/src/components.rs", "crates/nuxie-graph/src/lib.rs"]
verdict: DIVERGENT
axes:
  retained_identity: {status: "adapted-arena-identity", evidence: ["cpp@d788e8ec:src/shapes/rectangle.cpp:5-45", "crates/nuxie-graph/src/lib.rs:607-817", "crates/nuxie-runtime/src/artboard.rs:228-320"]}
  push_vs_poll: {status: "isomorphic push; extra drift tracking counted under compensation", cpp_pushes: true, evidence: ["cpp@d788e8ec:src/shapes/rectangle.cpp:5-45", "crates/nuxie-runtime/src/artboard.rs:3788-3790,3948-3966; crates/nuxie-runtime/src/draw.rs:10965-10975"]}
  update_ordering: {status: "phase-sequence-equivalent; representation divergent", phases_cpp: "property change -> ComponentDirt/dependent cascade -> component update -> draw", phases_rust: "property write -> dirt cascade -> epoch/snapshot write -> cache preparation/rebuild -> draw"}
  ownership: {status: "isomorphic-or-arena-adapted", evidence: ["cpp@d788e8ec:src/shapes/rectangle.cpp:5-45", "crates/nuxie-graph/src/lib.rs:607-817"]}
  compensation:
    status: "present"
    mechanisms: [{name: "path epoch fanout", kind: "cross-file drift tracker", mutation_gated: true, cpp_counterpart: "none", evidence: ["crates/nuxie-runtime/src/artboard.rs:3788-3790,3948-3966; crates/nuxie-runtime/src/draw.rs:10965-10975"]}]
    import_time_constants: [{name: "RuntimePathComposerLookupCacheKey.graph_identity", idiom_rule: "AF-5 import-time devirtualization", evidence: ["crates/nuxie-runtime/src/draw.rs:10920-10949"]}]
idiom_rules_invoked: ["AF-1 retained identity", "AF-2 push never reconstruct", "AF-4 one dirt model", "AF-8 no invented lifecycles"]
confidence: high
notes: "C++ mutates four retained vertices on Path dirt; Rust reconstructs rectangle commands under the shared path epoch. Coverage grep: generation|epoch|revision|dirty|observed|snapshot|candidate|alias plus mesh/effect/path family names across crates/nuxie-runtime/src and crates/nuxie-graph/src; sibling sweep found the off-file members named above."

## B6-0368

row_id: B6-0368
cpp_files: ["src/shapes/shape.cpp"]
rust_module: "crates/nuxie-runtime/src/draw.rs"
subsystem_cluster: "layout-shapes-paint"
sibling_files_swept: ["crates/nuxie-runtime/src/artboard.rs", "crates/nuxie-runtime/src/components.rs", "crates/nuxie-graph/src/lib.rs"]
verdict: DIVERGENT
axes:
  retained_identity: {status: "adapted-arena-identity", evidence: ["cpp@d788e8ec:src/shapes/shape.cpp:20-62,74-108,262-330", "crates/nuxie-graph/src/lib.rs:607-817", "crates/nuxie-runtime/src/artboard.rs:228-320"]}
  push_vs_poll: {status: "isomorphic push; extra drift tracking counted under compensation", cpp_pushes: true, evidence: ["cpp@d788e8ec:src/shapes/shape.cpp:20-62,74-108,262-330", "crates/nuxie-runtime/src/artboard.rs:3788-3790,3948-3994; crates/nuxie-runtime/src/draw.rs:9871-9918,11105-11129"]}
  update_ordering: {status: "phase-sequence-equivalent; representation divergent", phases_cpp: "property change -> ComponentDirt/dependent cascade -> component update -> draw", phases_rust: "property write -> dirt cascade -> epoch/snapshot write -> cache preparation/rebuild -> draw"}
  ownership: {status: "isomorphic-or-arena-adapted", evidence: ["cpp@d788e8ec:src/shapes/shape.cpp:20-62,74-108,262-330", "crates/nuxie-graph/src/lib.rs:607-817"]}
  compensation:
    status: "present"
    mechanisms: [{name: "shape/path epoch fanout", kind: "cross-file drift tracker", mutation_gated: true, cpp_counterpart: "none", evidence: ["crates/nuxie-runtime/src/artboard.rs:3788-3790,3948-3994; crates/nuxie-runtime/src/draw.rs:9871-9918,11105-11129"]}]
    import_time_constants: [{name: "RuntimePathComposerLookupCacheKey.graph_identity", idiom_rule: "AF-5 import-time devirtualization", evidence: ["crates/nuxie-runtime/src/draw.rs:10920-10949"]}]
idiom_rules_invoked: ["AF-1 retained identity", "AF-2 push never reconstruct", "AF-4 one dirt model", "AF-8 no invented lifecycles"]
confidence: high
notes: "C++ retains Path and ShapePaint pointers plus an owned PathComposer and pushes dirt through dependencies; Rust uses indexed graph copies and epoch-keyed path caches. Coverage grep: generation|epoch|revision|dirty|observed|snapshot|candidate|alias plus mesh/effect/path family names across crates/nuxie-runtime/src and crates/nuxie-graph/src; sibling sweep found the off-file members named above."

## B6-0369

row_id: B6-0369
cpp_files: ["src/shapes/shape_paint_container.cpp"]
rust_module: "crates/nuxie-runtime/src/draw.rs"
subsystem_cluster: "layout-shapes-paint"
sibling_files_swept: ["crates/nuxie-runtime/src/artboard.rs", "crates/nuxie-runtime/src/components.rs", "crates/nuxie-graph/src/lib.rs"]
verdict: DIVERGENT
axes:
  retained_identity: {status: "adapted-arena-identity", evidence: ["cpp@d788e8ec:src/shapes/shape_paint_container.cpp:17-75", "crates/nuxie-graph/src/lib.rs:607-817", "crates/nuxie-runtime/src/artboard.rs:228-320"]}
  push_vs_poll: {status: "isomorphic push; extra drift tracking counted under compensation", cpp_pushes: true, evidence: ["cpp@d788e8ec:src/shapes/shape_paint_container.cpp:17-75", "crates/nuxie-runtime/src/artboard.rs:3728-3755,3788-3790; crates/nuxie-runtime/src/draw.rs:9482-9513,9871-9918"]}
  update_ordering: {status: "phase-sequence-equivalent; representation divergent", phases_cpp: "property change -> ComponentDirt/dependent cascade -> component update -> draw", phases_rust: "property write -> dirt cascade -> epoch/snapshot write -> cache preparation/rebuild -> draw"}
  ownership: {status: "isomorphic-or-arena-adapted", evidence: ["cpp@d788e8ec:src/shapes/shape_paint_container.cpp:17-75", "crates/nuxie-graph/src/lib.rs:607-817"]}
  compensation:
    status: "present"
    mechanisms: [{name: "paint/path epoch fanout", kind: "cross-file drift tracker", mutation_gated: true, cpp_counterpart: "none", evidence: ["crates/nuxie-runtime/src/artboard.rs:3728-3755,3788-3790; crates/nuxie-runtime/src/draw.rs:9482-9513,9871-9918"]}]
    import_time_constants: [{name: "RuntimePathComposerLookupCacheKey.graph_identity", idiom_rule: "AF-5 import-time devirtualization", evidence: ["crates/nuxie-runtime/src/draw.rs:10920-10949"]}]
idiom_rules_invoked: ["AF-1 retained identity", "AF-2 push never reconstruct", "AF-4 one dirt model", "AF-8 no invented lifecycles"]
confidence: high
notes: "C++ retains ShapePaint pointers and directly invalidates effects/propagates opacity; Rust stores paint descriptors by value and synchronizes renderer state through epochs. Coverage grep: generation|epoch|revision|dirty|observed|snapshot|candidate|alias plus mesh/effect/path family names across crates/nuxie-runtime/src and crates/nuxie-graph/src; sibling sweep found the off-file members named above."

## B6-0370

row_id: B6-0370
cpp_files: ["src/shapes/slice_mesh.cpp"]
rust_module: "crates/nuxie-runtime/src/draw.rs"
subsystem_cluster: "layout-shapes-paint"
sibling_files_swept: ["crates/nuxie-runtime/src/artboard.rs", "crates/nuxie-runtime/src/components.rs", "crates/nuxie-graph/src/lib.rs"]
verdict: DIVERGENT
axes:
  retained_identity: {status: "adapted-arena-identity", evidence: ["cpp@d788e8ec:src/shapes/slice_mesh.cpp:23-54,56-147,316-400", "crates/nuxie-graph/src/lib.rs:607-817", "crates/nuxie-runtime/src/artboard.rs:228-320"]}
  push_vs_poll: {status: "isomorphic push; extra drift tracking counted under compensation", cpp_pushes: true, evidence: ["cpp@d788e8ec:src/shapes/slice_mesh.cpp:23-54,56-147,316-400", "crates/nuxie-runtime/src/draw.rs:9418-9447,13524-13558"]}
  update_ordering: {status: "phase-sequence-equivalent; representation divergent", phases_cpp: "property change -> ComponentDirt/dependent cascade -> component update -> draw", phases_rust: "property write -> dirt cascade -> epoch/snapshot write -> cache preparation/rebuild -> draw"}
  ownership: {status: "isomorphic-or-arena-adapted", evidence: ["cpp@d788e8ec:src/shapes/slice_mesh.cpp:23-54,56-147,316-400", "crates/nuxie-graph/src/lib.rs:607-817"]}
  compensation:
    status: "present"
    mechanisms: [{name: "slice-mesh input snapshot", kind: "cross-file drift tracker", mutation_gated: true, cpp_counterpart: "none", evidence: ["crates/nuxie-runtime/src/draw.rs:9418-9447,13524-13558"]}]
    import_time_constants: []
idiom_rules_invoked: ["AF-1 retained identity", "AF-2 push never reconstruct", "AF-4 one dirt model", "AF-8 no invented lifecycles"]
confidence: high
notes: "C++ SliceMesh is uniquely owned by NSlicer and rebuilt only from pushed dirt; Rust stores a complete last_update snapshot and compares inputs during preparation. Coverage grep: generation|epoch|revision|dirty|observed|snapshot|candidate|alias plus mesh/effect/path family names across crates/nuxie-runtime/src and crates/nuxie-graph/src; sibling sweep found the off-file members named above."

## B6-0371

row_id: B6-0371
cpp_files: ["src/shapes/star.cpp"]
rust_module: "crates/nuxie-runtime/src/draw.rs"
subsystem_cluster: "layout-shapes-paint"
sibling_files_swept: ["crates/nuxie-runtime/src/artboard.rs", "crates/nuxie-runtime/src/components.rs", "crates/nuxie-graph/src/lib.rs"]
verdict: DIVERGENT
axes:
  retained_identity: {status: "adapted-arena-identity", evidence: ["cpp@d788e8ec:src/shapes/star.cpp:9-47", "crates/nuxie-graph/src/lib.rs:607-817", "crates/nuxie-runtime/src/artboard.rs:228-320"]}
  push_vs_poll: {status: "isomorphic push; extra drift tracking counted under compensation", cpp_pushes: true, evidence: ["cpp@d788e8ec:src/shapes/star.cpp:9-47", "crates/nuxie-runtime/src/artboard.rs:3788-3790,3948-3966; crates/nuxie-runtime/src/draw.rs:10965-10975"]}
  update_ordering: {status: "phase-sequence-equivalent; representation divergent", phases_cpp: "property change -> ComponentDirt/dependent cascade -> component update -> draw", phases_rust: "property write -> dirt cascade -> epoch/snapshot write -> cache preparation/rebuild -> draw"}
  ownership: {status: "isomorphic-or-arena-adapted", evidence: ["cpp@d788e8ec:src/shapes/star.cpp:9-47", "crates/nuxie-graph/src/lib.rs:607-817"]}
  compensation:
    status: "present"
    mechanisms: [{name: "path epoch fanout", kind: "cross-file drift tracker", mutation_gated: true, cpp_counterpart: "none", evidence: ["crates/nuxie-runtime/src/artboard.rs:3788-3790,3948-3966; crates/nuxie-runtime/src/draw.rs:10965-10975"]}]
    import_time_constants: [{name: "RuntimePathComposerLookupCacheKey.graph_identity", idiom_rule: "AF-5 import-time devirtualization", evidence: ["crates/nuxie-runtime/src/draw.rs:10920-10949"]}]
idiom_rules_invoked: ["AF-1 retained identity", "AF-2 push never reconstruct", "AF-4 one dirt model", "AF-8 no invented lifecycles"]
confidence: high
notes: "C++ mutates retained polygon vertices under Path dirt; Rust regenerates star commands behind the shared path epoch. Coverage grep: generation|epoch|revision|dirty|observed|snapshot|candidate|alias plus mesh/effect/path family names across crates/nuxie-runtime/src and crates/nuxie-graph/src; sibling sweep found the off-file members named above."

## B6-0372

row_id: B6-0372
cpp_files: ["src/shapes/straight_vertex.cpp"]
rust_module: "crates/nuxie-runtime/src/draw.rs"
subsystem_cluster: "layout-shapes-paint"
sibling_files_swept: ["crates/nuxie-runtime/src/artboard.rs", "crates/nuxie-runtime/src/components.rs", "crates/nuxie-graph/src/lib.rs"]
verdict: DIVERGENT
axes:
  retained_identity: {status: "adapted-arena-identity", evidence: ["cpp@d788e8ec:src/shapes/straight_vertex.cpp:5", "crates/nuxie-graph/src/lib.rs:607-817", "crates/nuxie-runtime/src/artboard.rs:228-320"]}
  push_vs_poll: {status: "isomorphic push; extra drift tracking counted under compensation", cpp_pushes: true, evidence: ["cpp@d788e8ec:src/shapes/straight_vertex.cpp:5", "crates/nuxie-runtime/src/artboard.rs:3788-3790,3948-3966; crates/nuxie-runtime/src/draw.rs:10965-10975"]}
  update_ordering: {status: "phase-sequence-equivalent; representation divergent", phases_cpp: "property change -> ComponentDirt/dependent cascade -> component update -> draw", phases_rust: "property write -> dirt cascade -> epoch/snapshot write -> cache preparation/rebuild -> draw"}
  ownership: {status: "isomorphic-or-arena-adapted", evidence: ["cpp@d788e8ec:src/shapes/straight_vertex.cpp:5", "crates/nuxie-graph/src/lib.rs:607-817"]}
  compensation:
    status: "present"
    mechanisms: [{name: "path epoch fanout", kind: "cross-file drift tracker", mutation_gated: true, cpp_counterpart: "none", evidence: ["crates/nuxie-runtime/src/artboard.rs:3788-3790,3948-3966; crates/nuxie-runtime/src/draw.rs:10965-10975"]}]
    import_time_constants: [{name: "RuntimePathComposerLookupCacheKey.graph_identity", idiom_rule: "AF-5 import-time devirtualization", evidence: ["crates/nuxie-runtime/src/draw.rs:10920-10949"]}]
idiom_rules_invoked: ["AF-1 retained identity", "AF-2 push never reconstruct", "AF-4 one dirt model", "AF-8 no invented lifecycles"]
confidence: high
notes: "C++ radiusChanged pushes geometry dirt to the retained parent Path; Rust refreshes the flattened vertex command through path epochs. Coverage grep: generation|epoch|revision|dirty|observed|snapshot|candidate|alias plus mesh/effect/path family names across crates/nuxie-runtime/src and crates/nuxie-graph/src; sibling sweep found the off-file members named above."

## B6-0373

row_id: B6-0373
cpp_files: ["src/shapes/triangle.cpp"]
rust_module: "crates/nuxie-runtime/src/draw.rs"
subsystem_cluster: "layout-shapes-paint"
sibling_files_swept: ["crates/nuxie-runtime/src/artboard.rs", "crates/nuxie-runtime/src/components.rs", "crates/nuxie-graph/src/lib.rs"]
verdict: DIVERGENT
axes:
  retained_identity: {status: "adapted-arena-identity", evidence: ["cpp@d788e8ec:src/shapes/triangle.cpp:7-31", "crates/nuxie-graph/src/lib.rs:607-817", "crates/nuxie-runtime/src/artboard.rs:228-320"]}
  push_vs_poll: {status: "isomorphic push; extra drift tracking counted under compensation", cpp_pushes: true, evidence: ["cpp@d788e8ec:src/shapes/triangle.cpp:7-31", "crates/nuxie-runtime/src/artboard.rs:3788-3790,3948-3966; crates/nuxie-runtime/src/draw.rs:10965-10975"]}
  update_ordering: {status: "phase-sequence-equivalent; representation divergent", phases_cpp: "property change -> ComponentDirt/dependent cascade -> component update -> draw", phases_rust: "property write -> dirt cascade -> epoch/snapshot write -> cache preparation/rebuild -> draw"}
  ownership: {status: "isomorphic-or-arena-adapted", evidence: ["cpp@d788e8ec:src/shapes/triangle.cpp:7-31", "crates/nuxie-graph/src/lib.rs:607-817"]}
  compensation:
    status: "present"
    mechanisms: [{name: "path epoch fanout", kind: "cross-file drift tracker", mutation_gated: true, cpp_counterpart: "none", evidence: ["crates/nuxie-runtime/src/artboard.rs:3788-3790,3948-3966; crates/nuxie-runtime/src/draw.rs:10965-10975"]}]
    import_time_constants: [{name: "RuntimePathComposerLookupCacheKey.graph_identity", idiom_rule: "AF-5 import-time devirtualization", evidence: ["crates/nuxie-runtime/src/draw.rs:10920-10949"]}]
idiom_rules_invoked: ["AF-1 retained identity", "AF-2 push never reconstruct", "AF-4 one dirt model", "AF-8 no invented lifecycles"]
confidence: high
notes: "C++ mutates three retained vertices on Path dirt; Rust reconstructs triangle commands under the path epoch. Coverage grep: generation|epoch|revision|dirty|observed|snapshot|candidate|alias plus mesh/effect/path family names across crates/nuxie-runtime/src and crates/nuxie-graph/src; sibling sweep found the off-file members named above."

## B6-0374

row_id: B6-0374
cpp_files: ["src/shapes/vertex.cpp"]
rust_module: "crates/nuxie-runtime/src/draw.rs"
subsystem_cluster: "layout-shapes-paint"
sibling_files_swept: ["crates/nuxie-runtime/src/artboard.rs", "crates/nuxie-runtime/src/components.rs", "crates/nuxie-graph/src/lib.rs"]
verdict: DIVERGENT
axes:
  retained_identity: {status: "adapted-arena-identity", evidence: ["cpp@d788e8ec:src/shapes/vertex.cpp:5-23", "crates/nuxie-graph/src/lib.rs:607-817", "crates/nuxie-runtime/src/artboard.rs:228-320"]}
  push_vs_poll: {status: "isomorphic push; extra drift tracking counted under compensation", cpp_pushes: true, evidence: ["cpp@d788e8ec:src/shapes/vertex.cpp:5-23", "crates/nuxie-runtime/src/artboard.rs:3788-3790,3948-3966; crates/nuxie-runtime/src/draw.rs:10965-10975"]}
  update_ordering: {status: "phase-sequence-equivalent; representation divergent", phases_cpp: "property change -> ComponentDirt/dependent cascade -> component update -> draw", phases_rust: "property write -> dirt cascade -> epoch/snapshot write -> cache preparation/rebuild -> draw"}
  ownership: {status: "isomorphic-or-arena-adapted", evidence: ["cpp@d788e8ec:src/shapes/vertex.cpp:5-23", "crates/nuxie-graph/src/lib.rs:607-817"]}
  compensation:
    status: "present"
    mechanisms: [{name: "path epoch fanout", kind: "cross-file drift tracker", mutation_gated: true, cpp_counterpart: "none", evidence: ["crates/nuxie-runtime/src/artboard.rs:3788-3790,3948-3966; crates/nuxie-runtime/src/draw.rs:10965-10975"]}]
    import_time_constants: [{name: "RuntimePathComposerLookupCacheKey.graph_identity", idiom_rule: "AF-5 import-time devirtualization", evidence: ["crates/nuxie-runtime/src/draw.rs:10920-10949"]}]
idiom_rules_invoked: ["AF-1 retained identity", "AF-2 push never reconstruct", "AF-4 one dirt model", "AF-8 no invented lifecycles"]
confidence: high
notes: "C++ pushes x/y geometry dirt and writes retained weighted translation; Rust reads arena values into rebuilt commands governed by path epochs. Coverage grep: generation|epoch|revision|dirty|observed|snapshot|candidate|alias plus mesh/effect/path family names across crates/nuxie-runtime/src and crates/nuxie-graph/src; sibling sweep found the off-file members named above."


## Post-audit additions (2026-08-04)

Four layout owners first appear upstream after the frozen audit ref: they are
absent at `d788e8ec` and present at the live pin `4ac7b327` (sync cycle S4,
`docs/sync/triage-2026-08-02-e0d4913f.md` S4-38/S4-41/S4-47/S4-48). They are
audited against `4ac7b327` with the same five axes as the `B6-0248..0258`
records above. C++ anchors cite that pin; Rust anchors cite the current tree.

Two standing facts frame all four, and neither is counted as a finding:

- The Yoga-to-Taffy engine substitution is declared register row **D3**
  (`docs/parity-gap-register.md`, FLR-20 layout-engine ceiling). A `YGStyle`
  write answered by a `taffy::Style` write is that ceiling, not drift.
- All four still live inside `crates/nuxie-runtime/src/draw.rs`. FLR-16 owns
  extracting them into direct Rust files; the mapped module below is therefore
  `draw.rs`, exactly as for `B6-0248..0253`.

The mutation-timing gate for every `layout revision/epoch fanout` record below:
`ArtboardInstance::mark_layout_changed` increments `layout_revision` and
`prepared_epoch` during the property-write/update cycle
(`crates/nuxie-runtime/src/artboard.rs:6483-6486`), and
`RuntimeLayoutBoundsCacheKey` consumes `layout_revision` as the retained
layout-solve cache key (`crates/nuxie-runtime/src/draw.rs:14751-14756`). Those
are cycle-time writes read to discover staleness, so they pass the gate; C++
instead marks the owning retained `YGNode` dirty and has no counterpart counter.

## B6-0453

Remediated 2026-08-04, same day, in the commit that follows this record: the
four `grid*Changed` pushes are now ported to
`crates/nuxie-runtime/src/layout/grid_item_placement.rs`, so the push axis is
isomorphic and the row takes B6-0252's shape — isomorphic push with the layout
epoch fanout still counted under compensation. The verdict is unchanged and the
axes below are stated post-fix. The pre-fix state is in this file's git history;
the invalidation hole it describes (uint span writes never reaching
`layout_revision`) is covered by
`grid_item_placement_span_write_dirties_the_owning_layout_node`.

row_id: B6-0453
cpp_files: ["src/layout/grid_item_placement.cpp"]
rust_module: "crates/nuxie-runtime/src/layout/grid_item_placement.rs; crates/nuxie-runtime/src/draw.rs"
subsystem_cluster: "layout-shapes-paint"
sibling_files_swept: ["crates/nuxie-runtime/src/artboard.rs", "crates/nuxie-runtime/src/components.rs", "crates/nuxie-runtime/src/layout_component.rs", "crates/nuxie-runtime/src/layout/layout_node_provider.rs", "crates/nuxie-graph/src/lib.rs"]
verdict: DIVERGENT
axes:
  retained_identity: {status: "adapted-arena-identity", evidence: ["cpp@4ac7b327:src/layout/grid_item_placement.cpp:8-22,24-42", "crates/nuxie-runtime/src/draw.rs:12640-12660", "crates/nuxie-graph/src/lib.rs:607-817"], note: "C++ additionally retains the object in the provider's applier list and the parent's dependent list; Rust resolves both by arena child scan, which is AF-3 correct for `from()` but not a retained registration."}
  push_vs_poll: {status: "isomorphic push; extra drift tracking counted under compensation", cpp_pushes: true, evidence: ["cpp@4ac7b327:src/layout/grid_item_placement.cpp:73-84", "crates/nuxie-runtime/src/layout/grid_item_placement.rs:13-48", "crates/nuxie-runtime/src/layout/layout_node_provider.rs:5-50"]}
  update_ordering: {status: "phase-sequence-equivalent; representation divergent", phases_cpp: "placement property change -> markOwnerDirty -> provider markLayoutNodeDirty -> syncStyleChanges applier pass -> solve", phases_rust: "placement property write -> mark_owner_dirty -> owner layout node dirty + layout_revision -> style_for_node rebuild -> taffy tree rebuild -> solve"}
  ownership: {status: "isomorphic-or-arena-adapted", evidence: ["cpp@4ac7b327:src/layout/grid_item_placement.cpp:24-42", "crates/nuxie-graph/src/lib.rs:607-817"]}
  compensation:
    status: "present"
    mechanisms: [{name: "layout revision fanout", kind: "cross-file drift tracker", mutation_gated: true, cpp_counterpart: "none", evidence: ["crates/nuxie-runtime/src/artboard.rs:6491-6502", "crates/nuxie-runtime/src/draw.rs:14751-14756"]}]
    import_time_constants: [{name: "GridItemPlacement property keys", idiom_rule: "AF-5 import-time devirtualization", evidence: ["crates/nuxie-runtime/src/draw.rs:12665-12680"]}]
idiom_rules_invoked: ["AF-1 retained identity", "AF-2 push never reconstruct", "AF-4 one dirt model"]
confidence: high
notes: "C++ keeps the placement object registered in two retained structures — `provider->addLayoutStyleApplier(this)` at onAddedClean and `parent()->addDependent(this)` at buildDependencies — and each of the four `grid*Changed` setters pushes `provider->markLayoutNodeDirty()` at the owning node. Rust now ports that push: `mark_owner_dirty` routes through `layout_node_provider::mark_layout_node_dirty`, which covers both provider shapes C++ resolves via `LayoutNodeProvider::from(parent())` — the parent itself when it is a retained LayoutComponent, and otherwise the layout owning a participant host. What remains divergent is representation, not information: the two retained registrations are still arena child scans at style-build time (`apply_grid_item_style`), and `mark_layout_node_changed` bumps the artboard-wide `layout_revision` that keys the retained layout-solve cache, which C++ has no counterpart for. The placement math is faithful, including the negative-cell shift (`cell < 0 ? cell - 1 : cell`) and the span>1 rule. Coverage grep: generation|epoch|revision|dirty|observed|snapshot|candidate|alias across crates/nuxie-runtime/src and crates/nuxie-graph/src; sibling sweep found the off-file members named above."

## B6-0454

Remediated 2026-08-04, same day, in the commit that follows this record. The
behavior note below was the load-bearing one: a bound or keyed GridTrack size
write reached `mark_prepared_changed` but never `mark_layout_changed`, and
`RuntimeLayoutBoundsCacheKey` is keyed on `layout_revision`, so the stale solve
was reused. All five `*Changed` pushes are now ported to
`crates/nuxie-runtime/src/layout/grid_track.rs` and covered by
`grid_track_size_write_dirties_the_retained_parent_layout_node` and
`grid_track_uint_writes_dirty_the_retained_parent_layout_node`. The verdict is
unchanged — the row now takes B6-0252's shape, isomorphic push with the layout
epoch fanout under compensation — and the axes below are stated post-fix.

row_id: B6-0454
cpp_files: ["src/layout/grid_track.cpp"]
rust_module: "crates/nuxie-runtime/src/layout/grid_track.rs; crates/nuxie-runtime/src/draw.rs"
subsystem_cluster: "layout-shapes-paint"
sibling_files_swept: ["crates/nuxie-runtime/src/artboard.rs", "crates/nuxie-runtime/src/components.rs", "crates/nuxie-runtime/src/layout_component.rs", "crates/nuxie-graph/src/lib.rs"]
verdict: DIVERGENT
axes:
  retained_identity: {status: "adapted-arena-identity", evidence: ["cpp@4ac7b327:src/layout/grid_track.cpp:93-116", "crates/nuxie-runtime/src/draw.rs:12509-12550", "crates/nuxie-graph/src/lib.rs:607-817"]}
  push_vs_poll: {status: "isomorphic push; extra drift tracking counted under compensation", cpp_pushes: true, evidence: ["cpp@4ac7b327:src/layout/grid_track.cpp:10-24", "crates/nuxie-runtime/src/layout/grid_track.rs:7-50", "crates/nuxie-runtime/src/layout_component.rs:5-7"]}
  update_ordering: {status: "phase-sequence-equivalent; representation divergent", phases_cpp: "track property change -> markLayoutDirty on the parent LayoutComponent -> syncContainerStyle rebuild -> solve", phases_rust: "track property write -> mark_layout_dirty on the parent LayoutComponent -> layout_revision -> apply_grid_container_style rebuild -> taffy tree rebuild -> solve"}
  ownership: {status: "isomorphic-or-arena-adapted", evidence: ["cpp@4ac7b327:src/layout/grid_track.cpp:93-116", "crates/nuxie-graph/src/lib.rs:607-817"]}
  compensation:
    status: "present"
    mechanisms: [{name: "layout revision fanout", kind: "cross-file drift tracker", mutation_gated: true, cpp_counterpart: "none", evidence: ["crates/nuxie-runtime/src/artboard.rs:6491-6502", "crates/nuxie-runtime/src/draw.rs:14751-14756"]}]
    import_time_constants: [{name: "GridTrack collection/trackType/trackMaxType closed enum switches", idiom_rule: "AF-5 import-time devirtualization", evidence: ["crates/nuxie-runtime/src/draw.rs:12553-12586"]}]
idiom_rules_invoked: ["AF-1 retained identity", "AF-2 push never reconstruct", "AF-4 one dirt model"]
confidence: high
notes: "`GridTrack::syncContainerStyle` and the Rust `apply_grid_container_style` agree structurally — both walk the container's children, skip non-tracks, bucket by `collection()` into the four template/auto lists, and drop collections past index 3 — and the size mapping agrees per case (points/percent/fr/auto, and the minmax pair when `trackMaxType() != 0`, with the C++ `trackMaxType - 1` shift matching the Rust max-type table). The invalidation route is now ported one-for-one, including C++'s `parent()->is<LayoutComponent>()` guard: a track under a non-layout parent dirties nothing on either side. What remains divergent is representation — `mark_layout_node_changed` bumps the artboard-wide `layout_revision` that keys the retained layout-solve cache, which C++ has no counterpart for because it re-solves through live Yoga nodes under their own dirty bits. Coverage grep as above; sibling sweep found the off-file members named."

## B6-0455

Remediated in part 2026-08-05 (F15/UNIV-1603): the absent `ParticipantAnimation`
lifecycle recorded below is now ported — `concrete.participant_layout` reuses
the LayoutComponent animation state in inherit-only mode, the cascade reaches
participants through transparent containers, the participant advances as its
own AdvancingComponent, the solve settles retargets through `retain_bounds`,
and parametric-path control size reads the animated slot. Upstream
`layout_participant_test.cpp` :203/:256 plus the disable-interpolation case are
ported and bind to the implementation. The verdict is unchanged: the two
mutation-gated mechanisms below still stand, and the taffy bounds map still
never merges animated x/y (the position half of the finding, shared with
LayoutComponent animation — see F15's remainder). The pre-port lifecycle text
below is retained as the audit-time finding.

row_id: B6-0455
cpp_files: ["src/layout/layout_participant.cpp"]
rust_module: "crates/nuxie-runtime/src/draw.rs"
subsystem_cluster: "layout-shapes-paint"
sibling_files_swept: ["crates/nuxie-runtime/src/artboard.rs", "crates/nuxie-runtime/src/components.rs", "crates/nuxie-runtime/src/layout_component.rs", "crates/nuxie-runtime/src/layout/layout_node_provider.rs", "crates/nuxie-graph/src/lib.rs"]
verdict: DIVERGENT
axes:
  retained_identity: {status: "divergent: C++ retains one LayoutData/YGNode per participant for its whole life; Rust owns no participant node and rebuilds the taffy tree each solve", evidence: ["cpp@4ac7b327:src/layout/layout_participant.cpp:114-127,188-212", "crates/nuxie-runtime/src/draw.rs:11340-11395", "crates/nuxie-runtime/src/draw.rs:14751-14756"]}
  push_vs_poll: {status: "divergent: C++ pushes host WorldTransform dirt and owner markLayoutNodeDirty from resync/onSizingChanged/updateLayoutBounds; Rust derives participant bounds from a revision-keyed solved-bounds map", cpp_pushes: true, evidence: ["cpp@4ac7b327:src/layout/layout_participant.cpp:188-212,398-455,457-477", "crates/nuxie-runtime/src/draw.rs:15251-15296", "crates/nuxie-runtime/src/artboard.rs:6483-6486"]}
  update_ordering: {status: "phase-sequence-equivalent; representation divergent", phases_cpp: "property change -> syncStyleChanges into the retained node -> markDirtyAndPropagate -> solve -> updateLayoutBounds -> controlSize + WorldTransform dirt", phases_rust: "property write -> layout_revision/prepared_epoch fence -> taffy tree rebuild -> solve -> layout_bounds map -> world-transform derivation"}
  ownership: {status: "divergent at the animation lifecycle; arena-adapted elsewhere", evidence: ["cpp@4ac7b327:src/layout/layout_participant.cpp:29-43,64-70,528-562", "crates/nuxie-runtime/src/components.rs:1518-1524", "crates/nuxie-runtime/src/artboard.rs:8629-8657"]}
  compensation:
    status: "present"
    mechanisms: [{name: "layout revision/prepared epoch fanout", kind: "cross-file drift tracker", mutation_gated: true, cpp_counterpart: "none", evidence: ["crates/nuxie-runtime/src/artboard.rs:6483-6486", "crates/nuxie-runtime/src/draw.rs:14751-14756"]}, {name: "world-transform memo keyed by cache_epoch/layout_revision", kind: "cross-file drift tracker", mutation_gated: true, cpp_counterpart: "none", evidence: ["crates/nuxie-runtime/src/draw.rs:14813-14817,15271-15296"]}]
    import_time_constants: [{name: "LayoutParticipant child lookup / LayoutNodeStyle property keys", idiom_rule: "AF-5 import-time devirtualization", evidence: ["crates/nuxie-runtime/src/draw.rs:8054-8062,8104-8130"]}]
idiom_rules_invoked: ["AF-1 retained identity", "AF-2 push never reconstruct", "AF-4 one dirt model", "AF-8 no invented lifecycles"]
confidence: high
notes: "The sizing half of this file is faithful: the participant is its own sizing style on both sides (`runtime_layout_node_style_local` falls through to the LayoutParticipant child, draw.rs:8047-8062), the measure hook is wired for hug axes (`TaffyMeasureContext::LayoutParticipantMeasure` -> `measure_layout_participant`, draw.rs:11373,13108-13130), and the applyBaseStyle fill/grow/shrink/basis and grid-branch rules are reproduced in `apply_flex_item_style`/`apply_grid_item_style` (draw.rs:12375-12445,12590-12628). The structural findings are the two above plus one absent lifecycle worth stating explicitly: the whole `ParticipantAnimation` family — `cascadeLayoutStyle` allocating state when the parent layout animates, `advanceComponent`, `applyInterpolation`, and the smoothing/retarget pair — has no Rust counterpart. Rust's layout animation state (`RuntimeLayoutComponentState`, which owns animation_a/b, is_smoothing_animation and the inherited interpolation trio) is constructed only for types that are-a LayoutComponent (`components.rs:1518-1524`), and `cascade_layout_component_animation_style` skips any child without that state (`artboard.rs:8629-8657`). A LayoutParticipant is-a LayoutNodeStyle/LayoutSizingStyle/Component, never a LayoutComponent, so a Shape/Text/Image participant under an animated layout snaps to each solved slot instead of interpolating toward it. This is recorded as a finding rather than a TRACKED-GAP because the two mechanisms above independently pass the mutation-timing gate (the spec forbids inventing a mechanism to reach DIVERGENT, not reporting real ones). Register row F15 (UNIV-1603) now owns the absent lifecycle; the verdict stays DIVERGENT on the mechanisms, and the gap analysis was independently confirmed against the fixture: `animated_participant.riv` carries `interpolationTime: 1.0` but zero keyframes, so its exact corpus status exercises only the initial solve, and upstream's own acceptance tests (`layout_participant_test.cpp:203,256`) drive the interpolation through a runtime resize the oracle never performs. Coverage grep as above; sibling sweep found the off-file members named."

## B6-0456

row_id: B6-0456
cpp_files: ["src/layout/layout_sizing_style.cpp"]
rust_module: "crates/nuxie-runtime/src/draw.rs"
subsystem_cluster: "layout-shapes-paint"
sibling_files_swept: ["crates/nuxie-runtime/src/artboard.rs", "crates/nuxie-runtime/src/components.rs", "crates/nuxie-runtime/src/layout/layout_component_style.rs", "crates/nuxie-graph/src/lib.rs"]
verdict: ADAPTED
axes:
  retained_identity: {status: "adapted-arena-identity", evidence: ["cpp@4ac7b327:src/layout/layout_sizing_style.cpp:38-51", "crates/nuxie-runtime/src/draw.rs:11912-11975", "crates/nuxie-graph/src/lib.rs:607-817"]}
  push_vs_poll: {status: "isomorphic", cpp_pushes: false, evidence: ["cpp@4ac7b327:src/layout/layout_sizing_style.cpp:12-36,38-51,53-72", "crates/nuxie-runtime/src/draw.rs:11709-11721,11912-11975"], note: "the pinned file registers no observer and pushes no dirt; it is called by the owner's style sync. The Rust projection is likewise pull-only at style build."}
  update_ordering: {status: "isomorphic", phases_cpp: "owner style sync -> applyBaseStyle/applyItemStyle write YGStyle", phases_rust: "owner style build -> style_for_node/apply_grid_item_style write taffy Style"}
  ownership: {status: "adapted", evidence: ["cpp@4ac7b327:src/layout/layout_sizing_style.cpp:38-51", "crates/nuxie-graph/src/lib.rs:607-817"]}
  compensation:
    status: "clear"
    mechanisms: []
    import_time_constants: [{name: "display/min/max unit and justify-self closed enum switches", idiom_rule: "AF-5 import-time devirtualization", evidence: ["crates/nuxie-runtime/src/draw.rs:11709-11721,11912-11975,12590-12628"]}]
idiom_rules_invoked: ["AF-1 arena-id retained identity", "AF-5 import-time devirtualization"]
confidence: high
notes: "This file is a pure style projection with no lifecycle of its own: `display()` collapses to none/flex, the four unit accessors are casts, `applyBaseStyle` writes min/max dimensions, and `applyItemStyle` writes the stack cell plus the justify-self resolution. Rust writes the same values into a taffy Style at the same phase — display at style_for_node:11709-11721, min/max at 11912-11975, and the stack-cell + hug-downgrade justify-self rule inside apply_grid_item_style:12590-12628, including the `fill` override back to stretch. The engine substitution is D3, not a finding, and the projection introduces no drift tracker of its own, so this row is ADAPTED on the same reasoning as B6-0253 rather than DIVERGENT like B6-0252 — the push obligations for these properties live in their owners (LayoutComponentStyle B6-0252, LayoutParticipant B6-0455), which is where the layout-revision finding is recorded. Coverage grep: generation|epoch|revision|dirty|observed|snapshot|candidate|alias across crates/nuxie-runtime/src and crates/nuxie-graph/src; sibling sweep found the off-file members named above."
