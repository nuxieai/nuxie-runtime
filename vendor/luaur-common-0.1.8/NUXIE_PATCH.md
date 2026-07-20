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
