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
    /// above is only an origin discriminator; both sinks fold into this same
    /// bind's `ComponentDirt::Bindings` latch and occurrence queue.
    additional_sources: Vec<RuntimeViewModelCell>,
    /// C++ `Flag::TargetOrigin`: which side the latched dirt came from.
    target_origin: bool,
    /// C++ `Flag::SuppressDirt`.
    suppress_dirt: bool,
    /// Pending dirt latched from the sink plus reconcile marks
    /// (C++ `m_Dirt`).
    dirt: RuntimeCellDirt,
}

impl std::fmt::Debug for RuntimeRetainedDataBind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RuntimeRetainedDataBind")
            .field("flags", &self.flags)
            .field("binds_once", &self.binds_once)
            .field("target_origin", &self.target_origin)
            .field("dirt", &self.dirt)
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
        cloned.target_origin = self.target_origin;
        cloned.dirt = self.dirt;
        cloned.sink.add_dirt(primary_dirt);
        cloned.additional_sink.add_dirt(additional_dirt);
        cloned
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
            target_origin: false,
            suppress_dirt: false,
            dirt: RuntimeCellDirt::NONE,
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
        self.register_primary_source();
        self.register_additional_sources();
        if !primary_dirt.is_empty() {
            self.sink.add_dirt(primary_dirt);
            queue.report_data_bind(data_bind_index);
        }
        if !additional_dirt.is_empty() {
            self.additional_sink.add_dirt(additional_dirt);
            queue.report_data_bind(data_bind_index);
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

    /// C++ `DataBind::addDirt` with the origin latch: a reconcile (both
    /// bits) resolves the origin by favored direction; a one-sided change
    /// records its own side. Suppressed applies never re-dirty.
    fn add_dirt(&mut self, dirt: RuntimeCellDirt) {
        if self.suppress_dirt || dirt.is_empty() || self.dirt.contains(dirt) {
            return;
        }
        let has_source = dirt.contains(RuntimeCellDirt::BINDINGS);
        let has_target = dirt.contains(RuntimeCellDirt::BINDINGS_TARGET);
        if has_source && has_target {
            self.target_origin = !self.source_to_target_runs_first();
        } else if has_target {
            self.target_origin = true;
        } else if has_source {
            self.target_origin = false;
        }
        self.dirt.insert(dirt);
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
        self.dirt = if self.dirt.contains(RuntimeCellDirt::BINDINGS_TARGET) {
            RuntimeCellDirt::BINDINGS_TARGET
        } else {
            RuntimeCellDirt::NONE
        };
        Some(primary)
    }

    /// Consume the concrete target adapter's queued notification. This is
    /// the authored-bind scheduler counterpart to [`Self::take_source_dirt`]
    /// and preserves any simultaneously pending source reconcile bit.
    pub(crate) fn take_target_dirt(&mut self) -> bool {
        if !self.dirt.contains(RuntimeCellDirt::BINDINGS_TARGET) {
            return false;
        }
        self.dirt = if self.dirt.contains(RuntimeCellDirt::BINDINGS) {
            RuntimeCellDirt::BINDINGS
        } else {
            RuntimeCellDirt::NONE
        };
        true
    }

    /// Consume a source bit whose concrete facade adapter has just applied.
    /// C++ removes the queued dirt before calling `DataBind::update`, so a
    /// later notification can latch a fresh origin (`data_bind_container.cpp:
    /// 144-147`, `data_bind.cpp:502-531`).
    pub(crate) fn take_pending_source_dirt(&mut self) -> bool {
        if !self.dirt.contains(RuntimeCellDirt::BINDINGS) {
            return false;
        }
        self.dirt = if self.dirt.contains(RuntimeCellDirt::BINDINGS_TARGET) {
            RuntimeCellDirt::BINDINGS_TARGET
        } else {
            RuntimeCellDirt::NONE
        };
        true
    }

    pub fn pending_dirt(&self) -> RuntimeCellDirt {
        self.dirt
    }

    pub fn target_origin(&self) -> bool {
        self.target_origin
    }

    /// C++ `DataBind::update(ComponentDirt::Bindings)`: apply source→target
    /// under dirt, self-notification suppressed. Consumes the source bit.
    pub fn update(&mut self, target: &mut dyn RuntimeDataBindTarget) -> bool {
        if !self.dirt.contains(RuntimeCellDirt::BINDINGS) {
            return false;
        }
        self.dirt = if self.dirt.contains(RuntimeCellDirt::BINDINGS_TARGET) {
            RuntimeCellDirt::BINDINGS_TARGET
        } else {
            RuntimeCellDirt::NONE
        };
        let Some(source) = self.source.as_ref() else {
            return false;
        };
        if !self.to_target() {
            return false;
        }
        let value = source.value();
        self.suppress_dirt = true;
        target.apply_to_target(&value);
        self.suppress_dirt = false;
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
        self.dirt = if self.dirt.contains(RuntimeCellDirt::BINDINGS) {
            RuntimeCellDirt::BINDINGS
        } else {
            RuntimeCellDirt::NONE
        };
        let Some(source) = self.source.as_ref() else {
            return false;
        };
        let Some(value) = target.read_target() else {
            return false;
        };
        let changed = source.set_value(value);
        // The cell cascade dirtied our own sink too; swallow the echo.
        self.sink.take_dirt();
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
