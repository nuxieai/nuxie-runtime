use std::{
    any::Any,
    cell::{Cell, RefCell},
    hash::{Hash, Hasher},
    rc::{Rc, Weak},
};

use crate::mechanical_port::source::generated::core_registry::CoreRegistryObject;
use crate::mechanical_port::source::{
    component_dirt::ComponentDirt, core::binary_reader::BinaryReader, core_context::CoreContext,
    data_bind::data_bind::DataBind, importers::import_stack::ImportStack, status_code::StatusCode,
};

pub mod binary_data_reader;
pub mod binary_reader;
pub mod binary_stream;
pub mod binary_writer;
pub mod field_types;
pub mod type_conversions;
pub mod vector_binary_stream;
pub mod vector_binary_writer;

pub type CoreTypeKey = u16;

/// Dynamic behavior retained by one arena-owned Rive object occurrence.
///
/// Concrete owners implement this together with `CoreRegistryObject`. The
/// arena is the owner; cross-object references retain only `CoreHandle`, so a
/// graph cycle cannot keep an artboard occurrence alive.
pub trait CoreObject: CoreRegistryObject + Any {
    fn core(&self) -> &Core;
    fn core_mut(&mut self) -> &mut Core;
    fn core_type(&self) -> CoreTypeKey;
    fn is_type_of(&self, type_key: CoreTypeKey) -> bool;
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

struct CoreArenaSlot {
    generation: Cell<u64>,
    occupied: Cell<bool>,
    core_type: Cell<CoreTypeKey>,
    component_graph_order: Cell<Option<u32>>,
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
            generation: Cell::new(0),
            occupied: Cell::new(false),
            core_type: Cell::new(0),
            component_graph_order: Cell::new(None),
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
#[derive(Clone, Default)]
pub struct CoreArena {
    inner: Rc<RefCell<CoreArenaInner>>,
}

impl CoreArena {
    /// An instance's root is the same typed Artboard owned by its runtime
    /// handle. The arena retains only a weak link, while that Artboard retains
    /// the graph arena; registering its root therefore creates no owner cycle.
    pub(crate) fn insert_runtime_artboard(
        &self,
        artboard: Weak<RefCell<crate::mechanical_port::source::artboard::ArtboardInstance>>,
    ) -> CoreHandle {
        let mut inner = self.inner.borrow_mut();
        let index = inner.slots.len();
        let slot = Rc::new(CoreArenaSlot::vacant());
        slot.core_type
            .set(crate::mechanical_port::source::generated::artboard_base::ArtboardBase::TYPE_KEY);
        slot.occupied.set(true);
        if let Some(root) = artboard.upgrade() {
            let root = root.borrow();
            slot.component_graph_order.set(Some(0));
            *slot.artboard_dirty.borrow_mut() = Some(root.base.dirty_handle());
        }
        *slot.runtime_artboard.borrow_mut() = Some(artboard);
        inner.slots.push(slot);
        CoreHandle {
            arena: Rc::downgrade(&self.inner),
            index,
            generation: 0,
        }
    }
    pub fn insert<T: CoreObject>(&self, value: T) -> CoreHandle {
        self.insert_boxed(Box::new(value))
    }

    pub fn insert_boxed(&self, mut value: Box<dyn CoreObject>) -> CoreHandle {
        let (index, slot) = {
            let mut inner = self.inner.borrow_mut();
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
            arena: Rc::downgrade(&self.inner),
            index,
            generation,
        };
        slot.core_type.set(value.core_type());
        slot.component_graph_order.set(
            value
                .as_component()
                .map(|component| component.graph_order()),
        );
        *slot.artboard_dirty.borrow_mut() =
            value.as_artboard().map(|artboard| artboard.dirty_handle());
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
        let slot = {
            let inner = self.inner.borrow();
            Rc::clone(inner.slots.get(handle.index)?)
        };
        if slot.generation.get() != handle.generation {
            return None;
        }
        let value = slot.object.borrow_mut().take()?;
        slot.data_bind_container.borrow_mut().take();
        slot.artboard_dirty.borrow_mut().take();
        slot.component_graph_order.set(None);
        slot.occupied.set(false);
        slot.generation.set(slot.generation.get().wrapping_add(1));
        self.inner.borrow_mut().free.push(handle.index);
        Some(value)
    }

