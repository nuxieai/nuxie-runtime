use std::{
    cell::{Ref, RefCell},
    rc::{Rc, Weak},
};

use crate::mechanical_port::source::{
    animation::state_machine_instance::RuntimeStateMachineInstanceWeakHandle,
    core::CoreHandle,
    data_bind::{
        data_bind::DataBind, data_bind_context::DataBindContext,
        data_context::RuntimeDataContextHandle,
    },
    sidecar::Sidecar,
};

pub const NONE: u32 = 0;
#[cfg(any(test, feature = "testing"))]
std::thread_local! {
    pub(crate) static SM_DATA_BIND_UPDATES: std::cell::Cell<u64> = const { std::cell::Cell::new(0) };
}
pub use super::data_bind::{BINDINGS, BINDINGS_TARGET, DEPENDENTS};

#[derive(Clone)]
pub enum DataBindContainerOwner {
    Authored(CoreHandle),
    StateMachine(RuntimeStateMachineInstanceWeakHandle),
}

impl DataBindContainerOwner {
    pub fn data_context_changed(&self) {
        match self {
            Self::Authored(owner) => {
                if let Some(dirty) = owner.artboard_dirty_handle() {
                    dirty.wake_if_quiet_row();
                }
            }
            Self::StateMachine(owner) => owner.data_context_changed(),
        }
    }
    pub fn main_view_model_instance_changed(&self) {
        match self {
            Self::Authored(owner) => {
                if owner.artboard_dirty_handle().is_some() {
                    crate::mechanical_port::source::artboard::Artboard::main_view_model_instance_changed_handle(owner);
                }
            }
            Self::StateMachine(owner) => owner.main_view_model_instance_changed(),
        }
    }

    pub fn drop_instance_value_binds_targeting(&self, target: &CoreHandle) {
        match self {
            Self::Authored(_) => {
                if let Some(container) = self.container() {
                    container.drop_instance_value_binds_targeting(target);
                }
            }
            Self::StateMachine(owner) => owner.drop_instance_value_binds_targeting(target),
        }
    }

    pub fn data_binds_processed(&self) {
        if let Self::Authored(owner) = self {
            if owner.artboard_dirty_handle().is_some() {
                crate::mechanical_port::source::artboard::Artboard::data_binds_processed_handle(
                    owner,
                );
            }
        }
    }

    fn container(&self) -> Option<DataBindContainer> {
        match self {
            Self::Authored(owner) => owner.data_bind_container(),
            Self::StateMachine(owner) => owner.data_bind_container(),
        }
    }

    pub fn data_binds(&self) -> Vec<CoreHandle> {
        self.container()
            .map(|container| container.data_binds().to_vec())
            .unwrap_or_default()
    }

    pub fn bind_data_binds_from_context(&self, context: RuntimeDataContextHandle) {
        self.container()
            .expect("live binding container")
            .bind_data_binds_from_context(context);
    }

    pub fn bind_data_binds_from_current_context(&self) {
        self.container()
            .expect("live binding container")
            .bind_data_binds_from_current_context();
    }

    pub fn unbind_data_binds(&self) {
        self.container()
            .expect("live binding container")
            .unbind_data_binds();
    }

    pub fn advance_data_binds(&self, elapsed: f32) -> bool {
        self.container()
            .is_some_and(|container| container.advance_data_binds(elapsed))
    }

    pub fn add_data_bind(&self, bind: CoreHandle) {
        self.container()
            .expect("live binding container")
            .add_data_bind(bind);
    }

    pub fn remove_data_bind(&self, bind: CoreHandle) {
        self.container()
            .expect("live binding container")
            .remove_data_bind(bind);
    }

    pub fn sort_data_binds(&self) {
        self.container()
            .expect("live binding container")
            .sort_data_binds();
    }

    pub fn update_data_binds(&self, apply_target_to_source: bool) {
        self.container()
            .expect("live binding container")
            .update_data_binds(apply_target_to_source);
    }

    pub fn flush_data_bind(&self, bind: &CoreHandle) {
        self.container()
            .expect("live binding container")
            .flush_data_bind(bind);
    }

    pub fn add_dirty_data_bind(&self, bind: CoreHandle) {
        if let Some(container) = self.prepare_dirty_data_bind(|| {
            bind.with(|owner| owner.as_data_bind().and_then(DataBind::target))
                .flatten()
        }) {
            // Preserve the entered container, but read the child's live flags
            // after its converter parent's callback, as upstream does.
            container.add_dirty_data_bind(bind);
        }
    }

