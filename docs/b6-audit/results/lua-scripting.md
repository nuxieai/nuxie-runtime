# B-6 Structural Audit — lua-scripting

Pinned C++: `/Users/levi/dev/oss/rive-runtime` at `d788e8ec6e8b598526607d6a1e8818e8b637b60c`. All 14 assigned C++ files were read completely. Coverage included the mapped Rust VM module, its full `vm/` sibling family, and crate-wide searches for epoch/generation/revision/dirty/observed/snapshot/candidate/alias/cache/rescan/refresh families. The pinned checkout was read-only.

## B6-0260

```yaml
row_id: B6-0260
cpp_files: ["src/lua/logging_scripting_context.cpp"]
rust_module: "crates/nuxie-scripting/src/vm.rs"
subsystem_cluster: lua-scripting
sibling_files_swept: ["crates/nuxie-scripting/src/vm.rs", "crates/nuxie-scripting/src/vm/bytecode.rs", "crates/nuxie-scripting/src/vm/host_commands.rs", "crates/nuxie-scripting/src/vm/listener_invocation.rs", "crates/nuxie-scripting/src/vm/mat4.rs", "crates/nuxie-scripting/src/vm/renderer.rs", "crates/nuxie-scripting/src/vm/resource_limits.rs", "crates/nuxie-scripting/src/vm/view_model.rs"]
verdict: UNKNOWN
axes:
  retained_identity: {status: unknown, evidence: ["src/lua/logging_scripting_context.cpp:14-79", "crates/nuxie-scripting/src/vm.rs:217-259"]}
  push_vs_poll: {status: unknown, cpp_pushes: true, evidence: ["src/lua/logging_scripting_context.cpp:26-79", "crates/nuxie-scripting/src/vm.rs:217-259"]}
  update_ordering: {status: unknown, phases_cpp: ["receive Lua print/error callback", "forward to ScriptingContext log sink"], phases_rust: ["convert VM failures into ScriptError", "return error to ScriptHost"]}
  ownership: {status: unknown, evidence: ["src/lua/logging_scripting_context.cpp:14-24", "crates/nuxie-scripting/src/vm.rs:217-259"]}
  compensation:
    status: unknown
    mechanisms: []
    import_time_constants:
      - {name: "static mlua module, userdata registry, and cache-key matches", idiom_rule: "AF-5 import-time devirtualization", evidence: ["crates/nuxie-scripting/src/vm.rs:751-782", "crates/nuxie-scripting/src/vm.rs:837-885"]}
idiom_rules_invoked: ["AF-1 retained identity", "AF-2 push, never reconstruct", "AF-5 import-time devirtualization", "AF-7 own-by-value", "AF-8 no invented lifecycles"]
confidence: low
notes: "Blocker: the mapped Rust seam has ScriptError propagation but no explicit host log-sink or Lua-print routing lifecycle comparable to LoggingScriptingContext, so a structural verdict would be speculative."
```

## B6-0261

```yaml
row_id: B6-0261
cpp_files: ["src/lua/lua_artboards.cpp"]
rust_module: "crates/nuxie-scripting/src/vm.rs"
subsystem_cluster: lua-scripting
sibling_files_swept: ["crates/nuxie-scripting/src/vm.rs", "crates/nuxie-scripting/src/vm/bytecode.rs", "crates/nuxie-scripting/src/vm/host_commands.rs", "crates/nuxie-scripting/src/vm/listener_invocation.rs", "crates/nuxie-scripting/src/vm/mat4.rs", "crates/nuxie-scripting/src/vm/renderer.rs", "crates/nuxie-scripting/src/vm/resource_limits.rs", "crates/nuxie-scripting/src/vm/view_model.rs"]
verdict: ADAPTED
axes:
  retained_identity: {status: adapted, evidence: ["src/lua/lua_artboards.cpp:18-1046", "crates/nuxie-scripting/src/vm/renderer.rs:198-480"]}
  push_vs_poll: {status: adapted, cpp_pushes: false, evidence: ["src/lua/lua_artboards.cpp:213-321", "crates/nuxie-scripting/src/vm/renderer.rs:232-480"]}
  update_ordering: {status: adapted, phases_cpp: ["resolve retained artboard/node userdata", "invoke artboard/node operation", "return retained wrapper or value"], phases_rust: ["resolve typed mlua userdata", "invoke ScriptHost artboard/node command", "return typed userdata or value"]}
  ownership: {status: adapted, evidence: ["src/lua/lua_artboards.cpp:18-83", "crates/nuxie-scripting/src/vm/renderer.rs:198-480"]}
  compensation:
    status: absent
    mechanisms: []
    import_time_constants:
      - {name: "static mlua module, userdata registry, and cache-key matches", idiom_rule: "AF-5 import-time devirtualization", evidence: ["crates/nuxie-scripting/src/vm.rs:751-782", "crates/nuxie-scripting/src/vm.rs:837-885"]}
idiom_rules_invoked: ["AF-1 retained identity", "AF-2 push, never reconstruct", "AF-5 import-time devirtualization", "AF-7 own-by-value", "AF-8 no invented lifecycles"]
confidence: high
notes: "The Rust binding consolidates the C++ Lua userdata families behind mlua and ScriptHost commands while preserving stable handle use; no live-cycle drift tracker was found."
```

