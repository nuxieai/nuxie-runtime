//! Complete LinearAnimationInstance owner translation for pinned Rive 160085c654874d35.
//! Authored/runtime handles are Rust lifetime glue; callback order follows the source.

use crate::mechanical_port::source::{
    animation::{
        interpolating_keyframe::KeyFrameValueContext,
        keyed_callback_reporter::KeyedCallbackReporter,
        linear_animation::{LinearAnimation, LinearAnimationOwner},
        linear_animation_instance_extras::LAIBindingExtras,
        r#loop::Loop,
        nested_animation::NestedEventNotifier,
    },
    artboard::{Artboard, RuntimeArtboardInstanceWeakHandle},
    core::{CoreHandle, field_types::core_callback_type::CallbackContext},
    data_bind::{
        bindable_property_boolean::BindablePropertyBoolean,
        bindable_property_color::BindablePropertyColor,
        bindable_property_number::BindablePropertyNumber,
        bindable_property_string::BindablePropertyString,
    },
    generated::{
        animation::{
            keyframe_bool_base::KeyFrameBoolBase, keyframe_color_base::KeyFrameColorBase,
            keyframe_double_base::KeyFrameDoubleBase, keyframe_string_base::KeyFrameStringBase,
        },
        data_bind::{
            bindable_property_boolean_base::BindablePropertyBooleanBase,
            bindable_property_color_base::BindablePropertyColorBase,
            bindable_property_number_base::BindablePropertyNumberBase,
            bindable_property_string_base::BindablePropertyStringBase,
        },
    },
    scene::{Scene, SceneBehavior},
    scripted::scripted_interpolator::ScriptedInterpolator,
};
use std::{
    cell::{RefCell, RefMut},
    rc::Rc,
};

enum AdvanceReporter<'a> {
    None,
    External(&'a mut dyn KeyedCallbackReporter),
    Instance,
}

pub struct LinearAnimationInstance {
    blend_accumulator: std::rc::Weak<RefCell<super::blend_accumulator::BlendAccumulator>>,
    animation: LinearAnimationOwner,
    scene: Scene,
    nested_event_notifier: NestedEventNotifier,
    time: f32,
    speed_direction: f32,
    total_time: f32,
    last_total_time: f32,
    spilled_time: f32,
    direction: f32,
    did_loop: bool,
    loop_value: i32,
    // Common instances pay only for a nullable cold-cluster pointer and the
    // borrow flag; copies deliberately start with no allocated cluster.
    binding_extras: RefCell<Option<Box<LAIBindingExtras>>>,
}
impl LinearAnimationInstance {
    pub fn new(
        animation: CoreHandle,
        artboard: RuntimeArtboardInstanceWeakHandle,
        speed_multiplier: f32,
    ) -> Self {
        Self::from_owner(
            LinearAnimationOwner::Authored(animation),
            artboard,
            speed_multiplier,
        )
    }

    pub fn new_runtime(
        animation: Rc<RefCell<LinearAnimation>>,
        artboard: RuntimeArtboardInstanceWeakHandle,
        speed_multiplier: f32,
    ) -> Self {
        Self::from_owner(
            LinearAnimationOwner::Runtime(animation),
            artboard,
            speed_multiplier,
        )
    }

    fn from_owner(
        animation: LinearAnimationOwner,
        artboard: RuntimeArtboardInstanceWeakHandle,
        speed_multiplier: f32,
    ) -> Self {
        let time = animation.with(|animation| {
            if speed_multiplier >= 0.0 {
                animation.start_time()
            } else {
                animation.end_time()
            }
        });
        Self {
            animation,
            blend_accumulator: std::rc::Weak::new(),
            scene: Scene::from_runtime_artboard_link(artboard),
            nested_event_notifier: NestedEventNotifier::default(),
            time,
            speed_direction: if speed_multiplier >= 0.0 { 1.0 } else { -1.0 },
            total_time: 0.0,
            last_total_time: 0.0,
            spilled_time: 0.0,
            direction: 1.0,
            did_loop: false,
            loop_value: -1,
            binding_extras: RefCell::new(None),
        }
    }

