use crate::mechanical_port::source::{
    core_context::CoreContext,
    generated::animation::cubic_value_interpolator_base::CubicValueInterpolatorBase,
    math::FloatContract, status_code::StatusCode,
};
pub struct CubicValueInterpolator {
    pub base: CubicValueInterpolatorBase,
    a: f32,
    b: f32,
    c: f32,
    d: f32,
    value_to: f32,
}
impl Default for CubicValueInterpolator {
    fn default() -> Self {
        let mut value = Self {
            base: CubicValueInterpolatorBase::default(),
            a: 0.0,
            b: 0.0,
            c: 0.0,
            d: 0.0,
            value_to: 0.0,
        };
        value.compute_parameters();
        value
    }
}
impl CubicValueInterpolator {
    pub fn new() -> Self {
        Self::default()
    }
    fn compute_parameters(&mut self) {
        let y1 = self.d;
        let y2 = self.base.base.base.y1();
        let y3 = self.base.base.base.y2();
        let y4 = self.value_to;
        #[cfg(feature = "strict-fp")]
        {
            self.a = y4 + 3.0 * (y2 - y3) - y1;
            self.b = 3.0 * (y3 - y2 * 2.0 + y1);
        }
        #[cfg(not(feature = "strict-fp"))]
        {
            self.a = (y2 - y3).mul_add(3.0, y4) - y1;
            self.b = (y1 + y2.mul_add(-2.0, y3)) * 3.0;
        }
        self.c = 3.0 * (y2 - y1);
    }
    pub fn transform_value(&mut self, from: f32, to: f32, factor: f32) -> f32 {
        if self.d != from || self.value_to != to {
            self.d = from;
            self.value_to = to;
            self.compute_parameters();
        }
        let t = self.base.base.solver.get_t(factor);
        self.a
            .contracted_mul_add(t, self.b)
            .contracted_mul_add(t, self.c)
            .contracted_mul_add(t, self.d)
    }
    pub fn transform(&self, factor: f32) -> f32 {
        debug_assert!(false);
        factor
    }
    pub fn on_added_dirty(&mut self, context: &mut dyn CoreContext) -> StatusCode {
        self.compute_parameters();
        self.base.base.on_added_dirty(context)
    }
}

#[cfg(test)]
mod tests {
    use super::CubicValueInterpolator;

    #[test]
    fn parameter_refresh_and_horner_evaluation_use_the_selected_source_mode() {
        let mut interpolator = CubicValueInterpolator::new();
        assert_eq!(
            [interpolator.a, interpolator.b, interpolator.c],
            [-3.0, 3.0, 0.0]
        );
        let y2 = f32::from_bits(0x3ed7_0a3d);
        let y3 = f32::from_bits(0x3f14_7ae1);
        interpolator.base.base.base.set_y1_value(y2);
        interpolator.base.base.base.set_y2_value(y3);
        interpolator.base.base.initialize();
        let from = f32::from_bits(0x3d82_a90a);
        let to = f32::from_bits(0x3f33_3334);
        let factor = f32::from_bits(0x3ed0_f81d);
        let actual = interpolator.transform_value(from, to, factor);
        #[cfg(feature = "strict-fp")]
        let a = to + 3.0 * (y2 - y3) - from;
        #[cfg(feature = "strict-fp")]
        let b = 3.0 * (y3 - y2 * 2.0 + from);
        #[cfg(not(feature = "strict-fp"))]
        let a = (y2 - y3).mul_add(3.0, to) - from;
        #[cfg(not(feature = "strict-fp"))]
        let b = (from + y2.mul_add(-2.0, y3)) * 3.0;
        let c = 3.0 * (y2 - from);
        assert_eq!(
            [interpolator.a, interpolator.b, interpolator.c].map(f32::to_bits),
            [a, b, c].map(f32::to_bits)
        );
        let t = interpolator.base.base.solver.get_t(factor);
        #[cfg(feature = "strict-fp")]
        let expected = ((a * t + b) * t + c) * t + from;
        #[cfg(not(feature = "strict-fp"))]
        let expected = a.mul_add(t, b).mul_add(t, c).mul_add(t, from);
        assert_eq!(actual.to_bits(), expected.to_bits());
        assert_eq!(interpolator.d.to_bits(), from.to_bits());
        assert_eq!(interpolator.value_to.to_bits(), to.to_bits());
        assert_eq!(
            interpolator.transform_value(from, to, factor).to_bits(),
            expected.to_bits()
        );
    }
}
