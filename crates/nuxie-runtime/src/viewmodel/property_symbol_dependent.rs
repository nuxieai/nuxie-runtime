// Direct Rust owner for pinned C++ `src/viewmodel/property_symbol_dependent.cpp`.
// Retained parent/dependent registration and recursive rebind propagation.

/// Retained structural-parent topology matching C++ `ViewModelInstance::m_parents`.
///
/// Scalar values keep their own mutation identity. Only ViewModel-reference
/// replacement walks this relay and invalidates each live containing instance.
#[derive(Debug)]
pub(super) struct RuntimeOwnedViewModelParentRelay {
    parents: RefCell<Vec<RuntimeOwnedViewModelParentEdge>>,
    dependents: RefCell<Vec<RuntimeCellDependent>>,
    observable_mutation_generation: Cell<u64>,
}

#[derive(Debug)]
struct RuntimeOwnedViewModelParentEdge {
    relay: Weak<RuntimeOwnedViewModelParentRelay>,
    edge_count: usize,
}

impl RuntimeOwnedViewModelParentRelay {
    fn new() -> Rc<Self> {
        Rc::new(Self {
            parents: RefCell::new(Vec::new()),
            dependents: RefCell::new(Vec::new()),
            observable_mutation_generation: Cell::new(0),
        })
    }

    fn add_parent(this: &Rc<Self>, parent: &Rc<Self>) {
        let mut parents = this.parents.borrow_mut();
        parents.retain(|candidate| candidate.relay.strong_count() != 0);
        if let Some(existing) = parents
            .iter()
            .position(|candidate| candidate.relay.ptr_eq(&Rc::downgrade(parent)))
        {
            parents[existing].edge_count = parents[existing]
                .edge_count
                .checked_add(1)
                .expect("view-model parent edge count overflowed");
            return;
        }
        parents.push(RuntimeOwnedViewModelParentEdge {
            relay: Rc::downgrade(parent),
            edge_count: 1,
        });
    }

    fn remove_parent(this: &Rc<Self>, parent: &Rc<Self>) {
        let parent = Rc::downgrade(parent);
        let mut parents = this.parents.borrow_mut();
        parents.retain(|candidate| candidate.relay.strong_count() != 0);
        if let Some(existing) = parents
            .iter()
            .position(|candidate| candidate.relay.ptr_eq(&parent))
        {
            if parents[existing].edge_count == 1 {
                parents.remove(existing);
            } else {
                parents[existing].edge_count -= 1;
            }
        }
    }

    pub(super) fn has_parents(&self) -> bool {
        let mut parents = self.parents.borrow_mut();
        parents.retain(|candidate| candidate.relay.strong_count() != 0);
        !parents.is_empty()
    }

    pub(super) fn observable_mutation_generation(&self) -> u64 {
        self.observable_mutation_generation.get()
    }

    pub(super) fn mark_observable_mutation(this: &Rc<Self>, generation: u64) {
        fn visit(
            relay: &Rc<RuntimeOwnedViewModelParentRelay>,
            generation: u64,
            visited: &mut BTreeSet<usize>,
        ) {
            let identity = Rc::as_ptr(relay) as usize;
            if !visited.insert(identity) {
                return;
            }
            relay.observable_mutation_generation.set(generation);
            let parents = relay
                .parents
                .borrow()
                .iter()
                .filter_map(|candidate| candidate.relay.upgrade())
                .collect::<Vec<_>>();
            relay
                .parents
                .borrow_mut()
                .retain(|candidate| candidate.relay.strong_count() != 0);
            for parent in parents {
                visit(&parent, generation, visited);
            }
        }

        visit(this, generation, &mut BTreeSet::new());
    }

    pub(super) fn add_dependent(&self, sink: &RuntimeCellDirtSink) {
        let dependent = sink.downgrade();
        let mut dependents = self.dependents.borrow_mut();
        dependents.retain(|candidate| candidate.add_dirt(RuntimeCellDirt::NONE));
        if !dependents
            .iter()
            .any(|candidate| candidate.ptr_eq(&dependent))
        {
            dependents.push(dependent);
        }
    }

    fn rebind_dependents(this: &Rc<Self>) {
        let relay = Rc::clone(this);
        if defer_host_mutation_notification(move || Self::rebind_dependents_now(&relay)) {
            return;
        }
        Self::rebind_dependents_now(this);
    }

    fn rebind_dependents_now(this: &Rc<Self>) {
        fn visit(relay: &Rc<RuntimeOwnedViewModelParentRelay>, visited: &mut BTreeSet<usize>) {
            let identity = Rc::as_ptr(relay) as usize;
            if !visited.insert(identity) {
                return;
            }
            relay
                .dependents
                .borrow_mut()
                .retain(|dependent| dependent.add_dirt(RuntimeCellDirt::BINDINGS));
            let parents = relay
                .parents
                .borrow()
                .iter()
                .filter_map(|candidate| candidate.relay.upgrade())
                .collect::<Vec<_>>();
            relay
                .parents
                .borrow_mut()
                .retain(|candidate| candidate.relay.strong_count() != 0);
            for parent in parents {
                visit(&parent, visited);
            }
        }

        visit(this, &mut BTreeSet::new());
    }
}

/// Retained property subscriptions owned by one runtime core-object listener.
///
/// C++ stores heap-allocated `PropertySymbolDependent` objects here. Rust
/// stores their source cells directly, but keeps the same explicit teardown:
/// every remap must unregister the old properties while their old instance is
/// still retained by the concrete listener.
#[derive(Debug, Default)]
pub(crate) struct RuntimeCoreObjectListener {
    properties: Vec<RuntimeViewModelCell>,
    sink: RuntimeCellDirtSink,
}

impl RuntimeCoreObjectListener {
    pub(crate) fn create_properties(
        &mut self,
        properties: impl IntoIterator<Item = RuntimeViewModelCell>,
    ) {
        self.delete_properties();
        for property in properties {
            property.add_dependent(&self.sink);
            self.properties.push(property);
        }
    }

    pub(crate) fn delete_properties(&mut self) {
        for property in self.properties.drain(..) {
            property.remove_dependent(&self.sink);
        }
        self.sink.take_dirt();
    }

    pub(crate) fn take_changed(&self) -> bool {
        self.sink.take_dirt().contains(RuntimeCellDirt::BINDINGS)
    }
}

impl Drop for RuntimeCoreObjectListener {
    fn drop(&mut self) {
        self.delete_properties();
    }
}

#[cfg(test)]
mod property_symbol_listener_tests {
    use super::*;

    #[test]
    fn creating_properties_deletes_the_previous_subscriptions_first() {
        let old = RuntimeViewModelCell::new(RuntimeViewModelCellValue::String(Vec::new().into()));
        let next = RuntimeViewModelCell::new(RuntimeViewModelCellValue::String(Vec::new().into()));
        let mut listener = RuntimeCoreObjectListener::default();

        listener.create_properties([old.clone()]);
        old.notify_bindings_value_changed();
        assert!(listener.take_changed());

        listener.create_properties([next.clone()]);
        old.notify_bindings_value_changed();
        assert!(!listener.take_changed());
        next.notify_bindings_value_changed();
        assert!(listener.take_changed());
    }
}
