use crate::mechanical_port::source::{
    animation::state_machine_instance::RuntimeStateMachineInstanceWeakHandle, core::CoreHandle,
    data_bind::data_context::RuntimeDataContextHandle,
};

pub const NONE: u32 = 0;
pub const DEPENDENTS: u32 = 1;
pub const BINDINGS: u32 = 2;
pub const BINDINGS_TARGET: u32 = 4;

#[derive(Clone)]
pub enum DataBindContainerOwner {
    Authored(CoreHandle),
    StateMachine(RuntimeStateMachineInstanceWeakHandle),
}

impl DataBindContainerOwner {
    pub fn add_dirty_data_bind(&self, bind: CoreHandle) {
        match self {
            Self::Authored(container) => {
                container.with_mut(|container| {
                    if let Some(container) = container.as_bind_container_mut() {
                        container.add_dirty_data_bind(bind);
                    }
                });
            }
            Self::StateMachine(container) => {
                container.with_instance_mut(|container| container.add_dirty_data_bind(bind));
            }
        }
    }

    pub fn rebuild_data_bind(&self, bind: CoreHandle) {
        match self {
            Self::Authored(container) => {
                container.with_mut(|container| {
                    if let Some(container) = container.as_bind_container_mut() {
                        container.rebuild_data_bind(bind);
                    }
                });
            }
            Self::StateMachine(container) => {
                container.with_instance_mut(|container| container.rebuild_data_bind(bind));
            }
        }
    }
}

#[derive(Default)]
pub struct DataBindContainer {
    owner: Option<DataBindContainerOwner>,
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
    pub fn set_owner(&mut self, owner: CoreHandle) {
        self.owner = Some(DataBindContainerOwner::Authored(owner));
    }

    pub fn set_state_machine_owner(&mut self, owner: RuntimeStateMachineInstanceWeakHandle) {
        self.owner = Some(DataBindContainerOwner::StateMachine(owner));
    }

    pub fn delete_data_binds(&mut self) {
        for bind in self.data_binds.drain(..) {
            bind.with_mut(|bind| {
                if let Some(bind) = bind.as_data_bind_mut() {
                    bind.unbind();
                    bind.set_container(None);
                }
            });
        }
        self.persisting.clear();
        self.dirty_to_source.clear();
        self.pending_dirty_to_source.clear();
        self.dirty.clear();
        self.pending_dirty.clear();
        self.pending_additions.clear();
        self.pending_removals.clear();
    }

    pub fn unbind_data_binds(&mut self) {
        for bind in &self.data_binds {
            bind.with_mut(|bind| {
                if let Some(bind) = bind.as_data_bind_mut() {
                    bind.unbind();
                }
            });
        }
        self.data_context = None;
    }

    pub fn bind_data_binds_from_context(&mut self, context: RuntimeDataContextHandle) {
        for bind in &self.data_binds {
            let context = context.clone();
            bind.with_mut(|bind| {
                if let Some(bind) = bind.as_data_bind_context_mut() {
                    bind.bind_from_context(Some(context));
                }
            });
        }
        self.data_context = Some(context);
    }

    pub fn advance_data_binds(&mut self, elapsed: f32) -> bool {
        let mut updated = false;
        for bind in &self.data_binds {
            updated |= bind
                .with_mut(|bind| {
                    bind.as_data_bind_mut()
                        .is_some_and(|bind| bind.advance(elapsed))
                })
                .unwrap_or(false);
        }
        updated
    }

    fn erase(list: &mut Vec<CoreHandle>, bind: &CoreHandle) {
        list.retain(|item| item != bind);
    }

    pub fn remove_data_bind(&mut self, bind: CoreHandle) {
        if self.is_processing {
            self.pending_removals.push(bind);
            return;
        }
        Self::erase(&mut self.data_binds, &bind);
        bind.with_mut(|object| {
            let Some(bind_value) = object.as_data_bind_mut() else {
                return;
            };
            if bind_value.in_persisting_list() {
                Self::erase(&mut self.persisting, &bind);
                bind_value.set_in_persisting_list(false);
            }
            if bind_value.in_dirty_list() {
                Self::erase(&mut self.dirty_to_source, &bind);
                Self::erase(&mut self.pending_dirty_to_source, &bind);
                Self::erase(&mut self.dirty, &bind);
                Self::erase(&mut self.pending_dirty, &bind);
                bind_value.set_in_dirty_list(false);
            }
            bind_value.set_container(None);
        });
    }

