use std::{
    any::Any,
    cell::{Cell, RefCell},
    hash::{Hash, Hasher},
    rc::{Rc, Weak},
};

use crate::mechanical_port::source::generated::core_registry::CoreRegistryObject;
use crate::mechanical_port::source::{
    component_dirt::ComponentDirt, core::binary_reader::BinaryReader, core_context::CoreContext,
    data_bind::data_bind::DataBind, importers::import_stack::ImportStack, math::mat2d::Mat2D,
    status_code::StatusCode,
};

pub mod binary_data_reader;
pub mod binary_reader;
pub mod binary_stream;
pub mod binary_writer;
pub mod field_types;
pub mod id;
pub mod type_conversions;
pub mod vector_binary_stream;
pub mod vector_binary_writer;

pub type CoreTypeKey = u16;

/// The generated C++ `T::typeKey` used by `Core::is<T>()`.
///
/// This belongs to source types, including abstract base owners, rather than
/// object instances. Type queries can therefore use an occurrence's cached
/// subtype predicate without constructing or borrowing an owner.
pub trait CoreType {
    const TYPE_KEY: CoreTypeKey;
}

/// Dynamic behavior retained by one arena-owned Rive object occurrence.
///
/// Concrete owners implement this together with `CoreRegistryObject`. The
/// arena is the owner; cross-object references retain only `CoreHandle`, so a
/// graph cycle cannot keep an artboard occurrence alive.
pub trait CoreObject: CoreRegistryObject + Any {
    fn painted_world_bounds(
        &mut self,
        out: &mut crate::mechanical_port::source::math::aabb::Aabb,
    ) -> crate::mechanical_port::source::drawable::BoundsFidelity {
        if let Some(owner) = self
            .as_registry_any_mut()
            .downcast_mut::<crate::mechanical_port::source::shapes::shape::Shape>()
        {
            return owner.painted_world_bounds(out);
        }
        if let Some(owner) = self
            .as_registry_any_mut()
            .downcast_mut::<crate::mechanical_port::source::shapes::image::Image>()
        {
            return owner.painted_world_bounds(out);
        }
        if let Some(owner) = self
            .as_registry_any_mut()
            .downcast_mut::<crate::mechanical_port::source::text::text::Text>()
        {
            return owner.painted_world_bounds(out);
        }
        if let Some(owner) =
            self.as_registry_any_mut()
                .downcast_mut::<crate::mechanical_port::source::text::text_input::TextInput>()
        {
            return owner.painted_world_bounds(out);
        }
        if let Some(owner)=self.as_registry_any_mut().downcast_mut::<crate::mechanical_port::source::foreground_layout_drawable::ForegroundLayoutDrawable>() { return owner.painted_world_bounds(out); }
        if let Some(owner) =
            self.as_registry_any_mut()
                .downcast_mut::<crate::mechanical_port::source::nested_artboard::NestedArtboard>()
        {
            return owner.painted_world_bounds(out);
        }
        // Artboard inherits LayoutComponent's virtual bounds implementation.
        if let Some(owner) = self.as_layout_component_mut() {
            return owner.painted_world_bounds(out);
        }
        crate::mechanical_port::source::drawable::BoundsFidelity::None
    }
    fn core(&self) -> &Core;
    fn core_mut(&mut self) -> &mut Core;
    fn core_type(&self) -> CoreTypeKey;
    fn is_type_of(&self, type_key: CoreTypeKey) -> bool;
    /// The generated type test is immutable vtable metadata. Retaining its
    /// function permits a base method to query its dynamic type while the
    /// concrete owner is already borrowed for that same method invocation.
    fn type_predicate(&self) -> fn(CoreTypeKey) -> bool;
    fn deserialize(&mut self, property_key: u16, reader: &mut BinaryReader<'_>) -> bool;
    fn clone_boxed(&self) -> Option<Box<dyn CoreObject>> {
        None
    }
    fn validate(&mut self, context: &mut dyn CoreContext) -> bool {
        crate::mechanical_port::source::generated::core_registry::CoreCapabilities::lifecycle_validate(
            self,
            context,
        )
        .unwrap_or_else(|| self.core_mut().validate(context))
    }
    fn on_added_dirty(&mut self, context: &mut dyn CoreContext) -> StatusCode {
        // C++ permits a malformed-but-valid ContainerComponent to name itself
        // as its parent. Component::onAddedDirty installs that self edge before
        // the derived lifecycle method continues. Do the same through the
        // already-borrowed concrete owner: following its CoreHandle here would
        // reborrow this arena slot and panic instead of preserving the C++
        // object graph.
        if let Some(this) = self
            .as_component_mut()
            .and_then(|component| component.prepare_self_parent_on_added_dirty(context))
        {
            if let Some(range) = self
                .as_registry_any_mut()
                .downcast_mut::<crate::mechanical_port::source::text::text_modifier_range::TextModifierRange>()
            {
                range.add_child(this);
            } else if let Some(container) = self.as_container_component_mut() {
                container.add_child(this);
            }
        }
        crate::mechanical_port::source::generated::core_registry::CoreCapabilities::lifecycle_on_added_dirty(
            self,
            context,
        )
        .unwrap_or_else(|| self.core_mut().on_added_dirty(context))
    }
    fn on_added_clean(&mut self, context: &mut dyn CoreContext) -> StatusCode {
        crate::mechanical_port::source::generated::core_registry::CoreCapabilities::lifecycle_on_added_clean(
            self,
            context,
        )
        .unwrap_or_else(|| self.core_mut().on_added_clean(context))
    }
    fn import(&mut self, import_stack: &mut ImportStack) -> StatusCode {
        crate::mechanical_port::source::generated::core_registry::CoreCapabilities::lifecycle_import(
            self,
            import_stack,
        )
        .unwrap_or_else(|| self.core_mut().import(import_stack))
    }

    fn set_core_handle(&mut self, handle: CoreHandle) {
        self.core_mut().set_handle(handle.clone());
        if let Some(artboard) = self.as_artboard_mut() {
            artboard.data_bind_container.set_owner(handle.clone());
        } else if let Some(converter) = self.as_data_converter_mut() {
            converter.data_binds.set_owner(handle.clone());
        }
        if let Some(referencer) = self.as_file_asset_referencer_mut() {
            referencer.attach(handle);
        }
    }

    fn as_any(&self) -> &dyn Any {
        self.as_registry_any()
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self.as_registry_any_mut()
    }
}

/// Weak aliases of the existing fields read by a Text background while its
/// Text or TextStylePaint ancestor is already borrowed by a renderer callback.
enum CoreTextReadProjection {
    Text(Weak<Cell<Mat2D>>),
    TextStylePaint(Weak<RefCell<Option<CoreHandle>>>),
}

struct CoreArenaSlot {
    property_observers: RefCell<Weak<PropertyObservers>>,
    generation: Cell<u64>,
    occupied: Cell<bool>,
    source_global_id: Cell<Option<u32>>,
    core_type: Cell<CoreTypeKey>,
    type_predicate: Cell<Option<fn(CoreTypeKey) -> bool>>,
    component_graph_order: Cell<Option<u32>>,
    text_read_projection: RefCell<Option<CoreTextReadProjection>>,
    artboard_dirty:
        RefCell<Option<crate::mechanical_port::source::artboard::RuntimeArtboardDirtyHandle>>,
    data_bind_container: RefCell<
        Option<crate::mechanical_port::source::data_bind::data_bind_container::DataBindContainer>,
    >,
    object: RefCell<Option<Box<dyn CoreObject>>>,
    runtime_artboard:
        RefCell<Option<Weak<RefCell<crate::mechanical_port::source::artboard::ArtboardInstance>>>>,
}

impl CoreArenaSlot {
    fn vacant() -> Self {
        Self {
            property_observers: RefCell::new(Weak::new()),
            generation: Cell::new(0),
            occupied: Cell::new(false),
            source_global_id: Cell::new(None),
            core_type: Cell::new(0),
            type_predicate: Cell::new(None),
            component_graph_order: Cell::new(None),
            text_read_projection: RefCell::new(None),
            artboard_dirty: RefCell::new(None),
            data_bind_container: RefCell::new(None),
            object: RefCell::new(None),
            runtime_artboard: RefCell::new(None),
        }
    }
}

#[derive(Default)]
struct CoreArenaInner {
    slots: Vec<Rc<CoreArenaSlot>>,
    free: Vec<usize>,
}

/// Single-threaded owner for one imported Rive object graph.
///
/// This deliberately follows the existing Rust runtime's occurrence-arena
/// boundary. Each slot has its own `RefCell`, rather than borrowing the whole
/// arena, because pinned callbacks may resolve and mutate another occurrence
/// while the current occurrence is borrowed.
#[derive(Clone)]
pub struct CoreArena {
    inner: Weak<RefCell<CoreArenaInner>>,
    owner: Option<Rc<RefCell<CoreArenaInner>>>,
}

impl Default for CoreArena {
    fn default() -> Self {
        Self::from_inner(Rc::new(RefCell::new(CoreArenaInner::default())))
    }
}

impl CoreArena {
    fn from_inner(owner: Rc<RefCell<CoreArenaInner>>) -> Self {
        Self {
            inner: Rc::downgrade(&owner),
            owner: Some(owner),
        }
    }
    /// A source Artboard resolves the File's objects but does not own the File
    /// arena that owns that same Artboard.
    pub fn weak_handle(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            owner: None,
        }
    }
    /// Runtime instances retain their definitions and cloned occurrences even
    /// when the importing File handle is released.
    pub fn strong_handle(&self) -> Self {
        Self::from_inner(self.inner.upgrade().expect("live Core graph owner"))
    }
    /// An instance's root is the same typed Artboard owned by its runtime
    /// handle. The arena retains only a weak link, while that Artboard retains
    /// the graph arena; registering its root therefore creates no owner cycle.
    pub(crate) fn insert_runtime_artboard(
        &self,
        artboard: Weak<RefCell<crate::mechanical_port::source::artboard::ArtboardInstance>>,
    ) -> CoreHandle {
        let arena = self.inner.upgrade().expect("live runtime Artboard arena");
        let mut inner = arena.borrow_mut();
        let index = inner.slots.len();
        let slot = Rc::new(CoreArenaSlot::vacant());
        slot.core_type
            .set(crate::mechanical_port::source::generated::artboard_base::ArtboardBase::TYPE_KEY);
        slot.type_predicate.set(Some(
            crate::mechanical_port::source::generated::artboard_base::ArtboardBase::is_type_of,
        ));
        slot.occupied.set(true);
        if let Some(root) = artboard.upgrade() {
            let root = root.borrow();
            slot.component_graph_order.set(Some(0));
            *slot.artboard_dirty.borrow_mut() = Some(root.base.dirty_handle());
        }
        *slot.runtime_artboard.borrow_mut() = Some(artboard);
        inner.slots.push(slot);
        CoreHandle {
            arena: self.inner.clone(),
            slot: Rc::downgrade(&inner.slots[index]),
            index,
            generation: 0,
        }
    }
    pub fn insert<T: CoreObject>(&self, value: T) -> CoreHandle {
        self.insert_boxed(Box::new(value))
    }

