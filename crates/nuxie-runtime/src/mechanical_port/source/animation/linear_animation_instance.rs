use crate::mechanical_port::source::{
    animation::{
        interpolating_keyframe::KeyFrameValueContext,
        keyed_callback_reporter::KeyedCallbackReporter, linear_animation::LinearAnimation,
        linear_animation_instance_extras::LAIBindingExtras, r#loop::Loop,
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
    scripted::scripted_interpolator::ScriptedInterpolator,
};
use std::{
    cell::{RefCell, RefMut},
    rc::Rc,
};

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
    blend_accumulator: std::rc::Weak<RefCell<super::blend_accumulator::BlendAccumulator>>,
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
            blend_accumulator: std::rc::Weak::new(),
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
        self.ensure_binding_extras()
            .keyframe_value_holders
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
            .artboard
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
            self.artboard
                .with_artboard_mut(|artboard| artboard.add_data_bind(bind))
                .expect("the animation retains its artboard while cloning an interpolator");
        })?;
        let context = self
            .artboard
            .with_artboard(|artboard| artboard.data_context())?;
        self.ensure_binding_extras().bind_container = self
            .artboard
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
            // Upstream's explicit copy constructor leaves this pointer null.
            blend_accumulator: std::rc::Weak::new(),
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
