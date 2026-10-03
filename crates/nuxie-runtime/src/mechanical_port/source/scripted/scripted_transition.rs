use std::collections::HashMap;

use crate::mechanical_port::source::{
    advance_flags::AdvanceFlags,
    animation::state_machine_instance::RuntimeStateMachineInstanceHandle,
    artboard::{Artboard, RuntimeArtboardInstanceHandle},
    component::{ActiveLayoutOwner, Component, ComponentOccurrenceHandle},
    component_dirt::ComponentDirt,
    core::{CoreHandle, CoreObject},
    drawable_flag::DrawableFlag,
    file::{File, RuntimeFileWeakHandle},
    generated::{
        core_registry::CoreCapabilities, scripted::scripted_transition_base::ScriptedTransitionBase,
    },
    layout_component::LayoutComponent,
    renderer::Renderer,
    scripted::scripted_object::{ScriptProtocol, ScriptUpdateRequestHost, ScriptedObject},
    viewmodel::{
        viewmodel_instance::ViewModelInstance,
        viewmodel_instance_list_item::ViewModelInstanceListItem,
    },
};

/// A source in the combined design-child/list-instance solo set.
#[derive(Clone, Default)]
struct TransitionSource {
    design: Option<CoreHandle>,
    instance: Option<RuntimeArtboardInstanceHandle>,
}
impl TransitionSource {
    fn empty(&self) -> bool {
        self.design.is_none() && self.instance.is_none()
    }
    fn instance_key(&self) -> Option<CoreHandle> {
        self.instance
            .as_ref()
            .map(RuntimeArtboardInstanceHandle::core_handle)
    }
}
impl PartialEq for TransitionSource {
    fn eq(&self, other: &Self) -> bool {
        self.design == other.design && self.instance_key() == other.instance_key()
    }
}

pub struct ScriptedTransition {
    pub base: ScriptedTransitionBase,
    current: TransitionSource,
    from: TransitionSource,
    to: TransitionSource,
    transitioning: bool,
    manage_to: bool,
    active_list_index: i32,
    file: Option<RuntimeFileWeakHandle>,
    list_items: Vec<CoreHandle>,
    list_instances: HashMap<CoreHandle, RuntimeArtboardInstanceHandle>,
    list_state_machines: HashMap<CoreHandle, RuntimeStateMachineInstanceHandle>,
    list_instances_by_index: Vec<Option<RuntimeArtboardInstanceHandle>>,
    artboards_map: HashMap<u32, CoreHandle>,
    pending_active_component_changed: bool,
    pending_list_source_changed: bool,
}
impl Default for ScriptedTransition {
    fn default() -> Self {
        Self {
            base: ScriptedTransitionBase::default(),
            current: TransitionSource::default(),
            from: TransitionSource::default(),
            to: TransitionSource::default(),
            transitioning: false,
            manage_to: true,
            active_list_index: -1,
            file: None,
            list_items: Vec::new(),
            list_instances: HashMap::new(),
            list_state_machines: HashMap::new(),
            list_instances_by_index: Vec::new(),
            artboards_map: HashMap::new(),
            pending_active_component_changed: false,
            pending_list_source_changed: false,
        }
    }
}

fn hidden(child: &Option<CoreHandle>, value: bool) {
    if let Some(child) = child {
        child.with_mut(|child| {
            let drawable = child
                .as_drawable_mut()
                .expect("transition design child is drawable");
            let flags = drawable.base.drawable_flags();
            drawable.base.set_drawable_flags_value(if value {
                flags | DrawableFlag::HIDDEN.0
            } else {
                flags & !DrawableFlag::HIDDEN.0
            });
        });
    }
}
fn collapse_child(child: &CoreHandle, value: bool) {
    ComponentOccurrenceHandle::Authored(child.clone()).collapse(value);
}

crate::mechanical_port::source::transform_component::impl_transform_update!(
    ScriptedTransition,
    |owner, dirt| {
        crate::mechanical_port::source::transform_component::update_transform_super::<Self>(
            owner, dirt,
        );
        if owner.is_alive() {
            crate::mechanical_port::source::scripted::scripted_drawable::ScriptedDrawable::update_after_super_occurrence(owner, dirt);
            if owner.is_alive() {
                Self::update_after_super_occurrence(owner, dirt);
            }
        }
    },
    crate::mechanical_port::source::transform_component::update_local_transform::<Self>,
    crate::mechanical_port::source::node::Node::update_world_transform_occurrence::<Self>,
    crate::mechanical_port::source::transform_component::compose_world_transform::<Self>,
    crate::mechanical_port::source::transform_component::update_constraints_super::<Self>
);

