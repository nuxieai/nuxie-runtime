# Nuxie platform clock patches for luaur-common 0.1.8

This directory vendors the crates.io `luaur-common` 0.1.8 package. Its clock
sources carry Apple and Android compatibility patches.

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
- Patches: Apple-vendor cfg widening plus Android Bionic `clock()` binding and
  clock-period fallback in `get_clock_timestamp.rs` and `get_clock_period.rs`

## Android Bionic clock compatibility

- `get_clock_timestamp.rs` binds Bionic's exported `clock()` function
  directly because the Rust `libc` crate does not expose that Android
  binding.
- `get_clock_period.rs` uses Bionic's defined `CLOCKS_PER_SEC` value of
  1,000,000 because the Rust `libc` crate does not expose the macro.

Together these preserve the upstream fallback profiler-clock semantics while
allowing Android scripting builds to compile.

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

## Luau fork rung 7

- Ported official Luau 0.731 delta (upstream e8ae48c4..f8ca77ac).
- Touched areas: bytecode vector-double tagging, dark rung flag registration,
  and the clipped `DenseHash2` hash-shift assertion.

## Luau fork rung 8

- Ported official Luau 0.732 delta (upstream f8ca77ac..decb2d05).
- Touched areas: `NEWCLASS`/v13/v100/proto-flag definitions, rung flag
  retirements and dark registrations, and `VecDeque` emplacement helpers.

## Luau fork rung 9

- Ported rive_0_732 fork patch set (decb2d05..86eb0096).
- Touched areas: the pinned bytecode-v7 target override and the exact 243,
  245-255 Rive builtin ABI block.

## Fast-call dispatch enablement

- Preserved the pinned engine's raw-default-OFF profile for
  `LuauIntegerFastcalls` and `LuauIntegerBufferFastcalls` when test helpers
  enable all other flags.
