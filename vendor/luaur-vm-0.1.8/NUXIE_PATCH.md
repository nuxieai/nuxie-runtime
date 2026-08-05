# Nuxie Apple clock patch for luaur-vm 0.1.8

This directory vendors the crates.io `luaur-vm` 0.1.8 package. The only Rust
source change widens the upstream Mach monotonic-clock branches from
`target_os = "macos"` to `target_vendor = "apple"`.

Without this patch the iOS fallback declares a `CLOCKS_PER_SEC` external
symbol even though Darwin supplies it as a C macro. Reusing the VM's existing
Mach clock implementation avoids that invalid link contract on every Apple
device and simulator target.

Provenance:

- Package: crates.io `luaur-vm` 0.1.8
- Original package checksum:
  `945d6993538f99bc25a424b7a7a55b9db953d609f7fc869e6d80495326e46ae2`
- Upstream repository: `https://github.com/pjankiewicz/luaur`
- Patch: Apple-vendor cfg widening in `clock_timestamp.rs` and
  `clock_period.rs`

## Luau fork rung 1

- Ported official Luau 0.725 delta (upstream 8f33df91..91caa731).
- Touched areas: API stack growth, table cloning, custom yieldable protected
  calls, userdata-metatable GC pinning, and direct-field initialization.

## Luau fork rung 3

- Ported official Luau 0.727 delta (upstream 86d2a9dc..f1f121dc).
- Touched areas: closure usage removal, CallInfo active protos, optimized proto
  links and promotion, interpreter/debug proto selection, and GC accounting.

## Luau fork rung 5

- Ported official Luau 0.729 delta (upstream ddcea05e..6e9b580e).
- Touched areas: bytecode-v12 loading and proto cost storage, coroutine C-call
  restoration, userdata tag APIs, and direct-field GC atomic remarking.

## Luau fork rung 6

- Ported official Luau 0.730 delta (upstream 6e9b580e..e8ae48c4).
- Touched areas: unsigned user-defined-class member offsets, dark negative-zero
  rounding, and flag-gated userdata direct-access GC marking/validation.

## Luau fork rung 7

- Ported official Luau 0.731 delta (upstream e8ae48c4..f8ca77ac).
- Touched areas: optional GC-backed double vectors, fixed-object allocation,
  vector bytecode/runtime callers, embedder GC and weak-reference APIs,
  backedge GC checks, public debug/memory APIs, and xpcall continuation depth.
