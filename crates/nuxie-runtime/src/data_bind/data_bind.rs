//! Direct Rust owner for pinned C++ `src/data_bind/data_bind.cpp`.
//!
//! Retained `DataBind` lifecycle — #RB-1 slice (d).
//!
//! Ports the direction engine of C++ `data_bind.cpp`:
//!
//! - `DataBind::source(rcp<ViewModelInstanceValue>)` retains the cell and
//!   registers the bind as a dependent (skipped for `bindsOnce()`).
//! - Two dirt bits drive the update cycle: `Bindings` (source changed →
//!   apply source→target in `update`) and `BindingsTarget` (target changed →
//!   apply target→source in `updateSourceBinding`).
//! - `reconcileDirt()` on (re)bind marks every supported direction so both
//!   sides sync in favor order; `TargetOrigin` latches which side a change
//!   came from, with the favored direction winning a reconcile.
//! - `suppressDirt` guards both apply paths so writing one side does not
//!   self-notify and schedule a spurious extra pass.
//!
//! Target application goes through [`RuntimeDataBindTarget`], the seam the
//! migration slice (e) wires to the instance object arena and the converter
//! stack. This module owns direction/dirt/identity semantics only.

use crate::data_bind_graph::{
    data_bind_flags_apply_source_to_target, data_bind_flags_apply_target_to_source,
    data_bind_flags_source_to_target_runs_first,
};
use crate::view_model_cell::{
    RuntimeCellDirt, RuntimeCellDirtSink, RuntimeCellNotificationQueue, RuntimeViewModelCell,
    RuntimeViewModelCellValue,
};
use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::sync::Arc;

/// The bind's target seam. C++ writes `Core*` properties through generated
/// setters; the Rust migration applies through the instance object arena.
pub trait RuntimeDataBindTarget {
    /// Apply a source-derived value to the target property
    /// (`DataBindContextValue::apply`).
    fn apply_to_target(&mut self, value: &RuntimeViewModelCellValue);

    /// Read the target property back for target→source application
    /// (`DataBindContextValue::applyToSource` input). `None` when the target
    /// cannot currently produce a value.
    fn read_target(&mut self) -> Option<RuntimeViewModelCellValue>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct RuntimeRetainedDataBindCollapse {
    pub(crate) changed: bool,
    pub(crate) requests_dirty_update: bool,
}

/// Canonical occurrence owner shared by Artboard and future DataBind hosts.
///
/// The host-specific target/source/converter types are parameters so the
/// mapped `DataBind` file owns their lifetime as one object without coupling
/// the direction engine to Artboard's execution adapters.
#[derive(Debug, Clone)]
pub(crate) struct RuntimeAuthoredDataBind<TTarget, TSource, TConverter> {
    pub(crate) target: Option<TTarget>,
    pub(crate) path: Arc<[u32]>,
    pub(crate) path_is_name_based: bool,
    pub(crate) retained: RuntimeRetainedDataBind,
    pub(crate) source: Option<TSource>,
    pub(crate) shared_converter: Option<TConverter>,
    pub(crate) suppress_target_notifications: bool,
}

/// Converter clone and its occurrence-local mutable state, owned together by
/// the outer DataBind exactly like C++ `DataBind::m_DataConverter`.
#[derive(Debug, Clone)]
pub(crate) struct RuntimeOwnedDataBindConverter<TConverter, TState> {
    pub(crate) converter: TConverter,
    pub(crate) converter_state: TState,
}

/// Occurrence-local portion of C++ `DataBind` that a child `DataConverter`
/// may wake through its retained `m_parentDataBind` pointer.
///
/// Rust converter bindings are stored beside, rather than inside, the outer
/// bind. Sharing only this state preserves the exact parent identity without
/// a self-referential Rust owner. Cloning a DataBind creates a fresh state;
/// converter clones are explicitly reattached to that fresh occurrence.
#[derive(Debug)]
struct RuntimeRetainedDataBindWakeState {
    source_to_target_runs_first: bool,
    target_origin: Cell<bool>,
    suppress_dirt: Cell<bool>,
    collapsed: Cell<bool>,
    dirt: Cell<RuntimeCellDirt>,
    notification: RefCell<Option<(RuntimeCellNotificationQueue, usize)>>,
}

impl RuntimeRetainedDataBindWakeState {
    fn new(flags: u64) -> Self {
        Self {
            source_to_target_runs_first: data_bind_flags_source_to_target_runs_first(flags),
            target_origin: Cell::new(false),
            suppress_dirt: Cell::new(false),
            collapsed: Cell::new(false),
            dirt: Cell::new(RuntimeCellDirt::NONE),
            notification: RefCell::new(None),
        }
    }

    fn accepts(&self, dirt: RuntimeCellDirt) -> bool {
        !self.suppress_dirt.get() && !dirt.is_empty() && !self.dirt.get().contains(dirt)
    }

    /// C++ `DataBind::addDirt` (`data_bind.cpp:502-546`).
    fn add_dirt(&self, dirt: RuntimeCellDirt) -> bool {
        if !self.accepts(dirt) {
            return false;
        }
        let has_source = dirt.contains(RuntimeCellDirt::BINDINGS);
        let has_target = dirt.contains(RuntimeCellDirt::BINDINGS_TARGET);
        if has_source && has_target {
            self.target_origin.set(!self.source_to_target_runs_first);
        } else if has_target {
            self.target_origin.set(true);
        } else if has_source {
            self.target_origin.set(false);
        }
        let mut pending = self.dirt.get();
        pending.insert(dirt);
        self.dirt.set(pending);
        if !self.collapsed.get()
            && let Some((queue, index)) = self.notification.borrow().as_ref()
        {
            queue.report_data_bind(*index);
        }
        true
    }