## B6-0267

```yaml
row_id: B6-0267
cpp_files: ["src/lua/lua_listener_invocation.cpp"]
rust_module: "crates/nuxie-scripting/src/vm.rs"
subsystem_cluster: lua-scripting
sibling_files_swept: ["crates/nuxie-scripting/src/vm.rs", "crates/nuxie-scripting/src/vm/bytecode.rs", "crates/nuxie-scripting/src/vm/host_commands.rs", "crates/nuxie-scripting/src/vm/listener_invocation.rs", "crates/nuxie-scripting/src/vm/mat4.rs", "crates/nuxie-scripting/src/vm/renderer.rs", "crates/nuxie-scripting/src/vm/resource_limits.rs", "crates/nuxie-scripting/src/vm/view_model.rs"]
verdict: UNKNOWN
axes:
  retained_identity: {status: unknown, evidence: ["src/lua/lua_listener_invocation.cpp:115-878", "crates/nuxie-scripting/src/vm/listener_invocation.rs:28-180"]}
  push_vs_poll: {status: unknown, cpp_pushes: true, evidence: ["src/lua/lua_listener_invocation.cpp:233-878", "crates/nuxie-scripting/src/vm/listener_invocation.rs:133-180"]}
  update_ordering: {status: unknown, phases_cpp: ["retain concrete listener invocation", "expose type-specific fields to Lua", "dispatch callback"], phases_rust: ["wrap Pointer or ReportedEvent invocation", "return false/nil for unsupported variants", "dispatch supported callback"]}
  ownership: {status: unknown, evidence: ["src/lua/lua_listener_invocation.cpp:115-878", "crates/nuxie-scripting/src/vm/listener_invocation.rs:28-180"]}
  compensation:
    status: unknown
    mechanisms: []
    import_time_constants:
      - {name: "static mlua module, userdata registry, and cache-key matches", idiom_rule: "AF-5 import-time devirtualization", evidence: ["crates/nuxie-scripting/src/vm.rs:751-782", "crates/nuxie-scripting/src/vm.rs:837-885"]}
idiom_rules_invoked: ["AF-1 retained identity", "AF-2 push, never reconstruct", "AF-5 import-time devirtualization", "AF-7 own-by-value", "AF-8 no invented lifecycles"]
confidence: low
notes: "Blocker and potential behavior gap: the mapped Rust seam implements Pointer and ReportedEvent but returns false/nil for keyboard, text, focus, view-model, and gamepad variants; those missing variants cannot be structurally audited honestly."
```

## B6-0269

