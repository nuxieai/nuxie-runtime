// Mirrors src/animation/linear_animation_instance.cpp and include/rive/animation/loop.hpp.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum AnimationLoop {
    OneShot,
    Loop,
    PingPong,
}

impl AnimationLoop {
    pub(crate) fn from_loop_value(value: i32) -> Self {
        match value {
            1 => Self::Loop,
            2 => Self::PingPong,
            _ => Self::OneShot,
        }
    }
}

fn positive_mod(value: f32, range: f32) -> f32 {
    ((value % range) + range) % range
}

/// Stable typed identity for one definition in an Artboard's immutable
/// LinearAnimation arena. C++ occurrences retain `const LinearAnimation*`;
/// Rust retains this non-dereferenceable handle and resolves it only through
/// the owning Artboard arena.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RuntimeLinearAnimationHandle(Option<usize>);

impl RuntimeLinearAnimationHandle {
    pub(crate) fn new(index: usize) -> Self {
        Self(Some(index))
    }

    pub(crate) fn empty() -> Self {
        Self(None)
    }

    pub(crate) fn resolve<'a>(
        self,
        definitions: &'a [RuntimeLinearAnimation],
        empty: &'a RuntimeLinearAnimation,
    ) -> Option<&'a RuntimeLinearAnimation> {
        match self.0 {
            Some(index) => definitions.get(index),
            None => Some(empty),
        }
    }

    pub fn index(self) -> usize {
        self.0.unwrap_or(usize::MAX)
    }

    pub(crate) fn definition_index(self) -> Option<usize> {
        self.0
    }
}

#[derive(Debug)]
pub struct LinearAnimationInstance {
    animation: RuntimeLinearAnimationHandle,
    animation_definitions: Arc<Vec<RuntimeLinearAnimation>>,
    empty_animation_definition: Arc<RuntimeLinearAnimation>,
    pub(crate) time: f32,
    pub(crate) speed_direction: f32,
    pub(crate) total_time: f32,
    pub(crate) last_total_time: f32,
    pub(crate) spilled_time: f32,
    pub(crate) direction: f32,
    pub(crate) did_loop: bool,
    /// C++ `m_loopValue`: `-1` means use the definition value.
    pub(crate) loop_value_override: i32,
    /// Live keyframe DataBind clones in C++ build order. The reusable
    /// StateMachine-level graph is only a prototype: every call corresponding
    /// to `buildStateKeyFrameBinds` appends a fresh graph with independent
    /// converter/random state. Keeping these occurrences before the holders
    /// also makes Rust drop the binds before their targets.
    key_frame_data_bind_graphs: Vec<RuntimeDataBindGraph>,
    key_frame_data_bind_occurrences: Vec<(
        Option<RuntimeKeyFrameDataBindOccurrenceId>,
        RuntimeKeyFrameDataBindEnrollment,
    )>,
    key_frame_rebuild_enrollment: RuntimeKeyFrameDataBindEnrollment,
    key_frame_value_holders: Option<Box<HashMap<u32, RuntimeKeyFrameValue>>>,
    key_frame_prototype_revision: u64,
    scripted_interpolators: RefCell<RuntimeScriptedInterpolatorState>,
    // Pinned advanceAndApply creates lazy interpolator clones during apply,
    // then advances their Artboard-owned binds. Retain elapsed time across
    // Rust's split advance/apply API to preserve that ordering.
    pending_scripted_bind_advance: Cell<Option<f32>>,
    #[cfg(test)]
    removed_key_frame_data_bind_occurrences: Vec<RuntimeKeyFrameDataBindOccurrenceId>,
}