    fn mark_converter_changed(&self) -> bool {
        let direction = if self.target_origin.get() {
            RuntimeCellDirt::BINDINGS_TARGET
        } else {
            RuntimeCellDirt::BINDINGS
        };
        self.add_dirt(direction)
    }
}

/// Cloneable equivalent of C++ `DataConverter::m_parentDataBind`.
#[derive(Clone, Debug)]
pub(crate) struct RuntimeConverterParentWake {
    state: Rc<RuntimeRetainedDataBindWakeState>,
}

impl RuntimeConverterParentWake {
    pub(crate) fn mark_converter_changed(&self) -> bool {
        self.state.mark_converter_changed()
    }
}

/// One retained data bind (C++ `DataBind`).
pub struct RuntimeRetainedDataBind {
    flags: u64,
    binds_once: bool,
    /// Primary `ViewModelInstanceValue` dependency. Formula converters also
    /// subscribe to this value in C++ so source-change randoms can distinguish
    /// it from an OperationViewModel operand notification.
    sink: RuntimeCellDirtSink,
    /// Additional converter operands still dirty this same outer DataBind,
    /// but retain a distinct dependency sink so the converter can preserve
    /// primary-source-only side effects (`data_converter_formula.cpp:526-543`;
    /// `data_converter_operation_viewmodel.cpp:48-59`).
    additional_sink: RuntimeCellDirtSink,
    source: Option<RuntimeViewModelCell>,
    /// Converter operands that dirty this SAME owning bind. C++
    /// `DataConverterOperationViewModel::bindFromContext` registers the
    /// outer `DataBind*` directly on its operand value
    /// (`data_converter_operation_viewmodel.cpp:48-59`). The distinct sink
    /// above is only an origin discriminator; both direct sinks fold into this
    /// same bind's source-origin `ComponentDirt::Bindings` latch.
    additional_sources: Vec<RuntimeViewModelCell>,
    wake_state: Rc<RuntimeRetainedDataBindWakeState>,
    /// Set only for a DataBind owned by a converter. C++ calls
    /// `DataConverter::markConverterDirty` before the inner container queues
    /// this occurrence (`data_converter.cpp:51-55`).
    container_wake: Option<RuntimeConverterParentWake>,
    /// `DataBind::collapse` exempts displayValue and targets that cannot push.
    collapse_eligible: bool,
}

impl std::fmt::Debug for RuntimeRetainedDataBind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RuntimeRetainedDataBind")
            .field("flags", &self.flags)
            .field("binds_once", &self.binds_once)
            .field("target_origin", &self.wake_state.target_origin.get())
            .field("collapsed", &self.wake_state.collapsed.get())
            .field("dirt", &self.wake_state.dirt.get())
            .finish_non_exhaustive()
    }
}

impl Clone for RuntimeRetainedDataBind {
    /// Duplicating a bind (graph clones for per-animation keyframe
    /// instances) re-registers a FRESH sink with the same retained cell so
    /// the copy observes subsequent writes independently; direction state
    /// carries over, pending sink dirt does not (the original consumes it).
    fn clone(&self) -> Self {
        let primary_dirt = self.sink.peek_dirt();
        let additional_dirt = self.additional_sink.peek_dirt();
        let mut cloned = Self::new(self.flags, self.binds_once);
        if let Some(source) = &self.source {
            cloned.set_source(source.clone());
        }
        cloned.set_additional_sources(self.additional_sources.clone());
        cloned
            .wake_state
            .target_origin
            .set(self.wake_state.target_origin.get());
        cloned.wake_state.dirt.set(self.wake_state.dirt.get());
        cloned.collapse_eligible = self.collapse_eligible;
        cloned.sink.add_dirt(primary_dirt);
        cloned.additional_sink.add_dirt(additional_dirt);
        cloned
    }
}

impl Drop for RuntimeRetainedDataBind {
    fn drop(&mut self) {
        // Pinned `~DataBind` immediately detaches the occurrence from every
        // retained source. Rust's weak dirt sinks make a later cascade safe,
        // but leaving dead weak entries until that later write would not
        // preserve the C++ teardown boundary (`data_bind.cpp:239-249,
        // 354-369`; `data_converter_operation_viewmodel.cpp:48-59`).
        self.unregister_additional_sources();
        self.unregister_primary_source();
    }
}

impl RuntimeRetainedDataBind {
    pub fn new(flags: u64, binds_once: bool) -> Self {
        Self {
            flags,
            binds_once,
            sink: RuntimeCellDirtSink::new(),
            additional_sink: RuntimeCellDirtSink::new(),
            source: None,
            additional_sources: Vec::new(),
            wake_state: Rc::new(RuntimeRetainedDataBindWakeState::new(flags)),
            container_wake: None,
            collapse_eligible: false,
        }
    }

    pub(crate) fn set_collapse_eligible(&mut self, collapse_eligible: bool) {
        self.collapse_eligible = collapse_eligible;
    }

    pub(crate) fn is_collapsed(&self) -> bool {
        self.wake_state.collapsed.get()
    }

    /// Exact `DataBind::collapse` branch table
    /// (`src/data_bind/data_bind.cpp:595-607`).
    pub(crate) fn collapse(&mut self, collapsed: bool) -> RuntimeRetainedDataBindCollapse {
        if self.wake_state.collapsed.get() == collapsed || !self.collapse_eligible {
            return RuntimeRetainedDataBindCollapse {
                changed: false,
                requests_dirty_update: false,
            };
        }
        self.wake_state.collapsed.set(collapsed);
        RuntimeRetainedDataBindCollapse {
            changed: true,
            requests_dirty_update: !collapsed && !self.wake_state.dirt.get().is_empty(),
        }
    }

    pub fn to_target(&self) -> bool {
        data_bind_flags_apply_source_to_target(self.flags)
    }

    pub fn to_source(&self) -> bool {
        data_bind_flags_apply_target_to_source(self.flags)
    }

    pub fn source_to_target_runs_first(&self) -> bool {
        data_bind_flags_source_to_target_runs_first(self.flags)
    }

    pub fn source(&self) -> Option<&RuntimeViewModelCell> {
        self.source.as_ref()
    }

    pub(crate) fn has_sources(&self) -> bool {
        self.source.is_some() || !self.additional_sources.is_empty()
    }

    fn primary_source_is_an_additional_source(&self) -> bool {
        self.source.as_ref().is_some_and(|primary| {
            self.additional_sources
                .iter()
                .any(|additional| primary.ptr_eq(additional))
        })
    }

    fn registers_primary_sink(&self) -> bool {
        !self.binds_once || self.primary_source_is_an_additional_source()
    }

    fn unregister_primary_source(&self) {
        if let Some(source) = &self.source
            && self.registers_primary_sink()
        {
            source.remove_dependent(&self.sink);
        }
    }

    fn unregister_additional_sources(&self) {
        for source in &self.additional_sources {
            if self
                .source
                .as_ref()
                .is_some_and(|primary| primary.ptr_eq(source))
            {
                continue;
            }
            source.remove_dependent(&self.additional_sink);
        }
    }

