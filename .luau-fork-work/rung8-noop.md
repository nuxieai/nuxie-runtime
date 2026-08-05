# Rung 8 verified no-ops

Range: `f8ca77ac..decb2d05`.

- `Ast/src/Parser.cpp` — `Parser::parseSimpleType`: formatting-only allocation reflow; no Rust behavior change.
- `Compiler/include/Luau/Compiler.h` — `CompileOptions::vectorPrecision`: default `0` already landed with rung 7 and is covered by the rung-7 test.
- `Compiler/include/luacode.h` — `lua_CompileOptions::vectorPrecision`: C-header documentation/default clarification only; the Rust ABI field and zero default already landed with rung 7.
- `VM/src/lapi.cpp` — `lua_getuserdataname`: whitespace-only hunk.
- `VM/src/lobject.h` — `LuauClass` member-offset comments: documentation-only; the Rust record layout is unchanged and the inheritance implementation follows the documented instance-before-static invariant.
- `Bytecode/src/BytecodeGraph.cpp` — removed C++ includes: Rust has no corresponding include cleanup; the behavioral class-shape graph hunks are ported.