    pub fn set_nested_artboard(&mut self, artboard: CoreHandle) {
        self.nested_event_notifier.set_nested_artboard(artboard);
    }

    pub fn set_blend_accumulator(
        &mut self,
        accumulator: &Rc<RefCell<super::blend_accumulator::BlendAccumulator>>,
    ) {
        self.blend_accumulator = Rc::downgrade(accumulator);
    }

    pub fn add_nested_event_listener(
        &mut self,
        listener: crate::mechanical_port::source::animation::state_machine_instance::RuntimeStateMachineInstanceWeakHandle,
    ) {
        self.nested_event_notifier
            .add_nested_event_listener(listener);
    }

    pub fn remove_nested_event_listener(
        &mut self,
        listener: crate::mechanical_port::source::animation::state_machine_instance::RuntimeStateMachineInstanceWeakHandle,
    ) {
        self.nested_event_notifier
            .remove_nested_event_listener(listener);
    }

    pub(crate) fn with_animation<R>(&self, f: impl FnOnce(&LinearAnimation) -> R) -> R {
        self.animation.with(f)
    }

    pub fn clear_spilled_time(&mut self) {
        self.spilled_time = 0.0
    }
    pub fn time(&self) -> f32 {
        self.time
    }
    pub fn direction(&self) -> f32 {
        self.direction
    }
    pub fn directed_speed(&self) -> f32 {
        self.direction * self.speed()
    }
    pub fn set_direction(&mut self, value: i32) {
        self.direction = if value > 0 { 1.0 } else { -1.0 }
    }
    pub fn set_time(&mut self, value: f32) {
        if self.time == value {
            return;
        }
        self.time = value;
        let difference = self.total_time - self.last_total_time;
        let start = self.with_animation(|animation| {
            (if animation.base.enable_work_area() {
                animation.base.work_start() as f32
            } else {
                0.0
            }) * animation.base.fps() as f32
        });
        self.total_time = value - start;
        self.last_total_time = self.total_time - difference;
        self.direction = 1.0
    }
    pub fn apply(&self, mix: f32) {
        if let Some(artboard) = self.scene.artboard_instance_ref().upgrade() {
            artboard.apply_linear_animation_owner(&self.animation, self.time, mix, Some(self));
        }
    }
    pub fn did_loop(&self) -> bool {
        self.did_loop
    }
    pub fn keep_going(&self) -> bool {
        self.keep_going_with_multiplier(1.0)
    }
    pub fn keep_going_with_multiplier(&self, m: f32) -> bool {
        self.loop_value() != Loop::OneShot.value() as i32
            || (self.directed_speed() * m > 0.0
                && self.time < self.with_animation(LinearAnimation::end_seconds))
            || (self.directed_speed() * m < 0.0
                && self.time > self.with_animation(LinearAnimation::start_seconds))
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
    pub fn duration_seconds(&self) -> f32 {
        self.with_animation(LinearAnimation::duration_seconds)
    }
    pub fn global_to_local_seconds(&self, seconds: f32) -> f32 {
        self.with_animation(|animation| animation.global_to_local_seconds(seconds))
    }
    pub fn fps(&self) -> u32 {
        self.with_animation(|animation| animation.base.fps())
    }
    pub fn duration(&self) -> u32 {
        self.with_animation(|animation| animation.base.duration())
    }
    pub fn speed(&self) -> f32 {
        self.with_animation(|animation| animation.base.speed())
    }
    pub fn start_time(&self) -> f32 {
        self.with_animation(LinearAnimation::start_time)
    }
    pub fn name(&self) -> String {
        self.with_animation(|animation| animation.base.base.name().to_owned())
    }
    pub fn loop_value(&self) -> i32 {
        if self.loop_value != -1 {
            self.loop_value
        } else {
            self.with_animation(|animation| animation.base.loop_value() as i32)
        }
    }
    pub fn set_loop_value(&mut self, value: i32) {
        if self.loop_value == value
            || (self.loop_value == -1
                && self.with_animation(|animation| animation.base.loop_value() as i32) == value)
        {
            return;
        }
        self.loop_value = value
    }
    pub fn reset(&mut self, m: f32) {
        self.time = self.with_animation(|animation| {
            if m >= 0.0 {
                animation.start_time()
            } else {
                animation.end_time()
            }
        })
    }
    pub fn keyframe_value_holder(&self, key: &CoreHandle) -> Option<CoreHandle> {
        let holder = self
            .binding_extras
            .borrow()
            .as_ref()
            .and_then(|extras| extras.keyframe_value_holders.get(key).cloned());
        if let Some(holder) = holder {
            // Release the cluster borrow before flushing: the bind may call
            // back into animation application.
            let (bind, container) = {
                let extras = self.binding_extras.borrow();
                let extras = extras.as_ref().unwrap();
                (
                    extras.keyframe_value_binds.get(key).cloned(),
                    extras.bind_container.upgrade(),
                )
            };
            if let Some(bind) = bind {
                if let Some(container) = container {
                    container.flush_data_bind(&bind);
                }
            }
            return Some(holder);
        }
        let source = self
            .scene
            .artboard_instance_ref()
            .with_artboard(|artboard| artboard.base.artboard_source_handle())
            .flatten()?;
        let bind = source
            .with_downcast::<Artboard, _>(|source| {
                if source.has_key_frame_source_binds() {
                    source.key_frame_source_bind(key)
                } else {
                    None
                }
            })
            .flatten()?;
        self.build_keyframe_value_holder(key, &bind)
    }

    fn build_keyframe_value_holder(
        &self,
        key: &CoreHandle,
        source_bind: &CoreHandle,
    ) -> Option<CoreHandle> {
        let (holder, property_key) = match key.core_type()? {
            KeyFrameDoubleBase::TYPE_KEY => (
                key.insert_sibling(BindablePropertyNumber::default())?,
                BindablePropertyNumberBase::PROPERTY_VALUE_PROPERTY_KEY,
            ),
            KeyFrameColorBase::TYPE_KEY => (
                key.insert_sibling(BindablePropertyColor::default())?,
                BindablePropertyColorBase::PROPERTY_VALUE_PROPERTY_KEY,
            ),
            KeyFrameBoolBase::TYPE_KEY => (
                key.insert_sibling(BindablePropertyBoolean::default())?,
                BindablePropertyBooleanBase::PROPERTY_VALUE_PROPERTY_KEY,
            ),
            KeyFrameStringBase::TYPE_KEY => (
                key.insert_sibling(BindablePropertyString::default())?,
                BindablePropertyStringBase::PROPERTY_VALUE_PROPERTY_KEY,
            ),
            _ => return None,
        };
        self.ensure_binding_extras()
            .keyframe_value_holders
            .insert(key.clone(), holder.clone());
        let clone = source_bind
            .clone_occurrence()
            .expect("source DataBind is cloneable");
        let file = source_bind
            .with(|source| {
                source
                    .as_data_bind()
                    .expect("source keyframe DataBind")
                    .file()
            })
            .expect("live source DataBind");
        clone.with_mut(|object| {
            let bind = object.as_data_bind_mut().expect("cloned DataBind");
            bind.set_file(file);
            bind.configure_target(holder.clone(), property_key as u32);
        });
        crate::source::data_bind::data_bind::DataBind::initialize_handle(&clone);
        let converter = source_bind
            .with(|source| {
                source
                    .as_data_bind()
                    .expect("source keyframe DataBind")
                    .converter()
            })
            .expect("live source DataBind");
        if let Some(converter) = converter {
            let converter = converter.clone_occurrence();
            clone.with_mut(|object| object.as_data_bind_mut().unwrap().set_converter(converter));
        }
        let container = self
            .scene
            .artboard_instance_ref()
            .with_artboard(|artboard| artboard.base.data_bind_container.clone())
            .expect("live animation artboard");
        self.ensure_binding_extras().bind_container = container.downgrade();
        container.add_data_bind(clone.clone());
        self.ensure_binding_extras()
            .keyframe_value_binds
            .insert(key.clone(), clone);
        Some(holder)
    }
    pub fn cache_scripted_interpolator(
        &mut self,
        key: CoreHandle,
        value: CoreHandle,
        binds: Vec<CoreHandle>,
    ) {
        let container = self
            .scene
            .artboard_instance_ref()
            .with_artboard(|artboard| artboard.base.data_bind_container.downgrade());
        let mut extras = self.ensure_binding_extras();
        extras.scripted_interpolators.insert(key, value);
        extras.cloned_artboard_data_binds.extend(binds);
        if let Some(container) = container {
            extras.bind_container = container;
        }
    }
    fn ensure_binding_extras(&self) -> RefMut<'_, LAIBindingExtras> {
        RefMut::map(self.binding_extras.borrow_mut(), |extras| {
            extras
                .get_or_insert_with(|| Box::new(LAIBindingExtras::default()))
                .as_mut()
        })
    }
    pub fn stateful_interpolator(
        &self,
        keyframe: CoreHandle,
        shared: CoreHandle,
    ) -> Option<CoreHandle> {
        let cached = self
            .ensure_binding_extras()
            .scripted_interpolators
            .get(&keyframe)
            .cloned();
        if let Some(cached) = cached {
            return Some(cached);
        }
        use crate::mechanical_port::source::scripted::scripted_object::{
            ScriptUpdateRequestHost, ScriptedObject,
        };
        let owner = ScriptedInterpolator::clone_scripted_occurrence(&shared, |bind| {
            self.scene
                .artboard_instance_ref()
                .with_artboard_mut(|artboard| artboard.add_data_bind(bind))
                .expect("the animation retains its artboard while cloning an interpolator");
        })?;
        let context = self
            .scene
            .artboard_instance_ref()
            .with_artboard(|artboard| artboard.data_context())?;
        self.ensure_binding_extras().bind_container = self
            .scene
            .artboard_instance_ref()
            .with_artboard(|artboard| artboard.base.data_bind_container.downgrade())?;
        let (properties, needs_init) =
            owner.with_downcast_mut::<ScriptedInterpolator, _>(|clone| {
                clone.scripted.set_data_context(context);
                (
                    clone.properties.clone(),
                    clone.scripted.script_asset().is_some() && !clone.scripted.user_lua_init_done(),
                )
            })?;
        // The input backlink is installed after addDataBind by cloneProperties.
        // Discover the same binds as the pinned owner, preserving their order.
        for property in &properties {
            if let Some(bind) = property
                .with(|property| property.script_input_data_bind())
                .flatten()
            {
                self.ensure_binding_extras()
                    .cloned_artboard_data_binds
                    .push(bind);
            }
        }
        if needs_init {
            let mut host = ScriptUpdateRequestHost::default();
            ScriptedObject::reinit_occurrence(&owner, &properties, &mut host);
        }
        self.ensure_binding_extras()
            .scripted_interpolators
            .insert(keyframe, owner.clone());
        Some(owner)
    }
    pub fn advance_and_apply(&mut self, seconds: f32) -> bool {
        // Scripted interpolators run in apply(), before Artboard::advance polls.
        if let Some(artboard) = self.scene.artboard_instance_ref().upgrade() {
            Artboard::poll_async_work_handle(&artboard.core_handle());
        }
        let mut more = self.advance_and_report_to_self(seconds);
        self.apply(1.0);
        if self
            .scene
            .artboard_instance_ref()
            .upgrade()
            .is_some_and(|artboard| artboard.advance_default(seconds))
        {
            more = true;
        }
        more || self.keep_going()
            || self
                .scene
                .artboard_instance_ref()
                .upgrade()
                .is_some_and(|artboard| {
                    artboard.with_artboard(|artboard| artboard.has_pending_async_work())
                })
    }

