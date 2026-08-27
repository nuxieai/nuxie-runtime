// Direct source-correspondence owner for pinned `src/math/contour_measure.cpp`.
#[derive(Debug, Clone)]
pub(crate) struct TrimContour {
    segments: Vec<TrimMeasuredSegment>,
    pub(crate) length: f32,
    pub(crate) is_closed: bool,
}

include!("path_measure.rs");
include!("bezier_utils.rs");
include!("raw_path_utils.rs");
include!("vec2d.rs");

fn cubic_measure_segment_count(points: [(f32, f32); 4], inv_tolerance: f32) -> u32 {
    wangs_cubic(points, inv_tolerance)
        .ceil()
        .ceil()
        .min(TRIM_CONTOUR_MAX_SEGMENTS as f32) as u32
}

fn quadratic_measure_segment_count(points: [(f32, f32); 3], inv_tolerance: f32) -> u32 {
    wangs_quadratic(points, inv_tolerance)
        .ceil()
        .min(TRIM_CONTOUR_MAX_SEGMENTS as f32) as u32
}

fn wangs_quadratic(points: [(f32, f32); 3], precision: f32) -> f32 {
    let x = -2.0 * points[1].0 + points[0].0 + points[2].0;
    let y = -2.0 * points[1].1 + points[0].1 + points[2].1;
    let length_term_pow2 = 4.0 / 64.0 * precision * precision;
    (x.mul_add(x, y * y) * length_term_pow2).sqrt().sqrt()
}

fn wangs_cubic(points: [(f32, f32); 4], precision: f32) -> f32 {
    let first = vector_length_squared((
        points[0].0 - 2.0 * points[1].0 + points[2].0,
        points[0].1 - 2.0 * points[1].1 + points[2].1,
    ));
    let second = vector_length_squared((
        points[1].0 - 2.0 * points[2].0 + points[3].0,
        points[1].1 - 2.0 * points[2].1 + points[3].1,
    ));
    let length_term_pow2 = 9.0 * 4.0 / 64.0 * precision * precision;
    (contour_cpp_std_max(first, second) * length_term_pow2)
        .sqrt()
        .sqrt()
}

fn contour_cpp_std_max(first: f32, second: f32) -> f32 {
    if first < second { second } else { first }
}

fn contour_cpp_std_min(first: f32, second: f32) -> f32 {
    if second < first { second } else { first }
}

fn quad_position_tangent(points: [(f32, f32); 3], t: f32) -> ((f32, f32), (f32, f32)) {
    let eval = EvalQuad::new(points);
    (
        eval.evaluate(t),
        normalized_vector((
            (eval.a.0 + eval.a.0).mul_add(t, eval.b.0),
            (eval.a.1 + eval.a.1).mul_add(t, eval.b.1),
        )),
    )
}

fn eval_cubic(points: [(f32, f32); 4], t: f32) -> (f32, f32) {
    bezier_utils_owner::EvalCubic::new(points).evaluate(t)
}

fn cubic_position_tangent(points: [(f32, f32); 4], t: f32) -> ((f32, f32), (f32, f32)) {
    if t == 0.0 {
        return (points[0], bezier_utils_owner::find_cubic_tan0(points));
    }
    if t == 1.0 {
        return (points[3], bezier_utils_owner::find_cubic_tan1(points));
    }

    let eval = bezier_utils_owner::EvalCubic::new(points);
    let tan = normalized_vector((
        (eval.a.0 * 3.0)
            .mul_add(t, eval.b.0 + eval.b.0)
            .mul_add(t, eval.c.0),
        (eval.a.1 * 3.0)
            .mul_add(t, eval.b.1 + eval.b.1)
            .mul_add(t, eval.c.1),
    ));
    (eval.evaluate(t), tan)
}

#[derive(Debug, Clone)]
pub struct RuntimeContourMeasure {
    contour: TrimContour,
}

#[derive(Debug, Clone)]
struct TrimMeasuredSegment {
    original_index: usize,
    kind: TrimSegmentKind,
    distance: f32,
    t: f32,
}

#[derive(Debug, Clone, Copy)]
enum TrimSegmentKind {
    Line {
        from: (f32, f32),
        to: (f32, f32),
    },
    Quad {
        p0: (f32, f32),
        p1: (f32, f32),
        p2: (f32, f32),
    },
    Cubic {
        p0: (f32, f32),
        p1: (f32, f32),
        p2: (f32, f32),
        p3: (f32, f32),
    },
}

