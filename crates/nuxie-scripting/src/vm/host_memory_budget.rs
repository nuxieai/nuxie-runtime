//! One budget per script VM for the host memory its bindings hold.
//!
//! The VM's memory ceiling covers the VM heap only. A binding that grows a
//! host-side container on a script's behalf (path commands, vertex data,
//! text runs, decoded images, copied strings) charges the bytes it is about
//! to reserve against this budget first, and the payload that owns the
//! container refunds them when it drops. The budget counts capacity a binding
//! reserves (or a stated upper bound of it, where the container is not ours),
//! never length, so a container that keeps its capacity after a clear keeps
//! its charge until it is dropped.
//!
//! The budget lives in the VM's app data, so every binding reaches it from
//! the `Lua` it is called with; each charge keeps its own handle, so a
//! payload refunds without a `Lua`.

use std::cell::Cell;
use std::rc::Rc;

use crate::luau::{Error, Lua, Result};

/// The budget a VM gets when its host does not choose one: the same number
/// as the default ceiling of a File's VM (`DEFAULT_SCRIPT_VM_MEMORY_BYTES`).
pub const DEFAULT_SCRIPT_HOST_MEMORY_BYTES: usize = super::DEFAULT_SCRIPT_VM_MEMORY_BYTES;

struct BudgetState {
    limit: Cell<usize>,
    used: Cell<usize>,
}


/// The host memory one script VM's bindings may hold, in bytes.
#[derive(Clone)]
pub(crate) struct HostMemoryBudget(Rc<BudgetState>);

impl HostMemoryBudget {
    fn new(limit: usize) -> Self {
        Self(Rc::new(BudgetState {
            limit: Cell::new(limit),
            used: Cell::new(0),
        }))
    }

    /// Installs the VM's budget with `limit`, or sets the limit of the one
    /// already installed (charges already made are kept).
    pub(crate) fn install(lua: &Lua, limit: usize) -> Result<()> {
        if let Some(budget) = lua.try_app_data_ref::<Self>()? {
            budget.0.limit.set(limit);
            return Ok(());
        }
        lua.try_set_app_data(Self::new(limit))?;
        Ok(())
    }

    /// The VM's budget. A `Lua` that no `ScriptVm` set up (a test's bare
    /// state) gets the default budget on first use.
    pub(crate) fn for_lua(lua: &Lua) -> Result<Self> {
        if let Some(budget) = lua.try_app_data_ref::<Self>()? {
            return Ok(budget.clone());
        }
        let budget = Self::new(DEFAULT_SCRIPT_HOST_MEMORY_BYTES);
        lua.try_set_app_data(budget.clone())?;
        Ok(budget)
    }

    pub(crate) fn limit(&self) -> usize {
        self.0.limit.get()
    }

    pub(crate) fn used(&self) -> usize {
        self.0.used.get()
    }

    fn charge(&self, bytes: usize) -> Result<()> {
        let used = self.0.used.get();
        let remaining = self.limit().saturating_sub(used);
        if bytes > remaining {
            return Err(budget_exceeded(self.limit()));
        }
        // `bytes <= limit - used`, so the sum is at most `limit`.
        self.0.used.set(used.saturating_add(bytes));
        Ok(())
    }

    fn refund(&self, bytes: usize) {
        self.0.used.set(self.0.used.get().saturating_sub(bytes));
    }
}

/// The one error every binding raises when its charge does not fit.
pub(crate) fn budget_exceeded(limit: usize) -> Error {
    Error::runtime(format!(
        "script VM exceeded its {limit}-byte host memory budget"
    ))
}

/// Bytes one payload holds against its VM's budget, refunded on drop.
///
/// A charge made without a `Lua` at hand (a payload built by host code) is
/// detached: it counts nothing until [`Self::attach`] gives it the VM's
/// budget, which every growing binding does before it charges.
#[derive(Default)]
pub(crate) struct HostMemoryCharge {
    budget: Option<HostMemoryBudget>,
    bytes: usize,
}

impl HostMemoryCharge {
    /// A charge of `bytes` against the VM's budget.
    pub(crate) fn for_lua(lua: &Lua, bytes: usize) -> Result<Self> {
        let mut charge = Self::default();
        charge.attach(lua)?;
        charge.reserve_to(bytes)?;
        Ok(charge)
    }

    /// Gives a detached charge the VM's budget, charging what it records.
    pub(crate) fn attach(&mut self, lua: &Lua) -> Result<()> {
        if self.budget.is_some() {
            return Ok(());
        }
        let budget = HostMemoryBudget::for_lua(lua)?;
        budget.charge(self.bytes)?;
        self.budget = Some(budget);
        Ok(())
    }

    pub(crate) fn bytes(&self) -> usize {
        self.bytes
    }