    pub fn insert_boxed(&self, mut value: Box<dyn CoreObject>) -> CoreHandle {
        let arena = self.inner.upgrade().expect("live Core graph insertion");
        let (index, slot) = {
            let mut inner = arena.borrow_mut();
            if let Some(index) = inner.free.pop() {
                (index, Rc::clone(&inner.slots[index]))
            } else {
                let index = inner.slots.len();
                let slot = Rc::new(CoreArenaSlot::vacant());
                inner.slots.push(Rc::clone(&slot));
                (index, slot)
            }
        };
        let generation = slot.generation.get();
        let handle = CoreHandle {
            arena: self.inner.clone(),
            slot: Rc::downgrade(&slot),
            index,
            generation,
        };
        slot.source_global_id.set(None);
        *slot.property_observers.borrow_mut() = Weak::new();
        slot.core_type.set(value.core_type());
        slot.type_predicate.set(Some(value.type_predicate()));
        slot.component_graph_order.set(
            value
                .as_component()
                .map(|component| component.graph_order()),
        );
        *slot.artboard_dirty.borrow_mut() =
            value.as_artboard().map(|artboard| artboard.dirty_handle());
        *slot.text_read_projection.borrow_mut() = if let Some(text) = value.as_text() {
            Some(CoreTextReadProjection::Text(
                text.shape_world_transform_weak(),
            ))
        } else if CoreObject::is_type_of(
            value.as_ref(),
            crate::mechanical_port::source::generated::text::text_style_paint_base::TextStylePaintBase::TYPE_KEY,
        ) {
            Some(CoreTextReadProjection::TextStylePaint(
                value
                    .as_component_mut()
                    .expect("TextStylePaint inherits Component")
                    .text_style_parent_weak(),
            ))
        } else {
            None
        };
        value.set_core_handle(handle.clone());
        let previous = slot.object.replace(Some(value));
        slot.occupied.set(true);
        debug_assert!(previous.is_none(), "CoreArena reused an occupied slot");
        handle
    }

