//! The memory budget (plan decision D3): the one accounting point for every
//! allocation a script can cause.
//!
//! [`Budget`] holds the bytes in use and `Limits::max_memory_bytes`. Every
//! charge is checked *before* the memory is allocated, and a refused charge
//! changes nothing, so a script that runs out of budget gets the memory
//! error (`LUA_ERRMEM`) and the heap is as it was.
//!
//! The count is exact with respect to the cost model in `heap.rs`
//! (`object_cost`): `used` always equals the cost of every live arena
//! object plus the VM's own buffers (`external`). `Vm::accounting_drift`
//! recomputes that sum from the arenas, and the differential harness
//! (`tools/luau-vm-diff`) asserts the drift is zero after every case, so an
//! allocation that bypasses the budget shows up as drift in the next run.
//!
//! # How a builtin charges
//!
//! A new builtin (or opcode) allocates in one of four ways. Each has one
//! API, and nothing else may allocate on behalf of a script:
//!
//! 1. **A new heap object** (table, string, closure, upvalue, prototype):
//!    call `Heap::new_table`, `Heap::intern`, `Heap::new_function`,
//!    `Heap::new_upval` or `Heap::new_proto` (or the `Vm` wrappers such as
//!    `Vm::new_table`). They charge the object's whole cost before it
//!    exists. A new object *kind* gets its own arena in `heap.rs`, a `new_*`
//!    that charges, and an arm in `object_cost`; the sweep credits it.
//! 2. **Growing an existing object**: tables grow only inside `table.rs`,
//!    whose storage is private and whose growing methods take
//!    `&mut Budget` (they `check` the predicted growth before changing
//!    anything and `settle` the actual size after); outside it, use
//!    `Vm::raw_set`, `Heap::table_set`, `Heap::table_resize_array` or
//!    `Heap::clone_table`. `Table::array_mut` hands out a slice, which
//!    cannot grow.
//! 3. **A temporary buffer** that the builtin builds and frees before it
//!    returns (a `Vec<u8>` for `string.rep`, `gsub`, `format`, `concat`):
//!    call `Heap::check_alloc(final_len)` before building it, or before
//!    each step that can grow it. Temporaries are checked, not charged:
//!    they are gone when the builtin returns. Interning the result then
//!    charges the string (rule 1).
//! 4. **A VM buffer that outlives the call** (today only the value stack):
//!    `Heap::charge_external`, which `accounting_drift` counts as
//!    `external`.
//!
//! Every builtin that allocates in proportion to its arguments gets a case
//! in the memory-budget suite (`tools/luau-vm-diff`, `limit_tests`): run it
//! under a budget too small for the result and expect the memory error.

/// The memory error, before it is turned into the VM's `Throw` (which
/// `Heap::memory_error` builds without allocating).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct OutOfMemory;

/// Bytes in use against the limit.
#[derive(Debug)]
pub(crate) struct Budget {
    /// Bytes charged and not yet credited.
    used: usize,
    /// Of `used`, the bytes held by VM buffers outside the arenas.
    external: usize,
    /// `Limits::max_memory_bytes`.
    limit: Option<usize>,
    /// While set, every charge and check of a non-zero size fails. The
    /// conformance harness's `setblockallocations` (gc.luau) uses it to
    /// show that collection itself never allocates.
    blocked: bool,
    /// Set by every refusal; cleared by the host (`Vm::clear_memory_exhausted`).
    /// A `Cell` because `check` takes `&self`.
    exhausted: std::cell::Cell<bool>,
}

impl Budget {
    pub(crate) fn new(limit: Option<usize>) -> Budget {
        Budget {
            used: 0,
            external: 0,
            limit,
            blocked: false,
            exhausted: std::cell::Cell::new(false),
        }
    }

    /// Fails if `bytes` more would pass the limit; charges nothing.
    #[inline]
    pub(crate) fn check(&self, bytes: usize) -> Result<(), OutOfMemory> {
        let refused = (self.blocked && bytes > 0)
            || matches!(self.limit, Some(limit) if self.used.saturating_add(bytes) > limit);
        if refused {
            self.exhausted.set(true);
            return Err(OutOfMemory);
        }
        Ok(())
    }

    /// Checks, then adds `bytes` to the count. A refused charge adds
    /// nothing.
    #[inline]
    pub(crate) fn charge(&mut self, bytes: usize) -> Result<(), OutOfMemory> {
        self.check(bytes)?;
        self.used = self.used.saturating_add(bytes);
        Ok(())
    }

    /// Returns `bytes` to the budget (an object freed or shrunk).
    #[inline]
    pub(crate) fn credit(&mut self, bytes: usize) {
        self.used = self.used.saturating_sub(bytes);
    }

    /// Records that an object's size changed from `before` to `after`
    /// bytes: a credit when it shrank, a charge when it grew. Growth must
    /// have passed [`Budget::check`] before the object changed; `settle`
    /// only keeps the count exact (it never refuses). `Table` is the one
    /// caller.
    #[inline]
    pub(crate) fn settle(&mut self, before: usize, after: usize) {
        if after >= before {
            self.used = self.used.saturating_add(after.saturating_sub(before));
        } else {
            self.credit(before.saturating_sub(after));
        }
    }

    /// Charges a VM buffer outside the arenas.
    pub(crate) fn charge_external(&mut self, bytes: usize) -> Result<(), OutOfMemory> {
        self.charge(bytes)?;
        self.external = self.external.saturating_add(bytes);
        Ok(())
    }

    /// Returns an external buffer's bytes (a dead thread's stack).
    pub(crate) fn credit_external(&mut self, bytes: usize) {
        self.credit(bytes);
        self.external = self.external.saturating_sub(bytes);
    }

    #[inline]
    pub(crate) fn used(&self) -> usize {
        self.used
    }

    pub(crate) fn external(&self) -> usize {
        self.external
    }

    pub(crate) fn limit(&self) -> Option<usize> {
        self.limit
    }

    /// Changes the limit. Objects already allocated stay; the next charge
    /// past the new limit fails.
    pub(crate) fn set_limit(&mut self, limit: Option<usize>) -> Option<usize> {
        std::mem::replace(&mut self.limit, limit)
    }

    /// Whether a charge or check was refused since the last clear.
    pub(crate) fn exhausted(&self) -> bool {
        self.exhausted.get()
    }

    pub(crate) fn clear_exhausted(&self) {
        self.exhausted.set(false);
    }

    pub(crate) fn set_blocked(&mut self, blocked: bool) {
        self.blocked = blocked;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn refused_charge_changes_nothing() {
        let mut budget = Budget::new(Some(100));
        assert_eq!(budget.charge(60), Ok(()));
        assert_eq!(budget.charge(41), Err(OutOfMemory));
        assert_eq!(budget.used(), 60);
        assert_eq!(budget.charge(40), Ok(()));
        budget.credit(30);
        assert_eq!(budget.used(), 70);
    }

    #[test]
    fn blocked_refuses_every_nonzero_size() {
        let mut budget = Budget::new(None);
        budget.set_blocked(true);
        assert_eq!(budget.charge(1), Err(OutOfMemory));
        assert_eq!(budget.check(0), Ok(()));
        budget.set_blocked(false);
        assert_eq!(budget.charge(1), Ok(()));
    }
}
