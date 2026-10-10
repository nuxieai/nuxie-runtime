use std::{
    cell::RefCell,
    rc::{Rc, Weak},
};

pub use crate::mechanical_port::source::component_dirt::ComponentDirt;
use crate::mechanical_port::source::{
    artboard::{Artboard, RuntimeArtboardDirtyHandle},
    container_component::ContainerComponent,
    core::CoreHandle,
    core_context::CoreContext,
    dependency_helper::{DependencyHelper, DirtDependent},
    generated::{
        artboard_base::ArtboardBase,
        component_base::{ComponentBase, ComponentBaseCallbacks},
        container_component_base::ContainerComponentBase,
        core_registry::CoreCapabilities,
    },
    importers::{artboard_importer::ArtboardImporter, import_stack::ImportStack},
    lazy_vector::{LazyVector, LazyVectorSnapshot},
    math::vec2d::Vec2D,
    status_code::StatusCode,
    text::text_modifier_range::TextModifierRange,
};
pub fn has_dirt(value: ComponentDirt, flags: ComponentDirt) -> bool {
    value.intersects(flags)
}

/// A dependency-graph occurrence is not always an authored Core object. The two
/// upstream runtime-only Component helpers retain weak identity in the graph;
/// their Shape/TextStyle owns the strong typed handle.
#[derive(Clone)]
pub enum ComponentOccurrenceHandle {
    Authored(CoreHandle),
    PathComposer(
        std::rc::Weak<
            std::cell::RefCell<crate::mechanical_port::source::shapes::path_composer::PathComposer>,
        >,
    ),
    TextVariationHelper(
        std::rc::Weak<
            std::cell::RefCell<
                crate::mechanical_port::source::text::text_variation_helper::TextVariationHelper,
            >,
        >,
    ),
}

/// Retains the dependency list across reentrant callbacks without copying it.
/// Entries remain weak occurrences and are generation-checked when visited.
pub type DependencySnapshot =
    crate::mechanical_port::source::lazy_vector::LazyVectorSnapshot<ComponentOccurrenceHandle>;

enum DirtyAction<T> {
    Callback(fn(&CoreHandle, ComponentDirt), ComponentDirt),
    Complete(T),
}

/// The most-derived owner of an active C++ `LayoutComponent` occurrence.
///
/// C++ dirt callbacks are reentrant through raw object pointers. Rust keeps
/// authored objects in `RefCell` arena slots, so a dirt cascade that returns
/// to the property setter's Layout occurrence must use the mutable owner that
/// is already on the stack. Retaining the concrete owner here also preserves
/// virtual `Artboard::onDirty` dispatch for the Artboard subclass.
pub(crate) enum ActiveLayoutOwner<'a> {
    Layout(&'a mut crate::mechanical_port::source::layout_component::LayoutComponent),
    Artboard(&'a mut Artboard),
}

impl ActiveLayoutOwner<'_> {
    fn component_add_dirt(&mut self, value: ComponentDirt, recurse: bool) -> bool {
        match self {
            Self::Layout(layout) => layout.component_add_dirt(value, recurse),
            Self::Artboard(artboard) => artboard.component_add_dirt(value, recurse),
        }
    }

    fn component_collapse(&mut self, value: bool) -> bool {
        match self {
            Self::Layout(layout) => layout.component_collapse(value),
            Self::Artboard(artboard) => artboard.component_collapse(value),
        }
    }

    pub(crate) fn is_constrained(&self) -> bool {
        match self {
            Self::Layout(layout) => !layout.constraints().is_empty(),
            Self::Artboard(artboard) => !artboard.constraints().is_empty(),
        }
    }

    pub(crate) fn collapse_after_component(&mut self, value: bool) {
        let (collapsed, handle, children) = match self {
            Self::Layout(layout) => (
                value || layout.component_is_collapsed(),
                crate::mechanical_port::source::core::CoreObject::core(&**layout).handle(),
                layout.children().to_vec(),
            ),
            Self::Artboard(artboard) => (
                value || artboard.component_is_collapsed(),
                crate::mechanical_port::source::core::CoreObject::core(&**artboard).handle(),
                artboard.children().to_vec(),
            ),
        };
        for child in children {
            let child = ComponentOccurrenceHandle::Authored(child);
            if let Some(handle) = handle.as_ref() {
                child.collapse_from_layout(collapsed, self, handle);
            } else {
                child.collapse(collapsed);
            }
        }
        // The second updateCollapsables reads the current virtual state after
        // child callbacks, while retaining the actual Artboard subclass.
        match self {
            Self::Layout(layout) => layout.component_update_collapsables(),
            Self::Artboard(artboard) => artboard.component_update_collapsables(),
        }
    }
}

