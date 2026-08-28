//! Host notification/capture policy and immutable value DTOs; runtime values live in Core owners.

use std::cell::{Cell, RefCell};
use std::collections::{BTreeMap, BTreeSet};
use std::rc::{Rc, Weak};
use std::sync::Arc;

#[path = "../assets/blob_asset.rs"]
mod blob_asset_owner;
pub use blob_asset_owner::RuntimeBlobAsset;

type RuntimeDeferredNotification = Box<dyn FnOnce()>;

thread_local! {
    static HOST_TRANSACTION_PUBLICATION: Cell<bool> = const { Cell::new(false) };
    static HOST_MUTATION_NOTIFICATIONS: RefCell<Option<Vec<RuntimeDeferredNotification>>> =
        const { RefCell::new(None) };
    static HOST_MUTATION_CALLBACK_FIREWALL_DEPTH: Cell<u32> = const { Cell::new(0) };
    static VIEW_MODEL_CHANGE_CAPTURE: RefCell<Option<RuntimeViewModelChangeCaptureState>> =
        const { RefCell::new(None) };
}

/// Host atomic batches, unlike ordinary runtime mutations, publish only on
/// commit. The flag never changes the source runtime's normal frame behavior.
pub(crate) struct RuntimeHostTransactionPublication;
impl RuntimeHostTransactionPublication {
    pub(crate) fn begin() -> Option<Self> {
        HOST_TRANSACTION_PUBLICATION.with(|active| (!active.replace(true)).then_some(Self))
    }
}
impl Drop for RuntimeHostTransactionPublication {
    fn drop(&mut self) {
        HOST_TRANSACTION_PUBLICATION.with(|active| active.set(false));
    }
}
pub(crate) fn defer_transaction_notification(notification: impl FnOnce() + 'static) -> bool {
    HOST_TRANSACTION_PUBLICATION.with(Cell::get) && defer_host_mutation_notification(notification)
}

/// Copied typed after-value for one operation-scoped view-model write.
///
/// Structural values are completed by the owning view-model graph after the
/// operation commits; scalar payloads are copied at the exact write boundary
/// so repeated writes never collapse into a final-state snapshot.
#[derive(Debug, Clone, PartialEq)]
pub enum RuntimeViewModelChangeValue {
    Number(f32),
    Boolean(bool),
    String(Arc<[u8]>),
    Color(u32),
    Enum(u64),
    Trigger(u64),
    ListIndex(u64),
    Image(u64),
    Font(u64),
    Blob(u64),
    Artboard(u64),
    List(Vec<u64>),
    ViewModel(Option<u64>),
}

#[derive(Debug, Clone)]
pub(crate) struct RuntimeViewModelCapturedChange {
    pub(crate) cell_identity: usize,
    pub(crate) value: RuntimeViewModelChangeValue,
}

#[derive(Debug)]
struct RuntimeViewModelChangeCaptureState {
    changes: Vec<RuntimeViewModelCapturedChange>,
    maximum_changes: usize,
    maximum_value_bytes: usize,
    value_bytes: usize,
    overflowed: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RuntimeViewModelChangeLimitExceeded;

/// RAII owner for one thread-confined, operation-scoped change journal.
/// Dropping an unfinished capture discards every entry, which lets the caller
/// align publication with its own transaction commit.
#[derive(Debug)]
pub struct RuntimeViewModelChangeCapture {
    armed: bool,
}

impl RuntimeViewModelChangeCapture {
    pub fn begin() -> Option<Self> {
        Self::begin_bounded(4_096, 8 * 1024 * 1024)
    }

    pub fn begin_bounded(maximum_changes: usize, maximum_value_bytes: usize) -> Option<Self> {
        VIEW_MODEL_CHANGE_CAPTURE.with(|slot| {
            let mut slot = slot.borrow_mut();
            if slot.is_some() {
                return None;
            }
            *slot = Some(RuntimeViewModelChangeCaptureState {
                changes: Vec::new(),
                maximum_changes,
                maximum_value_bytes,
                value_bytes: 0,
                overflowed: false,
            });
            Some(Self { armed: true })
        })
    }

