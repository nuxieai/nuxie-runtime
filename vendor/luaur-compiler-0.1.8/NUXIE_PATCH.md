# Nuxie patches for luaur-compiler 0.1.8

## Luau fork rung 1

- Ported official Luau 0.725 delta (upstream 8f33df91..91caa731).
- Touched areas: exported-class tracking and export-table/class expression
  compilation.

## Luau fork rung 2

- Ported official Luau 0.726 delta (upstream 91caa731..86d2a9dc).
- Touched areas: unconditional duptable constant packing and multi-return
  tracking for function-inlining eligibility.

## Luau fork rung 3

- Ported official Luau 0.727 delta (upstream 86d2a9dc..f1f121dc).
- Touched areas: promoted table-property constant folding, change-log undo,
  safe `next` specialization, and the FASTCALL3 cost model.

## Luau fork rung 5

- Ported official Luau 0.729 delta (upstream ddcea05e..6e9b580e).
- Touched areas: recursive type-alias bytecode typing and bytecode-v12 packed
  cost computation/emission for inlinable functions.

## Luau fork rung 6

- Ported official Luau 0.730 delta (upstream 6e9b580e..e8ae48c4).
- Touched areas: unconditional table-function inlining and escape-based table
  mutation tracking, including removal of the deprecated tracker.

## Luau fork rung 7

- Ported official Luau 0.731 delta (upstream e8ae48c4..f8ca77ac).
- Touched areas: precision-selectable vector constants and folding, hoisted
  class locals across parsing/value tracking/codegen, and dark IIFE inlining.

## Luau fork rung 8

- Ported official Luau 0.732 delta (upstream f8ca77ac..decb2d05).
- Touched areas: superclass-aware `NEWCLASS` emission, shaped export tables,
  export-aware value tracking, delayed function exports, and proto marking.

## Luau fork rung 9

- Ported rive_0_732 fork patch set (decb2d05..86eb0096).
- Touched areas: Rive `math.fround`/`Vector` builtin recognition, exact
  arity/safety metadata, and numeric/vector result type mapping.