impl From<CoreHandle> for ComponentOccurrenceHandle {
    fn from(value: CoreHandle) -> Self {
        Self::Authored(value)
    }
}
impl PartialEq for ComponentOccurrenceHandle {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Authored(left), Self::Authored(right)) => left == right,
            (Self::PathComposer(left), Self::PathComposer(right)) => left.ptr_eq(right),
            (Self::TextVariationHelper(left), Self::TextVariationHelper(right)) => {
                left.ptr_eq(right)
            }
            _ => false,
        }
    }
}
impl Eq for ComponentOccurrenceHandle {}
impl std::hash::Hash for ComponentOccurrenceHandle {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        use std::hash::Hash;
        std::mem::discriminant(self).hash(state);
        match self {
            Self::Authored(handle) => handle.hash(state),
            Self::PathComposer(handle) => handle.as_ptr().hash(state),
            Self::TextVariationHelper(handle) => handle.as_ptr().hash(state),
        }
    }
}
impl ComponentOccurrenceHandle {
    pub fn authored(&self) -> Option<&CoreHandle> {
        match self {
            Self::Authored(handle) => Some(handle),
            _ => None,
        }
    }
    pub fn with_component<R>(&self, f: impl FnOnce(&Component) -> R) -> Option<R> {
        match self {
            Self::Authored(handle) => handle.with(|object| object.as_component().map(f)).flatten(),
            Self::PathComposer(handle) => {
                handle.upgrade().map(|owner| f(&owner.borrow().component))
            }
            Self::TextVariationHelper(handle) => {
                handle.upgrade().map(|owner| f(&owner.borrow().component))
            }
        }
    }
    pub fn with_component_mut<R>(&self, f: impl FnOnce(&mut Component) -> R) -> Option<R> {
        match self {
            Self::Authored(handle) => handle
                .with_mut(|object| object.as_component_mut().map(f))
                .flatten(),
            Self::PathComposer(handle) => handle
                .upgrade()
                .map(|owner| f(&mut owner.borrow_mut().component)),
            Self::TextVariationHelper(handle) => handle
                .upgrade()
                .map(|owner| f(&mut owner.borrow_mut().component)),
        }
    }
    fn on_dirty(&self, _dirt: ComponentDirt) {
        // Release the composer borrow before Shape::pathChanged recursively
        // marks this helper; its retained dirt bit must already be set.
        if let Self::PathComposer(handle) = self {
            let shape = handle
                .upgrade()
                .and_then(|owner| owner.borrow_mut().dirty_shape());
            if let Some(shape) = shape {
                crate::mechanical_port::source::shapes::shape::Shape::path_changed_occurrence(
                    &shape,
                );
            }
        }
    }
    pub(crate) fn notify_artboard(&self) {
        self.with_component(Component::notify_artboard);
    }
    fn on_dirty_authored(handle: &CoreHandle, dirt: ComponentDirt) {
        let callback = handle
            .with_mut(|object| {
                if let Some(callback) = object.component_on_dirty_handler() {
                    Some(callback)
                } else {
                    object.component_on_dirty(dirt);
                    None
                }
            })
            .flatten();
        if let Some(callback) = callback {
            callback(handle, dirt);
        }
    }
    fn on_dirty_authored_from_layout(
        handle: &CoreHandle,
        dirt: ComponentDirt,
        active: &mut ActiveLayoutOwner<'_>,
        active_handle: &CoreHandle,
    ) {
        if handle.is_type_of(
            crate::mechanical_port::source::generated::constraints::constraint_base::ConstraintBase::TYPE_KEY,
        ) {
            crate::mechanical_port::source::constraints::constraint::Constraint::on_dirty_from_layout(
                handle, dirt, active, active_handle,
            );
        } else if handle.is_type_of(
            crate::mechanical_port::source::generated::shapes::path_base::PathBase::TYPE_KEY,
        ) {
            crate::mechanical_port::source::shapes::path::Path::on_dirty_from_layout(
                handle, dirt, active, active_handle,
            );
        } else if handle.is_type_of(
            crate::mechanical_port::source::generated::bones::skin_base::SkinBase::TYPE_KEY,
        ) {
            crate::mechanical_port::source::bones::skin::Skin::on_dirty_from_layout(
                handle, dirt, active, active_handle,
            );
        } else {
            Self::on_dirty_authored(handle, dirt);
        }
    }
    fn on_dirty_helper_from_layout(
        &self,
        _dirt: ComponentDirt,
        active: &mut ActiveLayoutOwner<'_>,
        active_handle: &CoreHandle,
    ) {
        if let Self::PathComposer(handle) = self {
            let shape = handle
                .upgrade()
                .and_then(|owner| owner.borrow_mut().dirty_shape());
            if let Some(shape) = shape {
                crate::mechanical_port::source::shapes::shape::Shape::path_changed_from_layout_occurrence(
                    &shape, active, active_handle,
                );
            }
        }
    }
    pub fn add_dirt(&self, value: ComponentDirt, recurse: bool) -> bool {
        if let Self::Authored(handle) = self {
            let action = handle
                .with_mut(|object| {
                    // The duplicate-bit return precedes virtual dispatch just
                    // as it does in Component::addDirt. A live receiver supplies
                    // the inherited callback; no separate type lookup is needed.
                    let dirt = object.as_component_mut()?.add_dirt_state(value)?;
                    if let Some(callback) = object.component_on_dirty_handler() {
                        return Some(DirtyAction::Callback(callback, dirt));
                    }
                    object.component_on_dirty(dirt);
                    Some(DirtyAction::Complete(
                        object
                            .as_component()?
                            .notify_and_snapshot_dependents(recurse),
                    ))
                })
                .flatten();
            let Some(action) = action else {
                return false;
            };
            let dependents = match action {
                DirtyAction::Callback(callback, dirt) => {
                    // The arena borrow and temporary strong slot reference end
                    // before a callback can edit or retire this occurrence.
                    callback(handle, dirt);
                    self.with_component(|component| {
                        component.notify_and_snapshot_dependents(recurse)
                    })
                    .unwrap_or_default()
                }
                DirtyAction::Complete(dependents) => dependents,
            };
            for dependent in dependents.iter() {
                dependent.add_dirt(value, true);
            }
            return true;
        }
        let shape = match self {
            Self::PathComposer(handle) => {
                let Some(owner) = handle.upgrade() else {
                    return false;
                };
                let mut helper = owner.borrow_mut();
                if helper.component.add_dirt_state(value).is_none() {
                    return false;
                }
                // PathComposer's onDirty first updates this helper's own gate.
                // No callback separates that gate from the dirt-state write.
                helper.dirty_shape()
            }
            Self::TextVariationHelper(handle) => {
                let Some(owner) = handle.upgrade() else {
                    return false;
                };
                if owner.borrow_mut().component.add_dirt_state(value).is_none() {
                    return false;
                }
                // TextVariationHelper inherits Component's empty onDirty.
                None
            }
            Self::Authored(_) => unreachable!(),
        };
        // Drop both the helper loan and its temporary strong owner before the
        // Shape callback can reenter or retire the occurrence.
        if let Some(shape) = shape {
            crate::mechanical_port::source::shapes::shape::Shape::path_changed_occurrence(&shape);
        }
        let dependents = self
            .with_component(|component| component.notify_and_snapshot_dependents(recurse))
            .unwrap_or_default();
        for dependent in dependents.iter() {
            dependent.add_dirt(value, true);
        }
        true
    }
    /// A scroll offset dirties its content, which in turn dirties the same
    /// active constraint through the dependency graph. Retain the real caller
    /// across that cascade instead of borrowing its Core slot again.
    pub(crate) fn add_dirt_from_scroll(
        &self,
        scroll: &mut crate::mechanical_port::source::constraints::scrolling::scroll_constraint::ScrollConstraint,
        value: ComponentDirt,
        recurse: bool,
    ) -> bool {
        let is_scroll = self.authored().is_some_and(|owner| {
            crate::mechanical_port::source::core::CoreObject::core(scroll)
                .handle()
                .as_ref()
                == Some(owner)
        });
        let changed = if is_scroll {
            scroll.component_add_dirt(value, false)
        } else {
            self.add_dirt(value, false)
        };
        if changed && recurse {
            let dependents = if is_scroll {
                scroll
                    .as_component()
                    .expect("ScrollConstraint inherits Component")
                    .dependents_snapshot()
            } else {
                self.with_component(Component::dependents_snapshot)
                    .unwrap_or_default()
            };
            for dependent in dependents.iter() {
                dependent.add_dirt_from_scroll(scroll, value, true);
            }
        }
        changed
    }