    pub fn len(&self) -> usize {
        self.inner
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
    index: usize,
    generation: u64,
}

impl CoreHandle {
    pub fn identity_key(&self) -> (usize, usize, u64) {
        (self.arena.as_ptr() as usize, self.index, self.generation)
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
    fn belongs_to(&self, arena: &CoreArena) -> bool {
        Weak::ptr_eq(&self.arena, &Rc::downgrade(&arena.inner))
    }

    fn slot(&self) -> Option<Rc<CoreArenaSlot>> {
        let arena = self.arena.upgrade()?;
        let inner = arena.borrow();
        let slot = Rc::clone(inner.slots.get(self.index)?);
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
        self.with(|object| object.is_type_of(type_key))
            .unwrap_or(false)
    }

    pub fn core_type(&self) -> Option<CoreTypeKey> {
        let slot = self.slot()?;
        slot.occupied.get().then(|| slot.core_type.get())
    }

    pub fn with<R>(&self, f: impl FnOnce(&dyn CoreObject) -> R) -> Option<R> {
        let slot = self.slot()?;
        if let Some(root) = slot.runtime_artboard.borrow().as_ref() {
            let root = root.upgrade()?;
            return Some(f(&root.borrow().base));
        }
        let object = slot.object.borrow();
        let object = object.as_deref()?;
        Some(f(object))
    }

    pub fn with_mut<R>(&self, f: impl FnOnce(&mut dyn CoreObject) -> R) -> Option<R> {
        let slot = self.slot()?;
        if let Some(root) = slot.runtime_artboard.borrow().as_ref() {
            let root = root.upgrade()?;
            return Some(f(&mut root.borrow_mut().base));
        }
        let mut object = slot.object.borrow_mut();
        let object = object.as_deref_mut()?;
        Some(f(object))
    }

    pub fn with_downcast<T: Any, R>(&self, f: impl FnOnce(&T) -> R) -> Option<R> {
        self.with(|object| object.as_any().downcast_ref::<T>().map(f))?
    }

    pub fn with_downcast_mut<T: Any, R>(&self, f: impl FnOnce(&mut T) -> R) -> Option<R> {
        self.with_mut(|object| object.as_any_mut().downcast_mut::<T>().map(f))?
    }

    pub fn clone_occurrence(&self) -> Option<CoreHandle> {
        let (clone, complete) =
            self.with(|source| (source.clone_boxed(), source.clone_completion_handler()))?;
        let clone = clone?;
        let arena = CoreArena {
            inner: self.arena.upgrade()?,
        };
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
        let arena = CoreArena {
            inner: self.arena.upgrade()?,
        };
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
        CoreArena { inner }.remove(self).is_some()
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
    observers: Vec<CoreHandle>,
}

impl Default for Core {
    fn default() -> Self {
        Self {
            handle: None,
            observers: Vec::new(),
        }
    }
}

impl Clone for Core {
    fn clone(&self) -> Self {
        Self::default()
    }
}

impl Core {
    pub const EMPTY_ID: u32 = u32::MAX;
    pub const INVALID_PROPERTY_KEY: i32 = 0;

    pub fn handle(&self) -> Option<CoreHandle> {
        self.handle.clone()
    }

    pub fn set_handle(&mut self, handle: CoreHandle) {
        self.handle = Some(handle);
    }

    pub fn core_type(&self) -> u16 {
        panic!("abstract Core::core_type");
    }

    pub fn is_type_of(&self, _type_key: u16) -> bool {
        panic!("abstract Core::is_type_of");
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
        // Clone the weak occurrence identities so a callback may detach itself
        // without invalidating traversal. New observers are inserted at the
        // front, matching the pinned linked-list callback order.
        let observers = self.observers.clone();
        for observer in observers {
            observer.with_downcast_mut::<DataBind, _>(|observer| {
                if observer.property_key() == u32::from(property_key) {
                    observer.add_dirt(u32::from(ComponentDirt::BINDINGS_TARGET.0), false);
                }
            });
        }
    }

    pub fn add_property_observer(&mut self, observer: CoreHandle) {
        assert!(
            !self.observers.contains(&observer),
            "DataBind already subscribed"
        );
        self.observers.insert(0, observer);
    }

    pub fn remove_property_observer(&mut self, observer: &CoreHandle) {
        if let Some(index) = self
            .observers
            .iter()
            .position(|candidate| candidate == observer)
        {
            self.observers.remove(index);
        }
    }
}

impl Drop for Core {
    fn drop(&mut self) {
        for observer in std::mem::take(&mut self.observers) {
            observer.with_downcast_mut::<DataBind, _>(DataBind::on_target_destroyed);
        }
    }
}