    pub fn add_dirty_data_bind_borrowed(&self, bind: &mut DataBind) {
        if let Some(container) = self.prepare_dirty_data_bind(|| bind.target()) {
            container.add_dirty_data_bind_borrowed(bind);
        }
    }

    fn prepare_dirty_data_bind(
        &self,
        target: impl FnOnce() -> Option<CoreHandle>,
    ) -> Option<DataBindContainer> {
        let container = self.container()?;
        if let Self::Authored(owner) = self {
            if let Some(dirty) = owner.artboard_dirty_handle() {
                dirty.wake_if_quiet_row();
                if let Some(order) = target().and_then(|target| target.component_graph_order()) {
                    dirty.on_component_dirty_at(order);
                }
            } else {
                // DataConverter::addDirtyDataBind marks its parent first.
                // This is the same retained field written by bindFromContext,
                // accessible without borrowing the converter during a setter.
                let parent = container.0.borrow().parent_data_bind.clone();
                if let Some(parent) = parent {
                    let dirt = parent.with(|owner| {
                        let parent = owner.as_data_bind().unwrap();
                        DEPENDENTS
                            | if parent.target_origin() {
                                BINDINGS_TARGET
                            } else {
                                BINDINGS
                            }
                    });
                    if let Some(dirt) = dirt {
                        DataBind::add_dirt_handle(&parent, dirt, false);
                    }
                }
            }
        }
        Some(container)
    }

    pub fn rebuild_data_bind(&self, bind: CoreHandle) {
        let context = match self {
            Self::Authored(owner) => owner
                .with(|owner| owner.as_artboard().and_then(|owner| owner.data_context()))
                .flatten(),
            Self::StateMachine(owner) => owner.data_context_handle(),
        };
        DataBindContext::bind_from_context_handle(&bind, context);
    }
}

/// The source container's queues are one retained field allocation, not a
/// snapshot of an Artboard or converter. Property notifications can mutate
/// these exact queues synchronously while their enclosing Core setter runs.
#[derive(Clone, Default)]
pub struct DataBindContainer(Rc<RefCell<DataBindContainerState>>);

#[derive(Clone, Default)]
pub(crate) struct DataBindContainerWeak(Weak<RefCell<DataBindContainerState>>);

impl DataBindContainerWeak {
    pub(crate) fn upgrade(&self) -> Option<DataBindContainer> {
        self.0.upgrade().map(DataBindContainer)
    }
}

#[derive(Default)]
struct DataBindContainerState {
    owner: Option<DataBindContainerOwner>,
    parent_data_bind: Option<CoreHandle>,
    data_binds: Vec<CoreHandle>,
    dirty: Vec<CoreHandle>,
    queues: Sidecar<DataBindQueues>,
    data_context: Option<RuntimeDataContextHandle>,
    is_processing: bool,
}

/// Cold queues allocate only on first use; the common toTarget dirty queue
/// remains inline in the retained container state.
#[derive(Default)]
struct DataBindQueues {
    persisting: Vec<CoreHandle>,
    dirty_to_source: Vec<CoreHandle>,
    pending_dirty_to_source: Vec<CoreHandle>,
    pending_dirty: Vec<CoreHandle>,
    pending_additions: Vec<CoreHandle>,
    pending_removals: Vec<CoreHandle>,
    pending_deletes: Vec<CoreHandle>,
}

impl DataBindContainer {
    pub fn is_processing_data_binds(&self) -> bool {
        self.0.borrow().is_processing
    }

    pub fn remove_and_delete_data_bind(&self, bind: &CoreHandle) {
        self.remove_data_bind(bind.clone());
        {
            let mut state = self.0.borrow_mut();
            if state.is_processing {
                let deletes = &mut state.queues.ensure_allocated().pending_deletes;
                if !deletes.contains(bind) {
                    deletes.push(bind.clone());
                }
                return;
            }
        }
        DataBind::unbind_handle(bind);
        bind.remove_occurrence();
    }

    pub fn drop_instance_value_binds_targeting(&self, target: &CoreHandle) {
        let binds = self.data_binds().to_vec();
        for bind in binds {
            if bind.with(|object| {
                let bind = object.as_data_bind().unwrap();
                bind.is_instance_value_bind() && bind.target().as_ref() == Some(target)
            }) == Some(true)
            {
                self.remove_and_delete_data_bind(&bind);
            }
        }
    }

