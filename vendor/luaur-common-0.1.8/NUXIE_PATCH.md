# Nuxie Apple clock patch for luaur-common 0.1.8

This directory vendors the crates.io `luaur-common` 0.1.8 package. The only
Rust source change widens the upstream Mach monotonic-clock branches from
`target_os = "macos"` to `target_vendor = "apple"`.

The upstream fallback uses `libc::clock` and `libc::CLOCKS_PER_SEC`, which are
not exposed by Rust's `libc` crate for iOS. Apple platforms provide the same
`mach_absolute_time` and `mach_timebase_info` APIs already used by the package
on macOS, so this keeps the pinned Luau implementation and clock semantics
while making device and simulator builds compile.

Provenance:

- Package: crates.io `luaur-common` 0.1.8
- Original package checksum:
  `0d9c24d960012cf14bd4cfd056a89d41758d5548305e91552fc71aa0318edae7`
- Upstream repository: `https://github.com/pjankiewicz/luaur`
- Patch: Apple-vendor cfg widening in `get_clock_timestamp.rs` and
  `get_clock_period.rs`

## Luau fork rung 1

- Ported official Luau 0.725 delta (upstream 8f33df91..91caa731).
- Touched areas: release flag registration, raw-default-OFF exceptions, and
  flag-version metadata.

## Luau fork rung 2

- Ported official Luau 0.726 delta (upstream 91caa731..86d2a9dc).
- Touched areas: bytecode version target and dark release-flag registration
  for attribute CST and virtual BytecodeBuilder scaffolding.

## Luau fork rung 3

- Ported official Luau 0.727 delta (upstream 86d2a9dc..f1f121dc).
- Touched areas: removed-flag cleanup and dark registration of parser,
  CallInfo/proto-promotion, and GC-accounting gates.

## Luau fork rung 4

- Ported official Luau 0.728 delta (upstream f1f121dc..ddcea05e).
- Touched areas: removal of the const-underfill, grouped-expression CST, and
  error-tolerant pretty-printing flags after their ON paths became mandatory.

## Luau fork rung 5

- Ported official Luau 0.729 delta (upstream ddcea05e..6e9b580e).
- Touched areas: bytecode-v12 and cost-model flag registration, JIT threshold
  defaults, `DenseHash2`, pointer hashing, and direct-field flag versioning.

## Luau fork rung 6

- Ported official Luau 0.730 delta (upstream 6e9b580e..e8ae48c4).
- Touched areas: bytecode-v9 target selection, retired compiler gates, new
  dark math/GC flags, and occupied-bucket lifetime support in `DenseHash2`.
