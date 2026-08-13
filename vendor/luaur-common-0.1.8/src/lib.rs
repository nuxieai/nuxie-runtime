extern crate alloc;

#[cfg(test)]
mod dense_hash_tests;
#[cfg(test)]
mod dense_hash2_tests;
pub mod enums;
pub mod functions;
#[cfg(test)]
mod insertion_ordered_map_tests;
pub mod macros;
pub mod methods;
pub mod records;
#[cfg(test)]
mod string_utils_tests;
pub mod type_aliases;
#[cfg(test)]
mod vec_deque_tests;

/// Minimal libc surface for wasm. On `wasm32-unknown-unknown` (no libc) every
/// shim is needed; on libc-bearing wasm (e.g. `wasm32-wasip1`, used to run the
/// suite on a 32-bit-pointer platform) the allocator shims are gated off inside
/// the module so they don't clash with wasi-libc's, while the functions wasi
/// lacks (mmap stubs, etc.) are still provided.
#[cfg(target_arch = "wasm32")]
pub mod wasm_libc;

/// Pure-Rust `strtod` shim for wasm (no libc on `wasm32-unknown-unknown`). The
/// scanning core is unit-tested natively, so the module is also compiled under
/// `test`; only the `#[no_mangle]` C entry point is wasm-gated.
#[cfg(any(target_arch = "wasm32", test))]
pub mod strtod_shim;

// C++ exposes this at namespace scope; codegen_assert! and friends reference
// `luaur_common::assert_call_handler` directly.
pub use functions::assert_call_handler::assert_call_handler;
pub use records::f_value::set_luau_bool_flags;

use records::f_value::FValue;

