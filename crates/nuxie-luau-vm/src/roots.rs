//! Host roots (docs/luau-safe-vm-embedding-core.md, sections 2.1 and 5.2).
//!
//! A [`Root`] is a generational index into the core's root table: plain
//! data the host keeps for as long as it wants a value kept alive, and
//! releases explicitly. The collector marks every live slot (`gc_roots`).
//!
//! A root carries its VM's id, the slot index and the slot's generation.
//! Releasing a slot bumps its generation, so a released root (and every
//! copy of it) reads as stale even after the slot is reused; another VM's
//! root is refused by the id. Either is `Error::Host`, never a wrong value.
//! A slot whose generation would wrap is retired rather than reused, so a
//! generation is never handed out twice.
//!
//! Root slots are host bookkeeping, not charged to the script's budget,
//! as lane 6's handle table was not.

use std::sync::atomic::{AtomicU32, Ordering};

use crate::error::Error;
use crate::value::Value;

/// Distinguishes VMs, so a root used with the wrong VM is caught. Ids
/// repeat only after 2^32 VMs in one process.
static NEXT_VM_ID: AtomicU32 = AtomicU32::new(1);

/// A value the host keeps alive: an index into its VM's root table.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Root {
    vm: u32,
    index: u32,
    generation: u32,
}

struct Slot {
    value: Value,
    generation: u32,
    live: bool,
}

/// The core's root table.
pub(crate) struct Roots {
    vm: u32,
    slots: Vec<Slot>,
    free: Vec<u32>,
    live: usize,
}

impl Roots {
    pub(crate) fn new() -> Roots {
        Roots {
            vm: NEXT_VM_ID.fetch_add(1, Ordering::Relaxed),
            slots: Vec::new(),
            free: Vec::new(),
            live: 0,
        }
    }

    /// This VM's id.
    pub(crate) fn vm_id(&self) -> u32 {
        self.vm
    }

    /// Live roots.
    #[cfg(test)]
    pub(crate) fn live(&self) -> usize {
        self.live
    }

    /// A new root keeping `value` alive until it is released.
    pub(crate) fn insert(&mut self, value: Value) -> Result<Root, Error> {
        let index = match self.free.pop() {
            Some(index) => index,
            None => {
                let index = u32::try_from(self.slots.len())
                    .map_err(|_| Error::Host("too many live roots".to_owned()))?;
                self.slots.push(Slot {
                    value: Value::Nil,
                    generation: 0,
                    live: false,
                });
                index
            }
        };
        let Some(slot) = self.slots.get_mut(index as usize) else {
            return Err(Error::Host("internal error: a free root slot is missing".to_owned()));
        };
        slot.value = value;
        slot.live = true;
        self.live = self.live.saturating_add(1);
        Ok(Root {
            vm: self.vm,
            index,
            generation: slot.generation,
        })
    }

    fn slot(&self, root: Root) -> Result<&Slot, Error> {
        if root.vm != self.vm {
            return Err(Error::Host(
                "a root was used with a VM other than the one that made it".to_owned(),
            ));
        }
        match self.slots.get(root.index as usize) {
            Some(slot) if slot.live && slot.generation == root.generation => Ok(slot),
            _ => Err(Error::Host("a stale root".to_owned())),
        }
    }

    /// The value `root` keeps.
    pub(crate) fn get(&self, root: Root) -> Result<Value, Error> {
        self.slot(root).map(|slot| slot.value)
    }

    /// Releases `root`; false (and nothing changes) when it is stale or
    /// another VM's.
    pub(crate) fn release(&mut self, root: Root) -> bool {
        if self.slot(root).is_err() {
            return false;
        }
        let Some(slot) = self.slots.get_mut(root.index as usize) else {
            return false;
        };
        slot.value = Value::Nil;
        slot.live = false;
        self.live = self.live.saturating_sub(1);
        // A slot without a next generation is retired: reusing it would
        // repeat an old root's.
        if let Some(next) = slot.generation.checked_add(1) {
            slot.generation = next;
            self.free.push(root.index);
        }
        true
    }

    /// Appends the live values (GC roots).
    pub(crate) fn values(&self, out: &mut Vec<Value>) {
        out.extend(self.slots.iter().filter(|s| s.live).map(|s| s.value));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    type TestResult = Result<(), String>;

    fn check(ok: bool, what: &str) -> TestResult {
        if ok { Ok(()) } else { Err(what.to_owned()) }
    }

    #[test]
    fn a_released_root_is_stale_after_its_slot_is_reused() -> TestResult {
        let mut roots = Roots::new();
        let a = roots.insert(Value::Number(1.0)).map_err(|e| e.to_string())?;
        check(roots.release(a), "released")?;
        check(!roots.release(a), "a second release is a no-op")?;
        let b = roots.insert(Value::Number(2.0)).map_err(|e| e.to_string())?;
        check(a.index == b.index, "the slot is reused")?;
        check(
            matches!(roots.get(a), Err(Error::Host(m)) if m == "a stale root"),
            "the old root is stale",
        )?;
        check(
            matches!(roots.get(b), Ok(Value::Number(n)) if n == 2.0),
            "the new root reads its value",
        )?;
        check(roots.live() == 1, "one live root")
    }

    #[test]
    fn another_vms_root_is_refused() -> TestResult {
        let mut ours = Roots::new();
        let mut theirs = Roots::new();
        let _mine = ours.insert(Value::Boolean(true)).map_err(|e| e.to_string())?;
        let foreign = theirs.insert(Value::Boolean(false)).map_err(|e| e.to_string())?;
        check(
            matches!(ours.get(foreign), Err(Error::Host(m)) if m.contains("other than the one")),
            "a foreign root is an error",
        )?;
        check(!ours.release(foreign), "and cannot release our slot")?;
        check(ours.live() == 1, "our root is untouched")
    }

    #[test]
    fn a_slot_is_retired_before_its_generation_wraps() -> TestResult {
        let mut roots = Roots::new();
        let a = roots.insert(Value::Nil).map_err(|e| e.to_string())?;
        if let Some(slot) = roots.slots.get_mut(a.index as usize) {
            slot.generation = u32::MAX;
        }
        let last = Root {
            generation: u32::MAX,
            ..a
        };
        check(roots.release(last), "released")?;
        let b = roots.insert(Value::Nil).map_err(|e| e.to_string())?;
        check(b.index != a.index, "the retired slot is not reused")
    }
}