    pub fn contains(&self, handle: &CoreHandle) -> bool {
        handle.belongs_to(self) && handle.is_alive()
    }

    pub fn remove(&self, handle: &CoreHandle) -> Option<Box<dyn CoreObject>> {
        if !handle.belongs_to(self) {
            return None;
        }
        let arena = self.inner.upgrade()?;
        let slot = {
            let inner = arena.borrow();
            Rc::clone(inner.slots.get(handle.index)?)
        };
        if slot.generation.get() != handle.generation {
            return None;
        }
        let mut value = slot.object.borrow_mut().take()?;
        // Intrusive links must be spliced while this generation is still
        // resolvable. The removed box can outlive its arena identity.
        if let Some(bind) = value.as_data_bind_mut() {
            bind.unsubscribe_target();
        }
        value.core_mut().detach_property_observers();
        *slot.property_observers.borrow_mut() = Weak::new();
        slot.data_bind_container.borrow_mut().take();
        slot.artboard_dirty.borrow_mut().take();
        slot.text_read_projection.borrow_mut().take();
        slot.component_graph_order.set(None);
        slot.source_global_id.set(None);
        slot.occupied.set(false);
        slot.generation.set(slot.generation.get().wrapping_add(1));
        arena.borrow_mut().free.push(handle.index);
        Some(value)
    }

    pub(crate) fn retire_runtime_artboard(&self, handle: &CoreHandle) {
        if !handle.belongs_to(self) {
            return;
        }
        let Some(arena) = self.inner.upgrade() else {
            return;
        };
        let Some(slot) = handle.slot() else {
            return;
        };
        if slot.runtime_artboard.borrow_mut().take().is_none() {
            return;
        }
        slot.data_bind_container.borrow_mut().take();
        slot.artboard_dirty.borrow_mut().take();
        slot.text_read_projection.borrow_mut().take();
        slot.component_graph_order.set(None);
        slot.source_global_id.set(None);
        slot.occupied.set(false);
        slot.generation.set(slot.generation.get().wrapping_add(1));
        arena.borrow_mut().free.push(handle.index);
    }

