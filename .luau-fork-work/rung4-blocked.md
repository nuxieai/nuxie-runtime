# Luau fork rung 4 blocked/deferred dispositions

Diff: `f1f121dc..ddcea05e` (official Luau 0.727 to 0.728)

No ledger rows are blocked or deferred in this rung. The scoped C++ diff has no
changed paths under `Require/` or the top-level `Inliner/` subsystem.

`Bytecode/include/Luau/BytecodeCallInliner.h` is part of the translated
`luaur-bytecode` surface, so its def-use and phi-anchoring changes were ported
rather than deferred.
