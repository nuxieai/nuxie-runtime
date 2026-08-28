use crate::mechanical_port::source::{
    animation::state_machine_instance::RuntimeStateMachineInstanceWeakHandle, core::CoreHandle,
    data_bind::data_context::RuntimeDataContextHandle,
};

pub const NONE: u32 = 0;
pub use super::data_bind::{BINDINGS, BINDINGS_TARGET, DEPENDENTS};

#[derive(Clone)]
pub enum DataBindContainerOwner {
    Authored(CoreHandle),
    StateMachine(RuntimeStateMachineInstanceWeakHandle),
}

impl DataBindContainerOwner {
    fn with_container_mut<R>(&self, f: impl FnOnce(&mut DataBindContainer) -> R) -> Option<R> {
        match self {
            Self::Authored(owner) => owner.with_mut(|owner| {
                if owner.as_artboard().is_some() {
                    f(&mut owner.as_artboard_mut().unwrap().data_bind_container)
                } else {
                    f(&mut owner
                        .as_data_converter_mut()
                        .expect("a binding container is an Artboard or DataConverter")
                        .data_binds)
                }
            }),
            Self::StateMachine(owner) => {
                owner.with_instance_mut(|owner| f(&mut owner.data_bind_container))
            }
        }
    }

    /// The container stays on its actual owner. Only the list operation is
    /// borrowed; every bind/converter/script callback runs after that borrow.
    pub fn data_binds(&self) -> Vec<CoreHandle> {
        self.with_container_mut(|container| container.data_binds.clone())
            .unwrap_or_default()
    }

    pub fn bind_data_binds_from_context(&self, context: RuntimeDataContextHandle) {
        for bind in self.data_binds() {
            super::data_bind_context::DataBindContext::bind_from_context_handle(
                &bind,
                Some(context.clone()),
            );
        }
        self.with_container_mut(|container| container.data_context = Some(context));
    }

    pub fn unbind_data_binds(&self) {
        for bind in self.data_binds() {
            super::data_bind::DataBind::unbind_handle(&bind);
        }
        self.with_container_mut(|container| container.data_context = None);
    }

    pub fn advance_data_binds(&self, elapsed: f32) -> bool {
        let mut updated = false;
        for bind in self.data_binds() {
            updated |= super::data_bind::DataBind::advance_handle(&bind, elapsed);
        }
        updated
    }

    pub fn add_data_bind(&self, bind: CoreHandle) {
        let deferred = self
            .with_container_mut(|container| {
                if container.is_processing {
                    container.pending_additions.push(bind.clone());
                    true
                } else {
                    container.data_binds.push(bind.clone());
                    false
                }
            })
            .unwrap_or(true);
        if deferred {
            return;
        }
        let persist = bind
            .with(|bind| {
                let bind = bind
                    .as_data_bind()
                    .expect("container owns DataBind occurrences");
                bind.to_source() && !bind.target_supports_push()
            })
            .unwrap_or(false);
        if persist {
            self.with_container_mut(|container| container.persisting.push(bind.clone()));
            bind.with_mut(|bind| {
                bind.as_data_bind_mut()
                    .unwrap()
                    .set_in_persisting_list(true)
            });
        }
        bind.with_mut(|bind| {
            bind.as_data_bind_mut()
                .unwrap()
                .set_container(Some(self.clone()))
        });
        let context = self
            .with_container_mut(|container| container.data_context.clone())
            .flatten();
        if let Some(context) = context {
            if bind
                .with_downcast::<super::data_bind_context::DataBindContext, _>(|_| ())
                .is_some()
            {
                super::data_bind_context::DataBindContext::bind_from_context_handle(
                    &bind,
                    Some(context),
                );
                super::data_bind::DataBind::update_data_bind_handle(&bind, true);
            }
        }
    }

    pub fn remove_data_bind(&self, bind: CoreHandle) {
        // Removal has no user callbacks; retaining both short borrows here
        // cannot reenter the owning Artboard or state machine.
        self.with_container_mut(|container| container.remove_data_bind(bind));
    }

    pub fn sort_data_binds(&self) {
        self.with_container_mut(DataBindContainer::sort_data_binds);
    }

