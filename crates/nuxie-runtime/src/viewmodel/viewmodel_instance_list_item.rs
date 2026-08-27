// Direct Rust owner for pinned C++ `src/viewmodel/viewmodel_instance_list_item.cpp`.
// Per-occurrence identity and authored child ownership/parent registration.

#[derive(Debug)]
pub(super) struct RuntimeOwnedViewModelListItem {
    occurrence_identity: u64,
    view_model_id: u32,
    view_model_instance_id: u32,
    instance: Option<Rc<RefCell<RuntimeOwnedViewModelInstance>>>,
    authored_source_object_id: Option<u32>,
    child_relay: Option<Rc<RuntimeOwnedViewModelParentRelay>>,
    parent_registered: bool,
    artboard_identity: Option<u64>,
}

impl RuntimeOwnedViewModelListItem {
    pub(super) fn new(instance: Rc<RefCell<RuntimeOwnedViewModelInstance>>) -> Self {
        Self::with_instance(Some(instance))
    }

    pub(super) fn empty() -> Self {
        Self::with_instance(None)
    }

    fn with_instance(instance: Option<Rc<RefCell<RuntimeOwnedViewModelInstance>>>) -> Self {
        use std::sync::atomic::{AtomicU64, Ordering};
        static NEXT_OCCURRENCE_IDENTITY: AtomicU64 = AtomicU64::new(0);
        let child_relay = instance
            .as_ref()
            .map(|instance| Rc::clone(&instance.borrow().parent_relay));
        Self {
            occurrence_identity: NEXT_OCCURRENCE_IDENTITY.fetch_add(1, Ordering::Relaxed),
            view_model_id: u32::MAX,
            view_model_instance_id: u32::MAX,
            instance,
            authored_source_object_id: None,
            child_relay,
            parent_registered: false,
            artboard_identity: None,
        }
    }

    fn from_authored(
        view_model_id: u32,
        view_model_instance_id: u32,
        instance: Option<Rc<RefCell<RuntimeOwnedViewModelInstance>>>,
        source_object_id: Option<u32>,
    ) -> Self {
        let mut item = Self::with_instance(instance);
        item.view_model_id = view_model_id;
        item.view_model_instance_id = view_model_instance_id;
        item.authored_source_object_id = source_object_id;
        item
    }

    fn copy_identity_from(
        source: &Self,
        instance: Option<Rc<RefCell<RuntimeOwnedViewModelInstance>>>,
    ) -> Self {
        let child_relay = instance
            .as_ref()
            .map(|instance| Rc::clone(&instance.borrow().parent_relay));
        Self {
            occurrence_identity: source.occurrence_identity,
            view_model_id: source.view_model_id,
            view_model_instance_id: source.view_model_instance_id,
            instance,
            authored_source_object_id: source.authored_source_object_id,
            child_relay,
            parent_registered: false,
            artboard_identity: source.artboard_identity,
        }
    }

    fn attach_parent(&mut self, parent: &Rc<RuntimeOwnedViewModelParentRelay>) {
        if let Some(child_relay) = self.child_relay.as_ref() {
            RuntimeOwnedViewModelParentRelay::add_parent(child_relay, parent);
            self.parent_registered = true;
        }
    }

    fn detach_parent(&mut self, parent: &Rc<RuntimeOwnedViewModelParentRelay>) {
        if self.parent_registered {
            if let Some(child_relay) = self.child_relay.as_ref() {
                RuntimeOwnedViewModelParentRelay::remove_parent(child_relay, parent);
            }
            self.parent_registered = false;
        }
    }

    fn disarm_parent_registration(&mut self) {
        self.parent_registered = false;
    }
}