    pub fn len(&self) -> usize {
        let Some(arena) = self.inner.upgrade() else {
            return 0;
        };
        arena
            .borrow()
            .slots
            .iter()
            .filter(|slot| slot.occupied.get())
            .count()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

/// Opaque stable identity for one occurrence in a `CoreArena`.
///
/// The weak arena reference prevents object graphs from owning themselves.
/// Generation checks make a handle to a removed occurrence permanently stale,
/// even if its slot is later reused.
#[derive(Clone)]
pub struct CoreHandle {
    arena: Weak<RefCell<CoreArenaInner>>,
    slot: Weak<CoreArenaSlot>,
    index: usize,
    generation: u64,
}

impl CoreHandle {
    pub(crate) fn notify_property_changed(&self, property_key: u16) {
        if let Some(observers) = self.property_observers() {
            observers.notify(property_key);
        }
    }
    pub(crate) fn property_observers(&self) -> Option<Rc<PropertyObservers>> {
        self.slot()?.property_observers.borrow().upgrade()
    }

    pub(crate) fn ensure_property_observers(&self) -> Option<Rc<PropertyObservers>> {
        if let Some(observers) = self.property_observers() {
            return Some(observers);
        }
        self.with_mut(|owner| owner.core_mut().ensure_property_observers())
    }
    /// Stable allocation identity for source tables that hash object addresses.
    /// This does not dereference the slot; equality still checks its generation.
    pub(crate) fn slot_address(&self) -> usize {
        self.slot.as_ptr() as usize
    }
    pub fn identity_key(&self) -> (usize, usize, u64) {
        (self.arena.as_ptr() as usize, self.index, self.generation)
    }
    pub(crate) fn source_global_id(&self) -> Option<u32> {
        self.slot()?.source_global_id.get()
    }
    pub(crate) fn set_source_global_id(&self, value: u32) -> bool {
        let Some(slot) = self.slot() else {
            return false;
        };
        slot.source_global_id.set(Some(value));
        true
    }
    pub fn component_graph_order(&self) -> Option<u32> {
        self.slot()?.component_graph_order.get()
    }
    pub fn data_bind_container(
        &self,
    ) -> Option<crate::mechanical_port::source::data_bind::data_bind_container::DataBindContainer>
    {
        self.slot()?.data_bind_container.borrow().clone()
    }
    pub fn set_data_bind_container(
        &self,
        container: crate::mechanical_port::source::data_bind::data_bind_container::DataBindContainer,
    ) {
        if let Some(slot) = self.slot() {
            *slot.data_bind_container.borrow_mut() = Some(container);
        }
    }
    pub(crate) fn set_component_graph_order(&self, order: u32) {
        if let Some(slot) = self.slot() {
            slot.component_graph_order.set(Some(order));
        }
    }
    pub fn artboard_dirty_handle(
        &self,
    ) -> Option<crate::mechanical_port::source::artboard::RuntimeArtboardDirtyHandle> {
        self.slot()?.artboard_dirty.borrow().clone()
    }

    pub(crate) fn text_shape_world_transform(&self) -> Option<Mat2D> {
        let slot = self.slot()?;
        if !slot.occupied.get() {
            return None;
        }
        let projection = slot.text_read_projection.borrow();
        let CoreTextReadProjection::Text(transform) = projection.as_ref()? else {
            return None;
        };
        Some(transform.upgrade()?.get())
    }

    pub(crate) fn text_style_parent(&self) -> Option<CoreHandle> {
        let slot = self.slot()?;
        if !slot.occupied.get() {
            return None;
        }
        let projection = slot.text_read_projection.borrow();
        let CoreTextReadProjection::TextStylePaint(parent) = projection.as_ref()? else {
            return None;
        };
        let parent = parent.upgrade()?;
        let handle = parent.borrow().clone();
        handle
    }

    fn belongs_to(&self, arena: &CoreArena) -> bool {
        Weak::ptr_eq(&self.arena, &arena.inner)
    }

    fn slot(&self) -> Option<Rc<CoreArenaSlot>> {
        let slot = self.slot.upgrade()?;
        (slot.generation.get() == self.generation).then_some(slot)
    }

    pub fn is_alive(&self) -> bool {
        self.slot().is_some_and(|slot| {
            slot.occupied.get()
                && slot
                    .runtime_artboard
                    .borrow()
                    .as_ref()
                    .is_none_or(|root| root.strong_count() > 0)
        })
    }

    pub fn is_type_of(&self, type_key: CoreTypeKey) -> bool {
        let Some(slot) = self.slot() else {
            return false;
        };
        let is_alive = slot.occupied.get()
            && slot
                .runtime_artboard
                .borrow()
                .as_ref()
                .is_none_or(|root| root.strong_count() > 0);
        if !is_alive {
            return false;
        }
        slot.type_predicate
            .get()
            .is_some_and(|predicate| predicate(type_key))
    }

    /// Existing immutable registration metadata, checked for this generation
    /// and a live owner without borrowing the object's mutable contents.
    pub(crate) fn type_metadata(&self) -> Option<(CoreTypeKey, fn(CoreTypeKey) -> bool)> {
        let slot = self.slot()?;
        let is_alive = slot.occupied.get()
            && slot
                .runtime_artboard
                .borrow()
                .as_ref()
                .is_none_or(|root| root.strong_count() > 0);
        if !is_alive {
            return None;
        }
        Some((slot.core_type.get(), slot.type_predicate.get()?))
    }

    pub fn core_type(&self) -> Option<CoreTypeKey> {
        let slot = self.slot()?;
        slot.occupied.get().then(|| slot.core_type.get())
    }

    /// Retain the existing artboard occurrence represented by this arena slot.
    /// This does not instantiate or copy a definition.
    pub(crate) fn runtime_artboard_instance(
        &self,
    ) -> Option<crate::mechanical_port::source::artboard::RuntimeArtboardInstanceHandle> {
        let slot = self.slot()?;
        if !slot.occupied.get() {
            return None;
        }
        let instance = slot.runtime_artboard.borrow().as_ref()?.upgrade()?;
        Some(
            crate::mechanical_port::source::artboard::RuntimeArtboardInstanceHandle::from_retained(
                instance,
            ),
        )
    }

    pub fn with<R>(&self, f: impl FnOnce(&dyn CoreObject) -> R) -> Option<R> {
        let slot = self.slot()?;
        // Authored objects and runtime Artboard roots occupy disjoint slots.
        // An authored receiver needs only its object borrow, not a second
        // borrow of the empty root route. Release an empty object guard before
        // root callbacks, which may resolve this same arena identity again.
        {
            let object = slot.object.borrow();
            if let Some(object) = object.as_deref() {
                return Some(f(object));
            }
        }
        let runtime_artboard = slot.runtime_artboard.borrow().clone();
        if let Some(root) = runtime_artboard {
            let root = crate::mechanical_port::source::artboard::RuntimeArtboardInstanceHandle::from_retained(root.upgrade()?);
            return Some(root.with_artboard(|root| f(&root.base)));
        }
        None
    }

    pub fn with_mut<R>(&self, f: impl FnOnce(&mut dyn CoreObject) -> R) -> Option<R> {
        let slot = self.slot()?;
        // Keep the two payload cells separate: liveness/type queries remain
        // available while an authored object is mutably borrowed. As above,
        // never carry the empty object guard into a runtime-root callback.
        {
            let mut object = slot.object.borrow_mut();
            if let Some(object) = object.as_deref_mut() {
                return Some(f(object));
            }
        }
        let runtime_artboard = slot.runtime_artboard.borrow().clone();
        if let Some(root) = runtime_artboard {
            let root = crate::mechanical_port::source::artboard::RuntimeArtboardInstanceHandle::from_retained(root.upgrade()?);
            return Some(root.with_artboard_mut(|root| f(&mut root.base)));
        }
        None
    }

    pub fn with_downcast<T: Any, R>(&self, f: impl FnOnce(&T) -> R) -> Option<R> {
        self.with(|object| object.as_any().downcast_ref::<T>().map(f))?
    }

    pub fn with_downcast_mut<T: Any, R>(&self, f: impl FnOnce(&mut T) -> R) -> Option<R> {
        self.with_mut(|object| object.as_any_mut().downcast_mut::<T>().map(f))?
    }

    pub fn clone_occurrence(&self) -> Option<CoreHandle> {
        self.clone_occurrence_into(&self.retain_arena()?)
    }

    pub fn retain_arena(&self) -> Option<CoreArena> {
        self.arena.upgrade().map(CoreArena::from_inner)
    }

    pub fn clone_occurrence_into(&self, arena: &CoreArena) -> Option<CoreHandle> {
        let (clone, complete) =
            self.with(|source| (source.clone_boxed(), source.clone_completion_handler()))?;
        let clone = clone?;
        let clone = arena.insert_boxed(clone);
        if let Some(complete) = complete {
            if !complete(self, &clone) {
                arena.remove(&clone);
                return None;
            }
        }
        Some(clone)
    }

    /// Insert a newly constructed occurrence into the same graph arena.
    ///
    /// Importers use this for pinned synthetic owners such as the generic
    /// LayerState inserted for an unknown serialized state type. The returned
    /// handle has the same ownership and generation guarantees as an object
    /// deserialized directly by the registry.
    pub fn insert_sibling<T: CoreObject>(&self, value: T) -> Option<CoreHandle> {
        let arena = CoreArena::from_inner(self.arena.upgrade()?);
        Some(arena.insert(value))
    }

    /// Remove this occurrence from its owning graph arena.
    ///
    /// The generation is advanced before the removed owner is dropped, so all
    /// cloned handles become stale and a later sibling cannot reuse this
    /// identity accidentally.
    pub fn remove_occurrence(&self) -> bool {
        let Some(inner) = self.arena.upgrade() else {
            return false;
        };
        CoreArena::from_inner(inner).remove(self).is_some()
    }
}

impl PartialEq for CoreHandle {
    fn eq(&self, other: &Self) -> bool {
        Weak::ptr_eq(&self.arena, &other.arena)
            && self.index == other.index
            && self.generation == other.generation
    }
}

impl Eq for CoreHandle {}

impl Hash for CoreHandle {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.arena.as_ptr().hash(state);
        self.index.hash(state);
        self.generation.hash(state);
    }
}

impl std::fmt::Debug for CoreHandle {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("CoreHandle")
            .field("arena", &self.arena.as_ptr())
            .field("index", &self.index)
            .field("generation", &self.generation)
            .finish()
    }
}