    pub(crate) fn finish(
        mut self,
    ) -> Result<Vec<RuntimeViewModelCapturedChange>, RuntimeViewModelChangeLimitExceeded> {
        self.armed = false;
        VIEW_MODEL_CHANGE_CAPTURE.with(|slot| {
            let state = slot
                .borrow_mut()
                .take()
                .ok_or(RuntimeViewModelChangeLimitExceeded)?;
            if state.overflowed {
                Err(RuntimeViewModelChangeLimitExceeded)
            } else {
                Ok(state.changes)
            }
        })
    }
}

impl Drop for RuntimeViewModelChangeCapture {
    fn drop(&mut self) {
        if self.armed {
            VIEW_MODEL_CHANGE_CAPTURE.with(|slot| {
                let _ = slot.borrow_mut().take();
            });
        }
    }
}

pub(crate) fn capture_view_model_change(cell_identity: usize, value: RuntimeViewModelChangeValue) {
    VIEW_MODEL_CHANGE_CAPTURE.with(|slot| {
        if let Some(state) = slot.borrow_mut().as_mut() {
            if state.overflowed {
                return;
            }
            let value_bytes = match &value {
                RuntimeViewModelChangeValue::String(value) => value.len(),
                RuntimeViewModelChangeValue::List(values) => {
                    values.len().saturating_mul(std::mem::size_of::<u64>())
                }
                _ => 0,
            };
            let Some(total) = state.value_bytes.checked_add(value_bytes) else {
                state.overflowed = true;
                return;
            };
            if state.changes.len() >= state.maximum_changes || total > state.maximum_value_bytes {
                state.overflowed = true;
                return;
            }
            state.value_bytes = total;
            state.changes.push(RuntimeViewModelCapturedChange {
                cell_identity,
                value,
            });
        }
    });
}

pub(crate) fn is_capturing_view_model_changes() -> bool {
    VIEW_MODEL_CHANGE_CAPTURE.with(|state| state.borrow().is_some())
}

struct RuntimeHostMutationCallbackFirewall;

impl RuntimeHostMutationCallbackFirewall {
    fn enter() -> Self {
        HOST_MUTATION_CALLBACK_FIREWALL_DEPTH.with(|depth| {
            depth.set(depth.get().saturating_add(1));
        });
        Self
    }
}

impl Drop for RuntimeHostMutationCallbackFirewall {
    fn drop(&mut self) {
        HOST_MUTATION_CALLBACK_FIREWALL_DEPTH.with(|depth| {
            depth.set(depth.get().saturating_sub(1));
        });
    }
}

/// Thread-confined notification buffer for one host mutation transaction.
/// Payload/topology writes remain immediately available to later operations
/// in the same batch, while dirt/listener publication is deferred to commit.
#[derive(Debug)]
pub(crate) struct RuntimeHostMutationNotifications {
    armed: bool,
}

impl RuntimeHostMutationNotifications {
    pub(crate) fn begin() -> Option<Self> {
        HOST_MUTATION_NOTIFICATIONS.with(|slot| {
            let mut slot = slot.borrow_mut();
            if slot.is_some() {
                return None;
            }
            *slot = Some(Vec::new());
            Some(Self { armed: true })
        })
    }

    pub(crate) fn commit(mut self) {
        self.armed = false;
        let notifications = HOST_MUTATION_NOTIFICATIONS.with(|slot| {
            slot.borrow_mut()
                .take()
                .expect("active host mutation notification buffer")
        });
        // Rust/script observers are host extension points and may panic. The
        // final publication phase isolates those callbacks individually (in
        // `cascade`) so one observer cannot convert an already validated
        // atomic batch into a partial failure or suppress later dependents.
        let _callback_firewall = RuntimeHostMutationCallbackFirewall::enter();
        for notification in notifications {
            notification();
        }
    }