    /// A Shape property callback may synchronously dirty its paths while the
    /// Shape is already borrowed. Carry that actual owner through the same
    /// dirt cascade rather than reacquiring its arena slot in Path::onDirty.
    pub(crate) fn add_dirt_from_shape(
        &self,
        shape: &mut crate::mechanical_port::source::shapes::shape::Shape,
        value: ComponentDirt,
        recurse: bool,
    ) -> bool {
        if self
            .authored()
            .is_some_and(|owner| shape.base.handle().as_ref() == Some(owner))
        {
            return shape.component_add_dirt(value, recurse);
        }
        let Some(dirt) = self
            .with_component_mut(|component| component.add_dirt_state(value))
            .flatten()
        else {
            return false;
        };
        match self {
            Self::Authored(handle) => {
                if handle.is_type_of(
                    crate::mechanical_port::source::generated::shapes::path_base::PathBase::TYPE_KEY,
                ) {
                    crate::mechanical_port::source::shapes::path::Path::on_dirty_from_shape(
                        handle, dirt, shape,
                    );
                } else if handle.is_type_of(
                    crate::mechanical_port::source::generated::constraints::constraint_base::ConstraintBase::TYPE_KEY,
                ) {
                    crate::mechanical_port::source::constraints::constraint::Constraint::on_dirty_from_shape(
                        handle, dirt, shape,
                    );
                } else if handle.is_type_of(
                    crate::mechanical_port::source::generated::bones::skin_base::SkinBase::TYPE_KEY,
                ) {
                    crate::mechanical_port::source::bones::skin::Skin::on_dirty_from_shape(
                        handle, dirt, shape,
                    );
                } else {
                    Self::on_dirty_authored(handle, dirt);
                }
            }
            Self::PathComposer(handle) => {
                let dirty_shape = handle
                    .upgrade()
                    .and_then(|helper| helper.borrow_mut().dirty_shape());
                if let Some(dirty_shape) = dirty_shape {
                    if shape.base.handle().as_ref() == Some(&dirty_shape) {
                        shape.path_changed();
                    } else {
                        crate::mechanical_port::source::shapes::shape::Shape::path_changed_occurrence(&dirty_shape);
                    }
                }
            }
            Self::TextVariationHelper(_) => self.on_dirty(dirt),
        }
        self.notify_artboard();
        if recurse {
            let dependents = self
                .with_component(Component::dependents_snapshot)
                .unwrap_or_default();
            for dependent in dependents.iter() {
                dependent.add_dirt_from_shape(shape, value, true);
            }
        }
        true
    }

    /// A Text setter can revisit the same Text through a Constraint or a
    /// public dependency edge. Keep its actual receiver through that cascade.
    pub(crate) fn add_dirt_from_text(
        &self,
        text: &mut crate::mechanical_port::source::text::text::Text,
        value: ComponentDirt,
        recurse: bool,
    ) -> bool {
        if self.authored().is_some_and(|owner| {
            crate::mechanical_port::source::core::CoreObject::core(text)
                .handle()
                .as_ref()
                == Some(owner)
        }) {
            return text.component_add_dirt(value, recurse);
        }
        let Some(dirt) = self
            .with_component_mut(|component| component.add_dirt_state(value))
            .flatten()
        else {
            return false;
        };
        match self {
            Self::Authored(handle)
                if handle.is_type_of(
                    crate::mechanical_port::source::generated::constraints::constraint_base::ConstraintBase::TYPE_KEY,
                ) =>
            {
                crate::mechanical_port::source::constraints::constraint::Constraint::on_dirty_from_text(
                    handle, dirt, text,
                );
            }
            Self::Authored(handle) => Self::on_dirty_authored(handle, dirt),
            Self::PathComposer(_) | Self::TextVariationHelper(_) => self.on_dirty(dirt),
        }
        self.notify_artboard();
        if recurse {
            let dependents = self
                .with_component(Component::dependents_snapshot)
                .unwrap_or_default();
            for dependent in dependents.iter() {
                dependent.add_dirt_from_text(text, value, true);
            }
        }
        true
    }

    /// Carry the active most-derived Layout owner through a reentrant dirt
    /// cascade. This is the Rust ownership translation of C++'s raw-pointer
    /// call chain; it changes no callback or dependent ordering.
    pub(crate) fn add_dirt_from_layout(
        &self,
        active: &mut ActiveLayoutOwner<'_>,
        active_handle: &CoreHandle,
        value: ComponentDirt,
        recurse: bool,
    ) -> bool {
        if self.authored() == Some(active_handle) {
            return active.component_add_dirt(value, recurse);
        }
        let Some(dirt) = self
            .with_component_mut(|component| component.add_dirt_state(value))
            .flatten()
        else {
            return false;
        };
        match self {
            Self::Authored(handle) => {
                Self::on_dirty_authored_from_layout(handle, dirt, active, active_handle);
            }
            Self::PathComposer(_) | Self::TextVariationHelper(_) => {
                self.on_dirty_helper_from_layout(dirt, active, active_handle);
            }
        }
        self.notify_artboard();
        if recurse {
            let dependents = self
                .with_component(Component::dependents_snapshot)
                .unwrap_or_default();
            for dependent in dependents.iter() {
                dependent.add_dirt_from_layout(active, active_handle, value, true);
            }
        }
        true
    }
    pub fn collapse(&self, value: bool) -> bool {
        self.collapse_with_layout(value, None)
    }

    pub(crate) fn collapse_from_layout(
        &self,
        value: bool,
        active: &mut ActiveLayoutOwner<'_>,
        active_handle: &CoreHandle,
    ) -> bool {
        if self.authored() == Some(active_handle) {
            return active.component_collapse(value);
        }
        self.collapse_with_layout(value, Some((active, active_handle)))
    }