```yaml
row_id: B6-0269
cpp_files: ["src/lua/lua_properties.cpp"]
rust_module: "crates/nuxie-scripting/src/vm.rs"
subsystem_cluster: lua-scripting
sibling_files_swept: ["crates/nuxie-scripting/src/vm.rs", "crates/nuxie-scripting/src/vm/bytecode.rs", "crates/nuxie-scripting/src/vm/host_commands.rs", "crates/nuxie-scripting/src/vm/listener_invocation.rs", "crates/nuxie-scripting/src/vm/mat4.rs", "crates/nuxie-scripting/src/vm/renderer.rs", "crates/nuxie-scripting/src/vm/resource_limits.rs", "crates/nuxie-scripting/src/vm/view_model.rs"]
verdict: DIVERGENT
axes:
  retained_identity: {status: mixed, evidence: ["src/lua/lua_properties.cpp:136-377", "crates/nuxie-scripting/src/vm/view_model.rs:22-45", "crates/nuxie-scripting/src/vm/view_model.rs:94-178"]}
  push_vs_poll: {status: divergent, cpp_pushes: true, evidence: ["src/lua/lua_properties.cpp:356-377", "src/lua/rive_lua_libs.cpp:1106-1142", "crates/nuxie-scripting/src/vm/view_model.rs:181-235"]}
  update_ordering: {status: divergent, phases_cpp: ["register retained ScriptViewModelInstance", "link/unlink retained parent relationships at property mutation", "query live hasParents at detached advance"], phases_rust: ["register tracked view-model handle", "mirror parent edges during link/list synchronization", "rescan live instances and rewrite parent mirror before detached advance"]}
  ownership: {status: mixed, evidence: ["src/lua/lua_properties.cpp:136-377", "crates/nuxie-scripting/src/vm/view_model.rs:22-45"]}
  compensation:
    status: present
    mechanisms:
      - {name: "detached view-model parent-edge mirror/rescan", kind: "AF-2/AF-8 bind-cycle relationship mirror", mutation_gated: true, cpp_counterpart: none, evidence: ["crates/nuxie-scripting/src/vm/view_model.rs:22-45", "crates/nuxie-scripting/src/vm/view_model.rs:131-178", "crates/nuxie-scripting/src/vm/view_model.rs:181-235"]}
    import_time_constants:
      - {name: "static mlua module, userdata registry, and cache-key matches", idiom_rule: "AF-5 import-time devirtualization", evidence: ["crates/nuxie-scripting/src/vm.rs:751-782", "crates/nuxie-scripting/src/vm.rs:837-885"]}
idiom_rules_invoked: ["AF-1 retained identity", "AF-2 push, never reconstruct", "AF-5 import-time devirtualization", "AF-7 own-by-value", "AF-8 no invented lifecycles"]
confidence: high
notes: "Rust rewrites a parent-edge mirror during detached-view-model advance, whereas C++ queries retained parent identity directly. This records current mid-RB-1 state; the active work is the #RB-1 mini-queue at docs/parity-closeout-status.md:210-225."
```

## B6-0270

```yaml
row_id: B6-0270
cpp_files: ["src/lua/lua_rive_base.cpp"]
rust_module: "crates/nuxie-scripting/src/vm.rs"
subsystem_cluster: lua-scripting
sibling_files_swept: ["crates/nuxie-scripting/src/vm.rs", "crates/nuxie-scripting/src/vm/bytecode.rs", "crates/nuxie-scripting/src/vm/host_commands.rs", "crates/nuxie-scripting/src/vm/listener_invocation.rs", "crates/nuxie-scripting/src/vm/mat4.rs", "crates/nuxie-scripting/src/vm/renderer.rs", "crates/nuxie-scripting/src/vm/resource_limits.rs", "crates/nuxie-scripting/src/vm/view_model.rs"]
verdict: UNKNOWN
axes:
  retained_identity: {status: unknown, evidence: ["src/lua/lua_rive_base.cpp:1-45", "crates/nuxie-scripting/src/vm.rs:596-782"]}
  push_vs_poll: {status: unknown, cpp_pushes: true, evidence: ["src/lua/lua_rive_base.cpp:10-44", "crates/nuxie-scripting/src/vm.rs:596-782"]}
  update_ordering: {status: unknown, phases_cpp: ["install custom print closure", "route arguments through ScriptingContext log"], phases_rust: ["create Luau VM", "install host globals and modules"]}
  ownership: {status: unknown, evidence: ["src/lua/lua_rive_base.cpp:10-44", "crates/nuxie-scripting/src/vm.rs:596-782"]}
  compensation:
    status: unknown
    mechanisms: []
    import_time_constants:
      - {name: "static mlua module, userdata registry, and cache-key matches", idiom_rule: "AF-5 import-time devirtualization", evidence: ["crates/nuxie-scripting/src/vm.rs:751-782", "crates/nuxie-scripting/src/vm.rs:837-885"]}
idiom_rules_invoked: ["AF-1 retained identity", "AF-2 push, never reconstruct", "AF-5 import-time devirtualization", "AF-7 own-by-value", "AF-8 no invented lifecycles"]
confidence: low
notes: "Blocker: no explicit Rust override of the Lua print global or corresponding host-log routing was found in the mapped module or sibling sweep, so the base-library lifecycle is not comparable."
```