impl TrimContour {
    pub(crate) fn from_commands(commands: &[RuntimePathCommand]) -> Vec<Self> {
        Self::from_commands_with_inv_tolerance(commands, TRIM_CONTOUR_DEFAULT_INV_TOLERANCE)
    }

    fn from_commands_with_inv_tolerance(
        commands: &[RuntimePathCommand],
        inv_tolerance: f32,
    ) -> Vec<Self> {
        let mut contours = Vec::new();
        let mut raw_segments = Vec::<TrimSegmentKind>::new();
        let mut start = None::<(f32, f32)>;
        let mut current = None::<(f32, f32)>;
        let mut is_closed = false;

        let finish_contour = |contours: &mut Vec<Self>,
                              raw_segments: &mut Vec<TrimSegmentKind>,
                              is_closed: &mut bool| {
            if let Some(contour) = Self::from_raw_segments(raw_segments, *is_closed, inv_tolerance)
            {
                contours.push(contour);
            }
            raw_segments.clear();
            *is_closed = false;
        };

        for command in commands {
            match *command {
                RuntimePathCommand::Move { x, y } => {
                    if !raw_segments.is_empty() {
                        finish_contour(&mut contours, &mut raw_segments, &mut is_closed);
                    }
                    start = Some((x, y));
                    current = Some((x, y));
                }
                RuntimePathCommand::Line { x, y } => {
                    let Some(from) = current else {
                        continue;
                    };
                    let to = (x, y);
                    if distance_squared(from, to) > 0.0 {
                        raw_segments.push(TrimSegmentKind::Line { from, to });
                    }
                    current = Some(to);
                }
                RuntimePathCommand::Cubic {
                    x1,
                    y1,
                    x2,
                    y2,
                    x3,
                    y3,
                } => {
                    let Some(p0) = current else {
                        continue;
                    };
                    let p1 = (x1, y1);
                    let p2 = (x2, y2);
                    let p3 = (x3, y3);
                    raw_segments.push(TrimSegmentKind::Cubic { p0, p1, p2, p3 });
                    current = Some(p3);
                }
                RuntimePathCommand::Close => {
                    if let (Some(from), Some(to)) = (current, start) {
                        if distance_squared(from, to) > 0.0 {
                            raw_segments.push(TrimSegmentKind::Line { from, to });
                        }
                        current = Some(to);
                    }
                    is_closed = true;
                }
            }
        }

        if !raw_segments.is_empty() {
            finish_contour(&mut contours, &mut raw_segments, &mut is_closed);
        }
        contours
    }

    fn from_raw_path_with_inv_tolerance(path: &RawPath, inv_tolerance: f32) -> Vec<Self> {
        let mut contours = Vec::new();
        let mut raw_segments = Vec::<TrimSegmentKind>::new();
        let mut start = None::<(f32, f32)>;
        let mut current = None::<(f32, f32)>;
        let mut is_closed = false;
        let mut point_index = 0usize;

        let finish_contour = |contours: &mut Vec<Self>,
                              raw_segments: &mut Vec<TrimSegmentKind>,
                              is_closed: &mut bool| {
            if let Some(contour) = Self::from_raw_segments(raw_segments, *is_closed, inv_tolerance)
            {
                contours.push(contour);
            }
            raw_segments.clear();
            *is_closed = false;
        };

        for verb in path.verbs() {
            match verb {
                RenderPathVerb::Move => {
                    let Some(point) = path.points().get(point_index) else {
                        break;
                    };
                    point_index += 1;
                    if !raw_segments.is_empty() {
                        finish_contour(&mut contours, &mut raw_segments, &mut is_closed);
                    }
                    start = Some((point.x, point.y));
                    current = start;
                }
                RenderPathVerb::Line => {
                    let Some(point) = path.points().get(point_index) else {
                        break;
                    };
                    point_index += 1;
                    let Some(from) = current else {
                        continue;
                    };
                    let to = (point.x, point.y);
                    if distance_squared(from, to) > 0.0 {
                        raw_segments.push(TrimSegmentKind::Line { from, to });
                    }
                    current = Some(to);
                }
                RenderPathVerb::Quad => {
                    let (Some(control), Some(point), Some(p0)) = (
                        path.points().get(point_index),
                        path.points().get(point_index + 1),
                        current,
                    ) else {
                        break;
                    };
                    point_index += 2;
                    let p1 = (control.x, control.y);
                    let p2 = (point.x, point.y);
                    raw_segments.push(TrimSegmentKind::Quad { p0, p1, p2 });
                    current = Some(p2);
                }
                RenderPathVerb::Cubic => {
                    let (Some(control_out), Some(control_in), Some(point), Some(p0)) = (
                        path.points().get(point_index),
                        path.points().get(point_index + 1),
                        path.points().get(point_index + 2),
                        current,
                    ) else {
                        break;
                    };
                    point_index += 3;
                    let p1 = (control_out.x, control_out.y);
                    let p2 = (control_in.x, control_in.y);
                    let p3 = (point.x, point.y);
                    raw_segments.push(TrimSegmentKind::Cubic { p0, p1, p2, p3 });
                    current = Some(p3);
                }
                RenderPathVerb::Close => {
                    if let (Some(from), Some(to)) = (current, start) {
                        if distance_squared(from, to) > 0.0 {
                            raw_segments.push(TrimSegmentKind::Line { from, to });
                        }
                        current = Some(to);
                    }
                    is_closed = true;
                }
            }
        }

        if !raw_segments.is_empty() {
            finish_contour(&mut contours, &mut raw_segments, &mut is_closed);
        }
        contours
    }

