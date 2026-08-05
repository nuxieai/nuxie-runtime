# Luau fork rung 7 skipped/blocked rows

## Runtime bytecode inliner (intentionally deferred)

The vendored crate family contains the compile-time `BytecodeGraph`, call inliner, and SCCP, but it does not translate the C++ runtime/JIT `Inliner/` component or its VM-backed constant evaluator. Per the standing lane directive, these rows remain deferred as one dependency unit:

- `Inliner/src/JitInliner.cpp`: `LuauBytecodeFold`, `createInlinedProto`, `hasFoldableConstants`, and `onInlineFunction`.
- `Inliner/src/RuntimeBytecodeBuilder.h`: `RuntimeBytecodeBuilder::constTypeToTT`, `dumpConstant`, and `emitCode`.
- `Inliner/src/TValueVmConstImpl.h`: `kDefaultBackingSize`, `TempTValueBacking` and all of its constructor/destructor/allocation methods, plus `TValueVmConstImpl` and its constructor.
- `Inliner/src/TValueVmConstImpl.cpp`: all `TValueVmConstImpl` behavior (`evaluate`, `falsey`, comparison/equality overloads, `makeNil`, `makeImm`, `asImm`, `isOrderable`, `kindEquals`, `isArithmeticConstant`, and `asNumber`).

`FFlag::LuauBytecodeFold` is nevertheless registered and pinned false. The landed compile-time SCCP/BcFunction changes are independently exercised by `luaur-bytecode`; no runtime post-inline SCCP entry point was invented.

## Require subsystem (intentionally deferred)

The pinned rive runtime supplies its own require integration and luaur has no translation of Luau's standalone `Require/` navigator. These rows cannot be ported without first translating that subsystem:

- Removal/hardwiring of `DFFlag::LuauRequireAliasOverrideOrderFix` and `FFlag::LuauRequireResolveAliasNullCheck`.
- `DFFlag::LuauSelfIsSelfAndAlwaysSelf` behavior in `Navigator::navigateImpl`.
- The unconditional null check and reset-to-requirer ordering in `Navigator::navigateToAndPopulateConfig`.

The new `LuauSelfIsSelfAndAlwaysSelf` flag is registered and pinned false as required by the rung policy. The two retired Require flags remain in the shared registry until the missing kept-side navigator behavior can land atomically; deleting inert registry entries alone would not port the upstream behavior.

## Central fork record

`docs/luau-fork.md` requires a central rung record, but this writer lane explicitly prohibits changes under `docs/`. The orchestrator must update that record outside this lane.