    fn register_primary_source(&self) {
        if let Some(source) = &self.source
            && self.registers_primary_sink()
        {
            source.add_dependent(&self.sink);
        }
    }

    fn register_additional_sources(&self) {
        // Converter operands register the outer DataBind even when the
        // primary bind is `bindsOnce`
        // (`data_converter_operation_viewmodel.cpp:48-59`).
        for source in &self.additional_sources {
            // C++ registers one `DataBind*`; DependencyHelper deduplicates
            // that identity when an operand aliases the primary source.
            if self
                .source
                .as_ref()
                .is_some_and(|primary| primary.ptr_eq(source))
            {
                continue;
            }
            source.add_dependent(&self.additional_sink);
        }
    }

    pub(crate) fn converter_parent_wake(&self) -> RuntimeConverterParentWake {
        RuntimeConverterParentWake {
            state: self.wake_state.clone(),
        }
    }

    fn install_container_wake_on_sinks(&self) {
        let callback = self.container_wake.as_ref().map(|parent| {
            let parent = parent.clone();
            let own = self.wake_state.clone();
            Rc::new(move |dirt: RuntimeCellDirt| {
                // `DataBind::addDirt` performs its already-dirty/suppressed
                // guard before `DataConverter::addDirtyDataBind`. Only an
                // accepted inner change may wake the outer occurrence.
                if !own.accepts(dirt) {
                    return false;
                }
                parent.mark_converter_changed();
                own.add_dirt(dirt)
            }) as Rc<dyn Fn(RuntimeCellDirt) -> bool>
        });
        self.sink.set_before_notify(callback.clone());
        self.additional_sink.set_before_notify(callback);
    }

    /// Attach this inner bind to the exact outer occurrence retained by its
    /// C++ `DataConverter::m_parentDataBind`.
    pub(crate) fn set_container_wake(&mut self, wake: Option<RuntimeConverterParentWake>) {
        self.container_wake = wake;
        self.install_container_wake_on_sinks();
    }

    /// C++ `DataBind::source(value)`: retain the cell, register as a
    /// dependent unless the bind binds once.
    pub fn set_source(&mut self, cell: RuntimeViewModelCell) {
        // Alias eligibility for converter operands changes with the primary.
        self.unregister_additional_sources();
        self.unregister_primary_source();
        self.source = Some(cell);
        self.register_primary_source();
        self.register_additional_sources();
        self.sink.take_dirt();
    }

    /// Re-home this bind's retained source sink onto an occurrence-indexed
    /// container queue. Artboard clones call this with their fresh queue, so
    /// each cloned container receives its own C++-shaped dirty occurrence.
    pub(crate) fn report_source_dirt_to(
        &mut self,
        queue: &RuntimeCellNotificationQueue,
        data_bind_index: usize,
    ) {
        let primary_dirt = self.sink.peek_dirt();
        let additional_dirt = self.additional_sink.peek_dirt();
        self.unregister_additional_sources();
        self.unregister_primary_source();
        self.sink = RuntimeCellDirtSink::reporting_data_bind(queue, data_bind_index);
        self.additional_sink = RuntimeCellDirtSink::reporting_data_bind(queue, data_bind_index);
        *self.wake_state.notification.borrow_mut() = Some((queue.clone(), data_bind_index));
        self.install_container_wake_on_sinks();
        self.register_primary_source();
        self.register_additional_sources();
        let mut must_report = !self.wake_state.dirt.get().is_empty();
        if !primary_dirt.is_empty() {
            self.sink.add_dirt(primary_dirt);
            must_report = true;
        }
        if !additional_dirt.is_empty() {
            self.additional_sink.add_dirt(additional_dirt);
            must_report = true;
        }
        if must_report {
            queue.report_data_bind(data_bind_index);
        }
    }

    /// Detach this occurrence from its container without dropping its
    /// retained source/value. Used by deferred DataBindContainer removal;
    /// source dirt raised after removal must not resurrect the old slot.
    pub(crate) fn clear_notification_queue(&mut self) {
        self.unregister_additional_sources();
        self.unregister_primary_source();
        self.sink = RuntimeCellDirtSink::new();
        self.additional_sink = RuntimeCellDirtSink::new();
        *self.wake_state.notification.borrow_mut() = None;
        self.install_container_wake_on_sinks();
        self.register_primary_source();
        self.register_additional_sources();
    }

    /// Recreate this occurrence's C++ direction engine while preserving its
    /// container/queue identity. DataContext rebinding replaces the retained
    /// source and dirt, not the `DataBind*` owned by the container.
    pub(crate) fn reset_preserving_notification(&mut self) {
        let notification = self.wake_state.notification.borrow().clone();
        let container_wake = self.container_wake.clone();
        let flags = self.flags;
        let binds_once = self.binds_once;
        *self = Self::new(flags, binds_once);
        self.set_container_wake(container_wake);
        if let Some((queue, index)) = notification {
            self.report_source_dirt_to(&queue, index);
        }
    }

    /// C++ `DataBind::clearSource()`.
    pub fn clear_source(&mut self) {
        // The former primary may remain registered as a converter operand.
        self.unregister_additional_sources();
        self.unregister_primary_source();
        self.source = None;
        self.register_additional_sources();
        // Any latched source dirt refers to the departed cell.
        self.sink.take_dirt();
    }

    /// Replace the converter-operand registrations owned by this bind.
    /// Duplicate cells are harmless: `RuntimeViewModelCell::add_dependent`
    /// deduplicates the shared sink by identity.
    pub(crate) fn set_additional_sources(&mut self, sources: Vec<RuntimeViewModelCell>) {
        if self.additional_sources.len() == sources.len()
            && self
                .additional_sources
                .iter()
                .zip(&sources)
                .all(|(current, next)| current.ptr_eq(next))
        {
            return;
        }
        self.unregister_additional_sources();
        self.unregister_primary_source();
        self.additional_sources = sources;
        self.register_primary_source();
        self.register_additional_sources();
        self.additional_sink.take_dirt();
    }

    /// Add converter-operand registrations without removing earlier ones.
    ///
    /// Pinned C++ `DataConverterOperationViewModel::bindFromContext` replaces
    /// its retained `m_source` pointer but never removes the owning
    /// `DataBind*` from the departed value. A later mutation of either the old
    /// or current operand therefore still dirties this exact outer bind.
    pub(crate) fn retain_additional_sources(&mut self, sources: Vec<RuntimeViewModelCell>) {
        let mut changed = false;
        for source in sources {
            if self
                .additional_sources
                .iter()
                .any(|current| current.ptr_eq(&source))
            {
                continue;
            }
            self.additional_sources.push(source);
            changed = true;
        }
        if !changed {
            return;
        }
        self.unregister_additional_sources();
        self.unregister_primary_source();
        self.register_primary_source();
        self.register_additional_sources();
        self.additional_sink.take_dirt();
    }