    fn from_raw_segments(
        raw_segments: &[TrimSegmentKind],
        is_closed: bool,
        inv_tolerance: f32,
    ) -> Option<Self> {
        let mut segments = Vec::new();
        let mut distance_so_far = 0.0;

        for (original_index, segment) in raw_segments.iter().copied().enumerate() {
            match segment {
                TrimSegmentKind::Line { from, to } => {
                    let length = distance(from, to);
                    if length == 0.0 {
                        continue;
                    }
                    distance_so_far += length;
                    segments.push(TrimMeasuredSegment {
                        original_index,
                        kind: segment,
                        distance: distance_so_far,
                        t: 1.0,
                    });
                }
                TrimSegmentKind::Quad { p0, p1, p2 } => {
                    let segment_count =
                        quadratic_measure_segment_count([p0, p1, p2], inv_tolerance);
                    if segment_count == 0 {
                        continue;
                    }
                    let dt = 1.0 / segment_count as f32;
                    let eval = EvalQuad::new([p0, p1, p2]);
                    let mut t = dt;
                    let mut previous = p0;
                    for _ in 1..segment_count {
                        let next = eval.evaluate(t);
                        distance_so_far += distance(previous, next);
                        segments.push(TrimMeasuredSegment {
                            original_index,
                            kind: segment,
                            distance: distance_so_far,
                            t: trim_contour_dot30_t(t),
                        });
                        previous = next;
                        t += dt;
                    }
                    distance_so_far += distance(previous, p2);
                    segments.push(TrimMeasuredSegment {
                        original_index,
                        kind: segment,
                        distance: distance_so_far,
                        t: 1.0,
                    });
                }
                TrimSegmentKind::Cubic { p0, p1, p2, p3 } => {
                    let segment_count =
                        cubic_measure_segment_count([p0, p1, p2, p3], inv_tolerance);
                    if segment_count == 0 {
                        continue;
                    }
                    let dt = 1.0 / segment_count as f32;
                    let mut t = dt;
                    let mut previous = p0;
                    for _ in 1..segment_count {
                        let next = eval_cubic([p0, p1, p2, p3], t);
                        distance_so_far += distance(previous, next);
                        segments.push(TrimMeasuredSegment {
                            original_index,
                            kind: segment,
                            distance: distance_so_far,
                            t: trim_contour_dot30_t(t),
                        });
                        previous = next;
                        t += dt;
                    }
                    distance_so_far += distance(previous, p3);
                    segments.push(TrimMeasuredSegment {
                        original_index,
                        kind: segment,
                        distance: distance_so_far,
                        t: 1.0,
                    });
                }
            }
        }

        (distance_so_far > 0.0 && !segments.is_empty()).then_some(Self {
            segments,
            length: distance_so_far,
            is_closed,
        })
    }

    pub(crate) fn get_segment(
        &self,
        start_distance: f32,
        end_distance: f32,
        commands: &mut Vec<RuntimePathCommand>,
        start_with_move: bool,
    ) {
        self.get_segment_into(start_distance, end_distance, commands, start_with_move);
    }

