# Luau fork rung 5 deferred rows

## Inliner subsystem (intentionally deferred)

The Rust fork does not translate the C++ `Inliner/` subsystem introduced in
official Luau 0.727. Per the rung-5 lane directive, the following 0.729 rows
remain deferred until that subsystem exists in Rust:

- `JitInliner::createInlinedProto` (`Inliner/src/JitInliner.cpp`): copies the
  caller proto's cost into a synthesized inlined proto.
- `kMaxFunctionBytecodeSize` removal (`Inliner/src/JitInliner.cpp`): replaces
  the fixed limit with `LuauJitInlineTooLongFunSize`.
- `JitInliner::computeCost` (`Inliner/src/JitInliner.cpp`): decodes the packed
  bytecode cost and applies constant-argument discounts.
- `JitInliner::isConstOp` (`Inliner/src/JitInliner.cpp`): recognizes constant
  loads and follows `MOVE` chains.
- `JitInliner::onInlineFunction` (`Inliner/src/JitInliner.cpp`): applies the
  configurable size and profitability checks.

The configuration surface is not deferred: the four new JIT FInts are
registered at 25, 300, 128, and `0xFFFF`, and the existing
`LuauInlineHitsThreshold` default is updated from 3 to 32. `Proto::cost` also
landed in its correct post-`optimized`/`deoptimized` position because those
prerequisite fields arrived in rung 3.