    pub(crate) fn discard(mut self) {
        self.armed = false;
        HOST_MUTATION_NOTIFICATIONS.with(|slot| {
            let _ = slot.borrow_mut().take();
        });
    }
}

impl Drop for RuntimeHostMutationNotifications {
    fn drop(&mut self) {
        if self.armed {
            HOST_MUTATION_NOTIFICATIONS.with(|slot| {
                let _ = slot.borrow_mut().take();
            });
        }
    }
}

pub(crate) fn defer_host_mutation_notification(notification: impl FnOnce() + 'static) -> bool {
    HOST_MUTATION_NOTIFICATIONS.with(|slot| {
        let mut slot = slot.borrow_mut();
        let Some(notifications) = slot.as_mut() else {
            return false;
        };
        notifications.push(Box::new(notification));
        true
    })
}

/// Retained mutation-order notification queue used by ViewModel listeners.
///
/// C++ `StateMachineInstance::m_reportedListenerViewModels` is appended from
/// `ListenerViewModelPropertyBinding::addDirt` and swapped into a second
/// retained vector by `applyEvents` (`state_machine_instance.cpp:1374-1380,
/// 2320-2335,3021-3025`). The shared handle lets a retained cell append
/// without borrowing the owning state machine; the state machine remains the
/// sole owner of the handle and drains it at the frame boundary.
#[derive(Clone, Default)]
pub(crate) struct RuntimeCellNotificationQueue {
    values: Rc<RefCell<Vec<usize>>>,
}

impl std::fmt::Debug for RuntimeCellNotificationQueue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RuntimeCellNotificationQueue")
            .field("len", &self.len())
            .finish()
    }
}

impl RuntimeCellNotificationQueue {
    /// Copy pending notification values into a distinct queue.
    ///
    /// Rust's public state-machine `Clone` is a snapshot adaptation with no
    /// C++ copy-constructor counterpart. The snapshot keeps pending values,
    /// but its listener sinks must append to separate storage.
    pub(crate) fn detached_clone(&self) -> Self {
        Self {
            values: Rc::new(RefCell::new(self.values.borrow().clone())),
        }
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.values.borrow().is_empty()
    }

    pub(crate) fn len(&self) -> usize {
        self.values.borrow().len()
    }

    /// Swap the pending queue into the retained reporting buffer. The empty
    /// reporting allocation becomes the next pending allocation, matching
    /// C++'s two retained vectors without dropping either capacity.
    pub(crate) fn swap_into(&self, reporting: &mut Vec<usize>) {
        reporting.clear();
        std::mem::swap(&mut *self.values.borrow_mut(), reporting);
    }

    /// Re-report retained DataBind dirt while moving a bind onto a fresh
    /// container queue (for example when cloning an artboard). The sink bits
    /// themselves are restored separately so primary and converter-operand
    /// origin remain distinguishable.
    pub(crate) fn report_data_bind(&self, data_bind_index: usize) {
        self.values.borrow_mut().push(data_bind_index);
    }

    /// Remove queued reports for an occurrence that is about to consume its
    /// already-active dirty turn. This is the Rust queue counterpart of C++
    /// clearing `DataBind::inDirtyList` immediately before `updateDataBind`.
    pub(crate) fn remove_data_bind(&self, data_bind_index: usize) {
        self.values
            .borrow_mut()
            .retain(|candidate| *candidate != data_bind_index);
    }

    fn downgrade(&self) -> Weak<RefCell<Vec<usize>>> {
        Rc::downgrade(&self.values)
    }
}

/// Dirt bits a cell mutation cascades to its dependents.
///
/// C++ cascades `ComponentDirt::Bindings`; the rebuild keeps one bit per
/// concern so the update cycle can cheaply ask "which of my binds observed a
/// source change since the last pass".
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RuntimeCellDirt(u8);

