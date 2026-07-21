//! Retained-identity view-model property cells — #RB-1 slice (a)+(b).
//!
//! Ports the C++ core that the flattened data-bind design discarded:
//!
//! - `ViewModelInstanceValue` (`viewmodel_instance_value.cpp`): one shared,
//!   typed, mutable property value with a changed flag and a dependent list.
//! - `DependencyHelper` (`dependency_helper.hpp`): mutation cascades dirt to
//!   every registered dependent; nothing else happens inline.
//! - `SuppressDelegation` (`viewmodel_instance_value.hpp`): scoped delegate
//!   suppression so internal writes (e.g. trigger `advanced()` zeroing) do
//!   not notify hosts.
//!
//! `Rc<RefCell<..>>` is the Rust analog of C++ `rcp`: cloning a
//! [`RuntimeViewModelCell`] shares identity, exactly like retaining the same
//! `ViewModelInstanceValue*`. The dirt cascade deliberately does nothing but
//! set bits on dependent sinks — mirroring C++ `addDirt`, and keeping the
//! cascade re-entrancy-free while listener actions mutate other cells.

use std::cell::{Cell, RefCell};
use std::rc::{Rc, Weak};

/// Dirt bits a cell mutation cascades to its dependents.
///
/// C++ cascades `ComponentDirt::Bindings`; the rebuild keeps one bit per
/// concern so the update cycle can cheaply ask "which of my binds observed a
/// source change since the last pass".
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RuntimeCellDirt(u8);

impl RuntimeCellDirt {
    pub const NONE: Self = Self(0);
    /// The bound source value changed (C++ `ComponentDirt::Bindings`).
    pub const BINDINGS: Self = Self(1 << 0);

    pub fn contains(self, other: Self) -> bool {
        self.0 & other.0 == other.0
    }

    pub fn insert(&mut self, other: Self) {
        self.0 |= other.0;
    }

    pub fn take(&mut self) -> Self {
        std::mem::replace(self, Self::NONE)
    }

    pub fn is_empty(self) -> bool {
        self.0 == 0
    }
}

/// One dependent's dirt sink.
///
/// C++ stores a raw `DataBind*` and calls `addDirt` on it. The Rust analog
/// is a shared bit-cell owned by the dependent (a retained bind, listener,
/// or converter operand); the cell holds only a weak reference, so a dropped
/// dependent unregisters itself implicitly, mirroring `~DataBind`'s
/// `removeDependent` without manual lifetime bookkeeping.
#[derive(Clone, Default)]
pub struct RuntimeCellDirtSink {
    bits: Rc<Cell<u8>>,
}

impl RuntimeCellDirtSink {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_dirt(&self, dirt: RuntimeCellDirt) {
        self.bits.set(self.bits.get() | dirt.0);
    }

    pub fn take_dirt(&self) -> RuntimeCellDirt {
        RuntimeCellDirt(self.bits.replace(0))
    }

    pub fn peek_dirt(&self) -> RuntimeCellDirt {
        RuntimeCellDirt(self.bits.get())
    }

    fn downgrade(&self) -> Weak<Cell<u8>> {
        Rc::downgrade(&self.bits)
    }
}

/// The typed payload of one cell.
///
/// Mirrors the concrete `ViewModelInstance*` C++ subclasses. `ViewModel` and
/// `List` payloads retain child cells/instances by identity in later #RB-1
/// slices; they enter here as the type lattice so the cell layer is complete
/// from the start.
#[derive(Debug, Clone, PartialEq)]
pub enum RuntimeViewModelCellValue {
    Number(f32),
    Boolean(bool),
    String(Vec<u8>),
    Color(u32),
    Enum(u32),
    /// Fired-counter semantics: the value is the cumulative fire count while
    /// live; C++ `ViewModelInstanceTrigger::advanced()` zeroes it each
    /// advance under suppressed delegation.
    Trigger(u64),
    SymbolListIndex(u32),
    AssetImage(u32),
    AssetFont(u32),
    Artboard(u32),
}

impl RuntimeViewModelCellValue {
    fn same_kind(&self, other: &Self) -> bool {
        std::mem::discriminant(self) == std::mem::discriminant(other)
    }
}

struct RuntimeViewModelCellState {
    value: RuntimeViewModelCellValue,
    /// C++ `ValueFlags::valueChanged`: set by writes, cleared by `advanced`.
    value_changed: bool,
    /// C++ `SuppressDelegation` depth: internal writes still cascade dirt to
    /// dependents but do not count as host-visible changes.
    suppress_depth: u32,
    dependents: Vec<Weak<Cell<u8>>>,
}

impl RuntimeViewModelCellState {
    fn cascade(&mut self, dirt: RuntimeCellDirt) {
        // Prune dead sinks while cascading; C++ relies on explicit
        // removeDependent in ~DataBind, which weak references replace.
        self.dependents.retain(|dependent| {
            let Some(bits) = dependent.upgrade() else {
                return false;
            };
            bits.set(bits.get() | dirt.0);
            true
        });
    }
}

