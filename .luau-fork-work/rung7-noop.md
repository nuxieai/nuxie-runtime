# Luau fork rung 7 verified no-op rows

## Source-only and representation-only deltas

- `LuauTrackPrefixLocal` / `LuauNoDuplicateBinaryPrefix` declaration order (`Ast/src/Parser.cpp`): source-order-only; Rust flags live in the central registry.
- `<algorithm>` include (`Bytecode/include/Luau/BytecodeGraph.h`): C++ include required by the new erase helpers; Rust uses `retain` and needs no include twin.
- `Sccp::updateBlockUses` (`Bytecode/include/Luau/Sccp.h`): Rust already narrows `len()` explicitly with `as u32`.
- `LUAU_MAYBE_UNUSED` (`Common/include/Luau/Common.h`): C++ compiler-warning annotation only; no Rust behavior or ABI.
- `DenseHashSet2` / `DenseHashMap2` constructor `explicit` removal (`Common/include/Luau/DenseHash2.h`): C++ implicit-conversion surface only. Rust exposes `new()` and `with_buckets()` explicitly, so there is no equivalent implicit constructor. The behavioral `DenseHash2::hash` assertion removal was ported.
- Coverage/counter declaration placement and `lua_Debug` comment (`VM/include/lua.h`): declaration ordering/comment-only; Rust symbols and layouts are unchanged.
- `newgcoblock` / `freegcoblock` force-inline annotations (`VM/src/lmem.cpp`; ledger names `newblock` / `freeblock`): optimizer hint only; the Rust allocator behavior is unchanged.

## Vector rows satisfied by landed generic representation

The following C++ hunks select `LUA_VECTOR_TYPE`, `condvector4`, or precision-dispatched math. Their Rust twins were verified after `LuaVectorType` became the feature-selected `f32`/`f64` alias and `vvalue`/`setvvalue` became representation-aware. They need no additional symbol-local edit beyond the already-landed shared representation/caller changes:

- `VM/src/lnumutils.h`: the double overloads and precision dispatch for vector equality, NaN detection, sign, clamp, and lerp. Rust's `luai_veceq`, `luai_vecisnan`, `luaui_signf`, `luaui_clampf`, and `luai_lerpf` operate on `LuaVectorType` and compile for both features.
- `VM/src/lbuiltins.cpp`: `luauF_vectormagnitude`, `luauF_vectordot`, and the portions of normalize/cross/floor/ceil/abs/sign/clamp/min/max/lerp whose only delta is the selected component type or selected scalar math. Their Rust functions consume `LuaVectorType`; functions with allocation or cast changes were edited directly.
- `VM/src/lveclib.cpp`: `vector_magnitude`, `vector_dot`, `vector_angle`, `vector_floor`, `vector_ceil`, `vector_abs`, `vector_sign`, `vector_clamp`, `vector_max`, `vector_index`, `createmetatable`, and `luaopen_vector`. These either inherit `LuaVectorType` or have no representation-sensitive body change. `vector_create`, normalize, cross, min, and lerp were edited directly.
- `VM/src/lvmexecute.cpp`: arithmetic opcode cases whose C hunk only changes a vector temporary/pointer from `float` to `LUA_VECTOR_TYPE` (`LOP_ADD`, `SUB`, `MUL`, `DIV`, `IDIV`, `MULK`, `DIVK`, `IDIVK`, `UNM`, and `DIVRK`). The translated interpreter uses the feature-selected alias. `LOP_GETTABLEKS`, backedge GC checks, and state-aware vector stores were edited directly.
- `VM/src/lvmutils.cpp`: portions of `luaV_doarithimpl` whose only change is the component type. The Rust twin is generic over `LuaVectorType`; vector allocation/store paths were edited directly.

## Other verified no-op rows

- `luaT_typenames` is not a no-op: both `luaT_init` and the Rust API's duplicated `lua_typename` mapping were updated for the double-vector tag position.
- `BcVmConst::operator==` was already supplied as the Rust `PartialEq` twin while the float/double graph representation landed; no separate method file is required.
- `Compiler` vector constant switch rows that share a Rust exhaustive match landed with the constant representation commit; no duplicate per-C++-overload wrapper was added.
- The stale `luaC_initobj` declaration has no remaining Rust match (`rg 'luaC_initobj|lua_c_initobj' vendor/luaur-vm-0.1.8` is empty), so there was nothing further to delete in this rung.