impl RuntimeCellDirt {
    pub const NONE: Self = Self(0);
    /// The bound source value changed (C++ `ComponentDirt::Bindings`).
    pub const BINDINGS: Self = Self(1 << 0);
    /// The bound target value changed (C++ `ComponentDirt::BindingsTarget`).
    pub const BINDINGS_TARGET: Self = Self(1 << 1);

    pub fn contains(self, other: Self) -> bool {
        self.0 & other.0 == other.0
    }

    pub fn insert(&mut self, other: Self) {
        self.0 |= other.0;
    }

    pub fn take(&mut self) -> Self {
        std::mem::replace(self, Self::NONE)
    }

    pub fn is_empty(self) -> bool {
        self.0 == 0
    }
}

/// One dependent's dirt sink.
///
/// C++ stores a raw `DataBind*` and calls `addDirt` on it. The Rust analog
/// is a shared bit-cell owned by the dependent (a retained bind, listener,
/// or converter operand); the cell holds only a weak reference, so a dropped
/// dependent unregisters itself implicitly, mirroring `~DataBind`'s
/// `removeDependent` without manual lifetime bookkeeping.
#[derive(Clone, Default)]
pub struct RuntimeCellDirtSink {
    bits: Rc<Cell<u8>>,
    /// C++ `DataBind::Flag::SuppressDirt`: generated target/source writes
    /// temporarily mute only the writing dependent while sibling dependents
    /// on the same retained value continue to receive dirt.
    suppressed: Rc<Cell<bool>>,
    notification: Option<RuntimeCellNotification>,
    /// C++ converter-owned DataBinds notify their parent converter before
    /// entering the converter's own dirty queue. The callback owns that
    /// complete parent-first notification path; ordinary sinks leave it
    /// empty and use `notification` directly.
    before_notify: Rc<RefCell<Option<Rc<dyn Fn(RuntimeCellDirt) -> bool>>>>,
    /// Retains an adapter's observer registration, never a mirrored value.
    retained_owner: Option<Rc<dyn std::any::Any>>,
}

impl std::fmt::Debug for RuntimeCellDirtSink {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RuntimeCellDirtSink")
            .field("dirt", &self.peek_dirt())
            .finish_non_exhaustive()
    }
}

#[derive(Clone)]
struct RuntimeCellNotification {
    queue: Weak<RefCell<Vec<usize>>>,
    value: usize,
    suppress_trigger_zero: bool,
    dedupe_while_dirty: bool,
}

impl RuntimeCellDirtSink {
    pub fn new() -> Self {
        Self::default()
    }

    pub(crate) fn reporting_listener(
        queue: &RuntimeCellNotificationQueue,
        listener_index: usize,
    ) -> Self {
        Self {
            bits: Rc::new(Cell::new(0)),
            suppressed: Rc::new(Cell::new(false)),
            notification: Some(RuntimeCellNotification {
                queue: queue.downgrade(),
                value: listener_index,
                suppress_trigger_zero: true,
                dedupe_while_dirty: false,
            }),
            before_notify: Rc::new(RefCell::new(None)),
            retained_owner: None,
        }
    }

    /// Report the exact retained DataBind index when this sink first becomes
    /// dirty. Further cascades stay coalesced until the owner drains `bits`,
    /// matching C++ `DataBind::addDirt`'s already-marked early return and the
    /// container's occurrence queue (`data_bind.cpp:502-507`;
    /// `data_bind_container.cpp:115-147`).
    pub(crate) fn reporting_data_bind(
        queue: &RuntimeCellNotificationQueue,
        data_bind_index: usize,
    ) -> Self {
        Self {
            bits: Rc::new(Cell::new(0)),
            suppressed: Rc::new(Cell::new(false)),
            notification: Some(RuntimeCellNotification {
                queue: queue.downgrade(),
                value: data_bind_index,
                suppress_trigger_zero: false,
                dedupe_while_dirty: true,
            }),
            before_notify: Rc::new(RefCell::new(None)),
            retained_owner: None,
        }
    }

