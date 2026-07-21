# B-6 Structural Audit — artboard

Pinned C++: `/Users/levi/dev/oss/rive-runtime` at `d788e8ec6e8b598526607d6a1e8818e8b637b60c`. Both assigned C++ implementations were read completely. Coverage included the mapped Rust artboard module plus crate-wide epoch/generation/revision/dirty/observed/snapshot/candidate/alias and nested-artboard family searches, followed by the sibling sweep recorded on each row. The pinned checkout was read-only.

## B6-0094

```yaml
row_id: B6-0094
cpp_files: ["src/artboard.cpp"]
rust_module: "crates/nuxie-runtime/src/artboard.rs"
subsystem_cluster: artboard
sibling_files_swept: ["crates/nuxie-runtime/src/artboard_data_bind.rs", "crates/nuxie-runtime/src/draw.rs", "crates/nuxie-runtime/src/components.rs", "crates/nuxie-runtime/src/data_bind_graph.rs", "crates/nuxie-runtime/src/state_machine/instance.rs", "crates/nuxie-runtime/src/text.rs"]
verdict: DIVERGENT
axes:
  retained_identity: {status: mixed, evidence: ["src/artboard.cpp:176-546", "crates/nuxie-runtime/src/artboard.rs:220-321", "crates/nuxie-runtime/src/artboard.rs:3948-3990"]}
  push_vs_poll: {status: divergent, cpp_pushes: true, evidence: ["src/artboard.cpp:997-1011", "src/artboard.cpp:1204-1240", "crates/nuxie-runtime/src/artboard.rs:3728-3795", "crates/nuxie-runtime/src/draw.rs:9159-9175"]}
  update_ordering: {status: mixed, phases_cpp: ["update data binds", "sync layout style", "apply joysticks", "drain retained component dirt", "post-component data binds"], phases_rust: ["advance retained/buffered data binds", "drain component dirt", "write epoch frontiers", "compare cached renderer/bind frontiers", "settle nested/list layout"]}
  ownership: {status: adapted, evidence: ["src/artboard.cpp:69-176", "crates/nuxie-runtime/src/artboard.rs:220-321"]}
  compensation:
    status: present
    mechanisms:
      - {name: "artboard multi-epoch cache frontier", kind: "epoch/revision drift-tracking family", mutation_gated: true, cpp_counterpart: none, evidence: ["crates/nuxie-runtime/src/artboard.rs:302-317", "crates/nuxie-runtime/src/artboard.rs:3728-3795", "crates/nuxie-runtime/src/artboard.rs:3948-3973", "crates/nuxie-runtime/src/draw.rs:9159-9175"]}
      - {name: "artboard data-bind dirty/processed frontier", kind: "bind-cycle generation comparison", mutation_gated: true, cpp_counterpart: none, evidence: ["crates/nuxie-runtime/src/artboard.rs:292-294", "crates/nuxie-runtime/src/artboard.rs:3733-3744", "crates/nuxie-runtime/src/artboard_data_bind.rs:5788-5820", "crates/nuxie-runtime/src/artboard_data_bind.rs:5921-5926"]}
    import_time_constants:
      - {name: "data_bind_observed and static graph membership", idiom_rule: "AF-5 import-time devirtualization", evidence: ["crates/nuxie-runtime/src/artboard.rs:935", "crates/nuxie-runtime/src/artboard.rs:1019-1053"]}
idiom_rules_invoked: ["AF-1 retained identity", "AF-2 push, never reconstruct", "AF-4 one dirt model", "AF-5 import-time devirtualization", "AF-7 own-by-value", "AF-8 no invented lifecycles"]
confidence: high
notes: "Rust preserves stable instance/object identity but adds cache and bind frontiers that are written and consumed during update/bind/render-preparation cycles; C++ uses retained object-local dirt and dependency order without those frontier families. RB-1 migration is in flight for the bind frontier; the current-state work is identified by the #RB-1 mini-queue at docs/parity-closeout-status.md:210-225. Potential behavior gap from the manifest: Artboard::volume propagation remains absent in Rust."
```