    fn get_segment_into<S: TrimSegmentSink>(
        &self,
        mut start_distance: f32,
        mut end_distance: f32,
        destination: &mut S,
        start_with_move: bool,
    ) {
        start_distance = contour_cpp_std_max(0.0, start_distance);
        end_distance = contour_cpp_std_min(self.length, end_distance);
        if start_distance >= end_distance {
            return;
        }

        let mut start_index = self.find_segment(start_distance);
        let end_index = self.find_segment(end_distance);
        let mut start_t = self.compute_t(start_index, start_distance);
        let end_t = self.compute_t(end_index, end_distance);

        if 1.0 - start_t < TRIM_CONTOUR_EPSILON && start_index < end_index {
            start_index += 1;
            start_t = 0.0;
        }

        let start = &self.segments[start_index];
        let end = &self.segments[end_index];
        if start.original_index == end.original_index {
            start
                .kind
                .extract(destination, start_t, end_t, start_with_move);
            return;
        }

        start
            .kind
            .extract(destination, start_t, 1.0, start_with_move);

        let mut original_index = start.original_index + 1;
        while original_index < end.original_index {
            if let Some(segment) = self.first_segment_for_original(original_index) {
                segment.kind.extract_full(destination);
            }
            original_index += 1;
        }

        end.kind.extract(destination, 0.0, end_t, false);
    }

    fn find_segment(&self, distance: f32) -> usize {
        let mut index = self
            .segments
            .iter()
            .position(|segment| segment.distance >= distance)
            .unwrap_or_else(|| self.segments.len() - 1);
        while self.segments[index].distance == 0.0 && index + 1 < self.segments.len() {
            index += 1;
        }
        index
    }

    fn compute_t(&self, index: usize, distance: f32) -> f32 {
        let segment = &self.segments[index];
        let mut previous_distance = 0.0;
        let mut previous_t = 0.0;
        if index > 0 {
            let previous = &self.segments[index - 1];
            previous_distance = previous.distance;
            if previous.original_index == segment.original_index {
                previous_t = previous.t;
            }
        }

        let denominator = segment.distance - previous_distance;
        if denominator == 0.0 {
            return previous_t;
        }
        let ratio = (distance - previous_distance) / denominator;
        previous_t
            .mul_add(1.0 - ratio, segment.t * ratio)
            .clamp(previous_t, segment.t)
    }

    fn first_segment_for_original(&self, original_index: usize) -> Option<&TrimMeasuredSegment> {
        self.segments
            .iter()
            .find(|segment| segment.original_index == original_index)
    }

    fn position_tangent_at_distance(&self, distance: f32) -> ((f32, f32), (f32, f32)) {
        let mut distance = distance;
        if distance > self.length {
            distance = self.length;
        }
        if distance < 0.0 {
            distance = 0.0;
        }
        let index = self.find_segment(distance);
        let segment = &self.segments[index];

        match segment.kind {
            TrimSegmentKind::Line { from, to } => {
                let mut previous_distance = 0.0;
                if index > 0 {
                    previous_distance = self.segments[index - 1].distance;
                }
                let denominator = segment.distance - previous_distance;
                let rel_d = if denominator == 0.0 {
                    0.0
                } else {
                    (distance - previous_distance) / denominator
                };
                let tan = normalized_vector((to.0 - from.0, to.1 - from.1));
                (lerp_point(from, to, rel_d), tan)
            }
            TrimSegmentKind::Quad { p0, p1, p2 } => {
                quad_position_tangent([p0, p1, p2], self.compute_t(index, distance))
            }
            TrimSegmentKind::Cubic { p0, p1, p2, p3 } => {
                cubic_position_tangent([p0, p1, p2, p3], self.compute_t(index, distance))
            }
        }
    }
}

impl RuntimeContourMeasure {
    pub fn from_commands(commands: &[RuntimePathCommand]) -> Vec<Self> {
        TrimContour::from_commands(commands)
            .into_iter()
            .map(|contour| Self { contour })
            .collect()
    }

    pub fn length(&self) -> f32 {
        self.contour.length
    }

    pub fn is_closed(&self) -> bool {
        self.contour.is_closed
    }

    pub fn at_distance(&self, distance: f32) -> RuntimePathSample {
        let (pos, tan) = self.contour.position_tangent_at_distance(distance);
        RuntimePathSample {
            pos,
            tan,
            ..RuntimePathSample::default()
        }
    }

    pub fn append_segment(
        &self,
        start: f32,
        end: f32,
        destination: &mut RawPath,
        start_with_move: bool,
    ) {
        self.contour
            .get_segment_into(start, end, destination, start_with_move);
    }

    pub fn segment(&self, start: f32, end: f32, start_with_move: bool) -> RawPath {
        let mut destination = RawPath::new();
        self.append_segment(start, end, &mut destination, start_with_move);
        destination
    }
}

