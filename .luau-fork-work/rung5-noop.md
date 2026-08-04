# Luau fork rung 5 no-op rows

- Translation-unit include set (`Common/include/Luau/HashUtil.h`): removing the
  unnecessary C++ `Luau/Common.h` include has no Rust twin or behavioral delta.
- `JitInliner::disable` (`Inliner/src/JitInliner.cpp`): the hunk only adds the
  file's final newline; there is no code change to translate.

Rust ownership and iterator traits provide the language-level equivalents of
the new `DenseHash2` copy/move/destructor and iterator-operator declarations;
those rows are implemented by the new container types and are not treated as
source no-ops.