    pub fn add_data_bind(&mut self, bind: CoreHandle) {
        if self.is_processing {
            self.pending_additions.push(bind);
            return;
        }
        if self.data_binds.contains(&bind) {
            return;
        }
        self.data_binds.push(bind.clone());
        let context = self.data_context.clone();
        let owner = self.owner.clone();
        let mut should_update = false;
        bind.with_mut(|object| {
            let Some(bind) = object.as_data_bind_mut() else {
                return;
            };
            if bind.to_source() && !bind.target_supports_push() {
                self.persisting
                    .push(self.data_binds.last().unwrap().clone());
                bind.set_in_persisting_list(true);
            }
            bind.set_container(owner);
        });
        if let Some(context) = context {
            bind.with_mut(|object| {
                if let Some(bind) = object.as_data_bind_context_mut() {
                    bind.bind_from_context(Some(context));
                    should_update = true;
                }
            });
        }
        if should_update {
            self.update_data_bind(bind, true);
        }
    }

    fn update_data_bind(&mut self, bind: CoreHandle, apply_target_to_source: bool) {
        bind.with_mut(|bind| {
            let Some(bind) = bind.as_data_bind_mut() else {
                return;
            };
            let dirt = bind.dirt();
            if dirt & DEPENDENTS == DEPENDENTS {
                bind.update_dependents();
            }
            let wants = apply_target_to_source
                && (bind.in_persisting_list() || dirt & BINDINGS_TARGET == BINDINGS_TARGET);
            if wants && !bind.source_to_target_runs_first() {
                bind.update_source_binding(false);
            }
            if dirt != NONE {
                bind.set_dirt(NONE);
                bind.update(dirt);
            }
            if wants && bind.source_to_target_runs_first() {
                bind.update_source_binding(false);
            }
        });
    }

    pub fn update_data_binds(&mut self, apply_target_to_source: bool) {
        if self.is_processing {
            return;
        }
        if self.persisting.is_empty() && self.dirty_to_source.is_empty() && self.dirty.is_empty() {
            return;
        }
        self.is_processing = true;
        for bind in self.persisting.clone() {
            let can_skip = bind
                .with(|bind| bind.as_data_bind().is_some_and(|bind| bind.can_skip()))
                .unwrap_or(false);
            if !can_skip {
                self.update_data_bind(bind, apply_target_to_source);
            }
        }
        for bind in self.dirty_to_source.clone() {
            bind.with_mut(|bind| {
                if let Some(bind) = bind.as_data_bind_mut() {
                    bind.set_in_dirty_list(false);
                }
            });
            self.update_data_bind(bind, apply_target_to_source);
        }
        for bind in self.dirty.clone() {
            bind.with_mut(|bind| {
                if let Some(bind) = bind.as_data_bind_mut() {
                    bind.set_in_dirty_list(false);
                }
            });
            self.update_data_bind(bind, apply_target_to_source);
        }
        self.dirty_to_source.clear();
        self.dirty.clear();
        if !self.pending_dirty_to_source.is_empty() {
            std::mem::swap(&mut self.dirty_to_source, &mut self.pending_dirty_to_source);
        }
        if !self.pending_dirty.is_empty() {
            std::mem::swap(&mut self.dirty, &mut self.pending_dirty);
        }
        self.is_processing = false;
        for bind in std::mem::take(&mut self.pending_additions) {
            self.add_data_bind(bind);
        }
        for bind in std::mem::take(&mut self.pending_removals) {
            self.remove_data_bind(bind);
        }
    }

    pub fn sort_data_binds(&mut self) {
        let mut to_source = 0;
        for index in 0..self.data_binds.len() {
            let is_to_source = self.data_binds[index]
                .with(|bind| bind.as_data_bind().is_some_and(|bind| bind.to_source()))
                .unwrap_or(false);
            if is_to_source {
                if index != to_source {
                    self.data_binds.swap(to_source, index);
                }
                to_source += 1;
            }
        }
    }

    pub fn add_dirty_data_bind(&mut self, bind: CoreHandle) {
        let state = bind.with_mut(|bind| {
            let bind = bind.as_data_bind_mut()?;
            if bind.to_source() && bind.in_persisting_list() || bind.in_dirty_list() {
                return None;
            }
            let to_source = bind.to_source();
            bind.set_in_dirty_list(true);
            Some(to_source)
        });
        let Some(Some(to_source)) = state else {
            return;
        };
        let list = if to_source {
            if self.is_processing {
                &mut self.pending_dirty_to_source
            } else {
                &mut self.dirty_to_source
            }
        } else if self.is_processing {
            &mut self.pending_dirty
        } else {
            &mut self.dirty
        };
        list.push(bind);
    }

    pub fn data_binds(&self) -> Vec<CoreHandle> {
        self.data_binds.clone()
    }

    pub fn rebind(&mut self) {}

    pub fn relink_data_context(&mut self) {}

    pub fn rebuild_data_bind(&mut self, _data_bind: CoreHandle) {}
}