    pub fn flush_data_bind(&self, bind: &CoreHandle) {
        DataBind::update_data_bind_handle(bind, false);
    }

    pub(crate) fn downgrade(&self) -> DataBindContainerWeak {
        DataBindContainerWeak(Rc::downgrade(&self.0))
    }

    pub fn set_owner(&self, owner: CoreHandle) {
        self.0.borrow_mut().owner = Some(DataBindContainerOwner::Authored(owner.clone()));
        owner.set_data_bind_container(self.clone());
    }

    pub fn set_state_machine_owner(&self, owner: RuntimeStateMachineInstanceWeakHandle) {
        self.0.borrow_mut().owner = Some(DataBindContainerOwner::StateMachine(owner));
    }

    pub(crate) fn parent_data_bind(&self) -> Option<CoreHandle> {
        self.0.borrow().parent_data_bind.clone()
    }

    pub(crate) fn set_parent_data_bind(&self, bind: Option<CoreHandle>) {
        self.0.borrow_mut().parent_data_bind = bind;
    }

    pub fn delete_data_binds(&self) {
        let binds = self.data_binds().to_vec();
        for bind in binds {
            DataBind::unbind_handle(&bind);
            // Retire after detaching, while source observers could still use
            // the bind's live identity. Drop then releases its owned converter.
            bind.remove_occurrence();
        }
    }

    pub fn unbind_data_binds(&self) {
        let binds = self.data_binds().to_vec();
        for bind in binds {
            DataBind::unbind_handle(&bind);
        }
        self.0.borrow_mut().data_context = None;
        self.data_context_changed();
    }

    pub fn bind_data_binds_from_context(&self, context: RuntimeDataContextHandle) {
        self.set_data_bind_context(Some(context));
        self.bind_data_binds_from_current_context();
    }

    pub fn data_bind_context(&self) -> Option<RuntimeDataContextHandle> {
        self.0.borrow().data_context.clone()
    }

    /// Set the one retained context without walking existing bindings.
    pub fn set_data_bind_context(&self, context: Option<RuntimeDataContextHandle>) {
        self.0.borrow_mut().data_context = context;
        self.data_context_changed();
    }

    fn data_context_changed(&self) {
        let owner = self.0.borrow().owner.clone();
        if let Some(owner) = owner {
            owner.data_context_changed();
        }
    }

    pub fn has_data_bind_work(&self) -> bool {
        let state = self.0.borrow();
        !state.dirty.is_empty()
            || state.queues.get().is_some_and(|queues| {
                !queues.persisting.is_empty()
                    || !queues.dirty_to_source.is_empty()
                    || !queues.pending_dirty_to_source.is_empty()
                    || !queues.pending_dirty.is_empty()
                    || !queues.pending_additions.is_empty()
                    || !queues.pending_removals.is_empty()
                    || !queues.pending_deletes.is_empty()
            })
    }

    pub fn may_advance_data_binds(&self) -> bool {
        self.0.borrow().data_binds.iter().any(|bind| {
            let converter = bind
                .with(|bind| bind.as_data_bind().unwrap().converter())
                .flatten();
            converter.is_some_and(|converter| {
                converter
                    .with(|converter| {
                        converter
                            .as_data_converter_capability()
                            .unwrap()
                            .may_advance()
                    })
                    .unwrap_or(false)
            })
        })
    }

    #[cfg(any(test, feature = "testing"))]
    pub fn data_bind_updates() -> u64 {
        SM_DATA_BIND_UPDATES.with(std::cell::Cell::get)
    }

    pub fn bind_data_binds_from_current_context(&self) {
        let binds = self.data_binds().to_vec();
        for bind in binds {
            DataBindContext::bind_from_context_handle(&bind, self.data_bind_context());
        }
    }

    pub fn advance_data_binds(&self, elapsed: f32) -> bool {
        let count = self.0.borrow().data_binds.len();
        if count == 0 {
            return false;
        }
        let mut updated = false;
        for index in 0..count {
            // Keep the retained source list in place. The handle clone only
            // extends this entry's lifetime while advance can call back into
            // the runtime; it is not a snapshot of the full object list.
            let Some(bind) = self.0.borrow().data_binds.get(index).cloned() else {
                break;
            };
            updated |= DataBind::advance_handle(&bind, elapsed);
        }
        updated
    }