## B6-0303

```yaml
row_id: B6-0303
cpp_files: ["src/nested_artboard.cpp"]
rust_module: "crates/nuxie-runtime/src/artboard.rs"
subsystem_cluster: artboard
sibling_files_swept: ["crates/nuxie-runtime/src/artboard_data_bind.rs", "crates/nuxie-runtime/src/draw.rs", "crates/nuxie-runtime/src/components.rs", "crates/nuxie-runtime/src/data_bind_graph.rs", "crates/nuxie-runtime/src/state_machine/instance.rs", "crates/nuxie-runtime/src/focus.rs"]
verdict: DIVERGENT
axes:
  retained_identity: {status: mixed, evidence: ["src/nested_artboard.cpp:92-135", "src/nested_artboard.cpp:228-396", "crates/nuxie-runtime/src/artboard.rs:339-386", "crates/nuxie-runtime/src/artboard.rs:5095-5130"]}
  push_vs_poll: {status: divergent, cpp_pushes: true, evidence: ["src/nested_artboard.cpp:626-651", "src/nested_artboard.cpp:965-1031", "crates/nuxie-runtime/src/artboard.rs:1615-1641", "crates/nuxie-runtime/src/draw.rs:2128-2179"]}
  update_ordering: {status: divergent, phases_cpp: ["resolve/swap retained nested instance", "push host dirt/context", "advance nested animations and instance", "run nested update pass"], phases_rust: ["resolve/swap boxed nested instance", "increment structure/render frontiers", "compare layout and bind snapshots", "refresh detached nested state", "advance/update child"]}
  ownership: {status: mixed, evidence: ["src/nested_artboard.cpp:40-90", "src/nested_artboard.cpp:228-396", "crates/nuxie-runtime/src/artboard.rs:339-386", "crates/nuxie-runtime/src/artboard.rs:5095-5130"]}
  compensation:
    status: present
    mechanisms:
      - {name: "nested structure/render revision frontier", kind: "cross-file epoch/revision drift tracker", mutation_gated: true, cpp_counterpart: none, evidence: ["crates/nuxie-runtime/src/artboard.rs:342-354", "crates/nuxie-runtime/src/artboard.rs:1615-1641", "crates/nuxie-runtime/src/artboard.rs:5111-5129", "crates/nuxie-runtime/src/draw.rs:2128-2179"]}
      - {name: "nested layout bounds and transfer-key snapshot", kind: "update-cycle copied layout frontier", mutation_gated: true, cpp_counterpart: none, evidence: ["crates/nuxie-runtime/src/artboard.rs:3361-3380", "crates/nuxie-runtime/src/artboard.rs:3491-3539"]}
      - {name: "stateful nested view-model detached-context reconciliation", kind: "bind-cycle dirty mirror/rescan", mutation_gated: true, cpp_counterpart: none, evidence: ["crates/nuxie-runtime/src/artboard.rs:289-294", "crates/nuxie-runtime/src/artboard.rs:5121-5126", "crates/nuxie-runtime/src/artboard_data_bind.rs:7508-7524"]}
    import_time_constants:
      - {name: "nested graph/global-id and static data-bind path descriptors", idiom_rule: "AF-5 import-time devirtualization", evidence: ["crates/nuxie-runtime/src/artboard.rs:256-268", "crates/nuxie-runtime/src/artboard.rs:354-386"]}
idiom_rules_invoked: ["AF-1 retained identity", "AF-2 push, never reconstruct", "AF-4 one dirt model", "AF-5 import-time devirtualization", "AF-7 own-by-value", "AF-8 no invented lifecycles"]
confidence: high
notes: "C++ retains the mounted ArtboardInstance and pushes host dirt/context directly. Rust retains the child by value but also writes structure/render revisions, layout transfer snapshots, and a detached-context dirty mirror during live cycles. The nested view-model mechanism records current mid-RB-1 state and cites the #RB-1 mini-queue at docs/parity-closeout-status.md:210-225. Potential behavior gap from the manifest: latent nested hit-propagation ceilings remain."
```
