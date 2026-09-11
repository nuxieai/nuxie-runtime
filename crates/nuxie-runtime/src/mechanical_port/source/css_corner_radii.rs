//! Checked, unresolved CSS corner values. Resolving does not mutate authored
//! values: each resize uses the current border-box width and height.

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum CssRadiusValue {
    Pixels(f32),
    /// CSS percentage points: 50 means 50%, not a fraction of 0.5.
    Percent(f32),
}

impl CssRadiusValue {
    fn valid(self) -> bool {
        let (Self::Pixels(value) | Self::Percent(value)) = self;
        value.is_finite() && value >= 0.
    }

    fn resolve(self, extent: f32) -> Option<f32> {
        let value = match self {
            Self::Pixels(value) => value as f64,
            Self::Percent(value) => value as f64 * extent as f64 / 100.,
        };
        let value = value as f32;
        value.is_finite().then_some(value)
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CssCornerRadii {
    /// TL/TR/BR/BL, each horizontal then vertical.
    corners: [[CssRadiusValue; 2]; 4],
}

impl CssCornerRadii {
    pub fn new(corners: [[CssRadiusValue; 2]; 4]) -> Option<Self> {
        corners.iter().flatten().all(|v| v.valid()).then_some(Self { corners })
    }

    /// Resolve before the shared path geometry applies overlap reduction.
    /// Invalid dimensions and overflow reject instead of producing corrupt paths.
    pub fn resolve(&self, width: f32, height: f32) -> Option<[[f32; 2]; 4]> {
        if ![width, height].into_iter().all(|v| v.is_finite() && v >= 0.) {
            return None;
        }
        let mut radii = [[0.; 2]; 4];
        for (output, input) in radii.iter_mut().zip(self.corners) {
            output[0] = input[0].resolve(width)?;
            output[1] = input[1].resolve(height)?;
        }
        Some(radii)
    }

    /// Resolve for painting, reducing overlapping corners in wider arithmetic
    /// only when raw values or their edge sums cannot be represented in f32.
    /// Percentage bases and painted extents are distinct because CSS paint
    /// bounds may be snapped while layout dimensions remain fractional.
    pub fn resolve_for_paint(&self, width: f32, height: f32, paint_width: f32, paint_height: f32) -> Option<[[f32; 2]; 4]> {
        if ![width, height, paint_width, paint_height].into_iter().all(|v| v.is_finite() && v >= 0.) {
            return None;
        }
        if let Some(r) = self.resolve(width, height) {
            if [r[0][0]+r[1][0], r[3][0]+r[2][0], r[0][1]+r[3][1], r[1][1]+r[2][1]]
                .into_iter().all(f32::is_finite) { return Some(r); }
        }
        let mut radii = [[0_f64; 2]; 4];
        for (output, input) in radii.iter_mut().zip(self.corners) {
            for axis in 0..2 {
                output[axis] = match input[axis] {
                    CssRadiusValue::Pixels(value) => value as f64,
                    CssRadiusValue::Percent(value) => value as f64 * [width, height][axis] as f64 / 100.,
                };
            }
        }
        let mut factor = 1_f64;
        for (extent, sum) in [
            (paint_width, radii[0][0]+radii[1][0]),
            (paint_width, radii[3][0]+radii[2][0]),
            (paint_height, radii[0][1]+radii[3][1]),
            (paint_height, radii[1][1]+radii[2][1]),
        ] {
            if sum > extent as f64 { factor = factor.min(extent as f64 / sum); }
        }
        Some(radii.map(|pair| pair.map(|value| (value * factor) as f32)))
    }
}

#[cfg(test)]
mod tests {
    use super::{CssCornerRadii, CssRadiusValue::{Pixels, Percent}};

    #[test]
    fn paint_resolution_reduces_overflow_proportionally() {
        let percentages = CssCornerRadii::new([[Percent(f32::MAX);2];4]).unwrap();
        assert_eq!(percentages.resolve_for_paint(200.,80.,200.,80.).unwrap(), [[100.,40.];4]);
        assert_eq!(percentages.resolve_for_paint(100.,160.,100.,160.).unwrap(), [[50.,80.];4]);
        let pixels = CssCornerRadii::new([[Pixels(f32::MAX);2];4]).unwrap();
        // Individual values fit, but their sums do not. Height constrains both axes.
        assert_eq!(pixels.resolve_for_paint(200.,80.,200.,80.).unwrap(), [[40.,40.];4]);
        assert_eq!(percentages.resolve_for_paint(200.,80.,100.,80.).unwrap(), [[50.,20.];4]);
        assert_eq!(percentages.resolve_for_paint(0.,80.,0.,80.).unwrap(), [[0.,40.];4]);
        assert!(percentages.resolve_for_paint(200.,80.,f32::NAN,80.).is_none());
    }

    #[test]
    fn percentages_resolve_per_axis_again_after_resize_and_clone() {
        let authored = [[Percent(50.), Percent(25.)], [Pixels(12.), Percent(80.)],
            [Percent(120.), Pixels(8.)], [Pixels(0.), Percent(100.)]];
        let radii = CssCornerRadii::new(authored).unwrap();
        let clone = radii;
        assert_eq!(radii.resolve(200., 80.).unwrap(),
            [[100.,20.], [12.,64.], [240.,8.], [0.,80.]]);
        assert_eq!(clone.resolve(100., 160.).unwrap(),
            [[50.,40.], [12.,128.], [120.,8.], [0.,160.]]);
        assert_eq!(radii.resolve(200., 80.), clone.resolve(200., 80.));
        assert_eq!(radii.corners, authored);
    }

    #[test]
    fn invalid_values_dimensions_and_overflow_reject() {
        for invalid in [-1., f32::NAN, f32::INFINITY] {
            for value in [Pixels(invalid), Percent(invalid)] {
                assert!(CssCornerRadii::new([[value;2];4]).is_none());
            }
            let radii = CssCornerRadii::new([[Percent(50.);2];4]).unwrap();
            assert!(radii.resolve(invalid, 100.).is_none());
            assert!(radii.resolve(100., invalid).is_none());
        }
        let radii = CssCornerRadii::new([[Percent(f32::MAX);2];4]).unwrap();
        assert!(radii.resolve(f32::MAX, 1.).is_none());
        assert_eq!(radii.resolve(0., 0.).unwrap(), [[0.;2];4]);
    }
}