    /// C++ `DataBind::bind()` tail: a (re)bind is a reconcile — mark every
    /// supported direction; the favored direction decides the origin.
    pub fn mark_rebind_reconcile(&mut self) {
        let mut dirt = RuntimeCellDirt::NONE;
        if self.to_target() {
            dirt.insert(RuntimeCellDirt::BINDINGS);
        }
        if self.to_source() {
            dirt.insert(RuntimeCellDirt::BINDINGS_TARGET);
        }
        self.add_dirt(dirt);
    }

    /// The target mutated (C++ push-observer notify / polling): latch
    /// target-origin dirt.
    pub fn mark_target_changed(&mut self) {
        self.add_dirt(RuntimeCellDirt::BINDINGS_TARGET);
    }

    /// The source mutated through a non-cell compatibility seam. Slice (f)
    /// keeps this entry point while those remaining source kinds move onto
    /// retained cells; origin still belongs to the DataBind direction engine.
    pub fn mark_source_changed(&mut self) {
        self.add_dirt(RuntimeCellDirt::BINDINGS);
    }

    /// Requeue an unvisited converter-owned bind after an update error.
    ///
    /// The active container snapshot has already removed this occurrence, so
    /// even already-latched dirt must re-enter both the outer parent and inner
    /// queues in C++ parent-first order.
    pub(crate) fn requeue_source_dirt(&mut self) {
        if self.wake_state.accepts(RuntimeCellDirt::BINDINGS) {
            self.add_dirt(RuntimeCellDirt::BINDINGS);
            return;
        }
        if let Some(parent) = self.container_wake.as_ref() {
            parent.mark_converter_changed();
        }
        if !self.wake_state.collapsed.get()
            && let Some((queue, index)) = self.wake_state.notification.borrow().as_ref()
        {
            queue.report_data_bind(*index);
        }
    }

    /// C++ `DataConverter::markConverterDirty`: keep the direction already
    /// latched on the parent DataBind and ensure that occurrence is scheduled.
    pub(crate) fn mark_converter_changed(&mut self) {
        self.wake_state.mark_converter_changed();
    }

    /// C++ `DataBind::addDirt` with the origin latch: a reconcile (both
    /// bits) resolves the origin by favored direction; a one-sided change
    /// records its own side. Suppressed applies never re-dirty.
    fn add_dirt(&mut self, dirt: RuntimeCellDirt) {
        if !self.wake_state.accepts(dirt) {
            return;
        }
        if let Some(parent) = self.container_wake.as_ref() {
            parent.mark_converter_changed();
        }
        self.wake_state.add_dirt(dirt);
    }

    /// Fold sink dirt (cell cascades) into the latched dirt with the same
    /// origin rules; the update cycle calls this once per pass, mirroring
    /// how C++ receives `addDirt` calls directly from `DependencyHelper`.
    /// Returns whether any sink dirt was folded — distinguishing a genuine
    /// cell cascade from dirt latched earlier (e.g. a rebind reconcile).
    fn collect_source_dirt_parts(&mut self) -> (bool, bool) {
        let primary = !self.sink.take_dirt().is_empty();
        let additional = !self.additional_sink.take_dirt().is_empty();
        if primary || additional {
            self.add_dirt(RuntimeCellDirt::BINDINGS);
        }
        (primary, additional)
    }

    pub fn collect_source_dirt(&mut self) -> bool {
        let (primary, additional) = self.collect_source_dirt_parts();
        primary || additional
    }

    /// Drain the retained source notification without applying through this
    /// module's generic target seam. Artboard's authored-bind scheduler owns
    /// the concrete target adapters and consumes this exact bit before it
    /// dispatches them (`data_bind_container.cpp:115-147,156-203`).
    #[cfg(test)]
    pub(crate) fn take_source_dirt(&mut self) -> bool {
        self.take_source_dirt_with_primary().is_some()
    }

    /// Drain one retained notification and report whether the primary source
    /// participated. C++ Formula source-change randoms subscribe to that
    /// primary value, whereas OperationViewModel operands dirty only the
    /// outer DataBind (`data_converter_formula.cpp:526-543`;
    /// `data_converter_operation_viewmodel.cpp:48-59`).
    pub(crate) fn take_source_dirt_with_primary(&mut self) -> Option<bool> {
        // `dirt` may already contain Bindings from a rebind reconcile. Only
        // report this compatibility drain when the retained cell's sink
        // actually pushed a new notification this pass.
        let (primary, additional) = self.collect_source_dirt_parts();
        if !primary && !additional {
            return None;
        }
        let dirt = self.wake_state.dirt.get();
        self.wake_state
            .dirt
            .set(if dirt.contains(RuntimeCellDirt::BINDINGS_TARGET) {
                RuntimeCellDirt::BINDINGS_TARGET
            } else {
                RuntimeCellDirt::NONE
            });
        Some(primary)
    }

    /// Consume the concrete target adapter's queued notification. This is
    /// the authored-bind scheduler counterpart to [`Self::take_source_dirt`]
    /// and preserves any simultaneously pending source reconcile bit.
    pub(crate) fn take_target_dirt(&mut self) -> bool {
        let dirt = self.wake_state.dirt.get();
        if !dirt.contains(RuntimeCellDirt::BINDINGS_TARGET) {
            return false;
        }
        self.wake_state
            .dirt
            .set(if dirt.contains(RuntimeCellDirt::BINDINGS) {
                RuntimeCellDirt::BINDINGS
            } else {
                RuntimeCellDirt::NONE
            });
        true
    }

    /// Consume a source bit whose concrete facade adapter has just applied.
    /// C++ removes the queued dirt before calling `DataBind::update`, so a
    /// later notification can latch a fresh origin (`data_bind_container.cpp:
    /// 144-147`, `data_bind.cpp:502-531`).
    pub(crate) fn take_pending_source_dirt(&mut self) -> bool {
        let dirt = self.wake_state.dirt.get();
        if !dirt.contains(RuntimeCellDirt::BINDINGS) {
            return false;
        }
        self.wake_state
            .dirt
            .set(if dirt.contains(RuntimeCellDirt::BINDINGS_TARGET) {
                RuntimeCellDirt::BINDINGS_TARGET
            } else {
                RuntimeCellDirt::NONE
            });
        true
    }

