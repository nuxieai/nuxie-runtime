//! Opt-in CSS decoration geometry; ordinary Rive text does not use this module.
use crate::source::math::{aabb::Aabb, path_types::PathVerb, raw_path::RawPath, vec2d::Vec2D};

/// Horizontal ink interval intersecting a glyph-space band, for CSS skip-ink.
/// The band can have zero height (a one-pixel CSS underline after insetting).
/// Non-finite coordinates or a reversed band do not produce an interval.
pub fn glyph_stripe_intercept(path: &RawPath, top: f32, bottom: f32) -> Option<(f32, f32)> {
    if !top.is_finite()
        || !bottom.is_finite()
        || top > bottom
        || path
            .points()
            .iter()
            .any(|point| !point.x.is_finite() || !point.y.is_finite())
    {
        return None;
    }
    let mut span = (f32::INFINITY, f32::NEG_INFINITY);
    let mut first = Vec2D::default();
    let mut last = first;
    for segment in path.segments() {
        match segment.verb {
            PathVerb::Move => {
                if let Some(point) = segment.points.first() {
                    first = *point;
                }
            }
            PathVerb::Line => add_line(segment.points, top, bottom, &mut span),
            PathVerb::Close => add_line(&[last, first], top, bottom, &mut span),
            PathVerb::Quad | PathVerb::Cubic => add_curve(segment.points, top, bottom, &mut span),
        }
        if let Some(point) = segment.points.last() {
            last = *point;
        }
    }
    (span.0 < span.1).then_some(span)
}

fn expand(span: &mut (f32, f32), x: f32) {
    span.0 = span.0.min(x);
    span.1 = span.1.max(x);
}

fn add_line(points: &[Vec2D], top: f32, bottom: f32, span: &mut (f32, f32)) {
    let [start, end] = points else {
        return;
    };
    for boundary in [top, bottom] {
        let t = (boundary - start.y) / (end.y - start.y);
        if (0.0..1.0).contains(&t) {
            expand(span, start.x + t * (end.x - start.x));
        }
    }
    for point in points {
        if top < point.y && point.y < bottom {
            expand(span, point.x);
        }
    }
}

// Chromium 153's Skia combines stripe-boundary crossings with control points
// strictly inside the stripe, producing one interval for the entire glyph.
// See validation/underline-research.md for the pinned source and qualification.
fn add_curve(points: &[Vec2D], top: f32, bottom: f32, span: &mut (f32, f32)) {
    for point in points {
        if top < point.y && point.y < bottom {
            expand(span, point.x);
        }
    }
    let ys: Vec<f64> = points.iter().map(|point| f64::from(point.y)).collect();
    let xs: Vec<f64> = points.iter().map(|point| f64::from(point.x)).collect();
    let Some(&initial) = ys.first() else {
        return;
    };
    if ys.iter().all(|value| *value == initial) {
        return;
    }
    let mut cuts = vec![0.0, 1.0];
    match *ys.as_slice() {
        [y0, y1, y2] => {
            let denominator = y0 - 2.0 * y1 + y2;
            if denominator != 0.0 {
                cuts.push((y0 - y1) / denominator);
            }
        }
        [y0, y1, y2, y3] => {
            let a = -y0 + 3.0 * y1 - 3.0 * y2 + y3;
            let b = 2.0 * (y0 - 2.0 * y1 + y2);
            let c = y1 - y0;
            if a == 0.0 {
                if b != 0.0 {
                    cuts.push(-c / b);
                }
            } else {
                let discriminant = b * b - 4.0 * a * c;
                if discriminant >= 0.0 {
                    let q = -0.5 * (b + discriminant.sqrt().copysign(b));
                    cuts.push(q / a);
                    if q != 0.0 {
                        cuts.push(c / q);
                    }
                }
            }
        }
        _ => return,
    }
    cuts.retain(|t| t.is_finite() && (0.0..=1.0).contains(t));
    cuts.sort_by(f64::total_cmp);
    cuts.dedup();
    // Each interval is monotonic in y, including degenerate quadratic cubics.
    // Solve there instead of flattening outlines with a scale-dependent error.
    for boundary in [f64::from(top), f64::from(bottom)] {
        let tolerance = ys.iter().fold(1.0_f64, |scale, y| scale.max(y.abs())) * 1e-12;
        for &t in &cuts {
            if (evaluate(&ys, t) - boundary).abs() <= tolerance {
                expand(span, evaluate(&xs, t) as f32);
            }
        }
        for interval in cuts.windows(2) {
            let &[mut left, mut right] = interval else {
                continue;
            };
            let left_value = evaluate(&ys, left) - boundary;
            let right_value = evaluate(&ys, right) - boundary;
            if left_value == 0.0
                || right_value == 0.0
                || left_value.is_sign_positive() == right_value.is_sign_positive()
            {
                continue;
            }
            for _ in 0..56 {
                let middle = (left + right) * 0.5;
                let value = evaluate(&ys, middle) - boundary;
                if value.is_sign_positive() == left_value.is_sign_positive() {
                    left = middle;
                } else {
                    right = middle;
                }
            }
            expand(span, evaluate(&xs, (left + right) * 0.5) as f32);
        }
    }
}