    fn collapse_with_layout(
        &self,
        value: bool,
        mut active_layout: Option<(&mut ActiveLayoutOwner<'_>, &CoreHandle)>,
    ) -> bool {
        if let Self::Authored(handle) = self {
            let action = handle
                .with_mut(|object| {
                    let dirt = object.as_component_mut()?.collapse_state(value)?;
                    if let Some(callback) = object.component_on_dirty_handler() {
                        return Some(DirtyAction::Callback(callback, dirt));
                    }
                    object.component_on_dirty(dirt);
                    object.as_component()?.notify_artboard();
                    Some(DirtyAction::Complete((
                        object.component_is_collapsed(),
                        object.as_component()?.collapsables_snapshot(),
                    )))
                })
                .flatten();
            let Some(action) = action else {
                return false;
            };
            let (collapsed, collapsables) = match action {
                DirtyAction::Callback(callback, dirt) => {
                    if let Some((active, active_handle)) = active_layout.as_mut() {
                        Self::on_dirty_authored_from_layout(handle, dirt, active, active_handle);
                    } else {
                        callback(handle, dirt);
                    }
                    let Some(post_dirty) = handle
                        .with(|object| {
                            object.as_component()?.notify_artboard();
                            Some((
                                object.component_is_collapsed(),
                                object.as_component()?.collapsables_snapshot(),
                            ))
                        })
                        .flatten()
                    else {
                        return true;
                    };
                    post_dirty
                }
                DirtyAction::Complete(post_dirty) => post_dirty,
            };
            for collapsable in collapsables.iter() {
                crate::source::data_bind::data_bind::DataBind::collapse_handle(
                    collapsable,
                    collapsed,
                );
            }
            if handle.is_type_of(
                crate::mechanical_port::source::generated::layout_component_base::LayoutComponentBase::TYPE_KEY,
            ) {
                if let Some((active, active_handle)) = active_layout.as_mut() {
                    crate::mechanical_port::source::layout_component::LayoutComponent::propagate_collapse_from_layout_occurrence(handle, value, active, active_handle);
                } else {
                    crate::mechanical_port::source::layout_component::LayoutComponent::propagate_collapse_occurrence(handle, value);
                }
                return true;
            }
            // Container children can synchronously call back into their host
            // (notably Path::onDirty into Shape). End the host borrow first.
            if handle.is_type_of(
                crate::mechanical_port::source::generated::scripted::scripted_transition_base::ScriptedTransitionBase::TYPE_KEY,
            ) {
                if let Some((active, active_handle)) = active_layout.as_mut() {
                    crate::mechanical_port::source::scripted::scripted_transition::ScriptedTransition::collapse_after_component_from_layout_occurrence(handle, value, active, active_handle);
                } else {
                    crate::mechanical_port::source::scripted::scripted_transition::ScriptedTransition::collapse_after_component_occurrence(handle, value);
                }
                return true;
            }
            if handle.is_type_of(
                crate::mechanical_port::source::generated::solo_base::SoloBase::TYPE_KEY,
            ) {
                let children = handle
                    .with_downcast::<crate::mechanical_port::source::solo::Solo, _>(|solo| {
                        solo.collapse_children(value)
                    })
                    .expect("live Solo");
                for (child, collapsed) in children {
                    if let Some((active, active_handle)) = active_layout.as_mut() {
                        Self::Authored(child).collapse_from_layout(
                            collapsed,
                            active,
                            active_handle,
                        );
                    } else {
                        Self::Authored(child).collapse(collapsed);
                    }
                }
            } else {
                let children = handle
                    .with(|object| {
                        object
                            .as_container_component()
                            .map(|container| container.component_children().to_vec())
                    })
                    .flatten()
                    .unwrap_or_default();
                for child in children {
                    if let Some((active, active_handle)) = active_layout.as_mut() {
                        child.collapse_from_layout(value, active, active_handle);
                    } else {
                        child.collapse(value);
                    }
                }
                let transformed = if let Some((active, active_handle)) = active_layout.as_mut() {
                    crate::mechanical_port::source::transform_component::TransformComponent::collapse_after_super_from_layout_occurrence(handle, active, active_handle)
                } else {
                    crate::mechanical_port::source::transform_component::TransformComponent::collapse_after_super_occurrence(handle)
                };
                if transformed {
                    // The Transform super call may synchronously dirty this
                    // owner again. Fetch each derived suffix after it returns.
                    if handle
                        .with(|object| object.as_shape().is_some())
                        .unwrap_or(false)
                    {
                        if let Some((active, active_handle)) = active_layout.as_mut() {
                            crate::mechanical_port::source::shapes::shape::Shape::collapse_after_super_from_layout_occurrence(handle, value, active, active_handle);
                        } else {
                            crate::mechanical_port::source::shapes::shape::Shape::collapse_after_super_occurrence(handle, value);
                        }
                    }
                    if handle
                        .with(|object| object.as_path().is_some())
                        .unwrap_or(false)
                    {
                        if let Some((active, active_handle)) = active_layout.as_mut() {
                            crate::mechanical_port::source::shapes::path::Path::collapse_after_super_from_layout_occurrence(handle, active, active_handle);
                        } else {
                            crate::mechanical_port::source::shapes::path::Path::collapse_after_super_occurrence(handle);
                        }
                    }
                } else {
                    // Preserve non-transform overrides such as SemanticData.
                    handle.with_mut(|object| object.component_collapse_after_container(value));
                }
                if handle.is_type_of(
                    crate::mechanical_port::source::generated::artboard_component_list_base::ArtboardComponentListBase::TYPE_KEY,
                ) {
                    crate::mechanical_port::source::artboard_component_list::ArtboardComponentList::collapse_after_super_occurrence(handle, value);
                } else if handle.is_type_of(
                    crate::mechanical_port::source::generated::nested_artboard_base::NestedArtboardBase::TYPE_KEY,
                ) {
                    crate::mechanical_port::source::nested_artboard::NestedArtboard::collapse_after_super_occurrence(handle, value);
                }
            }
            return true;
        }
        let Some(dirt) = self
            .with_component_mut(|component| component.collapse_state(value))
            .flatten()
        else {
            return false;
        };
        if let Some((active, active_handle)) = active_layout.as_mut() {
            self.on_dirty_helper_from_layout(dirt, active, active_handle);
        } else {
            self.on_dirty(dirt);
        }
        self.notify_artboard();
        if let Some((collapsed, collapsables)) = self.with_component(|component| {
            (component.is_collapsed(), component.collapsables_snapshot())
        }) {
            for collapsable in collapsables.iter() {
                crate::source::data_bind::data_bind::DataBind::collapse_handle(
                    collapsable,
                    collapsed,
                );
            }
        }
        true
    }
    /// Artboard::updateComponents' per-occurrence preparation. Read under a
    /// shared borrow so clean/collapsed entries need no exclusive access; clear
    /// dirt only for an actual update and release preparation before callbacks.
    pub(crate) fn update_if_dirty(&self) -> bool {
        let needs_update = |dirt: ComponentDirt| {
            dirt != ComponentDirt::NONE && !dirt.contains(ComponentDirt::COLLAPSED)
        };
        match self {
            Self::Authored(handle) => {
                let prepared = handle
                    .with(|object| {
                        let component = object.as_component().expect("dependency graph Component");
                        let dirt = component.dirt();
                        needs_update(dirt).then(|| (dirt, object.component_update_handler()))
                    })
                    .expect("live component in dependency graph");
                let Some((dirt, handler)) = prepared else {
                    return false;
                };
                handle.with_mut(|object| {
                    object
                        .as_component_mut()
                        .expect("dependency graph Component")
                        .set_dirt(ComponentDirt::NONE);
                });
                if let Some(update) = handler {
                    update(handle, dirt);
                } else {
                    handle.with_mut(|object| object.component_update(dirt));
                }
            }
            Self::PathComposer(handle) => {
                let owner = handle
                    .upgrade()
                    .expect("live component in dependency graph");
                let dirt = owner.borrow().component.dirt();
                if !needs_update(dirt) {
                    return false;
                }
                let shape = {
                    let mut owner = owner.borrow_mut();
                    owner.component.set_dirt(ComponentDirt::NONE);
                    owner.update(dirt)
                };
                if let Some((shape, measured_geometry_changed)) = shape {
                    shape.with_mut(|shape| {
                        let shape = shape.as_shape_mut().expect("PathComposer Shape");
                        if measured_geometry_changed {
                            shape.mark_bounds_dirty();
                        } else {
                            shape.mark_world_bounds_dirty();
                        }
                    });
                }
            }
            Self::TextVariationHelper(handle) => {
                let owner = handle
                    .upgrade()
                    .expect("live component in dependency graph");
                let dirt = owner.borrow().component.dirt();
                if !needs_update(dirt) {
                    return false;
                }
                let mut owner = owner.borrow_mut();
                owner.component.set_dirt(ComponentDirt::NONE);
                owner.update(dirt);
            }
        }
        true
    }

    pub fn update(&self, dirt: ComponentDirt) {
        match self {
            Self::Authored(handle) => {
                crate::mechanical_port::source::generated::core_registry::component_update_handle(
                    handle, dirt,
                );
            }
            Self::PathComposer(handle) => {
                let shape = handle
                    .upgrade()
                    .and_then(|owner| owner.borrow_mut().update(dirt));
                if let Some((shape, measured_geometry_changed)) = shape {
                    shape.with_mut(|shape| {
                        let shape = shape.as_shape_mut().expect("PathComposer Shape");
                        if measured_geometry_changed {
                            shape.mark_bounds_dirty();
                        } else {
                            shape.mark_world_bounds_dirty();
                        }
                    });
                }
            }
            Self::TextVariationHelper(handle) => {
                if let Some(owner) = handle.upgrade() {
                    owner.borrow_mut().update(dirt);
                }
            }
        }
    }
}

// C++ Artboard scheduling uses its inherited Component::m_Dirt. Only that
// root needs released access while callbacks borrow the Artboard's geometry;
// ordinary Components keep their dirt inline.
enum ComponentDirtStorage {
    Inline(ComponentDirt),
    Artboard(RuntimeArtboardDirtyHandle),
}

// A separate Background can read TextStylePaint's actual parent while a
// renderer callback holds that style. Other Components keep the field inline.
enum ComponentParentStorage {
    Inline(Option<CoreHandle>),
    TextStylePaint(Rc<RefCell<Option<CoreHandle>>>),
}

impl ComponentParentStorage {
    fn set(&mut self, value: Option<CoreHandle>) {
        match self {
            Self::Inline(parent) => *parent = value,
            Self::TextStylePaint(parent) => *parent.borrow_mut() = value,
        }
    }

