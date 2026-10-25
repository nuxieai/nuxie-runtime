use std::rc::Rc;

use crate::mechanical_port::source::{
    core::CoreHandle,
    data_bind::{
        data_context::RuntimeDataContextHandle,
        data_values::{
            data_type::DataType, data_value::DataValue, data_value_color::DataValueColor,
            data_value_number::DataValueNumber,
        },
    },
    generated::data_bind::converters::data_converter_interpolator_base::{
        DataConverterInterpolatorBase, DataConverterInterpolatorBaseCallbacks,
    },
    scripted::scripted_interpolator::{ScriptedInterpolator, StatefulScriptedInterpolator},
};
fn new_like(value: &dyn DataValue) -> Option<Box<dyn DataValue>> {
    if value.as_any().is::<DataValueNumber>() {
        Some(Box::new(DataValueNumber::default()))
    } else if value.as_any().is::<DataValueColor>() {
        Some(Box::new(DataValueColor::default()))
    } else {
        None
    }
}
#[derive(Default)]
pub struct InterpolatorAnimationData {
    pub elapsed_seconds: f32,
    from: Option<Box<dyn DataValue>>,
    to: Option<Box<dyn DataValue>>,
}
impl InterpolatorAnimationData {
    fn initialize(&mut self, value: &dyn DataValue) {
        self.from = new_like(value);
        self.to = new_like(value)
    }
    fn interpolate(&self, f: f32, store: &mut dyn DataValue) {
        self.from
            .as_ref()
            .unwrap()
            .interpolate(self.to.as_deref(), Some(store), f)
    }
    fn copy(&mut self, source: &Self) {
        source
            .from
            .as_ref()
            .unwrap()
            .copy_value(self.from.as_deref_mut());
        source
            .to
            .as_ref()
            .unwrap()
            .copy_value(self.to.as_deref_mut());
        self.elapsed_seconds = source.elapsed_seconds
    }
    fn dispose(&mut self) {
        self.elapsed_seconds = 0.0;
        self.from = None;
        self.to = None
    }
}
#[derive(Default)]
pub struct InterpolatorAdvancer {
    animation_a: InterpolatorAnimationData,
    animation_b: InterpolatorAnimationData,
    is_smoothing: bool,
    initialized: bool,
    current_value: Option<Box<dyn DataValue>>,
}
impl InterpolatorAdvancer {
    fn initialize(&mut self, value: &dyn DataValue) {
        self.initialized = true;
        self.animation_a.initialize(value);
        self.animation_b.initialize(value);
        self.current_value = new_like(value)
    }
    fn current(&self) -> &InterpolatorAnimationData {
        if self.is_smoothing {
            &self.animation_b
        } else {
            &self.animation_a
        }
    }
    fn current_mut(&mut self) -> &mut InterpolatorAnimationData {
        if self.is_smoothing {
            &mut self.animation_b
        } else {
            &mut self.animation_a
        }
    }
    fn reset_values(&mut self, input: &dyn DataValue) {
        input.copy_value(self.current_mut().from.as_deref_mut());
        input.copy_value(self.current_mut().to.as_deref_mut());
        input.copy_value(self.current_value.as_deref_mut())
    }
    fn reset_to_start(&mut self, input: &dyn DataValue) {
        self.reset_values(input);
        self.is_smoothing = false;
        self.animation_a.elapsed_seconds = 0.0;
        self.animation_b.elapsed_seconds = 0.0
    }
    fn update_values(&mut self, input: &dyn DataValue) {
        if !input.compare(self.current().to.as_deref()) {
            // Only smooth from an animation that is still in flight. One that
            // reached its target keeps its elapsed time because advance stops
            // once the value lands.
            if self.current().elapsed_seconds != 0.0
                && !self
                    .current_value
                    .as_ref()
                    .unwrap()
                    .compare(self.current().to.as_deref())
            {
                if self.is_smoothing {
                    let mut old = core::mem::take(&mut self.animation_a);
                    old.copy(&self.animation_b);
                    self.animation_a = old;
                }
                self.is_smoothing = true
            } else {
                self.is_smoothing = false
            }
            let current = if self.is_smoothing {
                &mut self.animation_b
            } else {
                &mut self.animation_a
            };
            self.current_value
                .as_ref()
                .unwrap()
                .copy_value(current.from.as_deref_mut());
            input.copy_value(current.to.as_deref_mut());
            current.elapsed_seconds = 0.0
        }
    }
    fn copy_current_value(&self, output: &mut dyn DataValue) {
        self.current_value
            .as_ref()
            .unwrap()
            .copy_value(Some(output))
    }
    fn reset(&mut self) {
        self.animation_a.dispose();
        self.animation_b.dispose();
        self.current_value = None;
        self.is_smoothing = false;
        self.initialized = false
    }
}
pub struct DataConverterInterpolator {
    pub base: DataConverterInterpolatorBase,
    interpolator: Option<CoreHandle>,
    scripted_interpolator: Option<Rc<StatefulScriptedInterpolator>>,
    output: Option<Box<dyn DataValue>>,
    advance_count: u8,
    advancer: InterpolatorAdvancer,
}