impl Clone for LinearAnimationInstance {
    fn clone(&self) -> Self {
        Self {
            animation: self.animation,
            animation_definitions: Arc::clone(&self.animation_definitions),
            empty_animation_definition: Arc::clone(&self.empty_animation_definition),
            time: self.time,
            speed_direction: self.speed_direction,
            total_time: self.total_time,
            last_total_time: self.last_total_time,
            spilled_time: self.spilled_time,
            direction: self.direction,
            did_loop: self.did_loop,
            loop_value_override: self.loop_value_override,
            // Keyframe holders model C++'s per-LAI runtime-owned bind targets.
            // A copied LAI starts unbound; state transitions move the outgoing
            // instance when they need to preserve its concrete binding identity.
            key_frame_data_bind_graphs: Vec::new(),
            key_frame_data_bind_occurrences: Vec::new(),
            key_frame_rebuild_enrollment: self
                .key_frame_data_bind_occurrences
                .first()
                .map(|(_, enrollment)| *enrollment)
                .unwrap_or(self.key_frame_rebuild_enrollment),
            key_frame_value_holders: None,
            key_frame_prototype_revision: 0,
            // C++ does not copy `m_StatefulInterpolators`: a copied LAI lazily
            // clones fresh ScriptedInterpolator tables for its own keyframes.
            scripted_interpolators: RefCell::new(RuntimeScriptedInterpolatorState::default()),
            pending_scripted_bind_advance: Cell::new(None),
            #[cfg(test)]
            removed_key_frame_data_bind_occurrences: Vec::new(),
        }
    }
}

impl LinearAnimationInstance {
    /// Reinitialize the private occurrence owned by a cloned
    /// `NestedSimpleAnimation`. Pinned C++ clones the generated nested object,
    /// then `NestedLinearAnimation::initializeAnimation` constructs a fresh
    /// `LinearAnimationInstance` against the cloned child artboard.
    pub(crate) fn cold_clone_for_nested_animation(&self) -> Self {
        Self::new(
            self.animation,
            Arc::clone(&self.animation_definitions),
            Arc::clone(&self.empty_animation_definition),
            1.0,
        )
        .expect("a live nested animation retains a resolvable definition")
    }

    pub(crate) fn new(
        animation: RuntimeLinearAnimationHandle,
        animation_definitions: Arc<Vec<RuntimeLinearAnimation>>,
        empty_animation_definition: Arc<RuntimeLinearAnimation>,
        speed_multiplier: f32,
    ) -> Option<Self> {
        let definition = animation.resolve(&animation_definitions, &empty_animation_definition)?;
        let time = definition.start_time_with_speed(speed_multiplier);
        Some(Self {
            animation,
            animation_definitions,
            empty_animation_definition,
            time,
            speed_direction: if speed_multiplier >= 0.0 { 1.0 } else { -1.0 },
            total_time: 0.0,
            last_total_time: 0.0,
            spilled_time: 0.0,
            direction: 1.0,
            did_loop: false,
            loop_value_override: -1,
            key_frame_data_bind_graphs: Vec::new(),
            key_frame_data_bind_occurrences: Vec::new(),
            key_frame_rebuild_enrollment: RuntimeKeyFrameDataBindEnrollment::Late,
            key_frame_value_holders: None,
            key_frame_prototype_revision: 0,
            scripted_interpolators: RefCell::new(RuntimeScriptedInterpolatorState::default()),
            pending_scripted_bind_advance: Cell::new(None),
            #[cfg(test)]
            removed_key_frame_data_bind_occurrences: Vec::new(),
        })
    }

    #[cfg(test)]
    pub(crate) fn new_for_test(
        animation: RuntimeLinearAnimationHandle,
        definition: &RuntimeLinearAnimation,
        speed_multiplier: f32,
    ) -> Self {
        let (animation_definitions, empty_animation_definition) =
            if let Some(definition_index) = animation.definition_index() {
                let mut definitions = (0..=definition_index)
                    .map(|_| RuntimeLinearAnimation::empty())
                    .collect::<Vec<_>>();
                definitions[definition_index] = definition.clone();
                (
                    Arc::new(definitions),
                    Arc::new(RuntimeLinearAnimation::empty()),
                )
            } else {
                (Arc::new(Vec::new()), Arc::new(definition.clone()))
            };
        Self::new(
            animation,
            animation_definitions,
            empty_animation_definition,
            speed_multiplier,
        )
        .expect("test animation definition is inserted at its retained handle")
    }

