//! Direct Rust owner for pinned C++ `src/data_bind/converters/data_converter_interpolator.cpp`.

use crate::RuntimeTransitionInterpolator;
use crate::data_bind_graph::{RuntimeDataBindGraphStatefulAdvance, RuntimeDataBindGraphValue};
use crate::draw::color_lerp;

/// Occurrence-local interpolator state. Like C++, this owns both alternating
/// animation records and the currently presented value.
#[derive(Debug, Clone, Default)]
pub(crate) struct RuntimeDataConverterInterpolatorState {
    advance_count: u8,
    advancer: Option<RuntimeDataConverterInterpolatorAdvancer>,
}

impl RuntimeDataConverterInterpolatorState {
    pub(crate) fn new() -> Self {
        Self::default()
    }

    pub(crate) fn is_initialized(&self) -> bool {
        self.advancer.is_some()
    }

    pub(crate) fn convert(
        &mut self,
        duration: f32,
        _interpolator: Option<RuntimeTransitionInterpolator>,
        input: &RuntimeDataBindGraphValue,
    ) -> Option<RuntimeDataBindGraphValue> {
        if duration == 0.0
            && let Some(advancer) = &mut self.advancer
        {
            if let Some(input_value) = RuntimeDataConverterInterpolatorValue::from_graph(input) {
                advancer.reset_to_start(&input_value);
            }
            return Some(input.clone());
        }

        if self.advancer.is_none() {
            let Some(input_value) = RuntimeDataConverterInterpolatorValue::from_graph(input) else {
                return Some(input.clone());
            };
            self.advancer = Some(RuntimeDataConverterInterpolatorAdvancer::new(&input_value));
        }

        let Some(input_value) = RuntimeDataConverterInterpolatorValue::from_graph(input) else {
            return Some(input.clone());
        };
        let advancer = self.advancer.as_mut().expect("advancer initialized");
        if self.advance_count < 2 {
            advancer.reset_values(&input_value);
        } else {
            advancer.update_values(&input_value);
        }
        Some(advancer.current_value().to_graph_value())
    }

    pub(crate) fn advance(
        &mut self,
        duration: f32,
        interpolator: Option<RuntimeTransitionInterpolator>,
        elapsed_seconds: f32,
    ) -> RuntimeDataBindGraphStatefulAdvance {
        if self.advance_count < 2 && elapsed_seconds > 0.0 {
            self.advance_count += 1;
        }
        let Some(advancer) = &mut self.advancer else {
            return RuntimeDataBindGraphStatefulAdvance {
                changed: true,
                keep_going: true,
            };
        };
        advancer.advance(duration, interpolator, elapsed_seconds)
    }
}

#[derive(Debug, Clone)]
struct RuntimeDataConverterInterpolatorAdvancer {
    animation_data_a: RuntimeDataConverterInterpolatorAnimationData,
    animation_data_b: RuntimeDataConverterInterpolatorAnimationData,
    current_value: RuntimeDataConverterInterpolatorValue,
    is_smoothing_animation: bool,
}

impl RuntimeDataConverterInterpolatorAdvancer {
    fn new(input: &RuntimeDataConverterInterpolatorValue) -> Self {
        let default_value = input.default_for_kind();
        Self {
            animation_data_a: RuntimeDataConverterInterpolatorAnimationData::new(
                default_value.clone(),
            ),
            animation_data_b: RuntimeDataConverterInterpolatorAnimationData::new(
                default_value.clone(),
            ),
            current_value: default_value,
            is_smoothing_animation: false,
        }
    }

    fn current_value(&self) -> &RuntimeDataConverterInterpolatorValue {
        &self.current_value
    }

    fn reset_values(&mut self, input: &RuntimeDataConverterInterpolatorValue) {
        if self.is_smoothing_animation {
            self.animation_data_b.reset_values(input);
        } else {
            self.animation_data_a.reset_values(input);
        }
        self.current_value.copy_from(input);
    }

    fn reset_to_start(&mut self, input: &RuntimeDataConverterInterpolatorValue) {
        self.reset_values(input);
        self.is_smoothing_animation = false;
        self.animation_data_a.elapsed_seconds = 0.0;
        self.animation_data_b.elapsed_seconds = 0.0;
    }

    fn update_values(&mut self, input: &RuntimeDataConverterInterpolatorValue) {
        if self.current_animation_data().to == *input {
            return;
        }

        if self.current_animation_data().elapsed_seconds != 0.0 {
            if self.is_smoothing_animation {
                self.animation_data_a
                    .copy_from(&self.animation_data_b.clone());
            }
            self.is_smoothing_animation = true;
        } else {
            self.is_smoothing_animation = false;
        }

        let current_value = self.current_value.clone();
        let animation_data = self.current_animation_data_mut();
        animation_data.from.copy_from(&current_value);
        animation_data.to.copy_from(input);
        animation_data.elapsed_seconds = 0.0;
    }

    fn advance(
        &mut self,
        duration: f32,
        interpolator: Option<RuntimeTransitionInterpolator>,
        elapsed_seconds: f32,
    ) -> RuntimeDataBindGraphStatefulAdvance {
        let animation_index = self.current_animation_index();
        if self.animation_data(animation_index).to == self.current_value || elapsed_seconds == 0.0 {
            return RuntimeDataBindGraphStatefulAdvance::default();
        }

        let previous_value = self.current_value.clone();
        self.advance_animation_data(duration, interpolator, elapsed_seconds, animation_index);
        RuntimeDataBindGraphStatefulAdvance {
            changed: self.current_value != previous_value,
            keep_going: self.animation_data(animation_index).elapsed_seconds < duration,
        }
    }