    /// Whether raising the charge to `bytes` would fit the budget.
    pub(crate) fn fits(&self, bytes: usize) -> bool {
        let growth = bytes.saturating_sub(self.bytes);
        self.budget
            .as_ref()
            .is_none_or(|budget| growth <= budget.limit().saturating_sub(budget.used()))
    }

    /// Raises the charge to `bytes` if it is lower; never lowers it, since a
    /// container keeps the capacity it grew to.
    pub(crate) fn reserve_to(&mut self, bytes: usize) -> Result<()> {
        let Some(growth) = bytes.checked_sub(self.bytes) else {
            return Ok(());
        };
        if let Some(budget) = &self.budget {
            budget.charge(growth)?;
        }
        self.bytes = bytes;
        Ok(())
    }

    /// Raises the charge by `bytes`.
    pub(crate) fn grow(&mut self, bytes: usize) -> Result<()> {
        let total = self.bytes.checked_add(bytes).ok_or_else(|| {
            budget_exceeded(self.budget.as_ref().map_or(usize::MAX, HostMemoryBudget::limit))
        })?;
        self.reserve_to(total)
    }

    /// Sets the charge to exactly `bytes`, refunding a decrease, for a
    /// container that released its capacity.
    pub(crate) fn set(&mut self, bytes: usize) -> Result<()> {
        match self.bytes.checked_sub(bytes) {
            Some(decrease) => {
                if let Some(budget) = &self.budget {
                    budget.refund(decrease);
                }
                self.bytes = bytes;
                Ok(())
            }
            None => self.reserve_to(bytes),
        }
    }
}

impl std::fmt::Debug for HostMemoryCharge {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("HostMemoryCharge")
            .field("bytes", &self.bytes)
            .field("attached", &self.budget.is_some())
            .finish()
    }
}

impl Drop for HostMemoryCharge {
    fn drop(&mut self) {
        if let Some(budget) = &self.budget {
            budget.refund(self.bytes);
        }
    }
}

/// Bytes `count` elements of `T` occupy, saturating: a saturated size is
/// refused by any budget.
pub(crate) const fn bytes_of<T>(count: usize) -> usize {
    count.saturating_mul(std::mem::size_of::<T>())
}

/// The capacity `Vec` growth reaches when `len + additional` elements must
/// fit a vector of capacity `capacity`: at least double, as the standard
/// library grows, so that charging it keeps pushes amortized.
pub(crate) fn grown_capacity(capacity: usize, len: usize, additional: usize) -> usize {
    let required = len.saturating_add(additional);
    if required <= capacity {
        capacity
    } else {
        required.max(capacity.saturating_mul(2)).max(4)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn charges_refund_on_drop_and_refuse_past_the_limit() {
        let lua = Lua::new();
        HostMemoryBudget::install(&lua, 100).unwrap();
        let budget = HostMemoryBudget::for_lua(&lua).unwrap();
        let mut first = HostMemoryCharge::for_lua(&lua, 60).unwrap();
        assert_eq!(budget.used(), 60);
        let error = HostMemoryCharge::for_lua(&lua, 41).err().unwrap();
        assert!(
            error
                .to_string()
                .ends_with("script VM exceeded its 100-byte host memory budget"),
            "{error}"
        );
        // A refused charge leaves the budget as it was.
        assert_eq!(budget.used(), 60);
        first.reserve_to(40).unwrap();
        assert_eq!(first.bytes(), 60, "reserve_to never lowers a charge");
        first.set(10).unwrap();
        assert_eq!(budget.used(), 10);
        let second = HostMemoryCharge::for_lua(&lua, 90).unwrap();
        assert_eq!(budget.used(), 100);
        drop(second);
        drop(first);
        assert_eq!(budget.used(), 0);
    }

    #[test]
    fn a_detached_charge_counts_once_attached() {
        let lua = Lua::new();
        HostMemoryBudget::install(&lua, 100).unwrap();
        let budget = HostMemoryBudget::for_lua(&lua).unwrap();
        let mut charge = HostMemoryCharge::default();
        charge.grow(70).unwrap();
        assert_eq!(budget.used(), 0);
        charge.attach(&lua).unwrap();
        assert_eq!(budget.used(), 70);
        assert!(charge.grow(31).is_err());
        assert_eq!(charge.bytes(), 70);
    }

    #[test]
    fn growth_at_least_doubles() {
        assert_eq!(grown_capacity(0, 0, 1), 4);
        assert_eq!(grown_capacity(4, 4, 1), 8);
        assert_eq!(grown_capacity(8, 3, 2), 8);
        assert_eq!(grown_capacity(8, 8, 100), 108);
        assert_eq!(grown_capacity(usize::MAX, usize::MAX, 1), usize::MAX);
    }
}
