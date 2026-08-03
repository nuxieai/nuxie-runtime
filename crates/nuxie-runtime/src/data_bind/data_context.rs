//! Direct Rust owner for pinned C++ `src/data_bind/data_context.cpp`.
//!
//! C++ shares one mutable DataContext between an artboard and any state
//! machines bound to it. The owner below retains that identity, keeps main and
//! global-slot order, repairs structural-rebind registrations when entries are
//! replaced, and notifies every enrolled container. Resolution projections
//! remain immutable borrows of this owner and are rebuilt only at the bind
//! boundary.

use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::{Rc, Weak};

use nuxie_binary::RuntimeFile;

use crate::artboard::ArtboardInstance;
use crate::artboard_data_bind::RuntimeOwnedDataContext;
use crate::view_model::{RuntimeOwnedViewModelContext, RuntimeOwnedViewModelHandle};
use crate::view_model_cell::{RuntimeCellDependent, RuntimeCellDirt, RuntimeCellDirtSink};

#[derive(Debug)]
struct RuntimeStateMachineDataContextRelay {
    handle: RuntimeOwnedViewModelHandle,
    sink: RuntimeCellDirtSink,
}

impl RuntimeStateMachineDataContextRelay {
    fn new(
        handle: RuntimeOwnedViewModelHandle,
        state: Weak<RefCell<RuntimeStateMachineDataContextState>>,
    ) -> Self {
        let sink = RuntimeCellDirtSink::new();
        sink.set_before_notify(Some(Rc::new(move |dirt| {
            if let Some(state) = state.upgrade() {
                let _ = dirt;
                state
                    .borrow_mut()
                    .notify_rebind_dependents(RuntimeCellDirt::BINDINGS);
            }
            // Forward immediately and remain clean so every later structural
            // mutation reaches all still-live container registrations.
            false
        })));
        handle.add_rebind_dependent(&sink);
        Self { handle, sink }
    }
}

#[derive(Debug, Default)]
struct RuntimeStateMachineDataContextState {
    context: RuntimeOwnedViewModelContext,
    /// Slot entries whose occupying instance belongs to a different authored
    /// ViewModel. C++ addresses the slot independently of the occupant.
    unusual_slot_handles: BTreeMap<usize, RuntimeOwnedViewModelHandle>,
    dependent_sinks: Vec<RuntimeCellDependent>,
    main_rebind_relay: Option<RuntimeStateMachineDataContextRelay>,
    global_rebind_relays: BTreeMap<usize, RuntimeStateMachineDataContextRelay>,
}

impl RuntimeStateMachineDataContextState {
    fn sync_rebind_relays(&mut self, state: &Weak<RefCell<Self>>) {
        self.main_rebind_relay = self.context.main_handle().cloned().map(|handle| {
            self.main_rebind_relay
                .take()
                .filter(|relay| relay.handle.ptr_eq(&handle))
                .unwrap_or_else(|| RuntimeStateMachineDataContextRelay::new(handle, state.clone()))
        });

        let mut prior_globals = std::mem::take(&mut self.global_rebind_relays);
        let mut slot_handles = self
            .context
            .global_slot_handles()
            .map(|(slot, handle)| (slot, handle.clone()))
            .collect::<BTreeMap<_, _>>();
        for (&slot, handle) in &self.unusual_slot_handles {
            slot_handles.insert(slot, handle.clone());
        }
        self.global_rebind_relays = slot_handles
            .into_iter()
            .map(|(slot, handle)| {
                let relay = prior_globals
                    .remove(&slot)
                    .filter(|relay| relay.handle.ptr_eq(&handle))
                    .unwrap_or_else(|| {
                        RuntimeStateMachineDataContextRelay::new(handle, state.clone())
                    });
                (slot, relay)
            })
            .collect();
    }

    fn notify_rebind_dependents(&mut self, dirt: RuntimeCellDirt) {
        self.dependent_sinks
            .retain(|dependent| dependent.add_dirt(dirt));
    }
}

/// Mutable shared DataContext identity used by the runtime frame loop.
#[derive(Debug, Clone)]
pub(crate) struct RuntimeStateMachineDataContext {
    state: Rc<RefCell<RuntimeStateMachineDataContextState>>,
}

impl Default for RuntimeStateMachineDataContext {
    fn default() -> Self {
        Self::from_owned_context(RuntimeOwnedViewModelContext::default())
    }
}

impl RuntimeStateMachineDataContext {
    pub(crate) fn from_owned_context(context: RuntimeOwnedViewModelContext) -> Self {
        Self::from_parts(context, BTreeMap::new())
    }

    fn from_parts(
        context: RuntimeOwnedViewModelContext,
        unusual_slot_handles: BTreeMap<usize, RuntimeOwnedViewModelHandle>,
    ) -> Self {
        let state = Rc::new(RefCell::new(RuntimeStateMachineDataContextState {
            context,
            unusual_slot_handles,
            ..RuntimeStateMachineDataContextState::default()
        }));
        state
            .borrow_mut()
            .sync_rebind_relays(&Rc::downgrade(&state));
        Self { state }
    }