    fn get(&self) -> Option<CoreHandle> {
        match self {
            Self::Inline(parent) => parent.clone(),
            Self::TextStylePaint(parent) => parent.borrow().clone(),
        }
    }

    fn with<R>(&self, use_parent: impl FnOnce(&CoreHandle) -> R) -> Option<R> {
        match self {
            Self::Inline(parent) => parent.as_ref().map(use_parent),
            Self::TextStylePaint(parent) => {
                // Release the field borrow before invoking another owner.
                let parent = parent.borrow().clone();
                parent.as_ref().map(use_parent)
            }
        }
    }
}

pub struct Component {
    pub base: ComponentBase,
    dependency_helper: DependencyHelper<Component>,
    parent: ComponentParentStorage,
    graph_order: u32,
    artboard: Option<CoreHandle>,
    collapsables: LazyVector<CoreHandle>,
    dirt: ComponentDirtStorage,
    runtime_occurrence: Option<ComponentOccurrenceHandle>,
}

impl Default for Component {
    fn default() -> Self {
        Self {
            base: ComponentBase::default(),
            dependency_helper: DependencyHelper::default(),
            parent: ComponentParentStorage::Inline(None),
            // No dependency order can assign the unsorted sentinel.
            graph_order: u32::MAX,
            artboard: None,
            collapsables: LazyVector::default(),
            dirt: ComponentDirtStorage::Inline(ComponentDirt::FILTHY),
            runtime_occurrence: None,
        }
    }
}

impl ComponentBaseCallbacks for Component {
    fn notify_property_changed(&mut self, property_key: u16) {
        self.base.base.notify_property_changed(property_key);
    }
}

impl Component {
    pub(crate) fn text_style_parent_weak(&mut self) -> Weak<RefCell<Option<CoreHandle>>> {
        if let ComponentParentStorage::Inline(parent) = &mut self.parent {
            self.parent =
                ComponentParentStorage::TextStylePaint(Rc::new(RefCell::new(parent.take())));
        }
        let ComponentParentStorage::TextStylePaint(parent) = &self.parent else {
            unreachable!()
        };
        Rc::downgrade(parent)
    }

    pub(crate) fn bind_artboard_dirt(&mut self, dirty: RuntimeArtboardDirtyHandle) {
        // Alias the root's existing field, including when a temporary generated
        // base stands in for it during copy/deserialize. Never replace its bits.
        self.dirt = ComponentDirtStorage::Artboard(dirty);
    }

