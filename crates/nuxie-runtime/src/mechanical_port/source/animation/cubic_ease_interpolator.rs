use crate::mechanical_port::source::animation::cubic_interpolator_solver::CubicInterpolatorSolver;
use crate::mechanical_port::source::generated::animation::cubic_ease_interpolator_base::CubicEaseInterpolatorBase;

#[inline]
fn transformed_value(value_from: f32, value_to: f32, transformed_factor: f32) -> f32 {
    (value_to - value_from).mul_add(transformed_factor, value_from)
}

#[derive(Default)]
pub struct CubicEaseInterpolator {
    pub base: CubicEaseInterpolatorBase,
}

impl CubicEaseInterpolator {
    pub fn transform_value(&self, value_from: f32, value_to: f32, factor: f32) -> f32 {
        transformed_value(value_from, value_to, self.transform(factor))
    }

    pub fn transform(&self, factor: f32) -> f32 {
        CubicInterpolatorSolver::calc_bezier(
            self.base.base.solver.get_t(factor),
            self.base.y1(),
            self.base.y2(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::transformed_value;

    #[test]
    fn transformed_value_matches_pinned_fused_bits() {
        let value_from = f32::from_bits(0x3d82_a90a);
        let value_to = f32::from_bits(0x0000_0000);
        let transformed_factor = f32::from_bits(0x3ed0_f81d);

        assert_eq!(
            transformed_value(value_from, value_to, transformed_factor).to_bits(),
            0x3d1a_aa19
        );
    }
}