impl ScriptedTransition {
    // Registry handle setters drain these immediately after releasing the
    // occurrence borrow, before dispatching property notifications.
    pub fn active_component_id_changed(&mut self) {
        self.pending_active_component_changed = true;
    }
    pub fn list_source_changed(&mut self) {
        self.pending_list_source_changed = true;
    }
    pub fn take_pending_property_changes(&mut self) -> (bool, bool) {
        (
            std::mem::take(&mut self.pending_active_component_changed),
            std::mem::take(&mut self.pending_list_source_changed),
        )
    }
    pub(crate) fn has_pending_property_changes(&self) -> bool {
        self.pending_active_component_changed || self.pending_list_source_changed
    }
    fn read<R>(owner: &CoreHandle, f: impl FnOnce(&Self) -> R) -> R {
        owner.with_downcast(f).expect("live ScriptedTransition")
    }
    fn write<R>(owner: &CoreHandle, f: impl FnOnce(&mut Self) -> R) -> R {
        owner.with_downcast_mut(f).expect("live ScriptedTransition")
    }
    fn component(&self) -> &Component {
        self.base.base.as_component().expect("transition component")
    }
    fn children(&self) -> &[CoreHandle] {
        self.base
            .base
            .as_container_component()
            .expect("transition container")
            .children()
    }
    fn design_children(&self) -> Vec<CoreHandle> {
        self.children().iter().filter(|child| {
            child.is_type_of(crate::mechanical_port::source::generated::nested_artboard_base::NestedArtboardBase::TYPE_KEY)
        }).cloned().collect()
    }
    fn design_child_count(&self) -> usize {
        self.design_children().len()
    }
    fn source_index(&self, source: &TransitionSource) -> i32 {
        if let Some(key) = source.instance_key() {
            return self
                .list_instances_by_index
                .iter()
                .position(|item| item.as_ref().is_some_and(|item| item.core_handle() == key))
                .map_or(-1, |index| (self.design_child_count() + index) as i32);
        }
        source
            .design
            .as_ref()
            .and_then(|design| {
                self.design_children()
                    .iter()
                    .position(|child| child == design)
            })
            .map_or(-1, |index| index as i32)
    }
    pub fn active_component(&self) -> Option<CoreHandle> {
        let active = self
            .component()
            .artboard_handle()?
            .with_downcast::<Artboard, _>(|artboard| {
                artboard.resolve_handle(self.base.active_component_id())
            })
            .flatten()?;
        self.design_children()
            .into_iter()
            .find(|child| child == &active)
    }
    pub fn set_file(&mut self, file: RuntimeFileWeakHandle) {
        self.file = Some(file);
    }
    pub fn file(&self) -> Option<RuntimeFileWeakHandle> {
        self.file.clone()
    }
    pub fn script_protocol(&self) -> ScriptProtocol {
        ScriptProtocol::Transition
    }