## B6-0275

```yaml
row_id: B6-0275
cpp_files: ["src/lua/math/lua_mat2d.cpp"]
rust_module: "crates/nuxie-scripting/src/vm.rs"
subsystem_cluster: lua-scripting
sibling_files_swept: ["crates/nuxie-scripting/src/vm.rs", "crates/nuxie-scripting/src/vm/bytecode.rs", "crates/nuxie-scripting/src/vm/host_commands.rs", "crates/nuxie-scripting/src/vm/listener_invocation.rs", "crates/nuxie-scripting/src/vm/mat4.rs", "crates/nuxie-scripting/src/vm/renderer.rs", "crates/nuxie-scripting/src/vm/resource_limits.rs", "crates/nuxie-scripting/src/vm/view_model.rs"]
verdict: ADAPTED
axes:
  retained_identity: {status: adapted, evidence: ["src/lua/math/lua_mat2d.cpp:23-437", "crates/nuxie-scripting/src/vm/renderer.rs:1332-1431"]}
  push_vs_poll: {status: adapted, cpp_pushes: false, evidence: ["src/lua/math/lua_mat2d.cpp:75-437", "crates/nuxie-scripting/src/vm/renderer.rs:1332-1431"]}
  update_ordering: {status: adapted, phases_cpp: ["read Mat2D userdata", "perform value operation", "return Mat2D/value"], phases_rust: ["borrow Mat2D userdata", "perform value operation", "return Mat2D/value"]}
  ownership: {status: adapted, evidence: ["src/lua/math/lua_mat2d.cpp:23-73", "crates/nuxie-scripting/src/vm/renderer.rs:1332-1431"]}
  compensation:
    status: absent
    mechanisms: []
    import_time_constants:
      - {name: "static mlua module, userdata registry, and cache-key matches", idiom_rule: "AF-5 import-time devirtualization", evidence: ["crates/nuxie-scripting/src/vm.rs:751-782", "crates/nuxie-scripting/src/vm.rs:837-885"]}
idiom_rules_invoked: ["AF-1 retained identity", "AF-2 push, never reconstruct", "AF-5 import-time devirtualization", "AF-7 own-by-value", "AF-8 no invented lifecycles"]
confidence: high
notes: "Direct value-type adaptation through mlua userdata; no update-cycle state or drift compensation."
```

## B6-0276

```yaml
row_id: B6-0276
cpp_files: ["src/lua/math/lua_mat4.cpp"]
rust_module: "crates/nuxie-scripting/src/vm.rs"
subsystem_cluster: lua-scripting
sibling_files_swept: ["crates/nuxie-scripting/src/vm.rs", "crates/nuxie-scripting/src/vm/bytecode.rs", "crates/nuxie-scripting/src/vm/host_commands.rs", "crates/nuxie-scripting/src/vm/listener_invocation.rs", "crates/nuxie-scripting/src/vm/mat4.rs", "crates/nuxie-scripting/src/vm/renderer.rs", "crates/nuxie-scripting/src/vm/resource_limits.rs", "crates/nuxie-scripting/src/vm/view_model.rs"]
verdict: ADAPTED
axes:
  retained_identity: {status: adapted, evidence: ["src/lua/math/lua_mat4.cpp:20-455", "crates/nuxie-scripting/src/vm/mat4.rs:55-98"]}
  push_vs_poll: {status: adapted, cpp_pushes: false, evidence: ["src/lua/math/lua_mat4.cpp:83-455", "crates/nuxie-scripting/src/vm/mat4.rs:315-550"]}
  update_ordering: {status: adapted, phases_cpp: ["read Mat4 userdata", "perform value operation", "return Mat4/value"], phases_rust: ["borrow Mat4 userdata", "perform value operation", "return Mat4/value"]}
  ownership: {status: adapted, evidence: ["src/lua/math/lua_mat4.cpp:20-81", "crates/nuxie-scripting/src/vm/mat4.rs:55-98"]}
  compensation:
    status: absent
    mechanisms: []
    import_time_constants:
      - {name: "static mlua module, userdata registry, and cache-key matches", idiom_rule: "AF-5 import-time devirtualization", evidence: ["crates/nuxie-scripting/src/vm.rs:751-782", "crates/nuxie-scripting/src/vm.rs:837-885"]}
idiom_rules_invoked: ["AF-1 retained identity", "AF-2 push, never reconstruct", "AF-5 import-time devirtualization", "AF-7 own-by-value", "AF-8 no invented lifecycles"]
confidence: high
notes: "Direct value-type adaptation through mlua userdata; no live-cycle mutation-gated mechanism."
```