    pub fn advance_and_report_to_self(&mut self, seconds: f32) -> bool {
        self.advance_with_reporter(seconds, AdvanceReporter::Instance)
    }

    pub fn advance(
        &mut self,
        elapsed: f32,
        reporter: Option<&mut dyn KeyedCallbackReporter>,
    ) -> bool {
        let reporter = match reporter {
            Some(reporter) => AdvanceReporter::External(reporter),
            None => AdvanceReporter::None,
        };
        self.advance_with_reporter(elapsed, reporter)
    }

    fn report_animation_callbacks(
        &mut self,
        reporter: &mut AdvanceReporter<'_>,
        from: f32,
        to: f32,
        from_pong: bool,
    ) {
        let speed_direction = self.speed_direction;
        match reporter {
            AdvanceReporter::None => {}
            AdvanceReporter::External(reporter) => self.animation.report_keyed_callbacks(
                *reporter,
                from,
                to,
                speed_direction,
                from_pong,
            ),
            AdvanceReporter::Instance => {
                // A local retained pointer allows this instance to be the
                // synchronous reporter without borrowing its animation field.
                let animation = self.animation.clone();
                animation.report_keyed_callbacks(self, from, to, speed_direction, from_pong);
            }
        }
    }

    fn advance_with_reporter(&mut self, elapsed: f32, mut reporter: AdvanceReporter<'_>) -> bool {
        let delta = elapsed * self.speed() * self.direction;
        self.spilled_time = 0.0;
        if delta == 0.0 {
            self.did_loop = false;
            return false;
        }

        self.last_total_time = self.total_time;
        self.total_time += delta.abs();
        let kill_spilled_time = !self.keep_going_with_multiplier(elapsed);
        let mut last_time = self.time;
        self.time += delta;
        self.report_animation_callbacks(&mut reporter, last_time, self.time, false);

        // These reads occur after the first synchronous report in C++.
        let (fps, start, end) = self.with_animation(|animation| {
            let fps = animation.base.fps() as f32;
            let start = if animation.base.enable_work_area() {
                animation.base.work_start() as f32
            } else {
                0.0
            };
            let end = if animation.base.enable_work_area() {
                animation.base.work_end() as f32
            } else {
                animation.base.duration() as f32
            };
            (fps, start, end)
        });
        let mut frames = self.time * fps;
        let range = end - start;
        let mut did_loop = false;
        let mut direction = if delta < 0.0 { -1 } else { 1 };
        match self.loop_value() {
            value if value == Loop::OneShot.value() as i32 => {
                if direction == 1 && frames > end {
                    let delta_frames = delta * fps;
                    let spilled_frames_ratio = (frames - end) / delta_frames;
                    self.spilled_time = spilled_frames_ratio * elapsed;
                    frames = end;
                    self.time = frames / fps;
                    did_loop = true;
                } else if direction == -1 && frames < start {
                    let delta_frames = (delta * fps).abs();
                    let spilled_frames_ratio = (start - frames) / delta_frames;
                    self.spilled_time = spilled_frames_ratio * elapsed;
                    frames = start;
                    self.time = frames / fps;
                    did_loop = true;
                }
            }
            value if value == Loop::Loop.value() as i32 => {
                if direction == 1 && frames >= end {
                    let delta_frames = delta * fps;
                    let remainder = (frames - start) % range;
                    let spilled_frames_ratio = remainder / delta_frames;
                    self.spilled_time = spilled_frames_ratio * elapsed;
                    frames = start + remainder;
                    self.time = frames / fps;
                    did_loop = true;
                    self.report_animation_callbacks(&mut reporter, 0.0, self.time, false);
                } else if direction == -1 && frames <= start {
                    let delta_frames = delta * fps;
                    let remainder = ((start - frames) % range).abs();
                    let spilled_frames_ratio = (remainder / delta_frames).abs();
                    self.spilled_time = spilled_frames_ratio * elapsed;
                    frames = end - remainder;
                    self.time = frames / fps;
                    did_loop = true;
                    self.report_animation_callbacks(&mut reporter, end / fps, self.time, false);
                }
            }
            value if value == Loop::PingPong.value() as i32 => {
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
                    self.report_animation_callbacks(&mut reporter, last_time, self.time, from_pong);
                    from_pong = !from_pong;
                }
            }
            // C++'s switch has no default: an unknown loop value does not wrap.
            _ => {}
        }
        if kill_spilled_time {
            self.spilled_time = 0.0;
        }
        self.did_loop = did_loop;
        self.keep_going_with_multiplier(elapsed)
    }

    pub fn is_translucent(&self) -> bool {
        self.scene
            .artboard_instance_ref()
            .with_artboard(|artboard| artboard.is_animation_instance_translucent(self))
            .unwrap_or(false)
    }
    pub fn report_event(&mut self, event: CoreHandle, _delay: f32) {
        self.nested_event_notifier.notify_listeners(&[event]);
    }
}
impl CallbackContext for LinearAnimationInstance {
    fn report_event(
        &mut self,
        event: &mut crate::mechanical_port::source::event::Event,
        delay: f32,
    ) {
        let event = event
            .base
            .handle()
            .expect("a reported Event has authored identity");
        LinearAnimationInstance::report_event(self, event, delay);
    }
}
impl SceneBehavior for LinearAnimationInstance {
    fn scene(&self) -> &Scene {
        &self.scene
    }