    pub fn add_dirt(&self, dirt: RuntimeCellDirt) {
        self.bits.set(self.bits.get() | dirt.0);
    }

    pub(crate) fn retain_owner(&mut self, owner: Rc<dyn std::any::Any>) {
        self.retained_owner = Some(owner);
    }

    pub fn take_dirt(&self) -> RuntimeCellDirt {
        RuntimeCellDirt(self.bits.replace(0))
    }

    pub fn peek_dirt(&self) -> RuntimeCellDirt {
        RuntimeCellDirt(self.bits.get())
    }

    pub(crate) fn suppress_dirt(&self, suppressed: bool) {
        self.suppressed.set(suppressed);
    }

    pub(crate) fn set_before_notify(
        &self,
        before_notify: Option<Rc<dyn Fn(RuntimeCellDirt) -> bool>>,
    ) {
        *self.before_notify.borrow_mut() = before_notify;
    }

    pub(crate) fn downgrade(&self) -> RuntimeCellDependent {
        RuntimeCellDependent {
            bits: Rc::downgrade(&self.bits),
            suppressed: Rc::downgrade(&self.suppressed),
            notification: self.notification.clone(),
            before_notify: Rc::downgrade(&self.before_notify),
        }
    }
}

#[derive(Clone)]
pub(crate) struct RuntimeCellDependent {
    bits: Weak<Cell<u8>>,
    suppressed: Weak<Cell<bool>>,
    notification: Option<RuntimeCellNotification>,
    before_notify: Weak<RefCell<Option<Rc<dyn Fn(RuntimeCellDirt) -> bool>>>>,
}

impl std::fmt::Debug for RuntimeCellDependent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RuntimeCellDependent")
            .field("alive", &(self.bits.strong_count() != 0))
            .finish_non_exhaustive()
    }
}

impl RuntimeCellDependent {
    pub(crate) fn add_dirt(&self, dirt: RuntimeCellDirt) -> bool {
        let Some(bits) = self.bits.upgrade() else {
            return false;
        };
        let Some(suppressed) = self.suppressed.upgrade() else {
            return false;
        };
        if suppressed.get() {
            return true;
        }
        let was_dirty = bits.get() & dirt.0 == dirt.0;
        let handled_notification = if !dirt.is_empty() && !was_dirty {
            invoke_before_notify(&self.before_notify, dirt)
        } else {
            None
        };
        if handled_notification == Some(false) {
            return true;
        }
        bits.set(bits.get() | dirt.0);
        if !dirt.is_empty()
            && handled_notification != Some(true)
            && let Some(notification) = &self.notification
            && (!notification.dedupe_while_dirty || !was_dirty)
            && !notification.suppress_trigger_zero
            && let Some(queue) = notification.queue.upgrade()
        {
            queue.borrow_mut().push(notification.value);
        }
        true
    }

    pub(crate) fn ptr_eq(&self, other: &Self) -> bool {
        self.bits.ptr_eq(&other.bits)
    }

    fn publish_dirt(
        &self,
        dirt: RuntimeCellDirt,
        invoke_delegate: bool,
        suppress_trigger_zero: bool,
    ) -> bool {
        let Some(bits) = self.bits.upgrade() else {
            return false;
        };
        let Some(suppressed) = self.suppressed.upgrade() else {
            return false;
        };
        if suppressed.get() {
            return true;
        }
        let was_dirty = bits.get() & dirt.0 == dirt.0;
        let handled_notification = if invoke_delegate && !dirt.is_empty() && !was_dirty {
            invoke_before_notify(&self.before_notify, dirt)
        } else {
            None
        };
        if handled_notification == Some(false) {
            return true;
        }
        bits.set(bits.get() | dirt.0);
        if let Some(notification) = &self.notification
            && handled_notification != Some(true)
            && (!notification.dedupe_while_dirty || !was_dirty)
            && !(notification.suppress_trigger_zero && suppress_trigger_zero)
            && let Some(queue) = notification.queue.upgrade()
        {
            queue.borrow_mut().push(notification.value);
        }
        true
    }
}

