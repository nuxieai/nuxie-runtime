// Direct source-correspondence owner for pinned `src/math/path_measure.cpp`.
#[derive(Debug, Clone, Default)]
pub struct RuntimePathMeasure {
    contours: Vec<TrimContour>,
    length: f32,
    raw_is_closed: bool,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct RuntimePathSample {
    pub pos: (f32, f32),
    pub tan: (f32, f32),
    /// Distance along the complete path supplied to `at_distance`.
    pub distance: f32,
    /// Squared distance to a point projected to the path.
    ///
    /// `PathMeasure::atDistance` initializes this to zero. Projection owners
    /// may replace it when they retain the corresponding result payload.
    pub sq_distance_to_point: f32,
}

impl RuntimePathMeasure {
    pub fn from_commands(commands: &[RuntimePathCommand]) -> Self {
        Self::from_commands_with_inv_tolerance(commands, TRIM_CONTOUR_DEFAULT_INV_TOLERANCE)
    }

    pub(crate) fn from_raw_path(path: &RawPath) -> Self {
        Self::from_raw_path_with_tolerance(path, TRIM_CONTOUR_DEFAULT_TOLERANCE)
    }

    pub(crate) fn from_raw_path_with_tolerance(path: &RawPath, tolerance: f32) -> Self {
        let inv_tolerance = 1.0 / path_measure_cpp_std_max(tolerance, 1.0 / 16.0);
        let contours = TrimContour::from_raw_path_with_inv_tolerance(path, inv_tolerance);
        let mut length = 0.0;
        for contour in &contours {
            length += contour.length;
        }
        let raw_is_closed = matches!(path.verbs().last(), Some(RenderPathVerb::Close));
        Self {
            contours,
            length,
            raw_is_closed,
        }
    }

    pub(crate) fn from_commands_with_tolerance(
        commands: &[RuntimePathCommand],
        tolerance: f32,
    ) -> Self {
        let min_tolerance = 1.0 / 16.0;
        let tolerance = path_measure_cpp_std_max(tolerance, min_tolerance);
        Self::from_commands_with_inv_tolerance(commands, 1.0 / tolerance)
    }

    fn from_commands_with_inv_tolerance(
        commands: &[RuntimePathCommand],
        inv_tolerance: f32,
    ) -> Self {
        let contours = TrimContour::from_commands_with_inv_tolerance(commands, inv_tolerance);
        let mut length = 0.0;
        for contour in &contours {
            length += contour.length;
        }
        let raw_is_closed = matches!(commands.last(), Some(RuntimePathCommand::Close));
        Self {
            contours,
            length,
            raw_is_closed,
        }
    }

    pub fn length(&self) -> f32 {
        self.length
    }

    pub fn at_percentage(&self, percentage_distance: f32) -> RuntimePathSample {
        let mut in_range_percentage = percentage_distance % 1.0;
        if in_range_percentage < 0.0 {
            in_range_percentage += 1.0;
        }
        if percentage_distance != 0.0 && in_range_percentage == 0.0 {
            in_range_percentage = 1.0;
        }
        self.at_distance(self.length * in_range_percentage)
    }

    pub fn at_distance(&self, distance: f32) -> RuntimePathSample {
        let mut current_distance = distance;
        for contour in &self.contours {
            let contour_length = contour.length;
            if current_distance - contour_length <= 0.0 {
                let (pos, tan) = contour.position_tangent_at_distance(current_distance);
                return RuntimePathSample {
                    pos,
                    tan,
                    distance,
                    sq_distance_to_point: 0.0,
                };
            }
            current_distance -= contour_length;
        }
        RuntimePathSample::default()
    }

    fn get_segment(
        &self,
        start_distance: f32,
        end_distance: f32,
        commands: &mut Vec<RuntimePathCommand>,
        start_with_move: bool,
    ) {
        if self.contours.is_empty() {
            return;
        }

        let start_distance =
            path_measure_cpp_std_max(0.0, path_measure_cpp_std_min(start_distance, self.length));
        let end_distance =
            path_measure_cpp_std_max(0.0, path_measure_cpp_std_min(end_distance, self.length));
        if start_distance >= end_distance {
            return;
        }

        let mut current_distance = 0.0;
        let mut is_first_segment = true;
        for contour in &self.contours {
            let contour_length = contour.length;
            let contour_start = current_distance;
            let contour_end = current_distance + contour_length;

            if contour_end > start_distance && contour_start < end_distance {
                let local_start = path_measure_cpp_std_max(0.0, start_distance - contour_start);
                let local_end =
                    path_measure_cpp_std_min(contour_length, end_distance - contour_start);
                contour.get_segment(
                    local_start,
                    local_end,
                    commands,
                    !is_first_segment || start_with_move,
                );
                is_first_segment = false;
            }

            current_distance += contour_length;
            if current_distance >= end_distance {
                break;
            }
        }
    }

    pub fn is_closed(&self) -> bool {
        self.contours.len() == 1 && self.contours[0].is_closed
    }

    pub(crate) fn raw_is_closed(&self) -> bool {
        self.raw_is_closed
    }
}

// Literal two-argument `std::min`/`std::max` comparison order. This preserves
// the pinned NaN and signed-zero behavior; Rust's float helpers do not.
fn path_measure_cpp_std_min(first: f32, second: f32) -> f32 {
    if second < first {
        second
    } else {
        first
    }
}

fn path_measure_cpp_std_max(first: f32, second: f32) -> f32 {
    if first < second {
        second
    } else {
        first
    }
}

impl RuntimePathMeasure {
    pub fn append_segment(
        &self,
        start: f32,
        end: f32,
        destination: &mut RawPath,
        start_with_move: bool,
    ) {
        let mut commands = Vec::new();
        self.get_segment(start, end, &mut commands, start_with_move);
        runtime_append_path_commands(destination, &commands);
    }

    pub fn segment(&self, start: f32, end: f32, start_with_move: bool) -> RawPath {
        let mut destination = RawPath::new();
        self.append_segment(start, end, &mut destination, start_with_move);
        destination
    }
}