    fn propagate_collapse_occurrence(owner: &CoreHandle, collapse: bool) {
        Self::propagate_collapse_with_layout_occurrence(owner, collapse, None);
    }
    fn propagate_collapse_with_layout_occurrence(
        owner: &CoreHandle,
        collapse: bool,
        mut active_layout: Option<(&mut ActiveLayoutOwner<'_>, &CoreHandle)>,
    ) {
        let (active, children) = Self::read(owner, |this| {
            let active = if collapse {
                None
            } else {
                // A direct Artboard collapse keeps that actual root borrowed.
                // Resolve the selection against it instead of its arena slot.
                let active_root_selection =
                    active_layout.as_ref().and_then(|(active, handle)| {
                        if this.component().artboard_handle().as_ref() != Some(*handle) {
                            return None;
                        }
                        let ActiveLayoutOwner::Artboard(artboard) = active else {
                            return None;
                        };
                        Some(
                            artboard
                                .resolve_handle(this.base.active_component_id())
                                .filter(|selected| {
                                    this.design_children().iter().any(|child| child == selected)
                                }),
                        )
                    });
                active_root_selection.unwrap_or_else(|| this.active_component())
            };
            (active, this.children().to_vec())
        });
        for child in children {
            let design = child.is_type_of(
                crate::mechanical_port::source::generated::nested_artboard_base::NestedArtboardBase::TYPE_KEY,
            );
            let value = if !design {
                collapse
            } else if collapse {
                Some(&child) != active.as_ref()
            } else {
                // C++ resolves active once, but reads the transition pair at
                // each child's call site, after earlier collapse callbacks.
                let Some(transition_child) = owner.with_downcast::<Self, _>(|this| {
                    this.transitioning
                        && (Some(&child) == this.from.design.as_ref()
                            || Some(&child) == this.to.design.as_ref())
                }) else {
                    return;
                };
                !transition_child && Some(&child) != active.as_ref()
            };
            if let Some((active, active_handle)) = active_layout.as_mut() {
                ComponentOccurrenceHandle::Authored(child)
                    .collapse_from_layout(value, active, active_handle);
            } else {
                collapse_child(&child, value);
            }
        }
    }
    pub fn collapse_after_component_occurrence(owner: &CoreHandle, value: bool) {
        Self::propagate_collapse_occurrence(owner, value);
    }
    pub(crate) fn collapse_after_component_from_layout_occurrence(
        owner: &CoreHandle,
        value: bool,
        active: &mut ActiveLayoutOwner<'_>,
        active_handle: &CoreHandle,
    ) {
        Self::propagate_collapse_with_layout_occurrence(
            owner,
            value,
            Some((active, active_handle)),
        );
    }
    fn propagate_current_collapse(owner: &CoreHandle) {
        let collapsed = Self::read(owner, |this| this.component().is_collapsed());
        Self::propagate_collapse_occurrence(owner, collapsed);
    }
    fn dirt(owner: &CoreHandle, value: ComponentDirt) {
        Self::write(owner, |this| {
            this.base.base.add_scripted_dirt(value, false);
        });
    }
    fn wake(owner: &CoreHandle) {
        Self::write(owner, |this| this.base.base.wake_advance());
    }
    fn find_list_artboard(&mut self, item: &CoreHandle) -> Option<CoreHandle> {
        let id = item
            .with_downcast::<ViewModelInstanceListItem, _>(|item| item.view_model_instance())
            .flatten()?
            .with_downcast::<ViewModelInstance, _>(|model| model.base.view_model_id())?;
        if let Some(source) = self.artboards_map.get(&id) {
            return Some(source.clone());
        }
        for source in self.file.as_ref()?.with_file(File::artboards)? {
            if source.with_downcast::<Artboard, _>(|source| source.base.view_model_id()) == Some(id)
            {
                self.artboards_map.insert(id, source.clone());
                return Some(source);
            }
        }
        None
    }
    fn ensure_list_instance(
        owner: &CoreHandle,
        index: i32,
    ) -> Option<RuntimeArtboardInstanceHandle> {
        let (item, existing) = Self::read(owner, |this| {
            let item = this.list_items.get(usize::try_from(index).ok()?)?.clone();
            let existing = this
                .list_instances_by_index
                .get(index as usize)
                .cloned()
                .flatten()
                .or_else(|| this.list_instances.get(&item).cloned());
            Some((item, existing))
        })?;
        if existing.is_some() {
            return existing;
        }
        let source = Self::write(owner, |this| this.find_list_artboard(&item))?;
        let instance = Artboard::nested_instance_from_handle(&source)?;
        let parent = Self::read(owner, |this| this.component().artboard_handle());
        if let Some(parent) = parent {
            let context = parent
                .with_downcast::<Artboard, _>(Artboard::data_context)
                .flatten();
            let model = item
                .with_downcast::<ViewModelInstanceListItem, _>(|item| item.view_model_instance())
                .flatten();
            instance.bind_view_model_instance_with_parent(model, context);
            instance.update_data_binds(true);
        }
        instance.with_artboard_mut(|artboard| artboard.set_frame_origin(false));
        let default_index =
            instance.with_artboard(|artboard| artboard.base.default_state_machine_index());
        let machine = instance.state_machine_instance_handle(default_index.max(0) as usize);
        if let Some(machine) = &machine {
            if let Some(context) = instance.data_context() {
                machine.with_instance_mut(|machine| machine.set_data_context_handle(context));
            }
            if let Some(container) = machine.downgrade().data_bind_container() {
                container.update_data_binds(false);
            }
        }
        Self::write(owner, |this| {
            if let Some(machine) = machine {
                this.list_state_machines
                    .insert(instance.core_handle(), machine);
            }
            this.list_instances.insert(item, instance.clone());
            if this.list_instances_by_index.len() <= index as usize {
                this.list_instances_by_index
                    .resize(this.list_items.len(), None);
            }
            this.list_instances_by_index[index as usize] = Some(instance.clone());
        });
        Some(instance)
    }
    fn intended_source(owner: &CoreHandle) -> TransitionSource {
        let index = Self::read(owner, |this| this.active_list_index);
        if let Some(instance) = Self::ensure_list_instance(owner, index) {
            return TransitionSource {
                design: None,
                instance: Some(instance),
            };
        }
        TransitionSource {
            design: Self::read(owner, Self::active_component),
            instance: None,
        }
    }
    fn apply_selection(owner: &CoreHandle) {
        let next = Self::intended_source(owner);
        if Self::read(owner, |this| this.current == next) {
            return;
        }
        let (prev, ready) = Self::write(owner, |this| {
            let prev = std::mem::replace(&mut this.current, next.clone());
            (
                prev,
                this.base.base.scripted.scripting_vm().is_some()
                    && this.base.base.scripted.self_ref() != 0,
            )
        });
        if ready && !prev.empty() && !next.empty() {
            Self::start_transition(owner, prev, next);
            return;
        }
        let pair = Self::write(owner, |this| {
            if !this.transitioning {
                return None;
            }
            this.transitioning = false;
            this.manage_to = true;
            Some((std::mem::take(&mut this.from), std::mem::take(&mut this.to)))
        });
        if let Some((from, to)) = pair {
            hidden(&from.design, false);
            hidden(&to.design, false);
        }
        Self::propagate_current_collapse(owner);
        Self::dirt(owner, ComponentDirt::PAINT);
    }
    fn start_transition(owner: &CoreHandle, from: TransitionSource, to: TransitionSource) {
        let previous = Self::write(owner, |this| {
            let previous = if this.transitioning {
                this.from.clone()
            } else {
                TransitionSource::default()
            };
            this.from = from.clone();
            this.to = to.clone();
            this.transitioning = true;
            previous
        });
        if previous != from && previous != to {
            hidden(&previous.design, false);
            if let Some(design) = previous.design {
                collapse_child(&design, true);
            }
        }
        for child in [&from.design, &to.design].into_iter().flatten() {
            collapse_child(child, false);
        }
        Self::propagate_current_collapse(owner);
        Self::call_changed(owner, &from, &to);
        let manage_to = to.instance.is_some() || Self::script_manages_to(owner);
        Self::write(owner, |this| this.manage_to = manage_to);
        hidden(&from.design, true);
        hidden(&to.design, manage_to);
        Self::wake(owner);
        Self::dirt(owner, ComponentDirt::COMPONENTS);
    }
    fn complete_transition(owner: &CoreHandle) {
        let pair = Self::write(owner, |this| {
            if !this.transitioning {
                return None;
            }
            this.transitioning = false;
            this.manage_to = true;
            Some((std::mem::take(&mut this.from), std::mem::take(&mut this.to)))
        });
        let Some((from, to)) = pair else {
            return;
        };
        hidden(&from.design, false);
        hidden(&to.design, false);
        Self::write(owner, |this| this.current = to);
        Self::propagate_current_collapse(owner);
        Self::dirt(owner, ComponentDirt::PAINT);
    }
    fn forget_instance(owner: &CoreHandle, key: &CoreHandle) {
        if Self::read(owner, |this| {
            this.transitioning
                && (this.from.instance_key().as_ref() == Some(key)
                    || this.to.instance_key().as_ref() == Some(key))
        }) {
            Self::complete_transition(owner);
        }
        Self::write(owner, |this| {
            if this.current.instance_key().as_ref() == Some(key) {
                this.current = TransitionSource::default();
            }
        });
    }
    pub fn update_list_occurrence(owner: &CoreHandle, list: &[CoreHandle]) {
        if Self::read(owner, |this| this.list_items == list) {
            return;
        }
        let old = Self::write(owner, |this| {
            std::mem::replace(&mut this.list_items, list.to_vec())
        });
        for item in old {
            if list.contains(&item) {
                continue;
            }
            if let Some(instance) =
                Self::read(owner, |this| this.list_instances.get(&item).cloned())
            {
                Self::forget_instance(owner, &instance.core_handle());
                let machine = Self::write(owner, |this| {
                    this.list_state_machines.remove(&instance.core_handle())
                });
                drop(machine);
                Self::write(owner, |this| {
                    // C++'s index cache is non-owning. Release its Rust retains
                    // before deleting the owner, rather than postponing child
                    // teardown until the full cache rebuild below.
                    for cached in &mut this.list_instances_by_index {
                        if cached
                            .as_ref()
                            .is_some_and(|cached| cached.core_handle() == instance.core_handle())
                        {
                            *cached = None;
                        }
                    }
                    this.list_instances.remove(&item);
                });
            }
        }
        Self::write(owner, |this| {
            this.list_instances_by_index = vec![None; list.len()]
        });
        for (index, item) in list.iter().enumerate() {
            item.with_downcast_mut::<ViewModelInstanceListItem, _>(|item| {
                item.assign_list_index(index as u32)
            });
            Self::write(owner, |this| {
                this.list_instances_by_index[index] = this.list_instances.get(item).cloned()
            });
        }
        Self::dirt(owner, ComponentDirt::COMPONENTS);
        Self::apply_selection(owner);
        Self::wake(owner);
    }
    pub fn list_source_changed_occurrence(owner: &CoreHandle) {
        if Self::read(owner, |this| this.base.list_source() == u32::MAX) {
            Self::update_list_occurrence(owner, &[]);
        }
    }
    pub fn active_component_id_changed_occurrence(owner: &CoreHandle) {
        Self::write(owner, |this| {
            if this.active_component().is_some() {
                this.active_list_index = -1;
            }
        });
        Self::apply_selection(owner);
        Self::recollect_owning_layout_occurrence(owner);
    }
    pub fn on_added_clean_occurrence(owner: &CoreHandle) {
        let current = Self::intended_source(owner);
        Self::write(owner, |this| this.current = current);
        Self::propagate_current_collapse(owner);
        Self::recollect_owning_layout_occurrence(owner);
    }
    fn recollect_owning_layout_occurrence(owner: &CoreHandle) {
        let mut parent = Self::read(owner, |this| this.component().parent_handle());
        while let Some(current) = parent {
            if current.is_type_of(crate::mechanical_port::source::generated::layout_component_base::LayoutComponentBase::TYPE_KEY) {
                LayoutComponent::sync_layout_children_occurrence(&current); return;
            }
            parent = current
                .with(|current| current.as_component().and_then(Component::parent_handle))
                .flatten();
        }
    }
    fn set_active_id(owner: &CoreHandle, value: u32) {
        if Self::write(owner, |this| this.base.set_active_component_id_value(value)) {
            Self::active_component_id_changed_occurrence(owner);
            if let Some(observers) = owner.property_observers() {
                observers.notify(ScriptedTransitionBase::ACTIVE_COMPONENT_ID_PROPERTY_KEY);
            }
        }
    }
    pub fn update_by_index_occurrence(owner: &CoreHandle, index: usize) {
        let (count, total) = Self::read(owner, |this| {
            (
                this.design_child_count(),
                this.design_child_count() + this.list_items.len(),
            )
        });
        if index >= total {
            return;
        }
        if index < count {
            let selected = Self::write(owner, |this| {
                this.active_list_index = -1;
                let child = this.design_children().get(index)?.clone();
                this.component()
                    .artboard_handle()?
                    .with_downcast::<Artboard, _>(|artboard| artboard.id_of(&child))
            });
            if let Some(id) = selected {
                Self::set_active_id(owner, id);
            }
        } else {
            Self::write(owner, |this| {
                this.active_list_index = (index - count) as i32
            });
            Self::set_active_id(owner, 0);
        }
        Self::apply_selection(owner);
    }
    pub fn update_by_name_occurrence(owner: &CoreHandle, name: &str) {
        let selected = Self::read(owner, |this| {
            let artboard = this.component().artboard_handle()?;
            this.design_children()
                .iter()
                .find(|child| {
                    child
                        .with(|child| {
                            child
                                .as_component()
                                .is_some_and(|child| child.base.name() == name)
                        })
                        .unwrap_or(false)
                })
                .and_then(|child| {
                    artboard.with_downcast::<Artboard, _>(|artboard| artboard.id_of(child))
                })
        });
        if let Some(id) = selected {
            Self::write(owner, |this| this.active_list_index = -1);
            Self::set_active_id(owner, id);
        }
    }
    pub fn get_active_child_index(&self) -> i32 {
        if self.active_list_index >= 0 {
            return self.design_child_count() as i32 + self.active_list_index;
        }
        self.active_component()
            .and_then(|active| {
                self.design_children()
                    .iter()
                    .position(|child| child == &active)
            })
            .map_or(-1, |index| index as i32)
    }
    pub fn get_active_child_name(&self) -> String {
        self.component()
            .artboard_handle()
            .and_then(|artboard| {
                artboard
                    .with_downcast::<Artboard, _>(|artboard| {
                        artboard.resolve_handle(self.base.active_component_id())
                    })
                    .flatten()
            })
            .and_then(|active| {
                active
                    .with(|active| {
                        active
                            .as_component()
                            .map(|component| component.base.name().to_owned())
                    })
                    .flatten()
            })
            .unwrap_or_default()
    }

    fn child_ref(
        owner: &CoreHandle,
        source: &TransitionSource,
    ) -> crate::scripting::ScriptTransitionChildRef {
        if let Some(design) = &source.design {
            return design
                .with(|object| {
                    let design = object.as_nested_artboard().expect("nested design child");
                    crate::scripting::ScriptTransitionChildRef {
                        artboard: design.source_artboard(),
                        transform: nuxie_render_api::Mat2D(
                            *design
                                .as_world_transform_component()
                                .expect("nested transform")
                                .world_transform()
                                .values(),
                        ),
                    }
                })
                .expect("live transition design child");
        }
        Self::read(owner, |this| crate::scripting::ScriptTransitionChildRef {
            artboard: source.instance_key(),
            transform: nuxie_render_api::Mat2D(
                *this
                    .base
                    .base
                    .as_world_transform_component()
                    .expect("transition transform")
                    .world_transform()
                    .values(),
            ),
        })
    }
    fn script_manages_to(owner: &CoreHandle) -> bool {
        let instance = Self::read(owner, |this| this.base.base.scripted.runtime_instance());
        instance.is_none_or(|instance| instance.borrow_mut().transition_manages_to())
    }
    fn call_changed(owner: &CoreHandle, from: &TransitionSource, to: &TransitionSource) {
        let (instance, from_index, to_index) = Self::read(owner, |this| {
            (
                this.base.base.scripted.runtime_instance(),
                this.source_index(from),
                this.source_index(to),
            )
        });
        let Some(instance) = instance else {
            return;
        };
        let direction = if from_index < 0 || to_index < 0 {
            0
        } else if to_index > from_index {
            1
        } else if to_index < from_index {
            -1
        } else {
            0
        };
        let from = Self::child_ref(owner, from);
        let to = Self::child_ref(owner, to);
        let mut host = ScriptUpdateRequestHost::default();
        let _ = instance
            .borrow_mut()
            .call_transition_changed(&from, &to, direction, &mut host);
        if host.take_requested() {
            ScriptedObject::apply_update_request(owner);
        }
    }
    pub fn advance_occurrence(owner: &CoreHandle, elapsed: f32, flags: AdvanceFlags) -> bool {
        let mut keep_going = false;
        if Self::read(owner, |this| this.transitioning) {
            if flags.0 & AdvanceFlags::ADVANCE_NESTED.0 == 0 {
                Self::complete_transition(owner);
            } else if elapsed == 0.0 {
                keep_going = true;
            } else {
                let instance = Self::read(owner, |this| {
                    let scripted = &this.base.base.scripted;
                    (scripted.advances() && scripted.self_ref() != 0)
                        .then(|| scripted.runtime_instance())
                        .flatten()
                });
                let mut host = ScriptUpdateRequestHost::default();
                let script_going = instance.is_some_and(|instance| {
                    instance
                        .borrow_mut()
                        .call_advance_truthy(elapsed, &mut host)
                        .unwrap_or(false)
                });
                if host.take_requested() {
                    ScriptedObject::apply_update_request(owner);
                }
                Self::dirt(owner, ComponentDirt::PAINT);
                if script_going {
                    keep_going = true;
                } else {
                    Self::complete_transition(owner);
                }
            }
        }
        let sources = Self::read(owner, |this| {
            if this.transitioning {
                vec![this.from.clone(), this.to.clone()]
            } else {
                vec![this.current.clone()]
            }
        });
        for source in sources {
            let Some(instance) = source.instance else {
                continue;
            };
            let machine = Self::read(owner, |this| {
                this.list_state_machines
                    .get(&instance.core_handle())
                    .cloned()
            });
            if let Some(machine) = machine.filter(|_| flags.0 & AdvanceFlags::ADVANCE_NESTED.0 != 0)
            {
                keep_going |= machine.advance_and_apply_view_models(elapsed, false);
            } else {
                keep_going |= instance.advance_internal(elapsed, flags);
            }
        }
        keep_going
    }
    pub fn draw_occurrence(owner: &CoreHandle, renderer: &mut Renderer) {
        let draw =
            Self::read(owner, |this| {
                if this.transitioning && this.base.base.scripted.draws() {
                    this.base.base.scripted.runtime_instance().map(|instance| {
                        (instance, this.from.clone(), this.to.clone(), this.manage_to)
                    })
                } else {
                    None
                }
            });
        if let Some((instance, from, to, manage_to)) = draw {
            let from = Self::child_ref(owner, &from);
            let mut to = Self::child_ref(owner, &to);
            if !manage_to {
                to.artboard = None;
            }
            let factory = Self::read(owner, |this| this.component().artboard_handle())
                .and_then(|artboard| {
                    artboard
                        .with_downcast::<Artboard, _>(Artboard::factory)
                        .flatten()
                })
                .expect("imported transition has a factory");
            let mut host = ScriptUpdateRequestHost::default();
            renderer.save();
            factory.with_factory_mut(|factory| {
                let _ = instance
                    .borrow_mut()
                    .call_transition_draw(factory, renderer, &from, &to, &mut host);
            });
            renderer.restore();
            if host.take_requested() {
                ScriptedObject::apply_update_request(owner);
            }
            return;
        }
        let current = Self::read(owner, |this| this.current.clone());
        if let Some(instance) = &current.instance {
            let child = Self::child_ref(owner, &current);
            renderer.save();
            renderer.transform(child.transform);
            let host = Self::read(owner, |this| this.component().artboard_handle()).expect("transition artboard");
            Artboard::draw_hosted_handle(&host, &instance.core_handle(), renderer);
            renderer.restore();
        }
    }
    pub fn will_draw(&self) -> bool {
        self.base.base.will_draw() && (self.transitioning || self.current.instance.is_some())
    }
    pub fn update_after_super_occurrence(owner: &CoreHandle, value: ComponentDirt) {
        let (sources, opacity) = Self::read(owner, |this| {
            let sources = if this.transitioning {
                vec![this.from.clone(), this.to.clone()]
            } else {
                vec![this.current.clone()]
            };
            (
                sources,
                this.base
                    .base
                    .as_transform_component()
                    .expect("transition transform")
                    .render_opacity(),
            )
        });
        for source in sources {
            let Some(instance) = source.instance else {
                continue;
            };
            if value.contains(ComponentDirt::RENDER_OPACITY) {
                crate::mechanical_port::source::generated::core_registry::CoreRegistry::set_double_handle(
                    &instance.core_handle(),
                    crate::mechanical_port::source::generated::world_transform_component_base::WorldTransformComponentBase::OPACITY_PROPERTY_KEY as i32,
                    opacity,
                );
            }
            if value.contains(ComponentDirt::COMPONENTS) {
                instance.update_pass(false);
            }
        }
    }
    pub fn clone_definition(&self) -> Self {
        let mut clone = Self::default();
        let mut base = std::mem::take(&mut clone.base);
        base.copy(&self.base, &mut clone);
        clone.base = base;
        clone
            .base
            .base
            .scripted
            .file_asset_referencer_mut()
            .set_asset_unattached(self.base.base.scripted.script_asset());
        clone.file = self.file.clone();
        clone
    }
}

impl Drop for ScriptedTransition {
    fn drop(&mut self) {
        // Match clearListInstances: settle a live list transition before its
        // sources disappear, unhide design children, then destroy machines
        // before their owning artboards. No owner-handle borrow during Drop.
        if self.transitioning && (self.from.instance.is_some() || self.to.instance.is_some()) {
            hidden(&self.from.design, false);
            hidden(&self.to.design, false);
            self.current = std::mem::take(&mut self.to);
            self.from = TransitionSource::default();
            self.transitioning = false;
            self.manage_to = true;
            let active = if self.component().is_collapsed() {
                None
            } else {
                self.active_component()
            };
            for child in self.children() {
                let is_design = child.is_type_of(crate::mechanical_port::source::generated::nested_artboard_base::NestedArtboardBase::TYPE_KEY);
                collapse_child(
                    child,
                    if is_design {
                        Some(child) != active.as_ref()
                    } else {
                        self.component().is_collapsed()
                    },
                );
            }
            self.base
                .base
                .add_scripted_dirt(ComponentDirt::PAINT, false);
        }
        if self.current.instance.is_some() {
            self.current = TransitionSource::default();
        }
        self.list_state_machines.clear();
        self.list_instances.clear();
        self.list_instances_by_index.clear();
        self.list_items.clear();
        self.artboards_map.clear();
        self.active_list_index = -1;
    }
}

#[cfg(test)]
mod keyed_setter_tests {
    use super::*;
    use crate::source::{
        animation::{
            keyed_object::{KeyedObject, KeyedObjectContext},
            keyed_property::KeyedProperty,
            keyframe_double::KeyFrameDouble,
            keyframe_id::KeyFrameId,
            keyframe_uint::KeyFrameUint,
        },
        core::CoreArena,
        core_context::CoreContext,
        generated::{
            core_registry::CoreRegistry,
            world_transform_component_base::WorldTransformComponentBase,
        },
    };