fn evaluate(points: &[f64], t: f64) -> f64 {
    let mix = |a: f64, b: f64| a * (1.0 - t) + b * t;
    match *points {
        [a, b, c] => mix(mix(a, b), mix(b, c)),
        [a, b, c, d] => mix(mix(mix(a, b), mix(b, c)), mix(mix(b, c), mix(c, d))),
        _ => 0.0,
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SkipInk {
    None,
    Auto,
    All,
}

/// Resolved solid underline in text-local CSS pixels. Metric/cascade resolution
/// belongs to the host; the runtime positions it relative to each line baseline.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ResolvedUnderline {
    color: u32,
    thickness: f32,
    offset: f32,
    skip_ink: SkipInk,
}
impl ResolvedUnderline {
    pub fn solid(color: u32, thickness: f32, offset: f32, skip_ink: SkipInk) -> Option<Self> {
        (thickness.is_finite() && thickness > 0.0 && offset.is_finite()).then_some(Self {
            color,
            thickness,
            offset,
            skip_ink,
        })
    }
    pub fn color(&self) -> u32 {
        self.color
    }
}

impl SkipInk {
    pub fn skips(self, character: char) -> bool {
        match self {
            Self::None => false,
            Self::All => true,
            Self::Auto => super::css_skip_ink::can_text_decoration_skip_ink(character),
        }
    }
}

/// Resolved solid strikethrough. The host resolves the top edge relative to
/// the decoration origin's baseline; the runtime applies it to each laid-out
/// line. Strikethrough never skips ink and must paint after glyphs.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ResolvedStrikethrough {
    stripe: ResolvedUnderline,
    line_baseline: f32,
}

impl ResolvedStrikethrough {
    pub fn solid(
        color: u32,
        thickness: f32,
        baseline_top: f32,
        line_baseline: f32,
    ) -> Option<Self> {
        if !line_baseline.is_finite() {
            return None;
        }
        ResolvedUnderline::solid(color, thickness, baseline_top, SkipInk::None).map(|stripe| Self {
            stripe,
            line_baseline,
        })
    }

    pub fn color(&self) -> u32 {
        self.stripe.color()
    }

    pub fn build_stripes(
        &self,
        lines: &[crate::source::text_engine::OrderedLine],
    ) -> Vec<StrikethroughStripe> {
        lines
            .iter()
            .flat_map(|line| {
                self.stripe
                    .build_stripes(std::slice::from_ref(line), &[])
                    .into_iter()
                    .map(|stripe| StrikethroughStripe {
                        bounds: stripe.bounds,
                        line_top: line.y() - self.line_baseline,
                        thickness: self.stripe.thickness,
                    })
            })
            .collect()
    }
}

/// Unsnapped stripe and layout line origin. Keep both until paint time so
/// fractional container placement cannot be baked into compiled geometry.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct StrikethroughStripe {
    pub bounds: Aabb,
    pub thickness: f32,
    pub line_top: f32,
}
impl StrikethroughStripe {
    /// CSS paint-space snapping before the host renderer's transform/DPR.
    /// `paint_offset_y` locates the text in the CSS scene's coordinate space.
    pub fn paint_bounds(&self, paint_offset_y: f32) -> Aabb {
        let line_top = self.line_top + paint_offset_y;
        let top = (line_top + 0.5).floor() + (self.bounds.min_y - self.line_top + 0.5).floor()
            - paint_offset_y;
        Aabb::new(
            self.bounds.min_x,
            top,
            self.bounds.max_x,
            top + self.thickness.floor().max(1.0),
        )
    }
}