impl Default for DataConverterInterpolator {
    fn default() -> Self {
        Self {
            base: DataConverterInterpolatorBase::default(),
            interpolator: None,
            scripted_interpolator: None,
            output: None,
            advance_count: 0,
            advancer: InterpolatorAdvancer::default(),
        }
    }
}

impl DataConverterInterpolator {
    pub fn interpolator(&self) -> Option<CoreHandle> {
        self.interpolator.clone()
    }

    pub fn active_interpolator(&self) -> Option<CoreHandle> {
        self.scripted_interpolator
            .as_ref()
            .and_then(|holder| holder.get())
            .or_else(|| self.interpolator.clone())
    }

    pub fn bind_from_context_handle(
        owner: &CoreHandle,
        context: RuntimeDataContextHandle,
        data_bind: Option<CoreHandle>,
    ) {
        super::data_converter::DataConverter::bind_from_context_handle(
            owner,
            context.clone(),
            data_bind,
        );
        let (source, holder) = owner
            .with_downcast::<Self, _>(|converter| {
                (
                    converter.interpolator.clone(),
                    converter.scripted_interpolator.clone(),
                )
            })
            .expect("live interpolator converter");
        if !source.as_ref().is_some_and(|source| {
            source
                .with_downcast::<ScriptedInterpolator, _>(|_| ())
                .is_some()
        }) {
            return;
        }
        let holder = holder.unwrap_or_else(|| {
            let holder = Rc::new(StatefulScriptedInterpolator::new_owned(
                source.as_ref(),
                Some(context.clone()),
            ));
            owner.with_downcast_mut::<Self, _>(|converter| {
                converter.scripted_interpolator = Some(holder.clone())
            });
            holder
        });
        holder.bind_from_context(Some(context));
    }

    pub fn unbind_handle(owner: &CoreHandle) {
        let _retained = owner.retain_arena();
        super::data_converter::DataConverter::unbind_base_handle(owner);
        let holder = owner
            .with_downcast::<Self, _>(|converter| converter.scripted_interpolator.clone())
            .flatten();
        if let Some(holder) = holder {
            holder.unbind();
        }
    }

    pub fn update_handle(owner: &CoreHandle) {
        super::data_converter::DataConverter::update_base_handle(owner);
        let holder = owner
            .with_downcast::<Self, _>(|converter| converter.scripted_interpolator.clone())
            .flatten();
        if let Some(holder) = holder {
            holder.update_data_binds();
        }
    }
    pub fn new(duration: f32) -> Self {
        let mut converter = Self::default();
        if converter.base.set_duration_value(duration) {
            DataConverterInterpolatorBaseCallbacks::duration_changed(&mut converter);
            crate::mechanical_port::source::core::CoreObject::core_mut(&mut converter)
                .notify_property_changed(DataConverterInterpolatorBase::DURATION_PROPERTY_KEY);
        }
        converter
    }
    pub fn output_type(&self) -> DataType {
        DataType::Input
    }
    pub fn interpolator_id(&self) -> u32 {
        self.base.interpolator_id()
    }
    pub fn set_interpolator(&mut self, value: Option<CoreHandle>) {
        self.interpolator = value
    }
    fn transform_factor(interpolator: Option<&CoreHandle>, factor: f32) -> f32 {
        interpolator
            .and_then(|interpolator| {
                interpolator
                    .with_mut(|interpolator| interpolator.keyframe_interpolator_transform(factor))
            })
            .flatten()
            .unwrap_or(factor)
    }
    pub fn may_advance(&self) -> bool {
        true
    }

