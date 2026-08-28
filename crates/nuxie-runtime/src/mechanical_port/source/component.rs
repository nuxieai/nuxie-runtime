pub use crate::mechanical_port::source::component_dirt::ComponentDirt;
use crate::mechanical_port::source::{
    artboard::Artboard,
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
    math::vec2d::Vec2D,
    status_code::StatusCode,
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
                .and_then(|owner| owner.borrow().dirty_shape());
            if let Some(shape) = shape {
                shape.with_mut(|shape| shape.as_shape_mut().map(|shape| shape.path_changed()));
            }
        }
    }
    pub(crate) fn notify_artboard(&self) {
        if let Some((Some(artboard), order)) =
            self.with_component(|component| (component.artboard_handle(), component.graph_order()))
        {
            if let Some(dirty) = artboard.artboard_dirty_handle() {
                dirty.on_component_dirty_at(order);
            }
        }
    }
    pub fn add_dirt(&self, value: ComponentDirt, recurse: bool) -> bool {
        if let Self::Authored(handle) = self {
            return handle
                .with_mut(|object| object.component_add_dirt(value, recurse))
                .unwrap_or(false);
        }
        let Some(dirt) = self
            .with_component_mut(|component| component.add_dirt_state(value))
            .flatten()
        else {
            return false;
        };
        self.on_dirty(dirt);
        self.notify_artboard();
        if recurse {
            for dependent in self
                .with_component(Component::dependents_snapshot)
                .unwrap_or_default()
            {
                dependent.add_dirt(value, true);
            }
        }
        true
    }
    pub fn collapse(&self, value: bool) -> bool {
        if let Self::Authored(handle) = self {
            return handle
                .with_mut(|object| object.component_collapse(value))
                .unwrap_or(false);
        }
        let Some(dirt) = self
            .with_component_mut(|component| component.collapse_state(value))
            .flatten()
        else {
            return false;
        };
        self.on_dirty(dirt);
        self.notify_artboard();
        self.with_component_mut(Component::update_collapsables);
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
                if let Some(shape) = shape {
                    shape.with_mut(|shape| {
                        shape
                            .as_shape_mut()
                            .expect("PathComposer Shape")
                            .mark_bounds_dirty()
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

pub struct Component {
    pub base: ComponentBase,
    dependency_helper: DependencyHelper<Component>,
    parent: Option<CoreHandle>,
    graph_order: u32,
    artboard: Option<CoreHandle>,
    collapsables: Vec<CoreHandle>,
    dirt: ComponentDirt,
    runtime_occurrence: Option<ComponentOccurrenceHandle>,
}

impl Default for Component {
    fn default() -> Self {
        Self {
            base: ComponentBase::default(),
            dependency_helper: DependencyHelper::default(),
            parent: None,
            graph_order: 0,
            artboard: None,
            collapsables: Vec::new(),
            dirt: ComponentDirt::FILTHY,
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
        self.parent = context.resolve(self.base.parent_id());
        let Some(occurrence) = self.runtime_occurrence.clone() else {
            return StatusCode::MissingObject;
        };
        self.parent
            .as_ref()
            .and_then(|parent| {
                parent
                    .with_mut(|parent| {
                        parent
                            .as_container_component_mut()
                            .map(|parent| parent.add_runtime_child(occurrence))
                    })
                    .flatten()
            })
            .map_or(StatusCode::MissingObject, |_| StatusCode::Ok)
    }
    pub fn dependency_root(&self) -> Option<CoreHandle> {
        self.artboard.clone()
    }

    pub fn artboard_handle(&self) -> Option<CoreHandle> {
        self.artboard.clone()
    }

    pub fn with_artboard<R>(&self, use_artboard: impl FnOnce(&Artboard) -> R) -> Option<R> {
        self.artboard
            .as_ref()?
            .with_downcast::<Artboard, _>(use_artboard)
    }

    pub fn with_artboard_mut<R>(&self, use_artboard: impl FnOnce(&mut Artboard) -> R) -> Option<R> {
        self.artboard
            .as_ref()?
            .with_downcast_mut::<Artboard, _>(use_artboard)
    }

    pub fn parent_handle(&self) -> Option<CoreHandle> {
        self.parent.clone()
    }

    pub fn with_parent<R>(&self, use_parent: impl FnOnce(&ContainerComponent) -> R) -> Option<R> {
        self.parent
            .as_ref()?
            .with(|parent| parent.as_container_component().map(use_parent))?
    }

    pub fn with_parent_mut<R>(
        &self,
        use_parent: impl FnOnce(&mut ContainerComponent) -> R,
    ) -> Option<R> {
        self.parent
            .as_ref()?
            .with_mut(|parent| parent.as_container_component_mut().map(use_parent))?
    }

    pub fn validate(&mut self, context: &mut dyn CoreContext) -> bool {
        context
            .resolve(self.base.parent_id())
            .is_some_and(|object| object.is_type_of(ContainerComponentBase::TYPE_KEY))
    }

    pub fn on_added_dirty(&mut self, context: &mut dyn CoreContext) -> StatusCode {
        let Some(this) = self.base.base.handle() else {
            return StatusCode::MissingObject;
        };
        self.artboard = context.resolve_handle(0);
        if self.artboard.as_ref() == Some(&this) {
            return StatusCode::Ok;
        }
        self.parent = context
            .resolve(self.base.parent_id())
            .filter(|object| object.is_type_of(ContainerComponentBase::TYPE_KEY));
        let Some(parent) = self.parent.as_ref() else {
            return StatusCode::MissingObject;
        };
        let added = parent
            .with_mut(|parent| {
                parent
                    .as_container_component_mut()
                    .map(|parent| parent.add_child(this))
            })
            .flatten()
            .is_some();
        if !added {
            return StatusCode::MissingObject;
        }
        StatusCode::Ok
    }

    pub fn add_collapsable(&mut self, collapsable: CoreHandle) {
        if !self.collapsables.contains(&collapsable) {
            self.collapsables.push(collapsable.clone());
            collapsable.with_mut(|collapsable| {
                if let Some(collapsable) = collapsable.as_data_bind_mut() {
                    collapsable.collapse(self.is_collapsed());
                }
            });
        }
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
        self.dirt
    }

    pub fn set_dirt(&mut self, value: ComponentDirt) {
        self.dirt = value;
    }

    /// Mutate only retained dirt state. The central virtual action sequences
    /// callbacks, Artboard notification, and recursion after this borrow ends.
    pub fn add_dirt_state(&mut self, value: ComponentDirt) -> Option<ComponentDirt> {
        if self.dirt.contains(value) {
            return None;
        }
        self.dirt |= value;
        Some(self.dirt)
    }

    /// Mutate only the retained collapsed bit. Most-derived propagation is
    /// sequenced by the central virtual action after this borrow ends.
    pub fn collapse_state(&mut self, value: bool) -> Option<ComponentDirt> {
        if self.is_collapsed() == value {
            return None;
        }
        if value {
            self.dirt |= ComponentDirt::COLLAPSED;
        } else {
            self.dirt &= !ComponentDirt::COLLAPSED;
        }
        Some(self.dirt)
    }

    pub fn collapsables_snapshot(&self) -> Vec<CoreHandle> {
        self.collapsables.clone()
    }

    pub fn dependents_snapshot(&self) -> Vec<ComponentOccurrenceHandle> {
        self.dependency_helper.dependents().to_vec()
    }

    pub fn add_dirt(&mut self, value: ComponentDirt, recurse: bool) -> bool {
        CoreCapabilities::component_add_dirt(self, value, recurse)
    }

    pub fn has_dirt(&self, flag: ComponentDirt) -> bool {
        self.dirt.contains(flag)
    }

    pub fn has_dirt_in(value: ComponentDirt, flag: ComponentDirt) -> bool {
        (value & flag) != ComponentDirt::NONE
    }

    pub fn import(&mut self, import_stack: &mut ImportStack) -> StatusCode {
        if self.base.base.is_type_of(ArtboardBase::TYPE_KEY) {
            return self.base.base.import(import_stack);
        }
        let Some(this) = self.base.base.handle() else {
            return StatusCode::MissingObject;
        };

        let Some(artboard_importer) =
            import_stack.latest::<ArtboardImporter>(ArtboardBase::TYPE_KEY)
        else {
            return StatusCode::MissingObject;
        };
        artboard_importer.add_component(Some(this));
        self.base.base.import(import_stack)
    }

    pub fn collapse(&mut self, value: bool) -> bool {
        CoreCapabilities::component_collapse(self, value)
    }

    pub fn is_collapsed(&self) -> bool {
        self.dirt.contains(ComponentDirt::COLLAPSED)
    }

    pub fn hit_test_point(
        &self,
        position: &Vec2D,
        skip_on_unclipped: bool,
        _is_primary_hit: bool,
    ) -> bool {
        self.parent
            .as_ref()
            .and_then(|parent| {
                parent
                    .with_mut(|parent| {
                        parent.component_hit_test_point(position, skip_on_unclipped, false)
                    })
                    .flatten()
            })
            .unwrap_or(true)
    }

    pub fn dependents(&self) -> &[ComponentOccurrenceHandle] {
        self.dependency_helper.dependents()
    }

    pub(crate) fn update_collapsables(&mut self) {
        let collapsed = self.is_collapsed();
        for collapsable in self.collapsables.iter().cloned() {
            collapsable.with_mut(|collapsable| {
                if let Some(collapsable) = collapsable.as_data_bind_mut() {
                    collapsable.collapse(collapsed);
                }
            });
        }
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