    pub fn pending_dirt(&self) -> RuntimeCellDirt {
        self.wake_state.dirt.get()
    }

    pub fn target_origin(&self) -> bool {
        self.wake_state.target_origin.get()
    }

    /// C++ `DataBind::update(ComponentDirt::Bindings)`: apply source→target
    /// under dirt, self-notification suppressed. Consumes the source bit.
    pub fn update(&mut self, target: &mut dyn RuntimeDataBindTarget) -> bool {
        let dirt = self.wake_state.dirt.get();
        if !dirt.contains(RuntimeCellDirt::BINDINGS) {
            return false;
        }
        self.wake_state
            .dirt
            .set(if dirt.contains(RuntimeCellDirt::BINDINGS_TARGET) {
                RuntimeCellDirt::BINDINGS_TARGET
            } else {
                RuntimeCellDirt::NONE
            });
        let Some(source) = self.source.as_ref() else {
            return false;
        };
        if !self.to_target() {
            return false;
        }
        let value = source.value();
        self.wake_state.suppress_dirt.set(true);
        target.apply_to_target(&value);
        self.wake_state.suppress_dirt.set(false);
        true
    }

    /// C++ `DataBind::updateSourceBinding()`: apply target→source. The
    /// source write cascades to OTHER dependents of the cell by identity;
    /// this bind's own sink echo is swallowed so it does not reschedule
    /// itself (the `suppressDirt` pattern).
    pub fn update_source_binding(&mut self, target: &mut dyn RuntimeDataBindTarget) -> bool {
        if !self.to_source() {
            return false;
        }
        let Some(value) = target.read_target() else {
            return false;
        };
        self.update_source_binding_value(value)
    }

    /// `DataBind::updateSourceBinding()` after a focused owner has read and,
    /// when necessary, reverse-converted its concrete target value.
    ///
    /// ScriptInput targets cannot implement the generic Core target adapter
    /// because their value may need a scripted converter before it reaches the
    /// retained ViewModel cell. Keep the direction/dirt/self-notification
    /// mechanics here so both owners still share the exact C++ behavior.
    pub(crate) fn update_source_binding_value(&mut self, value: RuntimeViewModelCellValue) -> bool {
        if !self.to_source() {
            return false;
        }
        let dirt = self.wake_state.dirt.get();
        self.wake_state
            .dirt
            .set(if dirt.contains(RuntimeCellDirt::BINDINGS) {
                RuntimeCellDirt::BINDINGS
            } else {
                RuntimeCellDirt::NONE
            });
        let Some(source) = self.source.as_ref() else {
            return false;
        };
        // C++ raises `SuppressDirt` around ContextValue::applyToSource, so the
        // writer never enters its container queue while sibling dependents
        // still observe the shared source write (`context_value.hpp:81-99`;
        // `data_bind.cpp:502-507`). The primary and converter-operand sinks
        // share that occurrence-local guard; other DataBinds remain live.
        self.sink.suppress_dirt(true);
        self.additional_sink.suppress_dirt(true);
        self.wake_state.suppress_dirt.set(true);
        let changed = source.set_value(value);
        self.wake_state.suppress_dirt.set(false);
        self.sink.suppress_dirt(false);
        self.additional_sink.suppress_dirt(false);
        changed
    }