impl ResolvedUnderline {
    /// Preserve the full stripe and floating-point exclusion rectangles. A
    /// renderer must apply exclusions as non-antialiased difference clips in
    /// device space; subtracting them from an antialiased path is only a fallback.
    pub fn build_stripes(
        &self,
        lines: &[crate::source::text_engine::OrderedLine],
        text: &[u32],
    ) -> Vec<UnderlineStripe> {
        let mut stripes = Vec::new();
        for line in lines {
            let start = line.glyph_line().start_x;
            let top = line.y() + self.offset;
            let bottom = top + self.thickness;
            let mut end = start;
            let mut gaps = Vec::new();
            for (run, index) in line.into_iter().take(line.decoration_glyph_count()) {
                let index = index as usize;
                let Some(&advance) = run.advances.get(index) else {
                    continue;
                };
                let eligible = match self.skip_ink {
                    SkipInk::None => false,
                    SkipInk::All => true,
                    SkipInk::Auto => run
                        .text_indices
                        .get(index)
                        .and_then(|index| text.get(*index as usize))
                        .and_then(|value| char::from_u32(*value))
                        .is_some_and(|character| self.skip_ink.skips(character)),
                };
                if eligible
                    && run.size.is_finite()
                    && run.size > 0.0
                    && let (Some(font), Some(&glyph), Some(offset)) =
                        (&run.font, run.glyphs.get(index), run.offsets.get(index))
                {
                    let inset = 0.5_f32.min(self.thickness / 2.0);
                    let band_top = (self.offset + inset - offset.y) / run.size;
                    let band_bottom = (self.offset + self.thickness - inset - offset.y) / run.size;
                    if let Some((left, right)) =
                        glyph_stripe_intercept(&font.get_path(glyph), band_top, band_bottom)
                    {
                        let padding = self.thickness.min(13.0);
                        gaps.push((
                            end + offset.x + left * run.size - padding,
                            end + offset.x + right * run.size + padding,
                        ));
                    }
                }
                end += advance;
            }
            if !start.is_finite()
                || !end.is_finite()
                || !top.is_finite()
                || !bottom.is_finite()
                || start >= end
            {
                continue;
            }
            let inset = 0.5_f32.min(self.thickness / 2.0);
            stripes.push(UnderlineStripe {
                bounds: Aabb::new(start, top, end, bottom),
                exclusions: gaps
                    .into_iter()
                    .map(|(left, right)| {
                        Aabb::new(left, top + inset - 1.0, right, bottom - inset + 1.0)
                    })
                    .filter(|rect| {
                        [rect.min_x, rect.min_y, rect.max_x, rect.max_y]
                            .iter()
                            .all(|value| value.is_finite())
                    })
                    .collect(),
            });
        }
        stripes
    }

    /// Antialiased geometric fallback retained until hard clipping is available.
    pub fn build_path(
        &self,
        lines: &[crate::source::text_engine::OrderedLine],
        text: &[u32],
    ) -> RawPath {
        let mut path = RawPath::default();
        for stripe in self.build_stripes(lines, text) {
            stripe.append_fallback(&mut path);
        }
        path
    }
}

/// One visual line and its unsnapped skip-ink rectangles, in text-local space.
/// Rectangles deliberately extend beyond the stripe; transforms are applied by
/// the renderer, after layout, rather than rounded into compiler output.
#[derive(Clone, Debug, PartialEq)]
pub struct UnderlineStripe {
    pub bounds: Aabb,
    pub exclusions: Vec<Aabb>,
}

impl UnderlineStripe {
    pub(super) fn append_fallback(&self, path: &mut RawPath) {
        use crate::source::math::path_types::PathDirection;
        let Aabb {
            min_x: start,
            min_y: top,
            max_x: end,
            max_y: bottom,
        } = self.bounds;
        let mut gaps: Vec<_> = self
            .exclusions
            .iter()
            .map(|rect| (rect.min_x, rect.max_x))
            .collect();
        gaps.sort_by(|left, right| left.0.total_cmp(&right.0));
        let mut cursor = start;
        for (left, right) in gaps {
            if right <= cursor {
                continue;
            }
            let stop = end.min(left);
            if cursor < stop {
                path.add_rect(
                    Aabb::new(cursor, top, stop, bottom),
                    PathDirection::Clockwise,
                );
            }
            cursor = cursor.max(right).min(end);
            if cursor >= end {
                break;
            }
        }
        if cursor < end {
            path.add_rect(
                Aabb::new(cursor, top, end, bottom),
                PathDirection::Clockwise,
            );
        }
    }
}