impl TrimSegmentKind {
    fn extract<S: TrimSegmentSink>(
        self,
        destination: &mut S,
        start_t: f32,
        end_t: f32,
        move_to: bool,
    ) {
        match self {
            Self::Line { from, to } => {
                let start = lerp_point(from, to, start_t);
                let end = lerp_point(from, to, end_t);
                if move_to {
                    destination.move_to(start);
                }
                destination.line_to(end);
            }
            Self::Quad { p0, p1, p2 } => {
                let [start, control, end] = quad_extract([p0, p1, p2], start_t, end_t);
                if move_to {
                    destination.move_to(start);
                }
                destination.quad_to(start, control, end);
            }
            Self::Cubic { p0, p1, p2, p3 } => {
                let [start, control_1, control_2, end] =
                    cubic_extract([p0, p1, p2, p3], start_t, end_t);
                if move_to {
                    destination.move_to(start);
                }
                destination.cubic_to(control_1, control_2, end);
            }
        }
    }

    fn extract_full<S: TrimSegmentSink>(self, destination: &mut S) {
        match self {
            Self::Line { to, .. } => destination.line_to(to),
            Self::Quad { p0, p1, p2 } => destination.quad_to(p0, p1, p2),
            Self::Cubic { p1, p2, p3, .. } => destination.cubic_to(p1, p2, p3),
        }
    }
}

trait TrimSegmentSink {
    fn move_to(&mut self, point: (f32, f32));
    fn line_to(&mut self, point: (f32, f32));
    fn quad_to(&mut self, start: (f32, f32), control: (f32, f32), end: (f32, f32));
    fn cubic_to(&mut self, control_1: (f32, f32), control_2: (f32, f32), end: (f32, f32));
}

impl TrimSegmentSink for RawPath {
    fn move_to(&mut self, point: (f32, f32)) {
        self.move_to(point.0, point.1);
    }

    fn line_to(&mut self, point: (f32, f32)) {
        self.line_to(point.0, point.1);
    }

    fn quad_to(&mut self, _start: (f32, f32), control: (f32, f32), end: (f32, f32)) {
        self.quad_to(control.0, control.1, end.0, end.1);
    }

    fn cubic_to(&mut self, control_1: (f32, f32), control_2: (f32, f32), end: (f32, f32)) {
        self.cubic_to(
            control_1.0,
            control_1.1,
            control_2.0,
            control_2.1,
            end.0,
            end.1,
        );
    }
}

impl TrimSegmentSink for Vec<RuntimePathCommand> {
    fn move_to(&mut self, point: (f32, f32)) {
        self.push(RuntimePathCommand::Move {
            x: point.0,
            y: point.1,
        });
    }

    fn line_to(&mut self, point: (f32, f32)) {
        self.push(RuntimePathCommand::Line {
            x: point.0,
            y: point.1,
        });
    }

    fn quad_to(&mut self, start: (f32, f32), control: (f32, f32), end: (f32, f32)) {
        let control_1 = (
            start.0 + (control.0 - start.0) * (2.0 / 3.0),
            start.1 + (control.1 - start.1) * (2.0 / 3.0),
        );
        let control_2 = (
            end.0 + (control.0 - end.0) * (2.0 / 3.0),
            end.1 + (control.1 - end.1) * (2.0 / 3.0),
        );
        self.cubic_to(control_1, control_2, end);
    }

    fn cubic_to(&mut self, control_1: (f32, f32), control_2: (f32, f32), end: (f32, f32)) {
        self.push(RuntimePathCommand::Cubic {
            x1: control_1.0,
            y1: control_1.1,
            x2: control_2.0,
            y2: control_2.1,
            x3: end.0,
            y3: end.1,
        });
    }
}

const TRIM_CONTOUR_EPSILON: f32 = 1.0 / 4096.0;
const TRIM_CONTOUR_DEFAULT_TOLERANCE: f32 = 0.5;
const TRIM_CONTOUR_DEFAULT_INV_TOLERANCE: f32 = 1.0 / TRIM_CONTOUR_DEFAULT_TOLERANCE;
const TRIM_CONTOUR_MAX_SEGMENTS: u32 = 100;
const TRIM_CONTOUR_DOT30_SCALE: f32 = (1u32 << 30) as f32;
const TRIM_CONTOUR_MAX_DOT30: u32 = (1u32 << 30) - 1;
const TRIM_CONTOUR_INV_MAX_DOT30: f32 = 1.0 / TRIM_CONTOUR_MAX_DOT30 as f32;

fn trim_contour_dot30_t(t: f32) -> f32 {
    ((t * TRIM_CONTOUR_DOT30_SCALE) as u32) as f32 * TRIM_CONTOUR_INV_MAX_DOT30
}
