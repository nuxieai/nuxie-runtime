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