## B6-0277

```yaml
row_id: B6-0277
cpp_files: ["src/lua/math/lua_math.cpp"]
rust_module: "crates/nuxie-scripting/src/vm.rs"
subsystem_cluster: lua-scripting
sibling_files_swept: ["crates/nuxie-scripting/src/vm.rs", "crates/nuxie-scripting/src/vm/bytecode.rs", "crates/nuxie-scripting/src/vm/host_commands.rs", "crates/nuxie-scripting/src/vm/listener_invocation.rs", "crates/nuxie-scripting/src/vm/mat4.rs", "crates/nuxie-scripting/src/vm/renderer.rs", "crates/nuxie-scripting/src/vm/resource_limits.rs", "crates/nuxie-scripting/src/vm/view_model.rs"]
verdict: ADAPTED
axes:
  retained_identity: {status: adapted, evidence: ["src/lua/math/lua_math.cpp:1-24", "crates/nuxie-scripting/src/vm.rs:751-782"]}
  push_vs_poll: {status: adapted, cpp_pushes: false, evidence: ["src/lua/math/lua_math.cpp:8-23", "crates/nuxie-scripting/src/vm.rs:751-782"]}
  update_ordering: {status: adapted, phases_cpp: ["construct math library table", "register Mat2D/Mat4/Vec2D modules"], phases_rust: ["construct VM globals", "install math userdata constructors/modules"]}
  ownership: {status: adapted, evidence: ["src/lua/math/lua_math.cpp:8-23", "crates/nuxie-scripting/src/vm.rs:751-782"]}
  compensation:
    status: absent
    mechanisms: []
    import_time_constants:
      - {name: "static mlua module, userdata registry, and cache-key matches", idiom_rule: "AF-5 import-time devirtualization", evidence: ["crates/nuxie-scripting/src/vm.rs:751-782", "crates/nuxie-scripting/src/vm.rs:837-885"]}
idiom_rules_invoked: ["AF-1 retained identity", "AF-2 push, never reconstruct", "AF-5 import-time devirtualization", "AF-7 own-by-value", "AF-8 no invented lifecycles"]
confidence: high
notes: "Registration is an AF-5 import-time adaptation, not a runtime compensation lifecycle."
```

## B6-0278

```yaml
row_id: B6-0278
cpp_files: ["src/lua/math/lua_vec2d.cpp"]
rust_module: "crates/nuxie-scripting/src/vm.rs"
subsystem_cluster: lua-scripting
sibling_files_swept: ["crates/nuxie-scripting/src/vm.rs", "crates/nuxie-scripting/src/vm/bytecode.rs", "crates/nuxie-scripting/src/vm/host_commands.rs", "crates/nuxie-scripting/src/vm/listener_invocation.rs", "crates/nuxie-scripting/src/vm/mat4.rs", "crates/nuxie-scripting/src/vm/renderer.rs", "crates/nuxie-scripting/src/vm/resource_limits.rs", "crates/nuxie-scripting/src/vm/view_model.rs"]
verdict: ADAPTED
axes:
  retained_identity: {status: adapted, evidence: ["src/lua/math/lua_vec2d.cpp:10-305", "crates/nuxie-scripting/src/vm.rs:1273-1400"]}
  push_vs_poll: {status: adapted, cpp_pushes: false, evidence: ["src/lua/math/lua_vec2d.cpp:56-305", "crates/nuxie-scripting/src/vm.rs:1273-1400"]}
  update_ordering: {status: adapted, phases_cpp: ["read Vec2D userdata", "perform value operation", "return Vec2D/value"], phases_rust: ["borrow vector userdata", "perform value operation", "return vector/value"]}
  ownership: {status: adapted, evidence: ["src/lua/math/lua_vec2d.cpp:10-54", "crates/nuxie-scripting/src/vm.rs:1273-1400"]}
  compensation:
    status: absent
    mechanisms: []
    import_time_constants:
      - {name: "static mlua module, userdata registry, and cache-key matches", idiom_rule: "AF-5 import-time devirtualization", evidence: ["crates/nuxie-scripting/src/vm.rs:751-782", "crates/nuxie-scripting/src/vm.rs:837-885"]}
idiom_rules_invoked: ["AF-1 retained identity", "AF-2 push, never reconstruct", "AF-5 import-time devirtualization", "AF-7 own-by-value", "AF-8 no invented lifecycles"]
confidence: high
notes: "The wrapper representation differs, but the value semantics and operation ordering are preserved without compensation."
```

