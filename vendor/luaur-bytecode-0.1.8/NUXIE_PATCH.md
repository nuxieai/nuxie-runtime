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

## Dormant divergence to re-audit (rung 6, 2026-08-04)

`Sccp::arithToK` (src/methods/sccp_arith_to_k.rs): for a const-lhs
`LOP_MOD`/`LOP_POW`, C at e8ae48c4 falls through with a default-constructed
constant and crashes on `bad_optional_access`; the Rust twin folds the
opcode check into the selection guard and skips the fold instead. Zero call
sites for bytecode `foldConstants` exist in either tree at this pin, so the
divergence is unreachable. Re-audit this site at the first rung that
introduces a caller ("crash vs silently-unfolded" becomes observable then).

## Luau fork rung 7

- Ported official Luau 0.731 delta (upstream e8ae48c4..f8ca77ac).
- Touched areas: float/double vector constant tags and graph round-tripping,
  centralized def-use-safe `BcFunction` rewrites, and SCCP IEEE division.

## Luau fork rung 8

- Ported official Luau 0.732 delta (upstream f8ca77ac..decb2d05).
- Touched areas: bytecode-v13 double-vector emission, v100 class shapes and
  `NEWCLASS` graph support, dead-PC sentinels, and sealed feedback migration.

## Luau fork rung 9

- Ported rive_0_732 fork patch set (decb2d05..86eb0096).
- Touched areas: default version-selection coverage for the pinned bytecode-v7
  compatibility target.
