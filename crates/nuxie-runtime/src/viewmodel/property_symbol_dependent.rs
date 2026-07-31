// Direct Rust owner for pinned C++ `src/viewmodel/property_symbol_dependent.cpp`.
// Retained parent/dependent registration and recursive rebind propagation.

/// Retained structural-parent topology matching C++ `ViewModelInstance::m_parents`.
///
/// Scalar values keep their own mutation identity. Only ViewModel-reference
/// replacement walks this relay and invalidates each live containing instance.
#[derive(Debug)]
pub(super) struct RuntimeOwnedViewModelParentRelay {
    pub(super) parents: RefCell<Vec<Weak<RuntimeOwnedViewModelParentRelay>>>,
    dependents: RefCell<Vec<RuntimeCellDependent>>,
}

impl RuntimeOwnedViewModelParentRelay {
    fn new() -> Rc<Self> {
        Rc::new(Self {
            parents: RefCell::new(Vec::new()),
            dependents: RefCell::new(Vec::new()),
        })
    }

    fn add_parent(this: &Rc<Self>, parent: &Rc<Self>) {
        let mut parents = this.parents.borrow_mut();
        parents.retain(|candidate| candidate.strong_count() != 0);
        if parents
            .iter()
            .any(|candidate| candidate.ptr_eq(&Rc::downgrade(parent)))
        {
            return;
        }
        parents.push(Rc::downgrade(parent));
    }

    fn remove_parent(this: &Rc<Self>, parent: &Rc<Self>) {
        let parent = Rc::downgrade(parent);
        this.parents
            .borrow_mut()
            .retain(|candidate| !candidate.ptr_eq(&parent) && candidate.strong_count() != 0);
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
                .filter_map(Weak::upgrade)
                .collect::<Vec<_>>();
            relay
                .parents
                .borrow_mut()
                .retain(|candidate| candidate.strong_count() != 0);
            for parent in parents {
                visit(&parent, visited);
            }
        }

        visit(this, &mut BTreeSet::new());
    }
}