fn invoke_before_notify(
    before_notify: &Weak<RefCell<Option<Rc<dyn Fn(RuntimeCellDirt) -> bool>>>>,
    dirt: RuntimeCellDirt,
) -> Option<bool> {
    let invoke = || {
        before_notify
            .upgrade()
            .and_then(|callback| callback.borrow().clone())
            .map(|callback| callback(dirt))
    };
    if HOST_MUTATION_CALLBACK_FIREWALL_DEPTH.with(Cell::get) != 0 {
        std::panic::catch_unwind(std::panic::AssertUnwindSafe(invoke))
            .ok()
            .flatten()
    } else {
        invoke()
    }
}

/// The typed payload of one cell.
///
/// Mirrors the concrete scalar `ViewModelInstance*` C++ subclasses. Structural
/// `ViewModel` and `List` variants are dirt/type markers: their owning source
/// objects retain the child/list identity, just as C++ keeps the value on the
/// structural property rather than copying it into `DependencyHelper`.
#[derive(Debug, Clone, PartialEq)]
pub enum RuntimeViewModelCellValue {
    Number(f32),
    Boolean(bool),
    String(Arc<[u8]>),
    Color(u32),
    Enum(u32),
    /// Fired-counter semantics: the value is the cumulative fire count while
    /// live; C++ `ViewModelInstanceTrigger::advanced()` zeroes it each
    /// advance under suppressed delegation.
    Trigger(u64),
    SymbolListIndex(u32),
    AssetImage(u32),
    /// Complete two-part C++ `ViewModelInstanceAssetFont` value. The cell is
    /// the sole owner of both the serialized file-asset index and retained
    /// live Font identity, so every alias retains one exact source object.
    AssetFont(RuntimeFontAssetValue),
    /// Complete id-or-live value stored by `ViewModelInstanceAssetBlob`.
    AssetBlob(RuntimeBlobAssetValue),
    Artboard(u32),
    /// Dirt identity for a retained `ViewModelInstanceList` property. The
    /// data-bind source retains the list itself and reads its current items;
    /// the cell deliberately carries no copied structural payload.
    List,
    /// Dirt identity for a retained ViewModel-valued property. The data-bind
    /// source retains the endpoint and reads its current linked child.
    ViewModel,
}

/// The two-part value stored by C++ `ViewModelInstanceAssetFont`.
///
/// `file_asset_index` is the serialized `propertyValue`. `live_font_bytes`
/// is the private runtime-only Font retained alongside it. Equality follows
/// the C++ setters: the index compares by value and the live Font compares by
/// retained pointer identity, not byte content.
#[derive(Debug, Clone)]
pub struct RuntimeFontAssetValue {
    file_asset_index: u64,
    live_font_bytes: Option<Arc<[u8]>>,
}

/// Serialized asset id plus the optional directly-set Blob asset.
///
/// A non-null live value is distinct from an id-bound value even when its
/// bytes are empty. Live identity follows retained-pointer identity.
#[derive(Debug, Clone)]
pub struct RuntimeBlobAssetValue {
    file_asset_index: u64,
    live_blob_asset: Option<Arc<RuntimeBlobAsset>>,
}

impl PartialEq for RuntimeBlobAssetValue {
    fn eq(&self, other: &Self) -> bool {
        self.same_runtime_value(other)
    }
}

impl RuntimeBlobAssetValue {
    pub const MISSING_FILE_ASSET_INDEX: u64 = u32::MAX as u64;

