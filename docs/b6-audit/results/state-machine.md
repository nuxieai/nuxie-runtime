# B-6 Structural Audit — state-machine

Pinned C++: `/Users/levi/dev/oss/rive-runtime` at `d788e8ec6e8b598526607d6a1e8818e8b637b60c`. The mapped Rust module and the live listener-instance sibling were swept, followed by a crate-wide listener/pointer/capture/previous-position/state grep. The upstream checkout was read-only.

## B6-0259

```yaml
row_id: B6-0259
cpp_files: ["src/listener_group.cpp"]
rust_module: "crates/nuxie-runtime/src/state_machine.rs"
subsystem_cluster: state-machine
sibling_files_swept: ["crates/nuxie-runtime/src/state_machine/instance.rs", "crates/nuxie-runtime/src/focus.rs", "crates/nuxie-runtime/src/scripting.rs"]
verdict: ADAPTED
axes:
  retained_identity: {status: adapted, idiom_rule: "AF-7 own-by-value", evidence: ["src/listener_group.cpp:24-39", "include/rive/listener_group.hpp:19-30", "crates/nuxie-runtime/src/state_machine/instance.rs:81-113", "crates/nuxie-runtime/src/state_machine/instance.rs:1141-1209"]}
  push_vs_poll: {status: isomorphic, cpp_pushes: true, evidence: ["src/listener_group.cpp:235-246", "crates/nuxie-runtime/src/state_machine/instance.rs:1393-1404", "crates/nuxie-runtime/src/state_machine/instance.rs:1974-1985"]}
  update_ordering: {status: isomorphic, phases_cpp: ["hit/hover and click-phase update", "perform listener changes", "mark state machine for advance", "record previous pointer position"], phases_rust: ["hit/hover and capture-phase update", "perform listener actions", "set needs_advance", "record previous pointer position"]}
  ownership: {status: adapted, evidence: ["include/rive/listener_group.hpp:60-66", "src/listener_group.cpp:12-38", "crates/nuxie-runtime/src/state_machine/instance.rs:81-113", "crates/nuxie-runtime/src/state_machine/instance.rs:1167-1176"]}
  compensation:
    status: clear
    mechanisms: []
    import_time_constants: []
idiom_rules_invoked: ["AF-7 own-by-value"]
confidence: high
notes: "C++ uniquely owns heap-allocated per-pointer listener data in a map plus reuse pool; Rust keeps the same per-listener/per-pointer identity and prior-position/hover/gesture state in owned vectors keyed by listener index and pointer ID. Rust's pointer_positions and pointer_down_listener_hits are event-phase pointer/capture state with direct C++ semantic counterparts, not drift trackers written during advance/update/bind, so they do not pass the compensation mutation-timing gate. The crate-wide family grep and recorded sibling sweep found no off-file observer, generation, alias-mirror, copied-cache refresh, or rescan compensation family."
```