    pub fn advance(&mut self, elapsed: f32) -> bool {
        if self.advance_count < 2 && elapsed > 0.0 {
            self.advance_count += 1;
        }
        if !self.advancer.initialized {
            return true;
        }
        let current_to_matches = self
            .advancer
            .current()
            .to
            .as_ref()
            .unwrap()
            .compare(self.advancer.current_value.as_deref());
        if current_to_matches || elapsed == 0.0 {
            return false;
        }
        let previous = self.advancer.current().elapsed_seconds;
        self.advance_animation_data(elapsed);
        if previous < self.base.duration() {
            self.base.base.mark_converter_dirty();
        }
        // advance_animation_data may have ended the smoothing animation.
        self.advancer.current().elapsed_seconds < self.base.duration()
    }
    fn advance_animation_data(&mut self, elapsed: f32) {
        let mut use_animation_b = self.advancer.is_smoothing;
        if self.advancer.is_smoothing {
            let mut f = (if self.base.duration() > 0.0 {
                self.advancer.animation_a.elapsed_seconds / self.base.duration()
            } else {
                1.0
            })
            .min(1.0);
            f = Self::transform_factor(self.active_interpolator().as_ref(), f);
            self.advancer
                .animation_a
                .interpolate(f, self.advancer.animation_b.from.as_deref_mut().unwrap());
            if f == 1.0 {
                let mut a = core::mem::take(&mut self.advancer.animation_a);
                a.copy(&self.advancer.animation_b);
                self.advancer.animation_a = a;
                self.advancer.is_smoothing = false;
                // B now lives in A, so keep advancing A rather than leaving
                // it one frame behind and repeating this frame's value.
                use_animation_b = self.advancer.is_smoothing;
            } else {
                self.advancer.animation_a.elapsed_seconds += elapsed;
            }
        }
        let current = if use_animation_b {
            &mut self.advancer.animation_b
        } else {
            &mut self.advancer.animation_a
        };
        if current.elapsed_seconds >= self.base.duration() {
            current
                .to
                .as_ref()
                .unwrap()
                .copy_value(self.advancer.current_value.as_deref_mut());
            if self.advancer.is_smoothing {
                self.advancer.is_smoothing = false;
                let mut a = core::mem::take(&mut self.advancer.animation_a);
                a.copy(&self.advancer.animation_b);
                self.advancer.animation_a = a;
                self.advancer.animation_a.elapsed_seconds = 0.0;
                self.advancer.animation_b.elapsed_seconds = 0.0
            } else {
                self.advancer.animation_a.elapsed_seconds = 0.0
            }
            return;
        }
        current.elapsed_seconds += elapsed;
        let mut f = (if self.base.duration() > 0.0 {
            current.elapsed_seconds / self.base.duration()
        } else {
            1.0
        })
        .min(1.0);
        f = Self::transform_factor(self.active_interpolator().as_ref(), f);
        let current = if use_animation_b {
            &self.advancer.animation_b
        } else {
            &self.advancer.animation_a
        };
        current.interpolate(f, self.advancer.current_value.as_deref_mut().unwrap());
    }
    pub fn convert<'a>(&'a mut self, input: &'a dyn DataValue) -> &'a dyn DataValue {
        if self.base.duration() == 0.0 && self.advancer.initialized {
            self.advancer.reset_to_start(input);
            return input;
        }
        if !self.advancer.initialized {
            let Some(output) = new_like(input) else {
                return input;
            };
            self.output = Some(output);
            self.advancer.initialize(input);
        }
        if (input.as_any().is::<DataValueNumber>() || input.as_any().is::<DataValueColor>())
            && self.output.is_some()
        {
            if self.advance_count < 2 {
                self.advancer.reset_values(input)
            } else {
                self.advancer.update_values(input)
            }
            self.advancer
                .copy_current_value(self.output.as_deref_mut().unwrap());
            return self.output.as_deref().unwrap();
        }
        input
    }
    pub fn reverse_convert<'a>(&'a mut self, input: &'a dyn DataValue) -> &'a dyn DataValue {
        self.convert(input)
    }
    pub fn reset(&mut self) {
        self.advance_count = 0;
        self.advancer.reset()
    }
    pub fn copy(&mut self, other: &Self) {
        self.interpolator = other.interpolator.clone();
        let mut base = std::mem::take(&mut self.base);
        base.copy(&other.base, self);
        self.base = base;
    }
    pub fn duration_changed(&mut self) {
        self.base.base.mark_converter_dirty()
    }
}

impl DataConverterInterpolatorBaseCallbacks for DataConverterInterpolator {
    fn notify_property_changed(&mut self, property_key: u16) {
        self.base
            .base
            .base
            .base
            .notify_property_changed(property_key);
    }

    fn duration_changed(&mut self) {
        Self::duration_changed(self);
    }
}

impl crate::mechanical_port::source::generated::core_registry::DataConverterCapability
    for DataConverterInterpolator
{
    fn convert(
        &mut self,
        input: &dyn DataValue,
        _data_bind: &CoreHandle,
        output: &mut dyn FnMut(&dyn DataValue),
    ) {
        output(Self::convert(self, input));
    }

    fn reverse_convert(
        &mut self,
        input: &dyn DataValue,
        _data_bind: &CoreHandle,
        output: &mut dyn FnMut(&dyn DataValue),
    ) {
        output(Self::reverse_convert(self, input));
    }

    fn output_type(&self) -> DataType {
        Self::output_type(self)
    }

    fn bind_context_handler(&self) -> crate::mechanical_port::source::data_bind::converters::data_converter::ConverterBindContextHandler{
        Self::bind_from_context_handle
    }

    fn unbind_handler(&self) -> super::data_converter::ConverterUnbindHandler {
        Self::unbind_handle
    }

    fn unbind(&mut self) {
        self.base.base.unbind();
        if let Some(holder) = &self.scripted_interpolator {
            holder.unbind();
        }
    }

    fn update(&mut self) {
        self.base.base.update();
        if let Some(holder) = &self.scripted_interpolator {
            holder.update_data_binds();
        }
    }

    fn reset(&mut self) {
        Self::reset(self);
    }

    fn advance(&mut self, elapsed: f32) -> bool {
        Self::advance(self, elapsed)
    }
    fn may_advance(&self) -> bool {
        true
    }
}

impl Drop for DataConverterInterpolator {
    fn drop(&mut self) {
        self.output = None;
        self.advancer.reset();
        self.scripted_interpolator = None;
    }
}