pub struct Core {
    handle: Option<CoreHandle>,
    type_metadata: Option<(CoreTypeKey, fn(CoreTypeKey) -> bool)>,
    observers: Option<Rc<PropertyObservers>>,
}

/// A generated setter's final notification, completed by its caller after
/// releasing the occurrence borrow. This is local to one write, never queued
/// on the object. Borrowed setters finish it synchronously before returning.
#[derive(Default)]
pub struct PropertySetterCompletion {
    borrowed_setter: bool,
    before_notification: Option<(CoreHandle, fn(&CoreHandle))>,
    notification: Option<(Rc<PropertyObservers>, u16)>,
}

impl PropertySetterCompletion {
    pub(crate) fn borrowed_setter() -> Self {
        Self {
            borrowed_setter: true,
            ..Self::default()
        }
    }
    pub(crate) fn is_borrowed_setter(&self) -> bool {
        self.borrowed_setter
    }
    pub(crate) fn before_notification(&mut self, owner: CoreHandle, callback: fn(&CoreHandle)) {
        debug_assert!(self.before_notification.is_none());
        self.before_notification = Some((owner, callback));
    }
    pub(crate) fn record(&mut self, core: &Core, property_key: u16) {
        debug_assert!(self.notification.is_none());
        self.notification = core.observers.as_ref().map(|head| (head.clone(), property_key));
    }

    pub fn finish(self) {
        if let Some((owner, callback)) = self.before_notification {
            callback(&owner);
        }
        if let Some((head, property_key)) = self.notification {
            head.notify(property_key);
        }
    }
}

/// The single authoritative intrusive-list head. Separate borrowing permits an
/// observer callback to unlink from a target whose generated setter is active.
/// Arena metadata keeps only a weak alias; there is no copied membership list.
#[derive(Default)]
pub(crate) struct PropertyObservers {
    first: RefCell<Option<CoreHandle>>,
}

