use crate::mechanical_port::source::data_bind::data_values::{
    data_type::DataType, data_value::DataValue, data_value_color::DataValueColor,
    data_value_number::DataValueNumber,
};
pub trait KeyFrameInterpolator {
    fn transform(&self, value: f32) -> f32;
}
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
            if self.current().elapsed_seconds != 0.0 {
                if self.is_smoothing {
                    let mut old = core::mem::take(&mut self.animation_a);
                    old.copy(&self.animation_b);
                    self.animation_a = old;
                }
                self.is_smoothing = true
            } else {
                self.is_smoothing = false
            }
            let current_value = self.current_value.as_ref().unwrap();
            current_value.copy_value(self.current_mut().from.as_deref_mut());
            input.copy_value(self.current_mut().to.as_deref_mut());
            self.current_mut().elapsed_seconds = 0.0
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
    interpolator: Option<*mut dyn KeyFrameInterpolator>,
    duration: f32,
    output: Option<Box<dyn DataValue>>,
    advance_count: u8,
    advancer: InterpolatorAdvancer,
    dirty: bool,
}
impl DataConverterInterpolator {
    pub fn new(duration: f32) -> Self {
        Self {
            interpolator: None,
            duration,
            output: None,
            advance_count: 0,
            advancer: InterpolatorAdvancer::default(),
            dirty: false,
        }
    }
    pub fn output_type(&self) -> DataType {
        DataType::Input
    }
    pub fn set_interpolator(&mut self, value: Option<*mut dyn KeyFrameInterpolator>) {
        self.interpolator = value
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
        if previous < self.duration {
            self.dirty = true;
        }
        self.advancer.current().elapsed_seconds < self.duration
    }
    fn advance_animation_data(&mut self, elapsed: f32) {
        let transform = |mut f: f32, interpolator: Option<*mut dyn KeyFrameInterpolator>| {
            if let Some(interpolator) = interpolator {
                f = unsafe { (&*interpolator).transform(f) }
            }
            f
        };
        if self.advancer.is_smoothing {
            let mut f = (if self.duration > 0.0 {
                self.advancer.animation_a.elapsed_seconds / self.duration
            } else {
                1.0
            })
            .min(1.0);
            f = transform(f, self.interpolator);
            self.advancer
                .animation_a
                .interpolate(f, self.advancer.animation_b.from.as_deref_mut().unwrap());
            if f == 1.0 {
                let mut a = core::mem::take(&mut self.advancer.animation_a);
                a.copy(&self.advancer.animation_b);
                self.advancer.animation_a = a;
                self.advancer.is_smoothing = false
            } else {
                self.advancer.animation_a.elapsed_seconds += elapsed;
            }
        }
        let current = self.advancer.current_mut();
        if current.elapsed_seconds >= self.duration {
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
        let mut f = (if self.duration > 0.0 {
            current.elapsed_seconds / self.duration
        } else {
            1.0
        })
        .min(1.0);
        f = transform(f, self.interpolator);
        current.interpolate(f, self.advancer.current_value.as_deref_mut().unwrap());
    }
    pub fn convert<'a>(&'a mut self, input: &'a dyn DataValue) -> &'a dyn DataValue {
        if self.duration == 0.0 && self.advancer.initialized {
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
        self.interpolator = other.interpolator;
        self.duration = other.duration
    }
    pub fn duration_changed(&mut self) {
        self.dirty = true
    }
}
