# Nuxie patches for luaur-bytecode 0.1.8

## Luau fork rung 2

- Ported official Luau 0.726 delta (upstream 91caa731..86d2a9dc).
- Touched areas: bytecode version 7 table constants, GETIMPORT graph
  round-tripping, kind-aware graph equality, jump remapping, CallInliner
  correctness, and virtual BytecodeBuilder scaffolding.
