# Luau fork rung 6 verified no-op dispositions

Diff: `6e9b580e..e8ae48c4` (official Luau 0.729 to 0.730)

## Formatting, include, and line-ending-only hunks

| C++ symbol | Rust twin or disposition | Verification |
|---|---|---|
| `AstStatLocalFunction` declaration and constructor | `vendor/luaur-ast-0.1.8/src/records/ast_stat_local_function.rs`; `vendor/luaur-ast-0.1.8/src/methods/ast_stat_local_function_ast_stat_local_function.rs` | C++ only collapses signatures to one line. The rung-3/4 `constKeywordBegin` field and five-argument constructor are present. |
| `LuauNoDuplicateBinaryPrefix`, `LuauTrackPrefixLocal` declarations | No Rust source edit | C++ swaps declaration order only; neither flag nor its behavior changes. |
| `Parser::parseLocal` | `vendor/luaur-ast-0.1.8/src/methods/parser_parse_local.rs` | C++ only reformats the unchanged `CstStatLocal` construction/assignment. |
| `BytecodeValidation.h` prologue | No translated header | Include/pragma ordering only; the duplicate C++ `<algorithm>` include has no Rust effect. |
| `VCONST` in `BytecodeBuilder::validateInstructions` | `vendor/luaur-bytecode-0.1.8/src/methods/bytecode_builder_validate_instructions.rs` | Ternary expression is unchanged; only C++ macro wrapping changes. |
| `DenseHashTable2::getBucket` | `vendor/luaur-common-0.1.8/src/methods/dense_hash_table2_dense_hash2.rs` | C++ adds braces around the same empty-bucket return. |
| `ConstantVisitor::visit(AstStatLocal*)` | `vendor/luaur-compiler-0.1.8/src/records/constant_visitor.rs` | Eligibility condition is only rewrapped. |
| compiler `foldConstants` | `vendor/luaur-compiler-0.1.8/src/functions/fold_constants.rs` | Aggregate initialization arguments and order are unchanged. |
| `JitInliner::createInlinedProto` | Untranslated `Inliner/` subsystem | Trailing whitespace removal only. |
| `lua_getuserdataname` | No existing Rust translation | CRLF normalization only; public API statements are unchanged. |
| `luaL_checkudatatagged` | No existing Rust translation | Line-ending normalization on the unchanged API call only. |
| `lmod`, `ceillog2` | `vendor/luaur-vm-0.1.8/src/macros/lmod.rs`; `vendor/luaur-vm-0.1.8/src/macros/ceillog_2.rs` | Whitespace around subtraction only. |
| `VM_ASSERT_PC` | `vendor/luaur-vm-0.1.8/src/macros/vm_assert_pc.rs` | C++ assertion wrapping only; interpreter behavior is unchanged. |
| `luau_callhook` | `vendor/luaur-vm-0.1.8/src/functions/luau_callhook.rs` | Assignment wrapping only. |
| `LOP_DUPCLOSURE` | `vendor/luaur-vm-0.1.8/src/functions/luau_execute.rs` | Conditional allocation wrapping only. |

## Satisfied without a file-local Rust edit

| C++ symbol | Rust twin | Verification |
|---|---|---|
| `detail::countTrailingZeroes` | Rust primitive use in `vendor/luaur-common-0.1.8/src/functions/count_trailing_zeroes.rs` | Rust's `trailing_zeros` already supplies the corrected portable behavior; the C++ MSVC/fallback typo does not translate. |
| `traverseclass`, `traverseobject` | `vendor/luaur-vm-0.1.8/src/functions/traverseclass.rs`; `vendor/luaur-vm-0.1.8/src/functions/traverseobject.rs` | Loop indices infer `u32` from the atomically changed class/object count fields. |
| `validateclass`, `validateobject` | `vendor/luaur-vm-0.1.8/src/functions/validateclass.rs`; `vendor/luaur-vm-0.1.8/src/functions/validateobject.rs` | Loop indices infer `u32` from the changed record fields. |
| `dumpclass`, `enumclass`, `enumobject` | `vendor/luaur-vm-0.1.8/src/functions/dumpclass.rs`; `vendor/luaur-vm-0.1.8/src/functions/enumclass.rs`; `vendor/luaur-vm-0.1.8/src/functions/enumobject.rs` | Rust ranges and subtraction now infer unsigned types from `LuauClass`; no separate cast or local declaration exists to edit. |

The previously listed prerequisites are all present: `AstStatLocalFunction`
has `constKeywordBegin`, bytecode target/max were at 7/12 before this rung's
target-9 change, and the rung-5 SCCP graph/lattice plus `DenseHash2` substrate
was available before the rung-6 evaluator/driver landed.
