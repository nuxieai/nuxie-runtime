// Direct source-correspondence owner for pinned `src/math/path_measure.cpp`.
#[derive(Debug, Clone)]
pub struct RuntimePathMeasure {
    contours: Vec<TrimContour>,
    length: f32,
    raw_is_closed: bool,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct RuntimePathSample {
    pub pos: (f32, f32),
    pub tan: (f32, f32),
}

impl RuntimePathMeasure {
    pub fn from_commands(commands: &[RuntimePathCommand]) -> Self {
        Self::from_commands_with_inv_tolerance(commands, TRIM_CONTOUR_DEFAULT_INV_TOLERANCE)
    }

    pub(crate) fn from_raw_path(path: &RawPath) -> Self {
        let contours =
            TrimContour::from_raw_path_with_inv_tolerance(path, TRIM_CONTOUR_DEFAULT_INV_TOLERANCE);
        let length = contours.iter().map(|contour| contour.length).sum();
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
        let tolerance = if tolerance.is_finite() {
            tolerance.max(min_tolerance)
        } else {
            TRIM_CONTOUR_DEFAULT_TOLERANCE
        };
        Self::from_commands_with_inv_tolerance(commands, 1.0 / tolerance)
    }

    fn from_commands_with_inv_tolerance(
        commands: &[RuntimePathCommand],
        inv_tolerance: f32,
    ) -> Self {
        let contours = TrimContour::from_commands_with_inv_tolerance(commands, inv_tolerance);
        let length = contours.iter().map(|contour| contour.length).sum();
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

    pub(crate) fn at_percentage(&self, percentage_distance: f32) -> RuntimePathSample {
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
                return RuntimePathSample { pos, tan };
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

        let start_distance = start_distance.clamp(0.0, self.length);
        let end_distance = end_distance.clamp(0.0, self.length);
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
                let local_start = (start_distance - contour_start).max(0.0);
                let local_end = (end_distance - contour_start).min(contour_length);
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

impl RuntimePathMeasure {
    pub fn segment(&self, start: f32, end: f32, start_with_move: bool) -> RawPath {
        let mut commands = Vec::new();
        self.get_segment(start, end, &mut commands, start_with_move);
        runtime_raw_path_from_commands(&commands)
    }
}