    fn scene_mut(&mut self) -> &mut Scene {
        &mut self.scene
    }

    fn name(&self) -> String {
        LinearAnimationInstance::name(self)
    }

    fn loop_(&self) -> Loop {
        Loop::from(self.loop_value() as u32)
    }

    fn is_translucent(&self) -> bool {
        LinearAnimationInstance::is_translucent(self)
    }

    fn duration_seconds(&self) -> f32 {
        LinearAnimationInstance::duration_seconds(self)
    }

    fn advance_and_apply(&mut self, elapsed_seconds: f32) -> bool {
        LinearAnimationInstance::advance_and_apply(self, elapsed_seconds)
    }
}
impl KeyFrameValueContext for LinearAnimationInstance {
    fn blend_accumulator(&self) -> Option<Rc<RefCell<super::blend_accumulator::BlendAccumulator>>> {
        self.blend_accumulator.upgrade()
    }
    fn bool_value(&self, keyframe: &CoreHandle) -> Option<bool> {
        self.keyframe_value_holder(keyframe)?
            .with_downcast::<BindablePropertyBoolean, _>(|holder| holder.base.property_value())
    }

    fn string_value(&self, keyframe: &CoreHandle) -> Option<String> {
        self.keyframe_value_holder(keyframe)?
            .with_downcast::<BindablePropertyString, _>(|holder| {
                holder.base.property_value().to_owned()
            })
    }