    fn erase(list: &mut Vec<CoreHandle>, bind: &CoreHandle) {
        list.retain(|item| item != bind);
    }

    pub fn remove_data_bind(&self, bind: CoreHandle) {
        {
            let mut state = self.0.borrow_mut();
            if state.is_processing {
                state.queues.ensure_allocated().pending_removals.push(bind);
                return;
            }
            Self::erase(&mut state.data_binds, &bind);
        }
        bind.with_mut(|object| {
            let bind_value = object
                .as_data_bind_mut()
                .expect("container owns DataBind occurrences");
            let mut state = self.0.borrow_mut();
            if bind_value.in_persisting_list() {
                if let Some(queues) = state.queues.get_mut() {
                    Self::erase(&mut queues.persisting, &bind);
                }
                bind_value.set_in_persisting_list(false);
            }
            if bind_value.in_dirty_list() {
                Self::erase(&mut state.dirty, &bind);
                if let Some(queues) = state.queues.get_mut() {
                    Self::erase(&mut queues.dirty_to_source, &bind);
                    Self::erase(&mut queues.pending_dirty_to_source, &bind);
                    Self::erase(&mut queues.pending_dirty, &bind);
                }
                bind_value.set_in_dirty_list(false);
            }
            bind_value.set_container(None);
        });
    }

    pub fn add_data_bind(&self, bind: CoreHandle) {
        {
            let mut state = self.0.borrow_mut();
            if state.is_processing {
                state.queues.ensure_allocated().pending_additions.push(bind);
                return;
            }
            state.data_binds.push(bind.clone());
        }
        let persist = bind
            .with(|object| {
                let bind = object
                    .as_data_bind()
                    .expect("container owns DataBind occurrences");
                bind.to_source() && !bind.target_supports_push()
            })
            .unwrap_or(false);
        if persist {
            self.0
                .borrow_mut()
                .queues
                .ensure_allocated()
                .persisting
                .push(bind.clone());
            bind.with_mut(|bind| {
                bind.as_data_bind_mut()
                    .unwrap()
                    .set_in_persisting_list(true)
            });
        }
        let owner = self.0.borrow().owner.clone();
        bind.with_mut(|bind| bind.as_data_bind_mut().unwrap().set_container(owner));
        let context = self.0.borrow().data_context.clone();
        if let Some(context) = context {
            if bind.with_downcast::<DataBindContext, _>(|_| ()).is_some() {
                DataBindContext::bind_from_context_handle(&bind, Some(context));
                DataBind::update_data_bind_handle(&bind, true);
            }
        }
    }

