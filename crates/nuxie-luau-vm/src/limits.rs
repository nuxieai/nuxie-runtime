//! Plan decision D3: every limit the VM enforces is a field of one struct.
//! Defaults follow the vendored VM's compile-time constants.

/// The limits a VM enforces. Exceeding one is a Lua error (or, for
/// bytecode, a load error), never a panic or an abort.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Limits {
    /// Lua call depth (`LUAI_MAXCALLS`). Exceeding it raises "stack overflow".
    pub max_calls: usize,
    /// C++ `LUAI_MAXCCALLS`: the count of nested counted calls (metamethods,
    /// `pcall` from a non-yieldable thread, library callbacks, coroutine
    /// resumes). Reaching it raises "C stack overflow". None of these
    /// recurse on the Rust stack (lane 4, `call.rs`), so this limit is
    /// semantic, as in C++, and does not bound Rust stack use.
    pub max_c_calls: u32,
    /// Nested Rust-stack re-entries into the interpreter: host callbacks
    /// calling back (`HostCtx`), the `xpcall` handler, `Vm::run`, and the
    /// host API's operations that need a metamethod's result in place
    /// (`Table::get` and friends). Exceeding it raises "C stack overflow".
    /// This is the only bound on the Rust stack the VM uses. A level costs
    /// about 48 KiB in an unoptimized build (measured 2026-11-11, aarch64
    /// macOS: 46,480 bytes through a host callback, 49,056 through
    /// `string.format`), so the default of 16 fits a 1 MiB thread stack;
    /// `tools/luau-vm-diff`'s `reentry_reaches_its_limits_on_a_one_mib_stack`
    /// runs every path to its limit on one. C++ has no separate limit (it
    /// counts these in `max_c_calls`), so a script nesting them deeper than
    /// this fails here first.
    pub max_reentry: u32,
    /// Value slots one thread's stack may grow to; capped below `u32::MAX`
    /// when the VM is built (slots are stored as `u32`).
    pub max_stack_slots: usize,
    /// Length of an `__index` / `__newindex` chain (`MAXTAGLOOP`).
    pub max_meta_chain: u32,
    /// Size of a bytecode chunk `Core::load` accepts.
    pub max_bytecode_bytes: usize,
    /// Length of a string (`MAXSSIZE`).
    pub max_string_bytes: usize,
    /// Approximate bytes of live heap objects; an allocation past it raises
    /// a memory error. `None` means unbounded. The accounting is lane 1's
    /// stand-in; lane 3 (collector) makes it exact.
    pub max_memory_bytes: Option<usize>,
}

impl Default for Limits {
    fn default() -> Limits {
        Limits {
            max_calls: 20_000,
            max_c_calls: 200,
            max_reentry: 16,
            max_stack_slots: 1_000_000,
            max_meta_chain: 100,
            max_bytecode_bytes: 64 << 20,
            max_string_bytes: 1 << 30,
            max_memory_bytes: None,
        }
    }
}