    fn color_value(&self, keyframe: &CoreHandle) -> Option<i32> {
        self.keyframe_value_holder(keyframe)?
            .with_downcast::<BindablePropertyColor, _>(|holder| holder.base.property_value())
    }

    fn number_value(&self, keyframe: &CoreHandle) -> Option<f32> {
        self.keyframe_value_holder(keyframe)?
            .with_downcast::<BindablePropertyNumber, _>(|holder| holder.base.property_value())
    }

    fn stateful_interpolator_transform_value(
        &self,
        keyframe: &CoreHandle,
        shared: &CoreHandle,
        from: f32,
        to: f32,
        factor: f32,
    ) -> Option<f32> {
        self.stateful_interpolator(keyframe.clone(), shared.clone())?
            .with_downcast_mut::<ScriptedInterpolator, _>(|interpolator| {
                interpolator.transform_value(from, to, factor)
            })
    }

    fn stateful_interpolator_transform(
        &self,
        keyframe: &CoreHandle,
        shared: &CoreHandle,
        factor: f32,
    ) -> Option<f32> {
        self.stateful_interpolator(keyframe.clone(), shared.clone())?
            .with_downcast_mut::<ScriptedInterpolator, _>(|interpolator| {
                interpolator.transform(factor)
            })
    }
}
impl KeyedCallbackReporter for LinearAnimationInstance {
    fn report_keyed_callback(&mut self, object_id: u32, property_key: u32, elapsed_seconds: f32) {
        let target = self
            .scene
            .artboard_instance_ref()
            .with_artboard(|artboard| artboard.base.resolve_handle(object_id))
            .flatten();
        if let Some(target) = target {
            if crate::mechanical_port::source::event::Event::trigger_builtin_occurrence(
                &target,
                property_key,
                self,
                elapsed_seconds,
                LinearAnimationInstance::report_event,
            ) {
                return;
            }
            crate::mechanical_port::source::generated::core_registry::CoreRegistry::set_callback_handle(
                &target, property_key as i32,
                crate::mechanical_port::source::core::field_types::core_callback_type::CallbackData::new(Some(self), elapsed_seconds),
            );
        }
    }
}
impl Clone for LinearAnimationInstance {
    fn clone(&self) -> Self {
        Self {
            animation: self.animation.clone(),
            // Upstream's explicit copy constructor leaves this pointer null.
            blend_accumulator: std::rc::Weak::new(),
            scene: self.scene.clone(),
            // The C++ copy constructor initializes Scene but default-constructs
            // the other base, NestedEventNotifier.
            nested_event_notifier: NestedEventNotifier::default(),
            time: self.time,
            speed_direction: self.speed_direction,
            total_time: self.total_time,
            last_total_time: self.last_total_time,
            spilled_time: self.spilled_time,
            direction: self.direction,
            did_loop: self.did_loop,
            loop_value: self.loop_value,
            binding_extras: RefCell::new(None),
        }
    }
}
impl Drop for LinearAnimationInstance {
    fn drop(&mut self) {
        let Some(mut extras) = self.binding_extras.get_mut().take() else {
            return;
        };
        for bind in extras
            .cloned_artboard_data_binds
            .drain(..)
            .chain(extras.keyframe_value_binds.drain().map(|(_, bind)| bind))
        {
            if let Some(container) = extras.bind_container.upgrade() {
                container.remove_data_bind(bind.clone());
            }
            bind.remove_occurrence();
        }
        for (_, holder) in extras.keyframe_value_holders.drain() {
            holder.remove_occurrence();
        }
        for (_, interpolator) in extras.scripted_interpolators.drain() {
            interpolator.remove_occurrence();
        }
    }
}

