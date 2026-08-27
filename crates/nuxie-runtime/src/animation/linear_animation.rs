// Mirrors src/animation/linear_animation.cpp and
// include/rive/animation/linear_animation.hpp.
//
// The approved AF-7 own-by-value/import-time lifecycle adaptation performs
// `import`, `onAddedDirty`, and `onAddedClean` while flattening RuntimeFile:
// LinearAnimationImporter requires the retained Artboard importer,
// build_linear_animations visits KeyedObjects in source order, removes every
// failed owner after dirty validation, and treats the only reachable failure,
// MissingObject, as non-fatal. No pinned KeyFrame subtype overrides the clean
// callback, so the clean traversal has no retained effect after publication.
// Rust drop supplies the C++ destructor ownership behavior.
#[derive(Debug)]
#[cfg_attr(test, derive(Clone))]
pub struct RuntimeLinearAnimation {
    pub global_id: u32,
    pub name: Option<Arc<str>>,
    pub fps: u64,
    pub duration: u64,
    pub speed: f32,
    pub loop_value: u64,
    pub work_start: u64,
    pub work_end: u64,
    pub enable_work_area: bool,
    pub quantize: bool,
    pub keyed_objects: Arc<Vec<RuntimeKeyedObject>>,
    pub(crate) key_frame_data_bind_templates: Arc<Vec<RuntimeKeyFrameDataBindTemplate>>,
    /// Authored callback frames are immutable after import. Retain their
    /// presence so ordinary animations do not enter Rust's deferred callback
    /// collection path on every advance.
    pub(crate) has_keyed_callbacks: bool,
}

#[cfg(test)]
pub(crate) static LINEAR_ANIMATION_DELETE_COUNT: std::sync::atomic::AtomicI32 =
    std::sync::atomic::AtomicI32::new(0);

#[cfg(test)]
impl Drop for RuntimeLinearAnimation {
    fn drop(&mut self) {
        // C++'s TESTING-only destructor counter is process-global. Atomic
        // storage is the Rust-safe representation when tests run in parallel.
        LINEAR_ANIMATION_DELETE_COUNT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    }
}

impl RuntimeLinearAnimation {
    // Rust's shared empty-definition sentinel; not an upstream constructor.
    pub(crate) fn empty() -> Self {
        Self {
            global_id: u32::MAX,
            name: None,
            fps: 60,
            duration: 60,
            speed: 1.0,
            loop_value: 0,
            work_start: u64::from(u32::MAX),
            work_end: u64::from(u32::MAX),
            enable_work_area: false,
            quantize: false,
            keyed_objects: Arc::new(Vec::new()),
            key_frame_data_bind_templates: Arc::new(Vec::new()),
            has_keyed_callbacks: false,
        }
    }

    fn add_keyed_object(&mut self, object: RuntimeKeyedObject) {
        Arc::make_mut(&mut self.keyed_objects).push(object);
    }

    pub(crate) fn apply(&self, instance: &mut ArtboardInstance, seconds: f32, mix: f32) -> bool {
        self.apply_with_key_frame_values(
            instance,
            seconds,
            mix,
            RuntimeKeyFrameValueContext::default(),
            None,
        )
    }

    fn apply_with_key_frame_values(
        &self,
        instance: &mut ArtboardInstance,
        seconds: f32,
        mix: f32,
        key_frame_values: RuntimeKeyFrameValueContext<'_>,
        animation_instance: Option<&LinearAnimationInstance>,
    ) -> bool {
        let seconds = if self.quantize {
            let fps = self.fps as f32;
            (seconds * fps).floor() / fps
        } else {
            seconds
        };

        let mut changed = false;
        for keyed_object in self.keyed_objects.iter() {
            changed |=
                keyed_object.apply(instance, seconds, mix, key_frame_values, animation_instance);
        }
        changed
    }

    fn loop_kind(&self) -> AnimationLoop {
        match self.loop_value {
            0 => AnimationLoop::OneShot,
            1 => AnimationLoop::Loop,
            2 => AnimationLoop::PingPong,
            _ => unreachable!("invalid LinearAnimation loopValue"),
        }
    }

    pub(crate) fn start_seconds(&self) -> f32 {
        (if self.enable_work_area {
            self.work_start as f32
        } else {
            0.0
        }) / self.fps as f32
    }

    fn end_seconds(&self) -> f32 {
        (if self.enable_work_area {
            self.work_end as f32
        } else {
            self.duration as f32
        }) / self.fps as f32
    }

