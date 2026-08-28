use std::{
    cell::RefCell,
    rc::{Rc, Weak},
};

use crate::mechanical_port::source::{
    animation::state_machine_instance::RuntimeStateMachineInstanceWeakHandle,
    core::CoreHandle,
    data_bind::{
        data_bind::DataBind, data_bind_context::DataBindContext,
        data_context::RuntimeDataContextHandle,
    },
};

pub const NONE: u32 = 0;
pub use super::data_bind::{BINDINGS, BINDINGS_TARGET, DEPENDENTS};

#[derive(Clone)]
pub enum DataBindContainerOwner {
    Authored(CoreHandle),
    StateMachine(RuntimeStateMachineInstanceWeakHandle),
}

impl DataBindContainerOwner {
    fn container(&self) -> Option<DataBindContainer> {
        match self {
            Self::Authored(owner) => owner.data_bind_container(),
            Self::StateMachine(owner) => owner.data_bind_container(),
        }
    }

    pub fn data_binds(&self) -> Vec<CoreHandle> {
        self.container()
            .map(|container| container.data_binds())
            .unwrap_or_default()
    }

    pub fn bind_data_binds_from_context(&self, context: RuntimeDataContextHandle) {
        self.container()
            .expect("live binding container")
            .bind_data_binds_from_context(context);
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

    pub fn add_dirty_data_bind(&self, bind: CoreHandle) {
        bind.with_mut(|bind| self.add_dirty_data_bind_borrowed(bind.as_data_bind_mut().unwrap()));
    }

    pub fn add_dirty_data_bind_borrowed(&self, bind: &mut DataBind) {
        let Some(container) = self.container() else {
            return;
        };
        if let Self::Authored(owner) = self {
            if let Some(dirty) = owner.artboard_dirty_handle() {
                if let Some(order) = bind
                    .target()
                    .and_then(|target| target.component_graph_order())
                {
                    dirty.on_component_dirty_at(order);
                }
            } else {
                // DataConverter::addDirtyDataBind marks its parent first.
                // This is the same retained field written by bindFromContext,
                // accessible without borrowing the converter during a setter.
                let parent = container.0.borrow().parent_data_bind.clone();
                if let Some(parent) = parent {
                    parent.with_mut(|parent| {
                        let parent = parent.as_data_bind_mut().unwrap();
                        parent.add_dirt(
                            DEPENDENTS
                                | if parent.target_origin() {
                                    BINDINGS_TARGET
                                } else {
                                    BINDINGS
                                },
                            false,
                        );
                    });
                }
            }
        }
        container.add_dirty_data_bind_borrowed(bind);
    }

    pub fn rebuild_data_bind(&self, bind: CoreHandle) {
        let context = match self {
            Self::Authored(owner) => owner
                .with(|owner| owner.as_artboard().and_then(|owner| owner.data_context()))
                .flatten(),
            Self::StateMachine(owner) => owner
                .with_instance(|owner| owner.data_context_handle())
                .flatten(),
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
    persisting: Vec<CoreHandle>,
    dirty_to_source: Vec<CoreHandle>,
    pending_dirty_to_source: Vec<CoreHandle>,
    dirty: Vec<CoreHandle>,
    pending_dirty: Vec<CoreHandle>,
    pending_additions: Vec<CoreHandle>,
    pending_removals: Vec<CoreHandle>,
    data_context: Option<RuntimeDataContextHandle>,
    is_processing: bool,
}

impl DataBindContainer {
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
        for bind in self.data_binds() {
            DataBind::unbind_handle(&bind);
            // Retire after detaching, while source observers could still use
            // the bind's live identity. Drop then releases its owned converter.
            bind.remove_occurrence();
        }
    }

    pub fn unbind_data_binds(&self) {
        for bind in self.data_binds() {
            DataBind::unbind_handle(&bind);
        }
        self.0.borrow_mut().data_context = None;
    }

    pub fn bind_data_binds_from_context(&self, context: RuntimeDataContextHandle) {
        for bind in self.data_binds() {
            DataBindContext::bind_from_context_handle(&bind, Some(context.clone()));
        }
        self.0.borrow_mut().data_context = Some(context);
    }

    pub fn advance_data_binds(&self, elapsed: f32) -> bool {
        let mut updated = false;
        for bind in self.data_binds() {
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
                state.pending_removals.push(bind);
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
                Self::erase(&mut state.persisting, &bind);
                bind_value.set_in_persisting_list(false);
            }
            if bind_value.in_dirty_list() {
                Self::erase(&mut state.dirty_to_source, &bind);
                Self::erase(&mut state.pending_dirty_to_source, &bind);
                Self::erase(&mut state.dirty, &bind);
                Self::erase(&mut state.pending_dirty, &bind);
                bind_value.set_in_dirty_list(false);
            }
            bind_value.set_container(None);
        });
    }

    pub fn add_data_bind(&self, bind: CoreHandle) {
        {
            let mut state = self.0.borrow_mut();
            if state.is_processing {
                state.pending_additions.push(bind);
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
            self.0.borrow_mut().persisting.push(bind.clone());
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
        let active = {
            let mut state = self.0.borrow_mut();
            if state.is_processing
                || (state.persisting.is_empty()
                    && state.dirty_to_source.is_empty()
                    && state.dirty.is_empty())
            {
                return;
            }
            state.is_processing = true;
            (
                state.persisting.clone(),
                state.dirty_to_source.clone(),
                state.dirty.clone(),
            )
        };
        for bind in active.0 {
            let can_skip = bind
                .with(|bind| bind.as_data_bind().unwrap().can_skip())
                .unwrap_or(false);
            if !can_skip {
                DataBind::update_data_bind_handle(&bind, apply_target_to_source);
            }
        }
        for bind in active.1.into_iter().chain(active.2) {
            bind.with_mut(|bind| bind.as_data_bind_mut().unwrap().set_in_dirty_list(false));
            DataBind::update_data_bind_handle(&bind, apply_target_to_source);
        }
        let additions = {
            let mut state = self.0.borrow_mut();
            state.dirty_to_source.clear();
            state.dirty.clear();
            let state = &mut *state;
            if !state.pending_dirty_to_source.is_empty() {
                std::mem::swap(
                    &mut state.dirty_to_source,
                    &mut state.pending_dirty_to_source,
                );
            }
            if !state.pending_dirty.is_empty() {
                std::mem::swap(&mut state.dirty, &mut state.pending_dirty);
            }
            state.is_processing = false;
            // Exactly the upstream deferred addition queue, not delayed user callbacks.
            std::mem::take(&mut state.pending_additions)
        };
        for bind in additions {
            self.add_data_bind(bind);
        }
        let removals = std::mem::take(&mut self.0.borrow_mut().pending_removals);
        for bind in removals {
            self.remove_data_bind(bind);
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
        if bind.to_source() && bind.in_persisting_list() || bind.in_dirty_list() {
            return;
        }
        let handle = bind.base.base.handle().expect("registered DataBind");
        {
            let mut state = self.0.borrow_mut();
            let list = if bind.to_source() {
                if state.is_processing {
                    &mut state.pending_dirty_to_source
                } else {
                    &mut state.dirty_to_source
                }
            } else if state.is_processing {
                &mut state.pending_dirty
            } else {
                &mut state.dirty
            };
            list.push(handle);
        }
        bind.set_in_dirty_list(true);
    }

    pub fn data_binds(&self) -> Vec<CoreHandle> {
        self.0.borrow().data_binds.clone()
    }

    pub fn rebind(&self) {}
    pub fn relink_data_context(&self) {}
    pub fn rebuild_data_bind(&self, _data_bind: CoreHandle) {}
}