/// Apply the complete supported Luau flag profile through a caller-selected
/// mutation strategy. Keeping the generated flag catalogue in one place makes
/// global startup configuration and thread-local compiler scopes identical.
#[allow(non_snake_case)]
fn apply_all_flags(value: bool, mut apply: impl FnMut(&'static FValue<bool>, bool)) {
    apply(&FFlag::FixMathNoisePrecision, value);
    apply(&FFlag::LuauAddRecursionCounterToNonStrictTypeChecker, value);
    apply(&FFlag::LuauAllowGlobalDeclarationToBeCalledClass, value);
    apply(&FFlag::LuauAlsoInstantiateInferredArguments, value);
    apply(&FFlag::LuauAutoStack, false);
    apply(&FFlag::LuauAutocompleteConst, value);
    apply(&FFlag::LuauAutocompleteExport, value);
    apply(&FFlag::LuauAutocompleteStringSingletonIntersection, value);
    apply(&FFlag::LuauBidirectionalInferenceBetterUnionHandling, value);
    apply(&FFlag::LuauCallFeedback, value);
    apply(&FFlag::LuauBackedgeHeapCheck, false);
    apply(&FFlag::LuauBytecodeCostModel, false);
    apply(&FFlag::LuauBytecodeFold, false);
    apply(&FFlag::LuauCIProto, false);
    apply(&FFlag::LuauCheckFunctionStatementTypes, value);
    apply(&FFlag::LuauCloneTableFix, false);
    apply(&FFlag::LuauCodeGenCallWrapperEmitInst, value);
    apply(&FFlag::LuauCodegenBufferInteger, value);
    apply(&FFlag::LuauCodegenDsePtrStoreTagCheck, value);
    apply(&FFlag::LuauCodegenDseRestoreHints, value);
    apply(&FFlag::LuauCodegenExtraTableOpts, value);
    apply(&FFlag::LuauCodegenFixBufferLenCheck, value);
    apply(&FFlag::LuauCodegenForwardRematerialize, value);
    apply(&FFlag::LuauCodegenFreeBlocks, value);
    apply(&FFlag::LuauCodegenInteger2, value);
    apply(&FFlag::LuauCodegenIntegerArg3Fix, value);
    apply(&FFlag::LuauCodegenIntegerFastcall2k, value);
    apply(&FFlag::LuauCodegenLinearSetupEntryState3, value);
    apply(&FFlag::LuauCodegenLoadPropagateOrigin, value);
    apply(&FFlag::LuauCodegenNopPadding, value);
    apply(&FFlag::LuauCodegenProtectData, value);
    apply(&FFlag::LuauCodegenRecordAllBlockExitInfo, value);
    apply(&FFlag::LuauCodegenRegTag2, value);
    apply(&FFlag::LuauCodegenSuggestArgumentRegisterX64, value);
    apply(&FFlag::LuauCodegenVmExitSync, value);
    apply(&FFlag::LuauCodegenVmExitSyncFix, value);
    apply(&FFlag::LuauCompileStringInterpTargetTop, value);
    apply(&FFlag::LuauCompileIifeInline, false);
    apply(&FFlag::LuauConcatDoesntAlwaysReturnString, value);
    apply(&FFlag::LuauConstraintGraph, value);
    apply(&FFlag::LuauCostModel, false);
    apply(&FFlag::LuauCompileEmitVectorDouble, false);
    apply(&FFlag::LuauDirectFieldGet, value);
    apply(&FFlag::LuauDisallowExternClassInTypeDefinitions, false);
    apply(&FFlag::LuauDisallowRedefiningBuiltinTypes, value);
    apply(&FFlag::LuauEmitCallFeedback, value);
    apply(&FFlag::LuauExplicitTypeInstantiationSupport, value);
    // Experimental "export values" syntax is intentionally NOT enabled here: it
    // is incomplete in this port — a closure that captures an exported local
    // mis-compiles the upvalue register (the C++ reference handles it), so it can
    // produce out-of-range bytecode. Keep it off (default false) until the
    // export-table/closure codegen is fixed. Tests that exercise it set the flag
    // explicitly via a scoped override.
    apply(&FFlag::LuauExportValueSyntax, false);
    apply(&FFlag::LuauExportValueTypecheck, false);
    apply(&FFlag::LuauExternTypesNormalizeWithShapes, value);
    apply(&FFlag::LuauFixIndexerSubtypingOrdering, value);
    apply(&FFlag::LuauFixPropReadsOnMetatableTypes, value);
    apply(&FFlag::LuauInstantiateFunctionTypeBeforePush, value);
    apply(&FFlag::LuauInstantiateInSubtyping, value);
    apply(&FFlag::LuauInstantiationUsesPolarity, value);
    apply(&FFlag::LuauIntegerBufferFastcalls, false);
    apply(&FFlag::LuauIntegerFastcalls, false);
    apply(&FFlag::LuauIntegerLibrary, value);
    apply(&FFlag::LuauIntegerType2, value);
    apply(&FFlag::LuauIterativeInstantiationQueuer, value);
    apply(&FFlag::LuauKnowsTheDataModel3, value);
    apply(&FFlag::LuauLValueCompoundAssignmentVisitLhs, value);
    apply(&FFlag::LuauLimitUnificationRecursion, value);
    apply(&FFlag::LuauMathRoundNegZero, false);
    apply(&FFlag::LuauGcTraceUdata, false);
    apply(&FFlag::LuauNativeCodeTargetCheck, value);
    apply(&FFlag::LuauNonStrictModeUseErrorSupressingTag, value);
    apply(&FFlag::LuauNoDuplicateBinaryPrefix, false);
    apply(&FFlag::LuauOccursCheckForAllBindings, value);
    apply(&FFlag::LuauOptimizeExportTable, false);
    apply(&FFlag::LuauPropagateFreeTypesIntoUnionAndIntersectionBounds, value);
    apply(&FFlag::LuauPropagateTypeAnnotationsInForInLoops, value);
    apply(&FFlag::LuauPropertyModifierMismatchErrors, value);
    apply(&FFlag::LuauPromoteProto, false);
    apply(&FFlag::LuauReadOnlyIndexers, value);
    apply(&FFlag::LuauRefineNilFromTableIndexerResultType, value);
    apply(&FFlag::LuauRemoveConstraintSolverEmplace, value);
    apply(&FFlag::LuauReplacerIsSolverAgnostic, value);
    apply(&FFlag::LuauRequireResolveAliasNullCheck, value);
    apply(&FFlag::LuauRbsConfigAliasResolution, false);
    apply(&FFlag::LuauSilenceDynamicFormatStringErrors, value);
    apply(&FFlag::LuauSolverV2, value);
    apply(&FFlag::LuauStoreConstKeywordBegin, false);
    apply(&FFlag::LuauSubtypingMissingPropertiesAsNil, value);
    apply(&FFlag::LuauSubtypingTablesHasBetterErrorSuppression, value);
    apply(&FFlag::LuauTrackPrefixLocal, false);
    apply(&FFlag::LuauTableFreezeCheckIsSubtype, value);
    apply(&FFlag::LuauTidyTypePrototyping, value);
    apply(&FFlag::LuauTransitiveSubtyping, value);
    apply(&FFlag::LuauTweakAccessViolationReporting, value);
    apply(&FFlag::LuauTypeFunctionRobustness, value);
    apply(&FFlag::LuauTypeFunctionSerializeArgNames, value);
    apply(&FFlag::LuauTypeFunctionStructuredErrors, value);
    apply(&FFlag::LuauTypeFunctionSupportsFrozen, value);
    apply(&FFlag::LuauUdataDirectAccess6, value);
    apply(&FFlag::LuauUdataMetatablePinned, false);
    apply(&FFlag::LuauUdtfTypeIsSubtypeOf, value);
    apply(&FFlag::LuauUseNativeStackGuard, value);
    apply(&FFlag::LuauVirtualBcBuilder, false);
    apply(&FFlag::LuauVisitCallTypeArgsInDfg, value);
    apply(&FFlag::LuauYieldIter2, value);
    apply(&FFlag::LuauXpcallFixMessageYieldPath, false);
    apply(&FFlag::LuauManagedDebugNames, false);
    apply(&DFFlag::LuauGcMarkUdataAccess, false);
    apply(&DFFlag::LuauGcTableStepFix, false);
    apply(&DFFlag::LuauSelfIsSelfAndAlwaysSelf, false);
}

/// C++ CLI `setLuauFlagsDefault(value)` analog: set every non-Debug FFlag.
/// This remains the process-global startup API used before worker threads run.
pub fn set_all_flags(value: bool) {
    apply_all_flags(value, |flag, enabled| flag.set(enabled));
}

/// Thread-local Luau flag profile for work that must compile with a different
/// compatibility floor while other Rust test threads execute runtime code.
pub struct ScopedAllFlags {
    flags: Vec<&'static FValue<bool>>,
}

impl ScopedAllFlags {
    pub fn enter(value: bool) -> Self {
        let mut flags = Vec::new();
        apply_all_flags(value, |flag, enabled| {
            flag.push_test_override(enabled);
            flags.push(flag);
        });
        Self { flags }
    }
}

impl Drop for ScopedAllFlags {
    fn drop(&mut self) {
        for flag in self.flags.iter().rev() {
            flag.pop_test_override();
        }
    }
}

/// FastFlag namespace `FFlag::` — static (non-dynamic) bool flags. Definitions
/// from `LUAU_FASTFLAGVARIABLE(...)` across this crate's sources are collected
/// here so C++ reads `FFlag::Name` map to `crate::FFlag::Name.get()`. (Rust
/// modules are not open like C++ namespaces, so the per-crate namespace module
/// is the aggregation point — see `crate::macros::luau_fastflagvariable`.)
#[allow(non_snake_case)]
pub mod FFlag {
    // CodeGen/src/IrRegAllocA64.cpp
    crate::LUAU_FASTFLAGVARIABLE!(DebugCodegenChaosA64);
    // CodeGen/src/CodeGen.cpp
    crate::LUAU_FASTFLAGVARIABLE!(DebugCodegenOptSize);
    // CodeGen/src/CodeGen.cpp
    crate::LUAU_FASTFLAGVARIABLE!(DebugCodegenSkipNumbering);
    // Analysis/src/FragmentAutocomplete.cpp
    crate::LUAU_FASTFLAGVARIABLE!(DebugLogFragmentsFromAutocomplete);
    // CodeGen/src/OptimizeConstProp.cpp
    crate::LUAU_FASTFLAGVARIABLE!(DebugLuauAbortingChecks);
    // Analysis/src/Frontend.cpp
    crate::LUAU_FASTFLAGVARIABLE!(DebugLuauAlwaysShowConstraintSolvingIncomplete);
    // Analysis/src/ConstraintSolver.cpp
    crate::LUAU_FASTFLAGVARIABLE!(DebugLuauAssertOnForcedConstraint);
    // Analysis/src/Normalize.cpp
    crate::LUAU_FASTFLAGVARIABLE!(DebugLuauCheckNormalizeInvariant);
    // Analysis/src/DumpCFG.cpp
    crate::LUAU_FASTFLAGVARIABLE!(DebugLuauDumpCFGJson);
    // Analysis/src/Frontend.cpp
    crate::LUAU_FASTFLAGVARIABLE!(DebugLuauForbidInternalTypes);
    // tests/Fixture.cpp
    crate::LUAU_FASTFLAGVARIABLE!(DebugLuauForceAllNewSolverTests);
    // tests/Fixture.cpp
    crate::LUAU_FASTFLAGVARIABLE!(DebugLuauForceAllOldSolverTests);
    // Analysis/src/Frontend.cpp
    crate::LUAU_FASTFLAGVARIABLE!(DebugLuauForceNonStrictMode);
    // Analysis/src/Frontend.cpp
    crate::LUAU_FASTFLAGVARIABLE!(DebugLuauForceOldSolver);
    // Analysis/src/Frontend.cpp
    crate::LUAU_FASTFLAGVARIABLE!(DebugLuauForceStrictMode);
    // Analysis/src/TypeArena.cpp
    crate::LUAU_FASTFLAGVARIABLE!(DebugLuauFreezeArena);
    // Analysis/src/TypeInfer.cpp
    crate::LUAU_FASTFLAGVARIABLE!(DebugLuauFreezeDuringUnification);
    // Analysis/src/ConstraintSolver.cpp
    crate::LUAU_FASTFLAGVARIABLE!(DebugLuauLogBindings);
    // Analysis/src/DumpCFG.cpp
    crate::LUAU_FASTFLAGVARIABLE!(DebugLuauLogCFG);
    // Analysis/src/ConstraintSolver.cpp
    crate::LUAU_FASTFLAGVARIABLE!(DebugLuauLogSolver);
    // Analysis/src/Frontend.cpp
    crate::LUAU_FASTFLAGVARIABLE!(DebugLuauLogSolverToJson);
    // Analysis/src/Frontend.cpp
    crate::LUAU_FASTFLAGVARIABLE!(DebugLuauLogSolverToJsonFile);
    // Analysis/src/TypeFunction.cpp
    crate::LUAU_FASTFLAGVARIABLE!(DebugLuauLogTypeFamilies);
    // Analysis/src/TypeInfer.cpp
    crate::LUAU_FASTFLAGVARIABLE!(DebugLuauMagicTypes);
    // Analysis/src/AutocompleteCore.cpp
    crate::LUAU_FASTFLAGVARIABLE!(DebugLuauMagicVariableNames);
    // Ast/src/Parser.cpp
    crate::LUAU_FASTFLAGVARIABLE!(DebugLuauNoInline);
    // Analysis/src/Subtyping.cpp
    crate::LUAU_FASTFLAGVARIABLE!(DebugLuauSubtypingCheckPathValidity);
    // Common/src/TimeTrace.cpp
    crate::LUAU_FASTFLAGVARIABLE!(DebugLuauTimeTracing);
    // Analysis/src/ToString.cpp
    crate::LUAU_FASTFLAGVARIABLE!(DebugLuauToStringNoLexicalSort);
    // Ast/src/Parser.cpp
    crate::LUAU_FASTFLAGVARIABLE!(DebugLuauUserDefinedClasses);
    // VM/src/lvmexecute.cpp
    crate::LUAU_FASTFLAGVARIABLE!(DebugLuauUserDefinedClassesRuntime);
    // VM/src/lmathlib.cpp
    crate::LUAU_FASTFLAGVARIABLE!(FixMathNoisePrecision);
    // Analysis/src/NonStrictTypeChecker.cpp
    crate::LUAU_FASTFLAGVARIABLE!(LuauAddRecursionCounterToNonStrictTypeChecker);
    // Ast/src/Parser.cpp
    crate::LUAU_FASTFLAGVARIABLE!(LuauAllowGlobalDeclarationToBeCalledClass);
    // Analysis/src/ConstraintSolver.cpp
    crate::LUAU_FASTFLAGVARIABLE!(LuauAlsoInstantiateInferredArguments);
    // VM/src/lapi.cpp
    crate::LUAU_FASTFLAGVARIABLE!(LuauAutoStack);
    // Analysis/src/AutocompleteCore.cpp
    crate::LUAU_FASTFLAGVARIABLE!(LuauAutocompleteConst);
    // Analysis/src/AutocompleteCore.cpp
    crate::LUAU_FASTFLAGVARIABLE!(LuauAutocompleteExport);
    // Analysis/src/AutocompleteCore.cpp
    crate::LUAU_FASTFLAGVARIABLE!(LuauAutocompleteStringSingletonIntersection);
    // Analysis/src/ExpectedTypeVisitor.cpp
    crate::LUAU_FASTFLAGVARIABLE!(LuauBidirectionalInferenceBetterUnionHandling);
    // VM/src/lvmexecute.cpp
    crate::LUAU_FASTFLAGVARIABLE!(LuauCallFeedback);
    // VM/src/lvmexecute.cpp
    crate::LUAU_FASTFLAGVARIABLE!(LuauBackedgeHeapCheck);
    crate::LUAU_FLAGVERSION!(LuauBackedgeHeapCheck, 2);
    // Bytecode/src/BytecodeBuilder.cpp
    crate::LUAU_FASTFLAGVARIABLE!(LuauBytecodeCostModel);
    crate::LUAU_FLAGVERSION!(LuauBytecodeCostModel, 2);
    // Inliner/src/JitInliner.cpp
    crate::LUAU_FASTFLAGVARIABLE!(LuauBytecodeFold);
    // VM/src/lvmexecute.cpp
    crate::LUAU_FASTFLAGVARIABLE!(LuauCIProto);
    // Analysis/src/TypeChecker2.cpp
    crate::LUAU_FASTFLAGVARIABLE!(LuauCheckFunctionStatementTypes);
    // VM/src/lapi.cpp
    crate::LUAU_FASTFLAGVARIABLE!(LuauCloneTableFix);
    // CodeGen/src/EmitInstructionX64.cpp
    crate::LUAU_FASTFLAGVARIABLE!(LuauCodeGenCallWrapperEmitInst);
    // CodeGen/src/IrTranslateBuiltins.cpp
    crate::LUAU_FASTFLAGVARIABLE!(LuauCodegenBufferInteger);
    // CodeGen/src/OptimizeDeadStore.cpp
    crate::LUAU_FASTFLAGVARIABLE!(LuauCodegenDsePtrStoreTagCheck);
    // CodeGen/src/OptimizeDeadStore.cpp
    crate::LUAU_FASTFLAGVARIABLE!(LuauCodegenDseRestoreHints);
    // CodeGen/src/OptimizeConstProp.cpp
    crate::LUAU_FASTFLAGVARIABLE!(LuauCodegenExtraTableOpts);
    // CodeGen/src/IrLoweringA64.cpp
    crate::LUAU_FASTFLAGVARIABLE!(LuauCodegenFixBufferLenCheck);
    // CodeGen/src/IrValueLocationTracking.cpp
    crate::LUAU_FASTFLAGVARIABLE!(LuauCodegenForwardRematerialize);
    // CodeGen/src/CodeAllocator.cpp
    crate::LUAU_FASTFLAGVARIABLE!(LuauCodegenFreeBlocks);
    // CodeGen/src/CodeGen.cpp
    crate::LUAU_FASTFLAGVARIABLE!(LuauCodegenInteger2);
    // CodeGen/src/IrTranslateBuiltins.cpp
    crate::LUAU_FASTFLAGVARIABLE!(LuauCodegenIntegerArg3Fix);
    // CodeGen/src/IrTranslation.cpp
    crate::LUAU_FASTFLAGVARIABLE!(LuauCodegenIntegerFastcall2k);
    // CodeGen/src/OptimizeConstProp.cpp
    crate::LUAU_FASTFLAGVARIABLE!(LuauCodegenLinearSetupEntryState3);
    // CodeGen/src/OptimizeConstProp.cpp
    crate::LUAU_FASTFLAGVARIABLE!(LuauCodegenLoadPropagateOrigin);
    // CodeGen/src/CodeGen.cpp
    crate::LUAU_FASTFLAGVARIABLE!(LuauCodegenNopPadding);
    // CodeGen/src/CodeAllocator.cpp
    crate::LUAU_FASTFLAGVARIABLE!(LuauCodegenProtectData);
    // CodeGen/src/OptimizeConstProp.cpp
    crate::LUAU_FASTFLAGVARIABLE!(LuauCodegenRecordAllBlockExitInfo);
    // CodeGen/src/BytecodeAnalysis.cpp
    crate::LUAU_FASTFLAGVARIABLE!(LuauCodegenRegTag2);
    // CodeGen/src/CodeGenX64.cpp
    crate::LUAU_FASTFLAGVARIABLE!(LuauCodegenSuggestArgumentRegisterX64);
    // CodeGen/src/IrAnalysis.cpp
    crate::LUAU_FASTFLAGVARIABLE!(LuauCodegenVmExitSync);
    // CodeGen/src/OptimizeDeadStore.cpp
    crate::LUAU_FASTFLAGVARIABLE!(LuauCodegenVmExitSyncFix);
    // Bytecode/src/BytecodeBuilder.cpp
    crate::LUAU_FASTFLAGVARIABLE!(LuauCompileEmitVectorDouble);
    crate::LUAU_FLAGVERSION!(LuauCompileEmitVectorDouble, 2);
    // Compiler/src/Compiler.cpp
    crate::LUAU_FASTFLAGVARIABLE!(LuauCompileStringInterpTargetTop);
    // Compiler/src/Compiler.cpp
    crate::LUAU_FASTFLAGVARIABLE!(LuauCompileIifeInline);
    // Analysis/src/BuiltinTypeFunctions.cpp
    crate::LUAU_FASTFLAGVARIABLE!(LuauConcatDoesntAlwaysReturnString);
    // Analysis/src/Constraint.cpp
    crate::LUAU_FASTFLAGVARIABLE!(LuauConstraintGraph);
    // Bytecode/src/BytecodeGraph.cpp; VM/src/lvmload.cpp
    crate::LUAU_FASTFLAGVARIABLE!(LuauCostModel);
    // VM/src/lvmexecute.cpp
    crate::LUAU_FASTFLAGVARIABLE!(LuauDirectFieldGet);
    crate::LUAU_FLAGVERSION!(LuauDirectFieldGet, 3);
    // Ast/src/Parser.cpp
    crate::LUAU_FASTFLAGVARIABLE!(LuauDisallowExternClassInTypeDefinitions);
    // Analysis/src/ConstraintGenerator.cpp
    crate::LUAU_FASTFLAGVARIABLE!(LuauDisallowRedefiningBuiltinTypes);
    // Compiler/src/Compiler.cpp
    crate::LUAU_FASTFLAGVARIABLE!(LuauEmitCallFeedback);
    // Analysis/src/TypeInfer.cpp
    crate::LUAU_FASTFLAGVARIABLE!(LuauExplicitTypeInstantiationSupport);
    // Ast/src/Parser.cpp
    crate::LUAU_FASTFLAGVARIABLE!(LuauExportValueSyntax);
    crate::LUAU_FLAGVERSION!(LuauExportValueSyntax, 4);
    // Analysis/src/Frontend.cpp
    crate::LUAU_FASTFLAGVARIABLE!(LuauExportValueTypecheck);
    // Analysis/src/Normalize.cpp
    crate::LUAU_FASTFLAGVARIABLE!(LuauExternTypesNormalizeWithShapes);
    // Analysis/src/Unifier.cpp
    crate::LUAU_FASTFLAGVARIABLE!(LuauFixIndexerSubtypingOrdering);
    // Analysis/src/ConstraintSolver.cpp
    crate::LUAU_FASTFLAGVARIABLE!(LuauFixPropReadsOnMetatableTypes);
    // Analysis/src/ConstraintSolver.cpp
    crate::LUAU_FASTFLAGVARIABLE!(LuauInstantiateFunctionTypeBeforePush);
    // Analysis/src/Unifier.cpp
    crate::LUAU_FASTFLAGVARIABLE!(LuauInstantiateInSubtyping);
    // Analysis/src/Instantiation.cpp
    crate::LUAU_FASTFLAGVARIABLE!(LuauInstantiationUsesPolarity);
    // Compiler/src/Builtins.cpp
    crate::LUAU_FASTFLAGVARIABLE!(LuauIntegerBufferFastcalls);
    // Compiler/src/Builtins.cpp
    crate::LUAU_FASTFLAGVARIABLE!(LuauIntegerFastcalls);
    // VM/src/lintlib.cpp
    crate::LUAU_FASTFLAGVARIABLE!(LuauIntegerLibrary);
    // Ast/src/Parser.cpp
    crate::LUAU_FASTFLAGVARIABLE!(LuauIntegerType2);
    // Analysis/src/ConstraintSolver.cpp
    crate::LUAU_FASTFLAGVARIABLE!(LuauIterativeInstantiationQueuer);
    // Analysis/src/Frontend.cpp
    crate::LUAU_FASTFLAGVARIABLE!(LuauKnowsTheDataModel3);
    // Analysis/src/TypeChecker2.cpp
    crate::LUAU_FASTFLAGVARIABLE!(LuauLValueCompoundAssignmentVisitLhs);
    // Analysis/src/Unifier2.cpp
    crate::LUAU_FASTFLAGVARIABLE!(LuauLimitUnificationRecursion);
    // VM/src/lapi.cpp
    crate::LUAU_FASTFLAGVARIABLE!(LuauManagedDebugNames);
    // VM/src/lbuiltins.cpp
    crate::LUAU_FASTFLAGVARIABLE!(LuauMathRoundNegZero);
    // VM/src/lgc.cpp
    crate::LUAU_FASTFLAGVARIABLE!(LuauGcTraceUdata);
    crate::LUAU_FLAGVERSION!(LuauGcTraceUdata, 2);
    // CodeGen/src/CodeGenUtils.cpp
    crate::LUAU_FASTFLAGVARIABLE!(LuauNativeCodeTargetCheck);
    // Analysis/src/NonStrictTypeChecker.cpp
    crate::LUAU_FASTFLAGVARIABLE!(LuauNonStrictModeUseErrorSupressingTag);
    // Ast/src/Parser.cpp
    crate::LUAU_FASTFLAGVARIABLE!(LuauNoDuplicateBinaryPrefix);
    // Analysis/src/ConstraintSolver.cpp
    crate::LUAU_FASTFLAGVARIABLE!(LuauOccursCheckForAllBindings);
    // Compiler/src/Compiler.cpp
    crate::LUAU_FASTFLAGVARIABLE!(LuauOptimizeExportTable);
    // Analysis/src/Unifier2.cpp
    crate::LUAU_FASTFLAGVARIABLE!(LuauPropagateFreeTypesIntoUnionAndIntersectionBounds);
    // Analysis/src/ConstraintGenerator.cpp
    crate::LUAU_FASTFLAGVARIABLE!(LuauPropagateTypeAnnotationsInForInLoops);
    // Analysis/src/TypeChecker2.cpp
    crate::LUAU_FASTFLAGVARIABLE!(LuauPropertyModifierMismatchErrors);
    // VM/src/lfunc.h
    crate::LUAU_FASTFLAGVARIABLE!(LuauPromoteProto);
    // Analysis/src/ConstraintGenerator.cpp
    crate::LUAU_FASTFLAGVARIABLE!(LuauReadOnlyIndexers);
    // Analysis/src/ConstraintSolver.cpp
    crate::LUAU_FASTFLAGVARIABLE!(LuauRefineNilFromTableIndexerResultType);
    // Analysis/src/ConstraintSolver.cpp
    crate::LUAU_FASTFLAGVARIABLE!(LuauRemoveConstraintSolverEmplace);
    // Analysis/src/Instantiation.cpp
    crate::LUAU_FASTFLAGVARIABLE!(LuauReplacerIsSolverAgnostic);
    // Require/src/RequireNavigator.cpp
    crate::LUAU_FASTFLAGVARIABLE!(LuauRequireResolveAliasNullCheck);
    // Config/src/LuauConfig.cpp
    crate::LUAU_FASTFLAGVARIABLE!(LuauRbsConfigAliasResolution);
    // Analysis/src/BuiltinDefinitions.cpp
    crate::LUAU_FASTFLAGVARIABLE!(LuauSilenceDynamicFormatStringErrors);
    // Ast/src/Parser.cpp
    crate::LUAU_FASTFLAGVARIABLE!(LuauSolverV2);
    // Analysis/src/Subtyping.cpp
    crate::LUAU_FASTFLAGVARIABLE!(LuauSubtypingMissingPropertiesAsNil);
    // Ast/src/Parser.cpp
    crate::LUAU_FASTFLAGVARIABLE!(LuauStoreConstKeywordBegin);
    // Analysis/src/Subtyping.cpp
    crate::LUAU_FASTFLAGVARIABLE!(LuauSubtypingTablesHasBetterErrorSuppression);
    // Ast/src/Parser.cpp
    crate::LUAU_FASTFLAGVARIABLE!(LuauTrackPrefixLocal);
    // Analysis/src/BuiltinDefinitions.cpp
    crate::LUAU_FASTFLAGVARIABLE!(LuauTableFreezeCheckIsSubtype);
    // Analysis/src/ConstraintGenerator.cpp
    crate::LUAU_FASTFLAGVARIABLE!(LuauTidyTypePrototyping);
    // Analysis/src/Unifier.cpp
    crate::LUAU_FASTFLAGVARIABLE!(LuauTransitiveSubtyping);
    // Analysis/src/Error.cpp
    crate::LUAU_FASTFLAGVARIABLE!(LuauTweakAccessViolationReporting);
    // Analysis/src/TypeFunctionRuntime.cpp
    crate::LUAU_FASTFLAGVARIABLE!(LuauTypeFunctionRobustness);
    // Analysis/src/TypeFunctionRuntime.cpp
    crate::LUAU_FASTFLAGVARIABLE!(LuauTypeFunctionSerializeArgNames);
    // Analysis/src/TypeFunctionRuntime.cpp
    crate::LUAU_FASTFLAGVARIABLE!(LuauTypeFunctionStructuredErrors);
    // Analysis/src/TypeFunctionRuntime.cpp
    crate::LUAU_FASTFLAGVARIABLE!(LuauTypeFunctionSupportsFrozen);
    // VM/src/lvmload.cpp
    crate::LUAU_FASTFLAGVARIABLE!(LuauUdataDirectAccess6);
    // VM/src/lgc.cpp
    crate::LUAU_FASTFLAGVARIABLE!(LuauUdataMetatablePinned);
    // Analysis/src/TypeFunctionRuntime.cpp
    crate::LUAU_FASTFLAGVARIABLE!(LuauUdtfTypeIsSubtypeOf);
    // Analysis/src/NativeStackGuard.cpp
    crate::LUAU_FASTFLAGVARIABLE!(LuauUseNativeStackGuard);
    // Bytecode/src/BytecodeBuilder.cpp
    crate::LUAU_FASTFLAGVARIABLE!(LuauVirtualBcBuilder);
    // Analysis/src/DataFlowGraph.cpp
    crate::LUAU_FASTFLAGVARIABLE!(LuauVisitCallTypeArgsInDfg);
    // VM/src/lvmexecute.cpp
    crate::LUAU_FASTFLAGVARIABLE!(LuauYieldIter2);
    // VM/src/ldo.cpp
    crate::LUAU_FASTFLAGVARIABLE!(LuauXpcallFixMessageYieldPath);
}

/// Static int FastFlags, mirroring `FFlag`. C++ collects every
/// `LUAU_FASTINTVARIABLE(...)` into `namespace FInt`; Rust modules aren't open,
/// so the consumers' flags are gathered here. Read as `FInt::Flag.get()`.
#[allow(non_snake_case)]
pub mod FInt {
    // CodeGen/src/CodeGen.cpp
    crate::LUAU_FASTINTVARIABLE!(CodegenHeuristicsBlockInstructionLimit, 65_536);
    // CodeGen/src/CodeGen.cpp
    crate::LUAU_FASTINTVARIABLE!(CodegenHeuristicsBlockLimit, 32_768);
    // CodeGen/src/CodeGen.cpp
    crate::LUAU_FASTINTVARIABLE!(CodegenHeuristicsInstructionLimit, 1_048_576);
    // CodeGen/src/CodeGenContext.cpp
    crate::LUAU_FASTINTVARIABLE!(LuauCodeGenBlockSize, 4 * 1024 * 1024);
    // CodeGen/src/CodeGenContext.cpp
    crate::LUAU_FASTINTVARIABLE!(LuauCodeGenMaxTotalSize, 256 * 1024 * 1024);
    // Analysis/src/Clone.cpp
    crate::LUAU_FASTINTVARIABLE!(LuauTypeCloneIterationLimit, 100_000);
    // Analysis/src/ToString.cpp
    crate::LUAU_FASTINTVARIABLE!(DebugLuauVerboseTypeNames, 0);
    // Analysis/src/TypeInfer.cpp
    crate::LUAU_FASTINTVARIABLE!(LuauCheckRecursionLimit, 300);
    // CodeGen/src/OptimizeConstProp.cpp
    crate::LUAU_FASTINTVARIABLE!(LuauCodeGenLiveSlotReuseLimit, 8);
    // CodeGen/src/OptimizeConstProp.cpp
    crate::LUAU_FASTINTVARIABLE!(LuauCodeGenMinLinearBlockPath, 3);
    // CodeGen/src/OptimizeConstProp.cpp
    crate::LUAU_FASTINTVARIABLE!(LuauCodeGenReuseSlotLimit, 64);
    // CodeGen/src/OptimizeConstProp.cpp
    crate::LUAU_FASTINTVARIABLE!(LuauCodeGenReuseUdataTagLimit, 64);
    // Compiler/src/Compiler.cpp
    crate::LUAU_FASTINTVARIABLE!(LuauCompileInlineDepth, 5);
    // Compiler/src/Compiler.cpp
    crate::LUAU_FASTINTVARIABLE!(LuauCompileInlineThreshold, 25);
    // Compiler/src/Compiler.cpp
    crate::LUAU_FASTINTVARIABLE!(LuauCompileInlineThresholdMaxBoost, 300);
    // Compiler/src/Compiler.cpp
    crate::LUAU_FASTINTVARIABLE!(LuauCompileLoopUnrollThreshold, 25);
    // Compiler/src/Compiler.cpp
    crate::LUAU_FASTINTVARIABLE!(LuauCompileLoopUnrollThresholdMaxBoost, 300);
    // Analysis/src/Generalization.cpp
    crate::LUAU_FASTINTVARIABLE!(LuauGenericCounterMaxDepth, 15);
    // Analysis/src/Generalization.cpp
    crate::LUAU_FASTINTVARIABLE!(LuauGenericCounterMaxSteps, 1500);
    // Analysis/src/Error.cpp
    crate::LUAU_FASTINTVARIABLE!(LuauIndentTypeMismatchMaxTypeLength, 10);
    // VM/src/lfunc.cpp
    crate::LUAU_FASTINTVARIABLE!(LuauInlineHitsThreshold, 32);
    // Inliner/src/JitInliner.cpp
    crate::LUAU_FASTINTVARIABLE!(LuauJitInlineSmallFunSize, 128);
    // Inliner/src/JitInliner.cpp
    crate::LUAU_FASTINTVARIABLE!(LuauJitInlineThreshold, 25);
    // Inliner/src/JitInliner.cpp
    crate::LUAU_FASTINTVARIABLE!(LuauJitInlineThresholdMaxBoost, 300);
    // Inliner/src/JitInliner.cpp
    crate::LUAU_FASTINTVARIABLE!(LuauJitInlineTooLongFunSize, 0xffff);
    // Analysis/src/NonStrictTypeChecker.cpp
    crate::LUAU_FASTINTVARIABLE!(LuauNonStrictTypeCheckerRecursionLimit, 300);
    // Analysis/src/Normalize.cpp
    crate::LUAU_FASTINTVARIABLE!(LuauNormalizeCacheLimit, 100000);
    // Analysis/src/Normalize.cpp
    crate::LUAU_FASTINTVARIABLE!(LuauNormalizerInitialFuel, 3000);
    // Ast/src/Parser.cpp
    crate::LUAU_FASTINTVARIABLE!(LuauParseErrorLimit, 100);
    // Analysis/src/ConstraintGenerator.cpp
    crate::LUAU_FASTINTVARIABLE!(LuauPrimitiveInferenceInTableLimit, 500);
    // Ast/src/Parser.cpp
    crate::LUAU_FASTINTVARIABLE!(LuauRecursionLimit, 1000);
    // Analysis/src/ConstraintSolver.cpp
    crate::LUAU_FASTINTVARIABLE!(LuauSolverConstraintLimit, 1000);
    // Analysis/src/ConstraintSolver.cpp
    crate::LUAU_FASTINTVARIABLE!(LuauSolverRecursionLimit, 500);
    // Analysis/src/NativeStackGuard.cpp
    crate::LUAU_FASTINTVARIABLE!(LuauStackGuardThreshold, 1024);
    // Analysis/src/Subtyping.cpp
    crate::LUAU_FASTINTVARIABLE!(LuauSubtypingIterationLimit, 20000);
    // Analysis/src/Subtyping.cpp
    crate::LUAU_FASTINTVARIABLE!(LuauSubtypingReasoningLimit, 100);
    // Analysis/src/Linter.cpp
    crate::LUAU_FASTINTVARIABLE!(LuauSuggestionDistance, 4);
    // Analysis/src/Type.cpp
    crate::LUAU_FASTINTVARIABLE!(LuauTableTypeMaximumStringifierLength, 0);
    // Analysis/src/Substitution.cpp
    crate::LUAU_FASTINTVARIABLE!(LuauTarjanChildLimit, 10000);
    // Analysis/src/Substitution.cpp
    crate::LUAU_FASTINTVARIABLE!(LuauTarjanPreallocationSize, 256);
    // Analysis/src/TypeInfer.cpp
    crate::LUAU_FASTINTVARIABLE!(LuauTypeInferIterationLimit, 20000);
    // Analysis/src/TypeInfer.cpp
    crate::LUAU_FASTINTVARIABLE!(LuauTypeInferRecursionLimit, 165);
    // Analysis/src/TypeInfer.cpp
    crate::LUAU_FASTINTVARIABLE!(LuauTypeInferTypePackLoopLimit, 5000);
    // Ast/src/Parser.cpp
    crate::LUAU_FASTINTVARIABLE!(LuauTypeLengthLimit, 1000);
    // Analysis/src/Type.cpp
    crate::LUAU_FASTINTVARIABLE!(LuauTypeMaximumStringifierLength, 500);
    // Analysis/src/TypeInfer.cpp
    crate::LUAU_FASTINTVARIABLE!(LuauVisitRecursionLimit, 500);
}

/// Dynamic bool flags (`DFFlag::`), mirroring `FFlag`.
#[allow(non_snake_case)]
pub mod DFFlag {
    // CodeGen/src/EmitCommonX64.cpp
    crate::LUAU_DYNAMIC_FASTFLAGVARIABLE!(AddReturnExectargetCheck, false);
    // Ast/src/Parser.cpp
    crate::LUAU_DYNAMIC_FASTFLAGVARIABLE!(DebugLuauReportReturnTypeVariadicWithTypeSuffix, false);
    // VM/src/lgc.cpp
    crate::LUAU_DYNAMIC_FASTFLAGVARIABLE!(LuauGcMarkUdataAccess, false);
    // VM/src/lgc.cpp
    crate::LUAU_DYNAMIC_FASTFLAGVARIABLE!(LuauGcTableStepFix, false);
    // Require/src/RequireNavigator.cpp
    crate::LUAU_DYNAMIC_FASTFLAGVARIABLE!(LuauRequireAliasOverrideOrderFix, false);
    // Require/src/RequireNavigator.cpp
    crate::LUAU_DYNAMIC_FASTFLAGVARIABLE!(LuauSelfIsSelfAndAlwaysSelf, false);
}

/// Dynamic int flags (`DFInt::`), mirroring `FInt`.
#[allow(non_snake_case)]
pub mod DFInt {
    // Analysis/src/TypeFunction.cpp
    crate::LUAU_DYNAMIC_FASTINTVARIABLE!(LuauTypeFamilyApplicationCartesianProductLimit, 5_000);
    // Analysis/src/TypeFunction.cpp
    crate::LUAU_DYNAMIC_FASTINTVARIABLE!(LuauTypeFamilyGraphReductionMaximumSteps, 1_000_000);
    // Analysis/src/TypeFunctionRuntimeBuilder.cpp
    crate::LUAU_DYNAMIC_FASTINTVARIABLE!(LuauTypeFunctionSerdeIterationLimit, 100_000);
    // Analysis/src/ConstraintGenerator.cpp
    crate::LUAU_DYNAMIC_FASTINTVARIABLE!(LuauConstraintGeneratorRecursionLimit, 300);
    // Analysis/src/Simplify.cpp
    crate::LUAU_DYNAMIC_FASTINTVARIABLE!(LuauSimplificationComplexityLimit, 8);
    // Analysis/src/BuiltinTypeFunctions.cpp
    crate::LUAU_DYNAMIC_FASTINTVARIABLE!(LuauStepRefineRecursionLimit, 64);
    // Analysis/src/Subtyping.cpp
    crate::LUAU_DYNAMIC_FASTINTVARIABLE!(LuauSubtypingRecursionLimit, 100);
    // Analysis/src/TypeFunction.cpp
    crate::LUAU_DYNAMIC_FASTINTVARIABLE!(LuauTypeFamilyUseGuesserDepth, -1);
    // Analysis/src/TypePath.cpp
    crate::LUAU_DYNAMIC_FASTINTVARIABLE!(LuauTypePathMaximumTraverseSteps, 100);
    // Analysis/src/Simplify.cpp
    crate::LUAU_DYNAMIC_FASTINTVARIABLE!(LuauTypeSimplificationIterationLimit, 128);
    // Analysis/src/Unifier2.cpp
    crate::LUAU_DYNAMIC_FASTINTVARIABLE!(LuauUnifierRecursionLimit, 100);
}

mod fastflag_timetrace_tests {
    /// The macro-defined flag reads its default; the TimeTrace consumer macros
    /// expand cleanly as no-ops (default `LUAU_ENABLE_TIME_TRACE` off).
    #[test]
    fn flag_default_and_timetrace_noops() {
        assert_eq!(crate::FFlag::DebugLuauTimeTracing.get(), false);
        crate::LUAU_TIMETRACE_SCOPE!("name", "category");
        crate::LUAU_TIMETRACE_OPTIONAL_TAIL_SCOPE!("name", "category", 100);
        crate::LUAU_TIMETRACE_ARGUMENT!("k", "v");
        crate::FFlag::DebugLuauTimeTracing.set(true);
        assert_eq!(crate::FFlag::DebugLuauTimeTracing.get(), true);
    }

    #[test]
    fn scoped_all_flags_do_not_change_parallel_runtime_threads() {
        crate::set_all_flags(true);
        let barrier = std::sync::Arc::new(std::sync::Barrier::new(2));
        let compiler_barrier = barrier.clone();
        let compiler = std::thread::spawn(move || {
            let _flags = crate::ScopedAllFlags::enter(false);
            assert!(!crate::FFlag::LuauCallFeedback.get());
            compiler_barrier.wait();
            compiler_barrier.wait();
        });

        barrier.wait();
        assert!(crate::FFlag::LuauCallFeedback.get());
        barrier.wait();
        compiler.join().expect("compiler flag scope exits cleanly");
        assert!(crate::FFlag::LuauCallFeedback.get());
    }
}
