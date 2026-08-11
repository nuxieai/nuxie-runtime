# Post-B6 port review index

These records preserve the structural dispositions that were reviewed after
the frozen B6 audit. They are an index of the corresponding manifest notes and
tests, not fresh proof against the current Rust tree. The proof-aware scorecard
therefore keeps them stale until owner-scoped source freshness is established.

- row_id: B6-0146; cpp_files: ["src/core.cpp"]; verdict: ADAPTED; review: "P3-b retained property-observer push boundary"
- row_id: B6-0260; cpp_files: ["src/lua/logging_scripting_context.cpp"]; verdict: ADAPTED; review: "P1G host logging and error routing"
- row_id: B6-0264; cpp_files: ["src/lua/lua_data_context.cpp"]; verdict: ADAPTED; review: "P3-b retained nil-model data-context topology"
- row_id: B6-0265; cpp_files: ["src/lua/lua_data_value.cpp"]; verdict: TRACKED-GAP; review: "LT-2 data-value semantics with remaining F7 surface"
- row_id: B6-0266; cpp_files: ["src/lua/lua_image_decode.cpp"]; verdict: ADAPTED; review: "P2A WorkPool image decode adaptation"
- row_id: B6-0267; cpp_files: ["src/lua/lua_listener_invocation.cpp"]; verdict: ISOMORPHIC; review: "P1J listener invocation wrappers"
- row_id: B6-0270; cpp_files: ["src/lua/lua_rive_base.cpp"]; verdict: ISOMORPHIC; review: "P1G host-routed Rive globals"
- row_id: B6-0271; cpp_files: ["src/lua/lua_scripted_context.cpp"]; verdict: TRACKED-GAP; review: "FTAIL retained context surface with open Canvas and markNeedsUpdate gaps"
- row_id: B6-0272; cpp_files: ["src/lua/lua_state.cpp"]; verdict: TRACKED-GAP; review: "LT-2 state constructor semantics with remaining F7 surface"
- row_id: B6-0273; cpp_files: ["src/lua/math/lua_color.cpp"]; verdict: ISOMORPHIC; review: "P1G Lua color projection"
- row_id: B6-0274; cpp_files: ["src/lua/math/lua_input.cpp"]; verdict: ADAPTED; review: "P1J pointer-event and typed input projection"
- row_id: B6-0279; cpp_files: ["src/lua/renderer/lua_blob.cpp"]; verdict: ADAPTED; review: "P2B immutable asset-registry adaptation"
- row_id: B6-0280; cpp_files: ["src/lua/renderer/lua_gpu.cpp"]; verdict: ADAPTED; review: "GPUCEIL D18/X3 renderer adaptation"
- row_id: B6-0281; cpp_files: ["src/lua/renderer/lua_gradient.cpp"]; verdict: TRACKED-GAP; review: "LT-2 gradient semantics with remaining renderer surface"
- row_id: B6-0282; cpp_files: ["src/lua/renderer/lua_image.cpp"]; verdict: ADAPTED; review: "P2A/P2B image wrapper adaptation"
- row_id: B6-0283; cpp_files: ["src/lua/renderer/lua_mesh.cpp"]; verdict: ADAPTED; review: "P2B mesh wrapper adaptation"
- row_id: B6-0322; cpp_files: ["src/scripted/scripted_drawable.cpp"]; verdict: ADAPTED; review: "post-RB-3 direct scripted owner lifecycle"
- row_id: B6-0325; cpp_files: ["src/scripted/scripted_object.cpp"]; verdict: ADAPTED; review: "post-RB-3 direct scripted owner lifecycle"
- row_id: B6-0326; cpp_files: ["src/scripted/scripted_path_effect.cpp"]; verdict: ADAPTED; review: "post-RB-3 direct scripted owner lifecycle"
- row_id: B6-0339; cpp_files: ["src/shapes/list_path.cpp"]; verdict: ADAPTED; review: "post-F10 concrete list-path ownership"
