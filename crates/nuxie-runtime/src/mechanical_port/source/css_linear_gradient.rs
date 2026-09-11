//! Layout-dependent CSS gradient geometry. Authored stops stay unresolved so a
//! resize can change pixel/percentage ordering without losing the original data.

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum CssGradientDirection {
    /// Clockwise degrees from upward; CSS default is 180 degrees.
    Degrees(f32),
    Corner {
        right: bool,
        bottom: bool,
    },
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum CssGradientPosition {
    Pixels(f32),
    /// Percentage points, not a fraction.
    Percent(f32),
}

#[derive(Clone, Debug, PartialEq)]
pub struct CssGradientLine {
    pub start: [f64; 2],
    pub end: [f64; 2],
    pub length: f64,
}

impl CssGradientDirection {
    pub fn resolve(self, width: f32, height: f32) -> Option<CssGradientLine> {
        if ![width, height]
            .into_iter()
            .all(|v| v.is_finite() && v >= 0.)
        {
            return None;
        }
        let (w, h) = (f64::from(width), f64::from(height));
        let (dx, dy) = match self {
            Self::Degrees(degrees) if degrees.is_finite() => {
                // Exact cardinal directions avoid tiny artificial gradient lengths.
                match degrees.rem_euclid(360.) {
                    0. => (0., -1.),
                    90. => (1., 0.),
                    180. => (0., 1.),
                    270. => (-1., 0.),
                    angle => {
                        let angle = f64::from(angle).to_radians();
                        (angle.sin(), -angle.cos())
                    }
                }
            }
            Self::Degrees(_) => return None,
            Self::Corner { right, bottom } => {
                // Perpendicular to the diagonal joining the neighboring corners.
                let magnitude = w.hypot(h);
                if magnitude == 0. {
                    (0., 1.)
                } else {
                    (
                        h / magnitude * if right { 1. } else { -1. },
                        w / magnitude * if bottom { 1. } else { -1. },
                    )
                }
            }
        };
        let length = (w * dx).abs() + (h * dy).abs();
        Some(CssGradientLine {
            start: [w / 2. - dx * length / 2., h / 2. - dy * length / 2.],
            end: [w / 2. + dx * length / 2., h / 2. + dy * length / 2.],
            length,
        })
    }
}

/// Distances from the gradient start, retaining duplicate and out-of-range
/// positions. Consumers must not clamp these to the nominal gradient line.
/// Color-stop expansion and resource limits are the caller's responsibility.
pub fn resolve_stop_positions(
    positions: &[Option<CssGradientPosition>],
    length: f64,
) -> Option<Vec<f64>> {
    if positions.len() < 2 || !length.is_finite() || length < 0. {
        return None;
    }
    let mut resolved = Vec::with_capacity(positions.len());
    for position in positions {
        let value = match position {
            None => None,
            Some(CssGradientPosition::Pixels(v)) if v.is_finite() => Some(f64::from(*v)),
            Some(CssGradientPosition::Percent(v)) if v.is_finite() => {
                Some(f64::from(*v) / 100. * length)
            }
            _ => return None,
        };
        if value.is_some_and(|v| !v.is_finite()) {
            return None;
        }
        resolved.push(value);
    }
    let last = resolved.len() - 1;
    resolved[0].get_or_insert(0.);
    resolved[last].get_or_insert(length);
    let mut previous = resolved[0].unwrap();
    for position in &mut resolved[1..] {
        if let Some(value) = position {
            *value = value.max(previous);
            previous = *value;
        }
    }
    let mut start = 0;
    while start < last {
        let end = (start + 1..=last).find(|&i| resolved[i].is_some()).unwrap();
        let a = resolved[start].unwrap();
        let b = resolved[end].unwrap();
        for (index, value) in resolved.iter_mut().enumerate().take(end).skip(start + 1) {
            let t = (index - start) as f64 / (end - start) as f64;
            *value = Some(a * (1. - t) + b * t);
        }
        start = end;
    }
    resolved.into_iter().collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use CssGradientPosition::{Percent as P, Pixels as X};

    #[test]
    fn cardinal_and_angle_geometry() {
        let right = CssGradientDirection::Degrees(90.)
            .resolve(200., 100.)
            .unwrap();
        assert_eq!(right.start, [0., 50.]);
        assert_eq!(right.end, [200., 50.]);
        assert_eq!(
            right,
            CssGradientDirection::Degrees(450.)
                .resolve(200., 100.)
                .unwrap()
        );
        let diagonal = CssGradientDirection::Degrees(45.)
            .resolve(200., 100.)
            .unwrap();
        for (actual, expected) in diagonal
            .start
            .into_iter()
            .chain(diagonal.end)
            .zip([25., 125., 175., -25.])
        {
            assert!((actual - expected).abs() < 1e-10);
        }
    }

    #[test]
    fn corner_midpoint_crosses_neighbors_after_resize() {
        for (w, h) in [(200., 100.), (100., 200.), (768., 140.)] {
            for right in [false, true] {
                for bottom in [false, true] {
                    let line = CssGradientDirection::Corner { right, bottom }
                        .resolve(w, h)
                        .unwrap();
                    let neighbor = [
                        if right { 0. } else { f64::from(w) },
                        if bottom { f64::from(h) } else { 0. },
                    ];
                    let d = [line.end[0] - line.start[0], line.end[1] - line.start[1]];
                    let projection = ((neighbor[0] - line.start[0]) * d[0]
                        + (neighbor[1] - line.start[1]) * d[1])
                        / line.length.powi(2);
                    assert!((projection - 0.5).abs() < 1e-12);
                }
            }
        }
    }

    #[test]
    fn stop_fixup_preserves_hard_stops_and_authored_values() {
        let positions = [Some(X(100.)), Some(P(50.))];
        assert_eq!(
            resolve_stop_positions(&positions, 150.).unwrap(),
            [100., 100.]
        );
        assert_eq!(
            resolve_stop_positions(&positions, 400.).unwrap(),
            [100., 200.]
        );
        assert_eq!(
            resolve_stop_positions(&positions, 150.).unwrap(),
            [100., 100.]
        );
        assert_eq!(
            resolve_stop_positions(&[Some(P(40.)), None, None, None], 100.).unwrap(),
            [40., 60., 80., 100.]
        );
        assert_eq!(
            resolve_stop_positions(&[Some(P(-50.)), None, Some(P(150.))], 100.).unwrap(),
            [-50., 50., 150.]
        );
        assert_eq!(
            resolve_stop_positions(&[None, Some(P(-50.)), Some(P(150.)), None], 100.).unwrap(),
            [0., 0., 150., 150.]
        );
    }

    #[test]
    fn degenerate_and_invalid_inputs() {
        assert_eq!(
            CssGradientDirection::Degrees(90.)
                .resolve(0., 100.)
                .unwrap()
                .length,
            0.
        );
        assert_eq!(
            CssGradientDirection::Corner {
                right: true,
                bottom: true
            }
            .resolve(0., 0.)
            .unwrap()
            .length,
            0.
        );
        assert_eq!(resolve_stop_positions(&[None, None], 0.).unwrap(), [0., 0.]);
        assert!(
            CssGradientDirection::Degrees(f32::NAN)
                .resolve(10., 10.)
                .is_none()
        );
        assert!(
            CssGradientDirection::Degrees(0.)
                .resolve(-1., 10.)
                .is_none()
        );
        assert!(resolve_stop_positions(&[Some(X(f32::INFINITY)), None], 10.).is_none());
        assert!(resolve_stop_positions(&[None], 10.).is_none());
    }
}

/// Checked authored paint values. Construction does not resolve against a box.
#[derive(Clone, Debug, PartialEq)]
pub struct CssLinearGradient {
    direction: CssGradientDirection,
    colors: Vec<u32>,
    positions: Vec<Option<CssGradientPosition>>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ResolvedCssLinearGradient {
    pub start: [f32; 2],
    pub end: [f32; 2],
    pub colors: Vec<u32>,
    pub positions: Vec<f32>,
}

impl CssLinearGradient {
    pub fn new(
        direction: CssGradientDirection,
        colors: Vec<u32>,
        positions: Vec<Option<CssGradientPosition>>,
    ) -> Option<Self> {
        if colors.len() != positions.len() || !(2..=256).contains(&colors.len()) {
            return None;
        }
        direction.resolve(1., 1.)?;
        resolve_stop_positions(&positions, 1.)?;
        Some(Self {
            direction,
            colors,
            positions,
        })
    }

    /// Resolve a positive-area gradient box, extending endpoints to preserve
    /// authored out-of-range stops while providing normalized shader positions.
    /// None means empty geometry or values not representable by the renderer.
    pub fn resolve(&self, width: f32, height: f32) -> Option<ResolvedCssLinearGradient> {
        let line = self.direction.resolve(width, height)?;
        if width == 0. || height == 0. || line.length == 0. {
            return None;
        }
        let positions = resolve_stop_positions(&self.positions, line.length)?;
        let low = positions[0].min(0.);
        let high = positions[positions.len() - 1].max(line.length);
        let span = high - low;
        let direction = [
            (line.end[0] - line.start[0]) / line.length,
            (line.end[1] - line.start[1]) / line.length,
        ];
        let start = std::array::from_fn(|i| (line.start[i] + direction[i] * low) as f32);
        let end = std::array::from_fn(|i| (line.start[i] + direction[i] * high) as f32);
        // A finite percentage may resolve beyond f32 coordinates even on a
        // small box. Only the nominal line is visible (also within a repeated
        // image tile), so trim that unrepresentable exterior in f64 before
        // creating renderer coordinates. Leave ordinary transport unchanged.
        if !start.iter().chain(end.iter()).all(|v| v.is_finite()) {
            return self.resolve_visible_domain(&line, &positions);
        }
        let mut positions: Vec<f32> = positions
            .iter()
            .map(|p| ((p - low) / span) as f32)
            .collect();
        if !start.iter().chain(end.iter()).all(|v| v.is_finite())
            || !positions
                .iter()
                .all(|v| v.is_finite() && (0. ..=1.).contains(v))
        {
            return None;
        }
        // Keep the constant exterior regions explicit. Native gradient endpoint
        // normalization otherwise loses the first color at coincident leading
        // stops. These extra stops are exactly equivalent, not an approximation.
        let mut colors = self.colors.clone();
        if positions[0] > 0. {
            positions.insert(0, 0.);
            colors.insert(0, colors[0]);
        }
        if *positions.last().unwrap() < 1. {
            positions.push(1.);
            colors.push(*colors.last().unwrap());
        }
        Some(ResolvedCssLinearGradient {
            start,
            end,
            colors,
            positions,
        })
    }

    fn resolve_visible_domain(&self, line: &CssGradientLine, stops: &[f64]) -> Option<ResolvedCssLinearGradient> {
        let start = line.start.map(|v| v as f32);
        let end = line.end.map(|v| v as f32);
        if !start.iter().chain(end.iter()).all(|v| v.is_finite()) { return None; }
        let sample = |distance: f64| {
            let upper = stops.partition_point(|position| *position <= distance);
            if upper == 0 { return self.colors[0]; }
            if upper == stops.len() { return *self.colors.last().unwrap(); }
            let lo = upper - 1;
            let t = (distance - stops[lo]) / (stops[upper] - stops[lo]);
            let a = self.colors[lo];
            let b = self.colors[upper];
            let alpha_a = (a >> 24) as f64;
            let alpha_b = (b >> 24) as f64;
            let alpha = alpha_a * (1. - t) + alpha_b * t;
            let mut color = (alpha.round() as u32) << 24;
            if alpha > 0. {
                for shift in [16, 8, 0] {
                    let channel = (((a >> shift) & 255) as f64 * alpha_a * (1. - t)
                        + ((b >> shift) & 255) as f64 * alpha_b * t) / alpha;
                    color |= (channel.round() as u32) << shift;
                }
            }
            color
        };
        let mut colors = vec![sample(0.)];
        let mut positions = vec![0.];
        for (&distance, &color) in stops.iter().zip(&self.colors) {
            if (0. ..=line.length).contains(&distance) {
                colors.push(color);
                positions.push((distance / line.length) as f32);
            }
        }
        // Duplicate endpoints retain the final coincident-stop selection.
        colors.push(sample(line.length));
        positions.push(1.);
        Some(ResolvedCssLinearGradient { start, end, colors, positions })
    }

}

#[cfg(test)]
mod paint_tests {
    use super::*;
    #[test]
    fn extends_domain_instead_of_clamping_colors_at_box_edges() {
        let paint = CssLinearGradient::new(
            CssGradientDirection::Degrees(90.),
            vec![0xffff0000, 0xff0000ff],
            vec![
                Some(CssGradientPosition::Percent(-50.)),
                Some(CssGradientPosition::Percent(150.)),
            ],
        )
        .unwrap();
        let resolved = paint.resolve(200., 100.).unwrap();
        assert_eq!(resolved.start, [-100., 50.]);
        assert_eq!(resolved.end, [300., 50.]);
        assert_eq!(resolved.positions, [0., 1.]);
        // Left and right box edges retain 25% and 75% ramp colors.
        assert_eq!(
            (0. - resolved.start[0]) / (resolved.end[0] - resolved.start[0]),
            0.25
        );
        assert_eq!(
            (200. - resolved.start[0]) / (resolved.end[0] - resolved.start[0]),
            0.75
        );
    }
    #[test]
    fn cloned_paint_keeps_original_stops_across_repeated_resizes() {
        let paint = CssLinearGradient::new(
            CssGradientDirection::Degrees(90.),
            vec![0xffff0000, 0xff0000ff],
            vec![
                Some(CssGradientPosition::Pixels(100.)),
                Some(CssGradientPosition::Percent(50.)),
            ],
        )
        .unwrap();
        let clone = paint.clone();
        for width in [150., 400., 150.] {
            let resolved = paint.resolve(width, 100.).unwrap();
            assert_eq!(Some(resolved.clone()), clone.resolve(width, 100.));
            if width == 150. {
                assert_eq!(resolved.positions[1], resolved.positions[2]);
            } else {
                assert_eq!(resolved.positions, [0., 0.25, 0.5, 1.]);
            }
        }
    }
    #[test]
    fn checks_lengths_stop_limits_and_empty_geometry() {
        let direction = CssGradientDirection::Degrees(0.);
        assert!(CssLinearGradient::new(direction, vec![0], vec![None]).is_none());
        assert!(CssLinearGradient::new(direction, vec![0, 1], vec![None]).is_none());
        assert!(CssLinearGradient::new(direction, vec![0; 257], vec![None; 257]).is_none());
        let paint = CssLinearGradient::new(direction, vec![0, 1], vec![None, None]).unwrap();
        assert!(paint.resolve(0., 100.).is_none());
        assert!(paint.resolve(f32::INFINITY, 100.).is_none());
    }
}

#[cfg(test)]
mod exterior_tests {
    use super::*;
    #[test]
    fn preserves_first_color_before_coincident_leading_stops() {
        let p = CssLinearGradient::new(
            CssGradientDirection::Degrees(90.),
            vec![0xffff0000, 0xff00ff00, 0xff0000ff],
            vec![
                Some(CssGradientPosition::Percent(70.)),
                Some(CssGradientPosition::Percent(30.)),
                None,
            ],
        )
        .unwrap();
        let r = p.resolve(200., 140.).unwrap();
        assert_eq!(r.positions, [0., 0.7, 0.7, 1.]);
        assert_eq!(r.colors, [0xffff0000, 0xffff0000, 0xff00ff00, 0xff0000ff]);
    }
}


#[cfg(test)]
mod extreme_stop_tests {
    use super::*;
    fn paint(colors: Vec<u32>, stops: &[f32]) -> CssLinearGradient {
        CssLinearGradient::new(CssGradientDirection::Degrees(90.), colors,
            stops.iter().map(|p| Some(CssGradientPosition::Percent(*p))).collect()).unwrap()
    }
    #[test]
    fn finite_extreme_percentages_preserve_visible_colors_across_resize() {
        for width in [240., 390., 768., 240.] {
            for (stops, expected) in [([0., 3e38], 0xffff0000), ([-3e38, 100.], 0xff0000ff)] {
                let resolved = paint(vec![0xffff0000, 0xff0000ff], &stops).resolve(width, 140.).unwrap();
                assert!(resolved.start.iter().chain(&resolved.end).all(|v| v.is_finite()));
                assert!(resolved.colors.iter().all(|color| *color == expected));
            }
        }
    }
    #[test]
    fn exterior_trimming_retains_interior_discontinuities_and_premultiplies_alpha() {
        let alpha = paint(vec![0x40ff0000, 0xc00000ff], &[-3e38, 3e38]).resolve(390., 140.).unwrap();
        assert_eq!(alpha.colors, [0x804000bf, 0x804000bf]);
        let hard = paint(vec![0xffff0000, 0xffff0000, 0xff0000ff, 0xff0000ff],
            &[-3e38, 50., 50., 3e38]).resolve(390., 140.).unwrap();
        assert_eq!(hard.positions, [0., 0.5, 0.5, 1.]);
        assert_eq!(hard.colors, [0xffff0000, 0xffff0000, 0xff0000ff, 0xff0000ff]);
    }
}