    pub(crate) fn bind_runtime_occurrence(&mut self, occurrence: ComponentOccurrenceHandle) {
        self.runtime_occurrence = Some(occurrence);
    }
    pub fn occurrence_handle(&self) -> Option<ComponentOccurrenceHandle> {
        self.runtime_occurrence.clone().or_else(|| {
            self.base
                .base
                .handle()
                .map(ComponentOccurrenceHandle::Authored)
        })
    }
    pub(crate) fn on_added_dirty_runtime(&mut self, context: &mut dyn CoreContext) -> StatusCode {
        self.artboard = context.resolve_handle(0);
        self.parent.set(context.resolve(self.base.parent_id()));
        let Some(occurrence) = self.runtime_occurrence.clone() else {
            return StatusCode::MissingObject;
        };
        self.parent
            .with(|parent| {
                parent
                    .with_mut(|parent| {
                        parent
                            .as_container_component_mut()
                            .map(|parent| parent.add_runtime_child(occurrence))
                    })
                    .flatten()
            })
            .flatten()
            .map_or(StatusCode::MissingObject, |_| StatusCode::Ok)
    }
    pub fn dependency_root(&self) -> Option<CoreHandle> {
        self.artboard.clone()
    }

    pub fn artboard_handle(&self) -> Option<CoreHandle> {
        self.artboard.clone()
    }

    pub(crate) fn notify_artboard(&self) {
        if let Some(dirty) = self
            .artboard
            .as_ref()
            .and_then(CoreHandle::artboard_dirty_handle)
        {
            dirty.on_component_dirty_at(self.graph_order());
        }
    }

    fn notify_and_snapshot_dependents(&self, recurse: bool) -> DependencySnapshot {
        self.notify_artboard();
        if recurse {
            self.dependents_snapshot()
        } else {
            DependencySnapshot::default()
        }
    }

    pub fn with_artboard<R>(&self, use_artboard: impl FnOnce(&Artboard) -> R) -> Option<R> {
        self.artboard.as_ref()?.with_artboard(use_artboard)
    }

    pub fn with_artboard_mut<R>(&self, use_artboard: impl FnOnce(&mut Artboard) -> R) -> Option<R> {
        self.artboard
            .as_ref()?
            .with_downcast_mut::<Artboard, _>(use_artboard)
    }

    pub fn parent_handle(&self) -> Option<CoreHandle> {
        self.parent.get()
    }

    pub fn with_parent<R>(&self, use_parent: impl FnOnce(&ContainerComponent) -> R) -> Option<R> {
        self.parent.with(|parent| {
            parent.with(|parent| parent.as_container_component().map(use_parent))
        })??
    }

    pub fn with_parent_mut<R>(
        &self,
        use_parent: impl FnOnce(&mut ContainerComponent) -> R,
    ) -> Option<R> {
        self.parent.with(|parent| {
            parent.with_mut(|parent| parent.as_container_component_mut().map(use_parent))
        })??
    }

    pub fn validate(&mut self, context: &mut dyn CoreContext) -> bool {
        context
            .resolve(self.base.parent_id())
            .is_some_and(|object| object.is_type_of(ContainerComponentBase::TYPE_KEY))
    }

    /// Prepare the C++-legal case where a ContainerComponent parents itself.
    /// The concrete CoreObject lifecycle wrapper uses the returned occurrence
    /// to call the virtual ContainerComponent::addChild implementation without
    /// reborrowing this same arena slot.
    pub(crate) fn prepare_self_parent_on_added_dirty(
        &mut self,
        context: &mut dyn CoreContext,
    ) -> Option<CoreHandle> {
        let this = self.base.base.handle()?;
        let artboard = context.resolve_handle(0);
        if artboard.as_ref() == Some(&this) {
            return None;
        }
        let parent = context
            .resolve(self.base.parent_id())
            .filter(|object| object.is_type_of(ContainerComponentBase::TYPE_KEY));
        if parent.as_ref() != Some(&this) {
            // Preserve the ordinary source order. The concrete lifecycle
            // method will resolve Component::m_Artboard/m_Parent when it calls
            // Super::onAddedDirty.
            return None;
        }
        self.artboard = artboard;
        self.parent.set(parent);
        Some(this)
    }

    pub fn on_added_dirty(&mut self, context: &mut dyn CoreContext) -> StatusCode {
        let Some(this) = self.base.base.handle() else {
            return StatusCode::MissingObject;
        };
        self.artboard = context.resolve_handle(0);
        if self.artboard.as_ref() == Some(&this) {
            return StatusCode::Ok;
        }
        self.parent.set(
            context
                .resolve(self.base.parent_id())
                .filter(|object| object.is_type_of(ContainerComponentBase::TYPE_KEY)),
        );
        self.parent
            .with(|parent| {
                // The CoreObject lifecycle wrapper has already performed this virtual
                // self-add through its concrete owner. Re-entering the occurrence by
                // handle would violate RefCell's exclusive borrow and does not model a
                // second C++ callback.
                if parent == &this || Self::add_child_to_parent(parent, this) {
                    StatusCode::Ok
                } else {
                    StatusCode::MissingObject
                }
            })
            .unwrap_or(StatusCode::MissingObject)
    }

    pub(crate) fn add_child_to_parent(parent: &CoreHandle, child: CoreHandle) -> bool {
        parent
            .with_mut(|parent| {
                if let Some(range) = parent.as_any_mut().downcast_mut::<TextModifierRange>() {
                    range.add_child(child);
                    true
                } else if let Some(shape) = parent.as_shape_mut() {
                    shape.add_child(child);
                    true
                } else if let Some(parent) = parent.as_container_component_mut() {
                    parent.add_child(child);
                    true
                } else {
                    false
                }
            })
            .unwrap_or(false)
    }

    pub fn add_collapsable(&mut self, collapsable: CoreHandle) {
        CoreCapabilities::component_add_collapsable(self, collapsable);
    }

    pub(crate) fn register_collapsable(&mut self, collapsable: CoreHandle) -> bool {
        let size_before = self.collapsables.size();
        self.collapsables.push_unique(collapsable);
        self.collapsables.size() != size_before
    }

    pub fn build_dependencies(&mut self) {}

    pub fn on_dirty(&mut self, _dirt: ComponentDirt) {}

    pub fn update(&mut self, _value: ComponentDirt) {}

    pub fn graph_order(&self) -> u32 {
        self.graph_order
    }

    pub fn set_graph_order(&mut self, value: u32) {
        self.graph_order = value;
        if let Some(owner) = self.base.base.handle() {
            owner.set_component_graph_order(value);
        }
    }