#[cfg(test)]
mod binding_extras_tests {
    use super::*;
    use crate::mechanical_port::source::{
        animation::keyframe_double::KeyFrameDouble, core::CoreArena,
    };

    #[test]
    fn ordinary_playback_and_unbound_lookup_leave_cluster_unallocated() {
        let arena = CoreArena::default();
        let keyframe = arena.insert(KeyFrameDouble::default());
        let mut instance = LinearAnimationInstance::new_runtime(
            Rc::new(RefCell::new(LinearAnimation::default())),
            RuntimeArtboardInstanceWeakHandle::default(),
            1.0,
        );
        assert!(!instance.did_loop());
        assert!(instance.binding_extras.borrow().is_none());
        instance.advance(0.0, None);
        instance.apply(1.0);
        assert!(instance.keyframe_value_holder(&keyframe).is_none());
        assert!(instance.binding_extras.borrow().is_none());
    }

    #[test]
    fn copied_instance_starts_fresh_while_original_keeps_its_cluster() {
        let mut instance = LinearAnimationInstance::new_runtime(
            Rc::new(RefCell::new(LinearAnimation::default())),
            RuntimeArtboardInstanceWeakHandle::default(),
            1.0,
        );
        // The same cluster is retained on repeated use and time seeking.
        let cluster = {
            let extras = instance.ensure_binding_extras();
            &*extras as *const LAIBindingExtras
        };
        instance.set_time(0.5);
        instance.reset(1.0);
        assert_eq!(
            cluster,
            &*instance.ensure_binding_extras() as *const LAIBindingExtras
        );
        instance.did_loop = true;
        let copy = instance.clone();
        assert!(copy.did_loop());
        assert_eq!(copy.time(), instance.time());
        assert!(copy.binding_extras.borrow().is_none());
        assert!(instance.binding_extras.borrow().is_some());
    }
}

#[cfg(test)]
#[path = "linear_animation_instance_contract_tests.rs"]
mod source_contract_tests;
