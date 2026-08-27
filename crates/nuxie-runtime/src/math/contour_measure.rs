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
                    let p3 = (point.x, point.y);
                    let p1 = (
                        p0.0 + (control.x - p0.0) * (2.0 / 3.0),
                        p0.1 + (control.y - p0.1) * (2.0 / 3.0),
                    );
                    let p2 = (
                        p3.0 + (control.x - p3.0) * (2.0 / 3.0),
                        p3.1 + (control.y - p3.1) * (2.0 / 3.0),
                    );
                    raw_segments.push(TrimSegmentKind::Cubic { p0, p1, p2, p3 });
                    current = Some(p3);
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
        mut start_distance: f32,
        mut end_distance: f32,
        commands: &mut Vec<RuntimePathCommand>,
        start_with_move: bool,
    ) {
        start_distance = start_distance.max(0.0);
        end_distance = end_distance.min(self.length);
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
                .extract(commands, start_t, end_t, start_with_move);
            return;
        }

        start.kind.extract(commands, start_t, 1.0, start_with_move);

        let mut original_index = start.original_index + 1;
        while original_index < end.original_index {
            if let Some(segment) = self.first_segment_for_original(original_index) {
                segment.kind.extract_full(commands);
            }
            original_index += 1;
        }

        end.kind.extract(commands, 0.0, end_t, false);
    }

    fn find_segment(&self, distance: f32) -> usize {
        self.segments
            .iter()
            .position(|segment| segment.distance >= distance)
            .unwrap_or_else(|| self.segments.len() - 1)
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
        let distance = distance.clamp(0.0, self.length);
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
        let mut commands = Vec::new();
        self.contour
            .get_segment(start, end, &mut commands, start_with_move);
        runtime_append_path_commands(destination, &commands);
    }

    pub fn segment(&self, start: f32, end: f32, start_with_move: bool) -> RawPath {
        let mut destination = RawPath::new();
        self.append_segment(start, end, &mut destination, start_with_move);
        destination
    }
}

impl TrimSegmentKind {
    fn extract(
        self,
        commands: &mut Vec<RuntimePathCommand>,
        start_t: f32,
        end_t: f32,
        move_to: bool,
    ) {
        match self {
            Self::Line { from, to } => {
                let start = weighted_lerp_point(from, to, start_t);
                let end = weighted_lerp_point(from, to, end_t);
                if move_to {
                    commands.push(RuntimePathCommand::Move {
                        x: start.0,
                        y: start.1,
                    });
                }
                commands.push(RuntimePathCommand::Line { x: end.0, y: end.1 });
            }
            Self::Cubic { p0, p1, p2, p3 } => {
                let [start, control_1, control_2, end] =
                    cubic_extract([p0, p1, p2, p3], start_t, end_t);
                if move_to {
                    commands.push(RuntimePathCommand::Move {
                        x: start.0,
                        y: start.1,
                    });
                }
                commands.push(RuntimePathCommand::Cubic {
                    x1: control_1.0,
                    y1: control_1.1,
                    x2: control_2.0,
                    y2: control_2.1,
                    x3: end.0,
                    y3: end.1,
                });
            }
        }
    }

    fn extract_full(self, commands: &mut Vec<RuntimePathCommand>) {
        match self {
            Self::Line { to, .. } => commands.push(RuntimePathCommand::Line { x: to.0, y: to.1 }),
            Self::Cubic { p1, p2, p3, .. } => commands.push(RuntimePathCommand::Cubic {
                x1: p1.0,
                y1: p1.1,
                x2: p2.0,
                y2: p2.1,
                x3: p3.0,
                y3: p3.1,
            }),
        }
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