## B6-0284

```yaml
row_id: B6-0284
cpp_files: ["src/lua/renderer/lua_paint.cpp"]
rust_module: "crates/nuxie-scripting/src/vm.rs"
subsystem_cluster: lua-scripting
sibling_files_swept: ["crates/nuxie-scripting/src/vm.rs", "crates/nuxie-scripting/src/vm/bytecode.rs", "crates/nuxie-scripting/src/vm/host_commands.rs", "crates/nuxie-scripting/src/vm/listener_invocation.rs", "crates/nuxie-scripting/src/vm/mat4.rs", "crates/nuxie-scripting/src/vm/renderer.rs", "crates/nuxie-scripting/src/vm/resource_limits.rs", "crates/nuxie-scripting/src/vm/view_model.rs"]
verdict: ADAPTED
axes:
  retained_identity: {status: adapted, evidence: ["src/lua/renderer/lua_paint.cpp:158-608", "crates/nuxie-scripting/src/vm/renderer.rs:974-1185"]}
  push_vs_poll: {status: adapted, cpp_pushes: true, evidence: ["src/lua/renderer/lua_paint.cpp:207-608", "crates/nuxie-scripting/src/vm/renderer.rs:974-1185"]}
  update_ordering: {status: adapted, phases_cpp: ["mutate retained ScriptedPaintData", "mark/update native render paint", "render with native paint"], phases_rust: ["mutate retained Lua paint userdata", "emit ScriptHost paint commands", "render with retained paint handle"]}
  ownership: {status: adapted, evidence: ["src/lua/renderer/lua_paint.cpp:158-205", "crates/nuxie-scripting/src/vm/renderer.rs:974-1185"]}
  compensation:
    status: absent
    mechanisms: []
    import_time_constants:
      - {name: "static mlua module, userdata registry, and cache-key matches", idiom_rule: "AF-5 import-time devirtualization", evidence: ["crates/nuxie-scripting/src/vm.rs:751-782", "crates/nuxie-scripting/src/vm.rs:837-885"]}
idiom_rules_invoked: ["AF-1 retained identity", "AF-2 push, never reconstruct", "AF-5 import-time devirtualization", "AF-7 own-by-value", "AF-8 no invented lifecycles"]
confidence: high
notes: "Rust routes paint mutation through the host command seam; matched dirty/cache terms belong to the retained renderer implementation and have direct semantic counterparts."
```

## B6-0285

```yaml
row_id: B6-0285
cpp_files: ["src/lua/renderer/lua_path.cpp"]
rust_module: "crates/nuxie-scripting/src/vm.rs"
subsystem_cluster: lua-scripting
sibling_files_swept: ["crates/nuxie-scripting/src/vm.rs", "crates/nuxie-scripting/src/vm/bytecode.rs", "crates/nuxie-scripting/src/vm/host_commands.rs", "crates/nuxie-scripting/src/vm/listener_invocation.rs", "crates/nuxie-scripting/src/vm/mat4.rs", "crates/nuxie-scripting/src/vm/renderer.rs", "crates/nuxie-scripting/src/vm/resource_limits.rs", "crates/nuxie-scripting/src/vm/view_model.rs"]
verdict: ADAPTED
axes:
  retained_identity: {status: adapted, evidence: ["src/lua/renderer/lua_path.cpp:14-700", "crates/nuxie-scripting/src/vm/renderer.rs:516-950"]}
  push_vs_poll: {status: adapted, cpp_pushes: true, evidence: ["src/lua/renderer/lua_path.cpp:42-700", "crates/nuxie-scripting/src/vm/renderer.rs:516-950"]}
  update_ordering: {status: adapted, phases_cpp: ["mutate retained ScriptedPathData", "mark native render path dirty", "rebuild/render retained path"], phases_rust: ["mutate retained path userdata", "emit path commands or rewind cached path", "render retained path handle"]}
  ownership: {status: adapted, evidence: ["src/lua/renderer/lua_path.cpp:14-40", "crates/nuxie-scripting/src/vm/renderer.rs:516-562"]}
  compensation:
    status: absent
    mechanisms: []
    import_time_constants:
      - {name: "static mlua module, userdata registry, and cache-key matches", idiom_rule: "AF-5 import-time devirtualization", evidence: ["crates/nuxie-scripting/src/vm.rs:751-782", "crates/nuxie-scripting/src/vm.rs:837-885"]}
idiom_rules_invoked: ["AF-1 retained identity", "AF-2 push, never reconstruct", "AF-5 import-time devirtualization", "AF-7 own-by-value", "AF-8 no invented lifecycles"]
confidence: med
notes: "Potential behavior gap: Rust lacks the C++ same-frame RenderPath replacement guard in ScriptedPathData::renderPath; recorded only, with no remediation decision."
```

