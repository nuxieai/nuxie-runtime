# Nuxie patches for luaur-bytecode 0.1.8

## Luau fork rung 2

- Ported official Luau 0.726 delta (upstream 91caa731..86d2a9dc).
- Touched areas: bytecode version 7 table constants, GETIMPORT graph
  round-tripping, kind-aware graph equality, jump remapping, CallInliner
  correctness, and virtual BytecodeBuilder scaffolding.

## Luau fork rung 4

- Ported official Luau 0.728 delta (upstream f1f121dc..ddcea05e).
- Touched areas: reverse def-use tracking, sealed-SSA phi construction,
  CallInliner phi anchoring, graph validation, and cyclic-phi serialization.

## Luau fork rung 5

- Ported official Luau 0.729 delta (upstream ddcea05e..6e9b580e).
- Touched areas: bytecode-v12 proto sizing and cost metadata, graph-parser
  metadata consumption, and the new SCCP constant-evaluation entities.

## Luau fork rung 6

- Ported official Luau 0.730 delta (upstream 6e9b580e..e8ae48c4).
- Touched areas: unconditional bytecode-v9 emission, `DenseHash2`-backed SCCP
  state, and the SCCP evaluator, propagation driver, and graph rewrites.
