use crate::mechanical_port::source::{
    animation::{
        interpolating_keyframe::KeyFrameValueContext,
        keyed_callback_reporter::KeyedCallbackReporter, linear_animation::LinearAnimation,
        r#loop::Loop, nested_animation::NestedEventNotifier,
    },
    artboard::{Artboard, RuntimeArtboardInstanceWeakHandle},
    core::{CoreHandle, field_types::core_callback_type::CallbackContext},
    data_bind::{
        bindable_property_boolean::BindablePropertyBoolean,
        bindable_property_color::BindablePropertyColor,
        bindable_property_number::BindablePropertyNumber,
        bindable_property_string::BindablePropertyString,
        data_bind_container::DataBindContainerWeak,
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
    scripted::scripted_interpolator::ScriptedInterpolator,
};
use std::{cell::RefCell, collections::HashMap, rc::Rc};

#[derive(Clone)]
enum LinearAnimationOwner {
    Authored(CoreHandle),
    Runtime(Rc<RefCell<LinearAnimation>>),
}

#[derive(Default)]
struct PendingKeyedCallbacks(Vec<(u32, u32, f32)>);

impl KeyedCallbackReporter for PendingKeyedCallbacks {
    fn report_keyed_callback(&mut self, object_id: u32, property_key: u32, elapsed_seconds: f32) {
        self.0.push((object_id, property_key, elapsed_seconds));
    }
}
pub struct LinearAnimationInstance {
    animation: LinearAnimationOwner,
    artboard: RuntimeArtboardInstanceWeakHandle,
    nested_event_notifier: NestedEventNotifier,
    time: f32,
    speed_direction: f32,
    total_time: f32,
    last_total_time: f32,
    spilled_time: f32,
    direction: f32,
    did_loop: bool,
    loop_value: i32,
    scripted_interpolators: RefCell<Option<HashMap<CoreHandle, CoreHandle>>>,
    cloned_artboard_data_binds: RefCell<Vec<CoreHandle>>,
    keyframe_value_holders: RefCell<Option<HashMap<CoreHandle, CoreHandle>>>,
    keyframe_value_binds: RefCell<HashMap<CoreHandle, CoreHandle>>,
    // Retained field projection for teardown while Artboard itself is borrowed.
    keyframe_bind_container: RefCell<DataBindContainerWeak>,
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
        let time = match &animation {
            LinearAnimationOwner::Authored(animation) => animation
                .with_downcast::<LinearAnimation, _>(|animation| {
                    if speed_multiplier >= 0.0 {
                        animation.start_time()
                    } else {
                        animation.end_time()
                    }
                })
                .expect("LinearAnimationInstance retains a LinearAnimation"),
            LinearAnimationOwner::Runtime(animation) => {
                let animation = animation.borrow();
                if speed_multiplier >= 0.0 {
                    animation.start_time()
                } else {
                    animation.end_time()
                }
            }
        };
        Self {
            animation,
            artboard,
            nested_event_notifier: NestedEventNotifier::default(),
            time,
            speed_direction: if speed_multiplier >= 0.0 { 1.0 } else { -1.0 },
            total_time: 0.0,
            last_total_time: 0.0,
            spilled_time: 0.0,
            direction: 1.0,
            did_loop: false,
            loop_value: -1,
            scripted_interpolators: RefCell::new(None),
            cloned_artboard_data_binds: RefCell::new(Vec::new()),
            keyframe_value_holders: RefCell::new(None),
            keyframe_value_binds: RefCell::new(HashMap::new()),
            keyframe_bind_container: RefCell::new(DataBindContainerWeak::default()),
        }
    }

    pub fn set_nested_artboard(&mut self, artboard: CoreHandle) {
        self.nested_event_notifier.set_nested_artboard(artboard);
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
        match &self.animation {
            LinearAnimationOwner::Authored(animation) => animation
                .with_downcast::<LinearAnimation, _>(f)
                .expect("LinearAnimationInstance retains a LinearAnimation"),
            LinearAnimationOwner::Runtime(animation) => f(&animation.borrow()),
        }
    }

    fn with_animation_mut<R>(&self, f: impl FnOnce(&mut LinearAnimation) -> R) -> R {
        match &self.animation {
            LinearAnimationOwner::Authored(animation) => animation
                .with_downcast_mut::<LinearAnimation, _>(f)
                .expect("LinearAnimationInstance retains a LinearAnimation"),
            LinearAnimationOwner::Runtime(animation) => f(&mut animation.borrow_mut()),
        }
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
        if let Some(artboard) = self.artboard.upgrade() {
            self.with_animation_mut(|animation| {
                artboard.apply_linear_animation(animation, self.time, mix, Some(self));
            });
        }
    }
    pub fn did_loop(&self) -> bool {
        self.did_loop
    }
    pub fn keep_going(&self) -> bool {
        self.keep_going_with_multiplier(1.0)
    }
    pub fn keep_going_with_multiplier(&self, m: f32) -> bool {
        self.loop_value() != Loop::OneShot as i32
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
            .keyframe_value_holders
            .borrow()
            .as_ref()
            .and_then(|holders| holders.get(key).cloned());
        if let Some(holder) = holder {
            let bind = self.keyframe_value_binds.borrow().get(key).cloned();
            if let Some(bind) = bind {
                let container = self.keyframe_bind_container.borrow().upgrade();
                if let Some(container) = container {
                    container.flush_data_bind(&bind);
                }
            }
            return Some(holder);
        }
        let source = self
            .artboard
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
        self.keyframe_value_holders
            .borrow_mut()
            .get_or_insert_with(HashMap::new)
            .insert(key.clone(), holder.clone());
        let clone = source_bind
            .clone_occurrence()
            .expect("source DataBind is cloneable");
        let (file, converter) = source_bind
            .with(|source| {
                let source = source.as_data_bind().expect("source keyframe DataBind");
                (source.file(), source.converter())
            })
            .expect("live source DataBind");
        clone.with_mut(|object| {
            let bind = object.as_data_bind_mut().expect("cloned DataBind");
            bind.set_file(file);
            bind.configure_target(holder.clone(), property_key as u32);
            bind.initialize();
        });
        if let Some(converter) = converter {
            let converter = converter.clone_occurrence();
            clone.with_mut(|object| object.as_data_bind_mut().unwrap().set_converter(converter));
        }
        let container = self
            .artboard
            .with_artboard(|artboard| artboard.base.data_bind_container.clone())
            .expect("live animation artboard");
        *self.keyframe_bind_container.borrow_mut() = container.downgrade();
        container.add_data_bind(clone.clone());
        self.keyframe_value_binds
            .borrow_mut()
            .insert(key.clone(), clone);
        Some(holder)
    }
    pub fn cache_scripted_interpolator(
        &mut self,
        key: CoreHandle,
        value: CoreHandle,
        binds: Vec<CoreHandle>,
    ) {
        self.scripted_interpolators
            .borrow_mut()
            .get_or_insert_with(HashMap::new)
            .insert(key, value);
        self.cloned_artboard_data_binds.borrow_mut().extend(binds)
    }
    pub fn stateful_interpolator(
        &self,
        keyframe: CoreHandle,
        shared: CoreHandle,
    ) -> Option<CoreHandle> {
        if let Some(cached) = self
            .scripted_interpolators
            .borrow()
            .as_ref()
            .and_then(|instances| instances.get(&keyframe))
        {
            return Some(cached.clone());
        }
        use crate::mechanical_port::source::scripted::scripted_object::{
            ScriptUpdateRequestHost, ScriptedObject,
        };
        let owner = ScriptedInterpolator::clone_scripted_occurrence(&shared, |bind| {
            self.artboard
                .with_artboard_mut(|artboard| artboard.add_data_bind(bind))
                .expect("the animation retains its artboard while cloning an interpolator");
        })?;
        let context = self
            .artboard
            .with_artboard(|artboard| artboard.data_context())?;
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
                self.cloned_artboard_data_binds.borrow_mut().push(bind);
            }
        }
        if needs_init {
            let mut host = ScriptUpdateRequestHost::default();
            ScriptedObject::reinit_occurrence(&owner, &properties, &mut host);
        }
        self.scripted_interpolators
            .borrow_mut()
            .get_or_insert_with(HashMap::new)
            .insert(keyframe, owner.clone());
        Some(owner)
    }
    pub fn advance_and_apply(&mut self, seconds: f32) -> bool {
        let mut reporter = PendingKeyedCallbacks::default();
        let mut more = self.advance(seconds, Some(&mut reporter));
        for (object_id, property_key, elapsed_seconds) in reporter.0 {
            self.report_keyed_callback(object_id, property_key, elapsed_seconds);
        }
        self.apply(1.0);
        if self
            .artboard
            .upgrade()
            .is_some_and(|artboard| artboard.advance_default(seconds))
        {
            more = true
        }
        more || self.keep_going()
    }
    pub fn advance_and_report_to_self(&mut self, seconds: f32) -> bool {
        let mut reporter = PendingKeyedCallbacks::default();
        let more = self.advance(seconds, Some(&mut reporter));
        for (object_id, property_key, elapsed_seconds) in reporter.0 {
            self.report_keyed_callback(object_id, property_key, elapsed_seconds);
        }
        more
    }
    pub fn advance(
        &mut self,
        elapsed: f32,
        mut reporter: Option<&mut dyn KeyedCallbackReporter>,
    ) -> bool {
        let (speed, fps, start, end) = self.with_animation(|animation| {
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
            (animation.base.speed(), fps, start, end)
        });
        let delta = elapsed * speed * self.direction;
        self.spilled_time = 0.0;
        if delta == 0.0 {
            self.did_loop = false;
            return false;
        }
        self.last_total_time = self.total_time;
        self.total_time += delta.abs();
        let kill = !self.keep_going_with_multiplier(elapsed);
        let mut last = self.time;
        self.time += delta;
        if let Some(r) = reporter.as_deref_mut() {
            self.with_animation(|animation| {
                animation.report_keyed_callbacks(r, last, self.time, self.speed_direction, false)
            })
        }
        let mut frames = self.time * fps;
        let range = end - start;
        let mut looped = false;
        let mut direction = if delta < 0.0 { -1 } else { 1 };
        match self.loop_value() {
            x if x == Loop::OneShot as i32 => {
                if direction == 1 && frames > end {
                    self.spilled_time = (frames - end) / (delta * fps) * elapsed;
                    frames = end;
                    self.time = frames / fps;
                    looped = true
                } else if direction == -1 && frames < start {
                    self.spilled_time = (start - frames) / (delta * fps).abs() * elapsed;
                    frames = start;
                    self.time = frames / fps;
                    looped = true
                }
            }
            x if x == Loop::Loop as i32 => {
                if direction == 1 && frames >= end {
                    let remainder = (frames - start) % range;
                    self.spilled_time = remainder / (delta * fps) * elapsed;
                    frames = start + remainder;
                    self.time = frames / fps;
                    looped = true;
                    if let Some(r) = reporter.as_deref_mut() {
                        self.with_animation(|animation| {
                            animation.report_keyed_callbacks(
                                r,
                                0.0,
                                self.time,
                                self.speed_direction,
                                false,
                            )
                        })
                    }
                } else if direction == -1 && frames <= start {
                    let remainder = ((start - frames) % range).abs();
                    self.spilled_time = (remainder / (delta * fps)).abs() * elapsed;
                    frames = end - remainder;
                    self.time = frames / fps;
                    looped = true;
                    if let Some(r) = reporter.as_deref_mut() {
                        self.with_animation(|animation| {
                            animation.report_keyed_callbacks(
                                r,
                                end / fps,
                                self.time,
                                self.speed_direction,
                                false,
                            )
                        })
                    }
                }
            }
            _ => {
                let mut from_pong = true;
                loop {
                    if direction == 1 && frames >= end {
                        self.spilled_time = (frames - end) / fps;
                        frames = end + (end - frames);
                        last = end / fps
                    } else if direction == -1 && frames < start {
                        self.spilled_time = (start - frames) / fps;
                        frames = start + (start - frames);
                        last = start / fps
                    } else {
                        break;
                    }
                    self.time = frames / fps;
                    self.direction *= -1.0;
                    direction *= -1;
                    looped = true;
                    if let Some(r) = reporter.as_deref_mut() {
                        self.with_animation(|animation| {
                            animation.report_keyed_callbacks(
                                r,
                                last,
                                self.time,
                                self.speed_direction,
                                from_pong,
                            )
                        })
                    }
                    from_pong = !from_pong
                }
            }
        }
        if kill {
            self.spilled_time = 0.0
        }
        self.did_loop = looped;
        self.keep_going_with_multiplier(elapsed)
    }
    pub fn is_translucent(&self) -> bool {
        self.artboard
            .with_artboard(|artboard| artboard.is_animation_instance_translucent(self))
            .unwrap_or(false)
    }
    pub fn report_event(&mut self, event: CoreHandle, _delay: f32) {
        self.nested_event_notifier.notify_listeners(&[event]);
    }
}
impl CallbackContext for LinearAnimationInstance {}
impl KeyFrameValueContext for LinearAnimationInstance {
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
        let artboard = self.artboard.clone();
        if let Some(target) = artboard
            .with_artboard(|artboard| artboard.base.resolve_handle(object_id))
            .flatten()
        {
            crate::mechanical_port::source::generated::core_registry::CoreRegistry::set_callback_handle(&target, property_key as i32, crate::mechanical_port::source::core::field_types::core_callback_type::CallbackData::new(Some(self), elapsed_seconds));
        }
    }
}
impl Clone for LinearAnimationInstance {
    fn clone(&self) -> Self {
        Self {
            animation: self.animation.clone(),
            artboard: self.artboard.clone(),
            nested_event_notifier: self.nested_event_notifier.clone(),
            time: self.time,
            speed_direction: self.speed_direction,
            total_time: self.total_time,
            last_total_time: self.last_total_time,
            spilled_time: self.spilled_time,
            direction: self.direction,
            did_loop: self.did_loop,
            loop_value: self.loop_value,
            scripted_interpolators: RefCell::new(None),
            cloned_artboard_data_binds: RefCell::new(Vec::new()),
            keyframe_value_holders: RefCell::new(None),
            keyframe_value_binds: RefCell::new(HashMap::new()),
            keyframe_bind_container: RefCell::new(DataBindContainerWeak::default()),
        }
    }
}
impl Drop for LinearAnimationInstance {
    fn drop(&mut self) {
        for bind in self.cloned_artboard_data_binds.get_mut().drain(..) {
            let _ = self
                .artboard
                .with_artboard_mut(|artboard| artboard.remove_data_bind(bind));
        }
        for bind in self
            .keyframe_value_binds
            .get_mut()
            .drain()
            .map(|(_, bind)| bind)
        {
            if let Some(container) = self.keyframe_bind_container.get_mut().upgrade() {
                container.remove_data_bind(bind.clone());
            }
            bind.remove_occurrence();
        }
        if let Some(scripted_interpolators) = self.scripted_interpolators.get_mut().take() {
            let _ = self.artboard.with_artboard_mut(|artboard| {
                for interpolator in scripted_interpolators.into_values() {
                    artboard.remove_runtime_object(interpolator);
                }
            });
        }
        if let Some(holders) = self.keyframe_value_holders.get_mut().take() {
            for holder in holders.into_values() {
                holder.remove_occurrence();
            }
        }
    }
}