## B6-0286

```yaml
row_id: B6-0286
cpp_files: ["src/lua/renderer/lua_renderer.cpp"]
rust_module: "crates/nuxie-scripting/src/vm.rs"
subsystem_cluster: lua-scripting
sibling_files_swept: ["crates/nuxie-scripting/src/vm.rs", "crates/nuxie-scripting/src/vm/bytecode.rs", "crates/nuxie-scripting/src/vm/host_commands.rs", "crates/nuxie-scripting/src/vm/listener_invocation.rs", "crates/nuxie-scripting/src/vm/mat4.rs", "crates/nuxie-scripting/src/vm/renderer.rs", "crates/nuxie-scripting/src/vm/resource_limits.rs", "crates/nuxie-scripting/src/vm/view_model.rs"]
verdict: ADAPTED
axes:
  retained_identity: {status: adapted, evidence: ["src/lua/renderer/lua_renderer.cpp:9-213", "crates/nuxie-scripting/src/vm/renderer.rs:407-513"]}
  push_vs_poll: {status: adapted, cpp_pushes: true, evidence: ["src/lua/renderer/lua_renderer.cpp:25-213", "crates/nuxie-scripting/src/vm/renderer.rs:407-513"]}
  update_ordering: {status: adapted, phases_cpp: ["receive Lua renderer call", "forward transform/clip/draw to retained Renderer", "restore renderer state"], phases_rust: ["receive typed userdata call", "emit transform/clip/draw ScriptHost command", "restore host renderer state"]}
  ownership: {status: adapted, evidence: ["src/lua/renderer/lua_renderer.cpp:9-23", "crates/nuxie-scripting/src/vm/renderer.rs:407-513"]}
  compensation:
    status: absent
    mechanisms: []
    import_time_constants:
      - {name: "static mlua module, userdata registry, and cache-key matches", idiom_rule: "AF-5 import-time devirtualization", evidence: ["crates/nuxie-scripting/src/vm.rs:751-782", "crates/nuxie-scripting/src/vm.rs:837-885"]}
idiom_rules_invoked: ["AF-1 retained identity", "AF-2 push, never reconstruct", "AF-5 import-time devirtualization", "AF-7 own-by-value", "AF-8 no invented lifecycles"]
confidence: high
notes: "The Rust host-command boundary adapts direct C++ Renderer calls without adding a drift-tracking lifecycle."
```

## B6-0287