    pub(crate) fn build_key_frame_data_binds(
        &mut self,
        prototype: &RuntimeDataBindGraph,
        enrollment: RuntimeKeyFrameDataBindEnrollment,
    ) -> bool {
        self.build_key_frame_data_binds_internal(prototype, enrollment, true)
    }

    fn build_key_frame_data_binds_internal(
        &mut self,
        prototype: &RuntimeDataBindGraph,
        enrollment: RuntimeKeyFrameDataBindEnrollment,
        apply_immediately: bool,
    ) -> bool {
        if self.key_frame_data_bind_graphs.is_empty() {
            self.key_frame_rebuild_enrollment = enrollment;
        }
        // Pinned C++ creates each typed holder before cloning the matching
        // DataBind. A duplicate build overwrites the holder lookup but retains
        // both bind clones in build order
        // (`state_machine_instance.cpp:3338-3369`).
        for target in &prototype.targets {
            let (global_id, value) = match target.target {
                RuntimeDataBindGraphTarget::KeyFrameNumber { global_id } => {
                    (global_id, RuntimeKeyFrameValue::Number(0.0))
                }
                RuntimeDataBindGraphTarget::KeyFrameColor { global_id } => {
                    (global_id, RuntimeKeyFrameValue::Color(0xFF1D1D1D))
                }
                RuntimeDataBindGraphTarget::KeyFrameBoolean { global_id } => {
                    (global_id, RuntimeKeyFrameValue::Boolean(false))
                }
                RuntimeDataBindGraphTarget::KeyFrameString { global_id } => {
                    (global_id, RuntimeKeyFrameValue::String(Vec::new()))
                }
                _ => continue,
            };
            self.add_key_frame_value_holder(global_id, value);
        }
        self.key_frame_data_bind_graphs
            .push(prototype.clone_for_key_frame_instance());
        self.key_frame_data_bind_occurrences
            .push((None, enrollment));
        self.key_frame_prototype_revision = prototype.key_frame_source_revision();

        if !apply_immediately {
            return false;
        }
        let updates = self
            .key_frame_data_bind_graphs
            .last_mut()
            .map(|graph| {
                graph.take_key_frame_binding_updates(
                    RuntimeDataBindGraphApplyPhase::BeforeStatefulAdvance,
                )
            })
            .unwrap_or_default();
        self.apply_key_frame_data_bind_updates(updates)
    }

    pub(crate) fn ensure_key_frame_data_binds(&mut self, prototype: &RuntimeDataBindGraph) {
        if self.key_frame_data_bind_graphs.is_empty() {
            self.build_key_frame_data_binds_internal(
                prototype,
                self.key_frame_rebuild_enrollment,
                false,
            );
        }
    }