    pub fn update_data_binds(&self, apply_target_to_source: bool) {
        let state_machine_owner = {
            let state = self.0.borrow();
            if state.is_processing {
                return;
            }
            match &state.owner {
                Some(DataBindContainerOwner::StateMachine(owner)) => Some(owner.clone()),
                _ => None,
            }
        };
        if let Some(owner) = state_machine_owner {
            owner.data_binds_processing_started();
        }
        let (persisting_count, dirty_to_source_count, dirty_count) = {
            let mut state = self.0.borrow_mut();
            if state.is_processing {
                return;
            }
            let (persisting_count, dirty_to_source_count) =
                state.queues.get().map_or((0, 0), |queues| {
                    (queues.persisting.len(), queues.dirty_to_source.len())
                });
            if persisting_count == 0 && dirty_to_source_count == 0 && state.dirty.is_empty() {
                return;
            }
            state.is_processing = true;
            (persisting_count, dirty_to_source_count, state.dirty.len())
        };
        for index in 0..persisting_count {
            // add/remove and new dirt are deferred while is_processing is
            // true, exactly like the pinned retained-pointer vectors. Borrow
            // only long enough to retain the current handle, then release it
            // before update callbacks re-enter the container.
            let bind = self.0.borrow().queues.get().unwrap().persisting[index].clone();
            let can_skip = bind
                .with(|bind| bind.as_data_bind().unwrap().can_skip())
                .unwrap_or(false);
            if !can_skip {
                DataBind::update_data_bind_handle(&bind, apply_target_to_source);
            }
        }
        for index in 0..dirty_to_source_count {
            let bind = self.0.borrow().queues.get().unwrap().dirty_to_source[index].clone();
            bind.with_mut(|bind| bind.as_data_bind_mut().unwrap().set_in_dirty_list(false));
            DataBind::update_data_bind_handle(&bind, apply_target_to_source);
        }
        for index in 0..dirty_count {
            let bind = self.0.borrow().dirty[index].clone();
            bind.with_mut(|bind| bind.as_data_bind_mut().unwrap().set_in_dirty_list(false));
            DataBind::update_data_bind_handle(&bind, apply_target_to_source);
        }
        let additions = {
            let mut state = self.0.borrow_mut();
            state.dirty.clear();
            let state = &mut *state;
            // Callbacks may have allocated the sidecar during this drain.
            if let Some(queues) = state.queues.get_mut() {
                queues.dirty_to_source.clear();
                if !queues.pending_dirty_to_source.is_empty() {
                    std::mem::swap(
                        &mut queues.dirty_to_source,
                        &mut queues.pending_dirty_to_source,
                    );
                }
                if !queues.pending_dirty.is_empty() {
                    std::mem::swap(&mut state.dirty, &mut queues.pending_dirty);
                }
            }
            state.is_processing = false;
            // Exactly the upstream deferred addition queue, not delayed user callbacks.
            state
                .queues
                .get_mut()
                .map(|queues| std::mem::take(&mut queues.pending_additions))
                .unwrap_or_default()
        };
        for bind in additions {
            self.add_data_bind(bind);
        }
        let removals = self
            .0
            .borrow_mut()
            .queues
            .get_mut()
            .map(|queues| std::mem::take(&mut queues.pending_removals))
            .unwrap_or_default();
        for bind in removals {
            self.remove_data_bind(bind);
        }
        let deletes = self
            .0
            .borrow_mut()
            .queues
            .get_mut()
            .map(|queues| std::mem::take(&mut queues.pending_deletes))
            .unwrap_or_default();
        for bind in deletes {
            DataBind::unbind_handle(&bind);
            bind.remove_occurrence();
        }
        let owner = self.0.borrow().owner.clone();
        if let Some(owner) = owner {
            owner.data_binds_processed();
        }
    }

    pub fn sort_data_binds(&self) {
        let mut to_source = 0;
        let count = self.0.borrow().data_binds.len();
        for index in 0..count {
            let bind = self.0.borrow().data_binds[index].clone();
            if bind
                .with(|bind| bind.as_data_bind().unwrap().to_source())
                .unwrap_or(false)
            {
                if index != to_source {
                    self.0.borrow_mut().data_binds.swap(to_source, index);
                }
                to_source += 1;
            }
        }
    }

    pub fn add_dirty_data_bind(&self, bind: CoreHandle) {
        bind.with_mut(|bind| self.add_dirty_data_bind_borrowed(bind.as_data_bind_mut().unwrap()));
    }

    fn add_dirty_data_bind_borrowed(&self, bind: &mut DataBind) {
        let owner = self.0.borrow().owner.clone();
        if let Some(DataBindContainerOwner::StateMachine(owner)) = owner {
            owner.data_bind_dirtied();
        }
        if bind.to_source() && bind.in_persisting_list() || bind.in_dirty_list() {
            return;
        }
        let handle = bind.base.base.handle().expect("registered DataBind");
        {
            let mut state = self.0.borrow_mut();
            if !bind.to_source() && !state.is_processing {
                state.dirty.push(handle);
                bind.set_in_dirty_list(true);
                return;
            }
            let is_processing = state.is_processing;
            let queues = state.queues.ensure_allocated();
            let list = if bind.to_source() {
                if is_processing {
                    &mut queues.pending_dirty_to_source
                } else {
                    &mut queues.dirty_to_source
                }
            } else {
                &mut queues.pending_dirty
            };
            list.push(handle);
        }
        bind.set_in_dirty_list(true);
    }

    /// Borrow the retained list without cloning it. Release this guard before
    /// callbacks that can re-enter the container; snapshot explicitly if needed.
    pub fn data_binds(&self) -> Ref<'_, [CoreHandle]> {
        Ref::map(self.0.borrow(), |state| state.data_binds.as_slice())
    }

    pub fn rebind(&self) {}
    pub fn relink_data_context(&self) {}
    pub fn rebuild_data_bind(&self, _data_bind: CoreHandle) {}
}
