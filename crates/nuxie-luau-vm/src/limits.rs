//! Plan decision D3: every limit the VM enforces is a field of one struct.
//! Defaults follow the vendored VM's compile-time constants.

/// The limits a VM enforces. Exceeding one is a Lua error (or, for
/// bytecode, a load error), never a panic or an abort.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Limits {
    /// Lua call depth (`LUAI_MAXCALLS`). Exceeding it raises "stack overflow".
    pub max_calls: usize,
    /// Nested re-entries from Rust into the interpreter: metamethods,
    /// `pcall`, host callbacks (`LUAI_MAXCCALLS`). Exceeding it raises
    /// "C stack overflow". This bounds the Rust stack the VM uses.
    pub max_c_calls: u32,
    /// Value slots one thread's stack may grow to.
    pub max_stack_slots: usize,
    /// Length of an `__index` / `__newindex` chain (`MAXTAGLOOP`).
    pub max_meta_chain: u32,
    /// Size of a bytecode chunk `Vm::load` accepts.
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
            max_stack_slots: 1_000_000,
            max_meta_chain: 100,
            max_bytecode_bytes: 64 << 20,
            max_string_bytes: 1 << 30,
            max_memory_bytes: None,
        }
    }
}