    fn start_time(&self) -> f32 {
        if self.speed >= 0.0 {
            self.start_seconds()
        } else {
            self.end_seconds()
        }
    }

    fn start_time_with_speed(&self, speed_multiplier: f32) -> f32 {
        if self.speed * speed_multiplier >= 0.0 {
            self.start_seconds()
        } else {
            self.end_seconds()
        }
    }

    fn end_time(&self) -> f32 {
        if self.speed >= 0.0 {
            self.end_seconds()
        } else {
            self.start_seconds()
        }
    }

    pub(crate) fn duration_seconds(&self) -> f32 {
        (self.end_seconds() - self.start_seconds()).abs()
    }
}

// Matches math::positive_mod as called by LinearAnimation's file-local
// positiveMod wrapper. In particular, an exact negative multiple stays -0.0.
fn positive_mod(value: f32, mut range: f32) -> f32 {
    debug_assert!(range > 0.0);
    if range < 0.0 {
        range = -range;
    }
    let mut value = value % range;
    if value < 0.0 {
        value += range;
    }
    value
}

impl RuntimeLinearAnimation {
    pub(crate) fn global_to_local_seconds(&self, seconds: f32) -> f32 {
        match self.loop_kind() {
            AnimationLoop::OneShot => seconds + self.start_time(),
            AnimationLoop::Loop => {
                positive_mod(seconds, self.duration_seconds()) + self.start_time()
            }
            AnimationLoop::PingPong => {
                let duration = self.duration_seconds();
                let local_time = positive_mod(seconds, duration);
                // Rust defines the otherwise-undefined C++ float-to-int cases;
                // values in C++'s defined i32 domain truncate identically.
                let direction = (seconds / duration) as i32 % 2;
                if direction == 0 {
                    local_time + self.start_time()
                } else {
                    self.end_time() - local_time
                }
            }
            // `loop_kind` rejects this before constructing the enum, matching
            // the pinned `RIVE_UNREACHABLE` after LinearAnimation's switch.
            AnimationLoop::Raw => unreachable!("invalid LinearAnimation loopValue"),
        }
    }

    pub fn get_object(&self, index: usize) -> Option<&RuntimeKeyedObject> {
        self.keyed_objects.get(index)
    }

    pub fn num_keyed_objects(&self) -> usize {
        self.keyed_objects.len()
    }

    fn report_keyed_callbacks(
        &self,
        seconds_from: f32,
        seconds_to: f32,
        speed_direction: f32,
        from_pong: bool,
        callback_sink: &mut dyn FnMut(RuntimeKeyedCallback, Option<StateMachineReportedEvent>),
    ) {
        let starting_time = self.start_time_with_speed(speed_direction);
        let is_at_start_frame = starting_time == seconds_from;

        if !is_at_start_frame || !from_pong {
            for keyed_object in self.keyed_objects.iter() {
                keyed_object.report_keyed_callbacks(
                    seconds_from,
                    seconds_to,
                    is_at_start_frame,
                    callback_sink,
                );
            }
        }
    }
}

// Rust-only helpers consumed by the occurrence and scripting adaptations.
impl RuntimeLinearAnimation {
    fn fps_as_f32(&self) -> f32 {
        self.fps as f32
    }

    fn start_frame(&self) -> f32 {
        if self.enable_work_area {
            self.work_start as f32
        } else {
            0.0
        }
    }

    fn end_frame(&self) -> f32 {
        if self.enable_work_area {
            self.work_end as f32
        } else {
            self.duration as f32
        }
    }

    /// File-global ScriptedInterpolator ids referenced by this animation's
    /// keyframes, in first-use order. This is the Rust scripting adaptation,
    /// not a pinned LinearAnimation body.
    #[doc(hidden)]
    pub fn scripted_interpolator_global_ids(&self) -> Vec<u32> {
        let mut ids = Vec::new();
        for interpolator in self
            .keyed_objects
            .iter()
            .flat_map(|object| &object.keyed_properties)
            .flat_map(|property| &property.key_frames)
            .filter_map(|frame| match frame {
                RuntimeKeyFrame::Double(frame) => frame.interpolator,
                RuntimeKeyFrame::Color(frame) => frame.interpolator,
                _ => None,
            })
        {
            if let RuntimeInterpolator::Scripted { global_id } = interpolator
                && !ids.contains(&global_id)
            {
                ids.push(global_id);
            }
        }
        ids
    }
}