    pub fn from_file_asset_index(file_asset_index: u64) -> Self {
        Self {
            file_asset_index,
            live_blob_asset: None,
        }
    }

    pub fn from_live_bytes(bytes: Arc<[u8]>) -> Self {
        Self::from_live_asset(Arc::new(RuntimeBlobAsset::new("", bytes)))
    }

    pub fn from_live_asset(asset: Arc<RuntimeBlobAsset>) -> Self {
        Self {
            file_asset_index: Self::MISSING_FILE_ASSET_INDEX,
            live_blob_asset: Some(asset),
        }
    }

    pub fn file_asset_index(&self) -> u64 {
        self.file_asset_index
    }

    pub fn live_blob_bytes(&self) -> Option<&[u8]> {
        self.live_blob_asset.as_deref().map(RuntimeBlobAsset::bytes)
    }

    pub fn live_blob_bytes_arc(&self) -> Option<Arc<[u8]>> {
        self.live_blob_asset
            .as_deref()
            .map(RuntimeBlobAsset::bytes_arc)
    }

    pub fn live_blob_asset(&self) -> Option<&Arc<RuntimeBlobAsset>> {
        self.live_blob_asset.as_ref()
    }

    fn same_runtime_value(&self, value: &Self) -> bool {
        self.file_asset_index == value.file_asset_index
            && match (&self.live_blob_asset, &value.live_blob_asset) {
                (Some(current), Some(next)) => Arc::ptr_eq(current, next),
                (None, None) => true,
                _ => false,
            }
    }

    pub(crate) fn set_file_asset_index(&mut self, file_asset_index: u64) -> bool {
        if self.file_asset_index == file_asset_index {
            return false;
        }
        self.file_asset_index = file_asset_index;
        true
    }

    pub(crate) fn set_live_blob_bytes(&mut self, bytes: Option<Arc<[u8]>>) -> bool {
        let same = match (&self.live_blob_asset, &bytes) {
            (Some(current), Some(next)) => current.bytes_arc_ptr_eq(next),
            (None, None) => true,
            _ => false,
        };
        let was_missing = self.file_asset_index == Self::MISSING_FILE_ASSET_INDEX;
        self.file_asset_index = Self::MISSING_FILE_ASSET_INDEX;
        if same {
            return !was_missing;
        }
        self.live_blob_asset = bytes.map(|bytes| Arc::new(RuntimeBlobAsset::new("", bytes)));
        true
    }

    pub(crate) fn set_live_blob_asset(&mut self, asset: Option<Arc<RuntimeBlobAsset>>) -> bool {
        let same = match (&self.live_blob_asset, &asset) {
            (Some(current), Some(next)) => Arc::ptr_eq(current, next),
            (None, None) => true,
            _ => false,
        };
        let was_missing = self.file_asset_index == Self::MISSING_FILE_ASSET_INDEX;
        self.file_asset_index = Self::MISSING_FILE_ASSET_INDEX;
        if same {
            return !was_missing;
        }
        self.live_blob_asset = asset;
        true
    }

    pub(crate) fn apply_data_bind_value(&mut self, value: &Self) -> bool {
        if self.same_runtime_value(value) {
            return false;
        }
        self.file_asset_index = value.file_asset_index;
        self.live_blob_asset = value.live_blob_asset.clone();
        true
    }
}

impl Default for RuntimeBlobAssetValue {
    fn default() -> Self {
        Self::from_file_asset_index(Self::MISSING_FILE_ASSET_INDEX)
    }
}

impl PartialEq for RuntimeFontAssetValue {
    fn eq(&self, other: &Self) -> bool {
        self.same_runtime_value(other)
    }
}

impl RuntimeFontAssetValue {
    pub const MISSING_FILE_ASSET_INDEX: u64 = u32::MAX as u64;

