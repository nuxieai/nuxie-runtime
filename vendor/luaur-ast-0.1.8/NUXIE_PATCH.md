# Nuxie patches for luaur-ast 0.1.8

## Luau fork rung 1

- Ported official Luau 0.725 delta (upstream 8f33df91..91caa731).
- Touched areas: local-declaration keyword locations, CST field ownership,
  parser recovery and table-type locations, and pretty-printing.

## Luau fork rung 2

- Ported official Luau 0.726 delta (upstream 91caa731..86d2a9dc).
- Touched areas: attribute CST records, flag-gated parser propagation, and
  attribute pretty-printing.

## Luau fork rung 3

- Ported official Luau 0.727 delta (upstream 86d2a9dc..f1f121dc).
- Touched areas: const syntax promotion, grouped type CSTs, local-function
  const keyword positions, and extern type-definition compatibility.

## Luau fork rung 4

- Ported official Luau 0.728 delta (upstream f1f121dc..ddcea05e).
- Touched areas: removed parser/CST flags, const-underfill AST recovery, and
  unconditional grouped-expression CST storage.