    pub fn update_data_binds(&self, apply_target_to_source: bool) {
        let active = self
            .with_container_mut(|container| {
                if container.is_processing
                    || (container.persisting.is_empty()
                        && container.dirty_to_source.is_empty()
                        && container.dirty.is_empty())
                {
                    return None;
                }
                container.is_processing = true;
                Some((
                    container.persisting.clone(),
                    container.dirty_to_source.clone(),
                    container.dirty.clone(),
                ))
            })
            .flatten();
        let Some((persisting, to_source, dirty)) = active else {
            return;
        };
        for bind in persisting {
            let can_skip = bind
                .with(|bind| bind.as_data_bind().unwrap().can_skip())
                .unwrap_or(false);
            if !can_skip {
                super::data_bind::DataBind::update_data_bind_handle(&bind, apply_target_to_source);
            }
        }
        for bind in to_source.into_iter().chain(dirty) {
            bind.with_mut(|bind| bind.as_data_bind_mut().unwrap().set_in_dirty_list(false));
            super::data_bind::DataBind::update_data_bind_handle(&bind, apply_target_to_source);
        }
        let additions = self
            .with_container_mut(|container| {
                container.dirty_to_source.clear();
                container.dirty.clear();
                if !container.pending_dirty_to_source.is_empty() {
                    std::mem::swap(
                        &mut container.dirty_to_source,
                        &mut container.pending_dirty_to_source,
                    );
                }
                if !container.pending_dirty.is_empty() {
                    std::mem::swap(&mut container.dirty, &mut container.pending_dirty);
                }
                container.is_processing = false;
                // These two swaps are upstream's explicit deferred add/remove
                // queues, not a deferred callback or temporarily empty container.
                std::mem::take(&mut container.pending_additions)
            })
            .unwrap_or_default();
        for bind in additions {
            self.add_data_bind(bind);
        }
        let removals = self
            .with_container_mut(|container| std::mem::take(&mut container.pending_removals))
            .unwrap_or_default();
        for bind in removals {
            self.remove_data_bind(bind);
        }
    }

    pub fn add_dirty_data_bind(&self, bind: CoreHandle) {
        bind.with_mut(|bind| self.add_dirty_data_bind_borrowed(bind.as_data_bind_mut().unwrap()));
    }

    pub fn add_dirty_data_bind_borrowed(&self, bind: &mut super::data_bind::DataBind) {
        if let Self::Authored(owner) = self {
            if owner
                .with(|owner| owner.as_artboard().is_some())
                .unwrap_or(false)
            {
                if let Some(target) = bind.target() {
                    let order = target
                        .with(|target| {
                            target
                                .as_component()
                                .map(|component| component.graph_order())
                        })
                        .flatten();
                    if let Some(order) = order {
                        owner.with_mut(|owner| {
                            owner
                                .as_artboard_mut()
                                .unwrap()
                                .on_component_dirty_at(order)
                        });
                    }
                }
            } else {
                let parent = owner
                    .with(|owner| owner.as_data_converter().unwrap().parent_data_bind())
                    .flatten();
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
        if bind.to_source() && bind.in_persisting_list() || bind.in_dirty_list() {
            return;
        }
        let handle = bind.base.base.handle().expect("registered DataBind");
        self.with_container_mut(|container| {
            let list = if bind.to_source() {
                if container.is_processing {
                    &mut container.pending_dirty_to_source
                } else {
                    &mut container.dirty_to_source
                }
            } else if container.is_processing {
                &mut container.pending_dirty
            } else {
                &mut container.dirty
            };
            list.push(handle);
        });
        bind.set_in_dirty_list(true);
    }

    pub fn rebuild_data_bind(&self, bind: CoreHandle) {
        let context = match self {
            Self::Authored(owner) => owner
                .with(|owner| owner.as_artboard().and_then(|owner| owner.data_context()))
                .flatten(),
            Self::StateMachine(owner) => owner
                .with_instance_mut(|owner| owner.data_context_handle())
                .flatten(),
        };
        super::data_bind_context::DataBindContext::bind_from_context_handle(&bind, context);
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