/// One shared, retained, typed view-model property value.
#[derive(Clone)]
pub struct RuntimeViewModelCell {
    state: Rc<RefCell<RuntimeViewModelCellState>>,
}

impl RuntimeViewModelCell {
    pub fn new(value: RuntimeViewModelCellValue) -> Self {
        Self {
            state: Rc::new(RefCell::new(RuntimeViewModelCellState {
                value,
                value_changed: false,
                suppress_depth: 0,
                dependents: Vec::new(),
            })),
        }
    }

    /// Shared-identity check: two handles retain the same C++ object.
    pub fn ptr_eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.state, &other.state)
    }

    pub fn value(&self) -> RuntimeViewModelCellValue {
        self.state.borrow().value.clone()
    }

    /// C++ `ViewModelInstanceValue::hasChanged` (`ValueFlags::valueChanged`).
    pub fn has_changed(&self) -> bool {
        self.state.borrow().value_changed
    }

    /// Register a dependent's dirt sink (C++ `addDependent`). A bind created
    /// with C++ `bindsOnce()` simply never registers.
    pub fn add_dependent(&self, sink: &RuntimeCellDirtSink) {
        let mut state = self.state.borrow_mut();
        let weak = sink.downgrade();
        if !state
            .dependents
            .iter()
            .any(|existing| existing.ptr_eq(&weak))
        {
            state.dependents.push(weak);
        }
    }

    /// C++ `removeDependent` (used by `clearSource`; drop also suffices).
    pub fn remove_dependent(&self, sink: &RuntimeCellDirtSink) {
        let weak = sink.downgrade();
        self.state
            .borrow_mut()
            .dependents
            .retain(|existing| !existing.ptr_eq(&weak));
    }

    /// Write a same-kind value. Mirrors the concrete C++ setters
    /// (`ViewModelInstanceNumber::propertyValue(..)` →
    /// `propertyValueChanged()`): a genuine change sets the changed flag
    /// (unless delegation is suppressed) and cascades `BINDINGS` dirt to
    /// every dependent. Returns whether the stored value changed.
    ///
    /// A kind mismatch is a programming error in the caller (C++ types this
    /// statically per subclass) and is rejected without mutation.
    pub fn set_value(&self, value: RuntimeViewModelCellValue) -> bool {
        let mut state = self.state.borrow_mut();
        if !state.value.same_kind(&value) {
            debug_assert!(
                false,
                "view-model cell kind mismatch: {:?} <- {:?}",
                state.value, value
            );
            return false;
        }
        if state.value == value {
            return false;
        }
        state.value = value;
        if state.suppress_depth == 0 {
            state.value_changed = true;
        }
        state.cascade(RuntimeCellDirt::BINDINGS);
        true
    }

    /// C++ `ViewModelInstanceTrigger::trigger()` analog: increment the fire
    /// counter. Cascades like any other write.
    pub fn fire_trigger(&self) -> bool {
        let current = {
            let state = self.state.borrow();
            match state.value {
                RuntimeViewModelCellValue::Trigger(count) => count,
                _ => {
                    debug_assert!(false, "fire_trigger on non-trigger cell");
                    return false;
                }
            }
        };
        self.set_value(RuntimeViewModelCellValue::Trigger(
            current.saturating_add(1),
        ))
    }

    /// C++ `ViewModelInstanceValue::advanced()`: acknowledge one update pass.
    /// Clears the changed flag; a trigger cell additionally zeroes its
    /// counter under suppressed delegation
    /// (`ViewModelInstanceTrigger::advanced()` → `propertyValue(0)` inside
    /// `SuppressDelegation`), still cascading dirt to dependents exactly as
    /// the C++ setter does.
    pub fn advanced(&self) {
        let is_trigger = {
            let state = self.state.borrow();
            matches!(state.value, RuntimeViewModelCellValue::Trigger(_))
        };
        if is_trigger {
            self.with_suppressed_delegation(|cell| {
                cell.set_value(RuntimeViewModelCellValue::Trigger(0));
            });
        }
        self.state.borrow_mut().value_changed = false;
    }

    /// C++ `SuppressDelegation` scope: writes inside `body` cascade dirt but
    /// do not mark the cell host-visibly changed.
    pub fn with_suppressed_delegation(&self, body: impl FnOnce(&Self)) {
        self.state.borrow_mut().suppress_depth += 1;
        body(self);
        let mut state = self.state.borrow_mut();
        debug_assert!(state.suppress_depth > 0);
        state.suppress_depth = state.suppress_depth.saturating_sub(1);
    }

    #[cfg(test)]
    fn dependent_count(&self) -> usize {
        self.state.borrow().dependents.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shared_identity_propagates_one_write_to_every_dependent_once() {
        let cell = RuntimeViewModelCell::new(RuntimeViewModelCellValue::Number(3.0));
        let alias = cell.clone();
        assert!(cell.ptr_eq(&alias));

        let bind_a = RuntimeCellDirtSink::new();
        let bind_b = RuntimeCellDirtSink::new();
        cell.add_dependent(&bind_a);
        cell.add_dependent(&bind_b);

        assert!(alias.set_value(RuntimeViewModelCellValue::Number(4.0)));
        assert_eq!(cell.value(), RuntimeViewModelCellValue::Number(4.0));
        assert!(bind_a.take_dirt().contains(RuntimeCellDirt::BINDINGS));
        assert!(bind_b.take_dirt().contains(RuntimeCellDirt::BINDINGS));

        // A writes; it has already consumed its own dirt, B observes exactly
        // one change — the scenario the deleted "observation cursor" polled
        // for, delivered here by the dependency edge alone.
        assert!(bind_a.peek_dirt().is_empty());
        assert!(bind_b.peek_dirt().is_empty());
    }

    #[test]
    fn unchanged_writes_do_not_cascade() {
        let cell = RuntimeViewModelCell::new(RuntimeViewModelCellValue::Number(4.0));
        let bind = RuntimeCellDirtSink::new();
        cell.add_dependent(&bind);
        assert!(!cell.set_value(RuntimeViewModelCellValue::Number(4.0)));
        assert!(bind.peek_dirt().is_empty());
        assert!(!cell.has_changed());
    }

    #[test]
    fn duplicate_registration_is_ignored_and_drop_unregisters() {
        let cell = RuntimeViewModelCell::new(RuntimeViewModelCellValue::Boolean(false));
        let bind = RuntimeCellDirtSink::new();
        cell.add_dependent(&bind);
        cell.add_dependent(&bind);
        assert_eq!(cell.dependent_count(), 1);

        drop(bind);
        cell.set_value(RuntimeViewModelCellValue::Boolean(true));
        assert_eq!(cell.dependent_count(), 0, "dead sinks prune on cascade");
    }

    #[test]
    fn explicit_remove_dependent_stops_cascade() {
        let cell = RuntimeViewModelCell::new(RuntimeViewModelCellValue::Color(0xff000000));
        let bind = RuntimeCellDirtSink::new();
        cell.add_dependent(&bind);
        cell.remove_dependent(&bind);
        cell.set_value(RuntimeViewModelCellValue::Color(0xffffffff));
        assert!(bind.peek_dirt().is_empty());
    }

    #[test]
    fn trigger_advanced_zeroes_counter_without_host_visible_change() {
        let cell = RuntimeViewModelCell::new(RuntimeViewModelCellValue::Trigger(0));
        let bind = RuntimeCellDirtSink::new();
        cell.add_dependent(&bind);

        assert!(cell.fire_trigger());
        assert!(cell.fire_trigger());
        assert_eq!(cell.value(), RuntimeViewModelCellValue::Trigger(2));
        assert!(cell.has_changed());
        assert!(bind.take_dirt().contains(RuntimeCellDirt::BINDINGS));

        // C++ ViewModelInstanceTrigger::advanced(): propertyValue(0) under
        // SuppressDelegation, then the changed flag clears. Dependents still
        // observe the zeroing write as dirt.
        cell.advanced();
        assert_eq!(cell.value(), RuntimeViewModelCellValue::Trigger(0));
        assert!(!cell.has_changed());
        assert!(bind.take_dirt().contains(RuntimeCellDirt::BINDINGS));
    }

    #[test]
    fn suppressed_writes_cascade_dirt_but_stay_host_invisible() {
        let cell = RuntimeViewModelCell::new(RuntimeViewModelCellValue::Number(1.0));
        let bind = RuntimeCellDirtSink::new();
        cell.add_dependent(&bind);

        cell.with_suppressed_delegation(|cell| {
            assert!(cell.set_value(RuntimeViewModelCellValue::Number(2.0)));
        });
        assert!(!cell.has_changed(), "suppressed write is not host-visible");
        assert!(
            bind.take_dirt().contains(RuntimeCellDirt::BINDINGS),
            "dependents still observe suppressed writes"
        );
    }

    #[test]
    fn kind_mismatch_is_rejected_without_mutation() {
        let cell = RuntimeViewModelCell::new(RuntimeViewModelCellValue::Number(1.0));
        let bind = RuntimeCellDirtSink::new();
        cell.add_dependent(&bind);
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            cell.set_value(RuntimeViewModelCellValue::Boolean(true))
        }));
        // Debug builds assert; release builds reject silently. Either way the
        // stored value and dirt are untouched.
        if let Ok(changed) = result {
            assert!(!changed);
            assert_eq!(cell.value(), RuntimeViewModelCellValue::Number(1.0));
            assert!(bind.peek_dirt().is_empty());
        }
    }
}