    pub fn dirt(&self) -> ComponentDirt {
        match &self.dirt {
            ComponentDirtStorage::Inline(dirt) => *dirt,
            ComponentDirtStorage::Artboard(dirty) => Self::artboard_dirt(dirty),
        }
    }

    // Ordinary Components own inline bits. Keep the runtime Artboard alias
    // read separate so that their hot getter need not compute both addresses.
    #[inline(never)]
    fn artboard_dirt(dirty: &RuntimeArtboardDirtyHandle) -> ComponentDirt {
        dirty.component_dirt()
    }

    pub fn set_dirt(&mut self, value: ComponentDirt) {
        match &mut self.dirt {
            ComponentDirtStorage::Inline(dirt) => *dirt = value,
            ComponentDirtStorage::Artboard(dirty) => dirty.set_component_dirt(value),
        }
    }

    /// Mutate only retained dirt state. The central virtual action sequences
    /// callbacks, Artboard notification, and recursion after this borrow ends.
    pub fn add_dirt_state(&mut self, value: ComponentDirt) -> Option<ComponentDirt> {
        let dirt = self.dirt();
        if dirt.contains(value) {
            return None;
        }
        let dirt = dirt | value;
        self.set_dirt(dirt);
        Some(dirt)
    }

    /// Mutate only the retained collapsed bit. Most-derived propagation is
    /// sequenced by the central virtual action after this borrow ends.
    pub fn collapse_state(&mut self, value: bool) -> Option<ComponentDirt> {
        if self.is_collapsed() == value {
            return None;
        }
        let dirt = if value {
            self.dirt() | ComponentDirt::COLLAPSED
        } else {
            self.dirt() & !ComponentDirt::COLLAPSED
        };
        self.set_dirt(dirt);
        Some(dirt)
    }

    pub fn collapsables_snapshot(&self) -> LazyVectorSnapshot<CoreHandle> {
        self.collapsables.snapshot()
    }

    pub fn dependents_snapshot(&self) -> DependencySnapshot {
        self.dependency_helper.dependents_snapshot()
    }

    pub fn add_dirt(&mut self, value: ComponentDirt, recurse: bool) -> bool {
        CoreCapabilities::component_add_dirt(self, value, recurse)
    }

    pub fn has_dirt(&self, flag: ComponentDirt) -> bool {
        self.dirt().contains(flag)
    }

    pub fn has_dirt_in(value: ComponentDirt, flag: ComponentDirt) -> bool {
        (value & flag) != ComponentDirt::NONE
    }

    pub fn claims_artboard_slot(&self, _import_stack: &mut ImportStack) -> bool {
        true
    }

    pub fn import(&mut self, import_stack: &mut ImportStack) -> StatusCode {
        let Some(this) = self.base.base.handle() else {
            return StatusCode::MissingObject;
        };
        // Artboard::import performs the root's addObject(this) against the
        // already-borrowed Artboard. Query the retained occurrence's immutable
        // type here, not the abstract Core base or a second owner borrow.
        if this.core_type() == Some(ArtboardBase::TYPE_KEY) {
            return self.base.base.import(import_stack);
        }

        let Some(_artboard_importer) =
            import_stack.latest::<ArtboardImporter>(ArtboardBase::TYPE_KEY)
        else {
            return StatusCode::MissingObject;
        };
        self.base.base.import(import_stack)
    }

    pub fn collapse(&mut self, value: bool) -> bool {
        CoreCapabilities::component_collapse(self, value)
    }

    pub fn is_collapsed(&self) -> bool {
        self.dirt().contains(ComponentDirt::COLLAPSED)
    }

    pub fn hit_test_point(
        &self,
        position: &Vec2D,
        skip_on_unclipped: bool,
        _is_primary_hit: bool,
    ) -> bool {
        self.parent
            .with(|parent| {
                parent
                    .with_mut(|parent| {
                        parent.component_hit_test_point(position, skip_on_unclipped, false)
                    })
                    .flatten()
            })
            .flatten()
            .unwrap_or(true)
    }

    pub fn dependents(&self) -> &[ComponentOccurrenceHandle] {
        self.dependency_helper.dependents()
    }

    pub(crate) fn update_collapsables(&mut self) {
        CoreCapabilities::component_update_collapsables(self);
    }

    pub fn add_dependent(&mut self, dependent: impl Into<ComponentOccurrenceHandle>) {
        self.dependency_helper.add_dependent(dependent.into());
    }

    pub fn remove_dependent(&mut self, dependent: &CoreHandle) {
        self.dependency_helper
            .remove_dependent(&ComponentOccurrenceHandle::Authored(dependent.clone()));
    }
}

impl DirtDependent for Component {
    fn add_dirt(&mut self, value: ComponentDirt, recurse: bool) {
        CoreCapabilities::component_add_dirt(self, value, recurse);
    }
}

impl std::ops::Deref for Component {
    type Target = ComponentBase;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl std::ops::DerefMut for Component {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}

#[cfg(test)]
mod dirt_dispatch_tests {
    use super::{Component, ComponentDirt, ComponentOccurrenceHandle};
    use crate::mechanical_port::source::{core::CoreArena, node::Node};

    #[test]
    fn update_preparation_preserves_shared_clean_and_collapsed_borrows_for_all_occurrences() {
        use crate::mechanical_port::source::{
            shapes::path_composer::RuntimePathComposerHandle,
            text::{
                text_style::TextStyle, text_variation_helper::RuntimeTextVariationHelperHandle,
            },
        };
        let arena = CoreArena::default();
        let node = arena.insert(Node::default());
        let style = arena.insert(TextStyle::default());
        let path = RuntimePathComposerHandle::new();
        let variation = RuntimeTextVariationHelperHandle::new(style);
        for occurrence in [node.into(), path.occurrence(), variation.occurrence()] {
            for dirt in [
                ComponentDirt::NONE,
                ComponentDirt::COLLAPSED | ComponentDirt::TRANSFORM,
            ] {
                occurrence.with_component_mut(|component| component.set_dirt(dirt));
                occurrence
                    .with_component(|component| {
                        assert!(!occurrence.update_if_dirty());
                        assert_eq!(component.dirt(), dirt);
                    })
                    .unwrap();
            }
            occurrence.with_component_mut(|component| component.set_dirt(ComponentDirt::TRANSFORM));
            assert!(occurrence.update_if_dirty());
            assert_eq!(
                occurrence.with_component(Component::dirt),
                Some(ComponentDirt::NONE)
            );
            assert!(!occurrence.update_if_dirty());
        }
    }

