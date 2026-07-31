// Direct Rust owner for pinned C++ `src/viewmodel/viewmodel_instance_list_item.cpp`.
// Per-occurrence identity and authored child ownership/parent registration.

#[derive(Debug)]
pub(super) struct RuntimeOwnedViewModelListItem {
    occurrence_identity: u64,
    instance: Rc<RefCell<RuntimeOwnedViewModelInstance>>,
    authored_source_object_id: Option<u32>,
    child_relay: Rc<RuntimeOwnedViewModelParentRelay>,
    parent_registered: bool,
}

impl RuntimeOwnedViewModelListItem {
    pub(super) fn new(instance: Rc<RefCell<RuntimeOwnedViewModelInstance>>) -> Self {
        use std::sync::atomic::{AtomicU64, Ordering};
        static NEXT_OCCURRENCE_IDENTITY: AtomicU64 = AtomicU64::new(0);
        let child_relay = Rc::clone(&instance.borrow().parent_relay);
        Self {
            occurrence_identity: NEXT_OCCURRENCE_IDENTITY.fetch_add(1, Ordering::Relaxed),
            instance,
            authored_source_object_id: None,
            child_relay,
            parent_registered: false,
        }
    }

    fn from_authored(
        instance: Rc<RefCell<RuntimeOwnedViewModelInstance>>,
        source_object_id: u32,
    ) -> Self {
        let mut item = Self::new(instance);
        item.authored_source_object_id = Some(source_object_id);
        item
    }

    fn copy_identity_from(
        source: &Self,
        instance: Rc<RefCell<RuntimeOwnedViewModelInstance>>,
    ) -> Self {
        let child_relay = Rc::clone(&instance.borrow().parent_relay);
        Self {
            occurrence_identity: source.occurrence_identity,
            instance,
            authored_source_object_id: source.authored_source_object_id,
            child_relay,
            parent_registered: false,
        }
    }

    fn attach_parent(&mut self, parent: &Rc<RuntimeOwnedViewModelParentRelay>) {
        RuntimeOwnedViewModelParentRelay::add_parent(&self.child_relay, parent);
        self.parent_registered = true;
    }

    fn detach_parent(&mut self, parent: &Rc<RuntimeOwnedViewModelParentRelay>) {
        if self.parent_registered {
            RuntimeOwnedViewModelParentRelay::remove_parent(&self.child_relay, parent);
            self.parent_registered = false;
        }
    }

    fn disarm_parent_registration(&mut self) {
        self.parent_registered = false;
    }
}
