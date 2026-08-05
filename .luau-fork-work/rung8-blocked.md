# Rung 8 deferred / blocked rows

Range: `f8ca77ac..decb2d05`.

## Config subsystem

Deferred because this repository vendors no `luaur-config` source crate. A grep of `vendor/` and `crates/` found no translated consumer of the new bytecode extraction API or its unconditional helpers. The new flag itself is registered and pinned false in `luaur-common`.

- `load` / `loadFromSource`
- `loadFromBytecode`
- `executeAndExtractConfig`
- `extractConfig`
- `parseLuauConfigTable`
- `extractLuauConfig`
- `extractLuauConfigFromBytecode`

## Require subsystem

Deferred under the standing rationale: no `luaur-require` crate or `luarequire_*` consumer exists in the vendored Rust set. The translated VM-side prerequisite `lua_usesexport` and `LPF_USES_EXPORT` are ported.

- `luarequire_Configuration::load`
- `ConfigStatus::PresentLuauBytecode`
- `NavigationContext::getAlias` / `RuntimeNavigationContext::getAlias`
- `luarequire_lockplaceholder`, `luarequire_populateplaceholder`, `luarequire_createplaceholder`
- `modulePlaceholdersKey`, `cyclicPlaceholderMetatableSentinel`, `cyclicPlaceholderProvidedKey`
- `isCached`, `invalidateModulePlaceholder`, `pushCyclicPlaceholderMetatable`
- `lockPlaceholder`, `createPlaceholder`, `populatePlaceholder`
- `kRequireStackValues`, `kRequireStackValues_DEPRECATED`
- `lua_requirecont`, `lua_requireinternal`

## Inliner / JIT subsystem

Deferred because `Inliner/` has no vendored Rust translation. Engine-side bytecode graph prerequisites in translated crates are ported (`CallInliner::migrateInstructions` and the `emitBytecode` dead-PC sentinel).

- `buildGraphFromProto`
- `emitCode`
- `onInlineFunction`
- `RuntimeBytecodeBuilder::dumpConstant`
- `TValueVmConstImpl::evaluate`

## Earlier-rung prerequisite mismatch discovered

The rung-7 class foundation is sufficient for rung-8 inheritance: an end-to-end runtime probe verified that `Child.live` resolves an inherited parent method. However, the active Rust twin of `Compiler::compileExprGlobal` is `methods/type_map_visitor_visit_types_alt_f.rs`, and it lacks rung 7's class-local lookup. Consequently a direct `Parent()` expression compiles as a global import and fails at runtime even though `Parent` is a valid local class value. This is outside `f8ca77ac..decb2d05`; it was not fixed in rung 8. The rung-8 superclass path is unaffected because `Compiler::getExprLocalReg` does contain the class-local lookup and emits `NEWCLASS Rchild Rparent`.
