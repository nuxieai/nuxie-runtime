# Luau fork rung 4 verified no-op dispositions

Diff: `f1f121dc..ddcea05e` (official Luau 0.727 to 0.728)

These ledger rows required no Rust source edit because the rung 1-3 tree
already had the target spelling or behavior:

| C++ symbol | Rust twin | Verification |
|---|---|---|
| `AstExprInstantiate::AstExprInstantiate` | `vendor/luaur-ast-0.1.8/src/methods/ast_expr_instantiate_ast_expr_instantiate.rs` | Parameter is already named `types`. |
| `Parser::getAttributeStartLocation` | `vendor/luaur-ast-0.1.8/src/methods/parser_get_attribute_start_location.rs` | Rung 2/3 twin exists and already uses `default_location`. |
| `Printer::visualize(AstExpr&)` | `vendor/luaur-ast-0.1.8/src/methods/printer_visualize_pretty_printer_alt_b.rs` | Group expressions already consult `CstExprGroup` unconditionally and fall back to the AST location. |
| `prettyPrint(string_view, ParseOptions, bool, bool)` | `vendor/luaur-ast-0.1.8/src/functions/pretty_print_pretty_printer_alt_c.rs` | Parse errors already abort only when `ignore_parse_errors` is false. |
| `BytecodeBuilder::dumpInstruction` | `vendor/luaur-bytecode-0.1.8/src/methods/bytecode_builder_dump_instruction.rs` | Output parameter is already named `result`. |
| `toFunctionBytecode` overload declarations | `vendor/luaur-bytecode-0.1.8/src/functions/to_function_bytecode_bytecode_graph.rs`; `vendor/luaur-bytecode-0.1.8/src/functions/to_function_bytecode_bytecode_graph_alt_b.rs` | Rust parameters already use `bcb` and `fn_`/`_fn_`; no signature or behavior delta. |
| `startsWith` declaration | `vendor/luaur-common-0.1.8/src/functions/starts_with.rs` | Parameters are already `haystack` and `needle`. |
| `getfuncname` forward declaration | `vendor/luaur-vm-0.1.8/src/functions/getfuncname.rs` | Parameter is already `cl`. |
| `luaS_hash` formatting | `vendor/luaur-vm-0.1.8/src/functions/lua_s_hash.rs` | C++ hunk adds only a blank line; no Rust behavior or symbol change. |