```yaml
row_id: B6-0287
cpp_files: ["src/lua/renderer/lua_renderer_library.cpp"]
rust_module: "crates/nuxie-scripting/src/vm.rs"
subsystem_cluster: lua-scripting
sibling_files_swept: ["crates/nuxie-scripting/src/vm.rs", "crates/nuxie-scripting/src/vm/bytecode.rs", "crates/nuxie-scripting/src/vm/host_commands.rs", "crates/nuxie-scripting/src/vm/listener_invocation.rs", "crates/nuxie-scripting/src/vm/mat4.rs", "crates/nuxie-scripting/src/vm/renderer.rs", "crates/nuxie-scripting/src/vm/resource_limits.rs", "crates/nuxie-scripting/src/vm/view_model.rs"]
verdict: ADAPTED
axes:
  retained_identity: {status: adapted, evidence: ["src/lua/renderer/lua_renderer_library.cpp:1-38", "crates/nuxie-scripting/src/vm/renderer.rs:45-210"]}
  push_vs_poll: {status: adapted, cpp_pushes: false, evidence: ["src/lua/renderer/lua_renderer_library.cpp:10-37", "crates/nuxie-scripting/src/vm/renderer.rs:45-210"]}
  update_ordering: {status: adapted, phases_cpp: ["construct renderer library table", "register color/path/paint constructors and enums"], phases_rust: ["construct renderer module", "register typed constructors and enum constants"]}
  ownership: {status: adapted, evidence: ["src/lua/renderer/lua_renderer_library.cpp:10-37", "crates/nuxie-scripting/src/vm/renderer.rs:45-210"]}
  compensation:
    status: absent
    mechanisms: []
    import_time_constants:
      - {name: "static mlua module, userdata registry, and cache-key matches", idiom_rule: "AF-5 import-time devirtualization", evidence: ["crates/nuxie-scripting/src/vm.rs:751-782", "crates/nuxie-scripting/src/vm.rs:837-885"]}
idiom_rules_invoked: ["AF-1 retained identity", "AF-2 push, never reconstruct", "AF-5 import-time devirtualization", "AF-7 own-by-value", "AF-8 no invented lifecycles"]
confidence: high
notes: "Library registration and enum tables are AF-5 import-time devirtualization."
```

## B6-0288

```yaml
row_id: B6-0288
cpp_files: ["src/lua/rive_lua_libs.cpp"]
rust_module: "crates/nuxie-scripting/src/vm.rs"
subsystem_cluster: lua-scripting
sibling_files_swept: ["crates/nuxie-scripting/src/vm.rs", "crates/nuxie-scripting/src/vm/bytecode.rs", "crates/nuxie-scripting/src/vm/host_commands.rs", "crates/nuxie-scripting/src/vm/listener_invocation.rs", "crates/nuxie-scripting/src/vm/mat4.rs", "crates/nuxie-scripting/src/vm/renderer.rs", "crates/nuxie-scripting/src/vm/resource_limits.rs", "crates/nuxie-scripting/src/vm/view_model.rs"]
verdict: DIVERGENT
axes:
  retained_identity: {status: mixed, evidence: ["src/lua/rive_lua_libs.cpp:329-1270", "crates/nuxie-scripting/src/vm.rs:837-1145", "crates/nuxie-scripting/src/vm/view_model.rs:22-45"]}
  push_vs_poll: {status: divergent, cpp_pushes: true, evidence: ["src/lua/rive_lua_libs.cpp:1106-1142", "crates/nuxie-scripting/src/vm/view_model.rs:181-235"]}
  update_ordering: {status: divergent, phases_cpp: ["install libraries and retain native userdata", "track ScriptViewModelInstance", "query each retained instance hasParents during detached advance"], phases_rust: ["install cached mlua modules and userdata", "track view-model handles plus parent mirror", "rescan live instances and rewrite parent edges before detached advance"]}
  ownership: {status: mixed, evidence: ["src/lua/rive_lua_libs.cpp:1106-1142", "crates/nuxie-scripting/src/vm/view_model.rs:22-45"]}
  compensation:
    status: present
    mechanisms:
      - {name: "detached view-model parent-edge mirror/rescan", kind: "AF-2/AF-8 advance-cycle relationship mirror", mutation_gated: true, cpp_counterpart: none, evidence: ["crates/nuxie-scripting/src/vm/view_model.rs:22-45", "crates/nuxie-scripting/src/vm/view_model.rs:131-178", "crates/nuxie-scripting/src/vm/view_model.rs:181-235"]}
    import_time_constants:
      - {name: "static mlua module, userdata registry, and cache-key matches", idiom_rule: "AF-5 import-time devirtualization", evidence: ["crates/nuxie-scripting/src/vm.rs:751-782", "crates/nuxie-scripting/src/vm.rs:837-885"]}
idiom_rules_invoked: ["AF-1 retained identity", "AF-2 push, never reconstruct", "AF-5 import-time devirtualization", "AF-7 own-by-value", "AF-8 no invented lifecycles"]
confidence: high
notes: "C++ checks retained ScriptViewModelInstance parent identity directly; Rust rebuilds a parallel parent-edge set before the detached advance. This is current mid-RB-1 state, tracked by the #RB-1 mini-queue at docs/parity-closeout-status.md:210-225."
```