    #[test]
    fn dependency_snapshot_owns_the_original_order_across_owner_mutation() {
        for count in [0, 1, 3] {
            let arena = CoreArena::default();
            let owner = arena.insert(Node::default());
            let handles: Vec<_> = (0..count).map(|_| arena.insert(Node::default())).collect();
            owner.with_mut(|object| {
                let component = object.as_component_mut().unwrap();
                for handle in &handles {
                    component.add_dependent(handle.clone());
                }
            });
            let mut snapshot = owner
                .with(|object| object.as_component().unwrap().dependents_snapshot())
                .unwrap();
            assert_eq!(snapshot.len(), count);
            // This is the reentrant boundary: no owner borrow remains, and
            // graph edits must not change the already-owned traversal.
            owner.with_mut(|object| {
                let component = object.as_component_mut().unwrap();
                for handle in &handles {
                    component.remove_dependent(handle);
                }
                component.add_dependent(owner.clone());
            });
            for (index, handle) in handles.iter().enumerate() {
                assert!(snapshot.next().unwrap().authored() == Some(handle));
                assert_eq!(snapshot.len(), count - index - 1);
            }
            assert!(snapshot.next().is_none());
            assert!(snapshot.next().is_none());
        }
    }

    #[test]
    fn dependency_read_view_does_not_retain_runtime_owners() {
        use crate::mechanical_port::source::shapes::path_composer::RuntimePathComposerHandle;
        for count in [1, 3] {
            let mut owner = Component::default();
            let helpers: Vec<_> = (0..count)
                .map(|_| RuntimePathComposerHandle::new())
                .collect();
            for helper in &helpers {
                owner.add_dependent(helper.occurrence());
            }
            let mut snapshot = owner.dependents_snapshot();
            // Vector storage can outlive its owning Component. Its elements
            // remain weak; neither consumed nor unvisited entries own helpers.
            drop(owner);
            drop(helpers);
            for occurrence in &mut snapshot {
                let ComponentOccurrenceHandle::PathComposer(weak) = &occurrence else {
                    unreachable!()
                };
                assert!(weak.upgrade().is_none());
                assert!(!occurrence.add_dirt(ComponentDirt::PATH, true));
            }
            assert_eq!(snapshot.len(), 0);
        }
    }

    #[test]
    fn retained_dependency_views_allow_recursive_cascades_and_later_graph_edits() {
        let arena = CoreArena::default();
        let root = arena.insert(Node::default());
        let first = arena.insert(Node::default());
        let second = arena.insert(Node::default());
        let added = arena.insert(Node::default());
        for handle in [&root, &first, &second, &added] {
            handle.with_mut(|owner| {
                owner
                    .as_component_mut()
                    .unwrap()
                    .set_dirt(ComponentDirt::NONE)
            });
        }
        root.with_mut(|owner| {
            let component = owner.as_component_mut().unwrap();
            component.add_dependent(first.clone());
            component.add_dependent(second.clone());
            component.add_dependent(first.clone()); // upstream pushUnique
        });
        first.with_mut(|owner| {
            owner
                .as_component_mut()
                .unwrap()
                .add_dependent(root.clone())
        });
        assert!(
            ComponentOccurrenceHandle::Authored(root.clone()).add_dirt(ComponentDirt::PATH, true)
        );
        for handle in [&root, &first, &second] {
            assert!(
                handle
                    .with(|owner| owner.as_component().unwrap().has_dirt(ComponentDirt::PATH))
                    .unwrap()
            );
        }
        let snapshot = root
            .with(|owner| owner.as_component().unwrap().dependents_snapshot())
            .unwrap();
        assert_eq!(snapshot.len(), 2);
        root.with_mut(|owner| {
            let component = owner.as_component_mut().unwrap();
            component.remove_dependent(&second);
            component.add_dependent(added.clone());
        });
        // The retained traversal uses the old membership while still applying
        // actual recursive dirt. Its first edge re-enters the edited root.
        for dependent in snapshot {
            dependent.add_dirt(ComponentDirt::WORLD_TRANSFORM, true);
        }
        for handle in [&root, &first, &second, &added] {
            assert!(
                handle
                    .with(|owner| owner
                        .as_component()
                        .unwrap()
                        .has_dirt(ComponentDirt::WORLD_TRANSFORM))
                    .unwrap()
            );
        }
        // A view retains the captured generation, never the replacement slot.
        let stale_view = root
            .with(|owner| owner.as_component().unwrap().dependents_snapshot())
            .unwrap();
        drop(arena.remove(&added).unwrap());
        let replacement = arena.insert(Node::default());
        assert_eq!(replacement.identity_key().1, added.identity_key().1);
        replacement.with_mut(|owner| {
            owner
                .as_component_mut()
                .unwrap()
                .set_dirt(ComponentDirt::NONE)
        });
        for dependent in stale_view {
            dependent.add_dirt(ComponentDirt::RENDER_OPACITY, true);
        }
        assert_eq!(
            replacement.with(|owner| owner.as_component().unwrap().dirt()),
            Some(ComponentDirt::NONE)
        );
    }

    #[test]
    fn dirt_dispatch_checks_the_live_generation_before_using_type_metadata() {
        let arena = CoreArena::default();
        let original = arena.insert(Node::default());
        original.with_mut(|object| {
            object
                .as_component_mut()
                .unwrap()
                .set_dirt(ComponentDirt::NONE);
        });
        let occurrence = ComponentOccurrenceHandle::Authored(original.clone());
        assert!(occurrence.add_dirt(ComponentDirt::RENDER_OPACITY, false));
        assert!(!occurrence.add_dirt(ComponentDirt::RENDER_OPACITY, false));

        drop(arena.remove(&original).expect("live original"));
        let replacement = arena.insert(Node::default());
        assert_eq!(original.identity_key().1, replacement.identity_key().1);
        replacement.with_mut(|object| {
            object
                .as_component_mut()
                .unwrap()
                .set_dirt(ComponentDirt::NONE);
        });
        assert!(!occurrence.add_dirt(ComponentDirt::WORLD_TRANSFORM, true));
        assert_eq!(
            replacement.with(|object| object.as_component().unwrap().dirt()),
            Some(ComponentDirt::NONE)
        );
        drop(arena);
        assert!(
            !ComponentOccurrenceHandle::Authored(replacement)
                .add_dirt(ComponentDirt::WORLD_TRANSFORM, true)
        );
    }
}

#[cfg(test)]
#[path = "component_helper_dirt_tests.rs"]
mod helper_dirt_tests;