    pub(crate) fn key_frame_data_bind_occurrence_ids(
        &self,
        enrollment: RuntimeKeyFrameDataBindEnrollment,
    ) -> impl Iterator<Item = RuntimeKeyFrameDataBindOccurrenceId> + '_ {
        self.key_frame_data_bind_occurrences
            .iter()
            .filter_map(move |(id, candidate)| (*candidate == enrollment).then_some(*id).flatten())
    }

    pub(crate) fn enroll_unassigned_key_frame_data_binds(&mut self, next_id: &mut u64) {
        for (id, _) in &mut self.key_frame_data_bind_occurrences {
            if id.is_some() {
                continue;
            }
            *id = Some(RuntimeKeyFrameDataBindOccurrenceId(*next_id));
            *next_id = next_id.wrapping_add(1);
        }
    }

    pub(crate) fn prepare_key_frame_data_bind_occurrence(
        &mut self,
        occurrence_id: RuntimeKeyFrameDataBindOccurrenceId,
        prototype: &RuntimeDataBindGraph,
    ) -> Option<bool> {
        self.sync_key_frame_data_bind_graph(prototype);
        let position = self
            .key_frame_data_bind_occurrences
            .iter()
            .position(|(id, _)| *id == Some(occurrence_id))?;
        let updates = self
            .key_frame_data_bind_graphs
            .get_mut(position)?
            .take_key_frame_binding_updates(RuntimeDataBindGraphApplyPhase::BeforeStatefulAdvance);
        Some(self.apply_key_frame_data_bind_updates(updates))
    }

    pub(crate) fn advance_key_frame_data_bind_occurrence(
        &mut self,
        occurrence_id: RuntimeKeyFrameDataBindOccurrenceId,
        prototype: &RuntimeDataBindGraph,
        elapsed_seconds: f32,
    ) -> Option<bool> {
        self.sync_key_frame_data_bind_graph(prototype);
        let position = self
            .key_frame_data_bind_occurrences
            .iter()
            .position(|(id, _)| *id == Some(occurrence_id))?;
        let advance = self
            .key_frame_data_bind_graphs
            .get_mut(position)?
            .advance_stateful_converters(elapsed_seconds);
        Some(advance.changed || advance.keep_going)
    }

    fn sync_key_frame_data_bind_graph(&mut self, prototype: &RuntimeDataBindGraph) {
        if self.key_frame_prototype_revision == prototype.key_frame_source_revision() {
            return;
        }
        for graph in &mut self.key_frame_data_bind_graphs {
            graph.sync_key_frame_sources_from(prototype);
        }
        self.key_frame_prototype_revision = prototype.key_frame_source_revision();
    }

    fn apply_key_frame_data_bind_updates(
        &mut self,
        updates: Vec<(RuntimeDataBindGraphTarget, crate::RuntimeDataBindGraphValue)>,
    ) -> bool {
        let mut changed = false;
        for (target, value) in updates {
            let (global_id, value) = match (target, value) {
                (
                    RuntimeDataBindGraphTarget::KeyFrameNumber { global_id },
                    crate::RuntimeDataBindGraphValue::Number(value),
                ) => (global_id, RuntimeKeyFrameValue::Number(value)),
                (
                    RuntimeDataBindGraphTarget::KeyFrameColor { global_id },
                    crate::RuntimeDataBindGraphValue::Color(value),
                ) => (global_id, RuntimeKeyFrameValue::Color(value)),
                (
                    RuntimeDataBindGraphTarget::KeyFrameBoolean { global_id },
                    crate::RuntimeDataBindGraphValue::Boolean(value),
                ) => (global_id, RuntimeKeyFrameValue::Boolean(value)),
                (
                    RuntimeDataBindGraphTarget::KeyFrameString { global_id },
                    crate::RuntimeDataBindGraphValue::String(value),
                ) => (global_id, RuntimeKeyFrameValue::String(value)),
                _ => continue,
            };
            let Some(holder) = self.key_frame_value_holder_mut(global_id) else {
                continue;
            };
            if *holder != value {
                *holder = value;
                changed = true;
            }
        }
        changed
    }

    pub(crate) fn prepare_key_frame_data_binds(
        &mut self,
        prototype: Option<&RuntimeDataBindGraph>,
    ) -> bool {
        let Some(prototype) = prototype else {
            return false;
        };
        if self.key_frame_data_bind_graphs.is_empty() {
            return self.build_key_frame_data_binds(prototype, self.key_frame_rebuild_enrollment);
        }
        self.sync_key_frame_data_bind_graph(prototype);
        let mut updates = Vec::new();
        for graph in &mut self.key_frame_data_bind_graphs {
            updates.extend(graph.take_key_frame_binding_updates(
                RuntimeDataBindGraphApplyPhase::BeforeStatefulAdvance,
            ));
        }
        self.apply_key_frame_data_bind_updates(updates)
    }

    pub(crate) fn advance_key_frame_data_binds(
        &mut self,
        prototype: Option<&RuntimeDataBindGraph>,
        elapsed_seconds: f32,
    ) -> bool {
        let Some(prototype) = prototype else {
            return false;
        };
        self.sync_key_frame_data_bind_graph(prototype);
        let mut keep_going = false;
        let mut changed = false;
        for graph in &mut self.key_frame_data_bind_graphs {
            let advance = graph.advance_stateful_converters(elapsed_seconds);
            changed |= advance.changed;
            keep_going |= advance.keep_going;
        }
        // C++ advances converters after every layer, but the resulting dirt is
        // consumed by the next frame's normal updateDataBinds(false) pass.
        // Do not apply an AfterStatefulAdvance update here.
        changed || keep_going
    }

    pub(crate) fn add_key_frame_value_holder(
        &mut self,
        key_frame_global_id: u32,
        value: RuntimeKeyFrameValue,
    ) {
        self.key_frame_value_holders
            .get_or_insert_with(|| Box::new(HashMap::new()))
            .insert(key_frame_global_id, value);
    }

    pub(crate) fn remove_key_frame_data_binds(&mut self) {
        // Drain and drop each occurrence explicitly because `Vec` does not
        // guarantee element drop order. Holders are released only afterward,
        // matching `removeStateKeyFrameBinds` and making repeated/unknown
        // removal a no-op in the Rust ownership model.
        for ((occurrence_id, _), graph) in self
            .key_frame_data_bind_occurrences
            .drain(..)
            .zip(self.key_frame_data_bind_graphs.drain(..))
        {
            #[cfg(test)]
            if let Some(occurrence_id) = occurrence_id {
                self.removed_key_frame_data_bind_occurrences
                    .push(occurrence_id);
            }
            drop(graph);
        }
        self.key_frame_value_holders = None;
        self.key_frame_prototype_revision = 0;
    }

    pub(crate) fn key_frame_value_holder(
        &self,
        key_frame_global_id: u32,
    ) -> Option<&RuntimeKeyFrameValue> {
        self.key_frame_value_holders
            .as_deref()?
            .get(&key_frame_global_id)
    }

    pub(crate) fn key_frame_value_holder_mut(
        &mut self,
        key_frame_global_id: u32,
    ) -> Option<&mut RuntimeKeyFrameValue> {
        self.key_frame_value_holders
            .as_deref_mut()?
            .get_mut(&key_frame_global_id)
    }

    fn key_frame_value_context(&self) -> RuntimeKeyFrameValueContext<'_> {
        RuntimeKeyFrameValueContext {
            holders: self.key_frame_value_holders.as_deref(),
        }
    }

    /// Apply the retained definition to the caller's mutable Artboard.
    ///
    /// C++ retains both `m_animation` and `m_artboard`. Rust cannot retain a
    /// mutable Artboard borrow for the occurrence lifetime, so the caller
    /// supplies only the application target. Definition lookup must still use
    /// this instance's retained immutable arena.
    pub(crate) fn apply(&self, artboard: &mut ArtboardInstance, mix: f32) -> bool {
        if self.animation.definition_index().is_none() {
            // C++'s shared empty animation owns no KeyedObjects.
            return false;
        }
        let Some(definition) = self.retained_definition() else {
            return false;
        };
        let changed = definition.apply_with_key_frame_values(
            artboard,
            self.time,
            mix,
            self.key_frame_value_context(),
            Some(self),
        );
        let scripted_bind_more = self
            .pending_scripted_bind_advance
            .take()
            .is_some_and(|elapsed_seconds| {
                self.scripted_interpolators
                    .borrow_mut()
                    .advance_stateful_converters(elapsed_seconds)
            });
        changed || scripted_bind_more
    }

    fn evaluate_scripted_interpolator(
        &self,
        artboard: &ArtboardInstance,
        key_frame_global_id: u32,
        interpolator_global_id: u32,
        method: ScriptInterpolatorMethod,
        arguments: &[f32],
        fallback: f32,
    ) -> f32 {
        let factory = artboard.scripted_interpolator_factory(interpolator_global_id);
        self.scripted_interpolators.borrow_mut().evaluate(
            Some(artboard),
            factory.as_ref(),
            key_frame_global_id,
            key_frame_global_id,
            interpolator_global_id,
            method,
            arguments,
            fallback,
        )
    }

    /// Script initialization/callback failures that fell back during apply.
    pub fn scripted_interpolator_diagnostics(&self) -> Vec<RuntimeScriptedInterpolatorDiagnostic> {
        self.scripted_interpolators.borrow().diagnostics()
    }

    pub fn animation_index(&self) -> usize {
        self.animation.index()
    }

    pub(crate) fn animation_handle(&self) -> RuntimeLinearAnimationHandle {
        self.animation
    }

    pub fn time(&self) -> f32 {
        self.time
    }

    pub fn speed_direction(&self) -> f32 {
        self.speed_direction
    }

    pub fn total_time(&self) -> f32 {
        self.total_time
    }

    pub fn last_total_time(&self) -> f32 {
        self.last_total_time
    }

    pub fn spilled_time(&self) -> f32 {
        self.spilled_time
    }

    pub fn direction(&self) -> f32 {
        self.direction
    }

    pub fn set_direction(&mut self, direction: i32) {
        self.direction = if direction > 0 { 1.0 } else { -1.0 };
    }

    pub fn did_loop(&self) -> bool {
        self.did_loop
    }

    pub fn clear_spilled_time(&mut self) {
        self.spilled_time = 0.0;
    }

    pub(crate) fn retained_definition(&self) -> Option<&RuntimeLinearAnimation> {
        self.animation.resolve(
            &self.animation_definitions,
            &self.empty_animation_definition,
        )
    }

    pub(crate) fn set_time_from_retained_definition(&mut self, seconds: f32) {
        let definitions = Arc::clone(&self.animation_definitions);
        let empty = Arc::clone(&self.empty_animation_definition);
        if let Some(definition) = self.animation.resolve(&definitions, &empty) {
            self.set_time(definition, seconds);
        }
    }

    fn loop_value_for_definition(&self, definition: &RuntimeLinearAnimation) -> i32 {
        if self.loop_value_override == -1 {
            definition.loop_value as i32
        } else {
            self.loop_value_override
        }
    }

    /// Mirrors C++ `LinearAnimationInstance::loopValue`: the `-1` sentinel
    /// delegates through the retained animation handle, while every other
    /// signed value is returned unchanged.
    pub fn loop_value(&self) -> i32 {
        self.retained_definition()
            .map(|definition| self.loop_value_for_definition(definition))
            .unwrap_or(self.loop_value_override)
    }

    pub fn set_loop_value(&mut self, value: i32) {
        let definition_loop_value = self
            .retained_definition()
            .map(|definition| definition.loop_value as i32)
            .unwrap_or(-1);
        if self.loop_value_override == value
            || (self.loop_value_override == -1 && definition_loop_value == value)
        {
            return;
        }
        self.loop_value_override = value;
    }

    pub(crate) fn set_time(&mut self, animation: &RuntimeLinearAnimation, value: f32) {
        if self.time == value {
            return;
        }
        self.time = value;
        let diff = self.total_time - self.last_total_time;
        let start = if animation.enable_work_area {
            animation.work_start as f32
        } else {
            0.0
        } * animation.fps_as_f32();
        self.total_time = value - start;
        self.last_total_time = self.total_time - diff;
        self.direction = 1.0;
    }

    pub(crate) fn reset(&mut self, animation: &RuntimeLinearAnimation, speed_multiplier: f32) {
        self.time = animation.start_time_with_speed(speed_multiplier);
    }

    pub fn directed_speed(&self, animation: &RuntimeLinearAnimation) -> f32 {
        self.direction * animation.speed
    }

    pub(crate) fn resolved_loop_kind(&self, animation: &RuntimeLinearAnimation) -> AnimationLoop {
        AnimationLoop::from_loop_value(if self.loop_value_override != -1 {
            self.loop_value_override
        } else {
            animation.loop_value as i32
        })
    }

    pub(crate) fn keep_going(&self) -> bool {
        let Some(animation) = self.retained_definition() else {
            return false;
        };
        self.resolved_loop_kind(animation) != AnimationLoop::OneShot
            || (self.directed_speed(animation) > 0.0 && self.time < animation.end_seconds())
            || (self.directed_speed(animation) < 0.0 && self.time > animation.start_seconds())
    }

    pub(crate) fn advance(&mut self, elapsed_seconds: f32) -> bool {
        let definitions = Arc::clone(&self.animation_definitions);
        let empty_definition = Arc::clone(&self.empty_animation_definition);
        let Some(animation) = self.animation.resolve(&definitions, &empty_definition) else {
            return false;
        };
        self.advance_and_report(animation, elapsed_seconds, None)
    }

    pub(crate) fn advance_with_events(
        &mut self,
        elapsed_seconds: f32,
        reported_events: &mut Vec<StateMachineReportedEvent>,
        keyed_callbacks: &mut Vec<RuntimeKeyedCallback>,
    ) -> bool {
        let definitions = Arc::clone(&self.animation_definitions);
        let empty_definition = Arc::clone(&self.empty_animation_definition);
        let Some(animation) = self.animation.resolve(&definitions, &empty_definition) else {
            return false;
        };
        let mut callback_sink =
            |callback: RuntimeKeyedCallback, event: Option<StateMachineReportedEvent>| {
                keyed_callbacks.push(callback);
                if let Some(event) = event {
                    reported_events.push(event);
                }
            };
        self.advance_and_report(animation, elapsed_seconds, Some(&mut callback_sink))
    }

    pub(crate) fn advance_with_callback_sink(
        &mut self,
        elapsed_seconds: f32,
        callback_sink: &mut dyn FnMut(RuntimeKeyedCallback, Option<StateMachineReportedEvent>),
    ) -> bool {
        let definitions = Arc::clone(&self.animation_definitions);
        let empty_definition = Arc::clone(&self.empty_animation_definition);
        let Some(animation) = self.animation.resolve(&definitions, &empty_definition) else {
            return false;
        };
        self.advance_and_report(animation, elapsed_seconds, Some(callback_sink))
    }

    fn advance_and_report(
        &mut self,
        animation: &RuntimeLinearAnimation,
        elapsed_seconds: f32,
        mut callback_sink: Option<
            &mut dyn FnMut(RuntimeKeyedCallback, Option<StateMachineReportedEvent>),
        >,
    ) -> bool {
        self.pending_scripted_bind_advance.set(Some(elapsed_seconds));
        let delta_seconds = elapsed_seconds * animation.speed * self.direction;
        self.spilled_time = 0.0;
        if delta_seconds == 0.0 {
            self.did_loop = false;
            return false;
        }

        self.last_total_time = self.total_time;
        self.total_time += delta_seconds.abs();
        let kill_spilled_time =
            !self.keep_going_with_speed_multiplier_for_definition(animation, elapsed_seconds);

        let mut last_time = self.time;
        self.time += delta_seconds;
        if let Some(callback_sink) = callback_sink.as_deref_mut() {
            animation.report_keyed_callbacks(
                last_time,
                self.time,
                self.speed_direction,
                false,
                callback_sink,
            );
        }
        let fps = animation.fps_as_f32();
        let mut frames = self.time * fps;
        let start = animation.start_frame();
        let end = animation.end_frame();
        let range = end - start;
        let mut did_loop = false;
        let mut direction = if delta_seconds < 0.0 { -1 } else { 1 };

        match self.resolved_loop_kind(animation) {
            AnimationLoop::OneShot => {
                if direction == 1 && frames > end {
                    let delta_frames = delta_seconds * fps;
                    let spilled_frames_ratio = (frames - end) / delta_frames;
                    self.spilled_time = spilled_frames_ratio * elapsed_seconds;
                    frames = end;
                    self.time = frames / fps;
                    did_loop = true;
                } else if direction == -1 && frames < start {
                    let delta_frames = (delta_seconds * fps).abs();
                    let spilled_frames_ratio = (start - frames) / delta_frames;
                    self.spilled_time = spilled_frames_ratio * elapsed_seconds;
                    frames = start;
                    self.time = frames / fps;
                    did_loop = true;
                }
            }
            AnimationLoop::Loop => {
                if direction == 1 && frames >= end {
                    let delta_frames = delta_seconds * fps;
                    let remainder = (frames - start) % range;
                    let spilled_frames_ratio = remainder / delta_frames;
                    self.spilled_time = spilled_frames_ratio * elapsed_seconds;
                    frames = start + remainder;
                    self.time = frames / fps;
                    did_loop = true;
                    if let Some(callback_sink) = callback_sink.as_deref_mut() {
                        animation.report_keyed_callbacks(
                            0.0,
                            self.time,
                            self.speed_direction,
                            false,
                            callback_sink,
                        );
                    }
                } else if direction == -1 && frames <= start {
                    let delta_frames = delta_seconds * fps;
                    let remainder = ((start - frames) % range).abs();
                    let spilled_frames_ratio = (remainder / delta_frames).abs();
                    self.spilled_time = spilled_frames_ratio * elapsed_seconds;
                    frames = end - remainder;
                    self.time = frames / fps;
                    did_loop = true;
                    if let Some(callback_sink) = callback_sink.as_deref_mut() {
                        animation.report_keyed_callbacks(
                            end / fps,
                            self.time,
                            self.speed_direction,
                            false,
                            callback_sink,
                        );
                    }
                }
            }
            AnimationLoop::PingPong => {
                let mut from_pong = true;
                loop {
                    if direction == 1 && frames >= end {
                        self.spilled_time = (frames - end) / fps;
                        frames = end + (end - frames);
                        last_time = end / fps;
                    } else if direction == -1 && frames < start {
                        self.spilled_time = (start - frames) / fps;
                        frames = start + (start - frames);
                        last_time = start / fps;
                    } else {
                        break;
                    }
                    self.time = frames / fps;
                    self.direction *= -1.0;
                    direction *= -1;
                    did_loop = true;
                    if let Some(callback_sink) = callback_sink.as_deref_mut() {
                        animation.report_keyed_callbacks(
                            last_time,
                            self.time,
                            self.speed_direction,
                            from_pong,
                            callback_sink,
                        );
                    }
                    from_pong = !from_pong;
                }
            }
        }

        if kill_spilled_time {
            self.spilled_time = 0.0;
        }
        self.did_loop = did_loop;
        self.keep_going_with_speed_multiplier_for_definition(animation, elapsed_seconds)
    }

    fn keep_going_with_speed_multiplier_for_definition(
        &self,
        animation: &RuntimeLinearAnimation,
        speed_multiplier: f32,
    ) -> bool {
        self.resolved_loop_kind(animation) != AnimationLoop::OneShot
            || (self.directed_speed(animation) * speed_multiplier > 0.0
                && self.time < animation.end_seconds())
            || (self.directed_speed(animation) * speed_multiplier < 0.0
                && self.time > animation.start_seconds())
    }
}

impl Drop for LinearAnimationInstance {
    fn drop(&mut self) {
        self.remove_key_frame_data_binds();
    }
}