    pub fn from_file_asset_index(file_asset_index: u64) -> Self {
        Self {
            file_asset_index,
            live_font_bytes: None,
        }
    }

    pub fn file_asset_index(&self) -> u64 {
        self.file_asset_index
    }

    pub fn live_font_bytes(&self) -> Option<&[u8]> {
        self.live_font_bytes.as_deref()
    }

    pub fn live_font_bytes_arc(&self) -> Option<&Arc<[u8]>> {
        self.live_font_bytes.as_ref()
    }

    pub(crate) fn same_runtime_value(&self, value: &Self) -> bool {
        if self.file_asset_index != value.file_asset_index {
            return false;
        }
        match (&self.live_font_bytes, &value.live_font_bytes) {
            (Some(current), Some(next)) => Arc::ptr_eq(current, next),
            (None, None) => true,
            _ => false,
        }
    }

    pub(crate) fn set_file_asset_index(&mut self, file_asset_index: u64) -> bool {
        if self.file_asset_index == file_asset_index {
            return false;
        }
        self.file_asset_index = file_asset_index;
        true
    }

    pub(crate) fn set_live_font_bytes(&mut self, font_bytes: Option<Arc<[u8]>>) -> bool {
        let same_live_font = match (&self.live_font_bytes, &font_bytes) {
            (Some(current), Some(next)) => Arc::ptr_eq(current, next),
            (None, None) => true,
            _ => false,
        };
        let was_missing = self.file_asset_index == Self::MISSING_FILE_ASSET_INDEX;
        self.file_asset_index = Self::MISSING_FILE_ASSET_INDEX;
        if same_live_font {
            return !was_missing;
        }
        self.live_font_bytes = font_bytes;
        true
    }

    /// Apply the complete value carried by a font data bind.
    ///
    /// A public `propertyValue` write preserves the private live Font in C++,
    /// while `ViewModelInstanceAssetFont::applyValue(DataValueInteger*)`
    /// first applies the retained Font payload and only falls back to the
    /// serialized file-asset index.
    pub(crate) fn apply_data_bind_value(&mut self, value: &Self) -> bool {
        if self.same_runtime_value(value) {
            return false;
        }
        self.file_asset_index = value.file_asset_index;
        self.live_font_bytes = value.live_font_bytes.clone();
        true
    }
}

impl Default for RuntimeFontAssetValue {
    fn default() -> Self {
        Self::from_file_asset_index(Self::MISSING_FILE_ASSET_INDEX)
    }
}

impl RuntimeViewModelCellValue {
    fn same_kind(&self, other: &Self) -> bool {
        std::mem::discriminant(self) == std::mem::discriminant(other)
    }

    fn captured_value(&self) -> Option<RuntimeViewModelChangeValue> {
        Some(match self {
            Self::Number(value) => RuntimeViewModelChangeValue::Number(*value),
            Self::Boolean(value) => RuntimeViewModelChangeValue::Boolean(*value),
            Self::String(value) => RuntimeViewModelChangeValue::String(Arc::clone(value)),
            Self::Color(value) => RuntimeViewModelChangeValue::Color(*value),
            Self::Enum(value) => RuntimeViewModelChangeValue::Enum(u64::from(*value)),
            Self::Trigger(value) => RuntimeViewModelChangeValue::Trigger(*value),
            Self::SymbolListIndex(value) => {
                RuntimeViewModelChangeValue::ListIndex(u64::from(*value))
            }
            Self::AssetImage(value) => RuntimeViewModelChangeValue::Image(u64::from(*value)),
            Self::AssetFont(value) => RuntimeViewModelChangeValue::Font(value.file_asset_index()),
            Self::AssetBlob(value) => RuntimeViewModelChangeValue::Blob(value.file_asset_index()),
            Self::Artboard(value) => RuntimeViewModelChangeValue::Artboard(u64::from(*value)),
            Self::List | Self::ViewModel => return None,
        })
    }
}