    struct Context<'a> {
        arena: &'a CoreArena,
        target: CoreHandle,
    }
    impl CoreContext for Context<'_> {
        fn core_arena(&self) -> &CoreArena {
            self.arena
        }
        fn resolve_handle(&self, id: u32) -> Option<CoreHandle> {
            (id == 1).then(|| self.target.clone())
        }
    }
    impl KeyedObjectContext for Context<'_> {
        fn resolves_object(&self, id: u32) -> bool {
            self.resolve_handle(id).is_some()
        }
        fn resolve_object(&mut self, id: u32) -> Option<CoreHandle> {
            self.resolve_handle(id)
        }
        fn object_supports_property(&self, _id: u32, _key: u32) -> bool {
            true
        }
        fn overrides_keyed_interpolation(&self, object: &CoreHandle, key: u32) -> bool {
            if key == u32::from(WorldTransformComponentBase::OPACITY_PROPERTY_KEY) {
                // The next real keyed property observes the previous setter's
                // callback, not merely the new stored ID.
                object
                    .with_downcast::<ScriptedTransition, _>(|transition| {
                        assert!(transition.list_items.is_empty());
                        assert!(!transition.has_pending_property_changes());
                    })
                    .unwrap();
            }
            false
        }
    }

    #[test]
    fn keyed_object_clean_ignores_property_status_like_upstream() {
        use crate::source::status_code::StatusCode;
        let arena = CoreArena::default();
        let target = arena.insert(ScriptedTransition::default());
        let mut context = Context {
            arena: &arena,
            target,
        };
        let frame = arena.insert(KeyFrameDouble::default());
        let mut property = KeyedProperty::default();
        property.add_key_frame(frame.clone());
        // A removed occurrence exercises the native property's failure return
        // without introducing a replacement test lifecycle implementation.
        drop(arena.remove(&frame).unwrap());
        assert_eq!(
            property.on_added_clean(&mut context),
            StatusCode::MissingObject
        );
        let mut object = KeyedObject::default();
        object.add_keyed_property(arena.insert(property));
        object.add_keyed_property(arena.insert(KeyedProperty::default()));
        assert_eq!(object.on_added_clean(&mut context), StatusCode::Ok);
    }

    #[test]
    fn id_and_held_uint_keyframes_finish_callbacks_before_next_property() {
        for use_id in [false, true] {
            for time in [0.0, 0.5] {
                let arena = CoreArena::default();
                let target = arena.insert(ScriptedTransition::default());
                let key = i32::from(ScriptedTransitionBase::LIST_SOURCE_PROPERTY_KEY);
                assert!(CoreRegistry::set_id_handle(&target, key, 0));
                let item = arena.insert(ViewModelInstanceListItem::default());
                ScriptedTransition::update_list_occurrence(&target, &[item]);
                assert_eq!(
                    target.with_downcast::<ScriptedTransition, _>(|owner| owner.list_items.len()),
                    Some(1)
                );

                let mut property = KeyedProperty::default();
                property.base.set_property_key_value(key as u32);
                for frame in [0, 60] {
                    let handle = if use_id {
                        let mut value = KeyFrameId::default();
                        value.base.set_value_value(u32::MAX);
                        arena.insert(value)
                    } else {
                        let mut value = KeyFrameUint::default();
                        value.base.set_value_value(u32::MAX);
                        arena.insert(value)
                    };
                    handle.with_mut(|owner| {
                        let keyframe = owner.as_key_frame_mut().unwrap();
                        keyframe.base.set_frame_value(frame);
                        keyframe.compute_seconds(60);
                    });
                    // Exercise both the exact-frame and interpolation dispatch.
                    CoreRegistry::set_uint_handle(&handle,
                        i32::from(crate::source::generated::animation::interpolating_keyframe_base::InterpolatingKeyFrameBase::INTERPOLATION_TYPE_PROPERTY_KEY), 1);
                    property.add_key_frame(handle);
                }
                let mut next = KeyedProperty::default();
                next.base.set_property_key_value(u32::from(
                    WorldTransformComponentBase::OPACITY_PROPERTY_KEY,
                ));
                next.add_key_frame(arena.insert(KeyFrameDouble::default()));
                let mut object = KeyedObject::default();
                object.base.set_object_id_value(1);
                object.add_keyed_property(arena.insert(property));
                object.add_keyed_property(arena.insert(next));
                object.apply(
                    &mut Context {
                        arena: &arena,
                        target: target.clone(),
                    },
                    time,
                    1.0,
                    None,
                );
                assert_eq!(
                    target.with_downcast::<ScriptedTransition, _>(|owner| owner.base.list_source()),
                    Some(u32::MAX)
                );
                // Unchanged writes neither queue nor redispatch callbacks.
                assert!(CoreRegistry::set_uint_handle(&target, key, u32::MAX));
                assert!(
                    !target
                        .with_downcast::<ScriptedTransition, _>(
                            |owner| owner.has_pending_property_changes()
                        )
                        .unwrap()
                );
            }
        }
    }

    #[test]
    fn unchanged_setter_drains_preexisting_callbacks_after_releasing_owner() {
        let arena = CoreArena::default();
        let target = arena.insert(ScriptedTransition::default());
        let key = i32::from(ScriptedTransitionBase::LIST_SOURCE_PROPERTY_KEY);
        CoreRegistry::set_id_handle(&target, key, 0);
        ScriptedTransition::update_list_occurrence(
            &target,
            &[arena.insert(ViewModelInstanceListItem::default())],
        );
        target.with_mut(|owner| {
            CoreRegistry::set_id(owner, key, u32::MAX);
            CoreRegistry::set_id(
                owner,
                i32::from(ScriptedTransitionBase::ACTIVE_COMPONENT_ID_PROPERTY_KEY),
                7,
            );
        });
        assert!(target.with_downcast::<ScriptedTransition, _>(|owner| owner.has_pending_property_changes()).unwrap());
        // The unchanged value must not hide callbacks queued by the borrowed
        // setter. Both callbacks re-enter the released target safely.
        assert!(CoreRegistry::set_id_handle(&target, key, u32::MAX));
        target
            .with_downcast::<ScriptedTransition, _>(|owner| {
                assert!(!owner.has_pending_property_changes());
                assert!(owner.list_items.is_empty());
            })
            .unwrap();
    }
}