    fn advance_animation_data(
        &mut self,
        duration: f32,
        interpolator: Option<RuntimeTransitionInterpolator>,
        elapsed_seconds: f32,
        animation_index: usize,
    ) {
        if self.is_smoothing_animation {
            let factor = interpolation_factor(
                duration,
                interpolator,
                self.animation_data_a.elapsed_seconds,
            );
            let interpolated = self.animation_data_a.interpolate(factor);
            self.animation_data_b.from.copy_from(&interpolated);
            if factor == 1.0 {
                self.animation_data_a
                    .copy_from(&self.animation_data_b.clone());
                self.is_smoothing_animation = false;
            } else {
                self.animation_data_a.elapsed_seconds += elapsed_seconds;
            }
        }

        if self.animation_data(animation_index).elapsed_seconds >= duration {
            self.current_value
                .copy_from(&self.animation_data(animation_index).to.clone());
            if self.is_smoothing_animation {
                self.is_smoothing_animation = false;
                self.animation_data_a
                    .copy_from(&self.animation_data_b.clone());
                self.animation_data_a.elapsed_seconds = 0.0;
                self.animation_data_b.elapsed_seconds = 0.0;
            } else {
                self.animation_data_a.elapsed_seconds = 0.0;
            }
            return;
        }

        self.animation_data_mut(animation_index).elapsed_seconds += elapsed_seconds;
        let factor = interpolation_factor(
            duration,
            interpolator,
            self.animation_data(animation_index).elapsed_seconds,
        );
        let interpolated = self.animation_data(animation_index).interpolate(factor);
        self.current_value.copy_from(&interpolated);
    }

    fn current_animation_data(&self) -> &RuntimeDataConverterInterpolatorAnimationData {
        self.animation_data(self.current_animation_index())
    }

    fn current_animation_data_mut(&mut self) -> &mut RuntimeDataConverterInterpolatorAnimationData {
        self.animation_data_mut(self.current_animation_index())
    }

    fn current_animation_index(&self) -> usize {
        usize::from(self.is_smoothing_animation)
    }

    fn animation_data(&self, index: usize) -> &RuntimeDataConverterInterpolatorAnimationData {
        if index == 0 {
            &self.animation_data_a
        } else {
            &self.animation_data_b
        }
    }

    fn animation_data_mut(
        &mut self,
        index: usize,
    ) -> &mut RuntimeDataConverterInterpolatorAnimationData {
        if index == 0 {
            &mut self.animation_data_a
        } else {
            &mut self.animation_data_b
        }
    }
}

#[derive(Debug, Clone)]
struct RuntimeDataConverterInterpolatorAnimationData {
    elapsed_seconds: f32,
    from: RuntimeDataConverterInterpolatorValue,
    to: RuntimeDataConverterInterpolatorValue,
}

impl RuntimeDataConverterInterpolatorAnimationData {
    fn new(value: RuntimeDataConverterInterpolatorValue) -> Self {
        Self {
            elapsed_seconds: 0.0,
            from: value.clone(),
            to: value,
        }
    }

    fn reset_values(&mut self, input: &RuntimeDataConverterInterpolatorValue) {
        self.from.copy_from(input);
        self.to.copy_from(input);
    }

    fn copy_from(&mut self, source: &Self) {
        self.from.copy_from(&source.from);
        self.to.copy_from(&source.to);
        self.elapsed_seconds = source.elapsed_seconds;
    }

    fn interpolate(&self, factor: f32) -> RuntimeDataConverterInterpolatorValue {
        self.from.interpolate(&self.to, factor)
    }
}

#[derive(Debug, Clone, PartialEq)]
enum RuntimeDataConverterInterpolatorValue {
    Number(f32),
    Color(u32),
}

impl RuntimeDataConverterInterpolatorValue {
    fn from_graph(value: &RuntimeDataBindGraphValue) -> Option<Self> {
        match value {
            RuntimeDataBindGraphValue::Number(value) => Some(Self::Number(*value)),
            RuntimeDataBindGraphValue::Color(value) => Some(Self::Color(*value)),
            _ => None,
        }
    }

    fn default_for_kind(&self) -> Self {
        match self {
            Self::Number(_) => Self::Number(0.0),
            Self::Color(_) => Self::Color(0),
        }
    }

    fn copy_from(&mut self, source: &Self) {
        if std::mem::discriminant(self) == std::mem::discriminant(source) {
            *self = source.clone();
        }
    }

    fn interpolate(&self, to: &Self, factor: f32) -> Self {
        match (self, to) {
            (Self::Number(from), Self::Number(to)) => {
                Self::Number(*to * factor + *from * (1.0 - factor))
            }
            (Self::Color(from), Self::Color(to)) => Self::Color(color_lerp(*from, *to, factor)),
            _ => self.clone(),
        }
    }

    fn to_graph_value(&self) -> RuntimeDataBindGraphValue {
        match self {
            Self::Number(value) => RuntimeDataBindGraphValue::Number(*value),
            Self::Color(value) => RuntimeDataBindGraphValue::Color(*value),
        }
    }
}

fn interpolation_factor(
    duration: f32,
    interpolator: Option<RuntimeTransitionInterpolator>,
    elapsed_seconds: f32,
) -> f32 {
    let mut factor = if duration > 0.0 {
        f32::min(1.0, elapsed_seconds / duration)
    } else {
        1.0
    };
    if let Some(interpolator) = interpolator {
        factor = interpolator.transform(factor);
    }
    factor
}