impl PropertyObservers {
    pub(crate) fn add(&self, observer: &mut DataBind) {
        let handle = observer.base.base.handle().expect("registered DataBind");
        let mut current = self.first.borrow().clone();
        while let Some(link) = current {
            if link == handle {
                debug_assert!(false, "DataBind already subscribed");
                return;
            }
            current = link
                .with(|owner| owner.as_data_bind().and_then(DataBind::next_observer))
                .flatten();
        }
        observer.set_next_observer(self.first.borrow().clone());
        *self.first.borrow_mut() = Some(handle);
    }

    pub(crate) fn remove(&self, observer: &mut DataBind) {
        let Some(handle) = observer.base.base.handle() else {
            return;
        };
        let mut previous: Option<CoreHandle> = None;
        let mut current = self.first.borrow().clone();
        while let Some(link) = current {
            if link == handle {
                let next = observer.next_observer();
                if let Some(previous) = previous {
                    previous.with_mut(|owner| {
                        if let Some(previous) = owner.as_data_bind_mut() {
                            previous.set_next_observer(next);
                        }
                    });
                } else {
                    *self.first.borrow_mut() = next;
                }
                observer.set_next_observer(None);
                return;
            }
            current = link
                .with(|owner| owner.as_data_bind().and_then(DataBind::next_observer))
                .flatten();
            previous = Some(link);
        }
    }

    pub(crate) fn notify(&self, property_key: u16) {
        let mut current = self.first.borrow().clone();
        while let Some(observer) = current {
            if observer
                .with(|owner| owner.as_data_bind().map(DataBind::property_key))
                .flatten()
                == Some(u32::from(property_key))
            {
                DataBind::add_dirt_handle(
                    &observer,
                    u32::from(ComponentDirt::BINDINGS_TARGET.0),
                    false,
                );
            }
            // Deliberately read AFTER addDirt: removing current clears this
            // link; removing its successor splices the live next pointer.
            current = observer
                .with(|owner| owner.as_data_bind().and_then(DataBind::next_observer))
                .flatten();
        }
    }

    fn detach_all(&self) {
        let mut current = self.first.borrow().clone();
        while let Some(observer) = current {
            current = observer
                .with_mut(|owner| {
                    let observer = owner.as_data_bind_mut()?;
                    let next = observer.next_observer();
                    observer.on_target_destroyed();
                    next
                })
                .flatten();
        }
        *self.first.borrow_mut() = None;
    }
}

impl Default for Core {
    fn default() -> Self {
        Self {
            handle: None,
            type_metadata: None,
            observers: None,
        }
    }
}

impl Clone for Core {
    fn clone(&self) -> Self {
        Self::default()
    }

    fn clone_from(&mut self, _source: &Self) {
        // C++ assignment preserves this occurrence and its observer list.
    }
}

impl Core {
    pub const EMPTY_ID: u32 = u32::MAX;
    pub const INVALID_PROPERTY_KEY: i32 = 0;

    pub fn handle(&self) -> Option<CoreHandle> {
        self.handle.clone()
    }

    pub fn set_handle(&mut self, handle: CoreHandle) {
        let slot = handle
            .slot()
            .expect("installing a concrete Core occurrence");
        self.type_metadata = Some((
            slot.core_type.get(),
            slot.type_predicate.get().expect("concrete type predicate"),
        ));
        if let Some(observers) = &self.observers {
            *slot.property_observers.borrow_mut() = Rc::downgrade(observers);
        }
        self.handle = Some(handle);
    }

    pub fn core_type(&self) -> u16 {
        self.type_metadata
            .expect("dynamic type requires a concrete Core occurrence")
            .0
    }

    pub fn is_type_of(&self, type_key: u16) -> bool {
        let (_, predicate) = self
            .type_metadata
            .expect("dynamic type requires a concrete Core occurrence");
        predicate(type_key)
    }

    pub fn deserialize(&mut self, _property_key: u16, _reader: &mut BinaryReader<'_>) -> bool {
        panic!("abstract Core::deserialize");
    }

    pub fn clone_core(&self) -> Option<Box<Core>> {
        None
    }

    pub fn validate(&mut self, _context: &mut dyn CoreContext) -> bool {
        true
    }

    pub fn on_added_dirty(&mut self, _context: &mut dyn CoreContext) -> StatusCode {
        StatusCode::Ok
    }

    pub fn on_added_clean(&mut self, _context: &mut dyn CoreContext) -> StatusCode {
        StatusCode::Ok
    }

    pub fn import(&mut self, _import_stack: &mut ImportStack) -> StatusCode {
        StatusCode::Ok
    }

    pub fn notify_property_changed(&mut self, property_key: u16) {
        if let Some(observers) = &self.observers {
            observers.notify(property_key);
        }
    }

    pub(crate) fn ensure_property_observers(&mut self) -> Rc<PropertyObservers> {
        let observers = self
            .observers
            .get_or_insert_with(|| Rc::new(PropertyObservers::default()));
        if let Some(slot) = self.handle.as_ref().and_then(CoreHandle::slot) {
            *slot.property_observers.borrow_mut() = Rc::downgrade(observers);
        }
        observers.clone()
    }

    fn detach_property_observers(&mut self) {
        if let Some(observers) = &self.observers {
            observers.detach_all();
        }
    }

    pub fn add_property_observer(&mut self, observer: &mut DataBind) {
        self.ensure_property_observers().add(observer);
    }