    /// One settle pass in C++ favor order: the favored direction applies
    /// first, then the other side reconciles from the result.
    pub fn reconcile(&mut self, target: &mut dyn RuntimeDataBindTarget) {
        if self.source_to_target_runs_first() || !self.to_source() {
            self.update(target);
            self.update_source_binding(target);
        } else {
            self.update_source_binding(target);
            self.update(target);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data_bind_graph::{
        DATA_BIND_FLAG_DIRECTION_TO_SOURCE, DATA_BIND_FLAG_SOURCE_TO_TARGET_RUNS_FIRST,
        DATA_BIND_FLAG_TWO_WAY,
    };

    struct FakeTarget {
        value: RuntimeViewModelCellValue,
        applied: Vec<RuntimeViewModelCellValue>,
    }

    impl FakeTarget {
        fn number(value: f32) -> Self {
            Self {
                value: RuntimeViewModelCellValue::Number(value),
                applied: Vec::new(),
            }
        }
    }

    impl RuntimeDataBindTarget for FakeTarget {
        fn apply_to_target(&mut self, value: &RuntimeViewModelCellValue) {
            self.value = value.clone();
            self.applied.push(value.clone());
        }

        fn read_target(&mut self) -> Option<RuntimeViewModelCellValue> {
            Some(self.value.clone())
        }
    }

    struct ReentrantSourceTarget {
        source: RuntimeViewModelCell,
        replacement: RuntimeViewModelCellValue,
    }

    impl RuntimeDataBindTarget for ReentrantSourceTarget {
        fn apply_to_target(&mut self, _value: &RuntimeViewModelCellValue) {
            self.source.set_value(self.replacement.clone());
        }

        fn read_target(&mut self) -> Option<RuntimeViewModelCellValue> {
            None
        }
    }

    const TO_TARGET: u64 = 0; // default direction: source → target
    const TO_SOURCE: u64 = DATA_BIND_FLAG_DIRECTION_TO_SOURCE;
    const TWO_WAY: u64 = DATA_BIND_FLAG_TWO_WAY;

    #[test]
    fn source_write_dirties_and_applies_to_target_once() {
        let cell = RuntimeViewModelCell::new(RuntimeViewModelCellValue::Number(1.0));
        let mut bind = RuntimeRetainedDataBind::new(TO_TARGET, false);
        bind.set_source(cell.clone());
        let mut target = FakeTarget::number(0.0);

        cell.set_value(RuntimeViewModelCellValue::Number(5.0));
        bind.collect_source_dirt();
        assert!(bind.update(&mut target));
        assert_eq!(target.value, RuntimeViewModelCellValue::Number(5.0));

        // No further dirt: the pass consumed it, and applying did not
        // self-notify.
        bind.collect_source_dirt();
        assert!(!bind.update(&mut target));
        assert_eq!(target.applied.len(), 1);
    }

    #[test]
    fn binds_once_never_registers_a_dependent() {
        let cell = RuntimeViewModelCell::new(RuntimeViewModelCellValue::Number(1.0));
        let mut bind = RuntimeRetainedDataBind::new(TO_TARGET, true);
        bind.set_source(cell.clone());

        cell.set_value(RuntimeViewModelCellValue::Number(9.0));
        bind.collect_source_dirt();
        assert!(
            bind.pending_dirt().is_empty(),
            "bindsOnce receives no cascade"
        );
    }

    #[test]
    fn binds_once_still_observes_converter_operand_cells() {
        let primary = RuntimeViewModelCell::new(RuntimeViewModelCellValue::Number(1.0));
        let operand = RuntimeViewModelCell::new(RuntimeViewModelCellValue::Number(2.0));
        let mut bind = RuntimeRetainedDataBind::new(TO_TARGET, true);
        bind.set_source(primary.clone());
        bind.set_additional_sources(vec![operand.clone()]);

        primary.set_value(RuntimeViewModelCellValue::Number(9.0));
        bind.collect_source_dirt();
        assert!(
            bind.pending_dirt().is_empty(),
            "bindsOnce skips only the primary source registration"
        );

        operand.set_value(RuntimeViewModelCellValue::Number(7.0));
        bind.collect_source_dirt();
        assert!(
            bind.pending_dirt().contains(RuntimeCellDirt::BINDINGS),
            "C++ OperationViewModel registers its outer DataBind even when the primary bind is bindsOnce"
        );
    }

    #[test]
    fn binds_once_still_observes_an_operand_that_aliases_its_primary() {
        let cell = RuntimeViewModelCell::new(RuntimeViewModelCellValue::Number(1.0));
        let mut bind = RuntimeRetainedDataBind::new(TO_TARGET, true);
        bind.set_source(cell.clone());
        bind.set_additional_sources(vec![cell.clone()]);

        assert!(cell.set_value(RuntimeViewModelCellValue::Number(2.0)));
        assert_eq!(
            bind.take_source_dirt_with_primary(),
            Some(true),
            "C++ skips the bindsOnce primary DataBind edge but OperationViewModel still registers that same outer DataBind (`data_bind.cpp:210-216`, `data_converter_operation_viewmodel.cpp:48-59`)"
        );
    }

    #[test]
    fn drop_immediately_unregisters_primary_and_converter_sources() {
        let primary = RuntimeViewModelCell::new(RuntimeViewModelCellValue::Number(1.0));
        let operand = RuntimeViewModelCell::new(RuntimeViewModelCellValue::Number(2.0));
        {
            let mut bind = RuntimeRetainedDataBind::new(TO_TARGET, false);
            bind.set_source(primary.clone());
            bind.set_additional_sources(vec![operand.clone()]);
            assert_eq!(primary.dependent_count(), 1);
            assert_eq!(operand.dependent_count(), 1);
        }

        assert_eq!(
            primary.dependent_count(),
            0,
            "C++ ~DataBind removes its primary source edge during owner teardown"
        );
        assert_eq!(
            operand.dependent_count(),
            0,
            "converter-owned source edges are removed at the same teardown boundary"
        );
    }

    #[test]
    fn target_to_source_write_reaches_sibling_binds_without_echo() {
        let cell = RuntimeViewModelCell::new(RuntimeViewModelCellValue::Number(1.0));
        let mut writer = RuntimeRetainedDataBind::new(TO_SOURCE, false);
        let mut reader = RuntimeRetainedDataBind::new(TO_TARGET, false);
        writer.set_source(cell.clone());
        reader.set_source(cell.clone());

        let mut writer_target = FakeTarget::number(42.0);
        writer.mark_target_changed();
        assert!(writer.update_source_binding(&mut writer_target));
        assert_eq!(cell.value(), RuntimeViewModelCellValue::Number(42.0));

        // The sibling observed the shared-cell write; the writer did not
        // re-observe its own write.
        reader.collect_source_dirt();
        assert!(reader.pending_dirt().contains(RuntimeCellDirt::BINDINGS));
        writer.collect_source_dirt();
        assert!(
            !writer.pending_dirt().contains(RuntimeCellDirt::BINDINGS),
            "own write echo is swallowed (C++ suppressDirt)"
        );
    }

    #[test]
    fn target_to_source_write_queues_only_sibling_occurrences() {
        let cell = RuntimeViewModelCell::new(RuntimeViewModelCellValue::Number(1.0));
        let queue = RuntimeCellNotificationQueue::default();
        let mut writer = RuntimeRetainedDataBind::new(TO_SOURCE, false);
        let mut reader = RuntimeRetainedDataBind::new(TO_TARGET, false);
        writer.set_source(cell.clone());
        reader.set_source(cell);
        writer.report_source_dirt_to(&queue, 0);
        reader.report_source_dirt_to(&queue, 1);

        writer.mark_target_changed();
        let mut reporting = Vec::new();
        queue.swap_into(&mut reporting);
        assert_eq!(reporting, vec![0]);

        let mut writer_target = FakeTarget::number(42.0);
        assert!(writer.update_source_binding(&mut writer_target));
        queue.swap_into(&mut reporting);
        assert_eq!(
            reporting,
            vec![1],
            "C++ SuppressDirt prevents the writer's phantom queue entry while preserving its sibling observer"
        );
    }

    #[test]
    fn aliased_converter_operand_does_not_requeue_own_target_to_source_write() {
        let cell = RuntimeViewModelCell::new(RuntimeViewModelCellValue::Number(1.0));
        let mut bind = RuntimeRetainedDataBind::new(TO_SOURCE, false);
        bind.set_source(cell.clone());
        bind.set_additional_sources(vec![cell]);

        let mut target = FakeTarget::number(9.0);
        bind.mark_target_changed();
        assert!(bind.update_source_binding(&mut target));
        assert_eq!(bind.take_source_dirt_with_primary(), None);
    }

    #[test]
    fn target_to_source_write_preserves_pending_distinct_operand_dirt() {
        let primary = RuntimeViewModelCell::new(RuntimeViewModelCellValue::Number(1.0));
        let operand = RuntimeViewModelCell::new(RuntimeViewModelCellValue::Number(2.0));
        let mut bind = RuntimeRetainedDataBind::new(TO_SOURCE, false);
        bind.set_source(primary);
        bind.set_additional_sources(vec![operand.clone()]);

        assert!(operand.set_value(RuntimeViewModelCellValue::Number(3.0)));
        let mut target = FakeTarget::number(9.0);
        bind.mark_target_changed();
        assert!(bind.update_source_binding(&mut target));
        assert_eq!(
            bind.take_source_dirt_with_primary(),
            Some(false),
            "swallowing the primary self-echo must not discard independent operand work"
        );
    }

    #[test]
    fn primary_changes_recompute_converter_operand_alias_registration() {
        let first = RuntimeViewModelCell::new(RuntimeViewModelCellValue::Number(1.0));
        let second = RuntimeViewModelCell::new(RuntimeViewModelCellValue::Number(2.0));
        let mut bind = RuntimeRetainedDataBind::new(TO_TARGET, false);
        bind.set_source(first.clone());
        bind.set_additional_sources(vec![first.clone()]);

        bind.set_source(second.clone());
        assert!(first.set_value(RuntimeViewModelCellValue::Number(3.0)));
        assert_eq!(bind.take_source_dirt_with_primary(), Some(false));

        bind.set_additional_sources(vec![second.clone()]);
        bind.clear_source();
        assert!(second.set_value(RuntimeViewModelCellValue::Number(4.0)));
        assert_eq!(
            bind.take_source_dirt_with_primary(),
            Some(false),
            "clearing the primary must enroll its retained operand alias"
        );
    }

    #[test]
    fn reconcile_favors_the_declared_direction() {
        // Two-way, source→target runs first: the source value wins the
        // reconcile and overwrites the target.
        let cell = RuntimeViewModelCell::new(RuntimeViewModelCellValue::Number(10.0));
        let mut bind = RuntimeRetainedDataBind::new(
            TWO_WAY | DATA_BIND_FLAG_SOURCE_TO_TARGET_RUNS_FIRST,
            false,
        );
        bind.set_source(cell.clone());
        let mut target = FakeTarget::number(99.0);
        bind.mark_rebind_reconcile();
        assert!(!bind.target_origin(), "favored source side wins the origin");
        bind.reconcile(&mut target);
        assert_eq!(target.value, RuntimeViewModelCellValue::Number(10.0));
        assert_eq!(cell.value(), RuntimeViewModelCellValue::Number(10.0));

        // Two-way, target favored: the serialized target value seeds the
        // source — the C++ init ordering the flattened port lost (the
        // instance-0 scroll scalar class of bug).
        let cell = RuntimeViewModelCell::new(RuntimeViewModelCellValue::Number(4022.0));
        let mut bind = RuntimeRetainedDataBind::new(TWO_WAY, false);
        bind.set_source(cell.clone());
        let mut target = FakeTarget::number(271.0);
        bind.mark_rebind_reconcile();
        assert!(bind.target_origin(), "favored target side wins the origin");
        bind.reconcile(&mut target);
        assert_eq!(
            cell.value(),
            RuntimeViewModelCellValue::Number(271.0),
            "target seeds the source before any source→target read"
        );
        assert_eq!(target.value, RuntimeViewModelCellValue::Number(271.0));
    }

    #[test]
    fn one_sided_changes_latch_their_own_origin() {
        let cell = RuntimeViewModelCell::new(RuntimeViewModelCellValue::Number(0.0));
        let mut bind = RuntimeRetainedDataBind::new(TWO_WAY, false);
        bind.set_source(cell.clone());

        cell.set_value(RuntimeViewModelCellValue::Number(1.0));
        bind.collect_source_dirt();
        assert!(!bind.target_origin());

        bind.mark_target_changed();
        assert!(bind.target_origin());

        // C++ returns before changing the origin when the same dirt bit is
        // already pending (`data_bind.cpp:502-507`).
        bind.mark_source_changed();
        assert!(bind.target_origin());
    }

    #[test]
    fn completed_target_first_reconcile_allows_a_fresh_source_origin() {
        let cell = RuntimeViewModelCell::new(RuntimeViewModelCellValue::Number(0.0));
        let mut bind = RuntimeRetainedDataBind::new(TWO_WAY, false);
        bind.set_source(cell.clone());
        bind.mark_rebind_reconcile();
        assert!(
            bind.target_origin(),
            "target-first reconcile starts at target"
        );

        assert!(bind.take_target_dirt());
        assert!(bind.take_pending_source_dirt());
        assert!(bind.pending_dirt().is_empty());

        assert!(cell.set_value(RuntimeViewModelCellValue::Number(1.0)));
        assert!(bind.take_source_dirt());
        assert!(
            !bind.target_origin(),
            "after processed dirt is cleared, a genuine source notification latches source origin"
        );
    }

    #[test]
    fn converter_owned_source_reasserts_the_settled_parent_origin() {
        let converter_source = RuntimeViewModelCell::new(RuntimeViewModelCellValue::Number(1.0));
        let mut target_origin = RuntimeRetainedDataBind::new(TWO_WAY, false);
        target_origin.mark_rebind_reconcile();
        assert!(target_origin.take_target_dirt());
        assert!(target_origin.take_pending_source_dirt());
        assert!(target_origin.target_origin());
        let mut inner = RuntimeRetainedDataBind::new(TO_TARGET, false);
        inner.set_container_wake(Some(target_origin.converter_parent_wake()));
        inner.set_source(converter_source.clone());

        assert!(converter_source.set_value(RuntimeViewModelCellValue::Number(2.0)));
        assert!(
            inner.collect_source_dirt(),
            "the source dirt belongs to the converter-owned inner DataBind"
        );
        assert!(
            target_origin.target_origin(),
            "DataConverter::markConverterDirty preserves the outer TargetOrigin flag"
        );
        assert!(
            target_origin
                .pending_dirt()
                .contains(RuntimeCellDirt::BINDINGS_TARGET)
        );
        assert!(
            !target_origin
                .pending_dirt()
                .contains(RuntimeCellDirt::BINDINGS)
        );

        let source = RuntimeViewModelCell::new(RuntimeViewModelCellValue::Number(3.0));
        let mut source_origin = RuntimeRetainedDataBind::new(
            TWO_WAY | DATA_BIND_FLAG_SOURCE_TO_TARGET_RUNS_FIRST,
            false,
        );
        source_origin.mark_rebind_reconcile();
        assert!(source_origin.take_pending_source_dirt());
        assert!(source_origin.take_target_dirt());
        assert!(!source_origin.target_origin());
        let mut source_inner = RuntimeRetainedDataBind::new(TO_TARGET, false);
        source_inner.set_container_wake(Some(source_origin.converter_parent_wake()));
        source_inner.set_source(source.clone());

        assert!(source.set_value(RuntimeViewModelCellValue::Number(4.0)));
        assert!(source_inner.collect_source_dirt());
        assert!(!source_origin.target_origin());
        assert!(
            source_origin
                .pending_dirt()
                .contains(RuntimeCellDirt::BINDINGS)
        );
        assert!(
            !source_origin
                .pending_dirt()
                .contains(RuntimeCellDirt::BINDINGS_TARGET)
        );
    }

    #[test]
    fn direct_converter_operand_remains_a_source_origin_control() {
        let operand = RuntimeViewModelCell::new(RuntimeViewModelCellValue::Number(1.0));
        let mut bind = RuntimeRetainedDataBind::new(TWO_WAY, false);
        bind.mark_rebind_reconcile();
        assert!(bind.take_target_dirt());
        assert!(bind.take_pending_source_dirt());
        assert!(bind.target_origin());
        bind.set_additional_sources(vec![operand.clone()]);

        assert!(operand.set_value(RuntimeViewModelCellValue::Number(2.0)));
        assert!(bind.collect_source_dirt());
        assert!(
            !bind.target_origin(),
            "OperationViewModel registers the outer DataBind directly, so its operand is a new source-origin notification"
        );
        assert!(bind.pending_dirt().contains(RuntimeCellDirt::BINDINGS));
    }

    #[test]
    fn converter_wake_reports_the_exact_idle_parent_occurrence() {
        let queue = RuntimeCellNotificationQueue::default();
        let mut bind = RuntimeRetainedDataBind::new(TWO_WAY, false);
        bind.report_source_dirt_to(&queue, 7);
        bind.mark_rebind_reconcile();
        let mut reported = Vec::new();
        queue.swap_into(&mut reported);
        assert_eq!(reported, vec![7]);
        assert!(bind.take_target_dirt());
        assert!(bind.take_pending_source_dirt());

        bind.converter_parent_wake().mark_converter_changed();
        queue.swap_into(&mut reported);
        assert_eq!(reported, vec![7]);
        assert!(bind.target_origin());
        assert!(
            bind.pending_dirt()
                .contains(RuntimeCellDirt::BINDINGS_TARGET)
        );
    }

    #[test]
    fn converter_owned_source_rejects_reentrant_dirt_while_applying() {
        let source = RuntimeViewModelCell::new(RuntimeViewModelCellValue::Number(1.0));
        let outer = RuntimeRetainedDataBind::new(TO_TARGET, false);
        let mut inner = RuntimeRetainedDataBind::new(TO_TARGET, false);
        inner.set_container_wake(Some(outer.converter_parent_wake()));
        inner.set_source(source.clone());
        inner.wake_state.dirt.set(RuntimeCellDirt::BINDINGS);

        let mut target = ReentrantSourceTarget {
            source,
            replacement: RuntimeViewModelCellValue::Number(2.0),
        };
        assert!(inner.update(&mut target));
        assert!(
            !inner.collect_source_dirt(),
            "DataBind::SuppressDirt rejects the reentrant source cascade instead of retaining it in the sink"
        );
        assert!(inner.pending_dirt().is_empty());
        assert!(
            outer.pending_dirt().is_empty(),
            "a rejected inner DataBind cascade cannot wake its parent converter occurrence"
        );
    }

    #[test]
    fn converter_owned_source_coalesces_while_inner_dirt_is_pending() {
        let source = RuntimeViewModelCell::new(RuntimeViewModelCellValue::Number(1.0));
        let queue = RuntimeCellNotificationQueue::default();
        let mut outer = RuntimeRetainedDataBind::new(TO_TARGET, false);
        outer.report_source_dirt_to(&queue, 0);
        let mut inner = RuntimeRetainedDataBind::new(TO_TARGET, false);
        inner.set_container_wake(Some(outer.converter_parent_wake()));
        inner.report_source_dirt_to(&queue, 1);
        inner.set_source(source.clone());

        assert!(source.set_value(RuntimeViewModelCellValue::Number(2.0)));
        let mut reported = Vec::new();
        queue.swap_into(&mut reported);
        assert_eq!(
            reported,
            vec![0, 1],
            "C++ wakes the parent DataBind before the inner converter-owned DataBind"
        );
        assert!(inner.collect_source_dirt());
        assert!(inner.pending_dirt().contains(RuntimeCellDirt::BINDINGS));

        assert!(source.set_value(RuntimeViewModelCellValue::Number(3.0)));
        queue.swap_into(&mut reported);
        assert!(
            reported.is_empty(),
            "DataBind::addDirt rejects a duplicate while the same bit is pending"
        );
        assert!(
            !inner.collect_source_dirt(),
            "the rejected duplicate is not retained in the source sink for a later redundant pass"
        );
    }

    #[test]
    fn primary_and_converter_operand_dirt_remain_distinguishable() {
        let primary = RuntimeViewModelCell::new(RuntimeViewModelCellValue::Number(1.0));
        let operand = RuntimeViewModelCell::new(RuntimeViewModelCellValue::Number(2.0));
        let mut bind = RuntimeRetainedDataBind::new(TO_TARGET, false);
        bind.set_source(primary.clone());
        bind.set_additional_sources(vec![operand.clone()]);

        assert!(operand.set_value(RuntimeViewModelCellValue::Number(3.0)));
        assert_eq!(
            bind.take_source_dirt_with_primary(),
            Some(false),
            "OperationViewModel operands dirty the outer bind without imitating Formula's primary-source subscription"
        );

        assert!(primary.set_value(RuntimeViewModelCellValue::Number(4.0)));
        assert_eq!(
            bind.take_source_dirt_with_primary(),
            Some(true),
            "the exact primary source remains visible for source-change formula invalidation"
        );
    }

    #[test]
    fn clear_source_stops_observation_and_drops_stale_dirt() {
        let cell = RuntimeViewModelCell::new(RuntimeViewModelCellValue::Number(0.0));
        let mut bind = RuntimeRetainedDataBind::new(TO_TARGET, false);
        bind.set_source(cell.clone());
        cell.set_value(RuntimeViewModelCellValue::Number(2.0));
        bind.clear_source();
        bind.collect_source_dirt();
        assert!(bind.pending_dirt().is_empty());

        cell.set_value(RuntimeViewModelCellValue::Number(3.0));
        bind.collect_source_dirt();
        assert!(
            bind.pending_dirt().is_empty(),
            "an unbound bind observes nothing"
        );
    }
}