    pub(crate) fn detached_snapshot(&self) -> Self {
        let state = self.state.borrow();
        Self::from_parts(state.context.clone(), state.unusual_slot_handles.clone())
    }

    pub(crate) fn add_rebind_dependent(&self, sink: &RuntimeCellDirtSink) {
        let dependent = sink.downgrade();
        let mut state = self.state.borrow_mut();
        state
            .dependent_sinks
            .retain(|candidate| candidate.add_dirt(RuntimeCellDirt::NONE));
        if !state
            .dependent_sinks
            .iter()
            .any(|candidate| candidate.ptr_eq(&dependent))
        {
            state.dependent_sinks.push(dependent);
        }
    }

    pub(crate) fn add_artboard_rebind_dependent(&self, artboard: &mut ArtboardInstance) {
        artboard.artboard_owned_view_model_rebind_sink = RuntimeCellDirtSink::new();
        self.add_rebind_dependent(&artboard.artboard_owned_view_model_rebind_sink);
    }

    pub(crate) fn set_main(&self, handle: RuntimeOwnedViewModelHandle) {
        let weak_state = Rc::downgrade(&self.state);
        let mut state = self.state.borrow_mut();
        state.context.set_main_handle(handle);
        state.sync_rebind_relays(&weak_state);
    }

    pub(crate) fn main_handle(&self) -> Option<RuntimeOwnedViewModelHandle> {
        self.state.borrow().context.main_handle().cloned()
    }

    pub(crate) fn set_global_named(
        &self,
        file: &RuntimeFile,
        name: &str,
        handle: RuntimeOwnedViewModelHandle,
    ) -> bool {
        let weak_state = Rc::downgrade(&self.state);
        let mut state = self.state.borrow_mut();
        if !state.context.set_global_named_handle(file, name, handle) {
            return false;
        }
        if let Some(slot) = file
            .view_models()
            .iter()
            .position(|view_model| view_model.object.string_property("name") == Some(name))
        {
            state.unusual_slot_handles.remove(&slot);
        }
        state.sync_rebind_relays(&weak_state);
        true
    }

    pub(crate) fn unset_global_named(&self, file: &RuntimeFile, name: &str) -> bool {
        let weak_state = Rc::downgrade(&self.state);
        let mut state = self.state.borrow_mut();
        if !state.context.unset_global_named(file, name) {
            return false;
        }
        if let Some(slot) = file
            .view_models()
            .iter()
            .position(|view_model| view_model.object.string_property("name") == Some(name))
        {
            state.unusual_slot_handles.remove(&slot);
        }
        state.sync_rebind_relays(&weak_state);
        true
    }

    pub(crate) fn complete_for_artboard(&self, file: &RuntimeFile, artboard_index: usize) -> bool {
        let weak_state = Rc::downgrade(&self.state);
        let mut state = self.state.borrow_mut();
        let unusual_slot_handles = state.unusual_slot_handles.clone();
        for (slot, handle) in unusual_slot_handles {
            state.context.set_global_slot_handle(file, slot, handle);
        }
        if !state.context.complete_for_artboard(file, artboard_index) {
            return false;
        }
        state.sync_rebind_relays(&weak_state);
        true
    }

    pub(crate) fn global_slot_handle(&self, slot: usize) -> Option<RuntimeOwnedViewModelHandle> {
        let state = self.state.borrow();
        state
            .unusual_slot_handles
            .get(&slot)
            .or_else(|| state.context.global_slot_handle(slot))
            .cloned()
    }

    pub(crate) fn projection(&self) -> RuntimeOwnedDataContext {
        RuntimeOwnedDataContext::from_owned_context(&self.state.borrow().context)
    }

    #[cfg(test)]
    pub(crate) fn snapshot(&self) -> RuntimeOwnedViewModelContext {
        self.state.borrow().context.clone()
    }

    #[cfg(test)]
    pub(crate) fn ptr_eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.state, &other.state)
    }

    #[cfg(test)]
    pub(crate) fn mark_main_rebind_for_test(&self) {
        let dependent = self
            .state
            .borrow()
            .main_rebind_relay
            .as_ref()
            .map(|relay| relay.sink.downgrade());
        if let Some(dependent) = dependent {
            dependent.add_dirt(RuntimeCellDirt::BINDINGS);
        }
    }

    #[cfg(test)]
    pub(crate) fn main_rebind_dependent_for_test(&self) -> Option<RuntimeCellDependent> {
        self.state
            .borrow()
            .main_rebind_relay
            .as_ref()
            .map(|relay| relay.sink.downgrade())
    }

    #[cfg(test)]
    pub(crate) fn shares_state_for_test(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.state, &other.state)
    }

    #[cfg(test)]
    pub(crate) fn set_unusual_slot_for_test(
        &self,
        slot: usize,
        handle: RuntimeOwnedViewModelHandle,
    ) {
        let weak_state = Rc::downgrade(&self.state);
        let mut state = self.state.borrow_mut();
        state.unusual_slot_handles.insert(slot, handle);
        state.sync_rebind_relays(&weak_state);
    }
}