    pub fn remove_property_observer(&mut self, observer: &mut DataBind) {
        if let Some(observers) = &self.observers {
            observers.remove(observer);
        }
    }
}

impl Drop for Core {
    fn drop(&mut self) {
        if let Some(observers) = self.observers.take() {
            observers.detach_all();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::CoreArena;
    use crate::mechanical_port::source::{node::Node, shapes::shape::Shape};

    #[test]
    fn authored_access_keeps_live_metadata_available_during_mutable_borrow() {
        let arena = CoreArena::default();
        let node = arena.insert(Node::default());
        let type_key = node.core_type().unwrap();
        node.with_mut(|object| {
            assert_eq!(object.core_type(), type_key);
            assert!(node.is_alive());
            assert!(arena.contains(&node));
            assert!(node.is_type_of(type_key));
            assert_eq!(node.type_metadata().unwrap().0, type_key);
            assert!(node.runtime_artboard_instance().is_none());
        })
        .unwrap();
        assert_eq!(node.with(|_| node.with(|_| 7)), Some(Some(7)));
    }

    #[test]
    fn runtime_root_access_releases_the_empty_object_guard_before_callbacks() {
        use crate::mechanical_port::source::artboard::{
            ArtboardInstance, RuntimeArtboardInstanceHandle,
        };
        let runtime = RuntimeArtboardInstanceHandle::new(ArtboardInstance::default());
        let root = runtime.core_handle();
        let arena = root.retain_arena().unwrap();
        root.with(|_| {
            assert!(arena.remove(&root).is_none());
            assert!(root.with(|object| object.as_artboard().is_some()).unwrap());
        })
        .unwrap();
        root.with_mut(|object| {
            assert!(object.as_artboard_mut().is_some());
            assert!(arena.remove(&root).is_none());
            assert!(root.is_alive());
            assert!(root.type_metadata().is_some());
        })
        .unwrap();
    }

    #[test]
    fn runtime_access_retains_last_owner_until_callback_and_then_retires_identity() {
        use crate::mechanical_port::source::artboard::Artboard;
        for mutable in [false, true] {
            let definitions = CoreArena::default();
            let source = definitions.insert(Artboard::default());
            let runtime = Artboard::instance_from_handle(&source).unwrap();
            let _manager = runtime.ensure_focus_manager();
            let weak = runtime.downgrade();
            let root = runtime.core_handle();
            let arena = root.retain_arena().unwrap();
            let during_callback = || {
                drop(runtime);
                assert!(weak.upgrade().is_some());
                assert!(root.is_alive());
            };
            if mutable {
                root.with_mut(|_| during_callback()).unwrap();
            } else {
                root.with(|_| during_callback()).unwrap();
            }
            assert!(weak.upgrade().is_none());
            assert!(!root.is_alive());
            assert!(root.with(|_| ()).is_none());
            assert!(root.with_mut(|_| ()).is_none());
            let replacement = arena.insert(Node::default());
            assert_eq!(root.identity_key().1, replacement.identity_key().1);
            assert_ne!(root.identity_key().2, replacement.identity_key().2);
            assert!(root.with(|_| ()).is_none());
            assert!(replacement.with(|_| ()).is_some());
        }
    }

    #[test]
    fn draw_dispatch_metadata_matches_registered_owner_projections() {
        use crate::mechanical_port::source::generated::{
            artboard_base::ArtboardBase, core_registry::CoreRegistry,
            nested_artboard_base::NestedArtboardBase,
            scripted::scripted_drawable_base::ScriptedDrawableBase,
        };
        let arena = CoreArena::default();
        let mut checked = std::collections::BTreeSet::new();
        let owners = (0..=u16::MAX)
            .filter_map(|key| CoreRegistry::make_core_box(i32::from(key)))
            .chain(std::iter::once(
                Box::new(crate::video::Video::default()) as Box<dyn super::CoreObject>
            ));
        for owner in owners {
            let handle = arena.insert_boxed(owner);
            let (key, predicate) = handle.type_metadata().expect("live registered owner");
            let projections = handle
                .with(|owner| {
                    (
                        owner.core_type(),
                        owner.as_nested_artboard().is_some(),
                        owner.as_scripted_drawable().is_some(),
                        owner.as_artboard().is_some(),
                    )
                })
                .unwrap();
            if key == crate::mechanical_port::source::generated::shapes::shape_base::ShapeBase::TYPE_KEY {
                assert_eq!(projections, (key, false, false, false), "Shape uses ordinary virtual draw");
            }
            assert_eq!(
                (
                    key,
                    predicate(NestedArtboardBase::TYPE_KEY),
                    predicate(ScriptedDrawableBase::TYPE_KEY),
                    predicate(ArtboardBase::TYPE_KEY)
                ),
                projections,
                "draw dispatch for registered type {key}",
            );
            checked.insert(key);
            drop(arena.remove(&handle).unwrap());
            assert!(handle.type_metadata().is_none());
        }
        for key in [
            1,
            92,
            451,
            452,
            603,
            637,
            110,
            100,
            559,
            crate::mechanical_port::source::generated::shapes::shape_base::ShapeBase::TYPE_KEY,
            crate::video::Video::TYPE_KEY,
        ] {
            assert!(checked.contains(&key), "dispatch owner {key} was exercised");
        }
    }

    #[test]
    fn reused_arena_slots_do_not_inherit_source_global_ids() {
        let arena = CoreArena::default();
        let authored = arena.insert(Shape::default());
        assert!(authored.set_source_global_id(42));
        assert_eq!(authored.source_global_id(), Some(42));
        let authored_identity = authored.identity_key();

        let removed = arena.remove(&authored).expect("authored owner is live");
        assert_eq!(authored.source_global_id(), None);
        drop(removed);

        let synthetic = arena.insert(Node::default());
        let synthetic_identity = synthetic.identity_key();
        assert_eq!(synthetic_identity.0, authored_identity.0, "same arena");
        assert_eq!(synthetic_identity.1, authored_identity.1, "slot is reused");
        assert_ne!(
            synthetic_identity.2, authored_identity.2,
            "generation changes on reuse"
        );
        assert_eq!(
            synthetic.source_global_id(),
            None,
            "a runtime owner must not inherit authored source identity"
        );
    }

    #[test]
    fn cloned_runtime_occurrences_do_not_copy_source_global_ids() {
        let arena = CoreArena::default();
        let authored = arena.insert(Shape::default());
        assert!(authored.set_source_global_id(42));

        let occurrence = authored
            .clone_occurrence()
            .expect("Shape has a concrete runtime clone");
        assert_eq!(authored.source_global_id(), Some(42));
        assert_eq!(
            occurrence.source_global_id(),
            None,
            "runtime identity is distinct from authored source identity"
        );
    }

    #[test]
    fn text_matrix_projection_is_live_and_occurrence_scoped() {
        use crate::mechanical_port::source::{math::mat2d::Mat2D, text::text::Text};

        let arena = CoreArena::default();
        let text = arena.insert(Text::default());
        let matrix = Mat2D::from_scale(2.0, 3.0);
        text.with_mut(|owner| {
            owner
                .as_text_mut()
                .unwrap()
                .set_shape_world_transform(matrix);
            // This is a read of another field of the active owner, not a
            // second borrow of the entire Text object.
            assert_eq!(text.text_shape_world_transform(), Some(matrix));
        })
        .unwrap();

        let twin = text.clone_occurrence().unwrap();
        assert_eq!(twin.text_shape_world_transform(), Some(Mat2D::default()));
        twin.with_mut(|owner| {
            owner
                .as_text_mut()
                .unwrap()
                .set_shape_world_transform(Mat2D::from_scale(4.0, 5.0));
        })
        .unwrap();
        assert_eq!(text.text_shape_world_transform(), Some(matrix));

        // Keep the removed owner (and its matrix cell) alive across slot reuse.
        let removed = arena.remove(&text).unwrap();
        let replacement = arena.insert(Text::default());
        assert_eq!(text.identity_key().1, replacement.identity_key().1);
        assert_ne!(text.identity_key().2, replacement.identity_key().2);
        assert_eq!(text.text_shape_world_transform(), None);
        assert_eq!(
            replacement.text_shape_world_transform(),
            Some(Mat2D::default())
        );
        assert_eq!(removed.as_text().unwrap().shape_world_transform(), matrix);

        drop(arena);
        assert_eq!(twin.text_shape_world_transform(), None);
        assert_eq!(replacement.text_shape_world_transform(), None);
    }

    #[test]
    fn style_parent_projection_tracks_lifecycle_without_aliasing_clones() {
        use super::CoreHandle;
        use crate::mechanical_port::source::{
            artboard::Artboard,
            core_context::CoreContext,
            generated::{component_base::ComponentBase, core_registry::CoreRegistry},
            status_code::StatusCode,
            text::{text::Text, text_style_paint::TextStylePaint},
        };

        struct Context {
            arena: CoreArena,
            objects: Vec<CoreHandle>,
        }
        impl CoreContext for Context {
            fn core_arena(&self) -> &CoreArena {
                &self.arena
            }
            fn resolve_handle(&self, id: u32) -> Option<CoreHandle> {
                self.objects.get(id as usize).cloned()
            }
        }
        fn hydrate(style: &CoreHandle, context: &mut Context, parent: u32) {
            assert!(CoreRegistry::set_uint_handle(
                style,
                ComponentBase::PARENT_ID_PROPERTY_KEY.into(),
                parent,
            ));
            style
                .with_mut(|owner| {
                    assert_eq!(
                        owner.as_component_mut().unwrap().on_added_dirty(context),
                        StatusCode::Ok,
                    );
                    assert_eq!(
                        style.text_style_parent(),
                        Some(context.objects[parent as usize].clone()),
                    );
                })
                .unwrap();
        }

        let arena = CoreArena::default();
        let root = arena.insert(Artboard::default());
        let first = arena.insert(Text::default());
        let second = arena.insert(Text::default());
        let style = arena.insert(TextStylePaint::default());
        let mut context = Context {
            arena: arena.clone(),
            objects: vec![root, first.clone(), second.clone()],
        };
        hydrate(&style, &mut context, 1);
        hydrate(&style, &mut context, 2);

        let twin = style.clone_occurrence().unwrap();
        assert_eq!(twin.text_style_parent(), None);
        hydrate(&twin, &mut context, 1);
        assert_eq!(style.text_style_parent(), Some(second.clone()));

        let removed = arena.remove(&style).unwrap();
        let replacement = arena.insert(TextStylePaint::default());
        assert_eq!(style.identity_key().1, replacement.identity_key().1);
        hydrate(&replacement, &mut context, 1);
        assert_eq!(style.text_style_parent(), None);
        assert_eq!(
            removed.as_component().unwrap().parent_handle(),
            Some(second)
        );
        assert_eq!(twin.text_style_parent(), Some(first));
        drop(context);
        drop(arena);
        assert_eq!(twin.text_style_parent(), None);
        assert_eq!(replacement.text_style_parent(), None);
    }
}
